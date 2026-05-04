# Session Context Handoff - 2026-05-04

## Repository State

- Repository root: `G:\Develop\Extractum`
- Current branch: `sources-contract-v2`
- Worktree before this handoff rewrite: clean
- `main` is currently at `48ef409 docs(sources): add contract v2 refactor plan`
- The user explicitly requested a normal branch, not a git worktree.
- Final requested integration path: after all plan tasks are complete, merge `sources-contract-v2` into `main`.

Important operating constraint:

```text
Execute exactly one top-level task from the plan per user turn, then stop and wait for explicit instruction.
```

The user allowed subagents when using Superpowers. Subagents were used for Tasks 5 and 6 review gates. In both tasks, the implementation worker became unresponsive after making changes; the main session closed the worker, inspected and completed the diff, then ran spec and quality review subagents.

## Active Plan

Plan file:

```text
docs/superpowers/plans/2026-05-03-sources-contract-v2.md
```

Design/spec file:

```text
docs/superpowers/specs/2026-05-03-sources-contract-v2-design.md
```

Review file:

```text
docs/code-review-results-2026-05-03.md
```

Use Superpowers plan execution discipline for remaining work.

Completed top-level tasks:

- Task 1: Backend Command Contract
- Task 2: Rust Source Domain Reuse
- Task 3: Frontend Domain Types And API Wrapper
- Task 4: Frontend Call Site Migration
- Task 5: Backend Typed Errors
- Task 6: Shared Source Test Fixtures

Next top-level task, if the user says to continue:

```text
Task 7: Targeted Rust Extraction
```

Do not start Task 7 until the user explicitly asks to continue.

## Current Branch History

```text
147fcae test(sources): share sqlite fixtures
0cf0ae1 refactor(sources): tighten source error typing
0bb2e20 docs(session): update sources contract v2 handoff
07742cd refactor(sources): move ui to source domain types
ad25194 refactor(sources): add typed frontend api facade
74512c9 refactor(sources): centralize source kind validation
5ba500e refactor(sources): introduce contract v2 command requests
48ef409 docs(sources): add contract v2 refactor plan
```

## Completed Work In This Branch

### Task 1: Backend Command Contract

Commit:

```text
5ba500e refactor(sources): introduce contract v2 command requests
```

Key changes:

- Added typed source contract request DTOs.
- Introduced `TelegramSourceKind`.
- Renamed backend item command from `get_items` to `list_source_items`.
- Registered `list_source_items` in the Tauri handler list.
- Updated source item request payloads to use camelCase Tauri wire fields.
- Added serde coverage for `ForumTopicFilter` with `topicId`.

Verification:

```text
cargo test sources --lib
```

### Task 2: Rust Source Domain Reuse

Commit:

```text
74512c9 refactor(sources): centralize source kind validation
```

Key changes:

- Centralized Telegram source kind parsing/validation in `src-tauri/src/sources/types.rs`.
- Added source kind parse and serde tests.
- Replaced duplicated source kind validation paths in source modules.
- Reused source kind helpers from Takeout-related code where practical.

Verification:

```text
cargo test sources --lib
cargo test takeout --lib
```

### Task 3: Frontend Domain Types And API Wrapper

Commit:

```text
ad25194 refactor(sources): add typed frontend api facade
```

Key changes:

- Added `src/lib/api/sources.ts`.
- Added `src/lib/api/sources.test.ts`.
- Added camelCase frontend source domain types while temporarily keeping old `*Record` exports.
- Centralized raw snake_case source DTO mapping in the frontend facade.

Public facade functions:

```text
listSources
listTelegramSources
addTelegramSource
deleteSource
getSyncSettings
saveSyncSettings
syncSource
listSourceItems
listSourceForumTopics
```

Source command constants in the facade:

```text
list_sources
list_telegram_sources
add_telegram_source
delete_source
get_sync_settings
save_sync_settings
sync_source
list_source_items
list_source_forum_topics
```

Verification:

```text
npm.cmd test -- src/lib/api/sources.test.ts
npm.cmd run check
```

