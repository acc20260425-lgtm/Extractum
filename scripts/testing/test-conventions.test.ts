import { describe, expect, it } from "vitest";
import packageJson from "../../package.json";
import { LEGACY_TEST_FILES, VITEST_PROJECT_DEFINITIONS } from "../../vitest.config";

const sourceTests = import.meta.glob("/src/**/*.test.ts", { query: "?raw", import: "default", eager: true });
const scriptTests = import.meta.glob("/scripts/**/*.test.ts", { query: "?raw", import: "default", eager: true });
const sidecarTests = import.meta.glob("/sidecars/**/*.test.ts", { query: "?raw", import: "default", eager: true });
const researchTests = import.meta.glob("/research/**/*.test.ts", { query: "?raw", import: "default", eager: true });
const testSources = { ...sourceTests, ...scriptTests, ...sidecarTests, ...researchTests } as Record<string, string>;
const environmentMarker = "@vitest-environment " + "jsdom";
const chromiumLauncherImport = new RegExp(
  String.raw`^\s*import\s+(?=[^;]*\bchromium\b)[^;]*\s+from\s+["']@playwright/test["']\s*;?`,
  "m",
);

describe("test conventions", () => {
  it("gives jsdom component tests component ownership and cleanup", () => {
    for (const [testPath, source] of Object.entries(testSources)) {
      if (!source.includes(environmentMarker)) continue;

      expect(testPath).toMatch(/\.component\.test\.ts$/);
      expect(source).toMatch(/import\s*\{[^}]*\bcleanup\b[^}]*\}\s*from\s*["']@testing-library\/svelte["']/);
      expect((source.match(/afterEach\(cleanup\)/g) ?? [])).toHaveLength(1);
    }
  });

  it("keeps Chromium launch ownership out of Vitest test sources", () => {
    for (const source of Object.values(testSources)) {
      expect(source).not.toMatch(chromiumLauncherImport);
    }
  });

  it("keeps the Vitest project commands and ledger-derived legacy inventory explicit", () => {
    expect({
      "test:unit": packageJson.scripts["test:unit"],
      "test:component": packageJson.scripts["test:component"],
      "test:architecture": packageJson.scripts["test:architecture"],
      "test:legacy-contract": packageJson.scripts["test:legacy-contract"],
      "test:integration:os": packageJson.scripts["test:integration:os"],
    }).toEqual({
      "test:unit": "node scripts/run-vitest.mjs run --project unit-node",
      "test:component": "node scripts/run-vitest.mjs run --project component",
      "test:architecture": "node scripts/run-vitest.mjs run --project architecture",
      "test:legacy-contract": "node scripts/run-vitest.mjs run --project legacy-contract",
      "test:integration:os": "node scripts/run-vitest.mjs run --project os-integration",
    });

    expect(VITEST_PROJECT_DEFINITIONS.map(({ name }) => name)).toEqual([
      "unit-node",
      "component",
      "architecture",
      "legacy-contract",
      "os-integration",
    ]);
    expect(VITEST_PROJECT_DEFINITIONS.every(({ exclude }) => exclude.includes("**/node_modules/**"))).toBe(true);

    expect(LEGACY_TEST_FILES).toEqual([
      "scripts/run-vitest.test.ts",
      "scripts/testing/run-observation.test.ts",
      "scripts/testing/slice-1-baseline.test.ts",
      "src/lib/accounts-route-add-account-modal.test.ts",
      "src/lib/accounts-ux-contract.test.ts",
      "src/lib/analysis-application-contract.test.ts",
      "src/lib/analysis-compact-source-rail.test.ts",
      "src/lib/analysis-companion-layout.test.ts",
      "src/lib/analysis-crate-boundary-contract.test.ts",
      "src/lib/analysis-evidence-source-navigation.test.ts",
      "src/lib/analysis-group-editor-props.test.ts",
      "src/lib/analysis-legacy-surfaces-cleanup.test.ts",
      "src/lib/analysis-llm-run-controls.test.ts",
      "src/lib/analysis-migration-fixture-contract.test.ts",
      "src/lib/analysis-priority-ux-contract.test.ts",
      "src/lib/analysis-redesign-route-contract.test.ts",
      "src/lib/analysis-redesign-safety-contract.test.ts",
      "src/lib/analysis-report-canvas-route.test.ts",
      "src/lib/analysis-report-canvas.test.ts",
      "src/lib/analysis-report-setup-props.test.ts",
      "src/lib/analysis-report-workspace-selection-props.test.ts",
      "src/lib/analysis-route-effects.test.ts",
      "src/lib/analysis-route-workspace-state.test.ts",
      "src/lib/analysis-run-companion-route.test.ts",
      "src/lib/analysis-run-companion-tabs.test.ts",
      "src/lib/analysis-source-access-placement.test.ts",
      "src/lib/analysis-source-readers-route.test.ts",
      "src/lib/analysis-source-readers.test.ts",
      "src/lib/analysis-state-legacy-selection-cleanup.test.ts",
      "src/lib/analysis-ui-smoke-contract.test.ts",
      "src/lib/analysis-workspace-tools.test.ts",
      "src/lib/analysis-youtube-source-specialization.test.ts",
      "src/lib/apalis-jobs-route-contract.test.ts",
      "src/lib/api/apalis-jobs.test.ts",
      "src/lib/api/diagnostics.test.ts",
      "src/lib/app-sidebar-behavior.test.ts",
      "src/lib/components/analysis/source-browser-shell.test.ts",
      "src/lib/components/extractum-ui/DataGrid.test.ts",
      "src/lib/components/research-projects/SourcesGrid.test.ts",
      "src/lib/crate-extraction-shell-cap-contract.test.ts",
      "src/lib/development-loop-performance-contract.test.ts",
      "src/lib/diagnostics-route-contract.test.ts",
      "src/lib/diagnostics-ux-contract.test.ts",
      "src/lib/dialog-bits-ui-migration.test.ts",
      "src/lib/external-process-lifecycle-contract.test.ts",
      "src/lib/focused-rust-loop-contract.test.ts",
      "src/lib/gemini-browser-crate-boundary-contract.test.ts",
      "src/lib/gemini-browser-provider-panel.test.ts",
      "src/lib/hidden-child-process-contract.test.ts",
      "src/lib/library-add-source-contract.test.ts",
      "src/lib/library-prototype-contract.test.ts",
      "src/lib/llm-crate-boundary-contract.test.ts",
      "src/lib/media-metadata-core-contract.test.ts",
      "src/lib/project-runs-screen-contract.test.ts",
      "src/lib/project-runs-tab-delete-contract.test.ts",
      "src/lib/prompt-pack-application-contract.test.ts",
      "src/lib/prompt-pack-completion-transport-contract.test.ts",
      "src/lib/prompt-pack-crate-boundary-contract.test.ts",
      "src/lib/prompt-pack-run-control-contract.test.ts",
      "src/lib/prompt-pack-run-store-contract.test.ts",
      "src/lib/prompt-pack-runtime-config-contract.test.ts",
      "src/lib/prompt-pack-stage-execution-contract.test.ts",
      "src/lib/prompt-pack-stage-request-policy-contract.test.ts",
      "src/lib/provider-test-console-placement.test.ts",
      "src/lib/research-projects-foundation-contract.test.ts",
      "src/lib/research-projects-import-boundary.test.ts",
      "src/lib/research-projects-lucide-import-contract.test.ts",
      "src/lib/research-projects-route-contract.test.ts",
      "src/lib/rust-workspace-core-contract.test.ts",
      "src/lib/settings-profile-ux-contract.test.ts",
      "src/lib/source-access-placement.test.ts",
      "src/lib/tauri-security-config-contract.test.ts",
      "src/lib/telegram-crate-boundary-contract.test.ts",
      "src/lib/youtube-summary-launch-contract.test.ts",
      "src/lib/youtube-summary-result-view-contract.test.ts",
      "src/lib/youtube-summary-smoke-fixture-contract.test.ts",
      "src/routes/projects/next/page-inspector.test.ts",
      "src/routes/projects/next/page-keyboard.test.ts",
    ]);
  });
});
