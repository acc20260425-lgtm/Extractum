use std::time::Instant;

use sha2::{Digest, Sha384};
use sqlx::{Connection, SqliteConnection};

use crate::error::{AppError, AppResult};
use crate::topic_memberships::{
    assert_all_topic_membership_invariants, catalog_backed_supergroup_source_ids,
    create_topic_membership_schema, ensure_never_run_state_for_supergroups_without_catalog,
    rebuild_topic_memberships_for_source_on_connection,
};

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
        let now = now_secs();
        let source_ids = catalog_backed_supergroup_source_ids(conn).await?;
        for source_id in source_ids {
            rebuild_topic_memberships_for_source_on_connection(conn, source_id, now, false).await?;
        }
        ensure_never_run_state_for_supergroups_without_catalog(conn, now).await?;
        assert_all_topic_membership_invariants(conn).await
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

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
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

    #[tokio::test]
    async fn migration_22_rebuilds_catalog_sources_and_creates_never_run_state() {
        let mut conn = memory_conn_with_history_through_21().await;
        seed_supergroup_source(&mut conn, 10, true).await;
        seed_supergroup_source(&mut conn, 20, false).await;
        seed_channel_source(&mut conn, 30).await;
        seed_topic(&mut conn, 10, 200, 700, "Roadmap").await;
        seed_item(&mut conn, 1001, 10, "701", Some(200), None).await;
        seed_item(&mut conn, 1002, 10, "999", Some(404), None).await;

        apply_topic_membership_materialization_on_connection(&mut conn)
            .await
            .expect("apply v22");

        let memberships: Vec<(i64, i64, String)> = sqlx::query_as(
            "SELECT item_id, topic_id, match_kind FROM item_topic_memberships ORDER BY item_id",
        )
        .fetch_all(&mut conn)
        .await
        .expect("load memberships");
        assert_eq!(
            memberships,
            vec![(1001, 200, "reply_to_top_id".to_string())]
        );

        let states: Vec<(i64, String, i64)> = sqlx::query_as(
            "SELECT source_id, status, unresolved_count FROM telegram_topic_resolution_state ORDER BY source_id",
        )
        .fetch_all(&mut conn)
        .await
        .expect("load states");
        assert_eq!(
            states,
            vec![
                (10, "ready".to_string(), 1),
                (20, "never_run".to_string(), 0),
            ]
        );
    }

    #[tokio::test]
    async fn migration_22_rejects_state_rows_for_non_supergroups() {
        let mut conn = memory_conn_with_history_through_21().await;
        seed_channel_source(&mut conn, 30).await;
        crate::topic_memberships::create_topic_membership_schema(&mut conn)
            .await
            .expect("schema");
        sqlx::query(
            "INSERT INTO telegram_topic_resolution_state (
                source_id, resolver_version, status, unresolved_count, pending_item_count
             ) VALUES (30, 1, 'ready', 0, 0)",
        )
        .execute(&mut conn)
        .await
        .expect("dirty state row");

        let error = apply_topic_membership_materialization_on_connection(&mut conn)
            .await
            .expect_err("state invariant fails");

        assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
        assert!(error.message.contains("telegram_topic_resolution_state"));
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

    async fn seed_supergroup_source(
        conn: &mut SqliteConnection,
        source_id: i64,
        with_identity: bool,
    ) {
        seed_account(conn).await;
        sqlx::query(
            "INSERT OR IGNORE INTO sources (
                id, source_type, source_subtype, account_id, external_id, title,
                is_active, is_member, created_at
             ) VALUES (?, 'telegram', 'supergroup', 1, ?, 'Supergroup', 1, 1, 1)",
        )
        .bind(source_id)
        .bind(source_id.to_string())
        .execute(&mut *conn)
        .await
        .expect("seed supergroup");
        if with_identity {
            sqlx::query(
                "INSERT OR IGNORE INTO telegram_sources (
                    source_id, account_id, source_subtype, peer_kind, peer_id, resolution_strategy
                 ) VALUES (?, 1, 'supergroup', 'channel', ?, 'dialog')",
            )
            .bind(source_id)
            .bind(source_id)
            .execute(&mut *conn)
            .await
            .expect("seed telegram source identity");
        }
    }

    async fn seed_channel_source(conn: &mut SqliteConnection, source_id: i64) {
        seed_account(conn).await;
        sqlx::query(
            "INSERT OR IGNORE INTO sources (
                id, source_type, source_subtype, account_id, external_id, title,
                is_active, is_member, created_at
             ) VALUES (?, 'telegram', 'channel', 1, ?, 'Channel', 1, 1, 1)",
        )
        .bind(source_id)
        .bind(source_id.to_string())
        .execute(&mut *conn)
        .await
        .expect("seed channel");
    }

    async fn seed_account(conn: &mut SqliteConnection) {
        sqlx::query(
            "INSERT OR IGNORE INTO accounts (id, label, api_id, api_hash, created_at)
             VALUES (1, 'Test', 1, 'hash', 1)",
        )
        .execute(&mut *conn)
        .await
        .expect("seed account");
    }

    async fn seed_topic(
        conn: &mut SqliteConnection,
        source_id: i64,
        topic_id: i64,
        top_message_id: i64,
        title: &str,
    ) {
        sqlx::query(
            "INSERT INTO telegram_forum_topics (
                source_id, topic_id, top_message_id, title, last_seen_at, updated_at
             ) VALUES (?, ?, ?, ?, 100, 100)",
        )
        .bind(source_id)
        .bind(topic_id)
        .bind(top_message_id)
        .bind(title)
        .execute(&mut *conn)
        .await
        .expect("seed topic");
    }

    async fn seed_item(
        conn: &mut SqliteConnection,
        item_id: i64,
        source_id: i64,
        external_id: &str,
        reply_to_top_id: Option<i64>,
        reply_to_msg_id: Option<i64>,
    ) {
        sqlx::query(
            "INSERT INTO items (
                id, source_id, external_id, item_kind, author, published_at,
                ingested_at, content_kind, has_media, reply_to_top_id, reply_to_msg_id
             ) VALUES (?, ?, ?, 'telegram_message', 'alice', ?, ?, 'text_only', 0, ?, ?)",
        )
        .bind(item_id)
        .bind(source_id)
        .bind(external_id)
        .bind(item_id)
        .bind(item_id)
        .bind(reply_to_top_id)
        .bind(reply_to_msg_id)
        .execute(&mut *conn)
        .await
        .expect("seed item");
    }
}
