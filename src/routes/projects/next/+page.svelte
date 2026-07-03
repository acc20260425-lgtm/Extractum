<script lang="ts">
  import { onMount } from "svelte";
  import ResearchProjectsShell from "$lib/components/research-projects/ResearchProjectsShell.svelte";
  import type { ComboOption } from "$lib/components/research-projects/ComboSelect.svelte";
  import type { InspectorSource } from "$lib/components/research-projects/Inspector.svelte";
  import {
    createProjectRailWorkflow,
    type ProjectRailState,
  } from "$lib/ui/research-projects-rail-workflow";
  import { buildPeriodPresets, type PeriodPreset } from "$lib/ui/research-projects-period";
  import { buildSourceRow } from "$lib/ui/research-projects-source-row";
  import ConnectFromLibrary from "$lib/components/research-projects/ConnectFromLibrary.svelte";
  import {
    PROJECT_SECTIONS,
    type ProjectSectionId,
  } from "$lib/components/research-projects/ProjectTabs.svelte";
  import ProjectEditorDialog from "$lib/components/research-projects/ProjectEditorDialog.svelte";
  import { listLibraryCatalog } from "$lib/api/library-sources";
  import {
    buildLibrarySourcesView,
    connectableSelection,
    projectViewId,
  } from "$lib/ui/research-projects-model";
  import {
    buildSourceFilterChips,
    countActiveSourceFilters,
    emptySourceFilters,
    filterProjectSources,
    removeSourceFilterChip,
    type SourceFilters,
  } from "$lib/ui/research-projects-source-filters";
  import type { LibraryCatalogRecord } from "$lib/types/library-sources";
  import {
    addProjectSources,
    createProject,
    deleteProject,
    getProjectDataRange,
    listProjectSources,
    listResearchProjects,
    removeProjectSources,
    setProjectArchived,
    setProjectPinned,
    startProjectAnalysis,
    updateProject,
  } from "$lib/api/projects";
  import { syncYoutubeSource } from "$lib/api/source-jobs";
  import { listAnalysisPromptTemplates } from "$lib/api/analysis-source-groups";
  import type { ProjectSourceRecord } from "$lib/types/projects";

  const now = Math.floor(Date.now() / 1000);

  let railState = $state<ProjectRailState>({
    summaries: [],
    dataRange: null,
    saving: false,
    status: "",
  });
  let selectedProjectId = $state<number | null>(null);
  let sources = $state<ProjectSourceRecord[]>([]);
  let selectedSourceIds = $state<string[]>([]);
  let editorOpen = $state(false);
  let editorProjectId = $state<number | null>(null);
  let filters = $state<SourceFilters>(emptySourceFilters());
  let filtersOpen = $state(false);
  let activeSection = $state<ProjectSectionId>("sources");
  let connectOpen = $state(false);
  let libraryCatalogRecords = $state<LibraryCatalogRecord[]>([]);
  let selectedLibrarySourceIds = $state<Set<string>>(new Set());
  let inspectorOpen = $state(true);
  let promptOptions = $state<ComboOption[]>([]);
  // Model options are loaded from the active LLM profile in a follow-up; for now
  // the selector is empty and the run falls back to the profile default model.
  let modelOptions = $state<ComboOption[]>([]);
  let selectedPeriodId = $state<string | undefined>("all");
  let selectedPromptValue = $state<string | undefined>(undefined);
  let selectedModelValue = $state<string | undefined>(undefined);

  const workflow = createProjectRailWorkflow({
    getState: () => railState,
    patch: (patch) => {
      railState = { ...railState, ...patch };
    },
    listResearchProjects,
    setProjectPinned,
    setProjectArchived,
    getProjectDataRange,
    formatError: (action, error) => `Не удалось выполнить: ${action} (${String(error)})`,
  });

  let selectedProject = $derived(
    railState.summaries.find((summary) => summary.id === selectedProjectId) ?? null,
  );
  let editorProject = $derived.by(() => {
    if (editorProjectId === null) return null;
    const summary = railState.summaries.find((s) => s.id === editorProjectId);
    return summary ? { title: summary.name, description: summary.description } : null;
  });
  let visibleSources = $derived(filterProjectSources(sources, filters));
  let filterChips = $derived(buildSourceFilterChips(filters));
  let filtersActive = $derived(countActiveSourceFilters(filters) > 0);
  let gridOverlay = $derived(
    filtersActive && visibleSources.length === 0
      ? "Под условия ничего не подходит"
      : "Нет источников",
  );
  let sectionPlaceholder = $derived(
    activeSection === "sources"
      ? ""
      : `Раздел «${PROJECT_SECTIONS.find((s) => s.id === activeSection)?.label ?? ""}» в разработке`,
  );
  let librarySources = $derived(
    buildLibrarySourcesView(
      libraryCatalogRecords,
      sources,
      selectedProjectId !== null ? projectViewId(selectedProjectId) : null,
    ),
  );
  let periodPresets = $derived(
    buildPeriodPresets(railState.dataRange ?? { from: null, to: null }, now),
  );
  let selectedPeriod = $derived<PeriodPreset | undefined>(
    periodPresets.find((preset) => preset.id === selectedPeriodId),
  );
  let selectedPromptLabel = $derived(
    promptOptions.find((option) => option.value === selectedPromptValue)?.label ?? "—",
  );
  let selectedSourceRow = $derived.by(() => {
    const id = selectedSourceIds[0];
    if (!id) return null;
    const record = sources.find((source) => String(source.source_id) === id);
    return record ? buildSourceRow(record) : null;
  });
  let syncableIds = $derived(
    sources
      .filter(
        (source) =>
          selectedSourceIds.includes(String(source.source_id)) &&
          source.provider === "youtube" &&
          (source.source_subtype === "video" || source.source_subtype === "playlist"),
      )
      .map((source) => source.source_id),
  );
  let bulkSyncDisabled = $derived(railState.saving || syncableIds.length === 0);
  let bulkSyncTitle = $derived(
    syncableIds.length === 0 ? "Нет источников, поддерживающих синхронизацию" : "",
  );
  let inspectorSource = $derived.by((): InspectorSource | null => {
    const row = selectedSourceRow;
    if (!row) return null;
    return {
      title: row.title,
      handle: row.handle,
      statusLabel: row.statusLabel,
      syncStatus: row.syncStatus,
      materialsLabel: row.materialsLabel,
      lastSyncLabel: row.lastSyncedAt
        ? new Date(row.lastSyncedAt * 1000).toLocaleString("ru-RU")
        : "—",
    };
  });
  let runDisabled = $derived(
    railState.saving || selectedProjectId === null || !selectedPeriod || !selectedPromptValue,
  );

  async function selectProject(id: number) {
    selectedProjectId = id;
    selectedSourceIds = [];
    selectedPeriodId = "all";
    filters = emptySourceFilters();
    filtersOpen = false;
    activeSection = "sources";
    sources = await listProjectSources(id);
    await workflow.loadDataRange({
      projectId: id,
      youtubeCorpusMode: null,
      includeMigratedHistory: false,
    });
  }

  async function runAnalysis() {
    if (selectedProjectId === null || !selectedPeriod || !selectedPromptValue) return;
    await startProjectAnalysis({
      projectId: selectedProjectId,
      periodFrom: selectedPeriod.from,
      periodTo: selectedPeriod.to,
      outputLanguage: "Russian",
      promptTemplateId: Number(selectedPromptValue),
      modelOverride: selectedModelValue ?? null,
      profileId: null,
      youtubeCorpusMode: "transcript_description",
      includeMigratedHistory: false,
    });
    await workflow.reload();
  }

  function clearSelection() {
    selectedSourceIds = [];
  }

  async function syncSelectedSources() {
    if (selectedProjectId === null || syncableIds.length === 0) return;
    railState = { ...railState, saving: true, status: "" };
    try {
      for (const id of syncableIds) {
        await syncYoutubeSource(id, { metadata: true, transcripts: true, comments: false });
      }
      sources = await listProjectSources(selectedProjectId);
    } catch (error) {
      railState = {
        ...railState,
        status: `Не удалось синхронизировать источники (${String(error)})`,
      };
    } finally {
      railState = { ...railState, saving: false };
    }
  }

  async function deleteSelectedSources() {
    if (selectedProjectId === null || selectedSourceIds.length === 0) return;
    railState = { ...railState, saving: true, status: "" };
    try {
      await removeProjectSources({
        projectId: selectedProjectId,
        sourceIds: selectedSourceIds.map((id) => Number(id)),
      });
      selectedSourceIds = [];
      sources = await listProjectSources(selectedProjectId);
      await workflow.reload();
    } catch (error) {
      railState = {
        ...railState,
        status: `Не удалось удалить источники (${String(error)})`,
      };
    } finally {
      railState = { ...railState, saving: false };
    }
  }

  function openCreateProject() {
    editorProjectId = null;
    editorOpen = true;
  }

  function openEditProject(id: number) {
    editorProjectId = id;
    editorOpen = true;
  }

  async function submitProjectEditor(input: { name: string; description: string | null }) {
    railState = { ...railState, saving: true, status: "" };
    try {
      if (editorProjectId === null) {
        await createProject(input);
      } else {
        await updateProject({ projectId: editorProjectId, ...input });
      }
      await workflow.reload();
    } catch (error) {
      railState = { ...railState, status: `Не удалось сохранить проект (${String(error)})` };
    } finally {
      railState = { ...railState, saving: false };
    }
  }

  async function deleteProjectById(id: number) {
    railState = { ...railState, saving: true, status: "" };
    try {
      await deleteProject(id);
      if (selectedProjectId === id) {
        selectedProjectId = null;
        sources = [];
        selectedSourceIds = [];
      }
      await workflow.reload();
    } catch (error) {
      railState = { ...railState, status: `Не удалось удалить проект (${String(error)})` };
    } finally {
      railState = { ...railState, saving: false };
    }
  }

  async function openConnectSources() {
    connectOpen = true;
    if (libraryCatalogRecords.length === 0) {
      try {
        const catalog = await listLibraryCatalog();
        libraryCatalogRecords = catalog.sources;
      } catch (error) {
        railState = { ...railState, status: `Не удалось загрузить библиотеку (${String(error)})` };
      }
    }
  }

  async function connectSelectedLibrarySources() {
    if (selectedProjectId === null) return;
    const sourceIds = connectableSelection(librarySources, selectedLibrarySourceIds).map(
      (source) => source.sourceId,
    );
    if (sourceIds.length === 0) return;
    railState = { ...railState, saving: true, status: "" };
    try {
      await addProjectSources({ projectId: selectedProjectId, sourceIds });
      selectedLibrarySourceIds = new Set();
      connectOpen = false;
      sources = await listProjectSources(selectedProjectId);
      await workflow.reload();
    } catch (error) {
      railState = { ...railState, status: `Не удалось подключить источники (${String(error)})` };
    } finally {
      railState = { ...railState, saving: false };
    }
  }

  onMount(async () => {
    await workflow.reload();
    const templates = await listAnalysisPromptTemplates("report");
    promptOptions = templates.map((template) => ({
      value: String(template.id),
      label: template.name,
    }));
    selectedPromptValue = promptOptions[0]?.value;
  });
