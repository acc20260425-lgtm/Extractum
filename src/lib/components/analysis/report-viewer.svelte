<script lang="ts">
  import type { AnalysisRunDetail } from "$lib/types/analysis";

  let {
    currentRun,
    loadingRunDetail,
    streamedOutput,
    traceRefCount,
    selectedTraceRef,
    formatTimestamp,
    formatPeriod,
    runTargetLabel,
    statusTone,
    reportLines,
    onFocusTraceRef,
  }: {
    currentRun: AnalysisRunDetail | null;
    loadingRunDetail: boolean;
    streamedOutput: string;
    traceRefCount: number;
    selectedTraceRef: string | null;
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
  } = $props();
</script>

<div class="report-viewer">
  <div class="panel-header">
    <div>
      <h3>Report Output</h3>
      {#if currentRun}
        <p class="sub">
          {runTargetLabel(currentRun)} - {currentRun.provider}/{currentRun.model}
        </p>
      {/if}
    </div>
  </div>

  {#if currentRun}
    <div class="run-summary-panel">
      <div class="run-summary-header">
        <div class="run-summary-title">
          <strong>Run #{currentRun.id}</strong>
          <span class={`badge badge-${statusTone(currentRun.status)}`}>{currentRun.status}</span>
        </div>
        <span class="sub">
          {currentRun.prompt_template_name ?? "Unknown template"} - v{currentRun.prompt_template_version}
        </span>
      </div>

      <div class="run-meta-grid">
        <div>
          <span class="meta-label">Period</span>
          <strong>{formatPeriod(currentRun.period_from, currentRun.period_to)}</strong>
        </div>
        <div>
          <span class="meta-label">Scope</span>
          <strong>{currentRun.scope_type === "source_group" ? "Source group" : "Single source"}</strong>
        </div>
        <div>
          <span class="meta-label">Output language</span>
          <strong>{currentRun.output_language}</strong>
        </div>
        <div>
          <span class="meta-label">Created</span>
          <strong>{formatTimestamp(currentRun.created_at)}</strong>
        </div>
        <div>
          <span class="meta-label">Completed</span>
          <strong>{formatTimestamp(currentRun.completed_at)}</strong>
        </div>
        <div>
          <span class="meta-label">Provider profile</span>
          <strong>{currentRun.provider_profile}</strong>
        </div>
        <div>
          <span class="meta-label">Trace refs</span>
          <strong>{traceRefCount}</strong>
        </div>
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
                <button
                  class="ref-chip"
                  class:active={segment.value === selectedTraceRef}
                  type="button"
                  onclick={() => void onFocusTraceRef(segment.value)}
                >
                  [{segment.value}]
                </button>
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

<style>
  .report-viewer {
    min-width: 0;
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

  .run-meta-grid > div {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    padding: 0.75rem 0.85rem;
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 10px;
  }

  .meta-label {
    color: var(--muted);
    font-size: 0.78rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
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

  .ref-chip {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.08rem 0.45rem;
    margin: 0 0.08rem;
    border-radius: 999px;
    background: color-mix(in srgb, var(--primary) 14%, var(--panel));
    color: var(--primary);
    border: 1px solid color-mix(in srgb, var(--primary) 24%, transparent);
    font-size: 0.82rem;
    font-weight: 600;
  }

  .ref-chip:hover,
  .ref-chip.active {
    background: color-mix(in srgb, var(--primary) 22%, var(--panel));
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
