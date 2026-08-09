import { describe, expect, it } from "vitest";
import {
  chatAvailabilityForRun,
  evidenceSourceActionDecision,
  filterCompanionRuns,
  hasActiveCompanionRunsFilter,
  runsFilterDefaults,
} from "$lib/analysis-run-companion-state";
import {
  clearCompanionRunsFilter,
  companionChunkPresentation,
  runCompanionPanelId,
  runCompanionTabId,
  runCompanionTabLabel,
} from "$lib/analysis-run-companion-tabs-model";
import { runTargetLabel } from "$lib/analysis-utils";
import type { AnalysisRunDetail, AnalysisRunSummary } from "$lib/types/analysis";

function run(id: number, overrides: Partial<AnalysisRunSummary> = {}): AnalysisRunSummary {
  return {
    id,
    status: "completed",
    created_at: 1_700_000_000 + id,
    source_id: 17,
    source_group_id: null,
    scope_type: "source",
    scope_label: "Research channel",
    source_title: "Research channel",
    source_group_name: null,
    prompt_template_name: "Brief",
    provider_profile: "Main",
    provider: "openai",
    model: "gpt-5",
    error: null,
    project_id: null,
    project_name: null,
    ...overrides,
  } as AnalysisRunSummary;
}

function detail(overrides: Partial<AnalysisRunDetail> = {}): AnalysisRunDetail {
  return {
    ...run(91),
    result_markdown: "Saved report",
    snapshot_state: "available",
    snapshot_captured_at: 1_700_000_100,
    snapshot_error: null,
    ...overrides,
  } as AnalysisRunDetail;
}