### Task 4: Frontend Call Site Migration

Commit:

```text
07742cd refactor(sources): move ui to source domain types
```

Key changes:

- Migrated `/analysis` source workflows from raw source `invoke(...)` calls to `$lib/api/sources`.
- Migrated source management dialog from raw source `invoke(...)` calls to `$lib/api/sources`.
- Renamed frontend source UI domain usage to camelCase.
- Removed old core compatibility exports from `src/lib/types/sources.ts`:

```text
TelegramSourceInfo
SourceRecord
ItemRecord
SourceForumTopicRecord
SyncResult
SyncSettingsRecord
```

Current frontend core source domain types:

```text
TelegramDialogSource
Source
SourceItem
SourceForumTopic
SyncSourceResult
SyncSettings
ForumTopicFilter
```

Task 4 TDD RED:

```text
npm.cmd test -- src/lib/analysis-state.test.ts src/lib/analysis-source-state.test.ts src/lib/analysis-scope-state.test.ts
```

It failed first for the expected reason: production code and tests still used old snake_case source fields such as `external_id`, `account_id`, and `topic_id`.

Final verification:

```text
npm.cmd test -- src/lib/analysis-state.test.ts src/lib/analysis-source-state.test.ts src/lib/analysis-scope-state.test.ts src/lib/api/sources.test.ts
npm.cmd test
npm.cmd run check
git diff --check
```

Results:

```text
Targeted frontend tests: 4 files passed, 48 tests passed
Full frontend tests: 11 files passed, 102 tests passed
Svelte check: 0 errors, 0 warnings
git diff --check: clean
```

### Task 5: Backend Typed Errors

Commit:

```text
0cf0ae1 refactor(sources): tighten source error typing
```

Key changes:

- Converted source command/service boundary helpers to `AppResult<T>`.
- Replaced classification-sensitive string errors with explicit constructors:
  - `AppError::validation`
  - `AppError::not_found`
  - `AppError::network`
  - `AppError::internal`
- Added focused error classification tests for:
  - unsupported source kind as `Validation`;
  - missing source as `NotFound`;
  - malformed numeric `external_id` as `Validation`;
  - Telegram dialog lookup miss as `NotFound`;
  - metadata decode failure as `Internal`.
- Updated `refresh_forum_topics` warning handling for typed errors.
- After spec review found `build_raw_payload(...) -> Result<Vec<u8>, String>` was still a cross-module boundary, it was converted to `AppResult<Vec<u8>>`.

Verification:

```text
cargo test sources --lib
cargo test
git diff --check
```

Results:

```text
Source tests: 40 passed
Full Rust tests: 140 passed
git diff --check: clean, with only Windows LF-to-CRLF warnings
Spec review: passed after one fix
Code quality review: approved
```

### Task 6: Shared Source Test Fixtures

Commit:

```text
147fcae test(sources): share sqlite fixtures
```

Key changes:

- Added `src-tauri/src/sources/test_support.rs`.
- Added `#[cfg(test)] mod test_support;` to `src-tauri/src/sources/mod.rs`.
- Shared fixture helpers now exist:

```text
memory_pool()
memory_pool_with_sources()
memory_pool_with_source_items_and_topics()
```

- Removed duplicated in-memory SQLite schema setup from source tests in:

```text
src-tauri/src/sources/items.rs
src-tauri/src/sources/topics.rs
src-tauri/src/sources/store.rs
src-tauri/src/sources/sync.rs
src-tauri/src/sources/settings.rs
```

- Added a smoke test confirming the shared source fixture creates expected tables.

Verification:

```text
cargo test sources --lib
git diff --check
```

Results:

```text
Source tests: 41 passed
git diff --check: clean, with only Windows LF-to-CRLF warnings
Spec review: passed
Code quality review: approved
```

## Current Code Shape

Backend source module:

```text
src-tauri/src/sources/
```

Important source files:

```text
avatar.rs
items.rs
mod.rs
peer_resolution.rs
settings.rs
store.rs
sync.rs
test_support.rs
topics.rs
types.rs
```

