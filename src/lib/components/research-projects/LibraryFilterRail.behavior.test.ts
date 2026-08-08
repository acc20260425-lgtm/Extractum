import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import { afterEach, expect, it, vi } from "vitest";

vi.mock("@svar-ui/svelte-core", async () => ({
  Locale: (await import("$lib/testing/SvarLocaleReceiver.svelte")).default,
}));
vi.mock("@svar-ui/svelte-grid", async () => ({
  Grid: (await import("$lib/testing/SvarGridReceiver.svelte")).default,
  Willow: (await import("$lib/testing/SvarWillowReceiver.svelte")).default,
}));
vi.mock("@svar-ui/core-locales", () => ({ ru: {} }));
vi.mock("@svar-ui/grid-locales", () => ({ en: {} }));

import LibraryFilterRail from "./LibraryFilterRail.svelte";

afterEach(cleanup);

it("library prototype contract > uses the TreeDataGrid wrapper for the collapsible filter rail", async () => {
  const onSelectedFilterIdChange = vi.fn();
  const onCollapsedChange = vi.fn();
  render(LibraryFilterRail, {
    rows: [
      { id: "all", label: "All sources", provider: "all", count: 10 },
      { id: "provider:youtube", label: "YouTube", provider: "youtube", count: 6 },
    ],
    selectedFilterId: "all",
    collapsed: false,
    onSelectedFilterIdChange,
    onCollapsedChange,
  });

  expect(screen.getByRole("complementary", { name: "Library filters" })).toBeTruthy();
  expect(screen.getByTestId("svar-grid").dataset.tree).toBe("true");
  expect(JSON.parse(screen.getByTestId("svar-grid").dataset.selected ?? "[]")).toEqual(["all"]);
  await fireEvent.click(screen.getByRole("button", { name: "Select first grid row" }));
  expect(onSelectedFilterIdChange).toHaveBeenCalledWith("all");
  await fireEvent.click(screen.getByRole("button", { name: "Collapse Library filters" }));
  expect(onCollapsedChange).toHaveBeenCalledWith(true);
});
