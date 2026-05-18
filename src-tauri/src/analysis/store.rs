use sqlx::{Pool, Sqlite};

use super::corpus::YoutubeCorpusMode;
use super::models::{
    AnalysisPromptTemplate, AnalysisRunDetail, AnalysisRunRow, AnalysisRunSummary,
    AnalysisSnapshotState, AnalysisSourceGroup, AnalysisSourceGroupMember, AnalysisSourceGroupRow,
    CorpusMessage, StoredRunSnapshotRow,
};
use super::{
    default_report_template_body, now_secs, ANALYSIS_RUN_TYPE_REPORT,
    ANALYSIS_SCOPE_TYPE_SOURCE_GROUP, ANALYSIS_STATUS_CANCELLED, ANALYSIS_STATUS_COMPLETED,
    ANALYSIS_STATUS_FAILED, ANALYSIS_STATUS_QUEUED, ANALYSIS_STATUS_RUNNING,
    DEFAULT_REPORT_TEMPLATE_NAME, TEMPLATE_KIND_REPORT,
};
use crate::compression::{compress_text, decompress_text};

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

fn compute_snapshot_state(row: &AnalysisRunRow) -> Option<AnalysisSnapshotState> {
    if row.snapshot_captured_at.is_some() && row.snapshot_error.is_none() {
        return Some(AnalysisSnapshotState::Captured);
    }

    if row.snapshot_error.is_some() {
        return Some(AnalysisSnapshotState::CaptureFailed);
    }

    match row.status.as_str() {
        ANALYSIS_STATUS_COMPLETED if row.snapshot_message_count == 0 => {
            Some(AnalysisSnapshotState::MissingLegacy)
        }
        ANALYSIS_STATUS_FAILED | ANALYSIS_STATUS_CANCELLED => {
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
            runs.youtube_corpus_mode,
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
        SELECT id, name, source_type, created_at, updated_at
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
        source_type: group.source_type,
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

pub(crate) struct DuplicateRunLookup<'a> {
    pub(crate) scope_type: &'a str,
    pub(crate) source_id: Option<i64>,
    pub(crate) source_group_id: Option<i64>,
    pub(crate) period_from: i64,
    pub(crate) period_to: i64,
    pub(crate) output_language: &'a str,
    pub(crate) prompt_template_id: i64,
    pub(crate) provider_profile: &'a str,
    pub(crate) model: &'a str,
    pub(crate) youtube_corpus_mode: YoutubeCorpusMode,
}

pub(crate) async fn find_active_duplicate_run(
    pool: &Pool<Sqlite>,
    lookup: &DuplicateRunLookup<'_>,
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
          AND youtube_corpus_mode = ?
          AND status IN (?, ?)
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(ANALYSIS_RUN_TYPE_REPORT)
    .bind(lookup.scope_type)
    .bind(lookup.source_id)
    .bind(lookup.source_id)
    .bind(lookup.source_group_id)
    .bind(lookup.source_group_id)
    .bind(lookup.period_from)
    .bind(lookup.period_to)
    .bind(lookup.output_language)
    .bind(lookup.prompt_template_id)
    .bind(lookup.provider_profile)
    .bind(lookup.model)
    .bind(lookup.youtube_corpus_mode.as_wire())
    .bind(ANALYSIS_STATUS_QUEUED)
    .bind(ANALYSIS_STATUS_RUNNING)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())
}

pub(crate) struct AnalysisRunInsert<'a> {
    pub(crate) scope_type: &'a str,
    pub(crate) source_id: Option<i64>,
    pub(crate) source_group_id: Option<i64>,
    pub(crate) period_from: i64,
    pub(crate) period_to: i64,
    pub(crate) output_language: &'a str,
    pub(crate) prompt_template: &'a AnalysisPromptTemplate,
    pub(crate) provider_profile: &'a str,
    pub(crate) provider: &'a str,
    pub(crate) model: &'a str,
    pub(crate) youtube_corpus_mode: YoutubeCorpusMode,
}

