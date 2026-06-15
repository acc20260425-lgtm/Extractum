use sqlx::SqlitePool;

use super::dto::{
    PreflightYoutubeSummaryRunRequest, StartYoutubeSummaryRunOutcomeDto,
    StartYoutubeSummaryRunRequest,
};
use super::json_repair::JsonRepairStageExecutionRequest;
#[cfg(test)]
pub(crate) use super::youtube_summary_execution::execute_youtube_summary_run_with_fake_completions;
pub(crate) use super::youtube_summary_execution::execute_youtube_summary_run_with_stage_executor;
pub(crate) use super::youtube_summary_preflight::preflight_youtube_summary_in_pool;
use super::youtube_summary_run_store::{load_run_by_client_request_id, load_run_summary};
pub(crate) use super::youtube_summary_snapshots::create_youtube_summary_run_skeleton_in_pool;
#[cfg(test)]
pub(crate) use super::youtube_summary_snapshots::{
    freeze_comment_material_refs, test_comment_policy,
};
#[cfg(test)]
pub(crate) use super::youtube_summary_stage_outputs::{
    execute_synthesis_stage_with_completion, execute_transcript_analysis_stage_with_completion,
};
#[cfg(test)]
pub(crate) use super::youtube_summary_synthesis_input::build_synthesis_stage_input;
use crate::error::{AppError, AppResult};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModelBudget {
    pub input_token_limit: Option<i64>,
}

pub(crate) async fn start_youtube_summary_run_in_pool(
    pool: &SqlitePool,
    request: StartYoutubeSummaryRunRequest,
) -> AppResult<StartYoutubeSummaryRunOutcomeDto> {
    if request.client_request_id.trim().is_empty() {
        return Err(AppError::validation("client_request_id cannot be empty"));
    }

    if let Some(run) = load_run_by_client_request_id(pool, &request.client_request_id).await? {
        return Ok(StartYoutubeSummaryRunOutcomeDto::Started { run });
    }

    let preflight_request = PreflightYoutubeSummaryRunRequest {
        project_id: request.project_id,
        source_ids: request.source_ids.clone(),
        profile_id: request.profile_id.clone(),
        model_override: request.model_override.clone(),
        output_language: request.output_language.clone(),
        control_preset: request.control_preset.clone(),
        evidence_mode: request.evidence_mode.clone(),
        include_comments: request.include_comments,
    };
    let preflight = preflight_youtube_summary_in_pool(
        pool,
        preflight_request,
        ModelBudget {
            input_token_limit: Some(32_000),
        },
    )
    .await?;

    if preflight.included_videos.is_empty() || !preflight.blocking_failures.is_empty() {
        return Ok(StartYoutubeSummaryRunOutcomeDto::Blocked { preflight });
    }

    let run_id = create_youtube_summary_run_skeleton_in_pool(pool, request, 0).await?;
    let run = load_run_summary(pool, run_id).await?;
    Ok(StartYoutubeSummaryRunOutcomeDto::Started { run })
}

