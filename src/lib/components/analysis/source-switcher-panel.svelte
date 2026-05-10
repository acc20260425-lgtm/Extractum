<script lang="ts">
  import { Archive, ExternalLink, Plus, RefreshCw, Search, Square, Trash2 } from "@lucide/svelte";
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import { membershipLabel, sourceCapabilities, sourceKindLabel } from "$lib/source-capabilities";
  import type { AccountRuntimeStatus } from "$lib/types/accounts";
  import type { AnalysisSourceGroup, AnalysisSourceOption } from "$lib/types/analysis";
  import type { Source, SourceJobRecord, TakeoutImportJobRecord } from "$lib/types/sources";
  import type { YoutubeRuntimeStatus, YoutubeSourceSummary } from "$lib/types/youtube";
  import type { WorkspaceSelection } from "$lib/analysis-workspace-state";
  import type { BadgeVariant } from "$lib/components/ui/types";

  let {
    sourceCatalog,
    groups,
    sourceMetrics,
    loadingSourceCatalog,
    loadingGroups,
    railQuery,
    filteredSourceCatalog,
    filteredGroups,
    workspaceSelection,
    syncingIds,
    deletingSourceIds,
    startingTakeoutSourceIds,
    takeoutJobsBySource,
    sourceJobsBySource,
    youtubeSummaries,
    youtubeRuntimeStatus,
    formatTimestamp,
    accountLabel,
    sourceInitial,
    runtimeStatus,
    runtimeBadge,
    sourceSyncDisabledReason,
    onChangeRailQuery,
    onSelectSource,
    onSelectGroup,
    onSyncSource,
    onStartTakeoutImport,
    onCancelTakeoutImport,
    onCancelSourceJob,
    onOpenSourceManager,
    onDeleteSource,
    onClose,
  }: {
    sourceCatalog: Source[];
    groups: AnalysisSourceGroup[];
    sourceMetrics: Record<number, AnalysisSourceOption>;
    loadingSourceCatalog: boolean;
    loadingGroups: boolean;
    railQuery: string;
    filteredSourceCatalog: Source[];
    filteredGroups: AnalysisSourceGroup[];
    workspaceSelection: WorkspaceSelection;
    syncingIds: Record<number, boolean>;
    deletingSourceIds: Record<number, boolean>;
    startingTakeoutSourceIds: Record<number, boolean>;
    takeoutJobsBySource: Record<number, TakeoutImportJobRecord>;
    sourceJobsBySource: Record<number, SourceJobRecord[]>;
    youtubeSummaries: Record<number, YoutubeSourceSummary>;
    youtubeRuntimeStatus: YoutubeRuntimeStatus | null;
    formatTimestamp: (value: number | null) => string;
    accountLabel: (accountId: number | null) => string;
    sourceInitial: (source: Source) => string;
    runtimeStatus: (accountId: number | null) => AccountRuntimeStatus | null;
    runtimeBadge: (runtime: AccountRuntimeStatus | null) => string;
    sourceSyncDisabledReason: (source: Source) => string | null;
    onChangeRailQuery: (value: string) => void;
    onSelectSource: (sourceId: number) => void;
    onSelectGroup: (groupId: number) => void;
    onSyncSource: (sourceId: number) => void;
    onStartTakeoutImport: (sourceId: number) => void;
    onCancelTakeoutImport: (jobId: string) => void;
    onCancelSourceJob: (jobId: string) => void;
    onOpenSourceManager: () => void;
    onDeleteSource: (source: Source) => void;
    onClose: () => void;
  } = $props();

  function isSelectedSource(sourceId: number) {
    return workspaceSelection.kind === "source" && workspaceSelection.sourceId === sourceId;
  }

  function isSelectedGroup(groupId: number) {
    return workspaceSelection.kind === "source_group" && workspaceSelection.sourceGroupId === groupId;
  }

  function isActiveTakeoutJob(job: TakeoutImportJobRecord | undefined) {
    return (
      job?.status === "queued" ||
      job?.status === "running" ||
      job?.status === "cancel_requested"
    );
  }

  function isActiveSourceJob(job: SourceJobRecord) {
    return job.status === "queued" || job.status === "running" || job.status === "cancel_requested";
  }

  function takeoutPhaseName(phase: TakeoutImportJobRecord["phase"]) {
    return String(phase).replaceAll("_", " ");
  }

  function takeoutPhaseLabel(job: TakeoutImportJobRecord) {
    if (job.status === "failed") return "Takeout failed";
    if (job.status === "completed") return "Takeout complete";
    if (job.status === "cancelled") return "Takeout cancelled";
    if (job.status === "cancel_requested") return "Cancelling Takeout";
    return `Takeout ${takeoutPhaseName(job.phase)}`;
  }

  function takeoutBadgeVariant(job: TakeoutImportJobRecord): BadgeVariant {
    if (job.status === "completed") return "success";
    if (job.status === "failed") return "danger";
    if (job.status === "cancelled") return "neutral";
    if (job.warnings.length > 0) return "warning";
    return "info";
  }

  function availabilityLabel(value: string | null) {
    return value ? value.replaceAll("_", " ") : null;
  }

  function youtubeMetaLine(summary: YoutubeSourceSummary | null) {
    if (!summary) return null;
    return (
      [summary.channelHandle ?? summary.channelTitle, summary.videoCount !== null ? `${summary.videoCount} videos` : null]
        .filter(Boolean)
        .join(" - ") || null
    );
  }
