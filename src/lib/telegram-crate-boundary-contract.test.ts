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

const checkpointThreeMediaOwnerPath =
  "src-tauri/src/telegram/media.rs";

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
const designPhase8StatusMatches = [
  ...design.matchAll(/^\*\*Status:\*\* (.+)$/gm),
];
if (designPhase8StatusMatches.length !== 1) {
  throw new Error(
    `Expected exactly one Phase 8 design status, found ${designPhase8StatusMatches.length}`,
  );
}
const designPhase8Status = designPhase8StatusMatches[0][1];
const statusLifecycle =
  telegramContractPaths.telegramLifecycleFromStatus(phase8Status);
const checkpoint3LeavesExist = [
  "src-tauri/src/telegram/dto.rs",
  "src-tauri/src/telegram/media.rs",
].every((relativePath) => existsSync(path.join(repoRoot, relativePath)));
const checkpointThreeLifecycle =
  statusLifecycle === "8a-checkpoint-2" && checkpoint3LeavesExist
    ? "8a-checkpoint-3"
    : statusLifecycle;
const checkpoint4LeafExists = existsSync(
  path.join(repoRoot, "src-tauri/src/telegram/session.rs"),
);
const lifecycle =
  checkpointThreeLifecycle === "8a-checkpoint-3" && checkpoint4LeafExists
    ? "8a-checkpoint-4"
    : checkpointThreeLifecycle;
const retainedPreparationStates = [
  {
    roadmapStatus: "8A preparation Checkpoint 5 retained",
    designStatus: "Approved; 8A preparation Checkpoint 5 retained",
    lifecycle: "8a-checkpoint-5",
  },
  {
    roadmapStatus: "8A preparation retained",
    designStatus: "Approved; 8A preparation retained; 8B not started",
    lifecycle: "8a-retained",
  },
] as const;

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
    /grammers_(?:client|session|mtsender|tl_types)|\b(?:get_client|get_authorized_client|get_authorized_runtime|TelegramApiHash|TelegramClientHandle|TelegramRuntime|AuthorizedTelegramRuntime|AccountClient|raw_client|raw_session|MemorySession|LoginToken)\b|\.accounts\s*\.lock\s*\(\s*\)\s*\.await/;
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
    expectedApp = [
      ...expectedApp,
      "src-tauri/src/telegram/runtime.rs",
    ];
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

function checkpointThreeAppRustSources(
  sourceOverrides: ReadonlyMap<string, string> = new Map(),
): ReadonlyMap<string, string> {
  return new Map(
    rustPathsUnder("src-tauri/src").map((relativePath) => [
      relativePath,
      sourceOverrides.get(relativePath)
      ?? telegramContractPaths.readTelegramContractFile(relativePath),
    ]),
  );
}

function productionRustSource(source: string): string {
  const searchable = maskRustLexicalNonCode(source);
  const testModule =
    /#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]\s*mod\s+tests\b/.exec(
      searchable,
    );
  return testModule?.index === undefined
    ? source
    : source.slice(0, testModule.index);
}

type RustItemBoundary = Readonly<
  { kind: "body"; index: number } | { kind: "semicolon"; index: number }
>;

function rustItemBoundary(
  source: string,
  searchable: string,
  start: number,
  label: string,
): RustItemBoundary {
  let parenDepth = 0;
  let squareDepth = 0;
  for (let index = start; index < searchable.length; index += 1) {
    const character = searchable[index];
    if (character === "(") parenDepth += 1;
    if (character === ")") parenDepth -= 1;
    if (character === "[") squareDepth += 1;
    if (character === "]") squareDepth -= 1;
    if (parenDepth < 0 || squareDepth < 0) {
      throw new Error(`${label} found an unmatched item delimiter`);
    }
    if (character === "{") {
      const close = rustClosingBraceIndex(source, index, label);
      if (parenDepth > 0 || squareDepth > 0) {
        index = close;
        continue;
      }
      let next = close + 1;
      while (/\s/.test(searchable[next] ?? "")) next += 1;
      if (searchable[next] === ">" || searchable[next] === ",") {
        index = close;
        continue;
      }
      return { kind: "body", index };
    }
    if (
      character === ";"
      && parenDepth === 0
      && squareDepth === 0
    ) {
      return { kind: "semicolon", index };
    }
  }
  throw new Error(`${label} found an item without a terminator`);
}

function maskExactCfgTestItemSpans(source: string): string {
  const searchable = maskRustLexicalNonCode(source);
  const ranges: Array<{ start: number; end: number }> = [];
  for (
    const match of searchable.matchAll(
      /#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]/g,
    )
  ) {
    if (match.index === undefined) continue;
    const previous = ranges.at(-1);
    if (previous && match.index < previous.end) continue;
    if (rustBraceDepthBefore(searchable, match.index) !== 0) {
      continue;
    }

    let cursor = match.index + match[0].length;
    while (cursor < searchable.length) {
      while (/\s/.test(searchable[cursor] ?? "")) cursor += 1;
      if (searchable[cursor] !== "#") break;
      let bracket = cursor + 1;
      while (/\s/.test(searchable[bracket] ?? "")) bracket += 1;
      if (searchable[bracket] !== "[") break;
      let depth = 1;
      bracket += 1;
      while (bracket < searchable.length && depth > 0) {
        if (searchable[bracket] === "[") depth += 1;
        if (searchable[bracket] === "]") depth -= 1;
        bracket += 1;
      }
      if (depth !== 0) {
        throw new Error(
          "Checkpoint 4 cfg(test) inventory found an unclosed item attribute",
        );
      }
      cursor = bracket;
    }

    const boundary = rustItemBoundary(
      source,
      searchable,
      cursor,
      "Checkpoint 4 cfg(test) inventory",
    );
    let end: number;
    if (boundary.kind === "semicolon") {
      end = boundary.index + 1;
    } else {
      end = rustClosingBraceIndex(
        source,
        boundary.index,
        "Checkpoint 4 cfg(test) item",
      ) + 1;
    }
    ranges.push({ start: match.index, end });
  }

  let result = "";
  let cursor = 0;
  for (const { start, end } of ranges) {
    result += source.slice(cursor, start);
    result += source.slice(start, end).replace(/[^\r\n]/g, " ");
    cursor = end;
  }
  return result + source.slice(cursor);
}

function assertCheckpointThreeOwnershipContract(
  sourceOverrides: ReadonlyMap<string, string> = new Map(),
): void {
  const sources = checkpointThreeAppRustSources(sourceOverrides);
  const searchableSources = [...sources.entries()].map(
    ([relativePath, source]) => [
      relativePath,
      maskRustLexicalNonCode(source),
    ] as const,
  );
  const forbidden = [
    /\bstruct\s+SourceItemInsert\b/g,
    /\bstruct\s+ExtractedItemPayload\b/g,
    /\bstruct\s+ExtractedMediaPayload\b/g,
    /\bfn\s+insert_source_item\s*\(/g,
    /#\s*\[\s*allow\s*\(\s*dead_code\s*\)\s*\]\s*(?:pub(?:\s*\([^)]*\))?\s+)?(?:fn\s+insert_source_item\b|external_id\s*:)/g,
  ];
  const forbiddenSites = searchableSources.flatMap(
    ([relativePath, source]) =>
      forbidden.flatMap((expression) =>
        [...source.matchAll(expression)].map(
          (match) => `${relativePath}:${normalize(match[0]).trim()}`,
        )
      ),
  );
  expect(
    forbiddenSites,
    "Checkpoint 3 ownership contract: forbidden legacy declarations",
  ).toEqual([]);

  const owners = [
    {
      label: "TelegramMessageIdentity",
      expression: /\bpub\s+struct\s+TelegramMessageIdentity\b/g,
      path: "src-tauri/src/telegram/dto.rs",
    },
    {
      label: "TelegramItemContext",
      expression: /\bpub\s+struct\s+TelegramItemContext\b/g,
      path: "src-tauri/src/telegram/dto.rs",
    },
    {
      label: "TelegramMessageDraft",
      expression: /\bpub\s+struct\s+TelegramMessageDraft\b/g,
      path: "src-tauri/src/telegram/dto.rs",
    },
    {
      label: "TelegramMediaPayload",
      expression: /\bpub\s+struct\s+TelegramMediaPayload\b/g,
      path: checkpointThreeMediaOwnerPath,
    },
    {
      label: "DocumentSignals",
      expression: /\bpub\s*\(\s*crate\s*\)\s+struct\s+DocumentSignals\b/g,
      path: checkpointThreeMediaOwnerPath,
    },
    {
      label: "ITEM_KIND_TELEGRAM_MESSAGE",
      expression: /\bpub\s+const\s+ITEM_KIND_TELEGRAM_MESSAGE\b/g,
      path: "src-tauri/src/telegram/dto.rs",
    },
    ...[
      "TELEGRAM_PEER_KIND_CHANNEL",
      "TELEGRAM_PEER_KIND_CHAT",
      "TELEGRAM_PEER_KIND_USER",
    ].map((label) => ({
      label,
      expression: new RegExp(
        `\\bpub\\s*\\(\\s*crate\\s*\\)\\s+const\\s+${label}\\b`,
        "g",
      ),
      path: "src-tauri/src/telegram/dto.rs",
    })),
    ...[
      "CONTENT_KIND_TEXT_ONLY",
      "CONTENT_KIND_TEXT_WITH_MEDIA",
    ].map((label) => ({
      label,
      expression: new RegExp(
        `\\bpub\\s*\\(\\s*crate\\s*\\)\\s+const\\s+${label}\\b`,
        "g",
      ),
      path: checkpointThreeMediaOwnerPath,
    })),
    {
      label: "CONTENT_KIND_MEDIA_ONLY",
      expression: /\bconst\s+CONTENT_KIND_MEDIA_ONLY\b/g,
      path: checkpointThreeMediaOwnerPath,
    },
  ];
  for (const owner of owners) {
    const sites = searchableSources.flatMap(([relativePath, source]) =>
      [...source.matchAll(owner.expression)].map(() => relativePath)
    );
    expect(
      sites,
      `Checkpoint 3 ownership contract: sole ${owner.label} owner`,
    ).toEqual([owner.path]);
  }

  const dto =
    sourceOverrides.get("src-tauri/src/telegram/dto.rs")
    ?? telegramContractPaths.readTelegramContractFile(
      "src-tauri/src/telegram/dto.rs",
    );
  expect(
    rustStructBody(dto, "TelegramMessageDraft"),
    "Checkpoint 3 ownership contract: draft external_id removal",
  ).not.toMatch(/\bpub\s+external_id\s*:/);
}

function assertCheckpointThreeApiContract(
  sourceOverrides: ReadonlyMap<string, string> = new Map(),
): void {
  const read = (relativePath: string): string =>
    sourceOverrides.get(relativePath)
    ?? telegramContractPaths.readTelegramContractFile(relativePath);
  const itemsPath = "src-tauri/src/sources/items.rs";
  const dtoPath = "src-tauri/src/telegram/dto.rs";
  const telegramPath = "src-tauri/src/telegram.rs";
  const appMediaPath = "src-tauri/src/media.rs";
  const items = read(itemsPath);
  const media = read(checkpointThreeMediaOwnerPath);
  const dto = read(dtoPath);
  const telegram = read(telegramPath);
  const appMedia = read(appMediaPath);

  const expectedItemDeclarations = new Map([
    [
      "prepare_source_item",
      "fn prepare_source_item(draft: &crate::telegram::TelegramMessageDraft,) -> extractum_core::error::AppResult<Option<PreparedSourceItem>>",
    ],
    [
      "insert_telegram_source_item",
      "pub(crate) async fn insert_telegram_source_item(pool: &sqlx::Pool<sqlx::Sqlite>, source_id: i64, draft: crate::telegram::TelegramMessageDraft,) -> extractum_core::error::AppResult<bool>",
    ],
    [
      "insert_telegram_source_item_outcome",
      "pub(crate) async fn insert_telegram_source_item_outcome(pool: &sqlx::Pool<sqlx::Sqlite>, source_id: i64, draft: crate::telegram::TelegramMessageDraft,) -> extractum_core::error::AppResult<TelegramItemInsertOutcome>",
    ],
    [
      "insert_telegram_source_item_with_observation",
      "pub(crate) async fn insert_telegram_source_item_with_observation(pool: &sqlx::Pool<sqlx::Sqlite>, batch_id: i64, source_id: i64, draft: crate::telegram::TelegramMessageDraft,) -> extractum_core::error::AppResult<TelegramItemInsertOutcome>",
    ],
    [
      "insert_telegram_source_item_with_observation_in_context",
      "pub(crate) async fn insert_telegram_source_item_with_observation_in_context(pool: &sqlx::Pool<sqlx::Sqlite>, batch_id: i64, source_id: i64, draft: TelegramMessageDraft, insert_context: TelegramInsertContext,) -> AppResult<TelegramItemInsertOutcome>",
    ],
    [
      "insert_telegram_source_item_on_connection",
      "async fn insert_telegram_source_item_on_connection(conn: &mut sqlx::SqliteConnection, source_id: i64, draft: TelegramMessageDraft, insert_context: TelegramInsertContext, archive_maintenance: ArchiveReadMaintenanceMode,) -> AppResult<TelegramItemInsertOutcome>",
    ],
  ]);
  for (const [name, declaration] of expectedItemDeclarations) {
    expect(
      normalizedRustFunctionDeclaration(items, name),
      `Checkpoint 3 API contract: exact ${name} declaration`,
    ).toBe(declaration);
  }
  expect(
    normalizedRustFunctionDeclaration(media, "extract_item_payload"),
    "Checkpoint 3 API contract: exact unnamed extract_item_payload tuple",
  ).toBe(
    "pub(crate) fn extract_item_payload(message: &grammers_client::message::Message,) -> Option<(Option<String>, &'static str, Option<TelegramMediaPayload>)>",
  );

  expect(
    normalizedRustUseDeclarations(telegram, "dto"),
    "Checkpoint 3 API contract: exact Telegram DTO facade",
  ).toEqual([
    "pub(crate) use dto::{TelegramItemContext, TelegramMessageDraft, TelegramMessageIdentity, ITEM_KIND_TELEGRAM_MESSAGE, TELEGRAM_PEER_KIND_CHANNEL, TELEGRAM_PEER_KIND_CHAT, TELEGRAM_PEER_KIND_USER,};",
  ]);
  expect(
    normalizedRustUseDeclarations(telegram, "media"),
    "Checkpoint 3 API contract: exact Telegram media facade",
  ).toEqual([
    "#[allow(unused_imports)] pub(crate) use media::{derive_content_kind, derive_document_media_kind, extract_item_payload, DocumentSignals, TelegramMediaPayload, CONTENT_KIND_TEXT_ONLY, CONTENT_KIND_TEXT_WITH_MEDIA,};",
  ]);
  expect(
    normalizedRustUseDeclarations(telegram, "session"),
    "Checkpoint 4 API contract: exact Telegram session facade",
  ).toEqual([
    "pub(crate) use session::{decode_session_json, encode_session_json, session_json_requires_existing_key, SessionEncryptionKey, TelegramSession,};",
  ]);
  expect(
    normalizedRustUseDeclarations(appMedia, "crate::telegram"),
    "Checkpoint 3 API contract: exact app media compatibility facade",
  ).toEqual([
    "pub(crate) use crate::telegram::{derive_content_kind, derive_document_media_kind, extract_item_payload, DocumentSignals, TelegramMediaPayload,};",
    "#[cfg(test)] pub(crate) use crate::telegram::{CONTENT_KIND_TEXT_ONLY, CONTENT_KIND_TEXT_WITH_MEDIA};",
  ]);
  expect(
    normalizedRustUseDeclarations(
      appMedia,
      "extractum_core::media_metadata",
    ),
    "Checkpoint 3 API contract: exact provider-neutral media facade",
  ).toEqual([
    "pub(crate) use extractum_core::media_metadata::{decode_media_metadata, encode_media_metadata, media_label, ItemMediaMetadata,};",
  ]);
  expect(
    `${maskRustLexicalNonCode(telegram)}\n${maskRustLexicalNonCode(appMedia)}`,
    "Checkpoint 3 API contract: facades stay curated",
  ).not.toMatch(/\bpub\s*\(\s*crate\s*\)\s+use\s+[^;]*::\s*\*/);

  const telegramStructure = maskRustLexicalNonCode(telegram);
  expect(
    [...telegramStructure.matchAll(
      /\b(?:pub(?:\s*\([^)]*\))?\s+)?mod\s+(dto|media|session)\s*;/g,
    )].map((match) =>
      normalize(match[0]).trim()
    ),
    "Checkpoint 3 API contract: private Telegram leaves",
  ).toEqual(["mod dto;", "mod media;", "mod session;"]);
  expect(
    normalizedRustUseDeclarations(dto, "super::media"),
    "Checkpoint 3 API contract: future-owner relative leaf import",
  ).toEqual(["use super::media::TelegramMediaPayload;"]);
  expect(
    `${maskRustLexicalNonCode(dto)}\n${maskRustLexicalNonCode(media)}\n${
      maskRustLexicalNonCode(read("src-tauri/src/telegram/session.rs"))
    }`,
    "Checkpoint 3/4 API contract: future-owner leaves avoid app facades",
  ).not.toMatch(/\bcrate::(?:telegram|media)\b/);

  const appSources = checkpointThreeAppRustSources(sourceOverrides);
  const directLeafImports = [...appSources.entries()]
    .filter(
      ([relativePath]) =>
        relativePath !== dtoPath
        && relativePath !== checkpointThreeMediaOwnerPath,
    )
    .flatMap(([relativePath, source]) =>
      [...maskRustLexicalNonCode(source).matchAll(
        /\bcrate::telegram::(?:dto|media|session)\b/g,
      )].map((match) => `${relativePath}:${match[0]}`)
    );
  expect(
    directLeafImports,
    "Checkpoint 3 API contract: app consumers use facades",
  ).toEqual([]);

  const expectedAppTelegramImports = new Map<string, string[]>([
    [
      itemsPath,
      [
        "use crate::telegram::{TelegramItemContext, TelegramMediaPayload, TelegramMessageDraft, ITEM_KIND_TELEGRAM_MESSAGE,};",
      ],
    ],
    [
      "src-tauri/src/sources/sync.rs",
      [
        "use crate::telegram::{TelegramMessageDraft, TelegramMessageIdentity, TelegramState, ITEM_KIND_TELEGRAM_MESSAGE, TELEGRAM_PEER_KIND_CHANNEL, TELEGRAM_PEER_KIND_CHAT, TELEGRAM_PEER_KIND_USER,};",
      ],
    ],
    [
      "src-tauri/src/takeout_import/raw_parse.rs",
      [
        "use crate::telegram::{TelegramItemContext, TelegramMessageDraft, TelegramMessageIdentity, ITEM_KIND_TELEGRAM_MESSAGE,};",
      ],
    ],
    [
      "src-tauri/src/sources/mod.rs",
      [
        "#[allow(unused_imports)] pub(crate) use crate::telegram::{TelegramItemContext, TelegramMessageDraft, TelegramMessageIdentity, ITEM_KIND_TELEGRAM_MESSAGE,};",
      ],
    ],
  ]);
  for (const [relativePath, expected] of expectedAppTelegramImports) {
    expect(
      normalizedRustUseDeclarations(
        productionRustSource(read(relativePath)),
        "crate::telegram",
      ),
      `Checkpoint 3 API contract: exact app facade import in ${relativePath}`,
    ).toEqual(expected);
  }

  expect(
    normalize(productionRustSource(read(
      "src-tauri/src/ingest_provenance.rs",
    ))),
    "Checkpoint 3 API contract: ingest provenance facade import",
  ).toContain("use crate::telegram::TelegramMessageIdentity;");
  expect(
    normalize(productionRustSource(read(
      "src-tauri/src/takeout_import/migrated_history.rs",
    ))),
    "Checkpoint 3 API contract: migrated identity facade path",
  ).toContain("crate::telegram::TelegramMessageIdentity");
  expect(
    normalizedRustUseDeclarations(
      productionRustSource(read(
        "src-tauri/src/takeout_import/raw_parse.rs",
      )),
      "crate::media",
    ),
    "Checkpoint 3 API contract: raw parser compatibility facade import",
  ).toEqual([
    "use crate::media::{derive_content_kind, derive_document_media_kind, media_label, DocumentSignals, ItemMediaMetadata, TelegramMediaPayload,};",
  ]);
  expect(
    normalize(productionRustSource(read(
      "src-tauri/src/sources/sync.rs",
    ))),
    "Checkpoint 3 API contract: live media compatibility import",
  ).toContain("use crate::media::extract_item_payload;");
}

