<script lang="ts">
  import { ExtractumDataGrid } from "$lib/components/extractum-ui";
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

  let {
    sources,
    selectedSourceIds = [],
    onSelectedSourceIdsChange = () => {},
  }: {
    sources: ProjectSourceRecord[];
    selectedSourceIds?: string[];
    onSelectedSourceIdsChange?: (ids: string[]) => void;
  } = $props();

  const RIGHT_ALIGNED = new Set(["materialsLabel"]);

  let rows = $derived(buildSourceGridRows(sources));
  const columns = sourceGridColumns().map((column) => {
    const cell = column.id ? CELL_BY_COLUMN[column.id] : undefined;
    return cell ? { ...column, cell } : column;
  });

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
  overlay="Нет источников"
/>

<style>
  /* svar has no column `align`; right-align numeric cells via columnStyle
     (returns a global class name). */
  :global(.extractum-grid-cell-right) {
    text-align: right;
  }
</style>
