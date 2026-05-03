import {
  activeRunSyncDecision,
  isActiveRunStatus,
} from "$lib/analysis-state";
import type { AnalysisHistoryScopeParams } from "$lib/analysis-scope-state";
import type { ListAnalysisRunsInput } from "$lib/api/analysis-runs";
import type {
  AnalysisRunDetail,
  AnalysisRunEvent,
  AnalysisRunSummary,
} from "$lib/types/analysis";

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
