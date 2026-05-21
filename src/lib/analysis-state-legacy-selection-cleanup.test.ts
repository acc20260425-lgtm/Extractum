import { describe, expect, it } from "vitest";
import analysisStateSource from "./analysis-state.ts?raw";

describe("analysis state legacy selection cleanup", () => {
  it("does not keep route-era selection helper exports", () => {
    expect(analysisStateSource).not.toContain("AnalysisSourceSelectionState");
    expect(analysisStateSource).not.toContain("AnalysisGroupSelectionState");
    expect(analysisStateSource).not.toContain("analysisSourceSelectionState");
    expect(analysisStateSource).not.toContain("analysisGroupSelectionState");
  });
});
