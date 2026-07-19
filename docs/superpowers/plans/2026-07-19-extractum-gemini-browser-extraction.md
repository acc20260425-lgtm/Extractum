# Extractum Gemini Browser Crate Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract the portable Gemini Browser domain engine into `extractum-gemini-browser` while preserving all existing IPC, serde, run-log, queue, process-lifecycle, and external Rust consumer behavior.

**Architecture:** `extractum-gemini-browser` owns DTOs, validation, run logs, protocol codec, portable state, submission/status decisions, reconciliation, and delivered-job lifecycle behind an object-safe `BrowserExecutor` port. The `extractum` application remains the only owner of Tauri, application path resolution, SQLx/Apalis, concrete sidecar/CDP processes, transport taint, containment, kill/reap, and shutdown. The existing private `crate::gemini_browser` facade keeps all eight external Rust consumers on their current paths.

**Tech Stack:** Rust 2021, Cargo workspace, Tokio, tokio-util, parking_lot, serde/serde_json, time, url, Tauri 2, SQLx/Apalis in the application adapter, Vitest source-boundary contracts, PowerShell on Windows.

## Global Constraints

- Authority: [approved boundary specification](../specs/2026-07-19-gemini-browser-crate-boundary-design.md) and [crate roadmap](../specs/2026-07-17-crate-roadmap.md).
- Execute this as one Phase 4 slice on one implementation branch/worktree. Do not replay the canceled `extractum-process` plan or create a diagnostic worktree.
- Keep the checkout's canonical `src-tauri/target`. Run Cargo commands sequentially and do not set `CARGO_TARGET_DIR`.
- Do not change frontend behavior, IPC command names/signatures, serialized values, database schema/migrations, Apalis retry semantics, or the eight external Rust consumer paths.
- Do not introduce `extractum-process`, a generic process service, `async-trait`, Tauri/SQLx/Apalis/Tower/reqwest/windows-sys in the new crate, or any PID/child/pipe/process-tree type in its API.
- Preserve these outward strings byte-for-byte: `Gemini Browser job timed out waiting for worker result`, `Gemini Browser job timed out after {seconds}s`, and `Cancelled`.
- Preserve waiter-timeout behavior: it returns an app-visible error and leaves an existing queued run-log record non-terminal. Do not invent a waiter-timeout `GeminiBrowserRunResult`.
- A `GeminiBrowserError` is not serialized. The app maps it explicitly to the existing `AppError`; do not implement `From` because both final types are owned by dependency crates.
- Test-only constructors/helpers stay private to the new crate. App-owned integration tests use production `Default`/public operations, not widened test APIs.
- Resolve every pre-manifest core/dependency mismatch during seam preparation. Do not patch the new manifest ad hoc during the mechanical move.
- The frozen baseline is 94 unique tests with the exact 75/19 disposition in Appendix A. New characterization tests do not change that frozen count.
- Use exact import/type/API regexes for forbidden-source checks. Never ban the substring `apalis` globally because a frozen crate-owned test name contains it.
- Do not modify `scripts/process-shell-diagnostic/git-state.*`; those files are historical B/C/E fixtures, not current workspace allowlists.
- Timing is advisory: one discarded warm-up plus three recorded samples on the same inert marker toggle. No shell A/B, scanner, quiet-window coordinator, Job Object, retry, stability rule, or ledger.
- Timing failure produces `incomplete / no conclusion` only after exact source restoration. It never rejects, reverts, or retains the slice.
- Keep commits scoped. Inspect the dirty worktree before every commit and stage only files named by the current task.

## Final File Map

```text
src-tauri/crates/extractum-gemini-browser/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── error.rs
    ├── types.rs
    ├── run_id.rs
    ├── run_log.rs
    ├── sidecar_launch.rs
    ├── protocol.rs
    ├── cdp.rs
    ├── state.rs
    ├── runtime.rs
    ├── status.rs
    ├── submission.rs
    ├── reconciliation.rs
    └── execution.rs

src-tauri/src/gemini_browser/
├── mod.rs
├── commands.rs
├── jobs.rs
├── paths.rs
├── sidecar.rs
├── cdp_chrome.rs
├── state.rs
└── executor.rs
```

The app files contain only adapters and concrete integration. The current whole-domain `types.rs`, `run_log.rs`, and `sidecar_launch.rs` no longer exist under the app module after the move.

## Exact Cross-Crate Interfaces

The seam-preparation checkpoint implements these final shapes before the manifest is created. Names in this section are not suggestions.

```rust
pub type GeminiBrowserResult<T> = Result<T, GeminiBrowserError>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GeminiBrowserErrorKind {
    Validation,
    NotFound,
    Conflict,
    Persistence,
    Protocol,
    Transport,
    Browser,
    Timeout,
    Cancellation,
    Invariant,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeminiBrowserError {
    kind: GeminiBrowserErrorKind,
    message: String,
}

impl GeminiBrowserError {
    pub fn kind(&self) -> GeminiBrowserErrorKind;
    pub fn message(&self) -> &str;
    pub fn validation(message: impl Into<String>) -> Self;
    pub fn not_found(message: impl Into<String>) -> Self;
    pub fn conflict(message: impl Into<String>) -> Self;
    pub fn persistence(message: impl Into<String>) -> Self;
    pub fn protocol(message: impl Into<String>) -> Self;
    pub fn transport(message: impl Into<String>) -> Self;
    pub fn browser(message: impl Into<String>) -> Self;
    pub fn timeout(message: impl Into<String>) -> Self;
    pub fn cancellation(message: impl Into<String>) -> Self;
    pub fn invariant(message: impl Into<String>) -> Self;
}
```

Implement `Display` and `std::error::Error` manually; do not add `thiserror`.

```rust
pub type BrowserExecutorFuture<'a, T> =
    Pin<Box<dyn Future<Output = GeminiBrowserResult<T>> + Send + 'a>>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BrowserSessionContext {
    pub browser_profile_dir: String,
    pub browser_config: Option<GeminiBrowserProviderConfig>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BrowserRunContext {
    pub request: GeminiBrowserRunRequest,
    pub browser_profile_dir: String,
    pub artifact_dir: String,
    pub browser_config: Option<GeminiBrowserProviderConfig>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BrowserStopReason {
    Requested,
    Cancelled { run_id: String },
    TimedOut {
        run_id: String,
        timeout: Duration,
    },
}

pub trait BrowserExecutor: Send + Sync {
    fn status(
        &self,
        context: BrowserSessionContext,
    ) -> BrowserExecutorFuture<'_, GeminiBrowserProviderStatus>;
    fn open(
        &self,
        context: BrowserSessionContext,
    ) -> BrowserExecutorFuture<'_, GeminiBrowserProviderStatus>;
    fn resume(
        &self,
        context: BrowserSessionContext,
    ) -> BrowserExecutorFuture<'_, GeminiBrowserProviderStatus>;
    fn send(
        &self,
        context: BrowserRunContext,
    ) -> BrowserExecutorFuture<'_, GeminiBrowserRunResult>;
    fn stop(&self, reason: BrowserStopReason) -> BrowserExecutorFuture<'_, ()>;
}
```

`BrowserStopReason` is non-serialized. `AppBrowserExecutor<'a>` stores only `&'a AppHandle` and `&'a GeminiBrowserState` and performs all concrete spawn/request/taint/kill/reap work.

```rust
pub trait StatusObserver: Send + Sync {
    fn publish(&self, status: &GeminiBrowserProviderStatus);
}
```

The portable status entry points are also fixed:

```rust
pub async fn read_provider_status(
    state: &GeminiBrowserDomainState,
    executor: &dyn BrowserExecutor,
    session: BrowserSessionContext,
    queue_depth: usize,
    live_timeout: Duration,
) -> GeminiBrowserResult<GeminiBrowserProviderStatus>;

pub fn read_reconciled_status_snapshot(
    state: &GeminiBrowserDomainState,
    runs_dir: &Path,
    browser_profile_dir: String,
) -> GeminiBrowserResult<GeminiBrowserProviderStatus>;

pub async fn open_provider(
    executor: &dyn BrowserExecutor,
    observer: &dyn StatusObserver,
    session: BrowserSessionContext,
) -> GeminiBrowserResult<GeminiBrowserProviderStatus>;

pub async fn resume_provider(
    executor: &dyn BrowserExecutor,
    observer: &dyn StatusObserver,
    session: BrowserSessionContext,
) -> GeminiBrowserResult<GeminiBrowserProviderStatus>;
```

`read_provider_status` initializes a missing cached snapshot from `session.browser_profile_dir`, overlays the domain-owned active run ID and supplied queue depth on a real live executor status, but does not mutate an existing cached snapshot. A `live_timeout`, cancellation, invariant/unexpected-status error, or validation/state error returns the cached snapshot. Transport, protocol-decode/correlation, or sidecar-reported browser failure constructs the existing `NotStarted` status with the supplied profile/active/queue fields. The executor never constructs either fallback. The app runs startup reconciliation first. `read_reconciled_status_snapshot` initializes from its app-resolved profile string when needed, then performs the existing fresh/stale run-log reconciliation and CAS internally. `open_provider`/`resume_provider` preserve current behavior by publishing but not caching the returned live status. Submission, cancellation, and execution update the private cached snapshot before publishing exactly that status.

```rust

#[derive(Clone, Debug)]
pub struct DeliveredJobInput {
    pub job: GeminiBrowserJob,
    pub runs_dir: PathBuf,
    pub browser_profile_dir: String,
    pub artifact_dir: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeliveryOutcome {
    Completed { result: GeminiBrowserRunResult },
    AlreadyTerminal { result: Option<GeminiBrowserRunResult> },
    Cancelled {
        result: GeminiBrowserRunResult,
        stop_error: Option<GeminiBrowserError>,
    },
    TimedOut {
        result: GeminiBrowserRunResult,
        stop_error: Option<GeminiBrowserError>,
    },
    Failed { result: GeminiBrowserRunResult },
}

pub async fn execute_delivered_job(
    runtime: &GeminiBrowserJobRuntime,
    state: &GeminiBrowserDomainState,
    executor: &dyn BrowserExecutor,
    observer: &dyn StatusObserver,
    input: DeliveredJobInput,
) -> GeminiBrowserResult<DeliveryOutcome>;
```

`GeminiBrowserDomainState` owns a private active-run control containing the run ID, `CancellationToken`, and an `Arc<tokio::sync::OnceCell<Option<GeminiBrowserError>>>` for idempotent stop. Both external cancellation and selected cancellation/timeout branches call one private `stop_executor_once` operation. No method returns or accepts a `CancellationToken` across the crate edge.

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CancelRunOutcome {
    ActiveCancellationRequested {
        stop_error: Option<GeminiBrowserError>,
    },
    QueuedCancelled {
        result: GeminiBrowserRunResult,
    },
    AlreadyTerminal,
    Missing,
}

pub async fn cancel_run(
    runs_dir: &Path,
    runtime: &GeminiBrowserJobRuntime,
    state: &GeminiBrowserDomainState,
    executor: &dyn BrowserExecutor,
    observer: &dyn StatusObserver,
    run_id: &str,
) -> GeminiBrowserResult<CancelRunOutcome>;
```

For an active run, `cancel_run` cancels the private token, awaits `stop_executor_once`, and returns without fabricating a terminal log entry. `execute_delivered_job` ignores/drops simultaneous or later success, writes the exact `Cancelled` result, and returns `DeliveryOutcome::Cancelled`.

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueuedGeminiBrowserJob {
    pub run_id: String,
    pub queue_position: Option<usize>,
}

pub async fn submit_and_wait<Enqueue, EnqueueFuture>(
    runs_dir: &Path,
    runtime: &GeminiBrowserJobRuntime,
    state: &GeminiBrowserDomainState,
    observer: &dyn StatusObserver,
    request: GeminiBrowserRunRequest,
    browser_config: Option<GeminiBrowserProviderConfig>,
    enqueue: Enqueue,
) -> GeminiBrowserResult<GeminiBrowserRunResult>
where
    Enqueue: FnOnce(GeminiBrowserJob) -> EnqueueFuture + Send,
    EnqueueFuture:
        Future<Output = GeminiBrowserResult<QueuedGeminiBrowserJob>> + Send;
```

