# Sources Contract V2 Design

## Purpose

Second-wave `src-tauri/src/sources` work should improve maintainability, consistency,
extensibility, testability, and duplication after the first backend split.

The first wave moved the old `src-tauri/src/sources.rs` monolith into focused modules while
preserving all command names and serialized DTO shapes. The second wave may change the external
API where the change removes real coupling. SQLite schema and stored source data stay compatible.

## Current Repo Facts

- Backend module is `src-tauri/src/sources/`.
- Core source commands are registered in `src-tauri/src/lib.rs`.
- Current command set:
  - `get_sync_settings`
  - `save_sync_settings`
  - `delete_source`
  - `list_telegram_sources`
  - `add_telegram_source`
  - `list_sources`
  - `sync_source`
  - `get_items`
  - `list_source_forum_topics`
- Frontend source calls are still raw `invoke(...)` calls in:
  - `src/routes/analysis/+page.svelte`
  - `src/lib/components/analysis/source-management-dialog.svelte`
- Existing frontend API wrapper pattern lives in:
  - `src/lib/api/analysis-runs.ts`
  - `src/lib/api/analysis-runs.test.ts`
  - `src/lib/api/llm.ts`
  - `src/lib/api/llm.test.ts`
- `src/lib/types/sources.ts` currently mixes core source DTOs with Takeout import and
  NotebookLM export DTOs.
- `cargo test sources --lib` currently passes with 30 tests.

## Decisions

- Scope is core `src-tauri/src/sources` commands only.
- Takeout import and NotebookLM export wrappers are deferred to separate work.
- Add a frontend source API facade in `src/lib/api/sources.ts`.
- UI-facing source types become camelCase domain types.
- Raw snake_case Tauri response DTOs stay private to the frontend wrapper.
- During implementation, old core `*Record` frontend exports may remain temporarily until all
  source UI call sites are migrated. They should be removed before the work is complete.
- New Rust request DTOs use camelCase Tauri wire fields with `serde(rename_all = "camelCase")`.
- Rename `get_items` to `list_source_items` with no compatibility alias.
- Do not add Rust-to-TypeScript type generation in this wave.
- Do not add a database migration.
- Further Rust splitting is targeted, not aggressive.

## Frontend Contract

`src/lib/api/sources.ts` will expose these functions:

| Function | Tauri command | Notes |
|---|---|---|
| `listSources(accountId)` | `list_sources` | Maps raw `SourceRecord` to `Source`. |
| `listTelegramSources(accountId)` | `list_telegram_sources` | Maps raw dialog source rows to `TelegramDialogSource`. |
| `addTelegramSource(input)` | `add_telegram_source` | Sends `{ request: AddTelegramSourceRequest }`. |
| `deleteSource(sourceId)` | `delete_source` | Keeps simple id payload. |
| `getSyncSettings()` | `get_sync_settings` | Maps raw sync settings to `SyncSettings`. |
| `saveSyncSettings(settings)` | `save_sync_settings` | Sends `{ settings: SaveSyncSettingsRequest }`. |
| `syncSource(sourceId)` | `sync_source` | Maps raw result to `SyncSourceResult`. |
| `listSourceItems(input)` | `list_source_items` | Sends `{ request: ListSourceItemsRequest }`. |
| `listSourceForumTopics(sourceId)` | `list_source_forum_topics` | Maps raw topics to `SourceForumTopic`. |

UI-facing source domain types in `src/lib/types/sources.ts`:

```ts
export type SourceType = "telegram";
export type TelegramSourceKind = "channel" | "supergroup" | "group";
export type InitialSyncMode = "recent_messages" | "recent_days";
export type DialogKindFilter = "all" | TelegramSourceKind;

export interface TelegramDialogSource {
  id: number;
  title: string;
  username: string | null;
  telegramSourceKind: TelegramSourceKind;
  isMember: boolean;
  photoDataUrl: string | null;
}

export interface Source {
  id: number;
  sourceType: SourceType;
  telegramSourceKind: TelegramSourceKind;
  accountId: number | null;
  externalId: string;
  title: string | null;
  lastSyncState: number | null;
  lastSyncedAt: number | null;
  isMember: boolean;
  isActive: boolean;
  createdAt: number;
  avatarDataUrl: string | null;
}

export interface SourceItem {
  id: number;
  sourceId: number;
  externalId: string;
  author: string | null;
  publishedAt: number;
  content: string | null;
  contentKind: string;
  hasMedia: boolean;
  mediaKind: string | null;
  mediaSummary: string | null;
  mediaFileName: string | null;
  mediaMimeType: string | null;
  hasRawData: boolean;
  forumTopicId: number | null;
  forumTopicTitle: string | null;
  forumTopicTopMessageId: number | null;
}

export type ForumTopicFilter =
  | { kind: "topic"; topicId: number }
  | { kind: "uncategorized" };

export interface SourceForumTopic {
  kind: "topic" | "uncategorized";
  key: string;
  title: string;
  messageCount: number;
  topicId: number | null;
  topMessageId: number | null;
  iconColor: number | null;
  iconEmojiId: number | null;
  isClosed: boolean;
  isPinned: boolean;
  isHidden: boolean;
  isDeleted: boolean;
  sortOrder: number | null;
}

export interface SyncSourceResult {
  inserted: number;
  skipped: number;
  lastMessageId: number | null;
  initialSyncPolicyApplied: string | null;
  warnings: string[];
}

export interface SyncSettings {
  initialSyncMode: InitialSyncMode;
  initialSyncValue: number;
}
```

