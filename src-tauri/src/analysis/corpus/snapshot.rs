use sqlx::{Pool, Sqlite};

use crate::analysis::models::{
    AnalysisRunDetail, AnalysisRunMessage, AnalysisRunMessageCursor, AnalysisRunMessagesPage,
    AnalysisSnapshotState, CorpusMessage, StoredRunSnapshotRow,
};
use crate::compression::{decompress_bytes, decompress_text};
use crate::error::{internal_error, AppError, AppResult};

pub(crate) async fn load_run_snapshot_messages(
    pool: &Pool<Sqlite>,
    run_id: i64,
) -> AppResult<Vec<CorpusMessage>> {
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
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    rows.into_iter()
        .map(|row| {
            Ok(CorpusMessage {
                item_id: row.item_id,
                source_id: row.source_id,
                external_id: row.external_id,
                published_at: row.published_at,
                author: row.author,
                content: decompress_text(&row.content_zstd).map_err(internal_error)?,
                r#ref: row.r#ref,
                item_kind: row.item_kind,
                source_type: row.source_type,
                source_subtype: row.source_subtype,
                metadata_zstd: row.metadata_zstd,
            })
        })
        .collect()
}

pub(crate) struct ListRunSnapshotMessagesRequest {
    pub(crate) run_id: i64,
    pub(crate) after: Option<AnalysisRunMessageCursor>,
    pub(crate) limit: usize,
    pub(crate) source_id: Option<i64>,
    pub(crate) around_ref: Option<String>,
}

fn decode_optional_metadata_json(
    metadata_zstd: Option<&[u8]>,
) -> AppResult<Option<serde_json::Value>> {
    let Some(bytes) = metadata_zstd else {
        return Ok(None);
    };

    let decompressed = decompress_bytes(bytes).map_err(internal_error)?;
    serde_json::from_slice(&decompressed)
        .map(Some)
        .map_err(|e| internal_error(format!("Failed to decode run message metadata JSON: {e}")))
}

fn run_message_from_snapshot_row(row: StoredRunSnapshotRow) -> AppResult<AnalysisRunMessage> {
    Ok(AnalysisRunMessage {
        item_id: row.item_id,
        source_id: row.source_id,
        external_id: row.external_id,
        author: row.author,
        published_at: row.published_at,
        r#ref: row.r#ref,
        content: decompress_text(&row.content_zstd).map_err(internal_error)?,
        item_kind: row.item_kind,
        source_type: row.source_type,
        source_subtype: row.source_subtype,
        metadata_json: decode_optional_metadata_json(row.metadata_zstd.as_deref())?,
    })
}

pub(crate) async fn list_run_snapshot_messages_page(
    pool: &Pool<Sqlite>,
    request: ListRunSnapshotMessagesRequest,
) -> AppResult<AnalysisRunMessagesPage> {
    let limit = request.limit.clamp(1, 500);
    let fetch_limit = (limit + 1) as i64;

    let rows: Vec<StoredRunSnapshotRow> = if let Some(after) = request.after {
        sqlx::query_as(
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
              AND (? IS NULL OR source_id = ?)
              AND (
                published_at > ?
                OR (published_at = ? AND ref > ?)
              )
            ORDER BY published_at ASC, ref ASC
            LIMIT ?
            "#,
        )
        .bind(request.run_id)
        .bind(request.source_id)
        .bind(request.source_id)
        .bind(after.published_at)
        .bind(after.published_at)
        .bind(after.r#ref)
        .bind(fetch_limit)
        .fetch_all(pool)
        .await
        .map_err(AppError::database)?
    } else if let Some(around_ref) = request.around_ref.as_deref() {
        let around_cursor = sqlx::query_as::<_, (i64, String)>(
            r#"
            SELECT published_at, ref
            FROM analysis_run_messages
            WHERE run_id = ?
              AND (? IS NULL OR source_id = ?)
              AND ref = ?
            LIMIT 1
            "#,
        )
        .bind(request.run_id)
        .bind(request.source_id)
        .bind(request.source_id)
        .bind(around_ref)
        .fetch_optional(pool)
        .await
        .map_err(AppError::database)?
        .map(|(published_at, r#ref)| AnalysisRunMessageCursor {
            published_at,
            r#ref,
        });

        if let Some(around) = around_cursor {
            sqlx::query_as(
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
                  AND (? IS NULL OR source_id = ?)
                  AND (
                    published_at > ?
                    OR (published_at = ? AND ref >= ?)
                  )
                ORDER BY published_at ASC, ref ASC
                LIMIT ?
                "#,
            )
            .bind(request.run_id)
            .bind(request.source_id)
            .bind(request.source_id)
            .bind(around.published_at)
            .bind(around.published_at)
            .bind(around.r#ref)
            .bind(fetch_limit)
            .fetch_all(pool)
            .await
            .map_err(AppError::database)?
        } else {
            sqlx::query_as(
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
                  AND (? IS NULL OR source_id = ?)
                ORDER BY published_at ASC, ref ASC
                LIMIT ?
                "#,
            )
            .bind(request.run_id)
            .bind(request.source_id)
            .bind(request.source_id)
            .bind(fetch_limit)
            .fetch_all(pool)
            .await
            .map_err(AppError::database)?
        }
    } else {
        sqlx::query_as(
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
              AND (? IS NULL OR source_id = ?)
            ORDER BY published_at ASC, ref ASC
            LIMIT ?
            "#,
        )
        .bind(request.run_id)
        .bind(request.source_id)
        .bind(request.source_id)
        .bind(fetch_limit)
        .fetch_all(pool)
        .await
        .map_err(AppError::database)?
    };

    let has_more = rows.len() > limit;
    let page_rows = rows.into_iter().take(limit).collect::<Vec<_>>();
    let mut messages = Vec::with_capacity(page_rows.len());
    for row in page_rows {
        messages.push(run_message_from_snapshot_row(row)?);
    }

    let next_cursor = if has_more {
        messages.last().map(|message| AnalysisRunMessageCursor {
            published_at: message.published_at,
            r#ref: message.r#ref.clone(),
        })
    } else {
        None
    };

    Ok(AnalysisRunMessagesPage {
        messages,
        next_cursor,
        has_more,
    })
}

#[allow(dead_code)]
pub(crate) async fn load_run_corpus_messages(
    pool: &Pool<Sqlite>,
    run: &AnalysisRunDetail,
) -> AppResult<Vec<CorpusMessage>> {
    let snapshot = load_run_snapshot_messages(pool, run.id).await?;
    ensure_captured_snapshot_rows(run, &snapshot)?;
    Ok(snapshot)
}

pub(crate) async fn load_trace_resolution_messages(
    pool: &Pool<Sqlite>,
    run: &AnalysisRunDetail,
) -> AppResult<Vec<CorpusMessage>> {
    let snapshot = load_run_snapshot_messages(pool, run.id).await?;
    ensure_captured_snapshot_rows(run, &snapshot)?;
    Ok(snapshot)
}

fn captured_snapshot_missing_error(run_id: i64) -> AppError {
    internal_error(format!(
        "Analysis run {run_id} captured snapshot is unavailable"
    ))
}

fn ensure_captured_snapshot_rows(
    run: &AnalysisRunDetail,
    snapshot: &[CorpusMessage],
) -> AppResult<()> {
    if run.snapshot_state == Some(AnalysisSnapshotState::Captured)
        && run.snapshot_message_count == 0
        && snapshot.is_empty()
    {
        return Err(captured_snapshot_missing_error(run.id));
    }
    Ok(())
}
