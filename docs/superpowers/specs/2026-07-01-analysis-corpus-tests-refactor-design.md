# Analysis Corpus Tests Refactor Design

**Date:** 2026-07-01
**Status:** active design, not implemented as of 2026-07-01; written spec ready for review before implementation planning
**Scope:** internal Rust test-only refactor of `src-tauri/src/analysis/corpus.rs` into nested corpus test modules.

## Goal

Reduce the size and review burden of `src-tauri/src/analysis/corpus.rs` by moving its large `#[cfg(test)] mod tests` body into focused nested test modules, without changing production behavior, public or crate-visible caller paths, SQL, fixtures, assertions, or test coverage.

This is the next conservative slice after extracting corpus snapshot, source-resolution, and live-loading production code. The production corpus facade is now small; the remaining file size is mostly shared SQLite test harness and corpus behavior tests.

## Current Shape

`src-tauri/src/analysis/corpus.rs` currently owns:

- production facade wiring for `live`, `snapshot`, and `source_resolution`;
- preflight structs and helpers;
- `YoutubeCorpusMode` and `CorpusLoadRequest`;
- a large inline `#[cfg(test)] mod tests` starting near the production code;
- shared in-memory SQLite schema setup and seed helpers;
- tests for live loading, preflight, snapshots, source resolution, playlist expansion, and project-scope behavior.

The test module is roughly the dominant part of the file. It mixes these concerns:

- harness/setup helpers:
  - `snapshot_pool`
  - `create_project_scope_schema`
  - `sample_corpus`
  - `sample_run`
  - `corpus_request`
  - source, Telegram, and YouTube seed helpers
  - metadata encode/decode helpers
- live corpus tests:
  - migrated Telegram history opt-in/default behavior
  - YouTube transcript/description/comment filtering
  - live ref formatting
  - corrupt live document content handling
  - live ordering by document order
- preflight tests:
  - estimate formulas
  - chunk count boundaries
  - run limit errors
  - model limit errors
  - preflight/live loader count alignment
- snapshot tests:
  - persisted run corpus roundtrips
  - snapshot-only pagination
  - corrupt/missing snapshot behavior
  - trace resolution from snapshot
  - completed run snapshot precedence
- source-resolution tests:
  - project source loading
  - mixed provider rejection
  - playlist expansion
  - snapshot-vs-live source-id resolution.

## Proposed Architecture

Move only test code into a nested test tree:

- `src-tauri/src/analysis/corpus/tests/mod.rs`
- `src-tauri/src/analysis/corpus/tests/harness.rs`
- `src-tauri/src/analysis/corpus/tests/live.rs`
- `src-tauri/src/analysis/corpus/tests/preflight.rs`
- `src-tauri/src/analysis/corpus/tests/snapshot.rs`
- `src-tauri/src/analysis/corpus/tests/source_resolution.rs`

Keep production code in `src-tauri/src/analysis/corpus.rs`.

Replace the inline test module in `corpus.rs` with:

```rust
#[cfg(test)]
mod tests;
```

`tests/mod.rs` should declare the test modules:

```rust
mod harness;
mod live;
mod preflight;
mod snapshot;
mod source_resolution;
```

`tests/harness.rs` owns shared helpers that multiple test modules use. The thematic modules import helpers through `super::harness`.

This split keeps the existing Cargo test path prefix under `analysis::corpus::tests::`. Individual tests gain one more module segment, such as `analysis::corpus::tests::live::load_corpus_messages_filters_youtube_transcript_only_to_transcripts`. The implementation plan must update verification filters accordingly and must prevent green `0 tests` runs.

## File Responsibilities

`src-tauri/src/analysis/corpus.rs`

- Keep all production definitions and facade re-exports.
- Keep `#[cfg(test)] mod tests;`.
- Do not gain new test helper code.

`src-tauri/src/analysis/corpus/tests/mod.rs`

- Declare child test modules.
- Avoid owning shared helper logic directly unless a helper is used only to wire modules.

`src-tauri/src/analysis/corpus/tests/harness.rs`

- Own shared schema/setup and seed helpers.
- Own shared fixture constructors such as `sample_corpus`, `sample_run`, and `corpus_request`.
- Own shared metadata helpers used by more than one thematic module.
- Import production corpus items through the parent corpus module or through crate paths, preserving current test semantics.

`src-tauri/src/analysis/corpus/tests/live.rs`

- Own live corpus loading tests and migrated-history tests.
- Own tests that primarily assert `load_corpus_messages`, `live_corpus_ref`, and live document-kind filtering behavior.
- Use harness helpers instead of duplicating setup.

`src-tauri/src/analysis/corpus/tests/preflight.rs`

- Own pure preflight formula tests and async preflight tests.
- Own tests that compare `preflight_analysis_run` with `load_corpus_messages`.

`src-tauri/src/analysis/corpus/tests/snapshot.rs`

- Own snapshot persistence/page/trace/completed-run behavior tests.
- Use `persist_run_snapshot` and snapshot facade functions through the corpus facade.

`src-tauri/src/analysis/corpus/tests/source_resolution.rs`

