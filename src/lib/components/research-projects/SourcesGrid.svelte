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

  let rows = $derived(buildSourceGridRows(sources));
  const columns = sourceGridColumns().map((column) => {
    const cell = column.id ? CELL_BY_COLUMN[column.id] : undefined;
    return cell ? { ...column, cell } : column;
  });
</script>

<ExtractumDataGrid
  {rows}
  {columns}
  selectedRowIds={selectedSourceIds}
  multiselect={true}
  onSelectedRowIdsChange={onSelectedSourceIdsChange}
  height="100%"
  ariaLabel="Источники проекта"
  overlay="Нет источников"
/>
