# Analysis Store Setup Refactor Design

**Date:** 2026-07-03
**Status:** active spec; implementation not started as of 2026-07-03 because `src-tauri/src/analysis/store/setup.rs` does not exist
**Scope:** internal Rust refactor of analysis store prompt-template setup, source existence validation, prompt-template fetch, and source-group fetch logic.

## Goal

Reduce the responsibility of `src-tauri/src/analysis/store.rs` by extracting analysis setup and lookup helpers into a focused private child module, without changing builtin template initialization, source validation, source-group loading, prompt-template fetch behavior, facade imports, command behavior, tests, database schema, or user-facing error messages.

This is the next conservative backend slice after the read-model and snapshot extractions. It intentionally avoids moving duplicate-run lookup, analysis run insertion, run status mutation, saved-run deletion, read-model logic, snapshot logic, or store tests.

## Current Shape

`src-tauri/src/analysis/store.rs` currently owns:

- read-model facade re-exports from `store/read_model.rs`;
- snapshot facade re-exports from `store/snapshot.rs`;
- builtin prompt-template initialization;
- source existence validation;
- prompt-template fetch;
- source-group fetch;
- duplicate-run lookup and analysis run insertion;
- run status mutation and saved-run deletion;
- inline tests for all of the above.

The setup cluster currently lives directly in `store.rs`:

- `builtin_report_template_exists`
- `ensure_builtin_report_template`
- `ensure_sources_exist`
- `fetch_prompt_template`
- `fetch_source_group`

Current consumers:

- `analysis/mod.rs` tests import `ensure_builtin_report_template` through `analysis::store`;
- `analysis/templates.rs` imports `ensure_builtin_report_template` through `analysis::store`;
- `analysis/groups.rs` imports `ensure_sources_exist` and `fetch_source_group` through `analysis::store`;
- `analysis/report.rs` imports `fetch_prompt_template` and `fetch_source_group` through `analysis::store`;
- `analysis/corpus/source_resolution.rs` imports `fetch_source_group` through `analysis::store`;
- `store.rs` tests call `ensure_sources_exist` and `fetch_prompt_template` through the parent facade.

## Proposed Architecture

Create a private child module declared from `src-tauri/src/analysis/store.rs`:

- `src-tauri/src/analysis/store/setup.rs`

Keep `src-tauri/src/analysis/store.rs` as the store facade:

- add `mod setup;`;
- re-export the existing setup API from `store.rs`:

```rust
pub(crate) use self::setup::{
    ensure_builtin_report_template, ensure_sources_exist, fetch_prompt_template,
    fetch_source_group,
};
```

- do not change imports in external consumers in this slice;
- keep `setup` private to `analysis::store`;
- keep store tests in `store.rs` for this slice.

Move these items from `store.rs` to `store/setup.rs`:

- `builtin_report_template_exists`
- `ensure_builtin_report_template`
- `ensure_sources_exist`
- `fetch_prompt_template`
- `fetch_source_group`

Keep these items in `store.rs` for this slice:

- read-model facade declarations and re-exports;
- snapshot facade declarations and re-exports;
- `DuplicateRunLookup`
- `find_active_duplicate_run`
- `AnalysisRunInsert`
- `insert_analysis_run`
- `set_run_status`
- `delete_saved_run`
- all current tests.

The inline test module stays in `store.rs` for this slice. Moving store tests can be a later test-only refactor.

## Visibility

`store/setup.rs` should expose only the existing setup API consumed through `analysis::store`:

```rust
pub(crate) async fn ensure_builtin_report_template(pool: &Pool<Sqlite>) -> AppResult<()>;

pub(crate) async fn ensure_sources_exist(pool: &Pool<Sqlite>, source_ids: &[i64]) -> AppResult<()>;

pub(crate) async fn fetch_prompt_template(
    pool: &Pool<Sqlite>,
    template_id: i64,
) -> AppResult<AnalysisPromptTemplate>;

pub(crate) async fn fetch_source_group(
    pool: &Pool<Sqlite>,
    group_id: i64,
) -> AppResult<Option<AnalysisSourceGroup>>;
```

Private helpers stay private inside `setup.rs`:

- `builtin_report_template_exists`

Expected production API changes outside `analysis::store`: none.

Expected root re-export changes in `analysis/mod.rs`: none.

## Imports

`store/setup.rs` should own imports needed by setup and source-group lookup logic:

- `sqlx::{Pool, Sqlite}`
- `super::super::models::{AnalysisPromptTemplate, AnalysisSourceGroup, AnalysisSourceGroupMember, AnalysisSourceGroupRow}`
- `super::super::{default_report_template_body, now_secs, DEFAULT_REPORT_TEMPLATE_NAME, TEMPLATE_KIND_REPORT}`
- `crate::error::{AppError, AppResult}`

`store.rs` should remove imports that only moved setup helpers use after extraction:

- `AnalysisSourceGroup`, if only `setup.rs` uses it;
- `AnalysisSourceGroupMember`, if only `setup.rs` uses it;
- `AnalysisSourceGroupRow`, if only `setup.rs` uses it;
- `default_report_template_body`, if only `setup.rs` uses it;
- `now_secs`, if only `setup.rs` uses it;
- `DEFAULT_REPORT_TEMPLATE_NAME`, if only `setup.rs` uses it;
- `TEMPLATE_KIND_REPORT`, if only `setup.rs` uses it.

Keep `AnalysisPromptTemplate` in `store.rs` because `AnalysisRunInsert` still owns a `prompt_template: &'a AnalysisPromptTemplate` field after this slice. Keep in `store.rs` imports needed by duplicate-run lookup, insert, status, delete, read-model facade, snapshot facade, and tests. Test-only imports can remain inside the inline `#[cfg(test)] mod tests`.

The implementation plan must include a production-import guard that checks the section of `store.rs` before `#[cfg(test)] mod tests`; moved-only imports must not remain in the production import block.

## Data Flow

No runtime data flow changes:

1. `analysis/templates.rs` still calls `store::ensure_builtin_report_template` through the same path before listing templates.
2. `analysis/groups.rs` still calls `store::ensure_sources_exist` and `store::fetch_source_group` through the same paths.
3. `analysis/report.rs` still calls `store::fetch_prompt_template` and `store::fetch_source_group` through the same paths.
4. `analysis/corpus/source_resolution.rs` still calls `store::fetch_source_group` through the same path.
5. `ensure_builtin_report_template` still checks for an existing builtin report template before inserting one.
6. Builtin report template insertion still uses `DEFAULT_REPORT_TEMPLATE_NAME`, `TEMPLATE_KIND_REPORT`, `default_report_template_body()`, version `1`, `is_builtin = 1`, and the same `now_secs()` timestamp for `created_at` and `updated_at`.
7. `ensure_sources_exist` still checks each source id individually and returns `Source {source_id} not found` for the first missing id.
8. `fetch_prompt_template` still calls `ensure_builtin_report_template` first, then fetches by id, and returns `Analysis prompt template {template_id} not found` when missing.
9. `fetch_source_group` still fetches the group row, returns `Ok(None)` when missing, fetches members with item counts, orders by `COALESCE(sources.title, ''), sources.id`, and returns `AnalysisSourceGroup` with the same fields.

## Error Handling

Preserve current error behavior exactly:

- builtin template existence and insertion database failures still use `AppError::database`;
- source existence database failures still use `AppError::database`;
- missing source ids still return `AppError::not_found(format!("Source {source_id} not found"))`;
- missing prompt templates still return `AppError::not_found(format!("Analysis prompt template {template_id} not found"))`;
- missing source groups still return `Ok(None)` from `fetch_source_group`;
- source-group database failures still use `AppError::database`;
- no new error codes, messages, SQL filters, DTO fields, migrations, or user-facing strings are introduced.

The implementation plan must include source guards for these literals and SQL fragments:

```powershell
rg -n -F "SELECT EXISTS(" src-tauri/src/analysis/store/setup.rs
rg -n -F "DEFAULT_REPORT_TEMPLATE_NAME" src-tauri/src/analysis/store/setup.rs
rg -n -F "Source {source_id} not found" src-tauri/src/analysis/store/setup.rs
rg -n -F "Analysis prompt template {template_id} not found" src-tauri/src/analysis/store/setup.rs
rg -n -F "ORDER BY COALESCE(sources.title, ''), sources.id" src-tauri/src/analysis/store/setup.rs
```

Expected: all setup and source-group behavior markers are present in `setup.rs` after extraction.

## Non-Goals

This slice does not:

- move duplicate-run lookup;
- move analysis run insertion;
- move run status mutation or saved-run deletion;
- move read-model logic or `store/read_model.rs`;
- move snapshot logic or `store/snapshot.rs`;
- split store tests into files;
- change SQL, DTO mappings, builtin template body, default template name, template kind, source-group member ordering, item-count logic, database schema, migrations, frontend code, Tauri command payloads, or event payloads.

## Implementation Hygiene

The implementation plan must include a pre-edit worktree guard:

```powershell
git status --short --untracked-files=all
```

Target files must be clean before editing. If `src-tauri/src/analysis/store.rs` or `src-tauri/src/analysis/store/setup.rs` has pre-existing tracked, staged, modified, or untracked work, stop and make a separate baseline commit before starting this refactor. This is required because the implementation plan should use full-file staging for the two target Rust files.

Inspect tracked target-file diffs before editing:

```powershell
git diff -- src-tauri/src/analysis/store.rs
git diff --cached -- src-tauri/src/analysis/store.rs
```

If `src-tauri/src/analysis/store/setup.rs` exists before execution, inspect it directly because untracked files are not shown by normal `git diff`:

```powershell
if (Test-Path -LiteralPath 'src-tauri/src/analysis/store/setup.rs') {
    git status --short --untracked-files=all -- src-tauri/src/analysis/store/setup.rs
    Get-Item -LiteralPath 'src-tauri/src/analysis/store/setup.rs' | Select-Object FullName, Length, LastWriteTime
    Get-FileHash -Algorithm SHA256 -LiteralPath 'src-tauri/src/analysis/store/setup.rs'
    Get-Content -Raw -LiteralPath 'src-tauri/src/analysis/store/setup.rs'
}
```

