import path from "node:path";
import * as svelte from "svelte/compiler";
import ts from "typescript";
import { describe, expect, it } from "vitest";

import sourceContractLedger from "../../testing/source-contract-ledger.json";
import { createRepositoryIndex } from "./repository-index.mjs";
import { evaluateRule, registeredRuleIds } from "./repository-rules.mjs";

const root = path.resolve("repository-rule-fixture");
const TELEGRAM_PATH = "src/lib/telegram-contract-paths.ts";
const ANALYSIS_SURFACE_PATH = "src/lib/components/analysis/report-source-surface.svelte";
const SYMBOL_MAP_PATH = "src/lib/telegram-8b-symbol-map.json";
const GRAMMERS_BASELINE_PATH = "src/lib/telegram-grammers-feature-baseline.json";

type RuleFixture = {
  positive: Record<string, string>;
  mutations: Record<string, Record<string, string>>;
};

const telegramPathHelper = String.raw`
  import { existsSync, readFileSync, realpathSync } from "node:fs";
  import path from "node:path";

  const repositoryRoot = "root";

  function assertRepositoryRelative(relativePath: string) {
    if (
      !relativePath
      || path.isAbsolute(relativePath)
      || relativePath.includes("\\")
      || relativePath.split("/").some((segment) => segment === "." || segment === "..")
    ) throw new Error("invalid path");
    const selected = path.resolve(repositoryRoot, relativePath);
    const relative = path.relative(repositoryRoot, selected);
    if (relative === "" || relative === ".." || relative.startsWith(".." + path.sep) || path.isAbsolute(relative)) {
      throw new Error("path escaped");
    }
    return selected;
  }

  export function resolveTelegramContractPath(relativePath: string) {
    const selected = assertRepositoryRelative(relativePath);
    if (!existsSync(selected)) throw new Error("missing path");
    const realSelected = realpathSync(selected);
    const realRelative = path.relative(repositoryRoot, realSelected);
    if (realRelative === ".." || realRelative.startsWith(".." + path.sep) || path.isAbsolute(realRelative)) {
      throw new Error("symlink escaped");
    }
    return realSelected;
  }

  export function readTelegramContractFile(relativePath: string) {
    return normalizeTelegramContractSourceText(readFileSync(resolveTelegramContractPath(relativePath), "utf8"));
  }

  export function normalizeTelegramContractSourceText(source: string) {
    return source;
  }
`;

const ruleFixtures: Record<string, RuleFixture> = {
  "rule:telegram-repository-path-safety": {
    positive: { [TELEGRAM_PATH]: telegramPathHelper },
    mutations: {
      "moves the first rejection throw into an unused nested function": {
        [TELEGRAM_PATH]: telegramPathHelper.replace(
          ') throw new Error("invalid path");',
          ') { function unusedRejection() { throw new Error("invalid path"); } }',
        ),
      },
      "inverts the empty-path rejection": {
        [TELEGRAM_PATH]: telegramPathHelper.replace("      !relativePath\n", "      relativePath\n"),
      },
      "inverts the absolute-input rejection": {
        [TELEGRAM_PATH]: telegramPathHelper.replace(
          "      || path.isAbsolute(relativePath)\n",
          "      || !path.isAbsolute(relativePath)\n",
        ),
      },
      "inverts the Windows-separator rejection": {
        [TELEGRAM_PATH]: telegramPathHelper.replace(
          '      || relativePath.includes("\\\\")\n',
          '      || !relativePath.includes("\\\\")\n',
        ),
      },
      "negates the complete dot-segment predicate": {
        [TELEGRAM_PATH]: telegramPathHelper.replace(
          '      || relativePath.split("/").some((segment) => segment === "." || segment === "..")',
          '      || !relativePath.split("/").some((segment) => segment === "." || segment === "..")',
        ),
      },
      "inverts the dot-segment predicate": {
        [TELEGRAM_PATH]: telegramPathHelper.replace(
          'segment === "." || segment === ".."',
          'segment !== "." && segment !== ".."',
        ),
      },
      "inverts the resolved-root empty rejection": {
        [TELEGRAM_PATH]: telegramPathHelper.replace('relative === ""', 'relative !== ""'),
      },
      "inverts the resolved-root parent rejection": {
        [TELEGRAM_PATH]: telegramPathHelper.replace('relative === ".."', 'relative !== ".."'),
      },
      "inverts the resolved-root parent-prefix rejection": {
        [TELEGRAM_PATH]: telegramPathHelper.replace("relative.startsWith", "!relative.startsWith"),
      },
      "inverts the resolved-root absolute rejection": {
        [TELEGRAM_PATH]: telegramPathHelper.replace(
          "|| path.isAbsolute(relative))",
          "|| !path.isAbsolute(relative))",
        ),
      },
      "inverts the missing-file predicate": {
        [TELEGRAM_PATH]: telegramPathHelper.replace("if (!existsSync(selected))", "if (existsSync(selected))"),
      },
      "inverts the realpath parent rejection": {
        [TELEGRAM_PATH]: telegramPathHelper.replace('realRelative === ".."', 'realRelative !== ".."'),
      },
      "inverts the realpath parent-prefix rejection": {
        [TELEGRAM_PATH]: telegramPathHelper.replace("realRelative.startsWith", "!realRelative.startsWith"),
      },
      "inverts the realpath absolute rejection": {
        [TELEGRAM_PATH]: telegramPathHelper.replace(
          "|| path.isAbsolute(realRelative))",
          "|| !path.isAbsolute(realRelative))",
        ),
      },
    },
  },
  "rule:analysis-source-reader-surface-composition": {
    positive: {
      [ANALYSIS_SURFACE_PATH]: `
        <script lang="ts">
          import SourceBrowserShell from "./source-browser-shell.svelte";
        </script>
        <SourceBrowserShell />
      `,
    },
    mutations: {
      "restores a transitional source reader": {
        [ANALYSIS_SURFACE_PATH]: `
          <script lang="ts">
            import TelegramTimelineReader from "./telegram-timeline-reader.svelte";
          </script>
          <TelegramTimelineReader />
        `,
      },
    },
  },
};

