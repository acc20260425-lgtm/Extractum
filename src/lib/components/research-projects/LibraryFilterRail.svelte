<script lang="ts">
  import { PanelLeftClose, PanelLeftOpen } from "@lucide/svelte";
  import { ExtractumButton, ExtractumTreeDataGrid } from "$lib/components/extractum-ui";
  import type {
    LibraryCatalogFilterId,
    LibraryCatalogFilterTreeRow,
  } from "$lib/ui/library-catalog-model";

  let {
    rows,
    selectedFilterId,
    collapsed,
    onSelectedFilterIdChange,
    onCollapsedChange,
  }: {
    rows: LibraryCatalogFilterTreeRow[];
    selectedFilterId: LibraryCatalogFilterId;
    collapsed: boolean;
    onSelectedFilterIdChange: (id: LibraryCatalogFilterId) => void;
    onCollapsedChange: (collapsed: boolean) => void;
  } = $props();
</script>

<aside
  data-ui-region="library-filter-rail"
  class:collapsed
  class="library-filter-rail"
  aria-label="Library filters"
>
  <div class="rail-header">
    {#if !collapsed}
      <h1 class="rail-title">Library</h1>
    {:else}
      <h1 class="sr-only">Library</h1>
    {/if}
    <ExtractumButton
      variant="ghost"
      size="icon"
      aria-label={collapsed ? "Expand Library filters" : "Collapse Library filters"}
      title={collapsed ? "Expand Library filters" : "Collapse Library filters"}
      onclick={() => onCollapsedChange(!collapsed)}
    >
      {#if collapsed}
        <PanelLeftOpen size={15} aria-hidden="true" />
      {:else}
        <PanelLeftClose size={15} aria-hidden="true" />
      {/if}
    </ExtractumButton>
  </div>

  <ExtractumTreeDataGrid
    rows={rows}
    selectedRowId={selectedFilterId}
    {collapsed}
    height="100%"
    onSelectedRowIdChange={(id) => {
      if (id) onSelectedFilterIdChange(id as LibraryCatalogFilterId);
    }}
  />
</aside>

<style>
  .library-filter-rail {
    display: flex;
    min-width: 0;
    min-height: 0;
    flex-direction: column;
    border-right: 1px solid var(--extractum-border);
    background: var(--extractum-surface-raised);
  }

  .rail-header {
    display: flex;
    min-height: 40px;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 6px 8px;
    border-bottom: 1px solid var(--extractum-border);
    color: var(--extractum-muted);
    font-size: 12px;
    font-weight: 700;
    text-transform: uppercase;
  }

  .rail-title {
    margin: 0;
    color: inherit;
    font-size: inherit;
    font-weight: inherit;
    text-transform: inherit;
  }

  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }

  .library-filter-rail.collapsed .rail-header {
    justify-content: center;
    padding-inline: 4px;
  }
</style>
