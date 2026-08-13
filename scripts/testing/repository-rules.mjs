import { generateFeatureBaseline } from "../telegram-grammers-feature-baseline.mjs";
import { generateRustDuplicateBaseline } from "../rust-duplicate-baseline.mjs";

const DATA_GRID_PATH = "src/lib/components/extractum-ui/DataGrid.svelte";
const TREE_DATA_GRID_PATH = "src/lib/components/extractum-ui/TreeDataGrid.svelte";
const GRID_RUNTIME_PATH = "src/lib/components/extractum-ui/data-grid-date-format.ts";
const APPROVED_SVAR_GRID_PATHS = new Set([DATA_GRID_PATH, TREE_DATA_GRID_PATH, GRID_RUNTIME_PATH]);
const GRAMMERS_BASELINE_PATH = "src/lib/telegram-grammers-feature-baseline.json";
const RUST_DUPLICATE_BASELINE_PATH = "scripts/testing/rust-duplicate-baseline.json";
const RUST_DEPENDENCY_POLICY_PATH = "scripts/testing/rust-dependency-policy.json";
const EXPECTED_PRODUCER_DEPENDENCIES = [
  ["base64", null, [], true, null, null],
  ["chacha20poly1305", null, ["std"], true, null, null],
  ["extractum-core", null, [], true, null, null],
  ["grammers-client", null, [], false, null, null],
  ["grammers-mtsender", null, [], true, null, null],
  ["grammers-session", null, ["serde"], false, null, null],
  ["grammers-tl-types", null, ["deserializable-functions"], true, null, null],
  ["rand_core", null, ["getrandom"], true, null, null],
  ["secrecy", null, [], true, null, null],
  ["serde", null, ["derive"], true, null, null],
  ["serde_json", null, [], true, null, null],
  ["tokio", null, ["rt", "sync", "time"], true, null, null],
  ["tokio", "dev", ["macros", "test-util"], true, null, null],
];
const EXPECTED_APP_TELEGRAM_DEPENDENCIES = [
  ["extractum-telegram", null, [], true, null, null],
  ["extractum-telegram", "dev", ["app-test-support"], true, null, null],
];

function hasNamedComponentFromModule(facts, componentName, moduleSource, importedName) {
  return facts.components.some(({ name }) => name === componentName)
    && facts.imports.some(({ source, bindings }) => source === moduleSource
      && bindings.some((binding) => binding.imported === importedName
        && binding.local === componentName
        && !binding.typeOnly));
}

function selectorContainsGlobalClass(selector, className) {
  return selector.some((part) => part.type === "pseudo"
    && part.name === "global"
    && part.arguments.some((argument) => argument.type === "class" && argument.name === className));
}

function evaluateExtractumGridWrapperBoundary(index) {
  const violations = [];
  const productionFiles = index.listFiles().filter((file) =>
    file.startsWith("src/")
    && /\.(?:svelte|[cm]?[jt]sx?)$/.test(file)
    && !/\.(?:test|spec)\./.test(file));
  for (const file of productionFiles) {
    const source = index.getText(file);
    // A backslash is the only way to spell module-specifier characters non-literally,
    // including line continuations, so always parse those files.
    if (!source.includes("@svar-ui/") && !source.includes("\\")) continue;
    const facts = file.endsWith(".svelte") ? index.getSvelte(file) : index.getTypeScript(file);
    const svarImports = facts.imports.filter(({ source }) => source.startsWith("@svar-ui/"));
    if (svarImports.length && !APPROVED_SVAR_GRID_PATHS.has(file)) {
      violations.push(`${file}: direct SVAR imports must stay inside Extractum grid wrappers`);
    }
  }

  for (const [file, tree] of [[DATA_GRID_PATH, false], [TREE_DATA_GRID_PATH, true]]) {
    const facts = index.getSvelte(file);
    for (const [component, moduleSource] of [
      ["Grid", "@svar-ui/svelte-grid"],
      ["Willow", "@svar-ui/svelte-grid"],
      ["Locale", "@svar-ui/svelte-core"],
    ]) {
      if (!hasNamedComponentFromModule(facts, component, moduleSource, component)) {
        violations.push(`${file}: missing ${component} wrapper composition`);
      }
    }
    if (tree && !facts.components.some(({ name, attributes }) => name === "Grid" && attributes.includes("tree"))) {
      violations.push(`${file}: tree wrapper must compose Grid with tree enabled`);
    }
  }

  const treeFacts = index.getSvelte(TREE_DATA_GRID_PATH);
  const hasScopedSvarCellStyle = treeFacts.styleRules.some((rule) =>
    rule.selectors.some((selector) => selector.some((part) =>
      part.type === "class" && part.name === "extractum-tree-data-grid")
      && selectorContainsGlobalClass(selector, "wx-cell")));
  if (!hasScopedSvarCellStyle) {
    violations.push(`${TREE_DATA_GRID_PATH}: SVAR cell styling must remain scoped by the Extractum tree wrapper`);
  }
  return violations;
}

