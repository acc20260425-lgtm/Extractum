# Sources Backend Split Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split the large `src-tauri/src/sources.rs` backend module into focused modules without changing user-visible behavior, Tauri command names, DTO shapes, database behavior, or Takeout import integration.

**Architecture:** Convert `sources.rs` into a `sources/` directory module. Keep `sources/mod.rs` as the facade that declares modules and re-exports the same public command and crate-level APIs. Move code by existing behavior boundaries: settings, avatars, peer resolution, store/query mapping, items, forum topics, and sync orchestration.

**Tech Stack:** Rust 2021, Tauri 2 commands, sqlx SQLite, grammers Telegram client/session types, serde, zstd compression, existing in-module Rust unit tests.

---

## Current Baseline

- Branch at planning time: `main`.
- Latest relevant implementation commit: `013ecc0 refactor(takeout): split import state pagination and export dc`.
- Current target file: `src-tauri/src/sources.rs`.
- Current target size: 3243 lines.
- Baseline command already run from `src-tauri/`:

```powershell
cargo test sources::tests --lib
```

Result:

```text
29 passed; 0 failed; 101 filtered out
```

## Non-Negotiable Contracts

Do not rename these Tauri commands:

```rust
get_sync_settings
save_sync_settings
delete_source
list_telegram_sources
add_telegram_source
list_sources
sync_source
get_items
list_source_forum_topics
```

Do not change these serialized DTO shapes:

```rust
InitialSyncMode
SyncSettingsRecord
TelegramSourceInfo
SourceRecord
SyncResult
ItemRecord
ForumTopicFilter
SourceForumTopicRecord
```

Do not break these crate-level APIs used by `src-tauri/src/takeout_import/mod.rs` and `src-tauri/src/takeout_import/raw_parse.rs`:

```rust
load_source
resolve_and_refresh_peer
finalize_sync
insert_source_item
SourceSyncTarget
ResolvedSyncPeer
SourceItemInsert
TelegramItemContext
```

`src-tauri/src/lib.rs` should continue importing from `sources` with the same command names:

```rust
use sources::{
    add_telegram_source, delete_source, get_items, get_sync_settings, list_source_forum_topics,
    list_sources, list_telegram_sources, save_sync_settings, sync_source,
};
```

## Target File Structure

Create this tree:

```text
src-tauri/src/sources/
  mod.rs
  avatar.rs
  items.rs
  peer_resolution.rs
  settings.rs
  store.rs
  sync.rs
  topics.rs
  types.rs
```

Remove the old file module after the directory module exists:

```text
src-tauri/src/sources.rs
```

### Module Ownership

`mod.rs` owns only module declarations and facade re-exports.

`types.rs` owns shared constants, public DTOs, shared DB row structs, and cross-module structs.

`settings.rs` owns sync setting mode parsing, validation, labels, persistence, and settings Tauri commands.

`avatar.rs` owns Telegram photo download, data URL encoding, avatar cache keys, cache writes, and cache reads.

`peer_resolution.rs` owns source metadata encode/decode, manual source ref parsing, Telegram source resolution, stored peer ref reconstruction, source kind validation, and `resolve_and_refresh_peer`.

`store.rs` owns source CRUD/listing command flows, dialog source listing, `load_source`, row-to-DTO mapping, and add-source persistence.

`items.rs` owns item insert/list behavior, item row mapping, live Telegram message to insert payload helpers, and shared item ingest structs.

`topics.rs` owns forum topic refresh, forum topic API listing, topic upserts, pagination cursors, and non-forum refresh error classification.

`sync.rs` owns `sync_source`, sync policy calculation, live message ingestion loop, and `finalize_sync`.

## Facade Design

`src-tauri/src/sources/mod.rs` should look like this after extraction:

```rust
mod avatar;
mod items;
mod peer_resolution;
mod settings;
mod store;
mod sync;
mod topics;
mod types;

pub use items::{get_items, ForumTopicFilter, ItemRecord};
pub use settings::{get_sync_settings, save_sync_settings, InitialSyncMode, SyncSettingsRecord};
pub use store::{add_telegram_source, delete_source, list_sources, list_telegram_sources};
pub use sync::{sync_source, SyncResult};
pub use topics::{list_source_forum_topics, SourceForumTopicRecord};
pub use types::{SourceRecord, TelegramSourceInfo};

pub(crate) use items::{insert_source_item, SourceItemInsert, TelegramItemContext};
pub(crate) use peer_resolution::{resolve_and_refresh_peer, ResolvedSyncPeer};
pub(crate) use store::load_source;
pub(crate) use sync::finalize_sync;
pub(crate) use types::SourceSyncTarget;
```