This preserves queued-log-before-enqueue ordering, duplicate rejection, waiter cleanup, enqueue-failure terminalization, queued status publication, and exact waiter-timeout text.

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NormalizedQueueState {
    Queued,
    Running,
    Succeeded,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueueInspectionSnapshot {
    Unavailable,
    Available(BTreeMap<String, NormalizedQueueState>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StartupReconciliationSnapshot {
    pub runs: Vec<GeminiBrowserRun>,
    pub queue: QueueInspectionSnapshot,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReconciliationAction {
    Finish {
        run_id: String,
        result: GeminiBrowserRunResult,
    },
}

pub fn reconcile_startup(
    snapshot: StartupReconciliationSnapshot,
) -> Vec<ReconciliationAction>;

pub async fn ensure_startup_reconciled<Load, LoadFuture, Apply, ApplyFuture>(
    state: &GeminiBrowserDomainState,
    load_snapshot: Load,
    apply_actions: Apply,
) -> GeminiBrowserResult<()>
where
    Load: FnOnce() -> LoadFuture + Send,
    LoadFuture: Future<Output = GeminiBrowserResult<StartupReconciliationSnapshot>> + Send,
    Apply: FnOnce(Vec<ReconciliationAction>) -> ApplyFuture + Send,
    ApplyFuture: Future<Output = GeminiBrowserResult<()>> + Send;

pub async fn run_registered_worker<Setup, SetupFuture, WorkerFuture, WorkerError>(
    runtime: &GeminiBrowserJobRuntime,
    setup: Setup,
) -> GeminiBrowserResult<()>
where
    Setup: FnOnce() -> SetupFuture + Send,
    SetupFuture: Future<Output = GeminiBrowserResult<WorkerFuture>> + Send,
    WorkerFuture: Future<Output = Result<(), WorkerError>> + Send,
    WorkerError: Display + Send;
```

The app converts exact Apalis strings to `NormalizedQueueState` in `load_snapshot` and still owns the Apalis worker, storage, timeout layer, delivery handler, and application of returned actions. `ensure_startup_reconciled` wraps load → `reconcile_startup` → apply in the domain state's private retry-on-error `OnceCell`; callers cannot access the admission cell.

`ChromeCdpLaunchSpec` does not contain an executable path:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChromeCdpLaunchSpec {
    pub args: Vec<String>,
    pub browser_profile_dir: String,
    pub cdp_endpoint: String,
}

pub fn build_chrome_cdp_launch_spec(
    browser_profile_dir: PathBuf,
    config: Option<&GeminiBrowserProviderConfig>,
) -> GeminiBrowserResult<ChromeCdpLaunchSpec>;
```

The app separately calls `find_chrome_executable()` and `spawn_chrome_cdp(&chrome_path, &spec)`. Endpoint normalization/port validation and `start_chrome_result` move; environment/executable discovery, reqwest readiness, `Child`, `Command`, and `ProcessTreeGuard` remain app-side.

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResumeSidecarOutcome {
    Status(GeminiBrowserProviderStatus),
    LegacyAck,
}

#[derive(Debug, Default)]
pub struct GeminiBrowserJsonlCodec {
    buffer: Vec<u8>,
}

impl GeminiBrowserJsonlCodec {
    pub fn new() -> Self;
    pub fn encode_request(
        &self,
        id: &str,
        command: &GeminiBrowserSidecarCommand,
    ) -> GeminiBrowserResult<Vec<u8>>;
    pub fn push_response_bytes(
        &mut self,
        expected_id: &str,
        chunk: &[u8],
    ) -> GeminiBrowserResult<Option<GeminiBrowserSidecarResponse>>;
}

pub fn classify_resume_response(
    response: GeminiBrowserSidecarResponse,
) -> GeminiBrowserResult<ResumeSidecarOutcome>;
```

Concrete stdin/stdout ownership and the async read/write loop remain in `extractum`; no handle or generic I/O trait crosses the API. The codec owns only buffered bytes, JSON encoding/decoding, correlation, stale-response skipping, partial/multiple-line handling, and resume classification. `push_response_bytes` consumes through the first matching complete frame, discards earlier stale complete frames, and retains every later complete or partial frame in `buffer` for the next call; it never drops read-ahead bytes.

## Rust Verification Loops

Affected packages are `extractum` before the move, `extractum-gemini-browser` after it, and immediate downstream `extractum` after every public-edge change.

Exact RED/GREEN seam tests before the move:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib gemini_browser::execution::tests::active_cancellation_stops_executor_once_and_ignores_late_success -- --exact
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib gemini_browser::execution::tests::execution_timeout_stops_executor_with_typed_timeout_reason -- --exact
```

Each command runs exactly one test. The initial RED is an unresolved port/type or failed stop/late-success assertion, never `0 tests`.

Preparation checkpoint:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
```

Post-move exact tests and package checkpoint:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --lib execution::tests::active_cancellation_stops_executor_once_and_ignores_late_success -- --exact
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --lib execution::tests::execution_timeout_stops_executor_with_typed_timeout_reason -- --exact
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --all-targets
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --all-targets
```

Immediate dependent checkpoint:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
```

App-owned process sentinels:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib gemini_browser::cdp_chrome::tests::explicit_shutdown_kills_and_reaps_the_owned_child_once -- --exact
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib gemini_browser::sidecar::tests::stderr_drain_consumes_sidecar_output_concurrently -- --exact
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib gemini_browser::executor::tests::cancelled_run_marks_the_sidecar_transport_tainted -- --exact
```

Linux package gate:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --all-targets --target x86_64-unknown-linux-gnu
```

End-of-slice completion gates:

```powershell
npm.cmd run check:rustfmt
cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets
cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets
npm.cmd run verify
```

Focused checks are accelerators only. The slice is incomplete until all four completion gates pass.

---

### Task 1: Freeze Baseline Behavior and Characterize Exact Legacy Output

**Files:**

- Modify: `src-tauri/src/gemini_browser/jobs.rs`
- Read: `src-tauri/src/gemini_browser/types.rs`
- Read: `src-tauri/src/gemini_browser/run_log.rs`
- Read: `src-tauri/src/gemini_browser/paths.rs`

**Interfaces:**

- Consumes: current `AppError`, result DTOs, run-log, cancellation, and timeout helpers.
- Produces: stronger assertions in four frozen tests; no production API.

- [ ] **Step 1: Confirm a clean baseline and frozen inventory.**

```powershell
git status --short
$listed = cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib -- --list
$gemini = @($listed | Select-String '^gemini_browser::.*: test$')
if ($gemini.Count -ne 94) { throw "Expected 94 Gemini baseline tests, found $($gemini.Count)" }
```

Expected: clean status and exactly 94 matching test lines. Stop on any correctness mismatch.

- [ ] **Step 2: Record the pre-seam core-use audit.**

```powershell
rg -n "extractum_core|crate::(?:error|time|media_metadata|compression)|AppError|AppResult|encode_media_metadata|decode_media_metadata" src-tauri/src/gemini_browser/types.rs src-tauri/src/gemini_browser/run_log.rs src-tauri/src/gemini_browser/paths.rs src-tauri/src/gemini_browser/sidecar_launch.rs src-tauri/src/gemini_browser/cdp_chrome.rs src-tauri/src/gemini_browser/commands.rs src-tauri/src/gemini_browser/jobs.rs src-tauri/src/gemini_browser/sidecar.rs src-tauri/src/gemini_browser/state.rs
```

Expected: no match in `types.rs`/`sidecar_launch.rs`; only facade `AppError/AppResult` in prepared fragments; no `extractum_core`, media metadata, compression, or core-time API.

- [ ] **Step 3: Add literal JSON fixtures and a value-only timestamp normalizer to the jobs test module.**

The helper rewrites only the contents of the two timestamp string values. It preserves the field names, indentation, colon spacing, commas, line endings, and every other byte; it never parses or reserializes the actual file.

```rust
fn replace_json_string_value(raw: &mut String, field: &str) {
    let key = format!("\"{field}\"");
    let key_start = raw.find(&key).unwrap_or_else(|| panic!("missing {field}"));
    let after_key = key_start + key.len();
    let open_quote = after_key
        + raw[after_key..]
            .find('"')
            .unwrap_or_else(|| panic!("missing opening quote for {field}"));
    let close_quote = open_quote
        + 1
        + raw[open_quote + 1..]
            .find('"')
            .unwrap_or_else(|| panic!("missing closing quote for {field}"));
    raw.replace_range(open_quote + 1..close_quote, "<timestamp>");
}

fn normalize_persisted_timestamps(raw: &str) -> String {
    let mut normalized = raw.to_string();
    replace_json_string_value(&mut normalized, "created_at");
    replace_json_string_value(&mut normalized, "updated_at");
    normalized
}

fn expected_non_terminal_run_json(run_id: &str, status: &str) -> String {
    format!(
        r#"{{
  "run_id": "{run_id}",
  "source": "settings_test",
  "status": "{status}",
  "prompt_preview": "hello",
  "created_at": "<timestamp>",
  "updated_at": "<timestamp>",
  "result": null
}}"#
    )
}

fn expected_terminal_run_json(
    run_id: &str,
    status: &str,
    message: &str,
    elapsed_ms: u64,
) -> String {
    format!(
        r#"{{
  "run_id": "{run_id}",
  "source": "settings_test",
  "status": "{status}",
  "prompt_preview": "hello",
  "created_at": "<timestamp>",
  "updated_at": "<timestamp>",
  "result": {{
    "run_id": "{run_id}",
    "status": "{status}",
    "text": null,
    "message": "{message}",
    "manual_action": null,
    "artifacts": {{
      "run_dir": null,
      "html": null,
      "screenshot": null,
      "telemetry": null,
      "answer_extraction": null,
      "artifact_write_error": null
    }},
    "elapsed_ms": {elapsed_ms},
    "debug_summary": null
  }}
}}"#
    )
}
```

Expected compact result strings are also handwritten literals, for example:

```rust
assert_eq!(
    serde_json::to_string(&cancelled).expect("serialize cancelled result"),
    "{\"run_id\":\"run-cancel-active\",\"status\":\"cancelled\",\"text\":null,\"message\":\"Cancelled\",\"manual_action\":null,\"artifacts\":{\"run_dir\":null,\"html\":null,\"screenshot\":null,\"telemetry\":null,\"answer_extraction\":null,\"artifact_write_error\":null},\"elapsed_ms\":0,\"debug_summary\":null}"
);
```

Never derive expected JSON with `serde_json`; otherwise an accidental field rename/reorder changes actual and expected together.

- [ ] **Step 4: Strengthen `wait_for_result_removes_waiter_on_timeout`.**

Create a queued run first, then assert:

```rust
assert_eq!(error.kind, crate::error::AppErrorKind::Internal);
assert_eq!(
    error.message,
    "Gemini Browser job timed out waiting for worker result"
);
assert_eq!(
    serde_json::to_string(&error).expect("serialize app error"),
    "{\"kind\":\"internal\",\"message\":\"Gemini Browser job timed out waiting for worker result\"}"
);
```

The persisted expected run stays `Queued` with `result: None`. Read `run-timeout/result.json` as text and compare `normalize_persisted_timestamps(&raw)` with `expected_non_terminal_run_json("run-timeout", "queued")`.

- [ ] **Step 5: Strengthen execution-timeout and cancellation tests.**

In `worker_timeout_marks_run_failed_and_processes_next_job` assert the full first result. The current test timeout is 25 ms, so exact values are `status: Failed`, message `Gemini Browser job timed out after 0s`, `elapsed_ms: 25`, all optional fields `None`, and default artifacts. Compare compact serde to a handwritten literal and normalized `result.json` to `expected_terminal_run_json("run-timeout-first", "failed", "Gemini Browser job timed out after 0s", 25)`.

In `cancel_gemini_browser_job_cancels_queued_run_and_waiter` and `cancel_gemini_browser_job_requests_stop_for_active_run` assert full `Cancelled` results with message `Cancelled`, `elapsed_ms: 0`, all optional fields `None`, default artifacts, exact handwritten compact serde, and normalized terminal JSON using `expected_terminal_run_json` with the literal run IDs and `"cancelled"`. Immediately after the active cancellation request, compare its normalized file with `expected_non_terminal_run_json("run-cancel-active", "running")`; it becomes terminal only after execution acknowledges cancellation.

- [ ] **Step 6: Run the four characterization tests.**

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib gemini_browser::jobs::tests::wait_for_result_removes_waiter_on_timeout -- --exact
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib gemini_browser::jobs::tests::worker_timeout_marks_run_failed_and_processes_next_job -- --exact
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib gemini_browser::jobs::tests::cancel_gemini_browser_job_cancels_queued_run_and_waiter -- --exact
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib gemini_browser::jobs::tests::cancel_gemini_browser_job_requests_stop_for_active_run -- --exact
```

