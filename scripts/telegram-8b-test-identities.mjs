import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(fileURLToPath(new URL("..", import.meta.url)));
const phase8APlanPath = path.join(
  repoRoot,
  "docs/superpowers/plans/2026-07-26-extractum-telegram-8a-preparation.md",
);
const phase8BPlanPath = path.join(
  repoRoot,
  "docs/superpowers/plans/2026-07-28-extractum-telegram-8b-preparation.md",
);
const artifactPath = path.join(
  repoRoot,
  "src/lib/telegram-8b-test-identities.json",
);

const immutableAuthority = {
  startHeading: "## Literal Immutable 140-Test Identity Map",
  endMarker: "\n### Exact New-Test Identity Map",
  bytes: 37_436,
  sha256: "ceab6cef728d396bf2136207f2130974dee2cc0be3c5184eabd8c8de5e58b3ca",
};
const additionAuthority = {
  startHeading: "### Exact New-Test Identity Map",
  endMarker: "\n---\n",
  bytes: 3_717,
  sha256: "a8dce5a0a00ac8cdcf83ef7eab2304f482e7c3967ec26ab8c8270d6fde42f539",
};
const deferredCompanions = [
  "telegram_impl::takeout::raw_parse::tests::raw_parse_preserves_distinct_history_peer_identity_for_equal_message_ids",
  "telegram_impl::takeout::raw_parse::tests::raw_parse_preserves_identical_native_identity_for_same_peer_and_message_id",
];
const phase8BNewAppId =
  "sources::sync::tests::telegram_batch_loop_preserves_entry_durability_limits_and_stops_after_error";
const exactPhase8BRows = [
  "| 3 | `telegram_impl::runtime::tests::client_preserves_missing_account_error_without_authorization_check` | non-authorized opaque lookup |",
  "| 4 | `telegram_impl::live::avatar::tests::peer_photo_bytes_returns_owned_bytes_and_suppresses_timeout_and_transport_failure` | 750 ms owned-byte avatar behavior |",
  "| 4 | `telegram_impl::live::peer::tests::dialog_listing_preserves_dialog_avatar_interleaving_and_budget` | dialog/avatar interleaving, 4 s cutoff, order, and owned descriptors |",
  "| 4 | `telegram_impl::live::peer::tests::resolution_primitives_preserve_username_dialog_and_subtype_outcomes` | app-owned plan primitives and exact outcomes |",
  "| 5 | `telegram_impl::live::messages::tests::message_batch_preserves_single_fetch_order_limit_offsets_and_terminal_rule` | one raw invoke, cache update under the validated `auto_cache_peers` invariant, 1..=100 limit, offsets, ordering, terminal rule, typed `NotModified` rejection |",
  "| 5 | `telegram_impl::live::messages::tests::live_message_maps_owned_draft_and_skips_empty_payload` | per-entry conversion and empty skip |",
  "| 5 | `telegram_impl::live::topics::tests::forum_topic_pages_preserve_order_deleted_ids_and_terminal_cursor` | owned topics/deletions/pagination |",
  "| 5 | `sources::sync::tests::telegram_batch_loop_preserves_entry_durability_limits_and_stops_after_error` | app coordinator durability, raw-message limit, and no post-error fetch |",
  "| 7 | `telegram_impl::takeout::transport::tests::transport_reports_attempt_and_fallback_after_success_or_error` | attempt snapshot and fallback queue after success/error |",
  "| 7 | `telegram_impl::takeout::operations::tests::start_takeout_returns_owned_session_and_selected_ranges` | self-check/init/split selection |",
  "| 7 | `telegram_impl::takeout::operations::tests::migration_probe_and_revalidation_return_owned_chat_identity` | migration detect/revalidate |",
  "| 7 | `telegram_impl::takeout::operations::tests::history_count_preserves_channel_private_fallback_outcome` | classified fallback queue and owned only-my count |",
  "| 7 | `telegram_impl::takeout::operations::tests::history_page_and_search_return_owned_takeout_messages` | concrete page/search operations |",
  "| 7 | `telegram_impl::takeout::operations::tests::finish_takeout_preserves_success_and_error_mapping` | concrete finish behavior |",
  "| 7 | `telegram_impl::takeout::forum_topics::tests::forum_topic_operation_returns_owned_snapshots` | post-Takeout remote topic result |",
];
const rustTestId = /^(?:[A-Za-z_][A-Za-z0-9_]*::)+tests::[A-Za-z_][A-Za-z0-9_]*$/;
const repositoryPath = /^(?!\/)(?!.*\\)(?!.*(?:^|\/)\.{1,2}(?:\/|$))[A-Za-z0-9_.-]+(?:\/[A-Za-z0-9_.-]+)*$/;

