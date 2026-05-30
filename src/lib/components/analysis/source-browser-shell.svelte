<script lang="ts">
  import Button from "$lib/components/ui/Button.svelte";
  import Select from "$lib/components/ui/Select.svelte";
  import StatusMessage from "$lib/components/ui/StatusMessage.svelte";
  import SourceActivityView from "$lib/components/analysis/source-activity-view.svelte";
  import SourceGroupActivityView from "$lib/components/analysis/source-group-activity-view.svelte";
  import SourceGroupMetadataView from "$lib/components/analysis/source-group-metadata-view.svelte";
  import SourceGroupSourcesView from "$lib/components/analysis/source-group-sources-view.svelte";
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

  type SourceGroupBrowserData = {
    liveReaderItems: SourceReaderItem[];
    sourceItems: SourceItem[];
    selectedSourceId: number | null;
    hasMoreBySource: Record<number, boolean>;
    sourceLabelForItem: (item: SourceItem) => string | null;
    onLoadSourcePage: (sourceId: number) => void | Promise<void>;
    youtubeDetailsBySource: Record<number, YoutubeVideoDetail | null>;
  };

  type Props = {
    subject?: SourceBrowserSubject | null;
    source: Source | null;
    groupBrowserData?: SourceGroupBrowserData | null;
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
    subject: explicitSubject = null,
    source,
    groupBrowserData = null,
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
  let lastSubjectKey = $state<string | null>(null);
  const subject = $derived(explicitSubject ?? (source ? { kind: "source" as const, source } : null));
  const tabs = $derived(subject ? sourceBrowserTabsForSubject(subject) : []);
  const sourceSubject = $derived(subject && subject.kind === "source" ? subject.source : null);
  const groupSubject = $derived(subject && subject.kind === "source_group" ? subject.group : null);
  const groupData = $derived(subject && subject.kind === "source_group" ? groupBrowserData : null);
  const subjectKey = $derived(
    subject
      ? subject.kind === "source"
        ? `source:${subject.source.id}`
        : `source_group:${subject.group.id}`
      : null,
  );
  const itemsForActiveSubject = $derived(groupData?.sourceItems ?? sourceItems);
  const itemsEmptyDescription = $derived(
    subject && subject.kind === "source_group"
      ? "Group items are limited to the source rows loaded in this browser session. Use Sources to load more rows for each member source."
      : sourceSubject?.sourceType === "youtube" && sourceSubject.sourceSubtype === "playlist"
        ? "Playlist videos live in the Videos tab. This Items tab only shows generic archived items loaded for this playlist source."
        : "No loaded items are available for this source window.",
  );
  const sortedSourceTopics = $derived(sourceSubject ? [...sourceTopics].sort(compareTopics) : []);
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

  function loadMoreGroupItems() {
    return undefined;
  }

  function loadMoreGroupSourcePage(sourceId: number) {
    return groupData?.onLoadSourcePage(sourceId);
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

  {#if activeTab === "sources" && groupSubject}
    <SourceGroupSourcesView
      items={groupData?.liveReaderItems ?? []}
      selectedGroupSourceId={groupData?.selectedSourceId ?? null}
      loading={loadingItems}
      hasMoreBySource={groupData?.hasMoreBySource ?? {}}
      youtubeDetailsBySource={groupData?.youtubeDetailsBySource ?? {}}
      {selectedTraceRef}
      {formatTimestamp}
      onLoadMoreSource={loadMoreGroupSourcePage}
    />
  {:else if activeTab === "timeline" && sourceSubject}
    {#if sourceSubject.sourceType === "telegram" && sourceSubject.migratedHistoryStatus === "available" && !sourceSubject.migratedHistoryImportCompleted}
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
    {#if sourceSubject.sourceType === "telegram" && sourceSubject.migratedHistoryImportCompleted && sourceSubject.migratedHistoryRowCount === 0 && telegramHistoryScope !== "current"}
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
  {:else if activeTab === "transcript" && sourceSubject}
    <YoutubeTranscriptReader
      detail={youtubeVideoDetail}
      segments={youtubeTranscriptSegments}
      snapshotItems={[]}
      loading={loadingYoutubeTranscriptSegments || loadingYoutubeDetail}
      hasMore={youtubeTranscriptHasMore}
      transcriptSearch={youtubeTranscriptSearch}
      sourceTitle={sourceSubject.title ?? sourceSubject.externalId}
      {selectedTraceRef}
      {formatTimestamp}
      onChangeTranscriptSearch={onChangeTranscriptSearch}
      onLoadMore={onLoadMoreYoutubeTranscriptSegments}
      onSyncTranscript={() => onSyncYoutubeTranscript(sourceSubject.id)}
      onSyncMetadata={() => onSyncYoutubeMetadata(sourceSubject.id)}
      onSyncComments={() => onSyncYoutubeComments(sourceSubject.id)}
    />
  {:else if activeTab === "videos" && sourceSubject}
    <YoutubePlaylistVideosView
      sourceTitle={sourceSubject.title ?? sourceSubject.externalId}
      playlist={youtubePlaylistDetail}
      loading={loadingYoutubeDetail}
      {formatTimestamp}
      onOpenSource={onOpenSource}
      onSyncPlaylist={() => onSyncYoutubePlaylist(sourceSubject.id)}
      onRetryFailedPlaylistVideos={() => onRetryFailedYoutubePlaylistVideos(sourceSubject.id)}
      onSyncPlaylistVideo={(videoSourceId) => onSyncYoutubePlaylistVideo(sourceSubject.id, videoSourceId)}
      onRetryPlaylistVideo={(videoSourceId) => onRetryYoutubePlaylistVideo(sourceSubject.id, videoSourceId)}
    />
  {:else if activeTab === "activity" && groupSubject}
    <SourceGroupActivityView />
  {:else if activeTab === "activity" && sourceSubject}
    <SourceActivityView
      source={sourceSubject}
      jobs={sourceJobs}
      takeoutRecovery={takeoutRecovery}
      sourceSyncDisabledReason={sourceSyncDisabledReason}
      {formatTimestamp}
      onSyncSource={() => onSyncSource(sourceSubject.id)}
      onSyncMetadata={() => onSyncYoutubeMetadata(sourceSubject.id)}
      onSyncTranscript={() => onSyncYoutubeTranscript(sourceSubject.id)}
      onSyncComments={() => onSyncYoutubeComments(sourceSubject.id)}
      onStartTakeoutImport={() => onStartTakeoutImport(sourceSubject.id)}
      onStartMigratedHistoryImport={() => onStartMigratedHistoryImport(sourceSubject.id)}
      onCancelSourceJob={onCancelSourceJob}
    />
  {:else if activeTab === "items"}
    <UniversalItemsView
      items={itemsForActiveSubject}
      loading={loadingItems}
      hasMore={subject && subject.kind === "source_group" ? false : sourceItemsHasMore}
      emptyDescription={itemsEmptyDescription}
      sourceLabelForItem={subject && subject.kind === "source_group" ? groupData?.sourceLabelForItem ?? null : null}
      {formatTimestamp}
      onLoadMore={subject && subject.kind === "source_group" ? loadMoreGroupItems : onLoadMoreSourceItems}
    />
  {:else if activeTab === "comments" && sourceSubject}
    <YoutubeCommentsView
      items={sourceItems}
      detail={youtubeVideoDetail}
      {sourceJobs}
      routeError={sourceRouteError}
      loading={loadingItems}
      hasMore={sourceItemsHasMore}
      {formatTimestamp}
      onLoadMore={onLoadMoreSourceItems}
      onSyncComments={() => onSyncYoutubeComments(sourceSubject.id)}
      onSyncMetadata={() => onSyncYoutubeMetadata(sourceSubject.id)}
    />
  {:else if activeTab === "metadata" && groupSubject}
    <SourceGroupMetadataView group={groupSubject} {formatTimestamp} />
  {:else if activeTab === "metadata" && sourceSubject}
    <SourceMetadataView
      source={sourceSubject}
      youtubeVideoDetail={youtubeVideoDetail}
      youtubePlaylistDetail={youtubePlaylistDetail}
      sourceTopics={sourceTopics}
      loading={loadingYoutubeDetail}
      {formatTimestamp}
      onSyncMetadata={() => onSyncYoutubeMetadata(sourceSubject.id)}
    />
  {:else}
    <StatusMessage tone="muted">
      {activeTab} source browser tab is disabled in this review slice. Loaded rows: {itemsForActiveSubject.length}.
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
