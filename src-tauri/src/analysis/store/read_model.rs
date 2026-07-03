use sqlx::{Pool, QueryBuilder, Sqlite};

use super::super::models::{
    AnalysisRunDetail, AnalysisRunRow, AnalysisRunSummary, AnalysisSnapshotState,
};
use super::super::{
    ANALYSIS_SCOPE_TYPE_PROJECT, ANALYSIS_SCOPE_TYPE_SOURCE_GROUP, ANALYSIS_STATUS_CANCELLED,
    ANALYSIS_STATUS_COMPLETED, ANALYSIS_STATUS_FAILED, ANALYSIS_STATUS_QUEUED,
    ANALYSIS_STATUS_RUNNING,
};
use crate::error::{AppError, AppResult};
use crate::time::ymd_to_unix_midnight;

fn resolve_run_scope_label_parts(
    scope_type: &str,
    source_id: Option<i64>,
    source_title: Option<&str>,
    source_group_id: Option<i64>,
    source_group_name: Option<&str>,
    project_id: Option<i64>,
    project_name: Option<&str>,
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

    if scope_type == ANALYSIS_SCOPE_TYPE_PROJECT {
        return project_name
            .filter(|value| !value.trim().is_empty())
            .map(ToString::to_string)
            .unwrap_or_else(|| format!("Project {}", project_id.unwrap_or_default()));
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
        row.project_id,
        row.project_name.as_deref(),
        row.scope_label_snapshot.as_deref(),
    )
}

fn compute_snapshot_state(row: &AnalysisRunRow) -> Option<AnalysisSnapshotState> {
    if row.snapshot_captured_at.is_some() && row.snapshot_error.is_none() {
        return Some(AnalysisSnapshotState::Captured);
    }

    if row.snapshot_error.is_some() {
        return Some(AnalysisSnapshotState::CaptureFailed);
    }

    match row.status.as_str() {
        ANALYSIS_STATUS_COMPLETED | ANALYSIS_STATUS_FAILED | ANALYSIS_STATUS_CANCELLED => {
            Some(AnalysisSnapshotState::CaptureFailed)
        }
        _ => None,
    }
}

pub(crate) fn map_run_summary(row: AnalysisRunRow) -> AnalysisRunSummary {
    let scope_label = resolve_run_row_scope_label(&row);
    let snapshot_state = compute_snapshot_state(&row);

    AnalysisRunSummary {
        id: row.id,
        run_type: row.run_type,
        scope_type: row.scope_type,
        source_id: row.source_id,
        source_title: row.source_title,
        source_group_id: row.source_group_id,
        source_group_name: row.source_group_name,
        project_id: row.project_id,
        project_name: row.project_name,
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
        youtube_corpus_mode: row.youtube_corpus_mode,
        telegram_history_scope: row.telegram_history_scope,
        status: row.status,
        error: row.error,
        has_trace_data: row.trace_data_zstd.is_some(),
        snapshot_state,
        snapshot_captured_at: row.snapshot_captured_at,
        snapshot_error: row.snapshot_error,
        created_at: row.created_at,
        completed_at: row.completed_at,
    }
}

