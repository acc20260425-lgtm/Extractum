<script module lang="ts">
  import type { ExtractumDataGridColumn } from "./data-grid-date-format";

  export type { ExtractumDataGridColumn };
</script>

<script lang="ts">
  import { tick, untrack } from "svelte";
  import { Grid, Willow } from "@svar-ui/svelte-grid";
  import { Locale } from "@svar-ui/svelte-core";
  import { en as gridEn } from "@svar-ui/grid-locales";
  import { ru as coreRu } from "@svar-ui/core-locales";
  import { cn } from "$lib/utils.js";
  import {
    enhanceDateTimeColumns,
    enhanceDateTimeResponsiveColumns,
  } from "./data-grid-date-format";
  import type { ExtractumDataGridResponsive } from "./data-grid-date-format";

  type GridRow = {
    id: string;
    connectable?: boolean;
    alreadyConnected?: boolean;
    status?: string;
    [key: string]: unknown;
  };

  let {
    rows,
    columns,
    selectedRowIds = [],
    height = "100%",
    multiselect = false,
    class: className,
    ariaLabel,
    overlay = "Нет данных",
    columnStyle,
    responsive,
    selectOnClick = true,
    activeRowId = null,
    onRowClick,
    onSelectedRowIdsChange = () => {},
  }: {
    rows: GridRow[];
    columns: ExtractumDataGridColumn[];
    selectedRowIds?: string[];
    height?: string;
    multiselect?: boolean;
    class?: string;
    ariaLabel?: string;
    overlay?: string;
    columnStyle?: (column: ExtractumDataGridColumn) => string;
    responsive?: ExtractumDataGridResponsive;
    selectOnClick?: boolean;
    activeRowId?: string | null;
    onRowClick?: (id: string) => void;
    onSelectedRowIdsChange?: (ids: string[]) => void;
  } = $props();

  // svar re-normalises data and clears sortMarks whenever a reactive prop
  // changes (its config effect re-runs), so object props must keep stable
  // references and selection must NOT flow through the reactive prop at all.
  const GRID_SIZES = { rowHeight: 34, headerHeight: 34, columnWidth: 160 };

  // Unique host marker for the active-row CSS rule below.
  const gridUid = Math.random().toString(36).slice(2, 10);

  let api = $state<any>(null);
  let host = $state<HTMLDivElement | null>(null);
  let visibleOverlay = $derived(rows.length === 0 ? overlay : undefined);
  let enhancedColumns = $derived(enhanceDateTimeColumns(columns));
  let enhancedResponsive = $derived(enhanceDateTimeResponsiveColumns(responsive));

  // Selection sync without the reactive prop: the grid gets a one-time
  // initial snapshot; later EXTERNAL changes (clear selection, project
  // switch) are applied as select-row actions, which do not reset sorting.
  // Internal changes (user clicks) flow out via onselectrow → emitSelection;
  // the diff below then sees equal sets and does nothing (no echo loop).
  const initialSelectedIds = untrack(() => [...selectedRowIds]);
  $effect(() => {
    const want = selectedRowIds.map(String);
    if (!api) return;
    const current: string[] = (untrack(() => api.getState().selectedRows) ?? []).map(String);
    const wantSet = new Set(want);
    const currentSet = new Set(current);
    if (want.length === currentSet.size && want.every((id) => currentSet.has(id))) return;
    for (const id of current) {
      if (!wantSet.has(id)) api.exec("select-row", { id, mode: false, toggle: true });
    }
    for (const id of want) {
      if (!currentSet.has(id)) api.exec("select-row", { id, mode: true, toggle: true });
    }
  });

  // Row click → onRowClick(id). Delegated on the host so svar's internal
  // re-renders don't detach it; checkbox zones opt out via ignore-click.
  $effect(() => {
    const element = host;
    const handler = (event: MouseEvent) => {
      if (!onRowClick) return;
      const target = event.target as HTMLElement;
      if (target.closest('[data-action="ignore-click"]')) return;
      if (target.closest(".wx-header")) return;
      const rowEl = target.closest(".wx-row") as HTMLElement | null;
      if (!rowEl || !element?.contains(rowEl)) return;
      const raw = rowEl.dataset.id ?? "";
      const id = raw.startsWith(":") ? raw.slice(1) : raw;
      if (id) onRowClick(id);
    };
    element?.addEventListener("click", handler);
    return () => element?.removeEventListener("click", handler);
  });

  // Active-row highlight as a dynamic CSS rule: survives svar re-renders
  // (sorting, data refresh) and never touches svar reactive props.
  let activeRowCss = $derived.by(() => {
    if (!activeRowId || !/^[\w:-]+$/.test(activeRowId)) return "";
    const rule =
      `background: color-mix(in srgb, var(--extractum-primary) 7%, var(--extractum-surface));` +
      ` box-shadow: inset 2px 0 0 var(--extractum-primary);`;
    const scope = `.extractum-data-grid[data-grid-uid="${gridUid}"]`;
    return (
      `<style>` +
      `${scope} .wx-row[data-id=":${activeRowId}"] .wx-cell, ` +
      `${scope} .wx-row[data-id="${activeRowId}"] .wx-cell { ${rule} }` +
      `</style>`
    );
  });

  $effect(() => {
    if (!host) return;
    void tick().then(() => {
      if (ariaLabel) {
        host?.querySelector("[role='grid'], .wx-grid")?.setAttribute("aria-label", ariaLabel);
      }
      // svar-ui renders columnheaders as div[role="columnheader"] with text inside a nested
      // .wx-text child — Chrome doesn't compute the accessible name from nested divs, so we
      // set aria-label explicitly. Map by aria-colindex (1-based, set by svar-ui) so the
      // mapping stays correct when columns are hidden, reordered, or grouped.
      const headerCells = host?.querySelectorAll("[role='columnheader']");
      headerCells?.forEach((cell) => {
        const colindex = Number(cell.getAttribute("aria-colindex"));
        const label = Number.isFinite(colindex) ? enhancedColumns[colindex - 1]?.header : undefined;
        if (label) {
          cell.setAttribute("aria-label", String(label));
        } else {
          cell.removeAttribute("aria-label");
        }
      });
    });
  });

  function rowStyle(row: GridRow) {
    return [
      row.connectable === false ? "is-disabled" : "",
      row.alreadyConnected ? "is-connected" : "",
      row.status ? `status-${row.status}` : "",
    ].filter(Boolean).join(" ");
  }

  function emitSelection() {
    if (!api) return;
    onSelectedRowIdsChange(api.getState().selectedRows.map(String));
  }
