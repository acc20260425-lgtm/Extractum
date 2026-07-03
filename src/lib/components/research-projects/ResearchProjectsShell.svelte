<script lang="ts">
  import type { ComponentProps } from "svelte";
  import Inspector from "./Inspector.svelte";
  import ProjectRailPanel from "./ProjectRailPanel.svelte";
  import ProjectTabs from "./ProjectTabs.svelte";
  import ProjectToolbar from "./ProjectToolbar.svelte";
  import RunDock from "./RunDock.svelte";
  import SourcesBulkBar from "./SourcesBulkBar.svelte";
  import SourcesFilterBar from "./SourcesFilterBar.svelte";
  import SourcesFilterRow from "./SourcesFilterRow.svelte";
  import SourcesGrid from "./SourcesGrid.svelte";
  import type { ProjectSourceRecord } from "$lib/types/projects";

  let {
    railPanel,
    selectedProjectId,
    sources = [],
    selectedSourceIds = [],
    toolbar,
    runDock,
    inspector,
    bulkBar,
    filterBar,
    filterRow,
    tabs,
    sectionPlaceholder = "",
    gridOverlay = "Нет источников",
    activeSourceId = null,
    onActivateSource,
    onSelectedSourceIdsChange,
  }: {
    railPanel: ComponentProps<typeof ProjectRailPanel>;
    selectedProjectId: number | null;
    sources?: ProjectSourceRecord[];
    selectedSourceIds?: string[];
    toolbar?: ComponentProps<typeof ProjectToolbar>;
    runDock?: ComponentProps<typeof RunDock>;
    inspector?: ComponentProps<typeof Inspector>;
    bulkBar?: ComponentProps<typeof SourcesBulkBar>;
    filterBar?: ComponentProps<typeof SourcesFilterBar>;
    filterRow?: ComponentProps<typeof SourcesFilterRow>;
    tabs?: ComponentProps<typeof ProjectTabs>;
    sectionPlaceholder?: string;
    gridOverlay?: string;
    activeSourceId?: string | null;
    onActivateSource?: (id: string) => void;
    onSelectedSourceIdsChange?: (ids: string[]) => void;
  } = $props();
</script>

<div class="research-projects-shell">
  <aside class="research-projects-shell__rail">
    <ProjectRailPanel {...railPanel} />
  </aside>

  <main class="research-projects-shell__main">
    {#if selectedProjectId !== null}
      {#if toolbar}
        <ProjectToolbar {...toolbar} />
      {/if}
      {#if tabs}
        <ProjectTabs {...tabs} />
      {/if}
      {#if sectionPlaceholder}
        <div class="research-projects-shell__section-placeholder">{sectionPlaceholder}</div>
      {:else}
        {#if filterBar}
          <div class="research-projects-shell__statsbar">
            <SourcesFilterBar {...filterBar} />
            {#if bulkBar}
              <SourcesBulkBar {...bulkBar} />
            {/if}
          </div>
        {/if}
        {#if filterRow}
          <SourcesFilterRow {...filterRow} />
        {/if}
        <div class="research-projects-shell__grid">
          <SourcesGrid
            {sources}
            {selectedSourceIds}
            {onSelectedSourceIdsChange}
            {activeSourceId}
            {onActivateSource}
            overlay={gridOverlay}
          />
        </div>
      {/if}
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

  .research-projects-shell__statsbar {
    position: relative;
    flex-shrink: 0;
  }

  .research-projects-shell__section-placeholder {
    flex: 1;
    display: grid;
    place-items: center;
    font: 400 13px/1.4 var(--extractum-font);
    color: var(--extractum-muted-2);
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
