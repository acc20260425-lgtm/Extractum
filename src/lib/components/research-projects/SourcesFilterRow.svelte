<script lang="ts">
  import {
    ExtractumPopover,
    ExtractumPopoverTrigger,
    ExtractumPopoverContent,
  } from "$lib/components/extractum-ui";
  import { SOURCE_FILTER_ROW_GRID_TEMPLATE } from "$lib/ui/research-projects-source-row";
  import type { SourceFilters } from "$lib/ui/research-projects-source-filters";

  let {
    filters,
    onChange,
  }: {
    filters: SourceFilters;
    onChange?: (filters: SourceFilters) => void;
  } = $props();

  const TYPE_OPTIONS = [
    { value: "telegram", dot: "var(--extractum-provider-telegram)" },
    { value: "youtube", dot: "var(--extractum-provider-youtube)" },
  ];

  const STATUS_OPTIONS = [
    { value: "active", dot: "var(--extractum-success)" },
    { value: "syncing", dot: "var(--extractum-primary)" },
    { value: "error", dot: "var(--extractum-danger)" },
    { value: "unavailable", dot: "var(--extractum-warning)" },
  ];

  function patch(partial: Partial<SourceFilters>) {
    onChange?.({ ...filters, ...partial });
  }

  function toggleIn(list: string[], value: string): string[] {
    return list.includes(value) ? list.filter((v) => v !== value) : [...list, value];
  }

  function multiLabel(selected: string[]): string {
    if (selected.length === 0) return "Все";
    if (selected.length === 1) return selected[0];
    return `${selected.length} выбр.`;
  }

  function numberOrNull(raw: string): number | null {
    const value = Number(raw.replace(/\D/g, ""));
    return raw.trim() === "" || !Number.isFinite(value) ? null : value;
  }
</script>

<div
  class="sources-filter-row"
  style={`grid-template-columns: ${SOURCE_FILTER_ROW_GRID_TEMPLATE};`}
