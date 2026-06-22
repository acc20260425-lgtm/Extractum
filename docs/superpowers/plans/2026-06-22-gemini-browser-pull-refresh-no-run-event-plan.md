# Gemini Browser Pull Refresh Without Run Events Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove the Gemini Browser `gemini-browser://run` event transport and make the Settings panel refresh Gemini Browser state through command reads, scheduler modes, and bounded polling.

**Architecture:** Backend keeps Apalis as execution/queue machinery and the file-backed Gemini Browser run log as user-facing run history. Frontend removes Tauri event listening and uses one refresh scheduler with `light` and `full` modes; polling uses cached snapshot reads and run-log reads only, while manual/full refresh may use live status probing. Selected run details are read by id and guarded by selection identity plus `updated_at` version.

**Tech Stack:** Rust/Tauri commands, Apalis-backed Gemini Browser worker, Svelte 5 Settings panel, Vitest source/unit tests, Cargo unit tests.

---

## File Structure

- `src-tauri/src/gemini_browser/run_log.rs`
  - Add `read_run(runs_dir, run_id)` for exact run detail reads from the file-backed run log.
- `src-tauri/src/gemini_browser/state.rs`
  - Add a startup reconciliation gate so snapshot/list/detail pull reads can wait for reconciled startup state.
- `src-tauri/src/gemini_browser/jobs.rs`
  - Expose the startup reconciliation gate helper, update cached status from reconciled run-log state, and remove worker event emission.
- `src-tauri/src/gemini_browser/commands.rs`
  - Add `gemini_bridge_status_snapshot` and `gemini_bridge_get_run`.
  - Remove `GEMINI_BROWSER_RUN_CHANGE_EVENT`, event payload helpers, and settings enqueue event callbacks.
- `src-tauri/src/gemini_browser/types.rs`
  - Remove `GeminiBrowserRunChangeEvent`.
- `src-tauri/src/gemini_browser/mod.rs`
  - Re-export new commands and remove event type export.
- `src-tauri/src/lib.rs`
  - Register new Tauri commands and remove no-longer-used imports.
- `src/lib/types/gemini-browser.ts`
  - Remove `GeminiBrowserRunChangeEvent`.
- `src/lib/api/gemini-browser.ts`
  - Remove Tauri event imports/exports and add `geminiBridgeStatusSnapshot` / `geminiBridgeGetRun`.
- `src/lib/gemini-browser-refresh-scheduler.ts`
  - Add `light` / `full` mode scheduling, strongest-mode coalescing, active refresh dominance, selected-detail loading, stale/version guards, and disposal.
- `src/lib/gemini-browser-polling.ts`
  - New small polling controller for idle/active cadence, polling in-flight guard, persistent failure backoff, and explicit active hints.
- `src/lib/components/settings/gemini-browser-provider-panel.svelte`
  - Remove event listener, wire scheduler modes, start panel-scoped polling, and handle pending test run state.
- `src/lib/api/gemini-browser.test.ts`
  - Update API tests for new command wrappers and absence of event exports.
- `src/lib/gemini-browser-refresh-scheduler.test.ts`
  - Add scheduler mode/detail/version/disposal tests.
- `src/lib/gemini-browser-polling.test.ts`
  - New polling controller tests.
- `src/lib/gemini-browser-provider-panel.test.ts`
  - Update source-contract tests for no event listener and correct polling/scheduler calls.
- `docs/browser-providers-llm-troubleshooting.md`
  - Replace active `gemini-browser://run` troubleshooting language with pull-refresh/polling language.
- `docs/architecture-deep-dive.md`
  - Update Gemini Browser Settings UI freshness model.

---

### Task 0: Branch And Baseline

**Files:**
- Read: `docs/superpowers/specs/2026-06-22-gemini-browser-pull-refresh-no-run-event-design.md`
- Read: `src-tauri/src/gemini_browser/commands.rs`
- Read: `src/lib/components/settings/gemini-browser-provider-panel.svelte`

- [x] **Step 1: Create a feature branch**

Run:

```powershell
git checkout -b gemini-browser-pull-refresh-no-run-event
```

Expected: branch switches to `gemini-browser-pull-refresh-no-run-event`.

- [x] **Step 2: Confirm starting status**

Run:

```powershell
git status --short --branch
```

Expected: current branch is `gemini-browser-pull-refresh-no-run-event`; only the approved spec/plan files may be untracked or modified.

- [x] **Step 3: Run focused baseline checks**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-pull-refresh --lib gemini_browser
```

Expected: current Gemini Browser Rust tests pass before behavior changes.

Run:

```powershell
npm.cmd test -- src/lib/api/gemini-browser.test.ts src/lib/gemini-browser-refresh-scheduler.test.ts src/lib/gemini-browser-provider-panel.test.ts
```

Expected: current frontend Gemini Browser tests pass before behavior changes.

---

### Task 1: Backend Pull Read Models

**Files:**
- Modify: `src-tauri/src/gemini_browser/run_log.rs`
- Modify: `src-tauri/src/gemini_browser/state.rs`
- Modify: `src-tauri/src/gemini_browser/jobs.rs`
- Modify: `src-tauri/src/gemini_browser/commands.rs`
- Modify: `src-tauri/src/gemini_browser/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [x] **Step 1: Add failing run-log detail tests**

In `src-tauri/src/gemini_browser/run_log.rs`, add these tests inside the existing `#[cfg(test)] mod tests`:

```rust
#[test]
fn read_run_returns_exact_run_by_id() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let runs_dir = temp.path();

    create_queued_run(runs_dir, "run-detail", "settings_test", "hello")
        .expect("create queued run");

    let run = read_run(runs_dir, "run-detail").expect("read run");

    assert_eq!(run.run_id, "run-detail");
    assert_eq!(run.status, GeminiBrowserRunStatus::Queued);
}

#[test]
fn read_run_returns_validation_error_for_missing_run() {
    let temp = tempfile::tempdir().expect("create temp dir");

    let error = read_run(temp.path(), "missing-run").expect_err("missing run errors");

    assert_eq!(error.kind, crate::error::AppErrorKind::NotFound);
    assert_eq!(error.message, "Gemini browser run was not found");
}
```

- [x] **Step 2: Run run-log tests and confirm failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-pull-refresh --lib gemini_browser::run_log::tests::read_run -- --nocapture
```

Expected: FAIL because `read_run` is not defined.

- [x] **Step 3: Implement `read_run`**

In `src-tauri/src/gemini_browser/run_log.rs`, add this function after `list_runs`:

```rust
pub(crate) fn read_run(runs_dir: &Path, run_id: &str) -> AppResult<GeminiBrowserRun> {
    prune_expired_runs(runs_dir)?;
    let path = run_file_path(runs_dir, run_id)?;
    if !path.exists() {
        return Err(AppError::not_found("Gemini browser run was not found"));
    }
    read_run_file(&path)
}
```

- [x] **Step 4: Re-run run-log tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-pull-refresh --lib gemini_browser::run_log::tests::read_run -- --nocapture
```

Expected: PASS.

- [x] **Step 5: Add startup reconciliation gate to state**

In `src-tauri/src/gemini_browser/state.rs`, add imports near the top:

```rust
use std::future::Future;
use tokio::sync::{Mutex, MutexGuard, OnceCell};
```

Replace the existing `use tokio::sync::{Mutex, MutexGuard};` line with the line above.

Add a field to `GeminiBrowserState`:

```rust
startup_reconciliation: OnceCell<()>,
```

Add this method inside `impl GeminiBrowserState`:

```rust
pub(crate) async fn ensure_startup_reconciled<F, Fut>(&self, reconcile: F) -> crate::error::AppResult<()>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = crate::error::AppResult<()>>,
{
    self.startup_reconciliation
        .get_or_try_init(|| async { reconcile().await })
        .await
        .map(|_| ())
}
```

Add this conditional snapshot write helper inside `impl GeminiBrowserState`:

```rust
pub(crate) fn set_status_snapshot_if_current(
    &self,
    expected: &GeminiBrowserProviderStatus,
    next: GeminiBrowserProviderStatus,
) -> bool {
    let mut guard = self.status_snapshot.write();
    if guard.as_ref() == Some(expected) {
        *guard = Some(next);
        true
    } else {
        false
    }
}
```

This helper is used by pull-refresh read commands only. Lifecycle-owned worker updates still use the existing direct/update helpers.

- [x] **Step 6: Add state gate tests**

In `src-tauri/src/gemini_browser/state.rs`, add these tests:

```rust
#[tokio::test]
async fn startup_reconciliation_gate_runs_once_after_success() {
    let state = GeminiBrowserState::new();
    let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));

    for _ in 0..2 {
        let calls = calls.clone();
        state
            .ensure_startup_reconciled(move || async move {
                calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok(())
            })
            .await
            .expect("reconcile succeeds");
    }

    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 1);
}

#[tokio::test]
async fn startup_reconciliation_gate_retries_after_failure() {
    let state = GeminiBrowserState::new();
    let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));

    let calls_first = calls.clone();
    let error = state
        .ensure_startup_reconciled(move || async move {
            calls_first.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Err(crate::error::AppError::internal("fixture failure"))
        })
        .await
        .expect_err("first attempt fails");
    assert!(error.to_string().contains("fixture failure"));

    let calls_second = calls.clone();
    state
        .ensure_startup_reconciled(move || async move {
            calls_second.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        })
        .await
        .expect("second attempt succeeds");

    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 2);
}

#[test]
fn set_status_snapshot_if_current_does_not_overwrite_newer_snapshot() {
    let state = GeminiBrowserState::new();
    let expected = GeminiBrowserState::not_started_status("profile-dir".to_string());
    let newer = GeminiBrowserProviderStatus {
        latest_message: Some("worker update".to_string()),
        ..expected.clone()
    };
    let stale_reconciled = GeminiBrowserProviderStatus {
        latest_message: Some("stale pull read".to_string()),
        ..expected.clone()
    };

    state.set_status_snapshot(expected.clone());
    state.set_status_snapshot(newer.clone());

    assert!(!state.set_status_snapshot_if_current(&expected, stale_reconciled));
    assert_eq!(state.status_snapshot_for_test(), Some(newer));
}
```

`status_snapshot_for_test()` already exists in `GeminiBrowserState` in the current codebase. If it is missing during implementation, add a `#[cfg(test)]` helper that returns `self.status_snapshot.read().clone()`.

- [x] **Step 7: Add backend read-model command tests**

In `src-tauri/src/gemini_browser/commands.rs`, add these tests in the existing `#[cfg(test)] mod tests`:

