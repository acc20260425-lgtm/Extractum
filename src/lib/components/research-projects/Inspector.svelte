<script lang="ts" module>
  export type InspectorSource = {
    title: string;
    handle: string | null;
    statusLabel: string;
    syncStatus: string;
    materialsLabel: string;
    lastSyncLabel: string;
  };
</script>

<script lang="ts">
  import { ChevronRight, RefreshCw, Trash2 } from "@lucide/svelte";

  let {
    open = true,
    selected,
    periodLabel,
    promptLabel,
    modelLabel,
    syncDisabled = false,
    onToggle,
    onSync,
    onDisconnect,
  }: {
    open?: boolean;
    selected: InspectorSource | null;
    periodLabel: string;
    promptLabel: string;
    modelLabel: string;
    syncDisabled?: boolean;
    onToggle?: () => void;
    onSync?: () => void;
    onDisconnect?: () => void;
  } = $props();
</script>

{#if open}
  <div class="inspector">
    <div class="inspector__header">
      <span class="inspector__eyebrow">Инспектор источника</span>
      <button
        class="inspector__toggle"
        type="button"
        title="Свернуть"
        aria-label="Свернуть"
        onclick={() => onToggle?.()}
      >
        <ChevronRight size={15} aria-hidden="true" />
      </button>
    </div>

    <div class="inspector__body">
      {#if selected}
        <div>
          <div class="inspector__name">{selected.title}</div>
          <div class="inspector__subline">
            {#if selected.handle}<span class="inspector__handle">{selected.handle}</span>{/if}
            <span class="inspector__status" data-status={selected.syncStatus}>
              {selected.statusLabel}
            </span>
          </div>
        </div>

        <div class="inspector__card">
          <div class="inspector__row">
            <span class="inspector__key">Материалы</span>
            <span class="inspector__num">{selected.materialsLabel}</span>
          </div>
          <div class="inspector__row">
            <span class="inspector__key">Последний сбор</span>
            <span class="inspector__val">{selected.lastSyncLabel}</span>
          </div>
        </div>
      {/if}

      <div>
        <div class="inspector__eyebrow inspector__eyebrow--section">Конфигурация проекта</div>
        <div class="inspector__card">
          <div class="inspector__row">
            <span class="inspector__key">Период</span>
            <span class="inspector__val">{periodLabel}</span>
          </div>
          <div class="inspector__row">
            <span class="inspector__key">Промпт</span>
            <span class="inspector__val">{promptLabel}</span>
          </div>
          <div class="inspector__row">
            <span class="inspector__key">Модель</span>
            <span class="inspector__val">{modelLabel}</span>
          </div>
        </div>
      </div>
    </div>

    {#if selected}
      <div class="inspector__footer">
        <button
          class="inspector__sync"
          type="button"
          disabled={syncDisabled}
          aria-label="Синхронизировать"
          onclick={() => onSync?.()}
        >
          <RefreshCw size={12} aria-hidden="true" />
          Синхронизировать
        </button>
        <button
          class="inspector__disconnect"
          type="button"
          title="Отключить источник"
          aria-label="Отключить источник"
          onclick={() => onDisconnect?.()}
        >
          <Trash2 size={14} aria-hidden="true" />
        </button>
      </div>
    {/if}
  </div>
{:else}
  <div class="inspector inspector--collapsed">
    <button
      class="inspector__toggle"
      type="button"
      title="Развернуть инспектор"
      aria-label="Развернуть инспектор"
      onclick={() => onToggle?.()}
    >
      <ChevronRight size={15} aria-hidden="true" style="transform: rotate(180deg);" />
    </button>
    <span class="inspector__collapsed-label">Инспектор</span>
  </div>
{/if}

<style>
  .inspector {
    display: flex;
    flex-direction: column;
    width: 300px;
    height: 100%;
    min-height: 0;
    background: var(--extractum-surface-subtle);
    border-left: 1px solid var(--extractum-border);
  }

  .inspector__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 42px;
    flex-shrink: 0;
    padding: 0 12px;
    border-bottom: 1px solid var(--extractum-border);
  }

  .inspector__eyebrow {
    font: 700 10.5px/1 var(--extractum-font);
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--extractum-muted-2);
  }

  .inspector__eyebrow--section {
    margin-bottom: 9px;
  }

  .inspector__toggle {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    border: none;
    border-radius: var(--extractum-radius);
    background: transparent;
    color: var(--extractum-muted);
    cursor: pointer;
  }

  .inspector__toggle:hover {
    background: var(--extractum-surface-raised);
  }

  .inspector__body {
    flex: 1;
    overflow: auto;
    padding: 14px;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .inspector__name {
    font: 600 16px/1.25 var(--extractum-font);
    color: var(--extractum-text);
  }

  .inspector__subline {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 6px;
  }

  .inspector__handle {
    font: 400 12px/1 var(--extractum-font);
    color: var(--extractum-muted-2);
  }

  .inspector__status {
    font: 500 11px/1 var(--extractum-font);
    color: var(--extractum-muted);
  }

  .inspector__status[data-status="active"] {
    color: var(--extractum-success);
  }
  .inspector__status[data-status="syncing"] {
    color: var(--extractum-primary);
  }
  .inspector__status[data-status="error"] {
    color: var(--extractum-danger);
  }
  .inspector__status[data-status="unavailable"] {
    color: var(--extractum-warning);
  }

  .inspector__card {
    background: var(--extractum-surface-raised);
    border: 1px solid var(--extractum-border);
    border-radius: 8px;
    overflow: hidden;
  }

  .inspector__row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 9px 12px;
  }

  .inspector__row + .inspector__row {
    border-top: 1px solid var(--extractum-surface-subtle);
  }

  .inspector__key {
    font: 500 12px/1 var(--extractum-font);
    color: var(--extractum-muted);
  }

  .inspector__val {
    font: 500 12.5px/1 var(--extractum-font);
    color: var(--extractum-text);
  }

  .inspector__num {
    font: 600 12.5px/1 var(--extractum-font);
    color: var(--extractum-text);
    font-variant-numeric: tabular-nums;
  }

  .inspector__footer {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 12px;
    border-top: 1px solid var(--extractum-border);
    background: var(--extractum-surface);
  }

  .inspector__sync {
    flex: 1;
    height: 30px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    border: 1px solid var(--extractum-border);
    border-radius: var(--extractum-radius);
    background: var(--extractum-surface-raised);
    color: var(--extractum-text);
    font: 500 12.5px/1 var(--extractum-font);
    cursor: pointer;
  }

  .inspector__sync:hover:not(:disabled) {
    background: var(--extractum-surface-subtle);
  }

  .inspector__sync:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .inspector__disconnect {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 30px;
    height: 30px;
    border: 1px solid color-mix(in srgb, var(--extractum-danger) 35%, var(--extractum-border));
    border-radius: var(--extractum-radius);
    background: var(--extractum-surface-raised);
    color: var(--extractum-danger);
    cursor: pointer;
  }

  .inspector__disconnect:hover {
    background: color-mix(in srgb, var(--extractum-danger) 8%, transparent);
  }

  .inspector--collapsed {
    width: 44px;
    align-items: center;
    padding: 9px 0;
    gap: 14px;
  }

  .inspector__collapsed-label {
    writing-mode: vertical-rl;
    transform: rotate(180deg);
    font: 600 11px/1 var(--extractum-font);
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--extractum-muted-2);
  }
</style>
