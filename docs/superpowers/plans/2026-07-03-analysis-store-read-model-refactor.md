# Analysis Store Read Model Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract analysis run read-model mapping and list/detail query logic from `src-tauri/src/analysis/store.rs` into `src-tauri/src/analysis/store/read_model.rs` without changing behavior or external call paths.

**Architecture:** `store.rs` remains the `analysis::store` facade and keeps write, snapshot, status, delete, prompt-template, source-group, duplicate-run, and test ownership. A private child module `read_model.rs` owns read-only run list/detail helpers and exposes only the existing `pub(crate)` read-model API through a facade re-export in `store.rs`. Existing consumers keep importing from `analysis::store`.

**Tech Stack:** Rust, Tauri backend, SQLx SQLite, Cargo tests run with `--manifest-path src-tauri/Cargo.toml` from the repository root.

## Global Constraints

- Behavioral refactor edits are limited to `src-tauri/src/analysis/store.rs` and `src-tauri/src/analysis/store/read_model.rs`.
- Do not change SQL, DTO mappings, filter semantics, date parsing, wildcard escaping, snapshot-state logic, database schema, migrations, frontend code, Tauri command payloads, or event payloads.
- Do not move prompt-template initialization, source existence checks, source-group loading, duplicate-run lookup, analysis run insertion, snapshot validation/capture/persistence/failure marking, run status mutation, saved-run deletion, or store tests in this slice.
- Keep `src-tauri/src/analysis/store/read_model.rs` private via `mod read_model;`; do not use `pub mod read_model`, `pub(crate) mod read_model`, or root re-exports outside `store.rs`.
- Preserve the `analysis::store` facade API for current consumers in `analysis/mod.rs`, `analysis/chat.rs`, `analysis/report/lifecycle.rs`, and `analysis/fixtures.rs`.
- Run Cargo commands from the repository root using `--manifest-path src-tauri/Cargo.toml`.
- Run post-change test commands separately, or use a stopping wrapper that checks `$LASTEXITCODE` after every native command.
- Target files must be clean before editing. If `src-tauri/src/analysis/store.rs` or `src-tauri/src/analysis/store/read_model.rs` has pre-existing tracked, staged, or untracked work, stop and make a separate baseline commit before starting this refactor. This plan uses full-file staging for target files.
- Do not stage unrelated dirty files such as local tool settings. Compare final status against the captured pre-edit status.

---

## File Structure

- Create `src-tauri/src/analysis/store/read_model.rs`: owns read-model imports, private mapping/filter/query helpers, `AnalysisRunListFilters`, `list_analysis_run_summaries`, `fetch_run_row`, and `resolve_run_scope_label`.
- Modify `src-tauri/src/analysis/store.rs`: declares private `mod read_model;`, re-exports the existing public read-model API, removes moved definitions and moved-only imports, and keeps all current tests.

---

### Task 1: Baseline And Worktree Guard

**Files:**
- Inspect: `src-tauri/src/analysis/store.rs`
- Inspect if present: `src-tauri/src/analysis/store/read_model.rs`

**Interfaces:**
- Consumes: approved design `docs/superpowers/specs/2026-07-03-analysis-store-read-model-refactor-design.md`.
- Produces: captured pre-edit status and baseline test evidence before extraction.

- [x] **Step 1: Capture the current worktree status**

Run:

```powershell
git status --short --untracked-files=all
```

Expected: record the exact output. Unrelated files may exist, but target files must be accounted for before editing.

- [x] **Step 2: Prove tracked target files are clean**

Run:

```powershell
git diff -- src-tauri/src/analysis/store.rs
git diff --cached -- src-tauri/src/analysis/store.rs
```

Expected: no target-file diff. If `src-tauri/src/analysis/store.rs` is modified or staged before this refactor starts, stop here and make a separate baseline commit before editing.

- [x] **Step 3: Inspect a pre-existing `read_model.rs` directly if present**

Run:

```powershell
if (Test-Path -LiteralPath 'src-tauri/src/analysis/store/read_model.rs') {
    git status --short --untracked-files=all -- src-tauri/src/analysis/store/read_model.rs
    Get-Item -LiteralPath 'src-tauri/src/analysis/store/read_model.rs' | Select-Object FullName, Length, LastWriteTime
    Get-FileHash -Algorithm SHA256 -LiteralPath 'src-tauri/src/analysis/store/read_model.rs'
    Get-Content -Raw -LiteralPath 'src-tauri/src/analysis/store/read_model.rs'
}
```

