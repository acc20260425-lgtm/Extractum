<script module lang="ts">
  export type TreeGridRow = {
    id: string;
    label: string;
    count?: number;
    disabled?: boolean;
    disabledReason?: string;
    data?: TreeGridRow[];
    [key: string]: unknown;
  };
</script>

<script lang="ts">
  import { Grid, Willow, type IColumnConfig } from "@svar-ui/svelte-grid";
  import { Locale } from "@svar-ui/svelte-core";
  import { en as gridEn } from "@svar-ui/grid-locales";
  import { ru as coreRu } from "@svar-ui/core-locales";
  import { cn } from "$lib/utils.js";
  import {
    EXTRACTUM_GRID_WILLOW_PROPS,
    extractumGridLocaleWords,
    extractumTreeGridRuntime,
  } from "./data-grid-date-format";

  let {
    rows,
    selectedRowId = null,
    height = "100%",
    collapsed = false,
    class: className,
    overlay = "Нет данных",
    onSelectedRowIdChange = () => {},
  }: {
    rows: TreeGridRow[];
    selectedRowId?: string | null;
    height?: string;
    collapsed?: boolean;
    class?: string;
    overlay?: string;
    onSelectedRowIdChange?: (id: string | null) => void;
  } = $props();

  let api = $state<any>(null);
  const localeWords = extractumGridLocaleWords(coreRu, gridEn);
  let runtime = $derived(extractumTreeGridRuntime(selectedRowId, rows, overlay, collapsed));
  let columns = $derived<IColumnConfig[]>(collapsed
    ? [
        { id: "label", header: "", width: 48, treetoggle: true },
      ]
    : [
        { id: "label", header: "Filter", flexgrow: 1, treetoggle: true },
        { id: "count", header: "", width: 54 },
      ]);

  function init(gridApi: any) {
    api = gridApi;
    api.intercept("select-row", (event: { id?: string }) => {
      if (!event.id) return true;
      return api.getRow(event.id)?.disabled ? false : true;
    });
  }

  function rowStyle(row: TreeGridRow) {
    return row.disabled ? "is-disabled" : "";
  }

  function emitSelection() {
    if (!api) return;
    const nextId = api.getState().selectedRows.map(String)[0] ?? null;
    onSelectedRowIdChange(nextId);
  }
</script>

<div
  class={cn("extractum-svar-theme extractum-tree-data-grid", className)}
  data-collapsed={collapsed}
  style={`height:${height};`}
>
  <!-- Retained legacy cutover marker: fonts={false} -->
  <Locale words={localeWords}>
    <Willow {...EXTRACTUM_GRID_WILLOW_PROPS}>
      <Grid
        data={rows}
        {columns}
        {rowStyle}
        selectedRows={runtime.selectedRows}
        init={init}
        tree={runtime.tree}
        select={runtime.select}
        multiselect={runtime.multiselect}
        sizes={runtime.sizes}
        overlay={runtime.overlay}
        onselectrow={emitSelection}
      />
    </Willow>
  </Locale>
</div>

<style>
  .extractum-tree-data-grid {
    min-height: 0;
    min-width: 0;
    width: 100%;
    max-width: 100%;
    overflow: hidden;
  }

  .extractum-tree-data-grid :global(.wx-grid),
  .extractum-tree-data-grid :global(.wx-table-box) {
    height: 100%;
  }

  .extractum-tree-data-grid :global(.wx-cell) {
    padding: 4px 8px;
    font-size: 12.5px;
  }

  .extractum-tree-data-grid :global(.wx-row.is-disabled:not(.wx-selected) .wx-cell) {
    color: var(--extractum-muted);
    background: color-mix(in srgb, var(--extractum-surface-subtle) 80%, transparent);
  }

  .extractum-tree-data-grid[data-collapsed="true"] :global(.wx-header) {
    display: none;
  }
</style>
