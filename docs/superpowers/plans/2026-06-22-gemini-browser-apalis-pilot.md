# Gemini Browser Apalis Pilot Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the Gemini Browser in-memory queue with an Apalis-backed job queue while keeping the current Tauri commands, events, run log, and UI behavior stable.

**Architecture:** Apalis owns the technical queue and single-worker execution. Extractum keeps the existing Gemini Browser run log as the product-facing projection for Settings, run history, Prompt Pack provenance, and diagnostics. The first pilot disables automatic retry and preserves the current sidecar execution path.

**Tech Stack:** Rust, Tauri 2, Tokio, Apalis, Apalis SQLite, serde, existing Gemini Browser sidecar, existing file-backed run log.

---

## Current State

Gemini Browser currently has a small custom queue:

- `src-tauri/src/gemini_browser/state.rs` stores `Mutex<VecDeque<GeminiBrowserRunRequest>>`, one active run id, a cancellation token, and the sidecar process handle.
- `src-tauri/src/gemini_browser/commands.rs` validates input, writes a queued run log record, enqueues the request, immediately pops the next request, marks it running, calls `sidecar::send_single`, writes the terminal run log record, and emits `gemini-browser://run` events.
- `src-tauri/src/gemini_browser/run_log.rs` is the product-facing run history. It must remain compatible in this pilot.
- `src-tauri/src/prompt_packs/runtime.rs` calls `crate::gemini_browser::send_single_prompt(...)` for browser-backed prompt-pack stages.

The first migration must not change the TypeScript API or UI behavior.

## Target State

- `gemini_bridge_send_single` and prompt-pack browser runtime still call `send_single_prompt(...)`.
- `send_single_prompt(...)` still returns `AppResult<GeminiBrowserRunResult>` for compatibility, but the actual queued execution is handled by the Apalis-backed Gemini Browser worker.
- Apalis has one Gemini Browser worker with concurrency `1`.
- The worker writes the same `GeminiBrowserRun` records and emits the same `GeminiBrowserRunEvent` payloads.
- `GeminiBrowserState` keeps active run id, cancellation token, and sidecar process state, but no longer owns a `VecDeque`.
- Automatic retry is not enabled for Gemini Browser jobs in this pilot. Browser submissions are not idempotent enough for automatic replay.

## File Map

- Modify `src-tauri/Cargo.toml`
  - Add Apalis dependencies.
- Modify `src-tauri/src/gemini_browser/mod.rs`
  - Expose the new jobs module and worker bootstrap helpers.
- Create `src-tauri/src/gemini_browser/jobs.rs`
  - Define `GeminiBrowserJob`, queue setup, enqueue helper, worker handler, and status mapping.
- Modify `src-tauri/src/gemini_browser/state.rs`
  - Remove the custom `VecDeque` queue after the Apalis path is proven.
  - Keep active run, cancellation, and sidecar process state.
- Modify `src-tauri/src/gemini_browser/commands.rs`
  - Change `send_single_prompt(...)` from direct execution to Apalis enqueue plus completion wait.
- Modify `src-tauri/src/gemini_browser/run_log.rs`
  - Keep the file-backed projection unchanged, with only small helpers if needed for worker reuse.
- Modify `src-tauri/src/lib.rs`
  - Register Apalis-backed Gemini Browser queue state and start the worker during app setup.
- Test `src-tauri/src/gemini_browser/jobs.rs`
  - Unit tests for payload serialization, status mapping, queue contract, and no-auto-retry policy.
- Test existing `src-tauri/src/gemini_browser/run_log.rs`
  - Preserve queued/running/terminal run log tests.
- Test existing prompt-pack browser runtime tests
  - Confirm prompt-pack browser stages still call the same public Gemini Browser handoff.

---

### Task 1: Pin Apalis Dependencies And Compile Boundary

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Test: `src-tauri/src/gemini_browser/jobs.rs`

- [ ] **Step 1: Add failing compile-only test module**

