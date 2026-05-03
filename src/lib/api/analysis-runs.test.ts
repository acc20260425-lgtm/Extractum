import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  ANALYSIS_RUN_EVENT,
  getAnalysisRun,
  listActiveAnalysisRuns,
  listAnalysisRuns,
  listenToAnalysisRunEvents,
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
