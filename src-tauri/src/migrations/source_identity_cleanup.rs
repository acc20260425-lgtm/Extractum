use std::time::Instant;

use sha2::{Digest, Sha384};
use sqlx::{Connection, SqliteConnection};

use crate::error::{AppError, AppResult};
use crate::tx::{begin_immediate_on_connection, finish_connection_transaction};

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

pub(super) async fn apply_source_identity_cleanup_on_connection(
    conn: &mut SqliteConnection,
) -> AppResult<()> {
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
    apply_standard_migrations_before_plugin_on_connection(&mut conn, migrations).await
}

pub(super) async fn apply_standard_migrations_before_plugin_on_connection(
    conn: &mut SqliteConnection,
    migrations: Vec<tauri_plugin_sql::Migration>,
) -> AppResult<()> {
    ensure_sqlx_migrations_table(&mut *conn).await?;
    reject_unsupported_pre_v18_telegram_upgrade(&mut *conn).await?;

    for migration in migrations
        .into_iter()
        .filter(|migration| migration.version < SOURCE_IDENTITY_CLEANUP_VERSION)
    {
        let exists = migration_record_exists(&mut *conn, migration.version).await?;
        if exists {
            continue;
        }

        let started_at = Instant::now();
        sqlx::raw_sql(migration.sql)
            .execute(&mut *conn)
            .await
            .map_err(AppError::database)?;
        record_migration_success(
            &mut *conn,
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

pub(super) async fn ensure_sqlx_migrations_table_for_runner(
    conn: &mut SqliteConnection,
) -> AppResult<()> {
    ensure_sqlx_migrations_table(conn).await
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

async fn reject_unsupported_pre_v18_telegram_upgrade(conn: &mut SqliteConnection) -> AppResult<()> {
    let max_version: Option<i64> =
        sqlx::query_scalar("SELECT MAX(version) FROM _sqlx_migrations WHERE success = 1")
            .fetch_optional(&mut *conn)
            .await
            .map_err(AppError::database)?
            .flatten();
    if max_version.unwrap_or(0) >= 18 {
        return Ok(());
    }

    let sources_exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_schema WHERE type = 'table' AND name = 'sources'",
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(AppError::database)?;
    if sources_exists == 0 {
        return Ok(());
    }

    let telegram_rows: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sources WHERE source_type IN ('telegram', 'telegram_channel')",
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(AppError::database)?;
    if telegram_rows == 0 {
        return Ok(());
    }

    Err(AppError::validation(
        "Source identity cleanup cannot upgrade pre-v18 databases with Telegram rows directly. Open the database with a v18 source identity repair build first, or restore a repaired backup before applying migration 19.",
    ))
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

    begin_immediate_on_connection(conn).await?;

    let rebuild_result = async {
        preflight_sources_for_v19(conn).await?;
        rebuild_sources_table(conn).await?;
        restore_sources_sequence(conn, sequence).await?;
        assert_foreign_key_check_clean(conn, "inside v19 transaction").await
    }
    .await;

    if let Err(error) = finish_connection_transaction(conn, rebuild_result).await {
        let _ = sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&mut *conn)
            .await;
        return Err(error);
    }

    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&mut *conn)
        .await
        .map_err(AppError::database)?;
    assert_foreign_key_check_clean(conn, "after v19 commit").await
}

async fn preflight_sources_for_v19(conn: &mut SqliteConnection) -> AppResult<()> {
    let invalid_telegram: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM sources
        WHERE source_type = 'telegram'
          AND (
              account_id IS NULL
              OR source_subtype IS NULL
              OR source_subtype NOT IN ('channel', 'supergroup', 'group')
          )
        "#,
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(AppError::database)?;
    if invalid_telegram != 0 {
        return Err(AppError::validation(
            "Source identity cleanup migration 19 requires repaired Telegram sources with account_id and supported source_subtype",
        ));
    }

    let invalid_youtube: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM sources
        WHERE source_type = 'youtube'
          AND (
              account_id IS NOT NULL
              OR source_subtype IS NULL
              OR source_subtype NOT IN ('video', 'playlist')
          )
        "#,
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(AppError::database)?;
    if invalid_youtube != 0 {
        return Err(AppError::validation(
            "Source identity cleanup migration 19 requires YouTube sources with account_id NULL and subtype video or playlist",
        ));
    }

    let duplicate_typed: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM (
            SELECT account_id, peer_kind, peer_id
            FROM telegram_sources
            GROUP BY account_id, peer_kind, peer_id
            HAVING COUNT(*) > 1
        )
        "#,
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(AppError::database)?;
    if duplicate_typed != 0 {
        return Err(AppError::validation(
            "Source identity cleanup migration 19 found duplicate typed Telegram peer identity",
        ));
    }

    Ok(())
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

    #[tokio::test]
    async fn v19_preserves_source_ids_sequence_and_reference_graph() {
        let mut conn = memory_conn_with_sqlx_history_through(18).await;
        seed_repaired_v18_graph(&mut conn).await;

        sqlx::query("UPDATE sqlite_sequence SET seq = 500 WHERE name = 'sources'")
            .execute(&mut conn)
            .await
            .expect("raise sources sequence");

        apply_source_identity_cleanup_on_connection(&mut conn)
            .await
            .expect("apply v19");

        let source_ids: Vec<i64> = sqlx::query_scalar("SELECT id FROM sources ORDER BY id")
            .fetch_all(&mut conn)
            .await
            .expect("load source ids");
        assert_eq!(source_ids, vec![101, 201, 202]);

        let sequence: i64 =
            sqlx::query_scalar("SELECT seq FROM sqlite_sequence WHERE name = 'sources'")
                .fetch_one(&mut conn)
                .await
                .expect("load sequence");
        assert_eq!(sequence, 500);

        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT source_id FROM items WHERE id = 301")
                .fetch_one(&mut conn)
                .await
                .expect("items source id"),
            101
        );
        assert_eq!(
            sqlx::query_scalar::<_, i64>(
                "SELECT source_id FROM telegram_sources WHERE source_id = 101",
            )
            .fetch_one(&mut conn)
            .await
            .expect("telegram typed source id"),
            101
        );
        assert_eq!(
            sqlx::query_scalar::<_, i64>(
                "SELECT source_id FROM telegram_forum_topics WHERE id = 401",
            )
            .fetch_one(&mut conn)
            .await
            .expect("topic source id"),
            101
        );
        assert_eq!(
            sqlx::query_scalar::<_, i64>(
                "SELECT source_id FROM source_identity_repair_notes WHERE id = 501",
            )
            .fetch_one(&mut conn)
            .await
            .expect("repair note source id"),
            101
        );
        assert_eq!(
            sqlx::query_scalar::<_, i64>(
                "SELECT playlist_source_id FROM youtube_playlist_items WHERE id = 601",
            )
            .fetch_one(&mut conn)
            .await
            .expect("playlist source id"),
            202
        );
        assert_eq!(
            sqlx::query_scalar::<_, i64>(
                "SELECT video_source_id FROM youtube_playlist_items WHERE id = 601",
            )
            .fetch_one(&mut conn)
            .await
            .expect("video source id"),
            201
        );
        assert_eq!(
            sqlx::query_scalar::<_, i64>(
                "SELECT source_id FROM youtube_transcript_segments WHERE id = 701",
            )
            .fetch_one(&mut conn)
            .await
            .expect("transcript source id"),
            201
        );
        assert_eq!(
            sqlx::query_scalar::<_, i64>("SELECT source_id FROM analysis_runs WHERE id = 801")
                .fetch_one(&mut conn)
                .await
                .expect("analysis run logical source id"),
            101
        );
        assert_eq!(
            sqlx::query_scalar::<_, i64>(
                "SELECT source_id FROM analysis_run_messages WHERE run_id = 801 AND ref = '1'",
            )
            .fetch_one(&mut conn)
            .await
            .expect("analysis message logical source id"),
            101
        );

        assert_foreign_key_check_clean(&mut conn, "test after graph migration")
            .await
            .expect("foreign keys clean");
    }

    #[tokio::test]
    async fn v19_rejects_invalid_repaired_v18_inputs_without_partial_schema() {
        for case in [
            InvalidV18Case::TelegramNullAccount,
            InvalidV18Case::TelegramNullSubtype,
            InvalidV18Case::DuplicateCanonicalTelegramIdentity,
            InvalidV18Case::DuplicateTypedTelegramPeer,
            InvalidV18Case::InvalidYoutubeSubtype,
            InvalidV18Case::YoutubeAccountId,
        ] {
            let mut conn = memory_conn_with_sqlx_history_through(18).await;
            seed_invalid_v18_case(&mut conn, case).await;

            let error = apply_source_identity_cleanup_on_connection(&mut conn)
                .await
                .expect_err("invalid v18 input must fail");
            assert!(
                error.message.contains("Source identity cleanup")
                    || error.message.contains("Database error"),
                "unexpected error for {case:?}: {}",
                error.message
            );
            assert_failed_v19_left_old_sources_table(&mut conn).await;
        }
    }

    #[tokio::test]
    async fn pre_v18_database_with_telegram_rows_gets_repair_window_error() {
        let mut conn = memory_conn_with_sqlx_history_through(17).await;
        sqlx::query(
            "INSERT INTO accounts (id, label, api_id, api_hash, created_at) VALUES (1, 'acct', 1, 'hash', 1)",
        )
        .execute(&mut conn)
        .await
        .expect("insert account");
        sqlx::query(
            r#"
            INSERT INTO sources (
                id, source_type, source_subtype, telegram_source_kind, account_id,
                external_id, title, created_at
            )
            VALUES (101, 'telegram', 'channel', 'channel', 1, '12345', 'source', 1)
            "#,
        )
        .execute(&mut conn)
        .await
        .expect("insert pre-v18 telegram source");

        let error = reject_unsupported_pre_v18_telegram_upgrade(&mut conn)
            .await
            .expect_err("pre-v18 telegram upgrade should fail");
        assert!(error.message.contains("v18 source identity repair build"));
        assert!(error.message.contains("repaired backup"));
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

    async fn seed_repaired_v18_graph(conn: &mut SqliteConnection) {
        sqlx::query(
            "INSERT INTO accounts (id, label, api_id, api_hash, created_at) VALUES (1, 'acct', 1, 'hash', 1)",
        )
        .execute(&mut *conn)
        .await
        .expect("insert account");
        sqlx::query(
            r#"
            INSERT INTO sources (
                id, source_type, source_subtype, telegram_source_kind, account_id,
                external_id, title, is_active, is_member, created_at
            )
            VALUES
                (101, 'telegram', 'supergroup', 'supergroup', 1, '12345', 'Forum', 1, 1, 10),
                (201, 'youtube', 'video', '', NULL, 'video-1', 'Video', 1, 0, 11),
                (202, 'youtube', 'playlist', '', NULL, 'playlist-1', 'Playlist', 1, 0, 12)
            "#,
        )
        .execute(&mut *conn)
        .await
        .expect("insert sources");
        sqlx::query(
            r#"
            INSERT INTO telegram_sources (
                source_id, account_id, source_subtype, peer_kind, peer_id,
                resolution_strategy, username
            )
            VALUES (101, 1, 'supergroup', 'channel', 12345, 'username', 'forum')
            "#,
        )
        .execute(&mut *conn)
        .await
        .expect("insert typed telegram source");
        sqlx::query(
            "INSERT INTO items (id, source_id, external_id, author, published_at, ingested_at, content_kind, item_kind) VALUES (301, 101, 'msg-1', 'alice', 1, 1, 'text_only', 'telegram_message')",
        )
        .execute(&mut *conn)
        .await
        .expect("insert item");
        sqlx::query(
            "INSERT INTO analysis_source_groups (id, name, created_at, updated_at, source_type) VALUES (1, 'group', 1, 1, 'telegram')",
        )
        .execute(&mut *conn)
        .await
        .expect("insert analysis source group");
        sqlx::query(
            "INSERT INTO analysis_source_group_members (group_id, source_id, created_at) VALUES (1, 101, 1)",
        )
        .execute(&mut *conn)
        .await
        .expect("insert analysis group member");
        sqlx::query(
            "INSERT INTO telegram_forum_topics (id, source_id, topic_id, top_message_id, title, last_seen_at, updated_at) VALUES (401, 101, 77, 88, 'topic', 1, 1)",
        )
        .execute(&mut *conn)
        .await
        .expect("insert topic");
        sqlx::query(
            "INSERT INTO source_identity_repair_notes (id, source_id, issue_code, detail, created_at) VALUES (501, 101, 'note', 'detail', 1)",
        )
        .execute(&mut *conn)
        .await
        .expect("insert repair note");
        sqlx::query(
            "INSERT INTO youtube_playlist_items (id, playlist_source_id, video_source_id, video_id, availability_status) VALUES (601, 202, 201, 'video-1', 'available')",
        )
        .execute(&mut *conn)
        .await
        .expect("insert playlist item");
        sqlx::query(
            "INSERT INTO youtube_transcript_segments (id, item_id, source_id, segment_index, start_ms, text) VALUES (701, 301, 201, 0, 0, 'caption')",
        )
        .execute(&mut *conn)
        .await
        .expect("insert transcript segment");
        sqlx::query(
            "INSERT INTO analysis_runs (id, run_type, scope_type, source_id, period_from, period_to, output_language, prompt_template_version, provider_profile, provider, model, status, created_at) VALUES (801, 'single_source', 'source', 101, 1, 2, 'en', 1, 'default', 'openai', 'gpt', 'completed', 1)",
        )
        .execute(&mut *conn)
        .await
        .expect("insert analysis run");
        sqlx::query(
            "INSERT INTO analysis_run_messages (run_id, item_id, source_id, external_id, author, published_at, ref, content_zstd, item_kind, source_type, source_subtype) VALUES (801, 301, 101, 'msg-1', 'alice', 1, '1', x'00', 'telegram_message', 'telegram', 'supergroup')",
        )
        .execute(&mut *conn)
        .await
        .expect("insert analysis run message");
    }

    #[derive(Clone, Copy, Debug)]
    enum InvalidV18Case {
        TelegramNullAccount,
        TelegramNullSubtype,
        DuplicateCanonicalTelegramIdentity,
        DuplicateTypedTelegramPeer,
        InvalidYoutubeSubtype,
        YoutubeAccountId,
    }

    async fn assert_failed_v19_left_old_sources_table(conn: &mut SqliteConnection) {
        let legacy_column_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM pragma_table_info('sources') WHERE name = 'telegram_source_kind'",
        )
        .fetch_one(&mut *conn)
        .await
        .expect("count legacy column after failed v19");
        assert_eq!(legacy_column_count, 1);

        let partial_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_schema WHERE type = 'table' AND name = 'sources_new'",
        )
        .fetch_one(&mut *conn)
        .await
        .expect("count sources_new");
        assert_eq!(partial_count, 0);
    }

    async fn seed_invalid_v18_case(conn: &mut SqliteConnection, case: InvalidV18Case) {
        sqlx::query(
            "INSERT INTO accounts (id, label, api_id, api_hash, created_at) VALUES (1, 'acct', 1, 'hash', 1)",
        )
        .execute(&mut *conn)
        .await
        .expect("insert account");

        match case {
            InvalidV18Case::TelegramNullAccount => {
                sqlx::query(
                    r#"
                    INSERT INTO sources (
                        id, source_type, source_subtype, telegram_source_kind,
                        account_id, external_id, title, created_at
                    )
                    VALUES (101, 'telegram', 'channel', 'channel', NULL, '12345', 'source', 1)
                    "#,
                )
                .execute(&mut *conn)
                .await
                .expect("insert null-account telegram source");
            }
            InvalidV18Case::TelegramNullSubtype => {
                sqlx::query(
                    r#"
                    INSERT INTO sources (
                        id, source_type, source_subtype, telegram_source_kind,
                        account_id, external_id, title, created_at
                    )
                    VALUES (101, 'telegram', NULL, 'channel', 1, '12345', 'source', 1)
                    "#,
                )
                .execute(&mut *conn)
                .await
                .expect("insert null-subtype telegram source");
            }
            InvalidV18Case::DuplicateCanonicalTelegramIdentity => {
                sqlx::query("DROP INDEX IF EXISTS idx_sources_ext")
                    .execute(&mut *conn)
                    .await
                    .expect("drop legacy source identity unique index for fixture");
                sqlx::query(
                    r#"
                    INSERT INTO sources (
                        id, source_type, source_subtype, telegram_source_kind,
                        account_id, external_id, title, created_at
                    )
                    VALUES
                        (101, 'telegram', 'channel', 'channel', 1, '12345', 'one', 1),
                        (102, 'telegram', 'channel', 'channel', 1, '12345', 'two', 2)
                    "#,
                )
                .execute(&mut *conn)
                .await
                .expect("insert duplicate canonical sources");
                insert_typed_telegram_projection(conn, 101, "channel", "channel", 12345).await;
                insert_typed_telegram_projection(conn, 102, "channel", "channel", 67890).await;
            }
            InvalidV18Case::DuplicateTypedTelegramPeer => {
                sqlx::query("DROP INDEX IF EXISTS idx_telegram_sources_account_peer")
                    .execute(&mut *conn)
                    .await
                    .expect("drop typed peer unique index for fixture");
                sqlx::query(
                    r#"
                    INSERT INTO sources (
                        id, source_type, source_subtype, telegram_source_kind,
                        account_id, external_id, title, created_at
                    )
                    VALUES
                        (101, 'telegram', 'channel', 'channel', 1, '12345', 'one', 1),
                        (102, 'telegram', 'supergroup', 'supergroup', 1, '67890', 'two', 2)
                    "#,
                )
                .execute(&mut *conn)
                .await
                .expect("insert typed duplicate sources");
                insert_typed_telegram_projection(conn, 101, "channel", "channel", 12345).await;
                insert_typed_telegram_projection(conn, 102, "supergroup", "channel", 12345).await;
            }
            InvalidV18Case::InvalidYoutubeSubtype => {
                sqlx::query(
                    r#"
                    INSERT INTO sources (
                        id, source_type, source_subtype, telegram_source_kind,
                        account_id, external_id, title, created_at
                    )
                    VALUES (201, 'youtube', 'channel', '', NULL, 'video-1', 'video', 1)
                    "#,
                )
                .execute(&mut *conn)
                .await
                .expect("insert invalid youtube subtype");
            }
            InvalidV18Case::YoutubeAccountId => {
                sqlx::query(
                    r#"
                    INSERT INTO sources (
                        id, source_type, source_subtype, telegram_source_kind,
                        account_id, external_id, title, created_at
                    )
                    VALUES (201, 'youtube', 'video', '', 1, 'video-1', 'video', 1)
                    "#,
                )
                .execute(&mut *conn)
                .await
                .expect("insert youtube account id");
            }
        }
    }

    async fn insert_typed_telegram_projection(
        conn: &mut SqliteConnection,
        source_id: i64,
        source_subtype: &str,
        peer_kind: &str,
        peer_id: i64,
    ) {
        sqlx::query(
            r#"
            INSERT INTO telegram_sources (
                source_id, account_id, source_subtype, peer_kind, peer_id,
                resolution_strategy
            )
            VALUES (?, 1, ?, ?, ?, 'unknown')
            "#,
        )
        .bind(source_id)
        .bind(source_subtype)
        .bind(peer_kind)
        .bind(peer_id)
        .execute(&mut *conn)
        .await
        .expect("insert typed telegram projection");
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
