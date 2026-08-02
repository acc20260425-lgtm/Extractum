import { configDefaults, defineConfig } from "vitest/config";
import { sveltekit } from "@sveltejs/kit/vite";
import tailwindcss from "@tailwindcss/vite";
import { svelteTesting } from "@testing-library/svelte/vite";
import sourceContractLedger from "./testing/source-contract-ledger.json";

const COMPONENT_TEST_PATTERN = "src/**/*.component.test.ts";
const DEFAULT_TEST_PATTERN = "**/*.{test,spec}.?(c|m)[jt]s?(x)";
const OS_INTEGRATION_FILES = Object.freeze([
  "scripts/process-shell-diagnostic/attempt.test.ts",
  "scripts/process-shell-diagnostic/coordinator.test.ts",
  "scripts/process-shell-diagnostic/git-state.test.ts",
  "scripts/process-shell-diagnostic/protocol.test.ts",
  "scripts/process-shell-diagnostic/report.test.ts",
  "scripts/process-shell-diagnostic/runtime.test.ts",
]);
const ARCHITECTURE_FILES = Object.freeze([
  "src/lib/lucide-direct-import-contract.test.ts",
]);
const PLAYWRIGHT_TESTS = Object.freeze([
  "research/gemini_browser_adapter/tests/**/*.spec.ts",
]);
const DEFAULT_PROJECT_EXCLUDES = Object.freeze([...configDefaults.exclude]);

function normalizeLedgerPath(path: string) {
  return path.replaceAll("\\", "/").replace(/^\.\//, "");
}

const ledgerFiles = [...new Set(
  sourceContractLedger.rows
    .map((row) => normalizeLedgerPath(row.path))
    .filter(Boolean),
)].sort();

export const LEGACY_TEST_FILES = Object.freeze(ledgerFiles.filter((path) =>
  !path.endsWith(".component.test.ts")
  && !OS_INTEGRATION_FILES.includes(path)
  && !ARCHITECTURE_FILES.includes(path),
));

export const VITEST_PROJECT_DEFINITIONS = Object.freeze([
  Object.freeze({
    name: "unit-node",
    include: Object.freeze([DEFAULT_TEST_PATTERN]),
    exclude: Object.freeze([
      ...DEFAULT_PROJECT_EXCLUDES,
      COMPONENT_TEST_PATTERN,
      ...OS_INTEGRATION_FILES,
      ...ARCHITECTURE_FILES,
      ...LEGACY_TEST_FILES,
      ...PLAYWRIGHT_TESTS,
    ]),
    environment: "node",
    pool: "threads",
  }),
  Object.freeze({
    name: "component",
    include: Object.freeze([COMPONENT_TEST_PATTERN]),
    exclude: DEFAULT_PROJECT_EXCLUDES,
    environment: "jsdom",
    pool: "threads",
    setupFiles: Object.freeze(["./scripts/testing/setup-component-tests.ts"]),
    svelteTestingOptions: Object.freeze({ autoCleanup: false }),
  }),
  Object.freeze({
    name: "architecture",
    include: Object.freeze([...ARCHITECTURE_FILES]),
    exclude: DEFAULT_PROJECT_EXCLUDES,
    environment: "node",
    pool: "threads",
  }),
  Object.freeze({
    name: "legacy-contract",
    include: Object.freeze([...LEGACY_TEST_FILES]),
    exclude: DEFAULT_PROJECT_EXCLUDES,
    environment: "node",
    pool: "threads",
  }),
  Object.freeze({
    name: "os-integration",
    include: Object.freeze([...OS_INTEGRATION_FILES]),
    exclude: DEFAULT_PROJECT_EXCLUDES,
    environment: "node",
    pool: "forks",
  }),
]);

export default defineConfig(() => {
  const sharedPlugins = [tailwindcss(), sveltekit()];

  return {
    plugins: sharedPlugins,
    test: {
      projects: VITEST_PROJECT_DEFINITIONS.map((definition) => ({
        extends: true as const,
        plugins: "svelteTestingOptions" in definition
          ? [svelteTesting(definition.svelteTestingOptions)]
          : [],
        test: {
          name: definition.name,
          include: [...definition.include],
          exclude: [...definition.exclude],
          environment: definition.environment,
          pool: definition.pool,
          setupFiles: "setupFiles" in definition ? [...definition.setupFiles] : undefined,
        },
      })),
    },
  };
});
