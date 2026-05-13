<script lang="ts">
  import { ExternalLink, RefreshCw, RotateCcw, Video } from "@lucide/svelte";
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import StatusMessage from "$lib/components/ui/StatusMessage.svelte";
  import YoutubeSourceActivity from "$lib/components/analysis/youtube-source-activity.svelte";
  import type { SourceJobRecord } from "$lib/types/sources";
  import type { YoutubePlaylistDetail, YoutubePlaylistItemDetail } from "$lib/types/youtube";

  const retryableStatuses = new Set([
    "live_ended_transcript_pending",
    "no_captions",
    "unavailable_unknown",
  ]);

  let {
    sourceTitle,
    playlist,
    loading,
    sourceJobs = [],
    formatTimestamp,
    onOpenSource,
    onSyncPlaylist,
    onRetryFailed,
    onSyncPlaylistVideo,
    onRetryPlaylistVideo,
    onCancelSourceJob = async () => {},
  }: {
    sourceTitle: string;
    playlist: YoutubePlaylistDetail | null;
    loading: boolean;
    sourceJobs?: SourceJobRecord[];
    formatTimestamp: (value: number | null) => string;
    onOpenSource: (sourceId: number) => void | Promise<void>;
    onSyncPlaylist: () => void | Promise<void>;
    onRetryFailed: () => void | Promise<void>;
    onSyncPlaylistVideo: (videoSourceId: number) => void | Promise<void>;
    onRetryPlaylistVideo: (videoSourceId: number) => void | Promise<void>;
    onCancelSourceJob?: (jobId: string) => void | Promise<void>;
  } = $props();

  const summary = $derived(playlist?.summary ?? null);
  const items = $derived(playlist?.items ?? []);

  function availabilityLabel(value: string | null | undefined) {
    return value ? value.replaceAll("_", " ") : "unknown";
  }

  function formatDuration(value: number | null) {
    if (value === null) return "";
    const minutes = Math.floor(value / 60);
    const seconds = value % 60;
    return `${minutes}:${String(seconds).padStart(2, "0")}`;
  }

  function canSyncItem(item: YoutubePlaylistItemDetail) {
    return item.videoSourceId !== null && !item.isRemovedFromPlaylist;
  }

  function canRetryItem(item: YoutubePlaylistItemDetail) {
    return canSyncItem(item) && retryableStatuses.has(item.availabilityStatus);
  }
</script>

