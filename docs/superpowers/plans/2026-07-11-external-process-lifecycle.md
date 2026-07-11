# External Process Lifecycle Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give Extractum bounded, testable ownership of `yt-dlp`, Gemini sidecar, and CDP Chrome processes across timeout, cancellation, shutdown, and Windows crash containment.

**Architecture:** Subsystems retain typed process ownership. A shared admission/shutdown core closes new spawns and coordinates concurrent cleanup; per-process Windows Job Objects contain raw-handle process trees. `yt-dlp` moves to managed tasks, the release sidecar moves from Tauri shell events to a unified Tokio JSONL transport, and Tauri exit events use a three-second graceful budget plus a four-second OS-thread watchdog.

**Tech Stack:** Rust, Tokio, `tokio-util::CancellationToken`, Tauri 2, `windows-sys`, SQLite-independent runtime state, Vitest source contracts, Windows release GUI verification.

## Global Constraints

- Preserve public Tauri command names, frontend DTOs, existing `yt-dlp` error strings, and source-job status semantics.
- Never log process arguments, cookies, prompts, stdout/stderr, profile directories, or executable paths.
- Use raw owned process handles for Job assignment; never reopen Tokio/std children by PID.
- Preserve `bundle.externalBin`; remove `tauri-plugin-shell` only after bundled Tokio launch works.
- Graceful shutdown has one three-second budget; the independent watchdog exits at approximately four seconds with the original exit code.
- Windows pre-assignment descendants and non-Windows crash containment remain documented limitations.
- Follow TDD for every task and commit only task-owned files.

---

### Task 1: Admission and shutdown coordinator core

**Files:**
- Create: `src-tauri/src/external_process.rs`
- Modify: `src-tauri/src/lib.rs`
- Create: `src/lib/external-process-lifecycle-contract.test.ts`

**Interfaces:**
- Produces: `ExternalProcessShutdownState`, `ShutdownPhase`, `AdmissionPermit`, `ShutdownTiming`, and test-injectable exit/watchdog callbacks.
- Consumes: no subsystem process types.

- [ ] **Step 1: Write failing Rust tests** for admission permit races, one transition to shutdown, rejection after closure, preserved first exit code, repeated requests, concurrent cleanup execution, failure isolation, and watchdog fallback.

```rust
let state = ExternalProcessShutdownState::new();
let permit = state.try_admit().expect("running admits");
assert!(state.begin_shutdown(Some(23)));
assert!(state.try_admit().is_err());
drop(permit);
state.wait_for_startups().await;
assert_eq!(state.exit_code(), 23);
```

Use injected `Arc<dyn Fn(i32) + Send + Sync>` and a test watchdog scheduler so tests record exit calls instead of terminating the test process.

- [ ] **Step 2: Write the failing source contract.** Require `mod external_process;`, managed state registration, phase names `running/shutting_down/completed`, no persisted/API phase strings, and constants for three/four-second budgets.

- [ ] **Step 3: Run RED.**

Run: `cargo test --manifest-path src-tauri/Cargo.toml external_process -- --nocapture`
Run: `npm.cmd run test -- src/lib/external-process-lifecycle-contract.test.ts`
Expected: FAIL because the module and contracts do not exist.

- [ ] **Step 4: Implement the minimal coordinator core.** Use a mutex-protected admission record (`open`, active startup count, first exit code) plus `Notify`; the phase itself may be an enum behind the same lock. `AdmissionPermit::drop` decrements and notifies. Do not spawn real OS threads in core unit tests.

```rust
pub(crate) enum ShutdownPhase { Running, ShuttingDown, Completed }

pub(crate) struct ExternalProcessShutdownState {
    inner: Mutex<AdmissionState>,
    startup_idle: Notify,
}

pub(crate) struct AdmissionPermit { state: Arc<ExternalProcessShutdownState> }
```

- [ ] **Step 5: Run GREEN and commit.**

Run: `cargo test --manifest-path src-tauri/Cargo.toml external_process -- --nocapture`
Run: `npm.cmd run test -- src/lib/external-process-lifecycle-contract.test.ts`
Expected: PASS.

