use sqlx::SqlitePool;

use super::entities::{
    build_or_quarantine_intermediate_entities_for_transcript_stage,
    insert_intermediate_entities_artifact_in_transaction,
    load_required_allowed_refs_for_live_synthesis,
};
use super::synthesis_input::build_synthesis_stage_input;
use super::LlmCompletion;
use crate::error::{AppError, AppResult};
use crate::prompt_packs::stage_io::{
    build_transcript_analysis_stage_input, extract_json_payload, insert_stage_artifact_in_pool,
    insert_stage_artifact_in_transaction, SYNTHESIS_OUTPUT_SCHEMA_ID,
    TRANSCRIPT_ANALYSIS_OUTPUT_SCHEMA_ID,
};
use crate::prompt_packs::validation::{
    quarantine_prompt_pack_validation_error, validate_and_quarantine_synthesis_output,
    validate_synthesis_output_with_allowed_refs, validate_transcript_analysis_output,
};

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
    validate_transcript_analysis_output(&input, &parsed)
        .map_err(|error| AppError::validation(error.message))?;
    let intermediate_graph = build_or_quarantine_intermediate_entities_for_transcript_stage(
        pool,
        run_id,
        stage_run_id,
        &parsed,
        1,
    )
    .await?;
    let metrics = serde_json::json!({
        "input_tokens": completion.input_tokens,
        "output_tokens": completion.output_tokens,
        "latency_ms": completion.latency_ms,
        "schema_id": TRANSCRIPT_ANALYSIS_OUTPUT_SCHEMA_ID,
        "validation_error_count": 0,
        "attempt_number": 1
    });
    let parsed_json = serde_json::to_string(&parsed)
        .map_err(|error| AppError::internal(format!("serialize parsed output: {error}")))?;
    let mut tx = pool.begin().await.map_err(AppError::database)?;
    insert_stage_artifact_in_transaction(
        &mut tx,
        run_id,
        stage_run_id,
        "metrics",
        1,
        4,
        &metrics.to_string(),
    )
    .await?;
    insert_intermediate_entities_artifact_in_transaction(
        &mut tx,
        run_id,
        stage_run_id,
        &intermediate_graph,
        1,
    )
    .await?;
    insert_stage_artifact_in_transaction(
        &mut tx,
        run_id,
        stage_run_id,
        "parsed_output",
        1,
        3,
        &parsed_json,
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
    .execute(&mut *tx)
    .await
    .map_err(AppError::database)?;
    tx.commit().await.map_err(AppError::database)?;
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
    let allowed_refs = match load_required_allowed_refs_for_live_synthesis(pool, run_id).await {
        Ok(allowed_refs) => allowed_refs,
        Err(error) => {
            mark_synthesis_stage_failed(pool, stage_run_id, &error.message).await?;
            return Err(error);
        }
    };
    if let Err(error) = validate_synthesis_output_with_allowed_refs(
        &parsed,
        &allowed_refs.source_refs,
        &allowed_refs.claim_refs,
        &allowed_refs.evidence_refs,
    ) {
        let validation_message = error.message.clone();
        quarantine_prompt_pack_validation_error(pool, run_id, stage_run_id, &parsed, error).await?;
        mark_synthesis_stage_failed(pool, stage_run_id, &validation_message).await?;
        return Err(AppError::validation(validation_message));
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
        "schema_id": SYNTHESIS_OUTPUT_SCHEMA_ID,
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

pub(crate) async fn mark_synthesis_stage_failed(
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

fn now_string() -> String {
    crate::time::now_rfc3339_utc()
}
