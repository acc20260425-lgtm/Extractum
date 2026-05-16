use std::time::Instant;

use sha2::{Digest, Sha384};
use sqlx::{Connection, SqliteConnection};

use crate::error::{AppError, AppResult};

pub(super) const SOURCE_IDENTITY_CLEANUP_VERSION: i64 = 19;
pub(super) const SOURCE_IDENTITY_CLEANUP_DESCRIPTION: &str = "remove legacy telegram source kind";
pub(super) const SOURCE_IDENTITY_CLEANUP_SENTINEL_SQL: &str =
    include_str!("../../migrations/19.sql");

pub(super) async fn apply_source_identity_cleanup_if_needed(db_url: &str) -> AppResult<()> {
    let mut conn = SqliteConnection::connect(db_url)
        .await
        .map_err(AppError::database)?;
    apply_source_identity_cleanup_on_connection(&mut conn).await
}

async fn apply_source_identity_cleanup_on_connection(conn: &mut SqliteConnection) -> AppResult<()> {
    ensure_not_missing_previous_migrations(conn).await?;
    if migration_19_recorded(conn).await? {
        return Ok(());
    }

    let started_at = Instant::now();
    run_source_identity_cleanup_rebuild(conn).await?;
    record_migration_success(
        conn,
        SOURCE_IDENTITY_CLEANUP_VERSION,
        SOURCE_IDENTITY_CLEANUP_DESCRIPTION,
        expected_migration_19_checksum(),
        started_at.elapsed().as_nanos().min(i64::MAX as u128) as i64,
    )
    .await?;
    Ok(())
}

pub(super) async fn apply_standard_migrations_before_plugin(
    db_url: &str,
    migrations: Vec<tauri_plugin_sql::Migration>,
) -> AppResult<()> {
    let mut conn = SqliteConnection::connect(db_url)
        .await
        .map_err(AppError::database)?;
    ensure_sqlx_migrations_table(&mut conn).await?;

    for migration in migrations
        .into_iter()
        .filter(|migration| migration.version < SOURCE_IDENTITY_CLEANUP_VERSION)
    {
        let exists = migration_record_exists(&mut conn, migration.version).await?;
        if exists {
            continue;
        }

        let started_at = Instant::now();
        sqlx::raw_sql(migration.sql)
            .execute(&mut conn)
            .await
            .map_err(AppError::database)?;
        record_migration_success(
            &mut conn,
            migration.version,
            migration.description,
            Sha384::digest(migration.sql.as_bytes()).to_vec(),
            started_at.elapsed().as_nanos().min(i64::MAX as u128) as i64,
        )
        .await?;
    }

    Ok(())
}

async fn ensure_sqlx_migrations_table(conn: &mut SqliteConnection) -> AppResult<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS _sqlx_migrations (
            version BIGINT PRIMARY KEY,
            description TEXT NOT NULL,
            installed_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            success BOOLEAN NOT NULL,
            checksum BLOB NOT NULL,
            execution_time BIGINT NOT NULL
        )
        "#,
    )
    .execute(&mut *conn)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

async fn record_migration_success(
    conn: &mut SqliteConnection,
    version: i64,
    description: &str,
    checksum: Vec<u8>,
    execution_time: i64,
) -> AppResult<()> {
    sqlx::query(
        r#"
        INSERT INTO _sqlx_migrations (
            version, description, success, checksum, execution_time
        )
        VALUES (?, ?, 1, ?, ?)
        "#,
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

async fn migration_record_exists(conn: &mut SqliteConnection, version: i64) -> AppResult<bool> {
    let exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM _sqlx_migrations WHERE version = ? AND success = 1",
    )
    .bind(version)
    .fetch_one(&mut *conn)
    .await
    .map_err(AppError::database)?;
    Ok(exists != 0)
}

async fn run_source_identity_cleanup_rebuild(conn: &mut SqliteConnection) -> AppResult<()> {
    let sequence = captured_sources_sequence(conn).await?;

    sqlx::query("PRAGMA foreign_keys = OFF")
        .execute(&mut *conn)
        .await
        .map_err(AppError::database)?;
    let foreign_keys: i64 = sqlx::query_scalar("PRAGMA foreign_keys")
        .fetch_one(&mut *conn)
        .await
        .map_err(AppError::database)?;
    if foreign_keys != 0 {
        return Err(AppError::internal(
            "SQLite foreign_keys stayed enabled before source identity cleanup rebuild",
        ));
    }

    sqlx::query("BEGIN IMMEDIATE")
        .execute(&mut *conn)
        .await
        .map_err(AppError::database)?;

    let rebuild_result = async {
        rebuild_sources_table(conn).await?;
        restore_sources_sequence(conn, sequence).await?;
        assert_foreign_key_check_clean(conn, "inside v19 transaction").await
    }
    .await;

    match rebuild_result {
        Ok(()) => {
            sqlx::query("COMMIT")
                .execute(&mut *conn)
                .await
                .map_err(AppError::database)?;
        }
        Err(error) => {
            let _ = sqlx::query("ROLLBACK").execute(&mut *conn).await;
            let _ = sqlx::query("PRAGMA foreign_keys = ON")
                .execute(&mut *conn)
                .await;
            return Err(error);
        }
    }

    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&mut *conn)
        .await
        .map_err(AppError::database)?;
    assert_foreign_key_check_clean(conn, "after v19 commit").await
}

