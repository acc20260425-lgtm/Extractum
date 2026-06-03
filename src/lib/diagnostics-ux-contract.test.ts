import { describe, expect, it } from "vitest";
import diagnosticsTableSource from "./components/diagnostics/DiagnosticCountTable.svelte?raw";
import diagnosticsViewModelSource from "./diagnostics-view-model?raw";
import diagnosticsPageSource from "../routes/diagnostics/+page.svelte?raw";

describe("diagnostics UX contract", () => {
  it("adds problem-first diagnostics table controls", () => {
    expect(diagnosticsPageSource).toContain("diagnosticsTableMode");
    expect(diagnosticsPageSource).toContain("Only issues");
    expect(diagnosticsPageSource).toContain("All tables");
    expect(diagnosticsPageSource).toContain("diagnosticsTableSections");
  });

  it("keeps large diagnostics tables collapsible", () => {
    expect(diagnosticsTableSource).toContain("<details");
    expect(diagnosticsTableSource).toContain("<summary");
    expect(diagnosticsTableSource).toContain("open = true");
  });

  it("puts diagnostics table controls before summary cards and filters issue rows", () => {
    const controlsIndex = diagnosticsPageSource.indexOf('class="diagnostics-table-controls"');
    const gridIndex = diagnosticsPageSource.indexOf('class="diagnostics-grid"');

    expect(controlsIndex).toBeGreaterThan(0);
    expect(gridIndex).toBeGreaterThan(0);
    expect(controlsIndex).toBeLessThan(gridIndex);
    expect(diagnosticsPageSource).toContain("visibleDiagnosticRows");
    expect(diagnosticsPageSource).toContain("diagnosticRowHasIssue");
    expect(diagnosticsViewModelSource).toContain("export function diagnosticRowHasIssue");
    expect(diagnosticsViewModelSource).toContain("export function filterDiagnosticIssueRows");
  });
});