Frontend source facade:

```text
src/lib/api/sources.ts
```

Frontend source facade tests:

```text
src/lib/api/sources.test.ts
```

Frontend source domain types:

```text
src/lib/types/sources.ts
```

Migrated frontend source UI/state files include:

```text
src/routes/analysis/+page.svelte
src/lib/analysis-state.ts
src/lib/analysis-source-state.ts
src/lib/analysis-scope-state.ts
src/lib/components/analysis/source-management-dialog.svelte
src/lib/components/analysis/workspace-main.svelte
src/lib/components/analysis/workspace-rail.svelte
src/lib/components/analysis/source-context-panel.svelte
src/lib/components/analysis/notebooklm-export-dialog.svelte
src/lib/components/source-messages-panel.svelte
src/lib/components/source-row.svelte
```

`/analysis` now imports source functions from:

```text
$lib/api/sources
```

and uses:

```text
listSources(null)
listSourceForumTopics(sourceId)
listSourceItems({ sourceId, limit: 120, beforePublishedAt: null, topicFilter })
syncSource(sourceId)
deleteSource(source.id)
```

Important remaining raw `invoke(...)` calls:

- Non-source analysis commands remain raw.
- Takeout import frontend calls remain raw.
- NotebookLM export frontend calls remain raw.

These are intentionally outside completed source Contract V2 tasks so far and should not be claimed as complete.

## Current Backend Command Names

Core source Tauri commands currently exposed through `src-tauri/src/lib.rs` and/or `$lib/api/sources`:

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

`get_items` should no longer exist.

## Next Task Details

Next task from the plan:

```text
Task 7: Targeted Rust Extraction
```

Files:

```text
Modify/Create under src-tauri/src/sources/
```

Allowed extractions only:

```text
src-tauri/src/sources/peer_resolution/manual_ref.rs
src-tauri/src/sources/peer_resolution/metadata.rs
src-tauri/src/sources/items/query.rs
src-tauri/src/sources/topics/list.rs
```

Task 7 intent:

- Extract only when it reduces active coupling.
- The new file must own complete behavior with tests and a clear caller.
- Do not create empty pass-through modules.
- Preserve the public/crate-visible facade in `src-tauri/src/sources/mod.rs`.
- Move pure behavior tests beside extracted modules.
- Keep integration-style database tests in orchestration modules unless the extracted module owns the query.
- Run `cargo test sources --lib` after each extraction.

Task 7 commit checkpoint:

```powershell
git add src-tauri/src/sources
git commit -m "refactor(sources): extract focused source helpers"
```

Do not run or implement Task 7 until the user explicitly says to continue.

## Remaining Plan After Task 7

Task 8: Final Verification And Documentation

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

Task 8 documentation targets:

```text
docs/code-review-results-2026-05-03.md
docs/session-context-2026-05-03.md
docs/superpowers/plans/2026-05-03-sources-contract-v2.md
```

Task 8 final commit checkpoint:

```powershell
git add docs src-tauri/src src/lib src/routes
git commit -m "docs(sources): record contract v2 completion"
```

## Known Notes And Constraints

- No SQLite migration should be introduced by this plan.
- No Rust-to-TypeScript type generation is included in this plan.
- Takeout import frontend API wrapper is deferred.
- NotebookLM export frontend API wrapper is deferred.
- Secure secret storage is deferred.
- Full media download/preview is deferred.
- CodeRabbit CLI was unavailable earlier in this environment due `Wsl/Service/E_ACCESSDENIED`; manual review subagents were used instead.
- `npm.cmd test` and `npm.cmd run check` may need sandbox escalation on this machine because Vite/esbuild can fail in the default sandbox with `spawn EPERM`.
- Git writes such as `git add` and `git commit` may need escalation because `.git/index.lock` can be denied by the sandbox on Windows.
- `git diff --check` may print Windows LF-to-CRLF warnings for edited files; previous runs had no real whitespace errors.

## Suggested Commit Message For This Handoff Update

```text
docs(session): refresh sources contract v2 handoff
```
