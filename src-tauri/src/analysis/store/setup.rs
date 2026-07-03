use sqlx::{Pool, Sqlite};

use super::super::models::{
    AnalysisPromptTemplate, AnalysisSourceGroup, AnalysisSourceGroupMember, AnalysisSourceGroupRow,
};
use super::super::{
    default_report_template_body, now_secs, DEFAULT_REPORT_TEMPLATE_NAME, TEMPLATE_KIND_REPORT,
};
use crate::error::{AppError, AppResult};

async fn builtin_report_template_exists(pool: &Pool<Sqlite>) -> AppResult<bool> {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM analysis_prompt_templates
            WHERE is_builtin = 1 AND template_kind = ?
        )
        "#,
    )
    .bind(TEMPLATE_KIND_REPORT)
    .fetch_one(pool)
    .await
    .map(|exists| exists != 0)
    .map_err(AppError::database)
}

pub(crate) async fn ensure_builtin_report_template(pool: &Pool<Sqlite>) -> AppResult<()> {
    if builtin_report_template_exists(pool).await? {
        return Ok(());
    }

    let now = now_secs();
    sqlx::query(
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
        VALUES (?, ?, ?, 1, 1, ?, ?)
        "#,
    )
    .bind(DEFAULT_REPORT_TEMPLATE_NAME)
    .bind(TEMPLATE_KIND_REPORT)
    .bind(default_report_template_body())
    .bind(now)
    .bind(now)
    .execute(pool)
    .await
    .map_err(AppError::database)?;

    Ok(())
}

pub(crate) async fn ensure_sources_exist(pool: &Pool<Sqlite>, source_ids: &[i64]) -> AppResult<()> {
    for source_id in source_ids {
        let exists =
            sqlx::query_scalar::<_, i64>("SELECT EXISTS(SELECT 1 FROM sources WHERE id = ?)")
                .bind(source_id)
                .fetch_one(pool)
                .await
                .map_err(AppError::database)?;

        if exists == 0 {
            return Err(AppError::not_found(format!("Source {source_id} not found")));
        }
    }

    Ok(())
}

pub(crate) async fn fetch_prompt_template(
    pool: &Pool<Sqlite>,
    template_id: i64,
) -> AppResult<AnalysisPromptTemplate> {
    ensure_builtin_report_template(pool).await?;

    sqlx::query_as(
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
    .ok_or_else(|| AppError::not_found(format!("Analysis prompt template {template_id} not found")))
}

pub(crate) async fn fetch_source_group(
    pool: &Pool<Sqlite>,
    group_id: i64,
) -> AppResult<Option<AnalysisSourceGroup>> {
    let group = sqlx::query_as::<_, AnalysisSourceGroupRow>(
        r#"
        SELECT id, name, source_type, created_at, updated_at
        FROM analysis_source_groups
        WHERE id = ?
        "#,
    )
    .bind(group_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)?;

    let Some(group) = group else {
        return Ok(None);
    };

    let members = sqlx::query_as::<_, AnalysisSourceGroupMember>(
        r#"
        SELECT
            sources.id AS source_id,
            sources.title AS source_title,
            COUNT(items.content_zstd) AS item_count
        FROM analysis_source_group_members members
        JOIN sources ON sources.id = members.source_id
        LEFT JOIN items ON items.source_id = sources.id
        WHERE members.group_id = ?
        GROUP BY sources.id, sources.title
        ORDER BY COALESCE(sources.title, ''), sources.id
        "#,
    )
    .bind(group_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    Ok(Some(AnalysisSourceGroup {
        id: group.id,
        name: group.name,
        source_type: group.source_type,
        members,
        created_at: group.created_at,
        updated_at: group.updated_at,
    }))
}
