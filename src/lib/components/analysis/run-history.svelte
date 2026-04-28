<script lang="ts">
  import type { AnalysisRunSummary } from "$lib/types/analysis";

  type RunFilter = "all" | "completed" | "failed" | "running";
  type HistoryScope = "all" | "current";

  let {
    runs,
    loadingRuns,
    historyScope,
    historyTargetReady,
    runFilter,
    activeRunId,
    filteredRuns,
    formatTimestamp,
    formatPeriod,
    runTargetLabel,
    statusTone,
    onRefresh,
    onOpenRun,
    onChangeFilter,
    onChangeHistoryScope,
  }: {
    runs: AnalysisRunSummary[];
    loadingRuns: boolean;
    historyScope: HistoryScope;
    historyTargetReady: boolean;
    runFilter: RunFilter;
    activeRunId: number | null;
    filteredRuns: AnalysisRunSummary[];
    formatTimestamp: (timestamp: number | null) => string;
    formatPeriod: (periodFromUnix: number, periodToUnix: number) => string;
    runTargetLabel: (
      run: Pick<
        AnalysisRunSummary,
        "scope_type" | "source_id" | "source_title" | "source_group_id" | "source_group_name" | "scope_label"
      >
    ) => string;
    statusTone: (status: string) => string;
    onRefresh: () => void | Promise<void>;
    onOpenRun: (runId: number) => void | Promise<void>;
    onChangeFilter: (next: RunFilter) => void;
    onChangeHistoryScope: (next: HistoryScope) => void;
  } = $props();
</script>

<section class="card history">
  <div class="panel-header">
    <div>
      <h3>Saved Runs</h3>
      <p class="sub">Immutable report runs with saved model, prompt version, and traceability data.</p>
    </div>
    <div class="history-actions">
      <div class="filter-group">
        <button class:activeFilter={historyScope === "all"} class="secondary" onclick={() => onChangeHistoryScope("all")}>
          All runs
        </button>
        <button
          class:activeFilter={historyScope === "current"}
          class="secondary"
          onclick={() => onChangeHistoryScope("current")}
        >
          Current scope
        </button>
      </div>
      <div class="filter-group">
        <button class:activeFilter={runFilter === "all"} class="secondary" onclick={() => onChangeFilter("all")}>All</button>
        <button class:activeFilter={runFilter === "completed"} class="secondary" onclick={() => onChangeFilter("completed")}>Completed</button>
        <button class:activeFilter={runFilter === "running"} class="secondary" onclick={() => onChangeFilter("running")}>Running</button>
        <button class:activeFilter={runFilter === "failed"} class="secondary" onclick={() => onChangeFilter("failed")}>Failed</button>
      </div>
      <button class="secondary" onclick={onRefresh}>Refresh</button>
    </div>
  </div>

  {#if loadingRuns}
    <p class="empty">Loading analysis runs...</p>
  {:else if historyScope === "current" && !historyTargetReady}
    <p class="empty">Select a source or source group to browse current-scope history.</p>
  {:else if runs.length === 0}
    <p class="empty">{historyScope === "all" ? "No analysis runs yet." : "No saved runs for the current scope yet."}</p>
  {:else if filteredRuns.length === 0}
    <p class="empty">No runs match the current filter.</p>
  {:else}
    <ul class="run-list">
      {#each filteredRuns as run (run.id)}
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
            {#if run.completed_at}
              <p class="sub">Completed: {formatTimestamp(run.completed_at)}</p>
            {/if}
            {#if run.error}
              <p class="run-list-error">{run.error}</p>
            {/if}
          </div>
          <button class="secondary" onclick={() => onOpenRun(run.id)}>Open</button>
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

  .history {
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

  .run-list-error {
    margin: 0;
    padding: 0.7rem 0.85rem;
    border-radius: 8px;
    background: var(--status-error-bg);
    color: var(--status-error-text);
    font-size: 0.88rem;
  }

  .history-actions,
  .filter-group {
    display: flex;
    gap: 0.6rem;
    flex-wrap: wrap;
    align-items: center;
  }

  .activeFilter {
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--primary) 14%, transparent);
    border-color: var(--primary);
  }

  @media (max-width: 720px) {
    .run-list li {
      flex-direction: column;
      align-items: stretch;
    }
  }
</style>
