# Analysis Store Snapshot Refactor Implementation Plan

**Status:** implemented historical execution record. Rust extraction landed in `b8368f69 refactor: extract analysis store snapshot logic`, verification landed in `25bd92e8 chore: verify analysis store snapshot extraction`, and this checklist was marked complete in `5acaed82 docs: complete analysis store snapshot plan`.

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract analysis snapshot capture, persistence, failure marking, reload, validation, and sanitization logic from `src-tauri/src/analysis/store.rs` into `src-tauri/src/analysis/store/snapshot.rs` without changing behavior or external call paths.

**Architecture:** `store.rs` remains the `analysis::store` facade and keeps prompt-template, source, duplicate-run, insert, status, delete, read-model facade, and inline test ownership. A private child module `snapshot.rs` owns snapshot persistence and sanitization helpers and exposes only the existing `pub(crate)` snapshot API through a facade re-export in `store.rs`. Existing consumers keep importing from `analysis::store`.

**Tech Stack:** Rust, Tauri backend, SQLx SQLite, zstd-backed compression helpers, Cargo tests run with `--manifest-path src-tauri/Cargo.toml` from the repository root.

## Global Constraints

- Behavioral refactor edits are limited to `src-tauri/src/analysis/store.rs` and `src-tauri/src/analysis/store/snapshot.rs`.
- Do not change SQL, DTO mappings, compression format, metadata handling, validation rules, sanitization rules, transaction boundaries, database schema, migrations, frontend code, Tauri command payloads, or event payloads.
- Do not move prompt-template initialization, source existence checks, source-group loading, duplicate-run lookup, analysis run insertion, run status mutation, saved-run deletion, read-model logic, or store tests in this slice.
- Keep `src-tauri/src/analysis/store/snapshot.rs` private via `mod snapshot;`; do not use `pub mod snapshot`, `pub(crate) mod snapshot`, or root re-exports outside `store.rs`.
- Preserve the `analysis::store` facade API for current consumers in `analysis/report/capture.rs`, `analysis/report/lifecycle.rs`, `analysis/corpus/tests/snapshot.rs`, and `analysis/corpus/tests/source_resolution.rs`.
- Preserve `#[allow(dead_code)]` on `persist_run_snapshot` unless a non-test production reader is added in the same implementation.
- Run Cargo commands from the repository root using `--manifest-path src-tauri/Cargo.toml`.
- Run post-change test commands separately, or use a stopping wrapper that checks `$LASTEXITCODE` after every native command.
- Target files must be clean before editing. If `src-tauri/src/analysis/store.rs` or `src-tauri/src/analysis/store/snapshot.rs` has pre-existing tracked, staged, or untracked work, stop and make a separate baseline commit before starting this refactor. This plan uses full-file staging for target files.
- Do not stage unrelated dirty files such as local tool settings. Compare final status against the captured pre-edit status.

---

## File Structure

- Create `src-tauri/src/analysis/store/snapshot.rs`: owns snapshot imports, private validation and reload helpers, public snapshot capture/persist/failure APIs, and snapshot/provider error sanitization.
- Modify `src-tauri/src/analysis/store.rs`: declares private `mod snapshot;`, re-exports the existing public snapshot API, removes moved definitions and moved-only imports, and keeps all current tests.

---

### Task 1: Baseline And Worktree Guard

**Files:**
- Inspect: `src-tauri/src/analysis/store.rs`
- Inspect if present: `src-tauri/src/analysis/store/snapshot.rs`

**Interfaces:**
- Consumes: approved design `docs/superpowers/specs/2026-07-03-analysis-store-snapshot-refactor-design.md`.
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

- [x] **Step 3: Inspect a pre-existing `snapshot.rs` directly if present**

Run:

```powershell
if (Test-Path -LiteralPath 'src-tauri/src/analysis/store/snapshot.rs') {
    git status --short --untracked-files=all -- src-tauri/src/analysis/store/snapshot.rs
    Get-Item -LiteralPath 'src-tauri/src/analysis/store/snapshot.rs' | Select-Object FullName, Length, LastWriteTime
    Get-FileHash -Algorithm SHA256 -LiteralPath 'src-tauri/src/analysis/store/snapshot.rs'
    Get-Content -Raw -LiteralPath 'src-tauri/src/analysis/store/snapshot.rs'
}
```