function sameJson(left, right) {
  return JSON.stringify(left) === JSON.stringify(right);
}

function workspacePackage(metadata, name) {
  const members = new Set(metadata?.workspace_members ?? []);
  const matches = (metadata?.packages ?? []).filter((candidate) => candidate?.name === name && members.has(candidate.id));
  return matches.length === 1 ? matches[0] : undefined;
}

function resolvedNode(metadata, packageId) {
  const matches = (metadata?.resolve?.nodes ?? []).filter((candidate) => candidate?.id === packageId);
  return matches.length === 1 ? matches[0] : undefined;
}

function dependencyTuple(dependency) {
  return [
    dependency?.name,
    dependency?.kind ?? null,
    [...(dependency?.features ?? [])].sort(),
    dependency?.uses_default_features ?? true,
    dependency?.target ?? null,
    dependency?.rename ?? null,
  ];
}

function sortedJson(values) {
  return values.map((value) => JSON.stringify(value)).sort().join("\n");
}

function exactDependencyInventory(actual, expected) {
  return sortedJson(actual.map(dependencyTuple)) === sortedJson(expected);
}

function exactDependencyKinds(edge, expectedKinds) {
  const actual = (edge?.dep_kinds ?? []).map(({ kind, target }) => [kind ?? null, target ?? null]);
  return sortedJson(actual) === sortedJson(expectedKinds.map((kind) => [kind, null]));
}