Expected: each runs one test and passes. Characterization is intentionally GREEN; Task 4 has the honest typed RED.

- [ ] **Step 7: Commit characterization.**

```powershell
git status --short
git add src-tauri/src/gemini_browser/jobs.rs
git commit -m "test: characterize Gemini browser terminal payloads"
```

---

### Task 2: Introduce the Typed Error and Browser Executor Port In-App

**Files:**

- Create: `src-tauri/src/gemini_browser/domain_error.rs`
- Create: `src-tauri/src/gemini_browser/browser_executor.rs`
- Create: `src-tauri/src/gemini_browser/executor.rs`
- Modify: `src-tauri/src/gemini_browser/sidecar.rs`
- Modify: `src-tauri/src/gemini_browser/commands.rs`
- Modify: `src-tauri/src/gemini_browser/jobs.rs`
- Modify: `src-tauri/src/gemini_browser/state.rs`
- Modify: `src-tauri/src/gemini_browser/mod.rs`

**Interfaces:**

- Consumes: concrete app sidecar/CDP operations and current app state.
- Produces: exact error/port/context/stop types, `AppBrowserExecutor`, and explicit app error mapper.

- [ ] **Step 1: Add a RED app error-mapping test.**

Create the three new files, add `mod domain_error; mod browser_executor; mod executor;` to `mod.rs`, and put `gemini_browser_error_maps_to_exact_legacy_app_error_json` under `executor.rs` tests before running RED. The test constructs `GeminiBrowserError::timeout("Gemini Browser job timed out waiting for worker result")`, calls `domain_error_to_app`, and expects `AppErrorKind::Internal` plus exact JSON:

```json
{"kind":"internal","message":"Gemini Browser job timed out waiting for worker result"}
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib gemini_browser::executor::tests::gemini_browser_error_maps_to_exact_legacy_app_error_json -- --exact
```

Expected RED: the error/mapping is undefined, not `0 tests`.

- [ ] **Step 2: Implement the non-serialized error and mapper.**

Implement the exact interface above. Preserve messages and use these complete app-owned mappings; do not derive serde and do not implement `From`:

```rust
#[derive(Clone, Copy)]
pub(crate) enum DomainErrorContext {
    Persistence,
    Protocol,
    Transport,
    Browser,
    Invariant,
}

pub(crate) fn domain_error_to_app(error: GeminiBrowserError) -> AppError {
    match error.kind() {
        GeminiBrowserErrorKind::Validation => AppError::validation(error.message()),
        GeminiBrowserErrorKind::NotFound => AppError::not_found(error.message()),
        GeminiBrowserErrorKind::Conflict => AppError::conflict(error.message()),
        GeminiBrowserErrorKind::Persistence
        | GeminiBrowserErrorKind::Protocol
        | GeminiBrowserErrorKind::Transport
        | GeminiBrowserErrorKind::Browser
        | GeminiBrowserErrorKind::Timeout
        | GeminiBrowserErrorKind::Cancellation
        | GeminiBrowserErrorKind::Invariant => AppError::internal(error.message()),
    }
}

pub(crate) fn app_error_to_domain(
    error: AppError,
    context: DomainErrorContext,
) -> GeminiBrowserError {
    let message = error.message;
    match error.kind {
        AppErrorKind::Validation => GeminiBrowserError::validation(message),
        AppErrorKind::NotFound => GeminiBrowserError::not_found(message),
        AppErrorKind::Conflict => GeminiBrowserError::conflict(message),
        AppErrorKind::Auth | AppErrorKind::Network | AppErrorKind::Internal => match context {
            DomainErrorContext::Persistence => GeminiBrowserError::persistence(message),
            DomainErrorContext::Protocol => GeminiBrowserError::protocol(message),
            DomainErrorContext::Transport => GeminiBrowserError::transport(message),
            DomainErrorContext::Browser => GeminiBrowserError::browser(message),
            DomainErrorContext::Invariant => GeminiBrowserError::invariant(message),
        },
    }
}
```

Use `app_error_to_domain` only at the concrete source that still returns `AppError`: `Persistence` for enqueue/storage adapters, `Transport` for spawn/containment/stdin/stdout/EOF, `Browser` for concrete CDP/provider operation failure, and `Invariant` only for impossible adapter states. JSON encode/decode/correlation code returns `GeminiBrowserError::protocol` directly. Never wrap a whole high-level browser method in one fallback context.

- [ ] **Step 3: Implement every `BrowserExecutor` method in `AppBrowserExecutor`.**

`AppBrowserExecutor<'a>` contains `handle: &'a AppHandle` and `state: &'a GeminiBrowserState`. Refactor the internal `sidecar::status/open_browser/resume/send_single` functions used by the port to return `GeminiBrowserResult` and classify at source:

| Source | Domain kind |
| --- | --- |
| spawn, containment, missing pipe, write/flush/read, EOF | `Transport` |
| JSON encode/decode, ID correlation, legacy-resume protocol mismatch | `Protocol` |
| `GeminiBrowserSidecarResponse::Error { message }` | `Browser` |
| status call receives a well-formed non-status response | `Invariant` |
| CDP endpoint validation | `Validation` |
| CDP/provider operation failure | `Browser` |

Each executor method destructures its owned context and directly returns the matching typed `sidecar::*` future; it does not remap the whole result to `Transport`. In particular, `sidecar::status` returns only a real `Status` response or typed error. It does not create `NotStarted` or read cached status. Until Task 3, the current `provider_status` wrapper owns both fallbacks and overlays `active_run_id`/`queue_depth`; Task 3 moves that exact decision to `status.rs`.

Implement stop with the following branch and private helper:

```rust
fn stop(&self, reason: BrowserStopReason) -> BrowserExecutorFuture<'_, ()> {
    Box::pin(async move {
        let result = match reason {
            BrowserStopReason::Requested => stop_owned_browser_resources(
                self.handle,
                self.state,
            ).await,
            BrowserStopReason::Cancelled { .. } | BrowserStopReason::TimedOut { .. } => {
                discard_abandoned_transport(self.state, |_| {}).await
            }
        };
        result.map_err(|error| app_error_to_domain(error, DomainErrorContext::Transport))
    })
}

async fn discard_abandoned_transport(
    state: &GeminiBrowserState,
    before_discard: impl FnOnce(bool),
) -> AppResult<()> {
    state.mark_sidecar_tainted().await;
    before_discard(state.sidecar_tainted().await);
    let process = state.sidecar().await.take();
    drop(process);
    let cdp_result = stop_owned_cdp_chrome(state).await;
    state.clear_sidecar_taint().await;
    cdp_result
}

async fn stop_owned_cdp_chrome(state: &GeminiBrowserState) -> AppResult<()> {
    let Some(mut process) = state.cdp_chrome_process().await.take() else {
        return Ok(());
    };
    tokio::task::spawn_blocking(move || process.shutdown())
        .await
        .map_err(|_| AppError::internal("Chrome shutdown task did not complete"))?
        .map_err(|error| AppError::internal(format!("Failed to stop Chrome: {error}")))
}

async fn stop_owned_browser_resources(
    handle: &AppHandle,
    state: &GeminiBrowserState,
) -> AppResult<()> {
    let cdp_result = stop_owned_cdp_chrome(state).await;
    let sidecar_result = sidecar::stop(handle, state).await;
    match cdp_result {
        Err(error) => Err(error),
        Ok(()) => sidecar_result,
    }
}
```

Keep all three helpers app-private. The moved frozen test calls `discard_abandoned_transport` without an `AppHandle`, asserts the callback observes `true`, then asserts taint is clear and a repeated call succeeds. Taking both concrete `Option` values is the no-reuse proof; no fake process API crosses the boundary.

- [ ] **Step 4: Preserve cancellation until the domain replacement is live.**

Do **not** delete the existing `state.cancellation_token()` select yet. Task 2 commits a working intermediate state; removing it before `execute_delivered_job` owns the select can strand cancellation behind the sidecar mutex. Only adjust error mapping required by the port. Task 4 Step 3 removes this select in the same edit that routes delivery through the crate-owned select.

- [ ] **Step 5: Route current calls through the adapter.**

Replace direct status/open/resume/send/stop calls in commands/jobs with the port without changing command signatures. Add an app-private unit `AppStatusObserver`; its `publish` is intentionally a no-op because the crate updates its private snapshot first and the current app has no Gemini Browser Tauri status event. Do not invent a new event or serialized name. Move frozen `cancelled_run_marks_the_sidecar_transport_tainted` from `state.rs` to `executor.rs` without renaming it and exercise the private discard helper described above.

- [ ] **Step 6: Run GREEN and process sentinels.**

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib gemini_browser::executor::tests::gemini_browser_error_maps_to_exact_legacy_app_error_json -- --exact
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib gemini_browser::executor::tests::cancelled_run_marks_the_sidecar_transport_tainted -- --exact
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib gemini_browser::sidecar::tests::stderr_drain_consumes_sidecar_output_concurrently -- --exact
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
```

Expected: one test per exact run and clean app check.

- [ ] **Step 7: Commit.**

```powershell
git status --short
git add src-tauri/src/gemini_browser
git commit -m "refactor: introduce Gemini browser executor port"
```

---

### Task 3: Prepare Portable Modules Without Creating a Crate

**Files:**

- Create: `src-tauri/src/gemini_browser/run_id.rs`
- Create: `src-tauri/src/gemini_browser/protocol.rs`
- Create: `src-tauri/src/gemini_browser/cdp_contract.rs`
- Create: `src-tauri/src/gemini_browser/portable_state.rs`
- Create: `src-tauri/src/gemini_browser/status.rs`
- Create: `src-tauri/src/gemini_browser/submission.rs`
- Modify: `src-tauri/src/gemini_browser/types.rs`
- Modify: `src-tauri/src/gemini_browser/run_log.rs`
- Modify: `src-tauri/src/gemini_browser/sidecar_launch.rs`
- Modify: `src-tauri/src/gemini_browser/paths.rs`
- Modify: `src-tauri/src/gemini_browser/sidecar.rs`
- Modify: `src-tauri/src/gemini_browser/cdp_chrome.rs`
- Modify: `src-tauri/src/gemini_browser/state.rs`
- Modify: `src-tauri/src/gemini_browser/commands.rs`
- Modify: `src-tauri/src/gemini_browser/mod.rs`

**Interfaces:**

- Consumes: prepared error/port, app-provided path strings, current DTOs.
- Produces: crate-ready run-ID/run-log/protocol/CDP/state/status/submission modules with no application imports; app composite/wrappers remain.

- [ ] **Step 1: Move `safe_run_id` to `run_id.rs` and return `GeminiBrowserResult<String>`.**

Keep base/application directory resolution, directory creation, `run_dir`, and `path_string` app-side. App path functions explicitly map the domain error.

- [ ] **Step 2: Convert all run-log operations to domain errors.**

Keep the six existing operations and seven tests together. Map invalid ID to `Validation`, missing run to `NotFound`, filesystem/serde errors to `Persistence`, and impossible state to `Invariant`. Do not change text, pruning, JSON formatting, or layout.

- [ ] **Step 3: Split generic protocol from concrete transport.**

Move `SidecarLine`, `GeminiBrowserJsonlCodec`, correlation/decoding, complete-line buffering, `classify_resume_response`, `sidecar_unavailable_result`, and six protocol tests to `protocol.rs`. Rewrite frozen `jsonl_transport_round_trips_a_duplex_request` to call `encode_request`, split one response across two byte chunks, then pass a chunk containing a stale complete frame, the matching complete frame, and the prefix of the next frame through `push_response_bytes`; assert the match is returned and the prefix survives for the next call. Preserve the test name and correlation assertions. Keep the async read/write loop, `BufReader<ChildStdout>`, concrete stdin/stdout/stderr/spawn/guard/taint/shutdown, and `stderr_drain_consumes_sidecar_output_concurrently` app-side.

- [ ] **Step 4: Split pure CDP contract from Chrome ownership.**

Move endpoint normalization, port validation, launch arguments, `start_chrome_result`, and two pure tests to `cdp_contract.rs`. Remove executable path from `ChromeCdpLaunchSpec`. Keep discovery/environment/reqwest/child/command/guard/spawn/shutdown and six app tests in `cdp_chrome.rs`.

Keep current-executable lookup app-side too. Move `bundled_sidecar_path_from_current_exe` out of the prepared launch module and implement this private wrapper in `sidecar.rs`:

```rust
fn bundled_sidecar_path_from_current_exe() -> std::io::Result<PathBuf> {
    std::env::current_exe().map(|executable| bundled_sidecar_path(&executable))
}
```

- [ ] **Step 5: Split domain state from app composite.**

`GeminiBrowserDomainState` implements `Default` and owns active control, private token, cached status, reconciliation gate, snapshot update/CAS, and run-status mapping; none of those fields or token/snapshot mutators is public. App `GeminiBrowserState` contains a defaulted domain state plus taint and concrete sidecar/CDP mutexes, with only an app-private `domain(&self) -> &GeminiBrowserDomainState` accessor. `status.rs` calls `BrowserExecutor::status(BrowserSessionContext)` and implements the exact matrix from the public-interface section: live success is overlaid, timeout/invariant/state error uses cache, transport/protocol/browser error constructs `NotStarted`. The executor never receives domain bookkeeping or constructs fallback status. Move six portable state tests; keep only the taint test app-side.

- [ ] **Step 6: Move 17 command-core tests to `status.rs`, `submission.rs`, and run-log owners.**

`status.rs` owns live fallback, startup ordering, cached snapshot reconciliation/CAS, and freshness. `submission.rs` owns artifact validation, duplicate decisions, queued-log-before-enqueue, enqueue failure terminalization, waiters, and status publication. `commands.rs` retains Tauri wrappers, app paths/opener, CDP spawn/readiness, error mapping, and adapter construction. Preserve all frozen test names.

- [ ] **Step 7: Run focused tests and the app checkpoint.**

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib gemini_browser::protocol::tests::jsonl_transport_round_trips_a_duplex_request -- --exact
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib gemini_browser::cdp_contract::tests::launch_spec_rejects_remote_cdp_endpoint -- --exact
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib gemini_browser::portable_state::tests::state_tracks_active_run_and_cancellation -- --exact
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib gemini_browser::submission::tests::send_single_prompt_handoff_writes_run_log_before_enqueue -- --exact
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib gemini_browser::status::tests::provider_status_uses_cached_snapshot_when_sidecar_is_busy -- --exact
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
```

