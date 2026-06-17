// @ts-nocheck
import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";

describe("project runs tab delete controls", () => {
  it("wires per-row delete controls for analysis project runs", () => {
    const source = readFileSync("src/lib/components/research-projects/ProjectRunsTab.svelte", "utf8");

    expect(source).toContain("Trash2");
    expect(source).toContain("deleteAnalysisRun");
    expect(source).toContain("openConfirmModal");
    expect(source).toContain("Delete project run?");
    expect(source).toContain('confirmLabel: "Delete"');
    expect(source).not.toContain("confirm(");
    expect(source).toContain("isAnalysisRunActive");
    expect(source).toContain("aria-label={`Delete project run ${run.id}`}");
    expect(source).toContain("disabled={isAnalysisRunActive(run) || deletingRunIds[run.id]}");
  });

  it("wires per-row delete controls for prompt pack runs", () => {
    const source = readFileSync("src/lib/components/research-projects/YoutubeSummaryRunsPanel.svelte", "utf8");

    expect(source).toContain("Trash2");
    expect(source).toContain("deletePromptPackRun");
    expect(source).toContain("openConfirmModal");
    expect(source).toContain("Delete Prompt Pack run?");
    expect(source).toContain('confirmLabel: "Delete"');
    expect(source).not.toContain("confirm(");
    expect(source).toContain("isPromptPackRunActive");
    expect(source).toContain("aria-label={`Delete Prompt Pack run ${run.runId}`}");
    expect(source).toContain("disabled={isPromptPackRunActive(run) || deletingRunIds[run.runId]}");
    expect(source).toContain("if (selectedRunId === run.runId) selectedRunId = null");
    expect(source).toContain("deletedRunIds");
    expect(source).toContain("filterDeletedRunIds(nextRuns, deletedRunIds)");
    expect(source).toContain("if (deletedRunIds[event.payload.runId]) return");
  });
});
