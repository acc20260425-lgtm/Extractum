# Project Add Source Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a project-scoped `Add source` action next to each project `Connect from Library` action, reuse the existing Library add-source dialog, and automatically connect resulting Library sources to the active project.

**Architecture:** Keep provider components reusable by passing an optional `ProjectAddSourceContext` prop into the existing dialog tree. Put project connect, refresh, dedupe, status, and idempotent outcome handling in a shared UI workflow helper used by both `ProjectsShell` and `/projects/next`. Preserve the standalone Library scalar `onSourcesChanged(sourceId?: number)` contract and use a project-only batch callback for playlist imports.

**Tech Stack:** Svelte 5 runes, TypeScript, Vitest, Testing Library Svelte, Tauri command wrappers through `$lib/api/projects`, existing Extractum UI wrappers, `@lucide/svelte` icons.

## Global Constraints

- Reuse the existing full `LibraryAddSourceDialog`; do not build a second add-source dialog.
- Keep standalone Library behavior unchanged: `onSourcesChanged` remains `(sourceId?: number) => void | Promise<void>`.
- Add a project-only `onConnectAddedSources(sourceIds: number[])` callback for playlist batch connections.
- Project provider components must not import project APIs or route state.
- `LibraryYoutubePlaylistImport` project mode connects only `summary.results` entries with `status === "added"` and non-null `sourceId`.
- Playlist rows skipped before add execution remain unconnected in v1 because skipped results have `sourceId: null` and no typed duplicate reason.
- YouTube Smart import playlist connects the playlist container source, not individual playlist videos.
- `connectedSourceIds`, `LibraryCatalogSourceView.sourceId`, `ProjectSourceRecord.source_id`, and `addProjectSources.sourceIds` use the same numeric `sources.id` / `source_id` identity space.
- Project workflow status copy must use:
  - `Source added and connected to project.`
  - `Already in Library. Connected to project.`
  - `Already connected to this project.`
  - `Source added to Library, but auto-connect could not be completed.`
  - `Source added to Library, but connecting it to the project failed: <error>`
- Use `npm.cmd`, not `npm`, for frontend validation commands on Windows.
- Use the official Svelte MCP before editing Svelte components and again after component fixes.
- Do not stage `.claude/settings.local.json`.

---

### Task 1: Shared Project Add-Source Helper

**Files:**
- Create: `src/lib/ui/project-add-source-context.ts`
- Create: `src/lib/ui/project-add-source-workflow.ts`
- Create: `src/lib/ui/project-add-source-workflow.test.ts`
- Modify: `src/lib/ui/research-projects-model.test.ts`

**Interfaces:**
- Produces `ProjectAddSourceContext`, consumed by `LibraryAddSourceDialog.svelte`, `LibraryYoutubeAddPanel.svelte`, `LibraryYoutubeSmartImport.svelte`, and `LibraryYoutubePlaylistImport.svelte`.
- Produces `connectProjectSourceIds(input)`, consumed by `research-projects-workflow.ts` and `/projects/next/+page.svelte`.
- Produces `connectedSourceIdsForProject(projectSources, projectId)`, consumed by `ProjectsShell.svelte`, `/projects/next/+page.svelte`, and workflow duplicate pre-checks.

- [ ] **Step 1: Add failing helper and id-space tests**

Create `src/lib/ui/project-add-source-workflow.test.ts`:

```ts
import { describe, expect, it, vi } from "vitest";
import {
  connectProjectSourceIds,
  connectedSourceIdsForProject,
  normalizeProjectSourceIds,
} from "./project-add-source-workflow";
import type { ProjectSourceRecord } from "$lib/types/projects";

function deps() {
  return {
    addProjectSources: vi.fn(),
    refreshAfterProjectSourceConnect: vi.fn(),
    setProjectAddSourceSaving: vi.fn(),
    setProjectAddSourceStatus: vi.fn(),
    formatError: vi.fn((action: string, error: unknown) => `Error ${action}: ${String(error)}`),
  };
}

describe("project add-source workflow", () => {
  it("normalizes source IDs before calling addProjectSources", async () => {
    const d = deps();
    d.addProjectSources.mockResolvedValue({ added_count: 1, already_present_count: 0 });

    await connectProjectSourceIds({
      projectId: 7,
      sourceIds: [10, 10, null, undefined, Number.NaN, 11],
      origin: "new_source",
      emptyBehavior: "missing_source_id_status",
      deps: d,
    });

    expect(d.addProjectSources).toHaveBeenCalledOnce();
    expect(d.addProjectSources).toHaveBeenCalledWith({ projectId: 7, sourceIds: [10, 11] });
    expect(d.setProjectAddSourceStatus).toHaveBeenCalledWith("Source added and connected to project.");
    expect(d.refreshAfterProjectSourceConnect).toHaveBeenCalledOnce();
  });

  it("reports an already-present project connection from backend outcome", async () => {
    const d = deps();
    d.addProjectSources.mockResolvedValue({ added_count: 0, already_present_count: 1 });

    await connectProjectSourceIds({
      projectId: 7,
      sourceIds: [10],
      origin: "existing_source",
      deps: d,
    });

    expect(d.setProjectAddSourceStatus).toHaveBeenCalledWith("Already connected to this project.");
    expect(d.refreshAfterProjectSourceConnect).toHaveBeenCalledOnce();
  });

  it("reports existing Library source connection success", async () => {
    const d = deps();
    d.addProjectSources.mockResolvedValue({ added_count: 1, already_present_count: 0 });

    await connectProjectSourceIds({
      projectId: 7,
      sourceIds: [10],
      origin: "existing_source",
      deps: d,
    });

    expect(d.setProjectAddSourceStatus).toHaveBeenCalledWith("Already in Library. Connected to project.");
  });

  it("refreshes and reports scalar missing source ID without connecting", async () => {
    const d = deps();

    await connectProjectSourceIds({
      projectId: 7,
      sourceIds: [undefined],
      origin: "new_source",
      emptyBehavior: "missing_source_id_status",
      deps: d,
    });

    expect(d.addProjectSources).not.toHaveBeenCalled();
    expect(d.refreshAfterProjectSourceConnect).toHaveBeenCalledOnce();
    expect(d.setProjectAddSourceStatus).toHaveBeenCalledWith(
      "Source added to Library, but auto-connect could not be completed.",
    );
  });

  it("keeps empty playlist batch silent and does not connect", async () => {
    const d = deps();

    await connectProjectSourceIds({
      projectId: 7,
      sourceIds: [],
      origin: "new_source",
      emptyBehavior: "silent",
      deps: d,
    });

    expect(d.addProjectSources).not.toHaveBeenCalled();
    expect(d.refreshAfterProjectSourceConnect).toHaveBeenCalledOnce();
    expect(d.setProjectAddSourceStatus).not.toHaveBeenCalled();
  });

  it("keeps Library add success visible when project connect fails", async () => {
    const d = deps();
    d.addProjectSources.mockRejectedValue(new Error("network"));

    await connectProjectSourceIds({
      projectId: 7,
      sourceIds: [10],
      origin: "new_source",
      deps: d,
    });

    expect(d.setProjectAddSourceStatus).toHaveBeenCalledWith(
      "Source added to Library, but connecting it to the project failed: Error connecting source to project: Error: network",
    );
  });

  it("derives connected source IDs from project source records", () => {
    const rows: Pick<ProjectSourceRecord, "project_id" | "source_id">[] = [
      { project_id: 7, source_id: 10 },
      { project_id: 7, source_id: 11 },
      { project_id: 8, source_id: 12 },
    ];

    expect([...connectedSourceIdsForProject(rows, 7)]).toEqual([10, 11]);
    expect([...connectedSourceIdsForProject(rows, null)]).toEqual([]);
  });

  it("filters non-finite and duplicate IDs", () => {
    expect(normalizeProjectSourceIds([1, 1, null, undefined, Number.NaN, 2])).toEqual([1, 2]);
  });
});
```

