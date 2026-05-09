import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  cancelSourceJob,
  listSourceJobs,
  listenToSourceJobEvents,
  retryFailedYoutubePlaylistVideos,
  syncYoutubePlaylistVideo,
  syncYoutubeSource,
} from "./source-jobs";

const invokeMock = vi.hoisted(() => vi.fn());
const listenMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: listenMock,
}));

describe("source job api wrappers", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    listenMock.mockReset();
  });

  it("lists source jobs with a filter wrapper", async () => {
    invokeMock.mockResolvedValueOnce([]);

    await expect(listSourceJobs({ sourceId: 7, status: "running", limit: 25 })).resolves.toEqual(
      [],
    );

    expect(invokeMock).toHaveBeenLastCalledWith("list_source_jobs", {
      filter: { sourceId: 7, status: "running", limit: 25 },
    });
  });

  it("starts youtube source sync with options", async () => {
    const record = {
      job_id: "source-job-1",
      source_id: 7,
      related_source_id: null,
      job_type: "youtube_video_full_sync",
      status: "queued",
      message: "queued",
      progress_current: null,
      progress_total: null,
      started_at: 1,
      finished_at: null,
      warnings: [],
      error: null,
    };
    invokeMock.mockResolvedValueOnce(record);

    await expect(
      syncYoutubeSource(7, { metadata: true, transcripts: true, comments: false }),
    ).resolves.toEqual(record);

    expect(invokeMock).toHaveBeenLastCalledWith("sync_youtube_source", {
      sourceId: 7,
      options: { metadata: true, transcripts: true, comments: false },
    });
  });

  it("starts a single playlist video sync with camelCase ids", async () => {
    const record = {
      job_id: "source-job-2",
      source_id: 10,
      related_source_id: 20,
      job_type: "youtube_playlist_video_sync",
      status: "queued",
      message: "queued",
      progress_current: null,
      progress_total: null,
      started_at: 1,
      finished_at: null,
      warnings: [],
      error: null,
    };
    const options = { metadata: true, transcripts: true, comments: false };
    invokeMock.mockResolvedValueOnce(record);

    await expect(syncYoutubePlaylistVideo(10, 20, options)).resolves.toEqual(record);

    expect(invokeMock).toHaveBeenLastCalledWith("sync_youtube_playlist_video", {
      playlistSourceId: 10,
      videoSourceId: 20,
      options,
    });
  });

  it("retries failed playlist videos and cancels source jobs", async () => {
    const options = { metadata: false, transcripts: true, comments: false };
    invokeMock.mockResolvedValueOnce({
      job_id: "source-job-3",
      source_id: 10,
      related_source_id: null,
      job_type: "youtube_playlist_full_sync",
      status: "queued",
      message: "queued",
      progress_current: null,
      progress_total: null,
      started_at: 1,
      finished_at: null,
      warnings: [],
      error: null,
    });

    await retryFailedYoutubePlaylistVideos(10, options);
    expect(invokeMock).toHaveBeenLastCalledWith("retry_failed_youtube_playlist_videos", {
      sourceId: 10,
      options,
    });

    invokeMock.mockResolvedValueOnce(undefined);
    await cancelSourceJob("source-job-3");
    expect(invokeMock).toHaveBeenLastCalledWith("cancel_source_job", {
      jobId: "source-job-3",
    });
  });

  it("listens to source job events and unwraps payloads", async () => {
    const unlisten = vi.fn();
    listenMock.mockImplementationOnce((_eventName, handler) => {
      handler({ payload: { job_id: "source-job-1", source_id: 7 } });
      return Promise.resolve(unlisten);
    });
    const callback = vi.fn();

    await expect(listenToSourceJobEvents(callback)).resolves.toBe(unlisten);

    expect(listenMock).toHaveBeenLastCalledWith("sources://source-job", expect.any(Function));
    expect(callback).toHaveBeenLastCalledWith({ job_id: "source-job-1", source_id: 7 });
  });
});