/**
 * @typedef {{
 *   startHeading: string,
 *   endMarker: string,
 *   bytes: number,
 *   sha256: string,
 * }} HashedAuthority
 */

/**
 * @typedef {{
 *   baselineFullId: string,
 *   stagedPath: string,
 *   finalOwner: string,
 *   finalFullId: string,
 *   companions: string[],
 * }} ImmutableIdentityRow
 */

/**
 * @typedef {{
 *   checkpoint: number,
 *   currentId: string,
 *   finalId: string,
 *   owner: string,
 * }} AdditionIdentityRow
 */

/**
 * @typedef {{
 *   checkpoint: number,
 *   identity: string,
 * }} Phase8BIdentityRow
 */

/**
 * @typedef {{
 *   schemaVersion: number,
 *   baselineDerived: string[],
 *   preNewApp: string[],
 *   preNewStaged: string[],
 *   phase8BNewApp: string[],
 *   phase8BNewStaged: string[],
 * }} IdentityAuthority
 */

/**
 * @param {string} message
 * @returns {never}
 */
function fail(message) {
  throw new Error(`Telegram Phase 8B test identity authority: ${message}`);
}

/**
 * @param {unknown} source
 * @returns {string}
 */
function normalize(source) {
  if (typeof source !== "string") fail("authority source is not text");
  return source.replace(/\r\n?/g, "\n");
}

/**
 * @param {unknown} source
 * @param {HashedAuthority} authority
 * @param {string} label
 * @returns {string}
 */
function sliceHashedSection(source, authority, label) {
  const normalized = normalize(source);
  const headingMarker = `${authority.startHeading}\n`;
  if (
    normalized.split(headingMarker).length - 1 !== 1
    || !(
      normalized.startsWith(headingMarker)
      || normalized.includes(`\n${headingMarker}`)
    )
  ) {
    fail(`malformed ${label} start heading`);
  }
  const start = normalized.indexOf(authority.startHeading);
  const end = normalized.indexOf(
    authority.endMarker,
    start + authority.startHeading.length,
  );
  if (start < 0 || end <= start) fail(`malformed ${label} end marker`);
  const section = normalized.slice(start, end);
  const bytes = Buffer.byteLength(section, "utf8");
  const sha256 = createHash("sha256").update(section).digest("hex");
  if (bytes !== authority.bytes || sha256 !== authority.sha256) {
    fail(
      `${label} content address drifted: ${bytes} bytes ${sha256}`,
    );
  }
  return section;
}

/**
 * @param {string} section
 * @param {string} header
 * @param {string} separator
 * @param {string} label
 * @returns {string[]}
 */
function tableRows(section, header, separator, label) {
  const lines = section.split("\n");
  const headerIndexes = lines.flatMap((line, index) =>
    line === header ? [index] : []
  );
  if (
    headerIndexes.length !== 1
    || lines[headerIndexes[0] + 1] !== separator
  ) {
    fail(`malformed ${label} table heading`);
  }
  const rows = [];
  for (const line of lines.slice(headerIndexes[0] + 2)) {
    if (line === "") {
      if (rows.length === 0) fail(`empty ${label} table`);
      break;
    }
    if (!line.startsWith("|")) {
      if (rows.length === 0) fail(`empty ${label} table`);
      break;
    }
    rows.push(line);
  }
  if (rows.length === 0) fail(`empty ${label} table`);
  return rows;
}

/**
 * @param {string[]} values
 * @param {string} label
 * @returns {void}
 */
function assertUnique(values, label) {
  const duplicates = values.filter(
    (value, index) => values.indexOf(value) !== index,
  );
  if (duplicates.length !== 0) {
    fail(`${label} contains duplicates: ${[...new Set(duplicates)].join(", ")}`);
  }
}

/**
 * @param {string} value
 * @param {string} label
 * @returns {void}
 */
function validatePath(value, label) {
  if (!repositoryPath.test(value)) fail(`malformed ${label}: ${value}`);
}

/**
 * @param {string} section
 * @returns {ImmutableIdentityRow[]}
 */
