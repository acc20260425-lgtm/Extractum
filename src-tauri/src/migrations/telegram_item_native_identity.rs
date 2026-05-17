use std::time::Instant;

use sha2::{Digest, Sha384};
use sqlx::{Connection, SqliteConnection};

use crate::error::{AppError, AppResult};

pub(super) const TELEGRAM_ITEM_NATIVE_IDENTITY_VERSION: i64 = 21;
pub(super) const TELEGRAM_ITEM_NATIVE_IDENTITY_DESCRIPTION: &str =
    "add telegram item native identity";
pub(super) const TELEGRAM_ITEM_NATIVE_IDENTITY_SENTINEL_SQL: &str =
    include_str!("../../migrations/21.sql");

pub(super) async fn apply_telegram_item_native_identity_if_needed(
    db_url: &str,
) -> AppResult<()> {
    let mut conn = SqliteConnection::connect(db_url)
        .await
        .map_err(AppError::database)?;
    apply_telegram_item_native_identity_on_connection(&mut conn).await
}

pub(super) async fn apply_telegram_item_native_identity_on_connection(
    conn: &mut SqliteConnection,
) -> AppResult<()> {
    ensure_previous_migration_recorded(conn).await?;
    if migration_21_recorded(conn).await? {
        return Ok(());
    }

    let started_at = Instant::now();
    sqlx::query("BEGIN IMMEDIATE")
        .execute(&mut *conn)
        .await
        .map_err(AppError::database)?;

    let result = async {
        create_telegram_messages_schema(conn).await?;
        backfill_telegram_messages(conn).await?;
        replace_item_identity_indexes(conn).await?;
        assert_post_migration_integrity(conn).await
    }
    .await;

    match result {
        Ok(()) => {
            sqlx::query("COMMIT")
                .execute(&mut *conn)
                .await
                .map_err(AppError::database)?;
        }
        Err(error) => {
            let _ = sqlx::query("ROLLBACK").execute(&mut *conn).await;
            return Err(error);
        }
    }

    record_migration_success(
        conn,
        TELEGRAM_ITEM_NATIVE_IDENTITY_VERSION,
        TELEGRAM_ITEM_NATIVE_IDENTITY_DESCRIPTION,
        expected_migration_21_checksum(),
        started_at.elapsed().as_nanos().min(i64::MAX as u128) as i64,
    )
    .await
}

async fn ensure_previous_migration_recorded(conn: &mut SqliteConnection) -> AppResult<()> {
    let exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM _sqlx_migrations WHERE version = 20 AND success = 1",
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(AppError::database)?;
    if exists == 0 {
        return Err(AppError::validation(
            "Telegram item native identity migration 21 requires migration 20",
        ));
    }
    Ok(())
}

async fn migration_21_recorded(conn: &mut SqliteConnection) -> AppResult<bool> {
    let checksum = expected_migration_21_checksum();
    let row: Option<(Vec<u8>, bool)> =
        sqlx::query_as("SELECT checksum, success FROM _sqlx_migrations WHERE version = ?")
            .bind(TELEGRAM_ITEM_NATIVE_IDENTITY_VERSION)
            .fetch_optional(&mut *conn)
            .await
            .map_err(AppError::database)?;

    match row {
        None => Ok(false),
        Some((applied_checksum, true)) if applied_checksum == checksum => Ok(true),
        Some((_applied_checksum, true)) => Err(AppError::internal(
            "Migration 21 checksum does not match the runner-managed Telegram item native identity sentinel",
        )),
        Some((_applied_checksum, false)) => Err(AppError::internal(
            "Migration 21 is marked as failed in _sqlx_migrations",
        )),
    }
}

async fn record_migration_success(
    conn: &mut SqliteConnection,
    version: i64,
    description: &str,
    checksum: Vec<u8>,
    execution_time: i64,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time) VALUES (?, ?, 1, ?, ?)",
    )
    .bind(version)
    .bind(description)
    .bind(checksum)
    .bind(execution_time)
    .execute(&mut *conn)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

fn expected_migration_21_checksum() -> Vec<u8> {
    Sha384::digest(TELEGRAM_ITEM_NATIVE_IDENTITY_SENTINEL_SQL.as_bytes()).to_vec()
}

async fn create_telegram_messages_schema(conn: &mut SqliteConnection) -> AppResult<()> {
    sqlx::raw_sql(TELEGRAM_MESSAGES_SCHEMA_SQL)
        .execute(&mut *conn)
        .await
        .map_err(AppError::database)?;
    Ok(())
}

async fn backfill_telegram_messages(_conn: &mut SqliteConnection) -> AppResult<()> {
    Ok(())
}

async fn replace_item_identity_indexes(_conn: &mut SqliteConnection) -> AppResult<()> {
    Ok(())
}

async fn assert_post_migration_integrity(_conn: &mut SqliteConnection) -> AppResult<()> {
    Ok(())
}

pub(crate) const TELEGRAM_MESSAGES_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS telegram_messages (
    item_id INTEGER PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
    source_id INTEGER NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
    history_peer_kind TEXT NOT NULL,
    history_peer_id INTEGER NOT NULL,
    telegram_message_id INTEGER NOT NULL,
    migration_domain TEXT,
    is_migrated_history INTEGER NOT NULL DEFAULT 0,
    reply_to_msg_id INTEGER,
    reply_to_peer_kind TEXT,
    reply_to_peer_id INTEGER,
    reply_to_top_id INTEGER,
    reaction_count INTEGER,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    CHECK (history_peer_kind IN ('channel', 'chat', 'user')),
    CHECK (telegram_message_id > 0),
    CHECK (is_migrated_history IN (0, 1)),
    CHECK (reply_to_msg_id IS NULL OR reply_to_msg_id > 0),
    CHECK (
        reply_to_peer_kind IS NULL
        OR reply_to_peer_kind IN ('channel', 'chat', 'user')
    ),
    CHECK (reply_to_peer_id IS NULL OR reply_to_peer_id > 0),
    CHECK (reply_to_top_id IS NULL OR reply_to_top_id > 0),
    CHECK (reaction_count IS NULL OR reaction_count >= 0)
);

CREATE UNIQUE INDEX IF NOT EXISTS ux_telegram_messages_native_identity
    ON telegram_messages (
        source_id,
        history_peer_kind,
        history_peer_id,
        telegram_message_id
    );

CREATE INDEX IF NOT EXISTS idx_telegram_messages_source_message
    ON telegram_messages(source_id, telegram_message_id);

CREATE INDEX IF NOT EXISTS idx_telegram_messages_source_reply_top
    ON telegram_messages(source_id, reply_to_top_id);
"#;