```rust
#[test]
fn status_snapshot_core_returns_cached_status_without_polling_live_sidecar() {
    let cached = GeminiBrowserProviderStatus {
        status: GeminiBrowserProviderStatusKind::Running,
        manual_action: None,
        active_run_id: Some("run-cached".to_string()),
        queue_depth: 1,
        browser_profile_dir: "profile-dir".to_string(),
        latest_message: Some("Cached".to_string()),
    };

    let returned = provider_status_snapshot_core(|| Ok(cached.clone()))
        .expect("snapshot succeeds");

    assert_eq!(returned, cached);
}

#[test]
fn provider_status_snapshot_from_reconciled_runs_does_not_keep_stale_running_snapshot() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let runs_root = temp.path();
    create_queued_run(runs_root, "run-stale", "settings_test", "hello")
        .expect("create queued");
    mark_running(runs_root, "run-stale").expect("mark running");

    let stale = GeminiBrowserProviderStatus {
        status: GeminiBrowserProviderStatusKind::Running,
        manual_action: None,
        active_run_id: Some("run-stale".to_string()),
        queue_depth: 1,
        browser_profile_dir: "profile-dir".to_string(),
        latest_message: Some("stale running".to_string()),
    };

    let reconciled = status_snapshot_from_reconciled_run_log(runs_root, stale, None)
        .expect("derive reconciled snapshot");

    assert_eq!(reconciled.status, GeminiBrowserProviderStatusKind::NotStarted);
    assert_eq!(reconciled.active_run_id, None);
    assert_eq!(reconciled.queue_depth, 0);
}

#[test]
fn provider_status_snapshot_from_reconciled_runs_preserves_live_active_run() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let runs_root = temp.path();
    create_queued_run(runs_root, "run-live", "settings_test", "hello")
        .expect("create queued");
    mark_running(runs_root, "run-live").expect("mark running");

    let stale = GeminiBrowserProviderStatus {
        latest_message: Some("Worker is submitting prompt".to_string()),
        ..GeminiBrowserState::not_started_status("profile-dir".to_string())
    };

    let reconciled = status_snapshot_from_reconciled_run_log(
        runs_root,
        stale,
        Some("run-live".to_string()),
    )
    .expect("derive reconciled snapshot");

    assert_eq!(reconciled.status, GeminiBrowserProviderStatusKind::Running);
    assert_eq!(reconciled.active_run_id.as_deref(), Some("run-live"));
    assert_eq!(
        reconciled.latest_message.as_deref(),
        Some("Worker is submitting prompt")
    );
}

#[test]
fn provider_status_snapshot_from_reconciled_runs_ignores_stale_queued_rows() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let runs_root = temp.path();
    create_queued_run(runs_root, "run-stale-queued", "settings_test", "hello")
        .expect("create queued");
    let mut stale_run = read_run(runs_root, "run-stale-queued").expect("read run");
    stale_run.updated_at = "2026-06-22T00:00:00Z".to_string();
    std::fs::write(
        runs_root.join("run-stale-queued").join("result.json"),
        serde_json::to_string_pretty(&stale_run).expect("serialize run"),
    )
    .expect("write stale run");

    let reconciled = status_snapshot_from_reconciled_run_log_at(
        runs_root,
        GeminiBrowserState::not_started_status("profile-dir".to_string()),
        None,
        time::OffsetDateTime::parse(
            "2026-06-22T00:31:00Z",
            &time::format_description::well_known::Rfc3339,
        )
        .expect("parse time"),
    )
    .expect("derive reconciled snapshot");

    assert_eq!(reconciled.status, GeminiBrowserProviderStatusKind::NotStarted);
    assert_eq!(reconciled.active_run_id, None);
    assert_eq!(reconciled.queue_depth, 0);
}

#[test]
fn provider_status_snapshot_read_core_writes_reconciled_snapshot_back() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let runs_root = temp.path();
    create_queued_run(runs_root, "run-stale", "settings_test", "hello")
        .expect("create queued");
    mark_running(runs_root, "run-stale").expect("mark running");

    let stale = GeminiBrowserProviderStatus {
        status: GeminiBrowserProviderStatusKind::Running,
        manual_action: None,
        active_run_id: Some("run-stale".to_string()),
        queue_depth: 1,
        browser_profile_dir: "profile-dir".to_string(),
        latest_message: Some("stale running".to_string()),
    };
    let mut written = None;

    let returned = provider_status_snapshot_read_core(
        runs_root,
        || Ok(stale),
        None,
        |expected, snapshot| {
            assert_eq!(expected.status, GeminiBrowserProviderStatusKind::Running);
            written = Some(snapshot);
            Ok(true)
        },
    )
    .expect("snapshot read succeeds");

    let written = written.expect("snapshot write-back");
    assert_eq!(returned, written);
    assert_eq!(written.status, GeminiBrowserProviderStatusKind::NotStarted);
    assert_eq!(written.active_run_id, None);
}

#[test]
fn provider_status_snapshot_read_core_skips_stale_write_back_when_snapshot_changed() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let stale = GeminiBrowserProviderStatus {
        status: GeminiBrowserProviderStatusKind::Running,
        manual_action: None,
        active_run_id: Some("run-stale".to_string()),
        queue_depth: 1,
        browser_profile_dir: "profile-dir".to_string(),
        latest_message: Some("stale running".to_string()),
    };
    let newer = GeminiBrowserProviderStatus {
        status: GeminiBrowserProviderStatusKind::Running,
        manual_action: None,
        active_run_id: Some("run-newer".to_string()),
        queue_depth: 0,
        browser_profile_dir: "profile-dir".to_string(),
        latest_message: Some("newer worker snapshot".to_string()),
    };
    let mut attempted = None;
    let mut reads = 0;

    let returned = provider_status_snapshot_read_core(
        temp.path(),
        || {
            reads += 1;
            if reads == 1 {
                Ok(stale.clone())
            } else {
                Ok(newer.clone())
            }
        },
        None,
        |expected, snapshot| {
            attempted = Some((expected.clone(), snapshot.clone()));
            Ok(false)
        },
    )
    .expect("snapshot read succeeds");

    let (_expected, _attempted_snapshot) = attempted.expect("conditional write attempted");
    assert_eq!(returned, newer);
}

#[test]
fn get_run_core_returns_exact_run_from_log() {
    let temp = tempfile::tempdir().expect("create temp dir");
    create_queued_run(temp.path(), "run-detail", "settings_test", "hello")
        .expect("create run");

    let run = get_run_core(temp.path(), "run-detail").expect("get run");

    assert_eq!(run.run_id, "run-detail");
}

#[tokio::test]
async fn provider_status_read_core_waits_for_startup_reconciliation_before_live_status() {
    let order = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let gate_order = order.clone();
    let active_order = order.clone();
    let live_order = order.clone();

    let status = provider_status_read_core(
        || async move {
            gate_order.lock().unwrap().push("gate");
            Ok(())
        },
        || async move {
            active_order.lock().unwrap().push("active_run_id");
            None
        },
        |_active_run_id| async move {
            live_order.lock().unwrap().push("live_status");
            Ok(GeminiBrowserState::not_started_status("profile-dir".to_string()))
        },
        std::time::Duration::from_millis(250),
        || Ok(GeminiBrowserState::not_started_status("fallback-dir".to_string())),
    )
    .await
    .expect("status read succeeds");

    assert_eq!(status.browser_profile_dir, "profile-dir");
    assert_eq!(
        order.lock().unwrap().as_slice(),
        ["gate", "active_run_id", "live_status"]
    );
}
```

- [x] **Step 8: Run backend command tests and confirm failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-pull-refresh --lib gemini_browser::commands::tests -- --nocapture
```

Expected: FAIL because `provider_status_read_core`, `provider_status_snapshot_core`, `provider_status_snapshot_read_core`, `status_snapshot_from_reconciled_run_log`, and `get_run_core` do not exist.

- [x] **Step 9: Implement backend read-model helpers and commands**

In `src-tauri/src/gemini_browser/commands.rs`:

1. Remove `Emitter` from the `tauri` import only after event removal in Task 2. For now keep it.
2. Add `read_run` and `GeminiBrowserProviderStatusKind` to the `super::{...}` imports. Also add:

```rust
use time::{format_description::well_known::Rfc3339, Duration, OffsetDateTime};
```

3. Remove the unused `state: &GeminiBrowserState` parameter from `provider_status_core(...)` and update existing `provider_status_core(...)` tests/call sites to pass only `live_status`, `timeout`, and `fallback_status`. Then add this helper after `provider_status_core`:

```rust
const STATUS_SNAPSHOT_RUN_SCAN_LIMIT: usize = 200;
const STATUS_SNAPSHOT_ACTIVITY_GRACE_MINUTES: i64 = 30;

async fn provider_status_read_core<Gate, GateFut, Active, ActiveFut, Live, LiveFut>(
    ensure_reconciled: Gate,
    active_run_id: Active,
    live_status: Live,
    timeout: std::time::Duration,
    fallback_status: impl FnOnce() -> AppResult<GeminiBrowserProviderStatus>,
) -> AppResult<GeminiBrowserProviderStatus>
where
    Gate: FnOnce() -> GateFut,
    GateFut: std::future::Future<Output = AppResult<()>>,
    Active: FnOnce() -> ActiveFut,
    ActiveFut: std::future::Future<Output = Option<String>>,
    Live: FnOnce(Option<String>) -> LiveFut,
    LiveFut: std::future::Future<Output = AppResult<GeminiBrowserProviderStatus>>,
{
    ensure_reconciled().await?;
    let active_run_id = active_run_id().await;
    provider_status_core(live_status(active_run_id), timeout, fallback_status).await
}

fn provider_status_snapshot_core(
    fallback_status: impl FnOnce() -> AppResult<GeminiBrowserProviderStatus>,
) -> AppResult<GeminiBrowserProviderStatus> {
    fallback_status()
}

fn provider_status_snapshot_read_core(
    runs_root: &std::path::Path,
    mut read_snapshot: impl FnMut() -> AppResult<GeminiBrowserProviderStatus>,
    active_run_id: Option<String>,
    mut write_snapshot_if_current: impl FnMut(
        &GeminiBrowserProviderStatus,
        GeminiBrowserProviderStatus,
    ) -> AppResult<bool>,
) -> AppResult<GeminiBrowserProviderStatus> {
    let snapshot = provider_status_snapshot_core(|| read_snapshot())?;
    let reconciled =
        status_snapshot_from_reconciled_run_log(runs_root, snapshot.clone(), active_run_id)?;
    if write_snapshot_if_current(&snapshot, reconciled.clone())? {
        Ok(reconciled)
    } else {
        read_snapshot()
    }
}

fn status_snapshot_from_reconciled_run_log(
    runs_root: &std::path::Path,
    mut snapshot: GeminiBrowserProviderStatus,
    active_run_id: Option<String>,
) -> AppResult<GeminiBrowserProviderStatus> {
    status_snapshot_from_reconciled_run_log_at(
        runs_root,
        snapshot,
        active_run_id,
        OffsetDateTime::now_utc(),
    )
}

fn status_snapshot_from_reconciled_run_log_at(
    runs_root: &std::path::Path,
    mut snapshot: GeminiBrowserProviderStatus,
    active_run_id: Option<String>,
    now: OffsetDateTime,
) -> AppResult<GeminiBrowserProviderStatus> {
    let runs = list_runs(runs_root, STATUS_SNAPSHOT_RUN_SCAN_LIMIT)?.runs;
    let fresh_queued_count = runs
        .iter()
        .filter(|run| {
            run.status == GeminiBrowserRunStatus::Queued && run_log_activity_is_fresh(run, now)
        })
        .count();
    let stale_queued_count = runs
        .iter()
        .filter(|run| {
            run.status == GeminiBrowserRunStatus::Queued && !run_log_activity_is_fresh(run, now)
        })
        .count();

    if let Some(active_run_id) = active_run_id {
        snapshot.status = GeminiBrowserProviderStatusKind::Running;
        snapshot.active_run_id = Some(active_run_id);
        snapshot.queue_depth = fresh_queued_count;
        if snapshot.latest_message.is_none() {
            snapshot.latest_message = Some("Running".to_string());
        }
        snapshot.manual_action = None;
        return Ok(snapshot);
    }

    if fresh_queued_count > 0 {
        snapshot.status = GeminiBrowserProviderStatusKind::Running;
        snapshot.active_run_id = None;
        snapshot.queue_depth = fresh_queued_count;
        if snapshot.latest_message.is_none() {
            snapshot.latest_message = Some("Queued".to_string());
        }
        snapshot.manual_action = None;
        return Ok(snapshot);
    }

    snapshot.active_run_id = None;
    snapshot.queue_depth = 0;
    if stale_queued_count > 0 && snapshot.status == GeminiBrowserProviderStatusKind::Running {
        snapshot.status = GeminiBrowserProviderStatusKind::NotStarted;
        if snapshot.latest_message.is_none() {
            snapshot.latest_message = Some(
                "Gemini browser has stale queued run-log entries; waiting for cleanup.".to_string(),
            );
        }
        snapshot.manual_action = None;
        return Ok(snapshot);
    }

    if snapshot.status == GeminiBrowserProviderStatusKind::Running {
        if let Some(latest) = runs.first().and_then(|run| run.result.as_ref()) {
            snapshot.status = GeminiBrowserState::provider_status_kind_for_run_status(&latest.status);
            snapshot.latest_message = latest.message.clone();
            snapshot.manual_action = latest.manual_action.clone();
        } else {
            snapshot.status = GeminiBrowserProviderStatusKind::NotStarted;
            snapshot.latest_message = Some("Gemini browser sidecar is not running.".to_string());
            snapshot.manual_action = None;
        }
    }
    Ok(snapshot)
}

fn run_log_activity_is_fresh(run: &GeminiBrowserRun, now: OffsetDateTime) -> bool {
    let Ok(updated_at) = OffsetDateTime::parse(&run.updated_at, &Rfc3339) else {
        return false;
    };
    now - updated_at <= Duration::minutes(STATUS_SNAPSHOT_ACTIVITY_GRACE_MINUTES)
}

