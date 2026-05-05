# Analysis Chat Wrapper And Controller

Status: completed and merged into `main`.

## Goal

Centralize Analysis chat frontend command/event access and extract route-level
chat orchestration from `/analysis` while preserving existing behavior.

## Completed Work

- Added a typed Analysis chat Tauri API wrapper.
- Added wrapper contract tests for command names, payload shapes, and event
  subscription behavior.
- Migrated `src/routes/analysis/+page.svelte` away from raw Analysis chat
  command strings, the raw `analysis://chat` listener, and the raw
  `cancel_llm_request` call.
- Added a dependency-injected Analysis chat workflow controller.
- Added workflow behavior tests for loading, asking, cancellation, clearing,
  event handling, stale request guards, and state reset.
- Delegated route-level chat orchestration to the workflow controller while
  keeping Svelte state, modal integration, and listener disposal in the route.
- Refreshed the session handoff documentation.

## Implementation

API wrapper:

```text
src/lib/api/analysis-chat.ts
src/lib/api/analysis-chat.test.ts
```

Workflow controller:

```text
src/lib/analysis-chat-workflow.ts
src/lib/analysis-chat-workflow.test.ts
```

Route integration:

```text
src/routes/analysis/+page.svelte
```

Existing reducer helpers still used by the workflow:

```text
src/lib/analysis-chat-state.ts
src/lib/analysis-chat-state.test.ts
```

## Public Frontend API

`$lib/api/analysis-chat.ts` exports:

```ts
ANALYSIS_CHAT_EVENT = "analysis://chat";
listAnalysisChatMessages;
askAnalysisRunQuestion;
clearAnalysisChatMessages;
listenToAnalysisChatEvents;
```

`$lib/analysis-chat-workflow.ts` exports:

```ts
createAnalysisChatWorkflow(deps): {
  loadMessages(runId, guard?): Promise<void>;
  askRunQuestion(): Promise<void>;
  cancelChat(options?): Promise<void>;
  clearMessages(): Promise<void>;
  clearState(): void;
  handleEvent(payload): void;
}
```

## Verification

Final verification performed before merge:

```powershell
rg -n "analysis://chat|list_analysis_chat_messages|ask_analysis_run_question|clear_analysis_chat_messages|cancel_llm_request" src\routes\analysis\+page.svelte
npm.cmd test -- analysis-chat analysis-chat-workflow analysis-chat-state
npm.cmd test -- analysis-chat analysis-chat-workflow analysis-chat-state analysis-runs llm notebooklm-export takeout-import sources
npm.cmd test
npm.cmd run check
git diff --check
```

Observed results:

```text
route cleanup rg: no matches
3 test files / 23 tests passed
8 test files / 41 tests passed
15 test files / 124 tests passed
svelte-check found 0 errors and 0 warnings
git diff --check exited 0
```

The post-merge verification on `main` also passed:

```text
npm.cmd test: 15 test files / 124 tests passed
```

## Commits

```text
bc636ea docs(analysis): add chat wrapper controller plan
8108b22 refactor(analysis): add chat api wrapper
344f2e0 refactor(analysis): extract chat workflow controller
86ffa78 docs(analysis): record chat controller completion
```

## Scope Preserved

- No Rust backend command or event changes.
- No Analysis chat DTO camelCase migration.
- No chat UI redesign.
- No template, source group, trace, report-run, accounts, sources, Takeout, or
  NotebookLM refactors.
- The backend event name remains `analysis://chat`.
- Existing optimistic pending chat exchange behavior is preserved.
- Saved-message reload after completed chat events is preserved.
- Clear-chat modal copy is preserved.
- Silent cancellation behavior used when switching runs or starting reports is
  preserved.
