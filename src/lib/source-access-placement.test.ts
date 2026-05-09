import { describe, expect, it } from "vitest";
import layoutSource from "../routes/+layout.svelte?raw";
import accountsPageSource from "../routes/accounts/+page.svelte?raw";
import settingsPageSource from "../routes/settings/+page.svelte?raw";

describe("source access placement", () => {
  it("keeps the YouTube auth and sync panel on the Accounts page", () => {
    const catalogStart = accountsPageSource.indexOf('<section class="desk-panel account-catalog">');
    const youtubePanel = accountsPageSource.indexOf("<YoutubeSettingsPanel />");
    const stackAfterYoutube = accountsPageSource.indexOf("</div>", youtubePanel);

    expect(accountsPageSource).toContain(
      'import YoutubeSettingsPanel from "$lib/components/settings/youtube-settings-panel.svelte";',
    );
    expect(accountsPageSource).toContain("<YoutubeSettingsPanel />");
    expect(catalogStart, "configured accounts catalog should be present").toBeGreaterThan(-1);
    expect(youtubePanel, "YouTube auth panel should be present").toBeGreaterThan(catalogStart);
    expect(
      accountsPageSource.slice(catalogStart, stackAfterYoutube),
      "YouTube auth panel should sit below configured accounts in the same stack",
    ).toContain("<YoutubeSettingsPanel />");
    expect(accountsPageSource).not.toContain('<div class="page-stack">\n      <YoutubeSettingsPanel />');
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
