<script lang="ts">
  import LibraryAddSourceDialog from "./LibraryAddSourceDialog.svelte";
  import LibraryFilterRail from "./LibraryFilterRail.svelte";
  import LibraryInspector from "./LibraryInspector.svelte";
  import LibraryWorkspace from "./LibraryWorkspace.svelte";
  import {
    LIBRARY_CATALOG_ALL_FILTER_ID,
    buildLibraryCatalogFilterTree,
    filterLibraryCatalogSources,
    reconcileLibraryCatalogSourceSelection,
    type LibraryCatalogFilterId,
  } from "$lib/ui/library-catalog-model";
  import type { LibraryCatalogWorkflowState } from "$lib/ui/library-catalog-workflow";

  let {
    state: workflowState,
    onRefresh,
  }: {
    state: LibraryCatalogWorkflowState;
    onRefresh: () => void | Promise<void>;
  } = $props();

  let selectedFilterId = $state<LibraryCatalogFilterId>(LIBRARY_CATALOG_ALL_FILTER_ID);
  let selectedSourceId = $state<string | null>(null);
  let query = $state("");
  let filterCollapsed = $state(false);
  let inspectorWidth = $state(380);
  let status = $state("");
  let addSourceDialogOpen = $state(false);

  let filterRows = $derived(buildLibraryCatalogFilterTree(workflowState.sources));
  let visibleSources = $derived(
    filterLibraryCatalogSources(workflowState.sources, { filterId: selectedFilterId, query }),
  );
  let selectedSource = $derived(
    visibleSources.find((source) => source.id === selectedSourceId) ?? null,
  );

  $effect(() => {
    const nextSelectedId = reconcileLibraryCatalogSourceSelection(visibleSources, selectedSourceId);
    if (nextSelectedId !== selectedSourceId) selectedSourceId = nextSelectedId;
  });

  function clampInspectorWidth(width: number) {
    return Math.min(500, Math.max(380, Math.round(width)));
  }

  function startInspectorResize(event: PointerEvent) {
    const startX = event.clientX;
    const startWidth = inspectorWidth;
    const target = event.currentTarget as HTMLElement;
    target.setPointerCapture(event.pointerId);

    function move(moveEvent: PointerEvent) {
      inspectorWidth = clampInspectorWidth(startWidth - (moveEvent.clientX - startX));
    }

    function up(upEvent: PointerEvent) {
      target.releasePointerCapture(upEvent.pointerId);
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", up);
    }

    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", up);
  }

  function resizeWithKeyboard(event: KeyboardEvent) {
    if (event.key === "ArrowLeft") {
      inspectorWidth = clampInspectorWidth(inspectorWidth + 16);
      event.preventDefault();
    }
    if (event.key === "ArrowRight") {
      inspectorWidth = clampInspectorWidth(inspectorWidth - 16);
      event.preventDefault();
    }
  }

  async function handleSourcesChanged(sourceId?: number) {
    await onRefresh();
    if (sourceId) {
      selectedSourceId = `source:${sourceId}`;
    }
  }
</script>

<div
  data-ui-screen="library"
  class="library-screen"
  style={`--library-filter-width:${filterCollapsed ? 64 : 240}px; --library-inspector-width:${inspectorWidth}px;`}
>
  <LibraryFilterRail
    rows={filterRows}
    selectedFilterId={selectedFilterId}
    collapsed={filterCollapsed}
    onSelectedFilterIdChange={(id) => (selectedFilterId = id)}
    onCollapsedChange={(collapsed) => (filterCollapsed = collapsed)}
  />

  <LibraryWorkspace
    sources={visibleSources}
    bind:query
    selectedSource={selectedSource}
    selectedSourceId={selectedSourceId}
    loading={workflowState.loading}
    onSelectedSourceIdChange={(id) => (selectedSourceId = id)}
    onAdd={() => (addSourceDialogOpen = true)}
    onEdit={() => (status = "Edit source flow is not implemented in this prototype.")}
    onDelete={() => (status = "Delete source flow is not implemented in this prototype.")}
    onRefresh={onRefresh}
  />

  <!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
  <div
    class="inspector-resize-handle"
    role="separator"
    aria-label="Resize source inspector"
    aria-orientation="vertical"
    aria-valuemin="380"
    aria-valuemax="500"
    aria-valuenow={inspectorWidth}
    tabindex="0"
    onpointerdown={startInspectorResize}
    onkeydown={resizeWithKeyboard}
  ></div>

  <LibraryInspector {selectedSource} />

  <LibraryAddSourceDialog
    bind:open={addSourceDialogOpen}
    sources={workflowState.sources}
    onSourcesChanged={handleSourcesChanged}
    onStatus={(message) => (status = message)}
  />

  {#if status || workflowState.status}
    <div class="library-status" role="status">{status || workflowState.status}</div>
  {/if}
</div>

<style>
  .library-screen {
    position: relative;
    display: grid;
    grid-template-columns: var(--library-filter-width) minmax(0, 1fr) 6px var(--library-inspector-width);
    min-height: calc(100vh - 68px);
    background: var(--extractum-surface);
  }

  .inspector-resize-handle {
    min-width: 6px;
    cursor: col-resize;
    border-inline: 1px solid var(--extractum-border);
    background: var(--extractum-surface-subtle);
  }

  .inspector-resize-handle:focus-visible {
    outline: 2px solid var(--extractum-primary);
    outline-offset: -2px;
  }

  .library-status {
    position: absolute;
    right: 14px;
    bottom: 12px;
    max-width: min(520px, calc(100% - 28px));
    border: 1px solid var(--extractum-border);
    border-radius: var(--extractum-radius);
    padding: 8px 10px;
    background: var(--extractum-surface-raised);
    color: var(--extractum-muted);
    font-size: 12px;
    box-shadow: 0 8px 22px rgb(15 23 42 / 0.10);
  }
</style>