describe("run companion tabs", () => {
  it("uses accessible Evidence, Chat, Chunks, and Runs tabs", () => {
    const chunk = companionChunkPresentation([{ total: 4 }, { total: 4 }], true, true);

    expect(runCompanionTabId("evidence")).toBe("run-companion-tab-evidence");
    expect(runCompanionTabLabel("evidence", chunk.label)).toBe("Evidence");
    expect(runCompanionTabId("chat")).toBe("run-companion-tab-chat");
    expect(runCompanionTabLabel("chat", chunk.label)).toBe("Chat");
    expect(runCompanionTabId("chunks")).toBe("run-companion-tab-chunks");
    expect(runCompanionTabLabel("chunks", chunk.label)).toBe("Chunks 2/4");
    expect(chunk.disabled).toBe(false);
    expect(runCompanionTabId("runs")).toBe("run-companion-tab-runs");
    expect(runCompanionTabLabel("runs", chunk.label)).toBe("Runs");
    expect(runCompanionPanelId()).toBe("run-companion-panel");
    expect(new Set((["evidence", "chat", "chunks", "runs"] as const).map((tab) => runCompanionTabId(tab))).size).toBe(4);
    expect(companionChunkPresentation([], false, false).disabled).toBe(true);
    expect(companionChunkPresentation([], false, true).disabled).toBe(false);
  });

  it("renders chunk summaries compactly inside the companion", () => {
    const waiting = companionChunkPresentation([], true, true);
    const terminal = companionChunkPresentation([], false, true);
    const populated = companionChunkPresentation([{ index: 1, total: 3 }, { index: 2, total: 3 }], true, true);

    expect(waiting.framed).toBe(false);
    expect(waiting.label).toBe("Chunks");
    expect(waiting.running).toBe(true);
    expect(waiting.disabled).toBe(false);
    expect(terminal.framed).toBe(false);
    expect(terminal.running).toBe(false);
    expect(populated.label).toBe("Chunks 2/3");
    expect(populated.disabled).toBe(false);
  });

  it("keeps Evidence focused on trace refs and Show in source", () => {
    const selectedTrace = { ref: "ref:1", source_id: 17 } as never;
    const decision = evidenceSourceActionDecision({
      currentRun: detail(), selectedTrace, snapshotAvailability: "available", snapshotProbeState: "available",
    });

    expect(decision.kind).toBe("run_snapshot");
    expect(decision).toHaveProperty("highlightedRef", "ref:1");
    expect(decision).toHaveProperty("canvasMode", "source");
    expect(decision).toHaveProperty("sourceViewBasis", "run_snapshot");
    expect(evidenceSourceActionDecision({ currentRun: null, selectedTrace, snapshotAvailability: "unknown", snapshotProbeState: "loading" }).kind).toBe("unavailable");
    expect(evidenceSourceActionDecision({ currentRun: detail(), selectedTrace: null, snapshotAvailability: "available", snapshotProbeState: "available" }).kind).toBe("unavailable");
  });

  it("keeps Chat explicit and availability-gated", () => {
    const none = chatAvailabilityForRun({ currentRun: null, snapshotAvailability: "unknown", snapshotProbeState: "loading" });
    const pending = chatAvailabilityForRun({ currentRun: detail({ status: "running" }), snapshotAvailability: "unknown", snapshotProbeState: "loading" });
    const ready = chatAvailabilityForRun({ currentRun: detail(), snapshotAvailability: "available", snapshotProbeState: "available" });

    expect(none.enabled).toBe(false);
    expect(none.reason).toBe("no_run");
    expect(pending.enabled).toBe(false);
    expect(pending.reason).toBe("pending_completion");
    expect(ready.enabled).toBe(true);
    expect(ready.reason).toBe("enabled");
  });

  it("contains only analysis report runs in the Runs tab", () => {
    const active = run(1, { status: "running", created_at: 1_700_000_300, provider: "openai" });
    const saved = run(2, { status: "completed", created_at: 1_700_000_200, provider: "anthropic" });
    const otherScope = run(3, { source_id: 99, created_at: 1_700_000_100, provider: "openai" });
    const all = filterCompanionRuns({ activeRuns: [active], savedRuns: [saved, otherScope], filter: runsFilterDefaults(), workspaceSelection: { kind: "source", sourceId: 17 } });
    const current = filterCompanionRuns({ activeRuns: [active], savedRuns: [saved, otherScope], filter: { ...runsFilterDefaults(), scope: "current" }, workspaceSelection: { kind: "source", sourceId: 17 } });
    const provider = filterCompanionRuns({ activeRuns: [active], savedRuns: [saved], filter: { ...runsFilterDefaults(), provider: "anthropic" }, workspaceSelection: { kind: "source", sourceId: 17 } });

    expect(all).toHaveLength(3);
    expect(all[0].kind).toBe("active");
    expect(all[0].run.id).toBe(1);
    expect(all[1].kind).toBe("saved");
    expect(all[1].run.id).toBe(2);
    expect(all[2].run.id).toBe(3);
    expect(current).toHaveLength(2);
    expect(current.map((entry) => entry.run.id)).toEqual([1, 2]);
    expect(provider).toHaveLength(1);
    expect(provider[0].run.id).toBe(2);
    expect(all.every((entry) => entry.kind === "active" || entry.kind === "saved")).toBe(true);
    expect(all.every((entry) => typeof entry.run.id === "number")).toBe(true);
    expect(Object.hasOwn(all[0], "sourceJob")).toBe(false);
    expect(Object.hasOwn(all[0], "takeoutJob")).toBe(false);
    expect(Object.hasOwn(all[0], "ingestJob")).toBe(false);
  });

  it("accepts project-scoped run labels in run surfaces", () => {
    const label = runTargetLabel(run(8, { source_id: null, project_id: 42, project_name: "Atlas", scope_type: "project", scope_label: "" }));

    expect(label).toContain("Atlas");
    expect(label).not.toContain("Research channel");
    expect(label.length).toBeGreaterThan(0);
  });

  it("keeps dense run filters behind an advanced filters disclosure", () => {
    expect(hasActiveCompanionRunsFilter({ ...runsFilterDefaults(), provider: "openai" })).toBe(true);
    expect(hasActiveCompanionRunsFilter({ ...runsFilterDefaults(), dateFrom: "2026-08-01" })).toBe(true);
    expect(hasActiveCompanionRunsFilter({ ...runsFilterDefaults(), template: "Brief" })).toBe(true);
    expect(clearCompanionRunsFilter({ ...runsFilterDefaults(), model: "gpt-5" })).toEqual(runsFilterDefaults());
  });

  it("offers a clear path when restored run filters hide all runs", () => {
    const restored = { ...runsFilterDefaults(), query: "missing", provider: "openai" };

    expect(hasActiveCompanionRunsFilter(restored)).toBe(true);
    expect(clearCompanionRunsFilter(restored)).toEqual(runsFilterDefaults());
    expect(hasActiveCompanionRunsFilter(clearCompanionRunsFilter(restored))).toBe(false);
  });
});
