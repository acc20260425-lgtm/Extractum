<script lang="ts">
  import ChunkSummaries from "$lib/components/analysis/chunk-summaries.svelte";
  import RunChatTab from "$lib/components/analysis/run-chat-tab.svelte";
  import RunCompanionRunsTab from "$lib/components/analysis/run-companion-runs-tab.svelte";
  import RunEvidenceTab from "$lib/components/analysis/run-evidence-tab.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import type { ChatAvailability, CompanionRunsFilterState } from "$lib/analysis-run-companion-state";
  import type { RunSnapshotAvailability } from "$lib/analysis-report-canvas-state";
  import type { SnapshotProbeState } from "$lib/analysis-run-snapshot-affordance";
  import type { CompanionTab, WorkspaceSelection } from "$lib/analysis-workspace-state";
  import type { BadgeVariant } from "$lib/components/ui/types";
  import type {
    AnalysisChatTurn,
    AnalysisChunkSummaryEvent,
    AnalysisRunDetail,
    AnalysisRunSummary,
    AnalysisTraceData,
    AnalysisTraceRef,
  } from "$lib/types/analysis";

  let {
    companionTab,
    currentRun,
    snapshotAvailability,
    snapshotProbeState,
    chatAvailability,
    traceData,
    selectedTraceRef,
    selectedTrace,
    focusedChunkSummaries = [],
    selectedRunIsActive = false,
    activeRuns,
    savedRuns,
    loadingActiveRuns,
    loadingRuns,
    activeRunId,
    deletingRunIds,
    workspaceSelection,
    runsFilter,
    loadingChat,
    chatMessages,
    chatQuestion,
    chatting,
    canCancelChat,
    clearingChat,
    formatTimestamp,
    formatPeriod,
    phaseLabel,
    livePhase,
    liveProgress,
    runTargetLabel,
    statusTone,
    traceRefOrigin,
    reportLines,
    onChangeCompanionTab,
    onSelectTraceRef,
    onShowSelectedTraceInSource,
    onFocusTraceRef,
    onAskQuestion,
    onCancelChat,
    onClearChat,
    onChangeChatQuestion,
    onChangeRunsFilter,
    onRefreshActiveRuns,
    onRefreshRuns,
    onOpenRun,
    onCancelRun,
    onDeleteRun,
  }: {
    companionTab: CompanionTab;
    currentRun: AnalysisRunDetail | null;
    snapshotAvailability: RunSnapshotAvailability;
    snapshotProbeState: SnapshotProbeState;
    chatAvailability: ChatAvailability;
    traceData: AnalysisTraceData;
    selectedTraceRef: string | null;
    selectedTrace: AnalysisTraceRef | null;
    focusedChunkSummaries?: AnalysisChunkSummaryEvent[];
    selectedRunIsActive?: boolean;
    activeRuns: AnalysisRunSummary[];
    savedRuns: AnalysisRunSummary[];
    loadingActiveRuns: boolean;
    loadingRuns: boolean;
    activeRunId: number | null;
    deletingRunIds: Record<number, boolean>;
    workspaceSelection: WorkspaceSelection;
    runsFilter: CompanionRunsFilterState;
    loadingChat: boolean;
    chatMessages: AnalysisChatTurn[];
    chatQuestion: string;
    chatting: boolean;
    canCancelChat: boolean;
    clearingChat: boolean;
    formatTimestamp: (timestamp: number | null) => string;
    formatPeriod: (periodFromUnix: number, periodToUnix: number) => string;
    phaseLabel: (phase: string) => string;
    livePhase: (runId: number) => string;
    liveProgress: (runId: number) => string;
    runTargetLabel: (
      run: Pick<
        AnalysisRunSummary,
        "scope_type" | "source_id" | "source_title" | "source_group_id" | "source_group_name" | "project_id" | "project_name" | "scope_label"
      >
    ) => string;
    statusTone: (status: string) => BadgeVariant;
    traceRefOrigin: (ref: string) => string;
    reportLines: (text: string) => Array<{
      key: string;
      segments: Array<{ type: "text" | "ref"; value: string; key: string }>;
    }>;
    onChangeCompanionTab: (tab: CompanionTab) => void;
    onSelectTraceRef: (ref: string) => void | Promise<void>;
    onShowSelectedTraceInSource: () => void | Promise<void>;
    onFocusTraceRef: (ref: string) => void | Promise<void>;
    onAskQuestion: () => void | Promise<void>;
    onCancelChat: () => void | Promise<void>;
    onClearChat: () => void | Promise<void>;
    onChangeChatQuestion: (value: string) => void;
    onChangeRunsFilter: (filter: CompanionRunsFilterState) => void;
    onRefreshActiveRuns: () => void | Promise<void>;
    onRefreshRuns: () => void | Promise<void>;
    onOpenRun: (runId: number) => void | Promise<void>;
    onCancelRun: (runId: number) => void | Promise<void>;
    onDeleteRun: (run: AnalysisRunSummary) => void | Promise<void>;
  } = $props();

  function tabId(tab: CompanionTab) {
    return `run-companion-tab-${tab}`;
  }

  function chunkTabLabel() {
    const count = focusedChunkSummaries.length;
    if (count === 0) return "Chunks";
    const total = focusedChunkSummaries.at(-1)?.total ?? null;
    return total && total > 0 ? `Chunks ${count}/${total}` : `Chunks ${count}`;
  }

  function chunksDisabled() {
    return currentRun === null;
  }