If a DTO naturally lives in `types.rs` instead of its behavior module, adjust only the re-export source path. The exported names and visibility must remain the same.

## Symbol Move Map

### `types.rs`

Move these constants:

```rust
TELEGRAM_SOURCE_TYPE
TELEGRAM_KIND_CHANNEL
TELEGRAM_KIND_SUPERGROUP
TELEGRAM_KIND_GROUP
```

Move these public DTOs and shared structs:

```rust
TelegramSourceInfo
SourceRecord
SourceSyncTarget
SourceRecordRow
StoredItemRow
SourceForumTopicRow
```

Keep `SourceSyncTarget` as `pub(crate)` with all current `pub(crate)` fields because `takeout_import` reads fields directly.

`StoredItemRow` and `SourceForumTopicRow` can remain `pub(super)` or `pub(crate)` depending on module test access. Prefer `pub(super)` unless cross-module use requires wider visibility.

### `settings.rs`

Move these constants and symbols:

```rust
DEFAULT_INITIAL_SYNC_MESSAGE_LIMIT
MIN_INITIAL_SYNC_MESSAGE_LIMIT
MAX_INITIAL_SYNC_MESSAGE_LIMIT
DEFAULT_INITIAL_SYNC_DAY_LIMIT
MIN_INITIAL_SYNC_DAY_LIMIT
MAX_INITIAL_SYNC_DAY_LIMIT
INITIAL_SYNC_MODE_SETTING_KEY
INITIAL_SYNC_VALUE_SETTING_KEY
SECONDS_PER_DAY
InitialSyncMode
SyncSettingsRecord
default_sync_settings
validate_sync_settings
initial_sync_policy_label
read_setting
write_setting
load_sync_settings_from_pool
save_sync_settings_to_pool
get_sync_settings
save_sync_settings
```

Keep these crate-visible for `sync.rs` tests and sync policy calculation:

```rust
pub(super) fn initial_sync_policy_label(settings: &SyncSettingsRecord) -> String
pub(super) async fn load_sync_settings_from_pool(pool: &sqlx::Pool<sqlx::Sqlite>) -> AppResult<SyncSettingsRecord>
pub(super) const SECONDS_PER_DAY: i64
```

Keep `default_sync_settings`, `validate_sync_settings`, and `save_sync_settings_to_pool` private unless module tests require `use super::*`.

### `avatar.rs`

Move these constants and symbols:

```rust
TELEGRAM_SOURCE_PHOTO_TIMEOUT_MS
TELEGRAM_SOURCE_PHOTO_LIST_BUDGET_MS
TELEGRAM_SOURCE_AVATAR_CACHE_DIR
peer_photo_data_url_with_timeout
peer_photo_bytes_with_timeout
peer_photo_bytes
photo_bytes_data_url
source_avatar_cache_key
source_avatar_cache_dir
cache_source_avatar
read_source_avatar_data_url
```

Use `pub(super)` for helpers consumed by `store.rs` and `peer_resolution.rs`:

```rust
pub(super) async fn peer_photo_data_url_with_timeout(...)
pub(super) async fn peer_photo_bytes_with_timeout(...)
pub(super) fn cache_source_avatar(...)
pub(super) fn read_source_avatar_data_url(...)
```

### `peer_resolution.rs`

Move these symbols:

```rust
SourcePeerResolutionStrategy
SourcePeerIdentity
SourceMetadata
ResolvedTelegramSource
ManualTelegramSourceRef
SourcePeerResolutionStep
parse_username
unsupported_manual_source_ref_message
unsupported_private_manual_source_ref_message
parse_supported_manual_telegram_source_ref
legacy_peer_identity
add_source_resolution_strategy
source_metadata_for_added_source
source_peer_resolution_plan
source_peer_resolution_failure
resolve_telegram_source_by_username
dialog_lookup_not_found_message
resolve_telegram_source_from_dialogs
resolve_telegram_source
telegram_source_kind_matches
validate_expected_telegram_source_kind
ensure_supported_telegram_source_kind
resolved_telegram_source_from_peer
peer_access_hash
encode_source_metadata
decode_source_metadata
resolve_source_peer
source_peer_ref_from_identity
peer_ref_for_source_kind
resolve_and_refresh_peer
refresh_source_avatar_cache
```

