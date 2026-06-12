// @ts-nocheck
import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const repoRoot = path.resolve(fileURLToPath(new URL("../..", import.meta.url)));

function collectFiles(relativeDir: string): string[] {
  const fullDir = path.join(repoRoot, relativeDir);
  if (!existsSync(fullDir)) return [];
  return readdirSync(fullDir).flatMap((entry) => {
    const fullPath = path.join(fullDir, entry);
    const stat = statSync(fullPath);
    if (stat.isDirectory()) return collectFiles(path.relative(repoRoot, fullPath));
    return [fullPath];
  });
}

function sourceOf(file: string) {
  return readFileSync(file, "utf8");
}

describe("research projects import boundaries", () => {
  it("keeps new feature screens behind Extractum UI wrappers", () => {
    const featureFiles = [
      ...collectFiles("src/lib/components/research-projects"),
      ...collectFiles("src/routes/projects"),
    ].filter((file) => file.endsWith(".svelte") || file.endsWith(".ts"));

    const offenders = featureFiles
      .map((file) => [path.relative(repoRoot, file).replaceAll("\\", "/"), sourceOf(file)] as const)
      .filter(([, source]) =>
        source.includes("@svar-ui/") ||
        source.includes("bits-ui") ||
        source.includes("$lib/components/ui/"),
      )
      .map(([file]) => file);

    expect(offenders).toEqual([]);
  });

  it("allows lower-level library imports only in the product wrapper layer", () => {
    const wrapperFiles = collectFiles("src/lib/components/extractum-ui");
    const wrapperSources = wrapperFiles.map(sourceOf).join("\n");

    expect(wrapperSources).toContain("$lib/components/ui/button/index.js");
    expect(wrapperSources).toContain("$lib/components/ui/sheet/index.js");
    expect(wrapperSources).not.toContain("src/lib/new-ui");
  });
});
