use super::super::{mark_interrupted_analysis_runs, request_analysis_run_cancel_for_pool};
use super::harness::{insert_cancel_request_run, request_cancel_pool_with_runs};
use crate::error::AppErrorKind;
use crate::llm::LlmSchedulerState;

#[tokio::test]
async fn interrupted_cleanup_preserves_captured_snapshot_state_marker() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect memory sqlite");
    sqlx::query(
        "CREATE TABLE analysis_runs (
            id INTEGER PRIMARY KEY,
            status TEXT NOT NULL,
            error TEXT,
            completed_at INTEGER,
            snapshot_captured_at TEXT,
            snapshot_error TEXT
        )",
    )
    .execute(&pool)
    .await
    .expect("create runs");
    sqlx::query(
        "INSERT INTO analysis_runs (id, status, snapshot_captured_at, snapshot_error)
         VALUES (1, 'running', '2026-05-18T10:00:00Z', NULL)",
    )
    .execute(&pool)
    .await
    .expect("insert running captured run");

    mark_interrupted_analysis_runs(&pool)
        .await
        .expect("mark interrupted");

    let row: (String, Option<String>, Option<String>) = sqlx::query_as(
        "SELECT status, snapshot_captured_at, snapshot_error FROM analysis_runs WHERE id = 1",
    )
    .fetch_one(&pool)
    .await
    .expect("load run");

    assert_eq!(row.0, crate::analysis::ANALYSIS_STATUS_CANCELLED);
    assert_eq!(row.1.as_deref(), Some("2026-05-18T10:00:00Z"));
    assert_eq!(row.2, None);
}

#[tokio::test]
async fn request_analysis_run_cancel_missing_run_keeps_not_found_message() {
    let pool = request_cancel_pool_with_runs().await;
    let state = crate::analysis::AnalysisState::new();
    let scheduler = LlmSchedulerState::new();
    let run_id = 404;

    let error = request_analysis_run_cancel_for_pool(&pool, &state, &scheduler, run_id)
        .await
        .expect_err("missing run should fail");

    assert_eq!(error.kind, AppErrorKind::NotFound);
    assert_eq!(error.message, format!("Analysis run {run_id} not found"));
}

#[tokio::test]
async fn request_analysis_run_cancel_completed_run_keeps_conflict_message() {
    let pool = request_cancel_pool_with_runs().await;
    insert_cancel_request_run(&pool, 405, crate::analysis::ANALYSIS_STATUS_COMPLETED).await;
    let state = crate::analysis::AnalysisState::new();
    let scheduler = LlmSchedulerState::new();
    let run_id = 405;

    let error = request_analysis_run_cancel_for_pool(&pool, &state, &scheduler, run_id)
        .await
        .expect_err("completed run should fail");

    assert_eq!(error.kind, AppErrorKind::Conflict);
    assert_eq!(
        error.message,
        format!("Analysis run {run_id} is not queued or running")
    );
}

#[tokio::test]
async fn request_analysis_run_cancel_running_but_inactive_keeps_conflict_message() {
    let pool = request_cancel_pool_with_runs().await;
    insert_cancel_request_run(&pool, 406, crate::analysis::ANALYSIS_STATUS_RUNNING).await;
    let state = crate::analysis::AnalysisState::new();
    let scheduler = LlmSchedulerState::new();
    let run_id = 406;

    let error = request_analysis_run_cancel_for_pool(&pool, &state, &scheduler, run_id)
        .await
        .expect_err("inactive running run should fail");

    assert_eq!(error.kind, AppErrorKind::Conflict);
    assert_eq!(
        error.message,
        format!("Analysis run {run_id} is no longer active")
    );
}
