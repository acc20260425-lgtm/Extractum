<script lang="ts">
  import { onMount } from "svelte";
  import ProjectsShell from "$lib/components/research-projects/ProjectsShell.svelte";
  import { listenToAnalysisRunEvents } from "$lib/api/analysis-runs";
  import { listAnalysisPromptTemplates } from "$lib/api/analysis-source-groups";
  import { listLibrarySources } from "$lib/api/library-sources";
  import {
    addProjectSources,
    createProject,
    deleteProject,
    listProjectRuns,
    listProjectSources,
    listProjects,
    removeProjectSources,
    startProjectAnalysis,
    updateProject,
  } from "$lib/api/projects";
  import { listSourceJobs } from "$lib/api/source-jobs";
  import { formatAppError } from "$lib/app-error";
  import {
    createResearchProjectsWorkflow,
    type ResearchProjectsWorkflowState,
  } from "$lib/ui/research-projects-workflow";

  const state = $state<ResearchProjectsWorkflowState>({
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
  });

  const workflow = createResearchProjectsWorkflow({
    getState: () => state,
    patch: (patch) => Object.assign(state, patch),
    listProjects,
    listProjectSources,
    listLibrarySources,
    listProjectRuns,
    listPromptTemplates: () => listAnalysisPromptTemplates("report"),
    listSourceJobs: () => listSourceJobs({ limit: 50 }),
    addProjectSources,
    removeProjectSources,
    createProject,
    updateProject,
    deleteProject,
    startProjectAnalysis,
    formatError: formatAppError,
  });

  let projectRunsRefreshTimer: ReturnType<typeof setTimeout> | null = null;

  function scheduleProjectRunsRefresh() {
    if (projectRunsRefreshTimer) {
      clearTimeout(projectRunsRefreshTimer);
    }
    projectRunsRefreshTimer = setTimeout(() => {
      projectRunsRefreshTimer = null;
      void workflow.loadWorkspace();
    }, 350);
  }

  function shouldRefreshForRunEvent(kind: string) {
    return kind === "queued" || kind === "started" || kind === "completed" || kind === "failed" || kind === "cancelled";
  }

  onMount(() => {
    let disposed = false;
    let unlisten: (() => void) | null = null;

    void workflow.loadWorkspace();
    void listenToAnalysisRunEvents(({ payload }) => {
      if (shouldRefreshForRunEvent(payload.kind)) {
        scheduleProjectRunsRefresh();
      }
    })
      .then((nextUnlisten) => {
        if (disposed) {
          nextUnlisten();
        } else {
          unlisten = nextUnlisten;
        }
      })
      .catch((error) => {
        state.status = formatAppError("listening to analysis run events", error);
      });

    return () => {
      disposed = true;
      unlisten?.();
      if (projectRunsRefreshTimer) {
        clearTimeout(projectRunsRefreshTimer);
      }
    };
  });

  function selectProject(projectId: string) {
    state.selectedProjectId = projectId;
    void workflow.refreshDerivedState();
  }
</script>

<section data-ui-route="research-projects">
  <ProjectsShell
    {state}
    onSelectProject={selectProject}
    onCreateProject={workflow.createProject}
    onUpdateProject={workflow.updateProject}
    onDeleteProject={workflow.deleteSelectedProject}
    onRemoveProjectSource={workflow.removeProjectSource}
    onRunProject={workflow.runProjectAnalysis}
    onConnectSelectedSources={workflow.connectSelectedSources}
    onSelectedLibrarySourceIdsChange={(ids) => (state.selectedLibrarySourceIds = new Set(ids))}
    onRefreshProjectRuns={workflow.loadWorkspace}
  />
</section>
