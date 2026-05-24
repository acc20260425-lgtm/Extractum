#![allow(clippy::needless_borrow, clippy::too_many_arguments)]

use grammers_client::{tl, Client};
use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::ingest_provenance::{
    create_telegram_takeout_batch, finalize_ingest_batch, mark_takeout_export_dc_attempted,
    mark_takeout_export_dc_fallback, mark_takeout_migrated_history_deferred,
    mark_takeout_only_my_messages_fallback, record_ingest_batch_warning,
    update_takeout_max_message_id, update_takeout_resolved_peer, update_takeout_session_started,
    update_takeout_split_metadata, CreateTelegramTakeoutBatch, TerminalBatchStatus,
};
use crate::source_ingest::{SourceIngestGuard, SourceIngestKind, SourceIngestLocks};
use crate::sources::{
    finalize_sync, load_source, require_source_identity_ready, resolve_and_refresh_peer,
    SourceIdentityRepairState, TelegramSourceKind, TELEGRAM_KIND_CHANNEL, TELEGRAM_KIND_GROUP,
    TELEGRAM_KIND_SUPERGROUP,
};
use crate::telegram::{get_authorized_runtime, AuthorizedTelegramRuntime, TelegramState};
use grammers_session::types::{PeerKind, PeerRef};

mod export_dc;
mod pagination;
#[allow(dead_code)]
mod raw_parse;
mod recovery;
mod state;
#[allow(dead_code)]
mod validation_diagnostics;

use export_dc::{
    export_dc_invoke, finish_takeout_session, prepare_export_dc_alias,
    takeout_init_request_for_source_subtype, ExportDcAlias, ExportDcAttemptState,
};
use pagination::{
    message_range_min_id, next_takeout_cursor, parse_takeout_page, select_history_splits,
    should_restart_with_descending_fallback, takeout_page_request,
    takeout_pagination_fallback_warning, TakeoutPageRequest, TakeoutPaginationCursor,
    TakeoutPaginationProfile,
};
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
    let record = state.create_job(source_id, account_id, batch_id).await?;
    Ok((record, ingest_guard))
}

async fn record_export_dc_attempt_if_needed(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    batch_id: i64,
    alias: &ExportDcAlias,
    attempts: &mut ExportDcAttemptState,
) -> AppResult<()> {
    if attempts.mark_attempted(alias.export_dc_id) {
        mark_takeout_export_dc_attempted(pool, batch_id, alias.export_dc_id).await?;
    }
    Ok(())
}

async fn record_export_dc_fallback_if_needed(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    batch_id: i64,
    warnings: &[String],
    fallback_before: bool,
    fallback_after: bool,
    attempts: &mut ExportDcAttemptState,
) -> AppResult<()> {
    if !fallback_before && fallback_after {
        let message = warnings
            .last()
            .cloned()
            .unwrap_or_else(|| "Export DC fallback was used.".to_string());
        if let Some(message) = attempts.mark_fallback(message) {
            mark_takeout_export_dc_fallback(pool, batch_id, &message).await?;
        }
    }
    Ok(())
}

async fn export_dc_invoke_with_provenance<R: tl::RemoteCall>(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    batch_id: i64,
    client: &Client,
    alias: &ExportDcAlias,
    request: &R,
    warnings: &mut Vec<String>,
    fallback_used: &mut bool,
    attempts: &mut ExportDcAttemptState,
) -> AppResult<R::Return> {
    let fallback_before = *fallback_used;
    record_export_dc_attempt_if_needed(pool, batch_id, alias, attempts).await?;
    let response = export_dc_invoke(client, alias, request, warnings, fallback_used).await;
    match response {
        Ok(response) => {
            record_export_dc_fallback_if_needed(
                pool,
                batch_id,
                warnings,
                fallback_before,
                *fallback_used,
                attempts,
            )
            .await?;
            Ok(response)
        }
        Err(error) => {
            let _ = record_export_dc_fallback_if_needed(
                pool,
                batch_id,
                warnings,
                fallback_before,
                *fallback_used,
                attempts,
            )
            .await;
            Err(error)
        }
    }
}

fn peer_ref_identity(peer: PeerRef) -> (&'static str, i64) {
    let kind = match peer.id.kind() {
        PeerKind::User | PeerKind::UserSelf => "user",
        PeerKind::Chat => "chat",
        PeerKind::Channel => "channel",
    };
    (kind, peer.id.bare_id())
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
    let runtime = get_authorized_runtime(&state, account_id).await?;

    run_export_dc_spike_for_runtime(source.id, account_id, &telegram_source_subtype, runtime).await
}

