import { describe, expect, it } from "vitest";
import reportCanvasSource from "./components/analysis/report-canvas.svelte?raw";
import reportSetupPanelSource from "./components/analysis/report-setup-panel.svelte?raw";
import analysisPageSource from "../routes/analysis/+page.svelte?raw";

describe("analysis group editor props", () => {
  it("keeps group editor selection owned by the report canvas workspace tools", () => {
    expect(analysisPageSource).toContain("selectedGroupEditorId={selectedGroupEditorId}");
    expect(reportCanvasSource).toContain("selectedGroupEditorId,");
    expect(reportCanvasSource).toContain("selectedGroupEditorId: string;");
    expect(reportCanvasSource).toContain("selectedGroupId={selectedGroupEditorId}");
    expect(reportSetupPanelSource).not.toContain("selectedGroupEditorId,");
    expect(reportSetupPanelSource).not.toContain("selectedGroupEditorId: string;");
    expect(reportCanvasSource).not.toContain("selectedGroupId,");
    expect(reportCanvasSource).not.toContain("selectedGroupId: string;");
    expect(reportSetupPanelSource).not.toContain("selectedGroupId={selectedGroupEditorId}");
  });
});
