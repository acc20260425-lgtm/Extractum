<script lang="ts">
  import ActiveRunList from "$lib/components/analysis/active-run-list.svelte";
  import ChunkSummaries from "$lib/components/analysis/chunk-summaries.svelte";
  import RunHistory from "$lib/components/analysis/run-history.svelte";
  import TracePanel from "$lib/components/analysis/trace-panel.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import type { BadgeVariant } from "$lib/components/ui/types";
  import type { AnalysisChunkSummaryEvent, AnalysisRunSummary, AnalysisTraceData, AnalysisTraceRef } from "$lib/types/analysis";

  let {
    inspectorMode,
    activeRuns,
    loadingActiveRuns,
    activeRunId,
    runs,
    loadingRuns,
    historyScope,
    historyTargetReady,
    runFilter,
    filteredRuns,
    traceData,
    selectedTraceRef,
    selectedTrace,
    focusedChunkSummaries,
    selectedRunIsActive,
    formatTimestamp,
    formatPeriod,
    phaseLabel,
    livePhase,
    liveProgress,
    runTargetLabel,
    statusTone,
    traceRefOrigin,
    onChangeInspectorMode,
    onRefreshActiveRuns,
    onOpenRun,
    onCancelRun,
    onRefreshRuns,
    onChangeFilter,
    onChangeHistoryScope,
    onSelectTraceRef,
  }: {
    inspectorMode: "active" | "history" | "trace" | "chunks";
    activeRuns: AnalysisRunSummary[];
    loadingActiveRuns: boolean;
    activeRunId: number | null;
    runs: AnalysisRunSummary[];
    loadingRuns: boolean;
    historyScope: "all" | "current";
    historyTargetReady: boolean;
    runFilter: "all" | "completed" | "failed";
    filteredRuns: AnalysisRunSummary[];
    traceData: AnalysisTraceData;
    selectedTraceRef: string | null;
    selectedTrace: AnalysisTraceRef | null;
    focusedChunkSummaries: AnalysisChunkSummaryEvent[];
    selectedRunIsActive: boolean;
    formatTimestamp: (value: number | null) => string;
    formatPeriod: (from: number, to: number) => string;
    phaseLabel: (value: string) => string;
    livePhase: (runId: number) => string;
    liveProgress: (runId: number) => string;
    runTargetLabel: (
      run: Pick<
        AnalysisRunSummary,
        "scope_type" | "source_id" | "source_title" | "source_group_id" | "source_group_name" | "scope_label"
      >
    ) => string;
    statusTone: (value: string) => BadgeVariant;
    traceRefOrigin: (ref: string) => string;
    onChangeInspectorMode: (mode: "active" | "history" | "trace" | "chunks") => void;
    onRefreshActiveRuns: () => void;
    onOpenRun: (runId: number) => void;
    onCancelRun: (runId: number) => void;
    onRefreshRuns: () => void;
    onChangeFilter: (mode: "all" | "completed" | "failed") => void;
    onChangeHistoryScope: (mode: "all" | "current") => void;
    onSelectTraceRef: (ref: string) => void;
  } = $props();

  function inspectorModeLabel(mode: "active" | "history" | "trace" | "chunks") {
    switch (mode) {
      case "active":
        return "Live runs";
      case "history":
        return "Saved runs";
      case "trace":
        return "Evidence view";
      default:
        return "Chunk stream";
    }
  }

  function inspectorModeSummary(mode: "active" | "history" | "trace" | "chunks") {
    switch (mode) {
      case "active":
        return "Monitor queued work and jump into the current live run.";
      case "history":
        return "Re-open saved runs without leaving the workspace.";
      case "trace":
        return "Inspect the exact refs behind report and chat claims.";
      default:
        return "Follow intermediate chunk summaries while a run is still streaming.";
    }
  }
</script>

