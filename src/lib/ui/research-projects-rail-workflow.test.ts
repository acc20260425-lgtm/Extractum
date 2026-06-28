import { describe, expect, it, vi } from "vitest";
import {
  createProjectRailWorkflow,
  type ProjectRailState,
} from "./research-projects-rail-workflow";
import type { ProjectDataRange, ProjectSummary } from "$lib/types/projects";

function summary(overrides: Partial<ProjectSummary> = {}): ProjectSummary {
  return {
    id: 1,
    name: "Alpha",
    description: null,
    source_count: 3,
    material_count: 100,
    status: "ready",
    last_run_at: null,
    pinned: false,
    archived: false,
    updated_at: 1,
    ...overrides,
  };
}

function createInitialState(): ProjectRailState {
  return { summaries: [], dataRange: null, saving: false, status: "" };
}

function createDeps(state: ProjectRailState) {
  return {
    getState: () => state,
    patch: vi.fn((patch: Partial<ProjectRailState>) => Object.assign(state, patch)),
    listResearchProjects: vi.fn(async () => [] as ProjectSummary[]),
    setProjectPinned: vi.fn(async () => {}),
    setProjectArchived: vi.fn(async () => {}),
    getProjectDataRange: vi.fn(async (): Promise<ProjectDataRange> => ({ from: null, to: null })),
    formatError: (action: string) => `ERR:${action}`,
  };
}

describe("createProjectRailWorkflow.setPinned", () => {
  it("pins a project and reloads the summaries", async () => {
    const state = createInitialState();
    const deps = createDeps(state);
    deps.listResearchProjects.mockResolvedValue([summary({ id: 7, pinned: true })]);
    const workflow = createProjectRailWorkflow(deps);

    await workflow.setPinned(7, true);

    expect(deps.setProjectPinned).toHaveBeenCalledWith({ projectId: 7, pinned: true });
    expect(state.summaries.map((row) => row.id)).toEqual([7]);
    expect(state.saving).toBe(false);
  });

  it("reports an error and clears saving when pinning fails", async () => {
    const state = createInitialState();
    const deps = createDeps(state);
    deps.setProjectPinned.mockRejectedValue(new Error("boom"));
    const workflow = createProjectRailWorkflow(deps);

    await workflow.setPinned(7, true);

    expect(state.status).toBe("ERR:pinning project");
    expect(state.saving).toBe(false);
    expect(deps.listResearchProjects).not.toHaveBeenCalled();
  });
});

describe("createProjectRailWorkflow.setArchived", () => {
  it("archives a project and reloads the summaries", async () => {
    const state = createInitialState();
    const deps = createDeps(state);
    deps.listResearchProjects.mockResolvedValue([summary({ id: 7, archived: true })]);
    const workflow = createProjectRailWorkflow(deps);

    await workflow.setArchived(7, true);

    expect(deps.setProjectArchived).toHaveBeenCalledWith({ projectId: 7, archived: true });
    expect(state.summaries.map((row) => row.id)).toEqual([7]);
    expect(state.saving).toBe(false);
  });

  it("reports an error when archiving fails", async () => {
    const state = createInitialState();
    const deps = createDeps(state);
    deps.setProjectArchived.mockRejectedValue(new Error("boom"));
    const workflow = createProjectRailWorkflow(deps);

    await workflow.setArchived(7, true);

    expect(state.status).toBe("ERR:archiving project");
    expect(state.saving).toBe(false);
  });
});

describe("createProjectRailWorkflow.loadDataRange", () => {
  const input = { projectId: 7, youtubeCorpusMode: null, includeMigratedHistory: false };

  it("loads and stores the project data range", async () => {
    const state = createInitialState();
    const deps = createDeps(state);
    deps.getProjectDataRange.mockResolvedValue({ from: 100, to: 200 });
    const workflow = createProjectRailWorkflow(deps);

    await workflow.loadDataRange(input);

    expect(deps.getProjectDataRange).toHaveBeenCalledWith(input);
    expect(state.dataRange).toEqual({ from: 100, to: 200 });
  });

  it("reports an error when loading the data range fails", async () => {
    const state = createInitialState();
    const deps = createDeps(state);
    deps.getProjectDataRange.mockRejectedValue(new Error("boom"));
    const workflow = createProjectRailWorkflow(deps);

    await workflow.loadDataRange(input);

    expect(state.status).toBe("ERR:loading project data range");
    expect(state.dataRange).toBeNull();
  });
});
