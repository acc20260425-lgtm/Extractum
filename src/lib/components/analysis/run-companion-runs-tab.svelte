<script lang="ts">
  import { PanelRightOpen, RefreshCw, Square, Trash2 } from "@lucide/svelte";
  import {
    filterCompanionRuns,
    hasActiveCompanionRunsFilter,
    runsFilterDefaults,
    type CompanionRunsFilterState,
    type CompanionRunEntry,
  } from "$lib/analysis-run-companion-state";
  import { snapshotAffordanceForRun } from "$lib/analysis-run-snapshot-affordance";
  import type { WorkspaceSelection } from "$lib/analysis-workspace-state";
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import type { BadgeVariant } from "$lib/components/ui/types";
  import type { AnalysisRunSummary } from "$lib/types/analysis";

  let {
    activeRuns,
    savedRuns,
    loadingActiveRuns,
    loadingRuns,
    activeRunId,
    deletingRunIds,
    workspaceSelection,
    runsFilter,
    formatTimestamp,
    formatPeriod,
    phaseLabel,
    livePhase,
    liveProgress,
    runTargetLabel,
    statusTone,
    onChangeRunsFilter,
    onRefreshActiveRuns,
    onRefreshRuns,
    onOpenRun,
    onCancelRun,
    onDeleteRun,
  }: {
    activeRuns: AnalysisRunSummary[];
    savedRuns: AnalysisRunSummary[];
    loadingActiveRuns: boolean;
    loadingRuns: boolean;
    activeRunId: number | null;
    deletingRunIds: Record<number, boolean>;
    workspaceSelection: WorkspaceSelection;
    runsFilter: CompanionRunsFilterState;
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
    statusTone: (status: string) => BadgeVariant;
    onChangeRunsFilter: (filter: CompanionRunsFilterState) => void;
    onRefreshActiveRuns: () => void | Promise<void>;
    onRefreshRuns: () => void | Promise<void>;
    onOpenRun: (runId: number) => void | Promise<void>;
    onCancelRun: (runId: number) => void | Promise<void>;
    onDeleteRun: (run: AnalysisRunSummary) => void | Promise<void>;
  } = $props();

  const entries = $derived(filterCompanionRuns({
    activeRuns,
    savedRuns,
    filter: runsFilter,
    workspaceSelection,
  }));
  const hasActiveFilters = $derived(hasActiveCompanionRunsFilter(runsFilter));
  const hasAnyRuns = $derived(activeRuns.length > 0 || savedRuns.length > 0);
  const showRunsToolbar = $derived(hasAnyRuns || hasActiveFilters);

  function updateFilter(patch: Partial<CompanionRunsFilterState>) {
    onChangeRunsFilter({ ...runsFilter, ...patch });
  }

  function inputValue(event: Event) {
    const target = event.currentTarget;
    if (target instanceof HTMLInputElement) {
      return target.value;
    }
    return "";
  }

  function snapshotAffordanceForRow(entry: CompanionRunEntry) {
    if (entry.kind !== "saved") return null;

    return snapshotAffordanceForRun({
      snapshotState: entry.run.snapshot_state,
      snapshotCapturedAt: entry.run.snapshot_captured_at,
      snapshotError: entry.run.snapshot_error,
      probeState: "unknown",
      runStatus: entry.run.status,
      surface: "runs-row",
    });
  }
</script>