function evaluateTelegramCrateManifestBoundary(index) {
  const metadata = index.getCargoMetadata();
  const violations = [];
  const app = workspacePackage(metadata, "extractum");
  const producer = workspacePackage(metadata, "extractum-telegram");
  if (!app) violations.push("Cargo metadata: missing extractum workspace package");
  if (!producer) return [...violations, "Cargo metadata: missing extractum-telegram workspace package"];
  const producerTargets = (producer.targets ?? []).filter((target) => target?.kind?.includes("lib"));
  if (producerTargets.length !== 1
    || producerTargets[0]?.name !== "extractum_telegram"
    || sortedJson(producerTargets[0]?.kind ?? []) !== sortedJson(["lib"])) {
    violations.push("extractum-telegram: library target inventory drifted");
  }
  if (!sameJson(producer.features ?? {}, { "app-test-support": [] })) {
    violations.push("extractum-telegram: feature inventory must contain only app-test-support");
  }
  if (!exactDependencyInventory(producer.dependencies ?? [], EXPECTED_PRODUCER_DEPENDENCIES)) {
    violations.push("extractum-telegram: dependency and dev-dependency inventory drifted");
  }
  if (app) {
    const edges = (app.dependencies ?? []).filter((dependency) => dependency?.name === "extractum-telegram");
    const normal = edges.filter((dependency) => dependency.kind === null);
    const development = edges.filter((dependency) => dependency.kind === "dev");
    if (normal.length !== 1
      || !exactDependencyInventory(normal, [EXPECTED_APP_TELEGRAM_DEPENDENCIES[0]])) {
      violations.push("extractum: production extractum-telegram edge must be feature-free");
    }
    if (development.length !== 1
      || !exactDependencyInventory(development, [EXPECTED_APP_TELEGRAM_DEPENDENCIES[1]])) {
      violations.push("extractum: dev extractum-telegram edge must enable only app-test-support");
    }
    if (!exactDependencyInventory(edges, EXPECTED_APP_TELEGRAM_DEPENDENCIES)) {
      violations.push("extractum: Telegram dependency kinds or targets drifted");
    }

    const appNode = resolvedNode(metadata, app.id);
    const resolvedEdges = (appNode?.deps ?? []).filter(({ pkg }) => pkg === producer.id);
    if (resolvedEdges.length !== 1 || !exactDependencyKinds(resolvedEdges[0], [null, "dev"])) {
      violations.push("extractum: resolver-v2 Telegram normal/dev edge semantics drifted");
    }
  }

  const producerNode = resolvedNode(metadata, producer.id);
  if (!producerNode || sortedJson(producerNode.features ?? []) !== sortedJson(["app-test-support"])) {
    violations.push("extractum-telegram: resolved app-test-support feature drifted");
  }
  const expectedResolvedDependencies = EXPECTED_PRODUCER_DEPENDENCIES
    .filter(([name, kind]) => !(name === "tokio" && kind === "dev"))
    .map(([name]) => String(name).replaceAll("-", "_"));
  const actualResolvedDependencies = (producerNode?.deps ?? []).map(({ name }) => name);
  if (sortedJson(actualResolvedDependencies) !== sortedJson(expectedResolvedDependencies)) {
    violations.push("extractum-telegram: resolved dependency inventory drifted");
  }
  const tokioResolved = (producerNode?.deps ?? []).filter(({ name }) => name === "tokio");
  if (tokioResolved.length !== 1 || !exactDependencyKinds(tokioResolved[0], [null, "dev"])) {
    violations.push("extractum-telegram: resolved Tokio normal/dev edge semantics drifted");
  }

  const workspaceMembers = new Set(metadata?.workspace_members ?? []);
  const featureMentions = [];
  for (const workspace of (metadata?.packages ?? []).filter(({ id }) => workspaceMembers.has(id))) {
    if (Object.prototype.hasOwnProperty.call(workspace.features ?? {}, "app-test-support")) {
      featureMentions.push(`${workspace.name}|feature`);
    }
    for (const dependency of workspace.dependencies ?? []) {
      if ((dependency.features ?? []).includes("app-test-support")) {
        featureMentions.push(`${workspace.name}|${dependency.kind ?? "normal"}`);
      }
    }
  }
  if (sortedJson(featureMentions) !== sortedJson([
    "extractum-telegram|feature",
    "extractum|dev",
  ])) {
    violations.push("Cargo metadata: app-test-support feature mentions drifted");
  }
  return violations;
}

function evaluateTelegramCrateDependencyOwnership(index) {
  const metadata = index.getCargoMetadata();
  const baseline = index.getJson(GRAMMERS_BASELINE_PATH);
  const violations = [];
  let generatedBaseline;
  try {
    generatedBaseline = generateFeatureBaseline(metadata);
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error);
    violations.push(`Cargo metadata: cannot generate Grammers feature baseline: ${detail}`);
    return violations;
  }
  if (!sameJson(baseline, generatedBaseline)) {
    violations.push(`${GRAMMERS_BASELINE_PATH}: generated baseline is not canonical or reproducible`);
  }
  const app = workspacePackage(metadata, "extractum");
  const producer = workspacePackage(metadata, "extractum-telegram");
  if (!app || !producer) return ["Cargo metadata: missing Telegram owner package"];
  const appGrammers = (app.dependencies ?? []).filter((dependency) => dependency?.name?.startsWith("grammers-"));
  if (appGrammers.length) violations.push(`extractum: direct Grammers dependencies are forbidden: ${appGrammers.map(({ name }) => name).sort().join(", ")}`);

  const expectedPackages = Array.isArray(baseline?.directPackages) ? baseline.directPackages : [];
  const producerGrammers = (producer.dependencies ?? []).filter((dependency) => dependency?.name?.startsWith("grammers-"));
  const actualNames = producerGrammers.map(({ name }) => name).sort();
  const expectedNames = expectedPackages.map(({ name }) => name).sort();
  if (actualNames.join("\n") !== expectedNames.join("\n")) {
    violations.push("extractum-telegram: direct Grammers dependency inventory drifted");
  }
  const resolvedPackages = Array.isArray(baseline?.resolvedPackages)
    ? baseline.resolvedPackages
    : [];
  for (const dependency of producerGrammers) {
    const resolvedPackage = resolvedPackages.find(({ name }) => name === dependency.name);
    if (!resolvedPackage) {
      violations.push(`${dependency.name}: missing resolved baseline entry`);
      continue;
    }
    const expectedRequirement = `=${resolvedPackage.version}`;
    if (dependency.req !== expectedRequirement) {
      violations.push(
        `${dependency.name}: direct manifest requirement must be ${expectedRequirement}`,
      );
    }
  }
  for (const expected of expectedPackages) {
    const packages = (metadata.packages ?? []).filter(({ name }) => name === expected.name);
    if (packages.length !== 1) {
      violations.push(`${expected.name}: expected one Cargo metadata package`);
      continue;
    }
    const selected = packages[0];
    if (Object.keys(selected.features ?? {}).sort().join("\n") !== [...expected.universe].sort().join("\n")) {
      violations.push(`${expected.name}: feature universe drifted`);
    }
    const node = resolvedNode(metadata, selected.id);
    if (!node) {
      violations.push(`${expected.name}: missing resolved Cargo node`);
      continue;
    }
    const enabled = [...(node.features ?? [])].sort();
    if (enabled.join("\n") !== [...expected.required].sort().join("\n")
      || enabled.some((feature) => expected.forbidden.includes(feature))) {
      violations.push(`${expected.name}: enabled feature closure drifted`);
    }
  }
  return violations;
}

