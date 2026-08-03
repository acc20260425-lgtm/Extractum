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

  it("retains predicate operators and operands as structured TypeScript guard facts", () => {
    const { index } = fixtureIndex({
      "src/guard.ts": 'function rejectDotSegment(segment: string) { if (segment === "." || segment === "..") throw new Error("dot segment"); }',
    });

    expect(index.getTypeScript("src/guard.ts").functions[0].guards).toEqual([
      {
        condition: {
          kind: "binary",
          operator: "||",
          left: {
            kind: "binary",
            operator: "===",
            left: { kind: "identifier", name: "segment" },
            right: { kind: "string", value: "." },
          },
          right: {
            kind: "binary",
            operator: "===",
            left: { kind: "identifier", name: "segment" },
            right: { kind: "string", value: ".." },
          },
        },
        consequenceThrows: true,
      },
    ]);
  });

  it("retains template operands in complete guard predicates", () => {
    const { index } = fixtureIndex({
      "src/guard.ts": 'function rejectParent(relative: string) { if (relative.startsWith(`..${path.sep}`)) throw new Error("parent path"); }',
    });

    expect(index.getTypeScript("src/guard.ts").functions[0].guards[0]).toEqual({
      condition: {
        kind: "call",
        callee: {
          kind: "member",
          object: { kind: "identifier", name: "relative" },
          property: "startsWith",
        },
        arguments: [{
          kind: "template",
          head: "..",
          spans: [{
            expression: {
              kind: "member",
              object: { kind: "identifier", name: "path" },
              property: "sep",
            },
            literal: "",
          }],
        }],
      },
      consequenceThrows: true,
    });
  });

  it("counts only throws executable in the guarded consequence scope", () => {
    const { index } = fixtureIndex({
      "src/guard.ts": [
        "function inspect(nestedOnly: boolean, sameScope: boolean) {",
        "  if (nestedOnly) {",
        '    function unused() { throw new Error("nested function"); }',
        '    class Deferred { run() { throw new Error("nested class"); } }',
        "  }",
        '  if (sameScope) { { throw new Error("same scope block"); } }',
        "}",
      ].join("\n"),
    });

    expect(index.getTypeScript("src/guard.ts").functions[0].guards.map(({ consequenceThrows }: any) => consequenceThrows))
      .toEqual([false, true]);
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