Expected: no output if the file does not exist. If it exists as tracked, staged, modified, or untracked work before this refactor starts, stop here and make a separate baseline commit before editing.

- [x] **Step 4: Save a unique pre-edit status snapshot**

Run:

```powershell
$env:ANALYSIS_STORE_SNAPSHOT_STATUS_TAG = "{0}-{1}" -f (Get-Date -Format 'yyyyMMddHHmmss'), $PID
$statusPointerPath = Join-Path $env:TEMP 'analysis-store-snapshot-latest-status-paths.txt'
$preEditStatusPath = Join-Path $env:TEMP "analysis-store-snapshot-$env:ANALYSIS_STORE_SNAPSHOT_STATUS_TAG-pre-edit-status.txt"
git status --short --untracked-files=all | Set-Content -Encoding utf8 -LiteralPath $preEditStatusPath
"ANALYSIS_STORE_SNAPSHOT_STATUS_TAG=$env:ANALYSIS_STORE_SNAPSHOT_STATUS_TAG" | Set-Content -Encoding utf8 -LiteralPath $statusPointerPath
"PRE_EDIT_STATUS_PATH=$preEditStatusPath" | Add-Content -Encoding utf8 -LiteralPath $statusPointerPath
Get-Content -Raw -LiteralPath $preEditStatusPath
Get-Content -Raw -LiteralPath $statusPointerPath
```

Expected: the pre-edit status file contains the same relevant baseline status from Step 1, and the pointer file records the tag and status path for later PowerShell sessions.

- [x] **Step 5: Run baseline snapshot sanitizer test**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::sanitize_snapshot_error
```

Expected: PASS and not a green `0 tests` run.

- [x] **Step 6: Run baseline provider sanitizer test**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::sanitize_provider_error
```

Expected: PASS and not a green `0 tests` run.

- [x] **Step 7: Run baseline snapshot capture test**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::capture_run_snapshot
```

Expected: PASS and not a green `0 tests` run.

- [x] **Step 8: Run baseline capture failure marker test**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::mark_run_capture_failed
```

Expected: PASS and not a green `0 tests` run.

- [x] **Step 9: Run baseline full store test slice**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::
```

Expected: PASS and not a green `0 tests` run.

- [x] **Step 10: Run baseline corpus snapshot consumer slice**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::snapshot
```

Expected: PASS and not a green `0 tests` run.

- [x] **Step 11: Run baseline source resolution consumer slice**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::source_resolution
```

Expected: PASS and not a green `0 tests` run.

- [x] **Step 12: Run baseline report test slice**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::
```

Expected: PASS and not a green `0 tests` run.

---

### Task 2: Extract Snapshot Module

**Files:**
- Create: `src-tauri/src/analysis/store/snapshot.rs`
- Modify: `src-tauri/src/analysis/store.rs`

**Interfaces:**
- Consumes from `store.rs`: `CorpusMessage`, `StoredRunSnapshotRow`, `Pool<Sqlite>`, compression helpers, `internal_error`, `AppError`, `AppResult`, and `ANALYSIS_STATUS_FAILED`.
- Produces through `analysis::store`: `sanitize_snapshot_error`, `sanitize_provider_error`, `capture_run_snapshot`, `persist_run_snapshot`, and `mark_run_capture_failed`.

- [x] **Step 1: Add the private child module and facade re-export in `store.rs`**

Add these lines near the top of `src-tauri/src/analysis/store.rs`, beside the existing `read_model` module declaration and facade re-export:

```rust
mod snapshot;

pub(crate) use self::snapshot::{
    capture_run_snapshot, mark_run_capture_failed, persist_run_snapshot, sanitize_provider_error,
    sanitize_snapshot_error,
};
```

Expected: `snapshot` is private. Do not write `pub mod snapshot;` or `pub(crate) mod snapshot;`.

- [x] **Step 2: Create `snapshot.rs` with the moved imports**

Create `src-tauri/src/analysis/store/snapshot.rs` with this import header before moving the functions:

```rust
use sqlx::{Pool, Sqlite};

use super::super::models::{CorpusMessage, StoredRunSnapshotRow};
use super::super::ANALYSIS_STATUS_FAILED;
use crate::compression::{compress_text, decompress_text};
use crate::error::{internal_error, AppError, AppResult};
```

