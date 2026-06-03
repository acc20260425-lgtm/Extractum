import { describe, expect, it } from "vitest";
import diagnosticsTableSource from "./components/diagnostics/DiagnosticCountTable.svelte?raw";
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
});
