import { expect, it, vi } from "vitest";

import * as gridContract from "../../src/lib/components/extractum-ui/data-grid-date-format";

it("SVAR grid APIs stay inside Extractum wrappers", () => {
  expect(gridContract.extractumGridLocaleWords({ core: true }, { grid: true })).toEqual({
    core: true,
    grid: true,
  });
  expect(gridContract.EXTRACTUM_GRID_WILLOW_PROPS).toEqual({ fonts: false });
  expect(gridContract.extractumGridOverlay([], "empty")).toBe("empty");
  expect(gridContract.extractumGridOverlay([{ id: "1" }], "empty")).toBeUndefined();
  expect(gridContract.extractumGridClickIsIgnored({
    closest: (selector: string) => selector === '[data-action="ignore-click"]' ? {} : null,
  })).toBe(true);
  expect(gridContract.extractumGridClickIsIgnored({ closest: () => null })).toBe(false);
  expect(gridContract.extractumGridSelection([1, "2"])).toEqual(["1", "2"]);
  expect(gridContract.extractumTreeGridSelection("tree-1")).toEqual(["tree-1"]);
  expect(gridContract.extractumTreeGridRuntime("tree-1", [], "empty", false)).toEqual({
    selectedRows: ["tree-1"],
    overlay: "empty",
    tree: true,
    select: true,
    multiselect: false,
    sizes: { rowHeight: 30, headerHeight: 30, columnWidth: 140 },
  });
  expect(gridContract.extractumDataGridRuntime([1, "2"], [], "empty")).toEqual({
    selectedRows: ["1", "2"],
    overlay: "empty",
  });
  const api = { exec: vi.fn() };
  gridContract.executeExtractumGridSelection(api, "row-1", true);
  expect(api.exec).toHaveBeenCalledWith("select-row", {
    id: "row-1",
    mode: true,
    toggle: true,
  });
});
