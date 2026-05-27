#![allow(dead_code)]

use crate::compression::{compress_json_bytes, compress_text};
use crate::error::{AppError, AppResult};
use crate::time::ymd_to_unix_midnight;
use sqlx::{Executor, Sqlite};

pub(crate) const ANALYSIS_DOCUMENTS_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS analysis_documents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    source_id INTEGER NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
    item_id INTEGER REFERENCES items(id) ON DELETE CASCADE,

    document_key TEXT NOT NULL,
    document_kind TEXT NOT NULL,

    source_type TEXT NOT NULL,
    source_subtype TEXT,
    external_id TEXT NOT NULL,

    author TEXT,
    published_at INTEGER NOT NULL,
    document_order INTEGER NOT NULL DEFAULT 0,

    ref TEXT NOT NULL,
    content_zstd BLOB NOT NULL,
    metadata_zstd BLOB,

    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,

    CHECK (document_kind IN (
        'telegram_message',
        'youtube_transcript',
        'youtube_comment',
        'youtube_description'
    )),
    CHECK (source_type IN ('telegram', 'youtube')),
    CHECK (
        (document_kind = 'telegram_message' AND source_type = 'telegram')
        OR
        (document_kind IN (
            'youtube_transcript',
            'youtube_comment',
            'youtube_description'
        ) AND source_type = 'youtube')
    ),
    CHECK (
        (source_type = 'telegram'
            AND COALESCE(source_subtype, '')
                IN ('channel', 'supergroup', 'group'))
        OR
        (source_type = 'youtube' AND COALESCE(source_subtype, '') = 'video')
    ),
    CHECK (
        (document_kind IN (
            'telegram_message',
            'youtube_transcript',
            'youtube_comment'
        ) AND item_id IS NOT NULL)
        OR
        (document_kind = 'youtube_description' AND item_id IS NULL)
    ),
    CHECK (
        (document_kind IN (
            'telegram_message',
            'youtube_transcript',
            'youtube_comment'
        ) AND document_key LIKE 'item:%')
        OR
        (document_kind = 'youtube_description'
            AND document_key = 'youtube:description')
    )
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_analysis_documents_source_key
ON analysis_documents(source_id, document_key);

CREATE INDEX IF NOT EXISTS idx_analysis_documents_source_published
ON analysis_documents(source_id, published_at, document_order, id);

CREATE INDEX IF NOT EXISTS idx_analysis_documents_kind_source_published
ON analysis_documents(document_kind, source_id, published_at, document_order, id);

CREATE INDEX IF NOT EXISTS idx_analysis_documents_ref
ON analysis_documents(ref);
"#;

pub(crate) async fn create_analysis_documents_schema<'e, E>(executor: E) -> AppResult<()>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::raw_sql(ANALYSIS_DOCUMENTS_SCHEMA_SQL)
        .execute(executor)
        .await
        .map_err(AppError::database)?;
    Ok(())
}

pub(crate) const DOCUMENT_KIND_TELEGRAM_MESSAGE: &str = "telegram_message";
pub(crate) const DOCUMENT_KIND_YOUTUBE_TRANSCRIPT: &str = "youtube_transcript";
pub(crate) const DOCUMENT_KIND_YOUTUBE_COMMENT: &str = "youtube_comment";
pub(crate) const DOCUMENT_KIND_YOUTUBE_DESCRIPTION: &str = "youtube_description";
pub(crate) const YOUTUBE_DESCRIPTION_DOCUMENT_KEY: &str = "youtube:description";
pub(crate) const ANALYSIS_METADATA_VERSION: i64 = 1;

pub(crate) fn live_item_ref(source_id: i64, item_id: i64) -> String {
    format!("s{source_id}-i{item_id}")
}

pub(crate) fn transcript_segment_ref(source_id: i64, item_id: i64, start_ms: i64) -> String {
    format!("s{source_id}-i{item_id}@{start_ms}ms")
}

pub(crate) fn youtube_description_ref(source_id: i64) -> String {
    format!("s{source_id}-i0")
}

#[derive(sqlx::FromRow)]
struct ItemDocumentRow {
    id: i64,
    source_id: i64,
    external_id: String,
    item_kind: String,
    author: Option<String>,
    published_at: i64,
    content_zstd: Vec<u8>,
    source_type: String,
    source_subtype: Option<String>,
}

