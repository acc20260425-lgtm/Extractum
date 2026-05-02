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
    telegram_source_kind: String,
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

pub(crate) async fn load_export_source(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source_id: i64,
) -> AppResult<NotebookLmExportSource> {
    let source: SourceRow = sqlx::query_as(
        r#"
        SELECT id, source_type, telegram_source_kind, external_id, title
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
    if !matches!(
        source.telegram_source_kind.as_str(),
        "channel" | "supergroup" | "group"
    ) {
        return Err(AppError::validation(format!(
            "Source {source_id} has unsupported Telegram kind '{}'",
            source.telegram_source_kind
        )));
    }

    Ok(NotebookLmExportSource {
        id: source.id,
        source_type: source.source_type,
        telegram_source_kind: source.telegram_source_kind,
        external_id: source.external_id,
        title: source.title,
    })
}

pub(crate) async fn load_export_messages(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source_id: i64,
    period_from: Option<i64>,
    period_to: Option<i64>,
) -> AppResult<Vec<NotebookLmExportMessage>> {
    let rows: Vec<ItemRow> = match (period_from, period_to) {
        (Some(from), Some(to)) => {
            sqlx::query_as(BASE_QUERY_WITH_FROM_TO)
                .bind(source_id)
                .bind(from)
                .bind(to)
                .fetch_all(pool)
                .await
        }
        (Some(from), None) => {
            sqlx::query_as(BASE_QUERY_WITH_FROM)
                .bind(source_id)
                .bind(from)
                .fetch_all(pool)
                .await
        }
        (None, Some(to)) => {
            sqlx::query_as(BASE_QUERY_WITH_TO)
                .bind(source_id)
                .bind(to)
                .fetch_all(pool)
                .await
        }
        (None, None) => {
            sqlx::query_as(BASE_QUERY)
                .bind(source_id)
                .fetch_all(pool)
                .await
        }
    }
    .map_err(|e| e.to_string())?;

    let reply_contexts = load_reply_contexts(pool, source_id, &rows).await?;

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

async fn load_reply_contexts(
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

        let placeholders = std::iter::repeat("?")
            .take(chunk.len())
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

const BASE_QUERY: &str = r#"
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
    LEFT JOIN telegram_forum_topics AS forum_topics
      ON forum_topics.source_id = items.source_id
     AND (
            items.reply_to_top_id = forum_topics.top_message_id
            OR (
                items.reply_to_top_id IS NULL
                AND items.external_id <> ''
                AND items.external_id NOT GLOB '*[^0-9]*'
                AND CAST(items.external_id AS INTEGER) = forum_topics.top_message_id
            )
        )
    WHERE items.source_id = ?
    ORDER BY items.published_at ASC, items.id ASC
"#;
const BASE_QUERY_WITH_FROM: &str = r#"
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
    LEFT JOIN telegram_forum_topics AS forum_topics
      ON forum_topics.source_id = items.source_id
     AND (
            items.reply_to_top_id = forum_topics.top_message_id
            OR (
                items.reply_to_top_id IS NULL
                AND items.external_id <> ''
                AND items.external_id NOT GLOB '*[^0-9]*'
                AND CAST(items.external_id AS INTEGER) = forum_topics.top_message_id
            )
        )
    WHERE items.source_id = ? AND items.published_at >= ?
    ORDER BY items.published_at ASC, items.id ASC
"#;
const BASE_QUERY_WITH_TO: &str = r#"
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
    LEFT JOIN telegram_forum_topics AS forum_topics
      ON forum_topics.source_id = items.source_id
     AND (
            items.reply_to_top_id = forum_topics.top_message_id
            OR (
                items.reply_to_top_id IS NULL
                AND items.external_id <> ''
                AND items.external_id NOT GLOB '*[^0-9]*'
                AND CAST(items.external_id AS INTEGER) = forum_topics.top_message_id
            )
        )
    WHERE items.source_id = ? AND items.published_at <= ?
    ORDER BY items.published_at ASC, items.id ASC
"#;
const BASE_QUERY_WITH_FROM_TO: &str = r#"
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
    LEFT JOIN telegram_forum_topics AS forum_topics
      ON forum_topics.source_id = items.source_id
     AND (
            items.reply_to_top_id = forum_topics.top_message_id
            OR (
                items.reply_to_top_id IS NULL
                AND items.external_id <> ''
                AND items.external_id NOT GLOB '*[^0-9]*'
                AND CAST(items.external_id AS INTEGER) = forum_topics.top_message_id
            )
        )
    WHERE items.source_id = ? AND items.published_at >= ? AND items.published_at <= ?
    ORDER BY items.published_at ASC, items.id ASC
"#;

#[cfg(test)]
mod tests {
    use super::load_export_messages;
    use crate::compression::compress_text;

    async fn export_pool() -> sqlx::SqlitePool {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");
        sqlx::query(
            r#"
            CREATE TABLE items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                source_id INTEGER NOT NULL,
                external_id TEXT NOT NULL,
                author TEXT,
                published_at INTEGER NOT NULL,
                content_zstd BLOB,
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
        pool
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
        .bind(700_i64)
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
                has_media
            )
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(1_i64)
        .bind("700")
        .bind("Bob")
        .bind(101_i64)
        .bind(compress_text("Root topic message").expect("compress root"))
        .bind("text_only")
        .bind(0_i64)
        .execute(&pool)
        .await
        .expect("insert topic root");

        let messages = load_export_messages(&pool, 1, None, None)
            .await
            .expect("load export messages");

        assert_eq!(messages.len(), 2);
        assert!(messages.iter().all(|message| message.forum_topic_id == Some(200)));
        assert!(messages
            .iter()
            .all(|message| message.forum_topic_title.as_deref() == Some("Roadmap")));
        assert!(messages
            .iter()
            .all(|message| message.forum_topic_top_message_id == Some(700)));
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
}
