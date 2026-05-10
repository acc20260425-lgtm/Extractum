<script lang="ts">
  import ChatPanel from "$lib/components/analysis/chat-panel.svelte";
  import NotebookLmExportDialog, {
    type NotebookLmExportForm,
  } from "$lib/components/analysis/notebooklm-export-dialog.svelte";
  import ReportRunHeader from "$lib/components/analysis/report-run-header.svelte";
  import ReportSetupPanel from "$lib/components/analysis/report-setup-panel.svelte";
  import ReportSourceSurface from "$lib/components/analysis/report-source-surface.svelte";
  import ReportViewer from "$lib/components/analysis/report-viewer.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import { sourceCapabilities } from "$lib/source-capabilities";
  import type {
    CanvasMode,
    SourceViewBasis,
  } from "$lib/analysis-workspace-state";
  import type { RunSnapshotAvailability } from "$lib/analysis-report-canvas-state";
  import type { BadgeVariant } from "$lib/components/ui/types";
  import type {
    AnalysisChatTurn,
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
    analysisScope,
    currentSource,
    currentGroup,
    currentSourceMetric,
    currentScopeTitle,
    currentScopeSummary,
    canvasMode,
    sourceViewBasis,
    runSnapshotAvailability,
    runSnapshotMessages,
    loadingRunSnapshotMessages,
    runSnapshotError,
    hasMoreRunSnapshotMessages,
    periodFrom,
    periodTo,
    selectedTemplateId,
    loadingTemplates,
    templates,
    outputLanguage,
    youtubeCorpusMode,
    llmProfiles,
    activeLlmProfile,
    selectedLlmProfileId,
    selectedLlmModel,
    customModelOverride,
    llmProviderModels,
    loadingLlmProviderModels,
    llmModelStatus,
    startingReport,
    selectedSourceId,
    selectedGroupId,
    currentRun,
    loadingRunDetail,
    selectedRunIsActive,
    activeProgress,
    activePhase,
    focusedStreamedOutput,
    canCancelCurrentRun,
    sourceItems,
    loadingItems,
    sourceTopics,
    loadingSourceTopics,
    selectedTopicKey,
    showTopicSelector,
    selectedTraceRef,
    traceRefCount,
    chatMessages,
    chatQuestion,
    chatting,
    canCancelChat,
    clearingChat,
    loadingChat,
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
    startOfDayUnix,
    endOfDayUnix,
    isGroupSourceSelected,
    onChangeCanvasMode,
    onViewLiveSource,
    onBackToRunSnapshot,
    onLoadMoreRunSnapshotMessages,
    onChangeSelectedTopicKey,
    onChangePeriodFrom,
    onChangePeriodTo,
    onChangeSelectedTemplateId,
    onChangeOutputLanguage,
    onChangeYoutubeCorpusMode,
    onChangeLlmProfile,
    onChangeLlmModel,
    onChangeCustomModelOverride,
    onRunReport,
    onSyncCurrentSource,
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
    onAskRunQuestion,
    onCancelChat,
    onClearChat,
    onChangeChatQuestion,
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
    analysisScope: "single_source" | "source_group";
    currentSource: Source | null;
    currentGroup: AnalysisSourceGroup | null;
    currentSourceMetric: AnalysisSourceOption | null;
    currentScopeTitle: string;
    currentScopeSummary: string;
    canvasMode: CanvasMode;
    sourceViewBasis: SourceViewBasis;
    runSnapshotAvailability: RunSnapshotAvailability;
    runSnapshotMessages: AnalysisRunMessage[];
    loadingRunSnapshotMessages: boolean;
    runSnapshotError: string;
    hasMoreRunSnapshotMessages: boolean;
    periodFrom: string;
    periodTo: string;
    selectedTemplateId: string;
    loadingTemplates: boolean;
    templates: AnalysisPromptTemplate[];
    outputLanguage: string;
    youtubeCorpusMode: YoutubeCorpusMode;
    llmProfiles: LlmProfile[];
    activeLlmProfile: string;
    selectedLlmProfileId: string;
    selectedLlmModel: string;
    customModelOverride: string;
    llmProviderModels: LlmProviderModel[];
    loadingLlmProviderModels: boolean;
    llmModelStatus: string;
    startingReport: boolean;
    selectedSourceId: string;
    selectedGroupId: string;
    currentRun: AnalysisRunDetail | null;
    loadingRunDetail: boolean;
    selectedRunIsActive: boolean;
    activeProgress: string;
    activePhase: string;
    focusedStreamedOutput: string;
    canCancelCurrentRun: boolean;
    sourceItems: SourceItem[];
    loadingItems: boolean;
    sourceTopics: SourceForumTopic[];
    loadingSourceTopics: boolean;
    selectedTopicKey: string;
    showTopicSelector: boolean;
    selectedTraceRef: string | null;
    traceRefCount: number;
    chatMessages: AnalysisChatTurn[];
    chatQuestion: string;
    chatting: boolean;
    canCancelChat: boolean;
    clearingChat: boolean;
    loadingChat: boolean;
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
    startOfDayUnix: (value: string) => number;
    endOfDayUnix: (value: string) => number;
    isGroupSourceSelected: (sourceId: number) => boolean;
    onChangeCanvasMode: (mode: CanvasMode) => void;
    onViewLiveSource: () => void;
    onBackToRunSnapshot: () => void;
    onLoadMoreRunSnapshotMessages: () => void | Promise<void>;
    onChangeSelectedTopicKey: (value: string) => void | Promise<void>;
    onChangePeriodFrom: (value: string) => void;
    onChangePeriodTo: (value: string) => void;
    onChangeSelectedTemplateId: (value: string) => void;
    onChangeOutputLanguage: (value: string) => void;
    onChangeYoutubeCorpusMode: (value: YoutubeCorpusMode) => void;
    onChangeLlmProfile: (value: string) => void;
    onChangeLlmModel: (value: string) => void;
    onChangeCustomModelOverride: (value: string) => void;
    onRunReport: () => void;
    onSyncCurrentSource: (sourceId: number) => void;
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
    onAskRunQuestion: () => void;
    onCancelChat: () => void;
    onClearChat: () => void;
    onChangeChatQuestion: (value: string) => void;
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
  } = $props();

  const currentSourceContentLabel = $derived(
    currentSource ? sourceCapabilities(currentSource).contentLabel : "items",
  );
