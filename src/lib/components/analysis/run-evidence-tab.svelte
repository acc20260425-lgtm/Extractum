<script lang="ts">
  import TracePanel from "$lib/components/analysis/trace-panel.svelte";
  import { evidenceSourceActionDecision } from "$lib/analysis-run-companion-state";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import StatusMessage from "$lib/components/ui/StatusMessage.svelte";
  import type { RunSnapshotAvailability } from "$lib/analysis-report-canvas-state";
  import type { AnalysisRunDetail, AnalysisTraceData, AnalysisTraceRef } from "$lib/types/analysis";

  let {
    currentRun,
    traceData,
    selectedTraceRef,
    selectedTrace,
    snapshotAvailability,
    formatTimestamp,
    traceRefOrigin,
    onSelectTraceRef,
    onShowSelectedTraceInSource,
  }: {
    currentRun: AnalysisRunDetail | null;
    traceData: AnalysisTraceData;
    selectedTraceRef: string | null;
    selectedTrace: AnalysisTraceRef | null;
    snapshotAvailability: RunSnapshotAvailability;
    formatTimestamp: (timestamp: number | null) => string;
    traceRefOrigin: (ref: string) => string;
    onSelectTraceRef: (ref: string) => void | Promise<void>;
    onShowSelectedTraceInSource: () => void | Promise<void>;
  } = $props();

  const sourceDecision = $derived(evidenceSourceActionDecision({
    currentRun,
    selectedTrace,
    snapshotAvailability,
  }));
</script>

<section class="run-evidence-tab">
  <!-- Show in source action is rendered by TracePanel for selected evidence. -->
  {#if !currentRun}
    <EmptyState
      title="No run open"
      description="Open a saved or active report to inspect trace evidence."
    />
  {:else}
    {#if sourceDecision.kind === "unavailable" && selectedTrace}
      <StatusMessage tone="default" className="evidence-warning">
        Snapshot unavailable: {sourceDecision.reason}
      </StatusMessage>
    {:else if sourceDecision.kind === "live_source"}
      <StatusMessage tone="default" className="evidence-warning">
        {sourceDecision.warning}
      </StatusMessage>
    {/if}

    <TracePanel
      traceRefs={traceData.refs}
      {selectedTraceRef}
      {selectedTrace}
      {formatTimestamp}
      {traceRefOrigin}
      showInSourceDisabledReason={sourceDecision.kind === "unavailable" ? sourceDecision.reason : ""}
      onSelectTraceRef={onSelectTraceRef}
      onShowInSource={onShowSelectedTraceInSource}
    />
  {/if}
</section>

<style>
  .run-evidence-tab {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }
</style>
