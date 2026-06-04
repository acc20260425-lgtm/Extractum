import { describe, expect, it } from "vitest";
import reportSourceSurfaceSource from "./components/analysis/report-source-surface.svelte?raw";
import sourceActivityViewSource from "./components/analysis/source-activity-view.svelte?raw";
import sourceBrowserShellSource from "./components/analysis/source-browser-shell.svelte?raw";
import sourceGroupActivityViewSource from "./components/analysis/source-group-activity-view.svelte?raw";
import sourceGroupMetadataViewSource from "./components/analysis/source-group-metadata-view.svelte?raw";
import sourceMetadataViewSource from "./components/analysis/source-metadata-view.svelte?raw";
import sourceReaderHeaderSource from "./components/analysis/source-reader-header.svelte?raw";
import sourceGroupSourcesViewSource from "./components/analysis/source-group-sources-view.svelte?raw";
import telegramMediaCardSource from "./components/analysis/telegram-media-card.svelte?raw";
import telegramTimelineSource from "./components/analysis/telegram-timeline-reader.svelte?raw";
import universalItemsViewSource from "./components/analysis/universal-items-view.svelte?raw";
import rawJsonPanelSource from "./components/analysis/raw-json-panel.svelte?raw";
import runSnapshotMetadataViewSource from "./components/analysis/run-snapshot-metadata-view.svelte?raw";
import youtubePlaylistVideosViewSource from "./components/analysis/youtube-playlist-videos-view.svelte?raw";
import youtubeCommentsViewSource from "./components/analysis/youtube-comments-view.svelte?raw";
import youtubeSourceActivitySource from "./components/analysis/youtube-source-activity.svelte?raw";
import youtubeTranscriptSource from "./components/analysis/youtube-transcript-reader.svelte?raw";
import snapshotGroupSourcesViewSource from "./components/analysis/snapshot-group-sources-view.svelte?raw";
import snapshotItemsViewSource from "./components/analysis/snapshot-items-view.svelte?raw";

const sourceGroupReaderTag = "<" + "Source" + "Group" + "Reader";

function matchCount(source: string, pattern: RegExp) {
  return source.match(pattern)?.length ?? 0;
}

function normalizeLineEndings(source: string) {
  return source.replace(/\r\n/g, "\n");
}

function sourceBetween(source: string, start: string, end: string) {
  const startIndex = source.indexOf(start);
  expect(startIndex).toBeGreaterThanOrEqual(0);
  const endIndex = source.indexOf(end, startIndex);
  expect(endIndex).toBeGreaterThan(startIndex);
  return source.slice(startIndex, endIndex + end.length);
}

function sourceBrowserShellCall(marker: string) {
  const markerIndex = reportSourceSurfaceSource.indexOf(marker);
  expect(markerIndex).toBeGreaterThanOrEqual(0);
  const openIndex = reportSourceSurfaceSource.lastIndexOf("<SourceBrowserShell", markerIndex);
  expect(openIndex).toBeGreaterThanOrEqual(0);
  return sourceBetween(reportSourceSurfaceSource.slice(openIndex), "<SourceBrowserShell", "/>");
}

function sourceBrowserShellCalls() {
  const calls: string[] = [];
  let searchIndex = 0;

  while (true) {
    const openIndex = reportSourceSurfaceSource.indexOf("<SourceBrowserShell", searchIndex);
    if (openIndex === -1) {
      break;
    }
    const closeIndex = reportSourceSurfaceSource.indexOf("/>", openIndex);
    expect(closeIndex).toBeGreaterThan(openIndex);
    calls.push(reportSourceSurfaceSource.slice(openIndex, closeIndex + 2));
    searchIndex = closeIndex + 2;
  }

  return calls;
}

