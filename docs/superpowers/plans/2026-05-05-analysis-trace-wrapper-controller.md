# Analysis Trace Wrapper And Controller Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Centralize Analysis trace frontend command access and move route-level trace orchestration out of `/analysis` while preserving existing behavior.

**Architecture:** Add a thin `$lib/api/analysis-trace.ts` wrapper for the two trace Tauri commands, then add a dependency-injected `$lib/analysis-trace-workflow.ts` controller for loading, focusing, clearing, stale guard handling, and status patching. Keep Svelte state and UI composition in `src/routes/analysis/+page.svelte`.

**Tech Stack:** Svelte 5 route state, Tauri `invoke`, TypeScript, Vitest, existing `$lib/analysis-state.ts` pure helpers.

---

## Execution Notes

- Do not create a git worktree for this plan.
- Follow the repository's current normal branch workflow.
- Execute one top-level task per user turn, then stop for the next instruction.
- Commit at the end of each top-level task.
- If `npm.cmd test` or `npm.cmd run check` fails with `spawn EPERM` in the
  default Windows sandbox, rerun the same command with approval outside the
  sandbox.

## File Structure

- Create `src/lib/api/analysis-trace.ts`.
  - Owns the raw `get_analysis_run_trace` and `resolve_analysis_trace_refs`
    command strings.
  - Contains no route state logic, no ref merging, no error formatting, and no
    stale guard behavior.
- Create `src/lib/api/analysis-trace.test.ts`.
  - Verifies wrapper command names, payload shapes, and typed return forwarding.
- Create `src/lib/analysis-trace-workflow.ts`.
  - Owns route-level trace behavior through dependency injection.
  - Reuses `mergeAnalysisTraceRefs(...)` from `$lib/analysis-state.ts`.
  - Imports no Svelte, Tauri APIs, modal helpers, or route-local modules.
- Create `src/lib/analysis-trace-workflow.test.ts`.
  - Verifies load, stale guard, clear, focus existing ref, focus missing ref,
    merge bookkeeping, and error behavior.
- Modify `src/routes/analysis/+page.svelte`.
  - Imports the wrapper and workflow.
  - Instantiates `createAnalysisTraceWorkflow(...)`.
  - Delegates `clearTraceState`, `loadTrace`, and `focusTraceRef`.
  - Removes raw trace command strings from the route.
- Update this plan after implementation is complete.
  - Mark status and verification evidence.

## Task 1: Add Analysis Trace API Wrapper

**Files:**

- Create: `src/lib/api/analysis-trace.test.ts`
- Create: `src/lib/api/analysis-trace.ts`

- [ ] **Step 1: Write the failing wrapper tests**

Create `src/lib/api/analysis-trace.test.ts` with:

```ts
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  getAnalysisRunTrace,
  resolveAnalysisTraceRefs,
} from "./analysis-trace";
import type { AnalysisTraceData, AnalysisTraceRef } from "$lib/types/analysis";

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

function traceRef(overrides: Partial<AnalysisTraceRef> = {}): AnalysisTraceRef {
  return {
    ref: "ref-a",
    item_id: 1,
    source_id: 2,
    external_id: "100",
    published_at: 1_700_000,
    excerpt: "Saved excerpt",
    ...overrides,
  };
}

function traceData(overrides: Partial<AnalysisTraceData> = {}): AnalysisTraceData {
  return {
    refs: [traceRef()],
    ...overrides,
  };
}

describe("analysis trace api wrapper", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("loads saved analysis trace data for a run", async () => {
    const data = traceData();
    invokeMock.mockResolvedValueOnce(data);

    await expect(getAnalysisRunTrace(7)).resolves.toEqual(data);

    expect(invokeMock).toHaveBeenLastCalledWith("get_analysis_run_trace", {
      runId: 7,
    });
  });

  it("resolves requested trace refs for a run", async () => {
    const refs = [traceRef({ ref: "ref-b", item_id: 2 })];
    invokeMock.mockResolvedValueOnce(refs);

    await expect(resolveAnalysisTraceRefs(7, ["ref-b"])).resolves.toEqual(refs);

    expect(invokeMock).toHaveBeenLastCalledWith("resolve_analysis_trace_refs", {
      runId: 7,
      refs: ["ref-b"],
    });
  });
});
```

