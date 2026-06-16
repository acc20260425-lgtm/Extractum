use sqlx::SqlitePool;

use super::stage_io::{
    build_transcript_analysis_stage_input, extract_json_payload, insert_stage_artifact_in_pool,
    SYNTHESIS_OUTPUT_SCHEMA_ID, TRANSCRIPT_ANALYSIS_OUTPUT_SCHEMA_ID,
};
use super::validation::{
    validate_and_quarantine_synthesis_output, validate_transcript_analysis_output,
};
use super::youtube_summary::entities::{
    build_or_quarantine_intermediate_entities_for_transcript_stage,
    insert_intermediate_entities_artifact,
};
use super::youtube_summary::LlmCompletion;
use crate::error::{AppError, AppResult};

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

pub(crate) async fn insert_json_repair_input_artifact(
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

pub(crate) async fn execute_transcript_analysis_stage_repair_completion(
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
    let intermediate_graph = build_or_quarantine_intermediate_entities_for_transcript_stage(
        pool,
        run_id,
        stage_run_id,
        &parsed,
        attempt_number,
    )
    .await?;
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
        "schema_id": TRANSCRIPT_ANALYSIS_OUTPUT_SCHEMA_ID,
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
    insert_intermediate_entities_artifact(
        pool,
        run_id,
        stage_run_id,
        &intermediate_graph,
        attempt_number,
    )
    .await?;
    mark_stage_repaired(pool, stage_run_id).await
}

pub(crate) async fn execute_synthesis_stage_repair_completion(
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
        "schema_id": SYNTHESIS_OUTPUT_SCHEMA_ID,
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
    mark_stage_repaired(pool, stage_run_id).await
}

async fn mark_stage_repaired(pool: &SqlitePool, stage_run_id: i64) -> AppResult<()> {
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

fn now_string() -> String {
    crate::time::now_rfc3339_utc()
}