<section class="youtube-playlist-reader" aria-label="YouTube playlist reader">
  <div class="playlist-header">
    <div class="playlist-title">
      <span class="eyebrow">YouTube playlist</span>
      <h3>{summary?.title ?? sourceTitle}</h3>
      <div class="playlist-meta">
        <Badge variant="info">{summary?.channelHandle ?? summary?.channelTitle ?? "YouTube"}</Badge>
        <Badge variant="neutral">{summary?.videoCount ?? playlist?.items.length ?? 0} videos</Badge>
        <Badge variant="neutral">{summary?.linkedVideoCount ?? 0} linked</Badge>
        {#if (summary?.unavailableCount ?? 0) > 0}
          <Badge variant="warning">{summary?.unavailableCount} unavailable</Badge>
        {/if}
      </div>
    </div>
    <div class="playlist-actions">
      <Button size="sm" variant="secondary" onclick={onSyncPlaylist}>
        <RefreshCw size={14} aria-hidden="true" /> Sync all
      </Button>
      <Button size="sm" variant="secondary" onclick={onRetryFailed}>
        <RotateCcw size={14} aria-hidden="true" /> Retry failed
      </Button>
    </div>
  </div>

  {#if loading}
    <StatusMessage tone="muted" surface={false}>Loading YouTube playlist...</StatusMessage>
  {:else if !playlist || !summary}
    <StatusMessage tone="muted" surface={false}>YouTube playlist detail is not loaded.</StatusMessage>
  {:else}
    <div class="playlist-status">
      {@render detailField("Captions", `${summary.captions.label} - ${formatTimestamp(summary.captions.lastSyncedAt)}`)}
      {@render detailField("Comments", `${summary.comments.label} - ${formatTimestamp(summary.comments.lastSyncedAt)}`)}
      {@render detailField("Availability", availabilityLabel(summary.availabilityStatus))}
    </div>

    <YoutubeSourceActivity
      jobs={sourceJobs}
      {formatTimestamp}
      onCancelJob={onCancelSourceJob}
      title="Playlist activity"
    />

    {#if playlist.items.length === 0}
      <StatusMessage tone="muted" surface={false}>
        No linked videos are available for this playlist. Sync the playlist to load video rows.
      </StatusMessage>
    {:else}
      <div class="playlist-items">
        {#each items as item (item.videoId)}
          <article class:removed={item.isRemovedFromPlaylist} class="playlist-row">
            <div class="playlist-thumb" aria-hidden="true">
              {#if item.thumbnailUrl}
                <img src={item.thumbnailUrl} alt="" loading="lazy" />
              {:else}
                <Video size={18} />
              {/if}
            </div>
            <div class="playlist-copy">
              <div class="playlist-title-line">
                <strong>{item.position !== null ? `${item.position}. ` : ""}{item.title ?? item.videoId}</strong>
                {#if item.durationSeconds !== null}
                  <span>{formatDuration(item.durationSeconds)}</span>
                {/if}
              </div>
              <div class="playlist-meta">
                <Badge variant={item.availabilityStatus === "available" ? "neutral" : "warning"}>
                  {availabilityLabel(item.availabilityStatus)}
                </Badge>
                <Badge variant={item.captions.state === "synced" ? "success" : item.captions.state === "unavailable" ? "warning" : "neutral"}>
                  {item.captions.label}
                </Badge>
                <Badge variant={item.comments.state === "synced" ? "success" : "neutral"}>
                  {item.comments.label}
                </Badge>
                {#if item.publishedAt !== null}
                  <span>{formatTimestamp(item.publishedAt)}</span>
                {/if}
              </div>
            </div>
            <div class="playlist-row-actions">
              <Button
                size="sm"
                variant="ghost"
                ariaLabel="Open video source"
                title="Open video source"
                disabled={item.videoSourceId === null}
                onclick={() => item.videoSourceId !== null && onOpenSource(item.videoSourceId)}
              >
                <ExternalLink size={15} aria-hidden="true" />
              </Button>
              <Button
                size="sm"
                variant="ghost"
                ariaLabel="Sync this video"
                title="Sync this video"
                disabled={!canSyncItem(item)}
                onclick={() => item.videoSourceId !== null && onSyncPlaylistVideo(item.videoSourceId)}
              >
                <RefreshCw size={15} aria-hidden="true" />
              </Button>
              <Button
                size="sm"
                variant="ghost"
                ariaLabel="Retry this video"
                title="Retry this video"
                disabled={!canRetryItem(item)}
                onclick={() => item.videoSourceId !== null && onRetryPlaylistVideo(item.videoSourceId)}
              >
                <RotateCcw size={15} aria-hidden="true" />
              </Button>
            </div>
          </article>
        {/each}
      </div>
    {/if}
  {/if}
</section>

{#snippet detailField(label: string, value: string)}
  <div class="detail-field">
    <span>{label}</span>
    <strong>{value}</strong>
  </div>
{/snippet}

<style>
  .youtube-playlist-reader {
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
    min-width: 0;
  }

  .playlist-header,
  .playlist-row {
    display: flex;
    gap: 0.75rem;
    align-items: flex-start;
  }

  .playlist-header {
    justify-content: space-between;
  }

  .playlist-title {
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

  .playlist-meta,
  .playlist-actions,
  .playlist-row-actions {
    display: flex;
    flex-wrap: wrap;
    gap: 0.45rem;
    align-items: center;
  }

  .playlist-status {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 0.6rem;
  }

  .detail-field {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    padding: 0.75rem;
    border-radius: 8px;
    border: 1px solid var(--border);
    background: var(--panel-strong);
    min-width: 0;
  }

  .detail-field span,
  .playlist-meta span {
    color: var(--muted);
    font-size: 0.75rem;
  }

  .detail-field strong,
  .playlist-title-line strong {
    overflow-wrap: anywhere;
  }

  .playlist-items {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .playlist-row {
    padding: 0.65rem;
    border-radius: 8px;
    border: 1px solid var(--border);
    background: var(--panel);
  }

  .playlist-row.removed {
    opacity: 0.72;
  }

  .playlist-thumb {
    flex: 0 0 4.5rem;
    width: 4.5rem;
    aspect-ratio: 16 / 9;
    border-radius: 6px;
    overflow: hidden;
    background: var(--panel-hover);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    color: var(--muted);
  }

  .playlist-thumb img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .playlist-copy {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .playlist-title-line {
    display: flex;
    justify-content: space-between;
    gap: 0.5rem;
  }

  @media (max-width: 840px) {
    .playlist-header,
    .playlist-row {
      flex-direction: column;
    }

    .playlist-status {
      grid-template-columns: 1fr;
    }

    .playlist-thumb {
      width: 100%;
      flex-basis: auto;
    }
  }
</style>
