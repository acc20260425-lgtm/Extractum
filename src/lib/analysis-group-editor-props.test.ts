import { describe, expect, it } from "vitest";
import reportCanvasSource from "./components/analysis/report-canvas.svelte?raw";
import reportSetupPanelSource from "./components/analysis/report-setup-panel.svelte?raw";
import analysisPageSource from "../routes/analysis/+page.svelte?raw";

describe("analysis group editor props", () => {
  it("names the report setup group editor selection explicitly", () => {
    expect(analysisPageSource).toContain("selectedGroupEditorId={selectedGroupEditorId}");
    expect(reportCanvasSource).toContain("selectedGroupEditorId,");
    expect(reportCanvasSource).toContain("selectedGroupEditorId: string;");
    expect(reportSetupPanelSource).toContain("selectedGroupEditorId,");
    expect(reportSetupPanelSource).toContain("selectedGroupEditorId: string;");
    expect(reportCanvasSource).not.toContain("selectedGroupId,");
    expect(reportCanvasSource).not.toContain("selectedGroupId: string;");
    expect(reportSetupPanelSource).not.toContain("selectedGroupId,");
    expect(reportSetupPanelSource).not.toContain("selectedGroupId: string;");
    expect(reportSetupPanelSource).toContain("selectedGroupId={selectedGroupEditorId}");
  });
});
