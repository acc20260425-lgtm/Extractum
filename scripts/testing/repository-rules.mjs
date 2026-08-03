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

function containsExpression(expression, predicate) {
  if (!expression || typeof expression !== "object") return false;
  if (predicate(expression)) return true;
  return Object.values(expression).some((value) => Array.isArray(value)
    ? value.some((item) => containsExpression(item, predicate))
    : containsExpression(value, predicate));
}

function equality(expression, operandName, value) {
  return expression?.kind === "binary"
    && expression.operator === "==="
    && identifier(expression.left, operandName)
    && stringLiteral(expression.right, value);
}

function exactDotSegmentPredicate(expression, parameter) {
  return expression?.kind === "binary"
    && expression.operator === "||"
    && equality(expression.left, parameter, ".")
    && equality(expression.right, parameter, "..");
}

function hasDotSegmentGuard(functionFact) {
  return functionFact?.guards.some((guard) => guard.consequenceThrows
    && containsExpression(guard.condition, (expression) => {
      if (expression?.kind !== "call" || expression.callee?.kind !== "member" || expression.callee.property !== "some") {
        return false;
      }
      const split = expression.callee.object;
      if (!call(split, "relativePath.split", (args) => args.length === 1 && stringLiteral(args[0], "/"))) return false;
      const predicate = expression.arguments[0];
      const parameter = predicate?.kind === "arrow" ? predicate.parameters[0] : undefined;
      return typeof parameter === "string" && exactDotSegmentPredicate(predicate.body, parameter);
    }));
}

function hasMissingFileGuard(functionFact) {
  return functionFact?.guards.some((guard) => guard.consequenceThrows
    && guard.condition?.kind === "unary"
    && guard.condition.operator === "!"
    && call(guard.condition.operand, "existsSync", (args) => args.length === 1 && identifier(args[0], "selected")));
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
  if (!hasDotSegmentGuard(repositoryGuard)) {
    violations.push(`${TELEGRAM_PATH}: assertRepositoryRelative must reject exactly the . and .. path segments`);
  }
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
  if (!hasMissingFileGuard(resolver)) {
    violations.push(`${TELEGRAM_PATH}: resolveTelegramContractPath must reject a missing selected path`);
  }
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
