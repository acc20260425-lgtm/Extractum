<script lang="ts">
  interface SourceRecord {
    id: number;
    account_id: number | null;
    external_id: string;
    title: string | null;
    last_sync_state: number | null;
    last_synced_at: number | null;
    is_member: boolean;
    is_active: boolean;
    created_at: number;
  }

  interface AccountRuntimeStatus {
    account_id: number;
    status: "not_initialized" | "restoring" | "ready" | "reauth_required" | "restore_failed";
    message: string | null;
  }

  let {
    source,
    selected,
    syncing,
    accountLabel,
    runtimeStatus,
    syncDisabledReason,
    formatDate,
    onSelect,
    onSync,
  }: {
    source: SourceRecord;
    selected: boolean;
    syncing: boolean;
    accountLabel: (id: number | null) => string;
    runtimeStatus: (accountId: number | null) => AccountRuntimeStatus | null;
    syncDisabledReason: (source: SourceRecord) => string | null;
    formatDate: (timestamp: number) => string;
    onSelect: (sourceId: number) => void | Promise<void>;
    onSync: (sourceId: number) => void | Promise<void>;
  } = $props();

  const syncReason = $derived(syncDisabledReason(source));
  const sourceRuntimeStatus = $derived(runtimeStatus(source.account_id));
  const runtimeBadgeLabel = $derived(
    !sourceRuntimeStatus
      ? null
      : sourceRuntimeStatus.status === "restoring"
        ? "restoring..."
        : sourceRuntimeStatus.status === "reauth_required"
          ? "sign in required"
          : sourceRuntimeStatus.status === "restore_failed"
            ? "restore failed"
            : sourceRuntimeStatus.status === "not_initialized"
              ? "account not connected"
              : null
  );
</script>

<li class:selected={selected}>
  <div class="channel-info">
    <button class="source-main" onclick={() => onSelect(source.id)}>
      <span class="title">{source.title ?? source.external_id}</span>
      <span class="sub">{accountLabel(source.account_id)}</span>
    </button>
  </div>
  <div class="channel-actions">
    {#if source.last_synced_at !== null}
      <span class="badge">synced {formatDate(source.last_synced_at)}</span>
    {/if}
    {#if source.account_id !== null}
      {#if runtimeBadgeLabel}
        <span
          class="badge warning"
          title={sourceRuntimeStatus?.status === "restore_failed" && sourceRuntimeStatus.message
            ? sourceRuntimeStatus.message
            : undefined}
        >
          {runtimeBadgeLabel}
        </span>
      {/if}
    {/if}
    {#if source.is_member}
      <span class="badge member">subscribed</span>
    {:else}
      <span class="badge">not subscribed</span>
    {/if}
    <button
      class="small"
      onclick={() => onSync(source.id)}
      disabled={syncing || syncReason !== null}
      title={syncReason ?? undefined}
    >
      {syncing ? "Syncing..." : "Sync"}
    </button>
    {#if syncReason}
      <p class="sync-reason">{syncReason}</p>
    {/if}
  </div>
</li>

<style>
  li {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.6rem 0.75rem;
    background: var(--panel-strong);
    border-radius: 8px;
    gap: 0.5rem;
    min-width: 0;
  }
  li.selected {
    outline: 1px solid color-mix(in srgb, var(--primary) 45%, transparent);
    background: color-mix(in srgb, var(--primary) 10%, var(--panel-strong));
  }
  .channel-info {
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
    min-width: 0;
    flex: 1 1 auto;
  }
  .channel-actions {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    flex: 0 1 auto;
    min-width: 0;
    flex-wrap: wrap;
    justify-content: flex-end;
  }
  .source-main {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 0.1rem;
    width: 100%;
    min-width: 0;
    padding: 0;
    border: 0;
    background: transparent;
    color: inherit;
    text-align: left;
    cursor: pointer;
  }
  .title {
    font-size: 0.95rem;
    max-width: 100%;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .sub { font-size: 0.75rem; color: var(--muted); }
  .badge {
    font-size: 0.7rem;
    padding: 0.15rem 0.5rem;
    border-radius: 4px;
    background: var(--panel-hover);
    color: var(--muted);
    max-width: 100%;
    white-space: normal;
    line-height: 1.2;
  }
  .badge.member {
    background: color-mix(in srgb, #22c55e 18%, var(--panel));
    color: #15803d;
  }
  .badge.warning {
    background: color-mix(in srgb, #f59e0b 22%, var(--panel));
    color: #b45309;
  }
  button.small { padding: 0.3rem 0.7rem; font-size: 0.8rem; }
  .sync-reason {
    margin: 0;
    width: 100%;
    font-size: 0.72rem;
    color: var(--muted);
    line-height: 1.3;
  }
  @media (max-width: 1200px) {
    li {
      align-items: flex-start;
    }
    .channel-actions {
      max-width: 42%;
    }
  }
  @media (max-width: 1024px) {
    li {
      flex-direction: column;
      align-items: stretch;
    }
    .channel-actions {
      max-width: 100%;
      justify-content: flex-start;
    }
    .title {
      white-space: normal;
      overflow: visible;
      text-overflow: clip;
      overflow-wrap: anywhere;
      word-break: break-word;
    }
  }
</style>