Add this test to `src/lib/ui/research-projects-model.test.ts` inside the existing `describe` block:

```ts
  it("keeps catalog, project records, project rows, and connect API IDs in one numeric id-space", () => {
    const libraryRows = buildLibrarySourcesView(library, projectSources, "project:1");
    const projectRows = buildProjectSourceLinksView("project:1", projectSources);
    const addProjectSourcesPayload = { projectId: 1, sourceIds: [libraryRows[0].sourceId] };

    expect(libraryRows[0].sourceId).toBe(projectSources[0].source_id);
    expect(projectRows[0].sourceNumericId).toBe(projectSources[0].source_id);
    expect(addProjectSourcesPayload.sourceIds).toEqual([projectSources[0].source_id]);
  });
```

- [ ] **Step 2: Run tests and verify the new helper fails before implementation**

Run: `npm.cmd run test -- src/lib/ui/project-add-source-workflow.test.ts src/lib/ui/research-projects-model.test.ts`

Expected: FAIL because `./project-add-source-workflow` does not exist.

- [ ] **Step 3: Create project context type**

Create `src/lib/ui/project-add-source-context.ts`:

```ts
export interface ProjectAddSourceContext {
  projectId: number;
  connectedSourceIds: Set<number>;
  onConnectExistingSource(sourceId: number): void | Promise<void>;
  onConnectAddedSources(sourceIds: number[]): void | Promise<void>;
}
```

- [ ] **Step 4: Create workflow helper**

Create `src/lib/ui/project-add-source-workflow.ts`:

```ts
import type { AddProjectSourcesOutcome, ProjectSourceRecord, ProjectSourcesInput } from "$lib/types/projects";

export type ProjectAddSourceConnectOrigin = "new_source" | "existing_source";
export type EmptyProjectAddSourceBehavior = "missing_source_id_status" | "silent";

export interface ProjectAddSourceWorkflowDeps {
  addProjectSources(input: ProjectSourcesInput): Promise<AddProjectSourcesOutcome>;
  refreshAfterProjectSourceConnect(): Promise<void>;
  setProjectAddSourceSaving(saving: boolean): void;
  setProjectAddSourceStatus(message: string): void;
  formatError(action: string, error: unknown): string;
}

export interface ConnectProjectSourceIdsInput {
  projectId: number | null;
  sourceIds: Array<number | null | undefined>;
  origin: ProjectAddSourceConnectOrigin;
  deps: ProjectAddSourceWorkflowDeps;
  emptyBehavior?: EmptyProjectAddSourceBehavior;
}

export function normalizeProjectSourceIds(sourceIds: Array<number | null | undefined>) {
  return [...new Set(sourceIds.filter((id): id is number => typeof id === "number" && Number.isFinite(id)))];
}

export function connectedSourceIdsForProject(
  projectSources: Pick<ProjectSourceRecord, "project_id" | "source_id">[],
  projectId: number | null,
) {
  if (projectId === null) return new Set<number>();
  return new Set(projectSources.filter((source) => source.project_id === projectId).map((source) => source.source_id));
}

function outcomeStatus(outcome: AddProjectSourcesOutcome, origin: ProjectAddSourceConnectOrigin) {
  if (outcome.added_count > 0) {
    return origin === "existing_source"
      ? "Already in Library. Connected to project."
      : "Source added and connected to project.";
  }
  if (outcome.already_present_count > 0) {
    return "Already connected to this project.";
  }
  return "Already connected to this project.";
}

export async function connectProjectSourceIds({
  projectId,
  sourceIds,
  origin,
  deps,
  emptyBehavior = "silent",
}: ConnectProjectSourceIdsInput) {
  if (projectId === null) {
    deps.setProjectAddSourceStatus("Select a project");
    return;
  }

  const normalizedSourceIds = normalizeProjectSourceIds(sourceIds);
  if (normalizedSourceIds.length === 0) {
    await deps.refreshAfterProjectSourceConnect();
    if (emptyBehavior === "missing_source_id_status") {
      deps.setProjectAddSourceStatus("Source added to Library, but auto-connect could not be completed.");
    }
    return;
  }

  deps.setProjectAddSourceSaving(true);
  try {
    const outcome = await deps.addProjectSources({ projectId, sourceIds: normalizedSourceIds });
    deps.setProjectAddSourceStatus(outcomeStatus(outcome, origin));
    await deps.refreshAfterProjectSourceConnect();
  } catch (error) {
    if (origin === "new_source") {
      deps.setProjectAddSourceStatus(
        `Source added to Library, but connecting it to the project failed: ${deps.formatError(
          "connecting source to project",
          error,
        )}`,
      );
    } else {
      deps.setProjectAddSourceStatus(deps.formatError("connecting source to project", error));
    }
  } finally {
    deps.setProjectAddSourceSaving(false);
  }
}
```

