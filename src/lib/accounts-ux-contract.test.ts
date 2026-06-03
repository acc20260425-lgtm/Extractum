import { describe, expect, it } from "vitest";
import youtubeSettingsPanelSource from "./components/settings/youtube-settings-panel.svelte?raw";
import accountsPageSource from "../routes/accounts/+page.svelte?raw";

describe("accounts UX contract", () => {
  it("separates Telegram identity from YouTube access", () => {
    expect(accountsPageSource).toContain("Telegram accounts");
    expect(accountsPageSource).toContain("YouTube access");
    expect(accountsPageSource).toContain("Manage cookies and sync limits without mixing them into Telegram account identity.");
    expect(accountsPageSource).toContain("<YoutubeSettingsPanel embedded />");
  });

  it("keeps YouTube auth and sync settings in separate visual groups", () => {
    expect(youtubeSettingsPanelSource).toContain('class="youtube-auth-section"');
    expect(youtubeSettingsPanelSource).toContain('class="youtube-sync-policy-section"');
    expect(youtubeSettingsPanelSource).toContain("Authentication");
    expect(youtubeSettingsPanelSource).toContain("Sync policy");
  });

  it("does not render embedded YouTube settings as a nested desk panel", () => {
    expect(youtubeSettingsPanelSource).toContain('class={`youtube-settings-panel ${embedded ? "embedded" : "desk-panel desk-panel-subtle"}`.trim()}');
    expect(youtubeSettingsPanelSource).not.toContain('class="desk-panel desk-panel-subtle youtube-settings-panel" class:embedded');
  });
});
