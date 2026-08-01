#![allow(clippy::needless_borrow, clippy::too_many_arguments)]

use std::future::Future;

use serde::Serialize;
use tauri::{AppHandle, Manager};
use tokio_util::sync::CancellationToken;

use crate::db::get_pool;
use crate::error::{AppError, AppErrorKind, AppResult};
use crate::ingest_provenance::{
    create_telegram_takeout_batch, finalize_ingest_batch, mark_takeout_export_dc_attempted,
    mark_takeout_export_dc_fallback, mark_takeout_migrated_history_deferred,
    mark_takeout_migrated_history_imported, mark_takeout_migrated_small_group_scope,
    mark_takeout_only_my_messages_fallback, record_ingest_batch_warning,
    update_takeout_max_message_id, update_takeout_resolved_peer, update_takeout_session_started,
    update_takeout_split_metadata, CreateTelegramTakeoutBatch, TerminalBatchStatus,
    TAKEOUT_HISTORY_SCOPE_CURRENT, TAKEOUT_HISTORY_SCOPE_MIGRATED_SMALL_GROUP,
};
use crate::source_ingest::{SourceIngestGuard, SourceIngestKind, SourceIngestLocks};
use crate::sources::{
    finalize_sync, load_source, require_source_identity_ready, resolve_and_refresh_peer,
    SourceIdentityRepairState, TelegramSourceKind, MIGRATED_HISTORY_STATUS_AVAILABLE,
    TELEGRAM_KIND_GROUP, TELEGRAM_KIND_SUPERGROUP,
};
use crate::telegram::TelegramState;
use crate::telegram_impl::TelegramClientHandle;
use crate::telegram_impl::{
    MessageRange, TakeoutAttempt, TakeoutCount, TakeoutFallback, TakeoutFallbackKind, TakeoutPage,
    TakeoutPeer, TakeoutTransport,
};
use crate::time::now_secs;
mod forum_topics;
pub(crate) mod migrated_history;
mod recovery;
mod state;
#[allow(dead_code)]
mod validation_diagnostics;

use forum_topics::refresh_forum_topics_after_completed_takeout;
use recovery::list_takeout_import_recovery_states_for_sources;
pub(crate) use recovery::TakeoutImportRecoveryState;
pub use state::TakeoutImportState;
use state::{
    emit_takeout_import_event, update_and_emit, CancelTakeoutImportResponse,
    StartTakeoutImportResponse, TakeoutImportJobRecord, PHASE_CANCELLED, PHASE_COMPLETED,
    PHASE_COUNTING, PHASE_FAILED, PHASE_FINISHING_TAKEOUT, PHASE_IMPORTING_HISTORY,
    PHASE_LOADING_SPLITS, PHASE_RESOLVING_SOURCE, PHASE_STARTING_TAKEOUT, PHASE_VALIDATING_PEER,
    STATUS_CANCELLED, STATUS_COMPLETED, STATUS_FAILED, STATUS_RUNNING,
};

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub(crate) struct TakeoutExportDcSpikeResult {
    pub(crate) source_id: i64,
    pub(crate) account_id: i64,
    pub(crate) telegram_source_subtype: String,
    pub(crate) home_dc_id: i32,
    pub(crate) export_dc_id: i32,
    pub(crate) used_export_dc: bool,
    pub(crate) fallback_used: bool,
    pub(crate) takeout_id: i64,
    pub(crate) split_count: usize,
    pub(crate) warnings: Vec<String>,
}

#[tauri::command]
pub async fn start_takeout_source_import(
    handle: AppHandle,
    repair_state: tauri::State<'_, SourceIdentityRepairState>,
    state: tauri::State<'_, TakeoutImportState>,
    source_id: i64,
) -> AppResult<StartTakeoutImportResponse> {
    require_source_identity_ready(repair_state.inner()).await?;
    let pool = get_pool(&handle).await?;
    let source = load_source(&pool, source_id).await?;
    let account_id = source.account_id.ok_or_else(|| {
        AppError::validation(format!("Source {source_id} is not linked to an account"))
    })?;
    let telegram_source_subtype = load_takeout_source_subtype(&pool, source.id).await?;
    let ingest_locks = handle.state::<SourceIngestLocks>();
    let (record, ingest_guard) = create_locked_takeout_start_records(
        &pool,
        &ingest_locks,
        state.inner(),
        source_id,
        account_id,
        telegram_source_subtype,
    )
    .await?;
    emit_takeout_import_event(&handle, &record);

    let job_id = record.job_id.clone();
    let task_handle = handle.clone();
    tauri::async_runtime::spawn(async move {
        run_takeout_import_job(task_handle, job_id, ingest_guard).await;
    });

    Ok(StartTakeoutImportResponse {
        job_id: record.job_id,
    })
}

#[tauri::command]
pub async fn start_takeout_migrated_history_import(
    handle: AppHandle,
    repair_state: tauri::State<'_, SourceIdentityRepairState>,
    state: tauri::State<'_, TakeoutImportState>,
    source_id: i64,
) -> AppResult<StartTakeoutImportResponse> {
    require_source_identity_ready(repair_state.inner()).await?;
    let pool = get_pool(&handle).await?;
    let source = load_source(&pool, source_id).await?;
    let account_id = source.account_id.ok_or_else(|| {
        AppError::validation(format!("Source {source_id} is not linked to an account"))
    })?;
    let telegram_source_subtype = load_takeout_source_subtype(&pool, source.id).await?;
    let ingest_locks = handle.state::<SourceIngestLocks>();
    let (record, ingest_guard) = create_locked_migrated_history_start_records(
        &pool,
        &ingest_locks,
        state.inner(),
        source_id,
        account_id,
        telegram_source_subtype,
    )
    .await?;
    emit_takeout_import_event(&handle, &record);

    let job_id = record.job_id.clone();
    let task_handle = handle.clone();
    tauri::async_runtime::spawn(async move {
        run_takeout_migrated_history_import_job(task_handle, job_id, ingest_guard).await;
    });

    Ok(StartTakeoutImportResponse {
        job_id: record.job_id,
    })
}

async fn create_locked_takeout_start_records(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    ingest_locks: &SourceIngestLocks,
    state: &TakeoutImportState,
    source_id: i64,
    account_id: i64,
    source_subtype: String,
) -> AppResult<(TakeoutImportJobRecord, SourceIngestGuard)> {
    let ingest_guard = ingest_locks
        .try_acquire(source_id, SourceIngestKind::TakeoutImport)
        .await?;
    let batch_id = create_telegram_takeout_batch(
        pool,
        CreateTelegramTakeoutBatch {
            source_id,
            account_id,
            source_subtype,
        },
    )
    .await?;
    let record = state
        .create_job(
            source_id,
            account_id,
            batch_id,
            TAKEOUT_HISTORY_SCOPE_CURRENT,
        )
        .await?;
    Ok((record, ingest_guard))
}

async fn create_locked_migrated_history_start_records(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    ingest_locks: &SourceIngestLocks,
    state: &TakeoutImportState,
    source_id: i64,
    account_id: i64,
    source_subtype: String,
) -> AppResult<(TakeoutImportJobRecord, SourceIngestGuard)> {
    let capability = migrated_history::load_migrated_history_capability(pool, source_id).await?;
    let is_available = capability
        .as_ref()
        .is_some_and(|capability| capability.status == MIGRATED_HISTORY_STATUS_AVAILABLE);
    if !is_available {
        return Err(AppError::validation("migrated_history_not_detected"));
    }
    if source_subtype != TELEGRAM_KIND_SUPERGROUP {
        return Err(AppError::validation(
            "migrated_history_not_detected: only Telegram supergroups can have migrated small-group history",
        ));
    }

    let ingest_guard = ingest_locks
        .try_acquire(source_id, SourceIngestKind::TakeoutImport)
        .await?;
    let batch_id = create_telegram_takeout_batch(
        pool,
        CreateTelegramTakeoutBatch {
            source_id,
            account_id,
            source_subtype,
        },
    )
    .await?;
    mark_takeout_migrated_small_group_scope(pool, batch_id).await?;
    let record = state
        .create_job(
            source_id,
            account_id,
            batch_id,
            TAKEOUT_HISTORY_SCOPE_MIGRATED_SMALL_GROUP,
        )
        .await?;
    Ok((record, ingest_guard))
}

fn migrated_history_detected_warning() -> String {
    "Migrated small-group history detected; current Takeout keeps it deferred until explicit historical import."
        .to_string()
}

async fn finalize_terminal_batch_best_effort(
    handle: &AppHandle,
    batch_id: i64,
    status: TerminalBatchStatus,
    terminal_error: Option<&str>,
) {
    if let Ok(pool) = get_pool(handle).await {
        let _ = finalize_ingest_batch(&pool, batch_id, status, terminal_error).await;
    }
}

#[tauri::command]
pub async fn cancel_takeout_source_import(
    handle: AppHandle,
    state: tauri::State<'_, TakeoutImportState>,
    job_id: String,
) -> AppResult<CancelTakeoutImportResponse> {
    let Some(record) = state.request_cancel(&job_id).await else {
        return Ok(CancelTakeoutImportResponse { cancelled: false });
    };
    emit_takeout_import_event(&handle, &record);
    Ok(CancelTakeoutImportResponse { cancelled: true })
}

#[tauri::command]
pub async fn list_takeout_source_import_jobs(
    state: tauri::State<'_, TakeoutImportState>,
) -> AppResult<Vec<TakeoutImportJobRecord>> {
    Ok(state.list_jobs().await)
}

#[cfg(dev)]
#[tauri::command]
pub async fn seed_takeout_cancellation_smoke_fixture(
    handle: AppHandle,
    state: tauri::State<'_, TakeoutImportState>,
) -> AppResult<TakeoutImportJobRecord> {
    state::seed_takeout_cancellation_smoke_fixture(handle, state.inner()).await
}

#[cfg(dev)]
#[tauri::command]
pub async fn clear_takeout_cancellation_smoke_fixture(
    state: tauri::State<'_, TakeoutImportState>,
) -> AppResult<usize> {
    state::clear_takeout_cancellation_smoke_fixture(state.inner()).await
}

#[tauri::command]
pub async fn list_takeout_import_recovery_states(
    handle: AppHandle,
    state: tauri::State<'_, TakeoutImportState>,
) -> AppResult<Vec<TakeoutImportRecoveryState>> {
    let pool = get_pool(&handle).await?;
    list_takeout_import_recovery_states_for_sources(&pool, state.inner(), None).await
}

#[tauri::command]
pub async fn run_takeout_export_dc_spike(
    handle: AppHandle,
    repair_state: tauri::State<'_, SourceIdentityRepairState>,
    state: tauri::State<'_, TelegramState>,
    source_id: i64,
) -> AppResult<TakeoutExportDcSpikeResult> {
    require_source_identity_ready(repair_state.inner()).await?;
    let pool = get_pool(&handle).await?;
    let source = load_source(&pool, source_id).await?;
    let account_id = source.account_id.ok_or_else(|| {
        AppError::validation(format!("Source {source_id} is not linked to an account"))
    })?;
    let telegram_source_subtype = load_takeout_source_subtype(&pool, source.id).await?;
    let client_handle = state.authorized_client(account_id).await?;

    run_export_dc_spike_for_handle(
        source.id,
        account_id,
        &telegram_source_subtype,
        client_handle,
    )
    .await
}