Expected: one test per exact run; full app checkpoint passes.

- [ ] **Step 8: Commit.**

```powershell
git status --short
git add src-tauri/src/gemini_browser
git commit -m "refactor: prepare Gemini browser portable modules"
```

---

### Task 4: Move Job Lifecycle and Reconciliation Behind the Port

**Files:**

- Create: `src-tauri/src/gemini_browser/runtime.rs`
- Create: `src-tauri/src/gemini_browser/reconciliation.rs`
- Create: `src-tauri/src/gemini_browser/execution.rs`
- Modify: `src-tauri/src/gemini_browser/jobs.rs`
- Modify: `src-tauri/src/gemini_browser/commands.rs`
- Modify: `src-tauri/src/gemini_browser/portable_state.rs`
- Modify: `src-tauri/src/gemini_browser/submission.rs`
- Modify: `src-tauri/src/gemini_browser/executor.rs`
- Modify: `src-tauri/src/gemini_browser/mod.rs`

**Interfaces:**

- Consumes: exact port, observer, prepared state/run-log/submission, app-normalized queue observations.
- Produces: runtime/job/outcome/cancellation/execution/reconciliation APIs and thin app Apalis adapter.

- [ ] **Step 1: Add honest RED tests.**

Create `runtime.rs`, `reconciliation.rs`, and `execution.rs`; register them as private modules in `mod.rs` before RED so exact commands cannot return `0 tests`. Add `active_cancellation_stops_executor_once_and_ignores_late_success` and `execution_timeout_stops_executor_with_typed_timeout_reason`.

The private `BlockingExecutor` contains `send_started`, `release_send`, `stop_started`, and `release_stop` `tokio::sync::Notify` values, `Mutex<Vec<BrowserStopReason>>`, one clonable send result, and one optional stop error. Its unused status/open/resume methods panic; `send` notifies `send_started`, awaits `release_send`, then clones its result; `stop` records the reason, notifies `stop_started`, awaits `release_stop`, then returns the configured error. A private observer records full status values in a mutex. Neither fake is exported.

Cancellation test sequence is exact: spawn delivery; await `send_started`; call `cancel_run` concurrently; await `stop_started`; assert one `Cancelled { run_id }`; release stop; then release a successful send; assert the outcome/log/waiter/status are cancelled, the exact result JSON is the Task 1 literal, success never overwrites it, and a second cancel does not add a stop. Timeout uses `#[tokio::test(start_paused = true)]`, advances by the configured execution timeout, asserts one `TimedOut { run_id, timeout }`, releases stop/send, and asserts the exact legacy timeout result plus preserved optional stop error. Run the two exact RED commands from `Rust Verification Loops`.

- [ ] **Step 2: Implement private stop-once coordination.**

Store `Arc<tokio::sync::OnceCell<Option<GeminiBrowserError>>>` with the active run. Both `cancel_run` and `execute_delivered_job` use this operation; clear active control only after terminal persistence and waiter completion:

```rust
async fn stop_executor_once(
    active: &ActiveRunControl,
    executor: &dyn BrowserExecutor,
    reason: BrowserStopReason,
) -> Option<GeminiBrowserError> {
    active
        .stop_result
        .get_or_init(|| async { executor.stop(reason).await.err() })
        .await
        .clone()
}
```

- [ ] **Step 3: Implement typed delivered-job selection.**

Atomically with routing the app worker through `execute_delivered_job`, remove the old `request_sidecar` cancellation select. Use this exact selection shape so dropping `send` happens before stop:

```rust
enum ExecutionSelection {
    Completed(GeminiBrowserResult<GeminiBrowserRunResult>),
    Cancelled,
    TimedOut,
}

let cancellation = active.cancellation.clone();
let timeout = runtime.execution_timeout();
let mut send = executor.send(run_context);
let selected = tokio::select! {
    biased;
    _ = cancellation.cancelled() => ExecutionSelection::Cancelled,
    _ = tokio::time::sleep(timeout) => ExecutionSelection::TimedOut,
    result = &mut send => ExecutionSelection::Completed(result),
};
drop(send);
```

For `Cancelled` and `TimedOut`, call `stop_executor_once` with the typed reason before writing the exact Task 1 result, completing the waiter, clearing active/cancelled state, publishing status, and returning the matching `DeliveryOutcome` with `stop_error`. A completed executor error becomes the existing failed result. Existing terminal or missing delivery returns `AlreadyTerminal` without invoking the executor.

- [ ] **Step 4: Delete string timeout classification.**

Remove `is_worker_timeout_result` and every `starts_with("Gemini Browser job timed out after ")` decision. App/Apalis mapping switches only on `DeliveryOutcome`.

- [ ] **Step 5: Implement snapshot-in/actions-out reconciliation.**

Move pure decisions to `reconciliation.rs`. `reconcile_startup` iterates non-terminal runs and returns actions in input order using this complete matrix:

| Run log | Queue unavailable | Queue missing | Queue queued | Queue running | Queue succeeded | Queue failed |
| --- | --- | --- | --- | --- | --- | --- |
| `Running` | interrupted failure | interrupted failure | interrupted failure | interrupted failure | legacy missing-result `Failed` | terminal `Failed` |
| `Queued` | no action | missing-from-Apalis failure | no action | running-without-sidecar failure | legacy missing-result `Failed` | terminal `Failed` |

Use the current literal messages `Gemini Browser worker was interrupted before completion`, `Gemini Browser queued job was missing from Apalis storage`, and `Gemini Browser queue state was running without an active sidecar`. `Succeeded` deliberately remains the existing `Failed` result with `Gemini Browser Apalis job completed before run log captured a result`; it must not fabricate success. Failed queue state retains `Gemini Browser Apalis job failed before completion` and all current `terminal_apalis_state_result` fields. App `jobs.rs` retains `ApalisQueueInspectionMode`, exact string normalization (`Pending`, `Running`, `Done`, `Killed`/`Failed`), SQL/table names, storage/setup/config/task builder, one-attempt policy, worker/layers, and thin handler. Its `ensure_gemini_browser_startup_reconciled` wrapper calls the exact public `ensure_startup_reconciled` interface: load returns `QueueInspectionSnapshot::Unavailable` in current degraded mode, and apply handles every returned `Finish` action. No admission cell is exposed.

The thin Apalis delivery handler uses this exact acknowledgement rule:

```rust
let outcome = execute_delivered_job(runtime, state, executor, observer, input)
    .await
    .map_err(|error| -> BoxDynError { Box::new(domain_error_to_app(error)) })?;
if let DeliveryOutcome::Cancelled { stop_error, .. }
| DeliveryOutcome::TimedOut { stop_error, .. } = &outcome
{
    if let Some(error) = stop_error {
        eprintln!("Gemini Browser stop diagnostic: {error}");
    }
}
match outcome {
    DeliveryOutcome::Completed { .. }
    | DeliveryOutcome::AlreadyTerminal { .. }
    | DeliveryOutcome::Cancelled { .. }
    | DeliveryOutcome::TimedOut { .. }
    | DeliveryOutcome::Failed { .. } => Ok(()),
}
```

Only a `GeminiBrowserError` returned before correct terminalization becomes `BoxDynError`. Every typed terminal outcome acknowledges the Apalis delivery; `stop_error` is diagnostic only and never changes retry/status semantics.

- [ ] **Step 6: Rewrite two timeout tests onto the portable fake executor.**

Preserve names `worker_timeout_marks_run_failed_and_processes_next_job` and `worker_timeout_clears_active_and_cancelled_state`, but remove their Apalis/SQLite harness. Execute two deliveries sequentially: first times out, second succeeds. Preserve exact event/stop/log/next-job/cleanup assertions.

- [ ] **Step 7: Keep 11 app job tests on production APIs.**

The frozen app tests stay in `jobs.rs`. Replace cross-crate uses of `new_for_test*` with `GeminiBrowserJobRuntime::default()` or public production operations. Do not widen test constructors.

