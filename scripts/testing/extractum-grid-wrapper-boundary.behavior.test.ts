import { expect, it, vi } from "vitest";

import * as gridContract from "../../src/lib/components/extractum-ui/data-grid-date-format";

it("SVAR grid APIs stay inside Extractum wrappers", () => {
  expect(gridContract.extractumGridOverlay([], "empty")).toBe("empty");
  expect(gridContract.extractumGridOverlay([{ id: "1" }], "empty")).toBeUndefined();
  expect(gridContract.extractumGridClickIsIgnored({
    closest: (selector: string) => selector === '[data-action="ignore-click"]' ? {} : null,
  })).toBe(true);
  expect(gridContract.extractumGridClickIsIgnored({ closest: () => null })).toBe(false);
  expect(gridContract.extractumGridSelection([1, "2"])).toEqual(["1", "2"]);
  expect(gridContract.extractumTreeGridSelection("tree-1")).toEqual(["tree-1"]);
  const api = { exec: vi.fn() };
  gridContract.executeExtractumGridSelection(api, "row-1", true);
  expect(api.exec).toHaveBeenCalledWith("select-row", {
    id: "row-1",
    mode: true,
    toggle: true,
  });
});
