import { expect, it } from "vitest";

import {
  filterSettingsModels,
  materializeSettingsProfile,
  settingsProfileStatus,
} from "./settings-profiles";

const alpha = {
  profile_id: "alpha",
  provider: "openai_compatible",
  default_model: "model-a",
  api_key_configured: true,
  base_url: "https://alpha.example/v1",
};

  it("separates active profile from the profile being edited", () => {
    const status = settingsProfileStatus("alpha", "beta", false);
    expect(status.activeLabel).toBe("Active profile: alpha");
    expect(status.editingLabel).toBe("Editing profile: beta");
    expect(status.showSetActiveAfterSave).toBe(true);
    expect(settingsProfileStatus("alpha", "alpha", false).showSetActiveAfterSave).toBe(false);
  });

  it("adds model search before the large model selector", () => {
    const models = [
      { model: "alpha-pro", display_name: "Alpha Pro" },
      { model: "beta-fast", display_name: "Beta Fast" },
    ];
    expect(filterSettingsModels(models, "alpha")).toEqual([models[0]]);
    expect(filterSettingsModels(models, "FAST")).toEqual([models[1]]);
    expect(filterSettingsModels(models, "  ")).toEqual(models);
  });

  it("uses the materialized snake_case backend URL when selecting a profile", () => {
    expect(materializeSettingsProfile(alpha).baseUrl).toBe("https://alpha.example/v1");
  });
