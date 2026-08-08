import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
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

import type { LibraryCatalogWorkflowState } from "$lib/ui/library-catalog-workflow";
import type { LibraryCatalogSourceView } from "$lib/ui/library-catalog-model";
import LibraryScreen from "./LibraryScreen.svelte";

afterEach(cleanup);

function source(overrides: Partial<LibraryCatalogSourceView> = {}): LibraryCatalogSourceView {
  return {
    id: "source:11",
    sourceId: 11,
    provider: "youtube",
    sourceSubtype: "video",
    title: "Evidence video",
    subtitle: "Research channel",
    typeLabel: "YouTube / Video",
    status: "active",
    statusDetail: null,
    projectCount: 1,
    itemCount: 4,
    itemCountLabel: "4 items",
    createdAt: 1_700_000_000,
    lastSyncedAt: 1_700_000_100,
    addedAtLabel: "14 November 2023",
    lastSyncedLabel: "14 November 2023",
    canonicalUrl: "https://youtube.example/watch?v=evidence",
    externalId: "evidence",
    youtube: {
      channel_title: "Research channel",
      video_form: "watch",
      duration_seconds: 120,
      playlist_video_count: null,
      availability_status: "available",
    },
    telegram: null,
    ...overrides,
  };
}

it("library prototype contract > coordinates filter selection, row selection, and Inspector resizing in the screen component", async () => {
  const state: LibraryCatalogWorkflowState = {
    catalogRecords: [],
    filterCounts: [
      { provider: "youtube", source_subtype: "video", count: 2, disabled: false, disabled_reason: null },
      { provider: "telegram", source_subtype: "channel", count: 1, disabled: false, disabled_reason: null },
    ],
    sources: [
      source(),
      source({
        id: "source:12",
        sourceId: 12,
        title: "Second evidence video",
        itemCount: 2,
        itemCountLabel: "2 items",
        canonicalUrl: "https://youtube.example/watch?v=second-evidence",
        externalId: "second-evidence",
      }),
      source({
        id: "source:21",
        sourceId: 21,
        provider: "telegram",
        sourceSubtype: "channel",
        title: "Telegram archive",
        subtitle: "Election monitoring",
        typeLabel: "Telegram / Channel",
        itemCount: 18,
        itemCountLabel: "18 items",
        canonicalUrl: "https://t.me/election-monitoring",
        externalId: "election-monitoring",
        youtube: null,
        telegram: { account_id: 7 },
      }),
    ],
    loading: false,
    status: "",
  };

  render(LibraryScreen, { state, onRefresh: vi.fn() });
  expect(screen.getByRole("complementary", { name: "Library filters" })).toBeTruthy();
  expect(await screen.findByRole("heading", { name: "Evidence video" })).toBeTruthy();

  await fireEvent.click(screen.getByRole("button", { name: "Select grid row provider:telegram" }));
  await waitFor(() => expect(screen.getByRole("heading", { name: "Telegram archive" })).toBeTruthy());

  await fireEvent.click(screen.getByRole("button", { name: "Select grid row provider:youtube" }));
  await waitFor(() => expect(screen.getByRole("heading", { name: "Evidence video" })).toBeTruthy());
  await fireEvent.click(screen.getByRole("button", { name: "Select grid row source:12" }));
  await waitFor(() => expect(screen.getByRole("heading", { name: "Second evidence video" })).toBeTruthy());

  const separator = screen.getByRole("separator", { name: "Resize source inspector" });
  expect(separator.getAttribute("aria-valuenow")).toBe("380");
  await fireEvent.keyDown(separator, { key: "ArrowLeft" });
  expect(separator.getAttribute("aria-valuenow")).toBe("396");
  await fireEvent.keyDown(separator, { key: "ArrowRight" });
  expect(separator.getAttribute("aria-valuenow")).toBe("380");
});
