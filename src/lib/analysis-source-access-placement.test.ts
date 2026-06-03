import { describe, expect, it } from "vitest";
import analysisPageSource from "../routes/analysis/+page.svelte?raw";
import runCompanionRunsTabSource from "./components/analysis/run-companion-runs-tab.svelte?raw";

describe("analysis source access placement", () => {
  it("uses the compact source rail inside the analysis route", () => {
    expect(analysisPageSource).toContain(
      'import CompactSourceRail from "$lib/components/analysis/compact-source-rail.svelte";',
    );
    expect(analysisPageSource).not.toContain(
      'import WorkspaceRail from "$lib/components/analysis/workspace-rail.svelte";',
    );
    expect(analysisPageSource).toContain("<CompactSourceRail");
    expect(analysisPageSource).toContain("workspaceSelection={workspaceUiState.workspaceSelection}");
    expect(analysisPageSource).toContain("onSelectSource={(sourceId) => void selectSource(sourceId)}");
    expect(analysisPageSource).toContain("onSelectGroup={selectGroup}");
    expect(analysisPageSource).toContain("{startingMigratedHistorySourceIds}");
    expect(analysisPageSource).toContain("onStartMigratedHistoryImport={(sourceId) => void startMigratedHistoryImport(sourceId)}");
    expect(analysisPageSource).toContain("sourceJobsBySource");
  });

  it("does not place source ingest jobs in the analysis runs companion", () => {
    expect(runCompanionRunsTabSource).not.toContain("SourceJobRecord");
    expect(runCompanionRunsTabSource).not.toContain("sourceJobs");
    expect(runCompanionRunsTabSource).not.toContain("Takeout");
  });

  it("keeps the left analysis column compact while rendering the run companion", () => {
    expect(analysisPageSource).toContain(
      "grid-template-columns: minmax(4.25rem, 4.75rem) minmax(0, 1fr) minmax(21rem, clamp(22rem, 26vw, 26rem));",
    );
    expect(analysisPageSource).toContain("<ReportCanvas");
    expect(analysisPageSource).toContain("<RunCompanionTabs");
    expect(analysisPageSource).not.toContain("<WorkspaceMain");
    expect(analysisPageSource).not.toContain("<WorkspaceInspector");
  });
});