pub(crate) async fn insert_analysis_run(
    pool: &Pool<Sqlite>,
    insert: &AnalysisRunInsert<'_>,
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
            youtube_corpus_mode,
            status,
            scope_label_snapshot,
            created_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NULL, ?)
        RETURNING id
        "#,
    )
    .bind(ANALYSIS_RUN_TYPE_REPORT)
    .bind(insert.scope_type)
    .bind(insert.source_id)
    .bind(insert.source_group_id)
    .bind(insert.period_from)
    .bind(insert.period_to)
    .bind(insert.output_language)
    .bind(insert.prompt_template.id)
    .bind(insert.prompt_template.version)
    .bind(insert.provider_profile)
    .bind(insert.provider)
    .bind(insert.model)
    .bind(insert.youtube_corpus_mode.as_wire())
    .bind(ANALYSIS_STATUS_QUEUED)
    .bind(created_at)
    .fetch_one(pool)
    .await
    .map_err(|e| e.to_string())
}

pub(crate) fn sanitize_snapshot_error(category: &str, raw: &str) -> String {
    let mut text = raw
        .chars()
        .map(|ch| if ch.is_control() { ' ' } else { ch })
        .collect::<String>();

    for marker in ["file://", "C:\\", "c:\\", "/home/", "/Users/", "/tmp/"] {
        while let Some(start) = text.find(marker) {
            let end = text[start..]
                .find(char::is_whitespace)
                .map(|offset| start + offset)
                .unwrap_or_else(|| text.len());
            text.replace_range(start..end, "[redacted]");
        }
    }

    for marker in ["http://", "https://"] {
        let mut search_from = 0usize;
        while let Some(relative_start) = text[search_from..].find(marker) {
            let start = search_from + relative_start;
            let end = text[start..]
                .find(char::is_whitespace)
                .map(|offset| start + offset)
                .unwrap_or_else(|| text.len());
            let url = &text[start..end];
            let clean_end = url.find(['?', '#']).unwrap_or(url.len());
            let replacement = format!("{}[redacted]", &url[..clean_end]);
            text.replace_range(start..end, &replacement);
            search_from = start + replacement.len();
        }
    }

    let lower = text.to_lowercase();
    if lower.contains("bearer ")
        || lower.contains("api_key")
        || lower.contains("apikey")
        || lower.contains("sk-")
        || lower.contains("cookie")
    {
        text = category.to_string();
    }

    let compact = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let bounded = compact.chars().take(512).collect::<String>();
    if bounded.trim().is_empty() {
        category.to_string()
    } else {
        bounded
    }
}

fn validate_snapshot_message(message: &CorpusMessage) -> Result<(), String> {
    if message.r#ref.trim().is_empty() {
        return Err("Snapshot message ref is required".to_string());
    }
    if message.content.trim().is_empty() {
        return Err(format!(
            "Snapshot message {} content is required",
            message.r#ref
        ));
    }
    if message.item_kind.as_deref().unwrap_or("").trim().is_empty() {
        return Err(format!(
            "Snapshot message {} item_kind is required",
            message.r#ref
        ));
    }
    let source_type = message.source_type.as_deref().unwrap_or("").trim();
    if source_type.is_empty() {
        return Err(format!(
            "Snapshot message {} source_type is required",
            message.r#ref
        ));
    }
    if matches!(source_type, "telegram" | "youtube")
        && message
            .source_subtype
            .as_deref()
            .unwrap_or("")
            .trim()
            .is_empty()
    {
        return Err(format!(
            "Snapshot message {} source_subtype is required for {source_type}",
            message.r#ref
        ));
    }
    Ok(())
}

