# Takeout Import Backend Split Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split the Takeout import backend into focused Rust modules without changing user-visible behavior.

**Architecture:** `src-tauri/src/takeout_import/mod.rs` remains the command and orchestration facade. Focused internal modules own job state, pure pagination logic, and export-DC helpers. The public Tauri command surface and event payloads stay unchanged.

**Tech Stack:** Rust 2021, Tauri 2, Tokio, sqlx SQLite, grammers Telegram client/session/mtsender, serde.

---

## Status

Planned on 2026-05-03. Implementation has not started.

This plan addresses the Takeout side of the code-review finding "Large backend modules mix unrelated behavior". `src-tauri/src/sources.rs` remains a separate follow-up target.

## Planning Decisions

- Priority: split `takeout_import` before `sources`.
- Depth: focused split.
- In first pass, extract `state`, `pagination`, and `export_dc`.
- Keep peer validation and history import orchestration in `mod.rs` for now.
- Keep `raw_parse.rs` unchanged except for import path adjustments if Rust module paths require them.
- Preserve command names, event names, DTO shapes, statuses, phases, warning text, and pagination behavior.
- Do not duplicate Telegram source-kind string constants across modules; keep one canonical set and import it where needed.

## Behavior Freeze Checklist

The refactor is successful only if these behavior-level contracts remain byte-for-byte or shape-for-shape stable:

- Tauri commands: `start_takeout_source_import`, `cancel_takeout_source_import`,
  `list_takeout_source_import_jobs`, `run_takeout_export_dc_spike`.
- Event name: `sources://takeout-import`.
- Response DTOs: `StartTakeoutImportResponse`, `CancelTakeoutImportResponse`,
  `TakeoutImportJobRecord`, `TakeoutExportDcSpikeResult`.
- Status values: `queued`, `running`, `cancel_requested`, `failed`, `cancelled`, `completed`.
- Phase values: `queued`, `resolving_source`, `starting_takeout`, `validating_peer`,
  `loading_splits`, `counting`, `importing_history`, `finishing_takeout`, `completed`,
  `failed`, `cancelled`.
- Pagination constants and warning text, especially `TAKEOUT_HISTORY_PAGE_LIMIT = 100` and the
  TDesktop fallback warning string.
- Export-DC behavior: `export_dc_id = home_dc_id + 4 * 10000`, local-transport fallback only,
  and unchanged `account.initTakeoutSession` flags.
- Cancellation behavior: same-source lock release, cancel state cleanup, and terminal job updates.

## File Structure

- Rename: `src-tauri/src/takeout_import.rs` -> `src-tauri/src/takeout_import/mod.rs`
- Existing: `src-tauri/src/takeout_import/raw_parse.rs`
- Create: `src-tauri/src/takeout_import/state.rs`
- Create: `src-tauri/src/takeout_import/pagination.rs`
- Create: `src-tauri/src/takeout_import/export_dc.rs`
- Modify: `src-tauri/src/lib.rs` only if the module rename requires import path cleanup.
- Modify after implementation: `docs/code-review-results-2026-05-03.md`

## Public Surface To Preserve

`src-tauri/src/lib.rs` must still import the same items from `takeout_import`:

```rust
use takeout_import::{
    cancel_takeout_source_import, list_takeout_source_import_jobs, run_takeout_export_dc_spike,
    start_takeout_source_import, TakeoutImportState,
};
```

The `takeout_import` module must still expose:

- `start_takeout_source_import`
- `cancel_takeout_source_import`
- `list_takeout_source_import_jobs`
- `run_takeout_export_dc_spike`
- `TakeoutImportState`

No frontend TypeScript or Svelte files should be changed for this refactor.

## Module Ownership

`state.rs` owns:

- `StartTakeoutImportResponse`
- `CancelTakeoutImportResponse`
- `TakeoutImportJobRecord`
- `TakeoutImportState`
- job state maps and cancel tracking
- status constants
- phase constants
- `emit_takeout_import_event`
- `update_and_emit`
- terminal status logic
- state-focused tests

