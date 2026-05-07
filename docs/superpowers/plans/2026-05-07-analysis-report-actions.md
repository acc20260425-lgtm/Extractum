# Analysis Report Actions Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move `start_analysis_report`, `cancel_analysis_run`, and `delete_analysis_run` out of `src/routes/analysis/+page.svelte` into typed API wrappers and `createAnalysisRunWorkflow`.

**Architecture:** Extend the existing analysis run boundary instead of adding a parallel report-actions module. `src/lib/api/analysis-runs.ts` owns Tauri command names, `src/lib/analysis-run-workflow.ts` owns action orchestration, and the Svelte route only adapts local `$state` into workflow dependencies.

**Tech Stack:** Svelte 5, TypeScript, Tauri `invoke`, Vitest, PowerShell on Windows.

---

## File Structure

- Modify `src/lib/types/analysis.ts` to hold the shared `AnalysisReportStartCommand` DTO.
- Modify `src/lib/analysis-state.ts` to import that DTO instead of defining it locally.
- Modify `src/lib/api/analysis-runs.ts` to add typed wrappers for report start, run cancellation, and run deletion.
- Modify `src/lib/api/analysis-runs.test.ts` to cover the new wrapper command names and argument shapes.
- Modify `src/lib/analysis-run-workflow.ts` to add `startReport`, `cancelRun`, and `deleteSavedRun`.
- Modify `src/lib/analysis-run-workflow.test.ts` to cover the new workflow behaviors.
- Modify `src/routes/analysis/+page.svelte` to inject the new API wrappers and delegate report action handlers to the workflow.
- Modify `docs/code-review-results-2026-05-03.md` after route wiring removes the remaining raw report action command surface.
- Modify `docs/session-context-2026-05-03.md` after implementation verification to refresh the handoff.

## Task 1: Add Analysis Run Action API Wrappers

**Files:**
- Modify: `src/lib/types/analysis.ts`
- Modify: `src/lib/analysis-state.ts`
- Modify: `src/lib/api/analysis-runs.ts`
- Test: `src/lib/api/analysis-runs.test.ts`

- [ ] **Step 1: Move the shared start command DTO type**

In `src/lib/types/analysis.ts`, add this interface after `AnalysisRunDetail`:

```ts
export interface AnalysisReportStartCommand {
  sourceId: number | null;
  sourceGroupId: number | null;
  periodFrom: number;
  periodTo: number;
  outputLanguage: string;
  promptTemplateId: number;
  modelOverride: string | null;
  profileId: null;
}
```

In `src/lib/analysis-state.ts`, add `AnalysisReportStartCommand` to the existing type import from `$lib/types/analysis`:

```ts
import type {
  AnalysisChunkSummaryEvent,
  AnalysisPromptTemplate,
  AnalysisChatTurn,
  AnalysisReportStartCommand,
  AnalysisRunDetail,
  AnalysisRunEvent,
  AnalysisRunSummary,
  AnalysisSourceGroup,
  AnalysisTraceData,
  AnalysisTraceRef,
} from "$lib/types/analysis";
```

Then delete the local `export type AnalysisReportStartCommand = { ... }` block from `src/lib/analysis-state.ts`. Keep `AnalysisReportStartState` and `AnalysisReportStartDecision`; `AnalysisReportStartDecision` should continue referencing `AnalysisReportStartCommand`.

- [ ] **Step 2: Write the failing API wrapper test**

In `src/lib/api/analysis-runs.test.ts`, update the import block to include the new wrapper names:

```ts
import {
  ANALYSIS_RUN_EVENT,
  cancelAnalysisRun,
  deleteAnalysisRun,
  getAnalysisRun,
  listActiveAnalysisRuns,
  listAnalysisRuns,
  listenToAnalysisRunEvents,
  startAnalysisReport,
} from "./analysis-runs";
```

Add this test before the listener test:

```ts
  it("wraps analysis report start and destructive run actions", async () => {
    invokeMock.mockResolvedValueOnce(77);
    await expect(startAnalysisReport({
      sourceId: 7,
      sourceGroupId: null,
      periodFrom: 1_776_038_400,
      periodTo: 1_776_211_199,
      outputLanguage: "Russian",
      promptTemplateId: 5,
      modelOverride: null,
      profileId: null,
    })).resolves.toBe(77);
    expect(invokeMock).toHaveBeenLastCalledWith("start_analysis_report", {
      sourceId: 7,
      sourceGroupId: null,
      periodFrom: 1_776_038_400,
      periodTo: 1_776_211_199,
      outputLanguage: "Russian",
      promptTemplateId: 5,
      modelOverride: null,
      profileId: null,
    });

    invokeMock.mockResolvedValueOnce(undefined);
    await expect(cancelAnalysisRun(77)).resolves.toBeUndefined();
    expect(invokeMock).toHaveBeenLastCalledWith("cancel_analysis_run", { runId: 77 });

    invokeMock.mockResolvedValueOnce(undefined);
    await expect(deleteAnalysisRun(77)).resolves.toBeUndefined();
    expect(invokeMock).toHaveBeenLastCalledWith("delete_analysis_run", { runId: 77 });
  });
```

- [ ] **Step 3: Run the API test to verify RED**

Run:

```powershell
npm.cmd test -- src/lib/api/analysis-runs.test.ts
```

Expected: FAIL with missing exports such as `startAnalysisReport`, `cancelAnalysisRun`, or `deleteAnalysisRun`.

- [ ] **Step 4: Implement the API wrappers**

In `src/lib/api/analysis-runs.ts`, import the moved DTO type:

```ts
import type {
  AnalysisReportStartCommand,
  AnalysisRunDetail,
  AnalysisRunEvent,
  AnalysisRunSummary,
  EventEnvelope,
} from "$lib/types/analysis";
```

Add these functions after `getAnalysisRun`:

```ts
export function startAnalysisReport(command: AnalysisReportStartCommand) {
  return invoke<number>("start_analysis_report", command);
}

export function cancelAnalysisRun(runId: number) {
  return invoke<void>("cancel_analysis_run", { runId });
}

export function deleteAnalysisRun(runId: number) {
  return invoke<void>("delete_analysis_run", { runId });
}
```

- [ ] **Step 5: Run focused API and state tests to verify GREEN**

Run:

```powershell
npm.cmd test -- src/lib/api/analysis-runs.test.ts src/lib/analysis-state.test.ts
```

Expected: PASS. The output should report both test files passed.

- [ ] **Step 6: Commit Task 1**

Run:

```powershell
git add src/lib/types/analysis.ts src/lib/analysis-state.ts src/lib/api/analysis-runs.ts src/lib/api/analysis-runs.test.ts
git commit -m "refactor(analysis): add report action api wrappers"
```

## Task 2: Move Report Action Orchestration Into Analysis Run Workflow

**Files:**
- Modify: `src/lib/analysis-run-workflow.ts`
- Test: `src/lib/analysis-run-workflow.test.ts`

- [ ] **Step 1: Extend workflow tests with report action harness state**

In `src/lib/analysis-run-workflow.test.ts`, extend the import from `./analysis-run-workflow`:

```ts
import {
  createAnalysisRunWorkflow,
  type AnalysisRunWorkflowPatch,
  type AnalysisRunWorkflowState,
} from "./analysis-run-workflow";
```

Keep this import shape and add no new workflow imports unless the implementation exports new public types.

Extend `AnalysisRunWorkflowHarnessState` with:

```ts
  startingReport: boolean;
  deletingRunIds: Record<number, boolean>;
```

Update the default `state` object in `createHarness`:

```ts
    startingReport: false,
    deletingRunIds: {},
```

Add these dependency mocks to `deps` in `createHarness`:

```ts
    startReport: vi.fn(),
    cancelRun: vi.fn(),
    deleteRun: vi.fn(),
    confirm: vi.fn(),
    clearOpenedRunState: vi.fn(),
    setInitialLiveRun: vi.fn(),
```

Do not remove existing mocks.

- [ ] **Step 2: Write RED tests for `startReport`**

Add these tests before `handleRunEvent` tests:

```ts
  it("reports report-start validation failures without invoking the api", async () => {
    const { state, deps, workflow } = createHarness();

    await workflow.startReport({
      analysisScope: "single_source",
      selectedSourceId: "",
      selectedGroupId: "",
      selectedTemplateId: "5",
      periodFrom: "2026-05-01",
      periodTo: "2026-05-03",
      outputLanguage: "Russian",
      modelOverride: "",
    });

    expect(state.status).toBe("Select a source first.");
    expect(deps.startReport).not.toHaveBeenCalled();
    expect(state.startingReport).toBe(false);
  });

  it("starts a report, resets focused state, tracks the queued run, and opens it", async () => {
    const { state, deps, workflow } = createHarness({
      activeChatRequestId: "chat-a",
      activeChatRunId: 4,
      currentRun: runDetail({ id: 4 }),
    });
    deps.startReport.mockResolvedValueOnce(77);
    deps.listActiveRuns.mockResolvedValueOnce([runSummary({ id: 77, status: "queued" })]);
    deps.getRun.mockResolvedValueOnce(runDetail({ id: 77, status: "queued" }));
    deps.loadChatMessages.mockResolvedValueOnce(undefined);

    await workflow.startReport({
      analysisScope: "single_source",
      selectedSourceId: "7",
      selectedGroupId: "",
      selectedTemplateId: "5",
      periodFrom: "2026-05-01",
      periodTo: "2026-05-03",
      outputLanguage: " Russian ",
      modelOverride: " ",
    });

    expect(deps.cancelChatSilently).toHaveBeenCalled();
    expect(deps.clearChatState).toHaveBeenCalled();
    expect(deps.clearTraceState).toHaveBeenCalled();
    expect(state.currentRun).toBeNull();
    expect(deps.startReport).toHaveBeenCalledWith(expect.objectContaining({
      sourceId: 7,
      sourceGroupId: null,
      outputLanguage: "Russian",
      promptTemplateId: 5,
      modelOverride: null,
      profileId: null,
    }));
    expect(deps.setInitialLiveRun).toHaveBeenCalledWith(77);
    expect(state.activeRunId).toBe(77);
    expect(state.currentRun?.id).toBe(77);
    expect(state.startingReport).toBe(false);
  });

  it("formats start report errors and clears the starting flag", async () => {
    const { state, deps, workflow } = createHarness();
    deps.startReport.mockRejectedValueOnce("model busy");

    await workflow.startReport({
      analysisScope: "source_group",
      selectedSourceId: "",
      selectedGroupId: "9",
      selectedTemplateId: "5",
      periodFrom: "2026-05-01",
      periodTo: "2026-05-03",
      outputLanguage: "English",
      modelOverride: "gemini-2.5-pro",
    });

    expect(state.status).toBe("Error starting the analysis report: model busy");
    expect(state.startingReport).toBe(false);
  });
```

- [ ] **Step 3: Write RED tests for `cancelRun` and `deleteSavedRun`**

Add these tests after the `startReport` tests:

```ts
  it("cancels a run and reports cancellation status", async () => {
    const { state, deps, workflow } = createHarness();
    deps.cancelRun.mockResolvedValueOnce(undefined);

    await workflow.cancelRun(77);

    expect(deps.cancelRun).toHaveBeenCalledWith(77);
    expect(state.status).toBe("Cancelling analysis run 77...");
  });

  it("formats cancel run errors", async () => {
    const { state, deps, workflow } = createHarness();
    deps.cancelRun.mockRejectedValueOnce("already stopped");

    await workflow.cancelRun(77);

    expect(state.status).toBe("Error cancelling the analysis run: already stopped");
  });

  it("blocks deleting active runs before confirmation", async () => {
    const { state, deps, workflow } = createHarness();

    await workflow.deleteSavedRun(runSummary({ id: 77, status: "running" }));

    expect(state.status).toBe("Cancel or wait for this run before deleting it.");
    expect(deps.confirm).not.toHaveBeenCalled();
    expect(deps.deleteRun).not.toHaveBeenCalled();
  });

  it("does not delete a saved run when confirmation is cancelled", async () => {
    const { deps, workflow } = createHarness();
    deps.confirm.mockResolvedValueOnce(false);

    await workflow.deleteSavedRun(runSummary({ id: 77, status: "completed" }));

    expect(deps.confirm).toHaveBeenCalledWith(expect.objectContaining({
      title: "Delete saved run?",
      confirmLabel: "Delete",
      tone: "danger",
    }));
    expect(deps.deleteRun).not.toHaveBeenCalled();
  });

  it("deletes a saved run, clears focused state, reloads runs, and clears pending state", async () => {
    const { state, deps, workflow } = createHarness({
      runs: [runSummary({ id: 77, status: "completed" }), runSummary({ id: 78, status: "failed" })],
      activeRuns: [runSummary({ id: 79, status: "queued" })],
      activeRunId: 77,
      currentRun: runDetail({ id: 77, status: "completed" }),
      activeChatRequestId: "chat-a",
      activeChatRunId: 77,
    });
    deps.confirm.mockResolvedValueOnce(true);
    deps.deleteRun.mockResolvedValueOnce(undefined);
    deps.listRuns.mockResolvedValueOnce([runSummary({ id: 78, status: "failed" })]);

    await workflow.deleteSavedRun(runSummary({ id: 77, status: "completed" }));

    expect(deps.cancelChatSilently).toHaveBeenCalled();
    expect(deps.deleteRun).toHaveBeenCalledWith(77);
    expect(state.runs.map((run) => run.id)).toEqual([78]);
    expect(state.activeRuns.map((run) => run.id)).toEqual([79]);
    expect(deps.clearOpenedRunState).toHaveBeenCalledWith(77);
    expect(state.inspectorMode).toBe("history");
    expect(state.status).toBe("Saved run 77 deleted.");
    expect(state.deletingRunIds).toEqual({});
  });

  it("formats delete run errors and clears pending state", async () => {
    const { state, deps, workflow } = createHarness();
    deps.confirm.mockResolvedValueOnce(true);
    deps.deleteRun.mockRejectedValueOnce("db locked");

    await workflow.deleteSavedRun(runSummary({ id: 77, status: "completed" }));

    expect(state.status).toBe("Error deleting the saved run: db locked");
    expect(state.deletingRunIds).toEqual({});
  });
```

- [ ] **Step 4: Run workflow tests to verify RED**

Run:

```powershell
npm.cmd test -- src/lib/analysis-run-workflow.test.ts
```

Expected: FAIL with missing workflow methods such as `startReport`, `cancelRun`, or `deleteSavedRun`, or missing dependency fields in the workflow type.

- [ ] **Step 5: Implement workflow types and dependencies**

In `src/lib/analysis-run-workflow.ts`, update imports:

```ts
import {
  activeRunSyncDecision,
  analysisReportStartCommand,
  isActiveRunStatus,
  runDeletedStatus,
  runDeletionDecision,
  type AnalysisReportStartState,
  type RunDeletionDialog,
} from "$lib/analysis-state";
import type { AnalysisHistoryScopeParams } from "$lib/analysis-scope-state";
import type { AnalysisReportStartCommand, AnalysisRunDetail, AnalysisRunEvent, AnalysisRunSummary } from "$lib/types/analysis";
```

Extend `AnalysisRunWorkflowPatch`:

```ts
export type AnalysisRunWorkflowPatch = Partial<{
  runs: AnalysisRunSummary[];
  activeRuns: AnalysisRunSummary[];
  activeRunId: number | null;
  currentRun: AnalysisRunDetail | null;
  inspectorMode: AnalysisRunInspectorMode;
  loadingRuns: boolean;
  loadingActiveRuns: boolean;
  loadingRunDetail: boolean;
  startingReport: boolean;
  deletingRunIds: Record<number, boolean>;
  status: string;
}>;
```

Extend `AnalysisRunWorkflowDeps`:

```ts
  startReport(command: AnalysisReportStartCommand): Promise<number>;
  cancelRun(runId: number): Promise<void>;
  deleteRun(runId: number): Promise<void>;
  confirm(options: RunDeletionDialog): Promise<boolean>;
  clearOpenedRunState(runId: number): void;
  setInitialLiveRun(runId: number): void;
```

