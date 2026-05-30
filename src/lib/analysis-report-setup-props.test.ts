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

  it("keeps setup focused on report configuration and source preparation", () => {
    expect(reportSetupPanelSource).toContain("Run report");
    expect(reportSetupPanelSource).toContain("Sync source");
    expect(reportSetupPanelSource).toContain("selectedTemplate");
    expect(reportSetupPanelSource).toContain("reportLaunchDisabledReason");
    expect(reportSetupPanelSource).not.toContain("Export for NotebookLM");
    expect(reportSetupPanelSource).not.toContain("onOpenNotebookLmExport");
    expect(reportSetupPanelSource).not.toContain("exportingNotebookLm");
    expect(reportSetupPanelSource).not.toContain("templateName");
    expect(reportSetupPanelSource).not.toContain("templateBody");
    expect(reportSetupPanelSource).not.toContain("savingTemplate");
    expect(reportSetupPanelSource).not.toContain("deletingTemplate");
    expect(reportSetupPanelSource).not.toContain("onSaveTemplateCopy");
    expect(reportSetupPanelSource).not.toContain("onSaveTemplateChanges");
    expect(reportSetupPanelSource).not.toContain("onDeleteTemplate");
    expect(reportSetupPanelSource).not.toContain("SourceGroupEditor");
    expect(reportSetupPanelSource).not.toContain("TemplateEditor");
  });
});
