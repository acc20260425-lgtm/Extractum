use super::super::{
    finalize_analysis_report_execution, mark_interrupted_analysis_runs,
    request_analysis_run_cancel_for_pool, AnalysisExecutionError,
};
use super::super::super::state::AnalysisState;
use super::super::super::{ANALYSIS_STATUS_CANCELLED, ANALYSIS_STATUS_COMPLETED, ANALYSIS_STATUS_RUNNING};
use super::harness::{insert_cancel_request_run, request_cancel_pool_with_runs};
use extractum_core::error::AppErrorKind;
use extractum_llm::LlmSchedulerState;

struct NoopAnalysisEventSink;

impl super::super::super::models::AnalysisEventSink for NoopAnalysisEventSink {
    fn publish_run(&self, _event: super::super::super::models::AnalysisRunEvent) {}

    fn publish_chat(&self, _event: super::super::super::models::AnalysisChatEvent) {}
}

#[tokio::test]
async fn terminal_cleanup_removes_active_state_when_terminal_persistence_fails() {
    let pool = request_cancel_pool_with_runs().await;
    let state = AnalysisState::new();
    let run_id = 407;
    state.insert_active_report_run(run_id).await;
    sqlx::query("DROP TABLE analysis_runs")
        .execute(&pool)
        .await
        .expect("remove persistence target");

    finalize_analysis_report_execution(
        Some(&pool),
        &state,
        &NoopAnalysisEventSink,
        run_id,
        Err(AnalysisExecutionError::Failed(
            "terminal persistence failed".to_string(),
        )),
    )
    .await;

    assert!(!state.active_report_run_ids().await.contains(&run_id));
}

#[tokio::test]
async fn interrupted_cleanup_preserves_captured_snapshot_state_marker() {
    let pool = super::super::super::test_schema::analysis_test_pool().await;
    sqlx::query(
        "INSERT INTO analysis_runs (
            id, run_type, scope_type, period_from, period_to, output_language,
            prompt_template_version, provider_profile, provider, model, status,
            snapshot_captured_at, snapshot_error, created_at
         ) VALUES (
            1, 'report', 'single_source', 1, 2, 'English',
            1, 'research', 'gemini', 'gemini-2.5-flash', 'running',
            '2026-05-18T10:00:00Z', NULL, 1
         )",
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

    assert_eq!(row.0, ANALYSIS_STATUS_CANCELLED);
    assert_eq!(row.1.as_deref(), Some("2026-05-18T10:00:00Z"));
    assert_eq!(row.2, None);
}

#[tokio::test]
async fn request_analysis_run_cancel_missing_run_keeps_not_found_message() {
    let pool = request_cancel_pool_with_runs().await;
    let state = AnalysisState::new();
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
    insert_cancel_request_run(&pool, 405, ANALYSIS_STATUS_COMPLETED).await;
    let state = AnalysisState::new();
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
    insert_cancel_request_run(&pool, 406, ANALYSIS_STATUS_RUNNING).await;
    let state = AnalysisState::new();
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
