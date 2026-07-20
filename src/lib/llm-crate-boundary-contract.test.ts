import { existsSync, readFileSync, readdirSync } from "node:fs";
import path from "node:path";
import { describe, expect, it } from "vitest";

const repoRoot = path.resolve(import.meta.dirname, "..", "..");
const read = (relativePath: string) => {
  const absolute = path.join(repoRoot, relativePath);
  return existsSync(absolute)
    ? readFileSync(absolute, "utf8").replace(/\r\n/g, "\n")
    : "";
};
const rustSources = (relativeDir: string): string[] => {
  const absolute = path.join(repoRoot, relativeDir);
  if (!existsSync(absolute)) return [];
  return readdirSync(absolute, { withFileTypes: true }).flatMap((entry) => {
    const child = path.join(relativeDir, entry.name).replaceAll("\\", "/");
    return entry.isDirectory()
      ? rustSources(child)
      : entry.isFile() && entry.name.endsWith(".rs")
        ? [read(child)]
        : [];
  });
};
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
const lockPackage = (source: string, name: string) =>
  source
    .split(/(?=^\[\[package\]\]$)/m)
    .find((block) => block.includes(`\nname = "${name}"\n`)) ?? "";
const lockDependencies = (block: string) => {
  const match = block.match(/^dependencies = \[\n([\s\S]*?)^\]$/m);
  return match
    ? [...match[1].matchAll(/^ "([^"]+)",?$/gm)]
        .map((entry) => entry[1])
        .sort()
    : [];
};
const rustTestPattern =
  /#\[(?:tokio::)?test(?:\([^\]]*\))?\]\s*(?:#\[[^\]]+\]\s*)*(?:async\s+)?fn\s+([A-Za-z0-9_]+)/g;
const testNames = (sources: string[]) =>
  sources.flatMap((source) =>
    [...source.matchAll(new RegExp(rustTestPattern.source, "g"))].map(
      (match) => match[1],
    ),
  );
const rustBlock = (source: string, marker: string) => {
  const start = source.indexOf(marker);
  if (start < 0) return "";
  const brace = source.indexOf("{", start);
  if (brace < 0) return "";
  let depth = 0;
  for (let index = brace; index < source.length; index += 1) {
    if (source[index] === "{") depth += 1;
    if (source[index] === "}") depth -= 1;
    if (depth === 0) return source.slice(start, index + 1);
  }
  return "";
};
const publicMethods = (source: string, typeName: string) =>
  [...rustBlock(source, `impl ${typeName}`).matchAll(/\bpub\s+(?:async\s+)?fn\s+(\w+)/g)]
    .map((match) => match[1])
    .sort();

const crateDir = "src-tauri/crates/extractum-llm";
const appDir = "src-tauri/src/llm";
const expectedCrateDependencies = [
  "extractum-core",
  "reqwest",
  "secrecy",
  "serde",
  "serde_json",
  "tokio",
  "tokio-util",
].sort();
const forbiddenCrateDependencyNames = [
  "apalis", "apalis-sqlite", "extractum", "extractum-analysis",
  "extractum-gemini-browser", "extractum-prompt-packs", "extractum-telegram",
  "grammers-client", "grammers-mtsender", "grammers-session", "grammers-tl-types",
  "keyring", "sqlx", "tauri", "windows-sys",
];

