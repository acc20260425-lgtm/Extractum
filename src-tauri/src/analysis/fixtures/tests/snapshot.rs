use super::super::{
    seed_analysis_redesign_fixtures_in_pool, CAPTURE_FAILED_SNAPSHOT_ERROR,
    CAPTURE_FAILED_SNAPSHOT_RUN_LABEL, COMPLETED_SNAPSHOT_RUN_LABEL, GROUP_SNAPSHOT_RUN_LABEL,
    MISSING_SNAPSHOT_RUN_LABEL,
};
use super::harness::{count, fixture_pool};
#[tokio::test]
async fn seeded_snapshot_runs_expose_captured_snapshot_state() {
    let pool = fixture_pool().await;
    seed_analysis_redesign_fixtures_in_pool(&pool)
        .await
        .expect("seed fixtures");

    for label in [COMPLETED_SNAPSHOT_RUN_LABEL, GROUP_SNAPSHOT_RUN_LABEL] {
        let run_id: i64 =
            sqlx::query_scalar("SELECT id FROM analysis_runs WHERE scope_label_snapshot = ?")
                .bind(label)
                .fetch_one(&pool)
                .await
                .expect("load fixture run id");
        let detail = crate::analysis::store::fetch_run_row(&pool, run_id)
            .await
            .expect("fetch fixture run")
            .map(crate::analysis::store::map_run_detail)
            .expect("fixture run exists");

        assert_eq!(
            detail.snapshot_state,
            Some(crate::analysis::models::AnalysisSnapshotState::Captured),
            "{label} should expose captured snapshot state"
        );
        assert!(
            detail.snapshot_captured_at.is_some(),
            "{label} should expose snapshot capture marker"
        );
        assert_eq!(detail.snapshot_error, None);
    }
}

#[tokio::test]
async fn fixture_trace_refs_cover_youtube_timestamp_and_telegram_snapshot() {
    let pool = fixture_pool().await;
    seed_analysis_redesign_fixtures_in_pool(&pool)
        .await
        .expect("seed fixtures");

    let youtube_trace: Vec<u8> = sqlx::query_scalar(
        "SELECT trace_data_zstd FROM analysis_runs WHERE scope_label_snapshot = ?",
    )
    .bind(COMPLETED_SNAPSHOT_RUN_LABEL)
    .fetch_one(&pool)
    .await
    .expect("load youtube trace");
    let telegram_trace: Vec<u8> = sqlx::query_scalar(
        "SELECT trace_data_zstd FROM analysis_runs WHERE scope_label_snapshot = ?",
    )
    .bind(GROUP_SNAPSHOT_RUN_LABEL)
    .fetch_one(&pool)
    .await
    .expect("load telegram trace");

    let youtube_json: serde_json::Value = serde_json::from_slice(
        &crate::compression::decompress_bytes(&youtube_trace).expect("decompress youtube trace"),
    )
    .expect("parse youtube trace");
    let telegram_json: serde_json::Value = serde_json::from_slice(
        &crate::compression::decompress_bytes(&telegram_trace).expect("decompress telegram trace"),
    )
    .expect("parse telegram trace");

    assert!(youtube_json["refs"]
        .as_array()
        .expect("youtube refs")
        .iter()
        .any(|value| value["ref"]
            .as_str()
            .unwrap_or_default()
            .contains("@754000ms")));
    assert!(telegram_json["refs"]
        .as_array()
        .expect("telegram refs")
        .iter()
        .any(|value| value["source_type"] == "telegram"));
}

