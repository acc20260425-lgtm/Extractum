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
import type {
  AnalysisReportStartCommand,
  AnalysisRunDetail,
  AnalysisRunEvent,
  AnalysisRunSummary,
  ListAnalysisRunsInput,
} from "$lib/types/analysis";

export type AnalysisRunInspectorMode = "active" | "history" | "trace" | "chunks";

export interface AnalysisRunWorkflowState {
  historyScopeParams: AnalysisHistoryScopeParams | null;
  activeRunId: number | null;
  currentRun: AnalysisRunDetail | null;
  activeChatRequestId: string | null;
  activeChatRunId: number | null;
  runs: AnalysisRunSummary[];
  activeRuns: AnalysisRunSummary[];
  deletingRunIds: Record<number, boolean>;
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
  startingReport: boolean;
  deletingRunIds: Record<number, boolean>;
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
  startReport(command: AnalysisReportStartCommand): Promise<number>;
  cancelRun(runId: number): Promise<void>;
  deleteRun(runId: number): Promise<void>;
  confirm(options: RunDeletionDialog): Promise<boolean>;
  cancelChatSilently(): Promise<void>;
  clearChatState(): void;
  clearOpenedRunState(runId: number): void;
  setInitialLiveRun(runId: number): void;
  loadChatMessages(runId: number, guard?: AnalysisRunRequestGuard): Promise<void>;
  loadTrace(runId: number, guard?: AnalysisRunRequestGuard): Promise<void>;
  clearTraceState(): void;
  onRunOpened?(run: AnalysisRunDetail): void;
  formatError(action: string, error: unknown): string;
}

export function createAnalysisRunWorkflow(deps: AnalysisRunWorkflowDeps) {
  let openRunRequestToken = 0;

  function createGuard(token: number): AnalysisRunRequestGuard {
    return {
      isCurrent: () => token === openRunRequestToken,
    };
  }

  async function loadRunsForScope(params: AnalysisHistoryScopeParams | null) {
    if (params === null) {
      deps.patch({ runs: [], loadingRuns: false });
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

  async function loadRuns() {
    await loadRunsForScope(deps.getState().historyScopeParams);
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
      deps.onRunOpened?.(run);
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
    loadRunsForScope,
    loadRuns,
    loadActiveRuns,
    openRun,
    startReport,
    cancelRun,
    deleteSavedRun,
    handleRunEvent,
    invalidateOpenRunRequests,
  };
}
