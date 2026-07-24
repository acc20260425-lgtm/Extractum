use sqlx::{Pool, Sqlite};

use extractum_core::error::{AppError, AppResult};
use extractum_llm::LlmSchedulerState;

use super::super::state::AnalysisState;
use super::super::store::{load_analysis_run_status, mark_run_capture_failed};
use super::super::{
    now_secs, ANALYSIS_STATUS_CANCELLED, ANALYSIS_STATUS_QUEUED, ANALYSIS_STATUS_RUNNING,
};
use super::{RunEvent, INTERRUPTED_RUN_MESSAGE};

pub(super) async fn persist_capture_failure_event(
    pool: Option<&Pool<Sqlite>>,
    run_id: i64,
    error: &str,
    completed_at: i64,
) -> RunEvent {
    if let Some(pool) = pool {
        let _ = mark_run_capture_failed(pool, run_id, error, completed_at).await;
    }

    RunEvent::new(run_id, "failed", "persist")
        .message("Report run failed before snapshot capture completed.".to_string())
        .error(error.to_string())
}

pub(crate) async fn mark_interrupted_analysis_runs(pool: &Pool<Sqlite>) -> AppResult<()> {
    sqlx::query(
        r#"
        UPDATE analysis_runs
        SET status = ?, error = ?, completed_at = ?
        WHERE status IN (?, ?)
        "#,
    )
    .bind(ANALYSIS_STATUS_CANCELLED)
    .bind(INTERRUPTED_RUN_MESSAGE)
    .bind(now_secs())
    .bind(ANALYSIS_STATUS_QUEUED)
    .bind(ANALYSIS_STATUS_RUNNING)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

pub(super) async fn request_analysis_run_cancel_for_pool(
    pool: &Pool<Sqlite>,
    state: &AnalysisState,
    scheduler: &LlmSchedulerState,
    run_id: i64,
) -> AppResult<String> {
    let status = load_analysis_run_status(pool, run_id)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Analysis run {run_id} not found")))?;

    if status != ANALYSIS_STATUS_QUEUED && status != ANALYSIS_STATUS_RUNNING {
        return Err(AppError::conflict(format!(
            "Analysis run {run_id} is not queued or running"
        )));
    }

    let requested = state.request_report_run_cancel(run_id).await;
    let cancelled_requests = scheduler.cancel_run_requests(run_id).await;
    if !requested && cancelled_requests == 0 {
        return Err(AppError::conflict(format!(
            "Analysis run {run_id} is no longer active"
        )));
    }

    Ok(status)
}