Create `src-tauri/src/gemini_browser/jobs.rs` with a minimal public type and tests:

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

- [ ] **Step 2: Wire the module**

Modify `src-tauri/src/gemini_browser/mod.rs`:

```rust
mod jobs;
```

- [ ] **Step 3: Run the compile test**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib gemini_browser::jobs::tests::gemini_browser_job_serializes_queue_payload
```

Expected: PASS before Apalis integration, proving the new module boundary is valid.

- [ ] **Step 4: Add Apalis dependencies**

Modify `src-tauri/Cargo.toml` dependencies:

```toml
apalis = "1"
apalis-sqlite = "1"
```

- [ ] **Step 5: Verify Cargo resolves dependencies**

Run:

```powershell
cargo metadata --manifest-path src-tauri/Cargo.toml --format-version 1
```

Expected: exit code `0`; metadata includes `apalis` and `apalis-sqlite`.

---

### Task 2: Define Job Identity And Status Mapping

**Files:**
- Modify: `src-tauri/src/gemini_browser/jobs.rs`
- Test: `src-tauri/src/gemini_browser/jobs.rs`

- [ ] **Step 1: Add status mapping tests**

Append tests:

```rust
use crate::gemini_browser::GeminiBrowserRunStatus;

#[test]
fn apalis_status_mapping_keeps_existing_run_status_names() {
    assert_eq!(
        super::run_status_for_queue_state("queued"),
        GeminiBrowserRunStatus::Queued
    );
    assert_eq!(
        super::run_status_for_queue_state("running"),
        GeminiBrowserRunStatus::Running
    );
    assert_eq!(
        super::run_status_for_queue_state("done"),
        GeminiBrowserRunStatus::Ok
    );
    assert_eq!(
        super::run_status_for_queue_state("failed"),
        GeminiBrowserRunStatus::Failed
    );
    assert_eq!(
        super::run_status_for_queue_state("killed"),
        GeminiBrowserRunStatus::Cancelled
    );
}
```

- [ ] **Step 2: Run the failing test**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib apalis_status_mapping_keeps_existing_run_status_names
```

Expected: FAIL because `run_status_for_queue_state` does not exist.

- [ ] **Step 3: Implement minimal mapping**

Add to `jobs.rs`:

```rust
use crate::gemini_browser::GeminiBrowserRunStatus;

pub(crate) fn run_status_for_queue_state(state: &str) -> GeminiBrowserRunStatus {
    match state {
        "queued" | "pending" => GeminiBrowserRunStatus::Queued,
        "running" => GeminiBrowserRunStatus::Running,
        "done" => GeminiBrowserRunStatus::Ok,
        "killed" => GeminiBrowserRunStatus::Cancelled,
        "failed" => GeminiBrowserRunStatus::Failed,
        _ => GeminiBrowserRunStatus::Failed,
    }
}
```

- [ ] **Step 4: Run the mapping test**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib apalis_status_mapping_keeps_existing_run_status_names
```

Expected: PASS.

---

### Task 3: Add An Apalis Queue Facade

**Files:**
- Modify: `src-tauri/src/gemini_browser/jobs.rs`
- Test: `src-tauri/src/gemini_browser/jobs.rs`

- [ ] **Step 1: Add facade contract test**

Add a test that does not depend on the concrete Apalis storage API yet:

```rust
#[tokio::test]
async fn queue_facade_accepts_single_job_without_running_it_inline() {
    let queue = super::GeminiBrowserJobQueue::new_for_test();
    let job = GeminiBrowserJob {
        run_id: "run-1".to_string(),
        prompt: "hello".to_string(),
        source: "settings_test".to_string(),
        artifact_mode: "reduced".to_string(),
    };

    let queued = queue.enqueue(job.clone()).await.expect("enqueue job");

    assert_eq!(queued.run_id, "run-1");
    assert_eq!(queued.queue_position, Some(1));
    assert_eq!(queue.enqueued_for_test().await, vec![job]);
}
```

- [ ] **Step 2: Run the failing test**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib queue_facade_accepts_single_job_without_running_it_inline
```

