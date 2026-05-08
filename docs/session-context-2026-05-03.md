# Session Context Handoff - 2026-05-08

## Purpose

This file is the restoration point for the current Extractum source-provider
readiness session. It is intentionally self-contained so a future Codex session
can resume from this file without reading the full chat transcript.

This file is ASCII-only on purpose. PowerShell output in this workspace has
previously displayed Cyrillic text as mojibake.

## Latest User Request

Current user request, summarized in English:

```text
Overwrite docs/session-context-2026-05-03.md with all information needed to
restore the current session context. The file may be overwritten. Provide a
commit message.
```

Suggested commit message for this handoff-only change:

```text
docs(session): refresh source provider readiness context
```

No commit has been created for this latest handoff refresh unless the current
session explicitly does it after writing this file.

## Repository And Environment

- Repository root: `G:\Develop\Extractum`.
- Shell: PowerShell on Windows.
- Current date from session context: Friday, 2026-05-08.
- Timezone from IDE context: `Europe/Minsk`.
- Network access is restricted.
- Collaboration mode: Default mode.
- Current branch: `source-provider-readiness`.
- Working tree before this handoff refresh: clean.
- Active IDE file: `docs/superpowers/plans/2026-05-07-source-provider-readiness.md`.
- Open tabs in the latest IDE context:
  - `docs/superpowers/plans/2026-05-07-source-provider-readiness.md`
  - `docs/session-context-2026-05-03.md`

Known environment behavior:

- Git writes such as `git add` and `git commit` can fail in the default sandbox
  with `.git/index.lock` permission errors. Rerunning the same git command with
  approval outside the sandbox has worked.
- Vitest can fail in the default sandbox with
  `Cannot read properties of undefined (reading 'config')`. Rerun the same
  focused test command outside the sandbox if that happens.
- Frontend checks can also fail in the default sandbox with child-process
  permission issues. Rerun outside the sandbox when the failure is clearly
  sandbox-related.
- Cargo tests can take longer than 120 seconds on first compile. Use a 300000 ms
  timeout for source/analysis Rust test commands.
- `git diff --check` may print LF/CRLF warnings. Exit code 0 means whitespace is
  clean.

## Active User Instructions

The user explicitly chose this workstream:

```text
Follow docs\superpowers\plans\2026-05-07-source-provider-readiness.md.
Do not create a worktree. Create a branch instead. After each task, commit.
Then wait for my explicit instruction to continue.
```

Operational implications:

- Do not create a git worktree for this plan.
- Continue on branch `source-provider-readiness`.
- Execute exactly one top-level plan task per user "next task" instruction.
- After each completed task, commit that task.
- After each task commit, stop and wait for the user's explicit instruction to
  continue.
- Do not skip ahead to later tasks.
- Do not revert user changes.
- Use `apply_patch` for manual file edits.
- Use `rg` / `rg --files` for search.

Superpowers skills that should be used when relevant:

- At the start of each "next task" turn, read/use:
  - `superpowers:executing-plans`
  - `superpowers:test-driven-development`
- Before claiming completion, before committing, or before saying tests pass,
  read/use:
  - `superpowers:verification-before-completion`
- If a test failure or unexpected behavior occurs, read/use:
  - `superpowers:systematic-debugging`
- `superpowers:using-superpowers` is active as the general skill-discovery rule.

Subagents were not used. Do not use subagents unless the user explicitly asks
for delegation or parallel agent work.

## Current Git State

Latest status before this handoff refresh:

```text
## source-provider-readiness
```

Latest commits before this handoff refresh:

```text
1a4dcb7 (HEAD -> source-provider-readiness) refactor(sources): expose provider subtype
677d561 refactor(analysis): gate source UI by capabilities
079349c refactor(sources): map provider source fields
f95ff80 refactor(sources): add provider capabilities
8527d4e (main) docs(sources): add provider readiness implementation plan
d1bc01e docs(sources): add provider readiness design
21221f3 docs(session): refresh current handoff context
a7e0647 fix(api): align response DTO contracts
```

The plan/design commits are on `main`; implementation task commits are on
`source-provider-readiness`.

## Plan Being Executed

Plan file:

```text
docs/superpowers/plans/2026-05-07-source-provider-readiness.md
```

Goal:

```text
Prepare the shared source model, source UI, backend sync boundary, and analysis
refs for non-Telegram providers without adding YouTube, RSS, or forum ingestion.
```

