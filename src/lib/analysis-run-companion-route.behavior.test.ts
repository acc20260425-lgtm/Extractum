import { describe, expect, it, vi } from "vitest";
import {
  changeCompanionTab,
  runCompanionRouteProps,
  runIdFromHref,
  showEvidenceInSource,
  submitCompanionQuestion,
} from "$lib/analysis-run-companion-route-runtime";
import { defaultAnalysisWorkspaceUiState } from "$lib/analysis-workspace-state";
import { runsFilterDefaults } from "$lib/analysis-run-companion-state";
import type { AnalysisRunDetail } from "$lib/types/analysis";
import { analysisRouteComponents } from "$lib/analysis-route-components";
import RunCompanionTabs from "$lib/components/analysis/run-companion-tabs.svelte";

const completedRun = {
  id: 91,
  status: "completed",
  source_id: 17,
  source_group_id: null,
  result_markdown: "Saved report",
  snapshot_state: "available",
  snapshot_captured_at: 1_700_000_000,
  snapshot_error: null,
} as unknown as AnalysisRunDetail;

describe("analysis route run companion wiring", () => {
  it("renders RunCompanionTabs instead of WorkspaceInspector", () => {
    const props = runCompanionRouteProps({
      workspaceUiState: defaultAnalysisWorkspaceUiState(), focusedChunkSummaries: [], selectedRunIsActive: false,
      activeRuns: [], savedRuns: [], runsFilter: runsFilterDefaults(),
    });

    expect(analysisRouteComponents.runCompanion).toBe(RunCompanionTabs);
    expect(props).toHaveProperty("companionTab", "runs");
    expect(Object.hasOwn(analysisRouteComponents, "workspaceInspector")).toBe(false);
    expect(Object.hasOwn(props, "inspectorMode")).toBe(false);
  });

  it("uses workspaceUiState.companionTab as the only companion tab source", () => {
    const state = { ...defaultAnalysisWorkspaceUiState(), openRunState: { kind: "saved" as const, runId: 91 }, companionTab: "chat" as const };
    const props = runCompanionRouteProps({ workspaceUiState: state, focusedChunkSummaries: [], selectedRunIsActive: false, activeRuns: [], savedRuns: [], runsFilter: runsFilterDefaults() });

    expect(props.companionTab).toBe("chat");
    expect(Object.hasOwn(props, "inspectorMode")).toBe(false);
    expect(Object.hasOwn(props, "defaultTab")).toBe(false);
    expect(Object.hasOwn(props, "selectedTab")).toBe(false);
    expect(Object.keys(props).filter((key) => key.toLowerCase().includes("tab"))).toEqual(["companionTab"]);
    expect(changeCompanionTab(state, "evidence").companionTab).toBe("evidence");
  });

  it("passes focused chunk summaries into the companion without auto-opening chunks", () => {
    const chunks = [{ index: 1, total: 2 }, { index: 2, total: 2 }];
    const state = { ...defaultAnalysisWorkspaceUiState(), companionTab: "evidence" as const };
    const props = runCompanionRouteProps({ workspaceUiState: state, focusedChunkSummaries: chunks, selectedRunIsActive: true, activeRuns: [], savedRuns: [], runsFilter: runsFilterDefaults() });

    expect(props.focusedChunkSummaries).toBe(chunks);
    expect(props.selectedRunIsActive).toBe(true);
    expect(props.companionTab).toBe("evidence");
    expect(props.companionTab).not.toBe("chunks");
  });

  it("activates Evidence for trace clicks and Show in source prefers snapshot", async () => {
    const calls: string[] = [];
    const dispatch = vi.fn((event) => calls.push(`dispatch:${event.type}`));
    const loadedInputs: Array<{ sourceScope: unknown; canonicalRef: string; requestId: string }> = [];
    const loadSourceWindow = vi.fn(async (input: { sourceScope: unknown; canonicalRef: string; requestId: string }) => {
      loadedInputs.push(input);
      calls.push("load");
    });
    const outcome = await showEvidenceInSource({
      currentRun: completedRun,
      selectedTrace: { ref: "ref:1", source_id: 17 } as never,
      snapshotAvailability: "available",
      snapshotProbeState: "available",
      sourceScope: { kind: "source", sourceId: 17 },
      nextRequestId: () => "request-1",
      clearHighlight: () => calls.push("clear"),
      setReturnContext: () => calls.push("return-context"),
      setPendingFocus: () => calls.push("pending"),
      dispatch,
      loadSourceWindow,
    });

    expect(outcome.kind).toBe("started");
    expect(outcome.decision.kind).toBe("run_snapshot");
    expect(outcome.decision).toHaveProperty("sourceViewBasis", "run_snapshot");
    expect(outcome.requestId).toBe("request-1");
    expect(outcome.canonicalRef).toBe("ref:1");
    expect(dispatch).toHaveBeenCalledWith({ type: "show_evidence_in_source", sourceViewBasis: "run_snapshot", highlightedRef: "ref:1" });
    expect(loadSourceWindow).toHaveBeenCalledTimes(1);
    expect(loadedInputs.at(0)?.sourceScope).toEqual({ kind: "source", sourceId: 17 });
    expect(loadedInputs.at(0)?.canonicalRef).toBe("ref:1");
    expect(loadedInputs.at(0)?.requestId).toBe("request-1");
    expect(calls).toContain("clear");
    expect(calls).toContain("pending");
    expect(calls.at(-1)).toBe("load");
  });

  it("establishes evidence source navigation request identity before focused source loads", async () => {
    const calls: string[] = [];
    const contexts: unknown[] = [];
    const pending: unknown[] = [];
    const loaded: unknown[] = [];
    await showEvidenceInSource({
      currentRun: completedRun,
      selectedTrace: { ref: "raw-ref", source_id: 17 } as never,
      highlightedRef: "canonical-ref",
      snapshotAvailability: "available",
      snapshotProbeState: "available",
      sourceScope: { kind: "source", sourceId: 17 },
      nextRequestId: () => { calls.push("identity"); return "request-9"; },
      clearHighlight: () => calls.push("clear"),
      setReturnContext: (value) => { calls.push("return-context"); contexts.push(value); },
      setPendingFocus: (value) => { calls.push("pending"); pending.push(value); },
      dispatch: (event) => calls.push(`dispatch:${event.type}`),
      loadSourceWindow: async (input) => { calls.push("load"); loaded.push(input); },
    });

    expect(calls[0]).toBe("identity");
    expect(calls[1]).toBe("clear");
    expect(calls[2]).toBe("return-context");
    expect(calls[3]).toBe("pending");
    expect(calls[4]).toBe("dispatch:show_evidence_in_source");
    expect(calls[5]).toBe("load");
    expect(contexts).toHaveLength(1);
    expect(contexts[0]).toHaveProperty("runId", 91);
    expect(contexts[0]).toHaveProperty("traceRef", "canonical-ref");
    expect(pending).toHaveLength(1);
    expect(pending[0]).toHaveProperty("requestId", "request-9");
    expect(pending[0]).toHaveProperty("traceRef", "canonical-ref");
    expect(pending[0]).toHaveProperty("sourceViewBasis", "run_snapshot");
    expect(loaded).toHaveLength(1);
    expect(loaded[0]).toHaveProperty("requestId", "request-9");
    expect(loaded[0]).toHaveProperty("canonicalRef", "canonical-ref");
    expect(loaded[0]).toHaveProperty("sourceScope", { kind: "source", sourceId: 17 });
  });

  it("models Back to evidence with an explicit evidence-review event", () => {
    const state = { ...defaultAnalysisWorkspaceUiState(), openRunState: { kind: "saved" as const, runId: 91 }, canvasMode: "source" as const };
    const next = changeCompanionTab(state, "evidence", "ref:1");

    expect(next.companionTab).toBe("evidence");
    expect(next.canvasMode).toBe("report");
    expect(next.selectedTraceRef).toBe("ref:1");
    expect(next.openRunState).toEqual({ kind: "saved", runId: 91 });
  });

  it("activates Chat only through tab selection or question submission", async () => {
    const initial = { ...defaultAnalysisWorkspaceUiState(), openRunState: { kind: "saved" as const, runId: 91 }, companionTab: "evidence" as const };
    const submit = vi.fn().mockResolvedValue(undefined);
    const tabSelected = changeCompanionTab(initial, "chat");
    const submitted = await submitCompanionQuestion(initial, submit);

    expect(tabSelected.companionTab).toBe("chat");
    expect(submitted.companionTab).toBe("chat");
    expect(submit).toHaveBeenCalledTimes(1);
    expect(submit).toHaveBeenCalledWith();
    expect(initial.companionTab).toBe("evidence");
    expect(changeCompanionTab(initial, "runs").companionTab).toBe("runs");
    expect(changeCompanionTab(initial, "chunks").companionTab).toBe("chunks");
    expect(changeCompanionTab(initial, "evidence").companionTab).toBe("evidence");
    expect(Object.hasOwn(submitted, "inspectorMode")).toBe(false);
  });

  it("keeps Runs filters durable and source ingest jobs out of Runs", () => {
    const filter = { ...runsFilterDefaults(), query: "research", scope: "current" as const };
    const props = runCompanionRouteProps({ workspaceUiState: defaultAnalysisWorkspaceUiState(), focusedChunkSummaries: [], selectedRunIsActive: false, activeRuns: [], savedRuns: [], runsFilter: filter });

    expect(props.runsFilter).toBe(filter);
    expect(Object.hasOwn(props, "sourceJobs")).toBe(false);
    expect(Object.hasOwn(props, "takeoutJobs")).toBe(false);
  });

  it("opens a run from the runId query parameter", () => {
    expect(runIdFromHref("https://extractum.local/analysis?runId=91")).toBe(91);
    expect(runIdFromHref("https://extractum.local/analysis?runId=invalid")).toBeNull();
    expect(runIdFromHref("https://extractum.local/analysis")).toBeNull();
  });
});
