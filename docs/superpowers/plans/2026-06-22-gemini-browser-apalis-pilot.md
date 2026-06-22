# Gemini Browser Apalis Pilot Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the Gemini Browser in-memory queue with a real Apalis-backed SQLite job queue while keeping the current Tauri commands, events, run log, Prompt Pack integration, and UI behavior stable.

**Architecture:** Apalis owns durable queue storage and single-worker execution. Extractum keeps the existing file-backed Gemini Browser run log as the product-facing projection for Settings, run history, Prompt Pack provenance, and diagnostics. `send_single_prompt(...)` remains synchronous from the caller perspective by enqueueing a durable job and waiting on a per-run completion waiter.

**Tech Stack:** Rust, Tauri 2, Tokio, `apalis = "1"`, `apalis-sqlite = "1"`, serde, existing Gemini Browser sidecar, existing file-backed run log.

---

## Review Fixes Applied To This Plan

- The first implementation tasks now require a real Apalis SQLite storage smoke test, a real `TaskSink::push(...)`, and a real worker processing a fake job before command refactors begin.
- The plan no longer creates a production `enqueue()` that returns an uninitialized-queue error instead of using Apalis.
- SQLite storage ownership is specified: Apalis owns its internal queue tables in a separate Gemini Browser queue database file under the existing Gemini Browser app data directory.
- Completion waiting is specified with a per-run waiter map, timeout behavior, worker failure behavior, and restart behavior.
- Cancellation is specified for queued and active Gemini Browser jobs, including Prompt Pack cancellation by concrete `browser_run_id`.
- Worker bootstrap must use a real `WorkerBuilder` and backend with `.concurrency(1)`.
- `GeminiBrowserRunResult` examples use the current fields: `run_id`, `status`, `text`, `message`, `manual_action`, `artifacts`, `elapsed_ms`, and `debug_summary`.
- Apalis status mapping is test-driven because current docs show core statuses such as `Done` and SQL examples such as `"completed"`.

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
- `send_single_prompt(...)` writes the queued run log record, pushes a real Apalis SQLite job, emits the queued event, then waits for worker completion through `GeminiBrowserJobRuntime`.
- Apalis has one Gemini Browser worker with concurrency `1`.
- The worker writes the same `GeminiBrowserRun` records and emits the same `GeminiBrowserRunEvent` payloads.
- `GeminiBrowserState` keeps active run id, cancellation token, and sidecar process state. It stops owning the `VecDeque` after the Apalis worker path is proven.
- Automatic retry is disabled for Gemini Browser jobs in this pilot. Browser submissions are not safe enough for automatic replay.

## Storage Decisions

- Use a dedicated Apalis SQLite database file for this pilot: `base_dir(handle)?.join("jobs.sqlite")`, where `base_dir` is the existing Gemini Browser app data directory from `src-tauri/src/gemini_browser/paths.rs`.
- Do not add Apalis internal tables to `src-tauri` application migrations for `extractum.db`.
- Do not hand-design Apalis internal queue tables. Let `apalis-sqlite` create or manage its own schema through its supported storage initialization API.
- Keep the file-backed run log in `base_dir(handle)?.join("runs")` as the product projection. Apalis rows are queue implementation details.
- Persist app-visible state in the run log, not by reading Apalis SQL rows in UI commands.
- Queue name / worker name: `gemini-browser`.
- Job type name: `gemini_browser.run.v1`.
- Job idempotency key: the Gemini Browser `run_id`.
- Retry policy: max attempts `1`, or the closest Apalis configuration that guarantees no automatic replay.

Current Context7 Apalis docs show SQL task rows with fields such as `job`, `id`, `job_type`, `status`, `attempts`, `max_attempts`, `run_at`, `last_result`, `lock_at`, `lock_by`, `done_at`, `priority`, `metadata`, and `idempotency_key`. The plan uses that only as orientation; implementation must not depend on hand-written SQL against those internal fields except in an isolated integration test that verifies actual serialized status values.

## Completion Waiter Contract

Create `GeminiBrowserJobRuntime` as managed Tauri state:

```rust
pub(crate) struct GeminiBrowserJobRuntime {
    waiters: tokio::sync::Mutex<
        std::collections::HashMap<
            String,
            tokio::sync::oneshot::Sender<crate::error::AppResult<GeminiBrowserRunResult>>,
        >,
    >,
    cancelled_runs: tokio::sync::Mutex<std::collections::HashSet<String>>,
}
```

