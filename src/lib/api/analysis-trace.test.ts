import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  getAnalysisRunTrace,
  resolveAnalysisTraceRefs,
} from "./analysis-trace";
import type { AnalysisTraceData, AnalysisTraceRef } from "$lib/types/analysis";

const invokeMock = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

function traceRef(overrides: Partial<AnalysisTraceRef> = {}): AnalysisTraceRef {
  return {
    ref: "ref-a",
    item_id: 1,
    source_id: 2,
    external_id: "100",
    published_at: 1_700_000,
    excerpt: "Saved excerpt",
    youtube_url: null,
    youtube_timestamp_seconds: null,
    youtube_display_label: null,
    is_synthetic: false,
    ...overrides,
  };
}

function traceData(overrides: Partial<AnalysisTraceData> = {}): AnalysisTraceData {
  return {
    refs: [traceRef()],
    ...overrides,
  };
}

describe("analysis trace api wrapper", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("loads saved analysis trace data for a run", async () => {
    const data = traceData();
    invokeMock.mockResolvedValueOnce(data);

    await expect(getAnalysisRunTrace(7)).resolves.toEqual(data);

    expect(invokeMock).toHaveBeenLastCalledWith("get_analysis_run_trace", {
      runId: 7,
    });
  });

  it("resolves requested trace refs for a run", async () => {
    const refs = [traceRef({ ref: "ref-b", item_id: 2 })];
    invokeMock.mockResolvedValueOnce(refs);

    await expect(resolveAnalysisTraceRefs(7, ["ref-b"])).resolves.toEqual(refs);

    expect(invokeMock).toHaveBeenLastCalledWith("resolve_analysis_trace_refs", {
      runId: 7,
      refs: ["ref-b"],
    });
  });
});
