<script lang="ts">
  import type { ComponentProps } from "svelte";
  import Inspector from "./Inspector.svelte";
  import ProjectRailSections from "./ProjectRailSections.svelte";
  import ProjectToolbar from "./ProjectToolbar.svelte";
  import RunDock from "./RunDock.svelte";
  import SourcesBulkBar from "./SourcesBulkBar.svelte";
  import SourcesGrid from "./SourcesGrid.svelte";
  import type { ProjectSourceRecord, ProjectSummary } from "$lib/types/projects";

  let {
    summaries,
    selectedProjectId,
    now,
    sources = [],
    selectedSourceIds = [],
    toolbar,
    runDock,
    inspector,
    bulkBar,
    onSelectProject,
    onSelectedSourceIdsChange,
  }: {
    summaries: ProjectSummary[];
    selectedProjectId: number | null;
    now: number;
    sources?: ProjectSourceRecord[];
    selectedSourceIds?: string[];
    toolbar?: ComponentProps<typeof ProjectToolbar>;
    runDock?: ComponentProps<typeof RunDock>;
    inspector?: ComponentProps<typeof Inspector>;
    bulkBar?: ComponentProps<typeof SourcesBulkBar>;
    onSelectProject?: (id: number) => void;
    onSelectedSourceIdsChange?: (ids: string[]) => void;
  } = $props();
</script>

<div class="research-projects-shell">
  <aside class="research-projects-shell__rail">
    <ProjectRailSections {summaries} {now} onSelect={onSelectProject} />
  </aside>

  <main class="research-projects-shell__main">
    {#if selectedProjectId !== null}
      {#if toolbar}
        <ProjectToolbar {...toolbar} />
      {/if}
      {#if bulkBar}
        <SourcesBulkBar {...bulkBar} />
      {/if}
      <div class="research-projects-shell__grid">
        <SourcesGrid {sources} {selectedSourceIds} {onSelectedSourceIdsChange} />
      </div>
      {#if runDock}
        <RunDock {...runDock} />
      {/if}
    {:else}
      <div class="research-projects-shell__empty">Выберите проект</div>
    {/if}
  </main>

  {#if inspector}
    <Inspector {...inspector} />
  {/if}
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
