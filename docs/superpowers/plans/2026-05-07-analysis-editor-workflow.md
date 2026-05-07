# Analysis Editor Workflow Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move remaining Analysis template and source group create/update orchestration out of `src/routes/analysis/+page.svelte` into tested API/workflow boundaries.

**Architecture:** Extend the existing `analysis-source-groups` API and workflow instead of creating a parallel editor workflow. Keep Svelte state and UI composition in the route, while the workflow owns validation decisions, async command orchestration, reload/rebind behavior, busy flags, and status/error messages.

**Tech Stack:** Svelte 5, TypeScript, Vitest, Tauri invoke wrappers, existing Analysis workflow/state helpers.

---

## File Structure

- Modify `src/lib/api/analysis-source-groups.ts` to add wrappers for report-template listing, template create/update, and group create/update.
- Modify `src/lib/api/analysis-source-groups.test.ts` to cover every wrapper command name and payload.
- Modify `src/lib/analysis-source-groups-workflow.ts` to add template create/update and source group create/update methods.
- Modify `src/lib/analysis-source-groups-workflow.test.ts` to cover validation, success, and failure paths for the new workflow methods.
- Modify `src/routes/analysis/+page.svelte` to delegate load/save/copy actions to the workflow and remove raw Tauri invoke usage for these commands.
- Modify `docs/code-review-results-2026-05-03.md` and `docs/session-context-2026-05-03.md` after implementation is complete.

## Task 1: Add Editor API Wrappers

**Files:**
- Modify: `src/lib/api/analysis-source-groups.ts`
- Test: `src/lib/api/analysis-source-groups.test.ts`

- [ ] **Step 1: Add failing API wrapper tests**

In `src/lib/api/analysis-source-groups.test.ts`, extend imports to include:

```ts
import {
  createAnalysisPromptTemplate,
  createAnalysisSourceGroup,
  deleteAnalysisPromptTemplate,
  deleteAnalysisSourceGroup,
  listAnalysisPromptTemplates,
  listAnalysisSourceGroups,
  updateAnalysisPromptTemplate,
  updateAnalysisSourceGroup,
} from "./analysis-source-groups";
import type { AnalysisPromptTemplate, AnalysisSourceGroup } from "$lib/types/analysis";
```

Replace the existing `AnalysisSourceGroup`-only type import with the combined import above.

Add these tests after the existing group-load test:

```ts
it("loads prompt templates with the expected template kind payload", async () => {
  const templates: AnalysisPromptTemplate[] = [{
    id: 5,
    name: "Report",
    template_kind: "report",
    body: "Summarize",
    version: 1,
    is_builtin: false,
    created_at: 100,
    updated_at: 200,
  }];
  invokeMock.mockResolvedValueOnce(templates);

  await expect(listAnalysisPromptTemplates("report")).resolves.toEqual(templates);

  expect(invokeMock).toHaveBeenLastCalledWith("list_analysis_prompt_templates", {
    templateKind: "report",
  });
});

it("creates a prompt template with the expected payload", async () => {
  const created: AnalysisPromptTemplate = {
    id: 6,
    name: "Copy",
    template_kind: "report",
    body: "Body",
    version: 1,
    is_builtin: false,
    created_at: 100,
    updated_at: 200,
  };
  invokeMock.mockResolvedValueOnce(created);

  await expect(createAnalysisPromptTemplate({
    name: "Copy",
    templateKind: "report",
    body: "Body",
  })).resolves.toEqual(created);

  expect(invokeMock).toHaveBeenLastCalledWith("create_analysis_prompt_template", {
    name: "Copy",
    templateKind: "report",
    body: "Body",
  });
});

it("updates a prompt template with the expected payload", async () => {
  const updated: AnalysisPromptTemplate = {
    id: 6,
    name: "Updated",
    template_kind: "report",
    body: "Body",
    version: 2,
    is_builtin: false,
    created_at: 100,
    updated_at: 300,
  };
  invokeMock.mockResolvedValueOnce(updated);

  await expect(updateAnalysisPromptTemplate({
    templateId: 6,
    name: "Updated",
    body: "Body",
  })).resolves.toEqual(updated);

  expect(invokeMock).toHaveBeenLastCalledWith("update_analysis_prompt_template", {
    templateId: 6,
    name: "Updated",
    body: "Body",
  });
});
```

Add these tests before the delete-source-group test:

