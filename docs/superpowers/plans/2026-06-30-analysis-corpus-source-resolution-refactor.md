# Analysis Corpus Source Resolution Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract analysis source-scope resolution from `src-tauri/src/analysis/corpus.rs` into `src-tauri/src/analysis/corpus/source_resolution.rs` without changing behavior, caller paths, SQL, or command contracts.

**Architecture:** Keep `analysis::corpus` as the internal facade. Add a private nested `source_resolution` module, move source selection, playlist expansion, and source-resolution error types into it, and re-export only the crate-visible surface current callers already use. Leave live corpus loading, preflight logic, snapshot facade re-exports, the shared sqlite test harness, and all existing tests in `corpus.rs`.

**Tech Stack:** Rust, Tauri backend, SQLx SQLite queries, Cargo test/check/fmt.

## Global Constraints

- Internal Rust refactor only: no frontend, Tauri command payload, event payload, SQL schema, migration, database query, or user-facing string changes.
- Preserve existing caller paths through `super::corpus`, `self::corpus`, and the current root re-exports in `analysis/mod.rs`; do not make callers import from `corpus::source_resolution`.
- `src-tauri/src/analysis/corpus/source_resolution.rs` must remain a private nested module.
- The shared sqlite test harness and all existing source-resolution tests stay in `src-tauri/src/analysis/corpus.rs` for this slice.
- Preserve single-source, source-group, project, and YouTube playlist expansion behavior exactly, including project ordering by `ps.added_at ASC, s.id ASC`.
- Preserve `AnalysisSourceResolutionErrorCode` values, messages, and `AppError` mapping exactly.
- Preserve crate-visible field and method visibility for `ResolvedAnalysisSources`, `AnalysisSourceResolutionErrorCode`, and `AnalysisSourceResolutionError`.
- Preserve `#[allow(dead_code)]` on `ResolvedAnalysisSources.skipped_unlinked_playlist_items` unless a real non-test reader is added in the same implementation.
- Keep `resolve_run_source_ids` test-only and re-export it from `corpus.rs` behind `#[cfg(test)]`.
- Run commands from the repository root with `--manifest-path src-tauri/Cargo.toml`.
- Every filtered `cargo test` command in this plan must run real tests, not green `0 tests` runs.
- Use `cargo fmt --manifest-path src-tauri/Cargo.toml`; after formatting inspect `git status --short` and keep unrelated rustfmt drift out of the refactor commit.
- Expected behavioral Rust changes for this slice are limited to `src-tauri/src/analysis/corpus.rs` and `src-tauri/src/analysis/corpus/source_resolution.rs`.

---

## File Structure

- Create `src-tauri/src/analysis/corpus/source_resolution.rs`
  - Owns `ResolvedAnalysisSources`, `AnalysisSourceResolutionErrorCode`, `AnalysisSourceResolutionError`, source-scope row loading, source-group/project resolution, YouTube playlist expansion, skipped unlinked playlist counting, and test-only run source resolution.
- Modify `src-tauri/src/analysis/corpus.rs`
  - Adds `mod source_resolution;`.
  - Re-exports the existing crate-visible source-resolution API from `source_resolution`.
  - Removes moved source-resolution definitions and imports.
  - Keeps `YoutubeCorpusMode`, `CorpusLoadRequest`, live corpus loaders, preflight helpers, snapshot facade re-exports, tests, and the shared sqlite test harness.
- No changes expected in `src-tauri/src/analysis/mod.rs`, `src-tauri/src/analysis/report.rs`, `src-tauri/src/projects/data_range.rs`, or `src-tauri/src/analysis/report_commands.rs`.
  - If any of those files need edits, stop and review why the `analysis::corpus` facade did not preserve existing paths.

---

### Task 1: Extract Source Resolution Behind The Corpus Facade

**Files:**
- Create: `src-tauri/src/analysis/corpus/source_resolution.rs`
- Modify: `src-tauri/src/analysis/corpus.rs`

**Interfaces:**
- Does not consume `YoutubeCorpusMode`; that enum remains in `corpus.rs` with live corpus loading and preflight logic.

- Consumes from `crate::analysis::models` only for test-only run source resolution:

```rust
#[cfg(test)]
AnalysisRunDetail
```

- Consumes from `crate::analysis::store`:

```rust
fetch_source_group
```

- Produces through the `src-tauri/src/analysis/corpus.rs` facade:

