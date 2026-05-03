# Session Context Handoff - 2026-05-03

## Current State

- Repository root: `G:\Develop\Extractum`
- Current branch: `sources-backend-split`
- `main` currently points at:

```text
7d0d121 docs(sources): add backend split implementation plan
```

- Current branch commits on top of `main`:

```text
b828f74 refactor(sources): extract items module
fbdc1ad refactor(sources): extract store commands module
013a06a docs(session): update sources split handoff context
ec69c25 refactor(sources): extract avatar and peer resolution modules
8238db7 refactor(sources): extract sync settings module
2ca3518 refactor(sources): create directory module skeleton
```

- Working tree was clean before this handoff document rewrite.
- This file is now intentionally updated as the current session handoff.

## User Instructions To Preserve

- Do not use git worktrees.
- Use the normal branch in the current directory.
- Use Superpowers subagents when useful.
- Execute the active implementation plan task-by-task.
- After each plan Task, stop and wait for explicit user instruction before continuing.
- After each Task, provide a commit message.
- Reviews for completed Tasks have used the `superpowers:subagent-driven-development`
  pattern:
  implementer subagent, spec compliance review subagent, code quality review
  subagent, then stop.
- Superpowers worktree advice is intentionally overridden by the user's explicit
  no-worktree instruction.

## Relevant Plans

- Active implementation plan:
  `docs/superpowers/plans/2026-05-03-sources-backend-split.md`
- Completed earlier plan:
  `docs/superpowers/plans/2026-05-03-takeout-import-backend-split.md`

## Completed Sources Split Tasks

### Task 1: Create Directory Module Skeleton

- Created `src-tauri/src/sources/mod.rs` facade with planned module declarations
  and re-exports.
- Created `src-tauri/src/sources/types.rs`.
- Added placeholder module files:
  `avatar.rs`, `items.rs`, `peer_resolution.rs`, `settings.rs`, `store.rs`,
  `sync.rs`, `topics.rs`.
- Moved shared constants and structs from `src-tauri/src/sources.rs` into
  `types.rs`:
  `TELEGRAM_SOURCE_TYPE`, `TELEGRAM_KIND_CHANNEL`, `TELEGRAM_KIND_SUPERGROUP`,
  `TELEGRAM_KIND_GROUP`, `TelegramSourceInfo`, `SourceRecord`,
  `SourceSyncTarget`, `SourceRecordRow`, `StoredItemRow`,
  `SourceForumTopicRow`.
- Spec review passed.
- Code quality review accepted the expected temporary `sources.rs` /
  `sources/mod.rs` module ambiguity and empty-module re-export noise.
- Commit:

```text
2ca3518 refactor(sources): create directory module skeleton
```

### Task 2: Extract Settings

- Moved sync settings constants, DTOs, helpers, pool functions, and Tauri
  commands into `src-tauri/src/sources/settings.rs`.
- Moved settings tests into `settings.rs`:
  `initial_sync_policy_label_formats_messages_and_days`,
  `validate_sync_settings_rejects_out_of_range_values`,
  `sync_settings_default_when_app_settings_are_missing`,
  `sync_settings_roundtrip_through_app_settings`.
- `settings.rs` includes a local memory SQLite helper for the `app_settings`
  table.
- Spec review passed.
- Code quality review passed.
- `cargo test sources::settings --lib` / `cargo check` were blocked only by the
  expected temporary module ambiguity between `src/sources.rs` and
  `src/sources/mod.rs`.
- Commit:

```text
8238db7 refactor(sources): extract sync settings module
```

### Task 3: Extract Avatar And Peer Resolution

- Moved avatar constants/helpers into `src-tauri/src/sources/avatar.rs`.
- Moved peer resolution, metadata encoding/decoding, peer-ref reconstruction,
  source resolution, and avatar refresh helpers into
  `src-tauri/src/sources/peer_resolution.rs`.
- Also moved `telegram_source_info_from_peer`, `telegram_group_kind`, and
  `telegram_group_is_member` into `peer_resolution.rs`.
- Moved 14 peer resolution tests into `peer_resolution.rs`.
- Added transitional wiring in `src-tauri/src/sources.rs` so the old active
  module root can still reference moved Task 3 modules while the split is
  incomplete.
- Updated `finalize_sync_updates_source_state_and_metadata` so it no longer
  constructs private peer metadata structs directly.
- First spec review found wiring/test visibility gaps. The implementer fixed
  them, then repeat spec review passed.
- Code quality review passed.
- `cargo test sources::peer_resolution --lib` and `cargo fmt --check` were
  blocked by the expected temporary `sources.rs` / `sources/mod.rs` ambiguity.
- Commit:

```text
ec69c25 refactor(sources): extract avatar and peer resolution modules
```

### Task 4: Extract Store Commands