</script>

<div class="projects-next">
  <!-- Temporary switch back to the current /projects screen. -->
  <a class="rp-ui-switch" href="/projects">← Старый интерфейс</a>
  <ResearchProjectsShell
    railPanel={{
      summaries: railState.summaries,
      selectedProjectId,
      now,
      onSelect: selectProject,
      onCreate: openCreateProject,
      onEdit: openEditProject,
      onTogglePin: (id, pinned) => void workflow.setPinned(id, pinned),
      onToggleArchive: (id, archived) => void workflow.setArchived(id, archived),
      onDelete: (id) => void deleteProjectById(id),
    }}
    {selectedProjectId}
    sources={visibleSources}
    {selectedSourceIds}
    {gridOverlay}
    tabs={selectedProject
      ? { active: activeSection, onSelect: (id) => (activeSection = id) }
      : undefined}
    sectionPlaceholder={selectedProject ? sectionPlaceholder : ""}
    filterBar={selectedProject && activeSection === "sources"
      ? {
          filtersOpen,
          onToggleFilters: () => (filtersOpen = !filtersOpen),
          chips: filterChips,
          onRemoveChip: (key) => (filters = removeSourceFilterChip(filters, key)),
          filtersActive,
          onClearAll: () => (filters = emptySourceFilters()),
          shownCount: visibleSources.length,
          totalCount: sources.length,
          onAddSource: () => void openConnectSources(),
        }
      : undefined}
    filterRow={selectedProject && activeSection === "sources" && filtersOpen
      ? {
          filters,
          onChange: (next) => (filters = next),
        }
      : undefined}
    onSelectedSourceIdsChange={(ids) => (selectedSourceIds = ids)}
    toolbar={selectedProject
      ? {
          title: selectedProject.name,
          periodPresets,
          selectedPeriodId,
          onSelectPeriod: (preset) => (selectedPeriodId = preset.id),
          promptOptions,
          selectedPromptValue,
          onSelectPrompt: (option) => (selectedPromptValue = option.value),
          modelOptions,
          selectedModelValue,
          onSelectModel: (option) => (selectedModelValue = option.value),
          runDisabled,
          onRun: runAnalysis,
        }
      : undefined}
    runDock={selectedProject
      ? {
          activeRunLabel:
            selectedProject.status === "running" ? `${selectedProject.name} · идёт анализ` : null,
          queueCount: railState.summaries.filter((summary) => summary.status === "running").length,
          onExport: () => {},
        }
      : undefined}
    bulkBar={activeSection === "sources" && selectedSourceIds.length > 0
      ? {
          count: selectedSourceIds.length,
          syncDisabled: bulkSyncDisabled,
          syncTitle: bulkSyncTitle,
          onClear: clearSelection,
          onSync: syncSelectedSources,
          onDelete: deleteSelectedSources,
        }
      : undefined}
    inspector={inspectorSource
      ? {
          open: inspectorOpen,
          selected: inspectorSource,
          periodLabel: selectedPeriod?.label ?? "—",
          promptLabel: selectedPromptLabel,
          modelLabel: selectedModelValue ?? "—",
          onToggle: () => (inspectorOpen = !inspectorOpen),
        }
      : undefined}
  />
  <ProjectEditorDialog
    bind:open={editorOpen}
    project={editorProject}
    saving={railState.saving}
    error={railState.status}
    onSubmit={submitProjectEditor}
  />
  <ConnectFromLibrary
    open={connectOpen}
    project={selectedProject ? { title: selectedProject.name } : null}
    {librarySources}
    selectedSourceIds={selectedLibrarySourceIds}
    saving={railState.saving}
    status={railState.status}
    onOpenChange={(open) => (connectOpen = open)}
    onSelectedSourceIdsChange={(ids) => (selectedLibrarySourceIds = new Set(ids))}
    onConnectSelectedSources={connectSelectedLibrarySources}
  />
</div>

<style>
  .projects-next {
    height: 100vh;
    min-height: 0;
  }

  .rp-ui-switch {
    position: fixed;
    bottom: 6px;
    right: 12px;
    z-index: 9999;
    font: 400 11px/1 var(--extractum-font);
    color: var(--extractum-muted-2);
    text-decoration: none;
    opacity: 0.75;
  }

  .rp-ui-switch:hover {
    opacity: 1;
    text-decoration: underline;
  }
</style>
