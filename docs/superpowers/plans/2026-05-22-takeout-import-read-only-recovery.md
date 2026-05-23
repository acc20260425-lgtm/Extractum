# Takeout Import Read-Only Recovery Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Surface the latest durable incomplete, interrupted, failed, cancelled, or partial Telegram Takeout attempt as read-only source recovery state without resuming, purging, retrying, or creating pseudo-jobs.

**Architecture:** Add a backend recovery query and Tauri command that derive one recovery DTO per source from the latest Telegram Takeout batch only. Frontend state maps those DTOs by source id, hides them while an active in-memory Takeout job exists, and renders one shared read-only notice in the source row and selected-source surface. Existing Takeout start/cancel/job semantics remain the runtime source of truth.

**Tech Stack:** Rust, Tauri commands, sqlx QueryBuilder, SQLite window functions, Svelte 5, TypeScript, Vitest.

---

## File Structure

- Create `src-tauri/src/takeout_import/recovery.rs`: recovery DTO, query helper, classification helpers, warning-code lookup, and backend unit tests.
- Modify `src-tauri/src/takeout_import/mod.rs`: expose the recovery module, add the Tauri command, and keep terminal batch finalization before releasing active in-memory jobs on failed and cancelled paths.
- Modify `src-tauri/src/lib.rs`: register `list_takeout_import_recovery_states`.
- Modify `src/lib/types/sources.ts`: add recovery DTO and enum-like union types.
- Modify `src/lib/api/takeout-import.ts` and `src/lib/api/takeout-import.test.ts`: add command wrapper and API test.
- Modify `src/lib/analysis-state.ts` and `src/lib/analysis-state.test.ts`: add recovery map, visibility, label, body, facts, and severity helpers.
- Create `src/lib/components/analysis/takeout-recovery-notice.svelte`: shared read-only notice used by both UI surfaces.
- Modify `src/lib/components/analysis/source-switcher-panel.svelte`: render the shared notice under source rows when no active Takeout job exists.
- Modify `src/lib/components/analysis/compact-source-rail.svelte`: accept and forward recovery state map.
- Modify `src/lib/components/analysis/report-source-surface.svelte`: render the shared notice above the live selected Telegram source material.
- Modify `src/lib/components/analysis/report-canvas.svelte`: accept and forward selected-source recovery state.
- Modify `src/routes/analysis/+page.svelte`: load recovery states on mount, refresh after relevant Takeout events and source catalog changes, and pass recovery state into child components.
- Modify raw component tests in `src/lib/analysis-compact-source-rail.test.ts` and `src/lib/analysis-source-readers.test.ts`: pin both recovery render surfaces to the shared component.
- Modify `docs/takeout-source-import.md`: document read-only recovery semantics.
- Modify `docs/backlog.md`: keep richer recovery actions open and mark this read-only slice as shipped current-state behavior, not a closed full policy.

---

### Task 1: Backend Recovery Query Tests

**Files:**
- Create: `src-tauri/src/takeout_import/recovery.rs`
- Modify: `src-tauri/src/takeout_import/mod.rs`

- [x] **Step 1: Add the module declaration and failing backend tests**

Add this module declaration near the existing `mod export_dc;`, `mod pagination;`, and `mod state;` declarations in `src-tauri/src/takeout_import/mod.rs`:

```rust
mod recovery;
```

Create `src-tauri/src/takeout_import/recovery.rs` with tests first. These tests intentionally reference recovery functions that do not exist yet.

