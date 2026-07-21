import { describe, expect, it } from "vitest";

import { readPromptPackDomainSource } from "./prompt-pack-contract-paths";

const promptPacksModuleSource = readPromptPackDomainSource("lib.rs", "mod.rs");
const runtimeSource = readPromptPackDomainSource("runtime.rs");
const stageExecutionSource = readPromptPackDomainSource("stage_execution.rs");

const normalized = (source: string) => source.replace(/\r\n/g, "\n");
const matches = (source: string, pattern: RegExp) => source.match(pattern) ?? [];

const stageFunctions = [
  "run_transcript_analysis_stage_request",
  "run_synthesis_stage_request",
  "run_json_repair_stage_request",
  "run_gem_analysis_part_stage_request",
  "run_gem_analysis_part_repair_request",
] as const;

const gemHelpers = ["gem_part_phase", "gem_part_started_message"] as const;

describe("Prompt Pack stage execution ownership", () => {
  it("registers a private stage_execution sibling module", () => {
    const source = normalized(promptPacksModuleSource);

    expect(source).toMatch(/^mod stage_execution;$/m);
    expect(source).not.toMatch(/pub(?:\([^)]*\))?\s+mod stage_execution;/);
  });

  it.each(stageFunctions)("moves %s out of runtime with sibling visibility", (name) => {
    const stageExecution = normalized(stageExecutionSource);
    const runtime = normalized(runtimeSource);
    const definition = new RegExp(`^pub\\(super\\) async fn ${name}\\s*\\(`, "m");
    const runtimeDefinition = new RegExp(`^(?:pub\\(super\\) )?async fn ${name}\\s*\\(`, "m");

    expect(stageExecution).toMatch(definition);
    expect(runtime).not.toMatch(runtimeDefinition);
    expect(matches(runtime, new RegExp(`\\b${name}\\s*\\(`, "g"))).toHaveLength(1);
  });

  it.each(gemHelpers)("keeps %s private beside its consumers", (name) => {
    const stageExecution = normalized(stageExecutionSource);
    const runtime = normalized(runtimeSource);

    expect(stageExecution).toMatch(new RegExp(`^fn ${name}\\s*\\(`, "m"));
    expect(stageExecution).not.toMatch(
      new RegExp(`^pub(?:\\([^)]*\\))?\\s+fn ${name}\\s*\\(`, "m"),
    );
    expect(runtime).not.toMatch(new RegExp(`^fn ${name}\\s*\\(`, "m"));
  });

  it("owns exactly five policy-to-transport bridges", () => {
    const stageExecution = normalized(stageExecutionSource);
    const runtime = normalized(runtimeSource);

    expect(
      matches(stageExecution, /completion_runtime\.model_context\(\)\.await\?/g),
    ).toHaveLength(5);
    expect(
      matches(stageExecution, /completion_runtime\s*\.execute\s*\(/g),
    ).toHaveLength(5);
    expect(runtime).not.toContain("completion_runtime.model_context().await?");
    expect(runtime).not.toMatch(/completion_runtime\s*\.execute\s*\(/);
  });

  it("keeps dispatch and lifecycle responsibilities in runtime", () => {
    const stageExecution = normalized(stageExecutionSource);
    const runtime = normalized(runtimeSource);

    expect(
      matches(runtime, /YoutubeSummaryStageExecutionRequest::/g),
    ).toHaveLength(5);
    const forbidden = [
      "YoutubeSummaryStageExecutionRequest::",
      "#[tauri::command]",
      "load_run_runtime_config",
      "preflight_youtube_summary_run",
      "browser_runtime_start_failures_for_request",
      "emit_youtube_summary_terminal_event",
      "cleanup_interrupted_prompt_pack_runs",
      "seed_prompt_pack_cancellation_smoke_fixture",
      "clear_prompt_pack_cancellation_smoke_fixture",
      "super::runtime",
    ];

    for (const marker of forbidden) {
      expect(stageExecution).not.toContain(marker);
    }
  });
});
