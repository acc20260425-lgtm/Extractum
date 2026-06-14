<script lang="ts">
  import { Pencil, Play, Trash2 } from "@lucide/svelte";
  import { ExtractumButton, ProviderBadge, StatusBadge } from "$lib/components/extractum-ui";
  import { projectRunDisabledReason } from "$lib/ui/research-projects-model";
  import type { AnalysisRunSummary } from "$lib/types/analysis";
  import type { ProjectSourceRecord } from "$lib/types/projects";
  import type { ProjectSourceLinkView, ResearchProjectView } from "$lib/ui/research-projects-model";
  import YoutubeSummaryRunsPanel from "./YoutubeSummaryRunsPanel.svelte";

  let {
    project,
    sources,
    selectedSource,
    runs,
    saving = false,
    onEditProject,
    onDeleteProject,
    onRunProject,
    onRemoveSource,
  }: {
    project: ResearchProjectView | null;
    sources: ProjectSourceRecord[];
    selectedSource: ProjectSourceLinkView | null;
    runs: AnalysisRunSummary[];
    saving?: boolean;
    onEditProject: () => void;
    onDeleteProject: () => void | Promise<void>;
    onRunProject: () => void;
    onRemoveSource: (sourceId: number) => void | Promise<void>;
  } = $props();

  const mixedProviderRunMessage = "Mixed-provider project runs are not supported yet.";
  const runDisabledReason = $derived(projectRunDisabledReason(project, sources));
  const providerBreakdown = $derived(Array.from(new Set(sources.map((source) => source.provider))));
</script>

<aside class="project-inspector-panel">
  <section>
    <span class="eyebrow">Project</span>
    <h2>{project?.title ?? "No project selected"}</h2>
    <p>{project?.description ?? "Create or select a project."}</p>
    <dl>
      <div><dt>Sources</dt><dd>{sources.length}</dd></div>
      <div><dt>Providers</dt><dd>{providerBreakdown.length}</dd></div>
    </dl>
    <div class="provider-row">
      {#each providerBreakdown as provider (provider)}
        <ProviderBadge {provider} />
      {/each}
    </div>
  </section>

  <section>
    <h3>Actions</h3>
    {#if runDisabledReason}
      {#if runDisabledReason !== mixedProviderRunMessage}
        <p class="hint">{runDisabledReason}</p>
      {:else}
        <p class="hint">{mixedProviderRunMessage}</p>
      {/if}
    {/if}
    <ExtractumButton disabled={saving || runDisabledReason !== null} onclick={onRunProject}>
      <Play size={14} aria-hidden="true" />
      Run project analysis
    </ExtractumButton>
    <ExtractumButton variant="outline" disabled={!project || saving} onclick={onEditProject}>
      <Pencil size={14} aria-hidden="true" />
      Edit project
    </ExtractumButton>
    <ExtractumButton variant="outline" disabled={!project || saving} onclick={onDeleteProject}>
      <Trash2 size={14} aria-hidden="true" />
      Delete project
    </ExtractumButton>
  </section>

  {#if selectedSource}
    <section>
      <h3>Selected source</h3>
      <p><strong>{selectedSource.title}</strong></p>
      <p>{selectedSource.subtitle ?? selectedSource.filterSummary}</p>
      <StatusBadge status="connected" />
      <ExtractumButton variant="outline" disabled={saving} onclick={() => onRemoveSource(selectedSource.sourceNumericId)}>
        Remove from project
      </ExtractumButton>
    </section>
  {/if}

  <section>
    <h3>Recent runs</h3>
    {#each runs.slice(0, 5) as run (run.id)}
      <p>{run.scope_label} - {run.status}</p>
    {:else}
      <p class="hint">No project runs</p>
    {/each}
  </section>

  <section>
    <YoutubeSummaryRunsPanel projectId={project?.projectId ?? null} />
  </section>
</aside>

<style>
  .project-inspector-panel {
    display: flex;
    min-height: 0;
    flex-direction: column;
    gap: 12px;
    padding: 12px;
    border-left: 1px solid var(--extractum-border);
    background: var(--extractum-surface-subtle);
    overflow: auto;
  }

  section {
    display: flex;
    flex-direction: column;
    gap: 8px;
    border: 1px solid var(--extractum-border);
    border-radius: var(--extractum-radius);
    background: var(--extractum-surface-raised);
    padding: 12px;
  }

  h2,
  h3,
  p,
  dl {
    margin: 0;
  }

  .eyebrow,
  .hint,
  dt {
    color: var(--extractum-muted);
    font-size: 12px;
  }

  dl {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
  }

  .provider-row {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
</style>