#[derive(sqlx::FromRow)]
struct YoutubeTranscriptDocumentRow {
    item_id: i64,
    source_id: i64,
    external_id: String,
    author: Option<String>,
    published_at: i64,
    source_external_id: String,
    source_title: Option<String>,
    typed_video_id: Option<String>,
    typed_canonical_url: Option<String>,
    typed_title: Option<String>,
    typed_channel_title: Option<String>,
    typed_channel_handle: Option<String>,
    segment_index: i64,
    start_ms: i64,
    end_ms: Option<i64>,
    text: String,
    caption_language: Option<String>,
    caption_track_kind: Option<String>,
}

#[derive(sqlx::FromRow)]
struct YoutubeDescriptionDocumentRow {
    source_id: i64,
    video_id: String,
    canonical_url: String,
    title: Option<String>,
    channel_title: Option<String>,
    channel_handle: Option<String>,
    published_at: Option<String>,
    description: Option<String>,
}

pub(crate) async fn rebuild_analysis_documents_for_source(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source_id: i64,
) -> AppResult<()> {
    let mut tx = pool.begin().await.map_err(AppError::database)?;
    rebuild_analysis_documents_for_source_on_connection(&mut tx, source_id).await?;
    tx.commit().await.map_err(AppError::database)
}

pub(crate) async fn rebuild_analysis_documents_for_source_on_connection(
    conn: &mut sqlx::SqliteConnection,
    source_id: i64,
) -> AppResult<()> {
    sqlx::query("DELETE FROM analysis_documents WHERE source_id = ?")
        .bind(source_id)
        .execute(&mut *conn)
        .await
        .map_err(AppError::database)?;

    insert_item_backed_documents_for_source(conn, source_id).await?;
    insert_youtube_transcript_documents_for_source(conn, source_id).await?;
    upsert_youtube_description_document_on_connection(conn, source_id).await
}

pub(crate) async fn backfill_all_analysis_documents_on_connection(
    conn: &mut sqlx::SqliteConnection,
) -> AppResult<()> {
    let source_ids: Vec<i64> = sqlx::query_scalar(
        "SELECT id FROM sources WHERE source_type IN ('telegram', 'youtube') ORDER BY id",
    )
    .fetch_all(&mut *conn)
    .await
    .map_err(AppError::database)?;

    for source_id in source_ids {
        rebuild_analysis_documents_for_source_on_connection(conn, source_id).await?;
    }
    Ok(())
}

async fn insert_item_backed_documents_for_source(
    conn: &mut sqlx::SqliteConnection,
    source_id: i64,
) -> AppResult<()> {
    let rows: Vec<ItemDocumentRow> = sqlx::query_as(
        r#"
        SELECT
            items.id,
            items.source_id,
            items.external_id,
            items.item_kind,
            items.author,
            items.published_at,
            items.content_zstd,
            sources.source_type,
            sources.source_subtype
        FROM items
        JOIN sources ON sources.id = items.source_id
        WHERE items.source_id = ?
          AND items.content_zstd IS NOT NULL
          AND items.content_kind IN ('text_only', 'text_with_media')
          AND items.item_kind IN ('telegram_message', 'youtube_comment')
          AND NOT EXISTS (
            SELECT 1 FROM telegram_messages tm
            WHERE tm.item_id = items.id
              AND tm.is_migrated_history = 1
          )
        ORDER BY items.id
        "#,
    )
    .bind(source_id)
    .fetch_all(&mut *conn)
    .await
    .map_err(AppError::database)?;

    for row in rows {
        upsert_item_document_row_on_connection(conn, row).await?;
    }
    Ok(())
}

pub(crate) async fn upsert_item_backed_document_on_connection(
    conn: &mut sqlx::SqliteConnection,
    item_id: i64,
) -> AppResult<()> {
    let row: Option<ItemDocumentRow> = sqlx::query_as(
        r#"
        SELECT
            items.id,
            items.source_id,
            items.external_id,
            items.item_kind,
            items.author,
            items.published_at,
            items.content_zstd,
            sources.source_type,
            sources.source_subtype
        FROM items
        JOIN sources ON sources.id = items.source_id
        WHERE items.id = ?
          AND items.content_zstd IS NOT NULL
          AND items.content_kind IN ('text_only', 'text_with_media')
          AND items.item_kind IN ('telegram_message', 'youtube_comment')
          AND NOT EXISTS (
            SELECT 1 FROM telegram_messages tm
            WHERE tm.item_id = items.id
              AND tm.is_migrated_history = 1
          )
        "#,
    )
    .bind(item_id)
    .fetch_optional(&mut *conn)
    .await
    .map_err(AppError::database)?;

    let Some(row) = row else {
        sqlx::query("DELETE FROM analysis_documents WHERE item_id = ?")
            .bind(item_id)
            .execute(&mut *conn)
            .await
            .map_err(AppError::database)?;
        return Ok(());
    };

    upsert_item_document_row_on_connection(conn, row).await
}

