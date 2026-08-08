<script lang="ts">
  import type { LibraryCatalogWorkflowState } from "$lib/ui/library-catalog-workflow";

  let { state, onRefresh } = $props<{
    state: LibraryCatalogWorkflowState;
    onRefresh: () => void | Promise<void>;
  }>();
</script>

<main data-ui-screen="library">
  <h1>Library</h1>
  <output aria-label="Library source count">{state.sources.length}</output>
  <output aria-label="Library loading state">{String(state.loading)}</output>
  <button type="button" onclick={() => onRefresh()}>Refresh Library</button>
</main>
