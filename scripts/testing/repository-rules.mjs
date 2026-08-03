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
