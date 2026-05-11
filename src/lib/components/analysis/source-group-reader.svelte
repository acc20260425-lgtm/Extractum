<script lang="ts">
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import Select from "$lib/components/ui/Select.svelte";
  import TelegramTimelineReader from "$lib/components/analysis/telegram-timeline-reader.svelte";
  import YoutubeTranscriptReader from "$lib/components/analysis/youtube-transcript-reader.svelte";
  import { groupReaderItemsBySource, type SourceReaderItem } from "$lib/source-reader-model";
  import type { YoutubeVideoDetail } from "$lib/types/youtube";

  const allGroupSourcesValue = "__all_group_sources__";

  let {
    items,
    selectedGroupSourceId,
    loading,
    hasMoreBySource,
    youtubeDetailsBySource,
    formatTimestamp,
    onLoadMoreSource,
    onChangeSelectedGroupSourceId = () => {},
  }: {
    items: SourceReaderItem[];
    selectedGroupSourceId: number | null;
    loading: boolean;
    hasMoreBySource: Record<number, boolean>;
    youtubeDetailsBySource: Record<number, YoutubeVideoDetail | null>;
    formatTimestamp: (value: number | null) => string;
    onLoadMoreSource: (sourceId: number) => void | Promise<void>;
    onChangeSelectedGroupSourceId?: (sourceId: number | null) => void;
  } = $props();

  const allSourceGroups = $derived(groupReaderItemsBySource(items));
  const sourceGroups = $derived(
    groupReaderItemsBySource(
      selectedGroupSourceId === null
        ? items
        : items.filter((item) => item.sourceId === selectedGroupSourceId),
    ),
  );

  function changeSelectedSource(event: Event) {
    const value = (event.currentTarget as HTMLSelectElement).value;
    onChangeSelectedGroupSourceId(value === allGroupSourcesValue ? null : Number(value));
  }
</script>

<section class="source-group-reader" aria-label="Source group reader">
  {#if allSourceGroups.length > 1}
    <label class="group-filter">
      <span>Source focus</span>
      <Select
        value={selectedGroupSourceId === null ? allGroupSourcesValue : String(selectedGroupSourceId)}
        onchange={changeSelectedSource}
      >
        <option value={allGroupSourcesValue}>All sources</option>
        {#each allSourceGroups as group (group.sourceId)}
          <option value={String(group.sourceId)}>{group.sourceTitle} ({group.items.length})</option>
        {/each}
      </Select>
    </label>
  {/if}

  {#if !loading && sourceGroups.length === 0}
    <EmptyState description="No source material is loaded for this group view." />
  {:else}
    {#each sourceGroups as group (group.sourceId)}
      {@const youtubeItems = group.items.filter((item) => item.kind === "youtube_transcript")}
      {@const telegramItems = group.items.filter((item) => item.kind !== "youtube_transcript")}
      <section class="source-bucket" aria-label={group.sourceTitle}>
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
            sourceTitle={group.sourceTitle}
            selectedTraceRef={null}
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
            {formatTimestamp}
            onLoadMore={() => onLoadMoreSource(group.sourceId)}
          />
        {/if}
      </section>
    {/each}
  {/if}
</section>

<style>
  .source-group-reader,
  .source-bucket {
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
    min-width: 0;
  }

  .group-filter {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    color: var(--muted);
    font-size: 0.75rem;
    max-width: 18rem;
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

  @media (max-width: 760px) {
    .group-filter {
      max-width: none;
    }

    .source-heading {
      align-items: flex-start;
      flex-direction: column;
    }
  }
</style>
