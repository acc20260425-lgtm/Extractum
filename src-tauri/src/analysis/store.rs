use sqlx::{Pool, QueryBuilder, Sqlite};

use super::models::{
    AnalysisChatMessage, AnalysisPromptTemplate, AnalysisRunDetail, AnalysisRunRow,
    AnalysisRunSummary, AnalysisSourceGroup, AnalysisSourceGroupMember, AnalysisSourceGroupRow,
    CorpusMessage, StoredAnalysisItemRow,
};
use super::{
    decompress_text, default_report_template_body, now_secs, validate_chat_role,
    ANALYSIS_RUN_TYPE_REPORT, ANALYSIS_SCOPE_TYPE_SINGLE_SOURCE, ANALYSIS_SCOPE_TYPE_SOURCE_GROUP,
    ANALYSIS_STATUS_QUEUED, ANALYSIS_STATUS_RUNNING, DEFAULT_REPORT_TEMPLATE_NAME,
    TEMPLATE_KIND_REPORT,
};

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

pub(crate) fn map_run_summary(row: AnalysisRunRow) -> AnalysisRunSummary {
    AnalysisRunSummary {
        id: row.id,
        run_type: row.run_type,
        scope_type: row.scope_type,
        source_id: row.source_id,
        source_title: row.source_title,
        source_group_id: row.source_group_id,
        source_group_name: row.source_group_name,
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
    AnalysisRunDetail {
        id: row.id,
        run_type: row.run_type,
        scope_type: row.scope_type,
        source_id: row.source_id,
        source_title: row.source_title,
        source_group_id: row.source_group_id,
        source_group_name: row.source_group_name,
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

pub(crate) async fn resolve_run_source_ids(
    pool: &Pool<Sqlite>,
    run: &AnalysisRunDetail,
) -> Result<Vec<i64>, String> {
    if run.scope_type == ANALYSIS_SCOPE_TYPE_SINGLE_SOURCE {
        let source_id = run
            .source_id
            .ok_or_else(|| format!("Analysis run {} is missing source_id", run.id))?;
        return Ok(vec![source_id]);
    }

    if run.scope_type == ANALYSIS_SCOPE_TYPE_SOURCE_GROUP {
        let group_id = run
            .source_group_id
            .ok_or_else(|| format!("Analysis run {} is missing source_group_id", run.id))?;
        let group = fetch_source_group(pool, group_id)
            .await?
            .ok_or_else(|| format!("Analysis source group {group_id} not found"))?;
        return Ok(group
            .members
            .into_iter()
            .map(|member| member.source_id)
            .collect());
    }

    Err(format!("Unsupported analysis scope '{}'", run.scope_type))
}

pub(crate) async fn load_chat_messages_from_pool(
    pool: &Pool<Sqlite>,
    run_id: i64,
) -> Result<Vec<AnalysisChatMessage>, String> {
    sqlx::query_as(
        r#"
        SELECT id, run_id, role, content, created_at
        FROM analysis_chat_messages
        WHERE run_id = ?
        ORDER BY created_at ASC, id ASC
        "#,
    )
    .bind(run_id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())
}

pub(crate) async fn persist_chat_exchange(
    pool: &Pool<Sqlite>,
    run_id: i64,
    user_question: &str,
    assistant_answer: &str,
) -> Result<(), String> {
    validate_chat_role("user")?;
    validate_chat_role("assistant")?;

    let now = now_secs();
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    sqlx::query(
        r#"
        INSERT INTO analysis_chat_messages (run_id, role, content, created_at)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(run_id)
    .bind("user")
    .bind(user_question)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    sqlx::query(
        r#"
        INSERT INTO analysis_chat_messages (run_id, role, content, created_at)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(run_id)
    .bind("assistant")
    .bind(assistant_answer)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(())
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
            created_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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

pub(crate) async fn load_corpus_messages(
    pool: &Pool<Sqlite>,
    source_ids: &[i64],
    period_from: i64,
    period_to: i64,
) -> Result<Vec<CorpusMessage>, String> {
    if source_ids.is_empty() {
        return Ok(Vec::new());
    }

    let mut query = QueryBuilder::<Sqlite>::new(
        "SELECT id, source_id, external_id, author, published_at, content_zstd FROM items WHERE content_zstd IS NOT NULL AND published_at >= ",
    );
    query.push_bind(period_from);
    query.push(" AND published_at <= ");
    query.push_bind(period_to);
    query.push(" AND source_id IN (");

    {
        let mut separated = query.separated(", ");
        for source_id in source_ids {
            separated.push_bind(source_id);
        }
    }

    query.push(") ORDER BY published_at ASC, id ASC");

    let rows: Vec<StoredAnalysisItemRow> = query
        .build_query_as()
        .fetch_all(pool)
        .await
        .map_err(|e| e.to_string())?;

    rows.into_iter()
        .map(|row| {
            let content = decompress_text(
                row.content_zstd
                    .as_deref()
                    .ok_or_else(|| format!("Item {} is missing content", row.id))?,
            )?;

            Ok(CorpusMessage {
                item_id: row.id,
                source_id: row.source_id,
                external_id: row.external_id.clone(),
                published_at: row.published_at,
                author: row.author,
                r#ref: format!("s{}-m{}", row.source_id, row.external_id),
                content,
            })
        })
        .collect()
}
