<script lang="ts">
  import { tick } from "svelte";
  import { Copy, ExternalLink, Search } from "@lucide/svelte";
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import StatusMessage from "$lib/components/ui/StatusMessage.svelte";
  import {
    formatYoutubeTime,
    groupYoutubeTranscriptItems,
    youtubeSegmentToReaderItem,
    youtubeTimestampUrl,
    type YoutubeTranscriptGroup,
    type SourceReaderItem,
  } from "$lib/source-reader-model";
  import type { EvidenceHighlightToken } from "$lib/analysis-evidence-source-navigation";
  import type { SourceItem, YoutubeTranscriptSegment } from "$lib/types/sources";
  import type { YoutubeVideoDetail } from "$lib/types/youtube";

  let {
    detail,
    segments,
    snapshotItems,
    loading,
    hasMore,
    transcriptSearch,
    showSyncActions = true,
    sourceTitle,
    selectedTraceRef,
    highlightToken = null,
    formatTimestamp,
    onChangeTranscriptSearch,
    onLoadMore,
    onSyncTranscript,
    onSyncMetadata,
    onSyncComments = null,
  }: {
    detail: YoutubeVideoDetail | null;
    segments: YoutubeTranscriptSegment[];
    snapshotItems: SourceReaderItem[];
    loading: boolean;
    hasMore: boolean;
    transcriptSearch: string;
    showSyncActions?: boolean;
    sourceTitle: string;
    selectedTraceRef: string | null;
    highlightToken?: EvidenceHighlightToken | null;
    formatTimestamp: (value: number | null) => string;
    onChangeTranscriptSearch: (value: string) => void;
    onLoadMore: () => void | Promise<void>;
    onSyncTranscript: () => void | Promise<void>;
    onSyncMetadata: () => void | Promise<void>;
    onSyncComments?: (() => void | Promise<void>) | null;
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
  const transcriptGroups = $derived(groupYoutubeTranscriptItems(readerItems));
  let transcriptElement: HTMLElement | null = $state(null);
  const consumedHighlightTokenIds = new Set<string>();

  $effect(() => {
    const selectedRef = transcriptGroups.find((group) => group.selected)?.refs[0] ?? null;
    if (selectedRef) {
      void scrollSelectedTranscriptGroupIntoView(selectedRef);
    }
  });

  $effect(() => {
    if (highlightToken && !consumedHighlightTokenIds.has(highlightToken.tokenId)) {
      const highlightedGroup = transcriptGroups.find((group) => isEvidenceHighlighted(group));
      if (!highlightedGroup) return;
      consumedHighlightTokenIds.add(highlightToken.tokenId);
      void scrollSelectedTranscriptGroupIntoView(highlightToken.traceRef);
    }
  });

  async function scrollSelectedTranscriptGroupIntoView(selectedRef: string) {
    await tick();
    const selected = transcriptElement?.querySelector<HTMLElement>(
      `[data-trace-refs~="${CSS.escape(selectedRef)}"]`,
    );
    selected?.scrollIntoView({ block: "center", behavior: "smooth" });
  }

  function isEvidenceHighlighted(group: YoutubeTranscriptGroup) {
    return highlightToken !== null && group.refs.includes(highlightToken.traceRef);
  }

  function timestampUrl(group: YoutubeTranscriptGroup) {
    if (canonicalUrl && group.startSeconds !== null) {
      return youtubeTimestampUrl(canonicalUrl, group.startSeconds);
    }
    return group.items[0]?.youtubeUrl ?? null;
  }

  function sourceBadge(sourceId: SourceItem["sourceId"]) {
    return `Source #${sourceId}`;
  }

  function refBadge(group: YoutubeTranscriptGroup) {
    if (group.refs.length === 0) return null;
    if (group.refs.length === 1) return group.refs[0];
    return `${group.refs.length} refs`;
  }

  async function copyLink(group: YoutubeTranscriptGroup) {
    const url = timestampUrl(group);
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

<section class="youtube-transcript-reader" aria-label="YouTube transcript reader" bind:this={transcriptElement}>
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
          <Badge variant={summary.comments.state === "synced" ? "success" : summary.comments.state === "failed" ? "danger" : "neutral"}>
            Comments {summary.comments.label}
          </Badge>
          <Badge variant="neutral">{summary.comments.itemCount} comments</Badge>
          <Badge variant="neutral">Comments synced {formatTimestamp(summary.comments.lastSyncedAt)}</Badge>
        {/if}
      </div>
    </div>
    {#if showSyncActions}
      <div class="transcript-actions">
        <Button type="button" size="sm" variant="secondary" onclick={onSyncMetadata}>Sync metadata</Button>
        <Button type="button" size="sm" variant="secondary" onclick={onSyncTranscript}>Sync transcript</Button>
        {#if onSyncComments}
          <Button type="button" size="sm" variant="secondary" onclick={onSyncComments}>Sync comments</Button>
        {/if}
      </div>
    {/if}
  </div>

  <label class="search-field">
    <span class="sr-only">Search transcript</span>
    <div class="search-input-wrap">
      <span class="search-icon">
        <Search size={15} aria-hidden="true" />
      </span>
      <Input
        type="search"
        value={transcriptSearch}
        placeholder="Search transcript"
        ariaLabel="Search transcript"
        oninput={(event) => onChangeTranscriptSearch(inputValue(event))}
      />
    </div>
  </label>

  {#if summary?.captions.state === "unavailable"}
    <StatusMessage tone="muted" surface={false}>
      {showSyncActions
        ? "Transcript unavailable for this video. Metadata and transcript sync actions remain available when the source supports retry."
        : "Transcript was unavailable in this run snapshot."}
    </StatusMessage>
  {/if}

  {#if !loading && transcriptGroups.length === 0}
    <EmptyState description="No transcript segments are loaded for this source view." />
  {:else}
    <ol class="transcript-group-list">
      {#each transcriptGroups as group (group.id)}
        {@const url = timestampUrl(group)}
        {@const visibleRef = refBadge(group)}
        <li
          class:selected={group.selected}
          data-trace-ref={group.refs[0] ?? undefined}
          data-trace-refs={group.refs.join(" ")}
          data-evidence-highlighted={isEvidenceHighlighted(group) ? "true" : undefined}
        >
          <div class="group-time">
            {#if group.startSeconds !== null && url}
              <a href={url} target="_blank" rel="noopener noreferrer">
                {formatYoutubeTime(group.startSeconds)}
                <ExternalLink size={13} aria-hidden="true" />
              </a>
            {:else if group.startSeconds !== null}
              <span>{formatYoutubeTime(group.startSeconds)}</span>
            {:else}
              <span>Transcript</span>
            {/if}
          </div>
          <p>{group.content}</p>
          <div class="group-actions">
            {#if group.captionLabel}<Badge variant="neutral">{group.captionLabel}</Badge>{/if}
            {#if visibleRef}<Badge variant="neutral">{visibleRef}</Badge>{/if}
            {#if group.sourceId !== null}<Badge variant="neutral">{sourceBadge(group.sourceId)}</Badge>{/if}
            {#if url}
              <Button type="button" size="sm" variant="ghost" ariaLabel="Copy timestamp link" title="Copy timestamp link" onclick={() => void copyLink(group)}>
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
  .group-actions {
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

  .search-input-wrap {
    position: relative;
    display: flex;
    align-items: center;
  }

  .search-icon {
    position: absolute;
    left: 0.75rem;
    z-index: 1;
    color: var(--muted);
    pointer-events: none;
  }

  .search-input-wrap :global(input) {
    padding-left: 2.15rem;
  }

  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border: 0;
  }

  .transcript-group-list {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .transcript-group-list li {
    display: grid;
    grid-template-columns: 5.5rem minmax(0, 1fr) auto;
    gap: 0.75rem;
    align-items: start;
    padding: 0.72rem 0.35rem 0.72rem 0.55rem;
    border-left: 2px solid transparent;
  }

  .transcript-group-list li + li {
    border-top: 1px solid color-mix(in srgb, var(--border) 70%, transparent);
  }

  .transcript-group-list li.selected,
  .transcript-group-list li[data-evidence-highlighted="true"] {
    border-left-color: var(--primary);
    background: color-mix(in srgb, var(--primary) 7%, transparent);
  }

  .group-time a,
  .group-time span {
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
    white-space: normal;
  }

  .reader-footer {
    display: flex;
    justify-content: center;
  }

  @media (max-width: 760px) {
    .transcript-header,
    .transcript-group-list li {
      display: flex;
      flex-direction: column;
    }

    .group-actions {
      gap: 0.35rem;
    }
  }
</style>
