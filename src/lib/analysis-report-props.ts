import type { WorkspaceSelection } from "$lib/analysis-workspace-state";

export function reportCanvasWorkspaceProps(workspaceSelection: WorkspaceSelection) {
  return { workspaceSelection };
}

export function reportSetupProps<TTemplate>({
  workspaceSelection,
  selectedTemplate,
  reportLaunchDisabledReason,
  onRunReport,
  onSyncCurrentSource,
}: {
  workspaceSelection: WorkspaceSelection;
  selectedTemplate: TTemplate;
  reportLaunchDisabledReason: string | null;
  onRunReport: () => void;
  onSyncCurrentSource: (sourceId: number) => void;
}) {
  return {
    workspaceSelection,
    selectedTemplate,
    reportLaunchDisabledReason,
    onRunReport,
    onSyncCurrentSource,
  };
}