</script>

<section class="source-switcher-panel" aria-label="Source switcher panel">
  <div class="panel-head">
    <div>
      <span class="eyebrow">Sources</span>
      <h2>Switch source context</h2>
    </div>
    <div class="panel-actions">
      <Button size="sm" variant="secondary" onclick={onOpenSourceManager}>
        <Plus size={14} aria-hidden="true" /> New source
      </Button>
      <Button size="sm" variant="ghost" onclick={onOpenSourceManager}>Manage sources</Button>
      <Button size="sm" variant="ghost" onclick={onClose}>Close</Button>
    </div>
  </div>

  <label class="search-field">
    <span>Search sources or groups</span>
    <div class="search-shell">
      <Search size={15} aria-hidden="true" />
      <Input
        type="search"
        value={railQuery}
        placeholder="Search sources or groups"
        ariaLabel="Search sources or groups"
        oninput={(event) => onChangeRailQuery((event.currentTarget as HTMLInputElement).value)}
      />
    </div>
  </label>

  <div class="panel-section">
    <div class="section-title">
      <span>Sources</span>
      <Badge>{loadingSourceCatalog ? "loading" : `${filteredSourceCatalog.length}/${sourceCatalog.length}`}</Badge>
    </div>

    {#if loadingSourceCatalog}
      <div class="panel-empty">Loading sources...</div>
    {:else if filteredSourceCatalog.length === 0}
      <div class="panel-empty">No sources match the current search.</div>
    {:else}
      <div class="source-list">
        {#each filteredSourceCatalog as source (source.id)}
          {@const metrics = sourceMetrics[source.id]}
          {@const capabilities = sourceCapabilities(source)}
          {@const kindLabel = sourceKindLabel(source)}
          {@const sourceMembershipLabel = membershipLabel(source)}
          {@const runtime = runtimeStatus(source.accountId)}
          {@const runtimeStateBadge = runtimeBadge(runtime)}
          {@const syncReason = sourceSyncDisabledReason(source)}
          {@const youtubeSummary = source.sourceType === "youtube" ? youtubeSummaries[source.id] ?? null : null}
          {@const sourceJobs = sourceJobsBySource[source.id] ?? []}
          {@const takeoutJob = takeoutJobsBySource[source.id]}
          {@const takeoutActive = isActiveTakeoutJob(takeoutJob)}
          {@const deleting = !!deletingSourceIds[source.id]}
          <article class:selected={isSelectedSource(source.id)} class="source-row">
            <button
              class="source-main"
              type="button"
              aria-pressed={isSelectedSource(source.id)}
              onclick={() => onSelectSource(source.id)}
            >
              <div class="source-avatar" aria-hidden="true">
                {#if youtubeSummary?.thumbnailUrl ?? source.avatarDataUrl}
                  <img src={youtubeSummary?.thumbnailUrl ?? source.avatarDataUrl ?? ""} alt="" loading="lazy" />
                {:else}
                  <span>{sourceInitial(source)}</span>
                {/if}
              </div>
              <div class="source-copy">
                <strong>{youtubeSummary?.title ?? source.title ?? source.externalId}</strong>
                <div class="source-meta">
                  <span>{kindLabel}</span>
                  <span>{youtubeMetaLine(youtubeSummary) ?? accountLabel(source.accountId)}</span>
                  {#if metrics}
                    <span>{metrics.item_count} {capabilities.contentLabel}</span>
                  {/if}
                </div>
              </div>
            </button>

            <div class="detail-badges">
              {#if runtimeStateBadge}
                <Badge variant="warning" title={runtime?.message ?? undefined}>{runtimeStateBadge}</Badge>
              {/if}
              {#if capabilities.hasMembershipState && sourceMembershipLabel}
                <Badge>{sourceMembershipLabel}</Badge>
              {/if}
              {#if source.sourceType === "youtube" && youtubeRuntimeStatus && !youtubeRuntimeStatus.ytdlpAvailable}
                <Badge variant="warning" title={youtubeRuntimeStatus.message}>yt-dlp unavailable</Badge>
              {/if}
              {#if youtubeSummary}
                <Badge variant={youtubeSummary.captions.state === "synced" ? "success" : youtubeSummary.captions.state === "unavailable" ? "warning" : "neutral"}>
                  {youtubeSummary.captions.label}
                </Badge>
                <Badge variant={youtubeSummary.comments.state === "synced" ? "success" : "neutral"}>
                  {youtubeSummary.comments.label}
                </Badge>
                {#if availabilityLabel(youtubeSummary.availabilityStatus)}
                  <Badge variant={youtubeSummary.availabilityStatus === "available" ? "neutral" : "warning"}>
                    {availabilityLabel(youtubeSummary.availabilityStatus)}
                  </Badge>
                {/if}
                {#if youtubeSummary.canonicalUrl}
                  <a class="panel-link" href={youtubeSummary.canonicalUrl} target="_blank" rel="noreferrer">
                    <ExternalLink size={13} aria-hidden="true" /> YouTube
                  </a>
                {/if}
              {/if}
              {#if takeoutJob}
                <Badge variant={takeoutBadgeVariant(takeoutJob)} title={takeoutJob.error ?? takeoutJob.message ?? undefined}>
                  {takeoutPhaseLabel(takeoutJob)}
                </Badge>
              {/if}
            </div>

            <div class="row-actions">
              {#if capabilities.canSync}
                <Button
                  size="sm"
                  variant="secondary"
                  onclick={() => onSyncSource(source.id)}
                  disabled={!!syncingIds[source.id] || deleting || takeoutActive || syncReason !== null}
                  title={takeoutActive ? "Takeout import is active." : syncReason ?? undefined}
                >
                  <RefreshCw size={13} aria-hidden="true" />
                  {syncingIds[source.id] ? "Syncing..." : "Sync"}
                </Button>
              {/if}
              {#if capabilities.canImportArchive}
                {#if takeoutActive && takeoutJob}
                  <Button
                    size="sm"
                    variant="secondary"
                    onclick={() => onCancelTakeoutImport(takeoutJob.job_id)}
                    disabled={takeoutJob.status === "cancel_requested"}
                  >
                    <Square size={13} aria-hidden="true" />
                    {takeoutJob.status === "cancel_requested" ? "Cancelling..." : "Cancel"}
                  </Button>
                {:else}
                  <Button
                    size="sm"
                    variant="secondary"
                    onclick={() => onStartTakeoutImport(source.id)}
                    disabled={!!startingTakeoutSourceIds[source.id] || deleting || !!syncingIds[source.id] || syncReason !== null}
                    title={syncReason ?? undefined}
                  >
                    <Archive size={13} aria-hidden="true" />
                    {startingTakeoutSourceIds[source.id] ? "Starting..." : "Takeout"}
                  </Button>
                {/if}
              {/if}
              <Button
                size="sm"
                variant="danger-soft"
                onclick={() => onDeleteSource(source)}
                disabled={deleting || !!syncingIds[source.id] || takeoutActive}
                title={takeoutActive ? "Takeout import is active." : undefined}
              >
                <Trash2 size={13} aria-hidden="true" />
                {deleting ? "Deleting..." : "Delete"}
              </Button>
            </div>

            {#if takeoutJob}
              <div class="takeout-status">
                <span>{takeoutPhaseName(takeoutJob.phase)}</span>
                <span>{takeoutJob.message ?? takeoutJob.error ?? `${takeoutJob.inserted} inserted, ${takeoutJob.skipped} skipped`}</span>
              </div>
            {/if}

            {#if sourceJobs.length > 0}
              <div class="source-job-list">
                {#each sourceJobs.slice(0, 3) as job (job.job_id)}
                  <div class="source-job-row">
                    <span>{job.job_type.replaceAll("_", " ")}</span>
                    <Badge variant={job.status === "failed" ? "danger" : job.status === "succeeded" ? "success" : "info"}>
                      {job.status.replaceAll("_", " ")}
                    </Badge>
                    {#if isActiveSourceJob(job)}
                      <Button size="sm" variant="ghost" onclick={() => onCancelSourceJob(job.job_id)} disabled={job.status === "cancel_requested"}>
                        <Square size={13} aria-hidden="true" /> Cancel
                      </Button>
                    {/if}
                  </div>
                {/each}
              </div>
            {/if}
          </article>
        {/each}
      </div>
    {/if}
  </div>

  <div class="panel-section">
    <div class="section-title">
      <span>Groups</span>
      <Badge>{loadingGroups ? "loading" : `${filteredGroups.length}/${groups.length}`}</Badge>
    </div>

    {#if loadingGroups}
      <div class="panel-empty">Loading groups...</div>
    {:else if filteredGroups.length === 0}
      <div class="panel-empty">No groups match the current search.</div>
    {:else}
      <div class="group-list">
        {#each filteredGroups as group (group.id)}
          <button
            class:selected={isSelectedGroup(group.id)}
            class="group-row"
            type="button"
            aria-pressed={isSelectedGroup(group.id)}
            onclick={() => onSelectGroup(group.id)}
          >
            <span class="group-avatar" aria-hidden="true">{group.name.trim().charAt(0).toUpperCase() || "G"}</span>
            <span class="group-copy">
              <strong>{group.name}</strong>
              <small>{group.members.length} sources - updated {formatTimestamp(group.updated_at)}</small>
            </span>
          </button>
        {/each}
      </div>
    {/if}
  </div>
</section>

<style>
  .source-switcher-panel {
    position: absolute;
    z-index: 20;
    left: calc(100% + 0.55rem);
    top: 0;
    width: min(31rem, calc(100vw - 7rem));
    max-height: calc(100vh - 6rem);
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
    padding: 0.85rem;
    overflow: auto;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--panel);
    box-shadow: var(--shadow);
  }

  .panel-head,
  .section-title,
  .source-main,
  .row-actions,
  .detail-badges,
  .source-meta,
  .panel-actions,
  .source-job-row {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    flex-wrap: wrap;
  }

  .panel-head,
  .section-title {
    justify-content: space-between;
  }

  .eyebrow,
  .search-field span,
  .section-title span {
    color: var(--muted);
    font-size: 0.72rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  h2 {
    margin: 0.15rem 0 0;
    font-size: 1rem;
  }

  .search-field {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .search-shell {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr);
    gap: 0.4rem;
    align-items: center;
  }

  .panel-section,
  .source-list,
  .group-list,
  .source-job-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .source-row {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 0.65rem;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--panel-strong);
  }

  .source-row.selected,
  .group-row.selected {
    border-color: var(--primary);
    box-shadow: 0 0 0 2px color-mix(in srgb, var(--primary) 12%, transparent);
  }

  .source-main,
  .group-row {
    width: 100%;
    min-width: 0;
    border: 0;
    background: transparent;
    color: inherit;
    text-align: left;
    cursor: pointer;
  }

  .source-avatar,
  .group-avatar {
    width: 2.25rem;
    height: 2.25rem;
    flex: 0 0 2.25rem;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
    border-radius: 8px;
    background: color-mix(in srgb, var(--primary) 12%, var(--panel));
    color: var(--primary);
    font-weight: 700;
  }

  .source-avatar img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .source-copy,
  .group-copy {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
  }

  .source-copy strong,
  .group-copy strong {
    overflow-wrap: anywhere;
  }

  .source-meta,
  .group-copy small,
  .takeout-status,
  .source-job-row {
    color: var(--muted);
    font-size: 0.78rem;
  }

  .panel-link {
    display: inline-flex;
    gap: 0.25rem;
    align-items: center;
    color: var(--text);
    text-decoration: none;
    font-size: 0.78rem;
  }

  .takeout-status,
  .source-job-row,
  .panel-empty {
    padding: 0.55rem;
    border: 1px solid color-mix(in srgb, var(--border) 78%, transparent);
    border-radius: 8px;
    background: color-mix(in srgb, var(--panel-strong) 70%, transparent);
  }

  .group-row {
    display: flex;
    gap: 0.6rem;
    align-items: center;
    padding: 0.65rem;
    border: 1px solid var(--border);
    border-radius: 8px;
  }

  @media (max-width: 1180px) {
    .source-switcher-panel {
      position: fixed;
      left: 0.75rem;
      right: 0.75rem;
      top: 4.5rem;
      width: auto;
      max-height: calc(100vh - 5.5rem);
    }
  }
</style>
