import { existsSync, readdirSync } from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";

import * as telegramContractPaths from "./telegram-contract-paths";

const repoRoot = path.resolve(import.meta.dirname, "../..");
const planPath =
  "docs/superpowers/plans/2026-07-26-extractum-telegram-8a-preparation.md";
const designPath =
  "docs/superpowers/specs/2026-07-26-telegram-crate-boundary-design.md";
const roadmapPath =
  "docs/superpowers/specs/2026-07-17-crate-roadmap.md";

type IdentityRow = {
  baselinePackage: string;
  baselineFullId: string;
  stagedPath: string;
  finalOwner: string;
  finalFullId: string;
  companionFinalIds: string[];
};

type AddedIdentity = {
  checkpoint: number;
  currentId: string;
  finalDisposition: string;
  finalId: string;
};

const normalize = telegramContractPaths.normalizeTelegramContractSourceText;
const plan = telegramContractPaths.readTelegramContractFile(planPath);
const design = telegramContractPaths.readTelegramContractFile(designPath);
const roadmap = telegramContractPaths.readTelegramContractFile(roadmapPath);

const directGrammersPaths = [
  "src-tauri/src/media.rs",
  "src-tauri/src/sources/avatar.rs",
  "src-tauri/src/sources/identity.rs",
  "src-tauri/src/sources/items.rs",
  "src-tauri/src/sources/peer_resolution.rs",
  "src-tauri/src/sources/sync.rs",
  "src-tauri/src/sources/topics.rs",
  "src-tauri/src/takeout_import/export_dc.rs",
  "src-tauri/src/takeout_import/forum_topics.rs",
  "src-tauri/src/takeout_import/mod.rs",
  "src-tauri/src/takeout_import/pagination.rs",
  "src-tauri/src/takeout_import/raw_parse.rs",
  "src-tauri/src/telegram.rs",
  "src-tauri/src/telegram_session_store.rs",
] as const;

const symbolDiscoveredPaths = [
  ...directGrammersPaths,
  "src-tauri/src/ingest_provenance.rs",
  "src-tauri/src/sources/mod.rs",
  "src-tauri/src/sources/store.rs",
  "src-tauri/src/sources/types.rs",
  "src-tauri/src/takeout_import/migrated_history.rs",
] as const;

const wiringPaths = ["src-tauri/src/lib.rs"] as const;
const ownershipMovePaths = [
  ...symbolDiscoveredPaths.filter(
    (relativePath) => relativePath !== "src-tauri/src/sources/store.rs",
  ),
  ...wiringPaths,
].sort();

const checkpointOneRawConsumerPaths = [
  ...directGrammersPaths,
  "src-tauri/src/sources/store.rs",
].sort();

const stagedRawConsumerPaths = [
  "src-tauri/src/telegram_impl/error.rs",
  "src-tauri/src/telegram_impl/live/avatar.rs",
  "src-tauri/src/telegram_impl/live/messages.rs",
  "src-tauri/src/telegram_impl/live/peer.rs",
  "src-tauri/src/telegram_impl/live/topics.rs",
  "src-tauri/src/telegram_impl/media.rs",
  "src-tauri/src/telegram_impl/runtime.rs",
  "src-tauri/src/telegram_impl/session.rs",
  "src-tauri/src/telegram_impl/takeout/export_dc.rs",
  "src-tauri/src/telegram_impl/takeout/forum_topics.rs",
  "src-tauri/src/telegram_impl/takeout/operations.rs",
  "src-tauri/src/telegram_impl/takeout/pagination.rs",
  "src-tauri/src/telegram_impl/takeout/raw_parse.rs",
  "src-tauri/src/telegram_impl/takeout/transport.rs",
] as const;

const crateRawConsumerPaths = stagedRawConsumerPaths.map((relativePath) =>
  relativePath.replace(
    "src-tauri/src/telegram_impl/",
    "src-tauri/crates/extractum-telegram/src/",
  ),
);

const baselineRootCounts = {
  error: 43,
  compression: 5,
  time: 2,
  sources: 46,
  ingest_provenance: 19,
  archive_read_model: 7,
  topic_memberships: 7,
  db: 6,
  youtube: 6,
  secret_store: 5,
  media: 4,
  source_ingest: 3,
  takeout_import: 3,
  telegram: 3,
  analysis_documents: 2,
  tx: 2,
  forum_topics: 1,
  readiness: 1,
  telegram_session_store: 1,
} as const;

const checkpointOneFacadeSites = [
  "src-tauri/src/ingest_provenance.rs|root|crate::error#1",
  "src-tauri/src/sources/avatar.rs|root|crate::error#1",
  "src-tauri/src/sources/identity.rs|root|crate::error#1",
  "src-tauri/src/sources/items.rs|root|crate::compression#1",
  "src-tauri/src/sources/items.rs|root|crate::error#1",
  "src-tauri/src/sources/items.rs|fn:insert_telegram_source_item_outcome|crate::error#1",
  "src-tauri/src/sources/items.rs|mod:tests|crate::compression#1",
  "src-tauri/src/sources/peer_resolution.rs|root|crate::compression#1",
  "src-tauri/src/sources/peer_resolution.rs|root|crate::error#1",
  "src-tauri/src/sources/peer_resolution.rs|mod:tests|crate::compression#1",
  "src-tauri/src/sources/peer_resolution.rs|mod:tests|crate::error#1",
  "src-tauri/src/sources/peer_resolution.rs|mod:tests/fn:typed_identity_rejects_subtype_peer_kind_mismatch|crate::error#1",
  "src-tauri/src/sources/sync.rs|root|crate::error#1",
  "src-tauri/src/sources/sync.rs|mod:tests/fn:sync_provider_rejects_manual_youtube_video_sources|crate::error#1",
  "src-tauri/src/sources/sync.rs|mod:tests/fn:finalize_sync_preserves_existing_legacy_metadata_blob|crate::compression#1",
  "src-tauri/src/sources/topics.rs|root|crate::error#1",
  "src-tauri/src/sources/types.rs|root|crate::time#1",
  "src-tauri/src/sources/types.rs|fn:from_source_subtype|crate::error#1",
  "src-tauri/src/sources/types.rs|fn:from_source_subtype|crate::error#2",
  "src-tauri/src/sources/types.rs|fn:parse|crate::error#1",
  "src-tauri/src/sources/types.rs|fn:encode_opaque|crate::error#1",
  "src-tauri/src/sources/types.rs|fn:encode_opaque|crate::error#2",
  "src-tauri/src/sources/types.rs|fn:decode_opaque|crate::error#1",
  "src-tauri/src/sources/types.rs|fn:decode_opaque|crate::error#2",
  "src-tauri/src/sources/types.rs|fn:decode_opaque|crate::error#3",
  "src-tauri/src/sources/types.rs|fn:decode_opaque|crate::error#4",
  "src-tauri/src/sources/types.rs|mod:tests/fn:telegram_source_subtype_rejects_unsupported_source_subtype|crate::error#1",
  "src-tauri/src/sources/types.rs|mod:tests/fn:telegram_source_subtype_rejects_unknown_values_as_validation|crate::error#1",
  "src-tauri/src/takeout_import/export_dc.rs|root|crate::error#1",
  "src-tauri/src/takeout_import/export_dc.rs|mod:tests|crate::error#1",
  "src-tauri/src/takeout_import/forum_topics.rs|root|crate::error#1",
  "src-tauri/src/takeout_import/migrated_history.rs|root|crate::error#1",
  "src-tauri/src/takeout_import/migrated_history.rs|mod:tests/fn:migrated_history_errors_are_typed_for_frontend_behavior|crate::error#1",
  "src-tauri/src/takeout_import/migrated_history.rs|mod:tests/fn:migrated_history_errors_are_typed_for_frontend_behavior|crate::error#2",
  "src-tauri/src/takeout_import/migrated_history.rs|mod:tests/fn:validation_rejects_missing_or_changed_revalidated_chat_id|crate::error#1",
  "src-tauri/src/takeout_import/migrated_history.rs|mod:tests/fn:validation_rejects_missing_or_changed_revalidated_chat_id|crate::error#2",
  "src-tauri/src/takeout_import/migrated_history.rs|mod:tests/fn:validation_rejects_missing_or_changed_revalidated_chat_id|crate::error#3",
  "src-tauri/src/takeout_import/mod.rs|root|crate::error#1",
  "src-tauri/src/takeout_import/mod.rs|root|crate::time#1",
  "src-tauri/src/takeout_import/mod.rs|mod:tests|crate::error#1",
  "src-tauri/src/takeout_import/pagination.rs|root|crate::error#1",
  "src-tauri/src/telegram.rs|root|crate::error#1",
  "src-tauri/src/telegram.rs|mod:tests|crate::error#1",
  "src-tauri/src/telegram_session_store.rs|root|crate::error#1",
] as const;

const phase8Status = /### Phase 8 — `extractum-telegram` \(([^)]+)\)/.exec(
  roadmap,
)?.[1];
if (!phase8Status) throw new Error("Missing Phase 8 roadmap status");
const lifecycle =
  telegramContractPaths.telegramLifecycleFromStatus(phase8Status);

function sectionBetween(source: string, start: string, end: string): string {
  const startIndex = source.indexOf(start);
  const endIndex = source.indexOf(end, startIndex + start.length);
  if (startIndex < 0 || endIndex <= startIndex) {
    throw new Error(`Missing Telegram contract section: ${start} -> ${end}`);
  }
  return source.slice(startIndex, endIndex);
}

function exactInventory(
  actual: readonly string[],
  expected: readonly string[],
  label: string,
): void {
  const actualSorted = [...actual].sort();
  const expectedSorted = [...expected].sort();
  expect(
    actualSorted.filter((item, index) => item === actualSorted[index - 1]),
    `${label}: duplicates`,
  ).toEqual([]);
  expect(actualSorted, label).toEqual(expectedSorted);
}

function tableDataRows(
  source: string,
  expectedHeader: string,
  expectedSeparator: string,
  label: string,
): string[] {
  const lines = source.split("\n");
  const headerIndex = lines.indexOf(expectedHeader);
  if (
    headerIndex < 0
    || lines[headerIndex + 1] !== expectedSeparator
  ) {
    throw new Error(`Malformed ${label} table header`);
  }
  const rows = lines.slice(headerIndex + 2).filter((line) => line.trim());
  const nonPipe = rows.find((line) => !line.startsWith("|"));
  if (nonPipe) {
    throw new Error(`Malformed non-pipe ${label} row: ${nonPipe}`);
  }
  if (rows.length === 0) throw new Error(`Empty ${label} table`);
  return rows;
}

function parseIdentityRows(source = plan): IdentityRow[] {
  const table = sectionBetween(
    source,
    "## Literal Immutable 140-Test Identity Map",
    "### Exact New-Test Identity Map",
  );
  const dataRows = tableDataRows(
    table,
    "| baseline_package | baseline_full_id | staged_path | final_owner | final_full_id | companion_final_ids |",
    "| --- | --- | --- | --- | --- | --- |",
    "immutable identity",
  );
  const rows = dataRows.map((line) => {
    const match = line.match(
      /^\| `([^`]+)` \| `([^`]+)` \| `([^`]+)` \| `([^`]+)` \| `([^`]+)` \| (.*?) \|$/,
    );
    if (!match) throw new Error(`Malformed immutable identity row: ${line}`);
    if (match[1] !== "extractum") {
      throw new Error(`Unsupported baseline package: ${match[1]}`);
    }
    if (!["extractum", "extractum-telegram"].includes(match[4])) {
      throw new Error(`Unsupported final owner: ${match[4]}`);
    }
    if (match[4] === "extractum") {
      const baselinePath = baselineSourcePath(match[2]);
      if (match[3] !== baselinePath) {
        throw new Error(
          `Invalid app-owned staged path for ${match[2]}: ${match[3]} != ${baselinePath}`,
        );
      }
      telegramContractPaths.resolveTelegramContractPath(match[3]);
    }
    if (
      match[6] !== "—"
      && !/^`[^`]+`(?:,\s*`[^`]+`)*$/.test(match[6])
    ) {
      throw new Error(`Malformed companion identity cell: ${match[6]}`);
    }
    return {
      baselinePackage: match[1],
      baselineFullId: match[2],
      stagedPath: match[3],
      finalOwner: match[4],
      finalFullId: match[5],
      companionFinalIds: [
        ...match[6].matchAll(/`([^`]+)`/g),
      ].map((companion) => companion[1]),
    };
  });
  return rows;
}

function parseAddedIdentities(source = plan): AddedIdentity[] {
  const table = sectionBetween(
    source,
    "### Exact New-Test Identity Map",
    "The exact test-only runtime seam",
  );
  const dataRows = tableDataRows(
    table,
    "| Checkpoint | Phase 8A exact identity in `extractum` | Future owner / final identity |",
    "| --- | --- | --- |",
    "plan-added identity",
  );
  return dataRows.map((line) => {
    const match = line.match(
      /^\| (\d+) \| `([^`]+)` \| (.*?) \|$/,
    );
    if (!match) throw new Error(`Malformed plan-added identity row: ${line}`);
    const checkpoint = Number(match[1]);
    if (![2, 3, 4, 5].includes(checkpoint)) {
      throw new Error(`Unsupported plan-added checkpoint: ${match[1]}`);
    }
    if (
      !/^(?:[A-Za-z_][A-Za-z0-9_]*::)+tests::[A-Za-z_][A-Za-z0-9_]*$/.test(
        match[2],
      )
    ) {
      throw new Error(
        `Invalid module-qualified current identity: ${match[2]}`,
      );
    }
    if (
      match[3] !== "app / unchanged"
      && !/^crate \/ `(?:[A-Za-z_][A-Za-z0-9_]*::)+tests::[A-Za-z_][A-Za-z0-9_]*`$/.test(
        match[3],
      )
    ) {
      throw new Error(`Unsupported plan-added disposition: ${match[3]}`);
    }
    const finalId = /^crate \/ `([^`]+)`$/.exec(match[3])?.[1];
    if (finalId && match[2] !== `telegram::${finalId}`) {
      throw new Error(
        `Invalid final qualified identity for ${match[2]}: ${finalId}`,
      );
    }
    return {
      checkpoint,
      currentId: match[2],
      finalDisposition: match[3],
      finalId: finalId ?? match[2],
    };
  });
}

function parseStoreIdentities(): string[] {
  const start = plan.indexOf("$sourcesStoreBroadRegressionTestIds = @(");
  const end = plan.indexOf("\n)", start);
  if (start < 0 || end <= start) {
    throw new Error("Missing sourcesStoreBroadRegressionTestIds literal");
  }
  return [
    ...plan.slice(start, end).matchAll(/^\s*'([^']+)'\s*$/gm),
  ].map((match) => match[1]);
}

