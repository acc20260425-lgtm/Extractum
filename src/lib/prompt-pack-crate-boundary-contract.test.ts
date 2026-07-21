import { createHash } from "node:crypto";
import { existsSync, readFileSync, readdirSync } from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";

const repositoryRoot = path.resolve(import.meta.dirname, "../..");
const appRoot = "src-tauri/src/prompt_packs";
const crateRoot = "src-tauri/crates/extractum-prompt-packs";
const crateManifestPath = `${crateRoot}/Cargo.toml`;
const crateExtracted = existsSync(path.join(repositoryRoot, crateManifestPath));

const normalize = (value: string) => value.replace(/\r\n/g, "\n");
const toRepositoryPath = (absolutePath: string) =>
  path.relative(repositoryRoot, absolutePath).replaceAll("\\", "/");
const read = (relativePath: string) =>
  normalize(readFileSync(path.join(repositoryRoot, relativePath), "utf8"));
const listFiles = (relativeDirectory: string, extension?: string): string[] => {
  const absoluteDirectory = path.join(repositoryRoot, relativeDirectory);
  if (!existsSync(absoluteDirectory)) return [];
  return readdirSync(absoluteDirectory, { withFileTypes: true }).flatMap((entry) => {
    const relativePath = path
      .join(relativeDirectory, entry.name)
      .replaceAll("\\", "/");
    if (entry.isDirectory()) return listFiles(relativePath, extension);
    if (!entry.isFile() || (extension && !entry.name.endsWith(extension))) return [];
    return [relativePath];
  });
};
const sorted = <T extends string>(values: readonly T[]) => [...values].sort();

