use extractum_analysis::{
    create_analysis_prompt_template as create_analysis_prompt_template_in_pool,
    delete_analysis_prompt_template as delete_analysis_prompt_template_in_pool,
    list_analysis_prompt_templates as list_analysis_prompt_templates_in_pool,
    update_analysis_prompt_template as update_analysis_prompt_template_in_pool,
    AnalysisPromptTemplate,
};
use extractum_core::error::AppResult;
use tauri::AppHandle;

use crate::db::get_pool;

#[tauri::command]
pub async fn list_analysis_prompt_templates(
    handle: AppHandle,
    template_kind: Option<String>,
) -> AppResult<Vec<AnalysisPromptTemplate>> {
    let pool = get_pool(&handle).await?;
    list_analysis_prompt_templates_in_pool(&pool, template_kind).await
}

#[tauri::command]
pub async fn create_analysis_prompt_template(
    handle: AppHandle,
    name: String,
    template_kind: String,
    body: String,
) -> AppResult<AnalysisPromptTemplate> {
    let pool = get_pool(&handle).await?;
    create_analysis_prompt_template_in_pool(&pool, name, template_kind, body).await
}

#[tauri::command]
pub async fn update_analysis_prompt_template(
    handle: AppHandle,
    template_id: i64,
    name: String,
    body: String,
) -> AppResult<AnalysisPromptTemplate> {
    let pool = get_pool(&handle).await?;
    update_analysis_prompt_template_in_pool(&pool, template_id, name, body).await
}

#[tauri::command]
pub async fn delete_analysis_prompt_template(handle: AppHandle, template_id: i64) -> AppResult<()> {
    let pool = get_pool(&handle).await?;
    delete_analysis_prompt_template_in_pool(&pool, template_id).await
}
