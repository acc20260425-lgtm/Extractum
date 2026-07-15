import { describe, expect, it } from "vitest";
import projectRailPanelSource from "./components/research-projects/ProjectRailPanel.svelte?raw";
import sourcesTabSource from "./components/research-projects/SourcesTab.svelte?raw";

const ROOT_LUCIDE_IMPORT = /from\s+["']@lucide\/svelte["']/;
const LUCIDE_PACKAGE_IMPORT = /from\s+["'](@lucide\/svelte[^"']*)["']/g;

function normalized(source: string): string {
  return source.replace(/\r\n/g, "\n");
}

function lucidePackageImports(source: string): string[] {
  return [...normalized(source).matchAll(LUCIDE_PACKAGE_IMPORT)].map((match) => {
    const specifier = match[1];
    if (!specifier) {
      throw new Error("Lucide import regex matched without a module specifier");
    }
    return specifier;
  });
}

describe("research-project Lucide import boundaries", () => {
  it.each([
    ["ProjectRailPanel", projectRailPanelSource],
    ["SourcesTab", sourcesTabSource],
  ])("keeps %s off the Lucide root barrel", (_name, rawSource) => {
    const source = normalized(rawSource);
    const imports = lucidePackageImports(source);

    expect(source).not.toMatch(ROOT_LUCIDE_IMPORT);
    expect(imports.every((specifier) => specifier.startsWith("@lucide/svelte/icons/"))).toBe(true);
  });
});