function parseImmutableRows(section) {
  const rows = tableRows(
    section,
    "| baseline_package | baseline_full_id | staged_path | final_owner | final_full_id | companion_final_ids |",
    "| --- | --- | --- | --- | --- | --- |",
    "immutable identity",
  ).map((line) => {
    const match = /^\| `([^`]+)` \| `([^`]+)` \| `([^`]+)` \| `([^`]+)` \| `([^`]+)` \| (.*?) \|$/.exec(
      line,
    );
    if (!match) fail(`malformed immutable identity row: ${line}`);
    const [
      ,
      baselinePackage,
      baselineFullId,
      stagedPath,
      finalOwner,
      finalFullId,
      companionCell,
    ] = match;
    if (baselinePackage !== "extractum") {
      fail(`unknown baseline owner: ${baselinePackage}`);
    }
    if (!["extractum", "extractum-telegram"].includes(finalOwner)) {
      fail(`unknown final owner: ${finalOwner}`);
    }
    if (!rustTestId.test(baselineFullId) || !rustTestId.test(finalFullId)) {
      fail(`malformed immutable identity: ${baselineFullId} -> ${finalFullId}`);
    }
    validatePath(stagedPath, "staged path");
    if (
      finalOwner === "extractum"
      && (
        stagedPath !== baselinePathFor(baselineFullId)
        || stagedPath !== baselinePathFor(finalFullId)
      )
    ) {
      fail(`app-owned identity changed path or ID: ${baselineFullId}`);
    }
    if (
      finalOwner === "extractum-telegram"
      && !stagedPath.startsWith("src-tauri/src/telegram_impl/")
    ) {
      fail(`future-owner identity has wrong staged path: ${stagedPath}`);
    }
    if (
      companionCell !== "—"
      && !/^`[^`]+`(?:, `[^`]+`)*$/.test(companionCell)
    ) {
      fail(`malformed companion identity cell: ${companionCell}`);
    }
    const companions = [
      ...companionCell.matchAll(/`([^`]+)`/g),
    ].map((companion) => companion[1]);
    if (companions.some((identity) => !rustTestId.test(identity))) {
      fail(`malformed companion identity in ${baselineFullId}`);
    }
    return {
      baselineFullId,
      stagedPath,
      finalOwner,
      finalFullId,
      companions,
    };
  });
  if (rows.length !== 140) fail(`expected 140 immutable rows, found ${rows.length}`);
  assertUnique(rows.map(({ baselineFullId }) => baselineFullId), "baseline IDs");
  assertUnique(rows.map(({ finalFullId }) => finalFullId), "final IDs");
  assertUnique(
    rows.map(
      ({ baselineFullId, stagedPath }) => `${baselineFullId}\0${stagedPath}`,
    ),
    "identity paths",
  );
  const app = rows.filter(({ finalOwner }) => finalOwner === "extractum");
  const staged = rows.filter(
    ({ finalOwner }) => finalOwner === "extractum-telegram",
  );
  if (app.length !== 99 || staged.length !== 41) {
    fail(`immutable owner counts drifted: app=${app.length} staged=${staged.length}`);
  }
  return rows;
}

/**
 * @param {string} identity
 * @returns {string}
 */
function baselinePathFor(identity) {
  /** @type {ReadonlyArray<readonly [string, string]>} */
  const mappings = [
    ["takeout_import::export_dc::", "src-tauri/src/takeout_import/export_dc.rs"],
    ["takeout_import::forum_topics::", "src-tauri/src/takeout_import/forum_topics.rs"],
    ["takeout_import::migrated_history::", "src-tauri/src/takeout_import/migrated_history.rs"],
    ["takeout_import::pagination::", "src-tauri/src/takeout_import/pagination.rs"],
    ["takeout_import::raw_parse::", "src-tauri/src/takeout_import/raw_parse.rs"],
    ["takeout_import::", "src-tauri/src/takeout_import/mod.rs"],
    ["sources::identity::", "src-tauri/src/sources/identity.rs"],
    ["sources::items::", "src-tauri/src/sources/items.rs"],
    ["sources::peer_resolution::", "src-tauri/src/sources/peer_resolution.rs"],
    ["sources::sync::", "src-tauri/src/sources/sync.rs"],
    ["sources::topics::", "src-tauri/src/sources/topics.rs"],
    ["sources::types::", "src-tauri/src/sources/types.rs"],
    ["ingest_provenance::", "src-tauri/src/ingest_provenance.rs"],
    ["telegram_session_store::", "src-tauri/src/telegram_session_store.rs"],
    ["telegram::", "src-tauri/src/telegram.rs"],
    ["media::", "src-tauri/src/media.rs"],
  ];
  const mapping = mappings.find(([prefix]) => identity.startsWith(prefix));
  if (!mapping) fail(`unknown baseline identity path: ${identity}`);
  return mapping[1];
}

