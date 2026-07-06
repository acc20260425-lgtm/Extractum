<script lang="ts">
  import type { SourceFilterChip } from "$lib/ui/research-projects-source-filters";

  let {
    filtersOpen,
    onToggleFilters,
    chips = [],
    onRemoveChip,
    filtersActive = false,
    onClearAll,
    shownCount,
    totalCount,
    onAddSource,
    onConnectFromLibrary,
  }: {
    filtersOpen: boolean;
    onToggleFilters?: () => void;
    chips?: SourceFilterChip[];
    onRemoveChip?: (key: string) => void;
    filtersActive?: boolean;
    onClearAll?: () => void;
    shownCount: number;
    totalCount: number;
    onAddSource?: () => void;
    onConnectFromLibrary?: () => void;
  } = $props();
</script>

<div class="sources-filter-bar">
  <div class="sources-filter-bar__left">
    <button
      type="button"
      class="sources-filter-bar__filters-btn"
      aria-expanded={filtersOpen}
      onclick={() => onToggleFilters?.()}
    >
      <svg
        width="13"
        height="13"
        viewBox="0 0 16 16"
        fill="none"
        stroke="currentColor"
        stroke-width="1.5"
      >
        <path d="M2 3.5h12l-4.5 5v4l-3 1.5V8.5z" />
      </svg>
      Фильтры
      {#if filtersActive}
        <span class="sources-filter-bar__badge">{chips.length}</span>
      {/if}
    </button>
    {#each chips as chip (chip.key)}
      <span class="sources-filter-bar__chip">
        {#if chip.dot}
          <span class="sources-filter-bar__chip-dot" style:background={chip.dot}></span>
        {/if}
        {chip.label}
        <button
          type="button"
          class="sources-filter-bar__chip-remove"
          aria-label={`Убрать фильтр ${chip.label}`}
          onclick={() => onRemoveChip?.(chip.key)}
        >
          ✕
        </button>
      </span>
    {/each}
    {#if filtersActive}
      <button type="button" class="sources-filter-bar__clear" onclick={() => onClearAll?.()}>
        Сбросить
      </button>
    {/if}
    <span class="sources-filter-bar__count">{shownCount} из {totalCount}</span>
  </div>
  <div class="sources-filter-bar__actions">
    <button
      type="button"
      class="sources-filter-bar__add"
      data-ui-action="add-source"
      aria-label="Add source"
      title="Add source"
      onclick={() => onAddSource?.()}
    >
      <span class="sources-filter-bar__add-plus">+</span>Add source
    </button>
    <button
      type="button"
      class="sources-filter-bar__connect"
      data-ui-action="connect-library"
      aria-label="Connect from Library"
      title="Connect from Library"
      onclick={() => onConnectFromLibrary?.()}
    >
      Connect from Library
    </button>
  </div>
</div>

<style>
  .sources-filter-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 14px;
    padding: 11px 14px 9px;
    background: var(--extractum-surface-raised);
    border-bottom: 1px solid var(--extractum-border-subtle, var(--extractum-border));
  }

  .sources-filter-bar__left {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
    min-width: 0;
  }

  /* scoped override глобального button-правила */
  .sources-filter-bar__left .sources-filter-bar__filters-btn {
    height: 30px;
    display: inline-flex;
    align-items: center;
    gap: 7px;
    padding: 0 11px;
    border: 1px solid var(--extractum-border);
    border-radius: 6px;
    background: var(--extractum-surface);
    color: var(--extractum-text);
    font: 600 12px/1 var(--extractum-font);
    cursor: pointer;
  }

  .sources-filter-bar__left .sources-filter-bar__filters-btn:hover {
    background: var(--extractum-surface-subtle);
  }

  .sources-filter-bar__badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 16px;
    height: 16px;
    padding: 0 4px;
    border-radius: 8px;
    background: var(--extractum-primary);
    color: #fff;
    font: 700 10px/1 var(--extractum-font);
  }

  .sources-filter-bar__chip {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 24px;
    padding: 0 6px 0 9px;
    border-radius: 6px;
    background: color-mix(in srgb, var(--extractum-primary) 8%, var(--extractum-surface));
    border: 1px solid color-mix(in srgb, var(--extractum-primary) 24%, transparent);
    font: 600 11.5px/1 var(--extractum-font);
    color: var(--extractum-primary);
  }

  .sources-filter-bar__chip-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
  }

  .sources-filter-bar__chip .sources-filter-bar__chip-remove {
    border: none;
    background: transparent;
    padding: 0;
    cursor: pointer;
    color: color-mix(in srgb, var(--extractum-primary) 55%, transparent);
    font-size: 13px;
    line-height: 1;
  }

  .sources-filter-bar__left .sources-filter-bar__clear {
    border: none;
    background: transparent;
    padding: 0;
    font: 500 11.5px/1 var(--extractum-font);
    color: var(--extractum-muted);
    cursor: pointer;
    text-decoration: underline;
    text-underline-offset: 2px;
  }

  .sources-filter-bar__count {
    font: 500 11.5px/1 var(--extractum-font);
    color: var(--extractum-muted-2);
  }

  .sources-filter-bar__actions {
    display: inline-flex;
    flex-shrink: 0;
    align-items: center;
    gap: 8px;
  }

  .sources-filter-bar__actions > button {
    height: 28px;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 0 11px;
    border-radius: 6px;
    font: 600 12px/1 var(--extractum-font);
    cursor: pointer;
  }

  .sources-filter-bar__add {
    border: 1px solid var(--extractum-primary);
    background: color-mix(in srgb, var(--extractum-primary) 6%, transparent);
    color: var(--extractum-primary);
  }

  .sources-filter-bar__add:hover {
    background: color-mix(in srgb, var(--extractum-primary) 12%, transparent);
  }

  .sources-filter-bar__connect {
    border: 1px solid var(--extractum-border);
    background: var(--extractum-surface);
    color: var(--extractum-text);
  }

  .sources-filter-bar__connect:hover {
    background: var(--extractum-surface-subtle);
  }

  .sources-filter-bar__add-plus {
    font-size: 14px;
  }
</style>
