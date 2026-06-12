<script lang="ts">
  import IconRail from "./IconRail.svelte";
  import ProjectRail from "./ProjectRail.svelte";
  import ProjectWorkspace from "./ProjectWorkspace.svelte";
  import TopCommandBar from "./TopCommandBar.svelte";
  import type { ResearchProjectsWorkflowState } from "$lib/ui/research-projects-workflow";

  let {
    state,
    onSelectProject,
    onConnectSelectedSources,
    onSelectedLibrarySourceIdsChange,
  }: {
    state: ResearchProjectsWorkflowState;
    onSelectProject: (projectId: string) => void;
    onConnectSelectedSources: () => void | Promise<void>;
    onSelectedLibrarySourceIdsChange: (ids: string[]) => void;
  } = $props();

  let currentProject = $derived(
    state.projects.find((project) => project.id === state.selectedProjectId) ?? state.projects[0] ?? null,
  );
</script>

<div class="projects-shell">
  <aside data-ui-region="icon-rail" class="icon-rail">
    <IconRail />
  </aside>

  <aside data-ui-region="project-rail" class="project-rail">
    <ProjectRail
      projects={state.projects}
      selectedProjectId={state.selectedProjectId}
      {onSelectProject}
    />
  </aside>

  <section class="workspace-column">
    <div data-ui-region="top-command-bar">
      <TopCommandBar project={currentProject} loading={state.loading} />
    </div>
    <div data-ui-region="project-workspace" class="workspace-region">
      <ProjectWorkspace project={currentProject} />
    </div>
  </section>
</div>

<style>
  .projects-shell {
    display: grid;
    grid-template-columns: 56px 260px minmax(0, 1fr);
    min-height: calc(100vh - 68px);
    border: 1px solid var(--extractum-border);
    background: var(--extractum-surface);
  }

  .icon-rail {
    border-right: 1px solid var(--extractum-border);
    background: var(--extractum-surface-raised);
  }

  .project-rail,
  .workspace-column,
  .workspace-region {
    min-width: 0;
    min-height: 0;
  }

  .workspace-column {
    display: flex;
    flex-direction: column;
  }

  .workspace-region {
    display: flex;
    flex: 1;
  }
</style>
