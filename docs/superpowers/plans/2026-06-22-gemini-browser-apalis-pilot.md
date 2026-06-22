# Gemini Browser Apalis Pilot Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the Gemini Browser in-memory queue with a real Apalis-backed SQLite job queue while keeping the current Tauri commands, events, run log, Prompt Pack integration, and UI behavior stable.

**Architecture:** Apalis owns durable queue storage and single-worker execution. Extractum keeps the existing file-backed Gemini Browser run log as the product-facing projection for Settings, run history, Prompt Pack provenance, and diagnostics. `send_single_prompt(...)` remains synchronous from the caller perspective by enqueueing a durable job and waiting on the receiver returned when that specific run was registered.

**Tech Stack:** Rust, Tauri 2, Tokio, `apalis = "=1.0.0-rc.8"`, `apalis-sqlite = "=1.0.0-rc.8"`, `tower` timeout middleware, `parking_lot`, serde, existing Gemini Browser sidecar, existing file-backed run log.

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
- Apalis dependencies are pinned to the verified `1.0.0-rc.8` pre-release because Cargo will not select pre-releases from a plain `"1"` requirement.
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
- If Apalis uses a separate SQLite connection to the same file, the connection options must be compatible with the app pool: create-if-missing policy through the shared DB path, busy timeout, foreign keys, and journal mode/WAL consistency.
- If `apalis-sqlite` does not expose a stable schema initialization API for an existing SQLite database, stop the migration and document the blocker before adding copied SQL to Extractum migrations.
- App migrations still own Extractum product tables. Apalis owns its internal queue schema, but it now lives physically in `extractum.db`.
- Keep the file-backed run log in `base_dir(handle)?.join("runs")` as the product projection. Apalis rows are queue implementation details.
- Persist app-visible state in the run log, not by reading Apalis SQL rows in UI commands.
- Queue name / worker name: `gemini-browser`.
- Job type name: `gemini_browser.run.v1`.
- Job idempotency key: the Gemini Browser `run_id`.
- Retry policy: every pushed task must use `TaskBuilder::attempts(1)` or the exact `apalis-sqlite` equivalent proven by tests. Context7 Apalis docs state that `attempts(n)` allows `n - 1` retries, so `attempts(1)` means one total attempt and zero retries.
- Dependency version policy: pin Apalis crates exactly to `=1.0.0-rc.8` until a stable `1.x` release is verified in this repo. Cargo currently resolves `apalis-sqlite` pre-releases up to `1.0.0-rc.8`; plain `"1"` may fail because Cargo does not opt into pre-release versions unless the requirement includes the pre-release.

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
    worker_hard_guard_timeout: std::time::Duration,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum GeminiBrowserWorkerStatus {
    Starting,
    Ready { started_at: String },
    Failed { started_at: Option<String>, error: String },
}
```

Rules:

- Production runtime uses `worker_execution_timeout = std::time::Duration::from_secs(20 * 60)` for product-facing sidecar execution timeout and run-log finalization.
- Production runtime uses `waiter_timeout = worker_execution_timeout + std::time::Duration::from_secs(5)`, so synchronous callers can receive the worker's product-facing terminal result before caller-side timeout.
- Production runtime uses `worker_hard_guard_timeout = worker_execution_timeout + std::time::Duration::from_secs(15)` for the outer Tower worker service guard.
- Tests may construct runtime with a shorter timeout through `GeminiBrowserJobRuntime::new_for_test(timeout)`. That constructor treats `timeout` as the worker execution timeout, then sets `waiter_timeout = timeout + std::time::Duration::from_millis(50)` and `worker_hard_guard_timeout = timeout + std::time::Duration::from_millis(100)`.
- Tests that need race-sensitive timeout control must use `GeminiBrowserJobRuntime::new_for_test_with_timeouts(waiter_timeout, worker_execution_timeout, worker_hard_guard_timeout)`, which must reject or panic unless `worker_execution_timeout < waiter_timeout < worker_hard_guard_timeout`.
- Tests that only exercise waiter timeout cleanup, with no worker future running, must use `GeminiBrowserJobRuntime::new_for_waiter_timeout_test(waiter_timeout)` instead of weakening the production timeout ordering.
- `waiters` and `cancelled_runs` use synchronous locks because their critical sections only insert, remove, or clone values and must not call `.await`.
- Do not hold any `parking_lot::Mutex`, `std::sync::Mutex`, or `parking_lot::RwLock` guard across `.await`. Remove/clone the needed value, let the guard drop, then await.
- `send_single_prompt(...)` checks `worker_status` before writing a queued run log record. If status is `Starting`, it waits up to `5` seconds for `Ready` or `Failed` through a receiver created with `self.worker_status.subscribe()`. If status is still `Starting` after that timeout, it returns before enqueue with `"Gemini Browser worker is still starting"`.
- If status is `Failed`, `send_single_prompt(...)` returns before enqueue with the stored worker error.
- `send_single_prompt(...)` rejects duplicate active `run_id` before registering a waiter. A duplicate means either an existing waiter for that `run_id` or any existing run log record for that `run_id`, terminal or non-terminal. The pilot treats `run_id` as globally unique because it is also the Apalis idempotency key.
- `send_single_prompt(...)` registers a waiter before pushing the Apalis job.
- If enqueue fails, `send_single_prompt(...)` removes the waiter and converts the just-created queued run log record to terminal `Failed` with message `"Gemini Browser job enqueue failed: {error}"`.
- `wait_for_registered_result(run_id, receiver)` waits on the registered receiver with the runtime timeout.
- On timeout, remove the waiter and return `AppError::internal("Gemini Browser job timed out waiting for worker result")`.
- On `oneshot::error::RecvError`, remove the waiter and return `AppError::internal("Gemini Browser worker channel closed unexpectedly")`.
- The worker always writes a terminal run log record before completing a waiter.
- If no waiter exists because the app restarted or the caller already timed out, the worker still writes the run log and emits events.
- `complete_waiter(...)` removes the sender from the map before sending and ignores `sender.send(result)` failure with `let _ = ...`, because the receiver may have been dropped after caller timeout.
- If worker startup fails, new `send_single_prompt(...)` calls fail before enqueue with a clear internal error.
- If the app restarts after enqueue, there is no in-memory waiter to satisfy. The restarted worker still processes pending Apalis jobs and repairs the run log to terminal state.
- `mark_worker_ready(...)`, `mark_worker_failed(...)`, `request_cancel(...)`, `is_cancelled(...)`, and `clear_cancelled(...)` are synchronous runtime methods. `tokio::sync::watch::Sender::send(...)` does not require `.await`.

## Cancellation Contract

- `GeminiBrowserJobRuntime::request_cancel(run_id)` records the run id in `cancelled_runs` for the current process.
- `GeminiBrowserJobRuntime::is_cancelled(run_id)` is checked by the worker before `mark_running(...)`.
- `GeminiBrowserJobRuntime::clear_cancelled(run_id)` is called after a queued-cancel acknowledgement or any terminal worker result for that run id.
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

- In-memory `runtime.is_cancelled(run_id)`: write or preserve a terminal `Cancelled` result, complete any waiter with `Cancelled`, call `runtime.clear_cancelled(run_id)`, and acknowledge the Apalis job without sidecar execution.
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

- Production product-facing worker execution timeout is `20` minutes.
- Production caller waiter timeout is `20` minutes + `5` seconds, so normal worker timeout finalization can reach synchronous callers.
- Production outer Tower hard-guard timeout is `20` minutes + `15` seconds.
- Tests must inject shorter worker execution timeouts.
- Product-facing timeout must be implemented inside the worker handler with `tokio::time::timeout(worker_execution_timeout, sidecar_future)` so it can write terminal run log state.
- The outer timeout must be implemented with Tower timeout middleware (`tower::timeout::TimeoutLayer` or `ServiceBuilder::timeout(...)`) using `worker_hard_guard_timeout`, with ordering `worker_execution_timeout < waiter_timeout < worker_hard_guard_timeout`.
- On product-facing timeout, the run log must be terminal `Failed`, the waiter must be completed when still present, the failed run event must be emitted, and the worker must continue to the next queued job.
- A waiter timeout alone is insufficient because it does not stop a hung sidecar future inside the single worker.

## File Map

- Modify `src-tauri/Cargo.toml`
  - Add exact Apalis RC pins: `apalis = "=1.0.0-rc.8"` and `apalis-sqlite = "=1.0.0-rc.8"`.
- Modify `src-tauri/src/gemini_browser/mod.rs`
  - Expose the new jobs module, runtime, enqueue helper, and worker bootstrap early; expose the cancel helper only in Task 10 after the function exists.
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

- [x] **Step 1: Add queue runtime dependencies**

Modify `src-tauri/Cargo.toml` dependencies:

```toml
apalis = "=1.0.0-rc.8"
apalis-sqlite = "=1.0.0-rc.8"
parking_lot = "0.12"
tower = { version = "0.5", features = ["timeout", "util"] }
```

Do not add a second `tempfile` entry for these tests. `tempfile = "3"` already exists in `src-tauri/Cargo.toml` under `[dependencies]` because production YouTube code uses `NamedTempFile` / `TempDir`; moving it to `[dev-dependencies]` is not part of this Apalis pilot.

- [x] **Step 2: Create the real job payload**

Create `src-tauri/src/gemini_browser/jobs.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum GeminiBrowserArtifactMode {
    Reduced,
    Full,
}

