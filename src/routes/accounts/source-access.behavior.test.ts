import { expect, it } from "vitest";

import YoutubeSettingsPanel from "$lib/components/settings/youtube-settings-panel.svelte";
import { ACCOUNT_SOURCE_ACCESS, sourceAccessNavigationItem } from "./source-access";

  it("keeps the YouTube auth and sync panel on the Accounts page", () => {
    const telegram = ACCOUNT_SOURCE_ACCESS.sections[0];
    const youtube = ACCOUNT_SOURCE_ACCESS.sections[1];
    expect(ACCOUNT_SOURCE_ACCESS.eyebrow).toBe("Source access");
    expect(ACCOUNT_SOURCE_ACCESS.pageTitle).toBe("Accounts");
    expect(ACCOUNT_SOURCE_ACCESS.description).toContain("source identities and authentication");
    expect(ACCOUNT_SOURCE_ACCESS.sections).toHaveLength(2);
    expect(ACCOUNT_SOURCE_ACCESS.sections.map((section) => section.kind)).toEqual(["telegram", "youtube"]);
    expect(telegram.heading).toBe("Telegram accounts");
    expect(youtube.heading).toBe("YouTube access");
    expect(youtube.panel).toBe(YoutubeSettingsPanel);
    expect(youtube.panelProps).toEqual({ embedded: true });
    expect(telegram.labelledBy).toBe("telegram-accounts-heading");
    expect(youtube.labelledBy).toBe("youtube-access-heading");
  });

  it("keeps embedded YouTube access visually inside one shell", () => {
    expect(ACCOUNT_SOURCE_ACCESS.sections[0].shell).toBe("desk-panel account-catalog");
    expect(ACCOUNT_SOURCE_ACCESS.sections[1].shell).toBe("desk-panel youtube-access-shell");
    expect(ACCOUNT_SOURCE_ACCESS.sections.map((section) => section.labelledBy)).toEqual(["telegram-accounts-heading", "youtube-access-heading"]);
  });

  it("keeps the navigation label as Accounts while broadening the caption", () => {
    expect(sourceAccessNavigationItem().label).toBe("Accounts");
    expect(sourceAccessNavigationItem().caption).toBe("Source access");
    expect(sourceAccessNavigationItem().href).toBe("/accounts");
  });
