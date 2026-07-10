<script lang="ts">
  import { tick } from "svelte";
  import {
    ExtractumDataGrid,
    GridSelectCell,
    GridSelectAllCell,
  } from "$lib/components/extractum-ui";
  import type { ExtractumDataGridColumn } from "$lib/components/extractum-ui";
  import {
    SOURCE_TABLE_LAYOUT,
    buildSourceGridRows,
    sourceGridColumns,
    sourceGridResponsiveColumns,
  } from "$lib/ui/research-projects-source-row";
  import {
    isSourceKeyboardEditableTarget,
    sourceGridRowIdsFromElement,
    sourceKeyboardCommand,
  } from "$lib/ui/research-projects-source-keyboard";
  import type { ProjectSourceRecord } from "$lib/types/projects";
  import SourceStatusCell from "./SourceStatusCell.svelte";
  import SourceTitleCell from "./SourceTitleCell.svelte";

  const CELL_BY_COLUMN: Record<string, ExtractumDataGridColumn["cell"]> = {
    title: SourceTitleCell,
    statusLabel: SourceStatusCell,
  };

  // Leading checkbox column: per-row select via GridSelectCell, tri-state
  // "select all" in the header via GridSelectAllCell (svar has no native
  // select-all column).
  const SELECT_COLUMN: ExtractumDataGridColumn = {
    id: "selected",
    // Svar typing for header/cell components is stricter than a plain
    // Component<{api,row}>; the runtime only needs `api`/`row`, so cast both.
    header: { cell: GridSelectAllCell } as unknown as ExtractumDataGridColumn["header"],
    width: SOURCE_TABLE_LAYOUT.select,
    cell: GridSelectCell as unknown as ExtractumDataGridColumn["cell"],
  };

  let {
    sources,
    selectedSourceIds = [],
    onSelectedSourceIdsChange = () => {},
    overlay = "Нет источников",
    activeSourceId = null,
    onActivateSource,
    keyboardNavigationEnabled = false,
    onKeyboardActivateSource,
    onKeyboardInspectSource,
    onKeyboardEscape,
  }: {
    sources: ProjectSourceRecord[];
    selectedSourceIds?: string[];
    onSelectedSourceIdsChange?: (ids: string[]) => void;
    overlay?: string;
    activeSourceId?: string | null;
    onActivateSource?: (id: string) => void;
    keyboardNavigationEnabled?: boolean;
    onKeyboardActivateSource?: (id: string) => void;
    onKeyboardInspectSource?: (id: string) => void;
    onKeyboardEscape?: () => boolean;
  } = $props();

  const RIGHT_ALIGNED = new Set(["materialsLabel"]);
  let host = $state<HTMLDivElement | null>(null);

  // Rows depend only on the data: GridSelectCell reads selection reactively
  // from the grid api, so toggling checkboxes does not rebuild rows (and does
  // not reset svar sorting).
  let rows = $derived(buildSourceGridRows(sources));
  function attachSourceCells(columns: ExtractumDataGridColumn[]): ExtractumDataGridColumn[] {
    return columns.map((column) => {
      const cell = column.id ? CELL_BY_COLUMN[column.id] : undefined;
      return cell ? { ...column, cell } : column;
    });
  }

  const columns = [{ ...SELECT_COLUMN }, ...attachSourceCells(sourceGridColumns())];
  const responsiveColumns = Object.fromEntries(
    Object.entries(sourceGridResponsiveColumns()).map(([breakpoint, config]) => [
      breakpoint,
      {
        ...config,
        columns: [{ ...SELECT_COLUMN }, ...attachSourceCells(config.columns)],
      },
    ]),
  );

  function columnStyle(column: ExtractumDataGridColumn): string {
    return column.id != null && RIGHT_ALIGNED.has(String(column.id))
      ? "extractum-grid-cell-right"
      : "";
  }

  function sourceRowSelector(sourceId: string): string {
    const escaped =
      typeof CSS !== "undefined" && CSS.escape
        ? CSS.escape(sourceId)
        : sourceId.replace(/["\\]/g, "\\$&");
    return `.wx-row[data-id=":${escaped}"], .wx-row[data-id="${escaped}"]`;
  }

  function scrollActiveSourceRowIntoView(sourceId: string) {
    void tick().then(() => {
      const row = host?.querySelector<HTMLElement>(sourceRowSelector(sourceId));
      const scrollBox = row?.closest<HTMLElement>(".wx-scroll, .wx-table-box");
      if (!row || !scrollBox) return;
      const rowTop = row.offsetTop;
      const rowBottom = rowTop + row.offsetHeight;
      const viewportTop = scrollBox.scrollTop;
      const viewportBottom = viewportTop + scrollBox.clientHeight;
      if (rowTop < viewportTop) {
        scrollBox.scrollTop = rowTop;
      } else if (rowBottom > viewportBottom) {
        scrollBox.scrollTop = rowBottom - scrollBox.clientHeight;
      }
    });
  }

  function handleDocumentKeydown(event: KeyboardEvent) {
    if (!keyboardNavigationEnabled || isSourceKeyboardEditableTarget(event.target)) return;
    const command = sourceKeyboardCommand({
      key: event.key,
      orderedSourceIds: sourceGridRowIdsFromElement(host),
      activeSourceId,
      selectedSourceIds,
    });
    if (!command.handled) return;

    if (command.kind === "escape") {
      if (onKeyboardEscape?.()) {
        event.preventDefault();
        event.stopPropagation();
      }
      return;
    }

    event.preventDefault();
    event.stopPropagation();
    if (command.kind === "activate") {
      (onKeyboardActivateSource ?? onActivateSource)?.(command.sourceId);
      scrollActiveSourceRowIntoView(command.sourceId);
    } else if (command.kind === "inspect") {
      onKeyboardInspectSource?.(command.sourceId);
      scrollActiveSourceRowIntoView(command.sourceId);
    } else if (command.kind === "toggleSelection") {
      onSelectedSourceIdsChange(command.selectedSourceIds);
    }
  }
</script>

<svelte:document onkeydown={handleDocumentKeydown} />

<div bind:this={host} class="sources-grid">
  <ExtractumDataGrid
    class="sources-grid__table"
    {rows}
    {columns}
    {columnStyle}
    responsive={responsiveColumns}
    selectedRowIds={selectedSourceIds}
    multiselect={true}
    onSelectedRowIdsChange={onSelectedSourceIdsChange}
    height="100%"
    ariaLabel="Источники проекта"
    {overlay}
    selectOnClick={false}
    activeRowId={activeSourceId}
    rowHeight={44}
    onRowClick={onActivateSource}
  />
</div>

<style>
  .sources-grid {
    height: 100%;
    min-height: 0;
  }

  /* svar has no column `align`; right-align numeric cells via columnStyle
     (returns a global class name). */
  :global(.extractum-grid-cell-right) {
    text-align: right;
  }

  :global(.sources-grid__table .wx-header .wx-cell) {
    font-weight: 700;
    color: var(--extractum-text);
    background: var(--extractum-surface-subtle);
    position: relative;
  }

  :global(.sources-grid__table .wx-header .wx-cell:has(.wx-sort)) {
    cursor: pointer;
  }

  /* призрачная стрелка-подсказка: видна только на hover и только пока колонка
     НЕ отсортирована. В svar-шапке у сортируемых колонок есть пустой `.wx-sort`,
     у несортированных aria-sort="none" (у отсортированной — ascending/descending
     плюс собственный индикатор в `.wx-sort`). */
  :global(.sources-grid__table .wx-header .wx-cell[aria-sort="none"]:has(.wx-sort)::after) {
    content: "";
    position: absolute;
    right: 10px;
    top: 50%;
    width: 7px;
    height: 7px;
    margin-top: -4px;
    border-right: 1.5px solid var(--extractum-muted);
    border-bottom: 1.5px solid var(--extractum-muted);
    transform: rotate(45deg);
    opacity: 0;
    transition: opacity .12s ease;
  }
  :global(.sources-grid__table .wx-header .wx-cell[aria-sort="none"]:has(.wx-sort):hover::after) {
    opacity: .4;
  }

  :global(.sources-grid__table .wx-row .wx-cell) {
    border-color: color-mix(in srgb, var(--extractum-border) 72%, transparent);
    /* the 44px row is taller than one line of text; align-content centers
       block-level cell content vertically without breaking text-overflow
       ellipsis and text-align:right the way display:flex would. */
    align-content: center;
  }
</style>
