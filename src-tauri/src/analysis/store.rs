use sqlx::{Pool, Sqlite};

use super::models::{
    AnalysisPromptTemplate, AnalysisRunDetail, AnalysisRunRow, AnalysisRunSummary,
    AnalysisSourceGroup, AnalysisSourceGroupMember, AnalysisSourceGroupRow, CorpusMessage,
};
use super::{
    default_report_template_body, now_secs, ANALYSIS_RUN_TYPE_REPORT,
    ANALYSIS_SCOPE_TYPE_SOURCE_GROUP, ANALYSIS_STATUS_QUEUED, ANALYSIS_STATUS_RUNNING,
    DEFAULT_REPORT_TEMPLATE_NAME, TEMPLATE_KIND_REPORT,
};
use crate::compression::compress_text;

async fn builtin_report_template_exists(pool: &Pool<Sqlite>) -> Result<bool, String> {
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
    .map_err(|e| e.to_string())
}

pub(crate) async fn ensure_builtin_report_template(pool: &Pool<Sqlite>) -> Result<(), String> {
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
    .map_err(|e| e.to_string())?;

    Ok(())
}

pub(crate) async fn ensure_sources_exist(
    pool: &Pool<Sqlite>,
    source_ids: &[i64],
) -> Result<(), String> {
    for source_id in source_ids {
        let exists =
            sqlx::query_scalar::<_, i64>("SELECT EXISTS(SELECT 1 FROM sources WHERE id = ?)")
                .bind(source_id)
                .fetch_one(pool)
                .await
                .map_err(|e| e.to_string())?;

        if exists == 0 {
            return Err(format!("Source {source_id} not found"));
        }
    }

    Ok(())
}

