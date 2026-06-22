# Gemini Browser Apalis Pilot Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the Gemini Browser in-memory queue with a real Apalis-backed SQLite job queue while keeping the current Tauri commands, events, run log, Prompt Pack integration, and UI behavior stable.

**Architecture:** Apalis owns durable queue storage and single-worker execution. Extractum keeps the existing file-backed Gemini Browser run log as the product-facing projection for Settings, run history, Prompt Pack provenance, and diagnostics. `send_single_prompt(...)` remains synchronous from the caller perspective by enqueueing a durable job and waiting on the receiver returned when that specific run was registered.

**Tech Stack:** Rust, Tauri 2, Tokio, `apalis = "=1.0.0-rc.9"`, `apalis-sqlite = "=1.0.0-rc.9"`, `tower` timeout middleware, `parking_lot`, serde, existing Gemini Browser sidecar, existing file-backed run log.

---

## Review Fixes Applied To This Plan

- The first implementation tasks now require a real Apalis SQLite storage smoke test, a real `TaskSink::push(...)`, and a real worker processing a fake job before command refactors begin.
- The plan no longer creates a production `enqueue()` that returns an uninitialized-queue error instead of using Apalis.
- SQLite storage ownership is specified: Apalis owns its internal queue tables inside the existing main `extractum.db` database.
- Completion waiting is specified with a per-run waiter map, timeout behavior, worker failure behavior, and restart behavior.
- Cancellation is specified for queued and active Gemini Browser jobs, including Prompt Pack cancellation by concrete `browser_run_id`.
- Worker bootstrap must use a real `WorkerBuilder` and backend with `.concurrency(1)`.
- `GeminiBrowserRunResult` examples use the current fields: `run_id`, `status`, `text`, `message`, `manual_action`, `artifacts`, `elapsed_ms`, and `debug_summary`.
- Apalis status mapping is test-driven because current docs show core statuses such as `Done` and SQL examples such as `"completed"`.
- Run log versus Apalis reconciliation is specified as an explicit startup and worker-entry policy.
- Status UI responsiveness is protected by a cached status snapshot and a short sidecar status timeout instead of waiting behind long `send_single` calls.
- No automatic retry is enforced with an Apalis `attempts(1)` task contract and an execution-count test.
- Prompt Pack browser cancellation has dedicated queued and active browser-stage tests.
- Worker startup state, duplicate `run_id` handling, enqueue failure cleanup, test timeouts, and Apalis query capability discovery are specified explicitly.
- In-memory waiter/cancellation maps and cached status snapshots use synchronous locks; no lock guard may be held across `.await`.
- Worker execution has a Tower timeout layer so a hung Chrome/sidecar task cannot block the single-concurrency queue forever.
- Apalis dependencies are pinned to the verified `1.0.0-rc.9` pre-release because Cargo will not select pre-releases from a plain `"1"` requirement.
- Closed waiter channels and duplicate Apalis idempotency conflicts are mapped to product-level `AppError` values instead of panics or raw SQLite errors.

## Current State

Gemini Browser currently has a custom in-memory queue:

- `src-tauri/src/gemini_browser/state.rs` stores `Mutex<VecDeque<GeminiBrowserRunRequest>>`, one active run id, a cancellation token, and the sidecar process handle.
- `src-tauri/src/gemini_browser/commands.rs` validates input, writes a queued run log record, enqueues the request, immediately pops the next request, marks it running, calls `sidecar::send_single`, writes the terminal run log record, and emits `gemini-browser://run` events.
- `src-tauri/src/gemini_browser/run_log.rs` is the product-facing run history and artifact projection.
- `src-tauri/src/prompt_packs/runtime.rs` calls `crate::gemini_browser::send_single_prompt(...)` for browser-backed Prompt Pack stages and already has a concrete `browser_run_id` before starting the browser future.

The first migration must not change the TypeScript API or UI behavior.

## Target State

- `gemini_bridge_send_single` and Prompt Pack browser runtime still call `send_single_prompt(...)`.
- `send_single_prompt(...)` still returns `AppResult<GeminiBrowserRunResult>`.
- `send_single_prompt(...)` writes the queued run log record, pushes a real Apalis SQLite job, emits the queued event, then waits for worker completion through the registered waiter receiver.
- Apalis has one Gemini Browser worker with concurrency `1`.
- The worker writes the same `GeminiBrowserRun` records and emits the same `GeminiBrowserRunEvent` payloads.
- `GeminiBrowserState` keeps active run id, cancellation token, and sidecar process state. It stops owning the `VecDeque` after the Apalis worker path is proven.
- Automatic retry is disabled for Gemini Browser jobs in this pilot. Browser submissions are not safe enough for automatic replay.
- The single-concurrency worker has a hard execution timeout so a hung Chrome/sidecar task cannot block later queued jobs forever.
- `gemini_bridge_status(...)` remains responsive while a browser job is running, even when the sidecar mutex is occupied by `send_single`.

## Storage Decisions

- Store Apalis SQLite tables in the existing main application database: `extractum.db`, the same database registered by `tauri_plugin_sql::Builder::default().add_migrations("sqlite:extractum.db", build_migrations())`.
- Do not create `gemini-browser/jobs.sqlite` or any other separate Apalis database file.
- Do not hand-design Apalis internal queue tables. Let `apalis-sqlite` create or manage its own schema through its supported storage initialization API against `extractum.db`.
- Reuse the main database identity from `src-tauri/src/db.rs`. Add shared `APP_IDENTIFIER`, `DB_FILENAME`, and config-directory path helpers there if needed instead of creating Gemini-Browser-local database constants.
- Prefer constructing Apalis storage from the existing `sqlx::SqlitePool` returned by `crate::db::get_pool(handle)` if the proven `apalis-sqlite` API supports that. If it only accepts a URL or path, open an Apalis storage connection to the same `extractum.db` file and document that choice in the helper.
- If `apalis-sqlite` does not expose a stable schema initialization API for an existing SQLite database, stop the migration and document the blocker before adding copied SQL to Extractum migrations.
- App migrations still own Extractum product tables. Apalis owns its internal queue schema, but it now lives physically in `extractum.db`.
- Keep the file-backed run log in `base_dir(handle)?.join("runs")` as the product projection. Apalis rows are queue implementation details.
- Persist app-visible state in the run log, not by reading Apalis SQL rows in UI commands.
- Queue name / worker name: `gemini-browser`.
- Job type name: `gemini_browser.run.v1`.
- Job idempotency key: the Gemini Browser `run_id`.
- Retry policy: every pushed task must use `TaskBuilder::attempts(1)` or the exact `apalis-sqlite` equivalent proven by tests. Context7 Apalis docs state that `attempts(n)` allows `n - 1` retries, so `attempts(1)` means one total attempt and zero retries.
- Dependency version policy: pin Apalis crates exactly to `=1.0.0-rc.9` until a stable `1.x` release is verified in this repo. Context7 shows current Apalis docs naming `1.0.0-rc.9`; plain `"1"` may fail because Cargo does not opt into pre-release versions unless the requirement includes the pre-release.

Current Context7 Apalis docs show SQL task rows with fields such as `job`, `id`, `job_type`, `status`, `attempts`, `max_attempts`, `run_at`, `last_result`, `lock_at`, `lock_by`, `done_at`, `priority`, `metadata`, and `idempotency_key`. The plan uses that only as orientation; implementation must not depend on hand-written SQL against those internal fields except in an isolated integration test that verifies actual serialized status values.

## Completion Waiter Contract

Create `GeminiBrowserJobRuntime` as managed Tauri state:

```rust
pub(crate) struct GeminiBrowserJobRuntime {
    waiters: parking_lot::Mutex<
        std::collections::HashMap<
            String,
            tokio::sync::oneshot::Sender<crate::error::AppResult<GeminiBrowserRunResult>>,
        >,
    >,
    cancelled_runs: parking_lot::Mutex<std::collections::HashSet<String>>,
    worker_status: tokio::sync::watch::Sender<GeminiBrowserWorkerStatus>,
    waiter_timeout: std::time::Duration,
    worker_execution_timeout: std::time::Duration,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum GeminiBrowserWorkerStatus {
    Starting,
    Ready { started_at: String },
    Failed { started_at: Option<String>, error: String },
}
```

Rules:

- Production runtime uses `waiter_timeout = std::time::Duration::from_secs(20 * 60)`.
- Production runtime uses `worker_execution_timeout = std::time::Duration::from_secs(20 * 60)`.
- Tests may construct runtime with a shorter timeout through `GeminiBrowserJobRuntime::new_for_test(timeout)`.
- `waiters` and `cancelled_runs` use synchronous locks because their critical sections only insert, remove, or clone values and must not call `.await`.
- Do not hold any `parking_lot::Mutex`, `std::sync::Mutex`, or `parking_lot::RwLock` guard across `.await`. Remove/clone the needed value, let the guard drop, then await.
- `send_single_prompt(...)` checks `worker_status` before writing a queued run log record. If status is `Starting`, it waits up to `5` seconds for `Ready` or `Failed`. If status is still `Starting` after that timeout, it returns before enqueue with `"Gemini Browser worker is still starting"`.
- If status is `Failed`, `send_single_prompt(...)` returns before enqueue with the stored worker error.
- `send_single_prompt(...)` rejects duplicate active `run_id` before registering a waiter. A duplicate means either an existing waiter for that `run_id` or a non-terminal run log record for that `run_id`.
- `send_single_prompt(...)` registers a waiter before pushing the Apalis job.
- If enqueue fails, `send_single_prompt(...)` removes the waiter and converts the just-created queued run log record to terminal `Failed` with message `"Gemini Browser job enqueue failed: {error}"`.
- `wait_for_registered_result(run_id, receiver)` waits on the registered receiver with the runtime timeout.
- On timeout, remove the waiter and return `AppError::internal("Gemini Browser job timed out waiting for worker result")`.
- On `oneshot::error::RecvError`, remove the waiter and return `AppError::internal("Gemini Browser worker channel closed unexpectedly")`.
- The worker always writes a terminal run log record before completing a waiter.
- If no waiter exists because the app restarted or the caller already timed out, the worker still writes the run log and emits events.
- If worker startup fails, new `send_single_prompt(...)` calls fail before enqueue with a clear internal error.
- If the app restarts after enqueue, there is no in-memory waiter to satisfy. The restarted worker still processes pending Apalis jobs and repairs the run log to terminal state.

## Cancellation Contract

- `GeminiBrowserJobRuntime::request_cancel(run_id)` records the run id in `cancelled_runs` for the current process.
- Cancellation must also be durable: `cancel_gemini_browser_job(...)` writes a cancelled terminal run log result when the run is still queued.
- Queued cancellation: if the job has not started, `cancel_gemini_browser_job(...)` writes `GeminiBrowserRunStatus::Cancelled`, emits a cancelled event, completes the waiter with a cancelled result, and leaves the Apalis job for the worker to acknowledge.
- Worker queued-cancel acknowledgement: before `mark_running(...)`, the worker reads the run log. If the run is already terminal `Cancelled`, the worker returns success to Apalis without calling the sidecar.
- Active cancellation: if `GeminiBrowserState::active_run_id()` equals `run_id`, call `GeminiBrowserState::request_stop()` and a helper inside the `gemini_browser` module that internally calls `super::sidecar::stop(...)`; worker writes a cancelled terminal result after the sidecar stops or returns a cancellation-shaped result.
- Prompt Pack cancellation: `run_browser_llm_request(...)` must call a new Gemini Browser cancel helper with the already-known `browser_run_id`, not only the one-slot `browser_state.request_stop()`.
- Manual Settings stop: `gemini_bridge_stop(...)` remains an active-run stop. It also records cancellation for the current active run id when one exists.
- Run log on cancellation uses `GeminiBrowserRunResult { run_id: run_id.to_string(), status: GeminiBrowserRunStatus::Cancelled, text: None, message: Some("Cancelled".to_string()), manual_action: None, artifacts: GeminiBrowserArtifactRefs::default(), elapsed_ms, debug_summary: None }`.

## State Reconciliation Contract

The file-backed run log remains the product projection, but it must not silently diverge from Apalis queue state. Reconciliation runs at worker startup and at worker job entry.

Task 1 must classify Apalis queue inspection capability:

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ApalisQueueInspectionMode {
    Supported,
    DegradedRunLogOnly,
}
```

`Supported` means the implementation has a typed or safely wrapped query path that can answer whether a `run_id` has an Apalis job and what verified status it has. `DegradedRunLogOnly` means no stable query API is available; in that mode startup reconciliation never claims a queued run is missing from Apalis storage, and worker-entry reconciliation remains the safety gate.

Startup policy:

- Run log `Queued` with an Apalis pending/queued job: leave as `Queued`.
- Run log `Queued` with an Apalis `running` job but no matching `GeminiBrowserState::active_run_id()`: mark `Failed` with message `"Gemini Browser queue state was running without an active sidecar"`. This check runs only in `Supported` inspection mode.
- Run log `Queued` with no matching Apalis job: mark `Failed` with message `"Gemini Browser queued job was missing from Apalis storage"`. This check runs only in `Supported` inspection mode.
- Run log `Queued` in `DegradedRunLogOnly` mode: leave as `Queued` at startup. The worker-entry policy will handle the job if Apalis later yields it.
- Run log `Running` with no active sidecar run in `GeminiBrowserState`: mark `Failed` with message `"Gemini Browser worker was interrupted before completion"`.
- Run log `Running` with Apalis terminal failed/killed state: mark `Failed` or `Cancelled` to match the verified Apalis terminal state and preserve Apalis `last_result` in `GeminiBrowserRunResult.message` when available. This check runs only in `Supported` inspection mode.
- Run log terminal status: leave unchanged.

Worker-entry policy:

- Run log missing for an Apalis job: acknowledge the Apalis job without sidecar execution and store an error in Apalis `last_result` if the backend supports it.
- Run log terminal `Cancelled`: acknowledge the Apalis job without sidecar execution.
- Run log terminal success/failure/manual-action: acknowledge the Apalis job without sidecar execution.
- Run log `Queued`: mark `Running` and execute sidecar.
- Run log `Running`: continue only if `GeminiBrowserState::active_run_id()` is the same `run_id`; otherwise mark `Failed` with message `"Gemini Browser run was running without an active sidecar"`.

Do not use Apalis SQL internals as a source of truth for UI state. Apalis internals may be inspected only in reconciliation tests and status serialization probes.

## Status Responsiveness Contract

`gemini_bridge_status(...)` must not wait behind a long-running `send_single` sidecar request.

- Add a lightweight cached status snapshot to Gemini Browser runtime/state.
- Worker updates the snapshot when a job is queued, running, terminal, cancelled, or failed.
- `provider_status(...)` may try a live sidecar status call, but it must use a short timeout of `250` milliseconds.
- If the sidecar mutex is busy or the timeout fires, return the cached snapshot with current `active_run_id` and queue depth.
- Status tests must prove that a simulated long-running sidecar operation does not block `provider_status(...)` longer than the timeout budget.

## Queue Stall Protection Contract

Apalis worker concurrency is intentionally `1`, so every production Gemini Browser worker must have a hard execution timeout in addition to caller-side waiter timeouts.

- Production worker execution timeout is `20` minutes.
- Tests must inject shorter worker execution timeouts.
- The timeout must be implemented with Tower timeout middleware (`tower::timeout::TimeoutLayer` or `ServiceBuilder::timeout(...)`) in the worker service path.
- On timeout, the product-facing run log must be terminal `Failed`, the waiter must be completed when still present, the failed run event must be emitted, and the worker must continue to the next queued job.
- A waiter timeout alone is insufficient because it does not stop a hung sidecar future inside the single worker.

## File Map

- Modify `src-tauri/Cargo.toml`
  - Add exact Apalis RC pins: `apalis = "=1.0.0-rc.9"` and `apalis-sqlite = "=1.0.0-rc.9"`.
- Modify `src-tauri/src/gemini_browser/mod.rs`
  - Expose the new jobs module, runtime, enqueue helper, cancel helper, and worker bootstrap.
- Create `src-tauri/src/gemini_browser/jobs.rs`
  - Define `GeminiBrowserJob`, Apalis storage initialization, `GeminiBrowserJobRuntime`, enqueue, completion waiters, cancellation helpers, reconciliation, worker handler, worker bootstrap, no-retry tests, and Apalis status verification tests.
- Modify `src-tauri/src/gemini_browser/state.rs`
  - Remove `VecDeque` only after the Apalis path is proven.
  - Keep active run, cancellation token, sidecar process state, and a status snapshot that can be read without waiting behind the sidecar mutex.
- Modify `src-tauri/src/gemini_browser/commands.rs`
  - Change `send_single_prompt(...)` from inline execution to Apalis enqueue plus completion wait.
  - Change `gemini_bridge_stop(...)` to record cancellation for the active run.
  - Keep `gemini_bridge_status(...)` responsive with cached status fallback.
- Modify `src-tauri/src/gemini_browser/run_log.rs`
  - Keep existing behavior; add small helpers for reconciliation and durable cancellation only when worker reuse needs them.
- Modify `src-tauri/src/prompt_packs/runtime.rs`
  - Replace generic one-slot browser stop on Prompt Pack cancellation with cancellation by concrete `browser_run_id`.
  - Add queued and active browser-stage cancellation tests.
- Modify `src-tauri/src/lib.rs`
  - Manage `GeminiBrowserJobRuntime` and start the Apalis-backed worker during app setup.
- Test `src-tauri/src/gemini_browser/jobs.rs`
  - Payload serialization, real Apalis SQLite push, real fake-worker smoke test, waiter behavior, cancellation behavior, no-auto-retry policy, and actual Apalis status serialization.
- Test existing Gemini Browser and Prompt Pack runtime modules
  - Confirm the public command and Prompt Pack handoff remain stable.

---

### Task 1: Pin Dependencies And Prove Real Apalis SQLite Storage

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Create: `src-tauri/src/gemini_browser/jobs.rs`
- Modify: `src-tauri/src/gemini_browser/mod.rs`
- Test: `src-tauri/src/gemini_browser/jobs.rs`

- [ ] **Step 1: Add queue runtime dependencies**

Modify `src-tauri/Cargo.toml` dependencies:

```toml
apalis = "=1.0.0-rc.9"
apalis-sqlite = "=1.0.0-rc.9"
parking_lot = "0.12"
tower = { version = "0.5", features = ["timeout", "util"] }
```

- [ ] **Step 2: Create the real job payload**

Create `src-tauri/src/gemini_browser/jobs.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct GeminiBrowserJob {
    pub run_id: String,
    pub prompt: String,
    pub source: String,
    pub artifact_mode: String,
}

