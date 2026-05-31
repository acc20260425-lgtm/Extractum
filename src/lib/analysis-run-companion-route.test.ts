import { describe, expect, it } from "vitest";
import analysisPageSource from "../routes/analysis/+page.svelte?raw";
import chatWorkflowSource from "./analysis-chat-workflow.ts?raw";
import traceWorkflowSource from "./analysis-trace-workflow.ts?raw";

function functionSlice(name: string, nextName: string) {
  const start = analysisPageSource.indexOf(`  ${name}`);
  const end = analysisPageSource.indexOf(`\n  ${nextName}`, start + 1);

  expect(start, `${name} should exist`).toBeGreaterThan(-1);
  expect(end, `${nextName} should follow ${name}`).toBeGreaterThan(start);

  return analysisPageSource.slice(start, end);
}

describe("analysis route run companion wiring", () => {
  it("renders RunCompanionTabs instead of WorkspaceInspector", () => {
    expect(analysisPageSource).toContain(
      'import RunCompanionTabs from "$lib/components/analysis/run-companion-tabs.svelte";',
    );
    expect(analysisPageSource).not.toContain(
      'import WorkspaceInspector from "$lib/components/analysis/workspace-inspector.svelte";',
    );
    expect(analysisPageSource).toContain("<RunCompanionTabs");
    expect(analysisPageSource).not.toContain("<WorkspaceInspector");
  });

  it("uses workspaceUiState.companionTab as the only companion tab source", () => {
    expect(analysisPageSource).toContain("companionTab={workspaceUiState.companionTab}");
    expect(analysisPageSource).toContain("function changeCompanionTab");
    expect(analysisPageSource).toContain("companionTab: nextTab");
    expect(analysisPageSource).not.toContain('inspectorMode: "chunks"');
    expect(analysisPageSource).not.toContain("let inspectorMode");
    expect(analysisPageSource).not.toContain("onChangeInspectorMode");
  });

  it("passes focused chunk summaries into the companion without auto-opening chunks", () => {
    expect(analysisPageSource).toContain("focusedRunChunkSummaries");
    expect(analysisPageSource).toContain(
      "focusedChunkSummaries={focusedRunChunkSummaries(focusedLiveRun)}",
    );
    expect(analysisPageSource).toContain("{selectedRunIsActive}");
    expect(analysisPageSource).not.toContain('companionTab: "chunks"');
  });

  it("activates Evidence for trace clicks and Show in source prefers snapshot", () => {
    expect(analysisPageSource).toContain("async function focusTraceRef");
    expect(analysisPageSource).toContain('changeCompanionTab("evidence")');
    expect(analysisPageSource).toContain("async function showSelectedTraceInSource");
    expect(analysisPageSource).toContain("evidenceSourceActionDecision");
    expect(analysisPageSource).toContain("snapshotProbeState: runSnapshotProbeState");
    expect(analysisPageSource).toContain('type: "show_evidence_in_source"');
    expect(analysisPageSource).toContain("sourceViewBasis: decision.sourceViewBasis");
    expect(analysisPageSource).not.toContain("canvasMode: decision.canvasMode");
    expect(analysisPageSource).toContain("await loadSourcePageAroundTrace({");
    expect(analysisPageSource).toContain("selectedSnapshotSourceId = trace.source_id");
    expect(analysisPageSource).toContain("sourceId: trace.source_id");
    expect(analysisPageSource).toContain("aroundRef: trace.ref");
  });

  it("establishes evidence source navigation request identity before focused source loads", () => {
    const showSelectedTrace = functionSlice(
      "async function showSelectedTraceInSource",
      "async function submitRunQuestionFromCompanion",
    );

    expect(showSelectedTrace).toContain(
      "const canonicalRef = canonicalEvidenceTraceRef(decision.highlightedRef, trace.ref);",
    );
    expect(showSelectedTrace).toContain("const sourceScope = currentEvidenceSourceScope(trace.source_id);");
    expect(showSelectedTrace).toContain(
      'status = "Selected evidence no longer belongs to the opened source group.";',
    );
    expect(showSelectedTrace).toContain("const liveTarget = focusedLiveSourceTargetForTrace(trace);");
    expect(showSelectedTrace).toContain(
      'status = "This evidence does not map to a browsable live source row yet.";',
    );
    expect(showSelectedTrace).toContain("const requestId = nextEvidenceSourceRequestId();");
    expect(showSelectedTrace).toContain("clearSourceHighlight();");
    expect(showSelectedTrace).toContain("sourceReturnContext = {");
    expect(showSelectedTrace).toContain("pendingEvidenceSourceFocus = {");
    expect(showSelectedTrace).toContain("selectedTraceRef = canonicalRef;");
    expect(showSelectedTrace).toContain("highlightedRef: canonicalRef");
    expect(showSelectedTrace).toContain("requestId,");
    expect(showSelectedTrace).toContain("canonicalRef,");
    expect(showSelectedTrace).toContain("sourceScope,");

    expect(showSelectedTrace.indexOf("clearSourceHighlight();")).toBeLessThan(
      showSelectedTrace.indexOf("pendingEvidenceSourceFocus = {"),
    );
    expect(showSelectedTrace.indexOf("pendingEvidenceSourceFocus = {")).toBeLessThan(
      showSelectedTrace.indexOf('type: "show_evidence_in_source"'),
    );
  });

  it("models Back to evidence with an explicit evidence-review event", () => {
    expect(analysisPageSource).toContain('type: "return_to_evidence_review"');
    expect(analysisPageSource).toContain("traceRef:");
    expect(analysisPageSource).not.toContain('type: "back_to_evidence"');
    expect(analysisPageSource).not.toContain('type: "back"');
  });

  it("activates Chat only through tab selection or question submission", () => {
    const submitQuestion = functionSlice(
      "async function submitRunQuestionFromCompanion",
      "function changeRunsFilter",
    );

    expect(analysisPageSource).toContain("submitRunQuestionFromCompanion");
    expect(analysisPageSource).toContain("chatAvailabilityForRun");
    expect(analysisPageSource).toContain("snapshotProbeState: runSnapshotProbeState");
    expect(analysisPageSource).toContain("snapshotProbeState={runSnapshotProbeState}");
    expect(analysisPageSource).toContain('type: "change_companion_tab"');
    expect(submitQuestion).toContain('changeCompanionTab("chat")');
    expect(submitQuestion).not.toContain('companionTab: "chat"');
    expect(analysisPageSource).not.toContain("onFocusChat");
    expect(chatWorkflowSource).not.toContain("companionTab");
  });

  it("keeps Runs filters durable and source ingest jobs out of Runs", () => {
    expect(analysisPageSource).toContain("runsFilter");
    expect(analysisPageSource).toContain("persistableAnalysisWorkspaceState(workspaceUiState");
    expect(analysisPageSource).toContain("runsFilter");
  });

  it("updates trace workflow patches from inspector mode to evidence companion", () => {
    expect(traceWorkflowSource).toContain('companionTab: "evidence"');
    expect(traceWorkflowSource).not.toContain("inspectorMode");
  });
});
