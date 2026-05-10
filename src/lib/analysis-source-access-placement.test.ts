import { describe, expect, it } from "vitest";
import analysisPageSource from "../routes/analysis/+page.svelte?raw";
import activeRunListSource from "./components/analysis/active-run-list.svelte?raw";
import runHistorySource from "./components/analysis/run-history.svelte?raw";
import workspaceInspectorSource from "./components/analysis/workspace-inspector.svelte?raw";

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
    expect(analysisPageSource).toContain("sourceJobsBySource");
  });

  it("does not place source ingest jobs in the analysis runs surfaces", () => {
    expect(workspaceInspectorSource).not.toContain("SourceJobRecord");
    expect(workspaceInspectorSource).not.toContain("sourceJobs");
    expect(workspaceInspectorSource).not.toContain("takeoutJobsBySource");
    expect(runHistorySource).not.toContain("SourceJobRecord");
    expect(runHistorySource).not.toContain("sourceJobs");
    expect(runHistorySource).not.toContain("Takeout");
    expect(activeRunListSource).not.toContain("SourceJobRecord");
    expect(activeRunListSource).not.toContain("sourceJobs");
    expect(activeRunListSource).not.toContain("Takeout");
  });

  it("keeps the left analysis column compact while preserving the inspector", () => {
    expect(analysisPageSource).toContain(
      "grid-template-columns: minmax(4.25rem, 4.75rem) minmax(0, 1.6fr) minmax(320px, 430px);",
    );
    expect(analysisPageSource).toContain("<ReportCanvas");
    expect(analysisPageSource).toContain("<WorkspaceInspector");
    expect(analysisPageSource).not.toContain("<WorkspaceMain");
    expect(analysisPageSource).not.toContain("<RunCompanionTabs");
  });
});