`pagination.rs` owns:

- `TAKEOUT_HISTORY_PAGE_LIMIT`
- `TakeoutPaginationProfile`
- `TakeoutPageRequest`
- `TakeoutPaginationCursor`
- `TakeoutCursorAdvance`
- `ParsedTakeoutPage`
- `TakeoutPaginationFallbackReason`
- `select_history_splits`
- `fallback_message_range`
- `takeout_page_request`
- `next_takeout_cursor`
- `should_restart_with_descending_fallback`
- `takeout_pagination_fallback_warning`
- `message_range_min_id`
- `message_range_max_id`
- `parse_takeout_page`
- pagination-focused tests

`export_dc.rs` owns:

- `ExportDcAlias`
- `EXPORT_DC_SHIFT`
- `TAKEOUT_FILE_MAX_SIZE`
- `prepare_export_dc_alias`
- `export_dc_id_for_home_dc`
- `takeout_init_request_for_source_kind`
- `export_dc_invoke`
- `should_fallback_export_dc_error`
- `finish_takeout_session`
- export-DC-focused tests

`mod.rs` keeps:

- canonical `TELEGRAM_KIND_CHANNEL`, `TELEGRAM_KIND_SUPERGROUP`, and `TELEGRAM_KIND_GROUP`
- Tauri commands
- `TakeoutExportDcSpikeResult`
- `TakeoutImportOutcome`
- `TakeoutHistoryImport`
- `CountedMessageRange`
- `TakeoutHistoryProbe`
- source loading and account runtime flow
- peer validation and supergroup migration checks
- history count probes
- history page requests
- page import loop
- `messages_response_count`
- `push_warning_once`
- `supports_only_my_messages_fallback`
- `is_channel_private_error`
- calls into `raw_parse::parse_raw_message`

## Internal Visibility Rules

- Use `pub` only for types/functions exposed through the existing Tauri module facade or serialized
  command return types.
- Use `pub(crate)` for helpers consumed across `takeout_import` submodules.
- Keep helpers private when they are used only by tests inside the same module.
- Prefer `use super::{TELEGRAM_KIND_CHANNEL, TELEGRAM_KIND_GROUP, TELEGRAM_KIND_SUPERGROUP};` in
  submodules over redefining those string constants.
- Do not introduce new crates, traits, service abstractions, or generated code.

## Derive And Serialization Rules

- Keep `#[derive(Clone, Debug, Serialize, PartialEq, Eq)]` on command/event DTOs that already have it.
- Keep `#[derive(Default)]` on `TakeoutImportStateInner`.
- Keep pagination derives unchanged:
  - cursor/profile/request/advance/fallback reason types keep `Clone`, `Copy`, `Debug`,
    `PartialEq`, and `Eq` where they currently exist;
  - `ParsedTakeoutPage` keeps `Clone`, `Debug`, and `PartialEq`.
- Keep `ExportDcAlias` as `#[derive(Clone, Debug, PartialEq, Eq)]`.
- Do not add or remove serde field attributes, because the command/event payload shape is part of
  the frontend IPC contract.

## Task 1: Create Module Shell And Move Job State

**Files:**
- Rename: `src-tauri/src/takeout_import.rs` -> `src-tauri/src/takeout_import/mod.rs`
- Create: `src-tauri/src/takeout_import/state.rs`
- Modify: `src-tauri/src/takeout_import/mod.rs`

- [ ] **Step 1: Establish baseline**

Run:

```powershell
Set-Location src-tauri
cargo test takeout_import
```

Expected: existing Takeout import tests pass before refactoring. If dependency or sandbox issues block the command, record the exact failure and rerun with the required escalation.

- [ ] **Step 2: Rename the module file**

Move the existing file into the module directory:

```text
src-tauri/src/takeout_import.rs
src-tauri/src/takeout_import/mod.rs
```

Keep the existing `mod raw_parse;` declaration in the new `mod.rs`.