describe("analysis source readers", () => {
  it("replaces transitional source panels in ReportSourceSurface", () => {
    expect(reportSourceSurfaceSource).toContain("<SourceBrowserShell");
    expect(reportSourceSurfaceSource).not.toContain("<TelegramTimelineReader");
    expect(reportSourceSurfaceSource).not.toContain("<YoutubeTranscriptReader");
    expect(reportSourceSurfaceSource).not.toContain(sourceGroupReaderTag);
    expect(reportSourceSurfaceSource).not.toContain("<YoutubePlaylistReader");
    expect(reportSourceSurfaceSource).not.toContain("<SourceContextPanel");
    expect(reportSourceSurfaceSource).not.toContain("<YoutubeSourceDetail");
    expect(reportSourceSurfaceSource).not.toContain("<YoutubePlaylistDetail");
    expect(reportSourceSurfaceSource).not.toContain("<RunCompanionTabs");
  });

  it("routes live browsable sources and source groups through SourceBrowserShell", () => {
    const liveSourceShellCall = sourceBrowserShellCall('subject={{ kind: "source", source: currentSource }}');
    const liveGroupShellCall = sourceBrowserShellCall('subject={{ kind: "source_group", group: currentGroup }}');

    expect(reportSourceSurfaceSource).toContain("sourceBrowserShellAppliesToSource(currentSource)");
    expect(reportSourceSurfaceSource).toContain("sourceBrowserShellAppliesToSubject");
    expect(reportSourceSurfaceSource).toContain('subject={{ kind: "source", source: currentSource }}');
    expect(reportSourceSurfaceSource).toContain('subject={{ kind: "source_group", group: currentGroup }}');
    expect(reportSourceSurfaceSource).toContain("groupLiveSourceItems");
    expect(reportSourceSurfaceSource).toContain("groupLiveTranscriptReaderItems");
    expect(reportSourceSurfaceSource).toContain("sourceBrowserData={{");
    expect(reportSourceSurfaceSource).toContain("groupBrowserData={{");
    expect(matchCount(reportSourceSurfaceSource, /sourceBrowserData=\{\{/g)).toBe(1);
    expect(matchCount(reportSourceSurfaceSource, /groupBrowserData=\{\{/g)).toBe(1);
    expect(liveSourceShellCall).toContain("sourceBrowserData={{");
    expect(liveGroupShellCall).toContain("groupBrowserData={{");
    expect(liveGroupShellCall).not.toContain("sourceBrowserData={{");
    expect(liveGroupShellCall).not.toContain("liveReaderItems={[]}");
    expect(liveGroupShellCall).not.toContain("sourceItems={[]}");
    expect(liveGroupShellCall).not.toContain("sourceJobs={[]}");
    expect(liveGroupShellCall).not.toContain("sourceTopics={[]}");
    expect(liveGroupShellCall).not.toContain("youtubeVideoDetail={null}");
    expect(liveGroupShellCall).not.toContain("youtubePlaylistDetail={null}");
    expect(liveGroupShellCall).not.toContain("sourceSyncDisabledReason={() => null}");
    expect(reportSourceSurfaceSource).toContain("sourceLabelForGroupItem");
    expect(reportSourceSurfaceSource).toContain("<SourceBrowserShell");
    expect(reportSourceSurfaceSource).not.toContain("<YoutubePlaylistReader");
  });

  it("passes explicit subjects to every SourceBrowserShell call without legacy source props", () => {
    const shellCalls = sourceBrowserShellCalls();
    const liveSourceShellCall = sourceBrowserShellCall('subject={{ kind: "source", source: currentSource }}');
    const liveGroupShellCall = sourceBrowserShellCall('subject={{ kind: "source_group", group: currentGroup }}');
    const snapshotShellCall = sourceBrowserShellCall("subject={runSnapshotSubject}");

    expect(shellCalls).toHaveLength(3);
    for (const shellCall of shellCalls) {
      expect(shellCall).toContain("subject=");
      expect(shellCall).not.toContain("source={");
    }

    expect(liveSourceShellCall).toContain('subject={{ kind: "source", source: currentSource }}');
    expect(liveSourceShellCall).toContain("sourceBrowserData={{");
    expect(liveSourceShellCall).not.toContain("source={currentSource}");

    expect(liveGroupShellCall).toContain('subject={{ kind: "source_group", group: currentGroup }}');
    expect(liveGroupShellCall).toContain("groupBrowserData={{");
    expect(liveGroupShellCall).not.toContain("source={null}");
    expect(liveGroupShellCall).not.toContain("sourceBrowserData={{");

    expect(snapshotShellCall).toContain("subject={runSnapshotSubject}");
    expect(snapshotShellCall).toContain("snapshotBrowserData={{");
    expect(snapshotShellCall).not.toContain("source={null}");
    expect(snapshotShellCall).not.toContain("sourceBrowserData={{");
  });

  it("routes available run snapshots through SourceBrowserShell while keeping the header route-owned", () => {
    const snapshotShellCall = sourceBrowserShellCall("subject={runSnapshotSubject}");

    expect(reportSourceSurfaceSource).toContain('sourceViewBasis === "run_snapshot"');
    expect(reportSourceSurfaceSource).toContain("<SourceReaderHeader");
    expect(reportSourceSurfaceSource).toContain("runSnapshotSubject");
    expect(reportSourceSurfaceSource).toContain("snapshotBrowserData={{");
    expect(matchCount(reportSourceSurfaceSource, /snapshotBrowserData=\{\{/g)).toBe(1);
    expect(snapshotShellCall).toContain("snapshotBrowserData={{");
    expect(snapshotShellCall).not.toContain("sourceBrowserData={{");
    expect(reportSourceSurfaceSource).toContain("subject={runSnapshotSubject}");
    expect(reportSourceSurfaceSource).toContain("{onViewLiveSource}");
    expect(reportSourceSurfaceSource).not.toContain(sourceGroupReaderTag);
  });

  it("keeps snapshot shell data frozen-only and live props empty", () => {
    const snapshotShellCall = sourceBrowserShellCall("subject={runSnapshotSubject}");

    expect(reportSourceSurfaceSource).toContain("deriveRunSnapshotBrowserKind");
    expect(reportSourceSurfaceSource).toContain("allSnapshotReaderItems");
    expect(snapshotShellCall).toContain("snapshotBrowserData={{");
    expect(snapshotShellCall).not.toContain("sourceJobs={[]}");
    expect(snapshotShellCall).not.toContain("takeoutRecovery={null}");
    expect(snapshotShellCall).not.toContain("sourceSyncDisabledReason={() => null}");
    expect(snapshotShellCall).not.toContain("liveReaderItems={[]}");
    expect(snapshotShellCall).not.toContain("sourceItems={[]}");
    expect(snapshotShellCall).not.toContain("sourceTopics={[]}");
    expect(snapshotShellCall).not.toContain("youtubeVideoDetail={null}");
    expect(snapshotShellCall).not.toContain("youtubePlaylistDetail={null}");
  });

  it("renders YouTube playlist videos through SourceBrowserShell", () => {
    expect(sourceBrowserShellSource).toContain("<YoutubePlaylistVideosView");
    expect(sourceBrowserShellSource).toContain('activeTab === "videos"');
    expect(sourceBrowserShellSource).toContain("youtubePlaylistDetail");
    expect(sourceBrowserShellSource).toContain("onRetryFailedPlaylistVideos");
    expect(sourceBrowserShellSource).toContain("onRetryPlaylistVideo");
  });

  it("keeps SourceBrowserShell mounted across supported live source switches", () => {
    const shellIndex = reportSourceSurfaceSource.indexOf("<SourceBrowserShell");
    expect(shellIndex).toBeGreaterThan(0);

    const shellPrelude = reportSourceSurfaceSource.slice(Math.max(0, shellIndex - 420), shellIndex);

    expect(shellPrelude).not.toContain("{#key");
    expect(shellPrelude).not.toContain("currentSource.id");
  });

  it("preserves the existing Telegram timeline controls through the shell", () => {
    expect(sourceBrowserShellSource).toContain("telegramHistoryScopeOptions");
    expect(sourceBrowserShellSource).toContain("onChangeTelegramHistoryScope");
    expect(sourceBrowserShellSource).toContain("showTopicSelector");
    expect(sourceBrowserShellSource).toContain("onChangeSelectedTopicKey");
    expect(sourceBrowserShellSource).toContain("<TelegramTimelineReader");
    expect(sourceBrowserShellSource).toContain("liveReaderItems");
  });

  it("keeps live source and run snapshot basis visible", () => {
    expect(sourceReaderHeaderSource).toContain("sourceViewBasis");
    expect(sourceReaderHeaderSource).toContain("sourceBasisState");
    expect(sourceReaderHeaderSource).toContain("run_snapshot_unavailable");
    expect(sourceReaderHeaderSource).toContain("Live source");
    expect(sourceReaderHeaderSource).toContain("Run snapshot");
    expect(sourceReaderHeaderSource).toContain("Back to run snapshot");
    expect(sourceReaderHeaderSource).toContain("View live source");
    expect(reportSourceSurfaceSource).toContain("sourceBasisState={canvasSurface}");
    expect(reportSourceSurfaceSource).toContain("snapshotAffordance.detailTitle");
    expect(reportSourceSurfaceSource).toContain("snapshotAffordance.detailDescription");
    expect(reportSourceSurfaceSource).not.toContain("<StatusMessage tone=\"error\">{runSnapshotError}</StatusMessage>");
    expect(reportSourceSurfaceSource).toContain("canViewLiveSourceForSnapshot");
  });

  it("uses a compact source reader heading instead of repeating the selected title", () => {
    expect(sourceReaderHeaderSource).toContain("surfaceLabel");
    expect(sourceReaderHeaderSource).not.toContain("<h2>{title}</h2>");
  });

  it("renders Telegram as a metadata-rich timeline without binary previews", () => {
    expect(telegramTimelineSource).toContain('class="telegram-timeline-reader"');
    expect(telegramTimelineSource).toContain("groupReaderItemsByDay");
    expect(telegramTimelineSource).toContain("topicLabel");
    expect(telegramTimelineSource).toContain("replyLabel");
    expect(telegramTimelineSource).toContain("reactionLabel");
    expect(telegramTimelineSource).toContain("<TelegramMediaCard");
    expect(telegramMediaCardSource).toContain("media-card");
    expect(telegramMediaCardSource).toContain("media.fileName");
    expect(telegramMediaCardSource).toContain("media.mimeType");
    expect(telegramMediaCardSource).not.toContain("<img");
    expect(telegramMediaCardSource).not.toContain("<video");
    expect(telegramMediaCardSource).not.toContain("<audio");
  });

  it("surfaces migrated Telegram history labels and scope controls", () => {
    expect(telegramTimelineSource).toContain("historyScopeLabel");
    expect(telegramTimelineSource).toContain('class="history-scope-badge"');
    expect(sourceBrowserShellSource).toContain("telegramHistoryScopeOptions");
    expect(sourceBrowserShellSource).toContain("Current supergroup history");
    expect(sourceBrowserShellSource).toContain("Migrated small-group history");
    expect(sourceBrowserShellSource).toContain("Merged timeline");
    expect(sourceBrowserShellSource).toContain("onChangeTelegramHistoryScope");
  });

  it("shows migrated Telegram history availability before imported rows are browsable", () => {
    expect(sourceBrowserShellSource).toContain('migratedHistoryStatus === "available"');
    expect(sourceBrowserShellSource).toContain("migratedHistoryImportCompleted");
    expect(sourceBrowserShellSource).toContain("migratedHistoryRowCount === 0");
    expect(sourceBrowserShellSource).toContain('tone="info"');
  });

  it("renders Telegram topic filtering only in live single-source mode", () => {
    expect(sourceBrowserShellSource).toContain('class="topic-filter"');
    expect(sourceBrowserShellSource).toContain("showTopicSelector");
    expect(sourceBrowserShellSource).toContain("sourceTopics");
    expect(sourceBrowserShellSource).toContain("loadingSourceTopics");
    expect(sourceBrowserShellSource).toContain("selectedTopicKey");
    expect(sourceBrowserShellSource).toContain("onChangeSelectedTopicKey");
    expect(sourceBrowserShellSource).toContain("__all_topics__");
    expect(sourceGroupSourcesViewSource).not.toContain("topic-filter");
  });

  it("uses the shared takeout recovery notice in the selected source surface", () => {
    expect(reportSourceSurfaceSource).toContain("TakeoutRecoveryNotice");
    expect(reportSourceSurfaceSource).toContain("takeoutRecovery");
  });

  it("keeps live single-source timeline readers pageable", () => {
    expect(sourceBrowserShellSource).toContain("sourceItemsHasMore");
    expect(sourceBrowserShellSource).toContain("onLoadMoreSourceItems");
    expect(sourceBrowserShellSource).toContain("hasMore={sourceData.sourceItemsHasMore}");
    expect(sourceBrowserShellSource).toContain("onLoadMore={sourceData.onLoadMoreSourceItems}");
  });

  it("keeps sticky date labels below overlay source switching UI", () => {
    expect(telegramTimelineSource).toContain(".day-label");
    expect(telegramTimelineSource).toContain("position: sticky;");
    expect(telegramTimelineSource).toContain("z-index: 0;");
    expect(telegramTimelineSource).not.toContain("z-index: 1;");
  });

  it("uses TDesktop-inspired Telegram message geometry and typography", () => {
    expect(telegramTimelineSource).toContain('class="telegram-message-bubble"');
    expect(telegramTimelineSource).toContain('class="telegram-message-text"');
    expect(telegramTimelineSource).toContain('class="telegram-message-time"');
    expect(telegramTimelineSource).toContain("max-width: 460px;");
    expect(telegramTimelineSource).toContain("border-radius: 12px;");
    expect(telegramTimelineSource).toContain("font-size: 0.9375rem;");
    expect(telegramTimelineSource).toContain("font-size: 0.8125rem;");
  });

  it("centers Telegram message bubbles under sticky day labels", () => {
    expect(telegramTimelineSource).toMatch(/li\s*{\s*display: flex;\s*justify-content: center;/);
  });

  it("keeps Telegram bubbles and attachments visually compact", () => {
    expect(telegramTimelineSource).toContain("padding: 0.5rem 0.625rem 0.4375rem;");
    expect(telegramTimelineSource).toContain("line-height: 1.4;");
    expect(telegramTimelineSource).toContain("box-shadow: 0 0 0 1px");
    expect(telegramMediaCardSource).toContain("padding: 0.375rem 0.5rem;");
    expect(telegramMediaCardSource).toContain("border-radius: 7px;");
    expect(telegramMediaCardSource).toContain("font-size: 0.8125rem;");
  });

  it("allows Telegram message text to hyphenate long words", () => {
    expect(telegramTimelineSource).toContain('lang="ru"');
    expect(telegramTimelineSource).toContain("hyphens: auto;");
    expect(telegramTimelineSource).toContain("overflow-wrap: break-word;");
    expect(telegramTimelineSource).toContain("word-break: normal;");
  });

  it("renders YouTube videos as transcript-first source readers", () => {
    expect(youtubeTranscriptSource).toContain('class="youtube-transcript-reader"');
    expect(youtubeTranscriptSource).toContain("showSyncActions = true");
    expect(youtubeTranscriptSource).toContain("{#if showSyncActions}");
    expect(youtubeTranscriptSource).toContain("groupYoutubeTranscriptItems");
    expect(youtubeTranscriptSource).toContain("formatYoutubeTime");
    expect(youtubeTranscriptSource).toContain("youtubeTimestampUrl");
    expect(youtubeTranscriptSource).toContain("Copy timestamp link");
    expect(youtubeTranscriptSource).toContain("Search transcript");
    expect(youtubeTranscriptSource).toContain("Load more transcript");
    expect(youtubeTranscriptSource).not.toContain("<iframe");
    expect(youtubeTranscriptSource).not.toContain("<video");
  });

  it("keeps YouTube live sync actions out of readonly snapshot transcript readers", () => {
    expect(sourceBrowserShellSource).toContain("showSyncActions={false}");
    expect(sourceGroupSourcesViewSource).toContain("showSyncActions={false}");
    expect(snapshotGroupSourcesViewSource).toContain("showSyncActions={false}");
    expect(sourceBrowserShellSource).toContain("onSyncTranscript={() => sourceData.onSyncYoutubeTranscript(sourceSubject.id)}");
  });

  it("keeps run snapshot YouTube readers detached from live video detail", () => {
    expect(sourceBrowserShellSource).toContain("detail={null}");
    expect(reportSourceSurfaceSource).not.toContain("detail={youtubeVideoDetail}");
    expect(sourceBrowserShellSource).toContain("detail={sourceData.youtubeVideoDetail}");
    expect(sourceBrowserShellSource).toContain('sourceTitle={sourceSubject.title ?? sourceSubject.externalId}');
  });

  it("keeps live YouTube video comments sync status and CTAs in transcript reader", () => {
    expect(youtubeTranscriptSource).toContain("onSyncComments");
    expect(youtubeTranscriptSource).toContain("youtubeProviderHeaderSummary");
    expect(youtubeTranscriptSource).toContain("youtubeContentStatusLine");
    expect(youtubeTranscriptSource).toContain("commentsStatus.countLabel");
    expect(youtubeTranscriptSource).toContain("commentsStatus.lastSyncedLabel");
    expect(youtubeTranscriptSource).toContain("Sync comments");
  });

  it("moves detailed source job cards into the Activity tab", () => {
    expect(sourceBrowserShellSource).toContain("activity");
    expect(sourceBrowserShellSource).toContain("<SourceActivityView");
    expect(sourceActivityViewSource).toContain("SourceJobRecord");
    expect(sourceActivityViewSource).toContain("Progress");
    expect(sourceActivityViewSource).toContain("Warnings");
    expect(sourceActivityViewSource).toContain("Error");
    expect(sourceActivityViewSource).toContain("Cancel");
  });

  it("keeps provider tabs to contextual CTAs instead of detailed job cards", () => {
    expect(youtubeTranscriptSource).not.toContain("<YoutubeSourceActivity");
    expect(youtubeTranscriptSource).not.toContain("SourceJobRecord");
    expect(youtubeTranscriptSource).toContain("Sync comments");
    expect(youtubeTranscriptSource).toContain("Sync metadata");
  });

  it("covers Telegram source activity without adding backend job APIs", () => {
    expect(sourceActivityViewSource).toContain("takeoutRecovery");
    expect(sourceActivityViewSource).toContain("sourceSyncDisabledReason");
    expect(sourceActivityViewSource).toContain("onStartTakeoutImport");
    expect(sourceActivityViewSource).toContain("onStartMigratedHistoryImport");
    expect(sourceActivityViewSource).toContain("Migrated history");
    expect(sourceActivityViewSource).toContain("Takeout");
  });

  it("renders universal Items as a loaded-window browser", () => {
    expect(universalItemsViewSource).toContain("Search loaded items");
    expect(universalItemsViewSource).toContain("All");
    expect(universalItemsViewSource).toContain("Load more items");
    expect(universalItemsViewSource).toContain("Unknown item kind");
    expect(universalItemsViewSource).toContain("emptyDescription");
    expect(universalItemsViewSource).toContain("helpDescription");
    expect(universalItemsViewSource).toContain("sourceLabelForItem");
    expect(universalItemsViewSource).toContain("Source #${item.sourceId}");
  });

  it("renders snapshot Items as a frozen SourceReaderItem browser", () => {
    expect(snapshotItemsViewSource).toContain("SourceReaderItem");
    expect(snapshotItemsViewSource).not.toContain("SourceItem");
    expect(snapshotItemsViewSource).toContain("Search snapshot items");
    expect(snapshotItemsViewSource).toContain("Snapshot items are limited to frozen rows loaded for this run");
    expect(snapshotItemsViewSource).toContain("Load older snapshot messages");
    expect(snapshotItemsViewSource).toContain("selectedTraceRef");
    expect(snapshotItemsViewSource).toContain("ariaPressed");
  });

  it("renders snapshot group Sources with global snapshot paging only", () => {
    expect(snapshotGroupSourcesViewSource).toContain("groupReaderItemsBySource");
    expect(snapshotGroupSourcesViewSource).toContain("Load older snapshot messages");
    expect(snapshotGroupSourcesViewSource).toContain("showSyncActions={false}");
    expect(snapshotGroupSourcesViewSource).toContain("otherItems");
    expect(snapshotGroupSourcesViewSource).toContain("other-item-list");
    expect(snapshotGroupSourcesViewSource).not.toContain("hasMoreBySource");
    expect(snapshotGroupSourcesViewSource).not.toContain("onLoadMoreSource");
  });

  it("renders run snapshot metadata from route-owned fields", () => {
    expect(runSnapshotMetadataViewSource).toContain("AnalysisRunDetail");
    expect(runSnapshotMetadataViewSource).toContain("Run snapshot");
    expect(runSnapshotMetadataViewSource).toContain("snapshot.readerKind");
    expect(runSnapshotMetadataViewSource).toContain("sourceOptions");
    expect(runSnapshotMetadataViewSource).not.toContain("RawJsonPanel");
  });

  it("renders YouTube comments as a loaded-window browser", () => {
    expect(youtubeCommentsViewSource).toContain("Search comments");
    expect(youtubeCommentsViewSource).not.toContain("Search loaded comments");
    expect(youtubeCommentsViewSource).toContain("Audience comments are user-generated evidence");
    expect(youtubeCommentsViewSource).toContain("Threaded");
    expect(youtubeCommentsViewSource).toContain("Flat");
    expect(youtubeCommentsViewSource).toContain("Most liked");
    expect(youtubeCommentsViewSource).toContain("parent not loaded");
    expect(youtubeCommentsViewSource).toContain("Sync comments");
  });

  it("renders source metadata in structured sections with bounded raw JSON", () => {
    expect(sourceMetadataViewSource).toContain("Summary");
    expect(sourceMetadataViewSource).toContain("Source state");
    expect(sourceMetadataViewSource).toContain("Technical");
    expect(sourceMetadataViewSource).toContain("<RawJsonPanel");
    expect(sourceMetadataViewSource).toContain("youtubePlaylistDetail");
    expect(sourceMetadataViewSource).toContain("Playlist ID");
    expect(sourceMetadataViewSource).toContain("Linked videos");
    expect(sourceMetadataViewSource).not.toContain("items.raw_data_zstd");
    expect(rawJsonPanelSource).toContain("Show raw JSON");
    expect(rawJsonPanelSource).toContain("Copy");
    expect(rawJsonPanelSource).toContain("Large payload");
  });

  it("passes playlist detail into metadata and playlist-specific empty copy into Items", () => {
    expect(sourceBrowserShellSource).toContain("youtubePlaylistDetail={sourceData.youtubePlaylistDetail}");
    expect(sourceBrowserShellSource).toContain("Playlist videos live in the Videos tab");
    expect(sourceBrowserShellSource).toContain("emptyDescription=");
  });

  it("passes live YouTube video comments and jobs only into live transcript readers", () => {
    expect(reportSourceSurfaceSource).toContain("sourceBrowserData={{");
    expect(reportSourceSurfaceSource).toContain("sourceJobs,");
    expect(sourceBrowserShellSource).toContain("onSyncComments={() => sourceData.onSyncYoutubeComments(sourceSubject.id)}");
    expect(sourceBrowserShellSource).toContain("onCancelSourceJob={sourceData.onCancelSourceJob}");
    expect(sourceBrowserShellSource).toContain("showSyncActions={false}");
    expect(reportSourceSurfaceSource).not.toContain("onSyncComments={() => {}}");
  });

  it("renders YouTube source job activity with progress warnings errors and cancel", () => {
    expect(youtubeSourceActivitySource).toContain('class="youtube-source-activity"');
    expect(youtubeSourceActivitySource).toContain("SourceJobRecord");
    expect(youtubeSourceActivitySource).toContain("progressLabel(job)");
    expect(youtubeSourceActivitySource).toContain("job.warnings");
    expect(youtubeSourceActivitySource).toContain("job.error");
    expect(youtubeSourceActivitySource).toContain("onCancelJob(job.job_id)");
    expect(youtubeSourceActivitySource).toContain("cancel_requested");
  });

  it("surfaces YouTube runtime diagnostics in the live source canvas", () => {
    expect(reportSourceSurfaceSource).toContain("sourceSyncDisabledReason");
    expect(reportSourceSurfaceSource).toContain("youtubeRuntimeDiagnostic");
    expect(reportSourceSurfaceSource).toContain('tone="error"');
  });

  it("renders transcript search as one compact input shell", () => {
    expect(youtubeTranscriptSource).toContain('placeholder="Search transcript"');
    expect(youtubeTranscriptSource).toContain('class="search-icon"');
    expect(youtubeTranscriptSource).toContain('class="search-input-wrap"');
  });

  it("renders grouped transcript rows as a continuous reading surface", () => {
    expect(youtubeTranscriptSource).toContain("transcriptGroups");
    expect(youtubeTranscriptSource).toContain('class="transcript-group-list"');
    expect(youtubeTranscriptSource).toContain('class:selected={group.selected}');
    expect(youtubeTranscriptSource).toContain(".transcript-group-list li + li");
    expect(youtubeTranscriptSource).not.toContain("border-radius: 8px;");
    expect(youtubeTranscriptSource).not.toContain("background: var(--panel);");
    expect(youtubeTranscriptSource).not.toContain("box-shadow: 0 0 0 3px");
  });

  it("scrolls selected Telegram and YouTube source rows into view", () => {
    expect(telegramTimelineSource).toContain("scrollSelectedMessageIntoView");
    expect(telegramTimelineSource).toContain("scrollIntoView");
    expect(telegramTimelineSource).toContain("data-trace-ref={item.ref}");
    expect(youtubeTranscriptSource).toContain("scrollSelectedTranscriptGroupIntoView");
    expect(youtubeTranscriptSource).toContain("scrollIntoView");
    expect(youtubeTranscriptSource).toContain('data-trace-refs={group.refs.join(" ")}');
    expect(youtubeTranscriptSource).not.toContain("data-trace-ref={visibleRef}");
  });

  it("adds one-shot evidence highlight support to trace-capable readers", () => {
    const traceCapableReaders = [
      telegramTimelineSource,
      youtubeTranscriptSource,
      snapshotItemsViewSource,
      snapshotGroupSourcesViewSource,
      sourceGroupSourcesViewSource,
      universalItemsViewSource,
      youtubeCommentsViewSource,
    ];

    for (const source of traceCapableReaders) {
      expect(source).toContain("EvidenceHighlightToken");
      expect(source).toContain("highlightToken?: EvidenceHighlightToken | null");
      expect(source).toContain("highlightToken = null");
      expect(source).toContain("data-evidence-highlighted=");
      expect(source).toContain("consumedHighlightTokenIds");
      expect(source).toContain("!consumedHighlightTokenIds.has(highlightToken.tokenId)");
      expect(source).toContain("consumedHighlightTokenIds.add(highlightToken.tokenId)");
    }
  });

  it("matches evidence highlights by concrete trace refs without replacing selected row behavior", () => {
    expect(telegramTimelineSource).toContain("function isEvidenceHighlighted(ref: string | null)");
    expect(telegramTimelineSource).toContain("highlightToken.traceRef === ref");
    expect(telegramTimelineSource).toContain('class:selected={item.selected}');
    expect(telegramTimelineSource).toContain('data-evidence-highlighted={isEvidenceHighlighted(item.ref) ? "true" : undefined}');

    expect(youtubeTranscriptSource).toContain("function isEvidenceHighlighted(group: YoutubeTranscriptGroup)");
    expect(youtubeTranscriptSource).toContain("group.refs.includes(highlightToken.traceRef)");
    expect(youtubeTranscriptSource).toContain("const highlightedGroup = transcriptGroups.find((group) => isEvidenceHighlighted(group));");
    expect(youtubeTranscriptSource).toContain('class:selected={group.selected}');
    expect(youtubeTranscriptSource).toContain('data-evidence-highlighted={isEvidenceHighlighted(group) ? "true" : undefined}');
    expect(youtubeTranscriptSource).toContain('data-trace-refs={group.refs.join(" ")}');

    expect(snapshotItemsViewSource).toContain("function isEvidenceHighlighted(ref: string | null)");
    expect(snapshotItemsViewSource).toContain("highlightToken.traceRef === ref");
    expect(snapshotItemsViewSource).toContain('class:selected={itemSelected(item)}');
    expect(snapshotItemsViewSource).toContain('data-evidence-highlighted={isEvidenceHighlighted(item.ref) ? "true" : undefined}');

    expect(snapshotGroupSourcesViewSource).toContain('data-evidence-highlighted={isEvidenceHighlighted(item.ref) ? "true" : undefined}');
    expect(sourceGroupSourcesViewSource).toContain("{highlightToken}");

    expect(universalItemsViewSource).toContain("liveSourceItemRef");
    expect(universalItemsViewSource).toContain("function isEvidenceHighlighted(item: SourceItem)");
    expect(universalItemsViewSource).toContain("highlightToken.traceRef === liveSourceItemRef(item)");
    expect(universalItemsViewSource).toContain("data-trace-ref={liveSourceItemRef(item)}");
    expect(universalItemsViewSource).toContain('data-evidence-highlighted={isEvidenceHighlighted(item) ? "true" : undefined}');

    expect(youtubeCommentsViewSource).toContain("liveSourceItemRef");
    expect(youtubeCommentsViewSource).toContain("function isEvidenceHighlighted(item: SourceItem)");
    expect(youtubeCommentsViewSource).toContain("highlightToken.traceRef === liveSourceItemRef(item)");
    expect(youtubeCommentsViewSource).toContain("data-trace-ref={liveSourceItemRef(item)}");
    expect(youtubeCommentsViewSource).toContain('data-evidence-highlighted={isEvidenceHighlighted(item) ? "true" : undefined}');
    expect(youtubeCommentsViewSource).toContain("{#snippet commentCard(item: SourceItem, parentLoaded: boolean)}");
  });

  it("extends existing selected row styles for tokenized evidence highlights without hardcoded colors", () => {
    expect(normalizeLineEndings(telegramTimelineSource)).toContain('li.selected,\n  li[data-evidence-highlighted="true"]');
    expect(normalizeLineEndings(youtubeTranscriptSource)).toContain('.transcript-group-list li.selected,\n  .transcript-group-list li[data-evidence-highlighted="true"]');
    expect(normalizeLineEndings(snapshotItemsViewSource)).toContain('article.selected,\n  article[data-evidence-highlighted="true"]');
    expect(normalizeLineEndings(snapshotGroupSourcesViewSource)).toContain('.other-item-list li.selected,\n  .other-item-list li[data-evidence-highlighted="true"]');
    expect(universalItemsViewSource).toContain('article[data-evidence-highlighted="true"]');
    expect(youtubeCommentsViewSource).toContain('article[data-evidence-highlighted="true"]');
    expect(telegramTimelineSource).toContain("var(--primary)");
    expect(youtubeTranscriptSource).toContain("var(--primary)");
    expect(snapshotItemsViewSource).toContain("var(--accent)");
    expect(snapshotGroupSourcesViewSource).toContain("var(--accent)");
    expect(universalItemsViewSource).toContain("var(--accent)");
    expect(youtubeCommentsViewSource).toContain("var(--accent)");
  });

  it("renders YouTube playlist videos as a job-free leaf view", () => {
    expect(youtubePlaylistVideosViewSource).toContain('aria-label="YouTube playlist videos"');
    expect(youtubePlaylistVideosViewSource).toContain("playlist.items");
    expect(youtubePlaylistVideosViewSource).toContain("onOpenSource");
    expect(youtubePlaylistVideosViewSource).toContain("onSyncPlaylist");
    expect(youtubePlaylistVideosViewSource).toContain("onRetryFailedPlaylistVideos");
    expect(youtubePlaylistVideosViewSource).toContain("onSyncPlaylistVideo");
    expect(youtubePlaylistVideosViewSource).toContain("onRetryPlaylistVideo");
    expect(youtubePlaylistVideosViewSource).toContain("isRetryableYoutubeAvailabilityStatus");
    expect(youtubePlaylistVideosViewSource).not.toContain("retryableStatuses");
    expect(youtubePlaylistVideosViewSource).not.toContain("SourceActivityView");
    expect(youtubePlaylistVideosViewSource).not.toContain("YoutubeSourceActivity");
    expect(youtubePlaylistVideosViewSource).not.toContain("sourceJobs");
    expect(youtubePlaylistVideosViewSource).not.toContain("onCancelSourceJob");
    expect(youtubePlaylistVideosViewSource).not.toContain("$lib/api/");
    expect(youtubePlaylistVideosViewSource).not.toContain("invoke(");
  });

  it("renders source group sources as a route-free tab leaf", () => {
    expect(sourceGroupSourcesViewSource).toContain('aria-label="Source group sources"');
    expect(sourceGroupSourcesViewSource).toContain("groupReaderItemsBySource");
    expect(sourceGroupSourcesViewSource).toContain("onLoadMoreSource");
    expect(sourceGroupSourcesViewSource).toContain("selectedGroupSourceId");
    expect(sourceGroupSourcesViewSource).toContain("selectedTraceRef");
    expect(sourceGroupSourcesViewSource).toContain("youtubeItems");
    expect(sourceGroupSourcesViewSource).toContain("telegramItems");
    expect(sourceGroupSourcesViewSource).not.toContain("$lib/api/");
    expect(sourceGroupSourcesViewSource).not.toContain("invoke(");
    expect(sourceGroupSourcesViewSource).not.toContain("SourceBrowserShell");
    expect(sourceGroupSourcesViewSource).not.toContain("SourceActivityView");
    expect(sourceGroupSourcesViewSource).not.toContain("<span>Source focus</span>");
  });

  it("keeps migrated source browser contracts on canonical shell and leaves", () => {
    expect(reportSourceSurfaceSource).not.toContain(sourceGroupReaderTag);
    expect(reportSourceSurfaceSource).toContain("<SourceBrowserShell");
    expect(sourceBrowserShellSource).toContain("<SourceGroupSourcesView");
    expect(sourceBrowserShellSource).toContain("<SnapshotGroupSourcesView");
    expect(sourceBrowserShellSource).toContain("<SnapshotItemsView");
    expect(sourceBrowserShellSource).toContain("<RunSnapshotMetadataView");
    expect(sourceGroupSourcesViewSource).not.toContain("$lib/api/");
    expect(sourceGroupSourcesViewSource).not.toContain("invoke(");
    expect(snapshotGroupSourcesViewSource).not.toContain("$lib/api/");
    expect(snapshotGroupSourcesViewSource).not.toContain("invoke(");
    expect(snapshotItemsViewSource).not.toContain("$lib/api/");
    expect(snapshotItemsViewSource).not.toContain("invoke(");
    expect(runSnapshotMetadataViewSource).not.toContain("$lib/api/");
    expect(runSnapshotMetadataViewSource).not.toContain("invoke(");
  });

  it("renders source group metadata from route-owned group fields", () => {
    expect(sourceGroupMetadataViewSource).toContain('aria-label="Source group metadata"');
    expect(sourceGroupMetadataViewSource).toContain("group.name");
    expect(sourceGroupMetadataViewSource).toContain("group.source_type");
    expect(sourceGroupMetadataViewSource).toContain("group.members.length");
    expect(sourceGroupMetadataViewSource).toContain("member.item_count");
    expect(sourceGroupMetadataViewSource).toContain("formatTimestamp(group.created_at)");
    expect(sourceGroupMetadataViewSource).toContain("formatTimestamp(group.updated_at)");
    expect(sourceGroupMetadataViewSource).not.toContain("$lib/api/");
    expect(sourceGroupMetadataViewSource).not.toContain("invoke(");
  });

  it("renders source group activity without source job cards", () => {
    expect(sourceGroupActivityViewSource).toContain('aria-label="Source group activity"');
    expect(sourceGroupActivityViewSource).toContain("Group activity is not available yet. Source jobs are still tracked per source.");
    expect(sourceGroupActivityViewSource).not.toContain("SourceActivityView");
    expect(sourceGroupActivityViewSource).not.toContain("SourceJobRecord");
    expect(sourceGroupActivityViewSource).not.toContain("onCancelSourceJob");
    expect(sourceGroupActivityViewSource).not.toContain("$lib/api/");
    expect(sourceGroupActivityViewSource).not.toContain("invoke(");
  });

  it("keeps source group activity out of SourceActivityView", () => {
    expect(sourceBrowserShellSource).toContain("<SourceGroupActivityView");
    expect(sourceBrowserShellSource).toContain("<SourceActivityView");
    expect(sourceBrowserShellSource).toContain('activeTab === "activity" && groupSubject');
    expect(sourceBrowserShellSource).toContain('activeTab === "activity" && sourceSubject');
    expect(sourceBrowserShellSource).toContain('subject.kind === "source_group"');
    expect(sourceBrowserShellSource).toContain('subject.kind === "source"');
    expect(sourceBrowserShellSource).toContain("sourceSubject");
  });

  it("keeps playlist video opening as source selection instead of nested browsing", () => {
    expect(youtubePlaylistVideosViewSource).toContain("onOpenSource");
    expect(youtubePlaylistVideosViewSource).toContain("videoSourceId");
    expect(youtubePlaylistVideosViewSource).not.toContain("<YoutubeTranscriptReader");
    expect(youtubePlaylistVideosViewSource).not.toContain("<SourceBrowserShell");
    expect(youtubePlaylistVideosViewSource).not.toContain("SourceActivityView");
    expect(youtubePlaylistVideosViewSource).not.toContain("$lib/api/");
  });

  it("groups source group material by source", () => {
    expect(sourceGroupSourcesViewSource).toContain('class="source-group-sources-view"');
    expect(sourceGroupSourcesViewSource).toContain("groupReaderItemsBySource");
    expect(sourceGroupSourcesViewSource).toContain("youtubeItems");
    expect(sourceGroupSourcesViewSource).toContain("telegramItems");
    expect(sourceGroupSourcesViewSource).toContain('item.kind === "youtube_transcript"');
    expect(sourceGroupSourcesViewSource).not.toContain("snapshotItems={group.items}");
    expect(sourceGroupSourcesViewSource).toContain("source-heading");
    expect(sourceGroupSourcesViewSource).toContain("selectedGroupSourceId");
    expect(reportSourceSurfaceSource).toContain("onChangeSelectedSourceId={onChangeSelectedGroupSourceId}");
  });

  it("merges focused group YouTube transcript DTOs into source-group reader items", () => {
    expect(reportSourceSurfaceSource).toContain("youtubeSegmentToReaderItem");
    expect(reportSourceSurfaceSource).toContain("groupLiveTranscriptSegmentsBySource");
    expect(reportSourceSurfaceSource).toContain("groupLiveTranscriptReaderItems");
    expect(reportSourceSurfaceSource).toContain("const sourceTitle = sourceTitleForGroupSource(sourceId)");
    expect(reportSourceSurfaceSource).toContain("youtubeSegmentToReaderItem(segment, {");
    expect(reportSourceSurfaceSource).toContain("sourceTitle,");
    expect(reportSourceSurfaceSource).toContain("canonicalUrl: null");
    expect(reportSourceSurfaceSource).toContain("selectedTraceRef");
    expect(reportSourceSurfaceSource).toContain("...groupLiveSourceItemReaderItems");
    expect(reportSourceSurfaceSource).toContain("...groupLiveTranscriptReaderItems");
  });

  it("uses a neutral timeline label for mixed source-group material", () => {
    expect(telegramTimelineSource).toContain("ariaLabel = \"Telegram source timeline\"");
    expect(telegramTimelineSource).toContain("aria-label={ariaLabel}");
    expect(sourceGroupSourcesViewSource).toContain('ariaLabel="Source material timeline"');
  });

  it("builds live source-group focus options from every group member", () => {
    expect(reportSourceSurfaceSource).toContain("sourceFilterOptionsFromGroupMembers");
    expect(reportSourceSurfaceSource).toContain("sourceOptions={analysisScope === \"source_group\" ? liveGroupSourceOptions : []}");
    expect(reportSourceSurfaceSource).not.toContain("sourceFilterOptions(groupLiveReaderItems)");
  });

  it("keeps run snapshot focus options based on the whole loaded snapshot page", () => {
    expect(reportSourceSurfaceSource).toContain("const allSnapshotReaderItems");
    expect(reportSourceSurfaceSource).toContain("sourceFilterOptionsFromReaderItems(allSnapshotReaderItems)");
    expect(reportSourceSurfaceSource).toContain("allSnapshotReaderItems.filter");
    expect(reportSourceSurfaceSource).not.toContain("sourceFilterOptionsFromReaderItems(snapshotReaderItems)");
  });

  it("keeps source focus controls in one reader header location", () => {
    expect(sourceReaderHeaderSource).toContain("<span>Source focus</span>");
    expect(sourceGroupSourcesViewSource).not.toContain("<span>Source focus</span>");
    expect(sourceGroupSourcesViewSource).not.toContain("group-filter");
  });
});
