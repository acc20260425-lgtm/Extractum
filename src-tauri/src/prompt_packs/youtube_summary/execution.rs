use std::future::Future;

use sqlx::SqlitePool;

#[cfg(test)]
use super::execution_result::persist_minimal_execution_result;
use super::execution_result::{cancelled_outcome, terminal_message, terminal_status_for_synthesis};
#[cfg(test)]
use super::outputs::execute_synthesis_stage_with_completion;
use super::outputs::execute_transcript_analysis_stage_with_completion;
use super::progress::{
    is_run_cancelled, mark_run_cancelled, mark_run_running, update_run_progress,
};
use super::synthesis_execution::execute_synthesis_if_ready;
#[cfg(test)]
use super::synthesis_execution::{
    mark_synthesis_stage_failed_with_artifact, mark_synthesis_stage_skipped, synthesis_stage_id,
};
use super::tail_stages::mark_pending_mvp_tail_stages_skipped;
use super::transcript_execution::{
    load_pending_transcript_stage_rows, mark_transcript_stage_cancelled,
    mark_transcript_stage_failed, mark_transcript_stage_failed_for_attempt,
};
use super::{
    LlmCompletion, TranscriptAnalysisStageExecutionRequest, YoutubeSummaryRunExecutionOutcome,
    YoutubeSummaryStageExecutionError, YoutubeSummaryStageExecutionRequest,
};
use crate::error::{AppError, AppResult};
use crate::prompt_packs::json_repair::{
    execute_transcript_analysis_stage_repair_completion, insert_json_repair_input_artifact,
    JsonRepairStageExecutionRequest,
};
use crate::prompt_packs::result_builder::build_youtube_summary_canonical_result;
use crate::prompt_packs::stage_io::build_transcript_analysis_stage_input;
#[cfg(test)]
use crate::prompt_packs::stage_io::insert_stage_artifact_in_pool;

use super::result_validation::validate_and_persist_final_result_transaction;

#[cfg(test)]
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
    execute_stage: F,
) -> AppResult<YoutubeSummaryRunExecutionOutcome>
where
    F: FnMut(YoutubeSummaryStageExecutionRequest) -> Fut,
    Fut: Future<Output = Result<LlmCompletion, YoutubeSummaryStageExecutionError>>,
{
    execute_youtube_summary_run_with_stage_executor_internal(pool, run_id, execute_stage, |_| {})
        .await
}

#[cfg(test)]
pub(crate) async fn execute_youtube_summary_run_with_stage_executor_and_result_mutator<F, Fut, M>(
    pool: &SqlitePool,
    run_id: i64,
    execute_stage: F,
    mutate_final_result: M,
) -> AppResult<YoutubeSummaryRunExecutionOutcome>
where
    F: FnMut(YoutubeSummaryStageExecutionRequest) -> Fut,
    Fut: Future<Output = Result<LlmCompletion, YoutubeSummaryStageExecutionError>>,
    M: FnOnce(&mut serde_json::Value),
{
    execute_youtube_summary_run_with_stage_executor_internal(
        pool,
        run_id,
        execute_stage,
        mutate_final_result,
    )
    .await
}

async fn execute_youtube_summary_run_with_stage_executor_internal<F, Fut, M>(
    pool: &SqlitePool,
    run_id: i64,
    mut execute_stage: F,
    mutate_final_result: M,
) -> AppResult<YoutubeSummaryRunExecutionOutcome>
where
    F: FnMut(YoutubeSummaryStageExecutionRequest) -> Fut,
    Fut: Future<Output = Result<LlmCompletion, YoutubeSummaryStageExecutionError>>,
    M: FnOnce(&mut serde_json::Value),
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
                                        mark_transcript_stage_failed_for_attempt(
                                            pool,
                                            run_id,
                                            stage.stage_run_id,
                                            2,
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
                                mark_transcript_stage_failed_for_attempt(
                                    pool,
                                    run_id,
                                    stage.stage_run_id,
                                    2,
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

    if is_run_cancelled(pool, run_id).await? {
        mark_run_cancelled(pool, run_id, successes, total).await?;
        return Ok(cancelled_outcome(run_id, successes, total));
    }
    let synthesis_status =
        execute_synthesis_if_ready(pool, run_id, successes, total, &mut execute_stage).await?;
    if synthesis_status == "cancelled" {
        mark_run_cancelled(pool, run_id, successes, total + 1).await?;
        return Ok(cancelled_outcome(run_id, successes, total + 1));
    }
    mark_pending_mvp_tail_stages_skipped(pool, run_id).await?;
    if is_run_cancelled(pool, run_id).await? {
        let progress_total = if successes > 1 { total + 1 } else { total };
        mark_run_cancelled(pool, run_id, successes, progress_total).await?;
        return Ok(cancelled_outcome(run_id, successes, progress_total));
    }
    let terminal_status =
        terminal_status_for_synthesis(successes, failures, total, synthesis_status);
    let progress_total = if successes > 1 { total + 1 } else { total };
    let progress_current = if synthesis_status == "succeeded" {
        successes + 1
    } else {
        successes
    };
    let mut canonical = build_youtube_summary_canonical_result(pool, run_id).await?;
    mutate_final_result(&mut canonical);
    validate_and_persist_final_result_transaction(pool, run_id, canonical, terminal_status).await?;
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

fn now_string() -> String {
    time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}
