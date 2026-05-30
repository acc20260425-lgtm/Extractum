<script lang="ts">
  import Button from "$lib/components/ui/Button.svelte";
  import Select from "$lib/components/ui/Select.svelte";
  import StatusMessage from "$lib/components/ui/StatusMessage.svelte";
  import SourceActivityView from "$lib/components/analysis/source-activity-view.svelte";
  import SourceMetadataView from "$lib/components/analysis/source-metadata-view.svelte";
  import TelegramTimelineReader from "$lib/components/analysis/telegram-timeline-reader.svelte";
  import UniversalItemsView from "$lib/components/analysis/universal-items-view.svelte";
  import YoutubeCommentsView from "$lib/components/analysis/youtube-comments-view.svelte";
  import YoutubePlaylistVideosView from "$lib/components/analysis/youtube-playlist-videos-view.svelte";
  import YoutubeTranscriptReader from "$lib/components/analysis/youtube-transcript-reader.svelte";
  import {
    reconcileSourceBrowserTab,
    sourceBrowserTabsForSource,
    type SourceBrowserTabId,
  } from "$lib/source-browser-model";
  import type { SourceReaderItem } from "$lib/source-reader-model";
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

  type Props = {
    source: Source;
    liveReaderItems: SourceReaderItem[];
    takeoutRecovery: TakeoutImportRecoveryState | null;
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
    selectedTraceRef: string | null;
    telegramHistoryScope: TelegramHistoryScope;
    currentSourceContentLabel: string;
    sourceSyncDisabledReason: (source: Source) => string | null;
    formatTimestamp: (value: number | null) => string;
    onSyncSource: (sourceId: number) => void | Promise<void>;
    onLoadMoreSourceItems: () => void | Promise<void>;
    onChangeSelectedTopicKey: (value: string) => void | Promise<void>;
    onChangeTelegramHistoryScope: (scope: TelegramHistoryScope) => void;
    onChangeTranscriptSearch: (value: string) => void;
    onLoadMoreYoutubeTranscriptSegments: () => void | Promise<void>;
    onOpenSource: (sourceId: number) => void | Promise<void>;
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

  let {
    source,
    liveReaderItems,
    takeoutRecovery,
    sourceItems,
    sourceRouteError,
    sourceItemsHasMore,
    loadingItems,
    sourceTopics,
    loadingSourceTopics,
    selectedTopicKey,
    showTopicSelector,
    youtubeVideoDetail,
    youtubePlaylistDetail,
    youtubeTranscriptSegments,
    youtubeTranscriptSearch,
    youtubeTranscriptHasMore,
    loadingYoutubeTranscriptSegments,
    loadingYoutubeDetail,
    sourceJobs,
    selectedTraceRef,
    telegramHistoryScope,
    currentSourceContentLabel,
    sourceSyncDisabledReason,
    formatTimestamp,
    onSyncSource,
    onLoadMoreSourceItems,
    onChangeSelectedTopicKey,
    onChangeTelegramHistoryScope,
    onChangeTranscriptSearch,
    onLoadMoreYoutubeTranscriptSegments,
    onOpenSource,
    onSyncYoutubeMetadata,
    onSyncYoutubeTranscript,
    onSyncYoutubeComments,
    onSyncYoutubePlaylist,
    onRetryFailedYoutubePlaylistVideos,
    onSyncYoutubePlaylistVideo,
    onRetryYoutubePlaylistVideo,
    onStartTakeoutImport,
    onStartMigratedHistoryImport,
    onCancelSourceJob,
  }: Props = $props();

  let activeTab = $state<SourceBrowserTabId | null>(null);
  let lastSourceId = $state<number | null>(null);
  const tabs = $derived(sourceBrowserTabsForSource(source));
  const sortedSourceTopics = $derived([...sourceTopics].sort(compareTopics));
  const telegramHistoryScopeOptions = $derived.by(() => {
    if (source.sourceType !== "telegram") return [];
    if (source.migratedHistoryRowCount <= 0) return [];
    return [
      { value: "current" as const, label: "Current supergroup history" },
      { value: "migrated" as const, label: "Migrated small-group history" },
      { value: "merged" as const, label: "Merged timeline" },
    ];
  });

  $effect(() => {
    if (lastSourceId !== source.id || !activeTab || !tabs.some((tab) => tab.id === activeTab)) {
      activeTab = reconcileSourceBrowserTab(activeTab, source);
      lastSourceId = source.id;
    }
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
    onChangeSelectedTopicKey((event.currentTarget as HTMLSelectElement).value);
  }

  function changeTelegramHistoryScope(event: Event) {
    onChangeTelegramHistoryScope((event.currentTarget as HTMLSelectElement).value as TelegramHistoryScope);
  }
</script>

<section class="source-browser-shell">
  <nav class="source-browser-tabs" aria-label="Source browser tabs">
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

  {#if activeTab === "timeline"}
    {#if source.sourceType === "telegram" && source.migratedHistoryStatus === "available" && !source.migratedHistoryImportCompleted}
      <StatusMessage tone="info">
        Migrated small-group history is detected but has not been imported for browsing yet.
      </StatusMessage>
    {/if}
    {#if telegramHistoryScopeOptions.length > 0}
      <label class="history-scope-control">
        <span>History scope</span>
        <Select value={telegramHistoryScope} onchange={changeTelegramHistoryScope}>
          {#each telegramHistoryScopeOptions as option (option.value)}
            <option value={option.value}>{option.label}</option>
          {/each}
        </Select>
      </label>
    {/if}
    {#if showTopicSelector && telegramHistoryScope === "current"}
      <label class="topic-filter">
        <span>Topic view</span>
        <Select value={selectedTopicKey} disabled={loadingSourceTopics} onchange={changeSelectedTopic}>
          <option value="__all_topics__">All topics</option>
          {#if loadingSourceTopics && sourceTopics.length === 0}
            <option value="__loading_topics__" disabled>Loading topics...</option>
          {:else}
            {#each sortedSourceTopics as topic (topic.key)}
              <option value={topic.key}>{topic.title} ({topic.messageCount})</option>
            {/each}
          {/if}
        </Select>
      </label>
    {/if}
    {#if source.sourceType === "telegram" && source.migratedHistoryImportCompleted && source.migratedHistoryRowCount === 0 && telegramHistoryScope !== "current"}
      <StatusMessage tone="info">
        Migrated history import completed with no browsable migrated rows for this source.
      </StatusMessage>
    {:else}
      <TelegramTimelineReader
        items={liveReaderItems}
        loading={loadingItems}
        hasMore={sourceItemsHasMore}
        contentLabel={currentSourceContentLabel}
        {formatTimestamp}
        onLoadMore={onLoadMoreSourceItems}
      />
    {/if}
  {:else if activeTab === "transcript"}
    <YoutubeTranscriptReader
      detail={youtubeVideoDetail}
      segments={youtubeTranscriptSegments}
      snapshotItems={[]}
      loading={loadingYoutubeTranscriptSegments || loadingYoutubeDetail}
      hasMore={youtubeTranscriptHasMore}
      transcriptSearch={youtubeTranscriptSearch}
      sourceTitle={source.title ?? source.externalId}
      {selectedTraceRef}
      {formatTimestamp}
      onChangeTranscriptSearch={onChangeTranscriptSearch}
      onLoadMore={onLoadMoreYoutubeTranscriptSegments}
      onSyncTranscript={() => onSyncYoutubeTranscript(source.id)}
      onSyncMetadata={() => onSyncYoutubeMetadata(source.id)}
      onSyncComments={() => onSyncYoutubeComments(source.id)}
    />
  {:else if activeTab === "videos"}
    <YoutubePlaylistVideosView
      sourceTitle={source.title ?? source.externalId}
      playlist={youtubePlaylistDetail}
      loading={loadingYoutubeDetail}
      {formatTimestamp}
      onOpenSource={onOpenSource}
      onSyncPlaylist={() => onSyncYoutubePlaylist(source.id)}
      onRetryFailedPlaylistVideos={() => onRetryFailedYoutubePlaylistVideos(source.id)}
      onSyncPlaylistVideo={(videoSourceId) => onSyncYoutubePlaylistVideo(source.id, videoSourceId)}
      onRetryPlaylistVideo={(videoSourceId) => onRetryYoutubePlaylistVideo(source.id, videoSourceId)}
    />
  {:else if activeTab === "activity"}
    <SourceActivityView
      source={source}
      jobs={sourceJobs}
      takeoutRecovery={takeoutRecovery}
      sourceSyncDisabledReason={sourceSyncDisabledReason}
      {formatTimestamp}
      onSyncSource={() => onSyncSource(source.id)}
      onSyncMetadata={() => onSyncYoutubeMetadata(source.id)}
      onSyncTranscript={() => onSyncYoutubeTranscript(source.id)}
      onSyncComments={() => onSyncYoutubeComments(source.id)}
      onStartTakeoutImport={() => onStartTakeoutImport(source.id)}
      onStartMigratedHistoryImport={() => onStartMigratedHistoryImport(source.id)}
      onCancelSourceJob={onCancelSourceJob}
    />
  {:else if activeTab === "items"}
    <UniversalItemsView
      items={sourceItems}
      loading={loadingItems}
      hasMore={sourceItemsHasMore}
      {formatTimestamp}
      onLoadMore={onLoadMoreSourceItems}
    />
  {:else if activeTab === "comments"}
    <YoutubeCommentsView
      items={sourceItems}
      detail={youtubeVideoDetail}
      {sourceJobs}
      routeError={sourceRouteError}
      loading={loadingItems}
      hasMore={sourceItemsHasMore}
      {formatTimestamp}
      onLoadMore={onLoadMoreSourceItems}
      onSyncComments={() => onSyncYoutubeComments(source.id)}
      onSyncMetadata={() => onSyncYoutubeMetadata(source.id)}
    />
  {:else if activeTab === "metadata"}
    <SourceMetadataView
      source={source}
      youtubeVideoDetail={youtubeVideoDetail}
      sourceTopics={sourceTopics}
      loading={loadingYoutubeDetail}
      {formatTimestamp}
      onSyncMetadata={() => onSyncYoutubeMetadata(source.id)}
    />
  {:else}
    <StatusMessage tone="muted">
      {activeTab} source browser tab is disabled in this review slice. Loaded rows: {sourceItems.length}.
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
