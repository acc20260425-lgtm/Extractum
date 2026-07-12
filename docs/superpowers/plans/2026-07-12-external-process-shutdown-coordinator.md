# External Process Shutdown Coordinator Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Delegate the production Tauri exit hook to the tested shutdown coordinator while fixing repeated-exit handling, cleanup panic isolation, and the shared three-second deadline.

**Architecture:** `external_process.rs` exposes a synchronous `start` decision and a one-shot async `ShutdownRun`. Start captures an injected monotonic deadline and schedules the OS-thread watchdog; the run waits for admission and executes boxed cleanup tasks concurrently within the same deadline. `lib.rs` only maps Tauri events and managed state into this framework-neutral API.

**Tech Stack:** Rust 2021, Tokio, Tauri 2, parking_lot, Vitest raw-source contracts, Markdown.

## Global Constraints

- The three-second graceful deadline starts at the first accepted exit request and includes admission waiting.
- The independent watchdog exits at approximately four seconds with the first requested exit code.
- `Started` and `AlreadyShuttingDown` call `api.prevent_exit()`; `Completed` passes through.
- YouTube, Gemini sidecar, and owned Chrome cleanup start concurrently after admission drains.
- A cleanup error or panic does not prevent other cleanup tasks from running.
- Coordinator warnings contain stage only; subsystem warnings contain operation ID plus stage. Neither includes secrets, arguments, errors, output, or paths.
- Do not add Tauri types to `external_process.rs`, an injectable warning sink, or a pure Tauri outcome mapper solely for tests.
- Do not change Job Objects, process ownership, sidecar protocol, or the configured durations.
- Reuse the existing CRLF/LF `normalized` helper in raw-source tests.
- Inspect but do not edit `docs/value-registry.md`; this adds no persisted/API/UI/domain values.
- Preserve and never stage `.claude/settings.local.json`.

---

### Task 1: Synchronous Shutdown Start Decision

**Files:**
- Modify: `src-tauri/src/external_process.rs:1-180`

**Interfaces:**
- Consumes: existing shutdown state, phase, timing, exit callback, and watchdog scheduler.
- Produces:
  - `type MonotonicClock = Arc<dyn Fn() -> std::time::Instant + Send + Sync>`
  - `enum ShutdownStart { Started(ShutdownRun), AlreadyShuttingDown, Completed }`
  - `struct ShutdownRun { state, deadline, clock, exit }`
  - `ExternalProcessShutdownState::start(code, timing, scheduler, exit, clock) -> ShutdownStart`
  - `system_monotonic_clock()` and `os_thread_watchdog_scheduler()`.

- [ ] **Step 1: Add RED tests for the three outcomes and single watchdog scheduling**

Add a scheduler fixture that stores timings and the `WatchdogTask` without running it:

```rust
fn recording_scheduler() -> (
    WatchdogScheduler,
    Arc<Mutex<Vec<ShutdownTiming>>>,
    Arc<Mutex<Option<WatchdogTask>>>,
) {
    let timings = Arc::new(Mutex::new(Vec::new()));
    let watchdog = Arc::new(Mutex::new(None));
    let recorded_timings = timings.clone();
    let recorded_watchdog = watchdog.clone();
    let scheduler: WatchdogScheduler = Arc::new(move |timing, task| {
        recorded_timings.lock().unwrap().push(timing);
        *recorded_watchdog.lock().unwrap() = Some(task);
    });
    (scheduler, timings, watchdog)
}
```

Add these tests inside `external_process.rs::tests`:

```rust
#[test]
fn start_returns_started_and_schedules_one_watchdog() {
    let state = ExternalProcessShutdownState::new();
    let (scheduler, timings, watchdog) = recording_scheduler();
    let result = state.start(
        Some(23), ShutdownTiming::default(), &scheduler,
        Arc::new(|_| {}), Arc::new(Instant::now),
    );
    assert!(matches!(result, ShutdownStart::Started(_)));
    assert_eq!(timings.lock().unwrap().as_slice(), &[ShutdownTiming::default()]);
    assert!(watchdog.lock().unwrap().is_some());
    assert_eq!(state.exit_code(), 23);
    assert!(state.try_admit().is_err());
}

#[test]
fn repeated_start_does_not_replace_code_or_schedule_again() {
    let state = ExternalProcessShutdownState::new();
    let (scheduler, timings, _) = recording_scheduler();
    let exit: ExitCallback = Arc::new(|_| {});
    let clock: MonotonicClock = Arc::new(Instant::now);
    assert!(matches!(state.start(Some(23), ShutdownTiming::default(), &scheduler, exit.clone(), clock.clone()), ShutdownStart::Started(_)));
    assert!(matches!(state.start(Some(99), ShutdownTiming::default(), &scheduler, exit, clock), ShutdownStart::AlreadyShuttingDown));
    assert_eq!(timings.lock().unwrap().len(), 1);
    assert_eq!(state.exit_code(), 23);
}
```

```rust
#[test]
fn start_reports_completed_after_watchdog_claims_exit() {
    let state = ExternalProcessShutdownState::new();
    let (scheduler, _, watchdog) = recording_scheduler();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let recorded_calls = calls.clone();
    let exit: ExitCallback = Arc::new(move |code| recorded_calls.lock().unwrap().push(code));
    let clock: MonotonicClock = Arc::new(Instant::now);

    let _ = state.start(Some(23), ShutdownTiming::default(), &scheduler, exit.clone(), clock.clone());
    watchdog.lock().unwrap().take().unwrap()();

    assert!(matches!(
        state.start(Some(99), ShutdownTiming::default(), &scheduler, exit, clock),
        ShutdownStart::Completed
    ));
    assert_eq!(*calls.lock().unwrap(), vec![23]);
}
```

- [ ] **Step 2: Run RED**

Run: `cargo test --manifest-path src-tauri/Cargo.toml external_process::tests::start_ -- --nocapture`

Expected: compilation fails because `MonotonicClock`, `ShutdownStart`, and `start` do not exist.

- [ ] **Step 3: Implement the minimum start API**

Under the admission mutex, map `Running` to `Started`, `ShuttingDown` to `AlreadyShuttingDown`, and `Completed` to `Completed`. Only the `Running` branch closes admission and saves `code.unwrap_or(0)`. Capture `deadline = clock() + timing.graceful`, schedule the watchdog synchronously, and return the single-use run token.

Implement the production helpers exactly as follows:

```rust
pub(crate) fn system_monotonic_clock() -> MonotonicClock {
    Arc::new(Instant::now)
}

pub(crate) fn os_thread_watchdog_scheduler() -> WatchdogScheduler {
    Arc::new(|timing, watchdog| {
        std::thread::spawn(move || {
            std::thread::sleep(timing.watchdog);
            watchdog();
        });
    })
}
```

- [ ] **Step 4: Run GREEN**

Run: `cargo test --manifest-path src-tauri/Cargo.toml external_process -- --nocapture`

Expected: all coordinator tests pass. Temporary unused warnings for the not-yet-consumed run token are acceptable only until Task 2.

- [ ] **Step 5: Commit**

```powershell
git add src-tauri/src/external_process.rs
git commit -m "refactor: centralize shutdown start decision"
```

---

### Task 2: Shared Deadline and Isolated Cleanup

**Files:**
- Modify: `src-tauri/src/external_process.rs:1-390`
- Modify: `src-tauri/Cargo.toml:38` and append a new `[dev-dependencies]` section

**Interfaces:**
- Consumes: `ShutdownRun`, `MonotonicClock`, `ShutdownCleanup`, admission permits, and watchdog completion.
- Produces:
  - `type CleanupFactory = Box<dyn FnOnce() -> Vec<ShutdownCleanup> + Send + 'static>`
  - `ShutdownRun::coordinate(self, cleanup_factory: CleanupFactory)`
  - `warn_shutdown_coordinator_stage(stage: &'static str)`
  - private `remaining()` and atomic `complete_and_exit()`.