#[derive(Clone, Debug)]
pub(crate) struct LlmCompletion {
    pub text: String,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub latency_ms: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TranscriptAnalysisStageExecutionRequest {
    pub run_id: i64,
    pub stage_run_id: i64,
    pub source_snapshot_id: i64,
    pub source_ref_id: String,
    pub prompt_input_json: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum YoutubeSummaryStageExecutionRequest {
    TranscriptAnalysis(TranscriptAnalysisStageExecutionRequest),
    Synthesis(SynthesisStageExecutionRequest),
    JsonRepair(JsonRepairStageExecutionRequest),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SynthesisStageExecutionRequest {
    pub run_id: i64,
    pub stage_run_id: i64,
    pub prompt_input_json: String,
}

pub(crate) const SYNTHESIS_STAGE_NAME: &str = "youtube_summary/synthesis";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct YoutubeSummaryRunExecutionOutcome {
    pub run_id: i64,
    pub run_status: String,
    pub progress_current: i64,
    pub progress_total: i64,
    pub message: String,
}

#[derive(Debug)]
pub(crate) enum YoutubeSummaryStageExecutionError {
    Cancelled,
    Failed(AppError),
}

impl From<AppError> for YoutubeSummaryStageExecutionError {
    fn from(error: AppError) -> Self {
        Self::Failed(error)
    }
}

pub(crate) fn now_string() -> String {
    crate::time::now_rfc3339_utc()
}

pub(crate) fn estimate_tokens(text: &str) -> i64 {
    ((text.chars().count() as f64) / 4.0).ceil() as i64
}

#[cfg(test)]
mod tests {
    use super::{
        freeze_comment_material_refs, now_string, preflight_youtube_summary_in_pool,
        start_youtube_summary_run_in_pool, test_comment_policy, ModelBudget,
    };
    use crate::prompt_packs::seed::seed_builtin_prompt_packs_in_pool;
    use crate::prompt_packs::youtube_summary::{
        execute_transcript_analysis_stage_with_completion,
        execute_youtube_summary_run_with_fake_completions,
        execute_youtube_summary_run_with_stage_executor, LlmCompletion,
    };
    use crate::prompt_packs::youtube_summary_snapshots::create_youtube_summary_run_skeleton_in_pool;
    use crate::prompt_packs::youtube_summary_test_support::*;

    #[test]
    fn now_string_uses_current_utc_time() {
        use time::{format_description::well_known::Rfc3339, Duration, OffsetDateTime};

        let before = OffsetDateTime::now_utc() - Duration::seconds(5);
        let value = now_string();
        let after = OffsetDateTime::now_utc() + Duration::seconds(5);
        let parsed =
            OffsetDateTime::parse(&value, &Rfc3339).expect("parse youtube summary timestamp");

        assert_ne!(value, "2026-06-14T00:00:00Z");
        assert!(
            parsed >= before && parsed <= after,
            "expected {value} to be between {before} and {after}"
        );
    }

    #[tokio::test]
    async fn build_synthesis_stage_input_collects_successful_transcript_outputs() {
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

        let input = super::build_synthesis_stage_input(&pool, 1)
            .await
            .expect("synthesis input");

        assert_eq!(input["stage"], "youtube_summary/synthesis");
        assert_eq!(input["videos"].as_array().expect("videos").len(), 2);
        assert_eq!(
            input["claim_candidates"].as_array().expect("claims").len(),
            2
        );
        assert_eq!(
            input["evidence_fragment_candidates"]
                .as_array()
                .expect("evidence")
                .len(),
            2
        );
    }

    #[tokio::test]
    async fn build_synthesis_stage_input_uses_latest_parsed_output_wrappers() {
        let pool = test_pool_with_two_frozen_youtube_summary_sources().await;
        persist_succeeded_transcript_stage_fixtures(
            &pool,
            1,
            vec![
                TranscriptStageFixture {
                    summary: "Old first summary",
                    claim: "Old first claim",
                    evidence: "Old first evidence",
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

        let first_stage_run_id: i64 = sqlx::query_scalar(
            "SELECT id FROM prompt_pack_stage_runs
             WHERE run_id = 1 AND stage_name = 'youtube_summary/transcript_analysis'
             ORDER BY id ASC
             LIMIT 1",
        )
        .fetch_one(&pool)
        .await
        .expect("first stage row");
        crate::prompt_packs::stage_io::insert_stage_artifact_in_pool(
            &pool,
            1,
            first_stage_run_id,
            "parsed_output",
            2,
            3,
            &transcript_analysis_json("New first summary", "New first claim", "New first evidence"),
        )
        .await
        .expect("insert retry parsed output");

        let input = super::build_synthesis_stage_input(&pool, 1)
            .await
            .expect("synthesis input");
        let claims = input["claim_candidates"].as_array().expect("claims");

        assert_eq!(claims.len(), 2);
        assert_eq!(claims[0]["source_ref_id"], "source_ref_1");
        assert_eq!(claims[0]["candidate"]["text"], "New first claim");
        assert!(claims[0]["candidate"].get("source_ref_id").is_none());
    }

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

        super::execute_synthesis_stage_with_completion(
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
        super::execute_synthesis_stage_with_completion(
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
    async fn preflight_explicit_video_without_transcript_is_blocking_failure() {
        let pool = test_pool_with_youtube_video_without_transcript().await;

        let response = preflight_youtube_summary_in_pool(
            &pool,
            request_for_video(901),
            ModelBudget {
                input_token_limit: Some(32_000),
            },
        )
        .await
        .expect("preflight");

        assert!(response.included_videos.is_empty());
        assert_eq!(response.blocking_failures[0].reason, "no_usable_transcript");
    }

    #[tokio::test]
    async fn preflight_playlist_video_without_transcript_is_skipped() {
        let pool = test_pool_with_playlist_one_ready_one_missing_transcript().await;

        let response = preflight_youtube_summary_in_pool(
            &pool,
            request_for_playlist(701),
            ModelBudget {
                input_token_limit: Some(32_000),
            },
        )
        .await
        .expect("preflight");

        assert_eq!(response.included_videos.len(), 1);
        assert_eq!(response.skipped_videos[0].reason, "no_usable_transcript");
        assert!(response.blocking_failures.is_empty());
    }

    #[tokio::test]
    async fn start_freezes_one_canonical_video_snapshot_with_multiple_origins() {
        let pool = test_pool_with_same_video_selected_explicitly_and_from_playlist().await;
        let request = start_request("req-freeze-1", vec![901, 701]);

        let run_id = create_youtube_summary_run_skeleton_in_pool(&pool, request, 10)
            .await
            .expect("create run");

        let snapshot_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM prompt_pack_run_source_snapshots WHERE run_id = ?",
        )
        .bind(run_id)
        .fetch_one(&pool)
        .await
        .expect("snapshot count");

        let origin_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM prompt_pack_run_source_origins WHERE run_id = ?",
        )
        .bind(run_id)
        .fetch_one(&pool)
        .await
        .expect("origin count");

        assert_eq!(snapshot_count, 1);
        assert_eq!(origin_count, 2);
    }

    #[tokio::test]
    async fn start_returns_existing_run_for_duplicate_client_request_id() {
        let pool = test_pool_with_ready_video().await;
        let request = start_request("req-duplicate-start", vec![901]);

        let first = start_youtube_summary_run_in_pool(&pool, request.clone())
            .await
            .expect("first start")
            .expect_started("first start returns a run");
        let second = start_youtube_summary_run_in_pool(&pool, request)
            .await
            .expect("duplicate start")
            .expect_started("duplicate start returns existing run");

        assert_eq!(first.run_id, second.run_id);

        let run_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM prompt_pack_runs WHERE client_request_id = 'req-duplicate-start'",
        )
        .fetch_one(&pool)
        .await
        .expect("run count");
        assert_eq!(run_count, 1);
    }

    #[tokio::test]
    async fn start_with_recomputed_blocking_preflight_returns_response_without_run() {
        let pool = test_pool_with_youtube_video_without_transcript().await;
        seed_builtin_prompt_packs_in_pool(&pool)
            .await
            .expect("seed pack");
        let request = start_request("req-blocked-start", vec![901]);

        let outcome = start_youtube_summary_run_in_pool(&pool, request)
            .await
            .expect("start command returns structured blocking response");

        let blocking = outcome.expect_blocked("blocking response");
        assert!(blocking.included_videos.is_empty());
        assert_eq!(blocking.blocking_failures[0].reason, "no_usable_transcript");

        let run_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM prompt_pack_runs WHERE client_request_id = 'req-blocked-start'",
        )
        .fetch_one(&pool)
        .await
        .expect("run count");
        assert_eq!(run_count, 0);
    }

    #[tokio::test]
    async fn comment_snapshot_selection_is_deterministic_when_enabled() {
        let pool = test_pool_with_comments_out_of_order().await;

        let first = freeze_comment_material_refs(&pool, 901, test_comment_policy())
            .await
            .expect("first freeze");
        let second = freeze_comment_material_refs(&pool, 901, test_comment_policy())
            .await
            .expect("second freeze");

        assert_eq!(first, second);
        assert_eq!(first[0].external_id.as_deref(), Some("comment-oldest"));
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
            vec!["prompt_input", "raw_output", "parsed_output", "metrics"],
        );
    }

    #[tokio::test]
    async fn execute_queued_run_with_stage_executor_finishes_complete() {
        let pool = test_pool_with_frozen_youtube_summary_run().await;

        let outcome =
            execute_youtube_summary_run_with_stage_executor(&pool, 1, |request| async move {
                match request {
                    super::YoutubeSummaryStageExecutionRequest::TranscriptAnalysis(request) => Ok(
                        fake_completion_with_valid_transcript_analysis_json_for_source(
                            &request.source_ref_id,
                        ),
                    ),
                    super::YoutubeSummaryStageExecutionRequest::Synthesis(_) => {
                        panic!("single-video run should not request synthesis")
                    }
                    super::YoutubeSummaryStageExecutionRequest::JsonRepair(_) => {
                        panic!("valid single-video run should not request repair")
                    }
                }
            })
            .await
            .expect("execute queued run");

        let (run_status, result_status, progress_current, progress_total): (
            String,
            String,
            Option<i64>,
            Option<i64>,
        ) = sqlx::query_as(
            "SELECT run_status, result_status, progress_current, progress_total
             FROM prompt_pack_runs WHERE id = 1",
        )
        .fetch_one(&pool)
        .await
        .expect("run status");
        let result_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM prompt_pack_results WHERE run_id = 1")
                .fetch_one(&pool)
                .await
                .expect("result count");

        assert_eq!(outcome.run_status, "complete");
        assert_eq!(run_status, "complete");
        assert_eq!(result_status, "complete");
        assert_eq!(progress_current, Some(1));
        assert_eq!(progress_total, Some(1));
        assert_eq!(result_count, 1);
    }

    #[tokio::test]
    async fn execute_queued_run_repairs_malformed_transcript_json() {
        use std::sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        };

        let pool = test_pool_with_frozen_youtube_summary_run().await;
        let transcript_calls = Arc::new(AtomicUsize::new(0));
        let repair_calls = Arc::new(AtomicUsize::new(0));

        let outcome = execute_youtube_summary_run_with_stage_executor(&pool, 1, |request| {
            let transcript_calls = Arc::clone(&transcript_calls);
            let repair_calls = Arc::clone(&repair_calls);
            async move {
                match request {
                    super::YoutubeSummaryStageExecutionRequest::TranscriptAnalysis(_) => {
                        transcript_calls.fetch_add(1, Ordering::SeqCst);
                        Ok(malformed_completion())
                    }
                    super::YoutubeSummaryStageExecutionRequest::JsonRepair(request) => {
                        repair_calls.fetch_add(1, Ordering::SeqCst);
                        assert_eq!(request.stage_name, "youtube_summary/transcript_analysis");
                        assert_eq!(request.attempt_number, 2);
                        assert!(request.error_message.contains("malformed JSON braces"));
                        assert!(request.raw_output.contains("evidence_fragment_candidates"));
                        Ok(fake_completion_with_valid_transcript_analysis_json())
                    }
                    super::YoutubeSummaryStageExecutionRequest::Synthesis(_) => {
                        panic!("single-video run should not request synthesis")
                    }
                }
            }
        })
        .await
        .expect("execute repaired run");

        let stage_id = transcript_analysis_stage_id(&pool, 1).await;
        let attempts = list_stage_artifact_attempts(&pool, stage_id).await;

        assert_eq!(outcome.run_status, "complete");
        assert_eq!(transcript_calls.load(Ordering::SeqCst), 1);
        assert_eq!(repair_calls.load(Ordering::SeqCst), 1);
        assert_eq!(
            attempts,
            vec![
                ("prompt_input".to_string(), 1, 1),
                ("raw_output".to_string(), 1, 2),
                ("repair_input".to_string(), 2, 1),
                ("raw_output".to_string(), 2, 2),
                ("parsed_output".to_string(), 2, 3),
                ("metrics".to_string(), 2, 4),
            ]
        );
    }

    #[tokio::test]
    async fn execute_queued_run_repairs_malformed_synthesis_json() {
        use std::sync::{
            atomic::{AtomicUsize, Ordering},
            Arc,
        };

        let pool = test_pool_with_two_frozen_youtube_summary_sources().await;
        let synthesis_calls = Arc::new(AtomicUsize::new(0));
        let repair_calls = Arc::new(AtomicUsize::new(0));

        let outcome = execute_youtube_summary_run_with_stage_executor(&pool, 1, |request| {
            let synthesis_calls = Arc::clone(&synthesis_calls);
            let repair_calls = Arc::clone(&repair_calls);
            async move {
                match request {
                    super::YoutubeSummaryStageExecutionRequest::TranscriptAnalysis(request) => Ok(
                        fake_completion_with_valid_transcript_analysis_json_for_source(
                            &request.source_ref_id,
                        ),
                    ),
                    super::YoutubeSummaryStageExecutionRequest::Synthesis(_) => {
                        synthesis_calls.fetch_add(1, Ordering::SeqCst);
                        Ok(malformed_completion())
                    }
                    super::YoutubeSummaryStageExecutionRequest::JsonRepair(request) => {
                        repair_calls.fetch_add(1, Ordering::SeqCst);
                        assert_eq!(request.stage_name, "youtube_summary/synthesis");
                        assert_eq!(request.attempt_number, 2);
                        assert!(request.error_message.contains("malformed JSON braces"));
                        Ok(LlmCompletion {
                            text: synthesis_json("Repaired combined summary"),
                            input_tokens: Some(110),
                            output_tokens: Some(210),
                            latency_ms: 310,
                        })
                    }
                }
            }
        })
        .await
        .expect("execute repaired synthesis run");

        let synthesis_stage_id: i64 = sqlx::query_scalar(
            "SELECT id FROM prompt_pack_stage_runs
             WHERE run_id = 1 AND stage_name = 'youtube_summary/synthesis'",
        )
        .fetch_one(&pool)
        .await
        .expect("synthesis stage");
        let attempts = list_stage_artifact_attempts(&pool, synthesis_stage_id).await;

        assert_eq!(outcome.run_status, "complete");
        assert_eq!(synthesis_calls.load(Ordering::SeqCst), 1);
        assert_eq!(repair_calls.load(Ordering::SeqCst), 1);
        assert_eq!(
            attempts,
            vec![
                ("prompt_input".to_string(), 1, 1),
                ("raw_output".to_string(), 1, 2),
                ("repair_input".to_string(), 2, 1),
                ("raw_output".to_string(), 2, 2),
                ("parsed_output".to_string(), 2, 3),
                ("metrics".to_string(), 2, 4),
            ]
        );
    }

    #[tokio::test]
    async fn execute_multi_video_run_with_one_provider_failure_finishes_partial() {
        let pool = test_pool_with_two_frozen_youtube_summary_sources().await;

        execute_youtube_summary_run_with_fake_completions(
            &pool,
            1,
            vec![
                Ok(fake_completion_with_valid_transcript_analysis_json_for_source("source_ref_1")),
                Err(fake_provider_failure("provider timeout for source_ref_2")),
            ],
        )
        .await
        .expect("execute partial run");

        let run_status: String =
            sqlx::query_scalar("SELECT run_status FROM prompt_pack_runs WHERE id = 1")
                .fetch_one(&pool)
                .await
                .expect("run status");
        let result_status: String =
            sqlx::query_scalar("SELECT result_status FROM prompt_pack_results WHERE run_id = 1")
                .fetch_one(&pool)
                .await
                .expect("result status");
        let error_artifacts: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM prompt_pack_stage_artifacts \
             WHERE run_id = 1 AND artifact_kind = 'error'",
        )
        .fetch_one(&pool)
        .await
        .expect("error artifacts");
        let warning_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM prompt_pack_result_warnings WHERE run_id = 1")
                .fetch_one(&pool)
                .await
                .expect("warning count");
        let quality_flag_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM prompt_pack_result_quality_flags WHERE run_id = 1",
        )
        .fetch_one(&pool)
        .await
        .expect("quality flags");

        assert_eq!(run_status, "partial");
        assert_eq!(result_status, "partial");
        assert_eq!(error_artifacts, 1);
        assert!(warning_count > 0);
        assert!(quality_flag_count > 0);
    }