- [ ] **Step 3: Add state module declaration and re-export**

At the top of `mod.rs`, declare:

```rust
mod raw_parse;
mod state;

pub use state::TakeoutImportState;
use state::{
    emit_takeout_import_event, update_and_emit, CancelTakeoutImportResponse,
    StartTakeoutImportResponse, TakeoutImportJobRecord, PHASE_CANCELLED, PHASE_COMPLETED,
    PHASE_COUNTING, PHASE_FAILED, PHASE_FINISHING_TAKEOUT, PHASE_IMPORTING_HISTORY,
    PHASE_LOADING_SPLITS, PHASE_RESOLVING_SOURCE, PHASE_STARTING_TAKEOUT, PHASE_VALIDATING_PEER,
    STATUS_CANCELLED, STATUS_COMPLETED, STATUS_FAILED, STATUS_RUNNING,
};
```

- [ ] **Step 4: Move state-owned code**

Move these items from `mod.rs` into `state.rs`:

```rust
const TAKEOUT_IMPORT_EVENT: &str = "sources://takeout-import";
const STATUS_QUEUED: &str = "queued";
const STATUS_RUNNING: &str = "running";
const STATUS_CANCEL_REQUESTED: &str = "cancel_requested";
const STATUS_FAILED: &str = "failed";
const STATUS_CANCELLED: &str = "cancelled";
const STATUS_COMPLETED: &str = "completed";
const PHASE_QUEUED: &str = "queued";
const PHASE_RESOLVING_SOURCE: &str = "resolving_source";
const PHASE_STARTING_TAKEOUT: &str = "starting_takeout";
const PHASE_VALIDATING_PEER: &str = "validating_peer";
const PHASE_LOADING_SPLITS: &str = "loading_splits";
const PHASE_COUNTING: &str = "counting";
const PHASE_IMPORTING_HISTORY: &str = "importing_history";
const PHASE_FINISHING_TAKEOUT: &str = "finishing_takeout";
const PHASE_COMPLETED: &str = "completed";
const PHASE_FAILED: &str = "failed";
const PHASE_CANCELLED: &str = "cancelled";
```

Also move the response structs, `TakeoutImportJobRecord`, `TakeoutImportStateInner`, `TakeoutImportState`, `emit_takeout_import_event`, `update_and_emit`, `is_terminal_status`, and `now_secs`.

The state module should import exactly the local dependencies it owns:

```rust
use std::collections::{HashMap, HashSet};

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

use crate::error::{AppError, AppResult};
```

Expose only the items consumed by `mod.rs` as `pub(crate)` or `pub`:

```rust
pub struct StartTakeoutImportResponse {
    pub job_id: String,
}

pub struct CancelTakeoutImportResponse {
    pub cancelled: bool,
}

pub struct TakeoutImportJobRecord {
    pub job_id: String,
    pub source_id: i64,
    pub account_id: i64,
    pub status: String,
    pub phase: String,
    pub message: Option<String>,
    pub inserted: i64,
    pub skipped: i64,
    pub progress_current: Option<i64>,
    pub progress_total: Option<i64>,
    pub started_at: i64,
    pub finished_at: Option<i64>,
    pub warnings: Vec<String>,
    pub error: Option<String>,
}

pub struct TakeoutImportState {
    inner: Mutex<TakeoutImportStateInner>,
}

pub(crate) const STATUS_RUNNING: &str = "running";
pub(crate) const STATUS_CANCEL_REQUESTED: &str = "cancel_requested";
pub(crate) const STATUS_FAILED: &str = "failed";
pub(crate) const STATUS_CANCELLED: &str = "cancelled";
pub(crate) const STATUS_COMPLETED: &str = "completed";
pub(crate) const PHASE_RESOLVING_SOURCE: &str = "resolving_source";
pub(crate) const PHASE_STARTING_TAKEOUT: &str = "starting_takeout";
pub(crate) const PHASE_VALIDATING_PEER: &str = "validating_peer";
pub(crate) const PHASE_LOADING_SPLITS: &str = "loading_splits";
pub(crate) const PHASE_COUNTING: &str = "counting";
pub(crate) const PHASE_IMPORTING_HISTORY: &str = "importing_history";
pub(crate) const PHASE_FINISHING_TAKEOUT: &str = "finishing_takeout";
pub(crate) const PHASE_COMPLETED: &str = "completed";
pub(crate) const PHASE_FAILED: &str = "failed";
pub(crate) const PHASE_CANCELLED: &str = "cancelled";
pub(crate) fn emit_takeout_import_event(handle: &AppHandle, record: &TakeoutImportJobRecord);
pub(crate) async fn update_and_emit<F>(
    handle: &AppHandle,
    state: &TakeoutImportState,
    job_id: &str,
    update: F,
)
where
    F: FnOnce(&mut TakeoutImportJobRecord);
```

