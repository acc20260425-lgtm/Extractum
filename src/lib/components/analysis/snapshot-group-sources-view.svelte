<script lang="ts">
  import Button from "$lib/components/ui/Button.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import TelegramTimelineReader from "$lib/components/analysis/telegram-timeline-reader.svelte";
  import YoutubeTranscriptReader from "$lib/components/analysis/youtube-transcript-reader.svelte";
  import { groupReaderItemsBySource, type SourceReaderItem } from "$lib/source-reader-model";

  let {
    items,
    selectedGroupSourceId,
    loading,
    hasMore,
    selectedTraceRef = null,
    formatTimestamp,
    onLoadMore,
  }: {
    items: SourceReaderItem[];
    selectedGroupSourceId: number | null;
    loading: boolean;
    hasMore: boolean;
    selectedTraceRef?: string | null;
    formatTimestamp: (value: number | null) => string;
    onLoadMore: () => void | Promise<void>;
  } = $props();

  const sourceGroups = $derived(
    groupReaderItemsBySource(
      selectedGroupSourceId === null
        ? items
        : items.filter((item) => item.sourceId === selectedGroupSourceId),
    ),
  );
</script>

<section class="snapshot-group-sources-view" aria-label="Run snapshot group sources">
  {#if !loading && sourceGroups.length === 0}
    <EmptyState description="No frozen source rows are loaded for this group snapshot." />
  {:else}
    {#each sourceGroups as group (group.sourceId)}
      {@const youtubeTranscriptItems = group.items.filter((item) => item.kind === "youtube_transcript")}
      {@const telegramItems = group.items.filter((item) => item.kind === "telegram_message")}
      {@const otherItems = group.items.filter((item) => item.kind !== "youtube_transcript" && item.kind !== "telegram_message")}
      <section class="source-bucket" aria-label={group.sourceTitle}>
        <div class="source-heading">
          <h3>{group.sourceTitle}</h3>
          <span>{group.items.length} frozen rows</span>
        </div>

        {#if youtubeTranscriptItems.length > 0}
          <YoutubeTranscriptReader
            detail={null}
            segments={[]}
            snapshotItems={youtubeTranscriptItems}
            {loading}
            hasMore={false}
            transcriptSearch=""
            showSyncActions={false}
            sourceTitle={group.sourceTitle}
            {selectedTraceRef}
            {formatTimestamp}
            onChangeTranscriptSearch={() => {}}
            onLoadMore={() => {}}
            onSyncTranscript={() => {}}
            onSyncMetadata={() => {}}
          />
        {/if}

        {#if telegramItems.length > 0}
          <TelegramTimelineReader
            items={telegramItems}
            {loading}
            hasMore={false}
            ariaLabel="Run snapshot source material timeline"
            {formatTimestamp}
            onLoadMore={() => {}}
          />
        {/if}

        {#if otherItems.length > 0}
          <ul class="other-item-list" aria-label={group.sourceTitle + " other snapshot rows"}>
            {#each otherItems as item (item.id)}
              <li class:selected={item.selected} data-trace-ref={item.ref}>
                <div>
                  <strong>{item.kind.replaceAll("_", " ")}</strong>
                  <span>{formatTimestamp(item.publishedAt)}</span>
                </div>
                <p>{item.content || "No text content captured for this snapshot row."}</p>
              </li>
            {/each}
          </ul>
        {/if}
      </section>
    {/each}

    {#if hasMore}
      <div class="source-group-footer">
        <Button type="button" variant="secondary" disabled={loading} onclick={onLoadMore}>
          {loading ? "Loading..." : "Load older snapshot messages"}
        </Button>
      </div>
    {/if}
  {/if}
</section>

<style>
  .snapshot-group-sources-view,
  .source-bucket {
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
    min-width: 0;
  }

  .source-bucket {
    padding-top: 0.8rem;
    border-top: 1px solid color-mix(in srgb, var(--border) 78%, transparent);
  }

  .source-heading {
    display: flex;
    justify-content: space-between;
    gap: 0.75rem;
    align-items: center;
  }

  .source-heading h3,
  .source-heading span {
    margin: 0;
  }

  .source-heading span {
    color: var(--muted);
    font-size: 0.82rem;
  }

  .other-item-list {
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .other-item-list li {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    padding: 0.65rem 0.75rem;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--panel);
  }

  .other-item-list li.selected {
    border-color: var(--accent);
    box-shadow: 0 0 0 2px color-mix(in srgb, var(--accent) 24%, transparent);
  }

  .other-item-list div {
    display: flex;
    justify-content: space-between;
    gap: 0.75rem;
    align-items: center;
  }

  .other-item-list span,
  .other-item-list p {
    color: var(--muted);
  }

  .other-item-list p {
    margin: 0;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }

  .source-group-footer {
    display: flex;
    justify-content: center;
  }

  @media (max-width: 760px) {
    .source-heading {
      align-items: flex-start;
      flex-direction: column;
    }
  }
</style>
