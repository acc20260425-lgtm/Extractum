# Analysis Report Tests Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Status:** active implementation plan; design approved, implementation not started as of 2026-07-03 because `src-tauri/src/analysis/report/tests/` does not exist.

**Goal:** Move the inline `#[cfg(test)] mod tests` body out of `src-tauri/src/analysis/report.rs` into focused nested report test modules without changing production behavior, visibility, assertions, or coverage.

**Architecture:** Keep `report.rs` as the production report workflow and facade, with only `#[cfg(test)] mod tests;` for test wiring. Create `src-tauri/src/analysis/report/tests/` with a small shared harness plus thematic modules for scope, capture, lifecycle, request, phase, preflight, and architecture tests. Keep tests exercising the parent report facade through explicit named `super::super` imports; add private parent `#[cfg(test)] use` imports when a production child helper must remain private.

**Tech Stack:** Rust, Tauri backend, SQLx SQLite in-memory tests, Cargo test/check/fmt with `--manifest-path src-tauri/Cargo.toml`, PowerShell on Windows.

## Global Constraints

- This is a Rust test-only refactor; do not change production behavior, report lifecycle behavior, capture behavior, request payloads, provider phase behavior, cancellation behavior, SQL schemas, fixture rows, validation messages, event payloads, assertions, or test coverage.
- Do not move or edit production code in `src-tauri/src/analysis/report/capture.rs`, `src-tauri/src/analysis/report/lifecycle.rs`, `src-tauri/src/analysis/report/phases.rs`, or `src-tauri/src/analysis/report/requests.rs`.
- Do not move `ReportRunError`, `RunEvent`, `ReportRunInput`, `StartAnalysisReportRequest`, `validate_report_preflight`, `run_report_pipeline`, or `start_analysis_report_run` out of `src-tauri/src/analysis/report.rs`.
- Keep production visibility unchanged. Do not widen production items to `pub(super)`, `pub(crate)`, or `pub` for this test split.
- The only allowed production-side test access adjustment is a private `#[cfg(test)] use self::requests::extract_json_payload;` facade import in `report.rs` if needed for moved request tests.
- Shared test helpers may use `pub(super)` only; do not use `pub(crate)` or `pub` in `src-tauri/src/analysis/report/tests/`.
- Tests must exercise parent report facade imports. Do not import private production child modules directly from report tests, including paths starting with `super::super::capture::`, `super::super::lifecycle::`, `super::super::phases::`, `super::super::requests::`, or `crate::analysis::report::{capture,lifecycle,phases,requests}::`.
- Keep imports explicit. Do not use `use super::super::*` or crate glob imports in report test modules.
- Run commands from the repository root with `--manifest-path src-tauri/Cargo.toml`.
- Run every `cargo test` command in the default dev test profile; do not use `--release` for required report slices.
- Run each `cargo`, `git`, and guard command separately or through a wrapper that stops on non-zero `$LASTEXITCODE`; plain multi-command PowerShell blocks can hide failures.
- Every filtered `cargo test` command in this plan must run real tests, not green `0 tests` runs.
- Target files must be clean before editing. If `src-tauri/src/analysis/report.rs` or `src-tauri/src/analysis/report/tests/` has pre-existing tracked, staged, modified, or untracked work, stop and make a separate baseline decision before starting.
- Do not stage unrelated dirty files, including `.claude/settings.local.json`.

---

## File Structure

- Modify: `src-tauri/src/analysis/report.rs`
  - Keep production definitions, production module declarations, imports, re-exports, and workflow code.
  - Add `extract_json_payload` to the existing private `#[cfg(test)] use self::requests::{build_map_request, build_reduce_request, parse_chunk_summary, ReduceRequestParams};` facade import if moved tests need it.
  - Replace the inline test module body with `#[cfg(test)] mod tests;`.

- Create: `src-tauri/src/analysis/report/tests/mod.rs`
  - Declare child test modules only.

- Create: `src-tauri/src/analysis/report/tests/harness.rs`
  - Own shared fixture constructors and cancel-request database helpers:
    - `SAMPLE_JSON`
    - `sample_chunk_summary`
    - `sample_prompt_template`
    - `sample_corpus_message`
    - `sample_resolved_profile`
    - `request_cancel_pool_with_runs`
    - `insert_cancel_request_run`

- Create: `src-tauri/src/analysis/report/tests/scope.rs`
  - Own report input, migrated Telegram history, request-shape, and chunk target sizing tests.

- Create: `src-tauri/src/analysis/report/tests/capture.rs`
  - Own capture snapshot reload behavior test.

- Create: `src-tauri/src/analysis/report/tests/lifecycle.rs`
  - Own interrupted cleanup and cancel-request tests.

- Create: `src-tauri/src/analysis/report/tests/requests.rs`
  - Own JSON extraction, chunk summary parsing, and map/reduce request construction tests.