fn get_run_core(runs_root: &std::path::Path, run_id: &str) -> AppResult<GeminiBrowserRun> {
    read_run(runs_root, run_id)
}
```

This helper deliberately does not derive `active_run_id` from `Running` run-log rows unless `GeminiBrowserState::active_run_id()` currently confirms the run is live. After startup, unconfirmed running rows are either handled by the live worker path or terminalized by reconciliation. A stale running row must not resurrect cached status as active, but a real active worker should not be hidden by a short stale-cache window.
`gemini_bridge_status_snapshot` is on the active polling path, so it must not do an unbounded run-log scan. Keep `STATUS_SNAPSHOT_RUN_SCAN_LIMIT` bounded and use `gemini_bridge_list_runs` as the authoritative visible history read. The snapshot helper may derive a lightweight/fresh queue signal from recent run-log entries, but stale queued rows older than `STATUS_SNAPSHOT_ACTIVITY_GRACE_MINUTES` must not keep the provider snapshot active.
When the helper corrects activity fields (`status`, `active_run_id`, `queue_depth`), it must preserve an existing `latest_message` unless it is applying a terminal run result or there is no existing message. Pull reads should not overwrite a useful lifecycle/worker message with generic `"Running"` or `"Queued"` text.

4. Route `provider_status(...)` through `provider_status_read_core(...)` so startup reconciliation happens before `active_run_id` or live sidecar probing:

```rust
let browser_profile_dir = path_string(&profile_dir(handle)?);
provider_status_read_core(
    || super::jobs::ensure_gemini_browser_startup_reconciled(handle),
    || state.active_run_id(),
    |active_run_id| sidecar::status(
        handle,
        state,
        browser_profile_dir,
        browser_config,
        active_run_id,
        0,
    ),
    std::time::Duration::from_millis(250),
    || state.status_snapshot(handle),
)
.await
```

5. Add commands after `gemini_bridge_status`:

```rust
#[tauri::command]
pub async fn gemini_bridge_status_snapshot(
    handle: AppHandle,
    state: State<'_, GeminiBrowserState>,
) -> AppResult<GeminiBrowserProviderStatus> {
    super::jobs::ensure_gemini_browser_startup_reconciled(&handle).await?;
    let active_run_id = state.active_run_id().await;
    provider_status_snapshot_read_core(
        &runs_dir(&handle)?,
        || state.status_snapshot(&handle),
        active_run_id,
        |expected, snapshot| Ok(state.set_status_snapshot_if_current(expected, snapshot)),
    )
}
```

The write-back is conditional. Once a pull-refresh read has corrected stale cached `running` state after startup reconciliation, later full/manual reads should not see the old cached snapshot again; however, the read must not overwrite a newer lifecycle-owned worker snapshot written after the fallback snapshot was read. If the conditional write returns `false`, re-read and return the current cached snapshot instead of returning the stale reconciled value to the UI.
`set_status_snapshot_if_current(...) == false` must mean only one thing: the cached snapshot no longer equals the expected snapshot because another writer, usually the lifecycle/worker path, has already written a different snapshot. Any other write-back failure must return `Err`, not `false`, so the caller does not silently lose a reconciled correction for the current refresh.

6. Add command after `gemini_bridge_list_runs`:

```rust
#[tauri::command]
pub async fn gemini_bridge_get_run(handle: AppHandle, run_id: String) -> AppResult<GeminiBrowserRun> {
    super::jobs::ensure_gemini_browser_startup_reconciled(&handle).await?;
    get_run_core(&runs_dir(&handle)?, &run_id)
}
```

7. Update `gemini_bridge_list_runs`:

```rust
#[tauri::command]
pub async fn gemini_bridge_list_runs(
    handle: AppHandle,
    limit: Option<usize>,
) -> AppResult<GeminiBrowserRunLogSummary> {
    super::jobs::ensure_gemini_browser_startup_reconciled(&handle).await?;
    list_runs(&runs_dir(&handle)?, limit.unwrap_or(20))
}
```

- [x] **Step 10: Add startup reconciliation helper**

In `src-tauri/src/gemini_browser/jobs.rs`, add this public helper near `start_gemini_browser_job_worker`:

```rust
pub(crate) async fn ensure_gemini_browser_startup_reconciled(
    handle: &tauri::AppHandle,
) -> crate::error::AppResult<()> {
    let state = handle.state::<crate::gemini_browser::GeminiBrowserState>();
    let handle = handle.clone();
    state
        .ensure_startup_reconciled(move || async move {
            let runs_root = crate::gemini_browser::runs_dir(&handle)?;
            reconcile_gemini_browser_run_log_at_startup(
                &runs_root,
                apalis_queue_inspection_mode(),
                |_run_id| Ok(None),
            )?;
            Ok(())
        })
        .await
}
```

This uses `ApalisQueueInspectionMode::DegradedRunLogOnly` through `apalis_queue_inspection_mode()` today, so the `None` lookup does not claim authoritative Apalis absence. If `apalis_queue_inspection_mode()` later returns `Supported`, this helper must be changed in the same task to query Apalis storage for each run id instead of returning `Ok(None)`.

Then update `start_gemini_browser_job_worker` setup block so it calls the helper instead of directly calling `reconcile_gemini_browser_run_log_at_startup`:

```rust
ensure_gemini_browser_startup_reconciled(&setup_handle).await?;
```

Keep `setup_gemini_browser_apalis_storage(&pool).await?;` before the reconciliation helper.

`ensure_gemini_browser_startup_reconciled` is the single shared startup gate for worker setup and read commands. It must use `GeminiBrowserState::ensure_startup_reconciled(...)` directly and must not call `gemini_bridge_status_snapshot`, `gemini_bridge_list_runs`, `gemini_bridge_get_run`, or any other command/read path internally. That avoids self-waiting on the same gate during worker startup and keeps retry semantics centralized in the state helper after a failed reconciliation attempt.

- [x] **Step 11: Re-export and register commands**

In `src-tauri/src/gemini_browser/mod.rs`, update exports:

```rust
pub use commands::{
    gemini_bridge_get_run, gemini_bridge_list_runs, gemini_bridge_open_browser,
    gemini_bridge_open_run_folder, gemini_bridge_resume, gemini_bridge_send_single,
    gemini_bridge_start_cdp_chrome, gemini_bridge_status, gemini_bridge_status_snapshot,
    gemini_bridge_stop,
};
```

In `src-tauri/src/gemini_browser/mod.rs`, update run-log re-exports:

```rust
pub(crate) use run_log::{
    create_queued_run, finish_run, list_runs, mark_running, read_run, recorded_run_dir,
};
```

In `src-tauri/src/lib.rs`, update imports:

```rust
gemini_bridge_get_run, gemini_bridge_list_runs, gemini_bridge_open_browser,
gemini_bridge_open_run_folder, gemini_bridge_resume, gemini_bridge_send_single,
gemini_bridge_start_cdp_chrome, gemini_bridge_status, gemini_bridge_status_snapshot,
gemini_bridge_stop, start_gemini_browser_job_worker,
```

In the Tauri `generate_handler!` list, add:

```rust
gemini_bridge_status_snapshot,
gemini_bridge_get_run,
```

Place `gemini_bridge_status_snapshot` after `gemini_bridge_status`, and `gemini_bridge_get_run` after `gemini_bridge_list_runs`.

- [x] **Step 12: Run backend read-model tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-pull-refresh --lib gemini_browser::run_log::tests::read_run -- --nocapture
```

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-pull-refresh --lib gemini_browser::state::tests::startup_reconciliation_gate -- --nocapture
```

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-pull-refresh --lib gemini_browser::commands::tests -- --nocapture
```

Expected: PASS.

- [x] **Step 13: Commit backend read-model commands**

Run:

```powershell
git add src-tauri/src/gemini_browser/run_log.rs src-tauri/src/gemini_browser/state.rs src-tauri/src/gemini_browser/jobs.rs src-tauri/src/gemini_browser/commands.rs src-tauri/src/gemini_browser/mod.rs src-tauri/src/lib.rs
git commit -m "feat: add Gemini Browser pull read models"
```

Expected: commit succeeds.

---

### Task 2: Remove Backend Run Event Transport

**Files:**
- Modify: `src-tauri/src/gemini_browser/types.rs`
- Modify: `src-tauri/src/gemini_browser/mod.rs`
- Modify: `src-tauri/src/gemini_browser/commands.rs`
- Modify: `src-tauri/src/gemini_browser/jobs.rs`

- [x] **Step 1: Remove Rust event assertion tests**

In `src-tauri/src/gemini_browser/commands.rs`, remove tests:

- `run_change_event_uses_run_log_updated_at_only`
- `run_change_event_emit_failure_is_best_effort`
- `status_open_and_resume_do_not_emit_run_change_events_directly`

Do not replace them with a Rust source test containing the forbidden strings.
The production-source removal is verified with the scoped `rg` gate in Step 7.

- [x] **Step 2: Remove event type**

In `src-tauri/src/gemini_browser/types.rs`, delete:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GeminiBrowserRunChangeEvent {
    pub run_id: String,
    pub run_updated_at: String,
}
```

In `src-tauri/src/gemini_browser/mod.rs`, remove `GeminiBrowserRunChangeEvent` from `pub use types::{...}`.

- [x] **Step 3: Remove command event helpers**

In `src-tauri/src/gemini_browser/commands.rs`:

1. Change the top import from:

```rust
use tauri::{AppHandle, Emitter, Manager, State};
```

to:

```rust
use tauri::{AppHandle, Manager, State};
```

2. Remove `GeminiBrowserRunChangeEvent` from the `super::{...}` import list.
3. Remove `QueuedGeminiBrowserJob` from the `super::jobs::{...}` import list if it is no longer used after the enqueue helper signature change.
4. Delete:

```rust
pub const GEMINI_BROWSER_RUN_CHANGE_EVENT: &str = "gemini-browser://run";
pub(crate) fn run_change_event_from_run(...)
pub(crate) fn emit_run_change_event_core(...)
fn emit_run_change_event(...)
```

5. Change `SendSinglePromptEnqueueHandoff` to:

```rust
#[derive(Debug)]
struct SendSinglePromptEnqueueHandoff {
    waiter: GeminiBrowserWaiterReceiver,
}
```

6. Add an internal enqueue error type near `SendSinglePromptEnqueueHandoff`:

```rust
#[derive(Debug)]
enum SendSinglePromptEnqueueError {
    App(AppError),
    EnqueueFailed {
        run_id: String,
        source: AppError,
        failed_result: GeminiBrowserRunResult,
    },
}

impl From<AppError> for SendSinglePromptEnqueueError {
    fn from(error: AppError) -> Self {
        Self::App(error)
    }
}
```

Keep this enum private to `commands.rs`, but keep `#[derive(Debug)]`; command tests may use `expect_err` or debug output while exercising enqueue failure paths.

7. Change `send_single_prompt_enqueue_core` signature from:

```rust
async fn send_single_prompt_enqueue_core<Enqueue, EnqueueFut, EmitEvent>(
    ...
    enqueue: Enqueue,
    mut emit_event: EmitEvent,
) -> AppResult<SendSinglePromptEnqueueHandoff>
where
    Enqueue: FnOnce(GeminiBrowserJob) -> EnqueueFut,
    EnqueueFut: std::future::Future<Output = AppResult<QueuedGeminiBrowserJob>>,
    EmitEvent: FnMut(&GeminiBrowserRun),
```

to:

```rust
async fn send_single_prompt_enqueue_core<Enqueue, EnqueueFut>(
    runs_root: &std::path::Path,
    runtime: &GeminiBrowserJobRuntime,
    request: GeminiBrowserRunRequest,
    browser_config: Option<GeminiBrowserProviderConfig>,
    enqueue: Enqueue,
) -> Result<SendSinglePromptEnqueueHandoff, SendSinglePromptEnqueueError>
where
    Enqueue: FnOnce(GeminiBrowserJob) -> EnqueueFut,
    EnqueueFut: std::future::Future<Output = AppResult<QueuedGeminiBrowserJob>>,
```

8. In the enqueue failure branch, keep `runtime.remove_waiter(&request.run_id);` and `finish_run(...)`, but remove `emit_event(&failed_run);`. Return the failed terminal result with the enqueue error:

```rust
let _failed_run = finish_run(runs_root, &request.run_id, failed.clone())?;
return Err(SendSinglePromptEnqueueError::EnqueueFailed {
    run_id: request.run_id.clone(),
    source: error,
    failed_result: failed,
});
```

9. Remove:

```rust
let queued_event = run_change_event_from_run(&queued_run);
emit_event(&queued_run);
```

and replace it with:

```rust
let _queued_run = queued_run;
```

10. Return:

```rust
Ok(SendSinglePromptEnqueueHandoff { waiter })
```

11. In `send_single_prompt`, remove the event callback argument from `send_single_prompt_enqueue_core(...)`. Handle the structured enqueue failure immediately after `.await`:

```rust
let handoff = match send_single_prompt_enqueue_core(...).await {
    Ok(handoff) => handoff,
    Err(SendSinglePromptEnqueueError::EnqueueFailed {
        run_id,
        source,
        failed_result,
    }) => {
        debug_assert_eq!(failed_result.run_id, run_id);
        if let Err(error) = state.update_status_snapshot(handle, |status| {
            status.status = GeminiBrowserState::provider_status_kind_for_run_status(
                &failed_result.status,
            );
            status.active_run_id = None;
            status.queue_depth = 0;
            status.latest_message = failed_result.message.clone();
            status.manual_action = failed_result.manual_action.clone();
        }) {
            eprintln!("Gemini Browser enqueue failure status snapshot update failed: {error}");
        }
        return Err(source);
    }
    Err(SendSinglePromptEnqueueError::App(error)) => return Err(error),
};
```

Add or update a command unit test for the enqueue failure branch so it asserts:

```rust
assert_eq!(failed_result.run_id, request.run_id);
```

If the test observes only the public command result, assert the failed run-log row has `run_id == request.run_id` before checking the status snapshot update.

After the handoff returns, update queued status snapshot directly:

```rust
if let Err(error) = state.update_status_snapshot(handle, |status| {
    status.status = GeminiBrowserProviderStatusKind::Running;
    status.active_run_id = None;
    status.queue_depth = 1;
    status.latest_message = Some("Queued".to_string());
    status.manual_action = None;
}) {
    eprintln!("Gemini Browser queued status snapshot update failed: {error}");
}
```

- [x] **Step 4: Remove worker event callbacks**

In `src-tauri/src/gemini_browser/jobs.rs`:

1. Remove `Emitter` imports if they become unused.
2. Delete `emit_gemini_browser_run_change_event`.
3. In `cancel_gemini_browser_job`, remove the event callback argument:

```rust
cancel_gemini_browser_job_core(
    &runtime,
    &state,
    &runs_root,
    run_id,
    |result| update_terminal_status_snapshot_best_effort(handle, &state, result),
    || stop_active_gemini_browser_sidecar(handle, &state),
)
.await
```