#[cfg(test)]
mod tests {
    use super::GeminiBrowserJob;

    #[test]
    fn gemini_browser_job_serializes_queue_payload() {
        let job = GeminiBrowserJob {
            run_id: "run-1".to_string(),
            prompt: "hello".to_string(),
            source: "settings_test".to_string(),
            artifact_mode: "reduced".to_string(),
        };

        let json = serde_json::to_string(&job).expect("serialize job");
        let decoded: GeminiBrowserJob = serde_json::from_str(&json).expect("decode job");

        assert_eq!(decoded, job);
    }
}
```

- [ ] **Step 3: Wire the module**

Modify `src-tauri/src/gemini_browser/mod.rs`:

```rust
mod jobs;
```

- [ ] **Step 4: Add a real Apalis SQLite smoke test**

Add a Tokio test in `jobs.rs` named `apalis_sqlite_storage_pushes_and_worker_processes_one_job`.

The test must:

- create a temp directory with `tempfile`;
- create a temp main database file named `extractum.db` in that temp directory;
- apply the existing Extractum app migrations to that temp `extractum.db` through the same migration helper used by other database tests;
- create an Apalis SQLite storage using the current `apalis-sqlite` API against that same temp `extractum.db`;
- push one `GeminiBrowserJob` through `apalis::prelude::TaskSink::push(...)`;
- run a real `apalis::prelude::WorkerBuilder` worker with `.concurrency(1)`;
- use a fake handler that sends the processed `run_id` over a Tokio oneshot channel and calls `WorkerContext::stop()?`;
- wrap worker execution in `tokio::time::timeout(std::time::Duration::from_secs(5), worker.run())`;
- assert that the processed run id is `"run-apalis-smoke"`.

The test must not use `Vec`, `VecDeque`, or a test-only queue facade as the queue under test.
The test must fail with a timeout error instead of hanging if the worker does not stop.

- [ ] **Step 5: Run the smoke test**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib apalis_sqlite_storage_pushes_and_worker_processes_one_job
```

Expected: PASS. Do not continue to Task 2 until this proves real Apalis SQLite storage and a real worker can process one job.

- [ ] **Step 6: Record the exact storage constructor in code**

Keep the final compiling storage setup in a helper such as:

```rust
async fn open_gemini_browser_job_storage(
    main_db: GeminiBrowserApalisDbTarget,
) -> crate::error::AppResult<GeminiBrowserApalisStorage> {
    // Use the exact apalis-sqlite storage type proven by the smoke test.
}
```

`GeminiBrowserApalisDbTarget` can be an existing `sqlx::SqlitePool` from `crate::db::get_pool(handle)` or a path/URL target for the same `extractum.db`, depending on the stable constructor proven in Step 4. `GeminiBrowserApalisStorage` may be a concrete type alias or a small wrapper around the concrete storage type required by `apalis-sqlite`.

- [ ] **Step 7: Record Apalis queue inspection capability**

Add a compile-tested helper:

```rust
pub(crate) fn apalis_queue_inspection_mode() -> ApalisQueueInspectionMode {
    // Return Supported only after proving a concrete query path for run_id/status.
    // Otherwise return DegradedRunLogOnly.
}
```

If `Supported`, add a test that queries a pushed job by `run_id` and reads its verified status. If `DegradedRunLogOnly`, add a test that asserts startup reconciliation leaves queued run log records unchanged and relies on worker-entry reconciliation.

---

### Task 2: Define Runtime State, Main Database Storage, And Queue Contract

**Files:**
- Modify: `src-tauri/src/db.rs`
- Modify: `src-tauri/src/gemini_browser/jobs.rs`
- Modify: `src-tauri/src/gemini_browser/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/migrations.rs`
- Test: `src-tauri/src/gemini_browser/jobs.rs`

- [ ] **Step 1: Add main database helper test**

Add a pure helper and test:

```rust
#[test]
fn apalis_storage_uses_shared_main_extractum_db_identity() {
    let config_dir = std::path::PathBuf::from("config");
    assert_eq!(crate::db::APP_IDENTIFIER, "org.ai.extractum");
    assert_eq!(crate::db::DB_FILENAME, "extractum.db");
    assert_eq!(crate::db::DB_URL, "sqlite:extractum.db");
    assert_eq!(
        crate::db::db_path_from_config_dir(&config_dir),
        config_dir.join("org.ai.extractum").join("extractum.db")
    );
}
```

- [ ] **Step 2: Centralize the main database helper**

Extend the existing `src-tauri/src/db.rs` `DB_URL` constant into the shared database identity:

```rust
pub const APP_IDENTIFIER: &str = "org.ai.extractum";
pub const DB_FILENAME: &str = "extractum.db";
pub const DB_URL: &str = "sqlite:extractum.db";

pub(crate) fn db_path_from_config_dir(config_dir: &std::path::Path) -> std::path::PathBuf {
    config_dir.join(APP_IDENTIFIER).join(DB_FILENAME)
}

pub(crate) fn app_config_db_path() -> Option<std::path::PathBuf> {
    dirs::config_dir().map(|dir| db_path_from_config_dir(&dir))
}
```

`src-tauri/src/migrations.rs` must reuse `crate::db::app_config_db_path()` instead of keeping a private copy of the database location. The Tauri wrapper must use the same helper when a path-based Apalis constructor is required. If the storage constructor can use the already-initialized pool, use `crate::db::get_pool(handle)` instead. Do not derive the Apalis database path from `gemini_browser::paths::base_dir(...)`; that directory remains only for profile, artifact, and run-log files.

- [ ] **Step 3: Add runtime state**

Add `GeminiBrowserJobRuntime`:

```rust
pub(crate) struct GeminiBrowserJobRuntime {
    waiters: parking_lot::Mutex<
        std::collections::HashMap<
            String,
            tokio::sync::oneshot::Sender<crate::error::AppResult<crate::gemini_browser::GeminiBrowserRunResult>>,
        >,
    >,
    cancelled_runs: parking_lot::Mutex<std::collections::HashSet<String>>,
    worker_status: tokio::sync::watch::Sender<GeminiBrowserWorkerStatus>,
    waiter_timeout: std::time::Duration,
    worker_execution_timeout: std::time::Duration,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum GeminiBrowserWorkerStatus {
    Starting,
    Ready { started_at: String },
    Failed { started_at: Option<String>, error: String },
}

impl Default for GeminiBrowserJobRuntime {
    fn default() -> Self {
        let (worker_status, _) = tokio::sync::watch::channel(GeminiBrowserWorkerStatus::Starting);
        Self {
            waiters: parking_lot::Mutex::new(std::collections::HashMap::new()),
            cancelled_runs: parking_lot::Mutex::new(std::collections::HashSet::new()),
            worker_status,
            waiter_timeout: std::time::Duration::from_secs(20 * 60),
            worker_execution_timeout: std::time::Duration::from_secs(20 * 60),
        }
    }
}
```

`waiters` and `cancelled_runs` must use synchronous locks because their critical sections do not perform async work. Do not hold either guard across `.await`; remove or clone the sender/cancellation bit first, drop the guard, then await if needed.

- [ ] **Step 4: Add worker readiness tests**

Add tests:

```rust
#[tokio::test]
async fn worker_status_blocks_enqueue_when_startup_failed() {
    let runtime = GeminiBrowserJobRuntime::new_for_test(std::time::Duration::from_secs(1));
    runtime.mark_worker_failed("storage open failed").await;

    let error = runtime.ensure_worker_ready_for_enqueue().await.expect_err("worker failed");

    assert!(error.to_string().contains("storage open failed"));
}

#[tokio::test]
async fn worker_status_allows_enqueue_after_ready() {
    let runtime = GeminiBrowserJobRuntime::new_for_test(std::time::Duration::from_secs(1));
    runtime.mark_worker_ready("2026-06-22T00:00:00Z".to_string()).await;

    runtime.ensure_worker_ready_for_enqueue().await.expect("worker ready");
}

#[tokio::test]
async fn worker_status_times_out_while_starting() {
    let runtime = GeminiBrowserJobRuntime::new_for_test(std::time::Duration::from_secs(1));

    let error = runtime
        .ensure_worker_ready_for_enqueue_with_timeout(std::time::Duration::from_millis(1))
        .await
        .expect_err("still starting");

    assert!(error.to_string().contains("worker is still starting"));
}
```