```rust
use std::collections::HashSet;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sources::test_support::{
        create_ingest_provenance_tables, memory_pool_with_sources,
    };
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
        sqlx::query(
            "INSERT INTO ingest_batch_warnings (batch_id, code, message) VALUES (?, ?, ?)",
        )
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
        seed_batch(&pool, BatchSeed {
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
        }).await;

        let states = list_takeout_import_recovery_states_for_sources(&pool, &state, None)
            .await
            .expect("list recovery states");

        assert!(states.is_empty());
    }

    #[tokio::test]
    async fn takeout_recovery_latest_complete_hides_older_failed() {
        let (pool, state) = recovery_fixture().await;
        seed_batch(&pool, BatchSeed {
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
        }).await;
        seed_batch(&pool, BatchSeed {
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
        }).await;

        let states = list_takeout_import_recovery_states_for_sources(&pool, &state, None)
            .await
            .expect("list recovery states");

        assert!(states.is_empty());
    }

    #[tokio::test]
    async fn takeout_recovery_latest_failed_wins_over_older_complete() {
        let (pool, state) = recovery_fixture().await;
        seed_batch(&pool, BatchSeed {
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
        }).await;
        let failed_id = seed_batch(&pool, BatchSeed {
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
        }).await;

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
        let partial_id = seed_batch(&pool, BatchSeed {
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
        }).await;
        seed_batch(&pool, BatchSeed {
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
        }).await;

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
        let running_id = seed_batch(&pool, BatchSeed {
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
        }).await;

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
        let running_id = seed_batch(&pool, BatchSeed {
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
        }).await;
        state.create_job(1, 1, running_id).await.expect("create active job");

        let states = list_takeout_import_recovery_states_for_sources(&pool, &state, None)
            .await
            .expect("list recovery states");

        assert!(states.is_empty());
    }

    #[tokio::test]
    async fn takeout_recovery_warning_codes_are_unique_sorted_and_message_free() {
        let (pool, state) = recovery_fixture().await;
        let batch_id = seed_batch(&pool, BatchSeed {
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
        }).await;
        seed_warning(&pool, batch_id, "only_my_messages", "private message body must not appear").await;
        seed_warning(&pool, batch_id, "export_dc_fallback", "dc detail must not appear").await;
        seed_warning(&pool, batch_id, "only_my_messages", "duplicate code must not appear").await;

        let states = list_takeout_import_recovery_states_for_sources(&pool, &state, None)
            .await
            .expect("list recovery states");

        assert_eq!(states.len(), 1);
        assert_eq!(states[0].warning_codes, vec!["export_dc_fallback", "only_my_messages"]);
        assert!(!states[0].warning_codes.iter().any(|code| code.contains("body")));
    }

    #[tokio::test]
    async fn takeout_recovery_source_filter_limits_results() {
        let (pool, state) = recovery_fixture().await;
        seed_batch(&pool, BatchSeed {
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
        }).await;
        seed_batch(&pool, BatchSeed {
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
        }).await;

        let states = list_takeout_import_recovery_states_for_sources(&pool, &state, Some(&[2]))
            .await
            .expect("list recovery states");
        let source_ids = states.iter().map(|state| state.source_id).collect::<HashSet<_>>();

        assert_eq!(source_ids, HashSet::from([2]));
    }
}
```

- [x] **Step 2: Run backend recovery tests and verify the expected red state**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml takeout_recovery
```

Expected: compile failure that mentions `cannot find function list_takeout_import_recovery_states_for_sources` and `cannot find type TakeoutImportRecoveryState`.

Commit nothing for this red step.

---

### Task 2: Backend Recovery DTO, Query, And Classification

**Files:**
- Modify: `src-tauri/src/takeout_import/recovery.rs`

- [x] **Step 1: Add the minimal public shapes and query implementation**

At the top of `src-tauri/src/takeout_import/recovery.rs`, above the test module, add the DTO, row shape, and query helpers.

```rust
use serde::Serialize;
use sqlx::{FromRow, QueryBuilder, Sqlite, SqlitePool};
use std::collections::HashSet;

use crate::error::AppResult;
use crate::sql_helpers::push_i64_bind_list;
use crate::takeout_import::TakeoutImportState;

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
        .filter_map(|batch| {
            recovery_kind(batch, &active_source_ids).map(|kind| (batch.batch_id, kind))
        })
        .collect::<Vec<_>>();
    let warning_codes_by_batch =
        load_warning_codes_for_batches(pool, &visible_batch_ids.iter().map(|(id, _)| *id).collect::<Vec<_>>())
            .await?;

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
        .await?)
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
) -> AppResult<std::collections::HashMap<i64, Vec<String>>> {
    if batch_ids.is_empty() {
        return Ok(std::collections::HashMap::new());
    }

    let mut query = QueryBuilder::<Sqlite>::new(
        "SELECT batch_id, code FROM ingest_batch_warnings WHERE batch_id IN (",
    );
    push_i64_bind_list(&mut query, batch_ids);
    query.push(") GROUP BY batch_id, code ORDER BY batch_id ASC, code ASC");

    let rows = query
        .build_query_as::<(i64, String)>()
        .fetch_all(pool)
        .await?;
    let mut codes_by_batch = std::collections::HashMap::<i64, Vec<String>>::new();
    for (batch_id, code) in rows {
        codes_by_batch.entry(batch_id).or_default().push(code);
    }
    Ok(codes_by_batch)
}
```

- [x] **Step 2: Run backend recovery tests and fix only compile-level mismatches**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml takeout_recovery
```

Expected: all `takeout_recovery_*` tests pass, or a compile error points to a local visibility/import mismatch in `recovery.rs`.

If a compile mismatch appears because `TakeoutImportState` is private through the module path, keep `pub use state::TakeoutImportState;` in `mod.rs` and import it as:

```rust
use super::TakeoutImportState;
```

- [x] **Step 3: Commit backend recovery query**

Run:

```powershell
git add src-tauri/src/takeout_import/mod.rs src-tauri/src/takeout_import/recovery.rs
git commit -m "feat: add takeout recovery query"
```

---

### Task 3: Backend Command And Terminal Finalization Ordering

**Files:**
- Modify: `src-tauri/src/takeout_import/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [x] **Step 1: Add the Tauri command wrapper**

In `src-tauri/src/takeout_import/mod.rs`, add this import near the other state/recovery exports:

```rust
pub(crate) use recovery::TakeoutImportRecoveryState;
use recovery::list_takeout_import_recovery_states_for_sources;
```

Add this command near `list_takeout_source_import_jobs`:

```rust
#[tauri::command]
pub async fn list_takeout_import_recovery_states(
    handle: AppHandle,
    state: tauri::State<'_, TakeoutImportState>,
) -> AppResult<Vec<TakeoutImportRecoveryState>> {
    let pool = get_pool(&handle).await?;
    list_takeout_import_recovery_states_for_sources(&pool, state.inner(), None).await
}
```

In `src-tauri/src/lib.rs`, add `list_takeout_import_recovery_states` to the existing `use takeout_import::{ start_takeout_source_import, cancel_takeout_source_import, list_takeout_source_import_jobs, run_takeout_export_dc_spike, TakeoutImportState }` import list and to the `tauri::generate_handler!` command list next to `list_takeout_source_import_jobs`.

- [x] **Step 2: Keep failure and cancellation finalization before active-job release**

In `run_takeout_import_job` in `src-tauri/src/takeout_import/mod.rs`, the first cancellation branch already finalizes before `finish_job`. Keep that order.

In the `Err(error)` branch, change the failed path so `finalize_terminal_batch_best_effort` runs before `takeout_state.finish_job`. The resulting failed branch should have this order:

```rust
let terminal_error = error.to_string();
finalize_terminal_batch_best_effort(
    &handle,
    batch_id,
    TerminalBatchStatus::Failed,
    Some(&terminal_error),
)
.await;
if let Some(record) = takeout_state
    .finish_job(&job_id, |job| {
        job.status = STATUS_FAILED.to_string();
        job.phase = PHASE_FAILED.to_string();
        job.message = None;
        job.error = Some(terminal_error.clone());
    })
    .await
{
    emit_takeout_import_event(&handle, &record);
}
```

Keep the cancellation branch in the same `Err(error)` arm in this order:

```rust
finalize_terminal_batch_best_effort(
    &handle,
    batch_id,
    TerminalBatchStatus::Cancelled,
    None,
)
.await;
if let Some(record) = takeout_state
    .finish_job(&job_id, |job| {
        job.status = STATUS_CANCELLED.to_string();
        job.phase = PHASE_CANCELLED.to_string();
        job.message = Some("Takeout import cancelled.".to_string());
    })
    .await
{
    emit_takeout_import_event(&handle, &record);
}
```

- [x] **Step 3: Run focused backend tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml takeout_recovery
```

Expected: all `takeout_recovery_*` tests pass.

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml takeout_import::tests::active_jobs_for_sources_filters_non_terminal_jobs
```

Expected: the existing active job state test passes.

- [x] **Step 4: Commit command and terminal-order wiring**

Run:

```powershell
git add src-tauri/src/takeout_import/mod.rs src-tauri/src/lib.rs
git commit -m "feat: expose takeout recovery command"
```

---

### Task 4: Frontend Types, API Wrapper, And State Helpers

**Files:**
- Modify: `src/lib/types/sources.ts`
- Modify: `src/lib/api/takeout-import.ts`
- Modify: `src/lib/api/takeout-import.test.ts`
- Modify: `src/lib/analysis-state.ts`
- Modify: `src/lib/analysis-state.test.ts`

- [x] **Step 1: Add failing API and state tests**

In `src/lib/api/takeout-import.test.ts`, import the new wrapper:

```ts
  listTakeoutImportRecoveryStates,
```

Add this test after the existing job list test:

```ts
  it("lists takeout import recovery states with the read-only command", async () => {
    invokeMock.mockResolvedValueOnce([]);

    await expect(listTakeoutImportRecoveryStates()).resolves.toEqual([]);

    expect(invokeMock).toHaveBeenLastCalledWith("list_takeout_import_recovery_states");
  });
```

In `src/lib/analysis-state.test.ts`, add the new imports:

```ts
  applyTakeoutImportRecoveryStates,
  takeoutRecoveryBody,
  takeoutRecoveryFacts,
  takeoutRecoverySeverity,
  takeoutRecoveryTitle,
  visibleTakeoutRecoveryForSource,
