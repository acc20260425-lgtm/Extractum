import {
  connectProjectSourceIds,
  connectedSourceIdsForProject,
  type ProjectAddSourceWorkflowDeps,
} from "./project-add-source-workflow";
import {
  buildLibrarySourcesView,
  buildProjectSourceLinksView,
  buildResearchProjectsView,
  connectableSelection,
  projectIdFromViewId,
  projectViewId,
  projectSourceLibraryDeleteStatus,
  selectedProjectSourceLibraryDeleteDisabledReason,
  selectedProjectSourcesSyncDisabledReason,
  type LibrarySourceView,
  type ProjectSourceLinkView,
  type ResearchProjectView,
} from "./research-projects-model";
import type { AnalysisPromptTemplate, AnalysisRunSummary } from "$lib/types/analysis";
import type { LibraryCatalogRecord, LibraryCatalogResponse } from "$lib/types/library-sources";
import type {
  AddProjectSourcesOutcome,
  DeleteProjectYoutubeVideoSourceInput,
  DeleteProjectYoutubeVideoSourceOutcome,
  ProjectAnalysisStartCommand,
  ProjectEditorInput,
  ProjectRecord,
  ProjectSourceRecord,
  ProjectSourcesInput,
  UpdateProjectInput,
} from "$lib/types/projects";
import type { SourceJobRecord, YoutubeSyncOptions } from "$lib/types/sources";

export interface ResearchProjectsWorkflowState {
  projectsRaw: ProjectRecord[];
  projectSources: ProjectSourceRecord[];
  runs: AnalysisRunSummary[];
  libraryCatalogRecords: LibraryCatalogRecord[];
  sourceJobs: SourceJobRecord[];
  promptTemplates: AnalysisPromptTemplate[];
  projects: ResearchProjectView[];
  librarySources: LibrarySourceView[];
  projectSourceLinks: ProjectSourceLinkView[];
  selectedProjectId: string | null;
  selectedLibrarySourceIds: Set<string>;
  loading: boolean;
  saving: boolean;
  status: string;
}

export interface ResearchProjectsWorkflowDeps {
  getState(): ResearchProjectsWorkflowState;
  patch(patch: Partial<ResearchProjectsWorkflowState>): void;
  listProjects(): Promise<ProjectRecord[]>;
  listProjectSources(projectId: number): Promise<ProjectSourceRecord[]>;
  listLibraryCatalog(): Promise<LibraryCatalogResponse>;
  listProjectRuns(projectId: number): Promise<AnalysisRunSummary[]>;
  listPromptTemplates(): Promise<AnalysisPromptTemplate[]>;
  listSourceJobs(): Promise<SourceJobRecord[]>;
  addProjectSources(input: ProjectSourcesInput): Promise<AddProjectSourcesOutcome>;
  removeProjectSources(input: ProjectSourcesInput): Promise<void>;
  deleteProjectYoutubeVideoSourceFromLibrary(
    input: DeleteProjectYoutubeVideoSourceInput,
  ): Promise<DeleteProjectYoutubeVideoSourceOutcome>;
  createProject(input: ProjectEditorInput): Promise<ProjectRecord>;
  updateProject(input: UpdateProjectInput): Promise<ProjectRecord>;
  deleteProject(projectId: number): Promise<void>;
  startProjectAnalysis(input: ProjectAnalysisStartCommand): Promise<number>;
  syncYoutubeSource(sourceId: number, options: YoutubeSyncOptions): Promise<SourceJobRecord>;
  formatError(action: string, error: unknown): string;
}

export interface LoadWorkspaceOptions {
  clearQueuedSyncStatus?: boolean;
}

function selectedProject(projects: ResearchProjectView[], selectedProjectId: string | null) {
  return projects.find((project) => project.id === selectedProjectId) ?? projects[0] ?? null;
}

