<script lang="ts">
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import type { AccountRuntimeStatus } from "$lib/types/accounts";
  import type { AnalysisSourceGroup, AnalysisSourceOption } from "$lib/types/analysis";
  import type { Source, TakeoutImportJobRecord } from "$lib/types/sources";
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
    analysisScope,
    selectedSourceId,
    selectedGroupId,
    syncingIds,
    deletingSourceIds,
    startingTakeoutSourceIds,
    takeoutJobsBySource,
    formatTimestamp,
    accountLabel,
    sourceKindLabel,
    membershipLabel,
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
    onOpenSourceManager,
    onDeleteSource,
  }: {
    sourceCatalog: Source[];
    groups: AnalysisSourceGroup[];
    sourceMetrics: Record<number, AnalysisSourceOption>;
    loadingSourceCatalog: boolean;
    loadingGroups: boolean;
    railQuery: string;
    filteredSourceCatalog: Source[];
    filteredGroups: AnalysisSourceGroup[];
    analysisScope: "single_source" | "source_group";
    selectedSourceId: string;
    selectedGroupId: string;
    syncingIds: Record<number, boolean>;
    deletingSourceIds: Record<number, boolean>;
    startingTakeoutSourceIds: Record<number, boolean>;
    takeoutJobsBySource: Record<number, TakeoutImportJobRecord>;
    formatTimestamp: (value: number) => string;
    accountLabel: (accountId: number | null) => string;
    sourceKindLabel: (kind: string) => string;
    membershipLabel: (kind: string, isMember: boolean) => string;
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
    onOpenSourceManager: () => void;
    onDeleteSource: (source: Source) => void;
  } = $props();

  const totalItems = $derived(sourceCatalog.length + groups.length);
  const visibleItems = $derived(filteredSourceCatalog.length + filteredGroups.length);

  function isActiveTakeoutJob(job: TakeoutImportJobRecord | undefined) {
    return (
      job?.status === "queued" ||
      job?.status === "running" ||
      job?.status === "cancel_requested"
    );
  }

  function takeoutPhaseLabel(job: TakeoutImportJobRecord) {
    if (job.status === "failed") return "Takeout failed";
    if (job.status === "completed") return "Takeout complete";
    if (job.status === "cancelled") return "Takeout cancelled";
    if (job.status === "cancel_requested") return "Cancelling Takeout";
    return `Takeout ${takeoutPhaseName(job.phase)}`;
  }

  function takeoutPhaseName(phase: TakeoutImportJobRecord["phase"]) {
    switch (phase) {
      case "queued":
        return "queued";
      case "resolving_source":
        return "resolving source";
      case "starting_takeout":
        return "starting session";
      case "validating_peer":
        return "validating peer";
      case "loading_splits":
        return "loading ranges";
      case "counting":
        return "counting history";
      case "importing_history":
        return "importing history";
      case "finishing_takeout":
        return "finishing session";
      case "completed":
        return "complete";
      case "failed":
        return "failed";
      case "cancelled":
        return "cancelled";
      default:
        return String(phase).replaceAll("_", " ");
    }
  }

  function takeoutBadgeVariant(job: TakeoutImportJobRecord): BadgeVariant {
    if (job.status === "completed") return "success";
    if (job.status === "failed") return "danger";
    if (job.status === "cancelled") return "neutral";
    if (job.warnings.length > 0) return "warning";
    return "info";
  }

  function takeoutProgressLabel(job: TakeoutImportJobRecord) {
    if (job.progress_current !== null && job.progress_total !== null) {
      return `${job.progress_current}/${job.progress_total}`;
    }
    if (job.status === "completed") {
      return "done";
    }
    if (job.status === "failed" || job.status === "cancelled") {
      return job.status;
    }
    return "pending";
  }

  function takeoutProgressValue(job: TakeoutImportJobRecord) {
    if (
      job.progress_current === null ||
      job.progress_total === null ||
      job.progress_total <= 0
    ) {
      return null;
    }
    return Math.min(100, Math.max(0, Math.round((job.progress_current / job.progress_total) * 100)));
  }

  function takeoutSummary(job: TakeoutImportJobRecord) {
    return `${job.inserted} inserted, ${job.skipped} skipped`;
  }
</script>

