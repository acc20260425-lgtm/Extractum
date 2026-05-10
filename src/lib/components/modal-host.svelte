<script lang="ts">
  import { AlertDialog } from "bits-ui";
  import { cubicOut } from "svelte/easing";
  import { fade, scale } from "svelte/transition";
  import {
    activeModal,
    confirmActiveModal,
    dismissActiveModal,
  } from "$lib/modals";

  function handleOpenChange(nextOpen: boolean) {
    if (!nextOpen && $activeModal) {
      dismissActiveModal();
    }
  }
</script>

<AlertDialog.Root open={$activeModal !== null} onOpenChange={handleOpenChange}>
  <AlertDialog.Portal>
    <AlertDialog.Overlay forceMount>
      {#snippet child({ props, open })}
        {#if open}
          <div
            {...props}
            class="modal-backdrop"
            transition:fade={{ duration: 120 }}
          ></div>
        {/if}
      {/snippet}
    </AlertDialog.Overlay>

    <AlertDialog.Content
      forceMount
      trapFocus={true}
      interactOutsideBehavior="ignore"
    >
      {#snippet child({ props, open })}
        {#if open && $activeModal?.kind === "confirm"}
          <div
            {...props}
            class="modal-card"
            class:danger={$activeModal.tone === "danger"}
            transition:scale={{ duration: 150, start: 0.985, easing: cubicOut }}
          >
            <header class="modal-header">
              <AlertDialog.Title id="modal-title" level={2} class="modal-title">
                {$activeModal.title}
              </AlertDialog.Title>
            </header>

            <div class="modal-body">
              <AlertDialog.Description id="modal-description" class="modal-description">
                {$activeModal.message}
              </AlertDialog.Description>
            </div>

            <footer class="modal-actions">
              <AlertDialog.Cancel>
                {#snippet child({ props })}
                  <button
                    {...props}
                    class="secondary"
                    type="button"
                    onclick={dismissActiveModal}
                  >
                    {$activeModal.cancelLabel}
                  </button>
                {/snippet}
              </AlertDialog.Cancel>
              <AlertDialog.Action>
                {#snippet child({ props })}
                  <button
                    {...props}
                    class:danger={$activeModal.tone === "danger"}
                    type="button"
                    onclick={confirmActiveModal}
                  >
                    {$activeModal.confirmLabel}
                  </button>
                {/snippet}
              </AlertDialog.Action>
            </footer>
          </div>
        {/if}
      {/snippet}
    </AlertDialog.Content>
  </AlertDialog.Portal>
</AlertDialog.Root>

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    z-index: 70;
    background: color-mix(in srgb, #0f172a 28%, transparent);
    backdrop-filter: blur(6px);
  }

  .modal-card {
    position: fixed;
    top: 50%;
    left: 50%;
    z-index: 71;
    translate: -50% -50%;
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

  .modal-title {
    margin: 0;
    font-size: 0.98rem;
    font-weight: 650;
    letter-spacing: 0;
  }

  .modal-body {
    padding-top: 0.3rem;
    padding-bottom: 0.9rem;
  }

  .modal-description {
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
    .modal-card {
      top: auto;
      right: 1rem;
      bottom: 1rem;
      left: 1rem;
      translate: none;
      width: auto;
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
