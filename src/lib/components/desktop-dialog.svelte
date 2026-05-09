<script lang="ts">
  import { X } from "@lucide/svelte";
  import { tick } from "svelte";
  import { cubicOut } from "svelte/easing";
  import { fade, scale } from "svelte/transition";
  import Button from "$lib/components/ui/Button.svelte";

  let {
    open,
    title,
    description = "",
    labelledBy = "desktop-dialog-title",
    width = "46rem",
    onClose,
    children,
  }: {
    open: boolean;
    title: string;
    description?: string;
    labelledBy?: string;
    width?: string;
    onClose: () => void;
    children?: import("svelte").Snippet;
  } = $props();

  let dialogElement = $state<HTMLDivElement | null>(null);
  let previousFocusedElement = $state<HTMLElement | null>(null);

  async function focusDialog() {
    if (!open) return;

    previousFocusedElement = document.activeElement instanceof HTMLElement
      ? document.activeElement
      : null;

    await tick();

    const focusable = dialogElement?.querySelector<HTMLElement>(
      'button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])'
    );

    focusable?.focus();
  }

  function restoreFocus() {
    previousFocusedElement?.focus();
    previousFocusedElement = null;
  }

  function trapFocus(event: KeyboardEvent) {
    if (event.key !== "Tab" || !dialogElement) return;

    const focusable = Array.from(
      dialogElement.querySelectorAll<HTMLElement>(
        'button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])'
      )
    ).filter((element) => !element.hasAttribute("hidden"));

    if (focusable.length === 0) {
      event.preventDefault();
      return;
    }

    const first = focusable[0];
    const last = focusable[focusable.length - 1];
    const active = document.activeElement;

    if (event.shiftKey && active === first) {
      event.preventDefault();
      last.focus();
      return;
    }

    if (!event.shiftKey && active === last) {
      event.preventDefault();
      first.focus();
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      onClose();
      return;
    }

    trapFocus(event);
  }

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      onClose();
    }
  }

  $effect(() => {
    if (open) {
      void focusDialog();
      return;
    }

    restoreFocus();
  });
</script>

{#if open}
  <div
    class="dialog-backdrop"
    role="presentation"
    onclick={handleBackdropClick}
    transition:fade={{ duration: 120 }}
  >
    <div
      bind:this={dialogElement}
      class="dialog-card"
      role="dialog"
      aria-modal="true"
      aria-labelledby={labelledBy}
      aria-describedby={description ? `${labelledBy}-description` : undefined}
      tabindex="-1"
      style={`--dialog-width: ${width};`}
      onkeydown={handleKeydown}
      transition:scale={{ duration: 150, start: 0.985, easing: cubicOut }}
    >
      <header class="dialog-header">
        <div class="dialog-copy">
          <h4 id={labelledBy}>{title}</h4>
          {#if description}
            <p id={`${labelledBy}-description`}>{description}</p>
          {/if}
        </div>
        <Button
          variant="ghost"
          type="button"
          onclick={onClose}
          ariaLabel="Close dialog"
          title="Close dialog"
          iconOnly={true}
        >
          <X size={16} aria-hidden="true" />
        </Button>
      </header>

      <div class="dialog-content">
        {@render children?.()}
      </div>
    </div>
  </div>
{/if}

<style>
  .dialog-backdrop {
    position: fixed;
    inset: 0;
    z-index: 60;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1.5rem;
    background: color-mix(in srgb, #0f172a 28%, transparent);
    backdrop-filter: blur(6px);
  }

  .dialog-card {
    width: min(var(--dialog-width), calc(100vw - 2rem));
    max-height: min(88vh, 56rem);
    display: flex;
    flex-direction: column;
    border-radius: 16px;
    background: color-mix(in srgb, var(--panel) 97%, transparent);
    border: 1px solid color-mix(in srgb, var(--border) 94%, transparent);
    box-shadow:
      0 18px 42px rgba(15, 23, 42, 0.18),
      0 3px 10px rgba(15, 23, 42, 0.08);
    overflow: hidden;
  }

  .dialog-header,
  .dialog-content {
    padding-left: 1rem;
    padding-right: 1rem;
  }

  .dialog-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 1rem;
    padding-top: 0.95rem;
    padding-bottom: 0.8rem;
    border-bottom: 1px solid var(--border);
  }

  .dialog-copy {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .dialog-copy h4 {
    margin: 0;
    font-size: 0.98rem;
    font-weight: 650;
  }

  .dialog-copy p {
    margin: 0;
    color: var(--muted);
    font-size: 0.9rem;
    line-height: 1.45;
  }

  .dialog-content {
    padding-top: 1rem;
    padding-bottom: 1rem;
    overflow: auto;
  }

  @media (max-width: 640px) {
    .dialog-backdrop {
      padding: 1rem;
      align-items: flex-end;
    }

    .dialog-card {
      width: 100%;
      max-height: 92vh;
      border-radius: 18px 18px 14px 14px;
    }
  }
</style>
