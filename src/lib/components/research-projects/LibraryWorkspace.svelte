<script lang="ts">
  import { Edit3, Plus, RefreshCw, Trash2 } from "@lucide/svelte";
  import {
    ExtractumButton,
    ExtractumDataGrid,
    ExtractumTextInput,
  } from "$lib/components/extractum-ui";
  import type { LibrarySourceView } from "$lib/ui/research-projects-model";
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
    sources: LibrarySourceView[];
    query: string;
    selectedSource: LibrarySourceView | null;
    selectedSourceId: string | null;
    loading?: boolean;
    onSelectedSourceIdChange: (id: string | null) => void;
    onAdd: () => void;
    onEdit: () => void;
    onDelete: () => void;
    onRefresh: () => void | Promise<void>;
  } = $props();

  const columns = [
    { id: "title", header: "Источник", flexgrow: 1, cell: LibrarySourceCell },
    { id: "provider", header: "Тип", width: 100 },
    { id: "status", header: "Статус", width: 118 },
    { id: "projectCount", header: "Проекты", width: 90 },
    { id: "localCopyLabel", header: "Локально", width: 116 },
    { id: "lastCollectedLabel", header: "Обновлен", width: 136 },
  ];
</script>

<section data-ui-region="library-workspace" class="library-workspace">
  <div class="toolbar">
    <ExtractumTextInput bind:value={query} placeholder="Search sources" aria-label="Search Library sources" />
    <ExtractumButton data-ui-action="library-add" onclick={onAdd}>
      <Plus size={14} aria-hidden="true" />
      Add
    </ExtractumButton>
    <ExtractumButton data-ui-action="library-edit" variant="outline" disabled={!selectedSource} onclick={onEdit}>
      <Edit3 size={14} aria-hidden="true" />
      Edit
    </ExtractumButton>
    <ExtractumButton data-ui-action="library-delete" variant="outline" disabled={!selectedSource} onclick={onDelete}>
      <Trash2 size={14} aria-hidden="true" />
      Delete
    </ExtractumButton>
    <ExtractumButton data-ui-action="library-refresh" variant="outline" disabled={loading} onclick={onRefresh}>
      <RefreshCw size={14} aria-hidden="true" />
      Refresh
    </ExtractumButton>
  </div>

  <div class="grid-host">
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
    display: flex;
    min-height: 46px;
    align-items: center;
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
