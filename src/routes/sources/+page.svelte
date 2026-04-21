<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { page } from "$app/stores";
  import SourceMessagesPanel from "$lib/components/source-messages-panel.svelte";

  interface AccountRecord {
    id: number;
    label: string;
    phone: string | null;
  }

  interface ChannelInfo {
    id: number;
    title: string;
    username: string | null;
    is_member: boolean;
  }

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

  interface ItemRecord {
    id: number;
    source_id: number;
    external_id: string;
    author: string | null;
    published_at: number;
    content: string;
    has_raw_data: boolean;
  }

  interface SyncResult {
    inserted: number;
    skipped: number;
    last_message_id: number | null;
  }

  interface AccountRuntimeStatus {
    account_id: number;
    initialized: boolean;
    authenticated: boolean;
  }

  let selectedAccountId = $state<number | null>(
    $page.url.searchParams.has("account")
      ? parseInt($page.url.searchParams.get("account")!, 10)
      : null
  );

  let accounts = $state<AccountRecord[]>([]);
  let sources = $state<SourceRecord[]>([]);
  let dialogs = $state<ChannelInfo[]>([]);
  let items = $state<ItemRecord[]>([]);
  let manualRef = $state("");
  let status = $state("");
  let accountStatuses = $state<Record<number, AccountRuntimeStatus>>({});
  let loadingDialogs = $state(false);
  let loadingItems = $state(false);
  let addingId = $state<number | string | null>(null);
  let selectedSourceId = $state<number | null>(null);
  let syncingIds = $state<Record<number, boolean>>({});

  async function loadAccounts() {
    try {
      accounts = await invoke<AccountRecord[]>("list_accounts");
      await loadAccountStatuses();
    } catch (e) {
      console.error(e);
    }
  }

  async function loadAccountStatuses() {
    if (accounts.length === 0) {
      accountStatuses = {};
      return;
    }

    try {
      const statuses = await invoke<AccountRuntimeStatus[]>("tg_get_account_statuses", {
        accountIds: accounts.map((account) => account.id),
      });
      accountStatuses = Object.fromEntries(
        statuses.map((runtimeStatus) => [runtimeStatus.account_id, runtimeStatus])
      );
    } catch (e) {
      console.error(e);
      accountStatuses = {};
    }
  }

  async function loadSources() {
    try {
      sources = await invoke<SourceRecord[]>("list_sources", {
        accountId: selectedAccountId,
      });
    } catch (e) {
      status = `Error loading sources: ${e}`;
    }
  }

  async function loadDialogs() {
    if (selectedAccountId === null) {
      status = "Select an account first";
      return;
    }
    if (!selectedAccountReady()) {
      status = "Initialize and sign in this account before loading Telegram channels.";
      return;
    }
    loadingDialogs = true;
    status = "";
    try {
      dialogs = await invoke<ChannelInfo[]>("list_telegram_channels", {
        accountId: selectedAccountId,
      });
    } catch (e) {
      status = `Error: ${e}`;
    } finally {
      loadingDialogs = false;
    }
  }

  async function loadItems(sourceId: number) {
    loadingItems = true;
    status = "";
    try {
      items = await invoke<ItemRecord[]>("get_items", {
        sourceId,
        limit: 50,
        beforePublishedAt: null,
      });
    } catch (e) {
      items = [];
      status = `Error loading messages: ${e}`;
    } finally {
      loadingItems = false;
    }
  }

  async function addFromDialog(channel: ChannelInfo) {
    if (selectedAccountId === null) return;
    if (!selectedAccountReady()) {
      status = "Initialize and sign in this account before adding sources.";
      return;
    }
    addingId = channel.id;
    try {
      const ref = channel.username ? `@${channel.username}` : String(channel.id);
      await invoke("add_telegram_source", { accountId: selectedAccountId, channelRef: ref });
      await loadSources();
    } catch (e) {
      status = `Error: ${e}`;
    } finally {
      addingId = null;
    }
  }

  async function addManual() {
    if (!manualRef.trim() || selectedAccountId === null) return;
    if (!selectedAccountReady()) {
      status = "Initialize and sign in this account before adding sources.";
      return;
    }
    addingId = "manual";
    status = "";
    try {
      await invoke("add_telegram_source", {
        accountId: selectedAccountId,
        channelRef: manualRef.trim(),
      });
      manualRef = "";
      await loadSources();
    } catch (e) {
      status = `Error: ${e}`;
    } finally {
      addingId = null;
    }
  }

  async function syncSource(sourceId: number) {
    syncingIds = { ...syncingIds, [sourceId]: true };
    status = "";
    try {
      const result = await invoke<SyncResult>("sync_channel", { sourceId });
      await loadSources();
      status =
        `Sync complete: inserted ${result.inserted}, skipped ${result.skipped}` +
        (result.last_message_id ? `, last message ${result.last_message_id}.` : ".");
      if (selectedSourceId === sourceId) {
        await loadItems(sourceId);
      }
    } catch (e) {
      status = `Error: ${e}`;
    } finally {
      const next = { ...syncingIds };
      delete next[sourceId];
      syncingIds = next;
    }
  }

  async function toggleMessages(sourceId: number) {
    if (selectedSourceId === sourceId) {
      selectedSourceId = null;
      items = [];
      return;
    }

    selectedSourceId = sourceId;
    await loadItems(sourceId);
  }

  function isAlreadyAdded(channel: ChannelInfo) {
    return sources.some((source) => source.external_id === String(channel.id));
  }

  function runtimeStatus(accountId: number | null) {
    if (accountId === null) return null;
    return accountStatuses[accountId] ?? null;
  }

  function syncDisabledReason(source: SourceRecord) {
    const runtime = runtimeStatus(source.account_id);
    if (source.account_id === null) return "Source is not linked to an account.";
    if (!runtime?.initialized) return "Initialize this account in Telegram before syncing.";
    if (!runtime.authenticated) return "Sign in to this account again before syncing.";
    return null;
  }

  function selectedAccountReady() {
    if (selectedAccountId === null) return false;
    const runtime = runtimeStatus(selectedAccountId);
    return Boolean(runtime?.initialized && runtime.authenticated);
  }

  function accountLabel(id: number | null) {
    if (id === null) return "—";
    return accounts.find((account) => account.id === id)?.label ?? `#${id}`;
  }

  function formatDate(timestamp: number) {
    return new Date(timestamp * 1000).toLocaleString();
  }

  $effect(() => {
    loadSources();
    dialogs = [];
  });

  $effect(() => {
    if (selectedSourceId !== null && !sources.some((source) => source.id === selectedSourceId)) {
      selectedSourceId = null;
      items = [];
    }
  });

  onMount(loadAccounts);
