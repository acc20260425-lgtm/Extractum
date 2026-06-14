<script lang="ts">
  import { onMount } from "svelte";
  import ProjectsShell from "$lib/components/research-projects/ProjectsShell.svelte";
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
    formatError: (action, error) => `Error ${action}: ${String(error)}`,
  });

  onMount(() => {
    void workflow.loadWorkspace();
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
  />
</section>
