<script lang="ts">
  import { Edit3, Plus, RefreshCw, Trash2 } from "@lucide/svelte";
  import {
    ExtractumButton,
    ExtractumDataGrid,
    ExtractumTextInput,
  } from "$lib/components/extractum-ui";
  import type { ExtractumDataGridColumn } from "$lib/components/extractum-ui";
  import type { LibraryCatalogSourceView } from "$lib/ui/library-catalog-model";
  import LibrarySourceCell from "./LibrarySourceCell.svelte";

  let {
    sources,
    query = $bindable(""),
    selectedSource,
    selectedSourceId,
    loading = false,
    onSelectedSourceIdChange,
    onAdd,
    onEdit,
    onDelete,
    onRefresh,
  }: {
    sources: LibraryCatalogSourceView[];
    query: string;
    selectedSource: LibraryCatalogSourceView | null;
    selectedSourceId: string | null;
    loading?: boolean;
    onSelectedSourceIdChange: (id: string | null) => void;
    onAdd: () => void;
    onEdit: () => void;
    onDelete: () => void;
    onRefresh: () => void | Promise<void>;
  } = $props();

  const columns: ExtractumDataGridColumn[] = [
    { id: "title", header: "Source", width: 320, cell: LibrarySourceCell },
    { id: "typeLabel", header: "Type", width: 150 },
    { id: "status", header: "Status", width: 110 },
    { id: "projectCount", header: "Projects", width: 92 },
    { id: "itemCountLabel", header: "Items", width: 100 },
    { id: "createdAt", header: "Added", width: 136, dateTimeFormat: "datetime" },
    { id: "lastSyncedAt", header: "Last synced", width: 136, dateTimeFormat: "datetime" },
  ];
</script>

<section data-ui-region="library-workspace" class="library-workspace">
  <div class="toolbar extractum-toolbar-row">
    <ExtractumTextInput bind:value={query} placeholder="Search sources" aria-label="Search Library sources" />
    <ExtractumButton
      data-ui-action="library-add"
      aria-label="Add library source"
      title="Add library source"
      onclick={onAdd}
    >
      <Plus size={14} aria-hidden="true" />
      Add
    </ExtractumButton>
    <ExtractumButton
      data-ui-action="library-edit"
      variant="outline"
      aria-label={`Edit selected library source${selectedSource ? `: ${selectedSource.title}` : ""}`}
      title={`Edit selected library source${selectedSource ? `: ${selectedSource.title}` : ""}`}
      disabled={!selectedSource}
      onclick={onEdit}
    >
      <Edit3 size={14} aria-hidden="true" />
      Edit
    </ExtractumButton>
    <ExtractumButton
      data-ui-action="library-delete"
      variant="outline"
      aria-label={`Delete selected library source${selectedSource ? `: ${selectedSource.title}` : ""}`}
      title={`Delete selected library source${selectedSource ? `: ${selectedSource.title}` : ""}`}
      disabled={!selectedSource}
      onclick={onDelete}
    >
      <Trash2 size={14} aria-hidden="true" />
      Delete
    </ExtractumButton>
    <ExtractumButton
      data-ui-action="library-refresh"
      variant="outline"
      aria-label="Refresh library source list"
      title="Refresh library source list"
      disabled={loading}
      onclick={onRefresh}
    >
      <RefreshCw size={14} aria-hidden="true" />
      Refresh
    </ExtractumButton>
  </div>

  <div class="grid-host extractum-grid-frame">
    <ExtractumDataGrid
      rows={sources}
      {columns}
      selectedRowIds={selectedSourceId ? [selectedSourceId] : []}
      overlay="No sources match this filter"
      onSelectedRowIdsChange={(ids) => onSelectedSourceIdChange(ids.at(-1) ?? null)}
    />
  </div>
</section>

<style>
  .library-workspace {
    display: flex;
    min-width: 0;
    min-height: 0;
    flex-direction: column;
    background: var(--extractum-surface);
  }

  .toolbar {
    justify-content: flex-start;
    min-height: 46px;
    flex-wrap: wrap;
    gap: 8px;
    padding: 8px;
    border-bottom: 1px solid var(--extractum-border);
  }

  .toolbar :global(.extractum-input) {
    flex: 1 1 auto;
    min-width: 160px;
  }

  .grid-host {
    min-width: 0;
    min-height: 0;
    flex: 1;
  }
</style>
