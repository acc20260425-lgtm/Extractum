import { describe, expect, it } from "vitest";
import layoutSource from "../routes/+layout.svelte?raw";
import accountsPageSource from "../routes/accounts/+page.svelte?raw";
import settingsPageSource from "../routes/settings/+page.svelte?raw";

describe("source access placement", () => {
  it("keeps the YouTube auth and sync panel on the Accounts page", () => {
    expect(accountsPageSource).toContain(
      'import YoutubeSettingsPanel from "$lib/components/settings/youtube-settings-panel.svelte";',
    );
    expect(accountsPageSource).toContain("<YoutubeSettingsPanel />");
    expect(accountsPageSource).toContain('<span class="page-eyebrow">Source access</span>');
    expect(accountsPageSource).toContain("Manage source identities and authentication used for sync and analysis.");
  });

  it("keeps Settings focused on LLM configuration", () => {
    expect(settingsPageSource).not.toContain("YoutubeSettingsPanel");
    expect(settingsPageSource).toContain("Settings stay focused on LLM provider profiles and test runs.");
  });

  it("keeps the navigation label as Accounts while broadening the caption", () => {
    expect(layoutSource).toContain('label: "Accounts"');
    expect(layoutSource).toContain('caption: "Source access"');
    expect(layoutSource).toContain("Source access");
  });
});
