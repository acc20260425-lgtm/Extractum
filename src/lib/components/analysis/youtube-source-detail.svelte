<script lang="ts">
  import { FileText, MessageSquare, RefreshCw, Square, Video } from "@lucide/svelte";
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import StatusMessage from "$lib/components/ui/StatusMessage.svelte";
  import type { SourceJobRecord } from "$lib/types/sources";
  import type { Source } from "$lib/types/sources";
  import type { YoutubeVideoDetail } from "$lib/types/youtube";

  type TabKey = "overview" | "transcript" | "comments" | "jobs";

  let {
    source,
    detail,
    jobs,
    loadingDetail,
    formatTimestamp,
    onSyncMetadata,
    onSyncTranscript,
    onSyncComments,
    onCancelJob,
  }: {
    source: Source;
    detail: YoutubeVideoDetail | null;
    jobs: SourceJobRecord[];
    loadingDetail: boolean;
    formatTimestamp: (value: number | null) => string;
    onSyncMetadata: (sourceId: number) => void | Promise<void>;
    onSyncTranscript: (sourceId: number) => void | Promise<void>;
    onSyncComments: (sourceId: number) => void | Promise<void>;
    onCancelJob: (jobId: string) => void | Promise<void>;
  } = $props();

  let activeTab = $state<TabKey>("overview");

  const summary = $derived(detail?.summary ?? null);
  const sourceJobs = $derived(jobs.filter((job) => job.source_id === source.id || job.related_source_id === source.id));

  function availabilityLabel(value: string | null | undefined) {
    return value ? value.replaceAll("_", " ") : "unknown";
  }

  function formatDuration(value: number | null | undefined) {
    if (value === null || value === undefined) return "Unknown";
    const hours = Math.floor(value / 3600);
    const minutes = Math.floor((value % 3600) / 60);
    const seconds = value % 60;
    if (hours > 0) {
      return `${hours}:${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`;
    }
    return `${minutes}:${String(seconds).padStart(2, "0")}`;
  }

  function isActiveJob(job: SourceJobRecord) {
    return job.status === "queued" || job.status === "running" || job.status === "cancel_requested";
  }

  function jobLabel(job: SourceJobRecord) {
    return job.job_type.replaceAll("_", " ");
  }
</script>

