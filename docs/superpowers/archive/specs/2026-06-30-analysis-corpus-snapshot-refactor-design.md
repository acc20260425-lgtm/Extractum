# Analysis Corpus Snapshot Refactor Design

**Date:** 2026-06-30
**Status:** snapshot-first slice approved, ready for spec review before implementation planning
**Scope:** internal Rust refactor of `src-tauri/src/analysis/corpus.rs` and a new private nested corpus module.

## Goal

Reduce the responsibility of `src-tauri/src/analysis/corpus.rs` by extracting saved run snapshot loading and pagination into a focused private module, without changing behavior or public command contracts.

This is the next conservative slice after the `AnalysisState` and event-helper extraction. It keeps the existing `analysis::corpus` call surface stable while making the largest analysis file easier to scan before deeper live-corpus and source-resolution refactors.

## Current Shape

`src-tauri/src/analysis/corpus.rs` is about 3,300 lines and currently owns several separate concerns:

- source scope resolution for single source, source group, and project runs;
- YouTube corpus mode parsing and document-kind filtering;
- live corpus loading from `analysis_documents` and migrated Telegram history;
- preflight size estimation and limit error formatting;
- saved run snapshot loading, pagination, trace-resolution corpus loading, and defensive snapshot validation;
- a large shared sqlite test harness plus tests for all of the above.

The snapshot behavior is clustered around these definitions:

- `load_run_snapshot_messages`
- `ListRunSnapshotMessagesRequest`
- `decode_optional_metadata_json`
- `run_message_from_snapshot_row`
- `list_run_snapshot_messages_page`
- `load_run_corpus_messages`
- `load_trace_resolution_messages`
- `captured_snapshot_missing_error`
- `ensure_captured_snapshot_rows`

Consumers are narrow:

- `analysis/mod.rs` imports `list_run_snapshot_messages_page`, `load_trace_resolution_messages`, and `ListRunSnapshotMessagesRequest` from `self::corpus`.
- `analysis/chat.rs` imports `load_run_snapshot_messages` from `super::corpus`.
- `analysis/corpus.rs` tests call the snapshot helpers through the current module.
- `load_run_corpus_messages` is currently kept for test coverage and future internal use with `#[allow(dead_code)]`.

## Proposed Architecture

Create a new nested private module:

- `src-tauri/src/analysis/corpus/snapshot.rs`

Keep `src-tauri/src/analysis/corpus.rs` as the internal facade for the existing corpus API:

- add `mod snapshot;`
- re-export only the current crate-visible snapshot surface with `pub(crate) use self::snapshot::{...};`
- keep existing consumers importing from `super::corpus` / `self::corpus`; do not make callers import from `corpus::snapshot`.

Move these items from `corpus.rs` to `corpus/snapshot.rs`:

- `load_run_snapshot_messages`
- `ListRunSnapshotMessagesRequest`
- `decode_optional_metadata_json`
- `run_message_from_snapshot_row`
- `list_run_snapshot_messages_page`
- `load_run_corpus_messages`
- `load_trace_resolution_messages`
- `captured_snapshot_missing_error`
- `ensure_captured_snapshot_rows`

Keep these items in `corpus.rs` for this slice:

- source resolution structs and helpers;
- `YoutubeCorpusMode`;
- `CorpusLoadRequest`;
- live corpus loaders;
- preflight structs and helpers;
- `push_analysis_document_kind_filter`;
- all existing tests and the shared sqlite test harness.

The test module stays in `corpus.rs` intentionally. The snapshot tests depend on `snapshot_pool`, `sample_run`, `sample_corpus`, and helper setup that are also shared by live-corpus and source-resolution tests. Moving those tests now would require a second refactor to create shared `corpus/test_support.rs`. That is a useful follow-up, but it is outside this implementation slice.

## Visibility

`corpus/snapshot.rs` should expose only the functions and request type that are already crate-visible today:

- `pub(crate) async fn load_run_snapshot_messages(...)`
- `pub(crate) struct ListRunSnapshotMessagesRequest`
- `pub(crate) async fn list_run_snapshot_messages_page(...)`
- `#[allow(dead_code)] pub(crate) async fn load_run_corpus_messages(...)`
- `pub(crate) async fn load_trace_resolution_messages(...)`

Snapshot-only helpers stay private inside `snapshot.rs`:

- `decode_optional_metadata_json`
- `run_message_from_snapshot_row`
- `captured_snapshot_missing_error`
- `ensure_captured_snapshot_rows`

