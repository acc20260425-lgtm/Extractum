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
  import {
    getProjectDataRange,
    listProjectSources,
    listResearchProjects,
    removeProjectSources,
    setProjectArchived,
    setProjectPinned,
    startProjectAnalysis,
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
    summaries={railState.summaries}
    {selectedProjectId}
    {now}
    {sources}
    {selectedSourceIds}
    onSelectProject={selectProject}
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
    bulkBar={selectedSourceIds.length > 0
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
