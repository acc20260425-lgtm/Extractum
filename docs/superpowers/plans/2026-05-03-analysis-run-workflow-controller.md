# Analysis Run Workflow Controller Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract analysis run loading, opening, and run-event orchestration from `src/routes/analysis/+page.svelte` into a tested route-local workflow controller.

**Architecture:** Add a typed analysis-run API wrapper for the Tauri command/event boundary, then add a plain TypeScript workflow factory that receives a state adapter and callbacks from the route. The route keeps Svelte `$state`, listener lifecycle, and UI composition; the controller owns async run workflow sequencing, stale-result guards, loading flags, status updates, and terminal run-event refresh behavior.

**Tech Stack:** Svelte 5, SvelteKit, TypeScript, Tauri v2 API, Vitest.

---

## File Structure

- Create `src/lib/api/analysis-runs.ts`: typed wrappers for analysis run Tauri commands and the `analysis://run` event name.
- Create `src/lib/api/analysis-runs.test.ts`: Vitest coverage for command names, argument shapes, and listener forwarding.
- Create `src/lib/analysis-run-workflow.ts`: workflow factory and types for route-owned run state orchestration.
- Create `src/lib/analysis-run-workflow.test.ts`: Vitest coverage for `loadRuns`, `loadActiveRuns`, `openRun`, and `handleRunEvent`.
- Modify `src/routes/analysis/+page.svelte`: instantiate the controller, delegate run workflow functions, and switch the run listener to the typed wrapper.

Do not modify backend Rust code. Do not extract chat, NotebookLM, Takeout, source management, report start, or run deletion workflows in this plan.

## Task 1: Typed Analysis Run API Wrapper

**Files:**
- Create: `src/lib/api/analysis-runs.ts`
- Create: `src/lib/api/analysis-runs.test.ts`

- [ ] **Step 1: Write the failing wrapper tests**

Create `src/lib/api/analysis-runs.test.ts`:

```ts
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  ANALYSIS_RUN_EVENT,
  getAnalysisRun,
  listActiveAnalysisRuns,
  listAnalysisRuns,
  listenToAnalysisRunEvents,
} from "./analysis-runs";
import type { AnalysisRunEvent } from "$lib/types/analysis";

const invokeMock = vi.hoisted(() => vi.fn());
const listenMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: listenMock,
}));

describe("analysis run api wrappers", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    listenMock.mockReset();
  });

  it("wraps analysis run list commands with typed arguments", async () => {
    invokeMock.mockResolvedValueOnce([]);
    await expect(listAnalysisRuns({
      sourceId: 7,
      sourceGroupId: null,
      limit: 50,
    })).resolves.toEqual([]);
    expect(invokeMock).toHaveBeenLastCalledWith("list_analysis_runs", {
      sourceId: 7,
      sourceGroupId: null,
      limit: 50,
    });

    invokeMock.mockResolvedValueOnce([]);
    await expect(listActiveAnalysisRuns()).resolves.toEqual([]);
    expect(invokeMock).toHaveBeenLastCalledWith("list_active_analysis_runs");
  });

  it("wraps analysis run detail loading", async () => {
    invokeMock.mockResolvedValueOnce(null);

    await expect(getAnalysisRun(42)).resolves.toBeNull();
    expect(invokeMock).toHaveBeenLastCalledWith("get_analysis_run", { runId: 42 });
  });

  it("listens on the shared analysis run event name", async () => {
    const unlisten = vi.fn();
    const handler = vi.fn();
    listenMock.mockResolvedValueOnce(unlisten);

    await expect(listenToAnalysisRunEvents(handler)).resolves.toBe(unlisten);
    expect(ANALYSIS_RUN_EVENT).toBe("analysis://run");
    expect(listenMock).toHaveBeenCalledWith(ANALYSIS_RUN_EVENT, expect.any(Function));

    const payload: AnalysisRunEvent = {
      run_id: 7,
      request_id: null,
      kind: "progress",
      phase: "map",
      queue_position: null,
      message: "Mapping",
      progress_current: null,
      progress_total: null,
      delta: null,
      chunk_summary: null,
      error: null,
    };
    const event = { payload };
    listenMock.mock.calls[0][1](event);
    expect(handler).toHaveBeenCalledWith(event);
  });
});
```

- [ ] **Step 2: Run the wrapper tests to confirm RED**

Run:

```powershell
npm.cmd test -- src/lib/api/analysis-runs.test.ts
```

Expected: FAIL because `src/lib/api/analysis-runs.ts` does not exist.

