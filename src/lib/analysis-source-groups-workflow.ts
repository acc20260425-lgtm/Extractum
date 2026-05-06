import {
  groupDeleteDecision,
  groupDeletedStatus,
  groupFallbackSelection,
  templateDeleteDecision,
  templateDeletedStatus,
  templateFallbackSelection,
} from "$lib/analysis-editor-state";
import type { AnalysisPromptTemplate, AnalysisSourceGroup } from "$lib/types/analysis";

export interface AnalysisSourceGroupsWorkflowState {
  groups: AnalysisSourceGroup[];
  templates: AnalysisPromptTemplate[];
  selectedTemplate: AnalysisPromptTemplate | null;
  selectedGroup: AnalysisSourceGroup | null;
  selectedTemplateId: string;
  selectedGroupId: string;
  editorBoundTemplateId: number | null;
  editorBoundGroupId: number | null;
}

export type AnalysisSourceGroupsWorkflowPatch = Partial<{
  groups: AnalysisSourceGroup[];
  selectedTemplateId: string;
  selectedGroupId: string;
  loadingGroups: boolean;
  deletingTemplate: boolean;
  deletingGroup: boolean;
  status: string;
}>;

export interface ConfirmOptions {
  title: string;
  message: string;
  confirmLabel: string;
  cancelLabel: string;
  tone: "danger";
}

export interface AnalysisSourceGroupsWorkflowDeps {
  getState(): AnalysisSourceGroupsWorkflowState;
  patch(patch: AnalysisSourceGroupsWorkflowPatch): void;
  listGroups(): Promise<AnalysisSourceGroup[]>;
  deleteTemplate(templateId: number): Promise<void>;
  deleteGroup(groupId: number): Promise<void>;
  loadTemplates(): Promise<void>;
  confirm(options: ConfirmOptions): Promise<boolean>;
  bindTemplateEditor(template: AnalysisPromptTemplate | null): void;
  bindGroupEditor(group: AnalysisSourceGroup | null): void;
  formatError(action: string, error: unknown): string;
}

function selectedGroupFrom(groups: AnalysisSourceGroup[], selectedGroupId: string) {
  if (!selectedGroupId) {
    return groups[0] ?? null;
  }

  return groups.find((group) => group.id === Number(selectedGroupId)) ?? null;
}

export function createAnalysisSourceGroupsWorkflow(
  deps: AnalysisSourceGroupsWorkflowDeps,
) {
  async function loadGroups() {
    deps.patch({ loadingGroups: true });
    try {
      const groups = await deps.listGroups();
      const state = deps.getState();
      const selectedGroup = selectedGroupFrom(groups, state.selectedGroupId);
      deps.patch({
        groups,
        selectedGroupId: state.selectedGroupId || (selectedGroup ? String(selectedGroup.id) : ""),
      });
      if (selectedGroup && state.editorBoundGroupId !== selectedGroup.id) {
        deps.bindGroupEditor(selectedGroup);
      }
    } catch (error) {
      deps.patch({ status: deps.formatError("loading source groups", error) });
    } finally {
      deps.patch({ loadingGroups: false });
    }
  }

  async function deleteTemplate() {
    const decision = templateDeleteDecision(deps.getState().selectedTemplate);
    if (!decision.ok) {
      deps.patch({ status: decision.status });
      return;
    }

    const confirmed = await deps.confirm({
      title: "Delete template?",
      message: `The template "${decision.name}" will be removed from the local app.`,
      confirmLabel: "Delete",
      cancelLabel: "Cancel",
      tone: "danger",
    });
    if (!confirmed) {
      return;
    }

    deps.patch({ deletingTemplate: true });
    try {
      await deps.deleteTemplate(decision.templateId);
      deps.patch({ status: templateDeletedStatus(decision.name) });
      await deps.loadTemplates();
      const fallback = templateFallbackSelection(deps.getState().templates);
      deps.patch({ selectedTemplateId: fallback.selectedTemplateId });
      deps.bindTemplateEditor(fallback.template);
    } catch (error) {
      deps.patch({ status: deps.formatError("deleting the template", error) });
    } finally {
      deps.patch({ deletingTemplate: false });
    }
  }

  async function deleteGroup() {
    const decision = groupDeleteDecision(deps.getState().selectedGroup);
    if (!decision.ok) {
      deps.patch({ status: decision.status });
      return;
    }

    const confirmed = await deps.confirm({
      title: "Delete source group?",
      message: `The group "${decision.name}" will be removed, but its synced sources will stay available for analysis.`,
      confirmLabel: "Delete",
      cancelLabel: "Cancel",
      tone: "danger",
    });
    if (!confirmed) {
      return;
    }

    deps.patch({ deletingGroup: true });
    try {
      await deps.deleteGroup(decision.groupId);
      deps.patch({ status: groupDeletedStatus(decision.name) });
      await loadGroups();
      const fallback = groupFallbackSelection(deps.getState().groups);
      deps.patch({ selectedGroupId: fallback.selectedGroupId });
      deps.bindGroupEditor(fallback.group);
    } catch (error) {
      deps.patch({ status: deps.formatError("deleting the source group", error) });
    } finally {
      deps.patch({ deletingGroup: false });
    }
  }

  return {
    loadGroups,
    deleteTemplate,
    deleteGroup,
  };
}
