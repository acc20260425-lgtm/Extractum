import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  createAnalysisRunWorkflow,
  type AnalysisRunWorkflowPatch,
  type AnalysisRunWorkflowState,
} from "./analysis-run-workflow";
import type { AnalysisHistoryScopeParams } from "./analysis-scope-state";
import type {
  AnalysisRunDetail,
  AnalysisRunEvent,
  AnalysisRunSummary,
} from "./types/analysis";

function runSummary(overrides: Partial<AnalysisRunSummary> = {}): AnalysisRunSummary {
  return {
    id: 1,
    run_type: "daily",
    scope_type: "single_source",
    source_id: 2,
    source_title: "Source",
    source_group_id: null,
    source_group_name: null,
    scope_label: "Source",
    period_from: 100,
    period_to: 200,
    output_language: "en",
    prompt_template_id: 3,
    prompt_template_name: "Template",
    prompt_template_version: 1,
    provider_profile: "default",
    provider: "gemini",
    model: "gemini-2.5-flash",
    status: "completed",
    error: null,
    has_trace_data: false,
    created_at: 100,
    completed_at: 200,
    ...overrides,
  };
}

function runDetail(overrides: Partial<AnalysisRunDetail> = {}): AnalysisRunDetail {
  return {
    ...runSummary(overrides),
    result_markdown: "saved result",
    ...overrides,
  };
}

function runEvent(overrides: Partial<AnalysisRunEvent> = {}): AnalysisRunEvent {
  return {
    run_id: 7,
    request_id: null,
    kind: "progress",
    phase: "map",
    queue_position: null,
    message: null,
    progress_current: null,
    progress_total: null,
    delta: null,
    chunk_summary: null,
    error: null,
    ...overrides,
  };
}

function createHarness(initial: Partial<AnalysisRunWorkflowState> = {}) {
  const state: AnalysisRunWorkflowState & {
    runs: AnalysisRunSummary[];
    activeRuns: AnalysisRunSummary[];
    loadingRuns: boolean;
    loadingActiveRuns: boolean;
    loadingRunDetail: boolean;
    inspectorMode: "active" | "history" | "trace" | "chunks";
    status: string;
  } = {
    historyScopeParams: { sourceId: null, sourceGroupId: null },
    activeRunId: null,
    currentRun: null,
    activeChatRequestId: null,
    activeChatRunId: null,
    runs: [],
    activeRuns: [],
    loadingRuns: false,
    loadingActiveRuns: false,
    loadingRunDetail: false,
    inspectorMode: "history",
    status: "",
    ...initial,
  };

  const deps = {
    getState: () => state,
    patch: vi.fn((patch: AnalysisRunWorkflowPatch) => Object.assign(state, patch)),
    listRuns: vi.fn(),
    listActiveRuns: vi.fn(),
    getRun: vi.fn(),
    syncRunSnapshot: vi.fn(),
    pruneLiveRuns: vi.fn(),
    applyRunEvent: vi.fn(),
    cancelChatSilently: vi.fn(),
    clearChatState: vi.fn(),
    loadChatMessages: vi.fn(),
    loadTrace: vi.fn(),
    clearTraceState: vi.fn(),
    formatError: vi.fn((action: string, error: unknown) => `Error ${action}: ${String(error)}`),
  };

  const workflow = createAnalysisRunWorkflow(deps);
  return { state, deps, workflow };
}

