import path from "node:path";
import * as svelte from "svelte/compiler";
import ts from "typescript";
import { describe, expect, it } from "vitest";

import { createRepositoryIndex } from "./repository-index.mjs";
import { evaluateRule, registeredRuleIds } from "./repository-rules.mjs";
import { generateRustDependencyPolicy } from "../rust-dependency-policy.mjs";
import rustDependencyPolicy from "./rust-dependency-policy.json";

const root = path.resolve("repository-rule-fixture");
const GRAMMERS_BASELINE_PATH = "src/lib/telegram-grammers-feature-baseline.json";
const RUST_DUPLICATE_BASELINE_PATH = "scripts/testing/rust-duplicate-baseline.json";
const RUST_DEPENDENCY_POLICY_PATH = "scripts/testing/rust-dependency-policy.json";
const RUST_SUPPLY_CHAIN_EXCEPTIONS_PATH = "scripts/testing/rust-supply-chain-exceptions.json";
const rustSupplyChainExceptions = { schemaVersion: 1, licenseExceptions: [], advisoryExceptions: [], duplicateGrowthExceptions: [] };
const canonicalToolchain = [
  "[toolchain]",
  'channel = "1.95.0"',
  'components = ["rustfmt", "clippy"]',
  'targets = ["x86_64-pc-windows-msvc"]',
  'profile = "minimal"',
  "",
].join("\n");
const rustDuplicateBaseline = {
  schemaVersion: 1,
  target: "x86_64-pc-windows-msvc",
  duplicateNameCount: 1,
  duplicateVersionInstanceCount: 2,
  duplicateCardinality: { alpha: 2 },
};
const rustDuplicateTree = [
  "extractum v0.2.0 (C:\\repo\\src-tauri)",
  "alpha v1.0.0",
  "alpha v2.0.0 (*)",
  "serde v1.0.229",
].join("\n");
const DATA_GRID_PATH = "src/lib/components/extractum-ui/DataGrid.svelte";
const TREE_DATA_GRID_PATH = "src/lib/components/extractum-ui/TreeDataGrid.svelte";
const GRID_SELECT_CELL_PATH = "src/lib/components/extractum-ui/GridSelectCell.svelte";

const extractumGridBoundarySources = {
  [DATA_GRID_PATH]: `
    <script lang="ts">
      import { Grid, Willow } from "@svar-ui/svelte-grid";
      import { Locale } from "@svar-ui/svelte-core";
    </script>
    <Locale><Willow fonts={false}><Grid /></Willow></Locale>
  `,
  [TREE_DATA_GRID_PATH]: `
    <script lang="ts">
      import { Grid, Willow } from "@svar-ui/svelte-grid";
      import { Locale } from "@svar-ui/svelte-core";
    </script>
    <Locale><Willow fonts={false}><Grid tree /></Willow></Locale>
    <style>.extractum-tree-data-grid :global(.wx-cell) { padding: 4px; }</style>
  `,
  [GRID_SELECT_CELL_PATH]: `<input data-action="ignore-click" />`,
  "src/lib/components/research-projects/FeatureGrid.svelte": `<section>Feature</section>`,
};

type RuleFixture = {
  positive: Record<string, string>;
  mutations: Record<string, Record<string, string>>;
};