- Own `resolve_analysis_sources`, `resolve_run_source_ids`, project-scope, mixed-provider, and playlist expansion tests.
- Keep playlist expansion coverage in this module, because it validates source-resolution behavior rather than live corpus loading.

## Visibility

This refactor is test-only, but moving tests out of the inline child module changes privacy boundaries.

Implementation should prefer one of these visibility strategies, in this order:

1. Keep helpers private to `tests/harness.rs` when used only in the test tree, and mark them `pub(super)` only when sibling test modules need them.
2. Keep thematic test helper functions private to their module.
3. Avoid widening production item visibility solely for tests.
4. If a production item is private and a moved test still needs it, first question whether that assertion belongs in a facade-level test. Only use `pub(crate)` or `pub(super)` on production code if no reasonable test-only alternative exists and the implementation plan names the reason.

Expected production visibility changes: none.

Expected test-helper visibility changes: helpers shared from `harness.rs` become `pub(super)`.

The expected `tests/harness.rs` helper contract is:

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

If a listed helper ends up used by only one thematic module after the split, the implementation may keep it private in that thematic module instead of exporting it from `harness.rs`. Do not widen any helper beyond `pub(super)`.

## Data Flow

No runtime data flow changes:

1. In-memory SQLite setup still creates the same schemas.
2. Seed helpers still insert the same rows and metadata.
3. Live corpus tests still exercise the same `load_corpus_messages` facade.
4. Preflight tests still exercise `preflight_analysis_run` and related helpers.
5. Snapshot tests still use the same snapshot facade and persisted snapshot rows.
6. Source-resolution tests still exercise the same project/source/playlist setup.

