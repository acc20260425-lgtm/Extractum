<script lang="ts">
  import ProjectRow from "./ProjectRow.svelte";
  import { groupProjectRail } from "$lib/ui/research-projects-rail";
  import type { ProjectSummary } from "$lib/types/projects";

  let {
    summaries,
    now,
    onSelect,
  }: {
    summaries: ProjectSummary[];
    now: number;
    onSelect?: (id: number) => void;
  } = $props();

  let sections = $derived(groupProjectRail(summaries, now));
</script>

<div class="project-rail">
  {#if sections.pinned.length > 0}
    <div class="project-rail__section">
      <div class="project-rail__header">Закреплённые</div>
      {#each sections.pinned as row (row.id)}
        <ProjectRow {row} {onSelect} />
      {/each}
    </div>
  {/if}
  {#if sections.normal.length > 0}
    <div class="project-rail__section">
      <div class="project-rail__header">Проекты</div>
      {#each sections.normal as row (row.id)}
        <ProjectRow {row} {onSelect} />
      {/each}
    </div>
  {/if}
  {#if sections.archived.length > 0}
    <div class="project-rail__section">
      <div class="project-rail__header">Архив</div>
      {#each sections.archived as row (row.id)}
        <ProjectRow {row} {onSelect} />
      {/each}
    </div>
  {/if}
</div>

<style>
  .project-rail {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 4px;
  }

  .project-rail__section {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .project-rail__header {
    padding: 4px 8px;
    font: 700 10px/1 var(--extractum-font);
    letter-spacing: 0.05em;
    text-transform: uppercase;
    color: var(--extractum-muted-2);
  }
</style>