function indexFor(sources: Record<string, string>) {
  return createRepositoryIndex({
    root,
    readFile(absolutePath: string) {
      const relativePath = path.relative(root, absolutePath).replaceAll("\\", "/");
      const source = sources[relativePath];
      if (source === undefined) throw new Error(`missing fixture: ${relativePath}`);
      return source;
    },
    ts,
    svelte,
    loadCargoMetadata: () => ({ packages: [] }),
  });
}

function clone<T>(value: T): T {
  return JSON.parse(JSON.stringify(value));
}

const grammersBaseline = {
  schemaVersion: 1,
  revision: "1f901ce6e973fdcf0e74267f3d8efad5c729daaa",
  packages: [
    { name: "grammers-client", required: [], forbidden: ["default"], universe: ["default"] },
    { name: "grammers-mtsender", required: [], forbidden: ["proxy"], universe: ["proxy"] },
    { name: "grammers-session", required: ["serde"], forbidden: ["default"], universe: ["default", "serde"] },
    { name: "grammers-tl-types", required: ["default", "deserializable-functions"], forbidden: ["impl-serde"], universe: ["default", "deserializable-functions", "impl-serde"] },
  ],
};

function cargoMetadata() {
  const revision = grammersBaseline.revision;
  const grammers = grammersBaseline.packages.map((entry) => ({
    id: `${entry.name} 0.1.0 (git+https://codeberg.org/Lonami/grammers?rev=${revision}#${revision})`,
    name: entry.name,
    source: `git+https://codeberg.org/Lonami/grammers?rev=${revision}#${revision}`,
    features: Object.fromEntries(entry.universe.map((feature) => [feature, []])),
    targets: [{ kind: ["lib"], name: entry.name.replaceAll("-", "_") }],
    dependencies: [],
  }));
  const app = {
    id: "path+file:///repo/src-tauri#extractum@0.2.0",
    name: "extractum",
    source: null,
    manifest_path: "C:/repo/src-tauri/Cargo.toml",
    features: {},
    targets: [{ kind: ["lib"], name: "extractum_lib" }],
    dependencies: [
      { name: "extractum-telegram", kind: null, source: null, path: "C:/repo/src-tauri/crates/extractum-telegram", target: null, rename: null, features: [] },
      { name: "extractum-telegram", kind: "dev", source: null, path: "C:/repo/src-tauri/crates/extractum-telegram", target: null, rename: null, features: ["app-test-support"] },
    ],
  };
  const producer = {
    id: "path+file:///repo/src-tauri/crates/extractum-telegram#0.2.0",
    name: "extractum-telegram",
    source: null,
    manifest_path: "C:/repo/src-tauri/crates/extractum-telegram/Cargo.toml",
    features: { "app-test-support": [] },
    targets: [{ kind: ["lib"], name: "extractum_telegram" }],
    dependencies: grammers.map(({ name, source }) => ({ name, kind: null, source: source.replace(`#${revision}`, ""), path: null, target: null, rename: null, features: [] })),
  };
  return {
    packages: [app, producer, ...grammers],
    workspace_members: [app.id, producer.id],
    resolve: {
      nodes: [
        { id: app.id, features: [], deps: [{ name: "extractum_telegram", pkg: producer.id, dep_kinds: [{ kind: null, target: null }, { kind: "dev", target: null }] }] },
        { id: producer.id, features: ["app-test-support"], deps: grammers.map(({ id, name }) => ({ name: name.replaceAll("-", "_"), pkg: id, dep_kinds: [{ kind: null, target: null }] })) },
        ...grammers.map((entry) => ({ id: entry.id, features: [...grammersBaseline.packages.find(({ name }) => name === entry.name)!.required], deps: [] })),
      ],
    },
  };
}

