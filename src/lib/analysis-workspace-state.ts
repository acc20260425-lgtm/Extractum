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

export type AnalysisWorkspaceEvent =
  | { type: "open_run"; run: RunWorkspaceInput }
  | { type: "select_source"; sourceId: number }
  | { type: "select_source_group"; sourceGroupId: number }
  | { type: "change_canvas_mode"; canvasMode: CanvasMode }
  | { type: "view_live_source_for_opened_run" }
  | { type: "switch_source_basis_to_run_snapshot" }
  | { type: "change_companion_tab"; companionTab: CompanionTab }
  | {
      type: "show_evidence_in_source";
      sourceViewBasis: SourceViewBasis;
      highlightedRef: string;
    }
  | { type: "return_to_evidence_review"; traceRef: string }
  | { type: "restore_persisted_state"; state: AnalysisWorkspaceUiState };

function numericId(value: string) {
  if (!value.trim()) return null;
  const parsed = Number(value);
  return Number.isInteger(parsed) && parsed > 0 ? parsed : null;
}

function isRunBoundCompanionTab(tab: CompanionTab) {
  return tab === "evidence" || tab === "chat" || tab === "chunks";
}

function normalizeWorkspaceState(state: AnalysisWorkspaceUiState): AnalysisWorkspaceUiState {
  if (state.openRunState.kind !== "none") {
    return state;
  }

  return {
    ...state,
    sourceViewBasis: "live_source",
    companionTab: isRunBoundCompanionTab(state.companionTab) ? "runs" : state.companionTab,
    selectedTraceRef: null,
  };
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
  return transitionAnalysisWorkspaceState(current, { type: "open_run", run });
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
  return transitionAnalysisWorkspaceState(current, { type: "select_source", sourceId });
}

export function selectSourceGroupWorkspace(
  current: AnalysisWorkspaceUiState,
  sourceGroupId: number,
): AnalysisWorkspaceUiState {
  return transitionAnalysisWorkspaceState(current, {
    type: "select_source_group",
    sourceGroupId,
  });
}

export function normalizeRestoredWorkspaceState(
  state: AnalysisWorkspaceUiState,
): AnalysisWorkspaceUiState {
  return transitionAnalysisWorkspaceState(defaultAnalysisWorkspaceUiState(), {
    type: "restore_persisted_state",
    state,
  });
}

export function transitionAnalysisWorkspaceState(
  current: AnalysisWorkspaceUiState,
  event: AnalysisWorkspaceEvent,
): AnalysisWorkspaceUiState {
  switch (event.type) {
    case "open_run":
      return {
        ...current,
        workspaceSelection: workspaceSelectionFromRunScope(event.run),
        openRunState: openRunStateForStatus(event.run.status, event.run.runId),
        canvasMode: "report",
        sourceViewBasis: "run_snapshot",
        companionTab: defaultCompanionTabForRun(event.run.status),
        selectedTraceRef: null,
      };

    case "select_source":
      return {
        ...clearRunBoundWorkspaceState(current),
        workspaceSelection: { kind: "source", sourceId: event.sourceId },
        canvasMode: "source",
      };

    case "select_source_group":
      return {
        ...clearRunBoundWorkspaceState(current),
        workspaceSelection: { kind: "source_group", sourceGroupId: event.sourceGroupId },
        canvasMode: "source",
      };

    case "change_canvas_mode":
      return normalizeWorkspaceState({
        ...current,
        canvasMode: event.canvasMode,
      });

    case "view_live_source_for_opened_run":
      return {
        ...current,
        canvasMode: "source",
        sourceViewBasis: "live_source",
      };

    case "switch_source_basis_to_run_snapshot":
      if (current.openRunState.kind === "none") {
        return normalizeWorkspaceState({
          ...current,
          canvasMode: "source",
        });
      }

      return {
        ...current,
        canvasMode: "source",
        sourceViewBasis: "run_snapshot",
      };

    case "change_companion_tab":
      return normalizeWorkspaceState({
        ...current,
        companionTab: event.companionTab,
      });

    case "show_evidence_in_source":
      if (current.openRunState.kind === "none") {
        return normalizeWorkspaceState({
          ...current,
          canvasMode: "source",
          sourceViewBasis: event.sourceViewBasis,
        });
      }

      return {
        ...current,
        canvasMode: "source",
        sourceViewBasis: event.sourceViewBasis,
        companionTab: "evidence",
        selectedTraceRef: event.highlightedRef,
      };

    case "return_to_evidence_review":
      if (current.openRunState.kind === "none") {
        return normalizeWorkspaceState(current);
      }

      return {
        ...current,
        canvasMode: "report",
        companionTab: "evidence",
        selectedTraceRef: event.traceRef,
      };

    case "restore_persisted_state":
      return normalizeWorkspaceState(event.state);
  }
}
