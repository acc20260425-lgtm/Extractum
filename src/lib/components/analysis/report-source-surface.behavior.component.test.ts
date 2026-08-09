import { cleanup, fireEvent, render, screen, within } from "@testing-library/svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { ComponentProps } from "svelte";
import type { Source, SourceItem } from "$lib/types/sources";
import ReportSourceSurface from "./report-source-surface.svelte";

afterEach(cleanup);

type Props = ComponentProps<typeof ReportSourceSurface>;

function source(overrides: Partial<Source> = {}): Source {
  return {
    id: 7, sourceType: "youtube", sourceSubtype: "video", accountId: null,
    externalId: "video-7", title: "Research video", lastSyncState: 4,
    lastSyncedAt: 1_700_000_000, isMember: true, isActive: true, createdAt: 1,
    telegramUsername: null, avatarDataUrl: null, migratedHistoryStatus: "none",
    migratedHistoryDetectedAt: null, migratedHistoryRefreshedAt: null,
    migratedHistoryRowCount: 0, migratedHistoryImportCompleted: false, ...overrides,
  };
}

function item(id: number, sourceId: number, content: string): SourceItem {
  return {
    id, sourceId, externalId: `item-${id}`, itemKind: "telegram_message", author: `Author ${id}`,
    publishedAt: 1_700_000_000 + id, content, contentKind: "text", hasMedia: false,
    mediaKind: null, mediaSummary: null, mediaFileName: null, mediaMimeType: null, hasRawData: false,
    forumTopicId: null, forumTopicTitle: null, forumTopicTopMessageId: null, replyToMessageId: null,
    replyToPeerKind: null, replyToPeerId: null, replyToTopMessageId: null, reactionCount: null,
    historyScope: "current", isMigratedHistory: false, migrationDomain: null,
    historyScopeLabel: "Current history", pageCursor: `cursor-${id}`,
  };
}

function props(overrides: Partial<Props> = {}): Props {
  return {
    sourceHeaderCompact: true, sourceBrowserBounded: true, currentRun: null,
    sourceViewBasis: "live_source", snapshotAvailability: "unknown", snapshotProbeState: "unknown",
    runSnapshotMessages: [], loadingRunSnapshotMessages: false, runSnapshotError: "",
    hasMoreRunSnapshotMessages: false, workspaceSelection: { kind: "source", sourceId: 7 },
    currentSource: source(), takeoutRecovery: null, currentGroup: null, currentSourceMetric: null,
    sourceItems: [], sourceItemsError: null, sourceItemsHasMore: false, loadingItems: false,
    sourceTopics: [], loadingSourceTopics: false, selectedTopicKey: "__all_topics__",
    showTopicSelector: false, currentSourceContentLabel: "transcript segments", telegramHistoryScope: "current",
    sourceJobs: [], youtubeVideoDetail: null, youtubePlaylistDetail: null, youtubeDetailError: null,
    loadingYoutubeDetail: false, selectedTraceRef: null, highlightToken: null, sourceReturnContext: null,
    currentScopeTitle: "Research video", youtubeTranscriptSegments: [], loadingYoutubeTranscriptSegments: false,
    youtubeTranscriptHasMore: false, youtubeTranscriptSearch: "", groupLiveItemsBySource: {},
    groupLiveTranscriptSegmentsBySource: {}, groupLiveHasMoreBySource: {}, selectedGroupSourceId: null,
    selectedSnapshotSourceId: null, formatTimestamp: (value) => `time:${value}`,
    onChangeSelectedTopicKey: vi.fn(), onOpenSource: vi.fn(), onSyncSource: vi.fn(),
    onSyncYoutubeMetadata: vi.fn(), onSyncYoutubeTranscript: vi.fn(), onSyncYoutubeComments: vi.fn(),
    onStartTakeoutImport: vi.fn(), onStartMigratedHistoryImport: vi.fn(), onSyncYoutubePlaylist: vi.fn(),
    onRetryFailedYoutubePlaylistVideos: vi.fn(), onSyncYoutubePlaylistVideo: vi.fn(),
    onRetryYoutubePlaylistVideo: vi.fn(), onCancelSourceJob: vi.fn(), onViewLiveSource: vi.fn(),
    onBackToRunSnapshot: vi.fn(), onReturnToEvidenceReview: vi.fn(), sourceSyncDisabledReason: () => null,
    onLoadMoreRunSnapshotMessages: vi.fn(), onLoadMoreSourceItems: vi.fn(),
    onChangeTelegramHistoryScope: vi.fn(), onChangeTranscriptSearch: vi.fn(),
    onLoadMoreYoutubeTranscriptSegments: vi.fn(), onLoadLiveGroupSourcePage: vi.fn(),
    onChangeSelectedGroupSourceId: vi.fn(), onChangeSelectedSnapshotSourceId: vi.fn(), ...overrides,
  };
}

