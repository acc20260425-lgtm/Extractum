<script lang="ts">
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
    totalRows = rows.length,
    emptyMessage = "No diagnostic counts reported",
    open = true,
  }: {
    title: string;
    description?: string;
    columns: DiagnosticTableColumn[];
    rows: DiagnosticTableRow[];
    totalRows?: number;
    emptyMessage?: string;
    open?: boolean;
  } = $props();

  const rowText = $derived(
    rows.length === totalRows ? `${rows.length} rows` : `${rows.length}/${totalRows} rows`,
  );

  function cellValue(row: DiagnosticTableRow, key: string) {
    return row[key] ?? "";
  }

  function rowKey(row: DiagnosticTableRow, index: number) {
    const key = columns.map((column) => String(cellValue(row, column.key))).join("|");
    return key || String(index);
  }
</script>

<section class="extractum-panel-shell diagnostic-count-table">
  <details class="diagnostic-count-details" {open} aria-label={`Expand ${title} diagnostics section`}>
    <summary>
      <span>{title}</span>
      <span class="muted-copy">{rowText}</span>
    </summary>
    {#if description}
      <p>{description}</p>
    {/if}
    <div class="extractum-grid-frame table-scroll">
      <table class="diagnostic-count-table-grid" aria-label={`Diagnostic counts for ${title}`}>
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
  </details>
</section>

<style>
  :global(.diagnostic-count-table.extractum-panel-shell) {
    padding-bottom: 0.5rem;
    gap: 0.7rem;
  }

  .diagnostic-count-details {
    display: flex;
    flex-direction: column;
    gap: 0.65rem;
  }

  summary {
    display: flex;
    justify-content: space-between;
    gap: 1rem;
    cursor: pointer;
    color: var(--text);
    font-weight: 650;
  }

  .diagnostic-count-details .muted-copy {
    margin: 0;
    font-size: 0.8rem;
    font-weight: 500;
  }

  p {
    margin: 0;
    color: var(--muted);
    font-size: 0.86rem;
    line-height: 1.45;
  }

  .table-scroll {
    overflow-x: auto;
  }

  .diagnostic-count-table-grid {
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