    #[tokio::test]
    async fn youtube_summary_single_video_run_skips_synthesis() {
        let pool = test_pool_with_frozen_youtube_summary_run().await;
        execute_youtube_summary_run_with_fake_completions(
            &pool,
            1,
            vec![Ok(LlmCompletion {
                text: transcript_analysis_json("Only summary", "Only claim", "Only evidence"),
                input_tokens: Some(10),
                output_tokens: Some(20),
                latency_ms: 30,
            })],
        )
        .await
        .expect("execute run");

        let status: String = sqlx::query_scalar(
            "SELECT stage_status FROM prompt_pack_stage_runs
             WHERE run_id = 1 AND stage_name = 'youtube_summary/synthesis'",
        )
        .fetch_one(&pool)
        .await
        .expect("synthesis status");

        let result =
            crate::prompt_packs::result_builder::build_youtube_summary_canonical_result(&pool, 1)
                .await
                .expect("canonical result");

        assert_eq!(status, "skipped");
        assert!(result["outputs"]["pack_data"]["youtube_summary"]["synthesis"].is_null());

        let progress: (i64, i64) = sqlx::query_as(
            "SELECT progress_current, progress_total
             FROM prompt_pack_runs
             WHERE id = 1",
        )
        .fetch_one(&pool)
        .await
        .expect("progress");

        assert_eq!(progress, (1, 1));
    }

