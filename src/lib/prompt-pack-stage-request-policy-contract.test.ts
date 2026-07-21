import { describe, expect, it } from "vitest";

import { readPromptPackDomainSource } from "./prompt-pack-contract-paths";

const assetsSource = readPromptPackDomainSource("assets.rs");
const promptPacksModuleSource = readPromptPackDomainSource("lib.rs", "mod.rs");
const runtimeSource = readPromptPackDomainSource("runtime.rs");
const stageRequestPolicySource = readPromptPackDomainSource("stage_request_policy.rs");

const normalized = (source: string) => source.replace(/\r\n/g, "\n");

const extractedFunctions = [
  "transcript_analysis_control_preset",
  "build_transcript_analysis_llm_request",
  "build_synthesis_llm_request",
  "gem_part_request_suffix",
  "gem_part_repair_request_suffix",
  "gem_analysis_part_max_output_tokens",
  "build_gem_analysis_part_llm_request",
  "build_gem_analysis_part_repair_llm_request",
  "build_json_repair_llm_request",
  "transcript_analysis_stage_max_output_token_budget",
  "transcript_analysis_stage_max_prompt_token_budget",
  "transcript_analysis_stage_max_output_token_budget_for_control_preset",
  "synthesis_stage_max_output_token_budget",
  "transcript_analysis_max_output_tokens",
  "gem_input_cap",
] as const;

const movedConstants = [
  "DETAILED_REPORT_CONTROL_PRESET",
  "STANDARD_VIDEO_SUMMARY_PROMPT",
  "DETAILED_VIDEO_SUMMARY_PROMPT",
] as const;

const centralizedAssetConstants = [
  "TRANSCRIPT_RUNTIME_JSON",
  "SYNTHESIS_RUNTIME_JSON",
] as const;

const movedStructs = [
  "StageRuntimeConfigAsset",
  "StageRuntimeConfiguration",
  "StageBudgetLimits",
] as const;

describe("Prompt Pack stage request policy ownership", () => {
  it("registers a private stage_request_policy sibling module", () => {
    const source = normalized(promptPacksModuleSource);

    expect(source).toMatch(/^mod stage_request_policy;$/m);
    expect(source).not.toMatch(/pub(?:\([^)]*\))?\s+mod stage_request_policy;/);
  });

  it.each(extractedFunctions)("moves %s out of runtime", (functionName) => {
    const policy = normalized(stageRequestPolicySource);
    const runtime = normalized(runtimeSource);
    const policyDefinition = new RegExp(
      `^pub\\(super\\)\\s+fn\\s+${functionName}\\s*\\(`,
      "m",
    );
    const runtimeDefinition = new RegExp(
      `^(?:pub\\(super\\)\\s+)?fn\\s+${functionName}\\s*\\(`,
      "m",
    );

    expect(policy).toMatch(policyDefinition);
    expect(runtime).not.toMatch(runtimeDefinition);
  });

  it("centralizes runtime assets while preserving prompt and budget policy", () => {
    const assets = normalized(assetsSource);
    const policy = normalized(stageRequestPolicySource);
    const runtime = normalized(runtimeSource);
    const productionRuntime = runtime.split("\n#[cfg(test)]", 1)[0];

    for (const constantName of movedConstants) {
      expect(policy).toMatch(new RegExp(`\\b${constantName}\\b`));
      expect(runtime).not.toMatch(new RegExp(`\\b(?:const|static)\\s+${constantName}\\b`));
    }
    for (const constantName of centralizedAssetConstants) {
      expect(assets).toMatch(new RegExp(`\\bconst\\s+${constantName}\\b`));
      expect(policy).toMatch(new RegExp(`\\b${constantName}\\b`));
      expect(policy).not.toMatch(new RegExp(`\\bconst\\s+${constantName}\\b`));
      expect(runtime).not.toMatch(new RegExp(`\\bconst\\s+${constantName}\\b`));
    }
    for (const structName of movedStructs) {
      expect(policy).toMatch(new RegExp(`^struct\\s+${structName}\\s*\\{`, "m"));
      expect(runtime).not.toMatch(new RegExp(`^struct\\s+${structName}\\s*\\{`, "m"));
    }
    expect(policy).toMatch(
      /^pub\(super\) const DETAILED_REPORT_CONTROL_PRESET: &str = "detailed_report";$/m,
    );
    expect(policy).not.toContain("include_str!");
    expect(productionRuntime).not.toContain("include_str!");
  });

  it("keeps execution lifecycle messages out of request policy", () => {
    const policy = normalized(stageRequestPolicySource);

    expect(policy).not.toMatch(/^fn gem_part_phase\s*\(/m);
    expect(policy).not.toMatch(/^fn gem_part_started_message\s*\(/m);
  });

  it("keeps the policy module independent from runtime infrastructure", () => {
    const policy = normalized(stageRequestPolicySource);

    expect(policy).not.toMatch(/\btauri\b/);
    expect(policy).not.toMatch(/\bsqlx\b/);
    expect(policy).not.toMatch(/\bCancellationToken\b/);
    expect(policy).not.toMatch(/\bsuper::runtime\b/);
    expect(policy).not.toMatch(/\bAppHandle\b/);
  });
});