Expected: no output if the file does not exist. If it exists as tracked, staged, modified, or untracked work before this refactor starts, stop here and make a separate baseline commit before editing.

- [x] **Step 4: Save a unique pre-edit status snapshot**

Run:

```powershell
$env:ANALYSIS_STORE_READ_MODEL_STATUS_TAG = "{0}-{1}" -f (Get-Date -Format 'yyyyMMddHHmmss'), $PID
$statusPointerPath = Join-Path $env:TEMP 'analysis-store-read-model-latest-status-paths.txt'
$preEditStatusPath = Join-Path $env:TEMP "analysis-store-read-model-$env:ANALYSIS_STORE_READ_MODEL_STATUS_TAG-pre-edit-status.txt"
git status --short --untracked-files=all | Set-Content -Encoding utf8 -LiteralPath $preEditStatusPath
"ANALYSIS_STORE_READ_MODEL_STATUS_TAG=$env:ANALYSIS_STORE_READ_MODEL_STATUS_TAG" | Set-Content -Encoding utf8 -LiteralPath $statusPointerPath
"PRE_EDIT_STATUS_PATH=$preEditStatusPath" | Add-Content -Encoding utf8 -LiteralPath $statusPointerPath
Get-Content -Raw -LiteralPath $preEditStatusPath
Get-Content -Raw -LiteralPath $statusPointerPath
```

Expected: the pre-edit status file contains the same relevant baseline status from Step 1, and the pointer file records the tag and status path for later PowerShell sessions.

- [x] **Step 5: Run baseline focused store list test**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::list_analysis_run_summaries
```

Expected: PASS and not a green `0 tests` run.

- [x] **Step 6: Run baseline summary mapping test**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::map_run_summary
```

Expected: PASS and not a green `0 tests` run.

- [x] **Step 7: Run baseline detail mapping test**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::map_run_detail
```

Expected: PASS and not a green `0 tests` run.

- [x] **Step 8: Run baseline scope-label test**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::resolve_run_scope_label
```

Expected: PASS and not a green `0 tests` run.

- [x] **Step 9: Run baseline full store test slice**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::
```

Expected: PASS and not a green `0 tests` run.

- [x] **Step 10: Run baseline fixture consumer slice**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::
```

Expected: PASS and not a green `0 tests` run.

---

### Task 2: Extract Read Model Module

**Files:**
- Create: `src-tauri/src/analysis/store/read_model.rs`
- Modify: `src-tauri/src/analysis/store.rs`

**Interfaces:**
- Consumes from `store.rs`: `AnalysisRunRow`, `AnalysisRunDetail`, `AnalysisRunSummary`, `AnalysisSnapshotState`, analysis status/scope constants, `AppError`, `AppResult`, and `ymd_to_unix_midnight`.
- Produces through `analysis::store`: `AnalysisRunListFilters`, `map_run_summary`, `map_run_detail`, `list_analysis_run_summaries`, `fetch_run_row`, and `resolve_run_scope_label`.

- [ ] **Step 1: Add the private child module and facade re-export in `store.rs`**

Add these lines near the top of `src-tauri/src/analysis/store.rs`, after imports and before the first function:

```rust
mod read_model;

pub(crate) use self::read_model::{
    fetch_run_row, list_analysis_run_summaries, map_run_detail, map_run_summary,
    resolve_run_scope_label, AnalysisRunListFilters,
};
```

Expected: `read_model` is private. Do not write `pub mod read_model;` or `pub(crate) mod read_model;`.

- [ ] **Step 2: Create `read_model.rs` with the moved imports**

Create `src-tauri/src/analysis/store/read_model.rs` with this import header before moving the functions:

```rust
use sqlx::{Pool, QueryBuilder, Sqlite};

use super::super::models::{
    AnalysisRunDetail, AnalysisRunRow, AnalysisRunSummary, AnalysisSnapshotState,
};
use super::super::{
    ANALYSIS_SCOPE_TYPE_PROJECT, ANALYSIS_SCOPE_TYPE_SOURCE_GROUP, ANALYSIS_STATUS_CANCELLED,
    ANALYSIS_STATUS_COMPLETED, ANALYSIS_STATUS_FAILED, ANALYSIS_STATUS_QUEUED,
    ANALYSIS_STATUS_RUNNING,
};
use crate::error::{AppError, AppResult};
use crate::time::ymd_to_unix_midnight;
```

