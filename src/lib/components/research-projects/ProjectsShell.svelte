<script lang="ts">
  import BottomQueue from "./BottomQueue.svelte";
  import ConnectFromLibrary from "./ConnectFromLibrary.svelte";
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
  let connectOpen = $state(false);

  function openConnectLibrary() {
    connectOpen = true;
  }
</script>

<div class="projects-shell">
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
    <BottomQueue
      loading={workflowState.loading}
      saving={workflowState.saving}
      status={workflowState.status}
      sourceJobs={workflowState.sourceJobs}
      runs={workflowState.runs}
    />
  </section>

  <ConnectFromLibrary
    open={connectOpen}
    project={currentProject}
    librarySources={workflowState.librarySources}
    selectedSourceIds={workflowState.selectedLibrarySourceIds}
    saving={workflowState.saving}
    status={workflowState.status}
    onOpenChange={(open) => (connectOpen = open)}
    onSelectedSourceIdsChange={onSelectedLibrarySourceIdsChange}
    onConnectSelectedSources={onConnectSelectedSources}
  />
</div>

<style>
  .projects-shell {
    display: grid;
    grid-template-columns: 260px minmax(0, 1fr);
    min-height: calc(100vh - 68px);
    background: var(--extractum-surface);
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