describe("analysis-run-workflow", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it("clears saved runs without loading when history scope is unavailable", async () => {
    const { state, deps, workflow } = createHarness({
      historyScopeParams: null,
      runs: [runSummary({ id: 1 })],
      loadingRuns: true,
    });

    await workflow.loadRuns();

    expect(state.runs).toEqual([]);
    expect(deps.listRuns).not.toHaveBeenCalled();
    expect(state.loadingRuns).toBe(false);
  });

  it("loads saved runs for the current scope while excluding active statuses", async () => {
    const params: AnalysisHistoryScopeParams = { sourceId: 7, sourceGroupId: null };
    const { state, deps, workflow } = createHarness({ historyScopeParams: params });
    deps.listRuns.mockResolvedValueOnce([
      runSummary({ id: 1, status: "completed" }),
      runSummary({ id: 2, status: "running" }),
      runSummary({ id: 3, status: "queued" }),
      runSummary({ id: 4, status: "failed" }),
    ]);

    await workflow.loadRuns();

    expect(deps.listRuns).toHaveBeenCalledWith({
      sourceId: 7,
      sourceGroupId: null,
      limit: 50,
    });
    expect(state.runs.map((run) => run.id)).toEqual([1, 4]);
    expect(state.loadingRuns).toBe(false);
  });

  it("sets status and clears loading state when saved run loading fails", async () => {
    const { state, deps, workflow } = createHarness();
    deps.listRuns.mockRejectedValueOnce("db down");

    await workflow.loadRuns();

    expect(state.status).toBe("Error loading analysis runs: db down");
    expect(state.loadingRuns).toBe(false);
  });

  it("loads active runs, syncs live snapshots, and preserves the opened run", async () => {
    const { state, deps, workflow } = createHarness({
      activeRunId: 7,
      currentRun: runDetail({ id: 8, status: "running" }),
    });
    deps.listActiveRuns.mockResolvedValueOnce([
      runSummary({ id: 7, status: "running" }),
      runSummary({ id: 8, status: "queued" }),
    ]);

    await workflow.loadActiveRuns();

    expect(state.activeRuns.map((run) => run.id)).toEqual([7, 8]);
    expect(deps.syncRunSnapshot).toHaveBeenCalledWith(7, "running");
    expect(deps.syncRunSnapshot).toHaveBeenCalledWith(8, "queued");
    expect(deps.pruneLiveRuns).toHaveBeenCalledWith([7, 8], 8);
    expect(state.activeRunId).toBe(7);
    expect(state.loadingActiveRuns).toBe(false);
  });

  it("auto-opens the first active run when the selected active id is stale", async () => {
    const { state, deps, workflow } = createHarness({ activeRunId: 99 });
    deps.listActiveRuns.mockResolvedValueOnce([runSummary({ id: 7, status: "running" })]);
    deps.getRun.mockResolvedValueOnce(runDetail({ id: 7, status: "running" }));
    deps.loadChatMessages.mockResolvedValueOnce(undefined);

    await workflow.loadActiveRuns();

    expect(state.activeRunId).toBe(7);
    expect(deps.getRun).toHaveBeenCalledWith(7);
  });

  it("opens a run by loading detail, chat, and trace data", async () => {
    const { state, deps, workflow } = createHarness();
    deps.getRun.mockResolvedValueOnce(runDetail({
      id: 7,
      status: "completed",
      has_trace_data: true,
    }));
    deps.loadChatMessages.mockResolvedValueOnce(undefined);
    deps.loadTrace.mockResolvedValueOnce(undefined);

    await workflow.openRun(7);

    expect(state.inspectorMode).toBe("history");
    expect(state.activeRunId).toBe(7);
    expect(state.currentRun?.id).toBe(7);
    expect(deps.syncRunSnapshot).toHaveBeenCalledWith(7, "completed");
    expect(deps.loadChatMessages).toHaveBeenCalledWith(7, expect.objectContaining({
      isCurrent: expect.any(Function),
    }));
    expect(deps.loadTrace).toHaveBeenCalledWith(7, expect.objectContaining({
      isCurrent: expect.any(Function),
    }));
    expect(state.loadingRunDetail).toBe(false);
  });

  it("clears trace state when the opened run has no trace data", async () => {
    const { deps, workflow } = createHarness();
    deps.getRun.mockResolvedValueOnce(runDetail({ id: 7, has_trace_data: false }));
    deps.loadChatMessages.mockResolvedValueOnce(undefined);

    await workflow.openRun(7);

    expect(deps.clearTraceState).toHaveBeenCalled();
    expect(deps.loadTrace).not.toHaveBeenCalled();
  });

  it("cancels a foreign active chat before opening another run", async () => {
    const { deps, workflow } = createHarness({
      activeChatRequestId: "chat-a",
      activeChatRunId: 5,
    });
    deps.getRun.mockResolvedValueOnce(runDetail({ id: 7 }));
    deps.loadChatMessages.mockResolvedValueOnce(undefined);

    await workflow.openRun(7);

    expect(deps.cancelChatSilently).toHaveBeenCalled();
    expect(deps.clearChatState).toHaveBeenCalled();
  });

  it("reports a not-found run and clears current run only when it matches", async () => {
    const { state, deps, workflow } = createHarness({
      currentRun: runDetail({ id: 7 }),
    });
    deps.getRun.mockResolvedValueOnce(null);

    await workflow.openRun(7);

    expect(state.status).toBe("Analysis run 7 was not found.");
    expect(state.currentRun).toBeNull();
    expect(state.loadingRunDetail).toBe(false);

    const other = createHarness({
      currentRun: runDetail({ id: 8 }),
    });
    other.deps.getRun.mockResolvedValueOnce(null);

    await other.workflow.openRun(7);

    expect(other.state.status).toBe("Analysis run 7 was not found.");
    expect(other.state.currentRun?.id).toBe(8);
    expect(other.state.loadingRunDetail).toBe(false);
  });

  it("ignores stale openRun results from overlapping requests", async () => {
    const { state, deps, workflow } = createHarness();
    let resolveFirst: (run: AnalysisRunDetail) => void = () => {};
    deps.getRun
      .mockImplementationOnce(() => new Promise<AnalysisRunDetail>((resolve) => {
        resolveFirst = resolve;
      }))
      .mockResolvedValueOnce(runDetail({ id: 8, result_markdown: "second" }));
    deps.loadChatMessages.mockResolvedValue(undefined);

    const first = workflow.openRun(7);
    const second = workflow.openRun(8);
    resolveFirst(runDetail({ id: 7, result_markdown: "first" }));
    await Promise.all([first, second]);

    expect(state.currentRun?.id).toBe(8);
    expect(deps.loadChatMessages).toHaveBeenCalledWith(8, expect.any(Object));
    expect(deps.loadChatMessages).not.toHaveBeenCalledWith(7, expect.any(Object));
    expect(state.loadingRunDetail).toBe(false);
  });

  it("keeps stale loadChatMessages callbacks from changing route chat state", async () => {
    const { deps, workflow } = createHarness();
    let capturedGuard = { isCurrent: () => true };
    deps.getRun.mockResolvedValueOnce(runDetail({ id: 7 }));
    deps.loadChatMessages.mockImplementationOnce(async (_runId, guard) => {
      capturedGuard = guard;
    });

    await workflow.openRun(7);
    workflow.invalidateOpenRunRequests();

    expect(capturedGuard.isCurrent()).toBe(false);
  });

  it("applies run events and switches the inspector to chunks when chunk summaries arrive", () => {
    const { state, deps, workflow } = createHarness({ activeRunId: 7 });
    const payload = runEvent({
      run_id: 7,
      chunk_summary: {
        index: 1,
        total: 2,
        message_count: 10,
        summary: "chunk",
        topics: [],
        notable_points: [],
        candidate_refs: [],
      },
    });

    workflow.handleRunEvent(payload);

    expect(deps.applyRunEvent).toHaveBeenCalledWith(payload);
    expect(state.inspectorMode).toBe("chunks");
  });

  it("selects and opens the event run when no run is active", () => {
    const { state, deps, workflow } = createHarness({ activeRunId: null });
    deps.getRun.mockResolvedValueOnce(runDetail({ id: 7 }));
    deps.loadChatMessages.mockResolvedValueOnce(undefined);

    workflow.handleRunEvent(runEvent({
      run_id: 7,
      kind: "started",
      message: "Started",
    }));

    expect(state.activeRunId).toBe(7);
    expect(state.inspectorMode).toBe("history");
    expect(deps.getRun).toHaveBeenCalledWith(7);
    expect(state.status).toBe("Started");
  });

  it("updates progress status only for the focused run", () => {
    const { state, workflow } = createHarness({ activeRunId: 7 });

    workflow.handleRunEvent(runEvent({
      run_id: 8,
      kind: "progress",
      message: "Other run progress",
    }));
    expect(state.status).toBe("");

    workflow.handleRunEvent(runEvent({
      run_id: 7,
      kind: "progress",
      message: "Focused progress",
    }));
    expect(state.status).toBe("Focused progress");
  });

  it("refreshes active and saved runs on terminal events", () => {
    const { state, deps, workflow } = createHarness({
      activeRunId: 7,
      currentRun: runDetail({ id: 7 }),
    });
    deps.listActiveRuns.mockResolvedValue([]);
    deps.listRuns.mockResolvedValue([]);
    deps.getRun.mockResolvedValue(runDetail({ id: 7, status: "completed" }));
    deps.loadChatMessages.mockResolvedValue(undefined);

    workflow.handleRunEvent(runEvent({
      run_id: 7,
      kind: "completed",
      message: "Analysis complete",
    }));

    expect(state.status).toBe("Analysis complete");
    expect(deps.listActiveRuns).toHaveBeenCalled();
    expect(deps.listRuns).toHaveBeenCalled();
    expect(deps.getRun).toHaveBeenCalledWith(7);
  });

  it("uses terminal error status for focused failed events without a message", () => {
    const { state, deps, workflow } = createHarness({ activeRunId: 7 });
    deps.listActiveRuns.mockResolvedValue([]);
    deps.listRuns.mockResolvedValue([]);
    deps.getRun.mockResolvedValue(runDetail({ id: 7, status: "failed" }));
    deps.loadChatMessages.mockResolvedValue(undefined);

    workflow.handleRunEvent(runEvent({
      run_id: 7,
      kind: "failed",
      message: null,
      error: "model failed",
    }));

    expect(state.status).toBe("Analysis failed: model failed");
  });
});
