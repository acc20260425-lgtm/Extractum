use super::outputs::{
    execute_synthesis_stage_with_completion, execute_transcript_analysis_stage_with_completion,
};
use super::test_support::*;
use super::LlmCompletion;

#[tokio::test]
async fn execute_synthesis_stage_persists_raw_parsed_and_metrics_artifacts() {
    let pool = test_pool_with_two_frozen_youtube_summary_sources().await;
    persist_succeeded_transcript_stage_fixtures(
        &pool,
        1,
        vec![
            TranscriptStageFixture {
                summary: "First summary",
                claim: "First claim",
                evidence: "First evidence",
            },
            TranscriptStageFixture {
                summary: "Second summary",
                claim: "Second claim",
                evidence: "Second evidence",
            },
        ],
    )
    .await
    .expect("persist transcript fixtures");

    let stage_run_id: i64 = sqlx::query_scalar(
        "SELECT id FROM prompt_pack_stage_runs
         WHERE run_id = 1 AND stage_name = 'youtube_summary/synthesis'",
    )
    .fetch_one(&pool)
    .await
    .expect("stage row");

    execute_synthesis_stage_with_completion(
        &pool,
        stage_run_id,
        LlmCompletion {
            text: synthesis_json("Combined summary"),
            input_tokens: Some(100),
            output_tokens: Some(200),
            latency_ms: 300,
        },
    )
    .await
    .expect("execute synthesis");

    let kinds: Vec<String> = sqlx::query_scalar(
        "SELECT artifact_kind FROM prompt_pack_stage_artifacts
         WHERE stage_run_id = ?
         ORDER BY artifact_index ASC",
    )
    .bind(stage_run_id)
    .fetch_all(&pool)
    .await
    .expect("artifacts");

    assert_eq!(
        kinds,
        vec!["prompt_input", "raw_output", "parsed_output", "metrics"]
    );
}

#[tokio::test]
async fn execute_synthesis_stage_rejects_invalid_output_without_success_artifacts() {
    let pool = test_pool_with_two_frozen_youtube_summary_sources().await;
    persist_succeeded_transcript_stage_fixtures(
        &pool,
        1,
        vec![
            TranscriptStageFixture {
                summary: "First summary",
                claim: "First claim",
                evidence: "First evidence",
            },
            TranscriptStageFixture {
                summary: "Second summary",
                claim: "Second claim",
                evidence: "Second evidence",
            },
        ],
    )
    .await
    .expect("persist transcript fixtures");

    let stage_run_id: i64 = sqlx::query_scalar(
        "SELECT id FROM prompt_pack_stage_runs
         WHERE run_id = 1 AND stage_name = 'youtube_summary/synthesis'",
    )
    .fetch_one(&pool)
    .await
    .expect("stage row");

    let invalid = r#"{
        "stage_io_version": "1.0",
        "schema_version": "1.0",
        "stage": "youtube_summary/synthesis",
        "synthesis_candidate": {
            "summary_text": "Combined summary",
            "cross_video_themes": [{ "theme_id": "theme_1", "theme_text": "bad" }],
            "common_claims": [],
            "contradictions_across_videos": []
        },
        "limitations": [],
        "warning_candidates": []
    }"#;
    execute_synthesis_stage_with_completion(
        &pool,
        stage_run_id,
        LlmCompletion {
            text: invalid.to_string(),
            input_tokens: Some(100),
            output_tokens: Some(200),
            latency_ms: 300,
        },
    )
    .await
    .expect_err("invalid synthesis fails stage");

    let status: String =
        sqlx::query_scalar("SELECT stage_status FROM prompt_pack_stage_runs WHERE id = ?")
            .bind(stage_run_id)
            .fetch_one(&pool)
            .await
            .expect("stage status");
    let success_artifacts: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM prompt_pack_stage_artifacts
         WHERE stage_run_id = ? AND artifact_kind IN ('parsed_output', 'metrics')",
    )
    .bind(stage_run_id)
    .fetch_one(&pool)
    .await
    .expect("success artifacts");
    let quarantine_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM prompt_pack_result_quarantine_artifacts
         WHERE run_id = 1 AND stage_run_id = ?",
    )
    .bind(stage_run_id)
    .fetch_one(&pool)
    .await
    .expect("quarantine count");

    assert_eq!(status, "failed");
    assert_eq!(success_artifacts, 0);
    assert_eq!(quarantine_count, 1);
}

#[tokio::test]
async fn execute_transcript_analysis_stage_persists_raw_and_parsed_artifacts() {
    let pool = test_pool_with_frozen_youtube_summary_run().await;
    let stage_id = transcript_analysis_stage_id(&pool, 1).await;

    execute_transcript_analysis_stage_with_completion(
        &pool,
        stage_id,
        fake_completion_with_valid_transcript_analysis_json(),
    )
    .await
    .expect("execute stage");

    let artifact_kinds = list_stage_artifact_kinds(&pool, stage_id).await;
    assert_eq!(
        artifact_kinds,
        vec![
            "prompt_input",
            "raw_output",
            "parsed_output",
            "metrics",
            "intermediate_entities"
        ],
    );
}

