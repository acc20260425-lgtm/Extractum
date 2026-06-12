<script lang="ts">
  import IconRail from "./IconRail.svelte";
  import ProjectRail from "./ProjectRail.svelte";
  import ProjectWorkspace from "./ProjectWorkspace.svelte";
  import TopCommandBar from "./TopCommandBar.svelte";
  import type { ResearchProjectsWorkflowState } from "$lib/ui/research-projects-workflow";

  let {
    state: workflowState,
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
    workflowState.projects.find((project) => project.id === workflowState.selectedProjectId) ?? workflowState.projects[0] ?? null,
  );
  let connectLibraryRequested = $state(false);

  function openConnectLibrary() {
    connectLibraryRequested = true;
  }
</script>

<div class="projects-shell">
  <aside data-ui-region="icon-rail" class="icon-rail">
    <IconRail />
  </aside>

  <aside data-ui-region="project-rail" class="project-rail">
    <ProjectRail
      projects={workflowState.projects}
      selectedProjectId={workflowState.selectedProjectId}
      {onSelectProject}
    />
  </aside>

  <section class="workspace-column">
    <div data-ui-region="top-command-bar">
      <TopCommandBar project={currentProject} loading={workflowState.loading} />
    </div>
    <div data-ui-region="project-workspace" class="workspace-region">
      <ProjectWorkspace
        project={currentProject}
        projectSourceLinks={workflowState.projectSourceLinks}
        librarySources={workflowState.librarySources}
        onOpenConnectLibrary={openConnectLibrary}
      />
    </div>
  </section>

  {#if connectLibraryRequested}
    <span class="sr-only" data-ui-state="connect-library-requested">Connect from Library requested</span>
  {/if}
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

  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }
</style>
