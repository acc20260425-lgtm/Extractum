<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    label,
    className = "",
    children,
  }: {
    label: string;
    className?: string;
    children?: Snippet;
  } = $props();
</script>

<div class={`ui-meta-cell ${className}`.trim()}>
  <span class="label">{label}</span>
  <strong class="value">
    {@render children?.()}
  </strong>
</div>

<style>
  .ui-meta-cell {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    padding: 0.75rem 0.85rem;
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 10px;
  }

  .label {
    color: var(--muted);
    font-size: 0.78rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .value {
    color: var(--text);
  }
</style>
