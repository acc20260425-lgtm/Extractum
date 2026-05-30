<script lang="ts">
  import { RefreshCw } from "@lucide/svelte";
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import StatusMessage from "$lib/components/ui/StatusMessage.svelte";
  import RawJsonPanel from "$lib/components/analysis/raw-json-panel.svelte";
  import type { Source, SourceForumTopic } from "$lib/types/sources";
  import type { YoutubeVideoDetail } from "$lib/types/youtube";

  let {
    source,
    youtubeVideoDetail,
    sourceTopics,
    loading,
    formatTimestamp,
    onSyncMetadata,
  }: {
    source: Source;
    youtubeVideoDetail: YoutubeVideoDetail | null;
    sourceTopics: SourceForumTopic[];
    loading: boolean;
    formatTimestamp: (value: number | null) => string;
    onSyncMetadata: () => void | Promise<void>;
  } = $props();

  const youtubeMetadata = $derived(youtubeVideoDetail?.sourceMetadata ?? null);
  const summary = $derived(youtubeVideoDetail?.summary ?? null);
  const rawJson = $derived(youtubeMetadata?.rawMetadataJson ?? null);
  const sourceTitle = $derived(source.title ?? source.externalId);
  const sourceKind = $derived([source.sourceType, source.sourceSubtype].filter(Boolean).join(" / "));
  const visibleTopicCount = $derived(sourceTopics.filter((topic) => !topic.isDeleted).length);

  function textValue(value: string | number | null | undefined) {
    if (value === null || value === undefined || value === "") return "Not available";
    return String(value);
  }

  function yesNo(value: boolean) {
    return value ? "Yes" : "No";
  }
</script>

