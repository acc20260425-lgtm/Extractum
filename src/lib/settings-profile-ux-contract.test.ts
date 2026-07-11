import { describe, expect, it } from "vitest";
import settingsPageSource from "../routes/settings/+page.svelte?raw";

describe("settings profile UX contract", () => {
  it("separates active profile from the profile being edited", () => {
    expect(settingsPageSource).toContain('class="profile-status-strip"');
    expect(settingsPageSource).toContain("Active profile");
    expect(settingsPageSource).toContain("Editing profile");
    expect(settingsPageSource).toContain("Set active after save");
  });

  it("adds model search before the large model selector", () => {
    expect(settingsPageSource).toContain("modelQuery");
    expect(settingsPageSource).toContain("filteredAvailableModels");
    expect(settingsPageSource).toContain('ariaLabel="Search models"');
  });

  it("uses the materialized snake_case backend URL when selecting a profile", () => {
    expect(settingsPageSource).toContain("baseUrl = profile.base_url");
  });
});
