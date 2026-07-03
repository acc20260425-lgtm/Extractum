# Analysis Store Read Model Refactor Design

**Date:** 2026-07-03
**Status:** implemented historical design. Implemented by `c00d2fb2 refactor: extract analysis store read model`, verified by `f1ef1f7a chore: verify analysis store read model extraction`, and recorded complete by `06422079 docs: complete analysis store read model plan`.
**Scope:** internal Rust refactor of analysis run read-model mapping and list-query logic.

## Goal

Reduce the responsibility of `src-tauri/src/analysis/store.rs` by extracting analysis run read-model mapping, filter parsing, list-query construction, and run-row loading into a focused private child module, without changing database queries, mapped DTO fields, snapshot-state behavior, filtering semantics, public Tauri command behavior, or mutation/snapshot persistence logic.

This is the next conservative backend slice after the report module extractions. It intentionally avoids moving write paths, snapshot transactions, prompt-template storage, duplicate-run lookup, source-group loading, or status mutation so the extraction remains centered on read-only run list/detail behavior.

## Original Shape

`src-tauri/src/analysis/store.rs` currently owns several unrelated groups:

- prompt-template initialization and source existence checks;
- analysis run read-model mapping and list/detail queries;
- duplicate-run lookup and analysis run insertion;
- snapshot capture/persistence and capture failure marking;
- run status mutation and saved-run deletion;
- tests for list filters, mapping, snapshot state, snapshot persistence, and status behavior.

The read-model cluster is currently:

- `resolve_run_scope_label_parts`
- `resolve_run_row_scope_label`
- `compute_snapshot_state`
- `map_run_summary`
- `map_run_detail`
- `AnalysisRunListFilters`
- `ANALYSIS_RUN_LIST_SELECT`
- `RUN_QUERY_FIELDS`
- `trimmed_filter`
- `escaped_like_contains`
- `parse_yyyy_mm_dd_midnight`
- `parse_yyyy_mm_dd_day_end`
- `push_like_predicate`
- `push_search_term_predicate`
- `list_analysis_run_summaries`
- `fetch_run_row`
- `resolve_run_scope_label`

Current consumers:

- `analysis/mod.rs` imports `AnalysisRunListFilters`, `fetch_run_row`, `list_analysis_run_summaries`, `map_run_detail`, and `map_run_summary` through `analysis::store`;
- `analysis/chat.rs` imports `fetch_run_row`, `map_run_detail`, and `resolve_run_scope_label` through `analysis::store`;
- `analysis/report/lifecycle.rs` imports `fetch_run_row` through `analysis::store`;
- `analysis/fixtures.rs` tests use `fetch_run_row`, `map_run_detail`, `list_analysis_run_summaries`, and `AnalysisRunListFilters` through `crate::analysis::store`;
- `store.rs` tests call read-model helpers directly through the parent module.

## Proposed Architecture

Create a private child module declared from `src-tauri/src/analysis/store.rs`:

- `src-tauri/src/analysis/store/read_model.rs`

Keep `src-tauri/src/analysis/store.rs` as the store facade:

- add `mod read_model;`;
- re-export the existing read-model API from `store.rs`:

```rust
pub(crate) use self::read_model::{
    fetch_run_row, list_analysis_run_summaries, map_run_detail, map_run_summary,
    resolve_run_scope_label, AnalysisRunListFilters,
};
```

- do not change imports in external consumers in this slice;
- keep `read_model` private to `analysis::store`;
- keep prompt-template, source-group, duplicate-run, insertion, snapshot, status, delete, and tests in their current files for this slice.

Move these items from `store.rs` to `store/read_model.rs`:

- `resolve_run_scope_label_parts`
- `resolve_run_row_scope_label`
- `compute_snapshot_state`
- `map_run_summary`
- `map_run_detail`
- `AnalysisRunListFilters`
- `ANALYSIS_RUN_LIST_SELECT`
- `RUN_QUERY_FIELDS`
- `trimmed_filter`
- `escaped_like_contains`
- `parse_yyyy_mm_dd_midnight`
- `parse_yyyy_mm_dd_day_end`
- `push_like_predicate`
- `push_search_term_predicate`
- `list_analysis_run_summaries`
- `fetch_run_row`
- `resolve_run_scope_label`

