import { describe, expect, it } from "vitest";
import analysisPageSource from "../routes/analysis/+page.svelte?raw";

const legacyWorkspaceMainTag = "<" + "WorkspaceMain";
const legacyWorkspaceMainImport =
  'import ' + 'WorkspaceMain from "$lib/components/analysis/workspace-main.svelte";';
const runCompanionTabsTag = "<" + "Run" + "CompanionTabs";

describe("analysis route report canvas wiring", () => {
  it("renders ReportCanvas instead of the legacy WorkspaceMain", () => {
    expect(analysisPageSource).toContain(
      'import ReportCanvas from "$lib/components/analysis/report-canvas.svelte";',
    );
    expect(analysisPageSource).not.toContain(legacyWorkspaceMainImport);
    expect(analysisPageSource).toContain("<ReportCanvas");
    expect(analysisPageSource).not.toContain(legacyWorkspaceMainTag);
    expect(analysisPageSource).toContain("<WorkspaceInspector");
    expect(analysisPageSource).not.toContain(runCompanionTabsTag);
  });

  it("passes persisted canvas mode and source basis from workspace UI state", () => {
    expect(analysisPageSource).toContain("canvasMode={workspaceUiState.canvasMode}");
    expect(analysisPageSource).toContain("sourceViewBasis={workspaceUiState.sourceViewBasis}");
    expect(analysisPageSource).toContain("onChangeCanvasMode={(mode) =>");
    expect(analysisPageSource).toContain("canvasMode: mode");
    expect(analysisPageSource).toContain("onViewLiveSource={() =>");
    expect(analysisPageSource).toContain('sourceViewBasis: "live_source"');
    expect(analysisPageSource).toContain("onBackToRunSnapshot={() =>");
    expect(analysisPageSource).toContain('sourceViewBasis: "run_snapshot"');
  });

  it("loads run snapshot messages through the snapshot-only API", () => {
    expect(analysisPageSource).toContain("listAnalysisRunMessages");
    expect(analysisPageSource).toContain("loadRunSnapshotFirstPage");
    expect(analysisPageSource).toContain("loadMoreRunSnapshotMessages");
    expect(analysisPageSource).toContain("runSnapshotAvailability");
    expect(analysisPageSource).toContain("runSnapshotMessages");
    expect(analysisPageSource).toContain("runSnapshotCursor");
    expect(analysisPageSource).toContain("runSnapshotError");
    expect(analysisPageSource).not.toContain("listSourceItems({ runId");
  });

  it("does not switch back to snapshot automatically when the user explicitly views live source", () => {
    expect(analysisPageSource).toContain("lastSnapshotLoadKey");
    expect(analysisPageSource).toContain('workspaceUiState.sourceViewBasis === "run_snapshot"');
    expect(analysisPageSource).toContain("void loadRunSnapshotFirstPage(currentRun.id)");
    expect(analysisPageSource).not.toContain('sourceViewBasis: "run_snapshot", // automatic');
  });
});