- [ ] **Step 3: Implement the wrapper**

Create `src/lib/api/analysis-runs.ts`:

```ts
import { invoke } from "@tauri-apps/api/core";
import { listen, type Event, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  AnalysisRunDetail,
  AnalysisRunEvent,
  AnalysisRunSummary,
  EventEnvelope,
} from "$lib/types/analysis";

export const ANALYSIS_RUN_EVENT = "analysis://run";

export interface ListAnalysisRunsInput {
  sourceId: number | null;
  sourceGroupId: number | null;
  limit: number;
}

export function listAnalysisRuns(input: ListAnalysisRunsInput) {
  return invoke<AnalysisRunSummary[]>("list_analysis_runs", { ...input });
}

export function listActiveAnalysisRuns() {
  return invoke<AnalysisRunSummary[]>("list_active_analysis_runs");
}

export function getAnalysisRun(runId: number) {
  return invoke<AnalysisRunDetail | null>("get_analysis_run", { runId });
}

export function listenToAnalysisRunEvents(
  handler: (event: Event<AnalysisRunEvent>) => void,
): Promise<UnlistenFn> {
  return listen<AnalysisRunEvent>(
    ANALYSIS_RUN_EVENT,
    (event: EventEnvelope<AnalysisRunEvent> & Event<AnalysisRunEvent>) => handler(event),
  );
}
```

- [ ] **Step 4: Run the wrapper tests to confirm GREEN**

Run:

```powershell
npm.cmd test -- src/lib/api/analysis-runs.test.ts
```

Expected: PASS with 3 tests.

- [ ] **Step 5: Commit**

Run:

```powershell
git add src/lib/api/analysis-runs.ts src/lib/api/analysis-runs.test.ts
git commit -m "test(frontend): add analysis run api wrapper"
```

## Task 2: Workflow Types And Run Loading Workflows

**Files:**
- Create: `src/lib/analysis-run-workflow.ts`
- Create: `src/lib/analysis-run-workflow.test.ts`

- [ ] **Step 1: Write failing tests for `loadRuns` and `loadActiveRuns`**

Create `src/lib/analysis-run-workflow.test.ts` with the shared test harness and the first tests:

```ts
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  createAnalysisRunWorkflow,
  type AnalysisRunWorkflowPatch,
  type AnalysisRunWorkflowState,
} from "./analysis-run-workflow";
import type { AnalysisHistoryScopeParams } from "./analysis-scope-state";
import type {
  AnalysisRunDetail,
  AnalysisRunEvent,
  AnalysisRunSummary,
  AnalysisTraceData,
} from "./types/analysis";

function runSummary(overrides: Partial<AnalysisRunSummary> = {}): AnalysisRunSummary {
  return {
    id: 1,
    run_type: "daily",
    scope_type: "single_source",
    source_id: 2,
    source_title: "Source",
    source_group_id: null,
    source_group_name: null,
    scope_label: "Source",
    period_from: 100,
    period_to: 200,
    output_language: "en",
    prompt_template_id: 3,
    prompt_template_name: "Template",
    prompt_template_version: 1,
    provider_profile: "default",
    provider: "gemini",
    model: "gemini-2.5-flash",
    status: "completed",
    error: null,
    has_trace_data: false,
    created_at: 100,
    completed_at: 200,
    ...overrides,
  };
}

function runDetail(overrides: Partial<AnalysisRunDetail> = {}): AnalysisRunDetail {
  return {
    ...runSummary(overrides),
    result_markdown: "saved result",
    ...overrides,
  };
}

function runEvent(overrides: Partial<AnalysisRunEvent> = {}): AnalysisRunEvent {
  return {
    run_id: 7,
    request_id: null,
    kind: "progress",
    phase: "map",
    queue_position: null,
    message: null,
    progress_current: null,
    progress_total: null,
    delta: null,
    chunk_summary: null,
    error: null,
    ...overrides,
  };
}

function createHarness(initial: Partial<AnalysisRunWorkflowState> = {}) {
  const state: AnalysisRunWorkflowState & {
    runs: AnalysisRunSummary[];
    activeRuns: AnalysisRunSummary[];
    loadingRuns: boolean;
    loadingActiveRuns: boolean;
    loadingRunDetail: boolean;
    inspectorMode: "active" | "history" | "trace" | "chunks";
    status: string;
  } = {
    historyScopeParams: { sourceId: null, sourceGroupId: null },
    activeRunId: null,
    currentRun: null,
    activeChatRequestId: null,
    activeChatRunId: null,
    runs: [],
    activeRuns: [],
    loadingRuns: false,
    loadingActiveRuns: false,
    loadingRunDetail: false,
    inspectorMode: "history",
    status: "",
    ...initial,
  };

  const deps = {
    getState: () => state,
    patch: vi.fn((patch: AnalysisRunWorkflowPatch) => Object.assign(state, patch)),
    listRuns: vi.fn(),
    listActiveRuns: vi.fn(),
    getRun: vi.fn(),
    syncRunSnapshot: vi.fn(),
    pruneLiveRuns: vi.fn(),
    applyRunEvent: vi.fn(),
    cancelChatSilently: vi.fn(),
    clearChatState: vi.fn(),
    loadChatMessages: vi.fn(),
    loadTrace: vi.fn(),
    clearTraceState: vi.fn(),
    formatError: vi.fn((action: string, error: unknown) => `Error ${action}: ${String(error)}`),
  };

  const workflow = createAnalysisRunWorkflow(deps);
  return { state, deps, workflow };
}

describe("analysis-run-workflow", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it("clears saved runs without loading when history scope is unavailable", async () => {
    const { state, deps, workflow } = createHarness({
      historyScopeParams: null,
      runs: [runSummary({ id: 1 })],
    });

    await workflow.loadRuns();

    expect(state.runs).toEqual([]);
    expect(deps.listRuns).not.toHaveBeenCalled();
    expect(state.loadingRuns).toBe(false);
  });

  it("loads saved runs for the current scope while excluding active statuses", async () => {
    const params: AnalysisHistoryScopeParams = { sourceId: 7, sourceGroupId: null };
    const { state, deps, workflow } = createHarness({ historyScopeParams: params });
    deps.listRuns.mockResolvedValueOnce([
      runSummary({ id: 1, status: "completed" }),
      runSummary({ id: 2, status: "running" }),
      runSummary({ id: 3, status: "queued" }),
      runSummary({ id: 4, status: "failed" }),
    ]);

    await workflow.loadRuns();

    expect(deps.listRuns).toHaveBeenCalledWith({
      sourceId: 7,
      sourceGroupId: null,
      limit: 50,
    });
    expect(state.runs.map((run) => run.id)).toEqual([1, 4]);
    expect(state.loadingRuns).toBe(false);
  });

  it("sets status and clears loading state when saved run loading fails", async () => {
    const { state, deps, workflow } = createHarness();
    deps.listRuns.mockRejectedValueOnce("db down");

    await workflow.loadRuns();

    expect(state.status).toBe("Error loading analysis runs: db down");
    expect(state.loadingRuns).toBe(false);
  });

  it("loads active runs and syncs live snapshots", async () => {
    const { state, deps, workflow } = createHarness({ activeRunId: 7 });
    deps.listActiveRuns.mockResolvedValueOnce([
      runSummary({ id: 7, status: "running" }),
      runSummary({ id: 8, status: "queued" }),
    ]);

    await workflow.loadActiveRuns();

    expect(state.activeRuns.map((run) => run.id)).toEqual([7, 8]);
    expect(deps.syncRunSnapshot).toHaveBeenCalledWith(7, "running");
    expect(deps.syncRunSnapshot).toHaveBeenCalledWith(8, "queued");
    expect(deps.pruneLiveRuns).toHaveBeenCalledWith([7, 8], null);
    expect(state.activeRunId).toBe(7);
    expect(state.loadingActiveRuns).toBe(false);
  });

  it("auto-opens the first active run when the selected active id is stale", async () => {
    const { state, deps, workflow } = createHarness({ activeRunId: 99 });
    deps.listActiveRuns.mockResolvedValueOnce([runSummary({ id: 7, status: "running" })]);
    deps.getRun.mockResolvedValueOnce(runDetail({ id: 7, status: "running" }));
    deps.loadChatMessages.mockResolvedValueOnce(undefined);

    await workflow.loadActiveRuns();

    expect(state.activeRunId).toBe(7);
    expect(deps.getRun).toHaveBeenCalledWith(7);
  });
});
```

- [ ] **Step 2: Run the workflow tests to confirm RED**

Run:

```powershell
npm.cmd test -- src/lib/analysis-run-workflow.test.ts
```

Expected: FAIL because `src/lib/analysis-run-workflow.ts` does not exist.

- [ ] **Step 3: Implement workflow types, factory, `loadRuns`, and `loadActiveRuns`**

Create `src/lib/analysis-run-workflow.ts`:

```ts
import {
  activeRunSyncDecision,
  isActiveRunStatus,
} from "$lib/analysis-state";
import type { AnalysisHistoryScopeParams } from "$lib/analysis-scope-state";
import type {
  AnalysisRunDetail,
  AnalysisRunEvent,
  AnalysisRunSummary,
} from "$lib/types/analysis";
import type { ListAnalysisRunsInput } from "$lib/api/analysis-runs";

export type AnalysisRunInspectorMode = "active" | "history" | "trace" | "chunks";

export interface AnalysisRunWorkflowState {
  historyScopeParams: AnalysisHistoryScopeParams | null;
  activeRunId: number | null;
  currentRun: AnalysisRunDetail | null;
  activeChatRequestId: string | null;
  activeChatRunId: number | null;
}

export interface AnalysisRunRequestGuard {
  isCurrent(): boolean;
}

export type AnalysisRunWorkflowPatch = Partial<{
  runs: AnalysisRunSummary[];
  activeRuns: AnalysisRunSummary[];
  activeRunId: number | null;
  currentRun: AnalysisRunDetail | null;
  inspectorMode: AnalysisRunInspectorMode;
  loadingRuns: boolean;
  loadingActiveRuns: boolean;
  loadingRunDetail: boolean;
  status: string;
}>;

export interface AnalysisRunWorkflowDeps {
  getState(): AnalysisRunWorkflowState;
  patch(patch: AnalysisRunWorkflowPatch): void;
  listRuns(input: ListAnalysisRunsInput): Promise<AnalysisRunSummary[]>;
  listActiveRuns(): Promise<AnalysisRunSummary[]>;
  getRun(runId: number): Promise<AnalysisRunDetail | null>;
  syncRunSnapshot(runId: number, runStatus: string): void;
  pruneLiveRuns(activeRunIds: number[], preserveRunId: number | null): void;
  applyRunEvent(payload: AnalysisRunEvent): void;
  cancelChatSilently(): Promise<void>;
  clearChatState(): void;
  loadChatMessages(runId: number, guard?: AnalysisRunRequestGuard): Promise<void>;
  loadTrace(runId: number, guard?: AnalysisRunRequestGuard): Promise<void>;
  clearTraceState(): void;
  formatError(action: string, error: unknown): string;
}

export function createAnalysisRunWorkflow(deps: AnalysisRunWorkflowDeps) {
  let openRunRequestToken = 0;

  function createGuard(token: number): AnalysisRunRequestGuard {
    return {
      isCurrent: () => token === openRunRequestToken,
    };
  }

  async function loadRuns() {
    const params = deps.getState().historyScopeParams;
    if (params === null) {
      deps.patch({ runs: [] });
      return;
    }

    deps.patch({ loadingRuns: true });
    try {
      const summaries = await deps.listRuns({
        sourceId: params.sourceId,
        sourceGroupId: params.sourceGroupId,
        limit: 50,
      });
      deps.patch({ runs: summaries.filter((run) => !isActiveRunStatus(run.status)) });
    } catch (error) {
      deps.patch({ status: deps.formatError("loading analysis runs", error) });
    } finally {
      deps.patch({ loadingRuns: false });
    }
  }

  function syncActiveRunState(summaries: AnalysisRunSummary[]) {
    const state = deps.getState();
    const decision = activeRunSyncDecision(
      summaries,
      state.activeRunId,
      state.currentRun?.id ?? null,
    );

    for (const snapshot of decision.runSnapshots) {
      deps.syncRunSnapshot(snapshot.runId, snapshot.status);
    }

    deps.pruneLiveRuns(decision.activeRunIds, decision.preserveRunId);

    if (decision.runToOpen !== null) {
      void openRun(decision.runToOpen);
      return;
    }

    deps.patch({ activeRunId: decision.nextActiveRunId });
  }

  async function loadActiveRuns() {
    deps.patch({ loadingActiveRuns: true });
    try {
      const summaries = await deps.listActiveRuns();
      deps.patch({ activeRuns: summaries });
      syncActiveRunState(summaries);
    } catch (error) {
      deps.patch({ status: deps.formatError("loading active analysis runs", error) });
    } finally {
      deps.patch({ loadingActiveRuns: false });
    }
  }

  async function openRun(runId: number) {
    const requestToken = ++openRunRequestToken;
    const guard = createGuard(requestToken);
    deps.patch({ inspectorMode: "history" });

    const state = deps.getState();
    if (
      state.activeChatRequestId !== null &&
      state.activeChatRunId !== null &&
      state.activeChatRunId !== runId
    ) {
      await deps.cancelChatSilently();
      deps.clearChatState();
    }

    deps.patch({ activeRunId: runId, loadingRunDetail: true });
    try {
      const run = await deps.getRun(runId);
      if (!guard.isCurrent()) {
        return;
      }

      if (!run) {
        const currentRun = deps.getState().currentRun;
        deps.patch({
          status: `Analysis run ${runId} was not found.`,
          currentRun: currentRun?.id === runId ? null : currentRun,
        });
        return;
      }

      deps.patch({ currentRun: run });
      deps.syncRunSnapshot(run.id, run.status);
      await deps.loadChatMessages(run.id, guard);
      if (!guard.isCurrent()) {
        return;
      }

      if (run.has_trace_data) {
        await deps.loadTrace(run.id, guard);
      } else {
        deps.clearTraceState();
      }
    } catch (error) {
      if (!guard.isCurrent()) {
        return;
      }
      deps.patch({ status: deps.formatError("loading the analysis run", error) });
    } finally {
      if (guard.isCurrent()) {
        deps.patch({ loadingRunDetail: false });
      }
    }
  }

  function handleRunEvent(payload: AnalysisRunEvent) {
    deps.applyRunEvent(payload);

    if (payload.chunk_summary) {
      deps.patch({ inspectorMode: "chunks" });
    }

    if (deps.getState().activeRunId === null) {
      deps.patch({ activeRunId: payload.run_id, inspectorMode: "active" });
      void openRun(payload.run_id);
    }

    const focusedState = deps.getState();
    const isFocused =
      focusedState.activeRunId === null ||
      focusedState.activeRunId === payload.run_id ||
      focusedState.currentRun?.id === payload.run_id;

    if (
      payload.kind === "queued" ||
      payload.kind === "started" ||
      payload.kind === "progress"
    ) {
      if (payload.message && isFocused) {
        deps.patch({ status: payload.message });
      }
      return;
    }

    if (
      payload.kind === "completed" ||
      payload.kind === "failed" ||
      payload.kind === "cancelled"
    ) {
      if (payload.message && isFocused) {
        deps.patch({ status: payload.message });
      } else if (payload.error && isFocused) {
        deps.patch({ status: `Analysis failed: ${payload.error}` });
      }

      void loadActiveRuns();
      void loadRuns();

      const state = deps.getState();
      if (state.activeRunId === payload.run_id || state.currentRun?.id === payload.run_id) {
        void openRun(payload.run_id);
      }
    }
  }

  function invalidateOpenRunRequests() {
    openRunRequestToken += 1;
  }

  return {
    loadRuns,
    loadActiveRuns,
    openRun,
    handleRunEvent,
    invalidateOpenRunRequests,
  };
}
```

