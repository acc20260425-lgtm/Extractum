import { describe, expect, it } from "vitest";
import {
  defaultAnalysisWorkspaceUiState,
  transitionAnalysisWorkspaceState,
} from "./analysis-workspace-state";
import {
  hasActiveCompanionRunsFilter,
  runsFilterDefaults,
} from "./analysis-run-companion-state";

const completedRun = {
  runId: 7,
  status: "completed",
  sourceId: 10,
  sourceGroupId: null,
};

describe("analysis redesign final route contract", () => {
  it("keeps source switching, run opening, and canvas switching on separate state paths", () => {
    const initial = defaultAnalysisWorkspaceUiState();
    const source = transitionAnalysisWorkspaceState(initial, {
      type: "select_source",
      sourceId: 10,
    });
    const group = transitionAnalysisWorkspaceState(initial, {
      type: "select_source_group",
      sourceGroupId: 20,
    });
    const opened = transitionAnalysisWorkspaceState(initial, {
      type: "open_run",
      run: completedRun,
    });
    const sourceCanvas = transitionAnalysisWorkspaceState(opened, {
      type: "change_canvas_mode",
      canvasMode: "source",
    });

    expect(source.workspaceSelection).toEqual({ kind: "source", sourceId: 10 });
    expect(source.openRunState).toEqual({ kind: "none" });
    expect(source.canvasMode).toBe("source");
    expect(source.sourceViewBasis).toBe("live_source");
    expect(group.workspaceSelection).toEqual({ kind: "source_group", sourceGroupId: 20 });
    expect(group.openRunState).toEqual({ kind: "none" });
    expect(opened.workspaceSelection).toEqual({ kind: "source", sourceId: 10 });
    expect(opened.openRunState).toEqual({ kind: "saved", runId: 7 });
    expect(opened.canvasMode).toBe("report");
    expect(opened.sourceViewBasis).toBe("run_snapshot");
    expect(sourceCanvas.openRunState).toEqual({ kind: "saved", runId: 7 });
    expect(sourceCanvas.canvasMode).toBe("source");
  });

  it("keeps Runs focused on analysis report runs and durable filters", () => {
    const defaults = runsFilterDefaults();

    expect(defaults.query).toBe("");
    expect(defaults.status).toBe("all");
    expect(defaults.scope).toBe("all");
    expect(defaults.dateFrom).toBe("");
    expect(defaults.dateTo).toBe("");
    expect(defaults.provider).toBe("");
    expect(defaults.model).toBe("");
    expect(defaults.template).toBe("");
    expect(hasActiveCompanionRunsFilter(defaults)).toBe(false);
  });
});
