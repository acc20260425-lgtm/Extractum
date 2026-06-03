import { describe, expect, it } from "vitest";
import youtubeSettingsPanelSource from "./components/settings/youtube-settings-panel.svelte?raw";
import accountsPageSource from "../routes/accounts/+page.svelte?raw";

describe("accounts UX contract", () => {
  it("separates Telegram identity from YouTube access", () => {
    expect(accountsPageSource).toContain("Telegram accounts");
    expect(accountsPageSource).toContain("YouTube access");
    expect(accountsPageSource).toContain("Sync policy");
  });

  it("keeps YouTube auth and sync settings in separate visual groups", () => {
    expect(youtubeSettingsPanelSource).toContain('class="youtube-auth-section"');
    expect(youtubeSettingsPanelSource).toContain('class="youtube-sync-policy-section"');
    expect(youtubeSettingsPanelSource).toContain("Authentication");
    expect(youtubeSettingsPanelSource).toContain("Sync policy");
  });
});
