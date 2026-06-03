import { describe, expect, it } from "vitest";
import * as sourceBrowserModel from "./source-browser-model";
import {
  commentsCoverageState,
  deriveRunSnapshotBrowserKind,
  filterLoadedSourceItems,
  filterLoadedYoutubeComments,
  groupLoadedYoutubeComments,
  reconcileSourceBrowserTab,
  sortLoadedYoutubeComments,
  sourceBrowserShellAppliesToSource,
  sourceBrowserShellAppliesToSubject,
  sourceBrowserTabsForSubject,
  sourceItemKindChips,
  sourceItemContextLine,
  sourceItemPreviewText,
  sourceBrowserTabsForSource,
  smartDefaultSourceBrowserTab,
  sortLoadedSourceItems,
  type SourceBrowserTabId,
} from "./source-browser-model";
import type { SourceReaderItem } from "./source-reader-model";
import type { AnalysisSourceGroup } from "./types/analysis";
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

function sourceGroup(overrides: Partial<AnalysisSourceGroup> = {}): AnalysisSourceGroup {
  return {
    id: 100,
    name: "Research group",
    source_type: "telegram",
    members: [
      { source_id: 1, source_title: "Alpha", item_count: 12 },
      { source_id: 2, source_title: "Beta", item_count: 7 },
    ],
    created_at: 1710000000,
    updated_at: 1710000500,
    ...overrides,
  };
}

function snapshotSubject(
  readerKind: "source_group" | "telegram_timeline" | "youtube_transcript" | "generic_items",
  overrides: Partial<{
    runId: number;
    scopeType: "source" | "source_group";
    scopeLabel: string;
    sourceType: Source["sourceType"] | null;
    sourceSubtype: Source["sourceSubtype"] | null;
  }> = {},
) {
  return {
    kind: "run_snapshot" as const,
    snapshot: {
      runId: overrides.runId ?? 500,
      scopeType: overrides.scopeType ?? (readerKind === "source_group" ? "source_group" : "source"),
      scopeLabel: overrides.scopeLabel ?? "Snapshot run",
      readerKind,
      sourceType: overrides.sourceType ?? null,
      sourceSubtype: overrides.sourceSubtype ?? null,
    },
  };
}

