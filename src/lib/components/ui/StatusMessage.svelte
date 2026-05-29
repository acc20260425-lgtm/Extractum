<script lang="ts">
  type StatusTone = "default" | "error" | "info" | "muted";
  type StatusSize = "sm" | "md";

  let {
    tone = "default",
    size = "md",
    surface = true,
    className = "",
    children,
  }: {
    tone?: StatusTone;
    size?: StatusSize;
    surface?: boolean;
    className?: string;
    children?: import("svelte").Snippet;
  } = $props();
</script>

<p class={`ui-status-message ${tone} ${size} ${surface ? "surface" : "plain"} ${className}`.trim()}>
  {@render children?.()}
</p>

<style>
  .ui-status-message {
    margin: 0;
  }

  .ui-status-message.surface {
    padding: 0.6rem 1rem;
    border-radius: 6px;
    background: var(--status-bg);
  }

  .ui-status-message.plain {
    padding: 0;
    background: transparent;
  }

  .ui-status-message.md {
    font-size: 0.9rem;
  }

  .ui-status-message.sm {
    font-size: 0.85rem;
  }

  .ui-status-message.default {
    color: inherit;
  }

  .ui-status-message.info.surface {
    background: color-mix(in srgb, var(--primary) 10%, var(--panel));
    color: color-mix(in srgb, var(--primary) 70%, var(--text));
  }

  .ui-status-message.info.plain {
    color: var(--primary);
  }

  .ui-status-message.error.surface {
    background: var(--status-error-bg);
    color: var(--status-error-text);
  }

  .ui-status-message.error.plain,
  .ui-status-message.muted {
    color: var(--muted);
  }

  .ui-status-message.error.plain {
    color: var(--status-error-text);
  }
</style>
