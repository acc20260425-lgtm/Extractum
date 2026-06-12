import { describe, expect, it, vi } from "vitest";
import { createResearchProjectsWorkflow, type ResearchProjectsWorkflowState } from "./research-projects-workflow";
import type { AnalysisSourceGroup, AnalysisSourceOption } from "$lib/types/analysis";
import type { SourceJobRecord } from "$lib/types/sources";

function group(overrides: Partial<AnalysisSourceGroup> = {}): AnalysisSourceGroup {
  return {
    id: 10,
    name: "Рынок БПЛА",
    source_type: "telegram",
    members: [{ source_id: 1, source_title: "Radar BPLA", item_count: 12 }],
    created_at: 100,
    updated_at: 200,
    ...overrides,
  };
}

function source(overrides: Partial<AnalysisSourceOption> = {}): AnalysisSourceOption {
  return {
    id: 2,
    account_id: 1,
    source_type: "telegram",
    title: "Drone News",
    item_count: 20,
    last_synced_at: 300,
    ...overrides,
  };
}

function sourceJob(overrides: Partial<SourceJobRecord> = {}): SourceJobRecord {
  return {
    job_id: "job-1",
    source_id: 2,
    related_source_id: null,
    job_type: "youtube_video_full_sync",
    status: "running",
    message: "Syncing",
    progress_current: 1,
    progress_total: 3,
    started_at: 300,
    finished_at: null,
    warnings: [],
    error: null,
    ...overrides,
  };
}

function createHarness(initial: Partial<ResearchProjectsWorkflowState> = {}) {
  const state: ResearchProjectsWorkflowState = {
    groups: [],
    sources: [],
    runs: [],
    sourceJobs: [],
    projects: [],
    librarySources: [],
    projectSourceLinks: [],
    selectedProjectId: null,
    selectedLibrarySourceIds: new Set<string>(),
    loading: false,
    saving: false,
    status: "",
    ...initial,
  };

  const deps = {
    getState: () => state,
    patch: vi.fn((patch: Partial<ResearchProjectsWorkflowState>) => Object.assign(state, patch)),
    listGroups: vi.fn(),
    listSources: vi.fn(),
    listRuns: vi.fn(),
    listSourceJobs: vi.fn(),
    updateGroup: vi.fn(),
    formatError: vi.fn((action: string, error: unknown) => `Error ${action}: ${String(error)}`),
  };

  return { state, deps, workflow: createResearchProjectsWorkflow(deps) };
}

describe("research projects workflow", () => {
  it("loads projects, sources, and selects the first project", async () => {
    const { state, deps, workflow } = createHarness();
    deps.listGroups.mockResolvedValueOnce([group()]);
    deps.listSources.mockResolvedValueOnce([source({ id: 1, title: "Radar BPLA" })]);
    deps.listRuns.mockResolvedValueOnce([]);
    deps.listSourceJobs.mockResolvedValueOnce([]);

    await workflow.loadWorkspace();

    expect(state.selectedProjectId).toBe("source-group:10");
    expect(state.projects[0].title).toBe("Рынок БПЛА");
    expect(state.librarySources[0].alreadyConnected).toBe(true);
    expect(state.projectSourceLinks).toHaveLength(1);
    expect(state.loading).toBe(false);
  });

  it("threads source jobs into derived library rows", async () => {
    const { state, deps, workflow } = createHarness();
    deps.listGroups.mockResolvedValueOnce([group({ source_type: "youtube", members: [] })]);
    deps.listSources.mockResolvedValueOnce([
      source({ id: 2, source_type: "youtube", title: "Alpha Drones" }),
    ]);
    deps.listRuns.mockResolvedValueOnce([]);
    deps.listSourceJobs.mockResolvedValueOnce([sourceJob()]);

    await workflow.loadWorkspace();

    expect(state.sourceJobs).toHaveLength(1);
    expect(state.librarySources[0]).toEqual(expect.objectContaining({
      status: "syncing",
      connectable: false,
      disabledReason: "Источник сейчас синхронизируется.",
    }));
  });

  it("persists only safe selected rows through updateGroup", async () => {
    const currentGroup = group();
    const { state, deps, workflow } = createHarness({
      groups: [currentGroup],
      sources: [source({ id: 1, title: "Radar BPLA" }), source({ id: 2, title: "Drone News" })],
      selectedProjectId: "source-group:10",
      selectedLibrarySourceIds: new Set(["source:2"]),
    });
    deps.updateGroup.mockResolvedValueOnce({
      ...currentGroup,
      members: [...currentGroup.members, { source_id: 2, source_title: "Drone News", item_count: 20 }],
    });
    deps.listGroups.mockResolvedValueOnce([
      {
        ...currentGroup,
        members: [...currentGroup.members, { source_id: 2, source_title: "Drone News", item_count: 20 }],
      },
    ]);
    deps.listSources.mockResolvedValueOnce(state.sources);
    deps.listRuns.mockResolvedValueOnce([]);
    deps.listSourceJobs.mockResolvedValueOnce([]);

    await workflow.refreshDerivedState();
    await workflow.connectSelectedSources();

    expect(deps.updateGroup).toHaveBeenCalledWith({
      groupId: 10,
      name: "Рынок БПЛА",
      sourceType: "telegram",
      sourceIds: [1, 2],
    });
    expect(state.status).toBe("Подключено источников: 1.");
    expect(state.selectedLibrarySourceIds.size).toBe(0);
    expect(state.saving).toBe(false);
  });

  it("refuses unsupported or already-connected selections without calling updateGroup", async () => {
    const { state, deps, workflow } = createHarness({
      groups: [group()],
      sources: [source({ id: 3, source_type: "rss", title: "Новости БПЛА" })],
      selectedProjectId: "source-group:10",
      selectedLibrarySourceIds: new Set(["source:3"]),
    });

    await workflow.refreshDerivedState();
    await workflow.connectSelectedSources();

    expect(deps.updateGroup).not.toHaveBeenCalled();
    expect(state.status).toBe(
      "В выбранных строках нет источников, которые можно подключить к этому проекту.",
    );
  });
});