- [ ] **Step 2: Run the wrapper test to confirm RED**

Run:

```powershell
npm.cmd test -- analysis-trace
```

Expected result:

```text
FAIL src/lib/api/analysis-trace.test.ts
Error: Failed to resolve import "./analysis-trace"
```

- [ ] **Step 3: Implement the thin wrapper**

Create `src/lib/api/analysis-trace.ts` with:

```ts
import { invoke } from "@tauri-apps/api/core";
import type { AnalysisTraceData, AnalysisTraceRef } from "$lib/types/analysis";

export function getAnalysisRunTrace(runId: number) {
  return invoke<AnalysisTraceData>("get_analysis_run_trace", { runId });
}

export function resolveAnalysisTraceRefs(runId: number, refs: string[]) {
  return invoke<AnalysisTraceRef[]>("resolve_analysis_trace_refs", { runId, refs });
}
```

- [ ] **Step 4: Run the wrapper test to confirm GREEN**

Run:

```powershell
npm.cmd test -- analysis-trace
```

Expected result:

```text
1 test file passed
2 tests passed
```

- [ ] **Step 5: Commit Task 1**

Run:

```powershell
git add -- src\lib\api\analysis-trace.ts src\lib\api\analysis-trace.test.ts
git commit -m "refactor(analysis): add trace api wrapper"
```

## Task 2: Add Analysis Trace Workflow Controller

**Files:**

- Create: `src/lib/analysis-trace-workflow.test.ts`
- Create: `src/lib/analysis-trace-workflow.ts`

- [ ] **Step 1: Write the failing workflow tests**

Create `src/lib/analysis-trace-workflow.test.ts` with:

```ts
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  createAnalysisTraceWorkflow,
  type AnalysisTraceWorkflowPatch,
  type AnalysisTraceWorkflowState,
} from "./analysis-trace-workflow";
import type {
  AnalysisRunDetail,
  AnalysisRunSummary,
  AnalysisTraceData,
  AnalysisTraceRef,
} from "./types/analysis";

function runSummary(overrides: Partial<AnalysisRunSummary> = {}): AnalysisRunSummary {
  return {
    id: 7,
    run_type: "report",
    scope_type: "single_source",
    source_id: 2,
    source_title: "Source",
    source_group_id: null,
    source_group_name: null,
    scope_label: "Source",
    period_from: 100,
    period_to: 200,
    output_language: "Russian",
    prompt_template_id: 3,
    prompt_template_name: "Template",
    prompt_template_version: 1,
    provider_profile: "default",
    provider: "gemini",
    model: "gemini-2.5-flash",
    status: "completed",
    error: null,
    has_trace_data: true,
    created_at: 100,
    completed_at: 200,
    ...overrides,
  };
}

function runDetail(overrides: Partial<AnalysisRunDetail> = {}): AnalysisRunDetail {
  return {
    ...runSummary(overrides),
    result_markdown: "Saved report",
    ...overrides,
  };
}

function traceRef(overrides: Partial<AnalysisTraceRef> = {}): AnalysisTraceRef {
  return {
    ref: "ref-a",
    item_id: 1,
    source_id: 2,
    external_id: "100",
    published_at: 100,
    excerpt: "Saved excerpt",
    ...overrides,
  };
}

function traceData(refs: AnalysisTraceRef[] = [traceRef()]): AnalysisTraceData {
  return { refs };
}

type HarnessState = AnalysisTraceWorkflowState & {
  inspectorMode: "active" | "history" | "trace" | "chunks";
  status: string;
};

function createHarness(initial: Partial<HarnessState> = {}) {
  const state: HarnessState = {
    currentRun: runDetail(),
    traceData: { refs: [] },
    savedTraceRefs: [],
    resolvedTraceRefs: [],
    selectedTraceRef: null,
    inspectorMode: "history",
    status: "",
    ...initial,
  };

  const deps = {
    getState: () => state,
    patch: vi.fn((patch: AnalysisTraceWorkflowPatch) => Object.assign(state, patch)),
    getTrace: vi.fn(),
    resolveRefs: vi.fn(),
    formatError: vi.fn((action: string, error: unknown) => `Error ${action}: ${String(error)}`),
  };

  const workflow = createAnalysisTraceWorkflow(deps);
  return { state, deps, workflow };
}

describe("analysis-trace-workflow", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it("loads saved trace data and selects the first saved ref", async () => {
    const { state, deps, workflow } = createHarness();
    deps.getTrace.mockResolvedValueOnce(traceData([
      traceRef({ ref: "ref-b", published_at: 200 }),
      traceRef({ ref: "ref-a", published_at: 100 }),
    ]));

    await workflow.loadTrace(7);

    expect(deps.getTrace).toHaveBeenCalledWith(7);
    expect(state.traceData.refs.map((entry) => entry.ref)).toEqual(["ref-b", "ref-a"]);
    expect(state.savedTraceRefs).toEqual(["ref-b", "ref-a"]);
    expect(state.resolvedTraceRefs).toEqual([]);
    expect(state.selectedTraceRef).toBe("ref-b");
  });

  it("selects null when a saved trace load returns no refs", async () => {
    const { state, deps, workflow } = createHarness({
      selectedTraceRef: "old-ref",
    });
    deps.getTrace.mockResolvedValueOnce(traceData([]));

    await workflow.loadTrace(7);

    expect(state.traceData).toEqual({ refs: [] });
    expect(state.savedTraceRefs).toEqual([]);
    expect(state.resolvedTraceRefs).toEqual([]);
    expect(state.selectedTraceRef).toBeNull();
  });

  it("ignores stale guarded trace load success", async () => {
    const existing = traceRef({ ref: "existing" });
    const { state, deps, workflow } = createHarness({
      traceData: traceData([existing]),
      savedTraceRefs: ["existing"],
      selectedTraceRef: "existing",
    });
    deps.getTrace.mockResolvedValueOnce(traceData([traceRef({ ref: "stale" })]));

    await workflow.loadTrace(7, { isCurrent: () => false });

    expect(state.traceData.refs).toEqual([existing]);
    expect(state.savedTraceRefs).toEqual(["existing"]);
    expect(state.selectedTraceRef).toBe("existing");
  });

  it("ignores stale guarded trace load failure", async () => {
    const existing = traceRef({ ref: "existing" });
    const { state, deps, workflow } = createHarness({
      traceData: traceData([existing]),
      savedTraceRefs: ["existing"],
      selectedTraceRef: "existing",
    });
    deps.getTrace.mockRejectedValueOnce("db down");

    await workflow.loadTrace(7, { isCurrent: () => false });

    expect(state.traceData.refs).toEqual([existing]);
    expect(state.savedTraceRefs).toEqual(["existing"]);
    expect(state.status).toBe("");
  });

  it("clears trace state and reports status when current load fails", async () => {
    const { state, deps, workflow } = createHarness({
      traceData: traceData([traceRef({ ref: "existing" })]),
      savedTraceRefs: ["existing"],
      resolvedTraceRefs: ["resolved"],
      selectedTraceRef: "existing",
    });
    deps.getTrace.mockRejectedValueOnce("db down");

    await workflow.loadTrace(7);

    expect(state.traceData).toEqual({ refs: [] });
    expect(state.savedTraceRefs).toEqual([]);
    expect(state.resolvedTraceRefs).toEqual([]);
    expect(state.selectedTraceRef).toBeNull();
    expect(state.status).toBe("Error loading the analysis trace: db down");
  });

  it("does nothing when focusing a ref without a current run", async () => {
    const { state, deps, workflow } = createHarness({ currentRun: null });

    await workflow.focusTraceRef("ref-a");

    expect(deps.resolveRefs).not.toHaveBeenCalled();
    expect(state.inspectorMode).toBe("history");
    expect(state.selectedTraceRef).toBeNull();
  });

  it("selects an already loaded ref without resolving it again", async () => {
    const loaded = traceRef({ ref: "ref-a" });
    const { state, deps, workflow } = createHarness({
      traceData: traceData([loaded]),
    });

    await workflow.focusTraceRef("ref-a");

    expect(deps.resolveRefs).not.toHaveBeenCalled();
    expect(state.inspectorMode).toBe("trace");
    expect(state.selectedTraceRef).toBe("ref-a");
    expect(state.traceData.refs).toEqual([loaded]);
  });

  it("resolves a missing ref, merges it, and records resolved refs without duplicates", async () => {
    const existing = traceRef({ ref: "ref-b", published_at: 200 });
    const resolved = traceRef({ ref: "ref-a", item_id: 2, published_at: 100 });
    const { state, deps, workflow } = createHarness({
      traceData: traceData([existing]),
      savedTraceRefs: ["ref-b"],
      resolvedTraceRefs: ["ref-a"],
    });
    deps.resolveRefs.mockResolvedValueOnce([resolved]);

    await workflow.focusTraceRef("ref-a");

    expect(deps.resolveRefs).toHaveBeenCalledWith(7, ["ref-a"]);
    expect(state.traceData.refs).toEqual([resolved, existing]);
    expect(state.savedTraceRefs).toEqual(["ref-b"]);
    expect(state.resolvedTraceRefs).toEqual(["ref-a"]);
    expect(state.selectedTraceRef).toBe("ref-a");
    expect(state.inspectorMode).toBe("trace");
  });

  it("reports status when resolving a missing ref fails", async () => {
    const { state, deps, workflow } = createHarness();
    deps.resolveRefs.mockRejectedValueOnce("corpus unavailable");

    await workflow.focusTraceRef("ref-a");

    expect(state.status).toBe("Error resolving the trace reference: corpus unavailable");
    expect(state.selectedTraceRef).toBe("ref-a");
    expect(state.inspectorMode).toBe("trace");
  });

  it("clears trace state to the route default values", () => {
    const { state, workflow } = createHarness({
      traceData: traceData([traceRef()]),
      savedTraceRefs: ["ref-a"],
      resolvedTraceRefs: ["ref-b"],
      selectedTraceRef: "ref-a",
    });

    workflow.clearState();

    expect(state.traceData).toEqual({ refs: [] });
    expect(state.savedTraceRefs).toEqual([]);
    expect(state.resolvedTraceRefs).toEqual([]);
    expect(state.selectedTraceRef).toBeNull();
  });
});
```

