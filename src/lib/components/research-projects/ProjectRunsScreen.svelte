<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { RefreshCw, Save, Trash2, XCircle } from "@lucide/svelte";
  import {
    cancelPromptPackRun,
    deletePromptPackRun,
    listenToPromptPackRunEvents,
    listActivePromptPackRuns,
    listPromptPackRuns,
    updatePromptPackRun,
  } from "$lib/api/prompt-packs";
  import {
    ExtractumBadge,
    ExtractumButton,
    ExtractumDataGrid,
    ExtractumStatusMessage,
    ExtractumTextInput,
    type ExtractumDataGridColumn,
  } from "$lib/components/extractum-ui";
  import {
    retainSelectedRunId,
    statusLabel,
    updateRunListFromEvent,
  } from "$lib/ui/youtube-summary-workflow";
  import { openConfirmModal } from "$lib/modals";
  import type { PromptPackRunListItem, PromptPackRunStatus } from "$lib/types/prompt-packs";
  import ProjectRunReportPanel from "./ProjectRunReportPanel.svelte";

  type ProjectRunGridRow = {
    id: string;
    runId: number;
    runLabel: string;
    projectId: string;
    pack: string;
    status: PromptPackRunStatus;
    resultStatus: string;
    progress: string;
    createdAt: string;
    completedAt: string;
  };

  const columns: ExtractumDataGridColumn[] = [
    { id: "runId", header: "Run", width: 82, sort: true },
    { id: "runLabel", header: "Label", flexgrow: 1, sort: true },
    { id: "projectId", header: "Project", width: 100, sort: true },
    { id: "pack", header: "Pack", width: 170, sort: true },
    { id: "status", header: "Status", width: 120, sort: true },
    { id: "resultStatus", header: "Result", width: 110, sort: true },
    { id: "progress", header: "Progress", width: 110 },
    { id: "createdAt", header: "Created", width: 170, sort: true, dateTimeFormat: "datetime" },
    { id: "completedAt", header: "Completed", width: 170, sort: true, dateTimeFormat: "datetime" },
  ];

  let runs = $state<PromptPackRunListItem[]>([]);
  let selectedRunId = $state<number | null>(null);
  let labelDraft = $state("");
  let loading = $state(false);
  let saving = $state(false);
  let status = $state("");
  let unlisten: (() => void) | null = null;

  const selectedRun = $derived(runs.find((run) => run.runId === selectedRunId) ?? null);
  const selectedRowIds = $derived(selectedRunId === null ? [] : [String(selectedRunId)]);
  const rows = $derived(runs.map(runToGridRow));
  const selectedRunIsActive = $derived(
    selectedRun?.runStatus === "queued" || selectedRun?.runStatus === "running",
  );
  const canDeleteSelectedRun = $derived(Boolean(selectedRun && !selectedRunIsActive));

  $effect(() => {
    labelDraft = selectedRun?.runLabel ?? "";
  });

  onMount(() => {
    void refreshRuns();
    void listenToPromptPackRunEvents((event) => {
      runs = updateRunListFromEvent(runs, event.payload);
    }).then((stop) => {
      unlisten = stop;
    });
  });

  onDestroy(() => {
    unlisten?.();
  });

  function runToGridRow(run: PromptPackRunListItem): ProjectRunGridRow {
    return {
      id: String(run.runId),
      runId: run.runId,
      runLabel: run.runLabel?.trim() || `Run #${run.runId}`,
      projectId: run.projectId === null || run.projectId === undefined ? "None" : `#${run.projectId}`,
      pack: `${run.packId ?? "prompt_pack"}@${run.packVersion ?? "unknown"}`,
      status: run.runStatus,
      resultStatus: run.resultStatus ?? "none",
      progress: progressLabel(run),
      createdAt: run.createdAt ?? "",
      completedAt: run.completedAt ?? "",
    };
  }

  function progressLabel(run: PromptPackRunListItem) {
    if (run.progressCurrent === null || run.progressCurrent === undefined) {
      return run.queuePosition ? `queue ${run.queuePosition}` : "";
    }
    if (run.progressTotal === null || run.progressTotal === undefined) {
      return String(run.progressCurrent);
    }
    return `${run.progressCurrent}/${run.progressTotal}`;
  }

  async function refreshRuns() {
    loading = true;
    status = "";
    try {
      const [recent, active] = await Promise.all([
        listPromptPackRuns({ limit: 100 }),
        listActivePromptPackRuns(),
      ]);
      const nextRuns = [...recent, ...active]
        .filter((run, index, allRuns) => allRuns.findIndex((candidate) => candidate.runId === run.runId) === index)
        .sort((left, right) => right.runId - left.runId);
      runs = nextRuns;
      selectedRunId = retainSelectedRunId(selectedRunId, nextRuns) ?? nextRuns[0]?.runId ?? null;
    } catch (cause) {
      status = cause instanceof Error ? cause.message : String(cause);
    } finally {
      loading = false;
    }
  }

  function selectRows(ids: string[]) {
    const last = ids.at(-1);
    selectedRunId = last ? Number(last) : null;
  }

  async function saveSelectedRun() {
    if (!selectedRun) return;
    saving = true;
    status = "";
    try {
      const updated = await updatePromptPackRun({
        runId: selectedRun.runId,
        runLabel: labelDraft.trim() || null,
      });
      runs = runs.map((run) => (run.runId === updated.runId ? { ...run, ...updated } : run));
      status = "Run label updated.";
    } catch (cause) {
      status = cause instanceof Error ? cause.message : String(cause);
    } finally {
      saving = false;
    }
  }

  async function deleteSelectedRun() {
    if (!selectedRun || !canDeleteSelectedRun) return;
    const run = selectedRun;
    const confirmed = await openConfirmModal({
      title: "Delete project run?",
      message: `Project run #${run.runId} will be removed from the local database.`,
      confirmLabel: "Delete",
      cancelLabel: "Cancel",
      tone: "danger",
    });
    if (!confirmed) return;

    saving = true;
    status = "";
    try {
      await deletePromptPackRun(run.runId);
      const nextRuns = runs.filter((candidate) => candidate.runId !== run.runId);
      runs = nextRuns;
      selectedRunId = nextRuns[0]?.runId ?? null;
      status = "Run deleted.";
    } catch (cause) {
      status = cause instanceof Error ? cause.message : String(cause);
    } finally {
      saving = false;
    }
  }

  async function cancelSelectedRun() {
    if (!selectedRun || !selectedRunIsActive) return;
    const run = selectedRun;
    const confirmed = await openConfirmModal({
      title: "Cancel active run?",
      message: `Project run #${run.runId} is still ${statusLabel(run.runStatus).toLocaleLowerCase()}. Cancel this run now?`,
      confirmLabel: "Cancel run",
      cancelLabel: "Keep running",
      tone: "danger",
    });
    if (!confirmed) return;

    saving = true;
    status = "";
    try {
      await cancelPromptPackRun(run.runId);
      await refreshRuns();
    } catch (cause) {
      status = cause instanceof Error ? cause.message : String(cause);
    } finally {
      saving = false;
    }
  }