Do not stage unrelated dirty files, such as local tool settings. Unrelated dirty files must remain unstaged and must be accounted for in baseline/final status comparisons.

The implementation plan must capture pre-edit status using a unique tag and persist the paths for later PowerShell sessions:

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

After formatting, checks, and commit, compare final status against the captured baseline by reloading the pointer file.

## Testing

Run commands from the repository root.

Before editing, establish the current baseline:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::ensure_sources_exist
```

Expected: PASS and not a green `0 tests` run.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::fetch_prompt_template
```

Expected: PASS and not a green `0 tests` run.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::
```

Expected: PASS and not a green `0 tests` run.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::groups::tests::
```

Expected: PASS and not a green `0 tests` run.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::source_resolution::tests::
```

Expected: PASS and not a green `0 tests` run.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::
```

Expected: PASS and not a green `0 tests` run.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::tests::builtin_template_is_seeded_once
```

Expected: PASS and not a green `0 tests` run.

There is no required dedicated `analysis/templates.rs` runtime test in this slice. This is an accepted risk for this move-only refactor: `analysis::tests::builtin_template_is_seeded_once` covers builtin insertion behavior, `cargo check --all-targets` covers the `templates.rs` import/type boundary, and the moved function path stays behind the same `analysis::store` facade.

After editing and before committing, run each command separately with the same non-zero expectations. Do not paste these as one PowerShell block unless the block explicitly checks `$LASTEXITCODE` after every native command and stops on failure.

Also run consumer compile coverage:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
```

Expected: PASS. This covers production imports in `analysis/mod.rs`, `analysis/templates.rs`, `analysis/groups.rs`, `analysis/report.rs`, and `analysis/corpus/source_resolution.rs`. Existing warnings outside touched files may remain; new warnings mentioning `src-tauri/src/analysis/store.rs` or `src-tauri/src/analysis/store/setup.rs` are not acceptable.

Also run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
```

Expected: PASS with no diff output.

The implementation plan must include source guards:

```powershell
rg -n "^\s*(pub\([^)]*\)\s+|pub\s+)?(fn|async fn) (builtin_report_template_exists|ensure_builtin_report_template|ensure_sources_exist|fetch_prompt_template|fetch_source_group)\b" src-tauri/src/analysis/store.rs
rg -n "^mod setup;" src-tauri/src/analysis/store.rs
rg -n "^pub.*mod setup" src-tauri/src/analysis/store.rs
rg -n "^pub\(crate\) use self::setup::" src-tauri/src/analysis/store.rs
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
rg -n "^pub\(crate\) async fn (ensure_builtin_report_template|ensure_sources_exist|fetch_prompt_template|fetch_source_group)\b" src-tauri/src/analysis/store/setup.rs
rg -n "^async fn builtin_report_template_exists\b" src-tauri/src/analysis/store/setup.rs
rg -n "^\s*pub(\([^)]*\))?\s+async fn builtin_report_template_exists\b" src-tauri/src/analysis/store/setup.rs
$storeProduction = [regex]::Split((Get-Content -Raw -LiteralPath 'src-tauri/src/analysis/store.rs'), "#\[cfg\(test\)\]\s*mod tests", 2)[0]
foreach ($name in @('AnalysisSourceGroup', 'AnalysisSourceGroupMember', 'AnalysisSourceGroupRow', 'default_report_template_body', 'now_secs', 'DEFAULT_REPORT_TEMPLATE_NAME', 'TEMPLATE_KIND_REPORT')) {
    if ($storeProduction -match ("\b" + [regex]::Escape($name) + "\b")) {
        throw "moved-only production import remains in store.rs: $name"
    }
}
if ($storeProduction -notmatch "\bAnalysisPromptTemplate\b") {
    throw "AnalysisPromptTemplate should remain in store.rs for AnalysisRunInsert"
}
$storeTests = [regex]::Match((Get-Content -Raw -LiteralPath 'src-tauri/src/analysis/store.rs'), "(?s)#\[cfg\(test\)\]\s*mod tests \{.*\z").Value
if ($storeTests -match "super::setup|setup::") {
    throw "store tests must use the store facade, not setup directly"
}
```

Expected: first command has no matches; `rg` exit code `1` is expected for this no-match guard. The second command prints exactly one private module declaration. The third command has no matches; `rg` exit code `1` is expected. The facade loop completes without throwing. Public API guards print the four re-exported setup API items. The private-helper positive guard prints `builtin_report_template_exists`. The private-helper widening guard has no matches; `rg` exit code `1` is expected. The production-import guard completes without throwing and confirms `AnalysisPromptTemplate` remains available for `AnalysisRunInsert`. The store-test guard completes without throwing, proving inline tests still exercise the `store.rs` facade instead of `super::setup`.

## Commit Shape

Expected implementation-owned files:

- `src-tauri/src/analysis/store.rs`
- `src-tauri/src/analysis/store/setup.rs`

Expected implementation commit:

```text
refactor: extract analysis store setup logic
```

The design spec and implementation plan should be committed separately from the Rust refactor.