#[tokio::test]
async fn missing_snapshot_run_exposes_capture_failed_state_but_no_saved_messages() {
    let pool = fixture_pool().await;
    seed_analysis_redesign_fixtures_in_pool(&pool)
        .await
        .expect("seed fixtures");

    let run_id: i64 =
        sqlx::query_scalar("SELECT id FROM analysis_runs WHERE scope_label_snapshot = ?")
            .bind(MISSING_SNAPSHOT_RUN_LABEL)
            .fetch_one(&pool)
            .await
            .expect("load missing snapshot run");

    let summaries = crate::analysis::store::list_analysis_run_summaries(
        &pool,
        crate::analysis::store::AnalysisRunListFilters {
            query: Some(MISSING_SNAPSHOT_RUN_LABEL.to_string()),
            limit: 100,
            ..Default::default()
        },
    )
    .await
    .expect("list fixture runs");
    let summary = summaries
        .iter()
        .find(|run| run.scope_label == MISSING_SNAPSHOT_RUN_LABEL)
        .expect("missing snapshot summary");
    assert_eq!(
        summary.snapshot_state,
        Some(crate::analysis::models::AnalysisSnapshotState::CaptureFailed)
    );

    let detail = crate::analysis::store::fetch_run_row(&pool, run_id)
        .await
        .expect("fetch missing snapshot run")
        .map(crate::analysis::store::map_run_detail)
        .expect("missing snapshot run exists");
    assert_eq!(
        detail.snapshot_state,
        Some(crate::analysis::models::AnalysisSnapshotState::CaptureFailed)
    );
    assert_eq!(detail.snapshot_error, None);

    assert_eq!(
        count(
            &pool,
            &format!("SELECT COUNT(*) FROM analysis_run_messages WHERE run_id = {run_id}")
        )
        .await,
        0
    );
    assert_eq!(
        count(
            &pool,
            &format!(
                "SELECT COUNT(*) FROM analysis_runs WHERE id = {run_id} AND trace_data_zstd IS NOT NULL"
            )
        )
        .await,
        1
    );
}

#[tokio::test]
async fn capture_failed_snapshot_run_has_sanitized_error_trace_and_readable_report() {
    let pool = fixture_pool().await;
    seed_analysis_redesign_fixtures_in_pool(&pool)
        .await
        .expect("seed fixtures");

    let run_id: i64 =
        sqlx::query_scalar("SELECT id FROM analysis_runs WHERE scope_label_snapshot = ?")
            .bind(CAPTURE_FAILED_SNAPSHOT_RUN_LABEL)
            .fetch_one(&pool)
            .await
            .expect("load capture failed snapshot run");

    let summaries = crate::analysis::store::list_analysis_run_summaries(
        &pool,
        crate::analysis::store::AnalysisRunListFilters {
            query: Some(CAPTURE_FAILED_SNAPSHOT_RUN_LABEL.to_string()),
            limit: 100,
            ..Default::default()
        },
    )
    .await
    .expect("list fixture runs");
    let summary = summaries
        .iter()
        .find(|run| run.scope_label == CAPTURE_FAILED_SNAPSHOT_RUN_LABEL)
        .expect("capture failed summary");
    assert_eq!(
        summary.snapshot_state,
        Some(crate::analysis::models::AnalysisSnapshotState::CaptureFailed)
    );
    assert_eq!(
        summary.snapshot_error.as_deref(),
        Some(CAPTURE_FAILED_SNAPSHOT_ERROR)
    );

    let detail = crate::analysis::store::fetch_run_row(&pool, run_id)
        .await
        .expect("fetch capture failed snapshot run")
        .map(crate::analysis::store::map_run_detail)
        .expect("capture failed snapshot run exists");
    assert_eq!(detail.status, "failed");
    assert!(detail
        .result_markdown
        .as_deref()
        .unwrap_or_default()
        .contains("This capture-failed fixture report remains readable."));
    assert_eq!(
        detail.snapshot_state,
        Some(crate::analysis::models::AnalysisSnapshotState::CaptureFailed)
    );
    assert_eq!(
        detail.snapshot_error.as_deref(),
        Some(CAPTURE_FAILED_SNAPSHOT_ERROR)
    );
    assert_eq!(detail.snapshot_captured_at, None);

    assert_eq!(
        count(
            &pool,
            &format!("SELECT COUNT(*) FROM analysis_run_messages WHERE run_id = {run_id}")
        )
        .await,
        0
    );
    assert_eq!(
        count(
            &pool,
            &format!(
                "SELECT COUNT(*) FROM analysis_runs WHERE id = {run_id} AND trace_data_zstd IS NOT NULL"
            )
        )
        .await,
        1
    );
}