function assertCheckpointFourSessionContract(
  sourceOverrides: ReadonlyMap<string, string> = new Map(),
): void {
  const read = (relativePath: string): string =>
    sourceOverrides.get(relativePath)
    ?? telegramContractPaths.readTelegramContractFile(relativePath);
  const sessionPath = "src-tauri/src/telegram/session.rs";
  const adapterPath = "src-tauri/src/telegram_session_store.rs";
  const telegramPath = "src-tauri/src/telegram.rs";
  const session = read(sessionPath);
  const adapter = read(adapterPath);
  const telegram = read(telegramPath);
  const sessionProduction = maskExactCfgTestItemSpans(session);
  const sessionStructure = maskRustLexicalNonCode(session);
  const sessionImplBlocks = rustImplBlocks(sessionProduction);
  const adapterProduction = maskExactCfgTestItemSpans(adapter);
  const adapterStructure = maskRustLexicalNonCode(adapterProduction);
  const appSources = checkpointThreeAppRustSources(sourceOverrides);
  const wrapperNames = [
    "SessionEncryptionKey",
    "TelegramSession",
  ] as const;
  const wrapperToken = new RegExp(
    `(?:^|[^A-Za-z0-9_#])(?:r#)?(?:${wrapperNames.join("|")})\\b`,
  );
  const wrapperSourceInventories = [...appSources.entries()]
    .filter(([, source]) =>
      wrapperToken.test(maskRustLexicalNonCode(source))
    )
    .map(([relativePath, source]) => {
      const production = maskExactCfgTestItemSpans(source);
      return {
        relativePath,
        implBlocks: rustImplBlocks(
          production,
          `Checkpoint 4 impl inventory ${relativePath}`,
        ),
        aliases: rustWrapperAliasNames(production, wrapperNames),
        visibleFreeFunctions: visibleRustFreeFunctions(
          production,
          `Checkpoint 4 free-function inventory ${relativePath}`,
        ),
      };
    });
  exactInventory(
    wrapperSourceInventories.flatMap(({ relativePath, implBlocks }) =>
      implBlocks
        .filter(({ header }) =>
          wrapperNames.some((wrapper) =>
            rustTypeReferenceRegex(wrapper).test(header)
          )
        )
        .map(({ header }) => `${relativePath}|${header}`)
    ),
    [
      `${sessionPath}|SessionEncryptionKey`,
      `${sessionPath}|TelegramSession`,
    ],
    "Checkpoint 4 ownership contract: app-wide exact wrapper impl inventory",
  );
  exactInventory(
    wrapperSourceInventories.flatMap(
      ({ relativePath, aliases }) =>
        aliases.map((alias) => `${relativePath}|${alias}`),
    ),
    [],
    "Checkpoint 4 ownership contract: no wrapper alias declarations",
  );
  const rawTelegramOrSecretSurface =
    /(?:\bgrammers_(?:client|session|mtsender|tl_types)\b|\b(?:AccountClient|AuthorizedTelegramRuntime|MemorySession|RawClient|RawSession|SecretString|SecretVec|ExposeSecret)\b|\bVec\s*<\s*u8\s*>|\[\s*u8\b)/;
  exactInventory(
    wrapperSourceInventories.flatMap(
      ({ relativePath, visibleFreeFunctions }) =>
        visibleFreeFunctions
          .filter(({ signature }) =>
            wrapperNames.some((wrapper) =>
              rustTypeReferenceRegex(wrapper).test(signature)
            )
            && rawTelegramOrSecretSurface.test(signature)
          )
          .map(({ header }) => `${relativePath}|${header}`),
    ),
    [],
    "Checkpoint 4 secrecy contract: no app-wide visible free wrapper-to-raw signatures",
  );

  for (const owner of [
    ["SavedSession", /\bstruct\s+SavedSession\b/g],
    ["EncryptedSessionEnvelope", /\bstruct\s+EncryptedSessionEnvelope\b/g],
  ] as const) {
    const sites = [...appSources.entries()].flatMap(
      ([relativePath, source]) =>
        [...maskRustLexicalNonCode(source).matchAll(owner[1])].map(
          () => relativePath,
        ),
    );
    expect(
      sites,
      `Checkpoint 4 ownership contract: sole ${owner[0]} owner`,
    ).toEqual([sessionPath]);
  }

  expect(
    normalize(sessionStructure),
    "Checkpoint 4 API contract: Arc-backed cloneable session key",
  ).toMatch(
    /#\s*\[\s*derive\s*\(\s*Clone\s*\)\s*\]\s*pub struct SessionEncryptionKey\s*\(\s*Arc\s*<\s*SecretVec\s*<\s*u8\s*>\s*>\s*\)\s*;/,
  );
  expect(
    normalize(sessionStructure),
    "Checkpoint 4 API contract: opaque Telegram session",
  ).toMatch(
    /#\s*\[\s*derive\s*\(\s*Clone\s*\)\s*\]\s*pub struct TelegramSession\s*\{\s*inner\s*:\s*Arc\s*<\s*MemorySession\s*>\s*,?\s*\}/,
  );
  expect(
    rustStructBody(session, "TelegramSession"),
    "Checkpoint 4 API contract: TelegramSession field stays private",
  ).not.toMatch(/\bpub(?:\s*\([^)]*\))?\s+inner\s*:/);

  const exactDeclarations = new Map([
    [
      "try_from_encoded",
      "pub fn try_from_encoded(encoded: SecretString) -> AppResult<Self>",
    ],
    [
      "generate",
      "pub fn generate() -> (Self, SecretString)",
    ],
    [
      "empty",
      "pub fn empty() -> Self",
    ],
    [
      "raw_memory_session",
      "pub(super) fn raw_memory_session(&self) -> &Arc<MemorySession>",
    ],
    [
      "session_json_requires_existing_key",
      "pub fn session_json_requires_existing_key(json: &str) -> AppResult<bool>",
    ],
    [
      "decode_session_json",
      "pub fn decode_session_json(json: &str, account_id: i64, key: Option<&SessionEncryptionKey>,) -> AppResult<TelegramSession>",
    ],
    [
      "encode_session_json",
      "pub async fn encode_session_json(session: &TelegramSession, account_id: i64, key: &SessionEncryptionKey,) -> AppResult<String>",
    ],
  ]);
  for (const [name, declaration] of exactDeclarations) {
    expect(
      normalizedRustFunctionDeclaration(session, name),
      `Checkpoint 4 API contract: exact ${name} declaration`,
    ).toBe(declaration);
  }

  expect(
    visibleInherentAssociatedItemInventory(
      sessionImplBlocks,
      "SessionEncryptionKey",
    ),
    "Checkpoint 4 API contract: complete visible session-key associated-item inventory",
  ).toEqual([
    "pub fn try_from_encoded",
    "pub fn generate",
  ]);
  expect(
    visibleInherentAssociatedItemInventory(
      sessionImplBlocks,
      "TelegramSession",
    ),
    "Checkpoint 4 API contract: complete visible Telegram-session associated-item inventory",
  ).toEqual([
    "pub fn empty",
    "pub(super) fn raw_memory_session",
  ]);
  for (const wrapper of wrapperNames) {
    expect(
      explicitTraitImplInventory(sessionImplBlocks, wrapper),
      `Checkpoint 4 secrecy contract: ${wrapper} has no raw/conversion trait surface`,
    ).toEqual([]);
  }

  expect(
    countMatches(
      sessionStructure,
      /\bpub\s*\(\s*super\s*\)\s+fn\s+raw_memory_session\s*\(/g,
    ),
    "Checkpoint 4 API contract: sole sibling raw session accessor",
  ).toBe(1);
  expect(
    sessionStructure,
    "Checkpoint 4 API contract: no app-visible raw session accessor",
  ).not.toMatch(
    /\bpub(?:\s*\(\s*crate\s*\))?\s+fn\s+raw_memory_session\s*\(/,
  );

  expect(
    sessionStructure,
    "Checkpoint 4 secrecy contract: no secret Debug/public getter/trait exposure",
  ).not.toMatch(
    /(?:derive\s*\([^)]*\bDebug\b[^)]*\)\s*pub struct SessionEncryptionKey|impl\s+(?:std::fmt::)?Debug\s+for\s+SessionEncryptionKey|impl\s+(?:secrecy::)?ExposeSecret\s+for\s+SessionEncryptionKey|\bpub(?:\s*\([^)]*\))?\s+fn\s+(?:as_bytes|expose_secret|key_bytes|raw_key)\s*\()/,
  );
  expect(
    countMatches(
      maskRustLexicalNonCode(sessionProduction),
      /\.expose_secret\s*\(\s*\)/g,
    ),
    "Checkpoint 4 secrecy contract: exact private decode/encrypt/decrypt sinks",
  ).toBe(3);
  expect(
    countMatches(adapterStructure, /\.expose_secret\s*\(\s*\)/g),
    "Checkpoint 4 secrecy contract: sole generated-secret persistence sink",
  ).toBe(1);

  expect(
    sessionStructure,
    "Checkpoint 4 ownership contract: codec owns no app path/fs/keyring/store capability",
  ).not.toMatch(
    /\b(?:SecretStore|SecretStoreState|keyring|AppHandle|tauri|Path|PathBuf)\b|std::(?:fs|path)\b|telegram_account_session_key_secret/,
  );
  expect(
    adapterStructure,
    "Checkpoint 4 ownership contract: adapter owns no raw session or codec implementation",
  ).not.toMatch(
    /\bgrammers_session\b|\bMemorySession\b|\b(?:SavedSession|EncryptedSessionEnvelope)\b|\b(?:XChaCha20Poly1305|XNonce|Payload)\b|\b(?:encode_base64|decode_base64|associated_data|encrypt_saved_session|decrypt_saved_session)\b/,
  );
  expect(
    normalizedRustUseDeclarations(telegram, "session"),
    "Checkpoint 4 API contract: curated session facade",
  ).toEqual([
    "pub(crate) use session::{decode_session_json, encode_session_json, session_json_requires_existing_key, SessionEncryptionKey, TelegramSession,};",
  ]);

  expectOrdered(
    rustFunctionBody(session, "memory_session_to_saved"),
    [
      "session.home_dc_id()",
      "session.updates_state().await",
      "for dc_id in 1..=5i32",
      "session.dc_option(dc_id)",
      "SavedSession",
    ],
    "Checkpoint 4 codec MemorySession snapshot order",
  );
  expect(
    rustFunctionBody(session, "saved_to_telegram_session"),
    "Checkpoint 4 codec deliberately excludes peer cache from the envelope",
  ).toContain("peer_infos: HashMap::new()");
  expectOrdered(
    rustFunctionBody(session, "session_json_requires_existing_key"),
    [
      "serde_json::from_str::<EncryptedSessionEnvelope>(json)",
      "return Ok(true)",
      "serde_json::from_str::<SavedSession>(json)",
      "return Ok(false)",
      "AppError::internal(",
    ],
    "Checkpoint 4 encrypted-before-legacy classifier order",
  );
  expectOrdered(
    rustFunctionBody(session, "decode_session_json"),
    [
      "serde_json::from_str::<EncryptedSessionEnvelope>(json)",
      "AppError::auth(format!(",
      "decrypt_saved_session(account_id, key, &envelope)?",
      "serde_json::from_str::<SavedSession>(json)",
      "saved_to_telegram_session(saved)",
      "AppError::internal(",
    ],
    "Checkpoint 4 encrypted-before-legacy decode order",
  );
  expectOrdered(
    rustFunctionBody(session, "encode_session_json"),
    [
      "memory_session_to_saved(session).await",
      "encrypt_saved_session(account_id, key, &saved)?",
      "serde_json::to_string(&envelope)",
    ],
    "Checkpoint 4 snapshot-encrypt-serialize order",
  );

  expectOrdered(
    rustFunctionBody(adapter, "ensure_session_key"),
    [
      "read_session_key(secret_store, account_id).await?",
      "SessionEncryptionKey::generate()",
      "set_secret(",
      "encoded.expose_secret()",
      "Ok(key)",
    ],
    "Checkpoint 4 key read-generate-store order",
  );
  expectOrdered(
    rustFunctionBody(adapter, "write_encrypted_session_file"),
    [
      "ensure_session_key(secret_store, account_id).await?",
      "encode_session_json(session, account_id, &key).await?",
      "write_atomic(path, &json)",
    ],
    "Checkpoint 4 save key-encode-write order",
  );
  expectOrdered(
    rustFunctionBody(adapter, "load_session_from_path"),
    [
      "if !path.exists()",
      "TelegramSession::empty()",
      "fs::read_to_string(path)",
      "session_json_requires_existing_key(&json)?",
      "if requires_existing_key",
      "read_session_key(secret_store, account_id).await?",
      "return decode_session_json(&json, account_id, key.as_ref())",
      "let session = decode_session_json(&json, account_id, None)?",
      "ensure_session_key(secret_store, account_id).await?",
      "encode_session_json(&session, account_id, &key).await?",
      "write_atomic(path, &encrypted)?",
      "Ok(session)",
    ],
    "Checkpoint 4 classify/key/decode/legacy-migrate order",
  );
  expectOrdered(
    rustFunctionBody(adapter, "delete_session_from_path"),
    [
      "fs::remove_file(path)",
      "std::io::ErrorKind::NotFound",
      "delete_secret(telegram_account_session_key_secret(account_id))",
    ],
    "Checkpoint 4 file-before-key deletion order",
  );
}

function assertAccountCredentialsRowHasNoSecretDebug(
  sourceOverrides: ReadonlyMap<string, string> = new Map(),
  appSources = checkpointThreeAppRustSources(sourceOverrides),
): void {
  const ownerPath = "src-tauri/src/telegram.rs";
  const ownerSource = appSources.get(ownerPath);
  if (ownerSource === undefined) {
    throw new Error(
      "Checkpoint 5 secrecy contract: AccountCredentialsRow owner is missing",
    );
  }
  const ownerProduction = maskExactCfgTestItemSpans(ownerSource);
  const structure = maskRustLexicalNonCode(ownerProduction);
  expect(
    structure,
    "Checkpoint 5 secrecy contract: plaintext AccountCredentialsRow must not derive Debug",
  ).not.toMatch(
    /#\s*\[\s*derive\s*\([^)]*\bDebug\b[^)]*\)\s*\]\s*(?:#\s*\[[^\]]+\]\s*)*struct\s+AccountCredentialsRow\b/,
  );
  expect(
    rustStructOuterAttributeInventory(
      ownerProduction,
      ["AccountCredentialsRow"],
      "Checkpoint 5 AccountCredentialsRow outer-attribute inventory",
    ),
    "Checkpoint 5 secrecy contract: exact AccountCredentialsRow outer attributes",
  ).toEqual([
    "AccountCredentialsRow|#[derive(sqlx::FromRow)]",
  ]);
  const productionSources = [...appSources.entries()].flatMap(
    ([relativePath, source]) =>
      source.includes("AccountCredentialsRow")
        ? [[relativePath, maskExactCfgTestItemSpans(source)] as const]
        : [],
  );
  const explicitTraitImpls = productionSources.flatMap(
    ([relativePath, source]) =>
      explicitTraitImplInventory(
        rustImplBlocks(
          source,
          "Checkpoint 5 AccountCredentialsRow impl inventory",
        ),
        "AccountCredentialsRow",
      ).map((header) => `${relativePath}|${header}`),
  );
  expect(
    explicitTraitImpls,
    "Checkpoint 5 secrecy contract: plaintext AccountCredentialsRow has no explicit trait surface",
  ).toEqual([]);
  const aliases = productionSources.flatMap(([relativePath, source]) =>
    rustWrapperAliasNames(source, ["AccountCredentialsRow"]).map(
      (alias) => `${relativePath}|${alias}`,
    ),
  );
  expect(
    aliases,
    "Checkpoint 5 secrecy contract: plaintext AccountCredentialsRow has no alias escape",
  ).toEqual([]);
}