Architecture summary:

- Keep existing Telegram behavior intact.
- Make the shared frontend/backend source model provider-neutral.
- Add frontend source capabilities and provider labels.
- Add `source_subtype` to backend source records and migrations.
- Dispatch backend sync by provider, with Telegram as the only implemented
  sync provider for now.
- Move new analysis refs to local item identity while preserving legacy Telegram
  message refs.

The plan has 7 tasks:

1. Frontend Source Contract And Capabilities - done.
2. Source API Mapping For Provider Fields - done.
3. Capability-Driven UI And Topic Decisions - done.
4. Backend Source Subtype Persistence - done.
5. Backend Sync Provider Dispatcher - next task.
6. Provider-Neutral Analysis Refs And Prompt Wording - pending.
7. Full Verification And Route Boundary Check - pending.

## Completed Task 1

Commit:

```text
f95ff80 refactor(sources): add provider capabilities
```

Files changed:

- `src/lib/types/sources.ts`
- `src/lib/source-capabilities.ts`
- `src/lib/source-capabilities.test.ts`
- `src/lib/analysis-source-state.ts`
- `src/lib/analysis-source-state.test.ts`

Implemented:

- Added provider-ready frontend source types:
  - `SourceType = "telegram" | "youtube" | "rss" | "forum"`
  - `SourceSubtype`
  - `SourceContentLabel`
  - `SourceCapabilities`
  - `Source.sourceSubtype`
  - nullable `Source.telegramSourceKind`
- Added `src/lib/source-capabilities.ts` for source capability and label logic.
- `analysis-source-state` now delegates provider labels and membership labels to
  source capabilities.
- Sync-disabled reasons are capability-driven.
- Fixtures and tests were updated for nullable Telegram kind and provider
  subtype.

Verification:

- RED: missing `source-capabilities` module and old helper behavior failed.
- GREEN:
  `npm.cmd test -- src/lib/source-capabilities.test.ts src/lib/analysis-source-state.test.ts`
  passed, 2 files and 11 tests.
- `npm.cmd run check`: 0 errors, 0 warnings.
- `git diff --check`: exit code 0 with LF/CRLF warnings only.

## Completed Task 2

Commit:

```text
079349c refactor(sources): map provider source fields
```

Files changed:

- `src/lib/api/sources.ts`
- `src/lib/api/sources.test.ts`

Implemented:

- `RawSource` accepts optional `source_subtype`.
- `RawSource.telegram_source_kind` is optional and nullable.
- `mapSource` maps:
  - `sourceSubtype = source_subtype ?? telegram_source_kind ?? null`
  - `telegramSourceKind = telegram_source_kind ?? null`
- Added a non-Telegram YouTube source mapping test.
- Telegram raw source fixtures now include `source_subtype`.

Verification:

- Initial sandbox Vitest run failed with infrastructure error:
  `Cannot read properties of undefined (reading 'config')`.
- Rerunning outside the sandbox gave the useful RED/GREEN path.
- RED: non-Telegram source produced missing or undefined mapped fields.
- GREEN:
  `npm.cmd test -- src/lib/api/sources.test.ts` passed, 1 file and 6 tests.
- `npm.cmd run check`: 0 errors, 0 warnings.
- `git diff --check`: exit code 0 with LF/CRLF warnings only.

## Completed Task 3

Commit:

```text
677d561 refactor(analysis): gate source UI by capabilities
```

Files changed:

- `src/lib/analysis-state.ts`
- `src/lib/analysis-state.test.ts`
- `src/lib/components/source-row.svelte`
- `src/lib/components/analysis/workspace-rail.svelte`
- `src/lib/components/analysis/workspace-main.svelte`
- `src/routes/analysis/+page.svelte`

Implemented:

- `shouldShowTopicSelector` now takes `Source | null` and uses
  `sourceCapabilities(source).hasTopics` while loading.
- Topic selector tests now use full `Source` fixtures.
- Added a differentiating `forum/thread` case after the first RED case was too
  weak and old logic passed by coincidence.
- `source-row.svelte` imports `membershipLabel`, `sourceCapabilities`, and
  `sourceKindLabel` directly.
- `source-row.svelte` gates membership, sync, and account badges by capabilities.
- `workspace-rail.svelte` gates membership, sync, and Takeout actions by
  capabilities.
