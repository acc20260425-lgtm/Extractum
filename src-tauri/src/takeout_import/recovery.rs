use serde::Serialize;
use sqlx::{FromRow, QueryBuilder, Sqlite, SqlitePool};
use std::collections::{HashMap, HashSet};

use super::TakeoutImportState;
use crate::error::{AppError, AppResult};
use crate::sql_helpers::push_i64_bind_list;

const STATUS_RUNNING: &str = "running";
const STATUS_FAILED: &str = "failed";
const STATUS_CANCELLED: &str = "cancelled";
const STATUS_COMPLETED: &str = "completed";
const COMPLETENESS_PARTIAL: &str = "partial";

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub(crate) struct TakeoutImportRecoveryState {
    pub(crate) batch_id: i64,
    pub(crate) source_id: i64,
    pub(crate) status: String,
    pub(crate) recovery_kind: String,
    pub(crate) completeness: String,
    pub(crate) item_inserted_count: i64,
    pub(crate) item_duplicate_count: i64,
    pub(crate) item_skipped_count: i64,
    pub(crate) item_observed_count: i64,
    pub(crate) warning_count: i64,
    pub(crate) warning_codes: Vec<String>,
    pub(crate) terminal_error: Option<String>,
    pub(crate) started_at: i64,
    pub(crate) finished_at: Option<i64>,
    pub(crate) updated_at: i64,
}

#[derive(Clone, Debug, FromRow)]
struct LatestTakeoutBatchRow {
    batch_id: i64,
    source_id: i64,
    status: String,
    completeness: String,
    item_inserted_count: i64,
    item_duplicate_count: i64,
    item_skipped_count: i64,
    item_observed_count: i64,
    warning_count: i64,
    terminal_error: Option<String>,
    started_at: i64,
    finished_at: Option<i64>,
    updated_at: i64,
}

pub(crate) async fn list_takeout_import_recovery_states_for_sources(
    pool: &SqlitePool,
    state: &TakeoutImportState,
    source_ids: Option<&[i64]>,
) -> AppResult<Vec<TakeoutImportRecoveryState>> {
    if matches!(source_ids, Some(values) if values.is_empty()) {
        return Ok(Vec::new());
    }

    let latest_batches = latest_takeout_batches(pool, source_ids).await?;
    let latest_source_ids = latest_batches
        .iter()
        .map(|batch| batch.source_id)
        .collect::<Vec<_>>();
    let active_source_ids = state
        .active_jobs_for_sources(&latest_source_ids)
        .await
        .into_iter()
        .map(|job| job.source_id)
        .collect::<HashSet<_>>();
    let visible_batch_ids = latest_batches
        .iter()
        .filter_map(|batch| recovery_kind(batch, &active_source_ids).map(|_| batch.batch_id))
        .collect::<Vec<_>>();
    let warning_codes_by_batch = load_warning_codes_for_batches(pool, &visible_batch_ids).await?;

    let mut states = Vec::new();
    for batch in latest_batches {
        let Some(kind) = recovery_kind(&batch, &active_source_ids) else {
            continue;
        };
        let warning_codes = warning_codes_by_batch
            .get(&batch.batch_id)
            .cloned()
            .unwrap_or_default();
        let terminal_error = if kind == "failed" {
            batch.terminal_error.clone()
        } else {
            None
        };
        states.push(TakeoutImportRecoveryState {
            batch_id: batch.batch_id,
            source_id: batch.source_id,
            status: batch.status,
            recovery_kind: kind.to_string(),
            completeness: batch.completeness,
            item_inserted_count: batch.item_inserted_count,
            item_duplicate_count: batch.item_duplicate_count,
            item_skipped_count: batch.item_skipped_count,
            item_observed_count: batch.item_observed_count,
            warning_count: batch.warning_count,
            warning_codes,
            terminal_error,
            started_at: batch.started_at,
            finished_at: batch.finished_at,
            updated_at: batch.updated_at,
        });
    }
    states.sort_by_key(|state| state.source_id);
    Ok(states)
}

