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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::migrations::build_migrations;
    use sha2::{Digest, Sha384};
    use sqlx::SqliteConnection;

    #[tokio::test]
    async fn migration_22_creates_membership_and_state_schema() {
        let mut conn = memory_conn_with_history_through_21().await;

        apply_topic_membership_materialization_on_connection(&mut conn)
            .await
            .expect("apply v22");

        for table in ["item_topic_memberships", "telegram_topic_resolution_state"] {
            let exists: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?",
            )
            .bind(table)
            .fetch_one(&mut conn)
            .await
            .expect("check table");
            assert_eq!(exists, 1, "missing table {table}");
        }

        for index in [
            "idx_item_topic_memberships_source_topic",
            "idx_item_topic_memberships_source_item",
        ] {
            let exists: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = ?",
            )
            .bind(index)
            .fetch_one(&mut conn)
            .await
            .expect("check index");
            assert_eq!(exists, 1, "missing index {index}");
        }
    }

    #[tokio::test]
    async fn migration_22_records_sentinel_checksum_and_is_idempotent() {
        let mut conn = memory_conn_with_history_through_21().await;

        apply_topic_membership_materialization_on_connection(&mut conn)
            .await
            .expect("first v22");
        apply_topic_membership_materialization_on_connection(&mut conn)
            .await
            .expect("second v22");

        let row: (String, bool, Vec<u8>) = sqlx::query_as(
            "SELECT description, success, checksum FROM _sqlx_migrations WHERE version = 22",
        )
        .fetch_one(&mut conn)
        .await
        .expect("load v22 history");
        assert_eq!(row.0, TOPIC_MEMBERSHIP_MATERIALIZATION_DESCRIPTION);
        assert!(row.1);
        assert_eq!(row.2, expected_migration_22_checksum());
    }

    async fn memory_conn_with_history_through_21() -> SqliteConnection {
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
            sqlx::query(
                "INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time) VALUES (?, ?, 1, ?, 0)",
            )
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
        crate::migrations::telegram_item_native_identity::apply_telegram_item_native_identity_on_connection(
            &mut conn,
        )
        .await
        .expect("apply v21");

        conn
    }
}