- [ ] **Step 5: Run focused tests**

Run: `npm.cmd run test -- src/lib/ui/project-add-source-workflow.test.ts src/lib/ui/research-projects-model.test.ts`

Expected: PASS for both files.

- [ ] **Step 6: Commit Task 1**

Run:

```powershell
git add src/lib/ui/project-add-source-context.ts src/lib/ui/project-add-source-workflow.ts src/lib/ui/project-add-source-workflow.test.ts src/lib/ui/research-projects-model.test.ts
git commit -m "feat: add project add-source workflow helper"
```

Expected: commit succeeds.

---

### Task 2: Research Projects Workflow Wrappers

**Files:**
- Modify: `src/lib/ui/research-projects-workflow.ts`
- Modify: `src/lib/ui/research-projects-workflow.test.ts`

**Interfaces:**
- Consumes `connectProjectSourceIds` and `connectedSourceIdsForProject` from Task 1.
- Produces `workflow.connectAddedProjectSource(sourceId?: number)`.
- Produces `workflow.connectAddedProjectSources(sourceIds: number[])`.
- Produces `workflow.connectExistingProjectSource(sourceId: number)`.
- Produces `workflow.setStatus(message: string)`.

- [ ] **Step 1: Add failing workflow tests**

Append these tests to `src/lib/ui/research-projects-workflow.test.ts` inside the existing `describe` block:

```ts
  it("connects a newly added scalar Library source to the selected project", async () => {
    const state = createInitialState();
    state.selectedProjectId = "project:1";
    const deps = createDeps(state);
    deps.addProjectSources.mockResolvedValue({ added_count: 1, already_present_count: 0 });
    deps.listProjects.mockResolvedValue([project()]);
    deps.listProjectSources.mockResolvedValue([projectSource({ source_id: 10 })]);
    deps.listLibraryCatalog.mockResolvedValue({ sources: [libraryCatalogRecord()], filter_counts: [] });
    deps.listProjectRuns.mockResolvedValue([]);
    deps.listPromptTemplates.mockResolvedValue([]);
    deps.listSourceJobs.mockResolvedValue([]);

    const workflow = createResearchProjectsWorkflow(deps);
    await workflow.connectAddedProjectSource(10);

    expect(deps.addProjectSources).toHaveBeenCalledWith({ projectId: 1, sourceIds: [10] });
    expect(state.status).toBe("Source added and connected to project.");
    expect(deps.listLibraryCatalog).toHaveBeenCalled();
  });

  it("connects newly added playlist videos in one batch", async () => {
    const state = createInitialState();
    state.selectedProjectId = "project:1";
    const deps = createDeps(state);
    deps.addProjectSources.mockResolvedValue({ added_count: 2, already_present_count: 0 });
    deps.listProjects.mockResolvedValue([project()]);
    deps.listProjectSources.mockResolvedValue([
      projectSource({ source_id: 10 }),
      projectSource({ source_id: 11 }),
    ]);
    deps.listLibraryCatalog.mockResolvedValue({ sources: [libraryCatalogRecord()], filter_counts: [] });
    deps.listProjectRuns.mockResolvedValue([]);
    deps.listPromptTemplates.mockResolvedValue([]);
    deps.listSourceJobs.mockResolvedValue([]);

    const workflow = createResearchProjectsWorkflow(deps);
    await workflow.connectAddedProjectSources([10, 11, 11]);

    expect(deps.addProjectSources).toHaveBeenCalledOnce();
    expect(deps.addProjectSources).toHaveBeenCalledWith({ projectId: 1, sourceIds: [10, 11] });
  });

  it("reports a missing scalar source ID without calling addProjectSources", async () => {
    const state = createInitialState();
    state.selectedProjectId = "project:1";
    const deps = createDeps(state);
    deps.listProjects.mockResolvedValue([project()]);
    deps.listProjectSources.mockResolvedValue([]);
    deps.listLibraryCatalog.mockResolvedValue({ sources: [], filter_counts: [] });
    deps.listProjectRuns.mockResolvedValue([]);
    deps.listPromptTemplates.mockResolvedValue([]);
    deps.listSourceJobs.mockResolvedValue([]);

    const workflow = createResearchProjectsWorkflow(deps);
    await workflow.connectAddedProjectSource(undefined);

    expect(deps.addProjectSources).not.toHaveBeenCalled();
    expect(state.status).toBe("Source added to Library, but auto-connect could not be completed.");
  });

  it("skips a duplicate existing-source connect when current state already contains the source", async () => {
    const state = createInitialState();
    state.selectedProjectId = "project:1";
    state.projectSources = [projectSource({ source_id: 10 })];
    const deps = createDeps(state);

    const workflow = createResearchProjectsWorkflow(deps);
    await workflow.connectExistingProjectSource(10);

    expect(deps.addProjectSources).not.toHaveBeenCalled();
    expect(state.status).toBe("Already connected to this project.");
  });

  it("uses backend already-present outcome when an existing source connect races", async () => {
    const state = createInitialState();
    state.selectedProjectId = "project:1";
    const deps = createDeps(state);
    deps.addProjectSources.mockResolvedValue({ added_count: 0, already_present_count: 1 });
    deps.listProjects.mockResolvedValue([project()]);
    deps.listProjectSources.mockResolvedValue([projectSource({ source_id: 10 })]);
    deps.listLibraryCatalog.mockResolvedValue({ sources: [libraryCatalogRecord()], filter_counts: [] });
    deps.listProjectRuns.mockResolvedValue([]);
    deps.listPromptTemplates.mockResolvedValue([]);
    deps.listSourceJobs.mockResolvedValue([]);

    const workflow = createResearchProjectsWorkflow(deps);
    await workflow.connectExistingProjectSource(10);

    expect(deps.addProjectSources).toHaveBeenCalledWith({ projectId: 1, sourceIds: [10] });
    expect(state.status).toBe("Already connected to this project.");
  });

  it("can set project workflow status for dialog provider messages", () => {
    const state = createInitialState();
    const deps = createDeps(state);

    const workflow = createResearchProjectsWorkflow(deps);
    workflow.setStatus("Provider message");

    expect(state.status).toBe("Provider message");
  });
```

