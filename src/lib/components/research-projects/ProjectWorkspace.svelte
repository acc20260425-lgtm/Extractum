<script lang="ts">
  import {
    ExtractumTabs,
    ExtractumTabsContent,
    ExtractumTabsList,
    ExtractumTabsTrigger,
    StatusBadge,
  } from "$lib/components/extractum-ui";
  import type {
    LibrarySourceView,
    ProjectSourceLinkView,
    ResearchProjectView,
  } from "$lib/ui/research-projects-model";
  import type { AnalysisRunSummary } from "$lib/types/analysis";
  import ProjectRunsTab from "./ProjectRunsTab.svelte";
  import SourcesTab from "./SourcesTab.svelte";

  let {
    project,
    projectSourceLinks,
    librarySources,
    runs,
    loading = false,
    selectedSourceId,
    onSelectedSourceIdChange,
    onOpenConnectLibrary,
    onRefreshProjectRuns,
  }: {
    project: ResearchProjectView | null;
    projectSourceLinks: ProjectSourceLinkView[];
    librarySources: LibrarySourceView[];
    runs: AnalysisRunSummary[];
    loading?: boolean;
    selectedSourceId: string | null;
    onSelectedSourceIdChange: (sourceId: string | null) => void;
    onOpenConnectLibrary: () => void;
    onRefreshProjectRuns: () => void | Promise<void>;
  } = $props();

  let activeTab = $state("sources");
</script>

<section class="project-workspace-panel">
  <header class="workspace-header">
    <div>
      <span>Project workspace</span>
      <h1>{project?.title ?? "Research project"}</h1>
    </div>
    {#if project}
      <StatusBadge status={project.status === "needs_attention" ? "error" : project.status === "empty" ? "unavailable" : "active"} />
    {/if}
  </header>

  <ExtractumTabs bind:value={activeTab} class="workspace-tabs">
    <ExtractumTabsList variant="line">
      <ExtractumTabsTrigger value="overview">Overview</ExtractumTabsTrigger>
      <ExtractumTabsTrigger value="sources">Sources</ExtractumTabsTrigger>
      <ExtractumTabsTrigger value="evidence">Evidence</ExtractumTabsTrigger>
      <ExtractumTabsTrigger value="reports">Reports</ExtractumTabsTrigger>
      <ExtractumTabsTrigger value="runs">Runs</ExtractumTabsTrigger>
      <ExtractumTabsTrigger value="prompts">Prompts</ExtractumTabsTrigger>
    </ExtractumTabsList>

    <ExtractumTabsContent value="overview">
      <div class="overview-grid">
        <div>
          <span>Sources</span>
          <strong>{project?.sourceCount ?? 0}</strong>
        </div>
        <div>
          <span>Evidence</span>
          <strong>{project?.evidenceCount ?? 0}</strong>
        </div>
        <div>
          <span>Materials</span>
          <strong>{project?.materialCount ?? 0}</strong>
        </div>
        <div>
          <span>Last run</span>
          <strong>{project?.lastRunLabel ?? "No runs"}</strong>
        </div>
      </div>
    </ExtractumTabsContent>

    <ExtractumTabsContent value="sources">
      <SourcesTab
        {project}
        {projectSourceLinks}
        {librarySources}
        {selectedSourceId}
        {onSelectedSourceIdChange}
        {onOpenConnectLibrary}
      />
    </ExtractumTabsContent>
    <ExtractumTabsContent value="evidence">
      <div class="placeholder-panel">Evidence inventory is outside this first route shell task.</div>
    </ExtractumTabsContent>
    <ExtractumTabsContent value="reports">
      <div class="placeholder-panel">Reports stay in the legacy analysis workspace until the next slice.</div>
    </ExtractumTabsContent>
    <ExtractumTabsContent value="runs">
      <ProjectRunsTab {runs} {loading} {onRefreshProjectRuns} projectId={project?.projectId ?? null} />
    </ExtractumTabsContent>
    <ExtractumTabsContent value="prompts">
      <div class="placeholder-panel">Prompt controls are represented in the top command bar.</div>
    </ExtractumTabsContent>
  </ExtractumTabs>
</section>

<style>
  .project-workspace-panel {
    display: flex;
    min-height: 0;
    min-width: 0;
    width: 100%;
    flex: 1;
    flex-direction: column;
    gap: 14px;
    overflow: hidden;
    padding: 14px;
    background: var(--extractum-surface);
  }

  .workspace-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
  }

  .workspace-header span,
  .overview-grid span {
    color: var(--extractum-muted);
    font-size: 11px;
    text-transform: uppercase;
  }

  .workspace-header h1 {
    margin: 2px 0 0;
    font-size: 20px;
    letter-spacing: 0;
  }

  :global(.workspace-tabs) {
    min-height: 0;
    min-width: 0;
    width: 100%;
  }

  :global(.workspace-tabs [data-slot="tabs-content"]) {
    min-width: 0;
    max-width: 100%;
    overflow: hidden;
  }

  :global(.workspace-tabs [data-slot="tabs-list"]) {
    display: flex;
    gap: 24px;
    border-bottom: 1px solid var(--extractum-border);
    background: transparent !important;
    padding: 0 0 1px 0 !important;
    width: 100%;
    justify-content: flex-start;
    border-radius: 0 !important;
    height: auto !important;
  }

  :global(.workspace-tabs [data-slot="tabs-trigger"]) {
    background: transparent !important;
    border: none !important;
    box-shadow: none !important;
    padding: 6px 4px 10px 4px !important;
    font-size: 0.9rem !important;
    font-weight: 500 !important;
    color: var(--extractum-muted) !important;
    position: relative;
    cursor: pointer;
    transition: color 0.15s;
    border-radius: 0 !important;
    flex: none !important;
  }

  :global(.workspace-tabs [data-slot="tabs-trigger"]::after) {
    display: none !important;
  }

  :global(.workspace-tabs [data-slot="tabs-trigger"]:hover) {
    color: var(--extractum-text) !important;
  }

  :global(.workspace-tabs [data-slot="tabs-trigger"][data-state="active"]) {
    color: var(--extractum-primary) !important;
    font-weight: 600 !important;
  }

  :global(.workspace-tabs [data-slot="tabs-trigger"][data-state="active"]::after) {
    display: block !important;
    content: "" !important;
    position: absolute !important;
    bottom: 0 !important;
    left: 0 !important;
    right: 0 !important;
    height: 2px !important;
    background: var(--extractum-primary) !important;
    border-radius: 2px !important;
  }

  .overview-grid {
    display: grid;
    grid-template-columns: repeat(4, minmax(120px, 1fr));
    gap: 10px;
    padding-top: 12px;
  }

  .overview-grid div,
  .placeholder-panel {
    border: 1px solid var(--extractum-border);
    border-radius: var(--extractum-radius);
    background: var(--extractum-surface-raised);
    padding: 12px;
  }

  .overview-grid div {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .overview-grid strong {
    font-size: 18px;
  }

  .placeholder-panel {
    margin-top: 12px;
    color: var(--extractum-muted);
  }
</style>