Expected: `snapshot.rs` imports `ANALYSIS_STATUS_FAILED` and the moved `mark_run_capture_failed` body binds `ANALYSIS_STATUS_FAILED` directly. Do not keep `crate::analysis::ANALYSIS_STATUS_FAILED` in that moved function if this import is present.

- [x] **Step 3: Move snapshot helpers into `snapshot.rs`**

Move these exact items from `store.rs` into `snapshot.rs`, preserving bodies and order except for the `ANALYSIS_STATUS_FAILED` path adjustment described in Step 2:

- `pub(crate) fn sanitize_snapshot_error(category: &str, raw: &str) -> String`
- `pub(crate) fn sanitize_provider_error(category: &str, raw: &str) -> String`
- `fn validate_snapshot_message(message: &CorpusMessage) -> AppResult<()>`
- `async fn load_run_snapshot_messages_on_transaction(tx: &mut sqlx::Transaction<'_, Sqlite>, run_id: i64) -> AppResult<Vec<CorpusMessage>>`
- `pub(crate) async fn capture_run_snapshot(pool: &Pool<Sqlite>, run_id: i64, scope_label: &str, corpus: &[CorpusMessage]) -> AppResult<Vec<CorpusMessage>>`
- `#[allow(dead_code)] pub(crate) async fn persist_run_snapshot(pool: &Pool<Sqlite>, run_id: i64, scope_label: &str, corpus: &[CorpusMessage]) -> AppResult<()>`
- `pub(crate) async fn mark_run_capture_failed(pool: &Pool<Sqlite>, run_id: i64, snapshot_error: &str, completed_at: i64) -> AppResult<()>`

Implementation detail: the current contiguous move starts at `pub(crate) fn sanitize_snapshot_error(` and ends after the closing brace of `pub(crate) async fn mark_run_capture_failed(pool: &Pool<Sqlite>, run_id: i64, snapshot_error: &str, completed_at: i64) -> AppResult<()>`. Leave `set_run_status` and everything after it in `store.rs`.

- [x] **Step 4: Preserve the dead-code allowance**

Ensure the moved `persist_run_snapshot` keeps the allowance directly attached to the function:

```rust
#[allow(dead_code)]
pub(crate) async fn persist_run_snapshot(
    pool: &Pool<Sqlite>,
    run_id: i64,
    scope_label: &str,
    corpus: &[CorpusMessage],
) -> AppResult<()> {
    capture_run_snapshot(pool, run_id, scope_label, corpus)
        .await
        .map(|_| ())
}
```

Expected: normal non-test builds do not gain a dead-code warning for `persist_run_snapshot`.

- [x] **Step 5: Keep private helpers private**

Verify these moved helpers have no visibility modifier in `snapshot.rs`:

- `fn validate_snapshot_message`
- `async fn load_run_snapshot_messages_on_transaction`

Expected: only the five facade API items are `pub(crate)`.

- [x] **Step 6: Trim moved-only imports from `store.rs`**

Update the top of `src-tauri/src/analysis/store.rs` so moved-only imports are gone while remaining store and test code still compile. The production import set should no longer need `CorpusMessage`, `StoredRunSnapshotRow`, `compress_text`, `decompress_text`, `internal_error`, or `ANALYSIS_STATUS_FAILED`.

The opening imports should be shaped like this unless later compile feedback proves another remaining production item is needed:

```rust
use sqlx::{Pool, Sqlite};

use super::corpus::YoutubeCorpusMode;
use super::models::{
    AnalysisPromptTemplate, AnalysisSourceGroup, AnalysisSourceGroupMember, AnalysisSourceGroupRow,
};
use super::{
    default_report_template_body, now_secs, ANALYSIS_RUN_TYPE_REPORT, ANALYSIS_STATUS_QUEUED,
    ANALYSIS_STATUS_RUNNING, DEFAULT_REPORT_TEMPLATE_NAME, TEMPLATE_KIND_REPORT,
};
use crate::error::{AppError, AppResult};
```

Expected: test-only imports such as `CorpusMessage` can remain inside `#[cfg(test)] mod tests`, but moved-only names must not remain in the production section before the test module.

