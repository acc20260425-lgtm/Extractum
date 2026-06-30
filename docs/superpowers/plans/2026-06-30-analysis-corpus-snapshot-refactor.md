# Analysis Corpus Snapshot Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract saved analysis run snapshot loading and pagination from `src-tauri/src/analysis/corpus.rs` into `src-tauri/src/analysis/corpus/snapshot.rs` without changing behavior or caller paths.

**Architecture:** Keep `analysis::corpus` as the internal facade. Add a private nested `snapshot` module, move snapshot-specific implementation into it, and re-export only the crate-visible functions and request type that callers already use. Leave live corpus loading, source resolution, preflight logic, and the shared sqlite test harness in `corpus.rs`.

**Tech Stack:** Rust, Tauri backend, SQLx SQLite queries, zstd JSON/text compression helpers, Cargo test/check/fmt.

## Global Constraints

- Internal Rust refactor only: no frontend, Tauri command payload, event payload, SQL schema, or migration changes.
- Preserve existing caller paths through `super::corpus` / `self::corpus`; do not make callers import `corpus::snapshot`.
- `src-tauri/src/analysis/corpus/snapshot.rs` must remain a private nested module.
- The shared sqlite test harness and all existing tests stay in `src-tauri/src/analysis/corpus.rs` for this slice.
- Preserve pagination ordering, cursor semantics, page size clamping, `around_ref` behavior, snapshot fallback behavior, and error messages.
- Run commands from the repository root with `--manifest-path src-tauri/Cargo.toml`.
- Use `cargo fmt --manifest-path src-tauri/Cargo.toml`; after formatting inspect `git diff --name-only` and keep unrelated rustfmt drift out of the refactor commit.

---

## File Structure

- Create `src-tauri/src/analysis/corpus/snapshot.rs`
  - Owns saved run snapshot reads, run-message pagination, trace-resolution snapshot loading, and snapshot defensive validation.
- Modify `src-tauri/src/analysis/corpus.rs`
  - Adds `mod snapshot;`.
  - Re-exports existing snapshot API from `snapshot`.
  - Removes moved snapshot definitions and imports.
  - Keeps live corpus, source resolution, preflight, and tests.
- No changes expected in `src-tauri/src/analysis/mod.rs`, `src-tauri/src/analysis/chat.rs`, or `src-tauri/src/projects/data_range.rs`.
  - If those files need edits, stop and review why the facade re-export did not preserve existing paths.

---

### Task 1: Decouple Live-Corpus Metadata Test From Snapshot Helper

**Files:**
- Modify: `src-tauri/src/analysis/corpus.rs`

**Interfaces:**
- Consumes existing test helper:

```rust
fn decode_message_metadata_for_test(message: &CorpusMessage) -> serde_json::Value
```

- Produces no new production API. This task only makes the live-corpus migrated-history test stop calling `decode_optional_metadata_json`, so that helper can become private inside `snapshot.rs` in Task 2.

- [x] **Step 1: Run the characterization test before editing**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::opted_in_analysis_corpus_includes_migrated_rows_and_counts_preflight
```

Expected: PASS with output containing `1 passed`. If this fails before editing, stop and inspect the existing failure.

- [x] **Step 2: Replace the snapshot-helper call in the migrated-history test**

In `src-tauri/src/analysis/corpus.rs`, inside `opted_in_analysis_corpus_includes_migrated_rows_and_counts_preflight`, replace:

```rust
        let migrated_metadata =
            super::decode_optional_metadata_json(corpus[0].metadata_zstd.as_deref())
                .expect("decode metadata")
                .expect("metadata");
```

with:

```rust
        let migrated_metadata = decode_message_metadata_for_test(&corpus[0]);
```

Do not move `decode_message_metadata_for_test`; it remains in the existing test module and is already used by YouTube metadata tests.

- [x] **Step 3: Run the same characterization test after editing**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::opted_in_analysis_corpus_includes_migrated_rows_and_counts_preflight
```

Expected: PASS with output containing `1 passed`.

- [x] **Step 4: Commit the test-only preparation**

Run:

```powershell
git add -- src-tauri/src/analysis/corpus.rs
git commit -m "test: decouple corpus metadata test from snapshot helper"
```

Expected: commit succeeds and includes only `src-tauri/src/analysis/corpus.rs`.

---

### Task 2: Extract Snapshot Module Behind The Corpus Facade

**Files:**
- Create: `src-tauri/src/analysis/corpus/snapshot.rs`
- Modify: `src-tauri/src/analysis/corpus.rs`

**Interfaces:**
- Consumes from `crate::analysis::models`:

```rust
AnalysisRunDetail
AnalysisRunMessage
AnalysisRunMessageCursor
AnalysisRunMessagesPage
AnalysisSnapshotState
CorpusMessage
StoredRunSnapshotRow
```

- Produces through `src-tauri/src/analysis/corpus.rs` facade:

```rust
pub(crate) async fn load_run_snapshot_messages(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    run_id: i64,
) -> crate::error::AppResult<Vec<crate::analysis::models::CorpusMessage>>;

pub(crate) struct ListRunSnapshotMessagesRequest {
    pub(crate) run_id: i64,
    pub(crate) after: Option<crate::analysis::models::AnalysisRunMessageCursor>,
    pub(crate) limit: usize,
    pub(crate) source_id: Option<i64>,
    pub(crate) around_ref: Option<String>,
}

pub(crate) async fn list_run_snapshot_messages_page(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    request: ListRunSnapshotMessagesRequest,
) -> crate::error::AppResult<crate::analysis::models::AnalysisRunMessagesPage>;

#[allow(dead_code)]
pub(crate) async fn load_run_corpus_messages(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    run: &crate::analysis::models::AnalysisRunDetail,
) -> crate::error::AppResult<Vec<crate::analysis::models::CorpusMessage>>;

pub(crate) async fn load_trace_resolution_messages(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    run: &crate::analysis::models::AnalysisRunDetail,
) -> crate::error::AppResult<Vec<crate::analysis::models::CorpusMessage>>;
```

- Snapshot-only helpers remain private inside `snapshot.rs`:

```rust
fn decode_optional_metadata_json(
    metadata_zstd: Option<&[u8]>,
) -> crate::error::AppResult<Option<serde_json::Value>>;

fn run_message_from_snapshot_row(
    row: crate::analysis::models::StoredRunSnapshotRow,
) -> crate::error::AppResult<crate::analysis::models::AnalysisRunMessage>;

fn captured_snapshot_missing_error(run_id: i64) -> crate::error::AppError;

fn ensure_captured_snapshot_rows(
    run: &crate::analysis::models::AnalysisRunDetail,
    snapshot: &[crate::analysis::models::CorpusMessage],
) -> crate::error::AppResult<()>;
```

- [x] **Step 1: Create the nested module file with snapshot-specific imports**

Create `src-tauri/src/analysis/corpus/snapshot.rs` with this header:

```rust
use sqlx::{Pool, Sqlite};

use crate::analysis::models::{
    AnalysisRunDetail, AnalysisRunMessage, AnalysisRunMessageCursor, AnalysisRunMessagesPage,
    AnalysisSnapshotState, CorpusMessage, StoredRunSnapshotRow,
};
use crate::compression::{decompress_bytes, decompress_text};
use crate::error::{internal_error, AppError, AppResult};
```

- [x] **Step 2: Move snapshot definitions into `snapshot.rs`**

Move this contiguous block from `src-tauri/src/analysis/corpus.rs` into `src-tauri/src/analysis/corpus/snapshot.rs`, immediately after the imports:

```rust
pub(crate) async fn load_run_snapshot_messages(
    pool: &Pool<Sqlite>,
    run_id: i64,
) -> AppResult<Vec<CorpusMessage>>

pub(crate) struct ListRunSnapshotMessagesRequest {
    pub(crate) run_id: i64,
    pub(crate) after: Option<AnalysisRunMessageCursor>,
    pub(crate) limit: usize,
    pub(crate) source_id: Option<i64>,
    pub(crate) around_ref: Option<String>,
}

fn decode_optional_metadata_json(
    metadata_zstd: Option<&[u8]>,
) -> AppResult<Option<serde_json::Value>>

fn run_message_from_snapshot_row(row: StoredRunSnapshotRow) -> AppResult<AnalysisRunMessage>

pub(crate) async fn list_run_snapshot_messages_page(
    pool: &Pool<Sqlite>,
    request: ListRunSnapshotMessagesRequest,
) -> AppResult<AnalysisRunMessagesPage>

#[allow(dead_code)]
pub(crate) async fn load_run_corpus_messages(
    pool: &Pool<Sqlite>,
    run: &AnalysisRunDetail,
) -> AppResult<Vec<CorpusMessage>>

pub(crate) async fn load_trace_resolution_messages(
    pool: &Pool<Sqlite>,
    run: &AnalysisRunDetail,
) -> AppResult<Vec<CorpusMessage>>

fn captured_snapshot_missing_error(run_id: i64) -> AppError

fn ensure_captured_snapshot_rows(
    run: &AnalysisRunDetail,
    snapshot: &[CorpusMessage],
) -> AppResult<()>
```

Move the function and struct bodies verbatim except for the `AnalysisSnapshotState` path in `ensure_captured_snapshot_rows`. In `snapshot.rs`, use the imported type:

```rust
    if run.snapshot_state == Some(AnalysisSnapshotState::Captured)
        && run.snapshot_message_count == 0
        && snapshot.is_empty()
    {
        return Err(captured_snapshot_missing_error(run.id));
    }
```

Do not make `decode_optional_metadata_json`, `run_message_from_snapshot_row`, `captured_snapshot_missing_error`, or `ensure_captured_snapshot_rows` public.

- [x] **Step 3: Add the private module and facade re-exports in `corpus.rs`**

At the top of `src-tauri/src/analysis/corpus.rs`, add `mod snapshot;` and the re-export block. The top of the file should start like this after import cleanup:

```rust
mod snapshot;

use std::collections::HashSet;

pub(crate) use self::snapshot::{
    list_run_snapshot_messages_page, load_run_corpus_messages, load_run_snapshot_messages,
    load_trace_resolution_messages, ListRunSnapshotMessagesRequest,
};
use sqlx::{Pool, QueryBuilder, Sqlite};

use super::models::CorpusMessage;
#[cfg(test)]
use super::models::AnalysisRunDetail;
use super::store::fetch_source_group;
```

Keep this existing test-only scope-constant import block after the model/store imports:

```rust
#[cfg(test)]
use super::{
    ANALYSIS_SCOPE_TYPE_PROJECT, ANALYSIS_SCOPE_TYPE_SINGLE_SOURCE,
    ANALYSIS_SCOPE_TYPE_SOURCE_GROUP,
};
```

If rustfmt does not reorder these imports, leave the grouping readable and minimal.

- [x] **Step 4: Remove moved-only imports from `corpus.rs`**

In `src-tauri/src/analysis/corpus.rs`, remove imports that only snapshot code used:

```rust
AnalysisRunMessage
AnalysisRunMessageCursor
AnalysisRunMessagesPage
StoredRunSnapshotRow
decompress_bytes
```

Keep:

```rust
CorpusMessage
#[cfg(test)] AnalysisRunDetail
compress_json_bytes
decompress_text
internal_error
AppError
AppResult
```

`AnalysisRunDetail` must be gated with `#[cfg(test)]` because after the snapshot move it is only used by the test-only `resolve_run_source_ids` helper in `corpus.rs`.

- [x] **Step 5: Run rustfmt and inspect the touched files**

Run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml
git diff --name-only
```

Expected `git diff --name-only` output for this task:

```text
src-tauri/src/analysis/corpus.rs
src-tauri/src/analysis/corpus/snapshot.rs
```

If unrelated Rust files appear, do not stage them in the refactor commit. Either leave them unstaged for review or make a separate format-only commit before continuing.

- [x] **Step 6: Run focused snapshot tests**

Run each command separately:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::run_snapshot_roundtrips_frozen_corpus
```

Expected: PASS with output containing `1 passed`.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::list_run_snapshot_messages_page
```

Expected: PASS and not a green `0 tests` run. The output should include tests whose names start with `analysis::corpus::tests::list_run_snapshot_messages_page_`.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::load_run_corpus_messages
```

Expected: PASS and not a green `0 tests` run. The output should include tests whose names start with `analysis::corpus::tests::load_run_corpus_messages_`.

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::trace_resolution_does_not_fall_back_to_live_source_for_completed_missing_snapshot
```

Expected: PASS with output containing `1 passed`.

- [x] **Step 7: Run compile coverage for module boundaries**

Run:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
```

Expected: PASS. Existing warnings outside `analysis/corpus.rs` may still appear; new warnings mentioning `src/analysis/corpus.rs` or `src/analysis/corpus/snapshot.rs` are not acceptable.

- [x] **Step 8: Commit the snapshot extraction**

Run:

```powershell
git add -- src-tauri/src/analysis/corpus.rs src-tauri/src/analysis/corpus/snapshot.rs
git commit -m "refactor: extract analysis corpus snapshots"
```

Expected: commit succeeds and includes only `src-tauri/src/analysis/corpus.rs` and `src-tauri/src/analysis/corpus/snapshot.rs`.

---

### Task 3: Final Regression Verification

**Files:**
- Verify: `src-tauri/src/analysis/corpus.rs`
- Verify: `src-tauri/src/analysis/corpus/snapshot.rs`
- Verify: `src-tauri/src/analysis/chat.rs`
- Verify: `src-tauri/src/analysis/mod.rs`

**Interfaces:**
- Confirms the `analysis::corpus` facade still satisfies current callers:
  - `analysis/mod.rs`: `list_run_snapshot_messages_page`, `load_trace_resolution_messages`, `ListRunSnapshotMessagesRequest`
  - `analysis/chat.rs`: `load_run_snapshot_messages`
  - `analysis/corpus.rs` tests: re-exported snapshot helpers and request type

- [ ] **Step 1: Run the full corpus test module**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::corpus::tests::
```

Expected: PASS. This covers snapshot tests plus live corpus, source resolution, YouTube corpus mode, migrated history, and preflight tests that share the same harness.

- [ ] **Step 2: Run chat tests that consume saved run snapshots**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml analysis::chat::tests::
```

Expected: PASS. This covers the `analysis/chat.rs` import path through `super::corpus::load_run_snapshot_messages`.

- [ ] **Step 3: Run format check**

Run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
```

Expected: PASS with no diff output.

- [ ] **Step 4: Run all-target compile coverage**

Run:

```powershell
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
```

Expected: PASS. Existing warnings outside the touched files may remain; no new warnings should mention `src/analysis/corpus.rs` or `src/analysis/corpus/snapshot.rs`.

- [ ] **Step 5: Confirm final git state**

Run:

```powershell
git status --short
git log --oneline -3
```

Expected: `git status --short` is empty. The last commits should include:

```text
refactor: extract analysis corpus snapshots
test: decouple corpus metadata test from snapshot helper
```

If final verification requires a doc-only verification note, commit it separately. Do not amend the refactor commit after verification unless the user explicitly asks.
