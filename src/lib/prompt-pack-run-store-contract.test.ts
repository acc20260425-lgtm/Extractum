import { existsSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

import { readPromptPackDomainSource } from "./prompt-pack-contract-paths";

const promptPacksModuleSource = readPromptPackDomainSource("lib.rs", "mod.rs");
const runStoreSource = readPromptPackDomainSource("run_store.rs");
const runtimeSource = readPromptPackDomainSource("runtime.rs");

const crateExtracted = existsSync(
  resolve(import.meta.dirname, "../../src-tauri/crates/extractum-prompt-packs/Cargo.toml"),
);

const normalized = (source: string) => source.replace(/\r\n/g, "\n");

const extractedFunctions = [
  ["list_prompt_pack_runs_in_pool", "list_prompt_pack_run_rows"],
  ["update_prompt_pack_run_in_pool", "update_prompt_pack_run_row"],
  ["delete_prompt_pack_run_in_pool", "delete_prompt_pack_run_row"],
  ["list_prompt_pack_run_stages_in_pool", "list_prompt_pack_run_stages_rows"],
  ["load_run_summary_optional", "load_run_summary_optional"],
] as const;

describe("Prompt Pack run store ownership", () => {
  it("registers a private run_store sibling module", () => {
    const source = normalized(promptPacksModuleSource);

    expect(source).toMatch(/^mod run_store;$/m);
    expect(source).not.toMatch(/pub(?:\([^)]*\))?\s+mod run_store;/);
  });

  it.each(extractedFunctions)(
    "keeps %s SQL in run_store behind its runtime service",
    (functionName, runtimeCall) => {
      const store = normalized(runStoreSource);
      const runtime = normalized(runtimeSource);
      const definition = new RegExp(
        `pub\\(super\\)\\s+async\\s+fn\\s+${functionName}\\s*\\(`,
      );

      expect(store).toMatch(definition);
      expect(runtime).toMatch(new RegExp(`\\b${runtimeCall}\\s*\\(`));
      const runtimeDefinition = new RegExp(
        `pub(?:\\(crate\\))?\\s+async\\s+fn\\s+${functionName}\\s*\\(`,
      );
      if (functionName === "load_run_summary_optional") {
        expect(runtime).not.toMatch(runtimeDefinition);
      } else {
        expect(runtime).toMatch(runtimeDefinition);
      }
    },
  );

  it("keeps row mapping private and avoids a reverse runtime dependency", () => {
    const store = normalized(runStoreSource);

    expect(store).toContain("struct RunSummaryRow {");
    expect(store).toContain("impl From<RunSummaryRow> for PromptPackRunSummaryDto");
    const expectedTimePath = crateExtracted
      ? ".bind(extractum_core::time::now_rfc3339_utc())"
      : ".bind(crate::time::now_rfc3339_utc())";
    expect(store).toContain(expectedTimePath);
    expect(store).not.toContain("super::runtime");
    expect(store).not.toMatch(/(?:pub|pub\([^)]*\))\s+struct RunSummaryRow/);
  });

  it("leaves lifecycle and state-aware services in runtime", () => {
    const runtime = normalized(runtimeSource);
    const store = normalized(runStoreSource);

    for (const service of [
      "cleanup_interrupted_prompt_pack_runs_in_pool",
      "seed_prompt_pack_cancellation_smoke_fixture_in_pool",
      "clear_prompt_pack_cancellation_smoke_fixture_in_pool",
    ]) {
      expect(runtime).toMatch(new RegExp(`async\\s+fn\\s+${service}\\s*\\(`));
      expect(store).not.toMatch(new RegExp(`async\\s+fn\\s+${service}\\s*\\(`));
    }
    expect(runtime).toContain("SET run_status = 'cancelled'");
    expect(runtime).toContain("SET run_status = 'interrupted'");
  });
});