- Create: `src-tauri/src/analysis/report/tests/phases.rs`
  - Own cancellation wrapper and map-phase finishing tests.

- Create: `src-tauri/src/analysis/report/tests/preflight.rs`
  - Own report preflight validation tests.

- Create: `src-tauri/src/analysis/report/tests/architecture.rs`
  - Own report architecture guard test.

---

### Task 1: Split Report Tests Into Nested Modules

**Files:**
- Modify: `src-tauri/src/analysis/report.rs`
- Create: `src-tauri/src/analysis/report/tests/mod.rs`
- Create: `src-tauri/src/analysis/report/tests/harness.rs`
- Create: `src-tauri/src/analysis/report/tests/scope.rs`
- Create: `src-tauri/src/analysis/report/tests/capture.rs`
- Create: `src-tauri/src/analysis/report/tests/lifecycle.rs`
- Create: `src-tauri/src/analysis/report/tests/requests.rs`
- Create: `src-tauri/src/analysis/report/tests/phases.rs`
- Create: `src-tauri/src/analysis/report/tests/preflight.rs`
- Create: `src-tauri/src/analysis/report/tests/architecture.rs`

**Interfaces:**
- Consumes:
  - Current inline `#[cfg(test)] mod tests` from `src-tauri/src/analysis/report.rs`.
  - Parent report facade imports through explicit named `super::super` import lists inside thematic test modules.
  - Private parent `#[cfg(test)] use` imports for production child helpers that tests need, including `extract_json_payload`.
- Produces:
  - `#[cfg(test)] mod tests;` in `src-tauri/src/analysis/report.rs`.
  - `tests/mod.rs` declaring `architecture`, `capture`, `harness`, `lifecycle`, `phases`, `preflight`, `requests`, and `scope`.
  - Same test functions under new paths such as `analysis::report::tests::requests::build_reduce_request_keeps_run_scoped_request_and_profile`.

- [ ] **Step 1: Capture pre-edit worktree status**

Run:

```powershell
git status --short --untracked-files=all
```

Expected:

- `src-tauri/src/analysis/report.rs` is not modified or staged.
- `src-tauri/src/analysis/report/tests/` does not exist, or the executor stops for an explicit baseline decision before editing.
- Unrelated local files such as `.claude/settings.local.json` may exist, but must remain unstaged throughout this task.

- [ ] **Step 2: Persist a pre-edit status snapshot**

Run:

```powershell
$tag = "analysis-report-tests-refactor-" + (Get-Date -Format "yyyyMMddHHmmss")
$preEditStatusPath = Join-Path $env:TEMP "$tag-status-before.txt"
$pointerPath = Join-Path $env:TEMP "extractum-analysis-report-tests-refactor-status-pointer.txt"
git status --short --untracked-files=all | Set-Content -LiteralPath $preEditStatusPath
$preEditStatusPath | Set-Content -LiteralPath $pointerPath
$pointerPath
Get-Content -LiteralPath $pointerPath
```

Expected: PowerShell prints the pointer file path and then the saved status snapshot path. Later status comparison reads the path from the pointer file, so it works across separate shell sessions.

- [ ] **Step 3: Inspect target-file baseline**

Run:

```powershell
git diff -- src-tauri/src/analysis/report.rs src-tauri/src/analysis/report/tests
```

Expected: no diff.

Run:

```powershell
git diff --cached -- src-tauri/src/analysis/report.rs src-tauri/src/analysis/report/tests
```

Expected: no staged diff.

Run:

```powershell
git ls-files src-tauri/src/analysis/report/tests
```

Expected: no output. If any tracked `report/tests` file appears, stop and make a separate baseline decision before continuing.

Run:

```powershell
if (Test-Path -LiteralPath 'src-tauri/src/analysis/report/tests') {
    git status --short --untracked-files=all -- src-tauri/src/analysis/report/tests
    Get-ChildItem -Recurse -Force -LiteralPath 'src-tauri/src/analysis/report/tests'
    Get-ChildItem -Recurse -File -Force -LiteralPath 'src-tauri/src/analysis/report/tests' |
        ForEach-Object { $_.FullName; Get-Content -Raw -LiteralPath $_.FullName }
    throw "report/tests already exists; stop for a baseline decision"
}
```

Expected: no output if the directory does not exist. If it exists in any form, this command prints the baseline and stops.

- [ ] **Step 4: Run baseline report tests and compile check**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::
```

Expected: pass in the default dev test profile and not a green `0 tests` run. Current snapshot at plan authoring has inline report tests under `analysis::report::tests::`; do not require an exact count if nearby tests change before execution.

Run:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
```

Expected: pass. This establishes crate-wide compile coverage before the module-boundary refactor.

If either baseline command fails, stop. Record the failure as pre-existing and do not edit production code in this task.

- [ ] **Step 5: Create the nested test module declarations**

