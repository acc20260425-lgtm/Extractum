use tauri::AppHandle;

use crate::db::get_pool;

use super::models::AnalysisPromptTemplate;
use super::store::ensure_builtin_report_template;
use super::{now_secs, TEMPLATE_KIND_CHAT, TEMPLATE_KIND_REPORT};

pub(crate) fn validate_template_kind(template_kind: &str) -> Result<String, String> {
    let normalized = template_kind.trim().to_ascii_lowercase();
    match normalized.as_str() {
        TEMPLATE_KIND_REPORT | TEMPLATE_KIND_CHAT => Ok(normalized),
        _ => Err(format!("Unsupported template kind '{template_kind}'")),
    }
}

fn validate_template_input(
    name: &str,
    template_kind: &str,
    body: &str,
) -> Result<(String, String, String), String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("Template name cannot be empty".to_string());
    }

    let template_kind = validate_template_kind(template_kind)?;

    let body = body.trim().to_string();
    if body.is_empty() {
        return Err("Template body cannot be empty".to_string());
    }

    Ok((name, template_kind, body))
}

#[tauri::command]
pub async fn list_analysis_prompt_templates(
    handle: AppHandle,
    template_kind: Option<String>,
) -> Result<Vec<AnalysisPromptTemplate>, String> {
    let pool = get_pool(&handle).await?;
    ensure_builtin_report_template(&pool).await?;

    if let Some(template_kind) = template_kind {
        let template_kind = validate_template_kind(&template_kind)?;
        sqlx::query_as::<_, AnalysisPromptTemplate>(
            r#"
            SELECT id, name, template_kind, body, version, is_builtin, created_at, updated_at
            FROM analysis_prompt_templates
            WHERE template_kind = ?
            ORDER BY is_builtin DESC, updated_at DESC, id DESC
            "#,
        )
        .bind(template_kind)
        .fetch_all(&pool)
        .await
        .map_err(|e| e.to_string())
    } else {
        sqlx::query_as::<_, AnalysisPromptTemplate>(
            r#"
            SELECT id, name, template_kind, body, version, is_builtin, created_at, updated_at
            FROM analysis_prompt_templates
            ORDER BY template_kind ASC, is_builtin DESC, updated_at DESC, id DESC
            "#,
        )
        .fetch_all(&pool)
        .await
        .map_err(|e| e.to_string())
    }
}

#[tauri::command]
pub async fn create_analysis_prompt_template(
    handle: AppHandle,
    name: String,
    template_kind: String,
    body: String,
) -> Result<AnalysisPromptTemplate, String> {
    let pool = get_pool(&handle).await?;
    let (name, template_kind, body) = validate_template_input(&name, &template_kind, &body)?;
    let now = now_secs();

    sqlx::query_as::<_, AnalysisPromptTemplate>(
        r#"
        INSERT INTO analysis_prompt_templates (
            name,
            template_kind,
            body,
            version,
            is_builtin,
            created_at,
            updated_at
        )
        VALUES (?, ?, ?, 1, 0, ?, ?)
        RETURNING id, name, template_kind, body, version, is_builtin, created_at, updated_at
        "#,
    )
    .bind(name)
    .bind(template_kind)
    .bind(body)
    .bind(now)
    .bind(now)
    .fetch_one(&pool)
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_analysis_prompt_template(
    handle: AppHandle,
    template_id: i64,
    name: String,
    body: String,
) -> Result<AnalysisPromptTemplate, String> {
    let pool = get_pool(&handle).await?;
    let existing: AnalysisPromptTemplate = sqlx::query_as::<_, AnalysisPromptTemplate>(
        r#"
        SELECT id, name, template_kind, body, version, is_builtin, created_at, updated_at
        FROM analysis_prompt_templates
        WHERE id = ?
        "#,
    )
    .bind(template_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| format!("Analysis prompt template {template_id} not found"))?;

    if existing.is_builtin {
        return Err("Built-in templates cannot be edited directly".to_string());
    }

    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("Template name cannot be empty".to_string());
    }

    let body = body.trim().to_string();
    if body.is_empty() {
        return Err("Template body cannot be empty".to_string());
    }

    let now = now_secs();
    sqlx::query_as::<_, AnalysisPromptTemplate>(
        r#"
        UPDATE analysis_prompt_templates
        SET
            name = ?,
            body = ?,
            version = version + 1,
            updated_at = ?
        WHERE id = ?
        RETURNING id, name, template_kind, body, version, is_builtin, created_at, updated_at
        "#,
    )
    .bind(name)
    .bind(body)
    .bind(now)
    .bind(template_id)
    .fetch_one(&pool)
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_analysis_prompt_template(
    handle: AppHandle,
    template_id: i64,
) -> Result<(), String> {
    let pool = get_pool(&handle).await?;
    let template: Option<(i64, bool)> = sqlx::query_as(
        "SELECT id, is_builtin FROM analysis_prompt_templates WHERE id = ?",
    )
    .bind(template_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let Some((_, is_builtin)) = template else {
        return Err(format!("Analysis prompt template {template_id} not found"));
    };

    if is_builtin {
        return Err("Built-in templates cannot be deleted".to_string());
    }

    sqlx::query("DELETE FROM analysis_prompt_templates WHERE id = ?")
        .bind(template_id)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}