- [ ] **Step 2: Run tests and verify the workflow API fails before implementation**

Run: `npm.cmd run test -- src/lib/ui/research-projects-workflow.test.ts`

Expected: FAIL because `connectAddedProjectSource`, `connectAddedProjectSources`, `connectExistingProjectSource`, and `setStatus` are not returned by `createResearchProjectsWorkflow`.

- [ ] **Step 3: Add helper imports**

In `src/lib/ui/research-projects-workflow.ts`, add:

```ts
import {
  connectProjectSourceIds,
  connectedSourceIdsForProject,
  type ProjectAddSourceWorkflowDeps,
} from "./project-add-source-workflow";
```

- [ ] **Step 4: Add wrapper functions before the return object**

Insert this block before `return {` in `createResearchProjectsWorkflow`:

```ts
  function setStatus(message: string) {
    deps.patch({ status: message });
  }

  function selectedNumericProjectId() {
    return projectIdFromViewId(deps.getState().selectedProjectId);
  }

  function projectAddSourceDeps(): ProjectAddSourceWorkflowDeps {
    return {
      addProjectSources: deps.addProjectSources,
      refreshAfterProjectSourceConnect: () => loadWorkspace(),
      setProjectAddSourceSaving: (saving) => deps.patch({ saving }),
      setProjectAddSourceStatus: setStatus,
      formatError: deps.formatError,
    };
  }

  async function connectAddedProjectSource(sourceId?: number) {
    await connectProjectSourceIds({
      projectId: selectedNumericProjectId(),
      sourceIds: [sourceId],
      origin: "new_source",
      emptyBehavior: "missing_source_id_status",
      deps: projectAddSourceDeps(),
    });
  }

  async function connectAddedProjectSources(sourceIds: number[]) {
    await connectProjectSourceIds({
      projectId: selectedNumericProjectId(),
      sourceIds,
      origin: "new_source",
      emptyBehavior: "silent",
      deps: projectAddSourceDeps(),
    });
  }

  async function connectExistingProjectSource(sourceId: number) {
    const projectId = selectedNumericProjectId();
    const connectedSourceIds = connectedSourceIdsForProject(deps.getState().projectSources, projectId);
    if (connectedSourceIds.has(sourceId)) {
      deps.patch({ status: "Already connected to this project." });
      return;
    }

    await connectProjectSourceIds({
      projectId,
      sourceIds: [sourceId],
      origin: "existing_source",
      deps: projectAddSourceDeps(),
    });
  }
```

- [ ] **Step 5: Return the new workflow methods**

Add these properties to the returned object:

```ts
    connectAddedProjectSource,
    connectAddedProjectSources,
    connectExistingProjectSource,
    setStatus,
```

- [ ] **Step 6: Run focused workflow tests**

Run: `npm.cmd run test -- src/lib/ui/research-projects-workflow.test.ts src/lib/ui/project-add-source-workflow.test.ts`

Expected: PASS for both files.

- [ ] **Step 7: Commit Task 2**

Run:

```powershell
git add src/lib/ui/research-projects-workflow.ts src/lib/ui/research-projects-workflow.test.ts
git commit -m "feat: expose project add-source workflow"
```

Expected: commit succeeds.

---

### Task 3: Project-Aware Library Dialog Providers

**Files:**
- Modify: `src/lib/components/research-projects/LibraryAddSourceDialog.svelte`
- Modify: `src/lib/components/research-projects/LibraryYoutubeAddPanel.svelte`
- Modify: `src/lib/components/research-projects/LibraryYoutubeSmartImport.svelte`
- Modify: `src/lib/components/research-projects/LibraryYoutubePlaylistImport.svelte`
- Modify: `src/lib/library-add-source-contract.test.ts`

**Interfaces:**
- Consumes `ProjectAddSourceContext` from Task 1.
- Keeps `onSourcesChanged: (sourceId?: number) => void | Promise<void>` scalar in every component prop type.
- Uses `projectContext.onConnectExistingSource(sourceId)` only for Smart import duplicates.
- Uses `projectContext.onConnectAddedSources(sourceIds)` only for successful playlist batch adds.

- [ ] **Step 1: Add failing component contract tests**

Append these tests to `src/lib/library-add-source-contract.test.ts`:

```ts
  it("keeps the standalone scalar onSourcesChanged contract while accepting project context", () => {
    expect(dialogSource).toContain("projectContext?: ProjectAddSourceContext");
    expect(dialogSource).toContain("onSourcesChanged: (sourceId?: number) => void | Promise<void>");
    expect(youtubePanelSource).toContain("projectContext?: ProjectAddSourceContext");
    expect(youtubePanelSource).toContain("onSourcesChanged: (sourceId?: number) => void | Promise<void>");
    expect(playlistImportSource).toContain("onSourcesChanged: (sourceId?: number) => void | Promise<void>");
    expect(playlistImportSource).not.toContain("onSourcesChanged: (sourceIds: number[])");
  });

  it("passes project context through the YouTube add-source tree", () => {
    expect(dialogSource).toContain("<LibraryYoutubeAddPanel {sources} {onSourcesChanged} {onStatus} {projectContext}");
    expect(youtubePanelSource).toContain("<LibraryYoutubeSmartImport {sources} {onSourcesChanged} {onStatus} {projectContext}");
    expect(youtubePanelSource).toContain("<LibraryYoutubePlaylistImport {sources} {onSourcesChanged} {onStatus} {projectContext}");
  });

  it("allows Smart import duplicates to connect existing Library sources in project mode", () => {
    expect(smartImportSource).toContain("canConnectExistingSmartImportSource");
    expect(smartImportSource).toContain("projectContext.onConnectExistingSource(existingSmartImportSource.sourceId)");
    expect(smartImportSource).toContain("Connect to project");
    expect(smartImportSource).toContain("Already connected to this project");
  });

  it("connects all added playlist video source IDs through the project batch callback", () => {
    expect(playlistImportSource).toContain('result.status === "added"');
    expect(playlistImportSource).toContain("projectContext.onConnectAddedSources(addedSourceIds)");
    expect(playlistImportSource).toContain("await onSourcesChanged(summary.results.find((result) => result.sourceId !== null)?.sourceId ?? undefined)");
  });
```

- [ ] **Step 2: Run tests and verify component contracts fail before implementation**

Run: `npm.cmd run test -- src/lib/library-add-source-contract.test.ts`

