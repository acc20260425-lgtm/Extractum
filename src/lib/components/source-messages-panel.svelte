<script lang="ts">
  interface ItemRecord {
    id: number;
    source_id: number;
    external_id: string;
    author: string | null;
    published_at: number;
    content: string;
    has_raw_data: boolean;
  }

  let {
    loadingItems,
    items,
    formatDate,
  }: {
    loadingItems: boolean;
    items: ItemRecord[];
    formatDate: (timestamp: number) => string;
  } = $props();
</script>

<div class="card">
  <div class="card-header">
    <h3>Messages</h3>
    {#if loadingItems}
      <span class="subtle">Loading...</span>
    {/if}
  </div>

  {#if !loadingItems && items.length === 0}
    <p class="empty">No synced text messages yet for this source.</p>
  {:else if items.length > 0}
    <ul class="message-list">
      {#each items as item}
        <li>
          <div class="message-meta">
            <span>{formatDate(item.published_at)}</span>
            {#if item.author}<span>{item.author}</span>{/if}
          </div>
          <p>{item.content}</p>
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
  .subtle { font-size: 0.75rem; color: var(--muted); }
  .empty { color: var(--muted); font-size: 0.9rem; margin: 0; }
</style>
