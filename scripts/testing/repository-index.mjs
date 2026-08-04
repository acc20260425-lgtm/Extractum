import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import * as svelteCompiler from "svelte/compiler";
import tsCompiler from "typescript";

const CARGO_MANIFEST_PATH = "src-tauri/Cargo.toml";

function freeze(value) {
  if (!value || typeof value !== "object" || Object.isFrozen(value)) return value;
  for (const child of Object.values(value)) freeze(child);
  return Object.freeze(value);
}

function repositoryPath(root, inputPath) {
  const relativePath = String(inputPath ?? "");
  if (
    !relativePath
    || path.isAbsolute(relativePath)
    || relativePath.includes("\\")
    || relativePath.split("/").some((segment) => !segment || segment === "." || segment === "..")
  ) {
    throw new Error(`Invalid repository-relative path: ${relativePath}`);
  }

  const absolutePath = path.resolve(root, relativePath);
  const selected = path.relative(root, absolutePath);
  if (selected === "" || selected === ".." || selected.startsWith(`..${path.sep}`) || path.isAbsolute(selected)) {
    throw new Error(`Repository path escapes root: ${relativePath}`);
  }
  return { absolutePath, relativePath };
}

function errorFor(relativePath, error) {
  const detail = error instanceof Error ? error.message : String(error);
  return new Error(`RepositoryIndex failed for ${relativePath}: ${detail}`, { cause: error });
}

function scriptKind(relativePath, typescript) {
  if (relativePath.endsWith(".tsx")) return typescript.ScriptKind.TSX;
  if (relativePath.endsWith(".jsx")) return typescript.ScriptKind.JSX;
  if (relativePath.endsWith(".js") || relativePath.endsWith(".mjs") || relativePath.endsWith(".cjs")) {
    return typescript.ScriptKind.JS;
  }
  return typescript.ScriptKind.TS;
}

function importFact(node, typescript) {
  const clause = node.importClause;
  const named = clause?.namedBindings;
  const namedImports = named && typescript.isNamedImports(named)
    ? named.elements.map((specifier) => ({
        imported: specifier.propertyName?.text ?? specifier.name.text,
        local: specifier.name.text,
        typeOnly: Boolean(specifier.isTypeOnly),
      }))
    : [];

  return {
    source: node.moduleSpecifier.text,
    defaultImport: clause?.name?.text ?? null,
    namedImports,
    namespaceImport: named && typescript.isNamespaceImport(named) ? named.name.text : null,
    typeOnly: Boolean(clause?.isTypeOnly),
  };
}

function expressionName(node, typescript) {
  if (typescript.isIdentifier(node)) return node.text;
  if (typescript.isPropertyAccessExpression(node)) {
    const owner = expressionName(node.expression, typescript);
    return owner ? `${owner}.${node.name.text}` : node.name.text;
  }
  return undefined;
}

function operatorText(kind, typescript) {
  return typescript.tokenToString?.(kind) ?? typescript.SyntaxKind[kind];
}

