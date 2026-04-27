use sqlx::{Pool, QueryBuilder, Sqlite};

use super::models::{
    AnalysisChatMessage, AnalysisPromptTemplate, AnalysisRunDetail, AnalysisRunRow,
    AnalysisRunSummary, AnalysisSourceGroup, AnalysisSourceGroupMember, AnalysisSourceGroupRow,
    CorpusMessage, StoredAnalysisItemRow, StoredRunSnapshotRow,
};
use super::{
    decompress_text, default_report_template_body, now_secs, validate_chat_role,
    ANALYSIS_RUN_TYPE_REPORT, ANALYSIS_SCOPE_TYPE_SINGLE_SOURCE, ANALYSIS_SCOPE_TYPE_SOURCE_GROUP,
    ANALYSIS_STATUS_QUEUED, ANALYSIS_STATUS_RUNNING, DEFAULT_REPORT_TEMPLATE_NAME,
    TEMPLATE_KIND_REPORT,
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
    if let Some(label) = run.scope_label_snapshot.as_deref() {
        let trimmed = label.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    if run.scope_type == ANALYSIS_SCOPE_TYPE_SOURCE_GROUP {
        return run
            .source_group_name
            .clone()
            .unwrap_or_else(|| format!("Group {}", run.source_group_id.unwrap_or_default()));
    }

    run.source_title
        .clone()
        .unwrap_or_else(|| format!("Source {}", run.source_id.unwrap_or_default()))
}

pub(crate) async fn resolve_run_source_ids(
    pool: &Pool<Sqlite>,
    run: &AnalysisRunDetail,
) -> Result<Vec<i64>, String> {
    let snapshot_source_ids = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT DISTINCT source_id
        FROM analysis_run_messages
        WHERE run_id = ?
        ORDER BY source_id ASC
        "#,
    )
    .bind(run.id)
    .fetch_all(pool)
    .await
    .map_err(|e| e.to_string())?;

    if !snapshot_source_ids.is_empty() {
        return Ok(snapshot_source_ids);
    }

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

pub(crate) async fn load_run_snapshot_messages(
    pool: &Pool<Sqlite>,
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
            content_zstd
        FROM analysis_run_messages
        WHERE run_id = ?
        ORDER BY published_at ASC, ref ASC
        "#,
    )
    .bind(run_id)
    .fetch_all(pool)
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
            })
        })
        .collect()
}

pub(crate) async fn load_run_corpus_messages(
    pool: &Pool<Sqlite>,
    run: &AnalysisRunDetail,
) -> Result<Vec<CorpusMessage>, String> {
    let snapshot = load_run_snapshot_messages(pool, run.id).await?;
    if !snapshot.is_empty() {
        return Ok(snapshot);
    }

    let source_ids = resolve_run_source_ids(pool, run).await?;
    load_corpus_messages(pool, &source_ids, run.period_from, run.period_to).await
}

#[cfg(test)]
mod tests {
    use sqlx::SqlitePool;

    use super::{
        load_run_corpus_messages, load_run_snapshot_messages, persist_run_snapshot,
        resolve_run_scope_label, resolve_run_source_ids,
    };
    use crate::analysis::models::AnalysisRunDetail;

    fn sample_corpus() -> Vec<crate::analysis::models::CorpusMessage> {
        vec![
            crate::analysis::models::CorpusMessage {
                item_id: 11,
                source_id: 2,
                external_id: "100".to_string(),
                published_at: 1_710_000_000,
                author: Some("Alice".to_string()),
                content: "First frozen message".to_string(),
                r#ref: "s2-m100".to_string(),
            },
            crate::analysis::models::CorpusMessage {
                item_id: 12,
                source_id: 4,
                external_id: "101".to_string(),
                published_at: 1_710_000_100,
                author: None,
                content: "Second frozen message".to_string(),
                r#ref: "s4-m101".to_string(),
            },
        ]
    }

    async fn snapshot_pool() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");

