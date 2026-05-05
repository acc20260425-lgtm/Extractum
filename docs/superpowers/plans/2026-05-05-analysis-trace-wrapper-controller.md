# Analysis Trace Wrapper And Controller

Status: completed and merged into `main`.

## Goal

Centralize Analysis trace frontend command access and extract route-level trace
orchestration from `/analysis` while preserving existing behavior.

## Completed Work

- Added a typed Analysis trace Tauri API wrapper.
- Added wrapper contract tests for command names and payload shapes.
- Added a dependency-injected Analysis trace workflow controller.
- Added workflow behavior tests for loading, stale guards, clearing, focusing,
  resolving, merge bookkeeping, and error handling.
- Migrated `src/routes/analysis/+page.svelte` away from raw Analysis trace
  command strings.
- Delegated route-level trace orchestration to the workflow controller while
  keeping Svelte state and UI composition in the route.

## Implementation

API wrapper:

```text
src/lib/api/analysis-trace.ts
src/lib/api/analysis-trace.test.ts
```

Workflow controller:

```text
src/lib/analysis-trace-workflow.ts
src/lib/analysis-trace-workflow.test.ts
```

Route integration:

```text
src/routes/analysis/+page.svelte
```

## Public Frontend API

`$lib/api/analysis-trace.ts` exports:

```ts
getAnalysisRunTrace;
resolveAnalysisTraceRefs;
```

`$lib/analysis-trace-workflow.ts` exports:

```ts
createAnalysisTraceWorkflow(deps): {
  loadTrace(runId, guard?): Promise<void>;
  focusTraceRef(ref): Promise<void>;
  clearState(): void;
}
```

## Verification

Final verification performed before completion:

```powershell
rg -n "get_analysis_run_trace|resolve_analysis_trace_refs" src\routes\analysis\+page.svelte
npm.cmd test -- analysis-trace analysis-trace-workflow analysis-state analysis-runs
npm.cmd test
npm.cmd run check
git diff --check
```

Observed results:

```text
route cleanup rg: no matches
focused tests: 4 files passed, 47 tests passed
full frontend tests: 17 files passed, 136 tests passed
svelte-check found 0 errors and 0 warnings
git diff --check exited 0
```

## Scope Preserved

- No Rust backend command changes.
- No Analysis trace DTO camelCase migration.
- No trace UI redesign.
- No analysis run, chat, templates, source group, account, source, Takeout, or
  NotebookLM refactors.
