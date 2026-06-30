# Analysis Corpus Live Loading Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract live corpus loading from `src-tauri/src/analysis/corpus.rs` into `src-tauri/src/analysis/corpus/live.rs` without changing behavior, caller paths, SQL, ordering, error mapping, or Tauri command contracts.

**Architecture:** Keep `analysis::corpus` as the internal facade. Add a private nested `live` module, move live corpus reads, migrated-history loading, metadata construction, and document-kind filtering into it, and re-export only the crate-visible live-loading surface current callers already use. Leave `CorpusLoadRequest`, `YoutubeCorpusMode`, preflight structs/helpers, snapshot/source-resolution facade re-exports, all existing tests, and the shared sqlite test harness in `corpus.rs`.

**Tech Stack:** Rust, Tauri backend, SQLx SQLite queries, zstd compression helpers, Cargo test/check/fmt.

## Global Constraints

- Internal Rust refactor only: no frontend, Tauri command payload, event payload, SQL schema, migration, database query, ordering, error mapping, or user-facing string changes.
- Preserve existing caller paths through `super::corpus`, `self::corpus`, and the current root re-exports in `analysis/mod.rs`; do not make callers import from `corpus::live`.
- `src-tauri/src/analysis/corpus/live.rs` must remain a private nested module.
- The shared sqlite test harness and all existing live-loading tests stay in `src-tauri/src/analysis/corpus.rs` for this slice.
- Preserve Telegram current and migrated-history eligibility rules exactly.
- Preserve YouTube corpus mode document-kind filtering exactly.
- Preserve live corpus ordering exactly, including Telegram merged-row sort and analysis document `ORDER BY`.
- Preserve `CorpusLoadRequest` and `YoutubeCorpusMode` in `corpus.rs`.
- Preserve all `CorpusLoadRequest` fields as `pub(crate)`.
- Preserve `YoutubeCorpusMode::from_wire`, `YoutubeCorpusMode::as_wire`, `YoutubeCorpusMode::includes_description`, and `YoutubeCorpusMode::includes_comments` as `pub(crate)`.
- Preserve the `live_corpus_ref`, `load_corpus_messages`, and `push_analysis_document_kind_filter` facade paths through `analysis::corpus`.
- Run commands from the repository root with `--manifest-path src-tauri/Cargo.toml`.
- Every filtered `cargo test` command in this plan must run real tests, not green `0 tests` runs.
- Use `cargo fmt --manifest-path src-tauri/Cargo.toml`; after formatting inspect `git status --short --untracked-files=all` and keep unrelated rustfmt drift out of the refactor commit.
- Expected behavioral Rust changes for this slice are limited to `src-tauri/src/analysis/corpus.rs` and `src-tauri/src/analysis/corpus/live.rs`.

---

## File Structure

- Create `src-tauri/src/analysis/corpus/live.rs`
  - Owns `live_corpus_ref`, `load_corpus_messages`, Telegram current/migrated-history row loading, migrated-history metadata zstd construction, `AnalysisDocumentRow`, `push_analysis_document_kind_filter`, and analysis document live corpus loading.
- Modify `src-tauri/src/analysis/corpus.rs`
  - Adds `mod live;`.
  - Re-exports `live_corpus_ref`, `load_corpus_messages`, and `push_analysis_document_kind_filter` from `live`.
  - Removes moved live-loading definitions and imports.
  - Keeps `CorpusLoadRequest`, `YoutubeCorpusMode`, preflight structs/helpers, snapshot/source-resolution facade re-exports, tests, and the shared sqlite test harness.
- No changes expected in `src-tauri/src/analysis/mod.rs`, `src-tauri/src/analysis/report.rs`, `src-tauri/src/projects/data_range.rs`, or `src-tauri/src/analysis/store.rs`.
  - If those files need edits, stop and review why the `analysis::corpus` facade did not preserve existing paths.

---

### Task 1: Extract Live Loading Behind The Corpus Facade

**Files:**
- Create: `src-tauri/src/analysis/corpus/live.rs`
- Modify: `src-tauri/src/analysis/corpus.rs`

