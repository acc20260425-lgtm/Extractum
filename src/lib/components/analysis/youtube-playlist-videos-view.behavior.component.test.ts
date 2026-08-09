import { cleanup, fireEvent, render, screen } from "@testing-library/svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import YoutubePlaylistVideosView from "./youtube-playlist-videos-view.svelte";
import type { YoutubePlaylistDetail } from "$lib/types/youtube";

afterEach(cleanup);

function playlist(): YoutubePlaylistDetail {
  const synced = { state: "synced" as const, itemCount: 2, segmentCount: 3, lastSyncedAt: 1_700_000_000, label: "Synced" };
  return {
    summary: {
      sourceId: 1,
      sourceSubtype: "playlist",
      title: "Research playlist",
      channelTitle: "Research channel",
      channelHandle: "@research",
      canonicalUrl: "https://www.youtube.com/playlist?list=playlist-1",
      thumbnailUrl: null,
      durationSeconds: null,
      publishedAt: 1_700_000_000,
      availabilityStatus: "available",
      videoCount: 1,
      linkedVideoCount: 1,
      unavailableCount: 0,
      captions: synced,
      comments: synced,
    },
    items: [{
      position: 1,
      videoId: "video-2",
      videoSourceId: 2,
      title: "Playlist child video",
      canonicalUrl: "https://www.youtube.com/watch?v=video-2",
      thumbnailUrl: null,
      durationSeconds: 90,
      publishedAt: 1_700_000_000,
      availabilityStatus: "failed",
      isRemovedFromPlaylist: false,
      captions: synced,
      comments: synced,
    }],
  };
}

describe("analysis redesign final safety contract", () => {
  it("renders YouTube source material as transcript and playlist readers without an embedded player", async () => {
    const onOpenSource = vi.fn();
    const onSyncPlaylistVideo = vi.fn();
    const view = render(YoutubePlaylistVideosView, {
      props: {
        sourceTitle: "Fallback playlist",
        playlist: playlist(),
        loading: false,
        formatTimestamp: (value) => value === null ? "Never" : `time:${value}`,
        onOpenSource,
        onSyncPlaylist: vi.fn(),
        onRetryFailedPlaylistVideos: vi.fn(),
        onSyncPlaylistVideo,
        onRetryPlaylistVideo: vi.fn(),
      },
    });

    expect(screen.getByRole("region", { name: "YouTube playlist videos" })).toBeTruthy();
    expect(screen.getByRole("heading", { name: "Research playlist" })).toBeTruthy();
    expect(screen.getByText("@research")).toBeTruthy();
    expect(screen.getByText("1 videos")).toBeTruthy();
    expect(screen.getByText("1 linked")).toBeTruthy();
    expect(screen.getByText("Captions")).toBeTruthy();
    expect(screen.getAllByText("Synced - time:1700000000", { selector: "strong" })).toHaveLength(2);
    expect(screen.getByText("available", { selector: "strong" })).toBeTruthy();
    expect(screen.getByText("1. Playlist child video")).toBeTruthy();
    expect(screen.getByText("1:30")).toBeTruthy();
    expect(screen.getByRole("button", { name: "Open video source" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Sync this video" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Retry this video" })).toBeTruthy();
    await fireEvent.click(screen.getByRole("button", { name: "Open video source" }));
    expect(onOpenSource).toHaveBeenCalledWith(2);
    expect(view.container.querySelector("iframe")).toBeNull();
    expect(view.container.querySelector("video")).toBeNull();
  });
});