<section class="run-companion-runs-tab" data-smoke-id="run-companion-runs-panel">
  {#if showRunsToolbar}
    <div class="runs-toolbar">
      <label data-smoke-id="runs-search">
        <span>Search runs</span>
        <Input
          type="search"
          value={runsFilter.query}
          placeholder="Search target, template, provider, model, error"
          ariaLabel="Search runs"
          oninput={(event) => updateFilter({ query: inputValue(event) })}
        />
      </label>

      <div class="segmented" aria-label="Runs scope">
        <Button size="sm" variant="secondary" selected={runsFilter.scope === "current"} onclick={() => updateFilter({ scope: "current" })}>Current scope</Button>
        <Button size="sm" variant="secondary" selected={runsFilter.scope === "all"} onclick={() => updateFilter({ scope: "all" })}>All runs</Button>
      </div>

      <div class="segmented" aria-label="Runs status">
        <Button size="sm" variant="secondary" selected={runsFilter.status === "all"} onclick={() => updateFilter({ status: "all" })}>All</Button>
        <Button size="sm" variant="secondary" selected={runsFilter.status === "queued_running"} onclick={() => updateFilter({ status: "queued_running" })}>queued/running</Button>
        <Button size="sm" variant="secondary" selected={runsFilter.status === "completed"} onclick={() => updateFilter({ status: "completed" })}>Completed</Button>
        <Button size="sm" variant="secondary" selected={runsFilter.status === "failed"} onclick={() => updateFilter({ status: "failed" })}>Failed</Button>
        <Button size="sm" variant="secondary" selected={runsFilter.status === "cancelled"} onclick={() => updateFilter({ status: "cancelled" })}>Cancelled</Button>
      </div>

      <details class="advanced-filters" open={hasActiveFilters}>
        <summary>Advanced filters</summary>
        <div class="advanced-filter-grid">
          <div class="date-row" aria-label="Date range">
            <label>
              <span>From</span>
              <Input
                type="date"
                value={runsFilter.dateFrom}
                ariaLabel="Runs from date"
                oninput={(event) => updateFilter({ dateFrom: inputValue(event) })}
              />
            </label>
            <label>
              <span>To</span>
              <Input
                type="date"
                value={runsFilter.dateTo}
                ariaLabel="Runs to date"
                oninput={(event) => updateFilter({ dateTo: inputValue(event) })}
              />
            </label>
          </div>

          <div class="meta-row">
            <Input
              value={runsFilter.provider}
              placeholder="Provider"
              ariaLabel="Provider filter"
              oninput={(event) => updateFilter({ provider: inputValue(event) })}
            />
            <Input
              value={runsFilter.model}
              placeholder="Model"
              ariaLabel="Model filter"
              oninput={(event) => updateFilter({ model: inputValue(event) })}
            />
            <Input
              value={runsFilter.template}
              placeholder="Template"
              ariaLabel="Template filter"
              oninput={(event) => updateFilter({ template: inputValue(event) })}
            />
          </div>
        </div>
      </details>

      <div class="refresh-row">
        <Button size="sm" variant="secondary" onclick={onRefreshActiveRuns}>
          <RefreshCw size={14} aria-hidden="true" /> Active
        </Button>
        <Button size="sm" variant="secondary" onclick={onRefreshRuns}>
          <RefreshCw size={14} aria-hidden="true" /> Saved
        </Button>
      </div>
    </div>
  {/if}

  {#if loadingActiveRuns || loadingRuns}
    <EmptyState description="Loading analysis report runs..." />
  {:else if entries.length === 0}
    {#if hasActiveFilters}
      <div class="filtered-empty">
        <EmptyState description="No analysis report runs match these filters." />
        <Button size="sm" variant="secondary" onclick={() => onChangeRunsFilter(runsFilterDefaults())}>Clear filters</Button>
      </div>
    {:else}
      <div class="runs-empty-guidance">
        <EmptyState description="Run a report to create the first saved workspace." />
        <p>Completed reports will appear here with provider, model, snapshot, and error metadata.</p>
        <div class="refresh-row">
          <Button size="sm" variant="secondary" onclick={onRefreshActiveRuns}>
            <RefreshCw size={14} aria-hidden="true" /> Active
          </Button>
          <Button size="sm" variant="secondary" onclick={onRefreshRuns}>
            <RefreshCw size={14} aria-hidden="true" /> Saved
          </Button>
        </div>
      </div>
    {/if}
  {:else}
    <ul class="runs-list">
      {#each entries as entry (`${entry.kind}-${entry.run.id}`)}
        {@const run = entry.run}
        {@const snapshotAffordance = snapshotAffordanceForRow(entry)}
        <li class:selected={run.id === activeRunId}>
          <div class="run-copy">
            <div class="run-title">
              <strong>{runTargetLabel(run)}</strong>
              <Badge variant={statusTone(run.status)}>{run.status}</Badge>
              <Badge variant={entry.kind === "active" ? "info" : "neutral"}>{entry.kind}</Badge>
              {#if snapshotAffordance?.compactLabel && snapshotAffordance.badgeVariant}
                <Badge variant={snapshotAffordance.badgeVariant}>
                  {snapshotAffordance.compactLabel}
                </Badge>
              {/if}
            </div>
            <p>{formatTimestamp(run.created_at)} - {run.provider}/{run.model} - {run.prompt_template_name ?? "Unknown template"} v{run.prompt_template_version}</p>
            <p>Period: {formatPeriod(run.period_from, run.period_to)}</p>
            {#if entry.kind === "active"}
              <p>Phase: {phaseLabel(livePhase(run.id) || run.status)} {liveProgress(run.id)}</p>
            {/if}
            {#if run.error}
              <p class="run-error">{run.error}</p>
            {/if}
          </div>
          <div class="run-actions">
            <Button size="sm" variant="secondary" onclick={() => onOpenRun(run.id)}>
              <PanelRightOpen size={14} aria-hidden="true" /> Open
            </Button>
            {#if entry.kind === "active"}
              <Button size="sm" variant="danger-soft" onclick={() => onCancelRun(run.id)}>
                <Square size={14} aria-hidden="true" /> Cancel
              </Button>
            {:else}
              <Button size="sm" variant="danger-soft" disabled={deletingRunIds[run.id]} onclick={() => onDeleteRun(run)}>
                <Trash2 size={14} aria-hidden="true" />
                {deletingRunIds[run.id] ? "Deleting..." : "Delete"}
              </Button>
            {/if}
          </div>
        </li>
      {/each}
    </ul>
  {/if}
</section>

<style>
  .run-companion-runs-tab,
  .runs-toolbar,
  .runs-list {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    color: var(--muted);
    font-size: 0.82rem;
  }

  .segmented,
  .date-row,
  .meta-row,
  .refresh-row,
  .run-title,
  .run-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 0.45rem;
    align-items: center;
  }

  .advanced-filters {
    border: 1px solid color-mix(in srgb, var(--border) 78%, transparent);
    border-radius: 8px;
    background: color-mix(in srgb, var(--panel-strong) 62%, transparent);
  }

  .advanced-filters summary {
    cursor: pointer;
    padding: 0.55rem 0.65rem;
    color: var(--muted);
    font-size: 0.82rem;
    font-weight: 600;
  }

  .advanced-filter-grid {
    display: flex;
    flex-direction: column;
    gap: 0.55rem;
    padding: 0 0.65rem 0.65rem;
  }

  .runs-list {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .filtered-empty {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 0.65rem;
  }

  .runs-empty-guidance {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 0.6rem;
    color: var(--muted);
    font-size: 0.84rem;
  }

  .runs-empty-guidance p {
    margin: 0;
  }

  .runs-list li {
    display: flex;
    justify-content: space-between;
    gap: 0.75rem;
    padding: 0.85rem;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--panel-strong);
  }

  .runs-list li.selected {
    border-color: var(--primary);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--primary) 12%, transparent);
  }

  .run-copy {
    min-width: 0;
  }

  .run-copy p {
    margin: 0.25rem 0 0;
    color: var(--muted);
    font-size: 0.86rem;
    overflow-wrap: anywhere;
  }

  .run-error {
    color: var(--status-error-text);
  }

  @media (max-width: 720px) {
    .runs-list li {
      flex-direction: column;
      align-items: stretch;
    }
  }
</style>
