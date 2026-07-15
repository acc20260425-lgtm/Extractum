<script lang="ts">
  import BookOpen from "@lucide/svelte/icons/book-open";
  import Edit3 from "@lucide/svelte/icons/pen-line";
  import Plus from "@lucide/svelte/icons/plus";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import {
    ExtractumButton,
    ExtractumDataGrid,
    ExtractumTextInput,
  } from "$lib/components/extractum-ui";
  import { libraryCatalogGridColumns } from "$lib/ui/library-catalog-grid";
  import type { LibraryCatalogSourceView } from "$lib/ui/library-catalog-model";
  import LibrarySourceCell from "./LibrarySourceCell.svelte";

  let {
    sources,
    totalSourceCount = 0,
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
    totalSourceCount?: number;
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

  const isEmpty = $derived(totalSourceCount === 0 && !loading);

  const columns = libraryCatalogGridColumns(LibrarySourceCell);
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
      variant="destructive"
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
    {#if isEmpty}
      <div class="library-empty-state">
        <div class="library-empty-icon">
          <BookOpen size={48} aria-hidden="true" />
        </div>
        <h2>No sources yet</h2>
        <p>Add YouTube channels, Telegram channels, or other sources to start collecting content.</p>
        <ExtractumButton onclick={onAdd} aria-label="Add your first library source">
          <Plus size={14} aria-hidden="true" />
          Add your first source
        </ExtractumButton>
      </div>
    {:else}
      <ExtractumDataGrid
        rows={sources}
        {columns}
        selectedRowIds={selectedSourceId ? [selectedSourceId] : []}
        ariaLabel="Library source list"
        overlay="No sources match this filter"
        onSelectedRowIdsChange={(ids) => onSelectedSourceIdChange(ids.at(-1) ?? null)}
      />
    {/if}
  </div>
</section>

<style>
  .library-empty-state {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    justify-content: center;
    gap: 0.75rem;
    height: 100%;
    padding: 2rem;
  }

  .library-empty-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 72px;
    height: 72px;
    border-radius: 16px;
    background: var(--extractum-surface-subtle, #f0f4f8);
    color: var(--extractum-muted);
    margin-bottom: 0.25rem;
  }

  .library-empty-state h2 {
    margin: 0;
    font-size: 1.1rem;
    font-weight: 600;
  }

  .library-empty-state p {
    margin: 0;
    max-width: 52ch;
    color: var(--extractum-muted);
    line-height: 1.5;
    font-size: 0.9rem;
  }
</style>