Create `src-tauri/src/analysis/report/tests/mod.rs`:

```rust
mod architecture;
mod capture;
mod harness;
mod lifecycle;
mod phases;
mod preflight;
mod requests;
mod scope;
```

Create empty files:

```text
src-tauri/src/analysis/report/tests/harness.rs
src-tauri/src/analysis/report/tests/scope.rs
src-tauri/src/analysis/report/tests/capture.rs
src-tauri/src/analysis/report/tests/lifecycle.rs
src-tauri/src/analysis/report/tests/requests.rs
src-tauri/src/analysis/report/tests/phases.rs
src-tauri/src/analysis/report/tests/preflight.rs
src-tauri/src/analysis/report/tests/architecture.rs
```

- [ ] **Step 6: Add parent test facade access for `extract_json_payload`**

Modify the existing `#[cfg(test)] use self::requests::{build_map_request, build_reduce_request, parse_chunk_summary, ReduceRequestParams};` block in `src-tauri/src/analysis/report.rs` so it includes `extract_json_payload`:

```rust
#[cfg(test)]
use self::requests::{
    build_map_request, build_reduce_request, extract_json_payload, parse_chunk_summary,
    ReduceRequestParams,
};
```

Expected: moved request tests can import `extract_json_payload` through `super::super::extract_json_payload`; `requests.rs` remains private and no production child module visibility is widened.

- [ ] **Step 7: Move shared harness helpers**

Move these items from the inline test module in `src-tauri/src/analysis/report.rs` into `src-tauri/src/analysis/report/tests/harness.rs`:

- `SAMPLE_JSON`
- `sample_chunk_summary`
- `sample_prompt_template`
- `sample_corpus_message`
- `sample_resolved_profile`
- `request_cancel_pool_with_runs`
- `insert_cancel_request_run`

Use this import and visibility shape in `harness.rs`:

```rust
use crate::analysis::models::{AnalysisPromptTemplate, ChunkSummary, CorpusMessage};
use crate::llm::{ProviderKind, ResolvedLlmProfile};
use sqlx::SqlitePool;

pub(super) const SAMPLE_JSON: &str = r#"{"summary":"Brief","topics":["sync"],"notable_points":["Point"],"candidate_refs":["s1-i2"]}"#;

pub(super) fn sample_chunk_summary(label: &str) -> ChunkSummary {
    ChunkSummary {
        summary: label.to_string(),
        topics: vec![format!("{label}-topic")],
        notable_points: vec![format!("{label}-point")],
        candidate_refs: vec![format!("{label}-ref")],
    }
}

pub(super) fn sample_prompt_template() -> AnalysisPromptTemplate {
    AnalysisPromptTemplate {
        id: 7,
        name: "Report".to_string(),
        template_kind: "report".to_string(),
        body: "Write a concise report.".to_string(),
        version: 3,
        is_builtin: false,
        created_at: 1,
        updated_at: 1,
    }
}

pub(super) fn sample_corpus_message() -> CorpusMessage {
    CorpusMessage {
        item_id: 1,
        source_id: 2,
        external_id: "42".to_string(),
        published_at: 1_700_000_000,
        author: Some("analyst".to_string()),
        content: "Important update from the source".to_string(),
        r#ref: "s2-i1".to_string(),
        item_kind: Some("telegram_message".to_string()),
        source_type: Some("telegram".to_string()),
        source_subtype: Some("channel".to_string()),
        metadata_zstd: None,
    }
}

pub(super) fn sample_resolved_profile() -> ResolvedLlmProfile {
    ResolvedLlmProfile {
        profile_id: "research".to_string(),
        provider: ProviderKind::Gemini,
        default_model: "gemini-2.5-flash".to_string(),
        api_key: "secret-key".to_string().into(),
        base_url: String::new(),
    }
}

pub(super) async fn request_cancel_pool_with_runs() -> SqlitePool {
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

pub(super) async fn insert_cancel_request_run(pool: &SqlitePool, run_id: i64, status: &str) {
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

- [ ] **Step 8: Move scope tests**

Create `src-tauri/src/analysis/report/tests/scope.rs` with explicit imports:

```rust
use super::harness::{sample_prompt_template, sample_resolved_profile};
use super::super::{
    chunk_target_chars_for_model_input_limit, resolve_analysis_telegram_history_scope,
    ReportRunInput, StartAnalysisReportRequest,
};
use crate::analysis::corpus::{
    AnalysisRunPreflight, AnalysisRunPreflightLimits, CorpusLoadRequest, YoutubeCorpusMode,
};
```

Move these tests from the inline module into `scope.rs`, keeping each test body byte-for-byte except for imports and module paths:

- `report_run_input_carries_resolved_profile_snapshot`
- `telegram_history_scope_opt_in_preserves_policy_when_zero_migrated_rows_match`
- `migrated_history_opt_in_rejects_non_telegram_analysis`
- `report_start_request_carries_migrated_history_opt_in_to_corpus_request_shape`
- `chunk_target_chars_are_derived_from_model_input_limit_with_fallback`

- [ ] **Step 9: Move capture test**

Create `src-tauri/src/analysis/report/tests/capture.rs` with explicit imports:

```rust
use super::super::capture_report_corpus;
use crate::analysis::corpus::{CorpusLoadRequest, YoutubeCorpusMode};
```

Move `capture_report_corpus_returns_reloaded_snapshot_before_provider_phases` into `capture.rs`, keeping the test body byte-for-byte except for imports and module paths.

- [ ] **Step 10: Move lifecycle tests**

Create `src-tauri/src/analysis/report/tests/lifecycle.rs` with explicit imports:

```rust
use super::harness::{insert_cancel_request_run, request_cancel_pool_with_runs};
use super::super::{mark_interrupted_analysis_runs, request_analysis_run_cancel_for_pool};
use crate::error::AppErrorKind;
use crate::llm::LlmSchedulerState;
```

Move these tests into `lifecycle.rs`, keeping each test body byte-for-byte except for imports and module paths:

- `interrupted_cleanup_preserves_captured_snapshot_state_marker`
- `request_analysis_run_cancel_missing_run_keeps_not_found_message`
- `request_analysis_run_cancel_completed_run_keeps_conflict_message`
- `request_analysis_run_cancel_running_but_inactive_keeps_conflict_message`

- [ ] **Step 11: Move request tests**

Create `src-tauri/src/analysis/report/tests/requests.rs` with explicit imports:

```rust
use super::harness::{
    sample_chunk_summary, sample_corpus_message, sample_prompt_template, SAMPLE_JSON,
};
use super::super::{
    build_map_request, build_reduce_request, extract_json_payload, parse_chunk_summary,
    ReduceRequestParams,
};
```

Move these tests into `requests.rs`, keeping each test body byte-for-byte except for imports and module paths:

- `extracts_json_with_text_before_and_after`
- `extracts_json_inside_markdown_fence`
- `parse_chunk_summary_ignores_non_json_prefix_with_braces`
- `parse_chunk_summary_rejects_malformed_payload`
- `build_map_request_keeps_run_scoped_request_and_profile`
- `build_reduce_request_keeps_run_scoped_request_and_profile`

- [ ] **Step 12: Move phase tests**

Create `src-tauri/src/analysis/report/tests/phases.rs` with explicit imports:

```rust
use super::harness::sample_chunk_summary;
use super::super::{finish_map_phase, run_analysis_step_with_cancel, ReportRunError};
use crate::llm::LlmRequestError;
use tokio_util::sync::CancellationToken;
```

Move these tests into `phases.rs`, keeping each test body byte-for-byte except for imports and module paths:

- `analysis_step_cancel_wrapper_allows_completed_future`
- `analysis_step_cancel_wrapper_interrupts_pending_future`
- `finish_map_phase_preserves_chunk_order_by_original_index`
- `finish_map_phase_rejects_missing_chunk_before_reduce`
- `finish_map_phase_propagates_map_error_without_starting_reduce`

- [ ] **Step 13: Move preflight tests**

Create `src-tauri/src/analysis/report/tests/preflight.rs` with explicit imports:

```rust
use super::super::validate_report_preflight;
use crate::analysis::corpus::{AnalysisRunPreflight, AnalysisRunPreflightLimits};
use crate::error::AppErrorKind;
```

Move these tests into `preflight.rs`, keeping each test body byte-for-byte except for imports and module paths:

- `validate_report_preflight_rejects_empty_corpus`
- `validate_report_preflight_rejects_oversized_runs`
- `validate_report_preflight_allows_runs_within_limits`

- [ ] **Step 14: Move architecture test**

Create `src-tauri/src/analysis/report/tests/architecture.rs`:

```rust
#[test]
fn analysis_report_workflow_file_has_no_tauri_command_adapters() {
    let source = std::fs::read_to_string("src/analysis/report.rs").expect("read report.rs");
    let command_attribute = ["#[tauri", "::command]"].join("");

    assert!(
        !source.contains(&command_attribute),
        "Analysis report command adapters should live outside src/analysis/report.rs"
    );
}
```

Remove the original copy of this test from the inline module.

- [ ] **Step 15: Replace inline test module in `report.rs`**

After all helpers and tests are moved, replace the entire brace-delimited inline `mod tests` body in `src-tauri/src/analysis/report.rs` with exactly:

```rust
#[cfg(test)]
mod tests;
```

Expected: `report.rs` keeps all production code and contains no inline test helper bodies or test attributes.

- [ ] **Step 16: Run rustfmt or format check**

Run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
```

Expected: pass.

If it fails only because formatting is needed, run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml
```

Then run the format check again:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
```

