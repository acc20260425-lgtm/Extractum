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
  import {
    snapshotAffordanceForRun,
    type SnapshotProbeState,
  } from "$lib/analysis-run-snapshot-affordance";
  import type { BadgeVariant } from "$lib/components/ui/types";
  import type { SourceViewBasis } from "$lib/analysis-workspace-state";
  import type { AnalysisRunDetail } from "$lib/types/analysis";

  let {
    currentRun,
    sourceViewBasis,
    snapshotAvailability,
    snapshotProbeState,
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
    snapshotProbeState: SnapshotProbeState;
    traceRefCount: number;
    activePhase: string;
    activeProgress: string;
    canCancelCurrentRun: boolean;
    formatTimestamp: (value: number | null) => string;
    formatPeriod: (from: number, to: number) => string;
    runTargetLabel: (run: Pick<AnalysisRunDetail, "scope_type" | "source_id" | "source_title" | "source_group_id" | "source_group_name" | "project_id" | "project_name" | "scope_label">) => string;
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
  const snapshotAffordance = $derived(snapshotAffordanceForRun({
    snapshotState: currentRun.snapshot_state,
    snapshotCapturedAt: currentRun.snapshot_captured_at,
    snapshotError: currentRun.snapshot_error,
    probeState: snapshotProbeState,
    runStatus: currentRun.status,
    surface: "opened-header",
  }));
  const basisVariant = $derived(snapshotBadgeVariant(snapshotAvailability));
  const promptTemplateLabel = $derived(templateLabel(currentRun));

  function templateLabel(run: AnalysisRunDetail) {
    const version = run.prompt_template_version as number | null;
    return `${run.prompt_template_name ?? "Unknown"}${version === null ? "" : ` v${version}`}`;
  }

  function snapshotBadgeVariant(availability: RunSnapshotAvailability): BadgeVariant {
    if (availability === "available") return "success";
    if (availability === "capturing") return "info";
    if (availability === "unavailable") return "warning";
    return "neutral";
  }
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
      <Badge variant={basisVariant}>{basisLabel}</Badge>
      {#if canCancelCurrentRun}
        <Button variant="danger-soft" type="button" onclick={onCancelCurrentRun}>
          <Square size={15} aria-hidden="true" /> Cancel run
        </Button>
      {/if}
    </div>
  </div>

  {#if snapshotAffordance.headerWarning}
    <p class="snapshot-warning">
      {snapshotAffordance.headerWarning}
    </p>
  {/if}

  <div class="run-summary-strip">
    <MetaCell label="Period">{formatPeriod(currentRun.period_from, currentRun.period_to)}</MetaCell>
    <MetaCell label="Template">{promptTemplateLabel}</MetaCell>
    <MetaCell label="Provider/model">{currentRun.provider}/{currentRun.model}</MetaCell>
    <MetaCell label="Trace refs">{traceRefCount}</MetaCell>
  </div>

  <details class="run-details">
    <summary>Run details</summary>
    <div class="run-meta-grid">
      <MetaCell label="Scope">{runTargetLabel(currentRun)}</MetaCell>
      <MetaCell label="Status">{currentRun.status}</MetaCell>
      <MetaCell label="Created">{formatTimestamp(currentRun.created_at)}</MetaCell>
      <MetaCell label="Completed">{formatTimestamp(currentRun.completed_at)}</MetaCell>
      <MetaCell label="Provider profile">{currentRun.provider_profile}</MetaCell>
      <MetaCell label="Source basis">{basisDescription}</MetaCell>
      <MetaCell label="Snapshot status">{snapshotAffordance.detailTitle ?? basisLabel}</MetaCell>
      <MetaCell label="Snapshot captured">{currentRun.snapshot_captured_at ?? "Not recorded"}</MetaCell>
      {#if snapshotAffordance.detailDescription}
        <MetaCell label="Snapshot note">{snapshotAffordance.detailDescription}</MetaCell>
      {/if}
      {#if snapshotAffordance.sanitizedError}
        <MetaCell label="Snapshot error">{snapshotAffordance.sanitizedError}</MetaCell>
      {/if}
      <MetaCell label="YouTube corpus">{youtubeCorpusModeLabel(currentRun.youtube_corpus_mode)}</MetaCell>
      {#if currentRun.telegram_history_scope === "current_plus_migrated"}
        <MetaCell label="Telegram history">Current + migrated historical scope</MetaCell>
      {/if}
      <MetaCell label="Live phase">{activePhase || currentRun.status}</MetaCell>
      <MetaCell label="Live progress">{activeProgress || "n/a"}</MetaCell>
    </div>
  </details>

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

  .run-summary-strip {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 0.6rem;
  }

  .run-details {
    min-width: 0;
  }

  .run-details summary {
    width: fit-content;
    cursor: pointer;
    color: var(--muted);
    font-size: 0.82rem;
  }

  .run-details .run-meta-grid {
    margin-top: 0.7rem;
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

    .run-meta-grid,
    .run-summary-strip {
      grid-template-columns: 1fr;
    }
  }
</style>