```

Add this helper near `takeoutJob`:

```ts
function takeoutRecovery(
  overrides: Partial<TakeoutImportRecoveryState>,
): TakeoutImportRecoveryState {
  return Object.assign({
    batch_id: 10,
    source_id: 1,
    status: "running",
    recovery_kind: "interrupted",
    completeness: "unknown",
    item_inserted_count: 0,
    item_duplicate_count: 0,
    item_skipped_count: 0,
    item_observed_count: 0,
    warning_count: 0,
    warning_codes: [],
    terminal_error: null,
    started_at: 1_700_000,
    finished_at: null,
    updated_at: 1_700_030,
  }, overrides);
}
```

Add these tests near the existing Takeout job tests:

```ts
  it("maps takeout recovery states by source id", () => {
    const first = takeoutRecovery({ source_id: 1, batch_id: 11 });
    const second = takeoutRecovery({ source_id: 2, batch_id: 12, recovery_kind: "failed" });

    expect(applyTakeoutImportRecoveryStates([first, second])).toEqual({
      1: first,
      2: second,
    });
  });

  it("hides durable recovery while an active takeout job exists for the source", () => {
    const recovery = takeoutRecovery({ source_id: 1 });
    const active = takeoutJob({ source_id: 1, status: "running" });
    const terminal = takeoutJob({ source_id: 1, status: "completed" });

    expect(visibleTakeoutRecoveryForSource(1, { 1: active }, { 1: recovery })).toBeNull();
    expect(visibleTakeoutRecoveryForSource(1, { 1: terminal }, { 1: recovery })).toBe(recovery);
    expect(visibleTakeoutRecoveryForSource(2, { 1: active }, { 1: recovery })).toBeNull();
  });

  it("formats takeout recovery title, body, facts, and severity", () => {
    expect(takeoutRecoveryTitle(takeoutRecovery({ recovery_kind: "interrupted" })))
      .toBe("Previous Takeout import was interrupted");
    expect(takeoutRecoveryTitle(takeoutRecovery({ recovery_kind: "failed" })))
      .toBe("Previous Takeout import failed");
    expect(takeoutRecoveryTitle(takeoutRecovery({ recovery_kind: "cancelled" })))
      .toBe("Previous Takeout import was cancelled");
    expect(takeoutRecoveryTitle(takeoutRecovery({ recovery_kind: "partial_completed" })))
      .toBe("Previous Takeout import completed with partial history");
    expect(takeoutRecoveryBody()).toBe(
      "Run Takeout again to continue collecting available history. Messages already saved locally will be deduplicated.",
    );
    expect(takeoutRecoverySeverity(takeoutRecovery({ recovery_kind: "failed" }))).toBe("danger");
    expect(takeoutRecoverySeverity(takeoutRecovery({ recovery_kind: "interrupted" }))).toBe("warning");
    expect(takeoutRecoverySeverity(takeoutRecovery({ recovery_kind: "partial_completed" }))).toBe("warning");
    expect(takeoutRecoverySeverity(takeoutRecovery({ recovery_kind: "cancelled" }))).toBe("neutral");
  });

  it("formats takeout recovery facts and zero-count attempts", () => {
    expect(takeoutRecoveryFacts(takeoutRecovery({}))).toEqual([
      "No items were written in this attempt.",
    ]);
    expect(takeoutRecoveryFacts(takeoutRecovery({
      item_inserted_count: 2,
      item_duplicate_count: 3,
      item_skipped_count: 1,
      item_observed_count: 6,
      warning_count: 2,
    }))).toEqual([
      "2 inserted",
      "3 duplicates",
      "1 skipped",
      "6 observed",
      "2 warnings",
    ]);
  });
```

- [x] **Step 2: Run frontend focused tests and verify red state**

Run:

```powershell
npm.cmd test -- src/lib/api/takeout-import.test.ts src/lib/analysis-state.test.ts
```

Expected: failures mention missing `listTakeoutImportRecoveryStates`, missing `TakeoutImportRecoveryState`, and missing recovery helper exports.

- [x] **Step 3: Add frontend types and API wrapper**

In `src/lib/types/sources.ts`, add `batch_id` to `TakeoutImportJobRecord` so TypeScript matches the existing Rust payload:

```ts
  batch_id: number;
```

Add these types after `TakeoutImportEvent`:

```ts
export type TakeoutImportRecoveryStatus =
  | "running"
  | "failed"
  | "cancelled"
  | "completed";

export type TakeoutImportRecoveryKind =
  | "interrupted"
  | "failed"
  | "cancelled"
  | "partial_completed";

export type TakeoutImportCompleteness =
  | "unknown"
  | "complete"
  | "partial";