## Backend Contract

Rust source-domain types should live in `src-tauri/src/sources/types.rs`.

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

New request DTOs:

```rust
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddTelegramSourceRequest {
    pub account_id: i64,
    pub source_ref: String,
    pub expected_kind: Option<TelegramSourceKind>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListSourceItemsRequest {
    pub source_id: i64,
    pub limit: i64,
    pub before_published_at: Option<i64>,
    pub topic_filter: Option<ForumTopicFilter>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveSyncSettingsRequest {
    pub initial_sync_mode: InitialSyncMode,
    pub initial_sync_value: i64,
}
```

Command signatures should become:

```rust
pub async fn add_telegram_source(
    handle: AppHandle,
    state: tauri::State<'_, TelegramState>,
    request: AddTelegramSourceRequest,
) -> AppResult<SourceRecord>

pub async fn list_source_items(
    handle: AppHandle,
    request: ListSourceItemsRequest,
) -> AppResult<Vec<ItemRecord>>

pub async fn save_sync_settings(
    handle: AppHandle,
    settings: SaveSyncSettingsRequest,
) -> AppResult<SyncSettingsRecord>
```

`delete_source`, `list_sources`, `list_telegram_sources`, `sync_source`,
`get_sync_settings`, and `list_source_forum_topics` may keep their current simple argument
shape unless implementation reveals a concrete reason to wrap them.

`ForumTopicFilter` is part of the `list_source_items` request contract and should accept
camelCase topic ids on the Tauri wire:

```rust
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ForumTopicFilter {
    Topic {
        #[serde(rename = "topicId")]
        topic_id: i64,
    },
    Uncategorized,
}
```

This keeps Rust internals on `topic_id` while avoiding a mixed `topicFilter.topic_id` wire shape.

## Error Model

Source command and cross-module service boundaries should return `AppResult<T>` rather than
`Result<T, String>`.

Use explicit kinds:

- `AppError::validation` for unsupported source kinds, malformed manual refs, bad settings,
  invalid external ids, and source-kind mismatches.
- `AppError::not_found` for missing sources, missing Telegram dialogs, and unresolvable peers.
- `AppError::conflict` for same-source active operation conflicts, already handled by
  `SourceIngestLocks`.
- `AppError::network` for Telegram transport, timeout, or connection failures that leave the
  app in a retryable state.
- `AppError::internal` for compression, JSON, filesystem cache, and unexpected storage failures
  that are not user-correctable.

The work should reduce new reliance on `AppError::from(String)` in source modules.

## Rust Module Boundaries

Keep the current module layout and extract only where it reduces coupling:

- `types.rs`: domain enums, public DTOs, DB row structs, shared validation, time helper.
- `store.rs`: source list/add/delete/load command orchestration and source row mapping.
- `items.rs`: source item command orchestration and ingest entrypoint.
- `topics.rs`: topic listing command and forum topic refresh orchestration.
- `peer_resolution.rs`: source peer metadata and Telegram peer resolution.
- `sync.rs`: sync orchestration and sync state finalization.
- `avatar.rs`: avatar download/cache helpers.
- `test_support.rs`: `#[cfg(test)]` SQLite schema helpers shared by source module tests.

Potential targeted extractions:

- `peer_resolution/manual_ref.rs` if manual reference parsing remains noisy after typed errors.
- `peer_resolution/metadata.rs` if metadata compatibility code grows during type tightening.
- `items/query.rs` if `list_source_items` mapping becomes hard to test inside `items.rs`.
- `topics/list.rs` if topic listing and topic refresh continue to obscure each other.

Do not split every large file solely by line count.

## Verification

Required verification:

```powershell
Set-Location src-tauri
cargo test sources --lib
cargo test
Set-Location ..
npm.cmd test
npm.cmd run check
git diff --check
```

Known environment note: previous frontend verification needed to run outside the default sandbox
because Vite/esbuild failed to spawn with `spawn EPERM`.
