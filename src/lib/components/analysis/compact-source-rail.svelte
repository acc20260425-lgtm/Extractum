<script lang="ts">
  import { AlertTriangle, Folder, Loader2, Plus, RefreshCw, Search, Send, Video } from "@lucide/svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import SourceSwitcherPanel from "$lib/components/analysis/source-switcher-panel.svelte";
  import { sourceCapabilities } from "$lib/source-capabilities";
  import type { AccountRuntimeStatus } from "$lib/types/accounts";
  import type { AnalysisSourceGroup, AnalysisSourceOption } from "$lib/types/analysis";
  import type {
    Source,
    SourceJobRecord,
    TakeoutImportJobRecord,
    TakeoutImportRecoveryState,
  } from "$lib/types/sources";
  import type { YoutubeRuntimeStatus, YoutubeSourceSummary } from "$lib/types/youtube";
  import type { WorkspaceSelection } from "$lib/analysis-workspace-state";

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
    startingMigratedHistorySourceIds,
    takeoutJobsBySource,
    takeoutRecoveryBySource,
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
    onStartMigratedHistoryImport,
    onCancelTakeoutImport,
    onCancelSourceJob,
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
    workspaceSelection: WorkspaceSelection;
    syncingIds: Record<number, boolean>;
    deletingSourceIds: Record<number, boolean>;
    startingTakeoutSourceIds: Record<number, boolean>;
    startingMigratedHistorySourceIds: Record<number, boolean>;
    takeoutJobsBySource: Record<number, TakeoutImportJobRecord>;
    takeoutRecoveryBySource: Record<number, TakeoutImportRecoveryState>;
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
    onStartMigratedHistoryImport: (sourceId: number) => void;
    onCancelTakeoutImport: (jobId: string) => void;
    onCancelSourceJob: (jobId: string) => void;
    onOpenSourceManager: () => void;
    onDeleteSource: (source: Source) => void;
  } = $props();

  let sourceSwitcherOpen = $state(false);

  const currentSource = $derived.by(() =>
    workspaceSelection.kind === "source"
      ? sourceCatalog.find((source) => source.id === workspaceSelection.sourceId) ?? null
      : null,
  );
  const currentGroup = $derived.by(() =>
    workspaceSelection.kind === "source_group"
      ? groups.find((group) => group.id === workspaceSelection.sourceGroupId) ?? null
      : null,
  );
  const visibleSources = $derived(filteredSourceCatalog.slice(0, 8));
  const visibleGroups = $derived(filteredGroups.slice(0, 4));
  const criticalStatusLabel = $derived(criticalSourceStatus());
  const currentContextLabel = $derived(
    currentSource
      ? (youtubeSummaries[currentSource.id]?.title ?? currentSource.title ?? currentSource.externalId)
      : currentGroup
        ? currentGroup.name
        : "Choose source",
  );

  function isSelectedSource(sourceId: number) {
    return workspaceSelection.kind === "source" && workspaceSelection.sourceId === sourceId;
  }

  function isSelectedGroup(groupId: number) {
    return workspaceSelection.kind === "source_group" && workspaceSelection.sourceGroupId === groupId;
  }

  function sourceButtonLabel(source: Source) {
    const name = youtubeSummaries[source.id]?.title ?? source.title ?? source.externalId;
    const status = compactSourceStatus(source);
    return status ? `${name}. ${status}` : name;
  }

  function groupButtonLabel(group: AnalysisSourceGroup) {
    return `${group.name}. ${group.members.length} sources`;
  }

  function compactSourceStatus(source: Source) {
    const runtime = runtimeBadge(runtimeStatus(source.accountId));
    if (runtime) return runtime;
    const syncReason = sourceSyncDisabledReason(source);
    if (syncReason) return syncReason;
    if (syncingIds[source.id]) return "Syncing";
    const activeJob = (sourceJobsBySource[source.id] ?? []).find(isActiveSourceJob);
    if (activeJob) return activeJob.status.replaceAll("_", " ");
    const takeoutJob = takeoutJobsBySource[source.id];
    if (takeoutJob && isActiveTakeoutJob(takeoutJob)) return takeoutJob.phase.replaceAll("_", " ");
    return "";
  }

  function criticalSourceStatus() {
    const source = currentSource;
    if (!source) return "";
    return compactSourceStatus(source);
  }

  function isActiveSourceJob(job: SourceJobRecord) {
    return job.status === "queued" || job.status === "running" || job.status === "cancel_requested";
  }

  function isActiveTakeoutJob(job: TakeoutImportJobRecord | undefined) {
    return (
      job?.status === "queued" ||
      job?.status === "running" ||
      job?.status === "cancel_requested"
    );
  }

  function canUsePrimarySync(source: Source | null) {
    if (!source) return false;
    return sourceCapabilities(source).canSync && sourceSyncDisabledReason(source) === null;
  }

  function providerMark(source: Source) {
    if (source.sourceType === "telegram") return Send;
    if (source.sourceType === "youtube") return Video;
    return Folder;
  }

  function selectSourceAndClose(sourceId: number) {
    onSelectSource(sourceId);
    sourceSwitcherOpen = false;
  }

  function selectGroupAndClose(groupId: number) {
    onSelectGroup(groupId);
    sourceSwitcherOpen = false;
  }