/**
 * @param {string} identity
 * @returns {string}
 */
function stagedIdentity(identity) {
  return identity.startsWith("telegram_impl::")
    ? identity
    : `telegram_impl::${identity}`;
}

/**
 * @param {ImmutableIdentityRow} row
 * @returns {string}
 */
function finalIdentity(row) {
  return row.finalOwner === "extractum"
    ? row.finalFullId
    : stagedIdentity(row.finalFullId);
}

/**
 * @param {string} section
 * @returns {AdditionIdentityRow[]}
 */
function parseAdditionRows(section) {
  const rows = tableRows(
    section,
    "| Checkpoint | Phase 8A exact identity in `extractum` | Future owner / final identity |",
    "| --- | --- | --- |",
    "Phase 8A addition",
  ).map((line) => {
    const match = /^\| ([2-5]) \| `([^`]+)` \| (app \/ unchanged|crate \/ `([^`]+)`) \|$/.exec(
      line,
    );
    if (!match) fail(`malformed Phase 8A addition row: ${line}`);
    const currentId = match[2];
    const crateId = match[4];
    if (!rustTestId.test(currentId)) {
      fail(`malformed Phase 8A current identity: ${currentId}`);
    }
    if (crateId) {
      if (!rustTestId.test(crateId) || currentId !== `telegram::${crateId}`) {
        fail(`wrong Phase 8A future identity: ${currentId} -> ${crateId}`);
      }
    }
    return {
      checkpoint: Number(match[1]),
      currentId,
      finalId: crateId ? stagedIdentity(crateId) : currentId,
      owner: crateId ? "staged" : "app",
    };
  });
  if (rows.length !== 18) fail(`expected 18 Phase 8A additions, found ${rows.length}`);
  assertUnique(rows.map(({ currentId }) => currentId), "Phase 8A current IDs");
  assertUnique(rows.map(({ finalId }) => finalId), "Phase 8A final IDs");
  return rows;
}

/**
 * @param {unknown} source
 * @returns {string}
 */
function phase8BTableSection(source) {
  const normalized = normalize(source);
  const startHeading = "### Exact Phase 8B New-Test Table";
  const endMarker = "\nThe two deferred companions are separate";
  if (
    normalized.split(`${startHeading}\n`).length - 1 !== 1
    || !normalized.includes(`\n${startHeading}\n`)
  ) {
    fail("malformed Phase 8B new-test heading");
  }
  const start = normalized.indexOf(startHeading);
  const end = normalized.indexOf(endMarker, start + startHeading.length);
  if (start < 0 || end <= start) fail("malformed Phase 8B new-test end marker");
  return normalized.slice(start, end);
}

/**
 * @param {string} section
 * @returns {Phase8BIdentityRow[]}
 */
