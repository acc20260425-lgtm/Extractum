<script lang="ts">
  import ChatPanel from "$lib/components/analysis/chat-panel.svelte";
  import ReportViewer from "$lib/components/analysis/report-viewer.svelte";
  import SourceGroupEditor from "$lib/components/analysis/source-group-editor.svelte";
  import TemplateEditor from "$lib/components/analysis/template-editor.svelte";
  import SourceMessagesPanel from "$lib/components/source-messages-panel.svelte";
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import Select from "$lib/components/ui/Select.svelte";
  import type {
    AnalysisChatTurn,
    AnalysisPromptTemplate,
    AnalysisRunDetail,
    AnalysisSourceGroup,
    AnalysisSourceOption,
  } from "$lib/types/analysis";
  import type { ItemRecord, SourceRecord } from "$lib/types/sources";

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
    groupMemberSourceIds,
    selectedGroup,
    savingGroup,
    deletingGroup,
    sourceMetricsList,
    syncingIds,
    formatTimestamp,
    formatPeriod,
    runTargetLabel,
    statusTone,
    reportLines,
    phaseLabel,
    accountLabel,
    sourceKindLabel,
    sourceSyncDisabledReason,
    startOfDayUnix,
    endOfDayUnix,
    isGroupSourceSelected,
    onChangePeriodFrom,
    onChangePeriodTo,
    onChangeSelectedTemplateId,
    onChangeOutputLanguage,
    onChangeModelOverride,
    onRunReport,
    onSyncCurrentSource,
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
    onToggleGroupSource,
    onStartNewGroup,
    onSaveGroupCopy,
    onSaveGroupChanges,
    onDeleteGroup,
  }: {
    analysisScope: "single_source" | "source_group";
    currentSource: SourceRecord | null;
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
    sourceItems: ItemRecord[];
    loadingItems: boolean;
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
    groupMemberSourceIds: number[];
    selectedGroup: AnalysisSourceGroup | null;
    savingGroup: boolean;
    deletingGroup: boolean;
    sourceMetricsList: AnalysisSourceOption[];
    syncingIds: Record<number, boolean>;
    formatTimestamp: (value: number) => string;
    formatPeriod: (from: number, to: number) => string;
    runTargetLabel: (...args: unknown[]) => string;
    statusTone: (value: string) => "neutral" | "info" | "success" | "warning" | "danger";
    reportLines: (value: string | null | undefined) => string[];
    phaseLabel: (value: string) => string;
    accountLabel: (accountId: number | null) => string;
    sourceKindLabel: (kind: string) => string;
    sourceSyncDisabledReason: (source: SourceRecord) => string | null;
    startOfDayUnix: (value: string) => number;
    endOfDayUnix: (value: string) => number;
    isGroupSourceSelected: (sourceId: number) => boolean;
    onChangePeriodFrom: (value: string) => void;
    onChangePeriodTo: (value: string) => void;
    onChangeSelectedTemplateId: (value: string) => void;
    onChangeOutputLanguage: (value: string) => void;
    onChangeModelOverride: (value: string) => void;
    onRunReport: () => void;
    onSyncCurrentSource: (sourceId: number) => void;
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
    onToggleGroupSource: (sourceId: number) => void;
    onStartNewGroup: () => void;
    onSaveGroupCopy: () => void;
    onSaveGroupChanges: () => void;
    onDeleteGroup: () => void;
  } = $props();
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
        <Badge variant="info">{sourceKindLabel(currentSource.telegram_source_kind)}</Badge>
        <Badge>{accountLabel(currentSource.account_id)}</Badge>
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
          {currentSourceMetric.item_count} synced messages
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
          {startingReport ? "Starting..." : "Run report"}
        </Button>
        {#if analysisScope === "single_source" && currentSource}
          <Button
            variant="secondary"
            onclick={() => onSyncCurrentSource(currentSource.id)}
            disabled={!!syncingIds[currentSource.id] || sourceSyncDisabledReason(currentSource) !== null}
            title={sourceSyncDisabledReason(currentSource) ?? undefined}
          >
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

  {#if analysisScope === "single_source" && currentSource}
    <div class="context-panel">
      <div class="context-panel-header">
        <div>
          <span class="eyebrow">Source context</span>
          <h3>Recent synced messages</h3>
        </div>
        <Badge variant="neutral">
          {currentSourceMetric?.item_count ?? sourceItems.length} messages
        </Badge>
      </div>
      <SourceMessagesPanel
        {loadingItems}
        items={sourceItems}
        formatDate={formatTimestamp}
        embedded={true}
        previewLimit={120}
      />
    </div>
  {/if}

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
      {groupMemberSourceIds}
      sources={sourceMetricsList}
      {savingGroup}
      {deletingGroup}
      {formatTimestamp}
      {isGroupSourceSelected}
      onChangeSelectedGroupId={onChangeSelectedGroupId}
      onChangeGroupName={onChangeGroupName}
      onToggleSource={onToggleGroupSource}
      onStartNewGroup={onStartNewGroup}
      onSaveGroupCopy={onSaveGroupCopy}
      onSaveGroupChanges={onSaveGroupChanges}
      onDeleteGroup={onDeleteGroup}
    />
  </div>
</section>

<style>
  .center-pane {
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
    min-width: 0;
  }

  .scope-hero,
  .controls-panel,
  .context-panel {
    background: var(--panel);
    border: 1px solid var(--border);
    box-shadow: var(--shadow);
    border-radius: 16px;
    padding: 1rem;
  }

  .scope-hero,
  .context-panel-header {
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

  .scope-hero h2,
  .context-panel-header h3 {
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

  .context-panel {
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
    background:
      linear-gradient(180deg, color-mix(in srgb, var(--panel-strong) 44%, transparent), var(--panel));
  }

  .utility-strip {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 0.8rem;
  }

  @media (max-width: 1180px) {
    .scope-facts,
    .controls-grid,
    .utility-strip {
      grid-template-columns: 1fr;
    }
  }

  @media (max-width: 720px) {
    .scope-hero,
    .context-panel-header,
    .controls-bottom {
      flex-direction: column;
      align-items: stretch;
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