async fn latest_takeout_batches(
    pool: &SqlitePool,
    source_ids: Option<&[i64]>,
) -> AppResult<Vec<LatestTakeoutBatchRow>> {
    let mut query = QueryBuilder::<Sqlite>::new(
        r#"
        WITH latest_takeout AS (
          SELECT
            b.id AS batch_id,
            b.source_id,
            b.status,
            b.completeness,
            b.item_inserted_count,
            b.item_duplicate_count,
            b.item_skipped_count,
            b.item_observed_count,
            b.warning_count,
            b.terminal_error,
            CAST(strftime('%s', b.started_at) AS INTEGER) AS started_at,
            CASE
              WHEN b.finished_at IS NULL THEN NULL
              ELSE CAST(strftime('%s', b.finished_at) AS INTEGER)
            END AS finished_at,
            CAST(strftime('%s', b.updated_at) AS INTEGER) AS updated_at,
            ROW_NUMBER() OVER (
              PARTITION BY b.source_id
              ORDER BY b.started_at DESC, b.id DESC
            ) AS row_number
          FROM ingest_batches b
          JOIN telegram_takeout_batches t ON t.batch_id = b.id
          WHERE b.provider = 'telegram'
            AND b.ingest_kind = 'takeout'
        "#,
    );
    if let Some(source_ids) = source_ids {
        query.push(" AND b.source_id IN (");
        push_i64_bind_list(&mut query, source_ids);
        query.push(")");
    }
    query.push(
        r#"
        )
        SELECT
          batch_id,
          source_id,
          status,
          completeness,
          item_inserted_count,
          item_duplicate_count,
          item_skipped_count,
          item_observed_count,
          warning_count,
          terminal_error,
          started_at,
          finished_at,
          updated_at
        FROM latest_takeout
        WHERE row_number = 1
        ORDER BY source_id ASC
        "#,
    );

    Ok(query
        .build_query_as::<LatestTakeoutBatchRow>()
        .fetch_all(pool)
        .await
        .map_err(AppError::database)?)
}

fn recovery_kind<'a>(
    batch: &LatestTakeoutBatchRow,
    active_source_ids: &HashSet<i64>,
) -> Option<&'a str> {
    match batch.status.as_str() {
        STATUS_RUNNING if active_source_ids.contains(&batch.source_id) => None,
        STATUS_RUNNING => Some("interrupted"),
        STATUS_FAILED => Some("failed"),
        STATUS_CANCELLED => Some("cancelled"),
        STATUS_COMPLETED if is_incomplete_completeness(&batch.completeness) => {
            Some("partial_completed")
        }
        STATUS_COMPLETED => None,
        _ => None,
    }
}

fn is_incomplete_completeness(completeness: &str) -> bool {
    completeness == COMPLETENESS_PARTIAL
}