#[derive(sqlx::FromRow, Debug)]
struct ForeignKeyCheckRow {
    table: String,
    rowid: Option<i64>,
    parent: String,
    fkid: i64,
}

async fn assert_foreign_key_check_clean(conn: &mut SqliteConnection, phase: &str) -> AppResult<()> {
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
        "Source identity cleanup foreign_key_check failed {phase}: {detail}"
    )))
}

async fn captured_sources_sequence(conn: &mut SqliteConnection) -> AppResult<Option<i64>> {
    let seq = sqlx::query_scalar("SELECT seq FROM sqlite_sequence WHERE name = 'sources'")
        .fetch_optional(&mut *conn)
        .await
        .map_err(AppError::database)?;
    Ok(seq)
}

async fn restore_sources_sequence(conn: &mut SqliteConnection, seq: Option<i64>) -> AppResult<()> {
    sqlx::query("DELETE FROM sqlite_sequence WHERE name = 'sources'")
        .execute(&mut *conn)
        .await
        .map_err(AppError::database)?;

    if let Some(seq) = seq {
        sqlx::query("INSERT INTO sqlite_sequence(name, seq) VALUES ('sources', ?)")
            .bind(seq)
            .execute(&mut *conn)
            .await
            .map_err(AppError::database)?;
    }

    Ok(())
}

