import { describe, expect, it } from "vitest";
import analysisPageSource from "../routes/analysis/+page.svelte?raw";

describe("analysis route effects", () => {
  function historyScopeEffect() {
    const effectStart = analysisPageSource.indexOf("  $effect(() => {");
    const nextEffectStart = analysisPageSource.indexOf("\n  $effect(() => {", effectStart + 1);

    expect(effectStart, "analysis route should define a history-scope effect").toBeGreaterThan(-1);
    expect(
      nextEffectStart,
      "history-scope effect should be followed by another effect",
    ).toBeGreaterThan(effectStart);

    return analysisPageSource.slice(effectStart, nextEffectStart);
  }

  it("keeps saved run history loading out of effect dependency tracking", () => {
    const effect = historyScopeEffect();

    expect(effect, "history-scope effect should read only the explicit scope params").toContain(
      "const params = historyScopeParams;",
    );
    expect(effect, "history-scope effect should call the explicit-scope loader").toContain(
      "void runWorkflow.loadRunsForScope(params);",
    );
    expect(effect, "history-scope effect must not call the broad wrapper directly").not.toContain(
      "loadRuns();",
    );
    expect(effect, "history-scope effect should not need untrack after explicit params").not.toContain(
      "untrack(",
    );
  });
});
