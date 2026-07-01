# Analysis Corpus Tests Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Status:** active plan, not implemented as of 2026-07-01.

**Goal:** Split the large inline `analysis::corpus` test module into focused nested test files without changing production behavior or test assertions.

**Architecture:** Keep `src-tauri/src/analysis/corpus.rs` as the production corpus facade and replace the current inline `#[cfg(test)] mod tests` body with `#[cfg(test)] mod tests;`. Create `src-tauri/src/analysis/corpus/tests/` with a shared `harness.rs` and thematic `live.rs`, `preflight.rs`, `snapshot.rs`, and `source_resolution.rs` modules. Preserve existing behavior by moving test bodies and helpers verbatim, widening only test-helper visibility to `pub(super)` when sibling test modules need a helper.

**Tech Stack:** Rust, Cargo test/check/fmt, SQLx SQLite in-memory test harness.

## Global Constraints

- Internal Rust test-only refactor: no frontend, Tauri command payload, event payload, SQL schema, migration, runtime code, or user-facing behavior changes.
- Keep all production definitions and facade re-exports in `src-tauri/src/analysis/corpus.rs`.
- Replace the inline test module in `corpus.rs` with exactly `#[cfg(test)] mod tests;`.
- Do not move production code from `live.rs`, `snapshot.rs`, or `source_resolution.rs`.
- Do not add, delete, or weaken behavior tests.
- Preserve SQL, schema setup, seed data, ordering, error assertions, and assertion contents.
- Expected production visibility changes: none.
- Expected test-helper visibility changes: helpers shared from `harness.rs` become `pub(super)` and must not be widened beyond `pub(super)`.
- Run commands from the repository root with `--manifest-path src-tauri/Cargo.toml`.
- Every filtered `cargo test` command in this plan must run real tests, not green `0 tests` runs.
- Existing warnings outside touched files may remain; new warnings mentioning `src-tauri/src/analysis/corpus.rs` or `src-tauri/src/analysis/corpus/tests/` are not acceptable.
- The design spec and implementation plan are committed separately from the Rust refactor.

---

## File Structure

- Modify: `src-tauri/src/analysis/corpus.rs`
  - Replace the current inline `#[cfg(test)] mod tests` body with `#[cfg(test)] mod tests;`.
  - Keep all production code unchanged.
- Create: `src-tauri/src/analysis/corpus/tests/mod.rs`
  - Declares child modules only.
- Create: `src-tauri/src/analysis/corpus/tests/harness.rs`
  - Owns shared SQLite schema setup, seed helpers, fixture constructors, and metadata helpers.
- Create: `src-tauri/src/analysis/corpus/tests/live.rs`
  - Owns live corpus loading, migrated-history, YouTube mode, and live-document filtering tests.
- Create: `src-tauri/src/analysis/corpus/tests/preflight.rs`
  - Owns preflight formula, limit, and loader/preflight alignment tests.
- Create: `src-tauri/src/analysis/corpus/tests/snapshot.rs`
  - Owns snapshot persistence, pagination, trace-resolution, and completed-run snapshot behavior tests.
- Create: `src-tauri/src/analysis/corpus/tests/source_resolution.rs`
  - Owns project source-resolution, mixed-provider, playlist expansion, and run-source-id tests.

---

### Task 1: Baseline And Test Tree Scaffolding

**Files:**
- Modify: `src-tauri/src/analysis/corpus.rs`
- Create: `src-tauri/src/analysis/corpus/tests/mod.rs`
- Create: `src-tauri/src/analysis/corpus/tests/harness.rs`
- Create: `src-tauri/src/analysis/corpus/tests/live.rs`
- Create: `src-tauri/src/analysis/corpus/tests/preflight.rs`
- Create: `src-tauri/src/analysis/corpus/tests/snapshot.rs`
- Create: `src-tauri/src/analysis/corpus/tests/source_resolution.rs`

