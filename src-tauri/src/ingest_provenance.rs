#![allow(dead_code)]

use sqlx::{Sqlite, SqliteConnection};

use crate::error::{AppError, AppResult};
use crate::sources::TelegramMessageIdentity;
use crate::tx::{begin_immediate, finish_manual_transaction};

pub(crate) const PROVENANCE_TEXT_MAX_LEN: usize = 512;

pub(crate) struct CreateTelegramTakeoutBatch {
    pub(crate) source_id: i64,
    pub(crate) account_id: i64,
    pub(crate) source_subtype: String,
}

pub(crate) struct IngestObservation {
    pub(crate) batch_id: i64,
    pub(crate) source_id: i64,
    pub(crate) item_id: Option<i64>,
    pub(crate) provider_item_kind: &'static str,
    pub(crate) provider_identity_kind: &'static str,
    pub(crate) provider_identity: String,
    pub(crate) outcome: &'static str,
    pub(crate) reason_code: Option<&'static str>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TerminalBatchStatus {
    Completed,
    Failed,
    Cancelled,
}

impl TerminalBatchStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
}

pub(crate) fn telegram_provider_identity(identity: &TelegramMessageIdentity) -> String {
    format!(
        "telegram:history_peer:{}:{}:message:{}",
        identity.history_peer_kind, identity.history_peer_id, identity.telegram_message_id
    )
}

pub(crate) fn sanitize_provenance_text(value: &str) -> String {
    let mut sanitized = value.replace('\0', " ");
    for marker in [
        "api_hash",
        "auth_key",
        "authorization",
        "cookie",
        "session",
        "secret",
    ] {
        sanitized = sanitized.replace(marker, "[redacted]");
        sanitized = sanitized.replace(&marker.to_ascii_uppercase(), "[redacted]");
    }
    let trimmed = sanitized.trim();
    let without_raw_shape = if trimmed.starts_with('{') || trimmed.starts_with('[') {
        "sanitized structured Telegram error"
    } else {
        trimmed
    };
    without_raw_shape
        .chars()
        .take(PROVENANCE_TEXT_MAX_LEN)
        .collect()
}

pub(crate) async fn create_telegram_takeout_batch(
    pool: &sqlx::Pool<Sqlite>,
    input: CreateTelegramTakeoutBatch,
) -> AppResult<i64> {
    let mut conn = begin_immediate(pool).await?;

    let result: AppResult<i64> = async {
        let batch_id: i64 = sqlx::query_scalar(
            "INSERT INTO ingest_batches (source_id, provider, ingest_kind, status, completeness)
             VALUES (?, 'telegram', 'takeout', 'running', 'unknown')
             RETURNING id",
        )
        .bind(input.source_id)
        .fetch_one(&mut *conn)
        .await
        .map_err(AppError::database)?;

        sqlx::query(
            "INSERT INTO telegram_takeout_batches (batch_id, account_id, source_subtype)
             VALUES (?, ?, ?)",
        )
        .bind(batch_id)
        .bind(input.account_id)
        .bind(&input.source_subtype)
        .execute(&mut *conn)
        .await
        .map_err(AppError::database)?;

        Ok(batch_id)
    }
    .await;

    finish_manual_transaction(&mut conn, result).await
}

