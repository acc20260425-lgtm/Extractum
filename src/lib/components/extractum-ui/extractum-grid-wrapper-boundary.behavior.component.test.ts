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

afterEach(() => cleanup());

it("SVAR grid APIs stay inside Extractum wrappers", async () => {
  const selected: string[][] = [];
  render(DataGrid, {
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
  expect(JSON.parse(dataGrid.dataset.columns ?? "[]")).toEqual([
    { id: "createdAt", treetoggle: false, hasTemplate: true },
  ]);
  expect(screen.getByLabelText("Formatted grid date").textContent).not.toBe("");
  await fireEvent.click(screen.getByRole("button", { name: "Select first grid row" }));
  expect(selected).toEqual([["row-1"]]);

  cleanup();
  render(DataGrid, { rows: [], columns: [], overlay: "Empty grid" });
  expect(screen.getByTestId("svar-grid").dataset.overlay).toBe("Empty grid");

  cleanup();
  const treeSelected: Array<string | null> = [];
  render(TreeDataGrid, {
    rows: [{ id: "tree-1", label: "Tree" }],
    selectedRowId: "tree-1",
    onSelectedRowIdChange: (id) => treeSelected.push(id),
  });
  const treeGrid = screen.getByTestId("svar-grid");
  expect(screen.getByTestId("svar-locale").dataset.words).toBe(JSON.stringify({ coreWord: "core", gridWord: "grid" }));
  expect(screen.getByTestId("svar-willow").dataset.fonts).toBe("false");
  expect(treeGrid.dataset.tree).toBe("true");
  expect(treeGrid.dataset.select).toBe("true");
  expect(treeGrid.dataset.multiselect).toBe("false");
  expect(JSON.parse(treeGrid.dataset.selected ?? "[]")).toEqual(["tree-1"]);
  expect(JSON.parse(treeGrid.dataset.sizes ?? "{}")).toEqual({ rowHeight: 30, headerHeight: 30, columnWidth: 140 });
  expect(JSON.parse(treeGrid.dataset.columns ?? "[]")[0]).toMatchObject({ id: "label", treetoggle: true });
  await fireEvent.click(screen.getByRole("button", { name: "Select first grid row" }));
  expect(treeSelected).toEqual(["tree-1"]);

  cleanup();
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
