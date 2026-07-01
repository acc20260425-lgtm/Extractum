# Analysis Report Lifecycle Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract analysis report lifecycle, cancellation, and terminal status helpers from `src-tauri/src/analysis/report.rs` into `src-tauri/src/analysis/report/lifecycle.rs` while preserving current facade paths and behavior.

**Status:** active execution plan; implementation not started as of 2026-07-01 because `src-tauri/src/analysis/report/lifecycle.rs` does not exist. Mark this plan `implemented; historical execution record` only after the lifecycle refactor commit and final verification have completed.

**Architecture:** `report.rs` remains the report workflow facade and keeps map/reduce orchestration, `ReportRunError`, `RunEvent`, `ReportRunInput`, `ReportPipelineContext`, and `start_analysis_report_run`. A new private nested `lifecycle` module owns terminal status persistence and report-run cancellation request handling. `report.rs` forwards the existing public/crate-visible lifecycle facade paths so `analysis/mod.rs`, `lib.rs`, `report_commands.rs`, and inline report tests keep compiling through their current imports.

**Tech Stack:** Rust, Tauri test runtime, Tauri SQL plugin `DbInstances`, SQLx SQLite, existing `AnalysisState`, existing `LlmSchedulerState`, existing analysis report tests.

## Global Constraints

- Run commands from repository root `G:\Develop\Extractum`.
- Use cargo commands with `--manifest-path src-tauri/Cargo.toml` because the Rust manifest is under `src-tauri/`; examples in this plan use concrete subcommands such as `cargo test`, `cargo fmt`, and `cargo check`.
- Implementation-owned Rust files are `src-tauri/src/analysis/report.rs` and `src-tauri/src/analysis/report/lifecycle.rs`.
- Do not modify `src-tauri/src/analysis/mod.rs`; preserve its existing `pub use self::report::cleanup_interrupted_analysis_runs;`.
- Do not move map/reduce runtime orchestration, `ReportRunError`, `RunEvent`, `capture_report_corpus`, `start_analysis_report_run`, or report tests in this slice.
- Preserve current cancellation, interrupted-cleanup, status-update, event-emission, and error-string behavior exactly.
- Do not add root re-exports.
- Do not stage unrelated user work or unrelated rustfmt drift.

---

## File Structure

- Modify `src-tauri/src/analysis/report.rs`
  - Add `mod lifecycle;`.
  - Forward lifecycle facade paths from the new module.
  - Remove moved lifecycle helper bodies and imports only used by those helpers.
  - Keep tests in place and add focused cancellation request characterization tests before extraction.

- Create `src-tauri/src/analysis/report/lifecycle.rs`
  - Owns `fail_run`, `fail_capture_run`, `cancel_run`, `mark_interrupted_analysis_runs`, `cleanup_interrupted_analysis_runs`, and `request_analysis_run_cancel`.
  - Imports `RunEvent` and `INTERRUPTED_RUN_MESSAGE` from parent `report.rs`.

- Do not modify `src-tauri/src/analysis/mod.rs`.

---

### Task 1: Pre-Edit Baseline And Worktree Guard

**Files:**
- Inspect: `src-tauri/src/analysis/report.rs`
- Inspect if present: `src-tauri/src/analysis/report/lifecycle.rs`
- Verify: `docs/superpowers/specs/2026-07-01-analysis-report-lifecycle-refactor-design.md`

**Interfaces:**
- Consumes: approved lifecycle design spec.
- Produces: `PRE_EDIT_STATUS` baseline and clean pre-edit test baseline.

- [x] **Step 1: Capture pre-edit status**

Run:

```powershell
git status --short --untracked-files=all
```

Expected: clean output, or unrelated pre-existing changes. Save the exact output in execution notes as `PRE_EDIT_STATUS`.

Run:

```powershell
git status --short --untracked-files=all | Set-Content -Encoding utf8 -LiteralPath "$env:TEMP\analysis-report-lifecycle-pre-edit-status.txt"
Get-Content -Raw -LiteralPath "$env:TEMP\analysis-report-lifecycle-pre-edit-status.txt"
```

Expected: the file content matches the visible `git status` output. If `src-tauri/src/analysis/report.rs` or `src-tauri/src/analysis/report/lifecycle.rs` appears, inspect Steps 2-3 before editing.

- [x] **Step 2: Inspect dirty tracked target files**

Run separately if a target file is modified or staged:

```powershell
git diff -- src-tauri/src/analysis/report.rs
```

```powershell
git diff --cached -- src-tauri/src/analysis/report.rs
```

```powershell
git diff -- src-tauri/src/analysis/report/lifecycle.rs
```

```powershell
git diff --cached -- src-tauri/src/analysis/report/lifecycle.rs
```

Expected: no unreviewed target-file baseline changes. Stop before editing if a pre-existing target-file change overlaps lifecycle extraction.

- [x] **Step 3: Inspect pre-existing untracked lifecycle module**

Run:

```powershell
if (Test-Path -LiteralPath 'src-tauri/src/analysis/report/lifecycle.rs') {
    git status --short --untracked-files=all -- src-tauri/src/analysis/report/lifecycle.rs
    Get-Item -LiteralPath 'src-tauri/src/analysis/report/lifecycle.rs' | Select-Object FullName, Length, LastWriteTime
    Get-FileHash -Algorithm SHA256 -LiteralPath 'src-tauri/src/analysis/report/lifecycle.rs'
    Get-Content -Raw -LiteralPath 'src-tauri/src/analysis/report/lifecycle.rs'
}
```

Expected: if the file exists, you capture status, length, SHA-256 hash, and raw contents. Stop before editing if it contains pre-existing user work.

- [x] **Step 4: Establish report baseline**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::
```

Expected: PASS and not a green `0 tests` run.

- [x] **Step 5: Establish interrupted cleanup focused baseline**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::interrupted_cleanup_preserves_captured_snapshot_state_marker
```

Expected: PASS with `1 passed`, not a green `0 tests` run.

- [x] **Step 6: Establish corpus boundary baseline**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::
```

Expected: PASS and not a green `0 tests` run.

---

### Task 2: Add Cancellation Request Characterization Tests

**Files:**
- Modify: `src-tauri/src/analysis/report.rs`

**Interfaces:**
- Consumes: current `analysis::report::request_analysis_run_cancel` facade path.
- Produces: focused `request_analysis_run_cancel_` tests that guard current error strings before lifecycle extraction.

**Execution adjustment:** `tauri::test` requires the `tauri/test` feature in this workspace, and enabling it caused the Windows test binary to fail before tests ran with `STATUS_ENTRYPOINT_NOT_FOUND`. With user approval, Task 2 uses a small pool-based cancellation core inside `report.rs` for the three error-string characterization tests. The command consumer/facade path remains guarded by the existing `report_commands.rs` import plus Task 4 facade source checks and `cargo check --all-targets`.

- [x] **Step 1: Add test imports**

In the `#[cfg(test)] mod tests` import block in `src-tauri/src/analysis/report.rs`, add `request_analysis_run_cancel` to the existing `use super::{...}` list and extend the LLM import:

```rust
use super::{
    build_map_request, build_reduce_request, capture_report_corpus,
    chunk_target_chars_for_model_input_limit, finish_map_phase, mark_interrupted_analysis_runs,
    parse_chunk_summary, request_analysis_run_cancel, resolve_analysis_telegram_history_scope,
    run_analysis_step_with_cancel, validate_report_preflight, ReduceRequestParams,
    ReportRunError, ReportRunInput, StartAnalysisReportRequest,
};
use crate::llm::{LlmRequestError, LlmSchedulerState, ProviderKind, ResolvedLlmProfile};
```

Also add these imports in the test module:

```rust
use crate::db::DB_URL;
use crate::analysis::report_commands::cancel_analysis_run;
use sqlx::SqlitePool;
use tauri::Manager;
use tauri_plugin_sql::{DbInstances, DbPool};
```

- [x] **Step 2: Add DB test harness helpers**

Add these helpers near the existing sample helpers in the test module:

