<script lang="ts">
  import { ExtractumDataGrid } from "$lib/components/extractum-ui";
  import {
    buildSourceGridRows,
    sourceGridColumns,
  } from "$lib/ui/research-projects-source-row";
  import type { ProjectSourceRecord } from "$lib/types/projects";

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
  const columns = sourceGridColumns();
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
