# Analysis Source Groups Cleanup Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move analysis source group loading and template/group deletion command orchestration out of `src/routes/analysis/+page.svelte`.

**Architecture:** Add a small typed Tauri API wrapper for source-group and deletion commands, then add a framework-independent workflow that owns loading/deleting state transitions and fallback selection. The Svelte route remains the `$state`, modal, and UI composition layer.

**Tech Stack:** Svelte 5, TypeScript, Tauri `invoke`, Vitest, existing `$lib/types/analysis` and `$lib/analysis-editor-state` helpers.

---

### File Structure

- Create: `src/lib/api/analysis-source-groups.ts`
  - Owns `list_analysis_source_groups`, `delete_analysis_prompt_template`, and `delete_analysis_source_group` command calls.
- Create: `src/lib/api/analysis-source-groups.test.ts`
  - Verifies exact Tauri command names and argument payloads.
- Create: `src/lib/analysis-source-groups-workflow.ts`
  - Owns route-independent orchestration for `loadGroups()`, `deleteTemplate()`, and `deleteGroup()`.
- Create: `src/lib/analysis-source-groups-workflow.test.ts`
  - Verifies state patches, fallback selection, confirmation behavior, API calls, and error handling.
- Modify: `src/routes/analysis/+page.svelte`
  - Wires the new API and workflow into existing `$state` variables and event handlers.
- Modify near completion: `docs/code-review-results-2026-05-03.md`
  - Marks this cleanup slice as resolved and updates remaining raw command surfaces.
- Modify near completion: `docs/session-context-2026-05-03.md`
  - Refreshes handoff with this workstream status and next recommended slice.

---

### Task 1: Add Analysis Source Groups API Wrapper

**Files:**
- Create: `src/lib/api/analysis-source-groups.ts`
- Create: `src/lib/api/analysis-source-groups.test.ts`

- [ ] **Step 1: Write the failing API wrapper tests**

Create `src/lib/api/analysis-source-groups.test.ts`:

```ts
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  deleteAnalysisPromptTemplate,
  deleteAnalysisSourceGroup,
  listAnalysisSourceGroups,
} from "./analysis-source-groups";
import type { AnalysisSourceGroup } from "$lib/types/analysis";

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

describe("analysis source groups api wrappers", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("loads analysis source groups with the registered command name", async () => {
    const groups: AnalysisSourceGroup[] = [{
      id: 10,
      name: "Research",
      members: [{
        source_id: 7,
        source_title: "Source",
        item_count: 12,
      }],
      created_at: 100,
      updated_at: 200,
    }];
    invokeMock.mockResolvedValueOnce(groups);

    await expect(listAnalysisSourceGroups()).resolves.toEqual(groups);

    expect(invokeMock).toHaveBeenLastCalledWith("list_analysis_source_groups");
  });

  it("deletes a prompt template with the expected payload", async () => {
    invokeMock.mockResolvedValueOnce(undefined);

    await expect(deleteAnalysisPromptTemplate(42)).resolves.toBeUndefined();

    expect(invokeMock).toHaveBeenLastCalledWith("delete_analysis_prompt_template", {
      templateId: 42,
    });
  });

  it("deletes a source group with the expected payload", async () => {
    invokeMock.mockResolvedValueOnce(undefined);

    await expect(deleteAnalysisSourceGroup(9)).resolves.toBeUndefined();

    expect(invokeMock).toHaveBeenLastCalledWith("delete_analysis_source_group", {
      groupId: 9,
    });
  });
});
```

- [ ] **Step 2: Run the focused test to verify it fails**

Run:

```powershell
npm.cmd test -- src/lib/api/analysis-source-groups.test.ts
```

Expected: FAIL because `src/lib/api/analysis-source-groups.ts` does not exist.

- [ ] **Step 3: Add the API wrapper implementation**

Create `src/lib/api/analysis-source-groups.ts`:

```ts
import { invoke } from "@tauri-apps/api/core";
import type { AnalysisSourceGroup } from "$lib/types/analysis";

export function listAnalysisSourceGroups() {
  return invoke<AnalysisSourceGroup[]>("list_analysis_source_groups");
}

export function deleteAnalysisPromptTemplate(templateId: number) {
  return invoke<void>("delete_analysis_prompt_template", { templateId });
}

export function deleteAnalysisSourceGroup(groupId: number) {
  return invoke<void>("delete_analysis_source_group", { groupId });
}
```

- [ ] **Step 4: Run the focused API test to verify it passes**

Run:

```powershell
npm.cmd test -- src/lib/api/analysis-source-groups.test.ts
```

Expected: PASS with 1 test file and 3 tests.

- [ ] **Step 5: Commit Task 1**

Run:

```powershell
git add src/lib/api/analysis-source-groups.ts src/lib/api/analysis-source-groups.test.ts
git commit -m "refactor(analysis): add source groups api wrapper"
```

---

### Task 2: Extract Source Groups Workflow

