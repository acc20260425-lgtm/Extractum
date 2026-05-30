import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  getYoutubePlaylistDetail,
  getYoutubeRuntimeStatus,
  getYoutubeVideoDetail,
  listYoutubeSourceSummaries,
} from "./youtube-detail";
import type { YoutubeSourceSummary, YoutubeVideoDetail } from "$lib/types/youtube";

const invokeMock = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

describe("youtube detail API", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("checks youtube runtime status", async () => {
    invokeMock.mockResolvedValueOnce({
      ytdlpAvailable: false,
      ytdlpVersion: null,
      message: "yt-dlp is not available on PATH",
    });

    await expect(getYoutubeRuntimeStatus()).resolves.toMatchObject({
      ytdlpAvailable: false,
    });
    expect(invokeMock).toHaveBeenLastCalledWith("get_youtube_runtime_status");
  });

  it("lists youtube summaries with source ids", async () => {
    invokeMock.mockResolvedValueOnce([]);
    await listYoutubeSourceSummaries([10, 11]);
    expect(invokeMock).toHaveBeenLastCalledWith("list_youtube_source_summaries", {
      sourceIds: [10, 11],
    });
  });

  it("loads youtube video and playlist detail", async () => {
    const videoDetail = {
      summary: youtubeSummary(10),
      sourceMetadata: {
        sourceId: 10,
        videoId: "video01",
        canonicalUrl: "https://www.youtube.com/watch?v=video01",
        title: "Demo video",
        channelTitle: "Demo channel",
        channelId: "UCdemo",
        channelHandle: "@demo",
        channelUrl: "https://www.youtube.com/@demo",
        authorDisplay: "Demo channel",
        publishedAt: 1_779_916_800,
        durationSeconds: 120,
        description: "Demo description",
        thumbnailUrl: "https://img.youtube.com/vi/video01/hqdefault.jpg",
        viewCount: 10,
        likeCount: 2,
        commentCount: 1,
        category: "Education",
        videoForm: "regular",
        availabilityStatus: "available",
        captionLanguageOverride: "en",
        rawMetadataVersion: 1,
        rawMetadataJson: { id: "video01" },
      },
      playlistMemberships: [],
    } satisfies YoutubeVideoDetail;
    invokeMock.mockResolvedValueOnce(videoDetail);
    await expect(getYoutubeVideoDetail(10)).resolves.toEqual(videoDetail);
    expect(invokeMock).toHaveBeenLastCalledWith("get_youtube_video_detail", { sourceId: 10 });

    invokeMock.mockResolvedValueOnce({ summary: { sourceId: 20 }, items: [] });
    await getYoutubePlaylistDetail(20);
    expect(invokeMock).toHaveBeenLastCalledWith("get_youtube_playlist_detail", { sourceId: 20 });
  });
});

function youtubeSummary(sourceId: number): YoutubeSourceSummary {
  return {
    sourceId,
    sourceSubtype: "video",
    title: "Demo video",
    channelTitle: "Demo channel",
    channelHandle: "@demo",
    canonicalUrl: "https://www.youtube.com/watch?v=video01",
    thumbnailUrl: "https://img.youtube.com/vi/video01/hqdefault.jpg",
    durationSeconds: 120,
    publishedAt: 1_779_916_800,
    availabilityStatus: "available",
    videoCount: null,
    linkedVideoCount: null,
    unavailableCount: null,
    captions: {
      state: "synced",
      itemCount: 1,
      segmentCount: 2,
      lastSyncedAt: 1_800_000_000,
      label: "Captions synced",
    },
    comments: {
      state: "not_synced",
      itemCount: 0,
      segmentCount: 0,
      lastSyncedAt: null,
      label: "Comments not synced",
    },
  };
}