</script>

{@html activeRowCss}
<div
  bind:this={host}
  data-grid-uid={gridUid}
  class={cn("extractum-svar-theme extractum-data-grid", className)}
  style={`height:${height};`}
  role={ariaLabel ? "region" : undefined}
  aria-label={ariaLabel}
>
  <Locale words={{ ...coreRu, ...gridEn }}>
    <Willow fonts={false}>
      <Grid
        data={rows}
        columns={enhancedColumns}
        bind:this={api}
        selectedRows={initialSelectedIds}
        {rowStyle}
        columnStyle={columnStyle}
        overlay={visibleOverlay}
        multiselect={multiselect}
        select={selectOnClick}
        responsive={enhancedResponsive}
        sizes={GRID_SIZES}
        onselectrow={emitSelection}
      />
    </Willow>
  </Locale>
</div>

<style>
  .extractum-data-grid {
    min-height: 0;
    min-width: 0;
    width: 100%;
    max-width: 100%;
    overflow: hidden;
  }

  .extractum-data-grid :global(.wx-grid),
  .extractum-data-grid :global(.wx-table-box) {
    height: 100%;
  }

  .extractum-data-grid :global(.wx-cell) {
    padding: 5px 8px;
    font-size: 12.5px;
  }

  .extractum-data-grid :global(.wx-row.is-disabled:not(.wx-selected) .wx-cell) {
    color: var(--extractum-muted);
    background: color-mix(in srgb, var(--extractum-surface-subtle) 80%, transparent);
  }

  .extractum-data-grid :global(.wx-row.is-connected:not(.wx-selected) .wx-cell) {
    background: color-mix(in srgb, var(--extractum-success) 8%, var(--extractum-surface));
  }
</style>
