# Analysis Store Setup Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract analysis store setup and lookup helpers from `src-tauri/src/analysis/store.rs` into `src-tauri/src/analysis/store/setup.rs` without changing behavior or external call paths.

**Architecture:** `store.rs` remains the `analysis::store` facade and keeps duplicate-run, insert, status, delete, read-model facade, snapshot facade, and inline test ownership. A private child module `setup.rs` owns builtin report-template setup, source existence validation, prompt-template fetch, and source-group fetch helpers, exposing only the existing `pub(crate)` setup API through a facade re-export in `store.rs`. Existing consumers keep importing from `analysis::store`.

**Tech Stack:** Rust, Tauri backend, SQLx SQLite, Cargo tests run with `--manifest-path src-tauri/Cargo.toml` from the repository root.

## Global Constraints

- Behavioral refactor edits are limited to `src-tauri/src/analysis/store.rs` and `src-tauri/src/analysis/store/setup.rs`.
- Do not change SQL, DTO mappings, builtin template body, default template name, template kind, source-group member ordering, item-count logic, database schema, migrations, frontend code, Tauri command payloads, or event payloads.
- Do not move duplicate-run lookup, analysis run insertion, run status mutation, saved-run deletion, read-model logic, snapshot logic, or store tests in this slice.
- Keep `src-tauri/src/analysis/store/setup.rs` private via `mod setup;`; do not use `pub mod setup`, `pub(crate) mod setup`, or root re-exports outside `store.rs`.
- Preserve the `analysis::store` facade API for current consumers in `analysis/mod.rs`, `analysis/templates.rs`, `analysis/groups.rs`, `analysis/report.rs`, and `analysis/corpus/source_resolution.rs`.
- Keep `AnalysisPromptTemplate` imported in production `store.rs` because `AnalysisRunInsert` still owns `prompt_template: &'a AnalysisPromptTemplate` after this slice.
- Run Cargo commands from the repository root using `--manifest-path src-tauri/Cargo.toml`.
- Run post-change test commands separately, or use a stopping wrapper that checks `$LASTEXITCODE` after every native command.
- Target files must be clean before editing. If `src-tauri/src/analysis/store.rs` or `src-tauri/src/analysis/store/setup.rs` has pre-existing tracked, staged, or untracked work, stop and make a separate baseline commit before starting this refactor. This plan uses full-file staging for target files.
- Do not stage unrelated dirty files such as local tool settings. Compare final status against the captured pre-edit status.

---

## File Structure

- Create `src-tauri/src/analysis/store/setup.rs`: owns setup imports, private builtin-template existence helper, public builtin-template/source/prompt/source-group APIs.
- Modify `src-tauri/src/analysis/store.rs`: declares private `mod setup;`, re-exports the existing public setup API, removes moved definitions and moved-only imports, and keeps all current tests.

---

### Task 1: Baseline And Worktree Guard

**Files:**
- Inspect: `src-tauri/src/analysis/store.rs`
- Inspect if present: `src-tauri/src/analysis/store/setup.rs`

**Interfaces:**
- Consumes: approved design `docs/superpowers/specs/2026-07-03-analysis-store-setup-refactor-design.md`.
- Produces: captured pre-edit status and baseline test evidence before extraction.

- [x] **Step 1: Capture the current worktree status**

Run:

```powershell
git status --short --untracked-files=all
```

Expected: record the exact output. Unrelated files may exist, but target files must be clean or handled before editing.

- [x] **Step 2: Prove tracked target files are clean**

Run:

```powershell
git diff -- src-tauri/src/analysis/store.rs
git diff --cached -- src-tauri/src/analysis/store.rs
```

Expected: no target-file diff. If `src-tauri/src/analysis/store.rs` is modified or staged before this refactor starts, stop here and make a separate baseline commit before editing.

- [x] **Step 3: Inspect a pre-existing `setup.rs` directly if present**

Run:

```powershell
if (Test-Path -LiteralPath 'src-tauri/src/analysis/store/setup.rs') {
    git status --short --untracked-files=all -- src-tauri/src/analysis/store/setup.rs
    Get-Item -LiteralPath 'src-tauri/src/analysis/store/setup.rs' | Select-Object FullName, Length, LastWriteTime
    Get-FileHash -Algorithm SHA256 -LiteralPath 'src-tauri/src/analysis/store/setup.rs'
    Get-Content -Raw -LiteralPath 'src-tauri/src/analysis/store/setup.rs'
}
```

Expected: no output if the file does not exist. If it exists as tracked, staged, modified, or untracked work before this refactor starts, stop here and make a separate baseline commit before editing.

