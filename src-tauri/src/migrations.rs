#![allow(clippy::items_after_test_module)]

pub(crate) mod analysis_documents;
mod baseline_reset;
pub(crate) mod source_identity_cleanup;
pub(crate) mod telegram_item_native_identity;
pub(crate) mod topic_membership_materialization;
pub(crate) mod youtube_typed_source_metadata;

use sha2::{Digest, Sha384};
use std::path::{Path, PathBuf};
use tauri_plugin_sql::{Migration, MigrationKind};

const APP_IDENTIFIER: &str = "org.ai.extractum";
const DB_FILENAME: &str = "extractum.db";
const BASELINE_VERSION: i64 = 1;
const BASELINE_DESCRIPTION: &str = "current schema baseline";
const BASELINE_SQL: &str = include_str!("../migrations/0001_current_schema_baseline.sql");

/// Before the sql plugin runs, remove stale migration records whose SQL has changed.
/// This allows us to update migration files without deleting the database.
#[allow(dead_code)]
async fn patch_migrations(db_path: &Path) -> crate::error::AppResult<()> {
    use sqlx::SqlitePool;

    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| crate::error::AppError::internal(error.to_string()))?;
    }

    let url = format!("sqlite:{}", db_path.to_string_lossy());
    source_identity_cleanup::apply_standard_migrations_before_plugin(&url, build_migrations())
        .await?;

    let pool = SqlitePool::connect(&url)
        .await
        .map_err(crate::error::AppError::database)?;
    repair_line_ending_migration_checksums(&pool).await;
    repair_legacy_v2_migration_checksum(&pool).await;
    pool.close().await;

    source_identity_cleanup::apply_source_identity_cleanup_if_needed(&url).await?;
    youtube_typed_source_metadata::apply_youtube_typed_source_metadata_if_needed(&url).await?;
    telegram_item_native_identity::apply_telegram_item_native_identity_if_needed(&url).await?;
    topic_membership_materialization::apply_topic_membership_materialization_if_needed(&url)
        .await?;
    apply_regular_sql_migrations_before_runner(&url, 22, 24).await?;
    analysis_documents::apply_analysis_documents_if_needed(&url).await
}

#[allow(dead_code)]
async fn apply_regular_sql_migrations_before_runner(
    db_url: &str,
    after_version: i64,
    before_version: i64,
) -> crate::error::AppResult<()> {
    use sqlx::Connection;

    let mut conn = sqlx::SqliteConnection::connect(db_url)
        .await
        .map_err(crate::error::AppError::database)?;
    apply_regular_sql_migrations_before_runner_on_connection(
        &mut conn,
        after_version,
        before_version,
    )
    .await
}

#[allow(dead_code)]
async fn apply_regular_sql_migrations_before_runner_on_connection(
    conn: &mut sqlx::SqliteConnection,
    after_version: i64,
    before_version: i64,
) -> crate::error::AppResult<()> {
    source_identity_cleanup::ensure_sqlx_migrations_table_for_runner(conn).await?;

    for migration in build_migrations()
        .into_iter()
        .filter(|migration| migration.version > after_version && migration.version < before_version)
    {
        let exists: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM _sqlx_migrations WHERE version = ? AND success = 1",
        )
        .bind(migration.version)
        .fetch_one(&mut *conn)
        .await
        .map_err(crate::error::AppError::database)?;
        if exists != 0 {
            continue;
        }

        let started_at = std::time::Instant::now();
        sqlx::raw_sql(migration.sql)
            .execute(&mut *conn)
            .await
            .map_err(crate::error::AppError::database)?;
        sqlx::query(
            "INSERT INTO _sqlx_migrations (
                version, description, success, checksum, execution_time
             ) VALUES (?, ?, 1, ?, ?)",
        )
        .bind(migration.version)
        .bind(migration.description)
        .bind(Sha384::digest(migration.sql.as_bytes()).to_vec())
        .bind(started_at.elapsed().as_nanos().min(i64::MAX as u128) as i64)
        .execute(&mut *conn)
        .await
        .map_err(crate::error::AppError::database)?;
    }

    Ok(())
}