#[tokio::test]
async fn execute_transcript_analysis_stage_persists_intermediate_entities_artifact() {
    let pool = test_pool_with_frozen_youtube_summary_run().await;
    let stage_id = transcript_analysis_stage_id(&pool, 1).await;

    execute_transcript_analysis_stage_with_completion(
        &pool,
        stage_id,
        fake_completion_with_valid_transcript_analysis_json(),
    )
    .await
    .expect("execute transcript stage");

    let artifacts = list_stage_artifact_attempts(&pool, stage_id).await;
    assert!(artifacts.contains(&("metrics".to_string(), 1, 4)));
    assert!(artifacts.contains(&("intermediate_entities".to_string(), 1, 5)));
}

#[tokio::test]
async fn repaired_transcript_analysis_persists_intermediate_entities_for_repair_attempt() {
    let pool = test_pool_with_frozen_youtube_summary_run().await;
    let stage_id = transcript_analysis_stage_id(&pool, 1).await;

    crate::prompt_packs::json_repair::execute_transcript_analysis_stage_repair_completion(
        &pool,
        stage_id,
        fake_completion_with_valid_transcript_analysis_json(),
        2,
    )
    .await
    .expect("repair transcript stage");

    let artifacts = list_stage_artifact_attempts(&pool, stage_id).await;
    assert!(artifacts.contains(&("metrics".to_string(), 2, 4)));
    assert!(artifacts.contains(&("intermediate_entities".to_string(), 2, 5)));
}

#[tokio::test]
async fn malformed_intermediate_candidates_are_quarantined_without_graph_artifact() {
    let pool = test_pool_with_frozen_youtube_summary_run().await;
    let stage_id = transcript_analysis_stage_id(&pool, 1).await;

    let error = execute_transcript_analysis_stage_with_completion(
        &pool,
        stage_id,
        fake_completion_with_malformed_intermediate_candidates_json(),
    )
    .await
    .expect_err("malformed graph candidates fail stage");

    assert!(error.message.contains("quote_candidates must be an array"));
    let (status, error_message): (String, Option<String>) = sqlx::query_as(
        "SELECT stage_status, error_message FROM prompt_pack_stage_runs WHERE id = ?",
    )
    .bind(stage_id)
    .fetch_one(&pool)
    .await
    .expect("stage status");
    assert_eq!(status, "running");
    assert!(error_message.is_none());
    let artifacts = list_stage_artifact_attempts(&pool, stage_id).await;
    assert!(!artifacts.contains(&("metrics".to_string(), 1, 4)));
    assert!(!artifacts
        .iter()
        .any(|(kind, _, _)| kind == "intermediate_entities"));
    assert!(!artifacts.iter().any(|(kind, _, _)| kind == "error"));
    assert_quarantine_count(&pool, stage_id, 1).await;
}

#[tokio::test]
async fn repair_graph_build_failure_does_not_write_repaired_parsed_output() {
    let pool = test_pool_with_frozen_youtube_summary_run().await;
    let stage_id = transcript_analysis_stage_id(&pool, 1).await;

    crate::prompt_packs::json_repair::execute_transcript_analysis_stage_repair_completion(
        &pool,
        stage_id,
        fake_completion_with_malformed_intermediate_candidates_json(),
        2,
    )
    .await
    .expect_err("repair graph failure");

    let (status, error_message): (String, Option<String>) = sqlx::query_as(
        "SELECT stage_status, error_message FROM prompt_pack_stage_runs WHERE id = ?",
    )
    .bind(stage_id)
    .fetch_one(&pool)
    .await
    .expect("stage status");
    assert_eq!(status, "running");
    assert!(error_message.is_none());
    let artifacts = list_stage_artifact_attempts(&pool, stage_id).await;
    assert!(!artifacts.contains(&("parsed_output".to_string(), 2, 3)));
    assert!(!artifacts.contains(&("metrics".to_string(), 2, 4)));
    assert!(!artifacts.contains(&("intermediate_entities".to_string(), 2, 5)));
    assert!(!artifacts.iter().any(|(kind, _, _)| kind == "error"));
    assert_quarantine_count(&pool, stage_id, 1).await;
}

async fn assert_quarantine_count(pool: &sqlx::SqlitePool, stage_id: i64, expected_count: i64) {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM prompt_pack_result_quarantine_artifacts
         WHERE run_id = 1 AND stage_run_id = ?",
    )
    .bind(stage_id)
    .fetch_one(pool)
    .await
    .expect("quarantine count");

    assert_eq!(count, expected_count);
}