macro_rules! run_spike_transport_step {
    ($transport:ident, $warnings:ident, $fallback_used:ident, $operation:expr) => {{
        let result = $operation.await;
        for fallback in $transport.drain_fallbacks() {
            push_warning_once(&mut $warnings, fallback.warning().to_string());
            $fallback_used |= fallback.kind() == TakeoutFallbackKind::ExportDc;
        }
        result
    }};
}

async fn run_export_dc_spike_for_handle(
    source_id: i64,
    account_id: i64,
    telegram_source_subtype: &str,
    handle: TelegramClientHandle,
) -> AppResult<TakeoutExportDcSpikeResult> {
    handle.takeout_self_check().await?;
    let mut transport = handle.prepare_takeout().await?;
    let attempt = transport.export_dc_attempt();
    let mut warnings = Vec::new();
    let mut fallback_used = false;
    let takeout_id = run_spike_transport_step!(
        transport,
        warnings,
        fallback_used,
        transport.init(telegram_source_subtype)
    )?;
    let (split_count, _) = run_spike_transport_step!(
        transport,
        warnings,
        fallback_used,
        transport.message_ranges(takeout_id, telegram_source_subtype)
    )?;
    run_spike_transport_step!(
        transport,
        warnings,
        fallback_used,
        transport.finish(takeout_id, true)
    )?;

    Ok(TakeoutExportDcSpikeResult {
        source_id,
        account_id,
        telegram_source_subtype: telegram_source_subtype.to_string(),
        home_dc_id: attempt.home_dc_id(),
        export_dc_id: attempt.export_dc_id(),
        used_export_dc: !fallback_used,
        fallback_used,
        takeout_id,
        split_count: usize::try_from(split_count).unwrap_or(usize::MAX),
        warnings,
    })
}

async fn run_takeout_import_job(
    handle: AppHandle,
    job_id: String,
    ingest_guard: SourceIngestGuard,
) {
    let takeout_state = handle.state::<TakeoutImportState>();

    let Some(running_record) = takeout_state
        .update_job(&job_id, |job| {
            job.status = STATUS_RUNNING.to_string();
            job.phase = PHASE_RESOLVING_SOURCE.to_string();
            job.message = Some("Preparing Takeout import.".to_string());
        })
        .await
    else {
        drop(ingest_guard);
        return;
    };
    emit_takeout_import_event(&handle, &running_record);
    let batch_id = running_record.batch_id;

    if takeout_state.is_cancel_requested(&job_id).await {
        finalize_terminal_batch_best_effort(
            &handle,
            batch_id,
            TerminalBatchStatus::Cancelled,
            None,
        )
        .await;
        if let Some(record) = takeout_state
            .finish_job(&job_id, |job| {
                job.status = STATUS_CANCELLED.to_string();
                job.phase = PHASE_CANCELLED.to_string();
                job.message = Some("Takeout import cancelled.".to_string());
            })
            .await
        {
            emit_takeout_import_event(&handle, &record);
        }
        drop(ingest_guard);
        return;
    }

    match run_takeout_source_import(&handle, &job_id, batch_id).await {
        Ok(outcome) => {
            if let Some(record) = takeout_state
                .finish_job(&job_id, |job| {
                    job.status = STATUS_COMPLETED.to_string();
                    job.phase = PHASE_COMPLETED.to_string();
                    job.message = Some(format!(
                        "Takeout import completed. Inserted {}, skipped {}.",
                        outcome.inserted, outcome.skipped
                    ));
                    job.inserted = outcome.inserted;
                    job.skipped = outcome.skipped;
                    job.progress_current = outcome.progress_total;
                    job.progress_total = outcome.progress_total;
                    job.warnings = outcome.warnings;
                })
                .await
            {
                emit_takeout_import_event(&handle, &record);
            }
        }
        Err(error) => {
            if takeout_state.is_cancel_requested(&job_id).await {
                finalize_terminal_batch_best_effort(
                    &handle,
                    batch_id,
                    TerminalBatchStatus::Cancelled,
                    None,
                )
                .await;
                if let Some(record) = takeout_state
                    .finish_job(&job_id, |job| {
                        job.status = STATUS_CANCELLED.to_string();
                        job.phase = PHASE_CANCELLED.to_string();
                        job.message = Some("Takeout import cancelled.".to_string());
                    })
                    .await
                {
                    emit_takeout_import_event(&handle, &record);
                }
            } else {
                let terminal_error = error.to_string();
                finalize_terminal_batch_best_effort(
                    &handle,
                    batch_id,
                    TerminalBatchStatus::Failed,
                    Some(&terminal_error),
                )
                .await;
                if let Some(record) = takeout_state
                    .finish_job(&job_id, |job| {
                        job.status = STATUS_FAILED.to_string();
                        job.phase = PHASE_FAILED.to_string();
                        job.message = None;
                        job.error = Some(terminal_error.clone());
                    })
                    .await
                {
                    emit_takeout_import_event(&handle, &record);
                }
            }
        }
    }
    drop(ingest_guard);
}

async fn run_takeout_migrated_history_import_job(
    handle: AppHandle,
    job_id: String,
    ingest_guard: SourceIngestGuard,
) {
    let takeout_state = handle.state::<TakeoutImportState>();
    let Some(running_record) = takeout_state
        .update_job(&job_id, |job| {
            job.status = STATUS_RUNNING.to_string();
            job.phase = PHASE_VALIDATING_PEER.to_string();
            job.message = Some("Validating migrated history availability.".to_string());
        })
        .await
    else {
        drop(ingest_guard);
        return;
    };
    emit_takeout_import_event(&handle, &running_record);
    let batch_id = running_record.batch_id;
    let source_id = running_record.source_id;
    match run_takeout_migrated_history_import(&handle, &job_id, batch_id).await {
        Ok(outcome) => {
            if let Some(record) = takeout_state
                .finish_job(&job_id, |job| {
                    job.status = STATUS_COMPLETED.to_string();
                    job.phase = PHASE_COMPLETED.to_string();
                    job.message = Some(format!(
                        "Migrated history import completed. Inserted {}, skipped {}.",
                        outcome.inserted, outcome.skipped
                    ));
                    job.inserted = outcome.inserted;
                    job.skipped = outcome.skipped;
                    job.progress_current = outcome.progress_total;
                    job.progress_total = outcome.progress_total;
                    job.warnings = outcome.warnings;
                })
                .await
            {
                emit_takeout_import_event(&handle, &record);
            }
        }
        Err(error) => {
            if error.kind == AppErrorKind::Conflict
                && error.message == migrated_history::unavailable_error().message
            {
                if let Ok(pool) = get_pool(&handle).await {
                    let _ = migrated_history::mark_migrated_history_unavailable(
                        &pool,
                        source_id,
                        migrated_history::MIGRATED_HISTORY_REASON_REVALIDATION_FAILED,
                        now_secs(),
                    )
                    .await;
                }
            }
            let terminal_error = error.to_string();
            finalize_terminal_batch_best_effort(
                &handle,
                batch_id,
                TerminalBatchStatus::Failed,
                Some(&terminal_error),
            )
            .await;
            if let Some(record) = takeout_state
                .finish_job(&job_id, |job| {
                    job.status = STATUS_FAILED.to_string();
                    job.phase = PHASE_FAILED.to_string();
                    job.message = None;
                    job.error = Some(terminal_error.clone());
                })
                .await
            {
                emit_takeout_import_event(&handle, &record);
            }
        }
    }
    drop(ingest_guard);
}

struct TakeoutImportOutcome {
    inserted: i64,
    skipped: i64,
    progress_total: Option<i64>,
    warnings: Vec<String>,
}

enum FallbackRecordState {
    Empty,
    Pending(Vec<TakeoutFallback>),
    InFlight(Vec<TakeoutFallback>),
    Recorded,
    Failed {
        fallbacks: Vec<TakeoutFallback>,
        error: AppError,
    },
}

enum StepOutcome<T> {
    Remote(AppResult<T>),
    MetadataStopped(AppResult<T>),
    AttemptStopped {
        remote: AppResult<T>,
        error: AppError,
    },
}

#[cfg(test)]
#[derive(Clone, Copy)]
enum HistoryTransitionCommand {
    History,
    Search,
    Drain,
}

#[cfg(test)]
enum HistoryTransitionResponse<T> {
    Remote {
        result: AppResult<T>,
        fallbacks: Vec<TakeoutFallback>,
    },
    Drained(Vec<TakeoutFallback>),
}

fn queue_drained_fallbacks(
    state: &mut FallbackRecordState,
    mut fallbacks: Vec<TakeoutFallback>,
) -> AppResult<()> {
    if fallbacks.is_empty() {
        return Ok(());
    }
    match state {
        FallbackRecordState::Empty | FallbackRecordState::Recorded => {
            *state = FallbackRecordState::Pending(fallbacks);
            Ok(())
        }
        FallbackRecordState::Pending(pending) => {
            pending.append(&mut fallbacks);
            Ok(())
        }
        FallbackRecordState::InFlight(_) | FallbackRecordState::Failed { .. } => Err(
            AppError::internal("Takeout fallback metadata arrived in an invalid recorder state"),
        ),
    }
}

#[cfg(test)]
async fn record_pending_once<Record>(state: &mut FallbackRecordState, record: &mut Record)
where
    Record: AsyncFnMut(Vec<TakeoutFallback>) -> AppResult<()>,
{
    let FallbackRecordState::Pending(fallbacks) =
        std::mem::replace(state, FallbackRecordState::Empty)
    else {
        return;
    };
    *state = FallbackRecordState::InFlight(fallbacks);
    let owned = match state {
        FallbackRecordState::InFlight(fallbacks) => fallbacks.clone(),
        _ => unreachable!("fallback recorder must be in flight"),
    };
    match record(owned).await {
        Ok(()) => *state = FallbackRecordState::Recorded,
        Err(error) => {
            let fallbacks = match std::mem::replace(state, FallbackRecordState::Empty) {
                FallbackRecordState::InFlight(fallbacks) => fallbacks,
                _ => unreachable!("fallback recorder must retain in-flight values"),
            };
            *state = FallbackRecordState::Failed { fallbacks, error };
        }
    }
}

#[cfg(test)]
async fn record_after_select_once<Record>(state: &mut FallbackRecordState, record: &mut Record)
where
    Record: AsyncFnMut(Vec<TakeoutFallback>) -> AppResult<()>,
{
    if matches!(state, FallbackRecordState::Pending(_)) {
        record_pending_once(state, record).await;
        return;
    }
    let FallbackRecordState::InFlight(fallbacks) = state else {
        return;
    };
    let retained = fallbacks.clone();
    match record(retained).await {
        Ok(()) => *state = FallbackRecordState::Recorded,
        Err(error) => {
            let fallbacks = match std::mem::replace(state, FallbackRecordState::Empty) {
                FallbackRecordState::InFlight(fallbacks) => fallbacks,
                _ => unreachable!("fallback recovery must retain in-flight values"),
            };
            *state = FallbackRecordState::Failed { fallbacks, error };
        }
    }
}

fn resolve_metadata_precedence<T>(
    state: FallbackRecordState,
    selected: AppResult<StepOutcome<T>>,
) -> AppResult<T> {
    let metadata_failure = match state {
        FallbackRecordState::Failed { fallbacks, error } => Some((
            fallbacks
                .iter()
                .any(|fallback| fallback.kind() == TakeoutFallbackKind::OnlyMyMessages),
            error,
        )),
        _ => None,
    };
    if let Some((true, error)) = metadata_failure.as_ref() {
        return Err(error.clone());
    }
    match selected {
        Err(error) => Err(error),
        Ok(StepOutcome::AttemptStopped { error, .. }) => Err(error),
        Ok(StepOutcome::Remote(remote) | StepOutcome::MetadataStopped(remote)) => {
            match (metadata_failure, remote) {
                (Some((false, metadata_error)), Ok(_)) => Err(metadata_error),
                (_, remote) => remote,
            }
        }
    }
}