fn resolve_run_scope_label_parts(
    scope_type: &str,
    source_id: Option<i64>,
    source_title: Option<&str>,
    source_group_id: Option<i64>,
    source_group_name: Option<&str>,
    scope_label_snapshot: Option<&str>,
) -> String {
    if let Some(label) = scope_label_snapshot {
        let trimmed = label.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    if scope_type == ANALYSIS_SCOPE_TYPE_SOURCE_GROUP {
        return source_group_name
            .map(ToString::to_string)
            .unwrap_or_else(|| format!("Group {}", source_group_id.unwrap_or_default()));
    }

    source_title
        .map(ToString::to_string)
        .unwrap_or_else(|| format!("Source {}", source_id.unwrap_or_default()))
}

fn resolve_run_row_scope_label(row: &AnalysisRunRow) -> String {
    resolve_run_scope_label_parts(
        &row.scope_type,
        row.source_id,
        row.source_title.as_deref(),
        row.source_group_id,
        row.source_group_name.as_deref(),
        row.scope_label_snapshot.as_deref(),
    )
}

pub(crate) fn map_run_summary(row: AnalysisRunRow) -> AnalysisRunSummary {
    let scope_label = resolve_run_row_scope_label(&row);

    AnalysisRunSummary {
        id: row.id,
        run_type: row.run_type,
        scope_type: row.scope_type,
        source_id: row.source_id,
        source_title: row.source_title,
        source_group_id: row.source_group_id,
        source_group_name: row.source_group_name,
        scope_label,
        period_from: row.period_from,
        period_to: row.period_to,
        output_language: row.output_language,
        prompt_template_id: row.prompt_template_id,
        prompt_template_name: row.prompt_template_name,
        prompt_template_version: row.prompt_template_version,
        provider_profile: row.provider_profile,
        provider: row.provider,
        model: row.model,
        status: row.status,
        error: row.error,
        has_trace_data: row.trace_data_zstd.is_some(),
        created_at: row.created_at,
        completed_at: row.completed_at,
    }
}

pub(crate) fn map_run_detail(row: AnalysisRunRow) -> AnalysisRunDetail {
    let scope_label = resolve_run_row_scope_label(&row);

    AnalysisRunDetail {
        id: row.id,
        run_type: row.run_type,
        scope_type: row.scope_type,
        source_id: row.source_id,
        source_title: row.source_title,
        source_group_id: row.source_group_id,
        source_group_name: row.source_group_name,
        scope_label,
        period_from: row.period_from,
        period_to: row.period_to,
        output_language: row.output_language,
        prompt_template_id: row.prompt_template_id,
        prompt_template_name: row.prompt_template_name,
        prompt_template_version: row.prompt_template_version,
        provider_profile: row.provider_profile,
        provider: row.provider,
        model: row.model,
        status: row.status,
        result_markdown: row.result_markdown,
        error: row.error,
        has_trace_data: row.trace_data_zstd.is_some(),
        created_at: row.created_at,
        completed_at: row.completed_at,
        scope_label_snapshot: row.scope_label_snapshot,
    }
}

pub(crate) async fn fetch_run_row(
    pool: &Pool<Sqlite>,
    run_id: i64,
) -> Result<Option<AnalysisRunRow>, String> {
    sqlx::query_as(
        r#"
        SELECT
            runs.id,
            runs.run_type,
            runs.scope_type,
            runs.source_id,
            sources.title AS source_title,
            runs.source_group_id,
            groups.name AS source_group_name,
            runs.period_from,
            runs.period_to,
            runs.output_language,
            runs.prompt_template_id,
            templates.name AS prompt_template_name,
            runs.prompt_template_version,
            runs.provider_profile,
            runs.provider,
            runs.model,
            runs.status,
            runs.result_markdown,
            runs.trace_data_zstd,
            runs.scope_label_snapshot,
            runs.error,
            runs.created_at,
            runs.completed_at
        FROM analysis_runs runs
        LEFT JOIN sources ON sources.id = runs.source_id
        LEFT JOIN analysis_source_groups groups ON groups.id = runs.source_group_id
        LEFT JOIN analysis_prompt_templates templates ON templates.id = runs.prompt_template_id
        WHERE runs.id = ?
        "#,
    )
    .bind(run_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())
}

pub(crate) async fn fetch_prompt_template(
    pool: &Pool<Sqlite>,
    template_id: i64,
) -> Result<AnalysisPromptTemplate, String> {
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
    .map_err(|e| e.to_string())?
    .ok_or_else(|| format!("Analysis prompt template {template_id} not found"))
}

pub(crate) async fn fetch_source_group(
    pool: &Pool<Sqlite>,
    group_id: i64,
) -> Result<Option<AnalysisSourceGroup>, String> {
    let group = sqlx::query_as::<_, AnalysisSourceGroupRow>(
        r#"
        SELECT id, name, created_at, updated_at
        FROM analysis_source_groups
        WHERE id = ?
        "#,
    )
    .bind(group_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

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
    .map_err(|e| e.to_string())?;

    Ok(Some(AnalysisSourceGroup {
        id: group.id,
        name: group.name,
        members,
        created_at: group.created_at,
        updated_at: group.updated_at,
    }))
}

pub(crate) fn resolve_run_scope_label(run: &AnalysisRunDetail) -> String {
    if !run.scope_label.trim().is_empty() {
        return run.scope_label.clone();
    }

    resolve_run_scope_label_parts(
        &run.scope_type,
        run.source_id,
        run.source_title.as_deref(),
        run.source_group_id,
        run.source_group_name.as_deref(),
        run.scope_label_snapshot.as_deref(),
    )
}

pub(crate) async fn find_active_duplicate_run(
    pool: &Pool<Sqlite>,
    scope_type: &str,
    source_id: Option<i64>,
    source_group_id: Option<i64>,
    period_from: i64,
    period_to: i64,
    output_language: &str,
    prompt_template_id: i64,
    provider_profile: &str,
    model: &str,
) -> Result<Option<i64>, String> {
    sqlx::query_scalar::<_, i64>(
        r#"
        SELECT id
        FROM analysis_runs
        WHERE run_type = ?
          AND scope_type = ?
          AND (source_id = ? OR (source_id IS NULL AND ? IS NULL))
          AND (source_group_id = ? OR (source_group_id IS NULL AND ? IS NULL))
          AND period_from = ?
          AND period_to = ?
          AND output_language = ?
          AND prompt_template_id = ?
          AND provider_profile = ?
          AND model = ?
          AND status IN (?, ?)
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(ANALYSIS_RUN_TYPE_REPORT)
    .bind(scope_type)
    .bind(source_id)
    .bind(source_id)
    .bind(source_group_id)
    .bind(source_group_id)
    .bind(period_from)
    .bind(period_to)
    .bind(output_language)
    .bind(prompt_template_id)
    .bind(provider_profile)
    .bind(model)
    .bind(ANALYSIS_STATUS_QUEUED)
    .bind(ANALYSIS_STATUS_RUNNING)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())
}

pub(crate) async fn insert_analysis_run(
    pool: &Pool<Sqlite>,
    scope_type: &str,
    source_id: Option<i64>,
    source_group_id: Option<i64>,
    period_from: i64,
    period_to: i64,
    output_language: &str,
    prompt_template: &AnalysisPromptTemplate,
    provider_profile: &str,
    provider: &str,
    model: &str,
) -> Result<i64, String> {
    let created_at = now_secs();
    sqlx::query_scalar(
        r#"
        INSERT INTO analysis_runs (
            run_type,
            scope_type,
            source_id,
            source_group_id,
            period_from,
            period_to,
            output_language,
            prompt_template_id,
            prompt_template_version,
            provider_profile,
            provider,
            model,
            status,
            scope_label_snapshot,
            created_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NULL, ?)
        RETURNING id
        "#,
    )
    .bind(ANALYSIS_RUN_TYPE_REPORT)
    .bind(scope_type)
    .bind(source_id)
    .bind(source_group_id)
    .bind(period_from)
    .bind(period_to)
    .bind(output_language)
    .bind(prompt_template.id)
    .bind(prompt_template.version)
    .bind(provider_profile)
    .bind(provider)
    .bind(model)
    .bind(ANALYSIS_STATUS_QUEUED)
    .bind(created_at)
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())
}

pub(crate) async fn persist_run_snapshot(
    pool: &Pool<Sqlite>,
    run_id: i64,
    scope_label: &str,
    corpus: &[CorpusMessage],
) -> Result<(), String> {
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    sqlx::query(
        r#"
        UPDATE analysis_runs
        SET scope_label_snapshot = ?
        WHERE id = ?
        "#,
    )
    .bind(scope_label)
    .bind(run_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query("DELETE FROM analysis_run_messages WHERE run_id = ?")
        .bind(run_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    for message in corpus {
        let content_zstd = compress_text(&message.content)?;
        sqlx::query(
            r#"
            INSERT INTO analysis_run_messages (
                run_id,
                item_id,
                source_id,
                external_id,
                author,
                published_at,
                ref,
                content_zstd
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(run_id)
        .bind(message.item_id)
        .bind(message.source_id)
        .bind(&message.external_id)
        .bind(&message.author)
        .bind(message.published_at)
        .bind(&message.r#ref)
        .bind(content_zstd)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    }

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) async fn set_run_status(
    pool: &Pool<Sqlite>,
    run_id: i64,
    status: &str,
    result_markdown: Option<&str>,
    trace_data_zstd: Option<&[u8]>,
    error: Option<&str>,
    completed_at: Option<i64>,
) -> Result<(), String> {
    sqlx::query(
        r#"
        UPDATE analysis_runs
        SET
            status = ?,
            result_markdown = COALESCE(?, result_markdown),
            trace_data_zstd = COALESCE(?, trace_data_zstd),
            error = ?,
            completed_at = ?
        WHERE id = ?
        "#,
    )
    .bind(status)
    .bind(result_markdown)
    .bind(trace_data_zstd)
    .bind(error)
    .bind(completed_at)
    .bind(run_id)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{map_run_detail, map_run_summary, resolve_run_scope_label};
    use crate::analysis::models::{AnalysisRunDetail, AnalysisRunRow};

    fn sample_run_row() -> AnalysisRunRow {
        AnalysisRunRow {
            id: 1,
            run_type: "report".to_string(),
            scope_type: "source_group".to_string(),
            source_id: None,
            source_title: None,
            source_group_id: Some(9),
            source_group_name: Some("Live group".to_string()),
            period_from: 1_700_000_000,
            period_to: 1_800_000_000,
            output_language: "English".to_string(),
            prompt_template_id: Some(1),
            prompt_template_name: Some("Default".to_string()),
            prompt_template_version: 1,
            provider_profile: "default".to_string(),
            provider: "gemini".to_string(),
            model: "gemini-2.5-flash".to_string(),
            status: "completed".to_string(),
            result_markdown: Some("Saved report".to_string()),
            trace_data_zstd: Some(vec![1, 2, 3]),
            scope_label_snapshot: Some("Frozen group".to_string()),
            error: None,
            created_at: 1_710_000_500,
            completed_at: Some(1_710_000_600),
        }
    }

    fn sample_run() -> AnalysisRunDetail {
        map_run_detail(sample_run_row())
    }

    #[test]
    fn resolve_run_scope_label_prefers_frozen_value() {
        let run = sample_run();
        assert_eq!(resolve_run_scope_label(&run), "Frozen group");
    }

    #[test]
    fn map_run_summary_exposes_frozen_scope_label() {
        let summary = map_run_summary(sample_run_row());
        assert_eq!(summary.scope_label, "Frozen group");
    }
}
