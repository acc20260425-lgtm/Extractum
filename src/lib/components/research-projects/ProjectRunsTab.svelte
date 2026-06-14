<script lang="ts">
  import { ExternalLink, RefreshCw } from "@lucide/svelte";
  import { formatPeriod, formatTimestamp, runTargetLabel } from "$lib/analysis-utils";
  import { ExtractumButton } from "$lib/components/extractum-ui";
  import type { AnalysisRunSummary } from "$lib/types/analysis";

  let {
    runs,
    loading = false,
    onRefreshProjectRuns,
  }: {
    runs: AnalysisRunSummary[];
    loading?: boolean;
    onRefreshProjectRuns: () => void | Promise<void>;
  } = $props();

  const sortedRuns = $derived([...runs].sort((left, right) => right.created_at - left.created_at));
</script>

<section class="project-runs-tab" aria-label="Project runs">
  <div class="runs-toolbar">
    <div>
      <span>Project runs</span>
      <strong>{sortedRuns.length}</strong>
    </div>
    <ExtractumButton variant="outline" disabled={loading} onclick={onRefreshProjectRuns}>
      <RefreshCw size={14} aria-hidden="true" />
      Refresh
    </ExtractumButton>
  </div>

  {#if sortedRuns.length === 0}
    <div class="empty-runs">No project runs yet.</div>
  {:else}
    <ul class="runs-list">
      {#each sortedRuns as run (run.id)}
        <li>
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
          <a class="analysis-link" href="/analysis" aria-label={`Open Analysis workspace for run ${run.id}`}>
            <ExternalLink size={14} aria-hidden="true" />
            Open Analysis workspace
          </a>
        </li>
      {/each}
    </ul>
  {/if}
</section>

<style>
  .project-runs-tab {
    display: flex;
    min-width: 0;
    flex-direction: column;
    gap: 12px;
    padding-top: 12px;
  }

  .runs-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
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

  .runs-list li,
  .empty-runs {
    border: 1px solid var(--extractum-border);
    border-radius: var(--extractum-radius);
    background: var(--extractum-surface-raised);
    padding: 12px;
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

  .empty-runs {
    color: var(--extractum-muted);
  }

  @media (max-width: 720px) {
    .runs-list li,
    .runs-toolbar {
      flex-direction: column;
      align-items: stretch;
    }
  }
</style>
