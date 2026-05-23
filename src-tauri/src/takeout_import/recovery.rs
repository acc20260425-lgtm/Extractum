use std::collections::HashSet;

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
