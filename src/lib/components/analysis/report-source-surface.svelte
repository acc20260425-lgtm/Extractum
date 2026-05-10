<script lang="ts">
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import StatusMessage from "$lib/components/ui/StatusMessage.svelte";
  import RunSnapshotMessagesPanel from "$lib/components/analysis/run-snapshot-messages-panel.svelte";
  import SourceContextPanel from "$lib/components/analysis/source-context-panel.svelte";
  import YoutubePlaylistDetail from "$lib/components/analysis/youtube-playlist-detail.svelte";
  import YoutubeSourceDetail from "$lib/components/analysis/youtube-source-detail.svelte";
  import {
    canReturnToRunSnapshot,
    sourceBasisDescription,
    sourceBasisLabel,
    sourceCanvasSurface,
    type RunSnapshotAvailability,
  } from "$lib/analysis-report-canvas-state";
  import type { SourceViewBasis } from "$lib/analysis-workspace-state";
  import type { AnalysisScope } from "$lib/analysis-scope-state";
  import type {
    AnalysisRunDetail,
    AnalysisRunMessage,
    AnalysisSourceGroup,
    AnalysisSourceOption,
  } from "$lib/types/analysis";
  import type { Source, SourceForumTopic, SourceItem, SourceJobRecord } from "$lib/types/sources";
  import type { YoutubePlaylistDetail as YoutubePlaylistDetailData, YoutubeVideoDetail } from "$lib/types/youtube";

  let {
    currentRun,
    sourceViewBasis,
    snapshotAvailability,
    runSnapshotMessages,
    loadingRunSnapshotMessages,
    runSnapshotError,
    hasMoreRunSnapshotMessages,
    analysisScope,
    currentSource,
    currentGroup,
    currentSourceMetric,
    sourceItems,
    loadingItems,
    sourceTopics,
    loadingSourceTopics,
    selectedTopicKey,
    showTopicSelector,
    currentSourceContentLabel,
    sourceJobs,
    youtubeVideoDetail,
    youtubePlaylistDetail,
    loadingYoutubeDetail,
    formatTimestamp,
    onChangeSelectedTopicKey,
    onOpenSource,
    onSyncYoutubeMetadata,
    onSyncYoutubeTranscript,
    onSyncYoutubeComments,
    onSyncYoutubePlaylist,
    onRetryFailedYoutubePlaylistVideos,
    onSyncYoutubePlaylistVideo,
    onRetryYoutubePlaylistVideo,
    onCancelSourceJob,
    onViewLiveSource,
    onBackToRunSnapshot,
    onLoadMoreRunSnapshotMessages,
  }: {
    currentRun: AnalysisRunDetail | null;
    sourceViewBasis: SourceViewBasis;
    snapshotAvailability: RunSnapshotAvailability;
    runSnapshotMessages: AnalysisRunMessage[];
    loadingRunSnapshotMessages: boolean;
    runSnapshotError: string;
    hasMoreRunSnapshotMessages: boolean;
    analysisScope: AnalysisScope;
    currentSource: Source | null;
    currentGroup: AnalysisSourceGroup | null;
    currentSourceMetric: AnalysisSourceOption | null;
    sourceItems: SourceItem[];
    loadingItems: boolean;
    sourceTopics: SourceForumTopic[];
    loadingSourceTopics: boolean;
    selectedTopicKey: string;
    showTopicSelector: boolean;
    currentSourceContentLabel: string;
    sourceJobs: SourceJobRecord[];
    youtubeVideoDetail: YoutubeVideoDetail | null;
    youtubePlaylistDetail: YoutubePlaylistDetailData | null;
    loadingYoutubeDetail: boolean;
    formatTimestamp: (value: number | null) => string;
    onChangeSelectedTopicKey: (value: string) => void | Promise<void>;
    onOpenSource: (sourceId: number) => void | Promise<void>;
    onSyncYoutubeMetadata: (sourceId: number) => void | Promise<void>;
    onSyncYoutubeTranscript: (sourceId: number) => void | Promise<void>;
    onSyncYoutubeComments: (sourceId: number) => void | Promise<void>;
    onSyncYoutubePlaylist: (sourceId: number) => void | Promise<void>;
    onRetryFailedYoutubePlaylistVideos: (sourceId: number) => void | Promise<void>;
    onSyncYoutubePlaylistVideo: (playlistSourceId: number, videoSourceId: number) => void | Promise<void>;
    onRetryYoutubePlaylistVideo: (playlistSourceId: number, videoSourceId: number) => void | Promise<void>;
    onCancelSourceJob: (jobId: string) => void | Promise<void>;
    onViewLiveSource: () => void | Promise<void>;
    onBackToRunSnapshot: () => void | Promise<void>;
    onLoadMoreRunSnapshotMessages: () => void | Promise<void>;
  } = $props();

  const sourceBasis = $derived({
    currentRun,
    sourceViewBasis,
    snapshotAvailability,
  });
  const canvasSurface = $derived(sourceCanvasSurface(sourceBasis));
</script>