</script>

<aside class="run-companion-tabs">
  <div class="companion-header">
    <div>
      <span class="eyebrow">Companion</span>
      <h3>{currentRun ? `Run #${currentRun.id}` : "Runs"}</h3>
    </div>
    <div class="companion-tab-list" role="tablist" aria-label="Run companion tabs">
      <Button id={tabId("evidence")} role="tab" size="sm" variant="secondary" selected={companionTab === "evidence"} ariaSelected={companionTab === "evidence"} ariaControls="run-companion-panel" onclick={() => onChangeCompanionTab("evidence")}>Evidence</Button>
      <Button id={tabId("chat")} role="tab" size="sm" variant="secondary" selected={companionTab === "chat"} ariaSelected={companionTab === "chat"} ariaControls="run-companion-panel" onclick={() => onChangeCompanionTab("chat")}>Chat</Button>
      <Button
        id={tabId("chunks")}
        role="tab"
        size="sm"
        variant="secondary"
        selected={companionTab === "chunks"}
        ariaSelected={companionTab === "chunks"}
        ariaControls="run-companion-panel"
        disabled={chunksDisabled()}
        title={chunksDisabled() ? "Open a run to inspect chunk summaries." : undefined}
        onclick={() => {
          if (!chunksDisabled()) onChangeCompanionTab("chunks");
        }}
      >
        {chunkTabLabel()}
      </Button>
      <Button id={tabId("runs")} role="tab" size="sm" variant="secondary" selected={companionTab === "runs"} ariaSelected={companionTab === "runs"} ariaControls="run-companion-panel" smokeId="run-companion-runs-tab" onclick={() => onChangeCompanionTab("runs")}>Runs</Button>
    </div>
  </div>

  <div id="run-companion-panel" class="companion-panel" role="tabpanel" aria-labelledby={tabId(companionTab)}>
    {#if companionTab === "evidence"}
      <RunEvidenceTab
        {currentRun}
        {traceData}
        {selectedTraceRef}
        {selectedTrace}
        {snapshotAvailability}
        {snapshotProbeState}
        {formatTimestamp}
        {traceRefOrigin}
        onSelectTraceRef={onSelectTraceRef}
        onShowSelectedTraceInSource={onShowSelectedTraceInSource}
      />
    {:else if companionTab === "chat"}
      <RunChatTab
        {currentRun}
        {chatAvailability}
        {loadingChat}
        {chatMessages}
        {chatQuestion}
        {chatting}
        {canCancelChat}
        {clearingChat}
        {selectedTraceRef}
        {reportLines}
        onTraceRefSelect={onFocusTraceRef}
        onAskQuestion={onAskQuestion}
        onCancelChat={onCancelChat}
        onClearChat={onClearChat}
        onChangeChatQuestion={onChangeChatQuestion}
      />
    {:else if companionTab === "chunks"}
      {#if currentRun}
        <ChunkSummaries
          summaries={focusedChunkSummaries}
          running={selectedRunIsActive}
          framed={false}
        />
      {:else}
        <EmptyState description="Open a run to inspect chunk summaries." />
      {/if}
    {:else}
      <RunCompanionRunsTab
        {activeRuns}
        savedRuns={savedRuns}
        {loadingActiveRuns}
        {loadingRuns}
        {activeRunId}
        {deletingRunIds}
        {workspaceSelection}
        {runsFilter}
        {formatTimestamp}
        {formatPeriod}
        {phaseLabel}
        {livePhase}
        {liveProgress}
        {runTargetLabel}
        {statusTone}
        onChangeRunsFilter={onChangeRunsFilter}
        onRefreshActiveRuns={onRefreshActiveRuns}
        onRefreshRuns={onRefreshRuns}
        onOpenRun={onOpenRun}
        onCancelRun={onCancelRun}
        onDeleteRun={onDeleteRun}
      />
    {/if}
  </div>
</aside>

<style>
  .run-companion-tabs {
    position: sticky;
    top: 0;
    min-width: 0;
    max-height: calc(100vh - 4.75rem);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--panel);
    box-shadow: var(--shadow);
  }

  .companion-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 0.75rem;
    padding: 0.9rem;
    border-bottom: 1px solid color-mix(in srgb, var(--border) 76%, transparent);
  }

  .companion-header h3 {
    margin: 0;
  }

  .eyebrow {
    display: inline-block;
    margin-bottom: 0.2rem;
    color: var(--muted);
    font-size: 0.68rem;
    text-transform: uppercase;
  }

  .companion-tab-list {
    display: flex;
    flex-wrap: wrap;
    gap: 0.35rem;
    justify-content: flex-end;
  }

  .companion-panel {
    min-width: 0;
    min-height: 18rem;
    overflow: auto;
    padding: 0.9rem;
  }

  @media (max-width: 1500px) {
    .run-companion-tabs {
      position: static;
      max-height: none;
    }

    .companion-panel {
      overflow: visible;
    }
  }

  @media (max-width: 720px) {
    .companion-header {
      flex-direction: column;
    }

    .companion-tab-list {
      justify-content: flex-start;
    }
  }
</style>
