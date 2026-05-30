import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  createAnalysisRunWorkflow,
  type AnalysisRunWorkflowPatch,
  type AnalysisRunWorkflowState,
} from "./analysis-run-workflow";
import { runsFilterDefaults, type CompanionRunsFilterState } from "./analysis-run-companion-state";
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
    youtube_corpus_mode: "transcript_description",
    telegram_history_scope: "current",
    status: "completed",
    error: null,
    has_trace_data: false,
    snapshot_state: "captured",
    snapshot_captured_at: "2026-05-18T10:00:00Z",
    snapshot_error: null,
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

type AnalysisRunWorkflowHarnessState = AnalysisRunWorkflowState & {
  runsFilter: CompanionRunsFilterState;
  runs: AnalysisRunSummary[];
  activeRuns: AnalysisRunSummary[];
  loadingRuns: boolean;
  loadingActiveRuns: boolean;
  loadingRunDetail: boolean;
  inspectorMode: "active" | "history" | "trace" | "chunks";
  startingReport: boolean;
  deletingRunIds: Record<number, boolean>;
  status: string;
};

function createHarness(initial: Partial<AnalysisRunWorkflowHarnessState> = {}) {
  const state: AnalysisRunWorkflowHarnessState = {
    historyScopeParams: { sourceId: null, sourceGroupId: null },
    runsFilter: runsFilterDefaults(),
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
    startingReport: false,
    deletingRunIds: {},
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
    startReport: vi.fn(),
    cancelRun: vi.fn(),
    deleteRun: vi.fn(),
    confirm: vi.fn(),
    cancelChatSilently: vi.fn(),
    clearChatState: vi.fn(),
    clearOpenedRunState: vi.fn(),
    setInitialLiveRun: vi.fn(),
    loadChatMessages: vi.fn(),
    loadTrace: vi.fn(),
    clearTraceState: vi.fn(),
    onRunOpened: vi.fn(),
    formatError: vi.fn((action: string, error: unknown) => `Error ${action}: ${String(error)}`),
  };

  const workflow = createAnalysisRunWorkflow(deps);
  return { state, deps, workflow };
}

