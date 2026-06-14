import { describe, expect, it, vi } from "vitest";
import { createResearchProjectsWorkflow, type ResearchProjectsWorkflowState } from "./research-projects-workflow";
import type { AnalysisPromptTemplate, AnalysisRunSummary } from "$lib/types/analysis";
import type { LibraryCatalogRecord, LibrarySourceRecord } from "$lib/types/library-sources";
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

function libraryCatalogRecord(overrides: Partial<LibraryCatalogRecord> = {}): LibraryCatalogRecord {
  return {
    source: librarySource(),
    latest_job: null,
    status: "active",
    status_detail: null,
    capabilities: {
      can_refresh_source: true,
      can_delete: true,
      can_edit: false,
      can_connect_to_project: true,
    },
    disabled_reasons: {
      refresh_source: null,
      delete: null,
      edit: "Source editing is not available yet.",
      connect_to_project: null,
    },
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

function analysisRun(overrides: Partial<AnalysisRunSummary> = {}): AnalysisRunSummary {
  return {
    id: 403,
    run_type: "report",
    scope_type: "project",
    source_id: null,
    source_title: null,
    source_group_id: null,
    source_group_name: null,
    project_id: 1,
    project_name: "Alpha",
    scope_label: "Alpha",
    period_from: 0,
    period_to: 1,
    output_language: "English",
    prompt_template_id: 1,
    prompt_template_name: "Default",
    prompt_template_version: 1,
    provider_profile: "default",
    provider: "openai_compatible",
    model: "gpt-4.1",
    youtube_corpus_mode: "transcript_description",
    telegram_history_scope: "current",
    status: "completed",
    error: null,
    has_trace_data: false,
    snapshot_state: "captured",
    snapshot_captured_at: "2026-06-14T00:00:00Z",
    snapshot_error: null,
    created_at: 1,
    completed_at: 2,
    ...overrides,
  };
}

function createInitialState(): ResearchProjectsWorkflowState {
  return {
    projectsRaw: [],
    projectSources: [],
    runs: [],
    libraryCatalogRecords: [],
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
    listLibraryCatalog: vi.fn(),
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
    deps.listLibraryCatalog.mockResolvedValue({
      sources: [libraryCatalogRecord({ source: librarySource({ source_id: 10 }) })],
      filter_counts: [],
    });
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
    deps.listLibraryCatalog.mockResolvedValue({
      sources: [libraryCatalogRecord()],
      filter_counts: [],
    });
    deps.listProjectRuns.mockResolvedValue([]);
    deps.listPromptTemplates.mockResolvedValue([promptTemplate()]);
    deps.listSourceJobs.mockResolvedValue([sourceJob()]);

    const workflow = createResearchProjectsWorkflow(deps);
    await workflow.loadWorkspace();

    expect(state.selectedProjectId).toBe("project:1");
    expect(state.projects[0].sourceCount).toBe(1);
    expect(state.projectSourceLinks).toHaveLength(1);
    expect(state.promptTemplates).toHaveLength(1);
    expect(state.sourceJobs).toHaveLength(1);
    expect(state.librarySources[0].status).toBe("active");
    expect(deps.listSourceJobs).toHaveBeenCalledTimes(1);
  });

  it("clears queued project analysis status after successful workspace reload", async () => {
    const state = createInitialState();
    state.selectedProjectId = "project:1";
    const deps = createDeps(state);
    deps.startProjectAnalysis.mockResolvedValue(403);
    deps.listProjects.mockResolvedValue([project()]);
    deps.listProjectSources.mockResolvedValue([projectSource()]);
    deps.listLibraryCatalog.mockResolvedValue({
      sources: [libraryCatalogRecord()],
      filter_counts: [],
    });
    deps.listProjectRuns.mockResolvedValue([analysisRun({ id: 403, status: "completed" })]);
    deps.listPromptTemplates.mockResolvedValue([promptTemplate()]);
    deps.listSourceJobs.mockResolvedValue([]);

    const workflow = createResearchProjectsWorkflow(deps);
    await workflow.runProjectAnalysis({
      projectId: 1,
      periodFrom: 0,
      periodTo: 1,
      outputLanguage: "English",
      promptTemplateId: 1,
      profileId: null,
      modelOverride: null,
      youtubeCorpusMode: "transcript_description",
      includeMigratedHistory: false,
    });

    expect(deps.startProjectAnalysis).toHaveBeenCalledWith({
      projectId: 1,
      periodFrom: 0,
      periodTo: 1,
      outputLanguage: "English",
      promptTemplateId: 1,
      profileId: null,
      modelOverride: null,
      youtubeCorpusMode: "transcript_description",
      includeMigratedHistory: false,
    });
    expect(state.runs).toEqual([analysisRun({ id: 403, status: "completed" })]);
    expect(state.status).toBe("");
  });
});
