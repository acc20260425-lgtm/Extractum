use super::super::{
    finish_cancelled_fixture_run, fixture_run_ids, register_fixture_active_runs,
    remove_fixture_active_runs, seed_analysis_redesign_fixtures_in_pool, RUNNING_RUN_LABEL,
};
use super::harness::fixture_pool;
use crate::analysis::AnalysisState;
#[tokio::test]
async fn fixture_active_state_tracks_seeded_running_run() {
    let pool = fixture_pool().await;
    let state = AnalysisState::new();
    seed_analysis_redesign_fixtures_in_pool(&pool)
        .await
        .expect("seed fixtures");

    register_fixture_active_runs(&pool, &state)
        .await
        .expect("register active fixture runs");

    let active_run_ids = state.active_report_run_ids().await;
    let running_run_id: i64 =
        sqlx::query_scalar("SELECT id FROM analysis_runs WHERE scope_label_snapshot = ?")
            .bind(RUNNING_RUN_LABEL)
            .fetch_one(&pool)
            .await
            .expect("load running run");

    assert_eq!(active_run_ids.len(), 1);
    assert!(active_run_ids.contains(&running_run_id));
    let cancellation_wait = state
        .prepare_report_run_cancellation_wait(running_run_id)
        .await
        .expect("prepare fixture cancellation wait");

    let fixture_run_ids = fixture_run_ids(&pool).await.expect("load fixture run ids");
    remove_fixture_active_runs(&state, &fixture_run_ids).await;

    assert!(state.active_report_run_ids().await.is_empty());
    tokio::time::timeout(
        std::time::Duration::from_secs(1),
        cancellation_wait.cancelled(),
    )
    .await
    .expect("fixture child token cancelled");
}

#[tokio::test]
async fn fixture_cancel_waiter_marks_running_run_cancelled() {
    let pool = fixture_pool().await;
    let state = AnalysisState::new();
    seed_analysis_redesign_fixtures_in_pool(&pool)
        .await
        .expect("seed fixtures");
    register_fixture_active_runs(&pool, &state)
        .await
        .expect("register active fixture runs");
    let running_run_id: i64 =
        sqlx::query_scalar("SELECT id FROM analysis_runs WHERE scope_label_snapshot = ?")
            .bind(RUNNING_RUN_LABEL)
            .fetch_one(&pool)
            .await
            .expect("load running run");
    let cancellation_wait = state
        .prepare_report_run_cancellation_wait(running_run_id)
        .await
        .expect("prepare fixture cancellation wait");
    let (release_waiter, waiter_gate) = tokio::sync::oneshot::channel();
    let waiter = tokio::spawn(async move {
        waiter_gate.await.expect("release fixture waiter");
        cancellation_wait.cancelled().await;
    });

    state.request_report_run_cancel(running_run_id).await;
    state.remove_active_report_run(running_run_id).await;
    release_waiter.send(()).expect("release fixture waiter");
    tokio::time::timeout(std::time::Duration::from_secs(1), waiter)
        .await
        .expect("fixture cancellation wait completed")
        .expect("fixture waiter task completed");

    finish_cancelled_fixture_run(&pool, &state, running_run_id)
        .await
        .expect("finish cancelled fixture");

    let status: String = sqlx::query_scalar("SELECT status FROM analysis_runs WHERE id = ?")
        .bind(running_run_id)
        .fetch_one(&pool)
        .await
        .expect("load status");
    assert_eq!(status, crate::analysis::ANALYSIS_STATUS_CANCELLED);
    assert!(!state
        .active_report_run_ids()
        .await
        .contains(&running_run_id));
}