- Moved store/source CRUD and listing behavior into
  `src-tauri/src/sources/store.rs`:
  `delete_source`, `list_telegram_sources`, `load_source`,
  `add_telegram_source`, `list_sources`, `source_record_from_row`.
- Moved `load_source_returns_not_found_for_missing_source` into `store.rs`.
- Added a local `memory_pool_with_sources()` helper in `store.rs`.
- Added shared `now_secs()` to `src-tauri/src/sources/types.rs` so extracted
  modules can share time behavior.
- Kept transitional `mod store` and re-exports in `src-tauri/src/sources.rs`.
- Spec review passed.
- Code quality review approved.
- `cargo test sources::store --lib` was blocked by expected `E0761`; follow-on
  Tauri command macro errors were treated as downstream of module ambiguity.
- `git diff --check` passed with only CRLF normalization warnings.
- Commit:

```text
fbdc1ad refactor(sources): extract store commands module
```

### Task 5: Extract Items

- Moved item DTOs and ingest structs into `src-tauri/src/sources/items.rs`:
  `ItemRecord`, `ForumTopicFilter`, `SourceItemInsert`,
  `TelegramItemContext`.
- Moved item insert/list behavior and Telegram item helpers into `items.rs`:
  `insert_source_item`, `get_items`, `load_item_rows_from_pool`,
  `message_author`, `extract_telegram_context`, `reply_peer_context`,
  `build_raw_payload`.
- Moved item tests into `items.rs`:
  `text_roundtrip_through_zstd`,
  `media_metadata_roundtrip_through_zstd`,
  `insert_source_item_writes_payload_and_skips_duplicates`,
  `reply_peer_context_uses_telegram_peer_kinds`,
  `load_item_rows_attaches_topic_metadata_and_root_matches`.
- Kept transitional item re-exports/imports in `src-tauri/src/sources.rs`.
- Sync behavior still lives in `src-tauri/src/sources.rs` and calls moved item
  helpers through `self::items`.
- Spec review passed.
- Code quality review approved.
- `cargo test sources::items --lib` was blocked by expected `E0761`; follow-on
  Tauri command macro errors were treated as downstream of module ambiguity.
- `git diff --check` passed with only CRLF normalization warnings.
- Commit:

```text
b828f74 refactor(sources): extract items module
```

## Current Technical Caveat

The project is intentionally in a transitional split state:

- both `src-tauri/src/sources.rs` and `src-tauri/src/sources/mod.rs` exist;
- Rust reports module ambiguity `E0761` before module-specific tests can run
  normally;
- Tauri command macro lookup errors after `E0761` are expected downstream noise;
- this is expected until the remaining behavior is moved and the old monolith is
  deleted in the later cleanup task.

Do not treat that ambiguity as a new regression during Tasks 6-7 unless it is
accompanied by unrelated compile errors from the module being extracted.

## Current Code Shape

- `src-tauri/src/sources/mod.rs` is already the intended final facade shape and
  re-exports from `items`, `settings`, `store`, `sync`, `topics`, and `types`.
- `src-tauri/src/sources.rs` remains the transitional active monolith for the
  behavior not yet extracted. It currently still owns:
  - sync behavior:
    `SyncResult`, `SyncPolicy`, `IngestOutcome`, `determine_sync_policy`,
    `persist_items`, `sync_source`, `finalize_sync`;
  - forum topic behavior:
    `SourceForumTopicRecord`, `ForumTopicSnapshot`, `refresh_forum_topics`,
    `fetch_all_forum_topics`, `forum_topic_page_cursor`,
    `forum_topic_message_date`, `upsert_forum_topics_from_refresh`,
    `is_non_forum_topic_refresh_error`, `list_source_forum_topics`,
    `list_source_forum_topics_from_pool`;
  - tests for sync and forum topic behavior.
- `src-tauri/src/sources/topics.rs` and `src-tauri/src/sources/sync.rs` still
  need to be filled in later Tasks.

## Next Step

Wait for the user's explicit instruction before continuing.

The next plan item is:

```text
Task 6: Extract Forum Topics
```

Expected ownership for Task 6:

- create/fill `src-tauri/src/sources/topics.rs`;
- remove only forum-topic-related code and topic tests from
  `src-tauri/src/sources.rs`;
- preserve public Tauri command name `list_source_forum_topics`;
- keep `refresh_forum_topics` available to sync via `pub(super)`;
- do not move remaining sync behavior yet.

Expected commit message after Task 6:

```text
refactor(sources): extract forum topics module
```

## Verification Notes

- `git diff --check` has passed after each completed Task, with only CRLF
  normalization warnings.
- CodeRabbit CLI was unavailable earlier in this environment due
  `Wsl/Service/E_ACCESSDENIED`; reviews were performed by read-only subagents.
- Full verification from the plan has not run yet because the split is
  incomplete.