- [x] **Step 7: Leave the inline store tests in `store.rs`**

Keep the existing test module in `src-tauri/src/analysis/store.rs`. Its `use super` block should continue importing the facade names from `store.rs`:

```rust
use super::{
    capture_run_snapshot, delete_saved_run, ensure_sources_exist, fetch_prompt_template,
    list_analysis_run_summaries, map_run_detail, map_run_summary, mark_run_capture_failed,
    resolve_run_scope_label, sanitize_provider_error, sanitize_snapshot_error, set_run_status,
    AnalysisRunListFilters,
};
```

Expected: tests do not import from `super::snapshot` or call `snapshot::` directly. This keeps store tests exercising the facade contract.

- [x] **Step 8: Run formatter**

Run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml
```

Expected: formatting completes. If rustfmt touches unrelated Rust files, inspect `git status --short --untracked-files=all` and keep unrelated drift out of the refactor commit.

---

### Task 3: Source Guards And Verification

**Files:**
- Verify: `src-tauri/src/analysis/store.rs`
- Verify: `src-tauri/src/analysis/store/snapshot.rs`

**Interfaces:**
- Consumes: extracted facade from Task 2.
- Produces: proof that moved helpers left `store.rs`, facade exports are intact, private helpers stayed private, imports are clean, and behavior-focused tests pass.

- [x] **Step 1: Confirm moved definitions no longer live in `store.rs`**

Run:

```powershell
rg -n "^\s*(pub\([^)]*\)\s+|pub\s+)?(fn|async fn) (sanitize_snapshot_error|sanitize_provider_error|validate_snapshot_message|load_run_snapshot_messages_on_transaction|capture_run_snapshot|persist_run_snapshot|mark_run_capture_failed)\b" src-tauri/src/analysis/store.rs
```

Expected: no matches. `rg` exit code `1` is expected for this no-match guard.

- [x] **Step 2: Confirm `store.rs` has a private module declaration**

Run:

```powershell
rg -n "^mod snapshot;" src-tauri/src/analysis/store.rs
```

Expected: exactly one match.

Run:

```powershell
rg -n "^pub.*mod snapshot" src-tauri/src/analysis/store.rs
```

Expected: no matches. `rg` exit code `1` is expected.

- [x] **Step 3: Confirm facade re-export exists and contains every public snapshot name**

Run:

```powershell
rg -n "^pub\(crate\) use self::snapshot::" src-tauri/src/analysis/store.rs
```

Expected: exactly one facade anchor.

Run:

```powershell
$storeFacade = Get-Content -Raw -LiteralPath 'src-tauri/src/analysis/store.rs'
$snapshotReExport = [regex]::Match($storeFacade, "pub\(crate\) use self::snapshot::\{(?<block>[\s\S]*?)\};")
if (-not $snapshotReExport.Success) {
    throw "missing snapshot facade re-export block"
}
foreach ($name in @('capture_run_snapshot', 'mark_run_capture_failed', 'persist_run_snapshot', 'sanitize_provider_error', 'sanitize_snapshot_error')) {
    if ($snapshotReExport.Groups['block'].Value -notmatch ("\b" + [regex]::Escape($name) + "\b")) {
        throw "missing snapshot facade re-export: $name"
    }
}
```

Expected: loop completes without throwing.

- [x] **Step 4: Confirm moved public API exists in `snapshot.rs`**

Run:

```powershell
rg -n "^pub\(crate\) (fn|async fn) (sanitize_snapshot_error|sanitize_provider_error|capture_run_snapshot|persist_run_snapshot|mark_run_capture_failed)\b" src-tauri/src/analysis/store/snapshot.rs
```

Expected: five matches, one for each public snapshot API item.

- [x] **Step 5: Confirm private helpers stayed private**

Run:

```powershell
rg -n "^(fn|async fn) (validate_snapshot_message|load_run_snapshot_messages_on_transaction)\b" src-tauri/src/analysis/store/snapshot.rs
```

Expected: two matches.

Run:

```powershell
rg -n "^\s*pub(\([^)]*\))?\s+(fn|async fn) (validate_snapshot_message|load_run_snapshot_messages_on_transaction)\b" src-tauri/src/analysis/store/snapshot.rs
```

Expected: no matches. `rg` exit code `1` is expected.

- [x] **Step 6: Confirm moved-only production imports left `store.rs`**

Run:

```powershell
$storeProduction = [regex]::Split((Get-Content -Raw -LiteralPath 'src-tauri/src/analysis/store.rs'), "#\[cfg\(test\)\]\s*mod tests", 2)[0]
foreach ($name in @('CorpusMessage', 'StoredRunSnapshotRow', 'compress_text', 'decompress_text', 'internal_error', 'ANALYSIS_STATUS_FAILED')) {
    if ($storeProduction -match ("\b" + [regex]::Escape($name) + "\b")) {
        throw "moved-only production import remains in store.rs: $name"
    }
}
```

Expected: command completes without throwing.

- [x] **Step 7: Confirm store tests still exercise the facade**

Run:

```powershell
$storeTests = [regex]::Match((Get-Content -Raw -LiteralPath 'src-tauri/src/analysis/store.rs'), "(?s)#\[cfg\(test\)\]\s*mod tests \{.*\z").Value
if ($storeTests -match "super::snapshot|snapshot::") {
    throw "store tests must use the store facade, not snapshot directly"
}
```

Expected: command completes without throwing.

- [x] **Step 8: Confirm `persist_run_snapshot` kept its dead-code allowance**

Run:

```powershell
$snapshotSource = Get-Content -Raw -LiteralPath 'src-tauri/src/analysis/store/snapshot.rs'
if ($snapshotSource -notmatch "#\[allow\(dead_code\)\]\s*pub\(crate\) async fn persist_run_snapshot") {
    throw "persist_run_snapshot must preserve #[allow(dead_code)]"
}
```

Expected: command completes without throwing.

- [x] **Step 9: Confirm validation and SQL markers stayed in `snapshot.rs`**

Run each command separately:

```powershell
rg -n -F "Snapshot capture failed: empty corpus" src-tauri/src/analysis/store/snapshot.rs
```

```powershell
rg -n -F "Snapshot capture failed: reloaded snapshot is empty" src-tauri/src/analysis/store/snapshot.rs
```

```powershell
rg -n -F "source_subtype is required for" src-tauri/src/analysis/store/snapshot.rs
```

```powershell
rg -n -F "DELETE FROM analysis_run_messages WHERE run_id = ?" src-tauri/src/analysis/store/snapshot.rs
```

```powershell
rg -n -F "UPDATE analysis_runs SET snapshot_captured_at = datetime('now'), snapshot_error = NULL WHERE id = ?" src-tauri/src/analysis/store/snapshot.rs
```

```powershell
rg -n -F "Snapshot capture failed" src-tauri/src/analysis/store/snapshot.rs
```

Expected: every command prints at least one match.

- [x] **Step 10: Run snapshot sanitizer test**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::sanitize_snapshot_error
```