Expected: `read_model.rs` does not import prompt-template, source-group, snapshot persistence, status mutation, or compression items.

- [ ] **Step 3: Move read-model helpers into `read_model.rs`**

Move these exact items from `store.rs` into `read_model.rs`, preserving bodies and order:

- `fn resolve_run_scope_label_parts(scope_type: &str, source_id: Option<i64>, source_title: Option<&str>, source_group_id: Option<i64>, source_group_name: Option<&str>, project_id: Option<i64>, project_name: Option<&str>, scope_label_snapshot: Option<&str>) -> String`
- `fn resolve_run_row_scope_label(row: &AnalysisRunRow) -> String`
- `fn compute_snapshot_state(row: &AnalysisRunRow) -> Option<AnalysisSnapshotState>`
- `pub(crate) fn map_run_summary(row: AnalysisRunRow) -> AnalysisRunSummary`
- `pub(crate) fn map_run_detail(row: AnalysisRunRow) -> AnalysisRunDetail`
- `#[derive(Debug, Clone, Default)] pub(crate) struct AnalysisRunListFilters` with all eleven current fields
- `const ANALYSIS_RUN_LIST_SELECT: &str`
- `const RUN_QUERY_FIELDS: [&str; 9]`
- `fn trimmed_filter(value: Option<String>) -> Option<String>`
- `fn escaped_like_contains(value: &str) -> String`
- `fn parse_yyyy_mm_dd_midnight(value: &str) -> Option<i64>`
- `fn parse_yyyy_mm_dd_day_end(value: &str) -> Option<i64>`
- `fn push_like_predicate(query: &mut QueryBuilder<'_, Sqlite>, expression: &str, value: &str)`
- `fn push_search_term_predicate(query: &mut QueryBuilder<'_, Sqlite>, term: &str)`
- `pub(crate) async fn list_analysis_run_summaries(pool: &Pool<Sqlite>, filters: AnalysisRunListFilters) -> AppResult<Vec<AnalysisRunSummary>>`
- `pub(crate) async fn fetch_run_row(pool: &Pool<Sqlite>, run_id: i64) -> AppResult<Option<AnalysisRunRow>>`
- `pub(crate) fn resolve_run_scope_label(run: &AnalysisRunDetail) -> String`

Implementation detail: this is not one contiguous file range. First move the contiguous range from `fn resolve_run_scope_label_parts(` through the closing brace of `pub(crate) async fn fetch_run_row(pool: &Pool<Sqlite>, run_id: i64) -> AppResult<Option<AnalysisRunRow>>`. Then move `pub(crate) fn resolve_run_scope_label(run: &AnalysisRunDetail) -> String` separately from its later location. Leave `fetch_prompt_template`, `fetch_source_group`, and every non-read-model function in `store.rs`.

- [ ] **Step 4: Preserve field-level visibility for `AnalysisRunListFilters`**

Ensure the moved struct remains exactly field-accessible to crate consumers:

```rust
#[derive(Debug, Clone, Default)]
pub(crate) struct AnalysisRunListFilters {
    pub(crate) source_id: Option<i64>,
    pub(crate) source_group_id: Option<i64>,
    pub(crate) project_id: Option<i64>,
    pub(crate) limit: i64,
    pub(crate) query: Option<String>,
    pub(crate) status: Option<String>,
    pub(crate) provider: Option<String>,
    pub(crate) model: Option<String>,
    pub(crate) template: Option<String>,
    pub(crate) date_from: Option<String>,
    pub(crate) date_to: Option<String>,
}
```

Expected: none of these fields become private, `pub(super)`, or renamed.

- [ ] **Step 5: Keep private helpers private**

Verify these moved helpers have no visibility modifier in `read_model.rs`:

- `fn resolve_run_scope_label_parts`
- `fn resolve_run_row_scope_label`
- `fn compute_snapshot_state`
- `const ANALYSIS_RUN_LIST_SELECT`
- `const RUN_QUERY_FIELDS`
- `fn trimmed_filter`
- `fn escaped_like_contains`
- `fn parse_yyyy_mm_dd_midnight`
- `fn parse_yyyy_mm_dd_day_end`
- `fn push_like_predicate`
- `fn push_search_term_predicate`

Expected: only the six facade API items are `pub(crate)`.

- [ ] **Step 6: Trim moved-only imports from `store.rs`**

Update the top of `src-tauri/src/analysis/store.rs` so moved-only imports are gone while remaining store and test code still compile. The production import set should no longer need `QueryBuilder`, `AnalysisRunSummary`, `AnalysisSnapshotState`, or `ymd_to_unix_midnight`.

The opening imports should be shaped like this unless later compile feedback proves another remaining item is needed:

```rust
use sqlx::{Pool, Sqlite};

use super::corpus::YoutubeCorpusMode;
use super::models::{
    AnalysisPromptTemplate, AnalysisSourceGroup, AnalysisSourceGroupMember, AnalysisSourceGroupRow,
    CorpusMessage, StoredRunSnapshotRow,
};
use super::{
    default_report_template_body, now_secs, ANALYSIS_RUN_TYPE_REPORT,
    ANALYSIS_SCOPE_TYPE_SOURCE_GROUP, ANALYSIS_STATUS_QUEUED, ANALYSIS_STATUS_RUNNING,
    DEFAULT_REPORT_TEMPLATE_NAME, TEMPLATE_KIND_REPORT,
};
use crate::compression::{compress_text, decompress_text};
use crate::error::{internal_error, AppError, AppResult};
```

Expected: `store.rs` production imports no longer include `AnalysisRunDetail`, `AnalysisRunRow`, `AnalysisRunSummary`, `AnalysisSnapshotState`, `QueryBuilder`, or `ymd_to_unix_midnight`. If `store.rs` still uses `ANALYSIS_STATUS_CANCELLED`, `ANALYSIS_STATUS_COMPLETED`, or `ANALYSIS_STATUS_FAILED` only in tests through fully qualified `crate::analysis` paths, do not keep them in the production `use super` block.

- [ ] **Step 7: Leave the inline store tests in `store.rs`**

Keep the existing test module in `src-tauri/src/analysis/store.rs`. Its `use super` block should continue importing the facade names from `store.rs`:

```rust
use super::{
    capture_run_snapshot, delete_saved_run, ensure_sources_exist, fetch_prompt_template,
    list_analysis_run_summaries, map_run_detail, map_run_summary, mark_run_capture_failed,
    resolve_run_scope_label, sanitize_provider_error, sanitize_snapshot_error, set_run_status,
    AnalysisRunListFilters,
};
```

Expected: tests do not import from `super::read_model` directly. This keeps the test coverage on the facade contract.

- [ ] **Step 8: Run formatter**