Keep these items in `store.rs` for this slice:

- `builtin_report_template_exists`
- `ensure_builtin_report_template`
- `ensure_sources_exist`
- `fetch_prompt_template`
- `fetch_source_group`
- `DuplicateRunLookup`
- `find_active_duplicate_run`
- `AnalysisRunInsert`
- `insert_analysis_run`
- `sanitize_snapshot_error`
- `sanitize_provider_error`
- `validate_snapshot_message`
- `load_run_snapshot_messages_on_transaction`
- `capture_run_snapshot`
- `persist_run_snapshot`
- `mark_run_capture_failed`
- `set_run_status`
- `delete_saved_run`
- all current tests.

The inline test module stays in `store.rs` for this slice. Moving store tests can be a later test-only refactor if needed.

## Visibility

`store/read_model.rs` should expose only the existing read-model API consumed through `analysis::store`:

```rust
pub(crate) fn map_run_summary(row: AnalysisRunRow) -> AnalysisRunSummary;

pub(crate) fn map_run_detail(row: AnalysisRunRow) -> AnalysisRunDetail;

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

pub(crate) async fn list_analysis_run_summaries(
    pool: &Pool<Sqlite>,
    filters: AnalysisRunListFilters,
) -> AppResult<Vec<AnalysisRunSummary>>;

pub(crate) async fn fetch_run_row(
    pool: &Pool<Sqlite>,
    run_id: i64,
) -> AppResult<Option<AnalysisRunRow>>;

pub(crate) fn resolve_run_scope_label(run: &AnalysisRunDetail) -> String;
```

Private helpers stay private inside `read_model.rs`:

- `resolve_run_scope_label_parts`
- `resolve_run_row_scope_label`
- `compute_snapshot_state`
- `ANALYSIS_RUN_LIST_SELECT`
- `RUN_QUERY_FIELDS`
- `trimmed_filter`
- `escaped_like_contains`
- `parse_yyyy_mm_dd_midnight`
- `parse_yyyy_mm_dd_day_end`
- `push_like_predicate`
- `push_search_term_predicate`

Do not widen private helpers to `pub(crate)` unless a current non-test consumer already needs them. The implementation plan must include source guards that these helper names no longer appear in `store.rs` and that public facade re-exports exist in `store.rs`.

The implementation plan must also mechanically verify that all `AnalysisRunListFilters` fields remain `pub(crate)` after the move. This is part of the facade contract because command handlers construct filters directly through `analysis::store`; relying only on `cargo check` would find the issue late but would not document the expected field-level surface.

Expected production API changes outside `analysis::store`: none.

Expected root re-export changes in `analysis/mod.rs`: none.

## Imports

`store/read_model.rs` should own imports needed by read-model query and mapping logic:

- `sqlx::{Pool, QueryBuilder, Sqlite}`
- `super::super::models::{AnalysisRunDetail, AnalysisRunRow, AnalysisRunSummary, AnalysisSnapshotState}`
- `super::super::{ANALYSIS_SCOPE_TYPE_PROJECT, ANALYSIS_SCOPE_TYPE_SOURCE_GROUP, ANALYSIS_STATUS_CANCELLED, ANALYSIS_STATUS_COMPLETED, ANALYSIS_STATUS_FAILED, ANALYSIS_STATUS_QUEUED, ANALYSIS_STATUS_RUNNING}`
- `crate::error::{AppError, AppResult}`
- `crate::time::ymd_to_unix_midnight`

`store.rs` should remove imports that only moved read-model helpers use after extraction:

- `QueryBuilder`, if no remaining code in `store.rs` uses it;
- `AnalysisRunSummary`, if only `read_model.rs` uses it;
- `AnalysisSnapshotState`, if only `read_model.rs` uses it;
- `ymd_to_unix_midnight`, if only `read_model.rs` uses it;
- `ANALYSIS_STATUS_CANCELLED`, if only `read_model.rs` uses it.

Keep in `store.rs` imports needed by prompt-template, source-group, duplicate-run, insert, snapshot, status, delete, and tests. Test-only imports can remain in the inline `#[cfg(test)] mod tests`.

The implementation plan must verify unused imports after the move and keep the diff free of new warning debt in `src-tauri/src/analysis/store.rs` and `src-tauri/src/analysis/store/read_model.rs`.

