# Session Context Handoff - 2026-05-07

## Purpose

This file restores the current Codex session context for Extractum cleanup work.
It supersedes all earlier handoff contents in this file.

The immediate user request that produced this refresh was, in English:

```text
Write all information needed to restore the current session context into docs\session-context-2026-05-03.md. The file may be overwritten. Formulate a commit message.
```

Important: the user asked to write this file and formulate a commit message.
Do not assume they asked to create the commit unless they explicitly confirm it.

## Current Repository State

- Repository root: `G:\Develop\Extractum`.
- Current branch: `main`.
- Working tree before this handoff refresh: clean.
- This handoff refresh modifies only `docs/session-context-2026-05-03.md`.
- Git remotes: none configured.
- Local branches currently known:
  - `main`
  - `desktop-ui`
- Shell: PowerShell on Windows.
- Timezone: `Europe/Minsk`.
- Current date in this session: Thursday, 2026-05-07.
- Network access is restricted.
- Current collaboration mode at the time this file was written: Default mode.

Recent relevant history on `main`:

```text
c64ca8c docs(error): add typed error conversion plan
546a037 docs(session): refresh current cleanup handoff
ad3115f docs(session): refresh report actions post-merge handoff
c4fe2e2 refactor(analysis): use report action workflow
33e53fa refactor(analysis): move report actions into run workflow
30302b7 refactor(analysis): add report action api wrappers
abc570e docs(analysis): add report actions cleanup plan
373352d docs(analysis): add report actions cleanup design
b639057 docs(session): refresh post-merge cleanup handoff
9ab78c8 docs(analysis): refresh source groups cleanup context
39f3cd6 refactor(analysis): use source groups workflow
c8d7921 refactor(analysis): extract source groups workflow
47eec77 refactor(analysis): add source groups api wrapper
02b5010 docs(analysis): add source groups cleanup plan
4c4bbcc docs(analysis): add source groups cleanup design
d0ac8e4 docs(analysis): refresh workspace loading cleanup context
5293634 refactor(analysis): use workspace loading workflow
96a898c refactor(analysis): extract workspace loading workflow
```

## Current User Workflow Rules

- Do not create a git worktree.
- Use a normal branch workflow when implementing work.
- When executing an implementation plan, perform exactly one top-level task per
  user turn, then stop and wait for explicit instruction.
- At the end of each top-level implementation-plan task, create a commit.
- Prefer inline execution for this cleanup series unless the user explicitly
  changes the no-worktree constraint.
- If a workstream is complete and the finishing workflow is invoked, merge
  locally into `main` only if the user chooses that option.
- For this handoff refresh request, do not commit automatically. The requested
  output is the updated file plus a proposed commit message.

## Environment Notes

- Git writes such as `git add`, `git commit`, `git switch`, `git merge`, and
  `git branch -d` often fail in the default Windows sandbox with `.git/*.lock`
  permission errors. Rerunning the same command with approval outside the
  sandbox has worked.
- Frontend verification commands can fail in the default sandbox with
  `spawn EPERM` because Vite, esbuild, or Svelte preprocessing needs child
  processes. Rerunning outside the sandbox has worked.
- Cargo may need network access if dependencies are not already present in the
  local Cargo cache. Network is restricted, so rerun with approval only if a
  required Cargo command fails due to sandbox or network restrictions.
- `git diff --check` runs in the sandbox. LF/CRLF warnings from Git can appear,
  but exit code 0 means whitespace is clean.
- There are no configured Git remotes, so local merge workflows skip `git pull`.

## Active Documentation Policy

- `docs/superpowers/plans` and `docs/superpowers/specs` should contain only
  active future work when practical.
- Historical completed plan/spec files should not be recreated after their
  useful context is consolidated into this handoff and
  `docs/code-review-results-2026-05-03.md`.
- As of this handoff, these files exist:
  - `docs/superpowers/specs/2026-05-07-typed-error-conversion-design.md`
  - `docs/superpowers/plans/2026-05-07-typed-error-conversion.md`
  - `docs/superpowers/specs/2026-05-07-analysis-report-actions-design.md`
  - `docs/superpowers/plans/2026-05-07-analysis-report-actions.md`
- The typed-error files are active.
- The report-actions files are completed historical docs. Removing them is a
  separate docs-only cleanup decision and is not part of the typed-error
  workstream unless the user asks for it.
- The active review follow-up source is
  `docs/code-review-results-2026-05-03.md`.

## Current Active Workstream

Active workstream:

```text
Boundary-first typed error conversion
```

Why it is active:

- `docs/code-review-results-2026-05-03.md` still lists the first recommended
  follow-up as improving typed error conversion for remaining DB, Telegram,
  LLM, and validation paths.