</script>

<section class="project-runs-screen" aria-label="Prompt Pack runs">
  <header class="project-runs-header">
    <div>
      <span>Prompt Pack</span>
      <h1>Prompt Pack runs</h1>
    </div>
    <div class="header-actions">
      {#if selectedRun}
        <ExtractumBadge>{statusLabel(selectedRun.runStatus)}</ExtractumBadge>
      {/if}
      <ExtractumButton variant="outline" disabled={loading} onclick={() => void refreshRuns()}>
        <RefreshCw size={14} aria-hidden="true" />
        Refresh
      </ExtractumButton>
    </div>
  </header>

  <section class="runs-grid-panel" aria-label="Prompt Pack runs grid">
    <div class="runs-grid-toolbar">
      <label>
        <span>Run label</span>
        <ExtractumTextInput
          bind:value={labelDraft}
          disabled={!selectedRun || saving}
          placeholder="Run label"
          aria-label="Run label"
        />
      </label>
      <div class="runs-grid-actions">
        <ExtractumButton disabled={!selectedRun || saving} onclick={() => void saveSelectedRun()}>
          <Save size={14} aria-hidden="true" />
          Update
        </ExtractumButton>
        {#if selectedRunIsActive}
          <ExtractumButton variant="outline" disabled={saving} onclick={() => void cancelSelectedRun()}>
            <XCircle size={14} aria-hidden="true" />
            Cancel
          </ExtractumButton>
        {/if}
        <ExtractumButton
          variant="destructive"
          disabled={!canDeleteSelectedRun || saving}
          onclick={() => void deleteSelectedRun()}
        >
          <Trash2 size={14} aria-hidden="true" />
          Delete
        </ExtractumButton>
      </div>
    </div>

    {#if status}
      <ExtractumStatusMessage>{status}</ExtractumStatusMessage>
    {/if}

    <div class="runs-grid">
      <ExtractumDataGrid
        {rows}
        {columns}
        {selectedRowIds}
        ariaLabel="Prompt Pack runs"
        overlay={loading ? "Loading project runs..." : "No project runs yet."}
        onSelectedRowIdsChange={selectRows}
      />
    </div>
  </section>

  {#key selectedRunId}
    <ProjectRunReportPanel run={selectedRun} />
  {/key}
</section>

<style>
  .project-runs-screen {
    display: grid;
    grid-template-rows: auto minmax(260px, 32vh) minmax(0, 1fr);
    min-height: calc(100vh - 68px);
    min-width: 0;
    gap: 14px;
    overflow: hidden;
    padding: 14px;
    background: var(--extractum-surface);
  }

  .project-runs-header,
  .runs-grid-toolbar,
  .runs-grid-actions,
  .header-actions {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .project-runs-header {
    justify-content: space-between;
  }

  .project-runs-header span,
  .runs-grid-toolbar span {
    color: var(--extractum-muted);
    font-size: 11px;
    text-transform: uppercase;
  }

  .project-runs-header h1 {
    margin: 2px 0 0;
    font-size: 20px;
    letter-spacing: 0;
  }

  .runs-grid-panel {
    display: flex;
    min-width: 0;
    min-height: 0;
    flex-direction: column;
    gap: 10px;
    border-bottom: 1px solid var(--extractum-border);
    padding-bottom: 12px;
  }

  .runs-grid-toolbar {
    justify-content: space-between;
  }

  .runs-grid-toolbar label {
    display: grid;
    min-width: min(380px, 100%);
    gap: 4px;
  }

  .runs-grid {
    min-width: 0;
    min-height: 0;
    flex: 1;
  }

  @media (max-width: 860px) {
    .project-runs-screen {
      grid-template-rows: auto 340px minmax(0, 1fr);
      overflow: auto;
    }

    .project-runs-header,
    .runs-grid-toolbar {
      align-items: stretch;
      flex-direction: column;
    }

    .header-actions,
    .runs-grid-actions {
      justify-content: flex-start;
      flex-wrap: wrap;
    }
  }
</style>