- [ ] **Step 8: Run RED tests GREEN and legacy characterization.**

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib gemini_browser::execution::tests::active_cancellation_stops_executor_once_and_ignores_late_success -- --exact
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib gemini_browser::execution::tests::execution_timeout_stops_executor_with_typed_timeout_reason -- --exact
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib gemini_browser::runtime::tests::wait_for_result_removes_waiter_on_timeout -- --exact
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib gemini_browser::execution::tests::worker_timeout_marks_run_failed_and_processes_next_job -- --exact
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib gemini_browser::execution::tests::cancel_gemini_browser_job_cancels_queued_run_and_waiter -- --exact
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib gemini_browser::execution::tests::cancel_gemini_browser_job_requests_stop_for_active_run -- --exact
```

Expected: exactly one passed test each and unchanged payloads.

- [ ] **Step 9: Reprove the pre-manifest audit.**

```powershell
$prepared = @(
  'src-tauri/src/gemini_browser/domain_error.rs',
  'src-tauri/src/gemini_browser/browser_executor.rs',
  'src-tauri/src/gemini_browser/types.rs',
  'src-tauri/src/gemini_browser/run_id.rs',
  'src-tauri/src/gemini_browser/run_log.rs',
  'src-tauri/src/gemini_browser/sidecar_launch.rs',
  'src-tauri/src/gemini_browser/protocol.rs',
  'src-tauri/src/gemini_browser/cdp_contract.rs',
  'src-tauri/src/gemini_browser/portable_state.rs',
  'src-tauri/src/gemini_browser/runtime.rs',
  'src-tauri/src/gemini_browser/status.rs',
  'src-tauri/src/gemini_browser/submission.rs',
  'src-tauri/src/gemini_browser/reconciliation.rs',
  'src-tauri/src/gemini_browser/execution.rs'
)
$forbidden = rg -n "extractum_core|crate::error|crate::time|crate::media_metadata|crate::compression|AppError|AppResult|encode_media_metadata|decode_media_metadata|AsyncRead|AsyncWrite|ChildStdin|ChildStdout|ChildStderr" $prepared
if ($LASTEXITCODE -eq 0) { throw "Prepared crate files still use app/core APIs:`n$forbidden" }
if ($LASTEXITCODE -ne 1) { throw "Dependency audit command failed" }
rg -n "^use |tokio::|tokio_util::|parking_lot::|serde_json::|time::|url::|tempfile::" $prepared
```

Expected no forbidden match. Production roots are exactly `parking_lot`, `serde`, `serde_json`, `time`, `tokio`, `tokio-util`, `url`; dev roots are `tempfile` and Tokio test features. `extractum-core` is not required.

- [ ] **Step 10: Run preparation checkpoint and commit.**

```powershell
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
git status --short
git add src-tauri/src/gemini_browser
git commit -m "refactor: prepare Gemini browser job engine"
```

Expected: app package green; all 94 frozen tests still occur once in `extractum`; new tests pass.

---

### Task 5: Capture the Advisory Focused Baseline

**Files:**

- Temporarily modify and restore: `src-tauri/src/gemini_browser/types.rs`
- Record later in: `docs/superpowers/verification/2026-07-19-extractum-gemini-browser-extraction.md`

**Interfaces:**

- Consumes: clean preparation commit and canonical target.
- Produces: `{ hash, warmup_ms, samples_ms[3], median_ms, restored_hash, clean_status }` returned literally to the parent task; no repository artifact or commit.

- [ ] **Step 1: Confirm a clean preparation checkpoint.**

```powershell
git status --short
Get-FileHash src-tauri/src/gemini_browser/types.rs -Algorithm SHA256
```

Expected: clean status. Retain the printed hash for Task 9.

- [ ] **Step 2: Run four sequential checks with one toggled inert marker.**

Immediately before `#[cfg(test)] mod tests`, use `apply_patch` to alternate:

```rust
// cargo-measurement-probe: a
```

Sequence:

1. original → `a`, discarded warm-up;
2. `a` → `b`, recorded sample 1;
3. `b` → `a`, recorded sample 2;
4. `a` → `b`, recorded sample 3.

For each check:

```powershell
$elapsed = Measure-Command {
  cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
  if ($LASTEXITCODE -ne 0) { throw 'Focused baseline check failed' }
}
[int64]$elapsed.TotalMilliseconds
```

Use `apply_patch` for every toggle. Run no competing build.

- [ ] **Step 3: Execute unconditional cleanup and restoration proof.**

After successful checks 1–3, leave the marker in place and perform only the next declared `a ↔ b` toggle. After the fourth successful check, remove the `b` marker with inverse `apply_patch`. If any check fails, do not perform another toggle or check: immediately remove whichever `a` or `b` marker is present. In both paths, then run:

```powershell
Get-FileHash src-tauri/src/gemini_browser/types.rs -Algorithm SHA256
git status --short
```

Expected: original hash and clean status. On failure record `baseline incomplete / no conclusion` and do not retry.

- [ ] **Step 4: Compute the median without a ledger.**

Put the three recorded integers into transient `$recorded` and run:

```powershell
$ordered = @($recorded | Sort-Object)
$median = $ordered[1]
$median
```

Return `{ hash, warmup_ms, samples_ms[3], median_ms, restored_hash, clean_status }` with literal values to the parent implementing agent. Keep no script, ledger, or measurement file.

---

### Task 6: Add the Boundary Contract and Mechanically Create the Crate

**Files:**

- Create: `src/lib/gemini-browser-crate-boundary-contract.test.ts`
- Create: `src-tauri/crates/extractum-gemini-browser/Cargo.toml`
- Create: `src-tauri/crates/extractum-gemini-browser/src/lib.rs`
- Move/rename: the 14 prepared files listed in Step 4
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/Cargo.lock`
- Modify: `src-tauri/src/gemini_browser/mod.rs`
- Modify: `src-tauri/src/gemini_browser/commands.rs`
- Modify: `src-tauri/src/gemini_browser/jobs.rs`
- Modify: `src-tauri/src/gemini_browser/paths.rs`
- Modify: `src-tauri/src/gemini_browser/sidecar.rs`
- Modify: `src-tauri/src/gemini_browser/cdp_chrome.rs`
- Modify: `src-tauri/src/gemini_browser/state.rs`
- Modify: `src-tauri/src/gemini_browser/executor.rs`
- Modify: `src/lib/rust-workspace-core-contract.test.ts`

**Interfaces:**

- Consumes: prepared domain modules and frozen arrays in Appendix A.
- Produces: one new workspace member, exact app dependency/lock edge, curated crate root, explicit app facade, and GREEN boundary contract.

- [ ] **Step 1: Write the complete source-boundary contract with `node:fs`.**

Use `existsSync`, `readFileSync`, and `readdirSync` so a missing crate yields assertion RED rather than Vite import failure. Add exact test cases:

1. `declares one app-to-domain edge and locked package`;
2. `keeps the exact portable dependency and feature allowlist`;
3. `keeps a curated crate root and explicit private app facade`;
4. `moves every frozen baseline test exactly once`;
5. `keeps process Tauri SQL Apalis and worker infrastructure app-side`;
6. `keeps lifecycle transitions and cancellation ownership domain-side`;
7. `removes string timeout classification while preserving legacy output tests`.

Copy both arrays from Appendix A. Parse tests with:

```ts
const rustTestPattern =
  /#\[(?:tokio::)?test(?:\([^\]]*\))?\]\s*(?:#\[[^\]]+\]\s*)*(?:async\s+)?fn\s+([A-Za-z0-9_]+)/g;
```

Use these exact helpers rather than unbounded regexes across TOML sections:

```ts
const repoRoot = path.resolve(import.meta.dirname, "..", "..");
const read = (relativePath: string) => {
  const absolute = path.join(repoRoot, relativePath);
  return existsSync(absolute)
    ? readFileSync(absolute, "utf8").replace(/\r\n/g, "\n")
    : "";
};
const rustSources = (relativeDir: string): string[] => {
  const absolute = path.join(repoRoot, relativeDir);
  if (!existsSync(absolute)) return [];
  return readdirSync(absolute, { withFileTypes: true }).flatMap((entry) => {
    const child = path.join(relativeDir, entry.name).replaceAll("\\", "/");
    return entry.isDirectory()
      ? rustSources(child)
      : entry.isFile() && entry.name.endsWith(".rs")
        ? [read(child)]
        : [];
  });
};
const tomlSection = (source: string, heading: string) => {
  const marker = `[${heading}]`;
  const start = source.indexOf(marker);
  if (start < 0) return "";
  const bodyStart = start + marker.length;
  const next = source.slice(bodyStart).search(/^\[\[?[^\n]+\]?\]$/m);
  return source.slice(bodyStart, next < 0 ? undefined : bodyStart + next).trim();
};
const dependencyNames = (section: string) =>
  [...section.matchAll(/^([A-Za-z0-9_-]+)(?:\.workspace)?\s*=/gm)]
    .map((match) => match[1])
    .sort();
const lockPackage = (source: string, name: string) =>
  source
    .split(/(?=^\[\[package\]\]$)/m)
    .find((block) => block.includes(`\nname = "${name}"\n`)) ?? "";
const lockDependencies = (block: string) => {
  const match = block.match(/^dependencies = \[\n([\s\S]*?)^\]$/m);
  return match
    ? [...match[1].matchAll(/^ "([^"]+)",?$/gm)]
        .map((entry) => entry[1])
        .sort()
    : [];
};
const testNames = (sources: string[]) =>
  sources.flatMap((source) =>
    [...source.matchAll(new RegExp(rustTestPattern.source, "g"))].map(
      (match) => match[1],
    ),
  );
