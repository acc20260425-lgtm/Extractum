<script lang="ts">
  import { fade, scale } from "svelte/transition";
  import {
    activeModal,
    confirmActiveModal,
    dismissActiveModal,
  } from "$lib/modals";

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) {
      dismissActiveModal();
    }
  }
</script>

<svelte:window
  onkeydown={(event) => {
    if ($activeModal && event.key === "Escape") {
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
    transition:fade={{ duration: 140 }}
  >
    {#if $activeModal.kind === "confirm"}
      <div
        class="modal-card"
        class:danger={$activeModal.tone === "danger"}
        role="alertdialog"
        aria-modal="true"
        aria-labelledby="modal-title"
        aria-describedby="modal-description"
        tabindex="-1"
        transition:scale={{ duration: 180, start: 0.96 }}
      >
        <header class="modal-header">
          <h2 id="modal-title">{$activeModal.title}</h2>
        </header>

        <div class="modal-body">
          <p id="modal-description">{$activeModal.message}</p>
        </div>

        <footer class="modal-actions">
          <button class="secondary" type="button" onclick={dismissActiveModal}>
            {$activeModal.cancelLabel}
          </button>
          <button
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
    background: color-mix(in srgb, #020617 44%, transparent);
    backdrop-filter: blur(10px);
  }

  .modal-card {
    width: min(30rem, calc(100vw - 2rem));
    border-radius: 18px;
    background: color-mix(in srgb, var(--panel) 94%, transparent);
    border: 1px solid color-mix(in srgb, var(--border) 88%, transparent);
    box-shadow:
      0 28px 80px rgba(2, 6, 23, 0.28),
      0 6px 20px rgba(2, 6, 23, 0.12);
    overflow: hidden;
  }

  .modal-card.danger {
    border-color: color-mix(in srgb, var(--danger) 34%, var(--border));
  }

  .modal-header,
  .modal-body,
  .modal-actions {
    padding-left: 1.2rem;
    padding-right: 1.2rem;
  }

  .modal-header {
    padding-top: 1.15rem;
    padding-bottom: 0.35rem;
  }

  .modal-header h2 {
    margin: 0;
    font-size: 1.05rem;
    letter-spacing: 0.01em;
  }

  .modal-body {
    padding-top: 0.35rem;
    padding-bottom: 1rem;
  }

  .modal-body p {
    margin: 0;
    color: var(--muted);
    line-height: 1.55;
    white-space: pre-wrap;
  }

  .modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.65rem;
    padding-top: 0.95rem;
    padding-bottom: 1.15rem;
    border-top: 1px solid var(--border);
    background: color-mix(in srgb, var(--panel-strong) 72%, transparent);
  }

  button.danger {
    background: var(--danger);
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
