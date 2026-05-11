<script lang="ts">
  import { Copy, ExternalLink, Search } from "@lucide/svelte";
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import StatusMessage from "$lib/components/ui/StatusMessage.svelte";
  import {
    formatYoutubeTime,
    youtubeSegmentToReaderItem,
    youtubeTimestampUrl,
    type SourceReaderItem,
  } from "$lib/source-reader-model";
  import type { SourceItem, YoutubeTranscriptSegment } from "$lib/types/sources";
  import type { YoutubeVideoDetail } from "$lib/types/youtube";

  let {
    detail,
    segments,
    snapshotItems,
    loading,
    hasMore,
    transcriptSearch,
    sourceTitle,
    selectedTraceRef,
    formatTimestamp,
    onChangeTranscriptSearch,
    onLoadMore,
    onSyncTranscript,
    onSyncMetadata,
  }: {
    detail: YoutubeVideoDetail | null;
    segments: YoutubeTranscriptSegment[];
    snapshotItems: SourceReaderItem[];
    loading: boolean;
    hasMore: boolean;
    transcriptSearch: string;
    sourceTitle: string;
    selectedTraceRef: string | null;
    formatTimestamp: (value: number | null) => string;
    onChangeTranscriptSearch: (value: string) => void;
    onLoadMore: () => void | Promise<void>;
    onSyncTranscript: () => void | Promise<void>;
    onSyncMetadata: () => void | Promise<void>;
  } = $props();

  const summary = $derived(detail?.summary ?? null);
  const canonicalUrl = $derived(summary?.canonicalUrl ?? null);
  const liveItems = $derived(
    segments.map((segment) =>
      youtubeSegmentToReaderItem(segment, {
        sourceTitle,
        canonicalUrl,
        selectedTraceRef,
      }),
    ),
  );
  const readerItems = $derived(snapshotItems.length > 0 ? snapshotItems : liveItems);

  function timestampUrl(item: SourceReaderItem) {
    if (canonicalUrl && item.youtubeStartSeconds !== null) {
      return youtubeTimestampUrl(canonicalUrl, item.youtubeStartSeconds);
    }
    return item.youtubeUrl;
  }

  function sourceBadge(sourceId: SourceItem["sourceId"]) {
    return `Source #${sourceId}`;
  }

  async function copyLink(item: SourceReaderItem) {
    const url = timestampUrl(item);
    if (!url || typeof navigator === "undefined" || !navigator.clipboard) return;
    try {
      await navigator.clipboard.writeText(url);
    } catch (error) {
      console.error("Failed to copy timestamp link", error);
    }
  }

  function inputValue(event: Event) {
    const target = event.currentTarget;
    if (target instanceof HTMLInputElement) {
      return target.value;
    }
    return "";
  }
</script>

<section class="youtube-transcript-reader" aria-label="YouTube transcript reader">
  <div class="transcript-header">
    <div class="transcript-title">
      <span class="eyebrow">YouTube transcript</span>
      <h3>{summary?.title ?? sourceTitle}</h3>
      <div class="transcript-meta">
        {#if summary}
          <Badge variant={summary.captions.state === "synced" ? "success" : summary.captions.state === "unavailable" ? "warning" : "neutral"}>
            {summary.captions.label}
          </Badge>
          <Badge variant="neutral">{summary.captions.segmentCount} segments</Badge>
          <Badge variant="neutral">Last synced {formatTimestamp(summary.captions.lastSyncedAt)}</Badge>
        {/if}
      </div>
    </div>
    <div class="transcript-actions">
      <Button type="button" size="sm" variant="secondary" onclick={onSyncMetadata}>Sync metadata</Button>
      <Button type="button" size="sm" variant="secondary" onclick={onSyncTranscript}>Sync transcript</Button>
    </div>
  </div>

  <label class="search-field">
    <span>Search transcript</span>
    <div class="search-shell">
      <Search size={15} aria-hidden="true" />
      <Input
        type="search"
        value={transcriptSearch}
        ariaLabel="Search transcript"
        oninput={(event) => onChangeTranscriptSearch(inputValue(event))}
      />
    </div>
  </label>

  {#if summary?.captions.state === "unavailable"}
    <StatusMessage tone="muted" surface={false}>
      Transcript unavailable for this video. Metadata and transcript sync actions remain available when the source supports retry.
    </StatusMessage>
  {/if}

  {#if !loading && readerItems.length === 0}
    <EmptyState description="No transcript segments are loaded for this source view." />
  {:else}
    <ol class="segment-list">
      {#each readerItems as item (item.id)}
        {@const url = timestampUrl(item)}
        <li class:selected={item.selected}>
          <div class="segment-time">
            {#if item.youtubeStartSeconds !== null && url}
              <a href={url} target="_blank" rel="noopener noreferrer">
                {formatYoutubeTime(item.youtubeStartSeconds)}
                <ExternalLink size={13} aria-hidden="true" />
              </a>
            {:else if item.youtubeStartSeconds !== null}
              <span>{formatYoutubeTime(item.youtubeStartSeconds)}</span>
            {:else}
              <span>Transcript</span>
            {/if}
          </div>
          <p>{item.content}</p>
          <div class="segment-actions">
            {#if item.captionLabel}<Badge variant="neutral">{item.captionLabel}</Badge>{/if}
            {#if item.ref}<Badge variant="neutral">{item.ref}</Badge>{/if}
            <Badge variant="neutral">{sourceBadge(item.sourceId)}</Badge>
            {#if url}
              <Button type="button" size="sm" variant="ghost" ariaLabel="Copy timestamp link" title="Copy timestamp link" onclick={() => void copyLink(item)}>
                <Copy size={14} aria-hidden="true" />
              </Button>
            {/if}
          </div>
        </li>
      {/each}
    </ol>
  {/if}

  {#if hasMore}
    <div class="reader-footer">
      <Button type="button" variant="secondary" disabled={loading} onclick={onLoadMore}>
        {loading ? "Loading..." : "Load more transcript"}
      </Button>
    </div>
  {/if}
</section>

<style>
  .youtube-transcript-reader {
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
    min-width: 0;
  }

  .transcript-header {
    display: flex;
    justify-content: space-between;
    gap: 0.75rem;
    align-items: flex-start;
  }

  .transcript-title {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    min-width: 0;
  }

  .eyebrow {
    display: inline-block;
    font-size: 0.68rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--muted);
  }

  h3 {
    margin: 0;
  }

  .transcript-meta,
  .transcript-actions,
  .segment-actions,
  .search-shell {
    display: flex;
    flex-wrap: wrap;
    gap: 0.45rem;
    align-items: center;
  }

  .search-field {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    color: var(--muted);
    font-size: 0.8rem;
  }

  .segment-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .segment-list li {
    display: grid;
    grid-template-columns: 5.5rem minmax(0, 1fr) auto;
    gap: 0.75rem;
    align-items: start;
    padding: 0.8rem;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--panel);
  }

  .segment-list li.selected {
    border-color: var(--primary);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--primary) 12%, transparent);
  }

  .segment-time a,
  .segment-time span {
    display: inline-flex;
    gap: 0.25rem;
    align-items: center;
    color: var(--primary);
    text-decoration: none;
    font-variant-numeric: tabular-nums;
    font-weight: 700;
  }

  p {
    margin: 0;
    line-height: 1.5;
    white-space: pre-wrap;
  }

  .reader-footer {
    display: flex;
    justify-content: center;
  }

  @media (max-width: 760px) {
    .transcript-header,
    .segment-list li {
      display: flex;
      flex-direction: column;
    }
  }
</style>