- [x] **Step 4: Save a unique pre-edit status snapshot**

Run:

```powershell
$env:ANALYSIS_STORE_SETUP_STATUS_TAG = "{0}-{1}" -f (Get-Date -Format 'yyyyMMddHHmmss'), $PID
$statusPointerPath = Join-Path $env:TEMP 'analysis-store-setup-latest-status-paths.txt'
$preEditStatusPath = Join-Path $env:TEMP "analysis-store-setup-$env:ANALYSIS_STORE_SETUP_STATUS_TAG-pre-edit-status.txt"
git status --short --untracked-files=all | Set-Content -Encoding utf8 -LiteralPath $preEditStatusPath
"ANALYSIS_STORE_SETUP_STATUS_TAG=$env:ANALYSIS_STORE_SETUP_STATUS_TAG" | Set-Content -Encoding utf8 -LiteralPath $statusPointerPath
"PRE_EDIT_STATUS_PATH=$preEditStatusPath" | Add-Content -Encoding utf8 -LiteralPath $statusPointerPath
Get-Content -Raw -LiteralPath $preEditStatusPath
Get-Content -Raw -LiteralPath $statusPointerPath
```

Expected: the pre-edit status file contains the same relevant baseline status from Step 1, and the pointer file records the tag and status path for later PowerShell sessions.

- [x] **Step 5: Run baseline source existence test**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::ensure_sources_exist
```

Expected: PASS and not a green `0 tests` run.

- [x] **Step 6: Run baseline prompt-template fetch test**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::fetch_prompt_template
```

Expected: PASS and not a green `0 tests` run.

- [x] **Step 7: Run baseline full store test slice**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::
```

Expected: PASS and not a green `0 tests` run.

- [x] **Step 8: Run baseline groups consumer slice**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::groups::tests::
```

Expected: PASS and not a green `0 tests` run.

- [x] **Step 9: Run baseline source-resolution consumer slice**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::source_resolution::tests::
```

Expected: PASS and not a green `0 tests` run.

- [x] **Step 10: Run baseline report test slice**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::
```

Expected: PASS and not a green `0 tests` run.

- [x] **Step 11: Run baseline builtin template insertion test**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::tests::builtin_template_is_seeded_once
```

Expected: PASS and not a green `0 tests` run.

---

### Task 2: Extract Setup Module

**Files:**
- Create: `src-tauri/src/analysis/store/setup.rs`
- Modify: `src-tauri/src/analysis/store.rs`

**Interfaces:**
- Consumes from `store.rs`: `AnalysisPromptTemplate`, `AnalysisSourceGroup`, `AnalysisSourceGroupMember`, `AnalysisSourceGroupRow`, builtin template constants/helpers, `Pool<Sqlite>`, `AppError`, and `AppResult`.
- Produces through `analysis::store`: `ensure_builtin_report_template`, `ensure_sources_exist`, `fetch_prompt_template`, and `fetch_source_group`.

- [x] **Step 1: Add the private child module and facade re-export in `store.rs`**

Add these lines near the top of `src-tauri/src/analysis/store.rs`, beside the existing `read_model` and `snapshot` module declarations and facade re-exports:

```rust
mod setup;

pub(crate) use self::setup::{
    ensure_builtin_report_template, ensure_sources_exist, fetch_prompt_template,
    fetch_source_group,
};
```

Expected: `setup` is private. Do not write `pub mod setup;` or `pub(crate) mod setup;`.

- [x] **Step 2: Create `setup.rs` with the moved imports**

Create `src-tauri/src/analysis/store/setup.rs` with this import header before moving the functions:

```rust
use sqlx::{Pool, Sqlite};

