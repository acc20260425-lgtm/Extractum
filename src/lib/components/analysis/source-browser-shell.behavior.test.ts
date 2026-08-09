import { afterAll, afterEach, beforeAll, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import SourceBrowserShell from "./source-browser-shell.svelte";
import type { SourceBrowserSubject } from "$lib/source-browser-model";

const originalScroll = Object.getOwnPropertyDescriptor(HTMLElement.prototype, "scrollIntoView");
const originalCss = Object.getOwnPropertyDescriptor(globalThis, "CSS");
beforeAll(() => {
  Object.defineProperty(HTMLElement.prototype, "scrollIntoView", { configurable: true, value: vi.fn() });
  vi.stubGlobal("CSS", { escape: (value: string) => value });
});
afterAll(() => {
  originalScroll ? Object.defineProperty(HTMLElement.prototype, "scrollIntoView", originalScroll) : delete (HTMLElement.prototype as { scrollIntoView?: unknown }).scrollIntoView;
  vi.unstubAllGlobals();
  if (originalCss) Object.defineProperty(globalThis, "CSS", originalCss);
});
afterEach(cleanup);

const formatTimestamp = (value: number | null) => value === null ? "Never" : `time:${value}`;
const source = (overrides: Record<string, unknown> = {}) => ({
  id: 1, sourceType: "telegram", sourceSubtype: "supergroup", accountId: 7,
  externalId: "source-1", title: "Research channel", lastSyncState: 1,
  lastSyncedAt: 1_700_000_000, isMember: true, isActive: true, createdAt: 1_699_000_000,
  telegramUsername: "research", avatarDataUrl: null, migratedHistoryStatus: "none",
  migratedHistoryDetectedAt: null, migratedHistoryRefreshedAt: null, migratedHistoryRowCount: 0,
  migratedHistoryImportCompleted: false, ...overrides,
});
const item = (overrides: Record<string, unknown> = {}) => ({
  id: 1, sourceId: 1, externalId: "message-1", itemKind: "telegram_message", author: "Ada",
  publishedAt: 1_700_000_000, content: "Loaded item", contentKind: "text", hasMedia: false,
  mediaKind: null, mediaSummary: null, mediaFileName: null, mediaMimeType: null, hasRawData: false,
  forumTopicId: null, forumTopicTitle: null, forumTopicTopMessageId: null, replyToMessageId: null,
  replyToPeerKind: null, replyToPeerId: null, replyToTopMessageId: null, reactionCount: null,
  historyScope: "current", isMigratedHistory: false, migrationDomain: null,
  historyScopeLabel: "Current history", pageCursor: "cursor-1", ...overrides,
});
const reader = (overrides: Record<string, unknown> = {}) => ({
  id: "reader-1", sourceId: 1, sourceTitle: "Research channel", externalId: "message-1",
  ref: "source:1:item:1", kind: "telegram_message", author: "Ada", publishedAt: 1_700_000_000,
  content: "Reader item", topicLabel: null, replyLabel: null, reactionLabel: null, mediaCards: [],
  youtubeStartSeconds: null, youtubeEndSeconds: null, youtubeUrl: null, captionLabel: null,
  historyScope: "current", historyScopeLabel: null, isMigratedHistory: false, selected: false, ...overrides,
});
const videoDetail = () => {
  const synced = { state: "synced", itemCount: 1, segmentCount: 1, lastSyncedAt: 1, label: "Synced" };
  return { summary: { sourceId: 1, sourceSubtype: "video", title: "Video title", channelTitle: "Channel", channelHandle: "@channel", canonicalUrl: "https://youtu.be/v", thumbnailUrl: null, durationSeconds: 60, publishedAt: 1, availabilityStatus: "available", videoCount: null, linkedVideoCount: null, unavailableCount: null, captions: synced, comments: synced }, sourceMetadata: { sourceId: 1, videoId: "v", canonicalUrl: "https://youtu.be/v", title: "Video title", channelTitle: "Channel", channelId: "c", channelHandle: "@channel", channelUrl: null, authorDisplay: "Channel", publishedAt: 1, durationSeconds: 60, description: "Description", thumbnailUrl: null, viewCount: 1, likeCount: 1, commentCount: 1, category: null, videoForm: "long", availabilityStatus: "available", captionLanguageOverride: null, rawMetadataVersion: 1, rawMetadataJson: {} }, playlistMemberships: [] };
};
const sourceData = (overrides: Record<string, unknown> = {}) => ({
  liveReaderItems: [reader()], sourceItems: [item()], sourceRouteError: null, sourceItemsHasMore: false,
  loadingItems: false, sourceTopics: [], loadingSourceTopics: false, selectedTopicKey: "__all_topics__",
  showTopicSelector: false, youtubeVideoDetail: null, youtubePlaylistDetail: null, youtubeDetailError: null,
  youtubeTranscriptSegments: [], youtubeTranscriptSearch: "", youtubeTranscriptHasMore: false,
  loadingYoutubeTranscriptSegments: false, loadingYoutubeDetail: false, sourceJobs: [], takeoutRecovery: null,
  sourceSyncDisabledReason: () => null, telegramHistoryScope: "current", currentSourceContentLabel: "messages",
  onLoadMoreSourceItems: vi.fn(), onChangeSelectedTopicKey: vi.fn(), onChangeTelegramHistoryScope: vi.fn(),
  onChangeTranscriptSearch: vi.fn(), onLoadMoreYoutubeTranscriptSegments: vi.fn(), onOpenSource: vi.fn(),
  onSyncSource: vi.fn(), onSyncYoutubeMetadata: vi.fn(), onSyncYoutubeTranscript: vi.fn(),
  onSyncYoutubeComments: vi.fn(), onSyncYoutubePlaylist: vi.fn(), onRetryFailedYoutubePlaylistVideos: vi.fn(),
  onSyncYoutubePlaylistVideo: vi.fn(), onRetryYoutubePlaylistVideo: vi.fn(), onStartTakeoutImport: vi.fn(),
  onStartMigratedHistoryImport: vi.fn(), onCancelSourceJob: vi.fn(), ...overrides,
});
const run = { id: 30, run_type: "report", scope_type: "project", source_id: null, source_title: null, source_group_id: null, source_group_name: null, project_id: 4, project_name: "Alpha", scope_label: "Project sources", period_from: 1, period_to: 2, output_language: "en", prompt_template_id: 1, prompt_template_name: "Report", prompt_template_version: 1, provider_profile: "Default", provider: "openai", model: "model", youtube_corpus_mode: "transcript_only", telegram_history_scope: "current", status: "completed", error: null, has_trace_data: true, snapshot_state: "captured", snapshot_captured_at: "2026-08-01", snapshot_error: null, created_at: 1, completed_at: 2, result_markdown: "# Result" };
const snapshotData = (overrides: Record<string, unknown> = {}) => ({ run, readerItems: [reader({ id: "snapshot", ref: "snapshot:1", content: "Frozen row" })], selectedSourceId: null, sourceOptions: [{ id: 1, label: "Research channel", count: 1 }], loading: false, hasMore: false, availability: "available", error: "", selectedTraceRef: null, onLoadMore: vi.fn(), ...overrides });
const live = (overrides: Record<string, unknown> = {}): SourceBrowserSubject => ({ kind: "source", source: source(overrides) } as never);
const snapshot = (readerKind: string): SourceBrowserSubject => ({ kind: "run_snapshot", snapshot: { runId: 30, scopeType: "project", scopeLabel: "Project sources", readerKind, sourceType: readerKind === "youtube_transcript" ? "youtube" : "telegram", sourceSubtype: readerKind === "youtube_transcript" ? "video" : "supergroup" } } as never);
const props = (subject: SourceBrowserSubject, overrides: Record<string, unknown> = {}) => ({ subject, sourceBrowserData: subject.kind === "source" ? sourceData() : null, groupBrowserData: null, snapshotBrowserData: subject.kind === "run_snapshot" ? snapshotData() : null, formatTimestamp, ...overrides });

describe("source browser shell", () => {
  it("renders provider readers and playlist videos", async () => {
    const telegram = render(SourceBrowserShell, { props: props(live()) as never });
    expect(await screen.findByRole("region", { name: "Telegram source timeline" })).toBeTruthy();
    expect(screen.getByText("Reader item")).toBeTruthy();
    telegram.unmount();
    const video = render(SourceBrowserShell, { props: props(live({ sourceType: "youtube", sourceSubtype: "video" }), { sourceBrowserData: sourceData({ youtubeVideoDetail: videoDetail(), youtubeDetailError: { sourceId: 1, sourceSubtype: "video", message: "Metadata unavailable" } }) }) as never });
    expect(await screen.findByRole("region", { name: "YouTube transcript reader" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Transcript" }).getAttribute("aria-selected")).toBe("true");
    await fireEvent.click(screen.getByRole("button", { name: "Activity" }));
    expect(screen.getByText("Metadata unavailable")).toBeTruthy();
    video.unmount();
    render(SourceBrowserShell, { props: props(live({ sourceType: "youtube", sourceSubtype: "playlist" })) as never });
    expect(await screen.findByRole("region", { name: "YouTube playlist videos" })).toBeTruthy();
    expect(screen.getByText("YouTube playlist detail is not loaded.")).toBeTruthy();
  });

  it("renders snapshot tabs without live activity controls", async () => {
    render(SourceBrowserShell, { props: props(snapshot("source_group")) as never });
    expect(await screen.findByRole("button", { name: "Sources" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Items" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Metadata" })).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Activity" })).toBeNull();
    expect(screen.queryByRole("button", { name: /Sync/ })).toBeNull();
    expect(screen.getByText("Frozen row")).toBeTruthy();
    expect(screen.queryByText("Loaded item")).toBeNull();
    expect(screen.queryByRole("banner")).toBeNull();
  });

  it("forwards evidence highlights to trace-capable readers", async () => {
    const token = { tokenId: "live-1", runId: 30, sourceScope: { kind: "source", sourceId: 1 }, sourceViewBasis: "live_source", traceRef: "source:1:item:1", createdAt: 1 };
    const view = render(SourceBrowserShell, { props: props(live(), { highlightToken: token }) as never });
    await waitFor(() => expect(document.querySelector('[data-trace-ref="source:1:item:1"]')).toBeTruthy());
    const row = document.querySelector('[data-trace-ref="source:1:item:1"]');
    expect(row?.getAttribute("data-evidence-highlighted")).toBe("true");
    expect(HTMLElement.prototype.scrollIntoView).toHaveBeenCalled();
    expect(screen.getByText("Reader item")).toBeTruthy();
    await view.rerender(props(live(), { highlightToken: { ...token, tokenId: "live-2" } }) as never);
    expect(document.querySelector('[data-trace-ref="source:1:item:1"]')?.getAttribute("data-evidence-highlighted")).toBe("true");
    view.unmount();
    render(SourceBrowserShell, { props: props(snapshot("generic_items"), { snapshotBrowserData: snapshotData({ readerItems: [reader({ ref: "snapshot:1" })] }), highlightToken: { ...token, tokenId: "snapshot", sourceViewBasis: "run_snapshot", traceRef: "snapshot:1" } }) as never });
    await waitFor(() => expect(document.querySelector('[data-trace-ref="snapshot:1"]')).toBeTruthy());
    expect(document.querySelector('[data-trace-ref="snapshot:1"]')?.getAttribute("data-evidence-highlighted")).toBe("true");
    expect(screen.getByText("Reader item")).toBeTruthy();
    expect(document.querySelectorAll('[data-evidence-highlighted="true"]')).toHaveLength(1);
    expect(document.querySelector('[data-trace-ref="snapshot:1"]')?.isConnected).toBe(true);
    expect(document.querySelector('[data-trace-ref="snapshot:1"]')?.textContent).toContain("Reader item");
    expect(screen.getByRole("navigation", { name: "Source browser tabs" })).toBeTruthy();
  });

  it("selects the source-item evidence tab once per token", async () => {
    const comment = item({ id: 9, itemKind: "youtube_comment", content: "Highlighted comment", youtubeComment: { author: "Ada" } });
    const token = { tokenId: "comment-1", runId: 30, sourceScope: { kind: "source", sourceId: 1 }, sourceViewBasis: "live_source", traceRef: "s1-i9", createdAt: 1 };
    const view = render(SourceBrowserShell, { props: props(live({ sourceType: "youtube", sourceSubtype: "video" }), { sourceBrowserData: sourceData({ sourceItems: [comment], youtubeVideoDetail: videoDetail() }), highlightToken: token }) as never });
    const comments = await screen.findByRole("button", { name: "Comments" });
    expect(comments.getAttribute("aria-selected")).toBe("true");
    expect(screen.getByText("Highlighted comment")).toBeTruthy();
    expect(document.querySelector('[data-trace-ref="s1-i9"]')).toBeTruthy();
    expect(document.querySelector('[data-trace-ref="s1-i9"]')?.getAttribute("data-evidence-highlighted")).toBe("true");
    expect(screen.getByRole("button", { name: "Transcript" }).getAttribute("aria-selected")).toBe("false");
    expect(screen.getByRole("button", { name: "Items" }).getAttribute("aria-selected")).toBe("false");
    expect(screen.getByRole("button", { name: "Activity" }).getAttribute("aria-selected")).toBe("false");
    await fireEvent.click(screen.getByRole("button", { name: "Items" }));
    expect(screen.getByRole("button", { name: "Items" }).getAttribute("aria-selected")).toBe("true");
    await view.rerender(props(live({ sourceType: "youtube", sourceSubtype: "video" }), { sourceBrowserData: sourceData({ sourceItems: [comment], youtubeVideoDetail: videoDetail() }), highlightToken: token }) as never);
    expect(screen.getByRole("button", { name: "Items" }).getAttribute("aria-selected")).toBe("true");
    expect(screen.getByRole("button", { name: "Comments" }).getAttribute("aria-selected")).toBe("false");
    await view.rerender(props(live({ sourceType: "youtube", sourceSubtype: "video" }), { sourceBrowserData: sourceData({ sourceItems: [comment], youtubeVideoDetail: videoDetail() }), highlightToken: { ...token, tokenId: "comment-2" } }) as never);
    expect(screen.getByRole("button", { name: "Comments" }).getAttribute("aria-selected")).toBe("true");
    expect(screen.getByText("Highlighted comment")).toBeTruthy();
    expect(document.querySelectorAll('[data-evidence-highlighted="true"]')).toHaveLength(1);
    expect(document.querySelector('[data-trace-ref="s1-i9"]')?.getAttribute("data-evidence-highlighted")).toBe("true");
    expect(HTMLElement.prototype.scrollIntoView).toHaveBeenCalled();
    expect(view.container.querySelector('[aria-label="Source browser tabs"]')).toBeTruthy();
    expect(screen.getAllByRole("button", { name: "Comments" })).toHaveLength(1);
  });

  it("selects the snapshot evidence tab once per token", async () => {
    const token = { tokenId: "snapshot-1", runId: 30, sourceScope: { kind: "source", sourceId: 1 }, sourceViewBasis: "run_snapshot", traceRef: "snapshot:1", createdAt: 1 };
    const view = render(SourceBrowserShell, { props: props(snapshot("generic_items"), { highlightToken: token }) as never });
    expect((await screen.findByRole("button", { name: "Items" })).getAttribute("aria-selected")).toBe("true");
    expect(screen.getByText("Frozen row")).toBeTruthy();
    expect(document.querySelector('[data-trace-ref="snapshot:1"]')?.getAttribute("data-evidence-highlighted")).toBe("true");
    expect(screen.queryByRole("button", { name: "Activity" })).toBeNull();
    await fireEvent.click(screen.getByRole("button", { name: "Metadata" }));
    expect(screen.getByRole("button", { name: "Metadata" }).getAttribute("aria-selected")).toBe("true");
    await view.rerender(props(snapshot("generic_items"), { highlightToken: token }) as never);
    expect(screen.getByRole("button", { name: "Metadata" }).getAttribute("aria-selected")).toBe("true");
    await view.rerender(props(snapshot("generic_items"), { highlightToken: { ...token, tokenId: "snapshot-2" } }) as never);
    expect(screen.getByRole("button", { name: "Items" }).getAttribute("aria-selected")).toBe("true");
    expect(document.querySelectorAll('[data-evidence-highlighted="true"]')).toHaveLength(1);
    expect(HTMLElement.prototype.scrollIntoView).toHaveBeenCalled();
  });
});