Keep these `TakeoutImportState` methods `pub(crate)` because `mod.rs` calls them:

```rust
pub(crate) async fn create_job(
    &self,
    source_id: i64,
    account_id: i64,
) -> AppResult<TakeoutImportJobRecord>;

pub(crate) async fn list_jobs(&self) -> Vec<TakeoutImportJobRecord>;

pub(crate) async fn request_cancel(&self, job_id: &str) -> Option<TakeoutImportJobRecord>;

pub(crate) async fn is_cancel_requested(&self, job_id: &str) -> bool;

pub(crate) async fn update_job<F>(
    &self,
    job_id: &str,
    update: F,
) -> Option<TakeoutImportJobRecord>
where
    F: FnOnce(&mut TakeoutImportJobRecord);

pub(crate) async fn finish_job<F>(
    &self,
    job_id: &str,
    update: F,
) -> Option<TakeoutImportJobRecord>
where
    F: FnOnce(&mut TakeoutImportJobRecord);
```

Keep `TakeoutImportState::new` public because `lib.rs` manages it through Tauri state.

- [ ] **Step 5: Move state tests**

Move the two existing async state tests into `state.rs`:

- `job_state_rejects_duplicate_active_source_jobs`
- `job_state_can_cancel_and_finish_job`

The assertions must stay the same:

- duplicate active source returns `AppErrorKind::Conflict`;
- cancellation sets `STATUS_CANCEL_REQUESTED`;
- finishing clears cancel state and releases the source for a new job.

- [ ] **Step 6: Verify Task 1**

Run:

```powershell
Set-Location src-tauri
cargo test takeout_import::state
cargo test takeout_import
```

Expected: state tests and full Takeout import tests pass.

## Task 2: Move Pure Pagination Logic

**Files:**
- Create: `src-tauri/src/takeout_import/pagination.rs`
- Modify: `src-tauri/src/takeout_import/mod.rs`

- [ ] **Step 1: Add pagination module declaration**

At the top of `mod.rs`, add:

```rust
mod pagination;
```

Import the pagination items used by orchestration:

```rust
use pagination::{
    message_range_max_id, message_range_min_id, next_takeout_cursor, parse_takeout_page,
    select_history_splits, should_restart_with_descending_fallback, takeout_page_request,
    takeout_pagination_fallback_warning, TakeoutPaginationCursor, TakeoutPaginationProfile,
};
```

- [ ] **Step 2: Move pagination types and helpers**

Move these items into `pagination.rs`:

The pagination module should import:

```rust
use grammers_client::tl;

use super::{TELEGRAM_KIND_CHANNEL, TELEGRAM_KIND_GROUP, TELEGRAM_KIND_SUPERGROUP};
use crate::error::{AppError, AppResult};
```