4. Change `cancel_gemini_browser_job_core` signature to remove `EmitEvent` and `mut emit_event`.
5. Remove `emit_event(&cancelled_run);` and `emit_event(&failed_run);`. Keep `finish_run`, waiter completion, snapshot update, and `runtime.clear_cancelled`.
6. In `process_gemini_browser_job`, remove:

```rust
emit_gemini_browser_run_change_event(handle, &running_run);
emit_gemini_browser_run_change_event(handle, &run);
```

Keep the status snapshot updates.

7. In `finish_timed_out_job` and `finish_completed_job`, remove `emit_gemini_browser_run_change_event(...)`.

- [x] **Step 5: Update Rust tests that captured event vectors**

Search:

```powershell
rg -n "GeminiBrowserRunChangeEvent|run_change_event_from_run|emit_event|events =" src-tauri\src\gemini_browser
```

For command tests:

- Replace event vector assertions with run-log assertions using `list_runs(temp.path(), 10)`.
- Keep waiter cleanup assertions such as `!runtime.has_waiter_for_test(...)`.

For job tests:

- Replace expected event sequences with run-log/status snapshot assertions.
- Keep existing `test_events` vectors used for worker handler progress if they are not Tauri run-change events. Do not remove worker test instrumentation like `"run-timeout-first:running"` if it is local test handler state.

Example replacement assertion:

```rust
let run = list_runs(temp.path(), 10)
    .expect("list runs")
    .runs
    .into_iter()
    .find(|run| run.run_id == "run-1")
    .expect("run exists");
assert_eq!(run.status, GeminiBrowserRunStatus::Failed);
```

- [x] **Step 6: Run backend event-removal tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-pull-refresh --lib gemini_browser -- --nocapture
```

Expected: PASS.

- [x] **Step 7: Verify no production backend event transport remains**

Run:

```powershell
rg -n "gemini-browser://run|GEMINI_BROWSER_RUN_CHANGE_EVENT|GeminiBrowserRunChangeEvent|run_change_event_from_run|emit_run_change_event" src-tauri\src\gemini_browser
```

Expected: no output.

Also run the backend-wide gate:

```powershell
rg -n "gemini-browser://run|GEMINI_BROWSER_RUN_CHANGE_EVENT|GeminiBrowserRunChangeEvent|run_change_event_from_run|emit_run_change_event" src-tauri\src
```

Expected: no output. This catches stale imports/re-exports in `src-tauri/src/lib.rs` or any non-`gemini_browser` Rust module before frontend work starts.

- [x] **Step 8: Commit backend event removal**

Run:

```powershell
git add src-tauri/src/gemini_browser/types.rs src-tauri/src/gemini_browser/mod.rs src-tauri/src/gemini_browser/commands.rs src-tauri/src/gemini_browser/jobs.rs
git commit -m "refactor: remove Gemini Browser run event backend"
```

Expected: commit succeeds.

---

### Task 3: Frontend API Surface Without Events

**Files:**
- Modify: `src/lib/types/gemini-browser.ts`
- Modify: `src/lib/api/gemini-browser.ts`
- Modify: `src/lib/api/gemini-browser.test.ts`

- [x] **Step 1: Update API tests for pull commands**

In `src/lib/api/gemini-browser.test.ts`:

1. Remove imports:

```ts
GEMINI_BROWSER_RUN_CHANGE_EVENT,
listenToGeminiBrowserRunChanges,
```

2. Add imports:

```ts
  geminiBridgeGetRun,
  geminiBridgeStatusSnapshot,
  isGeminiBrowserRunNotFoundError,
```

3. Remove `listenMock` and the `vi.mock("@tauri-apps/api/event", ...)` block.
4. Update `beforeEach` to only reset `invokeMock`.
5. Replace the `"lists runs and subscribes..."` test with:

```ts
it("wraps pull read-model commands", async () => {
  await geminiBridgeStatusSnapshot();
  expect(invokeMock).toHaveBeenLastCalledWith("gemini_bridge_status_snapshot");

  await geminiBridgeListRuns(5);
  expect(invokeMock).toHaveBeenLastCalledWith("gemini_bridge_list_runs", { limit: 5 });

  await geminiBridgeGetRun("run-1");
  expect(invokeMock).toHaveBeenLastCalledWith("gemini_bridge_get_run", { runId: "run-1" });
});

it("detects typed not-found run detail errors", () => {
  expect(isGeminiBrowserRunNotFoundError({ kind: "not_found", message: "missing" })).toBe(true);
  expect(isGeminiBrowserRunNotFoundError({ kind: "network", message: "not found upstream" })).toBe(false);
  expect(isGeminiBrowserRunNotFoundError(new Error("not found text only"))).toBe(false);
});
```

This contract depends on `src-tauri/src/error.rs`: `AppErrorKind` uses `#[serde(rename_all = "snake_case")]`, so `AppErrorKind::NotFound` is serialized to frontend invoke rejections as `kind: "not_found"`. Do not accept plain string errors here; otherwise transient errors containing the words “not found” could clear selected detail or pending runs incorrectly.

6. Replace the legacy names test with:

```ts
it("does not expose Gemini Browser run event public names", async () => {
  const api = await import("./gemini-browser");
  expect("listenToGeminiBrowserRuns" in api).toBe(false);
  expect("GEMINI_BROWSER_RUN_EVENT" in api).toBe(false);
  expect("listenToGeminiBrowserRunChanges" in api).toBe(false);
  expect("GEMINI_BROWSER_RUN_CHANGE_EVENT" in api).toBe(false);
});
```

- [x] **Step 2: Run API test and confirm failure**

Run:

```powershell
npm.cmd test -- src/lib/api/gemini-browser.test.ts
```

Expected: FAIL because new wrappers do not exist and old event exports still exist.

- [x] **Step 3: Remove event type and add command wrappers**

In `src/lib/types/gemini-browser.ts`, delete:

```ts
export interface GeminiBrowserRunChangeEvent {
  run_id: string;
  run_updated_at: string;
}
```

In `src/lib/api/gemini-browser.ts`:

1. Remove:

```ts
import { listen, type Event, type UnlistenFn } from "@tauri-apps/api/event";
```

2. Remove `GeminiBrowserRunChangeEvent` from type imports.
3. Delete:

```ts
export const GEMINI_BROWSER_RUN_CHANGE_EVENT = "gemini-browser://run";
export function listenToGeminiBrowserRunChanges(...)
```

4. Add:

```ts
export function geminiBridgeStatusSnapshot() {
  return invoke<GeminiBrowserProviderStatus>("gemini_bridge_status_snapshot");
}

export function geminiBridgeGetRun(runId: string) {
  return invoke<GeminiBrowserRun>("gemini_bridge_get_run", { runId });
}

// AppErrorKind::NotFound is serialized by Tauri as { kind: "not_found", message }.
export function isGeminiBrowserRunNotFoundError(error: unknown) {
  return (
    typeof error === "object" &&
    error !== null &&
    "kind" in error &&
    (error as { kind?: unknown }).kind === "not_found"
  );
}
```

5. Add `GeminiBrowserRun` to the type import list.

- [x] **Step 4: Run API tests**

Run:

```powershell
npm.cmd test -- src/lib/api/gemini-browser.test.ts
```

Expected: PASS.

- [x] **Step 5: Commit frontend API removal**

Run:

```powershell
git add src/lib/types/gemini-browser.ts src/lib/api/gemini-browser.ts src/lib/api/gemini-browser.test.ts
git commit -m "refactor: remove Gemini Browser run event API"
```

Expected: commit succeeds.

---

### Task 4: Refresh Scheduler Modes And Detail Guards

**Files:**
- Modify: `src/lib/gemini-browser-refresh-scheduler.ts`
- Modify: `src/lib/gemini-browser-refresh-scheduler.test.ts`

- [x] **Step 1: Replace scheduler tests with mode/detail contract tests**

In `src/lib/gemini-browser-refresh-scheduler.test.ts`, update helper types:

```ts
import type {
  GeminiBrowserProviderStatus,
  GeminiBrowserRun,
  GeminiBrowserRunLogSummary,
} from "./types/gemini-browser";
```

Update `schedulerDeps` to include:

```ts
loadStatus: vi.fn(async () => status({ latest_message: "Live" })),
loadStatusSnapshot: vi.fn(async () => status({ latest_message: "Cached" })),
loadRuns: vi.fn(async () => ({ runs: [run("run-1")] })),
loadRun: vi.fn(async (runId: string) => run(runId)),
getSelectedRunId: vi.fn(() => null),
getSelectedDetailToken: vi.fn(() => 0),
applySelectedRun: vi.fn(),
applySelectedRunUnavailable: vi.fn(),
applySelectedRunError: vi.fn(),
isDisposed: vi.fn(() => false),
isRunNotFoundError: (error: unknown) =>
  typeof error === "object" &&
  error !== null &&
  "kind" in error &&
  (error as { kind?: unknown }).kind === "not_found",
```

Add tests:

```ts
it("light refresh uses cached status and never calls live status", async () => {
  const deps = schedulerDeps();
  await createGeminiBrowserRefreshScheduler(deps).scheduleRefresh({ mode: "light" });

  expect(deps.loadStatusSnapshot).toHaveBeenCalledTimes(1);
  expect(deps.loadStatus).not.toHaveBeenCalled();
  expect(deps.loadRuns).toHaveBeenCalledTimes(1);
});

it("defaults to light refresh for safety", async () => {
  const deps = schedulerDeps();
  await createGeminiBrowserRefreshScheduler(deps).scheduleRefresh();

  expect(deps.loadStatusSnapshot).toHaveBeenCalledTimes(1);
  expect(deps.loadStatus).not.toHaveBeenCalled();
});

it("full refresh uses live status", async () => {
  const deps = schedulerDeps();
  await createGeminiBrowserRefreshScheduler(deps).scheduleRefresh({ mode: "full" });

  expect(deps.loadStatus).toHaveBeenCalledTimes(1);
  expect(deps.loadStatusSnapshot).not.toHaveBeenCalled();
  expect(deps.loadRuns).toHaveBeenCalledTimes(1);
});

it("does not downgrade pending full refresh behind an active light refresh", async () => {
  const firstStatus = deferred<GeminiBrowserProviderStatus>();
  const firstRuns = deferred<GeminiBrowserRunLogSummary>();
  const deps = schedulerDeps({
    loadStatusSnapshot: vi.fn().mockReturnValueOnce(firstStatus.promise).mockResolvedValue(status()),
    loadRuns: vi.fn().mockReturnValueOnce(firstRuns.promise).mockResolvedValue({ runs: [] }),
  });
  const scheduler = createGeminiBrowserRefreshScheduler(deps);

  const active = scheduler.scheduleRefresh({ mode: "light" });
  const trailing = scheduler.scheduleRefresh({ mode: "full" });

  firstStatus.resolve(status());
  firstRuns.resolve({ runs: [] });
  await active;
  await trailing;

  expect(deps.loadStatus).toHaveBeenCalledTimes(1);
});

it("light request attaches to active full refresh without trailing light refresh", async () => {
  const firstStatus = deferred<GeminiBrowserProviderStatus>();
  const firstRuns = deferred<GeminiBrowserRunLogSummary>();
  const deps = schedulerDeps({
    loadStatus: vi.fn().mockReturnValueOnce(firstStatus.promise),
    loadRuns: vi.fn().mockReturnValueOnce(firstRuns.promise),
  });
  const scheduler = createGeminiBrowserRefreshScheduler(deps);

  const active = scheduler.scheduleRefresh({ mode: "full" });
  const attached = scheduler.scheduleRefresh({ mode: "light" });

  expect(attached).toBe(active);
  firstStatus.resolve(status());
  firstRuns.resolve({ runs: [] });
  await attached;

  expect(deps.loadStatus).toHaveBeenCalledTimes(1);
  expect(deps.loadStatusSnapshot).not.toHaveBeenCalled();
});

it("ignores selected detail response with stale updated_at", async () => {
  const latest = run("selected");
  latest.updated_at = "2026-06-22T00:00:02Z";
  const stale = run("selected");
  stale.status = "running";
  stale.updated_at = "2026-06-22T00:00:01Z";
  const deps = schedulerDeps({
    getSelectedRunId: vi.fn(() => "selected"),
    getSelectedDetailToken: vi.fn(() => 1),
    loadRuns: vi
      .fn()
      .mockResolvedValueOnce({ runs: [latest] })
      .mockResolvedValueOnce({ runs: [] }),
    loadRun: vi.fn(async () => stale),
  });
  const scheduler = createGeminiBrowserRefreshScheduler(deps);

  await scheduler.scheduleRefresh({ mode: "light" });
  await scheduler.scheduleRefresh({ mode: "light" });

  expect(deps.applySelectedRun).toHaveBeenCalledWith(latest);
  expect(deps.applySelectedRun).not.toHaveBeenCalledWith(stale);
});

it("applies selected row from list_runs when it is visible", async () => {
  const selected = run("selected");
  const deps = schedulerDeps({
    getSelectedRunId: vi.fn(() => "selected"),
    getSelectedDetailToken: vi.fn(() => 1),
    loadRuns: vi.fn(async () => ({ runs: [selected] })),
  });

  await createGeminiBrowserRefreshScheduler(deps).scheduleRefresh({ mode: "light" });

  expect(deps.applySelectedRun).toHaveBeenCalledWith(selected);
  expect(deps.loadRun).not.toHaveBeenCalled();
});

it("loads selected detail even when list_runs fails", async () => {
  const selected = run("selected");
  const deps = schedulerDeps({
    getSelectedRunId: vi.fn(() => "selected"),
    getSelectedDetailToken: vi.fn(() => 1),
    loadRuns: vi.fn(async () => {
      throw new Error("history down");
    }),
    loadRun: vi.fn(async () => selected),
  });

  const outcome = await createGeminiBrowserRefreshScheduler(deps).scheduleRefresh({ mode: "light" });

  expect(outcome.allFailed).toBe(false);
  expect(deps.applyRunsError).toHaveBeenCalled();
  expect(deps.loadRun).toHaveBeenCalledWith("selected");
  expect(deps.applySelectedRun).toHaveBeenCalledWith(selected);
});

it("applies externally created prompt-pack runs from light refresh list_runs", async () => {
  const promptPackRun = run("prompt-pack-run");
  promptPackRun.source = "prompt_pack";
  const deps = schedulerDeps({
    loadRuns: vi.fn(async () => ({ runs: [promptPackRun] })),
  });

  await createGeminiBrowserRefreshScheduler(deps).scheduleRefresh({ mode: "light" });

  expect(deps.loadStatusSnapshot).toHaveBeenCalledTimes(1);
  expect(deps.loadStatus).not.toHaveBeenCalled();
  expect(deps.applyRuns).toHaveBeenCalledWith([promptPackRun]);
});

it("ignores selected detail response for an obsolete selection token", async () => {
  let selectedToken = 1;
  const selected = run("selected");
  const deps = schedulerDeps({
    getSelectedRunId: vi.fn(() => "selected"),
    getSelectedDetailToken: vi.fn(() => selectedToken),
    loadRuns: vi.fn(async () => ({ runs: [] })),
    loadRun: vi.fn(async () => {
      selectedToken = 2;
      return selected;
    }),
  });

  await createGeminiBrowserRefreshScheduler(deps).scheduleRefresh({ mode: "light" });

  expect(deps.applySelectedRun).not.toHaveBeenCalled();
});

it("does not reuse selected detail version guard across selection tokens", async () => {
  let selectedToken = 1;
  const first = run("selected");
  first.updated_at = "not-a-date";
  const second = run("selected");
  second.updated_at = "not-a-date";
  const deps = schedulerDeps({
    getSelectedRunId: vi.fn(() => "selected"),
    getSelectedDetailToken: vi.fn(() => selectedToken),
    loadRuns: vi
      .fn()
      .mockResolvedValueOnce({ runs: [first] })
      .mockResolvedValueOnce({ runs: [second] }),
  });
  const scheduler = createGeminiBrowserRefreshScheduler(deps);

  await scheduler.scheduleRefresh({ mode: "light" });
  selectedToken = 2;
  await scheduler.scheduleRefresh({ mode: "light" });

  expect(deps.applySelectedRun).toHaveBeenCalledWith(first);
  expect(deps.applySelectedRun).toHaveBeenCalledWith(second);
});

it("does not apply callbacks after disposal", async () => {
  const deps = schedulerDeps({
    isDisposed: vi.fn(() => true),
  });

  await createGeminiBrowserRefreshScheduler(deps).scheduleRefresh({ mode: "light" });

  expect(deps.applyStatus).not.toHaveBeenCalled();
  expect(deps.applyRuns).not.toHaveBeenCalled();
});

it("resolves with allFailed when every requested light read model fails", async () => {
  const deps = schedulerDeps({
    loadStatusSnapshot: vi.fn(async () => {
      throw new Error("snapshot down");
    }),
    loadRuns: vi.fn(async () => {
      throw new Error("runs down");
    }),
    getSelectedRunId: vi.fn(() => "selected"),
    loadRun: vi.fn(async () => {
      throw new Error("detail down");
    }),
  });

  const outcome = await createGeminiBrowserRefreshScheduler(deps).scheduleRefresh({ mode: "light" });

  expect(outcome.allFailed).toBe(true);
  expect(deps.applyStatusError).toHaveBeenCalled();
  expect(deps.applyRunsError).toHaveBeenCalled();
  expect(deps.applySelectedRunError).toHaveBeenCalled();
});
```