- [ ] **Step 2: Run the workflow test to confirm RED**

Run:

```powershell
npm.cmd test -- analysis-trace-workflow
```

Expected result:

```text
FAIL src/lib/analysis-trace-workflow.test.ts
Error: Failed to resolve import "./analysis-trace-workflow"
```

- [ ] **Step 3: Implement the workflow controller**

Create `src/lib/analysis-trace-workflow.ts` with:

```ts
import { mergeAnalysisTraceRefs } from "$lib/analysis-state";
import type {
  AnalysisRunDetail,
  AnalysisTraceData,
  AnalysisTraceRef,
} from "$lib/types/analysis";

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

function guardIsCurrent(guard?: AnalysisTraceRequestGuard) {
  return !guard || guard.isCurrent();
}

function emptyTracePatch(): AnalysisTraceWorkflowPatch {
  return {
    traceData: { refs: [] },
    savedTraceRefs: [],
    resolvedTraceRefs: [],
    selectedTraceRef: null,
  };
}

function appendResolvedRefs(currentRefs: string[], nextRefs: AnalysisTraceRef[]) {
  const merged = [...currentRefs];
  for (const nextRef of nextRefs) {
    if (!merged.includes(nextRef.ref)) {
      merged.push(nextRef.ref);
    }
  }
  return merged;
}

export function createAnalysisTraceWorkflow(deps: AnalysisTraceWorkflowDeps) {
  async function loadTrace(runId: number, guard?: AnalysisTraceRequestGuard) {
    try {
      const traceData = await deps.getTrace(runId);
      if (!guardIsCurrent(guard)) {
        return;
      }
      deps.patch({
        traceData,
        savedTraceRefs: traceData.refs.map((ref) => ref.ref),
        resolvedTraceRefs: [],
        selectedTraceRef: traceData.refs[0]?.ref ?? null,
      });
    } catch (error) {
      if (!guardIsCurrent(guard)) {
        return;
      }
      deps.patch({
        ...emptyTracePatch(),
        status: deps.formatError("loading the analysis trace", error),
      });
    }
  }

  async function focusTraceRef(ref: string) {
    const state = deps.getState();
    const run = state.currentRun;
    if (!run) {
      return;
    }

    deps.patch({ inspectorMode: "trace", selectedTraceRef: ref });
    if (state.traceData.refs.some((entry) => entry.ref === ref)) {
      return;
    }

    try {
      const resolved = await deps.resolveRefs(run.id, [ref]);
      const latest = deps.getState();
      deps.patch({
        traceData: {
          refs: mergeAnalysisTraceRefs(latest.traceData.refs, resolved),
        },
        resolvedTraceRefs: appendResolvedRefs(latest.resolvedTraceRefs, resolved),
        selectedTraceRef: ref,
      });
    } catch (error) {
      deps.patch({
        status: deps.formatError("resolving the trace reference", error),
      });
    }
  }

  function clearState() {
    deps.patch(emptyTracePatch());
  }

  return {
    loadTrace,
    focusTraceRef,
    clearState,
  };
}
```

