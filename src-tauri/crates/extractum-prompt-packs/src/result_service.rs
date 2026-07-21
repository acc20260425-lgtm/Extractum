use extractum_core::compression::decompress_text;
use extractum_core::error::{AppError, AppResult};
use sqlx::SqlitePool;

use super::dto::{
    PromptPackAuditEventDto, PromptPackResultDto, PromptPackStageArtifactDto,
    PromptPackStageArtifactSummaryDto, PromptPackValidationFindingDto,
};

pub async fn get_prompt_pack_result_in_pool(
    pool: &SqlitePool,
    run_id: i64,
) -> AppResult<PromptPackResultDto> {
    let (result_status, canonical_json_zstd, storage_warning): (String, Vec<u8>, Option<String>) =
        sqlx::query_as(
            "SELECT result_status, canonical_json_zstd, storage_warning
             FROM prompt_pack_results
             WHERE run_id = ?",
        )
        .bind(run_id)
        .fetch_one(pool)
        .await
        .map_err(AppError::database)?;
    let canonical_text = decompress_text(&canonical_json_zstd).map_err(AppError::internal)?;
    let canonical = serde_json::from_str(&canonical_text)
        .map_err(|error| AppError::internal(format!("parse canonical result: {error}")))?;
    Ok(PromptPackResultDto {
        run_id,
        result_status,
        canonical,
        storage_warning,
    })
}

pub async fn list_prompt_pack_stage_artifacts_in_pool(
    pool: &SqlitePool,
    stage_run_id: i64,
) -> AppResult<Vec<PromptPackStageArtifactSummaryDto>> {
    sqlx::query_as::<_, (String, i64, i64, String, String, String)>(
        "SELECT artifact_kind, attempt_number, artifact_index, content_type, content_hash, created_at
         FROM prompt_pack_stage_artifacts
         WHERE stage_run_id = ?
         ORDER BY attempt_number ASC, artifact_index ASC",
    )
    .bind(stage_run_id)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(
                |(
                    artifact_kind,
                    attempt_number,
                    artifact_index,
                    content_type,
                    content_hash,
                    created_at,
                )| PromptPackStageArtifactSummaryDto {
                    stage_run_id,
                    artifact_kind,
                    attempt_number,
                    artifact_index,
                    content_type,
                    content_hash,
                    created_at,
                },
            )
            .collect()
    })
    .map_err(AppError::database)
}

pub async fn get_prompt_pack_stage_artifact_in_pool(
    pool: &SqlitePool,
    stage_run_id: i64,
    artifact_kind: String,
    attempt_number: i64,
    artifact_index: i64,
) -> AppResult<PromptPackStageArtifactDto> {
    let (content_type, content_zstd, created_at): (String, Vec<u8>, String) = sqlx::query_as(
        "SELECT content_type, content_zstd, created_at
         FROM prompt_pack_stage_artifacts
         WHERE stage_run_id = ?
           AND artifact_kind = ?
           AND attempt_number = ?
           AND artifact_index = ?",
    )
    .bind(stage_run_id)
    .bind(&artifact_kind)
    .bind(attempt_number)
    .bind(artifact_index)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)?;
    let content_text = decompress_text(&content_zstd).map_err(AppError::internal)?;
    let content = serde_json::from_str(&content_text)
        .unwrap_or_else(|_| serde_json::json!({ "text": content_text }));
    Ok(PromptPackStageArtifactDto {
        stage_run_id,
        artifact_kind,
        attempt_number,
        artifact_index,
        content_type,
        content,
        created_at,
    })
}

pub async fn get_prompt_pack_validation_findings_in_pool(
    pool: &SqlitePool,
    run_id: i64,
) -> AppResult<Vec<PromptPackValidationFindingDto>> {
    sqlx::query_as::<_, (Option<i64>, String, String, String, Option<String>, String)>(
        "SELECT stage_run_id, severity, code, message, object_path, created_at
         FROM prompt_pack_result_validation_findings
         WHERE run_id = ?
         ORDER BY id ASC",
    )
    .bind(run_id)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(
                |(stage_run_id, severity, code, message, object_path, created_at)| {
                    PromptPackValidationFindingDto {
                        run_id,
                        stage_run_id,
                        severity,
                        code,
                        message,
                        object_path,
                        created_at,
                    }
                },
            )
            .collect()
    })
    .map_err(AppError::database)
}

pub async fn list_prompt_pack_audit_events_in_pool(
    pool: &SqlitePool,
    run_id: i64,
) -> AppResult<Vec<PromptPackAuditEventDto>> {
    sqlx::query_as::<_, (String, Option<String>, Option<Vec<u8>>, String)>(
        "SELECT event_kind, message, payload_json_zstd, created_at
         FROM prompt_pack_audit_events
         WHERE run_id = ?
         ORDER BY id ASC",
    )
    .bind(run_id)
    .fetch_all(pool)
    .await
    .map(|rows| {
        rows.into_iter()
            .map(|(event_kind, message, payload_json_zstd, created_at)| {
                let payload = payload_json_zstd
                    .and_then(|bytes| decompress_text(&bytes).ok())
                    .and_then(|text| serde_json::from_str(&text).ok());
                PromptPackAuditEventDto {
                    run_id,
                    event_kind,
                    message,
                    payload,
                    created_at,
                }
            })
            .collect()
    })
    .map_err(AppError::database)
}