<section class="youtube-detail">
  <div class="detail-header">
    <div class="detail-title">
      <span class="eyebrow">YouTube video</span>
      <h3>{summary?.title ?? source.title ?? source.externalId}</h3>
      <div class="detail-meta">
        <Badge variant="info">{summary?.channelHandle ?? summary?.channelTitle ?? "YouTube"}</Badge>
        <Badge>{availabilityLabel(summary?.availabilityStatus)}</Badge>
        {#if detail && detail.playlistMemberships.length > 0}
          <Badge variant="neutral">{detail.playlistMemberships.length} playlist{detail.playlistMemberships.length === 1 ? "" : "s"}</Badge>
        {/if}
      </div>
    </div>
    <div class="detail-actions">
      <Button size="sm" variant="secondary" onclick={() => onSyncMetadata(source.id)}>
        <RefreshCw size={14} aria-hidden="true" /> Metadata
      </Button>
      <Button size="sm" variant="secondary" onclick={() => onSyncTranscript(source.id)}>
        <FileText size={14} aria-hidden="true" /> Transcript
      </Button>
      <Button size="sm" variant="secondary" onclick={() => onSyncComments(source.id)}>
        <MessageSquare size={14} aria-hidden="true" /> Comments
      </Button>
    </div>
  </div>

  <div class="tabs" role="tablist" aria-label="YouTube video detail">
    {#each ["overview", "transcript", "comments", "jobs"] as tab (tab)}
      <Button
        size="sm"
        variant="ghost"
        role="tab"
        selected={activeTab === tab}
        ariaSelected={activeTab === tab}
        onclick={() => (activeTab = tab as TabKey)}
      >
        {tab}
      </Button>
    {/each}
  </div>

  {#if loadingDetail}
    <StatusMessage tone="muted" surface={false}>Loading YouTube detail...</StatusMessage>
  {:else if !detail || !summary}
    <StatusMessage tone="muted" surface={false}>YouTube detail is not loaded.</StatusMessage>
  {:else if activeTab === "overview"}
    <div class="detail-grid">
      {@render detailField("Channel", summary.channelHandle ?? summary.channelTitle ?? "Unknown")}
      {@render detailField("Published", formatTimestamp(summary.publishedAt))}
      {@render detailField("Duration", formatDuration(summary.durationSeconds))}
      {@render detailField("Availability", availabilityLabel(summary.availabilityStatus))}
      {#if summary.canonicalUrl}
        <a class="detail-link" href={summary.canonicalUrl} target="_blank" rel="noreferrer">
          <Video size={15} aria-hidden="true" /> Open on YouTube
        </a>
      {/if}
    </div>
    {#if detail.playlistMemberships.length > 0}
      <div class="membership-list">
        {#each detail.playlistMemberships as membership (membership.playlistSourceId)}
          <Badge variant="neutral">
            {membership.playlistTitle ?? `Playlist #${membership.playlistSourceId}`}
            {membership.position !== null ? ` #${membership.position}` : ""}
          </Badge>
        {/each}
      </div>
    {/if}
  {:else if activeTab === "transcript"}
    <div class="status-grid">
      {@render detailField("State", summary.captions.label)}
      {@render detailField("Items", String(summary.captions.itemCount))}
      {@render detailField("Segments", String(summary.captions.segmentCount))}
      {@render detailField("Last synced", formatTimestamp(summary.captions.lastSyncedAt))}
    </div>
  {:else if activeTab === "comments"}
    <div class="status-grid">
      {@render detailField("State", summary.comments.label)}
      {@render detailField("Items", String(summary.comments.itemCount))}
      {@render detailField("Last synced", formatTimestamp(summary.comments.lastSyncedAt))}
    </div>
  {:else}
    <div class="job-list">
      {#if sourceJobs.length === 0}
        <StatusMessage tone="muted" surface={false}>No YouTube jobs for this source.</StatusMessage>
      {:else}
        {#each sourceJobs as job (job.job_id)}
          <article class="job-row">
            <div>
              <strong>{jobLabel(job)}</strong>
              <span>{job.message ?? job.error ?? job.status}</span>
            </div>
            <Badge variant={job.status === "failed" ? "danger" : job.status === "succeeded" ? "success" : "info"}>
              {job.status.replaceAll("_", " ")}
            </Badge>
            {#if isActiveJob(job)}
              <Button
                size="sm"
                variant="secondary"
                onclick={() => onCancelJob(job.job_id)}
                disabled={job.status === "cancel_requested"}
              >
                <Square size={13} aria-hidden="true" /> Cancel
              </Button>
            {/if}
          </article>
        {/each}
      {/if}
    </div>
  {/if}
</section>

{#snippet detailField(label: string, value: string)}
  <div class="detail-field">
    <span>{label}</span>
    <strong>{value}</strong>
  </div>
{/snippet}

<style>
  .youtube-detail {
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
    padding: 1rem;
    border: 1px solid var(--border);
    border-radius: 14px;
    background: var(--panel);
    box-shadow: var(--shadow);
  }

  .detail-header {
    display: flex;
    justify-content: space-between;
    gap: 0.75rem;
    align-items: flex-start;
  }

  .eyebrow {
    display: inline-block;
    font-size: 0.68rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--muted);
    margin-bottom: 0.2rem;
  }

  h3 {
    margin: 0;
  }

  .detail-meta,
  .detail-actions,
  .tabs,
  .membership-list {
    display: flex;
    flex-wrap: wrap;
    gap: 0.45rem;
    align-items: center;
  }

  .detail-title {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    min-width: 0;
  }

  .detail-grid,
  .status-grid {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
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

  .detail-field span {
    color: var(--muted);
    font-size: 0.75rem;
  }

  .detail-field strong {
    font-size: 0.9rem;
    overflow-wrap: anywhere;
  }

  .detail-link {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0.75rem;
    border-radius: 8px;
    border: 1px solid var(--border);
    color: var(--text);
    text-decoration: none;
  }

  .job-list {
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
  }

  .job-row {
    display: flex;
    gap: 0.55rem;
    justify-content: space-between;
    align-items: center;
    padding: 0.65rem;
    border-radius: 8px;
    border: 1px solid var(--border);
    background: var(--panel-strong);
  }

  .job-row div {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    min-width: 0;
  }

  .job-row span {
    color: var(--muted);
    font-size: 0.78rem;
    overflow-wrap: anywhere;
  }

  @media (max-width: 840px) {
    .detail-header {
      flex-direction: column;
    }

    .detail-grid,
    .status-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
