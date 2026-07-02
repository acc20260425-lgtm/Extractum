<script lang="ts">
  import { untrack } from "svelte";

  // Tri-state "select all" header checkbox for the SVAR grid selection column.
  // Lives in the extractum-ui wrapper layer so it may touch the svar grid api
  // (feature screens must not import svar/bits-ui directly).
  let { api } = $props<{
    api: {
      exec: (action: string, data: Record<string, unknown>) => void;
      getReactiveState: () => {
        selectedRows: { subscribe: (fn: (v: unknown[]) => void) => () => void };
        data: { subscribe: (fn: (v: unknown[]) => void) => () => void };
      };
    };
  }>();

  // `api` is a stable grid handle; read its reactive stores once.
  const reactiveState = untrack(() => api.getReactiveState());
  const selectedRows = reactiveState.selectedRows;
  const data = reactiveState.data;

  let selectedCount = $derived($selectedRows?.length ?? 0);
  let total = $derived(($data ?? []).length);
  let checked = $derived(total > 0 && selectedCount === total);
  let indeterminate = $derived(selectedCount > 0 && selectedCount < total);

  let el = $state<HTMLInputElement | null>(null);
  $effect(() => {
    if (el) el.indeterminate = indeterminate;
  });

  function toggleAll(event: Event) {
    const value = (event.currentTarget as HTMLInputElement).checked;
    for (const row of ($data ?? []) as Array<{ id: unknown }>) {
      api.exec("select-row", { id: String(row.id ?? ""), mode: value, toggle: true });
    }
  }
</script>

<div class="extractum-grid-select-cell" data-action="ignore-click">
  <input
    bind:this={el}
    type="checkbox"
    checked={checked}
    aria-label="Выбрать все источники"
    onchange={toggleAll}
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