Extend `AnalysisRunWorkflowState` with the state needed by deletion:

```ts
  runs: AnalysisRunSummary[];
  activeRuns: AnalysisRunSummary[];
  deletingRunIds: Record<number, boolean>;
```

- [ ] **Step 6: Implement workflow methods**

Inside `createAnalysisRunWorkflow`, add these functions before `handleRunEvent`:

```ts
  async function startReport(input: AnalysisReportStartState) {
    const decision = analysisReportStartCommand(input);
    if (!decision.ok) {
      deps.patch({ status: decision.status });
      return;
    }

    deps.patch({
      startingReport: true,
      inspectorMode: "active",
      currentRun: null,
    });

    if (deps.getState().activeChatRequestId !== null) {
      await deps.cancelChatSilently();
    }
    deps.clearChatState();
    deps.clearTraceState();

    try {
      const runId = await deps.startReport(decision.command);
      deps.setInitialLiveRun(runId);
      deps.patch({ activeRunId: runId });
      await Promise.all([loadActiveRuns(), openRun(runId)]);
    } catch (error) {
      deps.patch({ status: deps.formatError("starting the analysis report", error) });
    } finally {
      deps.patch({ startingReport: false });
    }
  }

  async function cancelRun(runId: number) {
    try {
      await deps.cancelRun(runId);
      deps.patch({ status: `Cancelling analysis run ${runId}...` });
    } catch (error) {
      deps.patch({ status: deps.formatError("cancelling the analysis run", error) });
    }
  }

  async function deleteSavedRun(run: AnalysisRunSummary) {
    const decision = runDeletionDecision(run);
    if (!decision.ok) {
      deps.patch({ status: decision.status });
      return;
    }

    const confirmed = await deps.confirm(decision.dialog);
    if (!confirmed) {
      return;
    }

    deps.patch({
      deletingRunIds: { ...deps.getState().deletingRunIds, [run.id]: true },
    });

    try {
      const state = deps.getState();
      if (state.activeChatRequestId !== null && state.activeChatRunId === run.id) {
        await deps.cancelChatSilently();
      }

      await deps.deleteRun(run.id);
      deps.patch({
        runs: deps.getState().runs.filter((entry) => entry.id !== run.id),
        activeRuns: deps.getState().activeRuns.filter((entry) => entry.id !== run.id),
        inspectorMode: "history",
        status: runDeletedStatus(run),
      });
      deps.clearOpenedRunState(run.id);
      await loadRuns();
    } catch (error) {
      deps.patch({ status: deps.formatError("deleting the saved run", error) });
    } finally {
      const next = { ...deps.getState().deletingRunIds };
      delete next[run.id];
      deps.patch({ deletingRunIds: next });
    }
  }
```

Update the returned object:

```ts
  return {
    loadRuns,
    loadActiveRuns,
    openRun,
    startReport,
    cancelRun,
    deleteSavedRun,
    handleRunEvent,
    invalidateOpenRunRequests,
  };
```

- [ ] **Step 7: Run workflow tests to verify GREEN**

Run:

```powershell
npm.cmd test -- src/lib/analysis-run-workflow.test.ts
```

Expected: PASS for `src/lib/analysis-run-workflow.test.ts`.

- [ ] **Step 8: Commit Task 2**

Run:

```powershell
git add src/lib/analysis-run-workflow.ts src/lib/analysis-run-workflow.test.ts
git commit -m "refactor(analysis): move report actions into run workflow"
```

## Task 3: Wire Route To Workflow And Refresh Cleanup Docs

**Files:**
- Modify: `src/routes/analysis/+page.svelte`
- Modify: `docs/code-review-results-2026-05-03.md`
- Modify: `docs/session-context-2026-05-03.md`

- [ ] **Step 1: Update route imports**

In `src/routes/analysis/+page.svelte`, remove these route imports if no remaining route code uses them:

```ts
  import { invoke } from "@tauri-apps/api/core";
```

From `$lib/api/analysis-runs`, add:

```ts
    cancelAnalysisRun,
    deleteAnalysisRun,
    startAnalysisReport,
```