pub(crate) fn map_run_detail(row: AnalysisRunRow) -> AnalysisRunDetail {
    let scope_label = resolve_run_row_scope_label(&row);
    let snapshot_state = compute_snapshot_state(&row);

    AnalysisRunDetail {
        id: row.id,
        run_type: row.run_type,
        scope_type: row.scope_type,
        source_id: row.source_id,
        source_title: row.source_title,
        source_group_id: row.source_group_id,
        source_group_name: row.source_group_name,
        project_id: row.project_id,
        project_name: row.project_name,
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
        youtube_corpus_mode: row.youtube_corpus_mode,
        telegram_history_scope: row.telegram_history_scope,
        status: row.status,
        result_markdown: row.result_markdown,
        error: row.error,
        has_trace_data: row.trace_data_zstd.is_some(),
        snapshot_state,
        snapshot_captured_at: row.snapshot_captured_at,
        snapshot_error: row.snapshot_error,
        created_at: row.created_at,
        completed_at: row.completed_at,
        scope_label_snapshot: row.scope_label_snapshot,
        snapshot_message_count: row.snapshot_message_count,
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct AnalysisRunListFilters {
    pub(crate) source_id: Option<i64>,
    pub(crate) source_group_id: Option<i64>,
    pub(crate) project_id: Option<i64>,
    pub(crate) limit: i64,
    pub(crate) query: Option<String>,
    pub(crate) status: Option<String>,
    pub(crate) provider: Option<String>,
    pub(crate) model: Option<String>,
    pub(crate) template: Option<String>,
    pub(crate) date_from: Option<String>,
    pub(crate) date_to: Option<String>,
}

const ANALYSIS_RUN_LIST_SELECT: &str = r#"
    SELECT
        runs.id,
        runs.run_type,
        runs.scope_type,
        runs.source_id,
        sources.title AS source_title,
        runs.source_group_id,
        groups.name AS source_group_name,
        runs.project_id,
        projects.name AS project_name,
        runs.period_from,
        runs.period_to,
        runs.output_language,
        runs.prompt_template_id,
        templates.name AS prompt_template_name,
        runs.prompt_template_version,
        runs.provider_profile,
        runs.provider,
        runs.model,
        runs.youtube_corpus_mode,
        COALESCE(runs.telegram_history_scope, 'current') AS telegram_history_scope,
        runs.status,
        runs.result_markdown,
        runs.trace_data_zstd,
        runs.scope_label_snapshot,
        runs.snapshot_captured_at,
        runs.snapshot_error,
        COALESCE(snapshot_counts.snapshot_message_count, 0) AS snapshot_message_count,
        runs.error,
        runs.created_at,
        runs.completed_at
    FROM analysis_runs runs
    LEFT JOIN sources ON sources.id = runs.source_id
    LEFT JOIN analysis_source_groups groups ON groups.id = runs.source_group_id
    LEFT JOIN projects ON projects.id = runs.project_id
    LEFT JOIN analysis_prompt_templates templates ON templates.id = runs.prompt_template_id
    LEFT JOIN (
        SELECT run_id, COUNT(*) AS snapshot_message_count
        FROM analysis_run_messages
        GROUP BY run_id
    ) snapshot_counts ON snapshot_counts.run_id = runs.id
    WHERE 1 = 1
"#;

const RUN_QUERY_FIELDS: [&str; 9] = [
    "lower(coalesce(runs.scope_label_snapshot, ''))",
    "lower(coalesce(sources.title, ''))",
    "lower(coalesce(groups.name, ''))",
    "lower(coalesce(projects.name, ''))",
    "lower(coalesce(templates.name, ''))",
    "lower(coalesce(runs.provider_profile, ''))",
    "lower(coalesce(runs.provider, ''))",
    "lower(coalesce(runs.model, ''))",
    "lower(coalesce(runs.error, ''))",
];

fn trimmed_filter(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn escaped_like_contains(value: &str) -> String {
    format!(
        "%{}%",
        value
            .trim()
            .to_lowercase()
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_")
    )
}

fn parse_yyyy_mm_dd_midnight(value: &str) -> Option<i64> {
    let value = value.trim();
    let bytes = value.as_bytes();
    if bytes.len() != 10
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || !bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| index == 4 || index == 7 || byte.is_ascii_digit())
    {
        return None;
    }

    ymd_to_unix_midnight(value)
}

fn parse_yyyy_mm_dd_day_end(value: &str) -> Option<i64> {
    parse_yyyy_mm_dd_midnight(value).map(|start| start + 86_399)
}

fn push_like_predicate(query: &mut QueryBuilder<'_, Sqlite>, expression: &str, value: &str) {
    query.push(" AND ");
    query.push(expression);
    query.push(" LIKE ");
    query.push_bind(escaped_like_contains(value));
    query.push(" ESCAPE '\\'");
}

fn push_search_term_predicate(query: &mut QueryBuilder<'_, Sqlite>, term: &str) {
    query.push(" AND (");
    for (index, field) in RUN_QUERY_FIELDS.iter().enumerate() {
        if index > 0 {
            query.push(" OR ");
        }
        query.push(*field);
        query.push(" LIKE ");
        query.push_bind(escaped_like_contains(term));
        query.push(" ESCAPE '\\'");
    }
    query.push(")");
}

pub(crate) async fn list_analysis_run_summaries(
    pool: &Pool<Sqlite>,
    filters: AnalysisRunListFilters,
) -> AppResult<Vec<AnalysisRunSummary>> {
    let scope_filter_count = [
        filters.source_id.is_some(),
        filters.source_group_id.is_some(),
        filters.project_id.is_some(),
    ]
    .into_iter()
    .filter(|selected| *selected)
    .count();
    if scope_filter_count > 1 {
        return Err(AppError::validation(
            "Pass only one of source_id, source_group_id, or project_id",
        ));
    }

    let mut query = QueryBuilder::<Sqlite>::new(ANALYSIS_RUN_LIST_SELECT);

    if let Some(source_id) = filters.source_id {
        query.push(" AND runs.source_id = ");
        query.push_bind(source_id);
    }

    if let Some(source_group_id) = filters.source_group_id {
        query.push(" AND runs.source_group_id = ");
        query.push_bind(source_group_id);
    }

    if let Some(project_id) = filters.project_id {
        query.push(" AND runs.project_id = ");
        query.push_bind(project_id);
    }

    match trimmed_filter(filters.status).as_deref() {
        Some("queued_running") => {
            query.push(" AND runs.status IN (");
            let mut separated = query.separated(", ");
            separated.push_bind(ANALYSIS_STATUS_QUEUED);
            separated.push_bind(ANALYSIS_STATUS_RUNNING);
            separated.push_unseparated(")");
        }
        Some("all") | None => {}
        Some(status) => {
            query.push(" AND runs.status = ");
            query.push_bind(status.to_string());
        }
    }

    if let Some(date_from) = trimmed_filter(filters.date_from)
        .as_deref()
        .and_then(parse_yyyy_mm_dd_midnight)
    {
        query.push(" AND runs.created_at >= ");
        query.push_bind(date_from);
    }

    if let Some(date_to) = trimmed_filter(filters.date_to)
        .as_deref()
        .and_then(parse_yyyy_mm_dd_day_end)
    {
        query.push(" AND runs.created_at <= ");
        query.push_bind(date_to);
    }

    if let Some(provider) = trimmed_filter(filters.provider) {
        push_like_predicate(&mut query, "lower(coalesce(runs.provider, ''))", &provider);
    }

    if let Some(model) = trimmed_filter(filters.model) {
        push_like_predicate(&mut query, "lower(coalesce(runs.model, ''))", &model);
    }

    if let Some(template) = trimmed_filter(filters.template) {
        push_like_predicate(&mut query, "lower(coalesce(templates.name, ''))", &template);
    }

    if let Some(search) = trimmed_filter(filters.query) {
        for term in search.split_whitespace() {
            push_search_term_predicate(&mut query, term);
        }
    }

    query.push(" ORDER BY runs.created_at DESC LIMIT ");
    query.push_bind(filters.limit.clamp(1, 100));

    let rows = query
        .build_query_as::<AnalysisRunRow>()
        .fetch_all(pool)
        .await
        .map_err(AppError::database)?;

    Ok(rows.into_iter().map(map_run_summary).collect())
}

pub(crate) async fn fetch_run_row(
    pool: &Pool<Sqlite>,
    run_id: i64,
) -> AppResult<Option<AnalysisRunRow>> {
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
            runs.project_id,
            projects.name AS project_name,
            runs.period_from,
            runs.period_to,
            runs.output_language,
            runs.prompt_template_id,
            templates.name AS prompt_template_name,
            runs.prompt_template_version,
            runs.provider_profile,
            runs.provider,
            runs.model,
            runs.youtube_corpus_mode,
            COALESCE(runs.telegram_history_scope, 'current') AS telegram_history_scope,
            runs.status,
            runs.result_markdown,
            runs.trace_data_zstd,
            runs.scope_label_snapshot,
            runs.snapshot_captured_at,
            runs.snapshot_error,
            COALESCE(snapshot_counts.snapshot_message_count, 0) AS snapshot_message_count,
            runs.error,
            runs.created_at,
            runs.completed_at
        FROM analysis_runs runs
        LEFT JOIN sources ON sources.id = runs.source_id
        LEFT JOIN analysis_source_groups groups ON groups.id = runs.source_group_id
        LEFT JOIN projects ON projects.id = runs.project_id
        LEFT JOIN analysis_prompt_templates templates ON templates.id = runs.prompt_template_id
        LEFT JOIN (
            SELECT run_id, COUNT(*) AS snapshot_message_count
            FROM analysis_run_messages
            GROUP BY run_id
        ) snapshot_counts ON snapshot_counts.run_id = runs.id
        WHERE runs.id = ?
        "#,
    )
    .bind(run_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)
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
        run.project_id,
        run.project_name.as_deref(),
        run.scope_label_snapshot.as_deref(),
    )
}
