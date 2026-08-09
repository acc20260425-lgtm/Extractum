import { describe, expect, it } from "vitest";
import { runTargetLabel } from "./analysis-utils";
import {
  sourceBasisLabel,
  sourceCanvasSurface,
} from "./analysis-report-canvas-state";
import {
  chatAvailabilityForRun,
  filterCompanionRuns,
  runsFilterDefaults,
} from "./analysis-run-companion-state";
import {
  defaultAnalysisWorkspaceUiState,
  transitionAnalysisWorkspaceState,
} from "./analysis-workspace-state";
import type { AnalysisRunDetail } from "./types/analysis";

function run(overrides: Partial<AnalysisRunDetail> = {}): AnalysisRunDetail {
  return {
    id: 42,
    run_type: "report",
    scope_type: "single_source",
    source_id: 7,
    source_title: "Telegram A",
    source_group_id: null,
    source_group_name: null,
    scope_label: "Telegram A snapshot",
    period_from: 1710000000,
    period_to: 1710100000,
    output_language: "Russian",
    prompt_template_id: 1,
    prompt_template_name: "Weekly",
    prompt_template_version: 3,
    provider_profile: "default",
    provider: "openai",
    model: "gpt-5.4",
    youtube_corpus_mode: "transcript_description",
    telegram_history_scope: "current",
    status: "completed",
    result_markdown: "Saved report",
    error: null,
    has_trace_data: true,
    snapshot_state: "captured",
    snapshot_captured_at: "2026-05-18T10:00:00Z",
    snapshot_error: null,
    created_at: 1710100010,
    completed_at: 1710100100,
    ...overrides,
  };
}

function openedRunState(liveScopeExists = true) {
  return transitionAnalysisWorkspaceState(defaultAnalysisWorkspaceUiState(), {
    type: "open_run",
    run: {
      runId: 42,
      status: "completed",
      sourceId: 7,
      sourceGroupId: null,
      liveScopeExists,
    },
  });
}

describe("analysis redesign final safety contract", () => {
  it("keeps run snapshot and live source basis explicit in Source mode", () => {
    const opened = openedRunState();
    const snapshot = transitionAnalysisWorkspaceState(opened, {
      type: "change_canvas_mode",
      canvasMode: "source",
    });
    const live = transitionAnalysisWorkspaceState(snapshot, {
      type: "view_live_source_for_opened_run",
    });

    expect(opened.sourceViewBasis).toBe("run_snapshot");
    expect(opened.canvasMode).toBe("report");
    expect(snapshot.canvasMode).toBe("source");
    expect(snapshot.sourceViewBasis).toBe("run_snapshot");
    expect(live.canvasMode).toBe("source");
    expect(live.sourceViewBasis).toBe("live_source");
    expect(sourceCanvasSurface({ currentRun: run(), sourceViewBasis: snapshot.sourceViewBasis, snapshotAvailability: "available" })).toBe("run_snapshot_available");
    expect(sourceBasisLabel({ currentRun: run(), sourceViewBasis: snapshot.sourceViewBasis, snapshotAvailability: "available" })).toBe("Snapshot available");
    expect(sourceCanvasSurface({ currentRun: run(), sourceViewBasis: live.sourceViewBasis, snapshotAvailability: "available" })).toBe("live_source");
  });

  it("gates completed-run chat on saved run context instead of live source context", () => {
    const ready = chatAvailabilityForRun({
      currentRun: run(),
      snapshotAvailability: "available",
      snapshotProbeState: "available",
    });
    const noRun = chatAvailabilityForRun({
      currentRun: null,
      snapshotAvailability: "unknown",
      snapshotProbeState: "unknown",
    });
    const active = chatAvailabilityForRun({
      currentRun: run({ status: "running", completed_at: null }),
      snapshotAvailability: "capturing",
      snapshotProbeState: "unknown",
    });
    const missingReport = chatAvailabilityForRun({
      currentRun: run({ result_markdown: null }),
      snapshotAvailability: "available",
      snapshotProbeState: "available",
    });
    const unavailable = chatAvailabilityForRun({
      currentRun: run(),
      snapshotAvailability: "unavailable",
      snapshotProbeState: "unavailable",
    });

    expect(ready.enabled).toBe(true);
    expect(ready.reason).toBe("enabled");
    expect(ready.title).toBe("Chat ready");
    expect(ready.description).toContain("saved report and saved run snapshot context");
    expect(noRun.reason).toBe("no_run");
    expect(active.reason).toBe("pending_completion");
    expect(missingReport.reason).toBe("missing_report");
    expect(unavailable.enabled).toBe(false);
    expect(unavailable.reason).toBe("inconsistent");
  });

  it("keeps missing or deleted run scope labeling visible in the run header", () => {
    const opened = openedRunState(false);

    expect(opened.workspaceSelection).toEqual({ kind: "none" });
    expect(opened.openRunState).toEqual({ kind: "saved", runId: 42 });
    expect(opened.canvasMode).toBe("report");
    expect(opened.sourceViewBasis).toBe("run_snapshot");
    expect(opened.companionTab).toBe("evidence");
    expect(opened.selectedTraceRef).toBeNull();
    expect(runTargetLabel(run({ scope_label: "Deleted source snapshot", source_title: null }))).toBe("Deleted source snapshot");
    expect(runTargetLabel(run({ scope_label: "", source_title: "Live source title" }))).toBe("Live source title");
    expect(runTargetLabel(run({ scope_label: "", source_title: null, source_id: 9 }))).toBe("Source 9");
    expect(runTargetLabel(run({ scope_type: "source_group", scope_label: "", source_id: null, source_group_id: 3, source_group_name: "Research group" }))).toBe("Research group");
    expect(runTargetLabel(run({ scope_type: "source_group", scope_label: "", source_id: null, source_group_id: 3, source_group_name: null }))).toBe("Group 3");
    expect(runTargetLabel(run({ scope_type: "project", scope_label: "", source_id: null, project_id: 8, project_name: null }))).toBe("Project #8");
  });

  it("does not hide completed chat persistence failures", () => {
    const unavailable = chatAvailabilityForRun({
      currentRun: run({
        snapshot_state: "capture_failed",
        snapshot_captured_at: null,
        snapshot_error: "sqlite write failed",
      }),
      snapshotAvailability: "unavailable",
      snapshotProbeState: "unavailable",
    });

    expect(unavailable.reason).toBe("capture_failed_with_error");
    expect(unavailable.description).toContain("could not save the frozen source context");
  });

  it("uses stable run filter normalization instead of locale-sensitive casing", () => {
    const savedRuns = [run({ scope_label: "Istanbul Analysis", provider: "OPENAI" })];
    const upper = filterCompanionRuns({
      activeRuns: [],
      savedRuns,
      filter: { ...runsFilterDefaults(), query: "ISTANBUL", provider: "openai" },
      workspaceSelection: { kind: "none" },
    });
    const lower = filterCompanionRuns({
      activeRuns: [],
      savedRuns,
      filter: { ...runsFilterDefaults(), query: "istanbul", provider: "OPENAI" },
      workspaceSelection: { kind: "none" },
    });

    expect(upper.map((entry) => entry.run.id)).toEqual([42]);
    expect(lower.map((entry) => entry.run.id)).toEqual([42]);
  });
});