</script>

<h1>Sources</h1>

{#if status}
  <p class="status" class:error={status.startsWith("Error")}>{status}</p>
{/if}

<div class="card">
  <h3>Account</h3>
  <div class="row">
    <select bind:value={selectedAccountId}>
      <option value={null}>All accounts</option>
      {#each accounts as acc}
        <option value={acc.id}>{acc.label}{acc.phone ? ` (${acc.phone})` : " (not signed in)"}</option>
      {/each}
    </select>
    {#if accounts.length === 0}
      <a href="/accounts" class="btn-link">Add account</a>
    {/if}
  </div>
</div>

<div class="card">
  <div class="card-header">
    <h3>Added Sources ({sources.length})</h3>
  </div>
  {#if sources.length === 0}
    <p class="empty">No sources yet.</p>
  {:else}
    <ul class="source-list">
      {#each sources as src}
        {@const syncReason = syncDisabledReason(src)}
        <li>
          <div class="channel-info">
            <span class="title">{src.title ?? src.external_id}</span>
            <span class="sub">{accountLabel(src.account_id)}</span>
          </div>
          <div class="channel-actions">
            {#if src.last_synced_at !== null}
              <span class="badge">synced {formatDate(src.last_synced_at)}</span>
            {/if}
            {#if src.account_id !== null}
              {#if !runtimeStatus(src.account_id)?.initialized}
                <span class="badge warning">account not connected</span>
              {:else if !runtimeStatus(src.account_id)?.authenticated}
                <span class="badge warning">sign in required</span>
              {/if}
            {/if}
            {#if src.is_member}
              <span class="badge member">subscribed</span>
            {:else}
              <span class="badge">not subscribed</span>
            {/if}
            <button class="secondary small" onclick={() => toggleMessages(src.id)} disabled={!!syncingIds[src.id]}>
              {selectedSourceId === src.id ? "Hide messages" : "View messages"}
            </button>
            <button
              class="small"
              onclick={() => syncSource(src.id)}
              disabled={!!syncingIds[src.id] || syncReason !== null}
              title={syncReason ?? undefined}
            >
              {syncingIds[src.id] ? "Syncing..." : "Sync"}
            </button>
          </div>
        </li>
      {/each}
    </ul>
  {/if}
</div>

{#if selectedSourceId !== null}
  <SourceMessagesPanel {loadingItems} {items} {formatDate} />
{/if}

{#if selectedAccountId !== null}
  <div class="card">
    <h3>Add by Username or Link</h3>
    <div class="row">
      <input
        type="text"
        bind:value={manualRef}
        placeholder="@channel or https://t.me/channel"
        onkeydown={(e) => e.key === "Enter" && addManual()}
      />
      <button onclick={addManual} disabled={addingId === "manual" || !manualRef.trim() || !selectedAccountReady()}>
        {addingId === "manual" ? "Adding..." : "Add"}
      </button>
    </div>
    {#if !selectedAccountReady()}
      <p class="empty">Initialize and sign in the selected account before adding sources.</p>
    {/if}
  </div>

  <div class="card">
    <div class="card-header">
      <h3>My Channels</h3>
      <button class="secondary small" onclick={loadDialogs} disabled={loadingDialogs || !selectedAccountReady()}>
        {loadingDialogs ? "Loading..." : dialogs.length ? "Refresh" : "Load"}
      </button>
    </div>

    {#if dialogs.length > 0}
      <ul class="source-list">
        {#each dialogs as ch}
          {@const added = isAlreadyAdded(ch)}
          <li>
            <div class="channel-info">
              <span class="title">{ch.title}</span>
              {#if ch.username}<span class="sub">@{ch.username}</span>{/if}
            </div>
            <div class="channel-actions">
              {#if !ch.is_member}<span class="badge">not subscribed</span>{/if}
              {#if added}
                <span class="badge active">added</span>
              {:else}
                <button class="small" onclick={() => addFromDialog(ch)} disabled={addingId === ch.id}>
                  {addingId === ch.id ? "..." : "Add"}
                </button>
              {/if}
            </div>
          </li>
        {/each}
      </ul>
    {:else if !loadingDialogs}
      <p class="empty">Click "Load" to see your Telegram channels.</p>
    {/if}
  </div>
{/if}

<style>
  .card {
    background: var(--panel);
    border: 1px solid var(--border);
    box-shadow: var(--shadow);
    border-radius: 12px;
    padding: 1.5rem;
    margin-bottom: 1.5rem;
  }
  .card-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem; }
  .card-header h3 { margin: 0; }
  .row { display: flex; gap: 0.5rem; align-items: center; }
  .row input { flex: 1; }
  select {
    flex: 1;
    padding: 0.6rem 0.8rem;
    border-radius: 6px;
    font-size: 0.95rem;
  }
  .source-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  .source-list li {
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
  .badge.member, .badge.active { background: color-mix(in srgb, #22c55e 18%, var(--panel)); color: #15803d; }
  .badge.warning {
    background: color-mix(in srgb, #f59e0b 22%, var(--panel));
    color: #b45309;
  }
  .empty { color: var(--muted); font-size: 0.9rem; margin: 0; }
  .status { padding: 0.6rem 1rem; border-radius: 6px; background: var(--status-bg); font-size: 0.9rem; margin-bottom: 1rem; }
  .status.error { background: var(--status-error-bg); color: var(--status-error-text); }
  button.small { padding: 0.3rem 0.7rem; font-size: 0.8rem; }
  .btn-link {
    padding: 0.6rem 1rem;
    border-radius: 6px;
    background: var(--primary);
    color: white;
    text-decoration: none;
    font-size: 0.9rem;
    font-weight: 600;
    white-space: nowrap;
  }
  .btn-link:hover { background: var(--primary-hover); }
</style>
