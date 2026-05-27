#![allow(dead_code)]

use crate::error::{AppError, AppResult};
use crate::sources::{MIGRATED_HISTORY_STATUS_AVAILABLE, MIGRATED_HISTORY_STATUS_UNAVAILABLE};

pub(crate) const MIGRATED_HISTORY_REASON_NOT_DETECTED: &str = "not_detected";
pub(crate) const MIGRATED_HISTORY_REASON_MISSING_FROM_CHAT_ID: &str =
    "missing_migrated_from_chat_id";
pub(crate) const MIGRATED_HISTORY_REASON_CURRENT_SOURCE_UNAVAILABLE: &str =
    "current_source_unavailable";
pub(crate) const MIGRATED_HISTORY_REASON_OLD_CHAT_INPUT_UNAVAILABLE: &str =
    "old_chat_input_unavailable";
pub(crate) const MIGRATED_HISTORY_REASON_REVALIDATION_FAILED: &str = "revalidation_failed";

#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub(crate) struct MigratedHistoryCapability {
    pub(crate) source_id: i64,
    pub(crate) status: String,
    pub(crate) unavailable_reason: Option<String>,
    pub(crate) migrated_from_chat_id: Option<i64>,
    pub(crate) detected_at: Option<i64>,
    pub(crate) refreshed_at: i64,
}

pub(crate) async fn create_migrated_history_capability_schema(
    pool: &sqlx::SqlitePool,
) -> AppResult<()> {
    sqlx::raw_sql(MIGRATED_HISTORY_CAPABILITY_SCHEMA_SQL)
        .execute(pool)
        .await
        .map_err(AppError::database)?;
    Ok(())
}

pub(crate) async fn load_migrated_history_capability(
    pool: &sqlx::SqlitePool,
    source_id: i64,
) -> AppResult<Option<MigratedHistoryCapability>> {
    sqlx::query_as(
        "SELECT source_id, status, unavailable_reason, migrated_from_chat_id,
                detected_at, refreshed_at
         FROM telegram_migrated_history_capabilities
         WHERE source_id = ?",
    )
    .bind(source_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)
}

pub(crate) async fn upsert_migrated_history_available(
    pool: &sqlx::SqlitePool,
    source_id: i64,
    migrated_from_chat_id: i64,
    observed_at: i64,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO telegram_migrated_history_capabilities (
             source_id, status, unavailable_reason, migrated_from_chat_id,
             detected_at, refreshed_at
         ) VALUES (?, ?, NULL, ?, ?, ?)
         ON CONFLICT(source_id) DO UPDATE SET
             status = excluded.status,
             unavailable_reason = NULL,
             migrated_from_chat_id = excluded.migrated_from_chat_id,
             detected_at = COALESCE(
                 telegram_migrated_history_capabilities.detected_at,
                 excluded.detected_at
             ),
             refreshed_at = excluded.refreshed_at",
    )
    .bind(source_id)
    .bind(MIGRATED_HISTORY_STATUS_AVAILABLE)
    .bind(migrated_from_chat_id)
    .bind(observed_at)
    .bind(observed_at)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

pub(crate) async fn mark_migrated_history_unavailable(
    pool: &sqlx::SqlitePool,
    source_id: i64,
    reason: &str,
    observed_at: i64,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO telegram_migrated_history_capabilities (
             source_id, status, unavailable_reason, migrated_from_chat_id,
             detected_at, refreshed_at
         ) VALUES (?, ?, ?, NULL, NULL, ?)
         ON CONFLICT(source_id) DO UPDATE SET
             status = excluded.status,
             unavailable_reason = excluded.unavailable_reason,
             migrated_from_chat_id = NULL,
             refreshed_at = excluded.refreshed_at",
    )
    .bind(source_id)
    .bind(MIGRATED_HISTORY_STATUS_UNAVAILABLE)
    .bind(reason)
    .bind(observed_at)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

