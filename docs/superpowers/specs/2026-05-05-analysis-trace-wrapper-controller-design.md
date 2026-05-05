# Analysis Trace Wrapper And Controller Design

## Purpose

Continue the `/analysis` route cleanup by moving Analysis trace frontend command
access and trace orchestration into focused modules.

The route currently owns the raw `get_analysis_run_trace` and
`resolve_analysis_trace_refs` Tauri command names plus the route-level workflow
for loading saved trace refs, resolving missing refs from report/chat clicks,
tracking saved versus resolved refs, and clearing trace state. This work
centralizes the Tauri command boundary in `$lib/api/analysis-trace.ts`, then
extracts the route-level trace workflow into `$lib/analysis-trace-workflow.ts`.

Behavior must stay unchanged. Opening a run with trace data should load saved
trace refs, select the first saved ref, and keep stale open-run responses from
mutating state. Focusing a ref from the report or chat should switch the
inspector to trace mode, avoid duplicate backend resolution when the ref is
already loaded, merge newly resolved refs with existing refs, and preserve the
current user-visible error messages.

## Scope

Included:

- Create `$lib/api/analysis-trace.ts`.
- Add Vitest coverage for the trace API wrapper.
- Replace Analysis trace raw `invoke(...)` calls in
  `src/routes/analysis/+page.svelte`.
- Create `$lib/analysis-trace-workflow.ts`.
- Add Vitest coverage for the trace workflow controller.
- Reuse existing pure helpers from `$lib/analysis-state.ts`, including
  `mergeAnalysisTraceRefs(...)` and `analysisTraceRefOrigin(...)`.

Excluded:

- Rust backend changes.
- Tauri command name changes.
- DTO field renames or camelCase migration.
- Trace UI redesign or component prop redesign.
- Analysis run, chat, templates, source groups, accounts, sources, Takeout, or
  NotebookLM refactors.
- Generated TypeScript types from Rust.

## Frontend API Contract

`src/lib/api/analysis-trace.ts` exposes:

```ts
export function getAnalysisRunTrace(runId: number): Promise<AnalysisTraceData>;

export function resolveAnalysisTraceRefs(
  runId: number,
  refs: string[],
): Promise<AnalysisTraceRef[]>;
```

Wrapped backend command names:

```text
get_analysis_run_trace
resolve_analysis_trace_refs
```

The wrapper is intentionally thin. It should not normalize refs, merge trace
state, handle stale guards, or format errors.

## Workflow Controller Contract

`src/lib/analysis-trace-workflow.ts` exposes:

```ts
export interface AnalysisTraceRequestGuard {
  isCurrent(): boolean;
}

export interface AnalysisTraceWorkflowState {
  currentRun: AnalysisRunDetail | null;
  traceData: AnalysisTraceData;
  savedTraceRefs: string[];
  resolvedTraceRefs: string[];
  selectedTraceRef: string | null;
}

export type AnalysisTraceWorkflowPatch = Partial<{
  inspectorMode: "trace";
  traceData: AnalysisTraceData;
  savedTraceRefs: string[];
  resolvedTraceRefs: string[];
  selectedTraceRef: string | null;
  status: string;
}>;

export interface AnalysisTraceWorkflowDeps {
  getState(): AnalysisTraceWorkflowState;
  patch(patch: AnalysisTraceWorkflowPatch): void;
  getTrace(runId: number): Promise<AnalysisTraceData>;
  resolveRefs(runId: number, refs: string[]): Promise<AnalysisTraceRef[]>;
  formatError(action: string, error: unknown): string;
}

export function createAnalysisTraceWorkflow(
  deps: AnalysisTraceWorkflowDeps,
): {
  loadTrace(runId: number, guard?: AnalysisTraceRequestGuard): Promise<void>;
  focusTraceRef(ref: string): Promise<void>;
  clearState(): void;
};
```

The workflow controller must be dependency-injected. It must not import Svelte,
Tauri APIs, route-local state, or modal helpers.

## Route Migration

The route keeps the Svelte state variables:

```text
traceData
selectedTraceRef
savedTraceRefs
resolvedTraceRefs
inspectorMode
status
```

The route should instantiate `createAnalysisTraceWorkflow(...)` with
`getState()` and `patch(...)`, then delegate:

- `clearTraceState()` to `traceWorkflow.clearState()`;
- `loadTrace(...)` to `traceWorkflow.loadTrace(...)`;
- `focusTraceRef(...)` to `traceWorkflow.focusTraceRef(...)`.

`createAnalysisRunWorkflow(...)` continues to receive `loadTrace` and
`clearTraceState` dependencies, but those route functions should call the trace
workflow methods.

The route may keep derived selectors and UI-only helpers such as
`selectedAnalysisTraceRef(...)` and `traceRefOrigin(...)` unless moving them is
needed to keep the route integration simple.

## Behavior Details

`loadTrace(runId, guard)`:

- toggles no loading flag, matching the current route behavior;
- calls `getTrace(runId)`;
- ignores success and failure when `guard?.isCurrent()` returns false;
- patches `traceData` with the returned trace data;
- sets `savedTraceRefs` from returned refs;
- resets `resolvedTraceRefs` to an empty array;
- selects the first returned ref or `null`;
- on current failure, clears trace state and reports
  `formatError("loading the analysis trace", error)`.

`focusTraceRef(ref)`:

- returns early when there is no current run;
- patches `inspectorMode: "trace"` and `selectedTraceRef: ref`;
- returns without backend work when the ref already exists in `traceData.refs`;
- calls `resolveRefs(currentRun.id, [ref])`;
- merges returned refs with existing refs through `mergeAnalysisTraceRefs(...)`;
- appends newly returned ref ids to `resolvedTraceRefs` without duplicates;
- keeps `selectedTraceRef` pointed at the requested ref;
- on failure, reports `formatError("resolving the trace reference", error)`.

`clearState()` resets:

```ts
{
  traceData: { refs: [] },
  savedTraceRefs: [],
  resolvedTraceRefs: [],
  selectedTraceRef: null,
}
```

## Testing

Wrapper tests verify:

- command names;
- payload shapes for `runId` and `refs`;
- typed return forwarding.

Workflow tests verify:

- load success patches trace data, saved refs, empty resolved refs, and first
  selected ref;
- load success with no refs selects `null`;
- guarded stale load success does not patch state;
- guarded stale load failure does not clear state or patch status;
- current load failure clears trace state and reports formatted status;
- focusing without a current run is a no-op;
- focusing an already loaded ref only switches inspector mode and selection;
- focusing a missing ref resolves through the API, merges refs, records resolved
  ref ids without duplicates, and preserves selection;
- resolve failure reports formatted status.

Required verification:

```powershell
npm.cmd test -- analysis-trace analysis-trace-workflow analysis-state analysis-runs
npm.cmd test
npm.cmd run check
git diff --check
```

Route cleanup check:

```powershell
rg -n "get_analysis_run_trace|resolve_analysis_trace_refs" src\routes\analysis\+page.svelte
```

Expected result:

```text
no matches
```

If Vite, esbuild, or Svelte preprocessing fails with `spawn EPERM` in the
default Windows sandbox, rerun frontend verification outside the sandbox after
approval, matching the existing repository notes.
