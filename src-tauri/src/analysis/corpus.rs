use sqlx::{Pool, QueryBuilder, Sqlite};

use super::models::{
    AnalysisRunDetail, CorpusMessage, StoredAnalysisItemRow, StoredRunSnapshotRow,
};
use super::store::fetch_source_group;
use super::{ANALYSIS_SCOPE_TYPE_SINGLE_SOURCE, ANALYSIS_SCOPE_TYPE_SOURCE_GROUP};
use crate::compression::decompress_text;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AnalysisRunPreflightLimits {
    pub max_messages_per_run: usize,
    pub max_chunks_per_run: usize,
    pub max_estimated_input_chars_per_run: usize,
    /// Reserved for future retry-aware budgeting. Currently equals
    /// `max_chunks_per_run` because each chunk creates exactly one
    /// background request.
    pub max_background_requests_per_run: usize,
}

impl Default for AnalysisRunPreflightLimits {
    fn default() -> Self {
        Self {
            max_messages_per_run: 10_000,
            max_chunks_per_run: 80,
            max_estimated_input_chars_per_run: 1_500_000,
            max_background_requests_per_run: 80,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AnalysisRunPreflight {
    pub source_ids: Vec<i64>,
    pub message_count: usize,
    pub estimated_input_chars: usize,
    pub estimated_chunks: usize,
    pub limits: AnalysisRunPreflightLimits,
}

pub(crate) fn estimate_message_input_chars(
    content: &str,
    r#ref: &str,
    author: Option<&str>,
) -> usize {
    content.len() + r#ref.len() + author.unwrap_or("").len() + 64
}

pub(crate) fn live_corpus_ref(source_id: i64, item_id: i64) -> String {
    format!("s{source_id}-i{item_id}")
}

pub(crate) fn estimate_preflight_chunk_count(message_sizes: &[usize], max_chars: usize) -> usize {
    let mut chunks = 0usize;
    let mut current_chars = 0usize;

    for size in message_sizes {
        if current_chars > 0 && current_chars + size > max_chars {
            chunks += 1;
            current_chars = 0;
        }
        current_chars += size;
    }

    if current_chars > 0 {
        chunks += 1;
    }

    chunks
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
                r#ref: format!("s{}-i{}", row.source_id, row.id),
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
        estimate_message_input_chars, estimate_preflight_chunk_count, load_corpus_messages,
        load_run_corpus_messages, load_run_snapshot_messages, resolve_run_source_ids,
        AnalysisRunPreflightLimits,
    };
    use crate::analysis::models::{AnalysisRunDetail, CorpusMessage};
    use crate::analysis::store::persist_run_snapshot;
    use crate::compression::compress_text;

    fn sample_corpus() -> Vec<CorpusMessage> {
        vec![
            CorpusMessage {
                item_id: 11,
                source_id: 2,
                external_id: "100".to_string(),
                published_at: 1_710_000_000,
                author: Some("Alice".to_string()),
                content: "First frozen message".to_string(),
                r#ref: "s2-m100".to_string(),
            },
            CorpusMessage {
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
            scope_label: "Frozen group".to_string(),
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

    #[test]
    fn estimated_message_chars_match_report_chunk_accounting() {
        let message = CorpusMessage {
            item_id: 11,
            source_id: 2,
            external_id: "100".to_string(),
            published_at: 1_710_000_000,
            author: Some("Alice".to_string()),
            content: "First live document".to_string(),
            r#ref: "s2-i11".to_string(),
        };

        assert_eq!(
            estimate_message_input_chars(
                &message.content,
                &message.r#ref,
                message.author.as_deref()
            ),
            message.content.len() + message.r#ref.len() + "Alice".len() + 64
        );
    }

    #[test]
    fn estimated_chunk_count_matches_chunk_boundary_behavior() {
        assert_eq!(estimate_preflight_chunk_count(&[], 16_000), 0);
        assert_eq!(estimate_preflight_chunk_count(&[8_000, 7_000], 16_000), 1);
        assert_eq!(estimate_preflight_chunk_count(&[8_000, 9_000], 16_000), 2);
        assert_eq!(estimate_preflight_chunk_count(&[20_000], 16_000), 1);
    }

    #[test]
    fn default_preflight_limits_are_conservative() {
        let limits = AnalysisRunPreflightLimits::default();

        assert_eq!(limits.max_messages_per_run, 10_000);
        assert_eq!(limits.max_chunks_per_run, 80);
        assert_eq!(limits.max_estimated_input_chars_per_run, 1_500_000);
        assert_eq!(limits.max_background_requests_per_run, 80);
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

    #[tokio::test]
    async fn live_corpus_refs_use_local_item_ids() {
        let pool = snapshot_pool().await;
        let first_content = compress_text("First live document").expect("compress first");
        let second_content = compress_text("Second live document").expect("compress second");
        sqlx::query(
            r#"
            INSERT INTO items (id, source_id, external_id, author, published_at, content_zstd)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(11_i64)
        .bind(2_i64)
        .bind("100")
        .bind("Alice")
        .bind(1_710_000_000_i64)
        .bind(first_content)
        .execute(&pool)
        .await
        .expect("insert first item");
        sqlx::query(
            r#"
            INSERT INTO items (id, source_id, external_id, author, published_at, content_zstd)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(12_i64)
        .bind(4_i64)
        .bind("101")
        .bind(Option::<String>::None)
        .bind(1_710_000_100_i64)
        .bind(second_content)
        .execute(&pool)
        .await
        .expect("insert second item");

        let corpus = load_corpus_messages(&pool, &[2, 4], 1_700_000_000_i64, 1_800_000_000_i64)
            .await
            .expect("load live corpus");

        assert_eq!(corpus.len(), 2);
        assert_eq!(corpus[0].r#ref, "s2-i11");
        assert_eq!(corpus[1].r#ref, "s4-i12");
    }
}
