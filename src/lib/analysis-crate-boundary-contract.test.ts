import {
  existsSync,
  readFileSync,
  readdirSync,
} from "node:fs";
import { createHash } from "node:crypto";
import path from "node:path";
import { describe, expect, it } from "vitest";

const repoRoot = path.resolve(import.meta.dirname, "../..");
const appAnalysisRoot = path.join(repoRoot, "src-tauri/src/analysis");
const analysisCrateRoot = path.join(
  repoRoot,
  "src-tauri/crates/extractum-analysis/src",
);
const analysisCrateManifest = path.join(
  repoRoot,
  "src-tauri/crates/extractum-analysis/Cargo.toml",
);
const implementationPlanPath =
  "docs/superpowers/plans/2026-07-22-extractum-analysis-extraction.md";
const approvedSpecificationPath =
  "docs/superpowers/specs/2026-07-22-analysis-crate-boundary-design.md";

const normalizeNewlines = (value: string): string =>
  value.replace(/\r\n/g, "\n");
const normalized = (value: string): string =>
  value.replace(/\s+/g, " ").trim();
const canonicalRustSignature = (value: string): string =>
  normalized(value)
    .replace(/\(\s+/g, "(")
    .replace(/,\s*\)/g, ")")
    .replace(/\s+\)/g, ")")
    .replace(/\bmut\s+self\b/g, "self");
const read = (relativePath: string): string =>
  normalizeNewlines(
    readFileSync(path.join(repoRoot, relativePath), "utf8"),
  );
const readOptional = (relativePath: string): string =>
  existsSync(path.join(repoRoot, relativePath)) ? read(relativePath) : "";
const implementationPlan = read(implementationPlanPath);
const approvedSpecification = read(approvedSpecificationPath);
const extracted = existsSync(analysisCrateManifest);
const telegramCrateExtracted = existsSync(
  path.join(repoRoot, "src-tauri/crates/extractum-telegram/Cargo.toml"),
);

type SourceFile = {
  owner: "app" | "crate";
  relative: string;
  source: string;
};

type FrozenMove = {
  before: string;
  after: string;
};

type FrozenIdentity = {
  current: string;
  final: string;
  owner: "app" | "crate";
};

type DispositionRow = {
  index: number;
  current: string;
  disposition: "A" | "C" | "S";
};

type RustFunction = {
  name: string;
  visibility: string;
  declarationStart: number;
  signature: string;
  params: string;
  result: string;
  body: string;
  bodyStart: number;
  bodyEnd: number;
};

type ModuleGraphTest = {
  owner: "app" | "crate";
  relative: string;
  identity: string;
  name: string;
  attributes: string[];
};

type ModuleGraph = {
  tests: ModuleGraphTest[];
  reached: Set<string>;
  includedSources: SourceFile[];
};

function filesWithExtension(root: string, extension: string): string[] {
  if (!existsSync(root)) return [];
  return readdirSync(root, { withFileTypes: true }).flatMap((entry) => {
    const selected = path.join(root, entry.name);
    if (entry.isDirectory()) return filesWithExtension(selected, extension);
    return entry.isFile() && entry.name.endsWith(extension) ? [selected] : [];
  });
}

function regularFiles(root: string): string[] {
  if (!existsSync(root)) return [];
  return readdirSync(root, { withFileTypes: true }).flatMap((entry) => {
    const selected = path.join(root, entry.name);
    if (entry.isDirectory()) return regularFiles(selected);
    return entry.isFile() ? [selected] : [];
  });
}

function rustFiles(root: string): string[] {
  return filesWithExtension(root, ".rs");
}

function sourceFiles(
  root: string,
  owner: "app" | "crate",
): SourceFile[] {
  return rustFiles(root).map((file) => ({
    owner,
    relative: path.relative(root, file).replaceAll("\\", "/"),
    source: normalizeNewlines(readFileSync(file, "utf8")),
  }));
}

function exactInventory(
  actual: readonly string[],
  expected: readonly string[],
  label: string,
): void {
  const actualSorted = [...actual].sort();
  const expectedSorted = [...expected].sort();
  const duplicates = actualSorted.filter(
    (item, index) => item === actualSorted[index - 1],
  );
  expect(duplicates, `${label}: duplicate paths`).toEqual([]);
  expect(actualSorted, label).toEqual(expectedSorted);
}

function textBlockAfter(marker: string, language: string): string {
  const markerIndex = implementationPlan.indexOf(marker);
  if (markerIndex < 0) throw new Error(`missing plan marker: ${marker}`);
  const opening = implementationPlan.indexOf(`\`\`\`${language}`, markerIndex);
  if (opening < 0) {
    throw new Error(`missing ${language} block after plan marker: ${marker}`);
  }
  const bodyStart = implementationPlan.indexOf("\n", opening) + 1;
  const closing = implementationPlan.indexOf("\n```", bodyStart);
  if (bodyStart <= 0 || closing < 0) {
    throw new Error(`unterminated ${language} block after: ${marker}`);
  }
  return implementationPlan.slice(bodyStart, closing);
}

function planIdentifierList(marker: string): string[] {
  const values = textBlockAfter(marker, "text")
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => /^[A-Za-z_][A-Za-z0-9_]*$/.test(line));
  if (values.length === 0 || new Set(values).size !== values.length) {
    throw new Error(`missing or duplicate plan identifiers after: ${marker}`);
  }
  return values;
}

function frozenMoves(name: "wholeMoves" | "splitMoves"): FrozenMove[] {
  const start = implementationPlan.indexOf(`$${name} = @(`);
  if (start < 0) throw new Error(`missing frozen $${name} plan map`);
  const end = implementationPlan.indexOf("\n)", start);
  if (end < 0) throw new Error(`unterminated frozen $${name} plan map`);
  const entries = [
    ...implementationPlan.slice(start, end).matchAll(
      /@\('src-tauri\/src\/analysis\/([^']+)', 'src-tauri\/crates\/extractum-analysis\/src\/([^']+)'\)/g,
    ),
  ].map((match) => ({ before: match[1], after: match[2] }));
  if (entries.length === 0) throw new Error(`empty frozen $${name} plan map`);
  if (
    new Set(entries.map(({ before }) => before)).size !== entries.length
    || new Set(entries.map(({ after }) => after)).size !== entries.length
  ) {
    throw new Error(`duplicate path in frozen $${name} plan map`);
  }
  return entries;
}

const wholeMoves = frozenMoves("wholeMoves");
const splitMoves = frozenMoves("splitMoves");
const allMoves = [...wholeMoves, ...splitMoves];

function dispositionRows(): DispositionRow[] {
  const start = implementationPlan.indexOf("## Frozen 54-File Disposition Map");
  const end = implementationPlan.indexOf("## Frozen 143-Test Ownership", start);
  if (start < 0 || end <= start) throw new Error("missing frozen disposition map");
  const rows = [
    ...implementationPlan.slice(start, end).matchAll(
      /^\|\s*(\d+)\s*\|\s*`([^`]+)`\s*\|\s*[\d,]+\s*\|\s*([ACS])(?:\s|:|\|)/gm,
    ),
  ].map((match) => ({
    index: Number(match[1]),
    current: match[2],
    disposition: match[3] as "A" | "C" | "S",
  }));
  if (rows.length !== 54) {
    throw new Error(`frozen disposition count drifted: ${rows.length}`);
  }
  expect(rows.map(({ index }) => index)).toEqual(
    Array.from({ length: 54 }, (_, index) => index + 1),
  );
  if (new Set(rows.map(({ current }) => current)).size !== 54) {
    throw new Error("frozen disposition map contains duplicate current paths");
  }
  const counts = rows.reduce(
    (result, row) => {
      result[row.disposition] += 1;
      return result;
    },
    { A: 0, C: 0, S: 0 },
  );
  expect(counts).toEqual({ A: 14, C: 20, S: 20 });
  return rows;
}

const dispositions = dispositionRows();
const retainedAppPaths = [
  ...dispositions
    .filter(({ disposition }) => disposition !== "C")
    .map(({ current }) => current),
  "tests_application.rs",
].sort();
const preMoveAppPaths = [
  ...retainedAppPaths,
  ...allMoves.map(({ before }) => before),
].sort();
const finalCratePaths = [
  ...allMoves.map(({ after }) => after),
  "lib.rs",
].sort();

function assertFrozenTopology(): void {
  expect(wholeMoves).toHaveLength(23);
  expect(splitMoves).toHaveLength(21);
  expect(allMoves).toHaveLength(44);
  expect(retainedAppPaths).toHaveLength(35);
  expect(finalCratePaths).toHaveLength(45);
  expect(
    splitMoves.filter(({ before }) => before === "domain_portable.rs"),
    "baseline mod.rs must yield crate domain.rs",
  ).toEqual([{ before: "domain_portable.rs", after: "domain.rs" }]);
  expect(
    splitMoves.filter(({ before }) => before === "tests_portable.rs"),
    "baseline mod.rs must yield crate tests.rs",
  ).toEqual([{ before: "tests_portable.rs", after: "tests.rs" }]);
  for (const added of [
    "tests_application.rs",
    "report/tests/corpus_port.rs",
    "report/tests/runtime.rs",
    "test_schema.rs",
  ]) {
    expect(
      preMoveAppPaths.includes(added),
      `missing separately counted plan-added source ${added}`,
    ).toBe(true);
  }

  if (!extracted) {
    exactInventory(
      sourceFiles(appAnalysisRoot, "app").map(({ relative }) => relative),
      preMoveAppPaths,
      "prepared pre-move app analysis topology",
    );
    expect(sourceFiles(analysisCrateRoot, "crate")).toEqual([]);
    for (const move of allMoves) {
      expect(
        existsSync(path.join(appAnalysisRoot, move.before)),
        `missing prepared move source ${move.before}`,
      ).toBe(true);
      expect(
        existsSync(path.join(analysisCrateRoot, move.after)),
        `crate destination exists before extraction: ${move.after}`,
      ).toBe(false);
    }
    return;
  }

  exactInventory(
    sourceFiles(appAnalysisRoot, "app").map(({ relative }) => relative),
    retainedAppPaths,
    "post-move app analysis topology",
  );
  exactInventory(
    sourceFiles(analysisCrateRoot, "crate").map(({ relative }) => relative),
    finalCratePaths,
    "post-move analysis crate topology",
  );
  for (const move of allMoves) {
    expect(
      existsSync(path.join(appAnalysisRoot, move.before)),
      `prepared source was copied instead of moved: ${move.before}`,
    ).toBe(false);
    expect(
      existsSync(path.join(analysisCrateRoot, move.after)),
      `missing mechanical destination: ${move.after}`,
    ).toBe(true);
  }
}

function parseAppendix(): FrozenIdentity[] {
  const marker = "## Appendix A: Frozen 143-Test Baseline";
  const start = approvedSpecification.indexOf(marker);
  if (start < 0) throw new Error("missing Appendix A heading");
  const appendix = approvedSpecification.slice(start);
  const ownerHeadings = [
    {
      marker: "### `extractum-analysis` — 95 identities",
      owner: "crate" as const,
      count: 95,
    },
    {
      marker: "### `extractum` — 48 identities",
      owner: "app" as const,
      count: 48,
    },
  ];
  const ownerStarts = ownerHeadings.map(({ marker: heading }) => {
    const index = appendix.indexOf(heading);
    if (index < 0) throw new Error(`missing Appendix A owner heading: ${heading}`);
    return index;
  });
  if (ownerStarts[0] >= ownerStarts[1]) {
    throw new Error("Appendix A owner headings are out of order");
  }

  const identities: FrozenIdentity[] = [];
  for (let ownerIndex = 0; ownerIndex < ownerHeadings.length; ownerIndex += 1) {
    const owner = ownerHeadings[ownerIndex];
    const section = appendix.slice(
      ownerStarts[ownerIndex],
      ownerStarts[ownerIndex + 1],
    );
    const headings = [...section.matchAll(/^#### .+ \((\d+)\)$/gm)];
    if (headings.length === 0) {
      throw new Error(`Appendix A ${owner.owner} has no identity groups`);
    }
    const ownerIdentities: FrozenIdentity[] = [];
    for (let index = 0; index < headings.length; index += 1) {
      const heading = headings[index];
      if (heading.index === undefined) {
        throw new Error("Appendix A group heading has no index");
      }
      const body = section.slice(
        heading.index + heading[0].length,
        headings[index + 1]?.index ?? section.length,
      );
      const prefixMatch = body.match(
        /^Current (?:and final )?prefix:\s*`([^`]+)`(?:\. Final prefix:\s*\n?`([^`]+)`)?\.$/m,
      );
      if (!prefixMatch) {
        throw new Error(`missing or malformed prefixes after ${heading[0]}`);
      }
      const currentPrefix = prefixMatch[1];
      const finalPrefix = prefixMatch[2] ?? currentPrefix;
      if (!currentPrefix.startsWith("analysis::")) {
        throw new Error(`unexpected current prefix: ${currentPrefix}`);
      }
      if (owner.owner === "crate" && finalPrefix.startsWith("analysis::")) {
        throw new Error(`unexpected crate final prefix: ${finalPrefix}`);
      }
      if (owner.owner === "app" && currentPrefix !== finalPrefix) {
        throw new Error(`unexpected app final prefix: ${finalPrefix}`);
      }
      const names = body
        .split("\n")
        .filter((line) => line.startsWith("- "))
        .map((line) => {
          const match = line.match(/^- `([A-Za-z0-9_]+)`$/);
          if (!match) throw new Error(`malformed Appendix A bullet: ${line}`);
          return match[1];
        });
      if (names.length !== Number(heading[1])) {
        throw new Error(
          `${heading[0]} declares ${heading[1]} identities but has ${names.length}`,
        );
      }
      ownerIdentities.push(
        ...names.map((name) => ({
          current: `${currentPrefix}::${name}`,
          final: `${finalPrefix}::${name}`,
          owner: owner.owner,
        })),
      );
    }
    if (ownerIdentities.length !== owner.count) {
      throw new Error(
        `Appendix A ${owner.owner} count drift: ${ownerIdentities.length}`,
      );
    }
    identities.push(...ownerIdentities);
  }
  if (identities.length !== 143) {
    throw new Error(`Appendix A total drift: ${identities.length}`);
  }
  if (new Set(identities.map(({ current }) => current)).size !== 143) {
    throw new Error("Appendix A contains duplicate current identities");
  }
  if (new Set(identities.map(({ final }) => final)).size !== 143) {
    throw new Error("Appendix A contains duplicate final identities");
  }
  return identities;
}

const appendix = parseAppendix();

function closingDelimiter(
  source: string,
  open: number,
  left: string,
  right: string,
): number {
  let depth = 0;
  for (let index = open; index < source.length; index += 1) {
    if (source[index] === left) depth += 1;
    if (source[index] === right) depth -= 1;
    if (depth === 0) return index;
  }
  throw new Error(`unclosed ${left}${right} delimiter`);
}

const rustMaskCache = new Map<string, { code?: string; comments?: string }>();

function maskRustNonCode(source: string, maskStrings: boolean): string {
  const cached = rustMaskCache.get(source);
  const cachedValue = maskStrings ? cached?.code : cached?.comments;
  if (cachedValue !== undefined) return cachedValue;
  const masked = source.split("");
  const blank = (start: number, end: number): void => {
    for (let index = start; index < end; index += 1) {
      if (masked[index] !== "\n") masked[index] = " ";
    }
  };
  const quotedEnd = (start: number, quote: string): number => {
    for (let index = start + 1; index < source.length; index += 1) {
      if (source[index] === "\\") index += 1;
      else if (source[index] === quote) return index + 1;
    }
    return source.length;
  };
  for (let index = 0; index < source.length;) {
    if (source.startsWith("//", index)) {
      const newline = source.indexOf("\n", index + 2);
      const end = newline < 0 ? source.length : newline;
      blank(index, end);
      index = end;
      continue;
    }
    if (source.startsWith("/*", index)) {
      const start = index;
      let depth = 1;
      index += 2;
      while (index < source.length && depth > 0) {
        if (source.startsWith("/*", index)) {
          depth += 1;
          index += 2;
        } else if (source.startsWith("*/", index)) {
          depth -= 1;
          index += 2;
        } else {
          index += 1;
        }
      }
      blank(start, index);
      continue;
    }
    const raw = source.slice(index).match(/^(?:br|rb|r)(#{0,255})"/);
    if (raw) {
      const closing = `"${raw[1]}`;
      const end = source.indexOf(closing, index + raw[0].length);
      const tokenEnd = end < 0 ? source.length : end + closing.length;
      if (maskStrings) blank(index, tokenEnd);
      index = tokenEnd;
      continue;
    }
    const quote = source[index] === '"'
      ? index
      : source.startsWith('b"', index)
        ? index + 1
        : -1;
    const character = source[index] === "'"
      && /^'(?:\\.|[^\\'\n])'/.test(source.slice(index));
    if (quote >= 0 || character) {
      const start = index;
      const actualQuote = quote >= 0 ? quote : index;
      const end = quotedEnd(actualQuote, source[actualQuote]);
      if (maskStrings) blank(start, end);
      index = end;
      continue;
    }
    index += 1;
  }
  const result = masked.join("");
  rustMaskCache.set(source, {
    ...cached,
    [maskStrings ? "code" : "comments"]: result,
  });
  return result;
}

