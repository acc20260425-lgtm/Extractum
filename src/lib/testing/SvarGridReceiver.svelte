<script lang="ts">
  import { onMount, untrack } from "svelte";
  type Row = Record<string, unknown>;
  type Column = Record<string, unknown> & { id?: string; treetoggle?: boolean; template?: (value: unknown) => unknown };

  let {
    data = [],
    columns = [],
    selectedRows: initialSelectedRows = [],
    rowStyle,
    overlay,
    tree = false,
    select = false,
    multiselect = false,
    sizes,
    init,
    onselectrow,
  } = $props<{
    data?: Row[];
    columns?: Column[];
    selectedRows?: unknown[];
    rowStyle?: (row: Row) => string;
    overlay?: string;
    tree?: boolean;
    select?: boolean;
    multiselect?: boolean;
    sizes?: Record<string, number>;
    init?: (api: unknown) => void;
    onselectrow?: () => void;
  }>();

  let selectedRows = $state(untrack(() => initialSelectedRows.map(String)));
  let executed = $state("");
  const subscribers = new Set<(ids: string[]) => void>();
  const interceptors = new Map<string, (event: { id?: string }) => boolean>();

  export function getState() {
    return { selectedRows };
  }

  export function getReactiveState() {
    return {
      selectedRows: {
        subscribe(fn: (ids: string[]) => void) {
          fn(selectedRows);
          subscribers.add(fn);
          return () => subscribers.delete(fn);
        },
      },
    };
  }

  export function exec(action: string, payload: Record<string, unknown>) {
    executed = `${action}:${String(payload.id)}:${String(payload.mode)}`;
    if (action === "select-row") {
      const id = String(payload.id);
      selectedRows = payload.mode === false
        ? selectedRows.filter((value: string) => value !== id)
        : [...new Set([...selectedRows, id])];
      subscribers.forEach((subscriber) => subscriber(selectedRows));
    }
  }

  export function intercept(action: string, handler: (event: { id?: string }) => boolean) {
    interceptors.set(action, handler);
  }

  export function getRow(id: string) {
    return data.find((row: Row) => String(row.id) === String(id));
  }

  const api = { getState, getReactiveState, exec, intercept, getRow };
  onMount(() => init?.(api));

  function selectFirstRow() {
    const id = String(data[0]?.id ?? "");
    if (!id || interceptors.get("select-row")?.({ id }) === false) return;
    selectedRows = [id];
    subscribers.forEach((subscriber) => subscriber(selectedRows));
    onselectrow?.();
  }

  let columnShape = $derived(columns.map((column: Column) => ({
    id: column.id,
    treetoggle: column.treetoggle === true,
    hasTemplate: typeof column.template === "function",
  })));
  let formattedDate = $derived.by(() => {
    const column = columns.find((candidate: Column) => typeof candidate.template === "function");
    return column?.template?.(data[0]?.[String(column.id)]) ?? "";
  });
</script>

<div
  data-testid="svar-grid"
  data-selected={JSON.stringify(selectedRows)}
  data-row-style={rowStyle?.(data[0] ?? {}) ?? ""}
  data-overlay={overlay ?? ""}
  data-tree={String(tree)}
  data-select={String(select)}
  data-multiselect={String(multiselect)}
  data-sizes={JSON.stringify(sizes ?? {})}
  data-columns={JSON.stringify(columnShape)}
>
  <output aria-label="Formatted grid date">{formattedDate}</output>
  <output aria-label="Executed grid action">{executed}</output>
  <button type="button" onclick={selectFirstRow}>Select first grid row</button>
</div>
