<script lang="ts">
  import { onMount } from "svelte";
  import LibraryScreen from "$lib/components/research-projects/LibraryScreen.svelte";
  import { listAnalysisSourceGroups, updateAnalysisSourceGroup } from "$lib/api/analysis-source-groups";
  import { listAnalysisSources } from "$lib/api/analysis-workspace";
  import { listActiveAnalysisRuns } from "$lib/api/analysis-runs";
  import { listSourceJobs } from "$lib/api/source-jobs";
  import {
    createResearchProjectsWorkflow,
    type ResearchProjectsWorkflowState,
  } from "$lib/ui/research-projects-workflow";

  const state = $state<ResearchProjectsWorkflowState>({
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
  });

  const workflow = createResearchProjectsWorkflow({
    getState: () => state,
    patch: (patch) => Object.assign(state, patch),
    listGroups: listAnalysisSourceGroups,
    listSources: listAnalysisSources,
    listRuns: listActiveAnalysisRuns,
    listSourceJobs: () => listSourceJobs({ limit: 50 }),
    updateGroup: updateAnalysisSourceGroup,
    formatError: (action, error) => `Ошибка ${action}: ${String(error)}`,
  });

  onMount(() => {
    void workflow.loadWorkspace();
  });
</script>

<section data-ui-route="library-prototype">
  <LibraryScreen {state} onRefresh={workflow.loadWorkspace} />
</section>