- [ ] **Step 1: Enable deterministic Tokio time and add a clock fixture**

Leave the production dependency at line 38 unchanged:

```toml
tokio = { version = "1", features = ["full"] }
```

Add a test-only dependency section:

```toml
[dev-dependencies]
tokio = { version = "1", features = ["test-util"] }
```

Cargo feature unification makes `test-util` available to test targets without compiling it into normal release builds.

```rust
fn tokio_aligned_clock() -> MonotonicClock {
    let std_origin = Instant::now();
    let tokio_origin = tokio::time::Instant::now();
    Arc::new(move || std_origin + tokio::time::Instant::now().duration_since(tokio_origin))
}
```

- [ ] **Step 2: Add named RED deadline tests**

Add `admission_wait_consumes_the_shared_graceful_budget`: with `#[tokio::test(start_paused = true)]`, acquire an admission permit before `start`, retain the recorded watchdog task, spawn `run.coordinate(factory)`, advance two seconds, drop the permit, then advance one second. The factory returns a cleanup sleeping two seconds. Assert final exit occurs at the original three-second boundary, not two seconds later.

Call `tokio::task::yield_now().await` immediately after spawning the coordinator and after dropping the permit so the tested futures register their timers before virtual time advances.

After normal coordinator completion, execute the recorded watchdog task and assert the callback remains `vec![0]`; this directly tests coordinator-wins-then-watchdog in addition to the existing watchdog-wins and watchdog-vs-watchdog cases.

```rust
task.await.unwrap();
assert_eq!(*calls.lock().unwrap(), vec![0]);
watchdog.lock().unwrap().take().unwrap()();
assert_eq!(*calls.lock().unwrap(), vec![0]);
```

Add `exhausted_admission_budget_skips_the_cleanup_factory`, a second paused test that keeps the permit for all three seconds and asserts the cleanup factory's `AtomicBool` remains false when the coordinator exits.

- [ ] **Step 3: Add RED concurrency and isolation test**

Create three boxed cleanup futures sharing a four-party `tokio::sync::Barrier`. Each increments a `started` counter before the barrier; after release one succeeds, one returns `ShutdownCleanupError::Failed`, and one panics. Assert all three started, the two non-panicking tasks settled, the coordinator task itself completed, and the exit callback ran once.

- [ ] **Step 4: Run RED**

Run: `cargo test --manifest-path src-tauri/Cargo.toml external_process::tests::admission_wait_consumes_the_shared_graceful_budget -- --nocapture`

Expected: compilation fails because `CleanupFactory` and `coordinate` do not exist.

- [ ] **Step 5: Implement absolute-budget coordination**

Use only `self.deadline.checked_duration_since((self.clock)())`. Apply the first remaining duration to `wait_for_startups`; on timeout emit `admission_deadline_elapsed`, claim completion, and exit without invoking the factory. Recompute remaining after admission. Spawn every returned cleanup in a `tokio::task::JoinSet`, logging only `cleanup_failed` or `cleanup_panicked`, and bound the entire join loop by the recomputed remaining duration. On timeout emit `cleanup_deadline_elapsed`.

The warning helper is deliberately stage-only:

```rust
pub(crate) fn warn_shutdown_coordinator_stage(stage: &'static str) {
    eprintln!("external process shutdown warning: stage={stage}");
}
```

Implement `complete_and_exit` by atomically changing only `ShuttingDown` to `Completed`, copying the saved code, releasing the mutex, and then invoking the callback. Make `run_watchdog` use this same gate.

- [ ] **Step 6: Remove superseded internals and migrate their tests**

Remove `run_cleanup_steps`; its concurrency and isolation behavior now belongs to `coordinate`. `start()` fully absorbs the state transition currently owned by `begin_shutdown`, and `complete_and_exit()` replaces `complete`; remove both old methods from the production impl. Migrate every old test that calls `begin_shutdown` or `complete` to `start` plus `recording_scheduler`/recorded exit callbacks. Keep admission wakeup/race tests, but begin their shutdown through `start`. After this step, neither `begin_shutdown` nor `complete` exists under production or `#[cfg(test)]`.