Expected: FAIL because `projectContext` is not defined in the dialog or provider components.

- [ ] **Step 3: Extend `LibraryAddSourceDialog.svelte` props and pass context**

Add this import:

```ts
  import type { ProjectAddSourceContext } from "$lib/ui/project-add-source-context";
```

Add `projectContext` to destructuring and type:

```ts
    projectContext,
```

```ts
    projectContext?: ProjectAddSourceContext;
```

Change the YouTube panel call to:

```svelte
        <LibraryYoutubeAddPanel {sources} {onSourcesChanged} {onStatus} {projectContext} />
```

Leave Telegram as:

```svelte
        <LibraryTelegramDialogImport {onSourcesChanged} {onStatus} />
```

- [ ] **Step 4: Extend `LibraryYoutubeAddPanel.svelte` props and pass context**

Add:

```ts
  import type { ProjectAddSourceContext } from "$lib/ui/project-add-source-context";
```

Add `projectContext` to props and type:

```ts
    projectContext,
```

```ts
    projectContext?: ProjectAddSourceContext;
```

Change provider calls to:

```svelte
      <LibraryYoutubeSmartImport {sources} {onSourcesChanged} {onStatus} {projectContext} />
```

```svelte
      <LibraryYoutubePlaylistImport {sources} {onSourcesChanged} {onStatus} {projectContext} />
```

- [ ] **Step 5: Implement Smart import project duplicate states**

In `LibraryYoutubeSmartImport.svelte`, add:

```ts
  import type { ProjectAddSourceContext } from "$lib/ui/project-add-source-context";
```

Add `projectContext` to props and type:

```ts
    projectContext,
```

```ts
    projectContext?: ProjectAddSourceContext;
```

Replace `canAdd` with these derived values:

```ts
  const existingSmartImportSourceConnected = $derived(
    Boolean(
      existingSmartImportSource &&
        projectContext?.connectedSourceIds.has(existingSmartImportSource.sourceId),
    ),
  );
  const canConnectExistingSmartImportSource = $derived(
    Boolean(
      projectContext &&
        existingSmartImportSource &&
        !existingSmartImportSourceConnected &&
        !previewing &&
        !adding,
    ),
  );
  const canAdd = $derived(Boolean(preview) && !existingSmartImportSource && !previewing && !adding);
```

Replace the duplicate branch in `addSource()` with:

```ts
    if (existingSmartImportSource) {
      if (projectContext) {
        if (projectContext.connectedSourceIds.has(existingSmartImportSource.sourceId)) {
          onStatus("Already connected to this project.");
          return;
        }
        await projectContext.onConnectExistingSource(existingSmartImportSource.sourceId);
        return;
      }
      status = `Already in Library: ${existingSmartImportSource.title}`;
      return;
    }
```

Change the duplicate status block to:

```svelte
  {#if existingSmartImportSource}
    <ExtractumStatusMessage tone="info">
      {#if existingSmartImportSourceConnected}
        Already connected to this project.
      {:else}
        Already in Library: {existingSmartImportSource.title}
      {/if}
    </ExtractumStatusMessage>
  {/if}
```

Change the action button disabled expression and label to:

```svelte
          <ExtractumButton onclick={addSource} disabled={!canAdd && !canConnectExistingSmartImportSource}>
            <Plus size={14} aria-hidden="true" />
            {#if existingSmartImportSourceConnected}
              Already connected to this project
            {:else if existingSmartImportSource && projectContext}
              Connect to project
            {:else if existingSmartImportSource}
              Already in Library
            {:else}
              {adding ? "Adding..." : "Add source"}
            {/if}
          </ExtractumButton>
```

- [ ] **Step 6: Implement playlist batch project callback**

In `LibraryYoutubePlaylistImport.svelte`, add:

```ts
  import type { ProjectAddSourceContext } from "$lib/ui/project-add-source-context";
```

Add `projectContext` to props and type:

```ts
    projectContext,
```

```ts
    projectContext?: ProjectAddSourceContext;
```

Replace the `if (summary.added > 0)` block with:

```ts
      if (summary.added > 0) {
        onStatus(`Added ${summary.added} YouTube video source${summary.added === 1 ? "" : "s"}.`);
        if (projectContext) {
          const addedSourceIds = summary.results
            .filter((result) => result.status === "added" && result.sourceId !== null)
            .map((result) => result.sourceId as number);
          if (addedSourceIds.length > 0) {
            await projectContext.onConnectAddedSources(addedSourceIds);
          }
        } else {
          await onSourcesChanged(summary.results.find((result) => result.sourceId !== null)?.sourceId ?? undefined);
        }
      }
```

- [ ] **Step 7: Run component contract tests**

Run: `npm.cmd run test -- src/lib/library-add-source-contract.test.ts`

Expected: PASS.

- [ ] **Step 8: Run Svelte check for component typing**

Run: `npm.cmd run check`

Expected: PASS. Existing Tauri IPC browser-console warnings are irrelevant because this command does not run a browser.

- [ ] **Step 9: Commit Task 3**

Run:

```powershell
git add src/lib/components/research-projects/LibraryAddSourceDialog.svelte src/lib/components/research-projects/LibraryYoutubeAddPanel.svelte src/lib/components/research-projects/LibraryYoutubeSmartImport.svelte src/lib/components/research-projects/LibraryYoutubePlaylistImport.svelte src/lib/library-add-source-contract.test.ts
git commit -m "feat: make library add-source dialog project-aware"
```

Expected: commit succeeds.

---

### Task 4: Current Projects Shell Wiring

**Files:**
- Modify: `src/lib/components/research-projects/SourcesTab.svelte`
- Modify: `src/lib/components/research-projects/ProjectWorkspace.svelte`
- Modify: `src/lib/components/research-projects/ProjectsShell.svelte`
- Modify: `src/routes/projects/+page.svelte`
- Modify: `src/routes/projects/list/+page.svelte`
- Modify: `src/lib/research-projects-route-contract.test.ts`

**Interfaces:**
- Consumes workflow methods from Task 2.
- Consumes `LibraryAddSourceDialog` with optional `projectContext` from Task 3.
- Consumes `buildLibraryCatalogSourcesView` from `src/lib/ui/library-catalog-model.ts`.
- Consumes `connectedSourceIdsForProject` from Task 1.

