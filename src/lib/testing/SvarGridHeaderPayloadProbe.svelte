<script lang="ts">
  let { api, cell, column, row, onaction } = $props<{
    api?: { exec?: unknown };
    cell?: { text?: string; cell?: unknown };
    column?: { id?: string };
    row?: number;
    onaction?: unknown;
  }>();

  const payload = $derived({
    api: typeof api?.exec === "function",
    cell: cell?.text ?? null,
    cellHasComponent: cell ? "cell" in cell : null,
    column: column?.id ?? null,
    row: row ?? null,
    onaction: typeof onaction === "function",
  });
</script>

<span role="status" aria-label="SVAR header payload">{JSON.stringify(payload)}</span>