- `workspace-main.svelte` imports provider-neutral source labels.
- `src/routes/analysis/+page.svelte` no longer imports or passes source label
  helper props to child components.

Verification:

- RED: `analysis-state` test failed for `forum/thread`, expected true but got
  false.
- GREEN:
  `npm.cmd test -- src/lib/analysis-state.test.ts src/lib/source-capabilities.test.ts src/lib/analysis-source-state.test.ts`
  passed, 3 files and 44 tests.
- `npm.cmd run check`: 0 errors, 0 warnings.
- `git diff --check`: exit code 0 with LF/CRLF warnings only.
- `rg -n "sourceKindLabel|membershipLabel" src\routes\analysis\+page.svelte`:
  no output, exit code 1.

## Completed Task 4

Commit:

```text
1a4dcb7 refactor(sources): expose provider subtype
```

Files changed:

- `src-tauri/migrations/15.sql`
- `src-tauri/src/sources/types.rs`
- `src-tauri/src/sources/test_support.rs`
- `src-tauri/src/sources/store.rs`
- `src-tauri/src/sources/sync.rs`
- `src-tauri/src/sources/peer_resolution.rs`
- `docs/superpowers/plans/2026-05-07-source-provider-readiness.md`

Implemented:

- Added migration `src-tauri/migrations/15.sql`:

```sql
ALTER TABLE sources ADD COLUMN source_subtype TEXT;

UPDATE sources
SET source_subtype = telegram_source_kind
WHERE source_type = 'telegram'
  AND source_subtype IS NULL;
```

- Added backend source provider constants:
  - `YOUTUBE_SOURCE_TYPE`
  - `RSS_SOURCE_TYPE`
  - `FORUM_SOURCE_TYPE`
- Expanded backend `SourceType` enum with:
  - `Youtube`
  - `Rss`
  - `Forum`
- `SourceRecord` now has:
  - `source_subtype: Option<String>`
  - `telegram_source_kind: Option<String>`
- `SourceSyncTarget` now has:
  - `source_subtype: Option<String>`
  - existing `telegram_source_kind: String` kept for the Telegram sync path.
- `SourceRecordRow` now has:
  - `source_subtype: Option<String>`
  - nullable `telegram_source_kind`.
- `load_source` selects `source_subtype` and uses
  `COALESCE(telegram_source_kind, '') AS telegram_source_kind`.
- `add_telegram_source` inserts/updates `source_subtype`.
- Source list queries select `source_subtype`.
- Added `source_record_from_row_parts` helper and a non-Telegram source mapping
  test in backend store tests.
- Updated source test schema for nullable subtype/kind fields.
- Updated sync and peer-resolution fixtures for the new fields.
- `peer_resolution` validation now reads `source_subtype` in the unsupported
  source-type validation message path, avoiding a dead-code warning on the new
  field.
- The plan file was updated to include sync/peer-resolution fixture alignment in
  Task 4.

Important Task 4 note:

- `cargo fmt` briefly reformatted unrelated files:
  - `src-tauri/src/llm/profiles.rs`
  - `src-tauri/src/telegram.rs`
- Those unrelated formatting hunks were manually reverted with `apply_patch`.
- Final Task 4 status included only relevant files.

Verification:

- RED: first `cargo test sources::` timed out at 120 seconds but showed the
  expected compile failure:
  `SourceRecordRow` had no field `source_subtype`.
- GREEN:
  `cargo test sources::` with a 300 second timeout passed, 42 tests.
- No warnings after reading `source_subtype` in the validation path.
- `git diff --cached --check`: exit code 0.
- Working tree was clean after commit.

## Next Task

The next user instruction meaning "next task" should execute Task 5 only:

```text
Task 5: Backend Sync Provider Dispatcher
```

Task 5 files:

- `src-tauri/src/sources/sync.rs`
- `src-tauri/src/sources/types.rs` only if a compile gap appears around
  `SourceSyncTarget`.

Task 5 intended commit message:

```text
refactor(sources): dispatch sync by provider
```

Task 5 expected approach:

1. Use `superpowers:executing-plans` and `superpowers:test-driven-development`.
2. Inspect the current `src-tauri/src/sources/sync.rs` before editing.
3. Add failing dispatcher tests first in the `sync.rs` test module.
4. Run:

```powershell
cd src-tauri
cargo test sources::sync
```

Expected RED:

```text
missing SyncProvider or sync_provider_for_source
```