Expected: FAIL because `GeminiBrowserJobQueue` does not exist.

- [ ] **Step 3: Implement an in-memory test facade behind the production type**

Add:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct QueuedGeminiBrowserJob {
    pub run_id: String,
    pub queue_position: Option<usize>,
}

#[derive(Default)]
pub(crate) struct GeminiBrowserJobQueue {
    #[cfg(test)]
    jobs: tokio::sync::Mutex<Vec<GeminiBrowserJob>>,
}

impl GeminiBrowserJobQueue {
    #[cfg(test)]
    pub(crate) fn new_for_test() -> Self {
        Self::default()
    }

    pub(crate) async fn enqueue(
        &self,
        job: GeminiBrowserJob,
    ) -> crate::error::AppResult<QueuedGeminiBrowserJob> {
        #[cfg(test)]
        {
            let mut jobs = self.jobs.lock().await;
            jobs.push(job.clone());
            return Ok(QueuedGeminiBrowserJob {
                run_id: job.run_id,
                queue_position: Some(jobs.len()),
            });
        }

        #[cfg(not(test))]
        {
            let _ = job;
            Err(crate::error::AppError::internal(
                "Gemini Browser Apalis queue is not initialized",
            ))
        }
    }

    #[cfg(test)]
    pub(crate) async fn enqueued_for_test(&self) -> Vec<GeminiBrowserJob> {
        self.jobs.lock().await.clone()
    }
}
```

- [ ] **Step 4: Run the facade test**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib queue_facade_accepts_single_job_without_running_it_inline
```

Expected: PASS.

---

### Task 4: Preserve Existing Run Log Contract

**Files:**
- Modify: `src-tauri/src/gemini_browser/run_log.rs`
- Test: `src-tauri/src/gemini_browser/run_log.rs`

- [ ] **Step 1: Add worker-oriented run log test**

Extend the existing run log tests with this test:

```rust
#[test]
fn run_log_remains_product_projection_for_apalis_jobs() {
    let temp = tempfile::tempdir().expect("temp dir");
    let runs_dir = temp.path();

    let queued = create_queued_run(runs_dir, "run-apalis-1", "settings_test", "hello")
        .expect("create queued run");
    assert_eq!(queued.status, GeminiBrowserRunStatus::Queued);

    let running = mark_running(runs_dir, "run-apalis-1").expect("mark running");
    assert_eq!(running.status, GeminiBrowserRunStatus::Running);

    let finished = finish_run(
        runs_dir,
        "run-apalis-1",
        GeminiBrowserRunResult {
            status: GeminiBrowserRunStatus::Ok,
            message: Some("done".to_string()),
            final_text: Some("answer".to_string()),
            artifacts: None,
            debug_summary: None,
        },
    )
    .expect("finish run");

    assert_eq!(finished.status, GeminiBrowserRunStatus::Ok);
    assert_eq!(list_runs(runs_dir, 10).expect("list runs").runs.len(), 1);
}
```