const ruleFixtures: Record<string, RuleFixture> = {
  "rule:extractum-grid-wrapper-boundary": {
    positive: extractumGridBoundarySources,
    mutations: {
      "imports SVAR from a feature component": {
        ...extractumGridBoundarySources,
        "src/lib/components/research-projects/FeatureGrid.svelte": `
          <script lang="ts">import { Grid } from "@svar-ui/svelte-grid";</script>
          <Grid />
        `,
      },
      "imports escaped SVAR from a feature component": {
        ...extractumGridBoundarySources,
        "src/lib/components/research-projects/FeatureGrid.svelte": String.raw`
          <script lang="ts">import { Grid } from "\x40svar-ui/svelte-grid";</script>
          <Grid />
        `,
      },
      "drops the tree wrapper scoped SVAR style": {
        ...extractumGridBoundarySources,
        [TREE_DATA_GRID_PATH]: extractumGridBoundarySources[TREE_DATA_GRID_PATH].replace(
          '<style>.extractum-tree-data-grid :global(.wx-cell) { padding: 4px; }</style>',
          "",
        ),
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
    listFiles: () => Object.keys(sources),
  });
}

function clone<T>(value: T): T {
  return JSON.parse(JSON.stringify(value));
}

const grammersResolvedVersions = {
  "grammers-client": "0.10.0",
  "grammers-crypto": "0.10.0",
  "grammers-mtproto": "0.10.0",
  "grammers-mtsender": "0.10.0",
  "grammers-session": "0.10.0",
  "grammers-tl-gen": "0.10.0",
  "grammers-tl-parser": "1.2.2",
  "grammers-tl-types": "0.10.0",
} as const;

const grammersSource = "registry+https://github.com/rust-lang/crates.io-index";
const grammersPackageId = (name: string, version: string) =>
  `${grammersSource}#${name}@${version}`;

const grammersBaseline = {
  schemaVersion: 2,
  directPackages: [
    { name: "grammers-client", required: [], forbidden: ["default"], universe: ["default"] },
    { name: "grammers-mtsender", required: [], forbidden: ["proxy"], universe: ["proxy"] },
    { name: "grammers-session", required: ["serde"], forbidden: ["default"], universe: ["default", "serde"] },
    { name: "grammers-tl-types", required: ["default", "deserializable-functions"], forbidden: ["impl-serde"], universe: ["default", "deserializable-functions", "impl-serde"] },
  ],
  resolvedPackages: Object.entries(grammersResolvedVersions).map(([name, version]) => ({
    name,
    version,
    source: grammersSource,
  })),
};

function cargoMetadata() {
  const grammers = Object.entries(grammersResolvedVersions).map(([name, version]) => {
    const directPolicy = grammersBaseline.directPackages.find((entry) => entry.name === name);
    return {
      id: grammersPackageId(name, version),
      name,
      version,
      source: grammersSource,
      features: Object.fromEntries((directPolicy?.universe ?? []).map((feature) => [feature, []])),
      targets: [{ kind: ["lib"], name: name.replaceAll("-", "_") }],
      dependencies: [],
    };
  });
  const app = {
    id: "path+file:///repo/src-tauri#extractum@0.2.0",
    name: "extractum",
    source: null,
    manifest_path: "C:/repo/src-tauri/Cargo.toml",
    features: {},
    targets: [{ kind: ["lib"], name: "extractum_lib" }],
    dependencies: [
      { name: "extractum-telegram", kind: null, source: null, path: "C:/repo/src-tauri/crates/extractum-telegram", target: null, rename: null, features: [], uses_default_features: true },
      { name: "extractum-telegram", kind: "dev", source: null, path: "C:/repo/src-tauri/crates/extractum-telegram", target: null, rename: null, features: ["app-test-support"], uses_default_features: true },
    ],
  };
  const normalProducerDependencies = [
    ["base64", [], true],
    ["chacha20poly1305", ["std"], true],
    ["extractum-core", [], true],
    ["grammers-client", [], false],
    ["grammers-mtsender", [], true],
    ["grammers-session", ["serde"], false],
    ["grammers-tl-types", ["deserializable-functions"], true],
    ["rand_core", ["getrandom"], true],
    ["secrecy", [], true],
    ["serde", ["derive"], true],
    ["serde_json", [], true],
    ["tokio", ["rt", "sync", "time"], true],
  ].map(([name, features, usesDefaultFeatures]) => ({
    name,
    kind: null,
    req: String(name).startsWith("grammers-") ? "=0.10.0" : "*",
    source: String(name).startsWith("grammers-") ? grammersSource : null,
    path: name === "extractum-core" ? "C:/repo/src-tauri/crates/extractum-core" : null,
    target: null,
    rename: null,
    features,
    uses_default_features: usesDefaultFeatures,
  }));
  const producer = {
    id: "path+file:///repo/src-tauri/crates/extractum-telegram#0.2.0",
    name: "extractum-telegram",
    source: null,
    manifest_path: "C:/repo/src-tauri/crates/extractum-telegram/Cargo.toml",
    features: { "app-test-support": [] },
    targets: [{ kind: ["lib"], name: "extractum_telegram" }],
    dependencies: [
      ...normalProducerDependencies,
      { name: "tokio", kind: "dev", source: null, path: null, target: null, rename: null, features: ["macros", "test-util"], uses_default_features: true },
    ],
  };
  const producerNodeDependencies = normalProducerDependencies.map(({ name }) => {
    const grammersPackage = grammers.find((candidate) => candidate.name === name);
    return {
      name: String(name).replaceAll("-", "_"),
      pkg: grammersPackage?.id ?? `registry+fixture#${name}@1.0.0`,
      dep_kinds: name === "tokio"
        ? [{ kind: null, target: null }, { kind: "dev", target: null }]
        : [{ kind: null, target: null }],
    };
  });
  return {
    packages: [app, producer, ...grammers],
    workspace_members: [app.id, producer.id],
    resolve: {
      nodes: [
        { id: app.id, features: [], deps: [{ name: "extractum_telegram", pkg: producer.id, dep_kinds: [{ kind: null, target: null }, { kind: "dev", target: null }] }] },
        { id: producer.id, features: ["app-test-support"], deps: producerNodeDependencies },
        ...grammers.map((entry) => ({
          id: entry.id,
          features: [
            ...(grammersBaseline.directPackages.find(({ name }) => name === entry.name)?.required ?? []),
          ],
          deps: [],
        })),
      ],
    },
  };
}

function cargoIndex(
  metadata = cargoMetadata(),
  cargoTree = rustDuplicateTree,
  duplicateBaseline = rustDuplicateBaseline,
  policy = rustDependencyPolicy,
  toolchain = canonicalToolchain,
  packageJson = { private: true, dependencies: {}, devDependencies: {} },
) {
  return createRepositoryIndex({
    root,
    readFile(absolutePath: string) {
      const relativePath = path.relative(root, absolutePath).replaceAll("\\", "/");
      if (relativePath === GRAMMERS_BASELINE_PATH) return JSON.stringify(grammersBaseline);
      if (relativePath === RUST_DUPLICATE_BASELINE_PATH) return JSON.stringify(duplicateBaseline);
      if (relativePath === RUST_SUPPLY_CHAIN_EXCEPTIONS_PATH) return JSON.stringify(rustSupplyChainExceptions);
      if (relativePath === RUST_DEPENDENCY_POLICY_PATH) return JSON.stringify(policy);
      if (relativePath === "rust-toolchain.toml") return toolchain;
      if (relativePath === "package.json") return JSON.stringify(packageJson);
      if (relativePath === "src-tauri/Cargo.toml") return ["edition = \"2021\"", "rust-version = \"1.95\"", "edition.workspace = true", "rust-version.workspace = true"].join("\n");
      if (relativePath.startsWith("src-tauri/crates/") && relativePath.endsWith("/Cargo.toml")) return ["edition.workspace = true", "rust-version.workspace = true"].join("\n");
      throw new Error(`missing fixture: ${relativePath}`);
    },
    ts,
    svelte,
    loadCargoMetadata: () => metadata,
    loadCargoTree: () => cargoTree,
  });
}

function rustPolicyCargoMetadata() {
  const metadata = clone(cargoMetadata());
  const app = metadata.packages.find(({ name }: any) => name === "extractum")!;
  const producer = metadata.packages.find(({ name }: any) => name === "extractum-telegram")!;
  const packages = [app, producer];
  for (const name of rustDependencyPolicy.toolchain.workspacePackages) {
    let selected = packages.find((candidate) => candidate.name === name);
    if (!selected) {
      selected = {
        id: `path+file:///repo/src-tauri/crates/${name}#0.2.0`,
        name,
        source: null,
        manifest_path: `C:/repo/src-tauri/crates/${name}/Cargo.toml`,
        features: {},
        targets: [{ kind: ["lib"], name: name.replaceAll("-", "_") }],
        dependencies: [],
      };
      metadata.packages.push(selected);
      metadata.resolve.nodes.push({ id: selected.id, features: [], deps: [] });
      packages.push(selected);
    }
    selected.rust_version = "1.95";
    selected.edition = "2021";
    selected.publish = [];
  }
  metadata.workspace_members = packages.map(({ id }) => id);
  app.dependencies.push(
    { name: "apalis", kind: null, req: "=1.0.0-rc.8" },
    { name: "apalis-sqlite", kind: null, req: "=1.0.0-rc.8" },
    { name: "tauri", kind: null, req: "^2" },
    { name: "tauri-build", kind: "build", req: "^2" },
    { name: "tauri-plugin-dialog", kind: null, req: "^2" },
    { name: "tauri-plugin-opener", kind: null, req: "^2" },
    { name: "tauri-plugin-sql", kind: null, req: "^2" },
    { name: "tauri-plugin-mcp-bridge", kind: null, req: "^0.11" },
  );
  return metadata;
}

function rustPolicyIndex(metadata = rustPolicyCargoMetadata(), toolchain = canonicalToolchain, packageJson = {
  private: true,
  dependencies: {
    "@tauri-apps/api": "^2",
    "@tauri-apps/plugin-dialog": "^2",
    "@tauri-apps/plugin-opener": "^2",
    "@tauri-apps/plugin-sql": "^2.4.0",
  },
  devDependencies: { "@tauri-apps/cli": "^2" },
}) {
  return cargoIndex(metadata, rustDuplicateTree, rustDuplicateBaseline, generateRustDependencyPolicy({ metadata: rustPolicyCargoMetadata(), packageJson, reviewed: rustDependencyPolicy }), toolchain, packageJson);
}

function realAuthorityIndex() {
  return createRepositoryIndex({ root: process.cwd() });
}

const telegramStructuredFixtures = {
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
        const app = metadata.packages.find(({ name }: any) => name === "extractum")!;
        app.dependencies = app.dependencies.filter(({ name, kind }: any) =>
          name !== "extractum-telegram" || kind !== "dev");
        return cargoIndex(metadata);
      },
      "adds an undeclared producer feature": () => {
        const metadata = clone(cargoMetadata());
        metadata.packages.find(({ name }: any) => name === "extractum-telegram")!.features.debug = [];
        return cargoIndex(metadata);
      },
      "adds an undeclared producer dependency": () => {
        const metadata = clone(cargoMetadata());
        metadata.packages.find(({ name }: any) => name === "extractum-telegram")!.dependencies.push({
          name: "anyhow", kind: null, source: null, path: null, target: null, rename: null, features: [],
        });
        return cargoIndex(metadata);
      },
      "widens the producer Tokio dev features": () => {
        const metadata = clone(cargoMetadata());
        const producer = metadata.packages.find(({ name }: any) => name === "extractum-telegram")!;
        producer.dependencies.find(({ name, kind }: any) => name === "tokio" && kind === "dev")!.features.push("rt-multi-thread");
        return cargoIndex(metadata);
      },
      "adds a build edge to the producer": () => {
        const metadata = clone(cargoMetadata());
        metadata.packages.find(({ name }: any) => name === "extractum")!.dependencies.push({
          name: "extractum-telegram", kind: "build", source: null, path: "C:/repo/src-tauri/crates/extractum-telegram", target: null, rename: null, features: [],
        });
        return cargoIndex(metadata);
      },
      "drops the dev dependency from the resolved app edge": () => {
        const metadata = clone(cargoMetadata());
        const app = metadata.packages.find(({ name }: any) => name === "extractum")!;
        const producer = metadata.packages.find(({ name }: any) => name === "extractum-telegram")!;
        metadata.resolve.nodes.find(({ id }: any) => id === app.id)!
          .deps.find(({ pkg }: any) => pkg === producer.id)!.dep_kinds = [{ kind: null, target: null }];
        return cargoIndex(metadata);
      },
      "adds a second workspace feature mention": () => {
        const metadata = clone(cargoMetadata());
        const observer = {
          id: "path+file:///repo/src-tauri/crates/observer#0.1.0",
          name: "observer",
          source: null,
          manifest_path: "C:/repo/src-tauri/crates/observer/Cargo.toml",
          features: {},
          targets: [{ kind: ["lib"], name: "observer" }],
          dependencies: [{ name: "extractum-telegram", kind: "dev", source: null, path: "C:/repo/src-tauri/crates/extractum-telegram", target: null, rename: null, features: ["app-test-support"] }],
        };
        metadata.packages.push(observer);
        metadata.workspace_members.push(observer.id);
        metadata.resolve.nodes.push({ id: observer.id, features: [], deps: [] });
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
      "drifts a transitive Grammers source": () => {
        const metadata = clone(cargoMetadata());
        metadata.packages.find(({ name }: any) => name === "grammers-crypto")!.source =
          "git+https://codeberg.org/Lonami/grammers?rev=wrong#wrong";
        return cargoIndex(metadata);
      },
      "drifts a transitive Grammers version": () => {
        const metadata = clone(cargoMetadata());
        metadata.packages.find(({ name }: any) => name === "grammers-mtproto")!.version = "0.10.1";
        return cargoIndex(metadata);
      },
      "widens a direct Grammers manifest requirement": () => {
        const metadata = clone(cargoMetadata());
        const producer = metadata.packages.find(({ name }: any) => name === "extractum-telegram")!;
        producer.dependencies.find(({ name }: any) => name === "grammers-client")!.req = "^0.10.0";
        return cargoIndex(metadata);
      },
      "enables a forbidden Grammers feature": () => {
        const metadata = clone(cargoMetadata());
        const selected = metadata.packages.find(({ name }: any) => name === "grammers-tl-types")!;
        metadata.resolve.nodes.find(({ id }: any) => id === selected.id)!.features.push("impl-serde");
        return cargoIndex(metadata);
      },
      "reorders the generated baseline direct packages": () => {
        const index = cargoIndex();
        return {
          ...index,
          getJson(inputPath: string) {
            const value = index.getJson(inputPath);
            return inputPath === GRAMMERS_BASELINE_PATH
              ? { ...value, directPackages: [...value.directPackages].reverse() }
              : value;
          },
        };
      },
    },
  },
} as const;

