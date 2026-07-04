<script lang="ts">
  import {
    formatPeriodDate,
    periodRangeLabel,
    type PeriodPreset,
  } from "$lib/ui/research-projects-period";

  let {
    presets,
    selectedId,
    dataRange,
    onSelect,
  }: {
    presets: PeriodPreset[];
    selectedId?: string;
    dataRange: { from: number; to: number } | null;
    onSelect?: (preset: PeriodPreset) => void;
  } = $props();

  let customFrom = $state("");
  let customTo = $state("");

  let applyDisabled = $derived(!customFrom || !customTo || customFrom > customTo);

  function dayStart(iso: string): number {
    return new Date(`${iso}T00:00:00`).getTime() / 1000;
  }

  function applyCustom() {
    if (applyDisabled) return;
    const from = dayStart(customFrom);
    const to = dayStart(customTo) + 86_399;
    onSelect?.({
      id: "custom",
      label: `${formatPeriodDate(from)}–${formatPeriodDate(to)}`,
      from,
      to,
    });
  }
</script>

<div class="period-panel">
  {#if dataRange}
    <div class="period-panel__span">
      <svg
        width="11"
        height="11"
        viewBox="0 0 16 16"
        fill="none"
        stroke="currentColor"
        stroke-width="1.5"
      >
        <circle cx="8" cy="8" r="6" />
        <path d="M8 5v3.2l2 1.3" />
      </svg>
      Данные проекта: {periodRangeLabel(dataRange.from, dataRange.to)}
    </div>
  {/if}
  <div role="listbox" class="period-panel__list">
    {#each presets as preset (preset.id)}
      <button
        type="button"
        role="option"
        class="period-panel__item"
        aria-selected={preset.id === selectedId}
        onclick={() => onSelect?.(preset)}
      >
        <span class="period-panel__item-body">
          <span class="period-panel__label">{preset.label}</span>
          <span class="period-panel__sub">{periodRangeLabel(preset.from, preset.to)}</span>
        </span>
        {#if preset.id === selectedId}
          <span class="period-panel__check">✓</span>
        {/if}
      </button>
    {/each}
  </div>
  <div class="period-panel__divider"></div>
  <div class="period-panel__custom">
    <div class="period-panel__custom-title">Произвольный диапазон</div>
    <div class="period-panel__dates">
      <input type="date" aria-label="Дата начала" bind:value={customFrom} />
      <span class="period-panel__arrow">→</span>
      <input type="date" aria-label="Дата конца" bind:value={customTo} />
    </div>
    <button
      type="button"
      class="period-panel__apply"
      disabled={applyDisabled}
      onclick={applyCustom}
    >
      Применить диапазон
    </button>
  </div>
</div>

<style>
  .period-panel {
    display: flex;
    flex-direction: column;
    width: 100%;
  }

  .period-panel__span {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 8px 7px;
    font: 500 10.5px/1.3 var(--extractum-font);
    color: var(--extractum-muted-2);
  }

  .period-panel__list {
    display: flex;
    flex-direction: column;
  }

  .period-panel__list .period-panel__item {
    display: flex;
    align-items: center;
    gap: 9px;
    padding: 8px;
    border: none;
    border-radius: 6px;
    background: transparent;
    text-align: left;
    cursor: pointer;
    color: var(--extractum-text);
  }

  .period-panel__list .period-panel__item:hover {
    background: var(--extractum-surface-subtle);
  }

  .period-panel__list .period-panel__item[aria-selected="true"] {
    background: color-mix(in srgb, var(--extractum-primary) 6%, transparent);
  }

  .period-panel__item-body {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .period-panel__label {
    font: 600 12.5px/1.2 var(--extractum-font);
  }

  .period-panel__sub {
    font: 500 10.5px/1.2 "SF Mono", Menlo, Consolas, monospace;
    color: var(--extractum-muted-2);
  }

  .period-panel__check {
    color: var(--extractum-primary);
    font-size: 12px;
  }

  .period-panel__divider {
    height: 1px;
    background: var(--extractum-border);
    margin: 5px 2px;
  }

  .period-panel__custom {
    padding: 4px 8px 6px;
  }

  .period-panel__custom-title {
    font: 600 10px/1 var(--extractum-font);
    letter-spacing: 0.04em;
    color: var(--extractum-muted-2);
    text-transform: uppercase;
    margin-bottom: 7px;
  }

  .period-panel__dates {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .period-panel__dates input {
    height: 29px;
    flex: 1;
    min-width: 0;
    border: 1px solid var(--extractum-border);
    border-radius: 5px;
    padding: 0 6px;
    font: 500 11.5px var(--extractum-font);
    color: var(--extractum-text);
    background: var(--extractum-surface-raised);
  }

  .period-panel__arrow {
    color: var(--extractum-muted-2);
    font-size: 11px;
  }

  .period-panel__custom .period-panel__apply {
    margin-top: 8px;
    width: 100%;
    height: 30px;
    border: none;
    border-radius: 6px;
    background: color-mix(in srgb, var(--extractum-primary) 10%, transparent);
    color: var(--extractum-primary);
    font: 600 12px/1 var(--extractum-font);
    cursor: pointer;
  }

  .period-panel__custom .period-panel__apply:hover:not(:disabled) {
    background: color-mix(in srgb, var(--extractum-primary) 18%, transparent);
  }

  .period-panel__custom .period-panel__apply:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
