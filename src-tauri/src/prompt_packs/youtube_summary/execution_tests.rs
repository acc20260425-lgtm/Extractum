use super::execution::{
    execute_youtube_summary_run_with_fake_completions,
    execute_youtube_summary_run_with_stage_executor,
};
use super::test_support::*;
use super::{LlmCompletion, YoutubeSummaryStageExecutionRequest};
use crate::error::AppError;
use crate::prompt_packs::youtube_summary::types::YoutubeSummaryStageExecutionError;

#[tokio::test]
async fn execute_queued_run_with_stage_executor_finishes_complete() {
    let pool = test_pool_with_frozen_youtube_summary_run().await;

    let outcome = execute_youtube_summary_run_with_stage_executor(&pool, 1, |request| async move {
        match request {
            YoutubeSummaryStageExecutionRequest::TranscriptAnalysis(request) => Ok(
                fake_completion_with_valid_transcript_analysis_json_for_source(
                    &request.source_ref_id,
                ),
            ),
            YoutubeSummaryStageExecutionRequest::Synthesis(_) => {
                panic!("single-video run should not request synthesis")
            }
            YoutubeSummaryStageExecutionRequest::JsonRepair(_) => {
                panic!("valid single-video run should not request repair")
            }
            YoutubeSummaryStageExecutionRequest::GemAnalysisPart(_)
            | YoutubeSummaryStageExecutionRequest::GemAnalysisPartRepair(_) => {
                panic!("standard run should not request gem analysis")
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
    let video_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM prompt_pack_youtube_videos WHERE run_id = 1")
            .fetch_one(&pool)
            .await
            .expect("video projections");
    let result_error_findings: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM prompt_pack_result_validation_findings
         WHERE run_id = 1 AND stage_run_id IS NULL AND severity = 'error'",
    )
    .fetch_one(&pool)
    .await
    .expect("result finding count");

    assert_eq!(outcome.run_status, "complete");
    assert_eq!(run_status, "complete");
    assert_eq!(result_status, "complete");
    assert_eq!(progress_current, Some(1));
    assert_eq!(progress_total, Some(1));
    assert_eq!(result_count, 1);
    assert_eq!(video_count, 1);
    assert_eq!(result_error_findings, 0);
}

#[tokio::test]
async fn youtube_summary_invalid_final_result_records_result_level_findings() {
    let pool = test_pool_with_frozen_youtube_summary_run().await;
    let outcome =
        super::execution::execute_youtube_summary_run_with_stage_executor_and_result_mutator(
            &pool,
            1,
            |request| async move {
                match request {
                    YoutubeSummaryStageExecutionRequest::TranscriptAnalysis(request) => Ok(
                        fake_completion_with_valid_transcript_analysis_json_for_source(
                            &request.source_ref_id,
                        ),
                    ),
                    YoutubeSummaryStageExecutionRequest::Synthesis(_) => {
                        panic!("single-video run should not request synthesis")
                    }
                    YoutubeSummaryStageExecutionRequest::JsonRepair(_) => {
                        panic!("valid single-video run should not request repair")
                    }
                    YoutubeSummaryStageExecutionRequest::GemAnalysisPart(_)
                    | YoutubeSummaryStageExecutionRequest::GemAnalysisPartRepair(_) => {
                        panic!("standard run should not request gem analysis")
                    }
                }
            },
            |canonical| {
                canonical["claims"][0]["claim_id"] = serde_json::json!("");
            },
        )
        .await;

    assert!(outcome.is_err());
    let result_rows: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM prompt_pack_results WHERE run_id = 1")
            .fetch_one(&pool)
            .await
            .expect("result rows");
    let result_findings: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM prompt_pack_result_validation_findings
         WHERE run_id = 1 AND stage_run_id IS NULL AND severity = 'error'",
    )
    .fetch_one(&pool)
    .await
    .expect("result findings");
    let run_status: String =
        sqlx::query_scalar("SELECT run_status FROM prompt_pack_runs WHERE id = 1")
            .fetch_one(&pool)
            .await
            .expect("run status");

    assert_eq!(result_rows, 0);
    assert!(result_findings > 0);
    assert_eq!(run_status, "failed");
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
                YoutubeSummaryStageExecutionRequest::TranscriptAnalysis(_) => {
                    transcript_calls.fetch_add(1, Ordering::SeqCst);
                    Ok(malformed_completion())
                }
                YoutubeSummaryStageExecutionRequest::JsonRepair(request) => {
                    repair_calls.fetch_add(1, Ordering::SeqCst);
                    assert_eq!(request.stage_name, "youtube_summary/transcript_analysis");
                    assert_eq!(request.attempt_number, 2);
                    assert!(request.error_message.contains("malformed JSON braces"));
                    assert!(request.raw_output.contains("evidence_fragment_candidates"));
                    Ok(fake_completion_with_valid_transcript_analysis_json())
                }
                YoutubeSummaryStageExecutionRequest::Synthesis(_) => {
                    panic!("single-video run should not request synthesis")
                }
                YoutubeSummaryStageExecutionRequest::GemAnalysisPart(_)
                | YoutubeSummaryStageExecutionRequest::GemAnalysisPartRepair(_) => {
                    panic!("standard run should not request gem analysis")
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
            ("intermediate_entities".to_string(), 2, 5),
        ]
    );
}

#[tokio::test]
async fn execution_graph_build_failure_after_failed_repair_marks_transcript_failed_once() {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    let pool = test_pool_with_frozen_youtube_summary_run().await;
    let repair_calls = Arc::new(AtomicUsize::new(0));

    let outcome = execute_youtube_summary_run_with_stage_executor(&pool, 1, |request| {
        let repair_calls = Arc::clone(&repair_calls);
        async move {
            match request {
                YoutubeSummaryStageExecutionRequest::TranscriptAnalysis(_) => {
                    Ok(fake_completion_with_malformed_intermediate_candidates_json())
                }
                YoutubeSummaryStageExecutionRequest::JsonRepair(_) => {
                    repair_calls.fetch_add(1, Ordering::SeqCst);
                    Ok(fake_completion_with_malformed_intermediate_candidates_json())
                }
                YoutubeSummaryStageExecutionRequest::Synthesis(_) => {
                    panic!("single-video failed transcript run should not request synthesis")
                }
                YoutubeSummaryStageExecutionRequest::GemAnalysisPart(_)
                | YoutubeSummaryStageExecutionRequest::GemAnalysisPartRepair(_) => {
                    panic!("standard run should not request gem analysis")
                }
            }
        }
    })
    .await
    .expect("execute run");

    let stage_id = transcript_analysis_stage_id(&pool, 1).await;
    let artifacts = list_stage_artifact_attempts(&pool, stage_id).await;
    let error_artifacts = artifacts
        .iter()
        .filter(|(kind, _, _)| kind == "error")
        .collect::<Vec<_>>();
    let (status, error_message): (String, Option<String>) = sqlx::query_as(
        "SELECT stage_status, error_message FROM prompt_pack_stage_runs WHERE id = ?",
    )
    .bind(stage_id)
    .fetch_one(&pool)
    .await
    .expect("stage status");

    assert_eq!(outcome.run_status, "failed");
    assert_eq!(repair_calls.load(Ordering::SeqCst), 1);
    assert_eq!(status, "failed");
    assert!(error_message
        .unwrap_or_default()
        .contains("quote_candidates must be an array"));
    assert_eq!(error_artifacts.len(), 1);
    assert_eq!(error_artifacts[0], &("error".to_string(), 2, 99));
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
                YoutubeSummaryStageExecutionRequest::TranscriptAnalysis(request) => Ok(
                    fake_completion_with_valid_transcript_analysis_json_for_source(
                        &request.source_ref_id,
                    ),
                ),
                YoutubeSummaryStageExecutionRequest::Synthesis(_) => {
                    synthesis_calls.fetch_add(1, Ordering::SeqCst);
                    Ok(malformed_completion())
                }
                YoutubeSummaryStageExecutionRequest::JsonRepair(request) => {
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
                YoutubeSummaryStageExecutionRequest::GemAnalysisPart(_)
                | YoutubeSummaryStageExecutionRequest::GemAnalysisPartRepair(_) => {
                    panic!("standard run should not request gem analysis")
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
                text: transcript_analysis_json("First summary", "First claim", "First evidence"),
                input_tokens: Some(10),
                output_tokens: Some(20),
                latency_ms: 30,
            }),
            Ok(LlmCompletion {
                text: transcript_analysis_json("Second summary", "Second claim", "Second evidence"),
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
                text: transcript_analysis_json("First summary", "First claim", "First evidence"),
                input_tokens: Some(10),
                output_tokens: Some(20),
                latency_ms: 30,
            }),
            Ok(LlmCompletion {
                text: transcript_analysis_json("Second summary", "Second claim", "Second evidence"),
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
                text: transcript_analysis_json("First summary", "First claim", "First evidence"),
                input_tokens: Some(10),
                output_tokens: Some(20),
                latency_ms: 30,
            }),
            Ok(LlmCompletion {
                text: transcript_analysis_json("Second summary", "Second claim", "Second evidence"),
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
                text: transcript_analysis_json("First summary", "First claim", "First evidence"),
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

#[tokio::test]
async fn execute_multi_video_run_stops_after_transcript_when_cancelled_before_synthesis() {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    let pool = test_pool_with_two_frozen_youtube_summary_sources().await;
    let transcript_calls = Arc::new(AtomicUsize::new(0));
    let transcript_calls_for_assert = Arc::clone(&transcript_calls);
    let pool_for_stage = pool.clone();
    let outcome = execute_youtube_summary_run_with_stage_executor(&pool, 1, move |request| {
        let transcript_calls = Arc::clone(&transcript_calls);
        let pool = pool_for_stage.clone();
        async move {
            match request {
                YoutubeSummaryStageExecutionRequest::TranscriptAnalysis(request) => {
                    let call_index = transcript_calls.fetch_add(1, Ordering::SeqCst) + 1;
                    let completion = fake_completion_with_valid_transcript_analysis_json_for_source(
                        &request.source_ref_id,
                    );
                    if call_index == 2 {
                        sqlx::query(
                            "UPDATE prompt_pack_runs SET run_status = 'cancelled' WHERE id = ?",
                        )
                        .bind(request.run_id)
                        .execute(&pool)
                        .await
                        .map_err(|error| {
                            YoutubeSummaryStageExecutionError::Failed(AppError::internal(format!(
                                "failed to flag run cancelled: {error}"
                            )))
                        })?;
                    }
                    Ok(completion)
                }
                YoutubeSummaryStageExecutionRequest::Synthesis(_) => {
                    panic!("cancel should prevent synthesis execution")
                }
                YoutubeSummaryStageExecutionRequest::JsonRepair(_) => {
                    panic!("valid transcript should not request repair")
                }
                YoutubeSummaryStageExecutionRequest::GemAnalysisPart(_)
                | YoutubeSummaryStageExecutionRequest::GemAnalysisPartRepair(_) => {
                    panic!("standard run should not request gem analysis")
                }
            }
        }
    })
    .await
    .expect("execute run");

    let (run_status, result_status): (String, String) =
        sqlx::query_as("SELECT run_status, result_status FROM prompt_pack_runs WHERE id = 1")
            .fetch_one(&pool)
            .await
            .expect("run status");
    let result_rows: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM prompt_pack_results WHERE run_id = 1")
            .fetch_one(&pool)
            .await
            .expect("result rows");
    let video_rows: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM prompt_pack_youtube_videos WHERE run_id = 1")
            .fetch_one(&pool)
            .await
            .expect("video rows");
    let synthesis_status: String = sqlx::query_scalar(
        "SELECT stage_status FROM prompt_pack_stage_runs
         WHERE run_id = 1 AND stage_name = 'youtube_summary/synthesis'",
    )
    .fetch_one(&pool)
    .await
    .expect("synthesis status");

    assert_eq!(outcome.run_status, "cancelled");
    assert_eq!(outcome.progress_current, 2);
    assert_eq!(outcome.progress_total, 2);
    assert_eq!(run_status, "cancelled");
    assert_eq!(result_status, "none");
    assert_eq!(result_rows, 0);
    assert_eq!(video_rows, 0);
    assert_eq!(synthesis_status, "pending");
    assert_eq!(transcript_calls_for_assert.load(Ordering::SeqCst), 2);
}
