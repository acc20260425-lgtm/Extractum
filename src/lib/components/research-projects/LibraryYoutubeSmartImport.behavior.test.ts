import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import "$lib/testing/dom-assertions";

const api = vi.hoisted(() => ({ addYoutubeSource: vi.fn(), previewYoutubeSource: vi.fn() }));
vi.mock("$lib/api/sources", () => api);

import LibraryYoutubeSmartImport from "./LibraryYoutubeSmartImport.svelte";

const preview = (kind: "video" | "playlist" = "video") => ({
  kind, externalId: "video-11", canonicalUrl: "https://youtu.be/video-11", title: "Evidence video",
  channelTitle: "Research", channelId: "channel-1", channelHandle: "@research", channelUrl: null,
  thumbnailUrl: null, durationSeconds: 120, publishedAt: null,
  playlistVideoCount: kind === "playlist" ? 3 : null, captionsEstimate: null,
  availabilityStatus: "available", warnings: [],
});
const existingSource = {
  id: "source:11", sourceId: 11, provider: "youtube", sourceSubtype: "video", typeLabel: "YouTube / Video",
  title: "Evidence video", subtitle: "Research", projectCount: 0, itemCount: 0, itemCountLabel: "0 items",
  status: "active", statusDetail: null, createdAt: null, lastSyncedAt: null, addedAtLabel: "Today",
  lastSyncedLabel: "Never", canonicalUrl: "https://youtu.be/video-11", externalId: "video-11",
  youtube: { video_form: "standard", duration_seconds: 120, playlist_video_count: null, channel_title: "Research", availability_status: "available" },
  telegram: null,
};

beforeEach(() => {
  api.previewYoutubeSource.mockResolvedValue(preview());
  api.addYoutubeSource.mockResolvedValue({ id: 22, title: "Evidence video", externalId: "video-11" });
});
afterEach(() => { cleanup(); vi.clearAllMocks(); });

async function previewUrl(url = "https://youtu.be/video-11") {
  await fireEvent.input(screen.getByLabelText("YouTube URL"), { target: { value: url } });
  await fireEvent.click(screen.getByRole("button", { name: "Preview" }));
  await screen.findByText("Evidence video");
}

describe("library add source contract", () => {
  it("classifies YouTube smart import before calling backend preview", async () => {
    render(LibraryYoutubeSmartImport, { sources: [], onSourcesChanged: vi.fn(), onStatus: vi.fn() });
    await previewUrl();
    expect(api.previewYoutubeSource).toHaveBeenCalledWith("https://youtu.be/video-11");
    expect(screen.getByRole("region", { name: "YouTube smart import" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Add source" })).toBeEnabled();
    expect(screen.getByText("video")).toBeTruthy();
    expect(screen.getByText("available")).toBeTruthy();
    expect(screen.getByText("2:00")).toBeTruthy();
    expect(screen.getByText("https://youtu.be/video-11")).toBeTruthy();
    expect(api.addYoutubeSource).not.toHaveBeenCalled();
  });

  it("does not materialize playlist videos from Library smart import", async () => {
    api.previewYoutubeSource.mockResolvedValue(preview("playlist"));
    render(LibraryYoutubeSmartImport, { sources: [], onSourcesChanged: vi.fn(), onStatus: vi.fn() });
    await previewUrl("https://www.youtube.com/playlist?list=video-11");
    await fireEvent.click(screen.getByRole("button", { name: "Add source" }));
    expect(api.addYoutubeSource).toHaveBeenCalledWith("https://www.youtube.com/playlist?list=video-11", { materializePlaylistVideos: false });
    expect(api.addYoutubeSource).toHaveBeenCalledOnce();
  });

  it("keeps duplicate YouTube smart import feedback inside the modal", async () => {
    render(LibraryYoutubeSmartImport, { sources: [existingSource] as never, onSourcesChanged: vi.fn(), onStatus: vi.fn() });
    await previewUrl();
    expect(screen.getByText("Already in Library: Evidence video")).toBeTruthy();
    expect(screen.getByRole("button", { name: "Already in Library" })).toBeDisabled();
    expect(api.addYoutubeSource).not.toHaveBeenCalled();
    expect(screen.getByRole("region", { name: "YouTube smart import" }).contains(screen.getByText("Already in Library: Evidence video"))).toBe(true);
  });

  it("allows Smart import duplicates to connect existing Library sources in project mode", async () => {
    const connect = vi.fn();
    render(LibraryYoutubeSmartImport, { sources: [existingSource] as never, onSourcesChanged: vi.fn(), onStatus: vi.fn(), projectContext: { projectId: 5, connectedSourceIds: new Set<number>(), onConnectExistingSource: connect, onConnectAddedSources: vi.fn() } });
    await previewUrl();
    const button = screen.getByRole("button", { name: "Connect to project" });
    expect(button).toBeEnabled();
    await fireEvent.click(button);
    await waitFor(() => expect(connect).toHaveBeenCalledWith(11));
    expect(connect).toHaveBeenCalledOnce();
    expect(api.addYoutubeSource).not.toHaveBeenCalled();
    expect(screen.getByText("Already in Library: Evidence video")).toBeTruthy();
  });

  it("keeps Smart import playlists on the scalar source callback path", async () => {
    const onSourcesChanged = vi.fn();
    api.previewYoutubeSource.mockResolvedValue(preview("playlist"));
    render(LibraryYoutubeSmartImport, { sources: [], onSourcesChanged, onStatus: vi.fn() });
    await previewUrl("https://www.youtube.com/playlist?list=video-11");
    await fireEvent.click(screen.getByRole("button", { name: "Add source" }));
    await waitFor(() => expect(onSourcesChanged).toHaveBeenCalledWith(22));
    expect(onSourcesChanged).toHaveBeenCalledOnce();
  });
});
