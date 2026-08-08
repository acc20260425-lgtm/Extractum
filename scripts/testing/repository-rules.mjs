import { createHash } from "node:crypto";
import { generateIdentityAuthority } from "../telegram-8b-test-identities.mjs";
import { generateSymbolAuthority } from "../telegram-8b-symbol-map.mjs";
import { generateFeatureBaseline } from "../telegram-grammers-feature-baseline.mjs";

const TELEGRAM_PATH = "src/lib/telegram-contract-paths.ts";
const ANALYSIS_SURFACE_PATH = "src/lib/components/analysis/report-source-surface.svelte";
const SOURCE_BROWSER_SHELL_PATH = "src/lib/components/analysis/source-browser-shell.svelte";
const SOURCE_GROUP_SOURCES_PATH = "src/lib/components/analysis/source-group-sources-view.svelte";
const SOURCE_GROUP_ACTIVITY_PATH = "src/lib/components/analysis/source-group-activity-view.svelte";
const SOURCE_BROWSER_SHELL_MODULE = "$lib/components/analysis/source-browser-shell.svelte";
const SOURCE_ACTIVITY_MODULE = "$lib/components/analysis/source-activity-view.svelte";
const SOURCE_GROUP_ACTIVITY_MODULE = "$lib/components/analysis/source-group-activity-view.svelte";
const EMPTY_STATE_MODULE = "$lib/components/ui/EmptyState.svelte";
const DATA_GRID_PATH = "src/lib/components/extractum-ui/DataGrid.svelte";
const TREE_DATA_GRID_PATH = "src/lib/components/extractum-ui/TreeDataGrid.svelte";
const GRID_RUNTIME_PATH = "src/lib/components/extractum-ui/data-grid-date-format.ts";
const APPROVED_SVAR_GRID_PATHS = new Set([DATA_GRID_PATH, TREE_DATA_GRID_PATH, GRID_RUNTIME_PATH]);
const CANONICAL_LEAF_PATHS = [
  ["SourceGroupSourcesView", SOURCE_GROUP_SOURCES_PATH, "$lib/components/analysis/source-group-sources-view.svelte"],
  ["SnapshotGroupSourcesView", "src/lib/components/analysis/snapshot-group-sources-view.svelte", "$lib/components/analysis/snapshot-group-sources-view.svelte"],
  ["SnapshotItemsView", "src/lib/components/analysis/snapshot-items-view.svelte", "$lib/components/analysis/snapshot-items-view.svelte"],
  ["RunSnapshotMetadataView", "src/lib/components/analysis/run-snapshot-metadata-view.svelte", "$lib/components/analysis/run-snapshot-metadata-view.svelte"],
];
const HIGHLIGHT_STYLE_TARGETS = [
  ["src/lib/components/analysis/telegram-timeline-reader.svelte", "li", "primary", true],
  ["src/lib/components/analysis/youtube-transcript-reader.svelte", "li", "primary", true],
  ["src/lib/components/analysis/snapshot-items-view.svelte", "article", "accent", true],
  ["src/lib/components/analysis/snapshot-group-sources-view.svelte", "li", "accent", true],
  ["src/lib/components/analysis/universal-items-view.svelte", "article", "accent", false],
  ["src/lib/components/analysis/youtube-comments-view.svelte", "article", "accent", false],
];
const PHASE_8A_PLAN_PATH = "docs/superpowers/plans/2026-07-26-extractum-telegram-8a-preparation.md";
const PHASE_8B_PLAN_PATH = "docs/superpowers/plans/2026-07-28-extractum-telegram-8b-preparation.md";
const SYMBOL_MAP_PATH = "src/lib/telegram-8b-symbol-map.json";
const TEST_IDENTITIES_PATH = "src/lib/telegram-8b-test-identities.json";
const STAGING_SHA_PATH = "src/lib/telegram-8b-staging-sha256.json";
const GRAMMERS_BASELINE_PATH = "src/lib/telegram-grammers-feature-baseline.json";
const PHASE_8_ROADMAP_PATH = "docs/superpowers/specs/2026-07-17-crate-roadmap.md";
const PHASE_8_DESIGN_PATH = "docs/superpowers/specs/2026-07-26-telegram-crate-boundary-design.md";
const FROZEN_STAGING_SHA256 = "12e99b10aaaccc471ae4c950b4a3ea0331ae68db45618823ea2aa58bae29d1a9";
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