`snapshot` itself remains a private nested module. Existing callers keep using the current paths through the `corpus` facade and root re-exports where they already exist, not `corpus::snapshot::{...}`.

## Imports

After the move, `corpus/snapshot.rs` owns snapshot-specific imports:

- `sqlx::{Pool, Sqlite}`
- `crate::compression::{decompress_bytes, decompress_text}`
- `crate::error::{internal_error, AppError, AppResult}`
- `crate::analysis::models::{AnalysisRunDetail, AnalysisRunMessage, AnalysisRunMessageCursor, AnalysisRunMessagesPage, AnalysisSnapshotState, CorpusMessage, StoredRunSnapshotRow}`

`corpus.rs` should remove imports that only snapshot code used:

- `decompress_bytes`, if no remaining live-corpus code uses it;
- `AnalysisRunMessage`, `AnalysisRunMessageCursor`, `AnalysisRunMessagesPage`, `AnalysisSnapshotState`, and `StoredRunSnapshotRow`, unless the retained test module still needs direct imports from `models`.

The implementation plan must verify unused imports after the move and keep the diff free of warning debt introduced by this slice.

## Data Flow

No runtime data flow changes:

1. Report runs still capture snapshots through `analysis/store.rs`.
2. Chat still loads saved snapshot corpus through `load_run_snapshot_messages`.
3. The run-messages command still pages `analysis_run_messages` through `list_run_snapshot_messages_page`.
4. Trace resolution still loads only saved run snapshot rows through `load_trace_resolution_messages`.
5. Captured snapshot markers with zero rows still return the same defensive internal error.
6. Completed or capture-failed runs still do not fall back to live source rows through these snapshot helpers.

## Error Handling

Preserve the current error behavior exactly:

- database query failures continue mapping through `AppError::database`;
- corrupt snapshot content and corrupt metadata continue mapping through `internal_error`;
- `list_run_snapshot_messages_page` continues returning typed internal errors for corrupt snapshot content rather than wrapping them as database errors;
- `ensure_captured_snapshot_rows` continues using the same message format from `captured_snapshot_missing_error`.

No new error codes, messages, or user-facing strings are introduced in this slice.

## Non-Goals

This slice does not:

- change database schema, migrations, or snapshot storage format;
- change pagination ordering, cursor semantics, page size clamping, or `around_ref` behavior;
- change live corpus loading from `analysis_documents`;
- change migrated Telegram history behavior;
- change source resolution, playlist expansion, or project validation;
- move the shared sqlite test harness;
- split preflight or YouTube corpus mode code;
- change frontend, Tauri command payloads, or event payloads.

## Testing

Run commands from the repository root with the manifest path because `Cargo.toml` is under `src-tauri/`.

Focused snapshot checks:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::run_snapshot_roundtrips_frozen_corpus
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::list_run_snapshot_messages_page
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::load_run_corpus_messages
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::trace_resolution_does_not_fall_back_to_live_source_for_completed_missing_snapshot
```

Broader regression checks:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::chat::tests::
```

```powershell
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
```

Expected result: all commands pass. The focused `list_run_snapshot_messages_page` and `load_run_corpus_messages` filters must run real tests, not green `0 tests` runs.

## Risks And Mitigations

**Risk:** widening or narrowing the internal snapshot API by accident.

**Mitigation:** keep `snapshot` private, re-export only the existing crate-visible items from `corpus.rs`, and leave all consumers importing through `corpus`.

**Risk:** changing fallback behavior while moving code.

**Mitigation:** run snapshot tests that explicitly assert saved snapshots are used and live rows are not reconstructed for completed or capture-failed runs.

**Risk:** creating import warning debt.

**Mitigation:** remove imports that moved to `snapshot.rs` and finish with `cargo check --all-targets`.

**Risk:** mixing implementation extraction with test harness extraction.

**Mitigation:** keep the test harness and tests in `corpus.rs` for this slice. Extract `corpus/test_support.rs` only as a later dedicated refactor.

## Follow-Up Slices

After this slice lands cleanly, good next candidates are:

1. Extract `corpus/source_resolution.rs` for `resolve_analysis_sources`, playlist expansion, and project/source-group scope handling.
2. Extract `corpus/live.rs` for `load_corpus_messages`, Telegram migrated history, and analysis document loading.
3. Extract shared `corpus/test_support.rs` so snapshot, live, source-resolution, and preflight tests can move next to their modules.
