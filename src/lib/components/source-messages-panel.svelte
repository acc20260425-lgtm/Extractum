<script lang="ts">
  import Badge from "$lib/components/ui/Badge.svelte";
  import Card from "$lib/components/ui/Card.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import PanelHeader from "$lib/components/ui/PanelHeader.svelte";
  import type { ItemRecord } from "$lib/types/sources";

  let {
    loadingItems,
    items,
    formatDate,
    embedded = false,
    previewLimit = 200,
  }: {
    loadingItems: boolean;
    items: ItemRecord[];
    formatDate: (timestamp: number) => string;
    embedded?: boolean;
    previewLimit?: number;
  } = $props();

  const visibleItems = $derived.by(() => {
    if (previewLimit <= 0) {
      return [];
    }
    return items.slice(0, previewLimit);
  });

  function formatMediaKind(kind: string | null) {
    if (!kind) return "media";
    return kind.replaceAll("_", " ");
  }

  function mediaDetails(item: ItemRecord) {
    return [item.media_summary, item.media_file_name, item.media_mime_type].filter(
      (value, index, values): value is string => !!value && values.indexOf(value) === index
    );
  }
</script>

<Card as="div" {embedded}>
  <PanelHeader title="Messages">
    {#if loadingItems}
      <span class="subtle">Loading...</span>
    {:else if items.length > 0}
      <span class="subtle">
        Showing {visibleItems.length} of {items.length} latest
        message{items.length === 1 ? "" : "s"}
      </span>
    {/if}
  </PanelHeader>

  {#if !loadingItems && items.length === 0}
    <EmptyState description="No synced messages yet for this source." />
  {:else if visibleItems.length > 0}
    <ul class="message-list">
      {#each visibleItems as item (item.id)}
        <li>
          <div class="message-meta">
            <span>{formatDate(item.published_at)}</span>
            {#if item.author}<span>{item.author}</span>{/if}
            {#if item.forum_topic_title}
              <Badge variant="neutral">{item.forum_topic_title}</Badge>
            {/if}
            {#if item.has_media}
              <Badge variant="info">{formatMediaKind(item.media_kind)}</Badge>
            {/if}
          </div>
          {#if item.content}
            <p>{item.content}</p>
          {/if}
          {#if item.has_media}
            {@const details = mediaDetails(item)}
            <div class="media-block">
              {#if !item.content}
                <p class="media-placeholder">Media-only post</p>
              {/if}
              {#if details.length > 0}
                <p class="media-summary">{details.join(" | ")}</p>
              {/if}
            </div>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</Card>

<style>
  .message-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  .message-list li {
    background: var(--panel-strong);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 0.9rem 1rem;
  }
  .message-list p {
    margin: 0;
    white-space: pre-wrap;
    line-height: 1.45;
  }
  .message-meta {
    display: flex;
    gap: 0.75rem;
    flex-wrap: wrap;
    color: var(--muted);
    font-size: 0.78rem;
    margin-bottom: 0.5rem;
  }
  .media-block {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }
  .media-placeholder,
  .media-summary {
    margin: 0;
    color: var(--muted);
    font-size: 0.84rem;
  }
  .media-placeholder {
    font-weight: 600;
  }
  .subtle { font-size: 0.75rem; color: var(--muted); }
</style>
