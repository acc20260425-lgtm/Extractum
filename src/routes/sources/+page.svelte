<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { page } from "$app/stores";
  import { formatAppError } from "$lib/app-error";
  import SourceMessagesPanel from "$lib/components/source-messages-panel.svelte";
  import SourceRow from "$lib/components/source-row.svelte";
  import { openConfirmModal } from "$lib/modals";
  import { pushErrorToast } from "$lib/toasts";

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
    content: string | null;
    content_kind: string;
    has_media: boolean;
    media_kind: string | null;
    media_summary: string | null;
    media_file_name: string | null;
    media_mime_type: string | null;
    has_raw_data: boolean;
  }

  interface SyncResult {
    inserted: number;
    skipped: number;
    last_message_id: number | null;
    initial_sync_policy_applied: string | null;
  }

  interface SyncSettingsRecord {
    initial_sync_mode: "recent_messages" | "recent_days";
    initial_sync_value: number;
  }

  interface AccountRuntimeStatus {
    account_id: number;
    status: "not_initialized" | "restoring" | "ready" | "reauth_required" | "restore_failed";
    message: string | null;
  }

  interface RuntimeStatusEvent<T> {
    payload: T;
  }

  interface RestoreFailureEvent {
    message: string;
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
  let syncStatus = $state("");
  let accountStatuses = $state<Record<number, AccountRuntimeStatus>>({});
  let loadingDialogs = $state(false);
  let loadingItems = $state(false);
  let addingId = $state<number | string | null>(null);
  let selectedSourceId = $state<number | null>(null);
  let syncingIds = $state<Record<number, boolean>>({});
  let deletingIds = $state<Record<number, boolean>>({});
  let initialSyncMode = $state<SyncSettingsRecord["initial_sync_mode"]>("recent_messages");
  let initialSyncValue = $state("500");
  let savedInitialSyncMode = $state<SyncSettingsRecord["initial_sync_mode"]>("recent_messages");
  let savedInitialSyncValue = $state<number>(500);
  let loadingSyncSettings = $state(false);
  let savingSyncSettings = $state(false);
  let loadSourcesRequestId = 0;
  let syncStatusTimer: ReturnType<typeof setTimeout> | null = null;

  function setSyncStatus(message: string) {
    syncStatus = message;
    if (syncStatusTimer !== null) {
      clearTimeout(syncStatusTimer);
    }
    syncStatusTimer = setTimeout(() => {
      syncStatus = "";
      syncStatusTimer = null;
    }, 5000);
  }

  async function loadAccounts() {
    try {
      accounts = await invoke<AccountRecord[]>("list_accounts");
      await loadAccountStatuses();
    } catch (e) {
      pushErrorToast(formatAppError("loading accounts for sources", e));
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
      pushErrorToast(formatAppError("refreshing Telegram account status", e));
      accountStatuses = {};
    }
  }

  async function loadSources() {
    const requestId = ++loadSourcesRequestId;
    try {
      const nextSources = await invoke<SourceRecord[]>("list_sources", {
        accountId: selectedAccountId,
      });
      if (requestId !== loadSourcesRequestId) return;
      sources = nextSources;
    } catch (e) {
      if (requestId !== loadSourcesRequestId) return;
      status = formatAppError("loading sources", e);
    }
  }

  async function loadSyncSettings() {
    loadingSyncSettings = true;
    try {
      const settings = await invoke<SyncSettingsRecord>("get_sync_settings");
      initialSyncMode = settings.initial_sync_mode;
      initialSyncValue = String(settings.initial_sync_value);
      savedInitialSyncMode = settings.initial_sync_mode;
      savedInitialSyncValue = settings.initial_sync_value;
    } catch (e) {
      pushErrorToast(formatAppError("loading sync settings", e));
    } finally {
      loadingSyncSettings = false;
    }
  }

  async function saveSyncSettings() {
    const parsedValue = parseInt(initialSyncValue, 10);
    if (!Number.isFinite(parsedValue)) {
      status = "Initial sync value must be a number.";
      return;
    }

    savingSyncSettings = true;
    try {
      const settings = await invoke<SyncSettingsRecord>("save_sync_settings", {
        initialSyncMode,
        initialSyncValue: parsedValue,
      });
      initialSyncMode = settings.initial_sync_mode;
      initialSyncValue = String(settings.initial_sync_value);
      savedInitialSyncMode = settings.initial_sync_mode;
      savedInitialSyncValue = settings.initial_sync_value;
      status = `Initial sync policy saved: ${initialSyncPolicyLabel(settings.initial_sync_mode, settings.initial_sync_value)}.`;
    } catch (e) {
      status = formatAppError("saving sync settings", e);
    } finally {
      savingSyncSettings = false;
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
      status = formatAppError("loading Telegram channels", e);
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
      status = formatAppError("loading messages", e);
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
      status = formatAppError("adding the source from dialogs", e);
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
      status = formatAppError("adding the source", e);
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
      setSyncStatus(
        `Sync complete: inserted ${result.inserted}, skipped ${result.skipped}` +
        (result.last_message_id ? `, last message ${result.last_message_id}.` : ".") +
        (result.initial_sync_policy_applied
          ? ` First sync policy applied: ${result.initial_sync_policy_applied}.`
          : "")
      );
      if (selectedSourceId === sourceId) {
        await loadItems(sourceId);
      }
    } catch (e) {
      status = formatAppError("syncing the source", e);
    } finally {
      const next = { ...syncingIds };
      delete next[sourceId];
      syncingIds = next;
    }
  }

  async function deleteSource(sourceId: number) {
    const source = sources.find((item) => item.id === sourceId);
    if (!source) return;

    const sourceLabel = source.title ?? source.external_id;
    const confirmed = await openConfirmModal({
      title: "Delete source?",
      message:
        `The source "${sourceLabel}" will be removed from the app.\n\n` +
        "All synced messages for this source will also be deleted from the local database.",
      confirmLabel: "Delete",
      cancelLabel: "Cancel",
      tone: "danger",
    });
    if (!confirmed) return;

    deletingIds = { ...deletingIds, [sourceId]: true };
    status = "";

    try {
      await invoke("delete_source", { sourceId });
      await loadSources();
      if (selectedSourceId === sourceId) {
        items = [];
      }
      setSyncStatus(`Source "${sourceLabel}" deleted from the local database.`);
    } catch (e) {
      status = formatAppError("deleting the source", e);
    } finally {
      const next = { ...deletingIds };
      delete next[sourceId];
      deletingIds = next;
    }
  }

  async function selectSource(sourceId: number) {
    if (selectedSourceId === sourceId) return;
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
    if (!runtime || runtime.status === "not_initialized") {
      return "Initialize this account in Telegram before syncing.";
    }
    if (runtime.status === "restoring") {
      return "This account is still restoring after app startup.";
    }
    if (runtime.status === "reauth_required") {
      return "Sign in to this account again before syncing.";
    }
    if (runtime.status === "restore_failed") {
      return runtime.message ?? "The saved Telegram session could not be restored.";
    }
    return null;
  }

  function selectedAccountReady() {
    if (selectedAccountId === null) return false;
    const runtime = runtimeStatus(selectedAccountId);
    return runtime?.status === "ready";
  }

  function runtimeBadge(runtime: AccountRuntimeStatus | null) {
    if (!runtime) return null;
    if (runtime.status === "restoring") return "restoring...";
    if (runtime.status === "reauth_required") return "sign in required";
    if (runtime.status === "restore_failed") return "restore failed";
    if (runtime.status === "not_initialized") return "account not connected";
    return null;
  }

  function selectedSource() {
    if (selectedSourceId === null) return null;
    return sources.find((source) => source.id === selectedSourceId) ?? null;
  }

  function accountLabel(id: number | null) {
    if (id === null) return "—";
    return accounts.find((account) => account.id === id)?.label ?? `#${id}`;
  }

  function formatDate(timestamp: number) {
    return new Date(timestamp * 1000).toLocaleString();
  }

  function initialSyncValueLabel() {
    const parsedValue = parseInt(initialSyncValue, 10);
    return Number.isFinite(parsedValue) ? parsedValue : null;
  }

  function initialSyncAllowedRange() {
    if (initialSyncMode === "recent_days") {
      return { min: 1, max: 365, unit: "days" };
    }

    return { min: 50, max: 5000, unit: "messages" };
  }

  function initialSyncValidationMessage() {
    const parsedValue = initialSyncValueLabel();
    if (parsedValue === null) {
      return "Initial sync value must be a number.";
    }

    const { min, max, unit } = initialSyncAllowedRange();
    if (parsedValue < min || parsedValue > max) {
      return `Initial sync value must be between ${min} and ${max} ${unit}.`;
    }

    return "";
  }

  function initialSyncPolicyLabel(
    mode: SyncSettingsRecord["initial_sync_mode"],
    value: number | null
  ) {
    if (value === null) return "an invalid setting";
    if (mode === "recent_days") {
      return `last ${value} ${value === 1 ? "day" : "days"}`;
    }
    return `last ${value} ${value === 1 ? "message" : "messages"}`;
  }

  function initialSyncPolicySummary() {
    return initialSyncPolicyLabel(savedInitialSyncMode, savedInitialSyncValue);
  }

  $effect(() => {
    void loadSources();
    dialogs = [];
  });

  $effect(() => {
    if (sources.length === 0) {
      selectedSourceId = null;
      items = [];
      return;
    }

    if (selectedSourceId !== null && sources.some((source) => source.id === selectedSourceId)) {
      return;
    }

    void selectSource(sources[0].id);
  });

  onMount(() => {
    let disposed = false;
    let detachListener: (() => void) | null = null;
    let detachRestoreFailureListener: (() => void) | null = null;

    void loadAccounts();
    void loadSyncSettings();
    void listen<AccountRuntimeStatus>("telegram://account-status", ({ payload }: RuntimeStatusEvent<AccountRuntimeStatus>) => {
      if (disposed) return;
      accountStatuses = {
        ...accountStatuses,
        [payload.account_id]: payload,
      };
    }).then((unlisten) => {
      if (disposed) {
        unlisten();
        return;
      }
      detachListener = unlisten;
    });
    void listen<RestoreFailureEvent>("telegram://restore-failure", ({ payload }: RuntimeStatusEvent<RestoreFailureEvent>) => {
      if (disposed) return;
      pushErrorToast(`Telegram session restore failed: ${payload.message}`);
    }).then((unlisten) => {
      if (disposed) {
        unlisten();
        return;
      }
      detachRestoreFailureListener = unlisten;
    });

    return () => {
      disposed = true;
      if (detachListener !== null) {
        detachListener();
      }
      if (detachRestoreFailureListener !== null) {
        detachRestoreFailureListener();
      }
      if (syncStatusTimer !== null) {
        clearTimeout(syncStatusTimer);
      }
    };
  });
</script>

<h1>Sources</h1>

{#if status}
  <p class="status" class:error={status.startsWith("Error")}>{status}</p>
{/if}

{#if syncStatus}
  <p class="status">{syncStatus}</p>
{/if}

<div class="card">
  <h3>Account</h3>
  <div class="row">
    <select bind:value={selectedAccountId}>
      <option value={null}>All accounts</option>
      {#each accounts as acc (acc.id)}
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
    <h3>Initial Sync Policy</h3>
  </div>
  <p class="hint">
    Applies only to sources that have never been synced before. Later syncs still fetch only newer
    messages incrementally.
  </p>
  <div class="policy-grid">
    <label>Mode
      <select bind:value={initialSyncMode} disabled={loadingSyncSettings || savingSyncSettings}>
        <option value="recent_messages">Recent messages</option>
        <option value="recent_days">Recent days</option>
      </select>
    </label>
    <label>Value
      <input
        type="number"
        min={initialSyncMode === "recent_days" ? 1 : 50}
        max={initialSyncMode === "recent_days" ? 365 : 5000}
        bind:value={initialSyncValue}
        disabled={loadingSyncSettings || savingSyncSettings}
      />
    </label>
  </div>
  <div class="row policy-actions">
    <button
      onclick={saveSyncSettings}
      disabled={loadingSyncSettings || savingSyncSettings || !!initialSyncValidationMessage()}
    >
      {savingSyncSettings ? "Saving..." : "Save policy"}
    </button>
    <span class="policy-summary">
      Current first sync window: {initialSyncPolicySummary()}
    </span>
  </div>
  {#if initialSyncValidationMessage()}
    <p class="validation-message">{initialSyncValidationMessage()}</p>
  {/if}
</div>

<section class="workspace">
  <div class="card pane pane-list">
    <div class="card-header">
      <h3>Added Sources ({sources.length})</h3>
    </div>
    {#if sources.length === 0}
      <p class="empty">No sources yet.</p>
    {:else}
      <ul class="source-list">
        {#each sources as src (src.id)}
          <SourceRow
            source={src}
            selected={selectedSourceId === src.id}
            syncing={!!syncingIds[src.id]}
            deleting={!!deletingIds[src.id]}
            {accountLabel}
            {runtimeStatus}
            {syncDisabledReason}
            {formatDate}
            onSelect={selectSource}
            onSync={syncSource}
            onDelete={deleteSource}
          />
        {/each}
      </ul>
    {/if}
  </div>

  <div class="card pane pane-content">
    {#if selectedSource()}
      {@const currentSource = selectedSource()!}
      {@const currentSyncReason = syncDisabledReason(currentSource)}
      {@const currentRuntimeStatus = runtimeStatus(currentSource.account_id)}
      <div class="detail-header">
        <div class="detail-title">
          <h3>{currentSource.title ?? currentSource.external_id}</h3>
          <p>{accountLabel(currentSource.account_id)}</p>
          {#if currentSource.last_sync_state === null}
            <p class="first-sync-note">
              First sync will import {initialSyncPolicySummary()}. After that, this source switches
              to incremental sync using only newer Telegram messages.
            </p>
          {/if}
        </div>
        <div class="detail-actions">
          {#if currentSource.last_synced_at !== null}
            <span class="badge">synced {formatDate(currentSource.last_synced_at)}</span>
          {/if}
          {#if currentSource.account_id !== null}
            {@const runtimeBadgeLabel = runtimeBadge(currentRuntimeStatus)}
            {#if runtimeBadgeLabel}
              <span
                class="badge warning"
                title={currentRuntimeStatus?.status === "restore_failed" && currentRuntimeStatus.message
                  ? currentRuntimeStatus.message
                  : undefined}
              >
                {runtimeBadgeLabel}
              </span>
            {/if}
          {/if}
          {#if currentSource.is_member}
            <span class="badge member">subscribed</span>
          {:else}
            <span class="badge">not subscribed</span>
          {/if}
          <button
            class="small"
            onclick={() => syncSource(currentSource.id)}
            disabled={!!syncingIds[currentSource.id] || !!deletingIds[currentSource.id] || currentSyncReason !== null}
            title={currentSyncReason ?? undefined}
          >
            {syncingIds[currentSource.id] ? "Syncing..." : "Sync"}
          </button>
          <button
            class="small danger secondary"
            onclick={() => deleteSource(currentSource.id)}
            disabled={!!deletingIds[currentSource.id] || !!syncingIds[currentSource.id]}
          >
            {deletingIds[currentSource.id] ? "Deleting..." : "Delete"}
          </button>
        </div>
      </div>
      <SourceMessagesPanel {loadingItems} {items} {formatDate} embedded={true} />
    {:else}
      <div class="empty-detail">
        <h3>No source selected</h3>
        <p>Select a source on the left to view synced messages and run sync actions.</p>
      </div>
    {/if}
  </div>
</section>

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
        {#each dialogs as ch (ch.id)}
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
  .workspace {
    display: grid;
    grid-template-columns: minmax(360px, 430px) minmax(0, 1fr);
    gap: 1.25rem;
    align-items: start;
    margin-bottom: 1.5rem;
  }
  .pane {
    margin-bottom: 0;
    min-height: 18rem;
  }
  .pane-list {
    position: sticky;
    top: 1rem;
  }
  .pane-content {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    min-width: 0;
    min-height: 40rem;
  }
  .detail-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 1rem;
    padding-bottom: 1rem;
    border-bottom: 1px solid var(--border);
    min-width: 0;
  }
  .detail-title {
    min-width: 0;
    flex: 1 1 auto;
  }
  .detail-title h3 {
    margin: 0 0 0.35rem 0;
    font-size: 1.1rem;
    max-width: 100%;
    overflow-wrap: anywhere;
    word-break: break-word;
  }
  .detail-title p {
    margin: 0;
    color: var(--muted);
    font-size: 0.85rem;
    overflow-wrap: anywhere;
  }
  .detail-actions {
    display: flex;
    gap: 0.4rem;
    align-items: center;
    justify-content: flex-end;
    flex-wrap: wrap;
    flex: 0 1 22rem;
    min-width: 0;
  }
  .row { display: flex; gap: 0.5rem; align-items: center; }
  .row input { flex: 1; }
  .policy-grid {
    display: grid;
    grid-template-columns: minmax(180px, 220px) minmax(120px, 180px);
    gap: 0.8rem;
    align-items: end;
  }
  .policy-actions {
    flex-wrap: wrap;
    justify-content: space-between;
  }
  .policy-summary {
    color: var(--muted);
    font-size: 0.85rem;
  }
  .validation-message {
    margin: 0.75rem 0 0 0;
    color: var(--status-error-text);
    font-size: 0.85rem;
  }
  select {
    flex: 1;
    padding: 0.6rem 0.8rem;
    border-radius: 6px;
    font-size: 0.95rem;
  }
  .first-sync-note {
    margin: 0.5rem 0 0 0;
    color: var(--muted);
    font-size: 0.82rem;
    line-height: 1.45;
    max-width: 42rem;
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
    list-style: none;
  }
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
  .empty-detail {
    display: flex;
    flex-direction: column;
    justify-content: center;
    min-height: 18rem;
    color: var(--muted);
  }
  .empty-detail h3 {
    margin: 0 0 0.5rem 0;
    color: var(--text);
    font-size: 1.05rem;
  }
  .empty-detail p {
    margin: 0;
    max-width: 36rem;
    line-height: 1.5;
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
  button.danger.secondary {
    border: 1px solid color-mix(in srgb, var(--danger) 35%, var(--border));
    background: color-mix(in srgb, var(--danger) 12%, var(--panel));
    color: var(--danger);
  }
  button.danger.secondary:hover {
    background: color-mix(in srgb, var(--danger) 18%, var(--panel-hover));
  }
  @media (max-width: 1180px) {
    .workspace {
      grid-template-columns: 1fr;
    }
    .policy-grid {
      grid-template-columns: 1fr;
    }
    .pane-list {
      position: static;
    }
    .detail-header {
      flex-direction: column;
      align-items: stretch;
    }
    .detail-actions {
      justify-content: flex-start;
      flex-basis: auto;
    }
  }
</style>