</script>

<aside class="compact-source-rail">
  <div class="rail-top">
    <Button
      iconOnly
      variant="secondary"
      ariaLabel="Open source switcher"
      title="Open source switcher"
      ariaExpanded={sourceSwitcherOpen}
      onclick={() => (sourceSwitcherOpen = !sourceSwitcherOpen)}
    >
      <Search size={16} aria-hidden="true" />
    </Button>

    <button
      class:active={workspaceSelection.kind !== "none"}
      class="current-context-button"
      type="button"
      title={currentSource ? sourceButtonLabel(currentSource) : currentGroup ? groupButtonLabel(currentGroup) : "No source selected"}
      aria-label={currentSource ? sourceButtonLabel(currentSource) : currentGroup ? groupButtonLabel(currentGroup) : "No source selected"}
      onclick={() => (sourceSwitcherOpen = true)}
    >
      {#if currentSource}
        {@const Mark = providerMark(currentSource)}
        <span class="context-avatar">
          {#if youtubeSummaries[currentSource.id]?.thumbnailUrl ?? currentSource.avatarDataUrl}
            <img src={youtubeSummaries[currentSource.id]?.thumbnailUrl ?? currentSource.avatarDataUrl ?? ""} alt="" loading="lazy" />
          {:else}
            {sourceInitial(currentSource)}
          {/if}
        </span>
        <Mark size={13} aria-hidden="true" />
      {:else if currentGroup}
        <span class="context-avatar group">{currentGroup.name.trim().charAt(0).toUpperCase() || "G"}</span>
        <Folder size={13} aria-hidden="true" />
      {:else}
        <span class="context-avatar empty">-</span>
      {/if}
    </button>

    <button
      class="mobile-current-label"
      type="button"
      onclick={() => (sourceSwitcherOpen = true)}
    >
      <span>{currentContextLabel}</span>
    </button>

    {#if criticalStatusLabel}
      <span class="status-dot" title={criticalStatusLabel} aria-label={criticalStatusLabel}>
        {#if criticalStatusLabel.toLocaleLowerCase().includes("sync") || criticalStatusLabel.toLocaleLowerCase().includes("running")}
          <Loader2 size={13} aria-hidden="true" />
        {:else}
          <AlertTriangle size={13} aria-hidden="true" />
        {/if}
      </span>
    {/if}
  </div>

  <div class="quick-list quick-list-scroll" aria-label="Quick source choices">
    {#each visibleSources as source (source.id)}
      {@const Mark = providerMark(source)}
      <Button
        iconOnly
        size="sm"
        variant="ghost"
        selected={isSelectedSource(source.id)}
        ariaLabel={sourceButtonLabel(source)}
        title={sourceButtonLabel(source)}
        ariaPressed={isSelectedSource(source.id)}
        onclick={() => onSelectSource(source.id)}
      >
        <span class="mini-avatar">
          {#if youtubeSummaries[source.id]?.thumbnailUrl ?? source.avatarDataUrl}
            <img src={youtubeSummaries[source.id]?.thumbnailUrl ?? source.avatarDataUrl ?? ""} alt="" loading="lazy" />
          {:else}
            {sourceInitial(source)}
          {/if}
        </span>
        <Mark size={10} aria-hidden="true" />
      </Button>
    {/each}

    {#each visibleGroups as group (group.id)}
      <Button
        iconOnly
        size="sm"
        variant="ghost"
        selected={isSelectedGroup(group.id)}
        ariaLabel={groupButtonLabel(group)}
        title={groupButtonLabel(group)}
        ariaPressed={isSelectedGroup(group.id)}
        onclick={() => onSelectGroup(group.id)}
      >
        <span class="mini-avatar group">{group.name.trim().charAt(0).toUpperCase() || "G"}</span>
      </Button>
    {/each}
  </div>

  <div class="context-primary-action">
    {#if sourceCatalog.length === 0 && groups.length === 0}
      <Button iconOnly size="sm" variant="primary" ariaLabel="New source" title="New source" onclick={onOpenSourceManager}>
        <Plus size={14} aria-hidden="true" />
      </Button>
    {:else if canUsePrimarySync(currentSource)}
      <Button
        iconOnly
        size="sm"
        variant="secondary"
        ariaLabel={`Sync ${currentSource?.title ?? currentSource?.externalId ?? "source"}`}
        title={`Sync ${currentSource?.title ?? currentSource?.externalId ?? "source"}`}
        disabled={currentSource ? !!syncingIds[currentSource.id] : true}
        onclick={() => currentSource && onSyncSource(currentSource.id)}
      >
        <RefreshCw size={14} aria-hidden="true" />
      </Button>
    {:else}
      <Button iconOnly size="sm" variant="secondary" ariaLabel="Open source switcher" title="Open source switcher" onclick={() => (sourceSwitcherOpen = true)}>
        <Search size={14} aria-hidden="true" />
      </Button>
    {/if}
  </div>

  {#if sourceSwitcherOpen}
    <SourceSwitcherPanel
      {sourceCatalog}
      {groups}
      {sourceMetrics}
      {loadingSourceCatalog}
      {loadingGroups}
      {railQuery}
      {filteredSourceCatalog}
      {filteredGroups}
      {workspaceSelection}
      {syncingIds}
      {deletingSourceIds}
      {startingTakeoutSourceIds}
      {startingMigratedHistorySourceIds}
      {takeoutJobsBySource}
      {takeoutRecoveryBySource}
      {sourceJobsBySource}
      {youtubeSummaries}
      {youtubeRuntimeStatus}
      {formatTimestamp}
      {accountLabel}
      {sourceInitial}
      {runtimeStatus}
      {runtimeBadge}
      {sourceSyncDisabledReason}
      {onChangeRailQuery}
      onSelectSource={selectSourceAndClose}
      onSelectGroup={selectGroupAndClose}
      {onSyncSource}
      {onStartTakeoutImport}
      {onStartMigratedHistoryImport}
      {onCancelTakeoutImport}
      {onCancelSourceJob}
      {onOpenSourceManager}
      {onDeleteSource}
      onClose={() => (sourceSwitcherOpen = false)}
    />
  {/if}
</aside>

<style>
  .compact-source-rail {
    position: sticky;
    top: 0;
    z-index: 30;
    width: 100%;
    min-width: 0;
    max-height: calc(100vh - 6rem);
    display: flex;
    flex-direction: column;
    gap: 0.55rem;
    align-items: center;
    padding: 0.35rem;
    border: 1px solid color-mix(in srgb, var(--border) 38%, transparent);
    border-radius: 8px;
    background: var(--panel);
    box-shadow: var(--shadow);
  }

  .rail-top,
  .quick-list,
  .context-primary-action {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
    align-items: center;
  }

  .current-context-button {
    width: 2.75rem;
    min-height: 3.2rem;
    display: inline-flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 0.2rem;
    border: 0;
    border-radius: 8px;
    background: var(--panel-strong);
    color: var(--text);
    cursor: pointer;
  }

  .current-context-button.active {
    box-shadow: 0 0 0 1px color-mix(in srgb, var(--primary) 44%, transparent);
  }

  .mobile-current-label {
    display: none;
    min-width: 0;
    border: 0;
    background: transparent;
    color: var(--text);
    text-align: left;
    cursor: pointer;
  }

  .mobile-current-label span {
    display: block;
    max-width: 14rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .context-avatar {
    width: 1.75rem;
    height: 1.75rem;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
    border-radius: 7px;
    background: color-mix(in srgb, var(--primary) 12%, var(--panel));
    color: var(--primary);
    font-size: 0.75rem;
    font-weight: 700;
  }

  .mini-avatar {
    width: 2.5rem;
    height: 2.5rem;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
    border-radius: 7px;
    background: color-mix(in srgb, var(--primary) 12%, var(--panel));
    color: var(--primary);
    font-size: 0.75rem;
    font-weight: 700;
  }

  .context-avatar.group,
  .mini-avatar.group {
    background: color-mix(in srgb, var(--accent) 12%, var(--panel));
  }

  .context-avatar.empty {
    color: var(--muted);
  }

  .context-avatar img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .mini-avatar img {
    width: 100%;
    height: 100%;
    object-fit: contain;
    background: var(--panel);
  }

  .quick-list {
    width: 100%;
    padding-top: 0.45rem;
    border-top: 1px solid color-mix(in srgb, var(--border) 76%, transparent);
  }

  .quick-list :global(.ui-button.icon-only.sm) {
    width: 3.75rem;
    height: 3rem;
  }

  .status-dot {
    width: 1.55rem;
    height: 1.55rem;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border-radius: 999px;
    color: var(--warning);
    background: color-mix(in srgb, var(--warning) 14%, var(--panel));
  }

  @media (max-width: 1180px) {
    .compact-source-rail {
      position: relative;
      top: auto;
      max-height: none;
      flex-direction: row;
      justify-content: flex-start;
      overflow-x: auto;
    }

    .rail-top,
    .quick-list,
    .context-primary-action {
      flex-direction: row;
    }

    .quick-list {
      width: auto;
      padding-top: 0;
      padding-left: 0.45rem;
      border-top: 0;
      border-left: 1px solid color-mix(in srgb, var(--border) 76%, transparent);
    }
  }

  @media (max-width: 720px) {
    .compact-source-rail {
      padding: 0.45rem;
      gap: 0.4rem;
    }

    .rail-top {
      min-width: 0;
      flex: 0 0 auto;
    }

    .current-context-button {
      width: 2.35rem;
      min-height: 2.35rem;
    }

    .current-context-button :global(svg) {
      display: none;
    }

    .mobile-current-label {
      display: inline-flex;
    }

    .quick-list-scroll {
      flex: 1 1 auto;
      min-width: 0;
      overflow-x: auto;
      scrollbar-width: thin;
    }
  }
</style>
