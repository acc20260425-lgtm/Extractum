use sqlx::{Pool, Row, Sqlite};

use crate::error::{AppError, AppResult};
use crate::migrations::build_migrations;

use super::{
    DiagnosticAnalysisRunCount, DiagnosticAnalysisRunsInfo, DiagnosticDatabaseInfo,
    DiagnosticIngestBatchCount, DiagnosticIngestInfo, DiagnosticIngestWarningCount,
    DiagnosticItemCount, DiagnosticItemsInfo, DiagnosticMigrationInfo, DiagnosticSourceCount,
    DiagnosticSourcesInfo,
};

pub(crate) async fn load_account_ids(pool: &Pool<Sqlite>) -> AppResult<Vec<i64>> {
    sqlx::query_scalar("SELECT id FROM accounts ORDER BY id")
        .fetch_all(pool)
        .await
        .map_err(AppError::database)
}

pub(crate) async fn load_database_diagnostics(
    pool: &Pool<Sqlite>,
) -> AppResult<(
    DiagnosticDatabaseInfo,
    DiagnosticSourcesInfo,
    DiagnosticItemsInfo,
    DiagnosticAnalysisRunsInfo,
    DiagnosticIngestInfo,
)> {
    let migrations = load_migration_info(pool).await?;
    let account_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM accounts")
        .fetch_one(pool)
        .await
        .map_err(AppError::database)?;

    Ok((
        DiagnosticDatabaseInfo {
            sqlite_available: true,
            migrations,
            account_count,
        },
        DiagnosticSourcesInfo {
            counts: load_source_counts(pool).await?,
        },
        DiagnosticItemsInfo {
            counts: load_item_counts(pool).await?,
        },
        DiagnosticAnalysisRunsInfo {
            counts: load_analysis_run_counts(pool).await?,
        },
        DiagnosticIngestInfo {
            batches: load_ingest_batch_counts(pool).await?,
            warnings: load_ingest_warning_counts(pool).await?,
        },
    ))
}

