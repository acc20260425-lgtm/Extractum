<script lang="ts">
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import Select from "$lib/components/ui/Select.svelte";
  import type { SourceReaderItem } from "$lib/source-reader-model";

  const ALL_KINDS = "__all_snapshot_item_kinds__";

  let {
    items,
    loading,
    hasMore,
    selectedTraceRef = null,
    formatTimestamp,
    onLoadMore,
  }: {
    items: SourceReaderItem[];
    loading: boolean;
    hasMore: boolean;
    selectedTraceRef?: string | null;
    formatTimestamp: (value: number | null) => string;
    onLoadMore: () => void | Promise<void>;
  } = $props();

  let search = $state("");
  let selectedKind = $state(ALL_KINDS);
  let sortMode = $state<"newest" | "oldest">("newest");

  const kindChips = $derived.by(() => {
    const counts = new Map<string, number>();
    for (const item of items) counts.set(item.kind, (counts.get(item.kind) ?? 0) + 1);
    return Array.from(counts, ([kind, count]) => ({ kind, label: itemKindLabel(kind), count }));
  });
  const visibleItems = $derived.by(() => {
    const query = search.trim().toLowerCase();
    const filtered = items.filter((item) => {
      if (selectedKind !== ALL_KINDS && item.kind !== selectedKind) return false;
      if (!query) return true;
      return [item.content, item.author, item.sourceTitle]
        .some((value) => value?.toLowerCase().includes(query));
    });
    const direction = sortMode === "newest" ? -1 : 1;
    return [...filtered].sort((left, right) => {
      const leftTime = left.publishedAt ?? 0;
      const rightTime = right.publishedAt ?? 0;
      return (leftTime - rightTime) * direction || left.id.localeCompare(right.id);
    });
  });

  function inputValue(event: Event) {
    const target = event.currentTarget;
    return target instanceof HTMLInputElement ? target.value : "";
  }

  function changeSort(event: Event) {
    sortMode = (event.currentTarget as HTMLSelectElement).value as "newest" | "oldest";
  }

  function itemKindLabel(kind: string) {
    const [first = "", ...rest] = kind.split("_");
    return [first === "youtube" ? "YouTube" : capitalize(first), ...rest].join(" ");
  }

  function capitalize(value: string) {
    if (!value) return value;
    return value.charAt(0).toUpperCase() + value.slice(1);
  }

  function itemSelected(item: SourceReaderItem) {
    return item.selected || (selectedTraceRef !== null && item.ref === selectedTraceRef);
  }
</script>

<section class="snapshot-items-view" aria-label="Run snapshot items">
  <div class="items-toolbar">
    <label class="search-field">
      <span>Search snapshot items</span>
      <Input
        type="search"
        value={search}
        placeholder="Search snapshot items"
        ariaLabel="Search snapshot items"
        oninput={(event) => (search = inputValue(event))}
      />
    </label>

    <label class="sort-field">
      <span>Sort snapshot items</span>
      <Select value={sortMode} onchange={changeSort}>
        <option value="newest">Newest first</option>
        <option value="oldest">Oldest first</option>
      </Select>
    </label>
  </div>

  <div class="kind-chips" aria-label="Snapshot item kinds">
    <Button
      type="button"
      size="sm"
      variant={selectedKind === ALL_KINDS ? "secondary" : "ghost"}
      selected={selectedKind === ALL_KINDS}
      ariaPressed={selectedKind === ALL_KINDS}
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
        ariaPressed={selectedKind === chip.kind}
        onclick={() => (selectedKind = chip.kind)}
      >
        {chip.label} ({chip.count})
      </Button>
    {/each}
  </div>

  <p class="items-help">
    Snapshot items are limited to frozen rows loaded for this run. Load older snapshot messages to fetch more captured rows.
  </p>

  {#if !loading && items.length === 0}
    <EmptyState description="No frozen source rows are loaded for this run snapshot." />
  {:else if !loading && visibleItems.length === 0}
    <EmptyState description="No snapshot items match the current filters." />
  {:else}
    <ul class="item-list">
      {#each visibleItems as item (item.id)}
        <li>
          <article class:selected={itemSelected(item)} data-trace-ref={item.ref}>
            <div class="item-heading">
              <strong>{itemKindLabel(item.kind)}</strong>
              <span>{formatTimestamp(item.publishedAt)}</span>
            </div>
            <div class="item-meta">
              <Badge variant="neutral">{item.sourceTitle}</Badge>
              {#if item.author}<Badge variant="neutral">{item.author}</Badge>{/if}
              {#if item.ref}<Badge variant="info">{item.ref}</Badge>{/if}
              <Badge variant="neutral">{item.externalId}</Badge>
            </div>
            <p>{item.content || "No text content captured for this snapshot row."}</p>
          </article>
        </li>
      {/each}
    </ul>
  {/if}

  {#if hasMore}
    <div class="reader-footer">
      <Button type="button" variant="secondary" disabled={loading} onclick={onLoadMore}>
        {loading ? "Loading..." : "Load older snapshot messages"}
      </Button>
    </div>
  {/if}
</section>

<style>
  .snapshot-items-view {
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

  .items-help,
  .item-heading span,
  p {
    color: var(--muted);
  }

  .items-help {
    margin: 0;
    font-size: 0.82rem;
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

  article.selected {
    border-color: var(--accent);
    box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent) 24%, transparent);
  }

  .item-heading {
    justify-content: space-between;
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
