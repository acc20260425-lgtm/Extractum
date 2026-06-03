<script lang="ts">
  import SurfaceCard from "$lib/components/ui/SurfaceCard.svelte";

  type DiagnosticTableValue = string | number;
  type DiagnosticTableRow = Record<string, DiagnosticTableValue>;
  type DiagnosticTableColumn = {
    key: string;
    label: string;
    align?: "start" | "end";
  };

  let {
    title,
    description = "",
    columns,
    rows,
    emptyMessage = "No diagnostic counts reported",
  }: {
    title: string;
    description?: string;
    columns: DiagnosticTableColumn[];
    rows: DiagnosticTableRow[];
    emptyMessage?: string;
  } = $props();

  function cellValue(row: DiagnosticTableRow, key: string) {
    return row[key] ?? "";
  }

  function rowKey(row: DiagnosticTableRow, index: number) {
    const key = columns.map((column) => String(cellValue(row, column.key))).join("|");
    return key || String(index);
  }
</script>

<SurfaceCard {title} meta={description} className="diagnostic-count-table">
  <div class="table-scroll">
    <table>
      <thead>
        <tr>
          {#each columns as column (column.key)}
            <th class:align-end={column.align === "end"}>{column.label}</th>
          {/each}
        </tr>
      </thead>
      <tbody>
        {#each rows as row, index (rowKey(row, index))}
          <tr>
            {#each columns as column (column.key)}
              <td class:align-end={column.align === "end"}>{cellValue(row, column.key)}</td>
            {/each}
          </tr>
        {:else}
          <tr>
            <td class="empty-row" colspan={columns.length}>{emptyMessage}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</SurfaceCard>

<style>
  :global(.diagnostic-count-table.ui-surface-card) {
    gap: 0.7rem;
  }

  .table-scroll {
    overflow-x: auto;
  }

  table {
    width: 100%;
    min-width: 520px;
    border-collapse: collapse;
    font-size: 0.86rem;
  }

  th,
  td {
    padding: 0.55rem 0.45rem;
    border-bottom: 1px solid color-mix(in srgb, var(--border) 78%, transparent);
    text-align: left;
    vertical-align: top;
  }

  th {
    color: var(--muted);
    font-size: 0.72rem;
    font-weight: 700;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  tr:last-child td {
    border-bottom: 0;
  }

  .align-end {
    text-align: right;
  }

  .empty-row {
    color: var(--muted);
    text-align: left;
  }
</style>
