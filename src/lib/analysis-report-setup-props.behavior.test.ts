import { describe, expect, it, vi } from "vitest";
import { reportSetupProps } from "$lib/analysis-report-props";

describe("analysis report setup props", () => {
  it("does not pass the route selected source id through setup-only components", () => {
    const props = reportSetupProps({
      workspaceSelection: { kind: "source", sourceId: 17 },
      selectedTemplate: { id: 4, name: "Research" },
      reportLaunchDisabledReason: null,
      onRunReport: vi.fn(),
      onSyncCurrentSource: vi.fn(),
    });

    expect(Object.hasOwn(props, "selectedSourceId")).toBe(false);
    expect(Object.hasOwn(props, "selectedGroupId")).toBe(false);
    expect(Object.hasOwn(props, "analysisScope")).toBe(false);
    expect(props.workspaceSelection).toEqual({ kind: "source", sourceId: 17 });
  });

  it("keeps setup focused on report configuration and source preparation", () => {
    const onRunReport = vi.fn();
    const onSyncSource = vi.fn();
    const props = reportSetupProps({
      workspaceSelection: { kind: "source", sourceId: 17 },
      selectedTemplate: { id: 4, name: "Research" },
      reportLaunchDisabledReason: "Sync required",
      onRunReport,
      onSyncCurrentSource: onSyncSource,
    });

    expect(props.workspaceSelection).toEqual({ kind: "source", sourceId: 17 });
    expect(props.selectedTemplate).toEqual({ id: 4, name: "Research" });
    expect(props.reportLaunchDisabledReason).toBe("Sync required");
    expect(props.onRunReport).toBe(onRunReport);
    expect(props.onSyncCurrentSource).toBe(onSyncSource);
    expect(Object.hasOwn(props, "onOpenNotebookLmExport")).toBe(false);
    expect(Object.hasOwn(props, "exportingNotebookLm")).toBe(false);
    expect(Object.hasOwn(props, "templateName")).toBe(false);
    expect(Object.hasOwn(props, "templateBody")).toBe(false);
    expect(Object.hasOwn(props, "savingTemplate")).toBe(false);
    expect(Object.hasOwn(props, "deletingTemplate")).toBe(false);
    expect(Object.hasOwn(props, "onSaveTemplateCopy")).toBe(false);
    expect(Object.hasOwn(props, "onSaveTemplateChanges")).toBe(false);
    expect(Object.hasOwn(props, "onDeleteTemplate")).toBe(false);
    expect(Object.hasOwn(props, "sourceGroupEditor")).toBe(false);
    expect(Object.hasOwn(props, "templateEditor")).toBe(false);
  });
});