Use `pub(super)` for helpers needed by `store.rs`:

```rust
pub(super) async fn resolve_telegram_source(...)
pub(super) fn source_metadata_for_added_source(...)
pub(super) fn encode_source_metadata(...)
pub(super) fn decode_source_metadata(...)
pub(super) fn telegram_source_info_from_peer(...)
```

`telegram_source_info_from_peer`, `telegram_group_kind`, and `telegram_group_is_member` can live in `peer_resolution.rs` because source kind derivation is part of peer interpretation. If this makes `store.rs` simpler, move these three from the old avatar/listing area into `peer_resolution.rs`:

```rust
telegram_source_info_from_peer
telegram_group_kind
telegram_group_is_member
```

Keep `ResolvedSyncPeer` as a facade-exported crate API. It can live in `types.rs` or `peer_resolution.rs`; if it lives in `peer_resolution.rs`, re-export it with `pub(crate) use peer_resolution::ResolvedSyncPeer`.

### `store.rs`

Move these symbols:

```rust
delete_source
list_telegram_sources
load_source
add_telegram_source
list_sources
source_record_from_row
```

Dependencies:

```rust
use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::source_ingest::{SourceIngestKind, SourceIngestLocks};
use crate::telegram::TelegramState;
use super::avatar::{cache_source_avatar, peer_photo_data_url_with_timeout, read_source_avatar_data_url};
use super::peer_resolution::{
    decode_source_metadata, encode_source_metadata, resolve_telegram_source,
    source_metadata_for_added_source, telegram_source_info_from_peer,
};
use super::types::{SourceRecord, SourceRecordRow, SourceSyncTarget, TelegramSourceInfo};
```

### `items.rs`

Move these symbols:

```rust
ItemRecord
ForumTopicFilter
SourceItemInsert
TelegramItemContext
insert_source_item
get_items
load_item_rows_from_pool
message_author
extract_telegram_context
reply_peer_context
build_raw_payload
```

Also move or expose `now_secs` here only if it remains private to item inserts. Prefer a small `time.rs` only if multiple modules become awkward; otherwise duplicate no logic and place `now_secs` in `types.rs` as `pub(super) fn now_secs() -> i64`.

Recommended shared helper placement:

```rust
// types.rs
pub(super) fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
```

### `topics.rs`

Move these symbols:

```rust
SourceForumTopicRecord
ForumTopicSnapshot
refresh_forum_topics
fetch_all_forum_topics
forum_topic_page_cursor
forum_topic_message_date
upsert_forum_topics_from_refresh
is_non_forum_topic_refresh_error
list_source_forum_topics
list_source_forum_topics_from_pool
```

Use `pub(super)` for `refresh_forum_topics` so `sync.rs` can call it.

Keep `SourceForumTopicRecord` public through the facade.

### `sync.rs`

Move these symbols:

```rust
SyncResult
SyncPolicy
IngestOutcome
determine_sync_policy
persist_items
sync_source
finalize_sync
```

Dependencies:

```rust
use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::media::extract_item_payload;
use crate::source_ingest::{SourceIngestKind, SourceIngestLocks};
use crate::telegram::TelegramState;
use super::items::{
    build_raw_payload, extract_telegram_context, insert_source_item, message_author,
    SourceItemInsert,
};
use super::peer_resolution::resolve_and_refresh_peer;
use super::settings::{initial_sync_policy_label, load_sync_settings_from_pool, InitialSyncMode, SyncSettingsRecord, SECONDS_PER_DAY};
use super::topics::refresh_forum_topics;
use super::types::{now_secs, SourceSyncTarget};
```

If private item helpers cause visibility friction, use `pub(super)` and keep them available only inside `sources`.

## Task 1: Create Directory Module Skeleton

**Files:**
- Create: `src-tauri/src/sources/mod.rs`
- Create: `src-tauri/src/sources/types.rs`
- Modify: `src-tauri/src/sources.rs`