async fn upsert_item_document_row_on_connection(
    conn: &mut sqlx::SqliteConnection,
    row: ItemDocumentRow,
) -> AppResult<()> {
    let document_kind = match row.item_kind.as_str() {
        "telegram_message" => DOCUMENT_KIND_TELEGRAM_MESSAGE,
        "youtube_comment" => DOCUMENT_KIND_YOUTUBE_COMMENT,
        _ => return Ok(()),
    };
    sqlx::query(
        r#"
        INSERT INTO analysis_documents (
            source_id, item_id, document_key, document_kind,
            source_type, source_subtype, external_id, author,
            published_at, document_order, ref, content_zstd,
            metadata_zstd, created_at, updated_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, NULL, strftime('%s','now'), strftime('%s','now'))
        ON CONFLICT(source_id, document_key) DO UPDATE SET
            item_id = excluded.item_id,
            document_kind = excluded.document_kind,
            source_type = excluded.source_type,
            source_subtype = excluded.source_subtype,
            external_id = excluded.external_id,
            author = excluded.author,
            published_at = excluded.published_at,
            document_order = excluded.document_order,
            ref = excluded.ref,
            content_zstd = excluded.content_zstd,
            metadata_zstd = excluded.metadata_zstd,
            updated_at = excluded.updated_at
        "#,
    )
    .bind(row.source_id)
    .bind(row.id)
    .bind(format!("item:{}", row.id))
    .bind(document_kind)
    .bind(&row.source_type)
    .bind(&row.source_subtype)
    .bind(&row.external_id)
    .bind(&row.author)
    .bind(row.published_at)
    .bind(row.id)
    .bind(live_item_ref(row.source_id, row.id))
    .bind(row.content_zstd)
    .execute(&mut *conn)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

async fn insert_youtube_transcript_documents_for_source(
    conn: &mut sqlx::SqliteConnection,
    source_id: i64,
) -> AppResult<()> {
    let rows: Vec<YoutubeTranscriptDocumentRow> = sqlx::query_as(
        r#"
        SELECT
            items.id AS item_id,
            items.source_id,
            items.external_id,
            items.author,
            items.published_at,
            sources.external_id AS source_external_id,
            sources.title AS source_title,
            yvs.video_id AS typed_video_id,
            yvs.canonical_url AS typed_canonical_url,
            yvs.title AS typed_title,
            yvs.channel_title AS typed_channel_title,
            yvs.channel_handle AS typed_channel_handle,
            segments.segment_index,
            segments.start_ms,
            segments.end_ms,
            segments.text,
            segments.caption_language,
            segments.caption_track_kind
        FROM items
        JOIN sources ON sources.id = items.source_id
        JOIN youtube_transcript_segments segments ON segments.item_id = items.id
        LEFT JOIN youtube_video_sources yvs ON yvs.source_id = sources.id
        WHERE items.source_id = ?
          AND items.item_kind = 'youtube_transcript'
          AND segments.text IS NOT NULL
        ORDER BY items.id ASC, segments.segment_index ASC
        "#,
    )
    .bind(source_id)
    .fetch_all(&mut *conn)
    .await
    .map_err(AppError::database)?;

    insert_youtube_transcript_document_rows(conn, rows).await
}

pub(crate) async fn rebuild_youtube_transcript_documents_for_item_on_connection(
    conn: &mut sqlx::SqliteConnection,
    item_id: i64,
) -> AppResult<()> {
    let source_id: Option<i64> = sqlx::query_scalar(
        "SELECT source_id FROM items WHERE id = ? AND item_kind = 'youtube_transcript'",
    )
    .bind(item_id)
    .fetch_optional(&mut *conn)
    .await
    .map_err(AppError::database)?;
    let Some(source_id) = source_id else {
        sqlx::query(
            "DELETE FROM analysis_documents
             WHERE item_id = ? AND document_kind = 'youtube_transcript'",
        )
        .bind(item_id)
        .execute(&mut *conn)
        .await
        .map_err(AppError::database)?;
        return Ok(());
    };

    sqlx::query(
        "DELETE FROM analysis_documents
         WHERE source_id = ? AND item_id = ? AND document_kind = 'youtube_transcript'",
    )
    .bind(source_id)
    .bind(item_id)
    .execute(&mut *conn)
    .await
    .map_err(AppError::database)?;

    insert_youtube_transcript_documents_for_source_and_item(conn, source_id, item_id).await
}

async fn insert_youtube_transcript_documents_for_source_and_item(
    conn: &mut sqlx::SqliteConnection,
    source_id: i64,
    item_id: i64,
) -> AppResult<()> {
    let rows: Vec<YoutubeTranscriptDocumentRow> = sqlx::query_as(
        r#"
        SELECT
            items.id AS item_id,
            items.source_id,
            items.external_id,
            items.author,
            items.published_at,
            sources.external_id AS source_external_id,
            sources.title AS source_title,
            yvs.video_id AS typed_video_id,
            yvs.canonical_url AS typed_canonical_url,
            yvs.title AS typed_title,
            yvs.channel_title AS typed_channel_title,
            yvs.channel_handle AS typed_channel_handle,
            segments.segment_index,
            segments.start_ms,
            segments.end_ms,
            segments.text,
            segments.caption_language,
            segments.caption_track_kind
        FROM items
        JOIN sources ON sources.id = items.source_id
        JOIN youtube_transcript_segments segments ON segments.item_id = items.id
        LEFT JOIN youtube_video_sources yvs ON yvs.source_id = sources.id
        WHERE items.source_id = ?
          AND items.id = ?
          AND items.item_kind = 'youtube_transcript'
          AND segments.text IS NOT NULL
        ORDER BY items.id ASC, segments.segment_index ASC
        "#,
    )
    .bind(source_id)
    .bind(item_id)
    .fetch_all(&mut *conn)
    .await
    .map_err(AppError::database)?;

    insert_youtube_transcript_document_rows(conn, rows).await
}

async fn insert_youtube_transcript_document_rows(
    conn: &mut sqlx::SqliteConnection,
    rows: Vec<YoutubeTranscriptDocumentRow>,
) -> AppResult<()> {
    for row in rows {
        let content_zstd = compress_text(&row.text).map_err(AppError::internal)?;
        let metadata_zstd = youtube_segment_metadata_zstd(&row)?;
        sqlx::query(
            r#"
            INSERT INTO analysis_documents (
                source_id, item_id, document_key, document_kind,
                source_type, source_subtype, external_id, author,
                published_at, document_order, ref, content_zstd,
                metadata_zstd, created_at, updated_at
            )
            VALUES (?, ?, ?, 'youtube_transcript', 'youtube', 'video', ?, ?, ?, ?, ?, ?, ?, strftime('%s','now'), strftime('%s','now'))
            ON CONFLICT(source_id, document_key) DO UPDATE SET
                item_id = excluded.item_id,
                document_kind = excluded.document_kind,
                source_type = excluded.source_type,
                source_subtype = excluded.source_subtype,
                external_id = excluded.external_id,
                author = excluded.author,
                published_at = excluded.published_at,
                document_order = excluded.document_order,
                ref = excluded.ref,
                content_zstd = excluded.content_zstd,
                metadata_zstd = excluded.metadata_zstd,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(row.source_id)
        .bind(row.item_id)
        .bind(format!("item:{}:segment:{}", row.item_id, row.segment_index))
        .bind(&row.external_id)
        .bind(&row.author)
        .bind(row.published_at)
        .bind(row.segment_index)
        .bind(transcript_segment_ref(
            row.source_id,
            row.item_id,
            row.start_ms,
        ))
        .bind(content_zstd)
        .bind(metadata_zstd)
        .execute(&mut *conn)
        .await
        .map_err(AppError::database)?;
    }
    Ok(())
}

pub(crate) async fn upsert_youtube_description_document_on_connection(
    conn: &mut sqlx::SqliteConnection,
    source_id: i64,
) -> AppResult<()> {
    let row: Option<YoutubeDescriptionDocumentRow> = sqlx::query_as(
        r#"
        SELECT source_id, video_id, canonical_url, title, channel_title,
               channel_handle, published_at, description
        FROM youtube_video_sources
        WHERE source_id = ?
        "#,
    )
    .bind(source_id)
    .fetch_optional(&mut *conn)
    .await
    .map_err(AppError::database)?;

    let Some(row) = row else {
        delete_youtube_description_document_on_connection(conn, source_id).await?;
        return Ok(());
    };
    let Some(description) = row
        .description
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
    else {
        delete_youtube_description_document_on_connection(conn, source_id).await?;
        return Ok(());
    };
    let Some(published_at) = row.published_at.as_deref().and_then(ymd_to_unix_midnight) else {
        delete_youtube_description_document_on_connection(conn, source_id).await?;
        return Ok(());
    };

    let mut materialized = row;
    materialized.description = Some(description);
    let content_zstd =
        compress_text(&youtube_description_content(&materialized)).map_err(AppError::internal)?;
    let metadata_zstd = youtube_description_metadata_zstd(&materialized)?;

    sqlx::query(
        r#"
        INSERT INTO analysis_documents (
            source_id, item_id, document_key, document_kind,
            source_type, source_subtype, external_id, author,
            published_at, document_order, ref, content_zstd,
            metadata_zstd, created_at, updated_at
        )
        VALUES (?, NULL, 'youtube:description', 'youtube_description', 'youtube', 'video', ?, ?, ?, -1, ?, ?, ?, strftime('%s','now'), strftime('%s','now'))
        ON CONFLICT(source_id, document_key) DO UPDATE SET
            item_id = excluded.item_id,
            document_kind = excluded.document_kind,
            source_type = excluded.source_type,
            source_subtype = excluded.source_subtype,
            external_id = excluded.external_id,
            author = excluded.author,
            published_at = excluded.published_at,
            document_order = excluded.document_order,
            ref = excluded.ref,
            content_zstd = excluded.content_zstd,
            metadata_zstd = excluded.metadata_zstd,
            updated_at = excluded.updated_at
        "#,
    )
    .bind(materialized.source_id)
    .bind(format!("description:{}", materialized.video_id))
    .bind(&materialized.channel_title)
    .bind(published_at)
    .bind(youtube_description_ref(materialized.source_id))
    .bind(content_zstd)
    .bind(metadata_zstd)
    .execute(&mut *conn)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

pub(crate) async fn delete_youtube_description_document_on_connection(
    conn: &mut sqlx::SqliteConnection,
    source_id: i64,
) -> AppResult<()> {
    sqlx::query(
        "DELETE FROM analysis_documents
         WHERE source_id = ? AND document_key = 'youtube:description'",
    )
    .bind(source_id)
    .execute(&mut *conn)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

fn youtube_segment_metadata_zstd(row: &YoutubeTranscriptDocumentRow) -> AppResult<Vec<u8>> {
    let video_id = row
        .typed_video_id
        .as_deref()
        .unwrap_or(row.source_external_id.as_str());
    let canonical_url = row
        .typed_canonical_url
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .map(ToString::to_string)
        .unwrap_or_else(|| format!("https://www.youtube.com/watch?v={video_id}"));
    let title = row.typed_title.as_deref().or(row.source_title.as_deref());
    let metadata = serde_json::json!({
        "metadata_version": ANALYSIS_METADATA_VERSION,
        "video_id": video_id,
        "canonical_url": canonical_url,
        "title": title,
        "channel_title": &row.typed_channel_title,
        "channel_handle": &row.typed_channel_handle,
        "caption_language": &row.caption_language,
        "caption_track_kind": &row.caption_track_kind,
        "segment_start_ms": row.start_ms,
        "segment_end_ms": row.end_ms,
        "item_kind": DOCUMENT_KIND_YOUTUBE_TRANSCRIPT,
    });
    let json =
        serde_json::to_vec(&metadata).map_err(|error| AppError::internal(error.to_string()))?;
    compress_json_bytes(&json).map_err(AppError::internal)
}

fn youtube_description_content(row: &YoutubeDescriptionDocumentRow) -> String {
    let title = row.title.clone().unwrap_or_else(|| row.video_id.clone());
    let channel = row
        .channel_title
        .clone()
        .unwrap_or_else(|| "unknown".to_string());
    let description = row.description.as_deref().unwrap_or_default().trim();
    format!(
        "YouTube video description\nTitle: {title}\nChannel: {channel}\nURL: {url}\n\n{description}",
        url = row.canonical_url,
    )
}

fn youtube_description_metadata_zstd(row: &YoutubeDescriptionDocumentRow) -> AppResult<Vec<u8>> {
    let metadata = serde_json::json!({
        "metadata_version": ANALYSIS_METADATA_VERSION,
        "video_id": &row.video_id,
        "canonical_url": &row.canonical_url,
        "title": &row.title,
        "channel_title": &row.channel_title,
        "channel_handle": &row.channel_handle,
        "item_kind": DOCUMENT_KIND_YOUTUBE_DESCRIPTION,
    });
    let json =
        serde_json::to_vec(&metadata).map_err(|error| AppError::internal(error.to_string()))?;
    compress_json_bytes(&json).map_err(AppError::internal)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compression::{compress_text, decompress_bytes, decompress_text};
    use crate::sources::test_support::memory_pool_with_source_items_and_topics;
    use serde_json::Value;

    async fn create_youtube_transcript_segments_table(pool: &sqlx::SqlitePool) {
        sqlx::raw_sql(
            r#"
            CREATE TABLE IF NOT EXISTS youtube_transcript_segments (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                item_id INTEGER NOT NULL REFERENCES items(id) ON DELETE CASCADE,
                source_id INTEGER NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
                segment_index INTEGER NOT NULL,
                start_ms INTEGER NOT NULL,
                end_ms INTEGER,
                text TEXT NOT NULL,
                chapter_index INTEGER,
                caption_language TEXT,
                caption_track_kind TEXT,
                is_auto_generated INTEGER NOT NULL DEFAULT 0,
                metadata_zstd BLOB,
                UNIQUE(item_id, segment_index)
            );

            CREATE INDEX IF NOT EXISTS idx_youtube_transcript_segments_item_time
                ON youtube_transcript_segments(item_id, start_ms);

            CREATE INDEX IF NOT EXISTS idx_youtube_transcript_segments_source
                ON youtube_transcript_segments(source_id);
            "#,
        )
        .execute(pool)
        .await
        .expect("create youtube_transcript_segments");
    }

    async fn seed_sources(pool: &sqlx::SqlitePool) {
        crate::sources::test_support::create_youtube_typed_source_tables(pool).await;
        create_youtube_transcript_segments_table(pool).await;
        sqlx::query(
            "INSERT INTO sources (
                id, source_type, source_subtype, external_id, title, is_active, is_member, created_at
             ) VALUES
                (1, 'telegram', 'supergroup', 'tg1', 'Telegram', 1, 1, 1),
                (2, 'youtube', 'video', 'video2', 'Video 2', 1, 1, 1)",
        )
        .execute(pool)
        .await
        .expect("seed sources");
        sqlx::query(
            "INSERT INTO youtube_video_sources (
                source_id, video_id, canonical_url, title, channel_title, channel_handle,
                published_at, description, video_form, availability_status
             ) VALUES (
                2, 'video2', 'https://www.youtube.com/watch?v=video2',
                'Video 2', 'Channel', '@channel', '2023-11-14',
                'Description body', 'regular', 'available'
             )",
        )
        .execute(pool)
        .await
        .expect("seed youtube metadata");
    }

    async fn seed_text_item(
        pool: &sqlx::SqlitePool,
        id: i64,
        source_id: i64,
        external_id: &str,
        item_kind: &str,
        published_at: i64,
        text: &str,
    ) {
        sqlx::query(
            "INSERT INTO items (
                id, source_id, external_id, item_kind, author, published_at,
                ingested_at, content_kind, has_media, content_zstd
             ) VALUES (?, ?, ?, ?, 'Author', ?, ?, 'text_only', 0, ?)",
        )
        .bind(id)
        .bind(source_id)
        .bind(external_id)
        .bind(item_kind)
        .bind(published_at)
        .bind(published_at)
        .bind(compress_text(text).expect("compress text"))
        .execute(pool)
        .await
        .expect("seed item");
    }

    async fn seed_segment(
        pool: &sqlx::SqlitePool,
        item_id: i64,
        source_id: i64,
        segment_index: i64,
        start_ms: i64,
        text: &str,
    ) {
        sqlx::query(
            "INSERT INTO youtube_transcript_segments (
                item_id, source_id, segment_index, start_ms, end_ms, text,
                caption_language, caption_track_kind, is_auto_generated
             ) VALUES (?, ?, ?, ?, ?, ?, 'en', 'manual', 0)",
        )
        .bind(item_id)
        .bind(source_id)
        .bind(segment_index)
        .bind(start_ms)
        .bind(start_ms + 1_000)
        .bind(text)
        .execute(pool)
        .await
        .expect("seed segment");
    }

    #[tokio::test]
    async fn schema_creates_analysis_documents_constraints_and_indexes() {
        let pool = memory_pool_with_source_items_and_topics().await;

        create_analysis_documents_schema(&pool)
            .await
            .expect("create analysis document schema");

        let table_exists: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'analysis_documents'",
        )
        .fetch_one(&pool)
        .await
        .expect("check table");
        assert_eq!(table_exists, 1);

        for index in [
            "idx_analysis_documents_source_key",
            "idx_analysis_documents_source_published",
            "idx_analysis_documents_kind_source_published",
            "idx_analysis_documents_ref",
        ] {
            let exists: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = ?",
            )
            .bind(index)
            .fetch_one(&pool)
            .await
            .expect("check index");
            assert_eq!(exists, 1, "missing index {index}");
        }

        sqlx::query(
            "INSERT INTO sources (
                id, source_type, source_subtype, external_id, title, is_active, is_member, created_at
             ) VALUES (1, 'telegram', 'supergroup', 'tg1', 'Telegram', 1, 1, 1)",
        )
        .execute(&pool)
        .await
        .expect("seed source");

        sqlx::query(
            "INSERT INTO items (
                id, source_id, external_id, item_kind, author, published_at,
                ingested_at, content_kind, has_media, content_zstd
             ) VALUES (10, 1, '10', 'telegram_message', 'alice', 100, 100, 'text_only', 0, x'01')",
        )
        .execute(&pool)
        .await
        .expect("seed item");

        sqlx::query(
            "INSERT INTO analysis_documents (
                source_id, item_id, document_key, document_kind, source_type,
                source_subtype, external_id, author, published_at, document_order,
                ref, content_zstd, created_at, updated_at
             ) VALUES (
                1, 10, 'item:10', 'telegram_message', 'telegram',
                'supergroup', '10', 'alice', 100, 10,
                's1-i10', x'01', 100, 100
             )",
        )
        .execute(&pool)
        .await
        .expect("valid item-backed document");

        let invalid_synthetic = sqlx::query(
            "INSERT INTO analysis_documents (
                source_id, item_id, document_key, document_kind, source_type,
                source_subtype, external_id, published_at, document_order,
                ref, content_zstd, created_at, updated_at
             ) VALUES (
                1, 10, 'youtube:description', 'youtube_description', 'youtube',
                'video', 'description:v1', 100, -1,
                's1-i0', x'01', 100, 100
             )",
        )
        .execute(&pool)
        .await;
        assert!(invalid_synthetic.is_err());
    }

    #[tokio::test]
    async fn rebuild_source_materializes_text_units_with_document_order() {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_analysis_documents_schema(&pool)
            .await
            .expect("schema");
        seed_sources(&pool).await;
        seed_text_item(
            &pool,
            10,
            1,
            "10",
            "telegram_message",
            1_700_000_000,
            "Telegram text",
        )
        .await;
        seed_text_item(
            &pool,
            20,
            2,
            "transcript:video2:en:manual",
            "youtube_transcript",
            1_700_000_000,
            "full transcript",
        )
        .await;
        seed_text_item(
            &pool,
            30,
            2,
            "comment:c1",
            "youtube_comment",
            1_700_000_001,
            "Comment",
        )
        .await;
        seed_segment(&pool, 20, 2, 0, 900, "early").await;
        seed_segment(&pool, 20, 2, 1, 10_000, "late").await;

        rebuild_analysis_documents_for_source(&pool, 1)
            .await
            .expect("rebuild telegram");
        rebuild_analysis_documents_for_source(&pool, 2)
            .await
            .expect("rebuild youtube");

        let rows: Vec<(String, String, i64, String, Option<i64>)> = sqlx::query_as(
            "SELECT document_kind, ref, document_order, external_id, item_id
             FROM analysis_documents
             ORDER BY source_id, published_at, document_order, id",
        )
        .fetch_all(&pool)
        .await
        .expect("load docs");

        assert_eq!(
            rows,
            vec![
                (
                    "telegram_message".to_string(),
                    "s1-i10".to_string(),
                    10,
                    "10".to_string(),
                    Some(10)
                ),
                (
                    "youtube_description".to_string(),
                    "s2-i0".to_string(),
                    -1,
                    "description:video2".to_string(),
                    None
                ),
                (
                    "youtube_transcript".to_string(),
                    "s2-i20@900ms".to_string(),
                    0,
                    "transcript:video2:en:manual".to_string(),
                    Some(20)
                ),
                (
                    "youtube_transcript".to_string(),
                    "s2-i20@10000ms".to_string(),
                    1,
                    "transcript:video2:en:manual".to_string(),
                    Some(20)
                ),
                (
                    "youtube_comment".to_string(),
                    "s2-i30".to_string(),
                    30,
                    "comment:c1".to_string(),
                    Some(30)
                ),
            ]
        );

        let content: Vec<String> = sqlx::query_scalar(
            "SELECT content_zstd FROM analysis_documents
             ORDER BY source_id, published_at, document_order, id",
        )
        .fetch_all(&pool)
        .await
        .expect("load content")
        .into_iter()
        .map(|bytes: Vec<u8>| decompress_text(&bytes).expect("decompress document"))
        .collect();
        assert_eq!(content[0], "Telegram text");
        assert_eq!(content[2], "early");
        assert_eq!(content[3], "late");
    }

    #[tokio::test]
    async fn rebuild_analysis_documents_excludes_migrated_history_rows() {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_analysis_documents_schema(&pool)
            .await
            .expect("create analysis docs");
        seed_sources(&pool).await;
        seed_text_item(&pool, 1, 1, "1", "telegram_message", 100, "Current").await;
        seed_text_item(&pool, 2, 1, "2", "telegram_message", 90, "Migrated").await;
        sqlx::query(
            "INSERT INTO telegram_messages (
                item_id, source_id, history_peer_kind, history_peer_id,
                telegram_message_id, migration_domain, is_migrated_history
             ) VALUES (1, 1, 'channel', 12345, 10, NULL, 0),
                      (2, 1, 'chat', 777, 10, 'migrated_from_chat', 1)",
        )
        .execute(&pool)
        .await
        .expect("seed telegram messages");

        rebuild_analysis_documents_for_source(&pool, 1)
            .await
            .expect("rebuild docs");

        let item_ids: Vec<i64> = sqlx::query_scalar(
            "SELECT item_id FROM analysis_documents WHERE source_id = 1 ORDER BY item_id",
        )
        .fetch_all(&pool)
        .await
        .expect("load docs");

        assert_eq!(item_ids, vec![1]);
    }

    #[tokio::test]
    async fn rebuild_source_removes_stale_documents_and_is_idempotent() {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_analysis_documents_schema(&pool)
            .await
            .expect("schema");
        seed_sources(&pool).await;
        seed_text_item(&pool, 10, 1, "10", "telegram_message", 100, "First").await;

        rebuild_analysis_documents_for_source(&pool, 1)
            .await
            .expect("first rebuild");
        rebuild_analysis_documents_for_source(&pool, 1)
            .await
            .expect("second rebuild");

        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM analysis_documents WHERE source_id = 1")
                .fetch_one(&pool)
                .await
                .expect("count docs");
        assert_eq!(count, 1);

        sqlx::query("UPDATE items SET content_zstd = NULL WHERE id = 10")
            .execute(&pool)
            .await
            .expect("clear content");
        rebuild_analysis_documents_for_source(&pool, 1)
            .await
            .expect("third rebuild");

        let count_after_delete: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM analysis_documents WHERE source_id = 1")
                .fetch_one(&pool)
                .await
                .expect("count docs after delete");
        assert_eq!(count_after_delete, 0);
    }

    #[tokio::test]
    async fn document_metadata_envelopes_match_current_evidence_shape() {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_analysis_documents_schema(&pool)
            .await
            .expect("schema");
        seed_sources(&pool).await;
        seed_text_item(
            &pool,
            20,
            2,
            "transcript:video2:en:manual",
            "youtube_transcript",
            1_700_000_000,
            "full transcript",
        )
        .await;
        seed_segment(&pool, 20, 2, 0, 900, "segment").await;

        rebuild_analysis_documents_for_source(&pool, 2)
            .await
            .expect("rebuild youtube");

        let metadata_rows: Vec<Vec<u8>> = sqlx::query_scalar(
            "SELECT metadata_zstd FROM analysis_documents
             WHERE document_kind IN ('youtube_transcript', 'youtube_description')
             ORDER BY document_kind",
        )
        .fetch_all(&pool)
        .await
        .expect("load metadata");
        assert_eq!(metadata_rows.len(), 2);

        let decoded = metadata_rows
            .iter()
            .map(|bytes| {
                serde_json::from_slice::<Value>(&decompress_bytes(bytes).expect("decompress json"))
                    .expect("json")
            })
            .collect::<Vec<_>>();
        assert!(decoded
            .iter()
            .any(|value| value["item_kind"] == "youtube_description"));
        assert!(decoded.iter().any(|value| {
            value["item_kind"] == "youtube_transcript"
                && value["segment_start_ms"] == 900
                && value["canonical_url"] == "https://www.youtube.com/watch?v=video2"
        }));
    }
}