<section class="report-source-surface" data-surface={canvasSurface}>
  <div class="source-basis-header">
    <div>
      <span class="eyebrow">Source</span>
      <h2>{sourceBasisLabel(sourceBasis)}</h2>
      <p>{sourceBasisDescription(sourceBasis)}</p>
    </div>
    <div class="source-basis-actions">
      {#if currentRun && sourceViewBasis === "run_snapshot" && snapshotAvailability !== "available"}
        <Button type="button" variant="secondary" onclick={onViewLiveSource}>View live source</Button>
      {/if}
      {#if currentRun && sourceViewBasis === "live_source" && canReturnToRunSnapshot(snapshotAvailability)}
        <Button type="button" variant="secondary" onclick={onBackToRunSnapshot}>Back to run snapshot</Button>
      {/if}
      {#if sourceViewBasis === "live_source"}
        <Badge variant="warning">Live source</Badge>
      {:else if snapshotAvailability === "available"}
        <Badge variant="success">Run snapshot</Badge>
      {:else if snapshotAvailability === "capturing"}
        <Badge variant="info">Snapshot pending</Badge>
      {:else if snapshotAvailability === "unavailable"}
        <Badge variant="warning">Snapshot unavailable</Badge>
      {:else}
        <Badge variant="neutral">Snapshot status unknown</Badge>
      {/if}
    </div>
  </div>

  {#if currentRun && sourceViewBasis === "run_snapshot"}
    {#if snapshotAvailability === "available"}
      <RunSnapshotMessagesPanel
        messages={runSnapshotMessages}
        {loadingRunSnapshotMessages}
        {hasMoreRunSnapshotMessages}
        {formatTimestamp}
        {onLoadMoreRunSnapshotMessages}
      />
    {:else if snapshotAvailability === "capturing"}
      <StatusMessage tone="muted">Snapshot pending. The frozen source corpus is not browsable yet.</StatusMessage>
    {:else if snapshotAvailability === "unavailable"}
      <StatusMessage>
        Snapshot unavailable. This run ended before Extractum could expose a frozen source snapshot.
      </StatusMessage>
      {#if runSnapshotError}
        <StatusMessage tone="error">{runSnapshotError}</StatusMessage>
      {/if}
    {:else}
      <StatusMessage tone="muted">Checking run snapshot availability...</StatusMessage>
    {/if}
  {:else}
    {@render liveSourceSurface()}
  {/if}
</section>

{#snippet liveSourceSurface()}
  {#if analysisScope === "single_source" && currentSource}
    {#key `${analysisScope}:${currentSource.id}:${currentRun?.id ?? "idle"}:live`}
      {#if currentSource.sourceType === "youtube" && currentSource.sourceSubtype === "video"}
        <YoutubeSourceDetail
          source={currentSource}
          detail={youtubeVideoDetail}
          jobs={sourceJobs}
          loadingDetail={loadingYoutubeDetail}
          {formatTimestamp}
          onSyncMetadata={onSyncYoutubeMetadata}
          onSyncTranscript={onSyncYoutubeTranscript}
          onSyncComments={onSyncYoutubeComments}
          onCancelJob={onCancelSourceJob}
        />
      {:else if currentSource.sourceType === "youtube" && currentSource.sourceSubtype === "playlist"}
        <YoutubePlaylistDetail
          source={currentSource}
          detail={youtubePlaylistDetail}
          jobs={sourceJobs}
          loadingDetail={loadingYoutubeDetail}
          {formatTimestamp}
          onOpenSource={onOpenSource}
          onSyncPlaylist={onSyncYoutubePlaylist}
          onRetryFailed={onRetryFailedYoutubePlaylistVideos}
          onSyncPlaylistVideo={onSyncYoutubePlaylistVideo}
          onRetryPlaylistVideo={onRetryYoutubePlaylistVideo}
          onCancelJob={onCancelSourceJob}
        />
      {:else}
        <SourceContextPanel
          currentRunOpen={!!currentRun}
          {currentSourceMetric}
          {sourceItems}
          {loadingItems}
          {sourceTopics}
          {loadingSourceTopics}
          {selectedTopicKey}
          {showTopicSelector}
          contentLabel={currentSourceContentLabel}
          {formatTimestamp}
          onChangeSelectedTopicKey={onChangeSelectedTopicKey}
        />
      {/if}
    {/key}
  {:else if analysisScope === "source_group" && currentGroup}
    <StatusMessage tone="muted" surface={false}>
      Source group live browsing remains summarized in this part. Full group readers are implemented in Part 5.
    </StatusMessage>
  {:else}
    <StatusMessage tone="muted" surface={false}>Select a source or source group to browse source material.</StatusMessage>
  {/if}
{/snippet}

<style>
  .report-source-surface {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    min-width: 0;
  }

  .source-basis-header {
    display: flex;
    justify-content: space-between;
    gap: 1rem;
    align-items: flex-start;
    padding: 1rem;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--panel);
  }

  .source-basis-header h2 {
    margin: 0.15rem 0 0;
    font-size: 1.05rem;
  }

  .source-basis-header p {
    margin: 0.35rem 0 0;
    color: var(--muted);
    line-height: 1.45;
  }

  .source-basis-actions {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    justify-content: flex-end;
    flex-wrap: wrap;
  }

  .eyebrow {
    display: inline-block;
    font-size: 0.68rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--muted);
  }

  @media (max-width: 760px) {
    .source-basis-header {
      flex-direction: column;
    }

    .source-basis-actions {
      justify-content: flex-start;
    }
  }
</style>
