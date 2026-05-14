import { describe, expect, it } from "vitest";
import {
  defaultAnalysisWorkspaceUiState,
  legacyScopeFromWorkspaceSelection,
  normalizeRestoredWorkspaceState,
  openRunWorkspaceState,
  selectSourceGroupWorkspace,
  selectSourceWorkspace,
  workspaceSelectionFromLegacy,
  workspaceSelectionFromRunScope,
  type AnalysisWorkspaceUiState,
} from "./analysis-workspace-state";

function baseState(overrides: Partial<AnalysisWorkspaceUiState> = {}): AnalysisWorkspaceUiState {
  return {
    ...defaultAnalysisWorkspaceUiState(),
    ...overrides,
  };
}

describe("analysis-workspace-state", () => {
  it("maps legacy route scope ids into explicit workspace selections", () => {
    expect(workspaceSelectionFromLegacy("single_source", "7", "")).toEqual({
      kind: "source",
      sourceId: 7,
    });
    expect(workspaceSelectionFromLegacy("source_group", "", "9")).toEqual({
      kind: "source_group",
      sourceGroupId: 9,
    });
    expect(workspaceSelectionFromLegacy("single_source", "", "9")).toEqual({ kind: "none" });
    expect(workspaceSelectionFromLegacy("source_group", "7", "")).toEqual({ kind: "none" });
    expect(workspaceSelectionFromLegacy("single_source", "not-a-number", "")).toEqual({
      kind: "none",
    });
  });

  it("maps explicit workspace selection back to legacy route scope ids", () => {
    expect(legacyScopeFromWorkspaceSelection({ kind: "source", sourceId: 7 })).toEqual({
      analysisScope: "single_source",
      selectedSourceId: "7",
      selectedGroupId: "",
    });
    expect(legacyScopeFromWorkspaceSelection({ kind: "source_group", sourceGroupId: 9 }))
      .toEqual({
        analysisScope: "source_group",
        selectedSourceId: "",
        selectedGroupId: "9",
      });
    expect(legacyScopeFromWorkspaceSelection({ kind: "none" })).toEqual({
      analysisScope: "single_source",
      selectedSourceId: "",
      selectedGroupId: "",
    });
  });

  it("opens a completed run as a saved report and defaults companion to evidence", () => {
    const next = openRunWorkspaceState(baseState(), {
      runId: 42,
      status: "completed",
      sourceId: 7,
      sourceGroupId: null,
      liveScopeExists: true,
    });

    expect(next).toMatchObject({
      workspaceSelection: { kind: "source", sourceId: 7 },
      openRunState: { kind: "saved", runId: 42 },
      canvasMode: "report",
      sourceViewBasis: "run_snapshot",
      companionTab: "evidence",
      selectedTraceRef: null,
    });
  });

  it("opens queued and running runs as active reports", () => {
    expect(openRunWorkspaceState(baseState(), {
      runId: 43,
      status: "queued",
      sourceId: null,
      sourceGroupId: 9,
      liveScopeExists: true,
    })).toMatchObject({
      workspaceSelection: { kind: "source_group", sourceGroupId: 9 },
      openRunState: { kind: "active", runId: 43 },
      canvasMode: "report",
      companionTab: "runs",
    });

    expect(openRunWorkspaceState(baseState(), {
      runId: 44,
      status: "running",
      sourceId: 7,
      sourceGroupId: null,
      liveScopeExists: true,
    }).openRunState).toEqual({ kind: "active", runId: 44 });
  });

  it("does not fake a live workspace selection for a run with deleted scope", () => {
    const next = openRunWorkspaceState(baseState({
      workspaceSelection: { kind: "source", sourceId: 99 },
    }), {
      runId: 45,
      status: "completed",
      sourceId: 7,
      sourceGroupId: null,
      liveScopeExists: false,
    });

    expect(workspaceSelectionFromRunScope({
      runId: 45,
      status: "completed",
      sourceId: 7,
      sourceGroupId: null,
      liveScopeExists: false,
    })).toEqual({ kind: "none" });
    expect(next.workspaceSelection).toEqual({ kind: "none" });
    expect(next.openRunState).toEqual({ kind: "saved", runId: 45 });
  });

  it("selecting a source clears run-bound state and returns to live source mode", () => {
    const next = selectSourceWorkspace(baseState({
      openRunState: { kind: "saved", runId: 42 },
      canvasMode: "report",
      sourceViewBasis: "run_snapshot",
      companionTab: "evidence",
      selectedTraceRef: "s7-i1",
    }), 8);

    expect(next).toEqual({
      workspaceSelection: { kind: "source", sourceId: 8 },
      openRunState: { kind: "none" },
      canvasMode: "source",
      sourceViewBasis: "live_source",
      companionTab: "runs",
      selectedTraceRef: null,
    });
  });

  it("selecting a source group clears run-bound state and returns to live source mode", () => {
    const next = selectSourceGroupWorkspace(baseState({
      openRunState: { kind: "active", runId: 42 },
      canvasMode: "report",
      sourceViewBasis: "run_snapshot",
      companionTab: "chat",
      selectedTraceRef: "s7-i1",
    }), 10);

    expect(next).toEqual({
      workspaceSelection: { kind: "source_group", sourceGroupId: 10 },
      openRunState: { kind: "none" },
      canvasMode: "source",
      sourceViewBasis: "live_source",
      companionTab: "runs",
      selectedTraceRef: null,
    });
  });

  it("normalizes restored run-bound UI state when no run is open", () => {
    expect(normalizeRestoredWorkspaceState(baseState({
      openRunState: { kind: "none" },
      canvasMode: "report",
      sourceViewBasis: "run_snapshot",
      companionTab: "chat",
      selectedTraceRef: "s7-i1",
    }))).toEqual({
      workspaceSelection: { kind: "none" },
      openRunState: { kind: "none" },
      canvasMode: "report",
      sourceViewBasis: "live_source",
      companionTab: "runs",
      selectedTraceRef: null,
    });

    expect(normalizeRestoredWorkspaceState(baseState({
      openRunState: { kind: "none" },
      canvasMode: "report",
      sourceViewBasis: "run_snapshot",
      companionTab: "chunks",
      selectedTraceRef: "s7-i1",
    }))).toEqual({
      workspaceSelection: { kind: "none" },
      openRunState: { kind: "none" },
      canvasMode: "report",
      sourceViewBasis: "live_source",
      companionTab: "runs",
      selectedTraceRef: null,
    });

    expect(normalizeRestoredWorkspaceState(baseState({
      openRunState: { kind: "saved", runId: 42 },
      sourceViewBasis: "run_snapshot",
      companionTab: "evidence",
      selectedTraceRef: "s7-i1",
    }))).toMatchObject({
      openRunState: { kind: "saved", runId: 42 },
      sourceViewBasis: "run_snapshot",
      companionTab: "evidence",
      selectedTraceRef: "s7-i1",
    });

    expect(normalizeRestoredWorkspaceState(baseState({
      openRunState: { kind: "active", runId: 42 },
      sourceViewBasis: "run_snapshot",
      companionTab: "chunks",
    }))).toMatchObject({
      openRunState: { kind: "active", runId: 42 },
      sourceViewBasis: "run_snapshot",
      companionTab: "chunks",
    });
  });
});