const rustDuplicateStructuredFixtures = {
  "rule:rust-duplicate-baseline": {
    positive: () => cargoIndex(),
    mutations: {
      "adds a new duplicate package name": () => cargoIndex(cargoMetadata(), [
        rustDuplicateTree,
        "beta v1.0.0",
        "beta v2.0.0 (*)",
      ].join("\n")),
      "adds a third version of an existing duplicate": () => cargoIndex(cargoMetadata(), [
        rustDuplicateTree,
        "alpha v3.0.0",
      ].join("\n")),
    },
    compatibleReplacement: () => cargoIndex(cargoMetadata(), [
      "extractum v0.2.0 (C:\\repo\\src-tauri)",
      "alpha v1.0.0",
      "alpha v3.0.0 (*)",
      "serde v1.0.229",
    ].join("\n")),
    packageSetReplacement: () => cargoIndex(cargoMetadata(), [
      "extractum v0.2.0 (C:\\repo\\src-tauri)",
      "beta v1.0.0",
      "beta v2.0.0 (*)",
      "serde v1.0.229",
    ].join("\n")),
    multiplePackageSetReplacement: () => cargoIndex(cargoMetadata(), [
      "extractum v0.2.0 (C:\\repo\\src-tauri)",
      "gamma v1.0.0",
      "gamma v2.0.0 (*)",
      "beta v1.0.0",
      "beta v2.0.0 (*)",
    ].join("\n"), {
      schemaVersion: 1,
      target: "x86_64-pc-windows-msvc",
      duplicateNameCount: 2,
      duplicateVersionInstanceCount: 4,
      duplicateCardinality: { alpha: 2, zeta: 2 },
    }),
  },
} as const;

