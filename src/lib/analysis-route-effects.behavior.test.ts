import { afterEach, describe, expect, it, vi } from "vitest";
import {
  clearEvidenceNavigation,
  createEvidenceRequestSequence,
  createLatestRequestGate,
  createRouteTimer,
  createSavedRunsLoadScheduler,
  focusedSourceLoadExit,
  focusedSourceRequestMatches,
  shouldProbeRunSnapshot,
  youtubeSyncOptionsForSource,
} from "$lib/analysis-route-effects";
import { runsFilterDefaults } from "$lib/analysis-run-companion-state";
import {
  evidenceHighlightMatchesCurrent,
  pendingFocusMatchesCurrent,
  sourceReturnContextIsActive,
  sourceScopesEqual,
} from "$lib/analysis-evidence-source-navigation";

afterEach(() => {
  vi.useRealTimers();
});

const request = {
  requestId: "request-1",
  runId: 91,
  sourceScope: { kind: "source" as const, sourceId: 17 },
  sourceViewBasis: "run_snapshot" as const,
  traceRef: "ref:1",
};

const current = {
  pending: request,
  runId: 91,
  sourceScope: { kind: "source" as const, sourceId: 17 },
  sourceViewBasis: "run_snapshot" as const,
  selectedTraceRef: "ref:1",
};