<aside class="rail">
  <div class="rail-header">
    <div>
      <span class="eyebrow">Research context</span>
      <h1>Workspace</h1>
    </div>
    <Badge variant="info">
      {railQuery.trim() ? `${visibleItems} visible` : `${totalItems} items`}
    </Badge>
  </div>

  <div class="rail-search-shell">
    <Input
      type="search"
      value={railQuery}
      placeholder="Search sources or groups"
      ariaLabel="Search sources or groups"
      oninput={(event) => onChangeRailQuery((event.currentTarget as HTMLInputElement).value)}
      className="rail-search"
    />
  </div>

  <div class="rail-section">
    <div class="rail-section-heading">
      <div class="rail-section-title">
        <span>Sources</span>
        <small>{filteredSourceCatalog.length}</small>
      </div>
      <Button size="sm" variant="secondary" onclick={onOpenSourceManager}>Add</Button>
    </div>
    <div class="rail-list">
      {#if loadingSourceCatalog}
        <div class="rail-empty">Loading sources...</div>
      {:else if filteredSourceCatalog.length === 0}
        <div class="rail-empty">No sources match the current search.</div>
      {:else}
        {#each filteredSourceCatalog as source (source.id)}
          {@const metrics = sourceMetrics[source.id]}
          {@const syncReason = sourceSyncDisabledReason(source)}
          {@const runtime = runtimeStatus(source.accountId)}
          {@const runtimeStateBadge = runtimeBadge(runtime)}
          {@const isSelected = analysisScope === "single_source" && selectedSourceId === String(source.id)}
          {@const deleting = !!deletingSourceIds[source.id]}
          {@const takeoutJob = takeoutJobsBySource[source.id]}
          {@const takeoutActive = isActiveTakeoutJob(takeoutJob)}
          {@const startingTakeout = !!startingTakeoutSourceIds[source.id]}
          <article class:selected={isSelected} class="rail-row">
            <button
              class="rail-row-main"
              type="button"
              aria-pressed={isSelected}
              onclick={() => onSelectSource(source.id)}
            >
              <div class="rail-avatar" aria-hidden="true">
                {#if source.avatarDataUrl}
                  <img src={source.avatarDataUrl} alt="" loading="lazy" />
                {:else}
                  <span>{sourceInitial(source)}</span>
                {/if}
              </div>
              <div class="rail-copy">
                <div class="rail-copy-top">
                  <strong>{source.title ?? source.externalId}</strong>
                  {#if metrics?.last_synced_at}
                    <span>{formatTimestamp(metrics.last_synced_at)}</span>
                  {/if}
                </div>
                <div class="rail-copy-meta">
                  <span>{accountLabel(source.accountId)}</span>
                  <span>{sourceKindLabel(source.telegramSourceKind)}</span>
                  {#if metrics}
                    <span>{metrics.item_count} msgs</span>
                  {/if}
                </div>
              </div>
            </button>
            <div class="rail-row-actions">
              <Badge>{membershipLabel(source.telegramSourceKind, source.isMember)}</Badge>
              {#if runtimeStateBadge}
                <Badge variant="warning" title={runtime?.message ?? undefined}>{runtimeStateBadge}</Badge>
              {/if}
              {#if takeoutJob}
                <Badge variant={takeoutBadgeVariant(takeoutJob)} title={takeoutJob.error ?? takeoutJob.message ?? undefined}>
                  {takeoutPhaseLabel(takeoutJob)}
                </Badge>
              {/if}
              <Button
                size="sm"
                variant="secondary"
                onclick={() => onSyncSource(source.id)}
                disabled={!!syncingIds[source.id] || deleting || takeoutActive || syncReason !== null}
                title={takeoutActive ? "Takeout import is active." : syncReason ?? undefined}
              >
                {syncingIds[source.id] ? "Syncing..." : "Sync"}
              </Button>
              {#if takeoutActive && takeoutJob}
                <Button
                  size="sm"
                  variant="secondary"
                  onclick={() => onCancelTakeoutImport(takeoutJob.job_id)}
                  disabled={takeoutJob.status === "cancel_requested"}
                >
                  {takeoutJob.status === "cancel_requested" ? "Cancelling..." : "Cancel"}
                </Button>
              {:else}
                <Button
                  size="sm"
                  variant="secondary"
                  onclick={() => onStartTakeoutImport(source.id)}
                  disabled={startingTakeout || deleting || !!syncingIds[source.id] || syncReason !== null}
                  title={syncReason ?? undefined}
                >
                  {startingTakeout ? "Starting..." : "Takeout"}
                </Button>
              {/if}
              <Button
                size="sm"
                variant="danger-soft"
                onclick={() => onDeleteSource(source)}
                disabled={deleting || !!syncingIds[source.id] || takeoutActive}
                title={takeoutActive ? "Takeout import is active." : undefined}
              >
                {deleting ? "Deleting..." : "Delete"}
              </Button>
            </div>
            {#if takeoutJob}
              {@const progressValue = takeoutProgressValue(takeoutJob)}
              <div class="takeout-status" class:terminal={!takeoutActive}>
                <div class="takeout-status-line">
                  <span>{takeoutPhaseName(takeoutJob.phase)}</span>
                  <span>{takeoutProgressLabel(takeoutJob)}</span>
                </div>
                {#if progressValue !== null}
                  <progress max="100" value={progressValue}>{progressValue}%</progress>
                {:else if takeoutActive}
                  <progress></progress>
                {/if}
                <div class="takeout-status-meta">
                  <span>{takeoutSummary(takeoutJob)}</span>
                  {#if takeoutJob.message}
                    <span>{takeoutJob.message}</span>
                  {/if}
                </div>
                {#if takeoutJob.error}
                  <div class="takeout-issue error">{takeoutJob.error}</div>
                {/if}
                {#if takeoutJob.warnings.length > 0}
                  <div class="takeout-issue">
                    {takeoutJob.warnings.length === 1
                      ? takeoutJob.warnings[0]
                      : `${takeoutJob.warnings.length} warnings. First: ${takeoutJob.warnings[0]}`}
                  </div>
                {/if}
              </div>
            {/if}
          </article>
        {/each}
      {/if}
    </div>
  </div>

  <div class="rail-section">
    <div class="rail-section-title">
      <span>Groups</span>
      <small>{filteredGroups.length}</small>
    </div>
    <div class="rail-list">
      {#if loadingGroups}
        <div class="rail-empty">Loading groups...</div>
      {:else if filteredGroups.length === 0}
        <div class="rail-empty">No groups match the current search.</div>
      {:else}
        {#each filteredGroups as group (group.id)}
          <button
            class:selected={analysisScope === "source_group" && selectedGroupId === String(group.id)}
            class="rail-row rail-group-row"
            type="button"
            aria-pressed={analysisScope === "source_group" && selectedGroupId === String(group.id)}
            onclick={() => onSelectGroup(group.id)}
          >
            <div class="rail-avatar group-avatar" aria-hidden="true">
              <span>{group.name.trim().charAt(0).toUpperCase() || "G"}</span>
            </div>
            <div class="rail-copy">
              <div class="rail-copy-top">
                <strong>{group.name}</strong>
                <span>{group.members.length} src</span>
              </div>
              <div class="rail-copy-meta">
                <span>Saved source group</span>
                <span>Updated {formatTimestamp(group.updated_at)}</span>
              </div>
            </div>
          </button>
        {/each}
      {/if}
    </div>
  </div>
</aside>

<style>
  .rail {
    position: sticky;
    top: 0;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 0.8rem;
    padding: 0.85rem;
    background:
      linear-gradient(180deg, color-mix(in srgb, var(--panel) 96%, white 4%), var(--panel));
    border: 1px solid var(--border);
    border-radius: 16px;
    box-shadow: var(--shadow);
    max-height: calc(100vh - 6rem);
    overflow: auto;
  }

  .rail-header {
    display: flex;
    justify-content: space-between;
    gap: 0.75rem;
    align-items: flex-start;
    background: var(--panel);
    border: 1px solid var(--border);
    box-shadow: var(--shadow);
    border-radius: 16px;
    padding: 1rem;
  }

  .eyebrow {
    display: inline-block;
    font-size: 0.68rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--muted);
    margin-bottom: 0.2rem;
  }

  .rail-header h1 {
    margin: 0;
  }

  .rail-section {
    display: flex;
    flex-direction: column;
    gap: 0.55rem;
    padding-top: 0.1rem;
  }

  .rail-section-heading {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.6rem;
  }

  .rail-section + .rail-section {
    padding-top: 0.85rem;
    border-top: 1px solid color-mix(in srgb, var(--border) 76%, transparent);
  }

  .rail-section-title {
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex: 1;
    min-width: 0;
    font-size: 0.78rem;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  .rail-search-shell :global(.rail-search) {
    min-height: 2.5rem;
  }

  .rail-list {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .rail-empty {
    padding: 0.85rem 0.95rem;
    border: 1px dashed var(--border);
    border-radius: 12px;
    color: var(--muted);
    font-size: 0.86rem;
    background: color-mix(in srgb, var(--panel-strong) 72%, transparent);
  }

  .rail-row {
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
    width: 100%;
    padding: 0.6rem;
    border-radius: 14px;
    border: 1px solid transparent;
    background: var(--panel-strong);
    transition: background 0.2s, border-color 0.2s, transform 0.2s, box-shadow 0.2s;
  }

  .rail-group-row {
    flex-direction: row;
    align-items: center;
    text-align: left;
    cursor: pointer;
  }

  .rail-row.selected,
  .rail-group-row.selected {
    border-color: color-mix(in srgb, var(--primary) 45%, transparent);
    background: color-mix(in srgb, var(--primary) 8%, var(--panel-strong));
    box-shadow: 0 0 0 2px color-mix(in srgb, var(--primary) 10%, transparent);
  }

  .rail-row:hover,
  .rail-group-row:hover {
    border-color: color-mix(in srgb, var(--border-strong) 68%, transparent);
    background: color-mix(in srgb, var(--panel-hover) 78%, var(--panel-strong));
    transform: translateY(-1px);
  }

  .rail-row-main {
    display: flex;
    gap: 0.6rem;
    align-items: flex-start;
    width: 100%;
    background: transparent;
    border: 0;
    color: inherit;
    padding: 0;
    text-align: left;
    cursor: pointer;
  }

  .rail-avatar {
    flex: 0 0 2.35rem;
    width: 2.35rem;
    height: 2.35rem;
    border-radius: 0.9rem;
    overflow: hidden;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: color-mix(in srgb, var(--primary) 12%, var(--panel));
    color: var(--primary);
    font-size: 0.9rem;
    font-weight: 700;
  }

  .group-avatar {
    border-radius: 0.8rem;
  }

  .rail-avatar img {
    display: block;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .rail-copy {
    min-width: 0;
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.16rem;
  }

  .rail-copy-top,
  .rail-copy-meta,
  .rail-row-actions {
    display: flex;
    gap: 0.45rem;
    flex-wrap: wrap;
    align-items: center;
  }

  .rail-copy-top {
    justify-content: space-between;
  }

  .rail-copy-top strong {
    font-size: 0.92rem;
    line-height: 1.25;
  }

  .rail-copy-top span,
  .rail-copy-meta span {
    font-size: 0.75rem;
    color: var(--muted);
  }

  .takeout-status {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    padding: 0.55rem;
    border-radius: 8px;
    border: 1px solid color-mix(in srgb, var(--primary) 20%, transparent);
    background: color-mix(in srgb, var(--primary) 7%, var(--panel));
  }

  .takeout-status.terminal {
    border-color: color-mix(in srgb, var(--border) 84%, transparent);
    background: color-mix(in srgb, var(--panel-hover) 60%, var(--panel));
  }

  .takeout-status-line,
  .takeout-status-meta {
    display: flex;
    justify-content: space-between;
    gap: 0.45rem;
    min-width: 0;
  }

  .takeout-status-line span {
    font-size: 0.76rem;
    font-weight: 700;
    color: var(--text);
    text-transform: capitalize;
  }

  .takeout-status-meta {
    flex-wrap: wrap;
  }

  .takeout-status-meta span,
  .takeout-issue {
    font-size: 0.72rem;
    line-height: 1.35;
    color: var(--muted);
  }

  .takeout-status progress {
    width: 100%;
    height: 0.45rem;
    accent-color: var(--primary);
  }

  .takeout-issue {
    padding-top: 0.2rem;
    border-top: 1px solid color-mix(in srgb, var(--border) 72%, transparent);
    color: #b45309;
  }

  .takeout-issue.error {
    color: var(--danger);
  }

  @media (max-width: 1180px) {
    .rail {
      position: static;
      max-height: none;
    }
  }
</style>
