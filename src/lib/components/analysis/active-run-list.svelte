<script lang="ts">
  import type { AnalysisRunSummary } from "$lib/types/analysis";

  let {
    activeRuns,
    loadingActiveRuns,
    activeRunId,
    formatTimestamp,
    formatPeriod,
    phaseLabel,
    livePhase,
    liveProgress,
    runTargetLabel,
    statusTone,
    onRefresh,
    onOpenRun,
    onCancelRun,
  }: {
    activeRuns: AnalysisRunSummary[];
    loadingActiveRuns: boolean;
    activeRunId: number | null;
    formatTimestamp: (timestamp: number | null) => string;
    formatPeriod: (periodFromUnix: number, periodToUnix: number) => string;
    phaseLabel: (phase: string) => string;
    livePhase: (runId: number) => string;
    liveProgress: (runId: number) => string;
    runTargetLabel: (
      run: Pick<
        AnalysisRunSummary,
        "scope_type" | "source_id" | "source_title" | "source_group_id" | "source_group_name" | "scope_label"
      >
    ) => string;
    statusTone: (status: string) => string;
    onRefresh: () => void | Promise<void>;
    onOpenRun: (runId: number) => void | Promise<void>;
    onCancelRun: (runId: number) => void | Promise<void>;
  } = $props();
</script>

<section class="card active-runs">
  <div class="panel-header">
    <div>
      <h3>Active Runs</h3>
      <p class="sub">Queued and running reports stay separate from historical saved runs.</p>
    </div>
    <button class="secondary" onclick={onRefresh}>Refresh</button>
  </div>

  {#if loadingActiveRuns}
    <p class="empty">Loading active runs...</p>
  {:else if activeRuns.length === 0}
    <p class="empty">No queued or running analysis runs.</p>
  {:else}
    <ul class="run-list">
      {#each activeRuns as run (run.id)}
        <li class:selected={run.id === activeRunId}>
          <div class="run-copy">
            <div class="run-title">
              <strong>{runTargetLabel(run)}</strong>
              <span class={`badge badge-${statusTone(run.status)}`}>{run.status}</span>
            </div>
            <p class="sub">
              {formatTimestamp(run.created_at)} - {run.provider}/{run.model} - {run.prompt_template_name ?? "Unknown template"} v{run.prompt_template_version}
            </p>
            <p class="sub">Period: {formatPeriod(run.period_from, run.period_to)}</p>
            <p class="sub">
              Phase: {phaseLabel(livePhase(run.id) || run.status)}
              {#if liveProgress(run.id)}
                - {liveProgress(run.id)}
              {/if}
            </p>
          </div>
          <div class="run-actions">
            <button class="secondary" onclick={() => onOpenRun(run.id)}>Open</button>
            <button class="danger-soft" onclick={() => onCancelRun(run.id)}>Cancel</button>
          </div>
        </li>
      {/each}
    </ul>
  {/if}
</section>

<style>
  .card {
    background: var(--panel);
    border: 1px solid var(--border);
    box-shadow: var(--shadow);
    border-radius: 12px;
    padding: 1.5rem;
  }

  .active-runs {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    gap: 1rem;
    align-items: center;
    flex-wrap: wrap;
  }

  .sub,
  .empty {
    margin: 0;
    color: var(--muted);
    font-size: 0.9rem;
  }

  .run-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .run-list li {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
    padding: 1rem;
    border: 1px solid var(--border);
    background: var(--panel-strong);
    border-radius: 10px;
  }

  .run-list li.selected {
    border-color: var(--primary);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--primary) 12%, transparent);
  }

  .run-copy {
    min-width: 0;
  }

  .run-actions {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    flex-wrap: wrap;
  }

  .run-title {
    display: flex;
    gap: 0.6rem;
    align-items: center;
    flex-wrap: wrap;
    margin-bottom: 0.35rem;
  }

  .badge {
    padding: 0.2rem 0.55rem;
    border-radius: 999px;
    background: var(--panel-hover);
    color: var(--muted);
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .badge-success {
    background: color-mix(in srgb, #1f8f5f 16%, var(--panel));
    color: #1f8f5f;
  }

  .badge-danger {
    background: color-mix(in srgb, var(--danger) 16%, var(--panel));
    color: var(--danger);
  }

  .badge-info {
    background: color-mix(in srgb, var(--primary) 16%, var(--panel));
    color: var(--primary);
  }

  .danger-soft {
    background: color-mix(in srgb, var(--danger) 14%, var(--panel));
    color: var(--danger);
    border: 1px solid color-mix(in srgb, var(--danger) 28%, transparent);
  }

  .danger-soft:hover {
    background: color-mix(in srgb, var(--danger) 22%, var(--panel));
  }

  @media (max-width: 720px) {
    .run-list li {
      flex-direction: column;
      align-items: stretch;
    }
  }
</style>