#[cfg(test)]
async fn run_history_search_transition<T, Invoke, RecordAttempt, RecordFallbacks>(
    cancellation_token: Option<CancellationToken>,
    attempt: TakeoutAttempt,
    mut invoke: Invoke,
    mut record_attempt: RecordAttempt,
    mut record_fallbacks: RecordFallbacks,
) -> AppResult<T>
where
    Invoke: AsyncFnMut(HistoryTransitionCommand) -> HistoryTransitionResponse<T>,
    RecordAttempt: AsyncFnMut(TakeoutAttempt) -> AppResult<()>,
    RecordFallbacks: AsyncFnMut(Vec<TakeoutFallback>) -> AppResult<()>,
{
    let mut record_state = FallbackRecordState::Empty;
    let selected = run_takeout_step_with_cancel(cancellation_token, async {
        record_attempt(attempt).await?;
        let HistoryTransitionResponse::Remote {
            result: history_result,
            fallbacks,
        } = invoke(HistoryTransitionCommand::History).await
        else {
            return Err(AppError::internal(
                "Takeout history transition returned drain metadata for a remote call",
            ));
        };
        let classified_only_my = history_result.is_err()
            && fallbacks
                .iter()
                .any(|fallback| fallback.kind() == TakeoutFallbackKind::OnlyMyMessages);
        queue_drained_fallbacks(&mut record_state, fallbacks)?;
        if classified_only_my {
            record_pending_once(&mut record_state, &mut record_fallbacks).await;
            if matches!(record_state, FallbackRecordState::Failed { .. }) {
                return Ok(StepOutcome::MetadataStopped(history_result));
            }
            if let Err(error) = record_attempt(attempt).await {
                return Ok(StepOutcome::AttemptStopped {
                    remote: history_result,
                    error,
                });
            }
            let HistoryTransitionResponse::Remote {
                result: search_result,
                fallbacks,
            } = invoke(HistoryTransitionCommand::Search).await
            else {
                return Err(AppError::internal(
                    "Takeout search transition returned drain metadata for a remote call",
                ));
            };
            queue_drained_fallbacks(&mut record_state, fallbacks)?;
            return Ok(StepOutcome::Remote(search_result));
        }
        Ok(StepOutcome::Remote(history_result))
    })
    .await;
    let HistoryTransitionResponse::Drained(fallbacks) =
        invoke(HistoryTransitionCommand::Drain).await
    else {
        return Err(AppError::internal(
            "Takeout history transition returned a remote result while draining metadata",
        ));
    };
    queue_drained_fallbacks(&mut record_state, fallbacks)?;
    record_after_select_once(&mut record_state, &mut record_fallbacks).await;
    resolve_metadata_precedence(record_state, selected)
}

#[derive(Default)]
struct TakeoutAttemptRecordState {
    recorded: Option<(i32, i32)>,
}

#[derive(Default)]
struct TakeoutFallbackRecordTracker {
    export_dc_recorded: bool,
    only_my_messages_recorded: bool,
}

async fn record_export_dc_attempt_if_needed(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    batch_id: i64,
    attempt: TakeoutAttempt,
    state: &mut TakeoutAttemptRecordState,
) -> AppResult<()> {
    let key = (attempt.home_dc_id(), attempt.export_dc_id());
    if state.recorded == Some(key) {
        return Ok(());
    }
    mark_takeout_export_dc_attempted(pool, batch_id, attempt.export_dc_id()).await?;
    state.recorded = Some(key);
    Ok(())
}

async fn record_export_dc_fallback_if_needed(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    batch_id: i64,
    warnings: &mut Vec<String>,
    tracker: &mut TakeoutFallbackRecordTracker,
    fallback: TakeoutFallback,
) -> AppResult<()> {
    push_warning_once(warnings, fallback.warning().to_string());
    if tracker.export_dc_recorded {
        return Ok(());
    }
    let message = fallback.provenance_message().unwrap_or(fallback.warning());
    mark_takeout_export_dc_fallback(pool, batch_id, message).await?;
    tracker.export_dc_recorded = true;
    Ok(())
}

async fn record_only_my_messages_fallback_if_needed(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    batch_id: i64,
    warnings: &mut Vec<String>,
    tracker: &mut TakeoutFallbackRecordTracker,
    fallback: TakeoutFallback,
) -> AppResult<()> {
    push_warning_once(warnings, fallback.warning().to_string());
    if tracker.only_my_messages_recorded {
        return Ok(());
    }
    let message = fallback.provenance_message().ok_or_else(|| {
        AppError::internal("OnlyMyMessages fallback is missing provenance metadata")
    })?;
    mark_takeout_only_my_messages_fallback(pool, batch_id, message).await?;
    tracker.only_my_messages_recorded = true;
    Ok(())
}

async fn record_takeout_fallbacks(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    batch_id: i64,
    warnings: &mut Vec<String>,
    tracker: &mut TakeoutFallbackRecordTracker,
    fallbacks: Vec<TakeoutFallback>,
) -> AppResult<()> {
    for fallback in fallbacks {
        match fallback.kind() {
            TakeoutFallbackKind::ExportDc => {
                record_export_dc_fallback_if_needed(pool, batch_id, warnings, tracker, fallback)
                    .await?;
            }
            TakeoutFallbackKind::OnlyMyMessages => {
                record_only_my_messages_fallback_if_needed(
                    pool, batch_id, warnings, tracker, fallback,
                )
                .await?;
            }
        }
    }
    Ok(())
}

async fn record_pending_takeout_fallbacks(
    state: &mut FallbackRecordState,
    pool: &sqlx::Pool<sqlx::Sqlite>,
    batch_id: i64,
    warnings: &mut Vec<String>,
    tracker: &mut TakeoutFallbackRecordTracker,
) {
    let FallbackRecordState::Pending(fallbacks) =
        std::mem::replace(state, FallbackRecordState::Empty)
    else {
        return;
    };
    *state = FallbackRecordState::InFlight(fallbacks);
    let owned = match state {
        FallbackRecordState::InFlight(fallbacks) => fallbacks.clone(),
        _ => unreachable!("fallback recorder must be in flight"),
    };
    match record_takeout_fallbacks(pool, batch_id, warnings, tracker, owned).await {
        Ok(()) => *state = FallbackRecordState::Recorded,
        Err(error) => {
            let fallbacks = match std::mem::replace(state, FallbackRecordState::Empty) {
                FallbackRecordState::InFlight(fallbacks) => fallbacks,
                _ => unreachable!("fallback recorder must retain in-flight values"),
            };
            *state = FallbackRecordState::Failed { fallbacks, error };
        }
    }
}

async fn record_after_select_takeout_fallbacks(
    state: &mut FallbackRecordState,
    pool: &sqlx::Pool<sqlx::Sqlite>,
    batch_id: i64,
    warnings: &mut Vec<String>,
    tracker: &mut TakeoutFallbackRecordTracker,
) {
    if matches!(state, FallbackRecordState::Pending(_)) {
        record_pending_takeout_fallbacks(state, pool, batch_id, warnings, tracker).await;
        return;
    }
    let FallbackRecordState::InFlight(fallbacks) = state else {
        return;
    };
    let retained = fallbacks.clone();
    match record_takeout_fallbacks(pool, batch_id, warnings, tracker, retained).await {
        Ok(()) => *state = FallbackRecordState::Recorded,
        Err(error) => {
            let fallbacks = match std::mem::replace(state, FallbackRecordState::Empty) {
                FallbackRecordState::InFlight(fallbacks) => fallbacks,
                _ => unreachable!("fallback recovery must retain in-flight values"),
            };
            *state = FallbackRecordState::Failed { fallbacks, error };
        }
    }
}

macro_rules! run_takeout_transport_step_with_provenance {
    (
        $cancellation_token:expr,
        $pool:expr,
        $batch_id:expr,
        $transport:ident,
        $warnings:ident,
        $attempt_state:ident,
        $fallback_tracker:ident,
        $operation:expr
    ) => {{
        let attempt = $transport.export_dc_attempt();
        let mut record_state = FallbackRecordState::Empty;
        let selected = run_takeout_step_with_cancel($cancellation_token, async {
            record_export_dc_attempt_if_needed($pool, $batch_id, attempt, &mut $attempt_state)
                .await?;
            let remote = $operation.await;
            queue_drained_fallbacks(&mut record_state, $transport.drain_fallbacks())?;
            Ok(StepOutcome::Remote(remote))
        })
        .await;
        queue_drained_fallbacks(&mut record_state, $transport.drain_fallbacks())?;
        record_after_select_takeout_fallbacks(
            &mut record_state,
            $pool,
            $batch_id,
            &mut $warnings,
            &mut $fallback_tracker,
        )
        .await;
        resolve_metadata_precedence(record_state, selected)
    }};
}

macro_rules! run_history_search_transition_with_provenance {
    (
        $cancellation_token:expr,
        $pool:expr,
        $batch_id:expr,
        $transport:ident,
        $warnings:ident,
        $attempt_state:ident,
        $fallback_tracker:ident,
        $history_operation:expr,
        $search_operation:expr
    ) => {{
        let attempt = $transport.export_dc_attempt();
        let mut record_state = FallbackRecordState::Empty;
        let selected = run_takeout_step_with_cancel($cancellation_token, async {
            record_export_dc_attempt_if_needed($pool, $batch_id, attempt, &mut $attempt_state)
                .await?;
            let history_result = $history_operation.await;
            let fallbacks = $transport.drain_fallbacks();
            let classified_only_my = history_result.is_err()
                && fallbacks
                    .iter()
                    .any(|fallback| fallback.kind() == TakeoutFallbackKind::OnlyMyMessages);
            queue_drained_fallbacks(&mut record_state, fallbacks)?;
            if classified_only_my {
                record_pending_takeout_fallbacks(
                    &mut record_state,
                    $pool,
                    $batch_id,
                    &mut $warnings,
                    &mut $fallback_tracker,
                )
                .await;
                if matches!(record_state, FallbackRecordState::Failed { .. }) {
                    return Ok(StepOutcome::MetadataStopped(history_result));
                }
                if let Err(error) = record_export_dc_attempt_if_needed(
                    $pool,
                    $batch_id,
                    attempt,
                    &mut $attempt_state,
                )
                .await
                {
                    return Ok(StepOutcome::AttemptStopped {
                        remote: history_result,
                        error,
                    });
                }
                let search_result = $search_operation.await;
                queue_drained_fallbacks(&mut record_state, $transport.drain_fallbacks())?;
                return Ok(StepOutcome::Remote(search_result));
            }
            Ok(StepOutcome::Remote(history_result))
        })
        .await;
        queue_drained_fallbacks(&mut record_state, $transport.drain_fallbacks())?;
        record_after_select_takeout_fallbacks(
            &mut record_state,
            $pool,
            $batch_id,
            &mut $warnings,
            &mut $fallback_tracker,
        )
        .await;
        resolve_metadata_precedence(record_state, selected)
    }};
}

async fn run_takeout_step_with_cancel<Fut, T>(
    cancellation_token: Option<CancellationToken>,
    future: Fut,
) -> AppResult<T>
where
    Fut: Future<Output = AppResult<T>>,
{
    let Some(cancellation_token) = cancellation_token else {
        return future.await;
    };

    if cancellation_token.is_cancelled() {
        return Err(AppError::validation("Takeout import cancelled"));
    }

    tokio::select! {
        result = future => result,
        _ = cancellation_token.cancelled() => Err(AppError::validation("Takeout import cancelled")),
    }
}

