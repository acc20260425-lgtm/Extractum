import { describe, expect, it } from "vitest";
import analysisPageSource from "../routes/analysis/+page.svelte?raw";

describe("analysis route effects", () => {
  it("keeps saved run history loading out of effect dependency tracking", () => {
    const importsUntrack = analysisPageSource.includes('import { onMount, untrack } from "svelte";');
    const usesUntrackedLoad = analysisPageSource.includes("void untrack(() => loadRuns());");
    const usesTrackedLoad = analysisPageSource.includes("void loadRuns();");

    expect(importsUntrack, "analysis route should import Svelte untrack").toBe(true);
    expect(usesUntrackedLoad, "history-scope effect should call loadRuns through untrack").toBe(true);
    expect(usesTrackedLoad, "history-scope effect must not call loadRuns directly").toBe(false);
  });
});
