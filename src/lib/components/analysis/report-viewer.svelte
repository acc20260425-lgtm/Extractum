<script lang="ts">
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Card from "$lib/components/ui/Card.svelte";
  import MetaCell from "$lib/components/ui/MetaCell.svelte";
  import PanelHeader from "$lib/components/ui/PanelHeader.svelte";
  import RefChip from "$lib/components/ui/RefChip.svelte";
  import type { AnalysisRunDetail } from "$lib/types/analysis";

  let {
    currentRun,
    loadingRunDetail,
    streamedOutput,
    traceRefCount,
    selectedTraceRef,
    livePhase,
    liveProgress,
    canCancelCurrentRun,
    formatTimestamp,
    formatPeriod,
    runTargetLabel,
    statusTone,
    reportLines,
    onFocusTraceRef,
    onCancelCurrentRun,
  }: {
    currentRun: AnalysisRunDetail | null;
    loadingRunDetail: boolean;
    streamedOutput: string;
    traceRefCount: number;
    selectedTraceRef: string | null;
    livePhase: string;
    liveProgress: string;
    canCancelCurrentRun: boolean;
    formatTimestamp: (timestamp: number | null) => string;
    formatPeriod: (periodFromUnix: number, periodToUnix: number) => string;
    runTargetLabel: (
      run: Pick<
        AnalysisRunDetail,
        "scope_type" | "source_id" | "source_title" | "source_group_id" | "source_group_name" | "scope_label"
      >
    ) => string;
    statusTone: (status: string) => string;
    reportLines: (text: string) => Array<{
      key: string;
      segments: Array<{ type: "text" | "ref"; value: string; key: string }>;
    }>;
    onFocusTraceRef: (ref: string) => void | Promise<void>;
    onCancelCurrentRun: () => void | Promise<void>;
  } = $props();
</script>

<Card>
  <div class="report-viewer">
    <PanelHeader
      title="Report Output"
      subtitle={currentRun ? `${runTargetLabel(currentRun)} - ${currentRun.provider}/${currentRun.model}` : ""}
    >
      {#if canCancelCurrentRun}
        <Button variant="danger-soft" type="button" onclick={onCancelCurrentRun}>Cancel run</Button>
      {/if}
    </PanelHeader>

    {#if currentRun}
      <div class="run-summary-panel">
        <div class="run-summary-header">
          <div class="run-summary-title">
            <strong>Run #{currentRun.id}</strong>
            <Badge variant={statusTone(currentRun.status)}>{currentRun.status}</Badge>
          </div>
          <span class="sub">
            {currentRun.prompt_template_name ?? "Unknown template"} - v{currentRun.prompt_template_version}
          </span>
        </div>

        <div class="run-meta-grid">
          <MetaCell label="Period">{formatPeriod(currentRun.period_from, currentRun.period_to)}</MetaCell>
          <MetaCell label="Scope">
            {currentRun.scope_type === "source_group" ? "Source group" : "Single source"}
          </MetaCell>
          <MetaCell label="Output language">{currentRun.output_language}</MetaCell>
          <MetaCell label="Created">{formatTimestamp(currentRun.created_at)}</MetaCell>
          <MetaCell label="Completed">{formatTimestamp(currentRun.completed_at)}</MetaCell>
          <MetaCell label="Provider profile">{currentRun.provider_profile}</MetaCell>
          <MetaCell label="Trace refs">{traceRefCount}</MetaCell>
          <MetaCell label="Live phase">{livePhase || currentRun.status}</MetaCell>
          <MetaCell label="Live progress">{liveProgress || "n/a"}</MetaCell>
        </div>

        {#if currentRun.error}
          <p class="run-error">{currentRun.error}</p>
        {/if}
      </div>
    {/if}

    <div class="report-body">
      {#if loadingRunDetail}
        <p class="empty">Loading saved run...</p>
      {:else if streamedOutput}
        <div class="report-output">
          {#each reportLines(streamedOutput) as line (line.key)}
            <div class="report-line">
              {#each line.segments as segment (segment.key)}
                {#if segment.type === "ref"}
                  <RefChip
                    refValue={segment.value}
                    active={segment.value === selectedTraceRef}
                    onclick={() => void onFocusTraceRef(segment.value)}
                  />
                {:else}
                  <span>{segment.value}</span>
                {/if}
              {/each}
            </div>
          {/each}
        </div>
      {:else}
        <p class="empty">No report output yet.</p>
      {/if}
    </div>
  </div>
</Card>

<style>
  .report-viewer {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .sub,
  .empty {
    margin: 0;
    color: var(--muted);
    font-size: 0.9rem;
  }

  .run-summary-panel {
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
    padding: 1rem;
    background: var(--panel-strong);
    border: 1px solid var(--border);
    border-radius: 10px;
  }

  .run-summary-header {
    display: flex;
    justify-content: space-between;
    gap: 1rem;
    align-items: flex-start;
    flex-wrap: wrap;
  }

  .run-summary-title {
    display: flex;
    gap: 0.6rem;
    align-items: center;
    flex-wrap: wrap;
  }

  .run-meta-grid {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 0.8rem;
  }

  .run-error {
    margin: 0;
    padding: 0.7rem 0.85rem;
    border-radius: 8px;
    background: var(--status-error-bg);
    color: var(--status-error-text);
    font-size: 0.88rem;
  }

  .report-body {
    min-width: 0;
  }

  .report-output {
    margin: 0;
    padding: 1rem;
    background: var(--panel-strong);
    border: 1px solid var(--border);
    border-radius: 10px;
    min-height: 22rem;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    font: inherit;
    line-height: 1.6;
  }

  .report-line {
    white-space: pre-wrap;
    word-break: break-word;
  }

  @media (max-width: 1080px) {
    .run-meta-grid {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }

  @media (max-width: 720px) {
    .run-meta-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