- [x] **Step 2: Run scheduler tests and confirm failure**

Run:

```powershell
npm.cmd test -- src/lib/gemini-browser-refresh-scheduler.test.ts
```

Expected: FAIL because scheduler mode/options/detail APIs do not exist.

- [x] **Step 3: Implement scheduler mode API**

In `src/lib/gemini-browser-refresh-scheduler.ts`, replace the public types with:

```ts
export type GeminiBrowserRefreshMode = "light" | "full";

export interface GeminiBrowserRefreshOptions {
  mode?: GeminiBrowserRefreshMode;
  forceTrailing?: boolean;
}

export interface GeminiBrowserRefreshOutcome {
  allFailed: boolean;
}

export interface GeminiBrowserRefreshSchedulerDeps {
  loadStatus: () => Promise<GeminiBrowserProviderStatus>;
  loadStatusSnapshot: () => Promise<GeminiBrowserProviderStatus>;
  loadRuns: () => Promise<GeminiBrowserRunLogSummary>;
  loadRun: (runId: string) => Promise<GeminiBrowserRun>;
  getSelectedRunId: () => string | null;
  getSelectedDetailToken: () => number;
  applyStatus: (status: GeminiBrowserProviderStatus) => void;
  applyRuns: (runs: GeminiBrowserRun[]) => void;
  applySelectedRun: (run: GeminiBrowserRun) => void;
  applySelectedRunUnavailable: (runId: string, message: string) => void;
  applySelectedRunError: (runId: string, message: string) => void;
  applyStatusError: (message: string | null) => void;
  applyRunsError: (message: string | null) => void;
  applyMessage: (message: string) => void;
  syncActivePromptResult: (runs: GeminiBrowserRun[]) => void;
  formatError: (context: string, error: unknown) => string;
  isRunNotFoundError: (error: unknown) => boolean;
  isDisposed?: () => boolean;
}

export interface GeminiBrowserRefreshScheduler {
  scheduleRefresh: (options?: GeminiBrowserRefreshOptions) => Promise<GeminiBrowserRefreshOutcome>;
  dispose: () => void;
}
```

Add helpers:

```ts
function modeRank(mode: GeminiBrowserRefreshMode) {
  return mode === "full" ? 2 : 1;
}

function strongestMode(
  left: GeminiBrowserRefreshMode,
  right: GeminiBrowserRefreshMode,
): GeminiBrowserRefreshMode {
  return modeRank(right) > modeRank(left) ? right : left;
}

function compareUpdatedAt(left: string, right: string) {
  const leftMs = Date.parse(left);
  const rightMs = Date.parse(right);
  if (Number.isNaN(leftMs) || Number.isNaN(rightMs)) return null;
  return leftMs - rightMs;
}

function selectedVersionKey(runId: string, token: number) {
  return `${token}:${runId}`;
}
```

Within `createGeminiBrowserRefreshScheduler`, keep:

```ts
let activeRefresh: Promise<GeminiBrowserRefreshOutcome> | null = null;
let activeMode: GeminiBrowserRefreshMode | null = null;
let trailingRequested = false;
let trailingMode: GeminiBrowserRefreshMode = "light";
let trailingPromise: Promise<GeminiBrowserRefreshOutcome> | null = null;
let disposed = false;
const latestSelectedRunVersions = new Map<string, string>();
```

Update `runRefreshOnce(mode)`:

- Use `deps.loadStatus()` only for `full`.
- Use `deps.loadStatusSnapshot()` only for `light`.
- Always call `deps.loadRuns()`.
- Read `const selectedRunId = deps.getSelectedRunId();` and `const selectedDetailToken = deps.getSelectedDetailToken();` before applying any selected run from `list_runs` or `get_run`.
- If `selectedRunId` is present in successfully returned runs, call `applySelectedRunIfCurrent(selectedRow, selectedRunId, selectedDetailToken)` and do not call `deps.loadRun(...)`; in this case selected detail was satisfied by `list_runs` and `get_run` is not a requested read model for this refresh.
- If `selectedRunId` is absent from successfully returned runs, or if `deps.loadRuns()` failed, call `deps.loadRun(selectedRunId)` and then `applySelectedRunIfCurrent(detailRun, selectedRunId, selectedDetailToken)`. Selected detail refresh must remain independent from history refresh failure.
- Apply `applySelectedRun(run)` only when current selected id and selected detail token still match, and `updated_at` is not older than `latestSelectedRunVersions.get(selectedVersionKey(run.run_id, requestedToken))`.
- On not found according to `deps.isRunNotFoundError(error)`, call `applySelectedRunUnavailable(selectedRunId, formattedMessage)`.
- On transient detail failure, call `applySelectedRunError(selectedRunId, formattedMessage)`.
- Before every `apply*` callback, check `disposed || deps.isDisposed?.()`.
- Return `{ allFailed: true }` only when every requested read model for the mode failed:
  - status is always requested: `loadStatusSnapshot` for `light`, `loadStatus` for `full`.
  - `list_runs` is always requested.
  - `get_run` is requested only when `selectedRunId` exists and `list_runs` either failed or did not contain that selected run.
  - if the selected row is present in successful `list_runs`, `get_run` is not requested and must not affect `allFailed`.
  - if at least one requested read model succeeded and was applied, return `{ allFailed: false }`.
- Expected read-model failures should set the corresponding error callback and resolve with the outcome. Unexpected callback/programming errors may still reject the promise.

Use this selected-run apply helper:

```ts
function applySelectedRunIfCurrent(
  run: GeminiBrowserRun,
  requestedRunId: string,
  requestedToken: number,
) {
  if (disposed || deps.isDisposed?.()) return;
  if (deps.getSelectedRunId() !== requestedRunId) return;
  if (deps.getSelectedDetailToken() !== requestedToken) return;
  const versionKey = selectedVersionKey(run.run_id, requestedToken);
  const latest = latestSelectedRunVersions.get(versionKey);
  if (latest) {
    const comparison = compareUpdatedAt(run.updated_at, latest);
    if (comparison !== null && comparison < 0) return;
    if (comparison === null && !Number.isNaN(Date.parse(latest))) return;
  }
  latestSelectedRunVersions.set(versionKey, run.updated_at);
  deps.applySelectedRun(run);
}
```

Update `scheduleRefresh(options)`:

```ts
function scheduleRefresh(options: GeminiBrowserRefreshOptions = {}) {
  const requestedMode = options.mode ?? "light";
  if (activeRefresh && !options.forceTrailing && activeMode && modeRank(activeMode) >= modeRank(requestedMode)) {
    return activeRefresh;
  }
  if (activeRefresh) {
    trailingRequested = true;
    trailingMode = strongestMode(trailingMode, requestedMode);
    return ensureTrailingPromise();
  }
  return startRefreshForCall(requestedMode);
}
```

Default mode is `light` for safety. Manual/user command paths must pass `{ mode: "full" }` explicitly when they need live status probing.

Update `finishRefresh` so it starts trailing with `const nextMode = trailingMode; trailingMode = "light"; const trailing = takeTrailingRequest(); const trailingRefresh = startRefreshForCall(nextMode);` and wires settlement:

```ts
void trailingRefresh.then(
  () => trailing.resolve?.(),
  (error) => trailing.reject?.(error),
);
```

`takeTrailingRequest()` must clear `trailingRequested`, `trailingPromise`, `resolveTrailing`, and `rejectTrailing` before starting the trailing refresh. This prevents callers awaiting a full trailing refresh behind an active light refresh from hanging.

Return:

```ts
return {
  scheduleRefresh,
  dispose() {
    disposed = true;
  },
};
```

- [x] **Step 4: Run scheduler tests**

Run:

```powershell
npm.cmd test -- src/lib/gemini-browser-refresh-scheduler.test.ts
```

Expected: PASS.

- [x] **Step 5: Commit scheduler modes**

Run:

```powershell
git add src/lib/gemini-browser-refresh-scheduler.ts src/lib/gemini-browser-refresh-scheduler.test.ts
git commit -m "feat: add Gemini Browser refresh modes"
```

Expected: commit succeeds.

---

### Task 5: Polling Controller

**Files:**
- Create: `src/lib/gemini-browser-polling.ts`
- Create: `src/lib/gemini-browser-polling.test.ts`

- [x] **Step 1: Add polling controller tests**

Create `src/lib/gemini-browser-polling.test.ts`:

```ts
import { beforeEach, describe, expect, it, vi } from "vitest";
import { createGeminiBrowserRefreshScheduler } from "./gemini-browser-refresh-scheduler";
import { createGeminiBrowserPollingController } from "./gemini-browser-polling";
import type {
  GeminiBrowserProviderStatus,
  GeminiBrowserRun,
} from "./types/gemini-browser";

describe("gemini browser polling controller", () => {
  let now = 0;

  beforeEach(() => {
    vi.useFakeTimers();
    now = 0;
  });

  it("uses idle cadence by default and active cadence when local pending work is fresh", async () => {
    const scheduleRefresh = vi.fn(async () => ({ allFailed: false }));
    const controller = createGeminiBrowserPollingController({
      scheduleRefresh,
      getActivitySnapshot: () => ({ runLogSignals: [], statusSignal: null }),
      now: () => now,
      idleMs: 5000,
      activeMs: 1000,
    });

    controller.start();
    await vi.advanceTimersByTimeAsync(5000);
    expect(scheduleRefresh).toHaveBeenCalledWith({ mode: "light" });

    controller.setLocalPendingRun("run-1");
    await vi.advanceTimersByTimeAsync(1000);
    expect(scheduleRefresh).toHaveBeenCalledTimes(2);

    controller.stop();
  });

  it("does not schedule a new polling refresh while one is in flight", async () => {
    let resolve!: () => void;
    const scheduleRefresh = vi.fn(
      () =>
        new Promise<{ allFailed: boolean }>((done) => {
          resolve = () => done({ allFailed: false });
        }),
    );
    const controller = createGeminiBrowserPollingController({
      scheduleRefresh,
      getActivitySnapshot: () => ({
        runLogSignals: [{ key: "run:active", updatedAt: "2026-06-22T00:00:00Z" }],
        statusSignal: null,
      }),
      now: () => Date.parse("2026-06-22T00:00:01Z"),
      idleMs: 5000,
      activeMs: 1000,
    });

    controller.start();
    await vi.advanceTimersToNextTimerAsync();
    await vi.advanceTimersByTimeAsync(5000);
    expect(scheduleRefresh).toHaveBeenCalledTimes(1);

    resolve();
    await Promise.resolve();
    controller.stop();
  });

  it("degrades active polling to idle cadence after three full polling failures", async () => {
    const scheduleRefresh = vi.fn(async () => ({ allFailed: true }));
    const controller = createGeminiBrowserPollingController({
      scheduleRefresh,
      getActivitySnapshot: () => ({
        runLogSignals: [{ key: "run:active", updatedAt: "2026-06-22T00:00:00Z" }],
        statusSignal: null,
      }),
      now: () => Date.parse("2026-06-22T00:00:01Z"),
      idleMs: 5000,
      activeMs: 1000,
      maxConsecutiveFailures: 3,
    });

    controller.start();
    for (let index = 0; index < 3; index += 1) {
      await vi.advanceTimersToNextTimerAsync();
      await Promise.resolve();
    }
    expect(scheduleRefresh).toHaveBeenCalledTimes(3);

    await vi.advanceTimersByTimeAsync(4999);
    expect(scheduleRefresh).toHaveBeenCalledTimes(3);
    await vi.advanceTimersByTimeAsync(1);
    expect(scheduleRefresh).toHaveBeenCalledTimes(4);
    controller.stop();
  });

  it("restores active cadence after a successful idle refresh", async () => {
    const scheduleRefresh = vi
      .fn()
      .mockResolvedValueOnce({ allFailed: true })
      .mockResolvedValueOnce({ allFailed: true })
      .mockResolvedValueOnce({ allFailed: true })
      .mockResolvedValue({ allFailed: false });
    const controller = createGeminiBrowserPollingController({
      scheduleRefresh,
      getActivitySnapshot: () => ({
        runLogSignals: [{ key: "run:active", updatedAt: "2026-06-22T00:00:00Z" }],
        statusSignal: null,
      }),
      now: () => Date.parse("2026-06-22T00:00:01Z"),
      idleMs: 5000,
      activeMs: 1000,
      maxConsecutiveFailures: 3,
    });

    controller.start();
    for (let index = 0; index < 3; index += 1) {
      await vi.advanceTimersToNextTimerAsync();
      await Promise.resolve();
    }
    await vi.advanceTimersByTimeAsync(5000);
    expect(scheduleRefresh).toHaveBeenCalledTimes(4);

    await vi.advanceTimersByTimeAsync(1000);
    expect(scheduleRefresh).toHaveBeenCalledTimes(5);
    controller.stop();
  });

  it("manual successful refresh outcome clears degraded polling state", async () => {
    const scheduleRefresh = vi.fn(async () => ({ allFailed: true }));
    const controller = createGeminiBrowserPollingController({
      scheduleRefresh,
      getActivitySnapshot: () => ({
        runLogSignals: [{ key: "run:active", updatedAt: "2026-06-22T00:00:00Z" }],
        statusSignal: null,
      }),
      now: () => Date.parse("2026-06-22T00:00:01Z"),
      idleMs: 5000,
      activeMs: 1000,
      maxConsecutiveFailures: 3,
    });

    controller.start();
    for (let index = 0; index < 3; index += 1) {
      await vi.advanceTimersToNextTimerAsync();
      await Promise.resolve();
    }

    controller.recordRefreshOutcome({ allFailed: false });
    await vi.advanceTimersByTimeAsync(1000);

    expect(scheduleRefresh).toHaveBeenCalledTimes(4);
    controller.stop();
  });

  it("clears local pending run only after confirmed terminal state", () => {
    const controller = createGeminiBrowserPollingController({
      scheduleRefresh: vi.fn(async () => ({ allFailed: false })),
      getActivitySnapshot: () => ({ runLogSignals: [], statusSignal: null }),
      now: () => now,
      idleMs: 5000,
      activeMs: 1000,
    });

    controller.setLocalPendingRun("run-1");
    expect(controller.hasLocalPendingRun()).toBe(true);
    controller.confirmPendingRunTerminal("run-1");
    expect(controller.hasLocalPendingRun()).toBe(false);
  });

  it("expires local pending runs after the grace window", async () => {
    const scheduleRefresh = vi.fn(async () => ({ allFailed: false }));
    const controller = createGeminiBrowserPollingController({
      scheduleRefresh,
      getActivitySnapshot: () => ({ runLogSignals: [], statusSignal: null }),
      now: () => now,
      idleMs: 5000,
      activeMs: 1000,
      activeGraceMs: 30 * 60 * 1000,
    });

    controller.setLocalPendingRun("run-1");
    now = 30 * 60 * 1000 + 1;
    controller.start();
    await vi.advanceTimersByTimeAsync(1000);
    expect(scheduleRefresh).not.toHaveBeenCalled();
    await vi.advanceTimersByTimeAsync(4000);
    expect(scheduleRefresh).toHaveBeenCalledTimes(1);
    expect(controller.hasLocalPendingRun()).toBe(false);
    controller.stop();
  });

  it("treats stale run-log activity as idle after the grace window", async () => {
    const scheduleRefresh = vi.fn(async () => ({ allFailed: false }));
    const controller = createGeminiBrowserPollingController({
      scheduleRefresh,
      getActivitySnapshot: () => ({
        runLogSignals: [{ key: "run:old", updatedAt: "2026-06-22T00:00:00Z" }],
        statusSignal: null,
      }),
      now: () => Date.parse("2026-06-22T00:31:00Z"),
      idleMs: 5000,
      activeMs: 1000,
      activeGraceMs: 30 * 60 * 1000,
    });

    controller.start();
    await vi.advanceTimersByTimeAsync(1000);
    expect(scheduleRefresh).not.toHaveBeenCalled();
    await vi.advanceTimersByTimeAsync(4000);
    expect(scheduleRefresh).toHaveBeenCalledTimes(1);
    controller.stop();
  });

  it("treats stale status-derived activity as idle after the grace window", async () => {
    const scheduleRefresh = vi.fn(async () => ({ allFailed: false }));
    const controller = createGeminiBrowserPollingController({
      scheduleRefresh,
      getActivitySnapshot: () => ({ runLogSignals: [], statusSignal: "status:running" }),
      now: () => now,
      idleMs: 5000,
      activeMs: 1000,
      activeGraceMs: 30 * 60 * 1000,
    });

    controller.start();
    await vi.advanceTimersByTimeAsync(1000);
    expect(scheduleRefresh).toHaveBeenCalledTimes(1);

    now = 30 * 60 * 1000 + 1;
    await vi.advanceTimersByTimeAsync(1000);
    expect(scheduleRefresh).toHaveBeenCalledTimes(1);
    await vi.advanceTimersByTimeAsync(5000);
    expect(scheduleRefresh).toHaveBeenCalledTimes(2);
    controller.stop();
  });

  it("requires two not-found confirmations after rejected pending run", () => {
    const controller = createGeminiBrowserPollingController({
      scheduleRefresh: vi.fn(async () => ({ allFailed: false })),
      getActivitySnapshot: () => ({ runLogSignals: [], statusSignal: null }),
      now: () => now,
    });

    controller.setLocalPendingRun("run-1");
    expect(controller.hasLocalPendingRun("run-1")).toBe(true);
    expect(controller.hasLocalPendingRun("other-run")).toBe(false);
    controller.markLocalPendingRunRejected("run-1");
    controller.confirmPendingRunNotFound("run-1");
    expect(controller.hasLocalPendingRun()).toBe(true);
    controller.confirmPendingRunNotFound("run-1");
    expect(controller.hasLocalPendingRun()).toBe(false);
  });

  it("idle polling discovers prompt-pack runs through scheduler light refresh without live status", async () => {
    const cachedStatus: GeminiBrowserProviderStatus = {
      status: "ready",
      manual_action: null,
      active_run_id: null,
      queue_depth: 0,
      browser_profile_dir: "profile-dir",
      latest_message: "Cached",
    };
    const promptPackRun: GeminiBrowserRun = {
      run_id: "prompt-pack-run",
      source: "prompt_pack",
      status: "ok",
      prompt_preview: "Summarize",
      created_at: "2026-06-22T00:00:00Z",
      updated_at: "2026-06-22T00:00:01Z",
      result: null,
    };
    const loadStatus = vi.fn(async () => cachedStatus);
    const loadStatusSnapshot = vi.fn(async () => cachedStatus);
    const applyRuns = vi.fn();
    const scheduler = createGeminiBrowserRefreshScheduler({
      loadStatus,
      loadStatusSnapshot,
      loadRuns: vi.fn(async () => ({ runs: [promptPackRun] })),
      loadRun: vi.fn(async () => promptPackRun),
      getSelectedRunId: vi.fn(() => null),
      getSelectedDetailToken: vi.fn(() => 0),
      applyStatus: vi.fn(),
      applyRuns,
      applySelectedRun: vi.fn(),
      applySelectedRunUnavailable: vi.fn(),
      applySelectedRunError: vi.fn(),
      applyStatusError: vi.fn(),
      applyRunsError: vi.fn(),
      applyMessage: vi.fn(),
      syncActivePromptResult: vi.fn(),
      formatError: (_context, error) => String(error),
      isRunNotFoundError: vi.fn(() => false),
    });
    const controller = createGeminiBrowserPollingController({
      scheduleRefresh: scheduler.scheduleRefresh,
      getActivitySnapshot: () => ({ runLogSignals: [], statusSignal: null }),
      idleMs: 5000,
      activeMs: 1000,
    });

    controller.start();
    await vi.advanceTimersByTimeAsync(5000);

    expect(loadStatusSnapshot).toHaveBeenCalledTimes(1);
    expect(loadStatus).not.toHaveBeenCalled();
    expect(applyRuns).toHaveBeenCalledWith([promptPackRun]);
    controller.stop();
  });
});
```

- [x] **Step 2: Run polling tests and confirm failure**

Run:

```powershell
npm.cmd test -- src/lib/gemini-browser-polling.test.ts
```

Expected: FAIL because `gemini-browser-polling.ts` does not exist.

- [x] **Step 3: Implement polling controller**

Create `src/lib/gemini-browser-polling.ts`:

```ts
import type {
  GeminiBrowserRefreshOptions,
  GeminiBrowserRefreshOutcome,
} from "./gemini-browser-refresh-scheduler";

export interface GeminiBrowserPollingSignal {
  key: string;
  updatedAt?: string | null;
}

export interface GeminiBrowserPollingActivitySnapshot {
  runLogSignals: GeminiBrowserPollingSignal[];
  statusSignal: string | null;
}

export interface GeminiBrowserPollingControllerDeps {
  scheduleRefresh: (options: GeminiBrowserRefreshOptions) => Promise<GeminiBrowserRefreshOutcome>;
  getActivitySnapshot: () => GeminiBrowserPollingActivitySnapshot;
  now?: () => number;
  idleMs?: number;
  activeMs?: number;
  activeGraceMs?: number;
  pendingNotFoundRetryMs?: number;
  maxConsecutiveFailures?: number;
}

export interface GeminiBrowserPollingController {
  start: () => void;
  stop: () => void;
  setLocalPendingRun: (runId: string) => void;
  markLocalPendingRunRejected: (runId: string) => void;
  confirmPendingRunNotFound: (runId: string) => void;
  clearLocalPendingRun: (runId: string) => void;
  confirmPendingRunTerminal: (runId: string) => void;
  recordRefreshOutcome: (outcome: GeminiBrowserRefreshOutcome) => void;
  hasLocalPendingRun: (runId?: string) => boolean;
}

export function createGeminiBrowserPollingController(
  deps: GeminiBrowserPollingControllerDeps,
): GeminiBrowserPollingController {
  const idleMs = deps.idleMs ?? 5000;
  const activeMs = deps.activeMs ?? 1000;
  const activeGraceMs = deps.activeGraceMs ?? 30 * 60 * 1000;
  const pendingNotFoundRetryMs = deps.pendingNotFoundRetryMs ?? 2000;
  const maxConsecutiveFailures = deps.maxConsecutiveFailures ?? 3;
  let timer: ReturnType<typeof setTimeout> | null = null;
  let running = false;
  let inFlight = false;
  let degraded = false;
  let consecutiveFailures = 0;
  const localPendingRuns = new Map<
    string,
    { startedAt: number; rejectedAt: number | null; notFoundCount: number }
  >();
  const firstSeenActivity = new Map<string, number>();

  function now() {
    return deps.now?.() ?? Date.now();
  }

  function clearTimer() {
    if (timer) {
      clearTimeout(timer);
      timer = null;
    }
  }

  function isFreshByTimestamp(updatedAt: string | null | undefined) {
    if (!updatedAt) return false;
    const updatedAtMs = Date.parse(updatedAt);
    if (Number.isNaN(updatedAtMs)) return false;
    return now() - updatedAtMs <= activeGraceMs;
  }

  function isFreshByFirstSeen(key: string) {
    const seenAt = firstSeenActivity.get(key);
    if (seenAt === undefined) {
      firstSeenActivity.set(key, now());
      return true;
    }
    return now() - seenAt <= activeGraceMs;
  }

  function isFreshSignal(signal: GeminiBrowserPollingSignal) {
    if (signal.updatedAt) return isFreshByTimestamp(signal.updatedAt);
    return isFreshByFirstSeen(`run:${signal.key}`);
  }

  function pruneExpiredPendingRuns() {
    for (const [runId, pending] of localPendingRuns) {
      const age = now() - pending.startedAt;
      const rejectedAge = pending.rejectedAt === null ? 0 : now() - pending.rejectedAt;
      if (age > activeGraceMs) {
        localPendingRuns.delete(runId);
      } else if (pending.notFoundCount >= 2) {
        localPendingRuns.delete(runId);
      } else if (pending.rejectedAt !== null && pending.notFoundCount > 0 && rejectedAge >= pendingNotFoundRetryMs) {
        localPendingRuns.delete(runId);
      }
    }
  }

  function hasFreshStatusSignal(snapshot: GeminiBrowserPollingActivitySnapshot, hasRunActivity: boolean) {
    if (!snapshot.statusSignal) return false;
    if (hasRunActivity) return true;
    return isFreshByFirstSeen(`status:${snapshot.statusSignal}`);
  }

  function active() {
    pruneExpiredPendingRuns();
    if (degraded) return false;
    const snapshot = deps.getActivitySnapshot();
    const hasRunActivity =
      localPendingRuns.size > 0 || snapshot.runLogSignals.some((signal) => isFreshSignal(signal));
    return hasRunActivity || hasFreshStatusSignal(snapshot, hasRunActivity);
  }

  function applyRefreshOutcome(outcome: GeminiBrowserRefreshOutcome) {
    if (outcome.allFailed) {
      consecutiveFailures += 1;
      if (consecutiveFailures >= maxConsecutiveFailures) {
        degraded = true;
      }
    } else {
      consecutiveFailures = 0;
      degraded = false;
    }
  }

  function scheduleNext() {
    if (!running) return;
    clearTimer();
    const wasActive = active();
    timer = setTimeout(() => {
      void tick(wasActive);
    }, wasActive ? activeMs : idleMs);
  }

  async function tick(wasActive: boolean) {
    if (!running) return;
    if (inFlight) {
      scheduleNext();
      return;
    }
    if (wasActive && !active()) {
      scheduleNext();
      return;
    }
    inFlight = true;
    try {
      const outcome = await deps.scheduleRefresh({ mode: "light" });
      applyRefreshOutcome(outcome);
    } catch (_error) {
      consecutiveFailures += 1;
      if (consecutiveFailures >= maxConsecutiveFailures) {
        degraded = true;
      }
    } finally {
      inFlight = false;
      scheduleNext();
    }
  }

  return {
    start() {
      if (running) return;
      running = true;
      scheduleNext();
    },
    stop() {
      running = false;
      clearTimer();
    },
    setLocalPendingRun(runId) {
      localPendingRuns.set(runId, { startedAt: now(), rejectedAt: null, notFoundCount: 0 });
      if (running) scheduleNext();
    },
    markLocalPendingRunRejected(runId) {
      const pending = localPendingRuns.get(runId);
      if (!pending) return;
      pending.rejectedAt = now();
    },
    confirmPendingRunNotFound(runId) {
      const pending = localPendingRuns.get(runId);
      if (!pending) return;
      pending.notFoundCount += 1;
      if (pending.rejectedAt === null) pending.rejectedAt = now();
      pruneExpiredPendingRuns();
    },
    clearLocalPendingRun(runId) {
      localPendingRuns.delete(runId);
    },
    confirmPendingRunTerminal(runId) {
      localPendingRuns.delete(runId);
    },
    recordRefreshOutcome(outcome) {
      applyRefreshOutcome(outcome);
      if (running) scheduleNext();
    },
    hasLocalPendingRun(runId) {
      pruneExpiredPendingRuns();
      return runId ? localPendingRuns.has(runId) : localPendingRuns.size > 0;
    },
  };
}
```

The controller owns the grace-window rules. The settings panel must not duplicate raw `status.running || runs.some(...)` logic outside `getActivitySnapshot()`, otherwise stale status/run rows can keep active polling alive forever.

- [x] **Step 4: Run polling tests**

Run:

```powershell
npm.cmd test -- src/lib/gemini-browser-polling.test.ts
```

Expected: PASS.

- [x] **Step 5: Commit polling controller**

Run:

```powershell
git add src/lib/gemini-browser-polling.ts src/lib/gemini-browser-polling.test.ts
git commit -m "feat: add Gemini Browser polling controller"
```

Expected: commit succeeds.

---

### Task 6: Settings Panel Pull Refresh Wiring

**Files:**
- Modify: `src/lib/components/settings/gemini-browser-provider-panel.svelte`
- Modify: `src/lib/gemini-browser-provider-panel.test.ts`

- [x] **Step 1: Update source-contract tests**

In `src/lib/gemini-browser-provider-panel.test.ts`:

1. Rename the scheduler routing test to:

```ts
it("routes mount, commands, and polling through the shared refresh scheduler", () => {
```

2. Replace event assertions:

```ts
expect(componentSource).toContain("createGeminiBrowserPollingController");
expect(componentSource).toContain("geminiBridgeStatusSnapshot");
expect(componentSource).toContain('scheduleRefresh({ mode: "light" })');
expect(componentSource).toContain('scheduleRefresh({ mode: "full" })');
expect(componentSource).not.toContain("listenToGeminiBrowserRunChanges");
expect(componentSource).not.toContain("listenToGeminiBrowserRuns");
expect(componentSource).not.toContain("@tauri-apps/api/event");
expect(componentSource).not.toContain("payload.");
```

3. Update config test:

```ts
expect(componentSource).toContain("loadStatus: () => geminiBridgeStatus(browserConfig())");
expect(componentSource).toContain("loadStatusSnapshot: () => geminiBridgeStatusSnapshot()");
expect(componentSource).toContain("loadRun: (runId) => geminiBridgeGetRun(runId)");
```

4. Add tests:

```ts
it("starts active polling before awaiting the terminal test prompt result", () => {
  const sendIndex = componentSource.indexOf("const sendPromise = geminiBridgeSendSingle");
  const pendingIndex = componentSource.indexOf("pollingController.setLocalPendingRun(runId)");
  const refreshIndex = componentSource.indexOf('await scheduleRefresh({ mode: "light" })');
  const awaitIndex = componentSource.indexOf("const completed = await sendPromise");

  expect(sendIndex).toBeGreaterThan(-1);
  expect(pendingIndex).toBeGreaterThan(-1);
  expect(refreshIndex).toBeGreaterThan(-1);
  expect(awaitIndex).toBeGreaterThan(-1);
  expect(pendingIndex).toBeLessThan(awaitIndex);
  expect(refreshIndex).toBeLessThan(awaitIndex);
});

it("uses light post-terminal refresh for test prompt completion", () => {
  const awaitIndex = componentSource.indexOf("const completed = await sendPromise");
  const finalRefreshIndex = componentSource.indexOf("await ensurePostTerminalRefresh(runId)", awaitIndex);
  const finalFullIndex = componentSource.indexOf(
    'await scheduleRefresh({ mode: "full", forceTrailing: true })',
    awaitIndex,
  );

  expect(finalRefreshIndex).toBeGreaterThan(awaitIndex);
  expect(finalFullIndex).toBe(-1);
});


it("creates polling controller synchronously before scheduler and send actions", () => {
  const pollingIndex = componentSource.indexOf(
    "const pollingController = createGeminiBrowserPollingController",
  );
  const schedulerIndex = componentSource.indexOf(
    "const refreshScheduler = createGeminiBrowserRefreshScheduler",
  );
  const sendIndex = componentSource.indexOf("async function sendTestPrompt");

  expect(pollingIndex).toBeGreaterThan(-1);
  expect(schedulerIndex).toBeGreaterThan(pollingIndex);
  expect(sendIndex).toBeGreaterThan(schedulerIndex);
  expect(componentSource).not.toContain("pollingController?.setLocalPendingRun");
});

it("routes selected run detail through scheduler token guard", () => {
  expect(componentSource).toContain("selectedDetailRequestToken");
  expect(componentSource).toContain("applySelectedRunFromScheduler");
  expect(componentSource).toContain("getSelectedDetailToken: () => selectedDetailRequestToken");
  expect(componentSource).not.toContain("latestSelectedRunUpdatedAt");
});

it("uses activity snapshots instead of raw active-work booleans", () => {
  expect(componentSource).toContain("getPollingActivitySnapshot");
  expect(componentSource).toContain("runLogSignals");
  expect(componentSource).toContain("statusSignal");
  expect(componentSource).not.toContain("hasActiveGeminiBrowserWork");
  expect(componentSource).not.toContain("hasActiveWork:");
});

it("discovers prompt-pack Gemini Browser runs through idle polling list_runs", () => {
  expect(componentSource).toContain("pollingController.start()");
  expect(componentSource).toContain("loadRuns: () => geminiBridgeListRuns()");
  expect(componentSource).toContain("applyRuns: (nextRuns) =>");
  expect(componentSource).toContain("runs = nextRuns");
  expect(componentSource).not.toContain("listenToGeminiBrowserRunChanges");
});

it("does not route polling through the background refresh wrapper", () => {
  const controllerIndex = componentSource.indexOf("const pollingController = createGeminiBrowserPollingController");
  const controllerBlock = componentSource.slice(controllerIndex, componentSource.indexOf("const refreshScheduler"));

  expect(controllerBlock).toContain("scheduleRefresh,");
  expect(controllerBlock).not.toContain("scheduleRefreshInBackground");
});

it("records initial mount refresh outcome for polling degradation", () => {
  expect(componentSource).toContain(
    'scheduleRefreshInBackground({ mode: "light" }, { recordPollingOutcome: true })',
  );
});

it("keeps rejected pending test runs until terminal, not-found confirmation, or grace expiry", () => {
  const catchIndex = componentSource.indexOf("} catch (error) {");
  const rejectedIndex = componentSource.indexOf(
    "pollingController.markLocalPendingRunRejected(runId)",
    catchIndex,
  );
  const finalRefreshIndex = componentSource.indexOf(
    "await ensurePostTerminalRefresh(runId)",
    catchIndex,
  );
  const unavailableIndex = componentSource.indexOf("applySelectedRunUnavailable");
  const notFoundIndex = componentSource.indexOf(
    "pollingController.confirmPendingRunNotFound(runId)",
    unavailableIndex,
  );

  expect(catchIndex).toBeGreaterThan(-1);
  expect(rejectedIndex).toBeGreaterThan(catchIndex);
  expect(finalRefreshIndex).toBeGreaterThan(rejectedIndex);
  expect(notFoundIndex).toBeGreaterThan(unavailableIndex);
  expect(componentSource).toContain("pollingController.hasLocalPendingRun(runId)");
  expect(componentSource).toContain("pollingController.confirmPendingRunTerminal");
});
```

- [x] **Step 2: Run panel tests and confirm failure**

Run:

```powershell
npm.cmd test -- src/lib/gemini-browser-provider-panel.test.ts
```

Expected: FAIL because the panel still imports/listens to run-change events and has no polling controller.

- [x] **Step 3: Update panel imports**

In `src/lib/components/settings/gemini-browser-provider-panel.svelte`:

1. Remove `listenToGeminiBrowserRunChanges` from the API imports.
2. Add API imports:

```ts
geminiBridgeGetRun,
geminiBridgeStatusSnapshot,
isGeminiBrowserRunNotFoundError,
```

3. Add:

```ts
import {
  createGeminiBrowserPollingController,
} from "$lib/gemini-browser-polling";
import type { GeminiBrowserRefreshOptions } from "$lib/gemini-browser-refresh-scheduler";
```

- [x] **Step 4: Add selected-detail state**

Near existing state declarations, add:

```ts
let disposed = false;
let selectedRunDetail = $state<GeminiBrowserRun | null>(null);
let selectedDetailError = $state<string | null>(null);
let selectedDetailRequestToken = 0;
```

Change `selectedInspectorRun` to prefer selected detail when it matches the selected id:

```ts
const selectedInspectorRun = $derived(
  selectedRunDetail?.run_id === (activeInspectorRunId ?? selectedHistoryRunId)
    ? selectedRunDetail
    : selectRunForHistory(runs, activeInspectorRunId, selectedHistoryRunId, runHistoryFilter),
);
```

