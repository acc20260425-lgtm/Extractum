use std::collections::{HashMap, HashSet};

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

use crate::error::{AppError, AppResult};
use crate::job_helpers::{ActiveJobGuards, CancellationState};
use crate::time::now_secs;

const TAKEOUT_IMPORT_EVENT: &str = "sources://takeout-import";
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
        TakeoutImportState, STATUS_CANCEL_REQUESTED, STATUS_COMPLETED, STATUS_FAILED,
    };
    use crate::error::AppErrorKind;

    #[tokio::test]
    async fn job_state_rejects_duplicate_active_source_jobs() {
        let state = TakeoutImportState::new();
        let first = state.create_job(7, 1, 100).await.expect("create first job");

        let error = state
            .create_job(7, 1, 101)
            .await
            .expect_err("duplicate source job should fail");

        assert_eq!(first.job_id, "takeout-1");
        assert_eq!(first.batch_id, 100);
        assert_eq!(error.kind, AppErrorKind::Conflict);
    }

    #[tokio::test]
    async fn job_state_can_cancel_and_finish_job() {
        let state = TakeoutImportState::new();
        let job = state.create_job(7, 1, 100).await.expect("create job");

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

        let next = state.create_job(7, 1, 101).await.expect("source released");
        assert_eq!(next.job_id, "takeout-2");
        assert_eq!(next.batch_id, 101);
    }

    #[tokio::test]
    async fn active_jobs_for_sources_filters_non_terminal_jobs() {
        let state = TakeoutImportState::new();
        let first = state.create_job(7, 1, 100).await.expect("first job");
        let second = state.create_job(8, 1, 101).await.expect("second job");
        let _third = state.create_job(9, 1, 102).await.expect("third job");
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
