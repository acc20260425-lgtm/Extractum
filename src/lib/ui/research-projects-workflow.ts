import {
  buildLibrarySourcesView,
  buildProjectSourceLinksView,
  buildResearchProjectsView,
  buildSourceGroupUpdateInput,
  type LibrarySourceView,
  type ProjectSourceLinkView,
  type ResearchProjectView,
} from "./research-projects-model";
import type {
  AnalysisRunSummary,
  AnalysisSourceGroup,
  AnalysisSourceOption,
  UpdateAnalysisSourceGroupInput,
} from "$lib/types/analysis";
import type { SourceJobRecord } from "$lib/types/sources";

export interface ResearchProjectsWorkflowState {
  groups: AnalysisSourceGroup[];
  sources: AnalysisSourceOption[];
  runs: AnalysisRunSummary[];
  sourceJobs: SourceJobRecord[];
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
  listGroups(): Promise<AnalysisSourceGroup[]>;
  listSources(): Promise<AnalysisSourceOption[]>;
  listRuns(): Promise<AnalysisRunSummary[]>;
  listSourceJobs(): Promise<SourceJobRecord[]>;
  updateGroup(input: UpdateAnalysisSourceGroupInput): Promise<AnalysisSourceGroup>;
  formatError(action: string, error: unknown): string;
}

function selectedProject(projects: ResearchProjectView[], selectedProjectId: string | null) {
  return projects.find((project) => project.id === selectedProjectId) ?? projects[0] ?? null;
}

function selectedGroup(groups: AnalysisSourceGroup[], selectedProjectId: string | null) {
  if (!selectedProjectId?.startsWith("source-group:")) return null;
  const groupId = Number(selectedProjectId.replace("source-group:", ""));
  return groups.find((group) => group.id === groupId) ?? null;
}

export function createResearchProjectsWorkflow(deps: ResearchProjectsWorkflowDeps) {
  async function refreshDerivedState() {
    const state = deps.getState();
    const projects = buildResearchProjectsView(state.groups, state.runs);
    const currentProject = selectedProject(projects, state.selectedProjectId);
    const selectedProjectId = currentProject?.id ?? null;
    const librarySources = buildLibrarySourcesView(
      state.sources,
      state.groups,
      selectedProjectId,
      state.sourceJobs,
    );
    deps.patch({
      projects,
      selectedProjectId,
      librarySources,
      projectSourceLinks: buildProjectSourceLinksView(selectedProjectId, librarySources),
    });
  }

  async function loadWorkspace() {
    deps.patch({ loading: true });
    try {
      const [groups, sources, runs, sourceJobs] = await Promise.all([
        deps.listGroups(),
        deps.listSources(),
        deps.listRuns(),
        deps.listSourceJobs(),
      ]);
      deps.patch({ groups, sources, runs, sourceJobs });
      await refreshDerivedState();
    } catch (error) {
      deps.patch({ status: deps.formatError("loading research projects", error) });
    } finally {
      deps.patch({ loading: false });
    }
  }

  async function connectSelectedSources() {
    const state = deps.getState();
    const project = selectedProject(state.projects, state.selectedProjectId);
    const group = selectedGroup(state.groups, state.selectedProjectId);
    const decision = buildSourceGroupUpdateInput(
      project,
      group,
      state.selectedLibrarySourceIds,
      state.librarySources,
    );

    if (!decision.ok) {
      deps.patch({ status: decision.reason });
      return;
    }

    deps.patch({ saving: true });
    try {
      await deps.updateGroup(decision.input);
      deps.patch({
        status: decision.refusedCount > 0
          ? `Подключено источников: ${decision.connectedCount}. Отклонено: ${decision.refusedCount}.`
          : `Подключено источников: ${decision.connectedCount}.`,
        selectedLibrarySourceIds: new Set<string>(),
      });
      await loadWorkspace();
    } catch (error) {
      deps.patch({ status: deps.formatError("connecting library sources", error) });
    } finally {
      deps.patch({ saving: false });
    }
  }

  return {
    refreshDerivedState,
    loadWorkspace,
    connectSelectedSources,
  };
}