async fn run_export_dc_spike_for_runtime(
    source_id: i64,
    account_id: i64,
    telegram_source_subtype: &str,
    runtime: AuthorizedTelegramRuntime,
) -> AppResult<TakeoutExportDcSpikeResult> {
    let client = runtime.client;
    client
        .invoke(&tl::functions::users::GetUsers {
            id: vec![tl::enums::InputUser::UserSelf],
        })
        .await
        .map_err(|e| AppError::network(format!("Telegram self check failed: {e}")))?;

    let alias = prepare_export_dc_alias(&runtime.session).await?;
    let init_request = takeout_init_request_for_source_subtype(telegram_source_subtype)?;
    let mut warnings = Vec::new();
    let mut fallback_used = false;

    let takeout = export_dc_invoke(
        &client,
        &alias,
        &init_request,
        &mut warnings,
        &mut fallback_used,
    )
    .await?;
    let tl::enums::account::Takeout::Takeout(takeout) = takeout;
    let takeout_id = takeout.id;

    let split_ranges = export_dc_invoke(
        &client,
        &alias,
        &tl::functions::InvokeWithTakeout {
            takeout_id,
            query: tl::functions::messages::GetSplitRanges {},
        },
        &mut warnings,
        &mut fallback_used,
    )
    .await?;

    export_dc_invoke(
        &client,
        &alias,
        &tl::functions::InvokeWithTakeout {
            takeout_id,
            query: tl::functions::account::FinishTakeoutSession { success: true },
        },
        &mut warnings,
        &mut fallback_used,
    )
    .await?;

    Ok(TakeoutExportDcSpikeResult {
        source_id,
        account_id,
        telegram_source_subtype: telegram_source_subtype.to_string(),
        home_dc_id: alias.home_dc_id,
        export_dc_id: alias.export_dc_id,
        used_export_dc: !fallback_used,
        fallback_used,
        takeout_id,
        split_count: split_ranges.len(),
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

struct TakeoutImportOutcome {
    inserted: i64,
    skipped: i64,
    progress_total: Option<i64>,
    warnings: Vec<String>,
}

async fn run_takeout_source_import(
    handle: &AppHandle,
    job_id: &str,
    batch_id: i64,
) -> AppResult<TakeoutImportOutcome> {
    let takeout_state = handle.state::<TakeoutImportState>();
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
    let runtime = get_authorized_runtime(&telegram_state, account_id).await?;
    let client = runtime.client;
    let session = runtime.session;

    update_and_emit(handle, &takeout_state, job_id, |job| {
        job.phase = PHASE_RESOLVING_SOURCE.to_string();
        job.message = Some("Resolving Telegram source.".to_string());
    })
    .await;
    let resolved_peer =
        resolve_and_refresh_peer(handle, &pool, &client, &source, account_id).await?;
    let (resolved_peer_kind, resolved_peer_id) = peer_ref_identity(resolved_peer.peer);
    update_takeout_resolved_peer(
        &pool,
        batch_id,
        resolved_peer_kind,
        resolved_peer_id,
        resolved_peer_kind,
        resolved_peer_id,
    )
    .await?;

    update_and_emit(handle, &takeout_state, job_id, |job| {
        job.phase = PHASE_STARTING_TAKEOUT.to_string();
        job.message = Some("Starting Takeout session.".to_string());
    })
    .await;
    client
        .invoke(&tl::functions::users::GetUsers {
            id: vec![tl::enums::InputUser::UserSelf],
        })
        .await
        .map_err(|e| AppError::network(format!("Telegram self check failed: {e}")))?;
    let alias = prepare_export_dc_alias(&session).await?;
    let init_request = takeout_init_request_for_source_subtype(&telegram_source_subtype)?;
    let mut warnings = Vec::new();
    let mut fallback_used = false;
    let mut export_attempts = ExportDcAttemptState::new();
    let takeout = export_dc_invoke_with_provenance(
        &pool,
        batch_id,
        &client,
        &alias,
        &init_request,
        &mut warnings,
        &mut fallback_used,
        &mut export_attempts,
    )
    .await?;
    let tl::enums::account::Takeout::Takeout(takeout) = takeout;
    let takeout_id = takeout.id;
    update_takeout_session_started(&pool, batch_id, takeout_id).await?;

    let started_result = run_started_takeout_source_import(
        handle,
        job_id,
        batch_id,
        &pool,
        &source,
        &telegram_source_subtype,
        resolved_peer,
        &client,
        &alias,
        takeout_id,
        warnings,
        fallback_used,
        &mut export_attempts,
    )
    .await;

    match started_result {
        Ok(outcome) => Ok(outcome),
        Err((error, mut warnings, mut fallback_used)) => {
            let fallback_before = fallback_used;
            let _ =
                record_export_dc_attempt_if_needed(&pool, batch_id, &alias, &mut export_attempts)
                    .await;
            if let Err(finish_error) = finish_takeout_session(
                &client,
                &alias,
                takeout_id,
                false,
                &mut warnings,
                &mut fallback_used,
            )
            .await
            {
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
            let _ = record_export_dc_fallback_if_needed(
                &pool,
                batch_id,
                &warnings,
                fallback_before,
                fallback_used,
                &mut export_attempts,
            )
            .await;
            update_and_emit(handle, &takeout_state, job_id, |job| {
                job.warnings = warnings;
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
    client: &Client,
    alias: &ExportDcAlias,
    takeout_id: i64,
    mut warnings: Vec<String>,
    mut fallback_used: bool,
    export_attempts: &mut ExportDcAttemptState,
) -> Result<TakeoutImportOutcome, (AppError, Vec<String>, bool)> {
    match run_started_takeout_source_import_inner(
        handle,
        job_id,
        batch_id,
        pool,
        source,
        telegram_source_subtype,
        resolved_peer,
        client,
        alias,
        takeout_id,
        &mut warnings,
        &mut fallback_used,
        export_attempts,
    )
    .await
    {
        Ok(outcome) => Ok(outcome),
        Err(error) => Err((error, warnings, fallback_used)),
    }
}

async fn run_started_takeout_source_import_inner(
    handle: &AppHandle,
    job_id: &str,
    batch_id: i64,
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source: &crate::sources::SourceSyncTarget,
    telegram_source_subtype: &str,
    resolved_peer: crate::sources::ResolvedSyncPeer,
    client: &Client,
    alias: &ExportDcAlias,
    takeout_id: i64,
    warnings: &mut Vec<String>,
    fallback_used: &mut bool,
    export_attempts: &mut ExportDcAttemptState,
) -> AppResult<TakeoutImportOutcome> {
    let takeout_state = handle.state::<TakeoutImportState>();
    let input_peer: tl::enums::InputPeer = resolved_peer.peer.into();
    let mut only_my_messages_recorded = false;

    update_and_emit(handle, &takeout_state, job_id, |job| {
        job.phase = PHASE_VALIDATING_PEER.to_string();
        job.message = Some("Validating Telegram source.".to_string());
        job.warnings.extend(warnings.clone());
    })
    .await;
    validate_takeout_peer(
        pool,
        batch_id,
        &client,
        &alias,
        takeout_id,
        telegram_source_subtype,
        resolved_peer.peer,
        warnings,
        fallback_used,
        export_attempts,
        &mut only_my_messages_recorded,
    )
    .await?;
    let migrated_detected = detect_supergroup_migration(
        pool,
        batch_id,
        client,
        alias,
        takeout_id,
        telegram_source_subtype,
        resolved_peer.peer,
        warnings,
        fallback_used,
        export_attempts,
    )
    .await?;
    if migrated_detected {
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
    let split_ranges = export_dc_invoke_with_provenance(
        pool,
        batch_id,
        &client,
        &alias,
        &tl::functions::InvokeWithTakeout {
            takeout_id,
            query: tl::functions::messages::GetSplitRanges {},
        },
        warnings,
        fallback_used,
        export_attempts,
    )
    .await?;
    let split_count = split_ranges.len() as i64;
    let selected_ranges = select_history_splits(telegram_source_subtype, split_ranges)?;
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
        let probe = takeout_history_count_probe(
            pool,
            batch_id,
            &client,
            &alias,
            takeout_id,
            input_peer.clone(),
            range.clone(),
            telegram_source_subtype,
            warnings,
            fallback_used,
            export_attempts,
            &mut only_my_messages_recorded,
        )
        .await?;
        total += probe.count;
        counted_ranges.push(CountedMessageRange {
            range,
            count: probe.count,
            only_my_messages: probe.only_my_messages,
        });
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
        &client,
        &alias,
        takeout_id,
        input_peer,
        counted_ranges,
        &source,
        total,
        telegram_source_subtype,
        warnings,
        fallback_used,
        export_attempts,
        &mut only_my_messages_recorded,
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
    let fallback_before = *fallback_used;
    record_export_dc_attempt_if_needed(pool, batch_id, alias, export_attempts).await?;
    finish_takeout_session(client, alias, takeout_id, true, warnings, fallback_used).await?;
    record_export_dc_fallback_if_needed(
        pool,
        batch_id,
        warnings,
        fallback_before,
        *fallback_used,
        export_attempts,
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
    range: tl::enums::MessageRange,
    count: i64,
    only_my_messages: bool,
}

struct TakeoutHistoryProbe {
    count: i64,
    only_my_messages: bool,
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

async fn validate_takeout_peer(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    batch_id: i64,
    client: &Client,
    alias: &ExportDcAlias,
    takeout_id: i64,
    telegram_source_subtype: &str,
    peer: grammers_session::types::PeerRef,
    warnings: &mut Vec<String>,
    fallback_used: &mut bool,
    export_attempts: &mut ExportDcAttemptState,
    only_my_messages_recorded: &mut bool,
) -> AppResult<()> {
    match telegram_source_subtype {
        TELEGRAM_KIND_CHANNEL | TELEGRAM_KIND_SUPERGROUP => {
            let input_channel: tl::enums::InputChannel = peer.into();
            let result = export_dc_invoke_with_provenance(
                pool,
                batch_id,
                client,
                alias,
                &tl::functions::InvokeWithTakeout {
                    takeout_id,
                    query: tl::functions::channels::GetChannels {
                        id: vec![input_channel],
                    },
                },
                warnings,
                fallback_used,
                export_attempts,
            )
            .await;
            if let Err(error) = result {
                if record_channel_private_fallback_if_supported(
                    pool,
                    batch_id,
                    telegram_source_subtype,
                    &error,
                    warnings,
                    only_my_messages_recorded,
                )
                .await?
                {
                    return Ok(());
                }
                return Err(error);
            }
        }
        TELEGRAM_KIND_GROUP => {
            export_dc_invoke_with_provenance(
                pool,
                batch_id,
                client,
                alias,
                &tl::functions::InvokeWithTakeout {
                    takeout_id,
                    query: tl::functions::messages::GetChats {
                        id: vec![peer.id.bare_id()],
                    },
                },
                warnings,
                fallback_used,
                export_attempts,
            )
            .await?;
        }
        other => {
            return Err(AppError::validation(format!(
                "Unsupported Telegram source_subtype '{other}'"
            )));
        }
    }

    Ok(())
}

async fn detect_supergroup_migration(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    batch_id: i64,
    client: &Client,
    alias: &ExportDcAlias,
    takeout_id: i64,
    telegram_source_subtype: &str,
    peer: grammers_session::types::PeerRef,
    warnings: &mut Vec<String>,
    fallback_used: &mut bool,
    export_attempts: &mut ExportDcAttemptState,
) -> AppResult<bool> {
    if telegram_source_subtype != TELEGRAM_KIND_SUPERGROUP {
        return Ok(false);
    }

    let input_channel: tl::enums::InputChannel = peer.into();
    let chat_full = export_dc_invoke_with_provenance(
        pool,
        batch_id,
        client,
        alias,
        &tl::functions::InvokeWithTakeout {
            takeout_id,
            query: tl::functions::channels::GetFullChannel {
                channel: input_channel,
            },
        },
        warnings,
        fallback_used,
        export_attempts,
    )
    .await?;

    let tl::enums::messages::ChatFull::Full(chat_full) = chat_full;
    if let tl::enums::ChatFull::ChannelFull(full) = chat_full.full_chat {
        if let Some(migrated_from_chat_id) = full.migrated_from_chat_id {
            warnings.push(format!(
                "Supergroup migrated_from_chat_id {migrated_from_chat_id} detected; migrated history import is deferred to avoid source item id collisions."
            ));
            return Ok(true);
        }
    }

    Ok(false)
}

async fn takeout_history_count_probe(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    batch_id: i64,
    client: &Client,
    alias: &ExportDcAlias,
    takeout_id: i64,
    input_peer: tl::enums::InputPeer,
    range: tl::enums::MessageRange,
    telegram_source_subtype: &str,
    warnings: &mut Vec<String>,
    fallback_used: &mut bool,
    export_attempts: &mut ExportDcAttemptState,
    only_my_messages_recorded: &mut bool,
) -> AppResult<TakeoutHistoryProbe> {
    let response = takeout_get_history(
        pool,
        batch_id,
        client,
        alias,
        takeout_id,
        input_peer.clone(),
        range.clone(),
        0,
        0,
        1,
        warnings,
        fallback_used,
        export_attempts,
    )
    .await;

    let response = match response {
        Ok(response) => response,
        Err(error)
            if supports_only_my_messages_fallback(telegram_source_subtype)
                && is_channel_private_error(&error) =>
        {
            record_only_my_messages_fallback_if_needed(
                pool,
                batch_id,
                warnings,
                only_my_messages_recorded,
            )
            .await?;
            let search_response = takeout_search_my_messages(
                pool,
                batch_id,
                client,
                alias,
                takeout_id,
                input_peer,
                range,
                0,
                0,
                1,
                warnings,
                fallback_used,
                export_attempts,
            )
            .await?;
            return Ok(TakeoutHistoryProbe {
                count: messages_response_count(search_response)?,
                only_my_messages: true,
            });
        }
        Err(error) => return Err(error),
    };

    Ok(TakeoutHistoryProbe {
        count: messages_response_count(response)?,
        only_my_messages: false,
    })
}

async fn import_takeout_history_ranges(
    handle: &AppHandle,
    job_id: &str,
    batch_id: i64,
    client: &Client,
    alias: &ExportDcAlias,
    takeout_id: i64,
    input_peer: tl::enums::InputPeer,
    ranges: Vec<CountedMessageRange>,
    source: &crate::sources::SourceSyncTarget,
    total: i64,
    telegram_source_subtype: &str,
    warnings: &mut Vec<String>,
    fallback_used: &mut bool,
    export_attempts: &mut ExportDcAttemptState,
    only_my_messages_recorded: &mut bool,
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
            client,
            alias,
            takeout_id,
            input_peer.clone(),
            counted_range,
            source,
            total,
            telegram_source_subtype,
            imported,
            warnings,
            fallback_used,
            export_attempts,
            only_my_messages_recorded,
        )
        .await?;
    }

    Ok(imported)
}

async fn import_takeout_history_pages(
    handle: &AppHandle,
    job_id: &str,
    batch_id: i64,
    client: &Client,
    alias: &ExportDcAlias,
    takeout_id: i64,
    input_peer: tl::enums::InputPeer,
    counted_range: CountedMessageRange,
    source: &crate::sources::SourceSyncTarget,
    total: i64,
    telegram_source_subtype: &str,
    mut imported: TakeoutHistoryImport,
    warnings: &mut Vec<String>,
    fallback_used: &mut bool,
    export_attempts: &mut ExportDcAttemptState,
    only_my_messages_recorded: &mut bool,
) -> AppResult<TakeoutHistoryImport> {
    let takeout_state = handle.state::<TakeoutImportState>();
    let pool = get_pool(handle).await?;
    let range = counted_range.range;
    let split_count = counted_range.count;
    let mut only_my_messages = counted_range.only_my_messages;
    let mut profile = TakeoutPaginationProfile::TDesktop;
    let mut cursor = TakeoutPaginationCursor::new(profile, &range);
    let mut page_index = 0_usize;

    loop {
        if takeout_state.is_cancel_requested(job_id).await {
            return Err(AppError::validation("Takeout import cancelled"));
        }

        let request = takeout_page_request(cursor);
        let response = takeout_history_page_response(
            &pool,
            batch_id,
            client,
            alias,
            takeout_id,
            input_peer.clone(),
            range.clone(),
            request,
            telegram_source_subtype,
            &mut only_my_messages,
            warnings,
            fallback_used,
            export_attempts,
            only_my_messages_recorded,
        )
        .await?;
        let page = parse_takeout_page(response, profile)?;
        let advance = next_takeout_cursor(cursor, &page, &range);

        if let Some(reason) = should_restart_with_descending_fallback(
            profile,
            split_count,
            page_index,
            &page,
            advance,
        ) {
            push_warning_once(
                warnings,
                takeout_pagination_fallback_warning(reason, &range),
            );
            update_and_emit(handle, &takeout_state, job_id, |job| {
                job.warnings = warnings.clone();
            })
            .await;
            profile = TakeoutPaginationProfile::DescendingFallback;
            cursor = TakeoutPaginationCursor::new(profile, &range);
            page_index = 0;
            continue;
        }

        if page.messages.is_empty() {
            break;
        }

        for message in page.messages {
            let message_id = message.id;
            if message_id <= message_range_min_id(&range) {
                continue;
            }
            let next_max_message_id = imported.max_message_id.max(i64::from(message_id));
            if next_max_message_id != imported.max_message_id {
                imported.max_message_id = next_max_message_id;
                update_takeout_max_message_id(&pool, batch_id, imported.max_message_id).await?;
            }
            match raw_parse::parse_raw_message(&source.title, message) {
                Ok(Some(item)) => {
                    let identity = item.telegram_identity.clone().ok_or_else(|| {
                        AppError::validation(
                            "Parsed Takeout Telegram item is missing native message identity",
                        )
                    })?;
                    match crate::sources::insert_telegram_source_item_with_observation(
                        &pool, batch_id, source.id, identity, item,
                    )
                    .await?
                    {
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
                Err(error) => return Err(AppError::internal(error)),
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

        if page.is_terminal_response || !advance.advanced || advance.reached_range_start {
            break;
        }
        cursor = advance.cursor;
        page_index += 1;
    }

    Ok(imported)
}

async fn takeout_history_page_response(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    batch_id: i64,
    client: &Client,
    alias: &ExportDcAlias,
    takeout_id: i64,
    input_peer: tl::enums::InputPeer,
    range: tl::enums::MessageRange,
    request: TakeoutPageRequest,
    telegram_source_subtype: &str,
    only_my_messages: &mut bool,
    warnings: &mut Vec<String>,
    fallback_used: &mut bool,
    export_attempts: &mut ExportDcAttemptState,
    only_my_messages_recorded: &mut bool,
) -> AppResult<tl::enums::messages::Messages> {
    if *only_my_messages {
        return takeout_search_my_messages(
            pool,
            batch_id,
            client,
            alias,
            takeout_id,
            input_peer,
            range,
            request.offset_id,
            request.add_offset,
            request.limit,
            warnings,
            fallback_used,
            export_attempts,
        )
        .await;
    }

    match takeout_get_history(
        pool,
        batch_id,
        client,
        alias,
        takeout_id,
        input_peer.clone(),
        range.clone(),
        request.offset_id,
        request.add_offset,
        request.limit,
        warnings,
        fallback_used,
        export_attempts,
    )
    .await
    {
        Ok(response) => Ok(response),
        Err(error)
            if supports_only_my_messages_fallback(telegram_source_subtype)
                && is_channel_private_error(&error) =>
        {
            *only_my_messages = true;
            record_only_my_messages_fallback_if_needed(
                pool,
                batch_id,
                warnings,
                only_my_messages_recorded,
            )
            .await?;
            takeout_search_my_messages(
                pool,
                batch_id,
                client,
                alias,
                takeout_id,
                input_peer,
                range,
                request.offset_id,
                request.add_offset,
                request.limit,
                warnings,
                fallback_used,
                export_attempts,
            )
            .await
        }
        Err(error) => Err(error),
    }
}

fn push_warning_once(warnings: &mut Vec<String>, warning: impl Into<String>) {
    let warning = warning.into();
    if !warnings.iter().any(|existing| existing == &warning) {
        warnings.push(warning);
    }
}

async fn record_only_my_messages_fallback_if_needed(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    batch_id: i64,
    warnings: &mut Vec<String>,
    only_my_messages_recorded: &mut bool,
) -> AppResult<()> {
    push_warning_once(
        warnings,
        "Channel history is private; falling back to messages.search(from_id=self).",
    );
    if !*only_my_messages_recorded {
        mark_takeout_only_my_messages_fallback(
            pool,
            batch_id,
            "Channel history is private; importing only messages visible through from_id=self fallback.",
        )
        .await?;
        *only_my_messages_recorded = true;
    }
    Ok(())
}

async fn record_channel_private_fallback_if_supported(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    batch_id: i64,
    telegram_source_subtype: &str,
    error: &AppError,
    warnings: &mut Vec<String>,
    only_my_messages_recorded: &mut bool,
) -> AppResult<bool> {
    if supports_only_my_messages_fallback(telegram_source_subtype)
        && is_channel_private_error(error)
    {
        record_only_my_messages_fallback_if_needed(
            pool,
            batch_id,
            warnings,
            only_my_messages_recorded,
        )
        .await?;
        return Ok(true);
    }
    Ok(false)
}

async fn takeout_get_history(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    batch_id: i64,
    client: &Client,
    alias: &ExportDcAlias,
    takeout_id: i64,
    input_peer: tl::enums::InputPeer,
    range: tl::enums::MessageRange,
    offset_id: i32,
    add_offset: i32,
    limit: i32,
    warnings: &mut Vec<String>,
    fallback_used: &mut bool,
    export_attempts: &mut ExportDcAttemptState,
) -> AppResult<tl::enums::messages::Messages> {
    export_dc_invoke_with_provenance(
        pool,
        batch_id,
        client,
        alias,
        &tl::functions::InvokeWithTakeout {
            takeout_id,
            query: tl::functions::InvokeWithMessagesRange {
                range,
                query: tl::functions::messages::GetHistory {
                    peer: input_peer,
                    offset_id,
                    offset_date: 0,
                    add_offset,
                    limit,
                    max_id: 0,
                    min_id: 0,
                    hash: 0,
                },
            },
        },
        warnings,
        fallback_used,
        export_attempts,
    )
    .await
}

async fn takeout_search_my_messages(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    batch_id: i64,
    client: &Client,
    alias: &ExportDcAlias,
    takeout_id: i64,
    input_peer: tl::enums::InputPeer,
    range: tl::enums::MessageRange,
    offset_id: i32,
    add_offset: i32,
    limit: i32,
    warnings: &mut Vec<String>,
    fallback_used: &mut bool,
    export_attempts: &mut ExportDcAttemptState,
) -> AppResult<tl::enums::messages::Messages> {
    export_dc_invoke_with_provenance(
        pool,
        batch_id,
        client,
        alias,
        &tl::functions::InvokeWithTakeout {
            takeout_id,
            query: tl::functions::InvokeWithMessagesRange {
                range,
                query: tl::functions::messages::Search {
                    peer: input_peer,
                    q: String::new(),
                    from_id: Some(tl::enums::InputPeer::PeerSelf),
                    saved_peer_id: None,
                    saved_reaction: None,
                    top_msg_id: None,
                    filter: tl::enums::MessagesFilter::InputMessagesFilterEmpty,
                    min_date: 0,
                    max_date: 0,
                    offset_id,
                    add_offset,
                    limit,
                    max_id: 0,
                    min_id: 0,
                    hash: 0,
                },
            },
        },
        warnings,
        fallback_used,
        export_attempts,
    )
    .await
}

fn supports_only_my_messages_fallback(telegram_source_subtype: &str) -> bool {
    matches!(
        telegram_source_subtype,
        TELEGRAM_KIND_CHANNEL | TELEGRAM_KIND_SUPERGROUP
    )
}

fn is_channel_private_error(error: &AppError) -> bool {
    error
        .message
        .to_ascii_uppercase()
        .contains("CHANNEL_PRIVATE")
}

fn messages_response_count(response: tl::enums::messages::Messages) -> AppResult<i64> {
    match response {
        tl::enums::messages::Messages::Messages(messages) => Ok(messages.messages.len() as i64),
        tl::enums::messages::Messages::Slice(messages) => Ok(i64::from(messages.count)),
        tl::enums::messages::Messages::ChannelMessages(messages) => Ok(i64::from(messages.count)),
        tl::enums::messages::Messages::NotModified(_) => Err(AppError::network(
            "Telegram returned messagesNotModified for Takeout history count probe",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        create_locked_takeout_start_records, is_channel_private_error, load_takeout_source_subtype,
        raw_parse, record_channel_private_fallback_if_supported,
        record_only_my_messages_fallback_if_needed, supports_only_my_messages_fallback,
        TELEGRAM_KIND_CHANNEL, TELEGRAM_KIND_GROUP, TELEGRAM_KIND_SUPERGROUP,
    };
    use crate::error::{AppError, AppErrorKind};
    use crate::ingest_provenance::{create_telegram_takeout_batch, CreateTelegramTakeoutBatch};
    use crate::source_ingest::{SourceIngestKind, SourceIngestLocks};
    use crate::sources::insert_telegram_source_item;
    use crate::sources::test_support::{
        create_analysis_documents_table, create_ingest_provenance_tables,
        memory_pool_with_source_items_and_topics, memory_pool_with_sources,
    };
    use crate::takeout_import::state::TakeoutImportState;
    use grammers_client::tl;

    #[test]
    fn only_my_messages_fallback_is_limited_to_channels() {
        assert!(supports_only_my_messages_fallback(TELEGRAM_KIND_CHANNEL));
        assert!(supports_only_my_messages_fallback(TELEGRAM_KIND_SUPERGROUP));
        assert!(!supports_only_my_messages_fallback(TELEGRAM_KIND_GROUP));
    }

    #[test]
    fn channel_private_detection_reads_rpc_name_from_error_message() {
        assert!(is_channel_private_error(&AppError::network(
            "Rpc error 400: CHANNEL_PRIVATE"
        )));
        assert!(!is_channel_private_error(&AppError::network(
            "Rpc error 400: TAKEOUT_INVALID"
        )));
    }

    #[tokio::test]
    async fn channel_private_count_probe_records_fallback_before_search_continuation() {
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
        let mut only_my_messages_recorded = false;

        record_only_my_messages_fallback_if_needed(
            &pool,
            batch_id,
            &mut warnings,
            &mut only_my_messages_recorded,
        )
        .await
        .expect("record fallback");
        record_only_my_messages_fallback_if_needed(
            &pool,
            batch_id,
            &mut warnings,
            &mut only_my_messages_recorded,
        )
        .await
        .expect("record fallback idempotently");

        assert!(only_my_messages_recorded);
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
        let mut only_my_messages_recorded = false;

        let should_continue = record_channel_private_fallback_if_supported(
            &pool,
            batch_id,
            TELEGRAM_KIND_CHANNEL,
            &AppError::network("Rpc error 400: CHANNEL_PRIVATE"),
            &mut warnings,
            &mut only_my_messages_recorded,
        )
        .await
        .expect("handle channel private validation");

        assert!(should_continue);
        assert!(only_my_messages_recorded);
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

        let current = takeout_raw_message_for_identity_test(
            42,
            tl::types::PeerChannel { channel_id: 12345 }.into(),
            "current",
        );
        let migrated = takeout_raw_message_for_identity_test(
            42,
            tl::types::PeerChat { chat_id: 777 }.into(),
            "migrated",
        );

        let current_item = raw_parse::parse_raw_message(&None, current)
            .expect("parse current")
            .expect("current item");
        let current_identity = current_item
            .telegram_identity
            .clone()
            .expect("current identity");
        let migrated_item = raw_parse::parse_raw_message(&None, migrated)
            .expect("parse migrated")
            .expect("migrated item");
        let migrated_identity = migrated_item
            .telegram_identity
            .clone()
            .expect("migrated identity");

        assert!(
            insert_telegram_source_item(&pool, 1, current_identity, current_item)
                .await
                .expect("insert current")
        );
        assert!(
            insert_telegram_source_item(&pool, 1, migrated_identity, migrated_item)
                .await
                .expect("insert migrated")
        );

        let item_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM items WHERE source_id = 1 AND external_id = '42'",
        )
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

        let first = takeout_raw_message_for_identity_test(
            42,
            tl::types::PeerChannel { channel_id: 12345 }.into(),
            "first",
        );
        let duplicate = takeout_raw_message_for_identity_test(
            42,
            tl::types::PeerChannel { channel_id: 12345 }.into(),
            "duplicate",
        );

        let first_item = raw_parse::parse_raw_message(&None, first)
            .expect("parse first")
            .expect("first item");
        let first_identity = first_item
            .telegram_identity
            .clone()
            .expect("first identity");
        let duplicate_item = raw_parse::parse_raw_message(&None, duplicate)
            .expect("parse duplicate")
            .expect("duplicate item");
        let duplicate_identity = duplicate_item
            .telegram_identity
            .clone()
            .expect("duplicate identity");

        assert!(
            insert_telegram_source_item(&pool, 1, first_identity, first_item)
                .await
                .expect("insert first")
        );
        assert!(
            !insert_telegram_source_item(&pool, 1, duplicate_identity, duplicate_item)
                .await
                .expect("skip duplicate")
        );

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

    fn takeout_raw_message_for_identity_test(
        id: i32,
        peer_id: tl::enums::Peer,
        text: &str,
    ) -> tl::types::Message {
        tl::types::Message {
            out: false,
            mentioned: false,
            media_unread: false,
            silent: false,
            post: false,
            from_scheduled: false,
            legacy: false,
            edit_hide: false,
            pinned: false,
            noforwards: false,
            invert_media: false,
            offline: false,
            video_processing_pending: false,
            paid_suggested_post_stars: false,
            paid_suggested_post_ton: false,
            id,
            from_id: None,
            from_boosts_applied: None,
            peer_id,
            saved_peer_id: None,
            fwd_from: None,
            via_bot_id: None,
            via_business_bot_id: None,
            reply_to: None,
            date: 1234,
            message: text.to_string(),
            media: None,
            reply_markup: None,
            entities: None,
            views: None,
            forwards: None,
            replies: None,
            edit_date: None,
            post_author: None,
            grouped_id: None,
            reactions: None,
            restriction_reason: None,
            ttl_period: None,
            quick_reply_shortcut_id: None,
            effect: None,
            factcheck: None,
            report_delivery_until_date: None,
            paid_message_stars: None,
            suggested_post: None,
            schedule_repeat_period: None,
            summary_from_language: None,
        }
    }
}
