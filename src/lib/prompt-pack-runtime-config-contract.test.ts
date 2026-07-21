import { describe, expect, it } from "vitest";

import {
  promptPackCrateExtracted,
  readPromptPackDomainSource,
} from "./prompt-pack-contract-paths";

const promptPacksModuleSource = readPromptPackDomainSource("lib.rs", "mod.rs");
const runtimeSource = readPromptPackDomainSource("runtime.rs");
const runtimeCommandsSource = readPromptPackDomainSource("runtime_commands.rs");
const runtimeConfigSource = readPromptPackDomainSource("runtime_config.rs");

const normalized = (source: string) => source.replace(/\r\n/g, "\n");
const matches = (source: string, pattern: RegExp) => source.match(pattern) ?? [];
const productionPart = (source: string) =>
  normalized(source).split("\n#[cfg(test)]\nmod tests")[0];

describe("Prompt Pack runtime config ownership", () => {
  it("registers a private runtime_config sibling module", () => {
    const source = normalized(promptPacksModuleSource);

    expect(source).toMatch(/^mod runtime_config;$/m);
    expect(source).not.toMatch(/pub(?:\([^)]*\))?\s+mod runtime_config;/);
  });

  it("moves provider parsing and loaded config out of runtime", () => {
    const runtime = productionPart(runtimeSource);
    const runtimeConfig = normalized(runtimeConfigSource);

    expect(runtimeConfig).toMatch(/^pub\(super\) enum RunRuntimeProvider\s*\{/m);
    expect(runtimeConfig).toMatch(/^pub\(super\) struct RunRuntimeConfig\s*\{/m);
    expect(runtimeConfig).toMatch(
      /^pub\(super\) async fn load_run_runtime_config\s*\(/m,
    );
    expect(runtimeConfig).toMatch(/^    fn parse\(value: &str\)/m);
    expect(runtimeConfig).not.toMatch(/^    pub(?:\([^)]*\))?\s+fn parse\(/m);
    for (const field of [
      "runtime_provider",
      "profile_id",
      "model_override",
      "browser_provider_config",
    ]) {
      expect(runtimeConfig).toMatch(
        new RegExp(`^\\s+pub\\(super\\) ${field}:`, "m"),
      );
    }

    expect(runtime).not.toMatch(/^enum RunRuntimeProvider\s*\{/m);
    expect(runtime).not.toMatch(/^struct RunRuntimeConfig\s*\{/m);
    expect(runtime).not.toMatch(/^async fn load_run_runtime_config\s*\(/m);
  });

  it("owns the persisted runtime-config query and decoding errors", () => {
    const runtime = productionPart(runtimeSource);
    const runtimeConfig = normalized(runtimeConfigSource);
    const selectMarker =
      "SELECT provider_profile_id, model, runtime_provider, browser_provider_config_json";

    expect(runtimeConfig).toContain(selectMarker);
    expect(runtimeConfig).toContain("serde_json::from_str");
    expect(runtimeConfig).toContain(
      "Unsupported prompt-pack runtime provider: {other}",
    );
    expect(runtimeConfig).toContain(
      "parse Browser Provider config snapshot: {error}",
    );
    expect(runtime).not.toContain(selectMarker);
  });

  it("keeps model resolution domain-side and profile resolution in the app task", () => {
    const runtime = productionPart(runtimeSource);
    const runtimeCommands = normalized(runtimeCommandsSource);

    expect(matches(runtime, /\bload_run_runtime_config\s*\(/g)).toHaveLength(1);
    expect(runtime).toContain("RunRuntimeProvider::Api");
    expect(runtime).toContain("RunRuntimeProvider::GeminiBrowser");
    expect(runtime).not.toContain("resolve_profile_for_backend");
    expect(runtime).toContain("resolve_effective_model");
    const inputTokenLimitResolver = promptPackCrateExtracted
      ? "resolve_model_input_token_limit"
      : "resolve_model_input_token_limit_for_backend";
    const legacyInputTokenLimitResolver = promptPackCrateExtracted
      ? "resolve_model_input_token_limit_for_backend"
      : "resolve_model_input_token_limit";
    expect(runtime).toMatch(new RegExp(`\\b${inputTokenLimitResolver}\\b`));
    expect(runtime).not.toMatch(
      new RegExp(`\\b${legacyInputTokenLimitResolver}\\b`),
    );
    expect(runtime).toContain("RunCompletionRuntime::Api");
    expect(runtime).toContain("RunCompletionRuntime::GeminiBrowser");
    expect(runtimeCommands).toContain("prepare_run_execution");
    expect(runtimeCommands).toContain("resolve_profile_for_backend");
    const executionTaskStart = runtimeCommands.indexOf(
      "fn build_youtube_summary_execution_task",
    );
    expect(executionTaskStart).toBeGreaterThanOrEqual(0);
    const executionTask = runtimeCommands.slice(executionTaskStart);
    expect(executionTask.indexOf("resolve_profile_for_backend")).toBeGreaterThan(
      executionTask.indexOf("prepare_run_execution"),
    );
  });

  it("keeps orchestration dependencies out of runtime_config", () => {
    const runtimeConfig = normalized(runtimeConfigSource);
    const forbidden = [
      "tauri::",
      "AppHandle",
      "#[tauri::command]",
      "RunCompletionRuntime",
      "resolve_profile_for_backend",
      "resolve_effective_model",
      "resolve_model_input_token_limit",
      "resolve_model_input_token_limit_for_backend",
      "YoutubeSummaryStageExecutionRequest",
      "PromptPackRunState",
      "super::runtime",
    ];

    for (const marker of forbidden) {
      expect(runtimeConfig).not.toContain(marker);
    }
  });
});
