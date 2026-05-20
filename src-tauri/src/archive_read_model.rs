#![allow(dead_code)]

use crate::error::{AppError, AppResult};
use crate::readiness::{is_ready_current, mark_failed, mark_stale, ModelVersion, ReadinessStatus};
use crate::sources::{ForumTopicFilter, StoredItemRow};
use crate::time::now_secs;
use sqlx::{FromRow, SqlitePool};

pub(crate) const ARCHIVE_READ_MODEL_VERSION: ModelVersion = 1;

pub(crate) const ARCHIVE_READ_MODEL_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS archive_read_model_state (
    source_id INTEGER PRIMARY KEY,
    model_version INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'never_built',
    built_at INTEGER,
    item_count INTEGER NOT NULL DEFAULT 0,
    row_count INTEGER NOT NULL DEFAULT 0,
    last_error TEXT,
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    FOREIGN KEY(source_id) REFERENCES sources(id) ON DELETE CASCADE,
    CHECK (status IN ('never_built', 'building', 'ready', 'stale', 'failed')),
    CHECK (item_count >= 0),
    CHECK (row_count >= 0)
);

CREATE TABLE IF NOT EXISTS archive_read_items (
    source_id INTEGER NOT NULL,
    item_id INTEGER NOT NULL,
    ref TEXT NOT NULL,
    external_id TEXT NOT NULL,
    item_kind TEXT NOT NULL,
    author TEXT,
    published_at INTEGER NOT NULL,
    content_kind TEXT NOT NULL,
    has_media INTEGER NOT NULL DEFAULT 0,
    media_kind TEXT,
    content_zstd BLOB,
    media_metadata_zstd BLOB,
    has_raw_data INTEGER NOT NULL DEFAULT 0,
    forum_topic_id INTEGER,
    forum_topic_title TEXT,
    forum_topic_top_message_id INTEGER,
    reply_to_msg_id INTEGER,
    reply_to_peer_kind TEXT,
    reply_to_peer_id TEXT,
    reply_to_top_id INTEGER,
    reaction_count INTEGER,
    model_version INTEGER NOT NULL,
    built_at INTEGER NOT NULL,
    PRIMARY KEY(source_id, item_id),
    FOREIGN KEY(source_id) REFERENCES sources(id) ON DELETE CASCADE,
    FOREIGN KEY(item_id) REFERENCES items(id) ON DELETE CASCADE,
    CHECK (has_media IN (0, 1)),
    CHECK (has_raw_data IN (0, 1))
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_archive_read_items_ref
    ON archive_read_items(ref);

CREATE INDEX IF NOT EXISTS idx_archive_read_items_source_published
    ON archive_read_items(source_id, published_at DESC, item_id DESC);

CREATE INDEX IF NOT EXISTS idx_archive_read_items_source_topic_published
    ON archive_read_items(source_id, forum_topic_id, published_at DESC, item_id DESC);
"#;

#[derive(Debug, FromRow)]
pub(crate) struct ArchiveReadModelState {
    pub(crate) source_id: i64,
    pub(crate) model_version: i64,
    pub(crate) status: String,
    pub(crate) built_at: Option<i64>,
    pub(crate) item_count: i64,
    pub(crate) row_count: i64,
    pub(crate) last_error: Option<String>,
    pub(crate) updated_at: i64,
}

impl ArchiveReadModelState {
    pub(crate) fn readiness_status(&self) -> Option<ReadinessStatus> {
        ReadinessStatus::parse(&self.status)
    }
}

#[derive(Debug, FromRow)]
pub(crate) struct ArchiveReadItemRow {
    pub(crate) source_id: i64,
    pub(crate) item_id: i64,
    #[sqlx(rename = "ref")]
    pub(crate) ref_: String,
    pub(crate) external_id: String,
    pub(crate) item_kind: String,
    pub(crate) author: Option<String>,
    pub(crate) published_at: i64,
    pub(crate) content_kind: String,
    pub(crate) has_media: bool,
    pub(crate) media_kind: Option<String>,
    pub(crate) content_zstd: Option<Vec<u8>>,
    pub(crate) media_metadata_zstd: Option<Vec<u8>>,
    pub(crate) has_raw_data: bool,
    pub(crate) forum_topic_id: Option<i64>,
    pub(crate) forum_topic_title: Option<String>,
    pub(crate) forum_topic_top_message_id: Option<i64>,
    pub(crate) reply_to_msg_id: Option<i64>,
    pub(crate) reply_to_peer_kind: Option<String>,
    pub(crate) reply_to_peer_id: Option<String>,
    pub(crate) reply_to_top_id: Option<i64>,
    pub(crate) reaction_count: Option<i64>,
    pub(crate) model_version: i64,
    pub(crate) built_at: i64,
}

#[derive(Debug, FromRow)]
struct SourceArchiveItemRow {
    id: i64,
    source_id: i64,
    external_id: String,
    item_kind: String,
    author: Option<String>,
    published_at: i64,
    content_kind: String,
    has_media: bool,
    media_kind: Option<String>,
    content_zstd: Option<Vec<u8>>,
    media_metadata_zstd: Option<Vec<u8>>,
    has_raw_data: bool,
    forum_topic_id: Option<i64>,
    forum_topic_title: Option<String>,
    forum_topic_top_message_id: Option<i64>,
    reply_to_msg_id: Option<i64>,
    reply_to_peer_kind: Option<String>,
    reply_to_peer_id: Option<String>,
    reply_to_top_id: Option<i64>,
    reaction_count: Option<i64>,
}

pub(crate) async fn create_archive_read_model_schema(pool: &SqlitePool) -> AppResult<()> {
    sqlx::raw_sql(ARCHIVE_READ_MODEL_SCHEMA_SQL)
        .execute(pool)
        .await
        .map_err(AppError::database)?;
    Ok(())
}

pub(crate) async fn load_source_state(
    pool: &SqlitePool,
    source_id: i64,
) -> AppResult<Option<ArchiveReadModelState>> {
    sqlx::query_as(
        "SELECT source_id, model_version, status, built_at, item_count, row_count, last_error, updated_at
         FROM archive_read_model_state
         WHERE source_id = ?",
    )
    .bind(source_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)
}

pub(crate) fn is_current_ready_state(state: Option<&ArchiveReadModelState>) -> bool {
    state.is_some_and(|state| {
        state.readiness_status().is_some_and(|status| {
            is_ready_current(status, state.model_version, ARCHIVE_READ_MODEL_VERSION)
        })
    })
}

pub(crate) async fn source_archive_model_is_ready(
    pool: &SqlitePool,
    source_id: i64,
) -> AppResult<bool> {
    let state = load_source_state(pool, source_id).await?;
    Ok(is_current_ready_state(state.as_ref()))
}

pub(crate) async fn rebuild_source(pool: &SqlitePool, source_id: i64) -> AppResult<()> {
    let started_at = now_secs();
    let mut tx = pool.begin().await.map_err(AppError::database)?;
    let result = rebuild_source_in_transaction(&mut tx, source_id, started_at).await;

    match result {
        Ok(()) => {
            tx.commit().await.map_err(AppError::database)?;
            Ok(())
        }
        Err(error) => {
            let _ = tx.rollback().await;
            mark_source_failed(pool, source_id, &error.message).await?;
            Err(error)
        }
    }
}

async fn rebuild_source_in_transaction(
    conn: &mut sqlx::SqliteConnection,
    source_id: i64,
    started_at: i64,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO archive_read_model_state (
            source_id, model_version, status, built_at, item_count, row_count, last_error, updated_at
         ) VALUES (?, ?, ?, NULL, 0, 0, NULL, ?)
         ON CONFLICT(source_id) DO UPDATE SET
            model_version = excluded.model_version,
            status = excluded.status,
            built_at = NULL,
            item_count = 0,
            row_count = 0,
            last_error = NULL,
            updated_at = excluded.updated_at",
    )
    .bind(source_id)
    .bind(ARCHIVE_READ_MODEL_VERSION)
    .bind(ReadinessStatus::Building.as_str())
    .bind(started_at)
    .execute(&mut *conn)
    .await
    .map_err(AppError::database)?;

    sqlx::query("DELETE FROM archive_read_items WHERE source_id = ?")
        .bind(source_id)
        .execute(&mut *conn)
        .await
        .map_err(AppError::database)?;

    let rows: Vec<SourceArchiveItemRow> = sqlx::query_as(
        r#"
        SELECT
            items.id,
            items.source_id,
            items.external_id,
            items.item_kind,
            items.author,
            items.published_at,
            items.content_kind,
            items.has_media,
            items.media_kind,
            items.content_zstd,
            items.media_metadata_zstd,
            items.raw_data_zstd IS NOT NULL AS has_raw_data,
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
        WHERE items.source_id = ?
        ORDER BY items.published_at DESC, items.id DESC
        "#,
    )
    .bind(source_id)
    .fetch_all(&mut *conn)
    .await
    .map_err(AppError::database)?;

    for row in &rows {
        upsert_archive_row_on_connection(conn, row, started_at).await?;
    }

    let row_count = i64::try_from(rows.len()).unwrap_or(i64::MAX);
    sqlx::query(
        "UPDATE archive_read_model_state
         SET status = ?,
             model_version = ?,
             built_at = ?,
             item_count = ?,
             row_count = ?,
             last_error = NULL,
             updated_at = ?
         WHERE source_id = ?",
    )
    .bind(ReadinessStatus::Ready.as_str())
    .bind(ARCHIVE_READ_MODEL_VERSION)
    .bind(started_at)
    .bind(row_count)
    .bind(row_count)
    .bind(started_at)
    .bind(source_id)
    .execute(&mut *conn)
    .await
    .map_err(AppError::database)?;

    Ok(())
}