Rules:

- `send_single_prompt(...)` registers a waiter before pushing the Apalis job.
- If enqueue fails, `send_single_prompt(...)` removes the waiter before returning the enqueue error.
- `wait_for_result(run_id)` waits with a fixed timeout of `20` minutes.
- On timeout, remove the waiter and return `AppError::internal("Gemini Browser job timed out waiting for worker result")`.
- The worker always writes a terminal run log record before completing a waiter.
- If no waiter exists because the app restarted or the caller already timed out, the worker still writes the run log and emits events.
- If worker startup fails, new `send_single_prompt(...)` calls fail before enqueue with a clear internal error.
- If the app restarts after enqueue, there is no in-memory waiter to satisfy. The restarted worker still processes pending Apalis jobs and repairs the run log to terminal state.

## Cancellation Contract

- `GeminiBrowserJobRuntime::request_cancel(run_id)` records the run id in `cancelled_runs` for the current process.
- Cancellation must also be durable: `cancel_gemini_browser_job(...)` writes a cancelled terminal run log result when the run is still queued.
- Queued cancellation: if the job has not started, `cancel_gemini_browser_job(...)` writes `GeminiBrowserRunStatus::Cancelled`, emits a cancelled event, completes the waiter with a cancelled result, and leaves the Apalis job for the worker to acknowledge.
- Worker queued-cancel acknowledgement: before `mark_running(...)`, the worker reads the run log. If the run is already terminal `Cancelled`, the worker returns success to Apalis without calling the sidecar.
- Active cancellation: if `GeminiBrowserState::active_run_id()` equals `run_id`, call `GeminiBrowserState::request_stop()` and `sidecar::stop(...)`; worker writes a cancelled terminal result after the sidecar stops or returns a cancellation-shaped result.
- Prompt Pack cancellation: `run_browser_llm_request(...)` must call a new Gemini Browser cancel helper with the already-known `browser_run_id`, not only the one-slot `browser_state.request_stop()`.
- Manual Settings stop: `gemini_bridge_stop(...)` remains an active-run stop. It also records cancellation for the current active run id when one exists.
- Run log on cancellation uses `GeminiBrowserRunResult { status: GeminiBrowserRunStatus::Cancelled, text: None, message: Some("Cancelled".to_string()), manual_action: None, artifacts: GeminiBrowserArtifactRefs::default(), elapsed_ms, debug_summary: None }`.

## File Map

- Modify `src-tauri/Cargo.toml`
  - Add `apalis = "1"` and `apalis-sqlite = "1"`.
- Modify `src-tauri/src/gemini_browser/mod.rs`
  - Expose the new jobs module, runtime, enqueue helper, cancel helper, and worker bootstrap.
- Create `src-tauri/src/gemini_browser/jobs.rs`
  - Define `GeminiBrowserJob`, Apalis storage initialization, `GeminiBrowserJobRuntime`, enqueue, completion waiters, cancellation helpers, worker handler, worker bootstrap, and Apalis status verification tests.
- Modify `src-tauri/src/gemini_browser/state.rs`
  - Remove `VecDeque` only after the Apalis path is proven.
  - Keep active run, cancellation token, and sidecar process state.
- Modify `src-tauri/src/gemini_browser/commands.rs`
  - Change `send_single_prompt(...)` from inline execution to Apalis enqueue plus completion wait.
  - Change `gemini_bridge_stop(...)` to record cancellation for the active run.
- Modify `src-tauri/src/gemini_browser/run_log.rs`
  - Keep existing behavior; add small helpers only when worker reuse needs them.
- Modify `src-tauri/src/prompt_packs/runtime.rs`
  - Replace generic one-slot browser stop on Prompt Pack cancellation with cancellation by concrete `browser_run_id`.
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

- [ ] **Step 1: Add Apalis dependencies**

Modify `src-tauri/Cargo.toml` dependencies:

```toml
apalis = "1"
apalis-sqlite = "1"
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
- create an Apalis SQLite storage using the current `apalis-sqlite` API against a file in that temp directory;
- push one `GeminiBrowserJob` through `apalis::prelude::TaskSink::push(...)`;
- run a real `apalis::prelude::WorkerBuilder` worker with `.concurrency(1)`;
- use a fake handler that sends the processed `run_id` over a Tokio oneshot channel and stops the worker;
- assert that the processed run id is `"run-apalis-smoke"`.

The test must not use `Vec`, `VecDeque`, or a test-only queue facade as the queue under test.

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
    db_path: &std::path::Path,
) -> crate::error::AppResult<GeminiBrowserApalisStorage> {
    // Use the exact apalis-sqlite storage type proven by the smoke test.
}
```

`GeminiBrowserApalisStorage` may be a concrete type alias or a small wrapper around the concrete storage type required by `apalis-sqlite`.

---

### Task 2: Define Runtime State, Storage Path, And Queue Contract

**Files:**
- Modify: `src-tauri/src/gemini_browser/jobs.rs`
- Modify: `src-tauri/src/gemini_browser/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/gemini_browser/jobs.rs`

- [ ] **Step 1: Add path helper test**

Add a pure helper and test:

```rust
#[test]
fn apalis_db_path_lives_under_gemini_browser_base_dir() {
    let base = std::path::PathBuf::from("app-data").join("gemini-browser");
    assert_eq!(
        super::jobs_db_path_from_base(&base),
        base.join("jobs.sqlite")
    );
}
```

- [ ] **Step 2: Implement the path helper**

Add:

```rust
pub(crate) fn jobs_db_path_from_base(base: &std::path::Path) -> std::path::PathBuf {
    base.join("jobs.sqlite")
}
```

- [ ] **Step 3: Add runtime state**

Add `GeminiBrowserJobRuntime`:

```rust
#[derive(Default)]
pub(crate) struct GeminiBrowserJobRuntime {
    waiters: tokio::sync::Mutex<
        std::collections::HashMap<
            String,
            tokio::sync::oneshot::Sender<crate::error::AppResult<crate::gemini_browser::GeminiBrowserRunResult>>,
        >,
    >,
    cancelled_runs: tokio::sync::Mutex<std::collections::HashSet<String>>,
}
```

- [ ] **Step 4: Manage runtime state**

Modify `src-tauri/src/lib.rs` next to `GeminiBrowserState::new()`:

```rust
.manage(GeminiBrowserJobRuntime::default())
```

Export it from `mod.rs`:

```rust
pub(crate) use jobs::GeminiBrowserJobRuntime;
```

- [ ] **Step 5: Add real enqueue helper**

Add `enqueue_gemini_browser_job(...)` that:

- opens or clones the real Apalis SQLite storage proven in Task 1;
- pushes `GeminiBrowserJob` through `TaskSink::push(...)`;
- returns `QueuedGeminiBrowserJob { run_id, queue_position: None }`;
- never executes the sidecar inline.

The helper may return `queue_position: None` because Apalis SQL queue depth is not part of the product contract in this pilot.

- [ ] **Step 6: Test real enqueue persists a job before worker startup**

Add an integration-style unit test that:

- creates a temp queue DB;
- calls `enqueue_gemini_browser_job(...)`;
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
    let runtime = GeminiBrowserJobRuntime::default();
    let receiver = runtime.register_waiter("run-waiter-1").await;
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

    runtime
        .complete_waiter("run-waiter-1", Ok(result.clone()))
        .await;

    assert_eq!(receiver.await.expect("waiter open").expect("worker result"), result);
}
```

- [ ] **Step 2: Add waiter timeout cleanup test**

Add:

```rust
#[tokio::test]
async fn wait_for_result_removes_waiter_on_timeout() {
    let runtime = GeminiBrowserJobRuntime::default();
    let error = runtime
        .wait_for_result_with_timeout("run-timeout", std::time::Duration::from_millis(1))
        .await
        .expect_err("timeout error");

    assert!(error.to_string().contains("timed out waiting for worker result"));
    assert!(!runtime.has_waiter_for_test("run-timeout").await);
}
```

- [ ] **Step 3: Implement waiter methods**

Implement:

```rust
impl GeminiBrowserJobRuntime {
    pub(crate) async fn register_waiter(
        &self,
        run_id: &str,
    ) -> tokio::sync::oneshot::Receiver<crate::error::AppResult<crate::gemini_browser::GeminiBrowserRunResult>>;

    pub(crate) async fn complete_waiter(
        &self,
        run_id: &str,
        result: crate::error::AppResult<crate::gemini_browser::GeminiBrowserRunResult>,
    );