- [ ] **Step 1: Add failing route and toolbar contract tests**

Append these tests to `src/lib/research-projects-route-contract.test.ts`:

```ts
  it("places Add source immediately before Connect from Library in the project sources toolbar", () => {
    expect(sourcesTabSource).toContain("onOpenAddSource");
    expect(sourcesTabSource).toContain('data-ui-action="add-source"');
    expect(sourcesTabSource).toContain('data-ui-action="connect-library"');
    expect(sourcesTabSource.indexOf('data-ui-action="add-source"')).toBeLessThan(
      sourcesTabSource.indexOf('data-ui-action="connect-library"'),
    );
    expect(sourcesTabSource).toContain("Plus");
    expect(sourcesTabSource).toContain("Add source");
  });

  it("wires the project Add source dialog through the current ProjectsShell", () => {
    expect(workspaceSource).toContain("onOpenAddSource");
    expect(shellSource).toContain("LibraryAddSourceDialog");
    expect(shellSource).toContain("addSourceOpen");
    expect(shellSource).toContain("projectAddSourceContext");
    expect(shellSource).toContain("buildLibraryCatalogSourcesView");
    expect(shellSource).toContain("connectedSourceIdsForProject");
    expect(shellSource).toContain("onConnectAddedProjectSource");
    expect(shellSource).toContain("onConnectAddedProjectSources");
    expect(shellSource).toContain("onConnectExistingProjectSource");
  });

  it("passes project add-source workflow callbacks from both current project routes", () => {
    expect(pageSource).toContain("onConnectAddedProjectSource={workflow.connectAddedProjectSource}");
    expect(pageSource).toContain("onConnectAddedProjectSources={workflow.connectAddedProjectSources}");
    expect(pageSource).toContain("onConnectExistingProjectSource={workflow.connectExistingProjectSource}");
    expect(pageSource).toContain("onSetStatus={workflow.setStatus}");
    expect(pageSource).not.toContain("onSourcesChanged={(ids)");

    const listPageSource = readFileSync(resolve(process.cwd(), "src/routes/projects/list/+page.svelte"), "utf8");
    expect(listPageSource).toContain("onConnectAddedProjectSource={workflow.connectAddedProjectSource}");
    expect(listPageSource).toContain("onConnectAddedProjectSources={workflow.connectAddedProjectSources}");
    expect(listPageSource).toContain("onConnectExistingProjectSource={workflow.connectExistingProjectSource}");
    expect(listPageSource).toContain("onSetStatus={workflow.setStatus}");
  });
```

- [ ] **Step 2: Run tests and verify current shell contracts fail before implementation**

Run: `npm.cmd run test -- src/lib/research-projects-route-contract.test.ts`

Expected: FAIL because `Add source`, `LibraryAddSourceDialog`, and project add-source callbacks are not wired.

- [ ] **Step 3: Add `Add source` button to `SourcesTab.svelte`**

Add `Plus` to the lucide import:

```ts
  import { Library, RefreshCw, Download, Trash2, X, Plus } from "@lucide/svelte";
```

Add `onOpenAddSource` to props:

```ts
    onOpenAddSource,
```

```ts
    onOpenAddSource: () => void;
```

Insert this button immediately before the existing `Connect from Library` button:

```svelte
        <ExtractumButton
          data-ui-action="add-source"
          onclick={onOpenAddSource}
          disabled={!project}
          aria-label="Add source to project"
          title="Add source to project"
        >
          <Plus size={14} aria-hidden="true" />
          Add source
        </ExtractumButton>
```

- [ ] **Step 4: Pass `onOpenAddSource` through `ProjectWorkspace.svelte`**

Add the prop to destructuring and type:

```ts
    onOpenAddSource,
```

```ts
    onOpenAddSource: () => void;
```

Pass it into `SourcesTab`:

```svelte
        {onOpenAddSource}
```

- [ ] **Step 5: Wire dialog state and project context in `ProjectsShell.svelte`**

Add imports:

```ts
  import LibraryAddSourceDialog from "./LibraryAddSourceDialog.svelte";
  import { buildLibraryCatalogSourcesView } from "$lib/ui/library-catalog-model";
  import type { ProjectAddSourceContext } from "$lib/ui/project-add-source-context";
  import { connectedSourceIdsForProject } from "$lib/ui/project-add-source-workflow";
```

Add props:

```ts
    onConnectAddedProjectSource,
    onConnectAddedProjectSources,
    onConnectExistingProjectSource,
    onSetStatus,
```

```ts
    onConnectAddedProjectSource: (sourceId?: number) => void | Promise<void>;
    onConnectAddedProjectSources: (sourceIds: number[]) => void | Promise<void>;
    onConnectExistingProjectSource: (sourceId: number) => void | Promise<void>;
    onSetStatus: (message: string) => void;
```

Add derived values and state near `connectOpen`:

```ts
  let addSourceOpen = $state(false);
  let libraryCatalogSources = $derived(buildLibraryCatalogSourcesView(workflowState.libraryCatalogRecords));
  let connectedSourceIds = $derived(
    connectedSourceIdsForProject(workflowState.projectSources, currentProject?.projectId ?? null),
  );
  let projectAddSourceContext = $derived<ProjectAddSourceContext | undefined>(
    currentProject
      ? {
          projectId: currentProject.projectId,
          connectedSourceIds,
          onConnectExistingSource: onConnectExistingProjectSource,
          onConnectAddedSources: onConnectAddedProjectSources,
        }
      : undefined,
  );
```

Add:

```ts
  function openAddSource() {
    addSourceOpen = true;
  }
```

Pass the new callback into `ProjectWorkspace`:

```svelte
          onOpenAddSource={openAddSource}
```

Render the add-source dialog before `ConnectFromLibrary`:

```svelte
  <LibraryAddSourceDialog
    bind:open={addSourceOpen}
    sources={libraryCatalogSources}
    onSourcesChanged={onConnectAddedProjectSource}
    onStatus={onSetStatus}
    projectContext={projectAddSourceContext}
  />
```

- [ ] **Step 6: Pass workflow callbacks from `/projects` and `/projects/list`**

In both `src/routes/projects/+page.svelte` and `src/routes/projects/list/+page.svelte`, add these props to `<ProjectsShell>`:

```svelte
    onConnectAddedProjectSource={workflow.connectAddedProjectSource}
    onConnectAddedProjectSources={workflow.connectAddedProjectSources}
    onConnectExistingProjectSource={workflow.connectExistingProjectSource}
    onSetStatus={workflow.setStatus}
```

- [ ] **Step 7: Run focused tests**

Run: `npm.cmd run test -- src/lib/research-projects-route-contract.test.ts src/lib/ui/research-projects-workflow.test.ts`

Expected: PASS.

- [ ] **Step 8: Run Svelte check**

Run: `npm.cmd run check`

Expected: PASS.

- [ ] **Step 9: Commit Task 4**

Run:

```powershell
git add src/lib/components/research-projects/SourcesTab.svelte src/lib/components/research-projects/ProjectWorkspace.svelte src/lib/components/research-projects/ProjectsShell.svelte src/routes/projects/+page.svelte src/routes/projects/list/+page.svelte src/lib/research-projects-route-contract.test.ts
git commit -m "feat: wire add source into projects shell"
```

Expected: commit succeeds.

---

### Task 5: `/projects/next` Project Add-Source Wiring

**Files:**
- Modify: `src/lib/components/research-projects/SourcesFilterBar.svelte`
- Modify: `src/lib/components/research-projects/SourcesFilterBar.test.ts`
- Modify: `src/routes/projects/next/+page.svelte`
- Modify: `src/lib/research-projects-route-contract.test.ts`

**Interfaces:**
- Consumes `connectProjectSourceIds`, `connectedSourceIdsForProject`, and `ProjectAddSourceWorkflowDeps` from Task 1.
- Consumes `LibraryAddSourceDialog` with `projectContext` from Task 3.
- Preserves the existing Connect-from-Library sheet by moving it to `onConnectFromLibrary`.

- [ ] **Step 1: Add failing `SourcesFilterBar` tests**

Update the existing add-source test in `src/lib/components/research-projects/SourcesFilterBar.test.ts` to use two callbacks:

```ts
  it("exposes separate Add source and Connect from Library actions", async () => {
    const onAddSource = vi.fn();
    const onConnectFromLibrary = vi.fn();
    render(SourcesFilterBar, { props: { ...base, onAddSource, onConnectFromLibrary } });

    await fireEvent.click(screen.getByRole("button", { name: "Add source" }));
    await fireEvent.click(screen.getByRole("button", { name: "Connect from Library" }));

    expect(onAddSource).toHaveBeenCalledOnce();
    expect(onConnectFromLibrary).toHaveBeenCalledOnce();
  });
```

If the test file does not import `screen` and `fireEvent`, change its import to:

```ts
import { fireEvent, render, screen } from "@testing-library/svelte";
```

- [ ] **Step 2: Add failing `/projects/next` route contract tests**

Add these assertions to the route contract test created in Task 4:

```ts
    const nextPageSource = readFileSync(resolve(process.cwd(), "src/routes/projects/next/+page.svelte"), "utf8");
    expect(nextPageSource).toContain("LibraryAddSourceDialog");
    expect(nextPageSource).toContain("connectAddedProjectSource");
    expect(nextPageSource).toContain("connectAddedProjectSources");
    expect(nextPageSource).toContain("connectExistingProjectSource");
    expect(nextPageSource).toContain("projectAddSourceContext");
    expect(nextPageSource).toContain("onConnectFromLibrary: () => void openConnectSources()");
    expect(nextPageSource).toContain("onAddSource: () => (addSourceOpen = true)");
```

- [ ] **Step 3: Run tests and verify `/projects/next` contracts fail before implementation**

Run: `npm.cmd run test -- src/lib/components/research-projects/SourcesFilterBar.test.ts src/lib/research-projects-route-contract.test.ts`

Expected: FAIL because `SourcesFilterBar` has one action callback and `/projects/next` does not render `LibraryAddSourceDialog`.

- [ ] **Step 4: Split `SourcesFilterBar` actions**

In `SourcesFilterBar.svelte`, add `onConnectFromLibrary` to props:

```ts
    onConnectFromLibrary,
```

```ts
    onConnectFromLibrary?: () => void;
```

Replace the single right-side action button with:

```svelte
  <div class="sources-filter-bar__actions">
    <button
      type="button"
      class="sources-filter-bar__add"
      data-ui-action="add-source"
      aria-label="Add source"
      title="Add source"
      onclick={() => onAddSource?.()}
    >
      <span class="sources-filter-bar__add-plus">+</span>Add source
    </button>
    <button
      type="button"
      class="sources-filter-bar__connect"
      data-ui-action="connect-library"
      aria-label="Connect from Library"
      title="Connect from Library"
      onclick={() => onConnectFromLibrary?.()}
    >
      Connect from Library
    </button>
  </div>
```

Replace `.sources-filter-bar > .sources-filter-bar__add` CSS selectors with:

```css
  .sources-filter-bar__actions {
    display: inline-flex;
    flex-shrink: 0;
    align-items: center;
    gap: 8px;
  }

  .sources-filter-bar__actions > button {
    height: 28px;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 0 11px;
    border-radius: 6px;
    font: 600 12px/1 var(--extractum-font);
    cursor: pointer;
  }

  .sources-filter-bar__add {
    border: 1px solid var(--extractum-primary);
    background: color-mix(in srgb, var(--extractum-primary) 6%, transparent);
    color: var(--extractum-primary);
  }

  .sources-filter-bar__add:hover {
    background: color-mix(in srgb, var(--extractum-primary) 12%, transparent);
  }

  .sources-filter-bar__connect {
    border: 1px solid var(--extractum-border);
    background: var(--extractum-surface);
    color: var(--extractum-text);
  }

  .sources-filter-bar__connect:hover {
    background: var(--extractum-surface-subtle);
  }
```

Keep `.sources-filter-bar__add-plus` as the plus glyph styling.

- [ ] **Step 5: Wire shared helper into `/projects/next/+page.svelte`**

Add imports:

```ts
  import LibraryAddSourceDialog from "$lib/components/research-projects/LibraryAddSourceDialog.svelte";
  import { buildLibraryCatalogSourcesView } from "$lib/ui/library-catalog-model";
  import type { ProjectAddSourceContext } from "$lib/ui/project-add-source-context";
  import {
    connectProjectSourceIds,
    connectedSourceIdsForProject,
    type ProjectAddSourceWorkflowDeps,
  } from "$lib/ui/project-add-source-workflow";
```

