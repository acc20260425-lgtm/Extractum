import { describe, expect, it } from "vitest";
import {
  commentsCoverageState,
  filterLoadedSourceItems,
  filterLoadedYoutubeComments,
  groupLoadedYoutubeComments,
  reconcileSourceBrowserTab,
  sortLoadedYoutubeComments,
  sourceBrowserShellAppliesToSource,
  sourceItemKindChips,
  sourceBrowserTabsForSource,
  smartDefaultSourceBrowserTab,
  sortLoadedSourceItems,
  type SourceBrowserTabId,
} from "./source-browser-model";
import type { Source, SourceItem, SourceJobRecord } from "./types/sources";
import type { YoutubeVideoDetail } from "./types/youtube";

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

function youtubeCommentItem(overrides: Partial<SourceItem> = {}): SourceItem {
  return sourceItem({
    itemKind: "youtube_comment",
    youtubeComment: {
      commentId: "c1",
      parentCommentId: null,
      isReply: false,
      likeCount: 0,
      isPinned: false,
      isHearted: false,
      authorChannelUrl: null,
    },
    ...overrides,
  });
}

function youtubeDetail(commentState: YoutubeVideoDetail["summary"]["comments"]["state"], itemCount = 0): YoutubeVideoDetail {
  return {
    summary: {
      sourceId: 20,
      sourceSubtype: "video",
      title: "Demo video",
      channelTitle: null,
      channelHandle: null,
      canonicalUrl: "https://www.youtube.com/watch?v=demo",
      thumbnailUrl: null,
      durationSeconds: null,
      publishedAt: null,
      availabilityStatus: "available",
      videoCount: null,
      linkedVideoCount: null,
      unavailableCount: null,
      captions: {
        state: "unknown",
        itemCount: 0,
        segmentCount: 0,
        lastSyncedAt: null,
        label: "Unknown",
      },
      comments: {
        state: commentState,
        itemCount,
        segmentCount: 0,
        lastSyncedAt: null,
        label: commentState,
      },
    },
    sourceMetadata: {
      sourceId: 20,
      videoId: "demo",
      canonicalUrl: "https://www.youtube.com/watch?v=demo",
      title: "Demo video",
      channelTitle: null,
      channelId: null,
      channelHandle: null,
      channelUrl: null,
      authorDisplay: null,
      publishedAt: null,
      durationSeconds: null,
      description: null,
      thumbnailUrl: null,
      viewCount: null,
      likeCount: null,
      commentCount: null,
      category: null,
      videoForm: "regular",
      availabilityStatus: "available",
      captionLanguageOverride: null,
      rawMetadataVersion: null,
      rawMetadataJson: null,
    },
    playlistMemberships: [],
  };
}

function sourceJob(overrides: Partial<SourceJobRecord> = {}): SourceJobRecord {
  return {
    job_id: "job-1",
    source_id: 20,
    related_source_id: null,
    job_type: "youtube_video_comments_sync",
    status: "running",
    message: null,
    progress_current: null,
    progress_total: null,
    started_at: 1,
    finished_at: null,
    warnings: [],
    error: null,
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

  it("derives comments coverage states from loaded rows detail jobs and route errors", () => {
    expect(commentsCoverageState({
      items: [],
      detail: null,
      jobs: [],
      routeError: null,
      loadingItems: false,
    })).toBe("unknown");
    expect(commentsCoverageState({
      items: [],
      detail: youtubeDetail("not_synced"),
      jobs: [],
      routeError: null,
      loadingItems: false,
    })).toBe("not_synced");
    expect(commentsCoverageState({
      items: [],
      detail: youtubeDetail("not_synced"),
      jobs: [sourceJob()],
      routeError: null,
      loadingItems: false,
    })).toBe("syncing");
    expect(commentsCoverageState({
      items: [],
      detail: youtubeDetail("synced"),
      jobs: [],
      routeError: "failed to load items",
      loadingItems: false,
    })).toBe("failed");
    expect(commentsCoverageState({
      items: [],
      detail: youtubeDetail("synced", 0),
      jobs: [],
      routeError: null,
      loadingItems: false,
    })).toBe("synced_empty");
    expect(commentsCoverageState({
      items: [youtubeCommentItem()],
      detail: youtubeDetail("synced", 1),
      jobs: [],
      routeError: null,
      loadingItems: false,
    })).toBe("synced_with_rows");
  });

  it("groups loaded YouTube comments while keeping orphan replies visible", () => {
    const parent = youtubeCommentItem({
      id: 1,
      youtubeComment: {
        commentId: "parent",
        parentCommentId: null,
        isReply: false,
        likeCount: 5,
        isPinned: false,
        isHearted: false,
        authorChannelUrl: null,
      },
    });
    const loadedReply = youtubeCommentItem({
      id: 2,
      youtubeComment: {
        commentId: "reply",
        parentCommentId: "parent",
        isReply: true,
        likeCount: 1,
        isPinned: false,
        isHearted: false,
        authorChannelUrl: null,
      },
    });
    const orphanReply = youtubeCommentItem({
      id: 3,
      youtubeComment: {
        commentId: "orphan",
        parentCommentId: "missing",
        isReply: true,
        likeCount: 2,
        isPinned: false,
        isHearted: false,
        authorChannelUrl: null,
      },
    });

    expect(groupLoadedYoutubeComments([loadedReply, orphanReply, parent])).toEqual([
      { item: parent, replies: [{ item: loadedReply, parentLoaded: true }], parentLoaded: true },
      { item: orphanReply, replies: [], parentLoaded: false },
    ]);
  });

  it("filters loaded YouTube comments by loaded text and author", () => {
    const items = [
      youtubeCommentItem({ id: 1, author: "Alice", content: "First comment", externalId: "hidden-match" }),
      youtubeCommentItem({ id: 2, author: "Bob", content: "Second note" }),
      sourceItem({ id: 3, itemKind: "telegram_message", author: "Alice", content: "Not a comment" }),
    ];

    expect(filterLoadedYoutubeComments(items, "alice").map((item) => item.id)).toEqual([1]);
    expect(filterLoadedYoutubeComments(items, "second").map((item) => item.id)).toEqual([2]);
    expect(filterLoadedYoutubeComments(items, "hidden-match")).toEqual([]);
  });

  it("sorts loaded YouTube comments by most liked loaded rows", () => {
    const items = [
      youtubeCommentItem({ id: 1, publishedAt: 30, youtubeComment: { commentId: "a", parentCommentId: null, isReply: false, likeCount: 1, isPinned: false, isHearted: false, authorChannelUrl: null } }),
      youtubeCommentItem({ id: 2, publishedAt: 20, youtubeComment: { commentId: "b", parentCommentId: null, isReply: false, likeCount: 10, isPinned: false, isHearted: false, authorChannelUrl: null } }),
      youtubeCommentItem({ id: 3, publishedAt: 10, youtubeComment: { commentId: "c", parentCommentId: null, isReply: false, likeCount: null, isPinned: false, isHearted: false, authorChannelUrl: null } }),
    ];

    expect(sortLoadedYoutubeComments(items, "most_liked").map((item) => item.id)).toEqual([2, 1, 3]);
  });
});
