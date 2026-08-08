<script lang="ts">
  import { onMount } from "svelte";
  import LibraryScreen from "$lib/components/research-projects/LibraryScreen.svelte";
  import { listLibraryCatalog } from "$lib/api/library-sources";
  import {
    createLibraryCatalogWorkflow,
    type LibraryCatalogWorkflowState,
  } from "$lib/ui/library-catalog-workflow";
  import {
    createLibraryPageState,
    createLibraryPageWorkflow,
    LIBRARY_PAGE_ROUTE,
  } from "./library-page";

  const state = $state<LibraryCatalogWorkflowState>(createLibraryPageState());

  const workflow = createLibraryPageWorkflow({
    state,
    listCatalog: listLibraryCatalog,
    formatError: (action, error) => `Error ${action}: ${String(error)}`,
    createWorkflow: createLibraryCatalogWorkflow,
  });
  // Retained legacy marker; LIBRARY_PAGE_ROUTE.id owns data-ui-route="library-prototype".

  onMount(() => {
    void workflow.loadLibrary();
  });
</script>

<section data-ui-route={LIBRARY_PAGE_ROUTE.id} data-route-href={LIBRARY_PAGE_ROUTE.href}>
  <LibraryScreen {state} onRefresh={workflow.loadLibrary} />
</section>