**Interfaces:**
- Consumes: current inline `#[cfg(test)] mod tests` from `src-tauri/src/analysis/corpus.rs`.
- Produces:
  - `#[cfg(test)] mod tests;` in `corpus.rs`.
  - `tests/mod.rs` declaring `harness`, `live`, `preflight`, `snapshot`, and `source_resolution`.
  - Empty thematic modules ready to receive moved tests.

- [x] **Step 1: Capture pre-edit worktree state**

Run:

```powershell
git status --short --untracked-files=all
```

Expected: record the full output as the Step 1 baseline. The usual clean baseline has no output. If unrelated pre-existing entries exist, leave them untouched unless the user explicitly separates or approves them.

- [x] **Step 2: Inspect target baseline if target files are already dirty**

Run:

```powershell
git diff -- src-tauri/src/analysis/corpus.rs
```

Then run:

```powershell
git diff --cached -- src-tauri/src/analysis/corpus.rs
```

If `src-tauri/src/analysis/corpus/tests` exists, run:

```powershell
if (Test-Path -LiteralPath 'src-tauri/src/analysis/corpus/tests') {
    git diff -- src-tauri/src/analysis/corpus/tests
    git diff --cached -- src-tauri/src/analysis/corpus/tests
    Get-ChildItem -Recurse -File -LiteralPath 'src-tauri/src/analysis/corpus/tests' |
        ForEach-Object { $_.FullName; Get-Content -Raw -LiteralPath $_.FullName }
}
```

Expected: no pre-existing target-file changes. If `corpus.rs` or `corpus/tests/*` is dirty, staged, or untracked, stop unless that baseline is intentionally separated first.

- [x] **Step 3: Run the full corpus test baseline**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::
```

Expected: PASS and not a green `0 tests` run. Current snapshot at plan authoring: `43 passed`; do not require this exact count if nearby tests changed before execution.

If this baseline command fails, or if it succeeds with `0 tests`, stop before editing. Treat the failure as pre-existing debt and resolve or explicitly document it before starting this refactor.

- [x] **Step 4: Create the nested test module declarations**

Create `src-tauri/src/analysis/corpus/tests/mod.rs`:

```rust
mod harness;
mod live;
mod preflight;
mod snapshot;
mod source_resolution;
```

Create these empty files:

```text
src-tauri/src/analysis/corpus/tests/harness.rs
src-tauri/src/analysis/corpus/tests/live.rs
src-tauri/src/analysis/corpus/tests/preflight.rs
src-tauri/src/analysis/corpus/tests/snapshot.rs
src-tauri/src/analysis/corpus/tests/source_resolution.rs
```

- [x] **Step 5: Replace the inline test module shell in `corpus.rs`**

In `src-tauri/src/analysis/corpus.rs`, replace the entire current inline test module, starting at `#[cfg(test)] mod tests {` and ending at that module's closing brace, with:

```rust
#[cfg(test)]
mod tests;
```

Do not change any production code above the test module. Move the removed test-module body into the new test files in the next tasks before running compile checks.

---

### Task 2: Shared Test Harness

**Files:**
- Modify: `src-tauri/src/analysis/corpus.rs`
- Modify: `src-tauri/src/analysis/corpus/tests/harness.rs`

**Interfaces:**
- Consumes: helper functions currently inside `src-tauri/src/analysis/corpus.rs::tests`.
- Produces these shared test helpers, with `pub(super)` only when used by sibling modules:

```rust
pub(super) fn sample_corpus() -> Vec<CorpusMessage>;
pub(super) async fn create_project_scope_schema(pool: &sqlx::SqlitePool);
pub(super) async fn snapshot_pool() -> sqlx::SqlitePool;
pub(super) fn corpus_request(
    source_type: &str,
    source_ids: Vec<i64>,
    youtube_corpus_mode: YoutubeCorpusMode,
) -> CorpusLoadRequest;
pub(super) async fn rebuild_documents_for_sources(pool: &sqlx::SqlitePool, source_ids: &[i64]);
pub(super) async fn seed_analysis_source(
    pool: &sqlx::SqlitePool,
    id: i64,
    source_type: &str,
    source_subtype: &str,
);
pub(super) async fn seed_telegram_item(
    pool: &sqlx::SqlitePool,
    id: i64,
    source_id: i64,
    external_id: &str,
    published_at: i64,
    content: &str,
    is_migrated_history: bool,
);
pub(super) fn youtube_metadata_zstd(
    video_id: &str,
    title: &str,
    description: Option<&str>,
) -> Vec<u8>;
pub(super) async fn insert_youtube_video_source(pool: &sqlx::SqlitePool, source_id: i64);
pub(super) async fn insert_youtube_video_source_with_typed_metadata(
    pool: &sqlx::SqlitePool,
    source_id: i64,
    metadata_zstd: Vec<u8>,
);
pub(super) async fn insert_typed_youtube_video_source(
    pool: &sqlx::SqlitePool,
    source_id: i64,
    video_id: &str,
    title: &str,
    description: Option<&str>,
);
pub(super) async fn insert_youtube_transcript_segment(
    pool: &sqlx::SqlitePool,
    item_id: i64,
    source_id: i64,
    published_at: i64,
    content: &str,
);
pub(super) fn decode_message_metadata_for_test(message: &CorpusMessage) -> serde_json::Value;
pub(super) fn sample_run() -> AnalysisRunDetail;
```

- [x] **Step 1: Add harness imports**

At the top of `src-tauri/src/analysis/corpus/tests/harness.rs`, add:

```rust
use crate::analysis::corpus::{
    CorpusLoadRequest, ListRunSnapshotMessagesRequest, YoutubeCorpusMode,
};
use crate::analysis::models::{AnalysisRunDetail, CorpusMessage};
use crate::compression::{compress_json_bytes, compress_text};
use crate::youtube::dto::{YoutubeAvailabilityStatus, YoutubeVideoForm, YoutubeVideoMetadata};
```

If `ListRunSnapshotMessagesRequest` is unused after moving helpers, remove it during the rustfmt/import-cleanup step.

- [x] **Step 2: Move shared fixture helpers into `harness.rs`**

Move these definitions from the old inline `tests` module into `harness.rs`:

```text
sample_corpus
create_project_scope_schema
snapshot_pool
corpus_request
rebuild_documents_for_sources
seed_analysis_source
seed_telegram_item
youtube_metadata_zstd
insert_youtube_video_source
insert_youtube_video_source_with_typed_metadata
insert_typed_youtube_video_source
insert_youtube_transcript_segment
decode_message_metadata_for_test
sample_run
```

Preserve each function body verbatim. Change function visibility to `pub(super)` only for helpers that are used by sibling modules. If a moved helper is used only inside `harness.rs`, keep it private.

- [x] **Step 3: Adjust helper type names for the new module**

Inside `harness.rs`, use fully qualified `sqlx::SqlitePool` in public helper signatures. The visible helper signatures must not depend on a local `use sqlx::SqlitePool;`.

Expected examples:

```rust
pub(super) async fn snapshot_pool() -> sqlx::SqlitePool
```

```rust
pub(super) async fn insert_youtube_video_source(
    pool: &sqlx::SqlitePool,
    source_id: i64,
)
```

- [x] **Step 4: Keep production imports out of `corpus.rs`**

After removing the inline test module body, `src-tauri/src/analysis/corpus.rs` should not import test-only items at the production module level. Do not add imports such as `CorpusMessage`, `compress_text`, `compress_json_bytes`, `AppErrorKind`, or YouTube DTOs to production scope.

---

### Task 3: Live Corpus Test Module