async fn run_takeout_migrated_history_import(
    handle: &AppHandle,
    job_id: &str,
    batch_id: i64,
) -> AppResult<TakeoutImportOutcome> {
    let takeout_state = handle.state::<TakeoutImportState>();
    let cancellation_token = takeout_state.cancellation_token(job_id).await;
    let telegram_state = handle.state::<TelegramState>();
    let repair_state = handle.state::<SourceIdentityRepairState>();
    require_source_identity_ready(repair_state.inner()).await?;
    let pool = get_pool(handle).await?;
    let source_id = takeout_state
        .update_job(job_id, |_| {})
        .await
        .ok_or_else(|| AppError::internal(format!("Takeout job {job_id} not found")))?
        .source_id;
    let source = load_source(&pool, source_id).await?;
    let capability = migrated_history::load_migrated_history_capability(&pool, source_id)
        .await?
        .ok_or_else(migrated_history::not_detected_error)?;
    let expected_chat_id = capability.migrated_from_chat_id;
    let account_id = source.account_id.ok_or_else(|| {
        AppError::validation(format!("Source {} is not linked to an account", source.id))
    })?;
    let client_handle = telegram_state.authorized_client(account_id).await?;

    update_and_emit(handle, &takeout_state, job_id, |job| {
        job.phase = PHASE_RESOLVING_SOURCE.to_string();
        job.message = Some("Resolving Telegram source.".to_string());
    })
    .await;
    let resolved_peer = run_takeout_step_with_cancel(
        cancellation_token.clone(),
        resolve_and_refresh_peer(handle, &pool, &client_handle, &source, account_id),
    )
    .await?;
    let current_peer = TakeoutPeer::from_descriptor(&resolved_peer.descriptor)?;
    update_takeout_resolved_peer(
        &pool,
        batch_id,
        current_peer.peer_kind(),
        current_peer.peer_id(),
        "chat",
        expected_chat_id.unwrap_or_default(),
    )
    .await?;

    update_and_emit(handle, &takeout_state, job_id, |job| {
        job.phase = PHASE_STARTING_TAKEOUT.to_string();
        job.message = Some("Starting Takeout session.".to_string());
    })
    .await;
    let mut transport =
        run_takeout_step_with_cancel(cancellation_token.clone(), client_handle.prepare_takeout())
            .await?;
    let mut warnings = Vec::new();
    let mut attempt_state = TakeoutAttemptRecordState::default();
    let mut fallback_tracker = TakeoutFallbackRecordTracker::default();
    let takeout_id = run_takeout_transport_step_with_provenance!(
        cancellation_token.clone(),
        &pool,
        batch_id,
        transport,
        warnings,
        attempt_state,
        fallback_tracker,
        transport.init(TELEGRAM_KIND_GROUP)
    )?;
    update_takeout_session_started(&pool, batch_id, takeout_id).await?;

    update_and_emit(handle, &takeout_state, job_id, |job| {
        job.phase = PHASE_VALIDATING_PEER.to_string();
        job.message = Some("Revalidating migrated history availability.".to_string());
    })
    .await;
    let revalidated = run_takeout_transport_step_with_provenance!(
        cancellation_token.clone(),
        &pool,
        batch_id,
        transport,
        warnings,
        attempt_state,
        fallback_tracker,
        transport.revalidate_migrated_peer(takeout_id, &current_peer)
    )?;
    let validation = migrated_history::validate_revalidated_chat_id(
        expected_chat_id,
        revalidated.as_ref().map(|(chat_id, _)| *chat_id),
    )?;
    let (_, migrated_peer) = revalidated.ok_or_else(migrated_history::unavailable_error)?;
    migrated_history::upsert_migrated_history_available(
        &pool,
        source_id,
        validation.migrated_from_chat_id,
        now_secs(),
    )
    .await?;

    update_and_emit(handle, &takeout_state, job_id, |job| {
        job.phase = PHASE_LOADING_SPLITS.to_string();
        job.message = Some("Loading migrated history Takeout message ranges.".to_string());
        job.warnings = warnings.to_vec();
    })
    .await;
    let (split_count, selected_ranges) = run_takeout_transport_step_with_provenance!(
        cancellation_token.clone(),
        &pool,
        batch_id,
        transport,
        warnings,
        attempt_state,
        fallback_tracker,
        transport.message_ranges(takeout_id, TELEGRAM_KIND_GROUP)
    )?;
    let selected_split_count = selected_ranges.len() as i64;

    update_and_emit(handle, &takeout_state, job_id, |job| {
        job.phase = PHASE_COUNTING.to_string();
        job.message = Some("Counting migrated history messages.".to_string());
        job.warnings = warnings.to_vec();
    })
    .await;
    let mut counted_ranges = Vec::new();
    let mut total = 0_i64;
    for range in selected_ranges {
        let count = takeout_history_count_probe(
            &pool,
            batch_id,
            cancellation_token.clone(),
            &mut transport,
            takeout_id,
            &migrated_peer,
            &range,
            TELEGRAM_KIND_GROUP,
            &mut warnings,
            &mut attempt_state,
            &mut fallback_tracker,
        )
        .await?;
        total += count.count();
        counted_ranges.push(CountedMessageRange { range, count });
    }
    update_takeout_split_metadata(
        &pool,
        batch_id,
        split_count,
        selected_split_count,
        Some(total),
    )
    .await?;

    update_and_emit(handle, &takeout_state, job_id, |job| {
        job.phase = PHASE_IMPORTING_HISTORY.to_string();
        job.message = Some("Importing migrated history.".to_string());
        job.progress_current = Some(0);
        job.progress_total = Some(total);
        job.warnings = warnings.to_vec();
    })
    .await;
    let import = import_takeout_history_ranges(
        handle,
        job_id,
        batch_id,
        &mut transport,
        takeout_id,
        &migrated_peer,
        counted_ranges,
        &source,
        total,
        &mut warnings,
        &mut attempt_state,
        &mut fallback_tracker,
        Some(validation.migrated_from_chat_id),
    )
    .await?;

    if takeout_state.is_cancel_requested(job_id).await {
        return Err(AppError::validation("Takeout import cancelled"));
    }

    update_and_emit(handle, &takeout_state, job_id, |job| {
        job.phase = PHASE_FINISHING_TAKEOUT.to_string();
        job.message = Some("Finishing Takeout session.".to_string());
        job.warnings = warnings.to_vec();
    })
    .await;
    run_takeout_transport_step_with_provenance!(
        cancellation_token.clone(),
        &pool,
        batch_id,
        transport,
        warnings,
        attempt_state,
        fallback_tracker,
        transport.finish(takeout_id, true)
    )?;
    mark_takeout_migrated_history_imported(&pool, batch_id).await?;
    finalize_ingest_batch(&pool, batch_id, TerminalBatchStatus::Completed, None).await?;

    Ok(TakeoutImportOutcome {
        inserted: import.inserted,
        skipped: import.skipped,
        progress_total: Some(total),
        warnings,
    })
}

async fn run_takeout_source_import(
    handle: &AppHandle,
    job_id: &str,
    batch_id: i64,
) -> AppResult<TakeoutImportOutcome> {
    let takeout_state = handle.state::<TakeoutImportState>();
    let cancellation_token = takeout_state.cancellation_token(job_id).await;
    let telegram_state = handle.state::<TelegramState>();
    let repair_state = handle.state::<SourceIdentityRepairState>();
    require_source_identity_ready(repair_state.inner()).await?;
    let pool = get_pool(handle).await?;
    let source_id = takeout_state
        .update_job(job_id, |_| {})
        .await
        .ok_or_else(|| AppError::internal(format!("Takeout job {job_id} not found")))?
        .source_id;
    let source = load_source(&pool, source_id).await?;
    let telegram_source_subtype = load_takeout_source_subtype(&pool, source.id).await?;

    let account_id = source.account_id.ok_or_else(|| {
        AppError::validation(format!("Source {} is not linked to an account", source.id))
    })?;
    let client_handle = telegram_state.authorized_client(account_id).await?;

    update_and_emit(handle, &takeout_state, job_id, |job| {
        job.phase = PHASE_RESOLVING_SOURCE.to_string();
        job.message = Some("Resolving Telegram source.".to_string());
    })
    .await;
    let resolved_peer = run_takeout_step_with_cancel(
        cancellation_token.clone(),
        resolve_and_refresh_peer(handle, &pool, &client_handle, &source, account_id),
    )
    .await?;
    let peer = TakeoutPeer::from_descriptor(&resolved_peer.descriptor)?;
    update_takeout_resolved_peer(
        &pool,
        batch_id,
        peer.peer_kind(),
        peer.peer_id(),
        peer.peer_kind(),
        peer.peer_id(),
    )
    .await?;

    update_and_emit(handle, &takeout_state, job_id, |job| {
        job.phase = PHASE_STARTING_TAKEOUT.to_string();
        job.message = Some("Starting Takeout session.".to_string());
    })
    .await;
    run_takeout_step_with_cancel(
        cancellation_token.clone(),
        client_handle.takeout_self_check(),
    )
    .await?;
    let mut transport =
        run_takeout_step_with_cancel(cancellation_token.clone(), client_handle.prepare_takeout())
            .await?;
    let mut warnings = Vec::new();
    let mut attempt_state = TakeoutAttemptRecordState::default();
    let mut fallback_tracker = TakeoutFallbackRecordTracker::default();
    let takeout_id = run_takeout_transport_step_with_provenance!(
        cancellation_token.clone(),
        &pool,
        batch_id,
        transport,
        warnings,
        attempt_state,
        fallback_tracker,
        transport.init(&telegram_source_subtype)
    )?;
    update_takeout_session_started(&pool, batch_id, takeout_id).await?;

    let started_result = run_started_takeout_source_import(
        handle,
        job_id,
        batch_id,
        &pool,
        &source,
        &telegram_source_subtype,
        resolved_peer,
        &client_handle,
        &peer,
        &mut transport,
        takeout_id,
        &mut warnings,
        &mut attempt_state,
        &mut fallback_tracker,
    )
    .await;

    match started_result {
        Ok(outcome) => Ok(outcome),
        Err(error) => {
            if let Err(finish_error) = run_takeout_transport_step_with_provenance!(
                None,
                &pool,
                batch_id,
                transport,
                warnings,
                attempt_state,
                fallback_tracker,
                transport.finish(takeout_id, false)
            ) {
                warnings.push(format!(
                    "Failed to finish Takeout session after error: {finish_error}"
                ));
                let _ = record_ingest_batch_warning(
                    &pool,
                    batch_id,
                    "finish_takeout_failed",
                    &format!(
                        "Failed to finish Takeout session after terminal error: {finish_error}"
                    ),
                )
                .await;
            }
            update_and_emit(handle, &takeout_state, job_id, |job| {
                job.warnings = warnings.clone();
            })
            .await;
            Err(error)
        }
    }
}