async fn load_run_snapshot_messages_on_transaction(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    run_id: i64,
) -> Result<Vec<CorpusMessage>, String> {
    let rows: Vec<StoredRunSnapshotRow> = sqlx::query_as(
        r#"
        SELECT
            item_id,
            source_id,
            external_id,
            author,
            published_at,
            ref,
            content_zstd,
            item_kind,
            source_type,
            source_subtype,
            metadata_zstd
        FROM analysis_run_messages
        WHERE run_id = ?
        ORDER BY published_at ASC, ref ASC
        "#,
    )
    .bind(run_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(|e| e.to_string())?;

    rows.into_iter()
        .map(|row| {
            Ok(CorpusMessage {
                item_id: row.item_id,
                source_id: row.source_id,
                external_id: row.external_id,
                published_at: row.published_at,
                author: row.author,
                content: decompress_text(&row.content_zstd)?,
                r#ref: row.r#ref,
                item_kind: row.item_kind,
                source_type: row.source_type,
                source_subtype: row.source_subtype,
                metadata_zstd: row.metadata_zstd,
            })
        })
        .collect()
}

pub(crate) async fn capture_run_snapshot(
    pool: &Pool<Sqlite>,
    run_id: i64,
    scope_label: &str,
    corpus: &[CorpusMessage],
) -> Result<Vec<CorpusMessage>, String> {
    if corpus.is_empty() {
        return Err("Snapshot capture failed: empty corpus".to_string());
    }

    for message in corpus {
        validate_snapshot_message(message)?;
    }

    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    sqlx::query(
        r#"
        UPDATE analysis_runs
        SET scope_label_snapshot = ?,
            snapshot_captured_at = NULL,
            snapshot_error = NULL
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
                content_zstd,
                item_kind,
                source_type,
                source_subtype,
                metadata_zstd
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
        .bind(message.item_kind.as_deref())
        .bind(message.source_type.as_deref())
        .bind(message.source_subtype.as_deref())
        .bind(message.metadata_zstd.as_deref())
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;
    }

    let captured = load_run_snapshot_messages_on_transaction(&mut tx, run_id).await?;
    if captured.is_empty() {
        return Err("Snapshot capture failed: reloaded snapshot is empty".to_string());
    }

    sqlx::query(
        "UPDATE analysis_runs SET snapshot_captured_at = datetime('now'), snapshot_error = NULL WHERE id = ?",
    )
    .bind(run_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(captured)
}

#[allow(dead_code)]
pub(crate) async fn persist_run_snapshot(
    pool: &Pool<Sqlite>,
    run_id: i64,
    scope_label: &str,
    corpus: &[CorpusMessage],
) -> Result<(), String> {
    capture_run_snapshot(pool, run_id, scope_label, corpus)
        .await
        .map(|_| ())
}

pub(crate) async fn mark_run_capture_failed(
    pool: &Pool<Sqlite>,
    run_id: i64,
    snapshot_error: &str,
    completed_at: i64,
) -> Result<(), String> {
    let sanitized = sanitize_snapshot_error("Snapshot capture failed", snapshot_error);
    sqlx::query(
        r#"
        UPDATE analysis_runs
        SET
            status = ?,
            error = ?,
            snapshot_error = ?,
            completed_at = ?
        WHERE id = ?
        "#,
    )
    .bind(crate::analysis::ANALYSIS_STATUS_FAILED)
    .bind(&sanitized)
    .bind(&sanitized)
    .bind(completed_at)
    .bind(run_id)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;
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

pub(crate) async fn delete_saved_run(pool: &Pool<Sqlite>, run_id: i64) -> Result<(), String> {
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    sqlx::query("DELETE FROM analysis_chat_messages WHERE run_id = ?")
        .bind(run_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    sqlx::query("DELETE FROM analysis_run_messages WHERE run_id = ?")
        .bind(run_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    let deleted = sqlx::query("DELETE FROM analysis_runs WHERE id = ?")
        .bind(run_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?
        .rows_affected();

    if deleted == 0 {
        return Err(format!("Analysis run {run_id} not found"));
    }

    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        capture_run_snapshot, delete_saved_run, map_run_detail, map_run_summary,
        resolve_run_scope_label, sanitize_snapshot_error,
    };
    use crate::analysis::models::{
        AnalysisPromptTemplate, AnalysisRunDetail, AnalysisRunRow, CorpusMessage,
    };

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
            youtube_corpus_mode: "transcript_description_comments".to_string(),
            status: "completed".to_string(),
            result_markdown: Some("Saved report".to_string()),
            trace_data_zstd: Some(vec![1, 2, 3]),
            scope_label_snapshot: Some("Frozen group".to_string()),
            snapshot_captured_at: Some("2026-05-18T10:00:00Z".to_string()),
            snapshot_error: None,
            snapshot_message_count: 2,
            error: None,
            created_at: 1_710_000_500,
            completed_at: Some(1_710_000_600),
        }
    }

    fn sample_run() -> AnalysisRunDetail {
        map_run_detail(sample_run_row())
    }

    async fn snapshot_store_pool() -> sqlx::SqlitePool {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");
        sqlx::query(
            r#"
            CREATE TABLE analysis_runs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                scope_label_snapshot TEXT,
                snapshot_captured_at TEXT,
                snapshot_error TEXT,
                status TEXT,
                error TEXT,
                completed_at INTEGER
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create runs");
        sqlx::query(
            r#"
            CREATE TABLE analysis_run_messages (
                run_id INTEGER NOT NULL,
                item_id INTEGER NOT NULL,
                source_id INTEGER NOT NULL,
                external_id TEXT NOT NULL,
                author TEXT,
                published_at INTEGER NOT NULL,
                ref TEXT NOT NULL,
                content_zstd BLOB NOT NULL,
                item_kind TEXT,
                source_type TEXT,
                source_subtype TEXT,
                metadata_zstd BLOB,
                PRIMARY KEY (run_id, ref)
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create messages");
        sqlx::query("INSERT INTO analysis_runs (id, status) VALUES (1, 'running')")
            .execute(&pool)
            .await
            .expect("insert run");
        pool
    }

    fn strict_snapshot_message(label: &str) -> CorpusMessage {
        CorpusMessage {
            item_id: 10,
            source_id: 2,
            external_id: label.to_string(),
            published_at: 1_710_000_000,
            author: Some("Alice".to_string()),
            content: format!("content {label}"),
            r#ref: format!("s2-i10-{label}"),
            item_kind: Some("telegram_message".to_string()),
            source_type: Some("telegram".to_string()),
            source_subtype: Some("channel".to_string()),
            metadata_zstd: None,
        }
    }

    #[test]
    fn sanitize_snapshot_error_bounds_lines_paths_urls_and_tokens() {
        let long = "x".repeat(600);
        let raw = format!(
            "failed at C:\\Users\\Dima\\AppData\\Local\\Extractum\\db.sqlite\n\
             see /home/dima/.config/extractum/db.sqlite and file:///tmp/secret.txt \
             https://example.test/path?token=abc#frag \
             bearer sk-live-secret api_key=secret {long}"
        );

        let sanitized = sanitize_snapshot_error("Snapshot capture failed", &raw);

        assert!(sanitized.chars().count() <= 512);
        assert!(!sanitized.contains('\n'));
        assert!(!sanitized.contains("C:\\"));
        assert!(!sanitized.contains("/home/dima"));
        assert!(!sanitized.contains("file://"));
        assert!(!sanitized.contains("?token="));
        assert!(!sanitized.contains("#frag"));
        assert!(!sanitized.to_lowercase().contains("bearer"));
        assert!(!sanitized.contains("sk-live-secret"));
        assert!(!sanitized.contains("api_key=secret"));
    }

    #[tokio::test]
    async fn capture_run_snapshot_marks_captured_after_reload_and_replaces_rows() {
        let pool = snapshot_store_pool().await;

        let first = capture_run_snapshot(&pool, 1, "Frozen scope", &[strict_snapshot_message("a")])
            .await
            .expect("capture first");
        let second =
            capture_run_snapshot(&pool, 1, "Frozen scope", &[strict_snapshot_message("b")])
                .await
                .expect("capture second");

        assert_eq!(first.len(), 1);
        assert_eq!(second.len(), 1);
        assert_eq!(second[0].external_id, "b");

        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM analysis_run_messages WHERE run_id = 1")
                .fetch_one(&pool)
                .await
                .expect("count messages");
        assert_eq!(count, 1);

        let marker: Option<String> =
            sqlx::query_scalar("SELECT snapshot_captured_at FROM analysis_runs WHERE id = 1")
                .fetch_one(&pool)
                .await
                .expect("load marker");
        assert!(marker.is_some());

        let snapshot_error: Option<String> =
            sqlx::query_scalar("SELECT snapshot_error FROM analysis_runs WHERE id = 1")
                .fetch_one(&pool)
                .await
                .expect("load snapshot error");
        assert_eq!(snapshot_error, None);
    }

    #[tokio::test]
    async fn capture_run_snapshot_rejects_missing_required_fields_without_marker() {
        let pool = snapshot_store_pool().await;
        let mut message = strict_snapshot_message("bad");
        message.item_kind = None;

        let error = match capture_run_snapshot(&pool, 1, "Frozen scope", &[message]).await {
            Ok(_) => panic!("missing item_kind should fail"),
            Err(error) => error,
        };
        assert!(error.contains("item_kind"));

        let marker: Option<String> =
            sqlx::query_scalar("SELECT snapshot_captured_at FROM analysis_runs WHERE id = 1")
                .fetch_one(&pool)
                .await
                .expect("load marker");
        assert_eq!(marker, None);

        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM analysis_run_messages WHERE run_id = 1")
                .fetch_one(&pool)
                .await
                .expect("count messages");
        assert_eq!(count, 0);
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

    #[test]
    fn map_run_summary_exposes_captured_snapshot_state() {
        let summary = map_run_summary(sample_run_row());

        assert_eq!(
            summary.snapshot_state,
            Some(crate::analysis::models::AnalysisSnapshotState::Captured)
        );
        assert_eq!(
            summary.snapshot_captured_at.as_deref(),
            Some("2026-05-18T10:00:00Z")
        );
        assert_eq!(summary.snapshot_error, None);
    }

    #[test]
    fn map_run_detail_exposes_missing_legacy_snapshot_state() {
        let mut row = sample_run_row();
        row.snapshot_captured_at = None;
        row.snapshot_error = None;
        row.snapshot_message_count = 0;
        row.status = crate::analysis::ANALYSIS_STATUS_COMPLETED.to_string();

        let detail = map_run_detail(row);

        assert_eq!(
            detail.snapshot_state,
            Some(crate::analysis::models::AnalysisSnapshotState::MissingLegacy)
        );
        assert_eq!(detail.snapshot_captured_at, None);
        assert_eq!(detail.snapshot_error, None);
    }

    #[test]
    fn map_run_summary_exposes_capture_failed_snapshot_state() {
        let mut row = sample_run_row();
        row.snapshot_captured_at = None;
        row.snapshot_error = Some("Snapshot capture failed".to_string());
        row.snapshot_message_count = 0;
        row.status = crate::analysis::ANALYSIS_STATUS_FAILED.to_string();

        let summary = map_run_summary(row);

        assert_eq!(
            summary.snapshot_state,
            Some(crate::analysis::models::AnalysisSnapshotState::CaptureFailed)
        );
        assert_eq!(
            summary.snapshot_error.as_deref(),
            Some("Snapshot capture failed")
        );
    }

    #[test]
    fn map_run_summary_exposes_null_snapshot_state_for_active_runs_before_capture() {
        let mut row = sample_run_row();
        row.snapshot_captured_at = None;
        row.snapshot_error = None;
        row.snapshot_message_count = 0;
        row.status = crate::analysis::ANALYSIS_STATUS_RUNNING.to_string();

        let summary = map_run_summary(row);

        assert_eq!(summary.snapshot_state, None);
    }

    #[test]
    fn failed_terminal_run_without_capture_marker_is_capture_failed() {
        let mut row = sample_run_row();
        row.snapshot_captured_at = None;
        row.snapshot_error = None;
        row.snapshot_message_count = 0;
        row.status = crate::analysis::ANALYSIS_STATUS_CANCELLED.to_string();

        let summary = map_run_summary(row);

        assert_eq!(
            summary.snapshot_state,
            Some(crate::analysis::models::AnalysisSnapshotState::CaptureFailed)
        );
    }

    #[test]
    fn map_run_summary_exposes_youtube_corpus_mode() {
        let summary = map_run_summary(sample_run_row());
        assert_eq!(
            summary.youtube_corpus_mode,
            "transcript_description_comments"
        );
    }

    #[test]
    fn map_run_detail_exposes_youtube_corpus_mode() {
        let detail = map_run_detail(sample_run_row());
        assert_eq!(
            detail.youtube_corpus_mode,
            "transcript_description_comments"
        );
    }

    #[tokio::test]
    async fn insert_analysis_run_persists_youtube_corpus_mode() {
        use super::{insert_analysis_run, AnalysisRunInsert};
        use crate::analysis::corpus::YoutubeCorpusMode;

        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");
        sqlx::query(
            r#"
            CREATE TABLE analysis_runs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                run_type TEXT NOT NULL,
                scope_type TEXT NOT NULL,
                source_id INTEGER,
                source_group_id INTEGER,
                period_from INTEGER NOT NULL,
                period_to INTEGER NOT NULL,
                output_language TEXT NOT NULL,
                prompt_template_id INTEGER,
                prompt_template_version INTEGER NOT NULL,
                provider_profile TEXT NOT NULL,
                provider TEXT NOT NULL,
                model TEXT NOT NULL,
                youtube_corpus_mode TEXT NOT NULL DEFAULT 'transcript_description',
                status TEXT NOT NULL,
                result_markdown TEXT,
                trace_data_zstd BLOB,
                scope_label_snapshot TEXT,
                snapshot_captured_at TEXT,
                snapshot_error TEXT,
                error TEXT,
                created_at INTEGER NOT NULL,
                completed_at INTEGER
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create runs");

        let template = AnalysisPromptTemplate {
            id: 5,
            name: "Report".to_string(),
            template_kind: "report".to_string(),
            body: "Body".to_string(),
            version: 3,
            is_builtin: false,
            created_at: 1,
            updated_at: 1,
        };

        let run_id = insert_analysis_run(
            &pool,
            &AnalysisRunInsert {
                scope_type: "single_source",
                source_id: Some(7),
                source_group_id: None,
                period_from: 10,
                period_to: 20,
                output_language: "English",
                prompt_template: &template,
                provider_profile: "default",
                provider: "gemini",
                model: "gemini-2.5-flash",
                youtube_corpus_mode: YoutubeCorpusMode::TranscriptDescriptionComments,
            },
        )
        .await
        .expect("insert run");

        let mode = sqlx::query_scalar::<_, String>(
            "SELECT youtube_corpus_mode FROM analysis_runs WHERE id = ?",
        )
        .bind(run_id)
        .fetch_one(&pool)
        .await
        .expect("load mode");

        assert_eq!(mode, "transcript_description_comments");
    }

    #[tokio::test]
    async fn delete_saved_run_removes_run_and_saved_children() {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");
        sqlx::query(
            "CREATE TABLE analysis_runs (
                id INTEGER PRIMARY KEY,
                snapshot_captured_at TEXT,
                snapshot_error TEXT
            )",
        )
        .execute(&pool)
        .await
        .expect("create runs");
        sqlx::query(
            "CREATE TABLE analysis_chat_messages (id INTEGER PRIMARY KEY, run_id INTEGER NOT NULL)",
        )
        .execute(&pool)
        .await
        .expect("create chat messages");
        sqlx::query(
            "CREATE TABLE analysis_run_messages (run_id INTEGER NOT NULL, ref TEXT NOT NULL)",
        )
        .execute(&pool)
        .await
        .expect("create run messages");

        sqlx::query("INSERT INTO analysis_runs (id) VALUES (42)")
            .execute(&pool)
            .await
            .expect("insert run");
        sqlx::query("INSERT INTO analysis_chat_messages (run_id) VALUES (42)")
            .execute(&pool)
            .await
            .expect("insert chat");
        sqlx::query("INSERT INTO analysis_run_messages (run_id, ref) VALUES (42, 's1-m1')")
            .execute(&pool)
            .await
            .expect("insert saved corpus");

        delete_saved_run(&pool, 42).await.expect("delete saved run");

        let runs = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM analysis_runs")
            .fetch_one(&pool)
            .await
            .expect("count runs");
        let chat_messages =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM analysis_chat_messages")
                .fetch_one(&pool)
                .await
                .expect("count chat messages");
        let saved_messages =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM analysis_run_messages")
                .fetch_one(&pool)
                .await
                .expect("count saved messages");

        assert_eq!(runs, 0);
        assert_eq!(chat_messages, 0);
        assert_eq!(saved_messages, 0);
    }
}
