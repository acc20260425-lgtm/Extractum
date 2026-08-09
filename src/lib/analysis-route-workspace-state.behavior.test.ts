import { describe, expect, it, vi } from "vitest";
import {
  persistWorkspaceWhenReady,
  restoreWorkspaceBeforeActiveRuns,
} from "$lib/analysis-route-workspace-runtime";
import {
  defaultAnalysisWorkspaceUiState,
  transitionAnalysisWorkspaceState,
} from "$lib/analysis-workspace-state";
import { runsFilterDefaults } from "$lib/analysis-run-companion-state";

describe("analysis route workspace state", () => {
  it("restores persisted workspace state before loading active runs", async () => {
    const calls: string[] = [];
    const result = await restoreWorkspaceBeforeActiveRuns({
      restore: () => calls.push("restore"),
      loadSourcesAndGroups: async () => { calls.push("catalog"); },
      applyRestoredSelection: async () => { calls.push("selection"); return true; },
      loadActiveRuns: async () => { calls.push("runs"); },
    });

    expect(calls[0]).toBe("restore");
    expect(calls[1]).toBe("catalog");
    expect(calls[2]).toBe("selection");
    expect(calls[3]).toBe("runs");
    expect(result.restoredSelectionApplied).toBe(true);
  });

  it("persists durable workspace state and excludes run-bound transient state", () => {
    const save = vi.fn();
    const state = {
      ...defaultAnalysisWorkspaceUiState(),
      workspaceSelection: { kind: "source" as const, sourceId: 17 },
      openRunState: { kind: "saved" as const, runId: 91 },
      canvasMode: "report" as const,
      sourceViewBasis: "run_snapshot" as const,
      companionTab: "chat" as const,
      selectedTraceRef: "ref:1",
    };
    const persisted = persistWorkspaceWhenReady({
      ready: true,
      state,
      runs: { historyScope: "current", runFilter: "completed", runsFilter: runsFilterDefaults() },
      save,
    });

    expect(persisted).not.toBeNull();
    expect(persisted?.workspaceSelection).toEqual({ kind: "source", sourceId: 17 });
    expect(persisted?.canvasMode).toBe("report");
    expect(persisted?.sourceViewBasis).toBe("run_snapshot");
    expect(persisted?.companionTab).toBe("chat");
    expect(Object.hasOwn(persisted!, "openRunState")).toBe(false);
    expect(Object.hasOwn(persisted!, "selectedTraceRef")).toBe(false);
    expect(save).toHaveBeenCalledTimes(1);
    expect(save).toHaveBeenCalledWith(persisted);
  });

  it("uses workspace transition events for source, group, and run opening", () => {
    const initial = defaultAnalysisWorkspaceUiState();
    const source = transitionAnalysisWorkspaceState(initial, { type: "select_source", sourceId: 17 });
    const group = transitionAnalysisWorkspaceState(source, { type: "select_source_group", sourceGroupId: 23 });
    const run = transitionAnalysisWorkspaceState(group, {
      type: "open_run",
      run: { runId: 91, status: "completed", sourceId: 17, sourceGroupId: null },
    });

    expect(source.workspaceSelection).toEqual({ kind: "source", sourceId: 17 });
    expect(source.openRunState).toEqual({ kind: "none" });
    expect(group.workspaceSelection).toEqual({ kind: "source_group", sourceGroupId: 23 });
    expect(group.openRunState).toEqual({ kind: "none" });
    expect(run.workspaceSelection).toEqual({ kind: "source", sourceId: 17 });
    expect(run.openRunState).toEqual({ kind: "saved", runId: 91 });
  });

  it("saves workspace state from a guarded effect after restore is complete", () => {
    const save = vi.fn();
    const input = { state: defaultAnalysisWorkspaceUiState(), runs: { historyScope: "all" as const, runFilter: "all" as const, runsFilter: runsFilterDefaults() }, save };

    expect(persistWorkspaceWhenReady({ ...input, ready: false })).toBeNull();
    expect(persistWorkspaceWhenReady({ ...input, ready: true })).not.toBeNull();
  });
});
