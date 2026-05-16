use std::time::Instant;

use sha2::{Digest, Sha384};
use sqlx::{Connection, SqliteConnection};

use crate::error::{AppError, AppResult};

pub(super) const SOURCE_IDENTITY_CLEANUP_VERSION: i64 = 19;
pub(super) const SOURCE_IDENTITY_CLEANUP_DESCRIPTION: &str =
    "remove legacy telegram source kind";
pub(super) const SOURCE_IDENTITY_CLEANUP_SENTINEL_SQL: &str =
    include_str!("../../migrations/19.sql");

pub(super) async fn apply_source_identity_cleanup_if_needed(
    db_url: &str,
) -> AppResult<()> {
    let mut conn = SqliteConnection::connect(db_url)
        .await
        .map_err(AppError::database)?;
    apply_source_identity_cleanup_on_connection(&mut conn).await
}

async fn apply_source_identity_cleanup_on_connection(
    conn: &mut SqliteConnection,
) -> AppResult<()> {
    ensure_not_missing_previous_migrations(conn).await?;
    if migration_19_recorded(conn).await? {
        return Ok(());
    }

    let _started_at = Instant::now();
    Err(AppError::internal(
        "source identity cleanup migration 19 is not implemented yet",
    ))
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
    let row: Option<(Vec<u8>, bool)> = sqlx::query_as(
        "SELECT checksum, success FROM _sqlx_migrations WHERE version = ?",
    )
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