export function createResearchProjectsWorkflow(deps: ResearchProjectsWorkflowDeps) {
  async function refreshDerivedState() {
    const state = deps.getState();
    const projects = buildResearchProjectsView(state.projectsRaw, state.projectSources, state.runs);
    const currentProject = selectedProject(projects, state.selectedProjectId);
    const selectedProjectId = currentProject?.id ?? null;
    const librarySources = buildLibrarySourcesView(
      state.libraryCatalogRecords,
      state.projectSources,
      selectedProjectId,
    );
    deps.patch({
      projects,
      selectedProjectId,
      librarySources,
      projectSourceLinks: buildProjectSourceLinksView(selectedProjectId, state.projectSources),
    });
  }

  async function loadWorkspace(options: LoadWorkspaceOptions = {}) {
    deps.patch({ loading: true });
    try {
      const [projectsRaw, libraryCatalog, sourceJobs, promptTemplates] = await Promise.all([
        deps.listProjects(),
        deps.listLibraryCatalog(),
        deps.listSourceJobs(),
        deps.listPromptTemplates(),
      ]);
      const allProjectSources = (
        await Promise.all(projectsRaw.map((project) => deps.listProjectSources(project.id)))
      ).flat();
      const runs = (
        await Promise.all(projectsRaw.map((project) => deps.listProjectRuns(project.id)))
      ).flat();
      deps.patch({
        projectsRaw,
        libraryCatalogRecords: libraryCatalog.sources,
        projectSources: allProjectSources,
        runs,
        sourceJobs,
        promptTemplates,
      });
      await refreshDerivedState();
      if (options.clearQueuedSyncStatus && deps.getState().status.startsWith("Queued sync for ")) {
        deps.patch({ status: "" });
      }
    } catch (error) {
      deps.patch({ status: deps.formatError("loading research projects", error) });
    } finally {
      deps.patch({ loading: false });
    }
  }

  async function connectSelectedSources() {
    const state = deps.getState();
    const projectId = projectIdFromViewId(state.selectedProjectId);
    if (!projectId) {
      deps.patch({ status: "Select a project" });
      return;
    }
    const sourceIds = connectableSelection(state.librarySources, state.selectedLibrarySourceIds).map(
      (source) => source.sourceId,
    );
    if (sourceIds.length === 0) {
      deps.patch({ status: "No selected sources can be connected." });
      return;
    }

    deps.patch({ saving: true });
    try {
      const outcome = await deps.addProjectSources({ projectId, sourceIds });
      deps.patch({
        status: `Connected sources: ${outcome.added_count}. Already in project: ${outcome.already_present_count}.`,
        selectedLibrarySourceIds: new Set<string>(),
      });
      await loadWorkspace();
    } catch (error) {
      deps.patch({ status: deps.formatError("connecting library sources", error) });
    } finally {
      deps.patch({ saving: false });
    }
  }

  async function removeProjectSource(sourceIdOrIds: number | number[]) {
    const sourceIds = Array.isArray(sourceIdOrIds) ? sourceIdOrIds : [sourceIdOrIds];
    const projectId = projectIdFromViewId(deps.getState().selectedProjectId);
    if (!projectId) {
      deps.patch({ status: "Select a project" });
      return;
    }
    deps.patch({ saving: true });
    try {
      await deps.removeProjectSources({ projectId, sourceIds });
      await loadWorkspace();
    } catch (error) {
      deps.patch({ status: deps.formatError("removing project source", error) });
    } finally {
      deps.patch({ saving: false });
    }
  }

  async function createProject(input: ProjectEditorInput) {
    deps.patch({ saving: true });
    try {
      const project = await deps.createProject(input);
      deps.patch({ selectedProjectId: projectViewId(project.id), status: "Project created." });
      await loadWorkspace();
    } catch (error) {
      deps.patch({ status: deps.formatError("creating project", error) });
    } finally {
      deps.patch({ saving: false });
    }
  }

  async function updateProject(input: ProjectEditorInput) {
    const projectId = projectIdFromViewId(deps.getState().selectedProjectId);
    if (!projectId) {
      deps.patch({ status: "Select a project" });
      return;
    }
    deps.patch({ saving: true });
    try {
      await deps.updateProject({ projectId, ...input });
      deps.patch({ status: "Project updated." });
      await loadWorkspace();
    } catch (error) {
      deps.patch({ status: deps.formatError("updating project", error) });
    } finally {
      deps.patch({ saving: false });
    }
  }

  async function deleteSelectedProject() {
    const projectId = projectIdFromViewId(deps.getState().selectedProjectId);
    if (!projectId) {
      deps.patch({ status: "Select a project" });
      return;
    }
    deps.patch({ saving: true });
    try {
      await deps.deleteProject(projectId);
      deps.patch({ selectedProjectId: null, status: "Project deleted." });
      await loadWorkspace();
    } catch (error) {
      deps.patch({ status: deps.formatError("deleting project", error) });
    } finally {
      deps.patch({ saving: false });
    }
  }

  async function runProjectAnalysis(input: ProjectAnalysisStartCommand) {
    deps.patch({ saving: true });
    try {
      const runId = await deps.startProjectAnalysis(input);
      const queuedStatus = `Project analysis queued: ${runId}`;
      deps.patch({ status: queuedStatus });
      await loadWorkspace();
      if (deps.getState().status === queuedStatus) {
        deps.patch({ status: "" });
      }
    } catch (error) {
      deps.patch({ status: deps.formatError("starting project analysis", error) });
    } finally {
      deps.patch({ saving: false });
    }
  }

  async function syncProjectSources(sourceIds: number[]) {
    const selectedSourceIds = new Set(sourceIds);
    const sources = deps
      .getState()
      .projectSourceLinks.filter((source) => selectedSourceIds.has(source.sourceNumericId));
    const disabledReason = selectedProjectSourcesSyncDisabledReason(sources);
    if (disabledReason) {
      deps.patch({ status: disabledReason });
      return;
    }

    deps.patch({ saving: true });
    try {
      const options = { metadata: true, transcripts: true, comments: true };
      for (const source of sources) {
        await deps.syncYoutubeSource(source.sourceNumericId, options);
      }
      deps.patch({
        status: `Queued sync for ${sources.length} ${sources.length === 1 ? "source" : "sources"}.`,
      });
      await loadWorkspace();
    } catch (error) {
      deps.patch({ status: deps.formatError("syncing project sources", error) });
    } finally {
      deps.patch({ saving: false });
    }
  }

  async function deleteProjectYoutubeVideoSourceFromLibrary(sourceId: number) {
    const state = deps.getState();
    const projectId = projectIdFromViewId(state.selectedProjectId);
    if (!projectId) {
      deps.patch({ status: "Select a project" });
      return;
    }

    const row = state.projectSourceLinks.find((source) => source.sourceNumericId === sourceId);
    const disabledReason = selectedProjectSourceLibraryDeleteDisabledReason(row ? [row] : []);
    if (disabledReason) {
      deps.patch({ status: disabledReason });
      return;
    }

    deps.patch({ saving: true });
    try {
      const outcome = await deps.deleteProjectYoutubeVideoSourceFromLibrary({ projectId, sourceId });
      deps.patch({ status: projectSourceLibraryDeleteStatus(outcome) });
      if (outcome.status === "deleted") {
        await loadWorkspace();
      }
    } catch (error) {
      deps.patch({ status: deps.formatError("deleting project source from Library", error) });
    } finally {
      deps.patch({ saving: false });
    }
  }

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

  return {
    refreshDerivedState,
    loadWorkspace,
    connectSelectedSources,
    removeProjectSource,
    createProject,
    updateProject,
    deleteSelectedProject,
    runProjectAnalysis,
    syncProjectSources,
    deleteProjectYoutubeVideoSourceFromLibrary,
    connectAddedProjectSource,
    connectAddedProjectSources,
    connectExistingProjectSource,
    setStatus,
  };
}