export interface TakeoutImportRecoveryState {
  batch_id: number;
  source_id: number;
  status: TakeoutImportRecoveryStatus;
  recovery_kind: TakeoutImportRecoveryKind;
  completeness: TakeoutImportCompleteness;
  item_inserted_count: number;
  item_duplicate_count: number;
  item_skipped_count: number;
  item_observed_count: number;
  warning_count: number;
  warning_codes: string[];
  terminal_error: string | null;
  started_at: number;
  finished_at: number | null;
  updated_at: number;
}
```

In `src/lib/api/takeout-import.ts`, import the type and add the wrapper:

```ts
  TakeoutImportRecoveryState,
```

```ts
export function listTakeoutImportRecoveryStates() {
  return invoke<TakeoutImportRecoveryState[]>("list_takeout_import_recovery_states");
}
```

Update the `TakeoutImportEvent` payload literal in `src/lib/api/takeout-import.test.ts` by adding:

```ts
      batch_id: 100,
```

- [x] **Step 4: Add frontend state helpers**

In `src/lib/analysis-state.ts`, import `BadgeVariant` and the recovery type:

```ts
import type { BadgeVariant } from "$lib/components/ui/types";
```

```ts
  TakeoutImportRecoveryState,
```

Add these helpers near the existing Takeout job helpers:

```ts
export function isActiveTakeoutImportJob(job: TakeoutImportJobRecord | undefined) {
  return (
    job?.status === "queued" ||
    job?.status === "running" ||
    job?.status === "cancel_requested"
  );
}

export function applyTakeoutImportRecoveryStates(
  states: TakeoutImportRecoveryState[],
) {
  return Object.fromEntries(states.map((state) => [state.source_id, state]));
}

export function visibleTakeoutRecoveryForSource(
  sourceId: number,
  takeoutJobsBySource: Record<number, TakeoutImportJobRecord>,
  recoveryBySource: Record<number, TakeoutImportRecoveryState>,
) {
  if (isActiveTakeoutImportJob(takeoutJobsBySource[sourceId])) {
    return null;
  }
  return recoveryBySource[sourceId] ?? null;
}

export function takeoutRecoveryTitle(recovery: TakeoutImportRecoveryState) {
  switch (recovery.recovery_kind) {
    case "interrupted":
      return "Previous Takeout import was interrupted";
    case "failed":
      return "Previous Takeout import failed";
    case "cancelled":
      return "Previous Takeout import was cancelled";
    case "partial_completed":
      return "Previous Takeout import completed with partial history";
  }
}

export function takeoutRecoveryBody() {
  return "Run Takeout again to continue collecting available history. Messages already saved locally will be deduplicated.";
}

export function takeoutRecoverySeverity(
  recovery: TakeoutImportRecoveryState,
): BadgeVariant {
  switch (recovery.recovery_kind) {
    case "failed":
      return "danger";
    case "interrupted":
    case "partial_completed":
      return "warning";
    case "cancelled":
      return "neutral";
  }
}

function plural(value: number, singular: string, pluralLabel: string) {
  return `${value} ${value === 1 ? singular : pluralLabel}`;
}

export function takeoutRecoveryFacts(recovery: TakeoutImportRecoveryState) {
  const facts: string[] = [];
  if (recovery.item_inserted_count > 0) {
    facts.push(plural(recovery.item_inserted_count, "inserted", "inserted"));
  }
  if (recovery.item_duplicate_count > 0) {
    facts.push(plural(recovery.item_duplicate_count, "duplicate", "duplicates"));
  }
  if (recovery.item_skipped_count > 0) {
    facts.push(plural(recovery.item_skipped_count, "skipped", "skipped"));
  }
  if (recovery.item_observed_count > 0) {
    facts.push(plural(recovery.item_observed_count, "observed", "observed"));
  }
  if (recovery.warning_count > 0) {
    facts.push(plural(recovery.warning_count, "warning", "warnings"));
  }
  return facts.length > 0 ? facts : ["No items were written in this attempt."];
}
```

Replace any local duplicate active-Takeout helper added later in components with `isActiveTakeoutImportJob` from this module.

- [x] **Step 5: Run frontend focused tests**

Run:

```powershell
npm.cmd test -- src/lib/api/takeout-import.test.ts src/lib/analysis-state.test.ts
```

Expected: both files pass.

- [x] **Step 6: Commit frontend API and helper layer**

Run:

```powershell
git add src/lib/types/sources.ts src/lib/api/takeout-import.ts src/lib/api/takeout-import.test.ts src/lib/analysis-state.ts src/lib/analysis-state.test.ts
git commit -m "feat: add takeout recovery frontend state"
```

---

### Task 5: Shared Recovery Notice Component

**Files:**
- Create: `src/lib/components/analysis/takeout-recovery-notice.svelte`
- Modify: `src/lib/analysis-compact-source-rail.test.ts`
- Modify: `src/lib/analysis-source-readers.test.ts`

- [x] **Step 1: Add raw tests for shared component usage**

In `src/lib/analysis-compact-source-rail.test.ts`, add:

```ts
  it("uses the shared takeout recovery notice in source rows", () => {
    expect(sourceSwitcherPanelSource).toContain("TakeoutRecoveryNotice");
    expect(sourceSwitcherPanelSource).toContain("visibleTakeoutRecoveryForSource");
  });
