import { describe, expect, it, vi } from "vitest";
import { createResearchProjectsWorkflow, type ResearchProjectsWorkflowState } from "./research-projects-workflow";
import type { AnalysisPromptTemplate } from "$lib/types/analysis";
import type { LibrarySourceRecord } from "$lib/types/library-sources";
import type { ProjectRecord, ProjectSourceRecord } from "$lib/types/projects";
import type { SourceJobRecord } from "$lib/types/sources";

function project(overrides: Partial<ProjectRecord> = {}): ProjectRecord {
  return {
    id: 1,
    name: "Alpha",
    description: null,
    created_at: 1,
    updated_at: 1,
    ...overrides,
  };
}

function projectSource(overrides: Partial<ProjectSourceRecord> = {}): ProjectSourceRecord {
  return {
    project_id: 1,
    source_id: 10,
    provider: "youtube",
    source_subtype: "video",
    title: "Video",
    subtitle: "Channel",
    item_count: 3,
    added_at: 1,
    ...overrides,
  };
}

function librarySource(overrides: Partial<LibrarySourceRecord> = {}): LibrarySourceRecord {
  return {
    source_id: 10,
    provider: "youtube",
    source_subtype: "video",
    account_id: null,
    external_id: "v1",
    title: "Video",
    subtitle: "Channel",
    canonical_url: "https://youtu.be/v1",
    created_at: 1,
    last_synced_at: 1,
    item_count: 3,
    project_count: 0,
    youtube: null,
    telegram: null,
    ...overrides,
  };
}

function sourceJob(overrides: Partial<SourceJobRecord> = {}): SourceJobRecord {
  return {
    job_id: "job-1",
    source_id: 10,
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

function promptTemplate(overrides: Partial<AnalysisPromptTemplate> = {}): AnalysisPromptTemplate {
  return {
    id: 1,
    name: "Default",
    template_kind: "report",
    body: "Body",
    version: 1,
    is_builtin: true,
    created_at: 1,
    updated_at: 1,
    ...overrides,
  };
}

function createInitialState(): ResearchProjectsWorkflowState {
  return {
    projectsRaw: [],
    projectSources: [],
    runs: [],
    libraryRecords: [],
    sourceJobs: [],
    promptTemplates: [],
    projects: [],
    librarySources: [],
    projectSourceLinks: [],
    selectedProjectId: null,
    selectedLibrarySourceIds: new Set<string>(),
    loading: false,
    saving: false,
    status: "",
  };
}

function createDeps(state: ResearchProjectsWorkflowState) {
  return {
    getState: () => state,
    patch: vi.fn((patch: Partial<ResearchProjectsWorkflowState>) => Object.assign(state, patch)),
    listProjects: vi.fn(),
    listProjectSources: vi.fn(),
    listLibrarySources: vi.fn(),
    listProjectRuns: vi.fn(),
    listPromptTemplates: vi.fn(),
    listSourceJobs: vi.fn(),
    addProjectSources: vi.fn(),
    removeProjectSources: vi.fn(),
    createProject: vi.fn(),
    updateProject: vi.fn(),
    deleteProject: vi.fn(),
    startProjectAnalysis: vi.fn(),
    formatError: vi.fn((action: string, error: unknown) => `Error ${action}: ${String(error)}`),
  };
}

describe("research projects workflow", () => {
  it("loads projects and connects selected Library sources through project APIs", async () => {
    const state = createInitialState();
    const deps = createDeps(state);
    deps.listProjects.mockResolvedValue([project()]);
    deps.listProjectSources.mockResolvedValue([]);
    deps.listLibrarySources.mockResolvedValue([librarySource({ source_id: 10 })]);
    deps.listProjectRuns.mockResolvedValue([]);
    deps.listPromptTemplates.mockResolvedValue([]);
    deps.listSourceJobs.mockResolvedValue([]);
    deps.addProjectSources.mockResolvedValue({ added_count: 1, already_present_count: 0 });

    const workflow = createResearchProjectsWorkflow(deps);
    await workflow.loadWorkspace();
    state.selectedLibrarySourceIds = new Set(["source:10"]);
    await workflow.connectSelectedSources();

    expect(deps.addProjectSources).toHaveBeenCalledWith({ projectId: 1, sourceIds: [10] });
    expect(state.status).toContain("Connected sources: 1");
  });

  it("loads project sources, runs, prompts and source jobs into derived state", async () => {
    const state = createInitialState();
    const deps = createDeps(state);
    deps.listProjects.mockResolvedValue([project()]);
    deps.listProjectSources.mockResolvedValue([projectSource()]);
    deps.listLibrarySources.mockResolvedValue([librarySource()]);
    deps.listProjectRuns.mockResolvedValue([]);
    deps.listPromptTemplates.mockResolvedValue([promptTemplate()]);
    deps.listSourceJobs.mockResolvedValue([sourceJob()]);

    const workflow = createResearchProjectsWorkflow(deps);
    await workflow.loadWorkspace();

    expect(state.selectedProjectId).toBe("project:1");
    expect(state.projects[0].sourceCount).toBe(1);
    expect(state.projectSourceLinks).toHaveLength(1);
    expect(state.promptTemplates).toHaveLength(1);
    expect(state.librarySources[0].status).toBe("syncing");
  });
});
