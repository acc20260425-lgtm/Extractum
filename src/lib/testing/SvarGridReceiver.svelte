<script lang="ts">
  import { onMount, untrack } from "svelte";

  type Row = Record<string, unknown>;
  type CellComponent = new (...args: never[]) => unknown;
  type SortMarks = Record<string, { order: "asc" | "desc"; index?: number }>;
  type Column = Record<string, unknown> & {
    id?: string;
    header?: string | { text?: string; cell?: CellComponent };
    cell?: CellComponent;
    sort?: boolean | ((left: Row, right: Row) => number);
    treetoggle?: boolean;
    template?: (value: unknown) => unknown;
  };

  let {
    data = [], columns = [], selectedRows: initialSelectedRows = [], rowStyle, columnStyle,
    overlay, tree = false, select = false, multiselect = false, sizes, responsive, sortMarks = {}, init, onselectrow,
  } = $props<{
    data?: Row[]; columns?: Column[]; selectedRows?: unknown[]; rowStyle?: (row: Row) => string;
    columnStyle?: (column: Column) => string; overlay?: string; tree?: boolean; select?: boolean;
    multiselect?: boolean; sizes?: Record<string, number>; responsive?: Record<string, unknown>; sortMarks?: SortMarks;
    init?: (api: unknown) => void; onselectrow?: (data: Record<string, unknown>) => void;
  }>();

  let selectedRows = $state(untrack(() => initialSelectedRows.map(String)));
  let gridElement: HTMLElement;
  const interceptors = new Map<string, (event: Record<string, unknown>) => boolean>();
  const dataSubscribers = new Set<(rows: Row[]) => void>();
  const selectionSubscribers = new Set<(ids: string[]) => void>();

  const selectedRowsStore = {
    subscribe(run: (ids: string[]) => void) { run(selectedRows); selectionSubscribers.add(run); return () => selectionSubscribers.delete(run); },
  };
  const dataStore = {
    subscribe(run: (rows: Row[]) => void) { run(data); dataSubscribers.add(run); return () => dataSubscribers.delete(run); },
  };

  export function getState() { return { selectedRows }; }
  export function getReactiveState() { return { selectedRows: selectedRowsStore, data: dataStore }; }
  export function exec(action: string, data: Record<string, unknown>) {
    if (interceptors.get(action)?.(data) === false) return;
    if (action !== "select-row") return;
    const id = String(data.id ?? "");
    if (!id) return;
    const exists = selectedRows.includes(id);
    if (data.toggle === true) {
      if (exists && data.mode !== true) selectedRows = selectedRows.filter((selectedId: string) => selectedId !== id);
      else if (!exists && data.mode !== false) selectedRows = [...selectedRows, id];
    } else {
      selectedRows = [id];
    }
    selectionSubscribers.forEach((run) => run(selectedRows));
    onselectrow?.(data);
  }
  export function intercept(action: string, handler: (event: Record<string, unknown>) => boolean) {
    interceptors.set(action, handler);
  }
  export function getRow(id: string) {
    return data.find((row: Row) => String(row.id) === String(id));
  }

  function headerCellConfig(header: Exclude<Column["header"], string | undefined>) {
    const { cell: _component, ...config } = header;
    return config;
  }

  function selectNativeRow(event: MouseEvent | KeyboardEvent, row: Row) {
    if (!select) return;
    const id = String(row.id);
    if (!multiselect && selectedRows.length === 1 && selectedRows[0] === id) return;
    exec("select-row", { id, toggle: multiselect && (event.ctrlKey || event.metaKey) });
  }

  const api = { getState, getReactiveState, exec, intercept, getRow };
  onMount(() => {
    init?.(api);
    const hostLabel = gridElement.closest("[role='region']")?.getAttribute("aria-label");
    if (hostLabel) gridElement.setAttribute("aria-label", hostLabel);
  });
  $effect(() => {
    const rows = data;
    dataSubscribers.forEach((run) => run(rows));
  });

  const publicResponsive = $derived(responsive ?? {});
  const rowIds = $derived(data.map((row: Row) => String(row.id)));
  const columnIds = $derived(columns.map((column: Column) => String(column.id ?? "")));
  const templateColumnIds = $derived(columns.filter((column: Column) => typeof column.template === "function").map((column: Column) => String(column.id ?? "")));
</script>

<div
  bind:this={gridElement}
  role="grid"
  class="wx-grid"
  data-testid="svar-grid"
  data-selected={JSON.stringify(selectedRows)}
  data-row-ids={JSON.stringify(rowIds)}
  data-column-ids={JSON.stringify(columnIds)}
  data-template-column-ids={JSON.stringify(templateColumnIds)}
  data-data={JSON.stringify(data)}
  data-sort-marks={JSON.stringify(sortMarks)}
  data-row-style={rowStyle?.(data[0] ?? {}) ?? ""}
  data-column-style={JSON.stringify(columns.map((column: Column) => columnStyle?.(column) ?? ""))}
  data-overlay={overlay ?? ""}
  data-tree={String(tree)}
  data-select={String(select)}
  data-multiselect={String(multiselect)}
  data-sizes={JSON.stringify(sizes ?? {})}
  data-responsive={JSON.stringify(publicResponsive)}
>
  {#each columns as column (String(column.id))}
    {#if typeof column.header === "object" && column.header?.cell}
      {@const HeaderCell = column.header.cell}
      <HeaderCell {api} cell={headerCellConfig(column.header)} {column} row={0} onaction={({ action, data }: { action: string; data: Record<string, unknown> }) => exec(action, data)} />
    {/if}
  {/each}
  {#each data as row (String(row.id))}
    <div
      class="wx-row"
      data-id={":" + String(row.id)}
      style={rowStyle?.(row) ?? ""}
      role="row"
      aria-label={String(row.id)}
      tabindex={select ? 0 : undefined}
      onclick={(event) => selectNativeRow(event, row)}
      onkeydown={(event) => (event.key === "Enter" || event.key === " ") && selectNativeRow(event, row)}
    >
      {#each columns as column (String(column.id))}
        {#if column.cell}
          {@const Cell = column.cell}
          <div class="wx-cell">
            <Cell {api} {row} {column} onaction={({ action, data }: { action: string; data: Record<string, unknown> }) => exec(action, data)} />
          </div>
        {/if}
      {/each}
    </div>
  {/each}
  {#if data.length === 0 && overlay}<div>{overlay}</div>{/if}
</div>
