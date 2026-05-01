<script lang="ts">
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import type { AccountRuntimeStatus } from "$lib/types/accounts";
  import type { AnalysisSourceGroup, AnalysisSourceOption } from "$lib/types/analysis";
  import type { SourceRecord } from "$lib/types/sources";

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
  }: {
    sourceCatalog: SourceRecord[];
    groups: AnalysisSourceGroup[];
    sourceMetrics: Record<number, AnalysisSourceOption>;
    loadingSourceCatalog: boolean;
    loadingGroups: boolean;
    railQuery: string;
    filteredSourceCatalog: SourceRecord[];
    filteredGroups: AnalysisSourceGroup[];
    analysisScope: "single_source" | "source_group";
    selectedSourceId: string;
    selectedGroupId: string;
    syncingIds: Record<number, boolean>;
    formatTimestamp: (value: number) => string;
    accountLabel: (accountId: number | null) => string;
    sourceKindLabel: (kind: string) => string;
    membershipLabel: (kind: string, isMember: boolean) => string;
    sourceInitial: (source: SourceRecord) => string;
    runtimeStatus: (accountId: number | null) => AccountRuntimeStatus | null;
    runtimeBadge: (runtime: AccountRuntimeStatus | null) => string;
    sourceSyncDisabledReason: (source: SourceRecord) => string | null;
    onChangeRailQuery: (value: string) => void;
    onSelectSource: (sourceId: number) => void;
    onSelectGroup: (groupId: number) => void;
    onSyncSource: (sourceId: number) => void;
  } = $props();
</script>

<aside class="rail">
  <div class="rail-header">
    <div>
      <span class="eyebrow">Research context</span>
      <h1>Workspace</h1>
    </div>
    <Badge variant="info">{sourceCatalog.length + groups.length} items</Badge>
  </div>

  <Input
    type="search"
    value={railQuery}
    placeholder="Search sources or groups"
    oninput={(event) => onChangeRailQuery((event.currentTarget as HTMLInputElement).value)}
    className="rail-search"
  />

  <div class="rail-section">
    <div class="rail-section-title">
      <span>Sources</span>
      <small>{filteredSourceCatalog.length}</small>
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
          {@const runtime = runtimeStatus(source.account_id)}
          {@const isSelected = analysisScope === "single_source" && selectedSourceId === String(source.id)}
          <article class:selected={isSelected} class="rail-row">
            <button class="rail-row-main" type="button" onclick={() => onSelectSource(source.id)}>
              <div class="rail-avatar" aria-hidden="true">
                {#if source.avatar_data_url}
                  <img src={source.avatar_data_url} alt="" loading="lazy" />
                {:else}
                  <span>{sourceInitial(source)}</span>
                {/if}
              </div>
              <div class="rail-copy">
                <div class="rail-copy-top">
                  <strong>{source.title ?? source.external_id}</strong>
                  {#if metrics?.last_synced_at}
                    <span>{formatTimestamp(metrics.last_synced_at)}</span>
                  {/if}
                </div>
                <div class="rail-copy-meta">
                  <span>{accountLabel(source.account_id)}</span>
                  <span>{sourceKindLabel(source.telegram_source_kind)}</span>
                  {#if metrics}
                    <span>{metrics.item_count} msgs</span>
                  {/if}
                </div>
              </div>
            </button>
            <div class="rail-row-actions">
              <Badge>{membershipLabel(source.telegram_source_kind, source.is_member)}</Badge>
              {#if runtimeBadge(runtime)}
                <Badge variant="warning" title={runtime?.message ?? undefined}>{runtimeBadge(runtime)}</Badge>
              {/if}
              <Button
                size="sm"
                variant="secondary"
                onclick={() => onSyncSource(source.id)}
                disabled={!!syncingIds[source.id] || syncReason !== null}
                title={syncReason ?? undefined}
              >
                {syncingIds[source.id] ? "Syncing..." : "Sync"}
              </Button>
            </div>
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

  .rail-section + .rail-section {
    padding-top: 0.85rem;
    border-top: 1px solid color-mix(in srgb, var(--border) 76%, transparent);
  }

  .rail-section-title {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 0.78rem;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  :global(.rail-search) {
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

  @media (max-width: 1180px) {
    .rail {
      position: static;
      max-height: none;
    }
  }
</style>