```ts
it("creates a source group with the expected payload", async () => {
  const created: AnalysisSourceGroup = {
    id: 12,
    name: "New group",
    members: [],
    created_at: 100,
    updated_at: 100,
  };
  invokeMock.mockResolvedValueOnce(created);

  await expect(createAnalysisSourceGroup({
    name: "New group",
    sourceIds: [3, 7],
  })).resolves.toEqual(created);

  expect(invokeMock).toHaveBeenLastCalledWith("create_analysis_source_group", {
    name: "New group",
    sourceIds: [3, 7],
  });
});

it("updates a source group with the expected payload", async () => {
  const updated: AnalysisSourceGroup = {
    id: 12,
    name: "Updated group",
    members: [],
    created_at: 100,
    updated_at: 200,
  };
  invokeMock.mockResolvedValueOnce(updated);

  await expect(updateAnalysisSourceGroup({
    groupId: 12,
    name: "Updated group",
    sourceIds: [7],
  })).resolves.toEqual(updated);

  expect(invokeMock).toHaveBeenLastCalledWith("update_analysis_source_group", {
    groupId: 12,
    name: "Updated group",
    sourceIds: [7],
  });
});
```

- [ ] **Step 2: Run the focused RED test**

Run:

```powershell
npm.cmd test -- src/lib/api/analysis-source-groups.test.ts
```

Expected: FAIL because `listAnalysisPromptTemplates`, `createAnalysisPromptTemplate`, `updateAnalysisPromptTemplate`, `createAnalysisSourceGroup`, and `updateAnalysisSourceGroup` are not exported yet.

- [ ] **Step 3: Add API wrapper functions**

In `src/lib/api/analysis-source-groups.ts`, update imports:

```ts
import type { AnalysisPromptTemplate, AnalysisSourceGroup } from "$lib/types/analysis";
```

Add interfaces after the import:

```ts
export interface CreateAnalysisPromptTemplateInput {
  name: string;
  templateKind: "report" | "chat";
  body: string;
}

export interface UpdateAnalysisPromptTemplateInput {
  templateId: number;
  name: string;
  body: string;
}

export interface CreateAnalysisSourceGroupInput {
  name: string;
  sourceIds: number[];
}

export interface UpdateAnalysisSourceGroupInput extends CreateAnalysisSourceGroupInput {
  groupId: number;
}
```

Add wrapper functions:

```ts
export function listAnalysisPromptTemplates(templateKind: "report" | "chat") {
  return invoke<AnalysisPromptTemplate[]>("list_analysis_prompt_templates", { templateKind });
}

export function createAnalysisPromptTemplate(input: CreateAnalysisPromptTemplateInput) {
  return invoke<AnalysisPromptTemplate>("create_analysis_prompt_template", { ...input });
}

export function updateAnalysisPromptTemplate(input: UpdateAnalysisPromptTemplateInput) {
  return invoke<AnalysisPromptTemplate>("update_analysis_prompt_template", { ...input });
}

export function createAnalysisSourceGroup(input: CreateAnalysisSourceGroupInput) {
  return invoke<AnalysisSourceGroup>("create_analysis_source_group", { ...input });
}

export function updateAnalysisSourceGroup(input: UpdateAnalysisSourceGroupInput) {
  return invoke<AnalysisSourceGroup>("update_analysis_source_group", { ...input });
}
```

Keep existing delete and list-group wrappers.

- [ ] **Step 4: Run focused GREEN verification**

Run:

```powershell
npm.cmd test -- src/lib/api/analysis-source-groups.test.ts
```

Expected: PASS.

- [ ] **Step 5: Commit Task 1**

Run:

```powershell
git add src/lib/api/analysis-source-groups.ts src/lib/api/analysis-source-groups.test.ts
git commit -m "refactor(analysis): add editor api wrappers"
```

## Task 2: Extract Template Create/Update Workflow

**Files:**
- Modify: `src/lib/analysis-source-groups-workflow.ts`
- Test: `src/lib/analysis-source-groups-workflow.test.ts`

- [ ] **Step 1: Add failing template workflow tests**

In `src/lib/analysis-source-groups-workflow.test.ts`, update `HarnessState`:

```ts
type HarnessState = AnalysisSourceGroupsWorkflowState & {
  groups: AnalysisSourceGroup[];
  loadingTemplates: boolean;
  loadingGroups: boolean;
  savingTemplate: boolean;
  deletingTemplate: boolean;
  deletingGroup: boolean;
  status: string;
};
```

Add `loadingTemplates: false` and `savingTemplate: false` to the default state.

Add deps:

```ts
listTemplates: vi.fn(),
createTemplate: vi.fn(),
updateTemplate: vi.fn(),
```

Add these tests before the existing delete-template tests:

```ts
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
```

