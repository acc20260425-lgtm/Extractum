import { cleanup, fireEvent, render, screen, within } from "@testing-library/svelte";
import { afterEach, expect, it, vi } from "vitest";
import LibraryYoutubeAddPanel from "./LibraryYoutubeAddPanel.svelte";

afterEach(cleanup);

it("library add source contract > keeps YouTube mode tabs inside the YouTube panel", async () => {
  render(LibraryYoutubeAddPanel, {
    sources: [],
    onSourcesChanged: vi.fn(),
    onStatus: vi.fn(),
  });

  const panel = screen.getByRole("region", { name: "YouTube Add Source" });
  const tabs = within(panel).getByRole("tablist", { name: "YouTube import modes" });
  const smart = within(tabs).getByRole("tab", { name: "Smart import" });
  const existing = within(tabs).getByRole("tab", { name: "From existing data" });
  expect(smart.getAttribute("aria-selected")).toBe("true");
  await fireEvent.click(existing);
  expect(existing.getAttribute("aria-selected")).toBe("true");
  const existingPanel = within(panel).getByRole("tabpanel", { name: "From existing data" });
  expect(existingPanel.hidden).toBe(false);
});