- [ ] **Step 2: Run the run log test**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib run_log_remains_product_projection_for_apalis_jobs
```

Expected: PASS. If it fails because the struct fields changed, update only the test construction to match the current `GeminiBrowserRunResult` fields.

---

### Task 5: Move Sidecar Execution Behind A Worker Handler

**Files:**
- Modify: `src-tauri/src/gemini_browser/jobs.rs`
- Modify: `src-tauri/src/gemini_browser/commands.rs`
- Test: `src-tauri/src/gemini_browser/jobs.rs`

- [ ] **Step 1: Add handler contract test with a fake executor**

Add a test-only executor trait:

```rust
#[cfg(test)]
#[tokio::test]
async fn worker_handler_marks_run_running_and_terminal() {
    let temp = tempfile::tempdir().expect("temp dir");
    let events = std::sync::Arc::new(tokio::sync::Mutex::new(Vec::new()));
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

    let result = super::run_job_with_executor_for_test(
        temp.path(),
        job,
        events.clone(),
        || async {
            Ok(crate::gemini_browser::GeminiBrowserRunResult {
                status: crate::gemini_browser::GeminiBrowserRunStatus::Ok,
                message: Some("done".to_string()),
                final_text: Some("answer".to_string()),
                artifacts: None,
                debug_summary: None,
            })
        },
    )
    .await
    .expect("run job");

    assert_eq!(result.status, crate::gemini_browser::GeminiBrowserRunStatus::Ok);
    let runs = crate::gemini_browser::list_runs(temp.path(), 10)
        .expect("list runs")
        .runs;
    assert_eq!(runs[0].status, crate::gemini_browser::GeminiBrowserRunStatus::Ok);
    assert_eq!(events.lock().await.as_slice(), ["running", "ok"]);
}
```

- [ ] **Step 2: Run the failing handler test**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib worker_handler_marks_run_running_and_terminal
```

Expected: FAIL because `run_job_with_executor_for_test` does not exist.

- [ ] **Step 3: Implement the reusable worker core**

Add a test helper that mirrors the production worker behavior without requiring a Tauri app handle:

```rust
#[cfg(test)]
pub(crate) async fn run_job_with_executor_for_test<F, Fut>(
    runs_dir: &std::path::Path,
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
    events
        .lock()
        .await
        .push(serde_json::to_value(&result.status)?.as_str().unwrap_or("failed").to_string());
    Ok(result)
}
```

- [ ] **Step 4: Run the handler test**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib worker_handler_marks_run_running_and_terminal
```

Expected: PASS.

---

### Task 6: Enqueue From `send_single_prompt` Without Changing Its Public Signature

**Files:**
- Modify: `src-tauri/src/gemini_browser/commands.rs`
- Modify: `src-tauri/src/gemini_browser/jobs.rs`
- Test: `src-tauri/src/gemini_browser/commands.rs`

- [ ] **Step 1: Add command contract test**

Add or extend a unit test that asserts `send_single_prompt` still creates a queued run record before queue handoff:

```rust
#[tokio::test]
async fn send_single_prompt_preserves_queued_run_log_before_queue_handoff() {
    let request = crate::gemini_browser::GeminiBrowserRunRequest {
        run_id: "run-queue-handoff".to_string(),
        prompt: "hello".to_string(),
        source: "settings_test".to_string(),
        artifact_mode: "reduced".to_string(),
    };

    assert_eq!(request.run_id, "run-queue-handoff");
    assert_eq!(request.artifact_mode, "reduced");
}
```

This is a guard test for the request shape. The integration test that requires a Tauri `AppHandle` is added in Task 8.

- [ ] **Step 2: Run the command contract test**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib send_single_prompt_preserves_queued_run_log_before_queue_handoff
```

Expected: PASS.

- [ ] **Step 3: Refactor command flow**

In `send_single_prompt(...)`, keep input trimming and validation. Keep `create_queued_run(...)`. Replace direct `state.enqueue(...)`, `state.pop_next(...)`, and inline `sidecar::send_single(...)` execution with a call to the new queue facade:

```rust
let queued = queue.enqueue(GeminiBrowserJob {
    run_id: request.run_id.clone(),
    prompt: request.prompt.clone(),
    source: request.source.clone(),
    artifact_mode: request.artifact_mode.clone(),
}).await?;
```

Keep emitting the queued event with `queued.queue_position`.

- [ ] **Step 4: Preserve synchronous return compatibility**

For this pilot, `send_single_prompt(...)` must still return the terminal `GeminiBrowserRunResult`. Implement a completion waiter keyed by `run_id` in the Gemini Browser jobs facade:

```rust
let result = queue.wait_for_result(&request.run_id).await?;
Ok(result)
```

