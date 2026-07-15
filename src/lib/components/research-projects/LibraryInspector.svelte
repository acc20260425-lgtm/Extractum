<script lang="ts">
  import ExternalLink from "@lucide/svelte/icons/external-link";
  import Link2 from "@lucide/svelte/icons/link-2";
  import PlayCircle from "@lucide/svelte/icons/circle-play";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
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

<aside
  data-ui-region="library-inspector"
  class="library-inspector flex min-h-0 min-w-0 flex-col gap-3 overflow-auto p-3 bg-[var(--extractum-surface-subtle)]"
  aria-label="Library source inspector"
>
  {#if selectedSource}
    <section class="extractum-panel-shell">
      <header class="flex items-start justify-between gap-3">
        <div>
          <p class="text-xs font-bold uppercase leading-none tracking-wide text-[var(--extractum-muted)]">Selected source</p>
          <h2 class="m-0 text-base font-bold leading-tight">{selectedSource.title}</h2>
        </div>
        <ProviderBadge provider={selectedSource.provider} />
      </header>

      <div class="status-row flex flex-wrap gap-2 mt-3">
        <StatusBadge status={selectedSource.status} />
        {#if selectedSource.statusDetail}
          <span class="rounded border border-[var(--extractum-border)] px-2 py-1 text-xs text-[var(--extractum-muted)]">{selectedSource.statusDetail}</span>
        {/if}
      </div>
    </section>

    <section class="extractum-panel-shell">
      <h3 class="m-0 text-xs font-bold uppercase tracking-wide text-[var(--extractum-muted)]">Source metadata</h3>
      <dl class="meta-list">
        {#each metadataRows as row (row.label)}
          <div>
            <dt class="text-[var(--extractum-muted)] text-xs font-medium">{row.label}</dt>
            <dd>
              {#if row.href && row.value}
                <a
                  class="text-[var(--extractum-primary)] no-underline hover:underline"
                  href={row.href}
                  target="_blank"
                  rel="noreferrer"
                >
                  {row.value}
                </a>
              {:else}
                {row.value ?? "N/A"}
              {/if}
            </dd>
          </div>
        {/each}
      </dl>
    </section>

    {#if selectedSource.youtube}
      <section class="detail-section extractum-panel-shell" aria-label="YouTube details">
        <h3 class="m-0 text-xs font-bold uppercase tracking-wide text-[var(--extractum-muted)]">YouTube details</h3>
        <dl class="meta-list">
          {#each youtubeRows as row (row.label)}
            <div>
              <dt class="text-[var(--extractum-muted)] text-xs font-medium">{row.label}</dt>
              <dd>{row.value ?? "N/A"}</dd>
            </div>
          {/each}
        </dl>
      </section>
    {/if}

    {#if selectedSource.telegram}
      <section class="detail-section extractum-panel-shell" aria-label="Telegram details">
        <h3 class="m-0 text-xs font-bold uppercase tracking-wide text-[var(--extractum-muted)]">Telegram details</h3>
        <dl class="meta-list">
          {#each telegramRows as row (row.label)}
            <div>
              <dt class="text-[var(--extractum-muted)] text-xs font-medium">{row.label}</dt>
              <dd>{row.value ?? "N/A"}</dd>
            </div>
          {/each}
        </dl>
      </section>
    {/if}

    <section class="extractum-panel-shell">
      <h3 class="m-0 text-xs font-bold uppercase tracking-wide text-[var(--extractum-muted)]">Commands</h3>
      <div class="commands grid grid-cols-2 gap-2 mt-3" aria-label="Inspector commands">
        <ExtractumButton
          variant="outline"
          aria-label={`Open selected source ${selectedSource.title}`}
          title={`Open selected source ${selectedSource.title}`}
        >
          <ExternalLink size={14} aria-hidden="true" />
          Open
        </ExtractumButton>
        <ExtractumButton
          variant="outline"
          aria-label={`Sync selected source ${selectedSource.title}`}
          title={`Sync selected source ${selectedSource.title}`}
        >
          <RefreshCw size={14} aria-hidden="true" />
          Sync
        </ExtractumButton>
        <ExtractumButton
          variant="outline"
          aria-label={`Open connection dialog for source ${selectedSource.title}`}
          title={`Open connection dialog for source ${selectedSource.title}`}
        >
          <Link2 size={14} aria-hidden="true" />
          Connect
        </ExtractumButton>
        <ExtractumButton
          variant="outline"
          aria-label={`Run report for source ${selectedSource.title}`}
          title={`Run report for source ${selectedSource.title}`}
        >
          <PlayCircle size={14} aria-hidden="true" />
          Run report
        </ExtractumButton>
        {#if canRunYoutubeSummary}
          <ExtractumButton
            variant="outline"
            aria-label={`Run YouTube summary for source ${selectedSource.title}`}
            title={`Run YouTube summary for source ${selectedSource.title}`}
            onclick={() => (youtubeSummaryOpen = true)}
          >
            <PlayCircle size={14} aria-hidden="true" />YouTube Summary
          </ExtractumButton>
        {/if}
      </div>
    </section>
    <YoutubeSummaryRunDialog bind:open={youtubeSummaryOpen} projectId={null} source={selectedSource} />
  {:else}
    <div class="empty-state extractum-panel-shell min-h-[220px] grid content-center gap-2 text-[var(--extractum-muted)]">
      <p class="text-xs font-bold uppercase tracking-wide text-[var(--extractum-muted)]">Inspector</p>
      <h2 class="m-0 text-base font-bold leading-tight">No source selected</h2>
      <p class="m-0">Select a source row to inspect metadata and available commands.</p>
    </div>
  {/if}
</aside>

<style>
  .meta-list {
    display: grid;
    gap: 8px;
    margin: 8px 0 0;
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
</style>
