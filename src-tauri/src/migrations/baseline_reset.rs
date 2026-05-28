use crate::error::{AppError, AppResult};
use sha2::{Digest, Sha384};

const OLD_FIRST_VERSION: i64 = 1;
const OLD_LAST_VERSION: i64 = 26;
const BASELINE_VERSION: i64 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MigrationHistoryState {
    BaselineReady,
    OldHistoryReadyForCutover,
}

pub(super) async fn classify_migration_history(
    pool: &sqlx::SqlitePool,
    baseline_sql: &str,
) -> AppResult<MigrationHistoryState> {
    let table_exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = '_sqlx_migrations'",
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::database)?;

    if table_exists == 0 {
        return Err(AppError::internal(
            "unsupported migration history: _sqlx_migrations is missing",
        ));
    }

    let failed_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations WHERE success = 0")
            .fetch_one(pool)
            .await
            .map_err(AppError::database)?;
    if failed_count != 0 {
        return Err(AppError::internal(
            "unsupported migration history: failed migration rows are present",
        ));
    }

    let expected_baseline_checksum = Sha384::digest(baseline_sql.as_bytes()).to_vec();
    let baseline_checksum: Option<Vec<u8>> =
        sqlx::query_scalar("SELECT checksum FROM _sqlx_migrations WHERE version = ?")
            .bind(BASELINE_VERSION)
            .fetch_optional(pool)
            .await
            .map_err(AppError::database)?;

    let total_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations")
        .fetch_one(pool)
        .await
        .map_err(AppError::database)?;

    if let Some(checksum) = baseline_checksum {
        if checksum == expected_baseline_checksum {
            return Ok(MigrationHistoryState::BaselineReady);
        }
        if total_count == 1 {
            return Err(AppError::internal(
                "unsupported migration history: baseline checksum mismatch",
            ));
        }
    }

    let old_success_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM _sqlx_migrations
         WHERE version BETWEEN ? AND ? AND success = 1",
    )
    .bind(OLD_FIRST_VERSION)
    .bind(OLD_LAST_VERSION)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)?;

    let old_last_success: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM _sqlx_migrations WHERE version = ? AND success = 1",
    )
    .bind(OLD_LAST_VERSION)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)?;

    if old_success_count == OLD_LAST_VERSION && old_last_success == 1 {
        return Ok(MigrationHistoryState::OldHistoryReadyForCutover);
    }

    Err(AppError::internal(
        "unsupported migration history: expected baseline v1 or old successful versions 1 through 26",
    ))
}

pub(super) trait BaselineResetBackup {
    fn create_backup(&self, db_path: &std::path::Path) -> AppResult<std::path::PathBuf>;
}

pub(super) struct FileSystemBaselineResetBackup;

impl BaselineResetBackup for FileSystemBaselineResetBackup {
    fn create_backup(&self, db_path: &std::path::Path) -> AppResult<std::path::PathBuf> {
        let timestamp = backup_timestamp();
        let file_name = db_path
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| AppError::internal("Database path has no valid file name"))?;
        let backup_path =
            db_path.with_file_name(format!("{file_name}.pre-baseline-reset-{timestamp}.bak"));
        std::fs::copy(db_path, &backup_path).map_err(|error| {
            AppError::internal(format!("Could not create baseline reset backup: {error}"))
        })?;
        Ok(backup_path)
    }
}

fn backup_timestamp() -> String {
    let now = time::OffsetDateTime::now_utc();
    format!(
        "{:04}{:02}{:02}-{:02}{:02}{:02}",
        now.year(),
        u8::from(now.month()),
        now.day(),
        now.hour(),
        now.minute(),
        now.second()
    )
}

pub(super) async fn apply_baseline_reset_if_needed<B: BaselineResetBackup>(
    db_path: &std::path::Path,
    baseline_sql: &str,
    backup: &B,
) -> AppResult<()> {
    let url = format!("sqlite:{}", db_path.to_string_lossy());
    let pool = sqlx::SqlitePool::connect(&url)
        .await
        .map_err(AppError::database)?;

    let state = classify_migration_history(&pool, baseline_sql).await?;
    pool.close().await;

    if state == MigrationHistoryState::BaselineReady {
        return Ok(());
    }

    backup.create_backup(db_path)?;

    let pool = sqlx::SqlitePool::connect(&url)
        .await
        .map_err(AppError::database)?;
    let state = classify_migration_history(&pool, baseline_sql).await?;
    if state != MigrationHistoryState::OldHistoryReadyForCutover {
        pool.close().await;
        return Err(AppError::internal(
            "unsupported migration history changed before baseline reset could be applied",
        ));
    }

    rewrite_migration_history_to_baseline(&pool, baseline_sql).await?;
    pool.close().await;
    Ok(())
}

