import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  createAnalysisTraceWorkflow,
  type AnalysisTraceWorkflowPatch,
  type AnalysisTraceWorkflowState,
} from "./analysis-trace-workflow";
import type {
  AnalysisRunDetail,
  AnalysisRunSummary,
  AnalysisTraceData,
  AnalysisTraceRef,
} from "./types/analysis";

function runSummary(overrides: Partial<AnalysisRunSummary> = {}): AnalysisRunSummary {
  return {
    id: 7,
    run_type: "report",
    scope_type: "single_source",
    source_id: 2,
    source_title: "Source",
    source_group_id: null,
    source_group_name: null,
    scope_label: "Source",
    period_from: 100,
    period_to: 200,
    output_language: "Russian",
    prompt_template_id: 3,
    prompt_template_name: "Template",
    prompt_template_version: 1,
    provider_profile: "default",
    provider: "gemini",
    model: "gemini-2.5-flash",
    status: "completed",
    error: null,
    has_trace_data: true,
    created_at: 100,
    completed_at: 200,
    ...overrides,
  };
}

function runDetail(overrides: Partial<AnalysisRunDetail> = {}): AnalysisRunDetail {
  return {
    ...runSummary(overrides),
    result_markdown: "Saved report",
    ...overrides,
  };
}

function traceRef(overrides: Partial<AnalysisTraceRef> = {}): AnalysisTraceRef {
  return {
    ref: "ref-a",
    item_id: 1,
    source_id: 2,
    external_id: "100",
    published_at: 100,
    excerpt: "Saved excerpt",
    ...overrides,
  };
}

function traceData(refs: AnalysisTraceRef[] = [traceRef()]): AnalysisTraceData {
  return { refs };
}

type HarnessState = AnalysisTraceWorkflowState & {
  inspectorMode: "active" | "history" | "trace" | "chunks";
  status: string;
};

function createHarness(initial: Partial<HarnessState> = {}) {
  const state: HarnessState = {
    currentRun: runDetail(),
    traceData: { refs: [] },
    savedTraceRefs: [],
    resolvedTraceRefs: [],
    selectedTraceRef: null,
    inspectorMode: "history",
    status: "",
    ...initial,
  };

  const deps = {
    getState: () => state,
    patch: vi.fn((patch: AnalysisTraceWorkflowPatch) => Object.assign(state, patch)),
    getTrace: vi.fn(),
    resolveRefs: vi.fn(),
    formatError: vi.fn((action: string, error: unknown) => `Error ${action}: ${String(error)}`),
  };

  const workflow = createAnalysisTraceWorkflow(deps);
  return { state, deps, workflow };
}