```

In `src/lib/analysis-source-readers.test.ts`, add:

```ts
  it("uses the shared takeout recovery notice in the selected source surface", () => {
    expect(reportSourceSurfaceSource).toContain("TakeoutRecoveryNotice");
    expect(reportSourceSurfaceSource).toContain("takeoutRecovery");
  });
```

- [x] **Step 2: Run raw tests and verify red state**

Run:

```powershell
npm.cmd test -- src/lib/analysis-compact-source-rail.test.ts src/lib/analysis-source-readers.test.ts
```

Expected: both new tests fail because the component is not imported or used yet.

- [x] **Step 3: Create the shared notice component**

Create `src/lib/components/analysis/takeout-recovery-notice.svelte`:

```svelte
<script lang="ts">
  import Badge from "$lib/components/ui/Badge.svelte";
  import {
    takeoutRecoveryBody,
    takeoutRecoveryFacts,
    takeoutRecoverySeverity,
    takeoutRecoveryTitle,
  } from "$lib/analysis-state";
  import type { TakeoutImportRecoveryState } from "$lib/types/sources";

  let {
    recovery,
    compact = false,
  }: {
    recovery: TakeoutImportRecoveryState;
    compact?: boolean;
  } = $props();

  const title = $derived(takeoutRecoveryTitle(recovery));
  const body = $derived(takeoutRecoveryBody());
  const severity = $derived(takeoutRecoverySeverity(recovery));
  const facts = $derived(takeoutRecoveryFacts(recovery));
  const showTerminalError = $derived(
    recovery.recovery_kind === "failed" && !!recovery.terminal_error,
  );
</script>