        sqlx::query(
            r#"
            CREATE TABLE sources (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create sources");

        sqlx::query(
            r#"
            CREATE TABLE items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                source_id INTEGER NOT NULL,
                external_id TEXT NOT NULL,
                author TEXT,
                published_at INTEGER NOT NULL,
                content_zstd BLOB
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create items");

        sqlx::query(
            r#"
            CREATE TABLE analysis_source_groups (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create groups");

        sqlx::query(
            r#"
            CREATE TABLE analysis_source_group_members (
                group_id INTEGER NOT NULL,
                source_id INTEGER NOT NULL,
                created_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create group members");

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
                status TEXT NOT NULL,
                result_markdown TEXT,
                trace_data_zstd BLOB,
                scope_label_snapshot TEXT,
                error TEXT,
                created_at INTEGER NOT NULL,
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
                PRIMARY KEY (run_id, ref)
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create run messages");

        pool
    }

    fn sample_run() -> AnalysisRunDetail {
        AnalysisRunDetail {
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
            error: None,
            has_trace_data: true,
            created_at: 1_710_000_500,
            completed_at: Some(1_710_000_600),
            scope_label_snapshot: Some("Frozen group".to_string()),
        }
    }

    #[tokio::test]
    async fn run_snapshot_roundtrips_frozen_corpus() {
        let pool = snapshot_pool().await;
        sqlx::query(
            r#"
            INSERT INTO analysis_runs (
                id,
                run_type,
                scope_type,
                source_group_id,
                period_from,
                period_to,
                output_language,
                prompt_template_version,
                provider_profile,
                provider,
                model,
                status,
                created_at
            )
            VALUES (1, 'report', 'source_group', 9, ?, ?, 'English', 1, 'default', 'gemini', 'model', 'completed', ?)
            "#,
        )
        .bind(1_700_000_000_i64)
        .bind(1_800_000_000_i64)
        .bind(1_710_000_500_i64)
        .execute(&pool)
        .await
        .expect("insert run");

        let corpus = sample_corpus();
        persist_run_snapshot(&pool, 1, "Frozen group", &corpus)
            .await
            .expect("persist snapshot");

        let loaded = load_run_snapshot_messages(&pool, 1)
            .await
            .expect("load snapshot");

        assert_eq!(loaded.len(), corpus.len());
        assert_eq!(loaded[0].r#ref, "s2-m100");
        assert_eq!(loaded[1].content, "Second frozen message");
    }

    #[tokio::test]
    async fn resolve_run_source_ids_prefers_snapshot_over_live_group_membership() {
        let pool = snapshot_pool().await;
        sqlx::query(
            r#"
            INSERT INTO analysis_source_groups (id, name, created_at, updated_at)
            VALUES (9, 'Live group', 1, 1)
            "#,
        )
        .execute(&pool)
        .await
        .expect("insert group");
        sqlx::query(
            r#"
            INSERT INTO analysis_source_group_members (group_id, source_id, created_at)
            VALUES (9, 77, 1)
            "#,
        )
        .execute(&pool)
        .await
        .expect("insert live member");
        sqlx::query(
            r#"
            INSERT INTO analysis_runs (
                id,
                run_type,
                scope_type,
                source_group_id,
                period_from,
                period_to,
                output_language,
                prompt_template_version,
                provider_profile,
                provider,
                model,
                status,
                created_at
            )
            VALUES (1, 'report', 'source_group', 9, ?, ?, 'English', 1, 'default', 'gemini', 'model', 'completed', ?)
            "#,
        )
        .bind(1_700_000_000_i64)
        .bind(1_800_000_000_i64)
        .bind(1_710_000_500_i64)
        .execute(&pool)
        .await
        .expect("insert run");

        persist_run_snapshot(&pool, 1, "Frozen group", &sample_corpus())
            .await
            .expect("persist snapshot");

        let source_ids = resolve_run_source_ids(&pool, &sample_run())
            .await
            .expect("resolve source ids");

        assert_eq!(source_ids, vec![2, 4]);
    }

    #[tokio::test]
    async fn load_run_corpus_messages_uses_snapshot_when_available() {
        let pool = snapshot_pool().await;
        sqlx::query(
            r#"
            INSERT INTO analysis_runs (
                id,
                run_type,
                scope_type,
                source_group_id,
                period_from,
                period_to,
                output_language,
                prompt_template_version,
                provider_profile,
                provider,
                model,
                status,
                created_at
            )
            VALUES (1, 'report', 'source_group', 9, ?, ?, 'English', 1, 'default', 'gemini', 'model', 'completed', ?)
            "#,
        )
        .bind(1_700_000_000_i64)
        .bind(1_800_000_000_i64)
        .bind(1_710_000_500_i64)
        .execute(&pool)
        .await
        .expect("insert run");
        persist_run_snapshot(&pool, 1, "Frozen group", &sample_corpus())
            .await
            .expect("persist snapshot");

        let corpus = load_run_corpus_messages(&pool, &sample_run())
            .await
            .expect("load run corpus");

        assert_eq!(corpus.len(), 2);
        assert_eq!(corpus[0].external_id, "100");
        assert_eq!(corpus[1].r#ref, "s4-m101");
    }

    #[test]
    fn resolve_run_scope_label_prefers_frozen_value() {
        let run = sample_run();
        assert_eq!(resolve_run_scope_label(&run), "Frozen group");
    }
}