const rustPolicyStructuredFixtures = {
  "rule:rust-toolchain-policy": {
    positive: () => rustPolicyIndex(),
    mutations: {
      "drops workspace MSRV inheritance from one package": () => {
        const metadata = rustPolicyCargoMetadata();
        metadata.packages.find(({ name }: any) => name === "extractum-analysis")!.rust_version = null;
        return rustPolicyIndex(metadata);
      },
      "changes the canonical toolchain channel": () => rustPolicyIndex(
        rustPolicyCargoMetadata(),
        canonicalToolchain.replace('channel = "1.95.0"', 'channel = "1.96.0"'),
      ),
      "makes the root package publishable": () => {
        const metadata = rustPolicyCargoMetadata();
        metadata.packages.find(({ name }: any) => name === "extractum")!.publish = ["crates-io"];
        return rustPolicyIndex(metadata);
      },
    },
  },
  "rule:rust-dependency-policy": {
    positive: () => rustPolicyIndex(),
    mutations: {
      "moves tauri-build outside the CLI major": () => {
        const metadata = rustPolicyCargoMetadata();
        metadata.packages.find(({ name }: any) => name === "extractum")!.dependencies
          .find(({ name, kind }: any) => name === "tauri-build" && kind === "build")!.req = "3";
        return rustPolicyIndex(metadata);
      },
      "moves the MCP bridge outside minor 0.11": () => {
        const metadata = rustPolicyCargoMetadata();
        metadata.packages.find(({ name }: any) => name === "extractum")!.dependencies
          .find(({ name }: any) => name === "tauri-plugin-mcp-bridge")!.req = "^0.12";
        return rustPolicyIndex(metadata);
      },
      "widens a Grammers exact pin": () => {
        const metadata = rustPolicyCargoMetadata();
        metadata.packages.find(({ name }: any) => name === "extractum-telegram")!.dependencies
          .find(({ name }: any) => name === "grammers-client")!.req = "^0.10.0";
        return rustPolicyIndex(metadata);
      },
      "introduces an unapproved prerelease": () => {
        const metadata = rustPolicyCargoMetadata();
        metadata.packages.find(({ name }: any) => name === "extractum")!.dependencies.push({
          name: "unapproved-release", kind: null, req: "=1.2.3-beta.1",
        });
        return rustPolicyIndex(metadata);
      },
    },
  },
} as const;

