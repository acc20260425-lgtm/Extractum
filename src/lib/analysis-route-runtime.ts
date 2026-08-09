import { snapshotProbeStateFromAvailability } from "$lib/analysis-run-snapshot-affordance";
import type { SourceReturnContext } from "$lib/analysis-evidence-source-navigation";
import type { RunSnapshotAvailability } from "$lib/analysis-report-canvas-state";
import type {
  AnalysisRunMessageCursor,
  AnalysisRunMessagesPage,
  ListAnalysisRunMessagesInput,
} from "$lib/types/analysis";
import type { AnalysisWorkspaceUiState, CanvasMode, SourceViewBasis, WorkspaceSelection } from "$lib/analysis-workspace-state";

export function reportCanvasRouteProps({
  workspaceUiState,
  snapshot,
  sourceReturnContext,
  onChangeCanvasMode,
  onViewLiveSource,
  onBackToRunSnapshot,
  onReturnToEvidenceReview,
}: {
  workspaceUiState: AnalysisWorkspaceUiState;
  snapshot?: { availability: RunSnapshotAvailability; loading: boolean; error: string };
  sourceReturnContext: SourceReturnContext;
  onChangeCanvasMode: (mode: CanvasMode) => void;
  onViewLiveSource: () => void;
  onBackToRunSnapshot: () => void;
  onReturnToEvidenceReview: () => void;
}) {
  const availability = snapshot?.availability ?? "unknown";
  const loading = snapshot?.loading ?? false;
  const error = snapshot?.error ?? "";
  return {
    canvasMode: workspaceUiState.canvasMode,
    sourceViewBasis: workspaceUiState.sourceViewBasis,
    sourceReturnContext,
    runSnapshotAvailability: availability,
    snapshotProbeState: snapshotProbeStateFromAvailability({
      snapshotAvailability: availability,
      loadingRunSnapshotMessages: loading,
      runSnapshotError: error,
    }),
    loadingRunSnapshotMessages: loading,
    runSnapshotError: error,
    onChangeCanvasMode,
    onViewLiveSource,
    onBackToRunSnapshot,
    onReturnToEvidenceReview,
  };
}

export function returnToEvidenceReview({
  activeContext,
  clearPendingFocus,
  clearHighlight,
  dispatch,
  clearReturnContext,
}: {
  activeContext: SourceReturnContext;
  clearPendingFocus: () => void;
  clearHighlight: () => void;
  dispatch: (event: { type: "return_to_evidence_review"; traceRef: string }) => void;
  clearReturnContext: () => void;
}) {
  if (activeContext?.kind !== "evidence") return false;
  clearPendingFocus();
  clearHighlight();
  dispatch({ type: "return_to_evidence_review", traceRef: activeContext.traceRef });
  clearReturnContext();
  return true;
}

export async function loadRunSnapshotPage({
  runId,
  sourceId,
  aroundRef,
  after,
  limit,
  listMessages,
}: {
  runId: number;
  sourceId: number;
  aroundRef: string;
  after: AnalysisRunMessageCursor | null;
  limit: number;
  listMessages: (input: ListAnalysisRunMessagesInput) => Promise<AnalysisRunMessagesPage>;
}) {
  return listMessages({ runId, sourceId, aroundRef, after, limit });
}

export function shouldLoadRunSnapshot({
  runId,
  sourceViewBasis,
  lastSnapshotLoadKey,
}: {
  runId: number | null;
  sourceViewBasis: SourceViewBasis;
  lastSnapshotLoadKey: string;
}) {
  return runId !== null && sourceViewBasis === "run_snapshot" && lastSnapshotLoadKey !== `${runId}:first`;
}

export function reportLaunchPreflightProps<T>(currentSourceMetric: T, reportLaunchDisabledReason: string | null) {
  return { currentSourceMetric, reportLaunchDisabledReason };
}

export async function refreshCatalogForTerminalSourceJob(
  job: { status: string; source_id: number },
  activeSourceId: number,
  deps: { loadSourceCatalog: () => Promise<void>; loadGroups: () => Promise<void> },
) {
  if (job.source_id !== activeSourceId || !["completed", "failed", "cancelled"].includes(job.status)) return false;
  await Promise.all([deps.loadSourceCatalog(), deps.loadGroups()]);
  return true;
}

export function compactSourceRailRouteProps<TStarting, TJobs>({
  workspaceSelection,
  startingMigratedHistorySourceIds,
  sourceJobsBySource,
  onSelectSource,
  onSelectGroup,
  onStartMigratedHistoryImport,
}: {
  workspaceSelection: WorkspaceSelection;
  startingMigratedHistorySourceIds: TStarting;
  sourceJobsBySource: TJobs;
  onSelectSource: (sourceId: number) => void;
  onSelectGroup: (groupId: number) => void;
  onStartMigratedHistoryImport: (sourceId: number) => void;
}) {
  return {
    workspaceSelection,
    railData: {
      startingMigratedHistorySourceIds,
      sourceJobsBySource,
    },
    onSelectSource,
    onSelectGroup,
    onStartMigratedHistoryImport,
  };
}