- [ ] **Step 4: Run the workflow test to confirm GREEN**

Run:

```powershell
npm.cmd test -- analysis-trace-workflow
```

Expected result:

```text
1 test file passed
10 tests passed
```

- [ ] **Step 5: Run focused trace-related tests**

Run:

```powershell
npm.cmd test -- analysis-trace analysis-trace-workflow analysis-state analysis-runs
```

Expected result:

```text
test files passed
tests passed
```

The exact file and test counts may differ if nearby tests are added, but the
command must exit 0 with no failed tests.

- [ ] **Step 6: Commit Task 2**

Run:

```powershell
git add -- src\lib\analysis-trace-workflow.ts src\lib\analysis-trace-workflow.test.ts
git commit -m "refactor(analysis): extract trace workflow controller"
```

## Task 3: Migrate The Analysis Route To The Trace Workflow

**Files:**

- Modify: `src/routes/analysis/+page.svelte`

- [ ] **Step 1: Add route imports**

In `src/routes/analysis/+page.svelte`, add wrapper imports near the other
`$lib/api/*` imports:

```ts
import {
  getAnalysisRunTrace,
  resolveAnalysisTraceRefs,
} from "$lib/api/analysis-trace";
```

Add workflow imports near the other workflow imports:

```ts
import {
  createAnalysisTraceWorkflow,
  type AnalysisTraceWorkflowPatch,
} from "$lib/analysis-trace-workflow";
```

- [ ] **Step 2: Add route patch application**

Replace the current direct `clearTraceState()` helper with a patch helper and
delegating clear function:

```ts
function applyTraceWorkflowPatch(patch: AnalysisTraceWorkflowPatch) {
  if ("traceData" in patch) traceData = patch.traceData ?? { refs: [] };
  if ("savedTraceRefs" in patch) savedTraceRefs = patch.savedTraceRefs ?? [];
  if ("resolvedTraceRefs" in patch) resolvedTraceRefs = patch.resolvedTraceRefs ?? [];
  if ("selectedTraceRef" in patch) selectedTraceRef = patch.selectedTraceRef ?? null;
  if ("inspectorMode" in patch && patch.inspectorMode) inspectorMode = patch.inspectorMode;
  if ("status" in patch && patch.status !== undefined) status = patch.status;
}

function clearTraceState() {
  traceWorkflow.clearState();
}
```

Keep the existing `traceRefOrigin(ref)` route helper:

```ts
function traceRefOrigin(ref: string) {
  return traceRefOriginFromState(ref, savedTraceRefs, resolvedTraceRefs);
}
```

- [ ] **Step 3: Instantiate the trace workflow**

Create the workflow after `applyTraceWorkflowPatch(...)` is declared and before
`createAnalysisRunWorkflow(...)` needs `loadTrace` and `clearTraceState`.

```ts
const traceWorkflow = createAnalysisTraceWorkflow({
  getState: () => ({
    currentRun,
    traceData,
    savedTraceRefs,
    resolvedTraceRefs,
    selectedTraceRef,
  }),
  patch: applyTraceWorkflowPatch,
  getTrace: getAnalysisRunTrace,
  resolveRefs: resolveAnalysisTraceRefs,
  formatError: formatAppError,
});
```