**Files:**
- Modify: `src-tauri/src/analysis/corpus/tests/live.rs`
- Modify: `src-tauri/src/analysis/corpus/tests/harness.rs` if a helper visibility adjustment is needed

**Interfaces:**
- Consumes helpers from `super::harness`.
- Consumes production corpus facade items from `crate::analysis::corpus`.
- Produces the live-loading test module under `analysis::corpus::tests::live`.

- [ ] **Step 1: Add live module imports**

At the top of `src-tauri/src/analysis/corpus/tests/live.rs`, add:

```rust
use super::harness::{
    corpus_request, decode_message_metadata_for_test, insert_youtube_transcript_segment,
    insert_youtube_video_source, insert_youtube_video_source_with_typed_metadata,
    rebuild_documents_for_sources, seed_analysis_source, seed_telegram_item, snapshot_pool,
};
use crate::analysis::corpus::{
    live_corpus_ref, load_corpus_messages, preflight_analysis_run, AnalysisRunPreflightLimits,
    YoutubeCorpusMode,
};
use crate::error::AppErrorKind;
```

If rustc reports an unused import after all live tests move, remove only that unused import.

- [ ] **Step 2: Move migrated-history and typed YouTube live tests**

Move these tests from the old inline module into `live.rs`, preserving bodies and assertions:

```text
default_analysis_corpus_excludes_migrated_history_documents
opted_in_analysis_corpus_includes_migrated_rows_and_counts_preflight
source_group_opt_in_includes_only_members_with_migrated_rows
explicit_analysis_opt_in_with_zero_migrated_rows_keeps_current_corpus
youtube_description_rows_use_typed_metadata_with_corrupt_source_blob
youtube_description_missing_typed_metadata_skips_without_decoding_source_blob
youtube_transcript_segment_evidence_uses_typed_source_context
```

- [ ] **Step 3: Move live loader behavior tests**

Move these tests from the old inline module into `live.rs`, preserving bodies and assertions:

```text
live_corpus_refs_use_local_item_ids
preflight_ref_format_matches_corpus_loader_ref_format
load_corpus_messages_returns_typed_internal_for_corrupt_live_document_content
load_corpus_messages_orders_transcript_segments_by_document_order_not_ref
youtube_corpus_mode_parses_wire_values_and_defaults
load_corpus_messages_filters_telegram_to_telegram_message
load_corpus_messages_filters_youtube_transcript_only_to_transcripts
load_corpus_messages_includes_youtube_comment_only_in_comments_mode
description_mode_creates_synthetic_description_message
```

- [ ] **Step 4: Keep local-only helpers local**

If any helper is used only by `live.rs` after the move, keep it private in `live.rs` rather than exporting it from `harness.rs`. Do not change test assertions to avoid moving a helper.

---

### Task 4: Preflight Test Module

**Files:**
- Modify: `src-tauri/src/analysis/corpus/tests/preflight.rs`
- Modify: `src-tauri/src/analysis/corpus/tests/harness.rs` if a helper visibility adjustment is needed

**Interfaces:**
- Consumes helpers from `super::harness`.
- Consumes production preflight helpers from `crate::analysis::corpus`.
- Produces the preflight test module under `analysis::corpus::tests::preflight`.

- [ ] **Step 1: Add preflight module imports**

At the top of `src-tauri/src/analysis/corpus/tests/preflight.rs`, add:

```rust
use super::harness::{
    corpus_request, insert_youtube_transcript_segment,
    insert_youtube_video_source_with_typed_metadata, snapshot_pool,
};
use crate::analysis::corpus::{
    estimate_message_input_chars, estimate_preflight_chunk_count, load_corpus_messages,
    model_limit_preflight_error, preflight_analysis_run, preflight_limit_error,
    AnalysisRunPreflight, AnalysisRunPreflightLimits, YoutubeCorpusMode,
};
```

If rustc reports an unused import after all preflight tests move, remove only that unused import.

- [ ] **Step 2: Move pure preflight tests**

