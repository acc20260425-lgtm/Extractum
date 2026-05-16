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

async fn run_source_identity_cleanup_rebuild(_conn: &mut SqliteConnection) -> AppResult<()> {
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
}