If this expression is too dense for Svelte parsing, create a small helper function:

```ts
function selectedRunIdForInspector() {
  return activeInspectorRunId ?? selectedHistoryRunId;
}
```

and use it in the derived expression.

- [x] **Step 5: Add selected-run apply helper**

In the `<script>` block, add:

```ts
function applySelectedRunFromScheduler(run: GeminiBrowserRun) {
  const selectedRunId = selectedRunIdForInspector();
  if (disposed || run.run_id !== selectedRunId) return;
  selectedRunDetail = run;
  selectedDetailError = null;
}
```

Do not duplicate the `updated_at` freshness guard in the panel. The scheduler owns selected-run version ordering through `latestSelectedRunVersions`; the panel callback only applies the already-authorized run if it still matches the current inspector selection.

In `selectHistoryRun(runId)`, add:

```ts
selectedDetailRequestToken += 1;
selectedRunDetail = null;
selectedDetailError = null;
```

- [x] **Step 6: Add refresh wrappers and create polling controller before scheduler**

Add the refresh wrappers before `refreshScheduler` is created:

```ts
function scheduleRefresh(options: GeminiBrowserRefreshOptions = {}) {
  return refreshScheduler.scheduleRefresh(options);
}
```

Then create the polling controller synchronously before `refreshScheduler`:

```ts
function getPollingActivitySnapshot() {
  const runLogSignals = runs
    .filter((run) => run.status === "queued" || run.status === "running")
    .map((run) => ({ key: run.run_id, updatedAt: run.updated_at }));
  const statusSignal =
    status?.status === "running" || status?.active_run_id || (status?.queue_depth ?? 0) > 0
      ? `${status.status}:${status.active_run_id ?? "none"}:${status.queue_depth ?? 0}`
      : null;
  return { runLogSignals, statusSignal };
}

const pollingController = createGeminiBrowserPollingController({
  scheduleRefresh,
  getActivitySnapshot: getPollingActivitySnapshot,
});
```

The controller is intentionally created synchronously during component initialization, before `sendTestPrompt()` can be invoked. Do not use optional chaining for local-pending registration.
`createGeminiBrowserPollingController(...)` must remain inert during construction: it may store the `scheduleRefresh` function, but it must not call it until `pollingController.start()` or another explicit controller method is invoked. This keeps the early controller construction safe even though `scheduleRefresh(...)` closes over `refreshScheduler`, which is declared immediately after the controller.
`status` is the last scheduler-applied provider status, regardless of mode. A light `geminiBridgeStatusSnapshot()` result must overwrite the same `status` variable used by `getPollingActivitySnapshot()`, so stale live/full status fields cannot keep active polling alive after cached snapshot reads report idle. Do not split this into separate `liveStatus` and `pollingStatus` unless the activity snapshot explicitly uses the cached/polling status.

After `refreshScheduler` is created, add the fire-and-forget wrapper:

```ts
function scheduleRefreshInBackground(
  options: GeminiBrowserRefreshOptions = {},
  behavior: { recordPollingOutcome?: boolean } = {},
) {
  void scheduleRefresh(options)
    .then((outcome) => {
      if (behavior.recordPollingOutcome) {
        pollingController.recordRefreshOutcome(outcome);
      }
    })
    .catch(reportUnexpectedRefreshError);
}
```

Polling must never use `scheduleRefreshInBackground`; it receives `scheduleRefresh` directly so `GeminiBrowserRefreshOutcome.allFailed` remains available to the controller.
Use `scheduleRefreshInBackground(...)` only for fire-and-forget UI paths. If the caller needs to update polling degradation state, it must either await `scheduleRefresh(...)` directly or pass `{ recordPollingOutcome: true }`.

Add a post-terminal helper:

```ts
function isTerminalRunVisible(runId: string) {
  return runs.some((run) => run.run_id === runId && run.status !== "queued" && run.status !== "running");
}

async function ensurePostTerminalRefresh(runId: string) {
  if (isTerminalRunVisible(runId)) {
    return;
  }
  const outcome = await scheduleRefresh({ mode: "light" });
  pollingController.recordRefreshOutcome(outcome);
  if (isTerminalRunVisible(runId)) {
    return;
  }
  const trailingOutcome = await scheduleRefresh({ mode: "light", forceTrailing: true });
  pollingController.recordRefreshOutcome(trailingOutcome);
}
```

This helper is deliberately conditional. It ensures at least one post-terminal cached status/run-log read when the currently applied `runs` state has not yet observed the terminal row, but it avoids an unconditional extra refresh when another refresh has already applied the terminal state. The fallback `forceTrailing` is light-only and is used only after the first post-terminal light read still has not made the terminal row visible.

- [x] **Step 7: Update scheduler deps**

Update `createGeminiBrowserRefreshScheduler({ ... })` deps:

```ts
loadStatus: () => geminiBridgeStatus(browserConfig()),
loadStatusSnapshot: () => geminiBridgeStatusSnapshot(),
loadRuns: () => geminiBridgeListRuns(),
loadRun: (runId) => geminiBridgeGetRun(runId),
getSelectedRunId: () => selectedRunIdForInspector(),
getSelectedDetailToken: () => selectedDetailRequestToken,
applyStatus: (nextStatus) => {
  status = nextStatus;
},
applyRuns: (nextRuns) => {
  runs = nextRuns;
  syncActivePromptResult(nextRuns);
},
applySelectedRun: (run) => applySelectedRunFromScheduler(run),
applySelectedRunUnavailable: (runId, message) => {
  if (selectedRunIdForInspector() !== runId) return;
  if (pollingController.hasLocalPendingRun(runId)) {
    pollingController.confirmPendingRunNotFound(runId);
  }
  selectedRunDetail = null;
  selectedDetailError = message;
},
applySelectedRunError: (runId, message) => {
  if (selectedRunIdForInspector() !== runId) return;
  selectedDetailError = message;
},
isRunNotFoundError: isGeminiBrowserRunNotFoundError,
isDisposed: () => disposed,
```

Update button handlers that use `scheduleRefreshInBackground` for manual refresh:

```svelte
onclick={() => scheduleRefreshInBackground({ mode: "full" }, { recordPollingOutcome: true })}
```

In `syncActivePromptResult(nextRuns)`, after detecting completed result and clearing `activeTestRunId`, add:

```ts
pollingController.confirmPendingRunTerminal(completedResult.run_id);
```

- [x] **Step 8: Update long-running command flow**

Replace `sendTestPrompt()` command body with this structure:

```ts
busy = true;
result = null;
const runId = newRunId();
activeTestRunId = runId;
selectedHistoryRunId = runId;
pollingController.setLocalPendingRun(runId);
try {
  const sendPromise = geminiBridgeSendSingle({
    runId,
    prompt: prompt.trim(),
    source: "settings_test",
    artifactMode: "reduced",
    browserConfig: browserConfig(),
  });
  await scheduleRefresh({ mode: "light" });
  const completed = await sendPromise;
  message = completed.message ?? completed.status;
  await ensurePostTerminalRefresh(runId);
} catch (error) {
  pollingController.markLocalPendingRunRejected(runId);
  message = formatAppError("running Gemini browser prompt", error);
  await ensurePostTerminalRefresh(runId);
} finally {
  busy = false;
}
```

Do not clear pending state directly in `catch`; it must clear through refreshed terminal/not-found/grace behavior.
The final post-terminal refresh path is intentionally light-only: it guarantees cached status/run-log reads after the `send_single` promise settles, but it must not schedule an extra `full` refresh or live `gemini_bridge_status` probe.

Update other command flows:

- `startCdpChrome`, `openBrowser`, `resumeProvider`, and `stopProvider` should call `const outcome = await scheduleRefresh({ mode: "full" }); pollingController.recordRefreshOutcome(outcome);`.
- Manual refresh buttons should call `scheduleRefreshInBackground({ mode: "full" }, { recordPollingOutcome: true })`.
- Mount should call `scheduleRefreshInBackground({ mode: "light" }, { recordPollingOutcome: true })` and start polling, so initial full read-model failure contributes to degraded cadence immediately.

- [x] **Step 9: Remove event listener on mount**

Replace the `onMount` block with:

```ts
onMount(() => {
  disposed = false;
  loadBrowserProviderConfig();
  scheduleRefreshInBackground({ mode: "light" }, { recordPollingOutcome: true });
  pollingController.start();
  return () => {
    disposed = true;
    pollingController.stop();
    refreshScheduler.dispose();
  };
});
```

No `listenToGeminiBrowserRunChanges` call should remain.

- [x] **Step 10: Run panel tests**

Run:

```powershell
npm.cmd test -- src/lib/gemini-browser-provider-panel.test.ts
```

Expected: PASS.

- [x] **Step 11: Run Svelte check**

Run:

```powershell
npm.cmd run check
```

Expected: PASS with 0 errors.

- [x] **Step 12: Commit panel wiring**

Run:

```powershell
git add src/lib/components/settings/gemini-browser-provider-panel.svelte src/lib/gemini-browser-provider-panel.test.ts
git commit -m "feat: poll Gemini Browser settings state"
```

Expected: commit succeeds.

---

### Task 7: Docs And Final Verification

**Files:**
- Modify: `docs/browser-providers-llm-troubleshooting.md`
- Modify: `docs/architecture-deep-dive.md`
- Modify: `docs/superpowers/specs/2026-06-22-gemini-browser-pull-refresh-no-run-event-design.md`

- [x] **Step 1: Update docs references**

Search:

```powershell
rg -n "gemini-browser://run|run-change|listenToGeminiBrowserRunChanges|GeminiBrowserRunChangeEvent|GEMINI_BROWSER_RUN_CHANGE_EVENT" docs src src-tauri
```

This search is discovery only, not a failure gate. It is expected to find historical specs/plans and this implementation plan while the task is in progress. The source failure gate is Step 5.

Expected: may print matches. Do not fail this task because of this command.

Update current docs outside historical plans/specs:

- `docs/browser-providers-llm-troubleshooting.md`: describe Settings panel freshness as `status_snapshot + list_runs + get_run` polling. Mention live `gemini_bridge_status` only for manual/full refresh.
- `docs/architecture-deep-dive.md`: describe Gemini Browser UI as pull-refresh, not event-invalidation.
- `docs/superpowers/specs/2026-06-22-gemini-browser-pull-refresh-no-run-event-design.md`: change `Status: draft for review.` to `Status: approved; implementation planned.`

Historical files under `docs/superpowers/plans/2026-06-20-*`, `docs/superpowers/plans/2026-06-22-gemini-browser-run-event-invalidation-plan.md`, and `docs/superpowers/specs/2026-06-22-gemini-browser-run-event-invalidation-design.md` may keep historical event references.

- [x] **Step 2: Run Rust verification**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-pull-refresh --lib gemini_browser
```

Expected: PASS.

- [x] **Step 3: Run frontend verification**

Run:

```powershell
npm.cmd test -- src/lib/api/gemini-browser.test.ts src/lib/gemini-browser-refresh-scheduler.test.ts src/lib/gemini-browser-polling.test.ts src/lib/gemini-browser-provider-panel.test.ts src/lib/gemini-browser-provider-panel-state.test.ts src/lib/gemini-browser-run-inspector.test.ts src/lib/gemini-browser-setup-status.test.ts
```

Expected: PASS.

Run:

```powershell
npm.cmd run check
```

Expected: PASS with 0 errors.

- [x] **Step 4: Run prompt-pack regression tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-pull-refresh --lib prompt_packs
```

Expected: PASS. This guards the Gemini Browser prompt-pack runtime provider path.

- [x] **Step 5: Run source removal gates**

Run:

```powershell
rg -n "gemini-browser://run|GEMINI_BROWSER_RUN_CHANGE_EVENT|GeminiBrowserRunChangeEvent|listenToGeminiBrowserRunChanges|run_change_event_from_run|emit_run_change_event" src-tauri\src src\lib\api\gemini-browser.ts src\lib\types\gemini-browser.ts src\lib\components\settings\gemini-browser-provider-panel.svelte src\lib\gemini-browser-refresh-scheduler.ts src\lib\gemini-browser-polling.ts
```

Expected: no output.

Run:

```powershell
rg -n "@tauri-apps/api/event" src\lib\api\gemini-browser.ts src\lib\components\settings\gemini-browser-provider-panel.svelte
```

Expected: no output.

- [x] **Step 6: Run formatting checks**

Run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml --check
```

Expected: PASS.

Run:

```powershell
git diff --check
```

Expected: no whitespace errors.

- [x] **Step 7: Commit docs and final verification updates**

Run:

```powershell
git add docs/browser-providers-llm-troubleshooting.md docs/architecture-deep-dive.md docs/superpowers/specs/2026-06-22-gemini-browser-pull-refresh-no-run-event-design.md docs/superpowers/plans/2026-06-22-gemini-browser-pull-refresh-no-run-event-plan.md
git commit -m "docs: describe Gemini Browser pull refresh"
```

Expected: commit succeeds.

- [x] **Step 8: Final status**

Run:

```powershell
git status --short --branch
```

Expected: clean working tree on `gemini-browser-pull-refresh-no-run-event`, ahead of `main` by the task commits.