async fn rebuild_sources_table(conn: &mut SqliteConnection) -> AppResult<()> {
    sqlx::raw_sql(
        r#"
        DROP INDEX IF EXISTS idx_sources_ext;
        DROP INDEX IF EXISTS idx_sources_unique_telegram_identity;
        DROP INDEX IF EXISTS idx_sources_unique_youtube_video;
        DROP INDEX IF EXISTS idx_sources_unique_youtube_playlist;

        CREATE TABLE sources_new (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            source_type TEXT NOT NULL,
            source_subtype TEXT,
            external_id TEXT NOT NULL,
            title TEXT,
            metadata_zstd BLOB,
            last_sync_state INTEGER,
            is_active BOOLEAN DEFAULT 1,
            is_member BOOLEAN DEFAULT 0,
            created_at INTEGER NOT NULL,
            account_id INTEGER REFERENCES accounts(id) ON DELETE CASCADE,
            last_synced_at INTEGER,
            CHECK (
                source_type <> 'telegram'
                OR (
                    account_id IS NOT NULL
                    AND source_subtype IS NOT NULL
                    AND source_subtype IN ('channel', 'supergroup', 'group')
                )
            ),
            CHECK (
                source_type <> 'youtube'
                OR (
                    account_id IS NULL
                    AND source_subtype IS NOT NULL
                    AND source_subtype IN ('video', 'playlist')
                )
            )
        );

        INSERT INTO sources_new (
            id, source_type, source_subtype, external_id, title, metadata_zstd,
            last_sync_state, is_active, is_member, created_at, account_id,
            last_synced_at
        )
        SELECT
            id, source_type, source_subtype, external_id, title, metadata_zstd,
            last_sync_state, is_active, is_member, created_at, account_id,
            last_synced_at
        FROM sources;

        DROP TABLE sources;
        ALTER TABLE sources_new RENAME TO sources;

        CREATE UNIQUE INDEX idx_sources_unique_telegram_identity
            ON sources(account_id, source_type, source_subtype, external_id)
            WHERE source_type = 'telegram';

        CREATE UNIQUE INDEX idx_sources_unique_youtube_video
            ON sources(source_type, source_subtype, external_id)
            WHERE source_type = 'youtube' AND source_subtype = 'video';

        CREATE UNIQUE INDEX idx_sources_unique_youtube_playlist
            ON sources(source_type, source_subtype, external_id)
            WHERE source_type = 'youtube' AND source_subtype = 'playlist';
        "#,
    )
    .execute(&mut *conn)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

async fn ensure_not_missing_previous_migrations(conn: &mut SqliteConnection) -> AppResult<()> {
    let max_version: Option<i64> =
        sqlx::query_scalar("SELECT MAX(version) FROM _sqlx_migrations WHERE success = 1")
            .fetch_optional(&mut *conn)
            .await
            .map_err(AppError::database)?
            .flatten();

    match max_version {
        Some(version) if version >= 18 => Ok(()),
        Some(version) => Err(AppError::validation(format!(
            "Source identity cleanup requires migration 18 before migration 19; current migration version is {version}"
        ))),
        None => Err(AppError::validation(
            "Source identity cleanup requires migrations 1 through 18 before migration 19",
        )),
    }
}

async fn migration_19_recorded(conn: &mut SqliteConnection) -> AppResult<bool> {
    let checksum = expected_migration_19_checksum();
    let row: Option<(Vec<u8>, bool)> =
        sqlx::query_as("SELECT checksum, success FROM _sqlx_migrations WHERE version = ?")
            .bind(SOURCE_IDENTITY_CLEANUP_VERSION)
            .fetch_optional(&mut *conn)
            .await
            .map_err(AppError::database)?;

    match row {
        None => Ok(false),
        Some((applied_checksum, true)) if applied_checksum == checksum => Ok(true),
        Some((_applied_checksum, true)) => Err(AppError::internal(
            "Migration 19 checksum does not match the runner-managed source identity cleanup sentinel",
        )),
        Some((_applied_checksum, false)) => Err(AppError::internal(
            "Migration 19 is marked as failed in _sqlx_migrations",
        )),
    }
}

fn expected_migration_19_checksum() -> Vec<u8> {
    Sha384::digest(SOURCE_IDENTITY_CLEANUP_SENTINEL_SQL.as_bytes()).to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::migrations::build_migrations;

    async fn memory_conn_with_sqlx_history_through(version: i64) -> SqliteConnection {
        let mut conn = SqliteConnection::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");

        apply_standard_migrations_through(&mut conn, version)
            .await
            .expect("apply standard migrations");

        conn
    }

    #[tokio::test]
    async fn migration_19_sentinel_checksum_is_recorded() {
        let mut conn = memory_conn_with_sqlx_history_through(18).await;

        apply_source_identity_cleanup_on_connection(&mut conn)
            .await
            .expect("apply v19");

        let row: (String, bool, Vec<u8>) = sqlx::query_as(
            "SELECT description, success, checksum FROM _sqlx_migrations WHERE version = 19",
        )
        .fetch_one(&mut conn)
        .await
        .expect("load v19 history");

        assert_eq!(row.0, SOURCE_IDENTITY_CLEANUP_DESCRIPTION);
        assert!(row.1);
        assert_eq!(row.2, expected_migration_19_checksum());
    }

    #[tokio::test]
    async fn migration_19_is_idempotent_when_checksum_matches() {
        let mut conn = memory_conn_with_sqlx_history_through(18).await;

        apply_source_identity_cleanup_on_connection(&mut conn)
            .await
            .expect("first v19");
        apply_source_identity_cleanup_on_connection(&mut conn)
            .await
            .expect("second v19");

        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations WHERE version = 19")
                .fetch_one(&mut conn)
                .await
                .expect("count v19 records");
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn v19_rebuild_removes_legacy_column_and_recreates_expected_indexes() {
        let mut conn = memory_conn_with_sqlx_history_through(18).await;

        apply_source_identity_cleanup_on_connection(&mut conn)
            .await
            .expect("apply v19");

        let legacy_column_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM pragma_table_info('sources') WHERE name = 'telegram_source_kind'",
        )
        .fetch_one(&mut conn)
        .await
        .expect("count legacy column");
        assert_eq!(legacy_column_count, 0);

        assert_sources_index(
            &mut conn,
            "idx_sources_unique_telegram_identity",
            true,
            &["account_id", "source_type", "source_subtype", "external_id"],
            "source_type = 'telegram'",
        )
        .await;
        assert_sources_index(
            &mut conn,
            "idx_sources_unique_youtube_video",
            true,
            &["source_type", "source_subtype", "external_id"],
            "source_type = 'youtube' AND source_subtype = 'video'",
        )
        .await;
        assert_sources_index(
            &mut conn,
            "idx_sources_unique_youtube_playlist",
            true,
            &["source_type", "source_subtype", "external_id"],
            "source_type = 'youtube' AND source_subtype = 'playlist'",
        )
        .await;

        let old_index_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_schema WHERE type = 'index' AND name = 'idx_sources_ext'",
        )
        .fetch_one(&mut conn)
        .await
        .expect("count old index");
        assert_eq!(old_index_count, 0);
    }

    #[tokio::test]
    async fn v19_schema_checks_reject_invalid_implemented_provider_rows() {
        let mut conn = memory_conn_with_sqlx_history_through(18).await;

        apply_source_identity_cleanup_on_connection(&mut conn)
            .await
            .expect("apply v19");

        assert_insert_fails(
            &mut conn,
            "INSERT INTO sources (source_type, source_subtype, account_id, external_id, created_at) VALUES ('telegram', 'channel', NULL, '123', 1)",
        )
        .await;
        assert_insert_fails(
            &mut conn,
            "INSERT INTO sources (source_type, source_subtype, account_id, external_id, created_at) VALUES ('telegram', NULL, 1, '123', 1)",
        )
        .await;
        assert_insert_fails(
            &mut conn,
            "INSERT INTO sources (source_type, source_subtype, account_id, external_id, created_at) VALUES ('telegram', 'video', 1, '123', 1)",
        )
        .await;
        assert_insert_fails(
            &mut conn,
            "INSERT INTO sources (source_type, source_subtype, account_id, external_id, created_at) VALUES ('youtube', NULL, NULL, 'abc', 1)",
        )
        .await;
        assert_insert_fails(
            &mut conn,
            "INSERT INTO sources (source_type, source_subtype, account_id, external_id, created_at) VALUES ('youtube', 'channel', NULL, 'abc', 1)",
        )
        .await;
        assert_insert_fails(
            &mut conn,
            "INSERT INTO sources (source_type, source_subtype, account_id, external_id, created_at) VALUES ('youtube', 'video', 1, 'abc', 1)",
        )
        .await;

        sqlx::query(
            "INSERT INTO sources (source_type, source_subtype, account_id, external_id, created_at) VALUES ('rss', 'feed', NULL, 'feed-1', 1)",
        )
        .execute(&mut conn)
        .await
        .expect("rss placeholder subtype remains allowed");
    }

    async fn apply_standard_migrations_through(
        conn: &mut SqliteConnection,
        version: i64,
    ) -> AppResult<()> {
        ensure_sqlx_migrations_table(conn).await?;
        for migration in build_migrations()
            .into_iter()
            .filter(|migration| migration.version <= version)
        {
            sqlx::raw_sql(migration.sql)
                .execute(&mut *conn)
                .await
                .map_err(AppError::database)?;
            record_migration_success(
                conn,
                migration.version,
                migration.description,
                Sha384::digest(migration.sql.as_bytes()).to_vec(),
                0,
            )
            .await?;
        }
        Ok(())
    }

    async fn assert_insert_fails(conn: &mut SqliteConnection, sql: &str) {
        let error = sqlx::query(sql)
            .execute(&mut *conn)
            .await
            .expect_err("insert should fail");
        let message = error.to_string();
        assert!(
            message.contains("CHECK constraint failed")
                || message.contains("FOREIGN KEY constraint failed")
                || message.contains("UNIQUE constraint failed"),
            "unexpected error: {message}"
        );
    }

    async fn assert_sources_index(
        conn: &mut SqliteConnection,
        name: &str,
        unique: bool,
        columns: &[&str],
        where_clause: &str,
    ) {
        let row: (String, String) = sqlx::query_as(
            "SELECT tbl_name, sql FROM sqlite_schema WHERE type = 'index' AND name = ?",
        )
        .bind(name)
        .fetch_one(&mut *conn)
        .await
        .unwrap_or_else(|_| panic!("missing index {name}"));
        assert_eq!(row.0, "sources");
        assert!(
            row.1.contains("CREATE UNIQUE INDEX") == unique,
            "unexpected uniqueness for {name}: {}",
            row.1
        );
        for column in columns {
            assert!(
                row.1.contains(column),
                "index {name} SQL missing column {column}: {}",
                row.1
            );
        }
        assert!(
            row.1.contains(where_clause),
            "index {name} SQL missing WHERE clause {where_clause}: {}",
            row.1
        );
    }
}
