# Session Context Handoff - 2026-05-04

## Repository State

- Repository root: `G:\Develop\Extractum`
- Current branch: `sources-contract-v2`
- Worktree before updating this handoff file: clean
- User explicitly requested a normal branch, not a git worktree.
- Do not proceed with more than one top-level task from the plan per user turn.
- After each completed top-level task, stop and wait for the user's direct instruction to continue.
- Final requested integration path: after all plan tasks are complete, merge this branch into `main`.

Current branch history:

```text
07742cd refactor(sources): move ui to source domain types
ad25194 refactor(sources): add typed frontend api facade
74512c9 refactor(sources): centralize source kind validation
5ba500e refactor(sources): introduce contract v2 command requests
48ef409 docs(sources): add contract v2 refactor plan
```

`main` is currently at:

```text
48ef409 docs(sources): add contract v2 refactor plan
```

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

Use Superpowers plan execution discipline for this work. The user allowed subagents when using
Superpowers, but the completed tasks so far were done directly in the main session.

Important user constraint:

```text
Execute exactly one top-level task from the plan, then stop and wait for explicit instruction.
```

Completed top-level tasks:

- Task 1: Backend Command Contract
- Task 2: Rust Source Domain Reuse
- Task 3: Frontend Domain Types And API Wrapper
- Task 4: Frontend Call Site Migration

Next top-level task, if the user says to continue:

```text
Task 5: Backend Typed Errors
```

Do not start Task 5 until the user explicitly asks to continue.

## Completed Work In This Branch

### Task 1: Backend Command Contract

Commit:

```text
5ba500e refactor(sources): introduce contract v2 command requests
```

Key changes:

- Added source contract request DTOs.
- Introduced `TelegramSourceKind`.
- Renamed the backend item command from `get_items` to `list_source_items`.
- Registered `list_source_items` in the Tauri handler list.
- Updated source item request payloads to use camelCase Tauri wire fields.
- Added serde coverage for `ForumTopicFilter` with `topicId`.

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

### Task 3: Frontend Domain Types And API Wrapper

Commit:

```text
ad25194 refactor(sources): add typed frontend api facade
```

Key changes:

- Added `src/lib/api/sources.ts`.
- Added `src/lib/api/sources.test.ts`.
- Added camelCase frontend source domain types while temporarily keeping old `*Record` types.
- Centralized raw snake_case source DTO mapping in the frontend facade.
- Public facade functions include:

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

Important remaining raw `invoke(...)` calls:

- Non-source analysis commands remain raw.
- Takeout import frontend calls remain raw.
- NotebookLM export frontend calls remain raw.

These are intentionally outside Tasks 1-4 and should not be claimed as complete.

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

## Verification So Far

Task 1 verification:

```text
cargo test sources --lib
```

Task 2 verification:

```text
cargo test sources --lib
cargo test takeout --lib
```

Task 3 verification:

```text
npm.cmd test -- src/lib/api/sources.test.ts
npm.cmd run check
```

Task 4 TDD RED:

```text
npm.cmd test -- src/lib/analysis-state.test.ts src/lib/analysis-source-state.test.ts src/lib/analysis-scope-state.test.ts
```

This failed first for the expected reason: production code and tests still used old snake_case
source fields such as `external_id`, `account_id`, and `topic_id`.

Task 4 final verification:

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

`npm.cmd test` and `npm.cmd run check` may need sandbox escalation on this machine because
Vite/esbuild can fail in the default sandbox with `spawn EPERM`.

Git writes such as `git add` may also need escalation because `.git/index.lock` can be denied by
the sandbox on Windows.

## Next Task Details

Next task from the plan:

```text
Task 5: Backend Typed Errors
```

Files listed by the plan:

```text
src-tauri/src/sources/items.rs
src-tauri/src/sources/store.rs
src-tauri/src/sources/settings.rs
src-tauri/src/sources/peer_resolution.rs
src-tauri/src/sources/topics.rs
src-tauri/src/sources/sync.rs
src-tauri/src/sources/avatar.rs
```

Task 5 intent:

- Convert source command and service boundaries to `AppResult<T>`.
- Replace classification-sensitive string errors with explicit `AppError` constructors.
- Add or update focused error tests.
- Verify with:

```powershell
Set-Location src-tauri
cargo test sources --lib
cargo test
```

Task 5 commit checkpoint:

```powershell
git add src-tauri/src/sources src-tauri/src/error.rs
git commit -m "refactor(sources): tighten source error typing"
```

Do not run or implement Task 5 until the user explicitly says to continue.

## Known Notes And Constraints

- No SQLite migration should be introduced by this plan.
- No Rust-to-TypeScript type generation is included in this plan.
- Takeout import frontend API wrapper is deferred.
- NotebookLM export frontend API wrapper is deferred.
- Secure secret storage is deferred.
- Full media download/preview is deferred.
- CodeRabbit CLI was unavailable earlier in this environment due `Wsl/Service/E_ACCESSDENIED`.

## Suggested Commit Message For This Handoff Update

```text
docs(session): update sources contract v2 handoff
```
