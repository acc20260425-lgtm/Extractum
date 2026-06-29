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
    setProjectArchived,
    setProjectPinned,
    startProjectAnalysis,
  } from "$lib/api/projects";
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
</style>
