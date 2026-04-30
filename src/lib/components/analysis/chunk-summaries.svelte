<script lang="ts">
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import PanelHeader from "$lib/components/ui/PanelHeader.svelte";
  import type { AnalysisChunkSummaryEvent } from "$lib/types/analysis";

  let {
    summaries,
    running,
  }: {
    summaries: AnalysisChunkSummaryEvent[];
    running: boolean;
  } = $props();
</script>

{#if running || summaries.length > 0}
  <section class="card chunk-summaries">
    <PanelHeader
      title="Chunk Summaries"
      subtitle={summaries.length > 0
        ? `${summaries.length}/${summaries[0]?.total ?? summaries.length} received`
        : "Waiting for the first chunk summary..."}
    />

    {#if summaries.length === 0}
      <EmptyState description="Intermediate LLM summaries will appear here during chunk analysis." />
    {:else}
      <div class="chunk-list">
        {#each summaries as chunk (chunk.index)}
          <details class="chunk-item" open={chunk.index === summaries.length}>
            <summary>
              <strong>Chunk {chunk.index}/{chunk.total}</strong>
              <span>{chunk.message_count} messages</span>
            </summary>

            <p>{chunk.summary}</p>

            {#if chunk.topics.length > 0}
              <div class="chunk-section">
                <span class="section-label">Topics</span>
                <div class="chip-list">
                  {#each chunk.topics as topic (topic)}
                    <span class="chip">{topic}</span>
                  {/each}
                </div>
              </div>
            {/if}

            {#if chunk.notable_points.length > 0}
              <div class="chunk-section">
                <span class="section-label">Notable points</span>
                <ul>
                  {#each chunk.notable_points as point (point)}
                    <li>{point}</li>
                  {/each}
                </ul>
              </div>
            {/if}

            {#if chunk.candidate_refs.length > 0}
              <div class="chunk-section">
                <span class="section-label">Candidate refs</span>
                <div class="chip-list refs">
                  {#each chunk.candidate_refs as ref (ref)}
                    <span class="chip">{ref}</span>
                  {/each}
                </div>
              </div>
            {/if}
          </details>
        {/each}
      </div>
    {/if}
  </section>
{/if}

<style>
  .card {
    background: var(--panel);
    border: 1px solid var(--border);
    box-shadow: var(--shadow);
    border-radius: 12px;
    padding: 1.5rem;
  }

  .chunk-summaries {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .chunk-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    max-height: 32rem;
    overflow: auto;
    padding-right: 0.25rem;
  }

  .chunk-item {
    padding: 0.85rem 0.95rem;
    background: var(--panel-strong);
    border: 1px solid var(--border);
    border-radius: 10px;
  }

  .chunk-item summary {
    display: flex;
    justify-content: space-between;
    gap: 1rem;
    cursor: pointer;
    color: var(--text);
  }

  .chunk-item summary span {
    color: var(--muted);
    font-size: 0.85rem;
    white-space: nowrap;
  }

  .chunk-item p {
    margin: 0.75rem 0 0;
    line-height: 1.5;
    color: var(--text);
  }

  .chunk-section {
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
    margin-top: 0.8rem;
  }

  .section-label {
    color: var(--muted);
    font-size: 0.76rem;
    font-weight: 700;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .chip-list {
    display: flex;
    flex-wrap: wrap;
    gap: 0.35rem;
  }

  .chip {
    display: inline-flex;
    align-items: center;
    padding: 0.18rem 0.5rem;
    border-radius: 999px;
    background: color-mix(in srgb, var(--primary) 12%, var(--panel));
    color: var(--primary);
    border: 1px solid color-mix(in srgb, var(--primary) 22%, transparent);
    font-size: 0.78rem;
    font-weight: 600;
  }

  .refs .chip {
    font-family: ui-monospace, SFMono-Regular, Consolas, "Liberation Mono", monospace;
  }

  ul {
    margin: 0;
    padding-left: 1.15rem;
    color: var(--text);
    line-height: 1.45;
  }

  li + li {
    margin-top: 0.35rem;
  }
</style>
