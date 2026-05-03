# Sources Contract V2 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Introduce a typed Sources Contract V2 that centralizes frontend source commands, moves UI code to camelCase domain types, tightens Rust source DTOs and errors, and keeps SQLite data compatible.

**Architecture:** Rust keeps small explicit Tauri commands and source-owned domain validation. Frontend code calls `$lib/api/sources` instead of raw source `invoke(...)` calls. The wrapper translates raw Tauri DTOs into camelCase domain types so UI code no longer mirrors Rust serde field names.

**Tech Stack:** Tauri 2, Rust 2021, serde, sqlx SQLite, SvelteKit 2, Svelte 5, TypeScript, Vitest, cargo test.

---

## Reference Spec

Read first:

- `docs/superpowers/specs/2026-05-03-sources-contract-v2-design.md`
- `docs/code-review-results-2026-05-03.md`
- `docs/superpowers/plans/2026-05-03-sources-backend-split.md`

Baseline facts:

- `cargo test sources --lib` currently passes with 30 tests.
- `src-tauri/src/sources.rs` no longer exists.
- Core source module files are under `src-tauri/src/sources/`.
- Frontend source calls are still raw in `/analysis` and source management dialog.

## Compilation Checkpoint Policy

Each top-level task must leave the project compiling at its listed verification checkpoint. Do not
stop after an individual checkbox inside a task unless the task's verification command has passed.

Important ordering constraints:

- Task 1 must add the minimal `TelegramSourceKind` type before using it in request DTOs.
- Task 3 must add camelCase domain types and the wrapper without deleting the old core `*Record`
  exports yet, so existing UI call sites continue to type-check.
- Task 4 migrates UI call sites and then removes the old core `*Record` exports.
- If an implementer chooses not to keep temporary frontend compatibility in Task 3, Tasks 3 and 4
  must be executed as one compile checkpoint.

## File Map

Create:

- `src/lib/api/sources.ts`: source command wrapper, raw DTOs, mappers.
- `src/lib/api/sources.test.ts`: wrapper command and mapping tests.
- `src-tauri/src/sources/test_support.rs`: shared source-module SQLite fixtures behind `#[cfg(test)]`.

Modify:

- `src/lib/types/sources.ts`: rename core UI types to camelCase domain types while preserving Takeout and NotebookLM types.
- `src/routes/analysis/+page.svelte`: replace source-related raw invokes and camelCase source fields.
- `src/lib/components/analysis/source-management-dialog.svelte`: replace source-related raw invokes and camelCase source fields.
- `src/lib/analysis-state.ts`: rename source types and fields.
- `src/lib/analysis-source-state.ts`: rename source type and fields.
- `src/lib/analysis-scope-state.ts`: rename source type.
- `src/lib/components/source-messages-panel.svelte`: rename item fields.
- `src/lib/components/source-row.svelte`: rename source fields.
- `src/lib/components/analysis/workspace-rail.svelte`: rename source fields.
- `src/lib/components/analysis/workspace-main.svelte`: rename source fields.
- `src/lib/components/analysis/source-context-panel.svelte`: rename source item/topic fields.
- `src/lib/components/analysis/notebooklm-export-dialog.svelte`: update source type imports if needed.
- `src-tauri/src/lib.rs`: register `list_source_items` instead of `get_items`.
- `src-tauri/src/sources/mod.rs`: export `list_source_items` and new request/domain types as needed.
- `src-tauri/src/sources/types.rs`: add source enums and validation helpers.
- `src-tauri/src/sources/items.rs`: add `ListSourceItemsRequest`, rename command, use typed request.
- `src-tauri/src/sources/store.rs`: add `AddTelegramSourceRequest`, use typed source kind.
- `src-tauri/src/sources/settings.rs`: add `SaveSyncSettingsRequest`.
- `src-tauri/src/sources/peer_resolution.rs`: use shared source-kind type/validation and typed errors.
- `src-tauri/src/sources/topics.rs`: use shared source-kind type/validation and shared test fixtures.
- `src-tauri/src/sources/sync.rs`: use shared source-kind type/validation and shared test fixtures.
- `src-tauri/src/takeout_import/mod.rs`, `src-tauri/src/takeout_import/export_dc.rs`, `src-tauri/src/takeout_import/pagination.rs`: remove duplicated source-kind constants where practical.

