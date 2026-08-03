const TELEGRAM_PATH = "src/lib/telegram-contract-paths.ts";
const ANALYSIS_SURFACE_PATH = "src/lib/components/analysis/report-source-surface.svelte";

const TRANSITIONAL_SOURCE_COMPONENTS = [
  "RunCompanionTabs",
  "SourceContextPanel",
  "SourceGroupReader",
  "TelegramTimelineReader",
  "YoutubePlaylistDetail",
  "YoutubePlaylistReader",
  "YoutubeSourceDetail",
  "YoutubeTranscriptReader",
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

function requireGuardFacts(violations, functionFact, functionName, expectedCalls, expectedLiterals, minimumThrows) {
  if (!functionFact) return;
  for (const call of expectedCalls) {
    if (!functionFact.guardCalls.includes(call)) {
      violations.push(`${TELEGRAM_PATH}: ${functionName} must guard with ${call}`);
    }
  }
  for (const literal of expectedLiterals) {
    if (!functionFact.guardStringLiterals.includes(literal)) {
      violations.push(`${TELEGRAM_PATH}: ${functionName} must guard the ${JSON.stringify(literal)} path case`);
    }
  }
  if (functionFact.throwCount < minimumThrows) {
    violations.push(`${TELEGRAM_PATH}: ${functionName} must fail closed from its path guards`);
  }
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
  requireGuardFacts(violations, repositoryGuard, "assertRepositoryRelative", [
    "path.isAbsolute",
    "relativePath.includes",
    "relativePath.split",
  ], ["", "\\", ".", ".."], 2);
  requireCalls(violations, resolver, "resolveTelegramContractPath", [
    "assertRepositoryRelative",
    "existsSync",
    "path.relative",
    "realpathSync",
  ]);
  requireGuardFacts(violations, resolver, "resolveTelegramContractPath", [
    "existsSync",
    "path.isAbsolute",
  ], [".."], 2);
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
  const componentNames = new Set(facts.components.map(({ name }) => name));
  const violations = [];

  if (!componentNames.has("SourceBrowserShell")) {
    violations.push(`${ANALYSIS_SURFACE_PATH}: SourceBrowserShell must own source browsing`);
  }
  for (const name of TRANSITIONAL_SOURCE_COMPONENTS) {
    if (componentNames.has(name)) {
      violations.push(`${ANALYSIS_SURFACE_PATH}: transitional ${name} composition is forbidden`);
    }
  }
  return violations;
}

const evaluators = new Map([
  ["rule:analysis-source-reader-surface-composition", evaluateAnalysisSourceReaderSurfaceComposition],
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
