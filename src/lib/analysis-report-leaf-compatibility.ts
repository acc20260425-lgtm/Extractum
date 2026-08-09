import {
  legacyScopeFromWorkspaceSelection,
  type WorkspaceSelection,
} from "$lib/analysis-workspace-state";

export function reportSetupCompatibility(workspaceSelection: WorkspaceSelection) {
  const { analysisScope } = legacyScopeFromWorkspaceSelection(workspaceSelection);
  const sourceGroup = analysisScope === "source_group";
  return {
    analysisScope,
    workspaceEyebrow: sourceGroup ? "Source group workspace" : "Source workspace",
    analysisModeLabel: sourceGroup ? "Group analysis" : "Single source",
    runDescription: sourceGroup ? "Run across the saved group." : "Run against the selected source.",
  };
}

export function reportSourceCompatibility(workspaceSelection: WorkspaceSelection) {
  const { analysisScope } = legacyScopeFromWorkspaceSelection(workspaceSelection);
  const showSourceGroup = analysisScope === "source_group";
  return {
    readerSurfaceLabel: showSourceGroup ? "Group sources" : "Source material",
    showSingleSource: !showSourceGroup,
    showSourceGroup,
  };
}
