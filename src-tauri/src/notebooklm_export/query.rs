use std::collections::{HashMap, HashSet};

use sqlx::FromRow;

use crate::compression::decompress_text;
use crate::error::{AppError, AppResult};
use crate::media::decode_media_metadata;
use crate::notebooklm_export::links::detect_urls;
use crate::notebooklm_export::media::render_media_placeholders;
use crate::notebooklm_export::model::{NotebookLmExportMessage, NotebookLmExportSource};

#[derive(FromRow)]
struct SourceRow {
    id: i64,
    source_type: String,
    source_subtype: Option<String>,
    external_id: String,
    title: Option<String>,
}

#[derive(FromRow)]
struct ItemRow {
    id: i64,
    source_id: i64,
    external_id: String,
    author: Option<String>,
    published_at: i64,
    content_zstd: Option<Vec<u8>>,
    content_kind: String,
    has_media: bool,
    media_kind: Option<String>,
    media_metadata_zstd: Option<Vec<u8>>,
    reply_to_msg_id: Option<i64>,
    reply_to_peer_kind: Option<String>,
    reply_to_peer_id: Option<String>,
    reply_to_top_id: Option<i64>,
    reaction_count: Option<i64>,
    forum_topic_id: Option<i64>,
    forum_topic_title: Option<String>,
    forum_topic_top_message_id: Option<i64>,
}