async fn run_started_takeout_source_import(
    handle: &AppHandle,
    job_id: &str,
    batch_id: i64,
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source: &crate::sources::SourceSyncTarget,
    telegram_source_subtype: &str,
    resolved_peer: crate::sources::ResolvedSyncPeer,
    client_handle: &TelegramClientHandle,
    peer: &TakeoutPeer,
    transport: &mut TakeoutTransport,
    takeout_id: i64,
    warnings: &mut Vec<String>,
    attempt_state: &mut TakeoutAttemptRecordState,
    fallback_tracker: &mut TakeoutFallbackRecordTracker,
) -> AppResult<TakeoutImportOutcome> {
    run_started_takeout_source_import_inner(
        handle,
        job_id,
        batch_id,
        pool,
        source,
        telegram_source_subtype,
        resolved_peer,
        client_handle,
        peer,
        transport,
        takeout_id,
        warnings,
        attempt_state,
        fallback_tracker,
    )
    .await
}

async fn run_started_takeout_source_import_inner(
    handle: &AppHandle,
    job_id: &str,
    batch_id: i64,
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source: &crate::sources::SourceSyncTarget,
    telegram_source_subtype: &str,
    resolved_peer: crate::sources::ResolvedSyncPeer,
    client_handle: &TelegramClientHandle,
    peer: &TakeoutPeer,
    transport: &mut TakeoutTransport,
    takeout_id: i64,
    mut warnings: &mut Vec<String>,
    mut attempt_state: &mut TakeoutAttemptRecordState,
    mut fallback_tracker: &mut TakeoutFallbackRecordTracker,
) -> AppResult<TakeoutImportOutcome> {
    let takeout_state = handle.state::<TakeoutImportState>();
    let cancellation_token = takeout_state.cancellation_token(job_id).await;

    update_and_emit(handle, &takeout_state, job_id, |job| {
        job.phase = PHASE_VALIDATING_PEER.to_string();
        job.message = Some("Validating Telegram source.".to_string());
        job.warnings.extend(warnings.clone());
    })
    .await;
    run_takeout_transport_step_with_provenance!(
        cancellation_token.clone(),
        pool,
        batch_id,
        transport,
        warnings,
        attempt_state,
        fallback_tracker,
        transport.validate_peer(takeout_id, peer, telegram_source_subtype)
    )?;
    let migrated_from_chat_id = run_takeout_transport_step_with_provenance!(
        cancellation_token.clone(),
        pool,
        batch_id,
        transport,
        warnings,
        attempt_state,
        fallback_tracker,
        transport.detect_supergroup_migration(takeout_id, peer, telegram_source_subtype)
    )?;
    if let Some(migrated_from_chat_id) = migrated_from_chat_id {
        push_warning_once(warnings, migrated_history_detected_warning());
        migrated_history::upsert_migrated_history_available(
            pool,
            source.id,
            migrated_from_chat_id,
            now_secs(),
        )
        .await?;
        mark_takeout_migrated_history_deferred(
            pool,
            batch_id,
            "Supergroup migrated history detected; current foundation import defers migrated history.",
        )
        .await?;
    }

    update_and_emit(handle, &takeout_state, job_id, |job| {
        job.phase = PHASE_LOADING_SPLITS.to_string();
        job.message = Some("Loading Takeout message ranges.".to_string());
        job.warnings = warnings.to_vec();
    })
    .await;
    let (split_count, selected_ranges) = run_takeout_transport_step_with_provenance!(
        cancellation_token.clone(),
        pool,
        batch_id,
        transport,
        warnings,
        attempt_state,
        fallback_tracker,
        transport.message_ranges(takeout_id, telegram_source_subtype)
    )?;
    let selected_split_count = selected_ranges.len() as i64;

    update_and_emit(handle, &takeout_state, job_id, |job| {
        job.phase = PHASE_COUNTING.to_string();
        job.message = Some("Counting messages.".to_string());
        job.warnings = warnings.to_vec();
    })
    .await;
    let mut counted_ranges = Vec::new();
    let mut total = 0_i64;
    for range in selected_ranges {
        let count = takeout_history_count_probe(
            pool,
            batch_id,
            cancellation_token.clone(),
            transport,
            takeout_id,
            peer,
            &range,
            telegram_source_subtype,
            warnings,
            attempt_state,
            fallback_tracker,
        )
        .await?;
        total += count.count();
        counted_ranges.push(CountedMessageRange { range, count });
    }
    update_takeout_split_metadata(
        pool,
        batch_id,
        split_count,
        selected_split_count,
        Some(total),
    )
    .await?;

    update_and_emit(handle, &takeout_state, job_id, |job| {
        job.phase = PHASE_IMPORTING_HISTORY.to_string();
        job.message = Some("Importing history.".to_string());
        job.progress_current = Some(0);
        job.progress_total = Some(total);
        job.warnings = warnings.to_vec();
    })
    .await;
    let import = import_takeout_history_ranges(
        handle,
        job_id,
        batch_id,
        transport,
        takeout_id,
        peer,
        counted_ranges,
        &source,
        total,
        warnings,
        attempt_state,
        fallback_tracker,
        None,
    )
    .await?;

    if takeout_state.is_cancel_requested(job_id).await {
        return Err(AppError::validation("Takeout import cancelled"));
    }

    update_and_emit(handle, &takeout_state, job_id, |job| {
        job.phase = PHASE_FINISHING_TAKEOUT.to_string();
        job.message = Some("Finishing Takeout session.".to_string());
        job.warnings = warnings.to_vec();
    })
    .await;
    run_takeout_transport_step_with_provenance!(
        cancellation_token.clone(),
        pool,
        batch_id,
        transport,
        warnings,
        attempt_state,
        fallback_tracker,
        transport.finish(takeout_id, true)
    )?;
    refresh_forum_topics_after_completed_takeout(
        pool,
        batch_id,
        client_handle,
        &resolved_peer.descriptor,
        source,
        telegram_source_subtype,
        warnings,
    )
    .await?;
    finalize_sync(
        &pool,
        &source,
        source.last_sync_state.unwrap_or(0),
        import.max_message_id,
        resolved_peer.refreshed_avatar_cache_key,
    )
    .await?;
    finalize_ingest_batch(pool, batch_id, TerminalBatchStatus::Completed, None).await?;

    Ok(TakeoutImportOutcome {
        inserted: import.inserted,
        skipped: import.skipped,
        progress_total: Some(total),
        warnings: warnings.to_vec(),
    })
}

struct TakeoutHistoryImport {
    inserted: i64,
    skipped: i64,
    max_message_id: i64,
}

struct CountedMessageRange {
    range: MessageRange,
    count: TakeoutCount,
}

fn ensure_supported_takeout_source_subtype(source_subtype: &str) -> AppResult<()> {
    TelegramSourceKind::from_source_subtype(source_subtype).map(|_| ())
}

async fn load_takeout_source_subtype(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source_id: i64,
) -> AppResult<String> {
    let identity = crate::sources::identity::load_telegram_source_identity(pool, source_id).await?;
    let source_subtype = identity.source_subtype.as_str();
    ensure_supported_takeout_source_subtype(source_subtype)?;
    Ok(source_subtype.to_string())
}

async fn takeout_history_count_probe(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    batch_id: i64,
    cancellation_token: Option<CancellationToken>,
    transport: &mut TakeoutTransport,
    takeout_id: i64,
    peer: &TakeoutPeer,
    range: &MessageRange,
    telegram_source_subtype: &str,
    mut warnings: &mut Vec<String>,
    mut attempt_state: &mut TakeoutAttemptRecordState,
    mut fallback_tracker: &mut TakeoutFallbackRecordTracker,
) -> AppResult<TakeoutCount> {
    run_history_search_transition_with_provenance!(
        cancellation_token,
        pool,
        batch_id,
        transport,
        warnings,
        attempt_state,
        fallback_tracker,
        transport.history_count(takeout_id, peer, range, telegram_source_subtype),
        transport.search_my_history_count(takeout_id, peer, range)
    )
}

async fn import_takeout_history_ranges(
    handle: &AppHandle,
    job_id: &str,
    batch_id: i64,
    transport: &mut TakeoutTransport,
    takeout_id: i64,
    peer: &TakeoutPeer,
    ranges: Vec<CountedMessageRange>,
    source: &crate::sources::SourceSyncTarget,
    total: i64,
    warnings: &mut Vec<String>,
    attempt_state: &mut TakeoutAttemptRecordState,
    fallback_tracker: &mut TakeoutFallbackRecordTracker,
    migrated_from_chat_id: Option<i64>,
) -> AppResult<TakeoutHistoryImport> {
    let mut imported = TakeoutHistoryImport {
        inserted: 0,
        skipped: 0,
        max_message_id: source.last_sync_state.unwrap_or(0),
    };

    for counted_range in ranges {
        imported = import_takeout_history_pages(
            handle,
            job_id,
            batch_id,
            transport,
            takeout_id,
            peer,
            counted_range,
            source,
            total,
            imported,
            warnings,
            attempt_state,
            fallback_tracker,
            migrated_from_chat_id,
        )
        .await?;
    }

    Ok(imported)
}

async fn import_takeout_history_pages(
    handle: &AppHandle,
    job_id: &str,
    batch_id: i64,
    transport: &mut TakeoutTransport,
    takeout_id: i64,
    peer: &TakeoutPeer,
    counted_range: CountedMessageRange,
    source: &crate::sources::SourceSyncTarget,
    total: i64,
    mut imported: TakeoutHistoryImport,
    warnings: &mut Vec<String>,
    attempt_state: &mut TakeoutAttemptRecordState,
    fallback_tracker: &mut TakeoutFallbackRecordTracker,
    migrated_from_chat_id: Option<i64>,
) -> AppResult<TakeoutHistoryImport> {
    let takeout_state = handle.state::<TakeoutImportState>();
    let cancellation_token = takeout_state.cancellation_token(job_id).await;
    let pool = get_pool(handle).await?;
    let range = counted_range.range;
    let count = counted_range.count;
    let mut previous: Option<TakeoutPage> = None;

    loop {
        let mut page = takeout_history_page_response(
            cancellation_token.clone(),
            &pool,
            batch_id,
            transport,
            takeout_id,
            peer,
            &range,
            &count,
            previous.as_ref(),
            warnings,
            attempt_state,
            fallback_tracker,
        )
        .await?;
        if let Some(warning) = page.take_pagination_fallback_warning() {
            warnings.push(warning);
            update_and_emit(handle, &takeout_state, job_id, |job| {
                job.warnings = warnings.clone();
            })
            .await;
            if takeout_state.is_cancel_requested(job_id).await {
                return Err(AppError::validation("Takeout import cancelled"));
            }
        }

        for message in page.take_messages() {
            let message_id = message.message_id();
            if message_id <= i64::from(range.min_id()) {
                continue;
            }
            let next_max_message_id = imported.max_message_id.max(message_id);
            if next_max_message_id != imported.max_message_id {
                imported.max_message_id = next_max_message_id;
                update_takeout_max_message_id(&pool, batch_id, imported.max_message_id).await?;
            }
            match message.into_draft(source.title.as_deref()) {
                Ok(Some(mut item)) => {
                    let parsed_identity = item.telegram_identity.clone().ok_or_else(|| {
                        AppError::validation(
                            "Parsed Takeout Telegram item is missing native message identity",
                        )
                    })?;
                    let insert_outcome = if let Some(migrated_from_chat_id) = migrated_from_chat_id
                    {
                        let identity = migrated_history::migrated_small_group_identity(
                            parsed_identity.telegram_message_id,
                            migrated_from_chat_id,
                        );
                        item.telegram_identity = Some(identity);
                        crate::sources::insert_telegram_source_item_with_observation_in_context(
                            &pool,
                            batch_id,
                            source.id,
                            item,
                            crate::sources::TelegramInsertContext::MigratedSmallGroupHistory,
                        )
                        .await?
                    } else {
                        crate::sources::insert_telegram_source_item_with_observation(
                            &pool, batch_id, source.id, item,
                        )
                        .await?
                    };
                    match insert_outcome {
                        crate::sources::TelegramItemInsertOutcome::Inserted { .. } => {
                            imported.inserted += 1;
                        }
                        crate::sources::TelegramItemInsertOutcome::DuplicateObserved { .. }
                        | crate::sources::TelegramItemInsertOutcome::Skipped { .. } => {
                            imported.skipped += 1;
                        }
                    }
                }
                Ok(None) => imported.skipped += 1,
                Err(error) => return Err(error),
            }
        }

        update_and_emit(handle, &takeout_state, job_id, |job| {
            job.inserted = imported.inserted;
            job.skipped = imported.skipped;
            job.progress_current = Some((imported.inserted + imported.skipped).min(total));
            job.progress_total = Some(total);
            job.warnings = warnings.clone();
        })
        .await;

        if takeout_state.is_cancel_requested(job_id).await {
            return Err(AppError::validation("Takeout import cancelled"));
        }

        if !page.has_next() {
            break;
        }
        previous = Some(page);
    }

    Ok(imported)
}

