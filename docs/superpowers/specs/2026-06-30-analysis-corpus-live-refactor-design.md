# Analysis Corpus Live Loading Refactor Design

**Date:** 2026-06-30
**Status:** design approved, written spec ready for review before implementation planning
**Scope:** internal Rust refactor of `src-tauri/src/analysis/corpus.rs` and a new private nested corpus module.

## Goal

Reduce the responsibility of `src-tauri/src/analysis/corpus.rs` by extracting live corpus loading into a focused private module, without changing behavior, caller paths, SQL, ordering, error mapping, or Tauri command contracts.

This is the next conservative slice after the snapshot and source-resolution extractions. It keeps `analysis::corpus` as the stable facade while separating live corpus reads from preflight estimation and limit formatting.

## Current Shape

After the snapshot and source-resolution splits, `src-tauri/src/analysis/corpus.rs` still owns these concerns:

- corpus request and YouTube corpus mode types;
- live corpus loading from `analysis_documents`;
- Telegram current and migrated-history corpus loading;
- live corpus ref construction and migrated-history metadata construction;
- analysis document kind filtering used by live loading and project data range;
- preflight input-size estimation and limit error formatting;
- snapshot and source-resolution facade re-exports;
- all existing tests and the shared sqlite test harness.

The live-loading behavior is clustered around these definitions:

- `live_corpus_ref`
- `load_corpus_messages`
- `telegram_history_metadata_zstd`
- `TelegramCorpusRow`
- `fetch_telegram_corpus_rows`
- `load_telegram_corpus_messages`
- `AnalysisDocumentRow`
- `push_analysis_document_kind_filter`
- `load_analysis_document_messages`

Consumers are narrow:

- `analysis/report.rs` imports `load_corpus_messages`, `preflight_analysis_run`, `preflight_limit_error`, `CorpusLoadRequest`, and `YoutubeCorpusMode` from `super::corpus`.
- `projects/data_range.rs` uses the root re-exported `push_analysis_document_kind_filter` and `YoutubeCorpusMode`.
- `analysis/mod.rs` re-exports `push_analysis_document_kind_filter` and `YoutubeCorpusMode`.
- `analysis/corpus.rs::preflight_analysis_run` is the internal consumer of the `load_corpus_messages` facade after the move.
- `analysis/corpus.rs` tests call `load_corpus_messages`, `live_corpus_ref`, and preflight helpers through the current module.
- `analysis/store.rs` imports `YoutubeCorpusMode` directly from `analysis::corpus`; this refactor must not move that enum.

## Proposed Architecture

Create a new nested private module:

- `src-tauri/src/analysis/corpus/live.rs`

Keep `src-tauri/src/analysis/corpus.rs` as the internal facade:

- add `mod live;`;
- re-export the existing crate-visible live-loading surface from `live.rs`, for example:

  ```rust
  #[allow(unused_imports)]
  pub(crate) use self::live::live_corpus_ref;
  pub(crate) use self::live::{load_corpus_messages, push_analysis_document_kind_filter};
  ```

- keep existing consumers importing through `super::corpus`, `self::corpus`, or existing root re-exports; do not make callers import from `corpus::live`.

Move these items from `corpus.rs` to `corpus/live.rs`:

- `live_corpus_ref`
- `load_corpus_messages`
- `telegram_history_metadata_zstd`
- `TelegramCorpusRow`
- `fetch_telegram_corpus_rows`
- `load_telegram_corpus_messages`
- `AnalysisDocumentRow`
- `push_analysis_document_kind_filter`
- `load_analysis_document_messages`

Keep these items in `corpus.rs` for this slice:

- `YoutubeCorpusMode`
- `CorpusLoadRequest`
- `estimate_message_input_chars`
- `estimate_preflight_chunk_count`
- `AnalysisRunPreflightLimits`
- `AnalysisRunPreflight`
- `preflight_analysis_run`
- `preflight_limit_error`
- `model_limit_preflight_error`
- snapshot and source-resolution facade re-exports;
- all existing tests and the shared sqlite test harness.

The test module stays in `corpus.rs` intentionally. Live-loading tests share `snapshot_pool`, source seed helpers, typed YouTube helper tables, migrated Telegram helpers, preflight assertions, and snapshot tests with the rest of the corpus module. Moving those tests should wait until a later `corpus/test_support.rs` extraction.

