import { describe, expect, it, vi } from "vitest";
import ts from "typescript";
import packageJson from "../../package.json";
import sourceContractLedger from "../../testing/source-contract-ledger.json";
import * as vitestConfiguration from "../../vitest.config";
import { discoverSourceReaders } from "./extract-source-contract-ledger.mjs";
import { createRepositoryIndex } from "./repository-index.mjs";
import { evaluateRule, registeredRuleIds } from "./repository-rules.mjs";
import componentSetupSource from "./setup-component-tests.ts?raw";

const { LEGACY_TEST_FILES, VITEST_PROJECT_DEFINITIONS } = vitestConfiguration;

const sourceTests = import.meta.glob("/src/**/*.{test,spec}.{js,jsx,ts,tsx,cjs,cjsx,cts,ctsx,mjs,mjsx,mts,mtsx}", { query: "?raw", import: "default", eager: true });
const scriptTests = import.meta.glob("/scripts/**/*.{test,spec}.{js,jsx,ts,tsx,cjs,cjsx,cts,ctsx,mjs,mjsx,mts,mtsx}", { query: "?raw", import: "default", eager: true });
const sidecarTests = import.meta.glob("/sidecars/**/*.{test,spec}.{js,jsx,ts,tsx,cjs,cjsx,cts,ctsx,mjs,mjsx,mts,mtsx}", { query: "?raw", import: "default", eager: true });
const researchTests = import.meta.glob("/research/**/*.{test,spec}.{js,jsx,ts,tsx,cjs,cjsx,cts,ctsx,mjs,mjsx,mts,mtsx}", { query: "?raw", import: "default", eager: true });
const testSources = Object.fromEntries(Object.entries({
  ...sourceTests,
  ...scriptTests,
  ...sidecarTests,
  ...researchTests,
}).filter(([testPath]) => !/^\/research\/gemini_browser_adapter\/tests\/.*\.spec\.ts$/.test(testPath))) as Record<string, string>;
const environmentMarker = "@vitest-environment " + "jsdom";
const chromiumLauncherImport = new RegExp(
  String.raw`^\s*import\s+(?=[^;]*\bchromium\b)[^;]*\s+from\s+["']@playwright/test["']\s*;?`,
  "m",
);
type ProjectConvention = {
  name: string;
  include?: readonly string[];
  exclude?: readonly string[];
  setupFiles?: readonly string[];
  svelteTestingOptions?: Readonly<{ autoCleanup: boolean }>;
};
const projectConventions = VITEST_PROJECT_DEFINITIONS as readonly ProjectConvention[];

const approvedStructuredSourceAuthorities = [
  "scripts/testing/repository-index.mjs",
  "scripts/testing/repository-rules.mjs",
] as const;

function isInspectedVitestSource(testPath: string) {
  return /\.(?:test|spec)\.(?:[cm]?[jt]sx?)$/i.test(testPath);
}

