import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  createAnalysisSourceGroupsWorkflow,
  type AnalysisSourceGroupsWorkflowPatch,
  type AnalysisSourceGroupsWorkflowState,
} from "./analysis-source-groups-workflow";
import type { AnalysisPromptTemplate, AnalysisSourceGroup } from "./types/analysis";

function template(overrides: Partial<AnalysisPromptTemplate> = {}): AnalysisPromptTemplate {
  return {
    id: 1,
    name: "Daily",
    template_kind: "report",
    body: "Summarize",
    version: 1,
    is_builtin: false,
    created_at: 100,
    updated_at: 200,
    ...overrides,
  };
}

function group(overrides: Partial<AnalysisSourceGroup> = {}): AnalysisSourceGroup {
  return {
    id: 10,
    name: "Research",
    source_type: "telegram",
    members: [{
      source_id: 7,
      source_title: "Source",
      item_count: 12,
    }],
    created_at: 100,
    updated_at: 200,
    ...overrides,
  };
}

type HarnessState = AnalysisSourceGroupsWorkflowState & {
  groups: AnalysisSourceGroup[];
  loadingTemplates: boolean;
  loadingGroups: boolean;
  savingTemplate: boolean;
  savingGroup: boolean;
  deletingTemplate: boolean;
  deletingGroup: boolean;
  status: string;
};

function createHarness(initial: Partial<HarnessState> = {}) {
  const state: HarnessState = {
    groups: [],
    templates: [],
    selectedTemplate: null,
    selectedGroup: null,
    selectedTemplateId: "",
    selectedGroupId: "",
    editorBoundTemplateId: null,
    editorBoundGroupId: null,
    loadingTemplates: false,
    loadingGroups: false,
    savingTemplate: false,
    savingGroup: false,
    deletingTemplate: false,
    deletingGroup: false,
    status: "",
    ...initial,
  };

  const deps = {
    getState: () => state,
    patch: vi.fn((patch: AnalysisSourceGroupsWorkflowPatch) => Object.assign(state, patch)),
    listTemplates: vi.fn(),
    listGroups: vi.fn(),
    createTemplate: vi.fn(),
    updateTemplate: vi.fn(),
    createGroup: vi.fn(),
    updateGroup: vi.fn(),
    deleteTemplate: vi.fn(),
    deleteGroup: vi.fn(),
    loadTemplates: vi.fn(),
    confirm: vi.fn(),
    bindTemplateEditor: vi.fn(),
    bindGroupEditor: vi.fn(),
    formatError: vi.fn((action: string, error: unknown) => `Error ${action}: ${String(error)}`),
  };

  return { state, deps, workflow: createAnalysisSourceGroupsWorkflow(deps) };
}