function expressionFact(node, typescript) {
  if (typescript.isParenthesizedExpression(node)) return expressionFact(node.expression, typescript);
  if (typescript.isIdentifier(node)) return { kind: "identifier", name: node.text };
  if (typescript.isStringLiteral(node) || typescript.isNoSubstitutionTemplateLiteral(node)) {
    return { kind: "string", value: node.text };
  }
  if (typescript.isTemplateExpression(node)) {
    return {
      kind: "template",
      head: node.head.text,
      spans: node.templateSpans.map((span) => ({
        expression: expressionFact(span.expression, typescript),
        literal: span.literal.text,
      })),
    };
  }
  if (typescript.isNumericLiteral(node)) return { kind: "number", value: Number(node.text) };
  if (node.kind === typescript.SyntaxKind.TrueKeyword || node.kind === typescript.SyntaxKind.FalseKeyword) {
    return { kind: "boolean", value: node.kind === typescript.SyntaxKind.TrueKeyword };
  }
  if (node.kind === typescript.SyntaxKind.NullKeyword) return { kind: "null" };
  if (typescript.isPrefixUnaryExpression(node)) {
    return {
      kind: "unary",
      operator: operatorText(node.operator, typescript),
      operand: expressionFact(node.operand, typescript),
    };
  }
  if (typescript.isBinaryExpression(node)) {
    return {
      kind: "binary",
      operator: operatorText(node.operatorToken.kind, typescript),
      left: expressionFact(node.left, typescript),
      right: expressionFact(node.right, typescript),
    };
  }
  if (typescript.isPropertyAccessExpression(node)) {
    return {
      kind: "member",
      object: expressionFact(node.expression, typescript),
      property: node.name.text,
    };
  }
  if (typescript.isCallExpression(node)) {
    return {
      kind: "call",
      callee: expressionFact(node.expression, typescript),
      arguments: node.arguments.map((argument) => expressionFact(argument, typescript)),
    };
  }
  if (typescript.isArrowFunction(node)) {
    return {
      kind: "arrow",
      parameters: node.parameters.map((parameter) => typescript.isIdentifier(parameter.name) ? parameter.name.text : null),
      body: typescript.isBlock(node.body)
        ? { kind: "block" }
        : expressionFact(node.body, typescript),
    };
  }
  return { kind: "unsupported", syntaxKind: typescript.SyntaxKind[node.kind] };
}

function isDeferredExecutionBoundary(node, typescript) {
  return typescript.isFunctionDeclaration(node)
    || typescript.isFunctionExpression(node)
    || typescript.isArrowFunction(node)
    || typescript.isMethodDeclaration(node)
    || typescript.isGetAccessorDeclaration(node)
    || typescript.isSetAccessorDeclaration(node)
    || typescript.isConstructorDeclaration(node)
    || typescript.isClassDeclaration(node)
    || typescript.isClassExpression(node);
}

function containsThrow(node, typescript) {
  let found = false;
  const visit = (child) => {
    if (child !== node && isDeferredExecutionBoundary(child, typescript)) return;
    if (typescript.isThrowStatement(child)) {
      found = true;
      return;
    }
    if (!found) typescript.forEachChild(child, visit);
  };
  visit(node);
  return found;
}

function functionFact(name, node, exported, typescript) {
  const calls = [];
  const guards = [];
  const guardCalls = [];
  const guardStringLiterals = [];
  let throwCount = 0;
  const visitGuard = (child) => {
    if (typescript.isCallExpression(child)) {
      const called = expressionName(child.expression, typescript);
      if (called) guardCalls.push(called);
    }
    if (typescript.isStringLiteral(child) || typescript.isNoSubstitutionTemplateLiteral(child)) {
      guardStringLiterals.push(child.text);
    }
    typescript.forEachChild(child, visitGuard);
  };
  const visit = (child) => {
    if (isDeferredExecutionBoundary(child, typescript)) return;
    if (typescript.isCallExpression(child)) {
      const called = expressionName(child.expression, typescript);
      if (called) calls.push(called);
    }
    if (typescript.isIfStatement(child)) {
      visitGuard(child.expression);
      guards.push({
        condition: expressionFact(child.expression, typescript),
        consequenceThrows: containsThrow(child.thenStatement, typescript),
      });
    }
    if (typescript.isThrowStatement(child)) throwCount += 1;
    typescript.forEachChild(child, visit);
  };
  if (node.body) visit(node.body);
  return {
    name,
    exported,
    calls: [...new Set(calls)].sort(),
    guardCalls: [...new Set(guardCalls)].sort(),
    guardStringLiterals: [...new Set(guardStringLiterals)].sort(),
    guards,
    throwCount,
  };
}

function hasExportModifier(node, typescript) {
  return Boolean(node.modifiers?.some((modifier) => modifier.kind === typescript.SyntaxKind.ExportKeyword));
}

