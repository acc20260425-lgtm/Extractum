# Account Deletion Coordination Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make account deletion safer by blocking known active related work at preflight time, returning typed `not_found` or `conflict` before any destructive mutation.

**Architecture:** Add a focused `account_deletion` preflight module that inspects owned source ids, source ingest locks, active Takeout jobs, active source jobs, active analysis runs, and active LLM requests. Keep the public error generic and sanitized; expose small read-only state helpers where existing state types need inspection. Wire `delete_account` so preflight runs before account-row deletion, then preserve the current cleanup order.

**Tech Stack:** Rust/Tauri commands, SQLx SQLite, existing in-memory job states, existing `SecretStoreState` test double, Vitest for frontend regression smoke.

---

## Current Design Commit

Use the approved spec in:

```text
docs/superpowers/specs/2026-05-22-account-deletion-coordination-design.md
```

Key constraints:

- no auto-cancel;
- no global account-deletion semaphore in this slice;
- conflict message stays generic and sanitized;
- source-group blockers use current `analysis_source_group_members`;
- active LLM follow-up chat blocks deletion through `owner_run_id = Some(run_id)` even if the report run is already completed.

## Files

- Create: `src-tauri/src/account_deletion.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/accounts.rs`
- Modify: `src-tauri/src/source_ingest.rs`
- Modify: `src-tauri/src/takeout_import/state.rs`
- Modify: `src-tauri/src/youtube/jobs.rs`
- Modify: `src-tauri/src/analysis/mod.rs`
- Modify: `src-tauri/src/analysis/chat.rs`
- Modify: `docs/backlog.md`
- Modify: `docs/superpowers/plans/2026-05-22-account-deletion-coordination.md`

## Task 1: Plan Checkpoint And Baseline

**Files:**
- Modify: `docs/superpowers/plans/2026-05-22-account-deletion-coordination.md`

- [x] **Step 1: Verify branch and tracked state**

Run:

```powershell
git status --short --branch
```

Expected: branch `account-deletion-coordination` and only this plan file modified/untracked.

- [x] **Step 2: Verify baseline tests still pass**

Run:

```powershell
npm.cmd test
```

Expected: `55` test files and `441` tests pass.

- [x] **Step 3: Mark Task 1 complete and commit**

Update this task's checkboxes to `[x]`, then run:

```powershell
git add docs/superpowers/plans/2026-05-22-account-deletion-coordination.md
git commit -m "docs: plan account deletion coordination"
```

## Task 2: Add Read-Only Active Work Helpers

**Files:**
- Modify: `src-tauri/src/source_ingest.rs`
- Modify: `src-tauri/src/takeout_import/state.rs`
- Modify: `src-tauri/src/youtube/jobs.rs`
- Modify: `src-tauri/src/llm/scheduler.rs`
- Modify: `docs/superpowers/plans/2026-05-22-account-deletion-coordination.md`

- [ ] **Step 1: Write failing tests for source ingest active-source inspection**

Add tests to `src-tauri/src/source_ingest.rs`:

```rust
#[tokio::test]
async fn active_kinds_for_sources_reports_matching_locks_only() {
    let locks = SourceIngestLocks::new();
    let _sync = locks
        .try_acquire(7, SourceIngestKind::Sync)
        .await
        .expect("sync lock");
    let _delete = locks
        .try_acquire(8, SourceIngestKind::Delete)
        .await
        .expect("delete lock");

    let active = locks
        .active_kinds_for_sources(&[7, 9, 8])
        .await
        .expect("active locks");

    assert_eq!(active.len(), 2);
    assert_eq!(active.get(&7), Some(&SourceIngestKind::Sync));
    assert_eq!(active.get(&8), Some(&SourceIngestKind::Delete));
    assert_eq!(active.get(&9), None);
}
```

- [ ] **Step 2: Run source ingest tests and verify RED**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml source_ingest::tests::active_kinds_for_sources_reports_matching_locks_only
```

Expected: compile failure because `active_kinds_for_sources` does not exist.

- [ ] **Step 3: Implement source ingest active-source inspection**

Add the imports and method in `src-tauri/src/source_ingest.rs`:

```rust
use std::collections::{HashMap, HashSet};
```

```rust
pub(crate) async fn active_kinds_for_sources(
    &self,
    source_ids: &[i64],
) -> AppResult<HashMap<i64, SourceIngestKind>> {
    let source_ids = source_ids.iter().copied().collect::<HashSet<_>>();
    let state = self
        .state
        .lock()
        .map_err(|_| AppError::internal("Source ingest lock state is poisoned"))?;
    Ok(state
        .active
        .iter()
        .filter_map(|(source_id, kind)| source_ids.contains(source_id).then_some((*source_id, *kind)))
        .collect())
}
```

- [ ] **Step 4: Write failing tests for Takeout active jobs**

Add to `src-tauri/src/takeout_import/state.rs` tests:

```rust
#[tokio::test]
async fn active_jobs_for_sources_filters_non_terminal_jobs() {
    let state = TakeoutImportState::new();
    let first = state.create_job(7, 1, 100).await.expect("first job");
    let second = state.create_job(8, 1, 101).await.expect("second job");
    state
        .finish_job(&second.job_id, |job| {
            job.status = STATUS_FAILED.to_string();
            job.phase = STATUS_FAILED.to_string();
        })
        .await
        .expect("finish second");

    let active = state.active_jobs_for_sources(&[7, 8, 9]).await;

    assert_eq!(active.len(), 1);
    assert_eq!(active[0].job_id, first.job_id);
    assert_eq!(active[0].source_id, 7);
}
```

- [ ] **Step 5: Run Takeout test and verify RED**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml takeout_import::state::tests::active_jobs_for_sources_filters_non_terminal_jobs
```

