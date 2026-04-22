<script lang="ts">
  interface ItemRecord {
    id: number;
    source_id: number;
    external_id: string;
    author: string | null;
    published_at: number;
    content: string | null;
    content_kind: string;
    has_media: boolean;
    media_kind: string | null;
    media_summary: string | null;
    media_file_name: string | null;
    media_mime_type: string | null;
    has_raw_data: boolean;
  }

  let {
    loadingItems,
    items,
    formatDate,
    embedded = false,
  }: {
    loadingItems: boolean;
    items: ItemRecord[];
    formatDate: (timestamp: number) => string;
    embedded?: boolean;
  } = $props();

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

<div class:embedded class="card">
  <div class="card-header">
    <h3>Messages</h3>
    {#if loadingItems}
      <span class="subtle">Loading...</span>
    {/if}
  </div>

  {#if !loadingItems && items.length === 0}
    <p class="empty">No synced messages yet for this source.</p>
  {:else if items.length > 0}
    <ul class="message-list">
      {#each items as item (item.id)}
        <li>
          <div class="message-meta">
            <span>{formatDate(item.published_at)}</span>
            {#if item.author}<span>{item.author}</span>{/if}
            {#if item.has_media}
              <span class="media-badge">{formatMediaKind(item.media_kind)}</span>
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
                <p class="media-summary">{details.join(" · ")}</p>
              {/if}
            </div>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .card {
    background: var(--panel);
    border: 1px solid var(--border);
    box-shadow: var(--shadow);
    border-radius: 12px;
    padding: 1.5rem;
    margin-bottom: 1.5rem;
  }
  .card.embedded {
    background: transparent;
    border: 0;
    box-shadow: none;
    border-radius: 0;
    padding: 0;
    margin-bottom: 0;
  }
  .card-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem; }
  .card-header h3 { margin: 0; }
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
  .media-badge {
    background: color-mix(in srgb, var(--accent) 14%, var(--panel));
    border: 1px solid color-mix(in srgb, var(--accent) 30%, transparent);
    border-radius: 999px;
    color: var(--accent);
    font-size: 0.72rem;
    font-weight: 600;
    padding: 0.08rem 0.5rem;
    text-transform: capitalize;
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
  .empty { color: var(--muted); font-size: 0.9rem; margin: 0; }
</style>