describe("analysis priority UX contract", () => {
  it("keeps the report canvas top chrome compact and action-oriented", async () => {
    const onChangeTranscriptSearch = vi.fn();
    const onSyncYoutubeTranscript = vi.fn();
    const onSyncYoutubeComments = vi.fn();
    render(ReportSourceSurface, { props: props({ onChangeTranscriptSearch, onSyncYoutubeTranscript, onSyncYoutubeComments }) });

    expect(screen.getByRole("banner", { name: "Research video" })).toBeTruthy();
    expect(screen.getByText("Source material")).toBeTruthy();
    expect(screen.getByText("Live source")).toBeTruthy();
    expect(screen.queryByText("Research video", { selector: "h2" })).toBeNull();
    expect(screen.getByRole("navigation", { name: "Source browser tabs" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Transcript" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Comments" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Metadata" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Activity" })).toBeTruthy();
    expect(screen.getByRole("searchbox", { name: "Search transcript" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Sync transcript" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Sync comments" })).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Back to run snapshot" })).toBeNull();
    await fireEvent.input(screen.getByRole("searchbox", { name: "Search transcript" }), { target: { value: "evidence" } });
    expect(onChangeTranscriptSearch).toHaveBeenCalledWith("evidence");
    await fireEvent.click(screen.getByRole("button", { name: "Sync transcript" }));
    expect(onSyncYoutubeTranscript).toHaveBeenCalledWith(7);
    await fireEvent.click(screen.getByRole("button", { name: "Sync comments" }));
    expect(onSyncYoutubeComments).toHaveBeenCalledWith(7);
    await fireEvent.click(screen.getByRole("button", { name: "Metadata" }));
    expect(screen.getByRole("region", { name: "Source metadata" })).toBeTruthy();
    expect(screen.getByText("youtube / video")).toBeTruthy();
    expect(screen.getByText("video-7")).toBeTruthy();
  });
});

describe("analysis redesign final safety contract", () => {
  it("keeps source ingest activity out of analysis Runs", async () => {
    const onSyncYoutubeMetadata = vi.fn();
    const onCancelSourceJob = vi.fn();
    render(ReportSourceSurface, {
      props: props({
        sourceJobs: [{ job_id: "job-1", source_id: 7, related_source_id: null, job_type: "youtube_video_comments_sync",
          status: "running", message: "Syncing live comments", progress_current: 1, progress_total: 2,
          started_at: 1, finished_at: null, warnings: [], error: null }],
        onSyncYoutubeMetadata, onCancelSourceJob,
      }),
    });

    expect(screen.getByRole("navigation", { name: "Source browser tabs" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Transcript" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Comments" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Metadata" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Activity" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Sync metadata" })).toBeTruthy();
    await fireEvent.click(screen.getByRole("button", { name: "Sync metadata" }));
    expect(onSyncYoutubeMetadata).toHaveBeenCalledWith(7);
    await fireEvent.click(screen.getByRole("button", { name: "Activity" }));
    expect(screen.getByText("Detailed jobs")).toBeTruthy();
    expect(screen.getByText("Syncing live comments")).toBeTruthy();
    expect(screen.getByText("Progress 1/2")).toBeTruthy();
    expect(screen.getByRole("button", { name: "Cancel" })).toBeTruthy();
    expect(screen.queryByText("Analysis report runs")).toBeNull();
    expect(screen.queryByRole("searchbox", { name: "Search runs" })).toBeNull();
    await fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    expect(onCancelSourceJob).toHaveBeenCalledWith("job-1");
  });

  it("keeps source groups grouped by source instead of merged into one pseudo-chat", async () => {
    const onChangeSelectedGroupSourceId = vi.fn();
    const onLoadLiveGroupSourcePage = vi.fn();
    render(ReportSourceSurface, {
      props: props({
        workspaceSelection: { kind: "source_group", sourceGroupId: 20 }, currentSource: null,
        currentGroup: { id: 20, name: "Research group", source_type: "telegram", members: [
          { source_id: 1, source_title: "Alpha channel", item_count: 1 },
          { source_id: 2, source_title: "Beta channel", item_count: 1 },
        ], created_at: 1, updated_at: 2 },
        currentScopeTitle: "Research group", sourceHeaderCompact: false,
        groupLiveItemsBySource: { 1: [item(11, 1, "Alpha evidence")], 2: [item(22, 2, "Beta evidence")] },
        groupLiveHasMoreBySource: { 1: true, 2: false }, onChangeSelectedGroupSourceId, onLoadLiveGroupSourcePage,
      }),
    });

    expect(screen.getByRole("region", { name: "Source group sources" })).toBeTruthy();
    const alphaSource = screen.getByRole("region", { name: "Alpha channel" });
    const betaSource = screen.getByRole("region", { name: "Beta channel" });
    expect(alphaSource).toBeTruthy();
    expect(betaSource).toBeTruthy();
    expect(within(alphaSource).getByText("Alpha evidence")).toBeTruthy();
    expect(within(betaSource).getByText("Beta evidence")).toBeTruthy();
    expect(within(alphaSource).queryByText("Beta evidence")).toBeNull();
    expect(screen.getByRole("combobox", { name: "Source focus" })).toBeTruthy();
    expect(screen.getByRole("option", { name: "Alpha channel (1)" })).toBeTruthy();
    await fireEvent.change(screen.getByRole("combobox", { name: "Source focus" }), { target: { value: "1" } });
    expect(onChangeSelectedGroupSourceId).toHaveBeenCalledWith(1);
    await fireEvent.click(screen.getByRole("button", { name: "Load older messages" }));
    expect(onLoadLiveGroupSourcePage).toHaveBeenCalledWith(1);
  });
});