function snapshotReaderItem(overrides: Partial<SourceReaderItem> = {}): SourceReaderItem {
  return {
    id: "snapshot:s1-i1",
    sourceId: 1,
    sourceTitle: "Snapshot source",
    externalId: "external-1",
    ref: "s1-i1",
    kind: "telegram_message",
    author: "Alice",
    publishedAt: 1710000000,
    content: "Snapshot row",
    topicLabel: null,
    replyLabel: null,
    reactionLabel: null,
    mediaCards: [],
    youtubeStartSeconds: null,
    youtubeEndSeconds: null,
    youtubeUrl: null,
    captionLabel: null,
    historyScope: "current",
    historyScopeLabel: null,
    isMigratedHistory: false,
    selected: false,
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
    expect(sourceBrowserTabsForSource(source({ sourceType: "youtube", sourceSubtype: "playlist" })).map((tab) => tab.id))
      .toEqual(["videos", "items", "metadata", "activity"]);
    expect(sourceBrowserTabsForSource(source({ sourceType: "rss", sourceSubtype: "feed" })).map((tab) => tab.id))
      .toEqual(["items", "metadata", "activity"]);
  });

  it("derives canonical tabs for live source group subjects", () => {
    const groupSubject = { kind: "source_group" as const, group: sourceGroup() };

    expect(sourceBrowserTabsForSubject(groupSubject).map((tab) => tab.id))
      .toEqual(["sources", "items", "metadata", "activity"]);
    expect(smartDefaultSourceBrowserTab(groupSubject)).toBe("sources");
    expect(sourceBrowserShellAppliesToSubject(groupSubject)).toBe(true);
  });

  it("derives canonical tabs for run snapshot subjects", () => {
    expect(sourceBrowserTabsForSubject(snapshotSubject("source_group")).map((tab) => tab.id))
      .toEqual(["sources", "items", "metadata"]);
    expect(sourceBrowserTabsForSubject(snapshotSubject("telegram_timeline")).map((tab) => tab.id))
      .toEqual(["timeline", "items", "metadata"]);
    expect(sourceBrowserTabsForSubject(snapshotSubject("youtube_transcript")).map((tab) => tab.id))
      .toEqual(["transcript", "items", "metadata"]);
    expect(sourceBrowserTabsForSubject(snapshotSubject("generic_items")).map((tab) => tab.id))
      .toEqual(["items", "metadata"]);

    expect(smartDefaultSourceBrowserTab(snapshotSubject("source_group"))).toBe("sources");
    expect(smartDefaultSourceBrowserTab(snapshotSubject("telegram_timeline"))).toBe("timeline");
    expect(smartDefaultSourceBrowserTab(snapshotSubject("youtube_transcript"))).toBe("transcript");
    expect(smartDefaultSourceBrowserTab(snapshotSubject("generic_items"))).toBe("items");
    expect(sourceBrowserShellAppliesToSubject(snapshotSubject("generic_items"))).toBe(true);
  });

  it("derives run snapshot reader kinds deterministically", () => {
    expect(deriveRunSnapshotBrowserKind({
      scopeType: "source_group",
      sourceType: "telegram",
      sourceSubtype: null,
      snapshotReaderItems: [],
    })).toBe("source_group");

    expect(deriveRunSnapshotBrowserKind({
      scopeType: "source",
      sourceType: "youtube",
      sourceSubtype: "video",
      snapshotReaderItems: [snapshotReaderItem({ kind: "youtube_transcript" })],
    })).toBe("youtube_transcript");

    expect(deriveRunSnapshotBrowserKind({
      scopeType: "source",
      sourceType: "telegram",
      sourceSubtype: "supergroup",
      snapshotReaderItems: [snapshotReaderItem({ kind: "telegram_message" })],
    })).toBe("telegram_timeline");

    expect(deriveRunSnapshotBrowserKind({
      scopeType: "source",
      sourceType: "youtube",
      sourceSubtype: "video",
      snapshotReaderItems: [
        snapshotReaderItem({ kind: "youtube_transcript" }),
        snapshotReaderItem({ id: "snapshot:s1-c1", kind: "youtube_comment" }),
      ],
    })).toBe("generic_items");

    expect(deriveRunSnapshotBrowserKind({
      scopeType: "source",
      sourceType: "youtube",
      sourceSubtype: "video",
      snapshotReaderItems: [snapshotReaderItem({ kind: "telegram_message" })],
    })).toBe("generic_items");

    expect(deriveRunSnapshotBrowserKind({
      scopeType: "source",
      sourceType: "telegram",
      sourceSubtype: "supergroup",
      snapshotReaderItems: [],
    })).toBe("generic_items");
  });

  it("keeps source helper behavior aligned with subject-aware helpers", () => {
    const samples = [
      source({ sourceType: "telegram", sourceSubtype: "supergroup" }),
      source({ sourceType: "youtube", sourceSubtype: "video" }),
      source({ sourceType: "youtube", sourceSubtype: "playlist" }),
      source({ sourceType: "rss", sourceSubtype: "feed" }),
    ];

    for (const candidate of samples) {
      const subject = { kind: "source" as const, source: candidate };
      expect(sourceBrowserTabsForSource(candidate)).toEqual(sourceBrowserTabsForSubject(subject));
      expect(smartDefaultSourceBrowserTab(candidate)).toBe(smartDefaultSourceBrowserTab(subject));
      expect(sourceBrowserShellAppliesToSource(candidate)).toBe(sourceBrowserShellAppliesToSubject(subject));
    }
  });

  it("reconciles source group tab transitions by subject support", () => {
    const groupSubject = { kind: "source_group" as const, group: sourceGroup() };
    const nextGroupSubject = {
      kind: "source_group" as const,
      group: sourceGroup({ id: 101, name: "Next group" }),
    };
    const telegramSubject = {
      kind: "source" as const,
      source: source({ id: 3, sourceType: "telegram", sourceSubtype: "supergroup" }),
    };
    const youtubeVideoSubject = {
      kind: "source" as const,
      source: source({ id: 4, sourceType: "youtube", sourceSubtype: "video" }),
    };
    const youtubePlaylistSubject = {
      kind: "source" as const,
      source: source({ id: 5, sourceType: "youtube", sourceSubtype: "playlist" }),
    };

    expect(reconcileSourceBrowserTab("items", groupSubject)).toBe("items");
    expect(reconcileSourceBrowserTab("metadata", groupSubject)).toBe("metadata");
    expect(reconcileSourceBrowserTab("activity", groupSubject)).toBe("activity");
    expect(reconcileSourceBrowserTab("timeline", groupSubject)).toBe("sources");
    expect(reconcileSourceBrowserTab("transcript", groupSubject)).toBe("sources");
    expect(reconcileSourceBrowserTab("comments", groupSubject)).toBe("sources");
    expect(reconcileSourceBrowserTab("videos", groupSubject)).toBe("sources");
    expect(reconcileSourceBrowserTab("sources", telegramSubject)).toBe("timeline");
    expect(reconcileSourceBrowserTab("sources", youtubeVideoSubject)).toBe("transcript");
    expect(reconcileSourceBrowserTab("sources", youtubePlaylistSubject)).toBe("videos");
    expect(reconcileSourceBrowserTab("sources", nextGroupSubject)).toBe("sources");
    expect(reconcileSourceBrowserTab("items", nextGroupSubject)).toBe("items");
    expect(reconcileSourceBrowserTab("metadata", nextGroupSubject)).toBe("metadata");
    expect(reconcileSourceBrowserTab("activity", nextGroupSubject)).toBe("activity");
  });

  it("reconciles run snapshot tab transitions without leaking live-only tabs", () => {
    const groupSnapshot = snapshotSubject("source_group");
    const telegramSnapshot = snapshotSubject("telegram_timeline");
    const youtubeSnapshot = snapshotSubject("youtube_transcript");
    const genericSnapshot = snapshotSubject("generic_items");
    const telegramSubject = {
      kind: "source" as const,
      source: source({ id: 3, sourceType: "telegram", sourceSubtype: "supergroup" }),
    };

    expect(reconcileSourceBrowserTab("items", groupSnapshot)).toBe("items");
    expect(reconcileSourceBrowserTab("metadata", groupSnapshot)).toBe("metadata");
    expect(reconcileSourceBrowserTab("activity", groupSnapshot)).toBe("sources");
    expect(reconcileSourceBrowserTab("comments", youtubeSnapshot)).toBe("transcript");
    expect(reconcileSourceBrowserTab("videos", telegramSnapshot)).toBe("timeline");
    expect(reconcileSourceBrowserTab("transcript", genericSnapshot)).toBe("items");
    expect(reconcileSourceBrowserTab("sources", telegramSnapshot)).toBe("timeline");
    expect(reconcileSourceBrowserTab("timeline", telegramSubject)).toBe("timeline");
    expect(reconcileSourceBrowserTab("metadata", telegramSubject)).toBe("metadata");
  });

  it("selects smart defaults by canonical tab id", () => {
    expect(smartDefaultSourceBrowserTab(source({ sourceType: "telegram" }))).toBe("timeline");
    expect(smartDefaultSourceBrowserTab(source({ sourceType: "youtube", sourceSubtype: "video" }))).toBe("transcript");
    expect(smartDefaultSourceBrowserTab(source({ sourceType: "youtube", sourceSubtype: "playlist" }))).toBe("videos");
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

  it("routes Telegram YouTube video and YouTube playlist live sources into the shell", () => {
    expect(sourceBrowserShellAppliesToSource(source({ sourceType: "telegram", sourceSubtype: "supergroup" }))).toBe(true);
    expect(sourceBrowserShellAppliesToSource(source({ sourceType: "youtube", sourceSubtype: "video" }))).toBe(true);
    expect(sourceBrowserShellAppliesToSource(source({ sourceType: "youtube", sourceSubtype: "playlist" }))).toBe(true);
    expect(sourceBrowserShellAppliesToSource(source({ sourceType: "rss", sourceSubtype: "feed" }))).toBe(false);
  });

  it("reconciles playlist tab transitions by canonical tab support", () => {
    const youtubeVideo = source({ id: 2, sourceType: "youtube", sourceSubtype: "video" });
    const youtubePlaylist = source({ id: 3, sourceType: "youtube", sourceSubtype: "playlist" });
    const telegram = source({ id: 4, sourceType: "telegram", sourceSubtype: "supergroup" });

    expect(reconcileSourceBrowserTab("metadata", youtubePlaylist)).toBe("metadata");
    expect(reconcileSourceBrowserTab("items", youtubePlaylist)).toBe("items");
    expect(reconcileSourceBrowserTab("activity", youtubePlaylist)).toBe("activity");
    expect(reconcileSourceBrowserTab("transcript", youtubePlaylist)).toBe("videos");
    expect(reconcileSourceBrowserTab("comments", youtubePlaylist)).toBe("videos");
    expect(reconcileSourceBrowserTab("videos", youtubeVideo)).toBe("transcript");
    expect(reconcileSourceBrowserTab("videos", telegram)).toBe("timeline");
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

  it("labels media-only source items without treating them as empty text", () => {
    expect(sourceItemPreviewText(sourceItem({ content: null, hasMedia: true, mediaKind: "photo" })))
      .toBe("Media-only item (photo). Text was not loaded.");
    expect(sourceItemPreviewText(sourceItem({ content: "Body", hasMedia: false, mediaKind: null }))).toBe("Body");
  });

  it("builds compact item context lines", () => {
    expect(sourceItemContextLine(
      sourceItem({ author: "Alice", externalId: "42", hasMedia: true, mediaKind: "photo" }),
      "Source #7",
    )).toBe("Alice - Source #7 - 42 - photo");
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

  it("formats small raw JSON previews without truncating", () => {
    const formatter = rawJsonFormatter();
    const result = formatter({ id: "video01", source: { visible: true } }, 200);

    expect(result).toEqual({
      preview: '{\n  "id": "video01",\n  "source": {\n    "visible": true\n  }\n}',
      full: '{\n  "id": "video01",\n  "source": {\n    "visible": true\n  }\n}',
      truncated: false,
    });
  });

  it("truncates large raw JSON previews while keeping the full payload", () => {
    const formatter = rawJsonFormatter();
    const result = formatter({ text: "abcdefghijklmnopqrstuvwxyz" }, 18);

    expect(result?.truncated).toBe(true);
    expect(result?.preview).toBe('{\n  "text": "abcde\n...');
    expect(result?.full).toContain("abcdefghijklmnopqrstuvwxyz");
  });

  it("returns null for missing or invalid raw JSON values", () => {
    const formatter = rawJsonFormatter();

    expect(formatter(null, 100)).toBeNull();
    expect(formatter(undefined, 100)).toBeNull();
    expect(formatter({ invalid: BigInt(1) }, 100)).toBeNull();
  });
});

function rawJsonFormatter() {
  const formatter = (sourceBrowserModel as Record<string, unknown>).formatRawJsonPreview;
  expect(formatter).toBeTypeOf("function");
  return formatter as (
    value: unknown,
    maxChars: number,
  ) => { preview: string; full: string; truncated: boolean } | null;
}