```rust
pub(crate) async fn resolve_analysis_sources(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source_id: Option<i64>,
    source_group_id: Option<i64>,
    project_id: Option<i64>,
) -> Result<ResolvedAnalysisSources, AnalysisSourceResolutionError>;

#[cfg(test)]
pub(crate) async fn resolve_run_source_ids(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    run: &crate::analysis::models::AnalysisRunDetail,
) -> Result<Vec<i64>, String>;

pub(crate) struct ResolvedAnalysisSources {
    pub(crate) source_type: String,
    pub(crate) source_ids: Vec<i64>,
    #[allow(dead_code)]
    pub(crate) skipped_unlinked_playlist_items: usize,
}

pub(crate) enum AnalysisSourceResolutionErrorCode {
    MixedProviderProject,
    NoLinkedYoutubeVideos,
}

pub(crate) struct AnalysisSourceResolutionError;
```

- Source-resolution-only helpers remain private inside `source_resolution.rs`:

```rust
struct AnalysisSourceScopeRow;

async fn load_source_scope_row(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source_id: i64,
) -> crate::error::AppResult<AnalysisSourceScopeRow>;

async fn linked_playlist_video_source_ids(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    playlist_source_id: i64,
) -> crate::error::AppResult<Vec<i64>>;

async fn count_skipped_unlinked_playlist_items(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    playlist_source_id: i64,
) -> crate::error::AppResult<usize>;

async fn push_scope_source(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source: AnalysisSourceScopeRow,
    source_ids: &mut Vec<i64>,
    seen_source_ids: &mut std::collections::HashSet<i64>,
    skipped_unlinked_playlist_items: &mut usize,
) -> crate::error::AppResult<()>;
```

- [ ] **Step 1: Run source-resolution characterization tests before editing**

Run each command separately:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::playlist_expansion_excludes_unlinked_and_removed_rows
```

Expected: PASS with output containing `1 passed`.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::resolve_analysis_sources
```

Expected: PASS and not a green `0 tests` run. The current test filter should run the three tests whose names start with `analysis::corpus::tests::resolve_analysis_sources_`.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::resolve_run_source_ids
```

Expected: PASS and not a green `0 tests` run. The current test filter should run the two tests whose names start with `analysis::corpus::tests::resolve_run_source_ids_`.

If any characterization test fails before editing, stop and inspect the existing failure before moving code.

- [ ] **Step 2: Create the nested module file with source-resolution imports**

Create `src-tauri/src/analysis/corpus/source_resolution.rs` with this header:

```rust
use std::collections::HashSet;

use sqlx::{Pool, Sqlite};

