<script lang="ts">
  import { Search } from "@lucide/svelte";
  import { ExtractumButton, ExtractumTextInput, StatusBadge } from "$lib/components/extractum-ui";
  import type { ResearchProjectView } from "$lib/ui/research-projects-model";

  let {
    projects,
    selectedProjectId,
    onSelectProject,
  }: {
    projects: ResearchProjectView[];
    selectedProjectId: string | null;
    onSelectProject: (projectId: string) => void;
  } = $props();

  let query = $state("");
  let visibleProjects = $derived(
    projects.filter((project) => project.title.toLocaleLowerCase().includes(query.toLocaleLowerCase())),
  );
</script>

<aside class="project-rail-panel">
  <div class="project-rail-header">
    <span>Проекты</span>
    <strong>{projects.length}</strong>
  </div>

  <label class="project-search">
    <Search size={14} aria-hidden="true" />
    <ExtractumTextInput bind:value={query} placeholder="Search projects" aria-label="Search projects" />
  </label>

  <div class="project-list" role="listbox" aria-label="Research projects">
    {#each visibleProjects as project (project.id)}
      <ExtractumButton
        variant="ghost"
        class={project.id === selectedProjectId ? "project-row is-selected" : "project-row"}
        aria-pressed={project.id === selectedProjectId}
        onclick={() => onSelectProject(project.id)}
      >
        <span class="project-row-main">
          <strong>{project.title}</strong>
          <span>{project.sourceCount} sources · {project.materialCount} materials</span>
        </span>
        <StatusBadge status={project.status === "needs_attention" ? "error" : project.status === "empty" ? "unavailable" : "active"} />
      </ExtractumButton>
    {:else}
      <p class="empty">No projects</p>
    {/each}
  </div>
</aside>

<style>
  .project-rail-panel {
    display: flex;
    min-height: 0;
    flex-direction: column;
    gap: 10px;
    padding: 12px;
    border-right: 1px solid var(--extractum-border);
    background: var(--extractum-surface-subtle);
  }

  .project-rail-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    color: var(--extractum-muted);
    font-size: 12px;
    text-transform: uppercase;
  }

  .project-search {
    position: relative;
    display: block;
  }

  .project-search :global(svg) {
    position: absolute;
    top: 9px;
    left: 8px;
    color: var(--extractum-muted);
    pointer-events: none;
  }

  .project-search :global(input) {
    padding-left: 28px;
  }

  .project-list {
    display: flex;
    min-height: 0;
    flex: 1;
    flex-direction: column;
    gap: 6px;
    overflow: auto;
  }

  :global(.project-row.extractum-button) {
    display: flex;
    height: auto;
    min-height: 58px;
    justify-content: space-between;
    gap: 8px;
    padding: 8px;
    text-align: left;
  }

  :global(.project-row.is-selected) {
    background: color-mix(in srgb, var(--extractum-primary) 12%, var(--extractum-surface));
  }

  .project-row-main {
    display: flex;
    min-width: 0;
    flex-direction: column;
    gap: 3px;
  }

  .project-row-main strong,
  .project-row-main span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .project-row-main span,
  .empty {
    color: var(--extractum-muted);
    font-size: 12px;
  }
</style>
