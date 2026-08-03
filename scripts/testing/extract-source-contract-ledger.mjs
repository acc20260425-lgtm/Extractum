import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import tsDefault from "typescript";

const TEST_NAMES = new Set(["it", "test"]);
const SUITE_NAMES = new Set(["describe", "suite"]);
const DISPOSITIONS = new Set(["behavior", "architecture", "tool_owned", "delete"]);
const LIFECYCLE_FIELDS = new Set(["status", "state", "pending", "dualRun", "retired", "timestamp", "timestamps", "duration", "durations", "counter", "counters"]);
const SOURCE_HELPERS = new Map([
  ["analysis-contract-paths", new Set(["readAppAnalysisSource", "readCrateAnalysisSource", "readAnalysisContractSource"])],
  ["telegram-contract-paths", new Set(["readTelegramContractFile", "readTelegramContentAddressedSection"])],
  ["prompt-pack-contract-paths", new Set(["readPromptPackDomainSource", "readPromptPackAppFacade"])],
]);
const FS_MODULES = new Set(["node:fs", "fs", "node:fs/promises", "fs/promises"]);
const FS_READS = new Set(["readFile", "readFileSync", "readdir", "readdirSync"]);
const EXACT_RANGE = /^(\d+):(\d+)-(\d+):(\d+)$/;
const TEST_FILE = /\.(?:test|spec)\.[cm]?[jt]sx?$/i;
const PATH_KINDS = new Set(["production", "configuration", "documentation", "fixture", "generated", "output"]);
const OBLIGATION_KINDS = new Set(["production", "configuration", "documentation"]);
const REPLACEMENT = /^(?:test:vitest:[^#\s]+#[^\r\n]+|test:playwright:[^\s]+|test:cargo:[^\s]+|rule:[A-Za-z0-9][A-Za-z0-9:./_-]*|tool:[A-Za-z0-9][A-Za-z0-9:./_-]*)$/;

const normalizeText = (text) => String(text ?? "").replace(/\r\n?/g, "\n");
const sha256 = (text) => createHash("sha256").update(normalizeText(text)).digest("hex");
const compareText = (a, b) => a < b ? -1 : a > b ? 1 : 0;
const normalizePath = (value) => String(value ?? "").replaceAll("\\", "/").replace(/^\.\//, "");

function rangeOf(file, node) {
  const start = file.getLineAndCharacterOfPosition(node.getStart(file));
  const end = file.getLineAndCharacterOfPosition(node.getEnd());
  return `${start.line + 1}:${start.character + 1}-${end.line + 1}:${end.character + 1}`;
}

function isExactRange(value) {
  const match = typeof value === "string" ? EXACT_RANGE.exec(value) : undefined;
  if (!match) return false;
  const [, sl, sc, el, ec] = match.map(Number);
  return sl > 0 && sc > 0 && el > 0 && ec > 0 && (el > sl || (el === sl && ec > sc));
}

function isNormalizedRepoPath(value) {
  return typeof value === "string"
    && value.length > 0
    && !value.includes("\\")
    && !/[*?[{]/.test(value)
    && !path.posix.isAbsolute(value)
    && value.split("/").every((part) => part && part !== "." && part !== "..");
}

function sourceEntries(sourceFiles, typescript) {
  return (sourceFiles ?? []).map((entry) => {
    if (entry?.kind !== undefined && typeof entry.getFullText === "function") {
      return { path: normalizePath(entry.fileName), source: entry.getFullText(), file: entry };
    }
    const filePath = normalizePath(entry?.path ?? entry?.fileName);
    const source = String(entry?.source ?? entry?.text ?? "");
    return { path: filePath, source, file: typescript.createSourceFile(filePath, source, typescript.ScriptTarget.Latest, true) };
  }).sort((a, b) => compareText(a.path, b.path));
}

function helperModuleName(moduleName) {
  const base = path.posix.basename(moduleName).replace(/\.[cm]?[jt]sx?$/, "");
  return SOURCE_HELPERS.has(base) ? base : undefined;
}

function createLexicalModel(entry, typescript) {
  let nextScope = 0;
  const scopeByNode = new WeakMap();
  const bindings = new Map();
  const makeScope = (parent) => ({ id: nextScope++, parent, bindings: new Map() });
  const root = makeScope(undefined);

  const addBinding = (scope, name, node, analysisNode, role = {}) => {
    const key = `${entry.path}:${node.getStart(entry.file)}:${name}`;
    const binding = { key, name, node, analysisNode, role, scope };
    const sameName = scope.bindings.get(name) ?? [];
    sameName.push(binding);
    scope.bindings.set(name, sameName);
    bindings.set(key, binding);
    return binding;
  };

  const addImportBindings = (node, scope) => {
    if (!node.importClause || !typescript.isStringLiteral(node.moduleSpecifier)) return;
    const moduleName = node.moduleSpecifier.text;
    const helper = helperModuleName(moduleName);
    const roleFor = (kind, importedName) => {
      if (moduleName.includes("?raw")) return { kind: "raw", moduleName };
      if (FS_MODULES.has(moduleName)) return { kind: `fs-${kind}`, moduleName, importedName };
      if (["node:assert", "node:assert/strict", "assert", "assert/strict"].includes(moduleName)) return { kind: "node-assert", moduleName, importedName };
      if (["node:path", "path"].includes(moduleName)) return { kind: `path-${kind}`, moduleName, importedName };
      if (["node:os", "os"].includes(moduleName)) return { kind: `os-${kind}`, moduleName, importedName };
      if (helper) return { kind: `helper-${kind}`, moduleName, helper, importedName };
      return { kind: `import-${kind}`, moduleName, importedName };
    };
    if (node.importClause.name) addBinding(scope, node.importClause.name.text, node.importClause.name, undefined, roleFor("default", "default"));
    const named = node.importClause.namedBindings;
    if (named && typescript.isNamespaceImport(named)) addBinding(scope, named.name.text, named.name, undefined, roleFor("namespace", "*"));
    if (named && typescript.isNamedImports(named)) {
      for (const specifier of named.elements) {
        addBinding(scope, specifier.name.text, specifier.name, undefined, roleFor("named", specifier.propertyName?.text ?? specifier.name.text));
      }
    }
  };

  const visit = (node, scope) => {
    scopeByNode.set(node, scope);
    if (typescript.isImportDeclaration(node)) {
      addImportBindings(node, scope);
      typescript.forEachChild(node, (child) => visit(child, scope));
      return;
    }
    if (typescript.isFunctionDeclaration(node)) {
      if (node.name) addBinding(scope, node.name.text, node.name, node.body);
      const functionScope = makeScope(scope);
      scopeByNode.set(node, functionScope);
      for (const parameter of node.parameters) {
        scopeByNode.set(parameter, functionScope);
        if (typescript.isIdentifier(parameter.name)) addBinding(functionScope, parameter.name.text, parameter.name, parameter.initializer);
      }
      if (node.body) visit(node.body, functionScope);
      return;
    }
    if (typescript.isFunctionExpression(node) || typescript.isArrowFunction(node)) {
      const functionScope = makeScope(scope);
      scopeByNode.set(node, functionScope);
      if (node.name) addBinding(functionScope, node.name.text, node.name, node.body);
      for (const parameter of node.parameters) {
        scopeByNode.set(parameter, functionScope);
        if (typescript.isIdentifier(parameter.name)) addBinding(functionScope, parameter.name.text, parameter.name, parameter.initializer);
      }
      visit(node.body, functionScope);
      return;
    }
    if (typescript.isBlock(node) && !typescript.isSourceFile(node)) {
      const blockScope = makeScope(scope);
      scopeByNode.set(node, blockScope);
      typescript.forEachChild(node, (child) => visit(child, blockScope));
      return;
    }
    if (typescript.isVariableDeclaration(node)) {
      if (typescript.isIdentifier(node.name)) addBinding(scope, node.name.text, node.name, node.initializer);
      else {
        const addPattern = (pattern) => {
          if (typescript.isIdentifier(pattern)) addBinding(scope, pattern.text, pattern, node.initializer, { kind: "unsupported-binding" });
          else typescript.forEachChild(pattern, addPattern);
        };
        addPattern(node.name);
      }
    }
    typescript.forEachChild(node, (child) => visit(child, scope));
  };
  visit(entry.file, root);

  const resolve = (identifier) => {
    let scope = scopeByNode.get(identifier) ?? root;
    while (scope) {
      const found = scope.bindings.get(identifier.text);
      if (found?.length) return found;
      scope = scope.parent;
    }
    return [];
  };
  return { ...entry, bindings, resolve, scopeByNode, typescript };
}

function isReferenceIdentifier(node, typescript) {
  const parent = node.parent;
  if (!parent) return true;
  if ((typescript.isVariableDeclaration(parent) || typescript.isParameter(parent) || typescript.isFunctionDeclaration(parent) || typescript.isFunctionExpression(parent)) && parent.name === node) return false;
  if (typescript.isImportClause(parent) || typescript.isImportSpecifier(parent) || typescript.isNamespaceImport(parent)) return false;
  if (typescript.isPropertyAccessExpression(parent) && parent.name === node) return false;
  if (typescript.isPropertyAssignment(parent) && parent.name === node) return false;
  if (typescript.isTypeReferenceNode(parent) || typescript.isTypeAliasDeclaration(parent) || typescript.isInterfaceDeclaration(parent)) return false;
  return true;
}

function bindingClosure(node, model) {
  const keys = new Set();
  const ambiguous = new Set();
  const visitedBindings = new Set();
  const visitBinding = (binding) => {
    if (!binding || visitedBindings.has(binding.key)) return;
    visitedBindings.add(binding.key);
    keys.add(binding.key);
    if (binding.analysisNode) visit(binding.analysisNode);
  };
  const visit = (child) => {
    if (model.typescript.isIdentifier(child) && isReferenceIdentifier(child, model.typescript)) {
      const resolved = model.resolve(child);
      if (resolved.length > 1) {
        ambiguous.add(child.text);
        for (const binding of resolved) visitBinding(binding);
      } else if (resolved.length === 1) visitBinding(resolved[0]);
    }
    model.typescript.forEachChild(child, visit);
  };
  if (node) visit(node);
  return { keys: [...keys].sort(compareText), ambiguous: [...ambiguous].sort(compareText) };
}

function rootIdentifier(node, typescript) {
  let current = node;
  while (typescript.isPropertyAccessExpression(current)) current = current.expression;
  return typescript.isIdentifier(current) ? current : undefined;
}

function assertionInfo(node, model) {
  const ordinals = [];
  const unknownDirectAssert = [];
  const visit = (child) => {
    if (model.typescript.isCallExpression(child)) {
      const expression = child.expression;
      let assertion = model.typescript.isIdentifier(expression) && expression.text === "expect";
      if (model.typescript.isIdentifier(expression) && expression.text === "assert") {
        const resolved = model.resolve(expression);
        if (resolved.length === 1 && resolved[0].role.kind === "node-assert") assertion = true;
        else if (resolved.length !== 1) unknownDirectAssert.push(rangeOf(model.file, child));
      }
      if (model.typescript.isIdentifier(expression)) {
        const resolved = model.resolve(expression);
        if (resolved.length === 1 && resolved[0].role.kind === "node-assert" && resolved[0].role.importedName !== "default") assertion = true;
      }
      if (model.typescript.isPropertyAccessExpression(expression)) {
        const root = rootIdentifier(expression, model.typescript);
        const resolved = root ? model.resolve(root) : [];
        if (resolved.length === 1 && resolved[0].role.kind === "node-assert") assertion = true;
      }
      if (assertion) ordinals.push(ordinals.length + 1);
    }
    model.typescript.forEachChild(child, visit);
  };
  if (node) visit(node);
  return { ordinals, unknownDirectAssert };
}

function staticText(node, typescript) {
  return typescript.isStringLiteral(node) || typescript.isNoSubstitutionTemplateLiteral(node) ? node.text : undefined;
}

function unwrapExpression(node, typescript) {
  let current = node;
  while (current && (typescript.isAsExpression(current) || typescript.isSatisfiesExpression(current) || typescript.isParenthesizedExpression(current) || typescript.isAwaitExpression(current))) current = current.expression;
  return current;
}

function callKind(node, typescript) {
  if (!typescript.isCallExpression(node)) return undefined;
  const callbackFor = (kind, args) => SUITE_NAMES.has(kind) && args.length >= 3
    && !(typescript.isArrowFunction(args[1]) || typescript.isFunctionExpression(args[1]))
    ? args[2]
    : args[1];
  if (typescript.isIdentifier(node.expression)) {
    const kind = node.expression.text;
    return { kind, args: node.arguments, callback: callbackFor(kind, node.arguments) };
  }
  if (!typescript.isCallExpression(node.expression)) return undefined;
  const each = node.expression;
  if (!typescript.isPropertyAccessExpression(each.expression) || !typescript.isIdentifier(each.expression.expression)) return undefined;
  const kind = each.expression.expression.text;
  if (each.expression.name.text === "each") return { kind, args: node.arguments, callback: node.arguments[1], eachTable: each.arguments[0] };
  if (["runIf", "skipIf"].includes(each.expression.name.text)) return { kind, args: node.arguments, callback: callbackFor(kind, node.arguments) };
  return undefined;
}

function referencedSymbols(node, model) {
  const names = new Set();
  const visit = (child) => {
    if (model.typescript.isIdentifier(child) && isReferenceIdentifier(child, model.typescript)
      && !["expect", "it", "test", "describe", "suite"].includes(child.text)) names.add(child.text);
    model.typescript.forEachChild(child, visit);
  };
  if (node) visit(node);
  return [...names].sort(compareText);
}

function manualRequirement(model, node, reason, analysisNode = node, title, titleExact = true, titleTemplate = false) {
  const assertions = assertionInfo(analysisNode, model);
  const closure = bindingClosure(analysisNode, model);
  return {
    path: model.path,
    sourceRange: rangeOf(model.file, node),
    sourceSlice: normalizeText(node.getText(model.file)),
    sourceOffset: node.getStart(model.file),
    sourceEnd: node.getEnd(),
    assertionOrdinals: assertions.ordinals,
    referencedBindingKeys: closure.keys,
    referencedSymbols: referencedSymbols(analysisNode, model),
    ...(title ? { title, titleExact } : {}),
    ...(titleTemplate ? { titleTemplate: true } : {}),
    reason,
  };
}

function insideNamedRegistration(node, typescript) {
  let current = node.parent;
  while (current) {
    if (typescript.isFunctionDeclaration(current) && current.name) return true;
    if ((typescript.isArrowFunction(current) || typescript.isFunctionExpression(current))
      && typescript.isVariableDeclaration(current.parent)
      && typescript.isIdentifier(current.parent.name)) return true;
    current = current.parent;
  }
  return false;
}

export function discoverTestDeclarations(sourceFiles, typescript = tsDefault) {
  const declarations = [];
  const manualRequirements = [];
  for (const entry of sourceEntries(sourceFiles, typescript)) {
    const model = createLexicalModel(entry, typescript);
    const visit = (node, parentTitles = []) => {
      const call = callKind(node, typescript);
      if (call && SUITE_NAMES.has(call.kind)) {
        const title = staticText(call.args[0], typescript);
        if (title === undefined) manualRequirements.push(manualRequirement(model, node, "computed suite title"));
        else if (!call.callback || !(typescript.isArrowFunction(call.callback) || typescript.isFunctionExpression(call.callback))) manualRequirements.push(manualRequirement(model, node, "dynamic suite factory"));
        else typescript.forEachChild(call.callback.body, (child) => visit(child, [...parentTitles, title]));
        return;
      }
      if (call && TEST_NAMES.has(call.kind)) {
        const title = staticText(call.args[0], typescript);
        const callback = call.callback;
        if (title === undefined) {
          manualRequirements.push(manualRequirement(model, node, "computed test title", callback ?? node));
          return;
        }
        if (!callback || !(typescript.isArrowFunction(callback) || typescript.isFunctionExpression(callback))) {
          manualRequirements.push(manualRequirement(model, node, "dynamic test factory", node, title === undefined ? undefined : [...parentTitles, title].join(" > "), true, Boolean(call.eachTable)));
          return;
        }
        const fullTitle = [...parentTitles, title].join(" > ");
        if (insideNamedRegistration(node, typescript)) {
          manualRequirements.push(manualRequirement(model, node, "test declaration inside registration function", callback, fullTitle, false, Boolean(call.eachTable)));
          return;
        }
        let eachAuthorityText;
        let eachAuthorityOffset;
        if (call.eachTable) {
          const table = unwrapExpression(call.eachTable, typescript);
          if (typescript.isIdentifier(table)) {
            const resolved = model.resolve(table);
            const initializer = resolved.length === 1 ? unwrapExpression(resolved[0].analysisNode, typescript) : undefined;
            if (!initializer || !(typescript.isArrayLiteralExpression(initializer) || typescript.isTaggedTemplateExpression(initializer))) {
              manualRequirements.push(manualRequirement(model, node, "unresolved dynamic .each table", callback, fullTitle, true, true));
              return;
            }
            const declaration = resolved[0].node.parent;
            const statement = declaration?.parent?.parent;
            eachAuthorityText = normalizeText(statement?.getText(model.file) ?? declaration?.getText(model.file));
            eachAuthorityOffset = statement?.getStart(model.file) ?? declaration?.getStart(model.file);
          } else if (typescript.isArrayLiteralExpression(table) || typescript.isTaggedTemplateExpression(table)) {
            eachAuthorityText = normalizeText(table.getText(model.file));
            eachAuthorityOffset = table.getStart(model.file);
          } else {
            manualRequirements.push(manualRequirement(model, node, "unresolved dynamic .each table", callback, fullTitle, true, true));
            return;
          }
        }
        const closure = bindingClosure(callback, model);
        if (closure.keys.some((key) => model.bindings.get(key)?.role.kind === "unsupported-binding")) {
          manualRequirements.push(manualRequirement(model, node, "unsupported destructuring or alias flow", callback, fullTitle, true, Boolean(call.eachTable)));
          return;
        }
        if (closure.ambiguous.length) {
          manualRequirements.push(manualRequirement(model, node, `ambiguous lexical binding: ${closure.ambiguous.join(", ")}`, callback, fullTitle, true, Boolean(call.eachTable)));
          return;
        }
        const assertions = assertionInfo(callback, model);
        if (assertions.unknownDirectAssert.length) {
          manualRequirements.push(manualRequirement(model, node, "unresolved direct assert binding", callback, fullTitle, true, Boolean(call.eachTable)));
          return;
        }
        declarations.push({
          path: model.path,
          title: fullTitle,
          sourceRange: rangeOf(model.file, node),
          sourceSlice: normalizeText(node.getText(model.file)),
          sourceOffset: node.getStart(model.file),
          sourceEnd: node.getEnd(),
          assertionOrdinals: assertions.ordinals,
          referencedBindingKeys: closure.keys,
          referencedSymbols: referencedSymbols(callback, model),
          eachAuthorityText,
          eachAuthorityOffset,
        });
        return;
      }
      if (typescript.isCallExpression(node) && typescript.isIdentifier(node.expression)) {
        const resolved = model.resolve(node.expression);
        const initializer = resolved.length === 1 ? unwrapExpression(resolved[0].analysisNode, typescript) : undefined;
        if (initializer && typescript.isIdentifier(initializer) && TEST_NAMES.has(initializer.text)) {
          manualRequirements.push(manualRequirement(model, node, "factory-created test declaration"));
          return;
        }
      }
      typescript.forEachChild(node, (child) => visit(child, parentTitles));
    };
    visit(model.file);
  }
  declarations.sort((a, b) => compareText(a.path, b.path) || a.sourceOffset - b.sourceOffset);
  manualRequirements.sort((a, b) => compareText(a.path, b.path) || a.sourceOffset - b.sourceOffset);
  Object.defineProperty(declarations, "manualRequirements", { value: manualRequirements, enumerable: true });
  return declarations;
}

function normalizeGitMetadata(input) {
  if (input instanceof Set || Array.isArray(input)) return { trackedPaths: new Set([...input].map(normalizePath)), pathKinds: new Map(), ignoredPaths: new Set(), directoryEntries: new Map(), readerSites: new Map() };
  const metadata = {
    trackedPaths: new Set([...(input?.trackedPaths ?? [])].map(normalizePath)),
    pathKinds: new Map([...(input?.pathKinds ?? [])].map(([key, value]) => [normalizePath(key), value])),
    ignoredPaths: new Set([...(input?.ignoredPaths ?? [])].map(normalizePath)),
    directoryEntries: new Map(input?.directoryEntries ?? []),
    readerSites: new Map(input?.readerSites ?? []),
  };
  for (const [pathValue, kind] of metadata.pathKinds) {
    if (!isNormalizedRepoPath(pathValue) || !PATH_KINDS.has(kind)) throw new Error(`invalid path kind for ${pathValue}: ${kind}`);
  }
  for (const [site, entries] of metadata.directoryEntries) {
    if (typeof site !== "string" || !Array.isArray(entries) || entries.some((entry) => !isNormalizedRepoPath(normalizePath(entry)))) throw new Error(`invalid directory provenance: ${site}`);
  }
  return metadata;
}

function bindingForIdentifier(identifier, model) {
  const resolved = model.resolve(identifier);
  return resolved.length === 1 ? resolved[0] : undefined;
}

function staticPathValue(node, model, seen = new Set()) {
  const current = unwrapExpression(node, model.typescript);
  const literal = staticText(current, model.typescript);
  if (literal !== undefined) return { kind: "path", value: literal };
  if (model.typescript.isIdentifier(current)) {
    const binding = bindingForIdentifier(current, model);
    if (!binding || seen.has(binding.key) || !binding.analysisNode) return { kind: "unknown" };
    seen.add(binding.key);
    return staticPathValue(binding.analysisNode, model, seen);
  }
  if (model.typescript.isPropertyAccessExpression(current) && current.expression.getText(model.file) === "import.meta" && current.name.text === "dirname") {
    return { kind: "path", value: path.posix.dirname(model.path) };
  }
  if (!model.typescript.isCallExpression(current)) return { kind: "unknown" };
  const expression = current.expression;
  if (model.typescript.isPropertyAccessExpression(expression)) {
    const base = rootIdentifier(expression, model.typescript);
    const binding = base ? bindingForIdentifier(base, model) : undefined;
    if (binding?.role.kind === "path-default" || binding?.role.kind === "path-namespace") {
      if (["join", "resolve"].includes(expression.name.text)) {
        const parts = current.arguments.map((argument) => staticPathValue(argument, model, new Set(seen)));
        if (parts.some((part) => part.kind === "temp")) return { kind: "temp" };
        if (parts.every((part) => part.kind === "path")) return { kind: "path", value: path.posix.normalize(parts.map((part) => part.value).join("/")) };
      }
    }
    if (expression.expression.getText(model.file) === "process" && expression.name.text === "cwd") return { kind: "path", value: "" };
  }
  if (model.typescript.isIdentifier(expression)) {
    const binding = bindingForIdentifier(expression, model);
    if (binding?.role.kind === "path-named" && ["join", "resolve"].includes(binding.role.importedName)) {
      const parts = current.arguments.map((argument) => staticPathValue(argument, model, new Set(seen)));
      if (parts.some((part) => part.kind === "temp")) return { kind: "temp" };
      if (parts.every((part) => part.kind === "path")) return { kind: "path", value: path.posix.normalize(parts.map((part) => part.value).join("/")) };
    }
    if ((binding?.role.kind === "os-named" && binding.role.importedName === "tmpdir") || (binding?.role.kind === "fs-named" && ["mkdtemp", "mkdtempSync"].includes(binding.role.importedName))) return { kind: "temp" };
  }
  return { kind: "unknown" };
}

function resolveAuthority(candidate, filePath) {
  const clean = normalizePath(String(candidate).replace(/\?.*$/, ""));
  if (String(candidate).startsWith(".")) return path.posix.normalize(path.posix.join(path.posix.dirname(filePath), clean));
  if (clean.startsWith("$lib/")) return `src/lib/${clean.slice(5)}`;
  return clean.replace(/^\//, "");
}

function classificationFor(authorityPath, metadata) {
  if (metadata.ignoredPaths.has(authorityPath)) return "ignored";
  const exact = metadata.pathKinds.get(authorityPath);
  if (exact) return exact;
  if (metadata.trackedPaths.has(authorityPath)) return TEST_FILE.test(authorityPath) ? "test" : "production";
  return undefined;
}

function directoryClassificationFor(authorityPath, metadata) {
  if (metadata.ignoredPaths.has(authorityPath)) return "ignored";
  const exact = metadata.pathKinds.get(authorityPath);
  if (exact) return exact;
  if (metadata.trackedPaths.has(authorityPath) && TEST_FILE.test(authorityPath)) return "test";
  return undefined;
}

function globMatcher(pattern) {
  let expression = "";
  for (let index = 0; index < pattern.length; index += 1) {
    const character = pattern[index];
    if (character === "*" && pattern[index + 1] === "*") {
      index += 1;
      if (pattern[index + 1] === "/") {
        index += 1;
        expression += "(?:[^/]+/)*";
      } else expression += ".*";
    } else if (character === "*") expression += "[^/]*";
    else if (character === "?") expression += "[^/]";
    else if (character === "{") {
      const closingBrace = pattern.indexOf("}", index + 1);
      const alternatives = closingBrace === -1 ? [] : pattern.slice(index + 1, closingBrace).split(",");
      if (alternatives.length > 1) {
        expression += `(?:${alternatives.map((alternative) => globMatcher(alternative).source.slice(1, -1)).join("|")})`;
        index = closingBrace;
      } else expression += "\\{";
    }
    else expression += character.replace(/[.+^$()|[\]{}\\]/g, "\\$&");
  }
  return new RegExp(`^${expression}$`);
}

function enclosingBindingKey(node, model) {
  let current = node.parent;
  while (current) {
    if (model.typescript.isVariableDeclaration(current) && model.typescript.isIdentifier(current.name)) {
      const candidates = [...model.bindings.values()].filter((binding) => binding.node === current.name);
      return candidates.length === 1 ? candidates[0].key : undefined;
    }
    if (model.typescript.isFunctionDeclaration(current) && current.name) {
      const candidates = [...model.bindings.values()].filter((binding) => binding.node === current.name);
      return candidates.length === 1 ? candidates[0].key : undefined;
    }
    current = current.parent;
  }
  return undefined;
}

function rawGlobCall(node, model) {
  if (!model.typescript.isPropertyAccessExpression(node.expression) || node.expression.name.text !== "glob" || node.expression.expression.getText(model.file) !== "import.meta") return false;
  const options = node.arguments[1];
  if (!options || !model.typescript.isObjectLiteralExpression(options)) return false;
  return options.properties.some((property) => model.typescript.isPropertyAssignment(property)
    && ["query", "as"].includes(property.name.getText(model.file).replace(/["']/g, ""))
    && ["?raw", "raw"].includes(staticText(property.initializer, model.typescript)));
}

function callImportRole(node, model) {
  const expression = node.expression;
  if (model.typescript.isIdentifier(expression)) {
    const binding = bindingForIdentifier(expression, model);
    return binding ? { binding, importedName: binding.role.importedName, role: binding.role } : undefined;
  }
  if (model.typescript.isPropertyAccessExpression(expression) && model.typescript.isIdentifier(expression.expression)) {
    const binding = bindingForIdentifier(expression.expression, model);
    return binding ? { binding, importedName: expression.name.text, role: binding.role } : undefined;
  }
  return undefined;
}

function readerRecord(model, node, values) {
  return {
    path: model.path,
    sourceRange: rangeOf(model.file, node),
    sourceOffset: node.getStart(model.file),
    sourceSlice: normalizeText(node.getText(model.file)),
    authorityText: normalizeText(node.getText(model.file)),
    bindingKey: enclosingBindingKey(node, model),
    ...values,
  };
}

function ancestorImportDeclaration(node, typescript) {
  let current = node;
  while (current && !typescript.isImportDeclaration(current)) current = current.parent;
  return current;
}

export function discoverSourceReaders(programOrFiles, gitMetadata, typescript = tsDefault) {
  const sourceFiles = Array.isArray(programOrFiles)
    ? programOrFiles
    : programOrFiles?.getSourceFiles?.().filter((file) => !file.isDeclarationFile).map((file) => ({ path: file.fileName, source: file.getFullText() })) ?? [];
  const metadata = normalizeGitMetadata(gitMetadata);
  const readers = [];
  for (const entry of sourceEntries(sourceFiles, typescript)) {
    const model = createLexicalModel(entry, typescript);
    for (const binding of model.bindings.values()) {
      if (binding.role.kind !== "raw") continue;
      const importDeclaration = ancestorImportDeclaration(binding.node, typescript);
      if (!importDeclaration || !typescript.isImportDeclaration(importDeclaration)) continue;
      const authorityPath = resolveAuthority(binding.role.moduleName, model.path);
      const classification = classificationFor(authorityPath, metadata);
      if (classification) readers.push(readerRecord(model, importDeclaration, { kind: "raw-import", authorityPath, classification, bindingKey: binding.key }));
      else readers.push(readerRecord(model, importDeclaration, { kind: "manual", reason: "untracked raw import", bindingKey: binding.key }));
    }

    const visit = (node) => {
      if (typescript.isCallExpression(node)) {
        if (rawGlobCall(node, model)) {
          const candidate = staticPathValue(node.arguments[0], model);
          if (candidate.kind !== "path") readers.push(readerRecord(model, node, { kind: "manual", reason: "dynamic raw glob" }));
          else {
            const pattern = resolveAuthority(candidate.value, model.path);
            const matcher = globMatcher(pattern);
            const matches = [...new Set([...metadata.trackedPaths, ...metadata.ignoredPaths])].filter((item) => matcher.test(item)).sort(compareText);
            if (!matches.length) readers.push(readerRecord(model, node, { kind: "manual", reason: "raw glob resolved no tracked or ignored authority" }));
            else for (const authorityPath of matches) readers.push(readerRecord(model, node, { kind: "import-meta-glob", authorityPath, classification: classificationFor(authorityPath, metadata) }));
          }
        } else {
          const imported = callImportRole(node, model);
          const fsKind = imported?.role.kind;
          if ((fsKind === "fs-named" || fsKind === "fs-default" || fsKind === "fs-namespace") && FS_READS.has(imported.importedName)) {
            const siteKey = `${model.path}:${rangeOf(model.file, node)}`;
            const explicitSite = metadata.readerSites.get(siteKey);
            const candidate = staticPathValue(node.arguments[0], model);
            if (explicitSite) {
              for (const authority of explicitSite.authorities ?? []) {
                if (!PATH_KINDS.has(authority.classification) || !isNormalizedRepoPath(authority.path)) throw new Error(`invalid reader-site provenance: ${siteKey}`);
                readers.push(readerRecord(model, node, {
                  kind: "fs-read",
                  authorityPath: authority.path,
                  classification: authority.classification,
                  ...(authority.classification === "fixture" && explicitSite.exception ? { exception: explicitSite.exception } : {}),
                }));
              }
            } else if (candidate.kind === "temp") readers.push(readerRecord(model, node, { kind: "fs-read", classification: "temp" }));
            else if (candidate.kind !== "path") readers.push(readerRecord(model, node, { kind: "manual", reason: "dynamic or unknown filesystem authority" }));
            else if (["readdir", "readdirSync"].includes(imported.importedName)) {
              const exactEntries = metadata.directoryEntries.get(siteKey);
              if (!Array.isArray(exactEntries) || !exactEntries.length) readers.push(readerRecord(model, node, { kind: "manual", reason: "directory enumeration requires exact file provenance" }));
              else for (const item of exactEntries.map(normalizePath).sort(compareText)) {
                const classification = directoryClassificationFor(item, metadata);
                if (!classification) readers.push(readerRecord(model, node, { kind: "manual", reason: `unclassified directory entry: ${item}` }));
                else readers.push(readerRecord(model, node, { kind: "fs-directory-read", authorityPath: item, classification }));
              }
            } else {
              const authorityPath = resolveAuthority(candidate.value, model.path);
              const classification = classificationFor(authorityPath, metadata);
              if (classification) readers.push(readerRecord(model, node, { kind: "fs-read", authorityPath, classification }));
              else readers.push(readerRecord(model, node, { kind: "manual", reason: "untracked filesystem authority" }));
            }
          } else if (imported?.role.kind === "helper-named" && SOURCE_HELPERS.get(imported.role.helper)?.has(imported.role.importedName)) {
            readers.push(readerRecord(model, node, { kind: "contract-path-helper", authorityPath: imported.role.moduleName, classification: "production", exportName: imported.role.importedName }));
          } else if (imported?.role.kind === "helper-namespace" && SOURCE_HELPERS.get(imported.role.helper)?.has(imported.importedName)) {
            readers.push(readerRecord(model, node, { kind: "contract-path-helper", authorityPath: imported.role.moduleName, classification: "production", exportName: imported.importedName }));
          } else if (imported?.role.kind?.startsWith("import-") && /^(?:read|load).*(?:source|file)/i.test(imported.importedName ?? imported.binding.name)) {
            readers.push(readerRecord(model, node, { kind: "manual", reason: "unknown source-reader wrapper" }));
          }
        }
      }
      typescript.forEachChild(node, visit);
    };
    visit(model.file);
  }
  return readers.sort((a, b) => compareText(a.path, b.path) || a.sourceOffset - b.sourceOffset || compareText(a.authorityPath ?? "", b.authorityPath ?? ""));
}

function declarationIdentity(value) {
  return value?.manual ? `${value.path}@${value.manual.sourceRange}` : `${value?.path}#${value?.title}`;
}

function readerApplies(declaration, reader) {
  if (reader.path !== declaration.path) return false;
  if (reader.dependentDeclarationKeys?.includes(`${declaration.path}#${declaration.title}`)) return true;
  if (reader.bindingKey && declaration.referencedBindingKeys?.includes(reader.bindingKey)) return true;
  return Number.isInteger(reader.sourceOffset) && reader.sourceOffset >= declaration.sourceOffset && reader.sourceOffset < (declaration.sourceEnd ?? declaration.sourceOffset + declaration.sourceSlice.length);
}

function titlesFor(pathValue, runnerTitlesByPath) {
  const values = runnerTitlesByPath instanceof Map ? runnerTitlesByPath.get(pathValue) : runnerTitlesByPath?.[pathValue];
  return Array.isArray(values) ? [...new Set(values.filter((value) => typeof value === "string" && value.trim()).map((value) => value.trim()))].sort(compareText) : [];
}

function eachTitleMatcher(title) {
  let expression = "";
  for (let index = 0; index < title.length; index += 1) {
    if (title[index] === "$" && /[A-Za-z_$]/.test(title[index + 1] ?? "")) {
      index += 1;
      while (index + 1 < title.length && /[\w$.-]/.test(title[index + 1])) index += 1;
      expression += ".+?";
    } else if (title[index] === "%" && /[sdifjoO#]/.test(title[index + 1] ?? "")) {
      index += 1;
      expression += ".+?";
    } else expression += title[index].replace(/[.+^$()|[\]{}\\]/g, "\\$&");
  }
  return new RegExp(`^${expression}$`);
}

function authorityTextFor(declaration, readers) {
  const parts = readers.filter((reader) => reader.authorityText).map((reader) => ({ offset: reader.sourceOffset, text: normalizeText(reader.authorityText) }));
  if (declaration.eachAuthorityText) parts.push({ offset: declaration.eachAuthorityOffset ?? declaration.sourceOffset, text: normalizeText(declaration.eachAuthorityText) });
  const seen = new Set();
  return parts.sort((a, b) => a.offset - b.offset || compareText(a.text, b.text)).filter((part) => !seen.has(`${part.offset}:${part.text}`) && seen.add(`${part.offset}:${part.text}`)).map((part) => part.text).join("\n");
}

function deriveObligations(declarationInventory, sourceReaders, runnerTitlesByPath, requireRunnerTitles) {
  const staticRows = [];
  const manualRows = [];
  const productionReaders = sourceReaders.filter((reader) => reader.kind !== "manual" && OBLIGATION_KINDS.has(reader.classification));
  const manualReaders = sourceReaders.filter((reader) => reader.kind === "manual");
  const allRelevantReaders = [...productionReaders, ...manualReaders];
  const ownedManualReaders = new Set();
  const manualRequirements = declarationInventory.manualRequirements ?? [];
  const knownAssignments = new Map();

  for (const requirement of manualRequirements) {
    if (!requirement.title) continue;
    const listed = titlesFor(requirement.path, runnerTitlesByPath);
    const reservedExact = new Set([
      ...declarationInventory.filter((item) => item.path === requirement.path && !item.eachAuthorityText).map((item) => item.title),
      ...manualRequirements.filter((item) => item !== requirement && item.path === requirement.path && item.title && !item.titleTemplate && item.titleExact).map((item) => item.title),
    ]);
    const matches = requirement.titleTemplate
      ? listed.filter((title) => eachTitleMatcher(requirement.title).test(title) && !reservedExact.has(title))
      : requirement.titleExact
      ? listed.filter((title) => title === requirement.title)
      : listed.filter((title) => title === requirement.title || title.endsWith(` > ${requirement.title}`));
    knownAssignments.set(requirement, matches);
  }

  const unknownByPath = new Map();
  for (const requirement of manualRequirements.filter((item) => !item.title)) {
    const values = unknownByPath.get(requirement.path) ?? [];
    values.push(requirement);
    unknownByPath.set(requirement.path, values);
  }
  const unknownAssignments = new Map();
  for (const [filePath, requirements] of unknownByPath) {
    const reserved = new Set(declarationInventory.filter((item) => item.path === filePath).map((item) => item.title));
    for (const [requirement, assigned] of knownAssignments) if (requirement.path === filePath) for (const title of assigned) reserved.add(title);
    const remaining = titlesFor(filePath, runnerTitlesByPath).filter((title) => !reserved.has(title));
    if (requirements.length === 1) unknownAssignments.set(requirements[0], remaining);
    else for (const requirement of requirements) unknownAssignments.set(requirement, []);
  }

  for (const declaration of declarationInventory) {
    const relatedManual = manualReaders.filter((reader) => readerApplies(declaration, reader));
    const relatedProduction = productionReaders.filter((reader) => readerApplies(declaration, reader));
    if (relatedManual.length) {
      for (const reader of relatedManual) ownedManualReaders.add(reader);
      const listedTitles = titlesFor(declaration.path, runnerTitlesByPath);
      const reservedExact = new Set(declarationInventory
        .filter((item) => item !== declaration && item.path === declaration.path && !item.eachAuthorityText)
        .map((item) => item.title));
      const runnerTitles = declaration.eachAuthorityText
        ? listedTitles.filter((title) => eachTitleMatcher(declaration.title).test(title) && !reservedExact.has(title))
        : listedTitles.filter((title) => title === declaration.title);
      if (requireRunnerTitles && (!runnerTitles.length || (!declaration.eachAuthorityText && runnerTitles.length !== 1))) throw new Error(`manual declaration ${declaration.path}#${declaration.title} requires exact non-empty freeze-time runnerTitles reconciliation`);
      manualRows.push({
        ...declaration,
        manualReason: [...new Set(relatedManual.map((reader) => reader.reason))].sort(compareText).join("; "),
        runnerTitles,
        authorityText: authorityTextFor(declaration, [...relatedProduction, ...relatedManual]),
      });
      continue;
    }
    if (!relatedProduction.length) continue;
    staticRows.push({ ...declaration, authorityText: authorityTextFor(declaration, relatedProduction) });
  }
  for (const requirement of manualRequirements) {
    const related = allRelevantReaders.filter((reader) => readerApplies(requirement, reader));
    if (!related.length) continue;
    for (const reader of related.filter((item) => item.kind === "manual")) ownedManualReaders.add(reader);
    const runnerTitles = requirement.title ? (knownAssignments.get(requirement) ?? []) : (unknownAssignments.get(requirement) ?? []);
    if (requireRunnerTitles && !runnerTitles.length) throw new Error(`manual requirement ${requirement.path}:${requirement.sourceRange} requires exact freeze-time runner-title reconciliation`);
    manualRows.push({ ...requirement, manualReason: requirement.reason, runnerTitles, authorityText: authorityTextFor(requirement, related) });
  }
  for (const reader of manualReaders.filter((item) => !ownedManualReaders.has(item))) {
    const dependents = declarationInventory.filter((declaration) => readerApplies(declaration, reader));
    const listedTitles = titlesFor(reader.path, runnerTitlesByPath);
    const dependentTitles = new Set(dependents.map((declaration) => declaration.title).filter(Boolean));
    const exactDependentTitles = dependentTitles.size ? listedTitles.filter((title) => dependentTitles.has(title)) : [];
    const runnerTitles = exactDependentTitles;
    if (requireRunnerTitles && runnerTitles.length !== 1) throw new Error(`owner-ambiguous manual reader ${reader.path}:${reader.sourceRange} requires exactly one reconciled freeze-time runner title`);
    manualRows.push({
      path: reader.path,
      sourceRange: reader.sourceRange,
      sourceSlice: reader.sourceSlice,
      sourceOffset: reader.sourceOffset,
      assertionOrdinals: dependents.flatMap((declaration) => declaration.assertionOrdinals).map((_, index) => index + 1),
      manualReason: reader.reason,
      runnerTitles,
      authorityText: reader.authorityText,
    });
  }
  return [...staticRows, ...manualRows].sort((a, b) => compareText(a.path, b.path) || a.sourceOffset - b.sourceOffset);
}

function draftRow(item, index) {
  const manual = Boolean(item.manualReason);
  const row = {
    id: `SC-${String(index + 1).padStart(6, "0")}`,
    path: item.path,
    ...(!manual && item.title ? { title: item.title } : { manual: { sourceRange: item.sourceRange, reason: item.manualReason, runnerTitles: item.runnerTitles } }),
    sourceHash: sha256(item.sourceSlice),
    assertionCount: item.assertionOrdinals.length,
    ...(item.authorityText ? { authorityHash: sha256(item.authorityText) } : {}),
    lineage: [],
    invariant: !manual && item.title ? "Review this source-contract assertion and preserve its user-observable invariant." : "Review this unsupported source-contract declaration manually.",
    disposition: "behavior",
    replacementIds: [],
  };
  return row;
}

export function buildLedgerDraft({
  declarationInventory = [], sourceReaders = [], frozenAtCommit, runnerTitlesByPath,
  outputPath, repoRoot, isIgnoredPath,
} = {}) {
  if (!/^[a-f0-9]{40}$/i.test(frozenAtCommit ?? "")) throw new Error("frozenAtCommit must be a full commit SHA");
  const obligations = deriveObligations(declarationInventory, sourceReaders, runnerTitlesByPath, true);
  const sourceReaderExceptions = [];
  const exceptionKeys = new Set();
  for (const reader of sourceReaders) {
    if (!reader.exception) continue;
    const key = `${reader.exception.path}:${reader.exception.sourceRange}`;
    if (!exceptionKeys.has(key)) sourceReaderExceptions.push(reader.exception);
    exceptionKeys.add(key);
  }
  const draft = { schemaVersion: 1, frozenAtCommit, sourceReaderExceptions, rows: obligations.map(draftRow) };
  if (outputPath !== undefined) {
    if (!repoRoot) throw new Error("repoRoot is required when writing a ledger draft");
    const artifactRoot = path.resolve(repoRoot, "artifacts");
    const resolvedOutput = path.resolve(repoRoot, outputPath);
    const relativeArtifact = path.relative(artifactRoot, resolvedOutput);
    if (!relativeArtifact || relativeArtifact.startsWith("..") || path.isAbsolute(relativeArtifact)) throw new Error("ledger drafts may only be written inside artifacts/");
    if (typeof isIgnoredPath !== "function") throw new Error("isIgnoredPath is required to verify draft output provenance");
    const repoRelative = normalizePath(path.relative(path.resolve(repoRoot), resolvedOutput));
    if (!isIgnoredPath(repoRelative)) throw new Error("ledger draft output must be Git-ignored");
    mkdirSync(path.dirname(resolvedOutput), { recursive: true });
    writeFileSync(resolvedOutput, `${JSON.stringify(draft, null, 2)}\n`, "utf8");
  }
  return draft;
}

function fieldIssues(value, allowed, label, issues, id = "unknown") {
  if (!value || typeof value !== "object" || Array.isArray(value)) return;
  for (const field of Object.keys(value)) {
    if (LIFECYCLE_FIELDS.has(field)) issues.push(`${id}: stored lifecycle field: ${field}`);
    else if (!allowed.has(field)) issues.push(`${id}: unknown ${label} field: ${field}`);
  }
}

function validateManual(manual, id, issues) {
  if (!manual || typeof manual !== "object" || Array.isArray(manual)) {
    issues.push(`${id}: invalid manual row`);
    return;
  }
  fieldIssues(manual, new Set(["sourceRange", "reason", "runnerTitles"]), "manual", issues, id);
  const titles = manual.runnerTitles;
  if (!isExactRange(manual.sourceRange)
    || typeof manual.reason !== "string" || !manual.reason.trim()
    || !Array.isArray(titles) || !titles.length
    || titles.some((title) => typeof title !== "string" || !title.trim())
    || new Set(titles).size !== titles.length) issues.push(`${id}: invalid manual row`);
}

function validateResolution(row, resolution, context, issues) {
  const prefix = `${row.id}:`;
  if (!resolution || typeof resolution !== "object" || Array.isArray(resolution)) return false;
  if (!DISPOSITIONS.has(resolution.disposition)) issues.push(`${prefix} invalid disposition`);
  if (resolution.disposition === "delete") {
    if (Object.prototype.hasOwnProperty.call(resolution, "replacementIds")) issues.push(`${prefix} delete must not contain replacementIds`);
    if (typeof resolution.deletionReason !== "string" || !resolution.deletionReason.trim()) issues.push(`${prefix} delete requires a specific deletionReason`);
    return DISPOSITIONS.has(resolution.disposition) && !Object.prototype.hasOwnProperty.call(resolution, "replacementIds") && typeof resolution.deletionReason === "string" && Boolean(resolution.deletionReason.trim());
  }
  if (Object.prototype.hasOwnProperty.call(resolution, "deletionReason")) issues.push(`${prefix} non-delete must not contain deletionReason`);
  if (!Array.isArray(resolution.replacementIds) || !resolution.replacementIds.length) {
    issues.push(`${prefix} missing replacementIds`);
    return false;
  }
  let syntaxValid = true;
  for (const id of resolution.replacementIds) {
    if (typeof id !== "string" || !REPLACEMENT.test(id)) {
      issues.push(`${prefix} unknown replacement namespace: ${id}`);
      syntaxValid = false;
    }
  }
  return syntaxValid && DISPOSITIONS.has(resolution.disposition) && !Object.prototype.hasOwnProperty.call(resolution, "deletionReason")
    && resolution.replacementIds.every((id) => replacementResolved(id, context));
}

function replacementResolved(id, context) {
  if (typeof id !== "string" || !id.startsWith("test:vitest:")) return false;
  const target = id.slice("test:vitest:".length);
  const split = target.indexOf("#");
  if (split < 1) return false;
  const filePath = target.slice(0, split);
  const title = target.slice(split + 1);
  if (!context.declarationInventory.some((declaration) => declaration.path === filePath && declaration.title === title)) return false;
  const owner = (context.liveCensus?.vitestOwners ?? []).find((candidate) => {
    const owned = candidate?.files ?? context.liveCensus?.vitestFiles?.[candidate?.id];
    return Array.isArray(owned) && owned.includes(filePath);
  });
  return Boolean(owner && context.verifySteps?.some((step) => step?.npmScript === owner.ownerScript));
}

function validateRowShape(row, issues) {
  const id = typeof row?.id === "string" ? row.id : "unknown";
  const mixed = row && Object.prototype.hasOwnProperty.call(row, "subgroups");
  const common = ["id", "path", "title", "manual", "sourceHash", "assertionCount", "authorityHash", "lineage", "invariant"];
  fieldIssues(row, new Set(mixed ? [...common, "subgroups"] : [...common, "disposition", "replacementIds", "deletionReason"]), "ledger row", issues, id);
  if (!isNormalizedRepoPath(row.path)) issues.push(`${id}: invalid path`);
  if (typeof row.sourceHash !== "string" || !/^[a-f0-9]{64}$/i.test(row.sourceHash)) issues.push(`${id}: invalid sourceHash`);
  if (!Number.isInteger(row.assertionCount) || row.assertionCount < 0) issues.push(`${id}: invalid assertionCount`);
  if (row.authorityHash !== undefined && (typeof row.authorityHash !== "string" || !/^[a-f0-9]{64}$/i.test(row.authorityHash))) issues.push(`${id}: invalid authorityHash`);
  if (typeof row.invariant !== "string" || !row.invariant.trim()) issues.push(`${id}: missing invariant`);
  const hasTitle = Object.prototype.hasOwnProperty.call(row, "title");
  const hasManual = Object.prototype.hasOwnProperty.call(row, "manual");
  if (hasTitle === hasManual) issues.push(`${id}: row requires exactly one title or manual`);
  if (hasTitle && (typeof row.title !== "string" || !row.title.trim())) issues.push(`${id}: invalid title`);
  if (hasManual) validateManual(row.manual, id, issues);
  const lineage = row.lineage;
  if (!Array.isArray(lineage) || lineage.some((item) => !isNormalizedRepoPath(item) || item === row.path) || new Set(lineage).size !== lineage.length) issues.push(`${id}: invalid lineage`);
  if (mixed) {
    if (!Array.isArray(row.subgroups) || row.subgroups.length < 2 || row.assertionCount < 2) issues.push(`${id}: unnecessary or invalid subgroups`);
    for (const subgroup of Array.isArray(row.subgroups) ? row.subgroups : []) {
      fieldIssues(subgroup, new Set(["assertionOrdinals", "invariant", "disposition", "replacementIds", "deletionReason"]), "subgroup", issues, id);
      if (typeof subgroup?.invariant !== "string" || !subgroup.invariant.trim()) issues.push(`${id}: subgroup missing invariant`);
    }
  }
}

function validException(exception) {
  return exception && typeof exception === "object" && !Array.isArray(exception)
    && Object.keys(exception).every((field) => ["path", "sourceRange", "reason", "owner"].includes(field))
    && isNormalizedRepoPath(exception.path) && isExactRange(exception.sourceRange)
    && typeof exception.reason === "string" && Boolean(exception.reason.trim())
    && typeof exception.owner === "string" && Boolean(exception.owner.trim());
}

export function validateSourceContractLedger(context = {}) {
  const ledger = context.ledger;
  const declarationInventory = Array.isArray(context.declarationInventory) ? context.declarationInventory : [];
  const sourceReaders = Array.isArray(context.sourceReaders) ? context.sourceReaders : [];
  const issues = [];
  const states = [];
  if (!ledger || typeof ledger !== "object" || Array.isArray(ledger)) return { issues: ["invalid source-contract ledger envelope"], rows: states };
  fieldIssues(ledger, new Set(["schemaVersion", "frozenAtCommit", "sourceReaderExceptions", "rows"]), "ledger envelope", issues);
  if (ledger.schemaVersion !== 1 || !Array.isArray(ledger.rows) || !Array.isArray(ledger.sourceReaderExceptions) || !/^[a-f0-9]{40}$/i.test(ledger.frozenAtCommit ?? "")) {
    return { issues: [...new Set([...issues, "invalid source-contract ledger envelope"])].sort(compareText), rows: states };
  }

  let obligations = [];
  try {
    obligations = deriveObligations(declarationInventory, sourceReaders, context.runnerTitlesByPath, true);
  } catch (error) {
    issues.push(error instanceof Error ? error.message : String(error));
    obligations = deriveObligations(declarationInventory, sourceReaders, context.runnerTitlesByPath, false);
  }
  const current = new Map();
  for (const obligation of obligations) {
    const value = obligation.manualReason
      ? { ...obligation, manual: { sourceRange: obligation.sourceRange } }
      : obligation;
    const key = declarationIdentity(value);
    if (current.has(key)) issues.push(`duplicate current identity: ${key}`);
    else current.set(key, obligation);
  }
  const replacementIdentities = new Set();
  for (const declaration of declarationInventory) {
    const key = declarationIdentity(declaration);
    if (replacementIdentities.has(key)) issues.push(`duplicate current identity: ${key}`);
    replacementIdentities.add(key);
  }

  const ids = new Set();
  const rowsByIdentity = new Map();
  const validRows = [];
  const invalidRowIds = new Set();
  for (const row of ledger.rows) {
    if (!row || typeof row !== "object" || Array.isArray(row)) {
      issues.push("invalid ledger row");
      continue;
    }
    const issueCount = issues.length;
    validateRowShape(row, issues);
    if (!/^SC-\d{6}$/.test(row.id ?? "") || ids.has(row.id)) issues.push(`duplicate or invalid ledger id: ${row.id}`);
    if (issues.length > issueCount) invalidRowIds.add(row.id);
    ids.add(row.id);
    const key = declarationIdentity(row);
    if (rowsByIdentity.has(key)) issues.push(`duplicate current identity: ${key}`);
    rowsByIdentity.set(key, row);
    validRows.push(row);
  }

  const manualTitleOwners = new Map();
  for (const row of validRows) {
    for (const title of row.manual?.runnerTitles ?? []) {
      const key = `${row.path}#${title}`;
      const owner = manualTitleOwners.get(key);
      if (owner) {
        issues.push(`duplicate manual runner-title ownership: ${key}`);
        invalidRowIds.add(owner.id);
        invalidRowIds.add(row.id);
      } else manualTitleOwners.set(key, row);
    }
  }

  for (const [key, obligation] of current) {
    const row = rowsByIdentity.get(key);
    if (!row) {
      issues.push(`missing ledger row: ${key}`);
      continue;
    }
    if (row.sourceHash !== sha256(obligation.sourceSlice)) issues.push(`${row.id}: sourceHash drift`);
    if (row.assertionCount !== obligation.assertionOrdinals.length) issues.push(`${row.id}: assertionCount drift`);
    const expectedAuthority = obligation.authorityText ? sha256(obligation.authorityText) : undefined;
    if (row.authorityHash !== expectedAuthority) issues.push(`${row.id}: authorityHash drift`);
    if (obligation.manualReason) {
      const expectedTitles = obligation.runnerTitles;
      if (row.manual?.reason !== obligation.manualReason || JSON.stringify(row.manual?.runnerTitles) !== JSON.stringify(expectedTitles)) issues.push(`${row.id}: manual requirement drift`);
    }
  }

  const resolutionContext = { ...context, declarationInventory };
  for (const row of validRows) {
    const currentRow = current.has(declarationIdentity(row));
    let resolved = false;
    if (Object.prototype.hasOwnProperty.call(row, "subgroups")) {
      if (["disposition", "replacementIds", "deletionReason"].some((field) => Object.prototype.hasOwnProperty.call(row, field))) issues.push(`${row.id}: mixed row has top-level resolution`);
      const owned = new Set();
      const subgroupClosed = [];
      for (const subgroup of Array.isArray(row.subgroups) ? row.subgroups : []) {
        if (!Array.isArray(subgroup?.assertionOrdinals) || !subgroup.assertionOrdinals.length) issues.push(`${row.id}: subgroup missing assertion ordinals`);
        for (const ordinal of subgroup?.assertionOrdinals ?? []) {
          if (!Number.isInteger(ordinal) || ordinal < 1 || ordinal > row.assertionCount) issues.push(`${row.id}: invalid subgroup assertion ordinal: ${ordinal}`);
          else if (owned.has(ordinal)) issues.push(`${row.id}: overlapping subgroup assertion ordinal: ${ordinal}`);
          else owned.add(ordinal);
        }
        subgroupClosed.push(validateResolution(row, subgroup, resolutionContext, issues));
      }
      for (let ordinal = 1; ordinal <= row.assertionCount; ordinal++) if (!owned.has(ordinal)) issues.push(`${row.id}: incomplete subgroup assertion ordinals`);
      resolved = subgroupClosed.length > 0 && subgroupClosed.every(Boolean) && !invalidRowIds.has(row.id) && !issues.some((issue) => issue.startsWith(`${row.id}:`));
    } else resolved = validateResolution(row, row, resolutionContext, issues) && !invalidRowIds.has(row.id);
    if (!currentRow && !resolved) issues.push(`${row.id}: unresolved historical row`);
    states.push({ id: row.id, state: currentRow ? "open" : resolved ? "closed" : "open" });
  }

  const exceptionKeys = new Set();
  for (const exception of ledger.sourceReaderExceptions) {
    fieldIssues(exception, new Set(["path", "sourceRange", "reason", "owner"]), "sourceReaderException", issues);
    if (!validException(exception)) {
      issues.push("invalid sourceReaderException");
      continue;
    }
    const key = `${exception.path}:${exception.sourceRange}`;
    if (exceptionKeys.has(key)) issues.push(`duplicate sourceReaderException: ${key}`);
    exceptionKeys.add(key);
  }
  const fixtureKeys = new Set(sourceReaders.filter((reader) => reader.classification === "fixture").map((reader) => `${reader.path}:${reader.sourceRange}`));
  for (const key of fixtureKeys) if (!exceptionKeys.has(key)) issues.push(`missing sourceReaderException: ${key}`);
  for (const key of exceptionKeys) if (!fixtureKeys.has(key)) issues.push(`stale sourceReaderException: ${key}`);
  return { issues: [...new Set(issues)].sort(compareText), rows: states };
}

function collectRunnerTitles(repoRoot) {
  const output = execFileSync(process.execPath, ["scripts/run-vitest.mjs", "list", "--json", "--no-color"], { cwd: repoRoot, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 });
  const listed = JSON.parse(output);
  const titles = {};
  for (const item of listed) {
    const relative = normalizePath(path.relative(repoRoot, item.file));
    (titles[relative] ??= []).push(item.name);
  }
  for (const key of Object.keys(titles)) titles[key] = [...new Set(titles[key])].sort(compareText);
  return titles;
}

export function selectVitestTrackedPaths(trackedPaths, runnerTitlesByPath) {
  const listed = new Set(Object.keys(runnerTitlesByPath ?? {}).map(normalizePath));
  return [...trackedPaths].map(normalizePath).filter((item) => TEST_FILE.test(item) && listed.has(item)).sort(compareText);
}

export function createCliGitMetadata(trackedPaths, ignoredPaths = new Set()) {
  const tracked = new Set([...trackedPaths].map(normalizePath));
  const ignored = new Set([...ignoredPaths].map(normalizePath));
  const fixturePaths = [
    "src-tauri/crates/extractum-analysis/src/test_schema.rs",
    "src-tauri/src/analysis/test_schema.rs",
  ];
  const pathKinds = new Map(fixturePaths.filter((item) => tracked.has(item)).map((item) => [item, "fixture"]));
  const promptPackEntries = [...tracked]
    .filter((item) => /^src-tauri\/src\/prompt_packs\/[^/]+\.rs$/.test(item))
    .sort(compareText);
  const lowerCrateManifests = [...tracked]
    .filter((item) => /^src-tauri\/crates\/[^/]+\/Cargo\.toml$/.test(item) && item !== "src-tauri/crates/extractum-analysis/Cargo.toml")
    .sort(compareText);
  for (const item of [...promptPackEntries, ...lowerCrateManifests]) pathKinds.set(item, "production");
  const directoryEntries = new Map([
    ["src/lib/prompt-pack-application-contract.test.ts:10:29-10:84", promptPackEntries],
    ["src/lib/analysis-crate-boundary-contract.test.ts:4172:28-4175:6", lowerCrateManifests],
  ]);
  const fixtureException = {
    path: "src/lib/analysis-migration-fixture-contract.test.ts",
    sourceRange: "14:13-14:74",
    reason: "test-only migration schema candidates are intentional fixture authorities",
    owner: "analysis migration fixture contract",
  };
  const readerSites = new Map([
    ["src/lib/analysis-migration-fixture-contract.test.ts:14:13-14:74", {
      authorities: [
        { path: "src-tauri/src/migrations.rs", classification: "production" },
        ...fixturePaths.filter((item) => tracked.has(item)).map((item) => ({ path: item, classification: "fixture" })),
      ],
      exception: fixtureException,
    }],
    ["src/lib/youtube-summary-smoke-fixture-contract.test.ts:14:30-14:56", {
      authorities: [
        "src/lib/api/prompt-packs.ts",
        "src/lib/components/research-projects/YoutubeSummaryResultView.svelte",
        "src/lib/components/research-projects/YoutubeSummaryRunDialog.svelte",
        "src/lib/components/research-projects/YoutubeSummaryRunsPanel.svelte",
      ].map((item) => ({ path: item, classification: "production" })),
    }],
  ]);
  return { trackedPaths: tracked, ignoredPaths: ignored, pathKinds, directoryEntries, readerSites };
}

async function main() {
  const outputIndex = process.argv.indexOf("--output");
  if (outputIndex < 0 || !process.argv[outputIndex + 1]) throw new Error("Use --output artifacts/.../source-contract-ledger.draft.json");
  const repoRoot = path.resolve(fileURLToPath(new URL("../..", import.meta.url)));
  const tracked = execFileSync("git", ["ls-files", "-z"], { cwd: repoRoot, encoding: "utf8" }).split("\0").filter(Boolean).map(normalizePath);
  const ignored = execFileSync("git", ["ls-files", "--others", "--ignored", "--exclude-standard", "-z"], { cwd: repoRoot, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 }).split("\0").filter(Boolean).map(normalizePath);
  const runnerTitlesByPath = collectRunnerTitles(repoRoot);
  const tests = selectVitestTrackedPaths(tracked, runnerTitlesByPath).map((item) => ({ path: item, source: readFileSync(path.join(repoRoot, item), "utf8") }));
  const declarationInventory = discoverTestDeclarations(tests, tsDefault);
  const sourceReaders = discoverSourceReaders(tests, createCliGitMetadata(new Set(tracked), new Set(ignored)), tsDefault);
  const outputPath = process.argv[outputIndex + 1];
  const draft = buildLedgerDraft({
    declarationInventory,
    sourceReaders,
    runnerTitlesByPath,
    frozenAtCommit: execFileSync("git", ["rev-parse", "HEAD"], { cwd: repoRoot, encoding: "utf8" }).trim(),
    outputPath,
    repoRoot,
    isIgnoredPath: (repoRelative) => {
      try {
        execFileSync("git", ["check-ignore", "--quiet", "--no-index", "--", repoRelative], { cwd: repoRoot, stdio: "ignore" });
        return true;
      } catch {
        return false;
      }
    },
  });
  const manualRows = draft.rows.filter((row) => row.manual);
  console.log(`Source-contract ledger draft: ${draft.rows.length} rows; ${declarationInventory.length} static declarations; ${declarationInventory.manualRequirements.length} declaration manual requirements; ${sourceReaders.length} reader records; ${manualRows.length} manual rows`);
  for (const row of manualRows) console.log(`${row.id} ${row.path}:${row.manual.sourceRange} titles=${row.manual.runnerTitles.length} reason=${row.manual.reason}`);
}

if (process.argv[1] && import.meta.url === new URL(`file:///${process.argv[1].replaceAll("\\", "/")}`).href) {
  main().catch((error) => { console.error(error instanceof Error ? error.message : String(error)); process.exitCode = 1; });
}
