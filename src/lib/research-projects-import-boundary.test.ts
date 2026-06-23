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
    expect(wrapperSources).toContain("$lib/components/ui/dialog/index.js");
    expect(wrapperSources).toContain("$lib/components/ui/StatusMessage.svelte");
    expect(wrapperSources).toContain("ExtractumDialog");
    expect(wrapperSources).toContain("ExtractumStatusMessage");
    expect(wrapperSources).not.toContain("src/lib/new-ui");
  });

  it("keeps Library route and feature screens out of direct shadcn and SVAR imports", () => {
    const libraryFiles = [
      path.join(repoRoot, "src/routes/projects/library/+page.svelte"),
      ...collectFiles("src/lib/components/research-projects")
        .filter((file) => path.basename(file).startsWith("Library")),
    ];

    const offenders = libraryFiles
      .map((file) => [path.relative(repoRoot, file).replaceAll("\\", "/"), sourceOf(file)] as const)
      .filter(([, source]) =>
        source.includes("@svar-ui/") ||
        source.includes("bits-ui") ||
        source.includes("$lib/components/ui/"),
      )
      .map(([file]) => file);

    expect(libraryFiles.some((file) => path.basename(file) === "LibraryAddSourceDialog.svelte")).toBe(true);
    expect(offenders).toEqual([]);
  });

  it("routes SVAR Grid through Extractum grid wrappers only", () => {
    const dataGridSource = readFileSync(
      path.join(repoRoot, "src/lib/components/extractum-ui/DataGrid.svelte"),
      "utf8",
    );
    expect(dataGridSource).toContain('from "@svar-ui/svelte-grid"');
    expect(dataGridSource).toContain("selectedRows");
    expect(dataGridSource).toContain("rowStyle");
    expect(dataGridSource).toContain("Locale");
    expect(dataGridSource).toContain("Willow");
    expect(dataGridSource).toContain("fonts={false}");
    expect(dataGridSource).toContain("visibleOverlay");
    expect(dataGridSource).toContain("rows.length === 0 ? overlay : undefined");
    expect(dataGridSource).toContain("enhanceDateTimeColumns");
    expect(dataGridSource).toContain("enhancedColumns");

    const treeGridSource = readFileSync(
      path.join(repoRoot, "src/lib/components/extractum-ui/TreeDataGrid.svelte"),
      "utf8",
    );
    expect(treeGridSource).toContain('from "@svar-ui/svelte-grid"');
    expect(treeGridSource).toContain("tree");
    expect(treeGridSource).toContain("treetoggle");
    expect(treeGridSource).toContain("selectedRows");
    expect(treeGridSource).toContain("onselectrow");
    expect(treeGridSource).toContain("Willow");
    expect(treeGridSource).toContain("Locale");
    expect(treeGridSource).toContain("fonts={false}");
    expect(treeGridSource).toContain(".extractum-tree-data-grid :global(.wx-");

    const selectCellSource = readFileSync(
      path.join(repoRoot, "src/lib/components/extractum-ui/GridSelectCell.svelte"),
      "utf8",
    );
    expect(selectCellSource).toContain('data-action="ignore-click"');
    expect(selectCellSource).toContain('api.exec("select-row"');
  });
});