describe("analysis-source-groups-workflow", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it("loads groups and selects the first group when no group is selected", async () => {
    const first = group({ id: 10 });
    const second = group({ id: 11, name: "Ops" });
    const { state, deps, workflow } = createHarness();
    deps.listGroups.mockResolvedValueOnce([first, second]);

    await workflow.loadGroups();

    expect(state.groups).toEqual([first, second]);
    expect(state.selectedGroupId).toBe("10");
    expect(deps.bindGroupEditor).toHaveBeenCalledWith(first);
    expect(state.loadingGroups).toBe(false);
  });

  it("preserves the selected group and binds it when the editor is stale", async () => {
    const selected = group({ id: 11, name: "Ops" });
    const { state, deps, workflow } = createHarness({
      selectedGroupId: "11",
      editorBoundGroupId: 10,
    });
    deps.listGroups.mockResolvedValueOnce([group({ id: 10 }), selected]);

    await workflow.loadGroups();

    expect(state.selectedGroupId).toBe("11");
    expect(deps.bindGroupEditor).toHaveBeenCalledWith(selected);
  });

  it("reports group loading errors and clears the loading flag", async () => {
    const { state, deps, workflow } = createHarness();
    deps.listGroups.mockRejectedValueOnce("db down");

    await workflow.loadGroups();

    expect(state.status).toBe("Error loading source groups: db down");
    expect(state.loadingGroups).toBe(false);
  });

  it("patches status and skips update when template changes are invalid", async () => {
    const { state, deps, workflow } = createHarness({ selectedTemplate: null });

    await workflow.saveTemplateChanges("Name", "Body");

    expect(state.status).toBe("Select a template first.");
    expect(deps.updateTemplate).not.toHaveBeenCalled();
    expect(state.savingTemplate).toBe(false);
  });

  it("updates a template, reloads templates, selects it, and rebinds the editor", async () => {
    const current = template({ id: 42, name: "Custom" });
    const updated = template({ id: 42, name: "Updated", body: "New body" });
    const { state, deps, workflow } = createHarness({
      selectedTemplate: current,
      templates: [current],
    });
    deps.updateTemplate.mockResolvedValueOnce(updated);
    deps.listTemplates.mockResolvedValueOnce([updated]);

    await workflow.saveTemplateChanges(" Updated ", " New body ");

    expect(deps.updateTemplate).toHaveBeenCalledWith({
      templateId: 42,
      name: "Updated",
      body: "New body",
    });
    expect(deps.listTemplates).toHaveBeenCalledWith("report");
    expect(state.templates).toEqual([updated]);
    expect(state.selectedTemplateId).toBe("42");
    expect(state.status).toBe("Template \"Updated\" saved.");
    expect(deps.bindTemplateEditor).toHaveBeenCalledWith(updated);
    expect(state.savingTemplate).toBe(false);
  });

  it("creates a template copy, reloads templates, selects it, and rebinds the editor", async () => {
    const created = template({ id: 77, name: "Copy", body: "Copied body" });
    const { state, deps, workflow } = createHarness();
    deps.createTemplate.mockResolvedValueOnce(created);
    deps.listTemplates.mockResolvedValueOnce([created]);

    await workflow.saveTemplateCopy(" Copy ", " Copied body ");

    expect(deps.createTemplate).toHaveBeenCalledWith({
      name: "Copy",
      templateKind: "report",
      body: "Copied body",
    });
    expect(deps.listTemplates).toHaveBeenCalledWith("report");
    expect(state.templates).toEqual([created]);
    expect(state.selectedTemplateId).toBe("77");
    expect(state.status).toBe("Template \"Copy\" created.");
    expect(deps.bindTemplateEditor).toHaveBeenCalledWith(created);
    expect(state.savingTemplate).toBe(false);
  });

  it("reports template save errors and clears saving state", async () => {
    const current = template({ id: 42, name: "Custom" });
    const { state, deps, workflow } = createHarness({ selectedTemplate: current });
    deps.updateTemplate.mockRejectedValueOnce("backend down");

    await workflow.saveTemplateChanges("Name", "Body");

    expect(state.status).toBe("Error saving the template: backend down");
    expect(state.savingTemplate).toBe(false);
  });

  it("patches status and skips confirmation when template deletion is invalid", async () => {
    const { state, deps, workflow } = createHarness({ selectedTemplate: null });

    await workflow.deleteTemplate();

    expect(state.status).toBe("Select a template first.");
    expect(deps.confirm).not.toHaveBeenCalled();
    expect(deps.deleteTemplate).not.toHaveBeenCalled();
  });

  it("exits template deletion when confirmation is cancelled", async () => {
    const current = template({ id: 42, name: "Custom" });
    const { state, deps, workflow } = createHarness({ selectedTemplate: current });
    deps.confirm.mockResolvedValueOnce(false);

    await workflow.deleteTemplate();

    expect(deps.confirm).toHaveBeenCalledWith({
      title: "Delete template?",
      message: "The template \"Custom\" will be removed from the local app.",
      confirmLabel: "Delete",
      cancelLabel: "Cancel",
      tone: "danger",
    });
    expect(deps.deleteTemplate).not.toHaveBeenCalled();
    expect(state.deletingTemplate).toBe(false);
  });

  it("deletes a template, reloads templates, and applies fallback selection", async () => {
    const current = template({ id: 42, name: "Custom" });
    const fallback = template({ id: 7, name: "Fallback" });
    const { state, deps, workflow } = createHarness({
      templates: [fallback],
      selectedTemplate: current,
    });
    deps.confirm.mockResolvedValueOnce(true);
    deps.deleteTemplate.mockResolvedValueOnce(undefined);
    deps.listTemplates.mockResolvedValueOnce([fallback]);

    await workflow.deleteTemplate();

    expect(deps.deleteTemplate).toHaveBeenCalledWith(42);
    expect(deps.listTemplates).toHaveBeenCalledWith("report");
    expect(state.status).toBe("Template \"Custom\" deleted.");
    expect(state.selectedTemplateId).toBe("7");
    expect(deps.bindTemplateEditor).toHaveBeenCalledWith(fallback);
    expect(state.deletingTemplate).toBe(false);
  });

  it("patches status and skips update when group changes are invalid", async () => {
    const { state, deps, workflow } = createHarness({ selectedGroup: null });

    await workflow.saveGroupChanges("Name", [7], "telegram");

    expect(state.status).toBe("Select a source group first.");
    expect(deps.updateGroup).not.toHaveBeenCalled();
    expect(state.savingGroup).toBe(false);
  });

  it("updates a group, reloads groups, selects it, and rebinds the editor", async () => {
    const current = group({ id: 10, name: "Research" });
    const updated = group({ id: 10, name: "Updated" });
    const { state, deps, workflow } = createHarness({
      selectedGroup: current,
      groups: [current],
    });
    deps.updateGroup.mockResolvedValueOnce(updated);
    deps.listGroups.mockResolvedValueOnce([updated]);

    await workflow.saveGroupChanges(" Updated ", [7], "youtube");

    expect(deps.updateGroup).toHaveBeenCalledWith({
      groupId: 10,
      name: "Updated",
      sourceType: "youtube",
      sourceIds: [7],
    });
    expect(deps.listGroups).toHaveBeenCalledOnce();
    expect(state.groups).toEqual([updated]);
    expect(state.selectedGroupId).toBe("10");
    expect(state.status).toBe("Source group \"Updated\" saved.");
    expect(deps.bindGroupEditor).toHaveBeenLastCalledWith(updated);
    expect(state.savingGroup).toBe(false);
  });

  it("creates a group copy, reloads groups, selects it, and rebinds the editor", async () => {
    const created = group({ id: 33, name: "New group" });
    const { state, deps, workflow } = createHarness();
    deps.createGroup.mockResolvedValueOnce(created);
    deps.listGroups.mockResolvedValueOnce([created]);

    await workflow.saveGroupCopy(" New group ", [7], "telegram");

    expect(deps.createGroup).toHaveBeenCalledWith({
      name: "New group",
      sourceType: "telegram",
      sourceIds: [7],
    });
    expect(deps.listGroups).toHaveBeenCalledOnce();
    expect(state.groups).toEqual([created]);
    expect(state.selectedGroupId).toBe("33");
    expect(state.status).toBe("Source group \"New group\" created.");
    expect(deps.bindGroupEditor).toHaveBeenLastCalledWith(created);
    expect(state.savingGroup).toBe(false);
  });

  it("reports group save errors and clears saving state", async () => {
    const current = group({ id: 10, name: "Research" });
    const { state, deps, workflow } = createHarness({ selectedGroup: current });
    deps.updateGroup.mockRejectedValueOnce("backend down");

    await workflow.saveGroupChanges("Name", [7], "telegram");

    expect(state.status).toBe("Error saving the source group: backend down");
    expect(state.savingGroup).toBe(false);
  });

  it("deletes a group, reloads groups, and applies fallback selection", async () => {
    const current = group({ id: 10, name: "Research" });
    const fallback = group({ id: 11, name: "Ops" });
    const { state, deps, workflow } = createHarness({
      groups: [fallback],
      selectedGroup: current,
    });
    deps.confirm.mockResolvedValueOnce(true);
    deps.deleteGroup.mockResolvedValueOnce(undefined);
    deps.listGroups.mockResolvedValueOnce([fallback]);

    await workflow.deleteGroup();

    expect(deps.deleteGroup).toHaveBeenCalledWith(10);
    expect(deps.listGroups).toHaveBeenCalledOnce();
    expect(state.status).toBe("Source group \"Research\" deleted.");
    expect(state.selectedGroupId).toBe("11");
    expect(deps.bindGroupEditor).toHaveBeenLastCalledWith(fallback);
    expect(state.deletingGroup).toBe(false);
  });

  it("reports deletion errors and clears deleting state", async () => {
    const current = group({ id: 10, name: "Research" });
    const { state, deps, workflow } = createHarness({ selectedGroup: current });
    deps.confirm.mockResolvedValueOnce(true);
    deps.deleteGroup.mockRejectedValueOnce("backend down");

    await workflow.deleteGroup();

    expect(state.status).toBe("Error deleting the source group: backend down");
    expect(state.deletingGroup).toBe(false);
  });
});
