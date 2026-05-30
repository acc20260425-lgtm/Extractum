<script lang="ts">
  import { Dialog } from "bits-ui";
  import { X } from "@lucide/svelte";
  import { cubicOut } from "svelte/easing";
  import { fade, scale } from "svelte/transition";

  let {
    open,
    title,
    description = "",
    labelledBy = "desktop-dialog-title",
    width = "46rem",
    smokeId,
    onClose,
    children,
  }: {
    open: boolean;
    title: string;
    description?: string;
    labelledBy?: string;
    width?: string;
    smokeId?: string;
    onClose: () => void;
    children?: import("svelte").Snippet;
  } = $props();

  function handleOpenChange(nextOpen: boolean) {
    if (!nextOpen && open) {
      onClose();
    }
  }
</script>

<Dialog.Root {open} onOpenChange={handleOpenChange}>
  <Dialog.Portal>
    <Dialog.Overlay forceMount>
      {#snippet child({ props, open })}
        {#if open}
          <div
            {...props}
            class="dialog-backdrop"
            transition:fade={{ duration: 120 }}
          ></div>
        {/if}
      {/snippet}
    </Dialog.Overlay>

    <Dialog.Content
      forceMount
      trapFocus={true}
      interactOutsideBehavior="ignore"
    >
      {#snippet child({ props, open })}
        {#if open}
          <div
            {...props}
            class="dialog-card"
            data-smoke-id={smokeId}
            style={`${String(props.style ?? "")}; --dialog-width: ${width};`}
            transition:scale={{ duration: 150, start: 0.985, easing: cubicOut }}
          >
            <header class="dialog-header">
              <div class="dialog-copy">
                <Dialog.Title id={labelledBy} level={4} class="dialog-title">
                  {title}
                </Dialog.Title>
                {#if description}
                  <Dialog.Description
                    id={`${labelledBy}-description`}
                    class="dialog-description"
                  >
                    {description}
                  </Dialog.Description>
                {/if}
              </div>
              <Dialog.Close>
                {#snippet child({ props })}
                  <button
                    {...props}
                    class="dialog-close"
                    type="button"
                    aria-label="Close dialog"
                    title="Close dialog"
                  >
                    <X size={16} aria-hidden="true" />
                  </button>
                {/snippet}
              </Dialog.Close>
            </header>

            <div class="dialog-content">
              {@render children?.()}
            </div>
          </div>
        {/if}
      {/snippet}
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>

<style>
  .dialog-backdrop {
    position: fixed;
    inset: 0;
    z-index: 60;
    background: color-mix(in srgb, #0f172a 28%, transparent);
    backdrop-filter: blur(6px);
  }

  .dialog-card {
    position: fixed;
    top: 50%;
    left: 50%;
    z-index: 61;
    translate: -50% -50%;
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

  .dialog-title {
    margin: 0;
    font-size: 0.98rem;
    font-weight: 650;
  }

  .dialog-description {
    margin: 0;
    color: var(--muted);
    font-size: 0.9rem;
    line-height: 1.45;
  }

  .dialog-close {
    width: 2.25rem;
    height: 2.25rem;
    padding: 0;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: 6px;
    border: 1px solid transparent;
    background: transparent;
    color: var(--muted);
    cursor: pointer;
    transition: background 0.2s, border-color 0.2s, color 0.2s;
  }

  .dialog-close:hover {
    color: var(--text);
    background: color-mix(in srgb, var(--panel-hover) 72%, transparent);
  }

  .dialog-close:focus-visible {
    outline: 2px solid var(--primary);
    outline-offset: 2px;
  }

  .dialog-content {
    padding-top: 1rem;
    padding-bottom: 1rem;
    overflow: auto;
  }

  @media (max-width: 640px) {
    .dialog-card {
      top: auto;
      right: 1rem;
      bottom: 1rem;
      left: 1rem;
      translate: none;
      width: auto;
      max-height: 92vh;
      border-radius: 18px 18px 14px 14px;
    }
  }
</style>
