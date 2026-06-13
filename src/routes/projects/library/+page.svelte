<script lang="ts">
  import { onMount } from "svelte";
  import LibraryScreen from "$lib/components/research-projects/LibraryScreen.svelte";
  import { listLibrarySources } from "$lib/api/library-sources";
  import { listSourceJobs } from "$lib/api/source-jobs";
  import {
    createLibraryCatalogWorkflow,
    type LibraryCatalogWorkflowState,
  } from "$lib/ui/library-catalog-workflow";

  const state = $state<LibraryCatalogWorkflowState>({
    sourceRecords: [],
    sourceJobs: [],
    sources: [],
    loading: false,
    status: "",
  });

  const workflow = createLibraryCatalogWorkflow({
    getState: () => state,
    patch: (patch) => Object.assign(state, patch),
    listSources: listLibrarySources,
    listSourceJobs: () => listSourceJobs({ limit: 50 }),
    formatError: (action, error) => `Error ${action}: ${String(error)}`,
  });

  onMount(() => {
    void workflow.loadLibrary();
  });
</script>

<section data-ui-route="library-prototype">
  <LibraryScreen {state} onRefresh={workflow.loadLibrary} />
</section>
