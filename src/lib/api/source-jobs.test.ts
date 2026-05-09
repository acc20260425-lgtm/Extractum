import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  listSourceJobs,
  listenToSourceJobEvents,
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