From `$lib/analysis-state`, remove these imports once route handlers delegate to the workflow:

```ts
    analysisReportStartCommand,
    createEmptyLiveRunState,
    runDeletedStatus,
    runDeletionDecision,
```

Keep `createEmptyLiveRunState` only if another route helper still needs it after adding `setInitialLiveRun`.

- [ ] **Step 2: Add workflow state and patch wiring**

Extend the `getState` object passed to `createAnalysisRunWorkflow` with:

```ts
      runs,
      activeRuns,
      deletingRunIds,
```

Extend `applyRunWorkflowPatch` or the local patch handler so it applies:

```ts
    if ("startingReport" in patch) startingReport = patch.startingReport ?? false;
    if ("deletingRunIds" in patch) deletingRunIds = patch.deletingRunIds ?? {};
```

Add the new dependencies in the `createAnalysisRunWorkflow` call:

```ts
    startReport: startAnalysisReport,
    cancelRun: cancelAnalysisRun,
    deleteRun: deleteAnalysisRun,
    confirm: openConfirmModal,
    clearOpenedRunState,
    setInitialLiveRun: (runId) => {
      liveRuns = {
        ...liveRuns,
        [runId]: {
          phase: "queued",
          progress: "",
          queuePosition: null,
          chunkSummaries: [],
          streamedOutput: "",
        },
      };
    },
```

- [ ] **Step 3: Replace route action bodies with workflow delegates**

Replace `runReport()` with:

```ts
  async function runReport() {
    await runWorkflow.startReport({
      analysisScope,
      selectedSourceId,
      selectedGroupId,
      selectedTemplateId,
      periodFrom,
      periodTo,
      outputLanguage,
      modelOverride,
    });
  }
```

Replace `cancelActiveRun(runId: number)` with:

```ts
  async function cancelActiveRun(runId: number) {
    await runWorkflow.cancelRun(runId);
  }
```

Replace `deleteSavedRun(run: AnalysisRunSummary)` with:

```ts
  async function deleteSavedRun(run: AnalysisRunSummary) {
    await runWorkflow.deleteSavedRun(run);
  }
```

- [ ] **Step 4: Run focused tests and route raw-command search**

Run:

```powershell
npm.cmd test -- src/lib/api/analysis-runs.test.ts src/lib/analysis-run-workflow.test.ts src/lib/analysis-state.test.ts
rg "start_analysis_report|cancel_analysis_run|delete_analysis_run" src/routes/analysis/+page.svelte
```

Expected: tests PASS. The `rg` command against `src/routes/analysis/+page.svelte` returns no output.

- [ ] **Step 5: Run Svelte check**

Run:

```powershell
npm.cmd run check
```

Expected: `svelte-check found 0 errors and 0 warnings`.

- [ ] **Step 6: Update review and session docs**

In `docs/code-review-results-2026-05-03.md`, update the remaining `/analysis` command surface section so it no longer lists:

```text
start_analysis_report
cancel_analysis_run
delete_analysis_run
```

Record the report action wrapper/controller extraction as resolved. The remaining recommended follow-up order should start with typed error conversion for remaining DB, Telegram, LLM, and validation paths.

In `docs/session-context-2026-05-03.md`, refresh the handoff with:

- branch name `analysis-report-actions-cleanup`;
- task commits from this workstream;
- new API wrappers in `src/lib/api/analysis-runs.ts`;
- new workflow methods in `src/lib/analysis-run-workflow.ts`;
- verification commands and outcomes;
- remaining follow-up as typed error conversion.

- [ ] **Step 7: Run full verification**

Run:

```powershell
npm.cmd test
npm.cmd run check
git diff --check
git status --short --branch
```

Expected:

- `npm.cmd test` reports all test files passed.
- `npm.cmd run check` reports 0 errors and 0 warnings.
- `git diff --check` exits 0.
- `git status --short --branch` shows only intended modified files before commit.

- [ ] **Step 8: Commit Task 3**

Run:

```powershell
git add src/routes/analysis/+page.svelte docs/code-review-results-2026-05-03.md docs/session-context-2026-05-03.md
git commit -m "refactor(analysis): use report action workflow"
```
