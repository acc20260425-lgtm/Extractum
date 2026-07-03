<script lang="ts">
  import { untrack } from "svelte";

  let { api, row } = $props<{
    api: {
      exec: (action: string, data: Record<string, unknown>) => void;
      getReactiveState: () => {
        selectedRows: { subscribe: (fn: (v: unknown[] | undefined) => void) => () => void };
      };
    };
    row: Record<string, unknown>;
  }>();

  // `api` is a stable grid handle; read its reactive store once. Selection is
  // read reactively (not from row.selected) so grid rows need not be rebuilt
  // on selection changes — that would reset svar sorting.
  const selectedRows = untrack(() => api.getReactiveState().selectedRows);

  let rowId = $derived(String(row.id ?? ""));
  let selected = $derived(($selectedRows ?? []).some((id: unknown) => String(id) === rowId));
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
