# Sources Backend Split Completion Record

## Status

Implemented and merged into `main`.

Final merge state:

```text
2f2a667 docs(session): mark sources split docs committed
```

## Outcome

The former `src-tauri/src/sources.rs` monolith was removed and replaced by a
focused `src-tauri/src/sources/` directory module.

Final module layout:

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

Module ownership:

- `mod.rs`: facade module declarations and public/crate-visible re-exports.
- `types.rs`: shared constants, DTOs, DB row structs, and shared time helper.
- `settings.rs`: sync settings parsing, validation, persistence, and commands.
- `avatar.rs`: Telegram photo and avatar cache helpers.
- `peer_resolution.rs`: source metadata, manual refs, peer resolution, peer
  refs, and avatar refresh during resolution.
- `store.rs`: source CRUD/listing flows and `load_source`.
- `items.rs`: item insert/list behavior and Telegram item payload helpers.
- `topics.rs`: forum topic refresh/list/upsert behavior.
- `sync.rs`: sync orchestration and `finalize_sync`.

## Preserved Contracts

Tauri command names were preserved:

```text
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

Serialized DTO names were preserved:

```text
InitialSyncMode
SyncSettingsRecord
TelegramSourceInfo
SourceRecord
SyncResult
ItemRecord
ForumTopicFilter
SourceForumTopicRecord
```

Takeout-facing crate APIs were preserved:

```text
load_source
resolve_and_refresh_peer
finalize_sync
insert_source_item
SourceSyncTarget
ResolvedSyncPeer
SourceItemInsert
TelegramItemContext
```

## Verification

Latest verification on merged `main`:

```text
cargo test sources --lib: 30 passed; 0 failed
cargo test: 130 passed; 0 failed
npm.cmd test: 10 test files passed; 97 tests passed
npm.cmd run check: 0 errors; 0 warnings
git diff --check: exit 0
```

Note: frontend verification commands needed to be rerun outside the sandbox
because Vite/esbuild failed to spawn in the default sandbox with `spawn EPERM`.

## Commit Trail

```text
2ca3518 refactor(sources): create directory module skeleton
8238db7 refactor(sources): extract sync settings module
ec69c25 refactor(sources): extract avatar and peer resolution modules
fbdc1ad refactor(sources): extract store commands module
b828f74 refactor(sources): extract items module
f476fca refactor(sources): extract forum topics module
0746e1c refactor(sources): extract sync module
7be867f refactor(sources): remove monolith facade
3d7d2aa docs(sources): record backend split completion
2f2a667 docs(session): mark sources split docs committed
```
