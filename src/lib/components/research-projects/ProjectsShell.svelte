<script lang="ts">
  import { tick } from "svelte";
  import { FolderKanban, ChevronDown, Folder, Plus, Pencil, Trash2 } from "@lucide/svelte";
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
  import { reconcileProjectSourceSelection } from "$lib/ui/research-projects-model";
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
    onSyncSelectedSources,
  }: {
    state: ResearchProjectsWorkflowState;
    showRail?: boolean;
    onSelectProject: (projectId: string) => void;
    onCreateProject: (input: ProjectEditorInput) => void | Promise<void>;
    onUpdateProject: (input: ProjectEditorInput) => void | Promise<void>;
    onDeleteProject: () => void | Promise<void>;
    onRemoveProjectSource: (sourceId: number | number[]) => void | Promise<void>;
    onRunProject: (input: ProjectAnalysisStartCommand) => void | Promise<void>;
    onConnectSelectedSources: () => void | Promise<void>;
    onSelectedLibrarySourceIdsChange: (ids: string[]) => void;
    onRefreshProjectRuns: () => void | Promise<void>;
    onSyncSelectedSources: (sourceIds: number[]) => void | Promise<void>;
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
  let currentProjectSourceLinks = $derived(
    workflowState.projectSourceLinks.filter((source) => source.projectId === workflowState.selectedProjectId),
  );
  let selectedSourceIds = $state<string[]>([]);
  let selectedSource = $derived(
    selectedSourceIds.length > 0
      ? currentProjectSourceLinks.find((source) => source.sourceId === selectedSourceIds[0]) ?? null
      : null,
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

  $effect(() => {
    const nextSelection = reconcileProjectSourceSelection(selectedSourceIds, currentProjectSourceLinks);
    if (
      nextSelection.length !== selectedSourceIds.length ||
      nextSelection.some((id, index) => id !== selectedSourceIds[index])
    ) {
      selectedSourceIds = nextSelection;
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

  async function handleDeleteProject(projectId: string, title: string) {
    onSelectProject(projectId);
    await tick();
    if (confirm(`Are you sure you want to delete "${title}"?`)) {
      await onDeleteProject();
    }
  }

  let contextMenu = $state<{
    visible: boolean;
    x: number;
    y: number;
    projectId: string;
    projectTitle: string;
  } | null>(null);

  function openContextMenu(e: MouseEvent, projectId: string, projectTitle: string) {
    e.preventDefault();
    contextMenu = {
      visible: true,
      x: e.clientX,
      y: e.clientY,
      projectId,
      projectTitle
    };
  }

  function closeContextMenu() {
    contextMenu = null;
  }

  function handleWindowKeyDown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      closeContextMenu();
    }
  }

  function handleEditProjectFromMenu() {
    if (!contextMenu) return;
    onSelectProject(contextMenu.projectId);
    closeContextMenu();
    openEditProject();
  }

  async function handleDeleteProjectFromMenu() {
    if (!contextMenu) return;
    const { projectId, projectTitle } = contextMenu;
    closeContextMenu();
    await handleDeleteProject(projectId, projectTitle);
  }
</script>

<svelte:window onclick={closeContextMenu} onkeydown={handleWindowKeyDown} />

<div class="projects-shell" style="--inspector-width: {inspectorWidth};">
  {#if showRail}
    <aside data-ui-region="project-rail" class="project-rail">
      <ProjectRail
        projects={workflowState.projects}
        selectedProjectId={workflowState.selectedProjectId}
        {onSelectProject}
        onCreateProject={openCreateProject}
      />
    </aside>
  {:else}
    <aside data-ui-region="project-tree-rail" class="project-tree-rail">
      <div class="tree-rail-header">
        <h1>Workspace</h1>
      </div>

      <div class="projects-tree-wrapper">
        <!-- Root Folder: Projects (Always Expanded) -->
        <div class="projects-tree-root-header">
          <span class="folder-chevron-wrapper"><ChevronDown size={12} aria-hidden="true" /></span>
          <span class="folder-icon-wrapper"><Folder size={12} aria-hidden="true" /></span>
          <span class="folder-label">Projects</span>
        </div>

        <!-- List of Projects -->
        <div class="projects-tree-list">
          {#each workflowState.projects as project (project.id)}
            <div
              class="tree-project-item-row group"
              class:active={workflowState.selectedProjectId === project.id}
            >
              <button
                type="button"
                class="tree-project-item"
                onclick={(e) => {
                  e.preventDefault();
                  e.stopPropagation();
                  onSelectProject(project.id);
                }}
                oncontextmenu={(e) => openContextMenu(e, project.id, project.title)}
              >
                <span class="project-dot-indicator" class:running={project.status === "running"} class:ready={project.status === "ready"}></span>
                <span class="project-title-text" title={project.title}>{project.title}</span>
              </button>

              <!-- Inline Action Buttons (Visible on Hover) -->
              <div class="tree-project-actions">
                <button
                  type="button"
                  class="action-btn edit-btn"
                  title="Edit project"
                  onclick={(e) => {
                    e.preventDefault();
                    e.stopPropagation();
                    onSelectProject(project.id);
                    openEditProject();
                  }}
                >
                  <Pencil size={11} aria-hidden="true" />
                </button>
                <button
                  type="button"
                  class="action-btn delete-btn"
                  title="Delete project"
                  onclick={(e) => {
                    e.preventDefault();
                    e.stopPropagation();
                    handleDeleteProject(project.id, project.title);
                  }}
                >
                  <Trash2 size={11} aria-hidden="true" />
                </button>
              </div>
            </div>
          {/each}

          <!-- Add Project Action -->
          <button
            type="button"
            class="tree-add-project-btn"
            onclick={(e) => {
              e.preventDefault();
              e.stopPropagation();
              openCreateProject();
            }}
          >
            <Plus size={12} aria-hidden="true" />
            <span>Create project</span>
          </button>
        </div>
      </div>
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
          saving={workflowState.saving}
          {selectedSourceIds}
          onSelectedSourceIdsChange={(ids) => (selectedSourceIds = ids)}
          onOpenConnectLibrary={openConnectLibrary}
          onRefreshProjectRuns={onRefreshProjectRuns}
          onRemoveSource={onRemoveProjectSource}
          {onSyncSelectedSources}
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

  {#if contextMenu && contextMenu.visible}
    <div
      class="context-menu"
      style="top: {contextMenu.y}px; left: {contextMenu.x}px;"
    >
      <button
        type="button"
        class="context-menu-item"
        onclick={handleEditProjectFromMenu}
      >
        <Pencil size={14} aria-hidden="true" />
        <span>Edit project</span>
      </button>
      <button
        type="button"
        class="context-menu-item delete-item"
        onclick={handleDeleteProjectFromMenu}
      >
        <Trash2 size={14} aria-hidden="true" />
        <span>Delete project</span>
      </button>
    </div>
  {/if}
</div>

<style>
  .projects-shell {
    display: grid;
    grid-template-columns: 260px minmax(0, 1fr) var(--inspector-width, 380px);
    min-height: calc(100vh - 68px);
    background: var(--extractum-surface);
    transition: grid-template-columns 0.2s ease;
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

  .project-tree-rail {
    display: flex;
    min-height: 0;
    flex-direction: column;
    gap: 16px;
    padding: 1.5rem 1rem;
    border-right: 1px solid var(--extractum-border);
    background: var(--extractum-surface-subtle);
  }

  .tree-rail-header h1 {
    font-size: 1.35rem;
    font-weight: 700;
    margin: 0;
    color: var(--extractum-foreground);
    letter-spacing: -0.02em;
  }

  .projects-tree-wrapper {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    padding-left: 0.1rem;
  }

  .projects-tree-root-header {
    display: flex;
    align-items: center;
    gap: 0.45rem;
    padding: 0.25rem 0.2rem;
    color: var(--extractum-muted);
    font-size: 0.82rem;
    font-weight: 600;
    user-select: none;
  }

  .folder-chevron-wrapper {
    color: var(--extractum-muted);
    opacity: 0.8;
    display: inline-flex;
    align-items: center;
  }

  .folder-icon-wrapper {
    color: var(--extractum-primary);
    opacity: 0.9;
    display: inline-flex;
    align-items: center;
  }

  .folder-label {
    text-transform: uppercase;
    font-size: 0.72rem;
    letter-spacing: 0.05em;
  }

  .projects-tree-list {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    padding-left: 0.9rem;
    border-left: 1px dashed var(--extractum-border);
    margin-left: 0.55rem;
  }

  .tree-project-item-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    border-radius: 8px;
    transition: background 0.15s, border-color 0.15s;
    border: 1px solid transparent;
    padding-right: 4px;
  }

  .tree-project-item-row:hover {
    background: color-mix(in srgb, var(--extractum-surface-raised) 50%, transparent);
  }

  .tree-project-item-row.active {
    color: var(--extractum-primary);
    background: color-mix(in srgb, var(--extractum-primary) 10%, transparent);
    border-color: color-mix(in srgb, var(--extractum-primary) 15%, transparent);
    font-weight: 600;
  }

  .tree-project-item {
    display: flex;
    align-items: center;
    gap: 0.45rem;
    padding: 0.35rem 0.5rem;
    color: var(--extractum-muted);
    font-size: 0.8rem;
    font-weight: inherit;
    text-align: left;
    background: transparent;
    border: none;
    cursor: pointer;
    flex: 1;
    min-width: 0;
  }

  .tree-project-item-row:hover .tree-project-item {
    color: var(--extractum-foreground);
  }

  .tree-project-item-row.active .tree-project-item {
    color: var(--extractum-primary);
  }

  .tree-project-actions {
    display: flex;
    align-items: center;
    gap: 2px;
    opacity: 0;
    transition: opacity 0.15s ease;
  }

  .tree-project-item-row:hover .tree-project-actions {
    opacity: 1;
  }

  .action-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    padding: 0;
    border-radius: 4px;
    background: transparent;
    border: none;
    color: var(--extractum-muted);
    cursor: pointer;
    transition: color 0.12s, background 0.12s;
  }

  .action-btn :global(svg) {
    width: 12px !important;
    height: 12px !important;
    stroke: currentColor !important;
    display: block !important;
  }

  .action-btn:hover {
    color: var(--extractum-text);
    background: color-mix(in srgb, var(--extractum-border) 40%, transparent);
  }

  .action-btn.delete-btn:hover {
    color: var(--extractum-danger);
    background: color-mix(in srgb, var(--extractum-danger) 10%, transparent);
  }

  .project-dot-indicator {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--extractum-border);
    flex-shrink: 0;
  }

  .project-dot-indicator.running {
    background: var(--extractum-primary);
    box-shadow: 0 0 8px var(--extractum-primary);
    animation: pulse 2s infinite;
  }

  .project-dot-indicator.ready {
    background: #10b981;
  }

  .project-title-text {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
  }

  .tree-add-project-btn {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0.35rem 0.5rem;
    border-radius: 8px;
    color: var(--extractum-muted);
    font-size: 0.78rem;
    background: transparent;
    border: 1px dashed var(--extractum-border);
    cursor: pointer;
    margin-top: 0.25rem;
    width: fit-content;
    transition: border-color 0.15s, color 0.15s;
  }

  .tree-add-project-btn:hover {
    border-color: var(--extractum-primary);
    color: var(--extractum-primary);
  }

  .context-menu {
    position: fixed;
    z-index: 1000;
    min-width: 150px;
    background: var(--extractum-surface-raised);
    border: 1px solid var(--extractum-border);
    border-radius: var(--extractum-radius);
    box-shadow: var(--shadow-soft);
    padding: 4px;
    display: flex;
    flex-direction: column;
    gap: 2px;
    animation: scaleIn 0.1s ease-out;
  }

  @keyframes scaleIn {
    from {
      opacity: 0;
      transform: scale(0.95);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }

  .context-menu-item {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 6px 12px;
    font-size: 0.8rem;
    font-weight: 500;
    color: var(--extractum-text);
    background: transparent;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    text-align: left;
    transition: background 0.1s, color 0.1s;
  }

  .context-menu-item:hover {
    background: color-mix(in srgb, var(--extractum-primary) 8%, transparent);
    color: var(--extractum-primary);
  }

  .context-menu-item.delete-item {
    color: var(--extractum-danger);
  }

  .context-menu-item.delete-item:hover {
    background: color-mix(in srgb, var(--extractum-danger) 8%, transparent);
    color: var(--extractum-danger);
  }

  .context-menu-item :global(svg) {
    width: 14px !important;
    height: 14px !important;
    stroke: currentColor !important;
    display: block !important;
  }
</style>
