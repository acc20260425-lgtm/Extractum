import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(fileURLToPath(new URL("..", import.meta.url)));
const planPath = path.join(
  repoRoot,
  "docs/superpowers/plans/2026-07-28-extractum-telegram-8b-preparation.md",
);
const artifactPath = path.join(repoRoot, "src/lib/telegram-8b-symbol-map.json");
const rustPath = /^(?:<new>|[A-Za-z0-9_]+(?:\/[A-Za-z0-9_]+)*\.rs)$/;
const rustSymbol = /^[A-Za-z_][A-Za-z0-9_]*(?:::[A-Za-z_][A-Za-z0-9_]*)*$/;
/** @type {Set<string>} */
const semanticOwners = new Set([
  "app",
  "staged",
  "staged-internal",
  "transitional",
]);
/** @type {Set<string>} */
const dispositions = new Set([
  "absorb-and-rewrite",
  "add-is-member-field",
  "delete",
  "move",
  "move-or-rename",
  "move-restricted-fields",
  "move-then-delete",
  "new",
  "new-private-test-seam",
  "new-restricted-bridge",
  "new-restricted-facade",
  "new-restricted-leaf",
  "new-then-delete",
  "redirect-compat-reexport",
  "replace",
  "replace-compat-reexport",
  "replace-fields-with-owned-values",
  "replace-module-root",
  "replace-with-transitional-copy",
  "retain-compat-reexport",
  "retain-then-delete",
  "rewrite-owned-query",
  "rewrite-owned-seam",
  "rewrite-owned-values",
  "split-app-command",
  "split-app-coordinator",
  "split-app-dispatch",
  "split-app-policy",
  "split-app-projection",
  "split-stage",
  "split-stage-attempt-value",
  "split-stage-count",
  "split-stage-facade",
  "split-stage-iterator",
  "split-stage-page",
  "split-stage-raw-invoke",
  "split-stage-search-count",
  "split-stage-search-page",
  "split-stage-tracker",
]);
const transitionLabels = [
  "CP3 raw handle callsites",
  "CP4 raw handle callsites",
  "CP4 ResolvedSyncPeer::peer consumers",
  "CP5 raw handle callsites",
  "CP5 ResolvedSyncPeer::peer consumers",
  "CP7 raw bridge symbols/callsites",
];
const transitionKeys = [
  "cp3RawHandleCallsites",
  "cp4RawHandleCallsites",
  "cp4ResolvedSyncPeerPeerConsumers",
  "cp5RawHandleCallsites",
  "cp5ResolvedSyncPeerPeerConsumers",
  "cp7RawBridgeSymbolsAndCallsites",
];
const expectedTransitionCounts = [6, 4, 3, 3, 2, 0];

/**
 * @typedef {{ path: string, symbol: string }} FinalTarget
 * @typedef {{
 *   currentPath: string,
 *   currentSymbol: string,
 *   finalTargets: FinalTarget[],
 *   semanticOwner: string,
 *   firstCheckpoint: "existing" | number,
 *   removalCheckpoint: "retained" | number,
 *   disposition: string,
 *   currentAnchors?: string[],
 * }} SymbolDispositionRow
 * @typedef {{ normalizedLfBytes: number, sha256: string }} ContentAddress
 */

/**
 * @param {string} message
 * @returns {never}
 */
function fail(message) {
  throw new Error(`Telegram Phase 8B symbol authority: ${message}`);
}

/**
 * @param {string} source
 * @returns {string}
 */
function normalize(source) {
  if (typeof source !== "string") fail("plan source is not text");
  return source.replace(/\r\n?/g, "\n");
}

/**
 * @param {string} source
 * @returns {ContentAddress}
 */
function contentAddress(source) {
  return {
    normalizedLfBytes: Buffer.byteLength(source, "utf8"),
    sha256: createHash("sha256").update(source, "utf8").digest("hex"),
  };
}

/**
 * @param {string} source
 * @param {string} startHeading
 * @param {string} endMarker
 * @param {string} label
 * @returns {string}
 */