>
  <div></div>

  <div class="sources-filter-row__search">
    <svg
      width="12"
      height="12"
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      stroke-width="1.5"
    >
      <circle cx="7" cy="7" r="4.5" />
      <path d="M10.5 10.5L14 14" />
    </svg>
    <input
      value={filters.query}
      placeholder="Поиск"
      oninput={(e) => patch({ query: (e.currentTarget as HTMLInputElement).value })}
    />
    {#if filters.query.length > 0}
      <button
        type="button"
        class="sources-filter-row__clear"
        title="Очистить поиск"
        onclick={() => patch({ query: "" })}
      >
        ×
      </button>
    {/if}
  </div>

  <ExtractumPopover>
    <ExtractumPopoverTrigger class="sources-filter-row__dd" aria-label="Фильтр по типу">
      {multiLabel(filters.types)}<span class="sources-filter-row__caret">▾</span>
    </ExtractumPopoverTrigger>
    <ExtractumPopoverContent class="sources-filter-row__popover" align="start">
      {#each TYPE_OPTIONS as option (option.value)}
        <label class="sources-filter-row__option">
          <input
            type="checkbox"
            aria-label={option.value}
            checked={filters.types.includes(option.value)}
            onchange={() => patch({ types: toggleIn(filters.types, option.value) })}
          />
          <span class="sources-filter-row__dot" style:background={option.dot}></span>
          {option.value}
        </label>
      {/each}
    </ExtractumPopoverContent>
  </ExtractumPopover>

  <div class="sources-filter-row__range">
    <input
      type="number"
      aria-label="Материалы от"
      placeholder="от"
      value={filters.materialsMin ?? ""}
      oninput={(e) =>
        patch({ materialsMin: numberOrNull((e.currentTarget as HTMLInputElement).value) })}
    />
    <input
      type="number"
      aria-label="Материалы до"
      placeholder="до"
      value={filters.materialsMax ?? ""}
      oninput={(e) =>
        patch({ materialsMax: numberOrNull((e.currentTarget as HTMLInputElement).value) })}
    />
  </div>

  <div class="sources-filter-row__range">
    <input
      type="date"
      aria-label="Синхронизирован с"
      value={filters.syncedFrom ?? ""}
      oninput={(e) => patch({ syncedFrom: (e.currentTarget as HTMLInputElement).value || null })}
    />
    <input
      type="date"
      aria-label="Синхронизирован по"
      value={filters.syncedTo ?? ""}
      oninput={(e) => patch({ syncedTo: (e.currentTarget as HTMLInputElement).value || null })}
    />
  </div>

  <ExtractumPopover>
    <ExtractumPopoverTrigger class="sources-filter-row__dd" aria-label="Фильтр по статусу">
      {multiLabel(filters.statuses)}<span class="sources-filter-row__caret">▾</span>
    </ExtractumPopoverTrigger>
    <ExtractumPopoverContent class="sources-filter-row__popover" align="end">
      {#each STATUS_OPTIONS as option (option.value)}
        <label class="sources-filter-row__option">
          <input
            type="checkbox"
            aria-label={option.value}
            checked={filters.statuses.includes(option.value)}
            onchange={() => patch({ statuses: toggleIn(filters.statuses, option.value) })}
          />
          <span class="sources-filter-row__dot" style:background={option.dot}></span>
          {option.value}
        </label>
      {/each}
    </ExtractumPopoverContent>
  </ExtractumPopover>
</div>

<style>
  .sources-filter-row {
    display: grid;
    align-items: center;
    gap: 6px;
    padding: 6px 8px;
    background: var(--extractum-surface);
    border-bottom: 1px solid var(--extractum-border-subtle, var(--extractum-border));
  }

  .sources-filter-row__search {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 28px;
    border: 1px solid var(--extractum-border);
    border-radius: 5px;
    background: var(--extractum-surface-raised);
    padding: 0 8px;
    color: var(--extractum-muted-2);
  }

  .sources-filter-row__search input {
    flex: 1;
    min-width: 0;
    border: none;
    outline: none;
    background: transparent;
    font: 400 12px/1 var(--extractum-font);
    color: var(--extractum-text);
  }

  .sources-filter-row__search .sources-filter-row__clear {
    border: none;
    background: transparent;
    padding: 0;
    cursor: pointer;
    color: var(--extractum-muted-2);
    font-size: 14px;
    line-height: 1;
  }

  /* поповер-триггеры: override глобального button-правила */
  .sources-filter-row :global(.sources-filter-row__dd) {
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    border: 1px solid var(--extractum-border);
    border-radius: 5px;
    background: var(--extractum-surface-raised);
    padding: 0 8px;
    font: 400 12px/1 var(--extractum-font);
    color: var(--extractum-text);
    cursor: pointer;
  }

  .sources-filter-row__caret {
    color: var(--extractum-muted-2);
    font-size: 10px;
  }

  :global(.sources-filter-row__popover) {
    display: flex;
    flex-direction: column;
    gap: 2px;
    width: 160px;
    padding: 5px;
  }

  :global(.sources-filter-row__popover .sources-filter-row__option) {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 8px;
    border-radius: 5px;
    font: 500 12px/1 var(--extractum-font);
    color: var(--extractum-text);
    cursor: pointer;
  }

  :global(.sources-filter-row__popover .sources-filter-row__option:hover) {
    background: var(--extractum-surface-subtle);
  }

  .sources-filter-row__dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
  }

  .sources-filter-row__range {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .sources-filter-row__range input {
    width: 0;
    flex: 1;
    height: 28px;
    border: 1px solid var(--extractum-border);
    border-radius: 5px;
    background: var(--extractum-surface-raised);
    padding: 0 6px;
    font: 400 11.5px/1 var(--extractum-font);
    color: var(--extractum-text);
    outline: none;
  }
</style>