</script>

<section class="report-canvas">
  <div class="canvas-toolbar">
    <div class="canvas-title">
      <span class="eyebrow">{currentRun ? "Run workspace" : "Analysis setup"}</span>
      <h2>{currentRun ? runTargetLabel(currentRun) : currentScopeTitle}</h2>
      <p>{currentRun ? "Read the report or inspect the source basis for this run." : currentScopeSummary}</p>
    </div>
    <div class="canvas-tabs" role="tablist" aria-label="Report canvas mode">
      <Button
        type="button"
        role="tab"
        variant="secondary"
        selected={canvasMode === "report"}
        ariaSelected={canvasMode === "report"}
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
        onclick={() => onChangeCanvasMode("source")}
      >
        Source
      </Button>
    </div>
  </div>

  {#if canvasMode === "report"}
    {#if currentRun}
      <ReportRunHeader
        {currentRun}
        {sourceViewBasis}
        snapshotAvailability={runSnapshotAvailability}
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
      <div class="temporary-follow-up" aria-label="Temporary follow-up chat until companion tabs ship">
        <ChatPanel
          {currentRun}
          {loadingChat}
          {chatMessages}
          {chatQuestion}
          {chatting}
          {canCancelChat}
          {clearingChat}
          {selectedTraceRef}
          {reportLines}
          onFocusTraceRef={onFocusTraceRef}
          onAskQuestion={onAskRunQuestion}
          onCancelChat={onCancelChat}
          onClearChat={onClearChat}
          onChangeChatQuestion={onChangeChatQuestion}
        />
      </div>
    {:else}
      <ReportSetupPanel
        {analysisScope}
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
        {llmProfiles}
        {activeLlmProfile}
        {selectedLlmProfileId}
        {selectedLlmModel}
        {customModelOverride}
        {llmProviderModels}
        {loadingLlmProviderModels}
        {llmModelStatus}
        {startingReport}
        {selectedSourceId}
        {selectedGroupId}
        {selectedRunIsActive}
        {activeProgress}
        {activePhase}
        {selectedTemplate}
        {templateName}
        {templateBody}
        {savingTemplate}
        {deletingTemplate}
        {groups}
        {groupName}
        {groupSourceType}
        {groupMemberSourceIds}
        {selectedGroup}
        {savingGroup}
        {deletingGroup}
        {sourceMetricsList}
        {syncingIds}
        {exportingNotebookLm}
        {formatTimestamp}
        {formatPeriod}
        {phaseLabel}
        {accountLabel}
        {sourceSyncDisabledReason}
        {startOfDayUnix}
        {endOfDayUnix}
        {isGroupSourceSelected}
        onChangePeriodFrom={onChangePeriodFrom}
        onChangePeriodTo={onChangePeriodTo}
        onChangeSelectedTemplateId={onChangeSelectedTemplateId}
        onChangeOutputLanguage={onChangeOutputLanguage}
        onChangeYoutubeCorpusMode={onChangeYoutubeCorpusMode}
        onChangeLlmProfile={onChangeLlmProfile}
        onChangeLlmModel={onChangeLlmModel}
        onChangeCustomModelOverride={onChangeCustomModelOverride}
        onRunReport={onRunReport}
        onSyncCurrentSource={onSyncCurrentSource}
        onOpenNotebookLmExport={onOpenNotebookLmExport}
        onSaveTemplateCopy={onSaveTemplateCopy}
        onSaveTemplateChanges={onSaveTemplateChanges}
        onDeleteTemplate={onDeleteTemplate}
        onChangeSelectedGroupId={onChangeSelectedGroupId}
        onChangeGroupName={onChangeGroupName}
        onChangeGroupSourceType={onChangeGroupSourceType}
        onToggleGroupSource={onToggleGroupSource}
        onStartNewGroup={onStartNewGroup}
        onSaveGroupCopy={onSaveGroupCopy}
        onSaveGroupChanges={onSaveGroupChanges}
        onDeleteGroup={onDeleteGroup}
      />
    {/if}
  {:else}
    <ReportSourceSurface
      {analysisScope}
      {currentRun}
      {sourceViewBasis}
      snapshotAvailability={runSnapshotAvailability}
      {runSnapshotMessages}
      {loadingRunSnapshotMessages}
      {runSnapshotError}
      {hasMoreRunSnapshotMessages}
      {currentSource}
      {currentGroup}
      {currentSourceMetric}
      {sourceItems}
      {loadingItems}
      {sourceTopics}
      {loadingSourceTopics}
      {selectedTopicKey}
      {showTopicSelector}
      {currentSourceContentLabel}
      {sourceJobs}
      {youtubeVideoDetail}
      {youtubePlaylistDetail}
      {loadingYoutubeDetail}
      {formatTimestamp}
      onViewLiveSource={onViewLiveSource}
      onBackToRunSnapshot={onBackToRunSnapshot}
      onLoadMoreRunSnapshotMessages={onLoadMoreRunSnapshotMessages}
      onChangeSelectedTopicKey={onChangeSelectedTopicKey}
      onSyncYoutubeMetadata={onSyncYoutubeMetadata}
      onSyncYoutubeTranscript={onSyncYoutubeTranscript}
      onSyncYoutubeComments={onSyncYoutubeComments}
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
    source={currentSource}
    form={notebookLmExportForm}
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

  .canvas-toolbar {
    display: flex;
    justify-content: space-between;
    gap: 0.8rem;
    align-items: flex-start;
    padding: 1rem;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--panel);
    box-shadow: var(--shadow);
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

  .temporary-follow-up {
    border: 1px solid color-mix(in srgb, var(--border) 80%, transparent);
    border-radius: 8px;
    background: var(--panel);
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
    .canvas-toolbar {
      flex-direction: column;
    }
  }
</style>