async fn rewrite_migration_history_to_baseline(
    pool: &sqlx::SqlitePool,
    baseline_sql: &str,
) -> AppResult<()> {
    let checksum = Sha384::digest(baseline_sql.as_bytes()).to_vec();
    let mut tx = pool.begin().await.map_err(AppError::database)?;

    sqlx::query("DELETE FROM _sqlx_migrations")
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?;

    sqlx::query(
        "INSERT INTO _sqlx_migrations (
            version, description, installed_on, success, checksum, execution_time
         ) VALUES (?, ?, CURRENT_TIMESTAMP, 1, ?, 0)",
    )
    .bind(BASELINE_VERSION)
    .bind("current schema baseline")
    .bind(checksum)
    .execute(&mut *tx)
    .await
    .map_err(AppError::database)?;

    tx.commit().await.map_err(AppError::database)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha384};

    const BASELINE_SQL_FOR_TEST: &str = "CREATE TABLE baseline_probe (id INTEGER PRIMARY KEY);";

    fn baseline_checksum() -> Vec<u8> {
        Sha384::digest(BASELINE_SQL_FOR_TEST.as_bytes()).to_vec()
    }

    async fn pool_with_migrations_table() -> sqlx::SqlitePool {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");
        create_sqlx_migrations_table_for_test(&pool).await;
        pool
    }

    async fn create_sqlx_migrations_table_for_test(pool: &sqlx::SqlitePool) {
        sqlx::query(
            r#"
            CREATE TABLE _sqlx_migrations (
                version BIGINT PRIMARY KEY,
                description TEXT NOT NULL,
                installed_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                success BOOLEAN NOT NULL,
                checksum BLOB NOT NULL,
                execution_time BIGINT NOT NULL
            )
            "#,
        )
        .execute(pool)
        .await
        .expect("create migrations table");
    }

    async fn insert_migration(
        pool: &sqlx::SqlitePool,
        version: i64,
        success: bool,
        checksum: Vec<u8>,
    ) {
        sqlx::query(
            "INSERT INTO _sqlx_migrations (
                version, description, installed_on, success, checksum, execution_time
             ) VALUES (?, ?, CURRENT_TIMESTAMP, ?, ?, 0)",
        )
        .bind(version)
        .bind(format!("migration {version}"))
        .bind(success)
        .bind(checksum)
        .execute(pool)
        .await
        .expect("insert migration row");
    }

    async fn connect_sqlite_file_for_test(
        db_path: &std::path::Path,
        create_if_missing: bool,
    ) -> sqlx::SqlitePool {
        let options = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(db_path)
            .create_if_missing(create_if_missing);
        sqlx::SqlitePool::connect_with(options)
            .await
            .expect("connect sqlite file")
    }

    async fn seed_old_history(pool: &sqlx::SqlitePool) {
        for version in 1_i64..=26_i64 {
            insert_migration(pool, version, true, vec![version as u8]).await;
        }
    }

    #[tokio::test]
    async fn classifies_baseline_history_only_when_checksum_matches() {
        let pool = pool_with_migrations_table().await;
        insert_migration(&pool, 1, true, baseline_checksum()).await;

        let state = classify_migration_history(&pool, BASELINE_SQL_FOR_TEST)
            .await
            .expect("classify history");

        assert_eq!(state, MigrationHistoryState::BaselineReady);
    }

    #[tokio::test]
    async fn classifies_baseline_history_after_post_baseline_migrations() {
        let pool = pool_with_migrations_table().await;
        insert_migration(&pool, 1, true, baseline_checksum()).await;
        insert_migration(&pool, 2, true, vec![2]).await;

        let state = classify_migration_history(&pool, BASELINE_SQL_FOR_TEST)
            .await
            .expect("classify history with post-baseline migration");

        assert_eq!(state, MigrationHistoryState::BaselineReady);
    }

    #[tokio::test]
    async fn rejects_baseline_history_with_wrong_checksum() {
        let pool = pool_with_migrations_table().await;
        insert_migration(&pool, 1, true, vec![1, 2, 3]).await;

        let error = classify_migration_history(&pool, BASELINE_SQL_FOR_TEST)
            .await
            .expect_err("reject checksum mismatch");

        assert!(error.message.contains("baseline checksum"));
    }

    #[tokio::test]
    async fn classifies_old_history_only_when_versions_one_through_twenty_six_are_successful() {
        let pool = pool_with_migrations_table().await;
        seed_old_history(&pool).await;

        let state = classify_migration_history(&pool, BASELINE_SQL_FOR_TEST)
            .await
            .expect("classify old history");

        assert_eq!(state, MigrationHistoryState::OldHistoryReadyForCutover);
    }

    #[tokio::test]
    async fn rejects_partial_old_history_without_version_twenty_six() {
        let pool = pool_with_migrations_table().await;
        for version in 1_i64..=25_i64 {
            insert_migration(&pool, version, true, vec![version as u8]).await;
        }

        let error = classify_migration_history(&pool, BASELINE_SQL_FOR_TEST)
            .await
            .expect_err("reject partial old history");

        assert!(error.message.contains("unsupported migration history"));
    }

    #[tokio::test]
    async fn rejects_failed_migration_history() {
        let pool = pool_with_migrations_table().await;
        seed_old_history(&pool).await;
        insert_migration(&pool, 99, false, vec![99]).await;

        let error = classify_migration_history(&pool, BASELINE_SQL_FOR_TEST)
            .await
            .expect_err("reject failed history");

        assert!(error.message.contains("failed migration"));
    }

    #[derive(Default)]
    struct RecordingBackup {
        calls: std::sync::Mutex<Vec<std::path::PathBuf>>,
    }

    impl BaselineResetBackup for RecordingBackup {
        fn create_backup(&self, db_path: &std::path::Path) -> AppResult<std::path::PathBuf> {
            self.calls
                .lock()
                .expect("lock calls")
                .push(db_path.to_path_buf());
            Ok(db_path.with_extension("bak"))
        }
    }

    struct FailingBackup;

    impl BaselineResetBackup for FailingBackup {
        fn create_backup(&self, _db_path: &std::path::Path) -> AppResult<std::path::PathBuf> {
            Err(AppError::internal("backup failed"))
        }
    }

    #[tokio::test]
    async fn backup_failure_prevents_migration_history_rewrite() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db_path = temp_dir.path().join("extractum.db");
        let pool = connect_sqlite_file_for_test(&db_path, true).await;
        create_sqlx_migrations_table_for_test(&pool).await;
        seed_old_history(&pool).await;
        pool.close().await;

        let error = apply_baseline_reset_if_needed(&db_path, BASELINE_SQL_FOR_TEST, &FailingBackup)
            .await
            .expect_err("backup failure blocks cutover");

        assert!(error.message.contains("backup failed"));

        let pool = connect_sqlite_file_for_test(&db_path, false).await;
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations")
            .fetch_one(&pool)
            .await
            .expect("count migrations");
        assert_eq!(count, 26);
    }

    #[tokio::test]
    async fn old_history_cutover_backs_up_then_rewrites_only_migration_history() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let db_path = temp_dir.path().join("extractum.db");
        let pool = connect_sqlite_file_for_test(&db_path, true).await;
        create_sqlx_migrations_table_for_test(&pool).await;
        seed_old_history(&pool).await;
        sqlx::query("CREATE TABLE product_probe (id INTEGER PRIMARY KEY, value TEXT NOT NULL)")
            .execute(&pool)
            .await
            .expect("create product probe");
        sqlx::query("INSERT INTO product_probe (id, value) VALUES (1, 'unchanged')")
            .execute(&pool)
            .await
            .expect("seed product probe");
        pool.close().await;

        let backup = RecordingBackup::default();
        apply_baseline_reset_if_needed(&db_path, BASELINE_SQL_FOR_TEST, &backup)
            .await
            .expect("apply baseline reset");

        assert_eq!(backup.calls.lock().expect("lock calls").len(), 1);

        let pool = connect_sqlite_file_for_test(&db_path, false).await;
        let rows: Vec<(i64, String, bool, Vec<u8>, i64)> = sqlx::query_as(
            "SELECT version, description, success, checksum, execution_time FROM _sqlx_migrations",
        )
        .fetch_all(&pool)
        .await
        .expect("read migrations");
        assert_eq!(
            rows,
            vec![(
                1,
                "current schema baseline".to_string(),
                true,
                baseline_checksum(),
                0,
            )]
        );

        let product_value: String =
            sqlx::query_scalar("SELECT value FROM product_probe WHERE id = 1")
                .fetch_one(&pool)
                .await
                .expect("read product probe");
        assert_eq!(product_value, "unchanged");
    }
}