5. Implement a pure provider dispatch decision in `sync.rs`:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SyncProvider {
    Telegram,
}

fn sync_provider_for_source(source: &SourceSyncTarget) -> AppResult<SyncProvider> {
    match source.source_type.as_str() {
        crate::sources::types::TELEGRAM_SOURCE_TYPE => Ok(SyncProvider::Telegram),
        other => Err(AppError::validation(format!(
            "Source {} with source_type '{}' is not syncable",
            source.id, other
        ))),
    }
}
```

6. In `sync_source`, keep ingest lock acquisition before dispatch.
7. After loading `source`, call `sync_provider_for_source(&source)?`.
8. Dispatch Telegram to a newly extracted `sync_telegram_source(...)`.
9. Extract the previous Telegram-specific body after
   `let source = load_source(&pool, source_id).await?;` into:

```rust
async fn sync_telegram_source(
    handle: AppHandle,
    state: tauri::State<'_, TelegramState>,
    source: SourceSyncTarget,
) -> AppResult<SyncResult> {
    // Existing Telegram sync body goes here.
}
```

10. Run:

```powershell
cd src-tauri
cargo test sources::sync
```

Expected GREEN:

```text
test result: ok.
```

11. Use `superpowers:verification-before-completion`.
12. Run a whitespace check.
13. Update Task 5 checkboxes in the plan file if appropriate.
14. Commit Task 5:

```powershell
git add src-tauri/src/sources/sync.rs src-tauri/src/sources/types.rs docs/superpowers/plans/2026-05-07-source-provider-readiness.md
git commit -m "refactor(sources): dispatch sync by provider"
```

Only include `src-tauri/src/sources/types.rs` if changed.

## Task 5 Test Details

The plan asks to extend the `sync.rs` test module import:

```rust
use super::{determine_sync_policy, finalize_sync, sync_provider_for_source, SyncProvider};
```

Add test for accepted Telegram sources:

```rust
#[test]
fn sync_provider_accepts_telegram_sources() {
    let source = SourceSyncTarget {
        id: 1,
        source_type: TELEGRAM_SOURCE_TYPE.to_string(),
        source_subtype: Some(TELEGRAM_KIND_CHANNEL.to_string()),
        telegram_source_kind: TELEGRAM_KIND_CHANNEL.to_string(),
        account_id: Some(1),
        external_id: "12345".to_string(),
        title: Some("Example".to_string()),
        metadata_zstd: None,
        last_sync_state: None,
    };

    assert_eq!(sync_provider_for_source(&source).unwrap(), SyncProvider::Telegram);
}
```

Add test for rejected manual YouTube video sources:

```rust
#[test]
fn sync_provider_rejects_manual_youtube_video_sources() {
    let source = SourceSyncTarget {
        id: 7,
        source_type: "youtube".to_string(),
        source_subtype: Some("video".to_string()),
        telegram_source_kind: "".to_string(),
        account_id: None,
        external_id: "dQw4w9WgXcQ".to_string(),
        title: Some("Demo video".to_string()),
        metadata_zstd: None,
        last_sync_state: None,
    };

    let error = sync_provider_for_source(&source).expect_err("manual video is not syncable");

    assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
    assert!(error.message.contains("Source 7"));
    assert!(error.message.contains("youtube"));
    assert!(error.message.contains("not syncable"));
}
```

## Current `sync.rs` Shape To Remember

At the end of Task 4, `src-tauri/src/sources/sync.rs` imports:

```rust
use super::types::{now_secs, SourceSyncTarget};
```

Current `sync_source` flow:

1. Get pool with `get_pool(&handle).await?`.
2. Acquire ingest lock.
3. Load source with `load_source(&pool, source_id).await?`.
4. Validate `account_id`.
5. Get Telegram runtime/client.
6. Resolve and refresh peer.
7. Refresh forum topics.
8. Determine sync policy.
9. Persist items.
10. Finalize sync.
11. Return `SyncResult`.

When extracting `sync_telegram_source`, keep `_ingest_guard` in `sync_source`
scope so delete/sync coordination remains unchanged.

Possible compile nuance:

- `tauri::State<'_, TelegramState>` should be passable into the extracted
  function according to the plan.
- If not, inspect the compiler error and make the smallest local adjustment.

## Pending Task 6

Task 6 is:

```text
Provider-Neutral Analysis Refs And Prompt Wording
```

