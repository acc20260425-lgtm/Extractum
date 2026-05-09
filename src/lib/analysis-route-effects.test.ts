import { describe, expect, it } from "vitest";
import analysisPageSource from "../routes/analysis/+page.svelte?raw";

describe("analysis route effects", () => {
  it("keeps saved run history loading out of effect dependency tracking", () => {
    const historyEffect = analysisPageSource.match(
      /\$effect\(\(\) => {\s+if \(historyScopeParams === null\) {\s+runs = \[];\s+return;\s+}\s+void [^;]+;\s+}\);/,
    )?.[0];

    expect(historyEffect).toBeDefined();
    expect(historyEffect).toContain("void untrack(() => loadRuns());");
  });
});
