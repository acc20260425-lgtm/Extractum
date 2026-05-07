import {
  groupDeleteDecision,
  groupDeletedStatus,
  groupFallbackSelection,
  templateCopyCommand,
  templateCreatedStatus,
  templateDeleteDecision,
  templateDeletedStatus,
  templateFallbackSelection,
  templateUpdateCommand,
  templateUpdatedStatus,
} from "$lib/analysis-editor-state";
import type {
  CreateAnalysisPromptTemplateInput,
  UpdateAnalysisPromptTemplateInput,
} from "$lib/api/analysis-source-groups";
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
  templates: AnalysisPromptTemplate[];
  groups: AnalysisSourceGroup[];
  selectedTemplateId: string;
  selectedGroupId: string;
  loadingTemplates: boolean;
  loadingGroups: boolean;
  savingTemplate: boolean;
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
  listTemplates(templateKind: "report" | "chat"): Promise<AnalysisPromptTemplate[]>;
  listGroups(): Promise<AnalysisSourceGroup[]>;
  createTemplate(input: CreateAnalysisPromptTemplateInput): Promise<AnalysisPromptTemplate>;
  updateTemplate(input: UpdateAnalysisPromptTemplateInput): Promise<AnalysisPromptTemplate>;
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

function selectedTemplateFrom(
  templates: AnalysisPromptTemplate[],
  selectedTemplateId: string,
) {
  if (!selectedTemplateId) {
    return templates[0] ?? null;
  }

  return templates.find((template) => template.id === Number(selectedTemplateId)) ?? null;
}

export function createAnalysisSourceGroupsWorkflow(
  deps: AnalysisSourceGroupsWorkflowDeps,
) {
  async function loadTemplates() {
    deps.patch({ loadingTemplates: true });
    try {
      const templates = await deps.listTemplates("report");
      const state = deps.getState();
      const selectedTemplate = selectedTemplateFrom(templates, state.selectedTemplateId);
      deps.patch({
        templates,
        selectedTemplateId: state.selectedTemplateId || (selectedTemplate ? String(selectedTemplate.id) : ""),
      });
      if (selectedTemplate && state.editorBoundTemplateId !== selectedTemplate.id) {
        deps.bindTemplateEditor(selectedTemplate);
      }
    } catch (error) {
      deps.patch({ status: deps.formatError("loading report templates", error) });
    } finally {
      deps.patch({ loadingTemplates: false });
    }
  }

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

  async function saveTemplateChanges(nextName: string, nextBody: string) {
    const command = templateUpdateCommand(deps.getState().selectedTemplate, nextName, nextBody);
    if (!command.ok) {
      deps.patch({ status: command.status });
      return;
    }

    deps.patch({ savingTemplate: true });
    try {
      const updated = await deps.updateTemplate({
        templateId: command.templateId,
        name: command.name,
        body: command.body,
      });
      deps.patch({ status: templateUpdatedStatus(updated) });
      await loadTemplates();
      deps.patch({ selectedTemplateId: String(updated.id) });
      deps.bindTemplateEditor(updated);
    } catch (error) {
      deps.patch({ status: deps.formatError("saving the template", error) });
    } finally {
      deps.patch({ savingTemplate: false });
    }
  }

  async function saveTemplateCopy(nextName: string, nextBody: string) {
    const command = templateCopyCommand(nextName, nextBody);
    if (!command.ok) {
      deps.patch({ status: command.status });
      return;
    }

    deps.patch({ savingTemplate: true });
    try {
      const created = await deps.createTemplate({
        name: command.name,
        templateKind: "report",
        body: command.body,
      });
      deps.patch({ status: templateCreatedStatus(created) });
      await loadTemplates();
      deps.patch({ selectedTemplateId: String(created.id) });
      deps.bindTemplateEditor(created);
    } catch (error) {
      deps.patch({ status: deps.formatError("creating the template", error) });
    } finally {
      deps.patch({ savingTemplate: false });
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
    loadTemplates,
    loadGroups,
    saveTemplateChanges,
    saveTemplateCopy,
    deleteTemplate,
    deleteGroup,
  };
}
