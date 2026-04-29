<script lang="ts">
  import type { Snippet } from "svelte";

  type BadgeVariant = "default" | "warning" | "member" | "info" | "success" | "danger" | "neutral";

  let {
    variant = "default",
    className = "",
    title,
    children,
  }: {
    variant?: BadgeVariant;
    className?: string;
    title?: string;
    children?: Snippet;
  } = $props();
</script>

<span
  {title}
  class={`ui-badge ${variant} ${className}`.trim()}
>
  {@render children?.()}
</span>

<style>
  .ui-badge {
    display: inline-flex;
    align-items: center;
    max-width: 100%;
    font-size: 0.7rem;
    line-height: 1.2;
    padding: 0.15rem 0.5rem;
    border-radius: 4px;
    background: var(--panel-hover);
    color: var(--muted);
    white-space: normal;
  }

  .ui-badge.member,
  .ui-badge.success {
    background: color-mix(in srgb, #22c55e 18%, var(--panel));
    color: #15803d;
  }

  .ui-badge.warning {
    background: color-mix(in srgb, #f59e0b 22%, var(--panel));
    color: #b45309;
  }

  .ui-badge.info {
    background: color-mix(in srgb, var(--primary) 14%, var(--panel));
    color: var(--primary);
  }

  .ui-badge.neutral {
    background: var(--panel-hover);
    color: var(--muted);
  }

  .ui-badge.danger {
    background: color-mix(in srgb, var(--danger) 12%, var(--panel));
    color: var(--danger);
  }
</style>
