<script lang="ts">
  import Button from "$lib/components/ui/Button.svelte";
  import Select from "$lib/components/ui/Select.svelte";
  import StatusMessage from "$lib/components/ui/StatusMessage.svelte";
  import SourceActivityView from "$lib/components/analysis/source-activity-view.svelte";
  import SourceGroupActivityView from "$lib/components/analysis/source-group-activity-view.svelte";
  import SourceGroupMetadataView from "$lib/components/analysis/source-group-metadata-view.svelte";
  import SourceGroupSourcesView from "$lib/components/analysis/source-group-sources-view.svelte";
  import RunSnapshotMetadataView from "$lib/components/analysis/run-snapshot-metadata-view.svelte";
  import SnapshotGroupSourcesView from "$lib/components/analysis/snapshot-group-sources-view.svelte";
  import SnapshotItemsView from "$lib/components/analysis/snapshot-items-view.svelte";
  import SourceMetadataView from "$lib/components/analysis/source-metadata-view.svelte";
  import TelegramTimelineReader from "$lib/components/analysis/telegram-timeline-reader.svelte";
  import UniversalItemsView from "$lib/components/analysis/universal-items-view.svelte";
  import YoutubeCommentsView from "$lib/components/analysis/youtube-comments-view.svelte";
  import YoutubePlaylistVideosView from "$lib/components/analysis/youtube-playlist-videos-view.svelte";
  import YoutubeTranscriptReader from "$lib/components/analysis/youtube-transcript-reader.svelte";
  import {
    reconcileSourceBrowserTab,
    sourceBrowserTabsForSubject,
    type SourceBrowserSubject,
    type SourceBrowserTabId,
  } from "$lib/source-browser-model";
  import type { EvidenceHighlightToken } from "$lib/analysis-evidence-source-navigation";
  import type { RunSnapshotAvailability } from "$lib/analysis-report-canvas-state";
  import { liveSourceItemRef, type SourceFilterOption, type SourceReaderItem } from "$lib/source-reader-model";
  import type { AnalysisRunDetail } from "$lib/types/analysis";
  import type {
    Source,
    SourceForumTopic,
    SourceItem,
    SourceJobRecord,
    TakeoutImportRecoveryState,
    TelegramHistoryScope,
    YoutubeTranscriptSegment,
  } from "$lib/types/sources";
  import type { YoutubePlaylistDetail, YoutubeVideoDetail } from "$lib/types/youtube";

  type SourceGroupBrowserData = {
    liveReaderItems: SourceReaderItem[];
    sourceItems: SourceItem[];
    selectedSourceId: number | null;
    hasMoreBySource: Record<number, boolean>;
    sourceLabelForItem: (item: SourceItem) => string | null;
    onLoadSourcePage: (sourceId: number) => void | Promise<void>;
    youtubeDetailsBySource: Record<number, YoutubeVideoDetail | null>;
  };

  type SnapshotBrowserData = {
    run: AnalysisRunDetail;
    readerItems: SourceReaderItem[];
    selectedSourceId: number | null;
    sourceOptions: SourceFilterOption[];
    loading: boolean;
    hasMore: boolean;
    availability: RunSnapshotAvailability;
    error: string;
    selectedTraceRef: string | null;
    onLoadMore: () => void | Promise<void>;
  };

  type SourceBrowserData = {
    liveReaderItems: SourceReaderItem[];
    sourceItems: SourceItem[];
    sourceRouteError: string | null;
    sourceItemsHasMore: boolean;
    loadingItems: boolean;
    sourceTopics: SourceForumTopic[];
    loadingSourceTopics: boolean;
    selectedTopicKey: string;
    showTopicSelector: boolean;
    youtubeVideoDetail: YoutubeVideoDetail | null;
    youtubePlaylistDetail: YoutubePlaylistDetail | null;
    youtubeTranscriptSegments: YoutubeTranscriptSegment[];
    youtubeTranscriptSearch: string;
    youtubeTranscriptHasMore: boolean;
    loadingYoutubeTranscriptSegments: boolean;
    loadingYoutubeDetail: boolean;
    sourceJobs: SourceJobRecord[];
    takeoutRecovery: TakeoutImportRecoveryState | null;
    sourceSyncDisabledReason: (source: Source) => string | null;
    telegramHistoryScope: TelegramHistoryScope;
    currentSourceContentLabel: string;
    onLoadMoreSourceItems: () => void | Promise<void>;
    onChangeSelectedTopicKey: (value: string) => void | Promise<void>;
    onChangeTelegramHistoryScope: (scope: TelegramHistoryScope) => void;
    onChangeTranscriptSearch: (value: string) => void;
    onLoadMoreYoutubeTranscriptSegments: () => void | Promise<void>;
    onOpenSource: (sourceId: number) => void | Promise<void>;
    onSyncSource: (sourceId: number) => void | Promise<void>;
    onSyncYoutubeMetadata: (sourceId: number) => void | Promise<void>;
    onSyncYoutubeTranscript: (sourceId: number) => void | Promise<void>;
    onSyncYoutubeComments: (sourceId: number) => void | Promise<void>;
    onSyncYoutubePlaylist: (sourceId: number) => void | Promise<void>;
    onRetryFailedYoutubePlaylistVideos: (sourceId: number) => void | Promise<void>;
    onSyncYoutubePlaylistVideo: (playlistSourceId: number, videoSourceId: number) => void | Promise<void>;
    onRetryYoutubePlaylistVideo: (playlistSourceId: number, videoSourceId: number) => void | Promise<void>;
    onStartTakeoutImport: (sourceId: number) => void | Promise<void>;
    onStartMigratedHistoryImport: (sourceId: number) => void | Promise<void>;
    onCancelSourceJob: (jobId: string) => void | Promise<void>;
  };

  type Props = {
    subject?: SourceBrowserSubject | null;
    sourceBrowserData?: SourceBrowserData | null;
    groupBrowserData?: SourceGroupBrowserData | null;
    snapshotBrowserData?: SnapshotBrowserData | null;
    selectedTraceRef?: string | null;
    highlightToken?: EvidenceHighlightToken | null;
    loadingItems?: boolean;
    formatTimestamp: (value: number | null) => string;
  };

  let {
    subject: explicitSubject = null,
    sourceBrowserData = null,
    groupBrowserData = null,
    snapshotBrowserData = null,
    selectedTraceRef = null,
    highlightToken = null,
    loadingItems = false,
    formatTimestamp,
  }: Props = $props();

  let activeTab = $state<SourceBrowserTabId | null>(null);
  let lastSubjectKey = $state<string | null>(null);
  let lastHighlightTabTokenId = $state<string | null>(null);
  const subject = $derived(explicitSubject);
  const tabs = $derived(subject ? sourceBrowserTabsForSubject(subject) : []);
  const sourceSubject = $derived(subject && subject.kind === "source" ? subject.source : null);
  const groupSubject = $derived(subject && subject.kind === "source_group" ? subject.group : null);
  const snapshotSubject = $derived(subject && subject.kind === "run_snapshot" ? subject.snapshot : null);
  const sourceData = $derived(subject && subject.kind === "source" ? sourceBrowserData : null);
  const groupData = $derived(subject && subject.kind === "source_group" ? groupBrowserData : null);
  const snapshotData = $derived(subject && subject.kind === "run_snapshot" ? snapshotBrowserData : null);
  const groupLoading = $derived(subject && subject.kind === "source_group" ? loadingItems : false);
  const subjectKey = $derived(
    subject
      ? subject.kind === "source"
        ? `source:${subject.source.id}`
        : subject.kind === "source_group"
          ? `source_group:${subject.group.id}`
          : `run_snapshot:${subject.snapshot.runId}:${subject.snapshot.readerKind}`
      : null,
  );
  const itemsForActiveSubject = $derived(groupData?.sourceItems ?? sourceData?.sourceItems ?? []);
  const itemsLoading = $derived(
    subject && subject.kind === "source_group" ? groupLoading : sourceData?.loadingItems ?? false,
  );
  const itemsHasMore = $derived(
    subject && subject.kind === "source_group" ? false : sourceData?.sourceItemsHasMore ?? false,
  );
  const itemsEmptyDescription = $derived(
    subject && subject.kind === "source_group"
      ? "Group items are limited to the source rows loaded in this browser session. Use Sources to load more rows for each member source."
      : sourceSubject?.sourceType === "youtube" && sourceSubject.sourceSubtype === "playlist"
        ? "Playlist videos live in the Videos tab. This Items tab only shows generic archived items loaded for this playlist source."
        : "No loaded items are available for this source window.",
  );
  const sortedSourceTopics = $derived(sourceSubject && sourceData
    ? [...sourceData.sourceTopics].sort(compareTopics)
    : []);
  const telegramHistoryScopeOptions = $derived.by(() => {
    if (!sourceSubject || sourceSubject.sourceType !== "telegram") return [];
    if (sourceSubject.migratedHistoryRowCount <= 0) return [];
    return [
      { value: "current" as const, label: "Current supergroup history" },
      { value: "migrated" as const, label: "Migrated small-group history" },
      { value: "merged" as const, label: "Merged timeline" },
    ];
  });

  $effect(() => {
    if (!subject) return;
    if (lastSubjectKey !== subjectKey || !activeTab || !tabs.some((tab) => tab.id === activeTab)) {
      activeTab = reconcileSourceBrowserTab(activeTab, subject);
      lastSubjectKey = subjectKey;
    }
  });

  $effect(() => {
    if (!highlightToken || lastHighlightTabTokenId === highlightToken.tokenId) return;
    const targetTab = highlightTabForToken();
    if (!targetTab) return;
    lastHighlightTabTokenId = highlightToken.tokenId;
    activeTab = targetTab;
  });

  function compareTopics(left: SourceForumTopic, right: SourceForumTopic) {
    if (left.kind !== right.kind) return left.kind === "topic" ? -1 : 1;
    if (left.isDeleted !== right.isDeleted) return left.isDeleted ? 1 : -1;
    const titleOrder = left.title.localeCompare(right.title, undefined, {
      sensitivity: "base",
      numeric: true,
    });
    return titleOrder || left.key.localeCompare(right.key, undefined, {
      sensitivity: "base",
      numeric: true,
    });
  }

  function changeSelectedTopic(event: Event) {
    return sourceData?.onChangeSelectedTopicKey((event.currentTarget as HTMLSelectElement).value);
  }

  function changeTelegramHistoryScope(event: Event) {
    return sourceData?.onChangeTelegramHistoryScope(
      (event.currentTarget as HTMLSelectElement).value as TelegramHistoryScope,
    );
  }

  function loadMoreGroupItems() {
    return undefined;
  }

  function loadMoreSourceItems() {
    return sourceData?.onLoadMoreSourceItems();
  }

  function loadMoreGroupSourcePage(sourceId: number) {
    return groupData?.onLoadSourcePage(sourceId);
  }

  function highlightTabForToken(): SourceBrowserTabId | null {
    if (!highlightToken || !subject) return null;

    if (subject.kind === "source" && sourceData && sourceSubject) {
      const matchingItem = sourceData.sourceItems.find((item) => liveSourceItemRef(item) === highlightToken.traceRef);
      if (!matchingItem) return null;
      if (
        sourceSubject.sourceType === "telegram" &&
        matchingItem.itemKind === "telegram_message" &&
        tabAvailable("timeline")
      ) {
        return "timeline";
      }
      if (
        sourceSubject.sourceType === "youtube" &&
        sourceSubject.sourceSubtype === "video" &&
        (matchingItem.youtubeComment || matchingItem.itemKind === "youtube_comment")
      ) {
        return tabAvailable("comments") ? "comments" : null;
      }
      return tabAvailable("items") ? "items" : null;
    }

    if (subject.kind === "source_group" && groupData) {
      const matchingGroupItem = groupData.sourceItems.some((item) => liveSourceItemRef(item) === highlightToken.traceRef)
        || groupData.liveReaderItems.some((item) => item.ref === highlightToken.traceRef);
      if (!matchingGroupItem) return null;
      return tabAvailable("sources") ? "sources" : null;
    }

    if (subject.kind === "run_snapshot" && snapshotData && snapshotSubject) {
      const matchingSnapshotItem = snapshotData.readerItems.some((item) => item.ref === highlightToken.traceRef);
      if (!matchingSnapshotItem) return null;
      if (snapshotSubject.readerKind === "source_group") {
        return tabAvailable("sources") ? "sources" : null;
      }
      if (snapshotSubject.readerKind === "telegram_timeline") {
        return tabAvailable("timeline") ? "timeline" : null;
      }
      if (snapshotSubject.readerKind === "youtube_transcript") {
        return tabAvailable("transcript") ? "transcript" : null;
      }
      return tabAvailable("items") ? "items" : null;
    }

    return null;
  }

  function tabAvailable(tabId: SourceBrowserTabId) {
    return tabs.some((tab) => tab.id === tabId);
  }
