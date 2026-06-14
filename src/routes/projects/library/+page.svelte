<script lang="ts">
  import { onMount } from "svelte";
  import LibraryScreen from "$lib/components/research-projects/LibraryScreen.svelte";
  import { listLibraryCatalog } from "$lib/api/library-sources";
  import {
    createLibraryCatalogWorkflow,
    type LibraryCatalogWorkflowState,
  } from "$lib/ui/library-catalog-workflow";

  const state = $state<LibraryCatalogWorkflowState>({
    catalogRecords: [],
    filterCounts: [],
    sources: [],
    loading: false,
    status: "",
  });

  const workflow = createLibraryCatalogWorkflow({
    getState: () => state,
    patch: (patch) => Object.assign(state, patch),
    listCatalog: listLibraryCatalog,
    formatError: (action, error) => `Error ${action}: ${String(error)}`,
  });

  onMount(() => {
    void workflow.loadLibrary();
  });
</script>

<section data-ui-route="library-prototype">
  <LibraryScreen {state} onRefresh={workflow.loadLibrary} />
</section>
