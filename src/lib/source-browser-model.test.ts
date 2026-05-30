import { describe, expect, it } from "vitest";
import {
  reconcileSourceBrowserTab,
  sourceBrowserShellAppliesToSource,
  sourceBrowserTabsForSource,
  smartDefaultSourceBrowserTab,
  type SourceBrowserTabId,
} from "./source-browser-model";
import type { Source } from "./types/sources";

function source(overrides: Partial<Source>): Source {
  return {
    id: 1,
    sourceType: "telegram",
    sourceSubtype: "supergroup",
    accountId: 10,
    externalId: "demo",
    title: "Demo",
    lastSyncState: null,
    lastSyncedAt: null,
    isMember: true,
    isActive: true,
    createdAt: 1,
    telegramUsername: null,
    avatarDataUrl: null,
    migratedHistoryStatus: "none",
    migratedHistoryDetectedAt: null,
    migratedHistoryRefreshedAt: null,
    migratedHistoryRowCount: 0,
    migratedHistoryImportCompleted: false,
    ...overrides,
  };
}

describe("source browser model", () => {
  it("derives canonical tabs for supported source types", () => {
    expect(sourceBrowserTabsForSource(source({ sourceType: "telegram" })).map((tab) => tab.id))
      .toEqual(["timeline", "items", "metadata", "activity"]);
    expect(sourceBrowserTabsForSource(source({ sourceType: "youtube", sourceSubtype: "video" })).map((tab) => tab.id))
      .toEqual(["transcript", "comments", "items", "metadata", "activity"]);
    expect(sourceBrowserTabsForSource(source({ sourceType: "rss", sourceSubtype: "feed" })).map((tab) => tab.id))
      .toEqual(["items", "metadata", "activity"]);
  });

  it("selects smart defaults by canonical tab id", () => {
    expect(smartDefaultSourceBrowserTab(source({ sourceType: "telegram" }))).toBe("timeline");
    expect(smartDefaultSourceBrowserTab(source({ sourceType: "youtube", sourceSubtype: "video" }))).toBe("transcript");
    expect(smartDefaultSourceBrowserTab(source({ sourceType: "forum", sourceSubtype: "thread" }))).toBe("items");
  });

  it("preserves an active tab across source changes only when supported", () => {
    const youtube = source({ id: 2, sourceType: "youtube", sourceSubtype: "video" });
    const telegram = source({ id: 3, sourceType: "telegram", sourceSubtype: "supergroup" });
    const active: SourceBrowserTabId = "comments";

    expect(reconcileSourceBrowserTab(active, youtube)).toBe("comments");
    expect(reconcileSourceBrowserTab(active, telegram)).toBe("timeline");
    expect(reconcileSourceBrowserTab("metadata", telegram)).toBe("metadata");
  });

  it("routes only Telegram and YouTube video live sources into the shell in this slice", () => {
    expect(sourceBrowserShellAppliesToSource(source({ sourceType: "telegram", sourceSubtype: "supergroup" }))).toBe(true);
    expect(sourceBrowserShellAppliesToSource(source({ sourceType: "youtube", sourceSubtype: "video" }))).toBe(true);
    expect(sourceBrowserShellAppliesToSource(source({ sourceType: "youtube", sourceSubtype: "playlist" }))).toBe(false);
    expect(sourceBrowserShellAppliesToSource(source({ sourceType: "rss", sourceSubtype: "feed" }))).toBe(false);
  });
});
