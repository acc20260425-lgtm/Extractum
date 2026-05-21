import { describe, expect, it } from "vitest";

const analysisComponentModules = import.meta.glob("./components/analysis/*.svelte", {
  eager: true,
  query: "?raw",
  import: "default",
});

const legacyAnalysisSurfaces = [
  "active-run-list.svelte",
  "run-controls.svelte",
  "run-history.svelte",
  "source-context-panel.svelte",
  "workspace-inspector.svelte",
  "workspace-main.svelte",
  "workspace-rail.svelte",
] as const;

describe("analysis legacy surface cleanup", () => {
  it("does not keep superseded pre-redesign analysis components in src", () => {
    const componentPaths = Object.keys(analysisComponentModules);

    for (const filename of legacyAnalysisSurfaces) {
      expect(
        componentPaths.some((path) => path.endsWith(`/${filename}`)),
        `${filename} should be archived or removed from the active frontend tree`,
      ).toBe(false);
    }
  });
});
