import { describe, expect, it } from "vitest";

import { readPromptPackDomainSource } from "./prompt-pack-contract-paths";

const promptPacksModuleSource = readPromptPackDomainSource("lib.rs", "mod.rs");
const runControlSource = readPromptPackDomainSource("run_control.rs");
const runtimeSource = readPromptPackDomainSource("runtime.rs");

const normalized = (source: string) => source.replace(/\r\n/g, "\n");

describe("Prompt Pack run control ownership", () => {
  it("registers a private run_control sibling module", () => {
    const source = normalized(promptPacksModuleSource);

    expect(source).toMatch(/^mod run_control;$/m);
    expect(source).not.toMatch(/pub(?:\([^)]*\))?\s+mod run_control;/);
  });

  it("moves the state and cancellation helper out of runtime", () => {
    const control = normalized(runControlSource);
    const runtime = normalized(runtimeSource);

    expect(control).toMatch(/^pub struct PromptPackRunState\s*\{/m);
    expect(control).toMatch(
      /^pub\(super\) async fn run_with_prompt_pack_run_cancellation<Fut, T>\s*\(/m,
    );
    expect(runtime).not.toMatch(/^pub struct PromptPackRunState\s*\{/m);
    expect(runtime).not.toMatch(
      /^(?:pub(?:\([^)]*\))?\s+)?async fn run_with_prompt_pack_run_cancellation<Fut, T>\s*\(/m,
    );
  });

  it("preserves both public PromptPackRunState paths", () => {
    const moduleSource = normalized(promptPacksModuleSource);
    const runtime = normalized(runtimeSource);

    expect(runtime).toMatch(
      /^pub use super::run_control::PromptPackRunState;$/m,
    );
    expect(moduleSource).toMatch(
      /pub use runtime::\{[\s\S]*?\bPromptPackRunState,\s*\n\};/,
    );
  });

  it("keeps the exact terminal event cleanup set", () => {
    const control = normalized(runControlSource);

    expect(control).toMatch(
      /"completed"\s*\|\s*"partial"\s*\|\s*"failed"\s*\|\s*"cancelled"\s*\|\s*"interrupted"/,
    );
  });

  it("keeps run control independent from runtime infrastructure", () => {
    const control = normalized(runControlSource);

    expect(control).not.toMatch(/\btauri\b/);
    expect(control).not.toMatch(/\bsqlx\b/);
    expect(control).not.toMatch(/\bAppHandle\b/);
    expect(control).not.toMatch(/\bEmitter\b/);
    expect(control).not.toMatch(/\bget_pool\b/);
    expect(control).not.toMatch(/\brun_store\b/);
    expect(control).not.toMatch(/\bstage_request_policy\b/);
  });
});