function assertCheckpointFiveRuntimeContract(
  sourceOverrides: ReadonlyMap<string, string> = new Map(),
): void {
  const read = (relativePath: string): string =>
    sourceOverrides.get(relativePath)
    ?? telegramContractPaths.readTelegramContractFile(relativePath);
  const runtimePath = "src-tauri/src/telegram/runtime.rs";
  const telegramPath = "src-tauri/src/telegram.rs";
  const storePath = "src-tauri/src/sources/store.rs";
  const syncPath = "src-tauri/src/sources/sync.rs";
  const takeoutPath = "src-tauri/src/takeout_import/mod.rs";
  const runtime = read(runtimePath);
  const telegram = read(telegramPath);
  const store = read(storePath);
  const sync = read(syncPath);
  const takeout = read(takeoutPath);
  const runtimeProduction = maskExactCfgTestItemSpans(runtime);
  const runtimeStructure = maskRustLexicalNonCode(runtimeProduction);
  const runtimeUnmaskedStructure = maskRustLexicalNonCode(runtime);
  const telegramProduction = maskExactCfgTestItemSpans(telegram);
  const telegramStructure = maskRustLexicalNonCode(telegramProduction);
  const storeProduction = maskExactCfgTestItemSpans(store);
  const syncProduction = maskExactCfgTestItemSpans(sync);
  const takeoutProduction = maskExactCfgTestItemSpans(takeout);
  const appSources = checkpointThreeAppRustSources(sourceOverrides);
  const runtimeWrapperNames = [
    "TelegramApiHash",
    "TelegramClientHandle",
    "TelegramLoginAttempt",
    "TelegramRuntime",
  ];
  const runtimeWrapperSourceEntries = [...appSources.entries()].filter(
    ([, source]) =>
      runtimeWrapperNames.some((typeName) => source.includes(typeName)),
  );
  const runtimeWrapperAliases = runtimeWrapperSourceEntries.flatMap(
    ([relativePath, source]) =>
      rustWrapperAliasNames(
        maskExactCfgTestItemSpans(source),
        runtimeWrapperNames,
      ).map((alias) => `${relativePath}|${alias}`),
  );
  expect(
    runtimeWrapperAliases,
    "Checkpoint 5 API contract: runtime wrappers have no type or use aliases",
  ).toEqual([]);
  const sensitiveMacroNames = [
    ...runtimeWrapperNames,
    "AccountCredentialsRow",
  ];
  const sensitiveMacroInventory = [...appSources.entries()]
    .filter(([, source]) =>
      sensitiveMacroNames.some((typeName) => source.includes(typeName))
    )
    .flatMap(
      ([relativePath, source]) =>
        rustSensitiveMacroInventory(
          maskExactCfgTestItemSpans(source),
          sensitiveMacroNames,
          `Checkpoint 5 sensitive macro inventory ${relativePath}`,
        ).map((evidence) => `${relativePath}|${evidence}`),
    );
  expect(
    sensitiveMacroInventory,
    "Checkpoint 5 API contract: runtime wrappers and plaintext credential row have no macro escape",
  ).toEqual([]);
  const runtimeWrapperImpls = runtimeWrapperSourceEntries.flatMap(
    ([relativePath, source]) => {
      const production = maskExactCfgTestItemSpans(source);
      const aliases = rustWrapperAliasNames(production, runtimeWrapperNames);
      const targetNames = [...runtimeWrapperNames, ...aliases];
      return rustImplBlocks(
        production,
        `Checkpoint 5 runtime wrapper impl inventory ${relativePath}`,
      )
        .filter(({ header }) =>
          targetNames.some((typeName) =>
            rustTypeReferenceRegex(typeName).test(header),
          )
        )
        .map(({ header }) => `${relativePath}|${header}`);
    },
  );
  expect(
    runtimeWrapperImpls,
    "Checkpoint 5 API contract: exact runtime wrapper impl sites and headers",
  ).toEqual([
    `${runtimePath}|TelegramApiHash`,
    `${runtimePath}|TelegramClientHandle`,
    `${runtimePath}|TelegramRuntime`,
  ]);
  const appProductionStructure = [...appSources.entries()]
    .map(([relativePath, source]) =>
      `${relativePath}\n${
        maskRustLexicalNonCode(maskExactCfgTestItemSpans(source))
      }`
    )
    .join("\n");
  const runtimeImpls = rustImplBlocks(
    runtimeProduction,
    "Checkpoint 5 runtime impl inventory",
  );
  expect(
    rustStructOuterAttributeInventory(
      runtimeProduction,
      runtimeWrapperNames,
      "Checkpoint 5 runtime wrapper outer-attribute inventory",
    ),
    "Checkpoint 5 API contract: exact runtime wrapper outer attributes",
  ).toEqual([
    "TelegramApiHash|#[derive(Clone)]",
    "TelegramClientHandle|#[derive(Clone)]",
    "TelegramLoginAttempt|",
    "TelegramRuntime|",
  ]);
  const inherentBody = (typeName: string, marker: string): string => {
    const matches = runtimeImpls.filter(
      ({ header, body }) =>
        header === typeName && body.includes(marker),
    );
    expect(
      matches,
      `Checkpoint 5 API contract: sole ${typeName} impl with ${marker}`,
    ).toHaveLength(1);
    return matches[0].body;
  };
  assertAccountCredentialsRowHasNoSecretDebug(sourceOverrides, appSources);

  expect(
    visibleRustTopLevelItemInventory(
      runtimeProduction,
      "Checkpoint 5 runtime visible top-level inventory",
    ),
    "Checkpoint 5 API contract: exact visible runtime top-level items",
  ).toEqual([
    "root|pub struct TelegramApiHash",
    "root|pub enum TelegramRuntimeStatus",
    "root|pub struct TelegramClientHandle",
    "root|pub struct TelegramLoginAttempt",
    "root|pub struct TelegramRuntime",
  ]);
  expect(
    normalize(runtimeStructure),
    "Checkpoint 5 API contract: opaque API hash",
  ).toMatch(
    /#\s*\[\s*derive\s*\(\s*Clone\s*\)\s*\]\s*pub struct TelegramApiHash\s*\(\s*SecretString\s*\)\s*;/,
  );
  expect(
    normalize(runtimeStructure),
    "Checkpoint 5 API contract: closed status vocabulary",
  ).toMatch(
    /#\s*\[\s*derive\s*\(\s*Clone\s*,\s*Copy\s*,\s*Debug\s*,\s*Eq\s*,\s*PartialEq\s*\)\s*\]\s*pub enum TelegramRuntimeStatus\s*\{\s*Ready\s*,\s*ReauthRequired\s*,?\s*\}/,
  );
  for (const typeName of [
    "TelegramClientHandle",
    "TelegramLoginAttempt",
    "TelegramRuntime",
  ]) {
    expect(
      rustStructBody(runtimeProduction, typeName),
      `Checkpoint 5 API contract: ${typeName} fields stay private`,
    ).not.toMatch(/\bpub(?:\s*\([^)]*\))?\s+[A-Za-z_][A-Za-z0-9_]*\s*:/);
  }

  const apiHashImpl = inherentBody("TelegramApiHash", "pub fn new");
  expect(
    normalizedRustFunctionDeclaration(apiHashImpl, "new"),
    "Checkpoint 5 API contract: exact API-hash constructor",
  ).toBe("pub fn new(value: SecretString) -> Self");
  expect(
    visibleInherentAssociatedItemInventory(runtimeImpls, "TelegramApiHash"),
    "Checkpoint 5 API contract: complete API-hash surface",
  ).toEqual(["pub fn new"]);
  expect(
    visibleInherentAssociatedItemInventory(
      runtimeImpls,
      "TelegramClientHandle",
    ),
    "Checkpoint 5 API contract: exact temporary raw adapters",
  ).toEqual([
    "pub(crate) fn raw_client",
    "pub(crate) fn raw_session",
  ]);
  expect(
    normalizedRustFunctionDeclaration(
      inherentBody("TelegramClientHandle", "raw_client"),
      "raw_client",
    ),
    "Checkpoint 5 API contract: exact raw-client adapter",
  ).toBe(
    "pub(crate) fn raw_client(&self) -> &grammers_client::Client",
  );
  expect(
    normalizedRustFunctionDeclaration(
      inherentBody("TelegramClientHandle", "raw_session"),
      "raw_session",
    ),
    "Checkpoint 5 API contract: exact raw-session adapter",
  ).toBe(
    "pub(crate) fn raw_session(&self) -> &std::sync::Arc<grammers_session::storages::MemorySession>",
  );

  const runtimeImpl = inherentBody("TelegramRuntime", "initialize_account");
  const exactRuntimeDeclarations = new Map([
    ["new", "pub fn new() -> Self"],
    [
      "initialize_account",
      "pub async fn initialize_account(&self, account_id: i64, api_id: i32, api_hash: TelegramApiHash, session: TelegramSession,) -> extractum_core::error::AppResult<TelegramRuntimeStatus>",
    ],
    [
      "is_authenticated",
      "pub async fn is_authenticated(&self, account_id: i64,) -> extractum_core::error::AppResult<bool>",
    ],
    [
      "request_login_code",
      "pub async fn request_login_code(&self, account_id: i64, phone: String,) -> extractum_core::error::AppResult<()>",
    ],
    [
      "sign_in",
      "pub async fn sign_in(&self, account_id: i64, code: String,) -> extractum_core::error::AppResult<TelegramSession>",
    ],
    [
      "authorized_client",
      "pub async fn authorized_client(&self, account_id: i64,) -> extractum_core::error::AppResult<TelegramClientHandle>",
    ],
    [
      "initialized_client",
      "pub(super) async fn initialized_client(&self, account_id: i64,) -> extractum_core::error::AppResult<TelegramClientHandle>",
    ],
    [
      "clear_account",
      "pub async fn clear_account(&self, account_id: i64, sign_out: bool)",
    ],
  ]);
  for (const [name, declaration] of exactRuntimeDeclarations) {
    expect(
      normalizedRustFunctionDeclaration(runtimeImpl, name),
      `Checkpoint 5 API contract: exact ${name} declaration`,
    ).toBe(declaration);
  }
  expect(
    visibleInherentAssociatedItemInventory(runtimeImpls, "TelegramRuntime"),
    "Checkpoint 5 API contract: complete runtime surface",
  ).toEqual([
    "pub fn new",
    "pub fn initialize_account",
    "pub fn is_authenticated",
    "pub fn request_login_code",
    "pub fn sign_in",
    "pub fn authorized_client",
    "pub(super) fn initialized_client",
    "pub fn clear_account",
  ]);
  for (const wrapper of [
    "TelegramApiHash",
    "TelegramClientHandle",
    "TelegramLoginAttempt",
    "TelegramRuntime",
  ]) {
    expect(
      explicitTraitImplInventory(runtimeImpls, wrapper),
      `Checkpoint 5 secrecy contract: ${wrapper} has no conversion/raw trait surface`,
    ).toEqual([]);
  }

  expect(
    runtimeStructure,
    "Checkpoint 5 secrecy contract: no secret Debug/getter/exposure trait",
  ).not.toMatch(
    /(?:derive\s*\([^)]*\bDebug\b[^)]*\)\s*pub struct TelegramApiHash|impl\s+(?:std::fmt::)?Debug\s+for\s+TelegramApiHash|impl\s+(?:secrecy::)?ExposeSecret\s+for\s+TelegramApiHash|\bpub(?:\s*\([^)]*\))?\s+fn\s+(?:as_str|expose_secret|api_hash|raw_hash)\s*\()/,
  );
  expect(
    countMatches(runtimeStructure, /\.expose_secret\s*\(\s*\)/g),
    "Checkpoint 5 secrecy contract: sole Grammers API-hash sink",
  ).toBe(1);
  expect(
    runtimeStructure,
    "Checkpoint 5 ownership contract: runtime owns no app capabilities",
  ).not.toMatch(
    /\b(?:sqlx|tauri|SecretStore|SecretStoreState|keyring|AppHandle|PathBuf)\b|std::(?:fs|path)\b|\bcrate::error\b/,
  );
  expect(
    runtimeUnmaskedStructure,
    "Checkpoint 5 API contract: callback seam stays private and test-only",
  ).toMatch(
    /#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]\s*type TelegramRuntimeTestFuture\b/,
  );
  expect(
    runtimeUnmaskedStructure,
    "Checkpoint 5 API contract: no callback seam item is visible",
  ).not.toMatch(
    /\bpub(?:\s*\([^)]*\))?\s+(?:type|struct|fn)\s+TelegramRuntimeTest/,
  );

  expect(
    normalizedRustUseDeclarations(telegramProduction, "runtime"),
    "Checkpoint 5 API contract: curated runtime facade",
  ).toEqual([
    "pub(crate) use runtime::{TelegramApiHash, TelegramClientHandle, TelegramRuntime, TelegramRuntimeStatus,};",
  ]);
  expect(
    normalizedRustFunctionDeclaration(telegramProduction, "get_client"),
    "Checkpoint 5 API contract: exact initialized lookup facade",
  ).toBe(
    "pub(crate) async fn get_client(state: &TelegramState, account_id: i64,) -> extractum_core::error::AppResult<TelegramClientHandle>",
  );
  expect(
    normalizedRustFunctionDeclaration(
      telegramProduction,
      "get_authorized_client",
    ),
    "Checkpoint 5 API contract: exact authorized lookup facade",
  ).toBe(
    "pub(crate) async fn get_authorized_client(state: &TelegramState, account_id: i64,) -> extractum_core::error::AppResult<TelegramClientHandle>",
  );
  expect(rustFunctionBody(telegramProduction, "get_client").trim()).toBe(
    "state.runtime.initialized_client(account_id).await",
  );
  expect(
    rustFunctionBody(telegramProduction, "get_authorized_client").trim(),
  ).toBe("state.runtime.authorized_client(account_id).await");
  expect(
    appProductionStructure,
    "Checkpoint 5 ownership contract: legacy runtime seams are absent",
  ).not.toMatch(
    /\b(?:AccountClient|AuthorizedTelegramRuntime|get_authorized_runtime)\b/,
  );

  expect(
    normalize(rustStructBody(telegramProduction, "AccountCredentialsRow"))
      .replace(/\s+/g, " ")
      .trim(),
    "Checkpoint 5 app contract: SQL row retains the only app plaintext hash field",
  ).toBe("id: i64, api_id: i64, api_hash: String,");
  expect(
    normalize(rustStructBody(telegramProduction, "AccountCredentials"))
      .replace(/\s+/g, " ")
      .trim(),
    "Checkpoint 5 app contract: resolved credentials carry only the opaque hash",
  ).toBe("id: i64, api_id: i64, api_hash: TelegramApiHash,");
  expect(
    countMatches(telegramStructure, /\.expose_secret\s*\(\s*\)/g),
    "Checkpoint 5 secrecy contract: exact app secure-store validation/write sinks",
  ).toBe(2);
  expect(
    rustFunctionBody(telegramProduction, "resolve_account_credentials"),
    "Checkpoint 5 secrecy contract: secure-store reads are never stringified or cloned",
  ).not.toMatch(
    /api_hash\.(?:to_string|clone)\s*\(|expose_secret\s*\(\s*\)\s*\.(?:to_string|clone)\s*\(|credentials\.api_hash\s*=/,
  );
  expect(
    telegramStructure,
    "Checkpoint 5 secrecy contract: no plaintext hash log or serialization path",
  ).not.toMatch(
    /(?:e?println!|dbg!|tracing::\w+!|serde_json::to_\w+)\s*\([^;]*api_hash/,
  );

  const storeBodies = [
    ["list_telegram_sources", "account_id"],
    ["add_telegram_source", "request.account_id"],
  ] as const;
  for (const [functionName, accountExpression] of storeBodies) {
    const body = rustFunctionBody(storeProduction, functionName);
    expect(
      normalize(body),
      `Checkpoint 5 consumer contract: ${functionName} uses the initialized handle`,
    ).toContain(
      `crate::telegram::get_client(state.inner(), ${accountExpression}).await?`,
    );
    expect(
      rustNamedCallCount(body, "get_client"),
      `Checkpoint 5 consumer contract: ${functionName} sole initialized lookup`,
    ).toBe(1);
    expect(
      rustIdentifierReferenceInventory(body, "raw_client"),
      `Checkpoint 5 consumer contract: ${functionName} sole raw client`,
    ).toEqual(["call"]);
    expect(
      body,
      `Checkpoint 5 consumer contract: ${functionName} owns no caller lock`,
    ).not.toMatch(
      /\bstate\.accounts\b|\.accounts\s*\.lock\s*\(|get_client\s*\(\s*&accounts/,
    );
  }

  const syncBody = rustFunctionBody(syncProduction, "sync_telegram_source");
  expect(normalize(syncBody)).toContain(
    "crate::telegram::get_authorized_client(state.inner(), account_id).await?",
  );
  expect(
    rustNamedCallCount(syncBody, "get_authorized_client"),
    "Checkpoint 5 consumer contract: sync sole authorized lookup",
  ).toBe(1);
  expect(
    rustIdentifierReferenceInventory(syncBody, "raw_client"),
    "Checkpoint 5 consumer contract: sync sole raw client",
  ).toEqual(["call"]);
  expect(
    rustIdentifierReferenceInventory(syncBody, "raw_session"),
    "Checkpoint 5 consumer contract: sync owns no raw session",
  ).toEqual([]);

  const takeoutWorkflows = [
    "run_export_dc_spike_for_handle",
    "run_takeout_migrated_history_import",
    "run_takeout_source_import",
  ] as const;
  for (const functionName of takeoutWorkflows) {
    const body = rustFunctionBody(takeoutProduction, functionName);
    expect(
      rustIdentifierReferenceInventory(body, "raw_client"),
      `Checkpoint 5 consumer contract: ${functionName} raw client`,
    ).toEqual(["call"]);
    expect(
      rustIdentifierReferenceInventory(body, "raw_session"),
      `Checkpoint 5 consumer contract: ${functionName} raw session`,
    ).toEqual(["call"]);
  }
  const takeoutLookupWorkflows = [
    ["run_takeout_export_dc_spike", "state.inner()"],
    ["run_takeout_migrated_history_import", "telegram_state.inner()"],
    ["run_takeout_source_import", "telegram_state.inner()"],
  ] as const;
  for (const [functionName, stateExpression] of takeoutLookupWorkflows) {
    const body = rustFunctionBody(takeoutProduction, functionName);
    expect(
      normalize(body),
      `Checkpoint 5 consumer contract: ${functionName} authorized lookup arguments`,
    ).toContain(
      `get_authorized_client(${stateExpression}, account_id).await?`,
    );
    expect(
      rustNamedCallCount(body, "get_authorized_client"),
      `Checkpoint 5 consumer contract: ${functionName} sole authorized lookup`,
    ).toBe(1);
  }
  expect(
    normalizedRustFunctionDeclaration(
      takeoutProduction,
      "run_export_dc_spike_for_handle",
    ),
    "Checkpoint 5 consumer contract: exact Takeout spike handle",
  ).toBe(
    "async fn run_export_dc_spike_for_handle(source_id: i64, account_id: i64, telegram_source_subtype: &str, handle: TelegramClientHandle,) -> AppResult<TakeoutExportDcSpikeResult>",
  );

  const initializedLookupSites = [...appSources.entries()].flatMap(
    ([relativePath, source]) =>
      Array.from(
        { length: rustNamedCallCount(
          maskExactCfgTestItemSpans(source),
          "get_client",
        ) },
        (_, index) => `${relativePath}#${index + 1}`,
      ),
  );
  const identifierReferenceSites = (name: string): string[] =>
    [...appSources.entries()].flatMap(([relativePath, source]) => {
      const counts = new Map<string, number>();
      return rustIdentifierReferenceInventory(
        maskExactCfgTestItemSpans(source),
        name,
      ).map((kind) => {
        const occurrence = (counts.get(kind) ?? 0) + 1;
        counts.set(kind, occurrence);
        return `${relativePath}|${kind}#${occurrence}`;
      });
    });
  const authorizedLookupSites = [...appSources.entries()].flatMap(
    ([relativePath, source]) =>
      Array.from(
        { length: rustNamedCallCount(
          maskExactCfgTestItemSpans(source),
          "get_authorized_client",
        ) },
        (_, index) => `${relativePath}#${index + 1}`,
      ),
  );
  exactInventory(
    identifierReferenceSites("get_client"),
    [
      `${storePath}|call#1`,
      `${storePath}|call#2`,
      `${telegramPath}|declaration#1`,
    ],
    "Checkpoint 5 consumer contract: exact initialized-lookup identifier references",
  );
  exactInventory(
    identifierReferenceSites("get_authorized_client"),
    [
      `${syncPath}|call#1`,
      `${takeoutPath}|import#1`,
      `${takeoutPath}|call#1`,
      `${takeoutPath}|call#2`,
      `${takeoutPath}|call#3`,
      `${telegramPath}|declaration#1`,
    ],
    "Checkpoint 5 consumer contract: exact authorized-lookup identifier references",
  );
  exactInventory(
    initializedLookupSites,
    [
      `${storePath}#1`,
      `${storePath}#2`,
    ],
    "Checkpoint 5 consumer contract: exact initialized-lookup call sites",
  );
  exactInventory(
    authorizedLookupSites,
    [
      `${syncPath}#1`,
      `${takeoutPath}#1`,
      `${takeoutPath}#2`,
      `${takeoutPath}#3`,
    ],
    "Checkpoint 5 consumer contract: exact authorized-lookup call sites",
  );
  exactInventory(
    identifierReferenceSites("raw_client"),
    [
      `${runtimePath}|declaration#1`,
      `${storePath}|call#1`,
      `${storePath}|call#2`,
      `${syncPath}|call#1`,
      `${takeoutPath}|call#1`,
      `${takeoutPath}|call#2`,
      `${takeoutPath}|call#3`,
    ],
    "Checkpoint 5 consumer contract: exact raw-client identifier references",
  );
  exactInventory(
    identifierReferenceSites("raw_session"),
    [
      `${runtimePath}|declaration#1`,
      `${takeoutPath}|call#1`,
      `${takeoutPath}|call#2`,
      `${takeoutPath}|call#3`,
    ],
    "Checkpoint 5 consumer contract: exact raw-session identifier references",
  );
  expect(
    runtimeStructure,
    "Checkpoint 5 API contract: adapters have no dead-code escape hatch",
  ).not.toMatch(
    /#\s*\[\s*allow\s*\(\s*dead_code\s*\)\s*\]\s*pub\s*\(\s*crate\s*\)\s+fn\s+raw_(?:client|session)/,
  );

  expectOrdered(
    rustFunctionBody(telegramProduction, "init_account_client"),
    [
      "STATUS_RESTORING",
      "load_session(handle, secret_store, account_id).await?",
      ".initialize_account(account_id, api_id, api_hash, session)",
      "runtime_status_to_wire(runtime_status)",
      "runtime_status == TelegramRuntimeStatus::Ready",
    ],
    "Checkpoint 5 app contract: restore-runtime-status order",
  );
  for (const functionName of [
    "restore_telegram_accounts",
    "tg_init",
  ]) {
    const body = rustFunctionBody(telegramProduction, functionName);
    expectOrdered(
      body,
      [
        "init_account_client(",
        "runtime.clear_account(",
        "STATUS_RESTORE_FAILED",
      ],
      `Checkpoint 5 app contract: ${functionName} clears the current entry before restore_failed`,
    );
  }
  expectOrdered(
    rustFunctionBody(telegramProduction, "clear_account_runtime"),
    [
      "runtime.clear_account(account_id, sign_out).await",
      "delete_session(handle, secret_store, account_id).await?",
      "STATUS_NOT_INITIALIZED",
    ],
    "Checkpoint 5 app contract: runtime-file-status clear order",
  );
  expectOrdered(
    rustFunctionBody(telegramProduction, "tg_sign_in"),
    [
      "runtime.sign_in(account_id, code).await?",
      "save_session(&handle, &secret_store, account_id, &session_to_save)",
      "STATUS_READY",
      "Ok(true)",
    ],
    "Checkpoint 5 app contract: runtime-save-status sign-in order",
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

function rustStructBody(source: string, name: string): string {
  const searchable = maskRustLexicalNonCode(source);
  const declaration = new RegExp(`\\bstruct\\s+${name}\\s*\\{`).exec(
    searchable,
  );
  if (!declaration || declaration.index === undefined) {
    throw new Error(`Missing Rust struct: ${name}`);
  }
  const open = searchable.indexOf("{", declaration.index);
  const close = rustClosingBraceIndex(source, open, `struct ${name}`);
  return maskRustLexicalNonCode(source.slice(open + 1, close));
}

function rustBraceDepthBefore(source: string, end: number): number {
  let depth = 0;
  for (let index = 0; index < end; index += 1) {
    if (source[index] === "{") depth += 1;
    if (source[index] === "}") depth -= 1;
    if (depth < 0) {
      throw new Error(
        "Checkpoint 4 Rust inventory encountered an unmatched closing brace",
      );
    }
  }
  return depth;
}

type RustImplBlock = Readonly<{ header: string; body: string }>;

function isRustImplItemCandidate(
  searchable: string,
  implIndex: number,
): boolean {
  let cursor = implIndex;
  while (cursor > 0) {
    while (cursor > 0 && /\s/.test(searchable[cursor - 1] ?? "")) {
      cursor -= 1;
    }
    const qualifier = /([A-Za-z_][A-Za-z0-9_]*)$/.exec(
      searchable.slice(0, cursor),
    )?.[1];
    if (
      qualifier === undefined
      || !["const", "default", "unsafe"].includes(qualifier)
    ) {
      break;
    }
    cursor -= qualifier.length;
  }
  while (cursor > 0 && /\s/.test(searchable[cursor - 1] ?? "")) {
    cursor -= 1;
  }
  return cursor === 0 || /[{};\]]/.test(searchable[cursor - 1] ?? "");
}

function rustImplBlocks(
  source: string,
  label = "Checkpoint 4 impl inventory",
): RustImplBlock[] {
  const searchable = maskRustLexicalNonCode(source);
  const blocks: Array<{ header: string; body: string }> = [];
  for (const match of searchable.matchAll(/\bimpl\b/g)) {
    if (match.index === undefined) continue;
    if (!isRustImplItemCandidate(searchable, match.index)) continue;
    const headerStart = match.index + match[0].length;
    const boundary = rustItemBoundary(
      source,
      searchable,
      headerStart,
      label,
    );
    if (boundary.kind !== "body") {
      throw new Error(
        "Checkpoint 4 Rust inventory found an impl without a body",
      );
    }
    const open = boundary.index;
    const close = rustClosingBraceIndex(
      source,
      open,
      label,
    );
    blocks.push({
      header: normalize(searchable.slice(headerStart, open)).trim(),
      body: searchable.slice(open + 1, close),
    });
  }
  return blocks;
}

type RustAuditToken = Readonly<{
  kind: "word" | "punctuation";
  text: string;
  start: number;
  end: number;
  rawIdentifier: boolean;
}>;

const rustPatternWhitespace = new Set([
  "\u0009",
  "\u000A",
  "\u000B",
  "\u000C",
  "\u000D",
  "\u0020",
  "\u0085",
  "\u200E",
  "\u200F",
  "\u2028",
  "\u2029",
]);
const rust2021StrictOrReservedKeywords = new Set([
  "abstract",
  "as",
  "async",
  "await",
  "become",
  "box",
  "break",
  "const",
  "continue",
  "crate",
  "do",
  "dyn",
  "else",
  "enum",
  "extern",
  "false",
  "final",
  "fn",
  "for",
  "if",
  "impl",
  "in",
  "let",
  "loop",
  "macro",
  "match",
  "mod",
  "move",
  "mut",
  "override",
  "priv",
  "pub",
  "ref",
  "return",
  "self",
  "Self",
  "static",
  "struct",
  "super",
  "trait",
  "true",
  "try",
  "type",
  "typeof",
  "unsafe",
  "unsized",
  "use",
  "virtual",
  "where",
  "while",
  "yield",
]);
const rustSimplePathKeywords = new Set(["crate", "self", "Self", "super"]);

function isConservativeRustWordStart(codePoint: string): boolean {
  if (codePoint.length === 0 || rustPatternWhitespace.has(codePoint)) {
    return false;
  }
  const scalar = codePoint.codePointAt(0);
  if (scalar === undefined) return false;
  if (scalar > 0x7F) return true;
  return codePoint === "_"
    || (codePoint >= "A" && codePoint <= "Z")
    || (codePoint >= "a" && codePoint <= "z");
}

function isConservativeRustWordContinue(codePoint: string): boolean {
  if (isConservativeRustWordStart(codePoint)) return true;
  return codePoint >= "0" && codePoint <= "9";
}

function rustCodePointAt(
  source: string,
  index: number,
): Readonly<{ text: string; width: number }> {
  const value = source.codePointAt(index);
  if (value === undefined) return { text: "", width: 0 };
  const text = String.fromCodePoint(value);
  return { text, width: text.length };
}

function rustAuditTokens(searchable: string): RustAuditToken[] {
  const tokens: RustAuditToken[] = [];
  for (let index = 0; index < searchable.length;) {
    const codePoint = rustCodePointAt(searchable, index);
    if (rustPatternWhitespace.has(codePoint.text)) {
      index += codePoint.width;
      continue;
    }

    if (searchable.startsWith("r#", index)) {
      const rawStart = rustCodePointAt(searchable, index + 2);
      if (isConservativeRustWordStart(rawStart.text)) {
        let cursor = index + 2 + rawStart.width;
        while (cursor < searchable.length) {
          const next = rustCodePointAt(searchable, cursor);
          if (!isConservativeRustWordContinue(next.text)) break;
          cursor += next.width;
        }
        tokens.push({
          kind: "word",
          text: searchable.slice(index, cursor),
          start: index,
          end: cursor,
          rawIdentifier: true,
        });
        index = cursor;
        continue;
      }
    }

    if (isConservativeRustWordStart(codePoint.text)) {
      let cursor = index + codePoint.width;
      while (cursor < searchable.length) {
        const next = rustCodePointAt(searchable, cursor);
        if (!isConservativeRustWordContinue(next.text)) break;
        cursor += next.width;
      }
      tokens.push({
        kind: "word",
        text: searchable.slice(index, cursor),
        start: index,
        end: cursor,
        rawIdentifier: false,
      });
      index = cursor;
      continue;
    }

    tokens.push({
      kind: "punctuation",
      text: codePoint.text,
      start: index,
      end: index + codePoint.width,
      rawIdentifier: false,
    });
    index += codePoint.width;
  }
  return tokens;
}

function normalizedRustAuditIdentifierAt(
  tokens: readonly RustAuditToken[],
  tokenIndex: number,
): string | undefined {
  const token = tokens[tokenIndex];
  if (token?.kind !== "word") return undefined;
  const preceding = tokens[tokenIndex - 1];
  if (preceding?.kind === "punctuation" && preceding.text === "'") {
    return undefined;
  }
  return token.rawIdentifier ? token.text.slice(2) : token.text;
}

function isRustAuditKeywordAt(
  tokens: readonly RustAuditToken[],
  tokenIndex: number,
  keyword: string,
): boolean {
  const token = tokens[tokenIndex];
  return token?.kind === "word"
    && !token.rawIdentifier
    && token.text === keyword;
}

function rustAuditDelimiterPairs(
  tokens: readonly RustAuditToken[],
  label: string,
): ReadonlyMap<number, number> {
  const closingFor = new Map([
    ["(", ")"],
    ["[", "]"],
    ["{", "}"],
  ]);
  const closing = new Set(closingFor.values());
  const stack: Array<{
    tokenIndex: number;
    expected: string;
  }> = [];
  const pairs = new Map<number, number>();
  for (let tokenIndex = 0; tokenIndex < tokens.length; tokenIndex += 1) {
    const token = tokens[tokenIndex];
    const expected = closingFor.get(token.text);
    if (expected !== undefined) {
      stack.push({ tokenIndex, expected });
      continue;
    }
    if (!closing.has(token.text)) continue;
    const open = stack.at(-1);
    if (open === undefined || open.expected !== token.text) {
      throw new Error(
        `${label} found a mismatched closing delimiter at offset ${token.start}`,
      );
    }
    stack.pop();
    pairs.set(open.tokenIndex, tokenIndex);
  }
  const unclosed = stack.at(-1);
  if (unclosed !== undefined) {
    throw new Error(
      `${label} found an unclosed delimiter at offset ${
        tokens[unclosed.tokenIndex].start
      }`,
    );
  }
  return pairs;
}

function canEndRust2021SimplePathSegment(
  tokens: readonly RustAuditToken[],
  tokenIndex: number,
): boolean {
  const token = tokens[tokenIndex];
  const identifier = normalizedRustAuditIdentifierAt(tokens, tokenIndex);
  if (token?.kind !== "word" || identifier === undefined) return false;
  if (token.rawIdentifier) return true;
  if (rustSimplePathKeywords.has(identifier)) return true;
  return !rust2021StrictOrReservedKeywords.has(identifier);
}

function rustAuditLineColumn(
  source: string,
  offset: number,
): Readonly<{ line: number; column: number }> {
  let line = 1;
  let column = 1;
  for (let index = 0; index < offset;) {
    const codePoint = rustCodePointAt(source, index);
    if (codePoint.text === "\r") {
      const next = rustCodePointAt(source, index + codePoint.width);
      index += codePoint.width;
      if (next.text === "\n") index += next.width;
      line += 1;
      column = 1;
      continue;
    }
    index += codePoint.width;
    if (codePoint.text === "\n") {
      line += 1;
      column = 1;
    } else {
      column += 1;
    }
  }
  return { line, column };
}

function rustSensitiveMacroInventory(
  source: string,
  sensitiveNames: readonly string[],
  label: string,
): string[] {
  const searchable = maskRustLexicalNonCode(source);
  const tokens = rustAuditTokens(searchable);
  const delimiterPairs = rustAuditDelimiterPairs(tokens, label);
  const sensitiveSet = new Set(sensitiveNames);
  const inventory: string[] = [];
  const referencedSensitiveNames = (
    start: number,
    end: number,
  ): Set<string> => {
    const referenced = new Set<string>();
    for (let tokenIndex = start; tokenIndex < end; tokenIndex += 1) {
      const identifier = normalizedRustAuditIdentifierAt(tokens, tokenIndex);
      if (identifier === undefined) continue;
      if (sensitiveSet.has(identifier)) referenced.add(identifier);
    }
    return referenced;
  };
  for (let tokenIndex = 0; tokenIndex < tokens.length; tokenIndex += 1) {
    if (
      isRustAuditKeywordAt(tokens, tokenIndex, "macro_rules")
      && tokens[tokenIndex + 1]?.text === "!"
      && normalizedRustAuditIdentifierAt(tokens, tokenIndex + 2)
        !== undefined
    ) {
      const openIndex = tokenIndex + 3;
      const closeIndex = delimiterPairs.get(openIndex);
      if (closeIndex !== undefined) {
        const position = rustAuditLineColumn(
          source,
          tokens[tokenIndex].start,
        );
        for (
          const sensitiveName
          of referencedSensitiveNames(openIndex + 1, closeIndex)
        ) {
          inventory.push(
            `macro_rules@${tokens[tokenIndex].start}/${
              position.line
            }:${position.column}|${sensitiveName}`,
          );
        }
      }
    }

    const bang = tokens[tokenIndex];
    if (
      bang.kind !== "punctuation"
      || bang.text !== "!"
      || !canEndRust2021SimplePathSegment(tokens, tokenIndex - 1)
    ) {
      continue;
    }
    const openIndex = tokenIndex + 1;
    const closeIndex = delimiterPairs.get(openIndex);
    if (closeIndex === undefined) continue;
    const referencedNames = referencedSensitiveNames(
      openIndex + 1,
      closeIndex,
    );
    if (referencedNames.size === 0) continue;
    const position = rustAuditLineColumn(source, bang.start);
    for (const sensitiveName of referencedNames) {
      inventory.push(
        `bang@${bang.start}/${position.line}:${
          position.column
        }|${sensitiveName}`,
      );
    }
  }
  return inventory;
}

function rustTypeReferenceRegex(typeName: string): RegExp {
  return new RegExp(
    `(?:^|[^A-Za-z0-9_#])(?:r#)?${typeName}\\b`,
  );
}

function rustAuditSemicolonEnd(
  tokens: readonly RustAuditToken[],
  delimiterPairs: ReadonlyMap<number, number>,
  start: number,
): number | undefined {
  for (let tokenIndex = start; tokenIndex < tokens.length; tokenIndex += 1) {
    const paired = delimiterPairs.get(tokenIndex);
    if (paired !== undefined) {
      tokenIndex = paired;
      continue;
    }
    const text = tokens[tokenIndex].text;
    if (text === ";") return tokenIndex;
    if (text === ")" || text === "]" || text === "}") return undefined;
  }
  return undefined;
}

function rustAuditTopLevelTokenIndex(
  tokens: readonly RustAuditToken[],
  delimiterPairs: ReadonlyMap<number, number>,
  start: number,
  end: number,
  predicate: (token: RustAuditToken, tokenIndex: number) => boolean,
): number | undefined {
  for (let tokenIndex = start; tokenIndex < end; tokenIndex += 1) {
    const token = tokens[tokenIndex];
    if (predicate(token, tokenIndex)) return tokenIndex;
    const paired = delimiterPairs.get(tokenIndex);
    if (paired !== undefined && paired < end) tokenIndex = paired;
  }
  return undefined;
}

function rustAuditPathIdentifiers(
  tokens: readonly RustAuditToken[],
  start: number,
  end: number,
): string[] {
  const identifiers: string[] = [];
  for (let tokenIndex = start; tokenIndex < end; tokenIndex += 1) {
    const identifier = normalizedRustAuditIdentifierAt(tokens, tokenIndex);
    if (identifier !== undefined) identifiers.push(identifier);
  }
  return identifiers;
}

function rustUseTreeAliasNames(
  tokens: readonly RustAuditToken[],
  delimiterPairs: ReadonlyMap<number, number>,
  wrapperNames: ReadonlySet<string>,
  start: number,
  end: number,
  prefix: readonly string[] = [],
): string[] {
  const openBrace = rustAuditTopLevelTokenIndex(
    tokens,
    delimiterPairs,
    start,
    end,
    (token) => token.text === "{",
  );
  if (openBrace !== undefined) {
    const closeBrace = delimiterPairs.get(openBrace);
    if (closeBrace === undefined || closeBrace > end) return [];
    const nextPrefix = [
      ...prefix,
      ...rustAuditPathIdentifiers(tokens, start, openBrace),
    ];
    const aliases: string[] = [];
    let itemStart = openBrace + 1;
    for (
      let tokenIndex = openBrace + 1;
      tokenIndex < closeBrace;
      tokenIndex += 1
    ) {
      const paired = delimiterPairs.get(tokenIndex);
      if (paired !== undefined && paired < closeBrace) {
        tokenIndex = paired;
        continue;
      }
      if (tokens[tokenIndex].text !== ",") continue;
      aliases.push(
        ...rustUseTreeAliasNames(
          tokens,
          delimiterPairs,
          wrapperNames,
          itemStart,
          tokenIndex,
          nextPrefix,
        ),
      );
      itemStart = tokenIndex + 1;
    }
    aliases.push(
      ...rustUseTreeAliasNames(
        tokens,
        delimiterPairs,
        wrapperNames,
        itemStart,
        closeBrace,
        nextPrefix,
      ),
    );
    return aliases;
  }

  const asIndex = rustAuditTopLevelTokenIndex(
    tokens,
    delimiterPairs,
    start,
    end,
    (_, tokenIndex) => isRustAuditKeywordAt(tokens, tokenIndex, "as"),
  );
  if (asIndex === undefined) return [];
  const alias = normalizedRustAuditIdentifierAt(tokens, asIndex + 1);
  if (alias === undefined) return [];
  const leaf = rustAuditPathIdentifiers(tokens, start, asIndex);
  const target = leaf.at(-1) === "self"
    ? prefix.at(-1)
    : leaf.at(-1);
  return target !== undefined && wrapperNames.has(target) ? [alias] : [];
}

function rustWrapperAliasNames(
  source: string,
  typeNames: readonly string[],
): string[] {
  const searchable = maskRustLexicalNonCode(source);
  const tokens = rustAuditTokens(searchable);
  const delimiterPairs = rustAuditDelimiterPairs(
    tokens,
    "Rust wrapper alias inventory",
  );
  const wrapperNames = new Set(typeNames);
  const aliases: string[] = [];

  for (let tokenIndex = 0; tokenIndex < tokens.length; tokenIndex += 1) {
    if (isRustAuditKeywordAt(tokens, tokenIndex, "type")) {
      const alias = normalizedRustAuditIdentifierAt(tokens, tokenIndex + 1);
      const semicolon = rustAuditSemicolonEnd(
        tokens,
        delimiterPairs,
        tokenIndex + 1,
      );
      if (alias === undefined || semicolon === undefined) continue;
      const equals = rustAuditTopLevelTokenIndex(
        tokens,
        delimiterPairs,
        tokenIndex + 2,
        semicolon,
        (token) => token.text === "=",
      );
      if (equals === undefined) continue;
      const targetsWrapper = Array.from(
        { length: semicolon - equals - 1 },
        (_, offset) => normalizedRustAuditIdentifierAt(
          tokens,
          equals + offset + 1,
        ),
      ).some((identifier) =>
        identifier !== undefined && wrapperNames.has(identifier)
      );
      if (targetsWrapper) aliases.push(alias);
      tokenIndex = semicolon;
      continue;
    }

    if (isRustAuditKeywordAt(tokens, tokenIndex, "use")) {
      const semicolon = rustAuditSemicolonEnd(
        tokens,
        delimiterPairs,
        tokenIndex + 1,
      );
      if (semicolon === undefined) continue;
      aliases.push(
        ...rustUseTreeAliasNames(
          tokens,
          delimiterPairs,
          wrapperNames,
          tokenIndex + 1,
          semicolon,
        ),
      );
      tokenIndex = semicolon;
    }
  }
  return aliases;
}

function rustTopLevelItemHeaders(
  body: string,
  label: string,
): string[] {
  const headers: string[] = [];
  let cursor = 0;
  while (cursor < body.length) {
    while (/\s/.test(body[cursor] ?? "")) cursor += 1;
    if (cursor >= body.length) break;
    const boundary = rustItemBoundary(
      body,
      body,
      cursor,
      label,
    );
    const header = normalize(body.slice(cursor, boundary.index)).trim();
    if (header.length > 0) headers.push(header);
    cursor = boundary.kind === "semicolon"
      ? boundary.index + 1
      : rustClosingBraceIndex(body, boundary.index, label) + 1;
  }
  return headers;
}

type RustVisibleFreeFunction = Readonly<{
  header: string;
  signature: string;
}>;

function visibleRustFreeFunctions(
  source: string,
  label: string,
): RustVisibleFreeFunction[] {
  const searchable = maskRustLexicalNonCode(source);
  const inventory: RustVisibleFreeFunction[] = [];
  const visitScope = (start: number, end: number): void => {
    let cursor = start;
    while (cursor < end) {
      while (cursor < end && /\s/.test(searchable[cursor] ?? "")) {
        cursor += 1;
      }
      if (cursor >= end) break;
      const boundary = rustItemBoundary(
        source,
        searchable,
        cursor,
        label,
      );
      const header = normalize(
        searchable.slice(cursor, boundary.index),
      ).trim();
      const visibleFunction =
        /\bpub(?:\s*\(\s*[^)]+?\s*\))?\s+(?:(?:async|const|default|unsafe)\s+)*(?:extern\s+)?fn\s+(?:r#)?[A-Za-z_][A-Za-z0-9_]*\b/
          .exec(header);
      if (visibleFunction?.index !== undefined) {
        inventory.push({
          header,
          signature: header.slice(
            visibleFunction.index + visibleFunction[0].length,
          ),
        });
      }
      if (boundary.kind === "semicolon") {
        cursor = boundary.index + 1;
        continue;
      }
      const close = rustClosingBraceIndex(
        source,
        boundary.index,
        label,
      );
      if (
        /\bmod\s+(?:r#)?[A-Za-z_][A-Za-z0-9_]*\s*$/.test(header)
      ) {
        visitScope(boundary.index + 1, close);
      }
      cursor = close + 1;
    }
  };
  visitScope(0, searchable.length);
  return inventory;
}

function splitLeadingRustOuterAttributes(header: string): {
  attributes: string[];
  declaration: string;
} {
  const attributes: string[] = [];
  let cursor = 0;
  while (cursor < header.length) {
    while (/\s/.test(header[cursor] ?? "")) cursor += 1;
    if (header[cursor] !== "#") break;
    let bracket = cursor + 1;
    while (/\s/.test(header[bracket] ?? "")) bracket += 1;
    if (header[bracket] !== "[") break;
    let depth = 1;
    bracket += 1;
    while (bracket < header.length && depth > 0) {
      if (header[bracket] === "[") depth += 1;
      if (header[bracket] === "]") depth -= 1;
      bracket += 1;
    }
    if (depth !== 0) {
      throw new Error(
        "Checkpoint 5 visible-item inventory found an unclosed outer attribute",
      );
    }
    attributes.push(header.slice(cursor, bracket).trim());
    cursor = bracket;
  }
  return {
    attributes,
    declaration: header.slice(cursor).trim(),
  };
}

function rustStructOuterAttributeInventory(
  source: string,
  typeNames: readonly string[],
  label: string,
): string[] {
  const typeNameSet = new Set(typeNames);
  return rustTopLevelItemHeaders(
    maskRustLexicalNonCode(source),
    label,
  ).flatMap((header) => {
    const { attributes, declaration } =
      splitLeadingRustOuterAttributes(header);
    const tokens = rustAuditTokens(declaration);
    const structIndex = tokens.findIndex((_, tokenIndex) =>
      isRustAuditKeywordAt(tokens, tokenIndex, "struct")
    );
    if (structIndex < 0) return [];
    const typeName = normalizedRustAuditIdentifierAt(
      tokens,
      structIndex + 1,
    );
    if (typeName === undefined || !typeNameSet.has(typeName)) return [];
    return [
      `${typeName}|${
        attributes.map((attribute) => normalize(attribute).trim()).join("|")
      }`,
    ];
  });
}

function visibleRustTopLevelItemInventory(
  source: string,
  label: string,
): string[] {
  const searchable = maskRustLexicalNonCode(source);
  const inventory: string[] = [];
  const visitScope = (
    start: number,
    end: number,
    scope: readonly string[],
  ): void => {
    let cursor = start;
    while (cursor < end) {
      while (cursor < end && /\s/.test(searchable[cursor] ?? "")) {
        cursor += 1;
      }
      if (cursor >= end) break;
      const boundary = rustItemBoundary(
        source,
        searchable,
        cursor,
        label,
      );
      const header = normalize(
        searchable.slice(cursor, boundary.index),
      ).trim();
      const { attributes, declaration } =
        splitLeadingRustOuterAttributes(header);
      const testOnly = attributes.some((attribute) =>
        /^#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]$/.test(attribute)
      );
      const visibility =
        /^pub(?:\s*\(\s*([^)]+?)\s*\))?(?=\s)/.exec(declaration);
      const macroExport = attributes.some((attribute) =>
        /^#\s*\[\s*macro_export(?:\s*\([^)]*\))?\s*\]$/.test(
          attribute,
        )
      );
      const itemDeclaration = visibility
        ? declaration.slice(visibility[0].length).trim()
        : declaration;
      if (!testOnly && (visibility || macroExport)) {
        const normalizedVisibility = macroExport && !visibility
          ? "pub"
          : visibility?.[1] === undefined
          ? "pub"
          : `pub(${visibility[1].replace(/\s+/g, "")})`;
        let item: string | undefined;
        const functionItem =
          /^(?:(?:async|const|default|unsafe)\s+)*(?:extern\s+)?fn\s+(?:r#)?([A-Za-z_][A-Za-z0-9_]*)\b/
            .exec(itemDeclaration);
        const namedItem =
          /^(?:unsafe\s+|auto\s+)?(struct|enum|union|trait|type|const|static|mod|macro)\s+(?:mut\s+)?(?:r#)?([A-Za-z_][A-Za-z0-9_]*)\b/
            .exec(itemDeclaration);
        const macroRules =
          /^macro_rules\s*!\s*(?:r#)?([A-Za-z_][A-Za-z0-9_]*)\b/
            .exec(itemDeclaration);
        if (functionItem?.[1] !== undefined) {
          item = `fn ${functionItem[1]}`;
        } else if (namedItem?.[2] !== undefined) {
          item = `${namedItem[1]} ${namedItem[2]}`;
        } else if (macroRules?.[1] !== undefined) {
          item = `macro ${macroRules[1]}`;
        } else if (/^use\b/.test(itemDeclaration)) {
          item = "use";
        } else if (/^extern\s+crate\b/.test(itemDeclaration)) {
          item = "extern crate";
        }
        if (!item || !normalizedVisibility) {
          throw new Error(
            `${label} found an unsupported visible top-level item`,
          );
        }
        inventory.push(
          `${scope.length === 0 ? "root" : scope.join("::")}|${normalizedVisibility} ${item}`,
        );
      }
      if (boundary.kind === "semicolon") {
        cursor = boundary.index + 1;
        continue;
      }
      const close = rustClosingBraceIndex(
        source,
        boundary.index,
        label,
      );
      const inlineModule =
        /^mod\s+(?:r#)?([A-Za-z_][A-Za-z0-9_]*)\s*$/.exec(
          itemDeclaration,
        );
      if (!testOnly && inlineModule?.[1] !== undefined) {
        visitScope(
          boundary.index + 1,
          close,
          [...scope, inlineModule[1]],
        );
      }
      cursor = close + 1;
    }
  };
  visitScope(0, searchable.length, []);
  return inventory;
}

function visibleInherentAssociatedItemInventory(
  implBlocks: ReadonlyArray<RustImplBlock>,
  typeName: string,
): string[] {
  const inherentType = new RegExp(
    `^(?:(?:(?:r#)?[A-Za-z_][A-Za-z0-9_]*)\\s*::\\s*)*(?:r#)?${typeName}(?:\\s+where\\b.*)?$`,
  );
  const inventory: string[] = [];
  for (
    const { header, body } of implBlocks
      .filter(({ header }) => inherentType.test(header))
  ) {
    for (
      const itemHeader of rustTopLevelItemHeaders(
        body,
        `Checkpoint 4 ${typeName} associated-item inventory`,
      )
    ) {
      const visibility =
        /\bpub(?:\s*\(\s*([^)]+?)\s*\))?/.exec(itemHeader);
      if (!visibility) continue;
      const normalizedVisibility = visibility[1] === undefined
        ? "pub"
        : `pub(${visibility[1].replace(/\s+/g, "")})`;
      const associatedItem =
        /\bfn\s+(?:r#)?([A-Za-z_][A-Za-z0-9_]*)\b/.exec(itemHeader)
        ?? /\bconst\s+(?:r#)?([A-Za-z_][A-Za-z0-9_]*)\b/.exec(
          itemHeader,
        )
        ?? /\btype\s+(?:r#)?([A-Za-z_][A-Za-z0-9_]*)\b/.exec(
          itemHeader,
        );
      if (associatedItem?.[1] === undefined) {
        throw new Error(
          `Checkpoint 4 ${typeName} inventory found an unsupported visible associated item`,
        );
      }
      const kind = /\bfn\s+(?:r#)?[A-Za-z_]/.test(itemHeader)
        ? "fn"
        : /\bconst\s+(?:r#)?[A-Za-z_]/.test(itemHeader)
        ? "const"
        : "type";
      inventory.push(
        `${normalizedVisibility} ${kind} ${associatedItem[1]}`,
      );
    }
  }
  return inventory;
}

function explicitTraitImplInventory(
  implBlocks: ReadonlyArray<RustImplBlock>,
  typeName: string,
): string[] {
  const wrapperReference = rustTypeReferenceRegex(typeName);
  return implBlocks
    .map(({ header }) => header)
    .filter((header) =>
      /\sfor\s/.test(header) && wrapperReference.test(header)
    );
}

function normalizedRustFunctionDeclaration(
  source: string,
  name: string,
): string {
  const searchable = maskRustLexicalNonCode(source);
  const declaration = new RegExp(
    `(^|\\n)[\\t ]*((?:pub(?:\\s*\\(\\s*(?:crate|super)\\s*\\))?\\s+)?(?:async\\s+)?fn\\s+${name}\\s*\\()`,
    "g",
  );
  const matches = [...searchable.matchAll(declaration)];
  if (
    matches.length !== 1
    || matches[0].index === undefined
    || matches[0][2] === undefined
  ) {
    throw new Error(
      `Expected one active Rust function declaration for ${name}, found ${matches.length}`,
    );
  }
  const start =
    matches[0].index + matches[0][0].lastIndexOf(matches[0][2]);
  const open = searchable.indexOf("{", start + matches[0][2].length);
  if (open < 0) throw new Error(`Missing Rust function body for ${name}`);
  return searchable
    .slice(start, open)
    .replace(/\s+/g, " ")
    .replace(/\(\s+/g, "(")
    .replace(/\s+\)/g, ")")
    .trim();
}

function normalizedRustUseDeclarations(
  source: string,
  target: string,
): string[] {
  const searchable = maskRustLexicalNonCode(source);
  const escapedTarget = target.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const declaration = new RegExp(
    `(?:#\\[[^\\]\\r\\n]+\\]\\s*)*(?:pub\\s*\\(\\s*crate\\s*\\)\\s+)?use\\s+${escapedTarget}::(\\{|\\*|[A-Za-z_][A-Za-z0-9_]*)`,
    "g",
  );
  return [...searchable.matchAll(declaration)].map((match) => {
    if (match.index === undefined) {
      throw new Error(`Missing Rust use declaration index for ${target}`);
    }
    const open = match[1] === "{"
      ? searchable.indexOf("{", match.index)
      : -1;
    const end =
      open >= 0 && open < match.index + match[0].length
        ? rustClosingBraceIndex(
          searchable,
          open,
          `${target} use declaration`,
        )
        : searchable.indexOf(";", match.index);
    const semicolon = searchable.indexOf(";", end);
    if (end < 0 || semicolon < 0) {
      throw new Error(`Unclosed Rust use declaration for ${target}`);
    }
    return searchable
      .slice(match.index, semicolon + 1)
      .replace(/\s+/g, " ")
      .replace(/\{\s+/g, "{")
      .replace(/\s+\}/g, "}")
      .trim();
  });
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

function rustNamedCallCount(source: string, name: string): number {
  const searchable = maskRustLexicalNonCode(source);
  const escapedName = name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  return [...searchable.matchAll(
    new RegExp(`\\b${escapedName}\\s*\\(`, "g"),
  )].filter((match) => {
    if (match.index === undefined) return false;
    return !/\bfn\s*$/.test(searchable.slice(0, match.index));
  }).length;
}

function rustIdentifierReferenceInventory(
  source: string,
  name: string,
): string[] {
  const searchable = maskRustLexicalNonCode(source);
  const escapedName = name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const useSpans = [...searchable.matchAll(/\buse\b/g)].flatMap(
    (match) => {
      if (match.index === undefined) return [];
      const end = searchable.indexOf(";", match.index);
      return end < 0 ? [] : [{ start: match.index, end }];
    },
  );
  return [...searchable.matchAll(
    new RegExp(`\\b${escapedName}\\b`, "g"),
  )].map((match) => {
    if (match.index === undefined) {
      throw new Error(`Missing Rust identifier index for ${name}`);
    }
    const prefix = searchable.slice(0, match.index);
    if (/\bfn\s*$/.test(prefix)) return "declaration";
    const suffix = searchable.slice(match.index + match[0].length);
    if (/^\s*\(/.test(suffix)) return "call";
    const useSpan = useSpans.find(
      ({ start, end }) => start < match.index! && match.index! < end,
    );
    if (useSpan) {
      const useSource = searchable.slice(useSpan.start, useSpan.end + 1);
      const withinUse = match.index - useSpan.start;
      const afterIdentifier = useSource.slice(
        withinUse + match[0].length,
      );
      if (/^\s+as\b/.test(afterIdentifier)) return "import-alias";
      const beforeUse = searchable.slice(
        Math.max(0, useSpan.start - 100),
        useSpan.start,
      );
      if (/\bpub(?:\s*\([^)]*\))?\s*$/.test(beforeUse)) {
        return "reexport";
      }
      return "import";
    }
    return "reference";
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
  it("records only the truthful retained 8A lifecycle states", () => {
    expect(retainedPreparationStates).toContainEqual({
      roadmapStatus: phase8Status,
      designStatus: designPhase8Status,
      lifecycle,
    });
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

  it("reconciles the exact active retained 8A identity accounting", () => {
    expect(
      retainedPreparationStates.map(({ lifecycle: state }) => state),
    ).toContain(lifecycle);
    const activeAdded = addedIdentities.filter(
      ({ checkpoint }) => checkpoint <= 5,
    );
    expect(activeAdded).toHaveLength(18);
    const activeCompanions = activeAdded.filter(({ currentId }) =>
      currentId.endsWith(
        "::telegram_item_kind_constant_matches_persisted_wire_value",
      ),
    );
    expect(activeCompanions).toHaveLength(1);
    expect(identityRows).toHaveLength(140);
    expect(identityRows.length + activeCompanions.length).toBe(141);
    expect(activeAdded.length - activeCompanions.length).toBe(17);
    expect(identityRows.length + activeAdded.length).toBe(158);
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
    const checkpoint5 = [
      ...checkpoint4,
      "src-tauri/src/telegram/runtime.rs",
    ];

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
      /(?:\b(?:get_client|get_authorized_client|get_authorized_runtime|TelegramApiHash|TelegramClientHandle|TelegramRuntime|AuthorizedTelegramRuntime|AccountClient|raw_client|raw_session|MemorySession|LoginToken|TelegramMessageIdentity|TelegramItemContext|SourceItemInsert|ExtractedItemPayload|ExtractedMediaPayload|ITEM_KIND_TELEGRAM_MESSAGE)\b|\.accounts\s*\.lock\s*\(\s*\)\s*\.await\b)/;
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
        checkpointThreeMediaOwnerPath,
      );
    const checkpoint3Source =
      `// Checkpoint 3 DTO split shifts retained source lines.\n${source}`;
    expect(() =>
      assertFacadeInventory(
        new Map([
          [typesPath, checkpoint3Source],
          [checkpointThreeMediaOwnerPath, mediaSource],
        ]),
        "8a-checkpoint-3",
      ),
    ).not.toThrow();
  });

  it("rejects legacy seams and duplicate Telegram owners across the complete Rust perimeter", () => {
    const overrides = new Map<string, string>([
      [
        "src-tauri/src/takeout_import/raw_parse.rs",
        `${telegramContractPaths.readTelegramContractFile(
          "src-tauri/src/takeout_import/raw_parse.rs",
        )}\nstruct ExtractedMediaPayload;\n`,
      ],
      [
        "src-tauri/src/sources/types.rs",
        `${telegramContractPaths.readTelegramContractFile(
          "src-tauri/src/sources/types.rs",
        )}\npub struct TelegramMessageDraft;\n`,
      ],
    ]);

    expect(() =>
      assertCheckpointThreeOwnershipContract(overrides),
    ).toThrow(/Checkpoint 3 ownership contract/);
  });

  it("rejects mutated persistence signatures, tuple shape, facade allowlist, and import directions", () => {
    const itemsPath = "src-tauri/src/sources/items.rs";
    const mediaPath = "src-tauri/src/telegram/media.rs";
    const telegramPath = "src-tauri/src/telegram.rs";
    const dtoPath = "src-tauri/src/telegram/dto.rs";
    const syncPath = "src-tauri/src/sources/sync.rs";
    const sources = new Map(
      [itemsPath, mediaPath, telegramPath, dtoPath, syncPath].map(
        (relativePath) => [
          relativePath,
          telegramContractPaths.readTelegramContractFile(relativePath),
        ],
      ),
    );
    const mutations = [
      new Map([
        [
          itemsPath,
          sources
            .get(itemsPath)!
            .replace(
              "draft: crate::telegram::TelegramMessageDraft,",
              "identity: crate::telegram::TelegramMessageIdentity,\n    draft: crate::telegram::TelegramMessageDraft,",
            ),
        ],
      ]),
      new Map([
        [
          mediaPath,
          sources
            .get(mediaPath)!
            .replace(
              "Option<(Option<String>, &'static str, Option<TelegramMediaPayload>)>",
              "Option<TelegramMediaPayload>",
            ),
        ],
      ]),
      new Map([
        [
          telegramPath,
          sources
            .get(telegramPath)!
            .replace(
              "TelegramMediaPayload,",
              "TelegramMediaPayload, TelegramLoginAttempt,",
            ),
        ],
      ]),
      new Map([
        [
          dtoPath,
          sources
            .get(dtoPath)!
            .replace(
              "use super::media::TelegramMediaPayload;",
              "use crate::telegram::media::TelegramMediaPayload;",
            ),
        ],
      ]),
      new Map([
        [
          syncPath,
          sources
            .get(syncPath)!
            .replace(
              "use crate::telegram::{",
              "use crate::telegram::dto::{",
            ),
        ],
      ]),
    ];

    for (const mutation of mutations) {
      expect(() =>
        assertCheckpointThreeApiContract(mutation),
      ).toThrow(/Checkpoint 3 API contract/);
    }
  });

  it("installs the actual Telegram media leaf in the Checkpoint 3 facade fixture", () => {
    expect(checkpointThreeMediaOwnerPath).toBe(
      "src-tauri/src/telegram/media.rs",
    );
    const mediaSource =
      telegramContractPaths.readTelegramContractFile(
        checkpointThreeMediaOwnerPath,
      );
    expect(mediaSource).toContain(
      "use grammers_client::{media::Media, tl};",
    );
  });

  it("removes legacy seams and keeps singular Telegram DTO/media owners", () => {
    assertCheckpointThreeOwnershipContract();
    const dto =
      telegramContractPaths.readTelegramContractFile(
        "src-tauri/src/telegram/dto.rs",
      );
    const draftBody = rustStructBody(dto, "TelegramMessageDraft");

    expect(
      [...draftBody.matchAll(/\bpub\s+([a-z_][a-z0-9_]*)\s*:/g)].map(
        (match) => match[1],
      ),
    ).toEqual([
      "telegram_identity",
      "telegram_context",
      "content",
      "content_kind",
      "author",
      "published_at",
      "raw_data",
      "item_kind",
      "media",
    ]);
  });

  it("pins exact Checkpoint 3 persistence, tuple, facade, and import surfaces", () => {
    assertCheckpointThreeApiContract();
  });

  it("isolates the opaque Telegram session codec from the app adapter", () => {
    assertCheckpointFourSessionContract();
  });

  it("keeps the plaintext Telegram credential row out of Debug", () => {
    assertAccountCredentialsRowHasNoSecretDebug();
  });

  it(
    "isolates the opaque Telegram runtime and its temporary raw consumer map",
    () => {
      assertCheckpointFiveRuntimeContract();
    },
    10_000,
  );

  const checkpointFiveRuntimePath =
    "src-tauri/src/telegram/runtime.rs";
  const checkpointFiveTelegramPath = "src-tauri/src/telegram.rs";
  const checkpointFiveStorePath = "src-tauri/src/sources/store.rs";
  const checkpointFiveSyncPath = "src-tauri/src/sources/sync.rs";
  const checkpointFiveRuntime =
    telegramContractPaths.readTelegramContractFile(
      checkpointFiveRuntimePath,
    );
  const checkpointFiveTelegram =
    telegramContractPaths.readTelegramContractFile(
      checkpointFiveTelegramPath,
    );
  const checkpointFiveStore =
    telegramContractPaths.readTelegramContractFile(checkpointFiveStorePath);
  const checkpointFiveSync =
    telegramContractPaths.readTelegramContractFile(checkpointFiveSyncPath);
  it.each([
    {
      name: "widened raw-client adapter",
      mutation: new Map([
        [
          checkpointFiveRuntimePath,
          checkpointFiveRuntime.replace(
            "pub(crate) fn raw_client(",
            "pub fn raw_client(",
          ),
        ],
      ]),
    },
    {
      name: "post-test API-hash plaintext getter",
      mutation: new Map([
        [
          checkpointFiveRuntimePath,
          `${checkpointFiveRuntime}
impl TelegramApiHash {
    pub fn as_str(&self) -> &str {
        self.0.expose_secret()
    }
}
`,
        ],
      ]),
    },
    {
      name: "extra crate-visible runtime free function",
      mutation: new Map([
        [
          checkpointFiveRuntimePath,
          `${checkpointFiveRuntime}
pub(crate) fn extra_runtime_seam() {}
`,
        ],
      ]),
    },
    {
      name: "legacy authorized runtime type in runtime owner",
      mutation: new Map([
        [
          checkpointFiveRuntimePath,
          `${checkpointFiveRuntime}
struct AuthorizedTelegramRuntime;
`,
        ],
      ]),
    },
    {
      name: "public API-hash field",
      mutation: new Map([
        [
          checkpointFiveRuntimePath,
          checkpointFiveRuntime.replace(
            "pub struct TelegramApiHash(SecretString);",
            "pub struct TelegramApiHash(pub SecretString);",
          ),
        ],
      ]),
    },
    {
      name: "API hash cfg_attr derives Debug",
      mutation: new Map([
        [
          checkpointFiveRuntimePath,
          checkpointFiveRuntime.replace(
            "#[derive(Clone)]\npub struct TelegramApiHash(SecretString);",
            "#[cfg_attr(all(), derive(Debug))]\n#[derive(Clone)]\npub struct TelegramApiHash(SecretString);",
          ),
        ],
      ]),
    },
    {
      name: "runtime macro exposes the opaque API hash",
      mutation: new Map([
        [
          checkpointFiveRuntimePath,
          `${checkpointFiveRuntime}
macro_rules! expose_runtime_wrapper {
    ($target:ty) => {
        impl secrecy::ExposeSecret<str> for $target {
            fn expose_secret(&self) -> &str {
                secrecy::ExposeSecret::expose_secret(&self.0)
            }
        }
    };
}

expose_runtime_wrapper!(TelegramApiHash);
`,
        ],
      ]),
    },
    {
      name: "runtime nested macro exposes the opaque API hash",
      mutation: new Map([
        [
          checkpointFiveRuntimePath,
          `${checkpointFiveRuntime}
macro_rules! expose_runtime_wrapper {
    ($target:ty) => {
        impl secrecy::ExposeSecret<str> for $target {
            fn expose_secret(&self) -> &str {
                secrecy::ExposeSecret::expose_secret(&self.0)
            }
        }
    };
}

macro_rules! forward_exposure {
    () => {
        expose_runtime_wrapper!(TelegramApiHash);
    };
}

forward_exposure!();
`,
        ],
      ]),
    },
    {
      name: "runtime Unicode macro exposes the opaque API hash",
      mutation: new Map([
        [
          checkpointFiveRuntimePath,
          `${checkpointFiveRuntime}
macro_rules! раскрыть_обертку {
    ($target:ty) => {
        impl secrecy::ExposeSecret<str> for $target {
            fn expose_secret(&self) -> &str {
                secrecy::ExposeSecret::expose_secret(&self.0)
            }
        }
    };
}

раскрыть_обертку!(TelegramApiHash);
`,
        ],
      ]),
    },
    {
      name: "runtime macro adds credential-row Debug",
      mutation: new Map([
        [
          checkpointFiveRuntimePath,
          `${checkpointFiveRuntime}
macro_rules! add_debug {
    ($target:ty) => {
        impl std::fmt::Debug for $target {
            fn fmt(
                &self,
                formatter: &mut std::fmt::Formatter<'_>,
            ) -> std::fmt::Result {
                formatter
                    .debug_struct("AccountCredentialsRow")
                    .field("api_hash", &self.api_hash)
                    .finish()
            }
        }
    };
}

add_debug!(super::AccountCredentialsRow);
`,
        ],
      ]),
    },
    {
      name: "runtime macro generates runtime wrapper alias",
      mutation: new Map([
        [
          checkpointFiveRuntimePath,
          `${checkpointFiveRuntime}
macro_rules! make_alias {
    ($name:ident) => {
        type $name = TelegramApiHash;
    };
}

make_alias!(HiddenHash);
`,
        ],
      ]),
    },
    {
      name: "sync Unicode type alias exposes the opaque API hash",
      mutation: new Map([
        [
          checkpointFiveSyncPath,
          `${checkpointFiveSync}
type Хэш = crate::telegram::TelegramApiHash;

impl secrecy::ExposeSecret<str> for Хэш {
    fn expose_secret(&self) -> &str {
        panic!("Unicode type alias exposure escape")
    }
}
`,
        ],
      ]),
    },
    {
      name: "sync Unicode use alias exposes the opaque API hash",
      mutation: new Map([
        [
          checkpointFiveSyncPath,
          `${checkpointFiveSync}
use crate::{telegram::{TelegramApiHash as Хэш}};

impl secrecy::ExposeSecret<str> for Хэш {
    fn expose_secret(&self) -> &str {
        panic!("Unicode use alias exposure escape")
    }
}
`,
        ],
      ]),
    },
    {
      name: "runtime Unicode type alias adds credential-row Debug",
      mutation: new Map([
        [
          checkpointFiveRuntimePath,
          `${checkpointFiveRuntime}
type Строка = super::AccountCredentialsRow;

impl std::fmt::Debug for Строка {
    fn fmt(&self, _formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        panic!("Unicode type alias Debug escape")
    }
}
`,
        ],
      ]),
    },
    {
      name: "runtime Unicode use alias adds credential-row Debug",
      mutation: new Map([
        [
          checkpointFiveRuntimePath,
          `${checkpointFiveRuntime}
use super::{AccountCredentialsRow as Строка};

impl std::fmt::Debug for Строка {
    fn fmt(&self, _formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        panic!("Unicode use alias Debug escape")
    }
}
`,
        ],
      ]),
    },
    {
      name: "plaintext credential row derives Debug",
      mutation: new Map([
        [
          checkpointFiveTelegramPath,
          checkpointFiveTelegram.replace(
            "#[derive(sqlx::FromRow)]\nstruct AccountCredentialsRow",
            "#[derive(\n    sqlx::FromRow,\n    Debug,\n)]\nstruct AccountCredentialsRow",
          ),
        ],
      ]),
    },
    {
      name: "plaintext credential row cfg_attr derives Debug",
      mutation: new Map([
        [
          checkpointFiveTelegramPath,
          checkpointFiveTelegram.replace(
            "#[derive(sqlx::FromRow)]\nstruct AccountCredentialsRow",
            "#[derive(sqlx::FromRow)]\n#[cfg_attr(all(), derive(Debug))]\nstruct AccountCredentialsRow",
          ),
        ],
      ]),
    },
    {
      name: "plaintext credential row implements Debug manually",
      mutation: new Map([
        [
          checkpointFiveTelegramPath,
          `${checkpointFiveTelegram}
impl std::fmt::Debug for AccountCredentialsRow {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("AccountCredentialsRow")
            .field("api_hash", &self.api_hash)
            .finish()
    }
}
`,
        ],
      ]),
    },
    {
      name: "plaintext credential row implements aliased Debug",
      mutation: new Map([
        [
          checkpointFiveTelegramPath,
          `${checkpointFiveTelegram}
use std::fmt::Debug as SensitiveFormatter;

impl SensitiveFormatter for AccountCredentialsRow {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("AccountCredentialsRow")
            .field("api_hash", &self.api_hash)
            .finish()
    }
}
`,
        ],
      ]),
    },
    {
      name: "plaintext credential row implements aliased Debug in child runtime module",
      mutation: new Map([
        [
          checkpointFiveRuntimePath,
          `${checkpointFiveRuntime}
use std::fmt::Debug as SensitiveFormatter;

impl SensitiveFormatter for super::AccountCredentialsRow {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("AccountCredentialsRow")
            .field("api_hash", &self.api_hash)
            .finish()
    }
}
`,
        ],
      ]),
    },
    {
      name: "store caller-held account lock",
      mutation: new Map([
        [
          checkpointFiveStorePath,
          checkpointFiveStore.replace(
            "let client_handle = crate::telegram::get_client(state.inner(), account_id).await?;",
            "let _accounts = state.accounts.lock().await;\n    let client_handle = crate::telegram::get_client(state.inner(), account_id).await?;",
          ),
        ],
      ]),
    },
    {
      name: "sync raw-session escape",
      mutation: new Map([
        [
          checkpointFiveSyncPath,
          checkpointFiveSync.replace(
            "let client = client_handle.raw_client().clone();",
            "let client = client_handle.raw_client().clone();\n    let _session = client_handle.raw_session();",
          ),
        ],
      ]),
    },
    {
      name: "sync UFCS raw-session escape",
      mutation: new Map([
        [
          checkpointFiveSyncPath,
          checkpointFiveSync.replace(
            "let client = client_handle.raw_client().clone();",
            "let client = client_handle.raw_client().clone();\n    let _session = crate::telegram::TelegramClientHandle::raw_session(&client_handle);",
          ),
        ],
      ]),
    },
    {
      name: "sync function-value raw-client escape",
      mutation: new Map([
        [
          checkpointFiveSyncPath,
          checkpointFiveSync.replace(
            "let client = client_handle.raw_client().clone();",
            "let client = client_handle.raw_client().clone();\n    let raw = crate::telegram::TelegramClientHandle::raw_client;\n    let _duplicate_client = raw(&client_handle);",
          ),
        ],
      ]),
    },
    {
      name: "sync aliases and exposes the opaque API hash",
      mutation: new Map([
        [
          checkpointFiveSyncPath,
          `${checkpointFiveSync}
type HashAlias = crate::telegram::TelegramApiHash;

impl secrecy::ExposeSecret<str> for HashAlias {
    fn expose_secret(&self) -> &str {
        panic!("secret exposure escape")
    }
}
`,
        ],
      ]),
    },
    {
      name: "duplicate sync authorized lookup",
      mutation: new Map([
        [
          checkpointFiveSyncPath,
          checkpointFiveSync.replace(
            "let client_handle = crate::telegram::get_authorized_client(state.inner(), account_id).await?;",
            "let _ignored_handle = crate::telegram::get_authorized_client(state.inner(), account_id).await?;\n    let client_handle = crate::telegram::get_authorized_client(state.inner(), account_id).await?;",
          ),
        ],
      ]),
    },
    {
      name: "sync authorized lookup through function-value alias",
      mutation: new Map([
        [
          checkpointFiveSyncPath,
          checkpointFiveSync.replace(
            "let client_handle = crate::telegram::get_authorized_client(state.inner(), account_id).await?;",
            "let lookup = crate::telegram::get_authorized_client;\n    let _ignored_handle = lookup(state.inner(), account_id).await?;\n    let client_handle = crate::telegram::get_authorized_client(state.inner(), account_id).await?;",
          ),
        ],
      ]),
    },
  ])("rejects Checkpoint 5 runtime mutation: $name", ({ mutation }) => {
    for (const [relativePath, source] of mutation) {
      expect(
        source,
        `Checkpoint 5 mutation changes ${relativePath}`,
      ).not.toBe(
        telegramContractPaths.readTelegramContractFile(relativePath),
      );
    }
    expect(() =>
      assertCheckpointFiveRuntimeContract(mutation)
    ).toThrow(/Checkpoint 5/);
  });

  const checkpointFourSessionPath = "src-tauri/src/telegram/session.rs";
  const checkpointFourAdapterPath =
    "src-tauri/src/telegram_session_store.rs";
  const checkpointFourSession =
    telegramContractPaths.readTelegramContractFile(
      checkpointFourSessionPath,
    );
  const checkpointFourAdapter =
    telegramContractPaths.readTelegramContractFile(
      checkpointFourAdapterPath,
    );
  it.each([
    {
      name: "widened raw-session accessor",
      mutation: new Map([
        [
          checkpointFourSessionPath,
          checkpointFourSession.replace(
            "pub(super) fn raw_memory_session(",
            "pub(crate) fn raw_memory_session(",
          ),
        ],
      ]),
    },
    {
      name: "session key loses Arc",
      mutation: new Map([
        [
          checkpointFourSessionPath,
          checkpointFourSession.replace(
            "pub struct SessionEncryptionKey(Arc<SecretVec<u8>>);",
            "pub struct SessionEncryptionKey(SecretVec<u8>);",
          ),
        ],
      ]),
    },
    {
      name: "public as_bytes secret getter",
      mutation: new Map([
        [
          checkpointFourSessionPath,
          `${checkpointFourSession}\nimpl SessionEncryptionKey { pub fn as_bytes(&self) -> &[u8] { &[] } }\n`,
        ],
      ]),
    },
    {
      name: "codec owns SecretStore",
      mutation: new Map([
        [
          checkpointFourSessionPath,
          `${checkpointFourSession}\nuse crate::secret_store::SecretStoreState;\n`,
        ],
      ]),
    },
    {
      name: "adapter duplicates encrypted envelope",
      mutation: new Map([
        [
          checkpointFourAdapterPath,
          `${checkpointFourAdapter}\nstruct EncryptedSessionEnvelope;\n`,
        ],
      ]),
    },
    {
      name: "legacy key creation precedes decode",
      mutation: new Map([
        [
          checkpointFourAdapterPath,
          checkpointFourAdapter.replace(
            "let session = decode_session_json(&json, account_id, None)?;\n    let key = ensure_session_key(secret_store, account_id).await?;",
            "let key = ensure_session_key(secret_store, account_id).await?;\n    let session = decode_session_json(&json, account_id, None)?;",
          ),
        ],
      ]),
    },
  ])(
    "rejects Checkpoint 4 standing mutation: $name",
    ({ mutation }) => {
      expect(
        [...mutation.entries()].some(([relativePath, source]) =>
          source
          !== (
            relativePath === checkpointFourSessionPath
              ? checkpointFourSession
              : checkpointFourAdapter
          )
        ),
        "Checkpoint 4 mutation changes its source fixture",
      ).toBe(true);
      expect(() =>
        assertCheckpointFourSessionContract(mutation),
      ).toThrow(/Checkpoint 4/);
    },
  );

  it.each([
    {
      name: "alternate public raw-session method",
      addition:
        "impl TelegramSession { pub fn memory_session(&self) -> &Arc<MemorySession> { &self.inner } }",
    },
    {
      name: "alternate public secret-material method",
      addition:
        "impl SessionEncryptionKey { pub fn material(&self) -> &[u8] { &[] } }",
    },
    {
      name: "generic ExposeSecret implementation",
      addition:
        "impl secrecy::ExposeSecret<Vec<u8>> for SessionEncryptionKey { fn expose_secret(&self) -> &Vec<u8> { unimplemented!() } }",
    },
    {
      name: "AsRef implementation",
      addition:
        "impl AsRef<[u8]> for SessionEncryptionKey { fn as_ref(&self) -> &[u8] { &[] } }",
    },
    {
      name: "Into conversion implementation",
      addition:
        "impl Into<Vec<u8>> for SessionEncryptionKey { fn into(self) -> Vec<u8> { Vec::new() } }",
    },
    {
      name: "nested production-module raw-session method",
      addition:
        "mod nested_session_escape { use super::*; impl TelegramSession { pub fn memory_session(&self) -> &Arc<MemorySession> { &self.inner } } }",
    },
    {
      name: "TelegramSession source conversion",
      addition:
        "impl From<TelegramSession> for Arc<MemorySession> { fn from(session: TelegramSession) -> Self { session.inner } }",
    },
    {
      name: "SessionEncryptionKey source conversion",
      addition:
        "impl From<SessionEncryptionKey> for Vec<u8> { fn from(_key: SessionEncryptionKey) -> Self { Vec::new() } }",
    },
  ])(
    "rejects Checkpoint 4 alternate wrapper exposure mutation: $name",
    ({ addition }) => {
      const mutatedSession = checkpointFourSession.replace(
        "\n#[cfg(test)]\nmod tests",
        `\n${addition}\n\n#[cfg(test)]\nmod tests`,
      );
      expect(
        mutatedSession,
        "Checkpoint 4 alternate exposure mutation changes its source fixture",
      ).not.toBe(checkpointFourSession);
      expect(() =>
        assertCheckpointFourSessionContract(
          new Map([
            [checkpointFourSessionPath, mutatedSession],
          ]),
        )
      ).toThrow(/Checkpoint 4/);
    },
  );

  it("rejects Checkpoint 4 cfg-order production accessor after the test module", () => {
    const mutatedSession =
      `${checkpointFourSession}\nimpl TelegramSession { pub fn memory_session(&self) -> &Arc<MemorySession> { &self.inner } }\n`;
    expect(
      mutatedSession,
      "Checkpoint 4 after-test production mutation changes its source fixture",
    ).not.toBe(checkpointFourSession);
    expect(() =>
      assertCheckpointFourSessionContract(
        new Map([
          [checkpointFourSessionPath, mutatedSession],
        ]),
      )
    ).toThrow(/Checkpoint 4/);
  });

  it("rejects Checkpoint 4 adapter cfg-order production raw signature after the test module", () => {
    const mutatedAdapter =
      `${checkpointFourAdapter}\nfn adapter_raw_session_escape(_: &grammers_session::storages::MemorySession) {}\n`;
    expect(
      mutatedAdapter,
      "Checkpoint 4 after-test adapter mutation changes its source fixture",
    ).not.toBe(checkpointFourAdapter);
    expect(() =>
      assertCheckpointFourSessionContract(
        new Map([
          [checkpointFourAdapterPath, mutatedAdapter],
        ]),
      )
    ).toThrow(/Checkpoint 4/);
  });

  it("excludes Checkpoint 4 cfg-order standalone test-only impl methods", () => {
    const mutatedSession = checkpointFourSession.replace(
      "\n#[cfg(test)]\nmod tests",
      "\n#[cfg(test)]\nimpl TelegramSession { pub fn test_only_probe(&self) {} }\n\n#[cfg(test)]\nmod tests",
    );
    expect(
      mutatedSession,
      "Checkpoint 4 standalone cfg(test) fixture changes its source",
    ).not.toBe(checkpointFourSession);
    expect(() =>
      assertCheckpointFourSessionContract(
        new Map([
          [checkpointFourSessionPath, mutatedSession],
        ]),
      )
    ).not.toThrow();
  });

  it("keeps nested cfg(test) attributes in their production container", () => {
    const source = String.raw`
      enum RuntimeHandle {
        Production,
        #[cfg(test)]
        Test,
      }

      #[cfg(test)]
      fn top_level_test_helper() {}
    `;
    const masked = maskExactCfgTestItemSpans(source);
    expect(masked).toContain("#[cfg(test)]\n        Test");
    expect(masked).not.toContain("top_level_test_helper");
    expect(masked).toContain("enum RuntimeHandle");
  });

  it("excludes test-only items inside inline production modules from visible inventory", () => {
    const source = String.raw`
      mod nested {
        #[cfg(test)]
        pub fn test_only_visible_helper() {}

        pub struct ProductionType;
      }
    `;
    expect(
      visibleRustTopLevelItemInventory(
        source,
        "Checkpoint 5 nested cfg(test) parser fixture",
      ),
    ).toEqual(["nested|pub struct ProductionType"]);
  });

  it("classifies attributed visible const functions as functions", () => {
    const source = String.raw`
      #[allow(dead_code)]
      pub const fn frozen_value() -> usize {
        7
      }
    `;
    expect(
      visibleRustTopLevelItemInventory(
        source,
        "Checkpoint 5 visible const-function parser fixture",
      ),
    ).toEqual(["root|pub fn frozen_value"]);
  });

  it("characterizes the Checkpoint 5 macro tokenizer positive forms", () => {
    const fixtures = [
      "обертка!(TelegramApiHash);",
      "r#раскрыть!(r#TelegramApiHash);",
      "crate /* path */ :: macros :: r#expose /* bang */ ! /* body */ [TelegramClientHandle];",
      "$crate::expose!{TelegramLoginAttempt};",
      "outer!(inner!((TelegramRuntime)));",
      "round!((TelegramApiHash));",
      "square![[TelegramClientHandle]];",
      "curly!{{TelegramRuntime}};",
      // Synthetic over-approximation tripwire; this is not compile-valid Rust.
      "\u{1F980}!(TelegramApiHash);",
    ];

    for (const source of fixtures) {
      const inventory = rustSensitiveMacroInventory(
        source,
        [
          "TelegramApiHash",
          "TelegramClientHandle",
          "TelegramLoginAttempt",
          "TelegramRuntime",
        ],
        "Checkpoint 5 macro tokenizer positive fixture",
      );
      expect(inventory, source).not.toEqual([]);
      expect(
        inventory.every((evidence) =>
          /^bang@\d+\/\d+:\d+\|Telegram(?:ApiHash|ClientHandle|LoginAttempt|Runtime)$/
            .test(evidence)
        ),
        source,
      ).toBe(true);
    }
  });

  it("characterizes the Checkpoint 5 macro tokenizer negative forms", () => {
    const source = String.raw`
      macro_rules! TelegramApiHash { () => {}; }
      let _ = left != (TelegramApiHash);
      let _ = !(TelegramApiHash);
      if !(TelegramApiHash) {}
      return !(TelegramApiHash);
      #![allow(TelegramApiHash)]
      fn never_returns() -> ! { loop {} }
      'label: loop { break 'label !(uses::<TelegramApiHash>()); }
      takes_lifetime!('TelegramApiHash);
      let _ = "fake!(TelegramApiHash)";
      let _ = r#"fake!(TelegramClientHandle)"#;
      let _ = '!';
      // fake!(TelegramLoginAttempt)
      /* fake!(TelegramRuntime) */
      обертка!(UnrelatedType);
    `;
    expect(
      rustSensitiveMacroInventory(
        source,
        [
          "TelegramApiHash",
          "TelegramClientHandle",
          "TelegramLoginAttempt",
          "TelegramRuntime",
        ],
        "Checkpoint 5 macro tokenizer negative fixture",
      ),
    ).toEqual([]);
  });

  it("fails closed on mismatched or unclosed Checkpoint 5 macro tokenizer delimiters", () => {
    for (
      const source of [
        "fn broken(]",
        "audit!(TelegramApiHash]",
        "audit!{TelegramApiHash",
      ]
    ) {
      expect(() =>
        rustSensitiveMacroInventory(
          source,
          ["TelegramApiHash"],
          "Checkpoint 5 macro tokenizer delimiter fixture",
        )
      ).toThrow(/Checkpoint 5 macro tokenizer delimiter fixture/);
    }
  });

  it("characterizes the Checkpoint 5 alias tokenizer positive forms", () => {
    const source = String.raw`
      type Хэш = crate::telegram::TelegramApiHash;
      type r#Строка = super::AccountCredentialsRow;
      use crate::telegram::TelegramApiHash as DirectAlias;
      use crate::{telegram::{TelegramApiHash as ХэшИзUse}};
      use super::{AccountCredentialsRow as r#СтрокаИзUse};
      use crate::telegram::TelegramApiHash::{self as Сам};
      use crate /* path */ :: telegram :: {
        TelegramApiHash /* alias */ as Коммент,
      };
    `;
    expect(
      rustWrapperAliasNames(source, [
        "TelegramApiHash",
        "AccountCredentialsRow",
      ]),
    ).toEqual([
      "Хэш",
      "Строка",
      "DirectAlias",
      "ХэшИзUse",
      "СтрокаИзUse",
      "Сам",
      "Коммент",
    ]);
  });

  it("characterizes the Checkpoint 5 alias tokenizer negative forms", () => {
    const source = String.raw`
      type TelegramApiHash = OtherType;
      type UnicodeNeighbor = crate::telegram::TelegramApiHashХ;
      type PrefixNeighbor = crate::telegram::СтрокаAccountCredentialsRow;
      type Lifetime<'TelegramApiHash> = &'TelegramApiHash OtherType;
      use crate::telegram::TelegramApiHash;
      use crate::telegram::*;
      use OtherType as TelegramApiHash;
      use crate::telegram::TelegramApiHashХ as UnicodeNeighborUse;
      use crate::telegram::СтрокаAccountCredentialsRow as PrefixNeighborUse;
      use crate::{telegram::{OtherType as TelegramApiHash}};
      use crate::telegram::OtherType as OtherAlias;
      // type Comment = TelegramApiHash;
      /* use AccountCredentialsRow as CommentAlias; */
      const TEXT: &str = "type StringAlias = TelegramApiHash;";
      takes_lifetime!('AccountCredentialsRow);
    `;
    expect(
      rustWrapperAliasNames(source, [
        "TelegramApiHash",
        "AccountCredentialsRow",
      ]),
    ).toEqual([]);
  });

  it("characterizes sensitive struct outer-attribute masking and nested brackets", () => {
    const source = String.raw`
      // #[derive(Debug)] pub struct TelegramApiHash;
      const TEXT: &str =
        "#[derive(Debug)] pub struct TelegramClientHandle;";
      #[derive(Clone)]
      /* [masked comment brackets] */
      #[audit[nested[deep]]]
      pub struct TelegramApiHash(SecretString);

      pub struct TelegramRuntime {
        marker: [u8; 1],
      }
    `;
    expect(
      rustStructOuterAttributeInventory(
        source,
        [
          "TelegramApiHash",
          "TelegramClientHandle",
          "TelegramRuntime",
        ],
        "Checkpoint 5 outer-attribute characterization",
      ),
    ).toEqual([
      "TelegramApiHash|#[derive(Clone)]|#[audit[nested[deep]]]",
      "TelegramRuntime|",
    ]);
  });

  it("excludes Checkpoint 4 cfg-order array-signature test-only items", () => {
    const mutatedSession = checkpointFourSession.replace(
      "\n#[cfg(test)]\nmod tests",
      "\n#[cfg(test)]\nfn cfg_array_probe() -> [u8; 32] {\n    impl TelegramSession { pub fn test_only_array_probe(&self) {} }\n    [0; 32]\n}\n\n#[cfg(test)]\nmod tests",
    );
    expect(
      mutatedSession,
      "Checkpoint 4 cfg(test) array-signature fixture changes its source",
    ).not.toBe(checkpointFourSession);
    expect(() =>
      assertCheckpointFourSessionContract(
        new Map([
          [checkpointFourSessionPath, mutatedSession],
        ]),
      )
    ).not.toThrow();
  });

  it("excludes Checkpoint 4 cfg-order braced-const-generic test-only items", () => {
    const mutatedSession = checkpointFourSession.replace(
      "\n#[cfg(test)]\nmod tests",
      "\n#[cfg(test)]\nfn cfg_const_probe<const N: usize>() -> Foo<{ N }> {\n    impl TelegramSession { pub fn test_only_const_probe(&self) {} }\n    unimplemented!()\n}\n\n#[cfg(test)]\nmod tests",
    );
    expect(
      mutatedSession,
      "Checkpoint 4 cfg(test) const-generic fixture changes its source",
    ).not.toBe(checkpointFourSession);
    expect(() =>
      assertCheckpointFourSessionContract(
        new Map([
          [checkpointFourSessionPath, mutatedSession],
        ]),
      )
    ).not.toThrow();
  });

  it("rejects Checkpoint 4 cfg-order production impl after a test-only comparison", () => {
    const mutatedSession = checkpointFourSession.replace(
      "\n#[cfg(test)]\nmod tests",
      "\n#[cfg(test)]\nconst TEST_ONLY: bool = 1 < 2;\nimpl TelegramSession { pub fn memory_session(&self) -> &Arc<MemorySession> { &self.inner } }\nfn angle_sentinel() -> usize { 0 }\n\n#[cfg(test)]\nmod tests",
    );
    expect(
      mutatedSession,
      "Checkpoint 4 cfg(test) comparison fixture changes its source",
    ).not.toBe(checkpointFourSession);
    expect(() =>
      assertCheckpointFourSessionContract(
        new Map([
          [checkpointFourSessionPath, mutatedSession],
        ]),
      )
    ).toThrow(/Checkpoint 4/);
  });

  it("rejects Checkpoint 4 production inherent impl with a braced const header", () => {
    const mutatedSession = checkpointFourSession.replace(
      "\n#[cfg(test)]\nmod tests",
      "\nimpl TelegramSession where [(); { 1 }]: Sized {\n    pub fn memory_session(&self) -> &Arc<MemorySession> { &self.inner }\n}\n\n#[cfg(test)]\nmod tests",
    );
    expect(
      mutatedSession,
      "Checkpoint 4 inherent braced-const fixture changes its source",
    ).not.toBe(checkpointFourSession);
    expect(() =>
      assertCheckpointFourSessionContract(
        new Map([
          [checkpointFourSessionPath, mutatedSession],
        ]),
      )
    ).toThrow(/Checkpoint 4/);
  });

  it("rejects Checkpoint 4 production trait impl with a braced const header", () => {
    const mutatedSession = checkpointFourSession.replace(
      "\n#[cfg(test)]\nmod tests",
      "\nimpl Into<[u8; { SESSION_KEY_BYTES }]> for SessionEncryptionKey {\n    fn into(self) -> [u8; SESSION_KEY_BYTES] { [0; SESSION_KEY_BYTES] }\n}\n\n#[cfg(test)]\nmod tests",
    );
    expect(
      mutatedSession,
      "Checkpoint 4 trait braced-const fixture changes its source",
    ).not.toBe(checkpointFourSession);
    expect(() =>
      assertCheckpointFourSessionContract(
        new Map([
          [checkpointFourSessionPath, mutatedSession],
        ]),
      )
    ).toThrow(/Checkpoint 4/);
  });

  it("rejects Checkpoint 4 production generic visible methods", () => {
    const mutatedSession = checkpointFourSession.replace(
      "\n#[cfg(test)]\nmod tests",
      "\nimpl TelegramSession {\n    pub fn memory_session<T>(&self) -> &Arc<MemorySession> { &self.inner }\n}\n\n#[cfg(test)]\nmod tests",
    );
    expect(
      mutatedSession,
      "Checkpoint 4 generic visible method fixture changes its source",
    ).not.toBe(checkpointFourSession);
    expect(() =>
      assertCheckpointFourSessionContract(
        new Map([
          [checkpointFourSessionPath, mutatedSession],
        ]),
      )
    ).toThrow(/Checkpoint 4/);
  });

  it("rejects Checkpoint 4 production visible associated raw-session consts", () => {
    const mutatedSession = checkpointFourSession.replace(
      "impl TelegramSession {\n    pub fn empty",
      "impl TelegramSession {\n    pub const MEMORY_SESSION: for<'a> fn(&'a Self) -> &'a Arc<MemorySession> = Self::raw_memory_session;\n\n    pub fn empty",
    );
    expect(
      mutatedSession,
      "Checkpoint 4 visible associated const fixture changes its source",
    ).not.toBe(checkpointFourSession);
    expect(() =>
      assertCheckpointFourSessionContract(
        new Map([
          [checkpointFourSessionPath, mutatedSession],
        ]),
      )
    ).toThrow(/Checkpoint 4/);
  });

  it("rejects Checkpoint 4 production raw-identifier wrapper impls", () => {
    const mutatedSession = checkpointFourSession.replace(
      "\n#[cfg(test)]\nmod tests",
      "\nimpl r#TelegramSession {\n    pub fn memory_session(&self) -> &Arc<MemorySession> { &self.inner }\n}\n\n#[cfg(test)]\nmod tests",
    );
    expect(
      mutatedSession,
      "Checkpoint 4 raw-identifier wrapper fixture changes its source",
    ).not.toBe(checkpointFourSession);
    expect(() =>
      assertCheckpointFourSessionContract(
        new Map([
          [checkpointFourSessionPath, mutatedSession],
        ]),
      )
    ).toThrow(/Checkpoint 4/);
  });

  it("rejects Checkpoint 4 production cross-module wrapper impls", () => {
    const telegramPath = "src-tauri/src/telegram.rs";
    const telegram =
      telegramContractPaths.readTelegramContractFile(telegramPath);
    const mutatedTelegram = telegram.replace(
      "\n#[cfg(test)]\nmod tests",
      "\nimpl TelegramSession {\n    pub fn memory_session(&self) {}\n}\n\n#[cfg(test)]\nmod tests",
    );
    expect(
      mutatedTelegram,
      "Checkpoint 4 cross-module wrapper fixture changes its source",
    ).not.toBe(telegram);
    expect(() =>
      assertCheckpointFourSessionContract(
        new Map([
          [telegramPath, mutatedTelegram],
        ]),
      )
    ).toThrow(/Checkpoint 4/);
  });

  it("rejects Checkpoint 4 production visible free raw-session unwraps", () => {
    const telegramPath = "src-tauri/src/telegram.rs";
    const telegram =
      telegramContractPaths.readTelegramContractFile(telegramPath);
    const mutatedTelegram = telegram.replace(
      "\n#[cfg(test)]\nmod tests",
      "\npub(crate) fn raw_session_escape(session: &TelegramSession) -> &Arc<MemorySession> { session.raw_memory_session() }\n\n#[cfg(test)]\nmod tests",
    );
    expect(
      mutatedTelegram,
      "Checkpoint 4 visible free raw-session fixture changes its source",
    ).not.toBe(telegram);
    expect(() =>
      assertCheckpointFourSessionContract(
        new Map([
          [telegramPath, mutatedTelegram],
        ]),
      )
    ).toThrow(/Checkpoint 4/);
  });

  it.each([
    [
      "use alias",
      "use crate::telegram::session::TelegramSession as Alias;",
    ],
    [
      "type alias",
      "type Alias = TelegramSession;",
    ],
  ])("rejects Checkpoint 4 production %s wrapper impls", (_, alias) => {
    const telegramPath = "src-tauri/src/telegram.rs";
    const telegram =
      telegramContractPaths.readTelegramContractFile(telegramPath);
    const mutatedTelegram = telegram.replace(
      "\n#[cfg(test)]\nmod tests",
      `\n${alias}\nimpl Alias {\n    pub fn memory_session(&self) {}\n}\n\n#[cfg(test)]\nmod tests`,
    );
    expect(
      mutatedTelegram,
      "Checkpoint 4 aliased wrapper fixture changes its source",
    ).not.toBe(telegram);
    expect(() =>
      assertCheckpointFourSessionContract(
        new Map([
          [telegramPath, mutatedTelegram],
        ]),
      )
    ).toThrow(/Checkpoint 4/);
  });

  it("rejects Checkpoint 4 production chained wrapper aliases", () => {
    const telegramPath = "src-tauri/src/telegram.rs";
    const telegram =
      telegramContractPaths.readTelegramContractFile(telegramPath);
    const mutatedTelegram = telegram.replace(
      "\n#[cfg(test)]\nmod tests",
      "\ntype WrapperAlias = TelegramSession;\ntype ChainedAlias = WrapperAlias;\nimpl ChainedAlias {\n    pub fn memory_session(&self) {}\n}\n\n#[cfg(test)]\nmod tests",
    );
    expect(
      mutatedTelegram,
      "Checkpoint 4 chained wrapper alias fixture changes its source",
    ).not.toBe(telegram);
    expect(() =>
      assertCheckpointFourSessionContract(
        new Map([
          [telegramPath, mutatedTelegram],
        ]),
      )
    ).toThrow(/Checkpoint 4/);
  });

  it("rejects Checkpoint 4 nested cfg(test) enum variant masking", () => {
    const mutatedSession = checkpointFourSession.replace(
      "\n#[cfg(test)]\nmod tests",
      "\nenum CfgVariantProbe {\n    #[cfg(test)]\n    TestOnly { marker: bool },\n    Production,\n}\nimpl TelegramSession { pub fn memory_session(&self) -> &Arc<MemorySession> { &self.inner } }\n\n#[cfg(test)]\nmod tests",
    );
    expect(
      mutatedSession,
      "Checkpoint 4 nested cfg(test) variant fixture changes its source",
    ).not.toBe(checkpointFourSession);
    expect(() =>
      assertCheckpointFourSessionContract(
        new Map([
          [checkpointFourSessionPath, mutatedSession],
        ]),
      )
    ).toThrow(/Checkpoint 4/);
  });

  it("rejects moving a facade reference while preserving aggregate counts", () => {
    const typesPath = "src-tauri/src/sources/types.rs";
    const source =
      telegramContractPaths.readTelegramContractFile(typesPath);
    const relocated = source
      .replace("crate::error", "crate::errors")
      .replace(
        "use serde::Serialize;",
        "use serde::Serialize;\nuse crate::error as relocated_error;",
      );
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
    const sessionAdapter =
      telegramContractPaths.readTelegramContractFile(
        "src-tauri/src/telegram_session_store.rs",
      );
    const sessionCodec =
      telegramContractPaths.readTelegramContractFile(
        "src-tauri/src/telegram/session.rs",
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
        ".initialize_account(account_id, api_id, api_hash, session)",
        "runtime_status_to_wire(runtime_status)",
        "runtime_status == TelegramRuntimeStatus::Ready",
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
        "state.runtime.request_login_code(account_id, phone).await?",
        "Ok(",
      ],
      "send-code remote and state order",
    );
    expect(
      normalizedActiveCallArguments(sendCodeOriginal, "AppError::auth"),
    ).toEqual([]);
    expect(normalizedActiveCallArguments(sendCodeOriginal, "Ok")).toEqual([
      '"Code sent".to_string()',
    ]);
    const signInBody = rustFunctionBody(telegram, "tg_sign_in");
    const signInOriginal = rustFunctionOriginalBody(telegram, "tg_sign_in");
    expectOrdered(
      signInBody,
      [
        "state.runtime.sign_in(account_id, code).await?",
        "save_session(&handle, &secret_store, account_id, &session_to_save)",
        "set_account_status(&handle, &state, account_id, STATUS_READY, None).await",
        "Ok(true)",
      ],
      "sign-in remote-save-status-result order",
    );
    expect(
      normalizedActiveCallArguments(signInOriginal, "AppError::auth"),
    ).toEqual([]);
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
      'format!( "Telegram API hash for account {} is missing from secure storage. Recreate the account credentials.", id )',
    ]);
    expect(rustFunctionBody(telegram, "get_client").trim()).toBe(
      "state.runtime.initialized_client(account_id).await",
    );
    expect(rustFunctionBody(telegram, "get_authorized_client").trim()).toBe(
      "state.runtime.authorized_client(account_id).await",
    );
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
        "runtime.clear_account(account_id, sign_out).await",
        "delete_session(handle, secret_store, account_id).await?",
        "STATUS_NOT_INITIALIZED",
        "Ok(())",
      ],
      "logout client-session-status order",
    );

    const sessionAdapterStructure =
      maskRustLexicalNonCode(sessionAdapter);
    const sessionPathBody = rustFunctionBody(
      sessionAdapter,
      "session_path",
    );
    const sessionPathOriginal = rustFunctionOriginalBody(
      sessionAdapter,
      "session_path",
    );
    expect(sessionPathBody).toContain("Ok(app_dir.join(format!(");
    expect(
      normalizedActiveCallArguments(sessionPathOriginal, "format!"),
    ).toEqual(['"telegram_{account_id}.session.json"']);
    expect(
      countMatches(
        sessionAdapterStructure,
        /\bsession_temp_path\s*\(/g,
      ),
    ).toBe(3);
    expect(
      countMatches(
        sessionAdapterStructure,
        /path\.with_extension\s*\(/g,
      ),
    ).toBe(1);
    expectOrdered(
      rustFunctionBody(sessionAdapter, "write_atomic"),
      [
        "let tmp_path = session_temp_path(path)",
        "fs::write(&tmp_path, contents)",
        "fs::rename(&tmp_path, path)",
      ],
      "session write-before-rename order",
    );
    expect(
      countMatches(
        rustFunctionBody(sessionAdapter, "write_atomic"),
        /map_err\(\|error\| AppError::internal\(error\.to_string\(\)\)\)/g,
      ),
    ).toBe(2);

    assertCheckpointFourSessionContract();
    expect(
      rustFunctionBody(sessionCodec, "encrypt_saved_session"),
    ).toContain("map_err(|_| AppError::internal(");
    const encodeErrors = normalizedActiveCallArguments(
      rustFunctionOriginalBody(sessionCodec, "decode_base64"),
      "AppError::internal",
    );
    expect(encodeErrors).toEqual([
      'format!( "Invalid encrypted Telegram session encoding: {error}" )',
    ]);
    const encryptErrors = normalizedActiveCallArguments(
      rustFunctionOriginalBody(sessionCodec, "encrypt_saved_session"),
      "AppError::internal",
    );
    expect(encryptErrors).toContain('"Invalid Telegram session key length"');
    expect(encryptErrors).toContain('"Failed to encrypt Telegram session"');
    const decryptErrors = normalizedActiveCallArguments(
      rustFunctionOriginalBody(sessionCodec, "decrypt_saved_session"),
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
    expect(
      normalizedActiveCallArguments(
        rustFunctionOriginalBody(sessionCodec, "decode_session_json"),
        "AppError::auth",
      ),
    ).toEqual([
      'format!( "Telegram session key for account {account_id} is missing from secure storage. Sign in again." )',
    ]);
    expect(
      normalizedActiveCallArguments(
        rustFunctionOriginalBody(
          sessionCodec,
          "session_json_requires_existing_key",
        ),
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
        "get_authorized_client(state.inner(), account_id).await?",
        "client_handle.raw_client().clone()",
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