function baselineSourcePath(fullId: string): string {
  const mappings = [
    ["takeout_import::export_dc::", "src-tauri/src/takeout_import/export_dc.rs"],
    [
      "takeout_import::forum_topics::",
      "src-tauri/src/takeout_import/forum_topics.rs",
    ],
    [
      "takeout_import::migrated_history::",
      "src-tauri/src/takeout_import/migrated_history.rs",
    ],
    [
      "takeout_import::pagination::",
      "src-tauri/src/takeout_import/pagination.rs",
    ],
    [
      "takeout_import::raw_parse::",
      "src-tauri/src/takeout_import/raw_parse.rs",
    ],
    ["takeout_import::", "src-tauri/src/takeout_import/mod.rs"],
    ["sources::identity::", "src-tauri/src/sources/identity.rs"],
    ["sources::items::", "src-tauri/src/sources/items.rs"],
    [
      "sources::peer_resolution::",
      "src-tauri/src/sources/peer_resolution.rs",
    ],
    ["sources::sync::", "src-tauri/src/sources/sync.rs"],
    ["sources::topics::", "src-tauri/src/sources/topics.rs"],
    ["sources::types::", "src-tauri/src/sources/types.rs"],
    ["ingest_provenance::", "src-tauri/src/ingest_provenance.rs"],
    ["telegram_session_store::", "src-tauri/src/telegram_session_store.rs"],
    ["telegram::", "src-tauri/src/telegram.rs"],
    ["media::", "src-tauri/src/media.rs"],
  ] as const;
  const selected = mappings.find(([prefix]) => fullId.startsWith(prefix));
  if (!selected) throw new Error(`Unknown baseline identity module: ${fullId}`);
  return selected[1];
}

function rustPathsUnder(relativeRoot: string): string[] {
  const absoluteRoot = path.join(repoRoot, relativeRoot);
  if (!existsSync(absoluteRoot)) return [];
  return readdirSync(absoluteRoot, { withFileTypes: true }).flatMap((entry) => {
    const selected = `${relativeRoot}/${entry.name}`;
    if (entry.isDirectory()) return rustPathsUnder(selected);
    return entry.isFile() && entry.name.endsWith(".rs") ? [selected] : [];
  });
}

function rawConsumerPaths(
  sources: ReadonlyMap<string, string>,
): string[] {
  const rawConsumer =
    /grammers_(?:client|session|mtsender|tl_types)|\b(?:get_client|get_authorized_runtime|AuthorizedTelegramRuntime|AccountClient|raw_client|raw_session|MemorySession|LoginToken)\b|\.accounts\s*\.lock\s*\(\s*\)\s*\.await/;
  return [...sources.entries()]
    .filter(([, source]) => rawConsumer.test(source))
    .map(([relativePath]) => relativePath);
}

function assertRawConsumerInventory(
  value: telegramContractPaths.TelegramLifecycle,
  appSources: ReadonlyMap<string, string>,
  crateSources: ReadonlyMap<string, string> = new Map(),
  crateManifestExists = false,
): void {
  const actualApp = rawConsumerPaths(appSources);
  if (value === "8c-extracted") {
    if (!crateManifestExists) {
      throw new Error("Missing required 8C crate manifest");
    }
    exactInventory(actualApp, [], "8C app raw consumer paths");
    exactInventory(
      rawConsumerPaths(crateSources),
      crateRawConsumerPaths,
      "8C crate raw consumer paths",
    );
    return;
  }
  if (value === "8b-preparation") {
    exactInventory(
      actualApp,
      stagedRawConsumerPaths,
      "8B staged raw consumer paths",
    );
    exactInventory(
      rawConsumerPaths(crateSources),
      [],
      "8B premature crate raw consumer paths",
    );
    return;
  }

  let expectedApp = [...checkpointOneRawConsumerPaths];
  if (checkpointNumber(value) >= 3) {
    expectedApp = expectedApp.map((relativePath) =>
      relativePath === "src-tauri/src/media.rs"
        ? "src-tauri/src/telegram/media.rs"
        : relativePath,
    );
  }
  if (checkpointNumber(value) >= 4) {
    expectedApp = expectedApp.map((relativePath) =>
      relativePath === "src-tauri/src/telegram_session_store.rs"
        ? "src-tauri/src/telegram/session.rs"
        : relativePath,
    );
  }
  if (checkpointNumber(value) >= 5) {
    expectedApp = expectedApp.map((relativePath) =>
      relativePath === "src-tauri/src/telegram.rs"
        ? "src-tauri/src/telegram/runtime.rs"
        : relativePath,
    );
  }
  exactInventory(
    actualApp,
    expectedApp,
    `${value} app raw consumer paths`,
  );
  exactInventory(
    rawConsumerPaths(crateSources),
    [],
    `${value} premature crate raw consumer paths`,
  );
}

function countMatches(source: string, expression: RegExp): number {
  return [...source.matchAll(expression)].length;
}

function facadeLifecycleSource(
  baselinePath: string,
): telegramContractPaths.TelegramLifecycleSource {
  if (baselinePath === "src-tauri/src/media.rs") {
    return {
      baselinePath,
      stagedPath: "src-tauri/src/telegram_impl/media.rs",
      finalOwner: "extractum-telegram",
    };
  }
  return {
    baselinePath,
    stagedPath: baselinePath,
    finalOwner: "extractum",
  };
}

function assertFacadeInventory(
  sourceOverrides: ReadonlyMap<string, string> = new Map(),
  value: telegramContractPaths.TelegramLifecycle = lifecycle,
): void {
  if (value === "8b-preparation" || value === "8c-extracted") return;
  const ownerPaths = ownershipMovePaths.map((baselinePath) =>
    telegramContractPaths.resolveTelegramLifecyclePath(
      facadeLifecycleSource(baselinePath),
      value,
    ),
  );
  const ownerSources = [...new Set(ownerPaths)].map((relativePath) => ({
    relativePath,
    source:
      sourceOverrides.get(relativePath)
      ?? telegramContractPaths.readTelegramContractFile(relativePath),
  }));
  const facadeCounts = {
    error: ownerSources.reduce(
      (total, { source }) =>
        total + countMatches(source, /\bcrate::error\b/g),
      0,
    ),
    compression: ownerSources.reduce(
      (total, { source }) =>
        total + countMatches(source, /\bcrate::compression\b/g),
      0,
    ),
    time: ownerSources.reduce(
      (total, { source }) =>
        total + countMatches(source, /\bcrate::time\b/g),
      0,
    ),
  };
  expect(facadeCounts).toEqual({ error: 37, compression: 5, time: 2 });
  expect(
    Object.values(facadeCounts).reduce(
      (total, count) => total + count,
      0,
    ),
  ).toBe(44);
  const actualSites = ownerSources.flatMap(({ relativePath, source }) =>
    semanticFacadeSites(relativePath, source),
  );
  const expectedSites = checkpointOneFacadeSites.map((site) => {
    const separator = site.indexOf("|");
    const baselinePath = site.slice(0, separator);
    const semanticSite = site.slice(separator + 1);
    const relativePath =
      telegramContractPaths.resolveTelegramLifecyclePath(
        facadeLifecycleSource(baselinePath),
        value,
      );
    return `${relativePath}|${semanticSite}`;
  });
  exactInventory(
    actualSites,
    expectedSites,
    "semantic 44-facade site inventory",
  );
}

function assertCheckpointOneRootInventory(
  value: telegramContractPaths.TelegramLifecycle,
  pathExists: (relativePath: string) => boolean = (relativePath) =>
    existsSync(path.join(repoRoot, relativePath)),
): void {
  if (checkpointNumber(value) > 2) return;
  const rootCounts = new Map<string, number>();
  for (const relativePath of ownershipMovePaths) {
    const source =
      telegramContractPaths.readTelegramContractFile(relativePath);
    for (const match of source.matchAll(
      /\bcrate::([A-Za-z_][A-Za-z0-9_]*)/g,
    )) {
      rootCounts.set(match[1], (rootCounts.get(match[1]) ?? 0) + 1);
    }
  }
  const prematureLayouts = [
    "src-tauri/src/telegram",
    "src-tauri/src/telegram_impl",
    "src-tauri/crates/extractum-telegram",
  ].filter(pathExists);
  if (prematureLayouts.length > 0) {
    throw new Error(
      `Checkpoint 1 premature owner layout: ${prematureLayouts.join(", ")}`,
    );
  }
  expect(
    Object.fromEntries(rootCounts),
    "Checkpoint 1 live crate-root inventory",
  ).toEqual({
    ...baselineRootCounts,
    error: 37,
  });
  expect(
    [...rootCounts.values()].reduce((total, count) => total + count, 0),
  ).toBe(160);
}

function rustCharLiteralEnd(source: string, start: number): number | undefined {
  if (source[start] !== "'") return undefined;
  let index = start + 1;
  if (source[index] === "\\") {
    index += 1;
    if (source[index] === "u" && source[index + 1] === "{") {
      const unicodeClose = source.indexOf("}", index + 2);
      if (unicodeClose < 0) return undefined;
      index = unicodeClose + 1;
    } else if (source[index] === "x") {
      index += 3;
    } else {
      index += 1;
    }
  } else {
    const codePoint = source.codePointAt(index);
    if (codePoint === undefined || source[index] === "\n") return undefined;
    index += codePoint > 0xffff ? 2 : 1;
  }
  return source[index] === "'" ? index : undefined;
}