```rust
async fn app_handle_with_analysis_pool(pool: SqlitePool) -> tauri::AppHandle {
    let app = tauri::test::mock_builder()
        .manage(DbInstances::default())
        .manage(crate::analysis::AnalysisState::new())
        .manage(LlmSchedulerState::new())
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
        .expect("build test app");
    let handle = app.handle().clone();
    {
        let instances = handle.state::<DbInstances>();
        let mut instances = instances.0.write().await;
        instances.insert(DB_URL.to_string(), DbPool::Sqlite(pool));
    }
    handle
}

async fn request_cancel_pool_with_runs() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect memory sqlite");

    sqlx::query(
        "CREATE TABLE analysis_runs (
            id INTEGER PRIMARY KEY,
            run_type TEXT NOT NULL DEFAULT 'report',
            scope_type TEXT NOT NULL DEFAULT 'single_source',
            source_id INTEGER,
            source_group_id INTEGER,
            project_id INTEGER,
            period_from INTEGER NOT NULL DEFAULT 0,
            period_to INTEGER NOT NULL DEFAULT 0,
            output_language TEXT NOT NULL DEFAULT 'English',
            prompt_template_id INTEGER NOT NULL DEFAULT 1,
            prompt_template_version INTEGER NOT NULL DEFAULT 1,
            provider_profile TEXT NOT NULL DEFAULT 'research',
            provider TEXT NOT NULL DEFAULT 'gemini',
            model TEXT NOT NULL DEFAULT 'gemini-2.5-flash',
            youtube_corpus_mode TEXT NOT NULL DEFAULT 'transcript_description',
            telegram_history_scope TEXT,
            status TEXT NOT NULL,
            result_markdown TEXT,
            trace_data_zstd BLOB,
            scope_label_snapshot TEXT,
            snapshot_captured_at TEXT,
            snapshot_error TEXT,
            error TEXT,
            created_at INTEGER NOT NULL DEFAULT 1,
            completed_at INTEGER
        )",
    )
    .execute(&pool)
    .await
    .expect("create analysis_runs");

    sqlx::query("CREATE TABLE sources (id INTEGER PRIMARY KEY, title TEXT)")
        .execute(&pool)
        .await
        .expect("create sources");
    sqlx::query("CREATE TABLE analysis_source_groups (id INTEGER PRIMARY KEY, name TEXT)")
        .execute(&pool)
        .await
        .expect("create groups");
    sqlx::query("CREATE TABLE projects (id INTEGER PRIMARY KEY, name TEXT)")
        .execute(&pool)
        .await
        .expect("create projects");
    sqlx::query("CREATE TABLE analysis_prompt_templates (id INTEGER PRIMARY KEY, name TEXT)")
        .execute(&pool)
        .await
        .expect("create templates");
    sqlx::query("CREATE TABLE analysis_run_messages (run_id INTEGER NOT NULL)")
        .execute(&pool)
        .await
        .expect("create run messages");

    pool
}

async fn insert_cancel_request_run(pool: &SqlitePool, run_id: i64, status: &str) {
    sqlx::query(
        "INSERT INTO analysis_runs (
            id, run_type, scope_type, status, period_from, period_to, output_language,
            prompt_template_id, prompt_template_version, provider_profile, provider, model,
            youtube_corpus_mode, created_at
        ) VALUES (
            ?, 'report', 'single_source', ?, 1, 2, 'English', 1, 1,
            'research', 'gemini', 'gemini-2.5-flash', 'transcript_description', 1
        )",
    )
    .bind(run_id)
    .bind(status)
    .execute(pool)
    .await
    .expect("insert analysis run");
}
```

The helper schema and insert values must keep `provider_profile`, `provider`, `model`, and `youtube_corpus_mode` non-null because `fetch_run_row` decodes them into `AnalysisRunRow` as `String`, not `Option<String>`. Do not remove these defaults or explicit inserted values; otherwise the characterization tests can fail during SQLx row decoding before reaching the cancellation error branches.

- [x] **Step 3: Add missing-run characterization test**

Add this test near `interrupted_cleanup_preserves_captured_snapshot_state_marker`:

```rust
#[tokio::test]
async fn request_analysis_run_cancel_missing_run_keeps_not_found_message() {
    let pool = request_cancel_pool_with_runs().await;
    let handle = app_handle_with_analysis_pool(pool).await;
    let state = crate::analysis::AnalysisState::new();
    let scheduler = LlmSchedulerState::new();
    let run_id = 404;

    let error = request_analysis_run_cancel(&handle, &state, &scheduler, run_id)
        .await
        .expect_err("missing run should fail");

    assert_eq!(error.kind, AppErrorKind::NotFound);
    assert_eq!(error.message, format!("Analysis run {run_id} not found"));
}
```

- [x] **Step 4: Add non-cancellable-status characterization test**

Add this test:

```rust
#[tokio::test]
async fn request_analysis_run_cancel_completed_run_keeps_conflict_message() {
    let pool = request_cancel_pool_with_runs().await;
    insert_cancel_request_run(&pool, 405, crate::analysis::ANALYSIS_STATUS_COMPLETED).await;
    let handle = app_handle_with_analysis_pool(pool).await;
    let state = crate::analysis::AnalysisState::new();
    let scheduler = LlmSchedulerState::new();
    let run_id = 405;

    let error = request_analysis_run_cancel(&handle, &state, &scheduler, run_id)
        .await
        .expect_err("completed run should fail");

    assert_eq!(error.kind, AppErrorKind::Conflict);
    assert_eq!(
        error.message,
        format!("Analysis run {run_id} is not queued or running")
    );
}
```

- [x] **Step 5: Add inactive active-run characterization test**

Add this test:

```rust
#[tokio::test]
async fn request_analysis_run_cancel_running_but_inactive_keeps_conflict_message() {
    let pool = request_cancel_pool_with_runs().await;
    insert_cancel_request_run(&pool, 406, crate::analysis::ANALYSIS_STATUS_RUNNING).await;
    let handle = app_handle_with_analysis_pool(pool).await;
    let state = crate::analysis::AnalysisState::new();
    let scheduler = LlmSchedulerState::new();
    let run_id = 406;

    let error = request_analysis_run_cancel(&handle, &state, &scheduler, run_id)
        .await
        .expect_err("inactive running run should fail");

    assert_eq!(error.kind, AppErrorKind::Conflict);
    assert_eq!(
        error.message,
        format!("Analysis run {run_id} is no longer active")
    );
}
```

- [x] **Step 6: Preserve command consumer facade guard through compile/source checks**

Approved adjustment: do not add this runtime-level Tauri command test in Task 2. The local Tauri test runtime path failed before test execution in this Windows workspace. Preserve the consumer guard through `src-tauri/src/analysis/report_commands.rs` continuing to call `report::request_analysis_run_cancel`, Task 4 facade source checks, and `cargo check --all-targets`.

- [x] **Step 7: Run focused characterization tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::request_analysis_run_cancel_
```

Expected: PASS and not a green `0 tests` run. Output includes three tests covering missing run, completed run, and inactive running run.

- [x] **Step 8: Run report tests after characterization**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::
```

Expected: PASS and not a green `0 tests` run.

- [x] **Step 8: Commit characterization tests**

Run:

```powershell
git add -- src-tauri/src/analysis/report.rs
git diff --cached --check
git commit -m "test: cover analysis report cancellation request errors"
```

Expected: commit succeeds and contains only the report test additions.

---

### Task 3: Extract Lifecycle Helpers

**Files:**
- Create: `src-tauri/src/analysis/report/lifecycle.rs`
- Modify: `src-tauri/src/analysis/report.rs`

**Interfaces:**
- Consumes from `report.rs`: `RunEvent`, `INTERRUPTED_RUN_MESSAGE`.
- Produces through `report.rs` facade:

```rust
pub use self::lifecycle::cleanup_interrupted_analysis_runs;
pub(crate) use self::lifecycle::{mark_interrupted_analysis_runs, request_analysis_run_cancel};
```

- Produces inside `analysis::report`:

```rust
pub(super) async fn fail_run(handle: &AppHandle, run_id: i64, error: String);
pub(super) async fn fail_capture_run(handle: &AppHandle, run_id: i64, error: String);
pub(super) async fn cancel_run(handle: &AppHandle, run_id: i64, message: String);
```

- [x] **Step 1: Create lifecycle module header**

Create `src-tauri/src/analysis/report/lifecycle.rs`:

```rust
use sqlx::{Pool, Sqlite};
use tauri::AppHandle;

use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::llm::LlmSchedulerState;

use super::{RunEvent, INTERRUPTED_RUN_MESSAGE};
use super::super::state::AnalysisState;
use super::super::store::{
    fetch_run_row, mark_run_capture_failed, sanitize_provider_error, set_run_status,
};
use super::super::{
    now_secs, ANALYSIS_STATUS_CANCELLED, ANALYSIS_STATUS_FAILED, ANALYSIS_STATUS_QUEUED,
    ANALYSIS_STATUS_RUNNING,
};
```

- [x] **Step 2: Move `fail_run` byte-for-byte**

Cut `fail_run` from `report.rs` and paste it into `lifecycle.rs`. Change only visibility:

```rust
pub(super) async fn fail_run(handle: &AppHandle, run_id: i64, error: String) {
```

Preserve `sanitize_provider_error("Report run failed", &error)`, the `set_run_status` arguments, and the emitted event body.

