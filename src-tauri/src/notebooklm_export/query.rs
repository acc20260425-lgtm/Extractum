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
            })
        })
        .collect::<Result<Vec<_>, String>>()
        .map_err(AppError::from)
}

const BASE_QUERY: &str = r#"
    SELECT
        id,
        source_id,
        external_id,
        author,
        published_at,
        content_zstd,
        content_kind,
        has_media,
        media_kind,
        media_metadata_zstd
    FROM items
    WHERE source_id = ?
    ORDER BY published_at ASC, id ASC
"#;
const BASE_QUERY_WITH_FROM: &str = r#"
    SELECT
        id,
        source_id,
        external_id,
        author,
        published_at,
        content_zstd,
        content_kind,
        has_media,
        media_kind,
        media_metadata_zstd
    FROM items
    WHERE source_id = ? AND published_at >= ?
    ORDER BY published_at ASC, id ASC
"#;
const BASE_QUERY_WITH_TO: &str = r#"
    SELECT
        id,
        source_id,
        external_id,
        author,
        published_at,
        content_zstd,
        content_kind,
        has_media,
        media_kind,
        media_metadata_zstd
    FROM items
    WHERE source_id = ? AND published_at <= ?
    ORDER BY published_at ASC, id ASC
"#;
const BASE_QUERY_WITH_FROM_TO: &str = r#"
    SELECT
        id,
        source_id,
        external_id,
        author,
        published_at,
        content_zstd,
        content_kind,
        has_media,
        media_kind,
        media_metadata_zstd
    FROM items
    WHERE source_id = ? AND published_at >= ? AND published_at <= ?
    ORDER BY published_at ASC, id ASC
"#;