## Visibility

`corpus/live.rs` should expose only the items already crate-visible today:

```rust
pub(crate) async fn load_corpus_messages(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    request: &CorpusLoadRequest,
) -> AppResult<Vec<CorpusMessage>>;

pub(crate) fn push_analysis_document_kind_filter(
    query: &mut sqlx::QueryBuilder<'_, sqlx::Sqlite>,
    source_type: &str,
    youtube_corpus_mode: YoutubeCorpusMode,
    table_alias: &str,
) -> AppResult<()>;

pub(crate) fn live_corpus_ref(source_id: i64, item_id: i64) -> String;
```

Preserve the current `live_corpus_ref` facade path. If that re-export is unused in non-test builds, use a targeted `#[allow(unused_imports)]` on the re-export rather than dropping the facade path.

Because `live.rs` is a sibling module of the remaining `corpus.rs` definitions, preserve the current visibility of the types it consumes:

- all `CorpusLoadRequest` fields stay `pub(crate)`;
- `YoutubeCorpusMode::includes_description()` and `YoutubeCorpusMode::includes_comments()` stay `pub(crate)`;
- `YoutubeCorpusMode::from_wire()` and `YoutubeCorpusMode::as_wire()` stay `pub(crate)` because current non-live consumers still call them through the corpus facade.

Private live-loading helpers stay private inside `live.rs`:

- `telegram_history_metadata_zstd`
- `TelegramCorpusRow`
- `fetch_telegram_corpus_rows`
- `load_telegram_corpus_messages`
- `AnalysisDocumentRow`
- `load_analysis_document_messages`

`live` itself remains a private nested module. Existing callers keep using the current paths through the `corpus` facade and root re-exports where they already exist.

## Imports

After the move, `corpus/live.rs` owns live-loading-specific imports:

- `sqlx::{Pool, QueryBuilder, Sqlite}`
- `super::{CorpusLoadRequest, YoutubeCorpusMode}`
- `crate::analysis::models::CorpusMessage`
- `crate::compression::{compress_json_bytes, decompress_text}`
- `crate::error::{internal_error, AppError, AppResult}`

`corpus.rs` should remove imports that only live-loading code used:

- `QueryBuilder`, if no remaining preflight code uses it;
- `super::models::CorpusMessage`, if only live-loading code used it outside tests;
- `compress_json_bytes`, if only live-loading code used it outside tests;
- `decompress_text`;
- `internal_error`, if only live-loading code used it;
- `AppError`, if only live-loading code used it.

Keep `Pool`, `Sqlite`, and `AppResult` in `corpus.rs` for preflight. The implementation plan must verify unused imports after the move and keep the diff free of warning debt introduced by this slice.

## Data Flow

No runtime data flow changes:

1. `load_corpus_messages` still returns an empty vector when `source_ids` is empty.
2. Telegram requests still load current `analysis_documents` rows first and append migrated Telegram rows only when `include_migrated_history` is true.
3. Migrated Telegram rows still require `items.item_kind = 'telegram_message'`, `tm.is_migrated_history = 1`, `tm.migration_domain = 'migrated_from_chat'`, non-null text content, and `content_kind IN ('text_only', 'text_with_media')`.
4. Telegram rows still build metadata with `history_scope`, `migration_domain`, `history_peer_kind`, and `history_peer_id`.
5. Telegram messages still sort by `published_at ASC`, `source_id ASC`, and `ref ASC` after current and migrated rows are merged.
6. Non-Telegram live corpus loading still reads from `analysis_documents`.
7. YouTube corpus mode filtering still includes transcript rows always, description rows only when `includes_description()`, and comment rows only when `includes_comments()`.
8. Live analysis document rows still order by `d.published_at ASC, d.source_id ASC, d.document_order ASC, d.id ASC`.
9. `preflight_analysis_run` still calls the same facade `load_corpus_messages` before estimating message sizes and chunks.
10. Project data range still uses the same `push_analysis_document_kind_filter` through the root analysis facade.

## Error Handling

Preserve the current error behavior exactly:

- unsupported live corpus source types still return `AppError::validation(format!("Unsupported analysis corpus source_type '{other}'"))`;
- database query failures still map through `AppError::database`;
- JSON serialization for Telegram migrated-history metadata still maps through `internal_error`;
- compression and decompression failures still map through `internal_error`;
- no new error codes, messages, or user-facing strings are introduced in this slice.

## Non-Goals

This slice does not:

- change database schema, migrations, or indexes;
- change `CorpusLoadRequest` or `YoutubeCorpusMode`;
- change source resolution, snapshot loading, or preflight formulas;
- change Telegram migrated-history eligibility rules;
- change YouTube document-kind filtering or ordering;
- move tests out of `corpus.rs`;
- move shared test helpers into `corpus/test_support.rs`;
- change frontend, Tauri command payloads, or event payloads.

## Implementation Hygiene

The implementation plan must include a pre-edit worktree guard:

```powershell
git status --short --untracked-files=all
```

If `src-tauri/src/analysis/corpus.rs` or `src-tauri/src/analysis/corpus/live.rs` is dirty before execution, inspect the baseline before editing. If `corpus/live.rs` is already untracked, `git diff` will not show its contents, so inspect it directly with:

```powershell
git diff -- src-tauri/src/analysis/corpus.rs src-tauri/src/analysis/corpus/live.rs
```

```powershell
git diff --cached -- src-tauri/src/analysis/corpus.rs src-tauri/src/analysis/corpus/live.rs
```

```powershell
Get-Content -Raw -LiteralPath 'src-tauri/src/analysis/corpus/live.rs'
```

Do not stage pre-existing target-file changes into the live-loading refactor commit.

The implementation plan must also run these consumer baseline checks before editing and again before creating the refactor commit:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::report::tests::
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml projects::data_range::tests::
```

Both filtered test commands must run real tests, not green `0 tests` runs.

If formatting fixes are needed during implementation, run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml
```

Then inspect the changed file list before staging:

```powershell
git status --short
```

Expected behavioral Rust changes for this slice are limited to:

- `src-tauri/src/analysis/corpus.rs`
- `src-tauri/src/analysis/corpus/live.rs`

Any unrelated `rustfmt` drift must be excluded from the refactor commit or handled in a separate format-only commit before final verification.

## Testing

Run commands from the repository root with the manifest path because `Cargo.toml` is under `src-tauri/`.

Focused live-loading checks:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::load_corpus_messages
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::live_corpus_refs_use_local_item_ids
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::default_analysis_corpus_excludes_migrated_history_documents
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::opted_in_analysis_corpus_includes_migrated_rows_and_counts_preflight
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::explicit_analysis_opt_in_with_zero_migrated_rows_keeps_current_corpus
```

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::source_group_opt_in_includes_only_members_with_migrated_rows
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

Expected result: all commands pass. Every filtered `cargo test` command listed above must run real tests, not green `0 tests` runs. The `analysis::corpus::tests::load_corpus_messages` filter should include multiple live-loading tests whose names start with `load_corpus_messages_`.

## Risks And Mitigations

**Risk:** changing live corpus row ordering while moving SQL.

**Mitigation:** preserve the exact SQL and sort logic, and run live corpus ordering and full corpus tests.

**Risk:** breaking project data range by moving `push_analysis_document_kind_filter`.

**Mitigation:** keep the `analysis::corpus` facade and `analysis/mod.rs` root re-export unchanged, and run `projects::data_range::tests::` plus `cargo check --all-targets`.

**Risk:** accidentally mixing live-loading extraction with preflight extraction.

**Mitigation:** keep `CorpusLoadRequest`, `YoutubeCorpusMode`, preflight structs, estimation helpers, and limit error formatting in `corpus.rs` for this slice.

**Risk:** losing the test-only `live_corpus_ref` facade path.

**Mitigation:** re-export `live_corpus_ref` from `corpus.rs` with targeted `#[allow(unused_imports)]` if needed.

## Follow-Up Slices

After this slice lands cleanly, good next candidates are:

1. Extract `corpus/preflight.rs` for preflight structs, input-size estimation, and limit error formatting.
2. Extract shared `corpus/test_support.rs` so snapshot, source-resolution, live, and preflight tests can move next to their modules.