- [x] **Step 3: Move `fail_capture_run` byte-for-byte**

Cut `fail_capture_run` from `report.rs` and paste it into `lifecycle.rs`. Change only visibility:

```rust
pub(super) async fn fail_capture_run(handle: &AppHandle, run_id: i64, error: String) {
```

Preserve `mark_run_capture_failed`, timestamp behavior, message text, and event error payload.

- [x] **Step 4: Move `cancel_run` byte-for-byte**

Cut `cancel_run` from `report.rs` and paste it into `lifecycle.rs`. Change only visibility:

```rust
pub(super) async fn cancel_run(handle: &AppHandle, run_id: i64, message: String) {
```

Preserve `ANALYSIS_STATUS_CANCELLED`, message persistence, and cancelled event emission.

- [x] **Step 5: Move `mark_interrupted_analysis_runs` byte-for-byte**

Cut `mark_interrupted_analysis_runs` from `report.rs` and paste it into `lifecycle.rs`. Keep crate visibility:

```rust
pub(crate) async fn mark_interrupted_analysis_runs(pool: &Pool<Sqlite>) -> AppResult<()> {
```

Preserve SQL text, bind order, `INTERRUPTED_RUN_MESSAGE`, and `now_secs()`.

- [x] **Step 6: Move `cleanup_interrupted_analysis_runs` byte-for-byte**

Cut `cleanup_interrupted_analysis_runs` from `report.rs` and paste it into `lifecycle.rs`. Keep public visibility:

```rust
pub async fn cleanup_interrupted_analysis_runs(handle: AppHandle) {
```

Preserve best-effort pool resolution and ignored cleanup result.

- [x] **Step 7: Move `request_analysis_run_cancel` byte-for-byte**

Cut `request_analysis_run_cancel` from `report.rs` and paste it into `lifecycle.rs`. Keep crate visibility:

```rust
pub(crate) async fn request_analysis_run_cancel(
    handle: &AppHandle,
    state: &AnalysisState,
    scheduler: &LlmSchedulerState,
    run_id: i64,
) -> AppResult<()> {
```

Preserve all three error strings exactly:

```rust
format!("Analysis run {run_id} not found")
format!("Analysis run {run_id} is not queued or running")
format!("Analysis run {run_id} is no longer active")
```

- [x] **Step 8: Declare module and facade forwards in `report.rs`**

Add near the existing `mod requests;` declaration:

```rust
mod lifecycle;
```

Add explicit lifecycle facade/imports near the existing `use self::requests::{...};` block:

```rust
pub use self::lifecycle::cleanup_interrupted_analysis_runs;
pub(crate) use self::lifecycle::{mark_interrupted_analysis_runs, request_analysis_run_cancel};
use self::lifecycle::{cancel_run, fail_capture_run, fail_run};
```

Do not modify `src-tauri/src/analysis/mod.rs`.

- [x] **Step 9: Widen only required `RunEvent` surface**

In `report.rs`, change:

```rust
struct RunEvent {
```

to:

```rust
pub(super) struct RunEvent {
```

Change only these methods to `pub(super)`:

```rust
pub(super) fn new(run_id: i64, kind: &str, phase: &str) -> Self
pub(super) fn message(mut self, message: String) -> Self
pub(super) fn error(mut self, error: String) -> Self
pub(super) fn emit(self, handle: &AppHandle)
```

Keep `request_id`, `queue_position`, `progress`, `delta`, and `chunk_summary` private.

- [x] **Step 10: Widen only `INTERRUPTED_RUN_MESSAGE`**

Change:

```rust
const INTERRUPTED_RUN_MESSAGE: &str = "Analysis run was interrupted when the app was restarted.";
```

to:

```rust
pub(super) const INTERRUPTED_RUN_MESSAGE: &str =
    "Analysis run was interrupted when the app was restarted.";
```

Keep `CANCELLED_RUN_MESSAGE` and `SNAPSHOT_CAPTURE_FAILED_MESSAGE` private.

- [x] **Step 11: Remove moved-only imports from `report.rs`**

In `report.rs`, remove these from `super::store::{...}` if no remaining code uses them:

```rust
fetch_run_row,
mark_run_capture_failed,
sanitize_provider_error,
```

Keep `get_pool`, `set_run_status`, `sanitize_snapshot_error`, and `capture_run_snapshot` in `report.rs` because the remaining report pipeline still uses them.

