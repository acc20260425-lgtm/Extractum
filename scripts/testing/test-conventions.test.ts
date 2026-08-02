import { describe, expect, it } from "vitest";

const sourceTests = import.meta.glob("/src/**/*.test.ts", { query: "?raw", import: "default", eager: true });
const scriptTests = import.meta.glob("/scripts/**/*.test.ts", { query: "?raw", import: "default", eager: true });
const sidecarTests = import.meta.glob("/sidecars/**/*.test.ts", { query: "?raw", import: "default", eager: true });
const researchTests = import.meta.glob("/research/**/*.test.ts", { query: "?raw", import: "default", eager: true });
const testSources = { ...sourceTests, ...scriptTests, ...sidecarTests, ...researchTests } as Record<string, string>;
const environmentMarker = "@vitest-environment " + "jsdom";

describe("test conventions", () => {
  it("gives jsdom component tests component ownership and cleanup", () => {
    for (const [testPath, source] of Object.entries(testSources)) {
      if (!source.includes(environmentMarker)) continue;

      expect(testPath).toMatch(/\.component\.test\.ts$/);
      expect(source).toMatch(/import\s*\{[^}]*\bcleanup\b[^}]*\}\s*from\s*["']@testing-library\/svelte["']/);
      expect((source.match(/afterEach\(cleanup\)/g) ?? [])).toHaveLength(1);
    }
  });
});