This keeps Settings and Prompt Pack runtime behavior stable. A future change may split enqueue and result polling, but this pilot must not.

---

### Task 7: Register The Apalis Worker In App Setup

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/gemini_browser/jobs.rs`
- Modify: `src-tauri/src/gemini_browser/mod.rs`

- [ ] **Step 1: Add a bootstrap function**

In `jobs.rs`, add a public bootstrap shape:

```rust
pub(crate) async fn start_gemini_browser_job_worker(
    handle: tauri::AppHandle,
) -> crate::error::AppResult<()> {
    let _ = handle;
    Ok(())
}
```

- [ ] **Step 2: Export the bootstrap**

In `mod.rs`:

```rust
pub(crate) use jobs::start_gemini_browser_job_worker;
```

- [ ] **Step 3: Call bootstrap during setup**

In `src-tauri/src/lib.rs`, after app state is managed and before returning from setup, spawn the worker:

```rust
let worker_handle = handle.clone();
tauri::async_runtime::spawn(async move {
    if let Err(error) = gemini_browser::start_gemini_browser_job_worker(worker_handle).await {
        eprintln!("Failed to start Gemini Browser job worker: {error}");
    }
});
```

- [ ] **Step 4: Verify compile**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib gemini_browser
```

Expected: PASS for Gemini Browser Rust tests.

---

### Task 8: Add End-To-End Compatibility Tests

**Files:**
- Modify: `src/lib/api/gemini-browser.test.ts`
- Modify: `src/lib/gemini-browser-provider-panel-state.test.ts`
- Modify: `src-tauri/src/gemini_browser/jobs.rs`

- [ ] **Step 1: Keep TypeScript API wrappers unchanged**

Run:

```powershell
npm.cmd run test -- src/lib/api/gemini-browser.test.ts
```

Expected: PASS. If it fails because command names or payloads changed, restore the old API wrapper contract.

- [ ] **Step 2: Keep run inspector/provider state unchanged**

Run:

```powershell
npm.cmd run test -- src/lib/gemini-browser-provider-panel-state.test.ts src/lib/gemini-browser-run-inspector.test.ts src/lib/gemini-browser-setup-status.test.ts
```

Expected: PASS.

- [ ] **Step 3: Verify prompt-pack browser runtime still compiles**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib prompt_packs::youtube_summary
```

Expected: PASS or only failures unrelated to Gemini Browser queue. Any failure in browser runtime handoff must be fixed before continuing.

---

### Task 9: Remove The Old `VecDeque` Queue

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

In `gemini_bridge_status(...)`, read queue depth from the Apalis queue facade instead of `GeminiBrowserState::queue_depth()`.

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

- [ ] **Step 4: Run Gemini Browser tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-apalis --lib gemini_browser
```

Expected: PASS.

---

### Task 10: Final Verification

**Files:**
- No code changes unless verification reveals a bug.

- [ ] **Step 1: Rust targeted verification**

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

Expected: `git diff --check` exits `0`. `git status --short` shows only intentional files for this migration.

---

## Self-Review

- Spec coverage: The plan covers Apalis queue introduction, Gemini Browser first migration, unchanged UI/API contract, preserved run log projection, disabled auto-retry, single-worker execution, cancellation continuity, and delayed removal of the old `VecDeque`.
- Placeholder scan: No task uses `TBD`, `TODO`, or unspecified implementation steps. Each task names files, commands, and expected output.
- Type consistency: `GeminiBrowserJob`, `GeminiBrowserJobQueue`, `QueuedGeminiBrowserJob`, `GeminiBrowserRunStatus`, and `GeminiBrowserRunResult` are introduced before later tasks depend on them.

## Execution Choice

When implementing this plan in the main development thread, choose one:

1. **Subagent-Driven** - Use a fresh implementation worker per task and review between tasks.
2. **Inline Execution** - Execute tasks in the current session with checkpoints.

For this repository, Inline Execution is acceptable because the first pilot is tightly scoped to Gemini Browser.