Run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml
```

Expected: formatting completes. If rustfmt touches unrelated Rust files, inspect `git status --short --untracked-files=all` and keep unrelated drift out of the refactor commit.

---

### Task 3: Source Guards And Verification

**Files:**
- Verify: `src-tauri/src/analysis/store.rs`
- Verify: `src-tauri/src/analysis/store/read_model.rs`

**Interfaces:**
- Consumes: extracted facade from Task 2.
- Produces: proof that moved helpers left `store.rs`, facade exports are intact, fields remain accessible, and behavior-focused tests pass.

- [ ] **Step 1: Confirm moved definitions no longer live in `store.rs`**

Run:

```powershell
rg -n "^\s*(pub\([^)]*\)\s+|pub\s+)?(fn|async fn|struct) (resolve_run_scope_label_parts|resolve_run_row_scope_label|compute_snapshot_state|map_run_summary|map_run_detail|AnalysisRunListFilters|trimmed_filter|escaped_like_contains|parse_yyyy_mm_dd_midnight|parse_yyyy_mm_dd_day_end|push_like_predicate|push_search_term_predicate|list_analysis_run_summaries|fetch_run_row|resolve_run_scope_label)\b|^\s*(pub\([^)]*\)\s+|pub\s+)?const (ANALYSIS_RUN_LIST_SELECT|RUN_QUERY_FIELDS)\b" src-tauri/src/analysis/store.rs
```

Expected: no matches. `rg` exit code `1` is expected for this no-match guard.

- [ ] **Step 2: Confirm `store.rs` has a private module declaration**

Run:

```powershell
rg -n "^mod read_model;" src-tauri/src/analysis/store.rs
rg -n "^pub.*mod read_model" src-tauri/src/analysis/store.rs
```

Expected: first command prints exactly one match. Second command has no matches; `rg` exit code `1` is expected.

- [ ] **Step 3: Confirm facade re-export exists and contains every public read-model name**

Run:

```powershell
rg -n "^pub\(crate\) use self::read_model::" src-tauri/src/analysis/store.rs
$storeFacade = Get-Content -Raw -LiteralPath 'src-tauri/src/analysis/store.rs'
$readModelReExport = [regex]::Match($storeFacade, "pub\(crate\) use self::read_model::\{(?<block>[\s\S]*?)\};")
if (-not $readModelReExport.Success) {
    throw "missing read_model facade re-export block"
}
foreach ($name in @('AnalysisRunListFilters', 'fetch_run_row', 'list_analysis_run_summaries', 'map_run_detail', 'map_run_summary', 'resolve_run_scope_label')) {
    if ($readModelReExport.Groups['block'].Value -notmatch ("\b" + [regex]::Escape($name) + "\b")) {
        throw "missing read_model facade re-export: $name"
    }
}
```

Expected: the `rg` command prints the facade anchor, and the loop completes without throwing.

- [ ] **Step 4: Confirm moved public API exists in `read_model.rs`**

Run:

```powershell
rg -n "^pub\(crate\) (fn|async fn|struct) (map_run_summary|map_run_detail|AnalysisRunListFilters|list_analysis_run_summaries|fetch_run_row|resolve_run_scope_label)\b" src-tauri/src/analysis/store/read_model.rs
```

Expected: six matches, one for each public read-model API item.

- [ ] **Step 5: Confirm `AnalysisRunListFilters` fields remain `pub(crate)`**

Run:

```powershell
rg -n "^\s+pub\(crate\) (source_id|source_group_id|project_id|limit|query|status|provider|model|template|date_from|date_to):" src-tauri/src/analysis/store/read_model.rs
```

Expected: eleven matches, one for each filter field.

- [ ] **Step 6: Confirm read-model literals and SQL markers stayed in `read_model.rs`**

Run:

```powershell
rg -n '"Pass only one of source_id, source_group_id, or project_id"|queued_running|ESCAPE|ORDER BY runs.created_at DESC LIMIT|COALESCE\(snapshot_counts.snapshot_message_count, 0\)' src-tauri/src/analysis/store/read_model.rs
```

Expected: all read-model filter/query behavior markers are present in `read_model.rs`.

- [ ] **Step 7: Confirm tests still exercise the facade, not the private module**

Run:

```powershell
$storeTests = [regex]::Match((Get-Content -Raw -LiteralPath 'src-tauri/src/analysis/store.rs'), "(?s)#\[cfg\(test\)\]\s*mod tests \{.*\z").Value
if ($storeTests -match "super::read_model|read_model::") {
    throw "store tests must use the store facade, not read_model directly"
}
```

Expected: command completes without throwing. Tests should keep using the `store.rs` facade imports from `super`.

- [ ] **Step 8: Run focused store list test**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::list_analysis_run_summaries
```

Expected: PASS and not a green `0 tests` run.

- [ ] **Step 9: Run summary mapping test**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::map_run_summary
```

Expected: PASS and not a green `0 tests` run.

- [ ] **Step 10: Run detail mapping test**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::map_run_detail
```

Expected: PASS and not a green `0 tests` run.

- [ ] **Step 11: Run scope-label test**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::resolve_run_scope_label
```

Expected: PASS and not a green `0 tests` run.

- [ ] **Step 12: Run full store test slice**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::
```

Expected: PASS and not a green `0 tests` run.

- [ ] **Step 13: Run fixture consumer behavior slice**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::
```

Expected: PASS and not a green `0 tests` run.

- [ ] **Step 14: Run all-target compile coverage**

Run:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
```

Expected: PASS. Existing warnings outside touched files may remain; new warnings mentioning `src-tauri/src/analysis/store.rs` or `src-tauri/src/analysis/store/read_model.rs` are not acceptable.

- [ ] **Step 15: Run format check**

