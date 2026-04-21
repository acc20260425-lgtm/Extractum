<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  interface ChannelInfo {
    id: number;
    title: string;
    username: string | null;
    is_member: boolean;
  }

  interface SourceRecord {
    id: number;
    external_id: string;
    title: string | null;
    is_member: boolean;
    is_active: boolean;
    created_at: number;
  }

  let sources = $state<SourceRecord[]>([]);
  let dialogs = $state<ChannelInfo[]>([]);
  let manualRef = $state("");
  let status = $state("");
  let loadingDialogs = $state(false);
  let addingId = $state<number | string | null>(null);

  async function loadSources() {
    try {
      sources = await invoke<SourceRecord[]>("list_sources");
    } catch (e) {
      status = `Error loading sources: ${e}`;
    }
  }

  async function loadDialogs() {
    loadingDialogs = true;
    status = "";
    try {
      dialogs = await invoke<ChannelInfo[]>("list_telegram_channels");
    } catch (e) {
      status = `Error: ${e}`;
    } finally {
      loadingDialogs = false;
    }
  }

  async function addFromDialog(channel: ChannelInfo) {
    addingId = channel.id;
    try {
      const ref = channel.username ? `@${channel.username}` : String(channel.id);
      await invoke("add_telegram_source", { channelRef: ref });
      await loadSources();
    } catch (e) {
      status = `Error: ${e}`;
    } finally {
      addingId = null;
    }
  }

  async function addManual() {
    if (!manualRef.trim()) return;
    addingId = "manual";
    status = "";
    try {
      await invoke("add_telegram_source", { channelRef: manualRef.trim() });
      manualRef = "";
      await loadSources();
    } catch (e) {
      status = `Error: ${e}`;
    } finally {
      addingId = null;
    }
  }

  function isAlreadyAdded(channel: ChannelInfo) {
    return sources.some((s) => s.external_id === String(channel.id));
  }

  onMount(loadSources);
</script>

<h1>Sources</h1>

{#if status}
  <p class="status" class:error={status.startsWith("Error")}>{status}</p>
{/if}

<!-- Added sources -->
<div class="card">
  <div class="card-header">
    <h3>Added Sources ({sources.length})</h3>
  </div>
  {#if sources.length === 0}
    <p class="empty">No sources yet. Add channels below.</p>
  {:else}
    <ul class="source-list">
      {#each sources as src}
        <li>
          <span class="title">{src.title ?? src.external_id}</span>
          <span class="badges">
            {#if src.is_member}
              <span class="badge member">subscribed</span>
            {:else}
              <span class="badge">not subscribed</span>
            {/if}
            {#if src.is_active}
              <span class="badge active">active</span>
            {/if}
          </span>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<!-- Manual add -->
<div class="card">
  <h3>Add by Username or Link</h3>
  <div class="row">
    <input
      type="text"
      bind:value={manualRef}
      placeholder="@channel or https://t.me/channel"
      onkeydown={(e) => e.key === "Enter" && addManual()}
    />
    <button onclick={addManual} disabled={addingId === "manual" || !manualRef.trim()}>
      {addingId === "manual" ? "Adding..." : "Add"}
    </button>
  </div>
</div>

<!-- From dialogs -->
<div class="card">
  <div class="card-header">
    <h3>My Channels</h3>
    <button class="secondary small" onclick={loadDialogs} disabled={loadingDialogs}>
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
            {#if ch.username}
              <span class="username">@{ch.username}</span>
            {/if}
          </div>
          <div class="channel-actions">
            {#if !ch.is_member}
              <span class="badge">not subscribed</span>
            {/if}
            {#if added}
              <span class="badge active">added</span>
            {:else}
              <button
                class="small"
                onclick={() => addFromDialog(ch)}
                disabled={addingId === ch.id}
              >
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

<style>
  .card {
    background: #2a2a2a;
    border-radius: 12px;
    padding: 1.5rem;
    margin-bottom: 1.5rem;
  }
  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }
  .card-header h3 { margin: 0; }
  .row {
    display: flex;
    gap: 0.5rem;
  }
  .row input { flex: 1; }
  .row button { flex-shrink: 0; }
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
    background: #1a1a1a;
    border-radius: 8px;
    gap: 0.5rem;
  }
  .channel-info { display: flex; flex-direction: column; gap: 0.1rem; min-width: 0; }
  .channel-actions { display: flex; align-items: center; gap: 0.4rem; flex-shrink: 0; }
  .title { font-size: 0.95rem; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .username { font-size: 0.8rem; color: #888; }
  .badges { display: flex; gap: 0.3rem; flex-shrink: 0; }
  .badge {
    font-size: 0.7rem;
    padding: 0.15rem 0.5rem;
    border-radius: 4px;
    background: #333;
    color: #aaa;
    white-space: nowrap;
  }
  .badge.member, .badge.active { background: #1a3a1a; color: #6f6; }
  .empty { color: #666; font-size: 0.9rem; margin: 0; }
  .status { padding: 0.6rem 1rem; border-radius: 6px; background: #1e3a5f; font-size: 0.9rem; margin-bottom: 1rem; }
  .status.error { background: #4a1a1a; color: #f88; }
  button.small { padding: 0.3rem 0.7rem; font-size: 0.8rem; }
</style>
