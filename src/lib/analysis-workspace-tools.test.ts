import { describe, expect, it } from "vitest";
import reportWorkspaceToolsSource from "./components/analysis/report-workspace-tools.svelte?raw";

describe("analysis workspace tools component contract", () => {
  it("stays presentational and route-state agnostic", () => {
    expect(reportWorkspaceToolsSource).toContain("showNotebookLmExport");
    expect(reportWorkspaceToolsSource).toContain("canExportNotebookLm");
    expect(reportWorkspaceToolsSource).toContain("exportDisabledReason");
    expect(reportWorkspaceToolsSource).toContain("exportingNotebookLm");
    expect(reportWorkspaceToolsSource).toContain("templateEditorOpen");
    expect(reportWorkspaceToolsSource).toContain("groupEditorOpen");
    expect(reportWorkspaceToolsSource).toContain("onOpenNotebookLmExport");
    expect(reportWorkspaceToolsSource).toContain("onToggleTemplateEditor");
    expect(reportWorkspaceToolsSource).toContain("onToggleGroupEditor");

    expect(reportWorkspaceToolsSource).not.toContain("currentSource");
    expect(reportWorkspaceToolsSource).not.toContain("currentGroup");
    expect(reportWorkspaceToolsSource).not.toContain("currentRun");
    expect(reportWorkspaceToolsSource).not.toContain("workspaceSelection");
  });

  it("does not import APIs or call Tauri invoke", () => {
    expect(reportWorkspaceToolsSource).not.toContain("$lib/api");
    expect(reportWorkspaceToolsSource).not.toContain("@tauri-apps/api");
    expect(reportWorkspaceToolsSource).not.toContain("invoke(");
  });

  it("renders accessible source-group export disabled reason", () => {
    expect(reportWorkspaceToolsSource).toContain(
      'const exportReasonId = "notebooklm-export-disabled-reason"',
    );
    expect(reportWorkspaceToolsSource).toContain('id="notebooklm-export-disabled-reason"');
    expect(reportWorkspaceToolsSource).toContain(
      "ariaDescribedby={!compact && exportDisabledReason ? exportReasonId : undefined}",
    );
    expect(reportWorkspaceToolsSource).toContain("{#if !compact && exportDisabledReason}");
    expect(reportWorkspaceToolsSource).toContain("{exportDisabledReason}");
    expect(reportWorkspaceToolsSource).toContain('class="workspace-tool-helper"');
  });

  it("uses explicit button types for workspace actions", () => {
    expect(reportWorkspaceToolsSource.match(/type="button"/g)?.length ?? 0).toBe(3);
  });
});