impl GeminiBrowserArtifactMode {
    pub(crate) fn from_wire(value: Option<&str>) -> crate::error::AppResult<Self> {
        match value.unwrap_or("reduced") {
            "reduced" => Ok(Self::Reduced),
            "full" => Ok(Self::Full),
            other => Err(crate::error::AppError::validation(format!(
                "unsupported Gemini Browser artifact_mode '{other}'"
            ))),
        }
    }

    pub(crate) fn as_wire(&self) -> &'static str {
        match self {
            Self::Reduced => "reduced",
            Self::Full => "full",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct GeminiBrowserJob {
    pub run_id: String,
    pub prompt: String,
    pub source: String,
    pub artifact_mode: GeminiBrowserArtifactMode,
    pub browser_config: Option<crate::gemini_browser::GeminiBrowserProviderConfig>,
}

impl GeminiBrowserJob {
    pub(crate) fn run_request(&self) -> crate::gemini_browser::GeminiBrowserRunRequest {
        crate::gemini_browser::GeminiBrowserRunRequest {
            run_id: self.run_id.clone(),
            prompt: self.prompt.clone(),
            source: self.source.clone(),
            artifact_mode: self.artifact_mode.as_wire().to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{GeminiBrowserArtifactMode, GeminiBrowserJob};

    #[test]
    fn gemini_browser_job_serializes_queue_payload() {
        let job = GeminiBrowserJob {
            run_id: "run-1".to_string(),
            prompt: "hello".to_string(),
            source: "settings_test".to_string(),
            artifact_mode: GeminiBrowserArtifactMode::Reduced,
            browser_config: Some(crate::gemini_browser::GeminiBrowserProviderConfig {
                mode: crate::gemini_browser::GeminiBrowserProviderMode::CdpAttach,
                cdp_endpoint: Some("http://127.0.0.1:9222".to_string()),
            }),
        };

        let json = serde_json::to_string(&job).expect("serialize job");
        let decoded: GeminiBrowserJob = serde_json::from_str(&json).expect("decode job");

        assert_eq!(decoded, job);
        assert_eq!(decoded.artifact_mode.as_wire(), "reduced");
        assert_eq!(
            decoded
                .browser_config
                .as_ref()
                .and_then(|config| config.cdp_endpoint.as_deref()),
            Some("http://127.0.0.1:9222")
        );
    }
}
```

`GeminiBrowserJob` must be a complete execution snapshot. It stores the typed artifact mode and the optional `GeminiBrowserProviderConfig` captured at enqueue time. The command path must convert the incoming `GeminiBrowserRunRequest.artifact_mode` string through `GeminiBrowserArtifactMode::from_wire(...)` before constructing a job; do not pass a raw `String` through the queue payload. The worker must not re-read Settings, `GeminiBrowserState`, or frontend state to decide between managed and CDP execution, because those settings may change after enqueue but before execution.

Before adding the job payload, verify `src-tauri/src/gemini_browser/types.rs` keeps `GeminiBrowserProviderConfig` compatible with the job derives:

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeminiBrowserProviderConfig {
    pub mode: GeminiBrowserProviderMode,
    #[serde(alias = "cdpEndpoint")]
    pub cdp_endpoint: Option<String>,
}
```

- [x] **Step 3: Wire the module**

Modify `src-tauri/src/gemini_browser/mod.rs`:

```rust
mod jobs;
```

- [x] **Step 4: Add a real Apalis SQLite smoke test**

Add a Tokio test in `jobs.rs` named `apalis_sqlite_storage_pushes_and_worker_processes_one_job`.

The test must:

- create a temp directory with `tempfile`;
- create a temp main database file named `extractum.db` in that temp directory;
- apply the existing Extractum app migrations to that temp `extractum.db` through the same migration helper used by other database tests;
- record the current product table names from `sqlite_master`;
- create an Apalis SQLite storage using the current `apalis-sqlite` API against that same temp `extractum.db`;
- assert key product tables such as `prompt_pack_runs`, `prompt_pack_stage_runs`, `prompt_pack_versions`, and `projects` still exist after Apalis storage initialization;
- assert Apalis internal table names do not collide with any pre-existing product table name from before Apalis storage initialization;
- push one `GeminiBrowserJob` through `apalis::prelude::TaskSink::push(...)`;
- run a real `apalis::prelude::WorkerBuilder` worker with `.concurrency(1)`;
- use a fake handler that sends the processed `run_id` over a Tokio oneshot channel and calls `WorkerContext::stop()?`;
- wrap worker execution in `tokio::time::timeout(std::time::Duration::from_secs(5), worker.run())`;
- assert that the processed run id is `"run-apalis-smoke"`.

The test must not use `Vec`, `VecDeque`, or a test-only queue facade as the queue under test.
The test must fail with a timeout error instead of hanging if the worker does not stop.
Do not read `_sqlx_migrations` in this unit smoke test. The current `apply_all_migrations_for_test_pool(...)` helper executes migration SQL directly and does not populate migration history.

- [x] **Step 5: Run the smoke test**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib apalis_sqlite_storage_pushes_and_worker_processes_one_job
```

Expected: PASS. Do not continue to Task 2 until this proves real Apalis SQLite storage, non-conflicting Apalis tables inside `extractum.db`, preserved product tables, and a real worker processing one job.

- [x] **Step 6: Add seeded migration history preservation test**

Add a unit test named `apalis_storage_preserves_existing_sqlx_migration_history_table`.

The test must:

- create a temp main database file named `extractum.db`;
- open a normal `sqlx::SqlitePool` against it;
- create a minimal `_sqlx_migrations` table fixture compatible with SQLx migration history;
- insert one fake applied migration row;
- open Apalis SQLite storage against the same temp `extractum.db`;
- assert the fake `_sqlx_migrations` row still exists and is unchanged.

Use this fixture schema inside the test rather than `apply_all_migrations_for_test_pool(...)`:

```rust
sqlx::raw_sql(
    "CREATE TABLE _sqlx_migrations (
        version BIGINT PRIMARY KEY,
        description TEXT NOT NULL,
        installed_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
        success BOOLEAN NOT NULL,
        checksum BLOB NOT NULL,
        execution_time BIGINT NOT NULL
    )",
)
.execute(&pool)
.await?;
```

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib apalis_storage_preserves_existing_sqlx_migration_history_table
```

Expected: PASS.

- [x] **Step 7: Record the exact storage constructor in code**

Keep the final compiling storage setup in a helper such as:

```rust
async fn open_gemini_browser_job_storage(
    main_db: GeminiBrowserApalisDbTarget,
) -> crate::error::AppResult<GeminiBrowserApalisStorage> {
    // Use the exact apalis-sqlite storage type proven by the smoke test.
}
```

`GeminiBrowserApalisDbTarget` can be an existing `sqlx::SqlitePool` from `crate::db::get_pool(handle)` or a path/URL target for the same `extractum.db`, depending on the stable constructor proven in Step 4. `GeminiBrowserApalisStorage` may be a concrete type alias or a small wrapper around the concrete storage type required by `apalis-sqlite`.

If `apalis-sqlite` requires opening a separate connection to the same file, use the same SQLite connection policy as the app pool:

- create the file if missing only through the same main DB path helper;
- use a busy timeout of at least `5` seconds;
- enable foreign keys;
- use the same journal mode as the app database if the app pool exposes or sets one; prefer WAL if both pools can enable it consistently.

Add a smoke test named `apalis_storage_shares_extractum_db_without_locking_app_pool`.

The test must:

- create a temp `extractum.db`;
- open the normal app `sqlx::SqlitePool` against it;
- apply app migrations;
- open Apalis storage against the same file using the final constructor;
- insert or read a small product row through the app pool while Apalis storage exists;
- push one Apalis job while the app pool remains open;
- read product tables again through the app pool;
- fail if either side returns `"database is locked"`.

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib apalis_storage_shares_extractum_db_without_locking_app_pool
```

Expected: PASS before later tasks use the production storage helper.

- [x] **Step 7: Record Apalis queue inspection capability**

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

- [x] **Step 1: Add main database helper test**

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

This test references `crate::db::APP_IDENTIFIER`, `DB_FILENAME`, and `db_path_from_config_dir(...)`, which are introduced in Step 2. Write Step 1 and Step 2 in the same code edit before running tests or committing; do not leave the workspace between those steps in a non-compiling state.

- [x] **Step 2: Centralize the main database helper**

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

- [x] **Step 3: Add runtime state**

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
    worker_hard_guard_timeout: std::time::Duration,
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
            worker_execution_timeout: std::time::Duration::from_secs(20 * 60),
            waiter_timeout: std::time::Duration::from_secs(20 * 60 + 5),
            worker_hard_guard_timeout: std::time::Duration::from_secs(20 * 60 + 15),
        }
    }
}
```

`waiters` and `cancelled_runs` must use synchronous locks because their critical sections do not perform async work. Do not hold either guard across `.await`; remove or clone the sender/cancellation bit first, drop the guard, then await if needed.

- [x] **Step 4: Add worker readiness tests**

Add tests:

```rust
#[tokio::test]
async fn worker_status_blocks_enqueue_when_startup_failed() {
    let runtime = GeminiBrowserJobRuntime::new_for_test(std::time::Duration::from_secs(1));
    runtime.mark_worker_failed("storage open failed");

    let error = runtime.ensure_worker_ready_for_enqueue().await.expect_err("worker failed");

    assert!(error.to_string().contains("storage open failed"));
}

#[tokio::test]
async fn worker_status_allows_enqueue_after_ready() {
    let runtime = GeminiBrowserJobRuntime::new_for_test(std::time::Duration::from_secs(1));
    runtime.mark_worker_ready("2026-06-22T00:00:00Z".to_string());

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

- [x] **Step 5: Manage runtime state**

Modify `src-tauri/src/lib.rs` next to `GeminiBrowserState::new()`:

```rust
.manage(GeminiBrowserJobRuntime::default())
```

Export it from `mod.rs`:

```rust
pub(crate) use jobs::GeminiBrowserJobRuntime;
```

- [x] **Step 6: Add real enqueue helpers**

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

- [x] **Step 7: Add duplicate idempotency conflict test**

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

- [x] **Step 8: Test real enqueue persists a job before worker startup**

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

- [x] **Step 1: Add waiter success test**

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

- [x] **Step 2: Add waiter timeout cleanup test**

Add:

```rust
#[tokio::test]
async fn wait_for_result_removes_waiter_on_timeout() {
    let runtime =
        GeminiBrowserJobRuntime::new_for_waiter_timeout_test(std::time::Duration::from_millis(1));
    let receiver = runtime.register_waiter("run-timeout").expect("register waiter");
    let error = runtime
        .wait_for_registered_result("run-timeout", receiver)
        .await
        .expect_err("timeout error");

    assert!(error.to_string().contains("timed out waiting for worker result"));
    assert!(!runtime.has_waiter_for_test("run-timeout"));
}
```

- [x] **Step 3: Add closed-channel waiter test**

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

- [x] **Step 4: Add duplicate waiter test**

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

- [x] **Step 5: Add dropped receiver completion test**

Add:

```rust
#[tokio::test]
async fn complete_waiter_ignores_dropped_receiver() {
    let runtime = GeminiBrowserJobRuntime::new_for_test(std::time::Duration::from_secs(1));
    let receiver = runtime
        .register_waiter("run-dropped-receiver")
        .expect("register waiter");
    drop(receiver);

    let result = crate::gemini_browser::GeminiBrowserRunResult {
        run_id: "run-dropped-receiver".to_string(),
        status: crate::gemini_browser::GeminiBrowserRunStatus::Ok,
        text: Some("late answer".to_string()),
        message: Some("done".to_string()),
        manual_action: None,
        artifacts: crate::gemini_browser::GeminiBrowserArtifactRefs::default(),
        elapsed_ms: 10,
        debug_summary: None,
    };

    runtime.complete_waiter("run-dropped-receiver", Ok(result));

    assert!(!runtime.has_waiter_for_test("run-dropped-receiver"));
}
```

- [x] **Step 6: Implement waiter methods**

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

    pub(crate) fn has_waiter(&self, run_id: &str) -> bool;

    pub(crate) async fn wait_for_registered_result(
        &self,
        run_id: &str,
        receiver: tokio::sync::oneshot::Receiver<
            crate::error::AppResult<crate::gemini_browser::GeminiBrowserRunResult>
        >,
    ) -> crate::error::AppResult<crate::gemini_browser::GeminiBrowserRunResult>;

    pub(crate) fn new_for_test(timeout: std::time::Duration) -> Self;

    pub(crate) fn new_for_test_with_timeouts(
        waiter_timeout: std::time::Duration,
        worker_execution_timeout: std::time::Duration,
        worker_hard_guard_timeout: std::time::Duration,
    ) -> Self;

    pub(crate) fn new_for_waiter_timeout_test(waiter_timeout: std::time::Duration) -> Self;

    pub(crate) fn worker_execution_timeout(&self) -> std::time::Duration;

    pub(crate) fn waiter_timeout(&self) -> std::time::Duration;

    pub(crate) fn worker_hard_guard_timeout(&self) -> std::time::Duration;

    pub(crate) fn mark_worker_ready(&self, started_at: String);

    pub(crate) fn mark_worker_failed(&self, error: impl Into<String>);

    pub(crate) fn request_cancel(&self, run_id: &str);

    pub(crate) fn is_cancelled(&self, run_id: &str) -> bool;

    pub(crate) fn clear_cancelled(&self, run_id: &str);

    pub(crate) async fn ensure_worker_ready_for_enqueue(&self) -> crate::error::AppResult<()>;

    async fn ensure_worker_ready_for_enqueue_with_timeout(
        &self,
        timeout: std::time::Duration,
    ) -> crate::error::AppResult<()>;
}
```

`complete_waiter(...)` must remove the sender from the map before sending, then ignore a dropped receiver:

```rust
if let Some(sender) = self.waiters.lock().remove(run_id) {
    let _ = sender.send(result);
}
```

`ensure_worker_ready_for_enqueue_with_timeout(...)` must not try to await on `watch::Sender` directly. Use `let mut status_rx = self.worker_status.subscribe();`, check `status_rx.borrow().clone()` first for an already-ready or already-failed worker, then await `status_rx.changed()` inside `tokio::time::timeout(...)` and read updates with `status_rx.borrow_and_update().clone()`.

`Default` must set `worker_execution_timeout` to `std::time::Duration::from_secs(20 * 60)`, set `waiter_timeout` to `worker_execution_timeout + std::time::Duration::from_secs(5)`, set `worker_hard_guard_timeout` to `worker_execution_timeout + std::time::Duration::from_secs(15)`, and set worker status to `Starting`. `new_for_test(timeout)` must set `worker_execution_timeout` to `timeout`, set `waiter_timeout` to `timeout + std::time::Duration::from_millis(50)`, and set `worker_hard_guard_timeout` to `timeout + std::time::Duration::from_millis(100)`. `new_for_test_with_timeouts(waiter_timeout, worker_execution_timeout, worker_hard_guard_timeout)` must reject or panic in tests unless `worker_execution_timeout < waiter_timeout < worker_hard_guard_timeout`, because equal or inverted deadlines reintroduce caller/worker/hard-guard timeout races. `new_for_waiter_timeout_test(waiter_timeout)` is allowed only for unit tests of waiter cleanup where no worker is running; it must not be used by command-flow or worker-flow tests. `ensure_worker_ready_for_enqueue()` must use a `5` second startup timeout in production.

- [x] **Step 7: Add cancellation set runtime test**

Add:

```rust
#[test]
fn runtime_tracks_and_clears_cancelled_run_ids() {
    let runtime = GeminiBrowserJobRuntime::new_for_test(std::time::Duration::from_secs(1));

    assert!(!runtime.is_cancelled("run-cancel"));
    runtime.request_cancel("run-cancel");
    assert!(runtime.is_cancelled("run-cancel"));
    runtime.clear_cancelled("run-cancel");
    assert!(!runtime.is_cancelled("run-cancel"));
}
```

- [x] **Step 8: Run waiter tests**

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

**Status mapping gate:** Task 4 must not implement `run_status_for_queue_state(...)` and must not hardcode Apalis SQL status strings. Leave that helper unavailable, behind `todo!()`, or limited to test-only placeholders until Task 5 completes `apalis_sqlite_status_probe_documents_actual_status_values`.

- [x] **Step 1: Add handler contract test with a fake executor**

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
        artifact_mode: GeminiBrowserArtifactMode::Reduced,
        browser_config: None,
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

- [x] **Step 2: Add handler error-path test**

Add:

```rust
#[tokio::test]
async fn worker_handler_converts_executor_error_to_terminal_failed_result() {
    let temp = tempfile::tempdir().expect("temp dir");
    let runtime = GeminiBrowserJobRuntime::new_for_test(std::time::Duration::from_secs(1));
    let receiver = runtime
        .register_waiter("run-worker-error")
        .expect("register waiter");
    let events = std::sync::Arc::new(parking_lot::Mutex::new(Vec::<String>::new()));
    let job = GeminiBrowserJob {
        run_id: "run-worker-error".to_string(),
        prompt: "hello".to_string(),
        source: "settings_test".to_string(),
        artifact_mode: GeminiBrowserArtifactMode::Reduced,
        browser_config: None,
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
        || async { Err(crate::error::AppError::internal("sidecar unavailable")) },
    )
    .await
    .expect("worker returns success to Apalis after terminal run result");

    assert_eq!(result.status, crate::gemini_browser::GeminiBrowserRunStatus::Failed);
    assert!(result.message.as_deref().unwrap_or("").contains("sidecar unavailable"));
    assert_eq!(receiver.await.expect("waiter open").expect("worker result"), result);
    assert_eq!(events.lock().as_slice(), ["running", "failed"]);

    let runs = crate::gemini_browser::list_runs(temp.path(), 10)
        .expect("list runs")
        .runs;
    assert_eq!(runs[0].status, crate::gemini_browser::GeminiBrowserRunStatus::Failed);
}
```

Production sidecar errors must be converted through `sidecar::sidecar_unavailable_result(...)` or an equivalent current helper before finishing the run. The worker handler must return success to Apalis after writing that terminal failed result, so Apalis does not retry a browser submission that already produced product-facing terminal state.

- [x] **Step 3: Implement testable worker core**

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

    let result = match executor().await {
        Ok(result) => result,
        Err(error) => crate::gemini_browser::GeminiBrowserRunResult {
            run_id: job.run_id.clone(),
            status: crate::gemini_browser::GeminiBrowserRunStatus::Failed,
            text: None,
            message: Some(error.to_string()),
            manual_action: None,
            artifacts: crate::gemini_browser::GeminiBrowserArtifactRefs::default(),
            elapsed_ms: 0,
            debug_summary: None,
        },
    };
    crate::gemini_browser::finish_run(runs_dir, &job.run_id, result.clone())?;
    events.lock().push(format!("{:?}", result.status).to_lowercase());
    runtime.complete_waiter(&job.run_id, Ok(result.clone()));
    Ok(result)
}
```

- [x] **Step 4: Run handler tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib worker_handler_marks_run_running_and_terminal
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib worker_handler_converts_executor_error_to_terminal_failed_result
```

Expected: PASS.

---

### Task 5: Verify Actual Apalis Status Serialization

**Files:**
- Modify: `src-tauri/src/gemini_browser/jobs.rs`
- Test: `src-tauri/src/gemini_browser/jobs.rs`

This is the first task allowed to implement `run_status_for_queue_state(...)` or rely on Apalis SQL status string values.

- [x] **Step 1: Add a status serialization probe**

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

- [x] **Step 2: Replace status mapping guesses with verified values**

Implement `run_status_for_queue_state(state: &str)` only after Step 1 is passing. It must map the actual observed Apalis SQL values to:

```rust
GeminiBrowserRunStatus::Queued
GeminiBrowserRunStatus::Running
GeminiBrowserRunStatus::Ok
GeminiBrowserRunStatus::Failed
```

Do not hardcode `"done"` or `"completed"` until the probe test proves the real value for this dependency version. Do not map Apalis `Killed` to `GeminiBrowserRunStatus::Cancelled` unless an implementation task proves Apalis emits `Killed` for this pilot. In the initial pilot, `GeminiBrowserRunStatus::Cancelled` comes from the durable run log cancellation path.

- [x] **Step 3: Run status tests**

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

- [x] **Step 1: Add no-retry task construction test**

Add a test named `gemini_browser_jobs_are_built_with_one_total_attempt`.

The test must:

- build a `GeminiBrowserJob` through the same helper production enqueue uses to create an Apalis task;
- assert the task uses max attempts `1`, using the current Apalis task metadata API;
- assert the idempotency key or task id contains the Gemini Browser `run_id`.

Use the current Apalis API proven in Task 1. Context7 Apalis docs describe `TaskBuilder::attempts(n)` as total attempts where `n = 1` means zero retries.

- [x] **Step 2: Implement a production task builder helper**

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

- [x] **Step 3: Add failing-handler no-retry integration test**

Add a test named `failed_gemini_browser_job_is_not_retried`.

The test must:

- push one job through the production task builder;
- start a real Apalis worker with a fake handler that increments `AtomicUsize` and returns an error;
- wait until Apalis marks the job terminal failed or the worker stops;
- assert the execution count is exactly `1`;
- inspect task metadata or SQL row and assert max attempts is `1` when the backend exposes it.

- [x] **Step 4: Run no-retry tests**

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

- [x] **Step 1: Add worker bootstrap**

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
    // Install a Tower hard-guard timeout layer using runtime.worker_hard_guard_timeout().
    // Build the production Gemini Browser job handler.
    // Mark GeminiBrowserJobRuntime worker status Ready after storage and worker construction succeed.
    // Mark worker status Failed before returning any startup error.
    // Run the worker future until Tauri shutdown.
    // If worker.run() returns Err or exits before shutdown, mark worker status Failed.
}
```

This function must not return `Ok(())` without starting a worker.
If `worker.run()` exits before application shutdown, record diagnostic message `"Gemini Browser job worker stopped unexpectedly"` unless the returned error has a more specific message.

- [x] **Step 2: Worker handler calls the existing sidecar path**

The production handler must:

- check queued cancellation before marking running;
- call `mark_running(...)`;
- call `GeminiBrowserState::start_run(...)`;
- emit `GeminiBrowserRunStatus::Running`;
- call `sidecar::send_single(...)` with `job.run_request()`, the same profile and artifact paths as the old command path, and `job.browser_config.clone()`;
- convert sidecar errors through `sidecar::sidecar_unavailable_result(...)`;
- call `finish_run(...)`;
- call `GeminiBrowserState::finish_run(...)`;
- complete the waiter;
- call `runtime.clear_cancelled(&job.run_id)` after any terminal result or queued-cancel acknowledgement;
- emit the terminal event.

- [x] **Step 3: Add worker execution timeout layer**

Use Tower timeout middleware when building the worker service:

```rust
let timeout = runtime.worker_hard_guard_timeout();
let timeout_layer = tower::timeout::TimeoutLayer::new(timeout);

let worker = apalis::prelude::WorkerBuilder::new("gemini-browser")
    .backend(storage)
    .concurrency(1)
    .layer(timeout_layer)
    .build(handler);
```

Context7 Tower docs show that timeout middleware returns an error that can be downcast to `tower::timeout::error::Elapsed` when the deadline expires.
`runtime.worker_hard_guard_timeout()` must be strictly greater than both `runtime.worker_execution_timeout()` and `runtime.waiter_timeout()`. In production it is `worker_execution_timeout + std::time::Duration::from_secs(15)`.

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

- write a terminal `GeminiBrowserRunStatus::Failed` run log result with message `"Gemini Browser job timed out after {seconds}s"`;
- complete the waiter with the same failed result when the waiter still exists;
- emit the failed run event;
- then stop the active Gemini Browser sidecar, if this job started it, as bounded best-effort with `tokio::time::timeout(std::time::Duration::from_secs(3), ...)`;
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
The outer `WorkerBuilder::layer(TimeoutLayer::new(...))` remains required as a hard guard around the whole worker service, but it must have a later deadline than both the product-facing helper and caller waiter timeout. The timeout-aware helper is required so product-facing run log and waiter state are finalized deterministically before the hard guard can abort the handler. Never await sidecar stop before writing the timeout result; if stop hangs, the run log and waiter must already be terminal.

- [x] **Step 4: Add worker timeout release test**

Add a test named `worker_timeout_marks_run_failed_and_processes_next_job`.

The test must:

- construct `GeminiBrowserJobRuntime::new_for_test_with_timeouts(std::time::Duration::from_millis(50), std::time::Duration::from_millis(25), std::time::Duration::from_millis(250))`;
- create queued run log records for `run-timeout-first` and `run-timeout-second`;
- run a real worker with `.concurrency(1)` and the same Tower timeout layer used in production;
- make the first fake handler wait forever with `std::future::pending::<crate::error::AppResult<crate::gemini_browser::GeminiBrowserRunResult>>()`;
- make the second fake handler return `GeminiBrowserRunStatus::Ok`;
- assert the first run log is terminal `Failed` with message containing `"timed out"`;
- assert the first run log is terminal before the hard-guard deadline would fire;
- assert a caller waiting through `wait_for_registered_result(...)` receives the terminal failed timeout result, not the caller waiter-timeout error;
- assert `finish_timed_out_job(...)` writes the first terminal failed run log and emits the failed event before attempting bounded sidecar stop;
- assert the second job is processed after the first timeout;
- assert the second run log is terminal `Ok`.

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib worker_timeout_marks_run_failed_and_processes_next_job
```

Expected: PASS.

- [x] **Step 5: Add worker startup failure test**

Add a test named `worker_startup_failure_marks_runtime_failed`.

The test must:

- force storage initialization to fail with a bad path or injected failing opener;
- call the worker bootstrap core;
- assert `GeminiBrowserJobRuntime` status is `Failed`;
- assert `ensure_worker_ready_for_enqueue()` returns an error containing the startup failure.

- [x] **Step 6: Add worker run failure test**

Add a test named `worker_run_failure_marks_runtime_failed`.

The test must:

- start the worker bootstrap core with a fake worker future that returns `Err("worker loop failed")`;
- assert `GeminiBrowserJobRuntime` status becomes `Failed`;
- assert `ensure_worker_ready_for_enqueue()` returns an error containing `"worker loop failed"`.

- [x] **Step 7: Export bootstrap and runtime**

In `mod.rs`:

```rust
pub(crate) use jobs::{
    enqueue_gemini_browser_job, start_gemini_browser_job_worker,
    GeminiBrowserJobRuntime,
};
```

Do not export `cancel_gemini_browser_job` in this task. That function is introduced and exported in Task 10.

- [x] **Step 8: Spawn worker during setup**

In `src-tauri/src/lib.rs`, after state is managed and inside setup:

```rust
let worker_handle = app.handle().clone();
tauri::async_runtime::spawn(async move {
    if let Err(error) = gemini_browser::start_gemini_browser_job_worker(worker_handle).await {
        eprintln!("Failed to start Gemini Browser job worker: {error}");
    }
});
```

- [x] **Step 9: Run worker tests**

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
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/gemini_browser/commands.rs`
- Test: `src-tauri/src/gemini_browser/state.rs`

- [x] **Step 1: Add cached status snapshot type**

Add a cached status field to `GeminiBrowserState` or `GeminiBrowserJobRuntime`:

```rust
status_snapshot: parking_lot::RwLock<Option<GeminiBrowserProviderStatus>>,
```

Use `parking_lot::RwLock` because status reads are frequent and the snapshot update path does not require async work. Do not hold the read or write guard across `.await`.

Do not call `profile_dir(handle)?` from `GeminiBrowserState::new()` or `GeminiBrowserJobRuntime::default()`, because those constructors run before a fully usable `AppHandle` is available through `manage(...)`. Constructors must set `status_snapshot` to `None`.

Add an explicit setup initializer:

```rust
pub(crate) fn init_status_snapshot(&self, handle: &tauri::AppHandle) -> crate::error::AppResult<()> {
    let snapshot = GeminiBrowserProviderStatus {
        status: GeminiBrowserProviderStatusKind::NotStarted,
        manual_action: None,
        active_run_id: None,
        queue_depth: 0,
        browser_profile_dir: path_string(&profile_dir(handle)?),
        latest_message: Some("Gemini browser sidecar is not running.".to_string()),
    };
    *self.status_snapshot.write() = Some(snapshot);
    Ok(())
}
```

- [x] **Step 2: Add snapshot update helpers**

Add helpers:

```rust
pub(crate) fn update_status_snapshot(
    &self,
    handle: &tauri::AppHandle,
    update: impl FnOnce(&mut GeminiBrowserProviderStatus),
) -> crate::error::AppResult<()>;

pub(crate) fn status_snapshot(
    &self,
    handle: &tauri::AppHandle,
) -> crate::error::AppResult<GeminiBrowserProviderStatus>;
```

If the snapshot is still `None`, `status_snapshot(handle)` and `update_status_snapshot(handle, ...)` must build the same `NotStarted` snapshot with `profile_dir(handle)?`, store it, and then read or mutate it. `src-tauri/src/lib.rs` setup should call `init_status_snapshot(app.handle())` after `manage(...)` so normal UI status reads do not pay the lazy initialization path.

Worker and command code must update the snapshot on queued, running, terminal, cancelled, failed, and manual-action states.

- [x] **Step 3: Add non-blocking status test**

Add a test named `provider_status_uses_cached_snapshot_when_sidecar_is_busy`.

The test must:

- arrange a fake sidecar/status provider that never returns or sleeps longer than one second;
- set a cached snapshot with `status: GeminiBrowserProviderStatusKind::Running` and `active_run_id: Some("run-busy".to_string())`;
- call the status core with timeout `std::time::Duration::from_millis(25)`;
- assert it returns in less than `200` milliseconds;
- assert the returned status is the cached `Running` status.

- [x] **Step 4: Refactor `provider_status(...)`**

Change `provider_status(...)` so it:

- reads `active_run_id` and Apalis queue depth without sidecar mutex blocking;
- tries `sidecar::status(...)` with `tokio::time::timeout(std::time::Duration::from_millis(250), ...)`;
- updates the cached snapshot on live status success;
- returns cached status on timeout or busy sidecar.

Do not hold the sidecar mutex while reading Apalis queue depth or formatting the cached response.

- [x] **Step 5: Run status responsiveness tests**

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

- [x] **Step 1: Add command handoff test**

Add a test around a small core function, not a request-shape-only test. The function should accept:

- runs directory;
- `GeminiBrowserJobRuntime`;
- fake enqueue function;
- `GeminiBrowserRunRequest`.
- the `Option<GeminiBrowserProviderConfig>` passed to `send_single_prompt(...)`.

The test must assert:

- `create_queued_run(...)` writes a queued run log record before enqueue;
- fake enqueue receives exactly the same `run_id`, `prompt`, `source`, and `artifact_mode`;
- fake enqueue receives the same `browser_config` snapshot that was passed into the command core;
- the returned queued event uses `GeminiBrowserRunStatus::Queued`.

- [x] **Step 2: Add duplicate run id and enqueue failure tests**

Add tests:

```rust
#[tokio::test]
async fn send_single_prompt_rejects_duplicate_non_terminal_run_id_before_enqueue() {
    // Arrange an existing queued run log record for run-duplicate.
    // Call the command core with the same run_id and a fake enqueue that panics if called.
    // Assert the error mentions duplicate Gemini Browser run_id.
}

#[tokio::test]
async fn send_single_prompt_rejects_duplicate_terminal_run_id_before_enqueue() {
    // Arrange an existing terminal Ok run log record for run-duplicate-terminal.
    // Call the command core with the same run_id and a fake enqueue that panics if called.
    // Assert the error mentions duplicate Gemini Browser run_id.
}

#[tokio::test]
async fn send_single_prompt_rejects_duplicate_waiter_before_enqueue() {
    // Register an active waiter for run-duplicate-waiter.
    // Call the command core with the same run_id and a fake enqueue that panics if called.
    // Assert the error mentions an active Gemini Browser waiter for that run_id.
}

#[tokio::test]
async fn send_single_prompt_marks_run_failed_when_enqueue_fails() {
    // Arrange a fake enqueue that returns AppError::internal("push failed").
    // Assert the waiter is removed.
    // Assert the run log terminal status is Failed with message containing
    // "Gemini Browser job enqueue failed: push failed".
}

#[tokio::test]
async fn send_single_prompt_rejects_invalid_artifact_mode_before_side_effects() {
    // Arrange request.artifact_mode = "invalid".
    // Call the command core with fake enqueue that panics if called.
    // Assert validation error mentions unsupported artifact_mode.
    // Assert no run log record was created.
    // Assert no waiter was registered.
}
```

- [x] **Step 3: Refactor command flow**

`gemini_bridge_send_single(...)` continues to receive `browser_config: Option<GeminiBrowserProviderConfig>` from the Settings/UI command payload and must pass it unchanged into `send_single_prompt(...)`. Prompt Pack browser runtime must pass its persisted `browser_provider_config` the same way. `send_single_prompt(...)` captures that value into `GeminiBrowserJob.browser_config` before enqueue; the worker must not read Settings again.

In `send_single_prompt(...)`, keep input trimming and validation. Parse `artifact_mode` before any side effects. Then check worker readiness and reject duplicate active `run_id` before `create_queued_run(...)`. Then create the queued run log record. Replace direct `state.enqueue(...)`, `state.pop_next(...)`, and inline `sidecar::send_single(...)` with:

```rust
let runtime = handle.state::<crate::gemini_browser::GeminiBrowserJobRuntime>();
let artifact_mode = GeminiBrowserArtifactMode::from_wire(Some(&request.artifact_mode))?;
runtime.ensure_worker_ready_for_enqueue().await?;
reject_duplicate_existing_run_or_waiter(&runtime, &runs_root, &request.run_id).await?;
create_queued_run(&runs_root, &request.run_id, &request.source, &request.prompt)?;
let waiter = runtime.register_waiter(&request.run_id)?;
let queued = match crate::gemini_browser::enqueue_gemini_browser_job(
    handle,
    GeminiBrowserJob {
        run_id: request.run_id.clone(),
        prompt: request.prompt.clone(),
        source: request.source.clone(),
        artifact_mode,
        browser_config: browser_config.clone(),
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

`reject_duplicate_existing_run_or_waiter(...)` must perform both checks from the Completion Waiter Contract:

```rust
async fn reject_duplicate_existing_run_or_waiter(
    runtime: &GeminiBrowserJobRuntime,
    runs_root: &std::path::Path,
    run_id: &str,
) -> crate::error::AppResult<()> {
    if runtime.has_waiter(run_id) {
        return Err(crate::error::AppError::conflict(format!(
            "Gemini Browser run_id '{run_id}' already has an active waiter"
        )));
    }

    if run_log_has_any_run(runs_root, run_id).await? {
        return Err(crate::error::AppError::conflict(format!(
            "Gemini Browser run_id '{run_id}' already exists"
        )));
    }

    Ok(())
}
```

Do not split this into two call sites in `send_single_prompt(...)`; the command handoff must reject duplicate waiters and any duplicate run log record, terminal or non-terminal, before `create_queued_run(...)`, `register_waiter(...)`, or enqueue. In this pilot, `run_id` is globally unique because it is also the Apalis idempotency key.

If `enqueue_gemini_browser_job(...)` returns an error after the queued run log record is written, the `Err` branch must:

- remove the waiter;
- call `finish_run(...)` with `GeminiBrowserRunStatus::Failed`;
- use message `"Gemini Browser job enqueue failed: {error}"`;
- emit a terminal failed run event;
- return the original enqueue error.

Emit the queued event with `queued.queue_position`.

- [x] **Step 4: Wait for terminal result**

After enqueue:

```rust
runtime.wait_for_registered_result(&request.run_id, waiter).await
```

The waiter must be removed on timeout, worker error, dropped sender, and success.

- [x] **Step 5: Verify command behavior**

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

- [x] **Step 1: Add queued cancellation test**

Test:

- create queued run log record;
- register waiter;
- call `GeminiBrowserJobRuntime::request_cancel("run-cancel-queued")`;
- call `cancel_gemini_browser_job(...)`;
- run worker handler against the still-present Apalis job;
- assert it does not call the fake executor;
- assert run log status is `Cancelled`;
- assert waiter receives a `GeminiBrowserRunResult` with `status: Cancelled`.

- [x] **Step 2: Add active cancellation test**

Test:

- start active run through `GeminiBrowserState::start_run("run-cancel-active".to_string())`;
- start the worker handler with a fake sidecar executor that blocks until stop is requested;
- call `cancel_gemini_browser_job(...)`;
- assert the active cancellation token is cancelled;
- unblock the fake sidecar executor and let the worker path finish;
- assert the worker writes a terminal `GeminiBrowserRunResult` with `GeminiBrowserRunStatus::Cancelled`.

`cancel_gemini_browser_job(...)` must not be expected to write the active terminal result directly. For active jobs it requests stop; the worker remains responsible for final run log state after the sidecar path returns.

- [x] **Step 3: Implement cancel helper**

Implement `cancel_gemini_browser_job(...)` with these branches:

- always record `runtime.request_cancel(run_id)`;
- if `GeminiBrowserState::active_run_id()` equals `run_id`, request stop and stop the sidecar;
- otherwise read the run log record from `runs_dir(handle)?`;
- if the run log record is missing, return `Ok(())` after recording cancellation; this covers Prompt Pack cancellation before browser enqueue creates a run log record;
- if the run log status is `Queued`, write a cancelled `GeminiBrowserRunResult`, emit a cancelled event, and complete the waiter with that result;
- if the run log is already terminal, leave it unchanged and return `Ok(())`;
- if the run log is `Running` but state does not report this active run, mark it `Failed` with message `"Gemini Browser run was running without an active sidecar"`.
- after writing or observing a terminal cancellation for this process, call `runtime.clear_cancelled(run_id)`.

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

After `cancel_gemini_browser_job(...)` exists, export it from `mod.rs`:

```rust
pub(crate) use jobs::cancel_gemini_browser_job;
```

- [x] **Step 4: Update Prompt Pack cancellation**

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

- [x] **Step 5: Update manual stop**

In `gemini_bridge_stop(...)`, read `state.active_run_id().await`. If present, call `cancel_gemini_browser_job(&handle, &run_id).await`. If no active run exists, keep the current stop behavior.

- [x] **Step 6: Run cancellation tests**

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

- [x] **Step 1: Add queued browser-stage cancellation test**

Add a test named `prompt_pack_browser_stage_cancelled_while_queued_cancels_browser_job`.

The test must:

- create a Prompt Pack browser stage with a known `run_id`, `stage_run_id`, and derived `browser_run_id`;
- enqueue the Gemini Browser job without starting the Apalis worker;
- cancel the Prompt Pack run through the existing `PromptPackRunState` cancellation token;
- assert `cancel_gemini_browser_job(&handle, &browser_run_id)` is called through a test seam or fake browser runtime;
- assert the Gemini Browser run log terminal status is `GeminiBrowserRunStatus::Cancelled`;
- assert the Prompt Pack stage/run path returns `YoutubeSummaryStageExecutionError::Cancelled`;
- assert browser provenance does not record `browser_run_status = 'ok'`.

- [x] **Step 2: Add active browser-stage cancellation test**

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

- [x] **Step 3: Add provenance guard test**

Add a test named `cancelled_browser_stage_does_not_persist_success_provenance`.

The test must:

- run the browser-stage cancellation path;
- query `prompt_pack_stage_runs`;
- assert `browser_run_status` is either `NULL` or `'cancelled'`;
- assert it is never `'ok'`, `'ready'`, or a partial-success status after cancellation.

- [x] **Step 4: Add pre-enqueue cancellation test**

Add a test named `prompt_pack_browser_stage_cancelled_before_enqueue_is_tolerated`.

The test must:

- create a Prompt Pack browser stage with a derived `browser_run_id`;
- cancel the Prompt Pack run before calling the Gemini Browser enqueue path;
- call `cancel_gemini_browser_job(&handle, &browser_run_id)`;
- assert the helper returns `Ok(())` even though no Gemini Browser run log record exists;
- assert no successful browser provenance is written;
- assert the Prompt Pack stage/run path returns `YoutubeSummaryStageExecutionError::Cancelled`.

- [x] **Step 5: Run Prompt Pack cancellation tests**

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

- [x] **Step 1: Add pending-after-restart test**

Test:

- create queued run log record;
- push Apalis job;
- drop the first runtime without starting worker;
- create a new runtime and start fake worker against the same temp SQLite file;
- assert the job is processed and run log becomes terminal.

- [x] **Step 2: Add run log versus Apalis reconciliation matrix tests**

Add tests for this matrix:

- run log `Queued` with matching Apalis queued job remains `Queued`;
- run log `Queued` with Apalis `running` but no active sidecar becomes `Failed` with message `"Gemini Browser queue state was running without an active sidecar"`;
- run log `Queued` with no matching Apalis job becomes `Failed` with message `"Gemini Browser queued job was missing from Apalis storage"`;
- run log `Running` with no active worker becomes `Failed` with message `"Gemini Browser worker was interrupted before completion"`;
- run log `Running` with Apalis terminal failed/killed state becomes a matching terminal run log record;
- run log terminal `Cancelled` with a remaining Apalis job is left terminal and worker acknowledges without sidecar;
- missing run log for an Apalis job is acknowledged without sidecar execution.

The Apalis-state-dependent startup cases run only when `apalis_queue_inspection_mode() == ApalisQueueInspectionMode::Supported`. Add a separate `DegradedRunLogOnly` test that proves queued run log records are left unchanged at startup when no stable Apalis query API is available.

- [x] **Step 3: Implement startup and worker-entry reconciliation**

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

- [x] **Step 4: Run recovery tests**

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

- [x] **Step 1: Delete queue fields and methods**

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

- [x] **Step 2: Replace provider status queue depth source**

In `gemini_bridge_status(...)`, keep using `GeminiBrowserProviderStatus.queue_depth`. Return queue depth from a real Apalis-backed helper only if the helper is proven by tests; otherwise return `0` in this pilot and rely on queued run events for per-request `queue_position`. Do not read `VecDeque`.

- [x] **Step 3: Update state test**

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

- [x] **Step 4: Run state tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib state_tracks_active_run_and_cancellation
```

Expected: PASS.

---

## Deferred Backlog

These are intentionally not part of the Gemini Browser Apalis pilot acceptance criteria.

### Backlog 1: Graceful Gemini Browser Sidecar Shutdown On App Exit

**Reason:** The pilot stops active sidecar work on cancellation and worker timeout, but it does not add a global Tauri shutdown path. If the app exits while Chrome/Playwright is active, the OS usually cleans up the child process on normal exit, but crash or abrupt termination may still leave a browser process behind.

**Future implementation outline:**

- Use the current Tauri 2 lifecycle API from `app.run(|app, event| { ... })` or plugin `on_event`.
- Handle `tauri::RunEvent::ExitRequested { .. }` or `tauri::RunEvent::Exit` and call the same internal `stop_active_gemini_browser_sidecar(...)` helper used by cancellation.
- Do not block the event loop for a long browser shutdown. Use a short timeout such as `std::time::Duration::from_secs(3)`.
- Add a test around a small shutdown core helper with a fake sidecar stopper:

```rust
#[tokio::test]
async fn shutdown_stops_active_gemini_browser_sidecar_with_timeout() {
    // Arrange active_run_id = Some("run-shutdown") and a fake sidecar stopper.
    // Call the shutdown helper with a short timeout.
    // Assert stop was requested once and active run state is not left as running.
}
```

**Not covered:** hard crash cleanup after process death. That requires a separate orphan-process strategy and is not required for this queue migration.

---

### Task 14: Compatibility Verification

**Files:**
- No code changes unless verification reveals a bug.

- [x] **Step 1: Rust Gemini Browser verification**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib gemini_browser
```

Expected: PASS.

- [x] **Step 2: Prompt Pack browser handoff verification**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib prompt_packs
```

Expected: PASS.

- [x] **Step 3: Targeted architectural regression verification**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib retry
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib apalis_storage_shares_extractum_db_without_locking_app_pool
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib worker_handler_converts_executor_error_to_terminal_failed_result
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib worker_timeout_marks_run_failed_and_processes_next_job
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib wait_for_result_removes_waiter_when_worker_channel_closes
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib enqueue_duplicate_run_id_returns_conflict
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib provider_status_
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib prompt_pack_browser_stage_cancelled
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib prompt_pack_browser_stage_cancelled_before_enqueue_is_tolerated
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib restart
```

Expected: PASS for no-retry, shared main DB storage, worker error conversion, worker timeout release, closed waiter channels, duplicate enqueue conflicts, status responsiveness, Prompt Pack browser cancellation including pre-enqueue cancellation, and reconciliation/restart tests.

- [x] **Step 4: TypeScript Gemini Browser verification**

Run:

```powershell
npm.cmd run test -- src/lib/api/gemini-browser.test.ts src/lib/gemini-browser-run-inspector.test.ts src/lib/gemini-browser-setup-status.test.ts src/lib/gemini-browser-provider-panel-state.test.ts
```

Expected: PASS.

- [x] **Step 5: Svelte/type verification**

Run:

```powershell
npm.cmd run check
```

Expected: PASS.

- [x] **Step 6: Diff hygiene**

Run:

```powershell
git diff --check
git status --short
```

Expected: `git diff --check` exits `0`. `git status --short` shows only intentional files for this migration plus unrelated pre-existing changes.

---

## Self-Review

- Spec coverage: The plan covers exact Apalis RC dependency pins, real early Apalis integration, fail-fast worker smoke tests, SQLite storage ownership inside `extractum.db`, app migration metadata preservation, shared SQLite connection options, explicit runtime construction, synchronous in-memory waiter/cancellation/status locks, worker readiness/error/early-exit state, explicit `mark_worker_*` methods, explicit `request_cancel/is_cancelled/clear_cancelled` methods, receiver-based completion waiter semantics, closed waiter channel handling, duplicate `run_id` rejection, duplicate Apalis idempotency conflict translation, storage-injected enqueue tests, enqueue failure cleanup, worker sidecar error conversion, split product-facing and hard-guard worker timeouts, cancellation continuity including pre-enqueue Prompt Pack cancellation, Prompt Pack cancellation by `browser_run_id`, run log versus Apalis reconciliation, status UI responsiveness, no automatic retry, crash/restart recovery, current `GeminiBrowserRunResult` fields, self-contained browser execution snapshots including managed/CDP config, and removal of the old `VecDeque`.
- Placeholder scan: The plan intentionally avoids placeholder markers, fake production facades, and no-op worker bootstrap. The only API discovery point is constrained to Task 1 and must compile before later tasks proceed.
- Type consistency: Result examples use the current `GeminiBrowserRunResult` shape with `run_id`, `text`, `manual_action`, `GeminiBrowserArtifactRefs::default()`, and `elapsed_ms`.
- Apalis status consistency: The plan requires a real `apalis-sqlite` status probe before mapping queue statuses, and Task 4 is explicitly forbidden from hardcoding `run_status_for_queue_state(...)` before that probe passes.
- Retry consistency: The plan requires `attempts(1)` or the verified Apalis equivalent plus an execution-count test proving one failed job is not retried.
- Query consistency: The plan requires Task 1 to classify Apalis queue inspection as `Supported` or `DegradedRunLogOnly` before reconciliation tasks rely on Apalis internals.
- Waiter consistency: The plan uses `register_waiter(...) -> Receiver` plus `wait_for_registered_result(...)`; there is no result wait that tries to recover a receiver from only `run_id`.
- Closed-channel consistency: `wait_for_registered_result(...)` must remove the waiter and return `"Gemini Browser worker channel closed unexpectedly"` on `oneshot::error::RecvError`; `complete_waiter(...)` removes the sender before sending and ignores dropped receivers with `let _ = sender.send(result)`.
- Worker readiness consistency: `ensure_worker_ready_for_enqueue_with_timeout(...)` must create a receiver with `self.worker_status.subscribe()` and wait through `Receiver::changed()`, not try to await on `watch::Sender` directly.
- Idempotency consistency: duplicate Apalis idempotency / SQLite unique failures are translated to `AppError::conflict`, with a concrete duplicate enqueue test.
- Duplicate run consistency: `send_single_prompt(...)` uses one helper, `reject_duplicate_existing_run_or_waiter(...)`, that checks both existing active waiters and any existing run log record, terminal or non-terminal, before creating a queued run or registering a new waiter.
- Payload consistency: `GeminiBrowserJob` carries typed `GeminiBrowserArtifactMode` and the `Option<GeminiBrowserProviderConfig>` captured at enqueue time, so the worker does not re-read Settings or drift from managed to CDP mode after queueing.
- Provider config consistency: `GeminiBrowserProviderConfig` keeps `Clone`, `Debug`, `PartialEq`, `Eq`, `Serialize`, and `Deserialize` derives because it is embedded in `GeminiBrowserJob`.
- Status snapshot consistency: cached provider status is initialized with `AppHandle` through `init_status_snapshot(...)` or a lazy `status_snapshot(handle)` path, never from `GeminiBrowserState::new()` or `GeminiBrowserJobRuntime::default()`.
- Dependency consistency: Apalis adds only `apalis`, `apalis-sqlite`, `parking_lot`, and `tower`; `tempfile` is already present in `src-tauri/Cargo.toml` because production YouTube code uses it, so this pilot must not add a duplicate dev-dependency entry.
- Lock consistency: Runtime maps use `parking_lot::Mutex`, cached provider status uses `parking_lot::RwLock`, and plan text forbids holding synchronous lock guards across `.await`.
- Timeout consistency: Caller waiter timeout, product-facing worker execution timeout, and outer Tower hard-guard timeout are separate and ordered as `worker_execution_timeout < waiter_timeout < worker_hard_guard_timeout`; the worker timeout test proves the caller receives the terminal worker timeout result, not a caller-side waiter timeout, and the timed-out first job does not block the next queued job.
- Timeout finalization consistency: `finish_timed_out_job(...)` writes terminal failed run log state, completes the waiter, and emits the failed event before attempting bounded best-effort sidecar stop.
- Main DB consistency: Apalis schema initialization preserves product tables in the unit smoke test, preserves a seeded `_sqlx_migrations` table in a separate migration-history preservation test, avoids product table name collisions, and shares the main `extractum.db` without `database is locked` failures.
- Worker error consistency: sidecar/executor errors are converted into terminal failed run results and acknowledged to Apalis as handled so the no-retry policy is preserved.
- Cancellation consistency: cancellation runtime APIs are explicit, worker-entry checks in-memory cancellation before `mark_running(...)`, Prompt Pack pre-enqueue cancellation tolerates missing Gemini Browser run log records, and `cancel_gemini_browser_job` is exported only in Task 10 after it is implemented.
- Backlog consistency: graceful sidecar shutdown on Tauri app exit is recorded as a deferred backlog item with `RunEvent::ExitRequested` / `RunEvent::Exit` guidance, not as a pilot blocker.
- Task atomicity consistency: Task 2 Step 1 and Step 2 are explicitly one edit unit because the test references database constants introduced by the helper step.

## Execution Choice

When implementing this plan in the main development thread, choose one:

1. **Subagent-Driven** - Use a fresh implementation worker per task and review between tasks.
2. **Inline Execution** - Execute tasks in the current session with checkpoints.

For this repository, Inline Execution is acceptable because the first pilot is tightly scoped to Gemini Browser.