#[cfg(test)]
use super::super::{
    ANALYSIS_SCOPE_TYPE_PROJECT, ANALYSIS_SCOPE_TYPE_SINGLE_SOURCE,
    ANALYSIS_SCOPE_TYPE_SOURCE_GROUP,
};
#[cfg(test)]
use crate::analysis::models::AnalysisRunDetail;
use crate::analysis::store::fetch_source_group;
use crate::error::{AppError, AppResult};
```

- [ ] **Step 3: Move source-resolution types and error mapping into `source_resolution.rs`**

Move this block from `src-tauri/src/analysis/corpus.rs` into `src-tauri/src/analysis/corpus/source_resolution.rs`, immediately after the imports:

```rust
#[derive(Debug)]
pub(crate) struct ResolvedAnalysisSources {
    pub(crate) source_type: String,
    pub(crate) source_ids: Vec<i64>,
    #[allow(dead_code)]
    pub(crate) skipped_unlinked_playlist_items: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AnalysisSourceResolutionErrorCode {
    MixedProviderProject,
    NoLinkedYoutubeVideos,
}

impl AnalysisSourceResolutionErrorCode {
    pub(crate) fn message(self) -> &'static str {
        match self {
            Self::MixedProviderProject => "mixed_provider_project_runs_not_supported",
            Self::NoLinkedYoutubeVideos => {
                "No linked YouTube videos are available for analysis in this scope"
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AnalysisSourceResolutionError {
    code: Option<AnalysisSourceResolutionErrorCode>,
    error: AppError,
}

impl AnalysisSourceResolutionError {
    pub(crate) fn validation(code: AnalysisSourceResolutionErrorCode) -> Self {
        Self {
            code: Some(code),
            error: AppError::validation(code.message()),
        }
    }

    pub(crate) fn code(&self) -> Option<AnalysisSourceResolutionErrorCode> {
        self.code
    }

    pub(crate) fn into_app_error(self) -> AppError {
        self.error
    }
}

impl From<AppError> for AnalysisSourceResolutionError {
    fn from(error: AppError) -> Self {
        Self { code: None, error }
    }
}
```

Do not move `estimate_message_input_chars`, `live_corpus_ref`, or `estimate_preflight_chunk_count`; they stay in `corpus.rs`.

- [ ] **Step 4: Move source-resolution helpers and functions into `source_resolution.rs`**

Move this contiguous block from `src-tauri/src/analysis/corpus.rs` into `src-tauri/src/analysis/corpus/source_resolution.rs`, immediately after the error mapping block from Step 3:

```rust
#[derive(sqlx::FromRow)]
struct AnalysisSourceScopeRow {
    id: i64,
    source_type: String,
    source_subtype: Option<String>,
}

async fn load_source_scope_row(
    pool: &Pool<Sqlite>,
    source_id: i64,
) -> AppResult<AnalysisSourceScopeRow>

async fn linked_playlist_video_source_ids(
    pool: &Pool<Sqlite>,
    playlist_source_id: i64,
) -> AppResult<Vec<i64>>

async fn count_skipped_unlinked_playlist_items(
    pool: &Pool<Sqlite>,
    playlist_source_id: i64,
) -> AppResult<usize>

pub(crate) async fn resolve_analysis_sources(
    pool: &Pool<Sqlite>,
    source_id: Option<i64>,
    source_group_id: Option<i64>,
    project_id: Option<i64>,
) -> Result<ResolvedAnalysisSources, AnalysisSourceResolutionError>

async fn push_scope_source(
    pool: &Pool<Sqlite>,
    source: AnalysisSourceScopeRow,
    source_ids: &mut Vec<i64>,
    seen_source_ids: &mut HashSet<i64>,
    skipped_unlinked_playlist_items: &mut usize,
) -> AppResult<()>

#[cfg(test)]
pub(crate) async fn resolve_run_source_ids(
    pool: &Pool<Sqlite>,
    run: &AnalysisRunDetail,
) -> Result<Vec<i64>, String>
```

Move each full function body verbatim. Do not change SQL text, `ORDER BY` clauses, playlist filtering, error strings, `HashSet` de-duplication, or `resolve_run_source_ids` fallback ordering.

- [ ] **Step 5: Add the private module and facade re-exports in `corpus.rs`**

At the top of `src-tauri/src/analysis/corpus.rs`, keep `mod snapshot;` and add `mod source_resolution;`. After import cleanup, the file should start like this:

```rust
mod snapshot;
mod source_resolution;

#[allow(unused_imports)]
pub(crate) use self::snapshot::load_run_corpus_messages;
pub(crate) use self::snapshot::{
    list_run_snapshot_messages_page, load_run_snapshot_messages, load_trace_resolution_messages,
    ListRunSnapshotMessagesRequest,
};
pub(crate) use self::source_resolution::{
    resolve_analysis_sources, AnalysisSourceResolutionError, AnalysisSourceResolutionErrorCode,
};
#[allow(unused_imports)]
pub(crate) use self::source_resolution::ResolvedAnalysisSources;
#[cfg(test)]
pub(crate) use self::source_resolution::resolve_run_source_ids;
use sqlx::{Pool, QueryBuilder, Sqlite};

use super::models::CorpusMessage;
use crate::compression::{compress_json_bytes, decompress_text};
use crate::error::{internal_error, AppError, AppResult};
```

The targeted `#[allow(unused_imports)]` on `ResolvedAnalysisSources` preserves the current facade-visible type name without introducing warning debt if callers continue to rely on inference instead of importing the type by name.

- [ ] **Step 6: Remove moved-only imports and definitions from `corpus.rs`**

Remove these top-level imports from `src-tauri/src/analysis/corpus.rs` because the moved source-resolution module now owns them:

```rust
use std::collections::HashSet;

#[cfg(test)]
use super::models::AnalysisRunDetail;
use super::store::fetch_source_group;
#[cfg(test)]
use super::{
    ANALYSIS_SCOPE_TYPE_PROJECT, ANALYSIS_SCOPE_TYPE_SINGLE_SOURCE,
    ANALYSIS_SCOPE_TYPE_SOURCE_GROUP,
};
```

Confirm these production imports remain in `corpus.rs`:

```rust
use sqlx::{Pool, QueryBuilder, Sqlite};

use super::models::CorpusMessage;
use crate::compression::{compress_json_bytes, decompress_text};
use crate::error::{internal_error, AppError, AppResult};
```

Remove the moved definitions from `corpus.rs` after they exist in `source_resolution.rs`. `corpus.rs` must still contain `YoutubeCorpusMode`, `CorpusLoadRequest`, `estimate_message_input_chars`, `live_corpus_ref`, `estimate_preflight_chunk_count`, live corpus loading, preflight logic, snapshot facade re-exports, and the `#[cfg(test)] mod tests` block.

- [ ] **Step 7: Run rustfmt and inspect the touched files**

Run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml
```

Then run:

```powershell
git status --short
```

Expected `git status --short` output for this task:

```text
 M src-tauri/src/analysis/corpus.rs
?? src-tauri/src/analysis/corpus/source_resolution.rs
```

If unrelated Rust files appear, do not stage them in the refactor commit. Either leave them unstaged for review or make a separate format-only commit before continuing.

- [ ] **Step 8: Run focused source-resolution tests after editing**

Run each command separately:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::playlist_expansion_excludes_unlinked_and_removed_rows
```

Expected: PASS with output containing `1 passed`.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::resolve_analysis_sources
```

Expected: PASS and not a green `0 tests` run. The output should include:

```text
analysis::corpus::tests::resolve_analysis_sources_rejects_mixed_provider_project
analysis::corpus::tests::resolve_analysis_sources_preserves_no_linked_youtube_error_message
analysis::corpus::tests::resolve_analysis_sources_loads_single_provider_project
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::resolve_run_source_ids
```

Expected: PASS and not a green `0 tests` run. The output should include:

```text
analysis::corpus::tests::resolve_run_source_ids_prefers_snapshot_over_live_group_membership
analysis::corpus::tests::resolve_run_source_ids_loads_project_sources_without_snapshot
```

- [ ] **Step 9: Run module-boundary compile coverage**

Run:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
```

Expected: PASS. Existing warnings outside the touched files may remain; new warnings mentioning `src/analysis/corpus.rs` or `src/analysis/corpus/source_resolution.rs` are not acceptable.

- [ ] **Step 10: Commit the source-resolution extraction**

Run:

```powershell
git add -- src-tauri/src/analysis/corpus.rs src-tauri/src/analysis/corpus/source_resolution.rs
```

Then run:

```powershell
git commit -m "refactor: extract analysis corpus source resolution"
```

Expected: commit succeeds and includes only `src-tauri/src/analysis/corpus.rs` and `src-tauri/src/analysis/corpus/source_resolution.rs`.

---

### Task 2: Final Regression Verification

**Files:**
- Verify: `src-tauri/src/analysis/corpus.rs`
- Verify: `src-tauri/src/analysis/corpus/source_resolution.rs`
- Verify: `src-tauri/src/analysis/report.rs`
- Verify: `src-tauri/src/projects/data_range.rs`
- Verify: `src-tauri/src/analysis/mod.rs`

**Interfaces:**
- Confirms the `analysis::corpus` facade still satisfies current callers:
  - `analysis/report.rs`: `resolve_analysis_sources` and `AnalysisSourceResolutionError`
  - `projects/data_range.rs`: root re-exported `resolve_analysis_sources` and `AnalysisSourceResolutionErrorCode`
  - `analysis/mod.rs`: root re-exports of `resolve_analysis_sources`, `AnalysisSourceResolutionError`, and `AnalysisSourceResolutionErrorCode`
  - `analysis/corpus.rs` tests: facade-visible `resolve_analysis_sources` and test-only `resolve_run_source_ids`

- [ ] **Step 1: Run the full corpus test module**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::
```

Expected: PASS and not a green `0 tests` run. The current baseline is `43 passed`.

- [ ] **Step 2: Run report consumer tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::
```

Expected: PASS and not a green `0 tests` run. This covers the `analysis/report.rs` import path through `super::corpus`.

- [ ] **Step 3: Run project data-range consumer tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml projects::data_range::tests::
```

Expected: PASS and not a green `0 tests` run. The current module contains eight filtered tests.

- [ ] **Step 4: Run format check**

Run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
```

Expected: PASS with no diff output.

- [ ] **Step 5: Run all-target compile coverage**

Run:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
```

Expected: PASS. Existing warnings outside the touched files may remain; new warnings mentioning `src/analysis/corpus.rs` or `src/analysis/corpus/source_resolution.rs` are not acceptable.

- [ ] **Step 6: Confirm final git state**

Run:

```powershell
git status --short
```

Expected: `git status --short` is empty after the refactor commit. If verification discovers a required fix, make that fix in a separate commit or clearly document why the refactor commit was updated.

Run:

```powershell
git log --oneline -3
```

Expected: the latest commits include:

```text
refactor: extract analysis corpus source resolution
```
