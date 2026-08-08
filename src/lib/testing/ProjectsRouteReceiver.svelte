<script lang="ts">
  import type { ComponentProps } from "svelte";
  import type ResearchProjectsShell from "$lib/components/research-projects/ResearchProjectsShell.svelte";

  type Props = ComponentProps<typeof ResearchProjectsShell>;

  let {
    railPanel,
    filterBar,
    onSelectedSourceIdsChange,
    bulkBar,
  }: Props = $props();
</script>

<main>
  <button aria-label="Select route project" onclick={() => railPanel.onSelect?.(1)}>
    Select project
  </button>
  <button aria-label="Open route add source" onclick={() => filterBar?.onAddSource?.()}>
    Add source
  </button>
  <button aria-label="Select route source" onclick={() => onSelectedSourceIdsChange?.(["11"])}>
    Select source
  </button>
  <button
    aria-label="Select ineligible route source"
    onclick={() => onSelectedSourceIdsChange?.(["12"])}
  >
    Select ineligible source
  </button>
  <button
    aria-label="Select mixed route sources"
    onclick={() => onSelectedSourceIdsChange?.(["11", "12"])}
  >
    Select mixed sources
  </button>
  <button
    aria-label="Run route Library delete"
    disabled={bulkBar?.libraryDeleteDisabled ?? true}
    title={bulkBar?.libraryDeleteTitle ?? ""}
    onclick={() => bulkBar?.onDeleteFromLibrary?.()}
  >
    Delete from Library
  </button>
  <button
    aria-label="Run route sync"
    disabled={bulkBar?.syncDisabled ?? true}
    title={bulkBar?.syncTitle ?? ""}
    onclick={() => bulkBar?.onSync?.()}
  >
    Sync
  </button>
</main>