fn app_config_db_path() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join(APP_IDENTIFIER).join(DB_FILENAME))
}

pub fn prepare_database() -> crate::error::AppResult<()> {
    let Some(db_path) = app_config_db_path() else {
        return Ok(());
    };
    prepare_database_at_path(&db_path)
}

fn prepare_database_at_path(db_path: &Path) -> crate::error::AppResult<()> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| crate::error::AppError::internal(error.to_string()))?;
    }

    if !db_path.exists() {
        return Ok(());
    }

    tauri::async_runtime::block_on(baseline_reset::apply_baseline_reset_if_needed(
        db_path,
        BASELINE_SQL,
        &baseline_reset::FileSystemBaselineResetBackup,
    ))
}

fn current_schema_baseline_migration() -> Migration {
    Migration {
        version: BASELINE_VERSION,
        description: BASELINE_DESCRIPTION,
        sql: BASELINE_SQL,
        kind: MigrationKind::Up,
    }
}

pub fn build_migrations() -> Vec<Migration> {
    vec![current_schema_baseline_migration()]
}

#[cfg(test)]
pub(crate) async fn apply_all_migrations_for_test_pool(
    pool: &sqlx::SqlitePool,
) -> crate::error::AppResult<()> {
    sqlx::raw_sql(BASELINE_SQL)
        .execute(pool)
        .await
        .map_err(crate::error::AppError::database)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        apply_all_migrations_for_test_pool, build_migrations, checksum_matches_line_ending_variant,
        current_schema_baseline_migration, prepare_database_at_path,
    };
    use sha2::{Digest, Sha384};

    #[test]
    fn current_schema_baseline_migration_is_version_one() {
        let migration = current_schema_baseline_migration();

        assert_eq!(migration.version, 1);
        assert_eq!(migration.description, "current schema baseline");
        assert!(migration.sql.contains("CREATE TABLE accounts"));
        assert!(migration.sql.contains("CREATE TABLE archive_read_items"));
    }

    #[tokio::test]
    async fn fresh_schema_includes_source_identity_tables_after_sql_managed_migrations() {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");

        for migration in build_migrations()
            .into_iter()
            .filter(|migration| migration.version < 19)
        {
            sqlx::raw_sql(migration.sql)
                .execute(&pool)
                .await
                .unwrap_or_else(|error| panic!("apply migration {}: {error}", migration.version));
        }

        for table in [
            "sources",
            "telegram_sources",
            "source_identity_repair_notes",
        ] {
            let exists: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?",
            )
            .bind(table)
            .fetch_one(&pool)
            .await
            .expect("check table");
            assert_eq!(exists, 1, "missing table {table}");
        }
    }

    #[test]
    fn includes_telegram_item_context_migration() {
        let migrations = build_migrations();
        let migration = migrations
            .iter()
            .find(|migration| migration.version == 13)
            .expect("version 13 migration is registered");

        for column in [
            "reply_to_msg_id",
            "reply_to_peer_kind",
            "reply_to_peer_id",
            "reply_to_top_id",
            "reaction_count",
        ] {
            assert!(migration.sql.contains(column), "missing column {column}");
        }
    }

    #[test]
    fn includes_telegram_forum_topics_migration() {
        let migrations = build_migrations();
        let migration = migrations
            .iter()
            .find(|migration| migration.version == 14)
            .expect("version 14 migration is registered");

        for fragment in [
            "CREATE TABLE IF NOT EXISTS telegram_forum_topics",
            "topic_id INTEGER NOT NULL",
            "top_message_id INTEGER NOT NULL",
            "FOREIGN KEY(source_id) REFERENCES sources(id) ON DELETE CASCADE",
            "idx_telegram_forum_topics_source_topic",
            "idx_telegram_forum_topics_source_top_message",
            "idx_items_source_reply_to_top",
        ] {
            assert!(
                migration.sql.contains(fragment),
                "missing migration fragment {fragment}"
            );
        }
    }

    #[test]
    fn includes_provider_source_subtype_migration() {
        let migrations = build_migrations();
        let migration = migrations
            .iter()
            .find(|migration| migration.version == 15)
            .expect("version 15 migration is registered");

        for fragment in [
            "ALTER TABLE sources ADD COLUMN source_subtype TEXT",
            "SET source_subtype = telegram_source_kind",
            "WHERE source_type = 'telegram'",
        ] {
            assert!(
                migration.sql.contains(fragment),
                "missing migration fragment {fragment}"
            );
        }
    }

    #[test]
    fn includes_youtube_source_foundation_migration() {
        let migrations = build_migrations();
        let migration = migrations
            .iter()
            .find(|migration| migration.version == 16)
            .expect("version 16 migration is registered");

        for fragment in [
            "ALTER TABLE items ADD COLUMN item_kind TEXT NOT NULL DEFAULT 'telegram_message'",
            "CREATE TABLE IF NOT EXISTS youtube_playlist_items",
            "CHECK (availability_status IN",
            "CREATE TABLE IF NOT EXISTS youtube_transcript_segments",
            "ALTER TABLE analysis_run_messages ADD COLUMN item_kind TEXT",
            "ALTER TABLE analysis_run_messages ADD COLUMN source_type TEXT",
            "ALTER TABLE analysis_run_messages ADD COLUMN source_subtype TEXT",
            "ALTER TABLE analysis_run_messages ADD COLUMN metadata_zstd BLOB",
            "ALTER TABLE analysis_source_groups ADD COLUMN source_type TEXT NOT NULL DEFAULT 'telegram'",
            "idx_sources_unique_youtube_video",
            "idx_sources_unique_youtube_playlist",
        ] {
            assert!(
                migration.sql.contains(fragment),
                "missing migration fragment {fragment}"
            );
        }
    }

    #[test]
    fn includes_analysis_run_youtube_corpus_mode_migration() {
        let migrations = build_migrations();
        let migration = migrations
            .iter()
            .find(|migration| migration.version == 17)
            .expect("version 17 migration is registered");

        for fragment in [
            "ALTER TABLE analysis_runs ADD COLUMN youtube_corpus_mode TEXT NOT NULL DEFAULT 'transcript_description'",
            "CHECK (youtube_corpus_mode IN",
            "'transcript_only'",
            "'transcript_description'",
            "'transcript_description_comments'",
        ] {
            assert!(
                migration.sql.contains(fragment),
                "missing migration fragment {fragment}"
            );
        }
    }

    #[test]
    fn includes_source_identity_schema_bridge_migration() {
        let migrations = build_migrations();
        let migration = migrations
            .iter()
            .find(|migration| migration.version == 18)
            .expect("version 18 migration is registered");

        for fragment in [
            "CREATE TABLE IF NOT EXISTS telegram_sources",
            "source_identity_repair_notes",
            "idx_telegram_sources_account_peer",
            "idx_telegram_sources_account_subtype",
            "idx_telegram_sources_account_username",
            "SET source_subtype = telegram_source_kind",
        ] {
            assert!(
                migration.sql.contains(fragment),
                "missing migration fragment {fragment}"
            );
        }
    }

    #[test]
    fn includes_runner_managed_source_identity_cleanup_migration() {
        let migrations = build_migrations();
        let migration = migrations
            .iter()
            .find(|migration| migration.version == 19)
            .expect("version 19 migration is registered");

        assert_eq!(migration.description, "remove legacy telegram source kind");
        assert!(
            migration
                .sql
                .contains("extractum_runner_managed_migration_19"),
            "v19 must fail if plugin-managed SQLx applies it directly"
        );
    }

    #[test]
    fn plugin_migration_list_keeps_v19_as_sentinel_only() {
        let migration = build_migrations()
            .into_iter()
            .find(|migration| migration.version == 19)
            .expect("version 19 migration is registered");

        assert!(!migration.sql.contains("DROP TABLE sources"));
        assert!(!migration.sql.contains("ALTER TABLE sources"));
        assert!(!migration.sql.contains("CREATE TABLE sources_new"));
    }

    #[test]
    fn includes_runner_managed_youtube_typed_source_metadata_migration() {
        let migrations = build_migrations();
        let migration = migrations
            .iter()
            .find(|migration| migration.version == 20)
            .expect("version 20 migration is registered");

        assert_eq!(migration.description, "add youtube typed source metadata");
        assert!(
            migration
                .sql
                .contains("extractum_runner_managed_migration_20"),
            "v20 must fail if plugin-managed SQL applies it directly"
        );
    }

    #[test]
    fn plugin_migration_list_keeps_v20_as_sentinel_only() {
        let migration = build_migrations()
            .into_iter()
            .find(|migration| migration.version == 20)
            .expect("version 20 migration is registered");

        assert!(!migration.sql.contains("CREATE TABLE youtube_video_sources"));
        assert!(!migration
            .sql
            .contains("CREATE TABLE youtube_playlist_sources"));
        assert!(!migration.sql.contains("INSERT INTO youtube_video_sources"));
    }

    #[test]
    fn includes_runner_managed_telegram_item_native_identity_migration() {
        let migrations = build_migrations();
        let migration = migrations
            .iter()
            .find(|migration| migration.version == 21)
            .expect("version 21 migration is registered");

        assert_eq!(migration.description, "add telegram item native identity");
        assert!(
            migration
                .sql
                .contains("extractum_runner_managed_migration_21"),
            "v21 must fail if plugin-managed SQL applies it directly"
        );
    }

    #[test]
    fn plugin_migration_list_keeps_v21_as_sentinel_only() {
        let migration = build_migrations()
            .into_iter()
            .find(|migration| migration.version == 21)
            .expect("version 21 migration is registered");

        assert!(!migration.sql.contains("CREATE TABLE telegram_messages"));
        assert!(!migration.sql.contains("DROP INDEX idx_items_ext"));
        assert!(!migration.sql.contains("INSERT INTO telegram_messages"));
    }

    #[test]
    fn includes_runner_managed_topic_membership_materialization_migration() {
        let migrations = build_migrations();
        let migration = migrations
            .iter()
            .find(|migration| migration.version == 22)
            .expect("version 22 migration is registered");

        assert_eq!(
            migration.description,
            "materialize telegram topic memberships"
        );
        assert!(
            migration
                .sql
                .contains("extractum_runner_managed_migration_22"),
            "v22 must fail if plugin-managed SQL applies it directly"
        );
    }

    #[test]
    fn plugin_migration_list_keeps_v22_as_sentinel_only() {
        let migration = build_migrations()
            .into_iter()
            .find(|migration| migration.version == 22)
            .expect("version 22 migration is registered");

        assert!(!migration
            .sql
            .contains("CREATE TABLE item_topic_memberships"));
        assert!(!migration
            .sql
            .contains("CREATE TABLE telegram_topic_resolution_state"));
        assert!(!migration.sql.contains("INSERT INTO item_topic_memberships"));
    }

    #[test]
    fn includes_regular_ingest_provenance_migration() {
        let migrations = build_migrations();
        let migration = migrations
            .iter()
            .find(|migration| migration.version == 23)
            .expect("version 23 migration is registered");

        assert_eq!(migration.description, "add ingest provenance foundation");
        assert!(migration.sql.contains("CREATE TABLE ingest_batches"));
        assert!(migration
            .sql
            .contains("CREATE TABLE telegram_takeout_batches"));
        assert!(migration
            .sql
            .contains("CREATE TABLE ingest_item_observations"));
        assert!(migration.sql.contains("CREATE TABLE ingest_batch_warnings"));
        assert!(!migration.sql.contains("runner_managed"));
    }

    #[test]
    fn includes_runner_managed_analysis_documents_migration() {
        let migrations = build_migrations();
        let migration = migrations
            .iter()
            .find(|migration| migration.version == 24)
            .expect("version 24 migration is registered");

        assert_eq!(
            migration.description,
            "add provider neutral analysis documents"
        );
        assert!(migration.sql.contains("runner-managed"));
        assert!(migration.sql.contains("analysis_documents"));
        assert!(!migration.sql.contains("CREATE TABLE analysis_documents"));
    }

    #[test]
    fn includes_analysis_snapshot_hardening_migration() {
        let migrations = build_migrations();
        let migration = migrations
            .iter()
            .find(|migration| migration.version == 25)
            .expect("version 25 migration is registered");

        for fragment in [
            "ALTER TABLE analysis_runs ADD COLUMN snapshot_captured_at TEXT",
            "ALTER TABLE analysis_runs ADD COLUMN snapshot_error TEXT",
        ] {
            assert!(
                migration.sql.contains(fragment),
                "missing migration fragment {fragment}"
            );
        }
    }

    #[test]
    fn includes_archive_read_model_migration() {
        let migrations = build_migrations();
        let migration = migrations
            .iter()
            .find(|migration| migration.version == 26)
            .expect("version 26 migration is registered");

        assert_eq!(
            migration.description,
            "add provider neutral archive read model"
        );
        for fragment in [
            "CREATE TABLE IF NOT EXISTS archive_read_model_state",
            "CREATE TABLE IF NOT EXISTS archive_read_items",
            "CHECK (status IN ('never_built', 'building', 'ready', 'stale', 'failed'))",
            "idx_archive_read_items_source_published",
            "idx_archive_read_items_source_topic_published",
            "idx_archive_read_items_ref",
        ] {
            assert!(
                migration.sql.contains(fragment),
                "missing migration fragment {fragment}"
            );
        }
    }

    #[test]
    fn build_migrations_starts_at_current_schema_baseline() {
        let migrations = build_migrations();
        let versions = migrations
            .iter()
            .map(|migration| migration.version)
            .collect::<Vec<_>>();

        assert_eq!(versions, vec![1]);
        assert_eq!(migrations[0].description, "current schema baseline");
        assert!(migrations[0]
            .sql
            .contains("CREATE TABLE archive_read_items"));
    }

    #[test]
    fn prepare_database_skips_cutover_when_database_file_is_missing() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db_path = temp_dir.path().join("extractum.db");

        prepare_database_at_path(&db_path).expect("prepare missing database path");

        assert!(
            !db_path.exists(),
            "prepare_database must not create a DB before the SQL plugin"
        );
    }

    #[tokio::test]
    async fn fresh_schema_includes_analysis_snapshot_markers() {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");

        apply_all_migrations_for_test_pool(&pool)
            .await
            .expect("apply migrations");

        for column in ["snapshot_captured_at", "snapshot_error"] {
            let exists: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM pragma_table_info('analysis_runs') WHERE name = ?",
            )
            .bind(column)
            .fetch_one(&pool)
            .await
            .expect("check analysis_runs column");
            assert_eq!(exists, 1, "missing analysis_runs.{column}");
        }
    }

    #[tokio::test]
    async fn fresh_schema_includes_archive_read_model_tables_indexes_and_constraints() {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");

        apply_all_migrations_for_test_pool(&pool)
            .await
            .expect("apply migrations");

        for table in ["archive_read_model_state", "archive_read_items"] {
            let exists: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?",
            )
            .bind(table)
            .fetch_one(&pool)
            .await
            .expect("check table");
            assert_eq!(exists, 1, "missing table {table}");
        }

        for index in [
            "idx_archive_read_items_source_published",
            "idx_archive_read_items_source_topic_published",
            "idx_archive_read_items_ref",
        ] {
            let exists: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = ?",
            )
            .bind(index)
            .fetch_one(&pool)
            .await
            .expect("check index");
            assert_eq!(exists, 1, "missing index {index}");
        }
    }

    #[tokio::test]
    async fn fresh_schema_includes_ingest_provenance_tables_indexes_and_constraints() {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");
        apply_all_migrations_for_test_pool(&pool)
            .await
            .expect("apply migrations");

        for table in [
            "ingest_batches",
            "telegram_takeout_batches",
            "ingest_item_observations",
            "ingest_batch_warnings",
        ] {
            let exists: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?",
            )
            .bind(table)
            .fetch_one(&pool)
            .await
            .expect("check table");
            assert_eq!(exists, 1, "missing table {table}");
        }

        for index in [
            "idx_ingest_batches_source_started",
            "idx_ingest_batches_status",
            "idx_telegram_takeout_batches_account",
            "idx_ingest_item_observations_batch",
            "idx_ingest_item_observations_item",
            "idx_ingest_item_observations_identity",
            "idx_ingest_item_observations_batch_outcome",
            "idx_ingest_batch_warnings_batch",
        ] {
            let exists: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = ?",
            )
            .bind(index)
            .fetch_one(&pool)
            .await
            .expect("check index");
            assert_eq!(exists, 1, "missing index {index}");
        }

        sqlx::query(
            "INSERT INTO accounts (id, label, api_id, api_hash, phone, created_at)
             VALUES (10, 'Test', 1, 'hash', '+10000000000', 1)",
        )
        .execute(&pool)
        .await
        .expect("seed account");

        sqlx::query(
            "INSERT INTO sources (
                id, source_type, source_subtype, account_id, external_id, title,
                metadata_zstd, last_sync_state, last_synced_at, is_active, is_member, created_at
             ) VALUES (1, 'telegram', 'supergroup', 10, '12345', 'Forum',
                NULL, NULL, NULL, 1, 1, 1)",
        )
        .execute(&pool)
        .await
        .expect("seed source");

        let batch_id: i64 = sqlx::query_scalar(
            "INSERT INTO ingest_batches (source_id, provider, ingest_kind, status)
             VALUES (1, 'telegram', 'takeout', 'running')
             RETURNING id",
        )
        .fetch_one(&pool)
        .await
        .expect("insert running batch");

        sqlx::query(
            "INSERT INTO telegram_takeout_batches (batch_id, account_id, source_subtype)
             VALUES (?, 10, 'supergroup')",
        )
        .bind(batch_id)
        .execute(&pool)
        .await
        .expect("insert takeout detail");

        let terminal_without_finished_at =
            sqlx::query("UPDATE ingest_batches SET status = 'completed' WHERE id = ?")
                .bind(batch_id)
                .execute(&pool)
                .await;
        assert!(terminal_without_finished_at.is_err());

        sqlx::query(
            "INSERT INTO ingest_item_observations (
                batch_id, source_id, provider_item_kind, provider_identity_kind,
                provider_identity, outcome
             ) VALUES (?, 1, 'telegram_message', 'telegram_message',
                'telegram:history_peer:channel:12345:message:42', 'duplicate_observed')",
        )
        .bind(batch_id)
        .execute(&pool)
        .await
        .expect("insert first observation");

        sqlx::query(
            "INSERT INTO ingest_item_observations (
                batch_id, source_id, provider_item_kind, provider_identity_kind,
                provider_identity, outcome
             ) VALUES (?, 1, 'telegram_message', 'telegram_message',
                'telegram:history_peer:channel:12345:message:42', 'duplicate_observed')",
        )
        .bind(batch_id)
        .execute(&pool)
        .await
        .expect("duplicate observation rows are allowed");

        sqlx::query(
            "INSERT INTO ingest_batch_warnings (batch_id, code, message)
             VALUES (?, 'export_dc_fallback', 'first')",
        )
        .bind(batch_id)
        .execute(&pool)
        .await
        .expect("insert first warning");
        sqlx::query(
            "INSERT INTO ingest_batch_warnings (batch_id, code, message)
             VALUES (?, 'export_dc_fallback', 'second')",
        )
        .bind(batch_id)
        .execute(&pool)
        .await
        .expect("duplicate warning codes are allowed");
    }

    #[tokio::test]
    async fn fresh_schema_includes_analysis_documents_table_indexes_and_constraints() {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");
        apply_all_migrations_for_test_pool(&pool)
            .await
            .expect("apply migrations");

        let table_exists: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'analysis_documents'",
        )
        .fetch_one(&pool)
        .await
        .expect("check table");
        assert_eq!(table_exists, 1);

        for index in [
            "idx_analysis_documents_source_key",
            "idx_analysis_documents_source_published",
            "idx_analysis_documents_kind_source_published",
            "idx_analysis_documents_ref",
        ] {
            let exists: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = ?",
            )
            .bind(index)
            .fetch_one(&pool)
            .await
            .expect("check index");
            assert_eq!(exists, 1, "missing index {index}");
        }
    }

    #[test]
    fn source_identity_schema_bridge_does_not_sql_backfill_typed_identity() {
        let migrations = build_migrations();
        let migration = migrations
            .iter()
            .find(|migration| migration.version == 18)
            .expect("version 18 migration is registered");

        let forbidden_fragments = [
            "INSERT INTO telegram_sources",
            "INSERT OR IGNORE INTO telegram_sources",
            "CAST(external_id",
            "GLOB",
            "idx_sources_unique_telegram_identity",
        ];

        for fragment in forbidden_fragments {
            assert!(
                !migration.sql.contains(fragment),
                "migration 18 must not contain {fragment}"
            );
        }
    }

    #[test]
    fn checksum_match_accepts_line_ending_only_differences() {
        let lf_sql = "ALTER TABLE sources ADD COLUMN source_subtype TEXT;\n\n";
        let crlf_sql = lf_sql.replace('\n', "\r\n");
        let applied_checksum = Sha384::digest(lf_sql.as_bytes()).to_vec();

        assert!(checksum_matches_line_ending_variant(
            &applied_checksum,
            crlf_sql.as_str()
        ));
    }
}

