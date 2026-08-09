import { render } from "svelte/server";
import { describe, expect, it, vi } from "vitest";
import ReportWorkspaceTools from "./components/analysis/report-workspace-tools.svelte";

function renderTools(overrides: Record<string, unknown> = {}) {
  return render(ReportWorkspaceTools, {
    props: {
      showNotebookLmExport: true,
      canExportNotebookLm: false,
      exportDisabledReason: "Source group export requires loaded messages.",
      exportingNotebookLm: false,
      templateEditorOpen: false,
      groupEditorOpen: false,
      onOpenNotebookLmExport: vi.fn(),
      onToggleTemplateEditor: vi.fn(),
      onToggleGroupEditor: vi.fn(),
      ...overrides,
    },
  }).body;
}

describe("analysis workspace tools component contract", () => {
  it("renders accessible source-group export disabled reason", () => {
    const body = renderTools();
    expect(body).toContain('aria-label="Workspace actions"');
    expect(body).toContain('id="notebooklm-export-disabled-reason"');
    expect(body).toContain('aria-describedby="notebooklm-export-disabled-reason"');
    expect(body).toContain("Source group export requires loaded messages.");
    expect(body).toContain("disabled");
    expect(body).toContain('data-smoke-id="notebooklm-export-disabled-reason"');
  });

  it("uses explicit button types for workspace actions", () => {
    expect(renderTools().match(/type="button"/g)).toHaveLength(3);
  });
});
