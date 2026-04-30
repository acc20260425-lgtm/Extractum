<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    title,
    subtitle = "",
    level = "h3",
    className = "",
    children,
  }: {
    title: string;
    subtitle?: string;
    level?: "h3" | "h4";
    className?: string;
    children?: Snippet;
  } = $props();
</script>

<div class={`ui-panel-header ${className}`.trim()}>
  <div class="copy">
    <svelte:element this={level}>{title}</svelte:element>
    {#if subtitle}
      <p class="subtitle">{subtitle}</p>
    {/if}
  </div>
  {#if children}
    <div class="actions">
      {@render children()}
    </div>
  {/if}
</div>

<style>
  .ui-panel-header {
    display: flex;
    justify-content: space-between;
    gap: 1rem;
    align-items: center;
    flex-wrap: wrap;
  }

  .copy {
    min-width: 0;
  }

  .copy h3,
  .copy h4 {
    margin: 0;
  }

  .subtitle {
    margin: 0.25rem 0 0 0;
    color: var(--muted);
    font-size: 0.9rem;
  }

  .actions {
    display: flex;
    gap: 0.6rem;
    flex-wrap: wrap;
    align-items: center;
  }
</style>
