use std::time::Instant;

use sha2::{Digest, Sha384};
use sqlx::{Connection, SqliteConnection};

use crate::error::{AppError, AppResult};
use crate::topic_memberships::create_topic_membership_schema;

pub(super) const TOPIC_MEMBERSHIP_MATERIALIZATION_VERSION: i64 = 22;
pub(super) const TOPIC_MEMBERSHIP_MATERIALIZATION_DESCRIPTION: &str =
    "materialize telegram topic memberships";
pub(super) const TOPIC_MEMBERSHIP_MATERIALIZATION_SENTINEL_SQL: &str =
    include_str!("../../migrations/22.sql");

pub(super) async fn apply_topic_membership_materialization_if_needed(
    db_url: &str,
) -> AppResult<()> {
    let mut conn = SqliteConnection::connect(db_url)
        .await
        .map_err(AppError::database)?;
    apply_topic_membership_materialization_on_connection(&mut conn).await
}

pub(super) async fn apply_topic_membership_materialization_on_connection(
    conn: &mut SqliteConnection,
) -> AppResult<()> {
    ensure_previous_migration_recorded(conn).await?;
    if migration_22_recorded(conn).await? {
        return Ok(());
    }

    let started_at = Instant::now();
    sqlx::query("BEGIN IMMEDIATE")
        .execute(&mut *conn)
        .await
        .map_err(AppError::database)?;

    let result = async {
        create_topic_membership_schema(conn).await?;
        Ok::<(), AppError>(())
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
        expected_migration_22_checksum(),
        started_at.elapsed().as_nanos().min(i64::MAX as u128) as i64,
    )
    .await
}

async fn ensure_previous_migration_recorded(conn: &mut SqliteConnection) -> AppResult<()> {
    let exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM _sqlx_migrations WHERE version = 21 AND success = 1",
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(AppError::database)?;
    if exists == 0 {
        return Err(AppError::validation(
            "Topic membership materialization migration 22 requires migration 21",
        ));
    }
    Ok(())
}

async fn migration_22_recorded(conn: &mut SqliteConnection) -> AppResult<bool> {
    let checksum = expected_migration_22_checksum();
    let row: Option<(Vec<u8>, bool)> =
        sqlx::query_as("SELECT checksum, success FROM _sqlx_migrations WHERE version = ?")
            .bind(TOPIC_MEMBERSHIP_MATERIALIZATION_VERSION)
            .fetch_optional(&mut *conn)
            .await
            .map_err(AppError::database)?;

    match row {
        None => Ok(false),
        Some((applied_checksum, true)) if applied_checksum == checksum => Ok(true),
        Some((_applied_checksum, true)) => Err(AppError::internal(
            "Migration 22 checksum does not match the runner-managed topic membership sentinel",
        )),
        Some((_applied_checksum, false)) => Err(AppError::internal(
            "Migration 22 is marked as failed in _sqlx_migrations",
        )),
    }
}

async fn record_migration_success(
    conn: &mut SqliteConnection,
    checksum: Vec<u8>,
    execution_time: i64,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time) VALUES (?, ?, 1, ?, ?)",
    )
    .bind(TOPIC_MEMBERSHIP_MATERIALIZATION_VERSION)
    .bind(TOPIC_MEMBERSHIP_MATERIALIZATION_DESCRIPTION)
    .bind(checksum)
    .bind(execution_time)
    .execute(&mut *conn)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

fn expected_migration_22_checksum() -> Vec<u8> {
    Sha384::digest(TOPIC_MEMBERSHIP_MATERIALIZATION_SENTINEL_SQL.as_bytes()).to_vec()
}