</script>

<section class="source-browser-shell">
  <nav class="source-browser-tabs" aria-label="Source browser tabs" data-smoke-id="source-browser-tabs">
    {#each tabs as tab (tab.id)}
      <Button
        type="button"
        variant={activeTab === tab.id ? "primary" : "ghost"}
        ariaSelected={activeTab === tab.id}
        onclick={() => (activeTab = tab.id)}
      >
        {tab.label}
      </Button>
    {/each}
  </nav>

  {#if activeTab === "sources" && snapshotSubject?.readerKind === "source_group"}
    <SnapshotGroupSourcesView
      items={snapshotData?.readerItems ?? []}
      selectedGroupSourceId={snapshotData?.selectedSourceId ?? null}
      loading={snapshotData?.loading ?? false}
      hasMore={snapshotData?.hasMore ?? false}
      selectedTraceRef={snapshotData?.selectedTraceRef ?? selectedTraceRef}
      {highlightToken}
      {formatTimestamp}
      onLoadMore={snapshotData?.onLoadMore ?? (() => {})}
    />
  {:else if activeTab === "items" && subject?.kind === "run_snapshot"}
    <SnapshotItemsView
      items={snapshotData?.readerItems ?? []}
      loading={snapshotData?.loading ?? false}
      hasMore={snapshotData?.hasMore ?? false}
      selectedTraceRef={snapshotData?.selectedTraceRef ?? selectedTraceRef}
      {highlightToken}
      {formatTimestamp}
      onLoadMore={snapshotData?.onLoadMore ?? (() => {})}
    />
  {:else if activeTab === "metadata" && snapshotSubject && snapshotData}
    <RunSnapshotMetadataView
      run={snapshotData.run}
      snapshot={snapshotSubject}
      readerItems={snapshotData.readerItems}
      sourceOptions={snapshotData.sourceOptions}
      snapshotAvailability={snapshotData.availability}
      snapshotError={snapshotData.error}
      {formatTimestamp}
    />
  {:else if activeTab === "timeline" && snapshotSubject?.readerKind === "telegram_timeline"}
    <TelegramTimelineReader
      items={snapshotData?.readerItems ?? []}
      loading={snapshotData?.loading ?? false}
      hasMore={snapshotData?.hasMore ?? false}
      ariaLabel="Run snapshot source material timeline"
      {highlightToken}
      {formatTimestamp}
      onLoadMore={snapshotData?.onLoadMore ?? (() => {})}
    />
  {:else if activeTab === "transcript" && snapshotSubject?.readerKind === "youtube_transcript"}
    <YoutubeTranscriptReader
      detail={null}
      segments={[]}
      snapshotItems={snapshotData?.readerItems ?? []}
      loading={snapshotData?.loading ?? false}
      hasMore={snapshotData?.hasMore ?? false}
      transcriptSearch=""
      showSyncActions={false}
      sourceTitle={snapshotSubject.scopeLabel}
      selectedTraceRef={snapshotData?.selectedTraceRef ?? selectedTraceRef}
      {highlightToken}
      {formatTimestamp}
      onChangeTranscriptSearch={() => {}}
      onLoadMore={snapshotData?.onLoadMore ?? (() => {})}
      onSyncTranscript={() => {}}
      onSyncMetadata={() => {}}
    />
  {:else if activeTab === "sources" && groupSubject}
    <SourceGroupSourcesView
      items={groupData?.liveReaderItems ?? []}
      selectedGroupSourceId={groupData?.selectedSourceId ?? null}
      loading={groupLoading}
      hasMoreBySource={groupData?.hasMoreBySource ?? {}}
      youtubeDetailsBySource={groupData?.youtubeDetailsBySource ?? {}}
      {selectedTraceRef}
      {highlightToken}
      {formatTimestamp}
      onLoadMoreSource={loadMoreGroupSourcePage}
    />
  {:else if activeTab === "timeline" && sourceSubject && sourceData}
    {#if sourceSubject.sourceType === "telegram" && sourceSubject.migratedHistoryStatus === "available" && !sourceSubject.migratedHistoryImportCompleted}
      <StatusMessage tone="info">
        Migrated small-group history is detected but has not been imported for browsing yet.
      </StatusMessage>
    {/if}
    {#if telegramHistoryScopeOptions.length > 0}
      <label class="history-scope-control">
        <span>History scope</span>
        <Select value={sourceData.telegramHistoryScope} onchange={changeTelegramHistoryScope}>
          {#each telegramHistoryScopeOptions as option (option.value)}
            <option value={option.value}>{option.label}</option>
          {/each}
        </Select>
      </label>
    {/if}
    {#if sourceData.showTopicSelector && sourceData.telegramHistoryScope === "current"}
      <label class="topic-filter">
        <span>Topic view</span>
        <Select value={sourceData.selectedTopicKey} disabled={sourceData.loadingSourceTopics} onchange={changeSelectedTopic}>
          <option value="__all_topics__">All topics</option>
          {#if sourceData.loadingSourceTopics && sourceData.sourceTopics.length === 0}
            <option value="__loading_topics__" disabled>Loading topics...</option>
          {:else}
            {#each sortedSourceTopics as topic (topic.key)}
              <option value={topic.key}>{topic.title} ({topic.messageCount})</option>
            {/each}
          {/if}
        </Select>
      </label>
    {/if}
    {#if sourceSubject.sourceType === "telegram" && sourceSubject.migratedHistoryImportCompleted && sourceSubject.migratedHistoryRowCount === 0 && sourceData.telegramHistoryScope !== "current"}
      <StatusMessage tone="info">
        Migrated history import completed with no browsable migrated rows for this source.
      </StatusMessage>
    {:else}
      <TelegramTimelineReader
        items={sourceData.liveReaderItems}
        loading={sourceData.loadingItems}
        hasMore={sourceData.sourceItemsHasMore}
        contentLabel={sourceData.currentSourceContentLabel}
        {highlightToken}
        {formatTimestamp}
        onLoadMore={sourceData.onLoadMoreSourceItems}
      />
    {/if}
  {:else if activeTab === "transcript" && sourceSubject && sourceData}
    <YoutubeTranscriptReader
      detail={sourceData.youtubeVideoDetail}
      segments={sourceData.youtubeTranscriptSegments}
      snapshotItems={[]}
      loading={sourceData.loadingYoutubeTranscriptSegments || sourceData.loadingYoutubeDetail}
      hasMore={sourceData.youtubeTranscriptHasMore}
      transcriptSearch={sourceData.youtubeTranscriptSearch}
      sourceTitle={sourceSubject.title ?? sourceSubject.externalId}
      {selectedTraceRef}
      {highlightToken}
      {formatTimestamp}
      onChangeTranscriptSearch={sourceData.onChangeTranscriptSearch}
      onLoadMore={sourceData.onLoadMoreYoutubeTranscriptSegments}
      onSyncTranscript={() => sourceData.onSyncYoutubeTranscript(sourceSubject.id)}
      onSyncMetadata={() => sourceData.onSyncYoutubeMetadata(sourceSubject.id)}
      onSyncComments={() => sourceData.onSyncYoutubeComments(sourceSubject.id)}
    />
  {:else if activeTab === "videos" && sourceSubject && sourceData}
    <YoutubePlaylistVideosView
      sourceTitle={sourceSubject.title ?? sourceSubject.externalId}
      playlist={sourceData.youtubePlaylistDetail}
      loading={sourceData.loadingYoutubeDetail}
      {formatTimestamp}
      onOpenSource={sourceData.onOpenSource}
      onSyncPlaylist={() => sourceData.onSyncYoutubePlaylist(sourceSubject.id)}
      onRetryFailedPlaylistVideos={() => sourceData.onRetryFailedYoutubePlaylistVideos(sourceSubject.id)}
      onSyncPlaylistVideo={(videoSourceId) => sourceData.onSyncYoutubePlaylistVideo(sourceSubject.id, videoSourceId)}
      onRetryPlaylistVideo={(videoSourceId) => sourceData.onRetryYoutubePlaylistVideo(sourceSubject.id, videoSourceId)}
    />
  {:else if activeTab === "activity" && groupSubject}
    <SourceGroupActivityView />
  {:else if activeTab === "activity" && sourceSubject && sourceData}
    <SourceActivityView
      source={sourceSubject}
      jobs={sourceData.sourceJobs}
      takeoutRecovery={sourceData.takeoutRecovery}
      sourceSyncDisabledReason={sourceData.sourceSyncDisabledReason}
      {formatTimestamp}
      onSyncSource={() => sourceData.onSyncSource(sourceSubject.id)}
      onSyncMetadata={() => sourceData.onSyncYoutubeMetadata(sourceSubject.id)}
      onSyncTranscript={() => sourceData.onSyncYoutubeTranscript(sourceSubject.id)}
      onSyncComments={() => sourceData.onSyncYoutubeComments(sourceSubject.id)}
      onStartTakeoutImport={() => sourceData.onStartTakeoutImport(sourceSubject.id)}
      onStartMigratedHistoryImport={() => sourceData.onStartMigratedHistoryImport(sourceSubject.id)}
      onCancelSourceJob={sourceData.onCancelSourceJob}
    />
  {:else if activeTab === "items"}
    <UniversalItemsView
      items={itemsForActiveSubject}
      loading={itemsLoading}
      hasMore={itemsHasMore}
      emptyDescription={itemsEmptyDescription}
      helpDescription={subject && subject.kind === "source_group" ? itemsEmptyDescription : null}
      sourceLabelForItem={subject && subject.kind === "source_group" ? groupData?.sourceLabelForItem ?? null : null}
      {highlightToken}
      {formatTimestamp}
      onLoadMore={subject && subject.kind === "source_group" ? loadMoreGroupItems : loadMoreSourceItems}
    />
  {:else if activeTab === "comments" && sourceSubject && sourceData}
    <YoutubeCommentsView
      items={sourceData.sourceItems}
      detail={sourceData.youtubeVideoDetail}
      sourceJobs={sourceData.sourceJobs}
      routeError={sourceData.sourceRouteError}
      loading={sourceData.loadingItems}
      hasMore={sourceData.sourceItemsHasMore}
      {highlightToken}
      {formatTimestamp}
      onLoadMore={sourceData.onLoadMoreSourceItems}
      onSyncComments={() => sourceData.onSyncYoutubeComments(sourceSubject.id)}
      onSyncMetadata={() => sourceData.onSyncYoutubeMetadata(sourceSubject.id)}
    />
  {:else if activeTab === "metadata" && groupSubject}
    <SourceGroupMetadataView group={groupSubject} {formatTimestamp} />
  {:else if activeTab === "metadata" && sourceSubject && sourceData}
    <SourceMetadataView
      source={sourceSubject}
      youtubeVideoDetail={sourceData.youtubeVideoDetail}
      youtubePlaylistDetail={sourceData.youtubePlaylistDetail}
      sourceTopics={sourceData.sourceTopics}
      loading={sourceData.loadingYoutubeDetail}
      {formatTimestamp}
      onSyncMetadata={() => sourceData.onSyncYoutubeMetadata(sourceSubject.id)}
    />
  {:else}
    <StatusMessage tone="muted">
      {activeTab} source browser tab is disabled in this review slice. Loaded rows: {snapshotData?.readerItems.length ?? itemsForActiveSubject.length}.
    </StatusMessage>
  {/if}
</section>

<style>
  .source-browser-shell {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    min-width: 0;
  }

  .source-browser-tabs {
    display: flex;
    flex-wrap: wrap;
    gap: 0.45rem;
    align-items: center;
  }

  .history-scope-control,
  .topic-filter {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    align-self: flex-start;
    min-width: min(18rem, 100%);
    color: var(--muted);
    font-size: 0.74rem;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .history-scope-control :global(select),
  .topic-filter :global(select) {
    min-width: 14rem;
    text-transform: none;
    letter-spacing: 0;
    font-size: 0.9rem;
    color: var(--text);
  }
</style>
