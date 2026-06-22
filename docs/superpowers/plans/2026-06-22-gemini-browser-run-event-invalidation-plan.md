# Gemini Browser Run Event Invalidation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Convert Gemini Browser run events from state-bearing payloads into best-effort invalidation notifications, with Apalis owning execution/queue state and the run log owning user-facing run history/results.

**Architecture:** Backend event payloads are built only from persisted `GeminiBrowserRun` records, so `run_updated_at` always comes from the run log. The Settings panel uses one shared refresh scheduler for mount, command, and event-triggered refreshes; status and run-history reads are applied independently so live status failures cannot hide run-log updates.

**Tech Stack:** Rust/Tauri 2, Apalis SQLite runtime, file-backed Gemini Browser run log, Svelte 5, TypeScript, Vitest.

---

## Spec And Current Hot Spots

Approved spec: `docs/superpowers/specs/2026-06-22-gemini-browser-run-event-invalidation-design.md`

Current legacy contract:

- `src-tauri/src/gemini_browser/types.rs` exports `GeminiBrowserRunEvent { run_id, status, message, queue_position }`.
- `src-tauri/src/gemini_browser/commands.rs` exports `GEMINI_BROWSER_RUN_EVENT`, emits state-bearing events during settings `send_single`, and currently mutates the cached provider snapshot from live `provider_status_core`.
- `src-tauri/src/gemini_browser/jobs.rs` emits state-bearing events for queued cancellation, running, timeout, terminal, and some reconciliation paths.
- `src/lib/types/gemini-browser.ts` exports `GeminiBrowserRunEvent` with state fields.
- `src/lib/api/gemini-browser.ts` exports `GEMINI_BROWSER_RUN_EVENT` and `listenToGeminiBrowserRuns`.
- `src/lib/components/settings/gemini-browser-provider-panel.svelte` reads event payload status/message, uses raw `refresh()`, and directly assigns command return values into `status`/`result`.

## File Structure

- Modify: `src-tauri/src/gemini_browser/types.rs`
  - Rename the event payload to `GeminiBrowserRunChangeEvent`.
  - Keep only `run_id` and `run_updated_at`.
- Modify: `src-tauri/src/gemini_browser/mod.rs`
  - Re-export `GeminiBrowserRunChangeEvent`, not `GeminiBrowserRunEvent`.
- Modify: `src-tauri/src/gemini_browser/commands.rs`
  - Rename event constant to `GEMINI_BROWSER_RUN_CHANGE_EVENT`.
  - Build invalidation events from persisted `GeminiBrowserRun`.
  - Stop mutating cached provider snapshot from live `provider_status_core`.
  - Keep settings enqueue tests focused on run-log writes and invalidation payload.
- Modify: `src-tauri/src/gemini_browser/jobs.rs`
  - Change worker/cancel emit callbacks to use `GeminiBrowserRun`.
  - Emit after queued/running/terminal run-log transitions.
  - Make snapshot update and event delivery best-effort after successful run-log writes.
- Modify: `src/lib/types/gemini-browser.ts`
  - Rename `GeminiBrowserRunEvent` to `GeminiBrowserRunChangeEvent`.
- Modify: `src/lib/api/gemini-browser.ts`
  - Rename public API to `GEMINI_BROWSER_RUN_CHANGE_EVENT` and `listenToGeminiBrowserRunChanges`.
- Modify: `src/lib/api/gemini-browser.test.ts`
  - Verify new public API names and absence of legacy helper usage.
- Create: `src/lib/gemini-browser-refresh-scheduler.ts`
  - Pure TypeScript refresh scheduler with coalescing, shared trailing promise, independent status/history results, and no rejection for expected request failures.
- Create: `src/lib/gemini-browser-refresh-scheduler.test.ts`
  - Unit tests for scheduler behavior.
- Modify: `src/lib/components/settings/gemini-browser-provider-panel.svelte`
  - Replace raw `refresh()` with `scheduleRefresh()`.
  - Stop assigning authoritative `status`/`runs`/`result` from command return values.
  - Subscribe to the renamed run-change helper and ignore event payload except for scheduling.
- Modify: `src/lib/gemini-browser-provider-panel.test.ts`
  - Update source-level contract tests to match the scheduler and new helper names.

---

### Task 1: Backend Live Status Boundary

**Files:**
- Modify: `src-tauri/src/gemini_browser/commands.rs`

- [ ] **Step 1: Write the failing Rust test for live probe snapshot isolation**

Add this test inside `#[cfg(test)] mod tests` in `src-tauri/src/gemini_browser/commands.rs`, next to `provider_status_uses_cached_snapshot_when_sidecar_is_busy`:

```rust
#[tokio::test]
async fn provider_status_live_probe_does_not_mutate_cached_snapshot() {
    let state = GeminiBrowserState::new();
    state.set_status_snapshot(GeminiBrowserProviderStatus {
        status: super::super::GeminiBrowserProviderStatusKind::Running,
        manual_action: None,
        active_run_id: Some("run-cached".to_string()),
        queue_depth: 1,
        browser_profile_dir: "profile-dir".to_string(),
        latest_message: Some("Cached running".to_string()),
    });

    let returned = provider_status_core(
        &state,
        async {
            Ok(GeminiBrowserProviderStatus {
                status: super::super::GeminiBrowserProviderStatusKind::Ready,
                manual_action: None,
                active_run_id: None,
                queue_depth: 0,
                browser_profile_dir: "profile-dir".to_string(),
                latest_message: Some("Live ready".to_string()),
            })
        },
        Duration::from_millis(25),
        || {
            state
                .status_snapshot_for_test()
                .ok_or_else(|| AppError::internal("expected cached Gemini Browser status"))
        },
    )
    .await
    .expect("live status returned");

    assert_eq!(
        returned.status,
        super::super::GeminiBrowserProviderStatusKind::Ready
    );
    assert_eq!(returned.latest_message.as_deref(), Some("Live ready"));

    let cached = state
        .status_snapshot_for_test()
        .expect("cached status remains present");
    assert_eq!(
        cached.status,
        super::super::GeminiBrowserProviderStatusKind::Running
    );
    assert_eq!(cached.active_run_id.as_deref(), Some("run-cached"));
    assert_eq!(cached.latest_message.as_deref(), Some("Cached running"));
}
```

- [ ] **Step 2: Run the failing Rust test**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-invalidation --lib gemini_browser::commands::tests::provider_status_live_probe_does_not_mutate_cached_snapshot
```

Expected: FAIL because `provider_status_core` currently calls `state.set_status_snapshot(status.clone())` on a successful live status probe.

- [ ] **Step 3: Stop live status probes from mutating the cached snapshot**

In `provider_status_core`, replace:

```rust
Ok(Ok(status)) => {
    state.set_status_snapshot(status.clone());
    Ok(status)
}
```

with:

```rust
Ok(Ok(status)) => Ok(status),
```

This satisfies the spec rule that live sidecar/browser probes may contribute to the immediate `gemini_bridge_status` return value but must not write back into the lifecycle-owned cached provider snapshot.

- [ ] **Step 4: Run the provider status tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-invalidation --lib gemini_browser::commands::tests::provider_status
```

Expected: PASS for both provider status tests.

- [ ] **Step 5: Commit Task 1**

Run:

```powershell
git add src-tauri/src/gemini_browser/commands.rs
git commit -m "fix: keep Gemini Browser live status out of cached snapshot"
```

---

### Task 2: Backend Settings Enqueue Emits Invalidation From Run Log

**Files:**
- Modify: `src-tauri/src/gemini_browser/types.rs`
- Modify: `src-tauri/src/gemini_browser/mod.rs`
- Modify: `src-tauri/src/gemini_browser/commands.rs`

- [ ] **Step 1: Write failing tests for queued, failed, and no-transition emit behavior**

Update existing command tests in `src-tauri/src/gemini_browser/commands.rs`:

1. Change event vectors from:

```rust
Vec::<GeminiBrowserRunEvent>::new()
```

to:

```rust
Vec::<GeminiBrowserRunChangeEvent>::new()
```

2. In `send_single_prompt_handoff_writes_run_log_before_enqueue`, replace the old status assertion:

```rust
assert_eq!(handoff.queued_event.status, GeminiBrowserRunStatus::Queued);
assert_eq!(events.lock().as_slice(), [handoff.queued_event.clone()]);
```

with:

```rust
let queued_run = read_run_by_id(temp.path(), &request.run_id);
assert_eq!(queued_run.status, GeminiBrowserRunStatus::Queued);
assert_eq!(
    handoff.queued_event,
    GeminiBrowserRunChangeEvent {
        run_id: request.run_id.clone(),
        run_updated_at: queued_run.updated_at.clone(),
    }
);
assert_eq!(events.lock().as_slice(), [handoff.queued_event.clone()]);
```

3. In `send_single_prompt_marks_run_failed_when_enqueue_fails`, replace:

```rust
assert_eq!(
    events.lock().last().map(|event| event.status.clone()),
    Some(GeminiBrowserRunStatus::Failed)
);
```

with:

```rust
assert_eq!(
    events.lock().as_slice(),
    [GeminiBrowserRunChangeEvent {
        run_id: request.run_id.clone(),
        run_updated_at: run.updated_at.clone(),
    }]
);
```

4. Add a new test after `send_single_prompt_rejects_invalid_artifact_mode_before_side_effects`:

```rust
#[tokio::test]
async fn failed_run_log_transition_does_not_emit_change_event() {
    let temp = tempfile::tempdir().expect("temp dir");
    let runtime = ready_runtime();
    let mut request = test_request("bad/run-id");
    request.run_id = "../bad".to_string();
    let events = Arc::new(Mutex::new(Vec::<GeminiBrowserRunChangeEvent>::new()));

    let error = send_single_prompt_enqueue_core(
        temp.path(),
        &runtime,
        request,
        None,
        |_job| async { panic!("enqueue should not be called") },
        {
            let events = events.clone();
            move |event| events.lock().push(event)
        },
    )
    .await
    .expect_err("invalid run id rejected before emit");

    assert!(!error.to_string().is_empty());
    assert!(events.lock().is_empty());
}
```

- [ ] **Step 2: Run the failing command tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-invalidation --lib gemini_browser::commands::tests
```

Expected: FAIL to compile until `GeminiBrowserRunChangeEvent`, `run_change_event_from_run`, `send_single_prompt_enqueue_core`, and `SendSinglePromptEnqueueHandoff` exist.

- [ ] **Step 3: Add the Rust run-change event alongside the legacy worker event**

In `src-tauri/src/gemini_browser/types.rs`, add this struct immediately after `GeminiBrowserRunLogSummary` and before the existing legacy `GeminiBrowserRunEvent`:

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeminiBrowserRunChangeEvent {
    pub run_id: String,
    pub run_updated_at: String,
}
```

Keep the legacy `GeminiBrowserRunEvent` struct in this task because `jobs.rs` still uses it. Task 3 removes it after the worker/cancel paths are migrated.

In `src-tauri/src/gemini_browser/mod.rs`, change the re-export line from:

```rust
GeminiBrowserRunEvent, GeminiBrowserRunLogSummary, GeminiBrowserRunRequest,
```

to:

```rust
GeminiBrowserRunChangeEvent, GeminiBrowserRunEvent, GeminiBrowserRunLogSummary,
GeminiBrowserRunRequest,
```

- [ ] **Step 4: Add the run-change event constant and helper while keeping the legacy alias**

In `src-tauri/src/gemini_browser/commands.rs`, update imports to include `GeminiBrowserRun` and `GeminiBrowserRunChangeEvent`.

Replace:

```rust
pub const GEMINI_BROWSER_RUN_EVENT: &str = "gemini-browser://run";
```

with:

```rust
pub const GEMINI_BROWSER_RUN_CHANGE_EVENT: &str = "gemini-browser://run";
pub(crate) const GEMINI_BROWSER_RUN_EVENT: &str = GEMINI_BROWSER_RUN_CHANGE_EVENT;
```

Add these helpers below the constants:

```rust
pub(crate) fn run_change_event_from_run(run: &GeminiBrowserRun) -> GeminiBrowserRunChangeEvent {
    GeminiBrowserRunChangeEvent {
        run_id: run.run_id.clone(),
        run_updated_at: run.updated_at.clone(),
    }
}

pub(crate) fn emit_run_change_event_core<Emit>(run: &GeminiBrowserRun, mut emit: Emit)
where
    Emit: FnMut(GeminiBrowserRunChangeEvent) -> Result<(), String>,
{
    if let Err(error) = emit(run_change_event_from_run(run)) {
        eprintln!("Gemini Browser run-change event emit failed: {error}");
    }
}

fn emit_run_change_event(handle: &AppHandle, run: &GeminiBrowserRun) {
    emit_run_change_event_core(run, |event| {
        handle
            .emit(GEMINI_BROWSER_RUN_CHANGE_EVENT, event)
            .map_err(|error| error.to_string())
    });
}
```

Keep the old `emit_run_event` helper in this task because it is still used by worker paths. Task 3 replaces it with `emit_run_change_event`.

- [ ] **Step 5: Update settings enqueue core to emit from persisted runs**

In `src-tauri/src/gemini_browser/commands.rs`, replace:

```rust
#[derive(Debug)]
struct SendSinglePromptEnqueueHandoff {
    queued_event: GeminiBrowserRunEvent,
    waiter: GeminiBrowserWaiterReceiver,
}
```

with:

```rust
#[derive(Debug)]
struct SendSinglePromptEnqueueHandoff {
    queued_event: GeminiBrowserRunChangeEvent,
    waiter: GeminiBrowserWaiterReceiver,
}
```

Change the generic callback bound from:

```rust
EmitEvent: FnMut(GeminiBrowserRunEvent),
```

to:

```rust
EmitEvent: FnMut(&GeminiBrowserRun),
```

In `send_single_prompt_enqueue_core`, capture the queued run:

```rust
let queued_run = create_queued_run(runs_root, &request.run_id, &request.source, &request.prompt)?;
```

Replace the enqueue failure finish block with:

```rust
let failed_run = finish_run(runs_root, &request.run_id, failed.clone())?;
emit_event(&failed_run);
return Err(error);
```

Replace the queued event construction with:

```rust
let queued_event = run_change_event_from_run(&queued_run);
emit_event(&queued_run);
```

- [ ] **Step 6: Update settings `send_single_prompt` snapshot handling**

The emit callback in `send_single_prompt` now receives the persisted run-log record. Replace:

```rust
|event| {
    let _ = state.update_status_snapshot(handle, |status| {
        status.status =
            GeminiBrowserState::provider_status_kind_for_run_status(&event.status);
        status.active_run_id = None;
        status.queue_depth = 0;
        status.latest_message = event.message.clone();
        status.manual_action = None;
    });
    emit_run_event(handle, event);
},
```

with:

```rust
|run| {
    if let Err(error) = state.update_status_snapshot(handle, |status| {
        status.status = GeminiBrowserState::provider_status_kind_for_run_status(&run.status);
        status.active_run_id = None;
        status.queue_depth = 0;
        status.latest_message = run
            .result
            .as_ref()
            .and_then(|result| result.message.clone())
            .or_else(|| Some(format!("{:?}", run.status)));
        status.manual_action = run
            .result
            .as_ref()
            .and_then(|result| result.manual_action.clone());
    }) {
        eprintln!("Gemini Browser status snapshot update failed: {error}");
    }
    emit_run_change_event(handle, run);
},
```

The command tests keep asserting externally captured `GeminiBrowserRunChangeEvent` values. Inside tests, pass this closure to `send_single_prompt_enqueue_core`:

```rust
move |run| events.lock().push(run_change_event_from_run(run))
```

- [ ] **Step 7: Run the command tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-invalidation --lib gemini_browser::commands::tests
```

Expected: PASS.

- [ ] **Step 8: Commit Task 2**

Run:

```powershell
git add src-tauri/src/gemini_browser/types.rs src-tauri/src/gemini_browser/mod.rs src-tauri/src/gemini_browser/commands.rs
git commit -m "refactor: emit Gemini Browser settings changes from run log"
```

---

### Task 3: Backend Worker And Cancellation Run-Change Events

**Files:**
- Modify: `src-tauri/src/gemini_browser/jobs.rs`
- Modify: `src-tauri/src/gemini_browser/commands.rs`

- [ ] **Step 1: Write failing worker/cancel tests for invalidation payload, snapshot ordering, and best-effort emit**

In `src-tauri/src/gemini_browser/jobs.rs`, update event vectors from `GeminiBrowserRunEvent` to `GeminiBrowserRunChangeEvent`.

In `cancel_gemini_browser_job_cancels_queued_run_and_waiter`, replace the old status event assertion with:

```rust
let cancelled_run = read_run_by_id(temp.path(), &job.run_id);
assert_eq!(
    events.lock().as_slice(),
    [crate::gemini_browser::GeminiBrowserRunChangeEvent {
        run_id: job.run_id.clone(),
        run_updated_at: cancelled_run.updated_at.clone(),
    }]
);
```

Add a unit test near the cancel tests:

```rust
#[tokio::test]
async fn cancel_missing_run_does_not_emit_change_event() {
    let temp = tempfile::tempdir().expect("temp dir");
    let runtime = GeminiBrowserJobRuntime::new_for_test(Duration::from_secs(1));
    let state = crate::gemini_browser::GeminiBrowserState::new();
    let events = Arc::new(Mutex::new(Vec::<
        crate::gemini_browser::GeminiBrowserRunChangeEvent,
    >::new()));

    cancel_gemini_browser_job_core(
        &runtime,
        &state,
        temp.path(),
        "missing-run",
        {
            let events = events.clone();
            move |run| events
                .lock()
                .push(crate::gemini_browser::commands::run_change_event_from_run(run))
        },
        |_result| {},
        || async { Ok(()) },
    )
    .await
    .expect("missing run is acknowledged");

    assert!(events.lock().is_empty());
}
```

Add a unit test near the queued cancel test to lock the required terminal ordering:

```rust
#[tokio::test]
async fn cancel_queued_run_updates_terminal_snapshot_before_change_event() {
    let temp = tempfile::tempdir().expect("temp dir");
    let runtime = GeminiBrowserJobRuntime::new_for_test(Duration::from_secs(1));
    let state = crate::gemini_browser::GeminiBrowserState::new();
    let job = test_job("run-cancel-order");
    crate::gemini_browser::create_queued_run(
        temp.path(),
        &job.run_id,
        &job.source,
        &job.prompt,
    )
    .expect("create queued run");
    runtime.request_cancel(&job.run_id);

    let order = Arc::new(Mutex::new(Vec::<&'static str>::new()));
    let run_id_for_emit = job.run_id.clone();

    cancel_gemini_browser_job_core(
        &runtime,
        &state,
        temp.path(),
        &job.run_id,
        {
            let order = order.clone();
            move |run| {
                assert_eq!(run.run_id, run_id_for_emit);
                order.lock().push("emit");
            }
        },
        {
            let order = order.clone();
            move |result| {
                assert_eq!(
                    result.status,
                    crate::gemini_browser::GeminiBrowserRunStatus::Cancelled
                );
                order.lock().push("snapshot");
            }
        },
        || async { Ok(()) },
    )
    .await
    .expect("cancel queued job");

    assert_eq!(order.lock().as_slice(), ["snapshot", "emit"]);
}
```

Add a test for event payload construction near `emit_running_event` tests or helper tests:

```rust
#[test]
fn run_change_event_uses_run_log_updated_at_only() {
    let run = crate::gemini_browser::GeminiBrowserRun {
        run_id: "run-event".to_string(),
        source: "settings_test".to_string(),
        status: crate::gemini_browser::GeminiBrowserRunStatus::Running,
        prompt_preview: "hello".to_string(),
        created_at: "2026-06-22T00:00:00Z".to_string(),
        updated_at: "2026-06-22T00:00:01Z".to_string(),
        result: None,
    };

    assert_eq!(
        crate::gemini_browser::commands::run_change_event_from_run(&run),
        crate::gemini_browser::GeminiBrowserRunChangeEvent {
            run_id: "run-event".to_string(),
            run_updated_at: "2026-06-22T00:00:01Z".to_string(),
        }
    );
}
```

Add this test in `src-tauri/src/gemini_browser/commands.rs` for best-effort emit failure:

```rust
#[test]
fn run_change_event_emit_failure_is_best_effort() {
    let run = crate::gemini_browser::GeminiBrowserRun {
        run_id: "run-emit-fails".to_string(),
        source: "settings_test".to_string(),
        status: crate::gemini_browser::GeminiBrowserRunStatus::Queued,
        prompt_preview: "hello".to_string(),
        created_at: "2026-06-22T00:00:00Z".to_string(),
        updated_at: "2026-06-22T00:00:01Z".to_string(),
        result: None,
    };
    let mut attempted = false;

    emit_run_change_event_core(&run, |event| {
        attempted = true;
        assert_eq!(event.run_id, "run-emit-fails");
        assert_eq!(event.run_updated_at, "2026-06-22T00:00:01Z");
        Err("emit failed".to_string())
    });

    assert!(attempted);
}
```

Add this source-level test in `src-tauri/src/gemini_browser/commands.rs` for provider operations that must not emit run-change events without run-log transitions:

```rust
#[test]
fn status_open_and_resume_do_not_emit_run_change_events_directly() {
    let source = include_str!("commands.rs");

    let status_command = source
        .split("pub async fn gemini_bridge_status")
        .nth(1)
        .expect("status command exists")
        .split("pub(crate) async fn provider_status")
        .next()
        .expect("status command section");
    assert!(!status_command.contains("emit_run_change_event"));

    let open_command = source
        .split("pub async fn gemini_bridge_open_browser")
        .nth(1)
        .expect("open command exists")
        .split("#[tauri::command]")
        .next()
        .expect("open command section");
    assert!(!open_command.contains("emit_run_change_event"));

    let resume_command = source
        .split("pub async fn gemini_bridge_resume")
        .nth(1)
        .expect("resume command exists")
        .split("#[tauri::command]")
        .next()
        .expect("resume command section");
    assert!(!resume_command.contains("emit_run_change_event"));
}
```

- [ ] **Step 2: Run failing worker tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-invalidation --lib gemini_browser::jobs::tests::cancel
```

Expected: FAIL to compile until job/cancel callbacks accept `&GeminiBrowserRun` or emit `GeminiBrowserRunChangeEvent`.

- [ ] **Step 3: Change cancellation callback and terminal snapshot hook to emit from finished run**

In `cancel_gemini_browser_job_core`, change:

```rust
async fn cancel_gemini_browser_job_core<EmitEvent, StopActive, StopFut>(
```

to:

```rust
async fn cancel_gemini_browser_job_core<
    EmitEvent,
    BeforeEmitTerminalSnapshot,
    StopActive,
    StopFut,
>(
    runtime: &GeminiBrowserJobRuntime,
    state: &crate::gemini_browser::GeminiBrowserState,
    runs_root: &std::path::Path,
    run_id: &str,
    mut emit_event: EmitEvent,
    mut before_emit_terminal_snapshot: BeforeEmitTerminalSnapshot,
    stop_active: StopActive,
) -> crate::error::AppResult<()>
where
    EmitEvent: FnMut(&crate::gemini_browser::GeminiBrowserRun),
    BeforeEmitTerminalSnapshot: FnMut(&crate::gemini_browser::GeminiBrowserRunResult),
    StopActive: FnOnce() -> StopFut,
    StopFut: std::future::Future<Output = crate::error::AppResult<()>>,
```

For queued cancellation, replace:

```rust
crate::gemini_browser::finish_run(runs_root, run_id, result.clone())?;
...
emit_event(crate::gemini_browser::GeminiBrowserRunEvent { ... });
```

with:

```rust
let cancelled_run = crate::gemini_browser::finish_run(runs_root, run_id, result.clone())?;
runtime.complete_waiter(run_id, Ok(result.clone()));
before_emit_terminal_snapshot(&result);
emit_event(&cancelled_run);
```

For running-without-active-sidecar failure, replace the same pattern with:

```rust
let failed_run = crate::gemini_browser::finish_run(runs_root, run_id, result.clone())?;
runtime.complete_waiter(run_id, Ok(result.clone()));
before_emit_terminal_snapshot(&result);
emit_event(&failed_run);
```

In the public `cancel_gemini_browser_job`, replace the emit closure with:

```rust
|run| emit_gemini_browser_run_change_event(handle, run),
|result| update_terminal_status_snapshot_best_effort(handle, &state, result),
|| stop_active_gemini_browser_sidecar(handle, &state),
```

The order for queued cancellation and running-without-active-sidecar failure is always:

1. `finish_run(...)` writes the terminal run-log record.
2. `before_emit_terminal_snapshot(&result)` attempts the cached provider snapshot update.
3. `emit_event(&finished_run)` sends the best-effort invalidation event.

The snapshot hook is intentionally synchronous. `GeminiBrowserState::update_status_snapshot(...)` is synchronous, so this avoids a borrowed async future lifetime trap. Tests that do not care about snapshots pass `|_result| {}`.

- [ ] **Step 4: Change worker reconciliation to return run-log records**

Change the worker decision enum from:

```rust
enum GeminiBrowserWorkerEntryDecision {
    Execute,
    Acknowledged,
    Terminal(crate::gemini_browser::GeminiBrowserRunResult),
}
```

to:

```rust
enum GeminiBrowserWorkerEntryDecision {
    Execute(crate::gemini_browser::GeminiBrowserRun),
    Acknowledged,
    Terminal {
        run: crate::gemini_browser::GeminiBrowserRun,
        result: crate::gemini_browser::GeminiBrowserRunResult,
    },
}
```

In `reconcile_gemini_browser_worker_entry`:

- When cancellation finishes a run, store `let run = finish_run(...)?;` and return `Terminal { run, result }`.
- When interrupted running finishes a run, store `let run = finish_run(...)?;` and return `Terminal { run, result }`.
- When queued becomes running, store `let running_run = mark_running(...)?;` and return `Execute(running_run)`.

Update the worker entry match:

```rust
match reconcile_gemini_browser_worker_entry(&runtime, &state, &runs_root, &job).await? {
    GeminiBrowserWorkerEntryDecision::Execute(running_run) => {
        update_running_status_snapshot_best_effort(handle, &state, &job.run_id);
        emit_gemini_browser_run_change_event(handle, &running_run);
    }
    GeminiBrowserWorkerEntryDecision::Acknowledged => return Ok(()),
    GeminiBrowserWorkerEntryDecision::Terminal { run, result } => {
        update_terminal_status_snapshot_best_effort(handle, &state, &result);
        emit_gemini_browser_run_change_event(handle, &run);
        return Ok(());
    }
}
```

Remove the later standalone `update_running_status_snapshot(...).await?; emit_running_event(...)` pair because the running transition is emitted in the `Execute(running_run)` arm.

- [ ] **Step 5: Add best-effort snapshot update helpers**

Make `update_running_status_snapshot` and `update_terminal_status_snapshot` synchronous functions returning `AppResult<()>`; they only call synchronous `GeminiBrowserState::update_status_snapshot(...)`.

Replace their call sites in worker completion paths with synchronous best-effort wrappers:

```rust
fn update_running_status_snapshot_best_effort(
    handle: &tauri::AppHandle,
    state: &crate::gemini_browser::GeminiBrowserState,
    run_id: &str,
) {
    if let Err(error) = update_running_status_snapshot(handle, state, run_id) {
        eprintln!("Gemini Browser running status snapshot update failed: {error}");
    }
}

fn update_terminal_status_snapshot_best_effort(
    handle: &tauri::AppHandle,
    state: &crate::gemini_browser::GeminiBrowserState,
    result: &crate::gemini_browser::GeminiBrowserRunResult,
) {
    if let Err(error) = update_terminal_status_snapshot(handle, state, result) {
        eprintln!("Gemini Browser terminal status snapshot update failed: {error}");
    }
}
```

Keep the original `update_running_status_snapshot` and `update_terminal_status_snapshot` functions returning `AppResult<()>` so tests can still exercise exact status mutation.

- [ ] **Step 6: Emit terminal/timeout events from `finish_run` return values**

In `finish_timed_out_job`, replace:

```rust
crate::gemini_browser::finish_run(runs_dir, &job.run_id, result.clone())?;
...
update_terminal_status_snapshot(handle, state, &result).await?;
emit_gemini_browser_run_event(handle, &job.run_id, &result, None);
```

with:

```rust
let timed_out_run = crate::gemini_browser::finish_run(runs_dir, &job.run_id, result.clone())?;
runtime.complete_waiter(&job.run_id, Ok(result.clone()));
update_terminal_status_snapshot_best_effort(handle, state, &result);
emit_gemini_browser_run_change_event(handle, &timed_out_run);
```

In `finish_completed_job`, replace the same pattern with:

```rust
let completed_run = crate::gemini_browser::finish_run(runs_dir, &job.run_id, result.clone())?;
state.finish_run(&job.run_id).await;
runtime.complete_waiter(&job.run_id, Ok(result.clone()));
runtime.clear_cancelled(&job.run_id);
update_terminal_status_snapshot_best_effort(handle, state, &result);
emit_gemini_browser_run_change_event(handle, &completed_run);
```

Replace `emit_running_event` and `emit_gemini_browser_run_event` with:

```rust
fn emit_gemini_browser_run_change_event(
    handle: &tauri::AppHandle,
    run: &crate::gemini_browser::GeminiBrowserRun,
) {
    crate::gemini_browser::commands::emit_run_change_event_core(run, |event| {
        handle
            .emit(
                crate::gemini_browser::commands::GEMINI_BROWSER_RUN_CHANGE_EVENT,
                event,
            )
            .map_err(|error| error.to_string())
    });
}
```

- [ ] **Step 7: Update test helper decisions**

Any test helper that matches `GeminiBrowserWorkerEntryDecision::Execute` must now match `Execute(_)`.

Any test helper that matches `Terminal(result)` must now match `Terminal { result, .. }`.

For example:

```rust
GeminiBrowserWorkerEntryDecision::Execute(_) => { ... }
GeminiBrowserWorkerEntryDecision::Terminal { result, .. } => {
    Ok(TestWorkerEntryDecision::Terminal(result.status))
}
```

- [ ] **Step 8: Remove temporary legacy Rust event symbols**

After `jobs.rs` no longer references the old state-bearing event:

1. Delete the legacy `GeminiBrowserRunEvent` struct from `src-tauri/src/gemini_browser/types.rs`.
2. Remove `GeminiBrowserRunEvent` from the public re-export list in `src-tauri/src/gemini_browser/mod.rs`.
3. Delete the temporary alias from `src-tauri/src/gemini_browser/commands.rs`:

```rust
pub(crate) const GEMINI_BROWSER_RUN_EVENT: &str = GEMINI_BROWSER_RUN_CHANGE_EVENT;
```

4. Delete the legacy helper from `src-tauri/src/gemini_browser/commands.rs`:

```rust
fn emit_run_event(handle: &AppHandle, event: GeminiBrowserRunEvent) {
    let _ = handle.emit(GEMINI_BROWSER_RUN_EVENT, event);
}
```

- [ ] **Step 9: Run Gemini Browser backend tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-invalidation --lib gemini_browser
```

Expected: PASS.

- [ ] **Step 10: Commit Task 3**

Run:

```powershell
git add src-tauri/src/gemini_browser/types.rs src-tauri/src/gemini_browser/mod.rs src-tauri/src/gemini_browser/jobs.rs src-tauri/src/gemini_browser/commands.rs
git commit -m "refactor: emit Gemini Browser worker changes from run log"
```

---

### Task 4: Frontend API And Type Rename

**Files:**
- Modify: `src/lib/types/gemini-browser.ts`
- Modify: `src/lib/api/gemini-browser.ts`
- Modify: `src/lib/api/gemini-browser.test.ts`

- [ ] **Step 1: Write failing API tests for new exports and old-name removal**

In `src/lib/api/gemini-browser.test.ts`, update imports to:

```ts
import {
  GEMINI_BROWSER_RUN_CHANGE_EVENT,
  geminiBridgeListRuns,
  geminiBridgeOpenBrowser,
  geminiBridgeOpenRunFolder,
  geminiBridgeResume,
  geminiBridgeSendSingle,
  geminiBridgeStartCdpChrome,
  geminiBridgeStatus,
  geminiBridgeStop,
  listenToGeminiBrowserRunChanges,
} from "./gemini-browser";
```

Replace the last test with:

```ts
it("lists runs and subscribes to run-change invalidation events", async () => {
  await geminiBridgeListRuns(5);
  expect(invokeMock).toHaveBeenLastCalledWith("gemini_bridge_list_runs", { limit: 5 });

  const unlisten = vi.fn();
  const handler = vi.fn();
  listenMock.mockResolvedValueOnce(unlisten);
  await expect(listenToGeminiBrowserRunChanges(handler)).resolves.toBe(unlisten);
  expect(GEMINI_BROWSER_RUN_CHANGE_EVENT).toBe("gemini-browser://run");
  expect(listenMock).toHaveBeenCalledWith(GEMINI_BROWSER_RUN_CHANGE_EVENT, handler);
});
```

Add this source-level test at the bottom:

```ts
it("does not expose legacy run event public names", async () => {
  const api = await import("./gemini-browser");
  expect("listenToGeminiBrowserRuns" in api).toBe(false);
  expect("GEMINI_BROWSER_RUN_EVENT" in api).toBe(false);
});
```

- [ ] **Step 2: Run failing API tests**

Run:

```powershell
npm test -- src/lib/api/gemini-browser.test.ts
```

Expected: FAIL until the API/type names are renamed.

- [ ] **Step 3: Rename the TypeScript event type**

In `src/lib/types/gemini-browser.ts`, replace:

```ts
export interface GeminiBrowserRunEvent {
  run_id: string;
  status: GeminiBrowserRunStatus;
  message: string | null;
  queue_position: number | null;
}
```

with:

```ts
export interface GeminiBrowserRunChangeEvent {
  run_id: string;
  run_updated_at: string;
}
```

- [ ] **Step 4: Rename API helper and constant**

In `src/lib/api/gemini-browser.ts`, replace the event imports and exports with:

```ts
import { listen, type Event, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  GeminiBridgeSendSingleInput,
  GeminiBrowserProviderConfig,
  GeminiBrowserProviderStatus,
  GeminiBrowserRunChangeEvent,
  GeminiBrowserRunLogSummary,
  GeminiBrowserRunResult,
  GeminiBrowserStartChromeResult,
} from "$lib/types/gemini-browser";

export const GEMINI_BROWSER_RUN_CHANGE_EVENT = "gemini-browser://run";
```

Replace the legacy helper:

```ts
export function listenToGeminiBrowserRuns(
  handler: (event: Event<GeminiBrowserRunEvent>) => void,
): Promise<UnlistenFn> {
  return listen<GeminiBrowserRunEvent>(GEMINI_BROWSER_RUN_EVENT, handler);
}
```

with:

```ts
export function listenToGeminiBrowserRunChanges(
  handler: (event: Event<GeminiBrowserRunChangeEvent>) => void,
): Promise<UnlistenFn> {
  return listen<GeminiBrowserRunChangeEvent>(GEMINI_BROWSER_RUN_CHANGE_EVENT, handler);
}
```

- [ ] **Step 5: Run API tests**

Run:

```powershell
npm test -- src/lib/api/gemini-browser.test.ts
```

Expected: PASS.

- [ ] **Step 6: Commit Task 4**

Run:

```powershell
git add src/lib/types/gemini-browser.ts src/lib/api/gemini-browser.ts src/lib/api/gemini-browser.test.ts
git commit -m "refactor: expose Gemini Browser run-change API"
```

---

### Task 5: Frontend Refresh Scheduler

**Files:**
- Create: `src/lib/gemini-browser-refresh-scheduler.ts`
- Create: `src/lib/gemini-browser-refresh-scheduler.test.ts`

- [ ] **Step 1: Create failing scheduler tests**

Create `src/lib/gemini-browser-refresh-scheduler.test.ts`:

```ts
import { describe, expect, it, vi } from "vitest";
import {
  createGeminiBrowserRefreshScheduler,
  type GeminiBrowserRefreshSchedulerDeps,
} from "./gemini-browser-refresh-scheduler";
import type {
  GeminiBrowserProviderStatus,
  GeminiBrowserRun,
  GeminiBrowserRunLogSummary,
} from "./types/gemini-browser";

function status(overrides: Partial<GeminiBrowserProviderStatus> = {}): GeminiBrowserProviderStatus {
  return {
    status: "ready",
    manual_action: null,
    active_run_id: null,
    queue_depth: 0,
    browser_profile_dir: "profile-dir",
    latest_message: "Ready",
    ...overrides,
  };
}

function run(run_id: string): GeminiBrowserRun {
  return {
    run_id,
    source: "settings_test",
    status: "ok",
    prompt_preview: "hello",
    created_at: "2026-06-22T00:00:00Z",
    updated_at: "2026-06-22T00:00:01Z",
    result: null,
  };
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (error: unknown) => void;
  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
}

function schedulerDeps(
  overrides: Partial<GeminiBrowserRefreshSchedulerDeps> = {},
): GeminiBrowserRefreshSchedulerDeps {
  return {
    loadStatus: vi.fn(async () => status()),
    loadRuns: vi.fn(async () => ({ runs: [run("run-1")] })),
    applyStatus: vi.fn(),
    applyRuns: vi.fn(),
    applyStatusError: vi.fn(),
    applyRunsError: vi.fn(),
    applyMessage: vi.fn(),
    syncActivePromptResult: vi.fn(),
    formatError: (context, error) => `${context}: ${String(error)}`,
    ...overrides,
  };
}

describe("gemini browser refresh scheduler", () => {
  it("applies status and run history independently", async () => {
    const deps = schedulerDeps({
      loadStatus: vi.fn(async () => {
        throw new Error("status down");
      }),
      loadRuns: vi.fn(async (): Promise<GeminiBrowserRunLogSummary> => ({ runs: [run("run-ok")] })),
    });

    await createGeminiBrowserRefreshScheduler(deps).scheduleRefresh();

    expect(deps.applyRuns).toHaveBeenCalledWith([run("run-ok")]);
    expect(deps.syncActivePromptResult).toHaveBeenCalledWith([run("run-ok")]);
    expect(deps.applyStatus).not.toHaveBeenCalled();
    expect(deps.applyStatusError).toHaveBeenCalledWith(
      "loading Gemini browser provider status: Error: status down",
    );
  });

  it("preserves previous state and records both errors when both requests fail", async () => {
    const deps = schedulerDeps({
      loadStatus: vi.fn(async () => {
        throw new Error("status down");
      }),
      loadRuns: vi.fn(async () => {
        throw new Error("runs down");
      }),
    });

    await expect(createGeminiBrowserRefreshScheduler(deps).scheduleRefresh()).resolves.toBeUndefined();

    expect(deps.applyStatus).not.toHaveBeenCalled();
    expect(deps.applyRuns).not.toHaveBeenCalled();
    expect(deps.applyStatusError).toHaveBeenCalledWith(
      "loading Gemini browser provider status: Error: status down",
    );
    expect(deps.applyRunsError).toHaveBeenCalledWith(
      "loading Gemini browser run history: Error: runs down",
    );
    expect(deps.applyMessage).toHaveBeenCalledWith(
      "loading Gemini browser provider status: Error: status down",
    );
  });

  it("shares one trailing promise for callers during an active refresh", async () => {
    const firstStatus = deferred<GeminiBrowserProviderStatus>();
    const firstRuns = deferred<GeminiBrowserRunLogSummary>();
    const deps = schedulerDeps({
      loadStatus: vi
        .fn()
        .mockReturnValueOnce(firstStatus.promise)
        .mockResolvedValue(status({ latest_message: "Second" })),
      loadRuns: vi
        .fn()
        .mockReturnValueOnce(firstRuns.promise)
        .mockResolvedValue({ runs: [run("run-2")] }),
    });
    const scheduler = createGeminiBrowserRefreshScheduler(deps);

    const active = scheduler.scheduleRefresh();
    const trailingA = scheduler.scheduleRefresh();
    const trailingB = scheduler.scheduleRefresh();

    expect(trailingA).toBe(trailingB);

    firstStatus.resolve(status({ latest_message: "First" }));
    firstRuns.resolve({ runs: [run("run-1")] });

    await active;
    await trailingA;

    expect(deps.loadStatus).toHaveBeenCalledTimes(2);
    expect(deps.loadRuns).toHaveBeenCalledTimes(2);
  });

  it("resolves the first caller and rejects only the trailing promise when trailing refresh throws unexpectedly", async () => {
    const firstStatus = deferred<GeminiBrowserProviderStatus>();
    const firstRuns = deferred<GeminiBrowserRunLogSummary>();
    const unexpected = new Error("apply exploded");
    const deps = schedulerDeps({
      loadStatus: vi
        .fn()
        .mockReturnValueOnce(firstStatus.promise)
        .mockResolvedValue(status({ latest_message: "Second" })),
      loadRuns: vi
        .fn()
        .mockReturnValueOnce(firstRuns.promise)
        .mockResolvedValue({ runs: [run("run-2")] }),
      applyRuns: vi
        .fn()
        .mockImplementationOnce(() => {})
        .mockImplementationOnce(() => {
          throw unexpected;
        }),
    });
    const scheduler = createGeminiBrowserRefreshScheduler(deps);

    const active = scheduler.scheduleRefresh();
    const trailingA = scheduler.scheduleRefresh();
    const trailingB = scheduler.scheduleRefresh();

    expect(trailingA).toBe(trailingB);
    const activeResolution = expect(active).resolves.toBeUndefined();
    const trailingRejection = expect(trailingA).rejects.toBe(unexpected);

    firstStatus.resolve(status({ latest_message: "First" }));
    firstRuns.resolve({ runs: [run("run-1")] });

    await activeResolution;
    await trailingRejection;
  });
});
```

- [ ] **Step 2: Run failing scheduler tests**

Run:

```powershell
npm test -- src/lib/gemini-browser-refresh-scheduler.test.ts
```

Expected: FAIL because `src/lib/gemini-browser-refresh-scheduler.ts` does not exist.

- [ ] **Step 3: Implement the scheduler**

Create `src/lib/gemini-browser-refresh-scheduler.ts`:

```ts
import type {
  GeminiBrowserProviderStatus,
  GeminiBrowserRun,
  GeminiBrowserRunLogSummary,
} from "./types/gemini-browser";

export interface GeminiBrowserRefreshSchedulerDeps {
  loadStatus: () => Promise<GeminiBrowserProviderStatus>;
  loadRuns: () => Promise<GeminiBrowserRunLogSummary>;
  applyStatus: (status: GeminiBrowserProviderStatus) => void;
  applyRuns: (runs: GeminiBrowserRun[]) => void;
  applyStatusError: (message: string | null) => void;
  applyRunsError: (message: string | null) => void;
  applyMessage: (message: string) => void;
  syncActivePromptResult: (runs: GeminiBrowserRun[]) => void;
  formatError: (context: string, error: unknown) => string;
}

export interface GeminiBrowserRefreshScheduler {
  scheduleRefresh: () => Promise<void>;
}

// Each caller gets a promise for the refresh requested by that call.
// Later trailing refreshes must not resolve or reject an earlier caller.
export function createGeminiBrowserRefreshScheduler(
  deps: GeminiBrowserRefreshSchedulerDeps,
): GeminiBrowserRefreshScheduler {
  let activeRefresh: Promise<void> | null = null;
  let trailingRequested = false;
  let trailingPromise: Promise<void> | null = null;
  let resolveTrailing: (() => void) | null = null;
  let rejectTrailing: ((error: unknown) => void) | null = null;

  async function runRefreshOnce() {
    const [statusResult, runsResult] = await Promise.allSettled([
      deps.loadStatus(),
      deps.loadRuns(),
    ]);

    if (statusResult.status === "fulfilled") {
      deps.applyStatus(statusResult.value);
      deps.applyStatusError(null);
      deps.applyMessage(statusResult.value.latest_message ?? "");
    } else {
      const formatted = deps.formatError(
        "loading Gemini browser provider status",
        statusResult.reason,
      );
      deps.applyStatusError(formatted);
      deps.applyMessage(formatted);
    }

    if (runsResult.status === "fulfilled") {
      deps.applyRuns(runsResult.value.runs);
      deps.applyRunsError(null);
      deps.syncActivePromptResult(runsResult.value.runs);
    } else {
      deps.applyRunsError(
        deps.formatError("loading Gemini browser run history", runsResult.reason),
      );
    }
  }

  function takeTrailingRequest() {
    const resolve = resolveTrailing;
    const reject = rejectTrailing;
    trailingRequested = false;
    trailingPromise = null;
    resolveTrailing = null;
    rejectTrailing = null;
    return { resolve, reject };
  }

  function finishRefresh(refresh: Promise<void>) {
    if (activeRefresh === refresh) {
      activeRefresh = null;
    }
    if (trailingRequested) {
      const trailing = takeTrailingRequest();
      const trailingRefresh = startRefreshForCall();
      void trailingRefresh.then(
        () => trailing.resolve?.(),
        (error) => trailing.reject?.(error),
      );
    }
  }

  function startRefreshForCall(): Promise<void> {
    const refresh = runRefreshOnce();
    activeRefresh = refresh;

    void refresh.then(
      () => finishRefresh(refresh),
      () => finishRefresh(refresh),
    );

    return refresh;
  }

  function ensureTrailingPromise() {
    if (!trailingPromise) {
      trailingPromise = new Promise<void>((resolve, reject) => {
        resolveTrailing = resolve;
        rejectTrailing = reject;
      });
    }
    return trailingPromise;
  }

  function scheduleRefresh() {
    if (activeRefresh) {
      trailingRequested = true;
      return ensureTrailingPromise();
    }
    return startRefreshForCall();
  }

  return { scheduleRefresh };
}
```

- [ ] **Step 4: Run scheduler tests**

Run:

```powershell
npm test -- src/lib/gemini-browser-refresh-scheduler.test.ts
```

Expected: PASS.

- [ ] **Step 5: Commit Task 5**

Run:

```powershell
git add src/lib/gemini-browser-refresh-scheduler.ts src/lib/gemini-browser-refresh-scheduler.test.ts
git commit -m "feat: add Gemini Browser refresh scheduler"
```

---

### Task 6: Settings Panel Uses Invalidation-Only Events And Shared Scheduler

**Files:**
- Modify: `src/lib/components/settings/gemini-browser-provider-panel.svelte`
- Modify: `src/lib/gemini-browser-provider-panel.test.ts`

- [ ] **Step 1: Write failing source-contract tests for the panel**

In `src/lib/gemini-browser-provider-panel.test.ts`, replace:

```ts
it("treats Resume as an open-or-reattach command that returns provider status", () => {
  expect(componentSource).toContain("status = await geminiBridgeResume(browserConfig());");
});
```

with:

```ts
it("routes mount, commands, and run-change events through the shared refresh scheduler", () => {
  expect(componentSource).toContain("createGeminiBrowserRefreshScheduler");
  expect(componentSource).toContain("const refreshScheduler");
  expect(componentSource).toContain("function scheduleRefresh()");
  expect(componentSource).toContain("function scheduleRefreshInBackground()");
  expect(componentSource).toContain("void scheduleRefresh().catch(reportUnexpectedRefreshError);");
  expect(componentSource).toContain("await scheduleRefresh();");
  expect(componentSource).toContain("listenToGeminiBrowserRunChanges");
  expect(componentSource).not.toContain("listenToGeminiBrowserRuns");
  expect(componentSource).not.toContain("payload.message");
  expect(componentSource).not.toContain("payload.status");
  expect(componentSource).not.toContain("payload.run_updated_at");
});

it("does not assign authoritative panel state from command return values", () => {
  expect(componentSource).not.toContain("status = await geminiBridgeOpenBrowser");
  expect(componentSource).not.toContain("status = await geminiBridgeResume");
  expect(componentSource).not.toContain("result = await geminiBridgeSendSingle");
});
```

Update the existing config test to replace:

```ts
expect(componentSource).toContain("geminiBridgeStatus(browserConfig())");
```

with:

```ts
expect(componentSource).toContain("loadStatus: () => geminiBridgeStatus(browserConfig())");
```

- [ ] **Step 2: Run failing panel tests**

Run:

```powershell
npm test -- src/lib/gemini-browser-provider-panel.test.ts
```

Expected: FAIL because the component still uses raw `refresh()`, old listener name, and direct command assignments.

- [ ] **Step 3: Update imports and local state in the panel**

In `src/lib/components/settings/gemini-browser-provider-panel.svelte`, replace `listenToGeminiBrowserRuns` import with `listenToGeminiBrowserRunChanges`.

Add:

```ts
import { createGeminiBrowserRefreshScheduler } from "$lib/gemini-browser-refresh-scheduler";
```

Add a history error state next to `statusLoadError`:

```ts
let runHistoryLoadError = $state<string | null>(null);
```

- [ ] **Step 4: Replace `refresh()` with the scheduler**

Delete the existing `async function refresh()` and add:

```ts
const refreshScheduler = createGeminiBrowserRefreshScheduler({
  loadStatus: () => geminiBridgeStatus(browserConfig()),
  loadRuns: () => geminiBridgeListRuns(8),
  applyStatus: (nextStatus) => {
    status = nextStatus;
  },
  applyRuns: (nextRuns) => {
    runs = nextRuns;
  },
  applyStatusError: (nextError) => {
    statusLoadError = nextError;
  },
  applyRunsError: (nextError) => {
    runHistoryLoadError = nextError;
  },
  applyMessage: (nextMessage) => {
    message = nextMessage;
  },
  syncActivePromptResult: (nextRuns) => {
    syncActivePromptResult(nextRuns);
  },
  formatError: formatAppError,
});

function scheduleRefresh() {
  return refreshScheduler.scheduleRefresh();
}

function reportUnexpectedRefreshError(error: unknown) {
  message = formatAppError("refreshing Gemini browser provider", error);
}

function scheduleRefreshInBackground() {
  void scheduleRefresh().catch(reportUnexpectedRefreshError);
}
```

Replace all `refresh()` calls:

- `void refresh();` -> `scheduleRefreshInBackground();`
- `await refresh();` -> `await scheduleRefresh();`
- setup check refresh action -> `await scheduleRefresh();`

- [ ] **Step 5: Stop command handlers from direct authoritative assignments**

In `openBrowser`, replace:

```ts
status = await geminiBridgeOpenBrowser(browserConfig());
statusLoadError = null;
message = status.latest_message ?? "Browser opened.";
```

with:

```ts
const opened = await geminiBridgeOpenBrowser(browserConfig());
message = opened.latest_message ?? "Browser opened.";
await scheduleRefresh();
```

In `resumeProvider`, replace:

```ts
status = await geminiBridgeResume(browserConfig());
message = status.latest_message ?? "Browser resumed.";
await refresh();
```

with:

```ts
const resumed = await geminiBridgeResume(browserConfig());
message = resumed.latest_message ?? "Browser resumed.";
await scheduleRefresh();
```

In `sendTestPrompt`, replace:

```ts
result = await geminiBridgeSendSingle({
  runId,
  prompt: prompt.trim(),
  source: "settings_test",
  artifactMode: "reduced",
  browserConfig: browserConfig(),
});
activeTestRunId = null;
message = result.message ?? result.status;
await refresh();
```

with:

```ts
const completed = await geminiBridgeSendSingle({
  runId,
  prompt: prompt.trim(),
  source: "settings_test",
  artifactMode: "reduced",
  browserConfig: browserConfig(),
});
message = completed.message ?? completed.status;
await scheduleRefresh();
```

Do not set `result` or clear `activeTestRunId` directly here; `syncActivePromptResult(log.runs)` in the scheduler-driven refresh owns that transition.

Update `startCdpChrome` and `stopProvider` to call `await scheduleRefresh();`.

- [ ] **Step 6: Update event listener to invalidation-only**

Replace the `onMount` listener block:

```ts
scheduleRefreshInBackground();
void listenToGeminiBrowserRuns(({ payload }) => {
  if (disposed) return;
  message = payload.message ?? payload.status;
  scheduleRefreshInBackground();
}).then((detach) => {
```

with:

```ts
scheduleRefreshInBackground();
void listenToGeminiBrowserRunChanges(() => {
  if (disposed) return;
  scheduleRefreshInBackground();
}).then((detach) => {
```

The callback intentionally does not read `payload`, including `payload.run_updated_at`.

- [ ] **Step 7: Render run-history error separately**

Near the run-history heading in the `<section class="runs-list" aria-label="Run history">`, add:

```svelte
{#if runHistoryLoadError}
  <p class="error-text">{runHistoryLoadError}</p>
{/if}
```

Add this style near the other run-history/status styles:

```css
.error-text {
  color: var(--destructive);
  font-size: 0.85rem;
  margin: 0;
}
```

- [ ] **Step 8: Run panel tests**

Run:

```powershell
npm test -- src/lib/gemini-browser-provider-panel.test.ts src/lib/gemini-browser-provider-panel-state.test.ts src/lib/gemini-browser-refresh-scheduler.test.ts
```

Expected: PASS.

- [ ] **Step 9: Commit Task 6**

Run:

```powershell
git add src/lib/components/settings/gemini-browser-provider-panel.svelte src/lib/gemini-browser-provider-panel.test.ts
git commit -m "refactor: route Gemini Browser panel refresh through scheduler"
```

---

### Task 7: Full Verification And Documentation Check

**Files:**
- Verify only unless a previous task exposed a mismatch.

- [ ] **Step 1: Search for legacy public names and state-bearing event payloads**

Run:

```powershell
rg -n "GeminiBrowserRunEvent|listenToGeminiBrowserRuns|GEMINI_BROWSER_RUN_EVENT|payload\\.status|payload\\.message|payload\\.queue_position" src-tauri/src/gemini_browser src/lib/api/gemini-browser.ts src/lib/api/gemini-browser.test.ts src/lib/types/gemini-browser.ts src/lib/components/settings/gemini-browser-provider-panel.svelte src/lib/gemini-browser-provider-panel.test.ts -S
rg -n "queue_position" src-tauri/src/gemini_browser/types.rs src-tauri/src/gemini_browser/commands.rs src/lib/api/gemini-browser.ts src/lib/types/gemini-browser.ts src/lib/components/settings/gemini-browser-provider-panel.svelte -S
```

Expected:

- No `GeminiBrowserRunEvent`, `listenToGeminiBrowserRuns`, or `GEMINI_BROWSER_RUN_EVENT`.
- No Gemini Browser event callback reading `payload.status`, `payload.message`, or `payload.queue_position`.
- No `queue_position` in the Gemini Browser event contract, commands event payload, TypeScript event type, or settings panel.
- Ignore unrelated `queue_position` matches outside these paths, such as analysis/LLM types, tests, or Apalis queue internals like `QueuedGeminiBrowserJob`.

- [ ] **Step 2: Run focused Rust tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-invalidation --lib gemini_browser
```

Expected: PASS.

- [ ] **Step 3: Run focused frontend tests**

Run:

```powershell
npm test -- src/lib/api/gemini-browser.test.ts src/lib/gemini-browser-refresh-scheduler.test.ts src/lib/gemini-browser-provider-panel.test.ts src/lib/gemini-browser-provider-panel-state.test.ts src/lib/gemini-browser-run-inspector.test.ts src/lib/gemini-browser-setup-status.test.ts
```

Expected: PASS.

- [ ] **Step 4: Run prompt-pack regression tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --target-dir src-tauri/target/codex-gemini-invalidation --lib prompt_packs
```

Expected: PASS.

- [ ] **Step 5: Run Svelte check**

Run:

```powershell
npm run check
```

Expected: PASS.

- [ ] **Step 6: Optional manual verification**

If a Tauri app and Chrome CDP setup are available:

1. Open Settings -> Browser Providers.
2. Start Chrome or Resume.
3. Send a test prompt.
4. Verify Run history and Run inspector update.
5. Reload or explicitly refresh the panel and verify the same run remains visible from persisted run-log data.
6. Optionally inspect the Tauri event payload with DevTools/logging/breakpoint and confirm it contains only `run_id` and `run_updated_at`.
7. Run YouTube Summary with Gemini Browser and verify it completes.

- [ ] **Step 7: Final commit if verification caused follow-up fixes**

If Steps 1-6 required follow-up changes, commit them:

```powershell
git status --short
git add src-tauri/src/gemini_browser/types.rs src-tauri/src/gemini_browser/mod.rs src-tauri/src/gemini_browser/commands.rs src-tauri/src/gemini_browser/jobs.rs src/lib/types/gemini-browser.ts src/lib/api/gemini-browser.ts src/lib/api/gemini-browser.test.ts src/lib/gemini-browser-refresh-scheduler.ts src/lib/gemini-browser-refresh-scheduler.test.ts src/lib/components/settings/gemini-browser-provider-panel.svelte src/lib/gemini-browser-provider-panel.test.ts
git commit -m "fix: finalize Gemini Browser run-change invalidation"
```

If no files changed, do not create an empty commit.
