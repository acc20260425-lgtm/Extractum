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
use crate::sources::load_source;
use crate::telegram::{get_authorized_runtime, TelegramState};

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
const PHASE_QUEUED: &str = "queued";
const PHASE_RESOLVING_SOURCE: &str = "resolving_source";
const PHASE_FAILED: &str = "failed";
const PHASE_CANCELLED: &str = "cancelled";

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
        run_noop_takeout_import_job(task_handle, job_id).await;
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

async fn run_noop_takeout_import_job(handle: AppHandle, job_id: String) {
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

    if let Some(record) = takeout_state
        .finish_job(&job_id, |job| {
            job.status = STATUS_FAILED.to_string();
            job.phase = PHASE_FAILED.to_string();
            job.message = None;
            job.error = Some("Takeout import is not implemented yet.".to_string());
        })
        .await
    {
        emit_takeout_import_event(&handle, &record);
    }
    drop(ingest_guard);
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
        export_dc_id_for_home_dc, should_fallback_export_dc_error,
        takeout_init_request_for_source_kind, TakeoutImportState, STATUS_CANCEL_REQUESTED,
        STATUS_FAILED, TAKEOUT_FILE_MAX_SIZE, TELEGRAM_KIND_CHANNEL, TELEGRAM_KIND_GROUP,
        TELEGRAM_KIND_SUPERGROUP,
    };
    use crate::error::AppErrorKind;
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
}
