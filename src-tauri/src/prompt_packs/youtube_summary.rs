use std::future::Future;

use sqlx::SqlitePool;

use super::dto::{
    PreflightYoutubeSummaryRunRequest, PromptPackRunSummaryDto, StartYoutubeSummaryRunOutcomeDto,
    StartYoutubeSummaryRunRequest, YoutubeSummaryPreflightFailure, YoutubeSummaryPreflightResponse,
    YoutubeSummaryPreflightSkippedVideo, YoutubeSummaryPreflightVideo,
};
use crate::compression::{compress_text, decompress_text};
use crate::error::{AppError, AppResult};
use crate::prompt_packs::stage_io::{
    build_transcript_analysis_stage_input, extract_json_payload, insert_stage_artifact_in_pool,
};
use crate::prompt_packs::validation::{
    validate_and_quarantine_synthesis_output, validate_transcript_analysis_output,
};
use crate::prompt_packs::{
    projections::persist_final_result_transaction,
    result_builder::build_youtube_summary_canonical_result,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModelBudget {
    pub input_token_limit: Option<i64>,
}

#[derive(Clone, Debug)]
struct SourceRow {
    id: i64,
    source_type: String,
    source_subtype: Option<String>,
    title: Option<String>,
}

#[derive(Clone, Debug)]
struct VideoCandidate {
    source_id: i64,
    video_id: String,
    title: String,
    description: Option<String>,
    is_playlist_child: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CommentSelectionPolicy {
    pub comment_count_cap: usize,
    pub comment_token_cap: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CommentMaterialRef {
    pub external_id: Option<String>,
    pub material_ref_id: String,
    pub token_estimate: i64,
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct JsonRepairStageExecutionRequest {
    pub run_id: i64,
    pub stage_run_id: i64,
    pub stage_name: String,
    pub attempt_number: i64,
    pub prompt_input_json: String,
    pub raw_output: String,
    pub error_message: String,
}

const SYNTHESIS_STAGE_NAME: &str = "youtube_summary/synthesis";
// Metrics-only schema identifier for this slice. Do not seed it into
// prompt_pack_schemas until a foundation/schema task adds the asset.
const SYNTHESIS_SCHEMA_ID: &str = "stage-io/youtube_summary_synthesis_output";

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

#[derive(Clone, Debug)]
struct TranscriptStageRow {
    stage_run_id: i64,
    source_snapshot_id: i64,
    source_ref_id: String,
}

pub(crate) async fn execute_transcript_analysis_stage_with_completion(
    pool: &SqlitePool,
    stage_run_id: i64,
    completion: LlmCompletion,
) -> AppResult<()> {
    let (run_id,): (i64,) =
        sqlx::query_as("SELECT run_id FROM prompt_pack_stage_runs WHERE id = ?")
            .bind(stage_run_id)
            .fetch_one(pool)
            .await
            .map_err(AppError::database)?;
    let input = build_transcript_analysis_stage_input(pool, stage_run_id).await?;
    let input_json = serde_json::to_string(&input)
        .map_err(|error| AppError::internal(format!("serialize stage input: {error}")))?;
    insert_stage_artifact_in_pool(
        pool,
        run_id,
        stage_run_id,
        "prompt_input",
        1,
        1,
        &input_json,
    )
    .await?;

    sqlx::query(
        "UPDATE prompt_pack_stage_runs
         SET stage_status = 'running', started_at = ?, updated_at = ?
         WHERE id = ?",
    )
    .bind(now_string())
    .bind(now_string())
    .bind(stage_run_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;

    insert_stage_artifact_in_pool(
        pool,
        run_id,
        stage_run_id,
        "raw_output",
        1,
        2,
        &completion.text,
    )
    .await?;
    let parsed = extract_json_payload(&completion.text)?;
    let parsed_json = serde_json::to_string(&parsed)
        .map_err(|error| AppError::internal(format!("serialize parsed output: {error}")))?;
    insert_stage_artifact_in_pool(
        pool,
        run_id,
        stage_run_id,
        "parsed_output",
        1,
        3,
        &parsed_json,
    )
    .await?;
    validate_transcript_analysis_output(&input, &parsed)
        .map_err(|error| AppError::validation(error.message))?;
    let metrics = serde_json::json!({
        "input_tokens": completion.input_tokens,
        "output_tokens": completion.output_tokens,
        "latency_ms": completion.latency_ms,
        "schema_id": "stage-io/youtube_summary_transcript_analysis_output",
        "validation_error_count": 0,
        "attempt_number": 1
    });
    insert_stage_artifact_in_pool(
        pool,
        run_id,
        stage_run_id,
        "metrics",
        1,
        4,
        &metrics.to_string(),
    )
    .await?;
    sqlx::query(
        "UPDATE prompt_pack_stage_runs
         SET stage_status = 'succeeded', completed_at = ?, updated_at = ?
         WHERE id = ?",
    )
    .bind(now_string())
    .bind(now_string())
    .bind(stage_run_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

async fn execute_transcript_analysis_stage_repair_completion(
    pool: &SqlitePool,
    stage_run_id: i64,
    completion: LlmCompletion,
    attempt_number: i64,
) -> AppResult<()> {
    let (run_id,): (i64,) =
        sqlx::query_as("SELECT run_id FROM prompt_pack_stage_runs WHERE id = ?")
            .bind(stage_run_id)
            .fetch_one(pool)
            .await
            .map_err(AppError::database)?;
    let input = build_transcript_analysis_stage_input(pool, stage_run_id).await?;

    sqlx::query(
        "UPDATE prompt_pack_stage_runs
         SET stage_status = 'running', updated_at = ?
         WHERE id = ?",
    )
    .bind(now_string())
    .bind(stage_run_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;

    insert_stage_artifact_in_pool(
        pool,
        run_id,
        stage_run_id,
        "raw_output",
        attempt_number,
        2,
        &completion.text,
    )
    .await?;
    let parsed = extract_json_payload(&completion.text)?;
    validate_transcript_analysis_output(&input, &parsed)
        .map_err(|error| AppError::validation(error.message))?;
    let parsed_json = serde_json::to_string(&parsed)
        .map_err(|error| AppError::internal(format!("serialize parsed output: {error}")))?;
    insert_stage_artifact_in_pool(
        pool,
        run_id,
        stage_run_id,
        "parsed_output",
        attempt_number,
        3,
        &parsed_json,
    )
    .await?;
    let metrics = serde_json::json!({
        "input_tokens": completion.input_tokens,
        "output_tokens": completion.output_tokens,
        "latency_ms": completion.latency_ms,
        "schema_id": "stage-io/youtube_summary_transcript_analysis_output",
        "validation_error_count": 0,
        "attempt_number": attempt_number,
        "repaired_from_attempt": attempt_number - 1
    });
    insert_stage_artifact_in_pool(
        pool,
        run_id,
        stage_run_id,
        "metrics",
        attempt_number,
        4,
        &metrics.to_string(),
    )
    .await?;
    sqlx::query(
        "UPDATE prompt_pack_stage_runs
         SET stage_status = 'succeeded',
             error_message = NULL,
             latest_message = 'Repaired JSON output',
             completed_at = ?,
             updated_at = ?
         WHERE id = ?",
    )
    .bind(now_string())
    .bind(now_string())
    .bind(stage_run_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

pub(crate) async fn execute_synthesis_stage_with_completion(
    pool: &SqlitePool,
    stage_run_id: i64,
    completion: LlmCompletion,
) -> AppResult<()> {
    let (run_id,): (i64,) =
        sqlx::query_as("SELECT run_id FROM prompt_pack_stage_runs WHERE id = ?")
            .bind(stage_run_id)
            .fetch_one(pool)
            .await
            .map_err(AppError::database)?;

    sqlx::query(
        "UPDATE prompt_pack_stage_runs
         SET stage_status = 'running', started_at = COALESCE(started_at, ?), updated_at = ?
         WHERE id = ?",
    )
    .bind(now_string())
    .bind(now_string())
    .bind(stage_run_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;

    let input = build_synthesis_stage_input(pool, run_id).await?;
    let input_json = serde_json::to_string(&input)
        .map_err(|error| AppError::internal(format!("serialize synthesis stage input: {error}")))?;
    insert_stage_artifact_in_pool(
        pool,
        run_id,
        stage_run_id,
        "prompt_input",
        1,
        1,
        &input_json,
    )
    .await?;

    insert_stage_artifact_in_pool(
        pool,
        run_id,
        stage_run_id,
        "raw_output",
        1,
        2,
        &completion.text,
    )
    .await?;

    let parsed = match extract_json_payload(&completion.text) {
        Ok(parsed) => parsed,
        Err(error) => {
            mark_synthesis_stage_failed(pool, stage_run_id, &error.message).await?;
            return Err(error);
        }
    };
    if let Err(error) =
        validate_and_quarantine_synthesis_output(pool, run_id, stage_run_id, &parsed).await
    {
        mark_synthesis_stage_failed(pool, stage_run_id, &error.message).await?;
        return Err(error);
    }

    let parsed_json = serde_json::to_string(&parsed).map_err(|error| {
        AppError::internal(format!("serialize synthesis parsed output: {error}"))
    })?;
    insert_stage_artifact_in_pool(
        pool,
        run_id,
        stage_run_id,
        "parsed_output",
        1,
        3,
        &parsed_json,
    )
    .await?;

    let metrics = serde_json::json!({
        "input_tokens": completion.input_tokens,
        "output_tokens": completion.output_tokens,
        "latency_ms": completion.latency_ms,
        "schema_id": SYNTHESIS_SCHEMA_ID,
        "validation_error_count": 0,
        "attempt_number": 1
    });
    insert_stage_artifact_in_pool(
        pool,
        run_id,
        stage_run_id,
        "metrics",
        1,
        4,
        &metrics.to_string(),
    )
    .await?;

    sqlx::query(
        "UPDATE prompt_pack_stage_runs
         SET stage_status = 'succeeded', completed_at = ?, updated_at = ?
         WHERE id = ?",
    )
    .bind(now_string())
    .bind(now_string())
    .bind(stage_run_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

async fn execute_synthesis_stage_repair_completion(
    pool: &SqlitePool,
    stage_run_id: i64,
    completion: LlmCompletion,
    attempt_number: i64,
) -> AppResult<()> {
    let (run_id,): (i64,) =
        sqlx::query_as("SELECT run_id FROM prompt_pack_stage_runs WHERE id = ?")
            .bind(stage_run_id)
            .fetch_one(pool)
            .await
            .map_err(AppError::database)?;

    sqlx::query(
        "UPDATE prompt_pack_stage_runs
         SET stage_status = 'running', updated_at = ?
         WHERE id = ?",
    )
    .bind(now_string())
    .bind(stage_run_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;

    insert_stage_artifact_in_pool(
        pool,
        run_id,
        stage_run_id,
        "raw_output",
        attempt_number,
        2,
        &completion.text,
    )
    .await?;
    let parsed = extract_json_payload(&completion.text)?;
    validate_and_quarantine_synthesis_output(pool, run_id, stage_run_id, &parsed).await?;
    let parsed_json = serde_json::to_string(&parsed).map_err(|error| {
        AppError::internal(format!("serialize synthesis parsed output: {error}"))
    })?;
    insert_stage_artifact_in_pool(
        pool,
        run_id,
        stage_run_id,
        "parsed_output",
        attempt_number,
        3,
        &parsed_json,
    )
    .await?;
    let metrics = serde_json::json!({
        "input_tokens": completion.input_tokens,
        "output_tokens": completion.output_tokens,
        "latency_ms": completion.latency_ms,
        "schema_id": SYNTHESIS_SCHEMA_ID,
        "validation_error_count": 0,
        "attempt_number": attempt_number,
        "repaired_from_attempt": attempt_number - 1
    });
    insert_stage_artifact_in_pool(
        pool,
        run_id,
        stage_run_id,
        "metrics",
        attempt_number,
        4,
        &metrics.to_string(),
    )
    .await?;

    sqlx::query(
        "UPDATE prompt_pack_stage_runs
         SET stage_status = 'succeeded',
             error_message = NULL,
             latest_message = 'Repaired JSON output',
             completed_at = ?,
             updated_at = ?
         WHERE id = ?",
    )
    .bind(now_string())
    .bind(now_string())
    .bind(stage_run_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

async fn insert_json_repair_input_artifact(
    pool: &SqlitePool,
    request: &JsonRepairStageExecutionRequest,
) -> AppResult<()> {
    let content = serde_json::json!({
        "stage": request.stage_name,
        "failed_attempt_number": request.attempt_number - 1,
        "repair_attempt_number": request.attempt_number,
        "error_message": request.error_message,
        "prompt_input_json": request.prompt_input_json,
        "raw_output": request.raw_output
    })
    .to_string();
    insert_stage_artifact_in_pool(
        pool,
        request.run_id,
        request.stage_run_id,
        "repair_input",
        request.attempt_number,
        1,
        &content,
    )
    .await
}

async fn mark_synthesis_stage_failed(
    pool: &SqlitePool,
    stage_run_id: i64,
    error: &str,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE prompt_pack_stage_runs
         SET stage_status = 'failed',
             error_message = ?,
             latest_message = ?,
             completed_at = ?,
             updated_at = ?
         WHERE id = ?",
    )
    .bind(error)
    .bind(error)
    .bind(now_string())
    .bind(now_string())
    .bind(stage_run_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

pub(crate) async fn execute_youtube_summary_run_with_fake_completions(
    pool: &SqlitePool,
    run_id: i64,
    completions: Vec<Result<LlmCompletion, String>>,
) -> AppResult<()> {
    let stages = sqlx::query_as::<_, (i64,)>(
        "SELECT id FROM prompt_pack_stage_runs
         WHERE run_id = ? AND stage_name = 'youtube_summary/transcript_analysis'
         ORDER BY id ASC",
    )
    .bind(run_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    let total = stages.len() as i64;
    let mut completions = completions.into_iter();
    mark_run_running(pool, run_id, total).await?;

    let mut successes = 0_i64;
    let mut failures = 0_i64;
    for (stage_id,) in stages {
        let completion = completions
            .next()
            .ok_or_else(|| AppError::internal("missing fake transcript-analysis completion"))?;
        match completion {
            Ok(completion) => {
                execute_transcript_analysis_stage_with_completion(pool, stage_id, completion)
                    .await?;
                successes += 1;
            }
            Err(error) => {
                failures += 1;
                insert_stage_artifact_in_pool(
                    pool,
                    run_id,
                    stage_id,
                    "error",
                    1,
                    1,
                    &serde_json::json!({ "error": error }).to_string(),
                )
                .await?;
                sqlx::query(
                    "UPDATE prompt_pack_stage_runs
                     SET stage_status = 'failed', error_message = ?, completed_at = ?, updated_at = ?
                     WHERE id = ?",
                )
                .bind(error)
                .bind(now_string())
                .bind(now_string())
                .bind(stage_id)
                .execute(pool)
                .await
                .map_err(AppError::database)?;
            }
        }
        update_run_progress(pool, run_id, successes, total).await?;
    }

    let synthesis_status = if successes > 1 {
        let synthesis_stage_id = synthesis_stage_id(pool, run_id).await?;
        update_run_progress(pool, run_id, successes, total + 1).await?;
        let completion = completions
            .next()
            .ok_or_else(|| AppError::internal("missing fake synthesis completion"))?;
        match completion {
            Ok(completion) => {
                match execute_synthesis_stage_with_completion(pool, synthesis_stage_id, completion)
                    .await
                {
                    Ok(()) => {
                        update_run_progress(pool, run_id, successes + 1, total + 1).await?;
                        "succeeded"
                    }
                    Err(error) => {
                        mark_synthesis_stage_failed_with_artifact(
                            pool,
                            run_id,
                            synthesis_stage_id,
                            &error.message,
                        )
                        .await?;
                        update_run_progress(pool, run_id, successes, total + 1).await?;
                        "failed"
                    }
                }
            }
            Err(error) => {
                mark_synthesis_stage_failed_with_artifact(pool, run_id, synthesis_stage_id, &error)
                    .await?;
                update_run_progress(pool, run_id, successes, total + 1).await?;
                "failed"
            }
        }
    } else {
        mark_synthesis_stage_skipped(pool, run_id).await?;
        "skipped"
    };

    mark_pending_mvp_tail_stages_skipped(pool, run_id).await?;
    let final_status = terminal_status_for_synthesis(successes, failures, total, synthesis_status);
    persist_minimal_execution_result(pool, run_id, final_status).await?;
    Ok(())
}

pub(crate) async fn execute_youtube_summary_run_with_stage_executor<F, Fut>(
    pool: &SqlitePool,
    run_id: i64,
    mut execute_stage: F,
) -> AppResult<YoutubeSummaryRunExecutionOutcome>
where
    F: FnMut(YoutubeSummaryStageExecutionRequest) -> Fut,
    Fut: Future<Output = Result<LlmCompletion, YoutubeSummaryStageExecutionError>>,
{
    let stages = load_pending_transcript_stage_rows(pool, run_id).await?;
    let total = stages.len() as i64;
    mark_run_running(pool, run_id, total).await?;

    let mut successes = 0_i64;
    let mut failures = 0_i64;
    for stage in stages {
        if is_run_cancelled(pool, run_id).await? {
            mark_transcript_stage_cancelled(pool, stage.stage_run_id).await?;
            return Ok(cancelled_outcome(run_id, successes, total));
        }

        let input = build_transcript_analysis_stage_input(pool, stage.stage_run_id).await?;
        let prompt_input_json = serde_json::to_string_pretty(&input)
            .map_err(|error| AppError::internal(format!("serialize stage input: {error}")))?;
        let request = TranscriptAnalysisStageExecutionRequest {
            run_id,
            stage_run_id: stage.stage_run_id,
            source_snapshot_id: stage.source_snapshot_id,
            source_ref_id: stage.source_ref_id,
            prompt_input_json,
        };
        let repair_prompt_input_json = request.prompt_input_json.clone();

        match execute_stage(YoutubeSummaryStageExecutionRequest::TranscriptAnalysis(
            request,
        ))
        .await
        {
            Ok(completion) => {
                let raw_output = completion.text.clone();
                match execute_transcript_analysis_stage_with_completion(
                    pool,
                    stage.stage_run_id,
                    completion,
                )
                .await
                {
                    Ok(()) => successes += 1,
                    Err(error) => {
                        let repair_request = JsonRepairStageExecutionRequest {
                            run_id,
                            stage_run_id: stage.stage_run_id,
                            stage_name: "youtube_summary/transcript_analysis".to_string(),
                            attempt_number: 2,
                            prompt_input_json: repair_prompt_input_json,
                            raw_output,
                            error_message: error.message,
                        };
                        insert_json_repair_input_artifact(pool, &repair_request).await?;
                        match execute_stage(YoutubeSummaryStageExecutionRequest::JsonRepair(
                            repair_request,
                        ))
                        .await
                        {
                            Ok(repair_completion) => {
                                match execute_transcript_analysis_stage_repair_completion(
                                    pool,
                                    stage.stage_run_id,
                                    repair_completion,
                                    2,
                                )
                                .await
                                {
                                    Ok(()) => successes += 1,
                                    Err(error) => {
                                        failures += 1;
                                        mark_transcript_stage_failed(
                                            pool,
                                            run_id,
                                            stage.stage_run_id,
                                            &error.message,
                                        )
                                        .await?;
                                    }
                                }
                            }
                            Err(YoutubeSummaryStageExecutionError::Cancelled) => {
                                mark_transcript_stage_cancelled(pool, stage.stage_run_id).await?;
                                mark_run_cancelled(pool, run_id, successes, total).await?;
                                return Ok(cancelled_outcome(run_id, successes, total));
                            }
                            Err(YoutubeSummaryStageExecutionError::Failed(error)) => {
                                failures += 1;
                                mark_transcript_stage_failed(
                                    pool,
                                    run_id,
                                    stage.stage_run_id,
                                    &error.message,
                                )
                                .await?;
                            }
                        }
                    }
                }
            }
            Err(error) => match error {
                YoutubeSummaryStageExecutionError::Cancelled => {
                    mark_transcript_stage_cancelled(pool, stage.stage_run_id).await?;
                    mark_run_cancelled(pool, run_id, successes, total).await?;
                    return Ok(cancelled_outcome(run_id, successes, total));
                }
                YoutubeSummaryStageExecutionError::Failed(error) => {
                    failures += 1;
                    mark_transcript_stage_failed(pool, run_id, stage.stage_run_id, &error.message)
                        .await?;
                }
            },
        }

        update_run_progress(pool, run_id, successes, total).await?;
    }

    let synthesis_status =
        execute_synthesis_if_ready(pool, run_id, successes, total, &mut execute_stage).await?;
    if synthesis_status == "cancelled" {
        mark_run_cancelled(pool, run_id, successes, total + 1).await?;
        return Ok(cancelled_outcome(run_id, successes, total + 1));
    }
    mark_pending_mvp_tail_stages_skipped(pool, run_id).await?;
    let terminal_status =
        terminal_status_for_synthesis(successes, failures, total, synthesis_status);
    let progress_total = if successes > 1 { total + 1 } else { total };
    let progress_current = if synthesis_status == "succeeded" {
        successes + 1
    } else {
        successes
    };
    let canonical = build_youtube_summary_canonical_result(pool, run_id).await?;
    persist_final_result_transaction(pool, run_id, canonical, terminal_status).await?;
    let message = terminal_message(terminal_status).to_string();
    sqlx::query(
        "UPDATE prompt_pack_runs
         SET latest_message = ?, progress_current = ?, progress_total = ?, updated_at = ?
         WHERE id = ?",
    )
    .bind(&message)
    .bind(progress_current)
    .bind(progress_total)
    .bind(now_string())
    .bind(run_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;

    Ok(YoutubeSummaryRunExecutionOutcome {
        run_id,
        run_status: terminal_status.to_string(),
        progress_current,
        progress_total,
        message,
    })
}

async fn load_pending_transcript_stage_rows(
    pool: &SqlitePool,
    run_id: i64,
) -> AppResult<Vec<TranscriptStageRow>> {
    sqlx::query_as::<_, (i64, i64, String)>(
        "SELECT stages.id, snapshots.id, snapshots.source_ref_id
         FROM prompt_pack_stage_runs stages
         JOIN prompt_pack_run_source_snapshots snapshots
           ON snapshots.id = stages.source_snapshot_id
          AND snapshots.run_id = stages.run_id
         WHERE stages.run_id = ?
           AND stages.stage_name = 'youtube_summary/transcript_analysis'
           AND stages.stage_status = 'pending'
         ORDER BY stages.id ASC",
    )
    .bind(run_id)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(
                |(stage_run_id, source_snapshot_id, source_ref_id)| TranscriptStageRow {
                    stage_run_id,
                    source_snapshot_id,
                    source_ref_id,
                },
            )
            .collect()
    })
    .map_err(AppError::database)
}

async fn mark_run_running(pool: &SqlitePool, run_id: i64, total: i64) -> AppResult<()> {
    sqlx::query(
        "UPDATE prompt_pack_runs
         SET run_status = 'running',
             started_at = COALESCE(started_at, ?),
             latest_message = 'Running',
             progress_current = COALESCE(progress_current, 0),
             progress_total = ?,
             updated_at = ?
         WHERE id = ? AND run_status = 'queued'",
    )
    .bind(now_string())
    .bind(total)
    .bind(now_string())
    .bind(run_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

async fn is_run_cancelled(pool: &SqlitePool, run_id: i64) -> AppResult<bool> {
    sqlx::query_scalar::<_, String>("SELECT run_status FROM prompt_pack_runs WHERE id = ?")
        .bind(run_id)
        .fetch_one(pool)
        .await
        .map(|status| status == "cancelled")
        .map_err(AppError::database)
}

async fn update_run_progress(
    pool: &SqlitePool,
    run_id: i64,
    successes: i64,
    total: i64,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE prompt_pack_runs
         SET progress_current = ?,
             progress_total = ?,
             latest_message = ?,
             updated_at = ?
         WHERE id = ?",
    )
    .bind(successes)
    .bind(total)
    .bind(format!("Processed {successes} of {total} video(s)"))
    .bind(now_string())
    .bind(run_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

async fn mark_run_cancelled(
    pool: &SqlitePool,
    run_id: i64,
    progress_current: i64,
    progress_total: i64,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE prompt_pack_runs
         SET run_status = 'cancelled',
             latest_message = 'Cancelled',
             progress_current = ?,
             progress_total = ?,
             completed_at = COALESCE(completed_at, ?),
             updated_at = ?
         WHERE id = ?",
    )
    .bind(progress_current)
    .bind(progress_total)
    .bind(now_string())
    .bind(now_string())
    .bind(run_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

async fn mark_transcript_stage_failed(
    pool: &SqlitePool,
    run_id: i64,
    stage_run_id: i64,
    error: &str,
) -> AppResult<()> {
    insert_stage_artifact_in_pool(
        pool,
        run_id,
        stage_run_id,
        "error",
        1,
        99,
        &serde_json::json!({ "error": error }).to_string(),
    )
    .await?;
    sqlx::query(
        "UPDATE prompt_pack_stage_runs
         SET stage_status = 'failed',
             error_message = ?,
             latest_message = ?,
             completed_at = ?,
             updated_at = ?
         WHERE id = ?",
    )
    .bind(error)
    .bind(error)
    .bind(now_string())
    .bind(now_string())
    .bind(stage_run_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

async fn mark_transcript_stage_cancelled(pool: &SqlitePool, stage_run_id: i64) -> AppResult<()> {
    sqlx::query(
        "UPDATE prompt_pack_stage_runs
         SET stage_status = 'cancelled',
             latest_message = 'Cancelled',
             completed_at = COALESCE(completed_at, ?),
             updated_at = ?
         WHERE id = ? AND stage_status IN ('pending', 'running')",
    )
    .bind(now_string())
    .bind(now_string())
    .bind(stage_run_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

async fn synthesis_stage_id(pool: &SqlitePool, run_id: i64) -> AppResult<i64> {
    sqlx::query_scalar(
        "SELECT id FROM prompt_pack_stage_runs
         WHERE run_id = ? AND stage_name = ?
         ORDER BY id DESC LIMIT 1",
    )
    .bind(run_id)
    .bind(SYNTHESIS_STAGE_NAME)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)
}

async fn mark_synthesis_stage_skipped(pool: &SqlitePool, run_id: i64) -> AppResult<()> {
    sqlx::query(
        "UPDATE prompt_pack_stage_runs
         SET stage_status = 'skipped',
             latest_message = 'Not enough successful videos for synthesis',
             completed_at = COALESCE(completed_at, ?),
             updated_at = ?
         WHERE run_id = ?
           AND stage_name = ?
           AND stage_status IN ('pending', 'not_implemented')",
    )
    .bind(now_string())
    .bind(now_string())
    .bind(run_id)
    .bind(SYNTHESIS_STAGE_NAME)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

async fn mark_synthesis_stage_failed_with_artifact(
    pool: &SqlitePool,
    run_id: i64,
    stage_run_id: i64,
    error: &str,
) -> AppResult<()> {
    insert_stage_artifact_in_pool(
        pool,
        run_id,
        stage_run_id,
        "error",
        1,
        99,
        &serde_json::json!({ "error": error }).to_string(),
    )
    .await?;
    mark_synthesis_stage_failed(pool, stage_run_id, error).await
}

async fn execute_synthesis_if_ready<F, Fut>(
    pool: &SqlitePool,
    run_id: i64,
    successes: i64,
    transcript_total: i64,
    execute_stage: &mut F,
) -> AppResult<&'static str>
where
    F: FnMut(YoutubeSummaryStageExecutionRequest) -> Fut,
    Fut: Future<Output = Result<LlmCompletion, YoutubeSummaryStageExecutionError>>,
{
    if successes <= 1 {
        mark_synthesis_stage_skipped(pool, run_id).await?;
        return Ok("skipped");
    }

    let stage_run_id = synthesis_stage_id(pool, run_id).await?;
    update_run_progress(pool, run_id, successes, transcript_total + 1).await?;
    let input = build_synthesis_stage_input(pool, run_id).await?;
    let prompt_input_json = serde_json::to_string_pretty(&input)
        .map_err(|error| AppError::internal(format!("serialize synthesis stage input: {error}")))?;
    let request = SynthesisStageExecutionRequest {
        run_id,
        stage_run_id,
        prompt_input_json: prompt_input_json.clone(),
    };

    match execute_stage(YoutubeSummaryStageExecutionRequest::Synthesis(request)).await {
        Ok(completion) => {
            let raw_output = completion.text.clone();
            match execute_synthesis_stage_with_completion(pool, stage_run_id, completion).await {
                Ok(()) => {
                    update_run_progress(pool, run_id, successes + 1, transcript_total + 1).await?;
                    Ok("succeeded")
                }
                Err(error) => {
                    let repair_request = JsonRepairStageExecutionRequest {
                        run_id,
                        stage_run_id,
                        stage_name: SYNTHESIS_STAGE_NAME.to_string(),
                        attempt_number: 2,
                        prompt_input_json,
                        raw_output,
                        error_message: error.message,
                    };
                    insert_json_repair_input_artifact(pool, &repair_request).await?;
                    match execute_stage(YoutubeSummaryStageExecutionRequest::JsonRepair(
                        repair_request,
                    ))
                    .await
                    {
                        Ok(repair_completion) => {
                            match execute_synthesis_stage_repair_completion(
                                pool,
                                stage_run_id,
                                repair_completion,
                                2,
                            )
                            .await
                            {
                                Ok(()) => {
                                    update_run_progress(
                                        pool,
                                        run_id,
                                        successes + 1,
                                        transcript_total + 1,
                                    )
                                    .await?;
                                    Ok("succeeded")
                                }
                                Err(error) => {
                                    mark_synthesis_stage_failed_with_artifact(
                                        pool,
                                        run_id,
                                        stage_run_id,
                                        &error.message,
                                    )
                                    .await?;
                                    update_run_progress(
                                        pool,
                                        run_id,
                                        successes,
                                        transcript_total + 1,
                                    )
                                    .await?;
                                    Ok("failed")
                                }
                            }
                        }
                        Err(YoutubeSummaryStageExecutionError::Cancelled) => Ok("cancelled"),
                        Err(YoutubeSummaryStageExecutionError::Failed(error)) => {
                            mark_synthesis_stage_failed_with_artifact(
                                pool,
                                run_id,
                                stage_run_id,
                                &error.message,
                            )
                            .await?;
                            update_run_progress(pool, run_id, successes, transcript_total + 1)
                                .await?;
                            Ok("failed")
                        }
                    }
                }
            }
        }
        Err(YoutubeSummaryStageExecutionError::Cancelled) => Ok("cancelled"),
        Err(YoutubeSummaryStageExecutionError::Failed(error)) => {
            mark_synthesis_stage_failed_with_artifact(pool, run_id, stage_run_id, &error.message)
                .await?;
            update_run_progress(pool, run_id, successes, transcript_total + 1).await?;
            Ok("failed")
        }
    }
}

async fn mark_pending_mvp_tail_stages_skipped(pool: &SqlitePool, run_id: i64) -> AppResult<()> {
    sqlx::query(
        "UPDATE prompt_pack_stage_runs
         SET stage_status = 'skipped',
             latest_message = 'Handled by combined MVP stage',
             completed_at = ?,
             updated_at = ?
         WHERE run_id = ?
           AND stage_status = 'pending'
           AND stage_name NOT IN (
               'youtube_summary/transcript_analysis',
               'youtube_summary/synthesis'
           )",
    )
    .bind(now_string())
    .bind(now_string())
    .bind(run_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

fn terminal_message(status: &str) -> &'static str {
    match status {
        "complete" => "Completed",
        "partial" => "Completed with partial results",
        _ => "Failed",
    }
}

fn cancelled_outcome(
    run_id: i64,
    progress_current: i64,
    progress_total: i64,
) -> YoutubeSummaryRunExecutionOutcome {
    YoutubeSummaryRunExecutionOutcome {
        run_id,
        run_status: "cancelled".to_string(),
        progress_current,
        progress_total,
        message: "Cancelled".to_string(),
    }
}

async fn persist_minimal_execution_result(
    pool: &SqlitePool,
    run_id: i64,
    result_status: &str,
) -> AppResult<()> {
    let canonical = serde_json::json!({
        "schema_version": "1.0",
        "result_id": format!("result_{run_id}"),
        "run_id": run_id,
        "pack_id": "youtube_summary",
        "pack_version": "1.0.0",
        "stage": "youtube_summary/transcript_analysis",
        "created_at": now_string(),
        "output_language": "en",
        "metadata": {},
        "run_context": {},
        "outputs": { "pack_data": { "youtube_summary": { "videos": [] } } },
        "source_refs": [],
        "claims": [],
        "evidence": [],
        "warnings": [],
        "limitations": [],
        "quality_flags": [],
        "audit_refs": []
    });
    let canonical_json = canonical.to_string();
    let result_row_id: i64 = sqlx::query_scalar(
        "INSERT INTO prompt_pack_results (
            run_id, result_id, result_status, schema_version, canonical_hash,
            canonical_json_zstd, projection_updated_at, created_at, updated_at
         )
         VALUES (?, ?, ?, '1.0', ?, ?, ?, ?, ?)
         RETURNING id",
    )
    .bind(run_id)
    .bind(format!("result_{run_id}"))
    .bind(result_status)
    .bind(format!("sha384-{}", simple_hash(&canonical_json)))
    .bind(compress_text(&canonical_json).map_err(AppError::internal)?)
    .bind(now_string())
    .bind(now_string())
    .bind(now_string())
    .fetch_one(pool)
    .await
    .map_err(AppError::database)?;

    if result_status == "partial" {
        sqlx::query(
            "INSERT INTO prompt_pack_result_warnings (
                result_row_id, run_id, warning_id, code, message
             )
             VALUES (?, ?, 'warning_1', 'partial_provider_failure', 'One or more videos failed')",
        )
        .bind(result_row_id)
        .bind(run_id)
        .execute(pool)
        .await
        .map_err(AppError::database)?;
        sqlx::query(
            "INSERT INTO prompt_pack_result_quality_flags (
                result_row_id, run_id, flag_id, severity, message
             )
             VALUES (?, ?, 'quality_flag_1', 'warning', 'Partial result')",
        )
        .bind(result_row_id)
        .bind(run_id)
        .execute(pool)
        .await
        .map_err(AppError::database)?;
    }

    sqlx::query(
        "UPDATE prompt_pack_runs
         SET run_status = ?, result_status = ?, completed_at = ?, updated_at = ?
         WHERE id = ?",
    )
    .bind(result_status)
    .bind(result_status)
    .bind(now_string())
    .bind(now_string())
    .bind(run_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

pub(crate) async fn create_youtube_summary_run_skeleton_in_pool(
    pool: &SqlitePool,
    request: StartYoutubeSummaryRunRequest,
    _pack_version_id_hint: i64,
) -> AppResult<i64> {
    if request.client_request_id.trim().is_empty() {
        return Err(AppError::validation("client_request_id cannot be empty"));
    }
    if let Some(run) = load_run_by_client_request_id(pool, &request.client_request_id).await? {
        return Ok(run.run_id);
    }

    let pack_version_id = ensure_pack_version(pool).await?;
    let preflight = preflight_youtube_summary_in_pool(
        pool,
        PreflightYoutubeSummaryRunRequest {
            project_id: request.project_id,
            source_ids: request.source_ids.clone(),
            profile_id: request.profile_id.clone(),
            model_override: request.model_override.clone(),
            output_language: request.output_language.clone(),
            control_preset: request.control_preset.clone(),
            evidence_mode: request.evidence_mode.clone(),
            include_comments: request.include_comments,
        },
        ModelBudget {
            input_token_limit: Some(32_000),
        },
    )
    .await?;
    if preflight.included_videos.is_empty() || !preflight.blocking_failures.is_empty() {
        return Err(AppError::validation(
            "start preflight did not include runnable videos",
        ));
    }

    let now = now_string();
    let request_json = serde_json::to_string(&serde_json::json!({
        "clientRequestId": request.client_request_id,
        "projectId": request.project_id,
        "sourceIds": request.source_ids,
        "outputLanguage": request.output_language,
        "controlPreset": request.control_preset,
        "evidenceMode": request.evidence_mode,
        "includeComments": request.include_comments
    }))
    .map_err(|error| AppError::internal(format!("serialize request: {error}")))?;
    let preflight_json = serde_json::to_string(&preflight)
        .map_err(|error| AppError::internal(format!("serialize preflight: {error}")))?;

    let run_id: i64 = sqlx::query_scalar(
        "INSERT INTO prompt_pack_runs (
            project_id, pack_version_id, pack_id, pack_version, schema_version,
            run_status, result_status, request_json_zstd, preflight_json_zstd,
            provider_profile_id, model, output_language, control_preset, evidence_mode,
            include_comments, latest_message, progress_current, progress_total,
            created_at, updated_at, client_request_id
         )
         VALUES (?, ?, 'youtube_summary', '1.0.0', '1.0',
            'queued', 'none', ?, ?, ?, ?, ?, ?, ?, ?, 'Queued',
            0, ?, ?, ?, ?)
         RETURNING id",
    )
    .bind(request.project_id)
    .bind(pack_version_id)
    .bind(compress_text(&request_json).map_err(AppError::internal)?)
    .bind(compress_text(&preflight_json).map_err(AppError::internal)?)
    .bind(&request.profile_id)
    .bind(&request.model_override)
    .bind(&request.output_language)
    .bind(&request.control_preset)
    .bind(&request.evidence_mode)
    .bind(request.include_comments)
    .bind(preflight.included_videos.len() as i64)
    .bind(&now)
    .bind(&now)
    .bind(&request.client_request_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)?;

    for source_id in &request.source_ids {
        let Some(source) = load_source(pool, *source_id).await? else {
            continue;
        };
        let scope_kind = match source.source_subtype.as_deref() {
            Some("playlist") => "playlist",
            _ => "explicit_video",
        };
        sqlx::query(
            "INSERT INTO prompt_pack_run_scopes (
                run_id, source_id, source_type, source_subtype, scope_kind,
                title, created_at
             )
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(run_id)
        .bind(source.id)
        .bind(&source.source_type)
        .bind(source.source_subtype.as_deref().unwrap_or("video"))
        .bind(scope_kind)
        .bind(&source.title)
        .bind(&now)
        .execute(pool)
        .await
        .map_err(AppError::database)?;
    }

    for (index, video) in preflight.included_videos.iter().enumerate() {
        let source_ref_id = format!("source_ref_{}", index + 1);
        insert_source_snapshot(pool, run_id, video, &source_ref_id, &now).await?;
        insert_material_snapshots(
            pool,
            run_id,
            video.source_id,
            &source_ref_id,
            request.include_comments,
            &now,
        )
        .await?;
    }
    insert_origins(pool, run_id, &request, &preflight, &now).await?;
    insert_stage_skeleton(pool, run_id, preflight.included_videos.len(), &now).await?;

    Ok(run_id)
}

pub(crate) async fn preflight_youtube_summary_in_pool(
    pool: &SqlitePool,
    request: PreflightYoutubeSummaryRunRequest,
    model_budget: ModelBudget,
) -> AppResult<YoutubeSummaryPreflightResponse> {
    let mut included_videos = Vec::new();
    let mut skipped_videos = Vec::new();
    let mut blocking_failures = Vec::new();
    let mut estimated_input_tokens = 0;

    for source_id in request.source_ids {
        let Some(source) = load_source(pool, source_id).await? else {
            blocking_failures.push(YoutubeSummaryPreflightFailure {
                source_id: Some(source_id),
                reason: "source_not_found".to_string(),
                message: Some("Source was not found".to_string()),
            });
            continue;
        };

        if source.source_type != "youtube" {
            blocking_failures.push(YoutubeSummaryPreflightFailure {
                source_id: Some(source.id),
                reason: "unsupported_source_type".to_string(),
                message: Some("Only YouTube sources can be summarized".to_string()),
            });
            continue;
        }

        match source.source_subtype.as_deref() {
            Some("video") => {
                if let Some(video) = load_video_candidate(pool, source.id, false).await? {
                    classify_video(
                        pool,
                        video,
                        model_budget,
                        &mut included_videos,
                        &mut skipped_videos,
                        &mut blocking_failures,
                        &mut estimated_input_tokens,
                    )
                    .await?;
                } else {
                    blocking_failures.push(YoutubeSummaryPreflightFailure {
                        source_id: Some(source.id),
                        reason: "missing_video_metadata".to_string(),
                        message: Some("YouTube video metadata is missing".to_string()),
                    });
                }
            }
            Some("playlist") => {
                let children = load_playlist_candidates(pool, source.id).await?;
                if children.is_empty() {
                    skipped_videos.push(YoutubeSummaryPreflightSkippedVideo {
                        source_id: Some(source.id),
                        video_id: None,
                        title: source.title,
                        reason: "empty_playlist".to_string(),
                    });
                }
                for child in children {
                    match child {
                        PlaylistCandidate::Linked(video) => {
                            classify_video(
                                pool,
                                video,
                                model_budget,
                                &mut included_videos,
                                &mut skipped_videos,
                                &mut blocking_failures,
                                &mut estimated_input_tokens,
                            )
                            .await?;
                        }
                        PlaylistCandidate::Unlinked { video_id, title } => {
                            skipped_videos.push(YoutubeSummaryPreflightSkippedVideo {
                                source_id: None,
                                video_id: Some(video_id),
                                title,
                                reason: "unlinked_playlist_item".to_string(),
                            });
                        }
                    }
                }
            }
            _ => blocking_failures.push(YoutubeSummaryPreflightFailure {
                source_id: Some(source.id),
                reason: "unsupported_source_subtype".to_string(),
                message: Some("Only YouTube video and playlist sources are supported".to_string()),
            }),
        }
    }

    Ok(YoutubeSummaryPreflightResponse {
        pack_id: "youtube_summary".to_string(),
        pack_version: "1.0.0".to_string(),
        included_videos,
        skipped_videos,
        blocking_failures,
        estimated_input_tokens,
        selected_model_input_limit: model_budget.input_token_limit,
    })
}

async fn classify_video(
    pool: &SqlitePool,
    video: VideoCandidate,
    model_budget: ModelBudget,
    included_videos: &mut Vec<YoutubeSummaryPreflightVideo>,
    skipped_videos: &mut Vec<YoutubeSummaryPreflightSkippedVideo>,
    blocking_failures: &mut Vec<YoutubeSummaryPreflightFailure>,
    estimated_input_tokens: &mut i64,
) -> AppResult<()> {
    let transcript_text = transcript_text_for_source(pool, video.source_id).await?;
    if transcript_text.trim().is_empty() {
        if video.is_playlist_child {
            skipped_videos.push(YoutubeSummaryPreflightSkippedVideo {
                source_id: Some(video.source_id),
                video_id: Some(video.video_id),
                title: Some(video.title),
                reason: "no_usable_transcript".to_string(),
            });
        } else {
            blocking_failures.push(YoutubeSummaryPreflightFailure {
                source_id: Some(video.source_id),
                reason: "no_usable_transcript".to_string(),
                message: Some("The selected YouTube video has no usable transcript".to_string()),
            });
        }
        return Ok(());
    }

    let token_estimate = estimate_tokens(&transcript_text)
        + estimate_tokens(video.description.as_deref().unwrap_or(""))
        + 800;
    if model_budget
        .input_token_limit
        .is_some_and(|limit| token_estimate > limit)
    {
        if video.is_playlist_child {
            skipped_videos.push(YoutubeSummaryPreflightSkippedVideo {
                source_id: Some(video.source_id),
                video_id: Some(video.video_id),
                title: Some(video.title),
                reason: "input_budget_exceeded".to_string(),
            });
        } else {
            blocking_failures.push(YoutubeSummaryPreflightFailure {
                source_id: Some(video.source_id),
                reason: "input_budget_exceeded".to_string(),
                message: Some(
                    "The selected YouTube video exceeds the model input budget".to_string(),
                ),
            });
        }
        return Ok(());
    }

    *estimated_input_tokens += token_estimate;
    included_videos.push(YoutubeSummaryPreflightVideo {
        source_id: video.source_id,
        video_id: video.video_id,
        title: video.title,
        estimated_input_tokens: token_estimate,
    });
    Ok(())
}

async fn ensure_pack_version(pool: &SqlitePool) -> AppResult<i64> {
    if let Some(id) = sqlx::query_scalar::<_, i64>(
        "SELECT id FROM prompt_pack_versions
         WHERE pack_id = 'youtube_summary' AND pack_version = '1.0.0'",
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)?
    {
        return Ok(id);
    }

    super::seed::seed_builtin_prompt_packs_in_pool(pool).await?;
    super::store::require_prompt_pack_version_id(pool, "youtube_summary", "1.0.0").await
}

async fn load_run_by_client_request_id(
    pool: &SqlitePool,
    client_request_id: &str,
) -> AppResult<Option<PromptPackRunSummaryDto>> {
    let run_id =
        sqlx::query_scalar::<_, i64>("SELECT id FROM prompt_pack_runs WHERE client_request_id = ?")
            .bind(client_request_id)
            .fetch_optional(pool)
            .await
            .map_err(AppError::database)?;
    match run_id {
        Some(run_id) => Ok(Some(load_run_summary(pool, run_id).await?)),
        None => Ok(None),
    }
}

async fn load_run_summary(pool: &SqlitePool, run_id: i64) -> AppResult<PromptPackRunSummaryDto> {
    sqlx::query_as::<
        _,
        (
            i64,
            Option<i64>,
            Option<String>,
            String,
            String,
            String,
            String,
            String,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<i64>,
            Option<i64>,
            Option<i64>,
        ),
    >(
        "SELECT id, project_id, run_label, pack_id, pack_version, run_status, result_status,
                created_at, started_at, completed_at, latest_message,
                progress_current, progress_total, queue_position
         FROM prompt_pack_runs
         WHERE id = ?",
    )
    .bind(run_id)
    .fetch_one(pool)
    .await
    .map(
        |(
            run_id,
            project_id,
            run_label,
            pack_id,
            pack_version,
            run_status,
            result_status,
            created_at,
            started_at,
            completed_at,
            latest_message,
            progress_current,
            progress_total,
            queue_position,
        )| PromptPackRunSummaryDto {
            run_id,
            project_id,
            run_label,
            pack_id,
            pack_version,
            run_status,
            result_status,
            created_at,
            started_at,
            completed_at,
            latest_message,
            progress_current,
            progress_total,
            queue_position,
        },
    )
    .map_err(AppError::database)
}

async fn insert_source_snapshot(
    pool: &SqlitePool,
    run_id: i64,
    video: &YoutubeSummaryPreflightVideo,
    source_ref_id: &str,
    now: &str,
) -> AppResult<i64> {
    sqlx::query(
        "INSERT OR IGNORE INTO prompt_pack_run_source_snapshots (
            run_id, source_id, source_ref_id, video_id, title, channel_title,
            published_at, url, created_at
         )
         SELECT ?, yvs.source_id, ?, yvs.video_id, COALESCE(yvs.title, sources.title),
                yvs.channel_title, yvs.published_at, yvs.canonical_url, ?
         FROM youtube_video_sources yvs
         JOIN sources ON sources.id = yvs.source_id
         WHERE yvs.source_id = ?
        ",
    )
    .bind(run_id)
    .bind(source_ref_id)
    .bind(now)
    .bind(video.source_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;

    sqlx::query_scalar(
        "SELECT id FROM prompt_pack_run_source_snapshots
         WHERE run_id = ? AND source_id = ?",
    )
    .bind(run_id)
    .bind(video.source_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)
}

async fn insert_material_snapshots(
    pool: &SqlitePool,
    run_id: i64,
    source_id: i64,
    source_ref_id: &str,
    include_comments: bool,
    now: &str,
) -> AppResult<()> {
    let source_snapshot_id: i64 = sqlx::query_scalar(
        "SELECT id FROM prompt_pack_run_source_snapshots WHERE run_id = ? AND source_id = ?",
    )
    .bind(run_id)
    .bind(source_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)?;

    let transcript = transcript_text_for_source(pool, source_id).await?;
    if !transcript.trim().is_empty() {
        insert_material(
            pool,
            run_id,
            source_snapshot_id,
            &format!("m_{}_transcript", source_ref_id),
            "transcript",
            None,
            0,
            &transcript,
            now,
        )
        .await?;
    }

    if let Some(description) = sqlx::query_scalar::<_, String>(
        "SELECT description FROM youtube_video_sources WHERE source_id = ?",
    )
    .bind(source_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)?
    {
        insert_material(
            pool,
            run_id,
            source_snapshot_id,
            &format!("m_{}_description", source_ref_id),
            "description",
            None,
            1,
            &description,
            now,
        )
        .await?;
    }

    if include_comments {
        for (index, comment) in freeze_comment_material_refs(pool, source_id, test_comment_policy())
            .await?
            .into_iter()
            .enumerate()
        {
            let text = load_comment_text(pool, source_id, comment.external_id.as_deref()).await?;
            insert_material(
                pool,
                run_id,
                source_snapshot_id,
                &comment.material_ref_id,
                "comment",
                comment.external_id.as_deref(),
                10 + index as i64,
                &text,
                now,
            )
            .await?;
        }
    }

    Ok(())
}

async fn insert_material(
    pool: &SqlitePool,
    run_id: i64,
    source_snapshot_id: i64,
    material_ref_id: &str,
    material_kind: &str,
    external_id: Option<&str>,
    sequence_index: i64,
    text: &str,
    now: &str,
) -> AppResult<()> {
    sqlx::query(
        "INSERT OR IGNORE INTO prompt_pack_run_material_snapshots (
            run_id, source_snapshot_id, material_ref_id, material_kind,
            external_id, sequence_index, text_zstd, token_estimate, created_at
         )
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(run_id)
    .bind(source_snapshot_id)
    .bind(material_ref_id)
    .bind(material_kind)
    .bind(external_id)
    .bind(sequence_index)
    .bind(compress_text(text).map_err(AppError::internal)?)
    .bind(estimate_tokens(text))
    .bind(now)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

async fn insert_origins(
    pool: &SqlitePool,
    run_id: i64,
    request: &StartYoutubeSummaryRunRequest,
    preflight: &YoutubeSummaryPreflightResponse,
    now: &str,
) -> AppResult<()> {
    for source_id in &request.source_ids {
        let scope_id: i64 = sqlx::query_scalar(
            "SELECT id FROM prompt_pack_run_scopes
             WHERE run_id = ? AND source_id = ?
             ORDER BY id DESC LIMIT 1",
        )
        .bind(run_id)
        .bind(source_id)
        .fetch_one(pool)
        .await
        .map_err(AppError::database)?;

        let Some(source) = load_source(pool, *source_id).await? else {
            continue;
        };
        if source.source_subtype.as_deref() == Some("playlist") {
            let rows = sqlx::query_as::<_, (Option<i64>, String)>(
                "SELECT video_source_id, video_id
                 FROM youtube_playlist_items
                 WHERE playlist_source_id = ? AND is_removed_from_playlist = 0
                 ORDER BY position ASC, id ASC",
            )
            .bind(source_id)
            .fetch_all(pool)
            .await
            .map_err(AppError::database)?;
            for (video_source_id, video_id) in rows {
                insert_one_origin(
                    pool,
                    run_id,
                    scope_id,
                    video_source_id,
                    &video_id,
                    preflight,
                    now,
                )
                .await?;
            }
        } else {
            let video_id = sqlx::query_scalar::<_, String>(
                "SELECT video_id FROM youtube_video_sources WHERE source_id = ?",
            )
            .bind(source_id)
            .fetch_one(pool)
            .await
            .map_err(AppError::database)?;
            insert_one_origin(
                pool,
                run_id,
                scope_id,
                Some(*source_id),
                &video_id,
                preflight,
                now,
            )
            .await?;
        }
    }
    Ok(())
}

async fn insert_one_origin(
    pool: &SqlitePool,
    run_id: i64,
    scope_id: i64,
    video_source_id: Option<i64>,
    video_id: &str,
    preflight: &YoutubeSummaryPreflightResponse,
    now: &str,
) -> AppResult<()> {
    let source_snapshot_id = match video_source_id {
        Some(source_id)
            if preflight
                .included_videos
                .iter()
                .any(|video| video.source_id == source_id) =>
        {
            sqlx::query_scalar::<_, i64>(
                "SELECT id FROM prompt_pack_run_source_snapshots
                 WHERE run_id = ? AND source_id = ?",
            )
            .bind(run_id)
            .bind(source_id)
            .fetch_optional(pool)
            .await
            .map_err(AppError::database)?
        }
        _ => None,
    };
    let inclusion_status = if source_snapshot_id.is_some() {
        "included"
    } else {
        "skipped"
    };
    let reason = if source_snapshot_id.is_some() {
        None
    } else {
        Some("not_included")
    };
    sqlx::query(
        "INSERT OR IGNORE INTO prompt_pack_run_source_origins (
            run_id, origin_scope_id, source_snapshot_id, video_source_id,
            video_id, inclusion_status, reason, created_at
         )
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(run_id)
    .bind(scope_id)
    .bind(source_snapshot_id)
    .bind(video_source_id)
    .bind(video_id)
    .bind(inclusion_status)
    .bind(reason)
    .bind(now)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

async fn insert_stage_skeleton(
    pool: &SqlitePool,
    run_id: i64,
    included_count: usize,
    now: &str,
) -> AppResult<()> {
    let source_ids = sqlx::query_as::<_, (i64, i64)>(
        "SELECT id, source_id FROM prompt_pack_run_source_snapshots
         WHERE run_id = ?
         ORDER BY id ASC",
    )
    .bind(run_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    insert_stage(pool, run_id, None, "source_ingestion", 10, "succeeded", now).await?;
    for (index, (snapshot_id, _)) in source_ids.into_iter().enumerate() {
        insert_stage(
            pool,
            run_id,
            Some(snapshot_id),
            "youtube_summary/transcript_analysis",
            20 + index as i64,
            "pending",
            now,
        )
        .await?;
    }
    for (offset, name) in [
        "segment_extraction",
        "key_point_extraction",
        "quote_extraction",
    ]
    .iter()
    .enumerate()
    {
        insert_stage(
            pool,
            run_id,
            None,
            name,
            100 + offset as i64,
            "not_implemented",
            now,
        )
        .await?;
    }
    let synthesis_status = if included_count > 1 {
        "pending"
    } else {
        "skipped"
    };
    insert_stage(
        pool,
        run_id,
        None,
        SYNTHESIS_STAGE_NAME,
        103,
        synthesis_status,
        now,
    )
    .await?;
    insert_stage(pool, run_id, None, "final_synthesis", 200, "pending", now).await?;
    insert_stage(pool, run_id, None, "validation", 300, "pending", now).await?;

    sqlx::query(
        "UPDATE prompt_pack_runs
         SET progress_total = ?, updated_at = ?
         WHERE id = ?",
    )
    .bind(included_count as i64)
    .bind(now)
    .bind(run_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

async fn insert_stage(
    pool: &SqlitePool,
    run_id: i64,
    source_snapshot_id: Option<i64>,
    stage_name: &str,
    stage_order: i64,
    stage_status: &str,
    now: &str,
) -> AppResult<()> {
    sqlx::query(
        "INSERT OR IGNORE INTO prompt_pack_stage_runs (
            run_id, source_snapshot_id, stage_name, stage_order, stage_status,
            created_at, updated_at
         )
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(run_id)
    .bind(source_snapshot_id)
    .bind(stage_name)
    .bind(stage_order)
    .bind(stage_status)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

pub(crate) fn test_comment_policy() -> CommentSelectionPolicy {
    CommentSelectionPolicy {
        comment_count_cap: 50,
        comment_token_cap: 4000,
    }
}

pub(crate) async fn freeze_comment_material_refs(
    pool: &SqlitePool,
    source_id: i64,
    policy: CommentSelectionPolicy,
) -> AppResult<Vec<CommentMaterialRef>> {
    let rows = sqlx::query_as::<_, (i64, String, Option<Vec<u8>>)>(
        "SELECT id, external_id, content_zstd
         FROM items
         WHERE source_id = ? AND item_kind = 'youtube_comment'
         ORDER BY published_at IS NULL ASC, published_at ASC, external_id ASC, id ASC
         LIMIT ?",
    )
    .bind(source_id)
    .bind(policy.comment_count_cap as i64)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    let mut refs = Vec::with_capacity(rows.len());
    for (index, (_id, external_id, content_zstd)) in rows.into_iter().enumerate() {
        let text = match content_zstd {
            Some(bytes) => decompress_text(&bytes).unwrap_or_default(),
            None => String::new(),
        };
        refs.push(CommentMaterialRef {
            external_id: Some(external_id),
            material_ref_id: format!("m_comment_{}", index + 1),
            token_estimate: estimate_tokens(&text).min(policy.comment_token_cap),
        });
    }
    Ok(refs)
}

async fn load_comment_text(
    pool: &SqlitePool,
    source_id: i64,
    external_id: Option<&str>,
) -> AppResult<String> {
    let Some(external_id) = external_id else {
        return Ok(String::new());
    };
    let bytes = sqlx::query_scalar::<_, Vec<u8>>(
        "SELECT content_zstd FROM items
         WHERE source_id = ? AND item_kind = 'youtube_comment' AND external_id = ?
         LIMIT 1",
    )
    .bind(source_id)
    .bind(external_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)?;
    Ok(bytes
        .as_deref()
        .and_then(|bytes| decompress_text(bytes).ok())
        .unwrap_or_default())
}

fn now_string() -> String {
    crate::time::now_rfc3339_utc()
}

fn simple_hash(value: &str) -> String {
    use sha2::{Digest, Sha384};
    Sha384::digest(value.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn estimate_tokens(text: &str) -> i64 {
    ((text.chars().count() as f64) / 4.0).ceil() as i64
}

async fn load_source(pool: &SqlitePool, source_id: i64) -> AppResult<Option<SourceRow>> {
    sqlx::query_as::<_, (i64, String, Option<String>, Option<String>)>(
        "SELECT id, source_type, source_subtype, title FROM sources WHERE id = ?",
    )
    .bind(source_id)
    .fetch_optional(pool)
    .await
    .map(|row| {
        row.map(|(id, source_type, source_subtype, title)| SourceRow {
            id,
            source_type,
            source_subtype,
            title,
        })
    })
    .map_err(AppError::database)
}

async fn load_video_candidate(
    pool: &SqlitePool,
    source_id: i64,
    is_playlist_child: bool,
) -> AppResult<Option<VideoCandidate>> {
    sqlx::query_as::<_, (String, Option<String>, Option<String>)>(
        "SELECT video_id, title, description FROM youtube_video_sources WHERE source_id = ?",
    )
    .bind(source_id)
    .fetch_optional(pool)
    .await
    .map(|row| {
        row.map(|(video_id, title, description)| VideoCandidate {
            source_id,
            title: title.unwrap_or_else(|| video_id.clone()),
            video_id,
            description,
            is_playlist_child,
        })
    })
    .map_err(AppError::database)
}

enum PlaylistCandidate {
    Linked(VideoCandidate),
    Unlinked {
        video_id: String,
        title: Option<String>,
    },
}

async fn load_playlist_candidates(
    pool: &SqlitePool,
    playlist_source_id: i64,
) -> AppResult<Vec<PlaylistCandidate>> {
    let rows = sqlx::query_as::<_, (Option<i64>, String, Option<String>)>(
        "SELECT video_source_id, video_id, title_snapshot
         FROM youtube_playlist_items
         WHERE playlist_source_id = ? AND is_removed_from_playlist = 0
         ORDER BY position ASC, id ASC",
    )
    .bind(playlist_source_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    let mut candidates = Vec::with_capacity(rows.len());
    for (video_source_id, video_id, title) in rows {
        if let Some(source_id) = video_source_id {
            if let Some(video) = load_video_candidate(pool, source_id, true).await? {
                candidates.push(PlaylistCandidate::Linked(video));
            } else {
                candidates.push(PlaylistCandidate::Unlinked { video_id, title });
            }
        } else {
            candidates.push(PlaylistCandidate::Unlinked { video_id, title });
        }
    }
    Ok(candidates)
}

async fn transcript_text_for_source(pool: &SqlitePool, source_id: i64) -> AppResult<String> {
    let segments = sqlx::query_scalar::<_, String>(
        "SELECT text
         FROM youtube_transcript_segments
         WHERE source_id = ?
         ORDER BY segment_index ASC, id ASC",
    )
    .bind(source_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;
    Ok(segments.join("\n"))
}

fn terminal_status_for_synthesis(
    successes: i64,
    failures: i64,
    transcript_total: i64,
    synthesis_status: &str,
) -> &'static str {
    if successes == 0 {
        return "failed";
    }
    if synthesis_status == "failed" {
        return "partial";
    }
    if failures > 0 || successes < transcript_total {
        return "partial";
    }
    if transcript_total > 1 && synthesis_status != "succeeded" {
        return "partial";
    }
    "complete"
}

pub(crate) async fn build_synthesis_stage_input(
    pool: &SqlitePool,
    run_id: i64,
) -> AppResult<serde_json::Value> {
    let rows = sqlx::query_as::<_, (i64, i64, String, Option<String>, Vec<u8>)>(
        "SELECT stages.id, snapshots.id, snapshots.source_ref_id, snapshots.title, artifacts.content_zstd
         FROM prompt_pack_run_source_snapshots snapshots
         JOIN prompt_pack_stage_runs stages
           ON stages.run_id = snapshots.run_id
          AND stages.source_snapshot_id = snapshots.id
          AND stages.stage_name = 'youtube_summary/transcript_analysis'
          AND stages.stage_status = 'succeeded'
         JOIN prompt_pack_stage_artifacts artifacts
           ON artifacts.stage_run_id = stages.id
          AND artifacts.artifact_kind = 'parsed_output'
          AND artifacts.id = (
              SELECT latest.id
              FROM prompt_pack_stage_artifacts latest
              WHERE latest.stage_run_id = stages.id
                AND latest.artifact_kind = 'parsed_output'
              ORDER BY latest.attempt_number DESC, latest.artifact_index DESC, latest.id DESC
              LIMIT 1
          )
         WHERE snapshots.run_id = ?
         ORDER BY snapshots.id ASC",
    )
    .bind(run_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    let mut videos = Vec::new();
    let mut claim_candidates = Vec::new();
    let mut evidence_fragment_candidates = Vec::new();
    let mut warning_candidates = Vec::new();

    for (_stage_run_id, source_snapshot_id, source_ref_id, title, content_zstd) in rows {
        let text = decompress_text(&content_zstd).map_err(AppError::internal)?;
        let parsed: serde_json::Value = serde_json::from_str(&text).map_err(|error| {
            AppError::internal(format!("parse transcript parsed_output: {error}"))
        })?;
        videos.push(serde_json::json!({
            "source_snapshot_id": source_snapshot_id,
            "source_ref_id": source_ref_id,
            "title": title,
            "video_candidate": parsed
                .get("video_candidate")
                .cloned()
                .unwrap_or_else(|| serde_json::json!({}))
        }));
        wrap_candidates(
            &mut claim_candidates,
            parsed.get("claim_candidates"),
            &source_ref_id,
        );
        wrap_candidates(
            &mut evidence_fragment_candidates,
            parsed.get("evidence_fragment_candidates"),
            &source_ref_id,
        );
        wrap_candidates(
            &mut warning_candidates,
            parsed.get("warning_candidates"),
            &source_ref_id,
        );
    }

    Ok(serde_json::json!({
        "stage_io_version": "1.0",
        "schema_version": "1.0",
        "stage": "youtube_summary/synthesis",
        "run_id": run_id,
        "videos": videos,
        "claim_candidates": claim_candidates,
        "evidence_fragment_candidates": evidence_fragment_candidates,
        "warning_candidates": warning_candidates
    }))
}

fn wrap_candidates(
    target: &mut Vec<serde_json::Value>,
    value: Option<&serde_json::Value>,
    source_ref_id: &str,
) {
    if let Some(items) = value.and_then(serde_json::Value::as_array) {
        for item in items {
            target.push(serde_json::json!({
                "source_ref_id": source_ref_id,
                "candidate": item
            }));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        create_youtube_summary_run_skeleton_in_pool, freeze_comment_material_refs, now_string,
        preflight_youtube_summary_in_pool, start_youtube_summary_run_in_pool, test_comment_policy,
        ModelBudget,
    };
    use crate::compression::compress_text;
    use crate::migrations::apply_all_migrations_for_test_pool;
    use crate::prompt_packs::dto::{
        PreflightYoutubeSummaryRunRequest, StartYoutubeSummaryRunRequest,
    };
    use crate::prompt_packs::seed::seed_builtin_prompt_packs_in_pool;
    use crate::prompt_packs::youtube_summary::{
        execute_transcript_analysis_stage_with_completion,
        execute_youtube_summary_run_with_fake_completions,
        execute_youtube_summary_run_with_stage_executor, LlmCompletion,
    };

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

    async fn migrated_pool() -> sqlx::SqlitePool {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");
        apply_all_migrations_for_test_pool(&pool)
            .await
            .expect("apply migrations");
        pool
    }

    fn request_for_video(source_id: i64) -> PreflightYoutubeSummaryRunRequest {
        PreflightYoutubeSummaryRunRequest {
            project_id: None,
            source_ids: vec![source_id],
            profile_id: None,
            model_override: Some("test-model".to_string()),
            output_language: "en".to_string(),
            control_preset: "standard".to_string(),
            evidence_mode: "standard".to_string(),
            include_comments: false,
        }
    }

    fn start_request(
        client_request_id: &str,
        source_ids: Vec<i64>,
    ) -> StartYoutubeSummaryRunRequest {
        StartYoutubeSummaryRunRequest {
            client_request_id: client_request_id.to_string(),
            project_id: None,
            source_ids,
            profile_id: None,
            model_override: Some("test-model".to_string()),
            output_language: "en".to_string(),
            control_preset: "standard".to_string(),
            evidence_mode: "standard".to_string(),
            include_comments: false,
        }
    }

    fn request_for_playlist(source_id: i64) -> PreflightYoutubeSummaryRunRequest {
        request_for_video(source_id)
    }

    async fn insert_youtube_video(pool: &sqlx::SqlitePool, source_id: i64, video_id: &str) {
        sqlx::query(
            "INSERT INTO sources (
                id, source_type, source_subtype, external_id, title,
                is_active, is_member, created_at
             )
             VALUES (?, 'youtube', 'video', ?, ?, 1, 0, 1)",
        )
        .bind(source_id)
        .bind(video_id)
        .bind(format!("Video {video_id}"))
        .execute(pool)
        .await
        .expect("insert source");

        sqlx::query(
            "INSERT INTO youtube_video_sources (
                source_id, video_id, canonical_url, title, description,
                video_form, availability_status
             )
             VALUES (?, ?, ?, ?, 'Description', 'regular', 'available')",
        )
        .bind(source_id)
        .bind(video_id)
        .bind(format!("https://www.youtube.com/watch?v={video_id}"))
        .bind(format!("Video {video_id}"))
        .execute(pool)
        .await
        .expect("insert video metadata");
    }

    async fn insert_playlist(pool: &sqlx::SqlitePool, playlist_source_id: i64) {
        sqlx::query(
            "INSERT INTO sources (
                id, source_type, source_subtype, external_id, title,
                is_active, is_member, created_at
             )
             VALUES (?, 'youtube', 'playlist', 'playlist-1', 'Playlist', 1, 0, 1)",
        )
        .bind(playlist_source_id)
        .execute(pool)
        .await
        .expect("insert playlist source");

        sqlx::query(
            "INSERT INTO youtube_playlist_sources (
                source_id, playlist_id, canonical_url, title, availability_status
             )
             VALUES (?, 'playlist-1', 'https://www.youtube.com/playlist?list=playlist-1', 'Playlist', 'available')",
        )
        .bind(playlist_source_id)
        .execute(pool)
        .await
        .expect("insert playlist metadata");
    }

    async fn insert_playlist_item(
        pool: &sqlx::SqlitePool,
        playlist_source_id: i64,
        video_source_id: Option<i64>,
        video_id: &str,
        position: i64,
    ) {
        sqlx::query(
            "INSERT INTO youtube_playlist_items (
                playlist_source_id, video_source_id, video_id, position,
                title_snapshot, availability_status, is_removed_from_playlist
             )
             VALUES (?, ?, ?, ?, ?, 'available', 0)",
        )
        .bind(playlist_source_id)
        .bind(video_source_id)
        .bind(video_id)
        .bind(position)
        .bind(format!("Video {video_id}"))
        .execute(pool)
        .await
        .expect("insert playlist item");
    }

    async fn insert_transcript(pool: &sqlx::SqlitePool, source_id: i64, text: &str) {
        let item_id: i64 = sqlx::query_scalar(
            "INSERT INTO items (
                source_id, external_id, published_at, ingested_at, item_kind
             )
             VALUES (?, ?, 1, 1, 'youtube_transcript')
             RETURNING id",
        )
        .bind(source_id)
        .bind(format!("item-{source_id}"))
        .fetch_one(pool)
        .await
        .expect("insert transcript item");

        sqlx::query(
            "INSERT INTO youtube_transcript_segments (
                item_id, source_id, segment_index, start_ms, end_ms, text
             )
             VALUES (?, ?, 0, 0, 1000, ?)",
        )
        .bind(item_id)
        .bind(source_id)
        .bind(text)
        .execute(pool)
        .await
        .expect("insert transcript segment");
    }

    async fn insert_comment(
        pool: &sqlx::SqlitePool,
        source_id: i64,
        external_id: &str,
        published_at: i64,
        text: &str,
    ) {
        sqlx::query(
            "INSERT INTO items (
                source_id, external_id, author, published_at, ingested_at,
                content_zstd, content_kind, has_media, item_kind
             )
             VALUES (?, ?, 'Alice', ?, 1, ?, 'text_only', 0, 'youtube_comment')",
        )
        .bind(source_id)
        .bind(external_id)
        .bind(published_at)
        .bind(compress_text(text).expect("compress comment"))
        .execute(pool)
        .await
        .expect("insert comment");
    }

    async fn test_pool_with_youtube_video_without_transcript() -> sqlx::SqlitePool {
        let pool = migrated_pool().await;
        insert_youtube_video(&pool, 901, "v-missing").await;
        pool
    }

    async fn test_pool_with_playlist_one_ready_one_missing_transcript() -> sqlx::SqlitePool {
        let pool = migrated_pool().await;
        insert_playlist(&pool, 701).await;
        insert_youtube_video(&pool, 901, "v-ready").await;
        insert_youtube_video(&pool, 902, "v-missing").await;
        insert_transcript(&pool, 901, "Ready transcript").await;
        insert_playlist_item(&pool, 701, Some(901), "v-ready", 1).await;
        insert_playlist_item(&pool, 701, Some(902), "v-missing", 2).await;
        pool
    }

    async fn test_pool_with_ready_video() -> sqlx::SqlitePool {
        let pool = migrated_pool().await;
        seed_builtin_prompt_packs_in_pool(&pool)
            .await
            .expect("seed pack");
        insert_youtube_video(&pool, 901, "v-ready").await;
        insert_transcript(&pool, 901, "Ready transcript").await;
        pool
    }

    async fn test_pool_with_same_video_selected_explicitly_and_from_playlist() -> sqlx::SqlitePool {
        let pool = migrated_pool().await;
        seed_builtin_prompt_packs_in_pool(&pool)
            .await
            .expect("seed pack");
        insert_playlist(&pool, 701).await;
        insert_youtube_video(&pool, 901, "v-ready").await;
        insert_transcript(&pool, 901, "Ready transcript").await;
        insert_playlist_item(&pool, 701, Some(901), "v-ready", 1).await;
        pool
    }

    async fn test_pool_with_comments_out_of_order() -> sqlx::SqlitePool {
        let pool = test_pool_with_ready_video().await;
        insert_comment(&pool, 901, "comment-newer", 20, "newer").await;
        insert_comment(&pool, 901, "comment-oldest", 10, "oldest").await;
        insert_comment(&pool, 901, "comment-middle", 15, "middle").await;
        pool
    }

    async fn test_pool_with_frozen_youtube_summary_run() -> sqlx::SqlitePool {
        let pool = test_pool_with_ready_video().await;
        let run_id = create_youtube_summary_run_skeleton_in_pool(
            &pool,
            start_request("req-execute-1", vec![901]),
            1,
        )
        .await
        .expect("run skeleton");
        assert_eq!(run_id, 1);
        pool
    }

    async fn test_pool_with_two_frozen_youtube_summary_sources() -> sqlx::SqlitePool {
        let pool = migrated_pool().await;
        seed_builtin_prompt_packs_in_pool(&pool)
            .await
            .expect("seed pack");
        insert_youtube_video(&pool, 901, "v-ready-1").await;
        insert_youtube_video(&pool, 902, "v-ready-2").await;
        insert_transcript(&pool, 901, "Ready transcript one").await;
        insert_transcript(&pool, 902, "Ready transcript two").await;
        create_youtube_summary_run_skeleton_in_pool(
            &pool,
            start_request("req-execute-2", vec![901, 902]),
            1,
        )
        .await
        .expect("run skeleton");
        pool
    }

    struct TranscriptStageFixture {
        summary: &'static str,
        claim: &'static str,
        evidence: &'static str,
    }

    fn transcript_analysis_json(summary: &str, claim: &str, evidence: &str) -> String {
        serde_json::json!({
            "stage_io_version": "1.0",
            "schema_version": "1.0",
            "stage": "youtube_summary/transcript_analysis",
            "video_candidate": {
                "summary_text": summary,
                "segment_candidates": [],
                "key_point_candidates": [],
                "quote_candidates": [],
                "action_item_candidates": [],
                "open_question_candidates": []
            },
            "claim_candidates": [
                {
                    "text": claim
                }
            ],
            "evidence_fragment_candidates": [
                {
                    "text": evidence
                }
            ],
            "warning_candidates": []
        })
        .to_string()
    }

    #[allow(dead_code)]
    fn synthesis_json(summary: &str) -> String {
        serde_json::json!({
            "stage_io_version": "1.0",
            "schema_version": "1.0",
            "stage": "youtube_summary/synthesis",
            "synthesis_candidate": {
                "summary_text": summary,
                "cross_video_themes": [
                    {
                        "theme_text": "Shared theme",
                        "source_refs": ["source_ref_1", "source_ref_2"],
                        "claim_refs": [],
                        "evidence_refs": []
                    }
                ],
                "common_claims": [],
                "contradictions_across_videos": []
            },
            "limitations": [],
            "warning_candidates": []
        })
        .to_string()
    }

    fn synthesis_json_with_backend_owned_id() -> String {
        serde_json::json!({
            "stage_io_version": "1.0",
            "schema_version": "1.0",
            "stage": "youtube_summary/synthesis",
            "synthesis_candidate": {
                "summary_text": "Invalid synthesis",
                "cross_video_themes": [
                    {
                        "theme_id": "theme_from_provider",
                        "theme_text": "Provider must not assign backend IDs",
                        "source_refs": ["source_ref_1", "source_ref_2"],
                        "claim_refs": [],
                        "evidence_refs": []
                    }
                ],
                "common_claims": [],
                "contradictions_across_videos": []
            },
            "limitations": [],
            "warning_candidates": []
        })
        .to_string()
    }

    async fn persist_succeeded_transcript_stage_fixtures(
        pool: &sqlx::SqlitePool,
        run_id: i64,
        fixtures: Vec<TranscriptStageFixture>,
    ) -> crate::error::AppResult<()> {
        let stage_rows = sqlx::query_as::<_, (i64, i64)>(
            "SELECT id, source_snapshot_id
             FROM prompt_pack_stage_runs
             WHERE run_id = ?
               AND stage_name = 'youtube_summary/transcript_analysis'
             ORDER BY id ASC",
        )
        .bind(run_id)
        .fetch_all(pool)
        .await
        .map_err(crate::error::AppError::database)?;

        assert_eq!(stage_rows.len(), fixtures.len());

        for ((stage_run_id, _source_snapshot_id), fixture) in stage_rows.into_iter().zip(fixtures) {
            sqlx::query(
                "UPDATE prompt_pack_stage_runs
                 SET stage_status = 'succeeded', updated_at = ?
                 WHERE id = ?",
            )
            .bind(super::now_string())
            .bind(stage_run_id)
            .execute(pool)
            .await
            .map_err(crate::error::AppError::database)?;

            let parsed = transcript_analysis_json(fixture.summary, fixture.claim, fixture.evidence);
            crate::prompt_packs::stage_io::insert_stage_artifact_in_pool(
                pool,
                run_id,
                stage_run_id,
                "parsed_output",
                1,
                3,
                &parsed,
            )
            .await?;
        }

        Ok(())
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

    async fn transcript_analysis_stage_id(pool: &sqlx::SqlitePool, run_id: i64) -> i64 {
        sqlx::query_scalar(
            "SELECT id FROM prompt_pack_stage_runs
             WHERE run_id = ? AND stage_name = 'youtube_summary/transcript_analysis'
             ORDER BY id ASC LIMIT 1",
        )
        .bind(run_id)
        .fetch_one(pool)
        .await
        .expect("stage id")
    }

    async fn list_stage_artifact_kinds(pool: &sqlx::SqlitePool, stage_id: i64) -> Vec<String> {
        sqlx::query_scalar(
            "SELECT artifact_kind FROM prompt_pack_stage_artifacts
             WHERE stage_run_id = ?
             ORDER BY attempt_number ASC, artifact_index ASC",
        )
        .bind(stage_id)
        .fetch_all(pool)
        .await
        .expect("artifact kinds")
    }

    async fn list_stage_artifact_attempts(
        pool: &sqlx::SqlitePool,
        stage_id: i64,
    ) -> Vec<(String, i64, i64)> {
        sqlx::query_as(
            "SELECT artifact_kind, attempt_number, artifact_index
             FROM prompt_pack_stage_artifacts
             WHERE stage_run_id = ?
             ORDER BY attempt_number ASC, artifact_index ASC",
        )
        .bind(stage_id)
        .fetch_all(pool)
        .await
        .expect("artifact attempts")
    }

    fn fake_completion_with_valid_transcript_analysis_json() -> LlmCompletion {
        fake_completion_with_valid_transcript_analysis_json_for_source("source_ref_1")
    }

    fn fake_completion_with_valid_transcript_analysis_json_for_source(
        source_ref_id: &str,
    ) -> LlmCompletion {
        LlmCompletion {
            text: serde_json::json!({
                "stage_io_version": "1.0",
                "schema_version": "1.0",
                "stage": "youtube_summary/transcript_analysis",
                "video_candidate": {
                    "summary_text": format!("Summary for {source_ref_id}"),
                    "segment_candidates": [],
                    "key_point_candidates": [],
                    "quote_candidates": [],
                    "action_item_candidates": [],
                    "open_question_candidates": []
                },
                "claim_candidates": [
                    {
                        "text": "Claim",
                        "material_refs": [format!("m_{source_ref_id}_transcript")]
                    }
                ],
                "evidence_fragment_candidates": [],
                "warning_candidates": []
            })
            .to_string(),
            input_tokens: Some(10),
            output_tokens: Some(20),
            latency_ms: 5,
        }
    }

    fn malformed_completion() -> LlmCompletion {
        LlmCompletion {
            text: r#"{
                "stage_io_version": "1.0",
                "schema_version": "1.0",
                "stage": "youtube_summary/transcript_analysis",
                "evidence_fragment_candidates":"#
                .to_string(),
            input_tokens: Some(10),
            output_tokens: Some(20),
            latency_ms: 30,
        }
    }

    fn fake_provider_failure(message: &str) -> String {
        message.to_string()
    }
}
