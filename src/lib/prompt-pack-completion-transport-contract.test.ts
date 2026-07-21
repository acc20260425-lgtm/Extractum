import { describe, expect, it } from "vitest";

import { readPromptPackDomainSource } from "./prompt-pack-contract-paths";

const completionTransportSource = readPromptPackDomainSource("completion_transport.rs");
const dtoSource = readPromptPackDomainSource("dto.rs");
const promptPacksModuleSource = readPromptPackDomainSource("lib.rs", "mod.rs");
const runtimeSource = readPromptPackDomainSource("runtime.rs");
const stageExecutionSource = readPromptPackDomainSource("stage_execution.rs");

const normalized = (source: string) => source.replace(/\r\n/g, "\n");
const matches = (source: string, pattern: RegExp) => source.match(pattern) ?? [];

describe("Prompt Pack completion transport ownership", () => {
  it("registers a private completion_transport sibling module", () => {
    const source = normalized(promptPacksModuleSource);

    expect(source).toMatch(/^mod completion_transport;$/m);
    expect(source).not.toMatch(/pub(?:\([^)]*\))?\s+mod completion_transport;/);
  });

  it("moves the provider transport interface out of runtime", () => {
    const transport = normalized(completionTransportSource);
    const runtime = normalized(runtimeSource);

    expect(transport).toMatch(/^pub\(super\) enum RunCompletionRuntime\s*\{/m);
    expect(transport).toMatch(/^pub\(super\) struct CompletionModelContext\s*\{/m);
    expect(transport).toMatch(/^pub\(super\) struct StageCompletionRequest\s*\{/m);
    expect(transport).toMatch(/pub\(super\) async fn model_context\s*\(/);
    expect(transport).toMatch(/pub\(super\) async fn execute\s*\(/);
    expect(transport).toMatch(/async fn run_api_llm_request\s*\(/);
    expect(transport).toMatch(/async fn run_browser_llm_request\s*\(/);
    const movedHelpers = [
      "llm_chat_request_to_browser_prompt",
      "browser_run_id_for_stage",
      "browser_run_source_for_stage",
      "browser_stage_completion_from_result",
      "run_browser_stage_result_with_cancellation",
      "persist_browser_stage_provenance",
      "non_empty_string",
    ];
    for (const helper of movedHelpers) {
      expect(transport).toMatch(new RegExp(`(?:async\\s+)?fn\\s+${helper}\\b`));
      expect(runtime).not.toMatch(new RegExp(`(?:async\\s+)?fn\\s+${helper}\\b`));
    }
    expect(transport).toContain("resolve_model_output_token_limit_for_backend");
    expect(runtime).not.toMatch(/^enum RunCompletionRuntime\s*\{/m);
    expect(runtime).not.toMatch(/async fn run_api_llm_request\s*\(/);
    expect(runtime).not.toMatch(/async fn run_browser_llm_request\s*\(/);
  });

  it("keeps all five stage bridges behind the transport interface", () => {
    const runtime = normalized(runtimeSource);
    const stageExecution = normalized(stageExecutionSource);

    expect(matches(runtime, /match\s+&?completion_runtime\b/g)).toHaveLength(0);
    expect(
      matches(stageExecution, /completion_runtime\.model_context\(\)\.await\?/g),
    ).toHaveLength(5);
    expect(
      matches(stageExecution, /completion_runtime\s*\.execute\s*\(/g),
    ).toHaveLength(5);
  });

  it("keeps one event constant definition and the runtime compatibility path", () => {
    const dto = normalized(dtoSource);
    const runtime = normalized(runtimeSource);
    const transport = normalized(completionTransportSource);
    const combined = [dto, runtime, transport].join("\n");

    expect(
      matches(combined, /pub const PROMPT_PACK_RUN_EVENT\s*:/g),
    ).toHaveLength(1);
    expect(dto).toContain(
      'pub const PROMPT_PACK_RUN_EVENT: &str = "prompt-pack-run-event";',
    );
    expect(runtime).toMatch(/^pub use super::dto::PROMPT_PACK_RUN_EVENT;$/m);
  });

  it("preserves direct transport events and the repair queue text", () => {
    const transport = normalized(completionTransportSource);

    expect(transport).toContain("JSON repair queued at position {position}");
    expect(
      matches(transport, /\.emit\(\s*PROMPT_PACK_RUN_EVENT,/g),
    ).toHaveLength(4);
    expect(transport).not.toContain("emit_prompt_pack_run_event");
    expect(transport).not.toContain("apply_event");
  });

  it("keeps orchestration and lifecycle responsibilities out of transport", () => {
    const transport = normalized(completionTransportSource);
    const forbidden = [
      "#[tauri::command]",
      "start_youtube_summary_run",
      "preflight_youtube_summary_run",
      "browser_runtime_start_blocking_failure",
      "cleanup_interrupted_prompt_pack_runs",
      "seed_prompt_pack_cancellation_smoke_fixture",
      "clear_prompt_pack_cancellation_smoke_fixture",
      "load_run_runtime_config",
    ];

    for (const marker of forbidden) {
      expect(transport).not.toContain(marker);
    }
  });
});
