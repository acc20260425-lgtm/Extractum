<script lang="ts">
  type MetaPillTone = "default" | "active";

  let {
    tone = "default",
    className = "",
    children,
  }: {
    tone?: MetaPillTone;
    className?: string;
    children?: import("svelte").Snippet;
  } = $props();
</script>

<span class={`ui-meta-pill ${tone} ${className}`.trim()}>
  {@render children?.()}
</span>

<style>
  .ui-meta-pill {
    display: inline-flex;
    align-items: center;
    max-width: 100%;
    padding: 0.25rem 0.55rem;
    border-radius: 999px;
    background: color-mix(in srgb, var(--panel-hover) 80%, transparent);
    color: var(--muted);
    font-size: 0.82rem;
    white-space: normal;
  }

  .ui-meta-pill.active {
    color: var(--text);
  }
</style>