Expected: PASS and not a green `0 tests` run.

- [x] **Step 11: Run provider sanitizer test**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::sanitize_provider_error
```

Expected: PASS and not a green `0 tests` run.

- [x] **Step 12: Run snapshot capture test**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::capture_run_snapshot
```

Expected: PASS and not a green `0 tests` run.

- [x] **Step 13: Run capture failure marker test**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::mark_run_capture_failed
```

Expected: PASS and not a green `0 tests` run.

- [x] **Step 14: Run full store test slice**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::
```

Expected: PASS and not a green `0 tests` run.

- [x] **Step 15: Run corpus snapshot consumer slice**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::snapshot
```

Expected: PASS and not a green `0 tests` run.

- [x] **Step 16: Run source resolution consumer slice**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::source_resolution
```

Expected: PASS and not a green `0 tests` run.

- [x] **Step 17: Run report test slice**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::
```

Expected: PASS and not a green `0 tests` run. This is broad report-module regression coverage; do not claim dedicated lifecycle runtime coverage unless a concrete lifecycle test is added or identified.

- [x] **Step 18: Run all-target compile coverage**

Run:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
```

Expected: PASS. Existing warnings outside touched files may remain; new warnings mentioning `src-tauri/src/analysis/store.rs` or `src-tauri/src/analysis/store/snapshot.rs` are not acceptable.

- [x] **Step 19: Run format check**

