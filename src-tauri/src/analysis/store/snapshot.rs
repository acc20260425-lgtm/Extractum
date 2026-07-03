use sqlx::{Pool, Sqlite};

use super::super::models::{CorpusMessage, StoredRunSnapshotRow};
use super::super::ANALYSIS_STATUS_FAILED;
use crate::compression::{compress_text, decompress_text};
use crate::error::{internal_error, AppError, AppResult};

pub(crate) fn sanitize_snapshot_error(category: &str, raw: &str) -> String {
    let mut text = raw
        .chars()
        .map(|ch| if ch.is_control() { ' ' } else { ch })
        .collect::<String>();

    for marker in ["file://", "C:\\", "c:\\", "/home/", "/Users/", "/tmp/"] {
        while let Some(start) = text.find(marker) {
            let end = text[start..]
                .find(char::is_whitespace)
                .map(|offset| start + offset)
                .unwrap_or_else(|| text.len());
            text.replace_range(start..end, "[redacted]");
        }
    }

    for marker in ["http://", "https://"] {
        let mut search_from = 0usize;
        while let Some(relative_start) = text[search_from..].find(marker) {
            let start = search_from + relative_start;
            let end = text[start..]
                .find(char::is_whitespace)
                .map(|offset| start + offset)
                .unwrap_or_else(|| text.len());
            let url = &text[start..end];
            let clean_end = url.find(['?', '#']).unwrap_or(url.len());
            let replacement = format!("{}[redacted]", &url[..clean_end]);
            text.replace_range(start..end, &replacement);
            search_from = start + replacement.len();
        }
    }

    let lower = text.to_lowercase();
    if lower.contains("bearer ")
        || lower.contains("api_key")
        || lower.contains("apikey")
        || lower.contains("sk-")
        || lower.contains("cookie")
    {
        text = category.to_string();
    }

    let compact = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let bounded = compact.chars().take(512).collect::<String>();
    if bounded.trim().is_empty() {
        category.to_string()
    } else {
        bounded
    }
}

pub(crate) fn sanitize_provider_error(category: &str, raw: &str) -> String {
    let sanitized = sanitize_snapshot_error(category, raw);
    let lower = raw.to_lowercase();
    if lower.contains("prompt")
        || lower.contains("payload")
        || lower.contains("raw provider")
        || lower.contains("authorization")
        || lower.contains("bearer")
        || lower.contains("api_key")
        || lower.contains("apikey")
        || lower.contains("sk-")
        || lower.contains("cookie")
    {
        category.to_string()
    } else {
        sanitized
    }
}

fn validate_snapshot_message(message: &CorpusMessage) -> AppResult<()> {
    if message.r#ref.trim().is_empty() {
        return Err(internal_error("Snapshot message ref is required"));
    }
    if message.content.trim().is_empty() {
        return Err(internal_error(format!(
            "Snapshot message {} content is required",
            message.r#ref
        )));
    }
    if message.item_kind.as_deref().unwrap_or("").trim().is_empty() {
        return Err(internal_error(format!(
            "Snapshot message {} item_kind is required",
            message.r#ref
        )));
    }
    let source_type = message.source_type.as_deref().unwrap_or("").trim();
    if source_type.is_empty() {
        return Err(internal_error(format!(
            "Snapshot message {} source_type is required",
            message.r#ref
        )));
    }
    if matches!(source_type, "telegram" | "youtube")
        && message
            .source_subtype
            .as_deref()
            .unwrap_or("")
            .trim()
            .is_empty()
    {
        return Err(internal_error(format!(
            "Snapshot message {} source_subtype is required for {source_type}",
            message.r#ref
        )));
    }
    Ok(())
}

async fn load_run_snapshot_messages_on_transaction(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
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
    .fetch_all(&mut **tx)
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

pub(crate) async fn capture_run_snapshot(
    pool: &Pool<Sqlite>,
    run_id: i64,
    scope_label: &str,
    corpus: &[CorpusMessage],
) -> AppResult<Vec<CorpusMessage>> {
    if corpus.is_empty() {
        return Err(internal_error("Snapshot capture failed: empty corpus"));
    }

    for message in corpus {
        validate_snapshot_message(message)?;
    }

    let mut tx = pool.begin().await.map_err(AppError::database)?;

    sqlx::query(
        r#"
        UPDATE analysis_runs
        SET scope_label_snapshot = ?,
            snapshot_captured_at = NULL,
            snapshot_error = NULL
        WHERE id = ?
        "#,
    )
    .bind(scope_label)
    .bind(run_id)
    .execute(&mut *tx)
    .await
    .map_err(AppError::database)?;

    sqlx::query("DELETE FROM analysis_run_messages WHERE run_id = ?")
        .bind(run_id)
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?;

    for message in corpus {
        let content_zstd = compress_text(&message.content).map_err(internal_error)?;
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
                content_zstd,
                item_kind,
                source_type,
                source_subtype,
                metadata_zstd
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
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
        .bind(message.item_kind.as_deref())
        .bind(message.source_type.as_deref())
        .bind(message.source_subtype.as_deref())
        .bind(message.metadata_zstd.as_deref())
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?;
    }

    let captured = load_run_snapshot_messages_on_transaction(&mut tx, run_id).await?;
    if captured.is_empty() {
        return Err(internal_error(
            "Snapshot capture failed: reloaded snapshot is empty",
        ));
    }

    sqlx::query(
        "UPDATE analysis_runs SET snapshot_captured_at = datetime('now'), snapshot_error = NULL WHERE id = ?",
    )
    .bind(run_id)
    .execute(&mut *tx)
    .await
    .map_err(AppError::database)?;

    tx.commit().await.map_err(AppError::database)?;
    Ok(captured)
}

#[allow(dead_code)]
pub(crate) async fn persist_run_snapshot(
    pool: &Pool<Sqlite>,
    run_id: i64,
    scope_label: &str,
    corpus: &[CorpusMessage],
) -> AppResult<()> {
    capture_run_snapshot(pool, run_id, scope_label, corpus)
        .await
        .map(|_| ())
}

pub(crate) async fn mark_run_capture_failed(
    pool: &Pool<Sqlite>,
    run_id: i64,
    snapshot_error: &str,
    completed_at: i64,
) -> AppResult<()> {
    let sanitized = sanitize_snapshot_error("Snapshot capture failed", snapshot_error);
    sqlx::query(
        r#"
        UPDATE analysis_runs
        SET
            status = ?,
            error = ?,
            snapshot_error = ?,
            completed_at = ?
        WHERE id = ?
        "#,
    )
    .bind(ANALYSIS_STATUS_FAILED)
    .bind(&sanitized)
    .bind(&sanitized)
    .bind(completed_at)
    .bind(run_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}
