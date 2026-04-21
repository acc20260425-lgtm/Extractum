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
    initialized: boolean;
    authenticated: boolean;
  }

  let {
    source,
    selected,
    syncing,
    accountLabel,
    runtimeStatus,
    syncDisabledReason,
    formatDate,
    onToggleMessages,
    onSync,
  }: {
    source: SourceRecord;
    selected: boolean;
    syncing: boolean;
    accountLabel: (id: number | null) => string;
    runtimeStatus: (accountId: number | null) => AccountRuntimeStatus | null;
    syncDisabledReason: (source: SourceRecord) => string | null;
    formatDate: (timestamp: number) => string;
    onToggleMessages: (sourceId: number) => void | Promise<void>;
    onSync: (sourceId: number) => void | Promise<void>;
  } = $props();

  const syncReason = $derived(syncDisabledReason(source));
  const sourceRuntimeStatus = $derived(runtimeStatus(source.account_id));
</script>

<li>
  <div class="channel-info">
    <span class="title">{source.title ?? source.external_id}</span>
    <span class="sub">{accountLabel(source.account_id)}</span>
  </div>
  <div class="channel-actions">
    {#if source.last_synced_at !== null}
      <span class="badge">synced {formatDate(source.last_synced_at)}</span>
    {/if}
    {#if source.account_id !== null}
      {#if !sourceRuntimeStatus?.initialized}
        <span class="badge warning">account not connected</span>
      {:else if !sourceRuntimeStatus?.authenticated}
        <span class="badge warning">sign in required</span>
      {/if}
    {/if}
    {#if source.is_member}
      <span class="badge member">subscribed</span>
    {:else}
      <span class="badge">not subscribed</span>
    {/if}
    <button class="secondary small" onclick={() => onToggleMessages(source.id)} disabled={syncing}>
      {selected ? "Hide messages" : "View messages"}
    </button>
    <button
      class="small"
      onclick={() => onSync(source.id)}
      disabled={syncing || syncReason !== null}
      title={syncReason ?? undefined}
    >
      {syncing ? "Syncing..." : "Sync"}
    </button>
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
  }
  .channel-info { display: flex; flex-direction: column; gap: 0.1rem; min-width: 0; }
  .channel-actions { display: flex; align-items: center; gap: 0.4rem; flex-shrink: 0; flex-wrap: wrap; justify-content: flex-end; }
  .title { font-size: 0.95rem; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .sub { font-size: 0.75rem; color: var(--muted); }
  .badge {
    font-size: 0.7rem;
    padding: 0.15rem 0.5rem;
    border-radius: 4px;
    background: var(--panel-hover);
    color: var(--muted);
    white-space: nowrap;
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
</style>
