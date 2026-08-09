import type {
  AnalysisWorkspaceUiState,
  CanvasMode,
  CompanionTab,
  OpenRunState,
  SourceViewBasis,
  WorkspaceSelection,
} from "./analysis-workspace-state";

/**
 * The analysis route passes these state-owned props to its three workspace
 * zones. Keeping this projection here makes the route boundary observable
 * without reaching into Svelte source text.
 */
export interface AnalysisWorkspaceRouteProps {
  zones: readonly ["compact-source-rail", "report-canvas", "run-companion-tabs"];
  workspaceSelection: WorkspaceSelection;
  openRunState: OpenRunState;
  canvasMode: CanvasMode;
  sourceViewBasis: SourceViewBasis;
  companionTab: CompanionTab;
  sourceSwitcherOpen: false;
}

export function workspaceRouteProps(state: AnalysisWorkspaceUiState): AnalysisWorkspaceRouteProps {
  return {
    zones: ["compact-source-rail", "report-canvas", "run-companion-tabs"],
    workspaceSelection: state.workspaceSelection,
    openRunState: state.openRunState,
    canvasMode: state.canvasMode,
    sourceViewBasis: state.sourceViewBasis,
    companionTab: state.companionTab,
    sourceSwitcherOpen: false,
  };
}
