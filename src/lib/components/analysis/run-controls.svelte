<script lang="ts">
  import Button from "$lib/components/ui/Button.svelte";
  import Card from "$lib/components/ui/Card.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import Select from "$lib/components/ui/Select.svelte";
  import type {
    AnalysisPromptTemplate,
    AnalysisSourceGroup,
    AnalysisSourceOption,
  } from "$lib/types/analysis";

  let {
    analysisScope,
    selectedSourceId,
    selectedGroupId,
    selectedTemplateId,
    periodFrom,
    periodTo,
    outputLanguage,
    modelOverride,
    sources,
    groups,
    templates,
    loadingSources,
    loadingGroups,
    loadingTemplates,
    launching,
    activePhase,
    activeProgress,
    showRunMeta,
    selectedGroupSourceCount,
    phaseLabel,
    onChangeScope,
    onChangeSelectedSourceId,
    onChangeSelectedGroupId,
    onChangePeriodFrom,
    onChangePeriodTo,
    onChangeOutputLanguage,
    onChangeSelectedTemplateId,
    onChangeModelOverride,
    onRunReport,
  }: {
    analysisScope: "single_source" | "source_group";
    selectedSourceId: string;
    selectedGroupId: string;
    selectedTemplateId: string;
    periodFrom: string;
    periodTo: string;
    outputLanguage: string;
    modelOverride: string;
    sources: AnalysisSourceOption[];
    groups: AnalysisSourceGroup[];
    templates: AnalysisPromptTemplate[];
    loadingSources: boolean;
    loadingGroups: boolean;
    loadingTemplates: boolean;
    launching: boolean;
    activePhase: string;
    activeProgress: string;
    showRunMeta: boolean;
    selectedGroupSourceCount: number | null;
    phaseLabel: (phase: string) => string;
    onChangeScope: (scope: "single_source" | "source_group") => void;
    onChangeSelectedSourceId: (value: string) => void;
    onChangeSelectedGroupId: (value: string) => void;
    onChangePeriodFrom: (value: string) => void;
    onChangePeriodTo: (value: string) => void;
    onChangeOutputLanguage: (value: string) => void;
    onChangeSelectedTemplateId: (value: string) => void;
    onChangeModelOverride: (value: string) => void;
    onRunReport: () => void | Promise<void>;
  } = $props();

  function canRunReport() {
    if (launching || !selectedTemplateId) return false;
    return analysisScope === "single_source" ? !!selectedSourceId : !!selectedGroupId;
  }
</script>

<Card>
  <div class="controls">
    <h3>Run Report</h3>

    <div class="scope-toggle">
      <Button
        selected={analysisScope === "single_source"}
        variant="secondary"
        type="button"
        onclick={() => onChangeScope("single_source")}
      >
        Single source
      </Button>
      <Button
        selected={analysisScope === "source_group"}
        variant="secondary"
        type="button"
        onclick={() => onChangeScope("source_group")}
      >
        Source group
      </Button>
    </div>

    {#if analysisScope === "single_source"}
      <label>Source
        <Select
          value={selectedSourceId}
          disabled={loadingSources}
          onchange={(event) => onChangeSelectedSourceId((event.currentTarget as HTMLSelectElement).value)}
        >
          {#if loadingSources}
            <option value="">Loading synced sources...</option>
          {:else if sources.length === 0}
            <option value="">No synced sources available</option>
          {/if}
          {#each sources as source (source.id)}
            <option value={String(source.id)}>
              {(source.title ?? `Source ${source.id}`)} - {source.item_count} messages
            </option>
          {/each}
        </Select>
      </label>
    {:else}
      <label>Source group
        <Select
          value={selectedGroupId}
          disabled={loadingGroups}
          onchange={(event) => onChangeSelectedGroupId((event.currentTarget as HTMLSelectElement).value)}
        >
          {#if loadingGroups}
            <option value="">Loading source groups...</option>
          {:else if groups.length === 0}
            <option value="">No saved groups available</option>
          {/if}
          {#each groups as group (group.id)}
            <option value={String(group.id)}>
              {group.name} - {group.members.length} sources
            </option>
          {/each}
        </Select>
      </label>

      {#if selectedGroupSourceCount !== null}
        <p class="sub">
          {selectedGroupSourceCount} sources selected for this group report.
        </p>
      {/if}
    {/if}

    <div class="grid">
      <label>From
        <Input
          type="date"
          value={periodFrom}
          oninput={(event) => onChangePeriodFrom((event.currentTarget as HTMLInputElement).value)}
        />
      </label>

      <label>To
        <Input
          type="date"
          value={periodTo}
          oninput={(event) => onChangePeriodTo((event.currentTarget as HTMLInputElement).value)}
        />
      </label>
    </div>

    <label>Output language
      <Input
        type="text"
        value={outputLanguage}
        placeholder="Russian"
        oninput={(event) => onChangeOutputLanguage((event.currentTarget as HTMLInputElement).value)}
      />
    </label>

    <label>Prompt template
      <Select
        value={selectedTemplateId}
        disabled={loadingTemplates}
        onchange={(event) => onChangeSelectedTemplateId((event.currentTarget as HTMLSelectElement).value)}
      >
        {#if loadingTemplates}
          <option value="">Loading report templates...</option>
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

    <label>Model override
      <Input
        type="text"
        value={modelOverride}
        placeholder="Use active profile default model"
        oninput={(event) => onChangeModelOverride((event.currentTarget as HTMLInputElement).value)}
      />
    </label>

    <Button onclick={onRunReport} disabled={!canRunReport()}>
      {launching ? "Starting..." : "Run report"}
    </Button>

    {#if showRunMeta}
      <div class="meta-panel">
        <div><strong>Phase:</strong> {phaseLabel(activePhase)}</div>
        {#if activeProgress}
          <div><strong>Progress:</strong> {activeProgress}</div>
        {/if}
      </div>
    {/if}
  </div>
</Card>

<style>
  .controls {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    min-width: 0;
  }

  .scope-toggle {
    display: flex;
    gap: 0.6rem;
    flex-wrap: wrap;
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 0.8rem;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    font-size: 0.9rem;
    color: var(--muted);
    min-width: 0;
  }

  .sub {
    margin: 0;
    color: var(--muted);
    font-size: 0.9rem;
  }

  .meta-panel {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    padding: 0.8rem 1rem;
    border-radius: 10px;
    background: var(--panel-strong);
    border: 1px solid var(--border);
    color: var(--muted);
    font-size: 0.9rem;
  }

  @media (max-width: 720px) {
    .grid {
      grid-template-columns: 1fr;
    }
  }
</style>