- [ ] **Step 1: Create facade skeleton**

Create `src-tauri/src/sources/mod.rs` with:

```rust
mod avatar;
mod items;
mod peer_resolution;
mod settings;
mod store;
mod sync;
mod topics;
mod types;

pub use items::{get_items, ForumTopicFilter, ItemRecord};
pub use settings::{get_sync_settings, save_sync_settings, InitialSyncMode, SyncSettingsRecord};
pub use store::{add_telegram_source, delete_source, list_sources, list_telegram_sources};
pub use sync::{sync_source, SyncResult};
pub use topics::{list_source_forum_topics, SourceForumTopicRecord};
pub use types::{SourceRecord, TelegramSourceInfo};

pub(crate) use items::{insert_source_item, SourceItemInsert, TelegramItemContext};
pub(crate) use peer_resolution::{resolve_and_refresh_peer, ResolvedSyncPeer};
pub(crate) use store::load_source;
pub(crate) use sync::finalize_sync;
pub(crate) use types::SourceSyncTarget;
```

- [ ] **Step 2: Move shared type declarations**

Create `src-tauri/src/sources/types.rs` by moving the shared constants and structs listed in `types.rs` above. Keep derives unchanged.

- [ ] **Step 3: Temporarily disable old file module conflict**

Move `src-tauri/src/sources.rs` content into the new modules as tasks progress. Rust cannot have both `sources.rs` and `sources/mod.rs` active at the same time. The implementation should complete the move in one working tree state before running compile checks.

- [ ] **Step 4: Compile checkpoint**

Run:

```powershell
Set-Location src-tauri
cargo test sources --lib
```

Expected at this exact early point: compile errors for missing modules until Task 2-7 are complete. Do not commit this partial state unless using a private worktree checkpoint branch.

## Task 2: Extract Settings

**Files:**
- Create: `src-tauri/src/sources/settings.rs`
- Modify: `src-tauri/src/sources/mod.rs`
- Modify: `src-tauri/src/sources/types.rs`

- [ ] **Step 1: Move settings code**

Move the settings constants, `InitialSyncMode`, `SyncSettingsRecord`, parsing, validation, app setting read/write, and Tauri commands into `settings.rs`.

- [ ] **Step 2: Keep these imports in `settings.rs`**

```rust
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::db::get_pool;
use crate::error::{AppError, AppResult};
```

- [ ] **Step 3: Move settings tests**

Move these tests from the old `sources::tests` module into `settings.rs`:

```rust
initial_sync_policy_label_formats_messages_and_days
validate_sync_settings_rejects_out_of_range_values
sync_settings_default_when_app_settings_are_missing
sync_settings_roundtrip_through_app_settings
```

Also move `memory_pool()` or create a local settings-specific version:

```rust
async fn memory_pool() -> sqlx::SqlitePool {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect memory sqlite");
    sqlx::query("CREATE TABLE app_settings (key TEXT PRIMARY KEY, value TEXT)")
        .execute(&pool)
        .await
        .expect("create app_settings");
    pool
}
```

- [ ] **Step 4: Run settings checkpoint**

Run:

```powershell
Set-Location src-tauri
cargo test sources::settings --lib
```

Expected after all symbols are wired: settings tests pass.

## Task 3: Extract Avatar And Peer Resolution

**Files:**
- Create: `src-tauri/src/sources/avatar.rs`
- Create: `src-tauri/src/sources/peer_resolution.rs`
- Modify: `src-tauri/src/sources/mod.rs`
- Modify: `src-tauri/src/sources/types.rs`

- [ ] **Step 1: Move avatar helpers**

Move photo timeout constants, photo download helpers, data URL conversion, and avatar cache helpers into `avatar.rs`.

- [ ] **Step 2: Keep these imports in `avatar.rs`**

```rust
use base64::{engine::general_purpose, Engine as _};
use grammers_client::peer::Peer;
use std::{fs, path::PathBuf};
use tauri::{AppHandle, Manager};
use tokio::time::{timeout, Duration};
```

- [ ] **Step 3: Move peer resolution helpers**

Move metadata structs, manual source ref parsing, peer resolution plan, add-source resolution, metadata encode/decode, peer ref reconstruction, and `resolve_and_refresh_peer` into `peer_resolution.rs`.

