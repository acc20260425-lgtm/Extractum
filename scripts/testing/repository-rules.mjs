import { createHash } from "node:crypto";
import { generateIdentityAuthority } from "../telegram-8b-test-identities.mjs";
import { generateSymbolAuthority } from "../telegram-8b-symbol-map.mjs";

const TELEGRAM_PATH = "src/lib/telegram-contract-paths.ts";
const ANALYSIS_SURFACE_PATH = "src/lib/components/analysis/report-source-surface.svelte";
const SOURCE_BROWSER_SHELL_PATH = "src/lib/components/analysis/source-browser-shell.svelte";
const SOURCE_GROUP_SOURCES_PATH = "src/lib/components/analysis/source-group-sources-view.svelte";
const SOURCE_GROUP_ACTIVITY_PATH = "src/lib/components/analysis/source-group-activity-view.svelte";
const SOURCE_BROWSER_SHELL_MODULE = "$lib/components/analysis/source-browser-shell.svelte";
const SOURCE_ACTIVITY_MODULE = "$lib/components/analysis/source-activity-view.svelte";
const SOURCE_GROUP_ACTIVITY_MODULE = "$lib/components/analysis/source-group-activity-view.svelte";
const EMPTY_STATE_MODULE = "$lib/components/ui/EmptyState.svelte";
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
const FROZEN_STAGING_SHA256 = "12e99b10aaaccc471ae4c950b4a3ea0331ae68db45618823ea2aa58bae29d1a9";

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

function functionByName(facts, name) {
  return facts.functions.find((item) => item.name === name);
}

function requireCalls(violations, functionFact, functionName, expectedCalls) {
  if (!functionFact) {
    violations.push(`${TELEGRAM_PATH}: missing function ${functionName}`);
    return;
  }
  for (const call of expectedCalls) {
    if (!functionFact.calls.includes(call)) {
      violations.push(`${TELEGRAM_PATH}: ${functionName} must call ${call}`);
    }
  }
}

function identifier(expression, name) {
  return expression?.kind === "identifier" && expression.name === name;
}

function stringLiteral(expression, value) {
  return expression?.kind === "string" && expression.value === value;
}

function expressionPath(expression) {
  if (expression?.kind === "identifier") return expression.name;
  if (expression?.kind !== "member") return undefined;
  const owner = expressionPath(expression.object);
  return owner ? `${owner}.${expression.property}` : undefined;
}

function call(expression, callee, argumentsMatch) {
  return expression?.kind === "call"
    && expressionPath(expression.callee) === callee
    && argumentsMatch(expression.arguments);
}

function unary(expression, operator, operandMatch) {
  return expression?.kind === "unary"
    && expression.operator === operator
    && operandMatch(expression.operand);
}

function comparison(expression, operator, operandName, value) {
  return expression?.kind === "binary"
    && expression.operator === operator
    && ((identifier(expression.left, operandName) && stringLiteral(expression.right, value))
      || (stringLiteral(expression.left, value) && identifier(expression.right, operandName)));
}

function exactDotSegmentPredicate(expression, parameter) {
  const comparisons = expression?.kind === "binary" && expression.operator === "||"
    ? [expression.left, expression.right]
    : [];
  return expression?.kind === "binary"
    && expression.operator === "||"
    && comparisons.some((item) => comparison(item, "===", parameter, "."))
    && comparisons.some((item) => comparison(item, "===", parameter, ".."));
}

function dotSegmentPredicate(expression) {
  if (expression?.kind !== "call" || expression.callee?.kind !== "member" || expression.callee.property !== "some") {
    return false;
  }
  const split = expression.callee.object;
  if (!call(split, "relativePath.split", (args) => args.length === 1 && stringLiteral(args[0], "/"))) return false;
  const predicate = expression.arguments[0];
  const parameter = predicate?.kind === "arrow" ? predicate.parameters[0] : undefined;
  return typeof parameter === "string" && exactDotSegmentPredicate(predicate.body, parameter);
}

function disjunctionTerms(expression) {
  return expression?.kind === "binary" && expression.operator === "||"
    ? [...disjunctionTerms(expression.left), ...disjunctionTerms(expression.right)]
    : [expression];
}

function throwingGuardTerms(functionFact) {
  return functionFact?.guards
    .filter(({ consequenceThrows }) => consequenceThrows)
    .flatMap(({ condition }) => disjunctionTerms(condition)) ?? [];
}

function requireThrowingGuard(violations, functionFact, label, match) {
  if (!throwingGuardTerms(functionFact).some(match)) {
    violations.push(`${TELEGRAM_PATH}: missing fail-closed ${label} guard`);
  }
}