Run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
```

Expected: PASS with no diff output.

---

### Task 4: Commit And Final Hygiene

**Files:**
- Stage: `src-tauri/src/analysis/store.rs`
- Stage: `src-tauri/src/analysis/store/read_model.rs`

**Interfaces:**
- Consumes: verified extraction from Task 3.
- Produces: one implementation commit with only the expected Rust files.

- [ ] **Step 1: Inspect implementation diff**

Run:

```powershell
git diff -- src-tauri/src/analysis/store.rs src-tauri/src/analysis/store/read_model.rs
```

Expected: `store.rs` loses only the moved read-model cluster and moved-only imports, gains `mod read_model;` plus the facade re-export, and `read_model.rs` contains the moved code.

- [ ] **Step 2: Check for whitespace errors before staging**

Run:

```powershell
git diff --check -- src-tauri/src/analysis/store.rs src-tauri/src/analysis/store/read_model.rs
```

Expected: no output and exit code `0`.

- [ ] **Step 3: Inspect status before staging**

Run:

```powershell
git status --short --untracked-files=all
```

Expected: only expected target-file changes plus any recorded pre-existing unrelated files. Do not stage unrelated files.

- [ ] **Step 4: Stage only implementation-owned files**

Run:

```powershell
git add -- src-tauri/src/analysis/store.rs src-tauri/src/analysis/store/read_model.rs
```

Expected: only the two Rust files are staged.

- [ ] **Step 5: Verify staged diff**

Run:

```powershell
git diff --cached --stat
```

Expected: cached stat lists only `src-tauri/src/analysis/store.rs` and `src-tauri/src/analysis/store/read_model.rs`.

Run:

```powershell
git diff --cached --check
```

Expected: no output and exit code `0`.

Run:

```powershell
git status --short --untracked-files=all
```

Expected: unrelated files remain unstaged.

- [ ] **Step 6: Commit the refactor**

Run:

```powershell
git commit -m "refactor: extract analysis store read model"
```

Expected: commit succeeds with exactly the staged Rust extraction.

- [ ] **Step 7: Compare final status to the pre-edit baseline**

Run:

```powershell
$statusPointerPath = Join-Path $env:TEMP 'analysis-store-read-model-latest-status-paths.txt'
$statusPointer = @{}
Get-Content -LiteralPath $statusPointerPath | ForEach-Object {
    $parts = $_ -split '=', 2
    if ($parts.Length -eq 2) {
        $statusPointer[$parts[0]] = $parts[1]
    }
}
$preEditStatusPath = $statusPointer['PRE_EDIT_STATUS_PATH']
$statusTag = $statusPointer['ANALYSIS_STORE_READ_MODEL_STATUS_TAG']
if (-not $preEditStatusPath -or -not $statusTag) {
    throw "missing pre-edit status pointer data"
}
$finalStatusPath = Join-Path $env:TEMP "analysis-store-read-model-$statusTag-final-status.txt"
git status --short --untracked-files=all | Set-Content -Encoding utf8 -LiteralPath $finalStatusPath
Compare-Object (Get-Content -LiteralPath $preEditStatusPath) (Get-Content -LiteralPath $finalStatusPath)
Get-Content -Raw -LiteralPath $finalStatusPath
```

Expected: no new unintended files or diffs remain after the commit. Any output from `Compare-Object` must be explained by intentional commit effects or pre-existing unrelated files. This step reloads paths from the temp pointer file so it works even when Task 1 and Task 4 run in separate PowerShell processes.

- [ ] **Step 8: Record final commit**

Run:

```powershell
git log -1 --oneline
```

Expected: latest commit is `refactor: extract analysis store read model`.

---

## Self-Review Notes

- Spec coverage: plan covers private `read_model` module creation, facade re-export, moved helper inventory, field-level visibility, import cleanup, worktree guard, fixture behavior coverage, source guards, Cargo tests, all-target check, format check, and commit hygiene.
- Placeholder scan: the forbidden-pattern scan was run and any inventory shorthand it found was replaced with concrete names.
- Type consistency: `AnalysisRunListFilters`, `AnalysisRunRow`, `AnalysisRunDetail`, `AnalysisRunSummary`, `AnalysisSnapshotState`, `AppError`, `AppResult`, and public facade function names match the approved design and current `store.rs`.