    #[tokio::test]
    async fn youtube_summary_run_executes_synthesis_after_transcript_stages() {
        let pool = test_pool_with_two_frozen_youtube_summary_sources().await;
        execute_youtube_summary_run_with_fake_completions(
            &pool,
            1,
            vec![
                Ok(LlmCompletion {
                    text: transcript_analysis_json(
                        "First summary",
                        "First claim",
                        "First evidence",
                    ),
                    input_tokens: Some(10),
                    output_tokens: Some(20),
                    latency_ms: 30,
                }),
                Ok(LlmCompletion {
                    text: transcript_analysis_json(
                        "Second summary",
                        "Second claim",
                        "Second evidence",
                    ),
                    input_tokens: Some(11),
                    output_tokens: Some(21),
                    latency_ms: 31,
                }),
                Ok(LlmCompletion {
                    text: synthesis_json("Combined summary"),
                    input_tokens: Some(100),
                    output_tokens: Some(200),
                    latency_ms: 300,
                }),
            ],
        )
        .await
        .expect("execute run");

        let status: String = sqlx::query_scalar(
            "SELECT stage_status FROM prompt_pack_stage_runs
             WHERE run_id = 1 AND stage_name = 'youtube_summary/synthesis'",
        )
        .fetch_one(&pool)
        .await
        .expect("synthesis status");

        assert_eq!(status, "succeeded");

        let progress: (i64, i64) = sqlx::query_as(
            "SELECT progress_current, progress_total
             FROM prompt_pack_runs
             WHERE id = 1",
        )
        .fetch_one(&pool)
        .await
        .expect("progress");

        assert_eq!(progress, (3, 3));
    }