- [ ] **Step 4: Run the workflow tests to confirm GREEN for loading workflows**

Run:

```powershell
npm.cmd test -- src/lib/analysis-run-workflow.test.ts
```

Expected: PASS for the first 5 workflow tests.

- [ ] **Step 5: Commit**

Run:

```powershell
git add src/lib/analysis-run-workflow.ts src/lib/analysis-run-workflow.test.ts
git commit -m "test(frontend): extract analysis run loading workflow"
```

## Task 3: Open Run Workflow Coverage

**Files:**
- Modify: `src/lib/analysis-run-workflow.test.ts`
- Modify: `src/lib/analysis-run-workflow.ts`

- [ ] **Step 1: Add focused `openRun` tests**

Append these tests inside the existing `describe("analysis-run-workflow", ...)` block:

```ts
  it("opens a run by loading detail, chat, and trace data", async () => {
    const { state, deps, workflow } = createHarness();
    deps.getRun.mockResolvedValueOnce(runDetail({
      id: 7,
      status: "completed",
      has_trace_data: true,
    }));
    deps.loadChatMessages.mockResolvedValueOnce(undefined);
    deps.loadTrace.mockResolvedValueOnce(undefined);

    await workflow.openRun(7);

    expect(state.inspectorMode).toBe("history");
    expect(state.activeRunId).toBe(7);
    expect(state.currentRun?.id).toBe(7);
    expect(deps.syncRunSnapshot).toHaveBeenCalledWith(7, "completed");
    expect(deps.loadChatMessages).toHaveBeenCalledWith(7, expect.objectContaining({
      isCurrent: expect.any(Function),
    }));
    expect(deps.loadTrace).toHaveBeenCalledWith(7, expect.objectContaining({
      isCurrent: expect.any(Function),
    }));
    expect(state.loadingRunDetail).toBe(false);
  });

  it("clears trace state when the opened run has no trace data", async () => {
    const { deps, workflow } = createHarness();
    deps.getRun.mockResolvedValueOnce(runDetail({ id: 7, has_trace_data: false }));
    deps.loadChatMessages.mockResolvedValueOnce(undefined);

    await workflow.openRun(7);

    expect(deps.clearTraceState).toHaveBeenCalled();
    expect(deps.loadTrace).not.toHaveBeenCalled();
  });

  it("cancels a foreign active chat before opening another run", async () => {
    const { deps, workflow } = createHarness({
      activeChatRequestId: "chat-a",
      activeChatRunId: 5,
    });
    deps.getRun.mockResolvedValueOnce(runDetail({ id: 7 }));
    deps.loadChatMessages.mockResolvedValueOnce(undefined);

    await workflow.openRun(7);

    expect(deps.cancelChatSilently).toHaveBeenCalled();
    expect(deps.clearChatState).toHaveBeenCalled();
  });

  it("reports a not-found run and clears current run only when it matches", async () => {
    const { state, deps, workflow } = createHarness({
      currentRun: runDetail({ id: 7 }),
    });
    deps.getRun.mockResolvedValueOnce(null);

    await workflow.openRun(7);

    expect(state.status).toBe("Analysis run 7 was not found.");
    expect(state.currentRun).toBeNull();
    expect(state.loadingRunDetail).toBe(false);
  });

  it("ignores stale openRun results from overlapping requests", async () => {
    const { state, deps, workflow } = createHarness();
    let resolveFirst: (run: AnalysisRunDetail) => void = () => {};
    deps.getRun
      .mockImplementationOnce(() => new Promise<AnalysisRunDetail>((resolve) => {
        resolveFirst = resolve;
      }))
      .mockResolvedValueOnce(runDetail({ id: 8, result_markdown: "second" }));
    deps.loadChatMessages.mockResolvedValue(undefined);

    const first = workflow.openRun(7);
    const second = workflow.openRun(8);
    resolveFirst(runDetail({ id: 7, result_markdown: "first" }));
    await Promise.all([first, second]);

    expect(state.currentRun?.id).toBe(8);
    expect(deps.loadChatMessages).toHaveBeenCalledWith(8, expect.any(Object));
    expect(deps.loadChatMessages).not.toHaveBeenCalledWith(7, expect.any(Object));
    expect(state.loadingRunDetail).toBe(false);
  });

  it("keeps stale loadChatMessages callbacks from changing route chat state", async () => {
    const { deps, workflow } = createHarness();
    let capturedGuard = { isCurrent: () => true };
    deps.getRun.mockResolvedValueOnce(runDetail({ id: 7 }));
    deps.loadChatMessages.mockImplementationOnce(async (_runId, guard) => {
      capturedGuard = guard;
    });

    await workflow.openRun(7);
    workflow.invalidateOpenRunRequests();

    expect(capturedGuard.isCurrent()).toBe(false);
  });
```