```powershell
git add src-tauri/src/external_process.rs src-tauri/src/lib.rs src/lib/external-process-lifecycle-contract.test.ts
git commit -m "feat: add external process shutdown coordinator"
```

### Task 2: Windows Job Object containment

**Files:**
- Create: `src-tauri/src/process_tree.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src/lib/external-process-lifecycle-contract.test.ts`

**Interfaces:**
- Produces: `ProcessTreeGuard::assign_tokio(&tokio::process::Child)`, `ProcessTreeGuard::assign_std(&std::process::Child)`, and `terminate()`; non-Windows no-op guard with the same API.
- Consumes: already-owned raw child handles only.

- [ ] **Step 1: Add failing source-contract tests** requiring `windows-sys` features, `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`, raw-handle assignment, and forbidding `OpenProcess` plus `.pid()` in `process_tree.rs`.

```ts
expect(processTreeSource).not.toContain("OpenProcess");
expect(processTreeSource).not.toMatch(/\.pid\s*\(/);
expect(processTreeSource).toContain("JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE");
```

- [ ] **Step 2: Add failing Windows Rust tests** for job creation, direct-child assignment, post-assignment descendant termination, `terminate()` idempotence, and RAII close. Spawn an OS-provided inert fixture that waits for a signal before creating its descendant, ensuring the descendant is created after assignment.
- [ ] **Step 3: Run RED.**

Run: `npm.cmd run test -- src/lib/external-process-lifecycle-contract.test.ts`
Run: `cargo test --manifest-path src-tauri/Cargo.toml process_tree -- --nocapture`
Expected: FAIL because containment is absent.

- [ ] **Step 4: Add the scoped dependency and implementation.** Add target-specific `windows-sys` features for Foundation, Threading, and Job Objects. On Windows create/set/assign the job using `AsRawHandle`/Tokio raw handle. Close every intermediate handle. On assignment failure kill/reap the just-spawned child in its caller before returning the sanitized error.
- [ ] **Step 5: Implement the non-Windows marker** without process-tree guarantees and without warnings.
- [ ] **Step 6: Run GREEN and commit.**

Run: `cargo test --manifest-path src-tauri/Cargo.toml process_tree -- --nocapture`
Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: PASS.

```powershell
git add src-tauri/Cargo.toml src-tauri/src/process_tree.rs src-tauri/src/lib.rs src/lib/external-process-lifecycle-contract.test.ts
git commit -m "feat: contain Windows child process trees"
```

### Task 3: Managed yt-dlp runtime