function typeScriptFacts(relativePath, source, typescript) {
  const sourceFile = typescript.createSourceFile(
    relativePath,
    source,
    typescript.ScriptTarget.Latest,
    true,
    scriptKind(relativePath, typescript),
  );
  const diagnostics = sourceFile.parseDiagnostics ?? [];
  if (diagnostics.length) {
    const detail = diagnostics
      .map((diagnostic) => typescript.flattenDiagnosticMessageText(diagnostic.messageText, "\n"))
      .join("; ");
    throw new Error(detail || "TypeScript parse failed");
  }

  const imports = sourceFile.statements
    .filter((statement) => typescript.isImportDeclaration(statement) && typescript.isStringLiteral(statement.moduleSpecifier))
    .map((statement) => importFact(statement, typescript));
  const functions = [];
  for (const statement of sourceFile.statements) {
    if (typescript.isFunctionDeclaration(statement) && statement.name) {
      functions.push(functionFact(statement.name.text, statement, hasExportModifier(statement, typescript), typescript));
      continue;
    }
    if (!typescript.isVariableStatement(statement)) continue;
    const exported = hasExportModifier(statement, typescript);
    for (const declaration of statement.declarationList.declarations) {
      if (
        typescript.isIdentifier(declaration.name)
        && declaration.initializer
        && (typescript.isArrowFunction(declaration.initializer) || typescript.isFunctionExpression(declaration.initializer))
      ) {
        functions.push(functionFact(declaration.name.text, declaration.initializer, exported, typescript));
      }
    }
  }

  return freeze({ path: relativePath, imports, functions });
}

function componentName(node) {
  if (typeof node.name === "string") return node.name;
  if (node.name && typeof node.name.name === "string") return node.name.name;
  return undefined;
}

function styleSelectorFact(selector) {
  if (selector.type === "TypeSelector") return { type: "tag", name: selector.name };
  if (selector.type === "ClassSelector") return { type: "class", name: selector.name };
  if (selector.type === "AttributeSelector") {
    return {
      type: "attribute",
      name: selector.name,
      matcher: selector.matcher ?? null,
      value: selector.value ?? null,
    };
  }
  return { type: "unsupported", syntaxKind: selector.type };
}

function styleRuleFacts(stylesheet) {
  return (stylesheet?.children ?? [])
    .filter((child) => child.type === "Rule")
    .map((rule) => ({
      selectors: (rule.prelude?.children ?? []).map((complexSelector) =>
        (complexSelector.children ?? []).flatMap((relativeSelector) =>
          (relativeSelector.selectors ?? []).map(styleSelectorFact))),
      declarations: (rule.block?.children ?? [])
        .filter((child) => child.type === "Declaration")
        .map(({ property, value }) => ({ property, value })),
    }));
}

function conditionIdentifiers(expression) {
  const names = new Set();
  const seen = new WeakSet();
  const visit = (value) => {
    if (!value || typeof value !== "object" || seen.has(value)) return;
    seen.add(value);
    if (value.type === "Identifier") names.add(value.name);
    for (const [key, child] of Object.entries(value)) {
      if (["loc", "metadata", "parent"].includes(key)) continue;
      if (Array.isArray(child)) child.forEach(visit);
      else visit(child);
    }
  };
  visit(expression);
  return [...names].sort();
}

function svelteImportFact(node) {
  const declarationTypeOnly = node.importKind === "type";
  const bindings = (node.specifiers ?? []).map((specifier) => ({
    imported: specifier.type === "ImportDefaultSpecifier"
      ? "default"
      : specifier.type === "ImportNamespaceSpecifier"
        ? "*"
        : specifier.imported?.name ?? specifier.imported?.value,
    local: specifier.local?.name,
    typeOnly: declarationTypeOnly || specifier.importKind === "type",
  }));
  return { source: node.source.value, bindings };
}

function svelteFacts(relativePath, source, svelte) {
  const ast = svelte.parse(source, { filename: relativePath, modern: true });
  const components = [];
  const seen = new WeakSet();
  const visit = (value, branches = []) => {
    if (!value || typeof value !== "object" || seen.has(value)) return;
    seen.add(value);
    if (value.type === "Component" || value.type === "SvelteComponent") {
      const name = componentName(value);
      if (name) {
        components.push({
          name,
          attributes: (value.attributes ?? [])
            .map((attribute) => typeof attribute.name === "string" ? attribute.name : undefined)
            .filter(Boolean)
            .sort(),
          ...(branches.length ? { branches } : {}),
        });
      }
    }
    if (value.type === "IfBlock") {
      const identifiers = conditionIdentifiers(value.test);
      visit(value.consequent, [...branches, { conditionIdentifiers: identifiers, polarity: "consequent" }]);
      visit(value.alternate, [...branches, { conditionIdentifiers: identifiers, polarity: "alternate" }]);
      return;
    }
    for (const child of Object.values(value)) {
      if (Array.isArray(child)) child.forEach((item) => visit(item, branches));
      else visit(child, branches);
    }
  };
  visit(ast);
  const imports = (ast.instance?.content?.body ?? [])
    .filter((node) => node.type === "ImportDeclaration" && typeof node.source?.value === "string")
    .map(svelteImportFact);
  return freeze({ path: relativePath, imports, components, styleRules: styleRuleFacts(ast.css) });
}

