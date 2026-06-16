import {
  buildLibrarySourcesView,
  buildProjectSourceLinksView,
  buildResearchProjectsView,
  connectableSelection,
  projectIdFromViewId,
  projectViewId,
  type LibrarySourceView,
  type ProjectSourceLinkView,
  type ResearchProjectView,
} from "./research-projects-model";
import type { AnalysisPromptTemplate, AnalysisRunSummary } from "$lib/types/analysis";
import type { LibraryCatalogRecord, LibraryCatalogResponse } from "$lib/types/library-sources";
import type {
  AddProjectSourcesOutcome,
  ProjectAnalysisStartCommand,
  ProjectEditorInput,
  ProjectRecord,
  ProjectSourceRecord,
  ProjectSourcesInput,
  UpdateProjectInput,
} from "$lib/types/projects";
import type { SourceJobRecord } from "$lib/types/sources";

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
  createProject(input: ProjectEditorInput): Promise<ProjectRecord>;
  updateProject(input: UpdateProjectInput): Promise<ProjectRecord>;
  deleteProject(projectId: number): Promise<void>;
  startProjectAnalysis(input: ProjectAnalysisStartCommand): Promise<number>;
  formatError(action: string, error: unknown): string;
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

  async function loadWorkspace() {
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

  return {
    refreshDerivedState,
    loadWorkspace,
    connectSelectedSources,
    removeProjectSource,
    createProject,
    updateProject,
    deleteSelectedProject,
    runProjectAnalysis,
  };
}
