import { describe, expect, it } from "vitest";
import analysisPageSource from "../routes/analysis/+page.svelte?raw";
import reportSetupPanelSource from "./components/analysis/report-setup-panel.svelte?raw";

describe("analysis LLM run controls", () => {
  it("loads LLM profiles and provider models for the analysis controls", () => {
    expect(analysisPageSource).toContain("getLlmProfiles");
    expect(analysisPageSource).toContain("listLlmProviderModels");
    expect(analysisPageSource).toContain("selectedLlmProfileId");
    expect(analysisPageSource).toContain("selectedLlmModel");
    expect(analysisPageSource).toContain("runModelOverride()");
    expect(analysisPageSource).toContain("runProfileId()");
  });

  it("uses profile and model selects instead of a plain model override field", () => {
    expect(reportSetupPanelSource).toContain("LLM profile");
    expect(reportSetupPanelSource).toContain("Use active profile");
    expect(reportSetupPanelSource).toContain("Model");
    expect(reportSetupPanelSource).toContain("Profile default");
    expect(reportSetupPanelSource).toContain("Custom model...");
    expect(reportSetupPanelSource).not.toContain(">Model override");
  });
});