pub(crate) async fn upsert_item_on_connection(
    conn: &mut sqlx::SqliteConnection,
    item_id: i64,
) -> AppResult<()> {
    let row = load_builder_row_for_item(conn, item_id).await?;
    let source_id = row.source_id;
    let state: Option<ArchiveReadModelState> = sqlx::query_as(
        "SELECT source_id, model_version, status, built_at, item_count, row_count, last_error, updated_at
         FROM archive_read_model_state
         WHERE source_id = ?",
    )
    .bind(source_id)
    .fetch_optional(&mut *conn)
    .await
    .map_err(AppError::database)?;

    if !is_current_ready_state(state.as_ref()) {
        return mark_source_stale_on_connection(conn, source_id).await;
    }

    let built_at = now_secs();
    upsert_archive_row_on_connection(conn, &row, built_at).await?;
    refresh_ready_counts_on_connection(conn, source_id, built_at).await
}

async fn load_builder_row_for_item(
    conn: &mut sqlx::SqliteConnection,
    item_id: i64,
) -> AppResult<SourceArchiveItemRow> {
    sqlx::query_as(
        r#"
        SELECT
            items.id,
            items.source_id,
            items.external_id,
            items.item_kind,
            items.author,
            items.published_at,
            items.content_kind,
            items.has_media,
            items.media_kind,
            items.content_zstd,
            items.media_metadata_zstd,
            items.raw_data_zstd IS NOT NULL AS has_raw_data,
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
        WHERE items.id = ?
        LIMIT 1
        "#,
    )
    .bind(item_id)
    .fetch_one(conn)
    .await
    .map_err(AppError::database)
}

