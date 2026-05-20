use std::time::Instant;

use sha2::{Digest, Sha384};
use sqlx::{Connection, SqliteConnection};

use crate::error::{AppError, AppResult};
use crate::tx::{begin_immediate_on_connection, finish_connection_transaction};

pub(super) const TELEGRAM_ITEM_NATIVE_IDENTITY_VERSION: i64 = 21;
pub(super) const TELEGRAM_ITEM_NATIVE_IDENTITY_DESCRIPTION: &str =
    "add telegram item native identity";
pub(super) const TELEGRAM_ITEM_NATIVE_IDENTITY_SENTINEL_SQL: &str =
    include_str!("../../migrations/21.sql");

pub(super) async fn apply_telegram_item_native_identity_if_needed(db_url: &str) -> AppResult<()> {
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
    begin_immediate_on_connection(conn).await?;

    let result = async {
        create_telegram_messages_schema(conn).await?;
        let stats = backfill_telegram_messages(conn).await?;
        let _backfilled = stats.backfilled;
        let _skipped = stats.skipped;
        replace_item_identity_indexes(conn).await?;
        assert_post_migration_integrity(conn).await
    }
    .await;

    finish_connection_transaction(conn, result).await?;

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

#[derive(sqlx::FromRow)]
struct ForeignKeyCheckRow {
    table: String,
    rowid: Option<i64>,
    parent: String,
    fkid: i64,
}

#[derive(Debug, PartialEq, Eq)]
struct BackfillStats {
    backfilled: i64,
    skipped: i64,
}

async fn backfill_telegram_messages(conn: &mut SqliteConnection) -> AppResult<BackfillStats> {
    let before_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM telegram_messages")
        .fetch_one(&mut *conn)
        .await
        .map_err(AppError::database)?;

    sqlx::query(
        r#"
        INSERT OR IGNORE INTO telegram_messages (
            item_id,
            source_id,
            history_peer_kind,
            history_peer_id,
            telegram_message_id,
            migration_domain,
            is_migrated_history,
            reply_to_msg_id,
            reply_to_peer_kind,
            reply_to_peer_id,
            reply_to_top_id,
            reaction_count,
            created_at,
            updated_at
        )
        SELECT
            items.id,
            items.source_id,
            telegram_sources.peer_kind,
            telegram_sources.peer_id,
            CAST(items.external_id AS INTEGER),
            NULL,
            0,
            CASE WHEN items.reply_to_msg_id > 0 THEN items.reply_to_msg_id ELSE NULL END,
            CASE
                WHEN items.reply_to_peer_kind IN ('channel', 'chat', 'user')
                THEN items.reply_to_peer_kind
                ELSE NULL
            END,
            CASE
                WHEN items.reply_to_peer_id IS NOT NULL
                 AND items.reply_to_peer_id <> ''
                 AND items.reply_to_peer_id NOT GLOB '*[^0-9]*'
                 AND CAST(items.reply_to_peer_id AS INTEGER) > 0
                THEN CAST(items.reply_to_peer_id AS INTEGER)
                ELSE NULL
            END,
            CASE WHEN items.reply_to_top_id > 0 THEN items.reply_to_top_id ELSE NULL END,
            CASE WHEN items.reaction_count >= 0 THEN items.reaction_count ELSE NULL END,
            strftime('%s','now'),
            strftime('%s','now')
        FROM items
        JOIN sources ON sources.id = items.source_id
        JOIN telegram_sources ON telegram_sources.source_id = sources.id
        WHERE items.item_kind = 'telegram_message'
          AND sources.source_type = 'telegram'
          AND items.external_id <> ''
          AND items.external_id NOT GLOB '*[^0-9]*'
          AND CAST(items.external_id AS INTEGER) > 0
        "#,
    )
    .execute(&mut *conn)
    .await
    .map_err(AppError::database)?;

    let after_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM telegram_messages")
        .fetch_one(&mut *conn)
        .await
        .map_err(AppError::database)?;
    let candidate_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM items WHERE item_kind = 'telegram_message'")
            .fetch_one(&mut *conn)
            .await
            .map_err(AppError::database)?;

    let backfilled = after_count - before_count;
    Ok(BackfillStats {
        backfilled,
        skipped: (candidate_count - backfilled).max(0),
    })
}

async fn replace_item_identity_indexes(conn: &mut SqliteConnection) -> AppResult<()> {
    assert_no_null_item_kind(conn).await?;
    assert_no_non_telegram_duplicate_external_ids(conn).await?;

    sqlx::raw_sql(
        r#"
        DROP INDEX IF EXISTS idx_items_ext;

        CREATE UNIQUE INDEX IF NOT EXISTS ux_items_non_telegram_external
            ON items(source_id, external_id)
            WHERE item_kind <> 'telegram_message';

        CREATE INDEX IF NOT EXISTS idx_items_source_external
            ON items(source_id, external_id);
        "#,
    )
    .execute(&mut *conn)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

async fn assert_no_null_item_kind(conn: &mut SqliteConnection) -> AppResult<()> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM items WHERE item_kind IS NULL")
        .fetch_one(&mut *conn)
        .await
        .map_err(AppError::database)?;
    if count != 0 {
        return Err(AppError::validation(format!(
            "Telegram item native identity migration 21 found {count} items with NULL item_kind"
        )));
    }
    Ok(())
}

async fn assert_no_non_telegram_duplicate_external_ids(
    conn: &mut SqliteConnection,
) -> AppResult<()> {
    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM (
            SELECT source_id, external_id
            FROM items
            WHERE item_kind <> 'telegram_message'
            GROUP BY source_id, external_id
            HAVING COUNT(*) > 1
        )
        "#,
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(AppError::database)?;
    if count != 0 {
        return Err(AppError::validation(format!(
            "Telegram item native identity migration 21 found {count} non-Telegram duplicate item external ids"
        )));
    }
    Ok(())
}

async fn assert_post_migration_integrity(conn: &mut SqliteConnection) -> AppResult<()> {
    assert_foreign_key_check_clean(conn).await?;
    assert_no_telegram_message_item_kind_mismatch(conn).await?;
    assert_no_telegram_message_source_mismatch(conn).await?;
    assert_no_duplicate_native_telegram_identity(conn).await?;
    Ok(())
}

async fn assert_foreign_key_check_clean(conn: &mut SqliteConnection) -> AppResult<()> {
    let rows: Vec<ForeignKeyCheckRow> = sqlx::query_as("PRAGMA foreign_key_check")
        .fetch_all(&mut *conn)
        .await
        .map_err(AppError::database)?;
    if rows.is_empty() {
        return Ok(());
    }

    let detail = rows
        .into_iter()
        .map(|row| {
            format!(
                "{} rowid {:?} references {} via fk {}",
                row.table, row.rowid, row.parent, row.fkid
            )
        })
        .collect::<Vec<_>>()
        .join("; ");
    Err(AppError::validation(format!(
        "Telegram item native identity migration 21 foreign_key_check failed: {detail}"
    )))
}

async fn assert_no_telegram_message_item_kind_mismatch(
    conn: &mut SqliteConnection,
) -> AppResult<()> {
    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM telegram_messages tm
        JOIN items i ON i.id = tm.item_id
        WHERE i.item_kind <> 'telegram_message'
        "#,
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(AppError::database)?;
    if count != 0 {
        return Err(AppError::validation(format!(
            "Migration 21 found {count} telegram_messages rows pointing to non-Telegram items"
        )));
    }
    Ok(())
}

async fn assert_no_telegram_message_source_mismatch(conn: &mut SqliteConnection) -> AppResult<()> {
    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM telegram_messages tm
        JOIN items i ON i.id = tm.item_id
        WHERE tm.source_id <> i.source_id
        "#,
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(AppError::database)?;
    if count != 0 {
        return Err(AppError::validation(format!(
            "Migration 21 found {count} telegram_messages rows with source_id mismatch"
        )));
    }
    Ok(())
}

async fn assert_no_duplicate_native_telegram_identity(
    conn: &mut SqliteConnection,
) -> AppResult<()> {
    let count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM (
            SELECT source_id, history_peer_kind, history_peer_id, telegram_message_id
            FROM telegram_messages
            GROUP BY source_id, history_peer_kind, history_peer_id, telegram_message_id
            HAVING COUNT(*) > 1
        )
        "#,
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(AppError::database)?;
    if count != 0 {
        return Err(AppError::validation(format!(
            "Migration 21 found {count} duplicate Telegram native message identities"
        )));
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compression::compress_text;
    use crate::migrations::build_migrations;
    use sqlx::SqliteConnection;

    #[tokio::test]
    async fn migration_21_backfills_valid_telegram_rows_and_skips_malformed_rows() {
        let mut conn = memory_conn_with_history_through_20().await;
        seed_telegram_source(&mut conn, 101, "channel", 12345).await;
        insert_telegram_item(
            &mut conn,
            301,
            101,
            "42",
            Some(7),
            Some("channel"),
            Some("12345"),
            Some(5),
            Some(2),
        )
        .await;
        insert_telegram_item(&mut conn, 302, 101, "bad-42", None, None, None, None, None).await;
        insert_telegram_item(&mut conn, 303, 101, " 43", None, None, None, None, None).await;

        apply_telegram_item_native_identity_on_connection(&mut conn)
            .await
            .expect("apply v21");

        let rows: Vec<(
            i64,
            i64,
            String,
            i64,
            i64,
            Option<i64>,
            Option<String>,
            Option<i64>,
            Option<i64>,
            Option<i64>,
        )> = sqlx::query_as(
            "SELECT item_id, source_id, history_peer_kind, history_peer_id, telegram_message_id, reply_to_msg_id, reply_to_peer_kind, reply_to_peer_id, reply_to_top_id, reaction_count FROM telegram_messages ORDER BY item_id",
        )
        .fetch_all(&mut conn)
        .await
        .expect("load typed rows");

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].0, 301);
        assert_eq!(rows[0].1, 101);
        assert_eq!(rows[0].2, "channel");
        assert_eq!(rows[0].3, 12345);
        assert_eq!(rows[0].4, 42);
        assert_eq!(rows[0].5, Some(7));
        assert_eq!(rows[0].6.as_deref(), Some("channel"));
        assert_eq!(rows[0].7, Some(12345));
        assert_eq!(rows[0].8, Some(5));
        assert_eq!(rows[0].9, Some(2));
    }

    #[tokio::test]
    async fn migration_21_allows_same_message_id_across_history_domains() {
        let mut conn = memory_conn_with_history_through_20().await;
        seed_telegram_source(&mut conn, 101, "channel", 12345).await;
        insert_telegram_item(&mut conn, 301, 101, "42", None, None, None, None, None).await;

        apply_telegram_item_native_identity_on_connection(&mut conn)
            .await
            .expect("apply v21");

        sqlx::query(
            "INSERT INTO items (id, source_id, external_id, item_kind, author, published_at, ingested_at, content_zstd, content_kind, has_media)
             VALUES (302, 101, '42', 'telegram_message', 'bob', 2, 2, ?, 'text_only', 0)",
        )
        .bind(compress_text("migrated").expect("compress"))
        .execute(&mut conn)
        .await
        .expect("insert overlapping legacy item");

        sqlx::query(
            "INSERT INTO telegram_messages (item_id, source_id, history_peer_kind, history_peer_id, telegram_message_id, is_migrated_history)
             VALUES (302, 101, 'chat', 777, 42, 1)",
        )
        .execute(&mut conn)
        .await
        .expect("insert migrated identity");

        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM items WHERE source_id = 101 AND external_id = '42'",
        )
        .fetch_one(&mut conn)
        .await
        .expect("count overlapping ids");
        assert_eq!(count, 2);
    }

    #[tokio::test]
    async fn migration_21_rejects_null_item_kind_before_replacing_idx_items_ext() {
        let mut conn = memory_conn_with_nullable_item_kind_recorded_through_20().await;
        sqlx::query("INSERT INTO items (id, source_id, external_id, item_kind, author, published_at, ingested_at, content_kind, has_media) VALUES (900, 1, 'null-kind', NULL, 'nobody', 1, 1, 'text_only', 0)")
            .execute(&mut conn)
            .await
            .expect("insert null item_kind fixture");

        let error = apply_telegram_item_native_identity_on_connection(&mut conn)
            .await
            .expect_err("null item_kind blocks migration");

        assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
        assert!(error.message.contains("item_kind"));
    }

    #[tokio::test]
    async fn migration_21_rejects_existing_non_telegram_duplicate_external_ids() {
        let mut conn = memory_conn_with_history_through_20().await;
        sqlx::query("DROP INDEX IF EXISTS idx_items_ext")
            .execute(&mut conn)
            .await
            .expect("drop legacy uniqueness for dirty fixture");
        insert_youtube_item(&mut conn, 801, 201, "comment:dup", "youtube_comment").await;
        insert_youtube_item(&mut conn, 802, 201, "comment:dup", "youtube_comment").await;

        let error = apply_telegram_item_native_identity_on_connection(&mut conn)
            .await
            .expect_err("duplicates block partial unique index");

        assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
        assert!(error.message.contains("non-Telegram duplicate"));
    }

    #[tokio::test]
    async fn migration_21_records_sentinel_checksum_and_is_idempotent() {
        let mut conn = memory_conn_with_history_through_20().await;

        apply_telegram_item_native_identity_on_connection(&mut conn)
            .await
            .expect("first v21");
        apply_telegram_item_native_identity_on_connection(&mut conn)
            .await
            .expect("second v21");

        let row: (String, bool, Vec<u8>) = sqlx::query_as(
            "SELECT description, success, checksum FROM _sqlx_migrations WHERE version = 21",
        )
        .fetch_one(&mut conn)
        .await
        .expect("load v21 history");
        assert_eq!(row.0, TELEGRAM_ITEM_NATIVE_IDENTITY_DESCRIPTION);
        assert!(row.1);
        assert_eq!(row.2, expected_migration_21_checksum());
    }

    async fn memory_conn_with_history_through_20() -> SqliteConnection {
        let mut conn = SqliteConnection::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS _sqlx_migrations (
                version BIGINT PRIMARY KEY,
                description TEXT NOT NULL,
                installed_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                success BOOLEAN NOT NULL,
                checksum BLOB NOT NULL,
                execution_time BIGINT NOT NULL
            )",
        )
        .execute(&mut conn)
        .await
        .expect("create migration history");

        for migration in build_migrations()
            .into_iter()
            .filter(|migration| migration.version < 19)
        {
            sqlx::raw_sql(migration.sql)
                .execute(&mut conn)
                .await
                .unwrap_or_else(|error| panic!("apply migration {}: {error}", migration.version));
            sqlx::query("INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time) VALUES (?, ?, 1, ?, 0)")
                .bind(migration.version)
                .bind(migration.description)
                .bind(Sha384::digest(migration.sql.as_bytes()).to_vec())
                .execute(&mut conn)
                .await
                .expect("record standard migration");
        }

        crate::migrations::source_identity_cleanup::apply_source_identity_cleanup_on_connection(
            &mut conn,
        )
        .await
        .expect("apply v19");
        crate::migrations::youtube_typed_source_metadata::apply_youtube_typed_source_metadata_on_connection(
            &mut conn,
        )
        .await
        .expect("apply v20");

        conn
    }

    async fn memory_conn_with_nullable_item_kind_recorded_through_20() -> SqliteConnection {
        let mut conn = SqliteConnection::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");

        sqlx::query(
            "CREATE TABLE _sqlx_migrations (
                version BIGINT PRIMARY KEY,
                description TEXT NOT NULL,
                installed_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                success BOOLEAN NOT NULL,
                checksum BLOB NOT NULL,
                execution_time BIGINT NOT NULL
            )",
        )
        .execute(&mut conn)
        .await
        .expect("create migration history");
        sqlx::query(
            "INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time)
             VALUES (20, 'add youtube typed source metadata', 1, ?, 0)",
        )
        .bind(
            build_migrations()
                .into_iter()
                .find(|migration| migration.version == 20)
                .map(|migration| Sha384::digest(migration.sql.as_bytes()).to_vec())
                .expect("v20 migration"),
        )
        .execute(&mut conn)
        .await
        .expect("record v20");

        sqlx::query(
            "CREATE TABLE sources (
                id INTEGER PRIMARY KEY,
                source_type TEXT NOT NULL,
                source_subtype TEXT,
                external_id TEXT NOT NULL
            )",
        )
        .execute(&mut conn)
        .await
        .expect("create minimal sources");
        sqlx::query(
            "CREATE TABLE telegram_sources (
                source_id INTEGER PRIMARY KEY,
                peer_kind TEXT NOT NULL,
                peer_id INTEGER NOT NULL
            )",
        )
        .execute(&mut conn)
        .await
        .expect("create minimal telegram_sources");
        sqlx::query(
            "CREATE TABLE items (
                id INTEGER PRIMARY KEY,
                source_id INTEGER NOT NULL,
                external_id TEXT NOT NULL,
                item_kind TEXT,
                author TEXT,
                published_at INTEGER,
                ingested_at INTEGER NOT NULL,
                content_zstd BLOB,
                content_kind TEXT NOT NULL DEFAULT 'text_only',
                has_media INTEGER NOT NULL DEFAULT 0,
                reply_to_msg_id INTEGER,
                reply_to_peer_kind TEXT,
                reply_to_peer_id TEXT,
                reply_to_top_id INTEGER,
                reaction_count INTEGER
            )",
        )
        .execute(&mut conn)
        .await
        .expect("create minimal nullable items");

        conn
    }

    async fn seed_telegram_source(
        conn: &mut SqliteConnection,
        source_id: i64,
        peer_kind: &str,
        peer_id: i64,
    ) {
        sqlx::query("INSERT OR IGNORE INTO accounts (id, label, api_id, api_hash, created_at) VALUES (1, 'acct', 1, 'hash', 1)")
            .execute(&mut *conn)
            .await
            .expect("insert account");
        sqlx::query("INSERT OR IGNORE INTO sources (id, source_type, source_subtype, account_id, external_id, title, is_active, is_member, created_at) VALUES (?, 'telegram', 'supergroup', 1, ?, 'Forum', 1, 1, 1)")
            .bind(source_id)
            .bind(peer_id.to_string())
            .execute(&mut *conn)
            .await
            .expect("insert source");
        sqlx::query("INSERT INTO telegram_sources (source_id, account_id, source_subtype, peer_kind, peer_id, resolution_strategy) VALUES (?, 1, 'supergroup', ?, ?, 'dialog')")
            .bind(source_id)
            .bind(peer_kind)
            .bind(peer_id)
            .execute(&mut *conn)
            .await
            .expect("insert telegram source");
    }

    async fn insert_telegram_item(
        conn: &mut SqliteConnection,
        id: i64,
        source_id: i64,
        external_id: &str,
        reply_to_msg_id: Option<i64>,
        reply_to_peer_kind: Option<&str>,
        reply_to_peer_id: Option<&str>,
        reply_to_top_id: Option<i64>,
        reaction_count: Option<i64>,
    ) {
        sqlx::query(
            "INSERT INTO items (
                id, source_id, external_id, item_kind, author, published_at, ingested_at,
                content_zstd, content_kind, has_media, reply_to_msg_id, reply_to_peer_kind,
                reply_to_peer_id, reply_to_top_id, reaction_count
             ) VALUES (?, ?, ?, 'telegram_message', 'alice', 1, 1, ?, 'text_only', 0, ?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(source_id)
        .bind(external_id)
        .bind(compress_text("hello").expect("compress"))
        .bind(reply_to_msg_id)
        .bind(reply_to_peer_kind)
        .bind(reply_to_peer_id)
        .bind(reply_to_top_id)
        .bind(reaction_count)
        .execute(&mut *conn)
        .await
        .expect("insert telegram item");
    }

    async fn insert_youtube_item(
        conn: &mut SqliteConnection,
        id: i64,
        source_id: i64,
        external_id: &str,
        item_kind: &str,
    ) {
        sqlx::query("INSERT OR IGNORE INTO sources (id, source_type, source_subtype, external_id, title, is_active, is_member, created_at) VALUES (?, 'youtube', 'video', 'video-1', 'Video', 1, 0, 1)")
            .bind(source_id)
            .execute(&mut *conn)
            .await
            .expect("insert youtube source");
        sqlx::query("INSERT INTO items (id, source_id, external_id, item_kind, author, published_at, ingested_at, content_zstd, content_kind, has_media) VALUES (?, ?, ?, ?, 'yt', 1, 1, ?, 'text_only', 0)")
            .bind(id)
            .bind(source_id)
            .bind(external_id)
            .bind(item_kind)
            .bind(compress_text("youtube").expect("compress"))
            .execute(&mut *conn)
            .await
            .expect("insert youtube item");
    }
}
