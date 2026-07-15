<script lang="ts">
  import Download from "@lucide/svelte/icons/download";

  // Note: the v10 mockup showed a progress bar + "62% · ~3 мин"; per the
  // session decision, run progress percentage is out of scope (the backend
  // does not track it), so the dock shows an indeterminate pulse instead.
  let {
    activeRunLabel = null,
    queueCount = 0,
    exportLabel = "Экспорт",
    exportDisabled = false,
    onExport,
  }: {
    activeRunLabel?: string | null;
    queueCount?: number;
    exportLabel?: string;
    exportDisabled?: boolean;
    onExport?: () => void;
  } = $props();
</script>

<div class="run-dock">
  {#if activeRunLabel}
    <span class="run-dock__active">
      <span class="run-dock__pulse"></span>
      {activeRunLabel}
    </span>
  {:else}
    <span class="run-dock__idle">Нет активных прогонов</span>
  {/if}

  <div class="run-dock__spacer"></div>

  <span class="run-dock__queue">Очередь: {queueCount}</span>

  <button
    class="run-dock__export"
    type="button"
    disabled={exportDisabled}
    onclick={() => onExport?.()}
  >
    <Download size={12} aria-hidden="true" />
    {exportLabel}
  </button>
</div>

<style>
  .run-dock {
    display: flex;
    align-items: center;
    gap: 14px;
    height: 50px;
    flex-shrink: 0;
    padding: 0 16px;
    background: var(--extractum-surface-subtle);
    border-top: 1px solid var(--extractum-border);
  }

  .run-dock__active {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    font: 600 12px/1 var(--extractum-font);
    color: var(--extractum-primary);
  }

  .run-dock__pulse {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--extractum-primary);
    animation: run-dock-pulse 1.4s infinite;
  }

  @keyframes run-dock-pulse {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.35;
    }
  }

  .run-dock__idle {
    font: 500 12px/1 var(--extractum-font);
    color: var(--extractum-muted);
  }

  .run-dock__spacer {
    flex: 1;
  }

  .run-dock__queue {
    font: 500 11.5px/1 var(--extractum-font);
    color: var(--extractum-muted);
  }

  .run-dock__export {
    height: 28px;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 0 11px;
    border: 1px solid var(--extractum-border);
    border-radius: var(--extractum-radius);
    background: var(--extractum-surface-raised);
    color: var(--extractum-text);
    font: 500 12px/1 var(--extractum-font);
    cursor: pointer;
  }

  .run-dock__export:hover:not(:disabled) {
    background: var(--extractum-surface-subtle);
  }

  .run-dock__export:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
