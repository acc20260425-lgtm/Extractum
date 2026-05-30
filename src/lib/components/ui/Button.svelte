<script lang="ts">
  import type { Snippet } from "svelte";

  type ButtonVariant =
    | "primary"
    | "secondary"
    | "danger"
    | "danger-soft"
    | "ghost";

  type ButtonSize = "sm" | "md";

  let {
    type = "button",
    variant = "primary",
    size = "md",
    disabled = false,
    selected = false,
    iconOnly = false,
    id,
    role,
    title,
    ariaLabel,
    ariaPressed,
    ariaSelected,
    ariaControls,
    ariaExpanded,
    ariaDescribedby,
    smokeId,
    tabIndex,
    className = "",
    onclick,
    children,
  }: {
    type?: "button" | "submit" | "reset";
    variant?: ButtonVariant;
    size?: ButtonSize;
    disabled?: boolean;
    selected?: boolean;
    iconOnly?: boolean;
    id?: string;
    role?: string;
    title?: string;
    ariaLabel?: string;
    ariaPressed?: boolean;
    ariaSelected?: boolean;
    ariaControls?: string;
    ariaExpanded?: boolean;
    ariaDescribedby?: string;
    smokeId?: string;
    tabIndex?: number;
    className?: string;
    onclick?: (event: MouseEvent) => unknown | Promise<unknown>;
    children?: Snippet;
  } = $props();
</script>

<button
  {id}
  {type}
  {role}
  {disabled}
  {title}
  aria-label={ariaLabel}
  aria-pressed={ariaPressed}
  aria-selected={ariaSelected}
  aria-controls={ariaControls}
  aria-expanded={ariaExpanded}
  aria-describedby={ariaDescribedby}
  data-smoke-id={smokeId}
  tabindex={tabIndex}
  class={`ui-button ${variant} ${size} ${selected ? "selected" : ""} ${iconOnly ? "icon-only" : ""} ${className}`.trim()}
  onclick={onclick}
>
  {@render children?.()}
</button>

<style>
  .ui-button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 0.35rem;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-weight: 600;
    transition: background 0.2s, border-color 0.2s, color 0.2s;
    white-space: nowrap;
  }

  .ui-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .ui-button:focus-visible:not(:disabled) {
    outline: 2px solid var(--primary);
    outline-offset: 2px;
  }

  .ui-button.md {
    padding: 0.6rem 1rem;
    font-size: 0.95rem;
  }

  .ui-button.sm {
    padding: 0.3rem 0.7rem;
    font-size: 0.8rem;
  }

  .ui-button.icon-only {
    width: 2.25rem;
    height: 2.25rem;
    padding: 0;
  }

  .ui-button.icon-only.sm {
    width: 1.9rem;
    height: 1.9rem;
  }

  .ui-button.primary {
    background: var(--primary);
    color: white;
  }

  .ui-button.primary:hover:enabled {
    background: var(--primary-hover);
  }

  .ui-button.secondary {
    background: transparent;
    border: 1px solid var(--border);
    color: var(--text);
  }

  .ui-button.secondary:hover:enabled {
    background: var(--panel-hover);
  }

  .ui-button.secondary.selected {
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--primary) 14%, transparent);
    border-color: var(--primary);
  }

  .ui-button.danger {
    background: var(--danger);
    color: white;
  }

  .ui-button.danger:hover:enabled {
    background: var(--danger-hover);
  }

  .ui-button.danger-soft {
    background: color-mix(in srgb, var(--danger) 14%, var(--panel));
    color: var(--danger);
    border: 1px solid color-mix(in srgb, var(--danger) 28%, transparent);
  }

  .ui-button.danger-soft:hover:enabled {
    background: color-mix(in srgb, var(--danger) 22%, var(--panel));
  }

  .ui-button.ghost {
    background: transparent;
    border: 1px solid transparent;
    color: var(--muted);
  }

  .ui-button.ghost:hover:enabled {
    color: var(--text);
    background: color-mix(in srgb, var(--panel-hover) 72%, transparent);
  }
</style>
