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

  it("orders diagnostics issue details before overview only in issue mode", () => {
    expect(diagnosticsPageSource).toContain('{#if diagnosticsTableMode === "issues"}');
    expect(diagnosticsPageSource).toContain("{@render diagnosticsTableArea(tableSections)}");
    expect(diagnosticsPageSource).toContain("{@render diagnosticsOverviewArea(summary)}");

    const issueBranchIndex = diagnosticsPageSource.indexOf('{#if diagnosticsTableMode === "issues"}');
    const issueTableIndex = diagnosticsPageSource.indexOf(
      "{@render diagnosticsTableArea(tableSections)}",
      issueBranchIndex,
    );
    const issueOverviewIndex = diagnosticsPageSource.indexOf(
      "{@render diagnosticsOverviewArea(summary)}",
      issueBranchIndex,
    );
    const allBranchIndex = diagnosticsPageSource.indexOf("{:else}", issueBranchIndex);
    const allOverviewIndex = diagnosticsPageSource.indexOf(
      "{@render diagnosticsOverviewArea(summary)}",
      allBranchIndex,
    );
    const allTableIndex = diagnosticsPageSource.indexOf(
      "{@render diagnosticsTableArea(tableSections)}",
      allBranchIndex,
    );

    expect(issueBranchIndex).toBeGreaterThan(0);
    expect(issueTableIndex).toBeGreaterThan(issueBranchIndex);
    expect(issueOverviewIndex).toBeGreaterThan(issueBranchIndex);
    expect(issueTableIndex).toBeLessThan(issueOverviewIndex);
    expect(allBranchIndex).toBeGreaterThan(issueBranchIndex);
    expect(allOverviewIndex).toBeGreaterThan(allBranchIndex);
    expect(allTableIndex).toBeGreaterThan(allBranchIndex);
    expect(allOverviewIndex).toBeLessThan(allTableIndex);
  });

  it("renders an immediate table-area empty state when issue mode has no matching rows", () => {
    expect(diagnosticsPageSource).toContain("visibleDiagnosticsTableSections");
    expect(diagnosticsPageSource).toContain('class="diagnostics-table-area diagnostics-tables"');
    expect(diagnosticsPageSource).toContain('class="diagnostics-overview-area"');
    expect(diagnosticsPageSource).toContain("No diagnostic issue rows match this view.");
  });
});
