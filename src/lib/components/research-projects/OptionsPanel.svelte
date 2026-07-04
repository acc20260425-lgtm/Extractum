<script lang="ts">
  import type { ComboOption } from "./ComboSelect.svelte";

  let {
    options,
    selectedValue,
    placeholder,
    emptyLabel = "Ничего не найдено",
    onSelect,
  }: {
    options: ComboOption[];
    selectedValue?: string;
    placeholder: string;
    emptyLabel?: string;
    onSelect?: (option: ComboOption) => void;
  } = $props();

  let query = $state("");

  let filtered = $derived.by(() => {
    const q = query.trim().toLowerCase();
    if (!q) return options;
    return options.filter(
      (option) =>
        option.label.toLowerCase().includes(q) ||
        (option.description ?? "").toLowerCase().includes(q) ||
        (option.mono ?? "").toLowerCase().includes(q),
    );
  });
</script>

<div class="options-panel">
  <div class="options-panel__search">
    <svg
      width="13"
      height="13"
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      stroke-width="1.5"
    >
      <circle cx="7" cy="7" r="4.5" />
      <path d="M10.5 10.5L14 14" />
    </svg>
    <input bind:value={query} {placeholder} aria-label={placeholder} />
  </div>
  <div class="options-panel__list" role="listbox">
    {#each filtered as option, index (option.value)}
      {#if option.group && (index === 0 || filtered[index - 1].group !== option.group)}
        <div class="options-panel__group" role="presentation">{option.group}</div>
      {/if}
      <button
        type="button"
        role="option"
        class="options-panel__item"
        aria-selected={option.value === selectedValue}
        onclick={() => onSelect?.(option)}
      >
        {#if option.dot}
          <span class="options-panel__dot" style:background={option.dot}></span>
        {/if}
        <span class="options-panel__body">
          <span class="options-panel__label">{option.label}</span>
          {#if option.description}
            <span class="options-panel__desc">{option.description}</span>
          {:else if option.mono}
            <span class="options-panel__mono">{option.mono}</span>
          {/if}
        </span>
        {#if option.value === selectedValue}
          <span class="options-panel__check">✓</span>
        {/if}
      </button>
    {/each}
    {#if filtered.length === 0}
      <div class="options-panel__empty">{emptyLabel}</div>
    {/if}
  </div>
</div>

<style>
  .options-panel {
    display: flex;
    flex-direction: column;
    width: 100%;
  }

  .options-panel__search {
    display: flex;
    align-items: center;
    gap: 7px;
    height: 32px;
    padding: 0 8px;
    border: 1px solid var(--extractum-border);
    border-radius: 6px;
    margin-bottom: 5px;
    background: var(--extractum-surface);
    color: var(--extractum-muted-2);
  }

  .options-panel__search input {
    border: none;
    outline: none;
    background: transparent;
    flex: 1;
    min-width: 0;
    font: 500 12.5px/1 var(--extractum-font);
    color: var(--extractum-text);
  }

  .options-panel__list {
    max-height: 248px;
    overflow-y: auto;
    overflow-x: hidden;
    display: flex;
    flex-direction: column;
  }

  .options-panel__group {
    font: 600 10px/1 var(--extractum-font);
    letter-spacing: 0.04em;
    color: var(--extractum-muted-2);
    text-transform: uppercase;
    padding: 8px 8px 4px;
  }

  .options-panel__list .options-panel__item {
    display: flex;
    align-items: flex-start;
    gap: 9px;
    padding: 8px;
    border: none;
    border-radius: 6px;
    background: transparent;
    text-align: left;
    cursor: pointer;
    color: var(--extractum-text);
  }

  .options-panel__list .options-panel__item:hover {
    background: var(--extractum-surface-subtle);
  }

  .options-panel__list .options-panel__item[aria-selected="true"] {
    background: color-mix(in srgb, var(--extractum-primary) 6%, transparent);
  }

  .options-panel__dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    flex-shrink: 0;
    margin-top: 3px;
  }

  .options-panel__body {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .options-panel__label {
    font: 600 12.5px/1.3 var(--extractum-font);
  }

  .options-panel__desc {
    font: 400 11px/1.3 var(--extractum-font);
    color: var(--extractum-muted-2);
  }

  .options-panel__mono {
    font: 500 10px/1.2 "SF Mono", Menlo, Consolas, monospace;
    color: var(--extractum-muted-2);
  }

  .options-panel__check {
    color: var(--extractum-primary);
    font-size: 12px;
    margin-top: 1px;
  }

  .options-panel__empty {
    padding: 18px 8px;
    text-align: center;
    font: 500 12px/1 var(--extractum-font);
    color: var(--extractum-muted-2);
  }
</style>