function cargoIndex(metadata = cargoMetadata()) {
  return createRepositoryIndex({
    root,
    readFile(absolutePath: string) {
      const relativePath = path.relative(root, absolutePath).replaceAll("\\", "/");
      if (relativePath === GRAMMERS_BASELINE_PATH) return JSON.stringify(grammersBaseline);
      throw new Error(`missing fixture: ${relativePath}`);
    },
    ts,
    svelte,
    loadCargoMetadata: () => metadata,
  });
}

function realAuthorityIndex() {
  return createRepositoryIndex({ root: process.cwd() });
}

const telegramStructuredFixtures = {
  "rule:telegram-phase-8b-authority-integrity": {
    positive: () => realAuthorityIndex(),
    mutations: {
      "changes the generated symbol authority": () => {
        const index = realAuthorityIndex();
        return {
          ...index,
          getJson(inputPath: string) {
            const value = index.getJson(inputPath);
            return inputPath === SYMBOL_MAP_PATH ? { ...value, schemaVersion: 2 } : value;
          },
        };
      },
      "changes the generated test identity authority": () => {
        const index = realAuthorityIndex();
        return {
          ...index,
          getJson(inputPath: string) {
            const value = index.getJson(inputPath);
            return inputPath === "src/lib/telegram-8b-test-identities.json"
              ? { ...value, schemaVersion: 2 }
              : value;
          },
        };
      },
      "changes the frozen staging content address": () => {
        const index = realAuthorityIndex();
        return {
          ...index,
          getText(inputPath: string) {
            const value = index.getText(inputPath);
            return inputPath === "src/lib/telegram-8b-staging-sha256.json" ? `${value} ` : value;
          },
        };
      },
    },
  },
  "rule:telegram-crate-manifest-boundary": {
    positive: () => cargoIndex(),
    mutations: {
      "removes the producer library target": () => {
        const metadata = clone(cargoMetadata());
        metadata.packages.find(({ name }: any) => name === "extractum-telegram")!.targets = [];
        return cargoIndex(metadata);
      },
      "enables app test support on the production edge": () => {
        const metadata = clone(cargoMetadata());
        metadata.packages.find(({ name }: any) => name === "extractum")!.dependencies[0].features = ["app-test-support"];
        return cargoIndex(metadata);
      },
      "removes the dev-only feature edge": () => {
        const metadata = clone(cargoMetadata());
        metadata.packages.find(({ name }: any) => name === "extractum")!.dependencies.pop();
        return cargoIndex(metadata);
      },
    },
  },
  "rule:telegram-crate-dependency-ownership": {
    positive: () => cargoIndex(),
    mutations: {
      "adds a direct app Grammers dependency": () => {
        const metadata = clone(cargoMetadata());
        metadata.packages.find(({ name }: any) => name === "extractum")!.dependencies.push({ name: "grammers-client", kind: null });
        return cargoIndex(metadata);
      },
      "drifts the Grammers source revision": () => {
        const metadata = clone(cargoMetadata());
        metadata.packages.find(({ name }: any) => name === "grammers-client")!.source = "git+https://codeberg.org/Lonami/grammers?rev=wrong#wrong";
        return cargoIndex(metadata);
      },
      "enables a forbidden Grammers feature": () => {
        const metadata = clone(cargoMetadata());
        const selected = metadata.packages.find(({ name }: any) => name === "grammers-tl-types")!;
        metadata.resolve.nodes.find(({ id }: any) => id === selected.id)!.features.push("impl-serde");
        return cargoIndex(metadata);
      },
    },
  },
} as const;