Files:

- `src-tauri/src/analysis/corpus.rs`
- `src-tauri/src/analysis/trace.rs`
- `src-tauri/src/analysis/report.rs`
- `src-tauri/src/analysis/chat.rs`
- `src-tauri/src/analysis/mod.rs`

High-level intent:

- New live corpus refs should use local item ids:
  - new format: `s{source_id}-i{item_id}`
  - legacy format kept: `s{source_id}-m{message_id}`
- `normalize_ref` should accept both `-i` and `-m`.
- Report/chat prompt wording should say "source documents" instead of Telegram
  messages.

Task 6 intended commit message:

```text
refactor(analysis): use provider-neutral refs
```

Do not start Task 6 until Task 5 is committed and the user explicitly says to
continue.

## Pending Task 7

Task 7 is final verification and route boundary check.

Expected checks:

```powershell
npm.cmd test -- src/lib/source-capabilities.test.ts src/lib/analysis-source-state.test.ts src/lib/api/sources.test.ts src/lib/analysis-state.test.ts
npm.cmd run check
cd src-tauri
cargo test sources::
cargo test analysis::
```

Route raw Tauri boundary checks from repo root:

```powershell
rg -n "\binvoke\s*(<|\()|@tauri-apps/api/core" src/routes
rg -n "@tauri-apps/api/event|listen<" src/routes
```

Expected result for both route searches:

```text
no output, exit code 1
```

Whitespace check:

```powershell
git diff --check
```

Task 7 only creates a commit if docs are changed during verification.

## Useful Verification Commands

Frontend focused tests used so far:

```powershell
npm.cmd test -- src/lib/source-capabilities.test.ts src/lib/analysis-source-state.test.ts
npm.cmd test -- src/lib/api/sources.test.ts
npm.cmd test -- src/lib/analysis-state.test.ts src/lib/source-capabilities.test.ts src/lib/analysis-source-state.test.ts
npm.cmd run check
```

Backend focused tests used so far:

```powershell
cd src-tauri
cargo test sources::
```

Git and search checks used so far:

```powershell
git status --short --branch
git log --oneline --decorate -8
git diff --check
git diff --cached --check
rg -n "sourceKindLabel|membershipLabel" src\routes\analysis\+page.svelte
```

## Important Local Files

Plan and context:

- `docs/superpowers/plans/2026-05-07-source-provider-readiness.md`
- `docs/session-context-2026-05-03.md`

Design and plan commits:

- `docs/superpowers/designs/2026-05-07-source-provider-readiness.md`
- `docs/superpowers/plans/2026-05-07-source-provider-readiness.md`

Frontend provider-readiness files:

- `src/lib/types/sources.ts`
- `src/lib/source-capabilities.ts`
- `src/lib/source-capabilities.test.ts`
- `src/lib/analysis-source-state.ts`
- `src/lib/analysis-source-state.test.ts`
- `src/lib/api/sources.ts`
- `src/lib/api/sources.test.ts`
- `src/lib/analysis-state.ts`
- `src/lib/analysis-state.test.ts`
- `src/lib/components/source-row.svelte`
- `src/lib/components/analysis/workspace-rail.svelte`
- `src/lib/components/analysis/workspace-main.svelte`
- `src/routes/analysis/+page.svelte`

Backend provider-readiness files:

- `src-tauri/migrations/15.sql`
- `src-tauri/src/sources/types.rs`
- `src-tauri/src/sources/test_support.rs`
- `src-tauri/src/sources/store.rs`
- `src-tauri/src/sources/sync.rs`
- `src-tauri/src/sources/peer_resolution.rs`
- `src-tauri/src/analysis/corpus.rs`
- `src-tauri/src/analysis/trace.rs`
- `src-tauri/src/analysis/report.rs`
- `src-tauri/src/analysis/chat.rs`
- `src-tauri/src/analysis/mod.rs`

## Handoff Refresh Verification Plan

After overwriting this file, verify:

```powershell
git diff --check
rg -n "[^[:ascii:]]" docs/session-context-2026-05-03.md
git status --short --branch
```

Expected:

- `git diff --check`: exit code 0, LF/CRLF warnings acceptable.
- ASCII check: no output, exit code 1.
- `git status --short --branch`: this file modified, unless committed.

Commit message for this handoff refresh:

```text
docs(session): refresh source provider readiness context
```