- The user asked how hard that workstream was compared with previous cleanup
  work, agreed with a boundary-first approach, asked to continue planning, and
  then asked to write the plan/spec files.

Current active docs:

- Design/spec:
  `docs/superpowers/specs/2026-05-07-typed-error-conversion-design.md`
- Implementation plan:
  `docs/superpowers/plans/2026-05-07-typed-error-conversion.md`

Already completed for this workstream:

- Task 1, "Persist Active Workstream Docs", was completed and committed:

```text
c64ca8c docs(error): add typed error conversion plan
```

Task 1 created:

- `docs/superpowers/specs/2026-05-07-typed-error-conversion-design.md`
- `docs/superpowers/plans/2026-05-07-typed-error-conversion.md`

Verification performed for Task 1:

```text
rg -n "TBD|TODO|implement later|fill in details|appropriate error handling|add validation|handle edge cases|Similar to Task" docs\superpowers\specs\2026-05-07-typed-error-conversion-design.md docs\superpowers\plans\2026-05-07-typed-error-conversion.md
```

Result: no output, exit code 1.

```text
git diff --check
```

Result: exit code 0.

After the Task 1 commit:

```text
git status --short --branch
## main

git log --oneline -1
c64ca8c docs(error): add typed error conversion plan
```

## Typed Error Workstream Scope

Chosen scope: boundary-first.

Keep:

- existing frontend-facing Tauri error wire shape: `{ kind, message }`;
- existing `AppErrorKind` values:
  - `validation`
  - `not_found`
  - `auth`
  - `network`
  - `conflict`
  - `internal`
- `From<String>` and `classify_message` in `src-tauri/src/error.rs` as
  compatibility fallbacks for untouched legacy paths;
- LLM streamed event payloads as plain text `error: string`.

Do not:

- remove every internal `Result<T, String>`;
- add new error kinds;
- change frontend `src/lib/app-error.ts`;
- refactor source, Takeout import, or NotebookLM export boundaries except if
  shared helper imports require small mechanical changes;
- clean up completed report-actions docs as part of typed-error implementation.

The design adds these explicit helper constructors in `src-tauri/src/error.rs`:

```rust
pub fn database(error: impl std::fmt::Display) -> Self {
    Self::internal(format!("Database error: {error}"))
}

pub fn telegram_network(error: impl std::fmt::Display) -> Self {
    Self::network(format!("Telegram request failed: {error}"))
}

pub fn llm_network(error: impl std::fmt::Display) -> Self {
    Self::network(format!("LLM request failed: {error}"))
}
```

Mapping rules:

- DB/SQL failures: `AppError::database(...)`, kind `internal`.
- Telegram client or transport calls: `AppError::telegram_network(...)`, kind
  `network`.
- LLM provider/model-listing HTTP, client, and timeout failures:
  `AppError::llm_network(...)`, kind `network`.
- User/config/request validation: `AppError::validation(...)`.
- Missing persisted entities: `AppError::not_found(...)`.
- Unauthenticated or not-initialized Telegram runtime:
  `AppError::auth(...)`.
- Active/duplicate/cannot-edit/cannot-delete states:
  `AppError::conflict(...)`.
- Local filesystem/session serialization/unexpected internal failures:
  `AppError::internal(...)`.

## Next Implementation Step

Next top-level task:

```text
Task 2: Add Error Helper Foundation
```

Source of truth:

```text
docs/superpowers/plans/2026-05-07-typed-error-conversion.md
```

Files:

- Modify/test: `src-tauri/src/error.rs`

Task 2 sequence:

1. Add failing tests in `src-tauri/src/error.rs`.
2. Run:

```powershell
cd src-tauri
cargo test error
```

Expected RED: missing associated functions `database`, `telegram_network`, and
`llm_network`.

3. Add the three helper constructors in `impl AppError`.
4. Run:

```powershell
cd src-tauri
cargo test error
```

Expected GREEN.

5. Commit:

```text
refactor(error): add typed conversion helpers
```

Important: execute only Task 2 in the next implementation turn, then stop and
wait for explicit instruction.

## Remaining Typed Error Plan

After Task 2, the current implementation plan continues with:

```text
Task 3: Type Account Database Failures
Commit: refactor(error): type account database failures

Task 4: Type Analysis Validation and Store Boundaries
Commit: refactor(error): type analysis validation failures

Task 5: Type Telegram Failures
Commit: refactor(error): type telegram failures

Task 6: Type LLM Command Failures
Commit: refactor(error): type llm command failures

Task 7: Refresh Review Docs and Session Handoff
Commit: docs(session): refresh typed error cleanup handoff
```

Focused verification commands by task:

```powershell
cd src-tauri
cargo test error
cargo test accounts
cargo test analysis
cargo test telegram
cargo test llm
```

Final verification before typed-error completion:

```powershell
cd src-tauri
cargo test
cd ..
npm.cmd test
npm.cmd run check
git diff --check
```

## Current Review Document State

`docs/code-review-results-2026-05-03.md` currently records:

- Completed:
  - analysis run loading/opening/event orchestration extraction;
  - source API wrapper/facade cleanup;
  - Takeout import API wrapper;
  - NotebookLM export API wrapper;
  - Analysis chat API wrapper and workflow;
  - Analysis trace API wrapper and workflow;
  - Analysis workspace loading API wrapper and workflow;
  - Analysis source group loading and template/group deletion wrapper/workflow;
  - Analysis report start/cancel/delete wrapper/workflow.
- Open Major:
  - `src/routes/analysis/+page.svelte` still owns remaining source group
    create-update actions, template create-update actions, listener lifecycle,
    and UI composition.
- Open Moderate:
  - remaining non-source frontend/backend contracts are manually mirrored.
- Open Moderate:
  - error typing is still partial outside source boundaries.

Recommended follow-up order in that document:

```text
1. Improve typed error conversion for remaining DB, Telegram, LLM, and
   validation paths.
```

Do not mark this finding resolved until the typed-error implementation tasks and
final verification are complete.

## Completed Historical Cleanup Workstreams

These workstreams are complete and should be treated as historical context:

- Analysis run workflow controller extraction.
- Takeout import backend split.
- Sources backend split.
- Source Contract V2 frontend/backend contract cleanup.
- Takeout import frontend API wrapper.
- NotebookLM export frontend API wrapper.
- Analysis chat API wrapper and workflow controller.
- Analysis trace API wrapper and workflow controller.
- Analysis workspace account/status and source metrics API wrapper and
  workflow controller.
- Analysis source groups and template deletion API wrapper and workflow
  controller.
- Analysis report start/cancel/delete API wrapper and workflow controller.

Important completed frontend boundaries:

- `src/lib/api/sources.ts` centralizes core source frontend command access and
  DTO mapping. `get_items` is no longer registered.
- `src/lib/api/takeout-import.ts` owns Takeout import commands and
  `sources://takeout-import`.
- `src/lib/api/notebooklm-export.ts` owns NotebookLM export command access and
  `notebooklm://export`.
- `src/lib/api/analysis-chat.ts` and `src/lib/analysis-chat-workflow.ts`
  centralize Analysis chat command/event access and orchestration.
- `src/lib/api/analysis-trace.ts` and `src/lib/analysis-trace-workflow.ts`
  centralize Analysis trace command access and orchestration.
- `src/lib/api/analysis-workspace.ts` and
  `src/lib/analysis-workspace-workflow.ts` centralize Analysis account/status
  loading and analysis source metrics loading.
- `src/lib/api/analysis-source-groups.ts` and
  `src/lib/analysis-source-groups-workflow.ts` centralize Analysis source group
  loading and template/group deletion orchestration.
- `src/lib/api/analysis-runs.ts` and `src/lib/analysis-run-workflow.ts`
  centralize Analysis run loading, event handling, report start, run cancel,
  and saved-run deletion orchestration.

## Remaining `/analysis` Cleanup Surface

As of this handoff, `src/routes/analysis/+page.svelte` no longer owns raw Tauri
command strings for report start/cancel/delete.

The route still coordinates:

- listener lifecycle;
- remaining template create-update actions;
- remaining source group create-update actions;
- UI composition.

Trace, chat, workspace loading, source group/template deletion, report
start/cancel/delete, Takeout import, NotebookLM export, source facade, and
analysis run workflow extraction are already complete.

## Current IDE Notes

Open tabs reported by the IDE include:

- `docs/code-review-results-2026-05-03.md`

## Verification For This Handoff Refresh

Before editing this file, these context checks were run:

```text
git status --short --branch
## main

git branch --list
  desktop-ui
* main

git remote -v
<no output>

rg --files docs\superpowers\specs docs\superpowers\plans
docs\superpowers\plans\2026-05-07-typed-error-conversion.md
docs\superpowers\specs\2026-05-07-typed-error-conversion-design.md
docs\superpowers\specs\2026-05-07-analysis-report-actions-design.md
docs\superpowers\plans\2026-05-07-analysis-report-actions.md

git show --stat --oneline --decorate -1
c64ca8c (HEAD -> main) docs(error): add typed error conversion plan
```

After writing this file, run at least:

```powershell
git diff --check
git status --short --branch
```

## Suggested Commit Message For This Handoff Refresh

```text
docs(session): refresh typed error planning handoff
```
