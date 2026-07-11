use std::collections::{HashMap, HashSet};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use crate::error::{AppError, AppResult};
use crate::job_helpers::{ActiveJobGuards, CancellationState};
use crate::time::now_secs;

const TAKEOUT_IMPORT_EVENT: &str = "sources://takeout-import";
#[cfg(dev)]
const TAKEOUT_CANCELLATION_SMOKE_FIXTURE_SOURCE_ID: i64 = -910_001;
#[cfg(dev)]
const TAKEOUT_CANCELLATION_SMOKE_FIXTURE_ACCOUNT_ID: i64 = -910_002;
#[cfg(dev)]
const TAKEOUT_CANCELLATION_SMOKE_FIXTURE_BATCH_ID: i64 = -910_003;
const STATUS_QUEUED: &str = "queued";
pub(crate) const STATUS_RUNNING: &str = "running";
pub(crate) const STATUS_CANCEL_REQUESTED: &str = "cancel_requested";
pub(crate) const STATUS_FAILED: &str = "failed";
pub(crate) const STATUS_CANCELLED: &str = "cancelled";
pub(crate) const STATUS_COMPLETED: &str = "completed";
const PHASE_QUEUED: &str = "queued";
pub(crate) const PHASE_RESOLVING_SOURCE: &str = "resolving_source";
pub(crate) const PHASE_STARTING_TAKEOUT: &str = "starting_takeout";
pub(crate) const PHASE_VALIDATING_PEER: &str = "validating_peer";
pub(crate) const PHASE_LOADING_SPLITS: &str = "loading_splits";
pub(crate) const PHASE_COUNTING: &str = "counting";
pub(crate) const PHASE_IMPORTING_HISTORY: &str = "importing_history";
pub(crate) const PHASE_FINISHING_TAKEOUT: &str = "finishing_takeout";
pub(crate) const PHASE_COMPLETED: &str = "completed";
pub(crate) const PHASE_FAILED: &str = "failed";
pub(crate) const PHASE_CANCELLED: &str = "cancelled";

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
    pub batch_id: i64,
    pub source_id: i64,
    pub account_id: i64,
    pub history_scope: String,
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
    active_jobs: ActiveJobGuards<i64>,
    cancel_requested: CancellationState,
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

    pub(crate) async fn create_job(
        &self,
        source_id: i64,
        account_id: i64,
        batch_id: i64,
        history_scope: &str,
    ) -> AppResult<TakeoutImportJobRecord> {
        let mut inner = self.inner.lock().await;
        if let Some(job_id) = inner.active_jobs.active_job_id(&source_id) {
            return Err(AppError::conflict(format!(
                "Source {source_id} already has active Takeout import job {job_id}"
            )));
        }

        inner.next_job_id += 1;
        let job_id = format!("takeout-{}", inner.next_job_id);
        let record = TakeoutImportJobRecord {
            job_id: job_id.clone(),
            batch_id,
            source_id,
            account_id,
            history_scope: history_scope.to_string(),
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

        inner.active_jobs.track(source_id, job_id.clone());
        inner.jobs.insert(job_id, record.clone());
        Ok(record)
    }

    pub(crate) async fn list_jobs(&self) -> Vec<TakeoutImportJobRecord> {
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

    pub(crate) async fn request_cancel(&self, job_id: &str) -> Option<TakeoutImportJobRecord> {
        let mut inner = self.inner.lock().await;
        if is_terminal_status(&inner.jobs.get(job_id)?.status) {
            return None;
        }

        inner.cancel_requested.request(job_id);
        let job = inner.jobs.get_mut(job_id)?;
        job.status = STATUS_CANCEL_REQUESTED.to_string();
        job.message = Some("Cancel requested.".to_string());
        Some(job.clone())
    }

    pub(crate) async fn is_cancel_requested(&self, job_id: &str) -> bool {
        self.inner
            .lock()
            .await
            .cancel_requested
            .is_requested(job_id)
    }

    pub(crate) async fn cancellation_token(&self, job_id: &str) -> Option<CancellationToken> {
        let mut inner = self.inner.lock().await;
        let job = inner.jobs.get(job_id)?;
        if is_terminal_status(&job.status) {
            return None;
        }
        Some(inner.cancel_requested.child_token(job_id))
    }

    pub(crate) async fn update_job<F>(
        &self,
        job_id: &str,
        update: F,
    ) -> Option<TakeoutImportJobRecord>
    where
        F: FnOnce(&mut TakeoutImportJobRecord),
    {
        let mut inner = self.inner.lock().await;
        let job = inner.jobs.get_mut(job_id)?;
        update(job);
        Some(job.clone())
    }

    pub(crate) async fn finish_job<F>(
        &self,
        job_id: &str,
        update: F,
    ) -> Option<TakeoutImportJobRecord>
    where
        F: FnOnce(&mut TakeoutImportJobRecord),
    {
        let mut inner = self.inner.lock().await;
        {
            let job = inner.jobs.get_mut(job_id)?;
            update(job);
            job.finished_at = Some(now_secs());
        }
        inner.active_jobs.release_by_job_id(job_id);
        inner.cancel_requested.clear(job_id);
        inner.jobs.get(job_id).cloned()
    }

    pub(crate) async fn active_jobs_for_sources(
        &self,
        source_ids: &[i64],
    ) -> Vec<TakeoutImportJobRecord> {
        let source_ids = source_ids.iter().copied().collect::<HashSet<_>>();
        let mut jobs = self
            .inner
            .lock()
            .await
            .jobs
            .values()
            .filter(|job| source_ids.contains(&job.source_id))
            .filter(|job| !is_terminal_status(&job.status))
            .cloned()
            .collect::<Vec<_>>();
        jobs.sort_by_key(|job| (job.started_at, job.job_id.clone()));
        jobs
    }

    #[cfg(dev)]
    async fn remove_cancellation_smoke_fixture_jobs(&self) -> usize {
        let mut inner = self.inner.lock().await;
        let job_ids = inner
            .jobs
            .values()
            .filter(|job| job.source_id == TAKEOUT_CANCELLATION_SMOKE_FIXTURE_SOURCE_ID)
            .map(|job| job.job_id.clone())
            .collect::<Vec<_>>();
        for job_id in &job_ids {
            inner.active_jobs.release_by_job_id(job_id);
            inner.cancel_requested.clear(job_id);
            inner.jobs.remove(job_id);
        }
        job_ids.len()
    }
}

#[cfg(dev)]
pub(crate) async fn seed_takeout_cancellation_smoke_fixture(
    handle: AppHandle,
    state: &TakeoutImportState,
) -> AppResult<TakeoutImportJobRecord> {
    clear_takeout_cancellation_smoke_fixture(state).await?;
    let record = seed_takeout_cancellation_smoke_fixture_in_state(state).await?;
    emit_takeout_import_event(&handle, &record);
    let job_id = record.job_id.clone();
    let token = state.cancellation_token(&job_id).await;
    let task_handle = handle.clone();
    tauri::async_runtime::spawn(async move {
        let Some(token) = token else {
            return;
        };
        token.cancelled().await;
        let state = task_handle.state::<TakeoutImportState>();
        if let Some(record) =
            finish_cancelled_takeout_cancellation_smoke_fixture(state.inner(), &job_id).await
        {
            emit_takeout_import_event(&task_handle, &record);
        }
    });
    Ok(record)
}

#[cfg(dev)]
pub(crate) async fn clear_takeout_cancellation_smoke_fixture(
    state: &TakeoutImportState,
) -> AppResult<usize> {
    Ok(state.remove_cancellation_smoke_fixture_jobs().await)
}

#[cfg(dev)]
async fn seed_takeout_cancellation_smoke_fixture_in_state(
    state: &TakeoutImportState,
) -> AppResult<TakeoutImportJobRecord> {
    let record = state
        .create_job(
            TAKEOUT_CANCELLATION_SMOKE_FIXTURE_SOURCE_ID,
            TAKEOUT_CANCELLATION_SMOKE_FIXTURE_ACCOUNT_ID,
            TAKEOUT_CANCELLATION_SMOKE_FIXTURE_BATCH_ID,
            crate::ingest_provenance::TAKEOUT_HISTORY_SCOPE_CURRENT,
        )
        .await?;
    state
        .update_job(&record.job_id, |job| {
            job.status = STATUS_RUNNING.to_string();
            job.phase = PHASE_IMPORTING_HISTORY.to_string();
            job.message = Some("Takeout cancellation smoke fixture running.".to_string());
            job.progress_current = Some(0);
            job.progress_total = Some(1);
        })
        .await
        .ok_or_else(|| AppError::not_found(format!("Takeout job {} not found", record.job_id)))
}

#[cfg(dev)]
async fn finish_cancelled_takeout_cancellation_smoke_fixture(
    state: &TakeoutImportState,
    job_id: &str,
) -> Option<TakeoutImportJobRecord> {
    state
        .finish_job(job_id, |job| {
            job.status = STATUS_CANCELLED.to_string();
            job.phase = PHASE_CANCELLED.to_string();
            job.message = Some("Takeout import cancelled.".to_string());
            job.error = None;
        })
        .await
}

pub(crate) async fn update_and_emit<F>(
    handle: &AppHandle,
    state: &TakeoutImportState,
    job_id: &str,
    update: F,
) where
    F: FnOnce(&mut TakeoutImportJobRecord),
{
    if let Some(record) = state.update_job(job_id, update).await {
        emit_takeout_import_event(handle, &record);
    }
}

pub(crate) fn emit_takeout_import_event(handle: &AppHandle, record: &TakeoutImportJobRecord) {
    let _ = handle.emit(TAKEOUT_IMPORT_EVENT, record);
}

fn is_terminal_status(status: &str) -> bool {
    matches!(status, STATUS_FAILED | STATUS_CANCELLED | STATUS_COMPLETED)
}

#[cfg(test)]
mod tests {
    use super::{
        clear_takeout_cancellation_smoke_fixture,
        finish_cancelled_takeout_cancellation_smoke_fixture,
        seed_takeout_cancellation_smoke_fixture_in_state, TakeoutImportState, STATUS_CANCELLED,
        STATUS_CANCEL_REQUESTED, STATUS_COMPLETED, STATUS_FAILED, STATUS_RUNNING,
        TAKEOUT_CANCELLATION_SMOKE_FIXTURE_SOURCE_ID,
    };
    use crate::error::AppErrorKind;
    use crate::ingest_provenance::{
        TAKEOUT_HISTORY_SCOPE_CURRENT, TAKEOUT_HISTORY_SCOPE_MIGRATED_SMALL_GROUP,
    };

    #[tokio::test]
    async fn job_state_rejects_duplicate_active_source_jobs() {
        let state = TakeoutImportState::new();
        let first = state
            .create_job(7, 1, 100, TAKEOUT_HISTORY_SCOPE_CURRENT)
            .await
            .expect("create first job");

        let error = state
            .create_job(7, 1, 101, TAKEOUT_HISTORY_SCOPE_MIGRATED_SMALL_GROUP)
            .await
            .expect_err("duplicate source job should fail");

        assert_eq!(first.job_id, "takeout-1");
        assert_eq!(first.batch_id, 100);
        assert_eq!(error.kind, AppErrorKind::Conflict);
    }

    #[tokio::test]
    async fn job_state_can_cancel_and_finish_job() {
        let state = TakeoutImportState::new();
        let job = state
            .create_job(7, 1, 100, TAKEOUT_HISTORY_SCOPE_CURRENT)
            .await
            .expect("create job");

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

        let next = state
            .create_job(7, 1, 101, TAKEOUT_HISTORY_SCOPE_CURRENT)
            .await
            .expect("source released");
        assert_eq!(next.job_id, "takeout-2");
        assert_eq!(next.batch_id, 101);
    }

    #[tokio::test]
    async fn job_state_cancels_child_tokens() {
        let state = TakeoutImportState::new();
        let job = state
            .create_job(7, 1, 100, TAKEOUT_HISTORY_SCOPE_CURRENT)
            .await
            .expect("create job");
        let token = state
            .cancellation_token(&job.job_id)
            .await
            .expect("cancellation token");

        assert!(!token.is_cancelled());

        state
            .request_cancel(&job.job_id)
            .await
            .expect("cancel active job");
        tokio::time::timeout(std::time::Duration::from_secs(1), token.cancelled())
            .await
            .expect("token cancelled");

        state
            .finish_job(&job.job_id, |job| {
                job.status = STATUS_COMPLETED.to_string();
                job.phase = STATUS_COMPLETED.to_string();
            })
            .await
            .expect("finish job");
        assert!(state.cancellation_token(&job.job_id).await.is_none());
    }

    #[tokio::test]
    async fn takeout_cancellation_smoke_fixture_tracks_running_job() {
        let state = TakeoutImportState::new();

        let job = seed_takeout_cancellation_smoke_fixture_in_state(&state)
            .await
            .expect("seed smoke fixture");

        assert_eq!(job.source_id, TAKEOUT_CANCELLATION_SMOKE_FIXTURE_SOURCE_ID);
        assert_eq!(job.status, STATUS_RUNNING);
        assert!(state.cancellation_token(&job.job_id).await.is_some());
        assert_eq!(
            state
                .active_jobs_for_sources(&[TAKEOUT_CANCELLATION_SMOKE_FIXTURE_SOURCE_ID])
                .await
                .len(),
            1
        );
    }

    #[tokio::test]
    async fn takeout_cancellation_smoke_fixture_finishes_cancelled_and_clears() {
        let state = TakeoutImportState::new();
        let job = seed_takeout_cancellation_smoke_fixture_in_state(&state)
            .await
            .expect("seed smoke fixture");
        let token = state.cancellation_token(&job.job_id).await.expect("token");

        state
            .request_cancel(&job.job_id)
            .await
            .expect("request cancel");
        tokio::time::timeout(std::time::Duration::from_secs(1), token.cancelled())
            .await
            .expect("token cancelled");
        let finished = finish_cancelled_takeout_cancellation_smoke_fixture(&state, &job.job_id)
            .await
            .expect("finish smoke fixture");

        assert_eq!(finished.status, STATUS_CANCELLED);
        assert_eq!(finished.phase, super::PHASE_CANCELLED);
        assert_eq!(
            finished.message.as_deref(),
            Some("Takeout import cancelled.")
        );
        assert!(state.cancellation_token(&job.job_id).await.is_none());

        let deleted = clear_takeout_cancellation_smoke_fixture(&state)
            .await
            .expect("clear smoke fixture");
        assert_eq!(deleted, 1);
        assert!(state
            .list_jobs()
            .await
            .into_iter()
            .filter(|job| job.source_id == TAKEOUT_CANCELLATION_SMOKE_FIXTURE_SOURCE_ID)
            .collect::<Vec<_>>()
            .is_empty());
    }

    #[tokio::test]
    async fn job_state_records_history_scope_for_frontend_labels() {
        let state = TakeoutImportState::new();
        let job = state
            .create_job(7, 1, 100, TAKEOUT_HISTORY_SCOPE_MIGRATED_SMALL_GROUP)
            .await
            .expect("create historical job");

        assert_eq!(
            job.history_scope,
            TAKEOUT_HISTORY_SCOPE_MIGRATED_SMALL_GROUP
        );
    }

    #[tokio::test]
    async fn active_jobs_for_sources_filters_non_terminal_jobs() {
        let state = TakeoutImportState::new();
        let first = state
            .create_job(7, 1, 100, TAKEOUT_HISTORY_SCOPE_CURRENT)
            .await
            .expect("first job");
        let second = state
            .create_job(8, 1, 101, TAKEOUT_HISTORY_SCOPE_CURRENT)
            .await
            .expect("second job");
        let _third = state
            .create_job(9, 1, 102, TAKEOUT_HISTORY_SCOPE_CURRENT)
            .await
            .expect("third job");
        state
            .finish_job(&first.job_id, |job| {
                job.status = STATUS_COMPLETED.to_string();
                job.phase = STATUS_COMPLETED.to_string();
            })
            .await
            .expect("finish first");
        state
            .request_cancel(&second.job_id)
            .await
            .expect("cancel requested remains active");

        let active = state.active_jobs_for_sources(&[7, 8, 10]).await;

        assert_eq!(active.len(), 1);
        assert_eq!(active[0].job_id, second.job_id);
        assert_eq!(active[0].source_id, 8);
        assert_eq!(active[0].status, STATUS_CANCEL_REQUESTED);
    }
}