Move these tests from the old inline module into `preflight.rs`, preserving bodies and assertions:

```text
estimated_message_chars_match_report_chunk_accounting
estimated_chunk_count_matches_chunk_boundary_behavior
default_preflight_limits_are_conservative
preflight_limit_error_reports_all_scale_dimensions
preflight_limit_error_allows_runs_within_limits
model_limit_preflight_allows_unknown_or_fitting_limits
model_limit_preflight_reports_oversized_chunks
```

- [ ] **Step 3: Move async preflight tests**

Move these tests from the old inline module into `preflight.rs`, preserving bodies and assertions:

```text
preflight_counts_eligible_text_messages_for_sources
preflight_ignores_media_only_items_without_text_content
preflight_count_matches_loader_for_youtube_corpus_modes
```

---

### Task 5: Snapshot Test Module

**Files:**
- Modify: `src-tauri/src/analysis/corpus/tests/snapshot.rs`
- Modify: `src-tauri/src/analysis/corpus/tests/harness.rs` if a helper visibility adjustment is needed

**Interfaces:**
- Consumes helpers from `super::harness`.
- Consumes snapshot facade functions from `crate::analysis::corpus`.
- Produces the snapshot test module under `analysis::corpus::tests::snapshot`.

- [ ] **Step 1: Add snapshot module imports**

At the top of `src-tauri/src/analysis/corpus/tests/snapshot.rs`, add:

```rust
use super::harness::{sample_corpus, sample_run, snapshot_pool};
use crate::analysis::corpus::{
    list_run_snapshot_messages_page, load_run_corpus_messages, load_run_snapshot_messages,
    load_trace_resolution_messages, ListRunSnapshotMessagesRequest,
};
use crate::analysis::models::AnalysisRunMessageCursor;
use crate::analysis::store::persist_run_snapshot;
use crate::error::AppErrorKind;
```

If rustc reports an unused import after all snapshot tests move, remove only that unused import.

- [ ] **Step 2: Move snapshot persistence and page tests**

Move these tests from the old inline module into `snapshot.rs`, preserving bodies and assertions:

```text
run_snapshot_roundtrips_frozen_corpus
list_run_snapshot_messages_page_reads_saved_snapshot_only
list_run_snapshot_messages_page_returns_typed_internal_for_corrupt_snapshot_content
list_run_snapshot_messages_page_starts_at_around_ref
list_run_snapshot_messages_page_does_not_fall_back_to_live_source
```

- [ ] **Step 3: Move trace and completed-run snapshot tests**

Move these tests from the old inline module into `snapshot.rs`, preserving bodies and assertions:

```text
trace_resolution_does_not_fall_back_to_live_source_for_completed_missing_snapshot
run_message_cursor_uses_ref_and_published_at
load_run_corpus_messages_uses_snapshot_when_available
load_run_corpus_messages_does_not_reconstruct_completed_capture_failed_from_live_rows
captured_marker_with_missing_rows_returns_corrupt_snapshot_error
source_group_membership_drift_after_capture_does_not_change_saved_run_corpus
```

---

### Task 6: Source Resolution Test Module

**Files:**
- Modify: `src-tauri/src/analysis/corpus/tests/source_resolution.rs`
- Modify: `src-tauri/src/analysis/corpus/tests/harness.rs` if a helper visibility adjustment is needed

**Interfaces:**
- Consumes helpers from `super::harness`.
- Consumes source-resolution facade functions from `crate::analysis::corpus`.
- Produces the source-resolution test module under `analysis::corpus::tests::source_resolution`.

- [ ] **Step 1: Add source-resolution module imports**

At the top of `src-tauri/src/analysis/corpus/tests/source_resolution.rs`, add:

```rust
use super::harness::{create_project_scope_schema, sample_corpus, sample_run, snapshot_pool};
use crate::analysis::corpus::{
    resolve_analysis_sources, resolve_run_source_ids, AnalysisSourceResolutionErrorCode,
};
use crate::analysis::store::persist_run_snapshot;
```