function directTextReaderViolations(sources: Record<string, string>) {
  const approvedOwners = new Set([
    ...sourceContractLedger.rows.map(({ path }) => path),
    ...sourceContractLedger.sourceReaderExceptions.map(({ path }) => path),
    ...(projectConventions.find(({ name }) => name === "architecture")?.include ?? []),
    "scripts/testing/repository-index.test.ts",
    "scripts/testing/repository-rules.test.ts",
    "scripts/testing/test-conventions.test.ts",
  ]);
  const entries = Object.entries(sources)
    .filter(([testPath]) => isInspectedVitestSource(testPath))
    .map(([testPath, source]) => ({
      path: testPath.replaceAll("\\", "/").replace(/^\//, ""),
      source,
    }));
  const readers = discoverSourceReaders(entries, new Set(), ts);
  const readerOwners = new Set(readers
    .filter(({ classification, kind }) => kind === "manual" || !["fixture", "generated", "ignored", "output", "temp", "test"].includes(classification ?? ""))
    .map(({ path }) => path));

  return [...readerOwners]
    .filter((testPath) => !approvedOwners.has(testPath))
    .sort()
    .map((testPath) => `${testPath}: direct text source reader is not an approved index or fixture owner`);
}

function componentTestEntries(sources: Record<string, string>) {
  return Object.entries(sources).filter(([testPath]) => testPath.endsWith(".component.test.ts"));
}

const expectedComponentSetupSource = [
  'import { setup } from "@testing-library/svelte/pure";',
  'import { beforeEach } from "vitest";',
  "",
  "beforeEach(setup);",
].join("\n");
// Keep this raw-source guard outside a test declaration so it cannot become a
// source-contract obligation; a mismatch still fails the convention suite at import.
if (componentSetupSource.replaceAll("\r\n", "\n").trim() !== expectedComponentSetupSource) {
  throw new Error("component setup must register beforeEach(setup) without cleanup or teardown");
}

describe("test conventions", () => {
  it("allows inline jsdom ownership only in component tests", () => {
    for (const [testPath, source] of Object.entries(testSources)) {
      if (!source.includes(environmentMarker)) continue;

      expect(testPath).toMatch(/\.component\.test\.ts$/);
    }
  });

  it("gives every component-owned test exactly one file-local cleanup owner", () => {
    for (const [, source] of componentTestEntries(testSources)) {
      expect(source).toMatch(/import\s*\{[^}]*\bcleanup\b[^}]*\}\s*from\s*["']@testing-library\/svelte["']/);
      expect((source.match(/afterEach\(cleanup\)/g) ?? [])).toHaveLength(1);
      expect((source.match(/\bcleanup\s*\(/g) ?? [])).toHaveLength(0);
    }
  });

  it("includes a component suffix without an inline jsdom marker in the cleanup cohort", () => {
    const markerlessPath = "/src/markerless.component.test.ts";
    const markerlessSource = [
      'import { cleanup } from "@testing-library/svelte";',
      "afterEach(cleanup);",
    ].join("\n");

    expect(componentTestEntries({ [markerlessPath]: markerlessSource }).map(([testPath]) => testPath))
      .toContain(markerlessPath);
  });

  it("keeps Testing Library setup component-only without plugin cleanup", () => {
    const componentProject = projectConventions.find(({ name }) => name === "component");

    expect(componentProject).toMatchObject({
      setupFiles: ["./scripts/testing/setup-component-tests.ts"],
      svelteTestingOptions: { autoCleanup: false },
    });
    for (const project of projectConventions.filter(({ name }) => name !== "component")) {
      expect(project.setupFiles ?? []).not.toContain("./scripts/testing/setup-component-tests.ts");
      expect(project.svelteTestingOptions).toBeUndefined();
    }
  });

  it("maps component setup and cleanup options into the runtime project config", () => {
    type RuntimeProject = {
      plugins: unknown[];
      test: { name: string; setupFiles?: string[] };
    };
    type ProjectMapper = (pluginFactory: (options: { autoCleanup: boolean }) => unknown) => RuntimeProject[];
    const createVitestProjects = (vitestConfiguration as unknown as {
      createVitestProjects?: ProjectMapper;
    }).createVitestProjects;

    expect(typeof createVitestProjects).toBe("function");
    if (!createVitestProjects) throw new Error("createVitestProjects is not exported");

    const componentPlugin = { name: "stub-component-plugin" };
    const pluginFactory = vi.fn(() => componentPlugin);
    const projects = createVitestProjects(pluginFactory);
    const componentProject = projects.find(({ test }) => test.name === "component");

    expect(pluginFactory).toHaveBeenCalledOnce();
    expect(pluginFactory).toHaveBeenCalledWith({ autoCleanup: false });
    expect(componentProject).toMatchObject({
      plugins: [componentPlugin],
      test: { setupFiles: ["./scripts/testing/setup-component-tests.ts"] },
    });
    for (const project of projects.filter(({ test }) => test.name !== "component")) {
      expect(project.plugins).toEqual([]);
      expect(project.test.setupFiles).toBeUndefined();
    }
  });

  it("gives component behavior ownership only to the component project", () => {
    const componentBehaviorPattern = "src/lib/components/**/*.behavior.test.ts";
    const componentProject = projectConventions.find(({ name }) => name === "component");
    const unitNodeProject = projectConventions.find(({ name }) => name === "unit-node");

    expect(componentProject?.include).toContain(componentBehaviorPattern);
    expect(unitNodeProject?.exclude).toContain(componentBehaviorPattern);
    expect(componentBehaviorPattern).toMatch(/^src\/lib\/components\/.*\.behavior\.test\.ts$/);
    expect("src/lib/telegram-checkpoint-2.behavior.test.ts").not.toMatch(/^src\/lib\/components\//);
    expect("src/lib/telegram-contract-paths.behavior.test.ts").not.toMatch(/^src\/lib\/components\//);
  });

  it("keeps Chromium launch ownership out of Vitest test sources", () => {
    for (const source of Object.values(testSources)) {
      expect(source).not.toMatch(chromiumLauncherImport);
    }
  });

  it("keeps production source inspection behind the structured rule runner", () => {
    expect(approvedStructuredSourceAuthorities).toEqual([
      "scripts/testing/repository-index.mjs",
      "scripts/testing/repository-rules.mjs",
    ]);
    expect(typeof createRepositoryIndex).toBe("function");
    expect(typeof evaluateRule).toBe("function");
    expect(registeredRuleIds).toEqual([
      "rule:analysis-evidence-highlight-token-styling",
      "rule:analysis-source-browser-canonical-composition",
      "rule:analysis-source-browser-explicit-subject-contract",
      "rule:analysis-source-group-activity-boundary",
      "rule:analysis-source-group-tab-leaf-boundary",
      "rule:analysis-source-reader-surface-composition",
      "rule:telegram-crate-dependency-ownership",
      "rule:telegram-crate-manifest-boundary",
      "rule:telegram-phase-8b-authority-integrity",
    ]);

    expect(directTextReaderViolations({
      "/src/lib/unapproved.test.ts": 'import productionSource from "./production.ts?raw";\nexpect(productionSource).toBeDefined();',
    })).toEqual([
      "src/lib/unapproved.test.ts: direct text source reader is not an approved index or fixture owner",
    ]);
    expect(directTextReaderViolations({
      "/src/lib/unapproved.spec.ts": 'import productionSource from "./production.ts?raw";\nexpect(productionSource).toBeDefined();',
    })).toEqual([
      "src/lib/unapproved.spec.ts: direct text source reader is not an approved index or fixture owner",
    ]);
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
