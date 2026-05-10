<script lang="ts">
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import PanelHeader from "$lib/components/ui/PanelHeader.svelte";
  import type { AnalysisRunMessage } from "$lib/types/analysis";

  let {
    messages,
    loadingRunSnapshotMessages,
    hasMoreRunSnapshotMessages,
    formatTimestamp,
    onLoadMoreRunSnapshotMessages,
  }: {
    messages: AnalysisRunMessage[];
    loadingRunSnapshotMessages: boolean;
    hasMoreRunSnapshotMessages: boolean;
    formatTimestamp: (value: number | null) => string;
    onLoadMoreRunSnapshotMessages: () => void | Promise<void>;
  } = $props();

  function sourceLabel(message: AnalysisRunMessage) {
    const type = message.source_type ?? "source";
    const subtype = message.source_subtype ? `/${message.source_subtype}` : "";
    return `${type}${subtype} #${message.source_id}`;
  }
</script>

<section class="run-snapshot-messages">
  <PanelHeader title="Run snapshot">
    {#if loadingRunSnapshotMessages}
      <span class="subtle">Loading snapshot...</span>
    {:else}
      <span class="subtle">{messages.length} snapshot messages loaded</span>
    {/if}
  </PanelHeader>

  {#if !loadingRunSnapshotMessages && messages.length === 0}
    <EmptyState description="No frozen source messages were returned for this run snapshot." />
  {:else}
    <ul class="snapshot-list">
      {#each messages as message (message.ref)}
        <li>
          <div class="snapshot-meta">
            <Badge variant="neutral">{message.ref}</Badge>
            <span>{sourceLabel(message)}</span>
            <span>{formatTimestamp(message.published_at)}</span>
            {#if message.author}<span>{message.author}</span>{/if}
          </div>
          <p>{message.content || "No text content captured for this snapshot row."}</p>
        </li>
      {/each}
    </ul>
  {/if}

  {#if hasMoreRunSnapshotMessages}
    <div class="snapshot-footer">
      <Button
        type="button"
        variant="secondary"
        disabled={loadingRunSnapshotMessages}
        onclick={onLoadMoreRunSnapshotMessages}
      >
        {loadingRunSnapshotMessages ? "Loading..." : "Load older snapshot messages"}
      </Button>
    </div>
  {/if}
</section>

<style>
  .run-snapshot-messages {
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
    min-width: 0;
  }

  .snapshot-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.55rem;
  }

  .snapshot-list li {
    padding: 0.9rem 1rem;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--panel-strong);
  }

  .snapshot-list p {
    margin: 0;
    white-space: pre-wrap;
    line-height: 1.5;
  }

  .snapshot-meta {
    display: flex;
    gap: 0.55rem;
    flex-wrap: wrap;
    color: var(--muted);
    font-size: 0.78rem;
    margin-bottom: 0.45rem;
  }

  .snapshot-footer {
    display: flex;
    justify-content: center;
  }

  .subtle {
    color: var(--muted);
    font-size: 0.78rem;
  }
</style>