function exactSection(source, startHeading, endMarker, label) {
  const normalized = normalize(source);
  if (
    normalized.split(`${startHeading}\n`).length - 1 !== 1
    || !(
      normalized.startsWith(`${startHeading}\n`)
      || normalized.includes(`\n${startHeading}\n`)
    )
  ) {
    fail(`malformed ${label} start heading`);
  }
  const start = normalized.indexOf(startHeading);
  const end = normalized.indexOf(endMarker, start + startHeading.length);
  if (start < 0 || end <= start) fail(`malformed ${label} end marker`);
  return normalized.slice(start, end);
}

/**
 * @param {string} source
 * @param {string} marker
 * @param {string} label
 * @returns {string}
 */
function fencedTextAfter(source, marker, label) {
  const normalized = normalize(source);
  const markerIndexes = [];
  for (
    let index = normalized.indexOf(marker);
    index >= 0;
    index = normalized.indexOf(marker, index + marker.length)
  ) {
    markerIndexes.push(index);
  }
  if (markerIndexes.length !== 1) fail(`expected one ${label} marker`);
  const fenceStart = normalized.indexOf("\n```text\n", markerIndexes[0]);
  if (fenceStart < 0) fail(`missing ${label} text fence`);
  const contentStart = fenceStart + "\n```text\n".length;
  const fenceEnd = normalized.indexOf("\n```", contentStart);
  if (fenceEnd < 0) fail(`unterminated ${label} text fence`);
  return normalized.slice(contentStart, fenceEnd);
}

/**
 * @param {string} cell
 * @param {string} label
 * @returns {string}
 */
