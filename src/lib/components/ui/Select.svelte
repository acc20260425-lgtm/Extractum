<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    value = "",
    disabled = false,
    name,
    className = "",
    onchange,
    children,
  }: {
    value?: string | number | null;
    disabled?: boolean;
    name?: string;
    className?: string;
    onchange?: (event: Event) => void;
    children?: Snippet;
  } = $props();
</script>

<select
  {value}
  {disabled}
  {name}
  class={`ui-select ${className}`.trim()}
  {onchange}
>
  {@render children?.()}
</select>

<style>
  .ui-select {
    width: 100%;
    min-width: 0;
    max-width: 100%;
    box-sizing: border-box;
    background: var(--panel-strong);
    border: 1px solid var(--border);
    color: var(--text);
    padding: 0.6rem 0.8rem;
    border-radius: 6px;
    font-size: 0.95rem;
  }

  .ui-select:focus {
    border-color: var(--primary);
    outline: none;
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--primary) 18%, transparent);
  }
</style>