- [ ] **Step 5: Manage runtime state**

Modify `src-tauri/src/lib.rs` next to `GeminiBrowserState::new()`:

```rust
.manage(GeminiBrowserJobRuntime::default())
```

Export it from `mod.rs`:

```rust
pub(crate) use jobs::GeminiBrowserJobRuntime;
```

- [ ] **Step 6: Add real enqueue helpers**

Add a lower-level helper that can be used without a Tauri `AppHandle`:

```rust
async fn enqueue_gemini_browser_job_to_storage(
    storage: &mut GeminiBrowserApalisStorage,
    job: GeminiBrowserJob,
) -> crate::error::AppResult<QueuedGeminiBrowserJob> {
    // Build the configured Apalis task and push it into the provided storage.
}
```

Add the Tauri wrapper `enqueue_gemini_browser_job(...)` that:

- opens or clones the real Apalis SQLite storage proven in Task 1;
- delegates to `enqueue_gemini_browser_job_to_storage(...)`;
- translates duplicate idempotency-key / unique-constraint failures for `run_id` into `AppError::conflict("Gemini Browser job with this run_id is already queued or running")`;
- returns `QueuedGeminiBrowserJob { run_id, queue_position: None }`;
- never executes the sidecar inline.

The helper may return `queue_position: None` because Apalis SQL queue depth is not part of the product contract in this pilot.

- [ ] **Step 7: Add duplicate idempotency conflict test**

Add a test named `enqueue_duplicate_run_id_returns_conflict`.

The test must:

- create a temp main `extractum.db`;
- apply existing Extractum app migrations to that temp database;
- open Apalis storage against that same temp `extractum.db`;
- call `enqueue_gemini_browser_job_to_storage(...)` with `run_id = "run-duplicate-idempotency"`;
- call `enqueue_gemini_browser_job_to_storage(...)` again with the same `run_id`;
- assert the second result is `Err`;
- assert `error.kind == crate::error::AppErrorKind::Conflict`;
- assert `error.message == "Gemini Browser job with this run_id is already queued or running"`.

Implement a narrow mapper used by both the storage helper and the Tauri wrapper:

```rust
fn map_enqueue_error(_run_id: &str, error: impl std::fmt::Display) -> crate::error::AppError {
    let message = error.to_string().to_ascii_lowercase();
    if message.contains("unique constraint")
        || message.contains("constraint failed")
        || message.contains("idempotency")
        || message.contains("duplicate")
        || message.contains("already exists")
    {
        return crate::error::AppError::conflict(
            "Gemini Browser job with this run_id is already queued or running",
        );
    }

    crate::error::AppError::internal(format!("Gemini Browser job enqueue failed: {error}"))
}
```

If the final `apalis-sqlite` error type exposes a typed database error such as `sqlx::Error::Database`, prefer checking `database_error.is_unique_violation()` or the SQLite error code before falling back to message matching. Keep the test above either way.

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib enqueue_duplicate_run_id_returns_conflict
```

Expected: PASS.

- [ ] **Step 8: Test real enqueue persists a job before worker startup**

Add an integration-style unit test that:

- creates a temp main `extractum.db`;
- applies existing Extractum app migrations to that temp database;
- opens Apalis storage against that same temp `extractum.db`;
- calls `enqueue_gemini_browser_job_to_storage(...)`;
- starts the fake worker after enqueue;
- asserts the fake worker receives the enqueued `run_id`.

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib enqueue_persists_job_before_worker_startup
```

Expected: PASS.

---

### Task 3: Implement Completion Waiters

**Files:**
- Modify: `src-tauri/src/gemini_browser/jobs.rs`
- Test: `src-tauri/src/gemini_browser/jobs.rs`

- [ ] **Step 1: Add waiter success test**

Add:

```rust
#[tokio::test]
async fn waiter_receives_terminal_worker_result() {
    let runtime = GeminiBrowserJobRuntime::new_for_test(std::time::Duration::from_secs(1));
    let receiver = runtime.register_waiter("run-waiter-1").expect("register waiter");
    let result = crate::gemini_browser::GeminiBrowserRunResult {
        run_id: "run-waiter-1".to_string(),
        status: crate::gemini_browser::GeminiBrowserRunStatus::Ok,
        text: Some("answer".to_string()),
        message: Some("done".to_string()),
        manual_action: None,
        artifacts: crate::gemini_browser::GeminiBrowserArtifactRefs::default(),
        elapsed_ms: 10,
        debug_summary: None,
    };

    runtime.complete_waiter("run-waiter-1", Ok(result.clone()));

    assert_eq!(receiver.await.expect("waiter open").expect("worker result"), result);
}
```

- [ ] **Step 2: Add waiter timeout cleanup test**

Add:

```rust
#[tokio::test]
async fn wait_for_result_removes_waiter_on_timeout() {
    let runtime = GeminiBrowserJobRuntime::new_for_test(std::time::Duration::from_millis(1));
    let receiver = runtime.register_waiter("run-timeout").expect("register waiter");
    let error = runtime
        .wait_for_registered_result("run-timeout", receiver)
        .await
        .expect_err("timeout error");

    assert!(error.to_string().contains("timed out waiting for worker result"));
    assert!(!runtime.has_waiter_for_test("run-timeout"));
}
```

- [ ] **Step 3: Add closed-channel waiter test**

Add:

```rust
#[tokio::test]
async fn wait_for_result_removes_waiter_when_worker_channel_closes() {
    let runtime = GeminiBrowserJobRuntime::new_for_test(std::time::Duration::from_secs(1));
    let receiver = runtime
        .register_waiter("run-channel-closed")
        .expect("register waiter");

    runtime.remove_waiter("run-channel-closed");

    let error = runtime
        .wait_for_registered_result("run-channel-closed", receiver)
        .await
        .expect_err("closed channel error");

    assert!(error
        .to_string()
        .contains("Gemini Browser worker channel closed unexpectedly"));
    assert!(!runtime.has_waiter_for_test("run-channel-closed"));
}
```

- [ ] **Step 4: Add duplicate waiter test**

Add:

```rust
#[tokio::test]
async fn register_waiter_rejects_duplicate_run_id() {
    let runtime = GeminiBrowserJobRuntime::new_for_test(std::time::Duration::from_secs(1));
    let _first = runtime.register_waiter("run-duplicate").expect("first waiter");

    let error = runtime
        .register_waiter("run-duplicate")
        .expect_err("duplicate waiter");

    assert!(error.to_string().contains("already has an active Gemini Browser waiter"));
}
```

- [ ] **Step 5: Implement waiter methods**

Implement:

```rust
impl GeminiBrowserJobRuntime {
    pub(crate) fn register_waiter(
        &self,
        run_id: &str,
    ) -> crate::error::AppResult<
        tokio::sync::oneshot::Receiver<crate::error::AppResult<crate::gemini_browser::GeminiBrowserRunResult>>
    >;

    pub(crate) fn complete_waiter(
        &self,
        run_id: &str,
        result: crate::error::AppResult<crate::gemini_browser::GeminiBrowserRunResult>,
    );

    pub(crate) fn remove_waiter(&self, run_id: &str);

    pub(crate) async fn wait_for_registered_result(
        &self,
        run_id: &str,
        receiver: tokio::sync::oneshot::Receiver<
            crate::error::AppResult<crate::gemini_browser::GeminiBrowserRunResult>
        >,
    ) -> crate::error::AppResult<crate::gemini_browser::GeminiBrowserRunResult>;

    pub(crate) fn new_for_test(waiter_timeout: std::time::Duration) -> Self;

    pub(crate) fn new_for_test_with_timeouts(
        waiter_timeout: std::time::Duration,
        worker_execution_timeout: std::time::Duration,
    ) -> Self;

    pub(crate) fn worker_execution_timeout(&self) -> std::time::Duration;

    pub(crate) async fn ensure_worker_ready_for_enqueue(&self) -> crate::error::AppResult<()>;

    async fn ensure_worker_ready_for_enqueue_with_timeout(
        &self,
        timeout: std::time::Duration,
    ) -> crate::error::AppResult<()>;
}
```

`Default` must set both `waiter_timeout` and `worker_execution_timeout` to `std::time::Duration::from_secs(20 * 60)` and worker status to `Starting`. Tests must use `new_for_test(...)` or `new_for_test_with_timeouts(...)` with short timeouts. `ensure_worker_ready_for_enqueue()` must use a `5` second startup timeout in production.

