# Analysis Corpus Source Resolution Refactor Design

**Date:** 2026-06-30
**Status:** proposed next slice, ready for review before implementation planning
**Scope:** internal Rust refactor of `src-tauri/src/analysis/corpus.rs` and a new private nested corpus module.

## Goal

Reduce the responsibility of `src-tauri/src/analysis/corpus.rs` by extracting analysis source-scope resolution into a focused private module, without changing behavior, caller paths, database queries, or Tauri command contracts.

This is the next conservative slice after the snapshot extraction. It keeps the existing `analysis::corpus` facade stable while separating source selection and playlist expansion from live corpus loading and preflight logic.

## Current Shape

After the snapshot split, `src-tauri/src/analysis/corpus.rs` still owns several concerns:

- source scope resolution for single source, source group, and project runs;
- YouTube playlist expansion into linked video source IDs;
- source-resolution errors and error-code mapping;
- test-only run source resolution for snapshot-aware tests;
- live corpus loading from `analysis_documents` and migrated Telegram history;
- preflight size estimation and limit error formatting;
- YouTube corpus mode parsing and document-kind filtering;
- the shared sqlite test harness.

The source-resolution behavior is clustered around these definitions:

- `ResolvedAnalysisSources`
- `AnalysisSourceResolutionErrorCode`
- `AnalysisSourceResolutionError`
- `AnalysisSourceScopeRow`
- `load_source_scope_row`
- `linked_playlist_video_source_ids`
- `count_skipped_unlinked_playlist_items`
- `resolve_analysis_sources`
- `push_scope_source`
- `resolve_run_source_ids`

Consumers are narrow:

- `analysis/report.rs` imports `resolve_analysis_sources` and `AnalysisSourceResolutionError` from `super::corpus`.
- `projects/data_range.rs` uses the root re-exported `resolve_analysis_sources` and `AnalysisSourceResolutionErrorCode`.
- `analysis/mod.rs` re-exports `resolve_analysis_sources`, `AnalysisSourceResolutionError`, and `AnalysisSourceResolutionErrorCode`.
- `analysis/corpus.rs` tests call `resolve_analysis_sources` and test-only `resolve_run_source_ids` through the current module.

## Proposed Architecture

Create a new nested private module:

- `src-tauri/src/analysis/corpus/source_resolution.rs`

Keep `src-tauri/src/analysis/corpus.rs` as the internal facade:

- add `mod source_resolution;`
- re-export the existing crate-visible source-resolution surface with `pub(crate) use self::source_resolution::{...};`
- keep existing consumers importing through `super::corpus` or existing root re-exports; do not make callers import from `corpus::source_resolution`.

Move these items from `corpus.rs` to `corpus/source_resolution.rs`:

- `ResolvedAnalysisSources`
- `AnalysisSourceResolutionErrorCode`
- `AnalysisSourceResolutionError`
- `impl AnalysisSourceResolutionErrorCode`
- `impl AnalysisSourceResolutionError`
- `impl From<AppError> for AnalysisSourceResolutionError`
- `AnalysisSourceScopeRow`
- `load_source_scope_row`
- `linked_playlist_video_source_ids`
- `count_skipped_unlinked_playlist_items`
- `resolve_analysis_sources`
- `push_scope_source`
- `resolve_run_source_ids`

Keep these items in `corpus.rs` for this slice:

- `YoutubeCorpusMode`
- `CorpusLoadRequest`
- live corpus loaders;
- preflight structs and helpers;
- `push_analysis_document_kind_filter`;
- snapshot facade re-exports;
- all existing tests and the shared sqlite test harness.

The test module stays in `corpus.rs` intentionally. Source-resolution tests share `snapshot_pool`, `create_project_scope_schema`, source seed helpers, and live corpus helpers with other corpus tests. Moving those tests should wait until a later `corpus/test_support.rs` extraction.

## Visibility

`corpus/source_resolution.rs` should expose only the items already crate-visible today:

- `pub(crate) struct ResolvedAnalysisSources`
- `pub(crate) enum AnalysisSourceResolutionErrorCode`
- `pub(crate) struct AnalysisSourceResolutionError`
- `pub(crate) async fn resolve_analysis_sources(...)`
- `#[cfg(test)] pub(crate) async fn resolve_run_source_ids(...)`

Private source-resolution helpers stay private inside `source_resolution.rs`:

- `AnalysisSourceScopeRow`
- `load_source_scope_row`
- `linked_playlist_video_source_ids`
- `count_skipped_unlinked_playlist_items`
- `push_scope_source`

`source_resolution` itself remains a private nested module. Existing callers keep using the current paths through the `corpus` facade and root re-exports where they already exist.

## Imports

After the move, `corpus/source_resolution.rs` owns source-resolution-specific imports:

- `std::collections::HashSet`
- `sqlx::{Pool, Sqlite}`
- `crate::error::{AppError, AppResult}`
- `crate::analysis::models::AnalysisRunDetail`, gated with `#[cfg(test)]`
- `crate::analysis::store::fetch_source_group`

It also needs test-only access to the scope constants:

- `super::super::{ANALYSIS_SCOPE_TYPE_PROJECT, ANALYSIS_SCOPE_TYPE_SINGLE_SOURCE, ANALYSIS_SCOPE_TYPE_SOURCE_GROUP}`, gated with `#[cfg(test)]`

The exact path may be adjusted in implementation if `rustfmt` or local module layout makes a different `super` path clearer, but the constants must remain owned by `analysis/mod.rs`.

`corpus.rs` should remove imports that only source-resolution code used:

- `std::collections::HashSet`, if no remaining live-corpus code uses it;
- `super::store::fetch_source_group`, if no remaining code uses it;
- `#[cfg(test)] use super::models::AnalysisRunDetail`, if only `resolve_run_source_ids` used it;
- the test-only scope constant import block, if only `resolve_run_source_ids` used it.

The implementation plan must verify unused imports after the move and keep the diff free of warning debt introduced by this slice.

## Data Flow

No runtime data flow changes:

1. Report startup still resolves exactly one selected scope through `resolve_analysis_sources`.
2. Single-source analysis still loads the source row directly.
3. Source-group analysis still loads group members through `fetch_source_group`.
4. Project analysis still loads `project_sources` in `added_at ASC, s.id ASC` order and rejects mixed-provider projects.
5. YouTube playlist sources still expand to linked, non-removed video source IDs in playlist order.
6. Unlinked, non-removed playlist items are still counted in `skipped_unlinked_playlist_items`.
7. YouTube scopes with no linked videos still return `NoLinkedYoutubeVideos`.
8. Test-only `resolve_run_source_ids` still prefers saved snapshot source IDs before falling back to live single-source, group, or project membership.

## Error Handling

Preserve the current error behavior exactly:

- invalid or ambiguous selected scopes still return `AppError::validation("Select exactly one analysis scope")`;
- missing single sources still return `AppError::not_found(format!("Source {source_id} not found"))`;
- missing source groups still return `AppError::not_found(format!("Analysis source group {group_id} not found"))`;
- empty projects still return `AppError::validation("Project does not contain any sources")`;
- mixed-provider projects still use `AnalysisSourceResolutionErrorCode::MixedProviderProject`;
- YouTube scopes with only unlinked playlist rows still use `AnalysisSourceResolutionErrorCode::NoLinkedYoutubeVideos` and preserve its message;
- database query failures still map through `AppError::database`.

No new error codes, messages, or user-facing strings are introduced in this slice.

## Non-Goals

This slice does not:

- change database schema, migrations, or indexes;
- change project, source-group, or playlist ordering;
- change playlist expansion rules for removed or unlinked rows;
- change live corpus loading from `analysis_documents`;
- change migrated Telegram history behavior;
- change preflight estimates or limits;
- move the shared sqlite test harness;
- move source-resolution tests out of `corpus.rs`;
- change frontend, Tauri command payloads, or event payloads.

## Testing

Run commands from the repository root with the manifest path because `Cargo.toml` is under `src-tauri/`.

Focused source-resolution checks:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::playlist_expansion_excludes_unlinked_and_removed_rows
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::resolve_analysis_sources
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::resolve_run_source_ids
```

Broader regression checks:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml projects::data_range::tests::
```

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
```

```powershell
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
```

Expected result: all commands pass. The focused `resolve_analysis_sources` and `resolve_run_source_ids` filters must run real tests, not green `0 tests` runs.

## Risks And Mitigations

**Risk:** changing source-resolution ordering while moving SQL.

**Mitigation:** preserve the exact SQL and run project and playlist source-resolution tests.

**Risk:** accidentally widening the source-resolution module API.

**Mitigation:** keep `source_resolution` private and re-export only the current crate-visible items from `corpus.rs`.

**Risk:** breaking root re-exports used by `projects/data_range.rs`.

**Mitigation:** keep `analysis/mod.rs` imports unchanged through the `corpus` facade and run `projects::data_range::tests::` plus `cargo check --all-targets`.

**Risk:** mixing source-resolution extraction with test harness extraction.

**Mitigation:** keep the test harness and tests in `corpus.rs` for this slice. Extract `corpus/test_support.rs` only as a later dedicated refactor.

## Follow-Up Slices

After this slice lands cleanly, good next candidates are:

1. Extract `corpus/live.rs` for `load_corpus_messages`, Telegram migrated history, and analysis document loading.
2. Extract `corpus/preflight.rs` for preflight structs, input-size estimation, and limit error formatting.
3. Extract shared `corpus/test_support.rs` so snapshot, source-resolution, live, and preflight tests can move next to their modules.
