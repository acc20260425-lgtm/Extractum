<script lang="ts">
  import NotebookLmExportDialog, {
    type NotebookLmExportForm,
  } from "$lib/components/analysis/notebooklm-export-dialog.svelte";
  import ReportRunHeader from "$lib/components/analysis/report-run-header.svelte";
  import ReportSetupPanel from "$lib/components/analysis/report-setup-panel.svelte";
  import ReportSourceSurface from "$lib/components/analysis/report-source-surface.svelte";
  import ReportViewer from "$lib/components/analysis/report-viewer.svelte";
  import ReportWorkspaceTools from "$lib/components/analysis/report-workspace-tools.svelte";
  import SourceGroupEditor from "$lib/components/analysis/source-group-editor.svelte";
  import TemplateEditor from "$lib/components/analysis/template-editor.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import { sourceCapabilities } from "$lib/source-capabilities";
  import {
    type CanvasMode,
    type SourceViewBasis,
    type WorkspaceSelection,
  } from "$lib/analysis-workspace-state";
  import type { ChatAvailability } from "$lib/analysis-run-companion-state";
  import type {
    EvidenceHighlightToken,
    SourceReturnContext,
  } from "$lib/analysis-evidence-source-navigation";
  import type { RunSnapshotAvailability } from "$lib/analysis-report-canvas-state";
  import type { SnapshotProbeState } from "$lib/analysis-run-snapshot-affordance";
  import type { BadgeVariant } from "$lib/components/ui/types";
  import type {
    AnalysisGroupSourceType,
    AnalysisPromptTemplate,
    AnalysisRunDetail,
    AnalysisRunMessage,
    AnalysisRunSummary,
    AnalysisSourceGroup,
    AnalysisSourceOption,
    ReportSegment,
    YoutubeCorpusMode,
  } from "$lib/types/analysis";
  import type { LlmProfile, LlmProviderModel } from "$lib/types/llm";
  import type {
    NotebookLmExportEvent,
    NotebookLmExportResult,
    Source,
    SourceForumTopic,
    SourceItem,
    SourceJobRecord,
    TakeoutImportRecoveryState,
    TelegramHistoryScope,
    YoutubeTranscriptSegment,
  } from "$lib/types/sources";
  import type {
    YoutubePlaylistDetail as YoutubePlaylistDetailDto,
    YoutubeVideoDetail,
  } from "$lib/types/youtube";

  export type NotebookLmExportProgressState = {
    phase: NotebookLmExportEvent["phase"];
    message: string;
    current: number | null;
    total: number | null;
  };

  let {
    workspaceSelection,
    currentSource,
    takeoutRecovery = null,
    currentGroup,
    currentSourceMetric,
    currentScopeTitle,
    currentScopeSummary,
    canvasMode,
    sourceViewBasis,
    runSnapshotAvailability,
    snapshotProbeState,
    runSnapshotMessages,
    loadingRunSnapshotMessages,
    runSnapshotError,
    hasMoreRunSnapshotMessages,
    youtubeTranscriptSegments,
    loadingYoutubeTranscriptSegments,
    youtubeTranscriptHasMore,
    youtubeTranscriptSearch,
    groupLiveItemsBySource,
    groupLiveTranscriptSegmentsBySource,
    groupLiveHasMoreBySource,
    selectedGroupSourceId,
    selectedSnapshotSourceId,
    periodFrom,
    periodTo,
    selectedTemplateId,
    loadingTemplates,
    templates,
    outputLanguage,
    youtubeCorpusMode,
    includeMigratedHistory,
    canIncludeMigratedHistory,
    llmProfiles,
    activeLlmProfile,
    selectedLlmProfileId,
    selectedLlmModel,
    customModelOverride,
    llmProviderModels,
    loadingLlmProviderModels,
    llmModelStatus,
    startingReport,
    selectedGroupEditorId,
    currentScopeHasSavedRuns,
    currentRun,
    chatAvailability,
    loadingRunDetail,
    selectedRunIsActive,
    activeProgress,
    activePhase,
    focusedStreamedOutput,
    canCancelCurrentRun,
    sourceItems,
    sourceItemsError,
    sourceItemsHasMore,
    loadingItems,
    sourceTopics,
    loadingSourceTopics,
    selectedTopicKey,
    showTopicSelector,
    telegramHistoryScope,
    selectedTraceRef,
    highlightToken = null,
    sourceReturnContext = null,
    traceRefCount,
    selectedTemplate,
    templateName,
    templateBody,
    savingTemplate,
    deletingTemplate,
    groups,
    groupName,
    groupSourceType,
    groupMemberSourceIds,
    selectedGroup,
    savingGroup,
    deletingGroup,
    sourceMetricsList,
    syncingIds,
    sourceJobs,
    youtubeVideoDetail,
    youtubePlaylistDetail,
    loadingYoutubeDetail,
    formatTimestamp,
    formatPeriod,
    runTargetLabel,
    statusTone,
    reportLines,
    phaseLabel,
    accountLabel,
    sourceSyncDisabledReason,
    reportLaunchDisabledReason,
    startOfDayUnix,
    endOfDayUnix,
    isGroupSourceSelected,
    onChangeCanvasMode,
    onViewLiveSource,
    onBackToRunSnapshot,
    onReturnToEvidenceReview,
    onLoadMoreRunSnapshotMessages,
    onLoadMoreSourceItems,
    onChangeTelegramHistoryScope,
    onChangeTranscriptSearch,
    onLoadMoreYoutubeTranscriptSegments,
    onLoadLiveGroupSourcePage,
    onChangeSelectedGroupSourceId,
    onChangeSelectedSnapshotSourceId,
    onChangeSelectedTopicKey,
    onChangePeriodFrom,
    onChangePeriodTo,
    onChangeSelectedTemplateId,
    onChangeOutputLanguage,
    onChangeYoutubeCorpusMode,
    onChangeIncludeMigratedHistory,
    onChangeLlmProfile,
    onChangeLlmModel,
    onChangeCustomModelOverride,
    onRunReport,
    onSyncCurrentSource,
    onStartTakeoutImport,
    onStartMigratedHistoryImport,
    onSyncYoutubeMetadata,
    onSyncYoutubeTranscript,
    onSyncYoutubeComments,
    onSyncYoutubePlaylist,
    onRetryFailedYoutubePlaylistVideos,
    onSyncYoutubePlaylistVideo,
    onRetryYoutubePlaylistVideo,
    onCancelSourceJob,
    onOpenSource,
    exportDialogOpen,
    notebookLmExportForm,
    notebookLmExportResult,
    notebookLmExportProgress,
    exportingNotebookLm,
    onOpenNotebookLmExport,
    onCloseNotebookLmExport,
    onChooseNotebookLmOutputDir,
    onChangeNotebookLmExportForm,
    onExportNotebookLm,
    onFocusTraceRef,
    onCancelCurrentRun,
    onSaveTemplateCopy,
    onSaveTemplateChanges,
    onDeleteTemplate,
    onChangeSelectedGroupId,
    onChangeGroupName,
    onChangeGroupSourceType,
    onToggleGroupSource,
    onStartNewGroup,
    onSaveGroupCopy,
    onSaveGroupChanges,
    onDeleteGroup,
  }: {
    workspaceSelection: WorkspaceSelection;
    currentSource: Source | null;
    takeoutRecovery?: TakeoutImportRecoveryState | null;
    currentGroup: AnalysisSourceGroup | null;
    currentSourceMetric: AnalysisSourceOption | null;
    currentScopeTitle: string;
    currentScopeSummary: string;
    canvasMode: CanvasMode;
    sourceViewBasis: SourceViewBasis;
    runSnapshotAvailability: RunSnapshotAvailability;
    snapshotProbeState: SnapshotProbeState;
    runSnapshotMessages: AnalysisRunMessage[];
    loadingRunSnapshotMessages: boolean;
    runSnapshotError: string;
    hasMoreRunSnapshotMessages: boolean;
    youtubeTranscriptSegments: YoutubeTranscriptSegment[];
    loadingYoutubeTranscriptSegments: boolean;
    youtubeTranscriptHasMore: boolean;
    youtubeTranscriptSearch: string;
    groupLiveItemsBySource: Record<number, SourceItem[]>;
    groupLiveTranscriptSegmentsBySource: Record<number, YoutubeTranscriptSegment[]>;
    groupLiveHasMoreBySource: Record<number, boolean>;
    selectedGroupSourceId: number | null;
    selectedSnapshotSourceId: number | null;
    periodFrom: string;
    periodTo: string;
    selectedTemplateId: string;
    loadingTemplates: boolean;
    templates: AnalysisPromptTemplate[];
    outputLanguage: string;
    youtubeCorpusMode: YoutubeCorpusMode;
    includeMigratedHistory: boolean;
    canIncludeMigratedHistory: boolean;
    llmProfiles: LlmProfile[];
    activeLlmProfile: string;
    selectedLlmProfileId: string;
    selectedLlmModel: string;
    customModelOverride: string;
    llmProviderModels: LlmProviderModel[];
    loadingLlmProviderModels: boolean;
    llmModelStatus: string;
    startingReport: boolean;
    selectedGroupEditorId: string;
    currentScopeHasSavedRuns: boolean;
    currentRun: AnalysisRunDetail | null;
    chatAvailability: ChatAvailability;
    loadingRunDetail: boolean;
    selectedRunIsActive: boolean;
    activeProgress: string;
    activePhase: string;
    focusedStreamedOutput: string;
    canCancelCurrentRun: boolean;
    sourceItems: SourceItem[];
    sourceItemsError: string | null;
    sourceItemsHasMore: boolean;
    loadingItems: boolean;
    sourceTopics: SourceForumTopic[];
    loadingSourceTopics: boolean;
    selectedTopicKey: string;
    showTopicSelector: boolean;
    telegramHistoryScope: TelegramHistoryScope;
    selectedTraceRef: string | null;
    highlightToken?: EvidenceHighlightToken | null;
    sourceReturnContext?: SourceReturnContext;
    traceRefCount: number;
    selectedTemplate: AnalysisPromptTemplate | null;
    templateName: string;
    templateBody: string;
    savingTemplate: boolean;
    deletingTemplate: boolean;
    groups: AnalysisSourceGroup[];
    groupName: string;
    groupSourceType: AnalysisGroupSourceType;
    groupMemberSourceIds: number[];
    selectedGroup: AnalysisSourceGroup | null;
    savingGroup: boolean;
    deletingGroup: boolean;
    sourceMetricsList: AnalysisSourceOption[];
    syncingIds: Record<number, boolean>;
    sourceJobs: SourceJobRecord[];
    youtubeVideoDetail: YoutubeVideoDetail | null;
    youtubePlaylistDetail: YoutubePlaylistDetailDto | null;
    loadingYoutubeDetail: boolean;
    formatTimestamp: (value: number | null) => string;
    formatPeriod: (from: number, to: number) => string;
    runTargetLabel: (
      run: Pick<
        AnalysisRunSummary,
        "scope_type" | "source_id" | "source_title" | "source_group_id" | "source_group_name" | "scope_label"
      >,
    ) => string;
    statusTone: (value: string) => BadgeVariant;
    reportLines: (value: string) => Array<{
      key: string;
      segments: ReportSegment[];
    }>;
    phaseLabel: (value: string) => string;
    accountLabel: (accountId: number | null) => string;
    sourceSyncDisabledReason: (source: Source) => string | null;
    reportLaunchDisabledReason: string | null;
    startOfDayUnix: (value: string) => number;
    endOfDayUnix: (value: string) => number;
    isGroupSourceSelected: (sourceId: number) => boolean;
    onChangeCanvasMode: (mode: CanvasMode) => void;
    onViewLiveSource: () => void;
    onBackToRunSnapshot: () => void;
    onReturnToEvidenceReview: () => void;
    onLoadMoreRunSnapshotMessages: () => void | Promise<void>;
    onLoadMoreSourceItems: () => void | Promise<void>;
    onChangeTelegramHistoryScope: (scope: TelegramHistoryScope) => void;
    onChangeTranscriptSearch: (value: string) => void;
    onLoadMoreYoutubeTranscriptSegments: () => void | Promise<void>;
    onLoadLiveGroupSourcePage: (sourceId: number) => void | Promise<void>;
    onChangeSelectedGroupSourceId: (sourceId: number | null) => void;
    onChangeSelectedSnapshotSourceId: (sourceId: number | null) => void;
    onChangeSelectedTopicKey: (value: string) => void | Promise<void>;
    onChangePeriodFrom: (value: string) => void;
    onChangePeriodTo: (value: string) => void;
    onChangeSelectedTemplateId: (value: string) => void;
    onChangeOutputLanguage: (value: string) => void;
    onChangeYoutubeCorpusMode: (value: YoutubeCorpusMode) => void;
    onChangeIncludeMigratedHistory: (value: boolean) => void;
    onChangeLlmProfile: (value: string) => void;
    onChangeLlmModel: (value: string) => void;
    onChangeCustomModelOverride: (value: string) => void;
    onRunReport: () => void;
    onSyncCurrentSource: (sourceId: number) => void;
    onStartTakeoutImport: (sourceId: number) => void | Promise<void>;
    onStartMigratedHistoryImport: (sourceId: number) => void | Promise<void>;
    onSyncYoutubeMetadata: (sourceId: number) => void | Promise<void>;
    onSyncYoutubeTranscript: (sourceId: number) => void | Promise<void>;
    onSyncYoutubeComments: (sourceId: number) => void | Promise<void>;
    onSyncYoutubePlaylist: (sourceId: number) => void | Promise<void>;
    onRetryFailedYoutubePlaylistVideos: (sourceId: number) => void | Promise<void>;
    onSyncYoutubePlaylistVideo: (playlistSourceId: number, videoSourceId: number) => void | Promise<void>;
    onRetryYoutubePlaylistVideo: (playlistSourceId: number, videoSourceId: number) => void | Promise<void>;
    onCancelSourceJob: (jobId: string) => void | Promise<void>;
    onOpenSource: (sourceId: number) => void | Promise<void>;
    exportDialogOpen: boolean;
    notebookLmExportForm: NotebookLmExportForm;
    notebookLmExportResult: NotebookLmExportResult | null;
    notebookLmExportProgress: NotebookLmExportProgressState | null;
    exportingNotebookLm: boolean;
    onOpenNotebookLmExport: () => void;
    onCloseNotebookLmExport: () => void;
    onChooseNotebookLmOutputDir: () => void | Promise<void>;
    onChangeNotebookLmExportForm: (form: NotebookLmExportForm) => void;
    onExportNotebookLm: () => void | Promise<void>;
    onFocusTraceRef: (ref: string) => void | Promise<void>;
    onCancelCurrentRun: () => void;
    onSaveTemplateCopy: () => void;
    onSaveTemplateChanges: () => void;
    onDeleteTemplate: () => void;
    onChangeSelectedGroupId: (value: string) => void;
    onChangeGroupName: (value: string) => void;
    onChangeGroupSourceType: (value: AnalysisGroupSourceType) => void;
    onToggleGroupSource: (sourceId: number) => void;
    onStartNewGroup: () => void;
    onSaveGroupCopy: () => void;
    onSaveGroupChanges: () => void;
    onDeleteGroup: () => void;
    [key: string]: unknown;
  } = $props();

  let templateEditorOpen = $state(false);
  let groupEditorOpen = $state(false);

  const currentSourceContentLabel = $derived(
    currentSource ? sourceCapabilities(currentSource).contentLabel : "items",
  );
  const showNotebookLmExport = $derived(currentSource !== null || currentGroup !== null);
  const youtubeSourceGroupNotebookLmExportReason =
    "YouTube source-group NotebookLM export is not implemented yet.";
  const notebookLmExportDisabledReason = $derived(
    currentGroup && !currentSource && currentGroup.source_type !== "telegram"
      ? youtubeSourceGroupNotebookLmExportReason
      : null,
  );
  const canExportNotebookLm = $derived(
    !exportingNotebookLm
      && (currentSource !== null || currentGroup?.source_type === "telegram"),
  );
  const notebookLmExportTargetLabel = $derived(
    currentSource
      ? (currentSource.title ?? currentSource.externalId)
      : currentGroup
        ? currentGroup.name
        : "",
  );
  const notebookLmExportTargetDescription = $derived(
    currentSource
      ? `Prepare Markdown files for ${currentSource.title ?? currentSource.externalId}.`
      : currentGroup
        ? `Prepare Markdown files for ${currentGroup.name} (${currentGroup.members.length} sources).`
        : "",
  );
  const showInlineWorkspaceTools = $derived(true);
