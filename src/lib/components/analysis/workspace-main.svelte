<script lang="ts">
  import { Download, Play, RefreshCw } from "@lucide/svelte";
  import ChatPanel from "$lib/components/analysis/chat-panel.svelte";
  import NotebookLmExportDialog, {
    type NotebookLmExportForm,
  } from "$lib/components/analysis/notebooklm-export-dialog.svelte";
  import ReportViewer from "$lib/components/analysis/report-viewer.svelte";
  import SourceContextPanel from "$lib/components/analysis/source-context-panel.svelte";
  import SourceGroupEditor from "$lib/components/analysis/source-group-editor.svelte";
  import TemplateEditor from "$lib/components/analysis/template-editor.svelte";
  import YoutubePlaylistDetail from "$lib/components/analysis/youtube-playlist-detail.svelte";
  import YoutubeSourceDetail from "$lib/components/analysis/youtube-source-detail.svelte";
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import Select from "$lib/components/ui/Select.svelte";
  import { sourceCapabilities, sourceKindLabel } from "$lib/source-capabilities";
  import type { BadgeVariant } from "$lib/components/ui/types";
  import type {
    AnalysisGroupSourceType,
    AnalysisChatTurn,
    AnalysisPromptTemplate,
    AnalysisRunDetail,
    AnalysisRunSummary,
    AnalysisSourceGroup,
    AnalysisSourceOption,
    ReportSegment,
    YoutubeCorpusMode,
  } from "$lib/types/analysis";
  import type {
    SourceForumTopic,
    SourceItem,
    NotebookLmExportEvent,
    NotebookLmExportResult,
    Source,
    SourceJobRecord,
  } from "$lib/types/sources";
  import type { YoutubePlaylistDetail as YoutubePlaylistDetailDto, YoutubeVideoDetail } from "$lib/types/youtube";

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
    periodFrom,
    periodTo,
    selectedTemplateId,
    loadingTemplates,
    templates,
    outputLanguage,
    youtubeCorpusMode,
    modelOverride,
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
    onChangeSelectedTopicKey,
    onChangePeriodFrom,
    onChangePeriodTo,
    onChangeSelectedTemplateId,
    onChangeOutputLanguage,
    onChangeYoutubeCorpusMode,
    onChangeModelOverride,
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
    periodFrom: string;
    periodTo: string;
    selectedTemplateId: string;
    loadingTemplates: boolean;
    templates: AnalysisPromptTemplate[];
    outputLanguage: string;
    youtubeCorpusMode: YoutubeCorpusMode;
    modelOverride: string;
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
      >
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
    onChangeSelectedTopicKey: (value: string) => void | Promise<void>;
    onChangePeriodFrom: (value: string) => void;
    onChangePeriodTo: (value: string) => void;
    onChangeSelectedTemplateId: (value: string) => void;
    onChangeOutputLanguage: (value: string) => void;
    onChangeYoutubeCorpusMode: (value: YoutubeCorpusMode) => void;
    onChangeModelOverride: (value: string) => void;
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

  const sourceContextKey = $derived(
    `${analysisScope}:${currentSource?.id ?? "none"}:${currentGroup?.id ?? "none"}:${currentRun?.id ?? "idle"}`,
  );
  const isYoutubeScope = $derived(
    (analysisScope === "single_source" && currentSource?.sourceType === "youtube") ||
      (analysisScope === "source_group" && currentGroup?.source_type === "youtube"),
  );
  const currentSourceContentLabel = $derived(currentSource ? sourceCapabilities(currentSource).contentLabel : "items");
</script>

