import { createHash } from "node:crypto";
import { existsSync, readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";

import {
  isAnalysisCrateExtracted,
  normalizeAnalysisContractSourceText,
  readAnalysisContractSource,
  readAppAnalysisSource,
  readCrateAnalysisSource,
} from "./analysis-contract-paths";

const repoRoot = path.resolve(import.meta.dirname, "../..");
const specification = readFileSync(
  path.join(
    repoRoot,
    "docs/superpowers/specs/2026-07-22-analysis-crate-boundary-design.md",
  ),
  "utf8",
).replace(/\r\n/g, "\n");
const appLib = readFileSync(path.join(repoRoot, "src-tauri/src/lib.rs"), "utf8");
const errorSource = readFileSync(
  path.join(repoRoot, "src-tauri/crates/extractum-core/src/error.rs"),
  "utf8",
);
const portableChatSource = () =>
  readAnalysisContractSource({
    before: "chat_engine.rs",
    after: { owner: "crate", path: "chat.rs" },
  });
const portableReportSource = () =>
  readAnalysisContractSource({
    before: "report_engine.rs",
    after: { owner: "crate", path: "report.rs" },
  });
const portableReportLifecycleSource = () =>
  readAnalysisContractSource({
    before: "report/lifecycle_portable.rs",
    after: { owner: "crate", path: "report/lifecycle.rs" },
  });

const analysisRelease = [
  "list_analysis_sources",
  "list_analysis_runs",
  "list_active_analysis_runs",
  "get_analysis_run",
  "list_analysis_run_messages",
  "get_analysis_run_trace",
  "delete_analysis_run",
  "resolve_analysis_trace_refs",
  "list_analysis_prompt_templates",
  "create_analysis_prompt_template",
  "update_analysis_prompt_template",
  "delete_analysis_prompt_template",
  "list_analysis_source_groups",
  "create_analysis_source_group",
  "update_analysis_source_group",
  "delete_analysis_source_group",
  "list_analysis_chat_messages",
  "clear_analysis_chat_messages",
  "ask_analysis_run_question",
  "start_analysis_report",
  "cancel_analysis_run",
] as const;
const projectRelease = [
  "start_project_analysis",
  "list_project_runs",
  "get_project_data_range",
] as const;
const devCommands = [
  "seed_analysis_redesign_fixtures",
  "clear_analysis_redesign_fixtures",
  "clear_analysis_redesign_fixture_active_runs",
] as const;

const commandWireContracts = {
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
  start_project_analysis: ["handle: AppHandle, state: tauri::State<'_, AnalysisState>, project_id: i64, period_from: i64, period_to: i64, output_language: String, prompt_template_id: i64, model_override: Option<String>, profile_id: Option<String>, youtube_corpus_mode: Option<String>, include_migrated_history: bool -> AppResult<i64>", ["projectId", "periodFrom", "periodTo", "outputLanguage", "promptTemplateId", "modelOverride", "profileId", "youtubeCorpusMode", "includeMigratedHistory"]],
  list_project_runs: ["handle: AppHandle, project_id: i64 -> AppResult<Vec<AnalysisRunSummary>>", ["projectId"]],
  get_project_data_range: ["handle: AppHandle, project_id: i64, youtube_corpus_mode: Option<String>, include_migrated_history: bool -> AppResult<ProjectDataRange>", ["projectId", "youtubeCorpusMode", "includeMigratedHistory"]],
  seed_analysis_redesign_fixtures: ["handle: AppHandle, state: State<'_, AnalysisState> -> AppResult<AnalysisRedesignFixtureSummary>", []],
  clear_analysis_redesign_fixtures: ["handle: AppHandle, state: State<'_, AnalysisState> -> AppResult<AnalysisRedesignFixtureSummary>", []],
  clear_analysis_redesign_fixture_active_runs: ["handle: AppHandle, state: State<'_, AnalysisState> -> AppResult<()>", []],
} as const;

function rustFiles(root: string): string[] {
  return readdirSync(root, { withFileTypes: true }).flatMap((entry) => {
    const selected = path.join(root, entry.name);
    if (entry.isDirectory()) return rustFiles(selected);
    return entry.isFile() && entry.name.endsWith(".rs") ? [selected] : [];
  });
}

const currentAnalysisRoot = path.join(repoRoot, "src-tauri/src/analysis");
const analysisCrateRoot = path.join(
  repoRoot,
  "src-tauri/crates/extractum-analysis/src",
);
const analysisExtracted = isAnalysisCrateExtracted();

type ExecutableAnalysisIdentity = {
  identity: string;
  owner: "app" | "crate";
};

function sourceIdentityPrefix(
  root: string,
  owner: "app" | "crate",
  file: string,
): string {
  let relative = path
    .relative(root, file)
    .replaceAll("\\", "/");
  if (owner === "app" && !analysisExtracted) {
    relative = relative
      .replace(/^(chat|report)_engine\.rs$/, "$1.rs")
      .replace(/^report\/lifecycle_portable\.rs$/, "report/lifecycle.rs")
      .replace(
        /\/(live|preflight|source_resolution|read_model|setup)_portable\.rs$/,
        "/$1.rs",
      );
  }
  relative = relative.replace(/\.rs$/, "");
  const parts = relative.replace(/\.rs$/, "").split("/");
  if (parts.at(-1) === "mod") parts.pop();
  if (owner === "app") {
    if (relative === "tests_portable") return "analysis::tests";
    if (relative === "tests_application") return "analysis::tests_application";
    if (parts.includes("tests")) return ["analysis", ...parts].join("::");
    return ["analysis", ...parts, "tests"].join("::");
  }
  if (relative === "tests") return "tests";
  if (parts.includes("tests")) return parts.join("::");
  return [...parts, "tests"].join("::");
}

function executableAnalysisIdentities(): ExecutableAnalysisIdentity[] {
  const roots: Array<{ owner: "app" | "crate"; root: string }> = [
    { owner: "app", root: currentAnalysisRoot },
    ...(analysisExtracted
      ? [{ owner: "crate" as const, root: analysisCrateRoot }]
      : []),
  ];
  return roots
    .flatMap(({ owner, root }) => rustFiles(root).flatMap((file) => {
      const source = readFileSync(file, "utf8");
      const prefix = sourceIdentityPrefix(root, owner, file);
      return [
        ...source.matchAll(
          /^\s*#\[(?:(?:tokio|sqlx)::)?test(?:\s*\([^\]]*\))?\]\s*(?:#\[[^\n]+\]\s*)*(?:async\s+)?fn\s+([A-Za-z0-9_]+)\s*\(/gm,
        ),
      ].map((match) => ({
        identity: `${prefix}::${match[1]}`,
        owner,
      }));
    }))
    .sort((left, right) => left.identity.localeCompare(right.identity));
}

const planAddedTests = [
  {
    current:
      "analysis::chat::tests::chat_persistence_failure_keeps_completed_answer_failure_message",
    final:
      "chat::tests::chat_persistence_failure_keeps_completed_answer_failure_message",
    owner: "crate" as const,
  },
  {
    current:
      "analysis::chat::tests::chat_execution_persists_turns_before_completed_event",
    final: "chat::tests::chat_execution_persists_turns_before_completed_event",
    owner: "crate" as const,
  },
  {
    current:
      "analysis::report::tests::corpus_port::report_execution_uses_distinct_preflight_and_capture_corpus_reads",
    final:
      "report::tests::corpus_port::report_execution_uses_distinct_preflight_and_capture_corpus_reads",
    owner: "crate" as const,
  },
  {
    current:
      "analysis::report::tests::corpus_port::started_load_items_uses_preflight_summary_before_empty_capture_failure",
    final:
      "report::tests::corpus_port::started_load_items_uses_preflight_summary_before_empty_capture_failure",
    owner: "crate" as const,
  },
  {
    current:
      "analysis::report::tests::corpus_port::started_load_items_uses_preflight_summary_before_error_capture_failure",
    final:
      "report::tests::corpus_port::started_load_items_uses_preflight_summary_before_error_capture_failure",
    owner: "crate" as const,
  },
  {
    current:
      "analysis::report::tests::lifecycle::terminal_cleanup_removes_active_state_when_terminal_persistence_fails",
    final:
      "report::tests::lifecycle::terminal_cleanup_removes_active_state_when_terminal_persistence_fails",
    owner: "crate" as const,
  },
  {
    current:
      "analysis::report::tests::runtime::report_execution_publishes_typed_events_in_existing_order",
    final:
      "report::tests::runtime::report_execution_publishes_typed_events_in_existing_order",
    owner: "crate" as const,
  },
  {
    current:
      "analysis::report::tests::runtime::terminal_cleanup_always_removes_active_report_state",
    final:
      "report::tests::runtime::terminal_cleanup_always_removes_active_report_state",
    owner: "crate" as const,
  },
  {
    current:
      "analysis::report::tests::scope::start_analysis_report_request_constructors_preserve_source_group_and_project_scopes",
    final:
      "report::tests::scope::start_analysis_report_request_constructors_preserve_source_group_and_project_scopes",
    owner: "crate" as const,
  },
  {
    current:
      "analysis::report::tests::scope::resolved_analysis_scope_rejects_zero_or_multiple_identities",
    final:
      "report::tests::scope::resolved_analysis_scope_rejects_zero_or_multiple_identities",
    owner: "crate" as const,
  },
  {
    current:
      "analysis::report::tests::scope::resolved_analysis_scope_requires_nonempty_stable_sources_and_label",
    final:
      "report::tests::scope::resolved_analysis_scope_requires_nonempty_stable_sources_and_label",
    owner: "crate" as const,
  },
  {
    current:
      "analysis::store::tests::read_model::analysis_run_list_filter_constructors_preserve_analysis_and_project_scopes",
    final:
      "store::tests::read_model::analysis_run_list_filter_constructors_preserve_analysis_and_project_scopes",
    owner: "crate" as const,
  },
  {
    current:
      "analysis::test_schema::tests::canonical_fixture_applies_analysis_consumed_schema",
    final:
      "test_schema::tests::canonical_fixture_applies_analysis_consumed_schema",
    owner: "crate" as const,
  },
  {
    current:
      "analysis::test_schema::tests::canonical_fixture_preserves_analysis_owned_indexes_and_foreign_keys",
    final:
      "test_schema::tests::canonical_fixture_preserves_analysis_owned_indexes_and_foreign_keys",
    owner: "crate" as const,
  },
  {
    current:
      "analysis::trace::tests::legacy_trace_bytes_decode_after_core_compression_handoff",
    final:
      "trace::tests::legacy_trace_bytes_decode_after_core_compression_handoff",
    owner: "crate" as const,
  },
  {
    current:
      "analysis::trace::tests::decode_trace_data_returns_typed_internal_for_invalid_json",
    final:
      "trace::tests::decode_trace_data_returns_typed_internal_for_invalid_json",
    owner: "crate" as const,
  },
  {
    current:
      "analysis::trace::tests::trace_ref_json_is_byte_compatible_for_telegram_and_youtube",
    final:
      "trace::tests::trace_ref_json_is_byte_compatible_for_telegram_and_youtube",
    owner: "crate" as const,
  },
  {
    current:
      "analysis::corpus::source_resolution::tests::source_group_resolution_orders_members_by_title_then_id_before_playlist_expansion",
    final:
      "analysis::corpus::source_resolution::tests::source_group_resolution_orders_members_by_title_then_id_before_playlist_expansion",
    owner: "app" as const,
  },
  {
    current:
      "analysis::groups::tests::prepare_analysis_source_group_input_preserves_baseline_error_precedence",
    final:
      "analysis::groups::tests::prepare_analysis_source_group_input_preserves_baseline_error_precedence",
    owner: "app" as const,
  },
  ...[
    "run_reads_preserve_deleted_blank_and_snapshot_scope_labels",
    "analysis_run_search_escapes_percent_underscore_and_backslash_before_limit",
    "chat_legacy_label_fallback_rereads_run_on_the_foreign_label_snapshot",
    "analysis_wire_values_serialize_to_exact_json_objects",
    "chat_profile_resolution_failure_is_async_after_request_id",
    "report_start_preserves_acceptance_order_and_two_corpus_reads",
    "report_profile_resolution_failure_prevents_run_creation",
  ].map((name) => ({
    current: `analysis::tests_application::${name}`,
    final: `analysis::tests_application::${name}`,
    owner: "app" as const,
  })),
] as const;

type FrozenIdentity = {
  current: string;
  final: string;
  owner: "app" | "crate";
};

function parseAppendix(): FrozenIdentity[] {
  const marker = "## Appendix A: Frozen 143-Test Baseline";
  const start = specification.indexOf(marker);
  if (start < 0) throw new Error("missing Appendix A heading");
  const appendix = specification.slice(start);
  const ownerHeadings = [
    { marker: "### `extractum-analysis` — 95 identities", owner: "crate" as const, count: 95 },
    { marker: "### `extractum` — 48 identities", owner: "app" as const, count: 48 },
  ];
  const ownerStarts = ownerHeadings.map(({ marker: heading }) => {
    const index = appendix.indexOf(heading);
    if (index < 0) throw new Error(`missing Appendix A owner heading: ${heading}`);
    return index;
  });
  if (ownerStarts[0] >= ownerStarts[1]) throw new Error("Appendix A owner headings are out of order");

  const identities: FrozenIdentity[] = [];
  for (let ownerIndex = 0; ownerIndex < ownerHeadings.length; ownerIndex += 1) {
    const owner = ownerHeadings[ownerIndex];
    const section = appendix.slice(ownerStarts[ownerIndex], ownerStarts[ownerIndex + 1]);
    const headings = [...section.matchAll(/^#### .+ \((\d+)\)$/gm)];
    if (headings.length === 0) throw new Error(`Appendix A ${owner.owner} has no identity groups`);
    const ownerIdentities: FrozenIdentity[] = [];
    for (let index = 0; index < headings.length; index += 1) {
      const heading = headings[index];
      if (heading.index === undefined) throw new Error("Appendix A group heading has no index");
      const body = section.slice(
        heading.index + heading[0].length,
        headings[index + 1]?.index ?? section.length,
      );
      const prefixMatch = body.match(
        /^Current (?:and final )?prefix:\s*`([^`]+)`(?:\. Final prefix:\s*\n?`([^`]+)`)?\.$/m,
      );
      if (!prefixMatch) throw new Error(`missing or malformed prefixes after ${heading[0]}`);
      const currentPrefix = prefixMatch[1];
      const finalPrefix = prefixMatch[2] ?? currentPrefix;
      if (!currentPrefix.startsWith("analysis::")) {
        throw new Error(`unexpected current prefix: ${currentPrefix}`);
      }
      if (owner.owner === "crate" && finalPrefix.startsWith("analysis::")) {
        throw new Error(`unexpected crate final prefix: ${finalPrefix}`);
      }
      if (owner.owner === "app" && finalPrefix !== currentPrefix) {
        throw new Error(`unexpected app final prefix: ${finalPrefix}`);
      }
      const bulletLines = body.split("\n").filter((line) => line.startsWith("- "));
      const names = bulletLines.map((line) => {
        const match = line.match(/^- `([A-Za-z0-9_]+)`$/);
        if (!match) throw new Error(`malformed Appendix A bullet: ${line}`);
        return match[1];
      });
      if (names.length !== Number(heading[1])) {
        throw new Error(`${heading[0]} declares ${heading[1]} identities but has ${names.length}`);
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
      throw new Error(`Appendix A ${owner.owner} count drift: ${ownerIdentities.length}`);
    }
    identities.push(...ownerIdentities);
  }
  const fullNames = identities.map(({ current }) => current);
  if (new Set(fullNames).size !== fullNames.length) {
    throw new Error("Appendix A contains duplicate full current identities");
  }
  return identities;
}

function handlerBody(): string {
  const marker = "tauri::generate_handler![";
  const start = appLib.indexOf(marker);
  if (start < 0) throw new Error("missing generate_handler! registration");
  let depth = 0;
  for (let index = start + marker.length - 1; index < appLib.length; index += 1) {
    if (appLib[index] === "[") depth += 1;
    if (appLib[index] === "]") depth -= 1;
    if (depth === 0) return appLib.slice(start + marker.length, index);
  }
  throw new Error("unclosed generate_handler! registration");
}

function closingDelimiter(source: string, open: number, left: string, right: string): number {
  let depth = 0;
  for (let index = open; index < source.length; index += 1) {
    if (source[index] === left) depth += 1;
    if (source[index] === right) depth -= 1;
    if (depth === 0) return index;
  }
  throw new Error(`unclosed ${left}${right} delimiter`);
}

function uniqueRustBracedBody(
  source: string,
  opening: RegExp,
  label: string,
): string {
  const flags = opening.flags.includes("g")
    ? opening.flags
    : `${opening.flags}g`;
  const matches = [...source.matchAll(new RegExp(opening.source, flags))];
  expect(matches, `${label} count`).toHaveLength(1);
  const match = matches[0];
  if (!match || match.index === undefined) {
    throw new Error(`missing ${label}`);
  }
  const braceOffset = match[0].lastIndexOf("{");
  if (braceOffset < 0) {
    throw new Error(`missing ${label} opening brace`);
  }
  const open = match.index + braceOffset;
  return source.slice(open + 1, closingDelimiter(source, open, "{", "}"));
}

function splitTopLevel(source: string): string[] {
  const parts: string[] = [];
  let start = 0;
  let depth = 0;
  for (let index = 0; index < source.length; index += 1) {
    if ("(<[{".includes(source[index])) depth += 1;
    if (")>]}".includes(source[index])) depth -= 1;
    if (source[index] === "," && depth === 0) {
      const part = source.slice(start, index).trim();
      if (part) parts.push(part.replace(/\s+/g, " "));
      start = index + 1;
    }
  }
  const tail = source.slice(start).trim();
  if (tail) parts.push(tail.replace(/\s+/g, " "));
  return parts;
}

function commandSignature(source: string, name: string): { signature: string; params: string[] } {
  const marker = `pub async fn ${name}`;
  const start = source.indexOf(marker);
  if (start < 0) throw new Error(`missing command declaration: ${name}`);
  const open = source.indexOf("(", start + marker.length);
  const close = closingDelimiter(source, open, "(", ")");
  const params = splitTopLevel(source.slice(open + 1, close));
  const body = source.indexOf("{", close);
  const returns = source.slice(close + 1, body).trim().replace(/^->\s*/, "").replace(/\s+/g, " ");
  return { signature: `${params.join(", ")} -> ${returns}`, params };
}

function canonicalWireSignature(signature: string): string {
  return signature
    .replace(/\bcrate::analysis::models::/g, "")
    .replace(/\bcrate::analysis::/g, "")
    .replace(/\bextractum_analysis::/g, "");
}

function camelCase(value: string): string {
  return value.replace(/_([a-z])/g, (_, letter: string) => letter.toUpperCase());
}

function expectOrdered(source: string, markers: readonly string[]): void {
  let offset = 0;
  for (const marker of markers) {
    const found = source.indexOf(marker, offset);
    expect(found, marker).toBeGreaterThanOrEqual(offset);
    offset = found + marker.length;
  }
}

function normalized(source: string): string {
  return source.replace(/\s+/g, " ").trim();
}

function literalOffsets(source: string, marker: string): number[] {
  const offsets: number[] = [];
  for (let start = 0;;) {
    const offset = source.indexOf(marker, start);
    if (offset < 0) return offsets;
    offsets.push(offset);
    start = offset + marker.length;
  }
}

type AnalysisSourceFile = { relative: string; source: string };
type FrozenMove = { before: string; after: string };

const rustCommentMaskCache = new Map<string, {
  strings?: string;
  comments?: string;
}>();

function maskRustComments(source: string, maskStrings: boolean): string {
  const cached = rustCommentMaskCache.get(source);
  const cachedValue = maskStrings ? cached?.strings : cached?.comments;
  if (cachedValue !== undefined) return cachedValue;
  const masked = source.split("");
  const blank = (start: number, end: number): void => {
    for (let index = start; index < end; index += 1) if (masked[index] !== "\n") masked[index] = " ";
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
      const end = source.indexOf("\n", index);
      blank(index, end < 0 ? source.length : end);
      index = end < 0 ? source.length : end;
    } else if (source.startsWith("/*", index)) {
      let depth = 1;
      let end = index + 2;
      while (end < source.length && depth > 0) {
        if (source.startsWith("/*", end)) { depth += 1; end += 2; }
        else if (source.startsWith("*/", end)) { depth -= 1; end += 2; }
        else end += 1;
      }
      blank(index, end);
      index = end;
    } else {
      const charLiteral = source[index] === "'" && /^'(?:\\.|[^\\'\n])'/.test(source.slice(index));
      if (source[index] === '"' || charLiteral) {
        const end = quotedEnd(index, source[index]);
        if (maskStrings) blank(index, end);
        index = end;
      } else {
        const raw = source.slice(index).match(/^r(#+)?"/);
        if (raw) {
          const terminator = `"${raw[1] ?? ""}`;
          const end = source.indexOf(terminator, index + raw[0].length);
          const after = end < 0 ? source.length : end + terminator.length;
          if (maskStrings) blank(index, after);
          index = after;
        } else index += 1;
      }
    }
  }
  const result = masked.join("");
  rustCommentMaskCache.set(source, {
    ...cached,
    [maskStrings ? "strings" : "comments"]: result,
  });
  return result;
}

function productionRust(source: string): string {
  const masked = source.split("");
  const blank = (start: number, end: number): void => {
    for (let index = start; index < end; index += 1) if (masked[index] !== "\n") masked[index] = " ";
  };
  let syntax = maskRustComments(source, true);
  const cfg = /#\s*\[\s*cfg\s*\(\s*(?:test|dev)\s*\)\s*\]/g;
  for (;;) {
    const match = cfg.exec(syntax);
    if (!match || match.index === undefined) break;
    const brace = syntax.indexOf("{", cfg.lastIndex);
    const semicolon = syntax.indexOf(";", cfg.lastIndex);
    const comma = syntax.indexOf(",", cfg.lastIndex);
    const header = syntax.slice(cfg.lastIndex, brace < 0 ? syntax.length : brace);
    const isBracedItem = brace >= 0
      && (semicolon < 0 || brace < semicolon)
      && /^\s*(?:(?:pub(?:\([^)]*\))?|async|unsafe|extern(?:\s+"[^"]*")?)\s+)*(?:fn|mod|impl|trait|struct|enum|union)\b/.test(header);
    const isUseItem = /^\s*use\b/.test(syntax.slice(cfg.lastIndex));
    const terminators = [semicolon, comma, brace].filter((index) => index >= 0);
    const end = isBracedItem
      ? closingDelimiter(syntax, brace, "{", "}") + 1
      : isUseItem && semicolon >= 0 ? semicolon + 1 : Math.min(...terminators) + 1;
    if (end <= match.index) throw new Error("unterminated cfg(test/dev) Rust item");
    blank(match.index, end);
    syntax = maskRustComments(masked.join(""), true);
    cfg.lastIndex = 0;
  }
  return masked.join("");
}

function analysisSourceFiles(): AnalysisSourceFile[] {
  return rustFiles(currentAnalysisRoot).map((file) => ({
    relative: path.relative(currentAnalysisRoot, file).replaceAll("\\", "/"),
    source: readFileSync(file, "utf8"),
  }));
}

function rustModuleReachability(
  files: readonly AnalysisSourceFile[],
  roots: readonly string[],
): { production: AnalysisSourceFile[]; cfgOnly: AnalysisSourceFile[] } {
  const normalizeRelative = (relative: string): string =>
    path.posix.normalize(relative.replaceAll("\\", "/")).replace(/^\.\//, "");
  const byRelative = new Map(files.map((file) => [
    normalizeRelative(file.relative),
    { ...file, relative: normalizeRelative(file.relative) },
  ]));
  const walk = (production: boolean): Set<string> => {
    const reached = new Set<string>();
    const visited = new Set<string>();
    const queue = roots.map((relative) => {
      const normalized = normalizeRelative(relative);
      return {
        relative: normalized,
        moduleDir: path.posix.dirname(normalized) === "." ? "" : path.posix.dirname(normalized),
      };
    });
    for (let queueIndex = 0; queueIndex < queue.length; queueIndex += 1) {
      const state = queue[queueIndex];
      const key = `${state.relative}\0${state.moduleDir}`;
      if (visited.has(key)) continue;
      visited.add(key);
      const file = byRelative.get(state.relative);
      if (!file) throw new Error(`Rust module graph is missing ${state.relative}`);
      reached.add(state.relative);
      const view = production ? productionRust(file.source) : file.source;
      const syntax = maskRustComments(view, true);
      const literals = decodedRustStringLiterals(file.source);
      const inlineModules = [...syntax.matchAll(
        /\bmod\s+([A-Za-z_][A-Za-z0-9_]*)\s*\{/g,
      )].map((match) => {
        if (match.index === undefined) throw new Error("inline Rust module has no offset");
        const open = match.index + match[0].lastIndexOf("{");
        return {
          name: match[1],
          open,
          close: closingDelimiter(syntax, open, "{", "}"),
        };
      });
      const moduleDirAt = (offset: number): string => {
        const names = inlineModules
          .filter(({ open, close }) => offset > open && offset < close)
          .sort((left, right) => left.open - right.open)
          .map(({ name }) => name);
        return path.posix.join(state.moduleDir, ...names);
      };
      const pathOverrideAt = (offset: number): string | undefined => {
        const boundary = Math.max(
          syntax.lastIndexOf(";", offset - 1),
          syntax.lastIndexOf("{", offset - 1),
          syntax.lastIndexOf("}", offset - 1),
        );
        const regionStart = boundary + 1;
        const region = file.source.slice(regionStart, offset);
        const pathAttrs = [...region.matchAll(
          /#\s*\[\s*path\s*=\s*((?:br|r)(?:#+)?"[\s\S]*?"#*|b?"(?:\\.|[^"\\])*")\s*\]/g,
        )];
        const attr = pathAttrs.at(-1);
        if (!attr || attr.index === undefined) return undefined;
        const absoluteStart = regionStart + attr.index;
        return literals.find(({ tokenStart, tokenEnd }) =>
          tokenStart >= absoluteStart
          && tokenEnd <= absoluteStart + attr[0].length)?.value;
      };
      const enqueue = (relative: string, moduleDir: string): void => {
        const normalized = normalizeRelative(relative);
        if (!byRelative.has(normalized)) {
          throw new Error(`${state.relative} reaches missing Rust source ${normalized}`);
        }
        queue.push({ relative: normalized, moduleDir: normalizeRelative(moduleDir) === "." ? "" : normalizeRelative(moduleDir) });
      };
      for (const declaration of syntax.matchAll(
        /\bmod\s+([A-Za-z_][A-Za-z0-9_]*)\s*;/g,
      )) {
        if (declaration.index === undefined) throw new Error("external Rust module has no offset");
        const name = declaration[1];
        const effectiveDir = moduleDirAt(declaration.index);
        const override = pathOverrideAt(declaration.index);
        if (override) {
          const relative = path.posix.join(path.posix.dirname(state.relative), override);
          const childDir = path.posix.dirname(relative);
          enqueue(relative, childDir);
          continue;
        }
        const candidates = [
          path.posix.join(effectiveDir, `${name}.rs`),
          path.posix.join(effectiveDir, name, "mod.rs"),
        ].filter((candidate) => byRelative.has(normalizeRelative(candidate)));
        if (candidates.length !== 1) {
          throw new Error(
            `${state.relative} mod ${name} resolves to ${candidates.length} Rust sources`,
          );
        }
        enqueue(candidates[0], path.posix.join(effectiveDir, name));
      }
      for (const include of syntax.matchAll(/\binclude\s*!\s*\(/g)) {
        if (include.index === undefined) throw new Error("include! has no offset");
        const open = include.index + include[0].lastIndexOf("(");
        const close = closingDelimiter(syntax, open, "(", ")");
        const literal = exactLiteralExpressionProgram(
          file.source,
          open + 1,
          close,
          literals,
          concatLiteralPrograms(file.source, literals),
        );
        if (!literal) throw new Error(`${state.relative} has a nonliteral include! edge`);
        enqueue(
          path.posix.join(path.posix.dirname(state.relative), literal.text),
          moduleDirAt(include.index),
        );
      }
    }
    return reached;
  };
  const production = walk(true);
  const all = walk(false);
  const select = (paths: Iterable<string>): AnalysisSourceFile[] =>
    [...paths].sort().map((relative) => byRelative.get(relative)!);
  return {
    production: select(production),
    cfgOnly: select([...all].filter((relative) => !production.has(relative))),
  };
}

const retainedAnalysisFiles = [
  "chat.rs",
  "corpus.rs",
  "corpus/live.rs",
  "corpus/source_resolution.rs",
  "corpus/tests/harness.rs",
  "corpus/tests/live.rs",
  "corpus/tests/mod.rs",
  "corpus/tests/preflight.rs",
  "corpus/tests/source_resolution.rs",
  "events.rs",
  "fixtures.rs",
  "fixtures/seed.rs",
  "fixtures/seed/runs.rs",
  "fixtures/tests/active_runs.rs",
  "fixtures/tests/clear.rs",
  "fixtures/tests/harness.rs",
  "fixtures/tests/mod.rs",
  "fixtures/tests/seed.rs",
  "fixtures/tests/snapshot.rs",
  "fixtures/tests/summary.rs",
  "groups.rs",
  "mod.rs",
  "report.rs",
  "report/lifecycle.rs",
  "report/tests/capture.rs",
  "report/tests/mod.rs",
  "report_commands.rs",
  "store.rs",
  "store/read_model.rs",
  "store/setup.rs",
  "store/tests/mod.rs",
  "store/tests/read_model.rs",
  "store/tests/setup.rs",
  "templates.rs",
  "tests_application.rs",
] as const;

const expectedCfgOnlyAppRustFiles = [
  "analysis/corpus/tests/harness.rs",
  "analysis/corpus/tests/live.rs",
  "analysis/corpus/tests/live_portable.rs",
  "analysis/corpus/tests/mod.rs",
  "analysis/corpus/tests/preflight.rs",
  "analysis/corpus/tests/preflight_portable.rs",
  "analysis/corpus/tests/snapshot.rs",
  "analysis/corpus/tests/source_resolution.rs",
  "analysis/corpus/tests/source_resolution_portable.rs",
  "analysis/fixtures.rs",
  "analysis/fixtures/seed.rs",
  "analysis/fixtures/seed/runs.rs",
  "analysis/fixtures/tests/active_runs.rs",
  "analysis/fixtures/tests/clear.rs",
  "analysis/fixtures/tests/harness.rs",
  "analysis/fixtures/tests/mod.rs",
  "analysis/fixtures/tests/seed.rs",
  "analysis/fixtures/tests/snapshot.rs",
  "analysis/fixtures/tests/summary.rs",
  "analysis/report/tests/architecture.rs",
  "analysis/report/tests/capture.rs",
  "analysis/report/tests/corpus_port.rs",
  "analysis/report/tests/harness.rs",
  "analysis/report/tests/lifecycle.rs",
  "analysis/report/tests/mod.rs",
  "analysis/report/tests/mod_portable.rs",
  "analysis/report/tests/phases.rs",
  "analysis/report/tests/preflight.rs",
  "analysis/report/tests/requests.rs",
  "analysis/report/tests/runtime.rs",
  "analysis/report/tests/scope.rs",
  "analysis/store/tests/harness.rs",
  "analysis/store/tests/mod.rs",
  "analysis/store/tests/read_model.rs",
  "analysis/store/tests/read_model_portable.rs",
  "analysis/store/tests/runs.rs",
  "analysis/store/tests/setup.rs",
  "analysis/store/tests/setup_portable.rs",
  "analysis/store/tests/snapshot.rs",
  "analysis/test_schema.rs",
  "analysis/tests_application.rs",
  "analysis/tests_portable.rs",
  "prompt_packs/youtube_summary/mod.rs",
  "prompt_packs/youtube_summary/snapshots_tests.rs",
  "prompt_packs/youtube_summary/test_support.rs",
  "sources/test_support.rs",
] as const;

const frozenUnreachableStagingTestFiles = [
  "analysis/corpus/tests/harness_portable.rs",
  "analysis/corpus/tests/mod_portable.rs",
  "analysis/store/tests/mod_portable.rs",
] as const;

function inventoryDrift(
  actual: readonly string[],
  expected: readonly string[],
): { duplicates: string[]; missing: string[]; unexpected: string[] } {
  const counts = new Map<string, number>();
  for (const item of actual) counts.set(item, (counts.get(item) ?? 0) + 1);
  const actualSet = new Set(actual);
  const expectedSet = new Set(expected);
  return {
    duplicates: [...counts].filter(([, count]) => count > 1).map(([item]) => item).sort(),
    missing: [...expectedSet].filter((item) => !actualSet.has(item)).sort(),
    unexpected: [...actualSet].filter((item) => !expectedSet.has(item)).sort(),
  };
}

function expectExactInventory(
  actual: readonly string[],
  expected: readonly string[],
  label: string,
): void {
  expect(inventoryDrift(actual, expected), label).toEqual({
    duplicates: [],
    missing: [],
    unexpected: [],
  });
  expect([...actual].sort(), `${label} exact sorted inventory`).toEqual([...expected].sort());
}

function filesWithExtension(root: string, extension: string): string[] {
  return readdirSync(root, { withFileTypes: true }).flatMap((entry) => {
    const selected = path.join(root, entry.name);
    if (entry.isDirectory()) return filesWithExtension(selected, extension);
    return entry.isFile() && entry.name.endsWith(extension) ? [selected] : [];
  });
}

type SqlTableOperation = {
  operation: string;
  table: string;
  offset: number;
  callStart: number;
  callEnd: number;
  consumerStart: number;
  consumerEnd: number;
  sql: string;
  sqlOffset: number;
};

function sqlQueryCallBounds(source: string): Array<{ start: number; end: number }> {
  return sqlxQueryCalls(source).map(({ start, end }) => ({ start, end }));
}

type DecodedRustStringLiteral = {
  tokenStart: number;
  contentStart: number;
  tokenEnd: number;
  value: string;
  offsets: number[];
};

const decodedRustStringCache = new Map<string, DecodedRustStringLiteral[]>();

function decodedRustStringLiterals(source: string): DecodedRustStringLiteral[] {
  const cached = decodedRustStringCache.get(source);
  if (cached) return cached;
  const literals: DecodedRustStringLiteral[] = [];
  const pushDecoded = (
    valueParts: string[],
    offsets: number[],
    value: string,
    sourceOffset: number,
  ): void => {
    valueParts.push(value);
    for (let index = 0; index < value.length; index += 1) offsets.push(sourceOffset);
  };
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
        if (source.startsWith("/*", index)) { depth += 1; index += 2; }
        else if (source.startsWith("*/", index)) { depth -= 1; index += 2; }
        else index += 1;
      }
      continue;
    }
    const raw = source.slice(index).match(/^(?:br|r)(#+)?"/);
    if (raw) {
      const contentStart = index + raw[0].length;
      const terminator = `"${raw[1] ?? ""}`;
      const close = source.indexOf(terminator, contentStart);
      const contentEnd = close < 0 ? source.length : close;
      const value = source.slice(contentStart, contentEnd);
      literals.push({
        tokenStart: index,
        contentStart,
        tokenEnd: close < 0 ? source.length : close + terminator.length,
        value,
        offsets: Array.from({ length: value.length }, (_, charIndex) => contentStart + charIndex),
      });
      index = close < 0 ? source.length : close + terminator.length;
      continue;
    }
    const bytePrefix = source[index] === "b" && source[index + 1] === '"' ? 1 : 0;
    const quote = index + bytePrefix;
    if (source[quote] === '"') {
      const tokenStart = index;
      const contentStart = quote + 1;
      const valueParts: string[] = [];
      const offsets: number[] = [];
      let cursor = contentStart;
      while (cursor < source.length && source[cursor] !== '"') {
        if (source[cursor] !== "\\") {
          pushDecoded(valueParts, offsets, source[cursor], cursor);
          cursor += 1;
          continue;
        }
        const escapeStart = cursor;
        const escape = source[cursor + 1];
        if (escape === "\n" || escape === "\r") {
          cursor += escape === "\r" && source[cursor + 2] === "\n" ? 3 : 2;
          while (cursor < source.length && /[ \t]/.test(source[cursor])) cursor += 1;
          continue;
        }
        const simple: Record<string, string> = {
          "0": "\0",
          "n": "\n",
          "r": "\r",
          "t": "\t",
          "\\": "\\",
          "\"": "\"",
          "'": "'",
        };
        if (escape in simple) {
          pushDecoded(valueParts, offsets, simple[escape], escapeStart);
          cursor += 2;
          continue;
        }
        if (escape === "x" && /^[0-9A-Fa-f]{2}$/.test(source.slice(cursor + 2, cursor + 4))) {
          pushDecoded(
            valueParts,
            offsets,
            String.fromCharCode(Number.parseInt(source.slice(cursor + 2, cursor + 4), 16)),
            escapeStart,
          );
          cursor += 4;
          continue;
        }
        if (escape === "u" && source[cursor + 2] === "{") {
          const close = source.indexOf("}", cursor + 3);
          const scalar = close < 0
            ? ""
            : source.slice(cursor + 3, close).replaceAll("_", "");
          if (/^[0-9A-Fa-f]{1,6}$/.test(scalar)) {
            pushDecoded(
              valueParts,
              offsets,
              String.fromCodePoint(Number.parseInt(scalar, 16)),
              escapeStart,
            );
            cursor = close + 1;
            continue;
          }
        }
        pushDecoded(valueParts, offsets, escape ?? "\\", escapeStart);
        cursor += escape === undefined ? 1 : 2;
      }
      literals.push({
        tokenStart,
        contentStart,
        tokenEnd: cursor < source.length ? cursor + 1 : source.length,
        value: valueParts.join(""),
        offsets,
      });
      index = cursor < source.length ? cursor + 1 : source.length;
      continue;
    }
    const charLiteral = source[index] === "'"
      ? source.slice(index).match(/^'(?:\\.|[^\\'\n])'/)
      : null;
    index += charLiteral ? charLiteral[0].length : 1;
  }
  decodedRustStringCache.set(source, literals);
  return literals;
}

type SqlProgram = {
  text: string;
  offsets: number[];
  start: number;
  end: number;
};

function concatenateSqlPrograms(
  programs: readonly SqlProgram[],
  start: number,
  end: number,
): SqlProgram {
  return {
    text: programs.map(({ text }) => text).join(""),
    offsets: programs.flatMap(({ offsets }) => offsets),
    start,
    end,
  };
}

const concatLiteralProgramCache = new Map<string, SqlProgram[]>();

function concatLiteralPrograms(
  source: string,
  literals: readonly DecodedRustStringLiteral[],
): SqlProgram[] {
  const cached = concatLiteralProgramCache.get(source);
  if (cached) return cached;
  const syntax = maskRustComments(source, true);
  const programs = [...syntax.matchAll(/\bconcat\s*!\s*\(/g)].flatMap((match) => {
    if (match.index === undefined) throw new Error("concat! has no offset");
    const open = match.index + match[0].lastIndexOf("(");
    const close = closingDelimiter(syntax, open, "(", ")");
    const pieces = literals
      .filter(({ tokenStart, tokenEnd }) => tokenStart > open && tokenEnd <= close)
      .sort((left, right) => left.tokenStart - right.tokenStart);
    if (pieces.length === 0) return [];
    const residue = syntax.slice(open + 1, close);
    if (residue.replace(/[\s,]/g, "") !== "") return [];
    return [concatenateSqlPrograms(
      pieces.map((piece) => ({
        text: piece.value,
        offsets: piece.offsets,
        start: piece.tokenStart,
        end: piece.tokenEnd,
      })),
      match.index,
      close + 1,
    )];
  });
  concatLiteralProgramCache.set(source, programs);
  return programs;
}

function exactLiteralExpressionProgram(
  source: string,
  start: number,
  end: number,
  literals: readonly DecodedRustStringLiteral[],
  concats: readonly SqlProgram[],
): SqlProgram | undefined {
  const syntax = maskRustComments(source, true);
  const candidates: SqlProgram[] = [
    ...literals.map((literal) => ({
      text: literal.value,
      offsets: literal.offsets,
      start: literal.tokenStart,
      end: literal.tokenEnd,
    })),
    ...concats,
  ].filter((program) => program.start >= start && program.end <= end);
  return candidates.find((program) => {
    const chars = syntax.slice(start, end).split("");
    for (let index = program.start - start; index < program.end - start; index += 1) {
      if (chars[index] !== "\n") chars[index] = " ";
    }
    return chars.join("").trim() === "";
  });
}

type SqlxQueryCall = {
  kind: string;
  start: number;
  end: number;
  open: number;
  close: number;
  literal?: SqlProgram;
};

const sqlxQueryCallCache = new Map<string, SqlxQueryCall[]>();

function firstTopLevelArgumentEnd(syntax: string, open: number, close: number): number {
  const closers: Record<string, string> = { "(": ")", "[": "]", "{": "}" };
  const stack: string[] = [];
  for (let index = open + 1; index < close; index += 1) {
    const char = syntax[index];
    if (char in closers) stack.push(closers[char]);
    else if (stack.at(-1) === char) stack.pop();
    else if (char === "," && stack.length === 0) return index;
  }
  return close;
}

type ImportedSqlxQueryAlias = ImportedSqlxModuleAlias & {
  kind: string;
};

function importedSqlxQueryAliases(
  source: string,
  syntax: string,
): ImportedSqlxQueryAlias[] {
  const aliases: Array<{ name: string; kind: string; offset: number }> = [];
  const add = (
    kind: string,
    alias: string | undefined,
    offset: number,
  ): void => {
    aliases.push({ name: alias ?? kind, kind, offset });
  };
  for (const match of syntax.matchAll(
    /\buse\s+sqlx\s*::\s*(raw_sql|query(?:_as|_scalar)?)(?:\s+as\s+([A-Za-z_][A-Za-z0-9_]*))?\s*;/g,
  )) {
    if (match.index === undefined) throw new Error("sqlx query import has no offset");
    add(match[1], match[2], match.index);
  }
  for (const match of syntax.matchAll(/\buse\s+sqlx\s*::\s*\{([^{};]+)\}\s*;/g)) {
    if (match.index === undefined) throw new Error("grouped sqlx import has no offset");
    for (const entry of match[1].split(",")) {
      const parsed = entry.trim().match(
        /^(raw_sql|query(?:_as|_scalar)?)(?:\s+as\s+([A-Za-z_][A-Za-z0-9_]*))?$/,
      );
      if (parsed) add(parsed[1], parsed[2], match.index);
    }
  }
  return aliases.map(({ name, kind, offset }) => ({
    name,
    kind,
    moduleKey: rustModuleScopeAt(source, offset).key,
    lexicalScope: rustLexicalBraceScopeAt(source, offset),
  }));
}

type RustModuleScope = {
  key: string;
  path: string;
  start: number;
  end: number;
};

const rustModuleScopeCache = new Map<string, RustModuleScope[]>();

function rustModuleScopes(source: string): RustModuleScope[] {
  const cached = rustModuleScopeCache.get(source);
  if (cached) return cached;
  const syntax = maskRustComments(source, true);
  const candidates = [...syntax.matchAll(
    /\bmod\s+([A-Za-z_][A-Za-z0-9_]*)\s*\{/g,
  )].map((match) => {
    if (match.index === undefined) throw new Error("inline Rust module has no offset");
    const open = match.index + match[0].lastIndexOf("{");
    return {
      name: match[1],
      start: open + 1,
      end: closingDelimiter(syntax, open, "{", "}"),
    };
  }).sort((left, right) =>
    (right.end - right.start) - (left.end - left.start));
  const scopes: RustModuleScope[] = [{
    key: `0:${source.length}`,
    path: "",
    start: 0,
    end: source.length,
  }];
  for (const candidate of candidates) {
    const parent = scopes.filter(({ start, end }) =>
      candidate.start >= start && candidate.end <= end)
      .sort((left, right) => (left.end - left.start) - (right.end - right.start))[0];
    const path = parent?.path
      ? `${parent.path}::${candidate.name}`
      : candidate.name;
    scopes.push({
      key: `${candidate.start}:${candidate.end}`,
      path,
      start: candidate.start,
      end: candidate.end,
    });
  }
  rustModuleScopeCache.set(source, scopes);
  return scopes;
}

function rustModuleScopeAt(source: string, offset: number): RustModuleScope {
  return rustModuleScopes(source).filter(({ start, end }) =>
    offset >= start && offset < end)
    .sort((left, right) => (left.end - left.start) - (right.end - right.start))[0];
}

type RustLexicalBraceScope = {
  start: number;
  end: number;
};

const rustLexicalBraceScopeCache = new Map<string, RustLexicalBraceScope[]>();

function rustLexicalBraceScopes(source: string): RustLexicalBraceScope[] {
  const cached = rustLexicalBraceScopeCache.get(source);
  if (cached) return cached;
  const syntax = maskRustComments(source, true);
  const scopes: RustLexicalBraceScope[] = [{
    start: 0,
    end: source.length,
  }];
  const openBraces: number[] = [];
  for (let index = 0; index < syntax.length; index += 1) {
    if (syntax[index] === "{") {
      openBraces.push(index);
    } else if (syntax[index] === "}") {
      const open = openBraces.pop();
      if (open !== undefined) scopes.push({ start: open + 1, end: index });
    }
  }
  rustLexicalBraceScopeCache.set(source, scopes);
  return scopes;
}

function rustLexicalBraceScopeAt(
  source: string,
  offset: number,
): RustLexicalBraceScope {
  return rustLexicalBraceScopes(source).filter(({ start, end }) =>
    offset >= start && offset < end)
    .sort((left, right) => (left.end - left.start) - (right.end - right.start))[0];
}

type ImportedSqlxModuleAlias = {
  name: string;
  moduleKey: string;
  lexicalScope: RustLexicalBraceScope;
};

function importedSqlxModuleAliases(
  source: string,
  syntax: string,
): ImportedSqlxModuleAlias[] {
  const matches: Array<{ name: string; offset: number }> = [];
  for (const match of syntax.matchAll(
    /\buse\s+sqlx\s+as\s+([A-Za-z_][A-Za-z0-9_]*)\s*;/g,
  )) {
    if (match.index === undefined) throw new Error("sqlx module alias has no offset");
    matches.push({ name: match[1], offset: match.index });
  }
  for (const match of syntax.matchAll(/\buse\s+sqlx\s*::\s*\{([^{};]+)\}\s*;/g)) {
    if (match.index === undefined) throw new Error("grouped sqlx import has no offset");
    for (const entry of match[1].split(",")) {
      const alias = entry.trim().match(
        /^self\s+as\s+([A-Za-z_][A-Za-z0-9_]*)$/,
      )?.[1];
      if (alias) matches.push({ name: alias, offset: match.index });
    }
  }
  return matches.map(({ name, offset }) => ({
    name,
    moduleKey: rustModuleScopeAt(source, offset).key,
    lexicalScope: rustLexicalBraceScopeAt(source, offset),
  }));
}

function sqlxImportApplies(
  source: string,
  alias: ImportedSqlxModuleAlias,
  offset: number,
): boolean {
  if (rustModuleScopeAt(source, offset).key !== alias.moduleKey) return false;
  return offset >= alias.lexicalScope.start && offset < alias.lexicalScope.end;
}

function sqlxQueryCalls(source: string): SqlxQueryCall[] {
  const cached = sqlxQueryCallCache.get(source);
  if (cached) return cached;
  const syntax = maskRustComments(source, true);
  const literals = decodedRustStringLiterals(source);
  const concats = concatLiteralPrograms(source, literals);
  let bindings: SimpleLiteralBinding[] | undefined;
  const resolveLiteral = (
    start: number,
    end: number,
    useOffset: number,
  ): SqlProgram | undefined => {
    const direct = exactLiteralExpressionProgram(source, start, end, literals, concats);
    if (direct) return direct;
    if (!/^[A-Za-z_][A-Za-z0-9_]*$/.test(syntax.slice(start, end).trim())) {
      return undefined;
    }
    bindings ??= simpleLiteralBindings(source);
    return literalExpressionProgram(
      source,
      start,
      end,
      useOffset,
      literals,
      concats,
      bindings,
    );
  };
  const candidates: Array<RegExpMatchArray & { sqlxKind: string }> = [];
  for (const match of syntax.matchAll(
    /\bsqlx\s*::\s*(raw_sql|query(?:_as|_scalar)?)(\s*::\s*<[^(){};]*>)?\s*\(/g,
  )) {
    candidates.push(Object.assign(match, { sqlxKind: match[1] }));
  }
  for (const alias of importedSqlxModuleAliases(source, syntax)) {
    const name = alias.name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    for (const match of syntax.matchAll(
      new RegExp(
        `\\b${name}\\s*::\\s*(raw_sql|query(?:_as|_scalar)?)(\\s*::\\s*<[^(){};]*>)?\\s*\\(`,
        "g",
      ),
    )) {
      if (
        match.index !== undefined
        && sqlxImportApplies(source, alias, match.index)
      ) {
        candidates.push(Object.assign(match, { sqlxKind: match[1] }));
      }
    }
  }
  for (const alias of importedSqlxQueryAliases(source, syntax)) {
    const name = alias.name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    for (const match of syntax.matchAll(
      new RegExp(`\\b(${name})(\\s*::\\s*<[^(){};]*>)?\\s*\\(`, "g"),
    )) {
      if (
        match.index !== undefined
        && sqlxImportApplies(source, alias, match.index)
      ) {
        candidates.push(Object.assign(match, { sqlxKind: alias.kind }));
      }
    }
  }
  const seenStarts = new Set<number>();
  const calls = candidates
    .sort((left, right) => (left.index ?? -1) - (right.index ?? -1))
    .filter((match) => {
      if (match.index === undefined || seenStarts.has(match.index)) return false;
      seenStarts.add(match.index);
      return true;
    })
    .map((match) => {
    if (match.index === undefined) throw new Error("sqlx query call has no offset");
    const open = match.index + match[0].lastIndexOf("(");
    const close = closingDelimiter(syntax, open, "(", ")");
    const argumentEnd = firstTopLevelArgumentEnd(syntax, open, close);
    const generic = (match[2] ?? "")
      .replace(/\s*::\s*/g, "::")
      .replace(/\s+/g, " ");
    return {
      kind: `${match.sqlxKind}${generic}`,
      start: match.index,
      end: close + 1,
      open,
      close,
      literal: resolveLiteral(open + 1, argumentEnd, match.index),
    };
  });
  sqlxQueryCallCache.set(source, calls);
  return calls;
}

function sqlxQueryLiterals(source: string): Array<{
  kind: string;
  sql: string;
  start: number;
  end: number;
}> {
  return sqlxQueryCalls(source).flatMap((call) =>
    call.literal
      ? [{ kind: call.kind, sql: call.literal.text, start: call.start, end: call.end }]
      : []);
}

type RustFunctionRange = {
  name: string;
  declarationStart: number;
  start: number;
  end: number;
};

const rustFunctionRangeCache = new Map<string, RustFunctionRange[]>();

function rustFunctionRanges(source: string): RustFunctionRange[] {
  const cached = rustFunctionRangeCache.get(source);
  if (cached) return cached;
  const syntax = maskRustComments(source, true);
  const candidates: RustFunctionRange[] = [];
  for (const match of syntax.matchAll(
    /\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\s*(?:<[^>{}]*>)?\s*\(/g,
  )) {
    if (match.index === undefined) throw new Error("Rust function has no offset");
    const paramsOpen = match.index + match[0].lastIndexOf("(");
    const paramsClose = closingDelimiter(syntax, paramsOpen, "(", ")");
    const bodyOpen = syntax.indexOf("{", paramsClose);
    const declarationEnd = syntax.indexOf(";", paramsClose);
    if (bodyOpen < 0 || (declarationEnd >= 0 && declarationEnd < bodyOpen)) continue;
    const bodyClose = closingDelimiter(syntax, bodyOpen, "{", "}");
    candidates.push({
      name: match[1],
      declarationStart: match.index,
      start: bodyOpen + 1,
      end: bodyClose,
    });
  }
  rustFunctionRangeCache.set(source, candidates);
  return candidates;
}

function enclosingRustFunctionRange(
  source: string,
  offset: number,
): RustFunctionRange | undefined {
  return rustFunctionRanges(source).filter(
    ({ start, end }) => offset >= start && offset < end,
  ).sort(
    (left, right) => (left.end - left.start) - (right.end - right.start),
  )[0];
}

function normalizedExecutableRust(source: string): string {
  const executable = maskRustComments(
    normalizeAnalysisContractSourceText(source),
    false,
  );
  let normalizedSource = "";
  let pendingWhitespace = false;
  const appendWhitespace = (): void => {
    if (pendingWhitespace && normalizedSource.length > 0) normalizedSource += " ";
    pendingWhitespace = false;
  };
  const quotedEnd = (start: number, quote: string): number => {
    for (let index = start + 1; index < executable.length; index += 1) {
      if (executable[index] === "\\") index += 1;
      else if (executable[index] === quote) return index + 1;
    }
    return executable.length;
  };
  for (let index = 0; index < executable.length;) {
    if (/\s/.test(executable[index] ?? "")) {
      pendingWhitespace = true;
      index += 1;
      continue;
    }
    const charLiteral = executable[index] === "'"
      && /^'(?:\\.|[^\\'\n])'/.test(executable.slice(index));
    if (executable[index] === '"' || charLiteral) {
      appendWhitespace();
      const end = quotedEnd(index, executable[index]);
      normalizedSource += executable.slice(index, end);
      index = end;
      continue;
    }
    const raw = executable.slice(index).match(/^r(#+)?"/);
    if (raw) {
      appendWhitespace();
      const terminator = `"${raw[1] ?? ""}`;
      const close = executable.indexOf(terminator, index + raw[0].length);
      const end = close < 0 ? executable.length : close + terminator.length;
      normalizedSource += executable.slice(index, end);
      index = end;
      continue;
    }
    appendWhitespace();
    normalizedSource += executable[index];
    index += 1;
  }
  return normalizedSource.trim();
}

type SimpleLiteralBinding = {
  name: string;
  declarationStart: number;
  expressionStart: number;
  expressionEnd: number;
  scope?: { start: number; end: number };
};

function simpleLiteralBindings(source: string): SimpleLiteralBinding[] {
  const syntax = maskRustComments(source, true);
  const commentsOnly = maskRustComments(source, false);
  return [...syntax.matchAll(
    /\b(?:const|static|let)\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*(?::[^=;\n]+)?=/g,
  )].flatMap((match) => {
    if (match.index === undefined) throw new Error("literal binding has no offset");
    let expressionStart = match.index + match[0].length;
    while (/\s/.test(commentsOnly[expressionStart] ?? "")) expressionStart += 1;
    const expressionEnd = syntax.indexOf(";", expressionStart);
    return expressionEnd < 0
      ? []
      : [{
          name: match[1],
          declarationStart: match.index,
          expressionStart,
          expressionEnd,
          scope: enclosingRustFunctionRange(source, match.index),
        }];
  });
}

function literalExpressionProgram(
  source: string,
  start: number,
  end: number,
  useOffset: number,
  literals: readonly DecodedRustStringLiteral[],
  concats: readonly SqlProgram[],
  bindings: readonly SimpleLiteralBinding[],
  resolving: ReadonlySet<string> = new Set(),
): SqlProgram | undefined {
  const direct = exactLiteralExpressionProgram(source, start, end, literals, concats);
  if (direct) return direct;
  const identifier = maskRustComments(source, true).slice(start, end).trim();
  if (!/^[A-Za-z_][A-Za-z0-9_]*$/.test(identifier) || resolving.has(identifier)) {
    return undefined;
  }
  const useScope = enclosingRustFunctionRange(source, useOffset);
  const sameScope = (binding: SimpleLiteralBinding): boolean =>
    binding.scope?.start === useScope?.start && binding.scope?.end === useScope?.end;
  const visible = bindings.filter((binding) =>
    binding.name === identifier && (sameScope(binding) || binding.scope === undefined));
  const preceding = visible.filter(({ declarationStart }) => declarationStart < useOffset);
  const selected = preceding.at(-1) ?? visible[0];
  if (!selected) return undefined;
  return literalExpressionProgram(
    source,
    selected.expressionStart,
    selected.expressionEnd,
    selected.declarationStart,
    literals,
    concats,
    bindings,
    new Set([...resolving, identifier]),
  );
}

function staticSqlFactoryPrograms(
  source: string,
  literals: readonly DecodedRustStringLiteral[],
  concats: readonly SqlProgram[],
  bindings: readonly SimpleLiteralBinding[],
): Map<string, SqlProgram> {
  const syntax = maskRustComments(source, true);
  const factories = new Map<string, SqlProgram>();
  for (const match of syntax.matchAll(
    /\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\s*(?:<[^>{}]*>)?\s*\(/g,
  )) {
    if (match.index === undefined) throw new Error("static SQL factory has no offset");
    const paramsOpen = match.index + match[0].lastIndexOf("(");
    const paramsClose = closingDelimiter(syntax, paramsOpen, "(", ")");
    if (syntax.slice(paramsOpen + 1, paramsClose).trim() !== "") continue;
    const bodyOpen = syntax.indexOf("{", paramsClose);
    const declarationEnd = syntax.indexOf(";", paramsClose);
    if (bodyOpen < 0 || (declarationEnd >= 0 && declarationEnd < bodyOpen)) continue;
    const bodyClose = closingDelimiter(syntax, bodyOpen, "{", "}");
    const body = syntax.slice(bodyOpen + 1, bodyClose);
    const explicitReturn = /\breturn\s+([\s\S]*?)\s*;\s*$/.exec(body);
    let expressionStart: number;
    let expressionEnd: number;
    if (explicitReturn && explicitReturn.index !== undefined) {
      expressionStart = bodyOpen + 1 + explicitReturn.index
        + explicitReturn[0].indexOf(explicitReturn[1]);
      expressionEnd = expressionStart + explicitReturn[1].length;
    } else {
      const tailStart = body.lastIndexOf(";") + 1;
      expressionStart = bodyOpen + 1 + tailStart;
      expressionEnd = bodyClose;
    }
    const program = literalExpressionProgram(
      source,
      expressionStart,
      expressionEnd,
      expressionStart,
      literals,
      concats,
      bindings,
    );
    if (program) factories.set(match[1], program);
  }
  return factories;
}

function staticSqlExpressionProgram(
  source: string,
  start: number,
  end: number,
  useOffset: number,
  literals: readonly DecodedRustStringLiteral[],
  concats: readonly SqlProgram[],
  bindings: readonly SimpleLiteralBinding[],
  factories: ReadonlyMap<string, SqlProgram>,
  resolving: ReadonlySet<string> = new Set(),
): SqlProgram | undefined {
  const direct = literalExpressionProgram(
    source,
    start,
    end,
    useOffset,
    literals,
    concats,
    bindings,
  );
  if (direct) return direct;
  const expression = maskRustComments(source, true).slice(start, end).trim();
  const factory = expression.match(/^([A-Za-z_][A-Za-z0-9_]*)\s*\(\s*\)$/)?.[1];
  if (factory && !resolving.has(`fn:${factory}`)) {
    const program = factories.get(factory);
    return program
      ? { ...program, start, end }
      : undefined;
  }
  if (!/^[A-Za-z_][A-Za-z0-9_]*$/.test(expression) || resolving.has(`let:${expression}`)) {
    return undefined;
  }
  const useScope = enclosingRustFunctionRange(source, useOffset);
  const visible = bindings.filter((binding) =>
    binding.name === expression
    && (
      binding.scope === undefined
      || (
        binding.scope.start === useScope?.start
        && binding.scope.end === useScope?.end
      )
    ));
  const selected = visible.filter(({ declarationStart }) => declarationStart < useOffset).at(-1)
    ?? visible[0];
  return selected
    ? staticSqlExpressionProgram(
        source,
        selected.expressionStart,
        selected.expressionEnd,
        selected.declarationStart,
        literals,
        concats,
        bindings,
        factories,
        new Set([...resolving, `let:${expression}`]),
      )
    : undefined;
}

function executorSqlAnalysis(
  source: string,
  literals: readonly DecodedRustStringLiteral[],
  concats: readonly SqlProgram[],
): {
  programs: SqlProgram[];
  unresolved: UnresolvedSqlConsumer[];
} {
  const cached = executorSqlAnalysisCache.get(source);
  if (cached) return cached;
  const syntax = maskRustComments(source, true);
  const bindings = simpleLiteralBindings(source);
  const factories = staticSqlFactoryPrograms(source, literals, concats, bindings);
  const queryCalls = sqlxQueryCalls(source);
  const builderPrograms = queryBuilderPrograms(source, literals, concats);
  type ReceiverWrite = {
    name: string;
    start: number;
    scope?: RustFunctionRange;
    blockEnd: number;
    provenQueryObject: boolean;
  };
  const receiverWrites: ReceiverWrite[] = [];
  const bindingNameStarts = new Set<number>();
  const blockEndAt = (
    scope: RustFunctionRange,
    offset: number,
  ): number => {
    const openBraces: number[] = [];
    for (let index = scope.start; index < offset; index += 1) {
      if (syntax[index] === "{") openBraces.push(index);
      else if (syntax[index] === "}") openBraces.pop();
    }
    const open = openBraces.at(-1);
    return open === undefined
      ? scope.end
      : closingDelimiter(syntax, open, "{", "}");
  };
  const recordWrite = (
    name: string,
    start: number,
    expressionStart: number,
  ): void => {
    const expressionEnd = syntax.indexOf(";", expressionStart);
    const scope = enclosingRustFunctionRange(source, start);
    if (expressionEnd < 0 || !scope || expressionEnd >= scope.end) return;
    const expressionSyntax = syntax.slice(expressionStart, expressionEnd);
    const provenSqlxQuery = queryCalls.some((call) =>
      call.start >= expressionStart && call.end <= expressionEnd);
    const provenBuilderBuild =
      /\.\s*build(?:_query_as|_query_scalar)?\s*\(\s*\)/.test(expressionSyntax)
      && builderPrograms.some((program) =>
        program.start < expressionEnd && program.end > expressionStart);
    receiverWrites.push({
      name,
      start,
      scope,
      blockEnd: blockEndAt(scope, start),
      provenQueryObject: provenSqlxQuery || provenBuilderBuild,
    });
  };
  for (const match of syntax.matchAll(
    /\blet\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)(?:\s*:[^=;\n]+)?\s*=/g,
  )) {
    if (match.index === undefined) throw new Error("receiver binding has no offset");
    const nameStart = match.index + match[0].lastIndexOf(match[1]);
    bindingNameStarts.add(nameStart);
    recordWrite(match[1], match.index, match.index + match[0].length);
  }
  for (const match of syntax.matchAll(
    /(?<![A-Za-z0-9_:.])([A-Za-z_][A-Za-z0-9_]*)\s*=(?!=|>)/g,
  )) {
    if (
      match.index === undefined
      || bindingNameStarts.has(match.index)
    ) {
      continue;
    }
    recordWrite(match[1], match.index, match.index + match[0].length);
  }
  const programs: SqlProgram[] = [];
  const unresolved: UnresolvedSqlConsumer[] = [];
  for (const match of syntax.matchAll(/\.\s*execute\s*\(/g)) {
    if (match.index === undefined) throw new Error("executor call has no offset");
    const open = match.index + match[0].lastIndexOf("(");
    const close = closingDelimiter(syntax, open, "(", ")");
    const functionScope = enclosingRustFunctionRange(source, match.index);
    const statementStart = Math.max(
      syntax.lastIndexOf(";", match.index) + 1,
      functionScope?.start ?? 0,
    );
    const receiverPrefix = syntax.slice(statementStart, match.index);
    const inlineQueryTerminal = queryCalls.some((call) =>
      call.end <= match.index
      && call.start >= statementStart
      && call.start >= (functionScope?.start ?? 0));
    const builderTerminal = builderPrograms.some((program) =>
      program.start < match.index && program.end > match.index);
    const receiver = receiverPrefix.match(
      /\b([A-Za-z_][A-Za-z0-9_]*)\s*$/,
    )?.[1];
    const latestReceiverWrite = receiver === undefined
      ? undefined
      : receiverWrites.filter((write) =>
        write.name === receiver
        && write.start < match.index
        && match.index < write.blockEnd
        && write.scope?.start === functionScope?.start
        && write.scope?.end === functionScope?.end)
        .sort((left, right) => left.start - right.start)
        .at(-1);
    const trackedQueryObject = latestReceiverWrite?.provenQueryObject === true;
    if (inlineQueryTerminal || builderTerminal || trackedQueryObject) continue;
    const argumentEnd = firstTopLevelArgumentEnd(syntax, open, close);
    const program = staticSqlExpressionProgram(
      source,
      open + 1,
      argumentEnd,
      match.index,
      literals,
      concats,
      bindings,
      factories,
    );
    if (program) {
      programs.push({ ...program, start: match.index, end: close + 1 });
    } else {
      unresolved.push({
        kind: "executor",
        expression: source.slice(open + 1, argumentEnd).trim(),
        start: open + 1,
        end: argumentEnd,
      });
    }
  }
  const analysis = { programs, unresolved };
  executorSqlAnalysisCache.set(source, analysis);
  return analysis;
}

function executorSqlPrograms(
  source: string,
  literals: readonly DecodedRustStringLiteral[],
  concats: readonly SqlProgram[],
): SqlProgram[] {
  return executorSqlAnalysis(source, literals, concats).programs;
}

function importedQueryBuilderAliases(
  source: string,
  syntax: string,
): ImportedSqlxModuleAlias[] {
  const aliases: Array<{ name: string; offset: number }> = [];
  for (const match of syntax.matchAll(
    /\buse\s+sqlx\s*::\s*QueryBuilder(?:\s+as\s+([A-Za-z_][A-Za-z0-9_]*))?\s*;/g,
  )) {
    if (match.index === undefined) throw new Error("QueryBuilder import has no offset");
    aliases.push({ name: match[1] ?? "QueryBuilder", offset: match.index });
  }
  for (const match of syntax.matchAll(/\buse\s+sqlx\s*::\s*\{([^{};]+)\}\s*;/g)) {
    if (match.index === undefined) throw new Error("grouped sqlx import has no offset");
    for (const entry of match[1].split(",")) {
      const parsed = entry.trim().match(
        /^QueryBuilder(?:\s+as\s+([A-Za-z_][A-Za-z0-9_]*))?$/,
      );
      if (parsed) {
        aliases.push({
          name: parsed[1] ?? "QueryBuilder",
          offset: match.index,
        });
      }
    }
  }
  return aliases.map(({ name, offset }) => ({
    name,
    moduleKey: rustModuleScopeAt(source, offset).key,
    lexicalScope: rustLexicalBraceScopeAt(source, offset),
  }));
}

type UnresolvedSqlConsumer = {
  kind: string;
  expression: string;
  start: number;
  end: number;
};

const executorSqlAnalysisCache = new Map<string, {
  programs: SqlProgram[];
  unresolved: UnresolvedSqlConsumer[];
}>();

function queryBuilderPrograms(
  source: string,
  literals: readonly DecodedRustStringLiteral[],
  concats: readonly SqlProgram[],
): SqlProgram[] {
  const cached = queryBuilderProgramCache.get(source);
  if (cached) return cached;
  const syntax = maskRustComments(source, true);
  const bindings = simpleLiteralBindings(source);
  const unresolved: UnresolvedSqlConsumer[] = [];
  const builders: RegExpMatchArray[] = [...syntax.matchAll(
    /(?<![A-Za-z0-9_:])sqlx\s*::\s*QueryBuilder(?:\s*::\s*<[^(){};]*>)?\s*::\s*new\s*\(/g,
  )];
  for (const alias of importedQueryBuilderAliases(source, syntax)) {
    const name = alias.name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    for (const match of syntax.matchAll(
      new RegExp(
        `(?<![A-Za-z0-9_:])${name}(?:\\s*::\\s*<[^(){};]*>)?\\s*::\\s*new\\s*\\(`,
        "g",
      ),
    )) {
      if (
        match.index !== undefined
        && sqlxImportApplies(source, alias, match.index)
      ) {
        builders.push(match);
      }
    }
  }
  for (const alias of importedSqlxModuleAliases(source, syntax)) {
    const name = alias.name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    for (const match of syntax.matchAll(
      new RegExp(
        `(?<![A-Za-z0-9_:])${name}\\s*::\\s*QueryBuilder(?:\\s*::\\s*<[^(){};]*>)?\\s*::\\s*new\\s*\\(`,
        "g",
      ),
    )) {
      if (
        match.index !== undefined
        && sqlxImportApplies(source, alias, match.index)
      ) {
        builders.push(match);
      }
    }
  }
  builders.sort((left, right) => (left.index ?? -1) - (right.index ?? -1));
  const programs = builders.flatMap((match) => {
    if (match.index === undefined) throw new Error("QueryBuilder::new has no offset");
    const open = match.index + match[0].lastIndexOf("(");
    const close = closingDelimiter(syntax, open, "(", ")");
    const argumentEnd = firstTopLevelArgumentEnd(syntax, open, close);
    let initial = literalExpressionProgram(
      source,
      open + 1,
      argumentEnd,
      match.index,
      literals,
      concats,
      bindings,
    );
    if (!initial) {
      unresolved.push({
        kind: "query_builder_new",
        expression: source.slice(open + 1, argumentEnd).trim(),
        start: open + 1,
        end: argumentEnd,
      });
      initial = {
        text: " ",
        offsets: [match.index],
        start: match.index,
        end: close + 1,
      };
    }
    const statementStart = syntax.lastIndexOf(";", match.index) + 1;
    const variable = syntax.slice(statementStart, match.index).match(
      /\blet\s+(?:mut\s+)?([a-z_][A-Za-z0-9_]*)\s*=\s*$/,
    )?.[1];
    const functionRange = enclosingRustFunctionRange(source, match.index);
    const limit = functionRange?.end ?? source.length;
    const pushOffsets = new Set<number>();
    const collectPushChain = (from: number, statementEnd: number): void => {
      for (const push of syntax.slice(from, statementEnd).matchAll(
        /\.\s*push(?:_unseparated)?\s*\(/g,
      )) {
        if (push.index !== undefined) pushOffsets.add(from + push.index);
      }
    };
    const directEnd = syntax.indexOf(";", close);
    collectPushChain(close, directEnd < 0 ? limit : Math.min(directEnd, limit));
    if (variable) {
      const collectVariablePushes = (name: string): void => {
        const call = new RegExp(
          `\\b${name}\\s*\\.\\s*push(?:_unseparated)?\\s*\\(`,
          "g",
        );
        for (const push of syntax.slice(close, limit).matchAll(call)) {
          if (push.index === undefined) continue;
          const absolute = close + push.index + push[0].search(/\.\s*push/);
          pushOffsets.add(absolute);
          const end = syntax.indexOf(";", absolute);
          collectPushChain(
            absolute + push[0].length,
            end < 0 ? limit : Math.min(end, limit),
          );
        }
      };
      collectVariablePushes(variable);
      const separated = new RegExp(
        `\\b${variable}\\s*\\.\\s*separated\\s*\\(`,
        "g",
      );
      for (const separatedCall of syntax.slice(close, limit).matchAll(separated)) {
        if (separatedCall.index === undefined) continue;
        const separatedStart = close + separatedCall.index;
        const separatorOpen = separatedStart + separatedCall[0].lastIndexOf("(");
        const separatorClose = closingDelimiter(syntax, separatorOpen, "(", ")");
        const separatorEnd = firstTopLevelArgumentEnd(
          syntax,
          separatorOpen,
          separatorClose,
        );
        if (!literalExpressionProgram(
          source,
          separatorOpen + 1,
          separatorEnd,
          separatedStart,
          literals,
          concats,
          bindings,
        )) {
          unresolved.push({
            kind: "query_builder_separator",
            expression: source.slice(separatorOpen + 1, separatorEnd).trim(),
            start: separatorOpen + 1,
            end: separatorEnd,
          });
        }
        const separatedStatementStart = syntax.lastIndexOf(";", separatedStart) + 1;
        const aliasName = syntax.slice(
          separatedStatementStart,
          separatedStart,
        ).match(
          /\blet\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)(?:\s*:[^=;\n]+)?\s*=\s*$/,
        )?.[1];
        if (aliasName) collectVariablePushes(aliasName);
        const separatedStatementEnd = syntax.indexOf(";", separatorClose);
        collectPushChain(
          separatorClose + 1,
          separatedStatementEnd < 0
            ? limit
            : Math.min(separatedStatementEnd, limit),
        );
      }
    }
    const pushes = [...pushOffsets].sort((left, right) => left - right).flatMap((offset) => {
      const openOffset = syntax.indexOf("(", offset);
      if (openOffset < 0 || openOffset >= limit) return [];
      const closeOffset = closingDelimiter(syntax, openOffset, "(", ")");
      const program = literalExpressionProgram(
        source,
        openOffset + 1,
        firstTopLevelArgumentEnd(syntax, openOffset, closeOffset),
        offset,
        literals,
        concats,
        bindings,
      );
      if (program) return [program];
      unresolved.push({
        kind: "query_builder_push",
        expression: source.slice(
          openOffset + 1,
          firstTopLevelArgumentEnd(syntax, openOffset, closeOffset),
        ).trim(),
        start: openOffset + 1,
        end: firstTopLevelArgumentEnd(syntax, openOffset, closeOffset),
      });
      return [];
    });
    let consumerEnd = pushes.at(-1)?.end ?? close + 1;
    if (variable) {
      const escapedVariable = variable.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
      const build = new RegExp(
        `\\b${escapedVariable}\\s*\\.\\s*build(?:_query_as|_query_scalar)?\\s*\\(`,
      ).exec(syntax.slice(close, limit));
      if (build?.index !== undefined) {
        const buildStart = close + build.index;
        const statementEnd = syntax.indexOf(";", buildStart);
        consumerEnd = statementEnd < 0 || statementEnd >= limit
          ? limit
          : statementEnd + 1;
      }
    } else if (directEnd >= 0 && directEnd < limit) {
      consumerEnd = directEnd + 1;
    }
    return [concatenateSqlPrograms(
      [initial, ...pushes],
      match.index,
      consumerEnd,
    )];
  });
  queryBuilderProgramCache.set(source, programs);
  queryBuilderUnresolvedConsumerCache.set(source, unresolved);
  return programs;
}

const queryBuilderProgramCache = new Map<string, SqlProgram[]>();
const queryBuilderUnresolvedConsumerCache = new Map<string, UnresolvedSqlConsumer[]>();
const allLiteralSqlProgramCache = new Map<string, SqlProgram[]>();

function allLiteralSqlPrograms(source: string): SqlProgram[] {
  const cached = allLiteralSqlProgramCache.get(source);
  if (cached) return cached;
  const literals = decodedRustStringLiterals(source);
  const literalPrograms = literals.map((literal) => ({
    text: literal.value,
    offsets: literal.offsets,
    start: literal.tokenStart,
    end: literal.tokenEnd,
  }));
  const concats = concatLiteralPrograms(source, literals);
  const builders = queryBuilderPrograms(source, literals, concats);
  const programs = [...literalPrograms, ...concats, ...builders];
  const seen = new Set<string>();
  const selected = programs.filter((program) => {
    const key = `${program.start}:${program.end}:${program.text}`;
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  });
  allLiteralSqlProgramCache.set(source, selected);
  return selected;
}

function maskSqlComments(sql: string): string {
  const masked = sql.split("");
  let quote: "'" | "\"" | "`" | "]" | null = null;
  for (let index = 0; index < sql.length;) {
    const char = sql[index];
    if (quote) {
      if (quote === "'" && char === "'" && sql[index + 1] === "'") index += 2;
      else if ((quote === "]" && char === "]") || (quote !== "]" && char === quote)) {
        quote = null;
        index += 1;
      } else index += 1;
    } else if (char === "'" || char === "\"" || char === "`" || char === "[") {
      quote = char === "[" ? "]" : char;
      index += 1;
    } else if (sql.startsWith("/*", index)) {
      const close = sql.indexOf("*/", index + 2);
      const end = close < 0 ? sql.length : close + 2;
      for (let cursor = index; cursor < end; cursor += 1) {
        if (masked[cursor] !== "\n") masked[cursor] = " ";
      }
      index = end;
    } else if (sql.startsWith("--", index)) {
      const newline = sql.indexOf("\n", index + 2);
      const end = newline < 0 ? sql.length : newline;
      for (let cursor = index; cursor < end; cursor += 1) masked[cursor] = " ";
      index = end;
    } else index += 1;
  }
  return masked.join("");
}

function splitSqlStatements(sql: string): string[] {
  const executable = maskSqlComments(sql);
  const statements: string[] = [];
  let start = 0;
  let quote: "'" | "\"" | "`" | "]" | null = null;
  for (let index = 0; index < executable.length; index += 1) {
    const char = executable[index];
    if (quote) {
      if (quote === "'" && char === "'" && executable[index + 1] === "'") index += 1;
      else if ((quote === "]" && char === "]") || (quote !== "]" && char === quote)) quote = null;
    } else if (char === "'" || char === "\"" || char === "`" || char === "[") {
      quote = char === "[" ? "]" : char;
    } else if (char === ";") {
      const statement = executable.slice(start, index).trim();
      if (statement) statements.push(statement);
      start = index + 1;
    }
  }
  const tail = executable.slice(start).trim();
  if (tail) statements.push(tail);
  return statements;
}

type SqlToken = {
  text: string;
  value: string;
  upper: string;
  start: number;
  end: number;
  depth: number;
  kind: "identifier" | "number" | "symbol";
};

function sqlTokens(sql: string): SqlToken[] {
  const executable = maskSqlComments(sql);
  const tokens: Array<Omit<SqlToken, "depth">> = [];
  for (let index = 0; index < executable.length;) {
    const char = executable[index];
    if (/\s/.test(char)) {
      index += 1;
      continue;
    }
    if (char === "'" || char === "\"" || char === "`" || char === "[") {
      const start = index;
      const close = char === "[" ? "]" : char;
      const value: string[] = [];
      index += 1;
      while (index < executable.length) {
        if (executable[index] === close) {
          if (executable[index + 1] === close) {
            value.push(close);
            index += 2;
            continue;
          }
          index += 1;
          break;
        }
        value.push(executable[index]);
        index += 1;
      }
      const decoded = value.join("");
      tokens.push({
        text: executable.slice(start, index),
        value: decoded,
        upper: decoded.toUpperCase(),
        start,
        end: index,
        kind: "identifier",
      });
      continue;
    }
    const identifier = executable.slice(index).match(/^[A-Za-z_][A-Za-z0-9_$]*/)?.[0];
    if (identifier) {
      tokens.push({
        text: identifier,
        value: identifier,
        upper: identifier.toUpperCase(),
        start: index,
        end: index + identifier.length,
        kind: "identifier",
      });
      index += identifier.length;
      continue;
    }
    const number = executable.slice(index).match(
      /^(?:0[xX][0-9A-Fa-f_]+|(?:[0-9][0-9_]*(?:\.[0-9_]*)?|\.[0-9][0-9_]*)(?:[eE][+-]?[0-9][0-9_]*)?)/,
    )?.[0];
    if (number) {
      tokens.push({
        text: number,
        value: number,
        upper: number,
        start: index,
        end: index + number.length,
        kind: "number",
      });
      index += number.length;
      continue;
    }
    tokens.push({
      text: char,
      value: char,
      upper: char,
      start: index,
      end: index + 1,
      kind: "symbol",
    });
    index += 1;
  }
  let depth = 0;
  return tokens.map((token) => {
    if (token.text === ")") depth = Math.max(0, depth - 1);
    const selected = { ...token, depth };
    if (token.text === "(") depth += 1;
    return selected;
  });
}

function migrationCreatedTables(sql: string): string[] {
  return splitSqlStatements(sql).flatMap((statement) => {
    const tokens = sqlTokens(statement);
    let index = 0;
    if (tokens[index]?.upper !== "CREATE") return [];
    index += 1;
    if (["TEMP", "TEMPORARY"].includes(tokens[index]?.upper ?? "")) index += 1;
    if (tokens[index]?.upper === "VIRTUAL") index += 1;
    if (tokens[index]?.upper !== "TABLE") return [];
    index += 1;
    if (tokens[index]?.upper === "IF") {
      if (tokens[index + 1]?.upper !== "NOT" || tokens[index + 2]?.upper !== "EXISTS") {
        return [];
      }
      index += 3;
    }
    let table = tokens[index];
    if (table?.kind !== "identifier") return [];
    if (tokens[index + 1]?.text === ".") {
      table = tokens[index + 2];
      if (table?.kind !== "identifier") return [];
    }
    return [table.value.toLowerCase()];
  });
}

function sqlTableTerm(
  tokens: readonly SqlToken[],
  start: number,
): { tokens: SqlToken[]; next: number } | undefined {
  const first = tokens[start];
  if (first?.text === "(") {
    const close = tokens.findIndex((token, index) =>
      index > start && token.text === ")" && token.depth === first.depth);
    if (close < 0) return undefined;
    const leading = tokens[start + 1];
    if (!leading || ["SELECT", "WITH", "VALUES"].includes(leading.upper)) {
      return { tokens: [], next: close + 1 };
    }
    const selected: SqlToken[] = [];
    const listDepth = first.depth + 1;
    let cursor = start + 1;
    while (cursor < close) {
      const term = sqlTableTerm(tokens, cursor);
      if (!term) break;
      selected.push(...term.tokens);
      cursor = term.next;
      while (cursor < close
        && !(tokens[cursor].text === "," && tokens[cursor].depth === listDepth)) {
        cursor += 1;
      }
      if (cursor < close) cursor += 1;
    }
    return { tokens: selected, next: close + 1 };
  }
  if (!first || first.kind !== "identifier" || ["SELECT", "WITH", "VALUES"].includes(first.upper)) {
    return undefined;
  }
  if (tokens[start + 1]?.text === "." && tokens[start + 2]?.kind === "identifier") {
    return { tokens: [tokens[start + 2]], next: start + 3 };
  }
  return { tokens: [first], next: start + 1 };
}

function sqlTableReferences(sql: string): Array<{
  operation: string;
  table: string;
  sqlOffset: number;
}> {
  const tokens = sqlTokens(sql);
  const references: Array<{ operation: string; table: string; sqlOffset: number }> = [];
  const add = (
    operation: string,
    keyword: SqlToken,
    start: number,
  ): void => {
    const selected = sqlTableTerm(tokens, start);
    if (!selected) return;
    for (const table of selected.tokens) {
      references.push({
        operation,
        table: table.value.toLowerCase(),
        sqlOffset: keyword.start,
      });
    }
  };
  const clauseEnd = new Set([
    "WHERE", "GROUP", "ORDER", "HAVING", "LIMIT", "OFFSET", "WINDOW",
    "UNION", "INTERSECT", "EXCEPT", "RETURNING",
  ]);
  const conflictActions = new Set(["ROLLBACK", "ABORT", "REPLACE", "FAIL", "IGNORE"]);
  for (let index = 0; index < tokens.length; index += 1) {
    const token = tokens[index];
    if (token.upper === "FROM") {
      const operation = tokens[index - 1]?.upper === "DELETE"
        && tokens[index - 1]?.depth === token.depth ? "DELETE FROM" : "FROM";
      add(
        operation,
        operation === "DELETE FROM" ? tokens[index - 1] : token,
        index + 1,
      );
      if (operation !== "FROM") continue;
      for (let cursor = index + 1; cursor < tokens.length; cursor += 1) {
        const candidate = tokens[cursor];
        if (candidate.depth < token.depth
          || (candidate.depth === token.depth
            && (candidate.text === ";" || clauseEnd.has(candidate.upper)))) break;
        if (candidate.depth === token.depth && candidate.text === ",") {
          add("FROM", candidate, cursor + 1);
        }
      }
    } else if (token.upper === "JOIN") {
      add("JOIN", token, index + 1);
    } else if (token.upper === "UPDATE") {
      const hasConflictAction = tokens[index + 1]?.upper === "OR"
        && conflictActions.has(tokens[index + 2]?.upper ?? "")
        && tokens[index + 1]?.depth === token.depth
        && tokens[index + 2]?.depth === token.depth;
      add("UPDATE", token, index + (hasConflictAction ? 3 : 1));
    } else if (token.upper === "INTO") {
      const previous = tokens[index - 1];
      const insert = previous?.upper === "INSERT"
        || (conflictActions.has(previous?.upper ?? "")
          && tokens[index - 2]?.upper === "OR"
          && tokens[index - 3]?.upper === "INSERT");
      add(insert ? "INSERT INTO" : "INTO", insert && previous?.upper !== "INSERT"
        ? tokens[index - 3] : insert ? previous : token, index + 1);
    }
  }
  return references;
}

function sqlConsumerPrograms(source: string): SqlProgram[] {
  const literals = decodedRustStringLiterals(source);
  const concats = concatLiteralPrograms(source, literals);
  return [
    ...sqlxQueryCalls(source).flatMap((call) => call.literal ? [call.literal] : []),
    ...executorSqlPrograms(source, literals, concats),
    ...queryBuilderPrograms(source, literals, concats),
  ];
}

function unresolvedSqlConsumers(source: string): UnresolvedSqlConsumer[] {
  const syntax = maskRustComments(source, true);
  const queryConsumers = sqlxQueryCalls(source).flatMap((call) => {
    if (call.literal) return [];
    const argumentEnd = firstTopLevelArgumentEnd(syntax, call.open, call.close);
    return [{
      kind: call.kind,
      expression: source.slice(call.open + 1, argumentEnd).trim(),
      start: call.open + 1,
      end: argumentEnd,
    }];
  });
  const literals = decodedRustStringLiterals(source);
  const concats = concatLiteralPrograms(source, literals);
  const executorConsumers = executorSqlAnalysis(source, literals, concats).unresolved;
  queryBuilderPrograms(source, literals, concats);
  return [
    ...queryConsumers,
    ...executorConsumers,
    ...(queryBuilderUnresolvedConsumerCache.get(source) ?? []),
  ].sort((left, right) => left.start - right.start);
}

type UnresolvedConsumerInventoryEntry = {
  relative: string;
  functionName: string;
  fingerprint: string;
  sourceFingerprint: string;
  kind: string;
  expression: string;
};

function unresolvedConsumerInventory(
  relative: string,
  source: string,
): UnresolvedConsumerInventoryEntry[] {
  const sourceFingerprint = createHash("sha256")
    .update(normalizedExecutableRust(source))
    .digest("hex");
  return unresolvedSqlConsumers(source).map((consumer) => {
    const owner = enclosingRustFunctionRange(source, consumer.start);
    const modulePath = rustModuleScopeAt(source, consumer.start).path;
    const functionName = owner
      ? [modulePath, owner.name].filter(Boolean).join("::")
      : modulePath || "<module>";
    const executableBody = owner
      ? source.slice(owner.start, owner.end)
      : source;
    return {
      relative,
      functionName,
      fingerprint: createHash("sha256")
        .update(normalizedExecutableRust(executableBody))
        .digest("hex"),
      sourceFingerprint,
      kind: consumer.kind,
      expression: normalized(consumer.expression),
    };
  });
}

function unresolvedConsumerIdentity(
  consumer: UnresolvedConsumerInventoryEntry,
): string {
  return [
    consumer.relative,
    consumer.functionName,
    consumer.fingerprint,
    consumer.sourceFingerprint,
    consumer.kind,
    consumer.expression,
  ].join(":");
}

function transactionControlSql(source: string): string[] {
  const controls = new Set([
    "BEGIN",
    "COMMIT",
    "END",
    "ROLLBACK",
    "SAVEPOINT",
    "RELEASE",
  ]);
  return sqlConsumerPrograms(source).flatMap((program) =>
    splitSqlStatements(program.text).flatMap((statement) => {
      const keyword = statement.match(/^([A-Za-z_][A-Za-z0-9_]*)/)?.[1].toUpperCase();
      return keyword && controls.has(keyword) ? [keyword] : [];
    }));
}

function foreignKeyDisableQueries(source: string): Array<{
  kind: string;
  sql: string;
  start: number;
  end: number;
}> {
  const disablesForeignKeys = (statement: string): boolean => {
    const tokens = sqlTokens(statement);
    if (tokens[0]?.upper !== "PRAGMA") return false;
    let index = 1;
    if (tokens[index + 1]?.text === ".") index += 2;
    if (tokens[index]?.value.toLowerCase() !== "foreign_keys") return false;
    index += 1;
    let parenthesized = false;
    let setting = false;
    if (tokens[index]?.text === "=") {
      setting = true;
      index += 1;
    }
    if (tokens[index]?.text === "(") {
      setting = true;
      parenthesized = true;
      index += 1;
    }
    if (!setting) return false;
    let sign = "";
    if (tokens[index]?.text === "+" || tokens[index]?.text === "-") {
      sign = tokens[index].text;
      index += 1;
    }
    const value = tokens[index];
    if (!value) return false;
    index += 1;
    if (parenthesized) {
      if (tokens[index]?.text !== ")") return false;
      index += 1;
    }
    if (index !== tokens.length) return false;
    const exactValue = value.value.toUpperCase();
    const normalizedValue = value.kind === "number"
      ? exactValue.replaceAll("_", "")
      : exactValue;
    const candidate = `${sign}${normalizedValue}`;
    if (["ON", "TRUE", "YES"].includes(exactValue)) return false;
    const hex = candidate.match(/^\+?0X([0-9A-F]+)$/);
    if (hex) return !/[1-9A-F]/.test(hex[1]);
    const decimal = candidate.match(/^\+?([0-9]+)(?:\.[0-9]*)?(?:E[+-]?[0-9]+)?$/);
    if (decimal) return Number.parseInt(decimal[1], 10) === 0;
    return true;
  };
  const literals = decodedRustStringLiterals(source);
  const concats = concatLiteralPrograms(source, literals);
  const consumers = [
    ...sqlxQueryLiterals(source),
    ...executorSqlPrograms(source, literals, concats).map((program) => ({
      kind: "executor",
      sql: program.text,
      start: program.start,
      end: program.end,
    })),
    ...queryBuilderPrograms(source, literals, concats).map((program) => ({
      kind: "query_builder",
      sql: program.text,
      start: program.start,
      end: program.end,
    })),
  ];
  return consumers.filter(({ sql }) =>
    splitSqlStatements(sql).some(disablesForeignKeys));
}

function borrowedParticipantCalls(source: string, participant: string): string[] {
  return maskRustComments(source, true).match(
    new RegExp(`\\b${participant}\\s*\\(\\s*&mut\\s*\\*transaction\\b`, "g"),
  ) ?? [];
}

const sqlTableOperationCache = new Map<string, SqlTableOperation[]>();
const allLiteralSqlTableOperationCache = new Map<string, SqlTableOperation[]>();

function assignedSqlxQueryConsumerEnd(
  source: string,
  syntax: string,
  call: { start: number; end: number },
): number | undefined {
  const functionScope = enclosingRustFunctionRange(source, call.start);
  if (!functionScope) return undefined;
  const statementStart = Math.max(
    syntax.lastIndexOf(";", call.start),
    syntax.lastIndexOf("{", call.start),
  ) + 1;
  const name = syntax.slice(statementStart, call.start).match(
    /\blet\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)(?:\s*:[^=;\n]+)?\s*=\s*$/,
  )?.[1];
  if (!name) return undefined;
  const assignmentEnd = syntax.indexOf(";", call.end);
  if (assignmentEnd < 0 || assignmentEnd >= functionScope.end) return undefined;
  const escapedName = name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const suffix = syntax.slice(assignmentEnd + 1, functionScope.end);
  const use = new RegExp(
    `\\b${escapedName}\\s*\\.\\s*(?:bind|fetch|fetch_all|fetch_one|fetch_optional|execute)\\s*\\(`,
  ).exec(suffix);
  if (!use || use.index === undefined) return undefined;
  const reassignment = new RegExp(
    `\\b(?:let\\s+(?:mut\\s+)?${escapedName}\\b|${escapedName}\\s*=)`,
  ).exec(suffix);
  if (
    reassignment?.index !== undefined
    && reassignment.index < use.index
  ) {
    return undefined;
  }
  const useStart = assignmentEnd + 1 + use.index;
  const statementEnd = syntax.indexOf(";", useStart);
  return statementEnd < 0 || statementEnd >= functionScope.end
    ? functionScope.end
    : statementEnd + 1;
}

function sqlTableOperationsFromPrograms(
  source: string,
  programs: readonly SqlProgram[],
  cache: Map<string, SqlTableOperation[]>,
): SqlTableOperation[] {
  const cached = cache.get(source);
  if (cached) return cached;
  const syntax = maskRustComments(source, true);
  const queryCalls = sqlQueryCallBounds(source);
  const operations = programs.flatMap((program) => {
    return sqlTableReferences(program.text).map((reference) => {
      const offset = program.offsets[reference.sqlOffset] ?? program.start;
      const call = queryCalls.find(({ start, end }) => offset >= start && offset < end);
      const statementEnd = call ? syntax.indexOf(";", call.end) : -1;
      const functionEnd = call
        ? enclosingRustFunctionRange(source, call.start)?.end ?? call.end
        : program.end;
      const nextCallStart = call
        ? queryCalls.find(({ start }) => start > call.end)?.start ?? functionEnd
        : program.end;
      const assignedConsumerEnd = call
        ? assignedSqlxQueryConsumerEnd(source, syntax, call)
        : undefined;
      return {
        operation: reference.operation,
        table: reference.table,
        offset,
        callStart: call?.start ?? -1,
        callEnd: call?.end ?? -1,
        consumerStart: call?.start ?? program.start,
        consumerEnd: call
          ? assignedConsumerEnd ?? (statementEnd >= 0 && statementEnd < functionEnd
            ? Math.min(statementEnd + 1, nextCallStart)
            : Math.min(functionEnd, nextCallStart))
          : program.end,
        sql: program.text,
        sqlOffset: reference.sqlOffset,
      };
    });
  });
  const seen = new Set<string>();
  const selected = operations.filter((operation) => {
    const key = [
      operation.operation,
      operation.table,
      operation.offset,
      operation.callStart,
      operation.callEnd,
      operation.consumerStart,
      operation.consumerEnd,
    ].join(":");
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  });
  cache.set(source, selected);
  return selected;
}

function sqlTableOperations(source: string): SqlTableOperation[] {
  return sqlTableOperationsFromPrograms(
    source,
    sqlConsumerPrograms(source),
    sqlTableOperationCache,
  );
}

function allLiteralSqlTableOperations(source: string): SqlTableOperation[] {
  return sqlTableOperationsFromPrograms(
    source,
    allLiteralSqlPrograms(source),
    allLiteralSqlTableOperationCache,
  );
}

function assertNegativeSentinelIdentifiers(source: string, operation: SqlTableOperation): void {
  if (operation.callStart < 0 || operation.callEnd < 0) {
    throw new Error(`foreign SQL operation at ${operation.offset} is not enclosed by sqlx::query(...)`);
  }
  const statement = maskSqlComments(operation.sql).slice(operation.sqlOffset);
  const identifier = (column: string): boolean => /^(?:id|[A-Za-z_][A-Za-z0-9_]*_id)$/i.test(column)
    && column.toLowerCase() !== "external_id";
  if (operation.operation === "INSERT INTO") {
    const insert = statement.match(/^INSERT\s+INTO\s+(?:["`\[]?[A-Za-z_][A-Za-z0-9_]*["`\]]?\s*\.\s*)?["`\[]?[A-Za-z_][A-Za-z0-9_]*["`\]]?\s*\(([^)]*)\)\s*VALUES\s*((?:\([^)]*\)\s*,?\s*)+)/is);
    if (!insert) throw new Error("foreign INSERT has no parseable column and VALUES tuples");
    const columns = insert[1].split(",").map((value) => value.trim().replace(/^["`\[]|["`\]]$/g, ""));
    const tuples = [...insert[2].matchAll(/\(([^)]*)\)/g)].map((match) => match[1].split(",").map((value) => value.trim()));
    if (tuples.length === 0) throw new Error("foreign INSERT has no VALUES tuples");
    const sentinelColumns = columns.map((column, index) => identifier(column) ? index : -1).filter((index) => index >= 0);
    if (sentinelColumns.length === 0) throw new Error("foreign INSERT has no sentinel identifier columns");
    for (const values of tuples) {
      if (values.length !== columns.length) throw new Error("foreign INSERT columns and values differ");
      for (const index of sentinelColumns) {
        if (!/^-\d+$/.test(values[index])) {
          throw new Error(`foreign INSERT ${columns[index]} is not a negative sentinel`);
        }
      }
    }
    return;
  }
  if (operation.operation === "DELETE FROM") {
    throw new Error("foreign DELETE is not authorized by the frozen sentinel setup");
  }
  throw new Error(`foreign ${operation.operation} is not an allowed sentinel mutation`);
}

function preRestoreControlFlow(source: string): string[] {
  return maskRustComments(source, true).match(
    /\?|\.(?:expect|unwrap)\s*\(|\b(?:return|break|continue|if|match|loop|while|for)\b|\b(?:panic|assert|assert_eq|assert_ne|debug_assert|debug_assert_eq|debug_assert_ne|unreachable|todo|unimplemented)!\s*\(|\b(?:std::)?process::(?:exit|abort)\s*\(/g,
  ) ?? [];
}

function sqlTableTokens(source: string): string[] {
  return sqlTableOperations(source).map(({ table }) => table);
}

function foreignOperationsOutsideAllowedBodies(
  source: string,
  allowedBodies: Array<{ start: number; end: number }>,
  foreignTables: ReadonlySet<string>,
): SqlTableOperation[] {
  return allLiteralSqlTableOperations(source).filter((operation) =>
    foreignTables.has(operation.table)
    && (operation.callStart < 0 || !allowedBodies.some(({ start, end }) =>
      operation.callStart >= start
      && operation.offset >= start
      && operation.callEnd <= end)),
  );
}

const negativeRustNumberSource = String.raw`-\s*(?:0[xX][0-9A-Fa-f_]+|0[oO][0-7_]+|0[bB][01_]+|[0-9][0-9_]*)(?:_?[A-Za-z][A-Za-z0-9_]*)?(?![A-Za-z0-9_])`;

function staticNegativeRustFactories(source: string): Set<string> {
  const syntax = maskRustComments(source, true);
  const negativeTail = new RegExp(`^\\s*${negativeRustNumberSource}\\s*$`);
  const negativeReturn = new RegExp(
    `^\\s*return\\s+${negativeRustNumberSource}\\s*;\\s*$`,
  );
  const factories = new Set<string>();
  for (const match of syntax.matchAll(
    /\bfn\s+([A-Za-z_][A-Za-z0-9_]*)\s*(?:<[^>{}]*>)?\s*\(/g,
  )) {
    if (match.index === undefined) throw new Error("negative factory has no offset");
    const paramsOpen = match.index + match[0].lastIndexOf("(");
    const paramsClose = closingDelimiter(syntax, paramsOpen, "(", ")");
    if (syntax.slice(paramsOpen + 1, paramsClose).trim() !== "") continue;
    const bodyOpen = syntax.indexOf("{", paramsClose);
    const declarationEnd = syntax.indexOf(";", paramsClose);
    if (bodyOpen < 0 || (declarationEnd >= 0 && declarationEnd < bodyOpen)) continue;
    const bodyClose = closingDelimiter(syntax, bodyOpen, "{", "}");
    const body = syntax.slice(bodyOpen + 1, bodyClose);
    if (negativeTail.test(body) || negativeReturn.test(body)) factories.add(match[1]);
  }
  return factories;
}

function negativeForeignReadOperations(
  source: string,
  foreignTables: ReadonlySet<string>,
): SqlTableOperation[] {
  const sourceSyntax = maskRustComments(source, true);
  const negativeBindings = [...sourceSyntax.matchAll(new RegExp(
      `\\b(?:const|static|let)\\s+(?:mut\\s+)?([A-Za-z_][A-Za-z0-9_]*)\\s*(?::[^=;\\n]+)?=\\s*${negativeRustNumberSource}`,
      "g",
    ))].map((match) => {
      if (match.index === undefined) throw new Error("negative binding has no offset");
      return {
        name: match[1],
        scope: enclosingRustFunctionRange(source, match.index),
      };
    });
  const aliases = [...sourceSyntax.matchAll(
    /\b(?:const|static|let)\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*(?::[^=;\n]+)?=\s*([A-Za-z_][A-Za-z0-9_]*)\s*;/g,
  )].map((match) => {
    if (match.index === undefined) throw new Error("negative alias has no offset");
    return {
      name: match[1],
      target: match[2],
      scope: enclosingRustFunctionRange(source, match.index),
    };
  });
  const negativeFactories = staticNegativeRustFactories(source);
  return sqlTableOperations(source).filter((operation) => {
    if (!foreignTables.has(operation.table) || !["FROM", "JOIN"].includes(operation.operation)) {
      return false;
    }
    const negativeSqlLiteral = new RegExp(
      `(^|[^A-Za-z0-9_])${negativeRustNumberSource}`,
    ).test(maskSqlComments(operation.sql));
    const operationScope = enclosingRustFunctionRange(source, operation.offset);
    const sameScope = (scope: RustFunctionRange | undefined): boolean =>
      scope === undefined
      || (
        scope.start === operationScope?.start
        && scope.end === operationScope?.end
      );
    const negativeNames = new Set(
      negativeBindings.filter(({ scope }) => sameScope(scope))
        .map(({ name }) => name),
    );
    for (let changed = true; changed;) {
      changed = false;
      for (const { name, target, scope } of aliases) {
        if (
          sameScope(scope)
          && !negativeNames.has(name)
          && negativeNames.has(target)
        ) {
          negativeNames.add(name);
          changed = true;
        }
      }
    }
    const consumerSyntax = maskRustComments(
      source.slice(operation.consumerStart, operation.consumerEnd),
      true,
    );
    const negativeBoundLiteral = new RegExp(
      `\\.(?:bind|push_bind)\\s*\\(\\s*${negativeRustNumberSource}`,
    ).test(consumerSyntax);
    const negativeNamedBind = [...negativeNames].some((name) =>
      new RegExp(`\\.(?:bind|push_bind)\\s*\\(\\s*${name}\\s*\\)`).test(
        consumerSyntax,
      ));
    const negativeFactoryBind = [...negativeFactories].some((name) =>
      new RegExp(
        `\\.(?:bind|push_bind)\\s*\\(\\s*${name}\\s*\\(\\s*\\)\\s*\\)`,
      ).test(consumerSyntax));
    return negativeSqlLiteral
      || negativeBoundLiteral
      || negativeNamedBind
      || negativeFactoryBind;
  });
}

type RustAsyncFunction = {
  signature: string;
  body: string;
};

function rustAsyncFunction(
  source: string,
  name: string,
  label: string,
): RustAsyncFunction {
  const syntax = maskRustComments(source, true);
  const declaration = new RegExp(
    `\\b(?:pub(?:\\([^)]*\\))?\\s+)?async\\s+fn\\s+${name}\\s*\\(`,
    "g",
  );
  const matches = [...syntax.matchAll(declaration)];
  expect(matches, `${label} declaration count`).toHaveLength(1);
  const match = matches[0];
  if (!match || match.index === undefined) throw new Error(`missing ${label}`);
  const open = match.index + match[0].lastIndexOf("(");
  const close = closingDelimiter(syntax, open, "(", ")");
  const brace = syntax.indexOf("{", close);
  if (brace < 0) throw new Error(`missing ${label} opening brace`);
  const end = closingDelimiter(syntax, brace, "{", "}");
  return {
    signature: normalized(source.slice(match.index, brace)),
    body: source.slice(brace + 1, end),
  };
}

function directSqlTerminals(source: string): string[] {
  const syntax = maskRustComments(source, true);
  return [...syntax.matchAll(/\.(fetch_all|fetch_one|fetch_optional|execute)\s*\(/g)]
    .map((match) => {
      if (match.index === undefined) throw new Error("SQL terminal has no offset");
      const open = match.index + match[0].lastIndexOf("(");
      const close = closingDelimiter(syntax, open, "(", ")");
      return `${match[1]}(${normalized(source.slice(open + 1, close))})`;
    })
    .sort();
}

function awaitedNamedCalls(source: string, names: readonly string[]): string[] {
  const syntax = maskRustComments(source, true);
  return names.flatMap((name) =>
    [...syntax.matchAll(new RegExp(`\\b${name}\\s*\\(`, "g"))].flatMap((match) => {
      if (match.index === undefined) throw new Error(`${name} call has no offset`);
      const open = match.index + match[0].lastIndexOf("(");
      const close = closingDelimiter(syntax, open, "(", ")");
      return /^\s*\.await\b/.test(syntax.slice(close + 1))
        ? [`${name}(${normalized(source.slice(open + 1, close))})`]
        : [];
    }),
  ).sort();
}

function awaitSiteCount(source: string): number {
  return maskRustComments(source, true).match(/\.await\b/g)?.length ?? 0;
}

function unqualifiedFreeFunctionCalls(source: string): string[] {
  const syntax = maskRustComments(source, true);
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
    if (match.index === undefined) throw new Error("free-function call has no offset");
    const previous = syntax.slice(0, match.index).match(/(\S)\s*$/)?.[1];
    const name = match[2];
    if (previous === "." || previous === ":" || keywords.has(name)) return [];
    return [name];
  });
}

function qualifiedCallablePaths(source: string): string[] {
  const syntax = maskRustComments(source, true);
  const constructorNames = new Set(["new", "with_capacity"]);
  return [...syntax.matchAll(
    /\b((?:(?:r#)?[A-Za-z_][A-Za-z0-9_]*\s*::\s*)+(?:r#)?[a-z_][A-Za-z0-9_]*)\b/g,
  )].flatMap((match) => {
    if (match.index === undefined) throw new Error("qualified callable path has no offset");
    const after = syntax.slice(match.index + match[0].length).match(/^\s*(.)/)?.[1];
    const selected = match[1]
      .replace(/\s*::\s*/g, "::")
      .replaceAll("r#", "");
    const terminal = selected.split("::").at(-1)!;
    return after === "!" || constructorNames.has(terminal) ? [] : [selected];
  });
}

function detachedCoordinatorBypasses(source: string): string[] {
  const syntax = maskRustComments(source, true);
  const findings = new Set<string>();
  if (/\b(?:spawn|spawn_blocking|spawn_local|block_in_place)\s*\(/.test(syntax)) {
    findings.add("spawn");
  }
  if (/\bblock_on\s*\(/.test(syntax)) findings.add("block_on");
  if (/\bdetached\s*\(/.test(syntax)) findings.add("detached");
  if (/\b(?:join|try_join|select)\s*!/.test(syntax)) findings.add("join/select");
  if (
    /\bpool\s*\.\s*(?:clone|to_owned)\s*\(/.test(syntax)
    || /\b(?:Clone\s*::\s*clone|ToOwned\s*::\s*to_owned)\s*\(\s*&?\s*pool\b/.test(syntax)
  ) findings.add("pool.clone");
  if (
    /\basync\s+(?:move\s*)?\{/.test(syntax)
    || /\b(?:std\s*::\s*)?mem\s*::\s*forget\s*\(/.test(syntax)
    || /\.\s*detach\s*\(/.test(syntax)
  ) findings.add("detached-future");
  return [...findings].sort();
}

function asyncTopology(source: string, childNames: readonly string[]): string[] {
  const syntax = maskRustComments(source, true);
  const tokens: Array<{ offset: number; value: string }> = [];
  for (const match of syntax.matchAll(/\bpool\s*\.\s*begin\s*\(\s*\)\s*\.await\b/g)) {
    if (match.index === undefined) throw new Error("transaction begin has no offset");
    tokens.push({ offset: match.index, value: "begin" });
  }
  for (const match of syntax.matchAll(/\btransaction\s*\.\s*commit\s*\(\s*\)\s*\.await\b/g)) {
    if (match.index === undefined) throw new Error("transaction commit has no offset");
    tokens.push({ offset: match.index, value: "commit" });
  }
  for (const name of childNames) {
    for (const match of syntax.matchAll(new RegExp(`\\b${name}\\s*\\(`, "g"))) {
      if (match.index === undefined) throw new Error(`${name} call has no offset`);
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
  for (const match of syntax.matchAll(/\.(fetch_all|fetch_one|fetch_optional|execute)\s*\(/g)) {
    if (match.index === undefined) throw new Error("SQL terminal has no offset");
    const open = match.index + match[0].lastIndexOf("(");
    const close = closingDelimiter(syntax, open, "(", ")");
    tokens.push({
      offset: match.index,
      value: `${match[1]}(${normalized(source.slice(open + 1, close))})`,
    });
  }
  return tokens.sort((left, right) => left.offset - right.offset).map(({ value }) => value);
}

function rustTestBodies(source: string): Array<{ name: string; body: string; start: number; end: number }> {
  const syntax = maskRustComments(source, true);
  return [...syntax.matchAll(
    /#\[\s*(?:(?:tokio|sqlx)\s*::\s*)?test\b/g,
  )].flatMap((attribute) => {
    if (attribute.index === undefined) throw new Error("test attribute has no location");
    const bracket = attribute.index + attribute[0].indexOf("[");
    const attributeEnd = closingDelimiter(syntax, bracket, "[", "]");
    const declaration = /(?:async\s+)?fn\s+([A-Za-z0-9_]+)\s*\([^)]*\)[^{]*\{/.exec(
      syntax.slice(attributeEnd + 1, attributeEnd + 241),
    );
    if (!declaration || declaration.index === undefined) return [];
    const declarationStart = attributeEnd + 1 + declaration.index;
    const brace = declarationStart + declaration[0].lastIndexOf("{");
    const end = closingDelimiter(syntax, brace, "{", "}");
    return [{
      name: declaration[1],
      body: source.slice(brace + 1, end),
      start: brace + 1,
      end,
    }];
  });
}

function frozenMoves(name: "wholeMoves" | "splitMoves"): FrozenMove[] {
  const plan = readFileSync(
    path.join(repoRoot, "docs/superpowers/plans/2026-07-22-extractum-analysis-extraction.md"),
    "utf8",
  );
  const start = plan.indexOf(`$${name} = @(`);
  if (start < 0) throw new Error(`missing frozen $${name} plan map`);
  const end = plan.indexOf("\n)", start);
  if (end < 0) throw new Error(`unterminated frozen $${name} plan map`);
  const entries = [...plan.slice(start, end).matchAll(
    /@\('src-tauri\/src\/analysis\/([^']+)', 'src-tauri\/crates\/extractum-analysis\/src\/([^']+)'\)/g,
  )].map((match) => ({ before: match[1], after: match[2] }));
  if (entries.length === 0) throw new Error(`no entries in frozen $${name} plan map`);
  return entries;
}

function frozenMoveSource(move: FrozenMove): string {
  return readAnalysisContractSource({
    before: move.before,
    after: { owner: "crate", path: move.after },
  });
}

const analysisSources = rustFiles(currentAnalysisRoot)
  .map((file) => readFileSync(file, "utf8"))
  .join("\n");
const projectSources = rustFiles(path.join(repoRoot, "src-tauri/src/projects"))
  .map((file) => readFileSync(file, "utf8"))
  .join("\n");

describe("analysis application boundary", () => {
  it("normalizes checkout line endings before matching and fingerprinting", () => {
    const lf = "fn query() {\n  let sql = r#\"SELECT\n1\"#;\n}\n";
    const crlf = lf.replaceAll("\n", "\r\n");
    const cr = lf.replaceAll("\n", "\r");

    expect(normalizeAnalysisContractSourceText(crlf)).toBe(lf);
    expect(normalizeAnalysisContractSourceText(cr)).toBe(lf);
    expect(normalizedExecutableRust(crlf)).toBe(normalizedExecutableRust(lf));
    expect(normalizedExecutableRust(cr)).toBe(normalizedExecutableRust(lf));
  });

  it("detects transaction controls through imported sqlx query aliases", () => {
    const source = `
      use sqlx::{query as statement, raw_sql as batch};
      async fn run(conn: &mut SqliteConnection) -> Result<()> {
        const CONTROL: &str = concat!("COM", "MIT");
        statement(CONTROL).execute(&mut *conn).await?;
        batch("PRAGMA foreign_keys = OFF").execute(&mut *conn).await?;
        Ok(())
      }
    `;

    expect(transactionControlSql(source)).toEqual(["COMMIT"]);
    expect(foreignKeyDisableQueries(source).map(({ sql }) => normalized(sql))).toEqual([
      "PRAGMA foreign_keys = OFF",
    ]);
  });

  it("detects executor SQL through static literals and factories", () => {
    const source = `
      fn rollback_sql() -> &'static str {
        concat!("ROLL", "BACK")
      }
      fn disable_foreign_keys_sql() -> &'static str {
        const SETTING: &str = "PRAGMA foreign_keys = OFF";
        SETTING
      }
      async fn run(conn: &mut SqliteConnection) -> Result<()> {
        const CONTROL: &str = rollback_sql();
        conn.execute(CONTROL).await?;
        conn.execute(disable_foreign_keys_sql()).await?;
        Ok(())
      }
    `;

    expect(transactionControlSql(source)).toEqual(["ROLLBACK"]);
    expect(foreignKeyDisableQueries(source).map(({ sql }) => normalized(sql))).toEqual([
      "PRAGMA foreign_keys = OFF",
    ]);
  });

  it("detects supported static SQL consumers without comment or quoted-value decoys", () => {
    const source = `
      use sqlx::{query as statement, query_scalar as scalar, QueryBuilder};
      const SAVE: &str = concat!("SAVE", "POINT imported_alias");
      async fn run(conn: &mut SqliteConnection) -> Result<()> {
        statement(SAVE).execute(&mut *conn).await?;
        scalar("SELECT 'ROLLBACK', \\"PRAGMA foreign_keys = OFF\\"")
          .fetch_one(&mut *conn)
          .await?;
        let mut control = QueryBuilder::<Sqlite>::new("REL");
        control.push_unseparated("EASE imported_alias");
        let pragma_tail = concat!("keys", " = OFF");
        let mut pragma = QueryBuilder::<Sqlite>::new("PRAGMA foreign_");
        pragma.push_unseparated(pragma_tail);
        // conn.execute("COMMIT").await?;
        let dead = "PRAGMA foreign_keys = OFF";
        Ok(())
      }
    `;

    expect(transactionControlSql(source)).toEqual(["SAVEPOINT", "RELEASE"]);
    expect(foreignKeyDisableQueries(source).map(({ sql }) => normalized(sql))).toEqual([
      "PRAGMA foreign_keys = OFF",
    ]);
  });

  it("associates imported QueryBuilder and separated-handle fragments", () => {
    const source = `
      use sqlx::QueryBuilder as QB;
      async fn probe(pool: &SqlitePool) {
        let mut query = QB::<Sqlite>::new("SELECT * FROM ");
        let mut parts = query.separated(", ");
        parts.push_unseparated("sources");
        query.build().fetch_all(pool).await;

        let mut pragma = QB::<Sqlite>::new("PRAGMA foreign_");
        let mut pragma_parts = pragma.separated("");
        const SETTING: &str = concat!("keys", " = OFF");
        pragma_parts.push_unseparated(SETTING);
        pragma.build().execute(pool).await;
      }
    `;

    expect(sqlTableOperations(source).map(({ table }) => table)).toEqual(["sources"]);
    expect(foreignKeyDisableQueries(source).map(({ sql }) => normalized(sql))).toEqual([
      "PRAGMA foreign_keys = OFF",
    ]);
  });

  it("detects executable consumers through explicit sqlx module aliases", () => {
    const source = `
      use sqlx as db;
      async fn probe(pool: &SqlitePool) {
        db::query("BEGIN").execute(pool).await;
        db::query_as::<_, Row>("SELECT * FROM sources").fetch_all(pool).await;
        db::query_scalar("PRAGMA foreign_keys = OFF").fetch_one(pool).await;
        db::raw_sql(concat!("COM", "MIT")).execute(pool).await;
        other::query("ROLLBACK").execute(pool).await;
        let dead = "db::query(\\"SAVEPOINT hidden\\")";
      }
    `;

    expect(transactionControlSql(source)).toEqual(["BEGIN", "COMMIT"]);
    expect(sqlTableOperations(source).map(({ table }) => table)).toEqual(["sources"]);
    expect(foreignKeyDisableQueries(source).map(({ sql }) => normalized(sql))).toEqual([
      "PRAGMA foreign_keys = OFF",
    ]);
  });

  it("reports unresolved executable SQL consumers fail closed", () => {
    const source = `
      use sqlx::QueryBuilder;
      async fn probe(pool: &SqlitePool, borrowed: &str, fragment: &str) {
        let sql = format!("SELECT * FROM {}", "analysis_runs");
        sqlx::query(&sql).execute(pool).await;
        sqlx::query(borrowed).execute(pool).await;

        let mut query = QueryBuilder::<Sqlite>::new(format!("SELECT {}", "*"));
        query.push(fragment);
        let separator = ", ";
        let mut parts = query.separated(&separator);
        parts.push_unseparated(" FROM sources");
        query.build().execute(pool).await;
      }
    `;

    expect(unresolvedSqlConsumers(source).map(
      ({ kind, expression }) => `${kind}:${normalized(expression)}`,
    )).toEqual([
      "query:&sql",
      "query:borrowed",
      "query_builder_new:format!(\"SELECT {}\", \"*\")",
      "query_builder_push:fragment",
      "query_builder_separator:&separator",
    ]);
  });

  it("reports unresolved direct Executor SQL without classifying query terminals", () => {
    const source = `
      use sqlx::QueryBuilder;
      async fn probe(
        conn: &mut SqliteConnection,
        dynamic_sql: &str,
      ) -> Result<()> {
        conn.execute(dynamic_sql).await?;
        sqlx::query("SELECT 1").execute(&mut *conn).await?;
        let mut builder = QueryBuilder::<Sqlite>::new("SELECT 2");
        builder.build().execute(&mut *conn).await?;
        let statement = sqlx::query("SELECT 3");
        statement.execute(&mut *conn).await?;
        Ok(())
      }
    `;

    expect(unresolvedSqlConsumers(source).map(
      ({ kind, expression }) => `${kind}:${normalized(expression)}`,
    )).toEqual(["executor:dynamic_sql"]);
  });

  it("uses the latest lexical receiver producer for Executor classification", () => {
    const source = `
      async fn probe(
        conn: &mut SqliteConnection,
        pool: &SqlitePool,
        dynamic_sql: &str,
      ) -> Result<()> {
        let statement = sqlx::query("SELECT 1");
        drop(statement);
        let statement = conn;
        statement.execute(dynamic_sql).await?;

        let mut reusable = sqlx::query("SELECT 2");
        reusable = sqlx::query("SELECT 3");
        reusable.execute(pool).await?;
        Ok(())
      }
    `;

    expect(unresolvedSqlConsumers(source).map(
      ({ kind, expression }) => `${kind}:${normalized(expression)}`,
    )).toEqual(["executor:dynamic_sql"]);
  });

  it("tracks temporary separated chains and module-aliased QueryBuilder", () => {
    const source = `
      use sqlx as db;
      async fn probe(
        pool: &SqlitePool,
        separator: &str,
        fragment: &str,
      ) {
        let mut pragma = db::QueryBuilder::<Sqlite>::new("PRAGMA foreign_");
        pragma.separated("").push_unseparated("keys = OFF");
        pragma.build().execute(pool).await;

        let mut dynamic = db::QueryBuilder::<Sqlite>::new("SELECT ");
        dynamic.separated(separator).push_unseparated(fragment);
        dynamic.build().execute(pool).await;

        let mut fake = other::QueryBuilder::<Sqlite>::new("PRAGMA foreign_");
        fake.separated("").push_unseparated("keys = OFF");
        fake.build().execute(pool).await;
      }
    `;

    expect(foreignKeyDisableQueries(source).map(({ sql }) => normalized(sql))).toEqual([
      "PRAGMA foreign_keys = OFF",
    ]);
    expect(unresolvedSqlConsumers(source).map(
      ({ kind, expression }) => `${kind}:${normalized(expression)}`,
    )).toEqual([
      "query_builder_separator:separator",
      "query_builder_push:fragment",
      "executor:pool",
    ]);
  });

  it("keeps sqlx module aliases inside their lexical Rust module", () => {
    const source = `
      mod first {
        async fn before(pool: &SqlitePool) {
          db::query("BEGIN").execute(pool).await;
        }
        use sqlx as db;
      }
      mod second {
        async fn unrelated(pool: &SqlitePool) {
          db::query("ROLLBACK").execute(pool).await;
        }
      }
      mod third {
        use sqlx::{self as database};
        async fn supported(pool: &SqlitePool) {
          database::raw_sql("COMMIT").execute(pool).await;
        }
      }
      mod fourth {
        use sqlx::{query as run, QueryBuilder as QB};
        async fn imported(pool: &SqlitePool) {
          run("SAVEPOINT imported").execute(pool).await;
          QB::<Sqlite>::new("RELEASE imported").build().execute(pool).await;
        }
      }
      mod fifth {
        async fn unrelated_imports(pool: &SqlitePool) {
          run("ROLLBACK").execute(pool).await;
          QB::<Sqlite>::new("BEGIN").build().execute(pool).await;
        }
      }
    `;

    expect(transactionControlSql(source)).toEqual([
      "BEGIN",
      "COMMIT",
      "SAVEPOINT",
      "RELEASE",
    ]);
  });

  it("bounds bare QueryBuilder recognition to its lexical import scope", () => {
    const source = `
      mod sql_owner {
        use sqlx::QueryBuilder;
        fn build() {
          QueryBuilder::<Sqlite>::new("BEGIN");
        }
      }
      mod local_owner {
        struct QueryBuilder;
        fn build() {
          QueryBuilder::<Sqlite>::new("ROLLBACK");
          fake::QueryBuilder::<Sqlite>::new("COMMIT");
        }
      }
    `;

    expect(transactionControlSql(source)).toEqual(["BEGIN"]);
  });

  it("keeps block-local sqlx imports inside their exact brace scope", () => {
    const source = `
      async fn probe(pool: &SqlitePool) {
        {
          use sqlx::QueryBuilder;
          QueryBuilder::<Sqlite>::new("BEGIN");
        }
        {
          struct QueryBuilder<T>(T);
          QueryBuilder::<Sqlite>::new("ROLLBACK");
        }
        {
          use sqlx::query as statement;
          statement("SAVEPOINT scoped");
        }
        {
          let statement = local_query;
          statement("COMMIT");
        }
        {
          use sqlx as db;
          db::raw_sql("RELEASE scoped");
        }
        {
          let db = local_module;
          db::raw_sql("ROLLBACK");
        }
        let _ = pool;
      }
    `;

    expect(transactionControlSql(source).sort()).toEqual([
      "BEGIN",
      "RELEASE",
      "SAVEPOINT",
    ]);
  });

  it("follows assigned query variables and scopes negative locals by function", () => {
    const source = `
      async fn variable_query(pool: &SqlitePool) {
        let statement = sqlx::query(
          "SELECT * FROM sources WHERE id = ?",
        );
        statement.bind(-7_i64).fetch_all(pool).await;
      }
      async fn negative_name_owner() {
        let sentinel = -8_i64;
      }
      async fn ordinary_query(pool: &SqlitePool, sentinel: i64) {
        sqlx::query("SELECT * FROM items WHERE source_id = ?")
          .bind(sentinel)
          .fetch_all(pool)
          .await;
      }
    `;

    expect(negativeForeignReadOperations(
      source,
      new Set(["items", "sources"]),
    ).map(({ table }) => table)).toEqual(["sources"]);
  });

  it("fingerprints unresolved allowances by enclosing function semantics", () => {
    const source = `
      fn helper_mode() -> i64 { 1 }
      async fn producer(pool: &SqlitePool, dynamic_sql: &str) {
        let mode = 1;
        sqlx::query(dynamic_sql).execute(pool).await;
      }
    `;
    const inventory = unresolvedConsumerInventory("probe.rs", source);
    expect(inventory).toHaveLength(1);
    expect(inventory[0]?.functionName).toBe("producer");
    expect(inventory[0]?.fingerprint).toMatch(/^[0-9a-f]{64}$/);
    expect(inventory[0]?.sourceFingerprint).toMatch(/^[0-9a-f]{64}$/);
    expect(
      unresolvedConsumerInventory(
        "probe.rs",
        source.replace("let mode = 1;", "let mode = 2;"),
      )[0]?.fingerprint,
    ).not.toBe(inventory[0]?.fingerprint);
    expect(
      unresolvedConsumerInventory(
        "probe.rs",
        source.replace("producer", "moved_producer"),
      )[0]?.functionName,
    ).toBe("moved_producer");
    const helperDrift = unresolvedConsumerInventory(
      "probe.rs",
      source.replace("helper_mode() -> i64 { 1 }", "helper_mode() -> i64 { 2 }"),
    )[0];
    expect(helperDrift?.fingerprint).toBe(inventory[0]?.fingerprint);
    expect(helperDrift?.sourceFingerprint).not.toBe(
      inventory[0]?.sourceFingerprint,
    );
  });

  it("parses the executable migration CREATE TABLE inventory", () => {
    const migration = `
      -- CREATE TABLE comment_decoy(id INTEGER);
      SELECT 'CREATE VIRTUAL TABLE quoted_value_decoy USING fts5(body)';
      CREATE TABLE IF EXISTS malformed(id INTEGER);
      CREATE TABLE main.;
      CREATE TABLE main.analysis_runs(id INTEGER);
      CREATE TABLE IF NOT EXISTS "audit"."event log"(id INTEGER);
      CREATE VIRTUAL TABLE temp.\`search_docs\` USING fts5(body);
      CREATE VIRTUAL TABLE IF NOT EXISTS [main].[search items] USING fts5(body);
    `;

    expect(migrationCreatedTables(migration)).toEqual([
      "analysis_runs",
      "event log",
      "search_docs",
      "search items",
    ]);
  });

  it("requires executable SQL consumers for positive owned-table evidence", () => {
    const deadStrings = `
      const DEAD_SQL: &str = "SELECT * FROM analysis_runs";
      tracing::error!("DELETE FROM analysis_chat_messages");
      anyhow::anyhow!("SELECT * FROM analysis_prompt_templates");
    `;
    const executable = `
      const OWNED_SQL: &str = concat!("SELECT * FROM analysis_", "runs");
      sqlx::query(OWNED_SQL).fetch_all(&pool).await?;
    `;

    expect(sqlTableOperations(deadStrings)).toEqual([]);
    expect(allLiteralSqlTableOperations(deadStrings).map(({ table }) => table)).toEqual([
      "analysis_runs",
      "analysis_chat_messages",
      "analysis_prompt_templates",
    ]);
    expect(sqlTableOperations(executable).map(({ table }) => table)).toEqual([
      "analysis_runs",
    ]);
  });

  it("recognizes argument-bearing Tokio and SQLx test attributes", () => {
    const source = `
      #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
      async fn flavored_tokio() {}

      #[sqlx::test]
      async fn default_sqlx(pool: SqlitePool) {}

      #[sqlx::test(fixtures(path = "./fixtures", scripts("users", "posts")))]
      async fn fixture_sqlx(pool: SqlitePool) {}
    `;

    expect(rustTestBodies(source).map(({ name }) => name)).toEqual([
      "flavored_tokio",
      "default_sqlx",
      "fixture_sqlx",
    ]);
  });

  it("recognizes statically negative zero-argument bind factories", () => {
    const source = `
      use sqlx::QueryBuilder;
      fn source_sentinel() -> i64 { -7_i64 }
      fn item_sentinel() -> i64 {
        return -0x8_i64;
      }
      async fn probe(pool: &SqlitePool) {
        sqlx::query("SELECT * FROM sources WHERE id = ?")
          .bind(source_sentinel())
          .fetch_all(pool)
          .await;
        let mut query = QueryBuilder::<Sqlite>::new(
          "SELECT * FROM items WHERE source_id = ",
        );
        query.push_bind(item_sentinel());
        query.build().fetch_all(pool).await;
      }
    `;

    expect(negativeForeignReadOperations(
      source,
      new Set(["items", "sources"]),
    ).map(({ table }) => table).sort()).toEqual(["items", "sources"]);
  });

  it("associates negative sentinel binds with their own SQL consumer", () => {
    const source = `
      use sqlx::QueryBuilder;
      async fn probe(pool: &SqlitePool, ordinary_id: i64) {
        sqlx::query("SELECT * FROM sources WHERE id = ?")
          .bind(ordinary_id)
          .fetch_all(pool)
          .await;
        sqlx::query("SELECT 1")
          .bind(-7_i64)
          .fetch_one(pool)
          .await;

        let mut foreign = QueryBuilder::<Sqlite>::new(
          "SELECT * FROM items WHERE source_id = ",
        );
        foreign.push_bind(ordinary_id);
        foreign.build().fetch_all(pool).await;
        let mut unrelated = QueryBuilder::<Sqlite>::new("SELECT ");
        unrelated.push_bind(-8_i64);
        unrelated.build().fetch_one(pool).await;
      }
    `;

    expect(negativeForeignReadOperations(
      source,
      new Set(["items", "sources"]),
    )).toEqual([]);
  });

  it("uses a manifest-keyed fail-closed dual-owner source selector", () => {
    expect(isAnalysisCrateExtracted()).toBe(analysisExtracted);
    expect(readAppAnalysisSource("mod.rs")).toContain("mod chat;");
    expect(
      readAnalysisContractSource({
        before: "mod.rs",
        after: { owner: "app", path: "mod.rs" },
      }),
    ).toContain("mod chat;");
    if (analysisExtracted) {
      expect(readCrateAnalysisSource("lib.rs")).toContain("mod chat;");
    } else {
      expect(() => readCrateAnalysisSource("lib.rs")).toThrow(
        /before extraction/,
      );
    }
    expect(() => readAppAnalysisSource("../lib.rs")).toThrow(/escapes selected root/);
    expect(() => readAppAnalysisSource("missing.rs")).toThrow(/is missing/);
  });

  it("stages checkpoint four portable engines through exact unsuffixed includes", () => {
    const chatAdapter = readAppAnalysisSource("chat.rs");
    const reportAdapter = readAppAnalysisSource("report.rs");
    const lifecycleAdapter = readAppAnalysisSource("report/lifecycle.rs");
    const testRoot = readAppAnalysisSource("report/tests/mod.rs");
    const portableTestRoot = readAnalysisContractSource({
      before: "report/tests/mod_portable.rs",
      after: { owner: "crate", path: "report/tests/mod.rs" },
    });

    if (analysisExtracted) {
      expect(chatAdapter).not.toMatch(/\binclude!\s*\(/);
      expect(reportAdapter).not.toMatch(/\binclude!\s*\(/);
      expect(lifecycleAdapter).not.toMatch(/\binclude!\s*\(/);
      expect(normalized(testRoot)).toBe("mod capture;");
      expect(readCrateAnalysisSource("report.rs")).toMatch(
        /(?:^|\n)mod lifecycle;/,
      );
      expect(readCrateAnalysisSource("report.rs")).toMatch(
        /#\[cfg\(test\)\]\s*mod tests;/,
      );
    } else {
      expect(chatAdapter.match(/include!\("chat_engine\.rs"\);/g) ?? [])
        .toHaveLength(1);
      expect(reportAdapter.match(/include!\("report_engine\.rs"\);/g) ?? [])
        .toHaveLength(1);
      expect(
        lifecycleAdapter.match(/include!\("lifecycle_portable\.rs"\);/g) ?? [],
      ).toHaveLength(1);
      expect(testRoot.match(/include!\("mod_portable\.rs"\);/g) ?? [])
        .toHaveLength(1);
      expect(normalized(testRoot)).toBe(
        'mod capture; include!("mod_portable.rs");',
      );
    }
    expect(normalized(portableTestRoot)).toBe(
      "mod architecture; mod corpus_port; mod harness; mod lifecycle; mod phases; mod preflight; mod requests; mod runtime; mod scope;",
    );

    for (const [adapter, suffixedModule] of [
      [chatAdapter, "chat_engine"],
      [reportAdapter, "report_engine"],
      [lifecycleAdapter, "lifecycle_portable"],
      [testRoot, "mod_portable"],
    ] as const) {
      expect(adapter).not.toMatch(new RegExp(`\\bmod\\s+${suffixedModule}\\s*;`));
    }
    expect(portableTestRoot).not.toMatch(/\bmod\s+capture\s*;/);

    expect(chatAdapter).toContain("pub async fn ask_analysis_run_question");
    expect(chatAdapter).toContain("get_pool(&handle)");
    expect(chatAdapter).toContain(
      "use super::store::resolve_legacy_analysis_chat_run_in_pool;",
    );
    expect(portableChatSource()).not.toContain(
      "resolve_legacy_analysis_chat_run_in_pool",
    );
    expect(reportAdapter).toContain("pub(crate) async fn start_analysis_report_run");
    expect(reportAdapter).toContain("get_pool(&handle)");
    expect(reportAdapter).not.toMatch(/\bimpl\s+RunEvent\s*\{/);
    expect(reportAdapter).not.toContain("emit_analysis_event");
    expect(lifecycleAdapter).toContain("pub async fn cleanup_interrupted_analysis_runs");
    expect(lifecycleAdapter).toContain("get_pool");
    expect(lifecycleAdapter).toContain(
      "let sink = TauriAnalysisEventSink::new(handle.clone());",
    );
    expect(lifecycleAdapter).toContain(
      analysisExtracted
        ? "request_analysis_run_cancel_in_pool(&pool, state, scheduler, &sink, run_id).await"
        : "super::request_analysis_run_cancel_in_pool(&pool, state, scheduler, &sink, run_id).await",
    );
    expect(lifecycleAdapter).not.toContain(".publish(&sink);");
    expect(lifecycleAdapter).not.toContain(".emit(handle)");
    expect(portableReportSource()).toContain(
      "pub async fn request_analysis_run_cancel_in_pool(",
    );
    expect(portableReportSource()).toContain(
      '.message("Cancelling analysis run...".to_string())',
    );
    expect(portableReportSource()).toContain(".publish(sink);");
  });

  it("keeps checkpoint four portable subjects application-free and depth-stable", () => {
    const forbiddenApplicationCapability =
      /\bAppHandle\b|\bget_pool\b|crate::(?:analysis|error|compression|llm|time|db|sources)\b/;
    const portableSources = [
      portableChatSource(),
      readAnalysisContractSource({
        before: "models.rs",
        after: { owner: "crate", path: "models.rs" },
      }),
      portableReportSource(),
      portableReportLifecycleSource(),
      readAnalysisContractSource({
        before: "report/capture.rs",
        after: { owner: "crate", path: "report/capture.rs" },
      }),
      readAnalysisContractSource({
        before: "report/phases.rs",
        after: { owner: "crate", path: "report/phases.rs" },
      }),
      readAnalysisContractSource({
        before: "report/requests.rs",
        after: { owner: "crate", path: "report/requests.rs" },
      }),
      readAnalysisContractSource({
        before: "report/tests/corpus_port.rs",
        after: { owner: "crate", path: "report/tests/corpus_port.rs" },
      }),
      readAnalysisContractSource({
        before: "report/tests/mod_portable.rs",
        after: { owner: "crate", path: "report/tests/mod.rs" },
      }),
      readAnalysisContractSource({
        before: "report/tests/runtime.rs",
        after: { owner: "crate", path: "report/tests/runtime.rs" },
      }),
      ...[
        "harness",
        "lifecycle",
        "phases",
        "preflight",
        "requests",
        "scope",
      ].map((leaf) =>
        readAnalysisContractSource({
          before: `report/tests/${leaf}.rs`,
          after: { owner: "crate", path: `report/tests/${leaf}.rs` },
        })),
      readAnalysisContractSource({
        before: "state.rs",
        after: { owner: "crate", path: "state.rs" },
      }),
    ];

    for (const source of portableSources) {
      expect(source).not.toMatch(forbiddenApplicationCapability);
    }
    expect(portableChatSource()).toContain("pub async fn execute_analysis_chat");
    expect(portableReportSource()).toContain("pub async fn execute_analysis_report");
    expect(portableReportSource()).not.toMatch(
      /#\[path\s*=\s*"report\/lifecycle(?:_portable)?\.rs"\]/,
    );
    expect(portableReportSource()).not.toContain('#[path = "report/lifecycle.rs"]');
    expect(portableReportSource()).not.toContain(
      "pub use self::lifecycle::cleanup_interrupted_analysis_runs",
    );
    expect(portableReportSource()).not.toMatch(
      /pub(?:\(crate\))?\s+use\s+self::lifecycle::(?:\{[^}]*\brequest_analysis_run_cancel\b|request_analysis_run_cancel\s*;)/,
    );
    expect(portableReportLifecycleSource()).toContain(
      "pub(super) async fn request_analysis_run_cancel_for_pool",
    );
    expect(readAppAnalysisSource("report.rs")).toContain(
      "pub use self::lifecycle::cleanup_interrupted_analysis_runs",
    );
    expect(readAppAnalysisSource("report.rs")).toContain(
      '#[path = "report/lifecycle.rs"]',
    );
    expect(readAppAnalysisSource("report.rs")).toContain(
      "pub(crate) use self::lifecycle::request_analysis_run_cancel",
    );
  });

  it("keeps the Tauri analysis event sink synchronous and side-effect minimal", () => {
    const events = readAppAnalysisSource("events.rs");
    expect(events).not.toContain("emit_analysis_event");
    const implementation = uniqueRustBracedBody(
      events,
      /(?:^|\n)\s*impl\s+AnalysisEventSink\s+for\s+TauriAnalysisEventSink\s*\{/,
      "TauriAnalysisEventSink implementation",
    );
    const publishRun = uniqueRustBracedBody(
      implementation,
      /(?:^|\n)\s*fn\s+publish_run\(\s*&self,\s*event:\s*AnalysisRunEvent\s*\)\s*\{/,
      "TauriAnalysisEventSink::publish_run",
    );
    const publishChat = uniqueRustBracedBody(
      implementation,
      /(?:^|\n)\s*fn\s+publish_chat\(\s*&self,\s*event:\s*AnalysisChatEvent\s*\)\s*\{/,
      "TauriAnalysisEventSink::publish_chat",
    );
    const forbiddenSideEffect =
      /\basync\b|\.await\b|\bblock_on\b|\bget_pool\b|\bSqlitePool\b|\bsqlx\b|\b(?:SELECT|INSERT|UPDATE|DELETE)\b|\.acquire\s*\(|\.begin\s*\(|\bsleep\b|\bretry\b|\bbackoff\b|\b(?:Mutex|RwLock)\b|\.lock\s*\(|\b(?:channel|mpsc|oneshot|broadcast|watch)\b|\b(?:spawn|spawn_blocking|JoinSet|join)\b|\b(?:std|tokio)::(?:fs|net)\b|\b(?:File|OpenOptions|TcpStream|TcpListener|UdpSocket|reqwest|hyper)\b/;

    expect(normalized(publishRun)).toBe(
      "let _ = self.handle.emit(ANALYSIS_RUN_EVENT, &event);",
    );
    expect(normalized(publishChat)).toBe(
      "let _ = self.handle.emit(ANALYSIS_CHAT_EVENT, &event);",
    );
    for (const [label, body] of [
      ["publish_run", publishRun],
      ["publish_chat", publishChat],
    ] as const) {
      expect(body.match(/\.emit\s*\(/g) ?? [], `${label} emit count`).toHaveLength(
        1,
      );
      expect(body, `${label} forbidden side effect`).not.toMatch(
        forbiddenSideEffect,
      );
    }
    expect(normalized(implementation)).toBe(
      "fn publish_run(&self, event: AnalysisRunEvent) { let _ = self.handle.emit(ANALYSIS_RUN_EVENT, &event); } fn publish_chat(&self, event: AnalysisChatEvent) { let _ = self.handle.emit(ANALYSIS_CHAT_EVENT, &event); }",
    );
  });

  it("exposes only the opaque fixture cancellation wait capability", () => {
    const state = readAnalysisContractSource({
      before: "state.rs",
      after: { owner: "crate", path: "state.rs" },
    });
    const fixtures = readAppAnalysisSource("fixtures.rs");
    const waitFields = state.match(
      /pub struct AnalysisReportCancellationWait\s*\{([\s\S]*?)\n\}/,
    )?.[1];

    for (const signature of [
      "pub async fn insert_active_report_run(&self, run_id: i64)",
      "pub async fn remove_active_report_run(&self, run_id: i64)",
      "pub async fn active_report_run_ids(&self) -> HashSet<i64>",
      "pub async fn request_report_run_cancel(&self, run_id: i64) -> bool",
    ]) {
      expect(state).toContain(signature);
    }
    expect(state).toMatch(
      /pub async fn prepare_report_run_cancellation_wait\(\s*&self,\s*run_id: i64,\s*\) -> Option<AnalysisReportCancellationWait>/,
    );
    expect(state).toContain("pub async fn cancelled(self)");
    expect(waitFields).toBeDefined();
    expect(waitFields).not.toMatch(
      /^\s*pub(?:\([^)]*\))?\s+[a-z_][a-z0-9_]*\s*:/m,
    );
    expect(state).not.toMatch(
      /#\[derive\((?=[^\]]*\bSerialize\b)[^\]]*\)\]\s*pub struct AnalysisReportCancellationWait/,
    );
    expect(state).not.toMatch(/pub\s+fn\s+(?:token|into_token)\b/);
    expect(state).toContain(
      "pub(crate) async fn report_run_child_token(&self, run_id: i64)",
    );
    expect(state).toContain("async fn ensure_report_run_token(&self, run_id: i64)");

    expectOrdered(fixtures.slice(fixtures.indexOf("async fn spawn_fixture_cancellation_waiters")), [
      "prepare_report_run_cancellation_wait(run_id).await",
      "tauri::async_runtime::spawn(async move",
      "cancellation_wait.cancelled().await",
    ]);
    expect(fixtures).not.toContain(".report_run_child_token(run_id)");
  });

  it("freezes the executable Appendix A partition at 95 crate and 48 app identities", () => {
    const frozen = parseAppendix();
    const actual = executableAnalysisIdentities();
    const expectedFrozen = frozen.map((identity) =>
      analysisExtracted ? identity.final : identity.current);
    const expectedAdded = planAddedTests.map((identity) =>
      analysisExtracted ? identity.final : identity.current);
    const expected = [...expectedFrozen, ...expectedAdded].sort();

    expect(actual.map(({ identity }) => identity)).toEqual(expected);
    expect(actual).toHaveLength(169);
    expect(expectedFrozen).toHaveLength(143);
    expect(expectedAdded).toHaveLength(26);
    expect(frozen.filter(({ owner }) => owner === "crate")).toHaveLength(95);
    expect(frozen.filter(({ owner }) => owner === "app")).toHaveLength(48);
    expect(planAddedTests.filter(({ owner }) => owner === "crate"))
      .toHaveLength(17);
    expect(planAddedTests.filter(({ owner }) => owner === "app"))
      .toHaveLength(9);
    const expectedOwners = new Map([
      ...frozen.map((identity) => [
        analysisExtracted ? identity.final : identity.current,
        analysisExtracted ? identity.owner : "app",
      ] as const),
      ...planAddedTests.map((identity) => [
        analysisExtracted ? identity.final : identity.current,
        analysisExtracted ? identity.owner : "app",
      ] as const),
    ]);
    for (const identity of actual) {
      expect(identity.owner, identity.identity).toBe(
        expectedOwners.get(identity.identity),
      );
    }
    expect(new Set(expectedFrozen).size).toBe(143);
    expect(new Set(expectedAdded).size).toBe(26);
    expect(expectedFrozen.filter((identity) => expectedAdded.includes(identity)))
      .toEqual([]);
    expect(new Set(frozen.map(({ final }) => final)).size).toBe(143);
  });

  it("manages one scheduler Arc and routes every app consumer through it", () => {
    const appProduction = (relativePath: string) =>
      readFileSync(path.join(repoRoot, "src-tauri/src", relativePath), "utf8")
        .split("#[cfg(test)]")[0];
    const appSchedulerConsumers = [
      "accounts.rs",
      "diagnostics/mod.rs",
      "llm/mod.rs",
      "prompt_packs/runtime_commands.rs",
      "analysis/chat.rs",
      "analysis/report.rs",
      "analysis/report/lifecycle.rs",
      "analysis/report_commands.rs",
    ].map(appProduction);
    const portableSchedulerConsumers = [
      portableChatSource(),
      portableReportSource(),
      readAnalysisContractSource({
        before: "report/phases.rs",
        after: { owner: "crate", path: "report/phases.rs" },
      }),
      portableReportLifecycleSource(),
    ].map((source) => source.split("#[cfg(test)]")[0]);
    const allConsumers = [
      ...appSchedulerConsumers,
      ...portableSchedulerConsumers,
    ].join("\n");

    expect(appSchedulerConsumers).toHaveLength(8);
    expect(portableSchedulerConsumers).toHaveLength(4);

    expect(
      appLib.match(/\.manage\(Arc::new\(LlmSchedulerState::new\(\)\)\)/g) ?? [],
    ).toHaveLength(1);
    expect(appLib).not.toContain(".manage(LlmSchedulerState::new())");
    expect(allConsumers).not.toMatch(
      /(?:tauri::)?State<'_,\s*LlmSchedulerState\s*>/,
    );
    expect(allConsumers).not.toContain("state::<LlmSchedulerState>()");
    expect(allConsumers).not.toContain("Arc::new(LlmSchedulerState::new())");

    for (const requiredConsumer of [
      "llm_scheduler: tauri::State<'_, Arc<LlmSchedulerState>>",
      "state: tauri::State<'_, Arc<LlmSchedulerState>>",
      "scheduler: State<'_, Arc<LlmSchedulerState>>",
      "scheduler: tauri::State<'_, Arc<LlmSchedulerState>>",
      "state::<Arc<LlmSchedulerState>>()",
    ]) {
      expect(allConsumers).toContain(requiredConsumer);
    }
  });

  it("stages checkpoint two portable values and safe construction without app compression ownership", () => {
    const moduleSource = readAppAnalysisSource("mod.rs");
    const domainSource = readAnalysisContractSource({
      before: "domain_portable.rs",
      after: { owner: "crate", path: "domain.rs" },
    });
    const models = readAnalysisContractSource({
      before: "models.rs",
      after: { owner: "crate", path: "models.rs" },
    });
    const report = portableReportSource();
    const trace = readAnalysisContractSource({
      before: "trace.rs",
      after: { owner: "crate", path: "trace.rs" },
    });
    const filters = readAnalysisContractSource({
      before: "store/owned_read_model.rs",
      after: { owner: "crate", path: "store/read_model.rs" },
    });
    const reportCommands = readAppAnalysisSource("report_commands.rs");

    if (analysisExtracted) {
      const crateRoot = readCrateAnalysisSource("lib.rs");
      expect(moduleSource).not.toMatch(/tests_portable|domain_portable/);
      expect(crateRoot).toMatch(/(?:^|\n)mod domain;/);
      expect(crateRoot).toMatch(/#\[cfg\(test\)\]\s*mod tests;/);
    } else {
      expect(moduleSource.match(/include!\("tests_portable\.rs"\)/g) ?? [])
        .toHaveLength(1);
      expect(moduleSource).toContain('#[path = "domain_portable.rs"]');
    }
    expect(moduleSource).toContain('const ANALYSIS_RUN_EVENT: &str = "analysis://run"');
    expect(moduleSource).toContain('const ANALYSIS_CHAT_EVENT: &str = "analysis://chat"');
    expect(moduleSource).not.toContain('const TEMPLATE_KIND_REPORT: &str = "report"');
    expect(domainSource).toContain('pub(crate) const TEMPLATE_KIND_REPORT: &str = "report"');
    expect(domainSource).toContain("pub(crate) use extractum_core::time::now_secs");

    expect(trace).toContain("compression::{compress_json_bytes, decompress_bytes}");
    expect(trace).not.toMatch(/\bzstd::/);
    expectOrdered(report, [
      "let reduce_result = run_reduce_phase",
      "let compressed_trace = compress_trace_data",
      "Some(&reduce_result.completion.text)",
      "Some(&compressed_trace)",
    ]);

    for (const constructor of ["from_command", "for_source", "for_source_group", "for_project"]) {
      expect(report).toContain(`pub fn ${constructor}(`);
    }
    const requestFields = report.match(
      /(?:^|\n)pub struct StartAnalysisReportRequest \{([\s\S]*?)\n\}/,
    )?.[1];
    expect(requestFields).toBeDefined();
    expect(requestFields).not.toMatch(/\bpub(?:\(crate\))?\s+\w+\s*:/);
    expect(reportCommands.indexOf("StartAnalysisReportRequest::from_command"))
      .toBeLessThan(reportCommands.indexOf("start_analysis_report_run"));
    expect(models).toContain("pub struct ResolvedAnalysisScope");
    expect(models).toContain("pub enum AnalysisScopeKind");
    expect(models).toContain("pub enum AnalysisSourceKind");
    expect(models).not.toMatch(/pub\s+source_ids:\s*Vec<i64>/);
    expect(filters).toContain("pub fn for_analysis(");
    expect(filters).toContain("pub fn for_project(project_id: i64, limit: i64) -> Self");
    expect(filters).toContain("pub fn foreign_label_search_terms(&self) -> &[String]");
  });

  it("freezes opaque report tickets and the explicit execution API", () => {
    const report = portableReportSource();
    const capture = readAnalysisContractSource({
      before: "report/capture.rs",
      after: { owner: "crate", path: "report/capture.rs" },
    });
    const ticketNames = [
      "AnalysisReportPreparationTicket",
      "AnalysisReportScopeTicket",
      "AnalysisReportExecutionTicket",
    ];

    for (const ticketName of ticketNames) {
      const fields = report.match(
        new RegExp(`(?:^|\\n)pub struct ${ticketName} \\{([\\s\\S]*?)\\n\\}`),
      )?.[1];
      expect(fields, `${ticketName} declaration`).toBeDefined();
      expect(fields).not.toMatch(
        /^\s*pub(?:\([^)]*\))?\s+[a-z_][a-z0-9_]*\s*:/m,
      );
      expect(report).not.toMatch(
        new RegExp(
          `#\\[derive\\((?=[^\\]]*\\bSerialize\\b)[^\\]]*\\)\\]\\s*pub struct ${ticketName}`,
        ),
      );
    }
    expect(report).not.toMatch(
      /#\[derive\((?=[^\]]*\bDebug\b)[^\]]*\)\]\s*pub struct AnalysisReportExecutionTicket/,
    );

    for (const accessor of [
      "pub fn requested_profile_id(&self) -> Option<&str>",
      "pub fn model_override(&self) -> Option<&str>",
      "pub fn resolve_youtube_corpus_mode(self) -> AppResult<AnalysisReportScopeTicket>",
      "pub fn scope_kind(&self) -> AnalysisScopeKind",
      "pub fn scope_id(&self) -> i64",
      "pub fn youtube_corpus_mode(&self) -> YoutubeCorpusMode",
      "pub fn run_id(&self) -> i64",
    ]) {
      expect(report).toContain(accessor);
    }

    const executionError = report.match(
      /pub enum AnalysisExecutionError \{([\s\S]*?)\n\}/,
    )?.[1];
    expect(executionError).toBeDefined();
    expect(
      [...(executionError ?? "").matchAll(/^\s*([A-Z][A-Za-z]+)\(/gm)].map(
        ([, variant]) => variant,
      ),
    ).toEqual(["Cancelled", "CaptureFailed", "Failed"]);

    for (const signature of [
      /pub async fn prepare_analysis_report\(\s*pool: &SqlitePool,\s*request: StartAnalysisReportRequest,\s*\) -> AppResult<AnalysisReportPreparationTicket>/,
      /pub async fn prepare_analysis_report_execution\(\s*pool: &SqlitePool,\s*state: &AnalysisState,\s*reader: &dyn AnalysisCorpusReader,\s*preparation: AnalysisReportScopeTicket,\s*scope: ResolvedAnalysisScope,\s*resolved_profile: ResolvedLlmProfile,\s*effective_model: String,\s*model_input_token_limit: Option<usize>,\s*\) -> AppResult<AnalysisReportExecutionTicket>/,
      /pub async fn execute_analysis_report\(\s*pool: &SqlitePool,\s*state: &AnalysisState,\s*scheduler: Arc<LlmSchedulerState>,\s*reader: &dyn AnalysisCorpusReader,\s*sink: Arc<dyn AnalysisEventSink>,\s*ticket: AnalysisReportExecutionTicket,\s*\) -> Result<\(\), AnalysisExecutionError>/,
      /pub async fn finalize_analysis_report_execution\(\s*pool: Option<&SqlitePool>,\s*state: &AnalysisState,\s*sink: &dyn AnalysisEventSink,\s*run_id: i64,\s*outcome: Result<\(\), AnalysisExecutionError>,\s*\)/,
    ]) {
      expect(report).toMatch(signature);
    }
    expect(report).toContain("pub use self::capture::capture_analysis_corpus");
    expect(capture).toMatch(
      /pub async fn capture_analysis_corpus\(\s*pool: &SqlitePool,\s*reader: &dyn AnalysisCorpusReader,\s*run_id: i64,\s*scope_label: &str,\s*request: &AnalysisCorpusRequest,\s*\) -> Result<Vec<AnalysisCorpusMessage>, AnalysisExecutionError>/,
    );
  });

  it("freezes opaque chat tickets and the explicit execution API", () => {
    const chat = portableChatSource();
    const chatAdapter = readAppAnalysisSource("chat.rs");
    const requestFields = chat.match(
      /(?:^|\n)pub struct AskAnalysisRunQuestionRequest \{([\s\S]*?)\n\}/,
    )?.[1];
    expect(requestFields).toBeDefined();
    expect(
      [...(requestFields ?? "").matchAll(/^\s+([a-z_]+):\s*([^,]+),$/gm)]
        .map(([, name, type]) => `${name}: ${type}`),
    ).toEqual([
      "run_id: i64",
      "question: String",
      "model_override: Option<String>",
      "profile_id: Option<String>",
    ]);
    expect(requestFields).not.toMatch(
      /^\s*pub(?:\([^)]*\))?\s+[a-z_][a-z0-9_]*\s*:/m,
    );

    for (const ticketName of [
      "AskAnalysisRunQuestionRequest",
      "AnalysisChatExecutionTicket",
      "AnalysisChatCompletionTicket",
    ]) {
      expect(chat).not.toMatch(
        new RegExp(
          `#\\[derive\\((?=[^\\]]*\\bSerialize\\b)[^\\]]*\\)\\]\\s*pub struct ${ticketName}`,
        ),
      );
    }
    for (const accessor of [
      "pub fn request_id(&self) -> &str",
      "pub fn profile_id(&self) -> &str",
      "pub fn run_id(&self) -> i64",
    ]) {
      expect(chat).toContain(accessor);
    }

    for (const signature of [
      /pub fn new\(\s*run_id: i64,\s*question: String,\s*model_override: Option<String>,\s*profile_id: Option<String>,\s*\) -> AppResult<Self>/,
      /pub async fn prepare_analysis_chat\(\s*pool: &SqlitePool,\s*request: AskAnalysisRunQuestionRequest,\s*run: AnalysisChatRun,\s*\) -> AppResult<AnalysisChatExecutionTicket>/,
      /pub async fn execute_analysis_chat\(\s*scheduler: Arc<LlmSchedulerState>,\s*sink: Arc<dyn AnalysisEventSink>,\s*ticket: AnalysisChatExecutionTicket,\s*resolved_profile: ResolvedLlmProfile,\s*\) -> Result<AnalysisChatCompletionTicket, AnalysisExecutionError>/,
      /pub async fn complete_analysis_chat\(\s*pool: &SqlitePool,\s*sink: &dyn AnalysisEventSink,\s*completion: AnalysisChatCompletionTicket,\s*\) -> AppResult<\(\)>/,
      /pub fn publish_analysis_chat_execution_error\(\s*sink: &dyn AnalysisEventSink,\s*request_id: &str,\s*run_id: i64,\s*error: &AnalysisExecutionError,\s*\)/,
      /pub fn publish_analysis_chat_persistence_error\(\s*sink: &dyn AnalysisEventSink,\s*request_id: &str,\s*run_id: i64,\s*error: &AppError,\s*\)/,
    ]) {
      expect(chat).toMatch(signature);
    }

    const command = chatAdapter.slice(
      chatAdapter.indexOf("pub async fn ask_analysis_run_question"),
    );
    expect(command).toContain(
      "resolve_legacy_analysis_chat_run_in_pool(&pool, run_id)",
    );
    expect(command).not.toMatch(/\brequest\.run_id\b/);
    expectOrdered(command, [
      "AskAnalysisRunQuestionRequest::new(",
      "get_pool(&handle)",
      "resolve_legacy_analysis_chat_run_in_pool",
      "prepare_analysis_chat(",
      "let request_id = ticket.request_id().to_string()",
      "let profile_id = ticket.profile_id().to_string()",
      "tokio::spawn(async move",
      "resolve_profile_for_backend",
      "execute_analysis_chat(",
      "get_pool(&app_handle)",
      "complete_analysis_chat(",
      "Ok(request_id)",
    ]);
    expect(command.match(/tokio::spawn\(async move/g) ?? []).toHaveLength(1);
    expect(command).not.toContain(".run_request(");
    expect(command).not.toContain("persist_chat_exchange(");
    expect(command).not.toContain(".emit(");
  });

  it("keeps analysis run-list filters public with private state", () => {
    const filters = readAnalysisContractSource({
      before: "store/owned_read_model.rs",
      after: { owner: "crate", path: "store/read_model.rs" },
    });
    const filterFields = filters.match(
      /pub(?:\([^)]*\))?\s+struct AnalysisRunListFilters\s*\{([\s\S]*?)\n\}/,
    )?.[1];
    const filterImpl = filters.match(
      /impl AnalysisRunListFilters\s*\{([\s\S]*?)\n\}\n\nconst ANALYSIS_RUN_LIST_SELECT/,
    )?.[1];
    const privateFields = filterFields ?? "";

    expect(filters).toMatch(/(?:^|\n)pub struct AnalysisRunListFilters\s*\{/);
    expect(filters).not.toMatch(
      /(?:^|\n)pub\((?:crate|super)\)\s+struct AnalysisRunListFilters\s*\{/,
    );
    expect(filterFields).toBeDefined();
    expect(privateFields).not.toMatch(
      /^\s*pub(?:\([^)]*\))?\s+[a-z_][a-z0-9_]*\s*:/m,
    );
    expect(
      [...privateFields.matchAll(/^\s*([a-z_][a-z0-9_]*)\s*:/gm)].map(
        ([, field]) => field,
      ),
    ).toEqual([
      "source_id",
      "source_group_id",
      "project_id",
      "limit",
      "query",
      "status",
      "provider",
      "model",
      "template",
      "date_from",
      "date_to",
      "foreign_label_search_terms",
    ]);
    expect(filters).not.toMatch(
      /#\[derive\((?=[^\]]*\bDefault\b)[^\]]*\)\]\s*pub struct AnalysisRunListFilters/,
    );
    expect(filters).not.toMatch(
      /impl\s+Default\s+for\s+AnalysisRunListFilters/,
    );
    expect(filterImpl).toBeDefined();
    expect(
      [...(filterImpl ?? "").matchAll(/^\s*pub fn ([a-z_][a-z0-9_]*)/gm)].map(
        ([, method]) => method,
      ),
    ).toEqual([
      "for_analysis",
      "for_project",
      "foreign_label_search_terms",
    ]);
  });

  it("requires analysis run-list tests to use the curated filter constructors", () => {
    const uncheckedConstructions = [
      "store/tests/read_model.rs",
      "tests_application.rs",
    ].flatMap((owner) => {
      const source = readAppAnalysisSource(owner);
      return [...source.matchAll(/AnalysisRunListFilters \{/g)].map((match) => {
        const line = source.slice(0, match.index).split("\n").length;
        return `${owner}:${line}`;
      });
    });

    expect(uncheckedConstructions).toEqual([]);
  });

  it("keeps project data range on typed app scope without analysis SQL helper imports", () => {
    const dataRange = readFileSync(
      path.join(repoRoot, "src-tauri/src/projects/data_range.rs"),
      "utf8",
    );
    const analysisFacade = readAppAnalysisSource("mod.rs");
    const corpusFacade = readAppAnalysisSource("corpus.rs");

    expect(dataRange).toContain("YoutubeCorpusMode::from_wire");
    expect(dataRange).toContain("resolve_analysis_sources(pool, None, None, Some(project_id))");
    expect(dataRange).toContain("resolved.scope()");
    expect(dataRange).toContain(
      "AnalysisSourceResolutionErrorCode::NoLinkedYoutubeVideos",
    );
    const productionDataRange = dataRange.split("#[cfg(test)]")[0];
    expect(productionDataRange).not.toMatch(/message\.(?:contains|starts_with)\(/);
    expect(dataRange).toContain("fn push_analysis_document_kind_predicate(");
    expect(dataRange).toContain("d.source_type = 'telegram'");
    expect(dataRange).toContain("d.source_type = 'youtube'");
    expect(dataRange).toContain("youtube_corpus_mode.includes_description()");
    expect(dataRange).toContain("youtube_corpus_mode.includes_comments()");
    expect(dataRange).not.toContain("push_analysis_document_kind_filter");
    expect(dataRange).not.toMatch(/use\s+crate::analysis::[^;]*push_analysis_document_kind_filter/);
    expect(corpusFacade).not.toMatch(/pub\(crate\)\s+use[^;]*push_analysis_document_kind_filter/);
    expect(analysisFacade).not.toMatch(/pub\(crate\)\s+use[^;]*push_analysis_document_kind_filter/);
  });

  it("stages checkpoint three corpus and foreign-label boundaries under original identities", () => {
    const corpus = readAnalysisContractSource({
      before: "corpus_portable.rs",
      after: { owner: "crate", path: "corpus.rs" },
    });
    const corpusAdapter = readAppAnalysisSource("corpus.rs");
    const liveAdapter = readAppAnalysisSource("corpus/live.rs");
    const scopeAdapter = readAppAnalysisSource("corpus/source_resolution.rs");
    const ownedReadModel = readAnalysisContractSource({
      before: "store/owned_read_model.rs",
      after: { owner: "crate", path: "store/read_model.rs" },
    });
    const appReadModel = readAppAnalysisSource("store/read_model.rs");
    const storeFacade = readAppAnalysisSource("store.rs");
    const appReadModelTests = readAppAnalysisSource("store/tests/read_model.rs");
    const portableReadModelTests = readAnalysisContractSource({
      before: "store/tests/read_model_portable.rs",
      after: { owner: "crate", path: "store/tests/read_model.rs" },
    });
    const portableHarness = readAnalysisContractSource({
      before: "corpus/tests/harness_portable.rs",
      after: { owner: "crate", path: "corpus/tests/harness.rs" },
    });
    const appHarness = readAppAnalysisSource("corpus/tests/harness.rs");
    const portableLeaves = [
      readAnalysisContractSource({
        before: "corpus/tests/live_portable.rs",
        after: { owner: "crate", path: "corpus/tests/live.rs" },
      }),
      readAnalysisContractSource({
        before: "corpus/tests/preflight_portable.rs",
        after: { owner: "crate", path: "corpus/tests/preflight.rs" },
      }),
      readAnalysisContractSource({
        before: "corpus/tests/source_resolution_portable.rs",
        after: { owner: "crate", path: "corpus/tests/source_resolution.rs" },
      }),
      portableReadModelTests,
    ];

    expect(corpus).toContain("pub trait AnalysisCorpusReader: Send + Sync + 'static");
    expect(corpus).toContain("pub struct AnalysisCorpusRequest");
    expect(corpus).toContain("pub struct AnalysisCorpusMessage");
    expect(corpus).toContain("pub async fn preflight_analysis_corpus(");
    expect(corpusAdapter).toContain("pub(crate) struct AppAnalysisCorpusReader");
    expect(corpusAdapter).toContain("impl AnalysisCorpusReader for AppAnalysisCorpusReader");
    expect(liveAdapter).toContain("pub(crate) async fn load_app_corpus_messages(");
    expect(scopeAdapter).toContain("pub(crate) struct AppAnalysisScopeResolution");
    expect(scopeAdapter).toContain("pub(crate) fn into_scope(self) -> ResolvedAnalysisScope");

    for (const participant of [
      "prepare_analysis_run_summaries",
      "prepare_active_analysis_run_summaries",
      "prepare_analysis_run_detail",
      "prepare_legacy_analysis_chat_run",
    ]) {
      expect(ownedReadModel).toMatch(
        new RegExp(
          `pub async fn ${participant}\\(\\s*conn: &mut SqliteConnection,`,
        ),
      );
      expect(storeFacade).not.toContain(participant);
    }
    expect(ownedReadModel).not.toMatch(/\bJOIN\s+sources\b/i);
    expect(ownedReadModel).not.toMatch(/\bJOIN\s+projects\b/i);
    for (const coordinator of [
      "list_analysis_runs_in_pool",
      "list_active_analysis_runs_in_pool",
      "get_analysis_run_in_pool",
      "resolve_legacy_analysis_chat_run_in_pool",
    ]) {
      expect(appReadModel).toContain(`async fn ${coordinator}(`);
    }
    expect(appReadModel).toContain("pool.begin().await");
    expect(appReadModel).toContain("transaction.commit().await");

    if (analysisExtracted) {
      expect(appReadModelTests).not.toMatch(/read_model_portable/);
      expect(appReadModel).toMatch(
        /use\s+extractum_analysis::\{[\s\S]*?\bprepare_analysis_run_summaries\b/,
      );
    } else {
      expect(
        appReadModelTests.match(/include!\("read_model_portable\.rs"\)/g) ?? [],
      ).toHaveLength(1);
    }
    expect(appReadModelTests).toContain(
      "list_analysis_run_summaries_filters_project_runs",
    );
    expect(appReadModelTests).toContain(
      "list_analysis_run_summaries_matches_all_query_terms_across_any_field",
    );
    expect(portableReadModelTests).not.toContain(
      "list_analysis_run_summaries_filters_project_runs",
    );
    expect(portableReadModelTests).not.toContain(
      "list_analysis_run_summaries_matches_all_query_terms_across_any_field",
    );
    for (const leaf of portableLeaves) {
      expect(leaf).toMatch(/^use\s+/);
      expect(leaf).not.toMatch(/\bcrate::(?:sources|youtube|analysis_documents)\b/);
    }
    expect(portableHarness).not.toMatch(
      /\bcrate::(?:sources|youtube|analysis_documents)\b/,
    );
    expect(appHarness).toContain("create_youtube_typed_source_tables");
    expect(appHarness).toContain("rebuild_analysis_documents_for_source");
    expect(readAnalysisContractSource({
      before: "corpus/tests/source_resolution_portable.rs",
      after: { owner: "crate", path: "corpus/tests/source_resolution.rs" },
    }))
      .not.toMatch(/\bresolve_run_source_ids\s*\(/);
    expect(portableReadModelTests).not.toContain("AnalysisRunRow");
    expect(portableReadModelTests).not.toMatch(/\bmap_run_detail\s*\(/);
    expect(portableReadModelTests).not.toMatch(/\bmap_run_summary\s*\(/);
    expect(portableReadModelTests).not.toContain(
      "list_analysis_run_summaries_owned",
    );
    expect(portableReadModelTests).toContain("prepare_analysis_run_summaries");
    expect(portableReadModelTests).toContain("prepare_analysis_run_detail");
    expect(portableReadModelTests).toContain(".finish(");
  });

  it("keeps owned read-model staging compiled at its final module identity", () => {
    const storeFacade = readAppAnalysisSource("store.rs");
    const portableStoreFacade = readAnalysisContractSource({
      before: "store_portable.rs",
      after: { owner: "crate", path: "store.rs" },
    });
    const portableStoreTestRoot = readAnalysisContractSource({
      before: "store/tests/mod_portable.rs",
      after: { owner: "crate", path: "store/tests/mod.rs" },
    });
    const appReadModel = readAppAnalysisSource("store/read_model.rs");
    const ownedReadModel = readAnalysisContractSource({
      before: "store/owned_read_model.rs",
      after: { owner: "crate", path: "store/read_model.rs" },
    });
    const portableReadModelTests = readAnalysisContractSource({
      before: "store/tests/read_model_portable.rs",
      after: { owner: "crate", path: "store/tests/read_model.rs" },
    });

    if (analysisExtracted) {
      expect(storeFacade).not.toMatch(/store_portable|owned_read_model/);
      expect(
        [...storeFacade.matchAll(
          /(?:^|\n)\s*(?:#\[[^\]]+\]\s*)*mod\s+([a-z_]+)\s*;/g,
        )].map((match) => match[1]),
      ).toEqual(["read_model", "setup", "tests"]);
    } else {
      expect(storeFacade.match(/include!\("store_portable\.rs"\);/g) ?? [])
        .toHaveLength(1);
      expect(storeFacade).not.toMatch(
        /\bmod\s+(?:read_model|runs|setup|snapshot|tests|store_portable)\s*;/,
      );
    }
    for (const moduleName of ["read_model", "runs", "setup", "snapshot"]) {
      expect(
        portableStoreFacade.match(
          new RegExp(
            `#\\[path\\s*=\\s*"store/${moduleName}\\.rs"\\]\\s*mod\\s+${moduleName}\\s*;`,
            "g",
          ),
        ) ?? [],
      ).toHaveLength(1);
    }
    expect(
      portableStoreFacade.match(
        /#\[cfg\(test\)\]\s*#\[path\s*=\s*"store\/tests\/mod\.rs"\]\s*mod\s+tests\s*;/g,
      ) ?? [],
    ).toHaveLength(1);
    expect(portableStoreFacade.match(/#\[path\s*=/g) ?? []).toHaveLength(5);
    expect(
      [...portableStoreFacade.matchAll(/^\s*mod\s+([a-z_]+)\s*;/gm)].map(
        (match) => match[1],
      ),
    ).toEqual(["read_model", "runs", "setup", "snapshot", "tests"]);
    expect(portableStoreFacade).not.toMatch(/\bmod\s+store_portable\s*;/);
    expect(portableStoreFacade).not.toContain("mod_portable");
    expect(portableStoreTestRoot.replace(/\r\n/g, "\n").trim()).toBe(
      [
        "mod harness;",
        "mod read_model;",
        "mod runs;",
        "mod setup;",
        "mod snapshot;",
      ].join("\n"),
    );
    if (analysisExtracted) {
      expect(appReadModel).not.toMatch(/\binclude!\s*\(/);
      expect(appReadModel).toMatch(
        /use\s+extractum_analysis::\{[\s\S]*?\bprepare_analysis_run_detail\b/,
      );
    } else {
      expect(appReadModel.match(/include!\("owned_read_model\.rs"\);/g) ?? [])
        .toHaveLength(1);
    }
    expect(appReadModel).not.toMatch(
      /#\[path\s*=\s*"owned_read_model\.rs"\]\s*mod\s+owned_read_model\s*;/,
    );
    expect(appReadModel).not.toMatch(/use\s+super::read_model::\{/);
    expect(ownedReadModel).toMatch(/use\s+super::super::domain::\{/);
    expect(ownedReadModel).toMatch(/use\s+super::super::models::\{/);
    expect(ownedReadModel).not.toContain("super::super::super::");
    expect(portableReadModelTests).toMatch(
      /use\s+super::super::read_model::\{/,
    );
    expect(portableReadModelTests).not.toMatch(
      /include!\(\s*"\.\.\/owned_read_model\.rs"\s*\)/,
    );
  });

  it("curates the owned corpus facade and forbids legacy corpus compatibility types", () => {
    const corpus = readAnalysisContractSource({
      before: "corpus_portable.rs",
      after: { owner: "crate", path: "corpus.rs" },
    });
    const corpusAdapter = readAppAnalysisSource("corpus.rs");
    const corpusMessageBody = corpus.match(
      /pub struct AnalysisCorpusMessage\s*\{([\s\S]*?)\n\}/,
    )?.[1];
    const authorizedConsumers = [
      ...rustFiles(currentAnalysisRoot),
      ...(analysisExtracted ? rustFiles(analysisCrateRoot) : []),
      path.join(repoRoot, "src-tauri/src/projects/mod.rs"),
      path.join(repoRoot, "src-tauri/src/projects/data_range.rs"),
    ]
      .map((file) => readFileSync(file, "utf8"))
      .join("\n");

    expect(corpusAdapter).not.toContain("corpus_portable::*");
    const corpusExports = [
      "AnalysisCorpusMessage",
      "AnalysisCorpusReader",
      "AnalysisCorpusRequest",
      "AnalysisPortFuture",
      "AnalysisRunPreflight",
      "AnalysisRunPreflightLimits",
      "YoutubeCorpusMode",
      "preflight_analysis_corpus",
    ] as const;
    if (analysisExtracted) {
      const crateRoot = readCrateAnalysisSource("lib.rs");
      const adapterExports = [
        ...corpusAdapter.matchAll(
          /pub\(crate\)\s+use\s+extractum_analysis::\{([\s\S]*?)\};/g,
        ),
      ].flatMap((match) => splitTopLevel(match[1]));
      expect(adapterExports).toEqual([
        "AnalysisCorpusMessage",
        "AnalysisCorpusReader",
        "AnalysisCorpusRequest",
        "AnalysisPortFuture",
        "YoutubeCorpusMode",
      ]);
      for (const exported of corpusExports) {
        expect(crateRoot).toMatch(
          new RegExp(`pub use corpus::\\{[\\s\\S]*?\\b${exported}\\b`),
        );
      }
      expect(corpusAdapter).toContain(
        "pub(crate) use extractum_analysis::resolve_analysis_telegram_history_scope;",
      );
    } else {
      for (const exported of corpusExports) {
        expect(corpusAdapter).toMatch(
          new RegExp(
            `pub\\(crate\\) use super::corpus_portable::\\{[\\s\\S]*\\b${exported}\\b`,
          ),
        );
      }
    }
    expect(corpusMessageBody).toBeDefined();
    expect(corpusMessageBody).not.toMatch(/\bpub(?:\([^)]*\))?\s+/);
    for (const accessor of [
      "item_id(&self) -> i64",
      "source_id(&self) -> i64",
      "external_id(&self) -> &str",
      "published_at(&self) -> i64",
      "author(&self) -> Option<&str>",
      "content(&self) -> &str",
      "reference(&self) -> &str",
      "item_kind(&self) -> Option<&str>",
      "source_type(&self) -> Option<&str>",
      "source_subtype(&self) -> Option<&str>",
      "metadata_zstd(&self) -> Option<&[u8]>",
    ]) {
      expect(corpus).toContain(`pub fn ${accessor}`);
    }
    expect(authorizedConsumers).not.toMatch(/\bCorpusLoadRequest\b/);
    expect(authorizedConsumers).not.toMatch(/\bCorpusMessage\b/);
  });

  it("keeps raw analysis run rows and mappers inside the owned read model", () => {
    const storeFacade = readAppAnalysisSource("store.rs");
    const appReadModel = readAppAnalysisSource("store/read_model.rs");
    const ownedReadModel = readAnalysisContractSource({
      before: "store/owned_read_model.rs",
      after: { owner: "crate", path: "store/read_model.rs" },
    });
    const models = readAnalysisContractSource({
      before: "models.rs",
      after: { owner: "crate", path: "models.rs" },
    });
    const lifecycle = portableReportLifecycleSource();
    const applicationTests = readAppAnalysisSource("tests_application.rs");
    const portableTests = readAnalysisContractSource({
      before: "tests_portable.rs",
      after: { owner: "crate", path: "tests.rs" },
    });

    for (const broadExport of [
      "fetch_run_row",
      "map_run_detail",
      "map_run_summary",
      "list_analysis_run_summaries",
      "list_analysis_run_summaries_owned",
    ]) {
      expect(storeFacade).not.toMatch(new RegExp(`\\b${broadExport}\\b`));
    }
    expect(appReadModel).not.toMatch(/\bfn\s+fetch_run_row\s*\(/);
    expect(appReadModel).not.toMatch(/\bfn\s+list_analysis_run_summaries\s*\(/);
    expect(ownedReadModel).not.toContain("list_analysis_run_summaries_owned");
    expect(models).not.toContain("AnalysisRunRow");
    expect(ownedReadModel).toMatch(/struct AnalysisRunRow\s*\{/);
    expect(ownedReadModel).not.toMatch(
      /pub(?:\([^)]*\))?\s+struct AnalysisRunRow\s*\{/,
    );
    expect(ownedReadModel).toMatch(/\nfn map_run_summary\s*\(/);
    expect(ownedReadModel).toMatch(/\nfn map_run_detail\s*\(/);
    expect(ownedReadModel).toMatch(/\nasync fn fetch_owned_run_row\s*\(/);

    expect(lifecycle).toContain("load_analysis_run_status(pool, run_id)");
    expect(lifecycle).not.toContain("fetch_run_row");
    for (const consumer of [applicationTests, portableTests]) {
      expect(consumer).not.toMatch(/\bfetch_run_row\b/);
      expect(consumer).not.toMatch(/\bmap_run_detail\b/);
      expect(consumer).not.toMatch(/\bmap_run_summary\b/);
      expect(consumer).not.toMatch(/\blist_analysis_run_summaries\b/);
    }
  });

  it("carries the resolved analysis scope through the report ticket", () => {
    const report = portableReportSource();
    const phases = readAnalysisContractSource({
      before: "report/phases.rs",
      after: { owner: "crate", path: "report/phases.rs" },
    });
    const models = readAnalysisContractSource({
      before: "models.rs",
      after: { owner: "crate", path: "models.rs" },
    });
    const resolver = readAppAnalysisSource("corpus/source_resolution.rs");
    const scopeTests = readAnalysisContractSource({
      before: "report/tests/scope.rs",
      after: { owner: "crate", path: "report/tests/scope.rs" },
    });
    const ticketBody = report.match(
      /struct ReportRunInput\s*\{([\s\S]*?)\n\}/,
    )?.[1];
    const scopeDerives = models.match(
      /((?:#\[derive\([^\]]*\)\]\s*)*)pub struct ResolvedAnalysisScope/,
    )?.[1];

    expect(ticketBody).toBeDefined();
    expect(ticketBody).toContain("scope: ResolvedAnalysisScope");
    expect(ticketBody).not.toMatch(/\bscope_label\s*:/);
    expect(scopeDerives).toBeDefined();
    expect(scopeDerives).not.toMatch(/\bClone\b/);
    expect(report).toMatch(/input:\s*ReportRunInput\s*\{[\s\S]*?\bscope,/);
    expect(report).toContain("scope.source_ids().to_vec()");
    expect(report).toContain("scope.scope_label_snapshot()");
    expect(report).toContain("input.scope.scope_label_snapshot()");
    expect(phases).toContain("input.scope.scope_label_snapshot()");
    expect(report).not.toContain(
      "SELECT EXISTS(SELECT 1 FROM sources WHERE id = ?)",
    );
    expect(report).not.toContain(
      "SELECT COUNT(*) FROM project_sources WHERE project_id = ?",
    );
    expect(report).not.toContain("fetch_source_group(&pool");
    expect(resolver).toContain(
      '"The selected source group does not contain any sources"',
    );
    expect(scopeTests).toContain("assert_eq!(input.scope.scope_kind()");
    expect(scopeTests).toContain("assert_eq!(input.scope.scope_label_snapshot()");
  });

  it("drives the frozen corpus port cases through capture and lifecycle seams", () => {
    const corpusPort = readAnalysisContractSource({
      before: "report/tests/corpus_port.rs",
      after: { owner: "crate", path: "report/tests/corpus_port.rs" },
    });
    const report = portableReportSource();
    const lifecycle = portableReportLifecycleSource();
    const frozenNames = [
      "report_execution_uses_distinct_preflight_and_capture_corpus_reads",
      "started_load_items_uses_preflight_summary_before_empty_capture_failure",
      "started_load_items_uses_preflight_summary_before_error_capture_failure",
    ];

    const testNames = [
      ...corpusPort.matchAll(/async fn ([A-Za-z0-9_]+)\s*\(/g),
    ].map((match) => match[1]);
    expect(testNames).toEqual(frozenNames);
    expect(corpusPort).toContain("capture_analysis_corpus");
    expect(corpusPort).not.toContain("execution_capture::capture_report_corpus");
    expect(corpusPort).not.toContain("load_execution_corpus");
    expect(corpusPort).not.toContain("fn started_event(");
    expect(corpusPort.match(/started_load_items_event\(/g) ?? []).toHaveLength(3);
    expect(report).toContain(
      "fn started_load_items_event(run_id: i64, preflight: &AnalysisRunPreflight) -> RunEvent",
    );
    expect(report).toContain(
      "started_load_items_event(run_id, &input.preflight).publish(sink.as_ref())",
    );
    expect(corpusPort.match(/reader\.request_log\(\)/g) ?? []).toHaveLength(3);
    expect(corpusPort.match(/request\.clone\(\), request\.clone\(\)/g) ?? [])
      .toHaveLength(3);
    expect(corpusPort.match(/call_count\(\), 2/g) ?? []).toHaveLength(3);
    for (const downstream of [
      "load_run_snapshot_messages",
      "chunk_messages",
      "build_map_request",
      "parse_chunk_summary",
      "build_reduce_request",
      "build_trace_data",
    ]) {
      expect(corpusPort).toContain(downstream);
    }
    expect(corpusPort.match(/persist_capture_failure_event/g) ?? [])
      .toHaveLength(2);
    expect(corpusPort).toContain("ANALYSIS_STATUS_FAILED");
    expect(corpusPort).toContain(
      "Report run failed before snapshot capture completed.",
    );
    expect(lifecycle).toContain("async fn persist_capture_failure_event(");
    expect(report).toContain(
      "self::lifecycle::persist_capture_failure_event(pool, run_id, &error, now_secs())",
    );
  });

  it("keeps retained preflight integrations independent from portable imports", () => {
    const retainedPreflight = readAppAnalysisSource("corpus/tests/preflight.rs");
    const portablePreflight = readAnalysisContractSource({
      before: "corpus/tests/preflight_portable.rs",
      after: { owner: "crate", path: "corpus/tests/preflight.rs" },
    });
    const retainedLive = readAppAnalysisSource("corpus/tests/live.rs");
    const portableLive = readAnalysisContractSource({
      before: "corpus/tests/live_portable.rs",
      after: { owner: "crate", path: "corpus/tests/live.rs" },
    });
    const retainedLiveWithoutPortableInclude = retainedLive.replace(
      /^\s*include!\("live_portable\.rs"\);\s*$/m,
      "",
    );

    expect(
      retainedPreflight.match(/#\[tokio::test\]\s*async fn /g) ?? [],
    ).toHaveLength(3);
    expect(portablePreflight.match(/#\[test\]\s*fn /g) ?? []).toHaveLength(7);
    expect(
      retainedPreflight.match(/AppAnalysisCorpusReader::new\(/g) ?? [],
    ).toHaveLength(3);
    expect(
      retainedPreflight.match(/\bpreflight_analysis_corpus\(/g) ?? [],
    ).toHaveLength(3);
    expect(retainedPreflight).toContain(
      "AnalysisRunPreflightLimits as AppAnalysisRunPreflightLimits",
    );
    expect(
      retainedPreflight.match(/AppAnalysisRunPreflightLimits::default\(\)/g) ?? [],
    ).toHaveLength(3);

    expect(retainedLive.match(/#\[tokio::test\]\s*async fn /g) ?? [])
      .toHaveLength(15);
    expect(portableLive.match(/#\[test\]\s*fn /g) ?? [])
      .toHaveLength(1);
    expect(retainedLive.match(/AppAnalysisCorpusReader::new\(/g) ?? [])
      .toHaveLength(2);
    expect(retainedLive.match(/\bpreflight_analysis_corpus\(/g) ?? [])
      .toHaveLength(2);
    expect(retainedLiveWithoutPortableInclude).toContain(
      "YoutubeCorpusMode as AppYoutubeCorpusMode",
    );
    expect(
      retainedLiveWithoutPortableInclude.match(
        /\bAppYoutubeCorpusMode::/g,
      ) ?? [],
    ).toHaveLength(15);
    expect(retainedLiveWithoutPortableInclude).not.toMatch(
      /\bYoutubeCorpusMode::/,
    );
    expect(portableLive).toContain(
      "use super::super::super::corpus::YoutubeCorpusMode;",
    );
    expect(portableLive).not.toContain("AppYoutubeCorpusMode");
    if (analysisExtracted) {
      expect(retainedPreflight).not.toMatch(/\binclude!\s*\(/);
      expect(retainedLive).not.toMatch(/\binclude!\s*\(/);
    } else {
      expect(retainedPreflight).toContain('include!("preflight_portable.rs");');
      expect(retainedLive).toContain('include!("live_portable.rs");');
    }
  });

  it("freezes the 21 release, three project, and three dev command inventory", () => {
    const registrations = handlerBody();
    const analysisCommandDeclarations = [
      ...analysisSources.matchAll(
        /#\[tauri::command\][\s\S]{0,240}?pub\s+async\s+fn\s+([A-Za-z0-9_]+)\s*\(/g,
      ),
    ].map((match) => match[1]);
    expect([...new Set(analysisCommandDeclarations)].sort()).toEqual(
      [...analysisRelease, ...devCommands].sort(),
    );
    for (const command of [...analysisRelease, ...projectRelease, ...devCommands]) {
      expect(registrations.match(new RegExp(`\\b${command}\\b`, "g")) ?? [], command).toHaveLength(1);
    }
    for (const command of projectRelease) {
      expect(
        projectSources.match(
          new RegExp(
            `#\\[tauri::command\\][\\s\\S]{0,240}?pub\\s+async\\s+fn\\s+${command}\\s*\\(`,
            "g",
          ),
        ) ?? [],
        command,
      ).toHaveLength(1);
    }
    expect(analysisRelease).toHaveLength(21);
    expect(projectRelease).toHaveLength(3);
    expect(devCommands).toHaveLength(3);
  });

  it("pins startup and named cross-domain compatibility paths outside the 27-command count", () => {
    expect(appLib).toContain("cleanup_interrupted_analysis_runs(handle.clone()).await");
    const paths = {
      projectList: readFileSync(path.join(repoRoot, "src-tauri/src/projects/read_model.rs"), "utf8"),
      projectDeletion: readFileSync(path.join(repoRoot, "src-tauri/src/projects/mod.rs"), "utf8"),
      accountDeletion: readFileSync(path.join(repoRoot, "src-tauri/src/account_deletion.rs"), "utf8"),
      notebookLm: readFileSync(path.join(repoRoot, "src-tauri/src/notebooklm_export/query.rs"), "utf8"),
      diagnostics: readFileSync(path.join(repoRoot, "src-tauri/src/diagnostics/database.rs"), "utf8"),
    };
    const projectDeletionProduction = paths.projectDeletion.split("#[cfg(test)]")[0];
    expect(paths.projectList).toContain("pub(crate) async fn list_research_projects_in_pool");

    expect(paths.projectDeletion).toContain("pub(crate) async fn delete_project_in_pool");
    expect(projectDeletionProduction).toContain(
      "delete_project_analysis_runs(&mut *transaction, project_id).await?",
    );
    expectOrdered(projectDeletionProduction, [
      "delete_project_analysis_runs(&mut *transaction, project_id).await?",
      'sqlx::query("DELETE FROM project_sources WHERE project_id = ?")',
      'sqlx::query("DELETE FROM projects WHERE id = ?")',
      "transaction.commit().await",
    ]);

    expect(paths.notebookLm).toContain(
      "pub(crate) async fn load_export_source_group_in_pool",
    );
    expect(paths.notebookLm).toMatch(
      /pub\(crate\)\s+use\s+(?:self::)?load_export_source_group_in_pool\s+as\s+load_export_source_group\s*;/,
    );

  });

  it("routes ordinary cross-domain analysis APIs through owned pool functions", () => {
    const runStore = readAnalysisContractSource({
      before: "store/runs.rs",
      after: { owner: "crate", path: "store/runs.rs" },
    });
    const storeRoot = readAnalysisContractSource({
      before: "store_portable.rs",
      after: { owner: "crate", path: "store.rs" },
    });
    const analysisFacade = readAnalysisContractSource({
      before: "mod.rs",
      after: { owner: "app", path: "mod.rs" },
    });
    const accountDeletion = readFileSync(
      path.join(repoRoot, "src-tauri/src/account_deletion.rs"),
      "utf8",
    );
    const diagnostics = readFileSync(
      path.join(repoRoot, "src-tauri/src/diagnostics/database.rs"),
      "utf8",
    );
    const lifecycle = readAppAnalysisSource("report/lifecycle.rs");
    const fixtures = readAppAnalysisSource("fixtures.rs");
    const accountTestModule = accountDeletion.search(
      /\n#\[cfg\(test\)\]\s*\nmod\s+tests\s*\{/,
    );
    const diagnosticsTestModule = diagnostics.search(
      /\n#\[cfg\(test\)\]\s*\nmod\s+tests\s*\{/,
    );
    expect(accountTestModule, "account-deletion test module").toBeGreaterThan(0);
    expect(diagnosticsTestModule, "diagnostics test module").toBeGreaterThan(0);
    const accountDeletionProduction = accountDeletion.slice(0, accountTestModule);
    const diagnosticsProduction = diagnostics.slice(0, diagnosticsTestModule);

    expect(runStore).toMatch(
      /pub\s+async\s+fn\s+analysis_run_ids_depending_on_sources\s*\(\s*pool:\s*&SqlitePool,\s*candidate_run_ids:\s*&HashSet<i64>,\s*owned_source_ids:\s*&\[i64\],\s*\)\s*->\s*AppResult<BTreeSet<i64>>/,
    );
    expect(runStore).toMatch(
      /pub\s+async\s+fn\s+load_analysis_run_diagnostics\s*\(\s*pool:\s*&SqlitePool,\s*\)\s*->\s*AppResult<Vec<AnalysisRunDiagnosticCount>>/,
    );

    const dependentRunLookup = uniqueRustBracedBody(
      runStore,
      /pub\s+async\s+fn\s+analysis_run_ids_depending_on_sources\s*\(\s*pool:\s*&SqlitePool,\s*candidate_run_ids:\s*&HashSet<i64>,\s*owned_source_ids:\s*&\[i64\],\s*\)\s*->\s*AppResult<BTreeSet<i64>>\s*\{/,
      "Step 8 source-dependent run lookup",
    );
    const groupMembershipLookup = uniqueRustBracedBody(
      runStore,
      /async\s+fn\s+group_has_owned_source\s*\(\s*pool:\s*&SqlitePool,\s*group_id:\s*i64,\s*owned_source_ids:\s*&HashSet<i64>,\s*\)\s*->\s*AppResult<bool>\s*\{/,
      "Step 8 source-group membership lookup",
    );
    expect(normalized(dependentRunLookup)).toContain(
      normalized(`
        if candidate_run_ids.is_empty() || owned_source_ids.is_empty() {
          return Ok(BTreeSet::new());
        }
      `),
    );
    expect(dependentRunLookup).toContain(
      '"SELECT id, source_id, source_group_id FROM analysis_runs ORDER BY id ASC"',
    );
    expect(groupMembershipLookup).toContain(
      '"SELECT source_id FROM analysis_source_group_members WHERE group_id = ?"',
    );
    expectOrdered(dependentRunLookup, [
      "candidate_run_ids.contains(&row.id)",
      ".source_id",
      "blocked.insert(row.id)",
      "continue;",
      "row.source_group_id",
      "group_has_owned_source(pool, group_id, &owned)",
    ]);
    expect(`${dependentRunLookup}\n${groupMembershipLookup}`).not.toMatch(
      /\b(?:project_id|projects|project_sources)\b/,
    );

    const diagnosticLookup = uniqueRustBracedBody(
      runStore,
      /pub\s+async\s+fn\s+load_analysis_run_diagnostics\s*\(\s*pool:\s*&SqlitePool,\s*\)\s*->\s*AppResult<Vec<AnalysisRunDiagnosticCount>>\s*\{/,
      "Step 8 analysis diagnostics lookup",
    );
    expect(normalized(diagnosticLookup)).toContain(normalized(`
      FROM analysis_runs
      GROUP BY provider, run_type, scope_type, status, snapshot_state, error_kind
      ORDER BY provider, run_type, scope_type, status, snapshot_state, error_kind
    `));
    expect(diagnosticLookup).toContain("END AS snapshot_state");
    expect(diagnosticLookup).toContain("END AS error_kind");
    expect(diagnosticLookup).not.toMatch(/\bSELECT\s+error\b/i);
    expect(diagnosticLookup).not.toContain('try_get("error")');

    const ownedSqlTables = [
      ...`${dependentRunLookup}\n${groupMembershipLookup}\n${diagnosticLookup}`.matchAll(
        /\b(?:FROM|JOIN|UPDATE|INTO|DELETE\s+FROM)\s+([A-Za-z_][A-Za-z0-9_]*)/gi,
      ),
    ].map((match) => match[1]).sort();
    expect([...new Set(ownedSqlTables)]).toEqual([
      "analysis_runs",
      "analysis_source_group_members",
    ]);
    for (const body of [dependentRunLookup, groupMembershipLookup, diagnosticLookup]) {
      expect(body).not.toMatch(
        /(?:\.|::)(?:acquire|begin|commit|rollback)\s*\(/,
      );
      expect(body).not.toContain("&mut SqliteConnection");
    }

    const diagnosticFields = uniqueRustBracedBody(
      runStore,
      /pub\s+struct\s+AnalysisRunDiagnosticCount\s*\{/,
      "Step 8 analysis diagnostics DTO",
    );
    expect(normalized(diagnosticFields)).toBe(normalized(`
      provider: String,
      run_type: String,
      scope_type: String,
      status: String,
      snapshot_state: String,
      error_kind: String,
      count: i64,
    `));
    const diagnosticAccessors = uniqueRustBracedBody(
      runStore,
      /impl\s+AnalysisRunDiagnosticCount\s*\{/,
      "Step 8 analysis diagnostics DTO accessors",
    );
    for (const accessor of [
      "pub fn provider(&self) -> &str",
      "pub fn run_type(&self) -> &str",
      "pub fn scope_type(&self) -> &str",
      "pub fn status(&self) -> &str",
      "pub fn snapshot_state(&self) -> &str",
      "pub fn error_kind(&self) -> &str",
      "pub fn count(&self) -> i64",
    ]) {
      expect(diagnosticAccessors).toContain(accessor);
    }

    const publicRunReexports = [
      ...storeRoot.matchAll(/pub\s+use\s+self::runs::\{([\s\S]*?)\};/g),
    ].flatMap((match) => splitTopLevel(match[1]));
    for (const item of [
      "analysis_run_ids_depending_on_sources",
      "load_analysis_run_diagnostics",
      "AnalysisRunDiagnosticCount",
    ]) {
      expect(publicRunReexports.filter((entry) => entry === item), item).toHaveLength(1);
    }
    if (analysisExtracted) {
      expect(analysisFacade).toMatch(
        /pub\(crate\)\s+use\s+extractum_analysis::\{[\s\S]*?\banalysis_run_ids_depending_on_sources\b[\s\S]*?\bdelete_project_analysis_runs\b[\s\S]*?\bload_analysis_run_diagnostics\b[\s\S]*?\};/,
      );
      expect(accountDeletionProduction).toMatch(
        /use\s+extractum_analysis::\{\s*analysis_run_ids_depending_on_sources,\s*AnalysisState\s*\};/,
      );
      expect(diagnosticsProduction).toContain(
        "use extractum_analysis::load_analysis_run_diagnostics;",
      );
    } else {
      expect(analysisFacade).toMatch(
        /pub\(crate\)\s+use\s+self::store::\{[\s\S]*?\banalysis_run_ids_depending_on_sources\b[\s\S]*?\bload_analysis_run_diagnostics\b[\s\S]*?\};/,
      );
    }

    expectOrdered(accountDeletionProduction, [
      "analysis_state.active_report_run_ids().await",
      "analysis_run_ids_depending_on_sources(pool, &active_run_ids, owned_source_ids).await?",
      "llm_scheduler.active_owner_run_ids().await",
      "analysis_run_ids_depending_on_sources(pool, &llm_owner_run_ids, owned_source_ids).await?",
    ]);
    expect(accountDeletionProduction).not.toContain("async fn run_ids_depending_on_sources");
    expect(accountDeletionProduction).not.toContain("async fn group_has_owned_source");
    expect(accountDeletionProduction).not.toContain("struct AnalysisRunScopeRow");
    expect(accountDeletionProduction).not.toMatch(
      /\b(?:analysis_runs|analysis_source_group_members)\b/i,
    );

    expect(diagnosticsProduction).toContain("load_analysis_run_diagnostics(pool).await?");
    for (const mapping of [
      "provider: count.provider().to_string()",
      "run_type: count.run_type().to_string()",
      "scope_type: count.scope_type().to_string()",
      "status: count.status().to_string()",
      "snapshot_state: count.snapshot_state().to_string()",
      "error_kind: count.error_kind().to_string()",
      "count: count.count()",
    ]) {
      expect(diagnosticsProduction).toContain(mapping);
    }
    expect(diagnosticsProduction).not.toContain("load_analysis_run_counts");
    expect(diagnosticsProduction).not.toMatch(/\banalysis_runs\b/i);

    expect(appLib).toContain("cleanup_interrupted_analysis_runs(handle.clone()).await");
    const cleanup = uniqueRustBracedBody(
      lifecycle,
      /pub\s+async\s+fn\s+cleanup_interrupted_analysis_runs\s*\(\s*handle:\s*AppHandle\s*\)\s*\{/,
      "silent startup analysis cleanup",
    );
    expect(normalized(cleanup)).toBe(normalized(
      analysisExtracted
        ? `
          if let Ok(pool) = get_pool(&handle).await {
              let _ = mark_interrupted_analysis_runs(&pool).await;
          }
        `
        : `
          if let Ok(pool) = get_pool(&handle).await {
              let _ = super::mark_interrupted_analysis_runs(&pool).await;
          }
        `,
    ));
    expect(fixtures).toContain("prepare_report_run_cancellation_wait(run_id).await");
    expect(fixtures).toContain("cancellation_wait.cancelled().await");
    expect(fixtures).not.toMatch(/\b(?:active_report_runs|report_run_tokens|report_run_child_token)\b/);
  });

  it("pins the closed Family 1 borrowed-connection inclusion and ordinary-pool exclusions", () => {
    const familySection = specification.slice(
      specification.indexOf("### SQL capability forms"),
      specification.indexOf("## Resolved Scope Boundary"),
    );
    const included = [
      "list_analysis_runs",
      "list_active_analysis_runs",
      "get_analysis_run",
      "ask_analysis_run_question",
    ] as const;
    const excluded = [
      "list_analysis_run_messages",
      "get_analysis_run_trace",
      "resolve_analysis_trace_refs",
      "delete_analysis_run",
      "list_analysis_chat_messages",
      "clear_analysis_chat_messages",
      "duplicate lookup",
      "snapshot",
      "cancellation",
      "startup cleanup",
      "terminal lifecycle reads",
    ] as const;
    for (const command of included) expect(familySection).toContain(`\`${command}\``);
    for (const command of [
      "get_analysis_run_trace",
      "resolve_analysis_trace_refs",
      "delete_analysis_run",
      "list_analysis_chat_messages",
      "clear_analysis_chat_messages",
      "cancellation",
    ]) expect(familySection).toContain(command);
    expect(new Set([...included, ...excluded]).size).toBe(included.length + excluded.length);
    for (const command of excluded.slice(0, 6)) {
      const definition = analysisSources.indexOf(`fn ${command}(`);
      expect(definition, command).toBeGreaterThanOrEqual(0);
      expect(analysisSources.slice(definition, definition + 900), command).toContain("get_pool");
    }
  });

  it("routes Family 2 enrichment through borrowed participants and app-owned transactions", () => {
    const groupStore = readAnalysisContractSource({
      before: "groups_store.rs",
      after: { owner: "crate", path: "groups.rs" },
    });
    const groupAdapter = readAppAnalysisSource("groups.rs");
    const setupAdapter = readAppAnalysisSource("store/setup.rs");
    const notebook = readFileSync(
      path.join(repoRoot, "src-tauri/src/notebooklm_export/query.rs"),
      "utf8",
    );
    const groupAdapterProduction = groupAdapter.split("#[cfg(test)]")[0];
    const notebookProduction = notebook.split("#[cfg(test)]")[0];

    const listParticipant = uniqueRustBracedBody(
      groupStore,
      /pub\s+async\s+fn\s+load_analysis_source_groups_for_enrichment\s*\(\s*conn:\s*&mut\s+SqliteConnection,\s*\)\s*->\s*AppResult<Vec<AnalysisSourceGroupRecord>>\s*\{/,
      "Family 2 group-list participant",
    );
    const oneParticipant = uniqueRustBracedBody(
      groupStore,
      /pub\s+async\s+fn\s+load_analysis_source_group_for_enrichment\s*\(\s*conn:\s*&mut\s+SqliteConnection,\s*group_id:\s*i64,\s*\)\s*->\s*AppResult<Option<AnalysisSourceGroupRecord>>\s*\{/,
      "Family 2 one-group participant",
    );
    for (const participant of [listParticipant, oneParticipant]) {
      expect(participant).not.toMatch(
        /\bSqlitePool\b|\.acquire\s*\(|\.begin\s*\(|\.commit\s*\(|\.rollback\s*\(/,
      );
    }
    expect(groupStore).not.toMatch(
      /\b(?:sources|items|projects|project_sources|analysis_documents)\b/i,
    );

    const listCoordinator = uniqueRustBracedBody(
      setupAdapter,
      /pub\(crate\)\s+async\s+fn\s+list_analysis_source_groups_in_pool\s*\([\s\S]*?\)\s*->\s*AppResult<Vec<AnalysisSourceGroup>>\s*\{/,
      "Family 2 group-list coordinator",
    );
    const oneCoordinator = uniqueRustBracedBody(
      setupAdapter,
      /pub\(crate\)\s+async\s+fn\s+get_analysis_source_group_response_in_pool\s*\([\s\S]*?\)\s*->\s*AppResult<Option<AnalysisSourceGroup>>\s*\{/,
      "Family 2 one-group coordinator",
    );
    for (const coordinator of [listCoordinator, oneCoordinator]) {
      expect(coordinator).toContain("pool.begin().await");
      expect(coordinator).toContain("&mut *transaction");
      expect(coordinator).toContain("transaction.commit().await");
    }
    expect(setupAdapter).toContain("FROM sources");
    expect(setupAdapter).toContain("LEFT JOIN items");
    expect(setupAdapter).toContain("COUNT(items.content_zstd)");

    expect(groupAdapterProduction).toContain(
      "list_analysis_source_groups_in_pool(&pool).await",
    );
    expect(groupAdapterProduction).toContain(
      "get_analysis_source_group_response_in_pool(&pool, group_id).await",
    );
    expect(groupAdapterProduction).not.toMatch(
      /\banalysis_source_(?:groups|group_members)\b/i,
    );

    const notebookCoordinator = uniqueRustBracedBody(
      notebookProduction,
      /pub\(crate\)\s+async\s+fn\s+load_export_source_group_in_pool\s*\([\s\S]*?\)\s*->\s*AppResult<NotebookLmExportSourceGroup>\s*\{/,
      "Family 2 NotebookLM coordinator",
    );
    expect(notebookCoordinator).toContain("pool.begin().await");
    expect(notebookCoordinator).toContain(
      "load_analysis_source_group_for_enrichment(&mut *transaction",
    );
    expect(notebookCoordinator).toContain("transaction.commit().await");
    expect(notebookProduction).not.toMatch(
      /\banalysis_source_(?:groups|group_members)\b/i,
    );
    expect(notebookCoordinator).toContain("sources.source_type AS source_type");
    expect(notebookCoordinator).toContain(
      "ORDER BY COALESCE(sources.title, ''), sources.id",
    );
  });

  it("routes Family 3 project-list composition through one app-owned transaction", () => {
    const runStore = readAnalysisContractSource({
      before: "store/runs.rs",
      after: { owner: "crate", path: "store/runs.rs" },
    });
    const storeRoot = readAnalysisContractSource({
      before: "store_portable.rs",
      after: { owner: "crate", path: "store.rs" },
    });
    const projectAdapter = readFileSync(
      path.join(repoRoot, "src-tauri/src/projects/read_model.rs"),
      "utf8",
    );
    const projectProduction = projectAdapter.split("#[cfg(test)]")[0];

    const aggregateFields = uniqueRustBracedBody(
      runStore,
      /pub\s+struct\s+ProjectAnalysisRunAggregate\s*\{/,
      "Family 3 aggregate DTO",
    );
    expect(normalized(aggregateFields)).toBe(normalized(`
      project_id: i64,
      latest_run_status: Option<String>,
      last_run_at: Option<i64>,
      has_active_run: bool,
    `));

    const aggregateAccessors = uniqueRustBracedBody(
      runStore,
      /impl\s+ProjectAnalysisRunAggregate\s*\{/,
      "Family 3 aggregate accessors",
    );
    expect(normalized(aggregateAccessors)).toBe(normalized(`
      pub fn project_id(&self) -> i64 {
        self.project_id
      }

      pub fn latest_run_status(&self) -> Option<&str> {
        self.latest_run_status.as_deref()
      }

      pub fn last_run_at(&self) -> Option<i64> {
        self.last_run_at
      }

      pub fn has_active_run(&self) -> bool {
        self.has_active_run
      }
    `));

    const participant = uniqueRustBracedBody(
      runStore,
      /pub\s+async\s+fn\s+load_project_analysis_run_aggregates\s*\(\s*conn:\s*&mut\s+SqliteConnection,\s*project_ids:\s*&\[i64\],\s*\)\s*->\s*AppResult<Vec<ProjectAnalysisRunAggregate>>\s*\{/,
      "Family 3 project-run aggregate participant",
    );
    expect(participant).toMatch(/\banalysis_runs\b/);
    expect(participant).not.toMatch(
      /\b(?:SqlitePool|SqlitePoolOptions)\b|(?:\.|::)(?:acquire|connect(?:_with|_lazy|_lazy_with)?|begin|commit|rollback)\s*\(/,
    );
    expect(participant).not.toMatch(
      /\b(?:projects|project_sources|sources|items|youtube_playlist_items|analysis_documents)\b/i,
    );
    expect(participant.match(/\bproject_ids\.is_empty\(\)/g) ?? []).toHaveLength(1);
    expect(participant.match(/\bQueryBuilder::<Sqlite>::new\s*\(/g) ?? []).toHaveLength(1);
    expect(participant.match(/ORDER BY latest\.created_at DESC, latest\.id DESC/g) ?? [])
      .toHaveLength(2);
    expect(normalized(participant)).toContain(normalized(`
      MAX(
        CASE
          WHEN runs.status IN ('queued', 'running') THEN 1
          ELSE 0
        END
      ) AS has_active_run
    `));
    expect(normalized(participant)).toContain(
      "GROUP BY runs.project_id ORDER BY runs.project_id ASC",
    );
    expect(participant.match(/\.await\b/g) ?? []).toHaveLength(1);
    expect(
      participant.match(/\.(?:fetch_all|fetch_one|fetch_optional|execute)\s*\(/g) ?? [],
    ).toHaveLength(1);
    expect(participant).toContain(".fetch_all(conn)");

    const runStoreProduction = runStore.split("#[cfg(test)]")[0];
    const runStoreSqlTables = [
      ...runStoreProduction.matchAll(
        /\b(?:FROM|JOIN|UPDATE|INTO|DELETE\s+FROM)\s+([A-Za-z_][A-Za-z0-9_]*)/gi,
      ),
    ].map((match) => match[1].toLowerCase());
    expect(runStoreSqlTables).toContain("analysis_runs");
    expect(
      [...new Set(runStoreSqlTables)].every((table) =>
        [
          "analysis_chat_messages",
          "analysis_run_messages",
          "analysis_runs",
          "analysis_source_group_members",
        ].includes(table)
      ),
      `unexpected SQL table in run store: ${[...new Set(runStoreSqlTables)].join(", ")}`,
    ).toBe(true);

    const migrationFiles: string[] = [];
    const visitMigrationDirectory = (directory: string): void => {
      for (const entry of readdirSync(directory, { withFileTypes: true })) {
        const selected = path.join(directory, entry.name);
        if (entry.isDirectory()) visitMigrationDirectory(selected);
        else if (entry.isFile() && entry.name.endsWith(".sql")) migrationFiles.push(selected);
      }
    };
    visitMigrationDirectory(path.join(repoRoot, "src-tauri/migrations"));
    const schemaTables = [
      ...migrationFiles
        .map((file) => readFileSync(file, "utf8"))
        .join("\n")
        .matchAll(
          /\bCREATE\s+TABLE(?:\s+IF\s+NOT\s+EXISTS)?\s+(?:"([^"]+)"|`([^`]+)`|\[([^\]]+)\]|([A-Za-z_][A-Za-z0-9_]*))/gi,
        ),
    ].map((match) => (match[1] ?? match[2] ?? match[3] ?? match[4]).toLowerCase());
    const analysisOwnedTables = new Set([
      "analysis_chat_messages",
      "analysis_prompt_templates",
      "analysis_run_messages",
      "analysis_runs",
      "analysis_source_group_members",
      "analysis_source_groups",
    ]);
    expect([...analysisOwnedTables].every((table) => schemaTables.includes(table))).toBe(true);
    const foreignSchemaTables = [...new Set(schemaTables)]
      .filter((table) => !analysisOwnedTables.has(table));
    expect(foreignSchemaTables.length).toBeGreaterThan(0);
    for (const table of foreignSchemaTables) {
      const escaped = table.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
      expect(runStoreProduction, `foreign table token in run store: ${table}`)
        .not.toMatch(new RegExp(`\\b${escaped}\\b`, "i"));
    }

    const publicRunReexportBodies = [
      ...storeRoot.matchAll(/pub\s+use\s+self::runs::\{([\s\S]*?)\};/g),
    ].map((match) => match[1]);
    expect(publicRunReexportBodies).toHaveLength(1);
    const publicRunReexports = splitTopLevel(publicRunReexportBodies[0]);
    expect(
      publicRunReexports.filter((entry) => entry === "load_project_analysis_run_aggregates"),
    ).toHaveLength(1);
    expect(
      publicRunReexports.filter((entry) => entry === "ProjectAnalysisRunAggregate"),
    ).toHaveLength(1);

    const coordinator = uniqueRustBracedBody(
      projectProduction,
      /pub\(crate\)\s+async\s+fn\s+list_research_projects_in_pool\s*\([\s\S]*?\)\s*->\s*AppResult<Vec<ProjectSummary>>\s*\{/,
      "Family 3 project-list coordinator",
    );
    expect(coordinator.match(/pool\.begin\(\)\.await/g) ?? []).toHaveLength(1);
    expect(coordinator.match(/transaction\.commit\(\)\.await/g) ?? []).toHaveLength(1);
    expect(coordinator.match(/(?:\.|::)begin\s*\(/g) ?? []).toHaveLength(1);
    expect(coordinator.match(/(?:\.|::)commit\s*\(/g) ?? []).toHaveLength(1);
    expect(coordinator.match(/\bpool\b/g) ?? []).toHaveLength(1);
    expect(
      coordinator.match(/\.(?:fetch_all|fetch_one|fetch_optional|execute)\s*\(/g) ?? [],
    ).toHaveLength(1);
    expect(coordinator).not.toMatch(
      /(?:\.|::)(?:acquire|connect(?:_with|_lazy|_lazy_with)?|rollback)\s*\(/,
    );
    expect(coordinator).toContain(".fetch_all(&mut *transaction)");
    expect(coordinator).toContain(
      "load_project_analysis_run_aggregates(&mut *transaction, &project_ids)",
    );
    expectOrdered(coordinator, [
      "pool.begin().await",
      ".fetch_all(&mut *transaction)",
      "let project_ids",
      "load_project_analysis_run_aggregates(&mut *transaction, &project_ids)",
      "let aggregates_by_project_id",
      "let projects = rows",
      "aggregates_by_project_id.get(&row.id)",
      "map_project_summary(row, aggregate)",
      "transaction.commit().await",
    ]);
    expect(projectProduction).not.toMatch(/\banalysis_runs\b/i);
  });

  it("routes Family 4 project deletion through one app-owned transaction", () => {
    const runStore = readAnalysisContractSource({
      before: "store/runs.rs",
      after: { owner: "crate", path: "store/runs.rs" },
    });
    const storeRoot = readAnalysisContractSource({
      before: "store_portable.rs",
      after: { owner: "crate", path: "store.rs" },
    });
    const analysisFacade = readAnalysisContractSource({
      before: "mod.rs",
      after: { owner: "app", path: "mod.rs" },
    });
    const projectAdapter = readFileSync(
      path.join(repoRoot, "src-tauri/src/projects/mod.rs"),
      "utf8",
    );
    const projectProduction = projectAdapter.split("#[cfg(test)]")[0];

    const participant = uniqueRustBracedBody(
      runStore,
      /pub\s+async\s+fn\s+delete_project_analysis_runs\s*\(\s*conn:\s*&mut\s+SqliteConnection,\s*project_id:\s*i64,\s*\)\s*->\s*AppResult<\(\)>\s*\{/,
      "Family 4 project-run deletion participant",
    );
    expect(normalized(participant)).toBe(normalized(`
      sqlx::query("DELETE FROM analysis_runs WHERE project_id = ?")
        .bind(project_id)
        .execute(&mut *conn)
        .await
        .map_err(AppError::database)?;
      Ok(())
    `));

    const publicRunReexports = [
      ...storeRoot.matchAll(/pub\s+use\s+self::runs::\{([\s\S]*?)\};/g),
    ].flatMap((match) => splitTopLevel(match[1]));
    expect(
      publicRunReexports.filter((entry) => entry === "delete_project_analysis_runs"),
    ).toHaveLength(1);

    const facadeReexports = [
      ...analysisFacade.matchAll(
        /pub\(crate\)\s+use\s+(?:self::store|extractum_analysis)::(\{[\s\S]*?\}|[A-Za-z_][A-Za-z0-9_]*)\s*;/g,
      ),
    ].flatMap((match) => {
      const selected = match[1].trim();
      return selected.startsWith("{")
        ? splitTopLevel(selected.slice(1, -1))
        : [selected];
    });
    expect(
      facadeReexports.filter((entry) => entry === "delete_project_analysis_runs"),
    ).toHaveLength(1);
    if (analysisExtracted) {
      expect(projectProduction).toMatch(
        /use\s+extractum_analysis::\{[\s\S]*?\bdelete_project_analysis_runs\b[\s\S]*?\bAnalysisRunListFilters\b[\s\S]*?\bAnalysisRunSummary\b[\s\S]*?\bAnalysisState\b[\s\S]*?\bStartAnalysisReportRequest\b[\s\S]*?\};/,
      );
      expect(projectProduction).not.toContain(
        "use crate::analysis::delete_project_analysis_runs;",
      );
    } else {
      expect(projectProduction).toContain(
        "use crate::analysis::delete_project_analysis_runs;",
      );
    }

    const coordinator = uniqueRustBracedBody(
      projectProduction,
      /pub\(crate\)\s+async\s+fn\s+delete_project_in_pool\s*\([\s\S]*?\)\s*->\s*AppResult<\(\)>\s*\{/,
      "Family 4 project-deletion coordinator",
    );
    expect(coordinator.match(/pool\.begin\(\)\.await/g) ?? []).toHaveLength(1);
    expect(coordinator.match(/transaction\.commit\(\)\.await/g) ?? []).toHaveLength(1);
    expect(coordinator.match(/(?:\.|::)begin\s*\(/g) ?? []).toHaveLength(1);
    expect(coordinator.match(/(?:\.|::)commit\s*\(/g) ?? []).toHaveLength(1);
    expect(coordinator.match(/\bpool\b/g) ?? []).toHaveLength(1);
    expect(coordinator.match(/\.await\b/g) ?? []).toHaveLength(5);
    expect(
      coordinator.match(/\.(?:fetch_all|fetch_one|fetch_optional|execute)\s*\(/g) ?? [],
    ).toHaveLength(2);
    expect(coordinator.match(/\.execute\(&mut \*transaction\)/g) ?? [])
      .toHaveLength(2);
    expect(coordinator.match(/sqlx::query\s*\(/g) ?? []).toHaveLength(2);
    expect(coordinator.match(/\.bind\(project_id\)/g) ?? []).toHaveLength(2);
    expect(coordinator).not.toMatch(
      /(?:\.|::)(?:acquire|connect(?:_with|_lazy|_lazy_with)?|rollback)\s*\(/,
    );
    expect(coordinator).not.toContain("ensure_project_exists");
    expectOrdered(coordinator, [
      "pool.begin().await",
      "delete_project_analysis_runs(&mut *transaction, project_id).await?",
      'sqlx::query("DELETE FROM project_sources WHERE project_id = ?")',
      'let result = sqlx::query("DELETE FROM projects WHERE id = ?")',
      "transaction.commit().await",
      "if result.rows_affected() == 0",
      '"Project {project_id} not found"',
    ]);
    expect(projectProduction).not.toMatch(/\banalysis_runs\b/i);
  });

  it("analysis_wire_contract_serializes_commands_events_and_errors_unchanged", () => {
    const models = readAnalysisContractSource({
      before: "models.rs",
      after: { owner: "crate", path: "models.rs" },
    });
    const moduleSource = readAnalysisContractSource({
      before: "mod.rs",
      after: { owner: "app", path: "mod.rs" },
    });
    const chat = portableChatSource();
    const report = portableReportSource();
    const requests = readAnalysisContractSource({
      before: "report/requests.rs",
      after: { owner: "crate", path: "report/requests.rs" },
    });
    const phases = readAnalysisContractSource({
      before: "report/phases.rs",
      after: { owner: "crate", path: "report/phases.rs" },
    });
    const lifecycle = portableReportLifecycleSource();
    const wireWitness = readAppAnalysisSource("tests_application.rs");

    const contextParams = new Set(["handle", "state", "scheduler", "repair_state"]);
    for (const [name, [expectedSignature, expectedWireParams]] of Object.entries(commandWireContracts)) {
      const source = (projectRelease as readonly string[]).includes(name)
        ? projectSources
        : analysisSources;
      const actual = commandSignature(source, name);
      expect(canonicalWireSignature(actual.signature), name)
        .toBe(expectedSignature);
      const actualWireParams = actual.params
        .map((param) => param.slice(0, param.indexOf(":")))
        .filter((param) => !contextParams.has(param))
        .map(camelCase);
      expect(actualWireParams, `${name} camelCase parameters`).toEqual(expectedWireParams);
    }

    expect(moduleSource).toContain('const ANALYSIS_RUN_EVENT: &str = "analysis://run"');
    expect(moduleSource).toContain('const ANALYSIS_CHAT_EVENT: &str = "analysis://chat"');
    for (const field of [
      "run_id", "request_id", "kind", "phase", "queue_position", "message",
      "progress_current", "progress_total", "delta", "chunk_summary", "error",
    ]) expect(models).toMatch(new RegExp(`pub ${field}:`));
    for (const field of ["request_id", "run_id", "kind", "queue_position", "delta", "message", "error"]) {
      expect(models).toMatch(new RegExp(`pub ${field}:`));
    }
    expect(models).not.toContain("skip_serializing_if");
    expect(requests).toContain('format!("analysis-map-{run_id}-{chunk_index}-{}", now_secs())');
    expect(requests).toContain('format!("analysis-reduce-{}-{}", params.run_id, now_secs())');
    expect(chat).toContain('format!("analysis-chat-{}-{}", params.run.id, now_secs())');
    expect(wireWitness).toContain("fn analysis_wire_values_serialize_to_exact_json_objects()");
    expect(wireWitness.match(/serde_json::to_value\(/g) ?? []).toHaveLength(3);
    expect(wireWitness).toContain('json!({"kind": "conflict", "message": "wire failure"})');

    expectOrdered(chat, [
      'ChatEvent::new(queued_request_id.clone(), run_id, "queued")',
      'ChatEvent::new(started_request_id, run_id, "started")',
      'ChatEvent::new(delta_request_id.clone(), run_id, "delta")',
      'ChatEvent::new(completion.request_id, completion.run_id, "completed")',
    ]);
    expect(chat).toContain('ChatEvent::new(request_id.to_string(), run_id, "failed")');
    expect(chat).toContain('ChatEvent::new(request_id.to_string(), run_id, "cancelled")');
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
    for (const terminal of [
      "Answer completed.", "Answer cancelled.", "Analysis run cancelled.",
      "Report run failed.", "Report run failed before snapshot capture completed.",
    ]) {
      expect(`${analysisSources}\n${chat}\n${report}\n${lifecycle}`)
        .toContain(terminal);
    }
    expect(errorSource).toContain("#[serde(rename_all = \"snake_case\")]");
    expect(errorSource).toMatch(/pub struct AppError\s*{\s*pub kind: AppErrorKind,\s*pub message: String,/s);
  });

  it("keeps every mechanical-move source portable before extraction", () => {
    const wholeMoves = frozenMoves("wholeMoves");
    const splitMoves = frozenMoves("splitMoves");
    const allMoves = [...wholeMoves, ...splitMoves];
    expect(wholeMoves).toHaveLength(23);
    expect(splitMoves).toHaveLength(21);
    expect(new Set(allMoves.map((move) => move.before)).size).toBe(44);
    expect(new Set(allMoves.map((move) => move.after)).size).toBe(44);
    expect(retainedAnalysisFiles).toHaveLength(35);
    expect(inventoryDrift(
      ["retained.rs", "unexpected.rs", "retained.rs"],
      ["retained.rs", "missing.rs"],
    )).toEqual({
      duplicates: ["retained.rs"],
      missing: ["missing.rs"],
      unexpected: ["unexpected.rs"],
    });

    const files = analysisSourceFiles();
    for (const move of allMoves) expect(frozenMoveSource(move), `missing frozen move source: ${move.before}`).not.toBe("");
    const appExpected = analysisExtracted
      ? [...retainedAnalysisFiles]
      : [...retainedAnalysisFiles, ...allMoves.map((move) => move.before)];
    expectExactInventory(
      files.map(({ relative }) => relative),
      appExpected,
      analysisExtracted
        ? "post-extraction app analysis files"
        : "pre-extraction app analysis files",
    );
    expect(appExpected).toHaveLength(analysisExtracted ? 35 : 79);
    if (analysisExtracted) {
      const crateFiles = rustFiles(analysisCrateRoot).map((file) =>
        path.relative(analysisCrateRoot, file).replaceAll("\\", "/"));
      expectExactInventory(
        crateFiles,
        [...allMoves.map((move) => move.after), "lib.rs"],
        "post-extraction analysis crate files",
      );
      expect(crateFiles).toHaveLength(45);
    }

    const forbidden = [
      /\bcrate\s*::/, /\b(?:tauri|Tauri)\b/, /\bAppHandle\b/, /\bget_pool\b/,
    ];
    expect(maskRustComments("use crate /* hidden */ :: error::AppError;", true)).toMatch(forbidden[0]);
    for (const move of allMoves) {
      const source = frozenMoveSource(move);
      const executable = maskRustComments(source, true);
      for (const token of forbidden) expect(executable, `${move.before} imports application capability ${token}`).not.toMatch(token);
    }
    expect(frozenMoveSource(splitMoves.find((move) => move.before === "store/owned_read_model.rs")!)).toMatch(/use\s+extractum_core(?:\s*::time::|\s*::\s*\{[\s\S]*?\btime::)ymd_to_unix_midnight\s*(?:;|,)/);
  });

  it("keeps analysis SQL ownership and borrowed coordinator capabilities fail closed", () => {
    expect(sqlxQueryLiterals(`
      sqlx::raw_sql("COMMIT");
      sqlx::raw_sql(concat!("PRAGMA foreign_", "keys = OFF"));
    `).map(({ kind, sql }) => `${kind}:${normalized(sql)}`)).toEqual([
      "raw_sql:COMMIT",
      "raw_sql:PRAGMA foreign_keys = OFF",
    ]);
    const aliasedRawSqlProbe = `
      const CONTROL_SQL: &str = concat!("COM", "MIT");
      const PRAGMA_SQL: &str = "PRAGMA foreign_keys = OFF";
      sqlx::raw_sql(CONTROL_SQL);
      sqlx::raw_sql(PRAGMA_SQL);
    `;
    expect(sqlxQueryLiterals(aliasedRawSqlProbe).map(
      ({ kind, sql }) => `${kind}:${normalized(sql)}`,
    )).toEqual([
      "raw_sql:COMMIT",
      "raw_sql:PRAGMA foreign_keys = OFF",
    ]);
    expect(transactionControlSql(aliasedRawSqlProbe)).toEqual(["COMMIT"]);
    expect(foreignKeyDisableQueries(aliasedRawSqlProbe).map(
      ({ kind, sql }) => `${kind}:${normalized(sql)}`,
    )).toEqual(["raw_sql:PRAGMA foreign_keys = OFF"]);
    expect(sqlTableOperations(
      String.raw`sqlx::query("SELECT * FROM \"analysis_runs\"")`,
    ).map(({ table }) => table)).toEqual(["analysis_runs"]);
    const quotedAndStructuredTableProbe = `
      sqlx::query("SELECT * FROM 'analysis_runs'");
      sqlx::query("SELECT * FROM 'main'.'analysis_runs'");
      sqlx::query("SELECT * FROM sqlite_master, analysis_runs");
      sqlx::query("SELECT * FROM (analysis_source_groups)");
      sqlx::query("SELECT * FROM ((sqlite_master, analysis_chat_messages))");
      sqlx::query("SELECT * FROM sqlite_master, (sources, analysis_run_messages)");
      sqlx::query("UPDATE OR ROLLBACK analysis_runs SET status = 'queued'");
      sqlx::query("UPDATE OR ABORT analysis_runs SET status = 'queued'");
      sqlx::query("UPDATE OR REPLACE analysis_runs SET status = 'queued'");
      sqlx::query("UPDATE OR FAIL analysis_runs SET status = 'queued'");
      sqlx::query("UPDATE OR IGNORE analysis_runs SET status = 'queued'");
    `;
    expect(sqlTableOperations(quotedAndStructuredTableProbe).map(
      ({ operation, table }) => `${operation}:${table}`,
    )).toEqual([
      "FROM:analysis_runs",
      "FROM:analysis_runs",
      "FROM:sqlite_master",
      "FROM:analysis_runs",
      "FROM:analysis_source_groups",
      "FROM:sqlite_master",
      "FROM:analysis_chat_messages",
      "FROM:sqlite_master",
      "FROM:sources",
      "FROM:analysis_run_messages",
      "UPDATE:analysis_runs",
      "UPDATE:analysis_runs",
      "UPDATE:analysis_runs",
      "UPDATE:analysis_runs",
      "UPDATE:analysis_runs",
    ]);
    expect(foreignKeyDisableQueries(`
      use sqlx::QueryBuilder;
      let mut query = QueryBuilder::<Sqlite>::new("PRA");
      query.push("GMA foreign_").push("keys = OFF");
    `).map(({ sql }) => normalized(sql))).toEqual(["PRAGMA foreign_keys = OFF"]);
    expect(sqlTableOperations(`
      use sqlx::QueryBuilder;
      const TABLE: &str = concat!("analysis_", "runs");
      QueryBuilder::<Sqlite>::new("SELECT * FROM ").push(TABLE);
    `).map(({ table }) => table)).toEqual(["analysis_runs"]);
    const cfgFieldFixture = productionRust(`
      struct Fixture {
        #[cfg(test)] test_only: i64,
        production_field: i64,
      }
      #[cfg(test)] use self::test_only::{first, second};
      fn production_after_cfg_field() {}
    `);
    expect(cfgFieldFixture).not.toContain("test_only");
    expect(cfgFieldFixture).not.toContain("second");
    expect(cfgFieldFixture).toContain("production_field");
    expect(cfgFieldFixture).toContain("production_after_cfg_field");
    const cfgReachabilityProbe = rustModuleReachability([
      { relative: "lib.rs", source: `
        mod production;
        #[path = "tests/live.rs"] mod live;
        #[cfg(test)] #[path = "ordinary.rs"] mod hidden;
        #[cfg(dev)] mod fixtures;
        #[cfg(test)] mod gated { include!("inside.rs"); }
      ` },
      { relative: "production.rs", source: "pub fn production() {}" },
      {
        relative: "tests/live.rs",
        source: "mod child; pub fn path_only_name_is_production() {}",
      },
      { relative: "tests/child.rs", source: "pub fn path_override_child() {}" },
      { relative: "ordinary.rs", source: "pub fn cfg_only_despite_plain_path() {}" },
      { relative: "fixtures.rs", source: "pub fn dev_only() {}" },
      { relative: "inside.rs", source: "pub fn inherited_test_only() {}" },
    ], ["lib.rs"]);
    expect(cfgReachabilityProbe.production.map(({ relative }) => relative).sort()).toEqual([
      "lib.rs",
      "production.rs",
      "tests/child.rs",
      "tests/live.rs",
    ]);
    expect(cfgReachabilityProbe.cfgOnly.map(({ relative }) => relative).sort()).toEqual([
      "fixtures.rs",
      "inside.rs",
      "ordinary.rs",
    ]);
    const ownedTables = new Set(["analysis_chat_messages", "analysis_prompt_templates", "analysis_run_messages", "analysis_runs", "analysis_source_group_members", "analysis_source_groups"]);
    const portableOwners = new Set(["analysis/chat_engine.rs", "analysis/corpus/snapshot.rs", "analysis/groups_store.rs", "analysis/report/lifecycle_portable.rs", "analysis/store/owned_read_model.rs", "analysis/store/owned_setup.rs", "analysis/store/runs.rs", "analysis/store/snapshot.rs", "analysis/templates_store.rs", "analysis/trace.rs"]);
    const appSourceRoot = path.join(repoRoot, "src-tauri/src");
    const crateSourceRoot = path.join(repoRoot, "src-tauri/crates/extractum-analysis/src");
    const beforeByAfter = new Map([...frozenMoves("wholeMoves"), ...frozenMoves("splitMoves")].map((move) => [move.after, move.before]));
    const appInventory = rustFiles(appSourceRoot).map((file) => ({
      relative: path.relative(appSourceRoot, file).replaceAll("\\", "/"), source: readFileSync(file, "utf8"),
    }));
    const appReachability = rustModuleReachability(appInventory, ["lib.rs", "main.rs"]);
    const telegramFoundationIsStaged = existsSync(
      path.join(appSourceRoot, "telegram_impl/lib.rs"),
    );
    const telegramPeerAvatarBoundaryIsStaged = existsSync(
      path.join(appSourceRoot, "telegram_impl/live/peer.rs"),
    );
    const productionAppPaths = appReachability.production
      .map(({ relative }) => relative);
    if (telegramFoundationIsStaged) {
      expect(
        productionAppPaths,
        "the staged Telegram root must be production-reachable when present",
      ).toContain("telegram_impl/lib.rs");
    } else {
      expect(productionAppPaths).not.toContain("telegram_impl/lib.rs");
    }
    const movedAppPaths = new Set(
      [...frozenMoves("wholeMoves"), ...frozenMoves("splitMoves")]
        .map(({ before }) => `analysis/${before}`),
    );
    expectExactInventory(
      appReachability.cfgOnly.map(({ relative }) => relative),
      analysisExtracted
        ? expectedCfgOnlyAppRustFiles.filter((relative) => !movedAppPaths.has(relative))
        : expectedCfgOnlyAppRustFiles,
      "cfg(test/dev)-only app Rust files",
    );
    const cfgClassified = new Set([
      ...appReachability.production.map(({ relative }) => relative),
      ...appReachability.cfgOnly.map(({ relative }) => relative),
    ]);
    expectExactInventory(
      appInventory.map(({ relative }) => relative).filter((relative) => !cfgClassified.has(relative)),
      analysisExtracted ? [] : frozenUnreachableStagingTestFiles,
      "frozen unreachable staging test files",
    );
    expect(appReachability.production.map(({ relative }) => relative)).toContain("projects/read_model.rs");
    expect(appReachability.production.map(({ relative }) => relative)).not.toContain("analysis/fixtures.rs");
    const productionExclusions = new Set([
      ...appReachability.cfgOnly.map(({ relative }) => relative),
      ...frozenUnreachableStagingTestFiles,
    ]);
    const appFiles = appInventory.filter(({ relative }) => !productionExclusions.has(relative));
    const crateFiles = analysisExtracted
      ? rustModuleReachability(rustFiles(crateSourceRoot).map((file) => ({
        relative: path.relative(crateSourceRoot, file).replaceAll("\\", "/"),
        source: readFileSync(file, "utf8"),
      })), ["lib.rs"]).production.map((file) => {
        const after = file.relative;
        return { relative: `analysis/${beforeByAfter.get(after) ?? after}`, source: file.source };
      })
      : [];
    const files = [...appFiles, ...crateFiles];
    const unresolvedProductionConsumers = files.flatMap(({ relative, source }) =>
      unresolvedConsumerInventory(relative, productionRust(source))
        .map(unresolvedConsumerIdentity));
    const notebookExportSourceFingerprint = analysisExtracted
      ? "4ce6899ba162eab0cc0155c5f0a57d460a5eec1ba372b7816ed51678ab3c67e6"
      : "63929f7402132bd7860e06b050ccc70e1d214cc70d699cc3c524d7786c231d9f";
    const notebookArchiveBodyFingerprint = analysisExtracted
      ? "94c56fc8c1c0b0bf5643792f6cdac45e0b6b31bc423240d61f63f9301c5400a5"
      : "2965c448aa0fd5bf74706df2609d6f52a2f30400736eb417ebe037ea3977252c";
    const notebookReplyBodyFingerprint = analysisExtracted
      ? "9b94e015798718e328bc265b0afa1e7f3575c8de94d4817718daff9b5489ce1d"
      : "f4a3902360ed77d4750749e55c73db371c54fd2e117828bc9187f0f4e8ec4333";
    const ingestProvenanceSourceFingerprint = telegramFoundationIsStaged
      ? "3f64c972ebc82996e65a396054ec8d16e73dc5fa911b92b9b10b79ed100b4a29"
      : "a327cabba5f1ab4f3af5c2f405ccb56a4500dc2bc736d8dc33a220f128323bde";
    const sourcesStoreSourceFingerprint = telegramPeerAvatarBoundaryIsStaged
      ? "c752a11642888a3c45df0c53587588e3a7603b584de1ca19e2a3c8edc025b3e9"
      : "e510e682120b6566460fddf405f6c45d31ee7e96b399948fe91b75a51fa4a92d";
    expect(
      unresolvedProductionConsumers,
      "production unresolved executable SQL consumer inventory",
    ).toEqual([
      "apalis_jobs.rs:apalis_jobs_prune_terminal_from_pool_with_hours:24948064c5dfb5fe043faf79c4bf4d05981516a9f5840bbec0bf0541dedd7adf:05b1185e47ec480cdeab26d4c62f499a747ac87a701a5830ecfca1eaf3095ba7:query_builder_push:&done_at_epoch",
      "apalis_jobs.rs:apalis_jobs_prune_terminal_from_pool_with_hours:24948064c5dfb5fe043faf79c4bf4d05981516a9f5840bbec0bf0541dedd7adf:05b1185e47ec480cdeab26d4c62f499a747ac87a701a5830ecfca1eaf3095ba7:query_builder_push:&done_at_epoch",
      "apalis_jobs.rs:fetch_job_summaries:e3f8a21ab8453e7638518b00acd97e8b934c1f51080c257bd38db95b66f33216:05b1185e47ec480cdeab26d4c62f499a747ac87a701a5830ecfca1eaf3095ba7:query_builder_push:text_expr(schema, \"id\", \"''\")",
      "apalis_jobs.rs:fetch_job_summaries:e3f8a21ab8453e7638518b00acd97e8b934c1f51080c257bd38db95b66f33216:05b1185e47ec480cdeab26d4c62f499a747ac87a701a5830ecfca1eaf3095ba7:query_builder_push:text_expr(schema, \"job_type\", \"''\")",
      "apalis_jobs.rs:fetch_job_summaries:e3f8a21ab8453e7638518b00acd97e8b934c1f51080c257bd38db95b66f33216:05b1185e47ec480cdeab26d4c62f499a747ac87a701a5830ecfca1eaf3095ba7:query_builder_push:text_expr(schema, \"status\", \"'unknown'\")",
      "apalis_jobs.rs:fetch_job_summaries:e3f8a21ab8453e7638518b00acd97e8b934c1f51080c257bd38db95b66f33216:05b1185e47ec480cdeab26d4c62f499a747ac87a701a5830ecfca1eaf3095ba7:query_builder_push:int_expr(schema, \"attempts\", \"0\")",
      "apalis_jobs.rs:fetch_job_summaries:e3f8a21ab8453e7638518b00acd97e8b934c1f51080c257bd38db95b66f33216:05b1185e47ec480cdeab26d4c62f499a747ac87a701a5830ecfca1eaf3095ba7:query_builder_push:nullable_int_expr(schema, \"max_attempts\")",
      "apalis_jobs.rs:fetch_job_summaries:e3f8a21ab8453e7638518b00acd97e8b934c1f51080c257bd38db95b66f33216:05b1185e47ec480cdeab26d4c62f499a747ac87a701a5830ecfca1eaf3095ba7:query_builder_push:nullable_text_expr(schema, \"run_at\")",
      "apalis_jobs.rs:fetch_job_summaries:e3f8a21ab8453e7638518b00acd97e8b934c1f51080c257bd38db95b66f33216:05b1185e47ec480cdeab26d4c62f499a747ac87a701a5830ecfca1eaf3095ba7:query_builder_push:nullable_text_expr(schema, \"lock_at\")",
      "apalis_jobs.rs:fetch_job_summaries:e3f8a21ab8453e7638518b00acd97e8b934c1f51080c257bd38db95b66f33216:05b1185e47ec480cdeab26d4c62f499a747ac87a701a5830ecfca1eaf3095ba7:query_builder_push:nullable_text_expr(schema, \"lock_by\")",
      "apalis_jobs.rs:fetch_job_summaries:e3f8a21ab8453e7638518b00acd97e8b934c1f51080c257bd38db95b66f33216:05b1185e47ec480cdeab26d4c62f499a747ac87a701a5830ecfca1eaf3095ba7:query_builder_push:nullable_text_expr(schema, \"done_at\")",
      "apalis_jobs.rs:fetch_job_summaries:e3f8a21ab8453e7638518b00acd97e8b934c1f51080c257bd38db95b66f33216:05b1185e47ec480cdeab26d4c62f499a747ac87a701a5830ecfca1eaf3095ba7:query_builder_push:nullable_int_expr(schema, \"priority\")",
      "apalis_jobs.rs:fetch_job_summaries:e3f8a21ab8453e7638518b00acd97e8b934c1f51080c257bd38db95b66f33216:05b1185e47ec480cdeab26d4c62f499a747ac87a701a5830ecfca1eaf3095ba7:query_builder_push:nullable_text_expr(schema, \"idempotency_key\")",
      "apalis_jobs.rs:fetch_job_summaries:e3f8a21ab8453e7638518b00acd97e8b934c1f51080c257bd38db95b66f33216:05b1185e47ec480cdeab26d4c62f499a747ac87a701a5830ecfca1eaf3095ba7:query_builder_push:last_activity_sort_expr(schema)",
      "apalis_jobs.rs:fetch_payloads_for_ids:5735685cb0084ad5a0b182cfe13df418fdddb9b4eb2eb212e52b6c4110117814:05b1185e47ec480cdeab26d4c62f499a747ac87a701a5830ecfca1eaf3095ba7:query_builder_push:text_expr(schema, \"id\", \"''\")",
      "apalis_jobs.rs:fetch_payloads_for_ids:5735685cb0084ad5a0b182cfe13df418fdddb9b4eb2eb212e52b6c4110117814:05b1185e47ec480cdeab26d4c62f499a747ac87a701a5830ecfca1eaf3095ba7:query_builder_push:blob_or_text_expr(schema, \"job\")",
      "apalis_jobs.rs:fetch_payloads_for_ids:5735685cb0084ad5a0b182cfe13df418fdddb9b4eb2eb212e52b6c4110117814:05b1185e47ec480cdeab26d4c62f499a747ac87a701a5830ecfca1eaf3095ba7:query_builder_push:blob_or_text_expr(schema, \"last_result\")",
      "apalis_jobs.rs:fetch_payloads_for_ids:5735685cb0084ad5a0b182cfe13df418fdddb9b4eb2eb212e52b6c4110117814:05b1185e47ec480cdeab26d4c62f499a747ac87a701a5830ecfca1eaf3095ba7:query_builder_push:blob_or_text_expr(schema, \"metadata\")",
      "archive_read_model.rs:load_item_rows_from_archive:8dd2a5f19a6a016668b832bf98eab1e0baa0b67cbcd32d3f019edea60a13a8a7:e57b034252ca308fe6c2c72cd803b6542a761604c3aabedf5481b6ef49accf23:query_as::<_, StoredItemRow>:&sql",
      `ingest_provenance.rs:mark_takeout_migrated_history_deferred:c1cf2a46340d983cdc4b17fc9d9b5d55fbb44e1fc86dc8f9c77d3775a738af3d:${ingestProvenanceSourceFingerprint}:query:&query`,
      `ingest_provenance.rs:mark_takeout_only_my_messages_fallback:43259a95d59cdd4db1407e3af3d0937060a333423c5008e0f00b61e9d1c2bf12:${ingestProvenanceSourceFingerprint}:query:&query`,
      `notebooklm_export/query.rs:load_export_messages_from_items_path:18e50b7eb7b9e2033ea62c6df497ea82f566e289e990ba46b3a5a421fbc6d59d:${notebookExportSourceFingerprint}:query_as:&sql`,
      `notebooklm_export/query.rs:load_export_messages_from_items_path:18e50b7eb7b9e2033ea62c6df497ea82f566e289e990ba46b3a5a421fbc6d59d:${notebookExportSourceFingerprint}:query_as:&sql`,
      `notebooklm_export/query.rs:load_export_messages_from_items_path:18e50b7eb7b9e2033ea62c6df497ea82f566e289e990ba46b3a5a421fbc6d59d:${notebookExportSourceFingerprint}:query_as:&sql`,
      `notebooklm_export/query.rs:load_export_messages_from_items_path:18e50b7eb7b9e2033ea62c6df497ea82f566e289e990ba46b3a5a421fbc6d59d:${notebookExportSourceFingerprint}:query_as:&sql`,
      `notebooklm_export/query.rs:load_export_messages_from_archive:${notebookArchiveBodyFingerprint}:${notebookExportSourceFingerprint}:query_as:&sql`,
      `notebooklm_export/query.rs:load_export_messages_from_archive:${notebookArchiveBodyFingerprint}:${notebookExportSourceFingerprint}:query_as:&sql`,
      `notebooklm_export/query.rs:load_export_messages_from_archive:${notebookArchiveBodyFingerprint}:${notebookExportSourceFingerprint}:query_as:&sql`,
      `notebooklm_export/query.rs:load_export_messages_from_archive:${notebookArchiveBodyFingerprint}:${notebookExportSourceFingerprint}:query_as:&sql`,
      `notebooklm_export/query.rs:load_reply_contexts_from_archive:${notebookReplyBodyFingerprint}:${notebookExportSourceFingerprint}:query_as::<_, ReplyLookupRow>:&sql`,
      "sources/items/query.rs:load_scoped_item_rows:c2fc5684aa0042787e68af601b66b3e72eb0dc8b6a5733be08870feafeb3a13f:432d7f537661761553773e9a1b9083e92fd4f873e4205b513aaa3b4d92338add:query_as::<_, BrowsableItemRow>:&sql",
      "sources/items/query.rs:load_item_cursor:2c9ef963c3ec312b3fb89a4863419196380548e5e6afd0100d3a63be07f4de00:432d7f537661761553773e9a1b9083e92fd4f873e4205b513aaa3b4d92338add:query_as::<_, BrowsableItemRow>:&sql",
      `sources/store.rs:delete_source_from_pool:f57c3fb2c5b7e0dab204d1ac4120eb119fde4c13a720a412e03feb126f616e8d:${sourcesStoreSourceFingerprint}:query:&format!( "PRAGMA busy_timeout = {SOURCE_DELETE_BUSY_TIMEOUT_MS}" )`,
      "takeout_import/validation_diagnostics.rs:scalar_i64:f7d827bd898cd0cd75ffb1152e53d54534dbacdd22cdd7befd324847a41dee8f:ee2183e9150108c029ce1482613464456e6323ec80b64400f1f1478c45b277eb:query_scalar:sql",
      "takeout_import/validation_diagnostics.rs:push_mismatch_category:068f1c3248bbbe7d9519155502f1f53d840c219db4a185a0f28c327d400ed473:ee2183e9150108c029ce1482613464456e6323ec80b64400f1f1478c45b277eb:query_scalar:sample_sql",
      "topic_memberships.rs:rebuild_topic_memberships_for_source_on_connection:8492a6d57388761a564774ee507c761263c23740782e3ae236e7c9c2d88d3299:e6e8534af6f578f52a29e408ecd82f27f69f70042129a03521b38bf417af52a3:query:&insert_sql",
      "topic_memberships.rs:resolve_scoped_topic_memberships_on_connection:a2ef104c67e930c6107e1e690175337157663dc31cd11ecaacde2cd791968231:e6e8534af6f578f52a29e408ecd82f27f69f70042129a03521b38bf417af52a3:query:&insert_sql",
      "topic_memberships.rs:resolve_scoped_topic_memberships_on_connection:a2ef104c67e930c6107e1e690175337157663dc31cd11ecaacde2cd791968231:e6e8534af6f578f52a29e408ecd82f27f69f70042129a03521b38bf417af52a3:query_scalar::<_, i64>:&eligible_sql",
    ]);
    const tokenOwners = new Map<string, Set<string>>();
    for (const { relative, source } of files) for (const table of sqlTableTokens(productionRust(source))) {
      if (ownedTables.has(table)) (tokenOwners.get(table) ?? tokenOwners.set(table, new Set()).get(table)!).add(relative);
    }
    for (const table of ownedTables) {
      const owners = tokenOwners.get(table) ?? new Set<string>();
      expect(owners.size, `no production owner recorded for ${table}`).toBeGreaterThan(0);
      for (const owner of owners) expect(portableOwners, `${table} query/DML escaped portable owner`).toContain(owner);
    }
    for (const owner of portableOwners) expect(files.some((file) => file.relative === owner), `missing prepared SQL owner ${owner}`).toBe(true);

    const portablePaths = new Set([...frozenMoves("wholeMoves"), ...frozenMoves("splitMoves")].map((move) => `analysis/${move.before}`));
    const schemaTables = new Set(
      filesWithExtension(path.join(repoRoot, "src-tauri/migrations"), ".sql")
        .flatMap((file) => migrationCreatedTables(readFileSync(file, "utf8"))),
    );
    const foreignTables = new Set([...schemaTables].filter((table) => !ownedTables.has(table)));
    for (const move of [...frozenMoves("wholeMoves"), ...frozenMoves("splitMoves")]) {
      const relative = `analysis/${move.before}`;
      if (!portablePaths.has(relative) || move.before.includes("/tests/") || ["test_schema.rs", "tests_portable.rs"].includes(move.before)) continue;
      const portableSource = productionRust(frozenMoveSource(move));
      expect(
        unresolvedSqlConsumers(portableSource),
        `unresolved executable SQL consumer escaped static ownership in ${relative}`,
      ).toEqual([]);
      expect(
        allLiteralSqlTableOperations(portableSource)
          .map(({ table }) => table)
          .filter((table) => foreignTables.has(table)),
        `foreign SQL escaped app adapter into ${relative}`,
      ).toEqual([]);
    }

    const participantSources = {
      readModel: frozenMoveSource(frozenMoves("splitMoves").find((move) => move.before === "store/owned_read_model.rs")!),
      groups: frozenMoveSource(frozenMoves("splitMoves").find((move) => move.before === "groups_store.rs")!),
      runs: frozenMoveSource(frozenMoves("wholeMoves").find((move) => move.before === "store/runs.rs")!),
    };
    const participantChildNames = [
      "prepare_analysis_run_detail",
      "load_analysis_source_group_member_ids",
    ] as const;
    const participantContracts = [
      {
        source: participantSources.readModel,
        name: "prepare_analysis_run_summaries",
        signature: "pub async fn prepare_analysis_run_summaries( conn: &mut SqliteConnection, filters: AnalysisRunListFilters, matches: Vec<AnalysisForeignLabelMatch>, ) -> AppResult<AnalysisRunSummaryEnrichment>",
        terminals: ["fetch_all(conn)"],
        children: [],
        awaits: 1,
      },
      {
        source: participantSources.readModel,
        name: "prepare_active_analysis_run_summaries",
        signature: "pub async fn prepare_active_analysis_run_summaries( conn: &mut SqliteConnection, run_ids: &HashSet<i64>, ) -> AppResult<AnalysisRunSummaryEnrichment>",
        terminals: ["fetch_all(conn)"],
        children: [],
        awaits: 1,
      },
      {
        source: participantSources.readModel,
        name: "prepare_analysis_run_detail",
        signature: "pub async fn prepare_analysis_run_detail( conn: &mut SqliteConnection, run_id: i64, ) -> AppResult<AnalysisRunDetailEnrichment>",
        terminals: ["fetch_optional(conn)"],
        children: [],
        awaits: 1,
      },
      {
        source: participantSources.readModel,
        name: "prepare_legacy_analysis_chat_run",
        signature: "pub async fn prepare_legacy_analysis_chat_run( conn: &mut SqliteConnection, run_id: i64, ) -> AppResult<AnalysisChatRunEnrichment>",
        terminals: [],
        children: ["prepare_analysis_run_detail(conn, run_id)"],
        awaits: 1,
      },
      {
        source: participantSources.groups,
        name: "load_analysis_source_groups_for_enrichment",
        signature: "pub async fn load_analysis_source_groups_for_enrichment( conn: &mut SqliteConnection, ) -> AppResult<Vec<AnalysisSourceGroupRecord>>",
        terminals: ["fetch_all(&mut *conn)"],
        children: ["load_analysis_source_group_member_ids(&mut *conn, row.id)"],
        awaits: 2,
      },
      {
        source: participantSources.groups,
        name: "load_analysis_source_group_for_enrichment",
        signature: "pub async fn load_analysis_source_group_for_enrichment( conn: &mut SqliteConnection, group_id: i64, ) -> AppResult<Option<AnalysisSourceGroupRecord>>",
        terminals: ["fetch_optional(&mut *conn)"],
        children: ["load_analysis_source_group_member_ids(&mut *conn, row.id)"],
        awaits: 2,
      },
      {
        source: participantSources.runs,
        name: "load_project_analysis_run_aggregates",
        signature: "pub async fn load_project_analysis_run_aggregates( conn: &mut SqliteConnection, project_ids: &[i64], ) -> AppResult<Vec<ProjectAnalysisRunAggregate>>",
        terminals: ["fetch_all(conn)"],
        children: [],
        awaits: 1,
      },
      {
        source: participantSources.runs,
        name: "delete_project_analysis_runs",
        signature: "pub async fn delete_project_analysis_runs( conn: &mut SqliteConnection, project_id: i64, ) -> AppResult<()>",
        terminals: ["execute(&mut *conn)"],
        children: [],
        awaits: 1,
      },
    ] as const;
    expect(participantContracts).toHaveLength(8);
    for (const contract of participantContracts) {
      const selected = rustAsyncFunction(
        contract.source,
        contract.name,
        `borrowed participant ${contract.name}`,
      );
      expect(selected.signature, `${contract.name} exact signature`).toBe(contract.signature);
      expect(selected.signature, `${contract.name} signature capability`).not.toMatch(/\b(?:Pool|SqlitePool)\b/);
      expect(selected.body, `${contract.name} capability`).not.toMatch(/\bSqlitePool\b|(?:\.|::)(?:acquire|begin|commit|rollback)\s*\(/);
      expect(directSqlTerminals(selected.body), `${contract.name} exact SQL terminals`).toEqual(contract.terminals);
      expect(
        awaitedNamedCalls(selected.body, participantChildNames),
        `${contract.name} exact async children`,
      ).toEqual(contract.children);
      expect(awaitSiteCount(selected.body), `${contract.name} exact await sites`).toBe(contract.awaits);
      expect(
        transactionControlSql(selected.body),
        `${contract.name} cannot issue transaction-control SQL`,
      ).toEqual([]);
      const builderCalls = maskRustComments(selected.body, true).match(
        /\bbuild_analysis_run_list_query\s*\(\s*filters\s*,\s*&matches\s*\)/g,
      ) ?? [];
      expect(builderCalls, `${contract.name} list-query builder edge`).toHaveLength(
        contract.name === "prepare_analysis_run_summaries" ? 1 : 0,
      );
    }
    const listQueryBuilder = uniqueRustBracedBody(
      participantSources.readModel,
      /fn\s+build_analysis_run_list_query\s*\([\s\S]*?\)\s*->\s*AppResult<QueryBuilder<'static,\s*Sqlite>>\s*\{/,
      "analysis-run list query builder",
    );
    expect(
      transactionControlSql(listQueryBuilder),
      "analysis-run list query builder cannot append transaction-control SQL",
    ).toEqual([]);
    expect(
      transactionControlSql(participantSources.readModel),
      "analysis-run read-model constants cannot inject transaction-control SQL",
    ).toEqual([]);

    const nestedAndAppHelpers = [
      {
        source: participantSources.groups,
        name: "load_analysis_source_group_member_ids",
        signature: "async fn load_analysis_source_group_member_ids( conn: &mut SqliteConnection, group_id: i64, ) -> AppResult<Vec<i64>>",
        terminals: ["fetch_all(&mut *conn)"],
        awaits: 1,
      },
      {
        source: readAppAnalysisSource("store/read_model.rs"),
        name: "match_foreign_labels",
        signature: "async fn match_foreign_labels( conn: &mut SqliteConnection, terms: &[String], ) -> AppResult<Vec<AnalysisForeignLabelMatch>>",
        terminals: ["fetch_all(&mut *conn)", "fetch_all(&mut *conn)"],
        awaits: 2,
      },
      {
        source: readAppAnalysisSource("store/read_model.rs"),
        name: "load_foreign_labels",
        signature: "async fn load_foreign_labels( conn: &mut SqliteConnection, refs: Vec<AnalysisForeignLabelRef>, ) -> AppResult<AnalysisForeignLabels>",
        terminals: ["fetch_all(&mut *conn)", "fetch_all(&mut *conn)"],
        awaits: 2,
      },
      {
        source: readAppAnalysisSource("store/setup.rs"),
        name: "enrich_analysis_source_group",
        signature: "async fn enrich_analysis_source_group( conn: &mut SqliteConnection, record: AnalysisSourceGroupRecord, ) -> AppResult<AnalysisSourceGroup>",
        terminals: ["fetch_all(&mut *conn)"],
        awaits: 1,
      },
    ] as const;
    expect(nestedAndAppHelpers).toHaveLength(4);
    for (const contract of nestedAndAppHelpers) {
      const selected = rustAsyncFunction(contract.source, contract.name, contract.name);
      expect(selected.signature, `${contract.name} exact signature`).toBe(contract.signature);
      expect(selected.body, `${contract.name} closed connection capability`).not.toMatch(
        /\bSqlitePool\b|(?:\.|::)(?:acquire|begin|commit|rollback)\s*\(/,
      );
      expect(directSqlTerminals(selected.body), `${contract.name} exact SQL terminals`).toEqual(contract.terminals);
      expect(awaitSiteCount(selected.body), `${contract.name} exact await sites`).toBe(contract.awaits);
      expect(
        transactionControlSql(selected.body),
        `${contract.name} cannot issue transaction-control SQL`,
      ).toEqual([]);
    }

    const coordinatorChildNames = [
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
    const synchronousBypassProbe = `
      detached(pool.clone());
      block_on(load());
      tokio::spawn(run());
    `;
    expect(unqualifiedFreeFunctionCalls(synchronousBypassProbe)).toEqual([
      "detached",
      "block_on",
      "load",
      "run",
    ]);
    expect(detachedCoordinatorBypasses(synchronousBypassProbe)).toEqual([
      "block_on",
      "detached",
      "pool.clone",
      "spawn",
    ]);
    expect(qualifiedCallablePaths(`
      self::refresh();
      helpers::refresh();
      sqlx::query("SELECT 1");
      let mapper = AppError::database;
      value.method();
      Type::new();
    `)).toEqual([
      "self::refresh",
      "helpers::refresh",
      "sqlx::query",
      "AppError::database",
    ]);
    const coordinators = [
      {
        label: "Family 1 list",
        relative: "store/read_model.rs",
        name: "list_analysis_runs_in_pool",
        awaits: 5,
        freeCalls: [
          "match_foreign_labels",
          "prepare_analysis_run_summaries",
          "load_foreign_labels",
        ],
        qualifiedCalls: [
          "AppError::database",
          "AppError::database",
        ],
        topology: [
          "begin",
          "match_foreign_labels(&mut *transaction, filters.foreign_label_search_terms())",
          "prepare_analysis_run_summaries(&mut *transaction, filters, matches)",
          "load_foreign_labels(&mut *transaction, enrichment.foreign_label_refs())",
          "commit",
        ],
      },
      {
        label: "Family 1 active",
        relative: "store/read_model.rs",
        name: "list_active_analysis_runs_in_pool",
        awaits: 4,
        freeCalls: [
          "prepare_active_analysis_run_summaries",
          "load_foreign_labels",
        ],
        qualifiedCalls: [
          "AppError::database",
          "AppError::database",
        ],
        topology: [
          "begin",
          "prepare_active_analysis_run_summaries(&mut *transaction, run_ids)",
          "load_foreign_labels(&mut *transaction, enrichment.foreign_label_refs())",
          "commit",
        ],
      },
      {
        label: "Family 1 detail",
        relative: "store/read_model.rs",
        name: "get_analysis_run_in_pool",
        awaits: 4,
        freeCalls: [
          "prepare_analysis_run_detail",
          "load_foreign_labels",
        ],
        qualifiedCalls: [
          "AppError::database",
          "AppError::database",
        ],
        topology: [
          "begin",
          "prepare_analysis_run_detail(&mut *transaction, run_id)",
          "load_foreign_labels(&mut *transaction, enrichment.foreign_label_refs())",
          "commit",
        ],
      },
      {
        label: "Family 1 legacy",
        relative: "store/read_model.rs",
        name: "resolve_legacy_analysis_chat_run_in_pool",
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
        label: "Family 2 list",
        relative: "store/setup.rs",
        name: "list_analysis_source_groups_in_pool",
        awaits: 4,
        freeCalls: [
          "load_analysis_source_groups_for_enrichment",
          "enrich_analysis_source_group",
        ],
        qualifiedCalls: [
          "AppError::database",
          "AppError::database",
        ],
        topology: [
          "begin",
          "load_analysis_source_groups_for_enrichment(&mut *transaction)",
          "enrich_analysis_source_group(&mut *transaction, record)",
          "commit",
        ],
      },
      {
        label: "Family 2 one",
        relative: "store/setup.rs",
        name: "get_analysis_source_group_response_in_pool",
        awaits: 4,
        freeCalls: [
          "load_analysis_source_group_for_enrichment",
          "enrich_analysis_source_group",
        ],
        qualifiedCalls: [
          "AppError::database",
          "AppError::database",
        ],
        topology: [
          "begin",
          "load_analysis_source_group_for_enrichment(&mut *transaction, group_id)",
          "enrich_analysis_source_group(&mut *transaction, record)",
          "commit",
        ],
      },
      {
        label: "Family 2 NotebookLM",
        relative: "../notebooklm_export/query.rs",
        name: "load_export_source_group_in_pool",
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
        label: "Family 3 project list",
        relative: "../projects/read_model.rs",
        name: "list_research_projects_in_pool",
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
        label: "Family 4 project delete",
        relative: "../projects/mod.rs",
        name: "delete_project_in_pool",
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
    expect(coordinators).toHaveLength(9);
    for (const contract of coordinators) {
      const source = contract.relative.startsWith("../")
        ? readFileSync(path.join(currentAnalysisRoot, contract.relative), "utf8")
        : readAppAnalysisSource(contract.relative);
      const selected = rustAsyncFunction(source, contract.name, `${contract.label} coordinator`);
      expect(
        asyncTopology(selected.body, coordinatorChildNames),
        `${contract.label} exact async topology`,
      ).toEqual(contract.topology);
      expect(awaitSiteCount(selected.body), `${contract.label} exact await sites`).toBe(contract.awaits);
      expect(
        unqualifiedFreeFunctionCalls(selected.body),
        `${contract.label} exact unqualified/local free calls`,
      ).toEqual(contract.freeCalls);
      expect(
        qualifiedCallablePaths(selected.body),
        `${contract.label} exact qualified callable paths`,
      ).toEqual(contract.qualifiedCalls);
      expect(
        detachedCoordinatorBypasses(selected.body),
        `${contract.label} cannot detach work or clone the pool`,
      ).toEqual([]);
      expect(
        transactionControlSql(selected.body),
        `${contract.label} cannot issue transaction-control SQL`,
      ).toEqual([]);
      expect(
        unresolvedSqlConsumers(selected.body),
        `${contract.label} cannot hide SQL behind an unresolved consumer`,
      ).toEqual([]);
      expect(maskRustComments(selected.body, true), `${contract.label} extra connection capability`).not.toMatch(
        /(?:\.|::)(?:acquire|connect|rollback)\s*\(/,
      );
      expect(
        allLiteralSqlTableOperations(selected.body)
          .map(({ table }) => table)
          .filter((table) => ownedTables.has(table)),
        `${contract.label} raw owned SQL`,
      ).toEqual([]);
    }
  }, 20_000);

  it("limits moving crate-test foreign sentinels and foreign-key suppression", () => {
    expect(borrowedParticipantCalls(
      '/* participant(&mut *transaction) */ let note = "participant(&mut *transaction)";',
      "participant",
    )).toEqual([]);
    const blockCommentProbe = 'sqlx::query("SELECT * FROM /* ownership gap */ sources")';
    const blockCommentOperations = sqlTableOperations(blockCommentProbe);
    expect(blockCommentOperations.map(
      ({ operation, table }) => `${operation}:${table}`,
    )).toEqual(["FROM:sources"]);
    expect(blockCommentOperations[0]?.offset).toBe(blockCommentProbe.indexOf("FROM"));
    expect(blockCommentOperations[0]?.callStart).toBe(blockCommentProbe.indexOf("sqlx::query"));
    expect(blockCommentOperations[0]?.callEnd).toBe(blockCommentProbe.length);
    const quotedIdentifierProbe = String.raw`sqlx::query("SELECT * FROM \"analysis_runs\"")`;
    expect(sqlTableOperations(quotedIdentifierProbe).map(
      ({ table }) => table,
    )).toEqual(["analysis_runs"]);
    const quotedSchemaProbe = String.raw`sqlx::query("SELECT * FROM main.\"analysis_runs\"")`;
    expect(sqlTableOperations(quotedSchemaProbe).map(
      ({ table }) => table,
    )).toEqual(["analysis_runs"]);
    const cookedLineCommentProbe = String.raw`sqlx::query("SELECT * FROM -- hide\n sources")`;
    expect(sqlTableOperations(cookedLineCommentProbe).map(
      ({ table }) => table,
    )).toEqual(["sources"]);
    expect(sqlTableOperations(
      'sqlx::query("DELETE /* ownership gap */ FROM analysis_runs WHERE id = -1")',
    ).map(({ operation, table }) => `${operation}:${table}`)).toEqual(["DELETE FROM:analysis_runs"]);
    expect(sqlTableOperations(
      'sqlx::query("SELECT * FROM -- ownership gap\n sources")',
    ).map(({ operation, table }) => `${operation}:${table}`)).toEqual(["FROM:sources"]);
    expect(sqlTableOperations(
      'sqlx::query("SELECT * FROM main /* ownership gap */ . -- dot gap\n sources")',
    ).map(({ operation, table }) => `${operation}:${table}`)).toEqual(["FROM:sources"]);
    expect(sqlTableOperations(
      'sqlx::QueryBuilder::<Sqlite>::new("SELECT * FROM /* ownership gap */ sources")',
    ).map(({ operation, table }) => `${operation}:${table}`)).toEqual(["FROM:sources"]);
    const concatSqlProbe = 'sqlx::query(concat!("SELECT * FROM ", "sources"))';
    expect(sqlTableOperations(concatSqlProbe).map(
      ({ operation, table }) => `${operation}:${table}`,
    )).toEqual(["FROM:sources"]);
    const assembledBuilderProbe = `
      use sqlx::QueryBuilder;
      let mut query = QueryBuilder::<Sqlite>::new("SELECT * FROM ");
      query.push("sou").push("rces");
    `;
    expect(sqlTableOperations(assembledBuilderProbe).map(
      ({ operation, table }) => `${operation}:${table}`,
    )).toEqual(["FROM:sources"]);
    expect(sqlTableOperations(
      'sqlx::QueryBuilder::<Sqlite>::new("SELECT * FROM ").push("sou").push("rces")',
    ).map(({ operation, table }) => `${operation}:${table}`)).toEqual(["FROM:sources"]);
    const transitiveNegativeAliasProbe = `
      async fn let_alias_probe() {
        let sentinel = -2_i64;
        let intermediate = sentinel;
        let id = intermediate;
        sqlx::query("SELECT * FROM alias_targets WHERE id = ?")
          .bind(id)
          .fetch_all(&pool)
          .await;
      }
      async fn mixed_alias_probe() {
        const SENTINEL: i64 = -3_i64;
        static INTERMEDIATE: i64 = SENTINEL;
        let id = INTERMEDIATE;
        sqlx::query("SELECT * FROM mixed_alias_targets WHERE id = ?")
          .bind(id)
          .fetch_all(&pool)
          .await;
      }
    `;
    expect(negativeForeignReadOperations(
      transitiveNegativeAliasProbe,
      new Set(["alias_targets", "mixed_alias_targets"]),
    ).map(({ table }) => table).sort()).toEqual([
      "alias_targets",
      "mixed_alias_targets",
    ]);
    expect(transactionControlSql(
      'sqlx::query("SELECT 1; /* decoy */ COMMIT; -- hidden\\n SAVEPOINT nested")',
    )).toEqual(["COMMIT", "SAVEPOINT"]);
    expect(transactionControlSql('sqlx::query("COMMIT")')).toEqual(["COMMIT"]);
    expect(transactionControlSql(`
      async fn participant(transaction: &mut Transaction<'_, Sqlite>) -> AppResult<()> {
        sqlx::raw_sql("COMMIT").execute(&mut **transaction).await?;
        Ok(())
      }
    `)).toEqual(["COMMIT"]);
    expect(transactionControlSql(`
      async fn coordinator(pool: &SqlitePool) -> AppResult<()> {
        let mut transaction = pool.begin().await?;
        sqlx::query("COMMIT").execute(&mut *transaction).await?;
        transaction.commit().await?;
        Ok(())
      }
    `)).toEqual(["COMMIT"]);
    expect(transactionControlSql(`
      async fn coordinator(pool: &SqlitePool) -> AppResult<()> {
        let mut transaction = pool.begin().await?;
        sqlx::raw_sql(concat!("COM", "MIT"))
          .execute(&mut *transaction)
          .await?;
        transaction.commit().await?;
        Ok(())
      }
    `)).toEqual(["COMMIT"]);
    const identifierBackedBuilderProbe = `
      use sqlx::QueryBuilder;
      const TABLE: &str = concat!("analysis_", "runs");
      let mut query = QueryBuilder::<Sqlite>::new("SELECT * FROM ");
      query.push(TABLE);
    `;
    expect(sqlTableOperations(identifierBackedBuilderProbe).map(
      ({ table }) => table,
    )).toEqual(["analysis_runs"]);
    const associationProbe = `
      sqlx::query("INSERT INTO sources (id) VALUES (-1)").execute(&pool).await;
      sqlx::query("INSERT INTO sources (id) VALUES (2)").execute(&pool).await;
    `;
    const probeOperations = sqlTableOperations(associationProbe);
    expect(probeOperations[0]?.callEnd).toBe(associationProbe.indexOf(").execute") + 1);
    expect(probeOperations[1]?.callStart).toBeGreaterThan(probeOperations[0]?.callEnd ?? 0);
    expect(() => assertNegativeSentinelIdentifiers(associationProbe, probeOperations[1]!)).toThrow(/negative sentinel/);
    expect(() => assertNegativeSentinelIdentifiers(
      'sqlx::query("INSERT INTO sources (id) VALUES (-1), (2)")',
      sqlTableOperations('sqlx::query("INSERT INTO sources (id) VALUES (-1), (2)")')[0]!,
    )).toThrow(/negative sentinel/);
    const multiStatementProbe = 'sqlx::query("INSERT INTO sources (id) VALUES (-1); INSERT INTO items (id, source_id) VALUES (2, -1)")';
    const multiStatementOperations = sqlTableOperations(multiStatementProbe);
    expect(multiStatementOperations).toHaveLength(2);
    expect(() => assertNegativeSentinelIdentifiers(
      multiStatementProbe,
      multiStatementOperations[0]!,
    )).not.toThrow();
    expect(() => assertNegativeSentinelIdentifiers(
      multiStatementProbe,
      multiStatementOperations[1]!,
    )).toThrow(/negative sentinel/);
    expect(() => assertNegativeSentinelIdentifiers(
      'sqlx::query("INSERT INTO sources (title) VALUES (\'x\')")',
      sqlTableOperations('sqlx::query("INSERT INTO sources (title) VALUES (\'x\')")')[0]!,
    )).toThrow(/identifier/);
    expect(() => assertNegativeSentinelIdentifiers(
      'sqlx::query("DELETE FROM sources WHERE id = -1 AND source_id = 2")',
      sqlTableOperations('sqlx::query("DELETE FROM sources WHERE id = -1 AND source_id = 2")')[0]!,
    )).toThrow(/not authorized/);
    expect(() => assertNegativeSentinelIdentifiers(
      'sqlx::query("DELETE FROM sources WHERE id = -1 AND source_id > 0")',
      sqlTableOperations('sqlx::query("DELETE FROM sources WHERE id = -1 AND source_id > 0")')[0]!,
    )).toThrow(/not authorized/);
    expect(() => assertNegativeSentinelIdentifiers(
      'sqlx::query("DELETE FROM sources WHERE id = -1 OR 1 = 1")',
      sqlTableOperations('sqlx::query("DELETE FROM sources WHERE id = -1 OR 1 = 1")')[0]!,
    )).toThrow(/not authorized/);
    expect(preRestoreControlFlow(
      'let insert_result = sqlx::query("INSERT INTO analysis_source_group_members (group_id, source_id) VALUES (?, ?)").execute(&mut *connection).await; return; let restore_result = sqlx::query("PRAGMA foreign_keys = ON")',
    )).toContain("return");
    expect(preRestoreControlFlow("assert_eq!(1, 2); ")).toContain("assert_eq!(");
    const bypassProbe = `
      async fn hidden_helper() { sqlx::query("INSERT INTO sources (id) VALUES (-1)"); }
      #[test] fn allowed_body() { sqlx::query("INSERT INTO sources (id) VALUES (-2)"); }
    `;
    const allowedStart = bypassProbe.indexOf("allowed_body");
    expect(
      foreignOperationsOutsideAllowedBodies(
        bypassProbe,
        [{ start: allowedStart, end: bypassProbe.length }],
        new Set(["sources"]),
    ).map((operation) => operation.table),
    ).toEqual(["sources"]);
    const negativeReadProbe = 'sqlx::query("SELECT * FROM sources WHERE id = -2").fetch_all(&pool).await;';
    expect(negativeForeignReadOperations(
      negativeReadProbe,
      new Set(["sources"]),
    ).map(({ operation, table }) => `${operation}:${table}`)).toEqual(["FROM:sources"]);
    const negativeBindReadProbe = `
      use sqlx::QueryBuilder;
      const SENTINEL_ID: i64 = -4;
      async fn suffixed_probe() {
        sqlx::query("SELECT * FROM sources WHERE id = ?").bind(-2_i64).fetch_all(&pool).await;
      }
      async fn radix_probe() {
        let mut query = QueryBuilder::<Sqlite>::new("SELECT * FROM items WHERE source_id = ");
        query.push_bind(-0x2_i64);
        query.build().fetch_all(&pool).await;
      }
      async fn named_probe() {
        sqlx::query("SELECT * FROM projects WHERE id = ?").bind(SENTINEL_ID).fetch_all(&pool).await;
      }
    `;
    expect(negativeForeignReadOperations(
      negativeBindReadProbe,
      new Set(["items", "projects", "sources"]),
    ).map(({ table }) => table).sort()).toEqual(["items", "projects", "sources"]);
    const pragmaSpoofProbe = String.raw`
      let comment_decoy = "PRAGMA foreign_keys = OFF";
      sqlx::query("SELECT 'PRAGMA foreign_keys = OFF'");
      sqlx::query("/* PRAGMA foreign_keys = OFF */ SELECT 1");
    `;
    expect(sqlxQueryLiterals(pragmaSpoofProbe).map(
      ({ kind, sql }) => `${kind}:${normalized(sql)}`,
    )).toEqual([
      "query:SELECT 'PRAGMA foreign_keys = OFF'",
      "query:/* PRAGMA foreign_keys = OFF */ SELECT 1",
    ]);
    expect(foreignKeyDisableQueries(pragmaSpoofProbe)).toEqual([]);
    const pragmaFalseForms = String.raw`
      sqlx::query("pragma FOREIGN_KEYS = off");
      sqlx::query("PRAGMA foreign_keys(0)");
      sqlx::query("PRAGMA foreign_keys = FALSE");
      sqlx::query("PRAGMA foreign_keys(NO)");
      sqlx::query("PRAGMA foreign_keys='OFF'");
      sqlx::query("PRAGMA \"foreign_keys\"=\"OFF\"");
      sqlx::query("PRAGMA [foreign_keys]='FALSE'");
      sqlx::query("PRAGMA 'main'.foreign_keys='NO'");
      sqlx::query("PRAGMA foreign_keys=-0");
      sqlx::query("PRAGMA foreign_keys=+0");
      sqlx::query("PRAGMA foreign_keys=00");
      sqlx::query("PRAGMA foreign_keys=0x0");
      sqlx::query("PRAGMA foreign_keys='0'");
      sqlx::query("PRAGMA foreign_keys=\"00\"");
      sqlx::query("PRAGMA foreign_keys='-0'");
      sqlx::query("PRAGMA foreign_keys='+0'");
      sqlx::query("PRAGMA foreign_keys='0x0'");
      sqlx::query("PRAGMA foreign_keys=xyz");
      sqlx::query("PRAGMA foreign_keys='full'");
      sqlx::query("PRAGMA foreign_keys=-1");
      sqlx::query("PRAGMA foreign_keys=0.5");
      sqlx::query("PRAGMA foreign_keys=.5");
      sqlx::query("PRAGMA foreign_keys=' ON '");
      sqlx::query("PRAGMA foreign_keys='TRUE '");
      sqlx::query("PRAGMA foreign_keys=' YES'");
      sqlx::query("PRAGMA foreign_keys=' 1 '");
      sqlx::query("PRAGMA foreign_keys='0_1'");
      sqlx::query("PRAGMA foreign_keys='T_R_U_E'");
    `;
    expect(foreignKeyDisableQueries(pragmaFalseForms).map(
      ({ sql }) => normalized(sql),
    )).toEqual([
      "pragma FOREIGN_KEYS = off",
      "PRAGMA foreign_keys(0)",
      "PRAGMA foreign_keys = FALSE",
      "PRAGMA foreign_keys(NO)",
      "PRAGMA foreign_keys='OFF'",
      "PRAGMA \"foreign_keys\"=\"OFF\"",
      "PRAGMA [foreign_keys]='FALSE'",
      "PRAGMA 'main'.foreign_keys='NO'",
      "PRAGMA foreign_keys=-0",
      "PRAGMA foreign_keys=+0",
      "PRAGMA foreign_keys=00",
      "PRAGMA foreign_keys=0x0",
      "PRAGMA foreign_keys='0'",
      "PRAGMA foreign_keys=\"00\"",
      "PRAGMA foreign_keys='-0'",
      "PRAGMA foreign_keys='+0'",
      "PRAGMA foreign_keys='0x0'",
      "PRAGMA foreign_keys=xyz",
      "PRAGMA foreign_keys='full'",
      "PRAGMA foreign_keys=-1",
      "PRAGMA foreign_keys=0.5",
      "PRAGMA foreign_keys=.5",
      "PRAGMA foreign_keys=' ON '",
      "PRAGMA foreign_keys='TRUE '",
      "PRAGMA foreign_keys=' YES'",
      "PRAGMA foreign_keys=' 1 '",
      "PRAGMA foreign_keys='0_1'",
      "PRAGMA foreign_keys='T_R_U_E'",
    ]);
    const pragmaEnabledForms = String.raw`
      sqlx::query("PRAGMA foreign_keys=ON");
      sqlx::query("PRAGMA foreign_keys='TRUE'");
      sqlx::query("PRAGMA foreign_keys(YES)");
      sqlx::query("PRAGMA foreign_keys=1");
      sqlx::query("PRAGMA foreign_keys=+2");
      sqlx::query("PRAGMA foreign_keys=0x1");
      sqlx::query("PRAGMA foreign_keys=1.5");
      sqlx::query("PRAGMA foreign_keys");
    `;
    expect(foreignKeyDisableQueries(pragmaEnabledForms)).toEqual([]);
    expect(foreignKeyDisableQueries(
      'sqlx::raw_sql("PRAGMA foreign_keys = OFF")',
    ).map(({ kind, sql }) => `${kind}:${normalized(sql)}`)).toEqual([
      "raw_sql:PRAGMA foreign_keys = OFF",
    ]);
    const fragmentedBuilderPragma = `
      use sqlx::QueryBuilder;
      let mut query = QueryBuilder::<Sqlite>::new("PRA");
      query.push("GMA foreign_").push("keys = OFF");
    `;
    expect(foreignKeyDisableQueries(fragmentedBuilderPragma).map(
      ({ sql }) => normalized(sql),
    )).toEqual(["PRAGMA foreign_keys = OFF"]);
    const snapshot = frozenMoveSource(frozenMoves("wholeMoves").find((move) => move.before === "corpus/tests/snapshot.rs")!);
    const noLiveFallback = new Set(["list_run_snapshot_messages_page_does_not_fall_back_to_live_source", "load_run_corpus_messages_does_not_reconstruct_completed_capture_failed_from_live_rows", "trace_resolution_does_not_fall_back_to_live_source_for_completed_missing_snapshot"]);
    const ownedTables = new Set(["analysis_chat_messages", "analysis_prompt_templates", "analysis_run_messages", "analysis_runs", "analysis_source_group_members", "analysis_source_groups"]);
    const foreignSchemaTables = new Set(
      filesWithExtension(path.join(repoRoot, "src-tauri/migrations"), ".sql")
        .flatMap((file) => migrationCreatedTables(readFileSync(file, "utf8")))
        .filter((table) => !ownedTables.has(table)),
    );
    const expectedSentinelTables = new Map([
      ["list_run_snapshot_messages_page_does_not_fall_back_to_live_source", ["items", "sources"]],
      ["trace_resolution_does_not_fall_back_to_live_source_for_completed_missing_snapshot", ["items", "sources"]],
      ["load_run_corpus_messages_does_not_reconstruct_completed_capture_failed_from_live_rows", ["analysis_documents", "items", "sources"]],
    ]);
    const setupEndMarkers = new Map([
      ["list_run_snapshot_messages_page_does_not_fall_back_to_live_source", "let page = list_run_snapshot_messages_page("],
      ["trace_resolution_does_not_fall_back_to_live_source_for_completed_missing_snapshot", "let messages = load_trace_resolution_messages("],
      ["load_run_corpus_messages_does_not_reconstruct_completed_capture_failed_from_live_rows", "let corpus = load_run_corpus_messages("],
    ]);
    const sentinelTests = rustTestBodies(snapshot).filter((test) => sqlTableOperations(test.body).some(({ table }) => foreignSchemaTables.has(table)));
    expect(sentinelTests.map((test) => test.name).sort()).toEqual([...noLiveFallback].sort());
    const canonicalPool = "test_schema::analysis_test_pool().await";
    const setupRanges = sentinelTests.map((test) => {
      const poolOffsets = literalOffsets(test.body, canonicalPool);
      expect(poolOffsets, `${test.name} canonical pool count`).toHaveLength(1);
      const endMarker = setupEndMarkers.get(test.name);
      expect(endMarker, `${test.name} setup end marker`).toBeTypeOf("string");
      const endOffsets = endMarker ? literalOffsets(test.body, endMarker) : [];
      expect(endOffsets, `${test.name} setup end marker count`).toHaveLength(1);
      const start = poolOffsets[0] + canonicalPool.length;
      const end = endOffsets[0];
      expect(end, `${test.name} setup follows canonical pool`).toBeGreaterThan(start);
      return {
        name: test.name,
        local: { start, end },
        absolute: { start: test.start + start, end: test.start + end },
      };
    });
    expect(
      foreignOperationsOutsideAllowedBodies(
        snapshot,
        setupRanges.map(({ absolute }) => absolute),
        foreignSchemaTables,
      ),
      "foreign SQL outside the three exact authorized setup ranges",
    ).toEqual([]);
    for (const test of sentinelTests) {
      expect(noLiveFallback, `${test.name} foreign sentinel exception`).toContain(test.name);
      const setup = setupRanges.find(({ name }) => name === test.name)?.local;
      if (!setup) throw new Error(`missing ${test.name} exact setup range`);
      expect(test.body).toMatch(/VALUES\s*\(\s*-\d+/i);
      const foreignOperations = sqlTableOperations(test.body).filter(({ table }) => foreignSchemaTables.has(table));
      expect(foreignOperations.map(({ table }) => table).sort()).toEqual(expectedSentinelTables.get(test.name));
      for (const operation of foreignOperations) {
        expect(operation.operation, `${test.name} foreign operation`).toBe("INSERT INTO");
        expect(operation.callStart, `${test.name} foreign SQL call starts in setup`).toBeGreaterThanOrEqual(setup.start);
        expect(operation.offset, `${test.name} foreign operation starts in setup`).toBeGreaterThanOrEqual(setup.start);
        expect(operation.callEnd, `${test.name} foreign SQL call ends in setup`).toBeLessThanOrEqual(setup.end);
        expect(() => assertNegativeSentinelIdentifiers(test.body, operation), `${test.name} negative sentinels`).not.toThrow();
      }
    }
    expect(snapshot).toMatch(/\nasync\s+fn\s+insert_orphan_group_member_with_foreign_keys_restored\s*\(/);
    expect(snapshot).not.toMatch(/\bpub(?:\([^)]*\))?\s+async\s+fn\s+insert_orphan_group_member_with_foreign_keys_restored\s*\(/);
    const helper = uniqueRustBracedBody(snapshot, /async\s+fn\s+insert_orphan_group_member_with_foreign_keys_restored\s*\([\s\S]*?\)\s*\{/, "private foreign-key restoration helper");
    expect(sqlxQueryCalls(helper), "foreign-key helper exact sqlx query call count").toHaveLength(4);
    expect(sqlxQueryLiterals(helper).map(
      ({ kind, sql }) => `${kind}:${normalized(sql)}`,
    ), "exact foreign-key helper SQL query sequence").toEqual([
      "query:PRAGMA foreign_keys = OFF",
      "query:INSERT INTO analysis_source_group_members (group_id, source_id, created_at) VALUES (?, ?, 1)",
      "query:PRAGMA foreign_keys = ON",
      "query_scalar::<_, i64>:PRAGMA foreign_keys",
    ]);
    expect(helper).toContain("PRAGMA foreign_keys = OFF");
    expect(helper).toContain("PRAGMA foreign_keys = ON");
    expect(helper).toContain("foreign keys must be restored before releasing the sole connection");
    expect(sqlTableOperations(helper).filter(({ table }) => foreignSchemaTables.has(table))).toEqual([]);
    expect(foreignKeyDisableQueries(snapshot)).toHaveLength(1);
    expect(snapshot.match(/PRAGMA\s+foreign_keys\s*=\s*ON/g) ?? []).toHaveLength(1);
    expectOrdered(helper, ["PRAGMA foreign_keys = OFF", "INSERT INTO analysis_source_group_members", "PRAGMA foreign_keys = ON", "PRAGMA foreign_keys", "assert_eq!("]);
    expect(helper).toMatch(/assert_eq!\(\s*restored_result[\s\S]*?\b1\s*,/);
    const capturedInsert = helper.indexOf("let insert_result =");
    const restoreDeclaration = helper.indexOf("let restore_result =");
    const restored = helper.indexOf("PRAGMA foreign_keys = ON");
    const asserted = helper.indexOf("assert_eq!(");
    const propagated = helper.indexOf("insert_result.expect");
    expect(capturedInsert, "capture orphan insert result").toBeGreaterThanOrEqual(0);
    const preRestore = helper.slice(capturedInsert, restoreDeclaration);
    expect(preRestoreControlFlow(preRestore), "orphan insert cannot exit before restore").toEqual([]);
    expect(normalized(maskRustComments(preRestore, true)), "capture then immediate restore").toBe(
      normalized("let insert_result = sqlx::query( , ) .bind(group_id) .bind(source_id) .execute(&mut *connection) .await;"),
    );
    expect(
      normalized(maskRustComments(helper.slice(capturedInsert), true)),
      "capture-through-propagation foreign-key restoration suffix",
    ).toBe(normalized(`
      let insert_result = sqlx::query( , )
        .bind(group_id)
        .bind(source_id)
        .execute(&mut *connection)
        .await;
      let restore_result = sqlx::query( )
        .execute(&mut *connection)
        .await;
      let restored_result = sqlx::query_scalar::<_, i64>( )
        .fetch_one(&mut *connection)
        .await;
      restore_result.expect( );
      assert_eq!(
        restored_result.expect( ),
        1,
      );
      drop(connection);
      insert_result.expect( );
    `));
    expect(restoreDeclaration, "start restoration").toBeGreaterThan(capturedInsert);
    expect(restored, "restore foreign keys").toBeGreaterThan(restoreDeclaration);
    expect(asserted, "assert restored foreign keys").toBeGreaterThan(restored);
    expect(propagated, "propagate captured insert after restore assertion").toBeGreaterThan(asserted);
    const drift = rustTestBodies(snapshot).find((test) => test.name === "source_group_membership_drift_after_capture_does_not_change_saved_run_corpus");
    expect(drift?.body).toContain("insert_orphan_group_member_with_foreign_keys_restored");
    expect(snapshot.match(/insert_orphan_group_member_with_foreign_keys_restored\(/g) ?? []).toHaveLength(2);

    const movingSources = [...frozenMoves("wholeMoves"), ...frozenMoves("splitMoves")]
      .filter((move) => move.before !== "test_schema.rs")
      .map((move) => ({ relative: move.before, source: frozenMoveSource(move) }));
    const unresolvedMovingConsumers = movingSources.flatMap(({ relative, source }) =>
      unresolvedConsumerInventory(relative, source).map(unresolvedConsumerIdentity));
    const chatExecutionFingerprint = analysisExtracted
      ? "551123334a658eeb318b997d0167ef9026bde7121ff9e61822bbe23e2ddef81c"
      : "80725a1f678c48be1b0b10b0e20840bb7ba238a2a4eeb2d5be04602405bf1e0f";
    const chatSourceFingerprint = analysisExtracted
      ? "9ba11787faa64f211c763876fa63fac51bd7dd8f767dbc5f0470919d4de44f37"
      : "26f83ba867a65bc4e99902fe804827ac6ebaeb83a862f8970f271f628b88bb17";
    expect(
      unresolvedMovingConsumers,
      "moving source unresolved executable SQL consumer inventory",
    ).toEqual([
      `chat_engine.rs:tests::chat_execution_persists_turns_before_completed_event:${chatExecutionFingerprint}:${chatSourceFingerprint}:query:statement`,
    ]);
    for (const { relative, source } of movingSources) {
      const executable = maskRustComments(source, true);
      expect(executable, `${relative} imports or calls an app source builder`).not.toMatch(/\b(?:AppAnalysisCorpusReader|AppAnalysisRunPreflightLimits|AppYoutubeCorpusMode|apply_all_migrations_for_test_pool|create_test_pool)\b/);
      const allowed = relative === "corpus/tests/snapshot.rs"
        ? setupRanges.map(({ absolute }) => absolute) : [];
      expect(
        foreignOperationsOutsideAllowedBodies(source, allowed, foreignSchemaTables),
        `${relative} foreign SQL outside authorized bodies`,
      ).toEqual([]);
    }
    for (const move of [...frozenMoves("wholeMoves"), ...frozenMoves("splitMoves")]) {
      const disabled = foreignKeyDisableQueries(frozenMoveSource(move));
      expect(disabled, `${move.before} foreign-key suppression`).toHaveLength(move.before === "corpus/tests/snapshot.rs" ? 1 : 0);
    }
    const appSourceRoot = path.join(repoRoot, "src-tauri/src");
    const crateSourceRoot = path.join(repoRoot, "src-tauri/crates/extractum-analysis/src");
    const fullAppInventory = rustFiles(appSourceRoot).map((file) => ({
      relative: path.relative(appSourceRoot, file).replaceAll("\\", "/"),
      source: readFileSync(file, "utf8"),
    }));
    const fullAppReachability = rustModuleReachability(
      fullAppInventory,
      ["lib.rs", "main.rs"],
    );
    const fullAppExclusions = new Set([
      ...fullAppReachability.cfgOnly.map(({ relative }) => relative),
      ...frozenUnreachableStagingTestFiles,
    ]);
    const crateInventory = analysisExtracted
      ? rustFiles(crateSourceRoot).map((file) => ({
        relative: `analysis-crate/${path.relative(crateSourceRoot, file).replaceAll("\\", "/")}`,
        source: readFileSync(file, "utf8"),
      }))
      : [];
    const crateReachability = crateInventory.length > 0
      ? rustModuleReachability(crateInventory, ["analysis-crate/lib.rs"])
      : { production: [], cfgOnly: [] };
    const crateClassified = new Set([
      ...crateReachability.production.map(({ relative }) => relative),
      ...crateReachability.cfgOnly.map(({ relative }) => relative),
    ]);
    expect(
      crateInventory.map(({ relative }) => relative).filter((relative) => !crateClassified.has(relative)),
      "analysis crate Rust files outside parsed production/cfg reachability",
    ).toEqual([]);
    const crateCfgOnly = new Set(crateReachability.cfgOnly.map(({ relative }) => relative));
    const fullProductionInventory = [
      ...fullAppInventory.filter(({ relative }) => !fullAppExclusions.has(relative)),
      ...crateInventory.filter(({ relative }) => !crateCfgOnly.has(relative)),
    ];
    expect(fullProductionInventory.map(({ relative }) => relative)).toContain("projects/read_model.rs");
    for (const { relative, source } of fullProductionInventory) {
      expect(
        negativeForeignReadOperations(productionRust(source), foreignSchemaTables),
        `${relative} structurally reads foreign tables with a negative identifier`,
      ).toEqual([]);
    }
  }, 20_000);
});
