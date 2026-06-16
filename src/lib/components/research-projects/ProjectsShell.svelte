<script lang="ts">
  import { FolderKanban } from "@lucide/svelte";
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
  import { projectsSharedState } from "$lib/projects-shared.svelte";

  let {
    state: workflowState,
    showRail = true,
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
    showRail?: boolean;
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
  let inspectorCollapsed = $state(false);
  let inspectorWidth = $derived(inspectorCollapsed ? "40px" : "380px");

  $effect(() => {
    if (projectsSharedState.showCreateDialog) {
      editorMode = "create";
      editorOpen = true;
    }
  });

  $effect(() => {
    if (!editorOpen && editorMode === "create") {
      projectsSharedState.showCreateDialog = false;
    }
  });

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

<div class="projects-shell" style="--inspector-width: {inspectorWidth};" class:no-rail={!showRail}>
  {#if showRail}
    <aside data-ui-region="project-rail" class="project-rail">
      <ProjectRail
        projects={workflowState.projects}
        selectedProjectId={workflowState.selectedProjectId}
        {onSelectProject}
        onCreateProject={openCreateProject}
      />
    </aside>
  {/if}

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
      {#if !showRail && (!workflowState.projects || workflowState.projects.length === 0)}
        <div class="empty-workspace-state">
          <div class="empty-icon-container">
            <FolderKanban size={48} aria-hidden="true" />
          </div>
          <h2>No Research Projects</h2>
          <p>Create a project to start organizing sources, analysis runs, and prompt packs.</p>
          <button
            class="create-project-cta-btn"
            type="button"
            onclick={() => {
              projectsSharedState.showCreateDialog = true;
            }}
          >
            Create your first project
          </button>
        </div>
      {:else}
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
      {/if}
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
      collapsed={inspectorCollapsed}
      onToggleCollapsed={() => (inspectorCollapsed = !inspectorCollapsed)}
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
    grid-template-columns: 260px minmax(0, 1fr) var(--inspector-width, 380px);
    min-height: calc(100vh - 68px);
    background: var(--extractum-surface);
    transition: grid-template-columns 0.2s ease;
  }

  .projects-shell.no-rail {
    grid-template-columns: minmax(0, 1fr) var(--inspector-width, 380px);
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

    .projects-shell.no-rail {
      grid-template-columns: minmax(0, 1fr);
    }

    .inspector-region {
      grid-column: 1 / -1;
      min-height: auto;
      border-top: 1px solid var(--extractum-border);
    }
  }

  .empty-workspace-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    flex: 1;
    padding: 3rem 2rem;
    text-align: center;
    background: var(--extractum-surface-raised);
    border: 1px dashed var(--extractum-border);
    border-radius: var(--extractum-radius);
    margin: 14px;
    height: calc(100% - 28px);
  }

  .empty-icon-container {
    width: 80px;
    height: 80px;
    border-radius: 50%;
    background: color-mix(in srgb, var(--extractum-primary) 8%, transparent);
    color: var(--extractum-primary);
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: 1.5rem;
    border: 1px solid color-mix(in srgb, var(--extractum-primary) 15%, transparent);
  }

  .empty-workspace-state h2 {
    font-size: 1.4rem;
    font-weight: 700;
    margin: 0 0 0.5rem;
    color: var(--extractum-foreground);
  }

  .empty-workspace-state p {
    font-size: 0.9rem;
    color: var(--extractum-muted);
    max-width: 320px;
    margin: 0 0 1.5rem;
    line-height: 1.5;
  }

  .create-project-cta-btn {
    padding: 0.6rem 1.2rem;
    font-size: 0.85rem;
    font-weight: 600;
    background: var(--extractum-primary);
    color: white;
    border: none;
    border-radius: var(--extractum-radius);
    cursor: pointer;
    box-shadow: 0 4px 12px color-mix(in srgb, var(--extractum-primary) 20%, transparent);
    transition: opacity 0.15s, transform 0.1s;
  }

  .create-project-cta-btn:hover {
    opacity: 0.92;
    transform: translateY(-1px);
  }

  .create-project-cta-btn:active {
    transform: translateY(0);
  }
</style>
