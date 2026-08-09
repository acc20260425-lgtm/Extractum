import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/svelte";
import "$lib/testing/dom-assertions";
const api = vi.hoisted(() => ({ previewYoutubeSource: vi.fn(), addYoutubeSource: vi.fn() }));
vi.mock("$lib/api/accounts", () => ({ listAccounts: vi.fn().mockResolvedValue([]), getAccountRuntimeStatuses: vi.fn().mockResolvedValue([]) }));
vi.mock("$lib/api/sources", () => ({ previewYoutubeSource: api.previewYoutubeSource, addYoutubeSource: api.addYoutubeSource, listTelegramSources: vi.fn(), addTelegramSource: vi.fn() }));
vi.mock("$lib/api/youtube-detail", () => ({ getYoutubePlaylistDetail: vi.fn() }));
import LibraryAddSourceDialog from "./LibraryAddSourceDialog.svelte";
afterEach(cleanup);
const existingSource = {
  id: "source:11", sourceId: 11, provider: "youtube", sourceSubtype: "video", typeLabel: "YouTube / Video",
  title: "Evidence video", subtitle: "Research", projectCount: 0, itemCount: 0, itemCountLabel: "0 items",
  status: "active", statusDetail: null, createdAt: null, lastSyncedAt: null, addedAtLabel: "Today",
  lastSyncedLabel: "Never", canonicalUrl: "https://youtu.be/video-11", externalId: "video-11",
  youtube: { video_form: "standard", duration_seconds: 120, playlist_video_count: null, channel_title: "Research", availability_status: "available" },
  telegram: null,
};
describe("library add source contract", () => {
  it("keeps the standalone scalar onSourcesChanged contract while accepting project context", async () => {
    const changed = vi.fn(); const project = { projectId: 5, connectedSourceIds: new Set<number>(), onConnectExistingSource: vi.fn(), onConnectAddedSources: vi.fn() };
    render(LibraryAddSourceDialog, { open: true, sources: [], onSourcesChanged: changed, onStatus: vi.fn(), projectContext: project });
    const dialog = screen.getByRole("dialog", { name: "Add source" });
    expect(dialog).toBeTruthy();
    expect(within(dialog).getByRole("tablist", { name: "Source providers" })).toBeTruthy();
    expect(within(dialog).getByRole("tab", { name: "YouTube" }).getAttribute("aria-selected")).toBe("true");
    expect(within(dialog).getByRole("region", { name: "YouTube smart import" })).toBeTruthy();
    expect(changed).not.toHaveBeenCalled();
    expect(project.onConnectAddedSources).not.toHaveBeenCalled();
  });
  it("passes project context through the YouTube add-source tree", async () => {
    const project = { projectId: 5, connectedSourceIds: new Set<number>(), onConnectExistingSource: vi.fn(), onConnectAddedSources: vi.fn() };
    api.previewYoutubeSource.mockResolvedValue({ kind: "video", externalId: "video-11", canonicalUrl: "https://youtu.be/video-11", title: "Evidence video", channelTitle: "Research", channelId: "channel-1", channelHandle: "@research", channelUrl: null, thumbnailUrl: null, durationSeconds: 120, publishedAt: null, playlistVideoCount: null, captionsEstimate: null, availabilityStatus: "available", warnings: [] });
    render(LibraryAddSourceDialog, { open: true, sources: [existingSource] as never, onSourcesChanged: vi.fn(), onStatus: vi.fn(), projectContext: project });
    const dialog = screen.getByRole("dialog", { name: "Add source" });
    await fireEvent.input(within(dialog).getByLabelText("YouTube URL"), { target: { value: "https://youtu.be/video-11" } });
    await fireEvent.click(within(dialog).getByRole("button", { name: "Preview" }));
    expect(await within(dialog).findByText("Already in Library: Evidence video")).toBeTruthy();
    const connect = within(dialog).getByRole("button", { name: "Connect to project" });
    expect(connect).toBeEnabled();
    await fireEvent.click(connect);
    await waitFor(() => expect(project.onConnectExistingSource).toHaveBeenCalledWith(11));
  });
});