Add state and derived values:

```ts
  let addSourceOpen = $state(false);
  let libraryCatalogSources = $derived(buildLibraryCatalogSourcesView(libraryCatalogRecords));
  let connectedSourceIds = $derived(connectedSourceIdsForProject(sources, selectedProjectId));
  let projectAddSourceContext = $derived<ProjectAddSourceContext | undefined>(
    selectedProjectId !== null
      ? {
          projectId: selectedProjectId,
          connectedSourceIds,
          onConnectExistingSource: connectExistingProjectSource,
          onConnectAddedSources: connectAddedProjectSources,
        }
      : undefined,
  );
```

Add helper deps and wrappers before `openConnectSources()`:

```ts
  async function refreshAfterProjectSourceConnect() {
    const catalog = await listLibraryCatalog();
    libraryCatalogRecords = catalog.sources;
    if (selectedProjectId !== null) {
      sources = await listProjectSources(selectedProjectId);
    }
    await workflow.reload();
  }

  function projectAddSourceDeps(): ProjectAddSourceWorkflowDeps {
    return {
      addProjectSources,
      refreshAfterProjectSourceConnect,
      setProjectAddSourceSaving: (saving) => {
        railState = { ...railState, saving };
      },
      setProjectAddSourceStatus: (status) => {
        railState = { ...railState, status };
      },
      formatError: (action, error) => `РќРµ СѓРґР°Р»РѕСЃСЊ РІС‹РїРѕР»РЅРёС‚СЊ: ${action} (${String(error)})`,
    };
  }

  async function connectAddedProjectSource(sourceId?: number) {
    await connectProjectSourceIds({
      projectId: selectedProjectId,
      sourceIds: [sourceId],
      origin: "new_source",
      emptyBehavior: "missing_source_id_status",
      deps: projectAddSourceDeps(),
    });
  }

  async function connectAddedProjectSources(sourceIds: number[]) {
    await connectProjectSourceIds({
      projectId: selectedProjectId,
      sourceIds,
      origin: "new_source",
      emptyBehavior: "silent",
      deps: projectAddSourceDeps(),
    });
  }

  async function connectExistingProjectSource(sourceId: number) {
    if (connectedSourceIds.has(sourceId)) {
      railState = { ...railState, status: "Already connected to this project." };
      return;
    }
    await connectProjectSourceIds({
      projectId: selectedProjectId,
      sourceIds: [sourceId],
      origin: "existing_source",
      deps: projectAddSourceDeps(),
    });
  }
```

- [ ] **Step 6: Wire `/projects/next` buttons and dialog**

Change the `filterBar` action props to:

```ts
          onAddSource: () => (addSourceOpen = true),
          onConnectFromLibrary: () => void openConnectSources(),
```

Render `LibraryAddSourceDialog` before `ConnectFromLibrary`:

```svelte
  <LibraryAddSourceDialog
    bind:open={addSourceOpen}
    sources={libraryCatalogSources}
    onSourcesChanged={connectAddedProjectSource}
    onStatus={(status) => (railState = { ...railState, status })}
    projectContext={projectAddSourceContext}
  />
```

- [ ] **Step 7: Run focused tests**

Run: `npm.cmd run test -- src/lib/components/research-projects/SourcesFilterBar.test.ts src/lib/research-projects-route-contract.test.ts`

Expected: PASS.

- [ ] **Step 8: Run Svelte check**

Run: `npm.cmd run check`

Expected: PASS.

- [ ] **Step 9: Commit Task 5**

Run:

```powershell
git add src/lib/components/research-projects/SourcesFilterBar.svelte src/lib/components/research-projects/SourcesFilterBar.test.ts src/routes/projects/next/+page.svelte src/lib/research-projects-route-contract.test.ts
git commit -m "feat: wire add source into projects next"
```

Expected: commit succeeds.

---

### Task 6: Final Validation And Regression Guard

**Files:**
- Modify only if a preceding verification command exposes a concrete defect in files already changed by Tasks 1-5.

**Interfaces:**
- Verifies the full project add-source surface after all implementation commits.
- Confirms the standalone Library callback contract remains scalar.

- [ ] **Step 1: Run the focused frontend test set**

Run:

```powershell
npm.cmd run test -- src/lib/ui/project-add-source-workflow.test.ts src/lib/ui/research-projects-workflow.test.ts src/lib/ui/research-projects-model.test.ts src/lib/library-add-source-contract.test.ts src/lib/research-projects-route-contract.test.ts src/lib/components/research-projects/SourcesFilterBar.test.ts
```

Expected: PASS.

- [ ] **Step 2: Run the full Svelte and TypeScript check**

Run: `npm.cmd run check`

Expected: PASS.

- [ ] **Step 3: Inspect the final diff**

Run: `git diff --check`

Expected: no whitespace errors.

Run: `git status --short`

Expected: only implementation files from this plan are modified or staged; `.claude/settings.local.json` remains untracked and unstaged.

- [ ] **Step 4: Commit final fixes if Step 1, Step 2, or Step 3 required a code correction**

If a correction was needed, run:

```powershell
git add src/lib/ui/project-add-source-context.ts src/lib/ui/project-add-source-workflow.ts src/lib/ui/project-add-source-workflow.test.ts src/lib/ui/research-projects-model.test.ts src/lib/ui/research-projects-workflow.ts src/lib/ui/research-projects-workflow.test.ts src/lib/components/research-projects/LibraryAddSourceDialog.svelte src/lib/components/research-projects/LibraryYoutubeAddPanel.svelte src/lib/components/research-projects/LibraryYoutubeSmartImport.svelte src/lib/components/research-projects/LibraryYoutubePlaylistImport.svelte src/lib/library-add-source-contract.test.ts src/lib/components/research-projects/SourcesTab.svelte src/lib/components/research-projects/ProjectWorkspace.svelte src/lib/components/research-projects/ProjectsShell.svelte src/routes/projects/+page.svelte src/routes/projects/list/+page.svelte src/lib/research-projects-route-contract.test.ts src/lib/components/research-projects/SourcesFilterBar.svelte src/lib/components/research-projects/SourcesFilterBar.test.ts src/routes/projects/next/+page.svelte
git commit -m "fix: stabilize project add-source flow"
```

Expected: commit succeeds. If no correction was needed, do not create an empty commit.
