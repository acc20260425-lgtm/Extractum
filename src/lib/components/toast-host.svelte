<script lang="ts">
  import { X } from "@lucide/svelte";
  import { dismissToast, toasts } from "$lib/toasts";
</script>

<div class="toast-host" aria-live="polite" aria-atomic="false">
  {#each $toasts as toast (toast.id)}
    <section class={`toast ${toast.kind}`} role={toast.kind === "error" ? "alert" : "status"}>
      <p>{toast.message}</p>
      <button
        class="toast-close"
        type="button"
        aria-label="Dismiss notification"
        onclick={() => dismissToast(toast.id)}
      >
        <X size={15} aria-hidden="true" />
      </button>
    </section>
  {/each}
</div>

<style>
  .toast-host {
    position: fixed;
    top: 4.75rem;
    right: 1.25rem;
    z-index: 60;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    width: min(24rem, calc(100vw - 2rem));
    pointer-events: none;
  }

  .toast {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 0.75rem;
    background: color-mix(in srgb, var(--panel) 92%, transparent);
    border: 1px solid var(--border);
    border-left: 4px solid var(--primary);
    border-radius: 12px;
    box-shadow: var(--shadow);
    padding: 0.85rem 0.9rem;
    backdrop-filter: blur(14px);
    pointer-events: auto;
  }

  .toast.error {
    border-left-color: var(--danger);
    background: color-mix(in srgb, var(--status-error-bg) 72%, var(--panel) 28%);
  }

  .toast.success {
    border-left-color: #15803d;
    background: color-mix(in srgb, #22c55e 14%, var(--panel));
  }

  .toast.info {
    border-left-color: var(--primary);
  }

  .toast p {
    margin: 0;
    color: var(--text);
    font-size: 0.9rem;
    line-height: 1.45;
    overflow-wrap: anywhere;
  }

  .toast-close {
    flex: 0 0 auto;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: 1px solid var(--border);
    color: var(--muted);
    width: 1.9rem;
    height: 1.9rem;
    border-radius: 999px;
    padding: 0;
    font-size: 1rem;
    line-height: 1;
  }

  .toast-close:hover {
    background: var(--panel-hover);
  }

  @media (max-width: 640px) {
    .toast-host {
      top: auto;
      bottom: 1rem;
      left: 1rem;
      right: 1rem;
      width: auto;
    }
  }
</style>
