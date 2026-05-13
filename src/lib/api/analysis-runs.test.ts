import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  ANALYSIS_RUN_EVENT,
  cancelAnalysisRun,
  deleteAnalysisRun,
  getAnalysisRun,
  listActiveAnalysisRuns,
  listAnalysisRunMessages,
  listAnalysisRuns,
  listenToAnalysisRunEvents,
  startAnalysisReport,
} from "./analysis-runs";
import type { AnalysisRunEvent } from "$lib/types/analysis";

const invokeMock = vi.hoisted(() => vi.fn());
const listenMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: listenMock,
}));

describe("analysis run api wrappers", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    listenMock.mockReset();
  });

  it("wraps analysis run list commands with typed arguments", async () => {
    invokeMock.mockResolvedValueOnce([]);
    await expect(listAnalysisRuns({
      sourceId: 7,
      sourceGroupId: null,
      limit: 50,
    })).resolves.toEqual([]);
    expect(invokeMock).toHaveBeenLastCalledWith("list_analysis_runs", {
      sourceId: 7,
      sourceGroupId: null,
      limit: 50,
    });

    invokeMock.mockResolvedValueOnce([]);
    await expect(listActiveAnalysisRuns()).resolves.toEqual([]);
    expect(invokeMock).toHaveBeenLastCalledWith("list_active_analysis_runs");
  });

  it("wraps analysis run detail loading", async () => {
    invokeMock.mockResolvedValueOnce(null);

    await expect(getAnalysisRun(42)).resolves.toBeNull();
    expect(invokeMock).toHaveBeenLastCalledWith("get_analysis_run", { runId: 42 });
  });

  it("wraps snapshot-only run message paging", async () => {
    invokeMock.mockResolvedValueOnce({
      messages: [],
      next_cursor: null,
      has_more: false,
    });

    await expect(listAnalysisRunMessages({
      runId: 42,
      after: { published_at: 1_710_000_000, ref: "s7-i1" },
      limit: 25,
      sourceId: 20,
    })).resolves.toEqual({
      messages: [],
      next_cursor: null,
      has_more: false,
    });

    expect(invokeMock).toHaveBeenLastCalledWith("list_analysis_run_messages", {
      runId: 42,
      after: { published_at: 1_710_000_000, ref: "s7-i1" },
      limit: 25,
      sourceId: 20,
    });
  });

  it("passes an around ref for focused snapshot paging", async () => {
    invokeMock.mockResolvedValueOnce({
      messages: [],
      next_cursor: null,
      has_more: false,
    });

    await expect(listAnalysisRunMessages({
      runId: 7,
      after: null,
      limit: 50,
      sourceId: 12,
      aroundRef: "s12-i99",
    })).resolves.toEqual({
      messages: [],
      next_cursor: null,
      has_more: false,
    });

    expect(invokeMock).toHaveBeenLastCalledWith("list_analysis_run_messages", {
      runId: 7,
      after: null,
      limit: 50,
      sourceId: 12,
      aroundRef: "s12-i99",
    });
  });

  it("wraps analysis report start and destructive run actions", async () => {
    invokeMock.mockResolvedValueOnce(77);
    await expect(startAnalysisReport({
      sourceId: 7,
      sourceGroupId: null,
      periodFrom: 1_776_038_400,
      periodTo: 1_776_211_199,
      outputLanguage: "Russian",
      promptTemplateId: 5,
      modelOverride: null,
      profileId: null,
      youtubeCorpusMode: "transcript_description",
    })).resolves.toBe(77);
    expect(invokeMock).toHaveBeenLastCalledWith("start_analysis_report", {
      sourceId: 7,
      sourceGroupId: null,
      periodFrom: 1_776_038_400,
      periodTo: 1_776_211_199,
      outputLanguage: "Russian",
      promptTemplateId: 5,
      modelOverride: null,
      profileId: null,
      youtubeCorpusMode: "transcript_description",
    });

    invokeMock.mockResolvedValueOnce(undefined);
    await expect(cancelAnalysisRun(77)).resolves.toBeUndefined();
    expect(invokeMock).toHaveBeenLastCalledWith("cancel_analysis_run", { runId: 77 });

    invokeMock.mockResolvedValueOnce(undefined);
    await expect(deleteAnalysisRun(77)).resolves.toBeUndefined();
    expect(invokeMock).toHaveBeenLastCalledWith("delete_analysis_run", { runId: 77 });
  });

  it("passes explicit analysis report profile ids through unchanged", async () => {
    invokeMock.mockResolvedValueOnce(78);

    await expect(startAnalysisReport({
      sourceId: null,
      sourceGroupId: 9,
      periodFrom: 1_776_038_400,
      periodTo: 1_776_211_199,
      outputLanguage: "Russian",
      promptTemplateId: 5,
      modelOverride: null,
      profileId: "work",
      youtubeCorpusMode: "transcript_description_comments",
    })).resolves.toBe(78);

    expect(invokeMock).toHaveBeenLastCalledWith("start_analysis_report", {
      sourceId: null,
      sourceGroupId: 9,
      periodFrom: 1_776_038_400,
      periodTo: 1_776_211_199,
      outputLanguage: "Russian",
      promptTemplateId: 5,
      modelOverride: null,
      profileId: "work",
      youtubeCorpusMode: "transcript_description_comments",
    });
  });

  it("listens on the shared analysis run event name", async () => {
    const unlisten = vi.fn();
    const handler = vi.fn();
    listenMock.mockResolvedValueOnce(unlisten);

    await expect(listenToAnalysisRunEvents(handler)).resolves.toBe(unlisten);
    expect(ANALYSIS_RUN_EVENT).toBe("analysis://run");
    expect(listenMock).toHaveBeenCalledWith(ANALYSIS_RUN_EVENT, expect.any(Function));

    const payload: AnalysisRunEvent = {
      run_id: 7,
      request_id: null,
      kind: "progress",
      phase: "map",
      queue_position: null,
      message: "Mapping",
      progress_current: null,
      progress_total: null,
      delta: null,
      chunk_summary: null,
      error: null,
    };
    const event = { payload };
    listenMock.mock.calls[0][1](event);
    expect(handler).toHaveBeenCalledWith(event);
  });
});