fn sha384_bytes(value: &str) -> Vec<u8> {
    Sha384::digest(value.as_bytes()).to_vec()
}

fn normalize_sql_lf(sql: &str) -> String {
    sql.replace("\r\n", "\n")
}

fn normalize_sql_crlf(sql: &str) -> String {
    normalize_sql_lf(sql).replace('\n', "\r\n")
}

fn checksum_matches_line_ending_variant(applied_checksum: &[u8], sql: &str) -> bool {
    let current_checksum = sha384_bytes(sql);
    if applied_checksum == current_checksum {
        return true;
    }

    applied_checksum == sha384_bytes(&normalize_sql_lf(sql))
        || applied_checksum == sha384_bytes(&normalize_sql_crlf(sql))
}

async fn repair_line_ending_migration_checksums(pool: &sqlx::SqlitePool) {
    let migrations = build_migrations();

    for migration in migrations {
        let current_checksum = sha384_bytes(migration.sql);
        let applied_checksum = sqlx::query_scalar::<_, Vec<u8>>(
            "SELECT checksum FROM _sqlx_migrations WHERE version = ?",
        )
        .bind(migration.version)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten();

        let Some(applied_checksum) = applied_checksum else {
            continue;
        };

        if applied_checksum == current_checksum
            || !checksum_matches_line_ending_variant(&applied_checksum, migration.sql)
        {
            continue;
        }

        let _ = sqlx::query(
            "UPDATE _sqlx_migrations
             SET description = ?, success = 1, checksum = ?
             WHERE version = ?",
        )
        .bind(migration.description)
        .bind(&current_checksum)
        .bind(migration.version)
        .execute(pool)
        .await;
    }
}