function rustLexicalNonCodeEnd(
  source: string,
  start: number,
  label: string,
): number | undefined {
  const rawPrefix = /^(?:b|c)?r(#+)?"/.exec(source.slice(start));
  if (rawPrefix) {
    const close = source.indexOf(
      `"${rawPrefix[1] ?? ""}`,
      start + rawPrefix[0].length,
    );
    if (close < 0) throw new Error(`Unclosed Rust raw string in: ${label}`);
    return close + (rawPrefix[1]?.length ?? 0);
  }
  if (source.startsWith("//", start)) {
    const newline = source.indexOf("\n", start + 2);
    return newline < 0 ? source.length - 1 : newline;
  }
  if (source.startsWith("/*", start)) {
    let commentDepth = 1;
    let index = start + 2;
    while (index < source.length && commentDepth > 0) {
      if (source.startsWith("/*", index)) {
        commentDepth += 1;
        index += 2;
      } else if (source.startsWith("*/", index)) {
        commentDepth -= 1;
        index += 2;
      } else {
        index += 1;
      }
    }
    if (commentDepth > 0) {
      throw new Error(`Unclosed Rust block comment in: ${label}`);
    }
    return index - 1;
  }
  if (source[start] === '"') {
    let index = start + 1;
    while (index < source.length) {
      if (source[index] === "\\") {
        index += 2;
      } else if (source[index] === '"') {
        return index;
      } else {
        index += 1;
      }
    }
    throw new Error(`Unclosed Rust string in: ${label}`);
  }
  return rustCharLiteralEnd(source, start);
}

function maskRustLexicalNonCode(source: string): string {
  const masked = source.split("");
  for (let index = 0; index < source.length; index += 1) {
    const end = rustLexicalNonCodeEnd(source, index, "semantic mask");
    if (end === undefined) continue;
    for (let cursor = index; cursor <= end; cursor += 1) {
      if (masked[cursor] !== "\n" && masked[cursor] !== "\r") {
        masked[cursor] = " ";
      }
    }
    index = end;
  }
  return masked.join("");
}

function rustClosingBraceIndex(
  source: string,
  open: number,
  label: string,
): number {
  let depth = 0;
  for (let index = open; index < source.length; index += 1) {
    const lexicalEnd = rustLexicalNonCodeEnd(source, index, label);
    if (lexicalEnd !== undefined) {
      index = lexicalEnd;
      continue;
    }
    if (source[index] === "{") depth += 1;
    if (source[index] === "}") depth -= 1;
    if (depth === 0) return index;
  }
  throw new Error(`Unclosed Rust block: ${label}`);
}

function rustFunctionOriginalBody(source: string, name: string): string {
  const searchable = maskRustLexicalNonCode(source);
  const declaration = new RegExp(
    `\\b(?:async\\s+)?fn\\s+${name}(?:\\s*<[^>{}]*>)?\\s*\\(`,
  ).exec(searchable);
  if (!declaration || declaration.index === undefined) {
    throw new Error(`Missing Rust function: ${name}`);
  }
  const open = searchable.indexOf(
    "{",
    declaration.index + declaration[0].length,
  );
  if (open < 0) throw new Error(`Missing Rust function body: ${name}`);
  const close = rustClosingBraceIndex(source, open, `function ${name}`);
  return source.slice(open + 1, close);
}

function rustFunctionBody(source: string, name: string): string {
  return maskRustLexicalNonCode(rustFunctionOriginalBody(source, name));
}

function normalizedRustCommandDeclaration(
  source: string,
  name: string,
): string {
  const searchable = maskRustLexicalNonCode(source);
  const declaration = new RegExp(
    `#\\[tauri::command\\]\\s*pub\\s+async\\s+fn\\s+${name}\\s*\\(`,
    "g",
  );
  const matches = [...searchable.matchAll(declaration)];
  if (matches.length !== 1 || matches[0].index === undefined) {
    throw new Error(
      `Expected one exact #[tauri::command] declaration for ${name}, found ${matches.length}`,
    );
  }
  const open = searchable.indexOf(
    "{",
    matches[0].index + matches[0][0].length,
  );
  if (open < 0) throw new Error(`Missing command body for ${name}`);
  return source
    .slice(matches[0].index, open)
    .replace(/\s+/g, " ")
    .replace(/\(\s+/g, "(")
    .replace(/\s+\)/g, ")")
    .trim();
}

function rustCommandIpcKeys(
  declaration: string,
  name: string,
): string[] {
  const parameterStart = declaration.indexOf(`fn ${name}(`);
  if (parameterStart < 0) {
    throw new Error(`Missing normalized command parameters for ${name}`);
  }
  const start = parameterStart + `fn ${name}(`.length;
  const parameters: string[] = [];
  let angleDepth = 0;
  let parameter = "";
  for (let index = start; index < declaration.length; index += 1) {
    const character = declaration[index];
    if (character === "<") angleDepth += 1;
    if (character === ">") angleDepth -= 1;
    if (character === ")" && angleDepth === 0) {
      if (parameter.trim()) parameters.push(parameter.trim());
      break;
    }
    if (character === "," && angleDepth === 0) {
      if (parameter.trim()) parameters.push(parameter.trim());
      parameter = "";
      continue;
    }
    parameter += character;
  }
  return parameters.flatMap((entry) => {
    const separator = entry.indexOf(":");
    if (separator < 0) {
      throw new Error(`Malformed command parameter for ${name}: ${entry}`);
    }
    const parameterName = entry.slice(0, separator).trim();
    const parameterType = entry.slice(separator + 1).trim();
    if (
      parameterType === "AppHandle" ||
      parameterType.startsWith("tauri::State<")
    ) {
      return [];
    }
    return [
      parameterName.replace(/_([a-z0-9])/g, (_match, letter: string) =>
        letter.toUpperCase(),
      ),
    ];
  });
}

function activeInvokeHandlerRegistrations(source: string): string[] {
  const searchable = maskRustLexicalNonCode(source);
  const invocation =
    /\.invoke_handler\s*\(\s*tauri::generate_handler!\s*\[/g;
  return [...searchable.matchAll(invocation)].map((match) => {
    const start = searchable.indexOf("[", match.index);
    let depth = 0;
    for (let index = start; index < searchable.length; index += 1) {
      if (searchable[index] === "[") depth += 1;
      if (searchable[index] === "]") depth -= 1;
      if (depth === 0) {
        return searchable.slice(start + 1, index).trim();
      }
    }
    throw new Error("Unclosed tauri::generate_handler! registration");
  });
}

function assertLiveSyncIteratorSelection(body: string): void {
  const normalized = maskRustLexicalNonCode(body).replace(/\s+/g, " ");
  expect(normalized).toMatch(
    /let mut messages = if let Some\(settings\) = sync_policy\.initial_sync_settings\.as_ref\(\) \{ match settings\.initial_sync_mode \{ InitialSyncMode::RecentMessages => client \.iter_messages\(peer\) \.limit\(settings\.initial_sync_value as usize\), InitialSyncMode::RecentDays => client\.iter_messages\(peer\), \} \} else \{ client\.iter_messages\(peer\) \};/,
  );
}

function rustCallArguments(source: string, name: string): string[] {
  const searchable = maskRustLexicalNonCode(source);
  const declaration = new RegExp(`\\b${name}\\s*\\(`, "g");
  return [...searchable.matchAll(declaration)].map((match) => {
    const open = searchable.indexOf("(", match.index);
    let depth = 0;
    for (let index = open; index < searchable.length; index += 1) {
      if (searchable[index] === "(") depth += 1;
      if (searchable[index] === ")") depth -= 1;
      if (depth === 0) return searchable.slice(open + 1, index);
    }
    throw new Error(`Unclosed Rust call: ${name}`);
  });
}

function rustActiveOriginalCallArguments(
  source: string,
  name: string,
): string[] {
  const searchable = maskRustLexicalNonCode(source);
  const declaration = new RegExp(
    `${name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}\\s*\\(`,
    "g",
  );
  return [...searchable.matchAll(declaration)].map((match) => {
    const open = searchable.indexOf("(", match.index);
    let depth = 0;
    for (let index = open; index < searchable.length; index += 1) {
      if (searchable[index] === "(") depth += 1;
      if (searchable[index] === ")") depth -= 1;
      if (depth === 0) return source.slice(open + 1, index);
    }
    throw new Error(`Unclosed Rust call: ${name}`);
  });
}

function normalizeRustFormattingWhitespace(source: string): string {
  let normalized = "";
  let pendingWhitespace = false;

  const appendPendingWhitespace = (): void => {
    if (pendingWhitespace && normalized.length > 0) normalized += " ";
    pendingWhitespace = false;
  };

  for (let index = 0; index < source.length; index += 1) {
    const lexicalEnd = rustLexicalNonCodeEnd(
      source,
      index,
      "active call arguments",
    );
    if (lexicalEnd !== undefined) {
      if (
        source.startsWith("//", index)
        || source.startsWith("/*", index)
      ) {
        pendingWhitespace = true;
      } else {
        appendPendingWhitespace();
        normalized += source.slice(index, lexicalEnd + 1);
      }
      index = lexicalEnd;
      continue;
    }
    if (/\s/u.test(source[index])) {
      pendingWhitespace = true;
      continue;
    }
    appendPendingWhitespace();
    normalized += source[index];
  }

  return normalized;
}

function normalizedActiveCallArguments(
  source: string,
  name: string,
): string[] {
  return rustActiveOriginalCallArguments(source, name).map((argumentsSource) =>
    normalizeRustFormattingWhitespace(argumentsSource),
  );
}

function assertAuthorizedRuntimeAuthArguments(source: string): void {
  const authorizedRuntimeOriginal = rustFunctionOriginalBody(
    source,
    "get_authorized_runtime",
  );
  expect(
    normalizedActiveCallArguments(
      authorizedRuntimeOriginal,
      "AppError::auth",
    ),
  ).toEqual([
    'format!("Account {account_id} not initialized")',
    'format!( "Account {account_id} is not authenticated" )',
  ]);
}

function assertWrappedTakeoutCalls(
  source: string,
  functionName: string,
  expectedMarkers: readonly string[],
): void {
  const wrappers = rustCallArguments(
    rustFunctionBody(source, functionName),
    "run_takeout_step_with_cancel",
  );
  expect(wrappers, `${functionName} cancellation wrappers`).toHaveLength(
    expectedMarkers.length,
  );
  wrappers.forEach((wrapper, index) => {
    expect(
      wrapper,
      `${functionName} remote wrapper ${index + 1}`,
    ).toContain(expectedMarkers[index]);
  });
}

function assertTakeoutObservableBoundaries(source: string): void {
  assertWrappedTakeoutCalls(source, "run_takeout_migrated_history_import", [
    "resolve_and_refresh_peer(",
    "prepare_export_dc_alias(",
    "export_dc_invoke_with_provenance(",
    "revalidate_migrated_from_chat_id(",
    "export_dc_invoke_with_provenance(",
    "takeout_history_count_probe(",
    "finish_takeout_session(",
  ]);
  assertWrappedTakeoutCalls(source, "run_takeout_source_import", [
    "resolve_and_refresh_peer(",
    "tl::functions::users::GetUsers",
    "prepare_export_dc_alias(",
    "export_dc_invoke_with_provenance(",
  ]);
  assertWrappedTakeoutCalls(
    source,
    "run_started_takeout_source_import_inner",
    [
      "validate_takeout_peer(",
      "detect_supergroup_migration(",
      "export_dc_invoke_with_provenance(",
      "takeout_history_count_probe(",
      "finish_takeout_session(",
    ],
  );
  assertWrappedTakeoutCalls(source, "import_takeout_history_pages", [
    "takeout_history_page_response(",
  ]);

  const pages = rustFunctionBody(source, "import_takeout_history_pages");
  expect(pages.replace(/\s+/g, " ")).toMatch(
    /update_and_emit\(handle, &takeout_state, job_id, \|job\| \{ job\.inserted = imported\.inserted; job\.skipped = imported\.skipped; job\.progress_current = Some\(\(imported\.inserted \+ imported\.skipped\)\.min\(total\)\); job\.progress_total = Some\(total\); job\.warnings = warnings\.clone\(\); \}\) \.await;/,
  );
  expectOrdered(
    pages,
    [
      "insert_telegram_source_item_with_observation",
      ".await?",
      "update_and_emit(handle, &takeout_state, job_id",
      "job.inserted = imported.inserted",
      "job.skipped = imported.skipped",
      "job.progress_current = Some((imported.inserted + imported.skipped).min(total))",
      "job.progress_total = Some(total)",
      "job.warnings = warnings.clone()",
      ".await",
    ],
    "Takeout persist-before-exact-page-progress callback",
  );

  const job = rustFunctionBody(source, "run_takeout_import_job");
  const remoteStart = job.indexOf("match run_takeout_source_import");
  const initialCancelled = job.slice(0, remoteStart);
  expectOrdered(
    initialCancelled,
    [
      "finish_job(&job_id",
      "job.status = STATUS_CANCELLED.to_string()",
      "job.phase = PHASE_CANCELLED.to_string()",
      "emit_takeout_import_event(&handle, &record)",
    ],
    "Takeout initial-cancel finish-mutate-emit",
  );
  const completedStart = job.indexOf("Ok(outcome)", remoteStart);
  const errorStart = job.indexOf("Err(error)", completedStart);
  expectOrdered(
    job.slice(completedStart, errorStart),
    [
      "finish_job(&job_id",
      "job.status = STATUS_COMPLETED.to_string()",
      "job.phase = PHASE_COMPLETED.to_string()",
      "emit_takeout_import_event(&handle, &record)",
    ],
    "Takeout completed finish-mutate-emit",
  );
  const errorBranch = job.slice(errorStart);
  const cancelledStart = errorBranch.indexOf(
    "if takeout_state.is_cancel_requested(&job_id).await",
  );
  const failedStart = errorBranch.indexOf("} else {", cancelledStart);
  expectOrdered(
    errorBranch.slice(cancelledStart, failedStart),
    [
      "finish_job(&job_id",
      "job.status = STATUS_CANCELLED.to_string()",
      "job.phase = PHASE_CANCELLED.to_string()",
      "emit_takeout_import_event(&handle, &record)",
    ],
    "Takeout error-cancel finish-mutate-emit",
  );
  expectOrdered(
    errorBranch.slice(failedStart),
    [
      "finish_job(&job_id",
      "job.status = STATUS_FAILED.to_string()",
      "job.phase = PHASE_FAILED.to_string()",
      "job.error = Some(terminal_error.clone())",
      "emit_takeout_import_event(&handle, &record)",
    ],
    "Takeout failed finish-mutate-emit",
  );
}

function expectOrdered(
  source: string,
  markers: readonly string[],
  label: string,
): void {
  let previous = -1;
  for (const marker of markers) {
    const current = source.indexOf(marker, previous + 1);
    expect(current, `${label}: missing or reordered ${marker}`).toBeGreaterThan(
      previous,
    );
    previous = current;
  }
}

type RustSemanticItemSpan = {
  label: string;
  start: number;
  end: number;
};

function rustSemanticItemSpans(
  source: string,
  searchable = maskRustLexicalNonCode(source),
): RustSemanticItemSpan[] {
  const spans: RustSemanticItemSpan[] = [];
  const declarations = [
    {
      expression: /\bmod\s+([A-Za-z_][A-Za-z0-9_]*)\s*\{/g,
      label: (name: string) => `mod:${name}`,
      open: (match: RegExpMatchArray) =>
        (match.index ?? 0) + match[0].lastIndexOf("{"),
    },
    {
      expression:
        /\b(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)(?:\s*<[^>{}]*>)?\s*\(/g,
      label: (name: string) => `fn:${name}`,
      open: (match: RegExpMatchArray) =>
        searchable.indexOf(
          "{",
          (match.index ?? 0) + match[0].length,
        ),
    },
  ];
  for (const declaration of declarations) {
    for (const match of searchable.matchAll(declaration.expression)) {
      if (match.index === undefined) continue;
      const open = declaration.open(match);
      if (open < 0) {
        throw new Error(`Missing Rust item body: ${match[1]}`);
      }
      spans.push({
        label: declaration.label(match[1]),
        start: match.index,
        end: rustClosingBraceIndex(
          source,
          open,
          `${declaration.label(match[1])} semantic item`,
        ),
      });
    }
  }
  return spans;
}

function semanticFacadeSites(
  relativePath: string,
  source: string,
): string[] {
  const searchable = maskRustLexicalNonCode(source);
  const spans = rustSemanticItemSpans(source, searchable);
  const occurrences = new Map<string, number>();
  return [
    ...searchable.matchAll(/\bcrate::(?:error|compression|time)\b/g),
  ].map((match) => {
    if (match.index === undefined) {
      throw new Error(`Missing facade match index in ${relativePath}`);
    }
    const item = spans
      .filter(({ start, end }) => start <= match.index! && match.index! < end)
      .sort((left, right) => left.start - right.start)
      .map(({ label }) => label)
      .join("/") || "root";
    const occurrenceKey = `${item}|${match[0]}`;
    const occurrence = (occurrences.get(occurrenceKey) ?? 0) + 1;
    occurrences.set(occurrenceKey, occurrence);
    return `${relativePath}|${occurrenceKey}#${occurrence}`;
  });
}

function parseFixtureIdentitySet(): string[] {
  const fixtureSection = sectionBetween(
    design,
    "The exact 43 app-fixture identities are frozen by module:",
    "Mixed production files and test modules split by subject",
  );
  const headings = [
    ...fixtureSection.matchAll(
      /^- `([^`]+)` \((\d+)\):(?:\r?\n|$)/gm,
    ),
  ];
  return headings.flatMap((heading, index) => {
    if (heading.index === undefined) throw new Error("Fixture heading index");
    const body = fixtureSection.slice(
      heading.index + heading[0].length,
      headings[index + 1]?.index ?? fixtureSection.length,
    );
    const names = [...body.matchAll(/`([A-Za-z0-9_]+)`/g)].map(
      (match) => match[1],
    );
    if (names.length !== Number(heading[2])) {
      throw new Error(
        `Fixture group ${heading[1]} declares ${heading[2]}, found ${names.length}`,
      );
    }
    return names.map((name) => `${heading[1]}::${name}`);
  });
}

function checkpointNumber(value: telegramContractPaths.TelegramLifecycle): number {
  if (value === "8a-retained" || value === "8b-preparation" || value === "8c-extracted") {
    return 5;
  }
  return Number(/^8a-checkpoint-(\d)$/.exec(value)?.[1] ?? 0);
}

function modulePathForRustSource(relativePath: string): string {
  const roots = [
    "src-tauri/src/",
    "src-tauri/crates/extractum-telegram/src/",
  ];
  const root = roots.find((candidate) => relativePath.startsWith(candidate));
  if (!root || !relativePath.endsWith(".rs")) {
    throw new Error(`Unsupported Rust module path: ${relativePath}`);
  }
  const withinRoot = relativePath.slice(root.length);
  const withoutFile =
    withinRoot === "lib.rs"
      ? ""
      : withinRoot.endsWith("/mod.rs")
        ? withinRoot.slice(0, -"/mod.rs".length)
        : withinRoot.slice(0, -".rs".length);
  if (!withoutFile) throw new Error(`Root module has no test prefix: ${relativePath}`);
  return withoutFile.replaceAll("/", "::");
}

function qualifiedTestIdentity(relativePath: string, name: string): string {
  return `${modulePathForRustSource(relativePath)}::tests::${name}`;
}

function assertIdentityDeclarations(
  rows: readonly IdentityRow[],
  value: telegramContractPaths.TelegramLifecycle,
): void {
  const currentCheckpoint = checkpointNumber(value);
  for (const row of rows) {
    const baselinePath = baselineSourcePath(row.baselineFullId);
    const baselineName = row.baselineFullId.slice(
      row.baselineFullId.lastIndexOf("::") + 2,
    );
    const expectedBaselineIdentity = qualifiedTestIdentity(
      baselinePath,
      baselineName,
    );
    if (row.baselineFullId !== expectedBaselineIdentity) {
      throw new Error(
        `Invalid baseline module-qualified identity: ${row.baselineFullId} != ${expectedBaselineIdentity}`,
      );
    }
    const finalName = row.finalFullId.slice(
      row.finalFullId.lastIndexOf("::") + 2,
    );
    const finalPath =
      row.finalOwner === "extractum"
        ? baselinePath
        : row.stagedPath.replace(
            "src-tauri/src/telegram_impl/",
            "src-tauri/crates/extractum-telegram/src/",
          );
    const expectedFinalIdentity = qualifiedTestIdentity(finalPath, finalName);
    if (row.finalFullId !== expectedFinalIdentity) {
      throw new Error(
        `Invalid final module-qualified identity: ${row.finalFullId} != ${expectedFinalIdentity}`,
      );
    }
    const currentPath =
      telegramContractPaths.resolveTelegramLifecyclePath(
        {
          baselinePath,
          stagedPath: row.stagedPath,
          finalOwner: row.finalOwner as
            | "extractum"
            | "extractum-telegram",
        },
        value,
      );
    const source =
      telegramContractPaths.readTelegramContractFile(currentPath);
    const name = currentCheckpoint >= 3 ? finalName : baselineName;
    const currentFullId = qualifiedTestIdentity(currentPath, name);
    expect(
      countMatches(
        source,
        new RegExp(`\\b(?:async\\s+)?fn\\s+${name}\\s*\\(`, "g"),
      ),
      `${currentFullId} declaration count in ${currentPath}`,
    ).toBe(1);
  }
}

function assertAddedIdentityDeclarations(
  rows: readonly AddedIdentity[],
  value: telegramContractPaths.TelegramLifecycle,
  rustSources: ReadonlyMap<string, string>,
): void {
  const currentCheckpoint = checkpointNumber(value);
  const expectedDeclarations = rows
    .filter(({ checkpoint }) => checkpoint <= currentCheckpoint)
    .map((added) => {
      if (added.finalDisposition === "app / unchanged") {
        return added.currentId;
      }
      if (value === "8c-extracted") return added.finalId;
      if (value === "8b-preparation") {
        return `telegram_impl::${added.finalId}`;
      }
      return added.currentId;
    });
  const actualDeclarations = rows.flatMap((added) => {
    const name = added.currentId.slice(added.currentId.lastIndexOf("::") + 2);
    return [...rustSources.entries()].flatMap(([relativePath, source]) => {
      const declarationCount = countMatches(
        source,
        new RegExp(`\\b(?:async\\s+)?fn\\s+${name}\\s*\\(`, "g"),
      );
      return Array.from(
        { length: declarationCount },
        () => qualifiedTestIdentity(relativePath, name),
      );
    });
  });
  exactInventory(
    actualDeclarations,
    expectedDeclarations,
    "module-qualified plan-added declarations",
  );
}

const identityRows = parseIdentityRows();
const addedIdentities = parseAddedIdentities();
const storeIdentities = parseStoreIdentities();

describe("Phase 8 Telegram crate boundary", () => {
  it("ignores block-comment, line-comment, and raw-string function declarations", () => {
    const source = String.raw`
      /* fn sample() { fake_block(); } */
      // fn sample() { fake_line(); }
      const FAKE: &str = r###"fn sample() { fake_raw(); }"###;
      fn sample() { real_body(); }
    `;
    expect(rustFunctionBody(source, "sample").trim()).toBe("real_body();");

    const commandSource = String.raw`
      /* #[tauri::command] pub async fn sample(fake: String) -> AppResult<()> { fake_block(); } */
      // #[tauri::command] pub async fn sample(fake: String) -> AppResult<()> { fake_line(); }
      const FAKE: &str = r###"#[tauri::command] pub async fn sample(fake: String) -> AppResult<()> { fake_raw(); }"###;
      #[tauri::command]
      pub async fn sample(account_id: i64) -> AppResult<bool> { Ok(true) }
    `;
    expect(normalizedRustCommandDeclaration(commandSource, "sample")).toBe(
      "#[tauri::command] pub async fn sample(account_id: i64) -> AppResult<bool>",
    );
  });

  it("finds every active invoke-handler registration and ignores lexical fakes", () => {
    const source = String.raw`
      /* app.invoke_handler(tauri::generate_handler![fake_block]) */
      // app.invoke_handler(tauri::generate_handler![fake_line])
      const FAKE: &str = r###"app.invoke_handler(tauri::generate_handler![fake_raw])"###;
      app.invoke_handler(tauri::generate_handler![first]);
      app.invoke_handler(tauri::generate_handler![second]);
    `;
    expect(activeInvokeHandlerRegistrations(source)).toEqual([
      "first",
      "second",
    ]);
  });

  it("rejects dead handler command text inside the active registration", () => {
    const source = String.raw`
      app.invoke_handler(tauri::generate_handler![
        active_command,
        // dead_command,
        /* dead_command */
      ]);
    `;
    const registrations = activeInvokeHandlerRegistrations(source);
    expect(registrations).toHaveLength(1);
    expect(countMatches(registrations[0], /\bdead_command\b/g)).toBe(0);
  });

  it("rejects dead wrapper remote text inside comments and strings", () => {
    const source = String.raw`
      run_takeout_step_with_cancel(token, async {
        // expected_remote()
        let dead = "expected_remote()";
        active_remote()
      })
    `;
    const wrappers = rustCallArguments(
      source,
      "run_takeout_step_with_cancel",
    );
    expect(wrappers).toHaveLength(1);
    expect(wrappers[0]).not.toContain("expected_remote(");
  });

  it("rejects dead canonical live-sync text beside mutated active code", () => {
    const body = String.raw`
      /*
      let mut messages = if let Some(settings) = sync_policy.initial_sync_settings.as_ref() {
        match settings.initial_sync_mode {
          InitialSyncMode::RecentMessages => client
            .iter_messages(peer)
            .limit(settings.initial_sync_value as usize),
          InitialSyncMode::RecentDays => client.iter_messages(peer),
        }
      } else {
        client.iter_messages(peer)
      };
      */
      let mut messages = client.iter_messages(peer);
    `;
    expect(() => assertLiveSyncIteratorSelection(body)).toThrow();
  });

  it("rejects a line-comment literal that spoofs an active result", () => {
    const source = String.raw`
      fn sample() {
        Ok("changed".to_string());
        // Ok("expected".to_string());
      }
    `;
    expect(
      normalizedActiveCallArguments(
        rustFunctionOriginalBody(source, "sample"),
        "Ok",
      ),
    ).toEqual(['"changed".to_string()']);
  });

  it("rejects a block-comment literal that spoofs an active constructor", () => {
    const source = String.raw`
      fn sample() {
        AppError::auth("changed");
        /* AppError::auth("expected"); */
      }
    `;
    expect(
      normalizedActiveCallArguments(
        rustFunctionOriginalBody(source, "sample"),
        "AppError::auth",
      ),
    ).toEqual(['"changed"']);
  });

  it("preserves double internal spaces in an active string literal", () => {
    const source = String.raw`
      fn sample() {
        Ok("Code  sent".to_string());
      }
    `;
    expect(
      normalizedActiveCallArguments(
        rustFunctionOriginalBody(source, "sample"),
        "Ok",
      ),
    ).toEqual(['"Code  sent".to_string()']);
  });

  it("rejects doubled internal spaces in the active authorized-runtime Auth literal", () => {
    const source = telegramContractPaths.readTelegramContractFile(
      "src-tauri/src/telegram.rs",
    );
    const mutation = source.replace(
      '"Account {account_id} is not authenticated"',
      '"Account {account_id} is not  authenticated"',
    );
    expect(mutation).not.toBe(source);
    expect(() => assertAuthorizedRuntimeAuthArguments(mutation)).toThrow();
  });

  it("rejects a live-sync mutation that bounds the RecentDays match arm", () => {
    const source =
      telegramContractPaths.readTelegramContractFile(
        "src-tauri/src/sources/sync.rs",
      );
    const body = rustFunctionBody(source, "persist_items");
    const mutation = body.replace(
      "InitialSyncMode::RecentDays => client.iter_messages(peer),",
      "InitialSyncMode::RecentDays => client.iter_messages(peer).limit(settings.initial_sync_value as usize),",
    );
    expect(mutation).not.toBe(body);
    expect(() => assertLiveSyncIteratorSelection(mutation)).toThrow();
  });

  it("rejects Takeout mutations that bypass cancellation, progress, or terminal emission order", () => {
    const source =
      telegramContractPaths.readTelegramContractFile(
        "src-tauri/src/takeout_import/mod.rs",
      );
    const unwrappedRemote = source.replace(
      "let resolved_peer = run_takeout_step_with_cancel(",
      "let resolved_peer = bypass_takeout_cancel(",
    );
    const wrongProgress = source.replace(
      "Some((imported.inserted + imported.skipped).min(total))",
      "Some((imported.inserted + imported.skipped).max(total))",
    );
    const wrongCompletedOrder = source.replace(
      "job.status = STATUS_COMPLETED.to_string();",
      "job.status = STATUS_RUNNING.to_string();",
    );
    expect(unwrappedRemote).not.toBe(source);
    expect(wrongProgress).not.toBe(source);
    expect(wrongCompletedOrder).not.toBe(source);
    expect(() => assertTakeoutObservableBoundaries(unwrappedRemote)).toThrow();
    expect(() => assertTakeoutObservableBoundaries(wrongProgress)).toThrow();
    expect(() =>
      assertTakeoutObservableBoundaries(wrongCompletedOrder),
    ).toThrow();
  });

  it("scans Rust function braces lexically across strings and comments", () => {
    const source = String.raw`
      fn sample() {
        let quoted = "}";
        let raw = r###"}"###;
        let lifetime: &'static str = "{";
        let escaped = '\u{7d}';
        // }
        /* outer } /* nested } */ */
        forbidden_call();
      }
    `;
    expect(rustFunctionBody(source, "sample")).toContain("forbidden_call()");
  });

  it("ignores declaration-shaped Rust strings and comments in facade enclosures", () => {
    const relativePath = "src-tauri/src/synthetic.rs";
    const source = String.raw`
      const DECLARATION_TEXT: &str = "fn fake(";
      // mod fake {
      /* fn also_fake( { */
      use crate::error::AppResult;
    `;
    expect(semanticFacadeSites(relativePath, source)).toEqual([
      `${relativePath}|root|crate::error#1`,
    ]);
  });

  it("installs the fail-closed Telegram contract path helper", () => {
    expect(
      existsSync(path.join(repoRoot, "src/lib/telegram-contract-paths.ts")),
    ).toBe(true);
  });

  it("exposes the closed repository-path and lifecycle API", () => {
    const api = telegramContractPaths as Record<string, unknown>;

    expect(api.normalizeTelegramContractSourceText).toBeTypeOf("function");
    expect(api.resolveTelegramContractPath).toBeTypeOf("function");
    expect(api.readTelegramContractFile).toBeTypeOf("function");
    expect(api.resolveTelegramLifecyclePath).toBeTypeOf("function");
    expect(api.telegramLifecycleFromStatus).toBeTypeOf("function");
  });

  it("reads only existing repository-relative files and rejects escapes", () => {
    expect(
      telegramContractPaths.normalizeTelegramContractSourceText("a\r\nb\rc"),
    ).toBe("a\nb\nc");
    expect(
      telegramContractPaths.resolveTelegramContractPath(
        "src-tauri/src/telegram.rs",
      ),
    ).toBe(path.join(repoRoot, "src-tauri/src/telegram.rs"));
    expect(
      telegramContractPaths.readTelegramContractFile(
        "src-tauri/src/telegram.rs",
      ),
    ).toContain("pub struct TelegramState");
    expect(() =>
      telegramContractPaths.resolveTelegramContractPath("../Cargo.toml"),
    ).toThrow(/dot path segment/);
    expect(() =>
      telegramContractPaths.resolveTelegramContractPath(
        "src-tauri/src/../Cargo.toml",
      ),
    ).toThrow(/dot path segment/);
    expect(() =>
      telegramContractPaths.resolveTelegramContractPath(
        "src-tauri/./Cargo.toml",
      ),
    ).toThrow(/dot path segment/);
    expect(() =>
      telegramContractPaths.resolveTelegramContractPath(
        "src-tauri/src/not-a-real-telegram-file.rs",
      ),
    ).toThrow(/missing/);
  });

  it("resolves the four closed layouts without asserting future staged paths early", () => {
    const dto = {
      baselinePath: "src-tauri/src/sources/types.rs",
      stagedPath: "src-tauri/src/telegram_impl/dto.rs",
      finalOwner: "extractum-telegram" as const,
    };
    const media = {
      baselinePath: "src-tauri/src/media.rs",
      stagedPath: "src-tauri/src/telegram_impl/media.rs",
      finalOwner: "extractum-telegram" as const,
    };
    const session = {
      baselinePath: "src-tauri/src/telegram_session_store.rs",
      stagedPath: "src-tauri/src/telegram_impl/session.rs",
      finalOwner: "extractum-telegram" as const,
    };
    const runtime = {
      baselinePath: "src-tauri/src/telegram.rs",
      stagedPath: "src-tauri/src/telegram_impl/runtime.rs",
      finalOwner: "extractum-telegram" as const,
    };
    const laterOwner = {
      baselinePath: "src-tauri/src/sources/peer_resolution.rs",
      stagedPath: "src-tauri/src/telegram_impl/live/peer.rs",
      finalOwner: "extractum-telegram" as const,
    };
    const appConsumer = {
      baselinePath: "src-tauri/src/ingest_provenance.rs",
      stagedPath: "src-tauri/src/ingest_provenance.rs",
      finalOwner: "extractum" as const,
    };

    for (const lifecycle of [
      "baseline",
      "8a-checkpoint-1",
      "8a-checkpoint-2",
    ] as const) {
      expect(
        telegramContractPaths.resolveTelegramLifecyclePath(dto, lifecycle),
      ).toBe(dto.baselinePath);
    }
    expect(
      telegramContractPaths.resolveTelegramLifecyclePath(
        dto,
        "8a-checkpoint-3",
      ),
    ).toBe("src-tauri/src/telegram/dto.rs");
    expect(
      telegramContractPaths.resolveTelegramLifecyclePath(
        media,
        "8a-checkpoint-3",
      ),
    ).toBe("src-tauri/src/telegram/media.rs");
    expect(
      telegramContractPaths.resolveTelegramLifecyclePath(
        session,
        "8a-checkpoint-3",
      ),
    ).toBe(session.baselinePath);
    expect(
      telegramContractPaths.resolveTelegramLifecyclePath(
        session,
        "8a-checkpoint-4",
      ),
    ).toBe("src-tauri/src/telegram/session.rs");
    expect(
      telegramContractPaths.resolveTelegramLifecyclePath(
        runtime,
        "8a-checkpoint-4",
      ),
    ).toBe(runtime.baselinePath);
    expect(
      telegramContractPaths.resolveTelegramLifecyclePath(
        runtime,
        "8a-checkpoint-5",
      ),
    ).toBe("src-tauri/src/telegram/runtime.rs");
    expect(
      telegramContractPaths.resolveTelegramLifecyclePath(
        laterOwner,
        "8a-retained",
      ),
    ).toBe(laterOwner.baselinePath);
    expect(
      telegramContractPaths.resolveTelegramLifecyclePath(
        laterOwner,
        "8b-preparation",
      ),
    ).toBe(laterOwner.stagedPath);
    expect(
      telegramContractPaths.resolveTelegramLifecyclePath(
        laterOwner,
        "8c-extracted",
      ),
    ).toBe(
      "src-tauri/crates/extractum-telegram/src/live/peer.rs",
    );
    expect(
      telegramContractPaths.resolveTelegramLifecyclePath(
        appConsumer,
        "8c-extracted",
      ),
    ).toBe(appConsumer.baselinePath);
    expect(() =>
      telegramContractPaths.resolveTelegramLifecyclePath(
        {
          ...laterOwner,
          stagedPath: "src-tauri/src/telegram_impl/../telegram.rs",
        },
        "8b-preparation",
      ),
    ).toThrow(/dot path segment/);
  });

  it("maps only the closed Phase 8 status vocabulary to lifecycle states", () => {
    expect(
      telegramContractPaths.telegramLifecycleFromStatus(
        "design approved; implementation not started",
      ),
    ).toBe("baseline");
    expect(
      telegramContractPaths.telegramLifecycleFromStatus(
        "8A preparation Checkpoint 1 retained",
      ),
    ).toBe("8a-checkpoint-1");
    expect(
      telegramContractPaths.telegramLifecycleFromStatus(
        "8A preparation Checkpoint 5 retained",
      ),
    ).toBe("8a-checkpoint-5");
    expect(
      telegramContractPaths.telegramLifecycleFromStatus(
        "8A preparation retained",
      ),
    ).toBe("8a-retained");
    expect(
      telegramContractPaths.telegramLifecycleFromStatus(
        "8B preparation retained; 8C pending",
      ),
    ).toBe("8b-preparation");
    expect(
      telegramContractPaths.telegramLifecycleFromStatus("done: retained"),
    ).toBe("8c-extracted");
    expect(() =>
      telegramContractPaths.telegramLifecycleFromStatus(
        "8A preparation Checkpoint 6 retained",
      ),
    ).toThrow(/Unsupported Phase 8 status/);
  });
});

describe("literal immutable Telegram test map", () => {
  it("records only the truthful retained Checkpoint 2 lifecycle state", () => {
    expect(phase8Status).toBe("8A preparation Checkpoint 2 retained");
    expect(design).toContain(
      "**Status:** Approved; 8A preparation Checkpoint 2 retained",
    );
  });

  it("parses the plan table without copied rows and freezes all identity totals", () => {
    expect(identityRows).toHaveLength(140);
    expect(
      identityRows.every(({ baselinePackage }) => baselinePackage === "extractum"),
    ).toBe(true);
    expect(
      new Set(identityRows.map(({ baselineFullId }) => baselineFullId)).size,
    ).toBe(140);
    expect(
      new Set(identityRows.map(({ finalFullId }) => finalFullId)).size,
    ).toBe(140);
    expect(
      identityRows.every(({ finalOwner }) =>
        ["extractum", "extractum-telegram"].includes(finalOwner),
      ),
    ).toBe(true);

    const appPrimaries = identityRows.filter(
      ({ finalOwner }) => finalOwner === "extractum",
    );
    const futureCratePrimaries = identityRows.filter(
      ({ finalOwner }) => finalOwner === "extractum-telegram",
    );
    const companions = identityRows.flatMap(
      ({ companionFinalIds }) => companionFinalIds,
    );
    expect(appPrimaries).toHaveLength(99);
    expect(futureCratePrimaries).toHaveLength(41);
    exactInventory(
      companions,
      [
        "dto::tests::telegram_item_kind_constant_matches_persisted_wire_value",
        "takeout::raw_parse::tests::raw_parse_preserves_distinct_history_peer_identity_for_equal_message_ids",
        "takeout::raw_parse::tests::raw_parse_preserves_identical_native_identity_for_same_peer_and_message_id",
      ],
      "three declared companion identities",
    );

    const eventualBaselineDerived = new Set([
      ...identityRows.map(({ finalFullId }) => finalFullId),
      ...companions,
    ]);
    expect(eventualBaselineDerived.size).toBe(143);
    expect(addedIdentities).toHaveLength(18);
    expect(
      new Set(addedIdentities.map(({ currentId }) => currentId)).size,
    ).toBe(18);
    const itemKindCompanion = addedIdentities.filter(({ currentId }) =>
      currentId.endsWith(
        "::telegram_item_kind_constant_matches_persisted_wire_value",
      ),
    );
    expect(itemKindCompanion).toHaveLength(1);
    expect(addedIdentities.length - itemKindCompanion.length).toBe(17);
    expect(identityRows.length + itemKindCompanion.length).toBe(141);
    expect(identityRows.length + addedIdentities.length).toBe(158);
    expect(
      new Set([
        ...eventualBaselineDerived,
        ...addedIdentities.map(({ finalId }) => finalId),
      ]).size,
    ).toBe(160);
  });

  it("rejects every malformed immutable and plan-added data row", () => {
    const immutableMutation = plan.replace(
      "| --- | --- | --- | --- | --- | --- |",
      [
        "| --- | --- | --- | --- | --- | --- |",
        "| `wrong-package` | `telegram::tests::malformed_extra` | `src-tauri/src/telegram.rs` | `extractum` | `telegram::tests::malformed_extra` | — |",
      ].join("\n"),
    );
    expect(() =>
      (parseIdentityRows as unknown as (source: string) => IdentityRow[])(
        immutableMutation,
      ),
    ).toThrow(/baseline package/);

    const addedMutation = plan.replace(
      [
        "| Checkpoint | Phase 8A exact identity in `extractum` | Future owner / final identity |",
        "| --- | --- | --- |",
      ].join("\n"),
      [
        "| Checkpoint | Phase 8A exact identity in `extractum` | Future owner / final identity |",
        "| --- | --- | --- |",
        "| 6 | `telegram::tests::forbidden_checkpoint` | app / unchanged |",
      ].join("\n"),
    );
    expect(() =>
      (parseAddedIdentities as unknown as (
        source: string,
      ) => AddedIdentity[])(addedMutation),
    ).toThrow(/checkpoint/);
  });

  it("rejects an app-owned immutable row staged away from its baseline", () => {
    const identity =
      "ingest_provenance::tests::completed_zero_observation_batch_is_complete_without_partial_flags";
    const rowPrefix =
      `| \`extractum\` | \`${identity}\` | \`src-tauri/src/ingest_provenance.rs\` | \`extractum\` |`;
    const mutation = plan.replace(
      rowPrefix,
      `| \`extractum\` | \`${identity}\` | \`src-tauri/src/telegram.rs\` | \`extractum\` |`,
    );
    expect(mutation).not.toBe(plan);
    expect(() => parseIdentityRows(mutation)).toThrow(
      /app-owned staged path/,
    );
  });

  it("rejects non-pipe plan-added body lines and open disposition values", () => {
    const firstAddedRow =
      "| 2 | `telegram::tests::telegram_status_and_event_payload_contract_is_exact` | app / unchanged |";
    const nonPipeMutation = plan.replace(
      firstAddedRow,
      `${firstAddedRow}\nmalformed non-pipe identity row`,
    );
    expect(() => parseAddedIdentities(nonPipeMutation)).toThrow(
      /non-pipe plan-added identity row/,
    );

    const dispositionMutation = plan.replace(
      firstAddedRow,
      firstAddedRow.replace("app / unchanged", "application / unchanged"),
    );
    expect(() => parseAddedIdentities(dispositionMutation)).toThrow(
      /disposition/,
    );
  });

  it("rejects a crate disposition with the wrong complete final identity", () => {
    const currentId =
      "telegram::dto::tests::telegram_message_draft_has_single_persistence_shape";
    const finalId =
      "dto::tests::telegram_message_draft_has_single_persistence_shape";
    const mutation = plan.replace(
      `crate / \`${finalId}\``,
      "crate / `wrong_module::tests::telegram_message_draft_has_single_persistence_shape`",
    );
    expect(mutation).not.toBe(plan);
    expect(() => parseAddedIdentities(mutation)).toThrow(
      new RegExp(`final qualified identity.*${currentId}`),
    );
  });

  it("rejects a plan-added current identity without a complete test module", () => {
    const currentId =
      "telegram::tests::telegram_status_and_event_payload_contract_is_exact";
    const mutation = plan.replace(
      `\`${currentId}\` | app / unchanged`,
      "`bogus` | app / unchanged",
    );
    expect(mutation).not.toBe(plan);
    expect(() => parseAddedIdentities(mutation)).toThrow(
      /module-qualified current identity/,
    );
  });

  it("resolves each row through the current lifecycle without requiring future paths", () => {
    assertIdentityDeclarations(identityRows, lifecycle);
  });

  it("rejects a matching leaf test under the wrong module-qualified identity", () => {
    const mutatedRows = identityRows.map((row) =>
      row.baselineFullId
        === "sources::types::tests::telegram_message_identity_validation_rejects_invalid_values"
        ? {
            ...row,
            baselineFullId:
              "sources::types::wrong_module::telegram_message_identity_validation_rejects_invalid_values",
          }
        : row,
    );
    expect(() =>
      assertIdentityDeclarations(mutatedRows, lifecycle),
    ).toThrow(/module-qualified identity/);
  });

  it("checkpoint-gates all 18 plan-added identities outside the immutable 140", () => {
    const allRustPaths = [
      ...rustPathsUnder("src-tauri/src"),
      ...rustPathsUnder("src-tauri/crates/extractum-telegram/src"),
    ];
    const rustSources = new Map(
      allRustPaths.map((relativePath) => [
        relativePath,
        telegramContractPaths.readTelegramContractFile(relativePath),
      ]),
    );
    assertAddedIdentityDeclarations(addedIdentities, lifecycle, rustSources);
  });

  it("rejects a plan-added leaf declaration under the wrong module", () => {
    const added = addedIdentities.find(
      ({ currentId }) =>
        currentId
        === "telegram::tests::telegram_status_and_event_payload_contract_is_exact",
    );
    if (!added) throw new Error("Missing Checkpoint 2 status identity");
    expect(() =>
      assertAddedIdentityDeclarations(
        [added],
        "8a-checkpoint-2",
        new Map([
          [
            "src-tauri/src/wrong_module.rs",
            "fn telegram_status_and_event_payload_contract_is_exact() {}",
          ],
        ]),
      ),
    ).toThrow(/module-qualified plan-added declarations/);
  });

  it("tracks a crate plan-added identity through 8A, 8B, and 8C modules", () => {
    const added = addedIdentities.find(
      ({ currentId }) =>
        currentId
        === "telegram::dto::tests::telegram_message_draft_has_single_persistence_shape",
    );
    if (!added) throw new Error("Missing Checkpoint 3 DTO identity");
    const declaration =
      "fn telegram_message_draft_has_single_persistence_shape() {}";
    assertAddedIdentityDeclarations(
      [added],
      "8a-checkpoint-3",
      new Map([["src-tauri/src/telegram/dto.rs", declaration]]),
    );
    assertAddedIdentityDeclarations(
      [added],
      "8b-preparation",
      new Map([["src-tauri/src/telegram_impl/dto.rs", declaration]]),
    );
    assertAddedIdentityDeclarations(
      [added],
      "8c-extracted",
      new Map([
        [
          "src-tauri/crates/extractum-telegram/src/dto.rs",
          declaration,
        ],
      ]),
    );
  });

  it("freezes the independent 24-test store suite without polluting the 140", () => {
    expect(storeIdentities).toHaveLength(24);
    expect(new Set(storeIdentities).size).toBe(24);
    expect(
      storeIdentities.every((identity) =>
        identity.startsWith("sources::store::tests::"),
      ),
    ).toBe(true);
    const baselineIds = new Set(
      identityRows.map(({ baselineFullId }) => baselineFullId),
    );
    expect(
      storeIdentities.filter((identity) => baselineIds.has(identity)),
    ).toEqual([]);

    const storeSource =
      telegramContractPaths.readTelegramContractFile(
        "src-tauri/src/sources/store.rs",
      );
    for (const identity of storeIdentities) {
      const name = identity.slice(identity.lastIndexOf("::") + 2);
      expect(
        countMatches(
          storeSource,
          new RegExp(`\\b(?:async\\s+)?fn\\s+${name}\\s*\\(`, "g"),
        ),
        `${identity} declared exactly once`,
      ).toBe(1);
      expect(rustFunctionBody(storeSource, name)).not.toMatch(
        /\b(?:list_telegram_sources|add_telegram_source)\s*\(/,
      );
    }
  });
});

describe("approved Telegram identity ownership", () => {
  it("keeps the exact 43 helper-dependent and three credential-SQL identities app-owned", () => {
    const fixtureIdentities = parseFixtureIdentitySet();
    expect(fixtureIdentities).toHaveLength(43);
    expect(new Set(fixtureIdentities).size).toBe(43);

    const credentialSection = sectionBetween(
      design,
      "three credential SQL identities also remain in `extractum`",
      "the remaining 73 identities",
    );
    const credentialNames = [
      ...credentialSection.matchAll(/`([A-Za-z0-9_]+)`/g),
    ].map((match) => match[1]).filter((name) => name !== "extractum");
    exactInventory(
      credentialNames,
      [
        "legacy_api_hash_migrates_to_secret_store_and_blanks_column",
        "legacy_api_hash_remains_when_secret_write_fails",
        "missing_secure_api_hash_for_blank_legacy_account_is_auth_error",
      ],
      "credential SQL names parsed from approved design",
    );
    const credentialIdentities = credentialNames.map(
      (name) => `telegram::tests::${name}`,
    );
    const byBaseline = new Map(
      identityRows.map((row) => [row.baselineFullId, row]),
    );
    for (const identity of [...fixtureIdentities, ...credentialIdentities]) {
      expect(byBaseline.get(identity)?.finalOwner, identity).toBe("extractum");
    }
  });

  it("individually accounts for all 119 direct-perimeter identities and the residual 73", () => {
    const fixtureIdentities = new Set(parseFixtureIdentitySet());
    const credentialIdentities = new Set([
      "telegram::tests::legacy_api_hash_migrates_to_secret_store_and_blanks_column",
      "telegram::tests::legacy_api_hash_remains_when_secret_write_fails",
      "telegram::tests::missing_secure_api_hash_for_blank_legacy_account_is_auth_error",
    ]);
    const directRows = identityRows.filter((row) =>
      directGrammersPaths.includes(
        baselineSourcePath(row.baselineFullId) as
          (typeof directGrammersPaths)[number],
      ),
    );
    expect(directRows).toHaveLength(119);
    const residual = directRows.filter(
      ({ baselineFullId }) =>
        !fixtureIdentities.has(baselineFullId)
        && !credentialIdentities.has(baselineFullId),
    );
    expect(residual).toHaveLength(73);
    expect(
      new Set(residual.map(({ baselineFullId }) => baselineFullId)).size,
    ).toBe(73);
  });

  it("freezes the 21-identity transitive type closure and its one mixed companion", () => {
    const closureGroups = {
      "sources::types::tests::": 8,
      "ingest_provenance::tests::": 7,
      "takeout_import::migrated_history::tests::": 6,
    };
    const closureRows = identityRows.filter(({ baselineFullId }) =>
      Object.keys(closureGroups).some((prefix) =>
        baselineFullId.startsWith(prefix),
      ),
    );
    expect(closureRows).toHaveLength(21);
    for (const [prefix, count] of Object.entries(closureGroups)) {
      expect(
        closureRows.filter(({ baselineFullId }) =>
          baselineFullId.startsWith(prefix),
        ),
        prefix,
      ).toHaveLength(count);
    }
    expect(
      closureRows.filter(({ finalOwner }) => finalOwner === "extractum"),
    ).toHaveLength(20);
    expect(
      closureRows.filter(
        ({ finalOwner }) => finalOwner === "extractum-telegram",
      ).map(({ baselineFullId }) => baselineFullId),
    ).toEqual([
      "sources::types::tests::telegram_message_identity_validation_rejects_invalid_values",
    ]);
    expect(
      closureRows.flatMap(({ companionFinalIds }) => companionFinalIds),
    ).toEqual([
      "dto::tests::telegram_item_kind_constant_matches_persisted_wire_value",
    ]);
  });

  it("preserves the three declared decompositions and exact retained identities", () => {
    const row = (baselineFullId: string): IdentityRow => {
      const selected = identityRows.find(
        (candidate) => candidate.baselineFullId === baselineFullId,
      );
      if (!selected) throw new Error(`Missing identity row: ${baselineFullId}`);
      return selected;
    };

    expect(
      row(
        "takeout_import::tests::takeout_parsed_items_with_same_message_id_insert_under_different_history_peers",
      ).companionFinalIds,
    ).toEqual([
      "takeout::raw_parse::tests::raw_parse_preserves_distinct_history_peer_identity_for_equal_message_ids",
    ]);
    expect(
      row(
        "takeout_import::tests::takeout_duplicate_parsed_item_updates_topic_unresolved_count_once",
      ).companionFinalIds,
    ).toEqual([
      "takeout::raw_parse::tests::raw_parse_preserves_identical_native_identity_for_same_peer_and_message_id",
    ]);
    expect(
      row(
        "sources::types::tests::item_kind_constants_match_persisted_wire_values",
      ).companionFinalIds,
    ).toEqual([
      "dto::tests::telegram_item_kind_constant_matches_persisted_wire_value",
    ]);
    expect(
      row(
        "sources::items::tests::insert_source_item_writes_payload_and_skips_duplicates",
      ).finalFullId,
    ).toBe(
      "sources::items::tests::insert_telegram_source_item_writes_payload_and_skips_duplicates",
    );
    const mediaMetadata = row(
      "sources::items::tests::media_metadata_roundtrip_through_zstd",
    );
    expect(mediaMetadata.finalOwner).toBe("extractum");
    expect(mediaMetadata.finalFullId).toBe(mediaMetadata.baselineFullId);
  });
});

describe("frozen Telegram source ownership perimeter", () => {
  it("freezes the exact raw-consumer inventory for every lifecycle layout", () => {
    const inventory = (relativePaths: readonly string[]) =>
      new Map(
        relativePaths.map((relativePath) => [
          relativePath,
          "grammers_client",
        ]),
      );
    const checkpoint3 = checkpointOneRawConsumerPaths.map((relativePath) =>
      relativePath === "src-tauri/src/media.rs"
        ? "src-tauri/src/telegram/media.rs"
        : relativePath,
    );
    const checkpoint4 = checkpoint3.map((relativePath) =>
      relativePath === "src-tauri/src/telegram_session_store.rs"
        ? "src-tauri/src/telegram/session.rs"
        : relativePath,
    );
    const checkpoint5 = checkpoint4.map((relativePath) =>
      relativePath === "src-tauri/src/telegram.rs"
        ? "src-tauri/src/telegram/runtime.rs"
        : relativePath,
    );

    assertRawConsumerInventory(
      "8a-checkpoint-1",
      inventory(checkpointOneRawConsumerPaths),
    );
    assertRawConsumerInventory(
      "8a-checkpoint-3",
      inventory(checkpoint3),
    );
    assertRawConsumerInventory(
      "8a-checkpoint-4",
      inventory(checkpoint4),
    );
    assertRawConsumerInventory(
      "8a-checkpoint-5",
      inventory(checkpoint5),
    );
    assertRawConsumerInventory(
      "8a-retained",
      inventory(checkpoint5),
    );
    assertRawConsumerInventory(
      "8b-preparation",
      inventory(stagedRawConsumerPaths),
    );
    assertRawConsumerInventory(
      "8c-extracted",
      new Map(),
      inventory(crateRawConsumerPaths),
      true,
    );
    expect(() =>
      assertRawConsumerInventory(
        "8b-preparation",
        inventory(checkpointOneRawConsumerPaths),
      ),
    ).toThrow(/8B staged raw consumer paths/);
  });

  it("rejects app raw access, missing manifest, and unowned crate raw access at 8C", () => {
    expect(() =>
      assertRawConsumerInventory(
        "8c-extracted",
        new Map([["src-tauri/src/telegram.rs", "grammers_client"]]),
        new Map(),
        true,
      ),
    ).toThrow(/8C app raw consumer paths/);

    expect(() =>
      assertRawConsumerInventory(
        "8c-extracted",
        new Map(),
        new Map(),
        false,
      ),
    ).toThrow(/8C crate manifest/);

    const crateSources = new Map(
      crateRawConsumerPaths.map((relativePath) => [
        relativePath,
        "grammers_client",
      ]),
    );
    crateSources.set(
      "src-tauri/crates/extractum-telegram/src/unowned.rs",
      "grammers_client",
    );
    expect(() =>
      assertRawConsumerInventory(
        "8c-extracted",
        new Map(),
        crateSources,
        true,
      ),
    ).toThrow(/8C crate raw consumer paths/);
  });

  it("keeps direct Grammers discovery, moved-symbol discovery, and wiring disjoint", () => {
    const appRustPaths = rustPathsUnder("src-tauri/src");
    const sources = new Map(
      appRustPaths.map((relativePath) => [
        relativePath,
        telegramContractPaths.readTelegramContractFile(relativePath),
      ]),
    );
    const crateSources = new Map(
      rustPathsUnder("src-tauri/crates/extractum-telegram/src").map(
        (relativePath) => [
          relativePath,
          telegramContractPaths.readTelegramContractFile(relativePath),
        ],
      ),
    );
    assertRawConsumerInventory(
      lifecycle,
      sources,
      crateSources,
      existsSync(
        path.join(
          repoRoot,
          "src-tauri/crates/extractum-telegram/Cargo.toml",
        ),
      ),
    );
    const actualDirect = appRustPaths.filter((relativePath) =>
      /grammers_(?:client|session|mtsender|tl_types)/.test(
        sources.get(relativePath) ?? "",
      ),
    );

    const fanPattern =
      /(?:\b(?:get_client|get_authorized_runtime|AuthorizedTelegramRuntime|AccountClient|raw_client|raw_session|MemorySession|LoginToken|TelegramMessageIdentity|TelegramItemContext|SourceItemInsert|ExtractedItemPayload|ExtractedMediaPayload|ITEM_KIND_TELEGRAM_MESSAGE)\b|\.accounts\s*\.lock\s*\(\s*\)\s*\.await\b)/;
    const fanPaths = appRustPaths.filter((relativePath) =>
      fanPattern.test(sources.get(relativePath) ?? ""),
    );
    const actualDiscovered = [...new Set([...actualDirect, ...fanPaths])];
    if (checkpointNumber(lifecycle) <= 2) {
      exactInventory(
        actualDirect,
        directGrammersPaths,
        "direct Grammers source paths",
      );
      exactInventory(
        actualDiscovered,
        symbolDiscoveredPaths,
        "19 symbol-discovered paths",
      );
      expect(actualDiscovered).toHaveLength(19);
      exactInventory(
        [...actualDiscovered, ...wiringPaths],
        [...symbolDiscoveredPaths, ...wiringPaths],
        "20-path discovered plus wiring union",
      );
    } else {
      const allowedLaterPaths = [
        ...symbolDiscoveredPaths,
        ...rustPathsUnder("src-tauri/src/telegram"),
        ...rustPathsUnder("src-tauri/src/telegram_impl"),
      ];
      expect(
        actualDiscovered.filter(
          (relativePath) => !allowedLaterPaths.includes(relativePath),
        ),
        "unexpected raw/moved-symbol consumer outside closed lifecycle roots",
      ).toEqual([]);
    }
    expect(
      actualDiscovered.filter((relativePath) =>
        wiringPaths.includes(relativePath as (typeof wiringPaths)[number]),
      ),
    ).toEqual([]);
    expect(ownershipMovePaths).toHaveLength(19);
    expect(
      actualDiscovered.filter(
        (relativePath) => relativePath === "src-tauri/src/sources/store.rs",
      ),
    ).toEqual(["src-tauri/src/sources/store.rs"]);
  });

  it("rejects a new raw alias, re-export, wrapper signature, or caller-lock consumer", () => {
    const appRustPaths = rustPathsUnder("src-tauri/src");
    const aliasOrReexport =
      /(?:use\s+grammers_[^;]+\bas\s+\w+|(?:pub(?:\([^)]*\))?\s+)?type\s+\w+\s*=\s*(?:grammers_|Client\b|MemorySession\b)|pub(?:\([^)]*\))?\s+use\s+grammers_)/g;
    const aliasMatches = appRustPaths.flatMap((relativePath) => {
      const source =
        telegramContractPaths.readTelegramContractFile(relativePath);
      return [...source.matchAll(aliasOrReexport)].map(
        (match) => `${relativePath}:${normalize(match[0]).trim()}`,
      );
    });
    expect(aliasMatches).toEqual([]);

    const explicitRawSignature =
      /pub(?:\([^)]*\))?\s+(?:async\s+)?fn\s+\w+[^\n]*grammers_[^\n{]+/g;
    const rawSignatureMatches = appRustPaths.flatMap((relativePath) => {
      const source =
        telegramContractPaths.readTelegramContractFile(relativePath);
      return [...source.matchAll(explicitRawSignature)].map(
        (match) => `${relativePath}:${match[0].replace(/\s+/g, " ").trim()}`,
      );
    });
    if (checkpointNumber(lifecycle) <= 2) {
      expect(rawSignatureMatches).toEqual([
        "src-tauri/src/sources/items.rs:pub(super) fn message_author(message: &grammers_client::message::Message) -> Option<String>",
      ]);
    } else {
      expect(
        rawSignatureMatches.filter((match) => {
          const relativePath = match.slice(0, match.indexOf(":"));
          return !directGrammersPaths.includes(
            relativePath as (typeof directGrammersPaths)[number],
          )
            && !relativePath.startsWith("src-tauri/src/telegram/")
            && !relativePath.startsWith("src-tauri/src/telegram_impl/");
        }),
        "unexpected explicit raw signature outside closed owner roots",
      ).toEqual([]);
    }
  });

  it("requires the synchronized dependent-only correction in both authorities", () => {
    const correction = (source: string): string => {
      const start = source.indexOf(
        "The immutable ownership/move surface remains 19 paths / 140 tests.",
      );
      const end = source.indexOf("\n\n", start);
      if (start < 0 || end <= start) {
        throw new Error("Missing Task 0 dependent-only correction");
      }
      return source.slice(start, end);
    };
    expect(correction(design)).toBe(correction(roadmap));
    const compactCorrection = correction(design).replace(/\s+/g, " ");
    expect(compactCorrection).toContain(
      "`src-tauri/src/sources/store.rs` is the sole known dependent-only raw-client",
    );
    expect(compactCorrection).toContain(
      "19 ownership/move paths plus one dependent-only consumer with 24 regressions",
    );
    expect(compactCorrection).toContain(
      "none directly invokes `list_telegram_sources` or `add_telegram_source`",
    );
  });
});

describe("Checkpoint 1 core-error seam", () => {
  it("retains the immutable 166-reference evidence and requires the 160-reference live layout", () => {
    expect(
      Object.values(baselineRootCounts).reduce(
        (total, count) => total + count,
        0,
      ),
    ).toBe(166);

    assertCheckpointOneRootInventory(lifecycle);
  });

  it("rejects a premature owner layout at retained Checkpoint 1", () => {
    expect(() =>
      assertCheckpointOneRootInventory(
        "8a-checkpoint-1",
        (relativePath) => relativePath === "src-tauri/src/telegram_impl",
      ),
    ).toThrow(/premature/);
  });

  it("does not read unavailable Checkpoint 1 paths after Checkpoint 2", () => {
    const mutableOwnershipPaths = ownershipMovePaths as string[];
    mutableOwnershipPaths.push(
      "src-tauri/src/unavailable-after-checkpoint-2.rs",
    );
    try {
      expect(() =>
        assertCheckpointOneRootInventory("8a-checkpoint-3"),
      ).not.toThrow();
    } finally {
      mutableOwnershipPaths.pop();
    }
  });

  it("normalizes exactly the six identity-validation paths directly to core", () => {
    const identityRow = identityRows.find(
      ({ baselineFullId }) =>
        baselineFullId
        === "sources::types::tests::telegram_message_identity_validation_rejects_invalid_values",
    );
    if (!identityRow) throw new Error("Missing identity validation map row");
    const identityPath =
      telegramContractPaths.resolveTelegramLifecyclePath(
        {
          baselinePath: baselineSourcePath(identityRow.baselineFullId),
          stagedPath: identityRow.stagedPath,
          finalOwner: identityRow.finalOwner as
            | "extractum"
            | "extractum-telegram",
        },
        lifecycle,
      );
    const source =
      telegramContractPaths.readTelegramContractFile(identityPath);
    const directCoreCounts = {
      result: countMatches(
        source,
        /\bextractum_core::error::AppResult\b/g,
      ),
      constructors: countMatches(
        source,
        /\bextractum_core::error::AppError::validation\b/g,
      ),
      kindAssertions: countMatches(
        source,
        /\bextractum_core::error::AppErrorKind::Validation\b/g,
      ),
    };
    expect(directCoreCounts).toEqual({
      result: 1,
      constructors: 3,
      kindAssertions: 2,
    });
    expect(
      Object.values(directCoreCounts).reduce(
        (total, count) => total + count,
        0,
      ),
    ).toBe(6);

    const validationBody = rustFunctionBody(source, "validate");
    const characterizationBody = rustFunctionBody(
      source,
      "telegram_message_identity_validation_rejects_invalid_values",
    );
    const characterizationOriginalBody = rustFunctionOriginalBody(
      source,
      "telegram_message_identity_validation_rejects_invalid_values",
    );
    expect(validationBody).not.toContain("crate::error");
    expect(characterizationBody).not.toContain("crate::error");
    expect(characterizationOriginalBody).toContain(
      "Unsupported Telegram history peer kind 'supergroup'",
    );
    expect(characterizationOriginalBody).toContain(
      "Telegram history peer id must be positive",
    );
    expect(characterizationOriginalBody).toContain(
      "Telegram message id must be positive",
    );
    expect(characterizationOriginalBody).toContain('"kind":"validation"');
    expect(characterizationOriginalBody).toContain("multiple invalid values");
  });

  it("sentinel-preserves the other 44 core-facade references", () => {
    assertFacadeInventory();

    const types =
      telegramContractPaths.readTelegramContractFile(
        "src-tauri/src/sources/types.rs",
      );
    expect(
      rustFunctionBody(types, "from_source_subtype"),
    ).toContain("crate::error::AppError::validation");
    expect(rustFunctionBody(types, "encode_opaque")).toContain(
      "crate::error::AppError::internal",
    );
    expect(rustFunctionBody(types, "decode_opaque")).toContain(
      "crate::error::AppError::validation",
    );
    expect(
      telegramContractPaths.readTelegramContractFile(
        "src-tauri/src/sources/items.rs",
      ),
    ).toContain("crate::compression");
    expect(
      telegramContractPaths.readTelegramContractFile(
        "src-tauri/src/sources/peer_resolution.rs",
      ),
    ).toContain("crate::compression");
    expect(
      telegramContractPaths.readTelegramContractFile(
        "src-tauri/src/sources/sync.rs",
      ),
    ).toContain("crate::compression");
    expect(
      telegramContractPaths.readTelegramContractFile(
        "src-tauri/src/takeout_import/mod.rs",
      ),
    ).toContain("crate::time::now_secs");
  });

  it("keeps semantic facade sites stable across Checkpoint 3 line shifts", () => {
    const typesPath = "src-tauri/src/sources/types.rs";
    const source =
      telegramContractPaths.readTelegramContractFile(typesPath);
    const mediaSource =
      telegramContractPaths.readTelegramContractFile(
        "src-tauri/src/media.rs",
      );
    const checkpoint3Source =
      `// Checkpoint 3 DTO split shifts retained source lines.\n${source}`;
    expect(() =>
      assertFacadeInventory(
        new Map([
          [typesPath, checkpoint3Source],
          ["src-tauri/src/telegram/media.rs", mediaSource],
        ]),
        "8a-checkpoint-3",
      ),
    ).not.toThrow();
  });

  it("rejects moving a facade reference while preserving aggregate counts", () => {
    const typesPath = "src-tauri/src/sources/types.rs";
    const source =
      telegramContractPaths.readTelegramContractFile(typesPath);
    const relocated = source
      .replace("crate::error", "crate::errors")
      .replace("extractum_core::error", "crate::error");
    expect(countMatches(relocated, /\bcrate::error\b/g)).toBe(
      countMatches(source, /\bcrate::error\b/g),
    );
    expect(() =>
      assertFacadeInventory(new Map([[typesPath, relocated]])),
    ).toThrow(/site inventory/);
  });
});

describe("Checkpoint 2 observable Telegram and Takeout behavior", () => {
  it("pins all twelve command declarations, registration entries, and default camelCase IPC keys", () => {
    const accountsPath = "src-tauri/src/accounts.rs";
    const telegramPath = "src-tauri/src/telegram.rs";
    const accounts =
      telegramContractPaths.readTelegramContractFile(accountsPath);
    const telegram =
      telegramContractPaths.readTelegramContractFile(telegramPath);
    const lib =
      telegramContractPaths.readTelegramContractFile("src-tauri/src/lib.rs");
    const commands = [
      {
        name: "list_accounts",
        source: accounts,
        declaration:
          "#[tauri::command] pub async fn list_accounts(handle: AppHandle) -> AppResult<Vec<AccountRecord>>",
      },
      {
        name: "get_account",
        source: accounts,
        declaration:
          "#[tauri::command] pub async fn get_account(handle: AppHandle, account_id: i64) -> AppResult<Option<AccountRecord>>",
      },
      {
        name: "create_account",
        source: accounts,
        declaration:
          "#[tauri::command] pub async fn create_account(handle: AppHandle, secret_store: tauri::State<'_, SecretStoreState>, label: String, api_id: i64, api_hash: String,) -> AppResult<AccountRecord>",
      },
      {
        name: "set_account_phone",
        source: accounts,
        declaration:
          "#[tauri::command] pub async fn set_account_phone(handle: AppHandle, account_id: i64, phone: String) -> AppResult<()>",
      },
      {
        name: "clear_account_phone",
        source: accounts,
        declaration:
          "#[tauri::command] pub async fn clear_account_phone(handle: AppHandle, account_id: i64) -> AppResult<()>",
      },
      {
        name: "delete_account",
        source: accounts,
        declaration:
          "#[tauri::command] pub async fn delete_account(handle: AppHandle, state: tauri::State<'_, TelegramState>, source_locks: tauri::State<'_, SourceIngestLocks>, takeout_state: tauri::State<'_, TakeoutImportState>, source_job_state: tauri::State<'_, SourceJobState>, analysis_state: tauri::State<'_, AnalysisState>, llm_scheduler: tauri::State<'_, Arc<LlmSchedulerState>>, secret_store: tauri::State<'_, SecretStoreState>, account_id: i64,) -> AppResult<()>",
      },
      {
        name: "tg_init",
        source: telegram,
        declaration:
          "#[tauri::command] pub async fn tg_init(handle: AppHandle, state: tauri::State<'_, TelegramState>, secret_store: tauri::State<'_, SecretStoreState>, account_id: i64,) -> AppResult<bool>",
      },
      {
        name: "tg_is_authenticated",
        source: telegram,
        declaration:
          "#[tauri::command] pub async fn tg_is_authenticated(state: tauri::State<'_, TelegramState>, account_id: i64,) -> AppResult<bool>",
      },
      {
        name: "tg_get_account_statuses",
        source: telegram,
        declaration:
          "#[tauri::command] pub async fn tg_get_account_statuses(state: tauri::State<'_, TelegramState>, account_ids: Vec<i64>,) -> AppResult<Vec<AccountRuntimeStatus>>",
      },
      {
        name: "tg_send_code",
        source: telegram,
        declaration:
          "#[tauri::command] pub async fn tg_send_code(state: tauri::State<'_, TelegramState>, account_id: i64, phone: String,) -> AppResult<String>",
      },
      {
        name: "tg_sign_in",
        source: telegram,
        declaration:
          "#[tauri::command] pub async fn tg_sign_in(handle: AppHandle, state: tauri::State<'_, TelegramState>, secret_store: tauri::State<'_, SecretStoreState>, account_id: i64, code: String,) -> AppResult<bool>",
      },
      {
        name: "tg_logout",
        source: telegram,
        declaration:
          "#[tauri::command] pub async fn tg_logout(handle: AppHandle, state: tauri::State<'_, TelegramState>, secret_store: tauri::State<'_, SecretStoreState>, account_id: i64,) -> AppResult<bool>",
      },
    ] as const;

    const registrations = activeInvokeHandlerRegistrations(lib);
    expect(registrations).toHaveLength(1);
    const handler = registrations[0];

    expect(commands.map(({ name }) => name)).toEqual([
      "list_accounts",
      "get_account",
      "create_account",
      "set_account_phone",
      "clear_account_phone",
      "delete_account",
      "tg_init",
      "tg_is_authenticated",
      "tg_get_account_statuses",
      "tg_send_code",
      "tg_sign_in",
      "tg_logout",
    ]);
    expect(
      commands.map(({ name, source }) => [
        name,
        rustCommandIpcKeys(
          normalizedRustCommandDeclaration(source, name),
          name,
        ),
      ]),
    ).toEqual([
      ["list_accounts", []],
      ["get_account", ["accountId"]],
      ["create_account", ["label", "apiId", "apiHash"]],
      ["set_account_phone", ["accountId", "phone"]],
      ["clear_account_phone", ["accountId"]],
      ["delete_account", ["accountId"]],
      ["tg_init", ["accountId"]],
      ["tg_is_authenticated", ["accountId"]],
      ["tg_get_account_statuses", ["accountIds"]],
      ["tg_send_code", ["accountId", "phone"]],
      ["tg_sign_in", ["accountId", "code"]],
      ["tg_logout", ["accountId"]],
    ]);
    for (const { name, source, declaration } of commands) {
      expect(normalizedRustCommandDeclaration(source, name)).toBe(declaration);
      expect(countMatches(handler, new RegExp(`\\b${name}\\b`, "g"))).toBe(1);
    }

    const accountsStructure = maskRustLexicalNonCode(accounts);
    const telegramStructure = maskRustLexicalNonCode(telegram);
    expect(accounts).toContain(
      "SELECT id, label, api_id, phone, created_at FROM accounts",
    );
    expect(accountsStructure).toContain("check_account_deletion(");
    expect(accountsStructure).toContain("source_locks.inner()");
    expect(accountsStructure).toContain("takeout_state.inner()");
    expect(accountsStructure).toContain("source_job_state.inner()");
    expect(accountsStructure).toContain("analysis_state.inner()");
    expect(accountsStructure).toContain("llm_scheduler.inner().as_ref()");
    expect(telegramStructure).toContain(
      "async fn resolve_account_credentials(",
    );
    expect(telegram).toContain("SELECT id, api_id, api_hash FROM accounts");

    const futureOwnerLeaves = rustPathsUnder("src-tauri/src/telegram").map(
      (relativePath) =>
        telegramContractPaths.readTelegramContractFile(relativePath),
    ).join("\n");
    const futureOwnerStructure = maskRustLexicalNonCode(futureOwnerLeaves);
    expect(futureOwnerStructure).not.toMatch(/#\[tauri::command\]/);
    expect(futureOwnerStructure).not.toMatch(
      /\b(?:list_accounts|get_account|create_account|set_account_phone|clear_account_phone|delete_account|tg_init|tg_is_authenticated|tg_get_account_statuses|tg_send_code|tg_sign_in|tg_logout)\s*\(/,
    );
    expect(futureOwnerStructure).not.toMatch(
      /\b(?:sqlx|check_account_deletion|generate_handler|AccountCredentials|resolve_account_credentials)\b/,
    );
  });

  it("pins Telegram event emission, status mutation, login result, and session ordering", () => {
    const telegram =
      telegramContractPaths.readTelegramContractFile(
        "src-tauri/src/telegram.rs",
      );
    const session =
      telegramContractPaths.readTelegramContractFile(
        "src-tauri/src/telegram_session_store.rs",
      );

    expectOrdered(
      rustFunctionBody(telegram, "set_account_status"),
      [
        "let runtime_status = AccountRuntimeStatus",
        "let mut statuses = state.statuses.lock().await",
        "statuses.insert(account_id, runtime_status.clone())",
        "drop(statuses)",
        "let _ = handle.emit(TELEGRAM_ACCOUNT_STATUS_EVENT, &runtime_status)",
      ],
      "account status state-before-best-effort-event order",
    );
    const restoreBody = rustFunctionBody(
      telegram,
      "restore_telegram_accounts",
    );
    expect(
      countMatches(
        restoreBody,
        /let _ = handle\.emit\(\s*TELEGRAM_RESTORE_FAILURE_EVENT/g,
      ),
    ).toBe(2);
    expectOrdered(
      rustFunctionBody(telegram, "init_account_client"),
      [
        "STATUS_RESTORING",
        "load_session(handle, secret_store, account_id).await?",
        "is_authorized()",
        "accounts.insert(",
        "STATUS_READY",
        "STATUS_REAUTH_REQUIRED",
        "set_account_status(handle, state, account_id, status, None).await",
        "Ok(is_auth)",
      ],
      "restore and initialization status order",
    );
    const sendCodeBody = rustFunctionBody(telegram, "tg_send_code");
    const sendCodeOriginal = rustFunctionOriginalBody(
      telegram,
      "tg_send_code",
    );
    expectOrdered(
      sendCodeBody,
      [
        "AppError::auth(",
        "request_login_code(&phone, &ac.api_hash.clone())",
        ".map_err(AppError::telegram_network)?",
        "ac.phone = Some(phone)",
        "ac.login_token = Some(token)",
        "Ok(",
      ],
      "send-code remote and state order",
    );
    expect(
      normalizedActiveCallArguments(sendCodeOriginal, "AppError::auth"),
    ).toEqual(['"Account not initialized"']);
    expect(normalizedActiveCallArguments(sendCodeOriginal, "Ok")).toEqual([
      '"Code sent".to_string()',
    ]);
    const signInBody = rustFunctionBody(telegram, "tg_sign_in");
    const signInOriginal = rustFunctionOriginalBody(telegram, "tg_sign_in");
    expectOrdered(
      signInBody,
      [
        "AppError::auth(",
        "AppError::auth(",
        ".sign_in(token, &code)",
        ".map_err(AppError::telegram_network)?",
        "ac.login_token = None",
        "save_session(&handle, &secret_store, account_id, &session_to_save)",
        "set_account_status(&handle, &state, account_id, STATUS_READY, None).await",
        "Ok(true)",
      ],
      "sign-in remote-save-status-result order",
    );
    expect(
      normalizedActiveCallArguments(signInOriginal, "AppError::auth"),
    ).toEqual([
      '"Account not initialized"',
      '"Call tg_send_code first"',
    ]);
    expect(normalizedActiveCallArguments(signInOriginal, "Ok")).toEqual([
      "true",
    ]);
    const credentialsBody = rustFunctionBody(
      telegram,
      "resolve_account_credentials",
    );
    const credentialsOriginal = rustFunctionOriginalBody(
      telegram,
      "resolve_account_credentials",
    );
    expect(credentialsBody).toContain("AppError::auth(format!(");
    expect(
      normalizedActiveCallArguments(
        credentialsOriginal,
        "AppError::auth",
      ),
    ).toEqual([
      'format!( "Telegram API hash for account {} is missing from secure storage. Recreate the account credentials.", credentials.id )',
    ]);
    expect(rustFunctionBody(telegram, "get_client")).toContain(
      "AppError::auth(format!(",
    );
    expect(
      normalizedActiveCallArguments(
        rustFunctionOriginalBody(telegram, "get_client"),
        "AppError::auth",
      ),
    ).toEqual([
      'format!("Account {account_id} not initialized")',
    ]);
    const authorizedRuntimeBody = rustFunctionBody(
      telegram,
      "get_authorized_runtime",
    ).replace(/\s+/g, " ");
    expect(authorizedRuntimeBody).toContain("AppError::auth(format!(");
    assertAuthorizedRuntimeAuthArguments(telegram);
    expect(
      normalizedActiveCallArguments(
        rustFunctionOriginalBody(telegram, "tg_logout"),
        "Ok",
      ),
    ).toEqual(["true"]);
    expectOrdered(
      rustFunctionBody(telegram, "tg_logout"),
      ["clear_account_runtime(", "Ok(true)"],
      "logout clear-result ownership",
    );
    expectOrdered(
      rustFunctionBody(telegram, "clear_account_runtime"),
      [
        "accounts.remove(&account_id)",
        "ac.client.sign_out().await",
        "runner.abort()",
        "delete_session(handle, secret_store, account_id).await?",
        "STATUS_NOT_INITIALIZED",
        "Ok(())",
      ],
      "logout client-session-status order",
    );

    const sessionStructure = maskRustLexicalNonCode(session);
    const sessionPathBody = rustFunctionBody(session, "session_path");
    const sessionPathOriginal = rustFunctionOriginalBody(
      session,
      "session_path",
    );
    expect(sessionPathBody).toContain("Ok(app_dir.join(format!(");
    expect(
      normalizedActiveCallArguments(sessionPathOriginal, "format!"),
    ).toEqual(['"telegram_{account_id}.session.json"']);
    expect(countMatches(sessionStructure, /\bsession_temp_path\s*\(/g)).toBe(3);
    expect(countMatches(
      sessionStructure,
      /path\.with_extension\s*\(/g,
    )).toBe(1);
    expectOrdered(
      rustFunctionBody(session, "write_atomic"),
      [
        "let tmp_path = session_temp_path(path)",
        "fs::write(&tmp_path, contents)",
        "fs::rename(&tmp_path, path)",
      ],
      "session write-before-rename order",
    );
    expect(
      countMatches(
        rustFunctionBody(session, "write_atomic"),
        /map_err\(\|error\| AppError::internal\(error\.to_string\(\)\)\)/g,
      ),
    ).toBe(2);
    expectOrdered(
      rustFunctionBody(session, "write_encrypted_session_file"),
      [
        "ensure_session_key(secret_store, account_id).await?",
        "encrypt_saved_session(account_id, &key, saved)?",
        "serde_json::to_string(&envelope)",
        "write_atomic(path, &json)",
      ],
      "session key-encrypt-write order",
    );
    expect(rustFunctionBody(session, "encrypt_saved_session")).toContain(
      "map_err(|_| AppError::internal(",
    );
    const encodeErrors = normalizedActiveCallArguments(
      rustFunctionOriginalBody(session, "decode_base64"),
      "AppError::internal",
    );
    expect(encodeErrors).toEqual([
      'format!( "Invalid encrypted Telegram session encoding: {error}" )',
    ]);
    const encryptErrors = normalizedActiveCallArguments(
      rustFunctionOriginalBody(session, "encrypt_saved_session"),
      "AppError::internal",
    );
    expect(encryptErrors).toContain('"Invalid Telegram session key length"');
    expect(encryptErrors).toContain('"Failed to encrypt Telegram session"');
    const decryptErrors = normalizedActiveCallArguments(
      rustFunctionOriginalBody(session, "decrypt_saved_session"),
      "AppError::internal",
    );
    for (const exactArgument of [
      '"Unsupported encrypted Telegram session format",',
      '"Invalid Telegram session key length"',
      '"Invalid encrypted Telegram session nonce length",',
      '"Failed to decrypt Telegram session"',
    ]) {
      expect(decryptErrors).toContain(exactArgument);
    }
    expectOrdered(
      rustFunctionBody(session, "delete_session_from_path"),
      [
        "fs::remove_file(path)",
        "std::io::ErrorKind::NotFound",
        "delete_secret(telegram_account_session_key_secret(account_id))",
      ],
      "session file-before-key deletion order",
    );
    const loadSessionBody = rustFunctionBody(
      session,
      "load_session_from_path",
    );
    const loadSessionOriginal = rustFunctionOriginalBody(
      session,
      "load_session_from_path",
    );
    expectOrdered(
      loadSessionBody,
      [
        "if !path.exists()",
        "fs::read_to_string(path)",
        "serde_json::from_str::<EncryptedSessionEnvelope>(&json)",
        "read_session_key(secret_store, account_id)",
        "AppError::auth(format!(",
        "decrypt_saved_session(account_id, &key, &envelope)?",
        "serde_json::from_str::<SavedSession>(&json)",
        "write_encrypted_session_file(path, secret_store, account_id, &saved).await?",
        "AppError::internal(",
      ],
      "encrypted-then-legacy session compatibility order",
    );
    expect(
      normalizedActiveCallArguments(loadSessionOriginal, "AppError::auth"),
    ).toEqual([
      'format!( "Telegram session key for account {account_id} is missing from secure storage. Sign in again." )',
    ]);
    expect(
      normalizedActiveCallArguments(
        loadSessionOriginal,
        "AppError::internal",
      ),
    ).toContain('"Telegram session file is not a supported format",');
  });

  it("pins Takeout mutation-before-event and terminal cancellation selection", () => {
    const state =
      telegramContractPaths.readTelegramContractFile(
        "src-tauri/src/takeout_import/state.rs",
      );
    const takeout =
      telegramContractPaths.readTelegramContractFile(
        "src-tauri/src/takeout_import/mod.rs",
      );

    const requestCancelBody = rustFunctionBody(state, "request_cancel");
    expectOrdered(
      requestCancelBody,
      [
        "is_terminal_status(&inner.jobs.get(job_id)?.status)",
        "inner.cancel_requested.request(job_id)",
        "job.status = STATUS_CANCEL_REQUESTED.to_string()",
        "job.message = Some(",
        "Some(job.clone())",
      ],
      "Takeout cancellation state order",
    );
    expect(
      normalizedActiveCallArguments(
        rustFunctionOriginalBody(state, "request_cancel"),
        "Some",
      ),
    ).toContain('"Cancel requested.".to_string()');
    expectOrdered(
      rustFunctionBody(state, "finish_job"),
      [
        "update(job)",
        "job.finished_at = Some(now_secs())",
        "inner.active_jobs.release_by_job_id(job_id)",
        "inner.cancel_requested.clear(job_id)",
        "inner.jobs.get(job_id).cloned()",
      ],
      "Takeout terminal cleanup order",
    );
    expectOrdered(
      rustFunctionBody(state, "update_and_emit"),
      [
        "state.update_job(job_id, update).await",
        "emit_takeout_import_event(handle, &record)",
      ],
      "Takeout update-before-event order",
    );
    expect(rustFunctionBody(state, "emit_takeout_import_event").trim()).toBe(
      "let _ = handle.emit(TAKEOUT_IMPORT_EVENT, record);",
    );
    expect(rustFunctionBody(state, "is_terminal_status")).toContain(
      "matches!(status, STATUS_FAILED | STATUS_CANCELLED | STATUS_COMPLETED)",
    );

    const jobBody = rustFunctionBody(takeout, "run_takeout_import_job");
    expectOrdered(
      jobBody,
      [
        "job.status = STATUS_RUNNING.to_string()",
        "emit_takeout_import_event(&handle, &running_record)",
        "if takeout_state.is_cancel_requested(&job_id).await",
        "TerminalBatchStatus::Cancelled",
        "job.status = STATUS_CANCELLED.to_string()",
        "emit_takeout_import_event(&handle, &record)",
        "match run_takeout_source_import(&handle, &job_id, batch_id).await",
      ],
      "Takeout initial cancellation and remote-start order",
    );
    expectOrdered(
      jobBody.slice(jobBody.indexOf("match run_takeout_source_import")),
      [
        "Ok(outcome)",
        "finish_job(&job_id",
        "job.status = STATUS_COMPLETED.to_string()",
        "emit_takeout_import_event(&handle, &record)",
        "Err(error)",
        "if takeout_state.is_cancel_requested(&job_id).await",
        "TerminalBatchStatus::Cancelled",
        "job.status = STATUS_CANCELLED.to_string()",
        "TerminalBatchStatus::Failed",
        "job.status = STATUS_FAILED.to_string()",
      ],
      "Takeout terminal status selection order",
    );
  });

  it("freezes live message selection, fallback identity, and persist-finalize boundaries", () => {
    const sync =
      telegramContractPaths.readTelegramContractFile(
        "src-tauri/src/sources/sync.rs",
      );
    const policy = rustFunctionBody(sync, "determine_sync_policy");
    expect(policy).toContain(
      "Some(now_secs() - settings.initial_sync_value * SECONDS_PER_DAY)",
    );
    expect(policy).toContain("InitialSyncMode::RecentMessages => None");

    const persist = rustFunctionBody(sync, "persist_items");
    assertLiveSyncIteratorSelection(persist);
    expectOrdered(
      persist,
      [
        ".iter_messages(peer)",
        ".limit(settings.initial_sync_value as usize)",
        "while let Some(message) = messages",
        "if sync_policy.previous_last_sync > 0 && message_id <= sync_policy.previous_last_sync",
        "if let Some(cutoff) = sync_policy.initial_sync_cutoff",
        "if published_at < cutoff",
        "extract_item_payload(&message)",
        "fallback_message_identity(peer, message_id)?",
        "insert_telegram_source_item(",
        ".await?",
        "Ok(IngestOutcome",
      ],
      "live fetch-filter-incremental-persist order",
    );
    expect(persist).not.toMatch(/\.(?:reverse|sort|sort_by|sort_by_key)\s*\(/);
    const fallback = rustFunctionBody(sync, "fallback_message_identity");
    expectOrdered(
      fallback,
      [
        "PeerKind::User => TELEGRAM_PEER_KIND_USER",
        "PeerKind::Chat => TELEGRAM_PEER_KIND_CHAT",
        "PeerKind::Channel => TELEGRAM_PEER_KIND_CHANNEL",
        "fallback_peer.id.bare_id()",
        "history_peer_kind",
        "history_peer_id",
        "telegram_message_id",
        "migration_domain: None",
        "is_migrated_history: false",
      ],
      "fallback identity vocabulary and shape",
    );
    expectOrdered(
      rustFunctionBody(sync, "sync_telegram_source"),
      [
        "get_authorized_runtime(&state, account_id).await?",
        "resolve_and_refresh_peer(",
        "refresh_forum_topics(",
        "determine_sync_policy(&pool, &source).await?",
        "persist_items(&pool, &client, resolved_peer.peer, &source, &sync_policy).await?",
        "finalize_sync(",
        "Ok(SyncResult",
      ],
      "live resolve-fetch-persist-finalize order",
    );
  });

  it("freezes Takeout range/page/fallback, incremental persistence, warnings, and finalization", () => {
    const takeout =
      telegramContractPaths.readTelegramContractFile(
        "src-tauri/src/takeout_import/mod.rs",
      );
    const pagination =
      telegramContractPaths.readTelegramContractFile(
        "src-tauri/src/takeout_import/pagination.rs",
      );
    const rawParse =
      telegramContractPaths.readTelegramContractFile(
        "src-tauri/src/takeout_import/raw_parse.rs",
      );
    const paginationStructure = maskRustLexicalNonCode(pagination);
    const rawParseStructure = maskRustLexicalNonCode(rawParse);
    const takeoutStructure = maskRustLexicalNonCode(takeout);
    assertTakeoutObservableBoundaries(takeout);

    expect(paginationStructure).toContain(
      "const TAKEOUT_HISTORY_PAGE_LIMIT: i32 = 100",
    );
    expectOrdered(
      rustFunctionBody(pagination, "takeout_page_request"),
      [
        "TakeoutPaginationCursor::TDesktop",
        "offset_id: largest_id_plus_one",
        "add_offset: -TAKEOUT_HISTORY_PAGE_LIMIT",
        "limit: TAKEOUT_HISTORY_PAGE_LIMIT",
        "TakeoutPaginationCursor::DescendingFallback",
        "add_offset: 0",
      ],
      "Takeout page request cursor order",
    );
    expect(paginationStructure).toContain(
      "TakeoutPaginationProfile::TDesktop",
    );
    expect(paginationStructure).toContain(
      "TakeoutPaginationProfile::DescendingFallback",
    );

    const started = rustFunctionBody(
      takeout,
      "run_started_takeout_source_import_inner",
    );
    expectOrdered(
      started,
      [
        "PHASE_VALIDATING_PEER",
        "validate_takeout_peer(",
        "detect_supergroup_migration(",
        "PHASE_LOADING_SPLITS",
        "GetSplitRanges",
        "select_history_splits(",
        "PHASE_COUNTING",
        "for range in selected_ranges",
        "takeout_history_count_probe(",
        "update_takeout_split_metadata(",
        "PHASE_IMPORTING_HISTORY",
        "import_takeout_history_ranges(",
        "if takeout_state.is_cancel_requested(job_id).await",
        "PHASE_FINISHING_TAKEOUT",
        "record_export_dc_attempt_if_needed(",
        "finish_takeout_session(",
        "record_export_dc_fallback_if_needed(",
        "refresh_forum_topics_after_completed_takeout(",
        "finalize_sync(",
        "finalize_ingest_batch(",
        "Ok(TakeoutImportOutcome",
      ],
      "Takeout operation-warning-finalize order",
    );

    const pages = rustFunctionBody(takeout, "import_takeout_history_pages");
    expectOrdered(
      pages,
      [
        "TakeoutPaginationProfile::TDesktop",
        "if takeout_state.is_cancel_requested(job_id).await",
        "let request = takeout_page_request(cursor)",
        "takeout_history_page_response(",
        "let page = parse_takeout_page(response, profile)?",
        "let advance = next_takeout_cursor(cursor, &page, &range)",
        "should_restart_with_descending_fallback(",
        "takeout_pagination_fallback_warning(reason, &range)",
        "update_and_emit(",
        "TakeoutPaginationProfile::DescendingFallback",
        "for message in page.messages",
        "update_takeout_max_message_id(",
        "raw_parse::parse_raw_message(",
        "insert_telegram_source_item_with_observation",
        "update_and_emit(",
        "if takeout_state.is_cancel_requested(job_id).await",
        "page.is_terminal_response",
        "cursor = advance.cursor",
      ],
      "Takeout page fetch-persist-event-cancel-cursor order",
    );
    const countProbe = rustFunctionBody(takeout, "takeout_history_count_probe");
    expectOrdered(
      countProbe,
      [
        "takeout_get_history(",
        "supports_only_my_messages_fallback(",
        "is_channel_private_error(&error)",
        "record_only_my_messages_fallback_if_needed(",
        "takeout_search_my_messages(",
        "only_my_messages: true",
      ],
      "Takeout count channel-private fallback order",
    );
    const pageResponse = rustFunctionBody(
      takeout,
      "takeout_history_page_response",
    );
    expectOrdered(
      pageResponse,
      [
        "if *only_my_messages",
        "takeout_search_my_messages(",
        "match takeout_get_history(",
        "is_channel_private_error(&error)",
        "*only_my_messages = true",
        "record_only_my_messages_fallback_if_needed(",
        "takeout_search_my_messages(",
      ],
      "Takeout page channel-private fallback order",
    );
    expectOrdered(
      rustFunctionBody(takeout, "record_only_my_messages_fallback_if_needed"),
      [
        "push_warning_once(",
        "mark_takeout_only_my_messages_fallback(",
        "*only_my_messages_recorded = true",
      ],
      "Takeout warning-before-provenance order",
    );
    expectOrdered(
      rustFunctionBody(takeout, "export_dc_invoke_with_provenance"),
      [
        "record_export_dc_attempt_if_needed(",
        "export_dc_invoke(",
        "record_export_dc_fallback_if_needed(",
      ],
      "Takeout export attempt-before-remote-before-fallback order",
    );
    expect(rawParseStructure).toContain(
      "let telegram_identity = raw_message_identity(&message);",
    );
    expect(rawParseStructure).toContain(
      "tl::enums::Peer::User(peer) => (      , peer.user_id)",
    );
    expect(rawParseStructure).toContain(
      "tl::enums::Peer::Chat(peer) => (      , peer.chat_id)",
    );
    expect(rawParseStructure).toContain(
      "tl::enums::Peer::Channel(peer) => (         , peer.channel_id)",
    );
    expect(rawParseStructure).toContain(
      "telegram_message_id: i64::from(message.id)",
    );
    expect(takeoutStructure).not.toMatch(
      /\btrait\s+\w*(?:Provider|Remote)\b/,
    );
  });

  it("uses one production session temp-path helper with the frozen extension", () => {
    const source =
      telegramContractPaths.readTelegramContractFile(
        "src-tauri/src/telegram_session_store.rs",
      );
    const helperBody = rustFunctionBody(source, "session_temp_path");
    const helperOriginal = rustFunctionOriginalBody(
      source,
      "session_temp_path",
    );
    expect(helperBody.trim()).toMatch(/^path\.with_extension\(\s+\)$/);
    expect(helperOriginal.trim()).toBe(
      'path.with_extension("session.json.tmp")',
    );
    expect(
      countMatches(
        maskRustLexicalNonCode(source),
        /\bsession_temp_path\s*\(/g,
      ),
    ).toBe(3);
    expect(rustFunctionBody(source, "write_atomic")).toContain(
      "let tmp_path = session_temp_path(path);",
    );
    expect(rustFunctionBody(source, "write_atomic")).not.toContain(
      "path.with_extension(",
    );
  });
});