Expected: pass. After any formatting write, inspect the changed file list before staging so unrelated rustfmt drift does not enter the refactor commit.

- [ ] **Step 17: Run source guard for `report.rs` test wiring**

Run:

```powershell
$lines = Get-Content src-tauri/src/analysis/report.rs
$cfgIndexes = @(
    for ($i = 0; $i -lt $lines.Count; $i++) {
        if ($lines[$i] -match '^\s*#\[cfg\(test\)\]\s*$') { $i }
    }
)
$modIndexes = @(
    for ($i = 0; $i -lt $lines.Count; $i++) {
        if ($lines[$i] -match '^\s*mod tests;\s*$') { $i }
    }
)
if ($cfgIndexes.Count -ne 1 -or $modIndexes.Count -ne 1 -or $modIndexes[0] -ne ($cfgIndexes[0] + 1)) {
    throw "report.rs must contain exactly one adjacent #[cfg(test)] / mod tests; pair"
}
$lines[$cfgIndexes[0]]
$lines[$modIndexes[0]]
```

Expected: exactly two adjacent lines are printed: `#[cfg(test)]` followed by `mod tests;`.

- [ ] **Step 18: Run source guard for removed inline tests and stable production visibility**

Run:

```powershell
$inlineTestMatches = @(rg -n "#\[tokio::test\]|#\[test\]|^mod tests \{|use super::\*|^\s*(pub(\([^)]*\))?\s+)?(async\s+fn|fn) (sample_chunk_summary|sample_prompt_template|sample_corpus_message|sample_resolved_profile|request_cancel_pool_with_runs|insert_cancel_request_run)\b" src-tauri/src/analysis/report.rs)
if ($inlineTestMatches.Count -ne 0) {
    $inlineTestMatches
    throw "inline report test body or helper remains in report.rs"
}
```

Expected: no output and no throw.

Run:

```powershell
$widenedMatches = @(rg -n "^\s*pub(\([^)]*\))?\s+(enum ReportRunError|struct ReportRunInput|fn validate_report_preflight|async fn run_report_pipeline)\b" src-tauri/src/analysis/report.rs)
if ($widenedMatches.Count -ne 0) {
    $widenedMatches
    throw "report production item visibility was widened for tests"
}
```

Expected: no output and no throw.

- [ ] **Step 19: Run source guard for parent request facade import**

Run:

```powershell
$reportSource = Get-Content -Raw src-tauri/src/analysis/report.rs
if ($reportSource -notmatch '(?s)#\[cfg\(test\)\]\s*use self::requests::\{[^}]*\bextract_json_payload\b[^}]*\};') {
    throw "report.rs must expose extract_json_payload to nested tests through a private #[cfg(test)] parent facade import"
}
```

Expected: no throw. This confirms request tests can avoid direct `super::super::requests::extract_json_payload` access.

- [ ] **Step 20: Run source guard for required test files and module wiring**

Run:

```powershell
foreach ($file in @(
    "mod.rs",
    "harness.rs",
    "scope.rs",
    "capture.rs",
    "lifecycle.rs",
    "requests.rs",
    "phases.rs",
    "preflight.rs",
    "architecture.rs"
)) {
    $path = "src-tauri/src/analysis/report/tests/$file"
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
        throw "missing report test module: $path"
    }
}
```

Expected: no throw.

Run:

```powershell
foreach ($module in @("architecture", "capture", "harness", "lifecycle", "phases", "preflight", "requests", "scope")) {
    rg -n "^mod $module;$" src-tauri/src/analysis/report/tests/mod.rs
    if ($LASTEXITCODE -ne 0) {
        throw "missing report test module declaration: $module"
    }
}
```

Expected: every module declaration is printed independently.

Run:

```powershell
$testModBodyMatches = @(rg -n "#\[tokio::test\]|#\[test\]|^\s*(pub(\([^)]*\))?\s+)?(async\s+fn|fn)\b|use super::|use crate::" src-tauri/src/analysis/report/tests/mod.rs)
if ($testModBodyMatches.Count -ne 0) {
    $testModBodyMatches
    throw "report tests/mod.rs must contain module declarations only"
}
```

Expected: no output and no throw.

- [ ] **Step 21: Run source guard for imports and helper visibility**

Run:

```powershell
$privateChildMatches = @(rg -n "super::super::(capture|lifecycle|phases|requests)::|crate::analysis::report::(capture|lifecycle|phases|requests)::" src-tauri/src/analysis/report/tests)
if ($privateChildMatches.Count -ne 0) {
    $privateChildMatches
    throw "report tests must use parent facade access, not private child module paths"
}
```

Expected: no output and no throw.

Run:

```powershell
$globImportMatches = @(rg -n "use\s+super::super::\*|use\s+crate::.*::\*" src-tauri/src/analysis/report/tests)
if ($globImportMatches.Count -ne 0) {
    $globImportMatches
    throw "report test modules must use explicit imports, not parent or crate glob imports"
}
```

