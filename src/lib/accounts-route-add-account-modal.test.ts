import { describe, expect, it } from "vitest";
import accountsPageSource from "../routes/accounts/+page.svelte?raw";

describe("accounts route add-account modal", () => {
  it("keeps the account creation form behind a configured-accounts header action", () => {
    const catalogStart = accountsPageSource.indexOf('<section class="desk-panel account-catalog">');
    const addDialogStart = accountsPageSource.indexOf("<DesktopDialog");

    expect(catalogStart, "accounts page should render the configured accounts catalog").toBeGreaterThan(-1);
    expect(addDialogStart, "accounts page should render the add-account dialog").toBeGreaterThan(-1);
    expect(
      catalogStart,
      "the configured accounts catalog should appear before the add-account dialog",
    ).toBeLessThan(addDialogStart);

    const catalogSource = accountsPageSource.slice(catalogStart, addDialogStart);

    expect(catalogSource).toContain('onclick={() => (accountDialogOpen = true)}');
    expect(catalogSource).toContain("<Plus");
    expect(catalogSource).toContain("Add");
    expect(accountsPageSource).toContain("let accountDialogOpen = $state(false);");
    expect(accountsPageSource).toContain("open={accountDialogOpen}");
    expect(accountsPageSource).toContain('title="New Telegram account"');
    expect(accountsPageSource).toContain("onClose={closeAccountDialog}");
  });
});