const TRANSITIONAL_SOURCE_COMPONENTS = [
  ["RunCompanionTabs", "$lib/components/analysis/run-companion-tabs.svelte"],
  ["SourceContextPanel", "$lib/components/analysis/source-context-panel.svelte"],
  ["SourceGroupReader", "$lib/components/analysis/source-group-reader.svelte"],
  ["TelegramTimelineReader", "$lib/components/analysis/telegram-timeline-reader.svelte"],
  ["YoutubePlaylistDetail", "$lib/components/analysis/youtube-playlist-detail.svelte"],
  ["YoutubePlaylistReader", "$lib/components/analysis/youtube-playlist-reader.svelte"],
  ["YoutubeSourceDetail", "$lib/components/analysis/youtube-source-detail.svelte"],
  ["YoutubeTranscriptReader", "$lib/components/analysis/youtube-transcript-reader.svelte"],
];

function identifier(expression, name) {
  return expression?.kind === "identifier" && expression.name === name;
}

function stringLiteral(expression, value) {
  return expression?.kind === "string" && expression.value === value;
}

function evaluateAnalysisSourceReaderSurfaceComposition(index) {
  const facts = index.getSvelte(ANALYSIS_SURFACE_PATH);
  const violations = [];

  if (!hasComponentFromModule(facts, SOURCE_BROWSER_SHELL_MODULE)) {
    violations.push(`${ANALYSIS_SURFACE_PATH}: SourceBrowserShell must own source browsing`);
  }
  for (const [name, moduleSource] of TRANSITIONAL_SOURCE_COMPONENTS) {
    if (hasComponentFromModule(facts, moduleSource)) {
      violations.push(`${ANALYSIS_SURFACE_PATH}: transitional ${name} composition is forbidden`);
    }
  }
  return violations;
}

function evaluateAnalysisSourceBrowserExplicitSubjectContract(index) {
  const facts = index.getSvelte(ANALYSIS_SURFACE_PATH);
  const shells = componentsFromModule(facts, SOURCE_BROWSER_SHELL_MODULE);
  const violations = [];
  if (!shells.length) {
    return [`${ANALYSIS_SURFACE_PATH}: missing SourceBrowserShell composition`];
  }
  for (const [position, shell] of shells.entries()) {
    if (!shell.attributes.includes("subject")) {
      violations.push(`${ANALYSIS_SURFACE_PATH}: SourceBrowserShell ${position + 1} must receive subject`);
    }
    if (shell.attributes.includes("source")) {
      violations.push(`${ANALYSIS_SURFACE_PATH}: SourceBrowserShell ${position + 1} must not receive legacy source`);
    }
  }
  return violations;
}

function selectorHas(selector, type, name, value) {
  return selector.some((part) => part.type === type
    && part.name === name
    && (value === undefined || part.value === value));
}

function evaluateAnalysisEvidenceHighlightTokenStyling(index) {
  const violations = [];
  for (const [path, tag, token, paired] of HIGHLIGHT_STYLE_TARGETS) {
    const facts = index.getSvelte(path);
    const matchingRule = facts.styleRules.find((rule) => {
      const evidenceSelector = rule.selectors.some((selector) =>
        selectorHas(selector, "tag", tag)
        && selectorHas(selector, "attribute", "data-evidence-highlighted", "true"));
      const selectedSelector = rule.selectors.some((selector) =>
        selectorHas(selector, "tag", tag) && selectorHas(selector, "class", "selected"));
      const semanticToken = rule.declarations.some(({ value }) => value.includes(`var(--${token})`));
      return evidenceSelector && (!paired || selectedSelector) && semanticToken;
    });
    if (!matchingRule) {
      violations.push(`${path}: evidence highlight must use the semantic --${token} selection token${paired ? " beside selected styling" : ""}`);
    }
  }
  return violations;
}

function routeApiImport(source) {
  return source === "$lib/api"
    || source.startsWith("$lib/api/")
    || source === "@tauri-apps/api"
    || source.startsWith("@tauri-apps/api/");
}

function importedDefaultSource(facts, localName) {
  const matches = facts.imports.filter(({ bindings }) => bindings.some((binding) =>
    binding.imported === "default" && binding.local === localName && !binding.typeOnly));
  return matches.length === 1 ? matches[0].source : undefined;
}

function componentsFromModule(facts, moduleSource) {
  return facts.components.filter((component) => importedDefaultSource(facts, component.name) === moduleSource);
}