- [x] **Step 12: Confirm moved definitions are gone from `report.rs`**

Run:

```powershell
rg -n "^(async fn fail_run|async fn fail_capture_run|async fn cancel_run|pub\\(crate\\) async fn mark_interrupted_analysis_runs|pub async fn cleanup_interrupted_analysis_runs|pub\\(crate\\) async fn request_analysis_run_cancel)" src-tauri/src/analysis/report.rs
```

Expected: no matches.

- [x] **Step 13: Confirm lifecycle module owns moved definitions**

Run:

```powershell
rg -n "^(pub\\(super\\) async fn fail_run|pub\\(super\\) async fn fail_capture_run|pub\\(super\\) async fn cancel_run|pub\\(crate\\) async fn mark_interrupted_analysis_runs|pub async fn cleanup_interrupted_analysis_runs|pub\\(crate\\) async fn request_analysis_run_cancel)" src-tauri/src/analysis/report/lifecycle.rs
```

Expected: six matches, one for each moved function.

---

### Task 4: Verify Lifecycle Extraction

**Files:**
- Verify: `src-tauri/src/analysis/report.rs`
- Verify: `src-tauri/src/analysis/report/lifecycle.rs`
- Verify unchanged facade: `src-tauri/src/analysis/mod.rs`
- Verify consumer: `src-tauri/src/analysis/report_commands.rs`

**Interfaces:**
- Consumes: lifecycle extraction from Task 3.
- Produces: verified Rust refactor ready to commit.

- [ ] **Step 1: Format Rust code**

Run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml
```

Expected: command exits 0.

- [ ] **Step 2: Check changed file list after formatting**

Run each command separately:

```powershell
git status --short --untracked-files=all
git diff --name-only
git diff --cached --name-only
git ls-files --others --exclude-standard
```

Expected implementation-owned changed paths only:

```text
src-tauri/src/analysis/report.rs
src-tauri/src/analysis/report/lifecycle.rs
```

If unrelated paths appear and were not present in `PRE_EDIT_STATUS`, resolve them before committing.

- [ ] **Step 3: Verify facade paths**

Run each command separately:

```powershell
rg -n "pub use self::lifecycle::cleanup_interrupted_analysis_runs" src-tauri/src/analysis/report.rs
rg -n "mark_interrupted_analysis_runs" src-tauri/src/analysis/report.rs
rg -n "request_analysis_run_cancel" src-tauri/src/analysis/report.rs
rg -n "pub use self::report::cleanup_interrupted_analysis_runs" src-tauri/src/analysis/mod.rs
rg -n "report::request_analysis_run_cancel" src-tauri/src/analysis/report_commands.rs
```

Expected: each command has at least one match. The `mark_interrupted_analysis_runs` and `request_analysis_run_cancel` matches in `report.rs` must come from facade forwarding/imports or tests, not from leftover moved function bodies. Do not require one exact grouped `pub(crate) use ... { ... }` line; split imports and rustfmt-wrapped imports are acceptable if the effective paths remain `analysis::report::mark_interrupted_analysis_runs` and `analysis::report::request_analysis_run_cancel`.

- [ ] **Step 4: Verify `RunEvent` surface**

Run:

```powershell
rg -n "RunEvent::(request_id|queue_position|progress|delta|chunk_summary)|\\.event" src-tauri/src/analysis/report/lifecycle.rs
```

Expected: no matches.

Run each command separately:

```powershell
rg -n "pub\\(super\\) struct RunEvent" src-tauri/src/analysis/report.rs
rg -n "pub\\(super\\) fn new" src-tauri/src/analysis/report.rs
rg -n "pub\\(super\\) fn message" src-tauri/src/analysis/report.rs
rg -n "pub\\(super\\) fn error" src-tauri/src/analysis/report.rs
rg -n "pub\\(super\\) fn emit" src-tauri/src/analysis/report.rs
rg -n "pub\\(super\\) fn (request_id|queue_position|progress|delta|chunk_summary)" src-tauri/src/analysis/report.rs
```

Expected: the first five commands each have at least one match; the final command has no matches. This avoids false negatives from acceptable rustfmt or import formatting while still preventing lifecycle from widening map/reduce event-builder methods.

- [ ] **Step 5: Run focused cancellation request tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::request_analysis_run_cancel_
```

Expected: PASS and not a green `0 tests` run. Output includes missing run, completed run, inactive running run, and command consumer path tests.