    pub(crate) async fn wait_for_result(
        &self,
        run_id: &str,
    ) -> crate::error::AppResult<crate::gemini_browser::GeminiBrowserRunResult>;

    async fn wait_for_result_with_timeout(
        &self,
        run_id: &str,
        timeout: std::time::Duration,
    ) -> crate::error::AppResult<crate::gemini_browser::GeminiBrowserRunResult>;
}
```

`wait_for_result(...)` must use `std::time::Duration::from_secs(20 * 60)`.

- [ ] **Step 4: Run waiter tests**

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
    let runtime = GeminiBrowserJobRuntime::default();
    let receiver = runtime.register_waiter("run-worker-1").await;
    let events = std::sync::Arc::new(tokio::sync::Mutex::new(Vec::<String>::new()));
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
    assert_eq!(events.lock().await.as_slice(), ["running", "ok"]);
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
    events: std::sync::Arc<tokio::sync::Mutex<Vec<String>>>,
    executor: F,
) -> crate::error::AppResult<crate::gemini_browser::GeminiBrowserRunResult>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<
        Output = crate::error::AppResult<crate::gemini_browser::GeminiBrowserRunResult>,
    >,
{
    crate::gemini_browser::mark_running(runs_dir, &job.run_id)?;
    events.lock().await.push("running".to_string());

    let result = executor().await?;
    crate::gemini_browser::finish_run(runs_dir, &job.run_id, result.clone())?;
    events.lock().await.push(format!("{:?}", result.status).to_lowercase());
    runtime.complete_waiter(&job.run_id, Ok(result.clone())).await;
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
- run the fake worker to completion;
- inspect status after completion;
- assert the observed queued and completed values against the actual values produced by `apalis-sqlite`;
- include a short code comment with the observed values.

This test may query Apalis SQL internals only because it is a probe that protects this migration plan from status-name drift.

- [ ] **Step 2: Replace status mapping guesses with verified values**

Implement `run_status_for_queue_state(state: &str)` only after Step 1 is passing. It must map the actual observed Apalis SQL values to:

```rust
GeminiBrowserRunStatus::Queued
GeminiBrowserRunStatus::Running
GeminiBrowserRunStatus::Ok
GeminiBrowserRunStatus::Failed
GeminiBrowserRunStatus::Cancelled
```

Do not hardcode `"done"` or `"completed"` until the probe test proves the real value for this dependency version.

- [ ] **Step 3: Run status tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib apalis_sqlite_status_probe_documents_actual_status_values
```

Expected: PASS.

---

### Task 6: Start A Real Worker During App Setup

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
    // Open the Apalis SQLite storage from base_dir(handle)?.join("jobs.sqlite").
    // Build a WorkerBuilder named "gemini-browser".
    // Set concurrency to 1.
    // Build the production Gemini Browser job handler.
    // Run the worker future until Tauri shutdown.
}
```

This function must not return `Ok(())` without starting a worker.

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

- [ ] **Step 3: Export bootstrap and runtime**

In `mod.rs`:

```rust
pub(crate) use jobs::{
    cancel_gemini_browser_job, enqueue_gemini_browser_job,
    start_gemini_browser_job_worker, GeminiBrowserJobRuntime,
};
```

- [ ] **Step 4: Spawn worker during setup**

In `src-tauri/src/lib.rs`, after state is managed and inside setup:

```rust
let worker_handle = app.handle().clone();
tauri::async_runtime::spawn(async move {
    if let Err(error) = gemini_browser::start_gemini_browser_job_worker(worker_handle).await {
        eprintln!("Failed to start Gemini Browser job worker: {error}");
    }
});
```

- [ ] **Step 5: Run worker tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib gemini_browser::jobs
```

Expected: PASS.

---

### Task 7: Refactor `send_single_prompt(...)` To Enqueue And Wait

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

- [ ] **Step 2: Refactor command flow**

In `send_single_prompt(...)`, keep input trimming and validation. Keep `create_queued_run(...)`. Replace direct `state.enqueue(...)`, `state.pop_next(...)`, and inline `sidecar::send_single(...)` with:

```rust
let runtime = handle.state::<crate::gemini_browser::GeminiBrowserJobRuntime>();
let waiter = runtime.register_waiter(&request.run_id).await;
let queued = crate::gemini_browser::enqueue_gemini_browser_job(
    handle,
    GeminiBrowserJob {
        run_id: request.run_id.clone(),
        prompt: request.prompt.clone(),
        source: request.source.clone(),
        artifact_mode: request.artifact_mode.clone(),
    },
)
.await?;
```

Emit the queued event with `queued.queue_position`.

- [ ] **Step 3: Wait for terminal result**

After enqueue:

```rust
runtime.wait_for_registered_result(&request.run_id, waiter).await
```

The waiter must be removed on timeout, worker error, dropped sender, and success.

- [ ] **Step 4: Verify command behavior**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib send_single_prompt_
```

Expected: PASS.

---

### Task 8: Implement Cancellation By Browser Run Id

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
- call `cancel_gemini_browser_job(...)`;
- assert the active cancellation token is cancelled;
- assert terminal result shape uses `GeminiBrowserRunStatus::Cancelled`.

- [ ] **Step 3: Implement cancel helper**

Implement `cancel_gemini_browser_job(...)` with these branches:

- always record `runtime.request_cancel(run_id).await`;
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
    runtime.request_cancel(run_id).await;

    let state = handle.state::<crate::gemini_browser::GeminiBrowserState>();
    if state.active_run_id().await.as_deref() == Some(run_id) {
        state.request_stop().await;
        crate::gemini_browser::sidecar::stop(handle, &state).await?;
    } else {
        // Read the run log and handle queued, terminal, or stale-running state.
    }

    Ok(())
}
```

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

### Task 9: Crash And Restart Recovery

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

- [ ] **Step 2: Add stale-running reconciliation test**

Test:

- create a run log record in `Running` state with no active worker;
- call startup reconciliation;
- assert it becomes `Failed` with message `"Gemini Browser worker was interrupted before completion"`.

- [ ] **Step 3: Implement startup reconciliation**

At worker startup:

- scan the Gemini Browser run log for non-terminal `Running` runs;
- mark them `Failed` with a current `GeminiBrowserRunResult`;
- leave `Queued` runs alone because Apalis owns pending job processing.

- [ ] **Step 4: Run recovery tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib restart
```

Expected: PASS.

---

### Task 10: Remove The Old `VecDeque` Queue

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

- [ ] **Step 2: Replace status queue depth source**

In `gemini_bridge_status(...)`, return `queue_position: None` or queue depth from a real Apalis-backed helper only if the helper is proven by tests. Do not read `VecDeque`.

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

### Task 11: Compatibility Verification

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

- [ ] **Step 3: TypeScript Gemini Browser verification**

Run:

```powershell
npm.cmd run test -- src/lib/api/gemini-browser.test.ts src/lib/gemini-browser-run-inspector.test.ts src/lib/gemini-browser-setup-status.test.ts src/lib/gemini-browser-provider-panel-state.test.ts
```

Expected: PASS.

- [ ] **Step 4: Svelte/type verification**

Run:

```powershell
npm.cmd run check
```

Expected: PASS.

- [ ] **Step 5: Diff hygiene**

Run:

```powershell
git diff --check
git status --short
```

Expected: `git diff --check` exits `0`. `git status --short` shows only intentional files for this migration plus unrelated pre-existing changes.

---

## Self-Review

- Spec coverage: The plan covers real early Apalis integration, SQLite storage ownership, worker lifecycle, completion waiter semantics, cancellation continuity, Prompt Pack cancellation by `browser_run_id`, crash/restart recovery, current `GeminiBrowserRunResult` fields, and removal of the old `VecDeque`.
- Placeholder scan: The plan intentionally avoids placeholder markers, fake production facades, and no-op worker bootstrap. The only API discovery point is constrained to Task 1 and must compile before later tasks proceed.
- Type consistency: Result examples use the current `GeminiBrowserRunResult` shape with `run_id`, `text`, `manual_action`, `GeminiBrowserArtifactRefs::default()`, and `elapsed_ms`.
- Apalis status consistency: The plan requires a real `apalis-sqlite` status probe before mapping queue statuses.

## Execution Choice

When implementing this plan in the main development thread, choose one:

1. **Subagent-Driven** - Use a fresh implementation worker per task and review between tasks.
2. **Inline Execution** - Execute tasks in the current session with checkpoints.

For this repository, Inline Execution is acceptable because the first pilot is tightly scoped to Gemini Browser.