function expectSemanticViolations(violations: string[], label: string) {
  expect(violations, label).not.toEqual([]);
  expect(violations.some((violation) => violation.startsWith("INFRA_ERROR:")), label).toBe(false);
}

describe("repository rule registry", () => {
  it("caches Cargo tree snapshots and errors for duplicate evaluation", () => {
    let calls = 0;
    const index = createRepositoryIndex({
      root,
      loadCargoTree: () => {
        calls += 1;
        return rustDuplicateTree;
      },
    });

    expect(index.getCargoTree()).toBe(rustDuplicateTree);
    expect(index.getCargoTree()).toBe(rustDuplicateTree);
    expect(calls).toBe(1);

    const failing = createRepositoryIndex({
      root,
      loadCargoTree: () => {
        throw new Error("cargo tree failed");
      },
    });
    expect(() => failing.getCargoTree()).toThrow(/src-tauri\/Cargo\.toml: cargo tree failed/);
    expect(() => failing.getCargoTree()).toThrow(/src-tauri\/Cargo\.toml: cargo tree failed/);
  });

  it(
    "accepts the current repository snapshot for every registered rule",
    () => {
      const index = realAuthorityIndex();
      for (const id of registeredRuleIds) {
        expect(evaluateRule({ id, index }), id).toEqual({ id, violations: [] });
      }
    },
    15_000,
  );

  it("skips production files without the @svar-ui/ marker", () => {
    const index = indexFor({
      ...extractumGridBoundarySources,
      // Invalid syntax is a parse detector, not a supported repository state.
      "src/lib/unrelated-without-marker.ts": "export const = ;",
    });

    expect(evaluateRule({
      id: "rule:extractum-grid-wrapper-boundary",
      index,
    })).toEqual({
      id: "rule:extractum-grid-wrapper-boundary",
      violations: [],
    });
  });

  it("registers every current repository rule", () => {
    expect(registeredRuleIds).toEqual([
      "rule:extractum-grid-wrapper-boundary",
      "rule:rust-dependency-policy",
      "rule:rust-duplicate-baseline",
      "rule:rust-toolchain-policy",
      "rule:telegram-crate-dependency-ownership",
      "rule:telegram-crate-manifest-boundary",
    ]);
  });

  it("gives every registered evaluator its own positive fixture and violating mutation", () => {
    expect([
      ...Object.keys(ruleFixtures),
      ...Object.keys(telegramStructuredFixtures),
      ...Object.keys(rustDuplicateStructuredFixtures),
      ...Object.keys(rustPolicyStructuredFixtures),
    ].sort()).toEqual(registeredRuleIds);

    for (const id of registeredRuleIds) {
      const fixture = ruleFixtures[id];
      const structured = telegramStructuredFixtures[id as keyof typeof telegramStructuredFixtures]
        ?? rustDuplicateStructuredFixtures[id as keyof typeof rustDuplicateStructuredFixtures]
        ?? rustPolicyStructuredFixtures[id as keyof typeof rustPolicyStructuredFixtures];
      const positiveIndex = structured ? structured.positive() : indexFor(fixture.positive);
      expect(evaluateRule({ id, index: positiveIndex }), `${id} positive`).toEqual({
        id,
        violations: [],
      });
      const mutations = structured?.mutations ?? fixture.mutations;
      expect(Object.keys(mutations), `${id} mutations`).not.toEqual([]);
      for (const [name, mutation] of Object.entries(mutations)) {
        const mutationIndex = structured ? mutation() : indexFor(mutation);
        expectSemanticViolations(evaluateRule({ id, index: mutationIndex }).violations, `${id}: ${name}`);
      }
    }
  });

  it("rule:rust-duplicate-baseline accepts version replacement at unchanged cardinality", () => {
    const fixture = rustDuplicateStructuredFixtures["rule:rust-duplicate-baseline"];

    expect(evaluateRule({
      id: "rule:rust-duplicate-baseline",
      index: fixture.compatibleReplacement(),
    })).toEqual({
      id: "rule:rust-duplicate-baseline",
      violations: [],
    });
  });

  it("rule:rust-duplicate-baseline rejects same-aggregate duplicate package-set replacement", () => {
    const fixture = rustDuplicateStructuredFixtures["rule:rust-duplicate-baseline"];

    expect(evaluateRule({
      id: "rule:rust-duplicate-baseline",
      index: fixture.packageSetReplacement(),
    })).toEqual({
      id: "rule:rust-duplicate-baseline",
      violations: ["current duplicate graph differs from committed baseline"],
    });

    expect(evaluateRule({
      id: "rule:rust-duplicate-baseline",
      index: fixture.multiplePackageSetReplacement(),
    }).violations).toEqual(["current duplicate graph differs from committed baseline"]);
  });

  it("rule:rust-duplicate-baseline strictly blocks aggregate and existing-name cardinality changes", () => {
    const fixture = rustDuplicateStructuredFixtures["rule:rust-duplicate-baseline"];

    const newDuplicate = evaluateRule({
      id: "rule:rust-duplicate-baseline",
      index: fixture.mutations["adds a new duplicate package name"](),
    });
    expect(newDuplicate.violations).toEqual(["current duplicate graph differs from committed baseline"]);

    expect(evaluateRule({
      id: "rule:rust-duplicate-baseline",
      index: fixture.mutations["adds a third version of an existing duplicate"](),
    })).toEqual({
      id: "rule:rust-duplicate-baseline",
      violations: ["current duplicate graph differs from committed baseline"],
    });
  });

  it("rule:telegram-crate-dependency-ownership rejects reordered generated direct packages", () => {
    const fixture = telegramStructuredFixtures["rule:telegram-crate-dependency-ownership"];
    expect(evaluateRule({
      id: "rule:telegram-crate-dependency-ownership",
      index: fixture.positive(),
    })).toEqual({
      id: "rule:telegram-crate-dependency-ownership",
      violations: [],
    });

    expectSemanticViolations(
      evaluateRule({
        id: "rule:telegram-crate-dependency-ownership",
        index: fixture.mutations["reorders the generated baseline direct packages"](),
      }).violations,
      "rule:telegram-crate-dependency-ownership: reordered generated baseline",
    );
  });

  it("throws for an unknown rule ID", () => {
    expect(() => evaluateRule({ id: "rule:not-registered", index: indexFor({}) })).toThrow(/rule:not-registered/);
  });
});
