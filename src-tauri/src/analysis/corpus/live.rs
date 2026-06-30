use sqlx::{Pool, QueryBuilder, Sqlite};

use super::{CorpusLoadRequest, YoutubeCorpusMode};
use crate::analysis::models::CorpusMessage;
use crate::compression::{compress_json_bytes, decompress_text};
use crate::error::{internal_error, AppError, AppResult};

#[allow(dead_code)]
pub(crate) fn live_corpus_ref(source_id: i64, item_id: i64) -> String {
    crate::analysis_documents::live_item_ref(source_id, item_id)
}

pub(crate) async fn load_corpus_messages(
    pool: &Pool<Sqlite>,
    request: &CorpusLoadRequest,
) -> AppResult<Vec<CorpusMessage>> {
    if request.source_ids.is_empty() {
        return Ok(Vec::new());
    }

    if request.source_type == "telegram" {
        return load_telegram_corpus_messages(pool, request).await;
    }

    load_analysis_document_messages(pool, request).await
}

fn telegram_history_metadata_zstd(
    history_scope: &str,
    migration_domain: Option<&str>,
    history_peer_kind: &str,
    history_peer_id: i64,
) -> AppResult<Vec<u8>> {
    compress_json_bytes(
        &serde_json::to_vec(&serde_json::json!({
            "history_scope": history_scope,
            "migration_domain": migration_domain,
            "history_peer_kind": history_peer_kind,
            "history_peer_id": history_peer_id
        }))
        .map_err(internal_error)?,
    )
    .map_err(internal_error)
}

#[derive(sqlx::FromRow)]
struct TelegramCorpusRow {
    item_id: i64,
    source_id: i64,
    external_id: String,
    author: Option<String>,
    published_at: i64,
    ref_: Option<String>,
    content_zstd: Vec<u8>,
    source_type: String,
    source_subtype: Option<String>,
    history_scope: String,
    migration_domain: Option<String>,
    history_peer_kind: String,
    history_peer_id: i64,
}

async fn fetch_telegram_corpus_rows(
    pool: &Pool<Sqlite>,
    request: &CorpusLoadRequest,
    include_migrated_rows: bool,
) -> AppResult<Vec<TelegramCorpusRow>> {
    let mut query = if include_migrated_rows {
        QueryBuilder::<Sqlite>::new(
            r#"
            SELECT
                items.id AS item_id,
                items.source_id,
                items.external_id,
                items.author,
                items.published_at,
                NULL AS ref_,
                items.content_zstd AS content_zstd,
                sources.source_type,
                sources.source_subtype,
                'migrated' AS history_scope,
                tm.migration_domain AS migration_domain,
                tm.history_peer_kind AS history_peer_kind,
                tm.history_peer_id AS history_peer_id
            FROM items
            JOIN sources ON sources.id = items.source_id
            JOIN telegram_messages tm ON tm.item_id = items.id
            WHERE items.published_at >=
            "#,
        )
    } else {
        QueryBuilder::<Sqlite>::new(
            r#"
            SELECT
                COALESCE(d.item_id, 0) AS item_id,
                d.source_id,
                d.external_id,
                d.author,
                d.published_at,
                d.ref AS ref_,
                d.content_zstd,
                d.source_type,
                d.source_subtype,
                'current' AS history_scope,
                NULL AS migration_domain,
                COALESCE(tm.history_peer_kind, 'channel') AS history_peer_kind,
                COALESCE(tm.history_peer_id, 0) AS history_peer_id
            FROM analysis_documents d
            LEFT JOIN telegram_messages tm ON tm.item_id = d.item_id
            WHERE d.published_at >=
            "#,
        )
    };

    query.push_bind(request.period_from);
    if include_migrated_rows {
        query.push(" AND items.published_at <= ");
    } else {
        query.push(" AND d.published_at <= ");
    }
    query.push_bind(request.period_to);
    if include_migrated_rows {
        query.push(" AND items.source_id IN (");
    } else {
        query.push(" AND d.source_id IN (");
    }
    {
        let mut separated = query.separated(", ");
        for source_id in &request.source_ids {
            separated.push_bind(source_id);
        }
    }
    query.push(")");
    if include_migrated_rows {
        query.push(
            r#"
              AND sources.source_type = 'telegram'
              AND items.item_kind = 'telegram_message'
              AND tm.is_migrated_history = 1
              AND tm.migration_domain = 'migrated_from_chat'
              AND items.content_zstd IS NOT NULL
              AND items.content_kind IN ('text_only', 'text_with_media')
            "#,
        );
    } else {
        query.push(" AND d.source_type = 'telegram' AND d.document_kind = 'telegram_message'");
    }

    query
        .build_query_as()
        .fetch_all(pool)
        .await
        .map_err(AppError::database)
}