Expected: no output and no throw.

Run:

```powershell
$publicish = @(rg -n "^\s*pub(\([^)]*\))?\s+(const|async fn|fn)\b" src-tauri/src/analysis/report/tests)
$badPublicish = @(
    $publicish | Where-Object {
        $_ -notmatch 'report/tests/harness\.rs:\d+:\s*pub\(super\)\s+const\s+SAMPLE_JSON\b' -and
        $_ -notmatch 'report/tests/harness\.rs:\d+:\s*pub\(super\)\s+fn\s+sample_chunk_summary\b' -and
        $_ -notmatch 'report/tests/harness\.rs:\d+:\s*pub\(super\)\s+fn\s+sample_prompt_template\b' -and
        $_ -notmatch 'report/tests/harness\.rs:\d+:\s*pub\(super\)\s+fn\s+sample_corpus_message\b' -and
        $_ -notmatch 'report/tests/harness\.rs:\d+:\s*pub\(super\)\s+fn\s+sample_resolved_profile\b' -and
        $_ -notmatch 'report/tests/harness\.rs:\d+:\s*pub\(super\)\s+async\s+fn\s+request_cancel_pool_with_runs\b' -and
        $_ -notmatch 'report/tests/harness\.rs:\d+:\s*pub\(super\)\s+async\s+fn\s+insert_cancel_request_run\b'
    }
)
if ($badPublicish.Count -ne 0) {
    $badPublicish
    throw "report test helper visibility widened outside the approved harness surface"
}
```

Expected: no output and no throw.

- [ ] **Step 22: Run source guard for required moved tests**

Run:

```powershell
$requiredTests = @{
    "scope.rs" = @(
        "report_run_input_carries_resolved_profile_snapshot",
        "telegram_history_scope_opt_in_preserves_policy_when_zero_migrated_rows_match",
        "migrated_history_opt_in_rejects_non_telegram_analysis",
        "report_start_request_carries_migrated_history_opt_in_to_corpus_request_shape",
        "chunk_target_chars_are_derived_from_model_input_limit_with_fallback"
    )
    "capture.rs" = @("capture_report_corpus_returns_reloaded_snapshot_before_provider_phases")
    "lifecycle.rs" = @(
        "interrupted_cleanup_preserves_captured_snapshot_state_marker",
        "request_analysis_run_cancel_missing_run_keeps_not_found_message",
        "request_analysis_run_cancel_completed_run_keeps_conflict_message",
        "request_analysis_run_cancel_running_but_inactive_keeps_conflict_message"
    )
    "requests.rs" = @(
        "extracts_json_with_text_before_and_after",
        "extracts_json_inside_markdown_fence",
        "parse_chunk_summary_ignores_non_json_prefix_with_braces",
        "parse_chunk_summary_rejects_malformed_payload",
        "build_map_request_keeps_run_scoped_request_and_profile",
        "build_reduce_request_keeps_run_scoped_request_and_profile"
    )
    "phases.rs" = @(
        "analysis_step_cancel_wrapper_allows_completed_future",
        "analysis_step_cancel_wrapper_interrupts_pending_future",
        "finish_map_phase_preserves_chunk_order_by_original_index",
        "finish_map_phase_rejects_missing_chunk_before_reduce",
        "finish_map_phase_propagates_map_error_without_starting_reduce"
    )
    "preflight.rs" = @(
        "validate_report_preflight_rejects_empty_corpus",
        "validate_report_preflight_rejects_oversized_runs",
        "validate_report_preflight_allows_runs_within_limits"
    )
    "architecture.rs" = @("analysis_report_workflow_file_has_no_tauri_command_adapters")
}
foreach ($entry in $requiredTests.GetEnumerator()) {
    foreach ($testName in $entry.Value) {
        $path = "src-tauri/src/analysis/report/tests/$($entry.Key)"
        $content = Get-Content -Raw $path
        $pattern = "(?s)#\[(tokio::test|test)\]\s*(async\s+fn|fn) $([regex]::Escape($testName))\b"
        if ($content -notmatch $pattern) {
            throw "missing report test $testName with test attribute in $path"
        }
    }
}
```

Expected: every required test is checked independently; the command throws on the first missing test.

- [ ] **Step 23: Run source guard for assertion markers**

Run:

```powershell
$requiredMarkers = @{
    "scope.rs" = @("ANALYSIS_TELEGRAM_HISTORY_SCOPE_CURRENT_PLUS_MIGRATED", "include_migrated_history")
    "preflight.rs" = @("No synced source documents were found for the selected analysis scope and period", "Analysis scope is too large")
    "lifecycle.rs" = @("Analysis run {run_id} not found", "Analysis run {run_id} is not queued or running", "Analysis run {run_id} is no longer active", "2026-05-18T10:00:00Z")
    "requests.rs" = @("Failed to parse chunk summary JSON", "analysis-map-55-2-", "analysis-reduce-77-")
    "phases.rs" = @("Some chunk summaries were not collected", "Analysis run cancelled.")
    "architecture.rs" = @("Analysis report command adapters should live outside src/analysis/report.rs")
}
foreach ($entry in $requiredMarkers.GetEnumerator()) {
    foreach ($marker in $entry.Value) {
        $path = "src-tauri/src/analysis/report/tests/$($entry.Key)"
        rg -n -F $marker $path
        if ($LASTEXITCODE -ne 0) {
            throw "missing report assertion marker '$marker' in $path"
        }
    }
}
```

Expected: every moved assertion marker is present in the thematic test modules.

- [ ] **Step 24: Run focused report module test slices**

Run each command separately:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::scope::
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::capture::
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::lifecycle::
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::requests::
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::phases::
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::preflight::
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::architecture::
```

Expected for each focused module slice: pass in the default dev test profile and not a green `0 tests` run.

- [ ] **Step 25: Verify Cargo test inventory**

Run:

```powershell
$testList = cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests:: -- --list
if ($LASTEXITCODE -ne 0) {
    throw "report test inventory command failed"
}
foreach ($testPath in @(
    "analysis::report::tests::scope::report_run_input_carries_resolved_profile_snapshot",
    "analysis::report::tests::scope::telegram_history_scope_opt_in_preserves_policy_when_zero_migrated_rows_match",
    "analysis::report::tests::scope::migrated_history_opt_in_rejects_non_telegram_analysis",
    "analysis::report::tests::scope::report_start_request_carries_migrated_history_opt_in_to_corpus_request_shape",
    "analysis::report::tests::scope::chunk_target_chars_are_derived_from_model_input_limit_with_fallback",
    "analysis::report::tests::capture::capture_report_corpus_returns_reloaded_snapshot_before_provider_phases",
    "analysis::report::tests::lifecycle::interrupted_cleanup_preserves_captured_snapshot_state_marker",
    "analysis::report::tests::lifecycle::request_analysis_run_cancel_missing_run_keeps_not_found_message",
    "analysis::report::tests::lifecycle::request_analysis_run_cancel_completed_run_keeps_conflict_message",
    "analysis::report::tests::lifecycle::request_analysis_run_cancel_running_but_inactive_keeps_conflict_message",
    "analysis::report::tests::requests::extracts_json_with_text_before_and_after",
    "analysis::report::tests::requests::extracts_json_inside_markdown_fence",
    "analysis::report::tests::requests::parse_chunk_summary_ignores_non_json_prefix_with_braces",
    "analysis::report::tests::requests::parse_chunk_summary_rejects_malformed_payload",
    "analysis::report::tests::requests::build_map_request_keeps_run_scoped_request_and_profile",
    "analysis::report::tests::requests::build_reduce_request_keeps_run_scoped_request_and_profile",
    "analysis::report::tests::phases::analysis_step_cancel_wrapper_allows_completed_future",
    "analysis::report::tests::phases::analysis_step_cancel_wrapper_interrupts_pending_future",
    "analysis::report::tests::phases::finish_map_phase_preserves_chunk_order_by_original_index",
    "analysis::report::tests::phases::finish_map_phase_rejects_missing_chunk_before_reduce",
    "analysis::report::tests::phases::finish_map_phase_propagates_map_error_without_starting_reduce",
    "analysis::report::tests::preflight::validate_report_preflight_rejects_empty_corpus",
    "analysis::report::tests::preflight::validate_report_preflight_rejects_oversized_runs",
    "analysis::report::tests::preflight::validate_report_preflight_allows_runs_within_limits",
    "analysis::report::tests::architecture::analysis_report_workflow_file_has_no_tauri_command_adapters"
)) {
    if ($testList -notmatch [regex]::Escape($testPath)) {
        throw "report test is missing from cargo test --list output: $testPath"
    }
}
```

Expected: every moved test path appears in Cargo's test inventory.

- [ ] **Step 26: Run full report test slice, compile check, and format check**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::
```

Expected: pass in the default dev test profile and not a green `0 tests` run.

Run:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
```

Expected: pass.

Run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
```

Expected: pass.

- [ ] **Step 27: Inspect implementation diff and status after formatting**

Run:

```powershell
git status --short --untracked-files=all
```

Expected implementation-owned status entries are limited to:

```text
 M	src-tauri/src/analysis/report.rs
??	src-tauri/src/analysis/report/tests/mod.rs
??	src-tauri/src/analysis/report/tests/harness.rs
??	src-tauri/src/analysis/report/tests/scope.rs
??	src-tauri/src/analysis/report/tests/capture.rs
??	src-tauri/src/analysis/report/tests/lifecycle.rs
??	src-tauri/src/analysis/report/tests/requests.rs
??	src-tauri/src/analysis/report/tests/phases.rs
??	src-tauri/src/analysis/report/tests/preflight.rs
??	src-tauri/src/analysis/report/tests/architecture.rs
```

