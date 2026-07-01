use sqlx::{Pool, Sqlite};
use tauri::AppHandle;

use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::llm::LlmSchedulerState;

use super::super::state::AnalysisState;
use super::super::store::{
    fetch_run_row, mark_run_capture_failed, sanitize_provider_error, set_run_status,
};
use super::super::{
    now_secs, ANALYSIS_STATUS_CANCELLED, ANALYSIS_STATUS_FAILED, ANALYSIS_STATUS_QUEUED,
    ANALYSIS_STATUS_RUNNING,
};
use super::{RunEvent, INTERRUPTED_RUN_MESSAGE};

pub(super) async fn fail_run(handle: &AppHandle, run_id: i64, error: String) {
    let sanitized_error = sanitize_provider_error("Report run failed", &error);
    if let Ok(pool) = get_pool(handle).await {
        let _ = set_run_status(
            &pool,
            run_id,
            ANALYSIS_STATUS_FAILED,
            None,
            None,
            Some(&sanitized_error),
            Some(now_secs()),
        )
        .await;
    }

    RunEvent::new(run_id, "failed", "persist")
        .message("Report run failed.".to_string())
        .error(sanitized_error)
        .emit(handle);
}

pub(super) async fn fail_capture_run(handle: &AppHandle, run_id: i64, error: String) {
    if let Ok(pool) = get_pool(handle).await {
        let _ = mark_run_capture_failed(&pool, run_id, &error, now_secs()).await;
    }

    RunEvent::new(run_id, "failed", "persist")
        .message("Report run failed before snapshot capture completed.".to_string())
        .error(error)
        .emit(handle);
}

pub(super) async fn cancel_run(handle: &AppHandle, run_id: i64, message: String) {
    if let Ok(pool) = get_pool(handle).await {
        let _ = set_run_status(
            &pool,
            run_id,
            ANALYSIS_STATUS_CANCELLED,
            None,
            None,
            Some(&message),
            Some(now_secs()),
        )
        .await;
    }

    RunEvent::new(run_id, "cancelled", "persist")
        .message(message)
        .emit(handle);
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

pub async fn cleanup_interrupted_analysis_runs(handle: AppHandle) {
    if let Ok(pool) = get_pool(&handle).await {
        let _ = mark_interrupted_analysis_runs(&pool).await;
    }
}

pub(super) async fn request_analysis_run_cancel_for_pool(
    pool: &Pool<Sqlite>,
    state: &AnalysisState,
    scheduler: &LlmSchedulerState,
    run_id: i64,
) -> AppResult<String> {
    let run = fetch_run_row(pool, run_id)
        .await?
        .ok_or_else(|| AppError::not_found(format!("Analysis run {run_id} not found")))?;

    if run.status != ANALYSIS_STATUS_QUEUED && run.status != ANALYSIS_STATUS_RUNNING {
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

    Ok(run.status)
}

pub(crate) async fn request_analysis_run_cancel(
    handle: &AppHandle,
    state: &AnalysisState,
    scheduler: &LlmSchedulerState,
    run_id: i64,
) -> AppResult<()> {
    let pool = get_pool(handle).await?;
    let status = request_analysis_run_cancel_for_pool(&pool, state, scheduler, run_id).await?;

    RunEvent::new(run_id, "progress", &status)
        .message("Cancelling analysis run...".to_string())
        .emit(handle);

    Ok(())
}