If rustc reports an unused import after all source-resolution tests move, remove only that unused import.

- [ ] **Step 2: Move run-source-id and project-source tests**

Move these tests from the old inline module into `source_resolution.rs`, preserving bodies and assertions:

```text
resolve_run_source_ids_prefers_snapshot_over_live_group_membership
resolve_run_source_ids_loads_project_sources_without_snapshot
playlist_expansion_excludes_unlinked_and_removed_rows
resolve_analysis_sources_rejects_mixed_provider_project
resolve_analysis_sources_preserves_no_linked_youtube_error_message
resolve_analysis_sources_loads_single_provider_project
```

- [ ] **Step 3: Check no tests remain in `corpus.rs`**

Run:

```powershell
rg -n "#\\[tokio::test\\]|#\\[test\\]|fn sample_corpus|async fn create_project_scope_schema|async fn snapshot_pool|fn corpus_request|async fn rebuild_documents_for_sources|async fn seed_analysis_source|async fn seed_telegram_item|fn youtube_metadata_zstd|async fn insert_youtube_video_source|async fn insert_youtube_video_source_with_typed_metadata|async fn insert_typed_youtube_video_source|async fn insert_youtube_transcript_segment|fn decode_message_metadata_for_test|fn sample_run" src-tauri/src/analysis/corpus.rs
```

Expected: no output. `corpus.rs` should contain `#[cfg(test)] mod tests;` but no inline test functions or test harness helpers.

Then run:

```powershell
rg -n "#\\[cfg\\(test\\)\\]|mod tests;" src-tauri/src/analysis/corpus.rs
```

Expected: output has exactly two matches, and those matches are the external module declaration:

```text
#[cfg(test)]
mod tests;
```

---

### Task 7: Verification And Refactor Commit

**Files:**
- Verify: `src-tauri/src/analysis/corpus.rs`
- Verify: `src-tauri/src/analysis/corpus/tests/mod.rs`
- Verify: `src-tauri/src/analysis/corpus/tests/harness.rs`
- Verify: `src-tauri/src/analysis/corpus/tests/live.rs`
- Verify: `src-tauri/src/analysis/corpus/tests/preflight.rs`
- Verify: `src-tauri/src/analysis/corpus/tests/snapshot.rs`
- Verify: `src-tauri/src/analysis/corpus/tests/source_resolution.rs`

**Interfaces:**
- Confirms `analysis::corpus::tests::` still runs the full corpus suite.
- Confirms representative tests exist under each new module path.
- Confirms consumer tests still compile and pass.

- [ ] **Step 1: Run rustfmt**

Run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml
```

Expected: command succeeds.

- [ ] **Step 2: Inspect implementation-owned file status**

Run:

```powershell
git status --short --untracked-files=all
```

Expected before commit: relative to the Step 1 baseline, only these implementation-owned paths are newly modified or untracked:

```text
 M src-tauri/src/analysis/corpus.rs
?? src-tauri/src/analysis/corpus/tests/mod.rs
?? src-tauri/src/analysis/corpus/tests/harness.rs
?? src-tauri/src/analysis/corpus/tests/live.rs
?? src-tauri/src/analysis/corpus/tests/preflight.rs
?? src-tauri/src/analysis/corpus/tests/snapshot.rs
?? src-tauri/src/analysis/corpus/tests/source_resolution.rs
```

If unrelated files appear beyond the Step 1 baseline, do not stage them in the refactor commit.

- [ ] **Step 3: Run the full corpus test module**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::
```

Expected: PASS and not a green `0 tests` run. The output must include real tests from all four thematic modules, including these exact names:

```text
analysis::corpus::tests::live::load_corpus_messages_filters_youtube_transcript_only_to_transcripts
analysis::corpus::tests::preflight::preflight_count_matches_loader_for_youtube_corpus_modes
analysis::corpus::tests::snapshot::run_snapshot_roundtrips_frozen_corpus
analysis::corpus::tests::source_resolution::playlist_expansion_excludes_unlinked_and_removed_rows
```

- [ ] **Step 4: Run report consumer tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::
```

Expected: PASS and not a green `0 tests` run. The output must include at least these representative report tests:

```text
analysis::report::tests::capture_report_corpus_returns_reloaded_snapshot_before_provider_phases
analysis::report::tests::report_start_request_carries_migrated_history_opt_in_to_corpus_request_shape
analysis::report::tests::validate_report_preflight_rejects_empty_corpus
```

- [ ] **Step 5: Run project data-range consumer tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml projects::data_range::tests::
```

Expected: PASS and not a green `0 tests` run. The output must include at least these representative project data-range tests:

```text
projects::data_range::tests::project_data_range_uses_youtube_mode_document_kinds
projects::data_range::tests::project_data_range_includes_telegram_migrated_history_when_requested
projects::data_range::tests::project_data_range_expands_playlist_to_linked_video_sources
```

- [ ] **Step 6: Run format check**

Run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
```

Expected: PASS with no diff output.

- [ ] **Step 7: Run all-target compile coverage**

Run:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
```

Expected: PASS. Existing warnings outside touched files may remain. New warnings mentioning `src-tauri/src/analysis/corpus.rs` or `src-tauri/src/analysis/corpus/tests/` are not acceptable.

- [ ] **Step 8: Check staged diff before committing**

Run:

```powershell
git diff -- src-tauri/src/analysis/corpus.rs src-tauri/src/analysis/corpus/tests
```

Expected: diff only moves the test module body into focused test files, replaces the inline module with `#[cfg(test)] mod tests;`, and adjusts test-helper imports/visibility. No production code above the test module should change.

- [ ] **Step 9: Stage only implementation-owned files**

Run:

```powershell
git add -- src-tauri/src/analysis/corpus.rs src-tauri/src/analysis/corpus/tests/mod.rs src-tauri/src/analysis/corpus/tests/harness.rs src-tauri/src/analysis/corpus/tests/live.rs src-tauri/src/analysis/corpus/tests/preflight.rs src-tauri/src/analysis/corpus/tests/snapshot.rs src-tauri/src/analysis/corpus/tests/source_resolution.rs
```

Then run:

```powershell
git diff --cached --check
```

Expected: PASS with no whitespace errors.

- [ ] **Step 10: Commit the test split**

Run:

```powershell
git commit -m "refactor: split analysis corpus tests"
```

Expected: commit succeeds and includes only:

```text
src-tauri/src/analysis/corpus.rs
src-tauri/src/analysis/corpus/tests/mod.rs
src-tauri/src/analysis/corpus/tests/harness.rs
src-tauri/src/analysis/corpus/tests/live.rs
src-tauri/src/analysis/corpus/tests/preflight.rs
src-tauri/src/analysis/corpus/tests/snapshot.rs
src-tauri/src/analysis/corpus/tests/source_resolution.rs
```

Then run:

```powershell
git rev-parse --short HEAD
```

Expected: record this exact commit hash as the refactor commit hash for final verification.

Then run:

```powershell
git show --name-only --oneline --no-renames HEAD
```

Expected: the first line is the recorded commit hash and message `refactor: split analysis corpus tests`; the file list contains only the seven implementation-owned files listed above.

- [ ] **Step 11: Confirm final git state**

Run:

```powershell
git status --short --untracked-files=all
```

Expected: no new unintended files or diffs outside the Step 1 baseline. If the Step 1 baseline was clean, this command is empty.

Run:

```powershell
git log --oneline -3
```

Expected: the first line starts with the recorded refactor commit hash from Step 10 and includes:

```text
refactor: split analysis corpus tests
```
