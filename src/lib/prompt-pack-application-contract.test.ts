import { readFileSync, readdirSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

import { readPromptPackDomainSource } from "./prompt-pack-contract-paths";

const repositoryRoot = resolve(import.meta.dirname, "../..");
const promptPackAppRoot = resolve(repositoryRoot, "src-tauri/src/prompt_packs");
const appLibSource = readFileSync(resolve(repositoryRoot, "src-tauri/src/lib.rs"), "utf8");
const appPromptPackSource = readdirSync(promptPackAppRoot, { withFileTypes: true })
  .filter((entry) => entry.isFile() && entry.name.endsWith(".rs"))
  .sort((left, right) => left.name.localeCompare(right.name))
  .map((entry) => readFileSync(resolve(promptPackAppRoot, entry.name), "utf8"))
  .join("\n");

const commandArguments = {
  get_prompt_pack_library: ["handle"],
  preflight_youtube_summary_run: [
    "handle", "project_id", "source_ids", "profile_id", "model_override", "runtime_provider",
    "browser_provider_config", "output_language", "control_preset", "evidence_mode",
    "include_comments",
  ],
  start_youtube_summary_run: [
    "handle", "state", "client_request_id", "project_id", "source_ids", "profile_id",
    "model_override", "runtime_provider", "browser_provider_config", "output_language",
    "control_preset", "evidence_mode", "include_comments",
  ],
  cancel_prompt_pack_run: ["handle", "state", "scheduler", "run_id"],
  update_prompt_pack_run: ["handle", "run_id", "run_label"],
  delete_prompt_pack_run: ["handle", "state", "run_id"],
  list_prompt_pack_runs: ["handle", "project_id", "limit"],
  list_active_prompt_pack_runs: ["handle", "state"],
  list_prompt_pack_run_stages: ["handle", "run_id"],
  get_prompt_pack_result: ["handle", "run_id"],
  list_prompt_pack_stage_artifacts: ["handle", "stage_run_id"],
  get_prompt_pack_stage_artifact: [
    "handle", "stage_run_id", "artifact_kind", "attempt_number", "artifact_index",
  ],
  get_prompt_pack_validation_findings: ["handle", "run_id"],
  list_prompt_pack_audit_events: ["handle", "run_id"],
} as const;

const devCommandArguments = {
  seed_prompt_pack_cancellation_smoke_fixture: ["handle", "state"],
  clear_prompt_pack_cancellation_smoke_fixture: ["handle", "state"],
} as const;

function matchingDelimiter(source: string, openIndex: number, open: string, close: string): number {
  let depth = 0;
  for (let index = openIndex; index < source.length; index += 1) {
    if (source[index] === open) depth += 1;
    if (source[index] === close) depth -= 1;
    if (depth === 0) return index;
  }
  throw new Error(`unclosed ${open}${close} delimiter`);
}

function functionParts(source: string, name: string) {
  const definition = new RegExp(
    `^\\s*(?:pub(?:\\([^)]*\\))?\\s+)?(?:async\\s+)?fn\\s+${name}\\s*\\(`,
    "gm",
  );
  const matches = [...source.matchAll(definition)];
  if (matches.length !== 1 || matches[0].index === undefined) {
    throw new Error(`expected exactly one Rust function ${name}, found ${matches.length}`);
  }
  const signatureOpen = source.indexOf("(", matches[0].index);
  const signatureClose = matchingDelimiter(source, signatureOpen, "(", ")");
  const bodyOpen = source.indexOf("{", signatureClose);
  const bodyClose = matchingDelimiter(source, bodyOpen, "{", "}");
  return {
    body: source.slice(bodyOpen + 1, bodyClose),
    parameters: source.slice(signatureOpen + 1, signatureClose),
  };
}

function splitTopLevelParameters(parameters: string): string[] {
  const result: string[] = [];
  let start = 0;
  let angleDepth = 0;
  let parenDepth = 0;
  let bracketDepth = 0;
  for (let index = 0; index < parameters.length; index += 1) {
    const character = parameters[index];
    if (character === "<") angleDepth += 1;
    else if (character === ">") angleDepth -= 1;
    else if (character === "(") parenDepth += 1;
    else if (character === ")") parenDepth -= 1;
    else if (character === "[") bracketDepth += 1;
    else if (character === "]") bracketDepth -= 1;
    else if (character === "," && angleDepth === 0 && parenDepth === 0 && bracketDepth === 0) {
      result.push(parameters.slice(start, index));
      start = index + 1;
    }
  }
  result.push(parameters.slice(start));
  return result.map((parameter) => parameter.trim()).filter(Boolean);
}

function parameterNames(source: string, name: string): string[] {
  return splitTopLevelParameters(functionParts(source, name).parameters).map((parameter) => {
    const identifier = parameter.match(/^(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*:/)?.[1];
    if (!identifier) throw new Error(`cannot parse ${name} parameter: ${parameter}`);
    return identifier;
  });
}

function handlerSource(source: string): string {
  const token = "tauri::generate_handler![";
  const tokenIndex = source.indexOf(token);
  if (tokenIndex < 0) throw new Error("missing tauri::generate_handler! registration");
  const openIndex = tokenIndex + token.length - 1;
  return source.slice(openIndex + 1, matchingDelimiter(source, openIndex, "[", "]"));
}

function orderedIndex(source: string, pattern: RegExp, after: number, label: string): number {
  const match = source.slice(after + 1).match(pattern);
  if (!match || match.index === undefined) throw new Error(`missing ordered marker: ${label}`);
  return after + 1 + match.index;
}

describe("Prompt Pack application boundary", () => {
  it("keeps all production and dev command attributes registrations and argument spellings", () => {
    const registrations = handlerSource(appLibSource);
    for (const [name, arguments_] of Object.entries({
      ...commandArguments,
      ...devCommandArguments,
    })) {
      expect(parameterNames(appPromptPackSource, name), name).toEqual(arguments_);
      expect(
        appPromptPackSource.match(new RegExp(
          `#\\[tauri::command\\]\\s*pub\\s+async\\s+fn\\s+${name}\\s*\\(`,
          "g",
        )) ?? [],
        name,
      ).toHaveLength(1);
      expect(registrations.match(new RegExp(`\\b${name}\\b`, "g")) ?? [], name).toHaveLength(1);
    }
  });

  it("keeps start idempotency readiness preflight queued-event spawn and profile-resolution order", () => {
    const runtime = readPromptPackDomainSource("runtime.rs");
    const hasPreparedService = /async\s+fn\s+start_youtube_summary_run_service\s*\(/.test(runtime);
    const startBody = hasPreparedService
      ? functionParts(runtime, "start_youtube_summary_run_service").body
      : functionParts(runtime, "start_youtube_summary_run").body;
    const commandBody = functionParts(appPromptPackSource, "start_youtube_summary_run").body;
    const flow = hasPreparedService ? `${startBody}\n${commandBody}` : startBody;

    let index = orderedIndex(
      flow,
      /client_request_id(?:\(\))?\.trim\(\)\.is_empty\(\)/,
      -1,
      "empty-ID guard",
    );
    index = orderedIndex(
      flow,
      /load_youtube_summary_run_by_client_request_id_in_pool\s*\(/,
      index,
      "first existing lookup",
    );
    index = orderedIndex(
      flow,
      /browser_runtime_start_failures_for_request\s*\(|\.read_status\s*\(/,
      index,
      "Browser readiness",
    );
    index = orderedIndex(
      flow,
      /load_youtube_summary_run_by_client_request_id_in_pool\s*\(|start_youtube_summary_run_with_preflight_failures_in_pool\s*\(|preflight_youtube_summary_run\s*\(/,
      index,
      "second lookup/preflight",
    );
    index = orderedIndex(flow, /track_if_absent\s*\(/, index, "track_if_absent");
    index = orderedIndex(
      flow,
      /state\.apply_event\s*\(|emit_prompt_pack_run_event\s*\(/,
      index,
      "queued state/event",
    );
    orderedIndex(flow, /spawn_youtube_summary_execution\s*\(/, index, "spawn");

    expect(commandBody).not.toContain("resolve_profile_for_backend");
    const spawnOwner = appPromptPackSource.includes("fn spawn_youtube_summary_execution(")
      ? appPromptPackSource
      : runtime;
    const spawnBody = functionParts(spawnOwner, "spawn_youtube_summary_execution").body;
    const spawnIndex = spawnBody.indexOf("tauri::async_runtime::spawn");
    expect(spawnIndex).toBeGreaterThanOrEqual(0);
    if (spawnBody.includes("resolve_profile_for_backend")) {
      expect(spawnBody.indexOf("resolve_profile_for_backend")).toBeGreaterThan(spawnIndex);
    } else {
      expect(spawnBody).toContain("execute_youtube_summary_run(");
      expect(functionParts(runtime, "execute_youtube_summary_run").body).toContain(
        "resolve_profile_for_backend",
      );
    }
  });

  it("keeps startup seed and interrupted-run cleanup wiring", () => {
    const seedIndex = appLibSource.indexOf("seed_builtin_prompt_packs(handle.clone())");
    const cleanupIndex = appLibSource.indexOf("cleanup_interrupted_prompt_pack_runs(handle.clone())");

    expect(seedIndex).toBeGreaterThanOrEqual(0);
    expect(cleanupIndex).toBeGreaterThan(seedIndex);
  });
});
