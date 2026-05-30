<script lang="ts">
  import StatusMessage from "$lib/components/ui/StatusMessage.svelte";
  import TakeoutRecoveryNotice from "$lib/components/analysis/takeout-recovery-notice.svelte";
  import SourceBrowserShell from "$lib/components/analysis/source-browser-shell.svelte";
  import SourceReaderHeader from "$lib/components/analysis/source-reader-header.svelte";
  import {
    canReturnToRunSnapshot,
    sourceBasisDescription,
    sourceBasisLabel,
    sourceCanvasSurface,
    type RunSnapshotAvailability,
  } from "$lib/analysis-report-canvas-state";
  import {
    legacyScopeFromWorkspaceSelection,
    type SourceViewBasis,
    type WorkspaceSelection,
  } from "$lib/analysis-workspace-state";
  import {
    analysisRunMessageToReaderItem,
    sourceFilterOptionsFromGroupMembers,
    sourceFilterOptionsFromReaderItems,
    sourceItemToReaderItem,
  } from "$lib/source-reader-model";
  import {
    deriveRunSnapshotBrowserKind,
    sourceBrowserShellAppliesToSource,
    sourceBrowserShellAppliesToSubject,
  } from "$lib/source-browser-model";
  import type {
    AnalysisRunDetail,
    AnalysisRunMessage,
    AnalysisSourceGroup,
    AnalysisSourceOption,
  } from "$lib/types/analysis";
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
    currentRun: AnalysisRunDetail | null;
    sourceViewBasis: SourceViewBasis;
    snapshotAvailability: RunSnapshotAvailability;
    runSnapshotMessages: AnalysisRunMessage[];
    loadingRunSnapshotMessages: boolean;
    runSnapshotError: string;
    hasMoreRunSnapshotMessages: boolean;
    workspaceSelection: WorkspaceSelection;
    currentSource: Source | null;
    takeoutRecovery?: TakeoutImportRecoveryState | null;
    currentGroup: AnalysisSourceGroup | null;
    currentSourceMetric: AnalysisSourceOption | null;
    sourceItems: SourceItem[];
    sourceItemsError: string | null;
    sourceItemsHasMore: boolean;
    loadingItems: boolean;
    sourceTopics: SourceForumTopic[];
    loadingSourceTopics: boolean;
    selectedTopicKey: string;
    showTopicSelector: boolean;
    currentSourceContentLabel: string;
    telegramHistoryScope: TelegramHistoryScope;
    sourceJobs: SourceJobRecord[];
    youtubeVideoDetail: YoutubeVideoDetail | null;
    youtubePlaylistDetail: YoutubePlaylistDetail | null;
    loadingYoutubeDetail: boolean;
    selectedTraceRef?: string | null;
    currentScopeTitle?: string;
    youtubeTranscriptSegments?: YoutubeTranscriptSegment[];
    loadingYoutubeTranscriptSegments?: boolean;
    youtubeTranscriptHasMore?: boolean;
    youtubeTranscriptSearch?: string;
    groupLiveItemsBySource?: Record<number, SourceItem[]>;
    groupLiveHasMoreBySource?: Record<number, boolean>;
    selectedGroupSourceId?: number | null;
    selectedSnapshotSourceId?: number | null;
    formatTimestamp: (value: number | null) => string;
    onChangeSelectedTopicKey: (value: string) => void | Promise<void>;
    onOpenSource: (sourceId: number) => void | Promise<void>;
    onSyncSource: (sourceId: number) => void | Promise<void>;
    onSyncYoutubeMetadata: (sourceId: number) => void | Promise<void>;
    onSyncYoutubeTranscript: (sourceId: number) => void | Promise<void>;
    onSyncYoutubeComments: (sourceId: number) => void | Promise<void>;
    onStartTakeoutImport: (sourceId: number) => void | Promise<void>;
    onStartMigratedHistoryImport: (sourceId: number) => void | Promise<void>;
    onSyncYoutubePlaylist: (sourceId: number) => void | Promise<void>;
    onRetryFailedYoutubePlaylistVideos: (sourceId: number) => void | Promise<void>;
    onSyncYoutubePlaylistVideo: (playlistSourceId: number, videoSourceId: number) => void | Promise<void>;
    onRetryYoutubePlaylistVideo: (playlistSourceId: number, videoSourceId: number) => void | Promise<void>;
    onCancelSourceJob: (jobId: string) => void | Promise<void>;
    onViewLiveSource: () => void | Promise<void>;
    onBackToRunSnapshot: () => void | Promise<void>;
    sourceSyncDisabledReason?: (source: Source) => string | null;
    onLoadMoreRunSnapshotMessages: () => void | Promise<void>;
    onLoadMoreSourceItems?: () => void | Promise<void>;
    onChangeTelegramHistoryScope: (scope: TelegramHistoryScope) => void;
    onChangeTranscriptSearch?: (value: string) => void;
    onLoadMoreYoutubeTranscriptSegments?: () => void | Promise<void>;
    onLoadLiveGroupSourcePage?: (sourceId: number) => void | Promise<void>;
    onChangeSelectedGroupSourceId?: (sourceId: number | null) => void;
    onChangeSelectedSnapshotSourceId?: (sourceId: number | null) => void;
  };

  let {
    currentRun,
    sourceViewBasis,
    snapshotAvailability,
    runSnapshotMessages,
    loadingRunSnapshotMessages,
    runSnapshotError,
    hasMoreRunSnapshotMessages,
    workspaceSelection,
    currentSource,
    takeoutRecovery = null,
    currentGroup,
    sourceItems,
    sourceItemsError,
    sourceItemsHasMore,
    loadingItems,
    sourceTopics,
    loadingSourceTopics,
    selectedTopicKey,
    showTopicSelector,
    currentSourceContentLabel,
    telegramHistoryScope,
    sourceJobs,
    youtubeVideoDetail,
    youtubePlaylistDetail,
    loadingYoutubeDetail,
    selectedTraceRef = null,
    currentScopeTitle,
    youtubeTranscriptSegments = [],
    loadingYoutubeTranscriptSegments = false,
    youtubeTranscriptHasMore = false,
    youtubeTranscriptSearch = "",
    groupLiveItemsBySource = {},
    groupLiveHasMoreBySource = {},
    selectedGroupSourceId = null,
    selectedSnapshotSourceId = null,
    formatTimestamp,
    onOpenSource,
    onChangeSelectedTopicKey,
    onSyncSource,
    onSyncYoutubeMetadata,
    onSyncYoutubeTranscript,
    onSyncYoutubeComments,
    onStartTakeoutImport,
    onStartMigratedHistoryImport,
    onSyncYoutubePlaylist,
    onRetryFailedYoutubePlaylistVideos,
    onSyncYoutubePlaylistVideo,
    onRetryYoutubePlaylistVideo,
    onCancelSourceJob,
    onViewLiveSource,
    onBackToRunSnapshot,
    sourceSyncDisabledReason = () => null,
    onLoadMoreRunSnapshotMessages,
    onLoadMoreSourceItems = () => {},
    onChangeTelegramHistoryScope,
    onChangeTranscriptSearch = () => {},
    onLoadMoreYoutubeTranscriptSegments = () => {},
    onLoadLiveGroupSourcePage = () => {},
    onChangeSelectedGroupSourceId = () => {},
    onChangeSelectedSnapshotSourceId = () => {},
  }: Props = $props();

  const sourceBasis = $derived({
    currentRun,
    sourceViewBasis,
    snapshotAvailability,
  });
  const legacyWorkspaceSelection = $derived(
    legacyScopeFromWorkspaceSelection(workspaceSelection),
  );
  const analysisScope = $derived(legacyWorkspaceSelection.analysisScope);
  const canvasSurface = $derived(sourceCanvasSurface(sourceBasis));
  const liveReaderItems = $derived.by(() =>
    sourceItems.map((item) =>
      sourceItemToReaderItem(item, {
        sourceTitle: currentSource?.title ?? currentSource?.externalId ?? `Source ${item.sourceId}`,
        selectedTraceRef,
      }),
    ),
  );
  const allSnapshotReaderItems = $derived.by(() =>
    runSnapshotMessages
      .map((message) =>
        analysisRunMessageToReaderItem(message, {
          sourceTitle: sourceTitleForSnapshotMessage(message.source_id),
          selectedTraceRef,
        }),
      ),
  );
  const snapshotReaderItems = $derived.by(() =>
    selectedSnapshotSourceId === null
      ? allSnapshotReaderItems
      : allSnapshotReaderItems.filter((item) => item.sourceId === selectedSnapshotSourceId),
  );
  const groupLiveReaderItems = $derived.by(() =>
    Object.entries(groupLiveItemsBySource).flatMap(([sourceId, items]) => {
      const source = groupMemberSource(Number(sourceId));
      const sourceTitle = source?.source_title ?? `Source ${sourceId}`;
      return items.map((item) => sourceItemToReaderItem(item, { sourceTitle, selectedTraceRef }));
    }),
  );
  const groupLiveSourceItems = $derived.by(() =>
    Object.values(groupLiveItemsBySource).flat(),
  );
  const snapshotSourceOptions = $derived.by(() =>
    sourceFilterOptionsFromReaderItems(allSnapshotReaderItems),
  );
  const snapshotSourceType = $derived.by(() => {
    if (currentSource) return currentSource.sourceType;
    const values = new Set(runSnapshotMessages.map((message) => message.source_type).filter(Boolean));
    return values.size === 1 ? Array.from(values)[0] ?? null : null;
  });
  const snapshotSourceSubtype = $derived.by(() => {
    if (currentSource) return currentSource.sourceSubtype;
    const values = new Set(runSnapshotMessages.map((message) => message.source_subtype).filter(Boolean));
    return values.size === 1 ? Array.from(values)[0] ?? null : null;
  });
  const runSnapshotBrowserKind = $derived(
    deriveRunSnapshotBrowserKind({
      scopeType: currentRun?.scope_type ?? null,
      sourceType: snapshotSourceType,
      sourceSubtype: snapshotSourceSubtype,
      snapshotReaderItems: allSnapshotReaderItems,
    }),
  );
  const runSnapshotSubject = $derived(
    currentRun && snapshotAvailability === "available"
      ? {
          kind: "run_snapshot" as const,
          snapshot: {
            runId: currentRun.id,
            scopeType: currentRun.scope_type === "source_group" ? "source_group" as const : "source" as const,
            scopeLabel: currentRun.scope_label,
            readerKind: runSnapshotBrowserKind,
            sourceType: snapshotBrowserSourceType(snapshotSourceType),
            sourceSubtype: snapshotBrowserSourceSubtype(snapshotSourceSubtype),
          },
        }
      : null,
  );
  const liveGroupSourceOptions = $derived.by(() =>
    currentGroup
      ? sourceFilterOptionsFromGroupMembers(currentGroup.members)
      : [],
  );
  const displayScopeTitle = $derived(currentScopeTitle ?? fallbackScopeTitle());
  const readerSurfaceLabel = $derived(analysisScope === "source_group" ? "Group sources" : "Source material");
  const youtubeRuntimeDiagnostic = $derived(
    currentSource?.sourceType === "youtube" ? sourceSyncDisabledReason(currentSource) : null,
  );

  function fallbackScopeTitle() {
    if (currentRun) return currentRun.scope_label;
    if (currentSource) return currentSource.title ?? currentSource.externalId;
    if (currentGroup) return currentGroup.name;
    return sourceBasisLabel(sourceBasis);
  }

  function sourceTitleForSnapshotMessage(sourceId: number) {
    if (currentSource?.id === sourceId) return currentSource.title ?? currentSource.externalId;
    const member = currentGroup?.members.find((candidate) => candidate.source_id === sourceId);
    return member?.source_title ?? `Source ${sourceId}`;
  }

  function groupMemberSource(sourceId: number) {
    return currentGroup?.members.find((member) => member.source_id === sourceId) ?? null;
  }

  function sourceLabelForGroupItem(item: SourceItem) {
    const member = groupMemberSource(item.sourceId);
    return member?.source_title ?? `Source ${item.sourceId}`;
  }

  function snapshotBrowserSourceType(value: string | null): Source["sourceType"] | null {
    if (value === "telegram" || value === "youtube" || value === "rss" || value === "forum") return value;
    return null;
  }

  function snapshotBrowserSourceSubtype(value: string | null): Source["sourceSubtype"] | null {
    if (
      value === "channel"
      || value === "supergroup"
      || value === "group"
      || value === "video"
      || value === "playlist"
      || value === "feed"
      || value === "thread"
      || value === "board"
      || value === "site"
    ) {
      return value;
    }
    return null;
  }