- [ ] **Step 7: Run GREEN and commit**

```powershell
cargo test --manifest-path src-tauri/Cargo.toml external_process -- --nocapture
git add src-tauri/src/external_process.rs src-tauri/Cargo.toml
git commit -m "fix: enforce shared external process shutdown deadline"
```

Expected: all coordinator tests pass; the joined panic fixture may print a panic line but must not fail its parent test.

---

### Task 3: Thin Tauri Adapter and Source Contract

**Files:**
- Modify: `src/lib/external-process-lifecycle-contract.test.ts:78-101`
- Modify: `src-tauri/src/lib.rs:1-3,371-402`

**Interfaces:**
- Consumes: Task 1-2 coordinator API and existing YouTube/Gemini cleanup functions.
- Produces: exhaustive Tauri outcome handling with no duplicate timing, admission, or completion logic.

- [ ] **Step 1: Write the RED source contract**

Replace old expectations for timeout constants and `std::thread::spawn` in `lib.rs` with:

```ts
expect(lib).toMatch(/ShutdownStart::Started\(run\)\s*=>\s*\{[\s\S]*?api\.prevent_exit\(\)[\s\S]*?run\.coordinate/);
expect(lib).toMatch(/ShutdownStart::AlreadyShuttingDown\s*=>\s*api\.prevent_exit\(\)/);
expect(lib).toMatch(/ShutdownStart::Completed\s*=>\s*\{\}/);
expect(lib).not.toContain("GRACEFUL_SHUTDOWN_TIMEOUT");
expect(lib).not.toContain("SHUTDOWN_WATCHDOG_TIMEOUT");
expect(lib).not.toContain("shutdown.wait_for_startups()");
expect(lib).not.toContain("shutdown.complete()");
expect(coordinator).not.toContain("fn begin_shutdown(");
expect(coordinator).not.toMatch(/pub\(crate\) fn complete\(/);
```

Extend the warning contract:

```ts
const coordinatorWarning = coordinator.match(
  /fn warn_shutdown_coordinator_stage\(stage: &'static str\)[\s\S]*?^}/m,
)?.[0];
expect(coordinatorWarning).toBeDefined();
expect(coordinatorWarning).not.toMatch(
  /operation_id|args|cookie|prompt|stdout|stderr|profile|executable|path|error/i,
);
const shutdownRunImpl = coordinator.match(/impl ShutdownRun \{[\s\S]*?^}/m)?.[0];
expect(shutdownRunImpl).toBeDefined();
expect(shutdownRunImpl).toContain("warn_shutdown_coordinator_stage(");
expect(shutdownRunImpl).not.toContain("warn_shutdown_stage(");
```

Keep the existing subsystem `warn_shutdown_stage(operation_id, stage)` assertions and `normalized` helper.

- [ ] **Step 2: Run RED**

Run: `npm.cmd run test -- src/lib/external-process-lifecycle-contract.test.ts`

Expected: FAIL because `lib.rs` still owns watchdog, admission wait, timeout, and completion.

- [ ] **Step 3: Implement exhaustive adapter delegation**

In `RunEvent::ExitRequested`, create the production scheduler, monotonic clock, and `AppHandle::exit` callback, then match the start result:

```rust
match shutdown.start(code, ShutdownTiming::default(), &scheduler, exit, clock) {
    ShutdownStart::Started(run) => {
        api.prevent_exit();
        let registry = app.state::<YoutubeProcessRegistry>().inner().clone();
        let handle = app.clone();
        tauri::async_runtime::spawn(async move {
            run.coordinate(Box::new(move || {
                let sidecar_handle = handle.clone();
                let chrome_handle = handle.clone();
                let youtube: ShutdownCleanup = Box::pin(async move {
                    registry.cancel_and_wait().await;
                    Ok(())
                });
                let sidecar: ShutdownCleanup = Box::pin(async move {
                    let state = sidecar_handle.state::<GeminiBrowserState>();
                    gemini_browser::shutdown_sidecar(&sidecar_handle, state.inner()).await;
                    Ok(())
                });
                let chrome: ShutdownCleanup = Box::pin(async move {
                    let state = chrome_handle.state::<GeminiBrowserState>();
                    gemini_browser::shutdown_cdp_chrome(state.inner()).await;
                    Ok(())
                });
                vec![youtube, sidecar, chrome]
            })).await;
        });
    }
    ShutdownStart::AlreadyShuttingDown => api.prevent_exit(),
    ShutdownStart::Completed => {}
}
```

All three current cleanup functions return `()`, so each boxed adapter returns `Ok(())` after awaiting it. If a signature changes before execution, map a real failure to `ShutdownCleanupError::Failed` without logging error text.

- [ ] **Step 4: Remove duplicate orchestration imports and run GREEN**

Remove timeout constants from `lib.rs`. Run:

```powershell
npm.cmd run test -- src/lib/external-process-lifecycle-contract.test.ts
cargo test --manifest-path src-tauri/Cargo.toml external_process -- --nocapture
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: all commands pass and `cargo check` reports no dead-code warning for coordinator timing, cleanup, scheduler, or start/run types.

- [ ] **Step 5: Commit**

```powershell
git add src-tauri/src/lib.rs src-tauri/src/external_process.rs src/lib/external-process-lifecycle-contract.test.ts
git commit -m "fix: delegate Tauri exit to shutdown coordinator"
```

---

### Task 4: Documentation and Full Verification

**Files:**
- Modify: `docs/project.md:445-449`
- Inspect only: `docs/value-registry.md`

**Interfaces:**
- Consumes: completed coordinator behavior.
- Produces: current-state deadline documentation and final evidence.

- [ ] **Step 1: Update current-state documentation**

Replace the shutdown sentence in `docs/project.md` with:

```markdown
Application exit closes external-process admission and starts one shared three-second graceful deadline at the first accepted exit request. Waiting for in-progress spawn/install permits and concurrent cleanup of YouTube, the Gemini sidecar, and owned Chrome all consume that same budget; an independent OS-thread watchdog enforces an approximately four-second hard cap with the original exit code.
```

Keep the surrounding Job Object, bundled sidecar, `externalBin`, and tainted transport text.

- [ ] **Step 2: Inspect value registry without editing it**

Run: `Select-String -Path docs/value-registry.md -Pattern 'ShutdownPhase|ShuttingDown|external process'`

Expected: no entry is required because coordinator phases/outcomes are runtime-only Rust control values.

- [ ] **Step 3: Run format and focused verification**

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
git diff --check
npm.cmd run test -- src/lib/external-process-lifecycle-contract.test.ts
cargo test --manifest-path src-tauri/Cargo.toml external_process -- --nocapture
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: every command exits 0 and no coordinator dead-code warning appears.

- [ ] **Step 4: Run full verification**

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
npm.cmd run test
npm.cmd run check
```

Expected: full Rust and Vitest suites pass; Svelte check reports zero errors and zero warnings.

- [ ] **Step 5: Inspect ownership and commit docs/final corrections**

```powershell
git status --short --untracked-files=all
git diff --stat
git diff --check
git add docs/project.md
git commit -m "docs: clarify external process shutdown deadline"
```

Expected: `.claude/settings.local.json` stays untracked and unstaged. If Task 3 needed final focused corrections, stage only their explicit files in addition to `docs/project.md`.

- [ ] **Step 6: Record completion evidence**

Report exact outputs/pass counts for focused coordinator tests, full Rust tests, focused and full Vitest, `cargo check`, and `npm.cmd run check`. State that no new live GUI smoke is required because this slice changes orchestration rather than process launch/containment; the prior release containment evidence remains applicable.