```

Import `path` from `node:path` along with the three `node:fs` functions. The dependency assertions are exact:

```ts
expect(tomlSection(rootCargo, "workspace")).toContain(
  'members = [".", "crates/extractum-core", "crates/extractum-gemini-browser"]',
);
expect(dependencyNames(tomlSection(rootCargo, "workspace.dependencies"))).toEqual([
  "parking_lot", "serde", "serde_json", "tempfile", "time", "tokio", "tokio-util", "url", "zstd",
].sort());
expect(tomlSection(rootCargo, "workspace.dependencies")).toBe([
  'parking_lot = "0.12"',
  'serde = { version = "1", features = ["derive"] }',
  'serde_json = "1"',
  'tempfile = "3"',
  'time = { version = "0.3", features = ["formatting", "parsing", "macros"] }',
  'tokio = "1"',
  'tokio-util = "0.7"',
  'url = "2"',
  'zstd = "0.13"',
].join("\n"));
expect(tomlSection(rootCargo, "dependencies")).toContain(
  'extractum-gemini-browser = { path = "crates/extractum-gemini-browser" }',
);
expect(
  tomlSection(rootCargo, "dependencies").match(/extractum-gemini-browser/g),
).toHaveLength(1);
for (const inherited of ["parking_lot", "tokio-util", "url", "tempfile"]) {
  expect(tomlSection(rootCargo, "dependencies")).toMatch(
    new RegExp(`^${inherited}\\s*=\\s*\\{ workspace = true`, "m"),
  );
}
expect(tomlSection(rootCargo, "dependencies")).toContain(
  'tokio = { workspace = true, features = ["full"] }',
);
expect(tomlSection(rootCargo, "dev-dependencies")).toContain(
  'tokio = { workspace = true, features = ["test-util"] }',
);
expect(dependencyNames(tomlSection(crateCargo, "dependencies"))).toEqual([
  "parking_lot", "serde", "serde_json", "time", "tokio", "tokio-util", "url",
].sort());
expect(dependencyNames(tomlSection(crateCargo, "dev-dependencies"))).toEqual([
  "tempfile", "tokio",
]);
expect(tomlSection(crateCargo, "dependencies")).toContain(
  'tokio = { workspace = true, features = ["macros", "sync", "time"] }',
);
expect(tomlSection(crateCargo, "dev-dependencies")).toContain(
  'tokio = { workspace = true, features = ["rt", "test-util"] }',
);
expect(lockDependencies(lockPackage(cargoLock, "extractum-gemini-browser"))).toEqual([
  "parking_lot", "serde", "serde_json", "tempfile", "time", "tokio", "tokio-util", "url",
].sort());
expect(lockDependencies(lockPackage(cargoLock, "extractum"))).toContain(
  "extractum-gemini-browser",
);
expect(crateCargo).not.toContain("extractum-core");
expect(crateCargo).not.toMatch(/^\[target\.|^\[profile\./m);
```

Compare the normalized crate root byte-for-byte with the Step 5 literal and the app facade byte-for-byte with the Step 6 literal. For test ownership, prove both frozen arrays are individually unique, their union is 94, every frozen name occurs once across `rustSources("src-tauri/crates/extractum-gemini-browser/src")` plus `rustSources("src-tauri/src/gemini_browser")`, and the two filtered/sorted locations equal the two frozen arrays; ignore only names outside the frozen union.

Use these exact source guards:

```ts
const crateSource = crateRust.join("\n");
const appSource = appRust.join("\n");
expect(crateSource).not.toMatch(
  /(?:tauri|sqlx|apalis(?:_sqlite)?|tower|reqwest|windows_sys)::|AppHandle|AppError|AppResult|ProcessTreeGuard|std::process|tokio::process|\bChild(?:Stdin|Stdout|Stderr)?\b|\bCommand\b|\bAsyncRead\b|\bAsyncWrite\b/,
);
expect(crateSource).not.toMatch(
  /extractum_process|external_process|child_process|process_tree|\bJobs\b|"Pending"|"Killed"/,
);
expect(appSource).not.toMatch(/CancellationToken|is_worker_timeout_result/);
expect(`${appSource}\n${crateSource}`).not.toMatch(
  /(?:starts_with|strip_prefix|contains|ends_with)\s*\(\s*"Gemini Browser job timed out after /,
);
expect(crateSource).toContain("CancellationToken");
expect(appSource).toMatch(/tauri::|AppHandle/);
expect(appSource).toMatch(/sqlx::/);
expect(appSource).toMatch(/apalis/);
expect(appSource).toMatch(/ProcessTreeGuard/);
for (const helper of [
  "new_for_test",
  "new_for_test_with_timeouts",
  "new_for_waiter_timeout_test",
  "has_waiter_for_test",
  "worker_status_for_test",
]) {
  expect(crateSource).not.toMatch(
    new RegExp(`\\bpub\\s+(?:async\\s+)?fn\\s+${helper}\\b`),
  );
}
expect(crateSource).not.toMatch(
  /\bpub\s+(?:struct|enum|fn)\s+(?:BlockingExecutor|RecordingStatusObserver|FakeExecutor|FakeObserver)\b/,
);
for (const exactText of [
  "Gemini Browser job timed out waiting for worker result",
  "Gemini Browser job timed out after ",
  "Cancelled",
  "wait_for_result_removes_waiter_on_timeout",
  "worker_timeout_marks_run_failed_and_processes_next_job",
  "cancel_gemini_browser_job_cancels_queued_run_and_waiter",
  "cancel_gemini_browser_job_requests_stop_for_active_run",
]) {
  expect(crateSource).toContain(exactText);
}
expect(testNames(appRust)).toContain(
  "gemini_browser_error_maps_to_exact_legacy_app_error_json",
);
expect(appSource).toContain(
  "Gemini Browser job timed out waiting for worker result",
);
expect(appSource).toMatch(
  /fn gemini_browser_error_maps_to_exact_legacy_app_error_json[\s\S]{0,2500}serde_json::to_string/,
);
```

The `apalis` guard is qualified/import-based; the frozen test name containing `apalis` remains allowed.

- [ ] **Step 2: Record the expected RED before moving files.**

```powershell
npm.cmd run test -- src/lib/gemini-browser-crate-boundary-contract.test.ts
```

Expected RED: member/manifest/lock/curated root absent and all 94 names still app-owned. Preserve the result for verification.

- [ ] **Step 3: Add exact workspace dependencies and manifests.**

Root members:

```toml
members = [".", "crates/extractum-core", "crates/extractum-gemini-browser"]
```

Root shared dependencies:

```toml
[workspace.dependencies]
parking_lot = "0.12"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tempfile = "3"
time = { version = "0.3", features = ["formatting", "parsing", "macros"] }
tokio = "1"
tokio-util = "0.7"
url = "2"
zstd = "0.13"
```

Convert app `parking_lot`, `tokio`, `tokio-util`, `url`, and `tempfile` to workspace inheritance while preserving app features, including `tokio = { workspace = true, features = ["full"] }` and dev `test-util`. Add:

```toml
extractum-gemini-browser = { path = "crates/extractum-gemini-browser" }
```

New manifest:

```toml
[package]
name = "extractum-gemini-browser"
version.workspace = true
edition.workspace = true
publish = false

[dependencies]
parking_lot.workspace = true
serde.workspace = true
serde_json.workspace = true
time.workspace = true
tokio = { workspace = true, features = ["macros", "sync", "time"] }
tokio-util.workspace = true
url.workspace = true

[dev-dependencies]
tempfile.workspace = true
tokio = { workspace = true, features = ["rt", "test-util"] }
```

`test-util` is required only by the paused-time timeout test from Task 4; the new crate does not enable `rt-multi-thread`.

No `extractum-core`, target dependency, or profile section.

- [ ] **Step 4: Move the prepared files mechanically with `git mv`.**

| From `src-tauri/src/gemini_browser` | To `src-tauri/crates/extractum-gemini-browser/src` |
| --- | --- |
| `domain_error.rs` | `error.rs` |
| `browser_executor.rs` | `executor.rs` |
| `types.rs` | `types.rs` |
| `run_id.rs` | `run_id.rs` |
| `run_log.rs` | `run_log.rs` |
| `sidecar_launch.rs` | `sidecar_launch.rs` |
| `protocol.rs` | `protocol.rs` |
| `cdp_contract.rs` | `cdp.rs` |
| `portable_state.rs` | `state.rs` |
| `runtime.rs` | `runtime.rs` |
| `status.rs` | `status.rs` |
| `submission.rs` | `submission.rs` |
| `reconciliation.rs` | `reconciliation.rs` |
| `execution.rs` | `execution.rs` |

Change only import paths required by the new crate root. No behavior refactor in this step.

- [ ] **Step 5: Create a curated crate root.**

Use this exact `lib.rs`; it is the complete visibility inventory and contains no `pub use ...::*` or `pub mod`:

```rust
mod cdp;
mod error;
mod execution;
mod executor;
mod protocol;
mod reconciliation;
mod run_id;
mod run_log;
mod runtime;
mod sidecar_launch;
mod state;
mod status;
mod submission;
mod types;

pub use cdp::{build_chrome_cdp_launch_spec, start_chrome_result, ChromeCdpLaunchSpec};
pub use error::{GeminiBrowserError, GeminiBrowserErrorKind, GeminiBrowserResult};
pub use execution::{
    cancel_run, execute_delivered_job, CancelRunOutcome, DeliveredJobInput, DeliveryOutcome,
};
pub use executor::{
    BrowserExecutor, BrowserExecutorFuture, BrowserRunContext, BrowserSessionContext,
    BrowserStopReason,
};
pub use protocol::{
    classify_resume_response, GeminiBrowserJsonlCodec, ResumeSidecarOutcome,
};
pub use reconciliation::{
    ensure_startup_reconciled, reconcile_startup, NormalizedQueueState, QueueInspectionSnapshot,
    ReconciliationAction, StartupReconciliationSnapshot,
};
pub use run_id::safe_run_id;
pub use run_log::{
    create_queued_run, finish_run, list_runs, mark_running, read_run, recorded_run_dir,
};
pub use runtime::{
    run_registered_worker, GeminiBrowserArtifactMode, GeminiBrowserJob, GeminiBrowserJobRuntime,
};
pub use sidecar_launch::{
    bundled_sidecar_path, dev_sidecar_script, resolve_launch_mode, GeminiBrowserBuildProfile,
    GeminiBrowserSidecarLaunch, GEMINI_BROWSER_SIDECAR_NAME,
};
pub use state::GeminiBrowserDomainState;
pub use status::{
    open_provider, read_provider_status, read_reconciled_status_snapshot, resume_provider,
    StatusObserver,
};
pub use submission::{submit_and_wait, QueuedGeminiBrowserJob};
pub use types::{
    GeminiBrowserAnswerCompletionReason, GeminiBrowserAnswerExtractionDebug,
    GeminiBrowserAnswerGrouping, GeminiBrowserArtifactRefs, GeminiBrowserCandidateRejectReason,
    GeminiBrowserDebugErrorStage, GeminiBrowserManualAction, GeminiBrowserProviderConfig,
    GeminiBrowserProviderMode, GeminiBrowserProviderStatus, GeminiBrowserProviderStatusKind,
    GeminiBrowserRun, GeminiBrowserRunDebugSummary, GeminiBrowserRunLogSummary,
    GeminiBrowserRunRequest, GeminiBrowserRunResult, GeminiBrowserRunStatus,
    GeminiBrowserSidecarCommand, GeminiBrowserSidecarEnvelope, GeminiBrowserSidecarResponse,
    GeminiBrowserStartChromeResult,
};
```

If implementation requires any additional public name, stop for design review rather than widening this list.

- [ ] **Step 6: Rebuild the private app facade without consumer rewrites.**

Use this exact module/facade shape; local adapter modules may import additional curated names directly from `extractum_gemini_browser`, but the eight existing consumers do not:

```rust
mod cdp_chrome;
mod commands;
mod executor;
mod jobs;
mod paths;
mod sidecar;
mod state;

pub(crate) use cdp_chrome::shutdown_cdp_chrome;
pub use commands::{
    gemini_bridge_get_run, gemini_bridge_list_runs, gemini_bridge_open_browser,
    gemini_bridge_open_run_folder, gemini_bridge_resume, gemini_bridge_send_single,
    gemini_bridge_start_cdp_chrome, gemini_bridge_status, gemini_bridge_status_snapshot,
    gemini_bridge_stop,
};
pub(crate) use commands::{provider_status, send_single_prompt};
pub(crate) use jobs::{cancel_gemini_browser_job, start_gemini_browser_job_worker};
#[cfg(test)]
pub(crate) use jobs::{
    enqueue_gemini_browser_job_to_storage, open_gemini_browser_job_storage,
    setup_gemini_browser_apalis_storage,
};
pub(crate) use paths::{chrome_cdp_profile_dir, path_string, profile_dir, run_dir, runs_dir};
pub(crate) use sidecar::shutdown_sidecar;
pub use state::GeminiBrowserState;

pub(crate) use extractum_gemini_browser::{
    create_queued_run, finish_run, list_runs, mark_running, read_run, recorded_run_dir,
    GeminiBrowserJobRuntime,
};
#[cfg(test)]
pub(crate) use extractum_gemini_browser::{GeminiBrowserArtifactMode, GeminiBrowserJob};
pub use extractum_gemini_browser::{
    GeminiBrowserAnswerCompletionReason, GeminiBrowserArtifactRefs, GeminiBrowserProviderConfig,
    GeminiBrowserProviderMode, GeminiBrowserProviderStatus, GeminiBrowserProviderStatusKind,
    GeminiBrowserRun, GeminiBrowserRunLogSummary, GeminiBrowserRunRequest, GeminiBrowserRunResult,
    GeminiBrowserRunStatus, GeminiBrowserSidecarCommand, GeminiBrowserSidecarEnvelope,
    GeminiBrowserSidecarResponse, GeminiBrowserStartChromeResult,
};
#[cfg(test)]
pub(crate) use extractum_gemini_browser::{
    GeminiBrowserDebugErrorStage, GeminiBrowserRunDebugSummary,
};
```

Verify the eight external consumers contain no direct crate import:

```powershell
rg -n "extractum_gemini_browser" src-tauri/src/lib.rs src-tauri/src/apalis_jobs.rs src-tauri/src/prompt_packs/gemini_browser_stage.rs src-tauri/src/prompt_packs/dto.rs src-tauri/src/prompt_packs/completion_transport.rs src-tauri/src/prompt_packs/runtime_config.rs src-tauri/src/prompt_packs/runtime.rs src-tauri/src/prompt_packs/youtube_summary/snapshots_tests.rs
```

Expected: no match; existing `crate::gemini_browser` paths compile unchanged.

- [ ] **Step 7: Generate and validate the lock update before `--locked` gates.**

```powershell
cargo metadata --manifest-path src-tauri/Cargo.toml --format-version 1 | Out-Null
if ($LASTEXITCODE -ne 0) { throw 'Cargo metadata/lock generation failed' }
cargo metadata --manifest-path src-tauri/Cargo.toml --locked --format-version 1 | Out-Null
if ($LASTEXITCODE -ne 0) { throw 'Locked Cargo metadata validation failed' }
git diff -- src-tauri/Cargo.lock
```

Expected:

- one new package block;
- its exact dependencies are `parking_lot`, `serde`, `serde_json`, `tempfile`, `time`, `tokio`, `tokio-util`, `url`;
- app block gains one new crate edge;
- new block has no `extractum-core`.

- [ ] **Step 8: Update only the current workspace allowlist.**

Change `src/lib/rust-workspace-core-contract.test.ts` to parse and compare:

```ts
[".", "crates/extractum-core", "crates/extractum-gemini-browser"]
```

Do not edit historical diagnostic fixtures.

- [ ] **Step 9: Make source contracts GREEN.**

```powershell
npm.cmd run test -- src/lib/gemini-browser-crate-boundary-contract.test.ts src/lib/rust-workspace-core-contract.test.ts
```

Expected:

- exact import/type/API regexes, not `apalis` substring ban;
- no `CancellationToken` in app Gemini sources; private token in crate;
- no concrete process/AppHandle/AppError/SQL/Apalis types in crate;
- no `is_worker_timeout_result` or message-prefix decisions;
- TOML sections stop at the next heading;
- dependency/features/lock exact;
- 94 names occur once with 75/19 ownership.

- [ ] **Step 10: Run package/downstream checkpoints.**

```powershell
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --all-targets
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --all-targets
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
```

Expected: new package owns 75 frozen domain tests plus characterization; app owns 19 frozen app tests plus mapping tests.

- [ ] **Step 11: Commit the extraction.**

```powershell
git status --short
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/crates/extractum-gemini-browser src-tauri/src/gemini_browser src/lib/gemini-browser-crate-boundary-contract.test.ts src/lib/rust-workspace-core-contract.test.ts
git commit -m "refactor: extract Gemini browser domain crate"
```

---

### Task 7: Capture Candidate Timing and Verify Both Packages

**Files:**

- Temporarily modify and restore: `src-tauri/crates/extractum-gemini-browser/src/types.rs`
- Record later in: `docs/superpowers/verification/2026-07-19-extractum-gemini-browser-extraction.md`

**Interfaces:**

- Consumes: clean extraction commit.
- Produces: candidate `{ hash, warmup_ms, samples_ms[3], median_ms, restored_hash, clean_status }`, advisory delta, and package/app/Linux results returned literally to the parent task.

- [ ] **Step 1: Run exact post-move characterization.**

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --lib execution::tests::active_cancellation_stops_executor_once_and_ignores_late_success -- --exact
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --lib execution::tests::execution_timeout_stops_executor_with_typed_timeout_reason -- --exact
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --lib runtime::tests::wait_for_result_removes_waiter_on_timeout -- --exact
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib gemini_browser::executor::tests::gemini_browser_error_maps_to_exact_legacy_app_error_json -- --exact
```

Expected: one passed test each.

- [ ] **Step 2: Capture candidate timing with a self-contained toggle sequence.**

First prove the extraction commit is clean and retain the original hash:

```powershell
git status --short
Get-FileHash src-tauri/crates/extractum-gemini-browser/src/types.rs -Algorithm SHA256
```

Immediately before `#[cfg(test)] mod tests`, use `apply_patch` four times to perform the exact sequence:

1. original → `// cargo-measurement-probe: a`, discarded warm-up;
2. `a` → `// cargo-measurement-probe: b`, recorded sample 1;
3. `b` → `// cargo-measurement-probe: a`, recorded sample 2;
4. `a` → `// cargo-measurement-probe: b`, recorded sample 3.

After each patch run exactly:

```powershell
$elapsed = Measure-Command {
  cargo check --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --all-targets
  if ($LASTEXITCODE -ne 0) { throw 'Focused candidate check failed' }
}
[int64]$elapsed.TotalMilliseconds
```

After successful checks 1–3, leave the marker in place and perform only the next declared `a ↔ b` toggle. After the fourth successful check, remove the `b` marker with inverse `apply_patch`. If any check fails, do not perform another toggle or check: inspect the current marker with `rg -n "cargo-measurement-probe"` and immediately apply the exact inverse hunk removing whichever single `a` or `b` line is present. In both paths, then run:

```powershell
Get-FileHash src-tauri/crates/extractum-gemini-browser/src/types.rs -Algorithm SHA256
git status --short
```

The restored hash must equal the original and status must be clean. Do not retry. Return `{ hash, warmup_ms, samples_ms[3], median_ms, restored_hash, clean_status }` literally to the parent; create no file.

- [ ] **Step 3: Compute advisory result.**

Compute candidate median, absolute delta (`candidate - baseline`), and percentage delta (`delta / baseline × 100`). If either series is incomplete, record `incomplete / no conclusion`. Apply no threshold or veto.

- [ ] **Step 4: Run package and immediate-dependent checkpoints.**

```powershell
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --all-targets
cargo test --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --all-targets
cargo check --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --all-targets
```

- [ ] **Step 5: Run exact app process sentinels.**

```powershell
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib gemini_browser::cdp_chrome::tests::explicit_shutdown_kills_and_reaps_the_owned_child_once -- --exact
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib gemini_browser::sidecar::tests::stderr_drain_consumes_sidecar_output_concurrently -- --exact
cargo test --manifest-path src-tauri/Cargo.toml -p extractum --lib gemini_browser::executor::tests::cancelled_run_marks_the_sidecar_transport_tainted -- --exact
```

- [ ] **Step 6: Ensure the Linux target and run portability.**

```powershell
$installed = @(rustup target list --installed)
if ($installed -notcontains 'x86_64-unknown-linux-gnu') {
  rustup target add x86_64-unknown-linux-gnu
  if ($LASTEXITCODE -ne 0) { throw 'Unable to install Linux Rust target' }
}
cargo check --manifest-path src-tauri/Cargo.toml -p extractum-gemini-browser --all-targets --target x86_64-unknown-linux-gnu
if ($LASTEXITCODE -ne 0) { throw 'Gemini Browser Linux package check failed' }
```

Missing-target installation may need user approval/network. Install failure is infrastructure and does not waive the gate; post-install compile failure is candidate failure.

---

### Task 8: Run Fixed Sidecar, CDP, Build, and Shutdown Smokes

**Files:**

- No source changes.
- Record later in: `docs/superpowers/verification/2026-07-19-extractum-gemini-browser-extraction.md`

**Interfaces:**

- Consumes: extracted integration and existing scripts.
- Produces: fixed sidecar/CDP/no-bundle/visible lifecycle evidence.

- [ ] **Step 1: Run fixed sidecar sequence in order.**

```powershell
npm.cmd run smoke:gemini-browser-sidecar:node
if ($LASTEXITCODE -ne 0) { throw 'Node sidecar smoke failed' }
npm.cmd run build:gemini-browser-sidecar
if ($LASTEXITCODE -ne 0) { throw 'Sidecar build failed' }
npm.cmd run check:gemini-browser-sidecar-binary
if ($LASTEXITCODE -ne 0) { throw 'Sidecar binary check failed' }
npm.cmd run smoke:gemini-browser-sidecar:binary
if ($LASTEXITCODE -ne 0) { throw 'Binary sidecar smoke failed' }
```

Expected: both smokes exit 0 with `id: "smoke-1"` and `response.type: "status"`; build/check find `src-tauri/binaries/gemini-browser-sidecar-x86_64-pc-windows-msvc.exe`.

- [ ] **Step 2: Run mandatory CDP negative path in `try/finally`.**

```powershell
$previousCdpEndpoint = [Environment]::GetEnvironmentVariable(
  'EXTRACTUM_GEMINI_BROWSER_CDP_ENDPOINT',
  'Process'
)
try {
  $env:EXTRACTUM_GEMINI_BROWSER_CDP_ENDPOINT = 'http://127.0.0.1:65530'
  npm.cmd run smoke:gemini-browser-sidecar:resume:node -- --expect-manual-action=start_chrome_cdp
  if ($LASTEXITCODE -ne 0) { throw 'CDP negative smoke failed' }
} finally {
  if ($null -eq $previousCdpEndpoint) {
    Remove-Item Env:\EXTRACTUM_GEMINI_BROWSER_CDP_ENDPOINT -ErrorAction SilentlyContinue
  } else {
    $env:EXTRACTUM_GEMINI_BROWSER_CDP_ENDPOINT = $previousCdpEndpoint
  }
}
```

Expected `needs_manual_action` / `start_chrome_cdp`.

- [ ] **Step 3: Build release without MSI/WiX.**

```powershell
npm.cmd run tauri -- build --no-bundle
if ($LASTEXITCODE -ne 0) { throw 'No-bundle build failed' }
if (-not (Test-Path 'src-tauri/target/release/extractum.exe')) {
  throw 'Missing release application'
}
if (-not (Test-Path 'src-tauri/target/release/gemini-browser-sidecar.exe')) {
  throw 'Missing packaged Gemini Browser sidecar'
}
```

Expected: exit 0 and both files exist. MSI/WiX stays excluded.

- [ ] **Step 4: Run visible normal startup/status/shutdown.**

Run the preflight and visible start:

```powershell
$existing = @(
  Get-Process -Name extractum,gemini-browser-sidecar -ErrorAction SilentlyContinue
)
if ($existing.Count -ne 0) {
  throw "Unrelated app/sidecar process exists: $($existing.Id -join ', ')"
}
$appPath = (Resolve-Path 'src-tauri/target/release/extractum.exe').Path
$app = Start-Process -FilePath $appPath -PassThru
$appPid = $app.Id
Start-Sleep -Seconds 3
if ($app.HasExited) { throw "Extractum exited early with code $($app.ExitCode)" }
```

In the visible app open **Settings → Browser Providers**, choose **Managed**, and click **Refresh Gemini Browser status**. Confirm the status operation completes, then capture exactly one owned sidecar:

```powershell
$sidecars = @(Get-Process -Name gemini-browser-sidecar -ErrorAction SilentlyContinue)
if ($sidecars.Count -ne 1) {
  throw "Expected exactly one owned sidecar, found $($sidecars.Count)"
}
$sidecarPid = $sidecars[0].Id
```

Close the main window normally through the UI; do not force-stop either process. Then prove shutdown:

```powershell
if (-not $app.WaitForExit(10000)) { throw 'Extractum did not exit after normal close' }
Start-Sleep -Seconds 2
if (Get-Process -Id $appPid -ErrorAction SilentlyContinue) {
  throw "Extractum PID $appPid survived normal close"
}
if (Get-Process -Id $sidecarPid -ErrorAction SilentlyContinue) {
  throw "Owned sidecar PID $sidecarPid survived normal app close"
}
```

Do not navigate to Gemini, start CDP Chrome, or change Google account state.

Classification:

- start/process-observation/helper failure is infrastructure until helper soundness is proven;
- confirmed early app exit, status behavior failure, or surviving owned sidecar is completion failure.

---

### Task 9: Run Completion Gates, Record Evidence, and Retain Phase 4

**Files:**

- Create: `docs/superpowers/verification/2026-07-19-extractum-gemini-browser-extraction.md`
- Modify: `docs/superpowers/specs/2026-07-19-gemini-browser-crate-boundary-design.md`
- Modify: `docs/superpowers/specs/2026-07-17-crate-roadmap.md`
- Modify: `src/lib/crate-extraction-shell-cap-contract.test.ts`
- Verify: `src/lib/gemini-browser-crate-boundary-contract.test.ts`

**Interfaces:**

- Consumes: actual commits, inventories, timing/hashes, package/Linux/smoke/build results.
- Produces: literal verification and retained status only when every mandatory gate passes.

- [ ] **Step 1: Run and record the mandatory workspace check while status is pending.**

```powershell
$workspaceCheckOutput = @()
cargo check --manifest-path src-tauri/Cargo.toml --workspace --all-targets 2>&1 | Tee-Object -Variable workspaceCheckOutput
if ($LASTEXITCODE -ne 0) { throw 'Mandatory workspace check failed' }
$finishedLines = @(
  $workspaceCheckOutput | ForEach-Object { $_.ToString() } | Select-String '^\s*Finished .+ in ([0-9.]+)s$'
)
if ($finishedLines.Count -ne 1) {
  throw "Expected one Cargo Finished duration, found $($finishedLines.Count)"
}
$workspaceCheckCargoLine = $finishedLines[0].Line.Trim()
if ($workspaceCheckCargoLine -notmatch ' in ([0-9.]+)s$') {
  throw 'Unable to parse Cargo Finished duration'
}
$workspaceCheckMs = [int64][Math]::Round(
  [double]::Parse($Matches[1], [Globalization.CultureInfo]::InvariantCulture) * 1000
)
$workspaceCheckCargoLine
$workspaceCheckMs
```

This is the direct mandatory Cargo run, not a wall-clock wrapper. Record its exact emitted `Finished ... in Xs` line; `$workspaceCheckMs` is only the mechanical conversion used by the roadmap's 15,000 ms rule. It becomes an ordinary roadmap signal only if every later gate passes and the slice is retained. Do not rerun it for the signal. A check nested inside `verify` does not count. Phase 4 alone cannot trigger the adjacent-two-completed-slices ≥15,000 ms rule.

- [ ] **Step 2: Run the other three completion gates while status is pending.**

```powershell
npm.cmd run check:rustfmt
if ($LASTEXITCODE -ne 0) { throw 'Rustfmt gate failed' }
cargo test --manifest-path src-tauri/Cargo.toml --workspace --all-targets
if ($LASTEXITCODE -ne 0) { throw 'Workspace test gate failed' }
npm.cmd run verify
if ($LASTEXITCODE -ne 0) { throw 'Repository verify gate failed' }
```

Expected: all three members pass. At this point every mandatory package, app, Linux, smoke, build, lifecycle, and completion gate is GREEN while the spec and roadmap still say implementation pending.

- [ ] **Step 3: Create the verification document from literal completed evidence.**

Use exact sections:

1. Scope and commits;
2. Final ownership and dependency roots;
3. Pre-manifest core-use audit;
4. Frozen 94-name inventory and 75/19 ownership;
5. Characterization and exact outward serialization;
6. Source-contract RED/GREEN;
7. Package, app, workspace, and Linux results;
8. Advisory timing and restoration hashes;
9. Ordinary workspace-check duration;
10. Sidecar/CDP/no-bundle/startup/shutdown evidence;
11. Infrastructure and MSI/WiX exclusions;
12. Result and next roadmap action.

Record literal commands and actual summaries, including `$workspaceCheckMs`. Timing contains the exact Task 5/7 handoff objects, medians, delta, percent, and hashes, or `incomplete / no conclusion` with restoration proof. Do not insert placeholders and never infer missing values.

If any correctness gate in Tasks 6–9 fails, stop the retained path. Keep both pending statuses, the roadmap text, and the current pending contract assertions byte-unchanged; do not add a verification link to either spec. Create only the verification file with the successful evidence, literal failing command/output, `Result: incomplete / not retained`, and the next required remediation. Then run:

```powershell
git add docs/superpowers/verification/2026-07-19-extractum-gemini-browser-extraction.md
$staged = @(git diff --cached --name-only)
if ($staged.Count -ne 1 -or $staged[0] -ne 'docs/superpowers/verification/2026-07-19-extractum-gemini-browser-extraction.md') {
  throw "Unexpected staged paths: $($staged -join ', ')"
}
git diff --cached --check -- docs/superpowers/verification/2026-07-19-extractum-gemini-browser-extraction.md
git commit -m "docs: record incomplete Gemini browser extraction"
git status --short
```

The verification file is the only staged path. Preserve any failed implementation diff for diagnosis, report the resulting status, and stop this plan without executing Steps 4–9.

- [ ] **Step 4: Change status only now, on the fully GREEN retained path.**

Apply these exact forms:

- spec status → `**Status:** Implemented and retained; [verification](../verification/2026-07-19-extractum-gemini-browser-extraction.md)`;
- roadmap heading is exactly `### Phase 4 — extractum-gemini-browser (done: retained)`, with `extractum-gemini-browser` enclosed in Markdown code backticks;
- roadmap links the same verification and records final dependencies, 75/19 ownership, and `$workspaceCheckMs` in its timing row;
- Phase 5 is next JIT design, not started.

- [ ] **Step 5: Update current policy contract.**

Change exact pending assertions in `crate-extraction-shell-cap-contract.test.ts` to the Step 4 status and heading, and assert the verification filename in both the spec and Phase 4 roadmap section. Keep timing-policy assertions unchanged.

- [ ] **Step 6: Verify the retained documentation changes.**

```powershell
npm.cmd run test -- src/lib/crate-extraction-shell-cap-contract.test.ts src/lib/gemini-browser-crate-boundary-contract.test.ts src/lib/rust-workspace-core-contract.test.ts
if ($LASTEXITCODE -ne 0) { throw 'Retained documentation/source contracts failed' }
```

Expected: all pass.

- [ ] **Step 7: Recheck evidence and intended diff.**

Confirm verification already contains actual totals/results/duration/hashes/commit IDs, then:

```powershell
npm.cmd run test -- src/lib/crate-extraction-shell-cap-contract.test.ts src/lib/gemini-browser-crate-boundary-contract.test.ts
if ($LASTEXITCODE -ne 0) { throw 'Final evidence contract recheck failed' }
git diff --check
git status --short
```

Expected: tests pass; no whitespace errors; intended changes only. `docs/value-registry.md` stays unchanged because no serialized value changed.

- [ ] **Step 8: Commit evidence.**

```powershell
git add docs/superpowers/verification/2026-07-19-extractum-gemini-browser-extraction.md docs/superpowers/specs/2026-07-19-gemini-browser-crate-boundary-design.md docs/superpowers/specs/2026-07-17-crate-roadmap.md src/lib/crate-extraction-shell-cap-contract.test.ts
git commit -m "docs: record Gemini browser crate verification"
```

- [ ] **Step 9: Verify final state.**

```powershell
git status --short
git log -5 --oneline
```

Expected: clean worktree and scoped characterization, seam/preparation, extraction, and verification commits.

---

## Appendix A: Frozen 94-Test Ownership Map

The source-boundary contract copies these arrays verbatim.

```ts
const crateOwnedBaselineTests = [
  "launch_spec_uses_endpoint_port_and_dedicated_profile",
  "launch_spec_rejects_remote_cdp_endpoint",
  "provider_status_uses_cached_snapshot_when_sidecar_is_busy",
  "provider_status_live_probe_does_not_mutate_cached_snapshot",
  "status_snapshot_core_returns_cached_status_without_polling_live_sidecar",
  "provider_status_snapshot_from_reconciled_runs_does_not_keep_stale_running_snapshot",
  "provider_status_snapshot_from_reconciled_runs_preserves_live_active_run",
  "provider_status_snapshot_from_reconciled_runs_ignores_stale_queued_rows",
  "provider_status_snapshot_read_core_writes_reconciled_snapshot_back",
  "provider_status_snapshot_read_core_skips_stale_write_back_when_snapshot_changed",
  "get_run_core_returns_exact_run_from_log",
  "provider_status_read_core_waits_for_startup_reconciliation_before_live_status",
  "send_single_prompt_handoff_writes_run_log_before_enqueue",
  "send_single_prompt_rejects_duplicate_non_terminal_run_id_before_enqueue",
  "send_single_prompt_rejects_duplicate_terminal_run_id_before_enqueue",
  "send_single_prompt_rejects_duplicate_waiter_before_enqueue",
  "send_single_prompt_marks_run_failed_when_enqueue_fails",
  "send_single_prompt_rejects_invalid_artifact_mode_before_side_effects",
  "failed_run_log_transition_returns_app_error_without_side_effects",
  "gemini_browser_job_serializes_queue_payload",
  "restart_reconciliation_degraded_leaves_queued_run_log_records",
  "restart_reconciliation_matrix_handles_supported_apalis_states",
  "restart_worker_entry_skips_terminal_cancelled_run_log",
  "restart_worker_entry_acknowledges_missing_run_log_without_sidecar",
  "degraded_apalis_queue_inspection_leaves_queued_run_log_records_for_worker_entry",
  "worker_status_blocks_enqueue_when_startup_failed",
  "worker_status_allows_enqueue_after_ready",
  "worker_status_times_out_while_starting",
  "waiter_receives_terminal_worker_result",
  "wait_for_result_removes_waiter_on_timeout",
  "wait_for_result_removes_waiter_when_worker_channel_closes",
  "register_waiter_rejects_duplicate_run_id",
  "complete_waiter_ignores_dropped_receiver",
  "runtime_tracks_and_clears_cancelled_run_ids",
  "worker_handler_marks_run_running_and_terminal",
  "worker_handler_converts_executor_error_to_terminal_failed_result",
  "cancel_gemini_browser_job_cancels_queued_run_and_waiter",
  "cancel_missing_run_returns_without_run_log_side_effects",
  "cancel_queued_run_updates_terminal_snapshot",
  "cancel_gemini_browser_job_requests_stop_for_active_run",
  "worker_startup_failure_marks_runtime_failed",
  "worker_run_failure_marks_runtime_failed",
  "worker_timeout_marks_run_failed_and_processes_next_job",
  "worker_timeout_clears_active_and_cancelled_state",
  "run_log_persists_queued_running_and_terminal_result",
  "read_run_returns_exact_run_by_id",
  "read_run_returns_validation_error_for_missing_run",
  "recorded_run_dir_requires_result_artifact_flag_and_returns_computed_dir",
  "list_runs_deletes_run_directories_outside_retention_window",
  "create_queued_run_prunes_expired_runs_before_writing_new_run",
  "recorded_run_dir_prunes_expired_run_before_opening_artifacts",
  "decode_sidecar_line_rejects_mismatched_ids",
  "decode_sidecar_line_accepts_ack_for_matching_id",
  "decode_sidecar_line_for_request_skips_stale_response_ids",
  "take_complete_jsonl_lines_handles_partial_and_multiple_chunks",
  "jsonl_transport_round_trips_a_duplex_request",
  "resume_response_classifies_legacy_ack_for_retry",
  "resolve_launch_mode_prefers_bundled_when_forced",
  "resolve_launch_mode_keeps_dev_node_fallback_for_debug_repo_runs",
  "resolve_launch_mode_uses_bundled_by_default_for_release_even_when_repo_dist_exists",
  "resolve_launch_mode_allows_explicit_dev_sidecar_override_in_release",
  "resolve_launch_mode_falls_back_to_bundled_when_debug_dev_script_is_absent",
  "bundled_sidecar_path_is_beside_the_packaged_executable",
  "state_tracks_active_run_and_cancellation",
  "status_snapshot_initializes_to_not_started_from_profile_dir",
  "update_status_snapshot_mutates_cached_status",
  "startup_reconciliation_gate_runs_once_after_success",
  "startup_reconciliation_gate_retries_after_failure",
  "set_status_snapshot_if_current_does_not_overwrite_newer_snapshot",
  "success_statuses_include_ready_and_ok",
  "sidecar_command_serializes_with_snake_case_tag",
  "manual_action_serializes_start_chrome_cdp",
  "resume_command_serializes_browser_profile_dir",
  "sidecar_command_serializes_browser_config",
  "run_result_serializes_optional_debug_summary",
] as const;

const appOwnedBaselineTests = [
  "explicit_shutdown_kills_and_reaps_the_owned_child_once",
  "drop_falls_back_to_owned_child_shutdown",
  "shutdown_does_not_claim_or_kill_an_already_exited_child",
  "shutdown_reaps_when_the_child_has_already_exited_during_kill",
  "wait_for_cdp_endpoint_accepts_json_version_response",
  "wait_for_cdp_endpoint_reports_unreachable_endpoint",
  "stderr_drain_consumes_sidecar_output_concurrently",
  "cancelled_run_marks_the_sidecar_transport_tainted",
  "apalis_storage_uses_shared_main_extractum_db_identity",
  "apalis_sqlite_storage_uses_app_managed_schema_and_worker_processes_one_job",
  "apalis_storage_preserves_existing_sqlx_migration_history_table",
  "apalis_storage_shares_extractum_db_without_locking_app_pool",
  "enqueue_duplicate_run_id_returns_conflict",
  "enqueue_persists_job_before_worker_startup",
  "worker_picks_up_job_quickly_after_idle",
  "restart_worker_processes_pending_job_after_runtime_restart",
  "apalis_sqlite_status_probe_documents_actual_status_values",
  "gemini_browser_jobs_are_built_with_one_total_attempt",
  "failed_gemini_browser_job_is_not_retried",
] as const;
```

Expected counts: 75, 19, union 94.

## Appendix B: Curated DTO Re-Exports

The crate root explicitly re-exports:

```text
GeminiBrowserProviderStatusKind
GeminiBrowserManualAction
GeminiBrowserProviderMode
GeminiBrowserDebugErrorStage
GeminiBrowserAnswerCompletionReason
GeminiBrowserCandidateRejectReason
GeminiBrowserAnswerGrouping
GeminiBrowserAnswerExtractionDebug
GeminiBrowserProviderConfig
GeminiBrowserStartChromeResult
GeminiBrowserProviderStatus
GeminiBrowserRunRequest
GeminiBrowserRunStatus
GeminiBrowserArtifactRefs
GeminiBrowserRunDebugSummary
GeminiBrowserRunResult
GeminiBrowserRun
GeminiBrowserRunLogSummary
GeminiBrowserSidecarCommand
GeminiBrowserSidecarEnvelope
GeminiBrowserSidecarResponse
```

No DTO or serde attribute changes. The app facade exposes its existing subset, while every transitive public DTO remains nameable from the new curated crate root.
