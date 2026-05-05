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