Run:

```powershell
$pointerPath = Join-Path $env:TEMP "extractum-analysis-report-tests-refactor-status-pointer.txt"
$preEditStatusPath = Get-Content -LiteralPath $pointerPath
$afterPath = Join-Path $env:TEMP "analysis-report-tests-refactor-status-after.txt"
git status --short --untracked-files=all | Set-Content -LiteralPath $afterPath
Compare-Object (Get-Content -LiteralPath $preEditStatusPath) (Get-Content -LiteralPath $afterPath)
```

Expected: differences are only the intended implementation-owned files. Pre-existing unrelated entries must not be modified or staged.

- [ ] **Step 28: Stage implementation files**

Run:

```powershell
git add -- src-tauri/src/analysis/report.rs src-tauri/src/analysis/report/tests/mod.rs src-tauri/src/analysis/report/tests/harness.rs src-tauri/src/analysis/report/tests/scope.rs src-tauri/src/analysis/report/tests/capture.rs src-tauri/src/analysis/report/tests/lifecycle.rs src-tauri/src/analysis/report/tests/requests.rs src-tauri/src/analysis/report/tests/phases.rs src-tauri/src/analysis/report/tests/preflight.rs src-tauri/src/analysis/report/tests/architecture.rs
```

Expected: only the report test split files are staged.

- [ ] **Step 29: Verify staged diff**

Run:

```powershell
git diff --cached --check
```

Expected: no whitespace errors.

Run:

```powershell
git diff --cached --name-status
```

Expected staged files:

```text
M	src-tauri/src/analysis/report.rs
A	src-tauri/src/analysis/report/tests/mod.rs
A	src-tauri/src/analysis/report/tests/harness.rs
A	src-tauri/src/analysis/report/tests/scope.rs
A	src-tauri/src/analysis/report/tests/capture.rs
A	src-tauri/src/analysis/report/tests/lifecycle.rs
A	src-tauri/src/analysis/report/tests/requests.rs
A	src-tauri/src/analysis/report/tests/phases.rs
A	src-tauri/src/analysis/report/tests/preflight.rs
A	src-tauri/src/analysis/report/tests/architecture.rs
```

Run:

```powershell
git status --short --untracked-files=all
```

Expected: implementation files are staged. Pre-existing unrelated files remain unstaged.

- [ ] **Step 30: Commit the Rust refactor**

Run:

```powershell
git commit -m "refactor: split analysis report tests"
```

Expected: commit succeeds with only the staged report test split files.

- [ ] **Step 31: Record post-commit status**

Run:

```powershell
git status --short --untracked-files=all
```

Expected: no dirty implementation files remain. Pre-existing unrelated files may remain if they were present before this task.

## Final Verification Checklist

Before reporting the implementation complete, confirm the execution log includes:

- [ ] pre-edit `git status --short --untracked-files=all` captured;
- [ ] target-file baseline proved `report.rs` was clean and `report/tests/` did not contain pre-existing tracked or untracked work without an explicit baseline decision;
- [ ] baseline `cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::` passed before editing and was not a green `0 tests` run;
- [ ] baseline `cargo check --manifest-path src-tauri/Cargo.toml --all-targets` passed before editing;
- [ ] source guards proved `report.rs` contains exactly one adjacent `#[cfg(test)]` / `mod tests;` pair;
- [ ] source guards proved `report.rs` no longer contains inline test body helpers or test attributes;
- [ ] source guards proved production item visibility was not widened;
- [ ] source guards proved `extract_json_payload` is available through a private parent `#[cfg(test)]` facade import, not through direct child-module test imports;
- [ ] source guards proved all required test files exist as files, not directories;
- [ ] source guards proved `tests/mod.rs` declares only modules and no test/helper logic;
- [ ] source guards proved tests do not import private production child modules directly;
- [ ] source guards proved report test modules do not use parent or crate glob imports;
- [ ] source guards proved shared helper visibility is limited to the approved `pub(super)` harness surface;
- [ ] source guards proved every required moved test has `#[test]` or `#[tokio::test]` immediately before its function;
- [ ] source guards proved assertion markers moved to the expected thematic modules;
- [ ] every focused report module test command passed and was not a green `0 tests` run;
- [ ] `cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests:: -- --list` contained every expected moved test path;
- [ ] full `cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::` passed and was not a green `0 tests` run;
- [ ] `cargo check --manifest-path src-tauri/Cargo.toml --all-targets` passed;
- [ ] `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` passed;
- [ ] staged diff contained only the expected report test split files;
- [ ] post-commit `git status --short --untracked-files=all` has no dirty implementation files.
