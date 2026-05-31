<script lang="ts">
  import { tick } from "svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import TelegramTimelineReader from "$lib/components/analysis/telegram-timeline-reader.svelte";
  import YoutubeTranscriptReader from "$lib/components/analysis/youtube-transcript-reader.svelte";
  import { groupReaderItemsBySource, type SourceReaderItem } from "$lib/source-reader-model";
  import type { EvidenceHighlightToken } from "$lib/analysis-evidence-source-navigation";
  import type { YoutubeVideoDetail } from "$lib/types/youtube";

  let {
    items,
    selectedGroupSourceId,
    loading,
    hasMoreBySource = {},
    hasMoreAll = false,
    loadMoreAllLabel = "Load more source material",
    youtubeDetailsBySource,
    selectedTraceRef = null,
    highlightToken = null,
    formatTimestamp,
    onLoadMoreSource,
    onLoadMoreAll = () => {},
  }: {
    items: SourceReaderItem[];
    selectedGroupSourceId: number | null;
    loading: boolean;
    hasMoreBySource?: Record<number, boolean>;
    hasMoreAll?: boolean;
    loadMoreAllLabel?: string;
    youtubeDetailsBySource: Record<number, YoutubeVideoDetail | null>;
    selectedTraceRef?: string | null;
    highlightToken?: EvidenceHighlightToken | null;
    formatTimestamp: (value: number | null) => string;
    onLoadMoreSource: (sourceId: number) => void | Promise<void>;
    onLoadMoreAll?: () => void | Promise<void>;
  } = $props();

  let sourcesElement: HTMLElement | null = $state(null);
  const consumedHighlightTokenIds = new Set<string>();
  const sourceGroups = $derived(
    groupReaderItemsBySource(
      selectedGroupSourceId === null
        ? items
        : items.filter((item) => item.sourceId === selectedGroupSourceId),
    ),
  );

  $effect(() => {
    if (highlightToken && !consumedHighlightTokenIds.has(highlightToken.tokenId)) {
      const highlightedRef = sourceGroups
        .flatMap((group) => group.items)
        .find((item) => isEvidenceHighlighted(item.ref))?.ref ?? null;
      if (!highlightedRef) return;
      consumedHighlightTokenIds.add(highlightToken.tokenId);
      void scrollHighlightedItemIntoView(highlightedRef);
    }
  });

  function isEvidenceHighlighted(ref: string | null) {
    return ref !== null && highlightToken !== null && highlightToken.traceRef === ref;
  }

  function groupHasEvidenceHighlight(group: { items: SourceReaderItem[] }) {
    return group.items.some((item) => isEvidenceHighlighted(item.ref));
  }

  async function scrollHighlightedItemIntoView(ref: string) {
    await tick();
    const highlighted = sourcesElement?.querySelector<HTMLElement>(
      `[data-trace-ref="${CSS.escape(ref)}"], [data-trace-refs~="${CSS.escape(ref)}"]`,
    );
    highlighted?.scrollIntoView({ block: "center", behavior: "smooth" });
  }
</script>

<section class="source-group-sources-view" aria-label="Source group sources" bind:this={sourcesElement}>
  {#if !loading && sourceGroups.length === 0}
    <EmptyState description="No source material is loaded for this group view." />
  {:else}
    {#each sourceGroups as group (group.sourceId)}
      {@const youtubeItems = group.items.filter((item) => item.kind === "youtube_transcript")}
      {@const telegramItems = group.items.filter((item) => item.kind !== "youtube_transcript")}
      <section
        class="source-bucket"
        aria-label={group.sourceTitle}
        data-evidence-highlighted={groupHasEvidenceHighlight(group) ? "true" : undefined}
      >
        <div class="source-heading">
          <h3>{group.sourceTitle}</h3>
          <span>{group.items.length} loaded items</span>
        </div>

        {#if youtubeItems.length > 0}
          <YoutubeTranscriptReader
            detail={youtubeDetailsBySource[group.sourceId] ?? null}
            segments={[]}
            snapshotItems={youtubeItems}
            {loading}
            hasMore={hasMoreBySource[group.sourceId] ?? false}
            transcriptSearch=""
            showSyncActions={false}
            sourceTitle={group.sourceTitle}
            {selectedTraceRef}
            {highlightToken}
            {formatTimestamp}
            onChangeTranscriptSearch={() => {}}
            onLoadMore={() => onLoadMoreSource(group.sourceId)}
            onSyncTranscript={() => {}}
            onSyncMetadata={() => {}}
          />
        {/if}

        {#if telegramItems.length > 0}
          <TelegramTimelineReader
            items={telegramItems}
            {loading}
            hasMore={hasMoreBySource[group.sourceId] ?? false}
            ariaLabel="Source material timeline"
            {highlightToken}
            {formatTimestamp}
            onLoadMore={() => onLoadMoreSource(group.sourceId)}
          />
        {/if}
      </section>
    {/each}

    {#if hasMoreAll}
      <div class="source-group-footer">
        <Button
          type="button"
          variant="secondary"
          disabled={loading}
          onclick={onLoadMoreAll}
        >
          {loading ? "Loading..." : loadMoreAllLabel}
        </Button>
      </div>
    {/if}
  {/if}
</section>

<style>
  .source-group-sources-view,
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