    #[tokio::test]
    async fn youtube_summary_run_marks_partial_when_synthesis_fails() {
        let pool = test_pool_with_two_frozen_youtube_summary_sources().await;
        execute_youtube_summary_run_with_fake_completions(
            &pool,
            1,
            vec![
                Ok(LlmCompletion {
                    text: transcript_analysis_json(
                        "First summary",
                        "First claim",
                        "First evidence",
                    ),
                    input_tokens: Some(10),
                    output_tokens: Some(20),
                    latency_ms: 30,
                }),
                Ok(LlmCompletion {
                    text: transcript_analysis_json(
                        "Second summary",
                        "Second claim",
                        "Second evidence",
                    ),
                    input_tokens: Some(11),
                    output_tokens: Some(21),
                    latency_ms: 31,
                }),
                Err("synthesis provider failed".to_string()),
            ],
        )
        .await
        .expect("execute run");

        let (run_status, result_status): (String, String) = sqlx::query_as(
            "SELECT runs.run_status, results.result_status
             FROM prompt_pack_runs runs
             JOIN prompt_pack_results results ON results.run_id = runs.id
             WHERE runs.id = 1",
        )
        .fetch_one(&pool)
        .await
        .expect("run result status");
        let synthesis_status: String = sqlx::query_scalar(
            "SELECT stage_status FROM prompt_pack_stage_runs
             WHERE run_id = 1 AND stage_name = 'youtube_summary/synthesis'",
        )
        .fetch_one(&pool)
        .await
        .expect("synthesis status");

        assert_eq!(run_status, "partial");
        assert_eq!(result_status, "partial");
        assert_eq!(synthesis_status, "failed");

        let progress: (i64, i64) = sqlx::query_as(
            "SELECT progress_current, progress_total
             FROM prompt_pack_runs
             WHERE id = 1",
        )
        .fetch_one(&pool)
        .await
        .expect("progress");

        assert_eq!(progress, (2, 3));
    }

