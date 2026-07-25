use sqlx::SqlitePool;

use super::domain::{now_secs, TEMPLATE_KIND_CHAT, TEMPLATE_KIND_REPORT};
use super::models::AnalysisPromptTemplate;
use super::store::ensure_builtin_report_template;
use extractum_core::error::{AppError, AppResult};

pub(crate) fn validate_template_kind(template_kind: &str) -> AppResult<String> {
    let normalized = template_kind.trim().to_ascii_lowercase();
    match normalized.as_str() {
        TEMPLATE_KIND_REPORT | TEMPLATE_KIND_CHAT => Ok(normalized),
        _ => Err(AppError::validation(format!(
            "Unsupported template kind '{template_kind}'"
        ))),
    }
}

fn validate_template_input(
    name: &str,
    template_kind: &str,
    body: &str,
) -> AppResult<(String, String, String)> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::validation("Template name cannot be empty"));
    }

    let template_kind = validate_template_kind(template_kind)?;
    let body = body.trim().to_string();
    if body.is_empty() {
        return Err(AppError::validation("Template body cannot be empty"));
    }

    Ok((name, template_kind, body))
}

pub async fn list_analysis_prompt_templates_in_pool(
    pool: &SqlitePool,
    template_kind: Option<String>,
) -> AppResult<Vec<AnalysisPromptTemplate>> {
    ensure_builtin_report_template(pool).await?;

    if let Some(template_kind) = template_kind {
        let template_kind = validate_template_kind(&template_kind)?;
        sqlx::query_as(
            r#"
            SELECT id, name, template_kind, body, version, is_builtin, created_at, updated_at
            FROM analysis_prompt_templates
            WHERE template_kind = ?
            ORDER BY is_builtin DESC, updated_at DESC, id DESC
            "#,
        )
        .bind(template_kind)
        .fetch_all(pool)
        .await
        .map_err(AppError::database)
    } else {
        sqlx::query_as(
            r#"
            SELECT id, name, template_kind, body, version, is_builtin, created_at, updated_at
            FROM analysis_prompt_templates
            ORDER BY template_kind ASC, is_builtin DESC, updated_at DESC, id DESC
            "#,
        )
        .fetch_all(pool)
        .await
        .map_err(AppError::database)
    }
}

pub async fn create_analysis_prompt_template_in_pool(
    pool: &SqlitePool,
    name: String,
    template_kind: String,
    body: String,
) -> AppResult<AnalysisPromptTemplate> {
    let (name, template_kind, body) = validate_template_input(&name, &template_kind, &body)?;
    let now = now_secs();

    sqlx::query_as(
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
    .fetch_one(pool)
    .await
    .map_err(AppError::database)
}

pub async fn update_analysis_prompt_template_in_pool(
    pool: &SqlitePool,
    template_id: i64,
    name: String,
    body: String,
) -> AppResult<AnalysisPromptTemplate> {
    let existing: AnalysisPromptTemplate = sqlx::query_as(
        r#"
        SELECT id, name, template_kind, body, version, is_builtin, created_at, updated_at
        FROM analysis_prompt_templates
        WHERE id = ?
        "#,
    )
    .bind(template_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)?
    .ok_or_else(|| {
        AppError::not_found(format!("Analysis prompt template {template_id} not found"))
    })?;

    if existing.is_builtin {
        return Err(AppError::conflict(
            "Built-in templates cannot be edited directly",
        ));
    }

    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::validation("Template name cannot be empty"));
    }
    let body = body.trim().to_string();
    if body.is_empty() {
        return Err(AppError::validation("Template body cannot be empty"));
    }

    let now = now_secs();
    sqlx::query_as(
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
    .fetch_one(pool)
    .await
    .map_err(AppError::database)
}

pub async fn delete_analysis_prompt_template_in_pool(
    pool: &SqlitePool,
    template_id: i64,
) -> AppResult<()> {
    let template: Option<(i64, bool)> =
        sqlx::query_as("SELECT id, is_builtin FROM analysis_prompt_templates WHERE id = ?")
            .bind(template_id)
            .fetch_optional(pool)
            .await
            .map_err(AppError::database)?;

    let Some((_, is_builtin)) = template else {
        return Err(AppError::not_found(format!(
            "Analysis prompt template {template_id} not found"
        )));
    };
    if is_builtin {
        return Err(AppError::conflict("Built-in templates cannot be deleted"));
    }

    sqlx::query("DELETE FROM analysis_prompt_templates WHERE id = ?")
        .bind(template_id)
        .execute(pool)
        .await
        .map_err(AppError::database)?;
    Ok(())
}
