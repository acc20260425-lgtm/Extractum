<script lang="ts">
  import { Square } from "@lucide/svelte";
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import MetaCell from "$lib/components/ui/MetaCell.svelte";
  import {
    sourceBasisDescription,
    sourceBasisLabel,
    youtubeCorpusModeLabel,
    type RunSnapshotAvailability,
  } from "$lib/analysis-report-canvas-state";
  import type { BadgeVariant } from "$lib/components/ui/types";
  import type { SourceViewBasis } from "$lib/analysis-workspace-state";
  import type { AnalysisRunDetail } from "$lib/types/analysis";

  let {
    currentRun,
    sourceViewBasis,
    snapshotAvailability,
    traceRefCount,
    activePhase,
    activeProgress,
    canCancelCurrentRun,
    formatTimestamp,
    formatPeriod,
    runTargetLabel,
    statusTone,
    onCancelCurrentRun,
  }: {
    currentRun: AnalysisRunDetail;
    sourceViewBasis: SourceViewBasis;
    snapshotAvailability: RunSnapshotAvailability;
    traceRefCount: number;
    activePhase: string;
    activeProgress: string;
    canCancelCurrentRun: boolean;
    formatTimestamp: (value: number | null) => string;
    formatPeriod: (from: number, to: number) => string;
    runTargetLabel: (run: Pick<AnalysisRunDetail, "scope_type" | "source_id" | "source_title" | "source_group_id" | "source_group_name" | "scope_label">) => string;
    statusTone: (value: string) => BadgeVariant;
    onCancelCurrentRun: () => void | Promise<void>;
  } = $props();

  const basisLabel = $derived(sourceBasisLabel({
    currentRun,
    sourceViewBasis,
    snapshotAvailability,
  }));
  const basisDescription = $derived(sourceBasisDescription({
    currentRun,
    sourceViewBasis,
    snapshotAvailability,
  }));
  const hasSnapshotWarning = $derived(
    currentRun.status === "completed" && snapshotAvailability === "unavailable",
  );
</script>

<section class="report-run-header" aria-label="Opened run metadata">
  <div class="run-header-top">
    <div>
      <span class="eyebrow">Opened run</span>
      <h2>Run #{currentRun.id}</h2>
      <p>{runTargetLabel(currentRun)}</p>
    </div>
    <div class="run-header-actions">
      <Badge variant={statusTone(currentRun.status)}>{currentRun.status}</Badge>
      <Badge variant={snapshotAvailability === "unavailable" ? "warning" : "neutral"}>{basisLabel}</Badge>
      {#if canCancelCurrentRun}
        <Button variant="danger-soft" type="button" onclick={onCancelCurrentRun}>
          <Square size={15} aria-hidden="true" /> Cancel run
        </Button>
      {/if}
    </div>
  </div>

  {#if hasSnapshotWarning}
    <p class="snapshot-warning">
      Frozen source snapshot is missing. The saved report can still be read, but exact source browsing is degraded.
    </p>
  {/if}

  <div class="run-meta-grid">
    <MetaCell label="Scope">{runTargetLabel(currentRun)}</MetaCell>
    <MetaCell label="Status">{currentRun.status}</MetaCell>
    <MetaCell label="Created">{formatTimestamp(currentRun.created_at)}</MetaCell>
    <MetaCell label="Completed">{formatTimestamp(currentRun.completed_at)}</MetaCell>
    <MetaCell label="Period">{formatPeriod(currentRun.period_from, currentRun.period_to)}</MetaCell>
    <MetaCell label="Template">{currentRun.prompt_template_name ?? "Unknown"} v{currentRun.prompt_template_version}</MetaCell>
    <MetaCell label="Provider profile">{currentRun.provider_profile}</MetaCell>
    <MetaCell label="Provider/model">{currentRun.provider}/{currentRun.model}</MetaCell>
    <MetaCell label="Source basis">{basisDescription}</MetaCell>
    <MetaCell label="YouTube corpus">{youtubeCorpusModeLabel(currentRun.youtube_corpus_mode)}</MetaCell>
    <MetaCell label="Trace refs">{traceRefCount}</MetaCell>
    <MetaCell label="Live phase">{activePhase || currentRun.status}</MetaCell>
    <MetaCell label="Live progress">{activeProgress || "n/a"}</MetaCell>
  </div>

  {#if currentRun.error}
    <p class="run-error">{currentRun.error}</p>
  {/if}
</section>

<style>
  .report-run-header {
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
    padding: 1rem;
    border: 1px solid color-mix(in srgb, var(--primary) 16%, var(--border));
    border-radius: 8px;
    background: var(--panel);
    box-shadow: var(--shadow-soft);
  }

  .run-header-top {
    display: flex;
    justify-content: space-between;
    gap: 0.75rem;
    align-items: flex-start;
  }

  .eyebrow {
    display: inline-block;
    font-size: 0.68rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--muted);
    margin-bottom: 0.2rem;
  }

  h2,
  p {
    margin: 0;
  }

  .run-header-top p,
  .snapshot-warning {
    color: var(--muted);
    line-height: 1.45;
  }

  .run-header-actions {
    display: flex;
    justify-content: flex-end;
    align-items: center;
    flex-wrap: wrap;
    gap: 0.45rem;
  }

  .run-meta-grid {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 0.7rem;
  }

  .snapshot-warning,
  .run-error {
    padding: 0.7rem 0.85rem;
    border-radius: 8px;
  }

  .snapshot-warning {
    background: var(--status-warning-bg);
    color: var(--status-warning-text);
  }

  .run-error {
    background: var(--status-error-bg);
    color: var(--status-error-text);
  }

  @media (max-width: 960px) {
    .run-header-top {
      flex-direction: column;
    }

    .run-meta-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
