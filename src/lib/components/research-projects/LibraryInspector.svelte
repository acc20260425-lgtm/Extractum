<script lang="ts">
  import { ExternalLink, Link2, PlayCircle, RefreshCw } from "@lucide/svelte";
  import { ExtractumButton, ProviderBadge, StatusBadge } from "$lib/components/extractum-ui";
  import type { LibraryCatalogSourceView } from "$lib/ui/library-catalog-model";
  import YoutubeSummaryRunDialog from "./YoutubeSummaryRunDialog.svelte";

  let { selectedSource }: { selectedSource: LibraryCatalogSourceView | null } = $props();
  let youtubeSummaryOpen = $state(false);

  type MetadataRow = { label: string; value: string | null; href?: string };

  function present(value: string | number | null | undefined) {
    if (value === null || value === undefined || value === "") return null;
    return String(value);
  }

  function secondsLabel(seconds: number | null | undefined) {
    if (seconds === null || seconds === undefined) return null;
    const minutes = Math.floor(seconds / 60);
    const remainder = seconds % 60;
    if (minutes <= 0) return `${remainder}s`;
    return `${minutes}m ${remainder}s`;
  }

  let metadataRows = $derived<MetadataRow[]>(
    selectedSource
      ? [
          { label: "Source ID", value: String(selectedSource.sourceId) },
          { label: "Type", value: selectedSource.typeLabel },
          {
            label: "Canonical URL",
            value: present(selectedSource.canonicalUrl),
            href: selectedSource.canonicalUrl ?? undefined,
          },
          { label: "External ID", value: present(selectedSource.externalId) },
          { label: "Added", value: selectedSource.addedAtLabel },
          { label: "Last synced", value: selectedSource.lastSyncedLabel },
          { label: "Items", value: selectedSource.itemCountLabel },
          { label: "Projects", value: String(selectedSource.projectCount) },
        ]
      : [],
  );

  let youtubeRows = $derived<MetadataRow[]>(
    selectedSource?.youtube
      ? [
          { label: "Channel", value: present(selectedSource.youtube.channel_title) },
          { label: "Video form", value: present(selectedSource.youtube.video_form) },
          { label: "Duration", value: secondsLabel(selectedSource.youtube.duration_seconds) },
          { label: "Playlist videos", value: present(selectedSource.youtube.playlist_video_count) },
          { label: "Availability", value: present(selectedSource.youtube.availability_status) },
        ]
      : [],
  );

  let telegramRows = $derived<MetadataRow[]>(
    selectedSource?.telegram
      ? [
          { label: "Subtype", value: present(selectedSource.sourceSubtype) },
          { label: "Account", value: present(selectedSource.telegram.account_id) },
        ]
      : [],
  );

  let canRunYoutubeSummary = $derived(
    selectedSource?.provider === "youtube" &&
      (selectedSource.sourceSubtype === "video" || selectedSource.sourceSubtype === "playlist") &&
      selectedSource.status === "active" &&
      selectedSource.lastSyncedLabel !== "Never",
  );
</script>

<aside data-ui-region="library-inspector" class="library-inspector extractum-panel-shell" aria-label="Library source inspector">
  {#if selectedSource}
    <header class="inspector-header">
      <div>
        <p class="eyebrow">Selected source</p>
        <h2>{selectedSource.title}</h2>
      </div>
      <ProviderBadge provider={selectedSource.provider} />
    </header>

    <div class="status-row">
      <StatusBadge status={selectedSource.status} />
      {#if selectedSource.statusDetail}
        <span class="meta-pill">{selectedSource.statusDetail}</span>
      {/if}
    </div>

    <dl class="meta-list">
      {#each metadataRows as row (row.label)}
        <div>
          <dt>{row.label}</dt>
          <dd>
            {#if row.href && row.value}
              <a href={row.href} target="_blank" rel="noreferrer">{row.value}</a>
            {:else}
              {row.value ?? "N/A"}
            {/if}
          </dd>
        </div>
      {/each}
    </dl>

    {#if selectedSource.youtube}
      <section class="detail-section" aria-label="YouTube details">
        <h3>YouTube details</h3>
        <dl class="meta-list">
          {#each youtubeRows as row (row.label)}
            <div>
              <dt>{row.label}</dt>
              <dd>{row.value ?? "N/A"}</dd>
            </div>
          {/each}
        </dl>
      </section>
    {/if}

    {#if selectedSource.telegram}
      <section class="detail-section" aria-label="Telegram details">
        <h3>Telegram details</h3>
        <dl class="meta-list">
          {#each telegramRows as row (row.label)}
            <div>
              <dt>{row.label}</dt>
              <dd>{row.value ?? "N/A"}</dd>
            </div>
          {/each}
        </dl>
      </section>
    {/if}

    <div class="commands" aria-label="Inspector commands">
      <ExtractumButton variant="outline" aria-label="Open selected source">
        <ExternalLink size={14} aria-hidden="true" />
        Open
      </ExtractumButton>
      <ExtractumButton variant="outline" aria-label="Sync selected source">
        <RefreshCw size={14} aria-hidden="true" />
        Sync
      </ExtractumButton>
      <ExtractumButton variant="outline" aria-label="Open selected source connection dialog">
        <Link2 size={14} aria-hidden="true" />
        Connect
      </ExtractumButton>
      <ExtractumButton variant="outline" aria-label="Run report for selected source">
        <PlayCircle size={14} aria-hidden="true" />
        Run report
      </ExtractumButton>
      {#if canRunYoutubeSummary}
        <ExtractumButton variant="outline" aria-label="Run YouTube summary for selected source" onclick={() => (youtubeSummaryOpen = true)}>
          <PlayCircle size={14} aria-hidden="true" />YouTube Summary
        </ExtractumButton>
      {/if}
    </div>
    <YoutubeSummaryRunDialog bind:open={youtubeSummaryOpen} projectId={null} source={selectedSource} />
  {:else}
    <div class="empty-state extractum-panel-shell">
      <p class="eyebrow">Inspector</p>
      <h2>No source selected</h2>
      <p>Select a source row to inspect metadata and available commands.</p>
    </div>
  {/if}
</aside>

<style>
  .library-inspector {
    min-width: 0;
    min-height: 0;
    padding: 12px;
    overflow: auto;
    background: transparent;
  }

  .inspector-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
  }

  .eyebrow {
    margin: 0 0 6px;
    color: var(--extractum-muted);
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
  }

  h2 {
    margin: 0;
    font-size: 16px;
    font-weight: 700;
    line-height: 1.25;
  }

  h3 {
    margin: 12px 0 8px;
    font-size: 13px;
    font-weight: 700;
  }

  a {
    color: var(--extractum-primary);
    text-decoration: none;
  }

  a:hover {
    text-decoration: underline;
  }

  .status-row,
  .commands {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-top: 12px;
  }

  .commands {
    display: grid;
    grid-template-columns: 1fr 1fr;
  }

  .meta-list {
    display: grid;
    gap: 8px;
    margin: 14px 0;
  }

  .meta-list div {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    border-bottom: 1px solid var(--extractum-border);
    padding-bottom: 6px;
  }

  dt {
    color: var(--extractum-muted);
    font-size: 12px;
  }

  dd {
    margin: 0;
    font-size: 12px;
    text-align: right;
  }

  .meta-pill {
    border: 1px solid var(--extractum-border);
    border-radius: var(--extractum-radius);
    padding: 4px 7px;
    color: var(--extractum-muted);
    font-size: 12px;
  }

  .empty-state {
    display: grid;
    min-height: 220px;
    align-content: center;
    gap: 8px;
    color: var(--extractum-muted);
  }
</style>