- [ ] **Step 6: Run focused interrupted cleanup test**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::interrupted_cleanup_preserves_captured_snapshot_state_marker
```

Expected: PASS with `1 passed`, not a green `0 tests` run.

- [ ] **Step 7: Run all report tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::
```

Expected: PASS and not a green `0 tests` run.

- [ ] **Step 8: Run corpus boundary tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::
```

Expected: PASS and not a green `0 tests` run.

- [ ] **Step 9: Run formatting check**

Run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
```

Expected: PASS with no diff output.

- [ ] **Step 10: Run all-target compile check**

Run:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
```

Expected: PASS. Existing warnings outside touched files may remain; new warnings mentioning `src-tauri/src/analysis/report.rs` or `src-tauri/src/analysis/report/lifecycle.rs` are not acceptable.

- [ ] **Step 11: Inspect final Rust diff**

Run:

```powershell
git diff -- src-tauri/src/analysis/report.rs src-tauri/src/analysis/report/lifecycle.rs
```

Expected:

- `report.rs` keeps pipeline code and tests.
- `report.rs` has lifecycle module declaration and facade forwards.
- `report.rs` no longer contains moved lifecycle helper bodies.
- `lifecycle.rs` contains moved lifecycle helper bodies with only visibility/import changes.
- `src-tauri/src/analysis/mod.rs` is unchanged.

- [ ] **Step 12: Check whitespace**

Run:

```powershell
git diff --check -- src-tauri/src/analysis/report.rs src-tauri/src/analysis/report/lifecycle.rs
```

Expected: no whitespace errors.

---

### Task 5: Commit Lifecycle Refactor

**Files:**
- Stage: `src-tauri/src/analysis/report.rs`
- Stage: `src-tauri/src/analysis/report/lifecycle.rs`

**Interfaces:**
- Consumes: verified lifecycle extraction from Task 4.
- Produces: one Rust refactor commit.

- [ ] **Step 1: Confirm intended files before staging**

Run:

```powershell
git status --short --untracked-files=all
```

Expected: only implementation-owned files are newly dirty beyond `PRE_EDIT_STATUS`.

- [ ] **Step 2: Stage implementation-owned files**

Run:

```powershell
git add -- src-tauri/src/analysis/report.rs src-tauri/src/analysis/report/lifecycle.rs
```

- [ ] **Step 3: Verify staged content**

Run:

```powershell
git diff --cached --stat
```

Expected: staged files are only `report.rs` and `report/lifecycle.rs`.

Run:

```powershell
git diff --cached --check
```

Expected: no whitespace errors.

- [ ] **Step 4: Commit**

Run:

```powershell
git commit -m "refactor: extract analysis report lifecycle helpers"
```

Expected: commit succeeds.

- [ ] **Step 5: Confirm final status against baseline**

Run:

```powershell
git status --short --untracked-files=all
```

Then run:

```powershell
git status --short --untracked-files=all | Set-Content -Encoding utf8 -LiteralPath "$env:TEMP\analysis-report-lifecycle-final-status.txt"
Compare-Object `
    (Get-Content -LiteralPath "$env:TEMP\analysis-report-lifecycle-pre-edit-status.txt") `
    (Get-Content -LiteralPath "$env:TEMP\analysis-report-lifecycle-final-status.txt")
```

Expected: no `Compare-Object` output if `PRE_EDIT_STATUS` was clean. If `PRE_EDIT_STATUS` had unrelated entries, any output must be explained and must not include lifecycle refactor files or new rustfmt drift.

---

## Self-Review

- Spec coverage: covers lifecycle-only extraction, facade preservation, root re-export preservation, `RunEvent` surface guards, cancellation request characterization tests, interrupted cleanup guard, corpus boundary tests, fmt, all-target compile, and status hygiene.
- Red-flag scan: no unfinished work items are intentionally left open.
- Type consistency: uses current `AnalysisState`, `LlmSchedulerState`, `AppError.message`, `DbInstances`, `DbPool::Sqlite`, and existing report facade paths.

---

## Execution Options

Plan complete and saved to `docs/superpowers/plans/2026-07-01-analysis-report-lifecycle-refactor.md`. Two execution options:

1. **Subagent-Driven (recommended)** - dispatch a fresh subagent per task, review between tasks, fast iteration.
2. **Inline Execution** - execute tasks in this session using executing-plans, batch execution with checkpoints.