    #[tokio::test]
    async fn youtube_summary_run_marks_partial_when_synthesis_output_is_invalid() {
        let pool = test_pool_with_two_frozen_youtube_summary_sources().await;
        execute_youtube_summary_run_with_fake_completions(
            &pool,
            1,
            vec![
                Ok(LlmCompletion {
                    text: transcript_analysis_json(
                        "First summary",
                        "First claim",
                        "First evidence",
                    ),
                    input_tokens: Some(10),
                    output_tokens: Some(20),
                    latency_ms: 30,
                }),
                Ok(LlmCompletion {
                    text: transcript_analysis_json(
                        "Second summary",
                        "Second claim",
                        "Second evidence",
                    ),
                    input_tokens: Some(11),
                    output_tokens: Some(21),
                    latency_ms: 31,
                }),
                Ok(LlmCompletion {
                    text: synthesis_json_with_backend_owned_id(),
                    input_tokens: Some(100),
                    output_tokens: Some(200),
                    latency_ms: 300,
                }),
            ],
        )
        .await
        .expect("execute run");

        let (run_status, result_status): (String, String) = sqlx::query_as(
            "SELECT runs.run_status, results.result_status
             FROM prompt_pack_runs runs
             JOIN prompt_pack_results results ON results.run_id = runs.id
             WHERE runs.id = 1",
        )
        .fetch_one(&pool)
        .await
        .expect("run result status");
        let synthesis_status: String = sqlx::query_scalar(
            "SELECT stage_status FROM prompt_pack_stage_runs
             WHERE run_id = 1 AND stage_name = 'youtube_summary/synthesis'",
        )
        .fetch_one(&pool)
        .await
        .expect("synthesis status");
        let quarantine_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM prompt_pack_result_quarantine_artifacts
             WHERE run_id = 1",
        )
        .fetch_one(&pool)
        .await
        .expect("quarantine count");
        let progress: (i64, i64) = sqlx::query_as(
            "SELECT progress_current, progress_total
             FROM prompt_pack_runs
             WHERE id = 1",
        )
        .fetch_one(&pool)
        .await
        .expect("progress");