<aside class="inspector">
  <div class="inspector-header">
    <div class="inspector-header-copy">
      <span class="eyebrow">Inspector</span>
      <h3>Runs and evidence</h3>
      <p>{inspectorModeSummary(inspectorMode)}</p>
    </div>
    <div class="inspector-header-meta">
      <span class="inspector-mode-pill">{inspectorModeLabel(inspectorMode)}</span>
      <div class="inspector-tabs" role="tablist" aria-label="Inspector sections">
      <Button
        variant="secondary"
        size="sm"
        className="inspector-tab"
        selected={inspectorMode === "active"}
        ariaPressed={inspectorMode === "active"}
        ariaControls="inspector-panel"
        onclick={() => onChangeInspectorMode("active")}
      >
        Active
      </Button>
      <Button
        variant="secondary"
        size="sm"
        className="inspector-tab"
        selected={inspectorMode === "history"}
        ariaPressed={inspectorMode === "history"}
        ariaControls="inspector-panel"
        onclick={() => onChangeInspectorMode("history")}
      >
        History
      </Button>
      <Button
        variant="secondary"
        size="sm"
        className="inspector-tab"
        selected={inspectorMode === "trace"}
        ariaPressed={inspectorMode === "trace"}
        ariaControls="inspector-panel"
        onclick={() => onChangeInspectorMode("trace")}
      >
        Trace
      </Button>
      <Button
        variant="secondary"
        size="sm"
        className="inspector-tab"
        selected={inspectorMode === "chunks"}
        ariaPressed={inspectorMode === "chunks"}
        ariaControls="inspector-panel"
        onclick={() => onChangeInspectorMode("chunks")}
      >
        Chunks
      </Button>
      </div>
    </div>
  </div>

  <div class="inspector-body" id="inspector-panel">
    {#if inspectorMode === "active"}
      <ActiveRunList
        {activeRuns}
        {loadingActiveRuns}
        {activeRunId}
        {formatTimestamp}
        {formatPeriod}
        {phaseLabel}
        {livePhase}
        {liveProgress}
        {runTargetLabel}
        {statusTone}
        onRefresh={onRefreshActiveRuns}
        onOpenRun={onOpenRun}
        onCancelRun={onCancelRun}
      />
    {:else if inspectorMode === "history"}
      <RunHistory
        {runs}
        {loadingRuns}
        {historyScope}
        {historyTargetReady}
        {runFilter}
        {activeRunId}
        {filteredRuns}
        {formatTimestamp}
        {formatPeriod}
        {runTargetLabel}
        {statusTone}
        onRefresh={onRefreshRuns}
        onOpenRun={onOpenRun}
        onChangeFilter={onChangeFilter}
        onChangeHistoryScope={onChangeHistoryScope}
      />
    {:else if inspectorMode === "trace"}
      <TracePanel
        traceRefs={traceData.refs}
        {selectedTraceRef}
        {selectedTrace}
        {formatTimestamp}
        {traceRefOrigin}
        onSelectTraceRef={onSelectTraceRef}
      />
    {:else}
      <ChunkSummaries summaries={focusedChunkSummaries} running={selectedRunIsActive} />
    {/if}
  </div>
</aside>

<style>
  .inspector {
    position: sticky;
    top: 0;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 0;
    max-height: calc(100vh - 4.75rem);
    overflow: hidden;
    background:
      linear-gradient(180deg, color-mix(in srgb, var(--panel) 97%, white 3%), var(--panel));
    border: 1px solid var(--border);
    box-shadow: var(--shadow);
    border-radius: 16px;
    padding: 1rem;
  }

  .inspector-header {
    display: flex;
    justify-content: space-between;
    gap: 0.75rem;
    align-items: flex-start;
    position: sticky;
    top: 0;
    z-index: 2;
    padding-bottom: 0.85rem;
    margin-bottom: 0.9rem;
    border-bottom: 1px solid color-mix(in srgb, var(--border) 76%, transparent);
    background:
      linear-gradient(180deg, color-mix(in srgb, var(--panel) 98%, white 2%), var(--panel));
  }

  .eyebrow {
    display: inline-block;
    font-size: 0.68rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--muted);
    margin-bottom: 0.2rem;
  }

  .inspector-header h3 {
    margin: 0;
  }

  .inspector-header-copy {
    min-width: 0;
  }

  .inspector-header-copy p {
    margin: 0.35rem 0 0 0;
    color: var(--muted);
    line-height: 1.45;
    max-width: 30ch;
  }

  .inspector-header-meta {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    gap: 0.55rem;
  }

  .inspector-mode-pill {
    display: inline-flex;
    align-items: center;
    padding: 0.28rem 0.65rem;
    border-radius: 999px;
    background: color-mix(in srgb, var(--primary) 10%, var(--panel));
    border: 1px solid color-mix(in srgb, var(--primary) 22%, transparent);
    color: var(--text);
    font-size: 0.76rem;
    font-weight: 700;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .inspector-tabs {
    display: flex;
    gap: 0.35rem;
    flex-wrap: wrap;
    justify-content: flex-end;
    padding: 0.2rem;
    border-radius: 12px;
    background: color-mix(in srgb, var(--panel-strong) 65%, transparent);
  }

  .inspector-body {
    min-width: 0;
    min-height: 18rem;
    overflow: auto;
    padding-right: 0.2rem;
  }

  .inspector :global(.ui-button.inspector-tab) {
    border-radius: 999px;
  }

  .inspector :global(.ui-button.inspector-tab.selected) {
    background: color-mix(in srgb, var(--primary) 14%, var(--panel));
    color: var(--text);
    border-color: color-mix(in srgb, var(--primary) 46%, transparent);
    box-shadow:
      0 0 0 3px color-mix(in srgb, var(--primary) 10%, transparent),
      inset 0 1px 0 color-mix(in srgb, white 8%, transparent);
  }

  @media (max-width: 1500px) {
    .inspector {
      position: static;
      max-height: none;
      overflow: visible;
      gap: 0.9rem;
    }

    .inspector-header {
      position: static;
      margin-bottom: 0;
    }

    .inspector-body {
      overflow: visible;
      padding-right: 0;
    }
  }

  @media (max-width: 720px) {
    .inspector-header {
      flex-direction: column;
      align-items: stretch;
    }

    .inspector-header-meta {
      align-items: stretch;
    }

    .inspector-tabs {
      justify-content: flex-start;
    }
  }
</style>