function normalizeText(source) {
  return source.replaceAll("\r\n", "\n").trimEnd() + "\n";
}

function dependencyByName(metadata, packageName, dependencyName) {
  const selected = workspacePackage(metadata, packageName);
  return selected?.dependencies?.find(
    ({ name, kind }) => name === dependencyName && (kind === null || kind === "build"),
  );
}

function requirementMajor(requirement) {
  const match = /(?:^|[^0-9])(\d+)(?:\.|$)/.exec(String(requirement));
  return match ? Number(match[1]) : undefined;
}

function requirementMajorMinor(requirement) {
  const match = /(?:^|[^0-9])(\d+)\.(\d+)(?:\.|$)/.exec(String(requirement));
  return match ? [Number(match[1]), Number(match[2])] : undefined;
}

function prereleaseVersion(requirement) {
  const match = /(\d+\.\d+\.\d+-[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)/.exec(String(requirement));
  return match?.[1];
}

function workspaceDependencies(metadata) {
  const members = new Set(metadata?.workspace_members ?? []);
  return (metadata?.packages ?? [])
    .filter(({ id }) => members.has(id))
    .flatMap(({ dependencies = [] }) => dependencies)
    .filter(({ kind }) => kind === null || kind === "build");
}

function npmRequirement(packageJson, name) {
  return packageJson?.dependencies?.[name] ?? packageJson?.devDependencies?.[name];
}

function evaluateRustToolchainPolicy(index) {
  const policy = index.getJson(RUST_DEPENDENCY_POLICY_PATH);
  const metadata = index.getCargoMetadata();
  const violations = [];
  const expectedToolchain = `[toolchain]
channel = "${policy.toolchain.channel}"
components = ["rustfmt", "clippy"]
targets = ["${policy.toolchain.target}"]
profile = "minimal"
`;
  if (normalizeText(index.getText("rust-toolchain.toml")) !== expectedToolchain) {
    violations.push("rust-toolchain.toml: canonical content drifted");
  }
  for (const name of policy.toolchain.workspacePackages) {
    const pkg = workspacePackage(metadata, name);
    if (!pkg) {
      violations.push(`${name}: missing workspace package`);
      continue;
    }
    if (pkg.rust_version !== policy.toolchain.rustVersion) violations.push(`${name}: rust-version drifted`);
    if (pkg.edition !== policy.toolchain.edition) violations.push(`${name}: edition drifted during Wave 0`);
    if (JSON.stringify(pkg.publish) !== "[]") violations.push(`${name}: package must be unpublished`);
  }
  return violations;
}

function evaluateRustDependencyPolicy(index) {
  const policy = index.getJson(RUST_DEPENDENCY_POLICY_PATH);
  const metadata = index.getCargoMetadata();
  const packageJson = index.getJson("package.json");
  const dependencies = workspaceDependencies(metadata);
  const violations = [];

  for (const [name, requirement] of Object.entries(policy.exactPins)) {
    const direct = dependencies.filter((dependency) => dependency.name === name);
    if (!direct.length) {
      violations.push(`${name}: missing direct dependency`);
      continue;
    }
    for (const dependency of direct) {
      if (dependency.req !== requirement) {
        violations.push(`${name}: direct manifest requirement must be ${requirement}`);
      }
    }
  }

  for (const dependency of dependencies) {
    const version = prereleaseVersion(dependency.req);
    if (version && policy.approvedPrereleases[dependency.name] !== version) {
      violations.push(`${dependency.name}: unapproved prerelease ${version}`);
    }
  }

  for (const [cargoName, npmName] of policy.tauriFamily.pairs) {
    const cargoDependency = dependencyByName(metadata, "extractum", cargoName);
    const npmVersion = npmRequirement(packageJson, npmName);
    if (!cargoDependency) violations.push(`${cargoName}: missing direct dependency`);
    else if (requirementMajor(cargoDependency.req) !== policy.tauriFamily.cargoMajor) {
      violations.push(`${cargoName}: Cargo requirement must use major ${policy.tauriFamily.cargoMajor}`);
    }
    if (!npmVersion) violations.push(`${npmName}: missing npm dependency`);
    else if (requirementMajor(npmVersion) !== policy.tauriFamily.npmMajor) {
      violations.push(`${npmName}: npm requirement must use major ${policy.tauriFamily.npmMajor}`);
    }
  }

  const mcpBridge = dependencyByName(metadata, "extractum", "tauri-plugin-mcp-bridge");
  const expectedMcpBridge = [0, policy.tauriFamily.mcpBridgeMinor];
  if (!mcpBridge) violations.push("tauri-plugin-mcp-bridge: missing direct dependency");
  else if (JSON.stringify(requirementMajorMinor(mcpBridge.req)) !== JSON.stringify(expectedMcpBridge)) {
    violations.push(`tauri-plugin-mcp-bridge: requirement must stay within 0.${policy.tauriFamily.mcpBridgeMinor}`);
  }
  return violations;
}

function evaluateRustDuplicateBaseline(index) {
  const baseline = index.getJson(RUST_DUPLICATE_BASELINE_PATH);
  const actual = generateRustDuplicateBaseline(index.getCargoTree());
  const violations = [];
  if (actual.duplicateNameCount > baseline.duplicateNameCount) violations.push("Rust duplicate-name count grew");
  if (actual.duplicateVersionInstanceCount > baseline.duplicateVersionInstanceCount) violations.push("Rust duplicate version-instance count grew");
  for (const [name, count] of Object.entries(actual.duplicateCardinality)) {
    if (Object.hasOwn(baseline.duplicateCardinality, name)
      && count > baseline.duplicateCardinality[name]) {
      violations.push(`${name}: duplicate version cardinality grew to ${count}`);
    }
  }
  const baselineNames = Object.keys(baseline.duplicateCardinality);
  const actualNames = Object.keys(actual.duplicateCardinality);
  const addedDuplicateNames = actualNames.filter((name) => !baselineNames.includes(name)).sort();
  const removedDuplicateNames = baselineNames.filter((name) => !actualNames.includes(name)).sort();
  return addedDuplicateNames.length || removedDuplicateNames.length
    ? { violations, review: { addedDuplicateNames, removedDuplicateNames } }
    : { violations };
}

const evaluators = new Map([
  ["rule:extractum-grid-wrapper-boundary", evaluateExtractumGridWrapperBoundary],
  ["rule:rust-dependency-policy", evaluateRustDependencyPolicy],
  ["rule:rust-duplicate-baseline", evaluateRustDuplicateBaseline],
  ["rule:rust-toolchain-policy", evaluateRustToolchainPolicy],
  ["rule:telegram-crate-dependency-ownership", evaluateTelegramCrateDependencyOwnership],
  ["rule:telegram-crate-manifest-boundary", evaluateTelegramCrateManifestBoundary],
]);

export const registeredRuleIds = Object.freeze([...evaluators.keys()].sort());

export function evaluateRule({ id, index }) {
  const evaluator = evaluators.get(id);
  if (!evaluator) throw new Error(`Unknown repository rule ID: ${id}`);
  try {
    const result = evaluator(index);
    return Array.isArray(result) ? { id, violations: result } : { id, ...result };
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error);
    return { id, violations: [`INFRA_ERROR: ${detail}`] };
  }
}