</script>

<section class="report-source-surface" data-surface={canvasSurface} data-smoke-id="analysis-source-surface">
  {#if currentRun && sourceViewBasis === "run_snapshot"}
    {#if snapshotAvailability === "available"}
      <SourceReaderHeader
        smokeId="run-snapshot-header"
        title="Run snapshot"
        surfaceLabel={currentRun.scope_type === "source_group" ? "Group sources" : "Source material"}
        subtitle="Frozen source material captured for the opened run."
        {sourceViewBasis}
        sourceBasisState={canvasSurface}
        canViewLiveSource={!!currentRun}
        canBackToRunSnapshot={false}
        selectedSourceId={selectedSnapshotSourceId}
        sourceOptions={snapshotSourceOptions}
        {onViewLiveSource}
        {onBackToRunSnapshot}
        onChangeSelectedSourceId={onChangeSelectedSnapshotSourceId}
      />

      {#if runSnapshotSubject && sourceBrowserShellAppliesToSubject(runSnapshotSubject)}
        <SourceBrowserShell
          subject={runSnapshotSubject}
          snapshotBrowserData={{
            run: currentRun,
            readerItems: snapshotReaderItems,
            selectedSourceId: selectedSnapshotSourceId,
            sourceOptions: snapshotSourceOptions,
            loading: loadingRunSnapshotMessages,
            hasMore: hasMoreRunSnapshotMessages,
            availability: snapshotAvailability,
            error: runSnapshotError,
            selectedTraceRef,
            onLoadMore: onLoadMoreRunSnapshotMessages,
          }}
          {selectedTraceRef}
          {formatTimestamp}
        />
      {:else}
        <StatusMessage tone="muted">This run snapshot is not browsable yet.</StatusMessage>
      {/if}
    {:else}
      <SourceReaderHeader
        smokeId="source-browser-header"
        title={sourceBasisLabel(sourceBasis)}
        surfaceLabel={readerSurfaceLabel}
        subtitle={sourceBasisDescription(sourceBasis)}
        {sourceViewBasis}
        sourceBasisState={canvasSurface}
        canViewLiveSource={true}
        canBackToRunSnapshot={false}
        selectedSourceId={null}
        sourceOptions={[]}
        {onViewLiveSource}
        {onBackToRunSnapshot}
        onChangeSelectedSourceId={() => {}}
      />

      {#if snapshotAvailability === "capturing"}
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
    {/if}
  {:else}
    <SourceReaderHeader
      smokeId="source-browser-header"
      title={currentRun && sourceViewBasis === "live_source" ? "Live source" : displayScopeTitle}
      surfaceLabel={readerSurfaceLabel}
      subtitle={sourceBasisDescription(sourceBasis)}
      {sourceViewBasis}
      sourceBasisState={canvasSurface}
      canViewLiveSource={false}
      canBackToRunSnapshot={!!currentRun && canReturnToRunSnapshot(snapshotAvailability)}
      selectedSourceId={analysisScope === "source_group" ? selectedGroupSourceId : null}
      sourceOptions={analysisScope === "source_group" ? liveGroupSourceOptions : []}
      {onViewLiveSource}
      {onBackToRunSnapshot}
      onChangeSelectedSourceId={onChangeSelectedGroupSourceId}
    />
    {#if canvasSurface === "live_source" && currentSource?.sourceType === "telegram" && takeoutRecovery}
      <TakeoutRecoveryNotice recovery={takeoutRecovery} />
    {/if}
    {@render liveSourceSurface()}
  {/if}
</section>

{#snippet liveSourceSurface()}
  {#if analysisScope === "single_source" && currentSource}
    {#if sourceBrowserShellAppliesToSource(currentSource)}
      {#if youtubeRuntimeDiagnostic}
        <StatusMessage tone="error">{youtubeRuntimeDiagnostic}</StatusMessage>
      {/if}
      <SourceBrowserShell
        subject={{ kind: "source", source: currentSource }}
        sourceBrowserData={{
          liveReaderItems,
          takeoutRecovery,
          sourceItems,
          sourceRouteError: sourceItemsError,
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
          telegramHistoryScope,
          currentSourceContentLabel,
          sourceSyncDisabledReason,
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
        }}
        {selectedTraceRef}
        {formatTimestamp}
      />
    {:else}
      <StatusMessage tone="muted" surface={false}>This source type is not browsable yet.</StatusMessage>
    {/if}
  {:else if analysisScope === "source_group" && currentGroup}
    {#if sourceBrowserShellAppliesToSubject({ kind: "source_group", group: currentGroup })}
      <SourceBrowserShell
        subject={{ kind: "source_group", group: currentGroup }}
        {loadingItems}
        {selectedTraceRef}
        {formatTimestamp}
        groupBrowserData={{
          liveReaderItems: groupLiveReaderItems,
          sourceItems: groupLiveSourceItems,
          selectedSourceId: selectedGroupSourceId,
          hasMoreBySource: groupLiveHasMoreBySource,
          sourceLabelForItem: sourceLabelForGroupItem,
          onLoadSourcePage: onLoadLiveGroupSourcePage,
          youtubeDetailsBySource: {},
        }}
      />
    {/if}
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

</style>
