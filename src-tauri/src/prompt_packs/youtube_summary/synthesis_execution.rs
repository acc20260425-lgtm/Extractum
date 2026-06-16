use std::future::Future;

use sqlx::SqlitePool;

use super::outputs::{execute_synthesis_stage_with_completion, mark_synthesis_stage_failed};
use super::progress::update_run_progress;
use super::synthesis_input::build_synthesis_stage_input;
use super::{
    LlmCompletion, SynthesisStageExecutionRequest, YoutubeSummaryStageExecutionError,
    YoutubeSummaryStageExecutionRequest, SYNTHESIS_STAGE_NAME,
};
use crate::error::{AppError, AppResult};
use crate::prompt_packs::json_repair::{
    execute_synthesis_stage_repair_completion, insert_json_repair_input_artifact,
    JsonRepairStageExecutionRequest,
};
use crate::prompt_packs::stage_io::insert_stage_artifact_in_pool;

pub(crate) async fn synthesis_stage_id(pool: &SqlitePool, run_id: i64) -> AppResult<i64> {
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

pub(crate) async fn mark_synthesis_stage_skipped(pool: &SqlitePool, run_id: i64) -> AppResult<()> {
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
    .bind(crate::time::now_rfc3339_utc())
    .bind(crate::time::now_rfc3339_utc())
    .bind(run_id)
    .bind(SYNTHESIS_STAGE_NAME)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

pub(crate) async fn mark_synthesis_stage_failed_with_artifact(
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

pub(crate) async fn execute_synthesis_if_ready<F, Fut>(
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