- [ ] **Step 4: Keep these imports in `peer_resolution.rs`**

```rust
use grammers_client::{peer::Peer, tl};
use grammers_session::types::{PeerAuth, PeerId, PeerRef};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::compression::{compress_json_bytes, decompress_bytes};
use super::avatar::{cache_source_avatar, peer_photo_bytes_with_timeout};
use super::types::{
    SourceSyncTarget, TelegramSourceInfo, TELEGRAM_KIND_CHANNEL, TELEGRAM_KIND_GROUP,
    TELEGRAM_KIND_SUPERGROUP, TELEGRAM_SOURCE_TYPE,
};
```

- [ ] **Step 5: Move peer resolution tests**

Move these tests into `peer_resolution.rs`:

```rust
source_metadata_decodes_old_username_only_payloads
source_metadata_decodes_old_dialog_payloads_into_peer_identity
source_metadata_roundtrip_preserves_peer_identity
parse_username_accepts_username_and_t_me_links
parse_supported_manual_telegram_source_ref_accepts_public_refs_and_numeric_ids
parse_supported_manual_telegram_source_ref_rejects_private_links
dialog_lookup_not_found_message_explains_numeric_manual_limit
add_source_resolution_strategy_distinguishes_username_and_dialog_flows
source_peer_resolution_plan_prefers_explicit_strategy_order
validate_expected_telegram_source_kind_reports_requested_and_actual_kind
peer_ref_from_identity_uses_channel_access_hash
peer_ref_from_identity_uses_supergroup_access_hash
peer_ref_from_identity_ignores_small_groups_without_supported_identity
source_peer_resolution_failure_explains_small_group_dialog_dependency
```

- [ ] **Step 6: Run peer checkpoint**

Run:

```powershell
Set-Location src-tauri
cargo test sources::peer_resolution --lib
```

Expected: peer and metadata tests pass.

## Task 4: Extract Store Commands

**Files:**
- Create: `src-tauri/src/sources/store.rs`
- Modify: `src-tauri/src/sources/mod.rs`
- Modify: `src-tauri/src/sources/types.rs`

- [ ] **Step 1: Move store functions**

Move source load/delete/list/add/list-dialog-source functions and `source_record_from_row` into `store.rs`.

- [ ] **Step 2: Keep these imports in `store.rs`**

```rust
use tauri::AppHandle;
use tokio::time::{Duration, Instant};

use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::source_ingest::{SourceIngestKind, SourceIngestLocks};
use crate::telegram::TelegramState;
use super::avatar::{
    cache_source_avatar, peer_photo_data_url_with_timeout, read_source_avatar_data_url,
    TELEGRAM_SOURCE_PHOTO_LIST_BUDGET_MS,
};
use super::peer_resolution::{
    decode_source_metadata, encode_source_metadata, resolve_telegram_source,
    source_metadata_for_added_source, telegram_source_info_from_peer,
};
use super::types::{now_secs, SourceRecord, SourceRecordRow, SourceSyncTarget};
```

- [ ] **Step 3: Move store tests**

Move this test into `store.rs`:

```rust
load_source_returns_not_found_for_missing_source
```

Use a local `memory_pool_with_sources()` helper containing the current `sources` table definition.

- [ ] **Step 4: Run store checkpoint**

Run:

```powershell
Set-Location src-tauri
cargo test sources::store --lib
```

Expected: store test passes. If no store-only module path exists after Rust test naming, run `cargo test load_source_returns_not_found_for_missing_source --lib`.

## Task 5: Extract Items

**Files:**
- Create: `src-tauri/src/sources/items.rs`
- Modify: `src-tauri/src/sources/mod.rs`
- Modify: `src-tauri/src/sources/types.rs`

- [ ] **Step 1: Move item DTOs and ingest structs**

Move `ItemRecord`, `ForumTopicFilter`, `SourceItemInsert`, and `TelegramItemContext` into `items.rs`.

- [ ] **Step 2: Move item behavior**

Move item insertion, item listing, item row loading, `message_author`, `extract_telegram_context`, `reply_peer_context`, and `build_raw_payload` into `items.rs`.

- [ ] **Step 3: Keep these imports in `items.rs`**