```rust
const TAKEOUT_HISTORY_PAGE_LIMIT: i32 = 100;

pub(crate) enum TakeoutPaginationProfile {
    TDesktop,
    DescendingFallback,
}

pub(crate) struct TakeoutPageRequest {
    pub(crate) offset_id: i32,
    pub(crate) add_offset: i32,
    pub(crate) limit: i32,
}

pub(crate) enum TakeoutPaginationCursor {
    TDesktop { largest_id_plus_one: i32 },
    DescendingFallback { offset_id: i32 },
}

pub(crate) struct TakeoutCursorAdvance {
    pub(crate) cursor: TakeoutPaginationCursor,
    pub(crate) advanced: bool,
    pub(crate) reached_range_start: bool,
}

pub(crate) struct ParsedTakeoutPage {
    pub(crate) messages: Vec<tl::types::Message>,
    pub(crate) first_regular_message_id: Option<i32>,
    pub(crate) last_regular_message_id: Option<i32>,
    pub(crate) oldest_regular_message_id: Option<i32>,
    pub(crate) newest_regular_message_id: Option<i32>,
    pub(crate) is_terminal_response: bool,
}

pub(crate) enum TakeoutPaginationFallbackReason {
    EmptyFirstPageWithNonZeroCount,
    NonAdvancingTDesktopCursor,
}

pub(crate) fn select_history_splits(
    telegram_source_kind: &str,
    ranges: Vec<tl::enums::MessageRange>,
) -> AppResult<Vec<tl::enums::MessageRange>>;

fn fallback_message_range() -> tl::enums::MessageRange;

impl TakeoutPaginationCursor {
    pub(crate) fn new(profile: TakeoutPaginationProfile, range: &tl::enums::MessageRange) -> Self;
}

pub(crate) fn takeout_page_request(cursor: TakeoutPaginationCursor) -> TakeoutPageRequest;

pub(crate) fn next_takeout_cursor(
    cursor: TakeoutPaginationCursor,
    page: &ParsedTakeoutPage,
    range: &tl::enums::MessageRange,
) -> TakeoutCursorAdvance;

pub(crate) fn should_restart_with_descending_fallback(
    profile: TakeoutPaginationProfile,
    split_count: i64,
    page_index: usize,
    page: &ParsedTakeoutPage,
    advance: TakeoutCursorAdvance,
) -> Option<TakeoutPaginationFallbackReason>;

pub(crate) fn takeout_pagination_fallback_warning(
    reason: TakeoutPaginationFallbackReason,
    range: &tl::enums::MessageRange,
) -> String;

pub(crate) fn parse_takeout_page(
    response: tl::enums::messages::Messages,
    profile: TakeoutPaginationProfile,
) -> AppResult<ParsedTakeoutPage>;

pub(crate) fn message_range_min_id(range: &tl::enums::MessageRange) -> i32;

pub(crate) fn message_range_max_id(range: &tl::enums::MessageRange) -> i32;
```

Make the types and functions used by `mod.rs` `pub(crate)`. Keep helper fields `pub(crate)` only when `mod.rs` reads them directly, for example `ParsedTakeoutPage.messages` and `TakeoutCursorAdvance.advanced`.

Use the parent source-kind constants instead of redefining them:

```rust
use super::{TELEGRAM_KIND_CHANNEL, TELEGRAM_KIND_GROUP, TELEGRAM_KIND_SUPERGROUP};
```

Keep `fallback_message_range` private unless the tests are moved outside `pagination.rs`.

- [ ] **Step 3: Keep behavior comments with the pagination code**

Move the existing TDesktop pagination comment with `TakeoutPaginationCursor`. It documents why the TDesktop-first and descending fallback profiles both exist and should stay next to the cursor implementation.

- [ ] **Step 4: Move pagination tests**

Move tests that assert split selection, fallback selection, page parsing, cursor advancement, and message range helpers into `pagination.rs`.

The moved test names include:

```text
split_selection_uses_last_range_for_channel_and_supergroup
split_selection_uses_all_ranges_for_small_group
split_selection_falls_back_when_telegram_returns_no_ranges
tdesktop_pagination_reverses_raw_order_and_advances_from_newest_id
descending_fallback_keeps_raw_order_and_moves_to_min_message_id
tdesktop_empty_first_page_with_nonzero_count_restarts_descending_fallback
tdesktop_non_advancing_cursor_restarts_descending_fallback
messages_response_without_slice_is_terminal_page
messages_not_modified_response_is_rejected_for_takeout_page
```

