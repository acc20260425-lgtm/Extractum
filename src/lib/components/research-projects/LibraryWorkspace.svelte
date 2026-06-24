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

<section data-ui-region="library-workspace" class="library-workspace extractum-grid-frame">
  <div
    class="toolbar extractum-toolbar-row flex-wrap justify-start gap-2 border-b border-[var(--extractum-border)] px-2 py-2 min-h-[46px]"
    role="group"
    aria-label="Library source actions"
  >
    <ExtractumTextInput
      bind:value={query}
      class="flex-1 min-w-[160px]"
      placeholder="Search sources"
      aria-label="Search library sources by title, type, or URL"
      aria-controls="library-sources-grid"
    />
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

  <div
    id="library-sources-grid"
    class="grid-host extractum-grid-frame min-h-0 min-w-0 flex-1"
  >
    <ExtractumDataGrid
      rows={sources}
      {columns}
      selectedRowIds={selectedSourceId ? [selectedSourceId] : []}
      ariaLabel="Library source list"
      overlay="No sources match this filter"
      onSelectedRowIdsChange={(ids) => onSelectedSourceIdChange(ids.at(-1) ?? null)}
    />
  </div>
</section>