**Files:**
- Create: `src-tauri/src/youtube/process_runtime.rs`
- Modify: `src-tauri/src/youtube/mod.rs`
- Modify: `src-tauri/src/youtube/ytdlp.rs`
- Modify: `src-tauri/src/youtube/preview.rs`
- Modify: `src-tauri/src/youtube/metadata.rs`
- Modify: `src-tauri/src/youtube/comments.rs`
- Modify: `src-tauri/src/youtube/captions.rs`
- Modify: `src-tauri/src/youtube/jobs.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Produces: `YoutubeProcessRegistry`, `ManagedYtdlpGuard`, `YtdlpLauncher`, `SpawnedYtdlp`, and managed execution preserving `YtdlpOutput`/`YtdlpRunOptions` semantics. `YtdlpLauncher::spawn` is synchronous and returns an owned child abstraction with piped streams plus boxed async `wait`/`kill_and_wait` methods; production wraps Tokio Child and tests use `FakeYtdlpLauncher`.
- Consumes: `AdmissionPermit`, `ProcessTreeGuard`, existing `hide_console_window`.

- [ ] **Step 1: Write failing registry tests** for reservation-before-spawn, spawn failure rollback, admission race, operation removal, shutdown cancellation, and rejection after shutdown.
- [ ] **Step 2: Write failing managed-runner tests** using an injected launcher for normal output, non-zero exit, NotFound, timeout, dropped caller future, >1 MiB stdout/stderr, and stuck reap.

```rust
let launcher = FakeYtdlpLauncher::with_large_output(1_048_577);
let output = runner.run_with(&launcher, args, options).await?;
assert!(output.stdout.len() > 1_048_576);
assert!(registry.is_empty().await);
```

- [ ] **Step 3: Add cookie-lifetime tests.** Verify the temp file exists while a cancelled child is being reaped, disappears after normal reap, and remains owned by the detached waiter on stuck-reap fallback.
- [ ] **Step 4: Run RED.**

Run: `cargo test --manifest-path src-tauri/Cargo.toml youtube::process_runtime -- --nocapture`
Expected: FAIL because managed runtime is absent.

- [ ] **Step 5: Implement reservation and managed ownership.** Reserve under admission before spawn; rollback on errors. Move child, `ProcessTreeGuard`, cookie file, piped streams, token, and registry completion guard into the managed task before releasing admission.
- [ ] **Step 6: Implement concurrent output draining.** Start stdout/stderr readers before selecting among child wait, timeout, and cancellation. On Windows use `TerminateJobObject`; elsewhere kill direct child. Use a distinct one-second mid-session reap budget and sanitized warning on detach.
- [ ] **Step 7: Thread `&YoutubeProcessRegistry` through the internal functions in `preview.rs`, `metadata.rs`, `comments.rs`, and `captions.rs`, with `jobs.rs` and the existing `AppHandle` command boundaries supplying managed state.** Keep public Tauri commands/DTOs unchanged and do not introduce a global singleton.
- [ ] **Step 8: Run focused and broad YouTube tests.**

Run: `cargo test --manifest-path src-tauri/Cargo.toml youtube -- --nocapture`
Expected: PASS, including existing timeout/error assertions.

- [ ] **Step 9: Commit.**

```powershell
git add src-tauri/src/youtube src-tauri/src/lib.rs
git commit -m "feat: manage yt-dlp process lifecycle"
```

### Task 4: Unified Tokio Gemini sidecar

**Files:**
- Modify: `src-tauri/src/gemini_browser/sidecar.rs`
- Modify: `src-tauri/src/gemini_browser/sidecar_launch.rs`
- Modify: `src-tauri/src/gemini_browser/state.rs`
- Modify: `src-tauri/src/gemini_browser/commands.rs`
- Modify: `src-tauri/src/gemini_browser/jobs.rs`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/lib/external-process-lifecycle-contract.test.ts`

**Interfaces:**
- Produces: one Tokio JSONL transport for Node-script and bundled binary; `shutdown_sidecar`; state-level request cancellation/taint.
- Consumes: admission coordinator, `ProcessTreeGuard`, `hide_console_window`.

- [ ] **Step 1: Add failing launch/path tests** for dev Node command and bundled `current_exe().parent()/gemini-browser-sidecar[.exe]` resolution.
- [ ] **Step 2: Add failing transport tests** for JSONL request/response, concurrent stderr drain, in-flight cancellation releasing the mutex, tainted transport skipping Stop, idle Stop ACK followed by stdin close/EOF exit, and force fallback.
- [ ] **Step 3: Extend source contracts** to require direct Tokio bundled spawn plus `hide_console_window`, forbid `CommandChild`, `CommandEvent`, `request_shell`, and keep `externalBin` in config.
- [ ] **Step 4: Run RED.**

Run: `cargo test --manifest-path src-tauri/Cargo.toml gemini_browser::sidecar -- --nocapture`
Run: `npm.cmd run test -- src/lib/external-process-lifecycle-contract.test.ts`
Expected: FAIL on the legacy shell arm.