describe("analysis-run-workflow", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it("clears saved runs for a null scope without reading workflow state", async () => {
    const { state, deps, workflow } = createHarness({
      historyScopeParams: { sourceId: 7, sourceGroupId: null },
      runs: [runSummary({ id: 1 })],
      loadingRuns: true,
    });
    const getState = vi.fn(() => {
      throw new Error("loadRunsForScope should use its params argument");
    });
    deps.getState = getState as typeof deps.getState;

    await workflow.loadRunsForScope(null, runsFilterDefaults());

    expect(state.runs).toEqual([]);
    expect(deps.listRuns).not.toHaveBeenCalled();
    expect(getState).not.toHaveBeenCalled();
    expect(state.loadingRuns).toBe(false);
  });

  it("loads saved runs for the provided scope without reading workflow state", async () => {
    const params: AnalysisHistoryScopeParams = { sourceId: 7, sourceGroupId: null };
    const { state, deps, workflow } = createHarness({
      historyScopeParams: { sourceId: 99, sourceGroupId: 100 },
    });
    const getState = vi.fn(() => {
      throw new Error("loadRunsForScope should use its params argument");
    });
    deps.getState = getState as typeof deps.getState;
    deps.listRuns.mockResolvedValueOnce([
      runSummary({ id: 1, status: "completed" }),
      runSummary({ id: 2, status: "running" }),
      runSummary({ id: 3, status: "failed" }),
    ]);
    const filter: CompanionRunsFilterState = {
      ...runsFilterDefaults(),
      query: "older target",
      status: "completed",
      dateFrom: "2026-05-01",
      dateTo: "2026-05-30",
      provider: "gemini",
      model: "flash",
      template: "digest",
    };

    await workflow.loadRunsForScope(params, filter);

    expect(deps.listRuns).toHaveBeenCalledWith({
      sourceId: 7,
      sourceGroupId: null,
      limit: 50,
      query: "older target",
      status: "completed",
      dateFrom: "2026-05-01",
      dateTo: "2026-05-30",
      provider: "gemini",
      model: "flash",
      template: "digest",
    });
    expect(state.runs.map((run) => run.id)).toEqual([1, 3]);
    expect(getState).not.toHaveBeenCalled();
    expect(state.loadingRuns).toBe(false);
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
      query: "",
      status: "all",
      dateFrom: "",
      dateTo: "",
      provider: "",
      model: "",
      template: "",
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

  it("ignores stale saved run responses when a newer load finishes first", async () => {
    const params: AnalysisHistoryScopeParams = { sourceId: 7, sourceGroupId: null };
    const { state, deps, workflow } = createHarness({ historyScopeParams: params });
    let resolveFirst: (runs: AnalysisRunSummary[]) => void = () => {};
    let resolveSecond: (runs: AnalysisRunSummary[]) => void = () => {};
    deps.listRuns
      .mockReturnValueOnce(new Promise<AnalysisRunSummary[]>((resolve) => {
        resolveFirst = resolve;
      }))
      .mockReturnValueOnce(new Promise<AnalysisRunSummary[]>((resolve) => {
        resolveSecond = resolve;
      }));

    const first = workflow.loadRunsForScope(params, {
      ...runsFilterDefaults(),
      query: "first",
    });
    const second = workflow.loadRunsForScope(params, {
      ...runsFilterDefaults(),
      query: "second",
    });

    resolveSecond([runSummary({ id: 2, scope_label: "Second" })]);
    await second;
    expect(state.runs.map((run) => run.id)).toEqual([2]);
    expect(state.loadingRuns).toBe(false);

    resolveFirst([runSummary({ id: 1, scope_label: "First" })]);
    await first;
    expect(state.runs.map((run) => run.id)).toEqual([2]);
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

  it("notifies route workspace state after a current run opens", async () => {
    const { deps, workflow } = createHarness();
    const run = runDetail({ id: 7, status: "completed" });
    deps.getRun.mockResolvedValueOnce(run);
    deps.loadChatMessages.mockResolvedValueOnce(undefined);

    await workflow.openRun(7);

    expect(deps.onRunOpened).toHaveBeenCalledWith(run);
  });

  it("does not notify route workspace state for a missing run", async () => {
    const { deps, workflow } = createHarness();
    deps.getRun.mockResolvedValueOnce(null);

    await workflow.openRun(7);

    expect(deps.onRunOpened).not.toHaveBeenCalled();
  });

  it("does not notify route workspace state for a stale open run response", async () => {
    const { deps, workflow } = createHarness();
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

    expect(deps.onRunOpened).toHaveBeenCalledTimes(1);
    expect(deps.onRunOpened).toHaveBeenCalledWith(expect.objectContaining({ id: 8 }));
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

  it("reports report-start validation failures without invoking the api", async () => {
    const { state, deps, workflow } = createHarness();

    await workflow.startReport({
      analysisScope: "single_source",
      selectedSourceId: "",
      selectedGroupId: "",
      selectedTemplateId: "5",
      periodFrom: "2026-05-01",
      periodTo: "2026-05-03",
      outputLanguage: "Russian",
      profileId: null,
      modelOverride: "",
      youtubeCorpusMode: "transcript_description",
      includeMigratedHistory: false,
    });

    expect(state.status).toBe("Select a source first.");
    expect(deps.startReport).not.toHaveBeenCalled();
    expect(state.startingReport).toBe(false);
  });

  it("starts a report, resets focused state, tracks the queued run, and opens it", async () => {
    const { state, deps, workflow } = createHarness({
      activeChatRequestId: "chat-a",
      activeChatRunId: 4,
      currentRun: runDetail({ id: 4 }),
    });
    deps.startReport.mockResolvedValueOnce(77);
    deps.listActiveRuns.mockResolvedValueOnce([runSummary({ id: 77, status: "queued" })]);
    deps.getRun.mockResolvedValueOnce(runDetail({ id: 77, status: "queued" }));
    deps.loadChatMessages.mockResolvedValueOnce(undefined);

    await workflow.startReport({
      analysisScope: "single_source",
      selectedSourceId: "7",
      selectedGroupId: "",
      selectedTemplateId: "5",
      periodFrom: "2026-05-01",
      periodTo: "2026-05-03",
      outputLanguage: " Russian ",
      profileId: "research",
      modelOverride: " ",
      youtubeCorpusMode: "transcript_description_comments",
      includeMigratedHistory: false,
    });

    expect(deps.cancelChatSilently).toHaveBeenCalled();
    expect(deps.clearChatState).toHaveBeenCalled();
    expect(deps.clearTraceState).toHaveBeenCalled();
    expect(deps.patch).toHaveBeenCalledWith(expect.objectContaining({
      currentRun: null,
      inspectorMode: "active",
      startingReport: true,
    }));
    expect(deps.startReport).toHaveBeenCalledWith(expect.objectContaining({
      sourceId: 7,
      sourceGroupId: null,
      outputLanguage: "Russian",
      promptTemplateId: 5,
      modelOverride: null,
      profileId: "research",
      youtubeCorpusMode: "transcript_description_comments",
    }));
    expect(deps.setInitialLiveRun).toHaveBeenCalledWith(77);
    expect(state.activeRunId).toBe(77);
    expect(state.currentRun?.id).toBe(77);
    expect(state.startingReport).toBe(false);
  });

  it("formats start report errors and clears the starting flag", async () => {
    const { state, deps, workflow } = createHarness();
    deps.startReport.mockRejectedValueOnce("model busy");

    await workflow.startReport({
      analysisScope: "source_group",
      selectedSourceId: "",
      selectedGroupId: "9",
      selectedTemplateId: "5",
      periodFrom: "2026-05-01",
      periodTo: "2026-05-03",
      outputLanguage: "English",
      profileId: null,
      modelOverride: "gemini-2.5-pro",
      youtubeCorpusMode: "transcript_only",
      includeMigratedHistory: false,
    });

    expect(state.status).toBe("Error starting the analysis report: model busy");
    expect(state.startingReport).toBe(false);
  });

  it("cancels a run and reports cancellation status", async () => {
    const { state, deps, workflow } = createHarness();
    deps.cancelRun.mockResolvedValueOnce(undefined);

    await workflow.cancelRun(77);

    expect(deps.cancelRun).toHaveBeenCalledWith(77);
    expect(state.status).toBe("Cancelling analysis run 77...");
  });

  it("formats cancel run errors", async () => {
    const { state, deps, workflow } = createHarness();
    deps.cancelRun.mockRejectedValueOnce("already stopped");

    await workflow.cancelRun(77);

    expect(state.status).toBe("Error cancelling the analysis run: already stopped");
  });

  it("blocks deleting active runs before confirmation", async () => {
    const { state, deps, workflow } = createHarness();

    await workflow.deleteSavedRun(runSummary({ id: 77, status: "running" }));

    expect(state.status).toBe("Cancel or wait for this run before deleting it.");
    expect(deps.confirm).not.toHaveBeenCalled();
    expect(deps.deleteRun).not.toHaveBeenCalled();
  });

  it("does not delete a saved run when confirmation is cancelled", async () => {
    const { deps, workflow } = createHarness();
    deps.confirm.mockResolvedValueOnce(false);

    await workflow.deleteSavedRun(runSummary({ id: 77, status: "completed" }));

    expect(deps.confirm).toHaveBeenCalledWith(expect.objectContaining({
      title: "Delete saved run?",
      confirmLabel: "Delete",
      tone: "danger",
    }));
    expect(deps.deleteRun).not.toHaveBeenCalled();
  });

  it("deletes a saved run, clears focused state, reloads runs, and clears pending state", async () => {
    const { state, deps, workflow } = createHarness({
      runs: [runSummary({ id: 77, status: "completed" }), runSummary({ id: 78, status: "failed" })],
      activeRuns: [runSummary({ id: 79, status: "queued" })],
      activeRunId: 77,
      currentRun: runDetail({ id: 77, status: "completed" }),
      activeChatRequestId: "chat-a",
      activeChatRunId: 77,
    });
    deps.confirm.mockResolvedValueOnce(true);
    deps.deleteRun.mockResolvedValueOnce(undefined);
    deps.listRuns.mockResolvedValueOnce([runSummary({ id: 78, status: "failed" })]);

    await workflow.deleteSavedRun(runSummary({ id: 77, status: "completed" }));

    expect(deps.cancelChatSilently).toHaveBeenCalled();
    expect(deps.deleteRun).toHaveBeenCalledWith(77);
    expect(state.runs.map((run) => run.id)).toEqual([78]);
    expect(state.activeRuns.map((run) => run.id)).toEqual([79]);
    expect(deps.clearOpenedRunState).toHaveBeenCalledWith(77);
    expect(state.inspectorMode).toBe("history");
    expect(state.status).toBe("Saved run 77 deleted.");
    expect(state.deletingRunIds).toEqual({});
  });

  it("formats delete run errors and clears pending state", async () => {
    const { state, deps, workflow } = createHarness();
    deps.confirm.mockResolvedValueOnce(true);
    deps.deleteRun.mockRejectedValueOnce("db locked");

    await workflow.deleteSavedRun(runSummary({ id: 77, status: "completed" }));

    expect(state.status).toBe("Error deleting the saved run: db locked");
    expect(state.deletingRunIds).toEqual({});
  });

  it("applies chunk summary run events without auto-switching visible companion state", () => {
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
    expect(state.inspectorMode).toBe("history");
    expect(deps.patch).not.toHaveBeenCalledWith(expect.objectContaining({
      inspectorMode: "chunks",
    }));
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
