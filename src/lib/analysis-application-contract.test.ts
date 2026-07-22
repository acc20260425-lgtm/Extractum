import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";

import {
  readAnalysisContractSource,
  readAppAnalysisSource,
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
  cancel_analysis_run: ["handle: AppHandle, state: tauri::State<'_, AnalysisState>, scheduler: tauri::State<'_, LlmSchedulerState>, run_id: i64 -> AppResult<()>", ["runId"]],
  start_project_analysis: ["handle: AppHandle, state: tauri::State<'_, crate::analysis::AnalysisState>, project_id: i64, period_from: i64, period_to: i64, output_language: String, prompt_template_id: i64, model_override: Option<String>, profile_id: Option<String>, youtube_corpus_mode: Option<String>, include_migrated_history: bool -> AppResult<i64>", ["projectId", "periodFrom", "periodTo", "outputLanguage", "promptTemplateId", "modelOverride", "profileId", "youtubeCorpusMode", "includeMigratedHistory"]],
  list_project_runs: ["handle: AppHandle, project_id: i64 -> AppResult<Vec<crate::analysis::models::AnalysisRunSummary>>", ["projectId"]],
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
const checkpointCharacterizationLeaves = new Set([
  "run_reads_preserve_deleted_blank_and_snapshot_scope_labels",
  "analysis_run_search_escapes_percent_underscore_and_backslash_before_limit",
  "chat_legacy_label_fallback_rereads_run_on_the_foreign_label_snapshot",
  "chat_profile_resolution_failure_is_async_after_request_id",
  "chat_persistence_failure_keeps_completed_answer_failure_message",
  "terminal_cleanup_removes_active_state_when_terminal_persistence_fails",
  "report_start_preserves_acceptance_order_and_two_corpus_reads",
  "legacy_trace_bytes_decode_after_core_compression_handoff",
  "decode_trace_data_returns_typed_internal_for_invalid_json",
  "trace_ref_json_is_byte_compatible_for_telegram_and_youtube",
  "start_analysis_report_request_constructors_preserve_source_group_and_project_scopes",
  "analysis_run_list_filter_constructors_preserve_analysis_and_project_scopes",
  "resolved_analysis_scope_rejects_zero_or_multiple_identities",
  "resolved_analysis_scope_requires_nonempty_stable_sources_and_label",
]);

function sourceIdentityPrefix(file: string): string {
  const relative = path.relative(currentAnalysisRoot, file).replaceAll("\\", "/");
  if (relative === "tests_portable.rs") return "analysis::tests";
  const parts = relative.replace(/\.rs$/, "").split("/");
  if (parts.at(-1) === "mod") parts.pop();
  if (parts.includes("tests")) return ["analysis", ...parts].join("::");
  if (parts.length === 0) return "analysis::tests";
  return ["analysis", ...parts, "tests"].join("::");
}

function executableAnalysisIdentities(): string[] {
  return rustFiles(currentAnalysisRoot)
    .filter((file) => path.basename(file) !== "tests_application.rs")
    .flatMap((file) => {
      const source = readFileSync(file, "utf8");
      const prefix = sourceIdentityPrefix(file);
      return [
        ...source.matchAll(
          /^\s*#\[(?:tokio::)?test\]\s*(?:#\[[^\n]+\]\s*)*(?:async\s+)?fn\s+([A-Za-z0-9_]+)\s*\(/gm,
        ),
      ]
        .filter((match) => !checkpointCharacterizationLeaves.has(match[1]))
        .map((match) => `${prefix}::${match[1]}`);
    })
    .sort();
}

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

const analysisSources = rustFiles(currentAnalysisRoot)
  .map((file) => readFileSync(file, "utf8"))
  .join("\n");
const projectSources = rustFiles(path.join(repoRoot, "src-tauri/src/projects"))
  .map((file) => readFileSync(file, "utf8"))
  .join("\n");

describe("analysis application boundary", () => {
  it("uses a manifest-keyed fail-closed dual-owner source selector", () => {
    expect(readAppAnalysisSource("mod.rs")).toContain("mod chat;");
    expect(
      readAnalysisContractSource({
        before: "mod.rs",
        after: { owner: "app", path: "mod.rs" },
      }),
    ).toContain("mod chat;");
    expect(() => readAppAnalysisSource("../lib.rs")).toThrow(/escapes selected root/);
    expect(() => readAppAnalysisSource("missing.rs")).toThrow(/is missing/);
  });

  it("freezes the executable Appendix A partition at 95 crate and 48 app identities", () => {
    const frozen = parseAppendix();
    const current = executableAnalysisIdentities();
    const crate = frozen.filter(({ owner }) => owner === "crate").map(({ current }) => current);
    const app = frozen.filter(({ owner }) => owner === "app").map(({ current }) => current);

    expect(current).toHaveLength(143);
    expect(crate).toHaveLength(95);
    expect(app).toHaveLength(48);
    expect(crate.filter((identity) => app.includes(identity))).toEqual([]);
    expect([...crate, ...app].sort()).toEqual(current);
    expect(new Set(frozen.map(({ final }) => final)).size).toBe(143);
  });

  it("stages checkpoint two portable values and safe construction without app compression ownership", () => {
    const moduleSource = readAppAnalysisSource("mod.rs");
    const domainSource = readAppAnalysisSource("domain_portable.rs");
    const models = readAppAnalysisSource("models.rs");
    const report = readAppAnalysisSource("report.rs");
    const trace = readAppAnalysisSource("trace.rs");
    const filters = readAppAnalysisSource("store/read_model.rs");
    const reportCommands = readAppAnalysisSource("report_commands.rs");

    expect(moduleSource.match(/include!\("tests_portable\.rs"\)/g) ?? []).toHaveLength(1);
    expect(moduleSource).toContain('#[path = "domain_portable.rs"]');
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
      /pub\(crate\) struct StartAnalysisReportRequest \{([\s\S]*?)\n\}/,
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
    expect(paths.projectList).toContain("pub(crate) async fn list_research_projects_in_pool");
    expect(normalized(paths.projectList)).toContain(normalized(`
      SELECT ar.status FROM analysis_runs ar WHERE ar.project_id = p.id
      ORDER BY ar.created_at DESC, ar.id DESC LIMIT 1
    `));
    expect(normalized(paths.projectList)).toContain(normalized(`
      SELECT 1 FROM analysis_runs ar WHERE ar.project_id = p.id
      AND ar.status IN ('queued', 'running')
    `));

    expect(paths.projectDeletion).toContain("pub(crate) async fn delete_project_in_pool");
    expect(paths.projectDeletion).toContain('sqlx::query("DELETE FROM analysis_runs WHERE project_id = ?")');
    expectOrdered(paths.projectDeletion, [
      'sqlx::query("DELETE FROM analysis_runs WHERE project_id = ?")',
      'sqlx::query("DELETE FROM project_sources WHERE project_id = ?")',
      'sqlx::query("DELETE FROM projects WHERE id = ?")',
      "tx.commit().await",
    ]);

    expect(paths.accountDeletion).toContain("async fn run_ids_depending_on_sources");
    expect(paths.accountDeletion).toContain(
      '"SELECT id, source_id, source_group_id FROM analysis_runs ORDER BY id ASC"',
    );
    expect(paths.accountDeletion).toContain("async fn group_has_owned_source");
    expect(paths.accountDeletion).toContain(
      '"SELECT source_id FROM analysis_source_group_members WHERE group_id = ?"',
    );

    expect(paths.notebookLm).toContain("pub(crate) async fn load_export_source_group");
    expect(normalized(paths.notebookLm)).toContain(normalized(`
      FROM analysis_source_group_members members
      JOIN sources ON sources.id = members.source_id
      WHERE members.group_id = ?
      ORDER BY COALESCE(sources.title, ''), sources.id
    `));

    expect(paths.diagnostics).toContain("async fn load_analysis_run_counts");
    expect(normalized(paths.diagnostics)).toContain(normalized(`
      FROM analysis_runs
      GROUP BY provider, run_type, scope_type, status, snapshot_state, error_kind
      ORDER BY provider, run_type, scope_type, status, snapshot_state, error_kind
    `));
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

  it("analysis_wire_contract_serializes_commands_events_and_errors_unchanged", () => {
    const models = readAnalysisContractSource({
      before: "models.rs",
      after: { owner: "crate", path: "models.rs" },
    });
    const moduleSource = readAnalysisContractSource({
      before: "mod.rs",
      after: { owner: "app", path: "mod.rs" },
    });
    const chat = readAnalysisContractSource({
      before: "chat.rs",
      after: { owner: "crate", path: "chat.rs" },
    });
    const report = readAnalysisContractSource({
      before: "report.rs",
      after: { owner: "crate", path: "report.rs" },
    });
    const requests = readAnalysisContractSource({
      before: "report/requests.rs",
      after: { owner: "crate", path: "report/requests.rs" },
    });
    const phases = readAnalysisContractSource({
      before: "report/phases.rs",
      after: { owner: "crate", path: "report/phases.rs" },
    });
    const lifecycle = readAnalysisContractSource({
      before: "report/lifecycle.rs",
      after: { owner: "crate", path: "report/lifecycle.rs" },
    });
    const wireWitness = readAppAnalysisSource("tests_application.rs");

    const contextParams = new Set(["handle", "state", "scheduler", "repair_state"]);
    for (const [name, [expectedSignature, expectedWireParams]] of Object.entries(commandWireContracts)) {
      const source = (projectRelease as readonly string[]).includes(name)
        ? projectSources
        : analysisSources;
      const actual = commandSignature(source, name);
      expect(actual.signature, name).toBe(expectedSignature);
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
      'ChatEvent::new(completed_request_id, run_id, "completed")',
    ]);
    expect(chat).toContain('ChatEvent::new(failed_request_id, run_id, "failed")');
    expect(chat).toContain('ChatEvent::new(cancelled_request_id, run_id, "cancelled")');
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
    expect(lifecycle).toContain('RunEvent::new(run_id, "cancelled", "persist")');
    for (const terminal of [
      "Answer completed.", "Answer cancelled.", "Analysis run cancelled.",
      "Report run failed.", "Report run failed before snapshot capture completed.",
    ]) expect(`${analysisSources}\n${chat}\n${report}`).toContain(terminal);
    expect(errorSource).toContain("#[serde(rename_all = \"snake_case\")]");
    expect(errorSource).toMatch(/pub struct AppError\s*{\s*pub kind: AppErrorKind,\s*pub message: String,/s);
  });
});
