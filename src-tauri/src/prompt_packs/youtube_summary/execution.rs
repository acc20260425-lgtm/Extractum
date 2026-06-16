use std::future::Future;

use sqlx::SqlitePool;

use super::outputs::{
    execute_synthesis_stage_with_completion, execute_transcript_analysis_stage_with_completion,
    mark_synthesis_stage_failed,
};
use super::progress::{
    is_run_cancelled, mark_run_cancelled, mark_run_running, update_run_progress,
};
use super::synthesis_input::build_synthesis_stage_input;
use super::transcript_execution::{
    load_pending_transcript_stage_rows, mark_transcript_stage_cancelled,
    mark_transcript_stage_failed,
};
use super::{
    LlmCompletion, SynthesisStageExecutionRequest, TranscriptAnalysisStageExecutionRequest,
    YoutubeSummaryRunExecutionOutcome, YoutubeSummaryStageExecutionError,
    YoutubeSummaryStageExecutionRequest, SYNTHESIS_STAGE_NAME,
};
#[cfg(test)]
use crate::compression::compress_text;
use crate::error::{AppError, AppResult};
use crate::prompt_packs::json_repair::{
    execute_synthesis_stage_repair_completion, execute_transcript_analysis_stage_repair_completion,
    insert_json_repair_input_artifact, JsonRepairStageExecutionRequest,
};
use crate::prompt_packs::stage_io::{
    build_transcript_analysis_stage_input, insert_stage_artifact_in_pool,
};
use crate::prompt_packs::{
    projections::persist_final_result_transaction,
    result_builder::build_youtube_summary_canonical_result,
};

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

#[cfg(test)]
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

fn now_string() -> String {
    time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

#[cfg(test)]
fn simple_hash(value: &str) -> String {
    use sha2::{Digest, Sha384};
    Sha384::digest(value.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