Expected: compile failure because `active_jobs_for_sources` does not exist.

- [ ] **Step 6: Implement Takeout active job inspection**

Add to `impl TakeoutImportState`:

```rust
pub(crate) async fn active_jobs_for_sources(
    &self,
    source_ids: &[i64],
) -> Vec<TakeoutImportJobRecord> {
    let source_ids = source_ids.iter().copied().collect::<std::collections::HashSet<_>>();
    let mut jobs = self
        .inner
        .lock()
        .await
        .jobs
        .values()
        .filter(|job| source_ids.contains(&job.source_id))
        .filter(|job| !is_terminal_status(&job.status))
        .cloned()
        .collect::<Vec<_>>();
    jobs.sort_by_key(|job| (job.started_at, job.job_id.clone()));
    jobs
}
```

- [ ] **Step 7: Write failing tests for generic source jobs**

Add to `src-tauri/src/youtube/jobs.rs` tests:

```rust
#[tokio::test]
async fn active_jobs_for_sources_matches_source_and_related_source() {
    let state = SourceJobState::new();
    let options = YoutubeSyncOptions {
        metadata: true,
        transcripts: false,
        comments: false,
    };
    let source_job = state
        .create_job(7, SourceJobType::YoutubeVideoFullSync, None, options.clone())
        .await
        .expect("source job");
    let related_job = state
        .create_job(
            20,
            SourceJobType::YoutubePlaylistVideoSync,
            Some(8),
            options.clone(),
        )
        .await
        .expect("related job");
    let terminal_job = state
        .create_job(9, SourceJobType::YoutubeVideoFullSync, None, options)
        .await
        .expect("terminal job");
    state
        .finish_job(&terminal_job.job_id, |job| {
            job.status = SourceJobStatus::Succeeded;
        })
        .await
        .expect("finish terminal");

    let active = state.active_jobs_for_sources(&[7, 8, 9]).await;
    let job_ids = active
        .iter()
        .map(|job| job.job_id.as_str())
        .collect::<Vec<_>>();

    assert_eq!(job_ids, vec![source_job.job_id.as_str(), related_job.job_id.as_str()]);
}
```

