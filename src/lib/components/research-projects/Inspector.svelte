<script lang="ts" module>
  export type InspectorSource = {
    title: string;
    handle: string | null;
    statusLabel: string;
    syncStatus: string;
    typeLabel: string;
    typeDot: string;
    materialsLabel: string;
    lastSyncLabel: string;
  };
</script>

<script lang="ts">
  import { ChevronRight, ExternalLink, Minus, RefreshCw } from "@lucide/svelte";

  let {
    open = true,
    selected,
    periodLabel,
    promptLabel,
    modelLabel,
    syncDisabled = false,
    openDisabled = false,
    onToggle,
    onSync,
    onOpen,
    onDisconnect,
  }: {
    open?: boolean;
    selected: InspectorSource | null;
    periodLabel: string;
    promptLabel: string;
    modelLabel: string;
    syncDisabled?: boolean;
    openDisabled?: boolean;
    onToggle?: () => void;
    onSync?: () => void;
    onOpen?: () => void;
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
        data-slot="button"
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
              <span class="inspector__status-dot" aria-hidden="true"></span>
              {selected.statusLabel}
            </span>
          </div>
        </div>

        <div class="inspector__card">
          <div class="inspector__row">
            <span class="inspector__key">Тип</span>
            <span class="inspector__type">
              <span class="inspector__type-dot" aria-hidden="true" style:background={selected.typeDot}></span>
              {selected.typeLabel}
            </span>
          </div>
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
          data-slot="button"
          disabled={syncDisabled}
          aria-label="Синхронизировать"
          onclick={() => onSync?.()}
        >
          <RefreshCw size={12} aria-hidden="true" />
          Синхронизировать
        </button>
        <div class="inspector__footer-actions">
          <button
            class="inspector__open"
            type="button"
            data-slot="button"
            title="Открыть источник"
            aria-label="Открыть"
            disabled={openDisabled || !onOpen}
            onclick={() => onOpen?.()}
          >
            <ExternalLink size={12} aria-hidden="true" />
            Открыть
          </button>
          <button
            class="inspector__disconnect"
            type="button"
            data-slot="button"
            title="Убрать из проекта"
            aria-label="Убрать"
            onclick={() => onDisconnect?.()}
          >
            <Minus size={12} aria-hidden="true" />
            Убрать
          </button>
        </div>
      </div>
    {/if}
  </div>
{:else}
  <div class="inspector inspector--collapsed">
    <button
      class="inspector__toggle"
      type="button"
      data-slot="button"
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

  button.inspector__toggle {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex: 0 0 26px;
    width: 26px;
    height: 26px;
    padding: 0;
    border: 1px solid var(--extractum-border);
    border-radius: var(--extractum-radius);
    background: var(--extractum-surface-raised);
    color: var(--extractum-muted);
    cursor: pointer;
    line-height: 1;
    transition: background 0.15s, border-color 0.15s, color 0.15s;
  }

  button.inspector__toggle:hover {
    border-color: var(--extractum-muted-2);
    background: var(--extractum-surface);
    color: var(--extractum-text);
  }

  button.inspector__toggle:focus-visible {
    outline: 2px solid var(--extractum-primary);
    outline-offset: 2px;
  }

  button.inspector__toggle :global(svg) {
    width: 15px;
    height: 15px;
    stroke-width: 2.25;
    flex-shrink: 0;
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
    display: inline-flex;
    align-items: center;
    gap: 5px;
    height: 19px;
    padding: 0 8px;
    border-radius: 5px;
    font: 500 11px/1 var(--extractum-font);
    color: var(--extractum-muted);
  }

  .inspector__status[data-status="active"] {
    color: var(--extractum-success);
    background: color-mix(in srgb, var(--extractum-success) 12%, transparent);
  }
  .inspector__status[data-status="syncing"] {
    color: var(--extractum-primary);
    background: color-mix(in srgb, var(--extractum-primary) 12%, transparent);
  }
  .inspector__status[data-status="error"] {
    color: var(--extractum-danger);
    background: color-mix(in srgb, var(--extractum-danger) 12%, transparent);
  }
  .inspector__status[data-status="unavailable"] {
    color: var(--extractum-warning);
    background: color-mix(in srgb, var(--extractum-warning) 12%, transparent);
  }

  .inspector__status-dot {
    width: 5px;
    height: 5px;
    border-radius: 999px;
    background: currentColor;
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

  .inspector__type {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font: 500 12.5px/1 var(--extractum-font);
    color: var(--extractum-text);
  }

  .inspector__type-dot {
    width: 6px;
    height: 6px;
    border-radius: 999px;
  }

  .inspector__num {
    font: 600 12.5px/1 var(--extractum-font);
    color: var(--extractum-text);
    font-variant-numeric: tabular-nums;
  }

  .inspector__footer {
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    align-items: stretch;
    gap: 8px;
    padding: 10px 12px;
    border-top: 1px solid var(--extractum-border);
    background: var(--extractum-surface);
  }

  .inspector__sync {
    width: 100%;
    height: 32px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    border: 1px solid var(--extractum-primary);
    border-radius: var(--extractum-radius);
    background: color-mix(in srgb, var(--extractum-primary) 6%, transparent);
    color: var(--extractum-primary);
    font: 600 12.5px/1 var(--extractum-font);
    cursor: pointer;
  }

  .inspector__sync:hover:not(:disabled) {
    background: color-mix(in srgb, var(--extractum-primary) 12%, transparent);
  }

  .inspector__sync:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .inspector__footer-actions {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .inspector__open,
  .inspector__disconnect {
    flex: 1;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    height: 30px;
    gap: 5px;
    border-radius: var(--extractum-radius);
    background: var(--extractum-surface-raised);
    font: 500 12px/1 var(--extractum-font);
    cursor: pointer;
  }

  .inspector__open {
    border: 1px solid var(--extractum-border);
    color: var(--extractum-text);
  }

  .inspector__open:hover:not(:disabled) {
    background: var(--extractum-surface-subtle);
  }

  .inspector__open:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .inspector__disconnect {
    border: 1px solid color-mix(in srgb, var(--extractum-danger) 35%, var(--extractum-border));
    color: var(--extractum-danger);
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