**Files:**
- Create: `src/lib/analysis-source-groups-workflow.ts`
- Create: `src/lib/analysis-source-groups-workflow.test.ts`

- [ ] **Step 1: Write the failing workflow tests**

Create `src/lib/analysis-source-groups-workflow.test.ts`:

```ts
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
  loadingGroups: boolean;
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
    loadingGroups: false,
    deletingTemplate: false,
    deletingGroup: false,
    status: "",
    ...initial,
  };

  const deps = {
    getState: () => state,
    patch: vi.fn((patch: AnalysisSourceGroupsWorkflowPatch) => Object.assign(state, patch)),
    listGroups: vi.fn(),
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
    deps.loadTemplates.mockImplementationOnce(async () => {
      state.templates = [fallback];
    });

    await workflow.deleteTemplate();

    expect(deps.deleteTemplate).toHaveBeenCalledWith(42);
    expect(deps.loadTemplates).toHaveBeenCalledOnce();
    expect(state.status).toBe("Template \"Custom\" deleted.");
    expect(state.selectedTemplateId).toBe("7");
    expect(deps.bindTemplateEditor).toHaveBeenCalledWith(fallback);
    expect(state.deletingTemplate).toBe(false);
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
```

- [ ] **Step 2: Run the focused workflow test to verify it fails**

Run:

```powershell
npm.cmd test -- src/lib/analysis-source-groups-workflow.test.ts
```

Expected: FAIL because `src/lib/analysis-source-groups-workflow.ts` does not exist.

- [ ] **Step 3: Add the workflow implementation**

Create `src/lib/analysis-source-groups-workflow.ts`:

```ts
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
```

- [ ] **Step 4: Run the focused workflow test to verify it passes**

Run:

```powershell
npm.cmd test -- src/lib/analysis-source-groups-workflow.test.ts
```

Expected: PASS with 1 test file and 8 tests.

- [ ] **Step 5: Run both new focused test files**

Run:

```powershell
npm.cmd test -- src/lib/api/analysis-source-groups.test.ts src/lib/analysis-source-groups-workflow.test.ts
```

Expected: PASS with 2 test files and 11 tests.

- [ ] **Step 6: Commit Task 2**

Run:

```powershell
git add src/lib/analysis-source-groups-workflow.ts src/lib/analysis-source-groups-workflow.test.ts
git commit -m "refactor(analysis): extract source groups workflow"
```

---

### Task 3: Wire Workflow Into Analysis Route

**Files:**
- Modify: `src/routes/analysis/+page.svelte`

- [ ] **Step 1: Add route imports**

In `src/routes/analysis/+page.svelte`, add API imports near the other `$lib/api/*` imports:

```ts
  import {
    deleteAnalysisPromptTemplate,
    deleteAnalysisSourceGroup,
    listAnalysisSourceGroups,
  } from "$lib/api/analysis-source-groups";
```

Add workflow import near the other workflow imports:

```ts
  import {
    createAnalysisSourceGroupsWorkflow,
    type AnalysisSourceGroupsWorkflowPatch,
  } from "$lib/analysis-source-groups-workflow";
```

- [ ] **Step 2: Add the workflow patch helper**

Near the existing route-local patch helpers, add:

```ts
  function applySourceGroupsWorkflowPatch(patch: AnalysisSourceGroupsWorkflowPatch) {
    if (patch.groups !== undefined) groups = patch.groups;
    if (patch.selectedTemplateId !== undefined) selectedTemplateId = patch.selectedTemplateId;
    if (patch.selectedGroupId !== undefined) selectedGroupId = patch.selectedGroupId;
    if (patch.loadingGroups !== undefined) loadingGroups = patch.loadingGroups;
    if (patch.deletingTemplate !== undefined) deletingTemplate = patch.deletingTemplate;
    if (patch.deletingGroup !== undefined) deletingGroup = patch.deletingGroup;
    if (patch.status !== undefined) status = patch.status;
  }
```

- [ ] **Step 3: Instantiate the workflow**

After `loadTemplates()`, `bindEditorToTemplate()`, and `bindEditorToGroup()` are in scope, instantiate:

```ts
  const sourceGroupsWorkflow = createAnalysisSourceGroupsWorkflow({
    getState: () => ({
      groups,
      templates,
      selectedTemplate,
      selectedGroup,
      selectedTemplateId,
      selectedGroupId,
      editorBoundTemplateId,
      editorBoundGroupId,
    }),
    patch: applySourceGroupsWorkflowPatch,
    listGroups: listAnalysisSourceGroups,
    deleteTemplate: deleteAnalysisPromptTemplate,
    deleteGroup: deleteAnalysisSourceGroup,
    loadTemplates,
    confirm: openConfirmModal,
    bindTemplateEditor: bindEditorToTemplate,
    bindGroupEditor: bindEditorToGroup,
    formatError: formatAppError,
  });
```

If Svelte reports a temporal-dead-zone issue because this appears before function declarations that are referenced in the object literal, place the `const sourceGroupsWorkflow = ...` block after the referenced function declarations and before the event handlers that call it.

