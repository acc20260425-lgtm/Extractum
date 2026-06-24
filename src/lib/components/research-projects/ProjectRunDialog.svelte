<script lang="ts">
  import { ExtractumButton, ExtractumDialog, ExtractumTextInput } from "$lib/components/extractum-ui";
  import { defaultDateOffset, endOfDayUnix, startOfDayUnix } from "$lib/analysis-utils";
  import type { AnalysisPromptTemplate, YoutubeCorpusMode } from "$lib/types/analysis";
  import type { ProjectAnalysisStartCommand } from "$lib/types/projects";
  import type { ResearchProjectView } from "$lib/ui/research-projects-model";

  const PROJECT_RUN_DEFAULT_FROM_DATE = "1970-01-01";

  let {
    open = $bindable(false),
    project,
    templates,
    saving = false,
    onSubmit,
  }: {
    open?: boolean;
    project: ResearchProjectView | null;
    templates: AnalysisPromptTemplate[];
    saving?: boolean;
    onSubmit: (input: ProjectAnalysisStartCommand) => void | Promise<void>;
  } = $props();

  let periodFrom = $state(PROJECT_RUN_DEFAULT_FROM_DATE);
  let periodTo = $state(defaultDateOffset(0));
  let outputLanguage = $state("en");
  let selectedTemplateId = $state("");
  let youtubeCorpusMode = $state<YoutubeCorpusMode>("transcript_description");

  $effect(() => {
    if (!selectedTemplateId && templates[0]) selectedTemplateId = String(templates[0].id);
  });

  async function submit() {
    if (!project || !selectedTemplateId) return;
    await onSubmit({
      projectId: project.projectId,
      periodFrom: startOfDayUnix(periodFrom),
      periodTo: endOfDayUnix(periodTo),
      outputLanguage,
      promptTemplateId: Number(selectedTemplateId),
      modelOverride: null,
      profileId: null,
      youtubeCorpusMode,
      includeMigratedHistory: false,
    });
    open = false;
  }
</script>

<ExtractumDialog bind:open title="Run project analysis">
  <form class="run-dialog" onsubmit={(event) => { event.preventDefault(); void submit(); }}>
    <p>{project?.title ?? "No project selected"}</p>
    <label><span>From</span><ExtractumTextInput type="date" bind:value={periodFrom} /></label>
    <label><span>To</span><ExtractumTextInput type="date" bind:value={periodTo} /></label>
    <label><span>Output language</span><ExtractumTextInput bind:value={outputLanguage} /></label>
    <label>
      <span>Prompt</span>
      <select bind:value={selectedTemplateId} aria-label="Prompt template">
        {#each templates as template (template.id)}
          <option value={String(template.id)}>{template.name}</option>
        {/each}
      </select>
    </label>
    <label>
      <span>YouTube corpus</span>
      <select bind:value={youtubeCorpusMode} aria-label="YouTube corpus mode">
        <option value="transcript_only">Transcript only</option>
        <option value="transcript_description">Transcript and description</option>
        <option value="transcript_description_comments">Transcript, description and comments</option>
      </select>
    </label>
    <footer>
      <ExtractumButton
        type="button"
        variant="outline"
        onclick={() => (open = false)}
        aria-label="Cancel run project analysis"
        title="Cancel run project analysis"
      >
        Cancel
      </ExtractumButton>
      <ExtractumButton
        type="submit"
        disabled={!project || !selectedTemplateId || saving}
        aria-label="Run project analysis"
        title="Run project analysis"
      >
        Run
      </ExtractumButton>
    </footer>
  </form>
</ExtractumDialog>

<style>
  .run-dialog {
    display: grid;
    min-width: min(520px, calc(100vw - 96px));
    gap: 12px;
  }

  p {
    margin: 0;
  }

  label {
    display: grid;
    gap: 6px;
  }

  label span {
    color: var(--extractum-muted);
    font-size: 12px;
  }

  select {
    min-height: 32px;
    border: 1px solid var(--extractum-border);
    border-radius: var(--extractum-radius);
    background: var(--extractum-surface);
    color: var(--extractum-text);
    padding: 4px 8px;
    font-size: 13px;
  }

  footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
  }
</style>
