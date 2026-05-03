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

  it("loads active runs and syncs live snapshots", async () => {
    const { state, deps, workflow } = createHarness({ activeRunId: 7 });
    deps.listActiveRuns.mockResolvedValueOnce([
      runSummary({ id: 7, status: "running" }),
      runSummary({ id: 8, status: "queued" }),
    ]);

    await workflow.loadActiveRuns();

    expect(state.activeRuns.map((run) => run.id)).toEqual([7, 8]);
    expect(deps.syncRunSnapshot).toHaveBeenCalledWith(7, "running");
    expect(deps.syncRunSnapshot).toHaveBeenCalledWith(8, "queued");
    expect(deps.pruneLiveRuns).toHaveBeenCalledWith([7, 8], null);
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
});