function hasComponentFromModule(facts, moduleSource) {
  return componentsFromModule(facts, moduleSource).length > 0;
}

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

function exactActivityTab(expression) {
  return expression?.kind === "binary"
    && expression.operator === "==="
    && ((identifier(expression.left, "activeTab") && stringLiteral(expression.right, "activity"))
      || (stringLiteral(expression.left, "activity") && identifier(expression.right, "activeTab")));
}

function conjunctionTerms(expression) {
  return expression?.kind === "binary" && expression.operator === "&&"
    ? [...conjunctionTerms(expression.left), ...conjunctionTerms(expression.right)]
    : [expression];
}

function exactPositiveActivityBranch(component, expectedIdentifiers) {
  return component.branches?.some((branch) => {
    if (branch.polarity !== "consequent") return false;
    const terms = conjunctionTerms(branch.condition);
    if (terms.length !== expectedIdentifiers.length + 1) return false;
    if (terms.filter(exactActivityTab).length !== 1) return false;
    return expectedIdentifiers.every((name) => terms.filter((term) => identifier(term, name)).length === 1);
  }) ?? false;
}

function evaluateAnalysisSourceGroupTabLeafBoundary(index) {
  const facts = index.getSvelte(SOURCE_GROUP_SOURCES_PATH);
  const violations = [];
  for (const [required, moduleSource] of [
    ["TelegramTimelineReader", "$lib/components/analysis/telegram-timeline-reader.svelte"],
    ["YoutubeTranscriptReader", "$lib/components/analysis/youtube-transcript-reader.svelte"],
  ]) {
    if (!hasComponentFromModule(facts, moduleSource)) {
      violations.push(`${SOURCE_GROUP_SOURCES_PATH}: missing leaf reader ${required}`);
    }
  }
  for (const [forbidden, moduleSource] of [
    ["SourceBrowserShell", SOURCE_BROWSER_SHELL_MODULE],
    ["SourceActivityView", SOURCE_ACTIVITY_MODULE],
  ]) {
    if (hasComponentFromModule(facts, moduleSource)) {
      violations.push(`${SOURCE_GROUP_SOURCES_PATH}: route-owning ${forbidden} is forbidden`);
    }
  }
  const routeImports = facts.imports.map(({ source }) => source).filter(routeApiImport);
  if (routeImports.length) {
    violations.push(`${SOURCE_GROUP_SOURCES_PATH}: route API imports are forbidden: ${routeImports.sort().join(", ")}`);
  }
  return violations;
}

function evaluateAnalysisSourceBrowserCanonicalComposition(index) {
  const surface = index.getSvelte(ANALYSIS_SURFACE_PATH);
  const shell = index.getSvelte(SOURCE_BROWSER_SHELL_PATH);
  const violations = [];
  if (!hasComponentFromModule(surface, SOURCE_BROWSER_SHELL_MODULE)) {
    violations.push(`${ANALYSIS_SURFACE_PATH}: missing canonical SourceBrowserShell`);
  }
  for (const [transitional, moduleSource] of TRANSITIONAL_SOURCE_COMPONENTS) {
    if (hasComponentFromModule(surface, moduleSource)) {
      violations.push(`${ANALYSIS_SURFACE_PATH}: transitional ${transitional} composition is forbidden`);
    }
  }
  for (const [component, path, moduleSource] of CANONICAL_LEAF_PATHS) {
    if (!hasComponentFromModule(shell, moduleSource)) {
      violations.push(`${SOURCE_BROWSER_SHELL_PATH}: missing canonical leaf ${component}`);
    }
    const leaf = index.getSvelte(path);
    const routeImports = leaf.imports.map(({ source }) => source).filter(routeApiImport);
    if (routeImports.length) {
      violations.push(`${path}: route API imports are forbidden: ${routeImports.sort().join(", ")}`);
    }
    if (hasComponentFromModule(leaf, SOURCE_BROWSER_SHELL_MODULE)) {
      violations.push(`${path}: nested SourceBrowserShell is forbidden`);
    }
  }
  return violations;
}

