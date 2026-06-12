<script lang="ts">
  let { api, row } = $props<{
    api: { exec: (action: string, data: Record<string, unknown>) => void };
    row: Record<string, unknown>;
  }>();

  let rowId = $derived(String(row.id ?? ""));
  let selected = $derived(Boolean(row.selected));
  let connectable = $derived(row.connectable !== false);
  let disabledReason = $derived(
    typeof row.disabledReason === "string" ? row.disabledReason : null,
  );

  function toggle(event: Event) {
    if (!connectable) return;
    const target = event.currentTarget as HTMLInputElement;
    api.exec("select-row", { id: rowId, mode: target.checked, toggle: true });
  }
</script>

<div class="extractum-grid-select-cell" data-action="ignore-click" title={disabledReason ?? undefined}>
  <input
    type="checkbox"
    disabled={!connectable}
    checked={selected}
    aria-label="Выбрать источник"
    onchange={toggle}
  />
</div>

<style>
  .extractum-grid-select-cell {
    display: grid;
    height: 100%;
    place-items: center;
  }

  .extractum-grid-select-cell input {
    width: 14px;
    height: 14px;
    margin: 0;
    accent-color: var(--extractum-primary);
  }
</style>