pub(crate) async fn update_takeout_resolved_peer(
    pool: &sqlx::Pool<Sqlite>,
    batch_id: i64,
    resolved_peer_kind: &str,
    resolved_peer_id: i64,
    history_peer_kind: &str,
    history_peer_id: i64,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE telegram_takeout_batches
         SET resolved_peer_kind = ?, resolved_peer_id = ?,
             history_peer_kind = ?, history_peer_id = ?, updated_at = CURRENT_TIMESTAMP
         WHERE batch_id = ?",
    )
    .bind(resolved_peer_kind)
    .bind(resolved_peer_id)
    .bind(history_peer_kind)
    .bind(history_peer_id)
    .bind(batch_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

pub(crate) async fn update_takeout_session_started(
    pool: &sqlx::Pool<Sqlite>,
    batch_id: i64,
    takeout_id: i64,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE telegram_takeout_batches
         SET takeout_id = ?, updated_at = CURRENT_TIMESTAMP
         WHERE batch_id = ?",
    )
    .bind(takeout_id)
    .bind(batch_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

pub(crate) async fn mark_takeout_export_dc_attempted(
    pool: &sqlx::Pool<Sqlite>,
    batch_id: i64,
    export_dc_id: i32,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE telegram_takeout_batches
         SET export_dc_id = ?, used_export_dc = 1, updated_at = CURRENT_TIMESTAMP
         WHERE batch_id = ?",
    )
    .bind(i64::from(export_dc_id))
    .bind(batch_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

pub(crate) async fn mark_takeout_export_dc_fallback(
    pool: &sqlx::Pool<Sqlite>,
    batch_id: i64,
    message: &str,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE telegram_takeout_batches
         SET fallback_used = 1, updated_at = CURRENT_TIMESTAMP
         WHERE batch_id = ?",
    )
    .bind(batch_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    record_ingest_batch_warning(pool, batch_id, "export_dc_fallback", message).await
}

pub(crate) async fn update_takeout_split_metadata(
    pool: &sqlx::Pool<Sqlite>,
    batch_id: i64,
    split_count: i64,
    selected_split_count: i64,
    message_count_estimate: Option<i64>,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE telegram_takeout_batches
         SET split_count = ?, selected_split_count = ?, message_count_estimate = ?,
             updated_at = CURRENT_TIMESTAMP
         WHERE batch_id = ?",
    )
    .bind(split_count)
    .bind(selected_split_count)
    .bind(message_count_estimate)
    .bind(batch_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

pub(crate) async fn mark_takeout_migrated_history_deferred(
    pool: &sqlx::Pool<Sqlite>,
    batch_id: i64,
    message: &str,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE telegram_takeout_batches
         SET migrated_history_detected = 1,
             migrated_history_imported = 0,
             history_scope = CASE
               WHEN only_my_messages = 1 THEN 'mixed_partial'
               ELSE 'current_history_with_migrated_deferred'
             END,
             updated_at = CURRENT_TIMESTAMP
         WHERE batch_id = ?",
    )
    .bind(batch_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    record_ingest_batch_warning(pool, batch_id, "migrated_history_deferred", message).await
}

pub(crate) async fn mark_takeout_only_my_messages_fallback(
    pool: &sqlx::Pool<Sqlite>,
    batch_id: i64,
    message: &str,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE telegram_takeout_batches
         SET only_my_messages = 1,
             history_scope = CASE
               WHEN migrated_history_detected = 1 THEN 'mixed_partial'
               ELSE 'partial_private_history'
             END,
             updated_at = CURRENT_TIMESTAMP
         WHERE batch_id = ?",
    )
    .bind(batch_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    record_ingest_batch_warning(pool, batch_id, "only_my_messages_fallback", message).await
}

pub(crate) async fn update_takeout_max_message_id(
    pool: &sqlx::Pool<Sqlite>,
    batch_id: i64,
    max_message_id: i64,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE telegram_takeout_batches
         SET max_message_id = MAX(COALESCE(max_message_id, 0), ?),
             updated_at = CURRENT_TIMESTAMP
         WHERE batch_id = ?",
    )
    .bind(max_message_id)
    .bind(batch_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

pub(crate) async fn record_ingest_batch_warning(
    pool: &sqlx::Pool<Sqlite>,
    batch_id: i64,
    code: &str,
    message: &str,
) -> AppResult<()> {
    let message = sanitize_provenance_text(message);
    sqlx::query(
        "INSERT INTO ingest_batch_warnings (batch_id, code, message)
         VALUES (?, ?, ?)",
    )
    .bind(batch_id)
    .bind(code)
    .bind(message)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

pub(crate) async fn record_ingest_observation(
    pool: &sqlx::Pool<Sqlite>,
    observation: IngestObservation,
) -> AppResult<()> {
    let mut conn = pool.acquire().await.map_err(AppError::database)?;
    record_ingest_observation_on_connection(&mut *conn, observation).await
}

pub(crate) async fn record_ingest_observation_on_connection(
    conn: &mut SqliteConnection,
    observation: IngestObservation,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO ingest_item_observations (
            batch_id, source_id, item_id, provider_item_kind, provider_identity_kind,
            provider_identity, provider_identity_version, outcome, reason_code
         ) VALUES (?, ?, ?, ?, ?, ?, 1, ?, ?)",
    )
    .bind(observation.batch_id)
    .bind(observation.source_id)
    .bind(observation.item_id)
    .bind(observation.provider_item_kind)
    .bind(observation.provider_identity_kind)
    .bind(observation.provider_identity)
    .bind(observation.outcome)
    .bind(observation.reason_code)
    .execute(conn)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

pub(crate) async fn finalize_ingest_batch(
    pool: &sqlx::Pool<Sqlite>,
    batch_id: i64,
    status: TerminalBatchStatus,
    terminal_error: Option<&str>,
) -> AppResult<()> {
    let mut conn = begin_immediate(pool).await?;

    let result: AppResult<()> = async {
        let counts: (i64, i64, i64, i64) = sqlx::query_as(
            "SELECT
                COUNT(*),
                COALESCE(SUM(CASE WHEN outcome = 'inserted' THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN outcome = 'duplicate_observed' THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN outcome = 'skipped' THEN 1 ELSE 0 END), 0)
             FROM ingest_item_observations
             WHERE batch_id = ?",
        )
        .bind(batch_id)
        .fetch_one(&mut *conn)
        .await
        .map_err(AppError::database)?;

        let warning_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM ingest_batch_warnings WHERE batch_id = ?")
                .bind(batch_id)
                .fetch_one(&mut *conn)
                .await
                .map_err(AppError::database)?;

        let detail: (i64, i64, String) = sqlx::query_as(
            "SELECT only_my_messages, migrated_history_detected, history_scope
             FROM telegram_takeout_batches WHERE batch_id = ?",
        )
        .bind(batch_id)
        .fetch_one(&mut *conn)
        .await
        .map_err(AppError::database)?;

        let completeness = classify_completeness(status, counts.0, detail);
        let terminal_error = terminal_error.map(sanitize_provenance_text);
        sqlx::query(
            "UPDATE ingest_batches
             SET status = ?, completeness = ?, finished_at = CURRENT_TIMESTAMP,
                 item_observed_count = ?, item_inserted_count = ?,
                 item_duplicate_count = ?, item_skipped_count = ?,
                 warning_count = ?, terminal_error = ?, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?",
        )
        .bind(status.as_str())
        .bind(completeness)
        .bind(counts.0)
        .bind(counts.1)
        .bind(counts.2)
        .bind(counts.3)
        .bind(warning_count)
        .bind(terminal_error)
        .bind(batch_id)
        .execute(&mut *conn)
        .await
        .map_err(AppError::database)?;

        Ok(())
    }
    .await;

    finish_manual_transaction(&mut conn, result).await
}

fn classify_completeness(
    status: TerminalBatchStatus,
    observation_count: i64,
    detail: (i64, i64, String),
) -> &'static str {
    let (only_my_messages, migrated_history_detected, history_scope) = detail;
    match status {
        TerminalBatchStatus::Completed
            if only_my_messages == 0
                && migrated_history_detected == 0
                && history_scope != "mixed_partial" =>
        {
            "complete"
        }
        TerminalBatchStatus::Completed => "partial",
        TerminalBatchStatus::Failed | TerminalBatchStatus::Cancelled if observation_count > 0 => {
            "partial"
        }
        TerminalBatchStatus::Failed | TerminalBatchStatus::Cancelled => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sources::test_support::{
        create_ingest_provenance_tables, memory_pool_with_source_items_and_topics,
    };

    async fn seed_source(pool: &sqlx::SqlitePool) {
        sqlx::query(
            "INSERT INTO sources (
                id, source_type, source_subtype, account_id, external_id, title,
                is_active, is_member, created_at
             ) VALUES (1, 'telegram', 'supergroup', 10, '12345', 'Forum', 1, 1, 1)",
        )
        .execute(pool)
        .await
        .expect("seed source");
    }

    async fn seed_item(pool: &sqlx::SqlitePool, item_id: i64) {
        sqlx::query(
            "INSERT INTO items (
                id, source_id, external_id, item_kind, published_at, ingested_at,
                content_kind, has_media
             ) VALUES (?, 1, '1', 'telegram_message', 1, 1, 'text', 0)",
        )
        .bind(item_id)
        .execute(pool)
        .await
        .expect("seed item");
    }

    #[tokio::test]
    async fn create_takeout_batch_inserts_generic_and_detail_rows_atomically() {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_ingest_provenance_tables(&pool).await;
        seed_source(&pool).await;

        let batch_id = create_telegram_takeout_batch(
            &pool,
            CreateTelegramTakeoutBatch {
                source_id: 1,
                account_id: 10,
                source_subtype: "supergroup".to_string(),
            },
        )
        .await
        .expect("create batch");

        let generic: (String, String, String, String, Option<String>) = sqlx::query_as(
            "SELECT provider, ingest_kind, status, completeness, finished_at
             FROM ingest_batches WHERE id = ?",
        )
        .bind(batch_id)
        .fetch_one(&pool)
        .await
        .expect("load generic batch");
        assert_eq!(
            generic,
            (
                "telegram".to_string(),
                "takeout".to_string(),
                "running".to_string(),
                "unknown".to_string(),
                None
            )
        );

        let detail: (i64, String) = sqlx::query_as(
            "SELECT account_id, source_subtype FROM telegram_takeout_batches WHERE batch_id = ?",
        )
        .bind(batch_id)
        .fetch_one(&pool)
        .await
        .expect("load detail");
        assert_eq!(detail, (10, "supergroup".to_string()));
    }

    #[tokio::test]
    async fn terminal_update_recalculates_counters_and_sanitizes_error() {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_ingest_provenance_tables(&pool).await;
        seed_source(&pool).await;
        seed_item(&pool, 11).await;
        let batch_id = create_telegram_takeout_batch(
            &pool,
            CreateTelegramTakeoutBatch {
                source_id: 1,
                account_id: 10,
                source_subtype: "supergroup".to_string(),
            },
        )
        .await
        .expect("create batch");

        record_ingest_observation(
            &pool,
            IngestObservation {
                batch_id,
                source_id: 1,
                item_id: Some(11),
                provider_item_kind: "telegram_message",
                provider_identity_kind: "telegram_message",
                provider_identity: "telegram:history_peer:channel:12345:message:1".to_string(),
                outcome: "inserted",
                reason_code: None,
            },
        )
        .await
        .expect("record inserted");
        record_ingest_observation(
            &pool,
            IngestObservation {
                batch_id,
                source_id: 1,
                item_id: Some(11),
                provider_item_kind: "telegram_message",
                provider_identity_kind: "telegram_message",
                provider_identity: "telegram:history_peer:channel:12345:message:1".to_string(),
                outcome: "duplicate_observed",
                reason_code: None,
            },
        )
        .await
        .expect("record duplicate");
        record_ingest_observation(
            &pool,
            IngestObservation {
                batch_id,
                source_id: 1,
                item_id: None,
                provider_item_kind: "telegram_message",
                provider_identity_kind: "telegram_message",
                provider_identity: "telegram:history_peer:channel:12345:message:2".to_string(),
                outcome: "skipped",
                reason_code: Some("empty_payload"),
            },
        )
        .await
        .expect("record skipped");
        record_ingest_batch_warning(
            &pool,
            batch_id,
            "generic_warning",
            "{\"raw\":\"payload\",\"api_hash\":\"secret\"}",
        )
        .await
        .expect("record warning");

        finalize_ingest_batch(
            &pool,
            batch_id,
            TerminalBatchStatus::Failed,
            Some("{\"raw\":\"payload\",\"session\":\"secret\"}"),
        )
        .await
        .expect("finalize batch");

        let row: (String, String, i64, i64, i64, i64, i64, String) = sqlx::query_as(
            "SELECT status, completeness, item_observed_count, item_inserted_count,
                    item_duplicate_count, item_skipped_count, warning_count, terminal_error
             FROM ingest_batches WHERE id = ?",
        )
        .bind(batch_id)
        .fetch_one(&pool)
        .await
        .expect("load finalized batch");

        assert_eq!(row.0, "failed");
        assert_eq!(row.1, "partial");
        assert_eq!(row.2, 3);
        assert_eq!(row.3, 1);
        assert_eq!(row.4, 1);
        assert_eq!(row.5, 1);
        assert_eq!(row.6, 1);
        assert!(!row.7.starts_with('{'));
        assert!(!row.7.contains("session"));

        let warning_message: String =
            sqlx::query_scalar("SELECT message FROM ingest_batch_warnings WHERE batch_id = ?")
                .bind(batch_id)
                .fetch_one(&pool)
                .await
                .expect("load warning");
        assert!(!warning_message.starts_with('{'));
        assert!(!warning_message.contains("api_hash"));
    }

    #[tokio::test]
    async fn completed_zero_observation_batch_is_complete_without_partial_flags() {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_ingest_provenance_tables(&pool).await;
        seed_source(&pool).await;
        let batch_id = create_telegram_takeout_batch(
            &pool,
            CreateTelegramTakeoutBatch {
                source_id: 1,
                account_id: 10,
                source_subtype: "supergroup".to_string(),
            },
        )
        .await
        .expect("create batch");

        finalize_ingest_batch(&pool, batch_id, TerminalBatchStatus::Completed, None)
            .await
            .expect("finalize complete empty batch");

        let completeness: String =
            sqlx::query_scalar("SELECT completeness FROM ingest_batches WHERE id = ?")
                .bind(batch_id)
                .fetch_one(&pool)
                .await
                .expect("load completeness");
        assert_eq!(completeness, "complete");
    }

    #[tokio::test]
    async fn mixed_partial_scope_finalizes_as_partial() {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_ingest_provenance_tables(&pool).await;
        seed_source(&pool).await;
        let batch_id = create_telegram_takeout_batch(
            &pool,
            CreateTelegramTakeoutBatch {
                source_id: 1,
                account_id: 10,
                source_subtype: "supergroup".to_string(),
            },
        )
        .await
        .expect("create batch");
        mark_takeout_only_my_messages_fallback(&pool, batch_id, "private history")
            .await
            .expect("mark private fallback");
        mark_takeout_migrated_history_deferred(&pool, batch_id, "migrated deferred")
            .await
            .expect("mark migrated deferred");

        finalize_ingest_batch(&pool, batch_id, TerminalBatchStatus::Completed, None)
            .await
            .expect("finalize partial batch");

        let row: (String, String) = sqlx::query_as(
            "SELECT b.completeness, t.history_scope
             FROM ingest_batches b
             JOIN telegram_takeout_batches t ON t.batch_id = b.id
             WHERE b.id = ?",
        )
        .bind(batch_id)
        .fetch_one(&pool)
        .await
        .expect("load final state");
        assert_eq!(row, ("partial".to_string(), "mixed_partial".to_string()));
    }
}