const matchingDelimiter = (
  source: string,
  openIndex: number,
  open: string,
  close: string,
) => {
  let depth = 0;
  for (let index = openIndex; index < source.length; index += 1) {
    if (source.startsWith("//", index)) {
      const newline = source.indexOf("\n", index + 2);
      index = newline < 0 ? source.length : newline;
      continue;
    }
    if (source.startsWith("/*", index)) {
      let commentDepth = 1;
      index += 2;
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
      index -= 1;
      continue;
    }
    const rawString = source.slice(index).match(/^(?:b)?r(#{0,255})"/);
    if (rawString) {
      const closing = `"${rawString[1]}`;
      const end = source.indexOf(closing, index + rawString[0].length);
      if (end < 0) throw new Error("unclosed Rust raw string literal");
      index = end + closing.length - 1;
      continue;
    }
    const quoteIndex = source[index] === '"' ? index : source.startsWith('b"', index) ? index + 1 : -1;
    if (quoteIndex >= 0) {
      index = quoteIndex + 1;
      while (index < source.length) {
        if (source[index] === "\\") index += 2;
        else if (source[index] === '"') break;
        else index += 1;
      }
      if (index >= source.length) throw new Error("unclosed Rust string literal");
      continue;
    }
    if (
      source[index] === "'" &&
      ((source[index + 1] === "\\" && source[index + 3] === "'") ||
        source[index + 2] === "'")
    ) {
      index += source[index + 1] === "\\" ? 3 : 2;
      continue;
    }
    if (source[index] === open) depth += 1;
    if (source[index] === close) depth -= 1;
    if (depth === 0) return index;
  }
  throw new Error(`unclosed ${open}${close} delimiter`);
};
const rustBlock = (source: string, marker: string) => {
  const start = source.indexOf(marker);
  if (start < 0) throw new Error(`missing Rust block marker: ${marker}`);
  const open = source.indexOf("{", start);
  if (open < 0) throw new Error(`missing Rust block opening brace: ${marker}`);
  return source.slice(start, matchingDelimiter(source, open, "{", "}") + 1);
};
const stripCfgTestItems = (source: string) => {
  const cfgTest = /#\s*\[\s*cfg\s*\(\s*test\s*\)\s*\]/g;
  let cursor = 0;
  let result = "";
  for (let match = cfgTest.exec(source); match; match = cfgTest.exec(source)) {
    result += source.slice(cursor, match.index);
    let itemStart = cfgTest.lastIndex;
    while (/\s/.test(source[itemStart] ?? "")) itemStart += 1;
    while (source.startsWith("#[", itemStart)) {
      const attributeEnd = source.indexOf("]", itemStart);
      if (attributeEnd < 0) throw new Error("unclosed Rust attribute after #[cfg(test)]");
      itemStart = attributeEnd + 1;
      while (/\s/.test(source[itemStart] ?? "")) itemStart += 1;
    }
    const brace = source.indexOf("{", itemStart);
    const semicolon = source.indexOf(";", itemStart);
    if (semicolon >= 0 && (brace < 0 || semicolon < brace)) {
      cursor = semicolon + 1;
    } else if (brace >= 0) {
      cursor = matchingDelimiter(source, brace, "{", "}") + 1;
    } else {
      throw new Error("unparseable Rust item after #[cfg(test)]");
    }
    cfgTest.lastIndex = cursor;
  }
  return result + source.slice(cursor);
};

const rustTestPattern =
  /^[\t ]*#\[(?:tokio::)?test(?:\([^\]]*\))?\][\t ]*\n(?:^[\t ]*#\[[^\]]+\][\t ]*\n)*^[\t ]*(?:async\s+)?fn\s+([A-Za-z0-9_]+)/gm;
const rustTestNames = (source: string) =>
  [...source.matchAll(new RegExp(rustTestPattern.source, "gm"))].map(
    (match) => match[1],
  );

const tomlSection = (source: string, heading: string) => {
  const marker = `[${heading}]`;
  const start = source.indexOf(marker);
  if (start < 0) return "";
  const bodyStart = start + marker.length;
  const next = source.slice(bodyStart).search(/^\[\[?[^\n]+\]?\]$/m);
  return source.slice(bodyStart, next < 0 ? undefined : bodyStart + next).trim();
};
const dependencyNames = (section: string) =>
  [...section.matchAll(/^([A-Za-z0-9_-]+)(?:\.workspace)?\s*=/gm)]
    .map((match) => match[1])
    .sort();
const lockPackages = (source: string, name: string) =>
  source
    .split(/(?=^\[\[package\]\]$)/m)
    .filter((block) => block.includes(`\nname = "${name}"\n`));
const lockDependencies = (block: string) => {
  const body = block.match(/^dependencies = \[\n([\s\S]*?)^\]$/m)?.[1];
  return body
    ? [...body.matchAll(/^ "([^"]+)",?$/gm)]
        .map((match) => match[1].replace(/ \d+\..*$/, ""))
        .sort()
    : [];
};

const crateRootFiles = [
  "assets.rs",
  "browser_port.rs",
  "completion_transport.rs",
  "dto.rs",
  "events.rs",
  "gemini_browser_stage.rs",
  "json_repair.rs",
  "lib.rs",
  "library.rs",
  "models.rs",
  "projections.rs",
  "result_builder.rs",
  "result_service.rs",
  "run_control.rs",
  "run_store.rs",
  "runtime.rs",
  "runtime_config.rs",
  "seed.rs",
  "source_port.rs",
  "stage_execution.rs",
  "stage_io.rs",
  "stage_output_normalization.rs",
  "stage_request_policy.rs",
  "store.rs",
  "test_schema.rs",
  "validation.rs",
] as const;
const crateYoutubeFiles = [
  "entities.rs",
  "entities_tests.rs",
  "execution.rs",
  "execution_result.rs",
  "execution_tests.rs",
  "facade_tests.rs",
  "gem_analysis.rs",
  "mod.rs",
  "outputs.rs",
  "outputs_tests.rs",
  "preflight.rs",
  "preflight_tests.rs",
  "progress.rs",
  "result_validation.rs",
  "snapshots.rs",
  "snapshots_tests.rs",
  "store.rs",
  "synthesis_execution.rs",
  "synthesis_input.rs",
  "synthesis_input_tests.rs",
  "tail_stages.rs",
  "test_support.rs",
  "transcript_execution.rs",
  "types.rs",
] as const;
const finalAppFiles = [
  "browser_adapter.rs",
  "event_adapter.rs",
  "library_command.rs",
  "mod.rs",
  "result_commands.rs",
  "runtime_commands.rs",
  "seed_command.rs",
  "source_adapter.rs",
  "youtube_summary/mod.rs",
  "youtube_summary/snapshots_tests.rs",
  "youtube_summary/test_support.rs",
] as const;

const crateModuleNames = [
  ...crateRootFiles.filter((file) => !["lib.rs", "test_schema.rs"].includes(file)).map(
    (file) => file.replace(/\.rs$/, ""),
  ),
  "test_schema",
  "youtube_summary",
].sort();
const finalCrateRustFiles = [
  ...crateRootFiles.map((file) => `${crateRoot}/src/${file}`),
  ...crateYoutubeFiles.map((file) => `${crateRoot}/src/youtube_summary/${file}`),
];
const finalAppRustFiles = finalAppFiles.map((file) => `${appRoot}/${file}`);

const preparedRustFiles = sorted([
  ...new Set([
    ...crateRootFiles.map((file) =>
      file === "lib.rs" ? `${appRoot}/mod.rs` : `${appRoot}/${file}`,
    ),
    ...crateYoutubeFiles.map((file) =>
      file === "snapshots_tests.rs"
        ? `${appRoot}/youtube_summary/domain_snapshots_tests.rs`
        : `${appRoot}/youtube_summary/${file}`,
    ),
    ...finalAppFiles.map((file) =>
      file === "youtube_summary/test_support.rs"
        ? `${appRoot}/youtube_summary/app_test_support.rs`
        : `${appRoot}/${file}`,
    ),
  ]),
]);

const domainPath = (relativePath: string) => {
  if (crateExtracted) return `${crateRoot}/src/${relativePath}`;
  if (relativePath === "lib.rs") return `${appRoot}/mod.rs`;
  if (relativePath === "youtube_summary/snapshots_tests.rs") {
    return `${appRoot}/youtube_summary/domain_snapshots_tests.rs`;
  }
  return `${appRoot}/${relativePath}`;
};

type FrozenTest = {
  logicalFile: string;
  name: string;
  owner: "app" | "crate";
};
const appOwnedIdentities = new Set([
  "youtube_summary/snapshots_tests.rs::transcript_text_for_source_uses_segment_renderer",
  "youtube_summary/snapshots_tests.rs::comment_snapshot_selection_is_deterministic_when_enabled",
]);
const parseFrozenTests = (spec: string): FrozenTest[] => {
  const appendixMarker = "## Appendix A: Frozen 225-Test Baseline";
  const appendixStart = spec.indexOf(appendixMarker);
  if (appendixStart < 0) throw new Error("missing Appendix A");
  const appendix = spec.slice(appendixStart);
  const youtubeStart = appendix.indexOf("### `youtube_summary` modules (125)");
  if (youtubeStart < 0) throw new Error("missing Appendix A youtube_summary group");
  const headings = [...appendix.matchAll(/^#### `([^`]+\.rs)` \((\d+)\)$/gm)];
  if (headings.length === 0) throw new Error("Appendix A contains no logical file headings");
  return headings.flatMap((heading, headingIndex) => {
    if (heading.index === undefined) throw new Error("Appendix A heading has no index");
    const bodyStart = heading.index + heading[0].length;
    const bodyEnd = headings[headingIndex + 1]?.index ?? appendix.length;
    const names = [
      ...appendix.slice(bodyStart, bodyEnd).matchAll(/^- `([A-Za-z0-9_]+)`$/gm),
    ].map((match) => match[1]);
    const declaredCount = Number(heading[2]);
    if (names.length !== declaredCount) {
      throw new Error(
        `Appendix A ${heading[1]} declares ${declaredCount}, parsed ${names.length}`,
      );
    }
    const logicalFile = `${heading.index > youtubeStart ? "youtube_summary/" : ""}${heading[1]}`;
    return names.map((name) => {
      const identity = `${logicalFile}::${name}`;
      return {
        logicalFile,
        name,
        owner: appOwnedIdentities.has(identity) ? "app" : "crate",
      } satisfies FrozenTest;
    });
  });
};

const boundarySpec = read(
  "docs/superpowers/specs/2026-07-20-prompt-packs-crate-boundary-design.md",
);
const frozenTests = parseFrozenTests(boundarySpec);
const appOwnedTestPath = `${appRoot}/youtube_summary/snapshots_tests.rs`;
const frozenTestPath = (test: FrozenTest) =>
  test.owner === "app" ? appOwnedTestPath : domainPath(test.logicalFile);

const promptPackTables = [
  "prompt_packs",
  "prompt_pack_versions",
  "prompt_pack_stage_templates",
  "prompt_pack_schema_assets",
  "prompt_pack_runs",
  "prompt_pack_run_scopes",
  "prompt_pack_run_source_snapshots",
  "prompt_pack_run_source_origins",
  "prompt_pack_run_material_snapshots",
  "prompt_pack_stage_runs",
  "prompt_pack_stage_artifacts",
  "prompt_pack_results",
  "prompt_pack_result_source_refs",
  "prompt_pack_result_claims",
  "prompt_pack_result_evidence",
  "prompt_pack_result_ref_edges",
  "prompt_pack_result_unknowns",
  "prompt_pack_result_verification_tasks",
  "prompt_pack_result_warnings",
  "prompt_pack_result_limitations",
  "prompt_pack_result_quality_flags",
  "prompt_pack_result_audit_refs",
  "prompt_pack_youtube_videos",
  "prompt_pack_youtube_segments",
  "prompt_pack_youtube_key_points",
  "prompt_pack_youtube_quotes",
  "prompt_pack_youtube_action_items",
  "prompt_pack_youtube_open_questions",
  "prompt_pack_youtube_synthesis_items",
  "prompt_pack_result_validation_findings",
  "prompt_pack_audit_events",
  "prompt_pack_result_quarantine_artifacts",
] as const;
const foreignTables = [
  "sources",
  "youtube_video_sources",
  "youtube_playlist_items",
  "youtube_transcript_segments",
  "items",
  "projects",
] as const;

const productionRootFiles = crateRootFiles.filter(
  (file) => !["lib.rs", "test_schema.rs"].includes(file),
);
const productionYoutubeFiles = crateYoutubeFiles.filter(
  (file) => !file.endsWith("_tests.rs") && file !== "test_support.rs",
);
const productionDomainFiles = [
  ...productionRootFiles.map((file) => domainPath(file)),
  ...productionYoutubeFiles.map((file) => domainPath(`youtube_summary/${file}`)),
];
const productionDomainSource = productionDomainFiles
  .map((file) => stripCfgTestItems(read(file)))
  .join("\n");

const expectedCrateManifest = `[package]
name = "extractum-prompt-packs"
version.workspace = true
edition.workspace = true
publish = false

[dependencies]
extractum-core = { path = "../extractum-core" }
extractum-gemini-browser = { path = "../extractum-gemini-browser" }
extractum-llm = { path = "../extractum-llm" }
jsonschema = { version = "0.46.5", default-features = false }
serde.workspace = true
serde_json.workspace = true
sha2.workspace = true
sqlx.workspace = true
tokio = { workspace = true, features = ["macros", "sync"] }
tokio-util.workspace = true

[dev-dependencies]
tempfile.workspace = true
time.workspace = true
tokio = { workspace = true, features = ["io-util", "net", "rt", "time"] }
`;

const curatedRootExports = [
  "CommentBodyReadRequest",
  "CommentCandidateReadRequest",
  "ListPromptPackRunsRequest",
  "PreparedApiRunExecution",
  "PreparedBrowserRunExecution",
  "PreparedRunExecution",
  "PreflightYoutubeSummaryRunRequest",
  "PromptPackAuditEventDto",
  "PromptPackBrowserCancelRequest",
  "PromptPackBrowserExecutor",
  "PromptPackBrowserFuture",
  "PromptPackBrowserRunRequest",
  "PromptPackBrowserStatusRequest",
  "PromptPackCommentCandidate",
  "PromptPackDto",
  "PromptPackEvent",
  "PromptPackEventSink",
  "PromptPackLibraryDto",
  "PromptPackPlaylistItemRecord",
  "PromptPackPortFuture",
  "PromptPackResultDto",
  "PromptPackRunState",
  "PromptPackRunSummaryDto",
  "PromptPackRuntimeProvider",
  "PromptPackSchemaAssetDto",
  "PromptPackSourceReader",
  "PromptPackSourceRecord",
  "PromptPackStageArtifactDto",
  "PromptPackStageArtifactSummaryDto",
  "PromptPackStageRunDto",
  "PromptPackStageTemplateDto",
  "PromptPackTranscriptSegment",
  "PromptPackValidationFindingDto",
  "PromptPackVersionDto",
  "PromptPackYoutubeVideoRecord",
  "RunExecutionTicket",
  "StartServiceOutcome",
  "StartYoutubeSummaryRunOutcomeDto",
  "StartYoutubeSummaryRunRequest",
  "YoutubeSummaryPreflightFailure",
  "YoutubeSummaryPreflightResponse",
  "YoutubeSummaryPreflightSkippedVideo",
  "YoutubeSummaryPreflightVideo",
  "YoutubeSummaryRunExecutionOutcome",
  "YoutubeVideoReadRequest",
  "cancel_prompt_pack_run_in_pool",
  "cleanup_interrupted_prompt_pack_runs_in_pool",
  "clear_prompt_pack_cancellation_smoke_fixture_in_pool",
  "delete_prompt_pack_run_in_pool",
  "execute_prepared_api_run",
  "execute_prepared_browser_run",
  "fail_run_execution",
  "get_prompt_pack_library_in_pool",
  "get_prompt_pack_result_in_pool",
  "get_prompt_pack_stage_artifact_in_pool",
  "get_prompt_pack_validation_findings_in_pool",
  "list_active_prompt_pack_runs_in_pool",
  "list_prompt_pack_audit_events_in_pool",
  "list_prompt_pack_run_stages_in_pool",
  "list_prompt_pack_runs_in_pool",
  "list_prompt_pack_stage_artifacts_in_pool",
  "preflight_youtube_summary_run",
  "prepare_run_execution",
  "seed_builtin_prompt_packs_in_pool",
  "seed_prompt_pack_cancellation_smoke_fixture_in_pool",
  "start_youtube_summary_run_service",
  "update_prompt_pack_run_in_pool",
] as const;

const publicReexportNames = (source: string) => {
  const names: string[] = [];
  for (const match of source.matchAll(/^pub use\s+([\s\S]*?);$/gm)) {
    const expression = match[1].trim();
    const brace = expression.match(/^.+?::\{([\s\S]*)\}$/);
    const items = brace ? brace[1].split(",") : [expression];
    for (const item of items) {
      const normalizedItem = item.trim();
      if (!normalizedItem) continue;
      names.push(
        normalizedItem
          .replace(/\s+as\s+.+$/, "")
          .split("::")
          .at(-1) as string,
      );
    }
  }
  return names.sort();
};
const moduleNames = (source: string) =>
  [...source.matchAll(/^(?:#\[[^\]]+\]\n)?(?:pub(?:\([^)]*\))?\s+)?mod\s+(\w+)\s*;/gm)]
    .map((match) => match[1])
    .sort();

const assetContract = {
  PACK_JSON: [
    "pack.json",
    "21d0e7803f25474bb761cbe5c9fe6e45ef363cf5d9c7f030f7c84ee02ef9b7d8dd3664dfed782a3e8c607b7a0f37cf06",
  ],
  SYNTHESIS_RUNTIME_JSON: [
    "runtime/synthesis.json",
    "36b1c4653bc4befdcd168b482929f3b34980c58d9179cb0e0e3db9ac4d3760f9e66dc834ad6a799df6df62618b28d367",
  ],
  TRANSCRIPT_RUNTIME_JSON: [
    "runtime/transcript_analysis.json",
    "a9ba63c8ff582429866042aad354693cf9a583f5fc05f319189f44266d9eec6871b0ceb40758719a4b0d95dc8f25ee8f",
  ],
  CANONICAL_RESULT_SCHEMA_JSON: [
    "schemas/canonical-result.json",
    "067ac18d452b6ec6ca2000899d3e7d8df87ace30e4676c7f88080a59cc4731887032943c7ea961ac39b69ab17e9697fd",
  ],
  SYNTHESIS_OUTPUT_SCHEMA_JSON: [
    "schemas/stage-io-youtube-summary-synthesis-output.json",
    "ff518213fba16805dfbde2c6c55f8d3ca204ca7f772fb2348cfc375e83070289bfd29623ea4af1b78044504e92a22dac",
  ],
  TRANSCRIPT_INPUT_SCHEMA_JSON: [
    "schemas/stage-io-youtube-summary-transcript-analysis-input.json",
    "bb75aad9fd645912f723ad470a715f7b43c3af964ee4ea74cd84bebb635a1d3bc5bb0ac5460c9608e15eabee07b74419",
  ],
  TRANSCRIPT_OUTPUT_SCHEMA_JSON: [
    "schemas/stage-io-youtube-summary-transcript-analysis-output.json",
    "9d3d32cf7b7bfd00fdc5ae6d74dac8ad06f488b05e31e52866553aeaa1cd836c1d6599d5dd21c2228abf51e4bcc5f693",
  ],
  TRANSCRIPT_STAGE_JSON: [
    "stages/transcript_analysis.json",
    "1b4f18dc3b1baf4b01389a6187d54b96ed689dc044aefd6338a2a176779f433359b0bdc77364fec1ef2ccb58a9088793",
  ],
} as const;

const parseMigrationRegistry = (source: string) => {
  const buildStart = source.indexOf("pub fn build_migrations() -> Vec<Migration>");
  if (buildStart < 0) throw new Error("missing build_migrations() registry");
  const vectorMarker = "let mut migrations = vec![";
  const vectorStart = source.indexOf(vectorMarker, buildStart);
  const extendStart = source.indexOf(
    "migrations.extend(apalis_sqlite_migrations())",
    vectorStart,
  );
  if (vectorStart < 0 || extendStart < 0) {
    throw new Error("unparseable build_migrations() non-Apalis prefix");
  }
  const vectorClose = source.lastIndexOf("];", extendStart);
  if (vectorClose < vectorStart) throw new Error("missing migration vector closing ];");
  const vectorBody = source.slice(vectorStart + vectorMarker.length, vectorClose);
  const calls = [...vectorBody.matchAll(/^\s*([a-z][a-z0-9_]*)\(\),\s*$/gm)].map(
    (match) => match[1],
  );
  if (calls.length !== 12) throw new Error(`expected 12 migration calls, found ${calls.length}`);
  if (vectorBody.replace(/^\s*[a-z][a-z0-9_]*\(\),\s*$/gm, "").trim() !== "") {
    throw new Error("unmatched token in build_migrations() prefix");
  }
  return calls.map((functionName) => {
    const matches = [
      ...source.matchAll(new RegExp(`^fn ${functionName}\\(\\) -> Migration \\{`, "gm")),
    ];
    if (matches.length !== 1 || matches[0].index === undefined) {
      throw new Error(`expected one migration function ${functionName}`);
    }
    const body = rustBlock(source.slice(matches[0].index), `fn ${functionName}()`);
    const sqlTokens = [...body.matchAll(/^\s*sql:\s*([A-Z][A-Z0-9_]+),\s*$/gm)].map(
      (match) => match[1],
    );
    if (sqlTokens.length !== 1) {
      throw new Error(`expected one sql token in ${functionName}`);
    }
    const constantPattern = new RegExp(
      `^const ${sqlTokens[0]}: &str =\\s*include_str!\\("([^"]+)"\\);$`,
      "gm",
    );
    const constants = [...source.matchAll(constantPattern)];
    if (constants.length !== 1) {
      throw new Error(`expected one include_str! constant ${sqlTokens[0]}`);
    }
    return constants[0][1];
  });
};

const parseFixtureMigrations = (source: string) => {
  const declaration =
    /const PROMPT_PACK_TEST_MIGRATIONS:\s*\[\(&str, &str\);\s*(\d+)\]\s*=\s*\[/m.exec(
      source,
    );
  if (!declaration || declaration.index === undefined) {
    throw new Error("unparseable PROMPT_PACK_TEST_MIGRATIONS declaration");
  }
  if (Number(declaration[1]) !== 12) {
    throw new Error(`fixture declares ${declaration[1]} migrations instead of 12`);
  }
  const bodyStart = declaration.index + declaration[0].length;
  const bodyEnd = source.indexOf("];", bodyStart);
  if (bodyEnd < 0) throw new Error("missing PROMPT_PACK_TEST_MIGRATIONS closing ];");
  const body = source.slice(bodyStart, bodyEnd);
  const pairPattern =
    /\(\s*"([^"]+)"\s*,\s*include_str!\(\s*"([^"]+)"\s*\)\s*,?\s*\)\s*,?/g;
  const pairs = [...body.matchAll(pairPattern)].map((match) => ({
    repositoryPath: match[1],
    includePath: match[2],
  }));
  if (pairs.length !== 12) throw new Error(`expected 12 fixture pairs, found ${pairs.length}`);
  if (body.replace(pairPattern, "").trim() !== "") {
    throw new Error("unmatched token in PROMPT_PACK_TEST_MIGRATIONS");
  }
  return pairs;
};

describe("extractum-prompt-packs crate boundary", () => {
  it("declares one app edge and the exact locked dependency surface", () => {
    expect(
      crateExtracted,
      "extractum-prompt-packs Cargo.toml is intentionally absent before the mechanical move",
    ).toBe(true);

    const rootCargo = read("src-tauri/Cargo.toml");
    const crateCargo = read(crateManifestPath);
    const cargoLock = read("src-tauri/Cargo.lock");
    const members = tomlSection(rootCargo, "workspace")
      .match(/^members\s*=\s*\[([^\]]+)\]$/m)?.[1]
      .split(",")
      .map((member) => member.trim().replace(/^"|"$/g, ""));
    expect(members).toEqual([
      ".",
      "crates/extractum-core",
      "crates/extractum-gemini-browser",
      "crates/extractum-llm",
      "crates/extractum-prompt-packs",
    ]);
    expect(crateCargo).toBe(expectedCrateManifest);
    expect(dependencyNames(tomlSection(crateCargo, "dependencies"))).toEqual(
      sorted([
        "extractum-core",
        "extractum-gemini-browser",
        "extractum-llm",
        "jsonschema",
        "serde",
        "serde_json",
        "sha2",
        "sqlx",
        "tokio",
        "tokio-util",
      ]),
    );
    expect(dependencyNames(tomlSection(crateCargo, "dev-dependencies"))).toEqual([
      "tempfile",
      "time",
      "tokio",
    ]);
    expect(tomlSection(rootCargo, "workspace.dependencies")).toContain('sha2 = "0.10"');
    expect(tomlSection(rootCargo, "workspace.dependencies")).toContain(
      'sqlx = { version = "0.8", features = ["sqlite", "runtime-tokio"] }',
    );
    const appDependencies = tomlSection(rootCargo, "dependencies");
    expect(
      appDependencies.match(
        /^extractum-prompt-packs = \{ path = "crates\/extractum-prompt-packs" \}$/gm,
      ) ?? [],
    ).toHaveLength(1);
    expect(appDependencies).toContain("sha2 = { workspace = true }");
    expect(appDependencies).toContain("sqlx = { workspace = true }");
    expect(appDependencies).not.toMatch(/^jsonschema\s*=/m);

    const cratePackages = lockPackages(cargoLock, "extractum-prompt-packs");
    expect(cratePackages).toHaveLength(1);
    expect(cratePackages[0]).not.toMatch(/^source\s*=|^checksum\s*=/m);
    expect(lockDependencies(cratePackages[0])).toEqual(
      sorted([
        "extractum-core",
        "extractum-gemini-browser",
        "extractum-llm",
        "jsonschema",
        "serde",
        "serde_json",
        "sha2",
        "sqlx",
        "tempfile",
        "time",
        "tokio",
        "tokio-util",
      ]),
    );
    const appPackage = lockPackages(cargoLock, "extractum");
    expect(appPackage).toHaveLength(1);
    expect(lockDependencies(appPackage[0]).filter((name) => name === "extractum-prompt-packs")).toEqual([
      "extractum-prompt-packs",
    ]);
    expect(lockDependencies(appPackage[0])).not.toContain("jsonschema");
    for (const lower of ["extractum-core", "extractum-gemini-browser", "extractum-llm"]) {
      const packages = lockPackages(cargoLock, lower);
      expect(packages, lower).toHaveLength(1);
      expect(lockDependencies(packages[0]), lower).not.toContain("extractum-prompt-packs");
      expect(read(`src-tauri/crates/${lower}/Cargo.toml`), lower).not.toContain(
        "extractum-prompt-packs",
      );
    }
    for (const [name, version] of Object.entries({
      jsonschema: "0.46.5",
      sha2: "0.10.9",
      sqlx: "0.8.6",
      tempfile: "3.27.0",
      time: "0.3.47",
      tokio: "1.52.1",
      "tokio-util": "0.7.18",
    })) {
      expect(lockPackages(cargoLock, name).map((block) => block.match(/^version = "([^"]+)"$/m)?.[1]), name).toEqual([
        version,
      ]);
    }
  });

  it("keeps a curated crate API and private explicit app facade", () => {
    const appFacade = read(`${appRoot}/mod.rs`);
    const appLib = read("src-tauri/src/lib.rs");
    expect(appLib).toMatch(/^mod prompt_packs;$/m);
    expect(appLib).not.toContain("extractum_prompt_packs::");

    if (!crateExtracted) {
      const preparedModules = moduleNames(appFacade);
      for (const moduleName of [
        "browser_adapter",
        "event_adapter",
        "library_command",
        "result_commands",
        "runtime_commands",
        "seed_command",
        "source_adapter",
      ]) {
        expect(preparedModules).toContain(moduleName);
      }
      const preparedSource = productionDomainFiles.map(read).join("\n");
      for (const symbol of curatedRootExports) {
        expect(preparedSource, `missing prepared API item ${symbol}`).toMatch(
          new RegExp(`\\b${symbol}\\b`),
        );
      }
      return;
    }

    const crateLib = read(`${crateRoot}/src/lib.rs`);
    expect(moduleNames(crateLib)).toEqual(crateModuleNames);
    expect(crateLib.match(/^#\[cfg\(test\)\]$/gm) ?? []).toHaveLength(1);
    expect(crateLib).toMatch(/#\[cfg\(test\)\]\nmod test_schema;/);
    expect(crateLib).not.toMatch(/^pub(?:\([^)]*\))?\s+mod\s+/m);
    expect(crateLib).not.toMatch(/pub\s+use\s+[^;]*\*/);
    expect(publicReexportNames(crateLib)).toEqual(sorted(curatedRootExports));
    expect(crateLib).not.toMatch(/#\[cfg\(test\)\]\s*pub use/);
    expect(publicReexportNames(crateLib).join("\n")).not.toMatch(
      /\b(?:test_support|test_schema|legacy_disabled_|for_test|Fake\w*)\b/,
    );

    expect(moduleNames(appFacade)).toEqual(
      sorted([
        "browser_adapter",
        "event_adapter",
        "library_command",
        "result_commands",
        "runtime_commands",
        "seed_command",
        "source_adapter",
        "youtube_summary",
      ]),
    );
    expect(appFacade).toMatch(/#\[cfg\(test\)\]\nmod youtube_summary;/);
    expect(appFacade).not.toMatch(/^pub(?:\([^)]*\))?\s+mod\s+/m);
    expect(appFacade).not.toMatch(/pub\s+use\s+[^;]*\*/);
    expect(publicReexportNames(appFacade)).toEqual(
      sorted([
        "PromptPackRunState",
        "cancel_prompt_pack_run",
        "cleanup_interrupted_prompt_pack_runs",
        "clear_prompt_pack_cancellation_smoke_fixture",
        "delete_prompt_pack_run",
        "get_prompt_pack_library",
        "get_prompt_pack_result",
        "get_prompt_pack_stage_artifact",
        "get_prompt_pack_validation_findings",
        "list_active_prompt_pack_runs",
        "list_prompt_pack_audit_events",
        "list_prompt_pack_run_stages",
        "list_prompt_pack_runs",
        "list_prompt_pack_stage_artifacts",
        "preflight_youtube_summary_run",
        "seed_builtin_prompt_packs",
        "seed_prompt_pack_cancellation_smoke_fixture",
        "start_youtube_summary_run",
        "update_prompt_pack_run",
      ]),
    );
    expect(read(`${appRoot}/youtube_summary/mod.rs`)).toBe(
      "#[cfg(test)]\nmod snapshots_tests;\n#[cfg(test)]\nmod test_support;\n",
    );
  });

  it("moves every frozen baseline identity to its approved 223/2 owner exactly once", () => {
    const identities = frozenTests.map((test) => `${test.logicalFile}::${test.name}`);
    expect(new Set(identities).size).toBe(225);
    expect(frozenTests.filter((test) => test.owner === "crate")).toHaveLength(223);
    expect(frozenTests.filter((test) => test.owner === "app")).toHaveLength(2);
    expect(
      frozenTests
        .filter((test) => test.owner === "app")
        .map((test) => `prompt_packs::${test.logicalFile.replace(/\.rs$/, "").replaceAll("/", "::")}::${test.name}`)
        .sort(),
    ).toEqual(
      [
        "prompt_packs::youtube_summary::snapshots_tests::comment_snapshot_selection_is_deterministic_when_enabled",
        "prompt_packs::youtube_summary::snapshots_tests::transcript_text_for_source_uses_segment_renderer",
      ].sort(),
    );

    const ownerFiles = new Set(frozenTests.map(frozenTestPath));
    const testsByFile = new Map(
      [...ownerFiles].map((file) => [file, rustTestNames(read(file))]),
    );
    for (const test of frozenTests) {
      const file = frozenTestPath(test);
      expect(
        testsByFile.get(file)?.filter((name) => name === test.name),
        `${test.logicalFile}::${test.name} in ${file}`,
      ).toHaveLength(1);
    }

    const trackedRustFiles = crateExtracted
      ? [...finalCrateRustFiles, ...finalAppRustFiles]
      : listFiles(appRoot, ".rs");
    const declarations = trackedRustFiles.flatMap((file) =>
      rustTestNames(read(file)).map((name) => ({ file, name })),
    );
    const frozenLeaves = new Set(frozenTests.map((test) => test.name));
    for (const leaf of frozenLeaves) {
      const expectedPaths = frozenTests
        .filter((test) => test.name === leaf)
        .map(frozenTestPath)
        .sort();
      const actualPaths = declarations
        .filter((test) => test.name === leaf)
        .map((test) => test.file)
        .sort();
      expect(actualPaths, leaf).toEqual(expectedPaths);
    }
    expect(
      frozenTests
        .filter((test) => test.name === "now_string_uses_current_utc_time")
        .map((test) => test.logicalFile)
        .sort(),
    ).toEqual(["runtime.rs", "youtube_summary/facade_tests.rs"]);

    if (!crateExtracted) {
      expect(read(`${appRoot}/youtube_summary/snapshots_tests.rs`)).toContain(
        'include!("domain_snapshots_tests.rs");',
      );
    }
  });

  it("rejects disabled renamed or copied legacy prompt-pack tests", () => {
    const rustFiles = crateExtracted
      ? [...finalCrateRustFiles, ...finalAppRustFiles]
      : listFiles(appRoot, ".rs");
    const source = rustFiles.map(read).join("\n");
    expect(source).not.toMatch(/#\s*\[\s*cfg\s*\(\s*any\s*\(\s*\)\s*\)\s*\]/);
    expect(source).not.toMatch(
      /#\s*\[\s*cfg\s*\(\s*(?:false|never|disabled|legacy_disabled|FALSE)\b/i,
    );
    expect(source).not.toMatch(/\blegacy_disabled_[A-Za-z0-9_]+\b/);
    expect(source).not.toMatch(/^\s*\/\/\s*#\[(?:tokio::)?test\b/m);
    expect(source).not.toMatch(/\/\*[\s\S]*?#\[(?:tokio::)?test\b[\s\S]*?\*\//);
    for (const test of frozenTests) {
      const commentedDefinition = new RegExp(
        `^\\s*//.*\\bfn\\s+${test.name}\\b`,
        "m",
      );
      expect(source, test.name).not.toMatch(commentedDefinition);
    }
  });

  it("keeps production SQL and app-only integrations in their approved owners", () => {
    const actualPromptPackFiles = sorted(listFiles(appRoot, ".rs"));
    if (crateExtracted) {
      expect(actualPromptPackFiles).toEqual(sorted(finalAppRustFiles));
      expect(sorted(listFiles(`${crateRoot}/src`, ".rs"))).toEqual(sorted(finalCrateRustFiles));
    } else {
      expect(actualPromptPackFiles).toEqual(preparedRustFiles);
    }

    if (crateExtracted) {
      const forbiddenProduction = [
        /\btauri::|#\[tauri/,
        /\bAppHandle\b/,
        /\bState\s*</,
        /\bEmitter\b/,
        /\bManager\b/,
        /\bget_pool\b/,
        /crate::(?:db|migrations|sources|secret_store|diagnostics|analysis)/,
        /(?:super|crate)::(?:source_adapter|browser_adapter|event_adapter|runtime_commands|result_commands|library_command|seed_command)/,
      ];
      for (const pattern of forbiddenProduction) {
        expect(productionDomainSource, pattern.source).not.toMatch(pattern);
      }

      const sqlTables = [
        ...productionDomainSource.matchAll(
          /\b(?:DO\s+UPDATE\s+SET|(?:FROM|JOIN|INSERT\s+INTO|UPDATE|DELETE\s+FROM|REFERENCES)\s+([A-Za-z_][A-Za-z0-9_]*))/g,
        ),
      ].flatMap((match) => (match[1] ? [match[1].toLowerCase()] : []));
      const allowedTables = new Set<string>(promptPackTables);
      for (const table of new Set(sqlTables)) {
        expect(allowedTables.has(table), `unapproved production SQL table ${table}`).toBe(true);
      }
      for (const foreignTable of foreignTables) {
        expect(sqlTables, foreignTable).not.toContain(foreignTable);
      }
    }

    const runtimeCommands = read(`${appRoot}/runtime_commands.rs`);
    const sourceAdapter = read(`${appRoot}/source_adapter.rs`);
    const browserAdapter = read(`${appRoot}/browser_adapter.rs`);
    const eventAdapter = read(`${appRoot}/event_adapter.rs`);
    expect(runtimeCommands).toMatch(/#\[tauri::command\]/);
    expect(runtimeCommands).toContain("crate::db::get_pool");
    expect(runtimeCommands).toContain("tauri::async_runtime::spawn");
    expect(runtimeCommands).toContain("resolve_profile_for_backend");
    for (const table of foreignTables.slice(0, 5)) {
      expect(sourceAdapter, table).toMatch(
        new RegExp(`\\b(?:FROM|JOIN)\\s+${table}\\b`, "i"),
      );
    }
    expect(browserAdapter).toMatch(/provider_status|send_single_prompt|cancel_gemini_browser_job/);
    expect(eventAdapter).toContain('PROMPT_PACK_RUN_EVENT: &str = "prompt-pack-run-event"');
    expect(eventAdapter).toMatch(/tauri::\{AppHandle, Emitter\}/);
  });

  it("pins the source browser event and execution-ticket handoffs", () => {
    const sourcePort = read(domainPath("source_port.rs"));
    const browserPort = read(domainPath("browser_port.rs"));
    const events = read(domainPath("events.rs"));
    const runtime = read(domainPath("runtime.rs"));
    const runtimeCommands = read(`${appRoot}/runtime_commands.rs`);
    const sourceTrait = rustBlock(sourcePort, "pub trait PromptPackSourceReader");
    const browserTrait = rustBlock(browserPort, "pub trait PromptPackBrowserExecutor");
    const eventTrait = rustBlock(events, "pub trait PromptPackEventSink");
    expect([...sourceTrait.matchAll(/^\s*fn\s+(\w+)\s*\(/gm)].map((match) => match[1])).toEqual([
      "load_source",
      "load_video",
      "load_playlist_items",
      "load_transcript_segments",
      "select_comment_candidates",
      "load_comment_body",
    ]);
    expect(sourcePort).toContain(
      "Pin<Box<dyn Future<Output = AppResult<T>> + Send + 'a>>",
    );
    expect([...browserTrait.matchAll(/^\s*fn\s+(\w+)\s*\(/gm)].map((match) => match[1])).toEqual([
      "read_status",
      "submit",
      "cancel",
    ]);
    expect(browserPort).toContain(
      "Pin<Box<dyn Future<Output = AppResult<T>> + Send + 'a>>",
    );
    expect(browserPort).toContain("PromptPackBrowserFuture<'_, GeminiBrowserRunResult>");
    expect(eventTrait).toMatch(/fn emit\(&self, event: PromptPackEvent\);/);
    expect(events).not.toMatch(/Serialize|Deserialize|tauri/);

    const ticket = rustBlock(runtime, "pub struct RunExecutionTicket");
    expect(ticket).toContain("run_id: i64");
    expect(ticket).not.toContain("pub run_id");
    const ticketPrefix = runtime.slice(
      Math.max(0, runtime.indexOf("pub struct RunExecutionTicket") - 120),
      runtime.indexOf("pub struct RunExecutionTicket"),
    );
    expect(ticketPrefix).not.toMatch(/derive\([^)]*(?:Clone|Serialize|Deserialize)/);
    expect(runtime).toMatch(/pub fn run_id\(&self\) -> i64/);
    expect(runtime).toMatch(/prepare_run_execution\([\s\S]*ticket: &RunExecutionTicket/);
    expect(runtime).toMatch(/fail_run_execution\([\s\S]*ticket: &RunExecutionTicket/);
    const publisher = rustBlock(runtime, "async fn emit_prompt_pack_run_event");
    expect(publisher.indexOf("state.apply_event(&event).await")).toBeGreaterThanOrEqual(0);
    expect(publisher.indexOf("events.emit(event)")).toBeGreaterThan(
      publisher.indexOf("state.apply_event(&event).await"),
    );

    const spawn = rustBlock(runtimeCommands, "fn spawn_youtube_summary_execution");
    const taskBuilder = rustBlock(
      runtimeCommands,
      "fn build_youtube_summary_execution_task",
    );
    const spawnIndex = spawn.indexOf("tauri::async_runtime::spawn");
    const asyncIndex = taskBuilder.indexOf("Box::pin(async move");
    const prepareIndex = taskBuilder.indexOf("prepare_run_execution");
    const profileIndex = taskBuilder.indexOf("resolve_profile_for_backend");
    expect(spawnIndex).toBeGreaterThanOrEqual(0);
    expect(spawn.match(/tauri::async_runtime::spawn/g) ?? []).toHaveLength(1);
    expect(spawn).toContain("dispatch_execution_ticket");
    expect(asyncIndex).toBeGreaterThanOrEqual(0);
    expect(prepareIndex).toBeGreaterThan(asyncIndex);
    expect(profileIndex).toBeGreaterThan(prepareIndex);
    expect(taskBuilder).toMatch(/fail_run_execution\([\s\S]*&ticket/);

    const snapshotTests = read(domainPath("youtube_summary/snapshots_tests.rs"));
    for (const characterization of [
      "runnable_start_uses_complete_fresh_source_read_sequence",
      "snapshot_start_source_preserves_repeated_preflight_and_post_insert_fresh_reads",
      "comment_snapshot_source_reads_candidates_for_estimates_then_selected_bodies_again",
    ]) {
      expect(rustTestNames(snapshotTests), characterization).toContain(characterization);
    }
  });

  it("centralizes all eight canonical bundled assets", () => {
    const attributes = read(".gitattributes");
    for (const line of [
      "src-tauri/prompt-packs/youtube_summary/1.0.0/pack.json text eol=crlf",
      "src-tauri/prompt-packs/youtube_summary/1.0.0/runtime/synthesis.json text eol=crlf",
      "src-tauri/prompt-packs/youtube_summary/1.0.0/runtime/transcript_analysis.json text eol=lf",
      "src-tauri/prompt-packs/youtube_summary/1.0.0/schemas/canonical-result.json -text whitespace=cr-at-eol",
      "src-tauri/prompt-packs/youtube_summary/1.0.0/schemas/stage-io-youtube-summary-synthesis-output.json text eol=lf",
      "src-tauri/prompt-packs/youtube_summary/1.0.0/schemas/stage-io-youtube-summary-transcript-analysis-input.json text eol=crlf",
      "src-tauri/prompt-packs/youtube_summary/1.0.0/schemas/stage-io-youtube-summary-transcript-analysis-output.json text eol=crlf",
      "src-tauri/prompt-packs/youtube_summary/1.0.0/stages/transcript_analysis.json text eol=crlf",
    ]) {
      expect(attributes).toContain(line);
    }

    const assetsPath = domainPath("assets.rs");
    const assets = read(assetsPath);
    const expectedPrefix = crateExtracted
      ? "/../../prompt-packs/youtube_summary/1.0.0/"
      : "/prompt-packs/youtube_summary/1.0.0/";
    const parsedAssets = [
      ...assets.matchAll(
        /pub\(crate\) const ([A-Z][A-Z0-9_]+): &str = include_str!\(concat!\(\s*env!\("CARGO_MANIFEST_DIR"\),\s*"([^"]+)"\s*\)\);/g,
      ),
    ].map((match) => [match[1], match[2]] as const);
    expect(parsedAssets).toHaveLength(8);
    expect(
      Object.fromEntries(parsedAssets),
    ).toEqual(
      Object.fromEntries(
        Object.entries(assetContract).map(([constantName, [relativePath]]) => [
          constantName,
          `${expectedPrefix}${relativePath}`,
        ]),
      ),
    );
    expect(assets).toContain(
      'BUNDLED_SOURCE_PATH: &str = "src-tauri/prompt-packs/youtube_summary/1.0.0"',
    );

    const allDomainFiles = crateExtracted
      ? listFiles(`${crateRoot}/src`, ".rs")
      : [
          ...crateRootFiles
            .filter((file) => file !== "lib.rs")
            .map((file) => `${appRoot}/${file}`),
          ...crateYoutubeFiles.map((file) =>
            file === "snapshots_tests.rs"
              ? `${appRoot}/youtube_summary/domain_snapshots_tests.rs`
              : `${appRoot}/youtube_summary/${file}`,
          ),
        ];
    const centralizedInclude =
      /include_str!\(concat!\(\s*env!\("CARGO_MANIFEST_DIR"\),\s*"[^\"]*prompt-packs\/youtube_summary\/1\.0\.0\//;
    const assetIncludeOwners = allDomainFiles.filter((file) =>
      centralizedInclude.test(read(file)),
    );
    expect(assetIncludeOwners).toEqual([assetsPath]);

    for (const [relativePath, expectedHash] of Object.values(assetContract)) {
      const absolute = path.join(
        repositoryRoot,
        "src-tauri/prompt-packs/youtube_summary/1.0.0",
        relativePath,
      );
      expect(existsSync(absolute), relativePath).toBe(true);
      expect(createHash("sha384").update(readFileSync(absolute)).digest("hex"), relativePath).toBe(
        expectedHash,
      );
    }
  });

  it("keeps the crate-private schema fixture in exact ordered parity with the registered non-Apalis migration prefix", () => {
    const migrationsPath = "src-tauri/src/migrations.rs";
    const fixturePath = domainPath("test_schema.rs");
    const migrations = read(migrationsPath);
    const fixture = read(fixturePath);
    const registryIncludes = parseMigrationRegistry(migrations);
    const fixturePairs = parseFixtureMigrations(fixture);
    expect(registryIncludes).toHaveLength(12);
    expect(new Set(registryIncludes).size).toBe(12);
    expect(fixturePairs).toHaveLength(12);
    expect(new Set(fixturePairs.map((pair) => pair.repositoryPath)).size).toBe(12);

    const registryPaths = registryIncludes.map((includePath) =>
      toRepositoryPath(
        path.resolve(path.dirname(path.join(repositoryRoot, migrationsPath)), includePath),
      ),
    );
    const fixtureIncludePaths = fixturePairs.map((pair) =>
      toRepositoryPath(
        path.resolve(path.dirname(path.join(repositoryRoot, fixturePath)), pair.includePath),
      ),
    );
    expect(fixturePairs.map((pair) => pair.repositoryPath)).toEqual(registryPaths);
    expect(fixtureIncludePaths).toEqual(registryPaths);
    for (const migrationPath of registryPaths) {
      expect(migrationPath).toMatch(/^src-tauri\/migrations\/00(?:0[1-9]|1[0-2])_/);
      expect(migrationPath).not.toContain("apalis");
      expect(existsSync(path.join(repositoryRoot, migrationPath)), migrationPath).toBe(true);
    }
    expect(fixture).toContain("sqlx::raw_sql");
    expect(fixture).toMatch(/pool\.begin\(\)\.await/);
    expect(fixture).toMatch(/transaction\s*\.commit\(\)\s*\.await/);
    expect(fixture).not.toMatch(/_sqlx_migrations|crate::migrations|\btauri::|#\[tauri/);
    expect(fixture).toContain("canonical_fixture_applies_declared_consumed_schema");
    expect(fixture).toContain(
      "canonical_fixture_preserves_consumed_indexes_and_foreign_keys",
    );
    const consumedTables = [
      ...fixture.matchAll(/^\s*\("?(prompt_pack(?:s|_[a-z0-9_]+))"?,\s*(?:&\[|\[)/gm),
    ].map((match) => match[1]);
    for (const table of promptPackTables) {
      expect(fixture, table).toContain(`"${table}"`);
    }
    expect(new Set(consumedTables).size).toBeLessThanOrEqual(promptPackTables.length);
  });
});