- [ ] **Step 2: Run the focused RED test**

Run:

```powershell
npm.cmd test -- src/lib/analysis-source-groups-workflow.test.ts
```

Expected: FAIL because the workflow deps and methods for template save/copy do not exist yet.

- [ ] **Step 3: Extend workflow types and methods for templates**

In `src/lib/analysis-source-groups-workflow.ts`, extend imports from `analysis-editor-state`:

```ts
  templateCopyCommand,
  templateCreatedStatus,
  templateUpdateCommand,
  templateUpdatedStatus,
```

Add API input type imports:

```ts
import type {
  CreateAnalysisPromptTemplateInput,
  UpdateAnalysisPromptTemplateInput,
} from "$lib/api/analysis-source-groups";
```

Extend `AnalysisSourceGroupsWorkflowPatch` with:

```ts
  templates: AnalysisPromptTemplate[];
  loadingTemplates: boolean;
  savingTemplate: boolean;
```

Extend deps:

```ts
  listTemplates(templateKind: "report" | "chat"): Promise<AnalysisPromptTemplate[]>;
  createTemplate(input: CreateAnalysisPromptTemplateInput): Promise<AnalysisPromptTemplate>;
  updateTemplate(input: UpdateAnalysisPromptTemplateInput): Promise<AnalysisPromptTemplate>;
```

Add helper:

```ts
function selectedTemplateFrom(
  templates: AnalysisPromptTemplate[],
  selectedTemplateId: string,
) {
  if (!selectedTemplateId) {
    return templates[0] ?? null;
  }

  return templates.find((template) => template.id === Number(selectedTemplateId)) ?? null;
}
```

Add `loadTemplates` inside `createAnalysisSourceGroupsWorkflow`:

```ts
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
```

Add template methods:

```ts
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
```

Return the new methods:

```ts
return {
  loadTemplates,
  loadGroups,
  saveTemplateChanges,
  saveTemplateCopy,
  deleteTemplate,
  deleteGroup,
};
```

- [ ] **Step 4: Update test harness for template deps**

Ensure the workflow test harness `deps` object includes `listTemplates`, `createTemplate`, and `updateTemplate`, and that `patch` accepts the extended patch type.

- [ ] **Step 5: Run focused GREEN verification**

Run:

```powershell
npm.cmd test -- src/lib/analysis-source-groups-workflow.test.ts
```

Expected: PASS.

- [ ] **Step 6: Commit Task 2**

Run:

```powershell
git add src/lib/analysis-source-groups-workflow.ts src/lib/analysis-source-groups-workflow.test.ts
git commit -m "refactor(analysis): move template editor workflow"
```

## Task 3: Extract Source Group Create/Update Workflow

**Files:**
- Modify: `src/lib/analysis-source-groups-workflow.ts`
- Test: `src/lib/analysis-source-groups-workflow.test.ts`

- [ ] **Step 1: Add failing source group workflow tests**

In `src/lib/analysis-source-groups-workflow.test.ts`, extend `HarnessState` with:

```ts
savingGroup: boolean;
```

Add `savingGroup: false` to default state.

Add deps:

```ts
createGroup: vi.fn(),
updateGroup: vi.fn(),
```

Add these tests before existing delete-group tests:

```ts
it("patches status and skips update when group changes are invalid", async () => {
  const { state, deps, workflow } = createHarness({ selectedGroup: null });

  await workflow.saveGroupChanges("Name", [7]);

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

  await workflow.saveGroupChanges(" Updated ", [7]);

  expect(deps.updateGroup).toHaveBeenCalledWith({
    groupId: 10,
    name: "Updated",
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

  await workflow.saveGroupCopy(" New group ", [7]);

  expect(deps.createGroup).toHaveBeenCalledWith({
    name: "New group",
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

  await workflow.saveGroupChanges("Name", [7]);

  expect(state.status).toBe("Error saving the source group: backend down");
  expect(state.savingGroup).toBe(false);
});
```

- [ ] **Step 2: Run the focused RED test**

Run:

```powershell
npm.cmd test -- src/lib/analysis-source-groups-workflow.test.ts
```

Expected: FAIL because `saveGroupChanges`, `saveGroupCopy`, `createGroup`, and `updateGroup` workflow pieces are not implemented yet.

- [ ] **Step 3: Extend workflow types and methods for source groups**

In `src/lib/analysis-source-groups-workflow.ts`, extend imports from `analysis-editor-state`:

```ts
  groupCopyCommand,
  groupCreatedStatus,
  groupUpdateCommand,
  groupUpdatedStatus,
```

Extend API input type imports:

```ts
  CreateAnalysisSourceGroupInput,
  UpdateAnalysisSourceGroupInput,
```

Extend `AnalysisSourceGroupsWorkflowPatch` with:

```ts
savingGroup: boolean;
```

Extend deps:

```ts
createGroup(input: CreateAnalysisSourceGroupInput): Promise<AnalysisSourceGroup>;
updateGroup(input: UpdateAnalysisSourceGroupInput): Promise<AnalysisSourceGroup>;
```

Add source group methods:

```ts
async function saveGroupChanges(nextName: string, nextSourceIds: number[]) {
  const command = groupUpdateCommand(deps.getState().selectedGroup, nextName, nextSourceIds);
  if (!command.ok) {
    deps.patch({ status: command.status });
    return;
  }

  deps.patch({ savingGroup: true });
  try {
    const updated = await deps.updateGroup({
      groupId: command.groupId,
      name: command.name,
      sourceIds: command.sourceIds,
    });
    deps.patch({ status: groupUpdatedStatus(updated) });
    await loadGroups();
    deps.patch({ selectedGroupId: String(updated.id) });
    deps.bindGroupEditor(updated);
  } catch (error) {
    deps.patch({ status: deps.formatError("saving the source group", error) });
  } finally {
    deps.patch({ savingGroup: false });
  }
}

async function saveGroupCopy(nextName: string, nextSourceIds: number[]) {
  const command = groupCopyCommand(nextName, nextSourceIds);
  if (!command.ok) {
    deps.patch({ status: command.status });
    return;
  }

  deps.patch({ savingGroup: true });
  try {
    const created = await deps.createGroup({
      name: command.name,
      sourceIds: command.sourceIds,
    });
    deps.patch({ status: groupCreatedStatus(created) });
    await loadGroups();
    deps.patch({ selectedGroupId: String(created.id) });
    deps.bindGroupEditor(created);
  } catch (error) {
    deps.patch({ status: deps.formatError("creating the source group", error) });
  } finally {
    deps.patch({ savingGroup: false });
  }
}
```

Return the new methods:

```ts
return {
  loadTemplates,
  loadGroups,
  saveTemplateChanges,
  saveTemplateCopy,
  deleteTemplate,
  saveGroupChanges,
  saveGroupCopy,
  deleteGroup,
};
```

- [ ] **Step 4: Run focused GREEN verification**

Run:

```powershell
npm.cmd test -- src/lib/analysis-source-groups-workflow.test.ts
```

Expected: PASS.

- [ ] **Step 5: Commit Task 3**

Run:

```powershell
git add src/lib/analysis-source-groups-workflow.ts src/lib/analysis-source-groups-workflow.test.ts
git commit -m "refactor(analysis): move source group editor workflow"
```

## Task 4: Wire Route To Editor Workflow

**Files:**
- Modify: `src/routes/analysis/+page.svelte`
- Test: `src/lib/analysis-source-groups-workflow.test.ts`
- Test: `src/lib/api/analysis-source-groups.test.ts`

- [ ] **Step 1: Remove raw editor imports**

In `src/routes/analysis/+page.svelte`, remove:

```ts
import { invoke } from "@tauri-apps/api/core";
```

from the top-level imports if no other route code uses `invoke` after this task.

Replace the `analysis-source-groups` API import with:

```ts
  createAnalysisPromptTemplate,
  createAnalysisSourceGroup,
  deleteAnalysisPromptTemplate,
  deleteAnalysisSourceGroup,
  listAnalysisPromptTemplates,
  listAnalysisSourceGroups,
  updateAnalysisPromptTemplate,
  updateAnalysisSourceGroup,
```

From the `analysis-editor-state` import, remove:

```ts
groupCopyCommand,
groupCreatedStatus,
groupUpdateCommand,
groupUpdatedStatus,
templateCopyCommand,
templateCreatedStatus,
templateUpdateCommand,
templateUpdatedStatus,
```

Keep:

```ts
groupEditorStateFromGroup,
isGroupSourceSelected as groupSourceIsSelected,
templateEditorStateFromTemplate,
toggleGroupSourceSelection,
```

- [ ] **Step 2: Extend route workflow patch handling**

In `applySourceGroupsWorkflowPatch`, support new patch fields:

```ts
if ("templates" in patch) templates = patch.templates ?? [];
if ("loadingTemplates" in patch) loadingTemplates = patch.loadingTemplates ?? false;
if ("savingTemplate" in patch) savingTemplate = patch.savingTemplate ?? false;
if ("savingGroup" in patch) savingGroup = patch.savingGroup ?? false;
```

