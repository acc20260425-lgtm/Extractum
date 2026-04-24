<script lang="ts">
  import { tick } from "svelte";
  import { cubicOut } from "svelte/easing";
  import { fade, scale } from "svelte/transition";
  import {
    activeModal,
    confirmActiveModal,
    dismissActiveModal,
  } from "$lib/modals";

  let modalElement = $state<HTMLDivElement | null>(null);
  let cancelButton = $state<HTMLButtonElement | null>(null);
  let confirmButton = $state<HTMLButtonElement | null>(null);
  let previousFocusedElement = $state<HTMLElement | null>(null);

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      dismissActiveModal();
    }
  }

  async function focusModal() {
    if (!$activeModal) return;

    previousFocusedElement = document.activeElement instanceof HTMLElement
      ? document.activeElement
      : null;

    await tick();
    cancelButton?.focus();
  }

  function restoreFocus() {
    previousFocusedElement?.focus();
    previousFocusedElement = null;
  }

  function trapFocus(event: KeyboardEvent) {
    if (event.key !== "Tab" || !modalElement) return;

    const focusable = Array.from(
      modalElement.querySelectorAll<HTMLElement>(
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

  function handleModalKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      dismissActiveModal();
      return;
    }

    if (event.key === "Enter") {
      const target = event.target as HTMLElement | null;
      if (target?.tagName === "BUTTON") return;

      event.preventDefault();
      confirmActiveModal();
      return;
    }

    trapFocus(event);
  }

  $effect(() => {
    if ($activeModal) {
      void focusModal();
      return;
    }

    restoreFocus();
  });
</script>

<svelte:window
  onkeydown={(event) => {
    if ($activeModal && event.key === "Escape" && event.target === document.body) {
      event.preventDefault();
      dismissActiveModal();
    }
  }}
/>

{#if $activeModal}
  <div
    class="modal-backdrop"
    role="presentation"
    onclick={handleBackdropClick}
    transition:fade={{ duration: 120 }}
  >
    {#if $activeModal.kind === "confirm"}
      <div
        bind:this={modalElement}
        class="modal-card"
        class:danger={$activeModal.tone === "danger"}
        role="dialog"
        aria-modal="true"
        aria-labelledby="modal-title"
        aria-describedby="modal-description"
        tabindex="-1"
        onkeydown={handleModalKeydown}
        transition:scale={{ duration: 150, start: 0.985, easing: cubicOut }}
      >
        <header class="modal-header">
          <h2 id="modal-title">{$activeModal.title}</h2>
        </header>

        <div class="modal-body">
          <p id="modal-description">{$activeModal.message}</p>
        </div>

        <footer class="modal-actions">
          <button
            bind:this={cancelButton}
            class="secondary"
            type="button"
            onclick={dismissActiveModal}
          >
            {$activeModal.cancelLabel}
          </button>
          <button
            bind:this={confirmButton}
            class:danger={$activeModal.tone === "danger"}
            type="button"
            onclick={confirmActiveModal}
          >
            {$activeModal.confirmLabel}
          </button>
        </footer>
      </div>
    {/if}
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    z-index: 70;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1.5rem;
    background: color-mix(in srgb, #0f172a 28%, transparent);
    backdrop-filter: blur(6px);
  }

  .modal-card {
    width: min(28rem, calc(100vw - 2rem));
    border-radius: 16px;
    background: color-mix(in srgb, var(--panel) 97%, transparent);
    border: 1px solid color-mix(in srgb, var(--border) 94%, transparent);
    box-shadow:
      0 18px 42px rgba(15, 23, 42, 0.18),
      0 3px 10px rgba(15, 23, 42, 0.08);
    overflow: hidden;
  }

  .modal-card.danger {
    border-color: color-mix(in srgb, var(--danger) 22%, var(--border));
  }

  .modal-header,
  .modal-body,
  .modal-actions {
    padding-left: 1rem;
    padding-right: 1rem;
  }

  .modal-header {
    padding-top: 0.95rem;
    padding-bottom: 0.2rem;
  }

  .modal-header h2 {
    margin: 0;
    font-size: 0.98rem;
    font-weight: 650;
    letter-spacing: 0;
  }

  .modal-body {
    padding-top: 0.3rem;
    padding-bottom: 0.9rem;
  }

  .modal-body p {
    margin: 0;
    color: var(--muted);
    font-size: 0.91rem;
    line-height: 1.45;
    white-space: pre-wrap;
  }

  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    padding-top: 0.85rem;
    padding-bottom: 0.95rem;
    border-top: 1px solid var(--border);
    background: color-mix(in srgb, var(--panel-strong) 54%, transparent);
  }

  button.danger {
    background: color-mix(in srgb, var(--danger) 90%, black 10%);
  }

  button.danger:hover {
    background: var(--danger-hover);
  }

  @media (max-width: 640px) {
    .modal-backdrop {
      padding: 1rem;
      align-items: flex-end;
    }

    .modal-card {
      width: 100%;
      border-radius: 18px 18px 14px 14px;
    }

    .modal-actions {
      flex-direction: column-reverse;
    }

    .modal-actions :global(button) {
      width: 100%;
    }
  }
</style>