describe("analysis route effects", () => {
  it("schedules saved run history loading from explicit scope params and runs filters", async () => {
    vi.useFakeTimers();
    const load = vi.fn().mockResolvedValue(undefined);
    const scheduler = createSavedRunsLoadScheduler(load, 250);
    const params = { sourceId: 17, sourceGroupId: null };
    const filter = { ...runsFilterDefaults(), scope: "current" as const };

    scheduler.schedule(params, filter);
    expect(load).not.toHaveBeenCalled();
    await vi.advanceTimersByTimeAsync(250);
    expect(load).toHaveBeenCalledTimes(1);
    expect(load).toHaveBeenCalledWith(params, filter);
    expect(load.mock.calls[0][0]).toBe(params);
    expect(load.mock.calls[0][1]).toBe(filter);
  });

  it("debounces saved run reloads and clears pending timers on teardown", async () => {
    vi.useFakeTimers();
    const load = vi.fn().mockResolvedValue(undefined);
    const scheduler = createSavedRunsLoadScheduler(load, 250);
    const firstParams = { sourceId: 1, sourceGroupId: null };
    const secondParams = { sourceId: 2, sourceGroupId: null };
    const firstFilter = runsFilterDefaults();
    const secondFilter = { ...runsFilterDefaults(), query: "latest" };
    scheduler.schedule(firstParams, firstFilter);
    await vi.advanceTimersByTimeAsync(125);
    expect(load).not.toHaveBeenCalled();
    scheduler.schedule(secondParams, secondFilter);
    await vi.advanceTimersByTimeAsync(249);
    expect(load).not.toHaveBeenCalled();
    await vi.advanceTimersByTimeAsync(1);
    expect(load).toHaveBeenCalledWith(secondParams, secondFilter);
    expect(load).not.toHaveBeenCalledWith(firstParams, firstFilter);
    scheduler.schedule({ sourceId: 3, sourceGroupId: null }, runsFilterDefaults());
    scheduler.dispose();
    await vi.advanceTimersByTimeAsync(250);
    expect(load).toHaveBeenCalledTimes(1);
  });

  it("includes YouTube comments when syncing a video source from the main sync action", () => {
    const options = youtubeSyncOptionsForSource("video");

    expect(options.metadata).toBe(true);
    expect(options.transcripts).toBe(true);
    expect(options.comments).toBe(true);
  });

  it("probes opened-run snapshot availability before the user switches to Source mode", () => {
    expect(shouldProbeRunSnapshot({ runId: 91, canvasMode: "report" })).toBe(true);
    expect(shouldProbeRunSnapshot({ runId: 91, canvasMode: "source" })).toBe(true);
    expect(shouldProbeRunSnapshot({ runId: null, canvasMode: "report" })).toBe(false);
    expect(shouldProbeRunSnapshot({ runId: null, canvasMode: "source" })).toBe(false);
  });

  it("ignores stale YouTube detail responses after the selected source changes", () => {
    const gate = createLatestRequestGate();
    const first = gate.begin("17:video");

    expect(first.key).toBe("17:video");
    expect(first.sequence).toBe(1);
    expect(gate.isCurrent(first)).toBe(true);
    gate.invalidate();
    expect(gate.isCurrent(first)).toBe(false);
    expect(gate.complete(first)).toBe(false);
    const second = gate.begin("18:playlist");
    expect(second.sequence).toBe(2);
    expect(gate.isCurrent(second)).toBe(true);
    expect(gate.complete(second)).toBe(true);
  });

  it("keeps evidence source route state and active return context local to the route", () => {
    const sequence = createEvidenceRequestSequence();
    const scope = { kind: "source" as const, sourceId: 17 };
    const context = { kind: "evidence" as const, runId: 91, sourceScope: scope, sourceViewBasis: "run_snapshot" as const, traceRef: "ref:1" };
    const pending = { requestId: "evidence-source-1", runId: 91, sourceScope: scope, sourceViewBasis: "run_snapshot" as const, traceRef: "ref:1" };
    const highlight = { tokenId: "highlight-1", runId: 91, sourceScope: scope, sourceViewBasis: "run_snapshot" as const, traceRef: "ref:1", createdAt: 1 };
    const active = { runId: 91, sourceScope: scope, sourceViewBasis: "run_snapshot" as const, selectedTraceRef: "ref:1" };

    expect(sequence.next()).toBe("evidence-source-1");
    expect(sequence.next()).toBe("evidence-source-2");
    expect(sourceReturnContextIsActive(context, active)).toBe(true);
    expect(sourceReturnContextIsActive(context, { ...active, runId: 92 })).toBe(false);
    expect(sourceReturnContextIsActive(context, { ...active, sourceViewBasis: "live_source" })).toBe(false);
    expect(sourceReturnContextIsActive(context, { ...active, selectedTraceRef: "ref:2" })).toBe(false);
    expect(sourceReturnContextIsActive(context, { ...active, sourceScope: { kind: "source", sourceId: 18 } })).toBe(false);
    expect(pendingFocusMatchesCurrent(pending, { requestId: "evidence-source-1", ...active })).toBe(true);
    expect(pendingFocusMatchesCurrent(pending, { requestId: "stale", ...active })).toBe(false);
    expect(pendingFocusMatchesCurrent(pending, { requestId: "evidence-source-1", ...active, runId: 92 })).toBe(false);
    expect(pendingFocusMatchesCurrent(pending, { requestId: "evidence-source-1", ...active, sourceViewBasis: "live_source" })).toBe(false);
    expect(evidenceHighlightMatchesCurrent(highlight, active)).toBe(true);
    expect(evidenceHighlightMatchesCurrent(highlight, { ...active, runId: 92 })).toBe(false);
    expect(evidenceHighlightMatchesCurrent(highlight, { ...active, selectedTraceRef: "ref:2" })).toBe(false);
    expect(sourceScopesEqual(scope, { kind: "source", sourceId: 17 })).toBe(true);
  });

  it("clears pending evidence source highlight timers on route teardown", () => {
    vi.useFakeTimers();
    const clearHighlight = vi.fn();
    const timer = createRouteTimer();
    timer.schedule(clearHighlight, 2500);
    timer.dispose();
    vi.runAllTimers();

    expect(clearHighlight).not.toHaveBeenCalled();
  });

  it("clears evidence source navigation when route context changes", () => {
    const calls: string[] = [];
    clearEvidenceNavigation({
      clearReturnContext: () => calls.push("return"),
      clearPendingFocus: () => calls.push("pending"),
      clearHighlight: () => calls.push("highlight"),
    });

    expect(calls).toEqual(["return", "pending", "highlight"]);
  });

  it("checks pending focus before assigning focused snapshot state", () => {
    expect(focusedSourceRequestMatches(request, { ...current, selectedTraceRef: "ref:stale" })).toBe(false);
  });

  it("compares focused-load requests against current route scope and source basis", () => {
    expect(focusedSourceRequestMatches(request, current)).toBe(true);
    expect(focusedSourceRequestMatches(request, { ...current, runId: 92 })).toBe(false);
    expect(focusedSourceRequestMatches(request, { ...current, selectedTraceRef: "ref:2" })).toBe(false);
    expect(focusedSourceRequestMatches(request, { ...current, sourceViewBasis: "live_source" })).toBe(false);
    expect(focusedSourceRequestMatches(request, { ...current, sourceScope: { kind: "source", sourceId: 18 } })).toBe(false);
    expect(focusedSourceRequestMatches(request, { ...current, pending: null })).toBe(false);
  });

  it("checks pending focus before assigning focused group-live state", () => {
    expect(focusedSourceRequestMatches(request, { ...current, pending: { ...request, requestId: "newer" } })).toBe(false);
  });

  it("checks pending focus before assigning focused single-source and transcript state", () => {
    expect(focusedSourceRequestMatches(request, current)).toBe(true);
    expect(focusedSourceRequestMatches(request, { ...current, sourceViewBasis: "live_source" })).toBe(false);
  });

  it("uses request-owned helpers for focused-load absence, failure, and loading cleanup", () => {
    const missing = focusedSourceLoadExit(request, current, { kind: "missing_target" });
    const failed = focusedSourceLoadExit(request, current, { kind: "failed", error: { kind: "network", message: "backend offline" } });
    const stale = focusedSourceLoadExit(request, { ...current, runId: 92 }, { kind: "failed", error: new Error("stale") });

    expect(missing.accepted).toBe(true);
    expect(missing.clearPendingFocus).toBe(true);
    expect(missing.clearHighlight).toBe(true);
    expect(missing.clearLoading).toBe(true);
    expect(missing.status).toBe("Selected evidence was not found in the loaded source window.");
    expect(failed.accepted).toBe(true);
    expect(failed.clearPendingFocus).toBe(true);
    expect(failed.clearHighlight).toBe(true);
    expect(failed.clearLoading).toBe(true);
    expect(failed.status).toBe("Error loading selected source evidence (network): backend offline");
    expect(stale.accepted).toBe(false);
  });

  it("completes active focused loads without target on unsupported, missing source, and superseded transcript exits", () => {
    const unsupported = focusedSourceLoadExit(request, current, { kind: "missing_target", reason: "unsupported" });
    const missing = focusedSourceLoadExit(request, current, { kind: "missing_target", reason: "missing_source" });
    const superseded = focusedSourceLoadExit(request, current, { kind: "missing_target", reason: "superseded_transcript" });

    expect(unsupported.accepted).toBe(true);
    expect(missing.accepted).toBe(true);
    expect(superseded.accepted).toBe(true);
    expect([unsupported, missing, superseded].every((exit) => exit.clearLoading)).toBe(true);
  });
});
