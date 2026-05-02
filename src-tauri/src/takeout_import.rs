use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use grammers_client::{tl, Client};
use grammers_mtsender::InvocationError;
use grammers_session::{storages::MemorySession, Session};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::Mutex;

use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::source_ingest::{SourceIngestKind, SourceIngestLocks};
use crate::sources::{finalize_sync, insert_source_item, load_source, resolve_and_refresh_peer};
use crate::telegram::{get_authorized_runtime, TelegramState};

#[allow(dead_code)]
mod raw_parse;

const TAKEOUT_IMPORT_EVENT: &str = "sources://takeout-import";
const EXPORT_DC_SHIFT: i32 = 4 * 10_000;
const TAKEOUT_FILE_MAX_SIZE: i64 = 8 * 1024 * 1024;
const TELEGRAM_KIND_CHANNEL: &str = "channel";
const TELEGRAM_KIND_SUPERGROUP: &str = "supergroup";
const TELEGRAM_KIND_GROUP: &str = "group";
const STATUS_QUEUED: &str = "queued";
const STATUS_RUNNING: &str = "running";
const STATUS_CANCEL_REQUESTED: &str = "cancel_requested";
const STATUS_FAILED: &str = "failed";
const STATUS_CANCELLED: &str = "cancelled";
const STATUS_COMPLETED: &str = "completed";
const PHASE_QUEUED: &str = "queued";
const PHASE_RESOLVING_SOURCE: &str = "resolving_source";
const PHASE_STARTING_TAKEOUT: &str = "starting_takeout";
const PHASE_VALIDATING_PEER: &str = "validating_peer";
const PHASE_LOADING_SPLITS: &str = "loading_splits";
const PHASE_COUNTING: &str = "counting";
const PHASE_IMPORTING_HISTORY: &str = "importing_history";
const PHASE_FINISHING_TAKEOUT: &str = "finishing_takeout";
const PHASE_COMPLETED: &str = "completed";
const PHASE_FAILED: &str = "failed";
const PHASE_CANCELLED: &str = "cancelled";
const TAKEOUT_HISTORY_PAGE_LIMIT: i32 = 100;

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct StartTakeoutImportResponse {
    pub job_id: String,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct CancelTakeoutImportResponse {
    pub cancelled: bool,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct TakeoutImportJobRecord {
    pub job_id: String,
    pub source_id: i64,
    pub account_id: i64,
    pub status: String,
    pub phase: String,
    pub message: Option<String>,
    pub inserted: i64,
    pub skipped: i64,
    pub progress_current: Option<i64>,
    pub progress_total: Option<i64>,
    pub started_at: i64,
    pub finished_at: Option<i64>,
    pub warnings: Vec<String>,
    pub error: Option<String>,
}

#[derive(Default)]
struct TakeoutImportStateInner {
    next_job_id: u64,
    jobs: HashMap<String, TakeoutImportJobRecord>,
    active_by_source: HashMap<i64, String>,
    cancel_requested: HashSet<String>,
}

pub struct TakeoutImportState {
    inner: Mutex<TakeoutImportStateInner>,
}

impl TakeoutImportState {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(TakeoutImportStateInner::default()),
        }
    }

    async fn create_job(
        &self,
        source_id: i64,
        account_id: i64,
    ) -> AppResult<TakeoutImportJobRecord> {
        let mut inner = self.inner.lock().await;
        if let Some(job_id) = inner.active_by_source.get(&source_id) {
            return Err(AppError::conflict(format!(
                "Source {source_id} already has active Takeout import job {job_id}"
            )));
        }

        inner.next_job_id += 1;
        let job_id = format!("takeout-{}", inner.next_job_id);
        let record = TakeoutImportJobRecord {
            job_id: job_id.clone(),
            source_id,
            account_id,
            status: STATUS_QUEUED.to_string(),
            phase: PHASE_QUEUED.to_string(),
            message: Some("Takeout import queued.".to_string()),
            inserted: 0,
            skipped: 0,
            progress_current: None,
            progress_total: None,
            started_at: now_secs(),
            finished_at: None,
            warnings: Vec::new(),
            error: None,
        };

        inner.active_by_source.insert(source_id, job_id.clone());
        inner.jobs.insert(job_id, record.clone());
        Ok(record)
    }

    async fn list_jobs(&self) -> Vec<TakeoutImportJobRecord> {
        let mut jobs = self
            .inner
            .lock()
            .await
            .jobs
            .values()
            .cloned()
            .collect::<Vec<_>>();
        jobs.sort_by_key(|job| (job.started_at, job.job_id.clone()));
        jobs
    }

    async fn request_cancel(&self, job_id: &str) -> Option<TakeoutImportJobRecord> {
        let mut inner = self.inner.lock().await;
        if is_terminal_status(&inner.jobs.get(job_id)?.status) {
            return None;
        }

        inner.cancel_requested.insert(job_id.to_string());
        let job = inner.jobs.get_mut(job_id)?;
        job.status = STATUS_CANCEL_REQUESTED.to_string();
        job.message = Some("Cancel requested.".to_string());
        Some(job.clone())
    }

    async fn is_cancel_requested(&self, job_id: &str) -> bool {
        self.inner.lock().await.cancel_requested.contains(job_id)
    }

    async fn update_job<F>(&self, job_id: &str, update: F) -> Option<TakeoutImportJobRecord>
    where
        F: FnOnce(&mut TakeoutImportJobRecord),
    {
        let mut inner = self.inner.lock().await;
        let job = inner.jobs.get_mut(job_id)?;
        update(job);
        Some(job.clone())
    }

    async fn finish_job<F>(&self, job_id: &str, update: F) -> Option<TakeoutImportJobRecord>
    where
        F: FnOnce(&mut TakeoutImportJobRecord),
    {
        let mut inner = self.inner.lock().await;
        let source_id = {
            let job = inner.jobs.get_mut(job_id)?;
            update(job);
            job.finished_at = Some(now_secs());
            job.source_id
        };
        inner.active_by_source.remove(&source_id);
        inner.cancel_requested.remove(job_id);
        inner.jobs.get(job_id).cloned()
    }
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub(crate) struct TakeoutExportDcSpikeResult {
    pub(crate) source_id: i64,
    pub(crate) account_id: i64,
    pub(crate) telegram_source_kind: String,
    pub(crate) home_dc_id: i32,
    pub(crate) export_dc_id: i32,
    pub(crate) used_export_dc: bool,
    pub(crate) fallback_used: bool,
    pub(crate) takeout_id: i64,
    pub(crate) split_count: usize,
    pub(crate) warnings: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ExportDcAlias {
    home_dc_id: i32,
    export_dc_id: i32,
}

#[tauri::command]
pub async fn start_takeout_source_import(
    handle: AppHandle,
    state: tauri::State<'_, TakeoutImportState>,
    source_id: i64,
) -> AppResult<StartTakeoutImportResponse> {
    let pool = get_pool(&handle).await?;
    let source = load_source(&pool, source_id).await?;
    let account_id = source.account_id.ok_or_else(|| {
        AppError::validation(format!("Source {source_id} is not linked to an account"))
    })?;
    let record = state.create_job(source_id, account_id).await?;
    emit_takeout_import_event(&handle, &record);

    let job_id = record.job_id.clone();
    let task_handle = handle.clone();
    tauri::async_runtime::spawn(async move {
        run_takeout_import_job(task_handle, job_id).await;
    });

    Ok(StartTakeoutImportResponse {
        job_id: record.job_id,
    })
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
pub async fn run_takeout_export_dc_spike(
    handle: AppHandle,
    state: tauri::State<'_, TelegramState>,
    source_id: i64,
) -> AppResult<TakeoutExportDcSpikeResult> {
    let pool = get_pool(&handle).await?;
    let source = load_source(&pool, source_id).await?;
    let account_id = source.account_id.ok_or_else(|| {
        AppError::validation(format!("Source {source_id} is not linked to an account"))
    })?;
    let runtime = get_authorized_runtime(&state, account_id).await?;

    run_export_dc_spike_for_runtime(
        source.id,
        account_id,
        &source.telegram_source_kind,
        runtime.client,
        runtime.session,
    )
    .await
}

async fn run_export_dc_spike_for_runtime(
    source_id: i64,
    account_id: i64,
    telegram_source_kind: &str,
    client: Client,
    session: Arc<MemorySession>,
) -> AppResult<TakeoutExportDcSpikeResult> {
    client
        .invoke(&tl::functions::users::GetUsers {
            id: vec![tl::enums::InputUser::UserSelf],
        })
        .await
        .map_err(|e| AppError::network(format!("Telegram self check failed: {e}")))?;

    let alias = prepare_export_dc_alias(&session).await?;
    let init_request = takeout_init_request_for_source_kind(telegram_source_kind)?;
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
        telegram_source_kind: telegram_source_kind.to_string(),
        home_dc_id: alias.home_dc_id,
        export_dc_id: alias.export_dc_id,
        used_export_dc: !fallback_used,
        fallback_used,
        takeout_id,
        split_count: split_ranges.len(),
        warnings,
    })
}

async fn run_takeout_import_job(handle: AppHandle, job_id: String) {
    let takeout_state = handle.state::<TakeoutImportState>();
    let ingest_locks = handle.state::<SourceIngestLocks>();

    let Some(running_record) = takeout_state
        .update_job(&job_id, |job| {
            job.status = STATUS_RUNNING.to_string();
            job.phase = PHASE_RESOLVING_SOURCE.to_string();
            job.message = Some("Preparing Takeout import.".to_string());
        })
        .await
    else {
        return;
    };
    emit_takeout_import_event(&handle, &running_record);

    let ingest_guard = match ingest_locks
        .try_acquire(running_record.source_id, SourceIngestKind::TakeoutImport)
        .await
    {
        Ok(guard) => guard,
        Err(error) => {
            if let Some(record) = takeout_state
                .finish_job(&job_id, |job| {
                    job.status = STATUS_FAILED.to_string();
                    job.phase = PHASE_FAILED.to_string();
                    job.message = None;
                    job.error = Some(error.to_string());
                })
                .await
            {
                emit_takeout_import_event(&handle, &record);
            }
            return;
        }
    };

    if takeout_state.is_cancel_requested(&job_id).await {
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

    match run_takeout_source_import(&handle, &job_id).await {
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
            } else if let Some(record) = takeout_state
                .finish_job(&job_id, |job| {
                    job.status = STATUS_FAILED.to_string();
                    job.phase = PHASE_FAILED.to_string();
                    job.message = None;
                    job.error = Some(error.to_string());
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

async fn run_takeout_source_import(
    handle: &AppHandle,
    job_id: &str,
) -> AppResult<TakeoutImportOutcome> {
    let takeout_state = handle.state::<TakeoutImportState>();
    let telegram_state = handle.state::<TelegramState>();
    let pool = get_pool(handle).await?;
    let source_id = takeout_state
        .update_job(job_id, |_| {})
        .await
        .ok_or_else(|| AppError::internal(format!("Takeout job {job_id} not found")))?
        .source_id;
    let source = load_source(&pool, source_id).await?;
    ensure_supported_takeout_source_kind(&source.telegram_source_kind)?;

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
    let resolved_peer = resolve_and_refresh_peer(handle, &client, &source, account_id).await?;

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
    let init_request = takeout_init_request_for_source_kind(&source.telegram_source_kind)?;
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

    let started_result = run_started_takeout_source_import(
        handle,
        job_id,
        &pool,
        &source,
        resolved_peer,
        &client,
        &alias,
        takeout_id,
        warnings,
        fallback_used,
    )
    .await;

    match started_result {
        Ok(outcome) => Ok(outcome),
        Err((error, mut warnings, mut fallback_used)) => {
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
            }
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
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source: &crate::sources::SourceSyncTarget,
    resolved_peer: crate::sources::ResolvedSyncPeer,
    client: &Client,
    alias: &ExportDcAlias,
    takeout_id: i64,
    mut warnings: Vec<String>,
    mut fallback_used: bool,
) -> Result<TakeoutImportOutcome, (AppError, Vec<String>, bool)> {
    match run_started_takeout_source_import_inner(
        handle,
        job_id,
        pool,
        source,
        resolved_peer,
        client,
        alias,
        takeout_id,
        &mut warnings,
        &mut fallback_used,
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
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source: &crate::sources::SourceSyncTarget,
    resolved_peer: crate::sources::ResolvedSyncPeer,
    client: &Client,
    alias: &ExportDcAlias,
    takeout_id: i64,
    warnings: &mut Vec<String>,
    fallback_used: &mut bool,
) -> AppResult<TakeoutImportOutcome> {
    let takeout_state = handle.state::<TakeoutImportState>();
    let input_peer: tl::enums::InputPeer = resolved_peer.peer.into();

    update_and_emit(handle, &takeout_state, job_id, |job| {
        job.phase = PHASE_VALIDATING_PEER.to_string();
        job.message = Some("Validating Telegram source.".to_string());
        job.warnings.extend(warnings.clone());
    })
    .await;
    validate_takeout_peer(
        &client,
        &alias,
        takeout_id,
        &source.telegram_source_kind,
        resolved_peer.peer,
        warnings,
        fallback_used,
    )
    .await?;
    detect_supergroup_migration(
        client,
        alias,
        takeout_id,
        &source.telegram_source_kind,
        resolved_peer.peer,
        warnings,
        fallback_used,
    )
    .await?;

    update_and_emit(handle, &takeout_state, job_id, |job| {
        job.phase = PHASE_LOADING_SPLITS.to_string();
        job.message = Some("Loading Takeout message ranges.".to_string());
        job.warnings = warnings.to_vec();
    })
    .await;
    let split_ranges = export_dc_invoke(
        &client,
        &alias,
        &tl::functions::InvokeWithTakeout {
            takeout_id,
            query: tl::functions::messages::GetSplitRanges {},
        },
        warnings,
        fallback_used,
    )
    .await?;
    let selected_ranges = select_history_splits(&source.telegram_source_kind, split_ranges)?;

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
            &client,
            &alias,
            takeout_id,
            input_peer.clone(),
            range.clone(),
            &source.telegram_source_kind,
            warnings,
            fallback_used,
        )
        .await?;
        total += probe.count;
        counted_ranges.push(CountedMessageRange {
            range,
            only_my_messages: probe.only_my_messages,
        });
    }

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
        &client,
        &alias,
        takeout_id,
        input_peer,
        counted_ranges,
        &source,
        total,
        &source.telegram_source_kind,
        warnings,
        fallback_used,
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
    finish_takeout_session(client, alias, takeout_id, true, warnings, fallback_used).await?;
    finalize_sync(
        &pool,
        &source,
        source.last_sync_state.unwrap_or(0),
        import.max_message_id,
        resolved_peer.refreshed_metadata_zstd,
    )
    .await?;

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
    only_my_messages: bool,
}

struct TakeoutHistoryProbe {
    count: i64,
    only_my_messages: bool,
}

async fn update_and_emit<F>(handle: &AppHandle, state: &TakeoutImportState, job_id: &str, update: F)
where
    F: FnOnce(&mut TakeoutImportJobRecord),
{
    if let Some(record) = state.update_job(job_id, update).await {
        emit_takeout_import_event(handle, &record);
    }
}

fn ensure_supported_takeout_source_kind(telegram_source_kind: &str) -> AppResult<()> {
    match telegram_source_kind {
        TELEGRAM_KIND_CHANNEL | TELEGRAM_KIND_SUPERGROUP | TELEGRAM_KIND_GROUP => Ok(()),
        other => Err(AppError::validation(format!(
            "Unsupported telegram_source_kind '{other}'"
        ))),
    }
}

async fn validate_takeout_peer(
    client: &Client,
    alias: &ExportDcAlias,
    takeout_id: i64,
    telegram_source_kind: &str,
    peer: grammers_session::types::PeerRef,
    warnings: &mut Vec<String>,
    fallback_used: &mut bool,
) -> AppResult<()> {
    match telegram_source_kind {
        TELEGRAM_KIND_CHANNEL | TELEGRAM_KIND_SUPERGROUP => {
            let input_channel: tl::enums::InputChannel = peer.into();
            export_dc_invoke(
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
            )
            .await?;
        }
        TELEGRAM_KIND_GROUP => {
            export_dc_invoke(
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
            )
            .await?;
        }
        other => {
            return Err(AppError::validation(format!(
                "Unsupported telegram_source_kind '{other}'"
            )));
        }
    }

    Ok(())
}

async fn detect_supergroup_migration(
    client: &Client,
    alias: &ExportDcAlias,
    takeout_id: i64,
    telegram_source_kind: &str,
    peer: grammers_session::types::PeerRef,
    warnings: &mut Vec<String>,
    fallback_used: &mut bool,
) -> AppResult<()> {
    if telegram_source_kind != TELEGRAM_KIND_SUPERGROUP {
        return Ok(());
    }

    let input_channel: tl::enums::InputChannel = peer.into();
    let chat_full = export_dc_invoke(
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
    )
    .await?;

    let tl::enums::messages::ChatFull::Full(chat_full) = chat_full;
    if let tl::enums::ChatFull::ChannelFull(full) = chat_full.full_chat {
        if let Some(migrated_from_chat_id) = full.migrated_from_chat_id {
            warnings.push(format!(
                "Supergroup migrated_from_chat_id {migrated_from_chat_id} detected; migrated history import is deferred to avoid source item id collisions."
            ));
        }
    }

    Ok(())
}

fn select_history_splits(
    telegram_source_kind: &str,
    split_ranges: Vec<tl::enums::MessageRange>,
) -> AppResult<Vec<tl::enums::MessageRange>> {
    let mut ranges = if split_ranges.is_empty() {
        vec![fallback_message_range()]
    } else {
        split_ranges
    };

    match telegram_source_kind {
        TELEGRAM_KIND_CHANNEL | TELEGRAM_KIND_SUPERGROUP => {
            Ok(vec![ranges.pop().unwrap_or_else(fallback_message_range)])
        }
        TELEGRAM_KIND_GROUP => Ok(ranges),
        other => Err(AppError::validation(format!(
            "Unsupported telegram_source_kind '{other}'"
        ))),
    }
}

fn fallback_message_range() -> tl::enums::MessageRange {
    tl::types::MessageRange {
        min_id: 1,
        max_id: i32::MAX,
    }
    .into()
}

async fn takeout_history_count_probe(
    client: &Client,
    alias: &ExportDcAlias,
    takeout_id: i64,
    input_peer: tl::enums::InputPeer,
    range: tl::enums::MessageRange,
    telegram_source_kind: &str,
    warnings: &mut Vec<String>,
    fallback_used: &mut bool,
) -> AppResult<TakeoutHistoryProbe> {
    let response = takeout_get_history(
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
    )
    .await;

    let response = match response {
        Ok(response) => response,
        Err(error)
            if supports_only_my_messages_fallback(telegram_source_kind)
                && is_channel_private_error(&error) =>
        {
            warnings.push(
                "Channel history is private; falling back to messages.search(from_id=self)."
                    .to_string(),
            );
            let search_response = takeout_search_my_messages(
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
    client: &Client,
    alias: &ExportDcAlias,
    takeout_id: i64,
    input_peer: tl::enums::InputPeer,
    ranges: Vec<CountedMessageRange>,
    source: &crate::sources::SourceSyncTarget,
    total: i64,
    telegram_source_kind: &str,
    warnings: &mut Vec<String>,
    fallback_used: &mut bool,
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
            client,
            alias,
            takeout_id,
            input_peer.clone(),
            counted_range.range,
            counted_range.only_my_messages,
            source,
            total,
            telegram_source_kind,
            imported,
            warnings,
            fallback_used,
        )
        .await?;
    }

    Ok(imported)
}

async fn import_takeout_history_pages(
    handle: &AppHandle,
    job_id: &str,
    client: &Client,
    alias: &ExportDcAlias,
    takeout_id: i64,
    input_peer: tl::enums::InputPeer,
    range: tl::enums::MessageRange,
    only_my_messages: bool,
    source: &crate::sources::SourceSyncTarget,
    total: i64,
    telegram_source_kind: &str,
    mut imported: TakeoutHistoryImport,
    warnings: &mut Vec<String>,
    fallback_used: &mut bool,
) -> AppResult<TakeoutHistoryImport> {
    let takeout_state = handle.state::<TakeoutImportState>();
    let pool = get_pool(handle).await?;
    let mut offset_id = message_range_max_id(&range);

    loop {
        if takeout_state.is_cancel_requested(job_id).await {
            return Err(AppError::validation("Takeout import cancelled"));
        }

        let response = if only_my_messages {
            takeout_search_my_messages(
                client,
                alias,
                takeout_id,
                input_peer.clone(),
                range.clone(),
                offset_id,
                -TAKEOUT_HISTORY_PAGE_LIMIT,
                TAKEOUT_HISTORY_PAGE_LIMIT,
                warnings,
                fallback_used,
            )
            .await?
        } else {
            match takeout_get_history(
                client,
                alias,
                takeout_id,
                input_peer.clone(),
                range.clone(),
                offset_id,
                -TAKEOUT_HISTORY_PAGE_LIMIT,
                TAKEOUT_HISTORY_PAGE_LIMIT,
                warnings,
                fallback_used,
            )
            .await
            {
                Ok(response) => response,
                Err(error)
                    if supports_only_my_messages_fallback(telegram_source_kind)
                        && is_channel_private_error(&error) =>
                {
                    warnings.push(
                        "Channel history is private; falling back to messages.search(from_id=self)."
                            .to_string(),
                    );
                    takeout_search_my_messages(
                        client,
                        alias,
                        takeout_id,
                        input_peer.clone(),
                        range.clone(),
                        offset_id,
                        -TAKEOUT_HISTORY_PAGE_LIMIT,
                        TAKEOUT_HISTORY_PAGE_LIMIT,
                        warnings,
                        fallback_used,
                    )
                    .await?
                }
                Err(error) => return Err(error),
            }
        };
        let messages = raw_messages_from_response(response)?;
        if messages.is_empty() {
            break;
        }

        let mut next_offset_id = offset_id;
        for message in messages {
            let message_id = message.id;
            if message_id <= message_range_min_id(&range) {
                continue;
            }
            imported.max_message_id = imported.max_message_id.max(i64::from(message_id));
            match raw_parse::parse_raw_message(&source.title, message) {
                Ok(Some(item)) => {
                    if insert_source_item(&pool, source.id, item).await? {
                        imported.inserted += 1;
                    } else {
                        imported.skipped += 1;
                    }
                }
                Ok(None) => imported.skipped += 1,
                Err(error) => return Err(AppError::internal(error)),
            }
            next_offset_id = next_offset_id.min(message_id);
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

        if next_offset_id == offset_id || next_offset_id <= message_range_min_id(&range) {
            break;
        }
        offset_id = next_offset_id;
    }

    Ok(imported)
}

async fn takeout_get_history(
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
) -> AppResult<tl::enums::messages::Messages> {
    export_dc_invoke(
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
    )
    .await
}

async fn takeout_search_my_messages(
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
) -> AppResult<tl::enums::messages::Messages> {
    export_dc_invoke(
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
    )
    .await
}

async fn finish_takeout_session(
    client: &Client,
    alias: &ExportDcAlias,
    takeout_id: i64,
    success: bool,
    warnings: &mut Vec<String>,
    fallback_used: &mut bool,
) -> AppResult<()> {
    export_dc_invoke(
        client,
        alias,
        &tl::functions::InvokeWithTakeout {
            takeout_id,
            query: tl::functions::account::FinishTakeoutSession { success },
        },
        warnings,
        fallback_used,
    )
    .await
    .map(|_| ())
}

fn supports_only_my_messages_fallback(telegram_source_kind: &str) -> bool {
    matches!(
        telegram_source_kind,
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

fn raw_messages_from_response(
    response: tl::enums::messages::Messages,
) -> AppResult<Vec<tl::types::Message>> {
    let messages = match response {
        tl::enums::messages::Messages::Messages(messages) => messages.messages,
        tl::enums::messages::Messages::Slice(messages) => messages.messages,
        tl::enums::messages::Messages::ChannelMessages(messages) => messages.messages,
        tl::enums::messages::Messages::NotModified(_) => {
            return Err(AppError::network(
                "Telegram returned messagesNotModified for Takeout history page",
            ));
        }
    };

    Ok(messages
        .into_iter()
        .filter_map(|message| match message {
            tl::enums::Message::Message(message) => Some(message),
            _ => None,
        })
        .collect())
}

fn message_range_min_id(range: &tl::enums::MessageRange) -> i32 {
    match range {
        tl::enums::MessageRange::Range(range) => range.min_id,
    }
}

fn message_range_max_id(range: &tl::enums::MessageRange) -> i32 {
    match range {
        tl::enums::MessageRange::Range(range) => range.max_id,
    }
}

fn emit_takeout_import_event(handle: &AppHandle, record: &TakeoutImportJobRecord) {
    let _ = handle.emit(TAKEOUT_IMPORT_EVENT, record);
}

async fn prepare_export_dc_alias(session: &Arc<MemorySession>) -> AppResult<ExportDcAlias> {
    let home_dc_id = session.home_dc_id();
    let export_dc_id = export_dc_id_for_home_dc(home_dc_id);
    let mut export_option = session.dc_option(home_dc_id).ok_or_else(|| {
        AppError::internal(format!(
            "Home DC option {home_dc_id} is missing from session"
        ))
    })?;
    export_option.id = export_dc_id;
    session.set_dc_option(&export_option).await;

    Ok(ExportDcAlias {
        home_dc_id,
        export_dc_id,
    })
}

fn export_dc_id_for_home_dc(home_dc_id: i32) -> i32 {
    home_dc_id + EXPORT_DC_SHIFT
}

fn takeout_init_request_for_source_kind(
    telegram_source_kind: &str,
) -> AppResult<tl::functions::account::InitTakeoutSession> {
    let (message_chats, message_megagroups, message_channels) = match telegram_source_kind {
        TELEGRAM_KIND_GROUP => (true, false, false),
        TELEGRAM_KIND_SUPERGROUP => (false, true, false),
        TELEGRAM_KIND_CHANNEL => (false, false, true),
        other => {
            return Err(AppError::validation(format!(
                "Unsupported telegram_source_kind '{other}'"
            )));
        }
    };

    Ok(tl::functions::account::InitTakeoutSession {
        contacts: false,
        message_users: false,
        message_chats,
        message_megagroups,
        message_channels,
        files: true,
        file_max_size: Some(TAKEOUT_FILE_MAX_SIZE),
    })
}

async fn export_dc_invoke<R: tl::RemoteCall>(
    client: &Client,
    alias: &ExportDcAlias,
    request: &R,
    warnings: &mut Vec<String>,
    fallback_used: &mut bool,
) -> AppResult<R::Return> {
    if !*fallback_used {
        match client.invoke_in_dc(alias.export_dc_id, request).await {
            Ok(response) => return Ok(response),
            Err(error) if should_fallback_export_dc_error(&error) => {
                *fallback_used = true;
                warnings.push(format!(
                    "Export DC {} failed with local transport error; falling back to home DC {}: {error}",
                    alias.export_dc_id, alias.home_dc_id
                ));
            }
            Err(error) => return Err(AppError::network(error.to_string())),
        }
    }

    client
        .invoke(request)
        .await
        .map_err(|error| AppError::network(error.to_string()))
}

fn should_fallback_export_dc_error(error: &InvocationError) -> bool {
    matches!(
        error,
        InvocationError::InvalidDc
            | InvocationError::Io(_)
            | InvocationError::Transport(_)
            | InvocationError::Authentication(_)
            | InvocationError::Dropped
    )
}

fn is_terminal_status(status: &str) -> bool {
    matches!(status, STATUS_FAILED | STATUS_CANCELLED | "completed")
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::{
        export_dc_id_for_home_dc, is_channel_private_error, message_range_max_id,
        message_range_min_id, select_history_splits, should_fallback_export_dc_error,
        supports_only_my_messages_fallback, takeout_init_request_for_source_kind,
        TakeoutImportState, STATUS_CANCEL_REQUESTED, STATUS_FAILED, TAKEOUT_FILE_MAX_SIZE,
        TELEGRAM_KIND_CHANNEL, TELEGRAM_KIND_GROUP, TELEGRAM_KIND_SUPERGROUP,
    };
    use crate::error::{AppError, AppErrorKind};
    use grammers_client::tl;
    use grammers_mtsender::{InvocationError, RpcError};

    #[test]
    fn export_dc_id_applies_tdesktop_shift() {
        assert_eq!(export_dc_id_for_home_dc(2), 40_002);
    }

    #[test]
    fn takeout_init_request_uses_source_kind_flags_and_file_limit() {
        let group = takeout_init_request_for_source_kind(TELEGRAM_KIND_GROUP).expect("group flags");
        assert!(group.message_chats);
        assert!(!group.message_megagroups);
        assert!(!group.message_channels);
        assert!(group.files);
        assert_eq!(group.file_max_size, Some(TAKEOUT_FILE_MAX_SIZE));

        let supergroup = takeout_init_request_for_source_kind(TELEGRAM_KIND_SUPERGROUP)
            .expect("supergroup flags");
        assert!(!supergroup.message_chats);
        assert!(supergroup.message_megagroups);
        assert!(!supergroup.message_channels);

        let channel =
            takeout_init_request_for_source_kind(TELEGRAM_KIND_CHANNEL).expect("channel flags");
        assert!(!channel.message_chats);
        assert!(!channel.message_megagroups);
        assert!(channel.message_channels);
    }

    #[test]
    fn export_dc_fallback_is_only_for_local_transport_errors() {
        assert!(should_fallback_export_dc_error(&InvocationError::InvalidDc));
        assert!(should_fallback_export_dc_error(&InvocationError::Dropped));
        assert!(!should_fallback_export_dc_error(&InvocationError::Rpc(
            RpcError {
                code: 400,
                name: "TAKEOUT_INVALID".to_string(),
                value: None,
                caused_by: None,
            }
        )));
    }

    #[test]
    fn split_selection_uses_last_range_for_channel_and_supergroup() {
        let ranges = vec![message_range(1, 10), message_range(11, 20)];

        let channel =
            select_history_splits(TELEGRAM_KIND_CHANNEL, ranges.clone()).expect("channel splits");
        let supergroup =
            select_history_splits(TELEGRAM_KIND_SUPERGROUP, ranges).expect("supergroup splits");

        assert_eq!(channel.len(), 1);
        assert_eq!(message_range_min_id(&channel[0]), 11);
        assert_eq!(message_range_max_id(&channel[0]), 20);
        assert_eq!(supergroup.len(), 1);
        assert_eq!(message_range_min_id(&supergroup[0]), 11);
        assert_eq!(message_range_max_id(&supergroup[0]), 20);
    }

    #[test]
    fn split_selection_uses_all_ranges_for_small_group() {
        let ranges = vec![message_range(1, 10), message_range(11, 20)];

        let selected = select_history_splits(TELEGRAM_KIND_GROUP, ranges).expect("group splits");

        assert_eq!(selected.len(), 2);
        assert_eq!(message_range_min_id(&selected[0]), 1);
        assert_eq!(message_range_max_id(&selected[0]), 10);
        assert_eq!(message_range_min_id(&selected[1]), 11);
        assert_eq!(message_range_max_id(&selected[1]), 20);
    }

    #[test]
    fn split_selection_falls_back_when_telegram_returns_no_ranges() {
        let selected =
            select_history_splits(TELEGRAM_KIND_GROUP, Vec::new()).expect("fallback split");

        assert_eq!(selected.len(), 1);
        assert_eq!(message_range_min_id(&selected[0]), 1);
        assert_eq!(message_range_max_id(&selected[0]), i32::MAX);
    }

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
    async fn job_state_rejects_duplicate_active_source_jobs() {
        let state = TakeoutImportState::new();
        let first = state.create_job(7, 1).await.expect("create first job");

        let error = state
            .create_job(7, 1)
            .await
            .expect_err("duplicate source job should fail");

        assert_eq!(first.job_id, "takeout-1");
        assert_eq!(error.kind, AppErrorKind::Conflict);
    }

    #[tokio::test]
    async fn job_state_can_cancel_and_finish_job() {
        let state = TakeoutImportState::new();
        let job = state.create_job(7, 1).await.expect("create job");

        let cancelled = state
            .request_cancel(&job.job_id)
            .await
            .expect("cancel active job");
        assert_eq!(cancelled.status, STATUS_CANCEL_REQUESTED);
        assert!(state.is_cancel_requested(&job.job_id).await);

        let finished = state
            .finish_job(&job.job_id, |job| {
                job.status = STATUS_FAILED.to_string();
                job.phase = STATUS_FAILED.to_string();
                job.error = Some("not implemented".to_string());
            })
            .await
            .expect("finish job");
        assert!(finished.finished_at.is_some());
        assert!(!state.is_cancel_requested(&job.job_id).await);

        let next = state.create_job(7, 1).await.expect("source released");
        assert_eq!(next.job_id, "takeout-2");
    }

    fn message_range(min_id: i32, max_id: i32) -> tl::enums::MessageRange {
        tl::types::MessageRange { min_id, max_id }.into()
    }
}
