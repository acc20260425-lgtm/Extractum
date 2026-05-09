import { describe, expect, it } from "vitest";
import analysisPageSource from "../routes/analysis/+page.svelte?raw";
import workspaceMainSource from "./components/analysis/workspace-main.svelte?raw";

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
    expect(workspaceMainSource).toContain("LLM profile");
    expect(workspaceMainSource).toContain("Use active profile");
    expect(workspaceMainSource).toContain("Model");
    expect(workspaceMainSource).toContain("Profile default");
    expect(workspaceMainSource).toContain("Custom model...");
    expect(workspaceMainSource).not.toContain(">Model override");
  });
});