- [ ] **Step 6: Run waiter tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib waiter_
```

Expected: PASS.

---

### Task 4: Implement Worker Handler Core With Current Result Types

**Files:**
- Modify: `src-tauri/src/gemini_browser/jobs.rs`
- Test: `src-tauri/src/gemini_browser/jobs.rs`

- [ ] **Step 1: Add handler contract test with a fake executor**

Add:

```rust
#[tokio::test]
async fn worker_handler_marks_run_running_and_terminal() {
    let temp = tempfile::tempdir().expect("temp dir");
    let runtime = GeminiBrowserJobRuntime::new_for_test(std::time::Duration::from_secs(1));
    let receiver = runtime.register_waiter("run-worker-1").expect("register waiter");
    let events = std::sync::Arc::new(parking_lot::Mutex::new(Vec::<String>::new()));
    let job = GeminiBrowserJob {
        run_id: "run-worker-1".to_string(),
        prompt: "hello".to_string(),
        source: "settings_test".to_string(),
        artifact_mode: "reduced".to_string(),
    };

    crate::gemini_browser::create_queued_run(
        temp.path(),
        &job.run_id,
        &job.source,
        &job.prompt,
    )
    .expect("create queued run");

    let result = run_job_with_executor_for_test(
        temp.path(),
        &runtime,
        job,
        events.clone(),
        || async {
            Ok(crate::gemini_browser::GeminiBrowserRunResult {
                run_id: "run-worker-1".to_string(),
                status: crate::gemini_browser::GeminiBrowserRunStatus::Ok,
                text: Some("answer".to_string()),
                message: Some("done".to_string()),
                manual_action: None,
                artifacts: crate::gemini_browser::GeminiBrowserArtifactRefs::default(),
                elapsed_ms: 10,
                debug_summary: None,
            })
        },
    )
    .await
    .expect("run job");

    assert_eq!(result.status, crate::gemini_browser::GeminiBrowserRunStatus::Ok);
    assert_eq!(receiver.await.expect("waiter open").expect("worker result"), result);

    let runs = crate::gemini_browser::list_runs(temp.path(), 10)
        .expect("list runs")
        .runs;
    assert_eq!(runs[0].status, crate::gemini_browser::GeminiBrowserRunStatus::Ok);
    assert_eq!(events.lock().as_slice(), ["running", "ok"]);
}
```

- [ ] **Step 2: Implement testable worker core**

Add a test helper that mirrors production worker behavior without requiring a Tauri `AppHandle`:

```rust
#[cfg(test)]
async fn run_job_with_executor_for_test<F, Fut>(
    runs_dir: &std::path::Path,
    runtime: &GeminiBrowserJobRuntime,
    job: GeminiBrowserJob,
    events: std::sync::Arc<parking_lot::Mutex<Vec<String>>>,
    executor: F,
) -> crate::error::AppResult<crate::gemini_browser::GeminiBrowserRunResult>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<
        Output = crate::error::AppResult<crate::gemini_browser::GeminiBrowserRunResult>,
    >,
{
    crate::gemini_browser::mark_running(runs_dir, &job.run_id)?;
    events.lock().push("running".to_string());

    let result = executor().await?;
    crate::gemini_browser::finish_run(runs_dir, &job.run_id, result.clone())?;
    events.lock().push(format!("{:?}", result.status).to_lowercase());
    runtime.complete_waiter(&job.run_id, Ok(result.clone()));
    Ok(result)
}
```

- [ ] **Step 3: Run handler test**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib worker_handler_marks_run_running_and_terminal
```

Expected: PASS.

---

### Task 5: Verify Actual Apalis Status Serialization

**Files:**
- Modify: `src-tauri/src/gemini_browser/jobs.rs`
- Test: `src-tauri/src/gemini_browser/jobs.rs`

- [ ] **Step 1: Add a status serialization probe**

Add a test named `apalis_sqlite_status_probe_documents_actual_status_values`.

The test must:

- push one job to a temp SQLite Apalis storage;
- inspect status after push;
- run a fake worker that blocks long enough to inspect the in-flight status;
- inspect status while the worker is processing;
- let the fake worker complete successfully;
- inspect status after completion;
- push a second job whose fake handler returns an error;
- inspect status after the failed handler is terminal;
- assert the observed queued, running, completed, and failed values against the actual values produced by `apalis-sqlite`;
- include a short code comment with the observed values.

This test may query Apalis SQL internals only because it is a probe that protects this migration plan from status-name drift.

- [ ] **Step 2: Replace status mapping guesses with verified values**

Implement `run_status_for_queue_state(state: &str)` only after Step 1 is passing. It must map the actual observed Apalis SQL values to:

```rust
GeminiBrowserRunStatus::Queued
GeminiBrowserRunStatus::Running
GeminiBrowserRunStatus::Ok
GeminiBrowserRunStatus::Failed
```

Do not hardcode `"done"` or `"completed"` until the probe test proves the real value for this dependency version. Do not map Apalis `Killed` to `GeminiBrowserRunStatus::Cancelled` unless an implementation task proves Apalis emits `Killed` for this pilot. In the initial pilot, `GeminiBrowserRunStatus::Cancelled` comes from the durable run log cancellation path.

- [ ] **Step 3: Run status tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib apalis_sqlite_status_probe_documents_actual_status_values
```

Expected: PASS.

---

### Task 6: Enforce No Automatic Retry

**Files:**
- Modify: `src-tauri/src/gemini_browser/jobs.rs`
- Test: `src-tauri/src/gemini_browser/jobs.rs`

- [ ] **Step 1: Add no-retry task construction test**

Add a test named `gemini_browser_jobs_are_built_with_one_total_attempt`.

The test must:

- build a `GeminiBrowserJob` through the same helper production enqueue uses to create an Apalis task;
- assert the task uses max attempts `1`, using the current Apalis task metadata API;
- assert the idempotency key or task id contains the Gemini Browser `run_id`.

Use the current Apalis API proven in Task 1. Context7 Apalis docs describe `TaskBuilder::attempts(n)` as total attempts where `n = 1` means zero retries.

- [ ] **Step 2: Implement a production task builder helper**

Add:

```rust
fn build_gemini_browser_task(job: GeminiBrowserJob) -> GeminiBrowserApalisTask {
    // Use the concrete TaskBuilder type proven in Task 1.
    // Required settings:
    // - job type/name: "gemini_browser.run.v1"
    // - id or idempotency key: job.run_id
    // - attempts: 1
}
```

`enqueue_gemini_browser_job_to_storage(...)` must push the task returned by this helper. It must not push the raw payload if that bypasses attempts/idempotency configuration.

- [ ] **Step 3: Add failing-handler no-retry integration test**

Add a test named `failed_gemini_browser_job_is_not_retried`.

The test must:

- push one job through the production task builder;
- start a real Apalis worker with a fake handler that increments `AtomicUsize` and returns an error;
- wait until Apalis marks the job terminal failed or the worker stops;
- assert the execution count is exactly `1`;
- inspect task metadata or SQL row and assert max attempts is `1` when the backend exposes it.

- [ ] **Step 4: Run no-retry tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib retry
```

Expected: PASS.

---

### Task 7: Start A Real Worker During App Setup

**Files:**
- Modify: `src-tauri/src/gemini_browser/jobs.rs`
- Modify: `src-tauri/src/gemini_browser/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/gemini_browser/jobs.rs`

- [ ] **Step 1: Add worker bootstrap**

Implement:

```rust
pub(crate) async fn start_gemini_browser_job_worker(
    handle: tauri::AppHandle,
) -> crate::error::AppResult<()> {
    // Open the Apalis SQLite storage against the main extractum.db database.
    // Prefer crate::db::get_pool(&handle) if the proven constructor supports it;
    // otherwise use crate::db::app_config_db_path() for the same file.
    // Build a WorkerBuilder named "gemini-browser".
    // Set concurrency to 1.
    // Install a Tower timeout layer using runtime.worker_execution_timeout().
    // Build the production Gemini Browser job handler.
    // Mark GeminiBrowserJobRuntime worker status Ready after storage and worker construction succeed.
    // Mark worker status Failed before returning any startup error.
    // Run the worker future until Tauri shutdown.
    // If worker.run() returns Err or exits before shutdown, mark worker status Failed.
}
```

This function must not return `Ok(())` without starting a worker.
If `worker.run()` exits before application shutdown, record diagnostic message `"Gemini Browser job worker stopped unexpectedly"` unless the returned error has a more specific message.

- [ ] **Step 2: Worker handler calls the existing sidecar path**

The production handler must:

- check queued cancellation before marking running;
- call `mark_running(...)`;
- call `GeminiBrowserState::start_run(...)`;
- emit `GeminiBrowserRunStatus::Running`;
- call `sidecar::send_single(...)` with the same profile and artifact paths as the old command path;
- convert sidecar errors through `sidecar::sidecar_unavailable_result(...)`;
- call `finish_run(...)`;
- call `GeminiBrowserState::finish_run(...)`;
- complete the waiter;
- emit the terminal event.

- [ ] **Step 3: Add worker execution timeout layer**

Use Tower timeout middleware when building the worker service:

```rust
let timeout = runtime.worker_execution_timeout();
let timeout_layer = tower::timeout::TimeoutLayer::new(timeout);

let worker = apalis::prelude::WorkerBuilder::new("gemini-browser")
    .backend(storage)
    .concurrency(1)
    .layer(timeout_layer)
    .build(handler);
```

Context7 Tower docs show that timeout middleware returns an error that can be downcast to `tower::timeout::error::Elapsed` when the deadline expires.

Do not rely on the outer worker middleware alone to update product-facing state. The worker handler must run sidecar execution through a timeout-aware helper that finalizes the run log before returning:

```rust
async fn run_job_with_execution_timeout<Fut>(
    handle: &tauri::AppHandle,
    runtime: &GeminiBrowserJobRuntime,
    state: &crate::gemini_browser::GeminiBrowserState,
    runs_dir: &std::path::Path,
    job: GeminiBrowserJob,
    sidecar_future: Fut,
) -> crate::error::AppResult<crate::gemini_browser::GeminiBrowserRunResult>
where
    Fut: std::future::Future<
        Output = crate::error::AppResult<crate::gemini_browser::GeminiBrowserRunResult>,
    >,
{
    match tokio::time::timeout(runtime.worker_execution_timeout(), sidecar_future).await {
        Ok(result) => result,
        Err(_elapsed) => finish_timed_out_job(handle, runtime, state, runs_dir, job).await,
    }
}
```

`finish_timed_out_job(...)` must:

- stop the active Gemini Browser sidecar if this job started it;
- write a terminal `GeminiBrowserRunStatus::Failed` run log result with message `"Gemini Browser job timed out after {seconds}s"`;
- complete the waiter with the same failed result when the waiter still exists;
- emit the failed run event;
- let the Apalis worker continue to the next queued job.

Use this shape:

```rust
async fn finish_timed_out_job(
    handle: &tauri::AppHandle,
    runtime: &GeminiBrowserJobRuntime,
    state: &crate::gemini_browser::GeminiBrowserState,
    runs_dir: &std::path::Path,
    job: GeminiBrowserJob,
) -> crate::error::AppResult<crate::gemini_browser::GeminiBrowserRunResult>;
```

Do not rely on waiter timeout alone. The waiter timeout only protects the caller; the worker timeout protects the single-concurrency queue.
The outer `WorkerBuilder::layer(TimeoutLayer::new(...))` remains required as a hard guard around the whole worker service. The timeout-aware helper is required so product-facing run log and waiter state are finalized deterministically.

- [ ] **Step 4: Add worker timeout release test**

Add a test named `worker_timeout_marks_run_failed_and_processes_next_job`.

The test must:

- construct `GeminiBrowserJobRuntime::new_for_test_with_timeouts(std::time::Duration::from_secs(1), std::time::Duration::from_millis(25))`;
- create queued run log records for `run-timeout-first` and `run-timeout-second`;
- run a real worker with `.concurrency(1)` and the same Tower timeout layer used in production;
- make the first fake handler wait forever with `std::future::pending::<crate::error::AppResult<crate::gemini_browser::GeminiBrowserRunResult>>()`;
- make the second fake handler return `GeminiBrowserRunStatus::Ok`;
- assert the first run log is terminal `Failed` with message containing `"timed out"`;
- assert the first waiter receives the same terminal failed result;
- assert the second job is processed after the first timeout;
- assert the second run log is terminal `Ok`.

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib worker_timeout_marks_run_failed_and_processes_next_job
```

Expected: PASS.

- [ ] **Step 5: Add worker startup failure test**

Add a test named `worker_startup_failure_marks_runtime_failed`.

The test must:

- force storage initialization to fail with a bad path or injected failing opener;
- call the worker bootstrap core;
- assert `GeminiBrowserJobRuntime` status is `Failed`;
- assert `ensure_worker_ready_for_enqueue()` returns an error containing the startup failure.

- [ ] **Step 6: Add worker run failure test**

Add a test named `worker_run_failure_marks_runtime_failed`.

The test must:

- start the worker bootstrap core with a fake worker future that returns `Err("worker loop failed")`;
- assert `GeminiBrowserJobRuntime` status becomes `Failed`;
- assert `ensure_worker_ready_for_enqueue()` returns an error containing `"worker loop failed"`.

- [ ] **Step 7: Export bootstrap and runtime**

In `mod.rs`:

```rust
pub(crate) use jobs::{
    cancel_gemini_browser_job, enqueue_gemini_browser_job,
    start_gemini_browser_job_worker, GeminiBrowserJobRuntime,
};
```

- [ ] **Step 8: Spawn worker during setup**

In `src-tauri/src/lib.rs`, after state is managed and inside setup:

```rust
let worker_handle = app.handle().clone();
tauri::async_runtime::spawn(async move {
    if let Err(error) = gemini_browser::start_gemini_browser_job_worker(worker_handle).await {
        eprintln!("Failed to start Gemini Browser job worker: {error}");
    }
});
```

- [ ] **Step 9: Run worker tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib gemini_browser::jobs
```

Expected: PASS.

---

### Task 8: Keep Status UI Responsive During Worker Execution

**Files:**
- Modify: `src-tauri/src/gemini_browser/state.rs`
- Modify: `src-tauri/src/gemini_browser/commands.rs`
- Modify: `src-tauri/src/gemini_browser/jobs.rs`
- Test: `src-tauri/src/gemini_browser/commands.rs`
- Test: `src-tauri/src/gemini_browser/state.rs`

- [ ] **Step 1: Add cached status snapshot type**

Add a cached status field to `GeminiBrowserState` or `GeminiBrowserJobRuntime`:

```rust
status_snapshot: parking_lot::RwLock<GeminiBrowserProviderStatus>,
```

Use `parking_lot::RwLock` because status reads are frequent and the snapshot update path does not require async work. Do not hold the read or write guard across `.await`.

The initial snapshot must use:

```rust
GeminiBrowserProviderStatus {
    status: GeminiBrowserProviderStatusKind::NotStarted,
    manual_action: None,
    active_run_id: None,
    queue_depth: 0,
    browser_profile_dir: path_string(&profile_dir(handle)?),
    latest_message: Some("Gemini browser sidecar is not running.".to_string()),
}
```

- [ ] **Step 2: Add snapshot update helpers**

Add helpers:

```rust
pub(crate) fn update_status_snapshot(
    &self,
    update: impl FnOnce(&mut GeminiBrowserProviderStatus),
);

pub(crate) fn status_snapshot(&self) -> GeminiBrowserProviderStatus;
```

Worker and command code must update the snapshot on queued, running, terminal, cancelled, failed, and manual-action states.

- [ ] **Step 3: Add non-blocking status test**

Add a test named `provider_status_uses_cached_snapshot_when_sidecar_is_busy`.

The test must:

- arrange a fake sidecar/status provider that never returns or sleeps longer than one second;
- set a cached snapshot with `status: GeminiBrowserProviderStatusKind::Running` and `active_run_id: Some("run-busy".to_string())`;
- call the status core with timeout `std::time::Duration::from_millis(25)`;
- assert it returns in less than `200` milliseconds;
- assert the returned status is the cached `Running` status.

- [ ] **Step 4: Refactor `provider_status(...)`**

Change `provider_status(...)` so it:

- reads `active_run_id` and Apalis queue depth without sidecar mutex blocking;
- tries `sidecar::status(...)` with `tokio::time::timeout(std::time::Duration::from_millis(250), ...)`;
- updates the cached snapshot on live status success;
- returns cached status on timeout or busy sidecar.

Do not hold the sidecar mutex while reading Apalis queue depth or formatting the cached response.

- [ ] **Step 5: Run status responsiveness tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib provider_status_
```

Expected: PASS.

---

### Task 9: Refactor `send_single_prompt(...)` To Enqueue And Wait

**Files:**
- Modify: `src-tauri/src/gemini_browser/commands.rs`
- Modify: `src-tauri/src/gemini_browser/jobs.rs`
- Test: `src-tauri/src/gemini_browser/commands.rs`
- Test: `src-tauri/src/gemini_browser/jobs.rs`

- [ ] **Step 1: Add command handoff test**

Add a test around a small core function, not a request-shape-only test. The function should accept:

- runs directory;
- `GeminiBrowserJobRuntime`;
- fake enqueue function;
- `GeminiBrowserRunRequest`.

The test must assert:

- `create_queued_run(...)` writes a queued run log record before enqueue;
- fake enqueue receives exactly the same `run_id`, `prompt`, `source`, and `artifact_mode`;
- the returned queued event uses `GeminiBrowserRunStatus::Queued`.

- [ ] **Step 2: Add duplicate run id and enqueue failure tests**

Add tests:

```rust
#[tokio::test]
async fn send_single_prompt_rejects_duplicate_non_terminal_run_id_before_enqueue() {
    // Arrange an existing queued run log record for run-duplicate.
    // Call the command core with the same run_id and a fake enqueue that panics if called.
    // Assert the error mentions duplicate Gemini Browser run_id.
}