- [ ] **Step 5: Unify launch modes.** Resolve a fixed bundled path, spawn both modes with Tokio, pipe stdin/stdout/stderr, hide Windows console, assign Job Object, and install under one admission permit.
- [ ] **Step 6: Implement request cancellation/taint.** Select the protocol future against the state shutdown token. Mark tainted before returning and release the sidecar mutex. Shutdown skips Stop when tainted.
- [ ] **Step 7: Implement idle graceful shutdown.** Send Stop, await ACK, take/drop stdin to deliver EOF, then wait for process exit; fall back to Job termination within budget.
- [ ] **Step 8: Preserve mid-session stop semantics.** Stop/ACK then remove from state; contained Drop initiates tree termination and best-effort reap.
- [ ] **Step 9: Remove dead shell integration.** Delete the shell transport code, `.plugin(tauri_plugin_shell::init())`, and Cargo dependency. Do not remove `bundle.externalBin`.
- [ ] **Step 10: Run sidecar suites and commit.**

Run: `cargo test --manifest-path src-tauri/Cargo.toml gemini_browser -- --nocapture`
Run: `npm.cmd run test -- src/lib/external-process-lifecycle-contract.test.ts`
Expected: PASS.

```powershell
git add src-tauri/src/gemini_browser src-tauri/src/lib.rs src-tauri/Cargo.toml src/lib/external-process-lifecycle-contract.test.ts
git commit -m "refactor: own bundled sidecar with Tokio"
```

### Task 5: CDP Chrome explicit lifecycle

**Files:**
- Modify: `src-tauri/src/gemini_browser/cdp_chrome.rs`
- Modify: `src-tauri/src/gemini_browser/commands.rs`
- Modify: `src-tauri/src/gemini_browser/state.rs`
- Modify: `src/lib/external-process-lifecycle-contract.test.ts`

**Interfaces:**
- Produces: `ChromeCdpProcess::shutdown`; async spawn/install wrapper using `spawn_blocking`.
- Consumes: admission coordinator and `ProcessTreeGuard::assign_std`.

- [ ] **Step 1: Add failing unit tests** with an injected std-child adapter for spawn failure, assignment failure cleanup, explicit shutdown kill/wait once, Drop fallback, and delegated-child exit detection.
- [ ] **Step 2: Extend source contracts** requiring `spawn_blocking` around spawn/assign and kill/wait, and forbidding termination of Chrome not held by `GeminiBrowserState`.
- [ ] **Step 3: Run RED.**

Run: `cargo test --manifest-path src-tauri/Cargo.toml gemini_browser::cdp_chrome -- --nocapture`
Expected: FAIL because explicit shutdown/containment is absent.

- [ ] **Step 4: Implement spawn/install under admission.** Hold permit across `spawn_blocking(Command::spawn + Job assignment)` and insertion into state. If the child exits during CDP readiness/delegation, remove the dead owner and do not claim the unrelated browser.
- [ ] **Step 5: Implement explicit shutdown and Drop fallback.** Normal shutdown takes the owner from state, terminates its job tree, and kill/waits through `spawn_blocking`; Drop stays non-panicking and idempotent.
- [ ] **Step 6: Run GREEN and commit.**

Run: `cargo test --manifest-path src-tauri/Cargo.toml gemini_browser::cdp_chrome -- --nocapture`
Run: `cargo check --manifest-path src-tauri/Cargo.toml`
Expected: PASS.

```powershell
git add src-tauri/src/gemini_browser/cdp_chrome.rs src-tauri/src/gemini_browser/commands.rs src-tauri/src/gemini_browser/state.rs src/lib/external-process-lifecycle-contract.test.ts
git commit -m "feat: manage CDP Chrome lifecycle"
```

### Task 6: Tauri exit-event integration

**Files:**
- Modify: `src-tauri/src/external_process.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/lib/external-process-lifecycle-contract.test.ts`

**Interfaces:**
- Produces: `handle_exit_requested`; concurrent subsystem cleanup; OS-thread watchdog.
- Consumes: YouTube registry shutdown, Gemini sidecar shutdown, Chrome shutdown.

- [ ] **Step 1: Add failing coordinator integration tests** for concurrent subsystem start, shared three-second deadline, force phase, subsystem error isolation, first exit-code preservation, one cleanup task, and OS-thread watchdog callback.
- [ ] **Step 2: Add failing source contract** requiring `RunEvent::ExitRequested`, `prevent_exit`, original `code`, and no terminal `.run(...).expect(...)` shortcut.
- [ ] **Step 3: Run RED.**