<section class="center-pane">
  <div class="scope-hero">
    <div class="scope-hero-copy">
      <span class="eyebrow">{analysisScope === "source_group" ? "Source group workspace" : "Source workspace"}</span>
      <h2>{currentScopeTitle}</h2>
      <p>{currentScopeSummary}</p>
    </div>
    <div class="scope-hero-meta">
      {#if analysisScope === "single_source" && currentSource}
        <Badge variant="info">{sourceKindLabel(currentSource)}</Badge>
        <Badge>{accountLabel(currentSource.accountId)}</Badge>
      {/if}
      {#if analysisScope === "source_group" && currentGroup}
        <Badge variant="info">{currentGroup.members.length} sources</Badge>
      {/if}
      {#if currentRun}
        <Badge variant={statusTone(currentRun.status)}>Run #{currentRun.id}</Badge>
      {/if}
    </div>
  </div>

  <div class="scope-facts">
    <div class="scope-fact">
      <span class="scope-fact-label">Scope</span>
      <strong>{analysisScope === "source_group" ? "Group analysis" : "Single source"}</strong>
    </div>
    <div class="scope-fact">
      <span class="scope-fact-label">Window</span>
      <strong>{formatPeriod(startOfDayUnix(periodFrom), endOfDayUnix(periodTo))}</strong>
    </div>
    <div class="scope-fact">
      <span class="scope-fact-label">Context</span>
      <strong>
        {#if analysisScope === "source_group" && currentGroup}
          {currentGroup.members.length} sources
        {:else if currentSourceMetric}
          {currentSourceMetric.item_count} synced {currentSourceContentLabel}
        {:else}
          Awaiting synced context
        {/if}
      </strong>
    </div>
    <div class="scope-fact">
      <span class="scope-fact-label">Output</span>
      <strong>{outputLanguage || "Default language"}</strong>
    </div>
  </div>

  <div class="controls-panel">
    <div class="controls-grid">
      <label>Period from
        <Input
          type="date"
          value={periodFrom}
          oninput={(event) => onChangePeriodFrom((event.currentTarget as HTMLInputElement).value)}
        />
      </label>
      <label>Period to
        <Input
          type="date"
          value={periodTo}
          oninput={(event) => onChangePeriodTo((event.currentTarget as HTMLInputElement).value)}
        />
      </label>
      <label>Prompt template
        <Select
          value={selectedTemplateId}
          disabled={loadingTemplates}
          onchange={(event) => onChangeSelectedTemplateId((event.currentTarget as HTMLSelectElement).value)}
        >
          {#if loadingTemplates}
            <option value="">Loading templates...</option>
          {:else if templates.length === 0}
            <option value="">No report templates available</option>
          {/if}
          {#each templates as template (template.id)}
            <option value={String(template.id)}>
              {template.name}{template.is_builtin ? " - builtin" : ""}
            </option>
          {/each}
        </Select>
      </label>
      <label>Output language
        <Input
          type="text"
          value={outputLanguage}
          placeholder="Russian"
          ariaLabel="Output language"
          oninput={(event) => onChangeOutputLanguage((event.currentTarget as HTMLInputElement).value)}
        />
      </label>
      {#if isYoutubeScope}
        <label>YouTube corpus
          <Select
            value={youtubeCorpusMode}
            onchange={(event) => onChangeYoutubeCorpusMode((event.currentTarget as HTMLSelectElement).value as YoutubeCorpusMode)}
          >
            <option value="transcript_only">Transcript</option>
            <option value="transcript_description">Transcript + description</option>
            <option value="transcript_description_comments">Transcript + description + comments</option>
          </Select>
        </label>
      {/if}
    </div>

    <div class="controls-bottom">
      <label class="model-field">Model override
        <Input
          type="text"
          value={modelOverride}
          placeholder="Use active profile default model"
          ariaLabel="Model override"
          oninput={(event) => onChangeModelOverride((event.currentTarget as HTMLInputElement).value)}
        />
      </label>
      <div class="controls-actions">
        <Button onclick={onRunReport} disabled={startingReport || !selectedTemplateId || (analysisScope === "single_source" ? !selectedSourceId : !selectedGroupId)}>
          <Play size={15} aria-hidden="true" />
          {startingReport ? "Starting..." : "Run report"}
        </Button>
        {#if analysisScope === "single_source" && currentSource}
          <Button
            variant="secondary"
            onclick={onOpenNotebookLmExport}
            disabled={exportingNotebookLm}
          >
            <Download size={15} aria-hidden="true" />
            {exportingNotebookLm ? "Exporting..." : "Export for NotebookLM"}
          </Button>
          <Button
            variant="secondary"
            onclick={() => onSyncCurrentSource(currentSource.id)}
            disabled={!!syncingIds[currentSource.id] || sourceSyncDisabledReason(currentSource) !== null}
            title={sourceSyncDisabledReason(currentSource) ?? undefined}
          >
            <RefreshCw size={15} aria-hidden="true" />
            {syncingIds[currentSource.id] ? "Syncing..." : "Sync source"}
          </Button>
        {/if}
      </div>
    </div>

    {#if selectedRunIsActive || currentRun}
      <div class="live-strip">
        <span><strong>Phase:</strong> {phaseLabel(activePhase)}</span>
        {#if activeProgress}
          <span><strong>Progress:</strong> {activeProgress}</span>
        {/if}
        {#if currentRun}
          <span><strong>Provider:</strong> {currentRun.provider}/{currentRun.model}</span>
        {/if}
      </div>
    {/if}
  </div>

  {#if !currentRun && !startingReport}
    <div class="preflight-panel">
      <div class="preflight-copy">
        <span class="eyebrow">Next step</span>
        <h3>Build the first report for this workspace</h3>
        <p>
          Set the date window, choose a prompt template, and start a run. Once the report is ready,
          this area will turn into a live document and follow-up conversation workspace.
        </p>
      </div>
      <div class="preflight-points">
        <div class="preflight-point">
          <strong>1. Scope</strong>
          <span>{analysisScope === "source_group" ? "Run across the saved group." : "Run against the selected source."}</span>
        </div>
        <div class="preflight-point">
          <strong>2. Template</strong>
          <span>{selectedTemplate ? selectedTemplate.name : "Pick a report template to continue."}</span>
        </div>
        <div class="preflight-point">
          <strong>3. Result</strong>
          <span>Inspect trace-backed output here, then continue with grounded chat.</span>
        </div>
      </div>
    </div>
  {/if}

  {#if analysisScope === "single_source" && currentSource}
    {#key sourceContextKey}
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
  {/if}

  <div class="conversation-shell" class:run-open={!!currentRun}>
    <div class="conversation-shell-header">
      <div>
        <span class="eyebrow">Analysis flow</span>
        <h3>Report and follow-up</h3>
        {#if currentRun}
          <p class="conversation-note">
            {#if selectedRunIsActive}
              Live run is open. The report stays primary while chat catches up after completion.
            {:else if currentRun.status === "completed"}
              Opened saved run. Report is ready, and grounded follow-up chat is available below.
            {:else}
              Opened saved run context. Review the report state here before returning to history or trace.
            {/if}
          </p>
        {/if}
      </div>
      <div class="conversation-status">
        {#if currentRun}
          <Badge variant={selectedRunIsActive ? "info" : "neutral"}>
            {selectedRunIsActive ? "Live run open" : "Opened saved run"}
          </Badge>
        {/if}
        <Badge variant={currentRun?.status === "completed" ? "success" : "neutral"}>
          {currentRun?.status === "completed" ? "Ready for chat" : "Chat follows report completion"}
        </Badge>
      </div>
    </div>

  <ReportViewer
    {currentRun}
    {loadingRunDetail}
    streamedOutput={focusedStreamedOutput}
    traceRefCount={traceRefCount}
    {selectedTraceRef}
    livePhase={activePhase}
    liveProgress={activeProgress}
    {canCancelCurrentRun}
    {formatTimestamp}
    {formatPeriod}
    {runTargetLabel}
    {statusTone}
    {reportLines}
    onFocusTraceRef={onFocusTraceRef}
    onCancelCurrentRun={onCancelCurrentRun}
  />

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

  <div class="utility-strip">
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

    <SourceGroupEditor
      compact={true}
      {groups}
      selectedGroupId={selectedGroupId}
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
  .center-pane {
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
    min-width: 0;
  }

  .scope-hero,
  .controls-panel {
    background: var(--panel);
    border: 1px solid var(--border);
    box-shadow: var(--shadow);
    border-radius: 16px;
    padding: 1rem;
  }

  .scope-hero {
    display: flex;
    justify-content: space-between;
    gap: 0.75rem;
    align-items: flex-start;
  }

  .eyebrow {
    display: inline-block;
    font-size: 0.68rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--muted);
    margin-bottom: 0.2rem;
  }

  .scope-hero h2 {
    margin: 0;
  }

  .scope-hero {
    background:
      linear-gradient(180deg, color-mix(in srgb, var(--panel) 98%, white 2%), var(--panel));
  }

  .scope-hero-copy p {
    margin: 0.35rem 0 0 0;
    color: var(--muted);
    line-height: 1.55;
    max-width: 64ch;
  }

  .scope-hero-meta {
    display: flex;
    gap: 0.45rem;
    flex-wrap: wrap;
    justify-content: flex-end;
  }

  .scope-facts {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 0.7rem;
  }

  .scope-fact {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    padding: 0.85rem 0.95rem;
    border-radius: 14px;
    border: 1px solid color-mix(in srgb, var(--border) 82%, transparent);
    background:
      linear-gradient(180deg, color-mix(in srgb, var(--panel-strong) 72%, transparent), transparent);
    box-shadow: var(--shadow-soft);
    min-width: 0;
  }

  .scope-fact-label {
    font-size: 0.68rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--muted);
  }

  .scope-fact strong {
    font-size: 0.88rem;
    line-height: 1.35;
  }

  .controls-panel {
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
    background:
      linear-gradient(180deg, color-mix(in srgb, var(--panel-strong) 54%, transparent), var(--panel));
  }

  .controls-grid {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 0.8rem;
  }

  .controls-bottom {
    display: flex;
    gap: 0.8rem;
    align-items: end;
    justify-content: space-between;
    flex-wrap: wrap;
  }

  .controls-actions {
    display: flex;
    gap: 0.55rem;
    align-items: center;
    flex-wrap: wrap;
  }

  .model-field {
    flex: 1 1 18rem;
    min-width: 16rem;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    font-size: 0.83rem;
    color: var(--muted);
  }

  .live-strip {
    display: flex;
    gap: 0.8rem;
    flex-wrap: wrap;
    align-items: center;
    padding-top: 0.2rem;
    border-top: 1px solid color-mix(in srgb, var(--border) 76%, transparent);
    color: var(--muted);
    font-size: 0.84rem;
  }

  .preflight-panel,
  .conversation-shell {
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
    padding: 1rem;
    border: 1px solid var(--border);
    border-radius: 16px;
    background: var(--panel);
    box-shadow: var(--shadow);
  }

  .preflight-panel {
    background:
      linear-gradient(180deg, color-mix(in srgb, var(--panel-strong) 54%, transparent), var(--panel));
  }

  .preflight-copy h3,
  .conversation-shell-header h3 {
    margin: 0;
  }

  .preflight-copy p {
    margin: 0.35rem 0 0 0;
    color: var(--muted);
    line-height: 1.55;
    max-width: 62ch;
  }

  .preflight-points {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 0.75rem;
  }

  .preflight-point {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    padding: 0.85rem 0.9rem;
    border-radius: 14px;
    border: 1px solid color-mix(in srgb, var(--border) 82%, transparent);
    background: color-mix(in srgb, var(--panel-strong) 72%, transparent);
  }

  .preflight-point strong {
    font-size: 0.84rem;
  }

  .preflight-point span {
    font-size: 0.82rem;
    line-height: 1.45;
    color: var(--muted);
  }

  .conversation-shell {
    background:
      linear-gradient(180deg, color-mix(in srgb, var(--panel) 98%, white 2%), var(--panel));
  }

  .conversation-shell.run-open {
    border-color: color-mix(in srgb, var(--primary) 22%, var(--border));
    box-shadow:
      var(--shadow),
      0 0 0 1px color-mix(in srgb, var(--primary) 10%, transparent);
  }

  .conversation-shell-header {
    display: flex;
    justify-content: space-between;
    gap: 0.75rem;
    align-items: flex-start;
  }

  .conversation-status {
    display: flex;
    justify-content: flex-end;
    flex-wrap: wrap;
    gap: 0.45rem;
  }

  .conversation-note {
    margin: 0.3rem 0 0 0;
    color: var(--muted);
    line-height: 1.45;
    max-width: 58ch;
  }

  .utility-strip {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 0.8rem;
  }

  @media (max-width: 1180px) {
    .preflight-points,
    .scope-facts,
    .controls-grid,
    .utility-strip {
      grid-template-columns: 1fr;
    }
  }

  @media (max-width: 720px) {
    .conversation-shell-header,
    .scope-hero,
    .controls-bottom {
      flex-direction: column;
      align-items: stretch;
    }

    .conversation-status {
      justify-content: flex-start;
    }

    .controls-grid {
      grid-template-columns: 1fr;
    }

    .model-field {
      min-width: 0;
    }

    .scope-hero-meta {
      justify-content: flex-start;
    }
  }
</style>
