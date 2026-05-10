import { describe, expect, it } from "vitest";
import {
  ANALYSIS_WORKSPACE_STATE_KEY,
  fallbackWorkspaceSelection,
  loadPersistedAnalysisWorkspaceState,
  persistableAnalysisWorkspaceState,
  restoredUiStateFromPersisted,
  savePersistedAnalysisWorkspaceState,
} from "./analysis-workspace-persistence";
import {
  defaultAnalysisWorkspaceUiState,
  type AnalysisWorkspaceUiState,
  type WorkspaceSelection,
} from "./analysis-workspace-state";
import type { AnalysisSourceGroup } from "./types/analysis";
import type { Source } from "./types/sources";

class MemoryStorage {
  values = new Map<string, string>();

  getItem(key: string) {
    return this.values.get(key) ?? null;
  }

  setItem(key: string, value: string) {
    this.values.set(key, value);
  }

  removeItem(key: string) {
    this.values.delete(key);
  }
}

function source(id: number): Source {
  return {
    id,
    sourceType: "telegram",
    sourceSubtype: "channel",
    telegramSourceKind: "channel",
    accountId: 1,
    externalId: `source-${id}`,
    title: `Source ${id}`,
    lastSyncState: null,
    lastSyncedAt: null,
    isMember: true,
    isActive: true,
    createdAt: 100,
    avatarDataUrl: null,
  };
}

function group(id: number): AnalysisSourceGroup {
  return {
    id,
    name: `Group ${id}`,
    source_type: "telegram",
    members: [],
    created_at: 100,
    updated_at: 100,
  };
}

function uiState(overrides: Partial<AnalysisWorkspaceUiState> = {}): AnalysisWorkspaceUiState {
  return {
    ...defaultAnalysisWorkspaceUiState(),
    ...overrides,
  };
}

describe("analysis-workspace-persistence", () => {
  it("serializes only durable workspace state", () => {
    const persisted = persistableAnalysisWorkspaceState(
      uiState({
        workspaceSelection: { kind: "source", sourceId: 7 },
        openRunState: { kind: "saved", runId: 42 },
        canvasMode: "report",
        sourceViewBasis: "run_snapshot",
        companionTab: "evidence",
        selectedTraceRef: "s7-i1",
      }),
      {
        historyScope: "current",
        runFilter: "completed",
      },
    );

    expect(persisted).toEqual({
      version: 1,
      workspaceSelection: { kind: "source", sourceId: 7 },
      canvasMode: "report",
      sourceViewBasis: "run_snapshot",
      companionTab: "evidence",
      runs: {
        historyScope: "current",
        runFilter: "completed",
      },
    });
    expect(JSON.stringify(persisted)).not.toContain("openRunState");
    expect(JSON.stringify(persisted)).not.toContain("selectedTraceRef");
    expect(JSON.stringify(persisted)).not.toContain("runId");
  });

  it("saves and loads persisted workspace state through the configured key", () => {
    const storage = new MemoryStorage();
    const persisted = persistableAnalysisWorkspaceState(
      uiState({
        workspaceSelection: { kind: "source_group", sourceGroupId: 9 },
        canvasMode: "source",
        sourceViewBasis: "live_source",
        companionTab: "runs",
      }),
      {
        historyScope: "all",
        runFilter: "failed",
      },
    );

    savePersistedAnalysisWorkspaceState(storage, persisted);

    expect(storage.getItem(ANALYSIS_WORKSPACE_STATE_KEY)).toContain('"version":1');
    expect(loadPersistedAnalysisWorkspaceState(storage)).toEqual(persisted);
  });

  it("rejects malformed, unsupported, or invalid persisted state", () => {
    const storage = new MemoryStorage();

    expect(loadPersistedAnalysisWorkspaceState(storage)).toBeNull();

    storage.setItem(ANALYSIS_WORKSPACE_STATE_KEY, "not json");
    expect(loadPersistedAnalysisWorkspaceState(storage)).toBeNull();

    storage.setItem(ANALYSIS_WORKSPACE_STATE_KEY, JSON.stringify({ version: 2 }));
    expect(loadPersistedAnalysisWorkspaceState(storage)).toBeNull();

    storage.setItem(ANALYSIS_WORKSPACE_STATE_KEY, JSON.stringify({
      version: 1,
      workspaceSelection: { kind: "source", sourceId: -1 },
      canvasMode: "report",
      sourceViewBasis: "live_source",
      companionTab: "runs",
      runs: { historyScope: "all", runFilter: "all" },
    }));
    expect(loadPersistedAnalysisWorkspaceState(storage)).toBeNull();
  });

  it("normalizes restored UI state because OpenRunState is never persisted", () => {
    const restored = restoredUiStateFromPersisted({
      version: 1,
      workspaceSelection: { kind: "source", sourceId: 7 },
      canvasMode: "report",
      sourceViewBasis: "run_snapshot",
      companionTab: "chat",
      runs: {
        historyScope: "current",
        runFilter: "completed",
      },
    });

    expect(restored).toEqual({
      workspaceSelection: { kind: "source", sourceId: 7 },
      openRunState: { kind: "none" },
      canvasMode: "report",
      sourceViewBasis: "live_source",
      companionTab: "runs",
      selectedTraceRef: null,
    });
  });

  it("falls back gracefully when persisted source or group ids are stale", () => {
    expect(fallbackWorkspaceSelection(
      { kind: "source", sourceId: 99 },
      [source(7), source(8)],
      [group(9)],
    )).toEqual({ kind: "source", sourceId: 7 });

    expect(fallbackWorkspaceSelection(
      { kind: "source_group", sourceGroupId: 99 },
      [source(7)],
      [group(9)],
    )).toEqual({ kind: "source_group", sourceGroupId: 9 });

    expect(fallbackWorkspaceSelection(
      { kind: "none" },
      [source(7)],
      [group(9)],
    )).toEqual({ kind: "source", sourceId: 7 });

    expect(fallbackWorkspaceSelection(
      { kind: "source", sourceId: 99 },
      [],
      [],
    )).toEqual({ kind: "none" });
  });

  it("preserves valid persisted source and group selections", () => {
    const sourceSelection: WorkspaceSelection = { kind: "source", sourceId: 8 };
    const groupSelection: WorkspaceSelection = { kind: "source_group", sourceGroupId: 9 };

    expect(fallbackWorkspaceSelection(sourceSelection, [source(7), source(8)], [group(9)]))
      .toEqual(sourceSelection);
    expect(fallbackWorkspaceSelection(groupSelection, [source(7)], [group(9)]))
      .toEqual(groupSelection);
  });
});
