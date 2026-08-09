import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";

const api = vi.hoisted(() => ({ addYoutubeSource: vi.fn(), getYoutubePlaylistDetail: vi.fn() }));
vi.mock("$lib/api/sources", () => ({ addYoutubeSource: api.addYoutubeSource }));
vi.mock("$lib/api/youtube-detail", () => ({ getYoutubePlaylistDetail: api.getYoutubePlaylistDetail }));
import LibraryYoutubePlaylistImport from "./LibraryYoutubePlaylistImport.svelte";

const playlistSource = { id: "source:31", sourceId: 31, provider: "youtube", sourceSubtype: "playlist", typeLabel: "YouTube / Playlist", title: "Evidence playlist", subtitle: "Research", externalId: "playlist-31", projectCount: 0, lastCollectedAt: null, lastCollectedLabel: "Never", localCopyLabel: "2 materials", status: "active", disabledReason: null, alreadyConnected: false, connectable: true };
const content = { state: "not_synced", itemCount: 0, segmentCount: 0, lastSyncedAt: null, label: "Not synced" };
const detail = { summary: { sourceId: 31, sourceSubtype: "playlist", title: "Evidence playlist", channelTitle: "Research", channelHandle: "@research", canonicalUrl: "https://youtube.test/playlist", thumbnailUrl: null, durationSeconds: null, publishedAt: null, availabilityStatus: "available", videoCount: 2, linkedVideoCount: 0, unavailableCount: 0, captions: content, comments: content }, items: [1, 2].map((n) => ({ position: n, videoId: `video-${n}`, videoSourceId: null, title: `Video ${n}`, canonicalUrl: `https://youtu.be/video-${n}`, thumbnailUrl: null, durationSeconds: 60, publishedAt: null, availabilityStatus: "available", isRemovedFromPlaylist: false, captions: content, comments: content })) };
beforeEach(() => { api.getYoutubePlaylistDetail.mockResolvedValue(detail); api.addYoutubeSource.mockImplementation(async (url: string) => ({ id: url.endsWith("1") ? 41 : 42, title: url, externalId: url })); });
afterEach(() => { cleanup(); vi.clearAllMocks(); });

async function loadAndSelect() {
  await fireEvent.click(screen.getByRole("button", { name: /Evidence playlist/ }));
  await screen.findByText("Video 1");
  await fireEvent.click(screen.getByRole("checkbox", { name: /Video 1/ }));
  await fireEvent.click(screen.getByRole("checkbox", { name: /Video 2/ }));
}

describe("library add source contract", () => {
  it("adds selected videos from existing playlist details", async () => {
    const onStatus = vi.fn(); const onSourcesChanged = vi.fn();
    render(LibraryYoutubePlaylistImport, { sources: [playlistSource] as never, onStatus, onSourcesChanged });
    await loadAndSelect(); await fireEvent.click(screen.getByRole("button", { name: "Add selected" }));
    await screen.findByText("Added 2, skipped 0, failed 0.");
    expect(api.getYoutubePlaylistDetail).toHaveBeenCalledWith(31);
    expect(api.addYoutubeSource).toHaveBeenCalledTimes(2);
    expect(api.addYoutubeSource).toHaveBeenNthCalledWith(1, "https://youtu.be/video-1");
    expect(api.addYoutubeSource).toHaveBeenNthCalledWith(2, "https://youtu.be/video-2");
    expect(onSourcesChanged).toHaveBeenCalledWith(41);
    expect(onStatus).toHaveBeenCalledWith("Added 2 YouTube video sources.");
    expect(screen.getAllByRole("listitem")).toHaveLength(2);
    expect(screen.getByRole("region", { name: "YouTube playlist import" })).toBeTruthy();
  });

  it("connects all added playlist video source IDs through the project batch callback", async () => {
    const connect = vi.fn(); const scalar = vi.fn();
    render(LibraryYoutubePlaylistImport, { sources: [playlistSource] as never, onStatus: vi.fn(), onSourcesChanged: scalar, projectContext: { projectId: 5, connectedSourceIds: new Set<number>(), onConnectExistingSource: vi.fn(), onConnectAddedSources: connect } });
    await loadAndSelect(); await fireEvent.click(screen.getByRole("button", { name: "Add selected" }));
    await waitFor(() => expect(connect).toHaveBeenCalledWith([41, 42]));
    expect(connect).toHaveBeenCalledOnce();
    expect(scalar).not.toHaveBeenCalled();
    expect(api.addYoutubeSource).toHaveBeenCalledTimes(2);
  });
});
