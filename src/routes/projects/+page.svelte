<script lang="ts">
  import { onMount } from "svelte";
  import { projectsSharedState } from "$lib/projects-shared.svelte";
  import ProjectsShell from "$lib/components/research-projects/ProjectsShell.svelte";
  import { listenToAnalysisRunEvents } from "$lib/api/analysis-runs";
  import { listAnalysisPromptTemplates } from "$lib/api/analysis-source-groups";
  import { listLibraryCatalog } from "$lib/api/library-sources";
  import {
    addProjectSources,
    createProject,
    deleteProject,
    deleteProjectYoutubeVideoSourceFromLibrary,
    listProjectRuns,
    listProjectSources,
    listProjects,
    removeProjectSources,
    startProjectAnalysis,
    updateProject,
  } from "$lib/api/projects";
  import { listenToSourceJobEvents, listSourceJobs, syncYoutubeSource } from "$lib/api/source-jobs";
  import { formatAppError } from "$lib/app-error";
  import {
    createResearchProjectsWorkflow,
    type ResearchProjectsWorkflowState,
  } from "$lib/ui/research-projects-workflow";

  const state = $state<ResearchProjectsWorkflowState>({
    projectsRaw: [],
    projectSources: [],
    runs: [],
    libraryCatalogRecords: [],
    sourceJobs: [],
    promptTemplates: [],
    projects: [],
    librarySources: [],
    projectSourceLinks: [],
    // Restore previously selected project so effects don't overwrite shared state with null on mount
    selectedProjectId: projectsSharedState.selectedProjectId,
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
    listLibraryCatalog,
    listProjectRuns,
    listPromptTemplates: () => listAnalysisPromptTemplates("report"),
    listSourceJobs: () => listSourceJobs({ limit: 50 }),
    addProjectSources,
    removeProjectSources,
    createProject,
    updateProject,
    deleteProject,
    startProjectAnalysis,
    syncYoutubeSource,
    deleteProjectYoutubeVideoSourceFromLibrary,
    formatError: formatAppError,
  });

  let projectRunsRefreshTimer: ReturnType<typeof setTimeout> | null = null;
  let clearQueuedSyncStatusOnNextRefresh = false;

  function scheduleProjectRunsRefresh(options: { clearQueuedSyncStatus?: boolean } = {}) {
    clearQueuedSyncStatusOnNextRefresh ||= options.clearQueuedSyncStatus ?? false;
    if (projectRunsRefreshTimer) {
      clearTimeout(projectRunsRefreshTimer);
    }
    projectRunsRefreshTimer = setTimeout(() => {
      projectRunsRefreshTimer = null;
      const clearQueuedSyncStatus = clearQueuedSyncStatusOnNextRefresh;
      clearQueuedSyncStatusOnNextRefresh = false;
      void workflow.loadWorkspace({ clearQueuedSyncStatus });
    }, 350);
  }

  function shouldRefreshForRunEvent(kind: string) {
    return kind === "queued" || kind === "started" || kind === "completed" || kind === "failed" || kind === "cancelled";
  }

  function shouldRefreshForSourceJobStatus(status: string) {
    return status === "succeeded" || status === "failed" || status === "cancelled";
  }

  onMount(() => {
    let disposed = false;
    let unlistenAnalysisRuns: (() => void) | null = null;
    let unlistenSourceJobs: (() => void) | null = null;

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
          unlistenAnalysisRuns = nextUnlisten;
        }
      })
      .catch((error) => {
        state.status = formatAppError("listening to analysis run events", error);
      });
    void listenToSourceJobEvents((job) => {
      if (shouldRefreshForSourceJobStatus(job.status)) {
        scheduleProjectRunsRefresh({ clearQueuedSyncStatus: true });
      }
    })
      .then((nextUnlisten) => {
        if (disposed) {
          nextUnlisten();
        } else {
          unlistenSourceJobs = nextUnlisten;
        }
      })
      .catch((error) => {
        state.status = formatAppError("listening to source job events", error);
      });

    return () => {
      disposed = true;
      unlistenAnalysisRuns?.();
      unlistenSourceJobs?.();
      if (projectRunsRefreshTimer) {
        clearTimeout(projectRunsRefreshTimer);
      }
    };
  });

  // Sync state with global projectsSharedState
  $effect(() => {
    projectsSharedState.projects = state.projects;
  });

  $effect(() => {
    if (projectsSharedState.selectedProjectId !== state.selectedProjectId) {
      projectsSharedState.selectedProjectId = state.selectedProjectId;
    }
  });

  $effect(() => {
    if (projectsSharedState.selectedProjectId !== state.selectedProjectId) {
      if (projectsSharedState.selectedProjectId) {
        selectProject(projectsSharedState.selectedProjectId);
      }
    }
  });

  function selectProject(projectId: string) {
    state.selectedProjectId = projectId;
    void workflow.refreshDerivedState();
  }
</script>

<section data-ui-route="research-projects">
  <!-- Temporary switch to the in-progress v10 shell; remove when /projects adopts it. -->
  <a class="rp-ui-switch" href="/projects/next">Новый интерфейс (beta) →</a>
  <ProjectsShell
    {state}
    showRail={false}
    onSelectProject={selectProject}
    onCreateProject={workflow.createProject}
    onUpdateProject={workflow.updateProject}
    onDeleteProject={workflow.deleteSelectedProject}
    onRemoveProjectSource={workflow.removeProjectSource}
    onRunProject={workflow.runProjectAnalysis}
    onConnectSelectedSources={workflow.connectSelectedSources}
    onConnectAddedProjectSource={workflow.connectAddedProjectSource}
    onConnectAddedProjectSources={workflow.connectAddedProjectSources}
    onConnectExistingProjectSource={workflow.connectExistingProjectSource}
    onSelectedLibrarySourceIdsChange={(ids) => (state.selectedLibrarySourceIds = new Set(ids))}
    onRefreshProjectRuns={workflow.loadWorkspace}
    onSyncSelectedSources={workflow.syncProjectSources}
    onDeleteProjectSourceFromLibrary={workflow.deleteProjectYoutubeVideoSourceFromLibrary}
    onSetStatus={workflow.setStatus}
  />
</section>

<style>
  .rp-ui-switch {
    position: fixed;
    bottom: 6px;
    right: 12px;
    z-index: 9999;
    font: 400 11px/1 var(--extractum-font);
    color: var(--extractum-muted-2);
    text-decoration: none;
    opacity: 0.75;
  }

  .rp-ui-switch:hover {
    opacity: 1;
    text-decoration: underline;
  }
</style>