<section class="takeout-recovery-notice" class:compact aria-label={title}>
  <div class="takeout-recovery-heading">
    <Badge variant={severity}>{recovery.recovery_kind.replaceAll("_", " ")}</Badge>
    <strong>{title}</strong>
  </div>
  {#if !compact}
    <p>{body}</p>
  {/if}
  <div class="takeout-recovery-facts">
    {#each facts as fact}
      <span>{fact}</span>
    {/each}
  </div>
  {#if recovery.warning_codes.length > 0}
    <div class="takeout-recovery-codes">
      {#each recovery.warning_codes as code}
        <Badge variant="neutral">{code}</Badge>
      {/each}
    </div>
  {/if}
  {#if showTerminalError}
    <p class="takeout-recovery-error">{recovery.terminal_error}</p>
  {/if}
</section>

<style>
  .takeout-recovery-notice {
    display: grid;
    gap: 0.45rem;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: color-mix(in srgb, var(--panel-hover) 60%, transparent);
    padding: 0.75rem;
    color: var(--text);
  }

  .takeout-recovery-notice.compact {
    padding: 0.6rem;
  }

  .takeout-recovery-heading,
  .takeout-recovery-facts,
  .takeout-recovery-codes {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 0.4rem;
    min-width: 0;
  }

  .takeout-recovery-heading strong {
    min-width: 0;
    font-size: 0.86rem;
    line-height: 1.25;
  }

  .takeout-recovery-notice p {
    margin: 0;
    color: var(--muted);
    font-size: 0.84rem;
    line-height: 1.45;
  }

  .takeout-recovery-facts span {
    color: var(--muted);
    font-size: 0.78rem;
    line-height: 1.35;
  }

  .takeout-recovery-error {
    color: var(--danger);
  }
</style>
```

- [x] **Step 4: Run the component-adjacent tests**

Run:

```powershell
npm.cmd test -- src/lib/analysis-state.test.ts
```

Expected: helper tests still pass.

The raw tests still fail until Task 6 wires the component into both surfaces.

- [x] **Step 5: Commit shared component**

Run:

```powershell
git add src/lib/components/analysis/takeout-recovery-notice.svelte src/lib/analysis-compact-source-rail.test.ts src/lib/analysis-source-readers.test.ts
git commit -m "feat: add takeout recovery notice component"
```

---

### Task 6: UI Wiring And Route Refresh Flow

**Files:**
- Modify: `src/lib/components/analysis/source-switcher-panel.svelte`
- Modify: `src/lib/components/analysis/compact-source-rail.svelte`
- Modify: `src/lib/components/analysis/report-source-surface.svelte`
- Modify: `src/lib/components/analysis/report-canvas.svelte`
- Modify: `src/routes/analysis/+page.svelte`

- [x] **Step 1: Wire source switcher row**

In `src/lib/components/analysis/source-switcher-panel.svelte`, import the component and helpers:

```svelte
  import TakeoutRecoveryNotice from "$lib/components/analysis/takeout-recovery-notice.svelte";
  import {
    isActiveTakeoutImportJob,
    visibleTakeoutRecoveryForSource,
  } from "$lib/analysis-state";
```

Add the type import:

```ts
    TakeoutImportRecoveryState,
```

Add this prop:

```ts
    takeoutRecoveryBySource,
```

and this prop type:

```ts
    takeoutRecoveryBySource: Record<number, TakeoutImportRecoveryState>;
```

Replace the local `isActiveTakeoutJob` function with calls to `isActiveTakeoutImportJob`.

Inside each source row, after `takeoutJob` and `takeoutActive`, add:

```svelte
          {@const takeoutRecovery = visibleTakeoutRecoveryForSource(source.id, takeoutJobsBySource, takeoutRecoveryBySource)}
```

After the existing `{#if takeoutJob}` Takeout status block, add:

```svelte
            {:else if takeoutRecovery}
              <TakeoutRecoveryNotice recovery={takeoutRecovery} compact />
```

The final priority in this row must be active job UI first, terminal job UI if present, recovery notice only when no job is present, then no Takeout notice.

- [x] **Step 2: Forward recovery state through CompactSourceRail**

In `src/lib/components/analysis/compact-source-rail.svelte`, import `TakeoutImportRecoveryState`, add `takeoutRecoveryBySource` to props, and forward it to `<SourceSwitcherPanel>`.

Add this prop type:

```ts
    takeoutRecoveryBySource: Record<number, TakeoutImportRecoveryState>;
```

Forward it:

```svelte
      {takeoutRecoveryBySource}
```

- [x] **Step 3: Wire selected source surface**

In `src/lib/components/analysis/report-source-surface.svelte`, import the shared component and type:

```svelte
  import TakeoutRecoveryNotice from "$lib/components/analysis/takeout-recovery-notice.svelte";
```

```ts
    TakeoutImportRecoveryState,
```

Add this prop:

```ts
    takeoutRecovery?: TakeoutImportRecoveryState | null;
```

Default it in `$props()`:

```ts
    takeoutRecovery = null,
```

Render the notice above the live selected Telegram source material. Place it after the reader header/status area and before the topic selector or timeline:

```svelte
  {#if canvasSurface === "live_source" && currentSource?.sourceType === "telegram" && takeoutRecovery}
    <TakeoutRecoveryNotice recovery={takeoutRecovery} />
  {/if}
```

- [x] **Step 4: Forward selected recovery through ReportCanvas**

In `src/lib/components/analysis/report-canvas.svelte`, import `TakeoutImportRecoveryState`, add:

```ts
    takeoutRecovery?: TakeoutImportRecoveryState | null;
```

Default to `null` and pass to `<ReportSourceSurface>`:

```svelte
      {takeoutRecovery}
```

- [x] **Step 5: Load and refresh recovery state on the analysis route**

In `src/routes/analysis/+page.svelte`, import:

```ts
    listTakeoutImportRecoveryStates,
```

and state helpers:

```ts
    applyTakeoutImportRecoveryStates,
    visibleTakeoutRecoveryForSource,
```

Add route state:

```ts
  let takeoutRecoveryBySource = $state<Record<number, TakeoutImportRecoveryState>>({});
```

Add loader:

```ts
  async function loadTakeoutImportRecoveryStates() {
    const states = await listTakeoutImportRecoveryStates();
    takeoutRecoveryBySource = applyTakeoutImportRecoveryStates(states);
  }
```

Add selected-source helper:

```ts
  function currentTakeoutRecovery() {
    const source = currentSource();
    if (!source) return null;
    return visibleTakeoutRecoveryForSource(
      source.id,
      takeoutJobsBySource,
      takeoutRecoveryBySource,
    );
  }
```

Call `loadTakeoutImportRecoveryStates()` on mount next to `loadTakeoutImportJobs()`.

In `applyTakeoutImportEvent`, after any terminal `completed`, `failed`, or `cancelled` event, call:

```ts
      void loadTakeoutImportRecoveryStates();
```

Use the terminal condition:

```ts
    if (job.status === "completed" || job.status === "failed" || job.status === "cancelled") {
      void loadTakeoutImportRecoveryStates();
    }
```

After a successful `startTakeoutImport`, call `void loadTakeoutImportRecoveryStates();` so stale durable recovery clears once the active job exists.

After source deletion completes and after source catalog reload paths that prune deleted sources, call `void loadTakeoutImportRecoveryStates();`.

Pass the map to `<CompactSourceRail>`:

```svelte
    {takeoutRecoveryBySource}
```

Pass selected recovery to `<ReportCanvas>`:

```svelte
    takeoutRecovery={currentTakeoutRecovery()}
```

- [x] **Step 6: Run UI-focused tests**

Run:

```powershell
npm.cmd test -- src/lib/analysis-compact-source-rail.test.ts src/lib/analysis-source-readers.test.ts src/lib/analysis-state.test.ts src/lib/api/takeout-import.test.ts
```

Expected: all focused frontend tests pass.

- [x] **Step 7: Commit UI wiring**

Run:

```powershell
git add src/lib/components/analysis/source-switcher-panel.svelte src/lib/components/analysis/compact-source-rail.svelte src/lib/components/analysis/report-source-surface.svelte src/lib/components/analysis/report-canvas.svelte src/routes/analysis/+page.svelte
git commit -m "feat: show takeout recovery notices"
```

---

### Task 7: Documentation And Backlog

**Files:**
- Modify: `docs/takeout-source-import.md`
- Modify: `docs/backlog.md`

- [ ] **Step 1: Update current-state Takeout docs**

In `docs/takeout-source-import.md`, add a new subsection after the existing in-memory job/restart discussion in section `2. User-Facing Behavior`:

```md
After restart, the analysis workspace can also show a read-only recovery notice
for the latest durable Telegram Takeout batch for a source. This notice is not a
job and has no cancel, resume, purge, or retry semantics.

Recovery notice priority is:

1. active in-memory Takeout job;
2. latest durable recovery state;
3. no Takeout notice.

The suggested recovery path is to run Takeout again. Existing messages already
saved locally are deduplicated by typed Telegram identity.
```

In section `9. Persistence Semantics`, replace the existing short `running` batch paragraph with:

```md
`running` batches survive restart. The schema does not persist an
`interrupted` status. The recovery query derives `interrupted` when the latest
Telegram Takeout batch for a source is still `running` and no active in-memory
Takeout job exists for that source.

Recovery selection uses latest-attempt-wins semantics. Older failed or
cancelled Takeout batches are hidden if a newer complete Takeout batch exists.
A newer failed, cancelled, interrupted, or partial completed Takeout batch is
shown even when an older complete attempt exists.

The read-only recovery DTO exposes counts, sorted warning codes, bounded failed
terminal error detail, timestamps, durable status, and derived recovery kind. It
does not expose warning messages, Telegram payloads, message text, source
identity details, account/session/API data, or full provenance history.
```

Add this sentence to the migrated-history paragraph:

```md
Read-only recovery state does not enable migrated-history import, resume, purge,
or automatic retry.
```

- [ ] **Step 2: Update backlog wording without closing broader policy work**

In `docs/backlog.md`, replace:

```md
- [ ] define the incomplete-import policy and user/recovery behavior on top of
  existing ingest batches, Telegram Takeout batch details, warnings, and item
  observations
```

with:

```md
- [ ] define richer incomplete-import recovery actions and user policy beyond
  the shipped read-only recovery state
```

Keep the real-data validation bullets open.

- [ ] **Step 3: Commit docs**

Run:

```powershell
git add docs/takeout-source-import.md docs/backlog.md
git commit -m "docs: document takeout read-only recovery"
```

---

### Task 8: Full Verification

**Files:**
- No planned file edits.

- [ ] **Step 1: Run frontend test suite**

Run:

```powershell
npm.cmd test
```

Expected: all frontend tests pass.

- [ ] **Step 2: Run Rust test suite**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: all Rust tests pass.

- [ ] **Step 3: Inspect working tree**

Run:

```powershell
git status --short --branch
```

Expected: clean working tree on `takeout-import-read-only-recovery`.

- [ ] **Step 4: Record final verification in handoff**

In the final handoff, report:

- the branch name;
- the last commit hash and subject;
- `npm.cmd test` result;
- `cargo test --manifest-path src-tauri/Cargo.toml` result;
- `git status --short --branch` result;
- any tests that were not run and why.

---

## Self-Review Checklist

- Spec coverage: The plan covers Telegram Takeout-only filtering, latest-attempt-wins selection, source-scoped internal helper, all-source public command, active-job suppression, warning-code sorting, terminal-error gating, interrupted completeness semantics, UI priority, both UI surfaces, no new button, refresh without polling, docs, and backlog wording.
- Placeholder scan: The plan contains concrete file paths, commands, and code snippets for each code-changing step.
- Type consistency: Backend field names use snake_case and match Tauri serialization. Frontend DTO fields use the same snake_case names already used by existing Tauri payloads. Recovery kinds are `interrupted`, `failed`, `cancelled`, and `partial_completed` everywhere.