```rust
use grammers_client::tl;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::AppHandle;

use crate::compression::{compress_json_bytes, compress_text, decompress_text};
use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::forum_topics::{resolved_topic_join, ResolvedTopicAliases};
use crate::media::{decode_media_metadata, encode_media_metadata, ExtractedItemPayload};
use super::types::{now_secs, StoredItemRow};
```

- [ ] **Step 4: Move item tests**

Move these tests into `items.rs`:

```rust
text_roundtrip_through_zstd
media_metadata_roundtrip_through_zstd
insert_source_item_writes_payload_and_skips_duplicates
reply_peer_context_uses_telegram_peer_kinds
load_item_rows_attaches_topic_metadata_and_root_matches
```

`text_roundtrip_through_zstd` and `media_metadata_roundtrip_through_zstd` may be better moved to `compression.rs` and `media.rs` in a later cleanup, but keep them in `items.rs` for this refactor to avoid expanding scope.

- [ ] **Step 5: Run item checkpoint**

Run:

```powershell
Set-Location src-tauri
cargo test sources::items --lib
```

Expected: item tests pass.

## Task 6: Extract Forum Topics

**Files:**
- Create: `src-tauri/src/sources/topics.rs`
- Modify: `src-tauri/src/sources/mod.rs`
- Modify: `src-tauri/src/sources/types.rs`

- [ ] **Step 1: Move topic DTO and refresh/list behavior**

Move `SourceForumTopicRecord`, `ForumTopicSnapshot`, topic refresh, topic pagination cursor, topic upsert, topic listing, and refresh error classification into `topics.rs`.

- [ ] **Step 2: Keep these imports in `topics.rs`**

```rust
use grammers_client::{tl, Client};
use grammers_session::types::PeerRef;
use serde::Serialize;
use tauri::AppHandle;

use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::forum_topics::{
    resolved_topic_join, resolved_topic_predicate, ResolvedTopicAliases,
    FORUM_TOPIC_UNCATEGORIZED_KEY, FORUM_TOPIC_UNCATEGORIZED_TITLE,
};
use super::types::{
    now_secs, SourceForumTopicRow, SourceSyncTarget, TELEGRAM_KIND_SUPERGROUP,
};
```

- [ ] **Step 3: Move topic tests**

Move these tests into `topics.rs`:

```rust
list_source_forum_topics_returns_sorted_topics_and_uncategorized_bucket
upsert_forum_topics_refresh_preserves_missing_topics_and_marks_deleted
non_forum_topic_refresh_errors_are_detected
```

Use a local `memory_pool_with_source_items_and_topics()` helper or move common test DB helpers to `sources/test_support.rs` behind `#[cfg(test)]`.

- [ ] **Step 4: Run topic checkpoint**

Run:

```powershell
Set-Location src-tauri
cargo test sources::topics --lib
```

Expected: topic tests pass.

## Task 7: Extract Sync

**Files:**
- Create: `src-tauri/src/sources/sync.rs`
- Modify: `src-tauri/src/sources/mod.rs`
- Modify: `src-tauri/src/sources/types.rs`

- [ ] **Step 1: Move sync DTO and orchestration**

Move `SyncResult`, `SyncPolicy`, `IngestOutcome`, `determine_sync_policy`, `persist_items`, `sync_source`, and `finalize_sync` into `sync.rs`.

- [ ] **Step 2: Keep these imports in `sync.rs`**

```rust
use grammers_session::types::PeerRef;
use serde::Serialize;
use tauri::AppHandle;

use crate::db::get_pool;
use crate::error::{AppError, AppResult};
use crate::media::extract_item_payload;
use crate::source_ingest::{SourceIngestKind, SourceIngestLocks};
use crate::telegram::TelegramState;
use super::items::{
    build_raw_payload, extract_telegram_context, insert_source_item, message_author,
    SourceItemInsert,
};
use super::peer_resolution::resolve_and_refresh_peer;
use super::settings::{
    initial_sync_policy_label, load_sync_settings_from_pool, InitialSyncMode, SyncSettingsRecord,
    SECONDS_PER_DAY,
};
use super::store::load_source;
use super::topics::refresh_forum_topics;
use super::types::{now_secs, SourceSyncTarget};
```

- [ ] **Step 3: Move sync tests**

Move these tests into `sync.rs`:

```rust
determine_sync_policy_only_applies_initial_settings_on_first_sync
finalize_sync_updates_source_state_and_metadata
```