- [ ] **Step 2: Run tests**

Run:

```powershell
npm.cmd test -- src/lib/analysis-run-workflow.test.ts
```

Expected: PASS. If any test fails because implementation was already completed in Task 2, fix only the test/implementation mismatch and keep behavior aligned with the route's current workflow.

- [ ] **Step 3: Commit**

Run:

```powershell
git add src/lib/analysis-run-workflow.ts src/lib/analysis-run-workflow.test.ts
git commit -m "test(frontend): extract open run workflow"
```

## Task 4: Run Event Orchestration Coverage

**Files:**
- Modify: `src/lib/analysis-run-workflow.test.ts`
- Modify: `src/lib/analysis-run-workflow.ts`

- [ ] **Step 1: Add `handleRunEvent` tests**

Append these tests inside the existing `describe("analysis-run-workflow", ...)` block:

```ts
  it("applies run events and switches the inspector to chunks when chunk summaries arrive", () => {
    const { state, deps, workflow } = createHarness({ activeRunId: 7 });
    const payload = runEvent({
      run_id: 7,
      chunk_summary: {
        index: 1,
        total: 2,
        message_count: 10,
        summary: "chunk",
        topics: [],
        notable_points: [],
        candidate_refs: [],
      },
    });

    workflow.handleRunEvent(payload);

    expect(deps.applyRunEvent).toHaveBeenCalledWith(payload);
    expect(state.inspectorMode).toBe("chunks");
  });

  it("selects and opens the event run when no run is active", () => {
    const { state, deps, workflow } = createHarness({ activeRunId: null });
    deps.getRun.mockResolvedValueOnce(runDetail({ id: 7 }));
    deps.loadChatMessages.mockResolvedValueOnce(undefined);

    workflow.handleRunEvent(runEvent({
      run_id: 7,
      kind: "started",
      message: "Started",
    }));

    expect(state.activeRunId).toBe(7);
    expect(state.inspectorMode).toBe("history");
    expect(deps.getRun).toHaveBeenCalledWith(7);
    expect(state.status).toBe("Started");
  });

  it("updates progress status only for the focused run", () => {
    const { state, workflow } = createHarness({ activeRunId: 7 });

    workflow.handleRunEvent(runEvent({
      run_id: 8,
      kind: "progress",
      message: "Other run progress",
    }));
    expect(state.status).toBe("");

    workflow.handleRunEvent(runEvent({
      run_id: 7,
      kind: "progress",
      message: "Focused progress",
    }));
    expect(state.status).toBe("Focused progress");
  });

  it("refreshes active and saved runs on terminal events", () => {
    const { state, deps, workflow } = createHarness({
      activeRunId: 7,
      currentRun: runDetail({ id: 7 }),
    });
    deps.listActiveRuns.mockResolvedValue([]);
    deps.listRuns.mockResolvedValue([]);
    deps.getRun.mockResolvedValue(runDetail({ id: 7, status: "completed" }));
    deps.loadChatMessages.mockResolvedValue(undefined);

    workflow.handleRunEvent(runEvent({
      run_id: 7,
      kind: "completed",
      message: "Analysis complete",
    }));

    expect(state.status).toBe("Analysis complete");
    expect(deps.listActiveRuns).toHaveBeenCalled();
    expect(deps.listRuns).toHaveBeenCalled();
    expect(deps.getRun).toHaveBeenCalledWith(7);
  });

  it("uses terminal error status for focused failed events without a message", () => {
    const { state, workflow } = createHarness({ activeRunId: 7 });

    workflow.handleRunEvent(runEvent({
      run_id: 7,
      kind: "failed",
      message: null,
      error: "model failed",
    }));

    expect(state.status).toBe("Analysis failed: model failed");
  });
```

