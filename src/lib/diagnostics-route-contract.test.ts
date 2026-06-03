import { describe, expect, it } from "vitest";
import layoutSource from "../routes/+layout.svelte?raw";
import diagnosticsPageSource from "../routes/diagnostics/+page.svelte?raw";
import diagnosticsTableSource from "./components/diagnostics/DiagnosticCountTable.svelte?raw";
import settingsPageSource from "../routes/settings/+page.svelte?raw";

const productionSources = [diagnosticsPageSource, diagnosticsTableSource];

describe("diagnostics frontend source contracts", () => {
  it("keeps Tauri invocation inside the diagnostics API wrapper", () => {
    expect(diagnosticsPageSource).toContain(
      'import { loadDiagnosticSummary } from "$lib/api/diagnostics";',
    );
    expect(diagnosticsPageSource).not.toContain("invoke(");
  });

  it("keeps raw payload, log, and copy affordances out of diagnostics production UI", () => {
    const forbidden = [
      "JSON.stringify",
      "Raw JSON",
      "Copy payload",
      "Copy JSON",
      "Copy logs",
      "Copy table",
      "Copy section",
      "Copy summary",
      "stack trace",
      "console.error",
    ];

    for (const source of productionSources) {
      for (const token of forbidden) {
        expect(source).not.toContain(token);
      }
    }
  });

  it("loads diagnostics only from mount and manual refresh state", () => {
    expect(diagnosticsPageSource).toMatch(/import\s*\{\s*onMount\s*\}\s*from\s*"svelte"/);
    expect(diagnosticsPageSource).toMatch(/onMount\s*\(\s*\(\)\s*=>/);
    expect(diagnosticsPageSource).toMatch(/refreshDiagnostics\s*\(\s*true\s*\)/);
    expect(diagnosticsPageSource).toMatch(/refreshDiagnostics\s*\(\s*false\s*\)/);
    expect(diagnosticsPageSource).not.toContain("export const load");
    expect(diagnosticsPageSource).not.toContain("setInterval");
  });

  it("keeps refresh failure state separate from the last successful summary", () => {
    expect(diagnosticsPageSource).toMatch(/let\s+summary\s*=\s*\$state(?:\s*<[^>]+>)?\s*\(\s*null\s*\)/);
    expect(diagnosticsPageSource).toMatch(/let\s+loading\s*=\s*\$state\s*\(\s*true\s*\)/);
    expect(diagnosticsPageSource).toMatch(/let\s+refreshing\s*=\s*\$state\s*\(\s*false\s*\)/);
    expect(diagnosticsPageSource).toMatch(/let\s+status\s*=\s*\$state\s*\(\s*""\s*\)/);
    expect(diagnosticsPageSource).toMatch(/let\s+error\s*=\s*\$state(?:\s*<[^>]+>)?\s*\(\s*null\s*\)/);
    expect(diagnosticsPageSource).toMatch(/if\s*\(\s*initial\s*\)\s*(?:\{\s*)?summary\s*=\s*null\s*;/);
  });

  it("does not render duplicate initial loading status", () => {
    expect(diagnosticsPageSource).toMatch(/if\s*\(\s*initial\s*\)\s*\{[\s\S]*status\s*=\s*"";/);
    expect(diagnosticsPageSource).toMatch(/else\s*\{[\s\S]*status\s*=\s*"Refreshing\.\.\.";/);
  });

  it("keeps privacy fallback tolerant of partial privacy payloads", () => {
    expect(diagnosticsPageSource).toContain("function privacyLabels");
    expect(diagnosticsPageSource).toContain("function privacyNote");
    expect(diagnosticsPageSource).toMatch(/summary\.privacy\?\.\s*excludedDataClasses/);
    expect(diagnosticsPageSource).not.toContain("{@const excludedClasses");
    expect(diagnosticsPageSource).not.toContain("{@const fallbackNote");
  });

  it("adds Diagnostics navigation without moving diagnostics into Settings", () => {
    expect(layoutSource).toContain("ShieldCheck");
    expect(layoutSource).toContain('label: "Diagnostics"');
    expect(layoutSource).toContain('caption: "Local health"');
    expect(layoutSource).toContain('pathname.startsWith("/diagnostics")');
    expect(layoutSource).toContain("Diagnostics");
    expect(settingsPageSource).not.toContain("$lib/api/diagnostics");
    expect(settingsPageSource).not.toContain("/diagnostics");
    expect(settingsPageSource).not.toContain("loadDiagnosticSummary");
  });
});
