# Session Context Handoff - 2026-05-03

## Current State

- Repository root: `G:\Develop\Extractum`
- Current branch: `main`
- Working tree was clean before this documentation cleanup.
- The `sources-backend-split` branch was fast-forward merged into `main` and
  deleted locally.
- `main` currently points at:

```text
2f2a667 docs(session): mark sources split docs committed
```

## Completed Work

The sources backend split plan has been implemented and merged.

Completion record:

```text
docs/superpowers/plans/2026-05-03-sources-backend-split.md
```

Earlier completed plan:

```text
docs/superpowers/plans/2026-05-03-takeout-import-backend-split.md
```

Review documentation now records the `sources.rs` split as implemented:

```text
docs/code-review-results-2026-05-03.md
```

## Current Code Shape

- `src-tauri/src/sources.rs` no longer exists.
- `src-tauri/src/sources/mod.rs` is the active facade.
- Sources modules:

```text
avatar
items
peer_resolution
settings
store
sync
topics
types
```

Facade public command/DTO exports:

```text
get_items
ForumTopicFilter
ItemRecord
get_sync_settings
save_sync_settings
InitialSyncMode
SyncSettingsRecord
add_telegram_source
delete_source
list_sources
list_telegram_sources
sync_source
SyncResult
list_source_forum_topics
SourceForumTopicRecord
SourceRecord
TelegramSourceInfo
```

Facade crate-visible Takeout-facing exports:

```text
insert_source_item
SourceItemInsert
TelegramItemContext
resolve_and_refresh_peer
ResolvedSyncPeer
load_source
finalize_sync
SourceSyncTarget
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

Frontend `npm.cmd test` and `npm.cmd run check` needed sandbox escalation
because Vite/esbuild failed to spawn in the default sandbox with `spawn EPERM`;
both passed when rerun outside the sandbox.

## Notes

- CodeRabbit CLI was unavailable earlier in this environment due
  `Wsl/Service/E_ACCESSDENIED`.
- The old transitional `E0761` module ambiguity is resolved.

## Next Step

No active sources split task remains. Suggested follow-up from the review doc:
extract the remaining non-run analysis route controllers/helpers.