- [ ] **Step 4: Replace route-local `loadGroups()` body**

Replace the current body with:

```ts
  async function loadGroups() {
    await sourceGroupsWorkflow.loadGroups();
  }
```

- [ ] **Step 5: Replace route-local `deleteTemplate()` body**

Replace the current body with:

```ts
  async function deleteTemplate() {
    await sourceGroupsWorkflow.deleteTemplate();
  }
```

- [ ] **Step 6: Replace route-local `deleteGroup()` body**

Replace the current body with:

```ts
  async function deleteGroup() {
    await sourceGroupsWorkflow.deleteGroup();
  }
```

- [ ] **Step 7: Verify raw command strings are gone from the route**

Run:

```powershell
rg "list_analysis_source_groups|delete_analysis_prompt_template|delete_analysis_source_group" src/routes/analysis/+page.svelte
```

Expected: no output.

- [ ] **Step 8: Run focused tests**

Run:

```powershell
npm.cmd test -- src/lib/api/analysis-source-groups.test.ts src/lib/analysis-source-groups-workflow.test.ts
```

Expected: PASS with 2 test files and 11 tests.

- [ ] **Step 9: Run Svelte check**

Run:

```powershell
npm.cmd run check
```

Expected: PASS with `svelte-check found 0 errors and 0 warnings`.

- [ ] **Step 10: Commit Task 3**

Run:

```powershell
git add src/routes/analysis/+page.svelte
git commit -m "refactor(analysis): use source groups workflow"
```

---

### Task 4: Refresh Cleanup Documentation And Verify

**Files:**
- Modify: `docs/code-review-results-2026-05-03.md`
- Modify: `docs/session-context-2026-05-03.md`

- [ ] **Step 1: Update review results**

In `docs/code-review-results-2026-05-03.md`:

- Add a resolved-work bullet:

```md
- Analysis source group loading and template/group deletion command access and
  route-level orchestration are centralized in
  `src/lib/api/analysis-source-groups.ts` and
  `src/lib/analysis-source-groups-workflow.ts`.
```

- Remove `list_analysis_source_groups`, `delete_analysis_prompt_template`, and
  `delete_analysis_source_group` from remaining raw route command surfaces.
- Update the recommended follow-up order so the next item is report
  start/cancel/delete actions, followed by typed error conversion.
- Add recent verification lines for the new focused tests, route raw-command
  search, full tests, Svelte check, and whitespace check.

- [ ] **Step 2: Update session handoff**

In `docs/session-context-2026-05-03.md`:

- Record the current branch `analysis-source-groups-cleanup` until the user chooses the finishing workflow.
- Add this workstream to the active/completed section once implementation and verification pass.
- Add implemented files:

```text
src/lib/api/analysis-source-groups.ts
src/lib/api/analysis-source-groups.test.ts
src/lib/analysis-source-groups-workflow.ts
src/lib/analysis-source-groups-workflow.test.ts
src/routes/analysis/+page.svelte
```

- Update remaining `/analysis` raw command surfaces to:

```text
start_analysis_report
cancel_analysis_run
delete_analysis_run
```

- Update the recommended next workstream to:

```text
Analysis report start/cancel/delete wrapper/controller
```

- [ ] **Step 3: Run route raw-command search**

Run:

```powershell
rg "list_analysis_source_groups|delete_analysis_prompt_template|delete_analysis_source_group" src/routes/analysis/+page.svelte
```

Expected: no output.

- [ ] **Step 4: Run focused tests**

Run:

```powershell
npm.cmd test -- src/lib/api/analysis-source-groups.test.ts src/lib/analysis-source-groups-workflow.test.ts
```

Expected: PASS with 2 test files and 11 tests.

- [ ] **Step 5: Run full test suite**

Run:

```powershell
npm.cmd test
```

Expected: PASS. The prior baseline was 19 test files and 145 tests; after this workstream expect 21 test files and 156 tests unless unrelated test counts changed.

- [ ] **Step 6: Run Svelte check**

Run:

```powershell
npm.cmd run check
```

Expected: PASS with `svelte-check found 0 errors and 0 warnings`.

- [ ] **Step 7: Run whitespace check**

Run:

```powershell
git diff --check
```

Expected: exit code 0. Git may print LF/CRLF warnings; those are not whitespace failures when the exit code is 0.

- [ ] **Step 8: Commit Task 4**

Run:

```powershell
git add docs/code-review-results-2026-05-03.md docs/session-context-2026-05-03.md
git commit -m "docs(analysis): refresh source groups cleanup context"
```

---

### Final Completion Check

- [ ] **Step 1: Verify branch status**

Run:

```powershell
git status --short --branch
```

Expected: branch `analysis-source-groups-cleanup` with a clean working tree.

- [ ] **Step 2: Offer finishing workflow choices**

Report the completed commits and ask the user whether to merge locally into `main`, keep the branch for more review, or continue with the next workstream.