function productionRust(source: string): string {
  const masked = source.split("");
  const blank = (start: number, end: number): void => {
    for (let index = start; index < end; index += 1) {
      if (masked[index] !== "\n") masked[index] = " ";
    }
  };
  const disabledItemEnd = (syntax: string, start: number): number => {
    let cursor = start;
    const skipWhitespace = (): void => {
      while (cursor < syntax.length && /\s/.test(syntax[cursor])) cursor += 1;
    };
    skipWhitespace();
    while (syntax[cursor] === "#") {
      let bracket = cursor + 1;
      while (bracket < syntax.length && /\s/.test(syntax[bracket])) {
        bracket += 1;
      }
      if (syntax[bracket] !== "[") {
        throw new Error("malformed attribute after cfg(test/dev)");
      }
      cursor = closingDelimiter(syntax, bracket, "[", "]") + 1;
      skipWhitespace();
    }
    if (cursor >= syntax.length) {
      throw new Error("cfg(test/dev) attribute has no Rust item");
    }

    const bracedItem = /^(?:(?:pub(?:\s*\([^)]*\))?|async|const|unsafe|default)\s+)*(?:fn|mod|impl|trait|struct|enum|union)\b|^extern(?:\s+[br#"]+)?\s*\{/
      .test(syntax.slice(cursor));
    const commaTerminated =
      /^(?:(?:pub(?:\s*\([^)]*\))?)\s+)?(?:r#)?[A-Za-z_][A-Za-z0-9_]*\s*:/
        .test(syntax.slice(cursor))
      || /^(?:r#)?[A-Za-z_][A-Za-z0-9_]*\s*,/
        .test(syntax.slice(cursor))
      || /^[A-Z][A-Za-z0-9_]*(?:\s*[\(\{])?/.test(syntax.slice(cursor));
    const stack: string[] = [];
    const pairs: Record<string, string> = {
      "(": ")",
      "[": "]",
      "{": "}",
    };
    let topLevelMatchArm = false;
    let matchArmRhsStart: number | undefined;
    let bareBlockEndCandidate: number | undefined;
    for (let index = cursor; index < syntax.length; index += 1) {
      const token = syntax[index];
      if (
        stack.length === 0
        && token === "="
        && syntax[index + 1] === ">"
      ) {
        if (topLevelMatchArm) {
          if (bareBlockEndCandidate !== undefined) {
            return bareBlockEndCandidate;
          }
          throw new Error(
            "cfg(test/dev) compound match arm has no terminating comma",
          );
        }
        topLevelMatchArm = true;
        matchArmRhsStart = index + 2;
        index += 1;
        continue;
      }
      if (pairs[token]) {
        if (token === "{" && stack.length === 0) {
          if (bracedItem && !topLevelMatchArm) {
            return closingDelimiter(syntax, index, "{", "}") + 1;
          }
          if (
            topLevelMatchArm
            && bareBlockEndCandidate === undefined
            && matchArmRhsStart !== undefined
            && syntax.slice(matchArmRhsStart, index).trim().length === 0
          ) {
            bareBlockEndCandidate =
              closingDelimiter(syntax, index, "{", "}") + 1;
          }
        }
        stack.push(pairs[token]);
        continue;
      }
      if (stack.at(-1) === token) {
        stack.pop();
        continue;
      }
      if (stack.length === 0 && token === ";") return index + 1;
      if (
        stack.length === 0
        && token === ","
        && (commaTerminated || topLevelMatchArm)
      ) {
        return index + 1;
      }
      if (stack.length === 0 && token === "}" && topLevelMatchArm) {
        if (bareBlockEndCandidate !== undefined) {
          return bareBlockEndCandidate;
        }
        throw new Error(
          "cfg(test/dev) compound match arm has no terminating comma",
        );
      }
      if (stack.length === 0 && token === "{") {
        return closingDelimiter(syntax, index, "{", "}") + 1;
      }
    }
    throw new Error(
      `unterminated cfg(test/dev) Rust item: ${normalized(
        syntax.slice(cursor, cursor + 160),
      )}`,
    );
  };
  let syntax = maskRustNonCode(source, true);
  const cfg = /#\s*\[\s*cfg\s*\(\s*(?:test|dev)\s*\)\s*\]/g;
  for (;;) {
    const match = cfg.exec(syntax);
    if (!match || match.index === undefined) break;
    const end = disabledItemEnd(syntax, cfg.lastIndex);
    if (end <= match.index || !Number.isFinite(end)) {
      throw new Error("unterminated cfg(test/dev) item");
    }
    blank(match.index, end);
    syntax = maskRustNonCode(masked.join(""), true);
    cfg.lastIndex = 0;
  }
  return masked.join("");
}

function rustFunctions(source: string): RustFunction[] {
  const syntax = maskRustNonCode(source, true);
  const functions: RustFunction[] = [];
  for (const match of syntax.matchAll(
    /\b(?:(pub(?:\s*\([^)]*\))?)\s+)?(?:const\s+)?(?:async\s+)?(?:unsafe\s+)?fn\s+((?:r#)?[A-Za-z_][A-Za-z0-9_]*)\s*(?:<[^>{}]*>)?\s*\(/g,
  )) {
    if (match.index === undefined) throw new Error("Rust function has no offset");
    const paramsOpen = match.index + match[0].lastIndexOf("(");
    const paramsClose = closingDelimiter(syntax, paramsOpen, "(", ")");
    const bodyOpen = syntax.indexOf("{", paramsClose);
    const declarationEnd = syntax.indexOf(";", paramsClose);
    if (bodyOpen < 0 || (declarationEnd >= 0 && declarationEnd < bodyOpen)) {
      continue;
    }
    const bodyClose = closingDelimiter(syntax, bodyOpen, "{", "}");
    functions.push({
      name: match[2].replace(/^r#/, ""),
      visibility: match[1] ?? "",
      declarationStart: match.index,
      signature: normalized(source.slice(match.index, bodyOpen)),
      params: source.slice(paramsOpen + 1, paramsClose),
      result: source.slice(paramsClose + 1, bodyOpen),
      body: source.slice(bodyOpen + 1, bodyClose),
      bodyStart: bodyOpen,
      bodyEnd: bodyClose,
    });
  }
  return functions;
}

function rustMacroDefinitions(source: string): string[] {
  const syntax = maskRustNonCode(source, true);
  return [...syntax.matchAll(
    /\bmacro_rules\s*!\s*([A-Za-z_][A-Za-z0-9_]*)\s*\{/g,
  )].map((definition) => {
    if (definition.index === undefined) {
      throw new Error("Rust macro_rules definition has no offset");
    }
    const open = definition.index + definition[0].lastIndexOf("{");
    closingDelimiter(syntax, open, "{", "}");
    return definition[1];
  });
}

function splitTopLevel(value: string, delimiter = ","): string[] {
  const parts: string[] = [];
  let start = 0;
  const stack: string[] = [];
  const pairs: Record<string, string> = { "(": ")", "[": "]", "{": "}", "<": ">" };
  for (let index = 0; index < value.length; index += 1) {
    const token = value[index];
    if (pairs[token]) stack.push(pairs[token]);
    else if (stack.at(-1) === token) stack.pop();
    else if (token === delimiter && stack.length === 0) {
      parts.push(value.slice(start, index));
      start = index + 1;
    }
  }
  parts.push(value.slice(start));
  return parts.map((part) => part.trim()).filter(Boolean);
}

function inlineModuleRegions(source: string): Array<{
  name: string;
  open: number;
  close: number;
  attributes: string;
}> {
  const syntax = maskRustNonCode(source, true);
  const commentsOnly = maskRustNonCode(source, false);
  return [...syntax.matchAll(
    /\bmod\s+([A-Za-z_][A-Za-z0-9_]*)\s*\{/g,
  )].map((match) => {
    if (match.index === undefined) {
      throw new Error("inline Rust module has no offset");
    }
    const open = match.index + match[0].lastIndexOf("{");
    return {
      name: match[1],
      open,
      close: closingDelimiter(syntax, open, "{", "}"),
      attributes: immediateRustAttributes(commentsOnly, match.index),
    };
  });
}

function rustBraceRegions(source: string): Array<{
  open: number;
  close: number;
}> {
  const syntax = maskRustNonCode(source, true);
  const stack: number[] = [];
  const regions: Array<{ open: number; close: number }> = [];
  for (let index = 0; index < syntax.length; index += 1) {
    if (syntax[index] === "{") {
      stack.push(index);
    } else if (syntax[index] === "}") {
      const open = stack.pop();
      if (open === undefined) throw new Error("unmatched Rust closing brace");
      regions.push({ open, close: index });
    }
  }
  if (stack.length !== 0) throw new Error("unmatched Rust opening brace");
  return regions;
}

function graphTestDeclarations(source: string): Array<{
  name: string;
  offset: number;
  attributes: string;
}> {
  const commentsOnly = maskRustNonCode(source, false);
  const moduleOpens = new Set(
    inlineModuleRegions(source).map(({ open }) => open),
  );
  const braces = rustBraceRegions(source);
  return rustFunctions(source).flatMap((fn) => {
    const attributes = immediateRustAttributes(
      commentsOnly,
      fn.declarationStart,
    );
    const isTest = /#\s*\[\s*(?:(?:tokio|sqlx)\s*::\s*)?test\b/.test(
      attributes,
    );
    const itemAncestry = braces.filter(({ open, close }) =>
      open < fn.declarationStart && fn.declarationStart < close);
    return isTest
        && itemAncestry.every(({ open }) => moduleOpens.has(open))
      ? [{
        name: fn.name,
        offset: fn.declarationStart,
        attributes,
      }]
      : [];
  });
}

function normalizedRelative(relative: string): string {
  return path.posix.normalize(relative.replaceAll("\\", "/"))
    .replace(/^\.\//, "");
}

function rustPathAttribute(attributes: string): string | undefined {
  const matches = [...attributes.matchAll(
    /#\s*\[\s*path\s*=\s*([\s\S]*?)\]/g,
  )];
  if (matches.length === 0) return undefined;
  if (matches.length !== 1) {
    throw new Error("Rust module has multiple path attributes");
  }
  const value = exactRustPathLiteral(matches[0][1].trim());
  if (value === undefined) {
    throw new Error(
      `unsupported Rust path attribute literal: ${normalized(matches[0][1])}`,
    );
  }
  return value;
}

function analysisModuleGraph(
  root: string,
  owner: "app" | "crate",
): ModuleGraph {
  if (!existsSync(root)) {
    return { tests: [], reached: new Set(), includedSources: [] };
  }
  const byRelative = new Map(
    sourceFiles(root, owner).map((file) => [
      normalizedRelative(file.relative),
      { ...file, relative: normalizedRelative(file.relative) },
    ]),
  );
  type GraphState = {
    relative: string;
    prefix: string[];
    moduleDir: string;
    inheritedAttributes: string[];
  };
  const queue: GraphState[] = [{
    relative: owner === "app" ? "mod.rs" : "lib.rs",
    prefix: owner === "app" ? ["analysis"] : [],
    moduleDir: "",
    inheritedAttributes: [],
  }];
  const visited = new Set<string>();
  const reached = new Set<string>();
  const tests: ModuleGraphTest[] = [];
  const includedSources: SourceFile[] = [];

  const enqueue = (state: GraphState): void => {
    const relative = normalizedRelative(state.relative);
    if (relative.startsWith("../") || path.posix.isAbsolute(relative)) {
      throw new Error(`analysis module graph escapes its package root: ${relative}`);
    }
    if (!byRelative.has(relative)) {
      const absolute = path.join(root, ...relative.split("/"));
      if (!existsSync(absolute)) {
        throw new Error(`analysis module graph reaches missing source ${relative}`);
      }
      const source = normalizeNewlines(readFileSync(absolute, "utf8"));
      const included = { owner, relative, source };
      byRelative.set(relative, included);
      includedSources.push(included);
    }
    queue.push({ ...state, relative });
  };

  for (let queueIndex = 0; queueIndex < queue.length; queueIndex += 1) {
    const state = queue[queueIndex];
    const key = [
      state.relative,
      state.prefix.join("::"),
      state.moduleDir,
    ].join("\0");
    if (visited.has(key)) {
      throw new Error(`duplicate analysis module/include instance: ${key}`);
    }
    visited.add(key);
    reached.add(state.relative);
    const file = byRelative.get(state.relative);
    if (!file) throw new Error(`missing queued analysis source ${state.relative}`);
    const syntax = maskRustNonCode(file.source, true);
    const commentsOnly = maskRustNonCode(file.source, false);
    const inline = inlineModuleRegions(file.source);
    const inlineOpens = new Set(inline.map(({ open }) => open));
    const braces = rustBraceRegions(file.source);
    const ancestryAt = (offset: number) =>
      inline
        .filter(({ open, close }) => offset > open && offset < close)
        .sort((left, right) => left.open - right.open);
    const isModuleScopeAt = (offset: number): boolean =>
      braces
        .filter(({ open, close }) => open < offset && offset < close)
        .every(({ open }) => inlineOpens.has(open));

    for (const test of graphTestDeclarations(file.source)) {
      const ancestry = ancestryAt(test.offset);
      const prefix = [
        ...state.prefix,
        ...ancestry.map(({ name }) => name),
      ];
      tests.push({
        owner,
        relative: file.relative,
        identity: [...prefix, test.name].join("::"),
        name: test.name,
        attributes: [
          ...state.inheritedAttributes,
          ...ancestry.map(({ attributes }) => attributes),
          test.attributes,
        ].filter(Boolean),
      });
    }

    for (const declaration of syntax.matchAll(
      /\bmod\s+([A-Za-z_][A-Za-z0-9_]*)\s*;/g,
    )) {
      if (declaration.index === undefined) {
        throw new Error("external Rust module has no offset");
      }
      if (!isModuleScopeAt(declaration.index)) continue;
      const name = declaration[1];
      const ancestry = ancestryAt(declaration.index);
      const moduleDir = normalizedRelative(
        path.posix.join(
          state.moduleDir,
          ...ancestry.map(({ name: ancestor }) => ancestor),
        ),
      ).replace(/^\.$/, "");
      const attributes = immediateRustAttributes(
        commentsOnly,
        declaration.index,
      );
      const override = rustPathAttribute(attributes);
      let relative: string;
      let childModuleDir: string;
      if (override) {
        relative = normalizedRelative(
          path.posix.join(path.posix.dirname(state.relative), override),
        );
        childModuleDir = path.posix.basename(relative) === "mod.rs"
          ? path.posix.dirname(relative)
          : path.posix.join(
            path.posix.dirname(relative),
            path.posix.basename(relative, path.posix.extname(relative)),
          );
      } else {
        const candidates = [
          path.posix.join(moduleDir, `${name}.rs`),
          path.posix.join(moduleDir, name, "mod.rs"),
        ].map(normalizedRelative)
          .filter((candidate) => byRelative.has(candidate));
        if (candidates.length !== 1) {
          throw new Error(
            `${state.relative} mod ${name} resolves to ${candidates.length} sources`,
          );
        }
        relative = candidates[0];
        childModuleDir = path.posix.join(moduleDir, name);
      }
      enqueue({
        relative,
        prefix: [
          ...state.prefix,
          ...ancestry.map(({ name: ancestor }) => ancestor),
          name,
        ],
        moduleDir: normalizedRelative(childModuleDir).replace(/^\.$/, ""),
        inheritedAttributes: [
          ...state.inheritedAttributes,
          ...ancestry.map(({ attributes: value }) => value),
          attributes,
        ].filter(Boolean),
      });
    }

    for (const include of syntax.matchAll(/\binclude\s*!\s*\(/g)) {
      if (include.index === undefined) {
        throw new Error("Rust include! has no offset");
      }
      if (!isModuleScopeAt(include.index)) continue;
      const open = include.index + include[0].lastIndexOf("(");
      const close = closingDelimiter(syntax, open, "(", ")");
      const expression = file.source.slice(open + 1, close).trim();
      const literal = expression.match(/^"((?:\\.|[^"\\])*)"$/)?.[1];
      if (literal === undefined) {
        throw new Error(`${state.relative} has nonliteral include!: ${expression}`);
      }
      const ancestry = ancestryAt(include.index);
      enqueue({
        relative: path.posix.join(
          path.posix.dirname(state.relative),
          literal.replaceAll("\\", "/"),
        ),
        prefix: [
          ...state.prefix,
          ...ancestry.map(({ name }) => name),
        ],
        moduleDir: normalizedRelative(
          path.posix.join(
            state.moduleDir,
            ...ancestry.map(({ name }) => name),
          ),
        ).replace(/^\.$/, ""),
        inheritedAttributes: [
          ...state.inheritedAttributes,
          ...ancestry.map(({ attributes }) => attributes),
        ].filter(Boolean),
      });
    }
  }

  return { tests, reached, includedSources };
}

function cfgRestrictions(
  attributes: readonly string[],
  identity: string,
): string[] {
  const restrictions: string[] = [];
  for (const block of attributes) {
    if (/#\s*\[\s*ignore\b/.test(block)) restrictions.push("ignore");
    if (/#\s*\[\s*cfg_attr\b/.test(block)) restrictions.push("cfg_attr");
    const syntax = maskRustNonCode(block, true);
    for (const match of syntax.matchAll(
      /#\s*\[\s*cfg\s*\(/g,
    )) {
      if (match.index === undefined) {
        throw new Error("cfg attribute has no offset");
      }
      const open = match.index + match[0].lastIndexOf("(");
      const close = closingDelimiter(syntax, open, "(", ")");
      const expression = normalized(block.slice(open + 1, close));
      if (
        expression !== "test"
        && !(
          expression === "dev"
          && identity.startsWith("analysis::fixtures::")
        )
      ) {
        restrictions.push(`cfg(${expression})`);
      }
    }
  }
  return restrictions;
}

function selectedMoveSource(relative: string): string {
  const move = allMoves.find(({ before, after }) =>
    before === relative || after === relative);
  if (!move) throw new Error(`path is not a frozen move: ${relative}`);
  const selected = extracted
    ? path.join(analysisCrateRoot, move.after)
    : path.join(appAnalysisRoot, move.before);
  if (!existsSync(selected)) {
    throw new Error(`selected frozen move source is missing: ${relative}`);
  }
  return normalizeNewlines(readFileSync(selected, "utf8"));
}

function selectedMoveFile(relative: string): SourceFile {
  const move = allMoves.find(({ before, after }) =>
    before === relative || after === relative);
  if (!move) throw new Error(`path is not a frozen move: ${relative}`);
  return {
    owner: extracted ? "crate" : "app",
    relative: extracted ? move.after : move.before,
    source: selectedMoveSource(relative),
  };
}

function planPublicApi(): { types: string[]; functions: string[] } {
  const types = planIdentifierList(
    "The exhaustive public root type allowlist is:",
  );
  const functions = planIdentifierList(
    "The exhaustive public root function allowlist is:",
  );
  if (types.length < 40 || functions.length < 30) {
    throw new Error("public API plan blocks are unexpectedly small");
  }
  return { types, functions };
}

const publicApi = planPublicApi();

function useTreeExports(tree: string): string[] {
  const value = tree.trim();
  const brace = value.indexOf("{");
  if (brace >= 0) {
    const close = closingDelimiter(value, brace, "{", "}");
    if (value.slice(close + 1).trim() !== "") {
      throw new Error(`unparsed pub use suffix: ${value}`);
    }
    return splitTopLevel(value.slice(brace + 1, close))
      .flatMap(useTreeExports);
  }
  const alias = value.match(/\bas\s+([A-Za-z_][A-Za-z0-9_]*)$/)?.[1];
  if (alias) return [alias];
  const leaf = value.match(/([A-Za-z_][A-Za-z0-9_]*)\s*$/)?.[1];
  if (!leaf || leaf === "self") throw new Error(`unparseable pub use leaf: ${value}`);
  return [leaf];
}

function useTreeMappings(
  tree: string,
  inherited: readonly string[] = [],
): Array<{ source: string; exported: string }> {
  const value = tree.trim();
  const brace = value.indexOf("{");
  if (brace >= 0) {
    const close = closingDelimiter(value, brace, "{", "}");
    if (value.slice(close + 1).trim() !== "") {
      throw new Error(`unparsed pub use suffix: ${value}`);
    }
    const prefix = value.slice(0, brace).trim().replace(/::$/, "");
    const segments = prefix
      ? prefix.split(/\s*::\s*/).filter((segment) =>
        segment && segment !== "self" && segment !== "crate")
      : [];
    return splitTopLevel(value.slice(brace + 1, close))
      .flatMap((child) =>
        useTreeMappings(child, [...inherited, ...segments]));
  }
  const alias = value.match(/\bas\s+([A-Za-z_][A-Za-z0-9_]*)$/)?.[1];
  const withoutAlias = value.replace(
    /\s+as\s+[A-Za-z_][A-Za-z0-9_]*$/,
    "",
  );
  const segments = withoutAlias.split(/\s*::\s*/)
    .filter((segment) =>
      segment && segment !== "self" && segment !== "crate");
  const full = [...inherited, ...segments];
  const leaf = full.at(-1);
  if (!leaf) throw new Error(`unparseable pub use mapping: ${value}`);
  return [{ source: full.join("::"), exported: alias ?? leaf }];
}

function publicRootMappings(source: string): Array<{
  source: string;
  exported: string;
}> {
  const syntax = maskRustNonCode(source, true);
  return [...syntax.matchAll(/\bpub\s+use\s+([\s\S]*?);/g)]
    .flatMap((match) => useTreeMappings(match[1]));
}

function publicRootExports(source: string): string[] {
  const syntax = maskRustNonCode(source, true);
  const exports: string[] = [];
  for (const match of syntax.matchAll(/\bpub\s+use\s+([\s\S]*?);/g)) {
    exports.push(...useTreeExports(match[1]));
  }
  return exports.sort();
}

function structBody(source: string, name: string): string {
  const syntax = maskRustNonCode(source, true);
  const matches = [...syntax.matchAll(
    new RegExp(`\\b(?:pub\\s+)?struct\\s+${name}\\b[^;{]*\\{`, "g"),
  )];
  if (matches.length !== 1 || matches[0].index === undefined) {
    throw new Error(`expected exactly one struct ${name}; found ${matches.length}`);
  }
  const open = matches[0].index + matches[0][0].lastIndexOf("{");
  const close = closingDelimiter(syntax, open, "{", "}");
  return source.slice(open + 1, close);
}

function structFields(source: string, name: string): Array<{
  visibility: string;
  name: string;
  type: string;
}> {
  return splitTopLevel(structBody(source, name)).map((field) => {
    const withoutAttributes = field.replace(
      /^(?:#\s*\[[\s\S]*?\]\s*)+/,
      "",
    ).trim();
    const match = withoutAttributes.match(
      /^(?:(pub(?:\([^)]*\))?)\s+)?(r#[A-Za-z_][A-Za-z0-9_]*|[A-Za-z_][A-Za-z0-9_]*)\s*:\s*([\s\S]+)$/,
    );
    if (!match) {
      throw new Error(`unparsed ${name} field: ${withoutAttributes}`);
    }
    return {
      visibility: match[1] ?? "",
      name: match[2],
      type: normalized(match[3]),
    };
  });
}

function enumBody(source: string, name: string): string {
  const syntax = maskRustNonCode(source, true);
  const matches = [...syntax.matchAll(
    new RegExp(`\\b(?:pub\\s+)?enum\\s+${name}\\b[^;{]*\\{`, "g"),
  )];
  if (matches.length !== 1 || matches[0].index === undefined) {
    throw new Error(`expected exactly one enum ${name}; found ${matches.length}`);
  }
  const open = matches[0].index + matches[0][0].lastIndexOf("{");
  const close = closingDelimiter(syntax, open, "{", "}");
  return source.slice(open + 1, close);
}

function exactPublicTrait(source: string, name: string): string {
  const syntax = maskRustNonCode(source, true);
  const matches = [...syntax.matchAll(
    new RegExp(`\\bpub\\s+trait\\s+${name}\\b[^\\{]*\\{`, "g"),
  )];
  if (matches.length !== 1 || matches[0].index === undefined) {
    throw new Error(`expected exactly one public trait ${name}; found ${matches.length}`);
  }
  const open = matches[0].index + matches[0][0].lastIndexOf("{");
  const close = closingDelimiter(syntax, open, "{", "}");
  return normalized(source.slice(matches[0].index, close + 1));
}

function exactPublicTypeAlias(source: string, name: string): string {
  const syntax = maskRustNonCode(source, true);
  const matches = [...syntax.matchAll(
    new RegExp(`\\bpub\\s+type\\s+${name}\\b[^;]*;`, "g"),
  )];
  if (matches.length !== 1 || matches[0].index === undefined) {
    throw new Error(
      `expected exactly one public type alias ${name}; found ${matches.length}`,
    );
  }
  return normalized(source.slice(
    matches[0].index,
    matches[0].index + matches[0][0].length,
  ));
}

function declarationAttributes(
  source: string,
  kind: "struct" | "enum",
  name: string,
): string {
  const syntax = maskRustNonCode(source, true);
  const matches = [...syntax.matchAll(
    new RegExp(`\\b(?:pub\\s+)?${kind}\\s+${name}\\b`, "g"),
  )];
  if (matches.length !== 1 || matches[0].index === undefined) {
    throw new Error(
      `expected exactly one ${kind} ${name}; found ${matches.length}`,
    );
  }
  return normalized(immediateRustAttributes(
    maskRustNonCode(source, false),
    matches[0].index,
  ));
}

function inherentPublicMethodSignatures(
  source: string,
  typeName: string,
): string[] {
  const syntax = maskRustNonCode(source, true);
  const signatures: Array<{ offset: number; signature: string }> = [];
  for (const implementation of syntax.matchAll(
    new RegExp(
      `\\bimpl(?:\\s*<[^>{}]*>)?\\s+(?:(?:crate|self|super|[A-Za-z_][A-Za-z0-9_]*)\\s*::\\s*)*${typeName}(?:\\s*<[^>{}]*>)?\\s*\\{`,
      "g",
    ),
  )) {
    if (implementation.index === undefined) {
      throw new Error(`${typeName} impl has no offset`);
    }
    const open = implementation.index + implementation[0].lastIndexOf("{");
    const close = closingDelimiter(syntax, open, "{", "}");
    const body = source.slice(open + 1, close);
    const bodySyntax = syntax.slice(open + 1, close);
    let depth = 0;
    const depthAt = new Array<number>(bodySyntax.length);
    for (let index = 0; index < bodySyntax.length; index += 1) {
      depthAt[index] = depth;
      if (bodySyntax[index] === "{") depth += 1;
      else if (bodySyntax[index] === "}") depth -= 1;
    }
    for (const fn of rustFunctions(body)) {
      if (
        fn.visibility === "pub"
        && depthAt[fn.declarationStart] === 0
      ) {
        signatures.push({
          offset: open + 1 + fn.declarationStart,
          signature: fn.signature,
        });
      }
    }
  }
  return signatures
    .sort((left, right) => left.offset - right.offset)
    .map(({ signature }) => signature);
}

function portableFiles(): SourceFile[] {
  return allMoves.map(({ before }) => selectedMoveFile(before));
}

function topLevelPublicDefinitions(files: readonly SourceFile[]): string[] {
  const results: string[] = [];
  for (const file of files) {
    const syntax = maskRustNonCode(file.source, true);
    let depth = 0;
    const depthAt = new Array<number>(syntax.length);
    for (let index = 0; index < syntax.length; index += 1) {
      depthAt[index] = depth;
      if (syntax[index] === "{") depth += 1;
      else if (syntax[index] === "}") depth -= 1;
    }
    for (const match of syntax.matchAll(
      /\bpub\s+(?!crate\b|super\b|self\b)(?:(?:async|const|unsafe)\s+)*(?:(?:extern\b)\s+)?(?:struct|enum|union|trait|type|fn|mod|const|static|macro)\s+([A-Za-z_][A-Za-z0-9_]*)/g,
    )) {
      if (match.index !== undefined && depthAt[match.index] === 0) {
        results.push(match[1]);
      }
    }
    for (const match of syntax.matchAll(
      /\bpub\s+extern\s+crate\s+([A-Za-z_][A-Za-z0-9_]*)(?:\s+as\s+([A-Za-z_][A-Za-z0-9_]*))?\s*;/g,
    )) {
      if (match.index !== undefined && depthAt[match.index] === 0) {
        results.push(match[2] ?? match[1]);
      }
    }
    for (const match of syntax.matchAll(
      /\bmacro_rules\s*!\s*([A-Za-z_][A-Za-z0-9_]*)/g,
    )) {
      if (
        match.index !== undefined
        && /#\s*\[\s*macro_export\b/.test(
          immediateRustAttributes(syntax, match.index),
        )
      ) {
        results.push(match[1]);
      }
    }
  }
  return results.sort();
}

function tomlSection(source: string, heading: string): string {
  const marker = `[${heading}]`;
  const start = source.indexOf(marker);
  if (start < 0) return "";
  const bodyStart = start + marker.length;
  const next = source.slice(bodyStart).search(/^\[\[?[^\n]+\]?\]$/m);
  return source.slice(
    bodyStart,
    next < 0 ? undefined : bodyStart + next,
  ).trim();
}

function dependencyNames(section: string): string[] {
  return [
    ...section.matchAll(/^([A-Za-z0-9_-]+)(?:\.workspace)?\s*=/gm),
  ].map((match) => match[1]).sort();
}

type TomlDependency = {
  section: string;
  key: string;
  packageName: string;
  path?: string;
  workspace: boolean;
  value: string;
};

function stripTomlComment(line: string): string {
  let quoted = false;
  let escaped = false;
  for (let index = 0; index < line.length; index += 1) {
    const token = line[index];
    if (escaped) {
      escaped = false;
      continue;
    }
    if (token === "\\" && quoted) {
      escaped = true;
      continue;
    }
    if (token === '"') quoted = !quoted;
    else if (token === "#" && !quoted) return line.slice(0, index);
  }
  return line;
}

function tomlDependencies(source: string): TomlDependency[] {
  const lines = normalizeNewlines(source).split("\n");
  const dependencies: TomlDependency[] = [];
  let section = "";
  for (let index = 0; index < lines.length; index += 1) {
    const line = stripTomlComment(lines[index]).trim();
    if (!line) continue;
    const heading = line.match(/^\[([^\]]+)\]$/)?.[1];
    if (heading) {
      section = heading;
      continue;
    }
    if (
      !/(?:^|\.)(?:dev-|build-)?dependencies$/.test(section)
      && section !== "workspace.dependencies"
    ) {
      continue;
    }
    const declaration = line.match(
      /^"?([A-Za-z0-9_-]+)"?(?:\.workspace)?\s*=\s*(.*)$/,
    );
    if (!declaration) {
      throw new Error(`unparsed dependency declaration in [${section}]: ${line}`);
    }
    let value = declaration[2];
    let braces = 0;
    const countBraces = (text: string): number =>
      [...text].reduce(
        (depth, token) => depth + (token === "{" ? 1 : token === "}" ? -1 : 0),
        0,
      );
    braces += countBraces(value);
    while (braces > 0) {
      index += 1;
      if (index >= lines.length) {
        throw new Error(`unterminated dependency ${declaration[1]}`);
      }
      const continuation = stripTomlComment(lines[index]).trim();
      value += ` ${continuation}`;
      braces += countBraces(continuation);
    }
    if (braces !== 0) {
      throw new Error(`malformed dependency ${declaration[1]}`);
    }
    const packageName =
      value.match(/\bpackage\s*=\s*"([^"]+)"/)?.[1]
      ?? declaration[1];
    dependencies.push({
      section,
      key: declaration[1],
      packageName,
      path: value.match(/\bpath\s*=\s*"([^"]+)"/)?.[1],
      workspace:
        /\.workspace\s*=/.test(line)
        || /\bworkspace\s*=\s*true\b/.test(value),
      value: normalized(value),
    });
  }
  return dependencies;
}

function analysisDependencies(
  source: string,
  workspaceAnalysisKeys: ReadonlySet<string> = new Set(),
): TomlDependency[] {
  return tomlDependencies(source).filter((dependency) =>
    dependency.packageName === "extractum-analysis"
    || dependency.path?.replaceAll("\\", "/").endsWith(
      "/extractum-analysis",
    )
    || dependency.path === "crates/extractum-analysis"
    || (dependency.workspace && workspaceAnalysisKeys.has(dependency.key)));
}

function workspaceMembers(rootCargo: string): string[] {
  const body = rootCargo.match(/^members\s*=\s*\[([^\]]+)\]$/m)?.[1];
  if (!body) throw new Error("workspace members are missing or not one-line");
  return body
    .split(",")
    .map((entry) => entry.trim().replace(/^"|"$/g, ""));
}

function lockPackage(source: string, name: string): string {
  const matches = source
    .split(/(?=^\[\[package\]\]$)/m)
    .filter((block) => block.includes(`\nname = "${name}"\n`));
  if (matches.length !== 1) {
    throw new Error(`expected one lock package ${name}; found ${matches.length}`);
  }
  return matches[0];
}

function optionalLockPackage(source: string, name: string): string {
  const matches = source
    .split(/(?=^\[\[package\]\]$)/m)
    .filter((block) => block.includes(`\nname = "${name}"\n`));
  if (matches.length > 1) {
    throw new Error(`duplicate lock package ${name}`);
  }
  return matches[0] ?? "";
}

function lockDependencies(block: string): string[] {
  const match = block.match(/^dependencies = \[\n([\s\S]*?)^\]$/m);
  if (!match) return [];
  return [...match[1].matchAll(/^ "([^"]+)",?$/gm)]
    .map((entry) => entry[1].replace(/ \d+\..*$/, ""))
    .sort();
}

function planTransactionMap(): Array<{
  family: number;
  participant: string;
  coordinators: string[];
}> {
  const start = implementationPlan.indexOf("## Frozen SQL and Transaction API Map");
  const end = implementationPlan.indexOf("## Frozen Manifest and Test-Schema Contract", start);
  if (start < 0 || end <= start) throw new Error("missing transaction API map");
  const rows = [
    ...implementationPlan.slice(start, end).matchAll(
      /^\|\s*([1-4])\s*\|\s*`([A-Za-z0-9_]+)`\s*\|\s*([^\n]+?)\s*\|$/gm,
    ),
  ].map((match) => ({
    family: Number(match[1]),
    participant: match[2],
    coordinators: [
      ...match[3].matchAll(
        /`(?:[A-Za-z0-9_]+::)*([A-Za-z0-9_]+_in_pool)`/g,
      ),
    ]
      .map((entry) => entry[1]),
  }));
  if (rows.length !== 8) {
    throw new Error(`borrowed participant count drifted: ${rows.length}`);
  }
  const coordinators = rows.flatMap(({ coordinators }) => coordinators);
  if (
    new Set(rows.map(({ participant }) => participant)).size !== 8
    || new Set(coordinators).size !== 9
  ) {
    throw new Error("transaction participant/coordinator map is not 8/9");
  }
  expect([...new Set(rows.map(({ family }) => family))].sort()).toEqual([
    1, 2, 3, 4,
  ]);
  return rows;
}

const transactionMap = planTransactionMap();

function findFunctions(
  files: readonly SourceFile[],
  name: string,
): Array<RustFunction & { file: SourceFile }> {
  return files.flatMap((file) =>
    rustFunctions(file.source)
      .filter((candidate) => candidate.name === name)
      .map((candidate) => ({ ...candidate, file })));
}

function soleFunction(
  files: readonly SourceFile[],
  name: string,
): RustFunction & { file: SourceFile } {
  const matches = findFunctions(files, name);
  if (matches.length !== 1) {
    throw new Error(`expected one function ${name}; found ${matches.length}`);
  }
  return matches[0];
}

function solePublicFunction(
  files: readonly SourceFile[],
  name: string,
): RustFunction & { file: SourceFile } {
  const matches = findFunctions(files, name)
    .filter(({ visibility }) => visibility === "pub");
  if (matches.length !== 1) {
    throw new Error(
      `expected one public function ${name}; found ${matches.length}`,
    );
  }
  return matches[0];
}

function firstParameter(params: string): string {
  return splitTopLevel(params)[0] ?? "";
}

function rustStringValues(source: string): string[] {
  const values: string[] = [];
  for (let index = 0; index < source.length;) {
    if (source.startsWith("//", index)) {
      const newline = source.indexOf("\n", index + 2);
      index = newline < 0 ? source.length : newline;
      continue;
    }
    if (source.startsWith("/*", index)) {
      let depth = 1;
      index += 2;
      while (index < source.length && depth > 0) {
        if (source.startsWith("/*", index)) {
          depth += 1;
          index += 2;
        } else if (source.startsWith("*/", index)) {
          depth -= 1;
          index += 2;
        } else index += 1;
      }
      continue;
    }
    const raw = source.slice(index).match(/^(?:br|rb|r)(#{0,255})"/);
    if (raw) {
      const closing = `"${raw[1]}`;
      const contentStart = index + raw[0].length;
      const close = source.indexOf(closing, contentStart);
      if (close < 0) throw new Error("unclosed Rust raw string");
      values.push(source.slice(contentStart, close));
      index = close + closing.length;
      continue;
    }
    const byte = source.startsWith('b"', index);
    if (source[index] === '"' || byte) {
      const quote = byte ? index + 1 : index;
      let end = quote + 1;
      while (end < source.length) {
        if (source[end] === "\\") end += 2;
        else if (source[end] === '"') break;
        else end += 1;
      }
      if (end >= source.length) throw new Error("unclosed Rust string");
      values.push(
        source.slice(quote + 1, end)
          .replace(/\\(?:r|n|t)/g, " ")
          .replace(/\\"/g, '"')
          .replace(/\\\\/g, "\\"),
      );
      index = end + 1;
      continue;
    }
    const character = source[index] === "'"
      && /^'(?:\\.|[^\\'\n])'/.test(source.slice(index));
    if (character) {
      const end = source.indexOf("'", index + 1);
      index = end < 0 ? source.length : end + 1;
      continue;
    }
    index += 1;
  }
  return values;
}

function migrationCreatedTables(sql: string): string[] {
  return [
    ...sql.matchAll(
      /\bCREATE\s+(?:VIRTUAL\s+)?TABLE\s+(?:IF\s+NOT\s+EXISTS\s+)?(?:(?:"[^"]+"|`[^`]+`|\[[^\]]+\]|[A-Za-z_][A-Za-z0-9_]*)\s*\.\s*)?(?:"([^"]+)"|`([^`]+)`|\[([^\]]+)\]|([A-Za-z_][A-Za-z0-9_]*))/gi,
    ),
  ].map((match) =>
    (match[1] ?? match[2] ?? match[3] ?? match[4]).toLowerCase());
}

const ownedTables = new Set([
  "analysis_runs",
  "analysis_run_messages",
  "analysis_chat_messages",
  "analysis_prompt_templates",
  "analysis_source_groups",
  "analysis_source_group_members",
]);
const nonTableScalarPragmas = new Set([
  "busy_timeout",
  "cache_size",
  "foreign_keys",
  "ignore_check_constraints",
  "journal_mode",
  "query_only",
  "synchronous",
  "temp_store",
]);
const tableBearingPragmas = new Set([
  "foreign_key_check",
  "foreign_key_list",
  "index_list",
  "integrity_check",
  "quick_check",
  "table_info",
  "table_xinfo",
]);
const schemaTables = new Set(
  filesWithExtension(path.join(repoRoot, "src-tauri/migrations"), ".sql")
    .flatMap((file) =>
      migrationCreatedTables(normalizeNewlines(readFileSync(file, "utf8")))),
);

type SqlReference = {
  operation: string;
  table: string;
};

function splitSqlStatements(sql: string): string[] {
  const statements: string[] = [];
  let start = 0;
  let quote = "";
  let lineComment = false;
  let blockComment = false;
  for (let index = 0; index < sql.length; index += 1) {
    const token = sql[index];
    const next = sql[index + 1];
    if (lineComment) {
      if (token === "\n") lineComment = false;
      continue;
    }
    if (blockComment) {
      if (token === "*" && next === "/") {
        blockComment = false;
        index += 1;
      }
      continue;
    }
    if (quote) {
      if (quote === "]") {
        if (token === "]") quote = "";
      } else if (token === quote) {
        if (next === quote) index += 1;
        else quote = "";
      }
      continue;
    }
    if (token === "-" && next === "-") {
      lineComment = true;
      index += 1;
    } else if (token === "/" && next === "*") {
      blockComment = true;
      index += 1;
    } else if (token === "'" || token === '"' || token === "`") {
      quote = token;
    } else if (token === "[") {
      quote = "]";
    } else if (token === ";") {
      const statement = sql.slice(start, index).trim();
      if (statement) statements.push(statement);
      start = index + 1;
    }
  }
  const tail = sql.slice(start).trim();
  if (tail) statements.push(tail);
  return statements;
}

function sqlWithoutComments(sql: string): string | null {
  if (!sql.includes("--") && !sql.includes("/*")) return sql;
  let output = "";
  let quote = "";
  let lineComment = false;
  let blockComment = false;
  for (let index = 0; index < sql.length; index += 1) {
    const token = sql[index];
    const next = sql[index + 1];
    if (lineComment) {
      if (token === "\r" || token === "\n") {
        lineComment = false;
        output += token;
      } else {
        output += " ";
      }
      continue;
    }
    if (blockComment) {
      if (token === "*" && next === "/") {
        output += "  ";
        blockComment = false;
        index += 1;
      } else {
        output += token === "\r" || token === "\n" ? token : " ";
      }
      continue;
    }
    if (quote) {
      output += token;
      if (token === quote) {
        if (next === quote) {
          output += next;
          index += 1;
        } else {
          quote = "";
        }
      }
      continue;
    }
    if (token === "-" && next === "-") {
      output += "  ";
      lineComment = true;
      index += 1;
    } else if (token === "/" && next === "*") {
      output += "  ";
      blockComment = true;
      index += 1;
    } else {
      output += token;
      if (
        token === "'"
        || token === '"'
        || token === "`"
      ) {
        quote = token;
      } else if (token === "[") {
        quote = "]";
      }
    }
  }
  return blockComment || quote ? null : output;
}

function sqlIdentifierArgument(value: string): string | null {
  const match =
    /^(?:"((?:[^"]|"")*)"|'((?:[^']|'')*)'|`((?:[^`]|``)*)`|\[((?:[^\]]|\]\])*)\]|([A-Za-z_][A-Za-z0-9_]*))$/
      .exec(value.trim());
  if (!match) return null;
  if (match[1] !== undefined) return match[1].replace(/""/g, '"').toLowerCase();
  if (match[2] !== undefined) return match[2].replace(/''/g, "'").toLowerCase();
  if (match[3] !== undefined) return match[3].replace(/``/g, "`").toLowerCase();
  if (match[4] !== undefined) return match[4].replace(/\]\]/g, "]").toLowerCase();
  return match[5].toLowerCase();
}

function statementSqlReferences(sql: string): SqlReference[] {
  const uncommentedSql = sqlWithoutComments(sql);
  if (uncommentedSql === null) {
    return [{
      operation: "UNRESOLVED SQL",
      table: "__unresolved_sql_statement__",
    }];
  }
  const executableSql = uncommentedSql.trimStart();
  const aliases = new Set(
    [...executableSql.matchAll(
      /\b(?:WITH(?:\s+RECURSIVE)?|,)\s*(?:"([^"]+)"|`([^`]+)`|\[([^\]]+)\]|([A-Za-z_][A-Za-z0-9_]*))(?:\s*\([^)]*\))?\s+AS\s*\(/gi,
    )].map((match) =>
      (match[1] ?? match[2] ?? match[3] ?? match[4]).toLowerCase()),
  );
  const references: SqlReference[] = [];
  const tablePattern =
    /\b(FROM|JOIN|REFERENCES|UPDATE(?:\s+OR\s+(?:ROLLBACK|ABORT|REPLACE|FAIL|IGNORE))?|(?:INSERT(?:\s+OR\s+(?:ROLLBACK|ABORT|REPLACE|FAIL|IGNORE))?\s+)?INTO|CREATE\s+(?:VIRTUAL\s+)?TABLE(?:\s+IF\s+NOT\s+EXISTS)?|ALTER\s+TABLE|DROP\s+TABLE(?:\s+IF\s+EXISTS)?|CREATE\s+(?:UNIQUE\s+)?INDEX(?:\s+IF\s+NOT\s+EXISTS)?\s+(?:(?:"[^"]+"|`[^`]+`|\[[^\]]+\]|[A-Za-z_][A-Za-z0-9_]*)\s*\.\s*)?(?:"[^"]+"|`[^`]+`|\[[^\]]+\]|[A-Za-z_][A-Za-z0-9_]*)\s+ON)\s+(?:(?:"([^"]+)"|`([^`]+)`|\[([^\]]+)\]|([A-Za-z_][A-Za-z0-9_]*))\s*\.\s*)?(?:"([^"]+)"|`([^`]+)`|\[([^\]]+)\]|([A-Za-z_][A-Za-z0-9_]*))/gi;
  for (const match of executableSql.matchAll(tablePattern)) {
    const schema = match[2] ?? match[3] ?? match[4] ?? match[5];
    const leaf = (match[6] ?? match[7] ?? match[8] ?? match[9])
      .toLowerCase();
    const table = schema ? `${schema.toLowerCase()}.${leaf}` : leaf;
    if (!aliases.has(table)) {
      const operation = normalized(match[1]).toUpperCase();
      references.push({
        operation: operation === "INTO"
          ? "INSERT INTO"
          : operation.startsWith("CREATE INDEX")
              || operation.startsWith("CREATE UNIQUE INDEX")
            ? "CREATE INDEX ON"
            : operation,
        table,
      });
    }
  }
  const maintenancePattern =
    /\b(ANALYZE|REINDEX)\s+(?:(?:"([^"]+)"|`([^`]+)`|\[([^\]]+)\]|([A-Za-z_][A-Za-z0-9_]*))\s*\.\s*)?(?:"([^"]+)"|`([^`]+)`|\[([^\]]+)\]|([A-Za-z_][A-Za-z0-9_]*))/gi;
  for (const match of executableSql.matchAll(maintenancePattern)) {
    const schema = match[2] ?? match[3] ?? match[4] ?? match[5];
    const leaf = (match[6] ?? match[7] ?? match[8] ?? match[9])
      .toLowerCase();
    references.push({
      operation: match[1].toUpperCase(),
      table: schema ? `${schema.toLowerCase()}.${leaf}` : leaf,
    });
  }
  const triggerPattern =
    /\bCREATE\s+(?:TEMP(?:ORARY)?\s+)?TRIGGER(?:\s+IF\s+NOT\s+EXISTS)?\s+(?:(?:"[^"]+"|`[^`]+`|\[[^\]]+\]|[A-Za-z_][A-Za-z0-9_]*)\s*\.\s*)?(?:"[^"]+"|`[^`]+`|\[[^\]]+\]|[A-Za-z_][A-Za-z0-9_]*)[\s\S]*?\bON\s+(?:(?:"([^"]+)"|`([^`]+)`|\[([^\]]+)\]|([A-Za-z_][A-Za-z0-9_]*))\s*\.\s*)?(?:"([^"]+)"|`([^`]+)`|\[([^\]]+)\]|([A-Za-z_][A-Za-z0-9_]*))/gi;
  for (const match of executableSql.matchAll(triggerPattern)) {
    const schema = match[1] ?? match[2] ?? match[3] ?? match[4];
    const leaf = (match[5] ?? match[6] ?? match[7] ?? match[8])
      .toLowerCase();
    references.push({
      operation: "CREATE TRIGGER ON",
      table: schema ? `${schema.toLowerCase()}.${leaf}` : leaf,
    });
  }
  const pragmaWithArgument =
    /^\s*PRAGMA\s+(?:((?:"(?:[^"]|"")*"|'(?:[^']|'')*'|`(?:[^`]|``)*`|\[(?:[^\]]|\]\])*\]|[A-Za-z_][A-Za-z0-9_]*))\s*\.\s*)?((?:"(?:[^"]|"")*"|'(?:[^']|'')*'|`(?:[^`]|``)*`|\[(?:[^\]]|\]\])*\]|[A-Za-z_][A-Za-z0-9_]*))\s*(?:\(\s*([\s\S]*?)\s*\)|=\s*([\s\S]*?))\s*;?\s*$/i
      .exec(executableSql);
  const pragmaArgumentSyntax =
    /^PRAGMA\b/i.test(executableSql)
    && (executableSql.includes("(") || executableSql.includes("="));
  let unresolvedIdentifierPragma =
    pragmaWithArgument === null && pragmaArgumentSyntax;
  if (pragmaWithArgument) {
    const schema = pragmaWithArgument[1] === undefined
      ? null
      : sqlIdentifierArgument(pragmaWithArgument[1]);
    const pragmaName = sqlIdentifierArgument(pragmaWithArgument[2]);
    const argument = sqlIdentifierArgument(
      pragmaWithArgument[3] ?? pragmaWithArgument[4],
    );
    const malformedQuotedArgument = argument === null
      && /^["'`\[]/.test(
        (pragmaWithArgument[3] ?? pragmaWithArgument[4]).trim(),
      );
    if (
      pragmaName !== null
      && argument !== null
      && tableBearingPragmas.has(pragmaName)
    ) {
      references.push({
        operation: `PRAGMA ${pragmaName.toUpperCase()}`,
        table: schema ? `${schema}.${argument}` : argument,
      });
    } else if (
      pragmaName !== null
      && (
        malformedQuotedArgument
        || (argument === null && tableBearingPragmas.has(pragmaName))
        || (
          argument !== null
          && !nonTableScalarPragmas.has(pragmaName)
        )
      )
    ) {
      unresolvedIdentifierPragma = true;
    }
  }
  if (
    references.length === 0
    && (
      /^(?:INSERT|REPLACE|UPDATE|DELETE|CREATE|ALTER|DROP|ANALYZE|REINDEX|TRUNCATE|MERGE)\b/i
        .test(executableSql)
      || unresolvedIdentifierPragma
    )
  ) {
    references.push({
      operation: "UNRESOLVED SQL",
      table: "__unresolved_sql_statement__",
    });
  }
  return references;
}

function sqlReferences(sql: string): SqlReference[] {
  return splitSqlStatements(sql).flatMap(statementSqlReferences);
}

function splitRustExpressions(value: string): string[] {
  const syntax = maskRustNonCode(value, true);
  const parts: string[] = [];
  const stack: string[] = [];
  const pairs: Record<string, string> = {
    "(": ")",
    "[": "]",
    "{": "}",
    "<": ">",
  };
  let start = 0;
  for (let index = 0; index < syntax.length; index += 1) {
    const token = syntax[index];
    if (pairs[token]) stack.push(pairs[token]);
    else if (stack.at(-1) === token) stack.pop();
    else if (token === "," && stack.length === 0) {
      parts.push(value.slice(start, index).trim());
      start = index + 1;
    }
  }
  parts.push(value.slice(start).trim());
  return parts.filter(Boolean);
}

type ScopedStringDefinition = {
  expression: string;
  declarationOffset: number;
  scopeOpen: number;
  scopeClose: number;
  visibleBeforeDeclaration: boolean;
  bindingOffset: number;
};

const dynamicStringBinding = "__extractum_dynamic_string_binding__";

function rustBindingName(parameter: string): string | undefined {
  const value = parameter
    .replace(/^(?:#\s*\[[\s\S]*?\]\s*)+/, "")
    .trim();
  const match = value.match(
    /^(?:&\s*(?:'[A-Za-z_][A-Za-z0-9_]*\s*)?(?:mut\s+)?)?(?:(?:mut|ref)\s+)*(r#[A-Za-z_][A-Za-z0-9_]*|[A-Za-z_][A-Za-z0-9_]*)(?:\s*:\s*[\s\S]+)?$/,
  );
  const name = match?.[1];
  return name && name !== "self" && name !== "_" ? name : undefined;
}

function rustExpressionEnd(syntax: string, start: number): number {
  const stack: string[] = [];
  const pairs: Record<string, string> = {
    "(": ")",
    "[": "]",
    "{": "}",
  };
  for (let index = start; index < syntax.length; index += 1) {
    const token = syntax[index];
    if (pairs[token]) stack.push(pairs[token]);
    else if (stack.at(-1) === token) stack.pop();
    else if (token === ";" && stack.length === 0) return index;
  }
  return -1;
}

function closureExpressionEnd(syntax: string, start: number): number {
  const stack: string[] = [];
  const pairs: Record<string, string> = {
    "(": ")",
    "[": "]",
    "{": "}",
  };
  for (let index = start; index < syntax.length; index += 1) {
    const token = syntax[index];
    if (pairs[token]) {
      stack.push(pairs[token]);
      continue;
    }
    if (stack.at(-1) === token) {
      stack.pop();
      continue;
    }
    if (
      stack.length === 0
      && (
        token === ","
        || token === ";"
        || token === ")"
        || token === "]"
        || token === "}"
      )
    ) return index;
  }
  return syntax.length;
}

function visibleStringDefinition(
  definitions: ReadonlyMap<string, readonly ScopedStringDefinition[]>,
  name: string,
  usageOffset: number,
): ScopedStringDefinition | undefined {
  const visible = (definitions.get(name) ?? [])
    .filter((candidate) =>
      candidate.scopeOpen < usageOffset
      && usageOffset < candidate.scopeClose
      && (
        candidate.visibleBeforeDeclaration
        || candidate.declarationOffset < usageOffset
      ));
  if (visible.length === 0) return undefined;
  const deepestScope = Math.max(...visible.map(({ scopeOpen }) => scopeOpen));
  return visible
    .filter(({ scopeOpen }) => scopeOpen === deepestScope)
    .sort((left, right) =>
      right.declarationOffset - left.declarationOffset)[0];
}

function stringDefinitions(
  source: string,
): Map<string, ScopedStringDefinition[]> {
  const syntax = maskRustNonCode(source, true);
  const scopes = rustBraceRegions(source);
  const conditionalRegions = [...syntax.matchAll(
    /\b(?:if|else|match|while|for|loop)\b[^;{]*\{/g,
  )].flatMap((match) => {
    if (match.index === undefined) return [];
    const open = match.index + match[0].lastIndexOf("{");
    return [{
      open,
      close: closingDelimiter(syntax, open, "{", "}"),
    }];
  });
  const deferredClosureRegions = [...syntax.matchAll(
    /(?:^|[=(:,;\[{]\s*|\b(?:async|move|return)\s+)\|([^|\n]*)\|/gm,
  )].flatMap((closure) => {
    if (closure.index === undefined) return [];
    const closePipe = closure.index + closure[0].lastIndexOf("|");
    let bodyStart = closePipe + 1;
    while (bodyStart < syntax.length && /\s/.test(syntax[bodyStart])) {
      bodyStart += 1;
    }
    return syntax[bodyStart] === "{"
      ? [{
          open: bodyStart,
          close: closingDelimiter(syntax, bodyStart, "{", "}"),
        }]
      : [{
          open: closePipe,
          close: closureExpressionEnd(syntax, bodyStart),
        }];
  });
  const deferredAsyncRegions = [...syntax.matchAll(
    /\basync(?:\s+move)?\s*\{/g,
  )].flatMap((block) => {
    if (block.index === undefined) return [];
    const open = block.index + block[0].lastIndexOf("{");
    return [{
      open,
      close: closingDelimiter(syntax, open, "{", "}"),
    }];
  });
  const definitions = new Map<string, ScopedStringDefinition[]>();
  const addDefinition = (
    name: string,
    definition: ScopedStringDefinition,
  ): void => {
    const values = definitions.get(name) ?? [];
    values.push(definition);
    definitions.set(name, values);
  };
  for (const match of syntax.matchAll(
    /\b(const|static|let(?:\s+mut)?)\s+([A-Za-z_][A-Za-z0-9_]*)(?:\s*:[^=;]+)?\s*=/g,
  )) {
    if (match.index === undefined) {
      throw new Error("Rust string definition has no offset");
    }
    const expressionStart = match.index + match[0].length;
    const expressionEnd = rustExpressionEnd(syntax, expressionStart);
    if (expressionEnd < 0) continue;
    const scope = scopes
      .filter(({ open, close }) => open < match.index && match.index < close)
      .sort((left, right) => right.open - left.open)[0]
      ?? { open: -1, close: source.length };
    addDefinition(match[2], {
      expression: source.slice(expressionStart, expressionEnd).trim(),
      declarationOffset: match.index,
      scopeOpen: scope.open,
      scopeClose: scope.close,
      visibleBeforeDeclaration: match[1] === "const" || match[1] === "static",
      bindingOffset: match.index,
    });
  }
  for (const fn of rustFunctions(source)) {
    for (const parameter of splitTopLevel(fn.params)) {
      const name = rustBindingName(parameter);
      if (!name) continue;
      addDefinition(name, {
        expression: dynamicStringBinding,
        declarationOffset: fn.bodyStart,
        scopeOpen: fn.bodyStart,
        scopeClose: fn.bodyEnd,
        visibleBeforeDeclaration: true,
        bindingOffset: fn.bodyStart,
      });
    }
  }
  for (const closure of syntax.matchAll(
    /(?:^|[=(:,;\[{]\s*|\b(?:async|move|return)\s+)\|([^|\n]*)\|/gm,
  )) {
    if (closure.index === undefined) {
      throw new Error("Rust closure has no offset");
    }
    const closePipe = closure.index + closure[0].lastIndexOf("|");
    let bodyStart = closePipe + 1;
    while (bodyStart < syntax.length && /\s/.test(syntax[bodyStart])) {
      bodyStart += 1;
    }
    const braced = syntax[bodyStart] === "{";
    const scopeOpen = braced ? bodyStart : closePipe;
    const scopeClose = braced
      ? closingDelimiter(syntax, bodyStart, "{", "}")
      : closureExpressionEnd(syntax, bodyStart);
    for (const parameter of splitTopLevel(closure[1])) {
      const name = rustBindingName(parameter);
      if (!name) continue;
      addDefinition(name, {
        expression: dynamicStringBinding,
        declarationOffset: closePipe,
        scopeOpen,
        scopeClose,
        visibleBeforeDeclaration: true,
        bindingOffset: closePipe,
      });
    }
  }
  for (const assignment of syntax.matchAll(
    /\b([A-Za-z_][A-Za-z0-9_]*)\s*(=|\+=|-=|\*=|\/=|%=|&=|\|=|\^=)(?!=|>)/g,
  )) {
    if (assignment.index === undefined) {
      throw new Error("Rust assignment has no offset");
    }
    const prefix = syntax.slice(
      Math.max(0, assignment.index - 120),
      assignment.index,
    );
    if (/\b(?:const|static|type|let(?:\s+mut)?)\s+$/.test(prefix)) {
      continue;
    }
    const target = visibleStringDefinition(
      definitions,
      assignment[1],
      assignment.index,
    );
    if (!target) continue;
    const expressionStart = assignment.index + assignment[0].length;
    const expressionEnd = rustExpressionEnd(syntax, expressionStart);
    if (expressionEnd < 0) continue;
    const branchLocal = [
      ...conditionalRegions,
      ...deferredClosureRegions,
      ...deferredAsyncRegions,
    ].some(({ open, close }) =>
      open < assignment.index! && assignment.index! < close);
    addDefinition(assignment[1], {
      expression: assignment[2] === "=" && !branchLocal
        ? source.slice(expressionStart, expressionEnd).trim()
        : dynamicStringBinding,
      declarationOffset: assignment.index,
      scopeOpen: target.scopeOpen,
      scopeClose: target.scopeClose,
      visibleBeforeDeclaration: false,
      bindingOffset: target.bindingOffset,
    });
  }
  return definitions;
}

function staticRustString(
  expression: string,
  definitions: ReadonlyMap<string, ScopedStringDefinition[]>,
  usageOffset: number,
  resolving: ReadonlySet<string> = new Set(),
): string | undefined {
  let value = expression.trim();
  while (value.startsWith("&")) value = value.slice(1).trim();
  for (;;) {
    if (!(value.startsWith("(") && value.endsWith(")"))) break;
    const syntax = maskRustNonCode(value, true);
    if (closingDelimiter(syntax, 0, "(", ")") !== value.length - 1) break;
    value = value.slice(1, -1).trim();
  }
  const literal = exactRustPathLiteral(value);
  if (literal !== undefined) return literal;
  const concat = value.match(/^concat\s*!\s*\(/);
  if (concat) {
    const syntax = maskRustNonCode(value, true);
    const open = value.indexOf("(", concat.index ?? 0);
    const close = closingDelimiter(syntax, open, "(", ")");
    if (value.slice(close + 1).trim() !== "") return undefined;
    const parts = splitRustExpressions(value.slice(open + 1, close))
      .map((part) => staticRustString(
        part,
        definitions,
        usageOffset,
        resolving,
      ));
    return parts.every((part) => part !== undefined)
      ? parts.join("")
      : undefined;
  }
  if (/^[A-Za-z_][A-Za-z0-9_]*$/.test(value)) {
    if (resolving.has(value)) return undefined;
    const definition = visibleStringDefinition(
      definitions,
      value,
      usageOffset,
    );
    if (!definition) return undefined;
    return staticRustString(
      definition.expression,
      definitions,
      definition.declarationOffset,
      new Set([...resolving, value]),
    );
  }
  return undefined;
}

const regexEscaped = (value: string): string =>
  value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");

type SqlConsumerAnalysis = {
  sql: string[];
  unresolved: string[];
  unresolvedDetails: Array<{
    kind: string;
    expression: string;
    offset: number;
  }>;
  resolved: Array<{ kind: string; sql: string; offset: number }>;
};

const sqlConsumerCache = new Map<string, SqlConsumerAnalysis>();

function sqlConsumerAnalysis(source: string): {
  sql: string[];
  unresolved: string[];
  unresolvedDetails: Array<{
    kind: string;
    expression: string;
    offset: number;
  }>;
  resolved: Array<{ kind: string; sql: string; offset: number }>;
} {
  const cached = sqlConsumerCache.get(source);
  if (cached) return cached;
  const production = productionRust(source);
  const syntax = maskRustNonCode(production, true);
  const definitions = stringDefinitions(production);
  const lexicalScopes = rustBraceRegions(production);
  const moduleRegions = inlineModuleRegions(production);
  const moduleOpens = new Set(moduleRegions.map(({ open }) => open));
  const isModuleScopeAt = (offset: number): boolean =>
    lexicalScopes
      .filter(({ open, close }) => open < offset && offset < close)
      .every(({ open }) => moduleOpens.has(open));
  const modulePathAt = (offset: number): string[] =>
    moduleRegions
      .filter(({ open, close }) => open < offset && offset < close)
      .sort((left, right) => left.open - right.open)
      .map(({ name }) => name);
  const moduleKeyAt = (offset: number): string =>
    moduleRegions
      .filter(({ open, close }) => open < offset && offset < close)
      .sort((left, right) => left.open - right.open)
      .map(({ open }) => String(open))
      .join("/");
  type LexicalScope = {
    open: number;
    close: number;
    moduleKey: string | null;
  };
  const scopeAt = (offset: number): LexicalScope => {
    const scope = lexicalScopes
      .filter(({ open, close }) => open < offset && offset < close)
      .sort((left, right) => right.open - left.open)[0]
      ?? { open: -1, close: production.length };
    return { ...scope, moduleKey: moduleKeyAt(offset) };
  };
  const inScope = (scope: LexicalScope, offset: number): boolean =>
    scope.open < offset
    && offset < scope.close
    && (
      scope.moduleKey === null
      || scope.moduleKey === moduleKeyAt(offset)
    );
  const sql: string[] = [];
  const unresolved: string[] = [];
  const unresolvedDetails: Array<{
    kind: string;
    expression: string;
    offset: number;
  }> = [];
  const addUnresolved = (
    kind: string,
    expression: string,
    offset: number,
  ): void => {
    const normalizedExpression = normalized(expression);
    unresolved.push(`${kind}:${normalizedExpression}`);
    unresolvedDetails.push({
      kind,
      expression: normalizedExpression,
      offset,
    });
  };
  const resolvedConsumers: Array<{
    kind: string;
    sql: string;
    offset: number;
  }> = [];
  const inspectedOpens = new Set<number>();
  const inspectExpression = (
    kind: string,
    expression: string,
    offset: number,
  ): void => {
    const resolved = staticRustString(expression, definitions, offset);
    if (resolved === undefined) {
      addUnresolved(kind, expression, offset);
    } else {
      sql.push(resolved);
      resolvedConsumers.push({ kind, sql: resolved, offset });
    }
  };
  const inspectCall = (
    kind: string,
    open: number,
    close: number,
    argumentIndex = 0,
    exactArgumentCount: number | undefined = 1,
  ): void => {
    if (inspectedOpens.has(open)) return;
    inspectedOpens.add(open);
    const argumentsList = splitRustExpressions(
      production.slice(open + 1, close),
    );
    const expression = argumentsList[argumentIndex]
      ?? production.slice(open + 1, close).trim();
    if (
      argumentsList[argumentIndex] === undefined
      || (
        exactArgumentCount !== undefined
        && argumentsList.length !== exactArgumentCount
      )
    ) {
      addUnresolved(kind, expression, open);
    } else {
      inspectExpression(kind, expression, open);
    }
  };

  const roots = new Set(["sqlx"]);
  const qualifiedRootAliases = new Set<string>();
  const rootScopes = new Map<string, LexicalScope[]>([
    ["sqlx", [{
      open: -1,
      close: production.length,
      moduleKey: null,
    }]],
  ]);
  const addScope = (
    bindings: Map<string, LexicalScope[]>,
    name: string,
    offset: number,
  ): void => {
    const scopes = bindings.get(name) ?? [];
    scopes.push(scopeAt(offset));
    bindings.set(name, scopes);
  };
  const activeIn = (
    bindings: ReadonlyMap<string, readonly LexicalScope[]>,
    name: string,
    offset: number,
  ): boolean => (bindings.get(name) ?? []).some((scope) =>
    inScope(scope, offset));
  for (const imported of syntax.matchAll(
    /\buse\s+(?:::)?sqlx\s+as\s+([A-Za-z_][A-Za-z0-9_]*)\s*;/g,
  )) {
    if (imported.index === undefined) throw new Error("sqlx alias has no offset");
    roots.add(imported[1]);
    addScope(rootScopes, imported[1], imported.index);
    if (isModuleScopeAt(imported.index)) {
      qualifiedRootAliases.add(
        [...modulePathAt(imported.index), imported[1]].join("::"),
      );
    }
  }
  for (const imported of syntax.matchAll(
    /\bextern\s+crate\s+sqlx\s+as\s+([A-Za-z_][A-Za-z0-9_]*)\s*;/g,
  )) {
    if (imported.index === undefined) {
      throw new Error("extern crate sqlx alias has no offset");
    }
    roots.add(imported[1]);
    addScope(rootScopes, imported[1], imported.index);
    if (isModuleScopeAt(imported.index)) {
      qualifiedRootAliases.add(
        [...modulePathAt(imported.index), imported[1]].join("::"),
      );
    }
  }
  for (const grouped of syntax.matchAll(
    /\buse\s+(?:::)?sqlx\s*::\s*\{([\s\S]*?)\}\s*;/g,
  )) {
    if (grouped.index === undefined) throw new Error("sqlx self alias has no offset");
    for (const item of splitTopLevel(grouped[1])) {
      const selfAlias = item.match(
        /^self\s+as\s+([A-Za-z_][A-Za-z0-9_]*)$/,
      )?.[1];
      if (!selfAlias) continue;
      roots.add(selfAlias);
      addScope(rootScopes, selfAlias, grouped.index);
      if (isModuleScopeAt(grouped.index)) {
        qualifiedRootAliases.add(
          [...modulePathAt(grouped.index), selfAlias].join("::"),
        );
      }
    }
  }
  const localFunctions = new Map<
    string,
    Array<{
      kind: string;
      scope: LexicalScope;
      visibleFrom: number;
      bindingOffset?: number;
    }>
  >();
  const builderTypes = new Set(["QueryBuilder"]);
  const builderTypeScopes = new Map<string, LexicalScope[]>();
  const executorTraits = new Set<string>();
  const executorTraitScopes = new Map<string, LexicalScope[]>();
  const preludeModuleScopes = new Map<string, LexicalScope[]>();
  const qualifiedPreludeAliases = new Set<string>();
  const qualifiedFunctionAliases = new Map<string, string>();
  const importItem = (item: string, offset: number): void => {
    const value = item.trim();
    const nestedPrelude = value.match(
      /^prelude\s*::\s*\{([\s\S]*)\}$/,
    );
    if (nestedPrelude) {
      for (const child of splitTopLevel(nestedPrelude[1])) {
        importItem(`prelude::${child}`, offset);
      }
      return;
    }
    const prelude = value.match(
      /^prelude\s*::\s*(Executor|\*)(?:\s+as\s+([A-Za-z_][A-Za-z0-9_]*))?$/,
    );
    if (prelude) {
      importItem(
        `Executor${prelude[2] ? ` as ${prelude[2]}` : ""}`,
        offset,
      );
      return;
    }
    if (value === "*") {
      for (const name of [
        "raw_sql",
        "query",
        "query_as",
        "query_scalar",
        "query_with",
        "query_as_with",
        "query_scalar_with",
        "query_file",
        "query_file_as",
        "query_file_scalar",
        "QueryBuilder",
        "Executor",
      ]) importItem(name, offset);
      return;
    }
    const parsed = item.trim().match(
      /^(raw_sql|query(?:_as|_scalar)?(?:_with)?|query_file(?:_as|_scalar)?|QueryBuilder|Executor)(?:\s+as\s+([A-Za-z_][A-Za-z0-9_]*))?$/,
    );
    if (!parsed) return;
    const local = parsed[2] ?? parsed[1];
    if (parsed[1] === "QueryBuilder") {
      builderTypes.add(local);
      addScope(builderTypeScopes, local, offset);
    } else if (parsed[1] === "Executor") {
      executorTraits.add(local);
      addScope(executorTraitScopes, local, offset);
    } else {
      const bindings = localFunctions.get(local) ?? [];
      const scope = scopeAt(offset);
      bindings.push({
        kind: parsed[1],
        scope,
        visibleFrom: scope.open + 1,
      });
      localFunctions.set(local, bindings);
      if (isModuleScopeAt(offset)) {
        qualifiedFunctionAliases.set(
          [...modulePathAt(offset), local].join("::"),
          parsed[1],
        );
      }
    }
  };
  const rootPattern = [...roots].map(regexEscaped).join("|");
  for (const imported of syntax.matchAll(
    new RegExp(
      `\\buse\\s+(?:::)?(${rootPattern})\\s*::\\s*prelude\\s+as\\s+([A-Za-z_][A-Za-z0-9_]*)\\s*;`,
      "g",
    ),
  )) {
    if (
      imported.index === undefined
      || !activeIn(rootScopes, imported[1], imported.index)
    ) continue;
    addScope(preludeModuleScopes, imported[2], imported.index);
    if (isModuleScopeAt(imported.index)) {
      qualifiedPreludeAliases.add(
        [...modulePathAt(imported.index), imported[2]].join("::"),
      );
    }
  }
  for (const imported of syntax.matchAll(
    new RegExp(
      `\\buse\\s+(?:::)?(${rootPattern})\\s*::\\s*((?:prelude\\s*::\\s*)?(?:[A-Za-z_][A-Za-z0-9_]*|\\*))(?:\\s+as\\s+([A-Za-z_][A-Za-z0-9_]*))?\\s*;`,
      "g",
    ),
  )) {
    if (
      imported.index !== undefined
      && activeIn(rootScopes, imported[1], imported.index)
    ) {
      importItem(
        `${imported[2]}${imported[3] ? ` as ${imported[3]}` : ""}`,
        imported.index,
      );
    }
  }
  for (const grouped of syntax.matchAll(
    new RegExp(
      `\\buse\\s+(?:::)?(${rootPattern})\\s*::\\s*(?:(prelude)\\s*::\\s*)?\\{([\\s\\S]*?)\\}\\s*;`,
      "g",
    ),
  )) {
    if (grouped.index === undefined) throw new Error("sqlx use group has no offset");
    if (!activeIn(rootScopes, grouped[1], grouped.index)) continue;
    for (const item of splitTopLevel(grouped[3])) {
      importItem(`${grouped[2] ? "prelude::" : ""}${item}`, grouped.index);
    }
  }

  type PreservedUseMapping = {
    source: string[];
    exported: string;
  };
  const preservedUseMappings = (
    tree: string,
    inherited: readonly string[] = [],
  ): PreservedUseMapping[] => {
    const value = tree.trim();
    const brace = value.indexOf("{");
    if (brace >= 0) {
      const close = closingDelimiter(value, brace, "{", "}");
      if (value.slice(close + 1).trim() !== "") return [];
      const prefix = value.slice(0, brace).trim().replace(/::$/, "");
      const segments = prefix
        ? prefix.split(/\s*::\s*/).filter(Boolean)
        : [];
      return splitTopLevel(value.slice(brace + 1, close))
        .flatMap((child) =>
          preservedUseMappings(child, [...inherited, ...segments]));
    }
    const alias = value.match(
      /\bas\s+([A-Za-z_][A-Za-z0-9_]*)$/,
    )?.[1];
    const withoutAlias = value.replace(
      /\s+as\s+[A-Za-z_][A-Za-z0-9_]*$/,
      "",
    );
    const segments = withoutAlias.split(/\s*::\s*/).filter(Boolean);
    const source = [...inherited, ...segments];
    const leaf = source.at(-1);
    return leaf ? [{ source, exported: alias ?? leaf }] : [];
  };
  const resolveRustPath = (
    offset: number,
    segments: readonly string[],
    usePath = false,
  ): string[] => {
    const remaining = [...segments];
    if (remaining[0] === "crate") {
      remaining.shift();
      return remaining;
    }
    let base = usePath ? [] : modulePathAt(offset);
    if (remaining[0] === "self") {
      remaining.shift();
      base = modulePathAt(offset);
    }
    if (remaining[0] === "super") {
      base = modulePathAt(offset);
    }
    while (remaining[0] === "super") {
      remaining.shift();
      base = base.slice(0, -1);
    }
    return [...base, ...remaining];
  };
  const consumerKind =
    /^(?:raw_sql|query(?:_as|_scalar)?(?:_with)?|query_file(?:_as|_scalar)?)$/;
  const publicUseDeclarations = [...syntax.matchAll(
    /\bpub(?:\s*\([^)]*\))?\s+use\s+([\s\S]*?);/g,
  )].flatMap((declaration) => {
    if (declaration.index === undefined) return [];
    return preservedUseMappings(declaration[1]).map((mapping) => ({
      ...mapping,
      offset: declaration.index!,
    }));
  });
  for (;;) {
    let changed = false;
    for (const declaration of publicUseDeclarations) {
      const leaf = declaration.source.at(-1) ?? "";
      const prefix = declaration.source.slice(0, -1);
      const resolvedSource = resolveRustPath(
        declaration.offset,
        declaration.source,
        true,
      );
      const exported = [
        ...modulePathAt(declaration.offset),
        declaration.exported,
      ].join("::");
      const exportsRoot = (
        declaration.source.length === 1
        && activeIn(rootScopes, leaf, declaration.offset)
      ) || qualifiedRootAliases.has(resolvedSource.join("::"));
      if (exportsRoot && !qualifiedRootAliases.has(exported)) {
        qualifiedRootAliases.add(exported);
        changed = true;
      }
      let kind: string | undefined;
      if (
        consumerKind.test(leaf)
        && prefix.length === 1
        && activeIn(rootScopes, prefix[0], declaration.offset)
      ) {
        kind = leaf;
      } else {
        kind = qualifiedFunctionAliases.get(resolvedSource.join("::"));
        if (
          !kind
          && consumerKind.test(resolvedSource.at(-1) ?? "")
          && qualifiedRootAliases.has(
            resolvedSource.slice(0, -1).join("::"),
          )
        ) kind = resolvedSource.at(-1);
      }
      if (!kind) continue;
      if (qualifiedFunctionAliases.get(exported) !== kind) {
        qualifiedFunctionAliases.set(exported, kind);
        changed = true;
      }
    }
    if (!changed) break;
  }
  const importPreludePath = (
    rawPath: string,
    item: string,
    offset: number,
  ): void => {
    const segments = rawPath.split(/\s*::\s*/).filter(Boolean);
    const local = segments.at(-1) ?? "";
    const resolved = resolveRustPath(offset, segments, true).join("::");
    if (
      !activeIn(preludeModuleScopes, local, offset)
      && !qualifiedPreludeAliases.has(resolved)
    ) return;
    importItem(`prelude::${item}`, offset);
  };
  for (const imported of syntax.matchAll(
    /\buse\s+((?:(?:crate|self|super|[A-Za-z_][A-Za-z0-9_]*)\s*::\s*)+)(\*|Executor(?:\s+as\s+[A-Za-z_][A-Za-z0-9_]*)?)\s*;/g,
  )) {
    if (imported.index !== undefined) {
      importPreludePath(imported[1], imported[2], imported.index);
    }
  }
  for (const grouped of syntax.matchAll(
    /\buse\s+((?:(?:crate|self|super|[A-Za-z_][A-Za-z0-9_]*)\s*::\s*)+)\{([\s\S]*?)\}\s*;/g,
  )) {
    if (grouped.index === undefined) continue;
    for (const item of splitTopLevel(grouped[2])) {
      if (/^(?:\*|Executor(?:\s+as\s+[A-Za-z_][A-Za-z0-9_]*)?)$/.test(item)) {
        importPreludePath(grouped[1], item, grouped.index);
      }
    }
  }
  for (const alias of syntax.matchAll(
    new RegExp(
      `\\btype\\s+([A-Za-z_][A-Za-z0-9_]*)(?:\\s*<[^;{}]*?>)?\\s*=\\s*(${rootPattern})\\s*::\\s*QueryBuilder\\b`,
      "g",
    ),
  )) {
    if (
      alias.index === undefined
      || !activeIn(rootScopes, alias[2], alias.index)
    ) continue;
    builderTypes.add(alias[1]);
    addScope(builderTypeScopes, alias[1], alias.index);
  }
  const constructorTypePattern = [
    ...[...roots].map((root) =>
      `${regexEscaped(root)}\\s*::\\s*QueryBuilder`),
    ...[...builderTypes].map(regexEscaped),
  ].join("|");
  for (const alias of syntax.matchAll(
    new RegExp(
      `\\blet\\s+(?:mut\\s+)?([A-Za-z_][A-Za-z0-9_]*)[^;=]*=\\s*(${constructorTypePattern})(?:\\s*::\\s*<[^;{}]*?>)?\\s*::\\s*(new|with_arguments)\\s*;`,
      "g",
    ),
  )) {
    if (alias.index === undefined) continue;
    const type = normalized(alias[2]).replace(/\s+/g, "");
    const root = type.match(
      /^([A-Za-z_][A-Za-z0-9_]*)::QueryBuilder$/,
    )?.[1];
    if (
      root
        ? !activeIn(rootScopes, root, alias.index)
        : !activeIn(builderTypeScopes, type, alias.index)
    ) continue;
    const bindings = localFunctions.get(alias[1]) ?? [];
    bindings.push({
      kind: `query_builder_${alias[3]}`,
      scope: scopeAt(alias.index),
      visibleFrom: alias.index,
      bindingOffset: alias.index,
    });
    localFunctions.set(alias[1], bindings);
  }
  for (const alias of syntax.matchAll(
    new RegExp(
      `\\blet\\s+(?:mut\\s+)?([A-Za-z_][A-Za-z0-9_]*)[^;=]*=\\s*((?:(?:crate|self|super|[A-Za-z_][A-Za-z0-9_]*)\\s*::\\s*)+)(${consumerKind.source.slice(1, -1)})(?:\\s*::\\s*<[^;{}]*?>)?\\s*;`,
      "g",
    ),
  )) {
    if (alias.index === undefined) continue;
    const prefix = alias[2].split(/\s*::\s*/).filter(Boolean);
    const resolved = resolveRustPath(
      alias.index,
      [...prefix, alias[3]],
    );
    let kind = qualifiedFunctionAliases.get(resolved.join("::"));
    if (
      !kind
      && prefix.length === 1
      && activeIn(rootScopes, prefix[0], alias.index)
    ) kind = alias[3];
    if (
      !kind
      && qualifiedRootAliases.has(
        resolved.slice(0, -1).join("::"),
      )
    ) kind = alias[3];
    if (!kind) continue;
    const bindings = localFunctions.get(alias[1]) ?? [];
    bindings.push({
      kind,
      scope: scopeAt(alias.index),
      visibleFrom: alias.index,
      bindingOffset: alias.index,
    });
    localFunctions.set(alias[1], bindings);
  }

  const associatedItemOpens = new Set(
    [...syntax.matchAll(/\b(?:impl|trait)\b[^;{]*\{/g)]
      .filter((match) => match.index !== undefined)
      .map((match) =>
        (match.index ?? 0) + match[0].lastIndexOf("{")),
  );
  const localFunctionScopes = new Map<string, LexicalScope[]>();
  for (const fn of rustFunctions(production)) {
    const scope = scopeAt(fn.declarationStart);
    if (associatedItemOpens.has(scope.open)) continue;
    const bindings = localFunctionScopes.get(fn.name) ?? [];
    bindings.push(scope);
    localFunctionScopes.set(fn.name, bindings);
  }
  const importedFunctionIsShadowed = (
    name: string,
    offset: number,
    bindingOffset: number | undefined,
  ): boolean => {
    const value = visibleStringDefinition(definitions, name, offset);
    return (
      value !== undefined
      && value.bindingOffset !== bindingOffset
    ) || activeIn(localFunctionScopes, name, offset);
  };

  const qualifiedConsumers = new RegExp(
    `\\b(${rootPattern})\\s*::\\s*(raw_sql|query(?:_as|_scalar)?(?:_with)?|query_file(?:_as|_scalar)?)(?:\\s*::\\s*<[^;{}]*?>)?\\s*(!)?\\s*\\(`,
    "g",
  );
  for (const call of syntax.matchAll(qualifiedConsumers)) {
    if (call.index === undefined) throw new Error("sqlx call has no offset");
    if (!activeIn(rootScopes, call[1], call.index)) continue;
    const open = call.index + call[0].lastIndexOf("(");
    const close = closingDelimiter(syntax, open, "(", ")");
    if (call[3] || call[2].startsWith("query_file")) {
      addUnresolved(
        `${call[2]}_${call[3] ? "macro" : "call"}`,
        production.slice(open + 1, close),
        open,
      );
      inspectedOpens.add(open);
    } else {
      inspectCall(
        call[2],
        open,
        close,
        0,
        call[2].endsWith("_with") ? 2 : 1,
      );
    }
  }
  for (const [local, bindings] of localFunctions) {
    const calls = new RegExp(
      `(?<![A-Za-z0-9_:.])${regexEscaped(local)}(?:\\s*::\\s*<[^;{}]*?>)?\\s*(!)?\\s*\\(`,
      "g",
    );
    for (const call of syntax.matchAll(calls)) {
      if (call.index === undefined) throw new Error("imported sqlx call has no offset");
      const active = bindings
        .filter(({ scope, visibleFrom }) =>
          visibleFrom < call.index! && inScope(scope, call.index!))
        .sort((left, right) =>
          right.scope.open - left.scope.open
          || right.visibleFrom - left.visibleFrom);
      if (active.length === 0) continue;
      if (
        importedFunctionIsShadowed(
          local,
          call.index,
          active[0].bindingOffset,
        )
      ) continue;
      const kind = active[0].kind;
      const open = call.index + call[0].lastIndexOf("(");
      const close = closingDelimiter(syntax, open, "(", ")");
      if (call[1] || kind.startsWith("query_file")) {
        addUnresolved(
          `${kind}_${call[1] ? "macro" : "call"}`,
          production.slice(open + 1, close),
          open,
        );
        inspectedOpens.add(open);
      } else {
        inspectCall(
          kind,
          open,
          close,
          0,
          kind.endsWith("_with")
              || kind === "query_builder_with_arguments"
            ? 2
            : 1,
        );
      }
    }
  }
  for (const call of syntax.matchAll(
    /(?<![A-Za-z0-9_])((?:(?:crate|self|super|[A-Za-z_][A-Za-z0-9_]*)\s*::\s*)+)([A-Za-z_][A-Za-z0-9_]*)(?:\s*::\s*<[^;{}]*?>)?\s*(!)?\s*\(/g,
  )) {
    if (call.index === undefined) {
      throw new Error("qualified SQL alias call has no offset");
    }
    const open = call.index + call[0].lastIndexOf("(");
    if (inspectedOpens.has(open)) continue;
    const prefix = call[1].split(/\s*::\s*/).filter(Boolean);
    const resolved = resolveRustPath(
      call.index,
      [...prefix, call[2]],
    );
    let kind = qualifiedFunctionAliases.get(resolved.join("::"));
    if (
      !kind
      && consumerKind.test(call[2])
      && qualifiedRootAliases.has(
        resolved.slice(0, -1).join("::"),
      )
    ) kind = call[2];
    if (!kind) continue;
    const close = closingDelimiter(syntax, open, "(", ")");
    if (call[3] || kind.startsWith("query_file")) {
      addUnresolved(
        `${kind}_${call[3] ? "macro" : "call"}`,
        production.slice(open + 1, close),
        open,
      );
      inspectedOpens.add(open);
    } else {
      inspectCall(
        kind,
        open,
        close,
        0,
        kind.endsWith("_with") ? 2 : 1,
      );
    }
  }
  type ValueBinding = {
    name: string;
    visibleFrom: number;
    scopeClose: number;
    moduleKey: string;
  };
  const builderBindings: ValueBinding[] = [];
  const addBuilder = (
    name: string,
    visibleFrom: number,
    scopeClose: number,
  ): boolean => {
    if (builderBindings.some((binding) =>
      binding.name === name
      && binding.visibleFrom === visibleFrom
      && binding.scopeClose === scopeClose)) return false;
    builderBindings.push({
      name,
      visibleFrom,
      scopeClose,
      moduleKey: moduleKeyAt(visibleFrom),
    });
    return true;
  };
  const isBuilderAt = (name: string, offset: number): boolean =>
    builderBindings.some((binding) =>
      binding.name === name
      && binding.visibleFrom <= offset
      && offset < binding.scopeClose
      && binding.moduleKey === moduleKeyAt(offset));
  const builderTypePattern = [
    ...[...roots].map((root) => `${regexEscaped(root)}\\s*::\\s*QueryBuilder`),
    ...[...builderTypes].map(regexEscaped),
  ].join("|");
  for (const declaration of syntax.matchAll(
    new RegExp(
      `\\blet\\s+(?:mut\\s+)?([A-Za-z_][A-Za-z0-9_]*)[^;=]*=\\s*(${builderTypePattern})(?:\\s*::\\s*<[^;{}]*?>)?\\s*::\\s*(new|with_arguments)\\s*\\(`,
      "g",
    ),
  )) {
    if (declaration.index === undefined) {
      throw new Error("QueryBuilder declaration has no offset");
    }
    const type = normalized(declaration[2]).replace(/\s+/g, "");
    const root = type.match(/^([A-Za-z_][A-Za-z0-9_]*)::QueryBuilder$/)?.[1];
    if (
      root
        ? !activeIn(rootScopes, root, declaration.index)
        : !activeIn(builderTypeScopes, type, declaration.index)
    ) continue;
    addBuilder(
      declaration[1],
      declaration.index,
      scopeAt(declaration.index).close,
    );
    const open = declaration.index + declaration[0].lastIndexOf("(");
    inspectCall(
      `query_builder_${declaration[3]}`,
      open,
      closingDelimiter(syntax, open, "(", ")"),
      0,
      declaration[3] === "new" ? 1 : 2,
    );
  }
  for (const parameter of syntax.matchAll(
    new RegExp(
      `\\b([A-Za-z_][A-Za-z0-9_]*)\\s*:\\s*&mut\\s+(${builderTypePattern})\\b`,
      "g",
    ),
  )) {
    if (parameter.index === undefined) throw new Error("QueryBuilder parameter has no offset");
    const type = normalized(parameter[2]).replace(/\s+/g, "");
    const root = type.match(/^([A-Za-z_][A-Za-z0-9_]*)::QueryBuilder$/)?.[1];
    if (
      root
        ? !activeIn(rootScopes, root, parameter.index)
        : !activeIn(builderTypeScopes, type, parameter.index)
    ) continue;
    const owner = rustFunctions(production).find((fn) =>
      fn.declarationStart <= parameter.index && parameter.index < fn.bodyStart);
    if (!owner) continue;
    addBuilder(parameter[1], owner.bodyStart, owner.bodyEnd);
  }

  for (;;) {
    let changed = false;
    for (const alias of syntax.matchAll(
      /\blet\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)[^;=]*=\s*&?\s*(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*;/g,
    )) {
      if (alias.index === undefined || !isBuilderAt(alias[2], alias.index)) {
        continue;
      }
      changed = addBuilder(
        alias[1],
        alias.index,
        scopeAt(alias.index).close,
      ) || changed;
    }
    for (const separated of syntax.matchAll(
      /\blet\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)[^;=]*=\s*([A-Za-z_][A-Za-z0-9_]*)\s*\.\s*separated\s*\(/g,
    )) {
      if (
        separated.index === undefined
        || !isBuilderAt(separated[2], separated.index)
      ) continue;
      const open = separated.index + separated[0].lastIndexOf("(");
      inspectCall(
        "query_builder_separated",
        open,
        closingDelimiter(syntax, open, "(", ")"),
      );
      changed = addBuilder(
        separated[1],
        separated.index,
        scopeAt(separated.index).close,
      ) || changed;
    }
    if (!changed) break;
  }

  for (const fragment of syntax.matchAll(
    /\b([A-Za-z_][A-Za-z0-9_]*)\s*\.\s*(push|push_unseparated|separated)\s*\(/g,
  )) {
    if (
      fragment.index === undefined
      || !isBuilderAt(fragment[1], fragment.index)
    ) continue;
    const open = fragment.index + fragment[0].lastIndexOf("(");
    inspectCall(
      `query_builder_${fragment[2]}`,
      open,
      closingDelimiter(syntax, open, "(", ")"),
    );
  }

  const queryObjectBindings: ValueBinding[] = [];
  const isQueryObjectAt = (name: string, offset: number): boolean =>
    queryObjectBindings.some((binding) =>
      binding.name === name
      && binding.visibleFrom <= offset
      && offset < binding.scopeClose
      && binding.moduleKey === moduleKeyAt(offset));
  for (const binding of syntax.matchAll(
    new RegExp(
      `\\blet\\s+(?:mut\\s+)?([A-Za-z_][A-Za-z0-9_]*)[^;=]*=\\s*(?:(?:${rootPattern})\\s*::\\s*)?(?:raw_sql|query(?:_as|_scalar)?(?:_with)?)(?:\\s*::\\s*<[^;{}]*?>)?\\s*\\(`,
      "g",
    ),
  )) {
    if (binding.index === undefined) throw new Error("query binding has no offset");
    queryObjectBindings.push({
      name: binding[1],
      visibleFrom: binding.index,
      scopeClose: scopeAt(binding.index).close,
      moduleKey: moduleKeyAt(binding.index),
    });
  }

  const executorMethodPattern =
    "execute(?:_many)?|fetch(?:_many|_all|_one|_optional)?|prepare_with|describe";
  const executorTypePattern = [
    ...[...roots].map((root) => `${regexEscaped(root)}\\s*::\\s*Executor`),
    ...[...executorTraits].map(regexEscaped),
  ].join("|");
  const hasActiveExecutorTrait = (text: string, offset: number): boolean => {
    const qualified = normalized(text).replace(/\s+/g, "").match(
      new RegExp(`\\b(${rootPattern})::Executor\\b`),
    )?.[1];
    if (qualified && activeIn(rootScopes, qualified, offset)) return true;
    return [...executorTraits].some((trait) =>
      new RegExp(`\\b${regexEscaped(trait)}\\b`).test(text)
      && activeIn(executorTraitScopes, trait, offset));
  };
  for (const ufcs of syntax.matchAll(
    new RegExp(
      `<[^;{}>]{0,240}?\\bas\\s+([^>]+)>\\s*::\\s*(${executorMethodPattern})\\s*\\(`,
      "g",
    ),
  )) {
    if (ufcs.index === undefined) throw new Error("Executor UFCS call has no offset");
    if (!hasActiveExecutorTrait(ufcs[1], ufcs.index)) continue;
    const open = ufcs.index + ufcs[0].lastIndexOf("(");
    inspectCall(
      `executor_${ufcs[2]}`,
      open,
      closingDelimiter(syntax, open, "(", ")"),
      1,
      undefined,
    );
  }
  for (const direct of syntax.matchAll(
    new RegExp(
      `\\b((?:${executorTypePattern}))\\s*::\\s*(${executorMethodPattern})\\s*\\(`,
      "g",
    ),
  )) {
    if (direct.index === undefined) throw new Error("Executor call has no offset");
    if (!hasActiveExecutorTrait(direct[1], direct.index)) continue;
    const open = direct.index + direct[0].lastIndexOf("(");
    inspectCall(
      `executor_${direct[2]}`,
      open,
      closingDelimiter(syntax, open, "(", ")"),
      1,
      undefined,
    );
  }
  if (executorTraits.size > 0) {
    for (const method of syntax.matchAll(
      new RegExp(
        `\\b([A-Za-z_][A-Za-z0-9_]*)\\s*\\.\\s*(${executorMethodPattern})\\s*\\(`,
        "g",
      ),
    )) {
      if (
        method.index === undefined
        || isBuilderAt(method[1], method.index)
        || isQueryObjectAt(method[1], method.index)
        || ![...executorTraits].some((trait) =>
          activeIn(executorTraitScopes, trait, method.index))
      ) continue;
      const open = method.index + method[0].lastIndexOf("(");
      inspectCall(
        `executor_${method[2]}`,
        open,
        closingDelimiter(syntax, open, "(", ")"),
        0,
        undefined,
      );
    }
  }
  const result = {
    sql,
    unresolved,
    unresolvedDetails,
    resolved: resolvedConsumers,
  };
  sqlConsumerCache.set(source, result);
  sqlConsumerCache.set(production, result);
  return result;
}

function sqlTables(source: string): string[] {
  return [...new Set(
    sourceSqlOccurrences(source).map(({ table }) => table),
  )];
}

function sourceSqlOccurrences(
  source: string,
): Array<SqlReference & { sourceOffset: number; sql: string }> {
  const production = productionRust(source);
  return sqlConsumerAnalysis(production).resolved.flatMap((consumer) =>
    sqlReferences(consumer.sql).map((reference) => ({
      ...reference,
      sourceOffset: consumer.offset,
      sql: consumer.sql,
    })));
}

function sourceSqlReferences(source: string): SqlReference[] {
  const unique = new Map<string, SqlReference>();
  for (const reference of sourceSqlOccurrences(source)) {
    unique.set(
      `${reference.operation}:${reference.table}`,
      { operation: reference.operation, table: reference.table },
    );
  }
  return [...unique.values()];
}

const executableFingerprint = (source: string): string =>
  createHash("sha256")
    .update(normalized(maskRustNonCode(source, false)))
    .digest("hex");

function unresolvedSqlInventory(
  relative: string,
  source: string,
): string[] {
  const production = productionRust(source);
  const functions = rustFunctions(production);
  const modules = inlineModuleRegions(production);
  const sourceFingerprint = executableFingerprint(production);
  return sqlConsumerAnalysis(source).unresolvedDetails.map((consumer) => {
    const owner = functions
      .filter((fn) =>
        fn.bodyStart < consumer.offset && consumer.offset < fn.bodyEnd)
      .sort((left, right) =>
        (left.bodyEnd - left.bodyStart) - (right.bodyEnd - right.bodyStart))[0];
    const modulePath = modules
      .filter(({ open, close }) =>
        open < consumer.offset && consumer.offset < close)
      .sort((left, right) => left.open - right.open)
      .map(({ name }) => name);
    const functionName = [
      ...modulePath,
      owner?.name ?? "<module>",
    ].join("::");
    const ownerFingerprint = executableFingerprint(owner?.body ?? production);
    return [
      relative,
      functionName,
      ownerFingerprint,
      sourceFingerprint,
      consumer.kind,
      consumer.expression,
    ].join(":");
  });
}

function rustTestFunctions(source: string): RustFunction[] {
  const commentsOnly = maskRustNonCode(source, false);
  return rustFunctions(source).filter((fn) =>
    /#\s*\[\s*(?:(?:tokio|sqlx)\s*::\s*)?test\b/.test(
      immediateRustAttributes(commentsOnly, fn.declarationStart),
    ));
}

function foreignTestExceptionNames(
  source: string,
  allowedTables: ReadonlySet<string>,
): string[] {
  return rustTestFunctions(source)
    .filter((fn) =>
      sourceSqlOccurrences(fn.body)
        .some(({ table }) => !allowedTables.has(table)))
    .map(({ name }) => name)
    .sort();
}

function portableProductionFiles(): SourceFile[] {
  return portableFiles().filter(({ relative }) =>
    !relative.includes("/tests/")
    && !path.posix.basename(relative).startsWith("tests")
    && path.posix.basename(relative) !== "test_schema.rs");
}

function exactRustPathLiteral(expression: string): string | undefined {
  const value = expression.trim();
  const raw = value.match(/^r(#+)?"([\s\S]*)"\1$/);
  if (raw) return raw[2];
  if (!/^"(?:\\.|[^"\\])*"$/.test(value)) return undefined;
  return value.slice(1, -1)
    .replace(/\\\r?\n[ \t]*/g, "")
    .replace(/\\n/g, "\n")
    .replace(/\\r/g, "\r")
    .replace(/\\t/g, "\t")
    .replace(/\\"/g, '"')
    .replace(/\\\\/g, "\\");
}

function productionModuleFiles(
  root: string,
  rootRelatives: readonly string[],
): SourceFile[] {
  const normalizeRelative = (relative: string): string =>
    normalizedRelative(relative);
  const byRelative = new Map(
    sourceFiles(root, "app").map((file) => [
      normalizeRelative(file.relative),
      { ...file, relative: normalizeRelative(file.relative) },
    ]),
  );
  type State = { relative: string; moduleDir: string };
  const queue: State[] = rootRelatives.map((relative) => ({
    relative,
    moduleDir: "",
  }));
  const reached = new Set<string>();
  const visited = new Set<string>();

  const enqueue = (relative: string, moduleDir: string): void => {
    const normalizedPath = normalizeRelative(relative);
    if (
      normalizedPath.startsWith("../")
      || path.posix.isAbsolute(normalizedPath)
    ) {
      throw new Error(`production module graph escapes app root: ${relative}`);
    }
    if (!byRelative.has(normalizedPath)) {
      throw new Error(`production module graph reaches missing ${normalizedPath}`);
    }
    queue.push({
      relative: normalizedPath,
      moduleDir: moduleDir === "." ? "" : normalizeRelative(moduleDir),
    });
  };

  for (let queueIndex = 0; queueIndex < queue.length; queueIndex += 1) {
    const state = queue[queueIndex];
    const key = `${state.relative}\0${state.moduleDir}`;
    if (visited.has(key)) continue;
    visited.add(key);
    reached.add(state.relative);
    const file = byRelative.get(state.relative);
    if (!file) throw new Error(`missing queued production source ${state.relative}`);
    const production = productionRust(file.source);
    const syntax = maskRustNonCode(production, true);
    const commentsOnly = maskRustNonCode(production, false);
    const inlineModules = [...syntax.matchAll(
      /\bmod\s+([A-Za-z_][A-Za-z0-9_]*)\s*\{/g,
    )].map((match) => {
      if (match.index === undefined) {
        throw new Error("inline production module has no offset");
      }
      const open = match.index + match[0].lastIndexOf("{");
      return {
        name: match[1],
        open,
        close: closingDelimiter(syntax, open, "{", "}"),
      };
    });
    const moduleDirAt = (offset: number): string => path.posix.join(
      state.moduleDir,
      ...inlineModules
        .filter(({ open, close }) => offset > open && offset < close)
        .sort((left, right) => left.open - right.open)
        .map(({ name }) => name),
    );

    for (const declaration of syntax.matchAll(
      /\bmod\s+([A-Za-z_][A-Za-z0-9_]*)\s*;/g,
    )) {
      if (declaration.index === undefined) {
        throw new Error("external production module has no offset");
      }
      const name = declaration[1];
      const logicalDir = moduleDirAt(declaration.index);
      const attributes = immediateRustAttributes(
        commentsOnly,
        declaration.index,
      );
      const pathExpression = attributes.match(
        /#\s*\[\s*path\s*=\s*((?:r#+?"[\s\S]*?"#+)|(?:"(?:\\.|[^"\\])*"))\s*\]/,
      )?.[1];
      if (pathExpression) {
        const override = exactRustPathLiteral(pathExpression);
        if (override === undefined) {
          throw new Error(
            `${state.relative} mod ${name} has a nonliteral path override`,
          );
        }
        const relative = path.posix.join(
          path.posix.dirname(state.relative),
          override,
        );
        enqueue(relative, path.posix.join(logicalDir, name));
        continue;
      }
      const candidates = [
        path.posix.join(logicalDir, `${name}.rs`),
        path.posix.join(logicalDir, name, "mod.rs"),
      ].filter((candidate) => byRelative.has(normalizeRelative(candidate)));
      if (candidates.length !== 1) {
        throw new Error(
          `${state.relative} production mod ${name} resolves to ${candidates.length} files`,
        );
      }
      enqueue(candidates[0], path.posix.join(logicalDir, name));
    }

    for (const include of syntax.matchAll(/\binclude\s*!\s*\(/g)) {
      if (include.index === undefined) {
        throw new Error("production include! has no offset");
      }
      const open = include.index + include[0].lastIndexOf("(");
      const close = closingDelimiter(syntax, open, "(", ")");
      const included = exactRustPathLiteral(
        file.source.slice(open + 1, close),
      );
      if (included === undefined) {
        throw new Error(
          `${state.relative} has a nonliteral production include! edge`,
        );
      }
      enqueue(
        path.posix.join(path.posix.dirname(state.relative), included),
        moduleDirAt(include.index),
      );
    }
  }

  return [...reached].sort().map((relative) => byRelative.get(relative)!);
}

function appProductionFiles(): SourceFile[] {
  const portableAppPaths = new Set(
    extracted
      ? []
      : portableProductionFiles()
        .map(({ relative }) => `analysis/${relative}`),
  );
  const files = productionModuleFiles(
    path.join(repoRoot, "src-tauri/src"),
    ["lib.rs", "main.rs"],
  ).filter(({ relative }) => !portableAppPaths.has(relative));
  if (!files.some(({ relative }) => relative === "analysis/mod.rs")) {
    throw new Error("production app graph did not reach analysis/mod.rs");
  }
  return files;
}

function commandInventory(): {
  analysis: string[];
  project: string[];
  dev: string[];
} {
  const block = textBlockAfter("assert these exact sets:", "text");
  const sections = [
    ["analysis", 21],
    ["project", 3],
    ["dev", 3],
  ] as const;
  const result = { analysis: [] as string[], project: [] as string[], dev: [] as string[] };
  for (let index = 0; index < sections.length; index += 1) {
    const [name, count] = sections[index];
    const marker = name === "analysis"
      ? "analysis release (21):"
      : name === "project"
        ? "project release (3):"
        : "dev (3):";
    const start = block.indexOf(marker);
    const end = index + 1 < sections.length
      ? block.indexOf(
        sections[index + 1][0] === "project"
          ? "project release (3):"
          : "dev (3):",
        start + marker.length,
      )
      : block.length;
    if (start < 0 || end <= start) throw new Error(`missing command section ${name}`);
    result[name].push(
      ...block.slice(start + marker.length, end)
        .split("\n")
        .map((line) => line.trim())
        .filter((line) => /^[a-z_][a-z0-9_]*$/.test(line)),
    );
    if (result[name].length !== count) {
      throw new Error(`${name} command inventory drift: ${result[name].length}`);
    }
  }
  return result;
}

const commands = commandInventory();

const commandWireContracts: Record<string, [string, string[]]> = {
  list_analysis_sources: ["handle: AppHandle, repair_state: tauri::State<'_, SourceIdentityRepairState> -> AppResult<Vec<AnalysisSourceOption>>", []],
  list_analysis_runs: ["handle: AppHandle, source_id: Option<i64>, source_group_id: Option<i64>, limit: Option<i64>, query: Option<String>, status: Option<String>, provider: Option<String>, model: Option<String>, template: Option<String>, date_from: Option<String>, date_to: Option<String> -> AppResult<Vec<AnalysisRunSummary>>", ["sourceId", "sourceGroupId", "limit", "query", "status", "provider", "model", "template", "dateFrom", "dateTo"]],
  list_active_analysis_runs: ["handle: AppHandle, state: tauri::State<'_, AnalysisState> -> AppResult<Vec<AnalysisRunSummary>>", []],
  get_analysis_run: ["handle: AppHandle, run_id: i64 -> AppResult<Option<AnalysisRunDetail>>", ["runId"]],
  list_analysis_run_messages: ["handle: AppHandle, run_id: i64, after: Option<AnalysisRunMessageCursor>, limit: Option<i64>, source_id: Option<i64>, around_ref: Option<String> -> AppResult<AnalysisRunMessagesPage>", ["runId", "after", "limit", "sourceId", "aroundRef"]],
  get_analysis_run_trace: ["handle: AppHandle, run_id: i64 -> AppResult<AnalysisTraceData>", ["runId"]],
  delete_analysis_run: ["handle: AppHandle, state: tauri::State<'_, AnalysisState>, run_id: i64 -> AppResult<()>", ["runId"]],
  resolve_analysis_trace_refs: ["handle: AppHandle, run_id: i64, refs: Vec<String> -> AppResult<Vec<AnalysisTraceRef>>", ["runId", "refs"]],
  list_analysis_prompt_templates: ["handle: AppHandle, template_kind: Option<String> -> AppResult<Vec<AnalysisPromptTemplate>>", ["templateKind"]],
  create_analysis_prompt_template: ["handle: AppHandle, name: String, template_kind: String, body: String -> AppResult<AnalysisPromptTemplate>", ["name", "templateKind", "body"]],
  update_analysis_prompt_template: ["handle: AppHandle, template_id: i64, name: String, body: String -> AppResult<AnalysisPromptTemplate>", ["templateId", "name", "body"]],
  delete_analysis_prompt_template: ["handle: AppHandle, template_id: i64 -> AppResult<()>", ["templateId"]],
  list_analysis_source_groups: ["handle: AppHandle -> AppResult<Vec<AnalysisSourceGroup>>", []],
  create_analysis_source_group: ["handle: AppHandle, name: String, source_type: String, source_ids: Vec<i64> -> AppResult<AnalysisSourceGroup>", ["name", "sourceType", "sourceIds"]],
  update_analysis_source_group: ["handle: AppHandle, group_id: i64, name: String, source_type: String, source_ids: Vec<i64> -> AppResult<AnalysisSourceGroup>", ["groupId", "name", "sourceType", "sourceIds"]],
  delete_analysis_source_group: ["handle: AppHandle, group_id: i64 -> AppResult<()>", ["groupId"]],
  list_analysis_chat_messages: ["handle: AppHandle, run_id: i64 -> AppResult<Vec<AnalysisChatMessage>>", ["runId"]],
  clear_analysis_chat_messages: ["handle: AppHandle, run_id: i64 -> AppResult<()>", ["runId"]],
  ask_analysis_run_question: ["handle: AppHandle, run_id: i64, question: String, model_override: Option<String>, profile_id: Option<String> -> AppResult<String>", ["runId", "question", "modelOverride", "profileId"]],
  start_analysis_report: ["handle: AppHandle, state: tauri::State<'_, AnalysisState>, source_id: Option<i64>, source_group_id: Option<i64>, period_from: i64, period_to: i64, output_language: String, prompt_template_id: i64, model_override: Option<String>, profile_id: Option<String>, youtube_corpus_mode: Option<String>, include_migrated_history: bool -> AppResult<i64>", ["sourceId", "sourceGroupId", "periodFrom", "periodTo", "outputLanguage", "promptTemplateId", "modelOverride", "profileId", "youtubeCorpusMode", "includeMigratedHistory"]],
  cancel_analysis_run: ["handle: AppHandle, state: tauri::State<'_, AnalysisState>, scheduler: tauri::State<'_, Arc<LlmSchedulerState>>, run_id: i64 -> AppResult<()>", ["runId"]],
  start_project_analysis: ["handle: AppHandle, state: tauri::State<'_, crate::analysis::AnalysisState>, project_id: i64, period_from: i64, period_to: i64, output_language: String, prompt_template_id: i64, model_override: Option<String>, profile_id: Option<String>, youtube_corpus_mode: Option<String>, include_migrated_history: bool -> AppResult<i64>", ["projectId", "periodFrom", "periodTo", "outputLanguage", "promptTemplateId", "modelOverride", "profileId", "youtubeCorpusMode", "includeMigratedHistory"]],
  list_project_runs: ["handle: AppHandle, project_id: i64 -> AppResult<Vec<crate::analysis::models::AnalysisRunSummary>>", ["projectId"]],
  get_project_data_range: ["handle: AppHandle, project_id: i64, youtube_corpus_mode: Option<String>, include_migrated_history: bool -> AppResult<ProjectDataRange>", ["projectId", "youtubeCorpusMode", "includeMigratedHistory"]],
  seed_analysis_redesign_fixtures: ["handle: AppHandle, state: State<'_, AnalysisState> -> AppResult<AnalysisRedesignFixtureSummary>", []],
  clear_analysis_redesign_fixtures: ["handle: AppHandle, state: State<'_, AnalysisState> -> AppResult<AnalysisRedesignFixtureSummary>", []],
  clear_analysis_redesign_fixture_active_runs: ["handle: AppHandle, state: State<'_, AnalysisState> -> AppResult<()>", []],
};

function immediateRustAttributes(
  syntax: string,
  declarationStart: number,
): string {
  const attributes: string[] = [];
  let cursor = declarationStart;
  for (;;) {
    while (cursor > 0 && /\s/.test(syntax[cursor - 1])) cursor -= 1;
    if (syntax[cursor - 1] !== "]") break;
    const close = cursor;
    let depth = 1;
    let open = cursor - 2;
    while (open >= 0 && depth > 0) {
      if (syntax[open] === "]") depth += 1;
      else if (syntax[open] === "[") depth -= 1;
      open -= 1;
    }
    open += 1;
    if (depth !== 0 || open === 0 || syntax[open - 1] !== "#") break;
    attributes.unshift(syntax.slice(open - 1, close));
    cursor = open - 1;
  }
  return attributes.join("\n");
}

function commandFunctions(
  files: readonly SourceFile[] = sourceFiles(
    path.join(repoRoot, "src-tauri/src"),
    "app",
  ),
): Array<RustFunction & { file: SourceFile }> {
  return files.flatMap((file) => {
    const syntax = maskRustNonCode(file.source, true);
    const scopes = rustBraceRegions(file.source);
    type AttributeBinding = {
      name: string;
      scopeOpen: number;
      scopeClose: number;
    };
    const scopeAt = (offset: number) =>
      scopes
        .filter(({ open, close }) => open < offset && offset < close)
        .sort((left, right) => right.open - left.open)[0]
        ?? { open: -1, close: file.source.length };
    const active = (
      bindings: readonly AttributeBinding[],
      name: string,
      offset: number,
    ): boolean => bindings.some((binding) =>
      binding.name === name
      && binding.scopeOpen < offset
      && offset < binding.scopeClose);
    const roots: AttributeBinding[] = [{
      name: "tauri",
      scopeOpen: -1,
      scopeClose: file.source.length,
    }];
    const locals: AttributeBinding[] = [];
    for (const alias of syntax.matchAll(
      /\buse\s+(?:::)?tauri\s+as\s+([A-Za-z_][A-Za-z0-9_]*)\s*;/g,
    )) {
      if (alias.index === undefined) throw new Error("tauri alias has no offset");
      const scope = scopeAt(alias.index);
      roots.push({
        name: alias[1],
        scopeOpen: scope.open,
        scopeClose: scope.close,
      });
    }
    for (const grouped of syntax.matchAll(
      /\buse\s+(?:::)?tauri\s*::\s*\{([\s\S]*?)\}\s*;/g,
    )) {
      if (grouped.index === undefined) throw new Error("tauri self alias has no offset");
      const scope = scopeAt(grouped.index);
      for (const item of splitTopLevel(grouped[1])) {
        const selfAlias = item.match(
          /^self\s+as\s+([A-Za-z_][A-Za-z0-9_]*)$/,
        )?.[1];
        if (!selfAlias) continue;
        roots.push({
          name: selfAlias,
          scopeOpen: scope.open,
          scopeClose: scope.close,
        });
      }
    }
    const rootNames = [...new Set(roots.map(({ name }) => name))]
      .map(regexEscaped).join("|");
    const addLocal = (name: string, offset: number): void => {
      const scope = scopeAt(offset);
      locals.push({
        name,
        scopeOpen: scope.open,
        scopeClose: scope.close,
      });
    };
    for (const imported of syntax.matchAll(
      new RegExp(
        `\\buse\\s+(?:::)?(${rootNames})\\s*::\\s*command(?:\\s+as\\s+([A-Za-z_][A-Za-z0-9_]*))?\\s*;`,
        "g",
      ),
    )) {
      if (
        imported.index !== undefined
        && active(roots, imported[1], imported.index)
      ) addLocal(imported[2] ?? "command", imported.index);
    }
    for (const grouped of syntax.matchAll(
      new RegExp(
        `\\buse\\s+(?:::)?(${rootNames})\\s*::\\s*\\{([\\s\\S]*?)\\}\\s*;`,
        "g",
      ),
    )) {
      if (
        grouped.index === undefined
        || !active(roots, grouped[1], grouped.index)
      ) continue;
      for (const item of splitTopLevel(grouped[2])) {
        const command = item.match(
          /^command(?:\s+as\s+([A-Za-z_][A-Za-z0-9_]*))?$/,
        );
        if (command) addLocal(command[1] ?? "command", grouped.index);
      }
    }
    return rustFunctions(file.source).flatMap((fn) => {
      const attributes = immediateRustAttributes(
        syntax,
        fn.declarationStart,
      );
      const attributeBodies: string[] = [];
      for (let index = 0; index < attributes.length;) {
        const marker = attributes.indexOf("#", index);
        if (marker < 0) break;
        const open = attributes.indexOf("[", marker + 1);
        if (open < 0) break;
        const close = closingDelimiter(attributes, open, "[", "]");
        attributeBodies.push(attributes.slice(open + 1, close).trim());
        index = close + 1;
      }
      const attributeIsCommand = (body: string): boolean => {
        const cfgAttr = body.match(/^cfg_attr\s*\(/);
        if (cfgAttr) {
          const bodySyntax = maskRustNonCode(body, true);
          const open = body.indexOf("(", cfgAttr.index ?? 0);
          const close = closingDelimiter(bodySyntax, open, "(", ")");
          if (body.slice(close + 1).trim() !== "") return false;
          return splitRustExpressions(body.slice(open + 1, close))
            .slice(1)
            .some(attributeIsCommand);
        }
        const path = body.match(
          /^(?:::)?([A-Za-z_][A-Za-z0-9_]*(?:::[A-Za-z_][A-Za-z0-9_]*)*)\b/,
        )?.[1];
        if (!path) return false;
        const qualified = path.match(
          /^([A-Za-z_][A-Za-z0-9_]*)::command$/,
        );
        if (qualified) {
          return active(roots, qualified[1], fn.declarationStart);
        }
        return active(locals, path, fn.declarationStart);
      };
      const isCommand = attributeBodies.some(attributeIsCommand);
      return isCommand
        ? [{ ...fn, file }]
        : [];
    });
  });
}

function resolvedSqlxMacroInvocations(
  source: string,
  macroName: string,
): number[] {
  const production = productionRust(source);
  const syntax = maskRustNonCode(production, true);
  const scopes = rustBraceRegions(production);
  const modules = inlineModuleRegions(production);
  const moduleOpens = new Set(modules.map(({ open }) => open));
  const modulePathAt = (offset: number): string[] =>
    modules
      .filter(({ open, close }) => open < offset && offset < close)
      .sort((left, right) => left.open - right.open)
      .map(({ name }) => name);
  const moduleKeyAt = (offset: number): string =>
    modules
      .filter(({ open, close }) => open < offset && offset < close)
      .sort((left, right) => left.open - right.open)
      .map(({ open }) => String(open))
      .join("/");
  const isModuleScopeAt = (offset: number): boolean =>
    scopes
      .filter(({ open, close }) => open < offset && offset < close)
      .every(({ open }) => moduleOpens.has(open));
  type Scope = {
    open: number;
    close: number;
    moduleKey: string | null;
  };
  const scopeAt = (offset: number): Scope => {
    const scope = scopes
      .filter(({ open, close }) => open < offset && offset < close)
      .sort((left, right) => right.open - left.open)[0]
      ?? { open: -1, close: production.length };
    return { ...scope, moduleKey: moduleKeyAt(offset) };
  };
  const active = (
    bindings: ReadonlyMap<string, readonly Scope[]>,
    name: string,
    offset: number,
  ): boolean => (bindings.get(name) ?? []).some((scope) =>
    scope.open < offset
    && offset < scope.close
    && (
      scope.moduleKey === null
      || scope.moduleKey === moduleKeyAt(offset)
    ));
  const add = (
    bindings: Map<string, Scope[]>,
    name: string,
    offset: number,
  ): void => {
    const values = bindings.get(name) ?? [];
    values.push(scopeAt(offset));
    bindings.set(name, values);
  };
  const roots = new Map<string, Scope[]>([
    ["sqlx", [{
      open: -1,
      close: production.length,
      moduleKey: null,
    }]],
  ]);
  const qualifiedRoots = new Set<string>();
  for (const alias of syntax.matchAll(
    /\buse\s+(?:::)?sqlx\s+as\s+([A-Za-z_][A-Za-z0-9_]*)\s*;/g,
  )) {
    if (alias.index !== undefined) {
      add(roots, alias[1], alias.index);
      if (isModuleScopeAt(alias.index)) {
        qualifiedRoots.add(
          [...modulePathAt(alias.index), alias[1]].join("::"),
        );
      }
    }
  }
  for (const alias of syntax.matchAll(
    /\bextern\s+crate\s+sqlx\s+as\s+([A-Za-z_][A-Za-z0-9_]*)\s*;/g,
  )) {
    if (alias.index !== undefined) {
      add(roots, alias[1], alias.index);
      if (isModuleScopeAt(alias.index)) {
        qualifiedRoots.add(
          [...modulePathAt(alias.index), alias[1]].join("::"),
        );
      }
    }
  }
  for (const grouped of syntax.matchAll(
    /\buse\s+(?:::)?sqlx\s*::\s*\{([\s\S]*?)\}\s*;/g,
  )) {
    if (grouped.index === undefined) continue;
    for (const item of splitTopLevel(grouped[1])) {
      const alias = item.match(
        /^self\s+as\s+([A-Za-z_][A-Za-z0-9_]*)$/,
      )?.[1];
      if (alias) {
        add(roots, alias, grouped.index);
        if (isModuleScopeAt(grouped.index)) {
          qualifiedRoots.add(
            [...modulePathAt(grouped.index), alias].join("::"),
          );
        }
      }
    }
  }
  const rootPattern = [...roots.keys()].map(regexEscaped).join("|");
  const locals = new Map<string, Scope[]>();
  const qualifiedMacros = new Set<string>();
  for (const imported of syntax.matchAll(
    new RegExp(
      `\\buse\\s+(?:::)?(${rootPattern})\\s*::\\s*${regexEscaped(macroName)}(?:\\s+as\\s+([A-Za-z_][A-Za-z0-9_]*))?\\s*;`,
      "g",
    ),
  )) {
    if (
      imported.index !== undefined
      && active(roots, imported[1], imported.index)
    ) {
      const local = imported[2] ?? macroName;
      add(locals, local, imported.index);
      if (isModuleScopeAt(imported.index)) {
        qualifiedMacros.add(
          [...modulePathAt(imported.index), local].join("::"),
        );
      }
    }
  }
  for (const grouped of syntax.matchAll(
    new RegExp(
      `\\buse\\s+(?:::)?(${rootPattern})\\s*::\\s*\\{([\\s\\S]*?)\\}\\s*;`,
      "g",
    ),
  )) {
    if (
      grouped.index === undefined
      || !active(roots, grouped[1], grouped.index)
    ) continue;
    for (const item of splitTopLevel(grouped[2])) {
      const imported = item.match(
        new RegExp(
          `^${regexEscaped(macroName)}(?:\\s+as\\s+([A-Za-z_][A-Za-z0-9_]*))?$`,
        ),
      );
      if (imported) {
        const local = imported[1] ?? macroName;
        add(locals, local, grouped.index);
        if (isModuleScopeAt(grouped.index)) {
          qualifiedMacros.add(
            [...modulePathAt(grouped.index), local].join("::"),
          );
        }
      }
    }
  }
  const offsets = new Set<number>();
  for (const call of syntax.matchAll(
    new RegExp(
      `\\b(${rootPattern})\\s*::\\s*${regexEscaped(macroName)}\\s*!`,
      "g",
    ),
  )) {
    if (
      call.index !== undefined
      && active(roots, call[1], call.index)
    ) offsets.add(call.index);
  }
  for (const [local] of locals) {
    for (const call of syntax.matchAll(
      new RegExp(
        `(?<![A-Za-z0-9_:])${regexEscaped(local)}\\s*!`,
        "g",
      ),
    )) {
      if (
        call.index !== undefined
        && active(locals, local, call.index)
      ) offsets.add(call.index);
    }
  }
  const resolvePath = (
    offset: number,
    segments: readonly string[],
  ): string[] => {
    const remaining = [...segments];
    if (remaining[0] === "crate") {
      remaining.shift();
      return remaining;
    }
    let base = modulePathAt(offset);
    if (remaining[0] === "self") remaining.shift();
    while (remaining[0] === "super") {
      remaining.shift();
      base = base.slice(0, -1);
    }
    return [...base, ...remaining];
  };
  for (const call of syntax.matchAll(
    /(?<![A-Za-z0-9_])((?:(?:crate|self|super|[A-Za-z_][A-Za-z0-9_]*)\s*::\s*)+)([A-Za-z_][A-Za-z0-9_]*)\s*!/g,
  )) {
    if (call.index === undefined) continue;
    const prefix = call[1].split(/\s*::\s*/).filter(Boolean);
    const resolved = resolvePath(call.index, [...prefix, call[2]]);
    if (
      qualifiedMacros.has(resolved.join("::"))
      || (
        call[2] === macroName
        && qualifiedRoots.has(resolved.slice(0, -1).join("::"))
      )
    ) offsets.add(call.index);
  }
  return [...offsets].sort((left, right) => left - right);
}

function camelCase(value: string): string {
  return value.replace(/_([a-z])/g, (_, letter: string) => letter.toUpperCase());
}

function canonicalCommandSignature(value: string): string {
  return normalized(value)
    .replace(/,\s*->/g, " ->")
    .replace(/\btauri::State\b/g, "State")
    .replace(/\bcrate::analysis::models::/g, "")
    .replace(/\bcrate::analysis::/g, "")
    .replace(/\bextractum_analysis::/g, "");
}

function expectOrdered(source: string, markers: readonly string[]): void {
  let offset = 0;
  for (const marker of markers) {
    const found = source.indexOf(marker, offset);
    expect(found, marker).toBeGreaterThanOrEqual(offset);
    offset = found + marker.length;
  }
}

function handlerBody(): string {
  const appLib = read("src-tauri/src/lib.rs");
  const marker = "tauri::generate_handler![";
  const start = appLib.indexOf(marker);
  if (start < 0) throw new Error("missing generate_handler! registration");
  const open = start + marker.length - 1;
  const close = closingDelimiter(appLib, open, "[", "]");
  return appLib.slice(open + 1, close);
}

function occurrenceCount(source: string, pattern: RegExp): number {
  const flags = pattern.flags.includes("g") ? pattern.flags : `${pattern.flags}g`;
  return [...source.matchAll(new RegExp(pattern.source, flags))].length;
}

function awaitSiteCount(source: string): number {
  return maskRustNonCode(source, true).match(/\.await\b/g)?.length ?? 0;
}

function unqualifiedFreeFunctionCalls(source: string): string[] {
  const syntax = maskRustNonCode(source, true);
  const keywords = new Set([
    "as",
    "async",
    "else",
    "for",
    "if",
    "let",
    "loop",
    "match",
    "move",
    "return",
    "unsafe",
    "while",
  ]);
  return [...syntax.matchAll(
    /\b(r#)?([a-z_][A-Za-z0-9_]*)(?:\s*::\s*<[^(){};]*>)?\s*\(/g,
  )].flatMap((match) => {
    if (match.index === undefined) {
      throw new Error("free-function call has no offset");
    }
    const previous = syntax.slice(0, match.index).match(/(\S)\s*$/)?.[1];
    const name = match[2];
    return previous === "." || previous === ":" || keywords.has(name)
      ? []
      : [name];
  });
}

function qualifiedCallablePaths(source: string): string[] {
  const syntax = maskRustNonCode(source, true);
  const constructorNames = new Set(["new", "with_capacity"]);
  return [...syntax.matchAll(
    /\b((?:(?:r#)?[A-Za-z_][A-Za-z0-9_]*\s*::\s*)+(?:r#)?[a-z_][A-Za-z0-9_]*)\b/g,
  )].flatMap((match) => {
    if (match.index === undefined) {
      throw new Error("qualified callable path has no offset");
    }
    const after = syntax.slice(match.index + match[0].length)
      .match(/^\s*(.)/)?.[1];
    const selected = match[1]
      .replace(/\s*::\s*/g, "::")
      .replaceAll("r#", "");
    const terminal = selected.split("::").at(-1)!;
    return after === "!" || constructorNames.has(terminal)
      ? []
      : [selected];
  });
}

function methodCallInventory(source: string): string[] {
  const syntax = maskRustNonCode(source, true);
  return [...syntax.matchAll(
    /\.\s*([a-z_][A-Za-z0-9_]*)(?:\s*::\s*<[^(){};]*>)?\s*\(/g,
  )].map((match) => {
    if (match.index === undefined) throw new Error("method call has no offset");
    const open = match.index + match[0].lastIndexOf("(");
    const close = closingDelimiter(syntax, open, "(", ")");
    return `${match[1]}(${normalized(source.slice(open + 1, close))})`;
  });
}

function macroInvocationInventory(source: string): string[] {
  const syntax = maskRustNonCode(source, true);
  return [...syntax.matchAll(
    /\b((?:[A-Za-z_][A-Za-z0-9_]*\s*::\s*)*[A-Za-z_][A-Za-z0-9_]*)\s*!\s*[\(\[\{]/g,
  )].map((match) => normalized(match[1]).replace(/\s+/g, ""));
}

function detachedCoordinatorBypasses(source: string): string[] {
  const syntax = maskRustNonCode(source, true);
  const findings = new Set<string>();
  if (/\b(?:spawn|spawn_blocking|spawn_local|block_in_place)\s*\(/.test(syntax)) {
    findings.add("spawn");
  }
  if (/\bblock_on\s*\(/.test(syntax)) findings.add("block_on");
  if (/\bdetached\s*\(/.test(syntax)) findings.add("detached");
  if (/\b(?:join|try_join|select)\s*!/.test(syntax)) {
    findings.add("join/select");
  }
  if (
    /\bpool\s*\.\s*(?:clone|to_owned)\s*\(/.test(syntax)
    || /\b(?:Clone\s*::\s*clone|ToOwned\s*::\s*to_owned)\s*\(\s*&?\s*pool\b/
      .test(syntax)
  ) {
    findings.add("pool.clone");
  }
  if (
    /\basync\s+(?:move\s*)?\{/.test(syntax)
    || /\b(?:std\s*::\s*)?mem\s*::\s*forget\s*\(/.test(syntax)
    || /\.\s*detach\s*\(/.test(syntax)
  ) {
    findings.add("detached-future");
  }
  return [...findings].sort();
}

function coordinatorTopology(
  source: string,
  childNames: readonly string[],
): string[] {
  const syntax = maskRustNonCode(source, true);
  const tokens: Array<{ offset: number; value: string }> = [];
  for (const match of syntax.matchAll(
    /\bpool\s*\.\s*begin\s*\(\s*\)\s*\.await\b/g,
  )) {
    if (match.index === undefined) throw new Error("begin has no offset");
    tokens.push({ offset: match.index, value: "begin" });
  }
  for (const match of syntax.matchAll(
    /\btransaction\s*\.\s*commit\s*\(\s*\)\s*\.await\b/g,
  )) {
    if (match.index === undefined) throw new Error("commit has no offset");
    tokens.push({ offset: match.index, value: "commit" });
  }
  for (const name of childNames) {
    for (const match of syntax.matchAll(
      new RegExp(`\\b${name}\\s*\\(`, "g"),
    )) {
      if (match.index === undefined) {
        throw new Error(`${name} call has no offset`);
      }
      const open = match.index + match[0].lastIndexOf("(");
      const close = closingDelimiter(syntax, open, "(", ")");
      if (/^\s*\.await\b/.test(syntax.slice(close + 1))) {
        tokens.push({
          offset: match.index,
          value: `${name}(${normalized(source.slice(open + 1, close))})`,
        });
      }
    }
  }
  for (const match of syntax.matchAll(
    /\.(fetch_all|fetch_one|fetch_optional|execute)\s*\(/g,
  )) {
    if (match.index === undefined) {
      throw new Error("SQL terminal has no offset");
    }
    const open = match.index + match[0].lastIndexOf("(");
    const close = closingDelimiter(syntax, open, "(", ")");
    tokens.push({
      offset: match.index,
      value: `${match[1]}(${normalized(source.slice(open + 1, close))})`,
    });
  }
  return tokens
    .sort((left, right) => left.offset - right.offset)
    .map(({ value }) => value);
}

function selectedAppSource(relative: string): string {
  const selected = path.join(appAnalysisRoot, relative);
  if (!existsSync(selected)) throw new Error(`missing app analysis source ${relative}`);
  return normalizeNewlines(readFileSync(selected, "utf8"));
}

describe("analysis crate boundary", () => {
  it("requires the extractum-analysis manifest and mechanical move", () => {
    expect(
      extracted,
      "extractum-analysis Cargo.toml is intentionally absent before the mechanical move",
    ).toBe(true);
    assertFrozenTopology();
  });

  it("declares one app edge and the exact locked dependency surface", () => {
    const rootCargo = read("src-tauri/Cargo.toml");
    const rootDependencies = tomlSection(rootCargo, "dependencies");
    const lock = read("src-tauri/Cargo.lock");
    const dependencyProbe = `
      [workspace.dependencies]
      inherited-analysis = { package = "extractum-analysis", path = "crates/extractum-analysis" }
      [dependencies]
      direct-analysis = { package = "extractum-analysis", path = "crates/extractum-analysis" }
      [dev-dependencies]
      dev-analysis = { package = "extractum-analysis", path = "../extractum-analysis" }
      [build-dependencies]
      build-analysis = { package = "extractum-analysis", path = "../extractum-analysis" }
      [target.'cfg(windows)'.dev-dependencies]
      inherited-analysis.workspace = true
    `;
    expect(
      analysisDependencies(
        dependencyProbe,
        new Set(["inherited-analysis"]),
      ).map(({ section, key }) => `${section}:${key}`),
      "resolved dependency parser covers aliases, workspace inheritance, dev/build, and target tables",
    ).toEqual([
      "workspace.dependencies:inherited-analysis",
      "dependencies:direct-analysis",
      "dev-dependencies:dev-analysis",
      "build-dependencies:build-analysis",
      "target.'cfg(windows)'.dev-dependencies:inherited-analysis",
    ]);
    expect(
      rootCargo.match(/\bextractum-analysis\b/g) ?? [],
      "root manifest analysis references (member plus one direct app edge)",
    ).toHaveLength(extracted ? 3 : 0);
    const parsedRootDependencies = tomlDependencies(rootCargo);
    const workspaceAnalysisKeys = new Set(
      parsedRootDependencies
        .filter((dependency) =>
          dependency.section === "workspace.dependencies"
          && (
            dependency.packageName === "extractum-analysis"
            || dependency.path?.replaceAll("\\", "/").endsWith(
              "/extractum-analysis",
            )
          ))
        .map(({ key }) => key),
    );
    const rootAnalysisDependencies = analysisDependencies(
      rootCargo,
      workspaceAnalysisKeys,
    );
    expect(rootAnalysisDependencies).toHaveLength(extracted ? 1 : 0);
    if (extracted) {
      expect(rootAnalysisDependencies[0]).toEqual({
        section: "dependencies",
        key: "extractum-analysis",
        packageName: "extractum-analysis",
        path: "crates/extractum-analysis",
        workspace: false,
        value: '{ path = "crates/extractum-analysis" }',
      });
    }
    const expectedMembers = [
      ".",
      "crates/extractum-core",
      "crates/extractum-gemini-browser",
      "crates/extractum-llm",
      "crates/extractum-prompt-packs",
      ...(extracted ? ["crates/extractum-analysis"] : []),
      ...(telegramCrateExtracted ? ["crates/extractum-telegram"] : []),
    ];
    expect(workspaceMembers(rootCargo)).toEqual(expectedMembers);
    expect(expectedMembers).toHaveLength(telegramCrateExtracted ? 7 : 6);

    const lowerManifests = readdirSync(
      path.join(repoRoot, "src-tauri/crates"),
      { withFileTypes: true },
    )
      .filter((entry) =>
        entry.isDirectory() && entry.name !== "extractum-analysis")
      .map((entry) => ({
        name: entry.name,
        source: read(`src-tauri/crates/${entry.name}/Cargo.toml`),
      }));
    for (const manifest of lowerManifests) {
      expect(
        analysisDependencies(manifest.source, workspaceAnalysisKeys),
        `${manifest.name} resolved analysis dependency edges`,
      ).toEqual([]);
      expect(
        manifest.source,
        `${manifest.name} must have no direct, aliased, target, build, dev, or workspace analysis reference`,
      ).not.toMatch(/\bextractum-analysis\b/);
    }

    if (!extracted) {
      expect(rootDependencies).not.toMatch(/^extractum-analysis\s*=/m);
      expect(rootDependencies).toMatch(/^zstd\s*=\s*\{\s*workspace\s*=\s*true\s*\}$/m);
      expect(optionalLockPackage(lock, "extractum-analysis")).toBe("");
      expect(lockDependencies(lockPackage(lock, "extractum"))).not.toContain(
        "extractum-analysis",
      );
      return;
    }

    const frozenManifest = textBlockAfter("The crate manifest is exact:", "toml");
    expect(read("src-tauri/crates/extractum-analysis/Cargo.toml").trim())
      .toBe(frozenManifest.trim());
    expect(
      occurrenceCount(
        rootDependencies,
        /^extractum-analysis\s*=\s*\{\s*path\s*=\s*"crates\/extractum-analysis"\s*\}$/m,
      ),
    ).toBe(1);
    expect(rootDependencies).not.toMatch(/^zstd(?:\.workspace)?\s*=/m);

    const analysisLock = lockPackage(lock, "extractum-analysis");
    for (const foreignDependency of [
      "extractum-telegram",
      "grammers-client",
      "grammers-mtsender",
      "grammers-session",
      "grammers-tl-types",
    ]) {
      expect(
        read("src-tauri/crates/extractum-analysis/Cargo.toml"),
      ).not.toContain(foreignDependency);
      expect(lockDependencies(analysisLock)).not.toContain(foreignDependency);
    }
    expect(lockDependencies(analysisLock)).toEqual([
      "extractum-core",
      "extractum-llm",
      "serde",
      "serde_json",
      "sqlx",
      "tokio",
      "tokio-util",
    ]);
    expect(analysisLock).not.toMatch(/^(?:source|checksum)\s*=/m);
    const appDependencies = lockDependencies(lockPackage(lock, "extractum"));
    expect(appDependencies.filter((name) => name === "extractum-analysis"))
      .toEqual(["extractum-analysis"]);
  });

  it("keeps a curated crate API and exhaustive visibility allowlist", () => {
    const expectedExports = [...publicApi.types, ...publicApi.functions].sort();
    const portable = portableFiles();
    const publicDefinitions = topLevelPublicDefinitions(portable);
    expect(
      topLevelPublicDefinitions([{
        owner: "crate",
        relative: "visibility_probe.rs",
        source: `
          #[macro_export]
          macro_rules! exported_legacy { () => {}; }
          pub struct PublicStruct;
          pub const PUBLIC_CONST: usize = 1;
          pub static PUBLIC_STATIC: usize = 1;
          pub unsafe extern "C" fn public_ffi() {}
          pub extern crate extractum_core as public_core;
          pub macro public_macro() {}
          pub mod public_module {}
          mod private_module {
            #[macro_export]
            macro_rules! exported_nested { () => {}; }
            pub struct NestedIsNotTopLevel;
          }
        `,
      }]),
      "visibility parser rejects every extra top-level public definition form",
    ).toEqual([
      "PUBLIC_CONST",
      "PUBLIC_STATIC",
      "PublicStruct",
      "exported_legacy",
      "exported_nested",
      "public_core",
      "public_ffi",
      "public_macro",
      "public_module",
    ]);
    const temporaryFunctionNames: Record<string, string> = {
      list_analysis_prompt_templates: "list_analysis_prompt_templates_in_pool",
      create_analysis_prompt_template: "create_analysis_prompt_template_in_pool",
      update_analysis_prompt_template: "update_analysis_prompt_template_in_pool",
      delete_analysis_prompt_template: "delete_analysis_prompt_template_in_pool",
      create_analysis_source_group: "create_analysis_source_group_in_pool",
      update_analysis_source_group: "update_analysis_source_group_in_pool",
      delete_analysis_source_group: "delete_analysis_source_group_in_pool",
      list_analysis_run_messages: "list_run_snapshot_messages_page",
      get_analysis_run_trace: "get_analysis_run_trace_in_pool",
      resolve_analysis_trace_refs: "resolve_analysis_trace_refs_in_pool",
      list_analysis_chat_messages: "list_analysis_chat_messages_in_pool",
      clear_analysis_chat_messages: "clear_analysis_chat_messages_in_pool",
      request_analysis_run_cancel: "request_analysis_run_cancel_in_pool",
    };
    for (const typeName of publicApi.types) {
      expect(
        publicDefinitions.includes(typeName),
        `prepared public type ${typeName}`,
      ).toBe(true);
    }
    for (const functionName of publicApi.functions) {
      const prepared = temporaryFunctionNames[functionName] ?? functionName;
      expect(
        publicDefinitions.includes(prepared),
        `prepared public function ${functionName} (${prepared})`,
      ).toBe(true);
    }
    exactInventory(
      publicDefinitions,
      [
        ...publicApi.types,
        ...publicApi.functions.map((name) =>
          temporaryFunctionNames[name] ?? name),
      ],
      "exhaustive prepared public definition allowlist",
    );

    const forbiddenPublic = [
      "AnalysisRunRow",
      "AnalysisSourceGroupRow",
      "StoredRunSnapshotRow",
      "ChunkSummary",
      "ReportPipelineContext",
      "ReportRunInput",
      "ChatRequestParams",
      "AnalysisRunInsert",
      "DuplicateRunLookup",
      "ListRunSnapshotMessagesRequest",
    ];
    for (const forbidden of forbiddenPublic) {
      expect(publicDefinitions, `forbidden public implementation type ${forbidden}`)
        .not.toContain(forbidden);
    }

    const portableSource = portable.map(({ source }) => source).join("\n");
    const exactPrivateLayouts: Record<string, Array<[string, string]>> = {
      ResolvedAnalysisScope: [
        ["scope_kind", "AnalysisScopeKind"],
        ["source_id", "Option<i64>"],
        ["source_group_id", "Option<i64>"],
        ["project_id", "Option<i64>"],
        ["source_kind", "AnalysisSourceKind"],
        ["source_ids", "Vec<i64>"],
        ["scope_label_snapshot", "String"],
      ],
      AnalysisCorpusRequest: [
        ["source_kind", "AnalysisSourceKind"],
        ["source_ids", "Vec<i64>"],
        ["period_from", "i64"],
        ["period_to", "i64"],
        ["youtube_corpus_mode", "YoutubeCorpusMode"],
        ["include_migrated_history", "bool"],
      ],
      AnalysisCorpusMessage: [
        ["item_id", "i64"],
        ["source_id", "i64"],
        ["external_id", "String"],
        ["published_at", "i64"],
        ["author", "Option<String>"],
        ["content", "String"],
        ["r#ref", "String"],
        ["item_kind", "Option<String>"],
        ["source_type", "Option<String>"],
        ["source_subtype", "Option<String>"],
        ["metadata_zstd", "Option<Vec<u8>>"],
      ],
      AnalysisRunPreflightLimits: [
        ["max_messages_per_run", "usize"],
        ["max_chunks_per_run", "usize"],
        ["max_estimated_input_chars_per_run", "usize"],
        ["max_background_requests_per_run", "usize"],
      ],
      AnalysisRunPreflight: [
        ["source_ids", "Vec<i64>"],
        ["message_count", "usize"],
        ["estimated_input_chars", "usize"],
        ["estimated_chunks", "usize"],
        ["limits", "AnalysisRunPreflightLimits"],
      ],
      AnalysisForeignLabelMatch: [
        ["term", "String"],
        ["source_ids", "Vec<i64>"],
        ["project_ids", "Vec<i64>"],
      ],
      AnalysisSourceLabel: [
        ["source_id", "i64"],
        ["title", "Option<String>"],
      ],
      AnalysisProjectLabel: [
        ["project_id", "i64"],
        ["name", "Option<String>"],
      ],
      AnalysisForeignLabels: [
        ["sources", "Vec<AnalysisSourceLabel>"],
        ["projects", "Vec<AnalysisProjectLabel>"],
      ],
      AskAnalysisRunQuestionRequest: [
        ["run_id", "i64"],
        ["question", "String"],
        ["model_override", "Option<String>"],
        ["profile_id", "Option<String>"],
      ],
      AnalysisSourceGroupInput: [
        ["name", "String"],
        ["source_kind", "AnalysisSourceKind"],
        ["source_ids", "Vec<i64>"],
      ],
      AnalysisSourceGroupRecord: [
        ["id", "i64"],
        ["name", "String"],
        ["source_kind", "AnalysisSourceKind"],
        ["member_source_ids", "Vec<i64>"],
        ["created_at", "i64"],
        ["updated_at", "i64"],
      ],
      ProjectAnalysisRunAggregate: [
        ["project_id", "i64"],
        ["latest_run_status", "Option<String>"],
        ["last_run_at", "Option<i64>"],
        ["has_active_run", "bool"],
      ],
      AnalysisRunDiagnosticCount: [
        ["provider", "String"],
        ["run_type", "String"],
        ["scope_type", "String"],
        ["status", "String"],
        ["snapshot_state", "String"],
        ["error_kind", "String"],
        ["count", "i64"],
      ],
      StartAnalysisReportRequest: [
        ["source_id", "Option<i64>"],
        ["source_group_id", "Option<i64>"],
        ["project_id", "Option<i64>"],
        ["period_from", "i64"],
        ["period_to", "i64"],
        ["output_language", "String"],
        ["prompt_template_id", "i64"],
        ["model_override", "Option<String>"],
        ["profile_id", "Option<String>"],
        ["youtube_corpus_mode", "Option<String>"],
        ["include_migrated_history", "bool"],
      ],
      AnalysisRunListFilters: [
        ["source_id", "Option<i64>"],
        ["source_group_id", "Option<i64>"],
        ["project_id", "Option<i64>"],
        ["limit", "i64"],
        ["query", "Option<String>"],
        ["status", "Option<String>"],
        ["provider", "Option<String>"],
        ["model", "Option<String>"],
        ["template", "Option<String>"],
        ["date_from", "Option<String>"],
        ["date_to", "Option<String>"],
        ["foreign_label_search_terms", "Vec<String>"],
      ],
    };
    for (const [typeName, expected] of Object.entries(
      exactPrivateLayouts,
    )) {
      expect(
        structFields(portableSource, typeName)
          .map(({ visibility, name, type }) => [visibility, name, type]),
        `${typeName} exact private field layout`,
      ).toEqual(expected.map(([name, type]) => ["", name, type]));
    }
    const exactPublicMethods: Record<string, string[]> = {
      ResolvedAnalysisScope: [
        "pub fn for_source(source_id: i64, source_kind: AnalysisSourceKind, source_ids: Vec<i64>, scope_label_snapshot: String) -> AppResult<Self>",
        "pub fn for_source_group(source_group_id: i64, source_kind: AnalysisSourceKind, source_ids: Vec<i64>, scope_label_snapshot: String) -> AppResult<Self>",
        "pub fn for_project(project_id: i64, source_kind: AnalysisSourceKind, source_ids: Vec<i64>, scope_label_snapshot: String) -> AppResult<Self>",
        "pub fn scope_kind(&self) -> AnalysisScopeKind",
        "pub fn source_id(&self) -> Option<i64>",
        "pub fn source_group_id(&self) -> Option<i64>",
        "pub fn project_id(&self) -> Option<i64>",
        "pub fn source_kind(&self) -> AnalysisSourceKind",
        "pub fn source_ids(&self) -> &[i64]",
        "pub fn scope_label_snapshot(&self) -> &str",
      ],
      AnalysisRunListFilters: [
        "pub fn for_analysis(source_id: Option<i64>, source_group_id: Option<i64>, limit: i64, query: Option<String>, status: Option<String>, provider: Option<String>, model: Option<String>, template: Option<String>, date_from: Option<String>, date_to: Option<String>) -> AppResult<Self>",
        "pub fn for_project(project_id: i64, limit: i64) -> Self",
        "pub fn foreign_label_search_terms(&self) -> &[String]",
      ],
      YoutubeCorpusMode: [
        "pub fn from_wire(value: Option<&str>) -> Result<Self, String>",
        "pub fn as_wire(self) -> &'static str",
        "pub fn includes_description(self) -> bool",
        "pub fn includes_comments(self) -> bool",
      ],
      AnalysisCorpusRequest: [
        "pub fn new(source_kind: AnalysisSourceKind, source_ids: Vec<i64>, period_from: i64, period_to: i64, youtube_corpus_mode: YoutubeCorpusMode, include_migrated_history: bool) -> AppResult<Self>",
        "pub fn source_kind(&self) -> AnalysisSourceKind",
        "pub fn source_ids(&self) -> &[i64]",
        "pub fn period_from(&self) -> i64",
        "pub fn period_to(&self) -> i64",
        "pub fn youtube_corpus_mode(&self) -> YoutubeCorpusMode",
        "pub fn include_migrated_history(&self) -> bool",
      ],
      AnalysisCorpusMessage: [
        "pub fn new(item_id: i64, source_id: i64, external_id: String, published_at: i64, author: Option<String>, content: String, r#ref: String, item_kind: Option<String>, source_type: Option<String>, source_subtype: Option<String>, metadata_zstd: Option<Vec<u8>>) -> Self",
        "pub fn item_id(&self) -> i64",
        "pub fn source_id(&self) -> i64",
        "pub fn external_id(&self) -> &str",
        "pub fn published_at(&self) -> i64",
        "pub fn author(&self) -> Option<&str>",
        "pub fn content(&self) -> &str",
        "pub fn reference(&self) -> &str",
        "pub fn item_kind(&self) -> Option<&str>",
        "pub fn source_type(&self) -> Option<&str>",
        "pub fn source_subtype(&self) -> Option<&str>",
        "pub fn metadata_zstd(&self) -> Option<&[u8]>",
      ],
      AnalysisRunPreflightLimits: [
        "pub fn max_messages_per_run(&self) -> usize",
        "pub fn max_chunks_per_run(&self) -> usize",
        "pub fn max_estimated_input_chars_per_run(&self) -> usize",
        "pub fn max_background_requests_per_run(&self) -> usize",
      ],
      AnalysisRunPreflight: [
        "pub fn source_ids(&self) -> &[i64]",
        "pub fn message_count(&self) -> usize",
        "pub fn estimated_input_chars(&self) -> usize",
        "pub fn estimated_chunks(&self) -> usize",
        "pub fn limits(&self) -> &AnalysisRunPreflightLimits",
      ],
      AnalysisForeignLabelMatch: [
        "pub fn new(term: String, source_ids: Vec<i64>, project_ids: Vec<i64>) -> AppResult<Self>",
      ],
      AnalysisForeignLabels: [
        "pub fn new(sources: Vec<AnalysisSourceLabel>, projects: Vec<AnalysisProjectLabel>) -> AppResult<Self>",
      ],
      AnalysisSourceLabel: [
        "pub fn new(source_id: i64, title: Option<String>) -> AppResult<Self>",
        "pub fn source_id(&self) -> i64",
        "pub fn title(&self) -> Option<&str>",
      ],
      AnalysisProjectLabel: [
        "pub fn new(project_id: i64, name: Option<String>) -> AppResult<Self>",
        "pub fn project_id(&self) -> i64",
        "pub fn name(&self) -> Option<&str>",
      ],
      AnalysisRunSummaryEnrichment: [
        "pub fn foreign_label_refs(&self) -> Vec<AnalysisForeignLabelRef>",
        "pub fn finish(self, labels: AnalysisForeignLabels) -> AppResult<Vec<AnalysisRunSummary>>",
      ],
      AnalysisRunDetailEnrichment: [
        "pub fn foreign_label_refs(&self) -> Vec<AnalysisForeignLabelRef>",
        "pub fn finish(self, labels: AnalysisForeignLabels) -> AppResult<Option<AnalysisRunDetail>>",
      ],
      AnalysisChatRunEnrichment: [
        "pub fn foreign_label_refs(&self) -> Vec<AnalysisForeignLabelRef>",
        "pub fn finish(self, labels: AnalysisForeignLabels) -> AppResult<Option<AnalysisChatRun>>",
      ],
      AnalysisChatRun: [
        "pub fn needs_legacy_foreign_label(&self) -> bool",
      ],
      AnalysisSourceGroupInput: [
        "pub fn new(name: String, source_kind: AnalysisSourceKind, source_ids: Vec<i64>) -> AppResult<Self>",
        "pub fn name(&self) -> &str",
        "pub fn source_kind(&self) -> AnalysisSourceKind",
        "pub fn source_ids(&self) -> &[i64]",
      ],
      AnalysisSourceGroupRecord: [
        "pub fn id(&self) -> i64",
        "pub fn name(&self) -> &str",
        "pub fn source_kind(&self) -> AnalysisSourceKind",
        "pub fn member_source_ids(&self) -> &[i64]",
        "pub fn created_at(&self) -> i64",
        "pub fn updated_at(&self) -> i64",
      ],
      ProjectAnalysisRunAggregate: [
        "pub fn project_id(&self) -> i64",
        "pub fn latest_run_status(&self) -> Option<&str>",
        "pub fn last_run_at(&self) -> Option<i64>",
        "pub fn has_active_run(&self) -> bool",
      ],
      AnalysisRunDiagnosticCount: [
        "pub fn provider(&self) -> &str",
        "pub fn run_type(&self) -> &str",
        "pub fn scope_type(&self) -> &str",
        "pub fn status(&self) -> &str",
        "pub fn snapshot_state(&self) -> &str",
        "pub fn error_kind(&self) -> &str",
        "pub fn count(&self) -> i64",
      ],
      AnalysisReportPreparationTicket: [
        "pub fn requested_profile_id(&self) -> Option<&str>",
        "pub fn model_override(&self) -> Option<&str>",
        "pub fn resolve_youtube_corpus_mode(self) -> AppResult<AnalysisReportScopeTicket>",
      ],
      AnalysisReportScopeTicket: [
        "pub fn scope_kind(&self) -> AnalysisScopeKind",
        "pub fn scope_id(&self) -> i64",
        "pub fn youtube_corpus_mode(&self) -> YoutubeCorpusMode",
      ],
      AnalysisReportExecutionTicket: [
        "pub fn run_id(&self) -> i64",
      ],
      AskAnalysisRunQuestionRequest: [
        "pub fn new(run_id: i64, question: String, model_override: Option<String>, profile_id: Option<String>) -> AppResult<Self>",
      ],
      AnalysisChatExecutionTicket: [
        "pub fn request_id(&self) -> &str",
        "pub fn profile_id(&self) -> &str",
      ],
      AnalysisChatCompletionTicket: [
        "pub fn request_id(&self) -> &str",
        "pub fn run_id(&self) -> i64",
      ],
      AnalysisState: [
        "pub fn new() -> Self",
        "pub async fn insert_active_report_run(&self, run_id: i64)",
        "pub async fn remove_active_report_run(&self, run_id: i64)",
        "pub async fn active_report_run_ids(&self) -> HashSet<i64>",
        "pub async fn request_report_run_cancel(&self, run_id: i64) -> bool",
        "pub async fn prepare_report_run_cancellation_wait(&self, run_id: i64) -> Option<AnalysisReportCancellationWait>",
      ],
      AnalysisReportCancellationWait: [
        "pub async fn cancelled(self)",
      ],
      StartAnalysisReportRequest: [
        "pub fn from_command(source_id: Option<i64>, source_group_id: Option<i64>, project_id: Option<i64>, period_from: i64, period_to: i64, output_language: String, prompt_template_id: i64, model_override: Option<String>, profile_id: Option<String>, youtube_corpus_mode: Option<String>, include_migrated_history: bool) -> AppResult<Self>",
        "pub fn for_source(source_id: i64, period_from: i64, period_to: i64, output_language: String, prompt_template_id: i64, model_override: Option<String>, profile_id: Option<String>, youtube_corpus_mode: Option<String>, include_migrated_history: bool) -> AppResult<Self>",
        "pub fn for_source_group(source_group_id: i64, period_from: i64, period_to: i64, output_language: String, prompt_template_id: i64, model_override: Option<String>, profile_id: Option<String>, youtube_corpus_mode: Option<String>, include_migrated_history: bool) -> AppResult<Self>",
        "pub fn for_project(project_id: i64, period_from: i64, period_to: i64, output_language: String, prompt_template_id: i64, model_override: Option<String>, profile_id: Option<String>, youtube_corpus_mode: Option<String>, include_migrated_history: bool) -> AppResult<Self>",
      ],
    };
    for (const [typeName, expected] of Object.entries(
      exactPublicMethods,
    )) {
      expect(
        inherentPublicMethodSignatures(portableSource, typeName)
          .map(canonicalRustSignature),
        `${typeName} exact public constructors/accessors`,
      ).toEqual(expected.map(canonicalRustSignature));
    }
    expect(
      inherentPublicMethodSignatures(
        "pub struct Probe; impl self::Probe { pub fn leaked(&self) {} }",
        "Probe",
      ).map(canonicalRustSignature),
      "qualified inherent impl paths remain inside the method allowlist",
    ).toEqual(["pub fn leaked(&self)"]);
    expect(portableSource).toMatch(
      /impl\s+Default\s+for\s+AnalysisRunPreflightLimits\s*\{/,
    );
    expect(portableSource).toMatch(
      /impl\s+std::str::FromStr\s+for\s+YoutubeCorpusMode\s*\{\s*type Err = String;/,
    );
    for (const opaque of [
      "ResolvedAnalysisScope",
      "AnalysisCorpusRequest",
      "AnalysisCorpusMessage",
      "AnalysisRunPreflightLimits",
      "AnalysisRunPreflight",
      "StartAnalysisReportRequest",
      "AnalysisRunListFilters",
      "AskAnalysisRunQuestionRequest",
      "AnalysisReportPreparationTicket",
      "AnalysisReportScopeTicket",
      "AnalysisReportExecutionTicket",
      "AnalysisChatExecutionTicket",
      "AnalysisChatCompletionTicket",
      "AnalysisReportCancellationWait",
      "AnalysisForeignLabelMatch",
      "AnalysisSourceLabel",
      "AnalysisProjectLabel",
      "AnalysisForeignLabels",
      "AnalysisRunSummaryEnrichment",
      "AnalysisRunDetailEnrichment",
      "AnalysisChatRunEnrichment",
      "AnalysisChatRun",
      "AnalysisSourceGroupInput",
      "AnalysisSourceGroupRecord",
      "ProjectAnalysisRunAggregate",
      "AnalysisRunDiagnosticCount",
    ]) {
      expect(
        structBody(portableSource, opaque),
        `${opaque} must not expose public fields`,
      ).not.toMatch(/^\s*pub(?:\([^)]*\))?\s+[A-Za-z_][A-Za-z0-9_]*\s*:/m);
    }
    for (const opaque of [
      "AnalysisRunSummaryEnrichment",
      "AnalysisRunDetailEnrichment",
      "AnalysisChatRunEnrichment",
      "AnalysisChatRun",
      "AnalysisReportPreparationTicket",
      "AnalysisReportScopeTicket",
      "AnalysisReportExecutionTicket",
      "AnalysisChatExecutionTicket",
      "AnalysisChatCompletionTicket",
      "AnalysisReportCancellationWait",
    ]) {
      expect(
        declarationAttributes(portableSource, "struct", opaque),
        `${opaque} must remain opaque and non-serializable`,
      ).not.toMatch(/\bSerialize\b/);
    }
    expect(
      declarationAttributes(
        portableSource,
        "struct",
        "AnalysisReportExecutionTicket",
      ),
      "execution ticket must not expose secrets through Debug",
    ).not.toMatch(/\bDebug\b/);

    if (!extracted) {
      expect(existsSync(analysisCrateManifest)).toBe(false);
      return;
    }

    const crateRoot = read("src-tauri/crates/extractum-analysis/src/lib.rs");
    exactInventory(
      topLevelPublicDefinitions([{
        owner: "crate",
        relative: "lib.rs",
        source: crateRoot,
      }]),
      [],
      "analysis crate root defines no public item outside exact pub-use exports",
    );
    expect(crateRoot).not.toMatch(/\bpub\s+mod\b/);
    expect(crateRoot).not.toMatch(/\bpub\s+use\b[^;]*\*/s);
    expect(crateRoot).not.toMatch(/\bpub\s+(?:const|static)\b/);
    exactInventory(
      publicRootExports(crateRoot),
      expectedExports,
      "analysis crate public root exports",
    );
    const mappings = publicRootMappings(crateRoot)
      .map(({ source, exported }) => `${source}->${exported}`);
    const nonIdentityAliases = [
      ["templates::list_analysis_prompt_templates_in_pool", "list_analysis_prompt_templates"],
      ["templates::create_analysis_prompt_template_in_pool", "create_analysis_prompt_template"],
      ["templates::update_analysis_prompt_template_in_pool", "update_analysis_prompt_template"],
      ["templates::delete_analysis_prompt_template_in_pool", "delete_analysis_prompt_template"],
      ["groups::create_analysis_source_group_in_pool", "create_analysis_source_group"],
      ["groups::update_analysis_source_group_in_pool", "update_analysis_source_group"],
      ["groups::delete_analysis_source_group_in_pool", "delete_analysis_source_group"],
      ["corpus::snapshot::list_run_snapshot_messages_page", "list_analysis_run_messages"],
      ["trace::get_analysis_run_trace_in_pool", "get_analysis_run_trace"],
      ["trace::resolve_analysis_trace_refs_in_pool", "resolve_analysis_trace_refs"],
      ["chat::list_analysis_chat_messages_in_pool", "list_analysis_chat_messages"],
      ["chat::clear_analysis_chat_messages_in_pool", "clear_analysis_chat_messages"],
      ["report::request_analysis_run_cancel_in_pool", "request_analysis_run_cancel"],
    ] as const;
    for (const [source, exported] of nonIdentityAliases) {
      expect(
        mappings.filter((mapping) => mapping === `${source}->${exported}`),
        `exact root alias ${source} as ${exported}`,
      ).toHaveLength(1);
    }
    const finalNameByPrepared = new Map(
      Object.entries(temporaryFunctionNames)
        .map(([finalName, preparedName]) => [preparedName, finalName]),
    );
    const expectedRootMappings = portable.flatMap((file) => {
      const move = allMoves.find(({ before, after }) =>
        before === file.relative || after === file.relative);
      if (!move) throw new Error(`public source is not a frozen move: ${file.relative}`);
      const after = move.after;
      const directModule = after.endsWith("/mod.rs")
        ? after.slice(0, -"/mod.rs".length)
        : after.replace(/\.rs$/, "");
      const module = directModule.startsWith("store/")
        ? "store"
        : directModule.startsWith("report/")
          ? "report"
          : directModule.replace(/\//g, "::");
      return topLevelPublicDefinitions([file]).map((preparedName) => {
        const finalName = finalNameByPrepared.get(preparedName) ?? preparedName;
        return `${module}::${preparedName}->${finalName}`;
      });
    });
    exactInventory(
      mappings,
      expectedRootMappings,
      "every public definition source-to-root-export mapping",
    );
    const syntax = maskRustNonCode(crateRoot, true);
    const modules = [
      ...syntax.matchAll(/(?:^|\n)\s*(?:#\[[^\]]+\]\s*)*mod\s+([A-Za-z_][A-Za-z0-9_]*)\s*;/g),
    ].map((match) => match[1]);
    exactInventory(
      modules,
      [
        "chat",
        "corpus",
        "domain",
        "groups",
        "models",
        "report",
        "state",
        "store",
        "templates",
        "test_schema",
        "tests",
        "trace",
      ],
      "private crate modules",
    );
    expect(crateRoot).toMatch(/#\[cfg\(test\)\]\s*mod test_schema;/);
    expect(crateRoot).toMatch(/#\[cfg\(test\)\]\s*mod tests;/);
    for (const match of syntax.matchAll(/\bpub\s+use\s+([\s\S]*?);/g)) {
      expect(match[1], "crate root must not re-export test support")
        .not.toMatch(/\b(?:test_schema|analysis_test_pool)\b/);
    }
  });

  it("moves every frozen baseline identity to its approved 95/48 owner exactly once", () => {
    const appGraph = analysisModuleGraph(appAnalysisRoot, "app");
    const crateGraph = analysisModuleGraph(analysisCrateRoot, "crate");
    expect(
      cfgRestrictions(
        ["#[cfg(test)]", "#[tokio::test]"],
        "analysis::tests::healthy",
      ),
      "ordinary test gating remains accepted",
    ).toEqual([]);
    expect(
      cfgRestrictions(
        ["#[cfg(test)]", "#[cfg(any(windows, unix))]"],
        "analysis::nested::tests::mutated",
      ),
      "nested inherited compound cfg is rejected",
    ).toEqual(["cfg(any(windows, unix))"]);
    expect(
      cfgRestrictions(
        ["#[cfg_attr(windows, ignore)]", "#[ignore]"],
        "analysis::tests::mutated",
      ),
      "cfg_attr and ignore cannot disable a frozen leaf",
    ).toEqual(["cfg_attr", "ignore"]);
    const frozenLeaves = new Set(
      appendix.map(({ current }) => current.split("::").at(-1)!),
    );
    const actualRecords = [...appGraph.tests, ...crateGraph.tests]
      .filter(({ name }) => frozenLeaves.has(name));
    const actual = actualRecords.map(({ identity }) => identity).sort();
    const expected = appendix.map((identity) =>
      extracted ? identity.final : identity.current).sort();
    exactInventory(
      actual,
      expected,
      "reachable frozen Cargo-style analysis test identities",
    );
    const expectedOwners = new Map(
      appendix.map((identity) => [
        extracted ? identity.final : identity.current,
        extracted ? identity.owner : "app",
      ]),
    );
    for (const record of actualRecords) {
      expect(record.owner, `${record.identity} final package owner`)
        .toBe(expectedOwners.get(record.identity));
      expect(
        cfgRestrictions(record.attributes, record.identity),
        `${record.identity} inherited cfg/cfg_attr/ignore restrictions`,
      ).toEqual([]);
    }

    const appUnreachable = sourceFiles(appAnalysisRoot, "app")
      .map(({ relative }) => relative)
      .filter((relative) => !appGraph.reached.has(relative))
      .sort();
    expect(appUnreachable).toEqual(
      extracted
        ? []
        : [
          "corpus/tests/harness_portable.rs",
          "corpus/tests/mod_portable.rs",
          "store/tests/mod_portable.rs",
        ],
    );
    const crateUnreachable = sourceFiles(analysisCrateRoot, "crate")
      .map(({ relative }) => relative)
      .filter((relative) => !crateGraph.reached.has(relative))
      .sort();
    expect(crateUnreachable).toEqual([]);
    expect(appendix.filter(({ owner }) => owner === "crate")).toHaveLength(95);
    expect(appendix.filter(({ owner }) => owner === "app")).toHaveLength(48);
    expect(expected).toHaveLength(143);
    expect(new Set(expected).size).toBe(143);
    if (extracted) {
      const crateExpected = new Set(
        appendix.filter(({ owner }) => owner === "crate")
          .map(({ final }) => final),
      );
      const appExpected = new Set(
        appendix.filter(({ owner }) => owner === "app")
          .map(({ final }) => final),
      );
      for (const identity of crateExpected) expect(identity).not.toMatch(/^analysis::/);
      for (const identity of appExpected) expect(identity).toMatch(/^analysis::/);
      expect([...crateExpected].filter((identity) => appExpected.has(identity)))
        .toEqual([]);
    }
  });

  it("rejects disabled renamed or copied legacy analysis tests", () => {
    expect(
      graphTestDeclarations(`
        #[test]
        fn registered_at_root() {}
        #[test]
        fn r#registered_raw() {}
        mod registered_module {
          #[test]
          fn registered_in_module() {}
        }
        fn ordinary_function() {
          #[test]
          fn not_registered_in_function() {}
          mod nested_but_still_inside_function {
            #[test]
            fn not_registered_through_nested_module() {}
          }
        }
        impl Probe {
          #[test]
          fn not_registered_in_impl() {}
        }
      `).map(({ name }) => name),
      "only module-scope test items are Cargo test registrations",
    ).toEqual([
      "registered_at_root",
      "registered_raw",
      "registered_in_module",
    ]);
    expect(rustPathAttribute('#[path = r"other.rs"]')).toBe("other.rs");
    expect(rustPathAttribute('#[path = r#"nested/other.rs"#]'))
      .toBe("nested/other.rs");
    expect(() => rustPathAttribute('#[path = b"other.rs"]'))
      .toThrow(/unsupported Rust path attribute literal/);
    const frozenLeaves = new Set(
      appendix.map(({ current }) => current.split("::").at(-1)!),
    );
    const packageSources = [
      ...regularFiles(path.join(repoRoot, "src-tauri/src"))
        .map((file) => ({
          owner: "app" as const,
          relative: path.relative(
            path.join(repoRoot, "src-tauri/src"),
            file,
          ).replaceAll("\\", "/"),
          source: normalizeNewlines(readFileSync(file, "utf8")),
        })),
      ...regularFiles(analysisCrateRoot)
        .map((file) => ({
          owner: "crate" as const,
          relative: path.relative(analysisCrateRoot, file)
            .replaceAll("\\", "/"),
          source: normalizeNewlines(readFileSync(file, "utf8")),
        })),
    ];
    const sourceRecords = packageSources.flatMap((file) =>
      graphTestDeclarations(file.source)
        .map((test) => ({ ...test, file })))
      .filter(({ name }) => frozenLeaves.has(name));
    for (const leaf of frozenLeaves) {
      const matches = sourceRecords.filter(({ name }) => name === leaf);
      expect(matches, `package-wide frozen test leaf ${leaf}`).toHaveLength(1);
      expect(matches[0].attributes, `${leaf} must remain directly enabled`)
        .not.toMatch(/#\s*\[\s*(?:ignore|cfg)\b/);
    }
    expect(sourceRecords).toHaveLength(143);
    assertFrozenTopology();
  });

  // Repository-wide fail-closed SQL analysis can exceed Vitest's 5s default.
  it("keeps production SQL in the exact six-table owner", () => {
    const cfgMatchArmProbe = productionRust(`
      match client {
        TelegramClientInner::Grammers(client) => live_before(client),
        #[cfg(test)]
        TelegramClientInner::TestIf => if condition {
          hidden_then()
        } else {
          hidden_else()
        },
        #[cfg(test)]
        TelegramClientInner::TestMethod { .. } =>
          Probe { value: hidden_value() }.hidden_method(),
        #[cfg(test)]
        TelegramClientInner::TestBlockMethod => {
          hidden_block_value()
        }.hidden_block_method(),
        #[cfg(test)]
        TelegramClientInner::TestBlock { .. } => {
          hidden_terminal_block()
        }
        TelegramClientInner::LiveAfter(client) => live_after(client),
      }
    `);
    expect(
      [
        "TelegramClientInner::TestIf",
        "TelegramClientInner::TestMethod",
        "TelegramClientInner::TestBlockMethod",
        "TelegramClientInner::TestBlock",
        "hidden_then",
        "hidden_else",
        "hidden_value",
        "hidden_method",
        "hidden_block_value",
        "hidden_block_method",
        "hidden_terminal_block",
      ].filter((marker) => cfgMatchArmProbe.includes(marker)),
      "cfg-disabled match-arm bodies must not remain in production Rust",
    ).toEqual([]);
    expect(normalized(cfgMatchArmProbe)).toContain(
      "TelegramClientInner::Grammers(client) => live_before(client),",
    );
    expect(normalized(cfgMatchArmProbe)).toContain(
      "TelegramClientInner::LiveAfter(client) => live_after(client),",
    );
    expect(() => rustBraceRegions(cfgMatchArmProbe)).not.toThrow();

    const portable = portableProductionFiles();
    const app = appProductionFiles();
    const telegramFoundationIsStaged = existsSync(
      path.join(repoRoot, "src-tauri/src/telegram_impl/lib.rs"),
    );
    const telegramPeerAvatarBoundaryPaths = [
      "telegram_impl/error.rs",
      "telegram_impl/live/avatar.rs",
      "telegram_impl/live/mod.rs",
      "telegram_impl/live/peer.rs",
    ];
    const telegramLiveConsumerPaths = [
      "telegram_impl/live/messages.rs",
      "telegram_impl/live/topics.rs",
    ];
    const telegramTakeoutBoundaryPaths = [
      "telegram_impl/takeout/mod.rs",
      "telegram_impl/takeout/types.rs",
      "telegram_impl/takeout/transport.rs",
      "telegram_impl/takeout/export_dc.rs",
      "telegram_impl/takeout/operations.rs",
      "telegram_impl/takeout/pagination.rs",
      "telegram_impl/takeout/raw_parse.rs",
      "telegram_impl/takeout/forum_topics.rs",
    ];
    const appPaths = new Set(app.map(({ relative }) => relative));
    const telegramPeerAvatarBoundaryPathCount =
      telegramPeerAvatarBoundaryPaths.filter((relativePath) =>
        appPaths.has(relativePath)
      ).length;
    expect(
      [0, telegramPeerAvatarBoundaryPaths.length],
      "Telegram CP4 peer/avatar graph stage must be absent or complete",
    ).toContain(telegramPeerAvatarBoundaryPathCount);
    const telegramPeerAvatarBoundaryIsStaged =
      telegramPeerAvatarBoundaryPathCount
        === telegramPeerAvatarBoundaryPaths.length;
    const telegramPeerAvatarBoundaryIsRetained =
      telegramPeerAvatarBoundaryIsStaged
      || existsSync(path.join(repoRoot, "src-tauri/crates/extractum-telegram/src/live/peer.rs"));
    const telegramLiveConsumerPathCount =
      telegramLiveConsumerPaths.filter((relativePath) =>
        appPaths.has(relativePath)
      ).length;
    expect(
      [0, telegramLiveConsumerPaths.length],
      "Telegram CP5 live-consumer graph stage must be absent or complete",
    ).toContain(telegramLiveConsumerPathCount);
    const telegramLiveConsumersAreStaged =
      telegramLiveConsumerPathCount === telegramLiveConsumerPaths.length;
    const telegramTakeoutBoundaryPathCount =
      telegramTakeoutBoundaryPaths.filter((relativePath) =>
        appPaths.has(relativePath)
      ).length;
    expect(
      [0, telegramTakeoutBoundaryPaths.length],
      "Telegram CP7 Takeout graph stage must be absent or complete",
    ).toContain(telegramTakeoutBoundaryPathCount);
    const telegramTakeoutBoundaryIsStaged =
      telegramTakeoutBoundaryPathCount === telegramTakeoutBoundaryPaths.length;
    const telegramTakeoutBoundaryIsRetained =
      telegramTakeoutBoundaryIsStaged
      || existsSync(path.join(repoRoot, "src-tauri/crates/extractum-telegram/src/takeout/mod.rs"));
    expect(
      [...ownedTables].filter((table) => !schemaTables.has(table)),
      "all six owned tables must exist in canonical migrations",
    ).toEqual([]);
    expect(app.length, "all-app production graph breadth")
      .toBe(
        telegramCrateExtracted
          ? 119
          : telegramTakeoutBoundaryIsStaged
          ? 137
          : telegramLiveConsumersAreStaged
          ? 132
          : telegramPeerAvatarBoundaryIsStaged
          ? 130
          : telegramFoundationIsStaged
          ? 126
          : 125,
      );
    for (const requiredPath of [
      "lib.rs",
      "main.rs",
      "accounts.rs",
      "analysis_documents.rs",
      "analysis/mod.rs",
      ...(telegramFoundationIsStaged ? ["telegram_impl/lib.rs"] : []),
      ...(telegramPeerAvatarBoundaryIsStaged
        ? telegramPeerAvatarBoundaryPaths
        : []),
      ...(telegramLiveConsumersAreStaged
        ? telegramLiveConsumerPaths
        : []),
      ...(telegramTakeoutBoundaryIsStaged
        ? telegramTakeoutBoundaryPaths
        : []),
      "projects/read_model.rs",
      "notebooklm_export/query.rs",
    ]) {
      expect(
        app.map(({ relative }) => relative),
        `all-app production graph misses ${requiredPath}`,
      ).toContain(requiredPath);
    }

    expect(sqlTables(
      'sqlx::query("SELECT * FROM analysis_runs")',
    )).toEqual(["analysis_runs"]);
    expect(sqlTables(
      'sqlx::raw_sql("DELETE FROM analysis_chat_messages")',
    )).toEqual(["analysis_chat_messages"]);
    expect(sqlTables(
      'sqlx::query(concat!("SELECT * FROM analysis_", "run_messages"))',
    )).toContain("analysis_run_messages");
    expect(sqlTables(
      'sqlx::query_with("SELECT * FROM analysis_runs", arguments)',
    )).toEqual(["analysis_runs"]);
    expect(sqlTables(`
      use sqlx::QueryBuilder;
      let mut query = QueryBuilder::<Sqlite>::new(concat!(
        "WITH staged AS (SELECT * FROM analysis_", "runs) ",
        "SELECT * FROM staged"
      ));
    `)).toContain("analysis_runs");
    expect(
      sqlReferences(
        "WITH analysis_runs AS (SELECT * FROM sources) "
        + "SELECT * FROM analysis_runs",
      ).map(({ table }) => table),
      "CTE aliases must not be mistaken for physical owned tables",
    ).toEqual(["sources"]);
    expect(
      sqlReferences(
        "WITH sources AS (SELECT 1) SELECT * FROM sources; "
        + "SELECT * FROM sources;",
      ),
      "CTE aliases are local to one raw-SQL statement",
    ).toEqual([{ operation: "FROM", table: "sources" }]);
    expect(
      sqlReferences("CREATE INDEX leaked_idx ON sources(id)"),
      "runtime CREATE INDEX owns its ON table",
    ).toEqual([{ operation: "CREATE INDEX ON", table: "sources" }]);
    expect(
      sqlReferences("SELECT * FROM aux.analysis_runs"),
      "an attached schema keeps an owned-looking leaf foreign",
    ).toEqual([{ operation: "FROM", table: "aux.analysis_runs" }]);
    expect(
      sqlReferences("SELECT * FROM `aux`.`analysis_runs`"),
      "quoted attached schemas remain explicit",
    ).toEqual([{ operation: "FROM", table: "aux.analysis_runs" }]);
    expect(
      sqlReferences("ANALYZE sources"),
      "runtime maintenance owns its target",
    ).toEqual([{ operation: "ANALYZE", table: "sources" }]);
    expect(
      sqlReferences("PRAGMA table_info(sources)"),
      "table-bearing PRAGMAs own their target",
    ).toEqual([{ operation: "PRAGMA TABLE_INFO", table: "sources" }]);
    expect(
      sqlReferences('PRAGMA table_info("sources""backup")'),
      "escaped quoted table identifiers are decoded before ownership checks",
    ).toEqual([{
      operation: "PRAGMA TABLE_INFO",
      table: 'sources"backup',
    }]);
    expect(
      sqlReferences("PRAGMA 'table_info'(sources)"),
      "single-quoted pragma identifiers use the shared decoder",
    ).toEqual([{ operation: "PRAGMA TABLE_INFO", table: "sources" }]);
    expect(
      sqlReferences("-- audit\nPRAGMA table_info(sources)"),
      "leading line comments cannot hide a table-bearing PRAGMA",
    ).toEqual([{ operation: "PRAGMA TABLE_INFO", table: "sources" }]);
    expect(
      sqlReferences("SELECT 1 -- FROM sources\n"),
      "line-comment contents are excluded from every table matcher",
    ).toEqual([]);
    expect(
      sqlReferences("SELECT 1 /* FROM analysis_runs */"),
      "block-comment contents are excluded from every table matcher",
    ).toEqual([]);
    expect(
      sqlReferences('PRAGMA "aux".table_info("analysis_runs")'),
      "schema-qualified table-bearing PRAGMAs retain their schema",
    ).toEqual([{
      operation: "PRAGMA TABLE_INFO",
      table: "aux.analysis_runs",
    }]);
    expect(
      sqlReferences(
        'PRAGMA "au""x"."table_info"("analysis""runs")',
      ),
      "schema pragma and argument identifiers share SQLite escape decoding",
    ).toEqual([{
      operation: "PRAGMA TABLE_INFO",
      table: 'au"x.analysis"runs',
    }]);
    expect(
      sqlReferences(
        "PRAGMA 'au''x'.'table_info'('analysis''runs')",
      ),
      "single-quoted schema pragma and argument escapes share decoding",
    ).toEqual([{
      operation: "PRAGMA TABLE_INFO",
      table: "au'x.analysis'runs",
    }]);
    expect(
      sqlReferences("PRAGMA foreign_key_check(analysis_runs)"),
      "optional table-scoped foreign-key checks own their target",
    ).toEqual([{
      operation: "PRAGMA FOREIGN_KEY_CHECK",
      table: "analysis_runs",
    }]);
    expect(
      sqlReferences(
        "/* audit */\nPRAGMA custom_table_probe(analysis_runs)",
      ),
      "leading block comments cannot hide an unknown table-bearing PRAGMA",
    ).toEqual([{
      operation: "UNRESOLVED SQL",
      table: "__unresolved_sql_statement__",
    }]);
    expect(
      sqlReferences("/* unterminated audit comment"),
      "unterminated block comments fail closed",
    ).toEqual([{
      operation: "UNRESOLVED SQL",
      table: "__unresolved_sql_statement__",
    }]);
    expect(
      sqlReferences('PRAGMA table_info("sources)'),
      "unterminated quoted identifiers fail closed",
    ).toEqual([{
      operation: "UNRESOLVED SQL",
      table: "__unresolved_sql_statement__",
    }]);
    expect(
      sqlReferences("PRAGMA custom_table_probe(sources)"),
      "unknown identifier-bearing PRAGMAs stay unresolved",
    ).toEqual([{
      operation: "UNRESOLVED SQL",
      table: "__unresolved_sql_statement__",
    }]);
    expect(
      sqlReferences("PRAGMA custom_table_probe = sources"),
      "unknown identifier assignment PRAGMAs stay unresolved",
    ).toEqual([{
      operation: "UNRESOLVED SQL",
      table: "__unresolved_sql_statement__",
    }]);
    expect(
      sqlReferences('PRAGMA custom_table_probe = "analysis_runs"'),
      "unknown quoted-identifier assignment PRAGMAs stay unresolved",
    ).toEqual([{
      operation: "UNRESOLVED SQL",
      table: "__unresolved_sql_statement__",
    }]);
    expect(
      sqlReferences("PRAGMA foreign_keys = OFF"),
      "approved scalar foreign-key toggles are not table references",
    ).toEqual([]);
    expect(
      sqlReferences("PRAGMA busy_timeout = 5000"),
      "approved scalar numeric PRAGMAs are not table references",
    ).toEqual([]);
    expect(
      sqlReferences("PRAGMA journal_mode = WAL"),
      "approved scalar identifier PRAGMAs are not table references",
    ).toEqual([]);
    expect(
      sqlReferences("PRAGMA journal_mode(WAL)"),
      "approved scalar call-form PRAGMAs are not table references",
    ).toEqual([]);
    expect(
      sqlReferences('PRAGMA journal_mode("WAL"x)'),
      "malformed quoted scalar arguments fail closed before dispatch",
    ).toEqual([{
      operation: "UNRESOLVED SQL",
      table: "__unresolved_sql_statement__",
    }]);
    expect(
      sqlReferences('PRAGMA "table_info"x(sources)'),
      "malformed quoted pragma names fail closed before dispatch",
    ).toEqual([{
      operation: "UNRESOLVED SQL",
      table: "__unresolved_sql_statement__",
    }]);
    expect(
      sqlReferences(
        "CREATE TRIGGER rogue AFTER INSERT ON sources "
        + "BEGIN SELECT 1; END",
      ),
      "runtime trigger DDL owns its ON table",
    ).toContainEqual({
      operation: "CREATE TRIGGER ON",
      table: "sources",
    });
    expect(
      sqlReferences("DROP VIEW rogue"),
      "unmapped table-bearing DDL fails closed",
    ).toEqual([{
      operation: "UNRESOLVED SQL",
      table: "__unresolved_sql_statement__",
    }]);
    expect(
      sqlConsumerAnalysis(
        'sqlx::query(format!("SELECT * FROM {}", "analysis_runs"));',
      ).unresolved,
      "dynamic SQL consumer must fail closed",
    ).toEqual([
      'query:format!("SELECT * FROM {}", "analysis_runs")',
    ]);
    expect(
      sqlConsumerAnalysis(`
        fn probe(dynamic_sql: &str) {
          let mut sql = "SELECT * FROM sources";
          sql = dynamic_sql;
          sqlx::query(sql);
        }
      `).unresolved,
      "a dynamic reassignment must replace an earlier static SQL binding",
    ).toEqual(["query:sql"]);
    expect(
      sqlConsumerAnalysis(`
        fn probe(dynamic_sql: &str) {
          let mut sql = dynamic_sql;
          sql = "SELECT * FROM analysis_runs";
          sqlx::query(sql);
        }
      `),
      "a later static reassignment is the use-site value",
    ).toMatchObject({
      sql: ["SELECT * FROM analysis_runs"],
      unresolved: [],
    });
    expect(
      sqlConsumerAnalysis(`
        const sql: &str = "SELECT * FROM sources";
        fn probe(sql: &str) { sqlx::query(sql); }
      `).unresolved,
      "function parameters shadow outer static bindings",
    ).toEqual(["query:sql"]);
    expect(
      sqlConsumerAnalysis(`
        fn probe() {
          let sql = "SELECT * FROM sources";
          let inspect = |sql: &str| sqlx::query(sql);
        }
      `).unresolved,
      "closure parameters shadow outer static bindings",
    ).toEqual(["query:sql"]);
    expect(
      sqlConsumerAnalysis(`
        fn probe(dynamic_sql: &str) {
          let sql = "SELECT * FROM sources";
          {
            let sql = dynamic_sql;
            sqlx::query(sql);
          }
          sqlx::query(sql);
        }
      `),
      "an inner dynamic shadow must not overwrite its static outer sibling",
    ).toMatchObject({
      sql: ["SELECT * FROM sources"],
      unresolved: ["query:sql"],
    });
    expect(
      sqlConsumerAnalysis(`
        fn probe(condition: bool, dynamic_sql: &str) {
          let mut sql = "SELECT * FROM sources";
          if condition {
            sql = dynamic_sql;
          } else {
            sql = "SELECT * FROM sources";
          }
          sqlx::query(sql);
        }
      `).unresolved,
      "branch-local assignments merge conservatively at the use site",
    ).toEqual(["query:sql"]);
    expect(
      sqlConsumerAnalysis(`
        fn probe(dynamic_sql: &str) {
          let mut sql = dynamic_sql;
          let set_safe = || { sql = "SELECT * FROM sources"; };
          drop(set_safe);
          sqlx::query(sql);
        }
      `).unresolved,
      "a deferred closure assignment cannot become a straight-line value",
    ).toEqual(["query:sql"]);
    expect(
      sqlConsumerAnalysis(`
        fn probe(dynamic_sql: &str) {
          let mut sql = dynamic_sql;
          let set_safe = async { sql = "SELECT * FROM sources"; };
          drop(set_safe);
          sqlx::query(sql);
        }
      `).unresolved,
      "an unpolled async-block assignment cannot become a straight-line value",
    ).toEqual(["query:sql"]);
    for (const [label, source] of [
      [
        "qualified public SQL function alias",
        "mod api { pub(crate) use sqlx::query as q; } fn probe() { api::q(dynamic_sql); }",
      ],
      [
        "chained qualified public SQL function alias",
        "mod api { pub(crate) use sqlx::query as q; } mod facade { pub(crate) use crate::api::q as query; } fn probe() { facade::query(dynamic_sql); }",
      ],
      [
        "qualified parent SQLx root alias",
        "mod parent { use sqlx as db; mod child { fn probe() { super::db::query(dynamic_sql); } } }",
      ],
      [
        "qualified private parent SQL function alias",
        "mod parent { use sqlx::query as q; mod child { fn probe() { super::q(dynamic_sql); } } }",
      ],
      [
        "chained qualified SQLx root re-export",
        "mod api { pub(crate) use sqlx as db; } mod facade { pub(crate) use crate::api::db as db2; } fn probe() { facade::db2::query(dynamic_sql); }",
      ],
    ] as const) {
      expect(
        sqlConsumerAnalysis(source).unresolved,
        label,
      ).toEqual(["query:dynamic_sql"]);
    }
    expect(
      sqlConsumerAnalysis(`
        use sqlx::QueryBuilder;
        let mut query = QueryBuilder::<Sqlite>::new("SELECT * FROM sources");
        query.push(dynamic_table);
      `).unresolved,
      "dynamic QueryBuilder SQL fragment must fail closed",
    ).toEqual(["query_builder_push:dynamic_table"]);
    for (const [label, mutation] of [
      [
        "imported query alias",
        "use sqlx::query as q; fn probe() { q(dynamic_sql); }",
      ],
      [
        "copied query function item",
        "fn probe() { let q = sqlx::query::<sqlx::Sqlite>; let _ = q(dynamic_sql); }",
      ],
      [
        "sqlx root alias",
        "use sqlx as db; fn probe() { db::query(dynamic_sql); }",
      ],
      [
        "grouped sqlx root alias",
        "use sqlx::{self as db}; fn probe() { db::query(dynamic_sql); }",
      ],
      [
        "absolute sqlx root alias",
        "use ::sqlx as db; fn probe() { db::query(dynamic_sql); }",
      ],
      [
        "extern crate sqlx root alias",
        "extern crate sqlx as db; fn probe() { db::query(dynamic_sql); }",
      ],
      [
        "glob-imported query",
        "use sqlx::*; fn probe() { query(dynamic_sql); }",
      ],
      [
        "grouped glob-imported query",
        "use sqlx::{*}; fn probe() { query(dynamic_sql); }",
      ],
      [
        "root-alias glob-imported query",
        "use sqlx as db; use db::*; fn probe() { query(dynamic_sql); }",
      ],
      [
        "publicly re-exported query alias",
        "pub use sqlx::query as public_query; fn probe() { public_query(dynamic_sql); }",
      ],
      [
        "publicly re-exported root alias",
        "pub use sqlx as db; fn probe() { db::query(dynamic_sql); }",
      ],
      [
        "qualified query_with",
        "fn probe(arguments: Args) { sqlx::query_with(dynamic_sql, arguments); }",
      ],
      [
        "qualified query_as_with",
        "fn probe(arguments: Args) { sqlx::query_as_with::<_, Row, _>(dynamic_sql, arguments); }",
      ],
      [
        "qualified query_scalar_with",
        "fn probe(arguments: Args) { sqlx::query_scalar_with::<_, i64, _>(dynamic_sql, arguments); }",
      ],
      [
        "imported query_with alias",
        "use sqlx::query_with as qw; fn probe(arguments: Args) { qw(dynamic_sql, arguments); }",
      ],
      [
        "imported query_as_with alias",
        "use sqlx::query_as_with as qaw; fn probe(arguments: Args) { qaw(dynamic_sql, arguments); }",
      ],
      [
        "imported query_scalar_with alias",
        "use sqlx::query_scalar_with as qsw; fn probe(arguments: Args) { qsw(dynamic_sql, arguments); }",
      ],
      [
        "root-alias query_with",
        "use sqlx as db; fn probe(arguments: Args) { db::query_with(dynamic_sql, arguments); }",
      ],
      [
        "QueryBuilder type alias",
        "use sqlx::QueryBuilder as Qb; fn probe() { let mut q = Qb::<Sqlite>::new(dynamic_sql); }",
      ],
      [
        "QueryBuilder Rust type alias",
        "type Qb<'a> = sqlx::QueryBuilder<'a, Sqlite>; fn probe() { let mut q = Qb::new(dynamic_sql); }",
      ],
      [
        "copied QueryBuilder constructor",
        "fn probe() { let make = sqlx::QueryBuilder::<Sqlite>::new; let mut q = make(dynamic_sql); }",
      ],
      [
        "QueryBuilder with_arguments",
        "use sqlx::QueryBuilder as Qb; fn probe(arguments: Args) { let mut q = Qb::<Sqlite>::with_arguments(dynamic_sql, arguments); }",
      ],
      [
        "derived separated handle",
        'use sqlx::QueryBuilder; fn probe(query: &mut QueryBuilder<Sqlite>) { let mut separated = query.separated(", "); separated.push(dynamic_sql); }',
      ],
      [
        "push_unseparated fragment",
        "use sqlx::QueryBuilder; fn probe(query: &mut QueryBuilder<Sqlite>) { query.push_unseparated(dynamic_sql); }",
      ],
      [
        "query macro",
        "fn probe() { sqlx::query!(dynamic_sql); }",
      ],
      [
        "query_file macro",
        "fn probe() { sqlx::query_file!(dynamic_path); }",
      ],
      [
        "Executor UFCS",
        "fn probe(pool: &SqlitePool) { <&SqlitePool as sqlx::Executor>::execute(pool, dynamic_sql); }",
      ],
      [
        "imported Executor method",
        "use sqlx::Executor; fn probe(pool: &SqlitePool) { pool.execute(dynamic_sql); }",
      ],
      [
        "anonymous imported Executor method",
        "use sqlx::Executor as _; fn probe(pool: &SqlitePool) { pool.execute(dynamic_sql); }",
      ],
      [
        "grouped anonymous imported Executor method",
        "use sqlx::{Executor as _}; fn probe(pool: &SqlitePool) { pool.execute(dynamic_sql); }",
      ],
      [
        "SQLx prelude glob Executor method",
        "use sqlx::prelude::*; fn probe(pool: &SqlitePool) { pool.execute(dynamic_sql); }",
      ],
      [
        "SQLx prelude direct Executor method",
        "use sqlx::prelude::Executor; fn probe(pool: &SqlitePool) { pool.execute(dynamic_sql); }",
      ],
      [
        "SQLx prelude grouped Executor method",
        "use sqlx::prelude::{Executor as _}; fn probe(pool: &SqlitePool) { pool.execute(dynamic_sql); }",
      ],
      [
        "nested SQLx prelude glob Executor method",
        "use sqlx::{prelude::*}; fn probe(pool: &SqlitePool) { pool.execute(dynamic_sql); }",
      ],
      [
        "root-alias SQLx prelude glob Executor method",
        "use sqlx as db; use db::prelude::*; fn probe(pool: &SqlitePool) { pool.execute(dynamic_sql); }",
      ],
      [
        "SQLx prelude module-alias glob Executor method",
        "use sqlx::prelude as sp; use self::sp::*; fn probe(pool: &SqlitePool) { pool.execute(dynamic_sql); }",
      ],
    ] as const) {
      expect(
        sqlConsumerAnalysis(mutation).unresolved.length,
        `${label} must remain a fail-closed SQL consumer`,
      ).toBeGreaterThan(0);
    }
    expect(
      sqlConsumerAnalysis(`
        mod consumer {
          use sqlx::query as q;
          fn probe() { q(dynamic_sql); }
        }
        mod unrelated_sibling {
          fn q(_: &str) {}
          fn probe() { q(dynamic_sql); }
        }
      `).unresolved,
      "SQL aliases must not leak across sibling module scopes",
    ).toEqual(["query:dynamic_sql"]);
    expect(
      sqlConsumerAnalysis(`
        mod glob_consumer {
          use sqlx::*;
          fn probe() { query(glob_sql); }
        }
        mod unrelated_glob_sibling {
          fn query(_: &str) {}
          fn probe() { query(not_sql); }
        }
        mod extern_consumer {
          extern crate sqlx as db;
          fn probe() { db::query(extern_sql); }
        }
        mod unrelated_extern_sibling {
          mod db { pub fn query(_: &str) {} }
          fn probe() { db::query(not_sql); }
        }
      `).unresolved.sort(),
      "glob and extern-crate aliases stay in their lexical module scopes",
    ).toEqual([
      "query:extern_sql",
      "query:glob_sql",
    ]);
    for (const [label, source] of [
      [
        "a local item wins over a glob import",
        "use sqlx::*; fn query(_: &str) {} fn probe() { query(not_sql); }",
      ],
      [
        "a local closure shadows an explicit import",
        "use sqlx::query as q; fn probe() { let q = |_: &str| (); q(not_sql); }",
      ],
      [
        "a parent import is not inherited by a child module",
        "use sqlx::query as q; mod child { fn q(_: &str) {} fn probe() { q(not_sql); } }",
      ],
    ] as const) {
      expect(
        sqlConsumerAnalysis(source).unresolved,
        label,
      ).toEqual([]);
    }
    expect(
      sqlTables('sqlx::query("SELECT * FROM attached_foreign")'),
      "unknown and attached physical tables remain foreign",
    ).toEqual(["attached_foreign"]);
    expect(
      sqlTables('tracing::info!("SELECT * FROM analysis_runs")'),
      "non-consumer log strings are not executable SQL",
    ).toEqual([]);

    const cfgProbe = productionRust(`
      struct Probe {
        #[cfg(test)] hidden: i64,
        #[cfg(dev)] hidden_shorthand,
        live: i64,
      }
      #[cfg(test)]
      #[allow(dead_code)]
      fn hidden_sql() { sqlx::query("SELECT * FROM analysis_runs"); }
      fn live_sql() { sqlx::query("SELECT * FROM sources"); }
    `);
    expect(cfgProbe).not.toContain("hidden_sql");
    expect(cfgProbe).not.toContain("hidden_shorthand");
    expect(cfgProbe).toContain("live_sql");
    for (const conservative of [
      '#[cfg(any(test, windows))] fn maybe_live() { sqlx::query("SELECT * FROM analysis_runs"); }',
      '#[cfg_attr(test, ignore)] fn production() { sqlx::query("SELECT * FROM analysis_runs"); }',
    ]) expect(productionRust(conservative)).toContain("analysis_runs");
    expect(() => productionRust("#[cfg(test)]"))
      .toThrow(/has no Rust item/);

    const portableReferences = portable.flatMap((file) =>
      sqlTables(file.source).map((table) => ({ file: file.relative, table })));
    const appReferences = app.flatMap((file) =>
      sqlTables(file.source).map((table) => ({ file: file.relative, table })));
    const portableUnresolved = portable.flatMap((file) =>
      sqlConsumerAnalysis(file.source).unresolved
        .map((consumer) => `${file.relative}:${consumer}`));
    const appUnresolved = app.flatMap((file) =>
      unresolvedSqlInventory(file.relative, file.source));
    const corpusLiveSourceFingerprint = extracted
      ? "e381e12ef540b1c480105cf13e9edb0240129a88295fe7b19c3826d9723c67f1"
      : "7849ae49551d7627df09c6fe7ffd76f03fe8ba8fc6fa4d8fe27631123574ee75";
    const notebookExportSourceFingerprint = extracted
      ? "e5ce8f4c7a525702ec74385a9059af358c9a8211d1605e3d667bce034fc67f91"
      : "02f29dae9838edfa219cadd1bae38764f2b87380783a455888b6413010fc0991";
    const ingestProvenanceSourceFingerprint = telegramFoundationIsStaged
      ? "c964e21b5349808deea16ac71b9f4b93955968041fd3b62e00281ae6dde9adaf"
      : "5a7f28da697b149cf3ba2f9ff01074aa662ea5f6a56c983e8ee24f83816372a2";
    const sourcesStoreSourceFingerprint = telegramTakeoutBoundaryIsRetained
      ? "340890b2099e76c9da00e7c52d1746c13a25876b3d9a9580025c5f6601e3f7bb"
      : telegramPeerAvatarBoundaryIsRetained
      ? "01f4773cdc95e94f83c14b21efd0ae62976c3326895d0938925f6ab36c5d6167"
      : "28a8eb092ae42884e8b06bd6681bc36c8378cb1facea61fd8076b6ed284e3895";
    expect(
      portableUnresolved,
      "portable production unresolved SQL consumer inventory",
    ).toEqual([
      `${extracted ? "store/read_model.rs" : "store/owned_read_model.rs"}:query_builder_push:expression`,
      `${extracted ? "store/read_model.rs" : "store/owned_read_model.rs"}:query_builder_push:*field`,
    ]);
    expect(
      appUnresolved,
      "all-app production unresolved SQL consumer inventory",
    ).toEqual([
      `analysis/corpus/live.rs:push_analysis_document_kind_filter:2232ead4131e520b5236f3c26dd81965d52188ce205cbb15ed8edd0bb13fec13:${corpusLiveSourceFingerprint}:query_builder_push:table_alias`,
      `analysis/corpus/live.rs:push_analysis_document_kind_filter:2232ead4131e520b5236f3c26dd81965d52188ce205cbb15ed8edd0bb13fec13:${corpusLiveSourceFingerprint}:query_builder_push:table_alias`,
      `analysis/corpus/live.rs:push_analysis_document_kind_filter:2232ead4131e520b5236f3c26dd81965d52188ce205cbb15ed8edd0bb13fec13:${corpusLiveSourceFingerprint}:query_builder_push:table_alias`,
      `analysis/corpus/live.rs:push_analysis_document_kind_filter:2232ead4131e520b5236f3c26dd81965d52188ce205cbb15ed8edd0bb13fec13:${corpusLiveSourceFingerprint}:query_builder_push:table_alias`,
      "apalis_jobs.rs:apalis_jobs_prune_terminal_from_pool_with_hours:24948064c5dfb5fe043faf79c4bf4d05981516a9f5840bbec0bf0541dedd7adf:05b1185e47ec480cdeab26d4c62f499a747ac87a701a5830ecfca1eaf3095ba7:query_builder_push:&done_at_epoch",
      "apalis_jobs.rs:fetch_job_summaries:e3f8a21ab8453e7638518b00acd97e8b934c1f51080c257bd38db95b66f33216:05b1185e47ec480cdeab26d4c62f499a747ac87a701a5830ecfca1eaf3095ba7:query_builder_push:text_expr(schema, \"id\", \"''\")",
      "apalis_jobs.rs:fetch_payloads_for_ids:5735685cb0084ad5a0b182cfe13df418fdddb9b4eb2eb212e52b6c4110117814:05b1185e47ec480cdeab26d4c62f499a747ac87a701a5830ecfca1eaf3095ba7:query_builder_push:text_expr(schema, \"id\", \"''\")",
      "archive_read_model.rs:load_item_rows_from_archive:723126e361dfda6c463279e599a7d20d30fc0768543d46b8a07e056eb880496d:7f696cedd49b194c8de5579349f85150b2fe5f05a836554f874ffef84b36a913:query_as:&sql",
      `ingest_provenance.rs:mark_takeout_migrated_history_deferred:c6b01d419d14e767d3411693b5b15b2d1201eaf6c811b4ab568392e0c90e822c:${ingestProvenanceSourceFingerprint}:query:&query`,
      `ingest_provenance.rs:mark_takeout_only_my_messages_fallback:c73be9308109c5aa4b724423ccc547e0f7380fdb30bfff26f29ad4f93cfded64:${ingestProvenanceSourceFingerprint}:query:&query`,
      `notebooklm_export/query.rs:load_export_messages_from_items_path:18e50b7eb7b9e2033ea62c6df497ea82f566e289e990ba46b3a5a421fbc6d59d:${notebookExportSourceFingerprint}:query_as:&sql`,
      `notebooklm_export/query.rs:load_export_messages_from_items_path:18e50b7eb7b9e2033ea62c6df497ea82f566e289e990ba46b3a5a421fbc6d59d:${notebookExportSourceFingerprint}:query_as:&sql`,
      `notebooklm_export/query.rs:load_export_messages_from_items_path:18e50b7eb7b9e2033ea62c6df497ea82f566e289e990ba46b3a5a421fbc6d59d:${notebookExportSourceFingerprint}:query_as:&sql`,
      `notebooklm_export/query.rs:load_export_messages_from_items_path:18e50b7eb7b9e2033ea62c6df497ea82f566e289e990ba46b3a5a421fbc6d59d:${notebookExportSourceFingerprint}:query_as:&sql`,
      `notebooklm_export/query.rs:load_export_messages_from_archive:e65baa887a73e82ca531e193b82a7cfc7de0bfbc446b930dc6839ab019ec0574:${notebookExportSourceFingerprint}:query_as:&sql`,
      `notebooklm_export/query.rs:load_export_messages_from_archive:e65baa887a73e82ca531e193b82a7cfc7de0bfbc446b930dc6839ab019ec0574:${notebookExportSourceFingerprint}:query_as:&sql`,
      `notebooklm_export/query.rs:load_export_messages_from_archive:e65baa887a73e82ca531e193b82a7cfc7de0bfbc446b930dc6839ab019ec0574:${notebookExportSourceFingerprint}:query_as:&sql`,
      `notebooklm_export/query.rs:load_export_messages_from_archive:e65baa887a73e82ca531e193b82a7cfc7de0bfbc446b930dc6839ab019ec0574:${notebookExportSourceFingerprint}:query_as:&sql`,
      `notebooklm_export/query.rs:load_reply_contexts_from_archive:66738ed721e900eb0f17ec4c586505676b218d8db107cd91b70148cc4a157458:${notebookExportSourceFingerprint}:query_as:&sql`,
      "sources/items/query.rs:load_scoped_item_rows:bfb50958f6b5793b0731984b9e12200dd5459f1b5c6e77699e073202b5d525d0:429bc8566779242fc32476367b6cb587010ce1dc79b6c6ceb508e10730ef9c3a:query_as:&sql",
      "sources/items/query.rs:load_item_cursor:f8cae30a17e3e8b3502f58c8b655e7a958f33c1ba4884a6e169d1ff636466652:429bc8566779242fc32476367b6cb587010ce1dc79b6c6ceb508e10730ef9c3a:query_as:&sql",
      `sources/store.rs:delete_source_from_pool:f57c3fb2c5b7e0dab204d1ac4120eb119fde4c13a720a412e03feb126f616e8d:${sourcesStoreSourceFingerprint}:query:&format!( "PRAGMA busy_timeout = {SOURCE_DELETE_BUSY_TIMEOUT_MS}" )`,
      "takeout_import/validation_diagnostics.rs:scalar_i64:f7d827bd898cd0cd75ffb1152e53d54534dbacdd22cdd7befd324847a41dee8f:c10c3ed708c568efa64586bb5a6e8da9b3a037866b1ec5e4c604f2ba0dd03a9d:query_scalar:sql",
      "takeout_import/validation_diagnostics.rs:distribution:edacbda83ddb78e356b2db59916e5025eabfd93cd42058ecb1f837e5e94a70ee:c10c3ed708c568efa64586bb5a6e8da9b3a037866b1ec5e4c604f2ba0dd03a9d:query_as:sql",
      "takeout_import/validation_diagnostics.rs:push_mismatch_category:068f1c3248bbbe7d9519155502f1f53d840c219db4a185a0f28c327d400ed473:c10c3ed708c568efa64586bb5a6e8da9b3a037866b1ec5e4c604f2ba0dd03a9d:query_scalar:sample_sql",
      "topic_memberships.rs:rebuild_topic_memberships_for_source_on_connection:a548a49d36456704e46473b9c23ae2860d9f9ec0405f3457dde9cc2ef893049e:4212bc4f7942314464e44e7f730f1bbc076af6b289071d23bdb7dde786d0fe6b:query:&insert_sql",
      "topic_memberships.rs:resolve_scoped_topic_memberships_on_connection:9dcc4be3d441df7e1b916742ceed8bfb1fe6f5ace77f1f540694fc08d1008b7c:4212bc4f7942314464e44e7f730f1bbc076af6b289071d23bdb7dde786d0fe6b:query:&insert_sql",
      "topic_memberships.rs:resolve_scoped_topic_memberships_on_connection:9dcc4be3d441df7e1b916742ceed8bfb1fe6f5ace77f1f540694fc08d1008b7c:4212bc4f7942314464e44e7f730f1bbc076af6b289071d23bdb7dde786d0fe6b:query_scalar:&eligible_sql",
    ]);
    const ownedReadModel = selectedMoveSource("store/owned_read_model.rs");
    const runQueryFields = ownedReadModel.match(
      /const\s+RUN_QUERY_FIELDS:\s*\[&str;\s*7\]\s*=\s*\[([\s\S]*?)\];/,
    )?.[1];
    expect(runQueryFields, "RUN_QUERY_FIELDS declaration").toBeTypeOf("string");
    expect(rustStringValues(runQueryFields ?? "")).toEqual([
      "lower(coalesce(runs.scope_label_snapshot, ''))",
      "lower(coalesce(groups.name, ''))",
      "lower(coalesce(templates.name, ''))",
      "lower(coalesce(runs.provider_profile, ''))",
      "lower(coalesce(runs.provider, ''))",
      "lower(coalesce(runs.model, ''))",
      "lower(coalesce(runs.error, ''))",
    ]);
    const pushLike = rustFunctions(ownedReadModel)
      .filter(({ name }) => name === "push_like_predicate");
    expect(pushLike).toHaveLength(1);
    expect(
      normalized(maskRustNonCode(pushLike[0].body, false)),
      "the one dynamic expression push is a private fixed-column helper",
    ).toBe(normalized(String.raw`
      query.push(" AND ");
      query.push(expression);
      query.push(" LIKE ");
      query.push_bind(escaped_like_contains(value));
      query.push(" ESCAPE '\\'");
    `));
    expect(
      occurrenceCount(ownedReadModel, /\bpush_like_predicate\s*\(/g),
      "dynamic expression helper definition plus exact callers",
    ).toBe(4);
    for (const expression of [
      "lower(coalesce(runs.provider, ''))",
      "lower(coalesce(runs.model, ''))",
      "lower(coalesce(templates.name, ''))",
    ]) {
      expect(ownedReadModel).toContain(
        `push_like_predicate(&mut query, "${expression}",`,
      );
    }
    expect(
      occurrenceCount(ownedReadModel, /query\s*\.\s*push\s*\(\s*\*field\s*\)/g),
      "RUN_QUERY_FIELDS is the sole indirect field push",
    ).toBe(1);

    const appOwnedStringTokens = app.flatMap((file) =>
      rustStringValues(productionRust(file.source)).flatMap((literal) =>
        [...ownedTables]
          .filter((table) =>
            new RegExp(`\\b${table}\\b`, "i").test(literal))
          .map((table) => `${file.relative}:${table}`)));
    expect(
      appOwnedStringTokens,
      "owned table literals must not hide in any all-app production path",
    ).toEqual([]);

    const exactForeignExceptions = [
      "corpus/tests/snapshot.rs:list_run_snapshot_messages_page_does_not_fall_back_to_live_source",
      "corpus/tests/snapshot.rs:load_run_corpus_messages_does_not_reconstruct_completed_capture_failed_from_live_rows",
      "corpus/tests/snapshot.rs:trace_resolution_does_not_fall_back_to_live_source_for_completed_missing_snapshot",
    ];
    const crateDomainFiles = portableFiles()
      .filter(({ relative }) => relative !== "test_schema.rs");
    expect(
      rustMacroDefinitions(`
        #[cfg(test)]
        macro_rules! extra_seed {
          () => { sqlx::query("INSERT INTO sources (id) VALUES (-99)") };
        }
      `),
      "cfg(test) macro expansion cannot hide an extra foreign seed",
    ).toEqual(["extra_seed"]);
    expect(
      crateDomainFiles.flatMap((file) =>
        rustMacroDefinitions(file.source)
          .map((name) => `${file.relative}:${name}`)),
      "portable sources use no macro_rules expansion seam for hidden SQL",
    ).toEqual([]);
    const actualForeignExceptions = crateDomainFiles.flatMap((file) =>
      foreignTestExceptionNames(file.source, ownedTables)
        .map((name) => `${file.relative}:${name}`));
    expect(
      actualForeignExceptions.sort(),
      "exact three crate-test foreign-sentinel exceptions",
    ).toEqual(exactForeignExceptions);
    const snapshotTests = selectedMoveSource("corpus/tests/snapshot.rs");
    expect(
      foreignTestExceptionNames(
        `${snapshotTests}
        #[test]
        fn fourth_exception_is_rejected() {
          sqlx::query("INSERT INTO sources (id) VALUES (-99)");
        }`,
        ownedTables,
      ),
      "a fourth crate-test exception must be rejected",
    ).toEqual([
      "fourth_exception_is_rejected",
      "list_run_snapshot_messages_page_does_not_fall_back_to_live_source",
      "load_run_corpus_messages_does_not_reconstruct_completed_capture_failed_from_live_rows",
      "trace_resolution_does_not_fall_back_to_live_source_for_completed_missing_snapshot",
    ]);
    const expectedForeignTables = new Map<string, string[]>([
      [
        "list_run_snapshot_messages_page_does_not_fall_back_to_live_source",
        ["items", "sources"],
      ],
      [
        "load_run_corpus_messages_does_not_reconstruct_completed_capture_failed_from_live_rows",
        ["analysis_documents", "items", "sources"],
      ],
      [
        "trace_resolution_does_not_fall_back_to_live_source_for_completed_missing_snapshot",
        ["items", "sources"],
      ],
    ]);
    const setupEndMarkers = new Map<string, string>([
      [
        "list_run_snapshot_messages_page_does_not_fall_back_to_live_source",
        "let page = list_run_snapshot_messages_page(",
      ],
      [
        "load_run_corpus_messages_does_not_reconstruct_completed_capture_failed_from_live_rows",
        "let corpus = load_run_corpus_messages(",
      ],
      [
        "trace_resolution_does_not_fall_back_to_live_source_for_completed_missing_snapshot",
        "let messages = load_trace_resolution_messages(",
      ],
    ]);
    for (const test of rustTestFunctions(snapshotTests).filter(({ name }) =>
      expectedForeignTables.has(name))) {
      const foreign = sourceSqlOccurrences(test.body)
        .filter(({ table }) => !ownedTables.has(table));
      expect(
        foreign.map(({ table }) => table).sort(),
        `${test.name} exact foreign sentinel occurrence tables`,
      ).toEqual(expectedForeignTables.get(test.name));
      expect(
        foreign.map(({ operation }) => operation),
        `${test.name} foreign access is setup-only INSERT`,
      ).toEqual(foreign.map(() => "INSERT INTO"));
      expect(test.body).toContain(
        "super::super::super::test_schema::analysis_test_pool().await",
      );
      expect(
        occurrenceCount(
          test.body,
          /super::super::super::test_schema::analysis_test_pool\(\)\.await/g,
        ),
        `${test.name} canonical schema pool count`,
      ).toBe(1);
      const setupEnd = test.body.indexOf(setupEndMarkers.get(test.name) ?? "");
      expect(setupEnd, `${test.name} exact setup boundary`).toBeGreaterThan(0);
      for (const occurrence of foreign) {
        expect(
          occurrence.sql,
          `${test.name} negative foreign sentinel occurrence`,
        ).toMatch(
          /\bINSERT(?:\s+OR\s+\w+)?\s+INTO\b[\s\S]*?\bVALUES\s*\(\s*-\d+/i,
        );
        expect(
          occurrence.sourceOffset,
          `${test.name} every foreign sentinel occurrence precedes production read`,
        ).toBeLessThan(setupEnd);
      }
    }
    const crateTestUnresolved = crateDomainFiles.flatMap((file) =>
      rustTestFunctions(file.source).flatMap((test) =>
        sqlConsumerAnalysis(test.body).unresolved
          .map((consumer) =>
            `${file.relative}:${test.name}:${executableFingerprint(test.body)}:${consumer}`)));
    expect(
      crateTestUnresolved,
      "exact vetted crate-test unresolved SQL consumer inventory",
    ).toEqual([
      `${extracted ? "chat.rs" : "chat_engine.rs"}:chat_execution_persists_turns_before_completed_event:${extracted ? "919782bccff127d7653fde08dcb032eacae40d811f33f8a28be16561bf6bea8c" : "94cd4e00a832b84026c636bfde8a5da25e7fe026f2006a95745f7062c5790e02"}:query:statement`,
    ]);
    for (const file of crateDomainFiles) {
      const withoutTests = file.source.split("");
      for (const test of rustTestFunctions(file.source)) {
        for (
          let index = test.declarationStart;
          index <= test.bodyEnd;
          index += 1
        ) {
          if (withoutTests[index] !== "\n") withoutTests[index] = " ";
        }
      }
      expect(
        sourceSqlReferences(withoutTests.join(""))
          .filter(({ table }) => !ownedTables.has(table)),
        `${file.relative} foreign SQL outside exact test bodies`,
      ).toEqual([]);
      expect(
        maskRustNonCode(file.source, true),
        `${file.relative} must not import an app source builder`,
      ).not.toMatch(
        /\b(?:AppAnalysisCorpusReader|AppAnalysisRunPreflightLimits|AppYoutubeCorpusMode|apply_all_migrations_for_test_pool|create_test_pool)\b/,
      );
    }
    const foreignKeyHelper = rustFunctions(snapshotTests)
      .filter(({ name }) =>
        name === "insert_orphan_group_member_with_foreign_keys_restored");
    expect(foreignKeyHelper).toHaveLength(1);
    expect(foreignKeyHelper[0].visibility).toBe("");
    expect(canonicalRustSignature(foreignKeyHelper[0].signature)).toBe(
      canonicalRustSignature(
        "async fn insert_orphan_group_member_with_foreign_keys_restored(pool: &sqlx::SqlitePool, group_id: i64, source_id: i64)",
      ),
    );
    expect(
      sqlConsumerAnalysis(foreignKeyHelper[0].body).sql.map(normalized),
      "foreign-key helper exact disable/insert/restore/assert query sequence",
    ).toEqual([
      "PRAGMA foreign_keys = OFF",
      "INSERT INTO analysis_source_group_members (group_id, source_id, created_at) VALUES (?, ?, 1)",
      "PRAGMA foreign_keys = ON",
      "PRAGMA foreign_keys",
    ]);
    expect(
      crateDomainFiles.reduce(
        (count, file) =>
          count
          + occurrenceCount(file.source, /PRAGMA\s+foreign_keys\s*=\s*OFF/gi),
        0,
      ),
      "one private foreign-key suppression helper",
    ).toBe(1);
    expect(snapshotTests).toContain(
      "foreign keys must be restored before releasing the sole connection",
    );
    expect(
      occurrenceCount(
        snapshotTests,
        /\binsert_orphan_group_member_with_foreign_keys_restored\s*\(/g,
      ),
    ).toBe(2);

    for (const reference of portableReferences) {
      expect(
        ownedTables.has(reference.table),
        `portable production ${reference.file} names foreign table ${reference.table}`,
      ).toBe(true);
    }
    for (const reference of appReferences) {
      expect(
        ownedTables.has(reference.table),
        `app production ${reference.file} names owned table ${reference.table}`,
      ).toBe(false);
    }
    expect(new Set(portableReferences.map(({ table }) => table)))
      .toEqual(ownedTables);

    const applicationContract = read(
      "src/lib/analysis-application-contract.test.ts",
    );
    for (const marker of [
      "keeps analysis SQL ownership and borrowed coordinator capabilities fail closed",
      "limits moving crate-test foreign sentinels and foreign-key suppression",
      "unresolvedConsumerInventory",
      "moving source unresolved executable SQL consumer inventory",
    ]) {
      expect(applicationContract).toContain(marker);
    }
    expect(
      read("src-tauri/src/analysis/mod.rs"),
      "dev fixtures must remain an explicitly cfg(dev) app module",
    ).toMatch(/#\[cfg\(dev\)\]\s*mod fixtures;/);
  }, 15_000);

  it("pins pool APIs and exactly four borrowed-connection workflow families", () => {
    const portable = portableProductionFiles();
    const participants = transactionMap.map(({ participant }) => participant);
    const publicBorrowed = portable.flatMap((file) =>
      rustFunctions(file.source)
        .filter((fn) =>
          fn.visibility === "pub"
          && /&mut\s+SqliteConnection/.test(firstParameter(fn.params)))
        .map((fn) => fn.name));
    exactInventory(publicBorrowed, participants, "public borrowed participants");

    for (const participant of participants) {
      const fn = soleFunction(portable, participant);
      expect(normalized(firstParameter(fn.params)), participant)
        .toMatch(/^conn:\s*&mut\s+SqliteConnection$/);
      expect(`${fn.params}\n${fn.body}`, participant)
        .not.toMatch(/\bSqlitePool\b|\bPool\s*<\s*Sqlite\s*>|\.acquire\s*\(|\.begin\s*\(|\.commit\s*\(|\.rollback\s*\(/);
    }

    const ordinaryPoolDefinitions: Record<string, string> = {
      list_analysis_prompt_templates: "list_analysis_prompt_templates_in_pool",
      create_analysis_prompt_template: "create_analysis_prompt_template_in_pool",
      update_analysis_prompt_template: "update_analysis_prompt_template_in_pool",
      delete_analysis_prompt_template: "delete_analysis_prompt_template_in_pool",
      create_analysis_source_group: "create_analysis_source_group_in_pool",
      update_analysis_source_group: "update_analysis_source_group_in_pool",
      delete_analysis_source_group: "delete_analysis_source_group_in_pool",
      get_analysis_source_group_record: "get_analysis_source_group_record",
      list_analysis_run_messages: "list_run_snapshot_messages_page",
      get_analysis_run_trace: "get_analysis_run_trace_in_pool",
      delete_analysis_run: "delete_analysis_run",
      resolve_analysis_trace_refs: "resolve_analysis_trace_refs_in_pool",
      list_analysis_chat_messages: "list_analysis_chat_messages_in_pool",
      clear_analysis_chat_messages: "clear_analysis_chat_messages_in_pool",
      load_analysis_chat_run: "load_analysis_chat_run",
      analysis_run_ids_depending_on_sources: "analysis_run_ids_depending_on_sources",
      load_analysis_run_diagnostics: "load_analysis_run_diagnostics",
      mark_interrupted_analysis_runs: "mark_interrupted_analysis_runs",
      request_analysis_run_cancel: "request_analysis_run_cancel_in_pool",
    };
    for (const [api, definition] of Object.entries(ordinaryPoolDefinitions)) {
      const fn = solePublicFunction(portable, definition);
      expect(
        normalized(firstParameter(fn.params)),
        `${api} ordinary pool capability`,
      ).toMatch(
        /^(?:pool:\s*&(?:SqlitePool|Pool\s*<\s*Sqlite\s*>)|pool:\s*Option<&SqlitePool>)$/,
      );
    }
    expect(transactionMap.map(({ family }) => new Set([family])).flat())
      .toHaveLength(8);
    expect([...new Set(transactionMap.map(({ family }) => family))])
      .toEqual([1, 2, 3, 4]);
  });

  it("threads one app-owned transaction through every approved coordinator", () => {
    const appFiles = [
      ...sourceFiles(appAnalysisRoot, "app"),
      ...sourceFiles(path.join(repoRoot, "src-tauri/src/projects"), "app"),
      {
        owner: "app" as const,
        relative: "notebooklm_export/query.rs",
        source: read("src-tauri/src/notebooklm_export/query.rs"),
      },
    ];
    const childNames = [
      "match_foreign_labels",
      "prepare_analysis_run_summaries",
      "prepare_active_analysis_run_summaries",
      "prepare_analysis_run_detail",
      "prepare_legacy_analysis_chat_run",
      "load_foreign_labels",
      "load_analysis_chat_run",
      "load_analysis_source_groups_for_enrichment",
      "load_analysis_source_group_for_enrichment",
      "enrich_analysis_source_group",
      "load_project_analysis_run_aggregates",
      "delete_project_analysis_runs",
    ] as const;
    const contracts = [
      {
        name: "list_analysis_runs_in_pool",
        participant: "prepare_analysis_run_summaries",
        awaits: 5,
        freeCalls: [
          "match_foreign_labels",
          "prepare_analysis_run_summaries",
          "load_foreign_labels",
        ],
        qualifiedCalls: ["AppError::database", "AppError::database"],
        topology: [
          "begin",
          "match_foreign_labels(&mut *transaction, filters.foreign_label_search_terms())",
          "prepare_analysis_run_summaries(&mut *transaction, filters, matches)",
          "load_foreign_labels(&mut *transaction, enrichment.foreign_label_refs())",
          "commit",
        ],
      },
      {
        name: "list_active_analysis_runs_in_pool",
        participant: "prepare_active_analysis_run_summaries",
        awaits: 4,
        freeCalls: [
          "prepare_active_analysis_run_summaries",
          "load_foreign_labels",
        ],
        qualifiedCalls: ["AppError::database", "AppError::database"],
        topology: [
          "begin",
          "prepare_active_analysis_run_summaries(&mut *transaction, run_ids)",
          "load_foreign_labels(&mut *transaction, enrichment.foreign_label_refs())",
          "commit",
        ],
      },
      {
        name: "get_analysis_run_in_pool",
        participant: "prepare_analysis_run_detail",
        awaits: 4,
        freeCalls: ["prepare_analysis_run_detail", "load_foreign_labels"],
        qualifiedCalls: ["AppError::database", "AppError::database"],
        topology: [
          "begin",
          "prepare_analysis_run_detail(&mut *transaction, run_id)",
          "load_foreign_labels(&mut *transaction, enrichment.foreign_label_refs())",
          "commit",
        ],
      },
      {
        name: "resolve_legacy_analysis_chat_run_in_pool",
        participant: "prepare_legacy_analysis_chat_run",
        awaits: 5,
        freeCalls: [
          "load_analysis_chat_run",
          "prepare_legacy_analysis_chat_run",
          "load_foreign_labels",
        ],
        qualifiedCalls: [
          "AppError::database",
          "AppError::not_found",
          "AppError::database",
        ],
        topology: [
          "load_analysis_chat_run(pool, run_id)",
          "begin",
          "prepare_legacy_analysis_chat_run(&mut *transaction, run_id)",
          "load_foreign_labels(&mut *transaction, enrichment.foreign_label_refs())",
          "commit",
        ],
      },
      {
        name: "list_analysis_source_groups_in_pool",
        participant: "load_analysis_source_groups_for_enrichment",
        awaits: 4,
        freeCalls: [
          "load_analysis_source_groups_for_enrichment",
          "enrich_analysis_source_group",
        ],
        qualifiedCalls: ["AppError::database", "AppError::database"],
        topology: [
          "begin",
          "load_analysis_source_groups_for_enrichment(&mut *transaction)",
          "enrich_analysis_source_group(&mut *transaction, record)",
          "commit",
        ],
      },
      {
        name: "get_analysis_source_group_response_in_pool",
        participant: "load_analysis_source_group_for_enrichment",
        awaits: 4,
        freeCalls: [
          "load_analysis_source_group_for_enrichment",
          "enrich_analysis_source_group",
        ],
        qualifiedCalls: ["AppError::database", "AppError::database"],
        topology: [
          "begin",
          "load_analysis_source_group_for_enrichment(&mut *transaction, group_id)",
          "enrich_analysis_source_group(&mut *transaction, record)",
          "commit",
        ],
      },
      {
        name: "load_export_source_group_in_pool",
        participant: "load_analysis_source_group_for_enrichment",
        awaits: 4,
        freeCalls: ["load_analysis_source_group_for_enrichment"],
        qualifiedCalls: [
          "AppError::database",
          "AppError::not_found",
          "AppError::database",
          "AppError::database",
        ],
        topology: [
          "begin",
          "load_analysis_source_group_for_enrichment(&mut *transaction, source_group_id)",
          "fetch_all(&mut *transaction)",
          "commit",
        ],
      },
      {
        name: "list_research_projects_in_pool",
        participant: "load_project_analysis_run_aggregates",
        awaits: 4,
        freeCalls: [
          "load_project_analysis_run_aggregates",
          "map_project_summary",
        ],
        qualifiedCalls: [
          "AppError::database",
          "sqlx::query_as",
          "AppError::database",
          "AppError::database",
        ],
        topology: [
          "begin",
          "fetch_all(&mut *transaction)",
          "load_project_analysis_run_aggregates(&mut *transaction, &project_ids)",
          "commit",
        ],
      },
      {
        name: "delete_project_in_pool",
        participant: "delete_project_analysis_runs",
        awaits: 5,
        freeCalls: ["delete_project_analysis_runs"],
        qualifiedCalls: [
          "AppError::database",
          "sqlx::query",
          "AppError::database",
          "sqlx::query",
          "AppError::database",
          "AppError::database",
          "AppError::not_found",
        ],
        topology: [
          "begin",
          "delete_project_analysis_runs(&mut *transaction, project_id)",
          "execute(&mut *transaction)",
          "execute(&mut *transaction)",
          "commit",
        ],
      },
    ] as const;
    expect(contracts).toHaveLength(9);
    const expectedCoordinatorMethods: Record<string, string[]> = {
      list_analysis_runs_in_pool: [
        "begin()",
        "map_err(AppError::database)",
        "foreign_label_search_terms()",
        "foreign_label_refs()",
        "finish(labels)",
        "commit()",
        "map_err(AppError::database)",
      ],
      list_active_analysis_runs_in_pool: [
        "begin()",
        "map_err(AppError::database)",
        "foreign_label_refs()",
        "finish(labels)",
        "commit()",
        "map_err(AppError::database)",
      ],
      get_analysis_run_in_pool: [
        "begin()",
        "map_err(AppError::database)",
        "foreign_label_refs()",
        "finish(labels)",
        "commit()",
        "map_err(AppError::database)",
      ],
      resolve_legacy_analysis_chat_run_in_pool: [
        "needs_legacy_foreign_label()",
        "begin()",
        "map_err(AppError::database)",
        "foreign_label_refs()",
        "finish(labels)",
        'ok_or_else(|| AppError::not_found(format!("Analysis run {run_id} not found")))',
        "commit()",
        "map_err(AppError::database)",
      ],
      list_analysis_source_groups_in_pool: [
        "begin()",
        "map_err(AppError::database)",
        "len()",
        "push(enrich_analysis_source_group(&mut *transaction, record).await?)",
        "commit()",
        "map_err(AppError::database)",
      ],
      get_analysis_source_group_response_in_pool: [
        "begin()",
        "map_err(AppError::database)",
        "commit()",
        "map_err(AppError::database)",
      ],
      load_export_source_group_in_pool: [
        "begin()",
        "map_err(AppError::database)",
        'ok_or_else(|| AppError::not_found(format!("Source group {source_group_id} not found")))',
        "member_source_ids()",
        "is_empty()",
        'separated(", ")',
        "member_source_ids()",
        "push_bind(source_id)",
        'push(r#" ) ORDER BY COALESCE(sources.title, \'\'), sources.id "#,)',
        "build_query_as()",
        "fetch_all(&mut *transaction)",
        "map_err(AppError::database)",
        "into_iter()",
        "map(|row| NotebookLmExportSourceGroupMember { source_id: row.source_id, source_title: row.source_title, source_type: row.source_type, })",
        "collect()",
        "source_kind()",
        "to_string()",
        "id()",
        "name()",
        "to_string()",
        "commit()",
        "map_err(AppError::database)",
      ],
      list_research_projects_in_pool: [
        "begin()",
        "map_err(AppError::database)",
        "fetch_all(&mut *transaction)",
        "map_err(AppError::database)",
        "iter()",
        "map(|row| row.id)",
        "collect()",
        "into_iter()",
        "map(|aggregate| (aggregate.project_id(), aggregate))",
        "project_id()",
        "collect()",
        "into_iter()",
        "map(|row| { let aggregate = aggregates_by_project_id.get(&row.id); map_project_summary(row, aggregate) })",
        "get(&row.id)",
        "collect()",
        "commit()",
        "map_err(AppError::database)",
      ],
      delete_project_in_pool: [
        "begin()",
        "map_err(AppError::database)",
        "bind(project_id)",
        "execute(&mut *transaction)",
        "map_err(AppError::database)",
        "bind(project_id)",
        "execute(&mut *transaction)",
        "map_err(AppError::database)",
        "commit()",
        "map_err(AppError::database)",
        "rows_affected()",
      ],
    };
    const coordinatorProbe = `
      let mut transaction = pool.begin().await?;
      prepare_analysis_run_detail(&mut *transaction, run_id).await?;
      transaction.commit().await.map_err(AppError::database)?;
    `;
    expect(
      coordinatorTopology(coordinatorProbe, childNames),
      "coordinator parser baseline",
    ).toEqual([
      "begin",
      "prepare_analysis_run_detail(&mut *transaction, run_id)",
      "commit",
    ]);
    expect(
      coordinatorTopology(
        coordinatorProbe.replace(
          "transaction.commit()",
          "load_foreign_labels(&mut *transaction, refs).await?; transaction.commit()",
        ),
        childNames,
      ),
      "an extra transaction participant mutates exact topology",
    ).not.toEqual([
      "begin",
      "prepare_analysis_run_detail(&mut *transaction, run_id)",
      "commit",
    ]);
    expect(
      detachedCoordinatorBypasses(
        "tokio::spawn(async move { let _ = pool.clone(); });",
      ),
      "detached/autocommit bypass mutation",
    ).toEqual(["detached-future", "pool.clone", "spawn"]);
    const planPairs = transactionMap.flatMap(({ participant, coordinators }) =>
      coordinators.map((name) => `${name}:${participant}`));
    exactInventory(
      contracts.map(({ name, participant }) => `${name}:${participant}`),
      planPairs,
      "transaction plan coordinator/participant pairs",
    );

    for (const contract of contracts) {
      const fn = soleFunction(appFiles, contract.name);
      const syntax = maskRustNonCode(fn.body, true);
      expect(fn.file.owner, contract.name).toBe("app");
      expect(
        coordinatorTopology(fn.body, childNames),
        `${contract.name} exact async/SQL topology`,
      ).toEqual(contract.topology);
      expect(
        unqualifiedFreeFunctionCalls(fn.body),
        `${contract.name} exact local helper calls`,
      ).toEqual(contract.freeCalls);
      expect(
        qualifiedCallablePaths(fn.body),
        `${contract.name} exact qualified calls`,
      ).toEqual(contract.qualifiedCalls);
      expect(
        methodCallInventory(fn.body),
        `${contract.name} exact receiver-method call inventory`,
      ).toEqual(expectedCoordinatorMethods[contract.name]);
      expect(
        macroInvocationInventory(fn.body),
        `${contract.name} exact macro invocation inventory`,
      ).toEqual(
        [
          "resolve_legacy_analysis_chat_run_in_pool",
          "load_export_source_group_in_pool",
          "delete_project_in_pool",
        ].includes(contract.name)
          ? ["format"]
          : [],
      );
      expect(awaitSiteCount(fn.body), `${contract.name} exact await sites`)
        .toBe(contract.awaits);
      expect(detachedCoordinatorBypasses(fn.body), contract.name).toEqual([]);
      expect(
        occurrenceCount(syntax, /\.\s*begin\s*\(\s*\)/g),
        `${contract.name} exactly one begin on any receiver`,
      ).toBe(1);
      expect(
        occurrenceCount(syntax, /\.\s*commit\s*\(\s*\)/g),
        `${contract.name} exactly one commit on any receiver`,
      ).toBe(1);
      expect(syntax, `${contract.name} forbidden connection control`).not
        .toMatch(/\.\s*(?:acquire|connect|rollback)\s*\(/);
      expect(
        rustStringValues(fn.body)
          .filter((literal) =>
            /\b(?:BEGIN|COMMIT|ROLLBACK|SAVEPOINT|RELEASE)\b/i.test(literal)),
        `${contract.name} raw transaction-control SQL`,
      ).toEqual([]);
      expect(
        sqlTables(fn.body).filter((table) => ownedTables.has(table)),
        `${contract.name} raw owned SQL`,
      ).toEqual([]);
    }
  });

  it("pins resolved scope corpus and distinct read A/read B handoffs", () => {
    const models = selectedMoveSource("models.rs");
    const corpus = selectedMoveSource("corpus_portable.rs");
    const report = selectedMoveSource("report_engine.rs");
    const capture = selectedMoveSource("report/capture.rs");
    const appReport = selectedAppSource("report.rs");
    expect(
      exactPublicTypeAlias(corpus, "AnalysisPortFuture"),
      "AnalysisPortFuture exact public alias",
    ).toBe(normalized(
      "pub type AnalysisPortFuture<'a, T> = Pin<Box<dyn Future<Output = AppResult<T>> + Send + 'a>>;",
    ));
    expect(
      exactPublicTrait(corpus, "AnalysisCorpusReader"),
      "AnalysisCorpusReader exact public trait surface",
    ).toBe(normalized(`
      pub trait AnalysisCorpusReader: Send + Sync + 'static {
        fn load_corpus(
          &self,
          request: AnalysisCorpusRequest,
        ) -> AnalysisPortFuture<'_, Vec<AnalysisCorpusMessage>>;
      }
    `));
    expect(
      exactPublicTrait(models, "AnalysisEventSink"),
      "AnalysisEventSink exact public trait surface",
    ).toBe(normalized(`
      pub trait AnalysisEventSink: Send + Sync + 'static {
        fn publish_run(&self, event: AnalysisRunEvent);
        fn publish_chat(&self, event: AnalysisChatEvent);
      }
    `));
    const portableDiagnosticLeaks = portableFiles().flatMap((file) =>
      occurrenceCount(
        file.source,
        /\bskipped_unlinked_playlist_items\b/g,
      ) === 0
        ? []
        : [file.relative]);
    expect(
      portableDiagnosticLeaks,
      "playlist skip diagnostics remain outside every portable source/ticket",
    ).toEqual([]);
    const appDiagnosticInventory = sourceFiles(appAnalysisRoot, "app")
      .flatMap((file) => {
        const count = occurrenceCount(
          file.source,
          /\bskipped_unlinked_playlist_items\b/g,
        );
        return count === 0 ? [] : [`${file.relative}:${count}`];
      })
      .sort();
    expect(
      appDiagnosticInventory,
      "playlist skip diagnostics stay in the app resolver and its characterization",
    ).toEqual([
      "corpus/source_resolution.rs:12",
      "corpus/tests/source_resolution.rs:1",
    ]);
    const appScopeResolution = selectedAppSource(
      "corpus/source_resolution.rs",
    );
    expect(
      structFields(appScopeResolution, "AppAnalysisScopeResolution")
        .map(({ visibility, name, type }) => [visibility, name, type]),
      "app-private scope wrapper exact field confinement",
    ).toEqual([
      ["", "scope", "ResolvedAnalysisScope"],
      ["", "skipped_unlinked_playlist_items", "usize"],
    ]);
    for (const signature of [
      "pub(crate) fn scope(&self) -> &ResolvedAnalysisScope",
      "pub(crate) fn into_scope(self) -> ResolvedAnalysisScope",
    ]) expect(appScopeResolution).toContain(signature);
    expect(
      appScopeResolution,
      "playlist skip diagnostic accessor remains characterization-only",
    ).toMatch(
      /#\[cfg\(test\)\]\s+pub\(crate\)\s+fn\s+skipped_unlinked_playlist_items\s*\(\s*&self\s*\)\s*->\s*usize/,
    );
    for (const marker of [
      "pub struct ResolvedAnalysisScope",
      "scope_kind: AnalysisScopeKind",
      "source_ids: Vec<i64>",
      "scope_label_snapshot: String",
      "pub fn source_ids(&self) -> &[i64]",
      "pub fn scope_label_snapshot(&self) -> &str",
    ]) expect(models).toContain(marker);
    for (const marker of [
      "pub trait AnalysisCorpusReader",
      "fn load_corpus(",
      "pub struct AnalysisCorpusRequest",
      "pub struct AnalysisCorpusMessage",
      "pub async fn preflight_analysis_corpus(",
      "reader.load_corpus(request.clone()).await?",
    ]) expect(corpus).toContain(marker);
    expect(capture).toContain("pub async fn capture_analysis_corpus(");
    expect(capture).toContain("reader.load_corpus(request.clone()).await");
    const preparation = soleFunction(
      [{ owner: extracted ? "crate" : "app", relative: "report.rs", source: report }],
      "prepare_analysis_report_execution",
    );
    expect(preparation.body).toContain("preflight_analysis_run(");
    expect(preparation.body).not.toContain("capture_analysis_corpus(");
    expect(report).toContain("capture_analysis_corpus(");
    expect(occurrenceCount(appReport, /AppAnalysisCorpusReader::new\s*\(/g))
      .toBe(2);
    expectOrdered(appReport, [
      "let reader = AppAnalysisCorpusReader::new(pool.clone())",
      "prepare_analysis_report_execution(",
      "tokio::spawn(async move",
      "get_pool(&app_handle).await",
      "let reader = AppAnalysisCorpusReader::new(execution_pool.clone())",
      "execute_analysis_report(",
    ]);
    const corpusPortTests = selectedMoveSource("report/tests/corpus_port.rs");
    for (const leaf of [
      "report_execution_uses_distinct_preflight_and_capture_corpus_reads",
      "started_load_items_uses_preflight_summary_before_empty_capture_failure",
      "started_load_items_uses_preflight_summary_before_error_capture_failure",
    ]) expect(corpusPortTests).toContain(`fn ${leaf}(`);
  });

  it("keeps commands events spawning migrations and fixtures app-owned", () => {
    const expectedCommands = [
      ...commands.analysis,
      ...commands.project,
      ...commands.dev,
    ];
    expect(expectedCommands).toHaveLength(27);
    expect(new Set(expectedCommands).size).toBe(27);
    const definitions = commandFunctions();
    exactInventory(
      definitions
        .filter(({ file }) => file.relative.startsWith("projects/"))
        .map(({ name }) => name),
      [
        "list_research_projects",
        "list_projects",
        "create_project",
        "update_project",
        "delete_project",
        "set_project_pinned",
        "set_project_archived",
        "list_project_sources",
        "add_project_sources",
        "remove_project_sources",
        "delete_project_youtube_video_source_from_library",
        "start_project_analysis",
        "list_project_runs",
        "get_project_data_range",
      ],
      "all project-scope Tauri command definitions",
    );
    const syntheticCommands = commandFunctions([{
      owner: "app",
      relative: "projects/mutation.rs",
      source: `
        #[tauri::command(rename_all = "snake_case")]
        fn option_bearing_command() {}
        #[cfg_attr(all(), tauri::command)]
        fn cfg_attr_command() {}
        #[::tauri::command]
        fn absolute_command() {}
        mod imported_scope {
          use tauri::command as adapter;
          #[adapter]
          fn imported_command() {}
          #[cfg_attr(all(), adapter)]
          fn cfg_attr_imported_command() {}
        }
        mod grouped_root_alias {
          use tauri::{self as ui};
          #[ui::command(rename_all = "snake_case")]
          fn grouped_alias_command() {}
        }
        mod absolute_root_alias {
          use ::tauri as ui;
          #[ui::command]
          fn absolute_alias_command() {}
        }
        mod unrelated_sibling {
          #[adapter]
          fn alias_must_not_leak() {}
        }
      `,
    }]).map(({ name }) => name);
    expect(
      syntheticCommands,
      "option-bearing/imported attrs are commands without sibling alias leaks",
    ).toEqual([
      "option_bearing_command",
      "cfg_attr_command",
      "absolute_command",
      "imported_command",
      "cfg_attr_imported_command",
      "grouped_alias_command",
      "absolute_alias_command",
    ]);
    expect(
      commandFunctions(portableFiles()).map(({ name }) => name),
      "portable sources own no direct, aliased, or cfg_attr Tauri command",
    ).toEqual([]);
    exactInventory(
      definitions
        .filter(({ file }) => file.relative.startsWith("analysis/"))
        .map(({ name }) => name),
      [...commands.analysis, ...commands.dev],
      "analysis-owned Tauri command definitions",
    );
    const projectModuleCommands = definitions
      .filter(({ file }) => file.relative === "projects/mod.rs")
      .sort((left, right) => left.declarationStart - right.declarationStart);
    const projectAnalysisStart = projectModuleCommands.find(
      ({ name }) => name === "start_project_analysis",
    )?.declarationStart;
    if (projectAnalysisStart === undefined) {
      throw new Error("missing project analysis command section");
    }
    exactInventory(
      projectModuleCommands
        .filter(({ declarationStart }) =>
          declarationStart >= projectAnalysisStart)
        .map(({ name }) => name),
      ["start_project_analysis", "list_project_runs"],
      "terminal project analysis Tauri command section",
    );
    exactInventory(
      definitions
        .filter(({ file }) => file.relative === "projects/data_range.rs")
        .map(({ name }) => name),
      ["get_project_data_range"],
      "project data-range Tauri command file",
    );
    for (const command of expectedCommands) {
      expect(
        definitions.filter(({ name }) => name === command),
        `app command ${command}`,
      ).toHaveLength(1);
    }
    const registration = handlerBody();
    const handlerEntries = splitTopLevel(registration)
      .map((entry) =>
        entry.replace(/#\s*\[[^\]]+\]/g, "").trim())
      .filter(Boolean);
    for (const entry of handlerEntries) {
      if (!/^[A-Za-z_][A-Za-z0-9_]*$/.test(entry)) {
        throw new Error(`unparsed Tauri handler entry: ${entry}`);
      }
    }
    exactInventory(
      handlerEntries.filter((entry) =>
        /analysis/.test(entry)
        || entry === "list_project_runs"
        || entry === "get_project_data_range"),
      expectedCommands,
      "analysis/project/dev generate_handler entries",
    );
    expect(
      [...handlerEntries, "extra_analysis_command"].filter((entry) =>
        /analysis/.test(entry)
        || entry === "list_project_runs"
        || entry === "get_project_data_range").sort(),
      "an extra relevant handler mutates the exact registration inventory",
    ).not.toEqual([...expectedCommands].sort());
    for (const command of expectedCommands) {
      expect(
        occurrenceCount(
          maskRustNonCode(registration, true),
          new RegExp(`\\b${command}\\b`, "g"),
        ),
        `registered command ${command}`,
      ).toBe(1);
    }

    const crateSource = sourceFiles(analysisCrateRoot, "crate")
      .map(({ source }) => source)
      .join("\n");
    expect(maskRustNonCode(crateSource, true)).not.toMatch(
      /#\s*\[\s*tauri\s*::\s*command\s*\]|\bAppHandle\b|\bget_pool\b|\btokio\s*::\s*spawn\s*\(/,
    );
    const portableProductionSources = portableProductionFiles();
    const portableProduction = portableProductionSources
      .map(({ source }) => productionRust(source))
      .join("\n");
    expect(
      portableProductionSources.flatMap((file) =>
        resolvedSqlxMacroInvocations(file.source, "migrate")
          .map((offset) => `${file.relative}:${offset}`)),
      "portable production owns no direct or aliased sqlx migrate macro",
    ).toEqual([]);
    expect(
      resolvedSqlxMacroInvocations(
        'use sqlx as db; fn probe() { let _ = db::migrate!("../../../migrations"); }',
        "migrate",
      ),
      "a resolved sqlx root alias cannot hide migrate!",
    ).toHaveLength(1);
    expect(
      resolvedSqlxMacroInvocations(
        'mod parent { use sqlx::migrate as m; mod child { fn probe() { let _ = super::m!("../../../migrations"); } } }',
        "migrate",
      ),
      "a qualified parent macro alias cannot hide migrate!",
    ).toHaveLength(1);
    expect(
      resolvedSqlxMacroInvocations(
        'pub use sqlx::migrate as m; fn probe() { let _ = crate::m!("../../../migrations"); }',
        "migrate",
      ),
      "a crate-qualified macro re-export cannot hide migrate!",
    ).toHaveLength(1);
    expect(
      maskRustNonCode(portableProduction, true),
      "portable production owns no migration runner or embedded migration root",
    ).not.toMatch(
      /\bMigrator\b|\bmigrations?\s*::|include_(?:str|bytes)\s*!\s*\([^)]*migrations?/,
    );
    const portableWithoutTestSchema = portableFiles()
      .filter(({ relative }) => relative !== "test_schema.rs")
      .map(({ source }) => source)
      .join("\n");
    expect(
      maskRustNonCode(portableWithoutTestSchema, true),
      "portable crate sources own no cfg(dev) fixture capability",
    ).not.toMatch(
      /#\s*\[\s*cfg\s*\(\s*dev\s*\)\s*\]|\b(?:AnalysisRedesignFixtureSummary|seed_analysis_redesign_fixtures|clear_analysis_redesign_fixtures)\b/,
    );
    const appModule = selectedAppSource("mod.rs");
    expect(appModule).toContain('const ANALYSIS_RUN_EVENT: &str = "analysis://run"');
    expect(appModule).toContain('const ANALYSIS_CHAT_EVENT: &str = "analysis://chat"');
    expect(selectedAppSource("chat.rs")).toContain("tokio::spawn(async move");
    expect(selectedAppSource("report.rs")).toContain("tokio::spawn(async move");
    expect(existsSync(path.join(repoRoot, "src-tauri/src/migrations.rs"))).toBe(true);
    expect(existsSync(path.join(repoRoot, "src-tauri/migrations"))).toBe(true);
    expect(existsSync(path.join(analysisCrateRoot, "migrations.rs"))).toBe(false);
    expect(existsSync(path.join(analysisCrateRoot, "fixtures.rs"))).toBe(false);
    for (const fixture of [
      "fixtures.rs",
      "fixtures/seed.rs",
      "fixtures/seed/runs.rs",
    ]) expect(existsSync(path.join(appAnalysisRoot, fixture))).toBe(true);
  });

  it("keeps the event adapter synchronous bounded and nonblocking", () => {
    const events = selectedAppSource("events.rs");
    const implementationMarker =
      "impl AnalysisEventSink for TauriAnalysisEventSink";
    const marker = events.indexOf(implementationMarker);
    expect(marker).toBeGreaterThanOrEqual(0);
    const open = events.indexOf("{", marker);
    const close = closingDelimiter(
      maskRustNonCode(events, true),
      open,
      "{",
      "}",
    );
    const implementation = events.slice(marker, close + 1);
    const adapterShape = (source: string) =>
      rustFunctions(source).map(({ name, visibility, signature, body }) => ({
        name,
        visibility,
        signature: canonicalRustSignature(signature),
        body: normalized(maskRustNonCode(body, false)),
      }));
    const expectedAdapterShape = [
      {
        name: "publish_run",
        visibility: "",
        signature: canonicalRustSignature(
          "fn publish_run(&self, event: AnalysisRunEvent)",
        ),
        body: "let _ = self.handle.emit(ANALYSIS_RUN_EVENT, &event);",
      },
      {
        name: "publish_chat",
        visibility: "",
        signature: canonicalRustSignature(
          "fn publish_chat(&self, event: AnalysisChatEvent)",
        ),
        body: "let _ = self.handle.emit(ANALYSIS_CHAT_EVENT, &event);",
      },
    ];
    expect(
      adapterShape(implementation),
      "the app event adapter must contain only the two bounded emit methods",
    ).toEqual(expectedAdapterShape);
    expect(
      adapterShape(implementation.replace(
        "let _ = self.handle.emit(ANALYSIS_RUN_EVENT, &event);",
        "audit_event(&event); "
        + "let _ = self.handle.emit(ANALYSIS_RUN_EVENT, &event);",
      )),
      "an extra adapter statement mutates the exact bounded shape",
    ).not.toEqual(expectedAdapterShape);
    expect(
      normalized(maskRustNonCode(implementation, false)),
      "the adapter impl must not hide extra statements or associated items",
    ).toBe(
      "impl AnalysisEventSink for TauriAnalysisEventSink { "
      + "fn publish_run(&self, event: AnalysisRunEvent) { "
      + "let _ = self.handle.emit(ANALYSIS_RUN_EVENT, &event); } "
      + "fn publish_chat(&self, event: AnalysisChatEvent) { "
      + "let _ = self.handle.emit(ANALYSIS_CHAT_EVENT, &event); } }",
    );
  });

  it("keeps the separately green migration fixture contract", () => {
    const fixtureContract = read(
      "src/lib/analysis-migration-fixture-contract.test.ts",
    );
    for (const title of [
      "requires exactly one analysis fixture owner",
      "rejects duplicate and malformed migration fixture syntax",
      "parses the non-Apalis registry prefix fail closed",
    ]) expect(fixtureContract).toContain(`it("${title}"`);

    const appFixture =
      "src-tauri/src/analysis/test_schema.rs";
    const crateFixture =
      "src-tauri/crates/extractum-analysis/src/test_schema.rs";
    expect(
      [appFixture, crateFixture].filter((relative) =>
        existsSync(path.join(repoRoot, relative))),
    ).toEqual([extracted ? crateFixture : appFixture]);
    const fixture = read(extracted ? crateFixture : appFixture);
    expect(fixture).toMatch(
      /const ANALYSIS_TEST_MIGRATIONS:\s*\[\(&str,\s*&str\);\s*12\]/,
    );
    expect(fixture).toContain(
      extracted
        ? 'include_str!("../../../migrations/'
        : 'include_str!("../../migrations/',
    );
    expect(fixture).not.toContain(
      extracted
        ? 'include_str!("../../migrations/'
        : 'include_str!("../../../migrations/',
    );
    expect(fixture).toContain("sqlx::raw_sql(sql)");
    expect(fixture).toContain(".execute(&mut *transaction)");
    expect(fixture).toContain("transaction\n        .commit()");
    const ownerRoot = extracted
      ? read("src-tauri/crates/extractum-analysis/src/lib.rs")
      : read("src-tauri/src/analysis/mod.rs");
    expect(ownerRoot).toMatch(/#\[cfg\(test\)\]\s*mod test_schema;/);
    expect(ownerRoot).not.toMatch(/pub(?:\([^)]*\))?\s+mod test_schema;/);
  });

  it("keeps trace compression core-owned and the app free of direct zstd", () => {
    const appRust = rustFiles(path.join(repoRoot, "src-tauri/src"))
      .map((file) => normalizeNewlines(readFileSync(file, "utf8")))
      .join("\n");
    expect(maskRustNonCode(appRust, true)).not.toMatch(
      /\b(?:extern\s+crate\s+zstd|use\s+(?:::)?zstd\b|zstd\s*::)/,
    );
    const trace = selectedMoveSource("trace.rs");
    expect(trace).toMatch(
      /extractum_core::[\s\S]*compression::\{compress_json_bytes,\s*decompress_bytes\}/,
    );
    expect(maskRustNonCode(trace, true)).not.toMatch(/\bzstd\s*::/);
    const coreCompression = read(
      "src-tauri/crates/extractum-core/src/compression.rs",
    );
    expect(coreCompression).toMatch(/\bzstd\s*::/);
    expect(
      tomlSection(
        read("src-tauri/crates/extractum-core/Cargo.toml"),
        "dependencies",
      ),
    ).toMatch(/^zstd\.workspace\s*=\s*true$/m);
    const appDependencies = tomlSection(
      read("src-tauri/Cargo.toml"),
      "dependencies",
    );
    const rootManifest = read("src-tauri/Cargo.toml");
    const workspaceZstdKeys = new Set(
      tomlDependencies(rootManifest)
        .filter(({ section, packageName }) =>
          section === "workspace.dependencies" && packageName === "zstd")
        .map(({ key }) => key),
    );
    const resolvedAppZstdEdges = tomlDependencies(rootManifest)
      .filter((dependency) =>
        dependency.section !== "workspace.dependencies"
        && (
          dependency.packageName === "zstd"
          || (
            dependency.workspace
            && workspaceZstdKeys.has(dependency.key)
          )
        ));
    if (extracted) {
      expect(appDependencies).not.toMatch(/^zstd(?:\.workspace)?\s*=/m);
      expect(
        resolvedAppZstdEdges,
        "post-move app has no normal/dev/build/target/aliased zstd edge",
      ).toEqual([]);
    } else {
      expect(appDependencies).toMatch(/^zstd\s*=\s*\{\s*workspace\s*=\s*true\s*\}$/m);
      expect(resolvedAppZstdEdges).toEqual([{
        section: "dependencies",
        key: "zstd",
        packageName: "zstd",
        path: undefined,
        workspace: true,
        value: '{ workspace = true }',
      }]);
    }
  });

  it("keeps command event and AppError wire contracts unchanged", () => {
    const definitions = commandFunctions();
    const expectedNames = Object.keys(commandWireContracts).sort();
    expect(expectedNames).toEqual(
      [...commands.analysis, ...commands.project, ...commands.dev].sort(),
    );
    const contextParameters = new Set([
      "handle",
      "state",
      "scheduler",
      "repair_state",
    ]);
    for (const [name, [expectedSignature, expectedWire]] of Object.entries(
      commandWireContracts,
    )) {
      const matches = definitions.filter((fn) => fn.name === name);
      expect(matches, name).toHaveLength(1);
      const fn = matches[0];
      const actualSignature = `${normalized(fn.params)} ${normalized(fn.result)}`;
      expect(
        canonicalCommandSignature(actualSignature),
        `${name} command signature`,
      ).toBe(canonicalCommandSignature(expectedSignature));
      const wire = splitTopLevel(fn.params)
        .map((parameter) => parameter.slice(0, parameter.indexOf(":")).trim())
        .filter((parameter) => !contextParameters.has(parameter))
        .map(camelCase);
      expect(wire, `${name} camelCase parameters`).toEqual(expectedWire);
    }

    const models = selectedMoveSource("models.rs");
    const moduleSource = selectedAppSource("mod.rs");
    const chat = selectedMoveSource("chat_engine.rs");
    const report = selectedMoveSource("report_engine.rs");
    const requests = selectedMoveSource("report/requests.rs");
    const phases = selectedMoveSource("report/phases.rs");
    const lifecycle = selectedMoveSource("report/lifecycle_portable.rs");
    const wireWitness = selectedAppSource("tests_application.rs");
    expect(moduleSource).toContain('const ANALYSIS_RUN_EVENT: &str = "analysis://run"');
    expect(moduleSource).toContain('const ANALYSIS_CHAT_EVENT: &str = "analysis://chat"');
    const exactWireLayouts: Record<
      string,
      Array<[visibility: string, name: string, type: string]>
    > = {
      AnalysisRunEvent: [
        ["pub", "run_id", "i64"],
        ["pub", "request_id", "Option<String>"],
        ["pub", "kind", "String"],
        ["pub", "phase", "String"],
        ["pub", "queue_position", "Option<usize>"],
        ["pub", "message", "Option<String>"],
        ["pub", "progress_current", "Option<i64>"],
        ["pub", "progress_total", "Option<i64>"],
        ["pub", "delta", "Option<String>"],
        ["pub", "chunk_summary", "Option<AnalysisChunkSummaryEvent>"],
        ["pub", "error", "Option<String>"],
      ],
      AnalysisChunkSummaryEvent: [
        ["pub", "index", "i64"],
        ["pub", "total", "i64"],
        ["pub", "message_count", "i64"],
        ["pub", "summary", "String"],
        ["pub", "topics", "Vec<String>"],
        ["pub", "notable_points", "Vec<String>"],
        ["pub", "candidate_refs", "Vec<String>"],
      ],
      AnalysisChatEvent: [
        ["pub", "request_id", "String"],
        ["pub", "run_id", "i64"],
        ["pub", "kind", "String"],
        ["pub", "queue_position", "Option<usize>"],
        ["pub", "delta", "Option<String>"],
        ["pub", "message", "Option<String>"],
        ["pub", "error", "Option<String>"],
      ],
    };
    for (const [typeName, expected] of Object.entries(exactWireLayouts)) {
      expect(
        structFields(models, typeName)
          .map(({ visibility, name, type }) => [visibility, name, type]),
        `${typeName} exact public wire field order and types`,
      ).toEqual(expected);
      expect(
        declarationAttributes(models, "struct", typeName),
        `${typeName} exact serialization derives`,
      ).toBe("#[derive(Serialize)]");
      expect(
        structBody(models, typeName),
        `${typeName} must not acquire field-level serde policy`,
      ).not.toContain("#[");
    }
    const mutatedRunEvent = models.replace(
      "    pub chunk_summary: Option<AnalysisChunkSummaryEvent>,\n"
      + "    pub error: Option<String>,",
      "    pub chunk_summary: Option<AnalysisChunkSummaryEvent>,\n"
      + "    pub mutation_sentinel: bool,\n"
      + "    pub error: Option<String>,",
    );
    expect(
      structFields(mutatedRunEvent, "AnalysisRunEvent")
        .map(({ visibility, name, type }) => [visibility, name, type]),
      "an added event field mutates the exact wire layout",
    ).not.toEqual(exactWireLayouts.AnalysisRunEvent);
    expect(models).not.toContain("skip_serializing_if");
    expect(requests).toContain(
      'format!("analysis-map-{run_id}-{chunk_index}-{}", now_secs())',
    );
    expect(requests).toContain(
      'format!("analysis-reduce-{}-{}", params.run_id, now_secs())',
    );
    expect(chat).toContain(
      'format!("analysis-chat-{}-{}", params.run.id, now_secs())',
    );
    expect(wireWitness).toContain(
      "fn analysis_wire_values_serialize_to_exact_json_objects()",
    );
    expect(wireWitness.match(/serde_json::to_value\(/g) ?? []).toHaveLength(3);
    expect(wireWitness).toContain(
      'json!({"kind": "conflict", "message": "wire failure"})',
    );
    expectOrdered(chat, [
      'ChatEvent::new(queued_request_id.clone(), run_id, "queued")',
      'ChatEvent::new(started_request_id, run_id, "started")',
      'ChatEvent::new(delta_request_id.clone(), run_id, "delta")',
      'ChatEvent::new(completion.request_id, completion.run_id, "completed")',
    ]);
    expectOrdered(report, [
      'RunEvent::new(run_id, "started", "load_items")',
      'RunEvent::new(run_id, "progress", "chunking")',
      'RunEvent::new(run_id, "progress", "persist")',
      'RunEvent::new(run_id, "completed", "persist")',
    ]);
    expectOrdered(phases, [
      'RunEvent::new(run_id, "queued", "map")',
      'RunEvent::new(run_id, "started", "map")',
      'RunEvent::new(run_id, "progress", "map")',
    ]);
    expectOrdered(phases, [
      'RunEvent::new(run_id, "queued", "reduce")',
      'RunEvent::new(run_id, "started", "reduce")',
      'RunEvent::new(run_id, "delta", "reduce")',
    ]);
    expect(lifecycle).toContain('RunEvent::new(run_id, "failed", "persist")');
    expect(report).toContain('RunEvent::new(run_id, "cancelled", "persist")');
    const errorSource = read(
      "src-tauri/crates/extractum-core/src/error.rs",
    );
    expect(
      splitTopLevel(enumBody(errorSource, "AppErrorKind")),
      "AppErrorKind exact wire variants and order",
    ).toEqual([
      "Validation",
      "NotFound",
      "Auth",
      "Network",
      "Conflict",
      "Internal",
    ]);
    expect(
      declarationAttributes(errorSource, "enum", "AppErrorKind"),
      "AppErrorKind exact derives and serde rename policy",
    ).toBe(
      '#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)] #[serde(rename_all = "snake_case")]',
    );
    expect(
      splitTopLevel(enumBody(
        errorSource.replace("    Internal,\n}", "    Internal,\n    Mutated,\n}"),
        "AppErrorKind",
      )),
      "an added AppErrorKind variant mutates the exact wire enum",
    ).not.toEqual([
      "Validation",
      "NotFound",
      "Auth",
      "Network",
      "Conflict",
      "Internal",
    ]);
    expect(
      structFields(errorSource, "AppError")
        .map(({ visibility, name, type }) => [visibility, name, type]),
      "AppError exact public wire field order and types",
    ).toEqual([
      ["pub", "kind", "AppErrorKind"],
      ["pub", "message", "String"],
    ]);
    expect(
      declarationAttributes(errorSource, "struct", "AppError"),
      "AppError exact derives",
    ).toBe(
      "#[derive(Debug, Clone, Serialize, PartialEq, Eq)]",
    );
    expect(
      read("src/lib/analysis-application-contract.test.ts"),
    ).toContain(
      'it("analysis_wire_contract_serializes_commands_events_and_errors_unchanged"',
    );
  });

  it("moves only pre-normalized portable sources", () => {
    assertFrozenTopology();
    const forbidden = [
      /\bcrate\s*::\s*analysis\b/,
      /\bcrate\s*::\s*error\b/,
      /\bcrate\s*::\s*compression\b/,
      /\bcrate\s*::\s*llm\b/,
      /\bcrate\s*::\s*time\b/,
      /\bcrate\s*::\s*db\b/,
      /\bsources\s*::\s*test_support\b/,
      /\b(?:tauri|Tauri)\b/,
      /\bAppHandle\b/,
      /\bget_pool\b/,
    ];
    const allowedSiblingProbe = maskRustNonCode(
      "use crate::models::AnalysisRun; // crate::analysis is a decoy",
      true,
    );
    for (const pattern of forbidden) {
      expect(
        allowedSiblingProbe,
        `portable sibling crate::models must not trigger ${pattern}`,
      ).not.toMatch(pattern);
    }
    expect(
      maskRustNonCode("use crate::analysis::models::AnalysisRun;", true),
      "one of the exact six app-root prefixes is rejected",
    ).toMatch(forbidden[0]);
    expect(
      maskRustNonCode("use crate::sources::test_support::create_pool;", true),
      "sources::test_support is rejected without banning all crate siblings",
    ).toMatch(forbidden[6]);
    for (const move of allMoves) {
      const source = selectedMoveSource(move.before);
      const executable = maskRustNonCode(source, true);
      for (const pattern of forbidden) {
        expect(
          executable,
          `${move.before} contains app-root capability ${pattern}`,
        ).not.toMatch(pattern);
      }
    }
    const ownedReadModel = selectedMoveSource("store/owned_read_model.rs");
    expect(ownedReadModel).toMatch(
      /use\s+extractum_core(?:\s*::time::|\s*::\s*\{[\s\S]*?\btime::)ymd_to_unix_midnight\s*(?:;|,)/,
    );
    if (extracted) {
      const architecture = selectedMoveSource(
        "report/tests/architecture.rs",
      );
      expect(architecture).toContain('include_str!("../../report.rs")');
      expect(architecture).not.toContain(
        'include_str!("../../report_engine.rs")',
      );
    }
  });

  it("removes Phase 7 extraction-only dead code and unused facade surface", () => {
    const forbiddenByFile = new Map<string, readonly string[]>([
      [
        "src-tauri/crates/extractum-analysis/src/models.rs",
        ["AnalysisSourceGroupRow"],
      ],
      [
        "src-tauri/crates/extractum-analysis/src/store/read_model.rs",
        ["load_analysis_run_trace_data"],
      ],
      ["src-tauri/src/lib.rs", ["internal_error"]],
      [
        "src-tauri/src/sources/mod.rs",
        [
          "ANALYSIS_TELEGRAM_HISTORY_SCOPE_CURRENT",
          "ANALYSIS_TELEGRAM_HISTORY_SCOPE_CURRENT_PLUS_MIGRATED",
          "TELEGRAM_SOURCE_TYPE",
        ],
      ],
      [
        "src-tauri/src/sources/types.rs",
        [
          "ANALYSIS_TELEGRAM_HISTORY_SCOPE_CURRENT",
          "ANALYSIS_TELEGRAM_HISTORY_SCOPE_CURRENT_PLUS_MIGRATED",
        ],
      ],
      [
        "src-tauri/src/llm/mod.rs",
        [
          "resolve_model_input_token_limit_for_backend",
          "resolve_model_output_token_limit_for_backend",
          "run_llm_collect_with_profile",
          "LlmCompletion",
          "LlmRequestSnapshotState",
        ],
      ],
    ]);

    for (const [relativePath, identifiers] of forbiddenByFile) {
      const source = read(relativePath);
      for (const identifier of identifiers) {
        expect(
          source,
          `${relativePath} retains extraction-only ${identifier}`,
        ).not.toMatch(new RegExp(`\\b${identifier}\\b`));
      }
    }
  });
});
