<script lang="ts">
  let { api, row } = $props<{
    api: { exec: (action: string, data: Record<string, unknown>) => void };
    row: { id: string; selected?: boolean; connectable?: boolean; disabledReason?: string | null };
  }>();

  function toggle(event: Event) {
    if (row.connectable === false) return;
    const target = event.currentTarget as HTMLInputElement;
    api.exec("select-row", { id: row.id, mode: target.checked, toggle: true });
  }
</script>

<div class="extractum-grid-select-cell" data-action="ignore-click" title={row.disabledReason ?? undefined}>
  <input
    type="checkbox"
    disabled={row.connectable === false}
    checked={!!row.selected}
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
