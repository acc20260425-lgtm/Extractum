<script lang="ts">
  import { ExternalLink, RefreshCw, Trash2 } from "@lucide/svelte";
  import { formatPeriod, formatTimestamp, runTargetLabel } from "$lib/analysis-utils";
  import { deleteAnalysisRun } from "$lib/api/analysis-runs";
  import { ExtractumButton } from "$lib/components/extractum-ui";
  import { openConfirmModal } from "$lib/modals";
  import type { AnalysisRunSummary } from "$lib/types/analysis";
  import YoutubeSummaryRunsPanel from "./YoutubeSummaryRunsPanel.svelte";

  let {
    runs,
    loading = false,
    onRefreshProjectRuns,
    projectId = null,
  }: {
    runs: AnalysisRunSummary[];
    loading?: boolean;
    onRefreshProjectRuns: () => void | Promise<void>;
    projectId?: number | null;
  } = $props();

  const sortedRuns = $derived([...runs].sort((left, right) => right.created_at - left.created_at));
  let deletingRunIds = $state<Record<number, boolean>>({});
  let deleteError = $state<string | null>(null);

  function analysisRunHref(run: AnalysisRunSummary) {
    return `/analysis?runId=${run.id}`;
  }

  function isAnalysisRunActive(run: AnalysisRunSummary) {
    return run.status === "queued" || run.status === "running";
  }

  async function deleteProjectRun(run: AnalysisRunSummary) {
    if (isAnalysisRunActive(run) || deletingRunIds[run.id]) return;
    const confirmed = await openConfirmModal({
      title: "Delete project run?",
      message: `Run ${run.id} will be removed with its saved report and artifacts.`,
      confirmLabel: "Delete",
      cancelLabel: "Cancel",
      tone: "danger",
    });
    if (!confirmed) return;
    deletingRunIds = { ...deletingRunIds, [run.id]: true };
    deleteError = null;
    try {
      await deleteAnalysisRun(run.id);
      await onRefreshProjectRuns();
    } catch (cause) {
      deleteError = cause instanceof Error ? cause.message : String(cause);
    } finally {
      const next = { ...deletingRunIds };
      delete next[run.id];
      deletingRunIds = next;
    }
  }
</script>

<section class="project-runs-tab" aria-label="Project runs">
  <div class="runs-toolbar extractum-toolbar-row">
    <div>
      <span>Project runs</span>
      <strong>{sortedRuns.length}</strong>
    </div>
    <ExtractumButton
      variant="outline"
      disabled={loading}
      onclick={onRefreshProjectRuns}
      aria-label="Refresh project runs"
    >
      <RefreshCw size={14} aria-hidden="true" />
      Refresh
    </ExtractumButton>
  </div>

  {#if sortedRuns.length === 0}
    <div class="empty-runs extractum-panel-shell">No project runs yet.</div>
  {:else}
    {#if deleteError}
      <p class="run-error">{deleteError}</p>
    {/if}
    <ul class="runs-list">
      {#each sortedRuns as run (run.id)}
        <li class="extractum-panel-shell">
          <div class="run-copy">
            <div class="run-title">
              <strong>{runTargetLabel(run)}</strong>
              <span class="status-pill" data-status={run.status}>{run.status}</span>
            </div>
            <p>{formatTimestamp(run.created_at)} - {run.provider}/{run.model}</p>
            <p>{run.prompt_template_name ?? "Unknown template"} v{run.prompt_template_version} - {formatPeriod(run.period_from, run.period_to)}</p>
            {#if run.completed_at}
              <p>Completed: {formatTimestamp(run.completed_at)}</p>
            {/if}
            {#if run.error}
              <p class="run-error">{run.error}</p>
            {/if}
          </div>
          <div class="run-actions">
            <a class="analysis-link" href={analysisRunHref(run)} aria-label={`Open report for run ${run.id}`}>
              <ExternalLink size={14} aria-hidden="true" />
              Open report
            </a>
            <ExtractumButton
              class="icon-button danger"
              variant="destructive"
              aria-label={`Delete project run ${run.id}`}
              title="Delete project run"
              disabled={isAnalysisRunActive(run) || deletingRunIds[run.id]}
              onclick={() => void deleteProjectRun(run)}
            >
              <Trash2 size={14} aria-hidden="true" />
            </ExtractumButton>
          </div>
        </li>
      {/each}
    </ul>
  {/if}

  {#key projectId ?? "none"}
    <YoutubeSummaryRunsPanel {projectId} />
  {/key}
</section>

<style>
  .project-runs-tab {
    display: flex;
    min-width: 0;
    flex-direction: column;
    gap: 12px;
    padding-top: 12px;
  }

  .runs-toolbar div {
    display: flex;
    align-items: baseline;
    gap: 8px;
  }

  .runs-toolbar span {
    color: var(--extractum-muted);
    font-size: 11px;
    text-transform: uppercase;
  }

  .runs-list {
    display: flex;
    min-width: 0;
    flex-direction: column;
    gap: 10px;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .runs-list li {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
  }

  .run-copy {
    min-width: 0;
  }

  .run-title {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
  }

  .run-copy p {
    margin: 5px 0 0;
    color: var(--extractum-muted);
    font-size: 13px;
    overflow-wrap: anywhere;
  }

  .run-error {
    color: var(--extractum-danger);
  }

  .status-pill {
    border: 1px solid var(--extractum-border);
    border-radius: 999px;
    padding: 2px 7px;
    color: var(--extractum-muted);
    font-size: 12px;
  }

  .status-pill[data-status="completed"] {
    color: var(--extractum-success);
  }

  .status-pill[data-status="failed"],
  .status-pill[data-status="cancelled"] {
    color: var(--extractum-danger);
  }

  .status-pill[data-status="queued"],
  .status-pill[data-status="running"] {
    color: var(--extractum-info);
  }

  .run-actions {
    display: flex;
    flex-shrink: 0;
    align-items: center;
    gap: 6px;
  }

  .analysis-link {
    display: inline-flex;
    flex-shrink: 0;
    align-items: center;
    gap: 6px;
    border: 1px solid var(--extractum-border);
    border-radius: var(--extractum-radius);
    padding: 6px 8px;
    color: var(--extractum-text);
    font-size: 13px;
    font-weight: 600;
    text-decoration: none;
  }

  .analysis-link:hover {
    background: var(--extractum-surface-subtle);
  }

  :global(.icon-button) {
    min-width: 32px;
    width: 32px;
    padding-inline: 0;
  }

  :global(.icon-button.danger) {
    color: var(--extractum-danger);
    border-color: color-mix(in srgb, var(--extractum-danger) 32%, transparent);
    background: color-mix(in srgb, var(--extractum-danger) 8%, transparent);
  }

  :global(.icon-button.danger:hover:enabled) {
    background: color-mix(in srgb, var(--extractum-danger) 14%, transparent);
  }

  :global(.icon-button.danger svg) {
    color: currentColor;
    stroke: currentColor;
  }

  .empty-runs {
    color: var(--extractum-muted);
  }

  @media (max-width: 720px) {
    .runs-list li,
    .runs-toolbar,
    .run-actions {
      flex-direction: column;
      align-items: stretch;
    }
  }
</style>
