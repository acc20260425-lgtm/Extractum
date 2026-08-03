import path from "node:path";
import * as svelteCompiler from "svelte/compiler";
import tsCompiler from "typescript";
import { describe, expect, it, vi } from "vitest";

import { createRepositoryIndex } from "./repository-index.mjs";

const root = path.resolve("repository-index-fixture");

function fixtureIndex(sources: Record<string, string>, cargoMetadata: unknown = { packages: [] }) {
  const readFile = vi.fn((absolutePath: string) => {
    const relativePath = path.relative(root, absolutePath).replaceAll("\\", "/");
    const source = sources[relativePath];
    if (source === undefined) throw new Error(`missing fixture: ${relativePath}`);
    return source;
  });
  const typescript = {
    ...tsCompiler,
    createSourceFile: vi.fn(tsCompiler.createSourceFile),
  };
  const svelte = {
    ...svelteCompiler,
    parse: vi.fn(svelteCompiler.parse),
  };
  const loadCargoMetadata = vi.fn(() => JSON.stringify(cargoMetadata));

  return {
    index: createRepositoryIndex({ root, readFile, ts: typescript, svelte, loadCargoMetadata }),
    loadCargoMetadata,
    readFile,
    svelte,
    typescript,
  };
}

describe("RepositoryIndex", () => {
  it("returns parsed TypeScript import facts", () => {
    const { index } = fixtureIndex({
      "src/example.ts": 'import fallback, { parse as parseValue, type Options } from "@scope/parser";\nexport const value = 1;',
    });

    expect(index.getTypeScript("src/example.ts").imports).toEqual([
      {
        source: "@scope/parser",
        defaultImport: "fallback",
        namedImports: [
          { imported: "parse", local: "parseValue", typeOnly: false },
          { imported: "Options", local: "Options", typeOnly: true },
        ],
        namespaceImport: null,
        typeOnly: false,
      },
    ]);
  });

  it("returns parsed Svelte component facts", () => {
    const { index } = fixtureIndex({
      "src/example.svelte": '<script lang="ts">import Child from "./Child.svelte";</script>\n<Child answer={42} />',
    });

    expect(index.getSvelte("src/example.svelte").components).toEqual([
      { name: "Child", attributes: ["answer"] },
    ]);
  });

  it("parses each declared input once for one immutable snapshot", () => {
    const fixture = fixtureIndex({
      "src/example.ts": 'import "first";',
      "src/example.svelte": "<Widget />",
    }, { packages: [{ name: "fixture" }] });

    const firstTypeScript = fixture.index.getTypeScript("src/example.ts");
    const firstSvelte = fixture.index.getSvelte("src/example.svelte");
    const firstCargo = fixture.index.getCargoMetadata();
    const secondTypeScript = fixture.index.getTypeScript("src/example.ts");
    const secondSvelte = fixture.index.getSvelte("src/example.svelte");
    const secondCargo = fixture.index.getCargoMetadata();

    expect(secondTypeScript).toBe(firstTypeScript);
    expect(secondSvelte).toBe(firstSvelte);
    expect(secondCargo).toBe(firstCargo);
    expect(fixture.readFile).toHaveBeenCalledTimes(2);
    expect(fixture.typescript.createSourceFile).toHaveBeenCalledOnce();
    expect(fixture.svelte.parse).toHaveBeenCalledOnce();
    expect(fixture.loadCargoMetadata).toHaveBeenCalledOnce();
  });

  it("throws a path-qualified error for malformed TypeScript", () => {
    const { index } = fixtureIndex({ "src/broken.ts": "export const = ;" });

    expect(() => index.getTypeScript("src/broken.ts")).toThrow(/src\/broken\.ts/);
  });

  it("retains a parse failure without reparsing the malformed snapshot input", () => {
    const fixture = fixtureIndex({ "src/broken.ts": "export const = ;" });

    expect(() => fixture.index.getTypeScript("src/broken.ts")).toThrow(/src\/broken\.ts/);
    expect(() => fixture.index.getTypeScript("src/broken.ts")).toThrow(/src\/broken\.ts/);
    expect(fixture.readFile).toHaveBeenCalledOnce();
    expect(fixture.typescript.createSourceFile).toHaveBeenCalledOnce();
  });

  it("throws a path-qualified error for malformed Svelte", () => {
    const { index } = fixtureIndex({ "src/broken.svelte": "<script>const value = ;</script>" });

    expect(() => index.getSvelte("src/broken.svelte")).toThrow(/src\/broken\.svelte/);
  });
});