Run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
```

Expected: PASS with no diff output.

---

### Task 4: Commit And Final Hygiene

**Files:**
- Stage: `src-tauri/src/analysis/store.rs`
- Stage: `src-tauri/src/analysis/store/snapshot.rs`

**Interfaces:**
- Consumes: verified extraction from Task 3.
- Produces: one implementation commit with only the expected Rust files.

- [x] **Step 1: Inspect implementation diff**

Run:

```powershell
git diff -- src-tauri/src/analysis/store.rs src-tauri/src/analysis/store/snapshot.rs
```

Expected: `store.rs` loses only the moved snapshot cluster and moved-only imports, gains `mod snapshot;` plus the facade re-export, and `snapshot.rs` contains the moved code.

- [x] **Step 2: Check for whitespace errors before staging**

Run:

```powershell
git diff --check -- src-tauri/src/analysis/store.rs src-tauri/src/analysis/store/snapshot.rs
```

Expected: no output and exit code `0`.

- [x] **Step 3: Inspect status before staging**

Run:

```powershell
git status --short --untracked-files=all
```

Expected: only expected target-file changes plus any recorded pre-existing unrelated files. Do not stage unrelated files.

- [x] **Step 4: Stage only implementation-owned files**

Run:

```powershell
git add -- src-tauri/src/analysis/store.rs src-tauri/src/analysis/store/snapshot.rs
```

Expected: only the two Rust files are staged.

- [x] **Step 5: Verify staged stat**

Run:

```powershell
git diff --cached --stat
```

Expected: cached stat lists only `src-tauri/src/analysis/store.rs` and `src-tauri/src/analysis/store/snapshot.rs`.

- [x] **Step 6: Verify staged whitespace**

Run:

```powershell
git diff --cached --check
```

Expected: no output and exit code `0`.

- [x] **Step 7: Verify unrelated files remain unstaged**

Run:

```powershell
git status --short --untracked-files=all
```

Expected: unrelated files remain unstaged.

- [x] **Step 8: Commit the refactor**

Run:

```powershell
git commit -m "refactor: extract analysis store snapshot logic"
```

Expected: commit succeeds with exactly the staged Rust extraction.

- [x] **Step 9: Compare final status to the pre-edit baseline**

Run:

```powershell
$statusPointerPath = Join-Path $env:TEMP 'analysis-store-snapshot-latest-status-paths.txt'
$statusPointer = @{}
Get-Content -LiteralPath $statusPointerPath | ForEach-Object {
    $parts = $_ -split '=', 2
    if ($parts.Length -eq 2) {
        $statusPointer[$parts[0]] = $parts[1]
    }
}
$preEditStatusPath = $statusPointer['PRE_EDIT_STATUS_PATH']
$statusTag = $statusPointer['ANALYSIS_STORE_SNAPSHOT_STATUS_TAG']
if (-not $preEditStatusPath -or -not $statusTag) {
    throw "missing pre-edit status pointer data"
}
$finalStatusPath = Join-Path $env:TEMP "analysis-store-snapshot-$statusTag-final-status.txt"
git status --short --untracked-files=all | Set-Content -Encoding utf8 -LiteralPath $finalStatusPath
Compare-Object (Get-Content -LiteralPath $preEditStatusPath) (Get-Content -LiteralPath $finalStatusPath)
Get-Content -Raw -LiteralPath $finalStatusPath
```

Expected: no new unintended files or diffs remain after the commit. Any output from `Compare-Object` must be explained by intentional commit effects or pre-existing unrelated files.

- [x] **Step 10: Record final commit**

Run:

```powershell
git log -1 --oneline
```

Expected: latest commit is `refactor: extract analysis store snapshot logic`.

---

## Self-Review Notes

- Spec coverage: plan covers private `snapshot` module creation, facade re-export, moved helper inventory, dead-code allowance, private helper visibility, import cleanup, worktree guard, baseline/post-change test slices, source guards, Cargo check, format check, and commit hygiene.
- Placeholder scan: the forbidden-pattern scan was run and no placeholder markers remain.
- Type consistency: `CorpusMessage`, `StoredRunSnapshotRow`, `Pool<Sqlite>`, `AppResult`, `sanitize_snapshot_error`, `sanitize_provider_error`, `capture_run_snapshot`, `persist_run_snapshot`, and `mark_run_capture_failed` match the approved design and current `store.rs`.