**Interfaces:**
- Consumes from `src-tauri/src/analysis/corpus.rs`:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum YoutubeCorpusMode {
    TranscriptOnly,
    TranscriptDescription,
    TranscriptDescriptionComments,
}

impl YoutubeCorpusMode {
    pub(crate) fn from_wire(value: Option<&str>) -> Result<Self, String>;
    pub(crate) fn as_wire(self) -> &'static str;
    pub(crate) fn includes_description(self) -> bool;
    pub(crate) fn includes_comments(self) -> bool;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CorpusLoadRequest {
    pub(crate) source_type: String,
    pub(crate) source_ids: Vec<i64>,
    pub(crate) period_from: i64,
    pub(crate) period_to: i64,
    pub(crate) youtube_corpus_mode: YoutubeCorpusMode,
    pub(crate) include_migrated_history: bool,
}
```

- Produces through the `src-tauri/src/analysis/corpus.rs` facade:

```rust
pub(crate) fn live_corpus_ref(source_id: i64, item_id: i64) -> String;

pub(crate) async fn load_corpus_messages(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    request: &CorpusLoadRequest,
) -> crate::error::AppResult<Vec<crate::analysis::models::CorpusMessage>>;

pub(crate) fn push_analysis_document_kind_filter(
    query: &mut sqlx::QueryBuilder<'_, sqlx::Sqlite>,
    source_type: &str,
    youtube_corpus_mode: YoutubeCorpusMode,
    table_alias: &str,
) -> crate::error::AppResult<()>;
```

- Live-only helpers remain private inside `live.rs`:

```rust
fn telegram_history_metadata_zstd(
    history_scope: &str,
    migration_domain: Option<&str>,
    history_peer_kind: &str,
    history_peer_id: i64,
) -> crate::error::AppResult<Vec<u8>>;

struct TelegramCorpusRow;

async fn fetch_telegram_corpus_rows(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    request: &CorpusLoadRequest,
    include_migrated_rows: bool,
) -> crate::error::AppResult<Vec<TelegramCorpusRow>>;

async fn load_telegram_corpus_messages(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    request: &CorpusLoadRequest,
) -> crate::error::AppResult<Vec<crate::analysis::models::CorpusMessage>>;

struct AnalysisDocumentRow;

async fn load_analysis_document_messages(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    request: &CorpusLoadRequest,
) -> crate::error::AppResult<Vec<crate::analysis::models::CorpusMessage>>;
```

- [ ] **Step 0: Capture pre-edit worktree state**

Run:

```powershell
git status --short --untracked-files=all
```

Record the full output as the Step 0 baseline. Expected: no output for the implementation-owned files:

```text
src-tauri/src/analysis/corpus.rs
src-tauri/src/analysis/corpus/live.rs
```

Unrelated pre-existing entries may remain in the worktree, but they are not part of this task and must remain unchanged unless the user explicitly separates or approves them.

If either target file is already modified, staged, or untracked before this task starts, inspect the baseline before editing:

```powershell
git diff -- src-tauri/src/analysis/corpus.rs src-tauri/src/analysis/corpus/live.rs
```

```powershell
git diff --cached -- src-tauri/src/analysis/corpus.rs src-tauri/src/analysis/corpus/live.rs
```

If `src-tauri/src/analysis/corpus/live.rs` appears as an untracked file, `git diff` will not show its contents. Inspect it directly before proceeding:

```powershell
Get-Content -Raw -LiteralPath 'src-tauri/src/analysis/corpus/live.rs'
```

Stop unless the baseline is intentionally cleanly separated first. Do not stage pre-existing target-file changes into the live-loading refactor commit.

- [ ] **Step 1: Run focused live-loading characterization tests before editing**

Run each command separately:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::load_corpus_messages
```

Expected: PASS and not a green `0 tests` run. The current filter should include these five tests:

```text
analysis::corpus::tests::load_corpus_messages_returns_typed_internal_for_corrupt_live_document_content
analysis::corpus::tests::load_corpus_messages_orders_transcript_segments_by_document_order_not_ref
analysis::corpus::tests::load_corpus_messages_filters_telegram_to_telegram_message
analysis::corpus::tests::load_corpus_messages_filters_youtube_transcript_only_to_transcripts
analysis::corpus::tests::load_corpus_messages_includes_youtube_comment_only_in_comments_mode
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::live_corpus_refs_use_local_item_ids
```

Expected: PASS with output containing `1 passed`.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::default_analysis_corpus_excludes_migrated_history_documents
```

Expected: PASS with output containing `1 passed`.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::opted_in_analysis_corpus_includes_migrated_rows_and_counts_preflight
```

Expected: PASS with output containing `1 passed`.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::explicit_analysis_opt_in_with_zero_migrated_rows_keeps_current_corpus
```

Expected: PASS with output containing `1 passed`.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::source_group_opt_in_includes_only_members_with_migrated_rows
```

Expected: PASS with output containing `1 passed`.

If any focused live-loading characterization test fails before editing, stop and inspect the existing failure before moving code.

- [ ] **Step 2: Run consumer baseline tests before editing**

Run each command separately:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::
```

Expected: PASS and not a green `0 tests` run. Current snapshot at plan authoring: `43 passed`; do not require this exact count if nearby tests changed before execution. This broad baseline covers neighboring snapshot, source-resolution, and preflight behavior before moving live-loading code.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::
```

Expected: PASS and not a green `0 tests` run. Current snapshot at plan authoring: `22 passed`; do not require this exact count if nearby tests changed before execution.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml projects::data_range::tests::
```

Expected: PASS and not a green `0 tests` run. Current snapshot at plan authoring: `8 passed`; do not require this exact count if nearby tests changed before execution.

If any broad or consumer baseline test fails before editing, stop and inspect the existing failure before moving code.

- [ ] **Step 3: Create the nested module file with live-loading imports**

Create `src-tauri/src/analysis/corpus/live.rs` with this header:

```rust
use sqlx::{Pool, QueryBuilder, Sqlite};

use super::{CorpusLoadRequest, YoutubeCorpusMode};
use crate::analysis::models::CorpusMessage;
use crate::compression::{compress_json_bytes, decompress_text};
use crate::error::{internal_error, AppError, AppResult};
```

- [ ] **Step 4: Move live corpus ref and live loading entry point into `live.rs`**

Move exactly these two definitions from `src-tauri/src/analysis/corpus.rs` into `src-tauri/src/analysis/corpus/live.rs`, immediately after the imports:

```rust
#[allow(dead_code)]
pub(crate) fn live_corpus_ref(source_id: i64, item_id: i64) -> String {
    crate::analysis_documents::live_item_ref(source_id, item_id)
}

pub(crate) async fn load_corpus_messages(
    pool: &Pool<Sqlite>,
    request: &CorpusLoadRequest,
) -> AppResult<Vec<CorpusMessage>> {
    if request.source_ids.is_empty() {
        return Ok(Vec::new());
    }

    if request.source_type == "telegram" {
        return load_telegram_corpus_messages(pool, request).await;
    }

    load_analysis_document_messages(pool, request).await
}
```

These are not contiguous in the current file: `estimate_preflight_chunk_count` sits between `live_corpus_ref` and `load_corpus_messages`. Move `live_corpus_ref` and `load_corpus_messages` as two separate ranges. Do not move `estimate_message_input_chars` or `estimate_preflight_chunk_count`; they stay in `corpus.rs`.

- [ ] **Step 5: Move Telegram live corpus loading into `live.rs`**

Move this contiguous block from `src-tauri/src/analysis/corpus.rs` into `src-tauri/src/analysis/corpus/live.rs`, immediately after `load_corpus_messages`:

```rust
fn telegram_history_metadata_zstd(
    history_scope: &str,
    migration_domain: Option<&str>,
    history_peer_kind: &str,
    history_peer_id: i64,
) -> AppResult<Vec<u8>>

#[derive(sqlx::FromRow)]
struct TelegramCorpusRow {
    item_id: i64,
    source_id: i64,
    external_id: String,
    author: Option<String>,
    published_at: i64,
    ref_: Option<String>,
    content_zstd: Vec<u8>,
    source_type: String,
    source_subtype: Option<String>,
    history_scope: String,
    migration_domain: Option<String>,
    history_peer_kind: String,
    history_peer_id: i64,
}

async fn fetch_telegram_corpus_rows(
    pool: &Pool<Sqlite>,
    request: &CorpusLoadRequest,
    include_migrated_rows: bool,
) -> AppResult<Vec<TelegramCorpusRow>>

async fn load_telegram_corpus_messages(
    pool: &Pool<Sqlite>,
    request: &CorpusLoadRequest,
) -> AppResult<Vec<CorpusMessage>>
```

Move each full function body verbatim. Do not change the current-row query, migrated-row query, migrated-history filters, metadata JSON keys, decompression mapping, `live_corpus_ref` fallback, or final sort:

```rust
messages.sort_by(|left, right| {
    left.published_at
        .cmp(&right.published_at)
        .then_with(|| left.source_id.cmp(&right.source_id))
        .then_with(|| left.r#ref.cmp(&right.r#ref))
});
```

- [ ] **Step 6: Move analysis-document live loading and document-kind filter into `live.rs`**

Move this contiguous block from `src-tauri/src/analysis/corpus.rs` into `src-tauri/src/analysis/corpus/live.rs`, immediately after `load_telegram_corpus_messages`:

```rust
#[derive(sqlx::FromRow)]
struct AnalysisDocumentRow {
    item_id: Option<i64>,
    source_id: i64,
    external_id: String,
    author: Option<String>,
    published_at: i64,
    ref_: String,
    content_zstd: Vec<u8>,
    document_kind: String,
    source_type: String,
    source_subtype: Option<String>,
    metadata_zstd: Option<Vec<u8>>,
}

pub(crate) fn push_analysis_document_kind_filter(
    query: &mut QueryBuilder<'_, Sqlite>,
    source_type: &str,
    youtube_corpus_mode: YoutubeCorpusMode,
    table_alias: &str,
) -> AppResult<()>

async fn load_analysis_document_messages(
    pool: &Pool<Sqlite>,
    request: &CorpusLoadRequest,
) -> AppResult<Vec<CorpusMessage>>
```

Move each full function body verbatim. Do not change the unsupported source-type validation string:

```rust
format!("Unsupported analysis corpus source_type '{other}'")
```

Do not change the analysis document ordering:

```rust
ORDER BY d.published_at ASC, d.source_id ASC, d.document_order ASC, d.id ASC
```

- [ ] **Step 7: Add the private module and facade re-exports in `corpus.rs`**

At the top of `src-tauri/src/analysis/corpus.rs`, add `mod live;` before the existing nested modules. After import cleanup, the top of the file should start like this:

```rust
mod live;
mod snapshot;
mod source_resolution;

#[allow(unused_imports)]
pub(crate) use self::live::live_corpus_ref;
pub(crate) use self::live::{load_corpus_messages, push_analysis_document_kind_filter};
#[allow(unused_imports)]
pub(crate) use self::snapshot::load_run_corpus_messages;
pub(crate) use self::snapshot::{
    list_run_snapshot_messages_page, load_run_snapshot_messages, load_trace_resolution_messages,
    ListRunSnapshotMessagesRequest,
};
#[cfg(test)]
pub(crate) use self::source_resolution::resolve_run_source_ids;
#[allow(unused_imports)]
pub(crate) use self::source_resolution::ResolvedAnalysisSources;
pub(crate) use self::source_resolution::{
    resolve_analysis_sources, AnalysisSourceResolutionError, AnalysisSourceResolutionErrorCode,
};
use sqlx::{Pool, Sqlite};

use crate::error::AppResult;
```

`Pool`, `Sqlite`, and `AppResult` remain in `corpus.rs` for `preflight_analysis_run`. `QueryBuilder`, `CorpusMessage`, `compress_json_bytes`, `decompress_text`, `internal_error`, and `AppError` move to `live.rs`.

- [ ] **Step 8: Remove moved-only imports and definitions from `corpus.rs`**

Replace the current grouped imports in `src-tauri/src/analysis/corpus.rs`.

Change this:

```rust
use sqlx::{Pool, QueryBuilder, Sqlite};

use super::models::CorpusMessage;
use crate::compression::{compress_json_bytes, decompress_text};
use crate::error::{internal_error, AppError, AppResult};
```

to this:

```rust
use sqlx::{Pool, Sqlite};

use crate::error::AppResult;
```

Remove the moved live-loading definitions from `corpus.rs` after they exist in `live.rs`. `corpus.rs` must still contain:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum YoutubeCorpusMode {
    TranscriptOnly,
    TranscriptDescription,
    TranscriptDescriptionComments,
}

impl YoutubeCorpusMode {
    pub(crate) fn from_wire(value: Option<&str>) -> Result<Self, String>;
    pub(crate) fn as_wire(self) -> &'static str;
    pub(crate) fn includes_description(self) -> bool;
    pub(crate) fn includes_comments(self) -> bool;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CorpusLoadRequest {
    pub(crate) source_type: String,
    pub(crate) source_ids: Vec<i64>,
    pub(crate) period_from: i64,
    pub(crate) period_to: i64,
    pub(crate) youtube_corpus_mode: YoutubeCorpusMode,
    pub(crate) include_migrated_history: bool,
}

pub(crate) fn estimate_message_input_chars(
    content: &str,
    r#ref: &str,
    author: Option<&str>,
) -> usize;

pub(crate) fn estimate_preflight_chunk_count(message_sizes: &[usize], max_chars: usize) -> usize;

pub(crate) async fn preflight_analysis_run(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    request: &CorpusLoadRequest,
    chunk_target_chars: usize,
    limits: AnalysisRunPreflightLimits,
) -> AppResult<AnalysisRunPreflight>;

pub(crate) fn preflight_limit_error(preflight: &AnalysisRunPreflight) -> Option<String>;

pub(crate) fn model_limit_preflight_error(
    preflight: &AnalysisRunPreflight,
    model_input_limit: Option<usize>,
) -> Option<String>;

#[cfg(test)] mod tests
```

Leave these retained definitions and their attributes byte-for-byte unless `rustfmt` changes whitespace.

- [ ] **Step 9: Run rustfmt and inspect the touched files**

Run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml
```

Then run:

```powershell
git status --short --untracked-files=all
```

Expected `git status --short --untracked-files=all` output for this task, relative to the Step 0 baseline:

```text
 M src-tauri/src/analysis/corpus.rs
?? src-tauri/src/analysis/corpus/live.rs
```

If unrelated Rust files newly appear beyond the Step 0 baseline, do not stage them in the refactor commit. Resolve unrelated rustfmt drift before final verification starts: either make a separate format-only commit after review, or otherwise separate those changes so the only new status entries are the implementation-owned files for this refactor.

- [ ] **Step 10: Run focused live-loading tests after editing**

Run each command separately:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::load_corpus_messages
```

Expected: PASS and not a green `0 tests` run. The output should include:

```text
analysis::corpus::tests::load_corpus_messages_returns_typed_internal_for_corrupt_live_document_content
analysis::corpus::tests::load_corpus_messages_orders_transcript_segments_by_document_order_not_ref
analysis::corpus::tests::load_corpus_messages_filters_telegram_to_telegram_message
analysis::corpus::tests::load_corpus_messages_filters_youtube_transcript_only_to_transcripts
analysis::corpus::tests::load_corpus_messages_includes_youtube_comment_only_in_comments_mode
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::live_corpus_refs_use_local_item_ids
```

Expected: PASS with output containing `1 passed`.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::default_analysis_corpus_excludes_migrated_history_documents
```

Expected: PASS with output containing `1 passed`.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::opted_in_analysis_corpus_includes_migrated_rows_and_counts_preflight
```

Expected: PASS with output containing `1 passed`.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::explicit_analysis_opt_in_with_zero_migrated_rows_keeps_current_corpus
```

Expected: PASS with output containing `1 passed`.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::source_group_opt_in_includes_only_members_with_migrated_rows
```

Expected: PASS with output containing `1 passed`.

- [ ] **Step 11: Run module-boundary compile coverage**

Run:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
```

Expected: PASS. Existing warnings outside the touched files may remain; new warnings mentioning `src/analysis/corpus.rs` or `src/analysis/corpus/live.rs` are not acceptable.

- [ ] **Step 12: Run full corpus and consumer behavior tests before committing**

Run each command separately:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::
```

Expected: PASS and not a green `0 tests` run. This must pass before the refactor commit is created; it covers neighboring snapshot, source-resolution, and preflight behavior through the same `analysis::corpus` facade.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::
```

Expected: PASS and not a green `0 tests` run. This verifies the `analysis/report.rs` live-loading consumer before the refactor commit is created.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml projects::data_range::tests::
```

Expected: PASS and not a green `0 tests` run. This verifies the project data-range document-kind filter consumer before the refactor commit is created.

If the full corpus suite or either consumer test slice fails here, stop and fix the refactor before committing. Do not defer these failures to Task 2.

- [ ] **Step 13: Commit the live-loading extraction**

Before staging, run:

```powershell
git status --short --untracked-files=all
```

Expected: compared with the Step 0 baseline, only these implementation-owned paths are modified or untracked:

```text
 M src-tauri/src/analysis/corpus.rs
?? src-tauri/src/analysis/corpus/live.rs
```

If `src-tauri/src/analysis/corpus.rs` had a pre-edit baseline diff from Step 0, stop here unless that baseline has already been separated from this refactor. Do not stage unrelated pre-existing Step 0 baseline entries.

Run:

```powershell
git add -- src-tauri/src/analysis/corpus.rs src-tauri/src/analysis/corpus/live.rs
```

Then run:

```powershell
git commit -m "refactor: extract analysis corpus live loading"
```

Expected: commit succeeds and includes only `src-tauri/src/analysis/corpus.rs` and `src-tauri/src/analysis/corpus/live.rs`.

---

### Task 2: Final Regression Verification

**Files:**
- Verify: `src-tauri/src/analysis/corpus.rs`
- Verify: `src-tauri/src/analysis/corpus/live.rs`
- Verify: `src-tauri/src/analysis/report.rs`
- Verify: `src-tauri/src/projects/data_range.rs`
- Verify: `src-tauri/src/analysis/mod.rs`
- Verify: `src-tauri/src/analysis/store.rs`

**Interfaces:**
- Confirms the `analysis::corpus` facade still satisfies current callers:
  - `analysis/report.rs`: `load_corpus_messages`, `preflight_analysis_run`, `preflight_limit_error`, `CorpusLoadRequest`, and `YoutubeCorpusMode`
  - `projects/data_range.rs`: root re-exported `push_analysis_document_kind_filter` and `YoutubeCorpusMode`
  - `analysis/mod.rs`: root re-export of `push_analysis_document_kind_filter` and `YoutubeCorpusMode`
  - `analysis/store.rs`: direct `analysis::corpus::YoutubeCorpusMode` import
  - `analysis/corpus.rs::preflight_analysis_run`: internal `load_corpus_messages` facade call
  - `analysis/corpus.rs` tests: facade-visible `load_corpus_messages`, `live_corpus_ref`, and preflight helpers

- [ ] **Step 1: Run the full corpus test module**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::
```

Expected: PASS and not a green `0 tests` run. Current snapshot at plan authoring: `43 passed`; do not require this exact count if nearby tests changed before execution.

- [ ] **Step 2: Run report consumer tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::
```

Expected: PASS and not a green `0 tests` run. Current snapshot at plan authoring: `22 passed`; do not require this exact count if nearby tests changed before execution.

- [ ] **Step 3: Run project data-range consumer tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml projects::data_range::tests::
```

Expected: PASS and not a green `0 tests` run. Current snapshot at plan authoring: `8 passed`; do not require this exact count if nearby tests changed before execution.

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

Expected: PASS. Existing warnings outside the touched files may remain; new warnings mentioning `src/analysis/corpus.rs` or `src/analysis/corpus/live.rs` are not acceptable.

- [ ] **Step 6: Confirm final git state**

Run:

```powershell
git status --short --untracked-files=all
```

Expected: after the refactor commit, `git status --short --untracked-files=all` matches the Step 0 baseline exactly. If the Step 0 baseline was empty, this command is empty. If verification discovers a required fix, make that fix in a separate commit or clearly document why the refactor commit was updated.

Run:

```powershell
git log --oneline -3
```

Expected: the latest commits include:

```text
refactor: extract analysis corpus live loading
```
