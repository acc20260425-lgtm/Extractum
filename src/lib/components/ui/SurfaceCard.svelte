<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    title = "",
    meta = "",
    compact = false,
    className = "",
    actions,
    children,
  }: {
    title?: string;
    meta?: string;
    compact?: boolean;
    className?: string;
    actions?: Snippet;
    children?: Snippet;
  } = $props();
</script>

<section class={`ui-surface-card ${compact ? "compact" : ""} ${className}`.trim()}>
  {#if title || meta || actions}
    <div class="surface-header">
      <div class="surface-copy">
        {#if title}
          <span class="surface-title">{title}</span>
        {/if}
        {#if meta}
          <span class="surface-meta">{meta}</span>
        {/if}
      </div>
      {#if actions}
        <div class="surface-actions">
          {@render actions()}
        </div>
      {/if}
    </div>
  {/if}

  {@render children?.()}
</section>

<style>
  .ui-surface-card {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    min-width: 0;
    padding: 1rem;
    border: 1px solid var(--border);
    border-radius: 10px;
    background: var(--panel-strong);
  }

  .ui-surface-card.compact {
    min-height: 10rem;
  }

  .surface-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 1rem;
    flex-wrap: wrap;
  }

  .surface-copy {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    min-width: 0;
  }

  .surface-title {
    font-size: 0.95rem;
    font-weight: 600;
    color: var(--text);
  }

  .surface-meta {
    color: var(--muted);
    font-size: 0.9rem;
  }

  .surface-actions {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    flex-wrap: wrap;
  }
</style>