async fn takeout_history_page_response(
    cancellation_token: Option<CancellationToken>,
    pool: &sqlx::Pool<sqlx::Sqlite>,
    batch_id: i64,
    transport: &mut TakeoutTransport,
    takeout_id: i64,
    peer: &TakeoutPeer,
    range: &MessageRange,
    count: &TakeoutCount,
    previous: Option<&TakeoutPage>,
    mut warnings: &mut Vec<String>,
    mut attempt_state: &mut TakeoutAttemptRecordState,
    mut fallback_tracker: &mut TakeoutFallbackRecordTracker,
) -> AppResult<TakeoutPage> {
    let only_my_messages = previous
        .map(TakeoutPage::only_my_messages)
        .unwrap_or_else(|| count.only_my_messages());
    if only_my_messages {
        return run_takeout_transport_step_with_provenance!(
            cancellation_token,
            pool,
            batch_id,
            transport,
            warnings,
            attempt_state,
            fallback_tracker,
            transport.search_my_history_page(takeout_id, peer, range, count, previous)
        );
    }
    run_history_search_transition_with_provenance!(
        cancellation_token,
        pool,
        batch_id,
        transport,
        warnings,
        attempt_state,
        fallback_tracker,
        transport.history_page(takeout_id, peer, range, count, previous),
        transport.search_my_history_page(takeout_id, peer, range, count, previous)
    )
}