function evaluateAnalysisSourceGroupActivityBoundary(index) {
  const shell = index.getSvelte(SOURCE_BROWSER_SHELL_PATH);
  const groupActivity = index.getSvelte(SOURCE_GROUP_ACTIVITY_PATH);
  const violations = [];
  const groupBranches = componentsFromModule(shell, SOURCE_GROUP_ACTIVITY_MODULE);
  const sourceBranches = componentsFromModule(shell, SOURCE_ACTIVITY_MODULE);
  if (!groupBranches.some((component) => exactPositiveActivityBranch(component, ["groupSubject"]))) {
    violations.push(`${SOURCE_BROWSER_SHELL_PATH}: SourceGroupActivityView must be under the exact positive activity/groupSubject branch`);
  }
  if (!sourceBranches.some((component) => exactPositiveActivityBranch(component, ["sourceSubject", "sourceData"]))) {
    violations.push(`${SOURCE_BROWSER_SHELL_PATH}: SourceActivityView must be under the exact positive activity/sourceSubject/sourceData branch`);
  }
  if (!hasComponentFromModule(groupActivity, EMPTY_STATE_MODULE)) {
    violations.push(`${SOURCE_GROUP_ACTIVITY_PATH}: missing group-owned activity leaf`);
  }
  if (hasComponentFromModule(groupActivity, SOURCE_ACTIVITY_MODULE)) {
    violations.push(`${SOURCE_GROUP_ACTIVITY_PATH}: per-source SourceActivityView is forbidden`);
  }
  const routeImports = groupActivity.imports.map(({ source }) => source).filter(routeApiImport);
  if (routeImports.length) {
    violations.push(`${SOURCE_GROUP_ACTIVITY_PATH}: route API imports are forbidden: ${routeImports.sort().join(", ")}`);
  }
  return violations;
}

function sameJson(left, right) {
  return JSON.stringify(left) === JSON.stringify(right);
}

function uniqueMatch(source, pattern) {
  const matches = [...source.matchAll(pattern)];
  return matches.length === 1 ? matches[0] : undefined;
}