#[derive(FromRow)]
struct ReplyLookupRow {
    external_id: String,
    author: Option<String>,
    content_zstd: Option<Vec<u8>>,
    has_media: bool,
    media_kind: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ReplyContext {
    author: Option<String>,
    snippet: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) enum ExportLoaderSelection {
    ArchiveReadModel {
        model_version: i64,
    },
    ItemsPath {
        reason: ArchiveReadinessFallbackReason,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) enum ArchiveReadinessFallbackReason {
    MissingState,
    NeverBuilt,
    Building,
    Stale,
    Failed,
    OldModelVersion { found: i64, current: i64 },
}

pub(crate) async fn load_export_source(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source_id: i64,
) -> AppResult<NotebookLmExportSource> {
    let source: SourceRow = sqlx::query_as(
        r#"
        SELECT id, source_type, source_subtype, external_id, title
        FROM sources
        WHERE id = ?
        "#,
    )
    .bind(source_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| AppError::not_found(format!("Source {source_id} not found")))?;

    if source.source_type != "telegram" {
        return Err(AppError::validation(format!(
            "Source {source_id} is not a Telegram source"
        )));
    }
    let source_subtype = source
        .source_subtype
        .ok_or_else(|| AppError::validation(format!("Source {source_id} has no source_subtype")))?;
    let source_subtype = crate::sources::TelegramSourceKind::from_source_subtype(&source_subtype)?;

    Ok(NotebookLmExportSource {
        id: source.id,
        source_type: source.source_type,
        source_subtype: source_subtype.as_str().to_string(),
        external_id: source.external_id,
        title: source.title,
    })
}

#[allow(dead_code)]
pub(crate) async fn select_notebooklm_export_loader(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source_id: i64,
) -> AppResult<ExportLoaderSelection> {
    let Some(state) = crate::archive_read_model::load_source_state(pool, source_id).await? else {
        return Ok(ExportLoaderSelection::ItemsPath {
            reason: ArchiveReadinessFallbackReason::MissingState,
        });
    };

    if state.status == crate::archive_read_model::STATUS_READY
        && state.model_version == crate::archive_read_model::ARCHIVE_READ_MODEL_VERSION
    {
        return Ok(ExportLoaderSelection::ArchiveReadModel {
            model_version: state.model_version,
        });
    }

    let reason = if state.status == crate::archive_read_model::STATUS_READY {
        ArchiveReadinessFallbackReason::OldModelVersion {
            found: state.model_version,
            current: crate::archive_read_model::ARCHIVE_READ_MODEL_VERSION,
        }
    } else {
        match state.status.as_str() {
            crate::archive_read_model::STATUS_NEVER_BUILT => {
                ArchiveReadinessFallbackReason::NeverBuilt
            }
            crate::archive_read_model::STATUS_BUILDING => ArchiveReadinessFallbackReason::Building,
            crate::archive_read_model::STATUS_STALE => ArchiveReadinessFallbackReason::Stale,
            crate::archive_read_model::STATUS_FAILED => ArchiveReadinessFallbackReason::Failed,
            _ => ArchiveReadinessFallbackReason::Failed,
        }
    };

    Ok(ExportLoaderSelection::ItemsPath { reason })
}

pub(crate) async fn load_export_messages_from_items_path(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source_id: i64,
    period_from: Option<i64>,
    period_to: Option<i64>,
) -> AppResult<Vec<NotebookLmExportMessage>> {
    let rows: Vec<ItemRow> = match (period_from, period_to) {
        (Some(from), Some(to)) => {
            let sql = base_query(
                "items.source_id = ? AND items.published_at >= ? AND items.published_at <= ?",
            );
            sqlx::query_as(&sql)
                .bind(source_id)
                .bind(from)
                .bind(to)
                .fetch_all(pool)
                .await
        }
        (Some(from), None) => {
            let sql = base_query("items.source_id = ? AND items.published_at >= ?");
            sqlx::query_as(&sql)
                .bind(source_id)
                .bind(from)
                .fetch_all(pool)
                .await
        }
        (None, Some(to)) => {
            let sql = base_query("items.source_id = ? AND items.published_at <= ?");
            sqlx::query_as(&sql)
                .bind(source_id)
                .bind(to)
                .fetch_all(pool)
                .await
        }
        (None, None) => {
            let sql = base_query("items.source_id = ?");
            sqlx::query_as(&sql).bind(source_id).fetch_all(pool).await
        }
    }
    .map_err(|e| e.to_string())?;

    let reply_contexts = load_reply_contexts_from_items_path(pool, source_id, &rows).await?;

    rows.into_iter()
        .map(|row| {
            let text = row
                .content_zstd
                .as_deref()
                .map(decompress_text)
                .transpose()?;
            let urls = text.as_deref().map(detect_urls).unwrap_or_default();
            let media_metadata = decode_media_metadata(row.media_metadata_zstd.as_deref())?;
            let media_placeholders =
                render_media_placeholders(row.media_kind.as_deref(), &media_metadata);
            let reply_context = row
                .reply_to_msg_id
                .and_then(|reply_to_msg_id| reply_contexts.get(&reply_to_msg_id));

            Ok(NotebookLmExportMessage {
                item_id: row.id,
                source_id: row.source_id,
                external_id: row.external_id,
                author: row.author,
                published_at: row.published_at,
                text,
                content_kind: row.content_kind,
                has_media: row.has_media,
                media_kind: row.media_kind,
                media_metadata,
                media_placeholders,
                urls,
                reply_to_msg_id: row.reply_to_msg_id,
                reply_to_author: reply_context.and_then(|context| context.author.clone()),
                reply_to_snippet: row.reply_to_msg_id.map(|_| {
                    reply_context
                        .map(|context| context.snippet.clone())
                        .unwrap_or_else(|| "Original message unavailable".to_string())
                }),
                reply_to_peer_kind: row.reply_to_peer_kind,
                reply_to_peer_id: row.reply_to_peer_id,
                reply_to_top_id: row.reply_to_top_id,
                reaction_count: row.reaction_count,
                forum_topic_id: row.forum_topic_id,
                forum_topic_title: row.forum_topic_title,
                forum_topic_top_message_id: row.forum_topic_top_message_id,
            })
        })
        .collect::<Result<Vec<_>, String>>()
        .map_err(AppError::from)
}

pub(crate) async fn load_export_messages(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source_id: i64,
    period_from: Option<i64>,
    period_to: Option<i64>,
) -> AppResult<Vec<NotebookLmExportMessage>> {
    load_export_messages_from_items_path(pool, source_id, period_from, period_to).await
}

async fn load_reply_contexts_from_items_path(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source_id: i64,
    rows: &[ItemRow],
) -> AppResult<HashMap<i64, ReplyContext>> {
    let mut reply_ids = rows
        .iter()
        .filter_map(|row| row.reply_to_msg_id)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    reply_ids.sort_unstable();

    let mut contexts = HashMap::new();
    for chunk in reply_ids.chunks(500) {
        if chunk.is_empty() {
            continue;
        }

        let placeholders = std::iter::repeat_n("?", chunk.len())
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!(
            r#"
            SELECT external_id, author, content_zstd, has_media, media_kind
            FROM items
            WHERE source_id = ? AND external_id IN ({placeholders})
            "#
        );

        let mut query = sqlx::query_as::<_, ReplyLookupRow>(&sql).bind(source_id);
        for reply_id in chunk {
            query = query.bind(reply_id.to_string());
        }

        let lookup_rows = query.fetch_all(pool).await.map_err(|e| e.to_string())?;
        for row in lookup_rows {
            let Ok(reply_id) = row.external_id.parse::<i64>() else {
                continue;
            };
            let snippet = reply_snippet(&row)?;
            contexts.insert(
                reply_id,
                ReplyContext {
                    author: row.author,
                    snippet,
                },
            );
        }
    }

    Ok(contexts)
}

fn reply_snippet(row: &ReplyLookupRow) -> Result<String, String> {
    let text = row
        .content_zstd
        .as_deref()
        .map(decompress_text)
        .transpose()?;

    if let Some(text) = text
        .as_deref()
        .map(str::trim)
        .filter(|text| !text.is_empty())
    {
        return Ok(truncate_snippet(&collapse_whitespace(text), 280));
    }

    if row.has_media {
        return Ok(format!(
            "[Media message: {}]",
            row.media_kind.as_deref().unwrap_or("media")
        ));
    }

    Ok("[Message has no text]".to_string())
}

fn collapse_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn truncate_snippet(value: &str, max_chars: usize) -> String {
    let mut chars = value.chars();
    let snippet = chars.by_ref().take(max_chars).collect::<String>();
    if chars.next().is_some() {
        format!("{snippet}...")
    } else {
        snippet
    }
}

fn base_query(where_clause: &str) -> String {
    format!(
        r#"
    SELECT
        items.id,
        items.source_id,
        items.external_id,
        items.author,
        items.published_at,
        items.content_zstd,
        items.content_kind,
        items.has_media,
        items.media_kind,
        items.media_metadata_zstd,
        items.reply_to_msg_id,
        items.reply_to_peer_kind,
        items.reply_to_peer_id,
        items.reply_to_top_id,
        items.reaction_count,
        forum_topics.topic_id AS forum_topic_id,
        forum_topics.title AS forum_topic_title,
        forum_topics.top_message_id AS forum_topic_top_message_id
    FROM items
    LEFT JOIN item_topic_memberships AS memberships
      ON memberships.item_id = items.id
    LEFT JOIN telegram_forum_topics AS forum_topics
      ON forum_topics.source_id = memberships.source_id
     AND forum_topics.topic_id = memberships.topic_id
    WHERE {where_clause}
    ORDER BY items.published_at ASC, items.id ASC
"#
    )
}

#[cfg(test)]
mod tests {
    use super::{
        load_export_messages, load_export_source, select_notebooklm_export_loader,
        ArchiveReadinessFallbackReason, ExportLoaderSelection,
    };
    use crate::compression::compress_text;

    async fn export_pool() -> sqlx::SqlitePool {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");
        sqlx::query(
            r#"
            CREATE TABLE sources (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                source_type TEXT NOT NULL,
                source_subtype TEXT,
                external_id TEXT NOT NULL,
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
                item_kind TEXT NOT NULL DEFAULT 'telegram_message',
                author TEXT,
                published_at INTEGER NOT NULL,
                ingested_at INTEGER NOT NULL DEFAULT 0,
                content_zstd BLOB,
                raw_data_zstd BLOB,
                content_kind TEXT NOT NULL,
                has_media INTEGER NOT NULL,
                media_kind TEXT,
                media_metadata_zstd BLOB,
                reply_to_msg_id INTEGER,
                reply_to_peer_kind TEXT,
                reply_to_peer_id TEXT,
                reply_to_top_id INTEGER,
                reaction_count INTEGER
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create items");
        sqlx::query(
            r#"
            CREATE TABLE telegram_forum_topics (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                source_id INTEGER NOT NULL,
                topic_id INTEGER NOT NULL,
                top_message_id INTEGER NOT NULL,
                title TEXT,
                is_deleted INTEGER NOT NULL DEFAULT 0
            )
            "#,
        )
        .execute(&pool)
        .await
        .expect("create telegram_forum_topics");
        sqlx::query(
            r#"
            CREATE UNIQUE INDEX idx_telegram_forum_topics_source_topic
            ON telegram_forum_topics(source_id, topic_id)
            "#,
        )
        .execute(&pool)
        .await
        .expect("create telegram_forum_topics source/topic index");
        sqlx::raw_sql(
            crate::migrations::telegram_item_native_identity::TELEGRAM_MESSAGES_SCHEMA_SQL,
        )
        .execute(&pool)
        .await
        .expect("create telegram_messages");
        seed_materialized_topic_schema(&pool).await;
        crate::sources::test_support::create_archive_read_model_tables(&pool).await;
        pool
    }

    async fn seed_materialized_topic_schema(pool: &sqlx::SqlitePool) {
        crate::sources::test_support::create_topic_membership_tables(pool).await;
    }

    async fn seed_export_source(pool: &sqlx::SqlitePool) {
        sqlx::query(
            "INSERT INTO sources (id, source_type, source_subtype, external_id, title)
             VALUES (1, 'telegram', 'supergroup', '12345', 'Forum')",
        )
        .execute(pool)
        .await
        .expect("seed export source");
    }

    async fn seed_archive_state(pool: &sqlx::SqlitePool, status: &str, model_version: i64) {
        sqlx::query(
            "INSERT INTO archive_read_model_state (
                source_id, model_version, status, built_at, item_count, row_count
             ) VALUES (1, ?, ?, 100, 0, 0)",
        )
        .bind(model_version)
        .bind(status)
        .execute(pool)
        .await
        .expect("seed archive state");
    }

    #[tokio::test]
    async fn notebooklm_export_loader_selection_reports_all_fallback_reasons() {
        let cases = [
            (
                "never_built",
                crate::archive_read_model::ARCHIVE_READ_MODEL_VERSION,
                ArchiveReadinessFallbackReason::NeverBuilt,
            ),
            (
                "building",
                crate::archive_read_model::ARCHIVE_READ_MODEL_VERSION,
                ArchiveReadinessFallbackReason::Building,
            ),
            (
                "stale",
                crate::archive_read_model::ARCHIVE_READ_MODEL_VERSION,
                ArchiveReadinessFallbackReason::Stale,
            ),
            (
                "failed",
                crate::archive_read_model::ARCHIVE_READ_MODEL_VERSION,
                ArchiveReadinessFallbackReason::Failed,
            ),
        ];

        for (status, version, expected_reason) in cases {
            let pool = export_pool().await;
            seed_export_source(&pool).await;
            seed_archive_state(&pool, status, version).await;

            let selection = select_notebooklm_export_loader(&pool, 1)
                .await
                .expect("select loader");

            assert_eq!(
                selection,
                ExportLoaderSelection::ItemsPath {
                    reason: expected_reason
                },
                "unexpected selection for {status}"
            );
        }
    }

    #[tokio::test]
    async fn notebooklm_export_loader_selection_reports_missing_and_old_version() {
        let pool = export_pool().await;
        seed_export_source(&pool).await;

        assert_eq!(
            select_notebooklm_export_loader(&pool, 1)
                .await
                .expect("select missing state"),
            ExportLoaderSelection::ItemsPath {
                reason: ArchiveReadinessFallbackReason::MissingState
            }
        );

        seed_archive_state(
            &pool,
            crate::archive_read_model::STATUS_READY,
            crate::archive_read_model::ARCHIVE_READ_MODEL_VERSION - 1,
        )
        .await;

        assert_eq!(
            select_notebooklm_export_loader(&pool, 1)
                .await
                .expect("select old state"),
            ExportLoaderSelection::ItemsPath {
                reason: ArchiveReadinessFallbackReason::OldModelVersion {
                    found: crate::archive_read_model::ARCHIVE_READ_MODEL_VERSION - 1,
                    current: crate::archive_read_model::ARCHIVE_READ_MODEL_VERSION,
                }
            }
        );
    }

    #[tokio::test]
    async fn notebooklm_export_loader_selection_uses_archive_for_ready_current_state() {
        let pool = export_pool().await;
        seed_export_source(&pool).await;
        seed_archive_state(
            &pool,
            crate::archive_read_model::STATUS_READY,
            crate::archive_read_model::ARCHIVE_READ_MODEL_VERSION,
        )
        .await;

        assert_eq!(
            select_notebooklm_export_loader(&pool, 1)
                .await
                .expect("select ready state"),
            ExportLoaderSelection::ArchiveReadModel {
                model_version: crate::archive_read_model::ARCHIVE_READ_MODEL_VERSION,
            }
        );
    }

    #[tokio::test]
    async fn load_export_source_rejects_non_telegram_before_message_loader_selection() {
        let pool = export_pool().await;
        sqlx::query(
            "INSERT INTO sources (id, source_type, source_subtype, external_id, title)
             VALUES (2, 'youtube', 'video', 'video-id', 'Video')",
        )
        .execute(&pool)
        .await
        .expect("seed youtube source");
        sqlx::query(
            "INSERT INTO archive_read_model_state (
                source_id, model_version, status, built_at, item_count, row_count
             ) VALUES (2, ?, 'ready', 100, 0, 0)",
        )
        .bind(crate::archive_read_model::ARCHIVE_READ_MODEL_VERSION)
        .execute(&pool)
        .await
        .expect("seed ready youtube archive state");

        let error = load_export_source(&pool, 2)
            .await
            .expect_err("youtube source is rejected before message loading");

        assert!(error.to_string().contains("is not a Telegram source"));
    }

    #[tokio::test]
    async fn load_export_source_uses_canonical_subtype_not_legacy_kind() {
        let pool = export_pool().await;
        sqlx::query(
            r#"
            INSERT INTO sources (
                id, source_type, source_subtype, external_id, title
            )
            VALUES (?, 'telegram', 'supergroup', ?, ?)
            "#,
        )
        .bind(7_i64)
        .bind("12345")
        .bind("Forum source")
        .execute(&pool)
        .await
        .expect("insert source");

        let source = load_export_source(&pool, 7)
            .await
            .expect("load export source");

        assert_eq!(source.source_subtype, "supergroup");
    }

    #[tokio::test]
    async fn load_export_messages_adds_local_reply_context_outside_period() {
        let pool = export_pool().await;
        sqlx::query(
            r#"
            INSERT INTO items (
                source_id,
                external_id,
                author,
                published_at,
                content_zstd,
                content_kind,
                has_media
            )
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(1_i64)
        .bind("10")
        .bind("Bob")
        .bind(10_i64)
        .bind(compress_text("Original reply target").expect("compress original"))
        .bind("text_only")
        .bind(0_i64)
        .execute(&pool)
        .await
        .expect("insert original");

        sqlx::query(
            r#"
            INSERT INTO items (
                source_id,
                external_id,
                author,
                published_at,
                content_zstd,
                content_kind,
                has_media,
                reply_to_msg_id,
                reply_to_peer_kind,
                reply_to_peer_id,
                reply_to_top_id,
                reaction_count
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(1_i64)
        .bind("20")
        .bind("Ada")
        .bind(100_i64)
        .bind(compress_text("Reply message").expect("compress reply"))
        .bind("text_only")
        .bind(0_i64)
        .bind(10_i64)
        .bind("channel")
        .bind("42")
        .bind(10_i64)
        .bind(3_i64)
        .execute(&pool)
        .await
        .expect("insert reply");

        let messages = load_export_messages(&pool, 1, Some(50), None)
            .await
            .expect("load export messages");

        assert_eq!(messages.len(), 1);
        let message = &messages[0];
        assert_eq!(message.external_id, "20");
        assert_eq!(message.reply_to_msg_id, Some(10));
        assert_eq!(message.reply_to_author.as_deref(), Some("Bob"));
        assert_eq!(
            message.reply_to_snippet.as_deref(),
            Some("Original reply target")
        );
        assert_eq!(message.reply_to_peer_kind.as_deref(), Some("channel"));
        assert_eq!(message.reply_to_peer_id.as_deref(), Some("42"));
        assert_eq!(message.reply_to_top_id, Some(10));
        assert_eq!(message.reaction_count, Some(3));
    }

    #[tokio::test]
    async fn load_export_messages_attaches_topic_metadata_for_reply_and_root_messages() {
        let pool = export_pool().await;
        sqlx::query(
            "INSERT INTO sources (id, source_type, source_subtype, external_id, title)
             VALUES (1, 'telegram', 'supergroup', '12345', 'Forum')",
        )
        .execute(&pool)
        .await
        .expect("seed source");

        sqlx::query(
            r#"
            INSERT INTO telegram_forum_topics (
                source_id,
                topic_id,
                top_message_id,
                title,
                is_deleted
            )
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(1_i64)
        .bind(200_i64)
        .bind(700_i64)
        .bind("Roadmap")
        .bind(0_i64)
        .execute(&pool)
        .await
        .expect("insert forum topic");

        sqlx::query(
            r#"
            INSERT INTO items (
                source_id,
                external_id,
                author,
                published_at,
                content_zstd,
                content_kind,
                has_media,
                reply_to_top_id
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(1_i64)
        .bind("701")
        .bind("Ada")
        .bind(100_i64)
        .bind(compress_text("Reply in topic").expect("compress reply"))
        .bind("text_only")
        .bind(0_i64)
        .bind(200_i64)
        .execute(&pool)
        .await
        .expect("insert topic reply");

        sqlx::query(
            r#"
            INSERT INTO items (
                source_id,
                external_id,
                author,
                published_at,
                content_zstd,
                content_kind,
                has_media,
                reply_to_msg_id
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(1_i64)
        .bind("702")
        .bind("Edsger")
        .bind(101_i64)
        .bind(compress_text("Top-level topic message").expect("compress topic fallback"))
        .bind("text_only")
        .bind(0_i64)
        .bind(200_i64)
        .execute(&pool)
        .await
        .expect("insert topic reply header fallback");

        sqlx::query(
            r#"
            INSERT INTO items (
                source_id,
                external_id,
                author,
                published_at,
                content_zstd,
                content_kind,
                has_media
            )
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(1_i64)
        .bind("not-numeric-root")
        .bind("Bob")
        .bind(102_i64)
        .bind(compress_text("Root topic message").expect("compress root"))
        .bind("text_only")
        .bind(0_i64)
        .execute(&pool)
        .await
        .expect("insert topic root");
        let root_item_id: i64 =
            sqlx::query_scalar("SELECT id FROM items WHERE external_id = 'not-numeric-root'")
                .fetch_one(&pool)
                .await
                .expect("load root item id");
        sqlx::query(
            "INSERT INTO telegram_messages (item_id, source_id, history_peer_kind, history_peer_id, telegram_message_id)
             VALUES (?, 1, 'channel', 12345, 700)",
        )
        .bind(root_item_id)
        .execute(&pool)
        .await
        .expect("insert typed message identity");
        for external_id in ["701", "702", "not-numeric-root"] {
            let item_id: i64 = sqlx::query_scalar("SELECT id FROM items WHERE external_id = ?")
                .bind(external_id)
                .fetch_one(&pool)
                .await
                .expect("load item id for membership");
            sqlx::query(
                "INSERT INTO item_topic_memberships (
                    item_id, source_id, topic_id, match_kind, resolver_version
                 ) VALUES (?, 1, 200, 'reply_to_top_id', 1)",
            )
            .bind(item_id)
            .execute(&pool)
            .await
            .expect("insert topic membership");
        }

        let messages = load_export_messages(&pool, 1, None, None)
            .await
            .expect("load export messages");

        assert_eq!(messages.len(), 3);
        assert!(messages
            .iter()
            .all(|message| message.forum_topic_id == Some(200)));
        assert!(messages
            .iter()
            .all(|message| message.forum_topic_title.as_deref() == Some("Roadmap")));
        assert!(messages
            .iter()
            .all(|message| message.forum_topic_top_message_id == Some(700)));
    }

    #[tokio::test]
    async fn load_export_messages_attaches_general_topic_when_topic_header_is_missing() {
        let pool = export_pool().await;

        sqlx::query(
            "INSERT INTO sources (id, source_type, source_subtype, external_id, title)
             VALUES (1, 'telegram', 'supergroup', '12345', 'Forum')",
        )
        .execute(&pool)
        .await
        .expect("seed source");

        sqlx::query(
            r#"
            INSERT INTO telegram_forum_topics (
                source_id,
                topic_id,
                top_message_id,
                title,
                is_deleted
            )
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(1_i64)
        .bind(1_i64)
        .bind(1_i64)
        .bind("General")
        .bind(0_i64)
        .execute(&pool)
        .await
        .expect("insert general topic");

        sqlx::query(
            r#"
            INSERT INTO items (
                source_id,
                external_id,
                author,
                published_at,
                content_zstd,
                content_kind,
                has_media
            )
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(1_i64)
        .bind("999")
        .bind("Ada")
        .bind(100_i64)
        .bind(compress_text("General message").expect("compress message"))
        .bind("text_only")
        .bind(0_i64)
        .execute(&pool)
        .await
        .expect("insert general message");
        let item_id: i64 = sqlx::query_scalar("SELECT id FROM items WHERE external_id = '999'")
            .fetch_one(&pool)
            .await
            .expect("load general item id");
        sqlx::query(
            "INSERT INTO item_topic_memberships (
                item_id, source_id, topic_id, match_kind, resolver_version
             ) VALUES (?, 1, 1, 'general_fallback', 1)",
        )
        .bind(item_id)
        .execute(&pool)
        .await
        .expect("insert general membership");

        let messages = load_export_messages(&pool, 1, None, None)
            .await
            .expect("load export messages");

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].forum_topic_id, Some(1));
        assert_eq!(messages[0].forum_topic_title.as_deref(), Some("General"));
        assert_eq!(messages[0].forum_topic_top_message_id, Some(1));
    }

    #[tokio::test]
    async fn load_export_messages_does_not_root_match_non_numeric_external_ids() {
        let pool = export_pool().await;

        sqlx::query(
            r#"
            INSERT INTO telegram_forum_topics (
                source_id,
                topic_id,
                top_message_id,
                title,
                is_deleted
            )
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(1_i64)
        .bind(200_i64)
        .bind(700_i64)
        .bind("Roadmap")
        .bind(0_i64)
        .execute(&pool)
        .await
        .expect("insert forum topic");

        sqlx::query(
            r#"
            INSERT INTO items (
                source_id,
                external_id,
                author,
                published_at,
                content_zstd,
                content_kind,
                has_media
            )
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(1_i64)
        .bind("700a")
        .bind("Ada")
        .bind(100_i64)
        .bind(compress_text("Looks numeric but is not").expect("compress message"))
        .bind("text_only")
        .bind(0_i64)
        .execute(&pool)
        .await
        .expect("insert message");

        let messages = load_export_messages(&pool, 1, None, None)
            .await
            .expect("load export messages");

        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].forum_topic_id, None);
        assert_eq!(messages[0].forum_topic_title, None);
        assert_eq!(messages[0].forum_topic_top_message_id, None);
    }

    #[tokio::test]
    async fn load_export_messages_reads_materialized_topic_memberships() {
        let pool = export_pool().await;
        sqlx::query(
            "INSERT INTO sources (id, source_type, source_subtype, external_id, title)
             VALUES (1, 'telegram', 'supergroup', '12345', 'Forum')",
        )
        .execute(&pool)
        .await
        .expect("seed source");
        sqlx::query(
            "INSERT INTO telegram_forum_topics (
                source_id, topic_id, top_message_id, title, is_deleted
             ) VALUES (1, 200, 700, 'Roadmap', 0)",
        )
        .execute(&pool)
        .await
        .expect("seed topic");
        sqlx::query(
            "INSERT INTO items (
                id, source_id, external_id, author, published_at, content_zstd,
                content_kind, has_media
             ) VALUES (1, 1, '701', 'Ada', 100, ?, 'text_only', 0)",
        )
        .bind(compress_text("Reply in topic").expect("compress"))
        .execute(&pool)
        .await
        .expect("seed item");
        sqlx::query(
            "INSERT INTO item_topic_memberships (
                item_id, source_id, topic_id, match_kind, resolver_version
             ) VALUES (1, 1, 200, 'reply_to_top_id', 1)",
        )
        .execute(&pool)
        .await
        .expect("seed membership");

        let messages = load_export_messages(&pool, 1, None, None)
            .await
            .expect("load export messages");

        assert_eq!(messages[0].forum_topic_id, Some(200));
        assert_eq!(messages[0].forum_topic_title.as_deref(), Some("Roadmap"));
    }
}