async fn load_migration_info(pool: &Pool<Sqlite>) -> AppResult<DiagnosticMigrationInfo> {
    let expected_versions = build_migrations()
        .into_iter()
        .map(|migration| migration.version)
        .collect::<Vec<_>>();
    let applied_versions = sqlx::query_scalar::<_, i64>(
        "SELECT version FROM _sqlx_migrations WHERE success = 1 ORDER BY version",
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;
    let failed_versions = sqlx::query_scalar::<_, i64>(
        "SELECT version FROM _sqlx_migrations WHERE success = 0 ORDER BY version",
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;
    let pending_versions = expected_versions
        .iter()
        .copied()
        .filter(|version| !applied_versions.contains(version) && !failed_versions.contains(version))
        .collect::<Vec<_>>();
    let status = if !failed_versions.is_empty() {
        "failed"
    } else if pending_versions.is_empty() {
        "current"
    } else {
        "pending"
    };

    Ok(DiagnosticMigrationInfo {
        status: status.to_string(),
        expected_versions,
        applied_versions,
        pending_versions,
        failed_versions,
    })
}

async fn load_source_counts(pool: &Pool<Sqlite>) -> AppResult<Vec<DiagnosticSourceCount>> {
    let rows = sqlx::query(
        "SELECT
            source_type,
            source_subtype,
            COALESCE(is_active, 0) AS active,
            CASE WHEN last_synced_at IS NULL THEN 'never_synced' ELSE 'synced' END AS sync_state,
            COUNT(*) AS count
         FROM sources
         GROUP BY source_type, source_subtype, active, sync_state
         ORDER BY source_type, source_subtype, active DESC, sync_state",
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    rows.into_iter()
        .map(|row| {
            Ok(DiagnosticSourceCount {
                source_type: row.try_get("source_type").map_err(AppError::database)?,
                source_subtype: row.try_get("source_subtype").map_err(AppError::database)?,
                active: row.try_get::<i64, _>("active").map_err(AppError::database)? != 0,
                sync_state: row.try_get("sync_state").map_err(AppError::database)?,
                count: row.try_get("count").map_err(AppError::database)?,
            })
        })
        .collect()
}

async fn load_item_counts(pool: &Pool<Sqlite>) -> AppResult<Vec<DiagnosticItemCount>> {
    let rows = sqlx::query(
        "SELECT
            s.source_type,
            s.source_subtype,
            i.item_kind,
            i.content_kind,
            CASE WHEN i.content_zstd IS NULL THEN 0 ELSE 1 END AS has_content,
            COALESCE(i.has_media, 0) AS has_media,
            i.media_kind,
            COUNT(*) AS count
         FROM items i
         JOIN sources s ON s.id = i.source_id
         GROUP BY s.source_type, s.source_subtype, i.item_kind, i.content_kind,
                  has_content, has_media, i.media_kind
         ORDER BY s.source_type, s.source_subtype, i.item_kind, i.content_kind,
                  has_content DESC, has_media DESC, i.media_kind",
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    rows.into_iter()
        .map(|row| {
            Ok(DiagnosticItemCount {
                source_type: row.try_get("source_type").map_err(AppError::database)?,
                source_subtype: row.try_get("source_subtype").map_err(AppError::database)?,
                item_kind: row.try_get("item_kind").map_err(AppError::database)?,
                content_kind: row.try_get("content_kind").map_err(AppError::database)?,
                has_content: row
                    .try_get::<i64, _>("has_content")
                    .map_err(AppError::database)?
                    != 0,
                has_media: row
                    .try_get::<i64, _>("has_media")
                    .map_err(AppError::database)?
                    != 0,
                media_kind: row.try_get("media_kind").map_err(AppError::database)?,
                count: row.try_get("count").map_err(AppError::database)?,
            })
        })
        .collect()
}

async fn load_analysis_run_counts(
    pool: &Pool<Sqlite>,
) -> AppResult<Vec<DiagnosticAnalysisRunCount>> {
    // Raw analysis error text is read only to derive a coarse error_kind.
    // It must never be selected into, copied into, or summarized in the DTO.
    let rows = sqlx::query(
        "SELECT
            provider,
            run_type,
            scope_type,
            status,
            CASE
                WHEN snapshot_captured_at IS NOT NULL THEN 'captured'
                WHEN snapshot_error IS NOT NULL THEN 'failed'
                ELSE 'not_captured'
            END AS snapshot_state,
            CASE
                WHEN error IS NULL OR TRIM(error) = '' THEN 'none'
                WHEN LOWER(error) LIKE '%timeout%' OR LOWER(error) LIKE '%network%' THEN 'network'
                WHEN LOWER(error) LIKE '%unauthorized%' OR LOWER(error) LIKE '%forbidden%' OR LOWER(error) LIKE '%api key%' THEN 'auth'
                WHEN LOWER(error) LIKE '%invalid%' THEN 'validation'
                ELSE 'internal'
            END AS error_kind,
            COUNT(*) AS count
         FROM analysis_runs
         GROUP BY provider, run_type, scope_type, status, snapshot_state, error_kind
         ORDER BY provider, run_type, scope_type, status, snapshot_state, error_kind",
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    rows.into_iter()
        .map(|row| {
            Ok(DiagnosticAnalysisRunCount {
                provider: row.try_get("provider").map_err(AppError::database)?,
                run_type: row.try_get("run_type").map_err(AppError::database)?,
                scope_type: row.try_get("scope_type").map_err(AppError::database)?,
                status: row.try_get("status").map_err(AppError::database)?,
                snapshot_state: row.try_get("snapshot_state").map_err(AppError::database)?,
                error_kind: row.try_get("error_kind").map_err(AppError::database)?,
                count: row.try_get("count").map_err(AppError::database)?,
            })
        })
        .collect()
}

async fn load_ingest_batch_counts(
    pool: &Pool<Sqlite>,
) -> AppResult<Vec<DiagnosticIngestBatchCount>> {
    // Raw terminal_error text is read only to derive a coarse error_kind.
    // It must never be selected into, copied into, or summarized in the DTO.
    let rows = sqlx::query(
        "SELECT
            provider,
            ingest_kind,
            status,
            completeness,
            CASE
                WHEN terminal_error IS NULL OR TRIM(terminal_error) = '' THEN 'none'
                WHEN LOWER(terminal_error) LIKE '%timeout%' OR LOWER(terminal_error) LIKE '%network%' THEN 'network'
                WHEN LOWER(terminal_error) LIKE '%unauthorized%' OR LOWER(terminal_error) LIKE '%forbidden%' OR LOWER(terminal_error) LIKE '%api key%' THEN 'auth'
                WHEN LOWER(terminal_error) LIKE '%invalid%' THEN 'validation'
                ELSE 'internal'
            END AS error_kind,
            COUNT(*) AS count
         FROM ingest_batches
         GROUP BY provider, ingest_kind, status, completeness, error_kind
         ORDER BY provider, ingest_kind, status, completeness, error_kind",
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    rows.into_iter()
        .map(|row| {
            Ok(DiagnosticIngestBatchCount {
                provider: row.try_get("provider").map_err(AppError::database)?,
                ingest_kind: row.try_get("ingest_kind").map_err(AppError::database)?,
                status: row.try_get("status").map_err(AppError::database)?,
                completeness: row.try_get("completeness").map_err(AppError::database)?,
                error_kind: row.try_get("error_kind").map_err(AppError::database)?,
                count: row.try_get("count").map_err(AppError::database)?,
            })
        })
        .collect()
}

async fn load_ingest_warning_counts(
    pool: &Pool<Sqlite>,
) -> AppResult<Vec<DiagnosticIngestWarningCount>> {
    let rows = sqlx::query(
        "SELECT
            b.provider,
            b.ingest_kind,
            b.status,
            w.code AS warning_code,
            COUNT(*) AS count
         FROM ingest_batch_warnings w
         JOIN ingest_batches b ON b.id = w.batch_id
         GROUP BY b.provider, b.ingest_kind, b.status, w.code
         ORDER BY b.provider, b.ingest_kind, b.status, w.code",
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;

    rows.into_iter()
        .map(|row| {
            Ok(DiagnosticIngestWarningCount {
                provider: row.try_get("provider").map_err(AppError::database)?,
                ingest_kind: row.try_get("ingest_kind").map_err(AppError::database)?,
                status: row.try_get("status").map_err(AppError::database)?,
                warning_code: row.try_get("warning_code").map_err(AppError::database)?,
                count: row.try_get("count").map_err(AppError::database)?,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::migrations::{apply_all_migrations_for_test_pool, build_migrations};

    async fn memory_pool() -> Pool<Sqlite> {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");
        apply_all_migrations_for_test_pool(&pool)
            .await
            .expect("apply migrations");
        reset_sqlx_migrations_for_test(&pool).await;
        let expected_versions = expected_migration_versions();
        seed_sqlx_migrations_for_test(&pool, &expected_versions, &[]).await;
        pool
    }

    fn expected_migration_versions() -> Vec<i64> {
        build_migrations()
            .into_iter()
            .map(|migration| migration.version)
            .collect()
    }

    async fn reset_sqlx_migrations_for_test(pool: &Pool<Sqlite>) {
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
        .execute(pool)
        .await
        .expect("create _sqlx_migrations");
        sqlx::query("DELETE FROM _sqlx_migrations")
            .execute(pool)
            .await
            .expect("clear _sqlx_migrations");
    }

    async fn seed_sqlx_migrations_for_test(pool: &Pool<Sqlite>, applied: &[i64], failed: &[i64]) {
        for version in applied {
            sqlx::query(
                "INSERT OR REPLACE INTO _sqlx_migrations
                 (version, description, success, checksum, execution_time)
                 VALUES (?, 'test', 1, X'00', 0)",
            )
            .bind(version)
            .execute(pool)
            .await
            .expect("insert applied migration");
        }
        for version in failed {
            sqlx::query(
                "INSERT OR REPLACE INTO _sqlx_migrations
                 (version, description, success, checksum, execution_time)
                 VALUES (?, 'test', 0, X'00', 0)",
            )
            .bind(version)
            .execute(pool)
            .await
            .expect("insert failed migration");
        }
    }

    #[tokio::test]
    async fn database_diagnostics_groups_only_allow_listed_aggregates() {
        let pool = memory_pool().await;
        seed_safe_rows(&pool).await;
        let expected_versions = expected_migration_versions();

        let (database, sources, items, analysis_runs, ingest) =
            load_database_diagnostics(&pool).await.expect("load diagnostics");
        let account_ids = load_account_ids(&pool).await.expect("load account ids");

        assert_eq!(database.sqlite_available, true);
        assert_eq!(database.account_count, 1);
        assert_eq!(database.migrations.status, "current");
        assert_eq!(database.migrations.expected_versions, expected_versions);
        assert_eq!(
            database.migrations.applied_versions,
            database.migrations.expected_versions
        );
        assert!(database.migrations.pending_versions.is_empty());
        assert!(database.migrations.failed_versions.is_empty());
        assert_eq!(account_ids, vec![10]);
        assert_eq!(sources.counts[0].source_type, "telegram");
        assert_eq!(
            sources.counts[0].source_subtype.as_deref(),
            Some("supergroup")
        );
        assert_eq!(sources.counts[0].sync_state, "synced");
        assert_eq!(items.counts[0].has_content, true);
        assert_eq!(analysis_runs.counts[0].error_kind, "network");
        assert_eq!(ingest.batches[0].error_kind, "internal");
        assert_eq!(ingest.warnings[0].warning_code, "export_dc_fallback");

        let json = serde_json::to_string(&(database, sources, items, analysis_runs, ingest))
            .expect("serialize aggregate tuple");
        for forbidden in [
            "Private Source Title",
            "private message body",
            "https://youtube.example/watch?v=private",
            "raw provider payload",
            "C:\\Users\\Dima\\AppData",
        ] {
            assert!(
                !json.contains(forbidden),
                "aggregate leaked {forbidden}: {json}"
            );
        }
    }

    #[tokio::test]
    async fn migration_status_reports_pending_and_failed_versions() {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");
        reset_sqlx_migrations_for_test(&pool).await;
        let expected_versions = expected_migration_versions();
        assert!(
            expected_versions.len() >= 2,
            "pending/failed migration test needs at least two migrations"
        );
        let applied_version = expected_versions[0];
        let failed_version = expected_versions[1];
        let expected_pending_versions = expected_versions
            .iter()
            .copied()
            .skip(2)
            .collect::<Vec<_>>();
        seed_sqlx_migrations_for_test(&pool, &[applied_version], &[failed_version]).await;

        let migrations = load_migration_info(&pool).await.expect("load migrations");

        assert_eq!(migrations.status, "failed");
        assert_eq!(migrations.applied_versions, vec![applied_version]);
        assert_eq!(migrations.failed_versions, vec![failed_version]);
        assert_eq!(migrations.pending_versions, expected_pending_versions);
        assert_eq!(migrations.expected_versions, expected_versions);
    }

    async fn seed_safe_rows(pool: &Pool<Sqlite>) {
        sqlx::query(
            "INSERT INTO accounts (id, label, api_id, api_hash, phone, created_at)
             VALUES (10, 'Private Account', 1, '', '+10000000000', 1)",
        )
        .execute(pool)
        .await
        .expect("insert account");

        sqlx::query(
            "INSERT INTO sources (
                id, source_type, source_subtype, account_id, external_id, title,
                metadata_zstd, last_sync_state, last_synced_at, is_active, is_member, created_at
             ) VALUES (
                20, 'telegram', 'supergroup', 10, 'private-external-id',
                'Private Source Title', NULL, 123456, 1000, 1, 1, 1
             )",
        )
        .execute(pool)
        .await
        .expect("insert source");

        sqlx::query(
            "INSERT INTO items (
                id, source_id, external_id, author, published_at, ingested_at,
                content_zstd, raw_data_zstd, content_kind, has_media, media_kind, item_kind
             ) VALUES (
                30, 20, 'private-message-id', 'private author', 1, 2,
                X'00', X'00', 'text_only', 0, NULL, 'telegram_message'
             )",
        )
        .execute(pool)
        .await
        .expect("insert item");

        sqlx::query(
            "INSERT INTO analysis_runs (
                id, run_type, scope_type, source_id, period_from, period_to,
                output_language, prompt_template_version, provider_profile,
                provider, model, status, error, created_at
             ) VALUES (
                40, 'report', 'single_source', 20, 1, 2, 'Russian', 1,
                'my-private-profile', 'gemini', 'private-model', 'failed',
                'network timeout with raw provider payload and private message body', 3
             )",
        )
        .execute(pool)
        .await
        .expect("insert analysis run");

        let batch_id: i64 = sqlx::query_scalar(
            "INSERT INTO ingest_batches (
                source_id, provider, ingest_kind, status, completeness,
                finished_at, item_inserted_count, item_observed_count,
                terminal_error
             ) VALUES (
                20, 'telegram', 'takeout', 'failed', 'partial',
                CURRENT_TIMESTAMP, 0, 0,
                'C:\\Users\\Dima\\AppData\\raw terminal error'
             )
             RETURNING id",
        )
        .fetch_one(pool)
        .await
        .expect("insert ingest batch");

        sqlx::query(
            "INSERT INTO ingest_batch_warnings (batch_id, code, message)
             VALUES (?, 'export_dc_fallback', 'raw warning message with https://youtube.example/watch?v=private')",
        )
        .bind(batch_id)
        .execute(pool)
        .await
        .expect("insert warning");
    }
}