## Data Flow

No runtime data flow changes:

1. `analysis/mod.rs` still calls `store::list_analysis_run_summaries`, `store::fetch_run_row`, `store::map_run_summary`, and `store::map_run_detail` through the same paths.
2. `analysis/chat.rs` still calls `store::fetch_run_row`, `store::map_run_detail`, and `store::resolve_run_scope_label` through the same paths.
3. `analysis/report/lifecycle.rs` still calls `store::fetch_run_row` through the same path.
4. `list_analysis_run_summaries` still validates that only one of `source_id`, `source_group_id`, or `project_id` is supplied.
5. List filters still trim empty strings, preserve `queued_running` semantics, parse `YYYY-MM-DD` date filters, escape LIKE wildcards, split search terms, apply all search terms, order by `runs.created_at DESC`, and clamp `limit` to `1..=100`.
6. `fetch_run_row` still returns the same `AnalysisRunRow` shape and joins the same source, group, project, template, and snapshot count data.
7. `map_run_summary` and `map_run_detail` still compute `scope_label`, `snapshot_state`, `has_trace_data`, `snapshot_message_count`, and YouTube/Telegram scope fields exactly as today.
8. `resolve_run_scope_label` still prefers a non-empty `run.scope_label`, then falls back through the same source/group/project/snapshot label logic.

## Error Handling

Preserve current error behavior exactly:

- multiple scope filters still return validation message `Pass only one of source_id, source_group_id, or project_id`;
- database failures still use `AppError::database`;
- invalid date filters still behave as ignored filters, because parsing returns `None`;
- empty or whitespace-only filter strings still behave as missing filters;
- LIKE search still escapes `\`, `%`, and `_` with `ESCAPE '\'`;
- no new error codes, messages, SQL filters, DTO fields, or user-facing strings are introduced.

The implementation plan must include source guards for these literals and SQL fragments:

```powershell
rg -n '"Pass only one of source_id, source_group_id, or project_id"|queued_running|ESCAPE|ORDER BY runs.created_at DESC LIMIT|COALESCE\(snapshot_counts.snapshot_message_count, 0\)' src-tauri/src/analysis/store/read_model.rs
```

Expected: all read-model filter/query behavior markers are present in `read_model.rs` after extraction.

## Non-Goals

This slice does not:

- move prompt-template initialization;
- move source existence checks;
- move source-group loading;
- move duplicate-run lookup;
- move analysis run insertion;
- move snapshot validation, capture, persistence, or capture failure marking;
- move run status mutation or saved-run deletion;
- split store tests into files;
- change SQL, DTO mappings, filters, date parsing, wildcard escaping, snapshot-state logic, database schema, migrations, frontend, Tauri command payloads, or event payloads.

## Implementation Hygiene

The implementation plan must include a pre-edit worktree guard:

```powershell
git status --short --untracked-files=all
```

If target files are already modified, staged, or untracked before this task starts, inspect the baseline before editing:

```powershell
git diff -- src-tauri/src/analysis/store.rs
git diff --cached -- src-tauri/src/analysis/store.rs
```

If `src-tauri/src/analysis/store/read_model.rs` exists before execution, inspect it directly because untracked files are not shown by normal `git diff`:

```powershell
if (Test-Path -LiteralPath 'src-tauri/src/analysis/store/read_model.rs') {
    git status --short --untracked-files=all -- src-tauri/src/analysis/store/read_model.rs
    Get-Item -LiteralPath 'src-tauri/src/analysis/store/read_model.rs' | Select-Object FullName, Length, LastWriteTime
    Get-FileHash -Algorithm SHA256 -LiteralPath 'src-tauri/src/analysis/store/read_model.rs'
    Get-Content -Raw -LiteralPath 'src-tauri/src/analysis/store/read_model.rs'
}
```

Do not stage pre-existing target-file changes into the read-model extraction commit. Unrelated dirty files, such as local tool settings, must remain unstaged and must be accounted for in baseline/final status comparisons.

The implementation plan must capture pre-edit status using a unique tag so repeated executions do not overwrite each other:

```powershell
$env:ANALYSIS_STORE_READ_MODEL_STATUS_TAG = "{0}-{1}" -f (Get-Date -Format 'yyyyMMddHHmmss'), $PID
$preEditStatusPath = Join-Path $env:TEMP "analysis-store-read-model-$env:ANALYSIS_STORE_READ_MODEL_STATUS_TAG-pre-edit-status.txt"
git status --short --untracked-files=all | Set-Content -Encoding utf8 -LiteralPath $preEditStatusPath
Get-Content -Raw -LiteralPath $preEditStatusPath
```

After formatting, checks, and commit, compare final status against the captured baseline.

## Testing

Run commands from the repository root.

Before editing, establish the current baseline:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::list_analysis_run_summaries
```

Expected: PASS and not a green `0 tests` run. This covers list filtering, query terms, scope filters, status/date filters, and LIKE escaping.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::map_run_summary
```

Expected: PASS and not a green `0 tests` run. This covers summary mapping, frozen scope labels, snapshot state, and corpus mode mapping.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::map_run_detail
```

Expected: PASS and not a green `0 tests` run. This covers detail mapping and corpus mode mapping.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::resolve_run_scope_label
```

Expected: PASS and not a green `0 tests` run. This covers scope label fallback behavior.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::
```

Expected: PASS and not a green `0 tests` run. This guards adjacent store behavior while imports move.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::
```

Expected: PASS and not a green `0 tests` run. This establishes the fixture consumer baseline for `fetch_run_row`, `map_run_detail`, `list_analysis_run_summaries`, and `AnalysisRunListFilters`.

After editing and before committing, run each command separately, with the same non-zero expectations. Do not paste these as one PowerShell block unless the block explicitly checks `$LASTEXITCODE` after every native command and stops on failure.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::list_analysis_run_summaries
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::map_run_summary
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::map_run_detail
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::resolve_run_scope_label
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::store::tests::
```

Also run the fixture consumer behavior slice:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::fixtures::tests::
```

Expected: PASS and not a green `0 tests` run. This is required consumer behavior coverage, not just compile coverage.

Also run consumer compile coverage:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
```

Expected: PASS. This covers `analysis/mod.rs`, `analysis/chat.rs`, `analysis/report/lifecycle.rs`, and fixture consumers at the import/type boundary. Existing warnings outside touched files may remain; new warnings mentioning `src-tauri/src/analysis/store.rs` or `src-tauri/src/analysis/store/read_model.rs` are not acceptable.

Also run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
```

Expected: PASS with no diff output.

The implementation plan must include source guards:

```powershell
rg -n "^\s*(pub\([^)]*\)\s+|pub\s+)?(fn|async fn|struct) (resolve_run_scope_label_parts|resolve_run_row_scope_label|compute_snapshot_state|map_run_summary|map_run_detail|AnalysisRunListFilters|trimmed_filter|escaped_like_contains|parse_yyyy_mm_dd_midnight|parse_yyyy_mm_dd_day_end|push_like_predicate|push_search_term_predicate|list_analysis_run_summaries|fetch_run_row|resolve_run_scope_label)\b|^\s*(pub\([^)]*\)\s+|pub\s+)?const (ANALYSIS_RUN_LIST_SELECT|RUN_QUERY_FIELDS)\b" src-tauri/src/analysis/store.rs
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
rg -n "^pub\(crate\) (fn|async fn|struct) (map_run_summary|map_run_detail|AnalysisRunListFilters|list_analysis_run_summaries|fetch_run_row|resolve_run_scope_label)\b" src-tauri/src/analysis/store/read_model.rs
rg -n "^\s+pub\(crate\) (source_id|source_group_id|project_id|limit|query|status|provider|model|template|date_from|date_to):" src-tauri/src/analysis/store/read_model.rs
```

Expected: first command has no matches; `rg` exit code `1` is expected for this no-match guard. The second command confirms the facade re-export anchor exists, and the PowerShell loop confirms each moved public name is inside that re-export instead of merely appearing in tests. The third command prints all moved public read-model API items in `read_model.rs`. The fourth command prints all eleven `AnalysisRunListFilters` fields; fewer matches means the field-level facade contract changed.

## Commit Shape

Expected implementation-owned files:

- `src-tauri/src/analysis/store.rs`
- `src-tauri/src/analysis/store/read_model.rs`

Expected implementation commit:

```text
refactor: extract analysis store read model
```

The design spec and implementation plan should be committed separately from the Rust refactor.
