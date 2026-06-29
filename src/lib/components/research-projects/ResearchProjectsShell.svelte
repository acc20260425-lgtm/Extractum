<script lang="ts">
  import type { ComponentProps } from "svelte";
  import ProjectRailSections from "./ProjectRailSections.svelte";
  import ProjectToolbar from "./ProjectToolbar.svelte";
  import SourcesGrid from "./SourcesGrid.svelte";
  import type { ProjectSourceRecord, ProjectSummary } from "$lib/types/projects";

  let {
    summaries,
    selectedProjectId,
    now,
    sources = [],
    selectedSourceIds = [],
    toolbar,
    onSelectProject,
    onSelectedSourceIdsChange,
  }: {
    summaries: ProjectSummary[];
    selectedProjectId: number | null;
    now: number;
    sources?: ProjectSourceRecord[];
    selectedSourceIds?: string[];
    toolbar?: ComponentProps<typeof ProjectToolbar>;
    onSelectProject?: (id: number) => void;
    onSelectedSourceIdsChange?: (ids: string[]) => void;
  } = $props();
</script>

<div class="research-projects-shell">
  <aside class="research-projects-shell__rail">
    <div class="research-projects-shell__rail-head">Проекты</div>
    <ProjectRailSections {summaries} {now} onSelect={onSelectProject} />
  </aside>

  <main class="research-projects-shell__main">
    {#if selectedProjectId !== null}
      {#if toolbar}
        <ProjectToolbar {...toolbar} />
      {/if}
      <div class="research-projects-shell__grid">
        <SourcesGrid {sources} {selectedSourceIds} {onSelectedSourceIdsChange} />
      </div>
    {:else}
      <div class="research-projects-shell__empty">Выберите проект</div>
    {/if}
  </main>
</div>

<style>
  .research-projects-shell {
    display: flex;
    height: 100%;
    min-height: 0;
    background: var(--extractum-surface);
  }

  .research-projects-shell__rail {
    width: 252px;
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    min-height: 0;
    overflow: auto;
    border-right: 1px solid var(--extractum-border);
  }

  .research-projects-shell__rail-head {
    padding: 12px 12px 8px;
    font: 700 11px/1 var(--extractum-font);
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--extractum-muted);
  }

  .research-projects-shell__main {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }

  .research-projects-shell__grid {
    flex: 1;
    min-height: 0;
    min-width: 0;
  }

  .research-projects-shell__empty {
    margin: auto;
    font: 400 13px/1.4 var(--extractum-font);
    color: var(--extractum-muted-2);
  }
</style>