fn push_warning_once(warnings: &mut Vec<String>, warning: impl Into<String>) {
    let warning = warning.into();
    if !warnings.iter().any(|existing| existing == &warning) {
        warnings.push(warning);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    };

    use super::{
        create_locked_migrated_history_start_records, create_locked_takeout_start_records,
        load_takeout_source_subtype, migrated_history_detected_warning, queue_drained_fallbacks,
        record_after_select_once, record_export_dc_attempt_if_needed,
        record_export_dc_fallback_if_needed, record_only_my_messages_fallback_if_needed,
        record_pending_once, record_takeout_fallbacks, resolve_metadata_precedence,
        run_history_search_transition, run_takeout_step_with_cancel, FallbackRecordState,
        HistoryTransitionCommand, HistoryTransitionResponse, StepOutcome,
        TakeoutAttemptRecordState, TakeoutFallbackRecordTracker,
    };
    use crate::error::{AppError, AppErrorKind, AppResult};
    use crate::ingest_provenance::{
        create_telegram_takeout_batch, finalize_ingest_batch, CreateTelegramTakeoutBatch,
        TerminalBatchStatus,
    };
    use crate::source_ingest::{SourceIngestKind, SourceIngestLocks};
    use crate::sources::test_support::{
        create_analysis_documents_table, create_ingest_provenance_tables,
        create_migrated_history_capability_tables, memory_pool_with_source_items_and_topics,
        memory_pool_with_sources,
    };
    use crate::sources::{
        insert_telegram_source_item, TELEGRAM_KIND_CHANNEL, TELEGRAM_KIND_SUPERGROUP,
    };
    use crate::takeout_import::state::TakeoutImportState;
    use crate::telegram_impl::{
        takeout_attempt_fixture, takeout_fallback_fixture, TakeoutFallbackKind,
        TelegramItemContext, TelegramMessageDraft, TelegramMessageIdentity,
        ITEM_KIND_TELEGRAM_MESSAGE,
    };
    use tokio_util::sync::CancellationToken;

    #[tokio::test]
    async fn takeout_step_cancel_wrapper_allows_completed_future() {
        let result = run_takeout_step_with_cancel(None, async { Ok::<_, AppError>("done") })
            .await
            .expect("step result");

        assert_eq!(result, "done");
    }

    #[tokio::test]
    async fn takeout_step_cancel_wrapper_interrupts_pending_future() {
        let token = CancellationToken::new();
        let cancellation = token.clone();
        let started = Arc::new(tokio::sync::Notify::new());
        let started_for_cancel = Arc::clone(&started);
        let recorder_calls = Arc::new(AtomicUsize::new(0));
        let recorder_calls_for_closure = Arc::clone(&recorder_calls);
        let search_calls = Arc::new(AtomicUsize::new(0));
        let progress_events = Arc::new(AtomicUsize::new(0));
        let fallback = takeout_fallback_fixture(
            TakeoutFallbackKind::OnlyMyMessages,
            "Channel history is private; falling back to messages.search(from_id=self).",
            Some(
                "Channel history is private; importing only messages visible through from_id=self fallback.",
            ),
        );
        let mut state = FallbackRecordState::Pending(vec![fallback]);
        let mut recorder = move |_fallbacks| {
            let call = recorder_calls_for_closure.fetch_add(1, Ordering::SeqCst);
            let started = Arc::clone(&started);
            async move {
                if call == 0 {
                    started.notify_one();
                    std::future::pending::<AppResult<()>>().await
                } else {
                    Ok(())
                }
            }
        };
        let cancel_task = tokio::spawn(async move {
            started_for_cancel.notified().await;
            cancellation.cancel();
        });

        let result = run_takeout_step_with_cancel(Some(token), async {
            record_pending_once(&mut state, &mut recorder).await;
            search_calls.fetch_add(1, Ordering::SeqCst);
            progress_events.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })
        .await;
        cancel_task.await.expect("cancel task");
        record_after_select_once(&mut state, &mut recorder).await;

        assert_eq!(
            result.expect_err("cancelled step").message,
            "Takeout import cancelled"
        );
        assert_eq!(recorder_calls.load(Ordering::SeqCst), 2);
        assert!(matches!(state, FallbackRecordState::Recorded));
        assert_eq!(search_calls.load(Ordering::SeqCst), 0);
        assert_eq!(progress_events.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn channel_private_count_probe_records_fallback_before_search_continuation() {
        let events = Arc::new(Mutex::new(Vec::new()));
        let invoke_events = Arc::clone(&events);
        let attempt_events = Arc::clone(&events);
        let record_events = Arc::clone(&events);
        let attempt = takeout_attempt_fixture(2, 40_002);
        let result = run_history_search_transition(
            None,
            attempt,
            move |command| {
                let events = Arc::clone(&invoke_events);
                async move {
                    match command {
                        HistoryTransitionCommand::History => {
                            events.lock().unwrap().push("history");
                            HistoryTransitionResponse::Remote {
                                result: Err(AppError::network("Rpc error 400: CHANNEL_PRIVATE")),
                                fallbacks: vec![takeout_fallback_fixture(
                                    TakeoutFallbackKind::OnlyMyMessages,
                                    "Channel history is private; falling back to messages.search(from_id=self).",
                                    Some("Channel history is private; importing only messages visible through from_id=self fallback."),
                                )],
                            }
                        }
                        HistoryTransitionCommand::Search => {
                            events.lock().unwrap().push("search");
                            HistoryTransitionResponse::Remote {
                                result: Ok(17_i64),
                                fallbacks: Vec::new(),
                            }
                        }
                        HistoryTransitionCommand::Drain => {
                            events.lock().unwrap().push("drain");
                            HistoryTransitionResponse::Drained(Vec::new())
                        }
                    }
                }
            },
            move |_attempt| {
                let events = Arc::clone(&attempt_events);
                async move {
                    events.lock().unwrap().push("attempt");
                    Ok(())
                }
            },
            move |_fallbacks| {
                let events = Arc::clone(&record_events);
                async move {
                    events.lock().unwrap().push("record");
                    Ok(())
                }
            },
        )
        .await
        .expect("OnlyMy search continuation");

        assert_eq!(result, 17);
        assert_eq!(
            *events.lock().unwrap(),
            vec!["attempt", "history", "record", "attempt", "search", "drain"]
        );

        let failed_events = Arc::new(Mutex::new(Vec::new()));
        let invoke_events = Arc::clone(&failed_events);
        let attempt_events = Arc::clone(&failed_events);
        let record_events = Arc::clone(&failed_events);
        let error = run_history_search_transition(
            None,
            attempt,
            move |command| {
                let events = Arc::clone(&invoke_events);
                async move {
                    match command {
                        HistoryTransitionCommand::History => {
                            events.lock().unwrap().push("history");
                            HistoryTransitionResponse::Remote {
                                result: Err(AppError::network("Rpc error 400: CHANNEL_PRIVATE")),
                                fallbacks: vec![takeout_fallback_fixture(
                                    TakeoutFallbackKind::OnlyMyMessages,
                                    "only-my warning",
                                    Some("only-my provenance"),
                                )],
                            }
                        }
                        HistoryTransitionCommand::Search => {
                            events.lock().unwrap().push("search");
                            HistoryTransitionResponse::Remote {
                                result: Ok(99_i64),
                                fallbacks: Vec::new(),
                            }
                        }
                        HistoryTransitionCommand::Drain => {
                            events.lock().unwrap().push("drain");
                            HistoryTransitionResponse::Drained(Vec::new())
                        }
                    }
                }
            },
            move |_attempt| {
                let events = Arc::clone(&attempt_events);
                async move {
                    events.lock().unwrap().push("attempt");
                    Ok(())
                }
            },
            move |_fallbacks| {
                let events = Arc::clone(&record_events);
                async move {
                    events.lock().unwrap().push("record");
                    Err(AppError::internal("fallback recorder failed"))
                }
            },
        )
        .await
        .expect_err("OnlyMy recorder failure must stop before search");
        assert_eq!(error.kind, AppErrorKind::Internal);
        assert_eq!(
            *failed_events.lock().unwrap(),
            vec!["attempt", "history", "record", "drain"]
        );

        let pool = memory_pool_with_source_items_and_topics().await;
        create_ingest_provenance_tables(&pool).await;
        seed_item_source(&pool, 1).await;
        let batch_id = create_telegram_takeout_batch(
            &pool,
            CreateTelegramTakeoutBatch {
                source_id: 1,
                account_id: 10,
                source_subtype: TELEGRAM_KIND_CHANNEL.to_string(),
            },
        )
        .await
        .expect("create takeout batch");
        let mut warnings = Vec::new();
        let mut fallback_tracker = TakeoutFallbackRecordTracker::default();
        let fallback = takeout_fallback_fixture(
            TakeoutFallbackKind::OnlyMyMessages,
            "Channel history is private; falling back to messages.search(from_id=self).",
            Some(
                "Channel history is private; importing only messages visible through from_id=self fallback.",
            ),
        );
        record_only_my_messages_fallback_if_needed(
            &pool,
            batch_id,
            &mut warnings,
            &mut fallback_tracker,
            fallback.clone(),
        )
        .await
        .expect("record validation fallback");
        record_only_my_messages_fallback_if_needed(
            &pool,
            batch_id,
            &mut warnings,
            &mut fallback_tracker,
            fallback,
        )
        .await
        .expect("dedupe validation fallback");
        let validation_events = Arc::new(Mutex::new(Vec::new()));
        let invoke_events = Arc::clone(&validation_events);
        let attempt_events = Arc::clone(&validation_events);
        let result = run_history_search_transition(
            None,
            attempt,
            move |command| {
                let events = Arc::clone(&invoke_events);
                async move {
                    match command {
                        HistoryTransitionCommand::History => {
                            events.lock().unwrap().push("history");
                            HistoryTransitionResponse::Remote {
                                result: Ok(23_i64),
                                fallbacks: Vec::new(),
                            }
                        }
                        HistoryTransitionCommand::Search => {
                            events.lock().unwrap().push("search");
                            HistoryTransitionResponse::Remote {
                                result: Ok(99_i64),
                                fallbacks: Vec::new(),
                            }
                        }
                        HistoryTransitionCommand::Drain => {
                            events.lock().unwrap().push("drain");
                            HistoryTransitionResponse::Drained(Vec::new())
                        }
                    }
                }
            },
            move |_attempt| {
                let events = Arc::clone(&attempt_events);
                async move {
                    events.lock().unwrap().push("attempt");
                    Ok(())
                }
            },
            |_fallbacks| async { Ok(()) },
        )
        .await
        .expect("history remains first after validation-only metadata");
        assert_eq!(result, 23);
        assert_eq!(
            *validation_events.lock().unwrap(),
            vec!["attempt", "history", "drain"]
        );
        let state: (i64, String) = sqlx::query_as(
            "SELECT only_my_messages, history_scope FROM telegram_takeout_batches WHERE batch_id = ?",
        )
        .bind(batch_id)
        .fetch_one(&pool)
        .await
        .expect("load takeout fallback state");
        assert_eq!(state, (1, "partial_private_history".to_string()));
        let warning_codes: Vec<String> =
            sqlx::query_scalar("SELECT code FROM ingest_batch_warnings WHERE batch_id = ?")
                .bind(batch_id)
                .fetch_all(&pool)
                .await
                .expect("load warning codes");
        assert_eq!(warning_codes, vec!["only_my_messages_fallback".to_string()]);
    }

    #[tokio::test]
    async fn export_dc_fallback_provenance_records_once_before_finalize() {
        async fn failed_export_state(calls: Arc<AtomicUsize>) -> FallbackRecordState {
            let fallback = takeout_fallback_fixture(
                TakeoutFallbackKind::ExportDc,
                "export fallback",
                Some("export fallback"),
            );
            let mut state = FallbackRecordState::Empty;
            queue_drained_fallbacks(&mut state, vec![fallback]).expect("queue fallback");
            let mut recorder = move |_fallbacks| {
                calls.fetch_add(1, Ordering::SeqCst);
                async { Err(AppError::internal("fallback recorder failed")) }
            };
            record_pending_once(&mut state, &mut recorder).await;
            state
        }

        let remote_error_calls = Arc::new(AtomicUsize::new(0));
        let remote_error = resolve_metadata_precedence(
            failed_export_state(Arc::clone(&remote_error_calls)).await,
            Ok(StepOutcome::<()>::Remote(Err(AppError::network(
                "remote history failed",
            )))),
        )
        .expect_err("remote error must beat ExportDc recorder error");
        assert_eq!(remote_error.kind, AppErrorKind::Network);
        assert_eq!(remote_error.message, "remote history failed");
        assert_eq!(remote_error_calls.load(Ordering::SeqCst), 1);

        let remote_success_calls = Arc::new(AtomicUsize::new(0));
        let recorder_error = resolve_metadata_precedence(
            failed_export_state(Arc::clone(&remote_success_calls)).await,
            Ok(StepOutcome::Remote(Ok("remote success"))),
        )
        .expect_err("ExportDc recorder error must beat remote success");
        assert_eq!(recorder_error.kind, AppErrorKind::Internal);
        assert_eq!(recorder_error.message, "fallback recorder failed");
        assert_eq!(remote_success_calls.load(Ordering::SeqCst), 1);

        let pool = memory_pool_with_source_items_and_topics().await;
        create_ingest_provenance_tables(&pool).await;
        seed_item_source(&pool, 1).await;
        let batch_id = create_telegram_takeout_batch(
            &pool,
            CreateTelegramTakeoutBatch {
                source_id: 1,
                account_id: 10,
                source_subtype: TELEGRAM_KIND_SUPERGROUP.to_string(),
            },
        )
        .await
        .expect("create takeout batch");
        let attempt = takeout_attempt_fixture(2, 40_002);
        let mut attempt_state = TakeoutAttemptRecordState::default();
        let mut fallback_tracker = TakeoutFallbackRecordTracker::default();
        let mut warnings = Vec::new();
        let fallback = takeout_fallback_fixture(
            TakeoutFallbackKind::ExportDc,
            "Export DC 40002 failed with local transport error; falling back to home DC 2: invalid DC",
            Some(
                "Export DC 40002 failed with local transport error; falling back to home DC 2: invalid DC",
            ),
        );
        record_export_dc_attempt_if_needed(&pool, batch_id, attempt, &mut attempt_state)
            .await
            .expect("record export DC attempt");
        record_export_dc_fallback_if_needed(
            &pool,
            batch_id,
            &mut warnings,
            &mut fallback_tracker,
            fallback.clone(),
        )
        .await
        .expect("record first export DC fallback");
        record_export_dc_fallback_if_needed(
            &pool,
            batch_id,
            &mut warnings,
            &mut fallback_tracker,
            fallback,
        )
        .await
        .expect("record duplicate fallback idempotently");
        finalize_ingest_batch(&pool, batch_id, TerminalBatchStatus::Completed, None)
            .await
            .expect("finalize batch after metadata");
        let state: (Option<i64>, i64, i64, i64) = sqlx::query_as(
            "SELECT t.export_dc_id, t.used_export_dc, t.fallback_used, b.warning_count
             FROM telegram_takeout_batches t
             JOIN ingest_batches b ON b.id = t.batch_id
             WHERE t.batch_id = ?",
        )
        .bind(batch_id)
        .fetch_one(&pool)
        .await
        .expect("load export DC provenance");
        assert_eq!(state, (Some(40_002), 1, 1, 1));
        let warning_codes: Vec<String> =
            sqlx::query_scalar("SELECT code FROM ingest_batch_warnings WHERE batch_id = ?")
                .bind(batch_id)
                .fetch_all(&pool)
                .await
                .expect("load warning codes");
        assert_eq!(warning_codes, vec!["export_dc_fallback".to_string()]);
    }

    #[tokio::test]
    async fn channel_private_validation_preflight_records_fallback_and_continues() {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_ingest_provenance_tables(&pool).await;
        seed_item_source(&pool, 1).await;
        let batch_id = create_telegram_takeout_batch(
            &pool,
            CreateTelegramTakeoutBatch {
                source_id: 1,
                account_id: 10,
                source_subtype: TELEGRAM_KIND_CHANNEL.to_string(),
            },
        )
        .await
        .expect("create takeout batch");
        let mut warnings = Vec::new();
        let mut fallback_tracker = TakeoutFallbackRecordTracker::default();
        record_takeout_fallbacks(
            &pool,
            batch_id,
            &mut warnings,
            &mut fallback_tracker,
            vec![takeout_fallback_fixture(
                TakeoutFallbackKind::OnlyMyMessages,
                "Channel history is private; falling back to messages.search(from_id=self).",
                Some(
                    "Channel history is private; importing only messages visible through from_id=self fallback.",
                ),
            )],
        )
        .await
        .expect("handle channel private validation");

        assert!(fallback_tracker.only_my_messages_recorded);
        assert_eq!(warnings.len(), 1);
        let state: (i64, String) = sqlx::query_as(
            "SELECT only_my_messages, history_scope FROM telegram_takeout_batches WHERE batch_id = ?",
        )
        .bind(batch_id)
        .fetch_one(&pool)
        .await
        .expect("load takeout fallback state");
        assert_eq!(state, (1, "partial_private_history".to_string()));
        let warning_codes: Vec<String> =
            sqlx::query_scalar("SELECT code FROM ingest_batch_warnings WHERE batch_id = ?")
                .bind(batch_id)
                .fetch_all(&pool)
                .await
                .expect("load warning codes");
        assert_eq!(warning_codes, vec!["only_my_messages_fallback".to_string()]);
    }

    #[tokio::test]
    async fn takeout_subtype_load_uses_typed_identity_not_legacy_kind() {
        let pool = memory_pool_with_sources().await;
        sqlx::query(
            r#"
            INSERT INTO sources (
                id, source_type, source_subtype, account_id,
                external_id, title, metadata_zstd, last_sync_state, is_active, is_member,
                created_at
            )
            VALUES (?, 'telegram', 'supergroup', ?, ?, ?, NULL, NULL, 1, 1, ?)
            "#,
        )
        .bind(7_i64)
        .bind(42_i64)
        .bind("12345")
        .bind("Forum source")
        .bind(1_i64)
        .execute(&pool)
        .await
        .expect("insert source");
        sqlx::query(
            r#"
            INSERT INTO telegram_sources (
                source_id, account_id, source_subtype, peer_kind, peer_id,
                resolution_strategy, username, access_hash, avatar_cache_key,
                identity_refreshed_at, created_at, updated_at
            )
            VALUES (?, ?, 'supergroup', 'channel', ?, 'legacy_metadata', NULL, ?, NULL, ?, ?, ?)
            "#,
        )
        .bind(7_i64)
        .bind(42_i64)
        .bind(12345_i64)
        .bind(98765_i64)
        .bind(1_i64)
        .bind(1_i64)
        .bind(1_i64)
        .execute(&pool)
        .await
        .expect("insert typed identity");

        let source_subtype = load_takeout_source_subtype(&pool, 7)
            .await
            .expect("load takeout source subtype");

        assert_eq!(source_subtype, TELEGRAM_KIND_SUPERGROUP);
    }

    #[tokio::test]
    async fn takeout_subtype_load_ignores_malformed_source_metadata_when_typed_identity_exists() {
        let pool = memory_pool_with_sources().await;
        sqlx::query(
            r#"
            INSERT INTO sources (
                id, source_type, source_subtype, account_id,
                external_id, title, metadata_zstd, last_sync_state, is_active, is_member,
                created_at
            )
            VALUES (?, 'telegram', 'supergroup', ?, ?, ?, x'00', NULL, 1, 1, ?)
            "#,
        )
        .bind(7_i64)
        .bind(42_i64)
        .bind("12345")
        .bind("Forum source")
        .bind(1_i64)
        .execute(&pool)
        .await
        .expect("insert source");
        sqlx::query(
            r#"
            INSERT INTO telegram_sources (
                source_id, account_id, source_subtype, peer_kind, peer_id,
                resolution_strategy, username, access_hash, avatar_cache_key,
                identity_refreshed_at, created_at, updated_at
            )
            VALUES (?, ?, 'supergroup', 'channel', ?, 'legacy_metadata', NULL, ?, NULL, ?, ?, ?)
            "#,
        )
        .bind(7_i64)
        .bind(42_i64)
        .bind(12345_i64)
        .bind(98765_i64)
        .bind(1_i64)
        .bind(1_i64)
        .bind(1_i64)
        .execute(&pool)
        .await
        .expect("insert typed identity");

        let source_subtype = load_takeout_source_subtype(&pool, 7)
            .await
            .expect("load takeout source subtype");

        assert_eq!(source_subtype, TELEGRAM_KIND_SUPERGROUP);
    }

    #[tokio::test]
    async fn takeout_parsed_items_with_same_message_id_insert_under_different_history_peers() {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_analysis_documents_table(&pool).await;
        seed_item_source(&pool, 1).await;

        let current_item = TelegramMessageDraft {
            telegram_identity: Some(TelegramMessageIdentity {
                history_peer_kind: "channel".to_string(),
                history_peer_id: 12345,
                telegram_message_id: 42,
                migration_domain: None,
                is_migrated_history: false,
            }),
            telegram_context: TelegramItemContext::default(),
            content: Some("current".to_string()),
            content_kind: "text_only",
            author: None,
            published_at: 1234,
            raw_data: br#"{"id":42,"text":"current"}"#.to_vec(),
            item_kind: ITEM_KIND_TELEGRAM_MESSAGE.to_string(),
            media: None,
        };
        let migrated_item = TelegramMessageDraft {
            telegram_identity: Some(TelegramMessageIdentity {
                history_peer_kind: "chat".to_string(),
                history_peer_id: 777,
                telegram_message_id: 42,
                migration_domain: None,
                is_migrated_history: false,
            }),
            telegram_context: TelegramItemContext::default(),
            content: Some("migrated".to_string()),
            content_kind: "text_only",
            author: None,
            published_at: 1234,
            raw_data: br#"{"id":42,"text":"migrated"}"#.to_vec(),
            item_kind: ITEM_KIND_TELEGRAM_MESSAGE.to_string(),
            media: None,
        };

        assert!(insert_telegram_source_item(&pool, 1, current_item)
            .await
            .expect("insert current"));
        assert!(insert_telegram_source_item(&pool, 1, migrated_item)
            .await
            .expect("insert migrated"));

        let item_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM items WHERE external_id = '42'")
                .fetch_one(&pool)
                .await
                .expect("count overlapping ids");
        assert_eq!(item_count, 2);
    }

    #[tokio::test]
    async fn takeout_duplicate_parsed_item_updates_topic_unresolved_count_once() {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_analysis_documents_table(&pool).await;
        seed_item_source(&pool, 1).await;
        sqlx::query(
            "INSERT INTO telegram_topic_resolution_state (
                source_id, resolver_version, status, unresolved_count, pending_item_count
             ) VALUES (1, 1, 'ready', 0, 0)",
        )
        .execute(&pool)
        .await
        .expect("seed ready topic state");

        let first_item = TelegramMessageDraft {
            telegram_identity: Some(TelegramMessageIdentity {
                history_peer_kind: "channel".to_string(),
                history_peer_id: 12345,
                telegram_message_id: 42,
                migration_domain: None,
                is_migrated_history: false,
            }),
            telegram_context: TelegramItemContext::default(),
            content: Some("first".to_string()),
            content_kind: "text_only",
            author: None,
            published_at: 1234,
            raw_data: br#"{"id":42,"text":"first"}"#.to_vec(),
            item_kind: ITEM_KIND_TELEGRAM_MESSAGE.to_string(),
            media: None,
        };
        let duplicate_item = TelegramMessageDraft {
            telegram_identity: Some(TelegramMessageIdentity {
                history_peer_kind: "channel".to_string(),
                history_peer_id: 12345,
                telegram_message_id: 42,
                migration_domain: None,
                is_migrated_history: false,
            }),
            telegram_context: TelegramItemContext::default(),
            content: Some("duplicate".to_string()),
            content_kind: "text_only",
            author: None,
            published_at: 1234,
            raw_data: br#"{"id":42,"text":"duplicate"}"#.to_vec(),
            item_kind: ITEM_KIND_TELEGRAM_MESSAGE.to_string(),
            media: None,
        };

        assert!(insert_telegram_source_item(&pool, 1, first_item)
            .await
            .expect("insert first"));
        assert!(!insert_telegram_source_item(&pool, 1, duplicate_item)
            .await
            .expect("skip duplicate"));

        let state: (String, i64) = sqlx::query_as(
            "SELECT status, unresolved_count FROM telegram_topic_resolution_state WHERE source_id = 1",
        )
        .fetch_one(&pool)
        .await
        .expect("load topic state");
        assert_eq!(state, ("ready".to_string(), 1));
    }

    #[tokio::test]
    async fn locked_start_conflict_creates_no_provenance_rows() {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_ingest_provenance_tables(&pool).await;
        seed_item_source(&pool, 1).await;
        let locks = SourceIngestLocks::new();
        let _existing = locks
            .try_acquire(1, SourceIngestKind::Sync)
            .await
            .expect("hold existing lock");
        let state = TakeoutImportState::new();

        let error = create_locked_takeout_start_records(
            &pool,
            &locks,
            &state,
            1,
            10,
            "supergroup".to_string(),
        )
        .await
        .expect_err("conflicting lock should reject start");

        assert_eq!(error.kind, AppErrorKind::Conflict);
        for table in [
            "ingest_batches",
            "telegram_takeout_batches",
            "ingest_item_observations",
            "ingest_batch_warnings",
        ] {
            let query = format!("SELECT COUNT(*) FROM {table}");
            let count: i64 = sqlx::query_scalar(&query)
                .fetch_one(&pool)
                .await
                .unwrap_or_else(|err| panic!("count {table}: {err}"));
            assert_eq!(count, 0, "unexpected rows in {table}");
        }
    }

    #[tokio::test]
    async fn locked_start_allows_only_one_batch_for_same_source() {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_ingest_provenance_tables(&pool).await;
        seed_item_source(&pool, 1).await;
        let locks = SourceIngestLocks::new();
        let state = TakeoutImportState::new();

        let first = create_locked_takeout_start_records(
            &pool,
            &locks,
            &state,
            1,
            10,
            "supergroup".to_string(),
        )
        .await
        .expect("first start");

        let second = create_locked_takeout_start_records(
            &pool,
            &locks,
            &state,
            1,
            10,
            "supergroup".to_string(),
        )
        .await;

        assert!(second.is_err());
        let batch_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ingest_batches")
            .fetch_one(&pool)
            .await
            .expect("count batches");
        assert_eq!(batch_count, 1);

        drop(first);
    }

    #[test]
    fn migrated_history_detected_warning_is_sanitized() {
        let warning = migrated_history_detected_warning();

        assert!(warning.contains("Migrated small-group history detected"));
        assert!(!warning.contains("migrated_from_chat_id"));
        assert!(!warning.contains("777"));
    }

    #[tokio::test]
    async fn migrated_history_start_records_use_same_source_takeout_lock() {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_ingest_provenance_tables(&pool).await;
        create_migrated_history_capability_tables(&pool).await;
        seed_takeout_source(&pool, 1, 10, "supergroup", "channel", 12345).await;
        crate::takeout_import::migrated_history::upsert_migrated_history_available(
            &pool, 1, 777, 100,
        )
        .await
        .expect("capability");
        let locks = SourceIngestLocks::new();
        let state = TakeoutImportState::new();
        let _current = locks
            .try_acquire(1, SourceIngestKind::TakeoutImport)
            .await
            .expect("current takeout lock");

        let error = create_locked_migrated_history_start_records(
            &pool,
            &locks,
            &state,
            1,
            10,
            "supergroup".to_string(),
        )
        .await
        .expect_err("same source historical import should be locked");

        assert_eq!(error.kind, AppErrorKind::Conflict);
    }

    #[tokio::test]
    async fn migrated_history_start_requires_available_capability() {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_ingest_provenance_tables(&pool).await;
        create_migrated_history_capability_tables(&pool).await;
        seed_takeout_source(&pool, 1, 10, "supergroup", "channel", 12345).await;
        let locks = SourceIngestLocks::new();
        let state = TakeoutImportState::new();

        let error = create_locked_migrated_history_start_records(
            &pool,
            &locks,
            &state,
            1,
            10,
            "supergroup".to_string(),
        )
        .await
        .expect_err("missing capability should fail");

        assert_eq!(error.kind, AppErrorKind::Validation);
        assert!(error.message.contains("migrated_history_not_detected"));
    }

    #[tokio::test]
    async fn historical_batch_completion_does_not_advance_source_watermark() {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_ingest_provenance_tables(&pool).await;
        seed_takeout_source(&pool, 1, 10, "supergroup", "channel", 12345).await;
        sqlx::query("UPDATE sources SET last_sync_state = 99, last_synced_at = 1000 WHERE id = 1")
            .execute(&pool)
            .await
            .expect("seed watermark");
        let batch_id = create_telegram_takeout_batch(
            &pool,
            CreateTelegramTakeoutBatch {
                source_id: 1,
                account_id: 10,
                source_subtype: "supergroup".to_string(),
            },
        )
        .await
        .expect("create batch");

        crate::ingest_provenance::mark_takeout_migrated_small_group_scope(&pool, batch_id)
            .await
            .expect("mark scope");
        crate::ingest_provenance::mark_takeout_migrated_history_imported(&pool, batch_id)
            .await
            .expect("mark imported");
        finalize_ingest_batch(&pool, batch_id, TerminalBatchStatus::Completed, None)
            .await
            .expect("finalize");

        let watermark: (Option<i64>, Option<i64>) =
            sqlx::query_as("SELECT last_sync_state, last_synced_at FROM sources WHERE id = 1")
                .fetch_one(&pool)
                .await
                .expect("load watermark");
        let batch: (String, String, i64) = sqlx::query_as(
            "SELECT b.status, t.history_scope, t.migrated_history_imported
             FROM ingest_batches b
             JOIN telegram_takeout_batches t ON t.batch_id = b.id
             WHERE b.id = ?",
        )
        .bind(batch_id)
        .fetch_one(&pool)
        .await
        .expect("load batch");

        assert_eq!(watermark, (Some(99), Some(1000)));
        assert_eq!(
            batch,
            (
                "completed".to_string(),
                "migrated_small_group_history".to_string(),
                1,
            )
        );
    }

    async fn seed_item_source(pool: &sqlx::SqlitePool, source_id: i64) {
        sqlx::query(
            "INSERT OR IGNORE INTO sources (id, source_type, source_subtype, external_id, title, is_active, is_member, created_at)
             VALUES (?, 'telegram', 'supergroup', '12345', 'Forum', 1, 1, 1)",
        )
        .bind(source_id)
        .execute(pool)
        .await
        .expect("seed source");
    }

    async fn seed_takeout_source(
        pool: &sqlx::SqlitePool,
        source_id: i64,
        account_id: i64,
        source_subtype: &str,
        peer_kind: &str,
        peer_id: i64,
    ) {
        sqlx::query(
            "INSERT OR IGNORE INTO sources (
                id, source_type, source_subtype, account_id, external_id, title,
                is_active, is_member, created_at
             ) VALUES (?, 'telegram', ?, ?, ?, 'Forum', 1, 1, 1)",
        )
        .bind(source_id)
        .bind(source_subtype)
        .bind(account_id)
        .bind(peer_id.to_string())
        .execute(pool)
        .await
        .expect("seed source");
        sqlx::query(
            "INSERT INTO telegram_sources (
                source_id, account_id, source_subtype, peer_kind, peer_id, resolution_strategy
             ) VALUES (?, ?, ?, ?, ?, 'dialog')",
        )
        .bind(source_id)
        .bind(account_id)
        .bind(source_subtype)
        .bind(peer_kind)
        .bind(peer_id)
        .execute(pool)
        .await
        .expect("seed telegram source");
    }
}