#[tokio::test]
async fn send_single_prompt_marks_run_failed_when_enqueue_fails() {
    // Arrange a fake enqueue that returns AppError::internal("push failed").
    // Assert the waiter is removed.
    // Assert the run log terminal status is Failed with message containing
    // "Gemini Browser job enqueue failed: push failed".
}
```

- [ ] **Step 3: Refactor command flow**

In `send_single_prompt(...)`, keep input trimming and validation. Before `create_queued_run(...)`, check worker readiness and reject duplicate active `run_id`. Then create the queued run log record. Replace direct `state.enqueue(...)`, `state.pop_next(...)`, and inline `sidecar::send_single(...)` with:

```rust
let runtime = handle.state::<crate::gemini_browser::GeminiBrowserJobRuntime>();
runtime.ensure_worker_ready_for_enqueue().await?;
reject_duplicate_non_terminal_run(&runs_root, &request.run_id).await?;
create_queued_run(&runs_root, &request.run_id, &request.source, &request.prompt)?;
let waiter = runtime.register_waiter(&request.run_id)?;
let queued = match crate::gemini_browser::enqueue_gemini_browser_job(
    handle,
    GeminiBrowserJob {
        run_id: request.run_id.clone(),
        prompt: request.prompt.clone(),
        source: request.source.clone(),
        artifact_mode: request.artifact_mode.clone(),
    },
)
.await
{
    Ok(queued) => queued,
    Err(error) => {
        runtime.remove_waiter(&request.run_id);
        let failed = GeminiBrowserRunResult {
            run_id: request.run_id.clone(),
            status: GeminiBrowserRunStatus::Failed,
            text: None,
            message: Some(format!("Gemini Browser job enqueue failed: {error}")),
            manual_action: None,
            artifacts: GeminiBrowserArtifactRefs::default(),
            elapsed_ms: 0,
            debug_summary: None,
        };
        finish_run(&runs_root, &request.run_id, failed.clone())?;
        emit_run_event(handle, GeminiBrowserRunEvent {
            run_id: request.run_id.clone(),
            status: GeminiBrowserRunStatus::Failed,
            message: failed.message.clone(),
            queue_position: None,
        });
        return Err(error);
    }
};
```

If `enqueue_gemini_browser_job(...)` returns an error after the queued run log record is written, the `Err` branch must:

- remove the waiter;
- call `finish_run(...)` with `GeminiBrowserRunStatus::Failed`;
- use message `"Gemini Browser job enqueue failed: {error}"`;
- emit a terminal failed run event;
- return the original enqueue error.

Emit the queued event with `queued.queue_position`.

- [ ] **Step 4: Wait for terminal result**

After enqueue:

```rust
runtime.wait_for_registered_result(&request.run_id, waiter).await
```

The waiter must be removed on timeout, worker error, dropped sender, and success.

- [ ] **Step 5: Verify command behavior**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib send_single_prompt_
```

Expected: PASS.

---

### Task 10: Implement Cancellation By Browser Run Id

**Files:**
- Modify: `src-tauri/src/gemini_browser/jobs.rs`
- Modify: `src-tauri/src/gemini_browser/commands.rs`
- Modify: `src-tauri/src/prompt_packs/runtime.rs`
- Test: `src-tauri/src/gemini_browser/jobs.rs`
- Test: `src-tauri/src/prompt_packs/runtime.rs`

- [ ] **Step 1: Add queued cancellation test**

Test:

- create queued run log record;
- register waiter;
- call `GeminiBrowserJobRuntime::request_cancel("run-cancel-queued")`;
- call `cancel_gemini_browser_job(...)`;
- run worker handler against the still-present Apalis job;
- assert it does not call the fake executor;
- assert run log status is `Cancelled`;
- assert waiter receives a `GeminiBrowserRunResult` with `status: Cancelled`.

- [ ] **Step 2: Add active cancellation test**

Test:

- start active run through `GeminiBrowserState::start_run("run-cancel-active".to_string())`;
- start the worker handler with a fake sidecar executor that blocks until stop is requested;
- call `cancel_gemini_browser_job(...)`;
- assert the active cancellation token is cancelled;
- unblock the fake sidecar executor and let the worker path finish;
- assert the worker writes a terminal `GeminiBrowserRunResult` with `GeminiBrowserRunStatus::Cancelled`.

`cancel_gemini_browser_job(...)` must not be expected to write the active terminal result directly. For active jobs it requests stop; the worker remains responsible for final run log state after the sidecar path returns.

- [ ] **Step 3: Implement cancel helper**

Implement `cancel_gemini_browser_job(...)` with these branches:

- always record `runtime.request_cancel(run_id)`;
- if `GeminiBrowserState::active_run_id()` equals `run_id`, request stop and stop the sidecar;
- otherwise read the run log record from `runs_dir(handle)?`;
- if the run log status is `Queued`, write a cancelled `GeminiBrowserRunResult`, emit a cancelled event, and complete the waiter with that result;
- if the run log is already terminal, leave it unchanged and return `Ok(())`;
- if the run log is `Running` but state does not report this active run, mark it `Failed` with message `"Gemini Browser run was running without an active sidecar"`.

Shape:

```rust
pub(crate) async fn cancel_gemini_browser_job(
    handle: &tauri::AppHandle,
    run_id: &str,
) -> crate::error::AppResult<()> {
    let runtime = handle.state::<GeminiBrowserJobRuntime>();
    runtime.request_cancel(run_id);

    let state = handle.state::<crate::gemini_browser::GeminiBrowserState>();
    if state.active_run_id().await.as_deref() == Some(run_id) {
        state.request_stop().await;
        stop_active_gemini_browser_sidecar(handle, &state).await?;
    } else {
        // Read the run log and handle queued, terminal, or stale-running state.
    }

    Ok(())
}
```

`stop_active_gemini_browser_sidecar(...)` must live in `src-tauri/src/gemini_browser/jobs.rs` or `commands.rs`, where it can call the private sibling module as `super::sidecar::stop(...)`. Do not require `pub(crate) mod sidecar` unless a later review chooses to expose that module deliberately.

- [ ] **Step 4: Update Prompt Pack cancellation**

In `run_browser_llm_request(...)`, keep the existing `browser_run_id` local. On `LlmRequestError::Cancelled`, call:

```rust
crate::gemini_browser::cancel_gemini_browser_job(&handle, &browser_run_id).await?;
```

Then return `YoutubeSummaryStageExecutionError::Cancelled`.

In the actual implementation, map the `AppError` explicitly:

```rust
crate::gemini_browser::cancel_gemini_browser_job(&handle, &browser_run_id)
    .await
    .map_err(YoutubeSummaryStageExecutionError::Failed)?;
```

- [ ] **Step 5: Update manual stop**

In `gemini_bridge_stop(...)`, read `state.active_run_id().await`. If present, call `cancel_gemini_browser_job(&handle, &run_id).await`. If no active run exists, keep the current stop behavior.

- [ ] **Step 6: Run cancellation tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib cancel
```

Expected: PASS.

---

### Task 11: Verify Prompt Pack Browser Cancellation Path

**Files:**
- Modify: `src-tauri/src/prompt_packs/runtime.rs`
- Modify: `src-tauri/src/gemini_browser/jobs.rs`
- Test: `src-tauri/src/prompt_packs/runtime.rs`
- Test: `src-tauri/src/gemini_browser/jobs.rs`

- [ ] **Step 1: Add queued browser-stage cancellation test**

Add a test named `prompt_pack_browser_stage_cancelled_while_queued_cancels_browser_job`.

The test must:

- create a Prompt Pack browser stage with a known `run_id`, `stage_run_id`, and derived `browser_run_id`;
- enqueue the Gemini Browser job without starting the Apalis worker;
- cancel the Prompt Pack run through the existing `PromptPackRunState` cancellation token;
- assert `cancel_gemini_browser_job(&handle, &browser_run_id)` is called through a test seam or fake browser runtime;
- assert the Gemini Browser run log terminal status is `GeminiBrowserRunStatus::Cancelled`;
- assert the Prompt Pack stage/run path returns `YoutubeSummaryStageExecutionError::Cancelled`;
- assert browser provenance does not record `browser_run_status = 'ok'`.

- [ ] **Step 2: Add active browser-stage cancellation test**

Add a test named `prompt_pack_browser_stage_cancelled_while_active_stops_sidecar`.

The test must:

- create a browser-backed Prompt Pack stage;
- start the Gemini Browser worker with a fake sidecar executor that blocks on a oneshot channel;
- wait until `GeminiBrowserState::active_run_id()` equals the derived `browser_run_id`;
- cancel the Prompt Pack run;
- assert the active browser cancellation token is cancelled;
- assert sidecar stop was requested through the fake executor;
- assert the Gemini Browser run log terminal status is `GeminiBrowserRunStatus::Cancelled`;
- assert the Prompt Pack stage/run returns `YoutubeSummaryStageExecutionError::Cancelled`.

- [ ] **Step 3: Add provenance guard test**

Add a test named `cancelled_browser_stage_does_not_persist_success_provenance`.

The test must:

- run the browser-stage cancellation path;
- query `prompt_pack_stage_runs`;
- assert `browser_run_status` is either `NULL` or `'cancelled'`;
- assert it is never `'ok'`, `'ready'`, or a partial-success status after cancellation.

- [ ] **Step 4: Run Prompt Pack cancellation tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib prompt_pack_browser_stage_cancelled
```