Keep existing group/delete/status patch handling.

- [ ] **Step 3: Replace route-local template loading**

Replace the existing `loadTemplates` body with:

```ts
async function loadTemplates() {
  await sourceGroupsWorkflow.loadTemplates();
}
```

- [ ] **Step 4: Extend workflow dependencies in the route**

In the `createAnalysisSourceGroupsWorkflow` call, add:

```ts
listTemplates: listAnalysisPromptTemplates,
createTemplate: createAnalysisPromptTemplate,
updateTemplate: updateAnalysisPromptTemplate,
createGroup: createAnalysisSourceGroup,
updateGroup: updateAnalysisSourceGroup,
```

Keep existing `listGroups`, delete dependencies, confirm, binders, and `formatError`.

- [ ] **Step 5: Replace route-local save/copy functions**

Replace `saveTemplateChanges` with:

```ts
async function saveTemplateChanges(nextName = templateName, nextBody = templateBody) {
  await sourceGroupsWorkflow.saveTemplateChanges(nextName, nextBody);
}
```

Replace `saveTemplateCopy` with:

```ts
async function saveTemplateCopy(nextName = templateName, nextBody = templateBody) {
  await sourceGroupsWorkflow.saveTemplateCopy(nextName, nextBody);
}
```

Replace `saveGroupChanges` with:

```ts
async function saveGroupChanges() {
  await sourceGroupsWorkflow.saveGroupChanges(groupName, groupMemberSourceIds);
}
```

Replace `saveGroupCopy` with:

```ts
async function saveGroupCopy() {
  await sourceGroupsWorkflow.saveGroupCopy(groupName, groupMemberSourceIds);
}
```

- [ ] **Step 6: Verify no raw editor command strings remain in the route**

Run:

```powershell
rg -n "create_analysis_prompt_template|update_analysis_prompt_template|create_analysis_source_group|update_analysis_source_group|list_analysis_prompt_templates|invoke<" src/routes/analysis/+page.svelte
```

Expected: no output and exit code 1.

- [ ] **Step 7: Run focused frontend verification**

Run:

```powershell
npm.cmd test -- src/lib/api/analysis-source-groups.test.ts src/lib/analysis-source-groups-workflow.test.ts
npm.cmd run check
```

Expected:

- API wrapper tests pass;
- source groups workflow tests pass;
- Svelte check reports 0 errors and 0 warnings.

- [ ] **Step 8: Commit Task 4**

Run:

```powershell
git add src/routes/analysis/+page.svelte
git commit -m "refactor(analysis): use editor workflow"
```

## Task 5: Refresh Review Docs and Session Handoff

**Files:**
- Modify: `docs/code-review-results-2026-05-03.md`
- Modify: `docs/session-context-2026-05-03.md`

- [ ] **Step 1: Update the review document**

In `docs/code-review-results-2026-05-03.md`, update the Major `/analysis` finding:

- move template create/update and source group create/update extraction into resolved work;
- describe the remaining route responsibilities as listener lifecycle and UI composition;
- remove this workstream from the first recommended follow-up.

Set the recommended follow-up order to:

```text
1. Add typed frontend API wrappers or shared DTO modules for remaining compact
   non-source Tauri command surfaces.
2. Opportunistically reduce lower-level Result<T, String> and classify_message
   fallback reliance when touching nearby backend code.
```

- [ ] **Step 2: Refresh the session handoff**

In `docs/session-context-2026-05-03.md`, record:

- completed analysis editor workflow extraction commits;
- verification commands and results;
- remaining recommended follow-up from the review document;
- current branch and clean/dirty state;
- that the no-worktree, one-task-per-turn workflow remains active.

- [ ] **Step 3: Run final verification**

Run:

```powershell
npm.cmd test
npm.cmd run check
git diff --check
```

Expected:

- `npm.cmd test` passes;
- `npm.cmd run check` reports 0 errors and 0 warnings;
- `git diff --check` exits 0.

- [ ] **Step 4: Commit Task 5**

Run:

```powershell
git add docs/code-review-results-2026-05-03.md docs/session-context-2026-05-03.md
git commit -m "docs(session): refresh analysis editor handoff"
```

## Execution Notes

- Use the existing workflow preference: no git worktree and inline execution.
- Execute exactly one top-level task per user turn.
- Commit at the end of each top-level task.
- If `npm.cmd test` or `npm.cmd run check` fails in the default sandbox with
  `spawn EPERM`, rerun the same command with approval outside the sandbox.
- Do not remove completed typed-error plan/spec files in this workstream.