use super::super::models::{
    AnalysisPromptTemplate, AnalysisSourceGroup, AnalysisSourceGroupMember, AnalysisSourceGroupRow,
};
use super::super::{
    default_report_template_body, now_secs, DEFAULT_REPORT_TEMPLATE_NAME, TEMPLATE_KIND_REPORT,
};
use crate::error::{AppError, AppResult};
```

Expected: `setup.rs` owns source-group model imports and builtin template constants/helpers.

- [x] **Step 3: Move setup helpers into `setup.rs`**

Move these exact items from `store.rs` into `setup.rs`, preserving bodies and order:

- `async fn builtin_report_template_exists(pool: &Pool<Sqlite>) -> AppResult<bool>`
- `pub(crate) async fn ensure_builtin_report_template(pool: &Pool<Sqlite>) -> AppResult<()>`
- `pub(crate) async fn ensure_sources_exist(pool: &Pool<Sqlite>, source_ids: &[i64]) -> AppResult<()>`
- `pub(crate) async fn fetch_prompt_template(pool: &Pool<Sqlite>, template_id: i64) -> AppResult<AnalysisPromptTemplate>`
- `pub(crate) async fn fetch_source_group(pool: &Pool<Sqlite>, group_id: i64) -> AppResult<Option<AnalysisSourceGroup>>`

Implementation detail: the current contiguous move starts at `async fn builtin_report_template_exists(` and ends after the closing brace of `pub(crate) async fn fetch_source_group(pool: &Pool<Sqlite>, group_id: i64) -> AppResult<Option<AnalysisSourceGroup>>`. Leave `DuplicateRunLookup` and everything after it in `store.rs`.

- [x] **Step 4: Keep private helper private**

Verify the moved builtin existence helper has no visibility modifier in `setup.rs`:

```rust
async fn builtin_report_template_exists(pool: &Pool<Sqlite>) -> AppResult<bool>
```

Expected: only the four facade API items are `pub(crate)`.

- [x] **Step 5: Trim moved-only imports from `store.rs`**

Update the top of `src-tauri/src/analysis/store.rs` so moved-only imports are gone while remaining store and test code still compile. The production import set should no longer need `AnalysisSourceGroup`, `AnalysisSourceGroupMember`, `AnalysisSourceGroupRow`, `default_report_template_body`, `now_secs`, `DEFAULT_REPORT_TEMPLATE_NAME`, or `TEMPLATE_KIND_REPORT`.

The opening imports should be shaped like this unless later compile feedback proves another remaining production item is needed:

```rust
use sqlx::{Pool, Sqlite};

use super::corpus::YoutubeCorpusMode;
use super::models::AnalysisPromptTemplate;
use super::{
    ANALYSIS_RUN_TYPE_REPORT, ANALYSIS_STATUS_QUEUED, ANALYSIS_STATUS_RUNNING,
};
use crate::error::{AppError, AppResult};
```

Expected: `AnalysisPromptTemplate` remains in production imports for `AnalysisRunInsert`. Test-only imports can remain inside `#[cfg(test)] mod tests`.

- [x] **Step 6: Leave the inline store tests in `store.rs`**

Keep the existing test module in `src-tauri/src/analysis/store.rs`. Its `use super` block should continue importing the facade names from `store.rs`:

```rust
use super::{
    capture_run_snapshot, delete_saved_run, ensure_sources_exist, fetch_prompt_template,
    list_analysis_run_summaries, map_run_detail, map_run_summary, mark_run_capture_failed,
    resolve_run_scope_label, sanitize_provider_error, sanitize_snapshot_error, set_run_status,
    AnalysisRunListFilters,
};
```

Expected: tests do not import from `super::setup` or call `setup::` directly. This keeps store tests exercising the facade contract.

- [x] **Step 7: Run formatter**

Run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml
```

Expected: formatting completes. If rustfmt touches unrelated Rust files, inspect `git status --short --untracked-files=all` and keep unrelated drift out of the refactor commit.

---

### Task 3: Source Guards And Verification

**Files:**
- Verify: `src-tauri/src/analysis/store.rs`
- Verify: `src-tauri/src/analysis/store/setup.rs`

**Interfaces:**
- Consumes: extracted facade from Task 2.
- Produces: proof that moved helpers left `store.rs`, facade exports are intact, private helper stayed private, imports are clean, and behavior-focused tests pass.

- [ ] **Step 1: Confirm moved definitions no longer live in `store.rs`**

Run:

```powershell
rg -n "^\s*(pub\([^)]*\)\s+|pub\s+)?(fn|async fn) (builtin_report_template_exists|ensure_builtin_report_template|ensure_sources_exist|fetch_prompt_template|fetch_source_group)\b" src-tauri/src/analysis/store.rs
```

Expected: no matches. `rg` exit code `1` is expected for this no-match guard.

- [ ] **Step 2: Confirm `store.rs` has a private module declaration**

Run:

```powershell
rg -n "^mod setup;" src-tauri/src/analysis/store.rs
```

Expected: exactly one match.

Run:

```powershell
rg -n "^pub.*mod setup" src-tauri/src/analysis/store.rs
```

Expected: no matches. `rg` exit code `1` is expected.

- [ ] **Step 3: Confirm facade re-export exists and contains every public setup name**

Run:

```powershell
rg -n "^pub\(crate\) use self::setup::" src-tauri/src/analysis/store.rs
```

Expected: exactly one facade anchor.

Run:

```powershell
$storeFacade = Get-Content -Raw -LiteralPath 'src-tauri/src/analysis/store.rs'
$setupReExport = [regex]::Match($storeFacade, "pub\(crate\) use self::setup::\{(?<block>[\s\S]*?)\};")
if (-not $setupReExport.Success) {
    throw "missing setup facade re-export block"
}
foreach ($name in @('ensure_builtin_report_template', 'ensure_sources_exist', 'fetch_prompt_template', 'fetch_source_group')) {
    if ($setupReExport.Groups['block'].Value -notmatch ("\b" + [regex]::Escape($name) + "\b")) {
        throw "missing setup facade re-export: $name"
    }
}
```

Expected: loop completes without throwing.

- [ ] **Step 4: Confirm moved public API exists in `setup.rs`**

Run:

```powershell
rg -n "^pub\(crate\) async fn (ensure_builtin_report_template|ensure_sources_exist|fetch_prompt_template|fetch_source_group)\b" src-tauri/src/analysis/store/setup.rs
```

Expected: four matches, one for each public setup API item.

- [ ] **Step 5: Confirm private helper stayed private**

Run:

```powershell
rg -n "^async fn builtin_report_template_exists\b" src-tauri/src/analysis/store/setup.rs
```

Expected: one match.

Run:

```powershell
rg -n "^\s*pub(\([^)]*\))?\s+async fn builtin_report_template_exists\b" src-tauri/src/analysis/store/setup.rs
```

Expected: no matches. `rg` exit code `1` is expected.

- [ ] **Step 6: Confirm moved-only production imports left `store.rs`**

Run:

```powershell
$storeProduction = [regex]::Split((Get-Content -Raw -LiteralPath 'src-tauri/src/analysis/store.rs'), "#\[cfg\(test\)\]\s*mod tests", 2)[0]
foreach ($name in @('AnalysisSourceGroup', 'AnalysisSourceGroupMember', 'AnalysisSourceGroupRow', 'default_report_template_body', 'now_secs', 'DEFAULT_REPORT_TEMPLATE_NAME', 'TEMPLATE_KIND_REPORT')) {
    if ($storeProduction -match ("\b" + [regex]::Escape($name) + "\b")) {
        throw "moved-only production import remains in store.rs: $name"
    }
}
if ($storeProduction -notmatch "\bAnalysisPromptTemplate\b") {
    throw "AnalysisPromptTemplate should remain in store.rs for AnalysisRunInsert"
}
```

Expected: command completes without throwing.

- [ ] **Step 7: Confirm store tests still exercise the facade**

Run:

```powershell
$storeTests = [regex]::Match((Get-Content -Raw -LiteralPath 'src-tauri/src/analysis/store.rs'), "(?s)#\[cfg\(test\)\]\s*mod tests \{.*\z").Value
if ($storeTests -match "super::setup|setup::") {
    throw "store tests must use the store facade, not setup directly"
}
```

Expected: command completes without throwing.

- [ ] **Step 8: Confirm setup literals and SQL markers stayed in `setup.rs`**

Run each command separately:

```powershell
rg -n -F "SELECT EXISTS(" src-tauri/src/analysis/store/setup.rs
```

```powershell
rg -n -F "DEFAULT_REPORT_TEMPLATE_NAME" src-tauri/src/analysis/store/setup.rs
```

```powershell
rg -n -F "Source {source_id} not found" src-tauri/src/analysis/store/setup.rs
```

```powershell
rg -n -F "Analysis prompt template {template_id} not found" src-tauri/src/analysis/store/setup.rs
```

```powershell
rg -n -F "ORDER BY COALESCE(sources.title, ''), sources.id" src-tauri/src/analysis/store/setup.rs
```

Expected: every command prints at least one match.

- [ ] **Step 9: Run source existence test**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::ensure_sources_exist
```

Expected: PASS and not a green `0 tests` run.

- [ ] **Step 10: Run prompt-template fetch test**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::fetch_prompt_template
```

Expected: PASS and not a green `0 tests` run.

- [ ] **Step 11: Run full store test slice**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::
```

Expected: PASS and not a green `0 tests` run.

- [ ] **Step 12: Run groups consumer slice**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::groups::tests::
```

Expected: PASS and not a green `0 tests` run.

- [ ] **Step 13: Run source-resolution consumer slice**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::source_resolution::tests::
```

Expected: PASS and not a green `0 tests` run.

- [ ] **Step 14: Run report test slice**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::
```

Expected: PASS and not a green `0 tests` run.

- [ ] **Step 15: Run builtin template insertion test**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::tests::builtin_template_is_seeded_once
```

Expected: PASS and not a green `0 tests` run.

- [ ] **Step 16: Run all-target compile coverage**

Run:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
```

Expected: PASS. Existing warnings outside touched files may remain; new warnings mentioning `src-tauri/src/analysis/store.rs` or `src-tauri/src/analysis/store/setup.rs` are not acceptable.

- [ ] **Step 17: Run format check**

Run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
```

Expected: PASS with no diff output.

---

### Task 4: Commit And Final Hygiene

**Files:**
- Stage: `src-tauri/src/analysis/store.rs`
- Stage: `src-tauri/src/analysis/store/setup.rs`

**Interfaces:**
- Consumes: verified extraction from Task 3.
- Produces: one implementation commit with only the expected Rust files.

- [ ] **Step 1: Inspect implementation diff**

Run:

```powershell
git diff -- src-tauri/src/analysis/store.rs src-tauri/src/analysis/store/setup.rs
```

Expected: `store.rs` loses only the moved setup cluster and moved-only imports, gains `mod setup;` plus the facade re-export, and `setup.rs` contains the moved code.

- [ ] **Step 2: Check for whitespace errors before staging**

Run:

```powershell
git diff --check -- src-tauri/src/analysis/store.rs src-tauri/src/analysis/store/setup.rs
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
git add -- src-tauri/src/analysis/store.rs src-tauri/src/analysis/store/setup.rs
```

Expected: only the two Rust files are staged.

- [ ] **Step 5: Verify staged stat**

Run:

```powershell
git diff --cached --stat
```

Expected: cached stat lists only `src-tauri/src/analysis/store.rs` and `src-tauri/src/analysis/store/setup.rs`.

- [ ] **Step 6: Verify staged whitespace**

Run:

```powershell
git diff --cached --check
```

Expected: no output and exit code `0`.

- [ ] **Step 7: Verify unrelated files remain unstaged**

Run:

```powershell
git status --short --untracked-files=all
```

Expected: unrelated files remain unstaged.

- [ ] **Step 8: Commit the refactor**

Run:

```powershell
git commit -m "refactor: extract analysis store setup logic"
```

Expected: commit succeeds with exactly the staged Rust extraction.

- [ ] **Step 9: Compare final status to the pre-edit baseline**

Run:

```powershell
$statusPointerPath = Join-Path $env:TEMP 'analysis-store-setup-latest-status-paths.txt'
$statusPointer = @{}
Get-Content -LiteralPath $statusPointerPath | ForEach-Object {
    $parts = $_ -split '=', 2
    if ($parts.Length -eq 2) {
        $statusPointer[$parts[0]] = $parts[1]
    }
}
$preEditStatusPath = $statusPointer['PRE_EDIT_STATUS_PATH']
$statusTag = $statusPointer['ANALYSIS_STORE_SETUP_STATUS_TAG']
if (-not $preEditStatusPath -or -not $statusTag) {
    throw "missing pre-edit status pointer data"
}
$finalStatusPath = Join-Path $env:TEMP "analysis-store-setup-$statusTag-final-status.txt"
git status --short --untracked-files=all | Set-Content -Encoding utf8 -LiteralPath $finalStatusPath
Compare-Object (Get-Content -LiteralPath $preEditStatusPath) (Get-Content -LiteralPath $finalStatusPath)
Get-Content -Raw -LiteralPath $finalStatusPath
```

Expected: no new unintended files or diffs remain after the commit. Any output from `Compare-Object` must be explained by intentional commit effects or pre-existing unrelated files.

- [ ] **Step 10: Record final commit**

Run:

```powershell
git log -1 --oneline
```

Expected: latest commit is `refactor: extract analysis store setup logic`.

---

## Self-Review Notes

- Spec coverage: plan covers private `setup` module creation, facade re-export, moved helper inventory, private helper visibility, import cleanup, worktree guard, baseline/post-change test slices, source guards, Cargo check, format check, and commit hygiene.
- Placeholder scan: the forbidden-pattern scan was run and no placeholder markers remain.
- Type consistency: `AnalysisPromptTemplate`, `AnalysisSourceGroup`, `AnalysisSourceGroupMember`, `AnalysisSourceGroupRow`, `Pool<Sqlite>`, `AppResult`, `ensure_builtin_report_template`, `ensure_sources_exist`, `fetch_prompt_template`, and `fetch_source_group` match the approved design and current `store.rs`.