Test execution flow changes only by module path. The full command:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::
```

must still run the full corpus test suite and must not be a green `0 tests` run.

## Error Handling

No error behavior changes:

- expected `AppErrorKind` assertions remain identical;
- validation and internal error message assertions remain identical;
- corrupt compressed payload tests remain identical;
- snapshot corruption and unavailable snapshot assertions remain identical.

## Non-Goals

This slice does not:

- move production corpus code;
- change `live.rs`, `snapshot.rs`, or `source_resolution.rs` production APIs;
- change SQL, schema setup, seed data, ordering, or assertion contents;
- add new behavior tests;
- delete existing tests;
- change frontend, Tauri command payloads, migrations, or event payloads;
- convert tests to integration tests outside the crate.

## Test Module Mapping

The implementation plan should map existing tests approximately as follows.

Move to `tests/live.rs`:

- `default_analysis_corpus_excludes_migrated_history_documents`
- `opted_in_analysis_corpus_includes_migrated_rows_and_counts_preflight`
- `source_group_opt_in_includes_only_members_with_migrated_rows`
- `explicit_analysis_opt_in_with_zero_migrated_rows_keeps_current_corpus`
- `youtube_description_rows_use_typed_metadata_with_corrupt_source_blob`
- `youtube_description_missing_typed_metadata_skips_without_decoding_source_blob`
- `youtube_transcript_segment_evidence_uses_typed_source_context`
- `live_corpus_refs_use_local_item_ids`
- `preflight_ref_format_matches_corpus_loader_ref_format`
- `load_corpus_messages_returns_typed_internal_for_corrupt_live_document_content`
- `load_corpus_messages_orders_transcript_segments_by_document_order_not_ref`
- `youtube_corpus_mode_parses_wire_values_and_defaults`
- `load_corpus_messages_filters_telegram_to_telegram_message`
- `load_corpus_messages_filters_youtube_transcript_only_to_transcripts`
- `load_corpus_messages_includes_youtube_comment_only_in_comments_mode`
- `description_mode_creates_synthetic_description_message`

Move to `tests/preflight.rs`:

- `estimated_message_chars_match_report_chunk_accounting`
- `estimated_chunk_count_matches_chunk_boundary_behavior`
- `default_preflight_limits_are_conservative`
- `preflight_limit_error_reports_all_scale_dimensions`
- `preflight_limit_error_allows_runs_within_limits`
- `model_limit_preflight_allows_unknown_or_fitting_limits`
- `model_limit_preflight_reports_oversized_chunks`
- `preflight_counts_eligible_text_messages_for_sources`
- `preflight_ignores_media_only_items_without_text_content`
- `preflight_count_matches_loader_for_youtube_corpus_modes`

Move to `tests/snapshot.rs`:

- `run_snapshot_roundtrips_frozen_corpus`
- `list_run_snapshot_messages_page_reads_saved_snapshot_only`
- `list_run_snapshot_messages_page_returns_typed_internal_for_corrupt_snapshot_content`
- `list_run_snapshot_messages_page_starts_at_around_ref`
- `list_run_snapshot_messages_page_does_not_fall_back_to_live_source`
- `trace_resolution_does_not_fall_back_to_live_source_for_completed_missing_snapshot`
- `run_message_cursor_uses_ref_and_published_at`
- `load_run_corpus_messages_uses_snapshot_when_available`
- `load_run_corpus_messages_does_not_reconstruct_completed_capture_failed_from_live_rows`
- `captured_marker_with_missing_rows_returns_corrupt_snapshot_error`
- `source_group_membership_drift_after_capture_does_not_change_saved_run_corpus`

Move to `tests/source_resolution.rs`:

- `resolve_run_source_ids_prefers_snapshot_over_live_group_membership`
- `resolve_run_source_ids_loads_project_sources_without_snapshot`
- `playlist_expansion_excludes_unlinked_and_removed_rows`
- `resolve_analysis_sources_rejects_mixed_provider_project`
- `resolve_analysis_sources_preserves_no_linked_youtube_error_message`
- `resolve_analysis_sources_loads_single_provider_project`

If implementation discovers a test fits a different thematic module better, the plan may move it there only if the full corpus suite still runs the same number of real tests and no assertion is weakened.

## Implementation Hygiene

The implementation plan must include a pre-edit worktree guard:

```powershell
git status --short --untracked-files=all
```

If any target file is already modified, staged, or untracked before this task starts, inspect the baseline before editing:

```powershell
git diff -- src-tauri/src/analysis/corpus.rs
```

```powershell
git diff --cached -- src-tauri/src/analysis/corpus.rs
```

If `src-tauri/src/analysis/corpus/tests` exists before execution, inspect tracked changes under it:

```powershell
if (Test-Path -LiteralPath 'src-tauri/src/analysis/corpus/tests') {
    git diff -- src-tauri/src/analysis/corpus/tests
    git diff --cached -- src-tauri/src/analysis/corpus/tests
}
```

If any `src-tauri/src/analysis/corpus/tests/*.rs` file is untracked before execution, `git diff` will not show its contents. Inspect it directly before proceeding:

```powershell
if (Test-Path -LiteralPath 'src-tauri/src/analysis/corpus/tests') {
    Get-ChildItem -Recurse -File -LiteralPath 'src-tauri/src/analysis/corpus/tests' |
        ForEach-Object { $_.FullName; Get-Content -Raw -LiteralPath $_.FullName }
}
```

Do not stage pre-existing target-file changes into the test split commit.

The implementation plan must make the refactor commit conditional on fresh verification before commit, not only final verification after commit.

## Testing

Run commands from the repository root.

Before editing, establish the current baseline:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::
```

Expected: PASS and not a green `0 tests` run. Current snapshot at spec authoring: `43 passed`; do not require this exact count if nearby tests change before execution.

After editing and before committing, run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::
```

Expected: PASS and not a green `0 tests` run. The implementation plan must require the executor to inspect the output and confirm that the full `analysis::corpus::tests::` filter ran real tests after the module-path redirect. It must also require these four representative tests to appear in the post-change output:

- `analysis::corpus::tests::live::load_corpus_messages_filters_youtube_transcript_only_to_transcripts`
- `analysis::corpus::tests::preflight::preflight_count_matches_loader_for_youtube_corpus_modes`
- `analysis::corpus::tests::snapshot::run_snapshot_roundtrips_frozen_corpus`
- `analysis::corpus::tests::source_resolution::playlist_expansion_excludes_unlinked_and_removed_rows`

Also run consumer and compile checks:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::
```

Expected: PASS and not a green `0 tests` run.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml projects::data_range::tests::
```

Expected: PASS and not a green `0 tests` run.

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
```

Expected: PASS with no diff output.

After any formatting command or format check, confirm that no unrelated worktree drift appeared:

```powershell
git status --short --untracked-files=all
```

Expected: before the refactor commit, only implementation-owned files for this test split are newly modified or untracked relative to the pre-edit baseline. After the refactor commit, the status check is a baseline comparison: the refactor must not introduce new unintended files or diffs outside `src-tauri/src/analysis/corpus.rs` and `src-tauri/src/analysis/corpus/tests/*`. Any pre-existing baseline entries must either remain byte-for-byte unchanged or be separated from the refactor before staging.

```powershell
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
```

Expected: PASS. Existing warnings outside touched files may remain; new warnings mentioning `src-tauri/src/analysis/corpus.rs` or `src-tauri/src/analysis/corpus/tests/` are not acceptable.

## Commit Shape

Expected implementation-owned files:

- `src-tauri/src/analysis/corpus.rs`
- `src-tauri/src/analysis/corpus/tests/mod.rs`
- `src-tauri/src/analysis/corpus/tests/harness.rs`
- `src-tauri/src/analysis/corpus/tests/live.rs`
- `src-tauri/src/analysis/corpus/tests/preflight.rs`
- `src-tauri/src/analysis/corpus/tests/snapshot.rs`
- `src-tauri/src/analysis/corpus/tests/source_resolution.rs`

If any of these paths already exist or are dirty at the pre-edit guard, their exact status and contents become part of the baseline. The implementation commit may include only the net changes intentionally required for this test split; pre-existing tracked or untracked content must be separated first or left untouched and unstaged.

Expected implementation commit:

```text
refactor: split analysis corpus tests
```

The design spec and implementation plan should be committed separately from the Rust refactor.
