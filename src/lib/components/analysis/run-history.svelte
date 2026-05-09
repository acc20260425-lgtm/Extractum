<script lang="ts">
  import { PanelRightOpen, RefreshCw, Trash2 } from "@lucide/svelte";
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Card from "$lib/components/ui/Card.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import PanelHeader from "$lib/components/ui/PanelHeader.svelte";
  import type { BadgeVariant } from "$lib/components/ui/types";
  import type { AnalysisRunSummary } from "$lib/types/analysis";

  type RunFilter = "all" | "completed" | "failed";
  type HistoryScope = "all" | "current";

  let {
    runs,
    loadingRuns,
    historyScope,
    historyTargetReady,
    runFilter,
    activeRunId,
    deletingRunIds,
    filteredRuns,
    formatTimestamp,
    formatPeriod,
    runTargetLabel,
    statusTone,
    onRefresh,
    onOpenRun,
    onDeleteRun,
    onChangeFilter,
    onChangeHistoryScope,
  }: {
    runs: AnalysisRunSummary[];
    loadingRuns: boolean;
    historyScope: HistoryScope;
    historyTargetReady: boolean;
    runFilter: RunFilter;
    activeRunId: number | null;
    deletingRunIds: Record<number, boolean>;
    filteredRuns: AnalysisRunSummary[];
    formatTimestamp: (timestamp: number | null) => string;
    formatPeriod: (periodFromUnix: number, periodToUnix: number) => string;
    runTargetLabel: (
      run: Pick<
        AnalysisRunSummary,
        "scope_type" | "source_id" | "source_title" | "source_group_id" | "source_group_name" | "scope_label"
      >
    ) => string;
    statusTone: (status: string) => BadgeVariant;
    onRefresh: () => void | Promise<void>;
    onOpenRun: (runId: number) => void | Promise<void>;
    onDeleteRun: (run: AnalysisRunSummary) => void | Promise<void>;
    onChangeFilter: (next: RunFilter) => void;
    onChangeHistoryScope: (next: HistoryScope) => void;
  } = $props();
</script>

<Card>
  <div class="history">
    <PanelHeader
      title="Saved Runs"
      subtitle="Saved reports with model, prompt version, and trace data."
    >
      <div class="history-actions">
        <div class="filter-group">
          <Button variant="secondary" selected={historyScope === "all"} onclick={() => onChangeHistoryScope("all")}>
            All runs
          </Button>
          <Button variant="secondary" selected={historyScope === "current"} onclick={() => onChangeHistoryScope("current")}>
            Current scope
          </Button>
        </div>
        <div class="filter-group">
          <Button variant="secondary" selected={runFilter === "all"} onclick={() => onChangeFilter("all")}>All</Button>
          <Button variant="secondary" selected={runFilter === "completed"} onclick={() => onChangeFilter("completed")}>
            Completed
          </Button>
          <Button variant="secondary" selected={runFilter === "failed"} onclick={() => onChangeFilter("failed")}>Failed</Button>
        </div>
        <Button variant="secondary" onclick={onRefresh}>
          <RefreshCw size={15} aria-hidden="true" /> Refresh
        </Button>
      </div>
    </PanelHeader>

    {#if loadingRuns}
      <EmptyState description="Loading saved runs..." />
    {:else if historyScope === "current" && !historyTargetReady}
      <EmptyState description="Select a source or source group to browse scope history." />
    {:else if runs.length === 0}
      <EmptyState description={historyScope === "all" ? "No saved runs yet." : "No saved runs for this scope yet."} />
    {:else if filteredRuns.length === 0}
      <EmptyState description="No runs match the current filter." />
    {:else}
      <ul class="run-list">
        {#each filteredRuns as run (run.id)}
          <li class:selected={run.id === activeRunId}>
            <div class="run-copy">
              <div class="run-title">
                <strong>{runTargetLabel(run)}</strong>
                <Badge variant={statusTone(run.status)}>{run.status}</Badge>
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
            <div class="run-actions">
              <Button variant="secondary" onclick={() => onOpenRun(run.id)} disabled={deletingRunIds[run.id]}>
                <PanelRightOpen size={15} aria-hidden="true" /> Open
              </Button>
              <Button variant="danger-soft" onclick={() => onDeleteRun(run)} disabled={deletingRunIds[run.id]}>
                <Trash2 size={15} aria-hidden="true" />
                {deletingRunIds[run.id] ? "Deleting..." : "Delete"}
              </Button>
            </div>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
</Card>

<style>
  .history {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .sub {
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

  .run-list-error {
    margin: 0;
    padding: 0.7rem 0.85rem;
    border-radius: 8px;
    background: var(--status-error-bg);
    color: var(--status-error-text);
    font-size: 0.88rem;
  }

  .history-actions,
  .filter-group,
  .run-actions {
    display: flex;
    gap: 0.6rem;
    flex-wrap: wrap;
    align-items: center;
  }

  .run-actions {
    justify-content: flex-end;
  }

  @media (max-width: 720px) {
    .run-list li {
      flex-direction: column;
      align-items: stretch;
    }

    .run-actions {
      justify-content: flex-start;
    }
  }
</style>
