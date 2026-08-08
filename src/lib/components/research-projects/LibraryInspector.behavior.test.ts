import { cleanup, render, screen } from "@testing-library/svelte";
import { afterEach, expect, it } from "vitest";
import type { LibraryCatalogSourceView } from "$lib/ui/library-catalog-model";
import LibraryInspector from "./LibraryInspector.svelte";

afterEach(cleanup);

it("library prototype contract > keeps the Inspector bound to selected source context", async () => {
  const selectedSource: LibraryCatalogSourceView = {
    id: "source:7",
    sourceId: 7,
    provider: "youtube",
    sourceSubtype: "video",
    title: "Belarus election briefing",
    subtitle: "Independent channel",
    typeLabel: "YouTube / Video",
    status: "active",
    statusDetail: "Ready",
    projectCount: 2,
    itemCount: 14,
    itemCountLabel: "14 items",
    createdAt: 1_717_243_200,
    lastSyncedAt: 1_717_333_200,
    addedAtLabel: "1 June 2024",
    lastSyncedLabel: "2 June 2024",
    canonicalUrl: "https://youtube.example/watch?v=election-briefing",
    externalId: "election-briefing",
    youtube: {
      channel_title: "Independent Channel",
      video_form: "watch",
      duration_seconds: 95,
      playlist_video_count: null,
      availability_status: "available",
    },
    telegram: null,
  };

  const view = render(LibraryInspector, { selectedSource });
  expect(screen.getByRole("complementary", { name: "Library source inspector" })).toBeTruthy();
  expect(screen.getByRole("heading", { name: "Belarus election briefing" })).toBeTruthy();
  expect(screen.getByText("YouTube / Video")).toBeTruthy();
  expect(screen.getByText("Independent Channel")).toBeTruthy();
  expect(screen.getByText("1m 35s")).toBeTruthy();
  expect(screen.getByRole("link", { name: "https://youtube.example/watch?v=election-briefing" })).toBeTruthy();

  await view.rerender({ selectedSource: null });
  expect(screen.getByRole("heading", { name: "No source selected" })).toBeTruthy();
  expect(screen.queryByText("Belarus election briefing")).toBeNull();
});