## Task 1: Backend Command Contract

**Files:**
- Modify: `src-tauri/src/sources/types.rs`
- Modify: `src-tauri/src/sources/items.rs`
- Modify: `src-tauri/src/sources/store.rs`
- Modify: `src-tauri/src/sources/settings.rs`
- Modify: `src-tauri/src/sources/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] Add minimal source kind domain type in `types.rs`.

This must happen before adding request DTOs that reference `TelegramSourceKind`.

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TelegramSourceKind {
    Channel,
    Supergroup,
    Group,
}

impl TelegramSourceKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Channel => "channel",
            Self::Supergroup => "supergroup",
            Self::Group => "group",
        }
    }
}
```

`TelegramSourceKind` must be `pub`, not `pub(crate)`, because it appears in a public command
request DTO field.

- [ ] Add `ListSourceItemsRequest` in `items.rs`.

Use this shape:

```rust
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListSourceItemsRequest {
    pub source_id: i64,
    pub limit: i64,
    pub before_published_at: Option<i64>,
    pub topic_filter: Option<ForumTopicFilter>,
}
```

- [ ] Update `ForumTopicFilter` wire casing.

Keep the Rust field name `topic_id`, but make the Tauri request payload use `topicId`:

```rust
#[derive(Clone, serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ForumTopicFilter {
    Topic {
        #[serde(rename = "topicId")]
        topic_id: i64,
    },
    Uncategorized,
}
```

Add a small serde test in `items.rs` or `types.rs`:

```rust
#[test]
fn forum_topic_filter_deserializes_camel_case_topic_id() {
    let filter: ForumTopicFilter =
        serde_json::from_str(r#"{"kind":"topic","topicId":200}"#).expect("deserialize");
    assert_eq!(filter, ForumTopicFilter::Topic { topic_id: 200 });
}
```

- [ ] Rename command function `get_items` to `list_source_items`.

Expected command signature:

```rust
#[tauri::command]
pub async fn list_source_items(
    handle: AppHandle,
    request: ListSourceItemsRequest,
) -> AppResult<Vec<ItemRecord>> {
    let pool = get_pool(&handle).await?;
    let limit = request.limit.clamp(1, 200);
    let rows = load_item_rows_from_pool(
        &pool,
        request.source_id,
        limit,
        request.before_published_at,
        request.topic_filter,
    )
    .await?;

    rows.into_iter().map(item_record_from_row).collect()
}
```

Create `item_record_from_row(row: StoredItemRow) -> AppResult<ItemRecord>` so row-to-DTO mapping is testable without the Tauri command wrapper.

- [ ] Add `AddTelegramSourceRequest` in `store.rs`.

Use this shape:

```rust
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddTelegramSourceRequest {
    pub account_id: i64,
    pub source_ref: String,
    pub expected_kind: Option<super::types::TelegramSourceKind>,
}
```

Change `add_telegram_source` to receive `request: AddTelegramSourceRequest`. Use
`request.expected_kind.map(TelegramSourceKind::as_str)` or an equivalent helper when calling
existing resolution code during the first pass.

- [ ] Add `SaveSyncSettingsRequest` in `settings.rs`.

Use this shape:

```rust
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveSyncSettingsRequest {
    pub initial_sync_mode: InitialSyncMode,
    pub initial_sync_value: i64,
}
```

Change `save_sync_settings` to receive `settings: SaveSyncSettingsRequest`.

- [ ] Update `src-tauri/src/sources/mod.rs`.

Export `list_source_items` instead of `get_items`. Export request DTOs only if tests or other
modules need them.

- [ ] Update `src-tauri/src/lib.rs`.

Replace imports and `tauri::generate_handler!` entry:

```rust
list_source_items,
```

Remove:

```rust
get_items,
```

- [ ] Run backend focused test.

