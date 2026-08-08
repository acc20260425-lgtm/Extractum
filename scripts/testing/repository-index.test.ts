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
    index: createRepositoryIndex({
      root,
      readFile,
      ts: typescript,
      svelte,
      loadCargoMetadata,
      listFiles: () => Object.keys(sources),
    }),
    loadCargoMetadata,
    readFile,
    svelte,
    typescript,
  };
}

describe("RepositoryIndex", () => {
  it("returns the frozen repository-relative file inventory", () => {
    const { index } = fixtureIndex({
      "src/zeta.svelte": "<p>Zeta</p>",
      "src/alpha.ts": "export const alpha = true;",
    });

    expect(index.listFiles()).toEqual(["src/alpha.ts", "src/zeta.svelte"]);
    expect(Object.isFrozen(index.listFiles())).toBe(true);
  });

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

  it("returns structured Svelte import and style facts", () => {
    const { index } = fixtureIndex({
      "src/example.svelte": [
        '<script lang="ts">import { invoke } from "@tauri-apps/api/core";</script>',
        '<article class:selected data-evidence-highlighted="true">Example</article>',
        "<style>",
        "  article.selected, article[data-evidence-highlighted=\"true\"] {",
        "    background: color-mix(in srgb, var(--accent) 8%, transparent);",
        "  }",
        "</style>",
      ].join("\n"),
    });

    expect(index.getSvelte("src/example.svelte").imports).toEqual([
      {
        source: "@tauri-apps/api/core",
        bindings: [
          { imported: "invoke", local: "invoke", typeOnly: false },
        ],
      },
    ]);
    expect(index.getSvelte("src/example.svelte").styleRules).toEqual([
      {
        selectors: [
          [
            { type: "tag", name: "article" },
            { type: "class", name: "selected" },
          ],
          [
            { type: "tag", name: "article" },
            { type: "attribute", name: "data-evidence-highlighted", matcher: "=", value: "true" },
          ],
        ],
        declarations: [
          { property: "background", value: "color-mix(in srgb, var(--accent) 8%, transparent)" },
        ],
      },
    ]);
  });

  it("preserves Svelte import aliases and component ancestor branch polarity", () => {
    const { index } = fixtureIndex({
      "src/example.svelte": [
        '<script lang="ts">',
        '  import CanonicalAlias from "./Canonical.svelte";',
        '  import { Child as NamedAlias, type Options as LocalOptions } from "./named";',
        "</script>",
        "{#if groupSubject}",
        "  <CanonicalAlias />",
        "{:else}",
        "  <NamedAlias />",
        "{/if}",
      ].join("\n"),
    });

    expect(index.getSvelte("src/example.svelte").imports).toEqual([
      {
        source: "./Canonical.svelte",
        bindings: [
          { imported: "default", local: "CanonicalAlias", typeOnly: false },
        ],
      },
      {
        source: "./named",
        bindings: [
          { imported: "Child", local: "NamedAlias", typeOnly: false },
          { imported: "Options", local: "LocalOptions", typeOnly: true },
        ],
      },
    ]);
    expect(index.getSvelte("src/example.svelte").components).toEqual([
      {
        name: "CanonicalAlias",
        attributes: [],
        branches: [{
          condition: { kind: "identifier", name: "groupSubject" },
          conditionIdentifiers: ["groupSubject"],
          polarity: "consequent",
        }],
      },
      {
        name: "NamedAlias",
        attributes: [],
        branches: [{
          condition: { kind: "identifier", name: "groupSubject" },
          conditionIdentifiers: ["groupSubject"],
          polarity: "alternate",
        }],
      },
    ]);
  });

  it("retains complete Svelte branch predicates instead of identifier presence", () => {
    const { index } = fixtureIndex({
      "src/example.svelte": [
        "{#if activeTab === \"activity\" && !groupSubject}",
        "  <GroupActivity />",
        "{:else if activeTab === \"activity\" || sourceSubject !== null}",
        "  <SourceActivity />",
        "{/if}",
      ].join("\n"),
    });

    expect(index.getSvelte("src/example.svelte").components).toEqual([
      {
        name: "GroupActivity",
        attributes: [],
        branches: [{
          condition: {
            kind: "binary",
            operator: "&&",
            left: {
              kind: "binary",
              operator: "===",
              left: { kind: "identifier", name: "activeTab" },
              right: { kind: "string", value: "activity" },
            },
            right: {
              kind: "unary",
              operator: "!",
              operand: { kind: "identifier", name: "groupSubject" },
            },
          },
          conditionIdentifiers: ["activeTab", "groupSubject"],
          polarity: "consequent",
        }],
      },
      {
        name: "SourceActivity",
        attributes: [],
        branches: [
          {
            condition: {
              kind: "binary",
              operator: "&&",
              left: {
                kind: "binary",
                operator: "===",
                left: { kind: "identifier", name: "activeTab" },
                right: { kind: "string", value: "activity" },
              },
              right: {
                kind: "unary",
                operator: "!",
                operand: { kind: "identifier", name: "groupSubject" },
              },
            },
            conditionIdentifiers: ["activeTab", "groupSubject"],
            polarity: "alternate",
          },
          {
            condition: {
              kind: "binary",
              operator: "||",
              left: {
                kind: "binary",
                operator: "===",
                left: { kind: "identifier", name: "activeTab" },
                right: { kind: "string", value: "activity" },
              },
              right: {
                kind: "binary",
                operator: "!==",
                left: { kind: "identifier", name: "sourceSubject" },
                right: { kind: "null" },
              },
            },
            conditionIdentifiers: ["activeTab", "sourceSubject"],
            polarity: "consequent",
          },
        ],
      },
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

  it("returns cached declared text and JSON artifacts", () => {
    const fixture = fixtureIndex({
      "docs/authority.md": "declared authority\n",
      "src/generated.json": '{"schemaVersion":1,"items":["one"]}',
    });

    const firstText = fixture.index.getText("docs/authority.md");
    const firstJson = fixture.index.getJson("src/generated.json");

    expect(firstText).toBe("declared authority\n");
    expect(firstJson).toEqual({ schemaVersion: 1, items: ["one"] });
    expect(fixture.index.getText("docs/authority.md")).toBe(firstText);
    expect(fixture.index.getJson("src/generated.json")).toBe(firstJson);
    expect(fixture.readFile).toHaveBeenCalledTimes(2);
  });

  it("shares one immutable raw snapshot between declared text and JSON views", () => {
    const fixture = fixtureIndex({
      "src/generated.json": '{"schemaVersion":1,"items":["one"]}',
    });

    expect(fixture.index.getText("src/generated.json")).toBe('{"schemaVersion":1,"items":["one"]}');
    expect(fixture.index.getJson("src/generated.json")).toEqual({ schemaVersion: 1, items: ["one"] });
    expect(fixture.readFile).toHaveBeenCalledOnce();
  });

  it("fails closed and caches malformed declared JSON", () => {
    const fixture = fixtureIndex({ "src/generated.json": "{not-json" });

    expect(() => fixture.index.getJson("src/generated.json")).toThrow(/src\/generated\.json/);
    expect(() => fixture.index.getJson("src/generated.json")).toThrow(/src\/generated\.json/);
    expect(fixture.readFile).toHaveBeenCalledOnce();
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
