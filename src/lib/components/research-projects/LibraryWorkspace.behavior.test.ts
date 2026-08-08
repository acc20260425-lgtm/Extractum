import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import { afterEach, expect, it, vi } from "vitest";

vi.mock("@svar-ui/svelte-core", async () => ({ Locale: (await import("$lib/testing/SvarLocaleReceiver.svelte")).default }));
vi.mock("@svar-ui/svelte-grid", async () => ({
  Grid: (await import("$lib/testing/SvarGridReceiver.svelte")).default,
  Willow: (await import("$lib/testing/SvarWillowReceiver.svelte")).default,
}));
vi.mock("@svar-ui/core-locales", () => ({ ru: {} }));
vi.mock("@svar-ui/grid-locales", () => ({ en: {} }));

import type { LibraryCatalogSourceView } from "$lib/ui/library-catalog-model";
import LibraryWorkspace from "./LibraryWorkspace.svelte";

afterEach(cleanup);

it("library prototype contract > renders source CRUD commands and disables selected-source commands without a source", async () => {
  const callbacks = { add: vi.fn(), edit: vi.fn(), delete: vi.fn(), refresh: vi.fn(), select: vi.fn() };
  const selectedSource: LibraryCatalogSourceView = {
    id: "source:11", sourceId: 11, provider: "youtube", sourceSubtype: "video",
    title: "Evidence video", subtitle: "Research channel", typeLabel: "YouTube / Video",
    status: "active", statusDetail: null, projectCount: 1, itemCount: 4, itemCountLabel: "4 items",
    createdAt: 1_700_000_000, lastSyncedAt: 1_700_000_100, addedAtLabel: "14 November 2023",
    lastSyncedLabel: "14 November 2023", canonicalUrl: "https://youtube.example/watch?v=evidence",
    externalId: "evidence", youtube: null, telegram: null,
  };
  const props = {
    sources: [selectedSource], totalSourceCount: 1, query: "", selectedSource: null,
    selectedSourceId: null, onSelectedSourceIdChange: callbacks.select, onAdd: callbacks.add,
    onEdit: callbacks.edit, onDelete: callbacks.delete, onRefresh: callbacks.refresh,
  };
  const view = render(LibraryWorkspace, props);

  expect(screen.getByRole("button", { name: "Add library source" })).toBeTruthy();
  expect((screen.getByRole("button", { name: "Edit selected library source" }) as HTMLButtonElement).disabled).toBe(true);
  expect((screen.getByRole("button", { name: "Delete selected library source" }) as HTMLButtonElement).disabled).toBe(true);
  await fireEvent.click(screen.getByRole("button", { name: "Refresh library source list" }));
  expect(callbacks.refresh).toHaveBeenCalledOnce();

  await view.rerender({ ...props, selectedSource, selectedSourceId: selectedSource.id });
  await fireEvent.click(screen.getByRole("button", { name: "Edit selected library source: Evidence video" }));
  await fireEvent.click(screen.getByRole("button", { name: "Delete selected library source: Evidence video" }));
  expect(callbacks.edit).toHaveBeenCalledOnce();
  expect(callbacks.delete).toHaveBeenCalledOnce();
});
