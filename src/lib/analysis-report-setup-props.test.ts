import { describe, expect, it } from "vitest";
import reportCanvasSource from "./components/analysis/report-canvas.svelte?raw";
import reportSetupPanelSource from "./components/analysis/report-setup-panel.svelte?raw";

describe("analysis report setup props", () => {
  it("does not pass the route selected source id through setup-only components", () => {
    expect(reportCanvasSource).not.toContain("selectedSourceId,");
    expect(reportCanvasSource).not.toContain("{selectedSourceId}");
    expect(reportSetupPanelSource).not.toContain("selectedSourceId,");
    expect(reportSetupPanelSource).not.toContain("selectedSourceId: string;");
  });
});