- [ ] **Step 2: Run workflow tests**

Run:

```powershell
npm.cmd test -- src/lib/analysis-run-workflow.test.ts
```

Expected: PASS for all workflow tests.

- [ ] **Step 3: Commit**

Run:

```powershell
git add src/lib/analysis-run-workflow.ts src/lib/analysis-run-workflow.test.ts
git commit -m "test(frontend): extract analysis run event workflow"
```

## Task 5: Route Wiring

**Files:**
- Modify: `src/routes/analysis/+page.svelte`

- [ ] **Step 1: Update imports**

In `src/routes/analysis/+page.svelte`, remove direct run-event use of `listen` for `analysis://run` and add these imports:

```ts
  import {
    getAnalysisRun,
    listActiveAnalysisRuns,
    listAnalysisRuns,
    listenToAnalysisRunEvents,
  } from "$lib/api/analysis-runs";
  import {
    createAnalysisRunWorkflow,
    type AnalysisRunRequestGuard,
    type AnalysisRunWorkflowPatch,
  } from "$lib/analysis-run-workflow";
```

Keep `listen` from `@tauri-apps/api/event` until chat, NotebookLM, and Takeout listeners are extracted.

- [ ] **Step 2: Replace route token state with controller invalidation**

Remove:

```ts
  let openRunRequestToken = 0;
```

In `clearOpenedRunState`, replace:

```ts
    openRunRequestToken += 1;
```

with:

```ts
    runWorkflow.invalidateOpenRunRequests();
```

- [ ] **Step 3: Add route patch helper and instantiate the workflow**

Place this after derived values and before the local run workflow wrapper functions:

```ts
  function applyRunWorkflowPatch(patch: AnalysisRunWorkflowPatch) {
    if ("runs" in patch) runs = patch.runs ?? [];
    if ("activeRuns" in patch) activeRuns = patch.activeRuns ?? [];
    if ("activeRunId" in patch) activeRunId = patch.activeRunId ?? null;
    if ("currentRun" in patch) currentRun = patch.currentRun ?? null;
    if ("inspectorMode" in patch && patch.inspectorMode) inspectorMode = patch.inspectorMode;
    if ("loadingRuns" in patch) loadingRuns = patch.loadingRuns ?? false;
    if ("loadingActiveRuns" in patch) loadingActiveRuns = patch.loadingActiveRuns ?? false;
    if ("loadingRunDetail" in patch) loadingRunDetail = patch.loadingRunDetail ?? false;
    if ("status" in patch && patch.status !== undefined) status = patch.status;
  }

  const runWorkflow = createAnalysisRunWorkflow({
    getState: () => ({
      historyScopeParams,
      activeRunId,
      currentRun,
      activeChatRequestId,
      activeChatRunId,
    }),
    patch: applyRunWorkflowPatch,
    listRuns: listAnalysisRuns,
    listActiveRuns: listActiveAnalysisRuns,
    getRun: getAnalysisRun,
    syncRunSnapshot,
    pruneLiveRuns,
    applyRunEvent,
    cancelChatSilently: () => cancelChat({ silent: true }),
    clearChatState,
    loadChatMessages,
    loadTrace,
    clearTraceState,
    formatError: formatAppError,
  });
```

- [ ] **Step 4: Replace local run workflow functions with delegating wrappers**

Replace the current bodies of `loadRuns`, `loadActiveRuns`, and `openRun` with:

```ts
  async function loadRuns() {
    await runWorkflow.loadRuns();
  }

  async function loadActiveRuns() {
    await runWorkflow.loadActiveRuns();
  }

  async function openRun(runId: number) {
    await runWorkflow.openRun(runId);
  }
```

Remove the route-local `syncActiveRunState` function after delegation, because the controller owns that behavior.

- [ ] **Step 5: Update guard-aware trace and chat loaders**

Change `loadTrace` signature:

```ts
  async function loadTrace(runId: number, guard?: AnalysisRunRequestGuard) {
```

Replace each stale check in `loadTrace`:

```ts
      if (requestToken !== undefined && requestToken !== openRunRequestToken) {
```

with:

```ts
      if (guard && !guard.isCurrent()) {
```

Change `loadChatMessages` signature:

```ts
  async function loadChatMessages(runId: number, guard?: AnalysisRunRequestGuard) {
```

Replace each stale check in `loadChatMessages`:

```ts
      if (requestToken !== undefined && requestToken !== openRunRequestToken) {
```

with:

```ts
      if (guard && !guard.isCurrent()) {
```

Replace the `finally` loading check:

```ts
      if (requestToken === undefined || requestToken === openRunRequestToken) {
        loadingChat = false;
      }
```

with:

```ts
      if (!guard || guard.isCurrent()) {
        loadingChat = false;
      }
```

- [ ] **Step 6: Replace the run listener body**

In `onMount`, replace the `listen<AnalysisRunEvent>("analysis://run", ...)` call with:

```ts
    void listenToAnalysisRunEvents(({ payload }) => {
      if (disposed) {
        return;
      }

      runWorkflow.handleRunEvent(payload);
    }).then((unlisten) => {
      if (disposed) {
        unlisten();
        return;
      }
      detachAnalysisListener = unlisten;
    });
```

- [ ] **Step 7: Run route checks**

Run:

```powershell
npm.cmd test -- src/lib/analysis-run-workflow.test.ts src/lib/api/analysis-runs.test.ts
npm.cmd run check
```

Expected: targeted tests PASS and `svelte-check found 0 errors and 0 warnings`.

- [ ] **Step 8: Commit**

Run:

```powershell
git add src/routes/analysis/+page.svelte src/lib/analysis-run-workflow.ts src/lib/analysis-run-workflow.test.ts src/lib/api/analysis-runs.ts src/lib/api/analysis-runs.test.ts
git commit -m "refactor(frontend): extract analysis run workflow controller"
```

## Final Verification

- [ ] **Run full frontend tests**

Run:

```powershell
npm.cmd test
```

Expected: all Vitest files PASS.

- [ ] **Run Svelte check**

Run:

```powershell
npm.cmd run check
```

Expected: `svelte-check found 0 errors and 0 warnings`.

- [ ] **Run whitespace check**

Run:

```powershell
git diff --check
```

Expected: no actionable whitespace errors. If the repo reports only known CRLF normalization warnings, record that explicitly in the handoff.

## Execution Notes

- Use `npm.cmd`, not `npm`, because PowerShell blocks `npm.ps1` in this environment.
- `npm.cmd test` and `npm.cmd run check` may require escalation because Vite/esbuild spawning can fail in the sandbox with `EPERM`.
- Keep commits focused. Do not merge report start, run deletion, chat listener, NotebookLM, Takeout, or backend changes into these commits.
- If using subagents, prefer one worker per task and review diffs between tasks. The user explicitly allowed subagents for Superpowers work.