</script>

<section class="report-canvas" data-smoke-id="analysis-report-canvas">
  <div class="canvas-context-bar" aria-label="Analysis context">
    <div class="canvas-title">
      <span class="eyebrow">{currentRun ? "Run workspace" : "Analysis workspace"}</span>
      <h2>{currentRun ? runTargetLabel(currentRun) : currentScopeTitle}</h2>
      <p>{currentRun ? "Report and source basis stay side by side." : currentScopeSummary}</p>
    </div>
    <div class="canvas-actions-row">
      <div class="canvas-tabs" role="tablist" aria-label="Report canvas mode">
        <Button
          type="button"
          role="tab"
          variant="secondary"
          selected={canvasMode === "report"}
          ariaSelected={canvasMode === "report"}
          smokeId="report-canvas-mode-report"
          onclick={() => onChangeCanvasMode("report")}
        >
          Report
        </Button>
        <Button
          type="button"
          role="tab"
          variant="secondary"
          selected={canvasMode === "source"}
          ariaSelected={canvasMode === "source"}
          smokeId="report-canvas-mode-source"
          onclick={() => onChangeCanvasMode("source")}
        >
          Source
        </Button>
      </div>

      {#if showInlineWorkspaceTools}
        <ReportWorkspaceTools
          compact
          {showNotebookLmExport}
          {canExportNotebookLm}
          exportDisabledReason={notebookLmExportDisabledReason}
          {exportingNotebookLm}
          {templateEditorOpen}
          {groupEditorOpen}
          onOpenNotebookLmExport={onOpenNotebookLmExport}
          onToggleTemplateEditor={() => (templateEditorOpen = !templateEditorOpen)}
          onToggleGroupEditor={() => (groupEditorOpen = !groupEditorOpen)}
        />
      {/if}
    </div>
  </div>

  {#if templateEditorOpen}
    <div class="workspace-template-editor-drawer" aria-label="Template editor drawer" data-smoke-id="template-editor-drawer">
      <TemplateEditor
        compact={true}
        {selectedTemplate}
        {templateName}
        {templateBody}
        {savingTemplate}
        {deletingTemplate}
        onSaveTemplateCopy={onSaveTemplateCopy}
        onSaveTemplateChanges={onSaveTemplateChanges}
        onDeleteTemplate={onDeleteTemplate}
      />
    </div>
  {/if}

  {#if groupEditorOpen}
    <div class="workspace-group-editor-drawer" aria-label="Source group editor drawer" data-smoke-id="source-group-editor-drawer">
      <SourceGroupEditor
        compact={true}
        {groups}
        selectedGroupId={selectedGroupEditorId}
        {selectedGroup}
        {groupName}
        {groupSourceType}
        {groupMemberSourceIds}
        sources={sourceMetricsList}
        {savingGroup}
        {deletingGroup}
        {formatTimestamp}
        {isGroupSourceSelected}
        onChangeSelectedGroupId={onChangeSelectedGroupId}
        onChangeGroupName={onChangeGroupName}
        onChangeGroupSourceType={onChangeGroupSourceType}
        onToggleSource={onToggleGroupSource}
        onStartNewGroup={onStartNewGroup}
        onSaveGroupCopy={onSaveGroupCopy}
        onSaveGroupChanges={onSaveGroupChanges}
        onDeleteGroup={onDeleteGroup}
      />
    </div>
  {/if}

  {#if canvasMode === "report"}
    {#if currentRun}
      <ReportRunHeader
        {currentRun}
        {sourceViewBasis}
        snapshotAvailability={runSnapshotAvailability}
        {snapshotProbeState}
        {traceRefCount}
        {activePhase}
        {activeProgress}
        {canCancelCurrentRun}
        {formatTimestamp}
        {formatPeriod}
        {runTargetLabel}
        {statusTone}
        {onCancelCurrentRun}
      />
      <ReportViewer
        {currentRun}
        {chatAvailability}
        {loadingRunDetail}
        streamedOutput={focusedStreamedOutput}
        {traceRefCount}
        {selectedTraceRef}
        livePhase={activePhase}
        liveProgress={activeProgress}
        {canCancelCurrentRun}
        {formatTimestamp}
        {formatPeriod}
        {runTargetLabel}
        {statusTone}
        {reportLines}
        {onFocusTraceRef}
        {onCancelCurrentRun}
      />
    {:else}
      <ReportSetupPanel
        {workspaceSelection}
        {currentSource}
        {currentGroup}
        {currentSourceMetric}
        {currentScopeTitle}
        {currentScopeSummary}
        {periodFrom}
        {periodTo}
        {selectedTemplateId}
        {loadingTemplates}
        {templates}
        {outputLanguage}
        {youtubeCorpusMode}
        {includeMigratedHistory}
        {canIncludeMigratedHistory}
        {llmProfiles}
        {activeLlmProfile}
        {selectedLlmProfileId}
        {selectedLlmModel}
        {customModelOverride}
        {llmProviderModels}
        {loadingLlmProviderModels}
        {llmModelStatus}
        {startingReport}
        {currentScopeHasSavedRuns}
        {selectedRunIsActive}
        {activeProgress}
        {activePhase}
        {selectedTemplate}
        {syncingIds}
        {formatTimestamp}
        {formatPeriod}
        {phaseLabel}
        {accountLabel}
        {sourceSyncDisabledReason}
        {reportLaunchDisabledReason}
        {startOfDayUnix}
        {endOfDayUnix}
        onChangePeriodFrom={onChangePeriodFrom}
        onChangePeriodTo={onChangePeriodTo}
        onChangeSelectedTemplateId={onChangeSelectedTemplateId}
        onChangeOutputLanguage={onChangeOutputLanguage}
        onChangeYoutubeCorpusMode={onChangeYoutubeCorpusMode}
        onChangeIncludeMigratedHistory={onChangeIncludeMigratedHistory}
        onChangeLlmProfile={onChangeLlmProfile}
        onChangeLlmModel={onChangeLlmModel}
        onChangeCustomModelOverride={onChangeCustomModelOverride}
        onRunReport={onRunReport}
        onSyncCurrentSource={onSyncCurrentSource}
      />
    {/if}
  {:else}
    <ReportSourceSurface
      {workspaceSelection}
      {currentRun}
      {sourceViewBasis}
      snapshotAvailability={runSnapshotAvailability}
      {snapshotProbeState}
      {runSnapshotMessages}
      {loadingRunSnapshotMessages}
      {runSnapshotError}
      {hasMoreRunSnapshotMessages}
      {selectedTraceRef}
      {highlightToken}
      {sourceReturnContext}
      {currentScopeTitle}
      {currentSource}
      {takeoutRecovery}
      {currentGroup}
      {currentSourceMetric}
      {sourceItems}
      {sourceItemsError}
      {sourceItemsHasMore}
      {loadingItems}
      {youtubeTranscriptSegments}
      {loadingYoutubeTranscriptSegments}
      {youtubeTranscriptHasMore}
      {youtubeTranscriptSearch}
      {groupLiveItemsBySource}
      {groupLiveTranscriptSegmentsBySource}
      {groupLiveHasMoreBySource}
      {selectedGroupSourceId}
      {selectedSnapshotSourceId}
      {sourceTopics}
      {loadingSourceTopics}
      {selectedTopicKey}
      {showTopicSelector}
      {telegramHistoryScope}
      {currentSourceContentLabel}
      {sourceJobs}
      {youtubeVideoDetail}
      {youtubePlaylistDetail}
      {loadingYoutubeDetail}
      {formatTimestamp}
      {sourceSyncDisabledReason}
      onViewLiveSource={onViewLiveSource}
      onBackToRunSnapshot={onBackToRunSnapshot}
      onReturnToEvidenceReview={onReturnToEvidenceReview}
      onLoadMoreRunSnapshotMessages={onLoadMoreRunSnapshotMessages}
      onLoadMoreSourceItems={onLoadMoreSourceItems}
      onChangeTelegramHistoryScope={onChangeTelegramHistoryScope}
      onChangeTranscriptSearch={onChangeTranscriptSearch}
      onLoadMoreYoutubeTranscriptSegments={onLoadMoreYoutubeTranscriptSegments}
      onLoadLiveGroupSourcePage={onLoadLiveGroupSourcePage}
      onChangeSelectedGroupSourceId={onChangeSelectedGroupSourceId}
      onChangeSelectedSnapshotSourceId={onChangeSelectedSnapshotSourceId}
      onChangeSelectedTopicKey={onChangeSelectedTopicKey}
      onSyncSource={onSyncCurrentSource}
      onSyncYoutubeMetadata={onSyncYoutubeMetadata}
      onSyncYoutubeTranscript={onSyncYoutubeTranscript}
      onSyncYoutubeComments={onSyncYoutubeComments}
      onStartTakeoutImport={onStartTakeoutImport}
      onStartMigratedHistoryImport={onStartMigratedHistoryImport}
      onSyncYoutubePlaylist={onSyncYoutubePlaylist}
      onRetryFailedYoutubePlaylistVideos={onRetryFailedYoutubePlaylistVideos}
      onSyncYoutubePlaylistVideo={onSyncYoutubePlaylistVideo}
      onRetryYoutubePlaylistVideo={onRetryYoutubePlaylistVideo}
      onCancelSourceJob={onCancelSourceJob}
      onOpenSource={onOpenSource}
    />
  {/if}

  <NotebookLmExportDialog
    open={exportDialogOpen}
    targetLabel={notebookLmExportTargetLabel}
    targetDescription={notebookLmExportTargetDescription}
    form={notebookLmExportForm}
    canIncludeMigratedHistory={canIncludeMigratedHistory}
    exporting={exportingNotebookLm}
    result={notebookLmExportResult}
    progress={notebookLmExportProgress}
    onClose={onCloseNotebookLmExport}
    onChooseFolder={onChooseNotebookLmOutputDir}
    onExport={onExportNotebookLm}
    onChangeForm={onChangeNotebookLmExportForm}
  />
</section>

<style>
  .report-canvas {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
  }

  .canvas-context-bar {
    display: flex;
    justify-content: space-between;
    gap: 0.9rem;
    align-items: flex-start;
    padding: 1rem;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--panel);
    box-shadow: var(--shadow);
  }

  .canvas-actions-row {
    display: flex;
    gap: 0.55rem;
    align-items: flex-start;
    justify-content: flex-end;
    flex-wrap: wrap;
  }

  .canvas-title {
    min-width: 0;
  }

  .canvas-title h2,
  .canvas-title p {
    margin: 0;
  }

  .canvas-title p {
    margin-top: 0.3rem;
    color: var(--muted);
    line-height: 1.45;
  }

  .canvas-tabs {
    display: flex;
    flex-wrap: wrap;
    gap: 0.35rem;
    padding: 0.2rem;
    border-radius: 8px;
    background: color-mix(in srgb, var(--panel-strong) 70%, transparent);
  }

  .workspace-template-editor-drawer,
  .workspace-group-editor-drawer {
    padding: 0.85rem;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--panel);
    box-shadow: var(--shadow-soft);
  }

  .eyebrow {
    display: inline-block;
    font-size: 0.68rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--muted);
    margin-bottom: 0.2rem;
  }

  @media (max-width: 720px) {
    .canvas-context-bar {
      flex-direction: column;
      align-items: stretch;
    }

    .canvas-actions-row {
      width: 100%;
      justify-content: space-between;
    }
  }
</style>
