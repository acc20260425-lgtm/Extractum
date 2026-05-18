use std::time::Instant;

use sha2::{Digest, Sha384};
use sqlx::{Connection, SqliteConnection};

use crate::error::{AppError, AppResult};

pub(super) const ANALYSIS_DOCUMENTS_VERSION: i64 = 24;
pub(super) const ANALYSIS_DOCUMENTS_DESCRIPTION: &str = "add provider neutral analysis documents";
pub(super) const ANALYSIS_DOCUMENTS_SENTINEL_SQL: &str = include_str!("../../migrations/24.sql");

pub(super) async fn apply_analysis_documents_if_needed(db_url: &str) -> AppResult<()> {
    let mut conn = SqliteConnection::connect(db_url)
        .await
        .map_err(AppError::database)?;
    apply_analysis_documents_on_connection(&mut conn).await
}

pub(super) async fn apply_analysis_documents_on_connection(
    conn: &mut SqliteConnection,
) -> AppResult<()> {
    ensure_previous_migration_recorded(conn).await?;
    if migration_24_recorded(conn).await? {
        return Ok(());
    }

    let started_at = Instant::now();
    sqlx::query("BEGIN IMMEDIATE")
        .execute(&mut *conn)
        .await
        .map_err(AppError::database)?;

    let result = async {
        crate::analysis_documents::create_analysis_documents_schema(&mut *conn).await?;
        crate::analysis_documents::backfill_all_analysis_documents_on_connection(&mut *conn).await
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
        expected_analysis_documents_checksum(),
        started_at.elapsed().as_nanos().min(i64::MAX as u128) as i64,
    )
    .await
}

async fn ensure_previous_migration_recorded(conn: &mut SqliteConnection) -> AppResult<()> {
    let exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM _sqlx_migrations WHERE version = 23 AND success = 1",
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(AppError::database)?;
    if exists == 0 {
        return Err(AppError::validation(
            "Analysis documents migration 24 requires migration 23",
        ));
    }
    Ok(())
}

async fn migration_24_recorded(conn: &mut SqliteConnection) -> AppResult<bool> {
    let checksum = expected_analysis_documents_checksum();
    let row: Option<(Vec<u8>, bool)> =
        sqlx::query_as("SELECT checksum, success FROM _sqlx_migrations WHERE version = ?")
            .bind(ANALYSIS_DOCUMENTS_VERSION)
            .fetch_optional(&mut *conn)
            .await
            .map_err(AppError::database)?;

    match row {
        None => Ok(false),
        Some((applied_checksum, true)) if applied_checksum == checksum => Ok(true),
        Some((_applied_checksum, true)) => Err(AppError::internal(
            "Migration 24 checksum does not match the runner-managed analysis documents sentinel",
        )),
        Some((_applied_checksum, false)) => Err(AppError::internal(
            "Migration 24 is marked as failed in _sqlx_migrations",
        )),
    }
}

async fn record_migration_success(
    conn: &mut SqliteConnection,
    checksum: Vec<u8>,
    execution_time: i64,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time)
         VALUES (?, ?, 1, ?, ?)",
    )
    .bind(ANALYSIS_DOCUMENTS_VERSION)
    .bind(ANALYSIS_DOCUMENTS_DESCRIPTION)
    .bind(checksum)
    .bind(execution_time)
    .execute(&mut *conn)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

fn expected_analysis_documents_checksum() -> Vec<u8> {
    Sha384::digest(ANALYSIS_DOCUMENTS_SENTINEL_SQL.as_bytes()).to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compression::{compress_text, decompress_text};
    use crate::migrations::build_migrations;
    use sha2::{Digest, Sha384};
    use sqlx::{Connection, SqliteConnection};

    #[tokio::test]
    async fn migration_24_creates_schema_backfills_and_records_sentinel() {
        let mut conn = memory_conn_with_history_through_23().await;
        seed_source_and_item(&mut conn).await;

        apply_analysis_documents_on_connection(&mut conn)
            .await
            .expect("apply v24");

        let row: (String, bool, Vec<u8>) = sqlx::query_as(
            "SELECT description, success, checksum FROM _sqlx_migrations WHERE version = 24",
        )
        .fetch_one(&mut conn)
        .await
        .expect("load v24 history");
        assert_eq!(row.0, ANALYSIS_DOCUMENTS_DESCRIPTION);
        assert!(row.1);
        assert_eq!(row.2, expected_analysis_documents_checksum());

        let doc: (String, Vec<u8>) =
            sqlx::query_as("SELECT ref, content_zstd FROM analysis_documents WHERE source_id = 1")
                .fetch_one(&mut conn)
                .await
                .expect("load doc");
        assert_eq!(doc.0, "s1-i10");
        assert_eq!(
            decompress_text(&doc.1).expect("decompress"),
            "Telegram text"
        );
    }

    #[tokio::test]
    async fn migration_24_is_restart_safe_when_schema_exists_but_version_is_unrecorded() {
        let mut conn = memory_conn_with_history_through_23().await;
        seed_source_and_item(&mut conn).await;
        crate::analysis_documents::create_analysis_documents_schema(&mut conn)
            .await
            .expect("precreate schema");

        apply_analysis_documents_on_connection(&mut conn)
            .await
            .expect("apply v24 after partial schema");

        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM analysis_documents WHERE source_id = 1")
                .fetch_one(&mut conn)
                .await
                .expect("count docs");
        assert_eq!(count, 1);

        apply_analysis_documents_on_connection(&mut conn)
            .await
            .expect("second v24");
        let migration_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations WHERE version = 24")
                .fetch_one(&mut conn)
                .await
                .expect("count v24");
        assert_eq!(migration_count, 1);
    }

    async fn memory_conn_with_history_through_23() -> SqliteConnection {
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
        crate::migrations::topic_membership_materialization::apply_topic_membership_materialization_on_connection(
            &mut conn,
        )
        .await
        .expect("apply v22");
        for migration in build_migrations()
            .into_iter()
            .filter(|migration| migration.version == 23)
        {
            sqlx::raw_sql(migration.sql)
                .execute(&mut conn)
                .await
                .expect("apply v23");
            sqlx::query(
                "INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time) VALUES (?, ?, 1, ?, 0)",
            )
            .bind(migration.version)
            .bind(migration.description)
            .bind(Sha384::digest(migration.sql.as_bytes()).to_vec())
            .execute(&mut conn)
            .await
            .expect("record v23");
        }
        conn
    }

    async fn seed_source_and_item(conn: &mut SqliteConnection) {
        sqlx::query(
            "INSERT INTO accounts (id, label, api_id, api_hash, phone, created_at)
             VALUES (10, 'Test', 1, 'hash', '+10000000000', 1)",
        )
        .execute(&mut *conn)
        .await
        .expect("seed account");
        sqlx::query(
            "INSERT INTO sources (
                id, source_type, source_subtype, account_id, external_id, title,
                is_active, is_member, created_at
             ) VALUES (1, 'telegram', 'supergroup', 10, 'tg1', 'Telegram', 1, 1, 1)",
        )
        .execute(&mut *conn)
        .await
        .expect("seed source");
        sqlx::query(
            "INSERT INTO items (
                id, source_id, external_id, item_kind, author, published_at,
                ingested_at, content_kind, has_media, content_zstd
             ) VALUES (10, 1, '10', 'telegram_message', 'alice', 100, 100, 'text_only', 0, ?)",
        )
        .bind(compress_text("Telegram text").expect("compress"))
        .execute(&mut *conn)
        .await
        .expect("seed item");
    }
}