function pathSeparatorMember(expression) {
  return expressionPath(expression) === "path.sep";
}

function parentPrefix(expression) {
  if (expression?.kind === "binary" && expression.operator === "+") {
    return stringLiteral(expression.left, "..") && pathSeparatorMember(expression.right);
  }
  return expression?.kind === "template"
    && expression.head === ".."
    && expression.spans.length === 1
    && pathSeparatorMember(expression.spans[0].expression)
    && expression.spans[0].literal === "";
}

function startsWithParent(expression, owner) {
  return call(expression, `${owner}.startsWith`, (args) => args.length === 1 && parentPrefix(args[0]));
}

function evaluateTelegramRepositoryPathSafety(index) {
  const facts = index.getTypeScript(TELEGRAM_PATH);
  const violations = [];
  const repositoryGuard = functionByName(facts, "assertRepositoryRelative");
  const resolver = functionByName(facts, "resolveTelegramContractPath");
  const reader = functionByName(facts, "readTelegramContractFile");
  const normalizer = functionByName(facts, "normalizeTelegramContractSourceText");

  requireCalls(violations, repositoryGuard, "assertRepositoryRelative", [
    "path.isAbsolute",
    "path.resolve",
    "path.relative",
    "relativePath.split",
  ]);
  requireThrowingGuard(violations, repositoryGuard, "empty repository path", (term) =>
    unary(term, "!", (operand) => identifier(operand, "relativePath")));
  requireThrowingGuard(violations, repositoryGuard, "absolute input path", (term) =>
    call(term, "path.isAbsolute", (args) => args.length === 1 && identifier(args[0], "relativePath")));
  requireThrowingGuard(violations, repositoryGuard, "Windows-separator path", (term) =>
    call(term, "relativePath.includes", (args) => args.length === 1 && stringLiteral(args[0], "\\")));
  requireThrowingGuard(violations, repositoryGuard, "dot-segment path", dotSegmentPredicate);
  requireThrowingGuard(violations, repositoryGuard, "resolved repository root", (term) =>
    comparison(term, "===", "relative", ""));
  requireThrowingGuard(violations, repositoryGuard, "resolved parent path", (term) =>
    comparison(term, "===", "relative", ".."));
  requireThrowingGuard(violations, repositoryGuard, "resolved parent-prefix path", (term) =>
    startsWithParent(term, "relative"));
  requireThrowingGuard(violations, repositoryGuard, "resolved absolute path", (term) =>
    call(term, "path.isAbsolute", (args) => args.length === 1 && identifier(args[0], "relative")));
  requireCalls(violations, resolver, "resolveTelegramContractPath", [
    "assertRepositoryRelative",
    "existsSync",
    "path.relative",
    "realpathSync",
  ]);
  requireThrowingGuard(violations, resolver, "missing selected path", (term) =>
    unary(term, "!", (operand) => call(operand, "existsSync", (args) => args.length === 1 && identifier(args[0], "selected"))));
  requireThrowingGuard(violations, resolver, "realpath parent escape", (term) =>
    comparison(term, "===", "realRelative", ".."));
  requireThrowingGuard(violations, resolver, "realpath parent-prefix escape", (term) =>
    startsWithParent(term, "realRelative"));
  requireThrowingGuard(violations, resolver, "realpath absolute escape", (term) =>
    call(term, "path.isAbsolute", (args) => args.length === 1 && identifier(args[0], "realRelative")));
  requireCalls(violations, reader, "readTelegramContractFile", [
    "normalizeTelegramContractSourceText",
    "readFileSync",
    "resolveTelegramContractPath",
  ]);

  for (const item of [resolver, reader, normalizer]) {
    if (item && !item.exported) violations.push(`${TELEGRAM_PATH}: ${item.name} must be exported`);
  }

  return violations;
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

function componentHasConsequentIdentifier(component, identifierName) {
  return component.branches?.some((branch) =>
    branch.polarity === "consequent" && branch.conditionIdentifiers.includes(identifierName)) ?? false;
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
  if (!groupBranches.some((component) => componentHasConsequentIdentifier(component, "groupSubject"))) {
    violations.push(`${SOURCE_BROWSER_SHELL_PATH}: SourceGroupActivityView must be under the groupSubject branch`);
  }
  if (!sourceBranches.some((component) => componentHasConsequentIdentifier(component, "sourceSubject"))) {
    violations.push(`${SOURCE_BROWSER_SHELL_PATH}: SourceActivityView must be under the sourceSubject branch`);
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

function evaluateTelegramCrateManifestBoundary(index) {
  const metadata = index.getCargoMetadata();
  const violations = [];
  const app = workspacePackage(metadata, "extractum");
  const producer = workspacePackage(metadata, "extractum-telegram");
  if (!app) violations.push("Cargo metadata: missing extractum workspace package");
  if (!producer) return [...violations, "Cargo metadata: missing extractum-telegram workspace package"];
  if (!(producer.targets ?? []).some((target) => target?.kind?.includes("lib") && target.name === "extractum_telegram")) {
    violations.push("extractum-telegram: missing library target");
  }
  if (!Object.prototype.hasOwnProperty.call(producer.features ?? {}, "app-test-support")) {
    violations.push("extractum-telegram: missing app-test-support feature");
  }
  if (app) {
    const edges = (app.dependencies ?? []).filter((dependency) => dependency?.name === "extractum-telegram");
    const normal = edges.filter((dependency) => dependency.kind === null);
    const development = edges.filter((dependency) => dependency.kind === "dev");
    if (normal.length !== 1 || (normal[0]?.features ?? []).length !== 0) {
      violations.push("extractum: production extractum-telegram edge must be feature-free");
    }
    if (development.length !== 1
      || [...(development[0]?.features ?? [])].sort().join(",") !== "app-test-support") {
      violations.push("extractum: dev extractum-telegram edge must enable only app-test-support");
    }
  }
  return violations;
}

function evaluateAnalysisCrateManifestBoundary(index) {
  const metadata = index.getCargoMetadata();
  if (!metadata || typeof metadata !== "object"
    || !Array.isArray(metadata.packages)
    || !Array.isArray(metadata.workspace_members)) {
    throw new Error("Cargo metadata: malformed workspace package inventory");
  }

  const violations = [];
  const app = workspacePackage(metadata, "extractum");
  const analysis = workspacePackage(metadata, "extractum-analysis");
  if (!app) violations.push("Cargo metadata: missing extractum workspace package");
  if (!analysis) return [...violations, "Cargo metadata: missing extractum-analysis workspace package"];

  if (!(analysis.targets ?? []).some((target) => target?.kind?.includes("lib") && target.name === "extractum_analysis")) {
    violations.push("extractum-analysis: missing library target");
  }

  if (app) {
    const normalPathEdges = (app.dependencies ?? []).filter((dependency) =>
      dependency?.name === "extractum-analysis"
      && dependency.kind === null
      && dependency.source === null
      && typeof dependency.path === "string"
      && dependency.path.length > 0);
    if (normalPathEdges.length !== 1) {
      violations.push("extractum: expected one normal path dependency on extractum-analysis");
    }
  }

  const applicationOnly = (analysis.dependencies ?? [])
    .map((dependency) => dependency?.name)
    .filter((name) => typeof name === "string" && (name === "tauri" || name.startsWith("tauri-")))
    .sort();
  if (applicationOnly.length) {
    violations.push(`extractum-analysis: application-only dependencies are forbidden: ${applicationOnly.join(", ")}`);
  }
  return violations;
}

function evaluateTelegramCrateDependencyOwnership(index) {
  const metadata = index.getCargoMetadata();
  const baseline = index.getJson(GRAMMERS_BASELINE_PATH);
  const violations = [];
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
  ["rule:analysis-crate-manifest-boundary", evaluateAnalysisCrateManifestBoundary],
  ["rule:analysis-evidence-highlight-token-styling", evaluateAnalysisEvidenceHighlightTokenStyling],
  ["rule:analysis-source-browser-canonical-composition", evaluateAnalysisSourceBrowserCanonicalComposition],
  ["rule:analysis-source-browser-explicit-subject-contract", evaluateAnalysisSourceBrowserExplicitSubjectContract],
  ["rule:analysis-source-group-activity-boundary", evaluateAnalysisSourceGroupActivityBoundary],
  ["rule:analysis-source-group-tab-leaf-boundary", evaluateAnalysisSourceGroupTabLeafBoundary],
  ["rule:analysis-source-reader-surface-composition", evaluateAnalysisSourceReaderSurfaceComposition],
  ["rule:telegram-crate-dependency-ownership", evaluateTelegramCrateDependencyOwnership],
  ["rule:telegram-crate-manifest-boundary", evaluateTelegramCrateManifestBoundary],
  ["rule:telegram-phase-8b-authority-integrity", evaluateTelegramPhase8BAuthorityIntegrity],
  ["rule:telegram-repository-path-safety", evaluateTelegramRepositoryPathSafety],
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
