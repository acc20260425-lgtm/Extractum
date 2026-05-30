import { describe, expect, it } from "vitest";
import {
  filterLoadedSourceItems,
  reconcileSourceBrowserTab,
  sourceBrowserShellAppliesToSource,
  sourceItemKindChips,
  sourceBrowserTabsForSource,
  smartDefaultSourceBrowserTab,
  sortLoadedSourceItems,
  type SourceBrowserTabId,
} from "./source-browser-model";
import type { Source, SourceItem } from "./types/sources";

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

function sourceItem(overrides: Partial<SourceItem> = {}): SourceItem {
  return {
    id: 1,
    sourceId: 10,
    externalId: "100",
    itemKind: "telegram_message",
    author: "Alice",
    publishedAt: 1710000000,
    content: "Hello",
    contentKind: "text_only",
    hasMedia: false,
    mediaKind: null,
    mediaSummary: null,
    mediaFileName: null,
    mediaMimeType: null,
    hasRawData: true,
    forumTopicId: null,
    forumTopicTitle: null,
    forumTopicTopMessageId: null,
    replyToMessageId: null,
    replyToPeerKind: null,
    replyToPeerId: null,
    replyToTopMessageId: null,
    reactionCount: null,
    historyScope: "current",
    isMigratedHistory: false,
    migrationDomain: null,
    historyScopeLabel: "Current supergroup history",
    pageCursor: "cursor",
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

  it("derives item kind chips only from loaded rows", () => {
    expect(sourceItemKindChips([
      sourceItem({ id: 1, itemKind: "telegram_message" }),
      sourceItem({ id: 2, itemKind: "youtube_comment" }),
      sourceItem({ id: 3, itemKind: "telegram_message" }),
    ])).toEqual([
      { kind: "telegram_message", label: "Telegram message", count: 2 },
      { kind: "youtube_comment", label: "YouTube comment", count: 1 },
    ]);
  });

  it("filters loaded source items by kind plus loaded content and author", () => {
    const items = [
      sourceItem({ id: 1, itemKind: "telegram_message", author: "Alice", content: "Alpha notes", externalId: "hidden-video" }),
      sourceItem({ id: 2, itemKind: "youtube_comment", author: "Bob", content: "Video comment" }),
      sourceItem({ id: 3, itemKind: "rss_entry", author: null, content: "Release notes" }),
    ];

    expect(filterLoadedSourceItems(items, { kind: "youtube_comment", search: "video" }).map((item) => item.id))
      .toEqual([2]);
    expect(filterLoadedSourceItems(items, { kind: null, search: "alice" }).map((item) => item.id))
      .toEqual([1]);
    expect(filterLoadedSourceItems(items, { kind: null, search: "hidden-video" })).toEqual([]);
  });

  it("sorts only the loaded source item rows by timestamp", () => {
    const items = [
      sourceItem({ id: 1, publishedAt: 30 }),
      sourceItem({ id: 2, publishedAt: 10 }),
      sourceItem({ id: 3, publishedAt: 20 }),
    ];

    expect(sortLoadedSourceItems(items, "newest").map((item) => item.id)).toEqual([1, 3, 2]);
    expect(sortLoadedSourceItems(items, "oldest").map((item) => item.id)).toEqual([2, 3, 1]);
    expect(items.map((item) => item.id)).toEqual([1, 2, 3]);
  });
});
