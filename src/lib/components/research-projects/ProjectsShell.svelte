<script lang="ts">
  import { tick, onMount } from "svelte";
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
  import { getLlmProfiles } from "$lib/api/llm";
  import type { LlmProfile } from "$lib/types/llm";

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

  // Roving tabindex: tracks which project owns tabindex=0
  let focusedProjectId = $state<string | null>(workflowState.selectedProjectId ?? null);

  $effect(() => {
    if (workflowState.selectedProjectId) {
      focusedProjectId = workflowState.selectedProjectId;
    }
  });

  $effect(() => {
    const ids = workflowState.projects.map((p) => p.id);
    if (focusedProjectId && !ids.includes(focusedProjectId)) {
      focusedProjectId = ids[0] ?? null;
    } else if (!focusedProjectId && ids.length > 0) {
      focusedProjectId = ids[0];
    }
  });

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

  let llmProfiles = $state<LlmProfile[]>([]);
  let selectedProfileId = $state<string>("");

  onMount(async () => {
    try {
      const state = await getLlmProfiles();
      llmProfiles = state.profiles;
      selectedProfileId = state.active_profile;
    } catch (e) {
      console.error("Failed to load LLM profiles:", e);
    }
  });

  async function handleRunProject(input: ProjectAnalysisStartCommand) {
    await onRunProject({
      ...input,
      profileId: selectedProfileId || null,
    });
  }

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
        <div class="projects-tree-list" role="tree" aria-label="Research projects"
          onkeydown={(e) => {
            const ids = workflowState.projects.map((p) => p.id);
            const current = ids.indexOf(focusedProjectId ?? "");
            let next: number | null = null;
            if (e.key === "ArrowDown") { e.preventDefault(); next = Math.min(current + 1, ids.length - 1); }
            else if (e.key === "ArrowUp") { e.preventDefault(); next = Math.max(current - 1, 0); }
            else if (e.key === "Home") { e.preventDefault(); next = 0; }
            else if (e.key === "End") { e.preventDefault(); next = ids.length - 1; }
            if (next !== null) {
              focusedProjectId = ids[next];
              // selection-follows-focus: arrow navigation selects immediately
              onSelectProject(ids[next]);
              (e.currentTarget as HTMLElement).querySelectorAll<HTMLElement>('[role="treeitem"]')[next]?.focus();
            }
          }}
        >
          {#each workflowState.projects as project (project.id)}
            <div
              role="treeitem"
              tabindex={focusedProjectId === project.id ? 0 : -1}
              class="tree-project-item-row extractum-project-row group"
              class:is-selected={workflowState.selectedProjectId === project.id}
              data-selected={workflowState.selectedProjectId === project.id}
              aria-selected={workflowState.selectedProjectId === project.id}
              onclick={(e) => {
                e.preventDefault();
                e.stopPropagation();
                focusedProjectId = project.id;
                onSelectProject(project.id);
              }}
              onkeydown={(e) => {
                if (e.key === "Enter" || e.key === " ") {
                  e.preventDefault();
                  focusedProjectId = project.id;
                  onSelectProject(project.id);
                }
              }}
              oncontextmenu={(e) => openContextMenu(e, project.id, project.title)}
            >
              <div
                class="tree-project-item"
                aria-hidden="true"
              >
                <span class="project-dot-indicator" class:running={project.status === "running"} class:ready={project.status === "ready"}></span>
                <span class="project-title-text" title={project.title}>{project.title}</span>
              </div>

              <!-- Inline Action Buttons (Visible on Hover) -->
              <div class="tree-project-actions">
                <button
                  type="button"
                  class="action-btn edit-btn"
                  title="Edit project"
                  aria-label={`Edit project ${project.title}`}
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
                  aria-label={`Delete project ${project.title}`}
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
            data-slot="button"
            class="tree-add-project-btn"
            aria-label="Create project"
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
        llmProfiles={llmProfiles}
        bind:selectedProfileId={selectedProfileId}
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
    onSubmit={handleRunProject}
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

  /* ─── Tree design tokens ────────────────────────────────────────────────────
     All intentional deviations from the global design system are declared here.
     Compact desktop-like feel: tighter rows, system font, no item border-radius.
  ──────────────────────────────────────────────────────────────────────────── */
  .project-tree-rail {
    --tree-font-size: 13px;
    --tree-label-font-size: 11px;
    --tree-row-line-height: 20px;
    --tree-item-padding-v: 3px;
    --tree-item-padding-h: 6px;
    --tree-item-radius: 0px;
    --tree-indent: 20px;
    --tree-hover-bg: color-mix(in srgb, var(--extractum-border) 30%, transparent);
    --tree-selected-bg: color-mix(in srgb, var(--extractum-primary) 14%, var(--extractum-surface));
    --tree-selected-accent: inset 2px 0 0 var(--extractum-primary);

    display: flex;
    min-height: 0;
    flex-direction: column;
    gap: 0;
    padding: 0;
    border-right: 1px solid var(--extractum-border);
    background: var(--extractum-surface-subtle);
    font-family: var(--extractum-font);
    font-size: var(--tree-font-size);
  }

  .tree-rail-header h1 {
    font-size: var(--tree-label-font-size);
    font-weight: 700;
    margin: 0;
    padding: 12px 12px 6px;
    color: var(--extractum-foreground);
    letter-spacing: 0.05em;
    text-transform: uppercase;
    opacity: 0.6;
  }

  .projects-tree-wrapper {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
    overflow: auto;
    padding-bottom: 4px;
  }

  .projects-tree-root-header {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: var(--tree-item-padding-v) var(--tree-item-padding-h);
    color: var(--extractum-muted);
    font-size: var(--tree-label-font-size);
    font-weight: 700;
    user-select: none;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    cursor: default;
  }

  .folder-chevron-wrapper {
    color: var(--extractum-muted);
    display: inline-flex;
    align-items: center;
  }

  .folder-icon-wrapper {
    color: var(--extractum-muted);
    display: inline-flex;
    align-items: center;
  }

  .folder-label {
    flex: 1;
  }

  .projects-tree-list {
    display: flex;
    flex-direction: column;
    gap: 0;
    padding-left: var(--tree-indent);
  }

  .tree-project-item-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    border-radius: var(--tree-item-radius);
  }

  .tree-project-item {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: var(--tree-item-padding-v) var(--tree-item-padding-h) var(--tree-item-padding-v) 4px;
    color: var(--extractum-foreground);
    font-size: var(--tree-font-size);
    font-weight: 400;
    text-align: left;
    background: transparent;
    border: none;
    cursor: pointer;
    flex: 1;
    min-width: 0;
    border-radius: var(--tree-item-radius);
    line-height: var(--tree-row-line-height);
    transition: none;
  }

  .tree-project-item-row:hover .tree-project-item,
  .tree-project-item-row.is-selected .tree-project-item {
    color: var(--extractum-foreground);
    font-weight: 400;
  }

  /* Override global row states for the tree zone */
  :global(.project-tree-rail .extractum-project-row:not(.is-selected):hover) {
    background: var(--tree-hover-bg);
    border-color: transparent;
    box-shadow: none;
    border-radius: var(--tree-item-radius);
  }

  :global(.project-tree-rail .extractum-project-row.is-selected) {
    background: var(--tree-selected-bg);
    border-color: transparent;
    box-shadow: var(--tree-selected-accent);
    border-radius: var(--tree-item-radius);
  }

  .tree-project-actions {
    display: flex;
    align-items: center;
    gap: 1px;
    padding-right: 4px;
    opacity: 0;
    transition: opacity 0.1s ease;
  }

  .tree-project-item-row:hover .tree-project-actions {
    opacity: 1;
  }

  .action-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    padding: 0;
    border-radius: 3px;
    background: transparent;
    border: none;
    color: var(--extractum-muted);
    cursor: pointer;
    transition: color 0.1s, background 0.1s;
  }

  .action-btn :global(svg) {
    width: 11px !important;
    height: 11px !important;
    stroke: currentColor !important;
    display: block !important;
  }

  .action-btn:hover {
    color: var(--extractum-foreground);
    background: color-mix(in srgb, var(--extractum-border) 50%, transparent);
  }

  .action-btn.delete-btn:hover {
    color: var(--extractum-danger);
    background: color-mix(in srgb, var(--extractum-danger) 10%, transparent);
  }

  .project-dot-indicator {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: color-mix(in srgb, var(--extractum-muted) 35%, transparent);
    flex-shrink: 0;
  }

  .project-dot-indicator.running {
    background: var(--extractum-primary);
    box-shadow: 0 0 5px color-mix(in srgb, var(--extractum-primary) 50%, transparent);
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
    gap: 5px;
    padding: var(--tree-item-padding-v) var(--tree-item-padding-h) var(--tree-item-padding-v) 4px;
    border-radius: var(--tree-item-radius);
    color: var(--extractum-muted);
    font-size: var(--tree-font-size);
    font-family: inherit;
    background: transparent;
    border: none;
    cursor: pointer;
    width: 100%;
    line-height: var(--tree-row-line-height);
    transition: color 0.1s, background 0.1s;
    margin-top: 2px;
  }

  .tree-add-project-btn:hover {
    color: var(--extractum-foreground);
    background: var(--tree-hover-bg);
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