function codeCell(cell, label) {
  const match = /^`([^`]+)`$/.exec(cell);
  if (!match) fail(`malformed ${label} cell: ${cell}`);
  return match[1];
}

/**
 * @param {string} value
 * @param {string} label
 * @param {RegExp} [validator]
 * @returns {string[]}
 */
function expandBraceValue(value, label, validator = rustSymbol) {
  if (/[*]|\.{3}|all helpers|as needed/i.test(value)) {
    fail(`wildcard or catch-all in ${label}: ${value}`);
  }
  const openCount = [...value].filter((character) => character === "{").length;
  const closeCount = [...value].filter((character) => character === "}").length;
  if (openCount === 0 && closeCount === 0) {
    if (!validator.test(value)) fail(`malformed ${label}: ${value}`);
    return [value];
  }
  if (
    openCount !== 1
    || closeCount !== 1
    || !value.startsWith("{")
    || !value.endsWith("}")
  ) {
    fail(`nested or malformed brace group in ${label}: ${value}`);
  }
  const items = value.slice(1, -1).split(",");
  if (
    items.length < 2
    || items.some((item) => item.trim() !== item || !validator.test(item))
  ) {
    fail(`malformed brace items in ${label}: ${value}`);
  }
  if (new Set(items).size !== items.length) {
    fail(`duplicate brace items in ${label}: ${value}`);
  }
  return items;
}

/**
 * @param {string[]} paths
 * @param {string[]} symbols
 * @param {string} currentSymbol
 * @param {string} label
 * @returns {FinalTarget[]}
 */
function targetPairs(paths, symbols, currentSymbol, label) {
  const targetSymbols =
    symbols.length === 1 && symbols[0] === "=" ? [currentSymbol] : symbols;
  if (targetSymbols.some((symbol) => symbol === "=" || !rustSymbol.test(symbol))) {
    fail(`malformed final target symbols in ${label}`);
  }
  if (paths.length !== 1 && paths.length !== targetSymbols.length) {
    fail(`final path/symbol cardinality mismatch in ${label}`);
  }
  return targetSymbols.map((symbol, index) => ({
    path: paths.length === 1 ? paths[0] : paths[index],
    symbol,
  }));
}

/**
 * @param {string} source
 * @returns {SymbolDispositionRow[]}
 */
function parseDispositionRows(source) {
  const section = exactSection(
    source,
    "### Literal Machine-Checked Production-Symbol Disposition",
    "\nThe `raw_memory_session` and `clone_memory_session` rows",
    "production-symbol disposition",
  );
  const lines = section.split("\n");
  const header =
    "| Current path | Current exact symbol(s) | Final path | Final exact symbol(s) | Semantic owner | First checkpoint | Removal checkpoint | Disposition |";
  const separator =
    "| --- | --- | --- | --- | --- | ---: | --- | --- |";
  const headerIndexes = lines.flatMap((line, index) =>
    line === header ? [index] : []
  );
  if (
    headerIndexes.length !== 1
    || lines[headerIndexes[0] + 1] !== separator
  ) {
    fail("malformed production-symbol disposition table heading");
  }
  const tableLines = lines.slice(headerIndexes[0] + 2);
  const terminalBlank = tableLines.indexOf("");
  if (
    terminalBlank < 0
    || terminalBlank !== tableLines.length - 1
  ) {
    fail("disposition table tail is malformed or nonempty");
  }
  const rawRows = tableLines.slice(0, terminalBlank);
  for (const line of rawRows) {
    if (!line.startsWith("| ")) fail(`malformed disposition row: ${line}`);
  }
  if (rawRows.length !== 99) {
    fail(`expected 99 disposition rows, found ${rawRows.length}`);
  }

  const rows = rawRows.flatMap((line, rowIndex) => {
    const cells = line.slice(2, -2).split(" | ");
    if (cells.length !== 8) fail(`malformed disposition cells: ${line}`);
    const currentPath = codeCell(cells[0], "current path");
    const currentValue = codeCell(cells[1], "current symbol");
    const finalPathValue = codeCell(cells[2], "final path");
    const finalValue = codeCell(cells[3], "final symbol");
    const semanticOwner = cells[4];
    const firstCheckpointCell = cells[5];
    const removalCheckpointCell = cells[6];
    if (
      !(
        firstCheckpointCell === "existing"
        || /^[3-7]$/.test(firstCheckpointCell)
      )
      || !(
        removalCheckpointCell === "retained"
        || /^[3-7]$/.test(removalCheckpointCell)
      )
    ) {
      fail(`invalid checkpoint in disposition row ${rowIndex + 1}`);
    }
    /** @type {"existing" | number} */
    const firstCheckpoint =
      firstCheckpointCell === "existing"
        ? "existing"
        : Number(firstCheckpointCell);
    /** @type {"retained" | number} */
    const removalCheckpoint =
      removalCheckpointCell === "retained"
        ? "retained"
        : Number(removalCheckpointCell);
    const disposition = cells[7];
    if (!rustPath.test(currentPath)) fail(`malformed current path: ${currentPath}`);
    const finalPaths = expandBraceValue(finalPathValue, "final path", rustPath);
    if (finalPaths.some((targetPath) => !rustPath.test(targetPath) || targetPath === "<new>")) {
      fail(`malformed final path: ${finalPathValue}`);
    }
    if (!semanticOwners.has(semanticOwner)) {
      fail(`unknown semantic owner: ${semanticOwner}`);
    }
    if (!dispositions.has(disposition)) {
      fail(`unknown disposition: ${disposition}`);
    }
    if (
      !(
        firstCheckpoint === "existing"
        || (Number.isInteger(firstCheckpoint)
          && firstCheckpoint >= 3
          && firstCheckpoint <= 7)
      )
      || !(
        removalCheckpoint === "retained"
        || (Number.isInteger(removalCheckpoint)
          && removalCheckpoint >= 3
          && removalCheckpoint <= 7)
      )
    ) {
      fail(`invalid checkpoint in disposition row ${rowIndex + 1}`);
    }
    if (
      typeof firstCheckpoint === "number"
      && typeof removalCheckpoint === "number"
      && removalCheckpoint < firstCheckpoint
    ) {
      fail(`removal precedes first checkpoint in row ${rowIndex + 1}`);
    }
    if (semanticOwner === "transitional" && removalCheckpoint === "retained") {
      fail(`transitional symbol requires finite removal in row ${rowIndex + 1}`);
    }
    if (disposition.includes("delete") && removalCheckpoint === "retained") {
      fail(`delete disposition requires finite removal in row ${rowIndex + 1}`);
    }

    const currentSymbols = expandBraceValue(currentValue, "current symbol");
    let finalSymbols;
    if (finalValue === "=") {
      finalSymbols = ["="];
    } else {
      finalSymbols = expandBraceValue(finalValue, "final symbol");
    }
    if (
      finalSymbols.length !== 1
      && finalSymbols.length !== currentSymbols.length
      && currentSymbols.length !== 1
    ) {
      fail(`current/final symbol cardinality mismatch in row ${rowIndex + 1}`);
    }

    return currentSymbols.map((currentSymbol, currentIndex) => {
      let selectedFinalSymbols;
      let selectedFinalPaths = finalPaths;
      if (finalSymbols.length === 1) {
        selectedFinalSymbols = finalSymbols;
      } else if (currentSymbols.length === finalSymbols.length) {
        selectedFinalSymbols = [finalSymbols[currentIndex]];
        if (finalPaths.length === currentSymbols.length) {
          selectedFinalPaths = [finalPaths[currentIndex]];
        }
      } else {
        selectedFinalSymbols = finalSymbols;
      }
      return {
        currentPath,
        currentSymbol,
        finalTargets: targetPairs(
          selectedFinalPaths,
          selectedFinalSymbols,
          currentSymbol,
          `row ${rowIndex + 1}`,
        ),
        semanticOwner,
        firstCheckpoint,
        removalCheckpoint,
        disposition,
      };
    });
  });
  const tuples = rows.map(
    ({ currentPath, currentSymbol, disposition }) =>
      `${currentPath}\0${currentSymbol}\0${disposition}`,
  );
  if (new Set(tuples).size !== tuples.length) {
    fail("duplicate current path/symbol/disposition tuple");
  }
  if (rows.some(({ finalTargets }) => finalTargets.length === 0)) {
    fail("disposition row has no final target");
  }
  return rows;
}

/**
 * @param {string} line
 * @param {string} label
 * @returns {string[]}
 */
function expandQualifiedFenceLine(line, label) {
  if (/[*]|\.{3}|all helpers|as needed/i.test(line)) {
    fail(`wildcard or catch-all in ${label}: ${line}`);
  }
  const openCount = [...line].filter((character) => character === "{").length;
  const closeCount = [...line].filter((character) => character === "}").length;
  if (openCount === 0 && closeCount === 0) {
    if (!rustSymbol.test(line)) fail(`malformed ${label}: ${line}`);
    return [line];
  }
  if (openCount !== 1 || closeCount !== 1) {
    fail(`nested brace group in ${label}: ${line}`);
  }
  const match = /^([A-Za-z_][A-Za-z0-9_]*(?:::[A-Za-z_][A-Za-z0-9_]*)*::)\{([^{}]+)\}$/.exec(
    line,
  );
  if (!match) fail(`malformed single-level brace group in ${label}: ${line}`);
  const items = match[2].split(",");
  if (
    items.length < 2
    || items.some((item) => item.trim() !== item || !rustSymbol.test(item))
  ) {
    fail(`malformed brace items in ${label}: ${line}`);
  }
  return items.map((item) => `${match[1]}${item}`);
}

/**
 * @param {string} source
 * @returns {string[]}
 */
function parseRestrictedSymbols(source) {
  const fence = fencedTextAfter(
    source,
    "The exact restricted internal bridge allowlist is below.",
    "restricted bridge allowlist",
  );
  const lines = fence.split("\n");
  if (lines.some((line) => !line.trim())) {
    fail("restricted bridge allowlist contains an empty line");
  }
  const symbols = lines.flatMap((line) =>
    expandQualifiedFenceLine(line, "restricted bridge")
  ).sort();
  if (symbols.length !== 69) {
    fail(`expected 69 restricted symbols, found ${symbols.length}`);
  }
  if (new Set(symbols).size !== symbols.length) {
    fail("restricted bridge allowlist contains duplicates");
  }
  return symbols;
}

/**
 * @param {string} source
 * @returns {string[]}
 */
function parsePublicSymbols(source) {
  const fence = fencedTextAfter(
    source,
    "The root re-export allowlist is exactly:",
    "root public allowlist",
  );
  const symbols = fence.split("\n");
  if (
    symbols.length !== 29
    || symbols.some((symbol) => !rustSymbol.test(symbol))
    || new Set(symbols).size !== symbols.length
  ) {
    fail("root public allowlist is malformed or duplicated");
  }
  return [...symbols].sort();
}

/**
 * @param {string} source
 * @returns {Record<string, string[]>}
 */
function parseTransitionInventories(source) {
  const fence = fencedTextAfter(
    source,
    "The generator also freezes three transition inventories that are not inferred\nfrom path existence:",
    "transition inventory",
  );
  /** @type {Map<string, string[]>} */
  const inventories = new Map();
  /** @type {string | undefined} */
  let activeLabel;
  for (const line of fence.split("\n")) {
    const inlineEmpty = /^(.*): empty$/.exec(line);
    if (inlineEmpty) {
      if (
        !transitionLabels.includes(inlineEmpty[1])
        || inventories.has(inlineEmpty[1])
      ) {
        fail(`unknown or duplicate transition inventory: ${inlineEmpty[1]}`);
      }
      inventories.set(inlineEmpty[1], []);
      activeLabel = undefined;
      continue;
    }
    const heading = /^([^ ].*):$/.exec(line);
    if (heading) {
      if (!transitionLabels.includes(heading[1]) || inventories.has(heading[1])) {
        fail(`unknown or duplicate transition inventory: ${heading[1]}`);
      }
      activeLabel = heading[1];
      inventories.set(activeLabel, []);
      continue;
    }
    if (!activeLabel || !line.startsWith("  ")) {
      fail(`malformed transition inventory line: ${line}`);
    }
    const values = inventories.get(activeLabel);
    if (!values) fail(`missing transition inventory: ${activeLabel}`);
    values.push(
      ...expandQualifiedFenceLine(line.slice(2), activeLabel),
    );
  }
  if (
    inventories.size !== transitionLabels.length
    || transitionLabels.some((label) => !inventories.has(label))
  ) {
    fail("transition inventory set drifted");
  }
  return Object.fromEntries(
    transitionLabels.map((label, index) => {
      const inventory = inventories.get(label);
      if (!inventory) fail(`missing transition inventory: ${label}`);
      const values = [...inventory].sort();
      if (
        values.length !== expectedTransitionCounts[index]
        || new Set(values).size !== values.length
      ) {
        fail(`transition inventory count or uniqueness drifted: ${label}`);
      }
      return [transitionKeys[index], values];
    }),
  );
}

/**
 * @param {string} source
 * @returns {Map<string, string[]>}
 */
function parseCurrentAnchors(source) {
  const fence = fencedTextAfter(
    source,
    "Synthetic `::segment` keys are source fragments, not Rust identifiers. Their\nJSON rows carry these exact pre-move anchors, each required in its enclosing\nfunction at CP1 and forbidden there after the removal checkpoint:",
    "synthetic current anchors",
  );
  /** @type {Map<string, string[]>} */
  const anchors = new Map();
  /** @type {string | undefined} */
  let activeSymbol;
  for (const line of fence.split("\n")) {
    const heading = /^([^ ].*):$/.exec(line);
    if (heading) {
      if (!rustSymbol.test(heading[1]) || !heading[1].endsWith("_segment")) {
        fail(`malformed synthetic fragment: ${heading[1]}`);
      }
      if (anchors.has(heading[1])) {
        fail(`duplicate synthetic fragment: ${heading[1]}`);
      }
      activeSymbol = heading[1];
      anchors.set(activeSymbol, []);
      continue;
    }
    if (!activeSymbol || !line.startsWith("  ") || !line.slice(2).trim()) {
      fail(`malformed synthetic anchor line: ${line}`);
    }
    const values = anchors.get(activeSymbol);
    if (!values) fail(`missing synthetic fragment: ${activeSymbol}`);
    values.push(line.slice(2));
  }
  if (anchors.size !== 5) fail(`expected five synthetic fragments, found ${anchors.size}`);
  for (const [symbol, values] of anchors) {
    if (values.length === 0 || new Set(values).size !== values.length) {
      fail(`empty or duplicate anchors for ${symbol}`);
    }
  }
  return anchors;
}

/**
 * @param {FinalTarget} target
 * @returns {string | undefined}
 */
function canonicalFinalSymbol(target) {
  if (!target.path.startsWith("telegram_impl/")) return undefined;
  const relative = target.path
    .slice("telegram_impl/".length)
    .replace(/\.rs$/, "")
    .replace(/\/mod$/, "");
  const module = relative === "lib" ? "" : relative.replaceAll("/", "::");
  return module ? `${module}::${target.symbol}` : target.symbol;
}

/**
 * @param {string} planSource
 * @returns {{
 *   schemaVersion: number,
 *   restrictedBridgeFenceAuthority: ContentAddress,
 *   symbols: SymbolDispositionRow[],
 *   transitionInventories: Record<string, string[]>,
 *   restrictedFinalSymbols: string[],
 * }}
 */
export function generateSymbolAuthority(planSource) {
  const rows = parseDispositionRows(planSource);
  const anchors = parseCurrentAnchors(planSource);
  for (const [symbol, currentAnchors] of anchors) {
    const matches = rows.filter(({ currentSymbol }) => currentSymbol === symbol);
    if (matches.length === 0) {
      fail(`synthetic fragment has no disposition row: ${symbol}`);
    }
    for (const match of matches) {
      match.currentAnchors = currentAnchors;
    }
  }
  const fragmentRows = rows.filter(({ currentSymbol }) =>
    currentSymbol.endsWith("_segment")
  );
  if (
    fragmentRows.length !== 6
    || fragmentRows.some(({ currentAnchors }) => !currentAnchors)
    || new Set(fragmentRows.map(({ currentSymbol }) => currentSymbol)).size
      !== anchors.size
  ) {
    fail("synthetic fragment disposition/anchor coverage drifted");
  }

  const restrictedFence = fencedTextAfter(
    planSource,
    "The exact restricted internal bridge allowlist is below.",
    "restricted bridge allowlist",
  );
  const restrictedFinalSymbols = parseRestrictedSymbols(planSource);
  const publicFinalSymbols = parsePublicSymbols(planSource);
  const finalSymbolSet = new Set(
    rows.flatMap(({ finalTargets }) =>
      finalTargets
        .map(canonicalFinalSymbol)
        .filter((symbol) => symbol !== undefined)
    ),
  );
  const missingRestricted = restrictedFinalSymbols.filter(
    (symbol) => !finalSymbolSet.has(symbol),
  );
  if (missingRestricted.length !== 0) {
    fail(
      `restricted symbols missing from final targets: ${missingRestricted.join(", ")}`,
    );
  }
  const publicSet = new Set(publicFinalSymbols);
  const publicRestrictedOverlap = restrictedFinalSymbols.filter((symbol) => {
    const leaf = symbol.slice(symbol.lastIndexOf("::") + 2);
    return publicSet.has(leaf);
  });
  if (publicRestrictedOverlap.length !== 0) {
    fail(
      `restricted symbols also occur in root public allowlist: ${publicRestrictedOverlap.join(", ")}`,
    );
  }

  return {
    schemaVersion: 1,
    restrictedBridgeFenceAuthority: contentAddress(restrictedFence),
    symbols: rows,
    transitionInventories: parseTransitionInventories(planSource),
    restrictedFinalSymbols,
  };
}

/**
 * @param {unknown} value
 * @returns {string}
 */
function serialize(value) {
  return `${JSON.stringify(value, null, 2)}\n`;
}

/**
 * @param {string} mode
 * @returns {void}
 */
function run(mode) {
  const rendered = serialize(
    generateSymbolAuthority(readFileSync(planPath, "utf8")),
  );
  if (mode === "--write") {
    writeFileSync(artifactPath, rendered, "utf8");
    return;
  }
  if (mode === "--check") {
    let existing;
    try {
      existing = readFileSync(artifactPath, "utf8");
    } catch (error) {
      fail(
        `artifact is missing or unreadable: ${
          error instanceof Error ? error.message : String(error)
        }`,
      );
    }
    if (existing !== rendered) {
      fail("artifact content or formatting drifted; run with --write");
    }
    return;
  }
  fail("expected exactly one argument: --write or --check");
}

const invokedPath = process.argv[1] ? path.resolve(process.argv[1]) : "";
if (invokedPath === fileURLToPath(import.meta.url)) {
  try {
    if (process.argv.length !== 3) {
      fail("expected exactly one argument: --write or --check");
    }
    run(process.argv[2]);
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exitCode = 1;
  }
}
