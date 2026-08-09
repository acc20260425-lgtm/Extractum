import { evidenceSourceActionDecision, type CompanionRunsFilterState } from "$lib/analysis-run-companion-state";
import {
  transitionAnalysisWorkspaceState,
  type AnalysisWorkspaceUiState,
  type CompanionTab,
} from "$lib/analysis-workspace-state";
import type { RunSnapshotAvailability } from "$lib/analysis-report-canvas-state";
import type { SnapshotProbeState } from "$lib/analysis-run-snapshot-affordance";
import type { EvidenceSourceScope } from "$lib/analysis-evidence-source-navigation";
import type { AnalysisRunDetail, AnalysisRunSummary, AnalysisTraceRef } from "$lib/types/analysis";

export function runCompanionRouteProps<TChunk>({
  workspaceUiState,
  focusedChunkSummaries,
  selectedRunIsActive,
  activeRuns,
  savedRuns,
  runsFilter,
}: {
  workspaceUiState: AnalysisWorkspaceUiState;
  focusedChunkSummaries: TChunk[];
  selectedRunIsActive: boolean;
  activeRuns: AnalysisRunSummary[];
  savedRuns: AnalysisRunSummary[];
  runsFilter: CompanionRunsFilterState;
}) {
  return {
    companionTab: workspaceUiState.companionTab,
    focusedChunkSummaries,
    selectedRunIsActive,
    activeRuns,
    savedRuns,
    runsFilter,
  };
}

export function changeCompanionTab(
  state: AnalysisWorkspaceUiState,
  companionTab: CompanionTab,
  returnToEvidenceTraceRef?: string,
) {
  return transitionAnalysisWorkspaceState(
    state,
    returnToEvidenceTraceRef === undefined
      ? { type: "change_companion_tab", companionTab }
      : { type: "return_to_evidence_review", traceRef: returnToEvidenceTraceRef },
  );
}

export async function submitCompanionQuestion(
  state: AnalysisWorkspaceUiState,
  submit: () => void | Promise<void>,
  applyState?: (state: AnalysisWorkspaceUiState) => void,
) {
  const next = changeCompanionTab(state, "chat");
  applyState?.(next);
  await submit();
  return next;
}

export function runIdFromHref(href: string) {
  const value = new URL(href).searchParams.get("runId");
  if (!value) return null;
  const runId = Number(value);
  return Number.isInteger(runId) && runId > 0 ? runId : null;
}

export async function showEvidenceInSource({
  currentRun,
  selectedTrace,
  highlightedRef,
  snapshotAvailability,
  snapshotProbeState,
  sourceScope,
  nextRequestId,
  clearHighlight,
  setReturnContext,
  setPendingFocus,
  dispatch,
  loadSourceWindow,
}: {
  currentRun: AnalysisRunDetail | null;
  selectedTrace: AnalysisTraceRef | null;
  highlightedRef?: string;
  snapshotAvailability: RunSnapshotAvailability;
  snapshotProbeState: SnapshotProbeState;
  sourceScope: EvidenceSourceScope;
  nextRequestId: () => string;
  clearHighlight: () => void;
  setReturnContext: (value: { kind: "evidence"; runId: number; traceRef: string; sourceScope: EvidenceSourceScope; sourceViewBasis: "run_snapshot" | "live_source" }) => void;
  setPendingFocus: (value: { requestId: string; runId: number; sourceScope: EvidenceSourceScope; sourceViewBasis: "run_snapshot" | "live_source"; traceRef: string }) => void;
  dispatch: (event: { type: "show_evidence_in_source"; sourceViewBasis: "run_snapshot" | "live_source"; highlightedRef: string }) => void;
  loadSourceWindow: (input: { decision: Exclude<ReturnType<typeof evidenceSourceActionDecision>, { kind: "unavailable" }>; trace: AnalysisTraceRef; requestId: string; canonicalRef: string; sourceScope: EvidenceSourceScope }) => void | Promise<void>;
}) {
  const decision = evidenceSourceActionDecision({ currentRun, selectedTrace, snapshotAvailability, snapshotProbeState });
  if (decision.kind === "unavailable" || !currentRun || !selectedTrace) {
    return { kind: "unavailable" as const, decision };
  }

  const requestId = nextRequestId();
  const canonicalRef = highlightedRef ?? decision.highlightedRef;
  clearHighlight();
  setReturnContext({ kind: "evidence", runId: currentRun.id, traceRef: canonicalRef, sourceScope, sourceViewBasis: decision.sourceViewBasis });
  setPendingFocus({ requestId, runId: currentRun.id, sourceScope, sourceViewBasis: decision.sourceViewBasis, traceRef: canonicalRef });
  dispatch({ type: "show_evidence_in_source", sourceViewBasis: decision.sourceViewBasis, highlightedRef: canonicalRef });
  await loadSourceWindow({ decision, trace: selectedTrace, requestId, canonicalRef, sourceScope });
  return { kind: "started" as const, decision, requestId, canonicalRef };
}