- [ ] **Step 8: Run source job test and verify RED**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml youtube::jobs::tests::active_jobs_for_sources_matches_source_and_related_source
```

Expected: compile failure because `active_jobs_for_sources` does not exist.

- [ ] **Step 9: Implement generic source job active inspection**

Add to `impl SourceJobState`:

```rust
pub(crate) async fn active_jobs_for_sources(&self, source_ids: &[i64]) -> Vec<SourceJobRecord> {
    let source_ids = source_ids.iter().copied().collect::<std::collections::HashSet<_>>();
    let mut jobs = self
        .inner
        .lock()
        .await
        .jobs
        .values()
        .filter(|job| {
            source_ids.contains(&job.source_id)
                || job
                    .related_source_id
                    .is_some_and(|related_source_id| source_ids.contains(&related_source_id))
        })
        .filter(|job| !is_terminal_status(&job.status))
        .cloned()
        .collect::<Vec<_>>();
    jobs.sort_by_key(|job| (job.started_at, job.job_id.clone()));
    jobs
}
```

- [ ] **Step 10: Write failing test for LLM owner-run helper**

Add to `src-tauri/src/llm/scheduler.rs` tests:

```rust
#[tokio::test]
async fn active_owner_run_ids_reports_running_and_queued_owned_requests() {
    let scheduler = Arc::new(LlmSchedulerState::new());
    let first_release = Arc::new(Notify::new());
    let second_release = Arc::new(Notify::new());

    let running = {
        let scheduler = Arc::clone(&scheduler);
        let first_release = Arc::clone(&first_release);
        tokio::spawn(async move {
            scheduler
                .run_request(
                    LlmRequestMetadata {
                        owner_run_id: Some(77),
                        ..metadata(
                            "owned-running",
                            "default",
                            LlmRequestPriority::Background,
                            LlmRequestKind::AnalysisReportMap,
                        )
                    },
                    |_| {},
                    move |control| async move {
                        control
                            .run_cancellable(async move {
                                first_release.notified().await;
                                Ok::<_, AppError>("done")
                            })
                            .await
                    },
                )
                .await
        })
    };
    let queued = {
        let scheduler = Arc::clone(&scheduler);
        let second_release = Arc::clone(&second_release);
        tokio::spawn(async move {
            scheduler
                .run_request(
                    LlmRequestMetadata {
                        owner_run_id: Some(88),
                        ..metadata(
                            "owned-queued",
                            "default",
                            LlmRequestPriority::Background,
                            LlmRequestKind::AnalysisChat,
                        )
                    },
                    |_| {},
                    move |control| async move {
                        control
                            .run_cancellable(async move {
                                second_release.notified().await;
                                Ok::<_, AppError>("done")
                            })
                            .await
                    },
                )
                .await
        })
    };

    timeout(Duration::from_secs(1), async {
        loop {
            let owners = scheduler.active_owner_run_ids().await;
            if owners.contains(&77) && owners.contains(&88) {
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("owned requests registered");

    let owners = scheduler.active_owner_run_ids().await;
    assert!(owners.contains(&77));
    assert!(owners.contains(&88));

    first_release.notify_waiters();
    second_release.notify_waiters();
    let _ = running.await;
    let _ = queued.await;
}
```

- [ ] **Step 11: Run LLM helper test and verify RED**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml llm::scheduler::tests::active_owner_run_ids_reports_running_and_queued_owned_requests
```

Expected: compile failure because `active_owner_run_ids` does not exist.

- [ ] **Step 12: Implement LLM owner-run helper**

Add to `impl LlmSchedulerState`:

```rust
pub(crate) async fn active_owner_run_ids(&self) -> std::collections::HashSet<i64> {
    self.inner
        .lock()
        .await
        .requests
        .values()
        .filter_map(|entry| entry.meta.owner_run_id)
        .collect()
}
```

- [ ] **Step 13: Run helper tests and commit**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml source_ingest::tests::active_kinds_for_sources_reports_matching_locks_only
cargo test --manifest-path src-tauri/Cargo.toml takeout_import::state::tests::active_jobs_for_sources_filters_non_terminal_jobs
cargo test --manifest-path src-tauri/Cargo.toml youtube::jobs::tests::active_jobs_for_sources_matches_source_and_related_source
cargo test --manifest-path src-tauri/Cargo.toml llm::scheduler::tests::active_owner_run_ids_reports_running_and_queued_owned_requests
```

Expected: all four tests pass.

Update this task's checkboxes to `[x]`, then run:

```powershell
git add src-tauri/src/source_ingest.rs src-tauri/src/takeout_import/state.rs src-tauri/src/youtube/jobs.rs src-tauri/src/llm/scheduler.rs docs/superpowers/plans/2026-05-22-account-deletion-coordination.md
git commit -m "feat: expose active work state for account deletion"
```

## Task 3: Add Account Deletion Preflight Module

**Files:**
- Create: `src-tauri/src/account_deletion.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/analysis/mod.rs`
- Modify: `docs/superpowers/plans/2026-05-22-account-deletion-coordination.md`

- [ ] **Step 1: Create failing preflight tests and module skeleton**

Create `src-tauri/src/account_deletion.rs` with tests first. Include this public surface and intentionally missing implementation body:

```rust
use std::collections::{BTreeSet, HashSet};

use sqlx::{Pool, Sqlite};

use crate::analysis::AnalysisState;
use crate::error::{AppError, AppResult};
use crate::llm::LlmSchedulerState;
use crate::source_ingest::SourceIngestLocks;
use crate::takeout_import::TakeoutImportState;
use crate::youtube::jobs::SourceJobState;

pub(crate) const ACCOUNT_DELETE_ACTIVE_WORK_CONFLICT_MESSAGE: &str =
    "Cannot delete account while source sync, import, source job, or analysis work is running for its sources. Stop or wait for the active work, then try again.";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AccountDeletionPlan {
    pub(crate) account_id: i64,
    pub(crate) owned_source_ids: Vec<i64>,
    #[cfg(test)]
    pub(crate) blocking_work: Vec<AccountDeletionBlocker>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AccountDeletionBlocker {
    SourceIngest { source_id: i64 },
    TakeoutImport { source_id: i64, job_id: String },
    SourceJob { source_id: i64, related_source_id: Option<i64>, job_id: String },
    AnalysisRun { run_id: i64 },
    LlmRequest { run_id: i64 },
}

pub(crate) async fn check_account_deletion(
    _pool: &Pool<Sqlite>,
    _account_id: i64,
    _source_locks: &SourceIngestLocks,
    _takeout_state: &TakeoutImportState,
    _source_job_state: &SourceJobState,
    _analysis_state: &AnalysisState,
    _llm_scheduler: &LlmSchedulerState,
) -> AppResult<AccountDeletionPlan> {
    unimplemented!("account deletion preflight")
}
```

Add `mod account_deletion;` to `src-tauri/src/lib.rs`.

- [ ] **Step 2: Add account-deletion test schema and basic tests**

In `account_deletion.rs`, add test helpers for in-memory schema:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use crate::analysis::AnalysisState;
    use crate::error::AppErrorKind;
    use crate::llm::LlmSchedulerState;
    use crate::source_ingest::{SourceIngestKind, SourceIngestLocks};
    use crate::takeout_import::TakeoutImportState;
    use crate::youtube::jobs::{
        SourceJobState, SourceJobType, YoutubeSyncOptions,
    };

    async fn pool() -> sqlx::SqlitePool {
        let pool = crate::sources::test_support::memory_pool_with_sources().await;
        sqlx::query(
            "CREATE TABLE accounts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                label TEXT NOT NULL,
                api_id INTEGER NOT NULL,
                api_hash TEXT NOT NULL,
                phone TEXT,
                created_at INTEGER NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .expect("create accounts");
        sqlx::query(
            "CREATE TABLE analysis_source_groups (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                source_type TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
        )
        .execute(&pool)
        .await
        .expect("create groups");
        sqlx::query(
            "CREATE TABLE analysis_source_group_members (
                group_id INTEGER NOT NULL,
                source_id INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                PRIMARY KEY (group_id, source_id)
            )",
        )
        .execute(&pool)
        .await
        .expect("create group members");
        sqlx::query(
            "CREATE TABLE analysis_runs (
                id INTEGER PRIMARY KEY,
                status TEXT NOT NULL,
                source_id INTEGER,
                source_group_id INTEGER,
                created_at INTEGER NOT NULL DEFAULT 1
            )",
        )
        .execute(&pool)
        .await
        .expect("create runs");
        pool
    }

    async fn insert_account(pool: &sqlx::SqlitePool, account_id: i64) {
        sqlx::query(
            "INSERT INTO accounts (id, label, api_id, api_hash, created_at)
             VALUES (?, 'Account', 12345, '', 1)",
        )
        .bind(account_id)
        .execute(pool)
        .await
        .expect("insert account");
    }

    async fn insert_source(pool: &sqlx::SqlitePool, source_id: i64, account_id: Option<i64>) {
        sqlx::query(
            "INSERT INTO sources (id, source_type, source_subtype, account_id, external_id, title, metadata_zstd, created_at)
             VALUES (?, 'telegram', 'channel', ?, ?, 'Source', x'00', 1)",
        )
        .bind(source_id)
        .bind(account_id)
        .bind(format!("source-{source_id}"))
        .execute(pool)
        .await
        .expect("insert source");
    }

    struct States {
        source_locks: SourceIngestLocks,
        takeout_state: TakeoutImportState,
        source_job_state: SourceJobState,
        analysis_state: AnalysisState,
        llm_scheduler: Arc<LlmSchedulerState>,
    }

    impl States {
        fn new() -> Self {
            Self {
                source_locks: SourceIngestLocks::new(),
                takeout_state: TakeoutImportState::new(),
                source_job_state: SourceJobState::new(),
                analysis_state: AnalysisState::new(),
                llm_scheduler: Arc::new(LlmSchedulerState::new()),
            }
        }

        async fn check(
            &self,
            pool: &sqlx::SqlitePool,
            account_id: i64,
        ) -> AppResult<AccountDeletionPlan> {
            check_account_deletion(
                pool,
                account_id,
                &self.source_locks,
                &self.takeout_state,
                &self.source_job_state,
                &self.analysis_state,
                self.llm_scheduler.as_ref(),
            )
            .await
        }
    }
}
```

Add these first tests:

```rust
#[tokio::test]
async fn missing_account_returns_not_found() {
    let pool = pool().await;
    let states = States::new();

    let error = states.check(&pool, 404).await.expect_err("missing account");

    assert_eq!(error.kind, AppErrorKind::NotFound);
    assert_eq!(error.message, "Account 404 not found");
}

#[tokio::test]
async fn existing_account_with_zero_sources_passes() {
    let pool = pool().await;
    insert_account(&pool, 11).await;
    let states = States::new();

    let plan = states.check(&pool, 11).await.expect("preflight");

    assert_eq!(plan.account_id, 11);
    assert!(plan.owned_source_ids.is_empty());
    assert!(plan.blocking_work.is_empty());
}
```

- [ ] **Step 3: Run basic preflight tests and verify RED**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml account_deletion::tests::missing_account_returns_not_found account_deletion::tests::existing_account_with_zero_sources_passes
```

Expected: failure from `unimplemented!("account deletion preflight")`.

- [ ] **Step 4: Implement account existence and owned-source loading**

Implement:

```rust
async fn account_exists(pool: &Pool<Sqlite>, account_id: i64) -> AppResult<bool> {
    sqlx::query_scalar::<_, i64>("SELECT EXISTS(SELECT 1 FROM accounts WHERE id = ?)")
        .bind(account_id)
        .fetch_one(pool)
        .await
        .map(|exists| exists != 0)
        .map_err(AppError::database)
}

async fn load_owned_source_ids(pool: &Pool<Sqlite>, account_id: i64) -> AppResult<Vec<i64>> {
    sqlx::query_scalar::<_, i64>(
        "SELECT id FROM sources WHERE account_id = ? ORDER BY id ASC",
    )
    .bind(account_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::database)
}
```

Start `check_account_deletion` with:

```rust
if !account_exists(pool, account_id).await? {
    return Err(AppError::not_found(format!("Account {account_id} not found")));
}
let owned_source_ids = load_owned_source_ids(pool, account_id).await?;
let blocking_work = Vec::new();
Ok(AccountDeletionPlan {
    account_id,
    owned_source_ids,
    #[cfg(test)]
    blocking_work,
})
```

- [ ] **Step 5: Add failing tests for source locks, Takeout, and source jobs**

Add tests:

```rust
#[tokio::test]
async fn source_ingest_lock_on_owned_source_blocks_without_deleting_rows() {
    let pool = pool().await;
    insert_account(&pool, 11).await;
    insert_source(&pool, 7, Some(11)).await;
    let states = States::new();
    let _guard = states
        .source_locks
        .try_acquire(7, SourceIngestKind::Sync)
        .await
        .expect("lock source");

    let error = states.check(&pool, 11).await.expect_err("blocked");

    assert_eq!(error.kind, AppErrorKind::Conflict);
    assert_eq!(error.message, ACCOUNT_DELETE_ACTIVE_WORK_CONFLICT_MESSAGE);
    let account_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM accounts WHERE id = 11")
        .fetch_one(&pool)
        .await
        .expect("count account");
    let source_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sources WHERE id = 7")
        .fetch_one(&pool)
        .await
        .expect("count source");
    assert_eq!(account_count, 1);
    assert_eq!(source_count, 1);
}

#[tokio::test]
async fn active_takeout_job_on_owned_source_blocks() {
    let pool = pool().await;
    insert_account(&pool, 11).await;
    insert_source(&pool, 7, Some(11)).await;
    let states = States::new();
    states
        .takeout_state
        .create_job(7, 11, 100)
        .await
        .expect("takeout job");

    let error = states.check(&pool, 11).await.expect_err("blocked");

    assert_eq!(error.kind, AppErrorKind::Conflict);
}

#[tokio::test]
async fn active_source_job_on_owned_source_blocks_but_unowned_job_does_not() {
    let pool = pool().await;
    insert_account(&pool, 11).await;
    insert_source(&pool, 7, Some(11)).await;
    insert_source(&pool, 8, None).await;
    let states = States::new();
    let options = YoutubeSyncOptions {
        metadata: true,
        transcripts: false,
        comments: false,
    };
    states
        .source_job_state
        .create_job(8, SourceJobType::YoutubeVideoFullSync, None, options.clone())
        .await
        .expect("unowned job");
    states
        .check(&pool, 11)
        .await
        .expect("unowned job ignored");

    states
        .source_job_state
        .create_job(7, SourceJobType::YoutubeVideoFullSync, None, options)
        .await
        .expect("owned job");

    let error = states.check(&pool, 11).await.expect_err("blocked");

    assert_eq!(error.kind, AppErrorKind::Conflict);
}
```

- [ ] **Step 6: Implement source-lock, Takeout, and source-job blocker collection**

In `check_account_deletion`, collect:

```rust
let mut blocking_work = Vec::new();
for (source_id, _kind) in source_locks
    .active_kinds_for_sources(&owned_source_ids)
    .await?
{
    blocking_work.push(AccountDeletionBlocker::SourceIngest { source_id });
}
for job in takeout_state.active_jobs_for_sources(&owned_source_ids).await {
    blocking_work.push(AccountDeletionBlocker::TakeoutImport {
        source_id: job.source_id,
        job_id: job.job_id,
    });
}
for job in source_job_state.active_jobs_for_sources(&owned_source_ids).await {
    blocking_work.push(AccountDeletionBlocker::SourceJob {
        source_id: job.source_id,
        related_source_id: job.related_source_id,
        job_id: job.job_id,
    });
}
```

Return conflict if blockers are non-empty:

```rust
if !blocking_work.is_empty() {
    return Err(AppError::conflict(ACCOUNT_DELETE_ACTIVE_WORK_CONFLICT_MESSAGE));
}
```

Keep `blocking_work` on the plan only for successful tests. Factor the blocker collection behind this private helper so later steps can extend it:

```rust
async fn collect_account_deletion_blockers(
    owned_source_ids: &[i64],
    source_locks: &SourceIngestLocks,
    takeout_state: &TakeoutImportState,
    source_job_state: &SourceJobState,
    _pool: &Pool<Sqlite>,
    _analysis_state: &AnalysisState,
    _llm_scheduler: &LlmSchedulerState,
) -> AppResult<Vec<AccountDeletionBlocker>> {
    let mut blocking_work = Vec::new();
    for (source_id, _kind) in source_locks
        .active_kinds_for_sources(owned_source_ids)
        .await?
    {
        blocking_work.push(AccountDeletionBlocker::SourceIngest { source_id });
    }
    for job in takeout_state.active_jobs_for_sources(owned_source_ids).await {
        blocking_work.push(AccountDeletionBlocker::TakeoutImport {
            source_id: job.source_id,
            job_id: job.job_id,
        });
    }
    for job in source_job_state.active_jobs_for_sources(owned_source_ids).await {
        blocking_work.push(AccountDeletionBlocker::SourceJob {
            source_id: job.source_id,
            related_source_id: job.related_source_id,
            job_id: job.job_id,
        });
    }
    Ok(blocking_work)
}
```

Then `check_account_deletion` calls this helper and maps a non-empty vector to the generic conflict.

- [ ] **Step 7: Add failing tests for direct and group analysis blockers**

Add tests:

```rust
async fn insert_run(
    pool: &sqlx::SqlitePool,
    run_id: i64,
    status: &str,
    source_id: Option<i64>,
    source_group_id: Option<i64>,
) {
    sqlx::query(
        "INSERT INTO analysis_runs (id, status, source_id, source_group_id, created_at)
         VALUES (?, ?, ?, ?, 1)",
    )
    .bind(run_id)
    .bind(status)
    .bind(source_id)
    .bind(source_group_id)
    .execute(pool)
    .await
    .expect("insert run");
}

async fn insert_group_member(pool: &sqlx::SqlitePool, group_id: i64, source_id: i64) {
    sqlx::query(
        "INSERT OR IGNORE INTO analysis_source_groups (id, name, source_type, created_at, updated_at)
         VALUES (?, 'Group', 'telegram', 1, 1)",
    )
    .bind(group_id)
    .execute(pool)
    .await
    .expect("insert group");
    sqlx::query(
        "INSERT INTO analysis_source_group_members (group_id, source_id, created_at)
         VALUES (?, ?, 1)",
    )
    .bind(group_id)
    .bind(source_id)
    .execute(pool)
    .await
    .expect("insert group member");
}

#[tokio::test]
async fn active_direct_source_analysis_run_blocks_owned_source_only() {
    let pool = pool().await;
    insert_account(&pool, 11).await;
    insert_source(&pool, 7, Some(11)).await;
    insert_source(&pool, 8, None).await;
    insert_run(&pool, 70, "running", Some(8), None).await;
    let states = States::new();
    states.analysis_state.insert_active_report_run(70).await;
    states.check(&pool, 11).await.expect("unowned active run ignored");

    insert_run(&pool, 71, "running", Some(7), None).await;
    states.analysis_state.insert_active_report_run(71).await;

    let error = states.check(&pool, 11).await.expect_err("blocked");

    assert_eq!(error.kind, AppErrorKind::Conflict);
}

#[tokio::test]
async fn active_group_analysis_run_blocks_when_any_member_source_is_owned() {
    let pool = pool().await;
    insert_account(&pool, 11).await;
    insert_source(&pool, 7, Some(11)).await;
    insert_source(&pool, 8, None).await;
    insert_group_member(&pool, 90, 8).await;
    insert_run(&pool, 90, "running", None, Some(90)).await;
    let states = States::new();
    states.analysis_state.insert_active_report_run(90).await;
    states.check(&pool, 11).await.expect("unowned group ignored");

    insert_group_member(&pool, 90, 7).await;

    let error = states.check(&pool, 11).await.expect_err("blocked");

    assert_eq!(error.kind, AppErrorKind::Conflict);
}
```

Make `AnalysisState::insert_active_report_run`, `remove_active_report_run`, and `active_report_run_ids` `pub(crate)` in `src-tauri/src/analysis/mod.rs`.

- [ ] **Step 8: Implement analysis blocker resolution**

Add helpers:

```rust
async fn run_ids_depending_on_sources(
    pool: &Pool<Sqlite>,
    candidate_run_ids: &HashSet<i64>,
    owned_source_ids: &[i64],
) -> AppResult<BTreeSet<i64>> {
    if candidate_run_ids.is_empty() || owned_source_ids.is_empty() {
        return Ok(BTreeSet::new());
    }
    let owned = owned_source_ids.iter().copied().collect::<HashSet<_>>();
    let mut blocked = BTreeSet::new();
    let rows = sqlx::query_as::<_, AnalysisRunScopeRow>(
        "SELECT id, source_id, source_group_id FROM analysis_runs ORDER BY id ASC",
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::database)?;
    for row in rows {
        if !candidate_run_ids.contains(&row.id) {
            continue;
        }
        if row.source_id.is_some_and(|source_id| owned.contains(&source_id)) {
            blocked.insert(row.id);
            continue;
        }
        if let Some(group_id) = row.source_group_id {
            if group_has_owned_source(pool, group_id, &owned).await? {
                blocked.insert(row.id);
            }
        }
    }
    Ok(blocked)
}

#[derive(sqlx::FromRow)]
struct AnalysisRunScopeRow {
    id: i64,
    source_id: Option<i64>,
    source_group_id: Option<i64>,
}
```

Use `analysis_state.active_report_run_ids().await` as one candidate set and push:

```rust
for run_id in run_ids_depending_on_sources(pool, &active_run_ids, &owned_source_ids).await? {
    blocking_work.push(AccountDeletionBlocker::AnalysisRun { run_id });
}
```

- [ ] **Step 9: Add failing tests for LLM owner-run blockers**

Add these imports to the `account_deletion.rs` test module:

```rust
use std::sync::Arc;
use tokio::sync::Notify;
use tokio::time::{timeout, Duration};
use crate::error::AppError;
use crate::llm::{LlmRequestKind, LlmRequestMetadata, LlmRequestPriority};
```

Add this helper to the `account_deletion.rs` test module:

```rust
async fn start_scheduler_request(
    scheduler: Arc<LlmSchedulerState>,
    request_id: &str,
    owner_run_id: Option<i64>,
) -> Arc<Notify> {
    let release = Arc::new(Notify::new());
    let release_for_task = Arc::clone(&release);
    let request_id = request_id.to_string();
    let request_id_for_wait = request_id.clone();
    let scheduler_for_task = Arc::clone(&scheduler);
    tokio::spawn(async move {
        let _ = scheduler_for_task
            .run_request(
                LlmRequestMetadata {
                    request_id,
                    profile_id: "default".to_string(),
                    provider: "gemini".to_string(),
                    kind: if owner_run_id.is_some() {
                        LlmRequestKind::AnalysisChat
                    } else {
                        LlmRequestKind::ProviderTest
                    },
                    priority: LlmRequestPriority::Interactive,
                    owner_run_id,
                },
                |_| {},
                move |control| async move {
                    control
                        .run_cancellable(async move {
                            release_for_task.notified().await;
                            Ok::<_, AppError>("done")
                        })
                        .await
                },
            )
            .await;
    });

    timeout(Duration::from_secs(1), async {
        loop {
            let snapshots = scheduler.request_snapshots().await;
            if snapshots
                .iter()
                .any(|snapshot| snapshot.request_id == request_id_for_wait)
            {
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("scheduler request registered");

    release
}
```

Then add this test:

```rust
#[tokio::test]
async fn active_llm_request_for_completed_owned_run_blocks_but_provider_test_does_not() {
    let pool = pool().await;
    insert_account(&pool, 11).await;
    insert_source(&pool, 7, Some(11)).await;
    insert_run(&pool, 77, "completed", Some(7), None).await;
    let states = States::new();

    let provider_release =
        start_scheduler_request(Arc::clone(&states.llm_scheduler), "provider-test", None).await;
    states.check(&pool, 11).await.expect("provider test ignored");

    let chat_release =
        start_scheduler_request(Arc::clone(&states.llm_scheduler), "chat-77", Some(77)).await;
    let error = states.check(&pool, 11).await.expect_err("blocked");

    assert_eq!(error.kind, AppErrorKind::Conflict);
    provider_release.notify_waiters();
    chat_release.notify_waiters();
}
```

- [ ] **Step 10: Implement LLM owner-run blocker resolution and multiple-blocker internal test**

Use `llm_scheduler.active_owner_run_ids().await` as candidate run ids, resolve through `run_ids_depending_on_sources`, and push:

```rust
for run_id in run_ids_depending_on_sources(pool, &llm_owner_run_ids, &owned_source_ids).await? {
    blocking_work.push(AccountDeletionBlocker::LlmRequest { run_id });
}
```

Extend the existing `collect_account_deletion_blockers` helper so the final signature is unchanged:

```rust
async fn collect_account_deletion_blockers(
    owned_source_ids: &[i64],
    source_locks: &SourceIngestLocks,
    takeout_state: &TakeoutImportState,
    source_job_state: &SourceJobState,
    pool: &Pool<Sqlite>,
    analysis_state: &AnalysisState,
    llm_scheduler: &LlmSchedulerState,
) -> AppResult<Vec<AccountDeletionBlocker>>
```

Add this internal diagnostic test:

```rust
#[tokio::test]
async fn blocker_collection_keeps_multiple_categories_for_internal_diagnostics() {
    let pool = pool().await;
    insert_account(&pool, 11).await;
    insert_source(&pool, 7, Some(11)).await;
    insert_run(&pool, 55, "running", Some(7), None).await;
    let states = States::new();
    let _guard = states
        .source_locks
        .try_acquire(7, SourceIngestKind::Sync)
        .await
        .expect("source lock");
    states.analysis_state.insert_active_report_run(55).await;

    let blockers = collect_account_deletion_blockers(
        &[7],
        &states.source_locks,
        &states.takeout_state,
        &states.source_job_state,
        &pool,
        &states.analysis_state,
        states.llm_scheduler.as_ref(),
    )
    .await
    .expect("blockers");

    assert!(blockers.contains(&AccountDeletionBlocker::SourceIngest { source_id: 7 }));
    assert!(blockers.contains(&AccountDeletionBlocker::AnalysisRun { run_id: 55 }));
}
```

- [ ] **Step 11: Run preflight tests and commit**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml account_deletion
```

Expected: all account deletion preflight tests pass.

Update this task's checkboxes to `[x]`, then run:

```powershell
git add src-tauri/src/account_deletion.rs src-tauri/src/lib.rs src-tauri/src/analysis/mod.rs docs/superpowers/plans/2026-05-22-account-deletion-coordination.md
git commit -m "feat: add account deletion active-work preflight"
```

## Task 4: Wire Preflight Into delete_account

**Files:**
- Modify: `src-tauri/src/accounts.rs`
- Modify: `docs/superpowers/plans/2026-05-22-account-deletion-coordination.md`

- [ ] **Step 1: Write failing tests for missing-account and cleanup ordering**

In `src-tauri/src/accounts.rs`, add tests:

```rust
#[tokio::test]
async fn deleting_missing_account_returns_not_found() {
    let pool = memory_pool().await;
    let (_store, secret_store) = memory_secret_store();

    let error = delete_account_from_pool(&pool, &secret_store, 404)
        .await
        .expect_err("missing account");

    assert_eq!(error.kind, AppErrorKind::NotFound);
    assert_eq!(error.message, "Account 404 not found");
}

#[tokio::test]
async fn secret_cleanup_failure_keeps_deleted_database_row_deleted() {
    let pool = memory_pool().await;
    let (store, secret_store) = memory_secret_store();
    let account = create_account_in_pool(
        &pool,
        &secret_store,
        "Personal".to_string(),
        12345,
        "api-hash".to_string(),
        1000,
    )
    .await
    .expect("create account");
    store.fail_delete("secret delete failed");

    let error = delete_account_from_pool(&pool, &secret_store, account.id)
        .await
        .expect_err("secret cleanup fails");

    assert_eq!(error.kind, AppErrorKind::Internal);
    assert_eq!(account_count(&pool).await, 0);
}
```

- [ ] **Step 2: Run account tests and verify RED**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml accounts::tests::deleting_missing_account_returns_not_found accounts::tests::secret_cleanup_failure_keeps_deleted_database_row_deleted
```

Expected: at least missing-account test fails because row deletion currently does not check `rows_affected`.

- [ ] **Step 3: Implement robust account row deletion**

Change `delete_account_row_from_pool`:

```rust
async fn delete_account_row_from_pool(pool: &Pool<Sqlite>, account_id: i64) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM accounts WHERE id = ?")
        .bind(account_id)
        .execute(pool)
        .await
        .map_err(AppError::database)?;

    if result.rows_affected() == 0 {
        return Err(AppError::not_found(format!("Account {account_id} not found")));
    }

    Ok(())
}
```

- [ ] **Step 4: Wire command preflight dependencies**

Update imports in `accounts.rs`:

```rust
use crate::account_deletion::check_account_deletion;
use crate::analysis::AnalysisState;
use crate::llm::LlmSchedulerState;
use crate::source_ingest::SourceIngestLocks;
use crate::takeout_import::TakeoutImportState;
use crate::youtube::jobs::SourceJobState;
```

Update the command signature:

```rust
pub async fn delete_account(
    handle: AppHandle,
    state: tauri::State<'_, TelegramState>,
    source_locks: tauri::State<'_, SourceIngestLocks>,
    takeout_state: tauri::State<'_, TakeoutImportState>,
    source_job_state: tauri::State<'_, SourceJobState>,
    analysis_state: tauri::State<'_, AnalysisState>,
    llm_scheduler: tauri::State<'_, LlmSchedulerState>,
    secret_store: tauri::State<'_, SecretStoreState>,
    account_id: i64,
) -> AppResult<()> {
    let pool = get_pool(&handle).await?;
    check_account_deletion(
        &pool,
        account_id,
        source_locks.inner(),
        takeout_state.inner(),
        source_job_state.inner(),
        analysis_state.inner(),
        llm_scheduler.inner(),
    )
    .await?;
    delete_account_row_from_pool(&pool, account_id).await?;
    let runtime_result =
        clear_account_runtime(&handle, &state, &secret_store, account_id, true).await;
    let api_hash_result = secret_store
        .delete_secret(telegram_account_api_hash_secret(account_id))
        .await;

    runtime_result?;
    api_hash_result
}
```

- [ ] **Step 5: Run account tests and build check**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml accounts::tests
cargo test --manifest-path src-tauri/Cargo.toml account_deletion
```

Expected: account tests and preflight tests pass.

- [ ] **Step 6: Commit delete_account wiring**

Update this task's checkboxes to `[x]`, then run:

```powershell
git add src-tauri/src/accounts.rs docs/superpowers/plans/2026-05-22-account-deletion-coordination.md
git commit -m "feat: guard account deletion with active-work preflight"
```

## Task 5: Attach Analysis Chat LLM Requests To Run Owners

**Files:**
- Modify: `src-tauri/src/analysis/chat.rs`
- Modify: `docs/superpowers/plans/2026-05-22-account-deletion-coordination.md`

- [ ] **Step 1: Write failing unit test for analysis chat request ownership**

In `src-tauri/src/analysis/chat.rs`, extract a helper in tests first by writing:

```rust
#[test]
fn analysis_chat_request_metadata_uses_run_owner() {
    let request = build_chat_request(ChatRequestParams {
        run: &sample_run(),
        profile_id: "default".to_string(),
        scope_label: "Source",
        history: &[],
        question: "What changed?",
        report_markdown: "Saved report",
        context_messages: &[],
        model_override: None,
    });
    let metadata = analysis_chat_request_metadata(
        &request,
        "default".to_string(),
        "gemini".to_string(),
        42,
    );

    assert_eq!(metadata.kind, LlmRequestKind::AnalysisChat);
    assert_eq!(metadata.owner_run_id, Some(42));
}
```

- [ ] **Step 2: Run chat test and verify RED**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::chat::tests::analysis_chat_request_metadata_uses_run_owner
```

Expected: compile failure because `analysis_chat_request_metadata` does not exist.

- [ ] **Step 3: Implement metadata helper and use it in follow-up chat**

Add near chat request code:

```rust
fn analysis_chat_request_metadata(
    request: &crate::llm::LlmChatRequest,
    profile_id: String,
    provider: String,
    run_id: i64,
) -> LlmRequestMetadata {
    LlmRequestMetadata {
        request_id: request.request_id.clone(),
        profile_id,
        provider,
        kind: LlmRequestKind::AnalysisChat,
        priority: LlmRequestPriority::Interactive,
        owner_run_id: Some(run_id),
    }
}
```

Replace the inline metadata construction in `ask_analysis_run_question` with:

```rust
let request_meta = analysis_chat_request_metadata(
    &request,
    resolved_profile.profile_id.clone(),
    resolved_profile.provider.as_str().to_string(),
    run_id,
);
```

- [ ] **Step 4: Run chat tests and commit**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::chat::tests
```

Expected: chat tests pass.

Update this task's checkboxes to `[x]`, then run:

```powershell
git add src-tauri/src/analysis/chat.rs docs/superpowers/plans/2026-05-22-account-deletion-coordination.md
git commit -m "fix: attach analysis chat llm requests to runs"
```

## Task 6: Full Verification And Backlog Update

**Files:**
- Modify: `docs/backlog.md`
- Modify: `docs/superpowers/plans/2026-05-22-account-deletion-coordination.md`

- [ ] **Step 1: Update backlog**

In `docs/backlog.md`, remove completed Account Deletion Coordination bullets:

- reject or cancel account deletion when any owned source has active sync, Takeout import, or delete work;
- decide whether account deletion should cancel owned analysis/LLM work or block until it finishes;
- return `not_found` when deleting a missing account;
- add backend tests for missing-account deletion and account deletion with active source work.

No remaining account deletion coordination backlog item is expected. If verification reveals a new risk, add one concrete sanitized follow-up that names the remaining behavior.

- [ ] **Step 2: Run targeted backend tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml account_deletion
cargo test --manifest-path src-tauri/Cargo.toml accounts::tests
cargo test --manifest-path src-tauri/Cargo.toml source_ingest::tests
cargo test --manifest-path src-tauri/Cargo.toml takeout_import::state::tests
cargo test --manifest-path src-tauri/Cargo.toml youtube::jobs::tests::active_jobs_for_sources_matches_source_and_related_source
cargo test --manifest-path src-tauri/Cargo.toml llm::scheduler::tests::active_owner_run_ids_reports_running_and_queued_owned_requests
cargo test --manifest-path src-tauri/Cargo.toml analysis::chat::tests
```

Expected: all targeted Rust tests pass.

- [ ] **Step 3: Run full project verification**

Run:

```powershell
git diff --check
npm.cmd test
cargo test --manifest-path src-tauri/Cargo.toml
git status --short --branch
```

Expected:

- no whitespace errors;
- frontend suite passes;
- full Rust suite passes;
- tracked changes are limited to the planned files.

- [ ] **Step 4: Commit final implementation**

Update this task's checkboxes to `[x]`, then run:

```powershell
git add docs/backlog.md docs/superpowers/plans/2026-05-22-account-deletion-coordination.md
git commit -m "docs: close account deletion coordination"
```

If all code changes were already committed in earlier tasks, this final commit may be docs-only.

## Task 7: Branch Completion

**Files:**
- Read: git state

- [ ] **Step 1: Verify final branch state**

Run:

```powershell
git status --short --branch
git log -5 --oneline
```

Expected: clean branch `account-deletion-coordination` with recent commits for spec, plan, active-work helpers, preflight, wiring, chat ownership, and docs closure.

- [ ] **Step 2: Present finish options**

Offer:

1. Merge back to `main` locally
2. Push and create a Pull Request
3. Keep the branch as-is
4. Discard this work