async fn load_warning_codes_for_batches(
    pool: &SqlitePool,
    batch_ids: &[i64],
) -> AppResult<HashMap<i64, Vec<String>>> {
    if batch_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let mut query = QueryBuilder::<Sqlite>::new(
        "SELECT batch_id, code FROM ingest_batch_warnings WHERE batch_id IN (",
    );
    push_i64_bind_list(&mut query, batch_ids);
    query.push(") GROUP BY batch_id, code ORDER BY batch_id ASC, code ASC");

    let rows = query
        .build_query_as::<(i64, String)>()
        .fetch_all(pool)
        .await
        .map_err(AppError::database)?;
    let mut codes_by_batch = HashMap::<i64, Vec<String>>::new();
    for (batch_id, code) in rows {
        codes_by_batch.entry(batch_id).or_default().push(code);
    }
    Ok(codes_by_batch)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sources::test_support::{create_ingest_provenance_tables, memory_pool_with_sources};
    use crate::takeout_import::TakeoutImportState;

    async fn seed_source(pool: &sqlx::SqlitePool, source_id: i64, external_id: &str) {
        sqlx::query(
            r#"
            INSERT INTO sources (
                id, source_type, source_subtype, account_id, external_id,
                title, last_sync_state, last_synced_at, is_active, is_member, created_at
            )
            VALUES (?, 'telegram', 'channel', 1, ?, ?, NULL, NULL, 1, 1, 1700000000)
            "#,
        )
        .bind(source_id)
        .bind(external_id)
        .bind(external_id)
        .execute(pool)
        .await
        .expect("seed source");
    }

    struct BatchSeed<'a> {
        source_id: i64,
        provider: &'a str,
        ingest_kind: &'a str,
        status: &'a str,
        completeness: &'a str,
        started_at: &'a str,
        finished_at: Option<&'a str>,
        updated_at: &'a str,
        inserted: i64,
        duplicate: i64,
        skipped: i64,
        observed: i64,
        warning_count: i64,
        terminal_error: Option<&'a str>,
        takeout: bool,
    }

    async fn seed_batch(pool: &sqlx::SqlitePool, seed: BatchSeed<'_>) -> i64 {
        let batch_id = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO ingest_batches (
                source_id, provider, ingest_kind, status, completeness,
                started_at, finished_at, item_inserted_count,
                item_observed_count, item_duplicate_count, item_skipped_count,
                warning_count, terminal_error, created_at, updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING id
            "#,
        )
        .bind(seed.source_id)
        .bind(seed.provider)
        .bind(seed.ingest_kind)
        .bind(seed.status)
        .bind(seed.completeness)
        .bind(seed.started_at)
        .bind(seed.finished_at)
        .bind(seed.inserted)
        .bind(seed.observed)
        .bind(seed.duplicate)
        .bind(seed.skipped)
        .bind(seed.warning_count)
        .bind(seed.terminal_error)
        .bind(seed.started_at)
        .bind(seed.updated_at)
        .fetch_one(pool)
        .await
        .expect("seed ingest batch");

        if seed.takeout {
            sqlx::query(
                r#"
                INSERT INTO telegram_takeout_batches (
                    batch_id, account_id, source_subtype, history_scope
                )
                VALUES (?, 1, 'channel', 'unknown')
                "#,
            )
            .bind(batch_id)
            .execute(pool)
            .await
            .expect("seed takeout batch detail");
        }

        batch_id
    }

    async fn seed_warning(pool: &sqlx::SqlitePool, batch_id: i64, code: &str, message: &str) {
        sqlx::query("INSERT INTO ingest_batch_warnings (batch_id, code, message) VALUES (?, ?, ?)")
            .bind(batch_id)
            .bind(code)
            .bind(message)
            .execute(pool)
            .await
            .expect("seed warning");
    }

    async fn recovery_fixture() -> (sqlx::SqlitePool, TakeoutImportState) {
        let pool = memory_pool_with_sources().await;
        create_ingest_provenance_tables(&pool).await;
        seed_source(&pool, 1, "source-one").await;
        seed_source(&pool, 2, "source-two").await;
        (pool, TakeoutImportState::new())
    }

    #[tokio::test]
    async fn takeout_recovery_ignores_non_takeout_batches() {
        let (pool, state) = recovery_fixture().await;
        seed_batch(
            &pool,
            BatchSeed {
                source_id: 1,
                provider: "telegram",
                ingest_kind: "sync",
                status: "failed",
                completeness: "partial",
                started_at: "2026-05-22 10:00:00",
                finished_at: Some("2026-05-22 10:01:00"),
                updated_at: "2026-05-22 10:01:00",
                inserted: 1,
                duplicate: 0,
                skipped: 0,
                observed: 1,
                warning_count: 0,
                terminal_error: Some("sync failed"),
                takeout: false,
            },
        )
        .await;

        let states = list_takeout_import_recovery_states_for_sources(&pool, &state, None)
            .await
            .expect("list recovery states");

        assert!(states.is_empty());
    }

    #[tokio::test]
    async fn takeout_recovery_latest_complete_hides_older_failed() {
        let (pool, state) = recovery_fixture().await;
        seed_batch(
            &pool,
            BatchSeed {
                source_id: 1,
                provider: "telegram",
                ingest_kind: "takeout",
                status: "failed",
                completeness: "partial",
                started_at: "2026-05-22 10:00:00",
                finished_at: Some("2026-05-22 10:05:00"),
                updated_at: "2026-05-22 10:05:00",
                inserted: 3,
                duplicate: 0,
                skipped: 0,
                observed: 3,
                warning_count: 0,
                terminal_error: Some("older failure"),
                takeout: true,
            },
        )
        .await;
        seed_batch(
            &pool,
            BatchSeed {
                source_id: 1,
                provider: "telegram",
                ingest_kind: "takeout",
                status: "completed",
                completeness: "complete",
                started_at: "2026-05-22 11:00:00",
                finished_at: Some("2026-05-22 11:05:00"),
                updated_at: "2026-05-22 11:05:00",
                inserted: 10,
                duplicate: 2,
                skipped: 0,
                observed: 12,
                warning_count: 0,
                terminal_error: None,
                takeout: true,
            },
        )
        .await;

        let states = list_takeout_import_recovery_states_for_sources(&pool, &state, None)
            .await
            .expect("list recovery states");

        assert!(states.is_empty());
    }

    #[tokio::test]
    async fn takeout_recovery_latest_failed_wins_over_older_complete() {
        let (pool, state) = recovery_fixture().await;
        seed_batch(
            &pool,
            BatchSeed {
                source_id: 1,
                provider: "telegram",
                ingest_kind: "takeout",
                status: "completed",
                completeness: "complete",
                started_at: "2026-05-22 10:00:00",
                finished_at: Some("2026-05-22 10:05:00"),
                updated_at: "2026-05-22 10:05:00",
                inserted: 10,
                duplicate: 0,
                skipped: 0,
                observed: 10,
                warning_count: 0,
                terminal_error: None,
                takeout: true,
            },
        )
        .await;
        let failed_id = seed_batch(
            &pool,
            BatchSeed {
                source_id: 1,
                provider: "telegram",
                ingest_kind: "takeout",
                status: "failed",
                completeness: "partial",
                started_at: "2026-05-22 12:00:00",
                finished_at: Some("2026-05-22 12:05:00"),
                updated_at: "2026-05-22 12:05:00",
                inserted: 2,
                duplicate: 1,
                skipped: 1,
                observed: 4,
                warning_count: 0,
                terminal_error: Some("failed later"),
                takeout: true,
            },
        )
        .await;

        let states = list_takeout_import_recovery_states_for_sources(&pool, &state, None)
            .await
            .expect("list recovery states");

        assert_eq!(states.len(), 1);
        assert_eq!(states[0].batch_id, failed_id);
        assert_eq!(states[0].source_id, 1);
        assert_eq!(states[0].recovery_kind, "failed");
        assert_eq!(states[0].terminal_error.as_deref(), Some("failed later"));
    }

    #[tokio::test]
    async fn takeout_recovery_returns_partial_completed_and_hides_complete() {
        let (pool, state) = recovery_fixture().await;
        let partial_id = seed_batch(
            &pool,
            BatchSeed {
                source_id: 1,
                provider: "telegram",
                ingest_kind: "takeout",
                status: "completed",
                completeness: "partial",
                started_at: "2026-05-22 10:00:00",
                finished_at: Some("2026-05-22 10:05:00"),
                updated_at: "2026-05-22 10:05:00",
                inserted: 5,
                duplicate: 2,
                skipped: 1,
                observed: 8,
                warning_count: 1,
                terminal_error: Some("must not display"),
                takeout: true,
            },
        )
        .await;
        seed_batch(
            &pool,
            BatchSeed {
                source_id: 2,
                provider: "telegram",
                ingest_kind: "takeout",
                status: "completed",
                completeness: "complete",
                started_at: "2026-05-22 10:00:00",
                finished_at: Some("2026-05-22 10:05:00"),
                updated_at: "2026-05-22 10:05:00",
                inserted: 5,
                duplicate: 0,
                skipped: 0,
                observed: 5,
                warning_count: 0,
                terminal_error: None,
                takeout: true,
            },
        )
        .await;

        let states = list_takeout_import_recovery_states_for_sources(&pool, &state, None)
            .await
            .expect("list recovery states");

        assert_eq!(states.len(), 1);
        assert_eq!(states[0].batch_id, partial_id);
        assert_eq!(states[0].recovery_kind, "partial_completed");
        assert!(states[0].terminal_error.is_none());
    }

    #[tokio::test]
    async fn takeout_recovery_running_without_active_job_is_interrupted() {
        let (pool, state) = recovery_fixture().await;
        let running_id = seed_batch(
            &pool,
            BatchSeed {
                source_id: 1,
                provider: "telegram",
                ingest_kind: "takeout",
                status: "running",
                completeness: "unknown",
                started_at: "2026-05-22 10:00:00",
                finished_at: None,
                updated_at: "2026-05-22 10:03:00",
                inserted: 0,
                duplicate: 0,
                skipped: 0,
                observed: 0,
                warning_count: 0,
                terminal_error: Some("must not display"),
                takeout: true,
            },
        )
        .await;

        let states = list_takeout_import_recovery_states_for_sources(&pool, &state, None)
            .await
            .expect("list recovery states");

        assert_eq!(states.len(), 1);
        assert_eq!(states[0].batch_id, running_id);
        assert_eq!(states[0].recovery_kind, "interrupted");
        assert_eq!(states[0].completeness, "unknown");
        assert!(states[0].terminal_error.is_none());
    }

    #[tokio::test]
    async fn takeout_recovery_running_with_active_job_is_hidden() {
        let (pool, state) = recovery_fixture().await;
        let running_id = seed_batch(
            &pool,
            BatchSeed {
                source_id: 1,
                provider: "telegram",
                ingest_kind: "takeout",
                status: "running",
                completeness: "unknown",
                started_at: "2026-05-22 10:00:00",
                finished_at: None,
                updated_at: "2026-05-22 10:03:00",
                inserted: 0,
                duplicate: 0,
                skipped: 0,
                observed: 0,
                warning_count: 0,
                terminal_error: None,
                takeout: true,
            },
        )
        .await;
        state
            .create_job(1, 1, running_id)
            .await
            .expect("create active job");

        let states = list_takeout_import_recovery_states_for_sources(&pool, &state, None)
            .await
            .expect("list recovery states");

        assert!(states.is_empty());
    }

    #[tokio::test]
    async fn takeout_recovery_warning_codes_are_unique_sorted_and_message_free() {
        let (pool, state) = recovery_fixture().await;
        let batch_id = seed_batch(
            &pool,
            BatchSeed {
                source_id: 1,
                provider: "telegram",
                ingest_kind: "takeout",
                status: "failed",
                completeness: "partial",
                started_at: "2026-05-22 10:00:00",
                finished_at: Some("2026-05-22 10:05:00"),
                updated_at: "2026-05-22 10:05:00",
                inserted: 1,
                duplicate: 1,
                skipped: 1,
                observed: 3,
                warning_count: 3,
                terminal_error: Some("redacted failure"),
                takeout: true,
            },
        )
        .await;
        seed_warning(
            &pool,
            batch_id,
            "only_my_messages",
            "private message body must not appear",
        )
        .await;
        seed_warning(
            &pool,
            batch_id,
            "export_dc_fallback",
            "dc detail must not appear",
        )
        .await;
        seed_warning(
            &pool,
            batch_id,
            "only_my_messages",
            "duplicate code must not appear",
        )
        .await;

        let states = list_takeout_import_recovery_states_for_sources(&pool, &state, None)
            .await
            .expect("list recovery states");

        assert_eq!(states.len(), 1);
        assert_eq!(
            states[0].warning_codes,
            vec!["export_dc_fallback", "only_my_messages"]
        );
        assert!(!states[0]
            .warning_codes
            .iter()
            .any(|code| code.contains("body")));
    }

    #[tokio::test]
    async fn takeout_recovery_source_filter_limits_results() {
        let (pool, state) = recovery_fixture().await;
        seed_batch(
            &pool,
            BatchSeed {
                source_id: 1,
                provider: "telegram",
                ingest_kind: "takeout",
                status: "failed",
                completeness: "partial",
                started_at: "2026-05-22 10:00:00",
                finished_at: Some("2026-05-22 10:05:00"),
                updated_at: "2026-05-22 10:05:00",
                inserted: 1,
                duplicate: 0,
                skipped: 0,
                observed: 1,
                warning_count: 0,
                terminal_error: Some("source one failed"),
                takeout: true,
            },
        )
        .await;
        seed_batch(
            &pool,
            BatchSeed {
                source_id: 2,
                provider: "telegram",
                ingest_kind: "takeout",
                status: "failed",
                completeness: "partial",
                started_at: "2026-05-22 10:00:00",
                finished_at: Some("2026-05-22 10:05:00"),
                updated_at: "2026-05-22 10:05:00",
                inserted: 1,
                duplicate: 0,
                skipped: 0,
                observed: 1,
                warning_count: 0,
                terminal_error: Some("source two failed"),
                takeout: true,
            },
        )
        .await;

        let states = list_takeout_import_recovery_states_for_sources(&pool, &state, Some(&[2]))
            .await
            .expect("list recovery states");
        let source_ids = states
            .iter()
            .map(|state| state.source_id)
            .collect::<HashSet<_>>();

        assert_eq!(source_ids, HashSet::from([2]));
    }
}
