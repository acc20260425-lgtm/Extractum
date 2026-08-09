import { describe, expect, it, vi } from "vitest";
import {
  compactSourceRailRouteProps,
} from "$lib/analysis-route-runtime";
import { analysisRouteComponents } from "$lib/analysis-route-components";
import CompactSourceRail from "$lib/components/analysis/compact-source-rail.svelte";
import ReportCanvas from "$lib/components/analysis/report-canvas.svelte";
import RunCompanionTabs from "$lib/components/analysis/run-companion-tabs.svelte";
import { runCompanionRouteProps } from "$lib/analysis-run-companion-route-runtime";
import { defaultAnalysisWorkspaceUiState } from "$lib/analysis-workspace-state";
import { runsFilterDefaults } from "$lib/analysis-run-companion-state";

describe("analysis source access placement", () => {
  it("uses the compact source rail inside the analysis route", () => {
    const onSelectSource = vi.fn();
    const onSelectGroup = vi.fn();
    const onStartMigratedHistoryImport = vi.fn();
    const startingMigratedHistorySourceIds = { 17: true };
    const sourceJobsBySource = { 17: [{ id: "job-1" }] };
    const props = compactSourceRailRouteProps({
      workspaceSelection: { kind: "source", sourceId: 17 },
      startingMigratedHistorySourceIds,
      sourceJobsBySource,
      onSelectSource,
      onSelectGroup,
      onStartMigratedHistoryImport,
    });

    expect(analysisRouteComponents.sourceRail).toBe(CompactSourceRail);
    expect(props.workspaceSelection).toEqual({ kind: "source", sourceId: 17 });
    expect(props.onSelectSource).toBe(onSelectSource);
    expect(props.onSelectGroup).toBe(onSelectGroup);
    expect(props.onStartMigratedHistoryImport).toBe(onStartMigratedHistoryImport);
    expect(props.railData.startingMigratedHistorySourceIds).toBe(startingMigratedHistorySourceIds);
    expect(props.railData.sourceJobsBySource).toBe(sourceJobsBySource);
    expect(Object.hasOwn(props, "workspaceRail")).toBe(false);
    expect(Object.hasOwn(props, "sourceJobs")).toBe(false);
  });

  it("does not place source ingest jobs in the analysis runs companion", () => {
    const props = runCompanionRouteProps({ workspaceUiState: defaultAnalysisWorkspaceUiState(), focusedChunkSummaries: [], selectedRunIsActive: false, activeRuns: [], savedRuns: [], runsFilter: runsFilterDefaults() });

    expect(Object.hasOwn(props, "sourceJobs")).toBe(false);
    expect(Object.hasOwn(props, "takeoutJobs")).toBe(false);
    expect(Object.hasOwn(props, "sourceJobsBySource")).toBe(false);
  });

  it("keeps the left analysis column compact while rendering the run companion", () => {
    expect(analysisRouteComponents.sourceRail).toBe(CompactSourceRail);
    expect(analysisRouteComponents.reportCanvas).toBe(ReportCanvas);
    expect(analysisRouteComponents.runCompanion).toBe(RunCompanionTabs);
    expect(analysisRouteComponents.reportCanvas).not.toBe(analysisRouteComponents.sourceRail);
    expect(new Set(Object.values(analysisRouteComponents)).size).toBe(3);
  });
});
