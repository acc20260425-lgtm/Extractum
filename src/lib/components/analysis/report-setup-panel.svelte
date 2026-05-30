<script lang="ts">
  import { Play, RefreshCw } from "@lucide/svelte";
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import CheckboxRow from "$lib/components/ui/CheckboxRow.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import Select from "$lib/components/ui/Select.svelte";
  import StatusMessage from "$lib/components/ui/StatusMessage.svelte";
  import { sourceCapabilities, sourceKindLabel } from "$lib/source-capabilities";
  import {
    legacyScopeFromWorkspaceSelection,
    type WorkspaceSelection,
  } from "$lib/analysis-workspace-state";
  import type {
    AnalysisPromptTemplate,
    AnalysisSourceGroup,
    AnalysisSourceOption,
    YoutubeCorpusMode,
  } from "$lib/types/analysis";
  import type { LlmProfile, LlmProviderModel } from "$lib/types/llm";
  import type { Source } from "$lib/types/sources";

  let {
    workspaceSelection,
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
    currentScopeHasSavedRuns,
    selectedRunIsActive,
    activeProgress,
    activePhase,
    selectedTemplate,
    syncingIds,
    formatTimestamp,
    formatPeriod,
    phaseLabel,
    accountLabel,
    sourceSyncDisabledReason,
    reportLaunchDisabledReason,
    startOfDayUnix,
    endOfDayUnix,
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
  }: {
    workspaceSelection: WorkspaceSelection;
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
    currentScopeHasSavedRuns: boolean;
    selectedRunIsActive: boolean;
    activeProgress: string;
    activePhase: string;
    selectedTemplate: AnalysisPromptTemplate | null;
    syncingIds: Record<number, boolean>;
    formatTimestamp: (value: number | null) => string;
    formatPeriod: (from: number, to: number) => string;
    phaseLabel: (value: string) => string;
    accountLabel: (accountId: number | null) => string;
    sourceSyncDisabledReason: (source: Source) => string | null;
    reportLaunchDisabledReason: string | null;
    startOfDayUnix: (value: string) => number;
    endOfDayUnix: (value: string) => number;
    onChangePeriodFrom: (value: string) => void;
    onChangePeriodTo: (value: string) => void;
    onChangeSelectedTemplateId: (value: string) => void;
    onChangeOutputLanguage: (value: string) => void;
    onChangeYoutubeCorpusMode: (value: YoutubeCorpusMode) => void;
    onChangeIncludeMigratedHistory: (value: boolean) => void;
    onChangeLlmProfile: (value: string) => void;
    onChangeLlmModel: (value: string) => void;
    onChangeCustomModelOverride: (value: string) => void;
    onRunReport: () => void | Promise<void>;
    onSyncCurrentSource: (sourceId: number) => void | Promise<void>;
  } = $props();

  const PROFILE_DEFAULT_MODEL_OPTION = "__profile_default__";
  const CUSTOM_MODEL_OPTION = "__custom_model__";

  const legacyWorkspaceSelection = $derived(
    legacyScopeFromWorkspaceSelection(workspaceSelection),
  );
  const analysisScope = $derived(legacyWorkspaceSelection.analysisScope);
  const isYoutubeScope = $derived(
    (analysisScope === "single_source" && currentSource?.sourceType === "youtube") ||
      (analysisScope === "source_group" && currentGroup?.source_type === "youtube"),
  );
  const selectedRunProfile = $derived(
    llmProfiles.find((profile) => profile.profile_id === (selectedLlmProfileId || activeLlmProfile)) ??
      null,
  );
  const currentSourceContentLabel = $derived(currentSource ? sourceCapabilities(currentSource).contentLabel : "items");
  const runReportDisabled = $derived(startingReport || reportLaunchDisabledReason !== null);
</script>