const crateOwnedBaselineTests = [
  "gemini_request_mapping_keeps_system_history_and_roles",
  "gemini_request_mapping_keeps_existing_messages_without_output_limit",
  "gemini_stream_chunk_text_and_usage_are_parsed",
  "gemini_model_mapping_uses_short_model_id",
  "gemini_request_rejects_unsupported_roles_with_typed_validation_error",
  "gemini_model_listing_requires_typed_auth_error",
  "gemini_server_error_message_includes_transient_recovery_hint",
  "openai_compat_request_keeps_standard_roles",
  "openai_compat_stream_chunk_mapping_reads_delta_and_usage",
  "openai_compat_model_mapping_uses_model_id",
  "openai_compat_model_mapping_reads_omniroute_limits_and_capabilities",
  "openai_compat_request_rejects_unsupported_roles_with_typed_validation_error",
  "openai_compat_retry_status_policy_is_bounded_to_transient_failures",
  "openai_compat_stream_retries_transient_http_before_streaming",
  "openai_compat_model_listing_requires_typed_auth_error",
  "validate_request_returns_typed_validation_error",
  "resolve_effective_model_returns_typed_validation_error",
  "run_llm_collect_returns_typed_validation_error",
  "requests_with_different_profiles_run_without_blocking_each_other",
  "interactive_requests_jump_ahead_of_background_queue",
  "queued_requests_can_be_cancelled_before_start",
  "cancelling_owned_run_requests_aborts_running_work",
  "request_snapshots_report_running_and_queued_requests",
  "active_owner_run_ids_reports_running_and_queued_owned_requests",
  "queue_positions_are_recomputed_after_cancelling_a_queued_request",
  "failed_requests_release_capacity_for_next_queued_request",
  "failed_requests_preserve_typed_error_kind",
  "sse_data_is_parsed_from_stream_chunks",
  "sse_data_decode_failures_are_typed_internal_errors",
  "provider_parse_returns_typed_validation_error",
  "provider_parse_accepts_openai_compatible_aliases",
  "model_input_token_limit_lookup_matches_provider_model_ids_and_names",
  "model_output_token_limit_lookup_matches_provider_model_ids_and_names",
  "normalize_base_url_returns_typed_validation_error",
  "normalize_base_url_allows_https_and_loopback_http_only",
  "llm_request_diagnostic_keys_are_stable_snake_case",
] as const;
const appOwnedBaselineTests = [
  "profile_settings_roundtrip_stores_api_key_in_secret_store",
  "active_profile_resolution_loads_key_from_secret_store",
  "legacy_remote_http_profile_is_rejected_before_request_configuration",
  "changing_key_scope_without_replacement_is_rejected",
  "keyed_legacy_profile_materializes_effective_base_url_while_unkeyed_stays_blank",
  "credential_scope_uses_provider_origin_and_effective_port_but_not_path",
  "materialization_write_failure_fails_closed_during_state_load",
  "profile_state_lists_multiple_saved_profiles",
  "validate_profile_id_rejects_invalid_characters",
  "set_active_profile_returns_typed_not_found_error",
  "empty_save_preserves_existing_secret",
  "clear_profile_api_key_deletes_secret",
  "delete_profile_removes_settings_and_secret_and_resets_active",
  "delete_profile_fails_if_secret_store_fails_leaving_db_settings_intact",
  "provider_diagnostics_exclude_profile_ids_and_base_urls",
] as const;

const curatedRoot = `mod gemini;
mod openai_compat;
mod provider;
mod runner;
mod scheduler;
mod streaming;
mod types;

pub use provider::{
    list_provider_models, normalize_base_url, resolve_model_input_token_limit,
    resolve_model_output_token_limit, ProviderKind,
};
pub use runner::{
    resolve_effective_model, run_llm_collect_with_profile, run_llm_stream_with_profile,
    validate_request,
};
pub use scheduler::{
    llm_request_kind_diagnostic_key, llm_request_state_diagnostic_key, LlmRequestControl,
    LlmRequestError, LlmRequestKind, LlmRequestMetadata, LlmRequestPriority,
    LlmRequestSnapshot, LlmRequestSnapshotState, LlmSchedulerState,
};
pub use types::{
    LlmChatRequest, LlmCompletion, LlmMessage, LlmProviderAccess, LlmProviderModel,
    LlmUsage, ResolvedLlmProfile,
};
`;

const commandNames = [
  "get_llm_profiles", "get_llm_request_snapshots", "save_llm_profile",
  "clear_llm_profile_api_key", "delete_llm_profile", "set_active_llm_profile",
  "list_llm_provider_models", "ask_llm_stream", "cancel_llm_request",
] as const;
const movedFiles = [
  "gemini.rs", "openai_compat.rs", "provider.rs", "runner.rs",
  "scheduler.rs", "streaming.rs", "types.rs",
];

