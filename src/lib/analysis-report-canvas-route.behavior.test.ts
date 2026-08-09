import { describe, expect, it, vi } from "vitest";
import {
  loadRunSnapshotPage,
  refreshCatalogForTerminalSourceJob,
  reportCanvasRouteProps,
  reportLaunchPreflightProps,
  returnToEvidenceReview,
  shouldLoadRunSnapshot,
} from "$lib/analysis-route-runtime";
import { analysisRouteComponents } from "$lib/analysis-route-components";
import CompactSourceRail from "$lib/components/analysis/compact-source-rail.svelte";
import ReportCanvas from "$lib/components/analysis/report-canvas.svelte";
import RunCompanionTabs from "$lib/components/analysis/run-companion-tabs.svelte";

describe("analysis route report canvas wiring", () => {
  it("renders ReportCanvas with RunCompanionTabs instead of legacy workspace panels", () => {
    expect(analysisRouteComponents.sourceRail).toBe(CompactSourceRail);
    expect(analysisRouteComponents.reportCanvas).toBe(ReportCanvas);
    expect(analysisRouteComponents.runCompanion).toBe(RunCompanionTabs);
    expect(Object.hasOwn(analysisRouteComponents, "workspaceRail")).toBe(false);
    expect(Object.hasOwn(analysisRouteComponents, "workspaceMain")).toBe(false);
    expect(Object.hasOwn(analysisRouteComponents, "workspaceInspector")).toBe(false);
    expect(Object.keys(analysisRouteComponents)).toEqual(["sourceRail", "reportCanvas", "runCompanion"]);
    expect(new Set(Object.values(analysisRouteComponents)).size).toBe(3);
  });

  it("passes persisted canvas mode and source basis from workspace UI state", () => {
    const onChangeCanvasMode = vi.fn();
    const onViewLiveSource = vi.fn();
    const onBackToRunSnapshot = vi.fn();
    const props = reportCanvasRouteProps({
      workspaceUiState: {
        workspaceSelection: { kind: "source", sourceId: 7 },
        openRunState: { kind: "saved", runId: 11 },
        canvasMode: "source",
        sourceViewBasis: "live_source",
        companionTab: "evidence",
        selectedTraceRef: null,
      },
      sourceReturnContext: null,
      onChangeCanvasMode,
      onViewLiveSource,
      onBackToRunSnapshot,
      onReturnToEvidenceReview: vi.fn(),
    });

    expect(props.canvasMode).toBe("source");
    expect(props.sourceViewBasis).toBe("live_source");
    expect(props.runSnapshotAvailability).toBe("unknown");
    expect(props.snapshotProbeState).toBe("unknown");
    expect(props.loadingRunSnapshotMessages).toBe(false);
    expect(props.onChangeCanvasMode).toBe(onChangeCanvasMode);
    expect(props.onViewLiveSource).toBe(onViewLiveSource);
    expect(props.onBackToRunSnapshot).toBe(onBackToRunSnapshot);
  });

  it("passes scoped evidence return state and callback into the report canvas", () => {
    const callback = vi.fn();
    const context = { kind: "evidence" as const, runId: 11, traceRef: "ref:1", sourceScope: { kind: "source" as const, sourceId: 7 }, sourceViewBasis: "run_snapshot" as const };
    const props = reportCanvasRouteProps({
      workspaceUiState: {
        workspaceSelection: { kind: "source", sourceId: 7 },
        openRunState: { kind: "saved", runId: 11 },
        canvasMode: "source",
        sourceViewBasis: "run_snapshot",
        companionTab: "evidence",
        selectedTraceRef: "ref:1",
      },
      sourceReturnContext: context,
      onChangeCanvasMode: vi.fn(),
      onViewLiveSource: vi.fn(),
      onBackToRunSnapshot: vi.fn(),
      onReturnToEvidenceReview: callback,
    });

    expect(props.sourceReturnContext).toBe(context);
    expect(props.onReturnToEvidenceReview).toBe(callback);
  });

  it("returns to Evidence only through the active source return context", () => {
    const active = { kind: "evidence" as const, runId: 11, traceRef: "ref:1", sourceScope: { kind: "source" as const, sourceId: 7 }, sourceViewBasis: "run_snapshot" as const };
    const clearPendingFocus = vi.fn();
    const clearHighlight = vi.fn();
    const dispatch = vi.fn();
    const clearReturnContext = vi.fn();

    const inactiveResult = returnToEvidenceReview({
      activeContext: null,
      clearPendingFocus,
      clearHighlight,
      dispatch,
      clearReturnContext,
    });
    const activeResult = returnToEvidenceReview({
      activeContext: active,
      clearPendingFocus,
      clearHighlight,
      dispatch,
      clearReturnContext,
    });

    expect(inactiveResult).toBe(false);
    expect(activeResult).toBe(true);
    expect(clearPendingFocus).toHaveBeenCalledTimes(1);
    expect(clearHighlight).toHaveBeenCalledTimes(1);
    expect(dispatch).toHaveBeenCalledTimes(1);
    expect(dispatch).toHaveBeenCalledWith({ type: "return_to_evidence_review", traceRef: "ref:1" });
    expect(clearReturnContext).toHaveBeenCalledTimes(1);
    expect(clearPendingFocus).toHaveBeenCalledBefore(clearHighlight);
    expect(clearHighlight).toHaveBeenCalledBefore(dispatch);
    expect(dispatch).toHaveBeenCalledBefore(clearReturnContext);
    expect(active.traceRef).toBe("ref:1");
  });

  it("loads run snapshot messages through the snapshot-only API", async () => {
    const listMessages = vi.fn().mockResolvedValue({ messages: [], next_cursor: null, has_more: false });
    const page = await loadRunSnapshotPage({
      runId: 11,
      sourceId: 7,
      aroundRef: "ref:1",
      after: null,
      limit: 50,
      listMessages,
    });

    expect(listMessages).toHaveBeenCalledTimes(1);
    expect(listMessages).toHaveBeenCalledWith({ runId: 11, sourceId: 7, aroundRef: "ref:1", after: null, limit: 50 });
    expect(page.messages).toEqual([]);
    expect(page.next_cursor).toBeNull();
    expect(page.has_more).toBe(false);
    expect(Object.keys(listMessages.mock.calls[0][0])).toContain("runId");
    expect(Object.keys(listMessages.mock.calls[0][0])).not.toContain("beforePublishedAt");
    expect(Object.keys(listMessages.mock.calls[0][0])).not.toContain("topicFilter");
  });

  it("derives snapshot probe state and passes it into the report canvas", () => {
    const props = reportCanvasRouteProps({
      workspaceUiState: {
        workspaceSelection: { kind: "none" }, openRunState: { kind: "saved", runId: 11 },
        canvasMode: "report", sourceViewBasis: "run_snapshot", companionTab: "evidence", selectedTraceRef: null,
      },
      snapshot: { availability: "available", loading: false, error: "" },
      sourceReturnContext: null,
      onChangeCanvasMode: vi.fn(), onViewLiveSource: vi.fn(), onBackToRunSnapshot: vi.fn(), onReturnToEvidenceReview: vi.fn(),
    });

    expect(props.runSnapshotAvailability).toBe("available");
    expect(props.snapshotProbeState).toBe("available");
    expect(props.loadingRunSnapshotMessages).toBe(false);
    expect(props.runSnapshotError).toBe("");
    expect(Object.keys(props)).toContain("snapshotProbeState");
    expect(Object.keys(props)).toContain("runSnapshotAvailability");
  });

  it("does not switch back to snapshot automatically when the user explicitly views live source", () => {
    expect(shouldLoadRunSnapshot({ runId: 11, sourceViewBasis: "run_snapshot", lastSnapshotLoadKey: "" })).toBe(true);
    expect(shouldLoadRunSnapshot({ runId: 11, sourceViewBasis: "live_source", lastSnapshotLoadKey: "" })).toBe(false);
    expect(shouldLoadRunSnapshot({ runId: 11, sourceViewBasis: "run_snapshot", lastSnapshotLoadKey: "11:first" })).toBe(false);
    expect(shouldLoadRunSnapshot({ runId: null, sourceViewBasis: "run_snapshot", lastSnapshotLoadKey: "" })).toBe(false);
  });

  it("passes source metrics into report launch preflight", () => {
    const metric = { source_id: 7, item_count: 42 };
    const props = reportLaunchPreflightProps(metric, "Source is still syncing.");

    expect(props.currentSourceMetric).toBe(metric);
    expect(props.reportLaunchDisabledReason).toBe("Source is still syncing.");
  });

  it("refreshes source metrics after terminal YouTube source jobs for report preflight", async () => {
    const loadSourceCatalog = vi.fn().mockResolvedValue(undefined);
    const loadGroups = vi.fn().mockResolvedValue(undefined);

    const refreshed = await refreshCatalogForTerminalSourceJob({ status: "completed", source_id: 7 }, 7, { loadSourceCatalog, loadGroups });

    expect(refreshed).toBe(true);
    expect(loadSourceCatalog).toHaveBeenCalledTimes(1);
    expect(loadGroups).toHaveBeenCalledTimes(1);
  });
});
