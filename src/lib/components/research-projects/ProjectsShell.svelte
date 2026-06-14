<script lang="ts">
  import BottomQueue from "./BottomQueue.svelte";
  import ConnectFromLibrary from "./ConnectFromLibrary.svelte";
  import ProjectEditorDialog from "./ProjectEditorDialog.svelte";
  import ProjectInspector from "./ProjectInspector.svelte";
  import ProjectRail from "./ProjectRail.svelte";
  import ProjectRunDialog from "./ProjectRunDialog.svelte";
  import ProjectWorkspace from "./ProjectWorkspace.svelte";
  import TopCommandBar from "./TopCommandBar.svelte";
  import type { ProjectAnalysisStartCommand, ProjectEditorInput } from "$lib/types/projects";
  import type { ResearchProjectsWorkflowState } from "$lib/ui/research-projects-workflow";

  let {
    state: workflowState,
    onSelectProject,
    onCreateProject,
    onUpdateProject,
    onDeleteProject,
    onRemoveProjectSource,
    onRunProject,
    onConnectSelectedSources,
    onSelectedLibrarySourceIdsChange,
    onRefreshProjectRuns,
  }: {
    state: ResearchProjectsWorkflowState;
    onSelectProject: (projectId: string) => void;
    onCreateProject: (input: ProjectEditorInput) => void | Promise<void>;
    onUpdateProject: (input: ProjectEditorInput) => void | Promise<void>;
    onDeleteProject: () => void | Promise<void>;
    onRemoveProjectSource: (sourceId: number) => void | Promise<void>;
    onRunProject: (input: ProjectAnalysisStartCommand) => void | Promise<void>;
    onConnectSelectedSources: () => void | Promise<void>;
    onSelectedLibrarySourceIdsChange: (ids: string[]) => void;
    onRefreshProjectRuns: () => void | Promise<void>;
  } = $props();

  let currentProject = $derived(
    workflowState.projects.find((project) => project.id === workflowState.selectedProjectId) ?? workflowState.projects[0] ?? null,
  );
  let currentProjectSources = $derived(
    workflowState.projectSources.filter((source) => currentProject && source.project_id === currentProject.projectId),
  );
  let currentRuns = $derived(
    workflowState.runs.filter((run) => currentProject && run.project_id === currentProject.projectId),
  );
  let selectedSourceId = $state<string | null>(null);
  let selectedSource = $derived(
    workflowState.projectSourceLinks.find((source) => source.sourceId === selectedSourceId) ?? null,
  );
  let connectOpen = $state(false);
  let editorOpen = $state(false);
  let editorMode = $state<"create" | "edit">("create");
  let runOpen = $state(false);

  function openConnectLibrary() {
    connectOpen = true;
  }

  function openCreateProject() {
    editorMode = "create";
    editorOpen = true;
  }

  function openEditProject() {
    editorMode = "edit";
    editorOpen = true;
  }

  async function submitProject(input: ProjectEditorInput) {
    if (editorMode === "edit") {
      await onUpdateProject(input);
    } else {
      await onCreateProject(input);
    }
  }
</script>

<div class="projects-shell">
  <aside data-ui-region="project-rail" class="project-rail">
    <ProjectRail
      projects={workflowState.projects}
      selectedProjectId={workflowState.selectedProjectId}
      {onSelectProject}
      onCreateProject={openCreateProject}
    />
  </aside>

  <section class="workspace-column">
    <div data-ui-region="top-command-bar">
      <TopCommandBar
        project={currentProject}
        sources={currentProjectSources}
        loading={workflowState.loading}
        onRunProject={() => (runOpen = true)}
      />
    </div>
    <div data-ui-region="project-workspace" class="workspace-region">
      <ProjectWorkspace
        project={currentProject}
        projectSourceLinks={workflowState.projectSourceLinks}
        librarySources={workflowState.librarySources}
        runs={currentRuns}
        loading={workflowState.loading}
        {selectedSourceId}
        onSelectedSourceIdChange={(id) => (selectedSourceId = id)}
        onOpenConnectLibrary={openConnectLibrary}
        onRefreshProjectRuns={onRefreshProjectRuns}
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

  <div data-ui-region="project-inspector" class="inspector-region">
    <ProjectInspector
      project={currentProject}
      sources={currentProjectSources}
      selectedSource={selectedSource}
      runs={currentRuns}
      saving={workflowState.saving}
      onEditProject={openEditProject}
      onDeleteProject={onDeleteProject}
      onRunProject={() => (runOpen = true)}
      onRemoveSource={onRemoveProjectSource}
    />
  </div>

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

  <ProjectEditorDialog
    bind:open={editorOpen}
    project={editorMode === "edit" ? currentProject : null}
    saving={workflowState.saving}
    error={workflowState.status}
    onSubmit={submitProject}
  />

  <ProjectRunDialog
    bind:open={runOpen}
    project={currentProject}
    templates={workflowState.promptTemplates}
    saving={workflowState.saving}
    onSubmit={onRunProject}
  />
</div>

<style>
  .projects-shell {
    display: grid;
    grid-template-columns: 260px minmax(0, 1fr) 380px;
    min-height: calc(100vh - 68px);
    background: var(--extractum-surface);
  }

  .project-rail,
  .workspace-column,
  .workspace-region,
  .inspector-region {
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

  @media (max-width: 1180px) {
    .projects-shell {
      grid-template-columns: 240px minmax(0, 1fr);
    }

    .inspector-region {
      grid-column: 1 / -1;
      min-height: 280px;
      border-top: 1px solid var(--extractum-border);
    }
  }
</style>
