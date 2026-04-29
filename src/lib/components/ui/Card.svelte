<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    as = "section",
    embedded = false,
    className = "",
    children,
  }: {
    as?: string;
    embedded?: boolean;
    className?: string;
    children?: Snippet;
  } = $props();
</script>

<svelte:element
  this={as}
  class={`ui-card ${embedded ? "embedded" : ""} ${className}`.trim()}
>
  {@render children?.()}
</svelte:element>

<style>
  .ui-card {
    background: var(--panel);
    border: 1px solid var(--border);
    box-shadow: var(--shadow);
    border-radius: 12px;
    padding: 1.5rem;
  }

  .ui-card.embedded {
    background: transparent;
    border: 0;
    box-shadow: none;
    border-radius: 0;
    padding: 0;
  }
</style>
