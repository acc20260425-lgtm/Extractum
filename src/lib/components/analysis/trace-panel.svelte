<script lang="ts">
  import type { AnalysisTraceRef } from "$lib/types/analysis";

  let {
    traceRefs,
    selectedTraceRef,
    selectedTrace,
    formatTimestamp,
    traceRefOrigin,
    onSelectTraceRef,
  }: {
    traceRefs: AnalysisTraceRef[];
    selectedTraceRef: string | null;
    selectedTrace: AnalysisTraceRef | null;
    formatTimestamp: (timestamp: number | null) => string;
    traceRefOrigin: (ref: string) => string;
    onSelectTraceRef: (ref: string) => void;
  } = $props();

  function originLabel(origin: string) {
    if (origin === "saved") return "Saved in report";
    if (origin === "resolved") return "Resolved from chat";
    return "Trace ref";
  }
</script>

<aside class="trace-panel">
  <div class="trace-header">
    <h4>Traceability</h4>
    {#if traceRefs.length > 0}
      <span class="trace-count">{traceRefs.length} refs</span>
    {/if}
  </div>

  {#if traceRefs.length === 0}
    <p class="empty">No saved trace data yet.</p>
  {:else}
    <div class="trace-list">
      {#each traceRefs as ref}
        <button
          class="trace-link"
          class:selected={ref.ref === selectedTraceRef}
          type="button"
          onclick={() => onSelectTraceRef(ref.ref)}
        >
          <div class="trace-link-top">
            <strong>{ref.ref}</strong>
            <span class={`trace-origin trace-origin-${traceRefOrigin(ref.ref)}`}>
              {originLabel(traceRefOrigin(ref.ref))}
            </span>
          </div>
          <span>{formatTimestamp(ref.published_at)}</span>
        </button>
      {/each}
    </div>

    {#if selectedTrace}
      <div class="trace-detail">
        <div class="trace-meta">
          <strong>{selectedTrace.ref}</strong>
          <span>Source {selectedTrace.source_id} / message {selectedTrace.external_id}</span>
          <span>{formatTimestamp(selectedTrace.published_at)}</span>
        </div>
        <blockquote>{selectedTrace.excerpt}</blockquote>
      </div>
    {/if}
  {/if}
</aside>

<style>
  .trace-panel {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
    padding: 1rem;
    background: var(--panel-strong);
    border: 1px solid var(--border);
    border-radius: 10px;
    min-height: 22rem;
  }

  .trace-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
    flex-wrap: wrap;
  }

  .trace-header h4 {
    margin: 0;
  }

  .trace-count,
  .empty {
    margin: 0;
    color: var(--muted);
    font-size: 0.85rem;
  }

  .trace-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .trace-link {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 0.2rem;
    width: 100%;
    padding: 0.75rem 0.85rem;
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 10px;
    color: var(--text);
    text-align: left;
  }

  .trace-link-top {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 0.6rem;
    width: 100%;
    flex-wrap: wrap;
  }

  .trace-origin {
    display: inline-flex;
    align-items: center;
    padding: 0.15rem 0.45rem;
    border-radius: 999px;
    font-size: 0.72rem;
    letter-spacing: 0.02em;
    border: 1px solid transparent;
  }

  .trace-origin-saved {
    background: color-mix(in srgb, var(--primary) 12%, var(--panel));
    color: var(--primary);
    border-color: color-mix(in srgb, var(--primary) 22%, transparent);
  }

  .trace-origin-resolved {
    background: color-mix(in srgb, #1f8f5f 12%, var(--panel));
    color: #1f8f5f;
    border-color: color-mix(in srgb, #1f8f5f 22%, transparent);
  }

  .trace-link:hover,
  .trace-link.selected {
    background: var(--panel-hover);
    border-color: var(--primary);
  }

  .trace-link span,
  .trace-meta span {
    color: var(--muted);
    font-size: 0.82rem;
  }

  .trace-detail {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
    padding-top: 0.25rem;
    border-top: 1px solid var(--border);
  }

  .trace-meta {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
  }

  blockquote {
    margin: 0;
    padding: 0.9rem 1rem;
    border-left: 4px solid color-mix(in srgb, var(--primary) 45%, transparent);
    background: color-mix(in srgb, var(--panel) 70%, transparent);
    border-radius: 0 10px 10px 0;
    color: var(--text);
    white-space: pre-wrap;
    word-break: break-word;
  }
</style>
