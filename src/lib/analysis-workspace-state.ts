import type { AnalysisRunSummary } from "$lib/types/analysis";

export type WorkspaceSelection =
  | { kind: "source"; sourceId: number }
  | { kind: "source_group"; sourceGroupId: number }
  | { kind: "none" };

export type OpenRunState =
  | { kind: "none" }
  | { kind: "active"; runId: number }
  | { kind: "saved"; runId: number };

export type CanvasMode = "report" | "source";
export type SourceViewBasis = "live_source" | "run_snapshot";
export type CompanionTab = "evidence" | "chat" | "chunks" | "runs";
export type LegacyAnalysisScope = "single_source" | "source_group";

export interface AnalysisWorkspaceUiState {
  workspaceSelection: WorkspaceSelection;
  openRunState: OpenRunState;
  canvasMode: CanvasMode;
  sourceViewBasis: SourceViewBasis;
  companionTab: CompanionTab;
  selectedTraceRef: string | null;
}

export interface LegacyAnalysisScopeState {
  analysisScope: LegacyAnalysisScope;
  selectedSourceId: string;
  selectedGroupId: string;
}

export interface RunWorkspaceInput {
  runId: number;
  status: string;
  sourceId: number | null;
  sourceGroupId: number | null;
  liveScopeExists?: boolean;
}

function numericId(value: string) {
  if (!value.trim()) return null;
  const parsed = Number(value);
  return Number.isInteger(parsed) && parsed > 0 ? parsed : null;
}

export function defaultAnalysisWorkspaceUiState(): AnalysisWorkspaceUiState {
  return {
    workspaceSelection: { kind: "none" },
    openRunState: { kind: "none" },
    canvasMode: "source",
    sourceViewBasis: "live_source",
    companionTab: "runs",
    selectedTraceRef: null,
  };
}

export function workspaceSelectionFromLegacy(
  analysisScope: LegacyAnalysisScope,
  selectedSourceId: string,
  selectedGroupId: string,
): WorkspaceSelection {
  if (analysisScope === "single_source") {
    const sourceId = numericId(selectedSourceId);
    return sourceId === null ? { kind: "none" } : { kind: "source", sourceId };
  }

  const sourceGroupId = numericId(selectedGroupId);
  return sourceGroupId === null ? { kind: "none" } : { kind: "source_group", sourceGroupId };
}

export function legacyScopeFromWorkspaceSelection(
  selection: WorkspaceSelection,
): LegacyAnalysisScopeState {
  if (selection.kind === "source") {
    return {
      analysisScope: "single_source",
      selectedSourceId: String(selection.sourceId),
      selectedGroupId: "",
    };
  }

  if (selection.kind === "source_group") {
    return {
      analysisScope: "source_group",
      selectedSourceId: "",
      selectedGroupId: String(selection.sourceGroupId),
    };
  }

  return {
    analysisScope: "single_source",
    selectedSourceId: "",
    selectedGroupId: "",
  };
}

export function runWorkspaceInputFromSummary(
  run: Pick<AnalysisRunSummary, "id" | "status" | "source_id" | "source_group_id">,
  liveScopeExists = true,
): RunWorkspaceInput {
  return {
    runId: run.id,
    status: run.status,
    sourceId: run.source_id,
    sourceGroupId: run.source_group_id,
    liveScopeExists,
  };
}

export function workspaceSelectionFromRunScope(run: RunWorkspaceInput): WorkspaceSelection {
  if (run.liveScopeExists === false) {
    return { kind: "none" };
  }

  if (run.sourceId !== null) {
    return { kind: "source", sourceId: run.sourceId };
  }

  if (run.sourceGroupId !== null) {
    return { kind: "source_group", sourceGroupId: run.sourceGroupId };
  }

  return { kind: "none" };
}

export function openRunStateForStatus(status: string, runId: number): OpenRunState {
  if (status === "queued" || status === "running") {
    return { kind: "active", runId };
  }

  return { kind: "saved", runId };
}

export function defaultCompanionTabForRun(status: string): CompanionTab {
  return status === "completed" ? "evidence" : "runs";
}

export function openRunWorkspaceState(
  current: AnalysisWorkspaceUiState,
  run: RunWorkspaceInput,
): AnalysisWorkspaceUiState {
  return {
    ...current,
    workspaceSelection: workspaceSelectionFromRunScope(run),
    openRunState: openRunStateForStatus(run.status, run.runId),
    canvasMode: "report",
    sourceViewBasis: "run_snapshot",
    companionTab: defaultCompanionTabForRun(run.status),
    selectedTraceRef: null,
  };
}

export function clearRunBoundWorkspaceState(
  current: AnalysisWorkspaceUiState,
): AnalysisWorkspaceUiState {
  return {
    ...current,
    openRunState: { kind: "none" },
    sourceViewBasis: "live_source",
    companionTab: "runs",
    selectedTraceRef: null,
  };
}

export function selectSourceWorkspace(
  current: AnalysisWorkspaceUiState,
  sourceId: number,
): AnalysisWorkspaceUiState {
  return {
    ...clearRunBoundWorkspaceState(current),
    workspaceSelection: { kind: "source", sourceId },
    canvasMode: "source",
  };
}

export function selectSourceGroupWorkspace(
  current: AnalysisWorkspaceUiState,
  sourceGroupId: number,
): AnalysisWorkspaceUiState {
  return {
    ...clearRunBoundWorkspaceState(current),
    workspaceSelection: { kind: "source_group", sourceGroupId },
    canvasMode: "source",
  };
}

export function normalizeRestoredWorkspaceState(
  state: AnalysisWorkspaceUiState,
): AnalysisWorkspaceUiState {
  if (state.openRunState.kind !== "none") {
    return state;
  }

  return {
    ...state,
    sourceViewBasis: "live_source",
    companionTab:
      state.companionTab === "evidence" ||
      state.companionTab === "chat" ||
      state.companionTab === "chunks"
        ? "runs"
        : state.companionTab,
    selectedTraceRef: null,
  };
}
