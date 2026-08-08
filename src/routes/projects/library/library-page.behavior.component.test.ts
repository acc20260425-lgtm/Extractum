import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { afterEach, describe, expect, it, vi } from "vitest";

const listLibraryCatalog = vi.hoisted(() => vi.fn());
vi.mock("$lib/api/library-sources", () => ({ listLibraryCatalog }));
vi.mock("$lib/components/research-projects/LibraryScreen.svelte", async () => ({
  default: (await import("$lib/testing/LibraryScreenReceiver.svelte")).default,
}));

import LibraryPage from "./+page.svelte";

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

describe("library prototype contract", () => {
  it("renders Library as a separate route backed by the current workflow", async () => {
    listLibraryCatalog.mockResolvedValue({
      sources: [{
        source: {
          source_id: 7,
          provider: "youtube",
          source_subtype: "video",
          account_id: null,
          external_id: "video-7",
          title: "Library source",
          subtitle: "Channel",
          canonical_url: "https://www.youtube.com/watch?v=video-7",
          created_at: 1_700_000_000,
          last_synced_at: null,
          item_count: 1,
          project_count: 0,
          youtube: {
            video_form: "watch",
            duration_seconds: 90,
            playlist_video_count: null,
            channel_title: "Channel",
            availability_status: "available",
          },
          telegram: null,
        },
        latest_job: null,
        status: "active",
        status_detail: null,
        capabilities: {
          can_refresh_source: true,
          can_delete: true,
          can_edit: true,
          can_connect_to_project: true,
        },
        disabled_reasons: {
          refresh_source: null,
          delete: null,
          edit: null,
          connect_to_project: null,
        },
      }],
      filter_counts: [],
    });
    const view = render(LibraryPage);

    expect(view.container.querySelector('[data-ui-route="library-prototype"]')).not.toBeNull();
    expect(view.container.querySelector('[data-route-href="/projects/library"]')).not.toBeNull();
    expect(screen.getByRole("heading", { name: "Library" })).not.toBeNull();
    await waitFor(() => expect(listLibraryCatalog).toHaveBeenCalledOnce());
    await waitFor(() => expect(screen.getByLabelText("Library source count").textContent).toBe("1"));

    await fireEvent.click(screen.getByRole("button", { name: "Refresh Library" }));
    await waitFor(() => expect(listLibraryCatalog).toHaveBeenCalledTimes(2));
  });
});
