import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  getYoutubePlaylistDetail,
  getYoutubeRuntimeStatus,
  getYoutubeVideoDetail,
  listYoutubeSourceSummaries,
} from "./youtube-detail";

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
    invokeMock.mockResolvedValueOnce({ summary: { sourceId: 10 }, playlistMemberships: [] });
    await getYoutubeVideoDetail(10);
    expect(invokeMock).toHaveBeenLastCalledWith("get_youtube_video_detail", { sourceId: 10 });

    invokeMock.mockResolvedValueOnce({ summary: { sourceId: 20 }, items: [] });
    await getYoutubePlaylistDetail(20);
    expect(invokeMock).toHaveBeenLastCalledWith("get_youtube_playlist_detail", { sourceId: 20 });
  });
});