Run: `cargo test --manifest-path src-tauri/Cargo.toml external_process -- --nocapture`
Run: `npm.cmd run test -- src/lib/external-process-lifecycle-contract.test.ts`
Expected: FAIL until the event-loop adapter exists.

- [ ] **Step 4: Implement the thin Tauri adapter.** First request prevents exit, stores code, closes admission, starts cleanup and an independent `std::thread` watchdog. Repeated requests while shutting down are prevented without spawning. Completed requests pass through.
- [ ] **Step 5: Run cleanup concurrently.** Start YouTube, sidecar, and Chrome cleanup futures together. At three seconds enter force cleanup; by four seconds the watchdog calls injected/real exit with the preserved code regardless of cleanup-task panic/stall.
- [ ] **Step 6: Run GREEN and commit.**

Run: `cargo test --manifest-path src-tauri/Cargo.toml external_process -- --nocapture`
Run: `npm.cmd run test -- src/lib/external-process-lifecycle-contract.test.ts`
Expected: PASS.

```powershell
git add src-tauri/src/external_process.rs src-tauri/src/lib.rs src/lib/external-process-lifecycle-contract.test.ts
git commit -m "feat: clean external processes on app exit"
```

### Task 7: Documentation and full verification

**Files:**
- Modify: `docs/project.md`
- Modify: `docs/browser-providers-llm-troubleshooting.md`
- Modify: `AGENTS.md`
- Inspect: `docs/value-registry.md`
- Create: `docs/superpowers/verification/2026-07-11-external-process-lifecycle.md`

**Interfaces:**
- Consumes: Tasks 1–6.
- Produces: current-state ownership rules and reproducible verification evidence.

- [ ] **Step 1: Update current-state docs** with admission, three/four-second budgets, direct bundled spawn, `externalBin` requirement, Job Object limitations, tainted sidecar behavior, mid-session stop semantics, and child-process rules for agents.
- [ ] **Step 2: Inspect the value registry.** Do not change it unless implementation introduced persisted/API values; runtime-only Rust enum variants do not qualify.
- [ ] **Step 3: Run full automated verification.**

```powershell
npm.cmd run test
npm.cmd run check
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
git diff --check
```

Expected: PASS; pre-existing warnings must be identified separately and no new lifecycle warning introduced.

- [ ] **Step 4: Build release GUI evidence.**

Run: `npm.cmd run tauri build -- --no-bundle --features csp-verification`
Expected: PASS, bundled sidecar remains beside the release executable.

- [ ] **Step 5: Verify yt-dlp lifecycle.** Start a long operation, cancel it, repeat and close Extractum, capture owned PIDs before action, and confirm direct process plus post-assignment test descendants terminate. Verify timeout errors remain unchanged.
- [ ] **Step 6: Verify sidecar lifecycle.** Launch the packaged binary directly through Extractum, close with idle sidecar, and confirm Stop ACK → stdin EOF → normal exit with no force warning. Repeat with a deliberately stalled/tainted request and confirm bounded force cleanup.
- [ ] **Step 7: Verify Chrome and app exit.** Start Extractum-owned CDP Chrome, close app, verify owned process cleanup and unrelated Chrome survival. Check empty-state exit, original nonzero exit-code preservation, normal ≈3-second target, and ≈4-second watchdog cap.
- [ ] **Step 8: Verify Windows crash containment.** Force-terminate Extractum while an assigned OS test tree is active; confirm descendants created after assignment terminate. Record pre-assignment limitation rather than claiming atomic containment.
- [ ] **Step 9: Record exact commands/results and commit.**

```powershell
git add docs/project.md docs/browser-providers-llm-troubleshooting.md AGENTS.md docs/superpowers/verification/2026-07-11-external-process-lifecycle.md
git commit -m "docs: verify external process lifecycle"
```