Keep the local test helper functions `message_range`, `message_ids`, `messages_slice_response`, `messages_messages_response`, and `raw_message` with the pagination tests.

- [ ] **Step 5: Verify Task 2**

Run:

```powershell
Set-Location src-tauri
cargo test takeout_import::pagination
cargo test takeout_import
```

Expected: pagination tests and full Takeout import tests pass. The warning string for TDesktop fallback must remain unchanged.

## Task 3: Move Export-DC Helpers

**Files:**
- Create: `src-tauri/src/takeout_import/export_dc.rs`
- Modify: `src-tauri/src/takeout_import/mod.rs`

- [ ] **Step 1: Add export module declaration**

At the top of `mod.rs`, add:

```rust
mod export_dc;
```

Import the export-DC items used by orchestration:

```rust
use export_dc::{
    export_dc_invoke, prepare_export_dc_alias, takeout_init_request_for_source_kind,
    finish_takeout_session, ExportDcAlias,
};
```

- [ ] **Step 2: Move export-DC types and helpers**

Move these items into `export_dc.rs`:

The export-DC module should import:

```rust
use std::sync::Arc;

use grammers_client::{tl, Client};
use grammers_mtsender::InvocationError;
use grammers_session::{storages::MemorySession, Session};

use super::{TELEGRAM_KIND_CHANNEL, TELEGRAM_KIND_GROUP, TELEGRAM_KIND_SUPERGROUP};
use crate::error::{AppError, AppResult};
```

```rust
const EXPORT_DC_SHIFT: i32 = 4 * 10_000;
const TAKEOUT_FILE_MAX_SIZE: i64 = 8 * 1024 * 1024;

pub(crate) struct ExportDcAlias {
    pub(crate) home_dc_id: i32,
    pub(crate) export_dc_id: i32,
}

pub(crate) async fn prepare_export_dc_alias(
    session: &Arc<MemorySession>,
) -> AppResult<ExportDcAlias>;

fn export_dc_id_for_home_dc(home_dc_id: i32) -> i32;

pub(crate) fn takeout_init_request_for_source_kind(
    telegram_source_kind: &str,
) -> AppResult<tl::functions::account::InitTakeoutSession>;

pub(crate) async fn export_dc_invoke<R: tl::RemoteCall>(
    client: &Client,
    alias: &ExportDcAlias,
    request: &R,
    warnings: &mut Vec<String>,
    fallback_used: &mut bool,
) -> AppResult<R::Return>;

fn should_fallback_export_dc_error(error: &InvocationError) -> bool;

pub(crate) async fn finish_takeout_session(
    client: &Client,
    alias: &ExportDcAlias,
    takeout_id: i64,
    success: bool,
    warnings: &mut Vec<String>,
    fallback_used: &mut bool,
) -> AppResult<()>;
```

Make `ExportDcAlias` and helpers used by `mod.rs` `pub(crate)`. Keep `export_dc_id_for_home_dc` and `should_fallback_export_dc_error` private unless tests need access from inside `export_dc.rs`.

Use the parent source-kind constants instead of redefining them:

```rust
use super::{TELEGRAM_KIND_CHANNEL, TELEGRAM_KIND_GROUP, TELEGRAM_KIND_SUPERGROUP};
```

Keep `TAKEOUT_FILE_MAX_SIZE` private inside `export_dc.rs`; tests in the same module can still assert against it.

- [ ] **Step 3: Move export-DC tests**

Move tests that assert export DC ID calculation, Takeout init flags, and local transport fallback classification into `export_dc.rs`:

```text
export_dc_id_applies_tdesktop_shift
takeout_init_request_uses_source_kind_flags_and_file_limit
export_dc_fallback_is_only_for_local_transport_errors
```

- [ ] **Step 4: Verify Task 3**

Run:

```powershell
Set-Location src-tauri
cargo test takeout_import::export_dc
cargo test takeout_import
```

Expected: export-DC tests and full Takeout import tests pass.

## Task 4: Clean The Facade Without Changing Behavior

**Files:**
- Modify: `src-tauri/src/takeout_import/mod.rs`
- Modify if needed: `src-tauri/src/lib.rs`

- [ ] **Step 1: Remove now-unused imports from `mod.rs`**

After moving code, clean imports that are no longer needed by orchestration:

- `std::collections::{HashMap, HashSet}` should be used only in `state.rs`.
- `tokio::sync::Mutex` should be used only in `state.rs`.
- `grammers_mtsender::InvocationError` should be used only in `export_dc.rs`.
- `grammers_session::storages::MemorySession` and `grammers_session::Session` should be used only in `export_dc.rs`.

- [ ] **Step 2: Confirm no public API drift**

Run:

```powershell
rg -n "start_takeout_source_import|cancel_takeout_source_import|list_takeout_source_import_jobs|run_takeout_export_dc_spike|TakeoutImportState" src-tauri\src\lib.rs src-tauri\src\takeout_import
```

Expected: `lib.rs` still imports the same five public items from `takeout_import`, and the public command function names are unchanged.

- [ ] **Step 3: Confirm constants were not duplicated**

Run:

```powershell
rg -n "const TELEGRAM_KIND_|sources://takeout-import|TAKEOUT_HISTORY_PAGE_LIMIT|TAKEOUT_FILE_MAX_SIZE" src-tauri\src\takeout_import
```

Expected:

- `TELEGRAM_KIND_CHANNEL`, `TELEGRAM_KIND_SUPERGROUP`, and `TELEGRAM_KIND_GROUP` appear only in
  `mod.rs`.
- `sources://takeout-import` appears only in `state.rs`.
- `TAKEOUT_HISTORY_PAGE_LIMIT` appears only in `pagination.rs`.
- `TAKEOUT_FILE_MAX_SIZE` appears only in `export_dc.rs`.

- [ ] **Step 4: Verify full Rust tests**

Run:

```powershell
Set-Location src-tauri
cargo test
```

Expected: all Rust tests pass.

## Task 5: Update Review Documentation

**Files:**
- Modify: `docs/code-review-results-2026-05-03.md`
- Modify if needed: `docs/takeout-source-import.md`

- [ ] **Step 1: Update code review finding after implementation**

In `docs/code-review-results-2026-05-03.md`, update the "Large backend modules mix unrelated behavior" finding to say:

- Takeout import has been split into `state`, `pagination`, and `export_dc` modules.
- `sources.rs` remains the next backend split target.
- Any remaining Takeout orchestration in `mod.rs` is intentional for this first slice.

- [ ] **Step 2: Update Takeout architecture notes only if behavior docs become stale**

If the refactor changes only file locations, do not rewrite the user-facing Takeout behavior in `docs/takeout-source-import.md`. Add a short implementation note only if the existing document names the old single-file implementation in a way that becomes misleading.

- [ ] **Step 3: Verify docs whitespace**

Run:

```powershell
git diff --check -- docs/code-review-results-2026-05-03.md docs/takeout-source-import.md
```

Expected: no whitespace errors. LF-to-CRLF warnings may appear in this Windows worktree and should be recorded separately if they are the only output.

## Final Verification

Run after all implementation tasks:

```powershell
Set-Location src-tauri
cargo test
Set-Location ..
npm.cmd test
npm.cmd run check
git diff --check
```

Expected:

- Rust tests pass.
- Frontend tests pass.
- Svelte check reports 0 errors and 0 warnings.
- Whitespace check reports no real whitespace errors.

Known environment note: `npm.cmd test` and `npm.cmd run check` may fail inside the sandbox with Vite/esbuild `spawn EPERM`. If that happens, rerun with escalation and record the sandbox failure plus the escalated result.

## Suggested Commit Message

```text
refactor(takeout): split import state pagination and export dc
```