function inSlice3ARanges(id: string) {
  const number = Number(id.slice("SC-".length));
  return (number >= 29 && number <= 59)
    || (number >= 221 && number <= 278)
    || (number >= 561 && number <= 658);
}

function telegramMutation(name: string) {
  const mutation = ruleFixtures["rule:telegram-repository-path-safety"].mutations[name];
  if (!mutation) throw new Error(`missing Telegram mutation fixture: ${name}`);
  return evaluateRule({
    id: "rule:telegram-repository-path-safety",
    index: indexFor(mutation),
  }).violations;
}

describe("repository rule registry", () => {
  const allowedRuleIds = new Set(
    sourceContractLedger.rows
      .filter((row) => inSlice3ARanges(row.id))
      .flatMap((row) => "replacementIds" in row ? row.replacementIds ?? [] : [])
      .filter((id): id is string => id.startsWith("rule:")),
  );

  it("derives the frozen 22-ID allowlist and registers the implemented Task 2B evaluators", () => {
    expect(allowedRuleIds.size).toBe(22);
    expect(registeredRuleIds).toEqual([
      "rule:analysis-source-reader-surface-composition",
      "rule:telegram-crate-dependency-ownership",
      "rule:telegram-crate-manifest-boundary",
      "rule:telegram-phase-8b-authority-integrity",
      "rule:telegram-repository-path-safety",
    ]);
    for (const id of registeredRuleIds) expect(allowedRuleIds.has(id), id).toBe(true);
  });

  it("gives every registered evaluator its own positive fixture and violating mutation", () => {
    expect([...Object.keys(ruleFixtures), ...Object.keys(telegramStructuredFixtures)].sort()).toEqual(registeredRuleIds);

    for (const id of registeredRuleIds) {
      const fixture = ruleFixtures[id];
      const structured = telegramStructuredFixtures[id as keyof typeof telegramStructuredFixtures];
      const positiveIndex = structured ? structured.positive() : indexFor(fixture.positive);
      expect(evaluateRule({ id, index: positiveIndex }), `${id} positive`).toEqual({
        id,
        violations: [],
      });
      const mutations = structured?.mutations ?? fixture.mutations;
      expect(Object.keys(mutations), `${id} mutations`).not.toEqual([]);
      for (const [name, mutation] of Object.entries(mutations)) {
        const mutationIndex = structured ? mutation() : indexFor(mutation);
        expect(evaluateRule({ id, index: mutationIndex }).violations, `${id}: ${name}`).not.toEqual([]);
      }
    }
  });

  it("rejects a negated complete dot-segment predicate", () => {
    expect(telegramMutation("negates the complete dot-segment predicate")).not.toEqual([]);
  });

  it("rejects a guard whose only throw is inside an unused nested function", () => {
    expect(telegramMutation("moves the first rejection throw into an unused nested function")).not.toEqual([]);
  });

  it("rejects an inverted Windows-separator guard", () => {
    expect(telegramMutation("inverts the Windows-separator rejection")).not.toEqual([]);
  });

  it("rejects an inverted realpath parent escape guard", () => {
    expect(telegramMutation("inverts the realpath parent rejection")).not.toEqual([]);
  });

  it("converts declared-input parse failures to INFRA_ERROR violations", () => {
    const result = evaluateRule({
      id: "rule:analysis-source-reader-surface-composition",
      index: indexFor({ [ANALYSIS_SURFACE_PATH]: "<script>const value = ;</script>" }),
    });

    expect(result).toEqual({
      id: "rule:analysis-source-reader-surface-composition",
      violations: [expect.stringMatching(/^INFRA_ERROR:.*report-source-surface\.svelte/)],
    });
  });

  it("throws for an unknown rule ID", () => {
    expect(() => evaluateRule({ id: "rule:not-registered", index: indexFor({}) })).toThrow(/rule:not-registered/);
  });
});