const rootCargo = read("src-tauri/Cargo.toml");
const crateCargo = read(`${crateDir}/Cargo.toml`);
const cargoLock = read("src-tauri/Cargo.lock");
const crateLib = read(`${crateDir}/src/lib.rs`);
const crateRust = rustSources(`${crateDir}/src`);
const appRust = rustSources(appDir);
const crateSource = crateRust.join("\n");
const appSource = appRust.join("\n");
const appMod = read(`${appDir}/mod.rs`);
const profiles = read(`${appDir}/profiles.rs`);
const appLib = read("src-tauri/src/lib.rs");
const frontendTypes = read("src/lib/types/llm.ts");
const frontendApi = read("src/lib/api/llm.ts");

describe("extractum-llm crate boundary", () => {
  it("owns the exact workspace, manifest, lock, and feature dependency surface", () => {
    expect(tomlSection(rootCargo, "workspace")).toContain(
      'members = [".", "crates/extractum-core", "crates/extractum-gemini-browser", "crates/extractum-llm"]',
    );
    expect((rootCargo.match(/extractum-llm\s*=\s*\{ path = "crates\/extractum-llm" \}/g) ?? []).length).toBe(1);
    expect(dependencyNames(tomlSection(crateCargo, "dependencies"))).toEqual(expectedCrateDependencies);
    expect(dependencyNames(tomlSection(crateCargo, "dev-dependencies"))).toEqual(["tokio"]);
    for (const name of forbiddenCrateDependencyNames) {
      expect([...dependencyNames(tomlSection(crateCargo, "dependencies")), ...dependencyNames(tomlSection(crateCargo, "dev-dependencies"))]).not.toContain(name);
    }
    expect(tomlSection(rootCargo, "workspace.dependencies")).toContain('reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls", "stream"] }');
    expect(tomlSection(rootCargo, "workspace.dependencies")).toContain('secrecy = "0.8"');
    expect(tomlSection(rootCargo, "dependencies")).toContain("reqwest = { workspace = true }");
    expect(tomlSection(rootCargo, "dependencies")).toContain("secrecy = { workspace = true }");
    expect(tomlSection(crateCargo, "dependencies")).toContain("reqwest.workspace = true");
    expect(tomlSection(crateCargo, "dependencies")).toContain("secrecy.workspace = true");
    expect(tomlSection(crateCargo, "dependencies")).toContain('tokio = { workspace = true, features = ["macros", "sync", "time"] }');
    expect(tomlSection(crateCargo, "dev-dependencies")).toContain('tokio = { workspace = true, features = ["io-util", "net", "rt", "test-util"] }');
    expect(lockDependencies(lockPackage(cargoLock, "extractum-llm"))).toEqual(expectedCrateDependencies);
    expect(lockDependencies(lockPackage(cargoLock, "extractum")).filter((name) => name === "extractum-llm")).toEqual(["extractum-llm"]);
  });

  it("exposes only the curated API and keeps credentials non-serializable", () => {
    expect(crateLib).toBe(curatedRoot);
    expect(crateLib).not.toMatch(/pub mod|pub use .*\*|#\[cfg\(test\)\]|test_helper/);
    const types = read(`${crateDir}/src/types.rs`);
    for (const typeName of ["LlmProviderAccess", "ResolvedLlmProfile"]) {
      expect(types).toMatch(new RegExp(`#\\[derive\\(Clone\\)\\]\\npub struct ${typeName}`));
      expect(types).not.toMatch(new RegExp(`derive\\([^)]*(?:Serialize|Deserialize|Debug)[^)]*\\)\\npub struct ${typeName}`));
      expect(types).not.toMatch(new RegExp(`impl (?:serde::)?(?:Serialize|Deserialize) for ${typeName}`));
    }
    expect(types).not.toMatch(/pub(?:\(crate\))?\s+(?:fn\s+api_key|api_key\s*:)|pub\s+fn\s+\w+\([^)]*\)\s*->\s*&?SecretString/);
    expect(crateLib).not.toContain("ExposeSecret");
    expect(publicMethods(types, "LlmProviderAccess")).toEqual(["new"]);
    expect(publicMethods(types, "ResolvedLlmProfile")).toEqual(["base_url", "default_model", "new", "profile_id", "provider"]);
    expect(publicMethods(read(`${crateDir}/src/scheduler.rs`), "LlmRequestControl")).toEqual(["run_cancellable"]);
    expect(publicMethods(read(`${crateDir}/src/provider.rs`), "ProviderKind")).toEqual(["as_str", "parse"]);
    expect(publicMethods(read(`${crateDir}/src/scheduler.rs`), "LlmSchedulerState")).toEqual([
      "active_owner_run_ids", "cancel_request", "cancel_run_requests", "new",
      "request_snapshots", "run_request",
    ]);
    expect(crateLib).not.toMatch(/display_name|list_provider_models_without_timeout|list_gemini_models|list_openai_compat_models|parse_sse_data|find_event_boundary/);
  });

  it("has one physical owner and the frozen 36/15 test disposition", () => {
    const forbiddenSourcePatterns = [
      /\btauri(?:_[a-z0-9_]+)?\b/, /\bsqlx\b/, /\bkeyring\b/, /\bapalis(?:_sqlite)?\b/,
      /\bgrammers(?:_[a-z_]+)?\b/, /\bwindows_sys\b/,
      /\b(?:Child|Command|Stdio|ProcessTreeGuard)\b/, /(?:std|tokio)::process/,
      /\bprocess_tree\b/, /\bextractum_process\b/,
      /crate::(?:db|secret_store|diagnostics|analysis|prompt_packs|telegram|gemini_browser)/,
      /extractum_(?:analysis|prompt_packs|telegram|gemini_browser)/,
    ];
    for (const pattern of forbiddenSourcePatterns) expect(crateSource).not.toMatch(pattern);
    for (const file of movedFiles) {
      expect(existsSync(path.join(repoRoot, appDir, file))).toBe(false);
      expect(existsSync(path.join(repoRoot, crateDir, "src", file))).toBe(true);
      expect(appMod).not.toMatch(new RegExp(`(?:mod|use)\\s+${file.replace(".rs", "")}`));
    }
    for (const file of ["app_types.rs", "profiles.rs", "mod.rs"]) expect(existsSync(path.join(repoRoot, appDir, file))).toBe(true);
    expect(`${crateSource}\n${appSource}`).not.toContain("#[cfg(any())]");
    expect(appSource).not.toMatch(/struct (?:SchedulerKey|RequestEntry|OpenAiCompatChatChunk)|fn (?:parse_sse_data|find_event_boundary)/);
    const crateTests = testNames(crateRust);
    const appTests = testNames(appRust);
    for (const name of crateOwnedBaselineTests) {
      expect(crateTests.filter((value) => value === name)).toHaveLength(1);
      expect(appTests).not.toContain(name);
    }
    for (const name of appOwnedBaselineTests) {
      expect(appTests.filter((value) => value === name)).toHaveLength(1);
      expect(crateTests).not.toContain(name);
    }
    expect(new Set([...crateOwnedBaselineTests, ...appOwnedBaselineTests]).size).toBe(51);
  });

  it("keeps commands, events, profiles, diagnostics, and frontend payloads app-owned", () => {
    const commandMatches = [...appMod.matchAll(/#\[tauri::command\][\s\S]*?pub async fn (\w+)/g)].map((match) => match[1]);
    expect(commandMatches.sort()).toEqual([...commandNames].sort());
    const llmImport = rustBlock(appLib, "use llm::");
    const handlerStart = appLib.indexOf("tauri::generate_handler![");
    const handler = handlerStart < 0
      ? ""
      : appLib.slice(handlerStart, appLib.indexOf("])" , handlerStart) + 2);
    for (const name of commandNames) {
      expect((llmImport.match(new RegExp(`\\b${name}\\b`, "g")) ?? []).length).toBe(1);
      expect((handler.match(new RegExp(`\\b${name}\\b`, "g")) ?? []).length).toBe(1);
    }
    expect(appMod).toContain('const LLM_RESPONSE_EVENT: &str = "llm://response";');
    expect(frontendApi).toContain('export const LLM_RESPONSE_EVENT = "llm://response";');
    expect(frontendTypes).toMatch(/kind:\s*"queued"\s*\|\s*"started"\s*\|\s*"delta"\s*\|\s*"completed"\s*\|\s*"failed"\s*\|\s*"cancelled"/);
    expect(profiles).toMatch(/llm\.active_provider_profile/);
    expect(profiles).toMatch(/llm\.profile\.\{profile_id\}\.provider/);
    expect(profiles).toMatch(/llm\.profile\.\{profile_id\}\.default_model/);
    expect(profiles).toMatch(/llm\.profile\.\{profile_id\}\.base_url/);
    expect(profiles).toContain("llm_profile_api_key_secret");
    expect(read("src-tauri/src/secret_store.rs")).toContain("fn llm_profile_api_key_secret");
    expect(crateSource).not.toMatch(/app_settings|SecretStoreState|llm_profile_api_key_secret|StreamEvent|LlmProviderDiagnostic/);
    expect(appSource).toMatch(/sqlx|SecretStoreState|struct StreamEvent|LlmProviderDiagnostic/);
    expect(frontendApi).toContain('invoke<LlmProfilesState>("get_llm_profiles")');
    expect(frontendApi).toContain('invoke<LlmProfilesState>("save_llm_profile", { ...input })');
    expect(frontendApi).toContain('invoke<LlmProviderModel[]>("list_llm_provider_models", { ...input })');
    expect(frontendApi).toContain('invoke<void>("ask_llm_stream", { ...input })');
    for (const wrapper of ["clear_llm_profile_api_key", "delete_llm_profile", "set_active_llm_profile"]) expect(frontendApi).toContain(`invoke<LlmProfilesState>("${wrapper}", { profileId })`);
    expect(frontendApi).toContain('invoke<void>("cancel_llm_request", { requestId })');
    const askBody = rustBlock(appMod, "pub async fn ask_llm_stream");
    expect(askBody.indexOf("tokio::spawn")).toBeGreaterThanOrEqual(0);
    expect(askBody.lastIndexOf("Ok(())")).toBeGreaterThan(askBody.indexOf("tokio::spawn"));
    expect(askBody).toMatch(/run_request[\s\S]*"queued"[\s\S]*"started"[\s\S]*run_cancellable[\s\S]*run_llm_stream_with_profile[\s\S]*"delta"/);
    expect(askBody).toMatch(/Ok\(completion\)[\s\S]*LlmRequestError::Failed[\s\S]*LlmRequestError::Cancelled/);
    expect(askBody).not.toMatch(/tokio::spawn\([\s\S]*?\)\.await/);
  });

  it("pins provider retries and timeout policy in the final owner", () => {
    const runner = read(`${crateDir}/src/runner.rs`);
    const provider = read(`${crateDir}/src/provider.rs`);
    const gemini = read(`${crateDir}/src/gemini.rs`);
    const openai = read(`${crateDir}/src/openai_compat.rs`);
    expect(runner).toContain("const LLM_STREAM_TIMEOUT_SECS: u64 = 90;");
    expect((runner.match(/LLM request timed out after \{LLM_STREAM_TIMEOUT_SECS\} seconds/g) ?? []).length).toBe(2);
    expect(provider).toContain("const GEMINI_MODELS_TIMEOUT_SECS: u64 = 30;");
    expect(provider).toContain("const OPENAI_COMPAT_MODELS_TIMEOUT_SECS: u64 = 30;");
    expect(provider).toContain("const MODEL_LIMIT_LOOKUP_TIMEOUT_SECS: u64 = 5;");
    expect(gemini).toContain("const GEMINI_STREAM_MAX_ATTEMPTS: usize = 3;");
    expect(gemini).toContain("const GEMINI_RETRY_DELAY_MS: u64 = 600;");
    expect(openai).toContain("const OPENAI_COMPAT_STREAM_MAX_ATTEMPTS: usize = 3;");
    expect(openai).toContain("const OPENAI_COMPAT_RETRY_DELAY_MS: u64 = 600;");
  });
});