function retainedPhase8StatusViolations(index) {
  const violations = [];
  const phase8B = index.getText(PHASE_8B_PLAN_PATH).replaceAll("\r\n", "\n");
  const sectionStart = phase8B.indexOf("### Phase 8B Status State Machine\n");
  const sectionEnd = phase8B.indexOf("\n`TelegramLifecycle` gains exact", sectionStart);
  const section = sectionStart >= 0 && sectionEnd > sectionStart
    ? phase8B.slice(sectionStart, sectionEnd)
    : "";
  const checkpointRange = uniqueMatch(
    section,
    /roadmap `8B preparation Checkpoint (\d+) retained` through\n\s+`8B preparation Checkpoint (\d+) retained`; design\n\s+`Approved; 8B preparation Checkpoint N retained`/g,
  );
  if (checkpointRange?.[1] !== "1" || checkpointRange?.[2] !== "8") {
    violations.push(`${PHASE_8B_PLAN_PATH}: retained checkpoint status range drifted`);
  }
  const terminal = [...section.matchAll(/roadmap `([^`]+)`; design `([^`]+)`/g)]
    .find((match) => match[1] === "8B preparation retained; 8C pending");
  if (terminal?.[1] !== "8B preparation retained; 8C pending"
    || terminal?.[2] !== "Approved; 8B preparation retained; 8C pending") {
    violations.push(`${PHASE_8B_PLAN_PATH}: terminal Phase 8B retained status pair drifted`);
  }

  const roadmap = index.getText(PHASE_8_ROADMAP_PATH);
  const roadmapStatus = uniqueMatch(roadmap, /^### Phase 8 — `extractum-telegram` \(([^)]+)\)$/gm)?.[1];
  if (roadmapStatus !== "done: retained") {
    violations.push(`${PHASE_8_ROADMAP_PATH}: retained Phase 8 roadmap status drifted`);
  }
  const design = index.getText(PHASE_8_DESIGN_PATH);
  const designStatus = uniqueMatch(design, /^\*\*Status:\*\* (.+)$/gm)?.[1];
  if (designStatus !== "Implemented and retained; [verification](../verification/2026-08-01-extractum-telegram-8c-extraction.md)") {
    violations.push(`${PHASE_8_DESIGN_PATH}: retained Phase 8 design status drifted`);
  }
  return violations;
}

function evaluateTelegramPhase8BAuthorityIntegrity(index) {
  const phase8A = index.getText(PHASE_8A_PLAN_PATH);
  const phase8B = index.getText(PHASE_8B_PLAN_PATH);
  const violations = [];
  if (!sameJson(index.getJson(SYMBOL_MAP_PATH), generateSymbolAuthority(phase8B))) {
    violations.push(`${SYMBOL_MAP_PATH}: generated authority drifted`);
  }
  if (!sameJson(index.getJson(TEST_IDENTITIES_PATH), generateIdentityAuthority(phase8A, phase8B))) {
    violations.push(`${TEST_IDENTITIES_PATH}: generated authority drifted`);
  }

  const stagingSource = index.getText(STAGING_SHA_PATH).replaceAll("\r\n", "\n");
  const staging = index.getJson(STAGING_SHA_PATH);
  const paths = Array.isArray(staging?.files) ? staging.files.map((entry) => entry?.path) : [];
  const validRecords = Array.isArray(staging?.files)
    && staging.files.length === 19
    && staging.files.every((entry) => entry
      && Object.keys(entry).sort().join(",") === "path,sha256"
      && typeof entry.path === "string"
      && /^[a-f0-9]{64}$/.test(entry.sha256));
  if (staging?.schemaVersion !== 1
    || staging?.algorithm !== "sha256"
    || staging?.root !== "src-tauri/src/telegram_impl"
    || !validRecords
    || new Set(paths).size !== paths.length
    || paths.some((value, index) => index > 0 && paths[index - 1].localeCompare(value) >= 0)) {
    violations.push(`${STAGING_SHA_PATH}: frozen artifact schema drifted`);
  }
  const contentAddress = createHash("sha256").update(stagingSource).digest("hex");
  if (contentAddress !== FROZEN_STAGING_SHA256) {
    violations.push(`${STAGING_SHA_PATH}: frozen content address drifted`);
  }
  violations.push(...retainedPhase8StatusViolations(index));
  return violations;
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

  const expectedPackages = Array.isArray(baseline?.packages) ? baseline.packages : [];
  const producerGrammers = (producer.dependencies ?? []).filter((dependency) => dependency?.name?.startsWith("grammers-"));
  const actualNames = producerGrammers.map(({ name }) => name).sort();
  const expectedNames = expectedPackages.map(({ name }) => name).sort();
  if (actualNames.join("\n") !== expectedNames.join("\n")) {
    violations.push("extractum-telegram: direct Grammers dependency inventory drifted");
  }
  for (const expected of expectedPackages) {
    const packages = (metadata.packages ?? []).filter(({ name }) => name === expected.name);
    if (packages.length !== 1) {
      violations.push(`${expected.name}: expected one Cargo metadata package`);
      continue;
    }
    const selected = packages[0];
    const expectedSource = `git+https://codeberg.org/Lonami/grammers?rev=${baseline.revision}#${baseline.revision}`;
    if (selected.source !== expectedSource) violations.push(`${expected.name}: source revision drifted`);
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

const evaluators = new Map([
  ["rule:analysis-evidence-highlight-token-styling", evaluateAnalysisEvidenceHighlightTokenStyling],
  ["rule:analysis-source-browser-canonical-composition", evaluateAnalysisSourceBrowserCanonicalComposition],
  ["rule:analysis-source-browser-explicit-subject-contract", evaluateAnalysisSourceBrowserExplicitSubjectContract],
  ["rule:analysis-source-group-activity-boundary", evaluateAnalysisSourceGroupActivityBoundary],
  ["rule:analysis-source-group-tab-leaf-boundary", evaluateAnalysisSourceGroupTabLeafBoundary],
  ["rule:analysis-source-reader-surface-composition", evaluateAnalysisSourceReaderSurfaceComposition],
  ["rule:extractum-grid-wrapper-boundary", evaluateExtractumGridWrapperBoundary],
  ["rule:telegram-crate-dependency-ownership", evaluateTelegramCrateDependencyOwnership],
  ["rule:telegram-crate-manifest-boundary", evaluateTelegramCrateManifestBoundary],
  ["rule:telegram-phase-8b-authority-integrity", evaluateTelegramPhase8BAuthorityIntegrity],
]);

export const registeredRuleIds = Object.freeze([...evaluators.keys()].sort());

export function evaluateRule({ id, index }) {
  const evaluator = evaluators.get(id);
  if (!evaluator) throw new Error(`Unknown repository rule ID: ${id}`);
  try {
    return { id, violations: evaluator(index) };
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error);
    return { id, violations: [`INFRA_ERROR: ${detail}`] };
  }
}
