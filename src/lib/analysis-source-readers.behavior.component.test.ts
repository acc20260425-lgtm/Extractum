import { afterEach, beforeAll, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import SourceBrowserShell from "$lib/components/analysis/source-browser-shell.svelte";
import SourceReaderHeader from "$lib/components/analysis/source-reader-header.svelte";
import type { SourceBrowserSubject } from "$lib/source-browser-model";
import type { SourceReaderItem } from "$lib/source-reader-model";
import type { AnalysisRunDetail, AnalysisSourceGroup } from "$lib/types/analysis";
import type { Source, SourceItem } from "$lib/types/sources";
import type { YoutubePlaylistDetail, YoutubeVideoDetail } from "$lib/types/youtube";

beforeAll(() => {
  Object.defineProperty(HTMLElement.prototype, "scrollIntoView", {
    configurable: true,
    value: vi.fn(),
  });
  vi.stubGlobal("CSS", { escape: (value: string) => value });
});

afterEach(cleanup);

const formatTimestamp = (value: number | null) => value === null ? "Never" : `time:${value}`;

function source(overrides: Partial<Source> = {}): Source {
  return {
    id: 1,
    sourceType: "telegram",
    sourceSubtype: "supergroup",
    accountId: 7,
    externalId: "source-1",
    title: "Research channel",
    lastSyncState: 1,
    lastSyncedAt: 1_700_000_000,
    isMember: true,
    isActive: true,
    createdAt: 1_699_000_000,
    telegramUsername: "research",
    avatarDataUrl: null,
    migratedHistoryStatus: "none",
    migratedHistoryDetectedAt: null,
    migratedHistoryRefreshedAt: null,
    migratedHistoryRowCount: 0,
    migratedHistoryImportCompleted: false,
    ...overrides,
  };
}

function readerItem(overrides: Partial<SourceReaderItem> = {}): SourceReaderItem {
  return {
    id: "reader-1",
    sourceId: 1,
    sourceTitle: "Research channel",
    externalId: "message-1",
    ref: "source:1:item:1",
    kind: "telegram_message",
    author: "Ada",
    publishedAt: 1_700_000_000,
    content: "A browsable source row",
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
    sourceId: 1,
    externalId: "message-1",
    itemKind: "telegram_message",
    author: "Ada",
    publishedAt: 1_700_000_000,
    content: "A loaded source item",
    contentKind: "text",
    hasMedia: false,
    mediaKind: null,
    mediaSummary: null,
    mediaFileName: null,
    mediaMimeType: null,
    hasRawData: false,
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
    pageCursor: "cursor-1",
    ...overrides,
  };
}

function group(overrides: Partial<AnalysisSourceGroup> = {}): AnalysisSourceGroup {
  return {
    id: 20,
    name: "Mixed research group",
    source_type: "youtube",
    members: [
      { source_id: 1, source_title: "Research channel", item_count: 1 },
      { source_id: 2, source_title: "Research video", item_count: 2 },
    ],
    created_at: 1_699_000_000,
    updated_at: 1_700_000_000,
    ...overrides,
  };
}

function run(overrides: Partial<AnalysisRunDetail> = {}): AnalysisRunDetail {
  return {
    id: 30,
    run_type: "report",
    scope_type: "project",
    source_id: null,
    source_title: null,
    source_group_id: null,
    source_group_name: null,
    project_id: 40,
    project_name: "Project Alpha",
    scope_label: "Project sources",
    period_from: 1_699_000_000,
    period_to: 1_700_000_000,
    output_language: "en",
    prompt_template_id: 1,
    prompt_template_name: "Research report",
    prompt_template_version: 3,
    provider_profile: "Default",
    provider: "openai",
    model: "model-a",
    youtube_corpus_mode: "transcript_only",
    telegram_history_scope: "current",
    status: "completed",
    error: null,
    has_trace_data: true,
    snapshot_state: "captured",
    snapshot_captured_at: "2026-08-03T10:00:00Z",
    snapshot_error: null,
    created_at: 1_700_000_000,
    completed_at: 1_700_000_100,
    result_markdown: "# Result",
    ...overrides,
  };
}

function youtubeVideoDetail(overrides: Partial<YoutubeVideoDetail> = {}): YoutubeVideoDetail {
  const synced = { state: "synced" as const, itemCount: 2, segmentCount: 3, lastSyncedAt: 1_700_000_000, label: "Synced" };
  return {
    summary: {
      sourceId: 1,
      sourceSubtype: "video",
      title: "Rendered video title",
      channelTitle: "Research channel",
      channelHandle: "@research",
      canonicalUrl: "https://www.youtube.com/watch?v=video-1",
      thumbnailUrl: null,
      durationSeconds: 120,
      publishedAt: 1_700_000_000,
      availabilityStatus: "available",
      videoCount: null,
      linkedVideoCount: null,
      unavailableCount: null,
      captions: synced,
      comments: synced,
    },
    sourceMetadata: {
      sourceId: 1,
      videoId: "video-1",
      canonicalUrl: "https://www.youtube.com/watch?v=video-1",
      title: "Rendered video title",
      channelTitle: "Research channel",
      channelId: "channel-1",
      channelHandle: "@research",
      channelUrl: "https://www.youtube.com/@research",
      authorDisplay: "Research channel",
      publishedAt: 1_700_000_000,
      durationSeconds: 120,
      description: "A research video description",
      thumbnailUrl: null,
      viewCount: 100,
      likeCount: 10,
      commentCount: 2,
      category: "Education",
      videoForm: "long",
      availabilityStatus: "available",
      captionLanguageOverride: null,
      rawMetadataVersion: 1,
      rawMetadataJson: { bounded: "metadata" },
    },
    playlistMemberships: [],
    ...overrides,
  };
}

function youtubePlaylistDetail(): YoutubePlaylistDetail {
  const synced = { state: "synced" as const, itemCount: 1, segmentCount: 1, lastSyncedAt: 1_700_000_000, label: "Synced" };
  return {
    summary: {
      sourceId: 1,
      sourceSubtype: "playlist",
      title: "Rendered playlist title",
      channelTitle: "Research channel",
      channelHandle: "@research",
      canonicalUrl: "https://www.youtube.com/playlist?list=playlist-1",
      thumbnailUrl: null,
      durationSeconds: null,
      publishedAt: 1_700_000_000,
      availabilityStatus: "available",
      videoCount: 1,
      linkedVideoCount: 1,
      unavailableCount: 0,
      captions: synced,
      comments: synced,
    },
    items: [{
      position: 1,
      videoId: "video-2",
      videoSourceId: 2,
      title: "Playlist child video",
      canonicalUrl: "https://www.youtube.com/watch?v=video-2",
      thumbnailUrl: null,
      durationSeconds: 90,
      publishedAt: 1_700_000_000,
      availabilityStatus: "available",
      isRemovedFromPlaylist: false,
      captions: synced,
      comments: synced,
    }],
  };
}

function sourceBrowserData(overrides: Record<string, unknown> = {}) {
  return {
    liveReaderItems: [readerItem()],
    sourceItems: [sourceItem()],
    sourceRouteError: null,
    sourceItemsHasMore: true,
    loadingItems: false,
    sourceTopics: [],
    loadingSourceTopics: false,
    selectedTopicKey: "__all_topics__",
    showTopicSelector: false,
    youtubeVideoDetail: null,
    youtubePlaylistDetail: null,
    youtubeDetailError: null,
    youtubeTranscriptSegments: [],
    youtubeTranscriptSearch: "",
    youtubeTranscriptHasMore: false,
    loadingYoutubeTranscriptSegments: false,
    loadingYoutubeDetail: false,
    sourceJobs: [],
    takeoutRecovery: null,
    sourceSyncDisabledReason: () => null,
    telegramHistoryScope: "current" as const,
    currentSourceContentLabel: "messages",
    onLoadMoreSourceItems: vi.fn(),
    onChangeSelectedTopicKey: vi.fn(),
    onChangeTelegramHistoryScope: vi.fn(),
    onChangeTranscriptSearch: vi.fn(),
    onLoadMoreYoutubeTranscriptSegments: vi.fn(),
    onOpenSource: vi.fn(),
    onSyncSource: vi.fn(),
    onSyncYoutubeMetadata: vi.fn(),
    onSyncYoutubeTranscript: vi.fn(),
    onSyncYoutubeComments: vi.fn(),
    onSyncYoutubePlaylist: vi.fn(),
    onRetryFailedYoutubePlaylistVideos: vi.fn(),
    onSyncYoutubePlaylistVideo: vi.fn(),
    onRetryYoutubePlaylistVideo: vi.fn(),
    onStartTakeoutImport: vi.fn(),
    onStartMigratedHistoryImport: vi.fn(),
    onCancelSourceJob: vi.fn(),
    ...overrides,
  };
}

function groupBrowserData(overrides: Record<string, unknown> = {}) {
  return {
    liveReaderItems: [readerItem(), readerItem({ id: "reader-2", sourceId: 2, sourceTitle: "Research video" })],
    sourceItems: [sourceItem(), sourceItem({ id: 2, sourceId: 2, externalId: "video-2" })],
    selectedSourceId: null,
    hasMoreBySource: { 1: false, 2: false },
    sourceLabelForItem: (item: SourceItem) => item.sourceId === 1 ? "Research channel" : "Research video",
    onLoadSourcePage: vi.fn(),
    youtubeDetailsBySource: {},
    ...overrides,
  };
}

function snapshotBrowserData(overrides: Record<string, unknown> = {}) {
  return {
    run: run(),
    readerItems: [readerItem({ id: "snapshot-1", ref: "snapshot:1", content: "Frozen snapshot row" })],
    selectedSourceId: null,
    sourceOptions: [
      { id: 1, label: "Research channel", count: 1 },
      { id: 2, label: "Research video", count: 2 },
    ],
    loading: false,
    hasMore: true,
    availability: "available" as const,
    error: "",
    selectedTraceRef: null,
    onLoadMore: vi.fn(),
    ...overrides,
  };
}

function renderBrowser(subject: SourceBrowserSubject, overrides: Record<string, unknown> = {}) {
  return render(SourceBrowserShell, {
    props: {
      subject,
      sourceBrowserData: subject.kind === "source" ? sourceBrowserData() : null,
      groupBrowserData: subject.kind === "source_group" ? groupBrowserData() : null,
      snapshotBrowserData: subject.kind === "run_snapshot" ? snapshotBrowserData() : null,
      formatTimestamp,
      ...overrides,
    },
  });
}

const telegramSubject = (): SourceBrowserSubject => ({ kind: "source", source: source() });
const youtubeVideoSubject = (): SourceBrowserSubject => ({
  kind: "source",
  source: source({ sourceType: "youtube", sourceSubtype: "video", title: "Research video" }),
});
const youtubePlaylistSubject = (): SourceBrowserSubject => ({
  kind: "source",
  source: source({ sourceType: "youtube", sourceSubtype: "playlist", title: "Research playlist" }),
});
const groupSubject = (): SourceBrowserSubject => ({ kind: "source_group", group: group() });
function snapshotSubject(readerKind: "source_group" | "telegram_timeline" | "youtube_transcript" | "generic_items" = "source_group"): SourceBrowserSubject {
  return {
    kind: "run_snapshot",
    snapshot: {
      runId: 30,
      scopeType: "project",
      scopeLabel: "Project sources",
      readerKind,
      sourceType: readerKind === "telegram_timeline" ? "telegram" : readerKind === "youtube_transcript" ? "youtube" : null,
      sourceSubtype: readerKind === "telegram_timeline" ? "supergroup" : readerKind === "youtube_transcript" ? "video" : null,
    },
  };
}

describe("analysis source readers", () => {
  it("routes live browsable sources and source groups through SourceBrowserShell", async () => {
    const live = renderBrowser(telegramSubject());
    expect(await screen.findByRole("navigation", { name: "Source browser tabs" })).toBeTruthy();
    expect(screen.getByRole("region", { name: "Telegram source timeline" })).toBeTruthy();
    live.unmount();

    renderBrowser(groupSubject());
    expect((await screen.findByRole("button", { name: "Sources" })).getAttribute("aria-selected")).toBe("true");
    expect(screen.getByText("Research channel")).toBeTruthy();
    expect(screen.getByText("Research video")).toBeTruthy();
  });

  it("routes available run snapshots through SourceBrowserShell while keeping the header route-owned", async () => {
    renderBrowser(snapshotSubject());
    expect((await screen.findByRole("button", { name: "Sources" })).getAttribute("aria-selected")).toBe("true");
    expect(screen.getByText("Frozen snapshot row")).toBeTruthy();
    expect(screen.queryByRole("banner")).toBeNull();
  });

  it("handles project-scoped run snapshots as grouped source material", async () => {
    renderBrowser(snapshotSubject());
    expect(await screen.findByRole("button", { name: "Sources" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Items" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Metadata" })).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Activity" })).toBeNull();
  });

  it("keeps snapshot shell data frozen-only and live props empty", async () => {
    renderBrowser(snapshotSubject("generic_items"));
    expect(await screen.findByText("Frozen snapshot row")).toBeTruthy();
    expect(screen.queryByText("A loaded source item")).toBeNull();
    expect(screen.queryByRole("button", { name: "Sync source" })).toBeNull();
  });

  it("renders YouTube playlist videos through SourceBrowserShell", async () => {
    renderBrowser(youtubePlaylistSubject());
    expect(await screen.findByRole("region", { name: "YouTube playlist videos" })).toBeTruthy();
    expect(screen.getByText("YouTube playlist detail is not loaded.")).toBeTruthy();
  });

  it("keeps SourceBrowserShell mounted across supported live source switches", async () => {
    const view = renderBrowser(telegramSubject());
    const shell = await screen.findByRole("navigation", { name: "Source browser tabs" });

    await view.rerender({
      subject: youtubeVideoSubject(),
      sourceBrowserData: sourceBrowserData(),
      groupBrowserData: null,
      snapshotBrowserData: null,
      formatTimestamp,
    });

    await waitFor(() => expect(screen.getByRole("button", { name: "Transcript" }).getAttribute("aria-selected")).toBe("true"));
    expect(shell.isConnected).toBe(true);
    expect(screen.getByRole("region", { name: "YouTube transcript reader" })).toBeTruthy();
  });

  it("preserves the existing Telegram timeline controls through the shell", async () => {
    const onLoadMoreSourceItems = vi.fn();
    renderBrowser(telegramSubject(), { sourceBrowserData: sourceBrowserData({ onLoadMoreSourceItems }) });

    await fireEvent.click(await screen.findByRole("button", { name: "Load older messages" }));
    expect(onLoadMoreSourceItems).toHaveBeenCalledOnce();
  });

  it("keeps live source and run snapshot basis visible", () => {
    const live = render(SourceReaderHeader, {
      props: {
        title: "Research channel",
        subtitle: "Current source material",
        sourceViewBasis: "live_source",
        canViewLiveSource: false,
        canBackToRunSnapshot: true,
        selectedSourceId: 1,
        sourceOptions: [],
        onViewLiveSource: vi.fn(),
        onBackToRunSnapshot: vi.fn(),
        onChangeSelectedSourceId: vi.fn(),
      },
    });
    expect(screen.getByText("Live source")).toBeTruthy();
    live.unmount();

    render(SourceReaderHeader, {
      props: {
        title: "Run snapshot",
        subtitle: "Frozen source material",
        sourceViewBasis: "run_snapshot",
        sourceBasisState: "run_snapshot_available",
        canViewLiveSource: true,
        canBackToRunSnapshot: false,
        selectedSourceId: null,
        sourceOptions: [],
        onViewLiveSource: vi.fn(),
        onBackToRunSnapshot: vi.fn(),
        onChangeSelectedSourceId: vi.fn(),
      },
    });
    expect(screen.getByText("Run snapshot")).toBeTruthy();
  });

  it("uses a compact source reader heading instead of repeating the selected title", () => {
    render(SourceReaderHeader, {
      props: {
        compact: true,
        title: "Selected source title",
        subtitle: "3 loaded rows",
        sourceViewBasis: "live_source",
        canViewLiveSource: false,
        canBackToRunSnapshot: false,
        selectedSourceId: null,
        sourceOptions: [],
        onViewLiveSource: vi.fn(),
        onBackToRunSnapshot: vi.fn(),
        onChangeSelectedSourceId: vi.fn(),
      },
    });

    expect(screen.getByText("Source material")).toBeTruthy();
    expect(screen.getByText("3 loaded rows")).toBeTruthy();
    expect(screen.queryByText("Selected source title")).toBeNull();
  });

  it("renders Telegram as a metadata-rich timeline without binary previews", async () => {
    renderBrowser(telegramSubject(), {
      sourceBrowserData: sourceBrowserData({
        liveReaderItems: [readerItem({
          topicLabel: "Research topic",
          replyLabel: "Reply to #42",
          reactionLabel: "3 reactions",
          mediaCards: [{ kind: "document", title: "research.pdf", summary: "PDF document", fileName: "research.pdf", mimeType: "application/pdf" }],
        })],
      }),
    });

    expect(await screen.findByText("Ada")).toBeTruthy();
    expect(screen.getByText("time:1700000000")).toBeTruthy();
    expect(screen.getByText("Research topic")).toBeTruthy();
    expect(screen.getByText("Reply to #42")).toBeTruthy();
    expect(screen.getByText("3 reactions")).toBeTruthy();
    expect(screen.getAllByText("research.pdf")).toHaveLength(2);
    expect(screen.queryByRole("img")).toBeNull();
  });

  it("surfaces migrated Telegram history labels and scope controls", async () => {
    const onChangeTelegramHistoryScope = vi.fn();
    renderBrowser({
      kind: "source",
      source: source({ migratedHistoryStatus: "available", migratedHistoryRowCount: 4, migratedHistoryImportCompleted: true }),
    }, {
      sourceBrowserData: sourceBrowserData({
        telegramHistoryScope: "merged",
        onChangeTelegramHistoryScope,
        liveReaderItems: [readerItem({
          historyScope: "migrated",
          historyScopeLabel: "Migrated small-group history",
          isMigratedHistory: true,
        })],
      }),
    });

    const scope = await screen.findByLabelText("History scope");
    expect(screen.getByRole("option", { name: "Current supergroup history" })).toBeTruthy();
    expect(screen.getByRole("option", { name: "Migrated small-group history" })).toBeTruthy();
    expect(screen.getByRole("option", { name: "Merged timeline" })).toBeTruthy();
    expect(screen.getByText("Migrated small-group history", { selector: "span" })).toBeTruthy();
    await fireEvent.change(scope, { target: { value: "migrated" } });
    expect(onChangeTelegramHistoryScope).toHaveBeenCalledWith("migrated");
  });

  it("shows migrated Telegram history availability before imported rows are browsable", async () => {
    renderBrowser({
      kind: "source",
      source: source({ migratedHistoryStatus: "available", migratedHistoryRowCount: 0, migratedHistoryImportCompleted: false }),
    });

    expect(await screen.findByText("Migrated small-group history is detected but has not been imported for browsing yet.")).toBeTruthy();
    expect(screen.getByText("A browsable source row")).toBeTruthy();
  });

  it("renders Telegram topic filtering only in live single-source mode", async () => {
    const live = renderBrowser(telegramSubject(), {
      sourceBrowserData: sourceBrowserData({
        showTopicSelector: true,
        sourceTopics: [{
          kind: "topic",
          key: "topic:7",
          title: "Methods",
          messageCount: 9,
          topicId: 7,
          topMessageId: 70,
          iconColor: null,
          iconEmojiId: null,
          isClosed: false,
          isPinned: false,
          isHidden: false,
          isDeleted: false,
          sortOrder: 1,
        }],
      }),
    });
    expect(await screen.findByLabelText("Topic view")).toBeTruthy();
    expect(screen.getByRole("option", { name: "Methods (9)" })).toBeTruthy();
    live.unmount();

    renderBrowser(snapshotSubject("telegram_timeline"));
    expect(await screen.findByRole("region", { name: "Run snapshot source material timeline" })).toBeTruthy();
    expect(screen.queryByLabelText("Topic view")).toBeNull();
  });

  it("uses the shared takeout recovery notice in the selected source surface", async () => {
    renderBrowser(telegramSubject(), {
      sourceBrowserData: sourceBrowserData({
        takeoutRecovery: {
          batch_id: 10,
          source_id: 1,
          history_scope: "partial_private_history",
          status: "failed",
          recovery_kind: "failed",
          completeness: "partial",
          item_inserted_count: 12,
          item_duplicate_count: 2,
          item_skipped_count: 1,
          item_observed_count: 15,
          warning_count: 1,
          warning_codes: ["only_my_messages_fallback"],
          terminal_error: "Telegram export stopped",
          started_at: 1_700_000_000,
          finished_at: 1_700_000_010,
          updated_at: 1_700_000_010,
        },
      }),
    });

    await fireEvent.click(await screen.findByRole("button", { name: "Activity" }));
    expect(screen.getByRole("region", { name: "Previous Takeout import failed" })).toBeTruthy();
    expect(screen.getByText("only_my_messages_fallback")).toBeTruthy();
  });

  it("keeps live single-source timeline readers pageable", async () => {
    const onLoadMoreSourceItems = vi.fn();
    renderBrowser(telegramSubject(), { sourceBrowserData: sourceBrowserData({ onLoadMoreSourceItems }) });

    const loadMore = await screen.findByRole("button", { name: "Load older messages" });
    await fireEvent.click(loadMore);
    expect(onLoadMoreSourceItems).toHaveBeenCalledOnce();
  });

  it("keeps sticky date labels below overlay source switching UI", async () => {
    renderBrowser(telegramSubject());
    const tabs = await screen.findByRole("navigation", { name: "Source browser tabs" });
    const day = screen.getByText("2023-11-14");
    expect(tabs.compareDocumentPosition(day) & Node.DOCUMENT_POSITION_FOLLOWING).not.toBe(0);
    expect(day.closest("section")?.getAttribute("aria-label")).toBe("2023-11-14");
  });

  it("allows Telegram message text to hyphenate long words", async () => {
    const longWord = "электрофотополупроводниковый".repeat(4);
    renderBrowser(telegramSubject(), {
      sourceBrowserData: sourceBrowserData({ liveReaderItems: [readerItem({ content: longWord })] }),
    });

    const message = await screen.findByText(longWord);
    expect(message.textContent).toBe(longWord);
    expect(message.getAttribute("lang")).toBe("ru");
  });

  it("renders YouTube videos as transcript-first source readers", async () => {
    renderBrowser(youtubeVideoSubject(), {
      sourceBrowserData: sourceBrowserData({
        youtubeVideoDetail: youtubeVideoDetail(),
        youtubeTranscriptSegments: [{
          id: 11,
          sourceId: 1,
          itemId: 101,
          segmentIndex: 0,
          startMs: 5_000,
          endMs: 8_000,
          text: "Transcript-first evidence",
          captionLanguage: "en",
          captionTrackKind: "manual",
          isAutoGenerated: false,
        }],
      }),
    });

    expect((await screen.findByRole("button", { name: "Transcript" })).getAttribute("aria-selected")).toBe("true");
    expect(screen.getByRole("region", { name: "YouTube transcript reader" })).toBeTruthy();
    expect(screen.getByText("Transcript-first evidence")).toBeTruthy();
  });

  it("keeps YouTube live sync actions out of readonly snapshot transcript readers", async () => {
    renderBrowser(snapshotSubject("youtube_transcript"), {
      snapshotBrowserData: snapshotBrowserData({
        readerItems: [readerItem({ kind: "youtube_transcript", content: "Frozen transcript evidence", youtubeStartSeconds: 5 })],
      }),
    });

    expect(await screen.findByText("Frozen transcript evidence")).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Sync metadata" })).toBeNull();
    expect(screen.queryByRole("button", { name: "Sync transcript" })).toBeNull();
    expect(screen.queryByRole("button", { name: "Sync comments" })).toBeNull();
  });

  it("keeps run snapshot YouTube readers detached from live video detail", async () => {
    renderBrowser(snapshotSubject("youtube_transcript"), {
      sourceBrowserData: sourceBrowserData({ youtubeVideoDetail: youtubeVideoDetail() }),
      snapshotBrowserData: snapshotBrowserData({
        readerItems: [readerItem({ kind: "youtube_transcript", content: "Snapshot-only transcript" })],
      }),
    });

    expect(await screen.findByText("Snapshot-only transcript")).toBeTruthy();
    expect(screen.queryByText("Rendered video title")).toBeNull();
    expect(screen.queryByText("A research video description")).toBeNull();
  });

  it("keeps live YouTube video comments sync status and CTAs in transcript reader", async () => {
    const onSyncYoutubeComments = vi.fn();
    renderBrowser(youtubeVideoSubject(), {
      sourceBrowserData: sourceBrowserData({
        youtubeVideoDetail: youtubeVideoDetail(),
        onSyncYoutubeComments,
      }),
    });

    expect((await screen.findAllByText("Synced")).length).toBeGreaterThanOrEqual(2);
    await fireEvent.click(screen.getByRole("button", { name: "Sync comments" }));
    expect(onSyncYoutubeComments).toHaveBeenCalledWith(1);
  });

  it("renders YouTube comments as a loaded-window browser", async () => {
    const onLoadMoreSourceItems = vi.fn();
    renderBrowser(youtubeVideoSubject(), {
      sourceBrowserData: sourceBrowserData({
        youtubeVideoDetail: youtubeVideoDetail(),
        onLoadMoreSourceItems,
        sourceItems: [sourceItem({
          itemKind: "youtube_comment",
          content: "A loaded audience comment",
          youtubeComment: {
            commentId: "comment-1",
            parentCommentId: null,
            isReply: false,
            likeCount: 7,
            isPinned: true,
            isHearted: false,
            authorChannelUrl: null,
          },
        })],
      }),
    });

    await fireEvent.click(await screen.findByRole("button", { name: "Comments" }));
    expect(screen.getByRole("region", { name: "YouTube comments" })).toBeTruthy();
    expect(screen.getByText("A loaded audience comment")).toBeTruthy();
    expect(screen.getByText("7 likes")).toBeTruthy();
    await fireEvent.click(screen.getByRole("button", { name: "Load more comments" }));
    expect(onLoadMoreSourceItems).toHaveBeenCalledOnce();
  });

  it("passes playlist detail into metadata and playlist-specific empty copy into Items", async () => {
    renderBrowser(youtubePlaylistSubject(), {
      sourceBrowserData: sourceBrowserData({
        sourceItems: [],
        sourceItemsHasMore: false,
        youtubePlaylistDetail: youtubePlaylistDetail(),
      }),
    });

    await fireEvent.click(await screen.findByRole("button", { name: "Metadata" }));
    expect(screen.getAllByText("Rendered playlist title")).toHaveLength(2);
    await fireEvent.click(screen.getByRole("button", { name: "Items" }));
    expect(screen.getByText("Playlist videos live in the Videos tab. This Items tab only shows generic archived items loaded for this playlist source.")).toBeTruthy();
  });

  it("passes live YouTube video comments and jobs only into live transcript readers", async () => {
    const live = renderBrowser(youtubeVideoSubject(), {
      sourceBrowserData: sourceBrowserData({ youtubeVideoDetail: youtubeVideoDetail() }),
    });
    expect(await screen.findByRole("button", { name: "Sync comments" })).toBeTruthy();
    live.unmount();

    renderBrowser(snapshotSubject("youtube_transcript"));
    expect(await screen.findByRole("region", { name: "YouTube transcript reader" })).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Sync comments" })).toBeNull();
    expect(screen.queryByRole("button", { name: "Activity" })).toBeNull();
  });

  it("renders YouTube source job activity with progress warnings errors and cancel", async () => {
    const onCancelSourceJob = vi.fn();
    renderBrowser(youtubeVideoSubject(), {
      sourceBrowserData: sourceBrowserData({
        youtubeVideoDetail: youtubeVideoDetail(),
        onCancelSourceJob,
        sourceJobs: [{
          job_id: "job-1",
          source_id: 1,
          related_source_id: null,
          job_type: "youtube_video_full_sync",
          status: "running",
          message: "Syncing provider data",
          progress_current: 2,
          progress_total: 5,
          started_at: 1_700_000_000,
          finished_at: null,
          warnings: ["Captions are delayed"],
          error: "Provider retry scheduled",
        }],
      }),
    });

    await fireEvent.click(await screen.findByRole("button", { name: "Activity" }));
    expect(screen.getByText("Progress 2/5")).toBeTruthy();
    expect(screen.getByText("Captions are delayed")).toBeTruthy();
    expect(screen.getByText("Error Provider retry scheduled")).toBeTruthy();
    await fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    expect(onCancelSourceJob).toHaveBeenCalledWith("job-1");
  });

  it("surfaces YouTube runtime diagnostics in the live source canvas", async () => {
    renderBrowser(youtubeVideoSubject(), {
      sourceBrowserData: sourceBrowserData({
        youtubeDetailError: { sourceId: 1, message: "yt-dlp runtime is unavailable" },
      }),
    });

    await fireEvent.click(await screen.findByRole("button", { name: "Activity" }));
    expect(screen.getByText("yt-dlp runtime is unavailable")).toBeTruthy();
    expect(screen.getByText("attention")).toBeTruthy();
  });

  it("renders transcript search as one compact input shell", async () => {
    const onChangeTranscriptSearch = vi.fn();
    renderBrowser(youtubeVideoSubject(), {
      sourceBrowserData: sourceBrowserData({ onChangeTranscriptSearch }),
    });

    const inputs = await screen.findAllByRole("searchbox", { name: "Search transcript" });
    expect(inputs).toHaveLength(1);
    await fireEvent.input(inputs[0], { target: { value: "evidence" } });
    expect(onChangeTranscriptSearch).toHaveBeenCalledWith("evidence");
  });

  it("renders YouTube playlist videos as a job-free leaf view", async () => {
    renderBrowser(youtubePlaylistSubject(), {
      sourceBrowserData: sourceBrowserData({
        youtubePlaylistDetail: youtubePlaylistDetail(),
        sourceJobs: [{ job_id: "hidden-job", source_id: 1 }],
      }),
    });

    const videos = await screen.findByRole("region", { name: "YouTube playlist videos" });
    expect(videos.textContent).toContain("Playlist child video");
    expect(videos.textContent).not.toContain("Detailed jobs");
    expect(videos.textContent).not.toContain("hidden-job");
  });

  it("keeps playlist video opening as source selection instead of nested browsing", async () => {
    const onOpenSource = vi.fn();
    renderBrowser(youtubePlaylistSubject(), {
      sourceBrowserData: sourceBrowserData({
        youtubePlaylistDetail: youtubePlaylistDetail(),
        onOpenSource,
      }),
    });

    await fireEvent.click(await screen.findByRole("button", { name: "Open video source" }));
    expect(onOpenSource).toHaveBeenCalledWith(2);
    expect(screen.getByRole("region", { name: "YouTube playlist videos" })).toBeTruthy();
  });

  it("moves detailed source job cards into the Activity tab", async () => {
    renderBrowser(telegramSubject(), {
      sourceBrowserData: sourceBrowserData({
        sourceJobs: [{
          job_id: "job-activity",
          source_id: 1,
          related_source_id: null,
          job_type: "youtube_video_metadata_sync",
          status: "succeeded",
          message: "Metadata refreshed",
          progress_current: 1,
          progress_total: 1,
          started_at: 1_700_000_000,
          finished_at: 1_700_000_010,
          warnings: [],
          error: null,
        }],
      }),
    });

    expect(await screen.findByRole("region", { name: "Telegram source timeline" })).toBeTruthy();
    expect(screen.queryByText("Metadata refreshed")).toBeNull();
    await fireEvent.click(screen.getByRole("button", { name: "Activity" }));
    expect(screen.getByRole("region", { name: "Detailed source jobs" })).toBeTruthy();
    expect(screen.getByText("Metadata refreshed")).toBeTruthy();
  });

  it("keeps provider tabs to contextual CTAs instead of detailed job cards", async () => {
    renderBrowser(youtubeVideoSubject(), {
      sourceBrowserData: sourceBrowserData({
        sourceJobs: [{ job_id: "provider-job", message: "Hidden from transcript" }],
      }),
    });

    const transcript = await screen.findByRole("region", { name: "YouTube transcript reader" });
    expect(screen.getByRole("button", { name: "Sync transcript" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Sync metadata" })).toBeTruthy();
    expect(transcript.textContent).not.toContain("Hidden from transcript");
    expect(screen.queryByRole("region", { name: "Detailed source jobs" })).toBeNull();
  });

  it("covers Telegram source activity without adding backend job APIs", async () => {
    const onStartMigratedHistoryImport = vi.fn();
    renderBrowser({
      kind: "source",
      source: source({ migratedHistoryStatus: "available", migratedHistoryRowCount: 3 }),
    }, {
      sourceBrowserData: sourceBrowserData({ onStartMigratedHistoryImport }),
    });

    await fireEvent.click(await screen.findByRole("button", { name: "Activity" }));
    expect(screen.getByRole("region", { name: "Migrated history" })).toBeTruthy();
    expect(screen.getByText(/3 imported migrated rows/)).toBeTruthy();
    await fireEvent.click(screen.getByRole("button", { name: "Start migrated history import" }));
    expect(onStartMigratedHistoryImport).toHaveBeenCalledWith(1);
  });

  it("renders universal Items as a loaded-window browser", async () => {
    const onLoadMoreSourceItems = vi.fn();
    renderBrowser(telegramSubject(), {
      sourceBrowserData: sourceBrowserData({
        onLoadMoreSourceItems,
        sourceItems: [
          sourceItem({ id: 1, content: "Alpha evidence", publishedAt: 10 }),
          sourceItem({ id: 2, content: "Beta evidence", publishedAt: 20 }),
        ],
      }),
    });

    await fireEvent.click(await screen.findByRole("button", { name: "Items" }));
    expect(screen.getByRole("region", { name: "Universal source items" })).toBeTruthy();
    await fireEvent.input(screen.getByRole("searchbox", { name: "Search loaded items" }), { target: { value: "Beta" } });
    expect(screen.getByText("Beta evidence")).toBeTruthy();
    expect(screen.queryByText("Alpha evidence")).toBeNull();
    await fireEvent.click(screen.getByRole("button", { name: "Load more items" }));
    expect(onLoadMoreSourceItems).toHaveBeenCalledOnce();
  });

  it("renders snapshot Items as a frozen SourceReaderItem browser", async () => {
    const onLoadMore = vi.fn();
    renderBrowser(snapshotSubject("generic_items"), {
      snapshotBrowserData: snapshotBrowserData({ onLoadMore }),
    });

    expect(await screen.findByRole("region", { name: "Run snapshot items" })).toBeTruthy();
    expect(screen.getByText("Frozen snapshot row")).toBeTruthy();
    expect(screen.getByText("snapshot:1")).toBeTruthy();
    await fireEvent.click(screen.getByRole("button", { name: "Load older snapshot messages" }));
    expect(onLoadMore).toHaveBeenCalledOnce();
  });

  it("renders snapshot group Sources with global snapshot paging only", async () => {
    const onLoadMore = vi.fn();
    renderBrowser(snapshotSubject("source_group"), {
      snapshotBrowserData: snapshotBrowserData({ onLoadMore, hasMore: true }),
    });

    expect(await screen.findByRole("region", { name: "Run snapshot group sources" })).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Load older messages" })).toBeNull();
    await fireEvent.click(screen.getByRole("button", { name: "Load older snapshot messages" }));
    expect(onLoadMore).toHaveBeenCalledOnce();
  });

  it("renders run snapshot metadata from route-owned fields", async () => {
    renderBrowser(snapshotSubject("source_group"));
    await fireEvent.click(await screen.findByRole("button", { name: "Metadata" }));

    expect(screen.getByRole("region", { name: "Run snapshot metadata" })).toBeTruthy();
    expect(screen.getAllByText("Project sources")).toHaveLength(2);
    expect(screen.getByText("source_group")).toBeTruthy();
    expect(screen.getByText("available")).toBeTruthy();
    expect(screen.getByText("Research channel")).toBeTruthy();
    expect(screen.getByText("Research video")).toBeTruthy();
  });

  it("renders source metadata in structured sections with bounded raw JSON", async () => {
    renderBrowser(youtubeVideoSubject(), {
      sourceBrowserData: sourceBrowserData({ youtubeVideoDetail: youtubeVideoDetail() }),
    });
    await fireEvent.click(await screen.findByRole("button", { name: "Metadata" }));

    expect(screen.getByRole("heading", { name: "Summary" })).toBeTruthy();
    expect(screen.getByRole("heading", { name: "Source state" })).toBeTruthy();
    expect(screen.getByRole("heading", { name: "Technical" })).toBeTruthy();
    expect(screen.queryByText(/"bounded"/)).toBeNull();
    await fireEvent.click(screen.getByRole("button", { name: "Show raw JSON" }));
    expect(screen.getByText(/"bounded": "metadata"/)).toBeTruthy();
    expect(screen.getByRole("button", { name: "Hide raw JSON" })).toBeTruthy();
  });

  it("renders grouped transcript rows as a continuous reading surface", async () => {
    renderBrowser(youtubeVideoSubject(), {
      sourceBrowserData: sourceBrowserData({
        youtubeVideoDetail: youtubeVideoDetail(),
        youtubeTranscriptSegments: [
          { id: 1, sourceId: 1, itemId: 1, segmentIndex: 0, startMs: 1_000, endMs: 2_000, text: "First sentence", captionLanguage: "en", captionTrackKind: "manual", isAutoGenerated: false },
          { id: 2, sourceId: 1, itemId: 1, segmentIndex: 1, startMs: 2_500, endMs: 3_500, text: "continues here.", captionLanguage: "en", captionTrackKind: "manual", isAutoGenerated: false },
        ],
      }),
    });

    expect(await screen.findByText("First sentence continues here.")).toBeTruthy();
    expect(screen.getAllByRole("listitem")).toHaveLength(1);
  });

  it("scrolls selected Telegram and YouTube source rows into view", async () => {
    const scrollIntoView = HTMLElement.prototype.scrollIntoView as ReturnType<typeof vi.fn>;
    scrollIntoView.mockClear();
    const telegram = renderBrowser(telegramSubject(), {
      sourceBrowserData: sourceBrowserData({ liveReaderItems: [readerItem({ selected: true })] }),
    });
    await waitFor(() => expect(scrollIntoView).toHaveBeenCalledOnce());
    telegram.unmount();

    renderBrowser(youtubeVideoSubject(), {
      sourceBrowserData: sourceBrowserData({
        youtubeTranscriptSegments: [{ id: 3, sourceId: 1, itemId: 1, segmentIndex: 0, startMs: 1_000, endMs: 2_000, text: "Selected transcript", captionLanguage: "en", captionTrackKind: "manual", isAutoGenerated: false }],
      }),
      selectedTraceRef: "s1-i1@1000ms",
    });
    await waitFor(() => expect(scrollIntoView).toHaveBeenCalledTimes(2));
  });

  it("adds one-shot evidence highlight support to trace-capable readers", async () => {
    const highlightToken = {
      tokenId: "highlight-1",
      runId: 30,
      sourceScope: { kind: "source" as const, sourceId: 1 },
      sourceViewBasis: "live_source" as const,
      traceRef: "source:1:item:1",
      createdAt: 1_700_000_000,
    };
    const view = renderBrowser(telegramSubject(), { highlightToken });

    const highlighted = await waitFor(() => document.querySelector('[data-trace-ref="source:1:item:1"]'));
    expect(highlighted?.getAttribute("data-evidence-highlighted")).toBe("true");
    await view.rerender({
      subject: telegramSubject(),
      sourceBrowserData: sourceBrowserData(),
      groupBrowserData: null,
      snapshotBrowserData: null,
      formatTimestamp,
      highlightToken,
    });
    expect(document.querySelectorAll('[data-evidence-highlighted="true"]')).toHaveLength(1);
  });

  it("matches evidence highlights by concrete trace refs without replacing selected row behavior", async () => {
    renderBrowser(telegramSubject(), {
      sourceBrowserData: sourceBrowserData({
        liveReaderItems: [
          readerItem({ id: "selected", ref: "source:1:item:selected", content: "Selected row", selected: true }),
          readerItem({ id: "highlighted", ref: "source:1:item:highlighted", content: "Highlighted row", selected: false }),
        ],
      }),
      highlightToken: {
        tokenId: "highlight-2",
        runId: 30,
        sourceScope: { kind: "source", sourceId: 1 },
        sourceViewBasis: "live_source",
        traceRef: "source:1:item:highlighted",
        createdAt: 1_700_000_000,
      },
    });

    const selected = (await screen.findByText("Selected row")).closest("li");
    const highlighted = screen.getByText("Highlighted row").closest("li");
    expect(selected?.classList.contains("selected")).toBe(true);
    expect(selected?.hasAttribute("data-evidence-highlighted")).toBe(false);
    expect(highlighted?.classList.contains("selected")).toBe(false);
    expect(highlighted?.getAttribute("data-evidence-highlighted")).toBe("true");
  });

  it("renders source group metadata from route-owned group fields", async () => {
    renderBrowser(groupSubject());
    await fireEvent.click(await screen.findByRole("button", { name: "Metadata" }));

    expect(screen.getByRole("region", { name: "Source group metadata" })).toBeTruthy();
    expect(screen.getAllByText("Mixed research group")).toHaveLength(2);
    expect(screen.getByText("3", { selector: "dd" })).toBeTruthy();
    expect(screen.getByText("time:1699000000")).toBeTruthy();
    expect(screen.getByText("time:1700000000")).toBeTruthy();
  });

  it("renders source group activity without source job cards", async () => {
    renderBrowser(groupSubject());
    await fireEvent.click(await screen.findByRole("button", { name: "Activity" }));

    const activity = screen.getByRole("region", { name: "Source group activity" });
    expect(activity.textContent).toContain("Group activity is not available yet. Source jobs are still tracked per source.");
    expect(screen.queryByRole("region", { name: "Detailed source jobs" })).toBeNull();
  });

  it("groups source group material by source", async () => {
    renderBrowser(groupSubject());

    expect(await screen.findByRole("region", { name: "Research channel" })).toBeTruthy();
    expect(screen.getByRole("region", { name: "Research video" })).toBeTruthy();
    expect(screen.getAllByText("1 loaded items")).toHaveLength(2);
  });

  it("merges focused group YouTube transcript DTOs into source-group reader items", async () => {
    renderBrowser(groupSubject(), {
      groupBrowserData: groupBrowserData({
        selectedSourceId: 2,
        liveReaderItems: [readerItem({
          id: "youtube-group-item",
          sourceId: 2,
          sourceTitle: "Research video",
          kind: "youtube_transcript",
          content: "Merged group transcript evidence",
          youtubeStartSeconds: 12,
          youtubeEndSeconds: 15,
        })],
      }),
    });

    expect(await screen.findByText("Merged group transcript evidence")).toBeTruthy();
    expect(screen.getByRole("region", { name: "Research video" })).toBeTruthy();
    expect(screen.queryByRole("region", { name: "Research channel" })).toBeNull();
  });

  it("uses a neutral timeline label for mixed source-group material", async () => {
    renderBrowser(groupSubject(), {
      groupBrowserData: groupBrowserData({ liveReaderItems: [readerItem()] }),
    });

    expect(await screen.findByRole("region", { name: "Source material timeline" })).toBeTruthy();
    expect(screen.queryByRole("region", { name: "Telegram source timeline" })).toBeNull();
  });

  it("builds live source-group focus options from every group member", async () => {
    const onChangeSelectedSourceId = vi.fn();
    render(SourceReaderHeader, {
      props: {
        title: "Mixed research group",
        subtitle: "3 loaded rows",
        sourceViewBasis: "live_source",
        canViewLiveSource: false,
        canBackToRunSnapshot: false,
        selectedSourceId: null,
        sourceOptions: [
          { id: 1, label: "Research channel", count: 1 },
          { id: 2, label: "Research video", count: 2 },
        ],
        onViewLiveSource: vi.fn(),
        onBackToRunSnapshot: vi.fn(),
        onChangeSelectedSourceId,
      },
    });

    const focus = screen.getByLabelText("Source focus");
    expect(screen.getByRole("option", { name: "Research channel (1)" })).toBeTruthy();
    expect(screen.getByRole("option", { name: "Research video (2)" })).toBeTruthy();
    await fireEvent.change(focus, { target: { value: "2" } });
    expect(onChangeSelectedSourceId).toHaveBeenCalledWith(2);
  });

  it("keeps run snapshot focus options based on the whole loaded snapshot page", () => {
    render(SourceReaderHeader, {
      props: {
        title: "Project sources",
        subtitle: "Frozen rows",
        sourceViewBasis: "run_snapshot",
        canViewLiveSource: true,
        canBackToRunSnapshot: false,
        selectedSourceId: 1,
        sourceOptions: snapshotBrowserData().sourceOptions,
        onViewLiveSource: vi.fn(),
        onBackToRunSnapshot: vi.fn(),
        onChangeSelectedSourceId: vi.fn(),
      },
    });

    expect(screen.getByRole("option", { name: "Research channel (1)" })).toBeTruthy();
    expect(screen.getByRole("option", { name: "Research video (2)" })).toBeTruthy();
    expect((screen.getByLabelText("Source focus") as HTMLSelectElement).value).toBe("1");
  });

  it("keeps source focus controls in one reader header location", () => {
    render(SourceReaderHeader, {
      props: {
        title: "Project sources",
        subtitle: "3 rows",
        sourceViewBasis: "run_snapshot",
        canViewLiveSource: true,
        canBackToRunSnapshot: false,
        selectedSourceId: null,
        sourceOptions: snapshotBrowserData().sourceOptions,
        onViewLiveSource: vi.fn(),
        onBackToRunSnapshot: vi.fn(),
        onChangeSelectedSourceId: vi.fn(),
      },
    });

    expect(screen.getAllByText("Source focus")).toHaveLength(1);
    expect(screen.getAllByRole("combobox")).toHaveLength(1);
    expect(screen.getAllByRole("banner")).toHaveLength(1);
  });
});
