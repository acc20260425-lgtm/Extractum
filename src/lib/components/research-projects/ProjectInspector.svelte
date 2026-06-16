<script lang="ts">
  import { ChevronLeft, ChevronRight, Pencil, Play, PlayCircle, Trash2 } from "@lucide/svelte";
  import { ExtractumButton } from "$lib/components/extractum-ui";
  import { projectRunDisabledReason } from "$lib/ui/research-projects-model";
  import type { AnalysisRunSummary } from "$lib/types/analysis";
  import type { ProjectSourceRecord } from "$lib/types/projects";
  import type { ProjectSourceLinkView, ResearchProjectView } from "$lib/ui/research-projects-model";
  import YoutubeSummaryRunDialog from "./YoutubeSummaryRunDialog.svelte";
  import YoutubeSummaryRunsPanel from "./YoutubeSummaryRunsPanel.svelte";

  let {
    project,
    sources,
    selectedSource,
    runs,
    saving = false,
    collapsed = false,
    onToggleCollapsed = () => {},
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
    collapsed?: boolean;
    onToggleCollapsed?: () => void;
    onEditProject: () => void;
    onDeleteProject: () => void | Promise<void>;
    onRunProject: () => void;
    onRemoveSource: (sourceId: number) => void | Promise<void>;
  } = $props();

  const mixedProviderRunMessage = "Mixed-provider project runs are not supported yet.";
  const runDisabledReason = $derived(projectRunDisabledReason(project, sources));
  const providerCount = $derived(new Set(sources.map((source) => source.provider)).size);
  let youtubeSummaryOpen = $state(false);
  let youtubeSummarySource = $derived(
    selectedSource
      ? {
          sourceId: selectedSource.sourceNumericId,
          title: selectedSource.title,
        }
      : null,
  );
  let canRunSelectedYoutubeSummary = $derived(
    Boolean(
      project &&
        selectedSource?.provider === "youtube" &&
        (selectedSource.subtype === "video" || selectedSource.subtype === "playlist") &&
        selectedSource.itemCount > 0,
    ),
  );
</script>

<aside class="project-inspector-panel" class:collapsed={collapsed}>
  <header class="inspector-header">
    {#if !collapsed}
      <span class="eyebrow uppercase">Inspector</span>
    {/if}
    <ExtractumButton variant="ghost" size="icon" class="toggle-collapse-btn" onclick={onToggleCollapsed} aria-label={collapsed ? "Expand inspector" : "Collapse inspector"}>
      {#if collapsed}
        <ChevronLeft size={16} aria-hidden="true" />
      {:else}
        <ChevronRight size={16} aria-hidden="true" />
      {/if}
    </ExtractumButton>
  </header>

  {#if collapsed}
    <div class="vertical-text">Inspector</div>
  {:else}
    <section>
      <span class="eyebrow">Project</span>
      <h2>{project?.title ?? "No project selected"}</h2>
      <p>{project?.description ?? "Create or select a project."}</p>
      <dl>
        <div><dt>Sources</dt><dd>{sources.length}</dd></div>
        <div><dt>Providers</dt><dd>{providerCount}</dd></div>
      </dl>
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
        {#if canRunSelectedYoutubeSummary}
          <ExtractumButton variant="outline" disabled={saving} onclick={() => (youtubeSummaryOpen = true)}>
            <PlayCircle size={14} aria-hidden="true" />
            YouTube Summary
          </ExtractumButton>
        {/if}
        <ExtractumButton variant="outline" disabled={saving} onclick={() => onRemoveSource(selectedSource.sourceNumericId)}>
          Remove from project
        </ExtractumButton>
      </section>
    {/if}

    <YoutubeSummaryRunDialog
      bind:open={youtubeSummaryOpen}
      projectId={project?.projectId ?? null}
      source={youtubeSummarySource}
    />

    <section>
      <h3>Recent runs</h3>
      {#each runs.slice(0, 5) as run (run.id)}
        <p>{run.scope_label} - {run.status}</p>
      {:else}
        <p class="hint">No project runs</p>
      {/each}
    </section>

    <section>
      {#key project?.projectId ?? "none"}
        <YoutubeSummaryRunsPanel projectId={project?.projectId ?? null} />
      {/key}
    </section>
  {/if}
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
    transition: width 0.2s ease, padding 0.2s ease;
  }

  .project-inspector-panel.collapsed {
    width: 40px;
    padding: 8px 4px;
    align-items: center;
    overflow: hidden;
  }

  .inspector-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
  }

  .project-inspector-panel.collapsed .inspector-header {
    justify-content: center;
  }

  :global(.toggle-collapse-btn.extractum-button) {
    width: 24px !important;
    height: 24px !important;
    padding: 0 !important;
  }

  .vertical-text {
    writing-mode: vertical-rl;
    text-transform: uppercase;
    font-size: 11px;
    letter-spacing: 0.1em;
    color: var(--extractum-muted);
    margin-top: 24px;
    user-select: none;
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

</style>
