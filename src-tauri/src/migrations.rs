mod baseline_reset;

use std::path::{Path, PathBuf};
use tauri_plugin_sql::{Migration, MigrationKind};

const APP_IDENTIFIER: &str = "org.ai.extractum";
const DB_FILENAME: &str = "extractum.db";
const BASELINE_VERSION: i64 = 1;
const BASELINE_DESCRIPTION: &str = "current schema baseline";
const BASELINE_SQL: &str = include_str!("../migrations/0001_current_schema_baseline.sql");

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
        apply_all_migrations_for_test_pool, build_migrations, current_schema_baseline_migration,
        prepare_database_at_path,
    };

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
}