```powershell
Set-Location src-tauri
cargo test sources --lib
```

Expected after completing this task: source tests compile and pass. Frontend may still call old
runtime command strings until Tasks 3-4, but TypeScript should still compile because raw invoke
strings are not checked against Rust command registration.

Commit checkpoint:

```powershell
git add src-tauri/src/lib.rs src-tauri/src/sources
git commit -m "refactor(sources): introduce contract v2 command requests"
```

## Task 2: Rust Source Domain Reuse

**Files:**
- Modify: `src-tauri/src/sources/types.rs`
- Modify: `src-tauri/src/sources/peer_resolution.rs`
- Modify: `src-tauri/src/sources/topics.rs`
- Modify: `src-tauri/src/sources/sync.rs`
- Modify: `src-tauri/src/sources/store.rs`
- Modify: `src-tauri/src/takeout_import/mod.rs`
- Modify: `src-tauri/src/takeout_import/export_dc.rs`
- Modify: `src-tauri/src/takeout_import/pagination.rs`

- [ ] Complete source domain helpers in `types.rs`.

Task 1 already added minimal `TelegramSourceKind`. In this task, add `SourceType`, add
`TelegramSourceKind::parse`, and add tests. Final enum values:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    Telegram,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TelegramSourceKind {
    Channel,
    Supergroup,
    Group,
}
```

Add helpers:

```rust
impl SourceType {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Telegram => "telegram",
        }
    }
}

impl TelegramSourceKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Channel => "channel",
            Self::Supergroup => "supergroup",
            Self::Group => "group",
        }
    }

    pub(crate) fn parse(value: &str) -> crate::error::AppResult<Self> {
        match value {
            "channel" => Ok(Self::Channel),
            "supergroup" => Ok(Self::Supergroup),
            "group" => Ok(Self::Group),
            other => Err(crate::error::AppError::validation(format!(
                "Unsupported telegram_source_kind '{other}'"
            ))),
        }
    }
}
```

Keep existing string constants during the transition only if replacing them all in one commit makes
the diff harder to review. The final state should not have duplicate source-kind constants in both
`sources` and Takeout modules.

- [ ] Add tests for source-kind parse and serde.

Add tests in `types.rs`:

```rust
#[test]
fn telegram_source_kind_parses_supported_values() {
    assert_eq!(TelegramSourceKind::parse("channel").unwrap(), TelegramSourceKind::Channel);
    assert_eq!(
        TelegramSourceKind::parse("supergroup").unwrap(),
        TelegramSourceKind::Supergroup
    );
    assert_eq!(TelegramSourceKind::parse("group").unwrap(), TelegramSourceKind::Group);
}

#[test]
fn telegram_source_kind_rejects_unknown_values_as_validation() {
    let error = TelegramSourceKind::parse("user").expect_err("unsupported kind");
    assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
}