function defaultCargoMetadata(root) {
  return execFileSync("cargo", ["metadata", "--locked", "--format-version", "1", "--manifest-path", "src-tauri/Cargo.toml"], {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 256 * 1024 * 1024,
    windowsHide: true,
  });
}

export function createRepositoryIndex({
  root,
  readFile = (absolutePath) => readFileSync(absolutePath, "utf8"),
  ts = tsCompiler,
  svelte = svelteCompiler,
  loadCargoMetadata = () => defaultCargoMetadata(root),
}) {
  const repositoryRoot = path.resolve(root);
  const rawSourceCache = new Map();
  const jsonCache = new Map();
  const typeScriptCache = new Map();
  const svelteCache = new Map();
  let cargoMetadata;
  let cargoError;
  let cargoLoaded = false;

  const cachedRawSource = (selected) => {
    if (rawSourceCache.has(selected.relativePath)) {
      const cached = rawSourceCache.get(selected.relativePath);
      if (cached.error) throw cached.error;
      return cached.value;
    }
    try {
      const source = readFile(selected.absolutePath, "utf8");
      const value = Buffer.isBuffer(source) ? source.toString("utf8") : String(source);
      rawSourceCache.set(selected.relativePath, { value });
      return value;
    } catch (error) {
      const wrapped = errorFor(selected.relativePath, error);
      rawSourceCache.set(selected.relativePath, { error: wrapped });
      throw wrapped;
    }
  };

  const cachedSource = (cache, inputPath, parse) => {
    const selected = repositoryPath(repositoryRoot, inputPath);
    if (cache.has(selected.relativePath)) {
      const cached = cache.get(selected.relativePath);
      if (cached.error) throw cached.error;
      return cached.value;
    }
    let source;
    try {
      source = cachedRawSource(selected);
    } catch (error) {
      cache.set(selected.relativePath, { error });
      throw error;
    }
    try {
      const parsed = parse(selected.relativePath, source);
      cache.set(selected.relativePath, { value: parsed });
      return parsed;
    } catch (error) {
      const wrapped = errorFor(selected.relativePath, error);
      cache.set(selected.relativePath, { error: wrapped });
      throw wrapped;
    }
  };

  return freeze({
    getText(inputPath) {
      return cachedRawSource(repositoryPath(repositoryRoot, inputPath));
    },
    getJson(inputPath) {
      return cachedSource(jsonCache, inputPath, (_relativePath, source) => freeze(JSON.parse(source)));
    },
    getTypeScript(inputPath) {
      return cachedSource(typeScriptCache, inputPath, (relativePath, source) => typeScriptFacts(relativePath, source, ts));
    },
    getSvelte(inputPath) {
      return cachedSource(svelteCache, inputPath, (relativePath, source) => svelteFacts(relativePath, source, svelte));
    },
    getCargoMetadata() {
      if (cargoLoaded) {
        if (cargoError) throw cargoError;
        return cargoMetadata;
      }
      cargoLoaded = true;
      try {
        const loaded = loadCargoMetadata();
        cargoMetadata = freeze(typeof loaded === "string" || Buffer.isBuffer(loaded)
          ? JSON.parse(loaded.toString())
          : loaded);
        if (!cargoMetadata || typeof cargoMetadata !== "object") throw new Error("cargo metadata returned a non-object value");
        return cargoMetadata;
      } catch (error) {
        cargoError = errorFor(CARGO_MANIFEST_PATH, error);
        throw cargoError;
      }
    },
  });
}