async fn upsert_archive_row_on_connection(
    conn: &mut sqlx::SqliteConnection,
    row: &SourceArchiveItemRow,
    built_at: i64,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO archive_read_items (
            source_id, item_id, ref, external_id, item_kind, author, published_at,
            content_kind, has_media, media_kind, content_zstd, media_metadata_zstd,
            has_raw_data, forum_topic_id, forum_topic_title, forum_topic_top_message_id,
            reply_to_msg_id, reply_to_peer_kind, reply_to_peer_id, reply_to_top_id,
            reaction_count, model_version, built_at
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(source_id, item_id) DO UPDATE SET
            ref = excluded.ref,
            external_id = excluded.external_id,
            item_kind = excluded.item_kind,
            author = excluded.author,
            published_at = excluded.published_at,
            content_kind = excluded.content_kind,
            has_media = excluded.has_media,
            media_kind = excluded.media_kind,
            content_zstd = excluded.content_zstd,
            media_metadata_zstd = excluded.media_metadata_zstd,
            has_raw_data = excluded.has_raw_data,
            forum_topic_id = excluded.forum_topic_id,
            forum_topic_title = excluded.forum_topic_title,
            forum_topic_top_message_id = excluded.forum_topic_top_message_id,
            reply_to_msg_id = excluded.reply_to_msg_id,
            reply_to_peer_kind = excluded.reply_to_peer_kind,
            reply_to_peer_id = excluded.reply_to_peer_id,
            reply_to_top_id = excluded.reply_to_top_id,
            reaction_count = excluded.reaction_count,
            model_version = excluded.model_version,
            built_at = excluded.built_at",
    )
    .bind(row.source_id)
    .bind(row.id)
    .bind(format!("s{}-i{}", row.source_id, row.id))
    .bind(&row.external_id)
    .bind(&row.item_kind)
    .bind(&row.author)
    .bind(row.published_at)
    .bind(&row.content_kind)
    .bind(row.has_media)
    .bind(&row.media_kind)
    .bind(&row.content_zstd)
    .bind(&row.media_metadata_zstd)
    .bind(row.has_raw_data)
    .bind(row.forum_topic_id)
    .bind(&row.forum_topic_title)
    .bind(row.forum_topic_top_message_id)
    .bind(row.reply_to_msg_id)
    .bind(&row.reply_to_peer_kind)
    .bind(&row.reply_to_peer_id)
    .bind(row.reply_to_top_id)
    .bind(row.reaction_count)
    .bind(ARCHIVE_READ_MODEL_VERSION)
    .bind(built_at)
    .execute(conn)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

async fn refresh_ready_counts_on_connection(
    conn: &mut sqlx::SqliteConnection,
    source_id: i64,
    built_at: i64,
) -> AppResult<()> {
    let row_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM archive_read_items
         WHERE source_id = ? AND model_version = ?",
    )
    .bind(source_id)
    .bind(ARCHIVE_READ_MODEL_VERSION)
    .fetch_one(&mut *conn)
    .await
    .map_err(AppError::database)?;

    sqlx::query(
        "UPDATE archive_read_model_state
         SET status = ?,
             model_version = ?,
             built_at = ?,
             item_count = ?,
             row_count = ?,
             last_error = NULL,
             updated_at = ?
         WHERE source_id = ?",
    )
    .bind(ReadinessStatus::Ready.as_str())
    .bind(ARCHIVE_READ_MODEL_VERSION)
    .bind(built_at)
    .bind(row_count)
    .bind(row_count)
    .bind(built_at)
    .bind(source_id)
    .execute(conn)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

pub(crate) async fn mark_source_stale(pool: &SqlitePool, source_id: i64) -> AppResult<()> {
    let mut conn = pool.acquire().await.map_err(AppError::database)?;
    mark_source_stale_on_connection(&mut conn, source_id).await
}

pub(crate) async fn mark_source_stale_on_connection(
    conn: &mut sqlx::SqliteConnection,
    source_id: i64,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO archive_read_model_state (
            source_id, model_version, status, updated_at
         ) VALUES (?, ?, ?, strftime('%s','now'))
         ON CONFLICT(source_id) DO UPDATE SET
            status = CASE
                WHEN archive_read_model_state.status = ? THEN ?
                ELSE archive_read_model_state.status
            END,
            model_version = excluded.model_version,
            updated_at = strftime('%s','now')",
    )
    .bind(source_id)
    .bind(ARCHIVE_READ_MODEL_VERSION)
    .bind(ReadinessStatus::Stale.as_str())
    .bind(ReadinessStatus::Ready.as_str())
    .bind(mark_stale(ReadinessStatus::Ready).as_str())
    .execute(conn)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

pub(crate) async fn mark_source_failed(
    pool: &SqlitePool,
    source_id: i64,
    last_error: &str,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO archive_read_model_state (
            source_id, model_version, status, last_error, updated_at
         ) VALUES (?, ?, ?, ?, strftime('%s','now'))
         ON CONFLICT(source_id) DO UPDATE SET
            model_version = excluded.model_version,
            status = excluded.status,
            last_error = excluded.last_error,
            updated_at = strftime('%s','now')",
    )
    .bind(source_id)
    .bind(ARCHIVE_READ_MODEL_VERSION)
    .bind(mark_failed().as_str())
    .bind(last_error)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

pub(crate) async fn load_item_rows_from_archive(
    pool: &SqlitePool,
    source_id: i64,
    limit: i64,
    before_published_at: Option<i64>,
    topic_filter: Option<ForumTopicFilter>,
    around_item_id: Option<i64>,
) -> AppResult<Vec<StoredItemRow>> {
    let around_published_at = if let Some(item_id) = around_item_id {
        sqlx::query_scalar::<_, i64>(
            "SELECT published_at
             FROM archive_read_items
             WHERE source_id = ? AND item_id = ? AND model_version = ?
             LIMIT 1",
        )
        .bind(source_id)
        .bind(item_id)
        .bind(ARCHIVE_READ_MODEL_VERSION)
        .fetch_optional(pool)
        .await
        .map_err(AppError::database)?
    } else {
        None
    };

    let state = crate::topic_memberships::load_topic_resolution_state(pool, source_id).await?;
    let is_ready = crate::topic_memberships::is_ready_current_state(state.as_ref());
    if matches!(topic_filter.as_ref(), Some(ForumTopicFilter::Uncategorized)) && !is_ready {
        return Ok(Vec::new());
    }

    let mut sql = String::from(
        r#"
        SELECT
            item_id AS id,
            source_id,
            external_id,
            item_kind,
            author,
            published_at,
            content_kind,
            has_media,
            media_kind,
            content_zstd,
            media_metadata_zstd,
            has_raw_data,
            reply_to_msg_id,
            reply_to_peer_kind,
            reply_to_peer_id,
            reply_to_top_id,
            reaction_count,
            forum_topic_id,
            forum_topic_title,
            forum_topic_top_message_id
        FROM archive_read_items
        WHERE source_id = ?
          AND model_version = ?
        "#,
    );

    if before_published_at.is_some() {
        sql.push_str(" AND published_at < ?");
    } else if around_published_at.is_some() {
        sql.push_str(" AND published_at <= ?");
    }

    match topic_filter.as_ref() {
        Some(ForumTopicFilter::Topic { .. }) => {
            sql.push_str(" AND forum_topic_id = ?");
        }
        Some(ForumTopicFilter::Uncategorized) => {
            sql.push_str(" AND forum_topic_id IS NULL");
        }
        None => {}
    }

    sql.push_str(" ORDER BY published_at DESC LIMIT ?");

    let mut query = sqlx::query_as::<_, StoredItemRow>(&sql)
        .bind(source_id)
        .bind(ARCHIVE_READ_MODEL_VERSION);
    if let Some(before) = before_published_at {
        query = query.bind(before);
    } else if let Some(around) = around_published_at {
        query = query.bind(around);
    }
    if let Some(ForumTopicFilter::Topic { topic_id }) = topic_filter.as_ref() {
        query = query.bind(*topic_id);
    }

    query
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(AppError::database)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compression::compress_text;
    use crate::sources::test_support::{
        create_analysis_documents_table, create_archive_read_model_tables,
        memory_pool_with_source_items_and_topics,
    };

    #[tokio::test]
    async fn create_schema_adds_state_and_item_tables() {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_archive_read_model_tables(&pool).await;

        for table in ["archive_read_model_state", "archive_read_items"] {
            let exists: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?",
            )
            .bind(table)
            .fetch_one(&pool)
            .await
            .expect("check table");
            assert_eq!(exists, 1, "missing table {table}");
        }
    }

    #[tokio::test]
    async fn rebuild_source_materializes_archive_fidelity_fields() {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_analysis_documents_table(&pool).await;
        create_archive_read_model_tables(&pool).await;
        seed_archive_source_fixture(&pool).await;

        rebuild_source(&pool, 1).await.expect("rebuild source");

        let row: ArchiveReadItemRow =
            sqlx::query_as("SELECT * FROM archive_read_items WHERE source_id = 1 AND item_id = 2")
                .fetch_one(&pool)
                .await
                .expect("load archive row");

        assert_eq!(row.ref_, "s1-i2");
        assert_eq!(row.external_id, "701");
        assert_eq!(row.item_kind, "telegram_message");
        assert_eq!(row.forum_topic_id, Some(200));
        assert_eq!(row.forum_topic_title.as_deref(), Some("Roadmap"));
        assert_eq!(row.forum_topic_top_message_id, Some(700));
        assert_eq!(row.reply_to_top_id, Some(200));
        assert_eq!(row.reaction_count, Some(4));
        assert!(row.has_raw_data);
        assert_eq!(row.model_version, ARCHIVE_READ_MODEL_VERSION);

        let state = load_source_state(&pool, 1)
            .await
            .expect("load state")
            .expect("state exists");
        assert_eq!(state.status, ReadinessStatus::Ready.as_str());
        assert_eq!(state.model_version, ARCHIVE_READ_MODEL_VERSION);
        assert_eq!(state.item_count, 2);
        assert_eq!(state.row_count, 2);
        assert!(state.built_at.is_some());
        assert_eq!(state.last_error, None);
    }

    #[tokio::test]
    async fn current_ready_state_rejects_old_model_version() {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_archive_read_model_tables(&pool).await;
        sqlx::query(
            "INSERT OR IGNORE INTO sources (
                id, source_type, source_subtype, external_id, title, is_active, is_member, created_at
             ) VALUES (1, 'telegram', 'supergroup', '12345', 'Forum', 1, 1, 1)",
        )
        .execute(&pool)
        .await
        .expect("seed source");
        sqlx::query(
            "INSERT INTO archive_read_model_state (
                source_id, model_version, status, built_at, item_count, row_count
             ) VALUES (1, ?, 'ready', 100, 1, 1)",
        )
        .bind(ARCHIVE_READ_MODEL_VERSION - 1)
        .execute(&pool)
        .await
        .expect("seed old state");

        assert!(!source_archive_model_is_ready(&pool, 1)
            .await
            .expect("check readiness"));
    }

    async fn seed_archive_source_fixture(pool: &sqlx::SqlitePool) {
        sqlx::query(
            "INSERT OR IGNORE INTO sources (
                id, source_type, source_subtype, external_id, title, is_active, is_member, created_at
             ) VALUES (1, 'telegram', 'supergroup', '12345', 'Forum', 1, 1, 1)",
        )
        .execute(pool)
        .await
        .expect("seed source");

        sqlx::query(
            "INSERT INTO telegram_forum_topics (
                source_id, topic_id, top_message_id, title, last_seen_at, updated_at
             ) VALUES (1, 200, 700, 'Roadmap', 100, 100)",
        )
        .execute(pool)
        .await
        .expect("seed topic");

        sqlx::query(
            "INSERT INTO items (
                id, source_id, external_id, item_kind, author, published_at, ingested_at,
                content_zstd, raw_data_zstd, content_kind, has_media, media_kind,
                media_metadata_zstd, reply_to_top_id, reaction_count
             ) VALUES
               (1, 1, '700', 'telegram_message', 'Ada', 100, 100, ?, ?, 'text_only', 0, NULL, NULL, NULL, NULL),
               (2, 1, '701', 'telegram_message', 'Bob', 101, 101, ?, ?, 'text_only', 0, NULL, NULL, 200, 4)",
        )
        .bind(compress_text("Topic root").expect("compress root"))
        .bind(vec![1_u8])
        .bind(compress_text("Topic reply").expect("compress reply"))
        .bind(vec![2_u8])
        .execute(pool)
        .await
        .expect("seed items");

        for item_id in [1_i64, 2_i64] {
            sqlx::query(
                "INSERT INTO item_topic_memberships (
                    item_id, source_id, topic_id, match_kind, resolver_version
                 ) VALUES (?, 1, 200, 'reply_to_top_id', 1)",
            )
            .bind(item_id)
            .execute(pool)
            .await
            .expect("seed membership");
        }
    }
}
