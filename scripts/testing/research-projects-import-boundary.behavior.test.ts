import path from "node:path";
import { expect, it } from "vitest";

import { createRepositoryIndex } from "./repository-index.mjs";

const repoRoot = path.resolve(".");
const index = createRepositoryIndex({ root: repoRoot });
const featurePaths = index
  .listFiles()
  .filter((candidate: string) =>
    (candidate.startsWith("src/lib/components/research-projects/") || candidate.startsWith("src/routes/projects/"))
      && (candidate.endsWith(".svelte") || candidate.endsWith(".ts")),
  );
const forbidden = new Set(["bits-ui", "@svar-ui/svelte-grid", "@svar-ui/svelte-core"]);

function directUiImports(candidate: string) {
  const facts = candidate.endsWith(".svelte") ? index.getSvelte(candidate) : index.getTypeScript(candidate);
  return facts.imports.filter((entry: { source: string }) =>
    forbidden.has(entry.source) || entry.source.startsWith("$lib/components/ui/"),
  );
}

  it("feature screens use Extractum UI wrappers", () => {
    expect(featurePaths.filter((candidate: string) => directUiImports(candidate).length > 0)).toEqual([]);
  });

  it("lower-level UI imports stay in wrapper modules", () => {
    const dataGrid = index.getSvelte("src/lib/components/extractum-ui/DataGrid.svelte");
    const treeGrid = index.getSvelte("src/lib/components/extractum-ui/TreeDataGrid.svelte");
    const dialog = index.getSvelte("src/lib/components/extractum-ui/Dialog.svelte");
    const status = index.getSvelte("src/lib/components/extractum-ui/StatusMessage.svelte");
    expect(dataGrid.imports.some((entry: { source: string }) => entry.source === "@svar-ui/svelte-grid")).toBe(true);
    expect(treeGrid.imports.some((entry: { source: string }) => entry.source === "@svar-ui/svelte-grid")).toBe(true);
    expect(dialog.imports.some((entry: { source: string }) => entry.source === "$lib/components/ui/dialog/index.js")).toBe(true);
    expect(status.imports.some((entry: { source: string }) => entry.source === "$lib/components/ui/StatusMessage.svelte")).toBe(true);
    expect(dataGrid.imports.some((entry: { source: string }) => entry.source.startsWith("src/lib/new-ui"))).toBe(false);
    expect(treeGrid.imports.some((entry: { source: string }) => entry.source.startsWith("src/lib/new-ui"))).toBe(false);
    expect(dialog.imports.some((entry: { source: string }) => entry.source.startsWith("src/lib/new-ui"))).toBe(false);
  });

  it("Library screens do not bypass Extractum wrappers", () => {
    const libraryPaths = featurePaths.filter((candidate: string) =>
      candidate === "src/routes/projects/library/+page.svelte"
        || path.posix.basename(candidate).startsWith("Library"),
    );
    expect(libraryPaths).toContain("src/lib/components/research-projects/LibraryAddSourceDialog.svelte");
    expect(libraryPaths.filter((candidate: string) => directUiImports(candidate).length > 0)).toEqual([]);
  });
