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
          </div>
        {/if}
        {#if bulkBar}
          <div class="research-projects-shell__bulkbar">
            <SourcesBulkBar {...bulkBar} />
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
    {#if inspector.open !== false}
      <button
        class="research-projects-shell__inspector-backdrop"
        data-slot="button"
        type="button"
        aria-label="Закрыть инспектор"
        onclick={() => inspector.onToggle?.()}
      ></button>
    {/if}
    <aside
      class="research-projects-shell__inspector"
      class:research-projects-shell__inspector--open={inspector.open !== false}
      class:research-projects-shell__inspector--closed={inspector.open === false}
    >
      <Inspector {...inspector} />
    </aside>
  {/if}
</div>

<style>
  .research-projects-shell {
    position: relative;
    display: flex;
    height: 100%;
    min-height: 0;
    overflow: hidden;
    container-type: inline-size;
    container-name: app;
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
    container-type: inline-size;
    container-name: sources;
    background: var(--extractum-surface);
  }

  .research-projects-shell__statsbar {
    flex-shrink: 0;
    min-height: 42px;
  }

  .research-projects-shell__bulkbar {
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
    background: var(--extractum-surface-raised);
  }

  .research-projects-shell__inspector {
    position: relative;
    z-index: 1;
    flex: 0 0 45px;
    width: 45px;
    min-height: 0;
    overflow: hidden;
    background: var(--extractum-surface-subtle);
    transition: width 0.18s ease, flex-basis 0.18s ease;
  }

  .research-projects-shell__inspector--open {
    flex-basis: 301px;
    width: 301px;
  }

  .research-projects-shell__inspector :global(.inspector) {
    width: 100%;
  }

  .research-projects-shell__inspector-backdrop {
    display: none;
  }

  .research-projects-shell__empty {
    margin: auto;
    font: 400 13px/1.4 var(--extractum-font);
    color: var(--extractum-muted-2);
  }

  @container app (max-width: 1160px) {
    .research-projects-shell__inspector-backdrop {
      position: absolute;
      inset: 0;
      z-index: 19;
      display: block;
      padding: 0;
      border: 0;
      border-radius: 0;
      background: rgba(23, 33, 43, 0.18);
      cursor: default;
    }

    .research-projects-shell__inspector--open {
      position: absolute;
      top: 0;
      right: 0;
      bottom: 0;
      z-index: 20;
      width: 324px;
      box-shadow: -10px 0 30px rgba(23, 33, 43, 0.16);
    }
  }
</style>
