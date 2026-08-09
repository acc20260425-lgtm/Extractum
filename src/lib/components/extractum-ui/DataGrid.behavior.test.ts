import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";

vi.mock("@svar-ui/svelte-core", async () => ({ Locale: (await import("$lib/testing/SvarLocaleReceiver.svelte")).default }));
vi.mock("@svar-ui/svelte-grid", async () => ({ Grid: (await import("$lib/testing/SvarGridReceiver.svelte")).default, Willow: (await import("$lib/testing/SvarWillowReceiver.svelte")).default }));
vi.mock("@svar-ui/core-locales", () => ({ ru: {} }));
vi.mock("@svar-ui/grid-locales", () => ({ en: {} }));

import DataGrid from "./DataGrid.svelte";
import HeaderPayloadProbe from "$lib/testing/SvarGridHeaderPayloadProbe.svelte";
import SvarGridReceiver from "$lib/testing/SvarGridReceiver.svelte";
afterEach(cleanup);

describe("Extractum DataGrid", () => {
  it("applies responsive column definitions", () => {
    const responsive = { 700: { columns: [{ id: "name" }, { id: "status", hidden: true }] } };
    const receiver = render(SvarGridReceiver, { data: [], columns: [] });
    expect(JSON.parse(receiver.getByRole("grid").dataset.sortMarks ?? "null")).toEqual({});
    receiver.unmount();
    render(DataGrid, { rows: [{ id: "2", name: "Beta" }], columns: [{ id: "name", header: { text: "Name", cell: HeaderPayloadProbe } as never }, { id: "status", header: "Status" }], responsive });
    const grid = screen.getByRole("grid");
    expect(JSON.parse(grid.dataset.responsive ?? "{}")).toEqual(responsive);
    expect(JSON.parse(screen.getByRole("status", { name: "SVAR header payload" }).textContent ?? "{}")).toEqual({ api: true, cell: "Name", cellHasComponent: false, column: "name", row: 0, onaction: true });
  });

  it("preserves sizing and sorting when row height changes", async () => {
    const responsive = { 640: { columns: [{ id: "name" }] } };
    const sortMarks = { name: { order: "asc" as const } };
    const onSelectedRowIdsChange = vi.fn();
    const props = { rows: [{ id: "2", name: "Beta" }, { id: "1", name: "Alpha" }], columns: [{ id: "name", header: "Name", sort: true }], selectedRowIds: ["2", "1"], multiselect: true, rowHeight: 34, responsive, sortMarks, onSelectedRowIdsChange };
    const view = render(DataGrid, props as never);
    const grid = screen.getByRole("grid");
    expect({ sizes: JSON.parse(grid.dataset.sizes ?? "{}"), sortMarks: JSON.parse(grid.dataset.sortMarks ?? "null") }).toEqual({ sizes: { rowHeight: 34, headerHeight: 34, columnWidth: 160 }, sortMarks });
    await view.rerender({ ...props, rowHeight: 52, responsive: { 720: { columns: [{ id: "name" }] } } } as never);
    expect({ sizes: JSON.parse(grid.dataset.sizes ?? "{}"), sortMarks: JSON.parse(grid.dataset.sortMarks ?? "null") }).toEqual({ sizes: { rowHeight: 34, headerHeight: 34, columnWidth: 160 }, sortMarks });
    await fireEvent.click(screen.getByRole("row", { name: "1" }), { ctrlKey: true });
    expect(onSelectedRowIdsChange).toHaveBeenLastCalledWith(["2"]);
    await fireEvent.click(screen.getByRole("row", { name: "1" }));
    expect(onSelectedRowIdsChange).toHaveBeenLastCalledWith(["1"]);
  });
});
