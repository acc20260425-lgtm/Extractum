import { describe, expect, it } from "vitest";
import {
  defaultAnalysisWorkspaceUiState,
  openRunWorkspaceState,
  selectSourceGroupWorkspace,
  selectSourceWorkspace,
  type WorkspaceSelection,
} from "./analysis-workspace-state";
import {
  restoredUiStateFromPersisted,
  type PersistedAnalysisWorkspaceState,
} from "./analysis-workspace-persistence";
import {
  runSnapshotAvailabilityFromPage,
  sourceBasisLabel,
  sourceCanvasSurface,
  type RunSnapshotAvailability,
} from "./analysis-report-canvas-state";
import {
  chatAvailabilityForRun,
  evidenceSourceActionDecision,
  filterCompanionRuns,
  runsFilterDefaults,
} from "./analysis-run-companion-state";
import type {
  AnalysisRunDetail,
  AnalysisRunMessagesPage,
  AnalysisRunSummary,
  AnalysisTraceRef,
} from "./types/analysis";

function run(overrides: Partial<AnalysisRunDetail> = {}): AnalysisRunDetail {
  return {
    id: 42,
    run_type: "report",
    scope_type: "single_source",
    source_id: 7,
    source_title: "Telegram A",
    source_group_id: null,
    source_group_name: null,
    scope_label: "Telegram A",
    scope_label_snapshot: "Telegram A at run time",
    period_from: 1710000000,
    period_to: 1710100000,
    output_language: "Russian",
    prompt_template_id: 3,
    prompt_template_name: "Weekly",
    prompt_template_version: 5,
    provider_profile: "default",
    provider: "openai",
    model: "gpt-5.4",
    youtube_corpus_mode: "transcript_description_comments",
    status: "completed",
    result_markdown: "Saved report",
    error: null,
    has_trace_data: true,
    created_at: 1710100010,
    completed_at: 1710100100,
    ...overrides,
  };
}

function summary(overrides: Partial<AnalysisRunSummary> = {}): AnalysisRunSummary {
  return {
    id: 42,
    run_type: "report",
    scope_type: "single_source",
    source_id: 7,
    source_title: "Telegram A",
    source_group_id: null,
    source_group_name: null,
    scope_label: "Telegram A",
    scope_label_snapshot: "Telegram A at run time",
    period_from: 1710000000,
    period_to: 1710100000,
    output_language: "Russian",
    prompt_template_id: 3,
    prompt_template_name: "Weekly",
    prompt_template_version: 5,
    provider_profile: "default",
    provider: "openai",
    model: "gpt-5.4",
    youtube_corpus_mode: "transcript_description_comments",
    status: "completed",
    error: null,
    has_trace_data: true,
    created_at: 1710100010,
    completed_at: 1710100100,
    ...overrides,
  };
}

function trace(overrides: Partial<AnalysisTraceRef> = {}): AnalysisTraceRef {
  return {
    ref: "s7-i11",
    item_id: 11,
    source_id: 7,
    external_id: "11",
    published_at: 1710000020,
    excerpt: "Saved excerpt",
    youtube_url: null,
    youtube_timestamp_seconds: null,
    youtube_display_label: null,
    is_synthetic: false,
    ...overrides,
  };
}

function snapshotPage(messageCount: number): AnalysisRunMessagesPage {
  return {
    messages: Array.from({ length: messageCount }, (_, index) => ({
      item_id: index + 1,
      source_id: 7,
      external_id: `m-${index + 1}`,
      author: "Alice",
      published_at: 1710000000 + index,
      ref: `s7-i${index + 1}`,
      content: `Message ${index + 1}`,
      item_kind: "telegram_message",
      source_type: "telegram",
      source_subtype: "channel",
      metadata_json: null,
    })),
    next_cursor: null,
    has_more: false,
  };
}