async fn load_telegram_corpus_messages(
    pool: &Pool<Sqlite>,
    request: &CorpusLoadRequest,
) -> AppResult<Vec<CorpusMessage>> {
    let mut rows = fetch_telegram_corpus_rows(pool, request, false).await?;
    if request.include_migrated_history {
        rows.extend(fetch_telegram_corpus_rows(pool, request, true).await?);
    }

    let mut messages = rows
        .into_iter()
        .map(|row| {
            let metadata_zstd = telegram_history_metadata_zstd(
                &row.history_scope,
                row.migration_domain.as_deref(),
                &row.history_peer_kind,
                row.history_peer_id,
            )?;
            Ok(CorpusMessage {
                item_id: row.item_id,
                source_id: row.source_id,
                external_id: row.external_id,
                published_at: row.published_at,
                author: row.author,
                content: decompress_text(&row.content_zstd).map_err(internal_error)?,
                r#ref: row
                    .ref_
                    .unwrap_or_else(|| live_corpus_ref(row.source_id, row.item_id)),
                item_kind: Some("telegram_message".to_string()),
                source_type: Some(row.source_type),
                source_subtype: row.source_subtype,
                metadata_zstd: Some(metadata_zstd),
            })
        })
        .collect::<AppResult<Vec<_>>>()?;

    messages.sort_by(|left, right| {
        left.published_at
            .cmp(&right.published_at)
            .then_with(|| left.source_id.cmp(&right.source_id))
            .then_with(|| left.r#ref.cmp(&right.r#ref))
    });

    Ok(messages)
}

#[derive(sqlx::FromRow)]
struct AnalysisDocumentRow {
    item_id: Option<i64>,
    source_id: i64,
    external_id: String,
    author: Option<String>,
    published_at: i64,
    ref_: String,
    content_zstd: Vec<u8>,
    document_kind: String,
    source_type: String,
    source_subtype: Option<String>,
    metadata_zstd: Option<Vec<u8>>,
}

pub(crate) fn push_analysis_document_kind_filter(
    query: &mut QueryBuilder<'_, Sqlite>,
    source_type: &str,
    youtube_corpus_mode: YoutubeCorpusMode,
    table_alias: &str,
) -> AppResult<()> {
    match source_type {
        "telegram" => {
            query.push(" AND ");
            query.push(table_alias);
            query.push(".source_type = 'telegram' AND ");
            query.push(table_alias);
            query.push(".document_kind = 'telegram_message'");
            Ok(())
        }
        "youtube" => {
            query.push(" AND ");
            query.push(table_alias);
            query.push(".source_type = 'youtube' AND ");
            query.push(table_alias);
            query.push(".document_kind IN (");
            query.push("'youtube_transcript'");
            if youtube_corpus_mode.includes_description() {
                query.push(", 'youtube_description'");
            }
            if youtube_corpus_mode.includes_comments() {
                query.push(", 'youtube_comment'");
            }
            query.push(")");
            Ok(())
        }
        other => Err(AppError::validation(format!(
            "Unsupported analysis corpus source_type '{other}'"
        ))),
    }
}

async fn load_analysis_document_messages(
    pool: &Pool<Sqlite>,
    request: &CorpusLoadRequest,
) -> AppResult<Vec<CorpusMessage>> {
    let mut query = QueryBuilder::<Sqlite>::new(
        r#"
        SELECT
            d.item_id,
            d.source_id,
            d.external_id,
            d.author,
            d.published_at,
            d.ref AS ref_,
            d.content_zstd,
            d.document_kind,
            d.source_type,
            d.source_subtype,
            d.metadata_zstd
        FROM analysis_documents d
        WHERE d.published_at >=
        "#,
    );
    query.push_bind(request.period_from);
    query.push(" AND d.published_at <= ");
    query.push_bind(request.period_to);
    query.push(" AND d.source_id IN (");
    {
        let mut separated = query.separated(", ");
        for source_id in &request.source_ids {
            separated.push_bind(source_id);
        }
    }
    query.push(")");
    push_analysis_document_kind_filter(
        &mut query,
        request.source_type.as_str(),
        request.youtube_corpus_mode,
        "d",
    )?;
    query.push(" ORDER BY d.published_at ASC, d.source_id ASC, d.document_order ASC, d.id ASC");

    let rows: Vec<AnalysisDocumentRow> = query
        .build_query_as()
        .fetch_all(pool)
        .await
        .map_err(AppError::database)?;

    rows.into_iter()
        .map(|row| {
            Ok(CorpusMessage {
                item_id: row.item_id.unwrap_or(0),
                source_id: row.source_id,
                external_id: row.external_id,
                published_at: row.published_at,
                author: row.author,
                content: decompress_text(&row.content_zstd).map_err(internal_error)?,
                r#ref: row.ref_,
                item_kind: Some(row.document_kind),
                source_type: Some(row.source_type),
                source_subtype: row.source_subtype,
                metadata_zstd: row.metadata_zstd,
            })
        })
        .collect()
}
