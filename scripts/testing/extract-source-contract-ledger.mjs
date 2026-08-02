import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import tsDefault from "typescript";

const TEST_NAMES = new Set(["it", "test"]);
const SUITE_NAMES = new Set(["describe", "suite"]);
const DISPOSITIONS = new Set(["behavior", "architecture", "tool_owned", "delete"]);
const REPLACEMENT = /^(?:test:(?:vitest|playwright|cargo):|rule:|tool:)[^\s][\s\S]*$/;
const LIFECYCLE_FIELDS = new Set(["status", "state", "pending", "dualRun", "retired", "timestamp", "timestamps", "duration", "durations", "counter", "counters"]);
const HELPER_MODULES = new Set(["analysis-contract-paths", "telegram-contract-paths", "prompt-pack-contract-paths"]);

const normalizeText = (text) => String(text).replace(/\r\n?/g, "\n");
const sha256 = (text) => createHash("sha256").update(normalizeText(text)).digest("hex");
const sourceRange = (file, node) => {
  const start = file.getLineAndCharacterOfPosition(node.getStart(file));
  const end = file.getLineAndCharacterOfPosition(node.getEnd());
  return `${start.line + 1}:${start.character + 1}-${end.line + 1}:${end.character + 1}`;
};
const relativePath = (from, target) => path.posix.normalize(path.posix.join(path.posix.dirname(from), target)).replace(/^\.\//, "");

function normalFiles(sourceFiles, typescript) {
  return (sourceFiles ?? []).map((entry) => {
    if (entry?.kind !== undefined && typeof entry.getFullText === "function") return { path: entry.fileName.replaceAll("\\", "/"), source: entry.getFullText(), file: entry };
    const filePath = (entry.path ?? entry.fileName ?? "").replaceAll("\\", "/");
    const source = entry.source ?? entry.text ?? "";
    return { path: filePath, source, file: typescript.createSourceFile(filePath, source, typescript.ScriptTarget.Latest, true) };
  });
}

function staticText(node, typescript) {
  return typescript.isStringLiteral(node) || typescript.isNoSubstitutionTemplateLiteral(node) ? node.text : undefined;
}

function callKind(node, typescript) {
  if (!typescript.isCallExpression(node)) return undefined;
  if (typescript.isIdentifier(node.expression)) return { kind: node.expression.text, args: node.arguments, callback: node.arguments[1] };
  if (!typescript.isCallExpression(node.expression)) return undefined;
  const each = node.expression;
  if (!typescript.isPropertyAccessExpression(each.expression) || each.expression.name.text !== "each" || !typescript.isIdentifier(each.expression.expression)) return undefined;
  return { kind: each.expression.expression.text, args: node.arguments, callback: node.arguments[1], eachTable: each.arguments[0] };
}

function findInitializer(file, name, typescript) {
  let found;
  const visit = (node) => {
    if (!found && typescript.isVariableDeclaration(node) && typescript.isIdentifier(node.name) && node.name.text === name && node.initializer) found = node;
    typescript.forEachChild(node, visit);
  };
  visit(file);
  return found;
}

function staticEachTable(node, typescript) {
  let current = node;
  while (typescript.isAsExpression(current) || typescript.isSatisfiesExpression(current) || typescript.isParenthesizedExpression(current)) current = current.expression;
  return typescript.isArrayLiteralExpression(current) || typescript.isTaggedTemplateExpression(current);
}

function assertionOrdinals(callback, typescript) {
  const ordinals = [];
  const visit = (node) => {
    if (typescript.isCallExpression(node)) {
      const expression = node.expression;
      const isExpect = typescript.isIdentifier(expression) && expression.text === "expect";
      const isAssert = typescript.isPropertyAccessExpression(expression) && typescript.isIdentifier(expression.expression) && ["assert", "strict"].includes(expression.expression.text);
      if (isExpect || isAssert) ordinals.push(ordinals.length + 1);
    }
    typescript.forEachChild(node, visit);
  };
  if (callback) visit(callback);
  return ordinals;
}

function references(callback, typescript) {
  const names = new Set();
  const visit = (node) => {
    if (typescript.isIdentifier(node) && !["expect", "assert", "it", "test", "describe", "suite"].includes(node.text)) names.add(node.text);
    typescript.forEachChild(node, visit);
  };
  if (callback) visit(callback);
  return [...names].sort();
}

export function discoverTestDeclarations(sourceFiles, typescript = tsDefault) {
  const declarations = [];
  const manualRequirements = [];
  for (const entry of normalFiles(sourceFiles, typescript)) {
    const visit = (node, parents = []) => {
      const call = callKind(node, typescript);
      if (call && SUITE_NAMES.has(call.kind)) {
        const title = staticText(call.args[0], typescript);
        if (title === undefined) manualRequirements.push({ path: entry.path, sourceRange: sourceRange(entry.file, node), reason: "computed suite title" });
        else if (call.callback && (typescript.isArrowFunction(call.callback) || typescript.isFunctionExpression(call.callback))) {
          const body = call.callback.body;
          typescript.forEachChild(body, (child) => visit(child, [...parents, title]));
        } else manualRequirements.push({ path: entry.path, sourceRange: sourceRange(entry.file, node), reason: "dynamic suite factory" });
        return;
      }
      if (call && TEST_NAMES.has(call.kind)) {
        const title = staticText(call.args[0], typescript);
        const callback = call.callback;
        if (title === undefined) manualRequirements.push({ path: entry.path, sourceRange: sourceRange(entry.file, node), reason: "computed test title" });
        else if (!callback || !(typescript.isArrowFunction(callback) || typescript.isFunctionExpression(callback))) manualRequirements.push({ path: entry.path, sourceRange: sourceRange(entry.file, node), reason: "dynamic test factory" });
        else {
          let eachAuthorityText;
          if (call.eachTable) {
            if (typescript.isIdentifier(call.eachTable)) {
              const declaration = findInitializer(entry.file, call.eachTable.text, typescript);
              if (!declaration || !staticEachTable(declaration.initializer, typescript)) {
                manualRequirements.push({ path: entry.path, sourceRange: sourceRange(entry.file, node), reason: "unresolved dynamic .each table" });
                return;
              }
              const statement = declaration.parent?.parent;
              eachAuthorityText = normalizeText(statement?.getText(entry.file) ?? declaration.getText(entry.file));
            } else if (staticEachTable(call.eachTable, typescript)) eachAuthorityText = normalizeText(call.eachTable.getText(entry.file));
            else {
              manualRequirements.push({ path: entry.path, sourceRange: sourceRange(entry.file, node), reason: "unresolved dynamic .each table" });
              return;
            }
          }
          const sourceSlice = normalizeText(node.getText(entry.file));
          declarations.push({ path: entry.path, title: [...parents, title].join(" > "), sourceSlice, sourceOffset: node.getStart(entry.file), assertionOrdinals: assertionOrdinals(callback, typescript), eachAuthorityText, referencedSymbols: references(callback, typescript) });
        }
        return;
      }
      typescript.forEachChild(node, (child) => visit(child, parents));
    };
    visit(entry.file);
  }
  declarations.sort((a, b) => a.path.localeCompare(b.path) || a.sourceOffset - b.sourceOffset);
  Object.defineProperty(declarations, "manualRequirements", { value: manualRequirements.sort((a, b) => a.path.localeCompare(b.path) || a.sourceRange.localeCompare(b.sourceRange)), enumerable: true });
  return declarations;
}

function staticPath(node, file, typescript) {
  const literal = staticText(node, typescript);
  if (literal !== undefined) return literal;
  if (typescript.isIdentifier(node)) {
    const declaration = findInitializer(file, node.text, typescript);
    if (declaration) return staticPath(declaration.initializer, file, typescript);
  }
  if (typescript.isPropertyAccessExpression(node) && node.expression.getText(file) === "import.meta" && node.name.text === "dirname") return path.posix.dirname(file.fileName.replaceAll("\\", "/"));
  if (typescript.isCallExpression(node)) {
    const propertyName = typescript.isPropertyAccessExpression(node.expression) && node.expression.expression.getText(file) === "path" ? node.expression.name.text : typescript.isIdentifier(node.expression) ? node.expression.text : undefined;
    if (["join", "resolve"].includes(propertyName)) {
      const parts = [...node.arguments].map((part) => staticPath(part, file, typescript));
      return parts.every((part) => typeof part === "string") ? path.posix.normalize(parts.join("/")).replace(/^\.\//, "") : undefined;
    }
    if (typescript.isPropertyAccessExpression(node.expression) && node.expression.expression.getText(file) === "process" && node.expression.name.text === "cwd") return "";
  }
  return undefined;
}

function trackedAuthority(candidate, filePath, tracked) {
  if (!candidate) return undefined;
  const clean = candidate.replace(/\?.*$/, "").replaceAll("\\", "/");
  const resolved = clean.startsWith(".") ? relativePath(filePath, clean) : clean.startsWith("$lib/") ? `src/lib/${clean.slice("$lib/".length)}` : clean.replace(/^\//, "");
  if (tracked.has(resolved)) return resolved;
  return [...tracked].find((item) => item.endsWith(`/${resolved}`))?.replaceAll("\\", "/");
}

function globAuthorities(candidate, filePath, tracked) {
  const clean = candidate.replace(/\?.*$/, "").replaceAll("\\", "/");
  const pattern = clean.startsWith(".") ? relativePath(filePath, clean) : clean.startsWith("$lib/") ? `src/lib/${clean.slice("$lib/".length)}` : clean;
  const expression = `^${pattern.split("**").map((part) => part.split("*").map((piece) => piece.replace(/[|\\{}()[\]^$+?.]/g, "\\$&")).join("[^/]*")).join("(?:.+/)?")}$`;
  const matcher = new RegExp(expression);
  return [...tracked].filter((item) => matcher.test(item)).sort();
}

function authorityClassification(authorityPath) {
  if (!authorityPath || /(^|\/)(node_modules|vendor|dist|build)(\/|$)/.test(authorityPath)) return "non-production";
  if (/\.(test|spec)\.[cm]?[jt]sx?$/i.test(authorityPath)) return "non-production";
  if (/(^|\/)(__fixtures__|fixtures?|testdata|artifacts)(\/|$)/i.test(authorityPath)) return "fixture";
  return "production";
}

function sourceSymbols(file, readers, typescript) {
  const bindings = new Map();
  const seeds = new Set(readers.filter((reader) => reader.path === file.fileName || reader.path === file.fileName.replaceAll("\\", "/")).flatMap((reader) => reader.symbolName ? [reader.symbolName] : []));
  const seedOffsets = new Set(readers.filter((reader) => reader.path === file.fileName || reader.path === file.fileName.replaceAll("\\", "/")).map((reader) => reader.sourceOffset));
  const identifiers = (node) => {
    const names = new Set(); const visit = (child) => { if (typescript.isIdentifier(child)) names.add(child.text); typescript.forEachChild(child, visit); }; visit(node); return names;
  };
  const visit = (node) => {
    if (typescript.isVariableDeclaration(node) && typescript.isIdentifier(node.name) && node.initializer) {
      bindings.set(node.name.text, { body: node.initializer, offset: node.initializer.getStart(file) });
      if ([...seedOffsets].some((offset) => offset >= node.initializer.getStart(file) && offset < node.initializer.getEnd())) seeds.add(node.name.text);
    }
    if (typescript.isFunctionDeclaration(node) && node.name && node.body) {
      bindings.set(node.name.text, { body: node.body, offset: node.body.getStart(file) });
      if ([...seedOffsets].some((offset) => offset >= node.body.getStart(file) && offset < node.body.getEnd())) seeds.add(node.name.text);
    }
    typescript.forEachChild(node, visit);
  };
  visit(file);
  let changed = true;
  while (changed) {
    changed = false;
    for (const [name, binding] of bindings) if (!seeds.has(name) && [...identifiers(binding.body)].some((item) => seeds.has(item))) { seeds.add(name); changed = true; }
  }
  return seeds;
}

export function discoverSourceReaders(programOrFiles, trackedPaths, typescript = tsDefault) {
  const sourceFiles = Array.isArray(programOrFiles) ? programOrFiles : programOrFiles?.getSourceFiles?.().filter((file) => !file.isDeclarationFile).map((file) => ({ path: file.fileName, source: file.getFullText() })) ?? [];
  const tracked = new Set([...trackedPaths].map((item) => String(item).replaceAll("\\", "/")));
  const readers = [];
  for (const entry of normalFiles(sourceFiles, typescript)) {
    const imports = new Map();
    const visit = (node) => {
      if (typescript.isImportDeclaration(node) && node.importClause && typescript.isStringLiteral(node.moduleSpecifier)) {
        const moduleName = node.moduleSpecifier.text;
        const binding = node.importClause.name?.text;
        const helper = HELPER_MODULES.has(path.posix.basename(moduleName).replace(/\.ts$/, ""));
        if (binding && moduleName.includes("?raw")) {
          const authorityPath = trackedAuthority(moduleName, entry.path, tracked);
          if (authorityPath) readers.push({ path: entry.path, sourceRange: sourceRange(entry.file, node), sourceOffset: node.getStart(entry.file), kind: "raw-import", authorityPath, classification: authorityClassification(authorityPath), symbolName: binding });
        }
        if (helper) {
          const bindings = node.importClause.namedBindings;
          if (typescript.isNamespaceImport(bindings)) readers.push({ path: entry.path, sourceRange: sourceRange(entry.file, bindings), sourceOffset: bindings.getStart(entry.file), kind: "contract-path-helper", authorityPath: moduleName, authorityText: moduleName, classification: "production", symbolName: bindings.name.text });
          else for (const specifier of bindings?.elements ?? []) readers.push({ path: entry.path, sourceRange: sourceRange(entry.file, specifier), sourceOffset: specifier.getStart(entry.file), kind: "contract-path-helper", authorityPath: moduleName, authorityText: moduleName, classification: "production", symbolName: specifier.name.text });
        }
        if (["node:fs", "fs", "node:fs/promises", "fs/promises"].includes(moduleName)) for (const specifier of node.importClause.namedBindings?.elements ?? []) imports.set(specifier.name.text, "fs");
      }
      if (typescript.isCallExpression(node)) {
        const expression = node.expression;
        const fsRead = typescript.isIdentifier(expression) && /^(readFile|readFileSync|readdir|readdirSync)$/.test(expression.text);
        if (fsRead) {
          const candidate = staticPath(node.arguments[0], entry.file, typescript);
          const authorityPath = trackedAuthority(candidate, entry.path, tracked);
          if (authorityPath) readers.push({ path: entry.path, sourceRange: sourceRange(entry.file, node), sourceOffset: node.getStart(entry.file), kind: "fs-read", authorityPath, classification: authorityClassification(authorityPath), symbolName: undefined });
          else if (/^readdir/.test(expression.text) && candidate !== undefined) for (const trackedPath of tracked) if (trackedPath.startsWith(`${candidate.replace(/\/$/, "")}/`)) readers.push({ path: entry.path, sourceRange: sourceRange(entry.file, node), sourceOffset: node.getStart(entry.file), kind: "fs-directory-read", authorityPath: trackedPath, classification: authorityClassification(trackedPath), symbolName: undefined });
          else if (candidate === undefined && !/(tmpdir|mkdtemp|artifacts\/|artifacts\\|runner.*result|generated)/i.test(node.arguments[0]?.getText(entry.file) ?? "")) readers.push({ path: entry.path, sourceRange: sourceRange(entry.file, node), sourceOffset: node.getStart(entry.file), kind: "manual", reason: "unknown source-reader wrapper or dynamic path" });
        }
        if (typescript.isPropertyAccessExpression(expression) && expression.name.text === "glob" && expression.expression.getText(entry.file) === "import.meta") {
          const candidate = staticPath(node.arguments[0], entry.file, typescript);
          const raw = /query\s*:\s*["']\?raw["']/.test(node.arguments[1]?.getText(entry.file) ?? "");
          if (candidate && raw && !/[?*[{]/.test(candidate)) {
            const authorityPath = trackedAuthority(candidate, entry.path, tracked);
            if (authorityPath) readers.push({ path: entry.path, sourceRange: sourceRange(entry.file, node), sourceOffset: node.getStart(entry.file), kind: "import-meta-glob", authorityPath, classification: authorityClassification(authorityPath) });
          }
          if (candidate && raw && /[*?[{]/.test(candidate)) for (const authorityPath of globAuthorities(candidate, entry.path, tracked)) readers.push({ path: entry.path, sourceRange: sourceRange(entry.file, node), sourceOffset: node.getStart(entry.file), kind: "import-meta-glob", authorityPath, classification: authorityClassification(authorityPath) });
          if (raw && candidate === undefined) readers.push({ path: entry.path, sourceRange: sourceRange(entry.file, node), sourceOffset: node.getStart(entry.file), kind: "manual", reason: "dynamic raw glob" });
          if (raw && candidate && /[*?[{]/.test(candidate) && !globAuthorities(candidate, entry.path, tracked).length) readers.push({ path: entry.path, sourceRange: sourceRange(entry.file, node), sourceOffset: node.getStart(entry.file), kind: "manual", reason: "raw glob resolved no tracked authority" });
        }
      }
      typescript.forEachChild(node, visit);
    };
    visit(entry.file);
    if (/\b(?:readFile|readFileSync|readdir|readdirSync)\s*\(\s*[^"'`]/.test(entry.source) && !readers.some((reader) => reader.path === entry.path && reader.kind === "manual" && /dynamic path/.test(reader.reason))) {
      readers.push({ path: entry.path, sourceRange: "1:1-1:1", sourceOffset: 0, kind: "manual", reason: "unknown source-reader wrapper or dynamic path" });
    }
    const symbols = sourceSymbols(entry.file, readers, typescript);
    for (const reader of readers) if (reader.path === entry.path) reader.symbolNames = [...symbols].sort();
  }
  return readers.sort((a, b) => a.path.localeCompare(b.path) || a.sourceRange.localeCompare(b.sourceRange));
}

function draftRow(declaration, index) {
  const row = { id: `SC-${String(index + 1).padStart(6, "0")}`, path: declaration.path, title: declaration.title, sourceHash: sha256(declaration.sourceSlice), assertionCount: declaration.assertionOrdinals.length, lineage: [], invariant: "Review this source-contract assertion and preserve its user-observable invariant.", disposition: "behavior", replacementIds: [] };
  if (declaration.eachAuthorityText) row.authorityHash = sha256(declaration.eachAuthorityText);
  return row;
}

export function buildLedgerDraft({ declarations = [], sourceReaders, frozenAtCommit, outputPath, repoRoot = process.cwd() } = {}) {
  if (!/^[a-f0-9]{40}$/i.test(frozenAtCommit ?? "")) throw new Error("frozenAtCommit must be a full commit SHA");
  const artifactRoot = path.resolve(repoRoot, "artifacts");
  const resolvedOutput = outputPath === undefined ? undefined : path.resolve(repoRoot, outputPath);
  if (resolvedOutput !== undefined && path.relative(artifactRoot, resolvedOutput).startsWith("..")) throw new Error("ledger drafts may only be written inside artifacts/");
  let selected = declarations;
  if (sourceReaders) {
    const symbols = new Map();
    const offsets = new Map();
    for (const reader of sourceReaders) if (reader.classification !== "fixture" && reader.classification !== "non-production") {
      for (const symbol of reader.symbolNames ?? (reader.symbolName ? [reader.symbolName] : [])) symbols.set(`${reader.path}:${symbol}`, true);
      if (Number.isInteger(reader.sourceOffset)) offsets.set(`${reader.path}:${reader.sourceOffset}`, true);
    }
    selected = declarations.filter((declaration) => declaration.referencedSymbols.some((symbol) => symbols.has(`${declaration.path}:${symbol}`)) || [...offsets.keys()].some((key) => {
      const [readerPath, readerOffset] = key.split(":");
      return declaration.path === readerPath && Number(readerOffset) >= declaration.sourceOffset && Number(readerOffset) < declaration.sourceOffset + declaration.sourceSlice.length;
    }));
  }
  const manual = [...(declarations.manualRequirements ?? []), ...(sourceReaders ?? []).filter((reader) => reader.kind === "manual").map((reader) => ({ path: reader.path, sourceRange: reader.sourceRange, sourceOffset: reader.sourceOffset ?? 0, reason: reader.reason, runnerTitles: reader.runnerTitles ?? [] }))];
  const rows = [...selected, ...manual].sort((a, b) => a.path < b.path ? -1 : a.path > b.path ? 1 : (a.sourceOffset ?? 0) - (b.sourceOffset ?? 0)).map((item, index) => item.title !== undefined ? draftRow(item, index) : ({ id: `SC-${String(index + 1).padStart(6, "0")}`, path: item.path, manual: { sourceRange: item.sourceRange, reason: item.reason, runnerTitles: item.runnerTitles ?? [] }, sourceHash: sha256(item.sourceSlice ?? ""), assertionCount: item.assertionCount ?? 0, lineage: [], invariant: "Review this unsupported source-contract declaration manually.", disposition: "behavior", replacementIds: [] }));
  const draft = { schemaVersion: 1, frozenAtCommit, sourceReaderExceptions: [], rows };
  if (resolvedOutput) {
    mkdirSync(path.dirname(resolvedOutput), { recursive: true });
    writeFileSync(resolvedOutput, `${JSON.stringify(draft, null, 2)}\n`, "utf8");
  }
  return draft;
}

function identity(value) { return value.manual ? `${value.path}@${value.manual.sourceRange}` : `${value.path}#${value.title}`; }
function replacementResolved(id, context) {
  if (!id.startsWith("test:vitest:")) return false;
  const target = id.slice("test:vitest:".length);
  const hash = target.indexOf("#");
  if (hash < 1) return false;
  const filePath = target.slice(0, hash); const title = target.slice(hash + 1);
  const owner = (context.liveCensus?.vitestOwners ?? []).find((entry) => {
    const files = entry.files ?? context.liveCensus?.vitestFiles?.[entry.id] ?? context.liveCensus?.vitestOwnership?.[filePath] ?? [];
    return Array.isArray(files) ? files.includes(filePath) : files === entry.id || files === filePath;
  });
  return Boolean(owner && context.verifySteps?.some((step) => step.npmScript === owner.ownerScript) && context.declarations.some((declaration) => declaration.path === filePath && declaration.title === title));
}

function resolutionIssues(row, resolution, assertionCount, context) {
  const issues = []; const prefix = `${row.id}:`;
  if (!DISPOSITIONS.has(resolution.disposition)) issues.push(`${prefix} invalid disposition`);
  const replacements = resolution.replacementIds;
  if (resolution.disposition === "delete") {
    if ("replacementIds" in resolution) issues.push(`${prefix} delete must not contain replacementIds`);
    if (typeof resolution.deletionReason !== "string" || !resolution.deletionReason.trim()) issues.push(`${prefix} delete requires a specific deletionReason`);
    return { issues, closed: !issues.length };
  }
  if ("deletionReason" in resolution) issues.push(`${prefix} non-delete must not contain deletionReason`);
  if (!Array.isArray(replacements) || !replacements.length) issues.push(`${prefix} missing replacementIds`);
  else for (const id of replacements) if (typeof id !== "string" || !REPLACEMENT.test(id)) issues.push(`${prefix} unknown replacement namespace: ${id}`);
  return { issues, closed: !issues.length && replacements.every((id) => replacementResolved(id, context)) };
}

function fieldsIssue(value, allowed, label, issues) {
  for (const field of Object.keys(value ?? {})) if (!allowed.has(field)) issues.push(`${value?.id ?? "unknown"}: unknown ${label} field: ${field}`);
}

function validateRowShape(row, issues) {
  const simple = new Set(["id", "path", "title", "manual", "sourceHash", "assertionCount", "authorityHash", "lineage", "invariant", "disposition", "replacementIds", "deletionReason"]);
  const mixed = new Set(["id", "path", "title", "manual", "sourceHash", "assertionCount", "authorityHash", "lineage", "invariant", "subgroups"]);
  const isMixed = Array.isArray(row.subgroups);
  fieldsIssue(row, isMixed ? mixed : simple, "ledger row", issues);
  if (typeof row.path !== "string" || !row.path || row.path.includes("\\") || row.path.split("/").some((part) => !part || part === "." || part === "..")) issues.push(`${row.id}: invalid path`);
  if (typeof row.sourceHash !== "string" || !/^[a-f0-9]{64}$/i.test(row.sourceHash)) issues.push(`${row.id}: invalid sourceHash`);
  if (!Number.isInteger(row.assertionCount) || row.assertionCount < 0) issues.push(`${row.id}: invalid assertionCount`);
  if (row.authorityHash !== undefined && (typeof row.authorityHash !== "string" || !/^[a-f0-9]{64}$/i.test(row.authorityHash))) issues.push(`${row.id}: invalid authorityHash`);
  if (typeof row.invariant !== "string" || !row.invariant.trim()) issues.push(`${row.id}: missing invariant`);
  if (Boolean(row.title) === Boolean(row.manual)) issues.push(`${row.id}: row requires exactly one title or manual`);
  if (row.title !== undefined && (typeof row.title !== "string" || !row.title.trim())) issues.push(`${row.id}: invalid title`);
  if (row.manual !== undefined && (!row.manual || Object.keys(row.manual).some((field) => !new Set(["sourceRange", "reason", "runnerTitles"]).has(field)) || typeof row.manual.sourceRange !== "string" || !row.manual.sourceRange || typeof row.manual.reason !== "string" || !row.manual.reason || !Array.isArray(row.manual.runnerTitles) || row.manual.runnerTitles.some((title) => typeof title !== "string" || !title))) issues.push(`${row.id}: invalid manual row`);
  if (isMixed) {
    if (!row.subgroups.length) issues.push(`${row.id}: mixed row requires subgroups`);
    for (const subgroup of row.subgroups) {
      fieldsIssue(subgroup, new Set(["assertionOrdinals", "invariant", "disposition", "replacementIds", "deletionReason"]), "subgroup", issues);
      if (typeof subgroup.invariant !== "string" || !subgroup.invariant.trim()) issues.push(`${row.id}: subgroup missing invariant`);
      for (const field of Object.keys(subgroup ?? {})) if (LIFECYCLE_FIELDS.has(field)) issues.push(`${row.id}: stored lifecycle field: ${field}`);
    }
  }
}

export function validateSourceContractLedger(context = {}) {
  const { ledger, declarations = [], sourceReaders = [] } = context;
  const issues = []; const states = [];
  if (!ledger || typeof ledger !== "object") return { issues: ["invalid source-contract ledger envelope"], rows: states };
  fieldsIssue(ledger, new Set(["schemaVersion", "frozenAtCommit", "sourceReaderExceptions", "rows"]), "ledger envelope", issues);
  if (ledger.schemaVersion !== 1 || !Array.isArray(ledger.rows) || !Array.isArray(ledger.sourceReaderExceptions) || !/^[a-f0-9]{40}$/i.test(ledger.frozenAtCommit ?? "")) return { issues: [...issues, "invalid source-contract ledger envelope"], rows: states };
  const ids = new Set(); const rowsByIdentity = new Map();
  for (const row of ledger.rows) {
    if (!row || typeof row !== "object") { issues.push("invalid ledger row"); continue; }
    validateRowShape(row, issues);
    for (const field of Object.keys(row)) if (LIFECYCLE_FIELDS.has(field)) issues.push(`${row.id ?? "unknown"}: stored lifecycle field: ${field}`);
    if (!/^SC-\d{6}$/.test(row.id ?? "") || ids.has(row.id)) issues.push(`duplicate or invalid ledger id: ${row.id}`); ids.add(row.id);
    if (!Array.isArray(row.lineage) || row.lineage.some((item) => typeof item !== "string" || !item || item === row.path)) issues.push(`${row.id}: invalid lineage`);
    const key = identity(row); if (rowsByIdentity.has(key)) issues.push(`duplicate current identity: ${key}`); rowsByIdentity.set(key, row);
  }
  const current = new Map();
  for (const declaration of declarations) {
    const key = identity(declaration);
    if (current.has(key)) issues.push(`duplicate current identity: ${key}`);
    else current.set(key, declaration);
  }
  for (const [key, declaration] of current) {
    const row = rowsByIdentity.get(key);
    if (!row) { issues.push(`missing ledger row: ${key}`); continue; }
    if (row.sourceHash !== sha256(declaration.sourceSlice)) issues.push(`${row.id}: sourceHash drift`);
    if (row.assertionCount !== declaration.assertionOrdinals.length) issues.push(`${row.id}: assertionCount drift`);
    if (declaration.eachAuthorityText && row.authorityHash !== sha256(declaration.eachAuthorityText)) issues.push(`${row.id}: authorityHash drift`);
  }
  for (const row of ledger.rows) {
    const isCurrent = current.has(identity(row));
    let resolved;
    if (Array.isArray(row.subgroups)) {
      if (row.disposition || row.replacementIds || row.deletionReason) issues.push(`${row.id}: mixed row has top-level resolution`);
      if (row.assertionCount < 2) issues.push(`${row.id}: unnecessary subgroups`);
      const owned = new Set(); const subgroupStates = [];
      for (const subgroup of row.subgroups) {
        if (!Array.isArray(subgroup.assertionOrdinals) || !subgroup.assertionOrdinals.length) issues.push(`${row.id}: subgroup missing assertion ordinals`);
        for (const ordinal of subgroup.assertionOrdinals ?? []) {
          if (!Number.isInteger(ordinal) || ordinal < 1 || ordinal > row.assertionCount) issues.push(`${row.id}: invalid subgroup assertion ordinal: ${ordinal}`);
          else if (owned.has(ordinal)) issues.push(`${row.id}: overlapping subgroup assertion ordinal: ${ordinal}`);
          else owned.add(ordinal);
        }
        const result = resolutionIssues(row, subgroup, row.assertionCount, context); issues.push(...result.issues); subgroupStates.push(result.closed);
      }
      for (let ordinal = 1; ordinal <= row.assertionCount; ordinal++) if (!owned.has(ordinal)) issues.push(`${row.id}: incomplete subgroup assertion ordinals`);
      resolved = subgroupStates.length > 0 && subgroupStates.every(Boolean) && !issues.some((issue) => issue.startsWith(`${row.id}:`));
    } else {
      resolved = resolutionIssues(row, row, row.assertionCount, context); issues.push(...resolved.issues); resolved = resolved.closed;
    }
    if (!isCurrent && !resolved) issues.push(`${row.id}: unresolved historical row`);
    states.push({ id: row.id, state: isCurrent ? "open" : resolved ? "closed" : "open" });
  }
  const exceptionKeys = new Set();
  for (const exception of ledger.sourceReaderExceptions) {
    const key = `${exception?.path}:${exception?.sourceRange}`;
    if (!exception || typeof exception.path !== "string" || /[*?[{]/.test(exception.path) || typeof exception.sourceRange !== "string" || typeof exception.reason !== "string" || typeof exception.owner !== "string") issues.push("invalid sourceReaderException");
    if (exceptionKeys.has(key)) issues.push(`duplicate sourceReaderException: ${key}`); exceptionKeys.add(key);
  }
  for (const reader of sourceReaders) {
    const key = `${reader.path}:${reader.sourceRange}`;
    if (reader.kind === "manual") issues.push(`manual source-reader requirement: ${key}: ${reader.reason}`);
    if (reader.classification === "fixture" && !exceptionKeys.has(key)) issues.push(`missing sourceReaderException: ${key}`);
  }
  for (const key of exceptionKeys) if (!sourceReaders.some((reader) => `${reader.path}:${reader.sourceRange}` === key && reader.classification === "fixture")) issues.push(`stale sourceReaderException: ${key}`);
  return { issues: [...new Set(issues)].sort(), rows: states };
}

async function main() {
  const outputIndex = process.argv.indexOf("--output");
  if (outputIndex < 0 || !process.argv[outputIndex + 1]) throw new Error("Use --output artifacts/.../source-contract-ledger.draft.json");
  const repoRoot = path.resolve(fileURLToPath(new URL("../..", import.meta.url)));
  const tracked = execFileSync("git", ["ls-files"], { cwd: repoRoot, encoding: "utf8" }).split(/\r?\n/).filter(Boolean);
  const tests = tracked.filter((item) => /\.(test|spec)\.[cm]?[jt]sx?$/i.test(item)).map((item) => ({ path: item, source: readFileSync(path.join(repoRoot, item), "utf8") }));
  const declarations = discoverTestDeclarations(tests, tsDefault);
  const readers = discoverSourceReaders(tests, new Set(tracked), tsDefault);
  const draft = buildLedgerDraft({ declarations, sourceReaders: readers, frozenAtCommit: execFileSync("git", ["rev-parse", "HEAD"], { cwd: repoRoot, encoding: "utf8" }).trim(), outputPath: process.argv[outputIndex + 1] });
  console.log(`Source-contract ledger draft: ${draft.rows.length} rows, ${declarations.manualRequirements.length} manual requirements, ${readers.length} reader sites`);
}

if (process.argv[1] && import.meta.url === new URL(`file:///${process.argv[1].replaceAll("\\", "/")}`).href) main().catch((error) => { console.error(error.message); process.exitCode = 1; });
