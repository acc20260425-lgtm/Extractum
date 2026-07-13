import { describe, expect, it } from "vitest";

import promptPacksModuleSource from "../../src-tauri/src/prompt_packs/mod.rs?raw";
import runStoreSource from "../../src-tauri/src/prompt_packs/run_store.rs?raw";
import runtimeSource from "../../src-tauri/src/prompt_packs/runtime.rs?raw";

const normalized = (source: string) => source.replace(/\r\n/g, "\n");

const extractedFunctions = [
  "list_prompt_pack_runs_in_pool",
  "update_prompt_pack_run_in_pool",
  "delete_prompt_pack_run_in_pool",
  "list_prompt_pack_run_stages_in_pool",
  "load_run_summary_optional",
] as const;

describe("Prompt Pack run store ownership", () => {
  it("registers a private run_store sibling module", () => {
    const source = normalized(promptPacksModuleSource);

    expect(source).toMatch(/^mod run_store;$/m);
    expect(source).not.toMatch(/pub(?:\([^)]*\))?\s+mod run_store;/);
  });

  it.each(extractedFunctions)("moves %s out of runtime", (functionName) => {
    const store = normalized(runStoreSource);
    const runtime = normalized(runtimeSource);
    const definition = new RegExp(
      `pub\\(super\\)\\s+async\\s+fn\\s+${functionName}\\s*\\(`,
    );
    const runtimeDefinition = new RegExp(
      `(?:pub\\(crate\\)\\s+|pub\\(super\\)\\s+)?async\\s+fn\\s+${functionName}\\s*\\(`,
    );

    expect(store).toMatch(definition);
    expect(runtime).not.toMatch(runtimeDefinition);
  });

  it("keeps row mapping private and avoids a reverse runtime dependency", () => {
    const store = normalized(runStoreSource);

    expect(store).toContain("struct RunSummaryRow {");
    expect(store).toContain("impl From<RunSummaryRow> for PromptPackRunSummaryDto");
    expect(store).toContain(".bind(crate::time::now_rfc3339_utc())");
    expect(store).not.toContain("super::runtime");
    expect(store).not.toMatch(/(?:pub|pub\([^)]*\))\s+struct RunSummaryRow/);
  });

  it("leaves lifecycle SQL in runtime", () => {
    const runtime = normalized(runtimeSource);

    expect(runtime).toContain("async fn mark_prompt_pack_run_failed(");
    expect(runtime).toContain("pub(crate) async fn cleanup_interrupted_prompt_pack_runs_in_pool(");
    expect(runtime).toContain("async fn seed_prompt_pack_cancellation_smoke_fixture_in_pool(");
  });
});