<section class="source-metadata-view" aria-label="Source metadata">
  <div class="metadata-header">
    <div>
      <span class="eyebrow">Metadata</span>
      <h3>{sourceTitle}</h3>
    </div>
    {#if source.sourceType === "youtube"}
      <Button type="button" variant="secondary" disabled={loading} onclick={onSyncMetadata}>
        <RefreshCw size={14} aria-hidden="true" />
        {loading ? "Syncing metadata" : "Sync metadata"}
      </Button>
    {/if}
  </div>

  {#if loading}
    <StatusMessage tone="info">Loading metadata...</StatusMessage>
  {/if}

  <section class="metadata-section" aria-labelledby="metadata-summary-title">
    <h4 id="metadata-summary-title">Summary</h4>
    <dl class="metadata-grid">
      <div>
        <dt>Title</dt>
        <dd>{textValue(summary?.title ?? source.title)}</dd>
      </div>
      <div>
        <dt>Kind</dt>
        <dd>{sourceKind}</dd>
      </div>
      <div>
        <dt>External ID</dt>
        <dd>{source.externalId}</dd>
      </div>
      <div>
        <dt>Canonical URL</dt>
        <dd>
          {#if summary?.canonicalUrl ?? youtubeMetadata?.canonicalUrl}
            <a href={summary?.canonicalUrl ?? youtubeMetadata?.canonicalUrl} target="_blank" rel="noreferrer">
              {summary?.canonicalUrl ?? youtubeMetadata?.canonicalUrl}
            </a>
          {:else}
            Not available
          {/if}
        </dd>
      </div>
      <div>
        <dt>Created</dt>
        <dd>{formatTimestamp(source.createdAt)}</dd>
      </div>
      <div>
        <dt>Last synced</dt>
        <dd>{formatTimestamp(source.lastSyncedAt)}</dd>
      </div>
    </dl>
  </section>

  <section class="metadata-section" aria-labelledby="metadata-state-title">
    <h4 id="metadata-state-title">Source state</h4>
    <div class="badge-row">
      <Badge variant={source.isActive ? "success" : "warning"}>{source.isActive ? "Active" : "Inactive"}</Badge>
      <Badge variant={source.isMember ? "success" : "neutral"}>{source.isMember ? "Member" : "Not member"}</Badge>
      {#if summary?.captions}
        <Badge variant={summary.captions.state === "synced" ? "success" : summary.captions.state === "unavailable" ? "warning" : "neutral"}>
          {summary.captions.label}
        </Badge>
      {/if}
      {#if summary?.comments}
        <Badge variant={summary.comments.state === "synced" ? "success" : "neutral"}>
          {summary.comments.label}
        </Badge>
      {/if}
    </div>
    <dl class="metadata-grid">
      {#if source.sourceType === "telegram"}
        <div>
          <dt>Topics</dt>
          <dd>{visibleTopicCount}</dd>
        </div>
        <div>
          <dt>Migrated history</dt>
          <dd>{source.migratedHistoryStatus}</dd>
        </div>
        <div>
          <dt>Migrated rows</dt>
          <dd>{source.migratedHistoryRowCount}</dd>
        </div>
        <div>
          <dt>Migrated import complete</dt>
          <dd>{yesNo(source.migratedHistoryImportCompleted)}</dd>
        </div>
      {:else if summary}
        <div>
          <dt>Captions</dt>
          <dd>{summary.captions.itemCount} items / {summary.captions.segmentCount} segments</dd>
        </div>
        <div>
          <dt>Comments</dt>
          <dd>{summary.comments.itemCount} loaded items</dd>
        </div>
        <div>
          <dt>Playlist memberships</dt>
          <dd>{youtubeVideoDetail?.playlistMemberships.length ?? 0}</dd>
        </div>
      {:else}
        <div>
          <dt>Status</dt>
          <dd>Not available</dd>
        </div>
      {/if}
    </dl>
  </section>

  <section class="metadata-section" aria-labelledby="metadata-technical-title">
    <h4 id="metadata-technical-title">Technical</h4>
    <dl class="metadata-grid">
      <div>
        <dt>Source ID</dt>
        <dd>{source.id}</dd>
      </div>
      <div>
        <dt>Account ID</dt>
        <dd>{textValue(source.accountId)}</dd>
      </div>
      <div>
        <dt>Provider type</dt>
        <dd>{source.sourceType}</dd>
      </div>
      <div>
        <dt>Provider subtype</dt>
        <dd>{textValue(source.sourceSubtype)}</dd>
      </div>
      {#if youtubeMetadata}
        <div>
          <dt>Video ID</dt>
          <dd>{youtubeMetadata.videoId}</dd>
        </div>
        <div>
          <dt>Video form</dt>
          <dd>{youtubeMetadata.videoForm}</dd>
        </div>
        <div>
          <dt>Availability</dt>
          <dd>{youtubeMetadata.availabilityStatus}</dd>
        </div>
        <div>
          <dt>Channel ID</dt>
          <dd>{textValue(youtubeMetadata.channelId)}</dd>
        </div>
        <div>
          <dt>Channel handle</dt>
          <dd>{textValue(youtubeMetadata.channelHandle)}</dd>
        </div>
        <div>
          <dt>Duration</dt>
          <dd>{textValue(youtubeMetadata.durationSeconds)}</dd>
        </div>
        <div>
          <dt>Published</dt>
          <dd>{formatTimestamp(youtubeMetadata.publishedAt)}</dd>
        </div>
        <div>
          <dt>Caption override</dt>
          <dd>{textValue(youtubeMetadata.captionLanguageOverride)}</dd>
        </div>
        <div>
          <dt>View count</dt>
          <dd>{textValue(youtubeMetadata.viewCount)}</dd>
        </div>
        <div>
          <dt>Like count</dt>
          <dd>{textValue(youtubeMetadata.likeCount)}</dd>
        </div>
        <div>
          <dt>Comment count</dt>
          <dd>{textValue(youtubeMetadata.commentCount)}</dd>
        </div>
        <div>
          <dt>Raw metadata version</dt>
          <dd>{textValue(youtubeMetadata.rawMetadataVersion)}</dd>
        </div>
      {/if}
    </dl>
    {#if source.sourceType === "youtube" && !loading && !youtubeMetadata}
      <EmptyState description="No typed YouTube metadata is loaded for this source." />
    {/if}
  </section>

  {#if source.sourceType === "youtube"}
    <section class="metadata-section" aria-labelledby="metadata-raw-title">
      <h4 id="metadata-raw-title">Raw JSON</h4>
      <RawJsonPanel value={rawJson} />
    </section>
  {/if}
</section>

<style>
  .source-metadata-view {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    min-width: 0;
  }

  .metadata-header,
  .badge-row {
    display: flex;
    gap: 0.55rem;
    align-items: center;
    flex-wrap: wrap;
  }

  .metadata-header {
    justify-content: space-between;
    align-items: flex-start;
  }

  .metadata-header h3,
  .metadata-section h4,
  .metadata-grid,
  .metadata-grid dd {
    margin: 0;
  }

  .metadata-header h3 {
    font-size: 1.05rem;
  }

  .eyebrow {
    color: var(--muted);
    font-size: 0.68rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  .metadata-section {
    display: flex;
    flex-direction: column;
    gap: 0.65rem;
    min-width: 0;
    padding-top: 0.85rem;
    border-top: 1px solid var(--border);
  }

  .metadata-section h4 {
    font-size: 0.95rem;
  }

  .metadata-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(13rem, 1fr));
    gap: 0.7rem 1rem;
  }

  .metadata-grid div {
    min-width: 0;
  }

  .metadata-grid dt {
    color: var(--muted);
    font-size: 0.72rem;
    line-height: 1.35;
  }

  .metadata-grid dd {
    color: var(--text);
    font-size: 0.9rem;
    line-height: 1.45;
    overflow-wrap: anywhere;
  }

  .metadata-grid a {
    color: var(--primary);
  }
</style>
