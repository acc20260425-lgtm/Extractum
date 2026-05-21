import { describe, expect, it } from "vitest";
import analysisPageSource from "../routes/analysis/+page.svelte?raw";
import reportCanvasSource from "./components/analysis/report-canvas.svelte?raw";
import reportSetupPanelSource from "./components/analysis/report-setup-panel.svelte?raw";

function componentTag(source: string, componentName: string) {
  const match = new RegExp(`(?:^|\\r?\\n)([ \\t]*)<${componentName}\\b`).exec(source);

  expect(match, `${componentName} tag should exist`).not.toBeNull();

  const start = match!.index + match![0].lastIndexOf("<");
  const indent = match![1];
  const lineStart = source.lastIndexOf("\n", start) + 1;
  const end = source.indexOf(`\n${indent}/>`, start);

  expect(lineStart, `${componentName} tag line should exist`).toBeGreaterThan(-1);
  expect(end, `${componentName} tag should close`).toBeGreaterThan(start);

  return source.slice(start, end);
}

describe("analysis report workspace selection props", () => {
  it("passes workspace selection into the report canvas instead of legacy scope ids", () => {
    const routeReportCanvas = componentTag(analysisPageSource, "ReportCanvas");

    expect(routeReportCanvas).toContain("workspaceSelection={workspaceUiState.workspaceSelection}");
    expect(routeReportCanvas).not.toContain("{analysisScope}");
  });

  it("keeps setup branch compatibility projection inside report components", () => {
    const setupPanelTag = componentTag(reportCanvasSource, "ReportSetupPanel");

    expect(reportCanvasSource).toContain("workspaceSelection,");
    expect(reportCanvasSource).toContain("workspaceSelection: WorkspaceSelection;");
    expect(reportCanvasSource).not.toContain("analysisScope,");
    expect(reportCanvasSource).not.toContain('analysisScope: "single_source" | "source_group";');
    expect(setupPanelTag).toContain("{workspaceSelection}");
    expect(setupPanelTag).not.toContain("{analysisScope}");

    expect(reportSetupPanelSource).toContain("workspaceSelection,");
    expect(reportSetupPanelSource).toContain("workspaceSelection: WorkspaceSelection;");
    expect(reportSetupPanelSource).not.toContain("analysisScope,");
    expect(reportSetupPanelSource).not.toContain('analysisScope: "single_source" | "source_group";');
    expect(reportSetupPanelSource).toContain("legacyScopeFromWorkspaceSelection(workspaceSelection)");
  });
});