function parsePhase8BRows(section) {
  const rawRows = tableRows(
    section,
    "| Checkpoint | Exact identity | Subject |",
    "| ---: | --- | --- |",
    "Phase 8B new-test",
  );
  if (
    rawRows.length !== exactPhase8BRows.length
    || rawRows.some((line, index) => line !== exactPhase8BRows[index])
  ) {
    fail("exact Phase 8B row order, checkpoint, identity, or subject drifted");
  }
  if (!section.endsWith(`${rawRows.at(-1)}\n`)) {
    fail("malformed trailing content after the Phase 8B table");
  }
  const rows = rawRows.map((line) => {
    const match = /^\| ([3457]) \| `([^`]+)` \| ([^|]+) \|$/.exec(line);
    if (!match || !match[3].trim()) {
      fail(`malformed Phase 8B new-test row: ${line}`);
    }
    if (!rustTestId.test(match[2])) {
      fail(`malformed Phase 8B test identity: ${match[2]}`);
    }
    return { checkpoint: Number(match[1]), identity: match[2] };
  });
  if (rows.length !== 15) fail(`expected 15 Phase 8B rows, found ${rows.length}`);
  assertUnique(rows.map(({ identity }) => identity), "Phase 8B new IDs");
  const checkpoints = new Map([
    [3, 1],
    [4, 3],
    [5, 4],
    [7, 7],
  ]);
  for (const [checkpoint, expected] of checkpoints) {
    const actual = rows.filter((row) => row.checkpoint === checkpoint).length;
    if (actual !== expected) {
      fail(`Checkpoint ${checkpoint} allocation drifted: ${actual} != ${expected}`);
    }
  }
  const app = rows.filter(({ identity }) => !identity.startsWith("telegram_impl::"));
  const staged = rows.filter(({ identity }) => identity.startsWith("telegram_impl::"));
  if (
    app.length !== 1
    || app[0].identity !== phase8BNewAppId
    || staged.length !== 14
  ) {
    fail("Phase 8B owner-prefix allocation drifted");
  }
  return rows;
}

/**
 * @param {unknown} phase8ASource
 * @param {unknown} phase8BSource
 * @returns {IdentityAuthority}
 */
export function generateIdentityAuthority(phase8ASource, phase8BSource) {
  const immutableRows = parseImmutableRows(
    sliceHashedSection(
      phase8ASource,
      immutableAuthority,
      "immutable Phase 8A identity map",
    ),
  );
  const additionRows = parseAdditionRows(
    sliceHashedSection(
      phase8ASource,
      additionAuthority,
      "Phase 8A exact-addition map",
    ),
  );
  const phase8BRows = parsePhase8BRows(phase8BTableSection(phase8BSource));

  const companionIds = immutableRows.flatMap(({ companions }) => companions);
  assertUnique(companionIds, "baseline companions");
  const expectedCompanions = [
    "dto::tests::telegram_item_kind_constant_matches_persisted_wire_value",
    "takeout::raw_parse::tests::raw_parse_preserves_distinct_history_peer_identity_for_equal_message_ids",
    "takeout::raw_parse::tests::raw_parse_preserves_identical_native_identity_for_same_peer_and_message_id",
  ];
  if (
    [...companionIds].sort().join("\n")
    !== [...expectedCompanions].sort().join("\n")
  ) {
    fail("baseline companion identities drifted");
  }

  const baselineDerived = [
    ...immutableRows.map(finalIdentity),
    ...companionIds.map(stagedIdentity),
  ].sort();
  const preNew = [
    ...additionRows.map(({ finalId }) => finalId),
    ...immutableRows.map(finalIdentity),
    ...deferredCompanions,
  ];
  const phase8BNewApp = phase8BRows
    .filter(({ identity }) => !identity.startsWith("telegram_impl::"))
    .map(({ identity }) => identity)
    .sort();
  const phase8BNewStaged = phase8BRows
    .filter(({ identity }) => identity.startsWith("telegram_impl::"))
    .map(({ identity }) => identity)
    .sort();
  assertUnique(baselineDerived, "baseline-derived IDs");
  assertUnique(preNew, "pre-new tracked IDs");
  assertUnique(
    [...preNew, ...phase8BNewApp, ...phase8BNewStaged],
    "final tracked IDs",
  );
  const preNewApp = preNew
    .filter((identity) => !identity.startsWith("telegram_impl::"))
    .sort();
  const preNewStaged = preNew
    .filter((identity) => identity.startsWith("telegram_impl::"))
    .sort();
  const metrics = {
    currentPackage: 719,
    eventualPackage: 736,
    baselineDerived: baselineDerived.length,
    preNewTracked: preNew.length,
    finalTracked: preNew.length + phase8BRows.length,
    finalApp: preNewApp.length + phase8BNewApp.length,
    finalStaged: preNewStaged.length + phase8BNewStaged.length,
  };
  const expectedMetrics = {
    currentPackage: 719,
    eventualPackage: 736,
    baselineDerived: 143,
    preNewTracked: 160,
    finalTracked: 175,
    finalApp: 104,
    finalStaged: 71,
  };
  if (JSON.stringify(metrics) !== JSON.stringify(expectedMetrics)) {
    fail(`identity metrics drifted: ${JSON.stringify(metrics)}`);
  }
  if (
    preNewApp.length !== 103
    || preNewStaged.length !== 57
    || phase8BNewApp.length !== 1
    || phase8BNewStaged.length !== 14
  ) {
    fail("identity partition counts drifted");
  }
  const tracked = new Set([
    ...preNewApp,
    ...preNewStaged,
    ...phase8BNewApp,
    ...phase8BNewStaged,
  ]);
  if (baselineDerived.some((identity) => !tracked.has(identity))) {
    fail("baseline-derived identity is absent from final tracked authority");
  }

  return {
    schemaVersion: 1,
    baselineDerived,
    preNewApp,
    preNewStaged,
    phase8BNewApp,
    phase8BNewStaged,
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
  const generated = generateIdentityAuthority(
    readFileSync(phase8APlanPath, "utf8"),
    readFileSync(phase8BPlanPath, "utf8"),
  );
  const rendered = serialize(generated);
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