<section class="report-setup-panel" aria-label="Report setup">
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
      {#if canIncludeMigratedHistory}
        <CheckboxRow
          title="Include migrated historical scope"
          description="Add imported small-group history to this saved snapshot."
          checked={includeMigratedHistory}
          disabled={startingReport}
          onchange={(event) => onChangeIncludeMigratedHistory((event.currentTarget as HTMLInputElement).checked)}
        />
      {/if}
    </div>

    <div class="controls-bottom">
      <div class="run-model-controls">
        <label>LLM profile
          <Select
            value={selectedLlmProfileId}
            onchange={(event) => onChangeLlmProfile((event.currentTarget as HTMLSelectElement).value)}
          >
            <option value="">Use active profile ({activeLlmProfile || "default"})</option>
            {#each llmProfiles as profile (profile.profile_id)}
              <option value={profile.profile_id}>
                {profile.profile_id} - {profile.provider}/{profile.default_model}
              </option>
            {/each}
          </Select>
        </label>

        <label>Model
          <Select
            value={selectedLlmModel}
            disabled={loadingLlmProviderModels}
            onchange={(event) => onChangeLlmModel((event.currentTarget as HTMLSelectElement).value)}
          >
            <option value={PROFILE_DEFAULT_MODEL_OPTION}>
              Profile default{selectedRunProfile?.default_model ? ` - ${selectedRunProfile.default_model}` : ""}
            </option>
            {#each llmProviderModels as model (model.model)}
              <option value={model.model}>{model.display_name} - {model.model}</option>
            {/each}
            <option value={CUSTOM_MODEL_OPTION}>Custom model...</option>
          </Select>
        </label>

        {#if selectedLlmModel === CUSTOM_MODEL_OPTION}
          <label>Custom model
            <Input
              type="text"
              value={customModelOverride}
              placeholder="gemini-2.5-pro"
              ariaLabel="Custom model"
              oninput={(event) => onChangeCustomModelOverride((event.currentTarget as HTMLInputElement).value)}
            />
          </label>
        {/if}

        {#if llmModelStatus}
          <span class:error={llmModelStatus.startsWith("Error")} class="model-status">
            {llmModelStatus}
          </span>
        {/if}
      </div>
      <div class="controls-actions">
        <Button
          onclick={onRunReport}
          disabled={runReportDisabled}
          title={reportLaunchDisabledReason ?? undefined}
        >
          <Play size={15} aria-hidden="true" />
          {startingReport ? "Starting..." : "Run report"}
        </Button>
        {#if analysisScope === "single_source" && currentSource}
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

    {#if selectedRunIsActive || startingReport}
      <div class="live-strip">
        <span><strong>Phase:</strong> {phaseLabel(activePhase)}</span>
        {#if activeProgress}
          <span><strong>Progress:</strong> {activeProgress}</span>
        {/if}
      </div>
    {/if}

    {#if reportLaunchDisabledReason && !startingReport}
      <StatusMessage tone="error">{reportLaunchDisabledReason}</StatusMessage>
    {/if}
  </div>

  {#if !startingReport && !selectedRunIsActive}
    <div class="preflight-panel">
      <div class="preflight-copy">
        <span class="eyebrow">Next step</span>
        <h3>{currentScopeHasSavedRuns ? "Run another report" : "Start the first report"}</h3>
        <p>
          {currentScopeHasSavedRuns
            ? "Choose the next date window and prompt template. Prior reports stay available in Runs while the new result is generated here."
            : "Set the date window, choose a prompt template, and start a run. Once the report is ready, this area will turn into a live document and follow-up conversation workspace."}
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
</section>

<style>
  .report-setup-panel {
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
    min-width: 0;
  }

  .scope-hero,
  .controls-panel,
  .preflight-panel {
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--panel);
    box-shadow: var(--shadow-soft);
  }

  .scope-hero {
    display: flex;
    justify-content: space-between;
    gap: 1rem;
    padding: 1rem;
  }

  .scope-hero-copy {
    min-width: 0;
  }

  .scope-hero-copy h2,
  .preflight-copy h3,
  p {
    margin: 0;
  }

  .scope-hero-copy p,
  .preflight-copy p,
  .preflight-point span {
    color: var(--muted);
    line-height: 1.45;
  }

  .scope-hero-meta {
    display: flex;
    align-items: flex-start;
    justify-content: flex-end;
    flex-wrap: wrap;
    gap: 0.45rem;
  }

  .eyebrow {
    display: inline-block;
    margin-bottom: 0.25rem;
    color: var(--muted);
    font-size: 0.68rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  .scope-facts {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 0.65rem;
  }

  .scope-fact {
    min-width: 0;
    padding: 0.8rem;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--panel);
  }

  .scope-fact-label {
    display: block;
    margin-bottom: 0.25rem;
    color: var(--muted);
    font-size: 0.72rem;
    text-transform: uppercase;
  }

  .controls-panel {
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
    padding: 1rem;
  }

  .controls-grid,
  .run-model-controls {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 0.75rem;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    min-width: 0;
    color: var(--muted);
    font-size: 0.78rem;
  }

  .controls-bottom {
    display: flex;
    justify-content: space-between;
    gap: 0.9rem;
    align-items: end;
  }

  .run-model-controls {
    flex: 1;
  }

  .model-status {
    align-self: end;
    color: var(--muted);
    font-size: 0.78rem;
  }

  .model-status.error {
    color: var(--danger);
  }

  .controls-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    justify-content: flex-end;
  }

  .live-strip {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;
    padding: 0.7rem 0.85rem;
    border-radius: 8px;
    background: var(--panel-strong);
    color: var(--muted);
    font-size: 0.85rem;
  }

  .preflight-panel {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(260px, 0.8fr);
    gap: 1rem;
    padding: 1rem;
  }

  .preflight-points {
    display: grid;
    gap: 0.65rem;
  }

  .preflight-point {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    padding: 0.7rem;
    border-radius: 8px;
    background: var(--panel-strong);
  }

  @media (max-width: 1100px) {
    .scope-facts,
    .controls-grid,
    .run-model-controls,
    .preflight-panel {
      grid-template-columns: 1fr;
    }

    .scope-hero,
    .controls-bottom {
      flex-direction: column;
      align-items: stretch;
    }

    .controls-actions,
    .scope-hero-meta {
      justify-content: flex-start;
    }
  }
</style>