describe("analysis redesign final workflow scenarios", () => {
  it("opens a completed run as report-first, evidence-first, and snapshot-backed", () => {
    const opened = openRunWorkspaceState(defaultAnalysisWorkspaceUiState(), {
      runId: 42,
      status: "completed",
      sourceId: 7,
      sourceGroupId: null,
      liveScopeExists: true,
    });
    const snapshotAvailability = runSnapshotAvailabilityFromPage({
      currentRun: run(),
      page: snapshotPage(2),
      loading: false,
      errorMessage: "",
    });

    expect(opened.workspaceSelection).toEqual({ kind: "source", sourceId: 7 });
    expect(opened.openRunState).toEqual({ kind: "saved", runId: 42 });
    expect(opened.canvasMode).toBe("report");
    expect(opened.sourceViewBasis).toBe("run_snapshot");
    expect(opened.companionTab).toBe("evidence");
    expect(snapshotAvailability).toBe("available");
    expect(sourceCanvasSurface({
      currentRun: run(),
      sourceViewBasis: opened.sourceViewBasis,
      snapshotAvailability,
    })).toBe("run_snapshot_available");
    expect(chatAvailabilityForRun({
      currentRun: run(),
      snapshotAvailability,
    })).toMatchObject({ enabled: true, reason: "enabled" });
    expect(evidenceSourceActionDecision({
      currentRun: run(),
      selectedTrace: trace(),
      snapshotAvailability,
    })).toMatchObject({
      kind: "run_snapshot",
      canvasMode: "source",
      sourceViewBasis: "run_snapshot",
      highlightedRef: "s7-i11",
    });
  });

  it("selecting a source clears run-bound state and makes Runs current-scope filtering meaningful", () => {
    const opened = openRunWorkspaceState(defaultAnalysisWorkspaceUiState(), {
      runId: 42,
      status: "completed",
      sourceId: 7,
      sourceGroupId: null,
      liveScopeExists: true,
    });
    const selected = selectSourceWorkspace(
      {
        ...opened,
        companionTab: "chat",
        selectedTraceRef: "s7-i11",
      },
      8,
    );
    const filtered = filterCompanionRuns({
      activeRuns: [
        summary({ id: 1, status: "running", source_id: 8, source_title: "Telegram B" }),
        summary({ id: 2, status: "running", source_id: 7, source_title: "Telegram A" }),
      ],
      savedRuns: [
        summary({ id: 3, status: "completed", source_id: 8, source_title: "Telegram B" }),
        summary({ id: 4, status: "completed", source_id: 9, source_title: "Telegram C" }),
      ],
      filter: {
        ...runsFilterDefaults(),
        scope: "current",
      },
      workspaceSelection: selected.workspaceSelection,
    });

    expect(selected).toEqual({
      workspaceSelection: { kind: "source", sourceId: 8 },
      openRunState: { kind: "none" },
      canvasMode: "source",
      sourceViewBasis: "live_source",
      companionTab: "runs",
      selectedTraceRef: null,
    });
    expect(filtered.map((entry) => entry.run.id)).toEqual([1, 3]);
  });

  it("selecting a source group clears run-bound state without opening a pseudo-chat merge", () => {
    const selected = selectSourceGroupWorkspace(
      {
        ...defaultAnalysisWorkspaceUiState(),
        openRunState: { kind: "saved", runId: 42 },
        canvasMode: "report",
        sourceViewBasis: "run_snapshot",
        companionTab: "evidence",
        selectedTraceRef: "s7-i11",
      },
      12,
    );

    expect(selected.workspaceSelection).toEqual({ kind: "source_group", sourceGroupId: 12 });
    expect(selected.openRunState).toEqual({ kind: "none" });
    expect(selected.canvasMode).toBe("source");
    expect(selected.sourceViewBasis).toBe("live_source");
    expect(selected.companionTab).toBe("runs");
    expect(selected.selectedTraceRef).toBeNull();
  });

  it("keeps active runs honest while snapshot capture is pending", () => {
    const currentRun = run({
      status: "running",
      completed_at: null,
      result_markdown: null,
    });
    const opened = openRunWorkspaceState(defaultAnalysisWorkspaceUiState(), {
      runId: currentRun.id,
      status: currentRun.status,
      sourceId: currentRun.source_id,
      sourceGroupId: currentRun.source_group_id,
      liveScopeExists: true,
    });
    const snapshotAvailability = runSnapshotAvailabilityFromPage({
      currentRun,
      page: snapshotPage(0),
      loading: false,
      errorMessage: "",
    });

    expect(opened.openRunState).toEqual({ kind: "active", runId: currentRun.id });
    expect(opened.canvasMode).toBe("report");
    expect(opened.companionTab).toBe("runs");
    expect(snapshotAvailability).toBe("capturing");
    expect(sourceCanvasSurface({
      currentRun,
      sourceViewBasis: "run_snapshot",
      snapshotAvailability,
    })).toBe("run_snapshot_pending");
    expect(chatAvailabilityForRun({
      currentRun,
      snapshotAvailability,
    })).toMatchObject({ enabled: false, reason: "pending_completion" });
  });

  it("does not resolve completed-run evidence or chat against live source when snapshot rows are missing", () => {
    const currentRun = run({ status: "completed" });
    const snapshotAvailability: RunSnapshotAvailability = "unavailable";

    expect(sourceCanvasSurface({
      currentRun,
      sourceViewBasis: "run_snapshot",
      snapshotAvailability,
    })).toBe("run_snapshot_unavailable");
    expect(sourceBasisLabel({
      currentRun,
      sourceViewBasis: "live_source",
      snapshotAvailability,
    })).toContain("Live source");
    expect(chatAvailabilityForRun({
      currentRun,
      snapshotAvailability,
    })).toMatchObject({ enabled: false, reason: "missing_snapshot" });
    expect(evidenceSourceActionDecision({
      currentRun,
      selectedTrace: trace(),
      snapshotAvailability,
    })).toMatchObject({
      kind: "unavailable",
      reason: expect.stringContaining("completed run has no saved snapshot rows"),
    });
  });

  it("keeps saved runs with deleted live scope openable without faking rail selection", () => {
    const opened = openRunWorkspaceState(
      {
        ...defaultAnalysisWorkspaceUiState(),
        workspaceSelection: { kind: "source", sourceId: 99 },
      },
      {
        runId: 42,
        status: "completed",
        sourceId: 7,
        sourceGroupId: null,
        liveScopeExists: false,
      },
    );

    expect(opened.openRunState).toEqual({ kind: "saved", runId: 42 });
    expect(opened.workspaceSelection).toEqual({ kind: "none" });
    expect(opened.canvasMode).toBe("report");
    expect(opened.companionTab).toBe("evidence");
  });

  it("normalizes persisted run-bound UI because OpenRunState is never restored", () => {
    const persisted: PersistedAnalysisWorkspaceState = {
      version: 1,
      workspaceSelection: { kind: "source", sourceId: 7 } satisfies WorkspaceSelection,
      canvasMode: "report",
      sourceViewBasis: "run_snapshot",
      companionTab: "chat",
      runs: {
        historyScope: "current",
        runFilter: "completed",
        runsFilter: {
          ...runsFilterDefaults(),
          query: "weekly",
          scope: "current",
          status: "completed",
        },
      },
    };

    const restored = restoredUiStateFromPersisted(persisted);

    expect(restored.workspaceSelection).toEqual({ kind: "source", sourceId: 7 });
    expect(restored.openRunState).toEqual({ kind: "none" });
    expect(restored.canvasMode).toBe("report");
    expect(restored.sourceViewBasis).toBe("live_source");
    expect(restored.companionTab).toBe("runs");
    expect(restored.selectedTraceRef).toBeNull();
  });
});
