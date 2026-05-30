<script lang="ts">
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import Select from "$lib/components/ui/Select.svelte";
  import {
    filterLoadedSourceItems,
    sortLoadedSourceItems,
    sourceItemKindChips,
    type LoadedSourceItemSort,
  } from "$lib/source-browser-model";
  import type { SourceItem } from "$lib/types/sources";

  const ALL_KINDS = "__all_source_item_kinds__";
  const KNOWN_ITEM_KINDS = new Set([
    "telegram_message",
    "youtube_transcript",
    "youtube_comment",
  ]);

  let {
    items,
    loading,
    hasMore,
    emptyDescription = "No loaded items are available for this source window.",
    sourceLabelForItem = null,
    formatTimestamp,
    onLoadMore,
  }: {
    items: SourceItem[];
    loading: boolean;
    hasMore: boolean;
    emptyDescription?: string;
    sourceLabelForItem?: ((item: SourceItem) => string | null) | null;
    formatTimestamp: (value: number | null) => string;
    onLoadMore: () => void | Promise<void>;
  } = $props();

  let search = $state("");
  let selectedKind = $state(ALL_KINDS);
  let sortMode = $state<LoadedSourceItemSort>("newest");

  const kindChips = $derived(sourceItemKindChips(items));
  const filteredItems = $derived.by(() =>
    filterLoadedSourceItems(items, {
      kind: selectedKind === ALL_KINDS ? null : selectedKind,
      search,
    }),
  );
  const visibleItems = $derived(sortLoadedSourceItems(filteredItems, sortMode));

  function inputValue(event: Event) {
    const target = event.currentTarget;
    return target instanceof HTMLInputElement ? target.value : "";
  }

  function changeSort(event: Event) {
    sortMode = (event.currentTarget as HTMLSelectElement).value as LoadedSourceItemSort;
  }

  function itemKindTitle(item: SourceItem) {
    if (!KNOWN_ITEM_KINDS.has(item.itemKind)) return "Unknown item kind";
    return item.itemKind.replaceAll("_", " ");
  }

  function itemSourceLabel(item: SourceItem) {
    return sourceLabelForItem?.(item) ?? `Source #${item.sourceId}`;
  }
</script>

<section class="universal-items-view" aria-label="Universal source items">
  <div class="items-toolbar">
    <label class="search-field">
      <span>Search loaded items</span>
      <Input
        type="search"
        value={search}
        placeholder="Search loaded items"
        ariaLabel="Search loaded items"
        oninput={(event) => (search = inputValue(event))}
      />
    </label>

    <label class="sort-field">
      <span>Sort loaded items</span>
      <Select value={sortMode} onchange={changeSort}>
        <option value="newest">Newest first</option>
        <option value="oldest">Oldest first</option>
      </Select>
    </label>
  </div>

  <div class="kind-chips" aria-label="Loaded item kinds">
    <Button
      type="button"
      size="sm"
      variant={selectedKind === ALL_KINDS ? "secondary" : "ghost"}
      selected={selectedKind === ALL_KINDS}
      onclick={() => (selectedKind = ALL_KINDS)}
    >
      All
    </Button>
    {#each kindChips as chip (chip.kind)}
      <Button
        type="button"
        size="sm"
        variant={selectedKind === chip.kind ? "secondary" : "ghost"}
        selected={selectedKind === chip.kind}
        onclick={() => (selectedKind = chip.kind)}
      >
        {chip.label} ({chip.count})
      </Button>
    {/each}
  </div>

  {#if !loading && items.length === 0}
    <EmptyState description={emptyDescription} />
  {:else if !loading && visibleItems.length === 0}
    <EmptyState description="No loaded items match the current filters." />
  {:else}
    <ul class="item-list">
      {#each visibleItems as item (item.id)}
        <li>
          <article class:unknown-kind={!KNOWN_ITEM_KINDS.has(item.itemKind)}>
            <div class="item-heading">
              <strong>{itemKindTitle(item)}</strong>
              <span>{formatTimestamp(item.publishedAt)}</span>
            </div>
            <div class="item-meta">
              {#if item.author}<Badge variant="neutral">{item.author}</Badge>{/if}
              <Badge variant="neutral">{itemSourceLabel(item)}</Badge>
              <Badge variant="neutral">{item.externalId}</Badge>
              {#if item.hasMedia}<Badge variant="info">{item.mediaKind ?? "media"}</Badge>{/if}
            </div>
            <p>{item.content ?? "No text content loaded."}</p>
          </article>
        </li>
      {/each}
    </ul>
  {/if}

  {#if hasMore}
    <div class="reader-footer">
      <Button type="button" variant="secondary" disabled={loading} onclick={onLoadMore}>
        {loading ? "Loading..." : "Load more items"}
      </Button>
    </div>
  {/if}
</section>

<style>
  .universal-items-view {
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
    min-width: 0;
  }

  .items-toolbar,
  .kind-chips,
  .item-meta,
  .item-heading {
    display: flex;
    gap: 0.55rem;
    align-items: center;
    flex-wrap: wrap;
  }

  .items-toolbar {
    align-items: flex-end;
  }

  .search-field,
  .sort-field {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    color: var(--muted);
    font-size: 0.78rem;
  }

  .search-field {
    flex: 1 1 16rem;
  }

  .sort-field {
    flex: 0 1 12rem;
  }

  .item-list {
    display: flex;
    flex-direction: column;
    gap: 0.55rem;
    list-style: none;
    margin: 0;
    padding: 0;
  }

  article {
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
    padding: 0.7rem 0.8rem;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--panel);
  }

  article.unknown-kind {
    border-style: dashed;
  }

  .item-heading {
    justify-content: space-between;
  }

  .item-heading span,
  p {
    color: var(--muted);
  }

  p {
    margin: 0;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
    line-height: 1.45;
  }

  .reader-footer {
    display: flex;
    justify-content: center;
  }
</style>
