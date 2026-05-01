<script lang="ts">
  import Badge from "$lib/components/ui/Badge.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import PanelHeader from "$lib/components/ui/PanelHeader.svelte";
  import type { BadgeVariant } from "$lib/components/ui/types";
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

  function originVariant(origin: string): BadgeVariant {
    if (origin === "saved") return "info";
    if (origin === "resolved") return "success";
    return "neutral";
  }
</script>

<aside class="trace-panel">
  <PanelHeader title="Traceability" level="h4">
    {#if traceRefs.length > 0}
      <span class="trace-count">{traceRefs.length} refs</span>
    {/if}
  </PanelHeader>

  {#if traceRefs.length === 0}
    <EmptyState
      title="Trace will appear here"
      description="Open a report with cited evidence, or follow a trace ref from the report or chat."
    />
  {:else}
    <div class="trace-list">
      {#each traceRefs as ref (ref.ref)}
        <button
          class="trace-link"
          class:selected={ref.ref === selectedTraceRef}
          type="button"
          onclick={() => onSelectTraceRef(ref.ref)}
        >
          <div class="trace-link-top">
            <strong>{ref.ref}</strong>
            <Badge variant={originVariant(traceRefOrigin(ref.ref))}>
              {originLabel(traceRefOrigin(ref.ref))}
            </Badge>
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

  .trace-count {
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

  .trace-link:hover,
  .trace-link.selected {
    background: var(--panel-hover);
    border-color: var(--primary);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--primary) 10%, transparent);
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