- [ ] **Step 4: Replace route trace orchestration with delegation**

Remove the route-local `mergeTraceRefs(...)` helper entirely.

Replace the existing `focusTraceRef(...)` body with:

```ts
async function focusTraceRef(ref: string) {
  await traceWorkflow.focusTraceRef(ref);
}
```

Replace the existing `loadTrace(...)` body with:

```ts
async function loadTrace(runId: number, guard?: AnalysisRunRequestGuard) {
  await traceWorkflow.loadTrace(runId, guard);
}
```

- [ ] **Step 5: Remove now-unused imports**

Remove these imports from `src/routes/analysis/+page.svelte` if TypeScript
reports them unused after the route migration:

```ts
mergeAnalysisTraceRefs,
AnalysisTraceRef,
```

Keep these imports because the route still uses them:

```ts
analysisTraceRefOrigin as traceRefOriginFromState,
selectedAnalysisTraceRef,
AnalysisTraceData,
```

- [ ] **Step 6: Run the route cleanup search**

Run:

```powershell
rg -n "get_analysis_run_trace|resolve_analysis_trace_refs" src\routes\analysis\+page.svelte
```

Expected result:

```text
no matches
```

- [ ] **Step 7: Run focused frontend tests**

Run:

```powershell
npm.cmd test -- analysis-trace analysis-trace-workflow analysis-state analysis-runs
```

Expected result:

```text
test files passed
tests passed
```

The command must exit 0 with no failed tests.

- [ ] **Step 8: Run Svelte check**

Run:

```powershell
npm.cmd run check
```

Expected result:

```text
svelte-check found 0 errors and 0 warnings
```

- [ ] **Step 9: Commit Task 3**

Run:

```powershell
git add -- src\routes\analysis\+page.svelte
git commit -m "refactor(analysis): use trace workflow controller"
```

## Task 4: Final Verification And Plan Closeout

**Files:**

- Modify: `docs/superpowers/plans/2026-05-05-analysis-trace-wrapper-controller.md`

- [ ] **Step 1: Run the final route cleanup check**

Run:

```powershell
rg -n "get_analysis_run_trace|resolve_analysis_trace_refs" src\routes\analysis\+page.svelte
```

Expected result:

```text
no matches
```

- [ ] **Step 2: Run the focused verification command**

Run:

```powershell
npm.cmd test -- analysis-trace analysis-trace-workflow analysis-state analysis-runs
```

Expected result:

```text
test files passed
tests passed
```

The command must exit 0 with no failed tests.

- [ ] **Step 3: Run the full frontend test suite**

Run:

```powershell
npm.cmd test
```

Expected result:

```text
test files passed
tests passed
```

The command must exit 0 with no failed tests.

- [ ] **Step 4: Run Svelte check**

Run:

```powershell
npm.cmd run check
```

Expected result:

```text
svelte-check found 0 errors and 0 warnings
```

- [ ] **Step 5: Run whitespace verification**

Run:

```powershell
git diff --check
```

Expected result:

```text
exit code 0
```

Git may print LF/CRLF warnings on Windows. Treat exit code 0 as clean
whitespace verification.

- [ ] **Step 6: Update this plan with completion notes**

Replace this implementation-plan body with a concise completed-plan summary in
the same style as `docs/superpowers/plans/2026-05-05-analysis-chat-wrapper-controller.md`.

Use this summary structure:

````markdown
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
focused tests: passed
full frontend tests: passed
svelte-check found 0 errors and 0 warnings
git diff --check exited 0
```

## Scope Preserved

- No Rust backend command changes.
- No Analysis trace DTO camelCase migration.
- No trace UI redesign.
- No analysis run, chat, templates, source group, account, source, Takeout, or
  NotebookLM refactors.
````

- [ ] **Step 7: Commit Task 4**

Run:

```powershell
git add -- docs\superpowers\plans\2026-05-05-analysis-trace-wrapper-controller.md
git commit -m "docs(analysis): record trace controller completion"
```