        assert_eq!(run_status, "partial");
        assert_eq!(result_status, "partial");
        assert_eq!(synthesis_status, "failed");
        assert_eq!(quarantine_count, 1);
        assert_eq!(progress, (2, 3));
    }

    #[tokio::test]
    async fn youtube_summary_multi_video_partial_transcripts_skip_synthesis_and_mark_partial() {
        let pool = test_pool_with_two_frozen_youtube_summary_sources().await;
        execute_youtube_summary_run_with_fake_completions(
            &pool,
            1,
            vec![
                Ok(LlmCompletion {
                    text: transcript_analysis_json(
                        "First summary",
                        "First claim",
                        "First evidence",
                    ),
                    input_tokens: Some(10),
                    output_tokens: Some(20),
                    latency_ms: 30,
                }),
                Err("transcript provider failed".to_string()),
            ],
        )
        .await
        .expect("execute run");

        let (run_status, result_status): (String, String) = sqlx::query_as(
            "SELECT runs.run_status, results.result_status
             FROM prompt_pack_runs runs
             JOIN prompt_pack_results results ON results.run_id = runs.id
             WHERE runs.id = 1",
        )
        .fetch_one(&pool)
        .await
        .expect("run result status");
        let synthesis_status: String = sqlx::query_scalar(
            "SELECT stage_status FROM prompt_pack_stage_runs
             WHERE run_id = 1 AND stage_name = 'youtube_summary/synthesis'",
        )
        .fetch_one(&pool)
        .await
        .expect("synthesis status");

        assert_eq!(run_status, "partial");
        assert_eq!(result_status, "partial");
        assert_eq!(synthesis_status, "skipped");

        let progress: (i64, i64) = sqlx::query_as(
            "SELECT progress_current, progress_total
             FROM prompt_pack_runs
             WHERE id = 1",
        )
        .fetch_one(&pool)
        .await
        .expect("progress");

        assert_eq!(progress, (1, 2));
    }
}
