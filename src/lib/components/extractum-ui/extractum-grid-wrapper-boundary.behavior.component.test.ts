import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import { afterEach, expect, it, vi } from "vitest";

vi.mock("@svar-ui/svelte-core", async () => ({
  Locale: (await import("$lib/testing/SvarLocaleReceiver.svelte")).default,
}));
vi.mock("@svar-ui/svelte-grid", async () => ({
  Grid: (await import("$lib/testing/SvarGridReceiver.svelte")).default,
  Willow: (await import("$lib/testing/SvarWillowReceiver.svelte")).default,
}));
vi.mock("@svar-ui/core-locales", () => ({ ru: { coreWord: "core" } }));
vi.mock("@svar-ui/grid-locales", () => ({ en: { gridWord: "grid" } }));

import DataGrid from "./DataGrid.svelte";
import GridSelectCell from "./GridSelectCell.svelte";
import TreeDataGrid from "./TreeDataGrid.svelte";

afterEach(cleanup);

it("SVAR grid APIs stay inside Extractum wrappers", async () => {
  const selected: string[][] = [];
  const dataView = render(DataGrid, {
    rows: [{ id: "row-1", connectable: false, alreadyConnected: true, status: "ready", createdAt: "2026-08-08T12:34:00Z" }],
    columns: [{ id: "createdAt", header: "Created", dateTimeFormat: "datetime" }],
    selectedRowIds: ["1"],
    onSelectedRowIdsChange: (ids) => selected.push(ids),
  });

  const dataLocale = screen.getByTestId("svar-locale");
  const dataWillow = screen.getByTestId("svar-willow");
  const dataGrid = screen.getByTestId("svar-grid");
  expect(JSON.parse(dataLocale.dataset.words ?? "{}")).toEqual({ coreWord: "core", gridWord: "grid" });
  expect(dataWillow.dataset.fonts).toBe("false");
  expect(JSON.parse(dataGrid.dataset.selected ?? "[]")).toEqual(["1"]);
  expect(dataGrid.dataset.rowStyle).toBe("is-disabled is-connected status-ready");
  expect(dataGrid.dataset.overlay).toBe("");
  expect(JSON.parse(dataGrid.dataset.columnIds ?? "[]")).toEqual(["createdAt"]);
  expect(JSON.parse(dataGrid.dataset.data ?? "[]")[0]).toMatchObject({ id: "row-1", createdAt: "2026-08-08T12:34:00Z" });
  await fireEvent.click(screen.getByRole("row", { name: "row-1" }));
  expect(selected).toEqual([["row-1"]]);

  dataView.unmount();
  const emptyView = render(DataGrid, { rows: [], columns: [], overlay: "Empty grid" });
  expect(screen.getByTestId("svar-grid").dataset.overlay).toBe("Empty grid");

  emptyView.unmount();
  const treeSelected: Array<string | null> = [];
  const treeView = render(TreeDataGrid, {
    rows: [{ id: "tree-1", label: "Tree" }, { id: "tree-disabled", label: "Disabled", disabled: true }, { id: "tree-2", label: "Other" }],
    selectedRowId: "tree-1",
    onSelectedRowIdChange: (id) => treeSelected.push(id),
  });
  const treeGrid = screen.getByTestId("svar-grid");
  expect(screen.getByTestId("svar-locale").dataset.words).toBe(JSON.stringify({ coreWord: "core", gridWord: "grid" }));
  expect(screen.getByTestId("svar-willow").dataset.fonts).toBe("false");
  expect({ tree: treeGrid.dataset.tree, select: treeGrid.dataset.select, multiselect: treeGrid.dataset.multiselect }).toEqual({ tree: "true", select: "true", multiselect: "false" });
  expect(JSON.parse(treeGrid.dataset.selected ?? "[]")).toEqual(["tree-1"]);
  expect(JSON.parse(treeGrid.dataset.sizes ?? "{}")).toEqual({ rowHeight: 30, headerHeight: 30, columnWidth: 140 });
  expect(JSON.parse(treeGrid.dataset.columnIds ?? "[]")).toEqual(["label", "count"]);
  expect(JSON.parse(treeGrid.dataset.rowIds ?? "[]")).toEqual(["tree-1", "tree-disabled", "tree-2"]);
  await fireEvent.click(screen.getByRole("row", { name: "tree-1" }));
  await fireEvent.click(screen.getByRole("row", { name: "tree-disabled" }));
  expect({ selected: JSON.parse(treeGrid.dataset.selected ?? "[]"), callbacks: treeSelected }).toEqual({ selected: ["tree-1"], callbacks: [] });
  await fireEvent.click(screen.getByRole("row", { name: "tree-2" }));
  expect({ selected: JSON.parse(treeGrid.dataset.selected ?? "[]"), callbacks: treeSelected }).toEqual({ selected: ["tree-2"], callbacks: ["tree-2"] });

  treeView.unmount();
  const api = {
    exec: vi.fn(),
    getReactiveState: () => ({ selectedRows: { subscribe: (fn: (ids: string[]) => void) => {
      fn([]);
      return () => {};
    } } }),
  };
  render(GridSelectCell, { api, row: { id: "row-9", connectable: true } });
  const checkbox = screen.getByRole("checkbox", { name: "Выбрать источник" });
  expect(checkbox.closest('[data-action="ignore-click"]')).not.toBeNull();
  await fireEvent.click(checkbox);
  expect(api.exec).toHaveBeenCalledWith("select-row", { id: "row-9", mode: true, toggle: true });
});