async fn repair_legacy_v2_migration_checksum(pool: &sqlx::SqlitePool) {
    let expected_checksum = Sha384::digest(include_str!("../migrations/2.sql").as_bytes()).to_vec();
    let has_v3 = sqlx::query_scalar::<_, i64>(
        "SELECT EXISTS(SELECT 1 FROM _sqlx_migrations WHERE version = 3)",
    )
    .fetch_one(pool)
    .await
    .map(|exists| exists != 0)
    .unwrap_or(false);

    let v2_checksum =
        sqlx::query_scalar::<_, Vec<u8>>("SELECT checksum FROM _sqlx_migrations WHERE version = 2")
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();

    match v2_checksum {
        Some(checksum) if checksum != expected_checksum => {
            if has_v3 {
                let _ = sqlx::query(
                    "UPDATE _sqlx_migrations
                     SET description = ?, success = 1, checksum = ?
                     WHERE version = 2",
                )
                .bind("add is_member to sources")
                .bind(&expected_checksum)
                .execute(pool)
                .await;
            } else {
                let _ = sqlx::query("DELETE FROM _sqlx_migrations WHERE version = 2")
                    .execute(pool)
                    .await;
            }
        }
        None if has_v3 => {
            let _ = sqlx::query(
                "INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time)
                 VALUES (?, ?, 1, ?, 0)",
            )
            .bind(2_i64)
            .bind("add is_member to sources")
            .bind(&expected_checksum)
            .execute(pool)
            .await;
        }
        _ => {}
    }
}
