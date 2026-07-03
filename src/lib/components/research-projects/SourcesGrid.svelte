<script lang="ts">
  import {
    ExtractumDataGrid,
    GridSelectCell,
    GridSelectAllCell,
  } from "$lib/components/extractum-ui";
  import type { ExtractumDataGridColumn } from "$lib/components/extractum-ui";
  import {
    buildSourceGridRows,
    sourceGridColumns,
  } from "$lib/ui/research-projects-source-row";
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
    width: 34,
    cell: GridSelectCell as unknown as ExtractumDataGridColumn["cell"],
  };

  let {
    sources,
    selectedSourceIds = [],
    onSelectedSourceIdsChange = () => {},
    overlay = "Нет источников",
    activeSourceId = null,
    onActivateSource,
  }: {
    sources: ProjectSourceRecord[];
    selectedSourceIds?: string[];
    onSelectedSourceIdsChange?: (ids: string[]) => void;
    overlay?: string;
    activeSourceId?: string | null;
    onActivateSource?: (id: string) => void;
  } = $props();

  const RIGHT_ALIGNED = new Set(["materialsLabel"]);

  // Rows depend only on the data: GridSelectCell reads selection reactively
  // from the grid api, so toggling checkboxes does not rebuild rows (and does
  // not reset svar sorting).
  let rows = $derived(buildSourceGridRows(sources));
  const columns = [
    SELECT_COLUMN,
    ...sourceGridColumns().map((column) => {
      const cell = column.id ? CELL_BY_COLUMN[column.id] : undefined;
      return cell ? { ...column, cell } : column;
    }),
  ];

  function columnStyle(column: ExtractumDataGridColumn): string {
    return column.id != null && RIGHT_ALIGNED.has(String(column.id))
      ? "extractum-grid-cell-right"
      : "";
  }
</script>

<ExtractumDataGrid
  {rows}
  {columns}
  {columnStyle}
  selectedRowIds={selectedSourceIds}
  multiselect={true}
  onSelectedRowIdsChange={onSelectedSourceIdsChange}
  height="100%"
  ariaLabel="Источники проекта"
  {overlay}
  selectOnClick={false}
  activeRowId={activeSourceId}
  onRowClick={onActivateSource}
/>

<style>
  /* svar has no column `align`; right-align numeric cells via columnStyle
     (returns a global class name). */
  :global(.extractum-grid-cell-right) {
    text-align: right;
  }
</style>
