# Sources Contract V2 Completion Record

## Status

Implemented on branch `sources-contract-v2`.

This plan has no remaining implementation tasks. Merge into `main` is still a
separate integration step.

## Outcome

Sources Contract V2 centralizes the core source frontend/backend boundary while
keeping existing SQLite data compatible.

Final frontend source facade:

```text
src/lib/api/sources.ts
```

Facade commands:

```text
get_sync_settings
save_sync_settings
delete_source
list_telegram_sources
add_telegram_source
list_sources
sync_source
list_source_items
list_source_forum_topics
```

Final frontend core source domain types in `src/lib/types/sources.ts`:

```text
TelegramDialogSource
Source
SourceItem
SourceForumTopic
SyncSourceResult
SyncSettings
ForumTopicFilter
```

Backend source module layout:

```text
src-tauri/src/sources/
  mod.rs
  avatar.rs
  items.rs
  items/query.rs
  peer_resolution.rs
  peer_resolution/manual_ref.rs
  settings.rs
  store.rs
  sync.rs
  test_support.rs
  topics.rs
  types.rs
```

## Contract Changes

- Frontend source workflows call `$lib/api/sources` instead of raw core source
  Tauri `invoke(...)` calls.
- UI-facing source objects use camelCase domain fields.
- Raw snake_case source DTO mapping is centralized in `src/lib/api/sources.ts`.
- Backend item listing command is now `list_source_items`; `get_items` is no
  longer registered.
- New source request DTOs use camelCase Tauri wire fields.
- Telegram source-kind validation is centralized in the source domain.
- Source command and service boundaries now use typed `AppError` constructors
  for source-local user-visible failures.
- Source module SQLite test fixtures are shared through
  `src-tauri/src/sources/test_support.rs`.

No SQLite migration was introduced.

## Verification

Latest verification on branch `sources-contract-v2`:

```text
cargo test sources --lib: 41 passed; 0 failed
cargo test: 141 passed; 0 failed
npm.cmd test: 11 test files passed; 102 tests passed
npm.cmd run check: 0 errors; 0 warnings
git diff --check: exit 0
```

Frontend verification commands needed to be rerun outside the sandbox because
Vite/esbuild failed to spawn in the default sandbox with `spawn EPERM`.

## Commit Trail

```text
5ba500e refactor(sources): introduce contract v2 command requests
74512c9 refactor(sources): centralize source kind validation
ad25194 refactor(sources): add typed frontend api facade
07742cd refactor(sources): move ui to source domain types
0cf0ae1 refactor(sources): tighten source error typing
147fcae test(sources): share sqlite fixtures
ca8e6a2 refactor(sources): extract focused source helpers
```

Documentation completion is recorded by:

```text
docs(sources): record contract v2 completion
```

## Deferred Work

- Rust-to-TypeScript type generation.
- Takeout import frontend API wrapper.
- NotebookLM export frontend API wrapper.
- Secure secret storage.
- Full media download/preview.
