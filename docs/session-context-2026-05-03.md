# Session Context Handoff - 2026-05-05

## Repository State

- Repository root: `G:\Develop\Extractum`
- Current branch: `main`
- Current HEAD:

```text
3f8204b docs(takeout): add frontend wrapper implementation plan
```

- Working tree before this handoff refresh was clean.
- The earlier `sources-contract-v2` branch work has already been integrated
  into `main`.
- The user previously requested a normal branch workflow, not a git worktree.

Important operating constraint from the prior source-contract workstream:

```text
Execute exactly one top-level task from the plan per user turn, then stop and wait for explicit instruction.
```

## Completed Source Contract V2 Work

Primary plan:

```text
docs/superpowers/plans/2026-05-03-sources-contract-v2.md
```

Design/spec:

```text
docs/superpowers/specs/2026-05-03-sources-contract-v2-design.md
```

Review file:

```text
docs/code-review-results-2026-05-03.md
```

Completed top-level tasks:

- Task 1: Backend Command Contract
- Task 2: Rust Source Domain Reuse
- Task 3: Frontend Domain Types And API Wrapper
- Task 4: Frontend Call Site Migration
- Task 5: Backend Typed Errors
- Task 6: Shared Source Test Fixtures
- Task 7: Targeted Rust Extraction
- Task 8: Final Verification And Documentation

Final source facade:

```text
src/lib/api/sources.ts
src/lib/api/sources.test.ts
src/lib/types/sources.ts
```

Core source facade functions:

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

Core source Tauri commands:

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

`get_items` is no longer registered.

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

Current backend source module:

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

Final Source Contract V2 verification recorded in docs:

```text
cargo test sources --lib: 41 passed; 0 failed
cargo test: 141 passed; 0 failed
npm.cmd test: 11 test files passed; 102 tests passed
npm.cmd run check: 0 errors; 0 warnings
git diff --check: exit 0
```

Environment note:

- `npm.cmd test` and `npm.cmd run check` can fail in the default sandbox with
  `spawn EPERM`; previous successful frontend verification reran those commands
  outside the sandbox so Vite/esbuild could spawn child processes.

## Other Completed Plans

Already completed and merged into `main`:

- `docs/superpowers/plans/2026-05-03-analysis-run-workflow-controller.md`
- `docs/superpowers/plans/2026-05-03-takeout-import-backend-split.md`
- `docs/superpowers/plans/2026-05-03-sources-backend-split.md`

Historical note:

- `sources-backend-split` preserved `get_items` at that time.
- `sources-contract-v2` later intentionally replaced it with
  `list_source_items`.

## Current Follow-Up Workstream

Manual review in `docs/code-review-results-2026-05-03.md` left the next
reasonable cleanup direction:

1. Extract remaining non-run analysis route controllers/helpers.
2. Add typed wrappers for Takeout import and NotebookLM export.
3. Improve typed error conversion outside source boundaries.
4. Keep secure secret storage as a separate backlog item.

The user chose to start with the Takeout import frontend wrapper.

Chosen Takeout wrapper scope:

```text
Wrapper only.
```

That means:

- centralize Takeout import command names and event name in `$lib/api`;
- keep existing Takeout DTO field names as snake_case;
- do not migrate Takeout UI/state/components to camelCase;
- do not extract a Takeout workflow controller yet;
- do not change Rust backend commands or events;
- do not include NotebookLM export in this task.

## Takeout Import Wrapper Planning Artifacts

Design/spec:

```text
docs/superpowers/specs/2026-05-05-takeout-import-frontend-wrapper-design.md
```

Implementation plan:

```text
docs/superpowers/plans/2026-05-05-takeout-import-frontend-wrapper.md
```

The plan/spec were committed at:

```text
3f8204b docs(takeout): add frontend wrapper implementation plan
```

## Takeout Wrapper Target Contract

Create:

```text
src/lib/api/takeout-import.ts
src/lib/api/takeout-import.test.ts
```

Expose:

```text
TAKEOUT_IMPORT_EVENT = "sources://takeout-import"
listTakeoutSourceImportJobs
startTakeoutSourceImport
cancelTakeoutSourceImport
listenToTakeoutImportEvents
```

Existing Takeout Tauri commands:

```text
list_takeout_source_import_jobs
start_takeout_source_import
cancel_takeout_source_import
```

Existing Takeout event:

```text
sources://takeout-import
```

Existing Takeout frontend types stay in:

```text
src/lib/types/sources.ts
```

Important Takeout types:

```text
TakeoutImportJobRecord
TakeoutImportEvent
StartTakeoutImportResponse
CancelTakeoutImportResponse
```

Keep existing Takeout DTO fields:

```text
job_id
source_id
account_id
progress_current
progress_total
started_at
finished_at
```

## Takeout Wrapper Implementation Outline

Task 1: Add wrapper tests.

- Create `src/lib/api/takeout-import.test.ts`.
- Mock `@tauri-apps/api/core`.
- Mock `@tauri-apps/api/event`.
- Verify command names, payloads, event constant, and listener forwarding.

Task 2: Add wrapper module.

- Create `src/lib/api/takeout-import.ts`.
- Follow the existing `src/lib/api/analysis-runs.ts` style.

Task 3: Migrate only Takeout calls in the analysis route.

- Modify `src/routes/analysis/+page.svelte`.
- Replace:
  - raw `list_takeout_source_import_jobs` invoke;
  - raw `start_takeout_source_import` invoke;
  - raw `cancel_takeout_source_import` invoke;
  - raw `sources://takeout-import` listener.
- Leave other raw non-Takeout `invoke(...)` and listeners untouched.

Task 4: Verify.

```powershell
npm.cmd test -- takeout-import
npm.cmd test -- analysis-runs sources takeout-import
npm.cmd test
npm.cmd run check
git diff --check
```

If frontend commands fail with `spawn EPERM` in the sandbox, rerun them outside
the sandbox.

Recommended implementation commit message:

```text
refactor(takeout): add frontend api wrapper
```

## Deferred Work

- NotebookLM export frontend API wrapper.
- Takeout import camelCase domain DTO migration.
- Takeout workflow controller extraction from `/analysis`.
- Chat workflow controller/helper extraction from `/analysis`.
- Template and source-group wrappers/controllers.
- Rust-to-TypeScript type generation.
- Secure secret storage.
- Full media download/preview.
- Further typed error conversion outside source boundaries.

## Git Notes

Recent branch history:

```text
3f8204b docs(takeout): add frontend wrapper implementation plan
e3f18ab docs(sources): record contract v2 completion
ca8e6a2 refactor(sources): extract focused source helpers
2516d3b docs(session): refresh sources contract v2 handoff
147fcae test(sources): share sqlite fixtures
0cf0ae1 refactor(sources): tighten source error typing
0bb2e20 docs(session): update sources contract v2 handoff
07742cd refactor(sources): move ui to source domain types
ad25194 refactor(sources): add typed frontend api facade
74512c9 refactor(sources): centralize source kind validation
5ba500e refactor(sources): introduce contract v2 command requests
48ef409 docs(sources): add contract v2 refactor plan
```

Git writes such as `git add` and `git commit` may need escalation because
`.git/index.lock` can be denied by the sandbox on Windows.

Suggested commit message for this session-context refresh:

```text
docs(session): refresh takeout wrapper handoff
```