pub(crate) const MIGRATED_HISTORY_CAPABILITY_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS telegram_migrated_history_capabilities (
    source_id INTEGER PRIMARY KEY REFERENCES telegram_sources(source_id) ON DELETE CASCADE,
    status TEXT NOT NULL,
    unavailable_reason TEXT,
    migrated_from_chat_id INTEGER,
    detected_at INTEGER,
    refreshed_at INTEGER NOT NULL,
    CHECK (status IN ('none', 'available', 'unavailable')),
    CHECK (
        unavailable_reason IS NULL
        OR unavailable_reason IN (
            'not_detected',
            'missing_migrated_from_chat_id',
            'current_source_unavailable',
            'old_chat_input_unavailable',
            'revalidation_failed'
        )
    ),
    CHECK (migrated_from_chat_id IS NULL OR migrated_from_chat_id > 0),
    CHECK (status <> 'available' OR migrated_from_chat_id IS NOT NULL),
    CHECK (status <> 'unavailable' OR unavailable_reason IS NOT NULL)
);

CREATE INDEX IF NOT EXISTS idx_telegram_migrated_history_capabilities_status
    ON telegram_migrated_history_capabilities(status);
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sources::test_support::{
        create_migrated_history_capability_tables, memory_pool_with_sources,
    };

    async fn seed_telegram_source(pool: &sqlx::SqlitePool) {
        sqlx::query(
            "INSERT INTO sources (
                id, source_type, source_subtype, account_id, external_id, title,
                is_active, is_member, created_at
             ) VALUES (1, 'telegram', 'supergroup', 10, '12345', 'Forum', 1, 1, 1)",
        )
        .execute(pool)
        .await
        .expect("seed source");
        sqlx::query(
            "INSERT INTO telegram_sources (
                source_id, account_id, source_subtype, peer_kind, peer_id, resolution_strategy
             ) VALUES (1, 10, 'supergroup', 'channel', 12345, 'dialog')",
        )
        .execute(pool)
        .await
        .expect("seed telegram source");
    }

    #[tokio::test]
    async fn capability_available_is_source_level_and_restart_safe() {
        let pool = memory_pool_with_sources().await;
        create_migrated_history_capability_tables(&pool).await;
        seed_telegram_source(&pool).await;

        upsert_migrated_history_available(&pool, 1, 777, 100)
            .await
            .expect("mark available");
        upsert_migrated_history_available(&pool, 1, 777, 200)
            .await
            .expect("refresh available");

        let capability = load_migrated_history_capability(&pool, 1)
            .await
            .expect("load capability")
            .expect("capability exists");

        assert_eq!(capability.status, MIGRATED_HISTORY_STATUS_AVAILABLE);
        assert_eq!(capability.unavailable_reason, None);
        assert_eq!(capability.migrated_from_chat_id, Some(777));
        assert_eq!(capability.detected_at, Some(100));
        assert_eq!(capability.refreshed_at, 200);
    }

    #[tokio::test]
    async fn capability_unavailable_keeps_reason_internal_and_clears_chat_hint() {
        let pool = memory_pool_with_sources().await;
        create_migrated_history_capability_tables(&pool).await;
        seed_telegram_source(&pool).await;

        upsert_migrated_history_available(&pool, 1, 777, 100)
            .await
            .expect("mark available");
        mark_migrated_history_unavailable(
            &pool,
            1,
            MIGRATED_HISTORY_REASON_OLD_CHAT_INPUT_UNAVAILABLE,
            250,
        )
        .await
        .expect("mark unavailable");

        let capability = load_migrated_history_capability(&pool, 1)
            .await
            .expect("load capability")
            .expect("capability exists");

        assert_eq!(capability.status, MIGRATED_HISTORY_STATUS_UNAVAILABLE);
        assert_eq!(
            capability.unavailable_reason.as_deref(),
            Some(MIGRATED_HISTORY_REASON_OLD_CHAT_INPUT_UNAVAILABLE)
        );
        assert_eq!(capability.migrated_from_chat_id, None);
        assert_eq!(capability.detected_at, Some(100));
        assert_eq!(capability.refreshed_at, 250);
    }
}
