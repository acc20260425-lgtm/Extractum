<script lang="ts">
  let { row }: { row: Record<string, unknown> } = $props();

  let provider = $derived(String(row.provider ?? ""));
  let title = $derived(String(row.title ?? ""));
  let handle = $derived(typeof row.handle === "string" && row.handle.length > 0 ? row.handle : null);
</script>

<div class="source-title-cell">
  <span
    class="source-title-cell__dot"
    data-testid="source-provider-dot"
    data-provider={provider}
  ></span>
  <span class="source-title-cell__text">
    <span class="source-title-cell__name">{title}</span>
    {#if handle}
      <span class="source-title-cell__handle" data-testid="source-handle">{handle}</span>
    {/if}
  </span>
</div>

<style>
  .source-title-cell {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    /* svar's .wx-cell is display:block with a fixed height; stretch to it so
       align-items:center actually centers the two-line text block. */
    height: 100%;
  }

  .source-title-cell__dot {
    flex-shrink: 0;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--extractum-muted);
  }

  .source-title-cell__dot[data-provider="telegram"] {
    background: var(--extractum-provider-telegram);
  }

  .source-title-cell__dot[data-provider="youtube"] {
    background: var(--extractum-provider-youtube);
  }

  .source-title-cell__text {
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 1px;
    min-width: 0;
  }

  .source-title-cell__name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font: 500 12.5px/1.2 var(--extractum-font);
    color: var(--extractum-text);
  }

  .source-title-cell__handle {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 100%;
    font: 400 11px/1.2 var(--extractum-font);
    color: var(--extractum-muted-2);
  }
</style>
