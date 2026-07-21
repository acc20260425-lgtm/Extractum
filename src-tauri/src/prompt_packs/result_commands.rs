use tauri::AppHandle;

use crate::db::get_pool;
use crate::error::AppResult;
use extractum_prompt_packs::{
    get_prompt_pack_result_in_pool, get_prompt_pack_stage_artifact_in_pool,
    get_prompt_pack_validation_findings_in_pool, list_prompt_pack_audit_events_in_pool,
    list_prompt_pack_stage_artifacts_in_pool, PromptPackAuditEventDto, PromptPackResultDto,
    PromptPackStageArtifactDto, PromptPackStageArtifactSummaryDto, PromptPackValidationFindingDto,
};

#[tauri::command]
pub async fn get_prompt_pack_result(
    handle: AppHandle,
    run_id: i64,
) -> AppResult<PromptPackResultDto> {
    let pool = get_pool(&handle).await?;
    get_prompt_pack_result_in_pool(&pool, run_id).await
}

#[tauri::command]
pub async fn list_prompt_pack_stage_artifacts(
    handle: AppHandle,
    stage_run_id: i64,
) -> AppResult<Vec<PromptPackStageArtifactSummaryDto>> {
    let pool = get_pool(&handle).await?;
    list_prompt_pack_stage_artifacts_in_pool(&pool, stage_run_id).await
}

#[tauri::command]
pub async fn get_prompt_pack_stage_artifact(
    handle: AppHandle,
    stage_run_id: i64,
    artifact_kind: String,
    attempt_number: i64,
    artifact_index: i64,
) -> AppResult<PromptPackStageArtifactDto> {
    let pool = get_pool(&handle).await?;
    get_prompt_pack_stage_artifact_in_pool(
        &pool,
        stage_run_id,
        artifact_kind,
        attempt_number,
        artifact_index,
    )
    .await
}

#[tauri::command]
pub async fn get_prompt_pack_validation_findings(
    handle: AppHandle,
    run_id: i64,
) -> AppResult<Vec<PromptPackValidationFindingDto>> {
    let pool = get_pool(&handle).await?;
    get_prompt_pack_validation_findings_in_pool(&pool, run_id).await
}

#[tauri::command]
pub async fn list_prompt_pack_audit_events(
    handle: AppHandle,
    run_id: i64,
) -> AppResult<Vec<PromptPackAuditEventDto>> {
    let pool = get_pool(&handle).await?;
    list_prompt_pack_audit_events_in_pool(&pool, run_id).await
}