describe("analysis-trace-workflow", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it("loads saved trace data and selects the first saved ref", async () => {
    const { state, deps, workflow } = createHarness();
    deps.getTrace.mockResolvedValueOnce(traceData([
      traceRef({ ref: "ref-b", published_at: 200 }),
      traceRef({ ref: "ref-a", published_at: 100 }),
    ]));

    await workflow.loadTrace(7);

    expect(deps.getTrace).toHaveBeenCalledWith(7);
    expect(state.traceData.refs.map((entry) => entry.ref)).toEqual(["ref-b", "ref-a"]);
    expect(state.savedTraceRefs).toEqual(["ref-b", "ref-a"]);
    expect(state.resolvedTraceRefs).toEqual([]);
    expect(state.selectedTraceRef).toBe("ref-b");
  });

  it("selects null when a saved trace load returns no refs", async () => {
    const { state, deps, workflow } = createHarness({
      selectedTraceRef: "old-ref",
    });
    deps.getTrace.mockResolvedValueOnce(traceData([]));

    await workflow.loadTrace(7);

    expect(state.traceData).toEqual({ refs: [] });
    expect(state.savedTraceRefs).toEqual([]);
    expect(state.resolvedTraceRefs).toEqual([]);
    expect(state.selectedTraceRef).toBeNull();
  });

  it("ignores stale guarded trace load success", async () => {
    const existing = traceRef({ ref: "existing" });
    const { state, deps, workflow } = createHarness({
      traceData: traceData([existing]),
      savedTraceRefs: ["existing"],
      selectedTraceRef: "existing",
    });
    deps.getTrace.mockResolvedValueOnce(traceData([traceRef({ ref: "stale" })]));

    await workflow.loadTrace(7, { isCurrent: () => false });

    expect(state.traceData.refs).toEqual([existing]);
    expect(state.savedTraceRefs).toEqual(["existing"]);
    expect(state.selectedTraceRef).toBe("existing");
  });

  it("ignores stale guarded trace load failure", async () => {
    const existing = traceRef({ ref: "existing" });
    const { state, deps, workflow } = createHarness({
      traceData: traceData([existing]),
      savedTraceRefs: ["existing"],
      selectedTraceRef: "existing",
    });
    deps.getTrace.mockRejectedValueOnce("db down");

    await workflow.loadTrace(7, { isCurrent: () => false });

    expect(state.traceData.refs).toEqual([existing]);
    expect(state.savedTraceRefs).toEqual(["existing"]);
    expect(state.status).toBe("");
  });

  it("clears trace state and reports status when current load fails", async () => {
    const { state, deps, workflow } = createHarness({
      traceData: traceData([traceRef({ ref: "existing" })]),
      savedTraceRefs: ["existing"],
      resolvedTraceRefs: ["resolved"],
      selectedTraceRef: "existing",
    });
    deps.getTrace.mockRejectedValueOnce("db down");

    await workflow.loadTrace(7);

    expect(state.traceData).toEqual({ refs: [] });
    expect(state.savedTraceRefs).toEqual([]);
    expect(state.resolvedTraceRefs).toEqual([]);
    expect(state.selectedTraceRef).toBeNull();
    expect(state.status).toBe("Error loading the analysis trace: db down");
  });

  it("does nothing when focusing a ref without a current run", async () => {
    const { state, deps, workflow } = createHarness({ currentRun: null });

    await workflow.focusTraceRef("ref-a");

    expect(deps.resolveRefs).not.toHaveBeenCalled();
    expect(state.inspectorMode).toBe("history");
    expect(state.selectedTraceRef).toBeNull();
  });

  it("selects an already loaded ref without resolving it again", async () => {
    const loaded = traceRef({ ref: "ref-a" });
    const { state, deps, workflow } = createHarness({
      traceData: traceData([loaded]),
    });

    await workflow.focusTraceRef("ref-a");

    expect(deps.resolveRefs).not.toHaveBeenCalled();
    expect(state.inspectorMode).toBe("trace");
    expect(state.selectedTraceRef).toBe("ref-a");
    expect(state.traceData.refs).toEqual([loaded]);
  });

  it("resolves a missing ref, merges it, and records resolved refs without duplicates", async () => {
    const existing = traceRef({ ref: "ref-b", published_at: 200 });
    const resolved = traceRef({ ref: "ref-a", item_id: 2, published_at: 100 });
    const { state, deps, workflow } = createHarness({
      traceData: traceData([existing]),
      savedTraceRefs: ["ref-b"],
      resolvedTraceRefs: ["ref-a"],
    });
    deps.resolveRefs.mockResolvedValueOnce([resolved]);

    await workflow.focusTraceRef("ref-a");

    expect(deps.resolveRefs).toHaveBeenCalledWith(7, ["ref-a"]);
    expect(state.traceData.refs).toEqual([resolved, existing]);
    expect(state.savedTraceRefs).toEqual(["ref-b"]);
    expect(state.resolvedTraceRefs).toEqual(["ref-a"]);
    expect(state.selectedTraceRef).toBe("ref-a");
    expect(state.inspectorMode).toBe("trace");
  });

  it("reports status when resolving a missing ref fails", async () => {
    const { state, deps, workflow } = createHarness();
    deps.resolveRefs.mockRejectedValueOnce("corpus unavailable");

    await workflow.focusTraceRef("ref-a");

    expect(state.status).toBe("Error resolving the trace reference: corpus unavailable");
    expect(state.selectedTraceRef).toBe("ref-a");
    expect(state.inspectorMode).toBe("trace");
  });

  it("clears trace state to the route default values", () => {
    const { state, workflow } = createHarness({
      traceData: traceData([traceRef()]),
      savedTraceRefs: ["ref-a"],
      resolvedTraceRefs: ["ref-b"],
      selectedTraceRef: "ref-a",
    });

    workflow.clearState();

    expect(state.traceData).toEqual({ refs: [] });
    expect(state.savedTraceRefs).toEqual([]);
    expect(state.resolvedTraceRefs).toEqual([]);
    expect(state.selectedTraceRef).toBeNull();
  });
});