Use local source table setup or shared `test_support`.

- [ ] **Step 4: Run sync checkpoint**

Run:

```powershell
Set-Location src-tauri
cargo test sources::sync --lib
```

Expected: sync tests pass.

## Task 8: Remove Old Monolith And Fix Visibility

**Files:**
- Delete: `src-tauri/src/sources.rs`
- Modify: `src-tauri/src/sources/*.rs`

- [ ] **Step 1: Delete old file module**

Delete `src-tauri/src/sources.rs` only after all moved symbols compile from the directory module.

- [ ] **Step 2: Tighten visibility**

Use this visibility rule:

```text
pub        only for serialized DTOs and Tauri commands exported through the facade
pub(crate) only for Takeout-facing APIs and structs
pub(super) only for cross-module helpers inside sources
private    for module-local helpers and tests
```

- [ ] **Step 3: Search for accidental broad exports**

Run:

```powershell
rg -n "pub\\(|pub(crate)|pub use|pub struct|pub enum|pub async fn|pub fn" src-tauri/src/sources
```

Expected: exports match the facade and Takeout-facing contract lists above.

## Task 9: Full Verification

**Files:**
- No intentional file edits in this task.

- [ ] **Step 1: Run backend sources tests**

Run:

```powershell
Set-Location src-tauri
cargo test sources --lib
```

Expected: all moved sources tests pass.

- [ ] **Step 2: Run full backend tests**

Run:

```powershell
Set-Location src-tauri
cargo test
```

Expected: full Rust test suite passes.

- [ ] **Step 3: Run frontend tests**

Run:

```powershell
Set-Location ..
npm.cmd test
```

Expected: frontend tests pass with the same test count as baseline unless unrelated tests were added.

- [ ] **Step 4: Run Svelte/type check**

Run:

```powershell
npm.cmd run check
```

Expected: 0 errors and 0 warnings.

- [ ] **Step 5: Run whitespace check**

Run:

```powershell
git diff --check
```

Expected: no real whitespace errors. Windows LF-to-CRLF warnings in Markdown may appear depending on Git settings and should not be confused with whitespace failures.

## Task 10: Update Review Documentation

**Files:**
- Modify: `docs/code-review-results-2026-05-03.md`
- Modify: `docs/session-context-2026-05-03.md`

- [ ] **Step 1: Update code review finding**

After implementation and verification, update the `Large backend modules mix unrelated behavior` section:

```markdown
Planning status:

- first Takeout implementation slice is documented in
  `docs/superpowers/plans/2026-05-03-takeout-import-backend-split.md` and has been implemented;
- `sources.rs` split is documented in
  `docs/superpowers/plans/2026-05-03-sources-backend-split.md` and has been implemented;
- any remaining Takeout orchestration in `mod.rs` is intentional for this first pass.
```

If implementation is not complete yet, write `is ready for implementation` instead of `has been implemented`.

- [ ] **Step 2: Update session handoff**

Add the completed or ready plan path to `docs/session-context-2026-05-03.md`.

- [ ] **Step 3: Run doc whitespace check**

Run:

```powershell
git diff --check docs/code-review-results-2026-05-03.md docs/session-context-2026-05-03.md docs/superpowers/plans/2026-05-03-sources-backend-split.md
```

Expected: no real whitespace errors.

## Implementation Notes

- Prefer moving code unchanged first, then running `cargo fmt` only after behavior is compiling. This reduces review noise.
- Do not introduce new abstractions beyond the listed modules.
- Do not change database SQL unless imports force tiny formatting changes. Query semantics must stay the same.
- Do not move Takeout orchestration out of `takeout_import/mod.rs` in this task.
- Do not change `src-tauri/src/lib.rs` unless the compiler requires import path cleanup; the final command import names must remain unchanged.
- If module-local test helpers duplicate the current in-memory SQLite setup, that is acceptable for this refactor. A shared `test_support.rs` is allowed only if duplication becomes harder to maintain than the helper.

## Suggested Commits

Use small commits if implementing inline:

```text
refactor(sources): split shared types and settings
refactor(sources): extract avatar and peer resolution
refactor(sources): extract store items topics and sync
docs(sources): record backend split plan status
```

For a single final commit after all tests pass:

```text
refactor(sources): split backend module by behavior
```