Expected: PASS.

---

### Task 12: Crash And Restart Recovery

**Files:**
- Modify: `src-tauri/src/gemini_browser/jobs.rs`
- Modify: `src-tauri/src/gemini_browser/run_log.rs`
- Test: `src-tauri/src/gemini_browser/jobs.rs`

- [ ] **Step 1: Add pending-after-restart test**

Test:

- create queued run log record;
- push Apalis job;
- drop the first runtime without starting worker;
- create a new runtime and start fake worker against the same temp SQLite file;
- assert the job is processed and run log becomes terminal.

- [ ] **Step 2: Add run log versus Apalis reconciliation matrix tests**

Add tests for this matrix:

- run log `Queued` with matching Apalis queued job remains `Queued`;
- run log `Queued` with Apalis `running` but no active sidecar becomes `Failed` with message `"Gemini Browser queue state was running without an active sidecar"`;
- run log `Queued` with no matching Apalis job becomes `Failed` with message `"Gemini Browser queued job was missing from Apalis storage"`;
- run log `Running` with no active worker becomes `Failed` with message `"Gemini Browser worker was interrupted before completion"`;
- run log `Running` with Apalis terminal failed/killed state becomes a matching terminal run log record;
- run log terminal `Cancelled` with a remaining Apalis job is left terminal and worker acknowledges without sidecar;
- missing run log for an Apalis job is acknowledged without sidecar execution.

The Apalis-state-dependent startup cases run only when `apalis_queue_inspection_mode() == ApalisQueueInspectionMode::Supported`. Add a separate `DegradedRunLogOnly` test that proves queued run log records are left unchanged at startup when no stable Apalis query API is available.

- [ ] **Step 3: Implement startup and worker-entry reconciliation**

At worker startup:

- scan the Gemini Browser run log for non-terminal `Running` runs;
- mark them `Failed` with a current `GeminiBrowserRunResult`;
- if `apalis_queue_inspection_mode() == Supported`, compare queued run log records with Apalis jobs and mark queued records without an Apalis job as `Failed`;
- if `apalis_queue_inspection_mode() == DegradedRunLogOnly`, leave queued run log records unchanged at startup.

At worker job entry:

- read the run log before `mark_running(...)`;
- skip sidecar execution for terminal run log statuses;
- skip sidecar execution and acknowledge missing run log jobs;
- only execute sidecar for `Queued` records;
- fail stale `Running` records that do not match `GeminiBrowserState::active_run_id()`.

- [ ] **Step 4: Run recovery tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib restart
```

Expected: PASS.

---

### Task 13: Remove The Old `VecDeque` Queue

**Files:**
- Modify: `src-tauri/src/gemini_browser/state.rs`
- Modify: `src-tauri/src/gemini_browser/commands.rs`
- Test: `src-tauri/src/gemini_browser/state.rs`

- [ ] **Step 1: Delete queue fields and methods**

Remove from `GeminiBrowserState`:

```rust
queue: Mutex<VecDeque<GeminiBrowserRunRequest>>,
pub async fn enqueue(...)
pub async fn pop_next(...)
pub async fn queue_depth(...)
```

Keep:

```rust
active_run_id: Mutex<Option<String>>,
cancellation: Mutex<Option<CancellationToken>>,
sidecar: Mutex<Option<super::sidecar::GeminiBrowserSidecarProcess>>,
```

- [ ] **Step 2: Replace provider status queue depth source**

In `gemini_bridge_status(...)`, keep using `GeminiBrowserProviderStatus.queue_depth`. Return queue depth from a real Apalis-backed helper only if the helper is proven by tests; otherwise return `0` in this pilot and rely on queued run events for per-request `queue_position`. Do not read `VecDeque`.

- [ ] **Step 3: Update state test**

Replace `queue_tracks_depth_and_active_run` with:

```rust
#[tokio::test]
async fn state_tracks_active_run_and_cancellation() {
    let state = GeminiBrowserState::new();
    let token = state.start_run("run-1".to_string()).await;
    assert!(!token.is_cancelled());
    assert_eq!(state.active_run_id().await, Some("run-1".to_string()));
    assert!(state.request_stop().await);
    assert!(token.is_cancelled());
    state.finish_run("run-1").await;
    assert_eq!(state.active_run_id().await, None);
}
```

- [ ] **Step 4: Run state tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib state_tracks_active_run_and_cancellation
```

Expected: PASS.

---

### Task 14: Compatibility Verification

**Files:**
- No code changes unless verification reveals a bug.

- [ ] **Step 1: Rust Gemini Browser verification**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib gemini_browser
```

Expected: PASS.

- [ ] **Step 2: Prompt Pack browser handoff verification**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib prompt_packs
```

Expected: PASS.

- [ ] **Step 3: Targeted architectural regression verification**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib retry
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib worker_timeout_marks_run_failed_and_processes_next_job
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib wait_for_result_removes_waiter_when_worker_channel_closes
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib enqueue_duplicate_run_id_returns_conflict
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib provider_status_
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib prompt_pack_browser_stage_cancelled
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib restart
```

Expected: PASS for no-retry, worker timeout release, closed waiter channels, duplicate enqueue conflicts, status responsiveness, Prompt Pack browser cancellation, and reconciliation/restart tests.

- [ ] **Step 4: TypeScript Gemini Browser verification**

Run:

```powershell
npm.cmd run test -- src/lib/api/gemini-browser.test.ts src/lib/gemini-browser-run-inspector.test.ts src/lib/gemini-browser-setup-status.test.ts src/lib/gemini-browser-provider-panel-state.test.ts
```

Expected: PASS.

- [ ] **Step 5: Svelte/type verification**

Run:

```powershell
npm.cmd run check
```

Expected: PASS.

- [ ] **Step 6: Diff hygiene**

Run:

```powershell
git diff --check
git status --short
```

Expected: `git diff --check` exits `0`. `git status --short` shows only intentional files for this migration plus unrelated pre-existing changes.

---

## Self-Review

- Spec coverage: The plan covers exact Apalis RC dependency pins, real early Apalis integration, fail-fast worker smoke tests, SQLite storage ownership, explicit runtime construction, synchronous in-memory waiter/cancellation/status locks, worker readiness/error/early-exit state, receiver-based completion waiter semantics, closed waiter channel handling, duplicate `run_id` rejection, duplicate Apalis idempotency conflict translation, storage-injected enqueue tests, enqueue failure cleanup, worker execution timeout and queue release, cancellation continuity, Prompt Pack cancellation by `browser_run_id`, run log versus Apalis reconciliation, status UI responsiveness, no automatic retry, crash/restart recovery, current `GeminiBrowserRunResult` fields, and removal of the old `VecDeque`.
- Placeholder scan: The plan intentionally avoids placeholder markers, fake production facades, and no-op worker bootstrap. The only API discovery point is constrained to Task 1 and must compile before later tasks proceed.
- Type consistency: Result examples use the current `GeminiBrowserRunResult` shape with `run_id`, `text`, `manual_action`, `GeminiBrowserArtifactRefs::default()`, and `elapsed_ms`.
- Apalis status consistency: The plan requires a real `apalis-sqlite` status probe before mapping queue statuses.
- Retry consistency: The plan requires `attempts(1)` or the verified Apalis equivalent plus an execution-count test proving one failed job is not retried.
- Query consistency: The plan requires Task 1 to classify Apalis queue inspection as `Supported` or `DegradedRunLogOnly` before reconciliation tasks rely on Apalis internals.
- Waiter consistency: The plan uses `register_waiter(...) -> Receiver` plus `wait_for_registered_result(...)`; there is no result wait that tries to recover a receiver from only `run_id`.
- Closed-channel consistency: `wait_for_registered_result(...)` must remove the waiter and return `"Gemini Browser worker channel closed unexpectedly"` on `oneshot::error::RecvError`.
- Idempotency consistency: duplicate Apalis idempotency / SQLite unique failures are translated to `AppError::conflict`, with a concrete duplicate enqueue test.
- Lock consistency: Runtime maps use `parking_lot::Mutex`, cached provider status uses `parking_lot::RwLock`, and plan text forbids holding synchronous lock guards across `.await`.
- Timeout consistency: Caller waiter timeout and worker execution timeout are separate; the worker timeout has a dedicated test proving a timed-out first job does not block the next queued job.

## Execution Choice

When implementing this plan in the main development thread, choose one:

1. **Subagent-Driven** - Use a fresh implementation worker per task and review between tasks.
2. **Inline Execution** - Execute tasks in the current session with checkpoints.

For this repository, Inline Execution is acceptable because the first pilot is tightly scoped to Gemini Browser.