#[test]
fn telegram_source_kind_serializes_as_existing_wire_value() {
    let value = serde_json::to_string(&TelegramSourceKind::Supergroup).expect("serialize");
    assert_eq!(value, "\"supergroup\"");
}
```

- [ ] Replace local validation helpers.

Replace `ensure_supported_telegram_source_kind` in `peer_resolution.rs` with
`TelegramSourceKind::parse`. Keep user-facing validation messages equivalent enough that frontend
status remains understandable.

- [ ] Reuse source-kind helpers in Takeout.

In Takeout modules, import source-kind constants/helpers from `crate::sources` only if visibility
stays sensible. If importing the enum creates awkward visibility, expose a small crate-visible helper
from `sources::types` rather than duplicating string constants again.

- [ ] Run backend tests.

```powershell
Set-Location src-tauri
cargo test sources --lib
cargo test takeout --lib
```

Expected: source and Takeout-focused tests pass.

Commit checkpoint:

```powershell
git add src-tauri/src/sources src-tauri/src/takeout_import
git commit -m "refactor(sources): centralize source kind validation"
```

## Task 3: Frontend Domain Types And API Wrapper

**Files:**
- Modify: `src/lib/types/sources.ts`
- Create: `src/lib/api/sources.ts`
- Create: `src/lib/api/sources.test.ts`

- [ ] Add camelCase domain types while keeping old core `Record` exports temporarily.

In `src/lib/types/sources.ts`, add these new domain types:

- `TelegramSourceInfo` to `TelegramDialogSource`
- `SourceRecord` to `Source`
- `ItemRecord` to `SourceItem`
- `SourceForumTopicRecord` to `SourceForumTopic`
- `SyncResult` to `SyncSourceResult`
- `SyncSettingsRecord` to `SyncSettings`

Use the exact type shapes from `docs/superpowers/specs/2026-05-03-sources-contract-v2-design.md`.

Do not remove the old `TelegramSourceInfo`, `SourceRecord`, `ItemRecord`,
`SourceForumTopicRecord`, `SyncResult`, or `SyncSettingsRecord` exports in this task. Existing UI
files still import and use them until Task 4.

Keep Takeout import and NotebookLM export types in the same file for this plan.

- [ ] Create `src/lib/api/sources.ts`.

The wrapper owns raw snake_case DTOs and mapping functions. Use command constants:

```ts
const SOURCE_COMMANDS = {
  listSources: "list_sources",
  listTelegramSources: "list_telegram_sources",
  addTelegramSource: "add_telegram_source",
  deleteSource: "delete_source",
  getSyncSettings: "get_sync_settings",
  saveSyncSettings: "save_sync_settings",
  syncSource: "sync_source",
  listSourceItems: "list_source_items",
  listSourceForumTopics: "list_source_forum_topics",
} as const;
```

Public functions:

```ts
export function listSources(accountId: number | null) { /* invoke + map */ }
export function listTelegramSources(accountId: number) { /* invoke + map */ }
export function addTelegramSource(input: AddTelegramSourceInput) { /* request wrapper + map */ }
export function deleteSource(sourceId: number) { return invoke<void>(...); }
export function getSyncSettings() { /* invoke + map */ }
export function saveSyncSettings(settings: SyncSettings) { /* request wrapper + map */ }
export function syncSource(sourceId: number) { /* invoke + map */ }
export function listSourceItems(input: ListSourceItemsInput) { /* request wrapper + map */ }
export function listSourceForumTopics(sourceId: number) { /* invoke + map */ }
```

Mapping rule:

- raw `telegram_source_kind` -> domain `telegramSourceKind`
- raw `account_id` -> domain `accountId`
- raw `last_sync_state` -> domain `lastSyncState`
- raw `last_synced_at` -> domain `lastSyncedAt`
- raw `avatar_data_url` -> domain `avatarDataUrl`
- raw `published_at` -> domain `publishedAt`
- raw topic fields -> corresponding camelCase names
- domain `ForumTopicFilter.Topic.topicId` -> Tauri wire `topicId`

- [ ] Add `src/lib/api/sources.test.ts`.

Follow the same mock pattern as `src/lib/api/analysis-runs.test.ts`.

Test cases:

- `listSources(null)` calls `list_sources` with `{ accountId: null }` and maps response to
  `accountId`, `telegramSourceKind`, and `avatarDataUrl`.
- `addTelegramSource(...)` calls `add_telegram_source` with
  `{ request: { accountId, sourceRef, expectedKind } }`.
- `saveSyncSettings(...)` calls `save_sync_settings` with `{ settings: { initialSyncMode, initialSyncValue } }`.
- `listSourceItems(...)` calls `list_source_items` with
  `{ request: { sourceId, limit, beforePublishedAt, topicFilter: { kind: "topic", topicId } } }`.
- `listSourceForumTopics(...)` maps `message_count`, `top_message_id`, and icon fields.

- [ ] Run frontend wrapper test.

```powershell
npm.cmd test -- src/lib/api/sources.test.ts
npm.cmd run check
```

Expected: new wrapper tests pass and Svelte check still reports 0 errors and 0 warnings because old
core `Record` exports remain available.

Commit checkpoint:

```powershell
git add src/lib/types/sources.ts src/lib/api/sources.ts src/lib/api/sources.test.ts
git commit -m "refactor(sources): add typed frontend api facade"
```

## Task 4: Frontend Call Site Migration

**Files:**
- Modify: `src/routes/analysis/+page.svelte`
- Modify: `src/lib/components/analysis/source-management-dialog.svelte`
- Modify: `src/lib/analysis-state.ts`
- Modify: `src/lib/analysis-source-state.ts`
- Modify: `src/lib/analysis-scope-state.ts`
- Modify: `src/lib/components/source-messages-panel.svelte`
- Modify: `src/lib/components/source-row.svelte`
- Modify: `src/lib/components/analysis/workspace-rail.svelte`
- Modify: `src/lib/components/analysis/workspace-main.svelte`
- Modify: `src/lib/components/analysis/source-context-panel.svelte`
- Modify: affected tests under `src/lib/*.test.ts`
- Modify: `src/lib/types/sources.ts`

- [ ] Replace source-related raw imports.

In `/analysis`, keep raw `invoke` only for non-source commands not included in this plan. Import
source functions from `$lib/api/sources`.

Replace:

```ts
invoke<SourceRecord[]>("list_sources", { accountId: null })
invoke<SourceForumTopicRecord[]>("list_source_forum_topics", { sourceId })
invoke<ItemRecord[]>("get_items", ...)
invoke<SyncResult>("sync_source", { sourceId })
invoke("delete_source", { sourceId: source.id })
```

with:

```ts
listSources(null)
listSourceForumTopics(sourceId)
listSourceItems({ sourceId, limit: 120, beforePublishedAt: null, topicFilter: currentTopicFilter() })
syncSource(sourceId)
deleteSource(source.id)
```

- [ ] Replace source-management dialog raw invokes.

Replace:

```ts
invoke<TelegramSourceInfo[]>("list_telegram_sources", { accountId })
invoke<SourceRecord>("add_telegram_source", { accountId, sourceRef, telegramSourceKind })
```

with:

```ts
listTelegramSources(accountId)
addTelegramSource({ accountId, sourceRef, expectedKind })
```

- [ ] Rename frontend source fields.

Apply these field migrations:

- `source.source_type` -> `source.sourceType`
- `source.telegram_source_kind` -> `source.telegramSourceKind`
- `source.account_id` -> `source.accountId`
- `source.external_id` -> `source.externalId`
- `source.last_sync_state` -> `source.lastSyncState`
- `source.last_synced_at` -> `source.lastSyncedAt`
- `source.is_member` -> `source.isMember`
- `source.is_active` -> `source.isActive`
- `source.created_at` -> `source.createdAt`
- `source.avatar_data_url` -> `source.avatarDataUrl`
- `dialogSource.telegram_source_kind` -> `dialogSource.telegramSourceKind`
- `dialogSource.is_member` -> `dialogSource.isMember`
- `dialogSource.photo_data_url` -> `dialogSource.photoDataUrl`
- `item.source_id` -> `item.sourceId`
- `item.external_id` -> `item.externalId`
- `item.published_at` -> `item.publishedAt`
- `item.content_kind` -> `item.contentKind`
- `item.has_media` -> `item.hasMedia`
- `item.media_kind` -> `item.mediaKind`
- `item.media_summary` -> `item.mediaSummary`
- `item.media_file_name` -> `item.mediaFileName`
- `item.media_mime_type` -> `item.mediaMimeType`
- `item.has_raw_data` -> `item.hasRawData`
- `item.forum_topic_title` -> `item.forumTopicTitle`
- `topic.message_count` -> `topic.messageCount`
- `topic.topic_id` -> `topic.topicId`
- `topic.top_message_id` -> `topic.topMessageId`
- `topic.icon_color` -> `topic.iconColor`
- `topic.icon_emoji_id` -> `topic.iconEmojiId`
- `topic.is_closed` -> `topic.isClosed`
- `topic.is_pinned` -> `topic.isPinned`
- `topic.is_hidden` -> `topic.isHidden`
- `topic.is_deleted` -> `topic.isDeleted`
- `topic.sort_order` -> `topic.sortOrder`

- [ ] Update topic filter helpers.

`currentTopicFilter` in `analysis-state.ts` should return:

```ts
{ kind: "topic", topicId: topic.topicId }
```

instead of:

```ts
{ kind: "topic", topic_id: topic.topic_id }
```

- [ ] Remove old core `Record` exports.

After all UI call sites and tests use the new domain type names and camelCase fields, remove these
temporary compatibility exports from `src/lib/types/sources.ts`:

- `TelegramSourceInfo`
- `SourceRecord`
- `ItemRecord`
- `SourceForumTopicRecord`
- `SyncResult`
- `SyncSettingsRecord`

Do not remove Takeout import or NotebookLM export types.

- [ ] Run frontend tests and check.

```powershell
npm.cmd test
npm.cmd run check
```

Expected: all Vitest tests pass and Svelte check reports 0 errors and 0 warnings.

Commit checkpoint:

```powershell
git add src/routes/analysis/+page.svelte src/lib
git commit -m "refactor(sources): move ui to source domain types"
```

## Task 5: Backend Typed Errors

**Files:**
- Modify: `src-tauri/src/sources/items.rs`
- Modify: `src-tauri/src/sources/store.rs`
- Modify: `src-tauri/src/sources/settings.rs`
- Modify: `src-tauri/src/sources/peer_resolution.rs`
- Modify: `src-tauri/src/sources/topics.rs`
- Modify: `src-tauri/src/sources/sync.rs`
- Modify: `src-tauri/src/sources/avatar.rs`

- [ ] Convert source command and service boundaries to `AppResult<T>`.

Prioritize functions that are called by commands or by other modules:

- `insert_source_item`
- `resolve_and_refresh_peer`
- `finalize_sync`
- `resolve_telegram_source`
- `encode_source_metadata`
- `decode_source_metadata`
- `refresh_forum_topics` warning creation path

Keep private compression helpers returning `String` only when changing them would expand the scope
outside `sources`.

- [ ] Replace classification-sensitive errors.

Use explicit constructors:

```rust
return Err(AppError::validation("Telegram source reference cannot be empty"));
return Err(AppError::not_found(format!("Source {source_id} not found")));
return Err(AppError::network(error.to_string()));
return Err(AppError::internal(error.to_string()));
```

Do not add new source code paths that rely on `AppError::from(e.to_string())` for user-visible
classification.

- [ ] Add or update focused error tests.

Coverage targets:

- unsupported source kind is `Validation`;
- missing source is `NotFound`;
- malformed numeric `external_id` is `Validation`;
- Telegram dialog lookup miss is `NotFound`;
- compression/metadata decode failure is `Internal` when surfaced at source boundary.

- [ ] Run backend tests.

```powershell
Set-Location src-tauri
cargo test sources --lib
cargo test
```

Expected: all source tests and full backend tests pass.

Commit checkpoint:

```powershell
git add src-tauri/src/sources src-tauri/src/error.rs
git commit -m "refactor(sources): tighten source error typing"
```

## Task 6: Shared Source Test Fixtures

**Files:**
- Create: `src-tauri/src/sources/test_support.rs`
- Modify: `src-tauri/src/sources/mod.rs`
- Modify: `src-tauri/src/sources/items.rs`
- Modify: `src-tauri/src/sources/topics.rs`
- Modify: `src-tauri/src/sources/store.rs`
- Modify: `src-tauri/src/sources/sync.rs`

- [ ] Add `test_support` module behind cfg.

In `mod.rs`:

```rust
#[cfg(test)]
mod test_support;
```

- [ ] Create shared helpers.

`test_support.rs` should provide:

```rust
pub(super) async fn memory_pool() -> sqlx::SqlitePool
pub(super) async fn memory_pool_with_sources() -> sqlx::SqlitePool
pub(super) async fn memory_pool_with_source_items_and_topics() -> sqlx::SqlitePool
```

Use the table definitions already duplicated in `items.rs`, `topics.rs`, `store.rs`, and `sync.rs`.
The helper must create exactly the columns used by existing source tests.

- [ ] Replace duplicated test schema setup.

Update source module tests to import:

```rust
use crate::sources::test_support::{
    memory_pool_with_source_items_and_topics,
    memory_pool_with_sources,
};
```

Use only the helpers needed by each test module.

- [ ] Run focused source tests.

```powershell
Set-Location src-tauri
cargo test sources --lib
```

Expected: 30 or more source tests pass, depending on new tests added in earlier tasks.

Commit checkpoint:

```powershell
git add src-tauri/src/sources
git commit -m "test(sources): share sqlite fixtures"
```

## Task 7: Targeted Rust Extraction

**Files:**
- Modify/Create under `src-tauri/src/sources/`

- [ ] Extract only when it reduces active coupling.

Allowed extractions:

- `src-tauri/src/sources/peer_resolution/manual_ref.rs`
- `src-tauri/src/sources/peer_resolution/metadata.rs`
- `src-tauri/src/sources/items/query.rs`
- `src-tauri/src/sources/topics/list.rs`

Use this rule: extract if the new file owns a complete behavior with tests and a clear caller.
Do not create empty pass-through modules.

- [ ] Preserve public facade.

`src-tauri/src/sources/mod.rs` should keep the same crate-visible responsibilities:

- public Tauri command exports;
- crate-visible Takeout-facing exports;
- source type exports needed outside the module.

- [ ] Move tests with behavior.

When extracting pure parsing/mapping/listing logic, move its tests beside the extracted module.
Keep integration-style database tests in the command-orchestration module unless the extracted
module owns the query.

- [ ] Run backend tests after each extraction.

```powershell
Set-Location src-tauri
cargo test sources --lib
```

Expected after each extraction: source tests pass.

Commit checkpoint after the last extraction:

```powershell
git add src-tauri/src/sources
git commit -m "refactor(sources): extract focused source helpers"
```

## Task 8: Final Verification And Documentation

**Files:**
- Modify: `docs/code-review-results-2026-05-03.md`
- Modify: `docs/session-context-2026-05-03.md`
- Modify: `docs/superpowers/plans/2026-05-03-sources-contract-v2.md`

- [ ] Run full verification.

```powershell
Set-Location src-tauri
cargo test sources --lib
cargo test
Set-Location ..
npm.cmd test
npm.cmd run check
git diff --check
```

Expected:

- source tests pass;
- full Rust tests pass;
- Vitest suite passes;
- Svelte check reports 0 errors and 0 warnings;
- `git diff --check` reports no real whitespace errors.

- [ ] Update review results.

In `docs/code-review-results-2026-05-03.md`, record that source Contract V2 is complete and remove
or narrow findings that this work resolves:

- raw source Tauri command strings in `/analysis`;
- manually mirrored source DTO risk for core sources;
- source-local string error classification.

Do not claim Takeout/NotebookLM wrappers are complete.

- [ ] Update session context.

In `docs/session-context-2026-05-03.md`, add:

- final command names;
- final frontend source API facade path;
- final verification results;
- any follow-up intentionally deferred.

- [ ] Mark this plan complete.

After merge, replace task details with a completion record if that is the repository's current
documentation style for finished plans.

Final commit checkpoint:

```powershell
git add docs src-tauri/src src/lib src/routes
git commit -m "docs(sources): record contract v2 completion"
```

## Acceptance Criteria

- No frontend source workflow calls core source Tauri commands directly.
- UI source domain objects are camelCase.
- Raw source DTO mapping is centralized in `src/lib/api/sources.ts`.
- `get_items` no longer exists; `list_source_items` is the registered command.
- New source request DTOs use camelCase Tauri wire fields.
- Source-kind validation is centralized.
- Source module service boundaries no longer add new string-classified user-visible errors.
- Repeated source SQLite test setup is consolidated.
- No SQLite migration is introduced.

## Deferred Work

- Rust-to-TypeScript type generation.
- Takeout import frontend API wrapper.
- NotebookLM export frontend API wrapper.
- Secure secret storage.
- Full media download/preview.
