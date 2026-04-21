<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  interface AccountRecord {
    id: number;
    label: string;
    api_id: number;
    api_hash: string;
    phone: string | null;
    created_at: number;
  }

  let accounts = $state<AccountRecord[]>([]);
  let status = $state("");
  let newLabel = $state("");
  let newApiId = $state("");
  let newApiHash = $state("");
  let creating = $state(false);

  async function loadAccounts() {
    try {
      accounts = await invoke<AccountRecord[]>("list_accounts");
    } catch (e) {
      status = `Error: ${e}`;
    }
  }

  async function createAccount() {
    if (!newLabel.trim() || !newApiId.trim() || !newApiHash.trim()) return;
    creating = true;
    status = "";
    try {
      await invoke("create_account", {
        label: newLabel.trim(),
        apiId: parseInt(newApiId),
        apiHash: newApiHash.trim(),
      });
      newLabel = "";
      newApiId = "";
      newApiHash = "";
      await loadAccounts();
    } catch (e) {
      status = `Error: ${e}`;
    } finally {
      creating = false;
    }
  }

  async function deleteAccount(account: AccountRecord) {
    const confirmed = window.confirm(
      `Delete account "${account.label}"?\n\nThis will also remove its linked sources from the local database.`
    );
    if (!confirmed) return;

    try {
      await invoke("delete_account", { accountId: account.id });
      await loadAccounts();
    } catch (e) {
      status = `Error: ${e}`;
    }
  }

  onMount(loadAccounts);
</script>

<h1>Accounts</h1>

{#if status}
  <p class="status" class:error={status.startsWith("Error")}>{status}</p>
{/if}

<!-- Account list -->
<div class="card">
  <h3>Configured Accounts ({accounts.length})</h3>
  {#if accounts.length === 0}
    <p class="empty">No accounts yet. Add one below.</p>
  {:else}
    <ul class="list">
      {#each accounts as acc}
        <li>
          <div class="info">
            <span class="label">{acc.label}</span>
            <span class="sub">{acc.phone ?? "not signed in"} · API ID: {acc.api_id}</span>
          </div>
          <div class="actions">
            <a href="/auth/{acc.id}" class="btn-link">
              {acc.phone ? "Re-auth" : "Sign in"}
            </a>
            <button class="danger small" onclick={() => deleteAccount(acc)}>Delete</button>
          </div>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<!-- Add account -->
<div class="card">
  <h3>Add Account</h3>
  <p class="hint">Get API credentials at <a href="https://my.telegram.org" target="_blank">my.telegram.org</a></p>
  <label>Label (e.g. "Personal", "Work")
    <input type="text" bind:value={newLabel} placeholder="Personal" />
  </label>
  <label>API ID
    <input type="text" bind:value={newApiId} placeholder="1234567" />
  </label>
  <label>API Hash
    <input type="text" bind:value={newApiHash} placeholder="abcdef..." />
  </label>
  <button onclick={createAccount} disabled={creating || !newLabel || !newApiId || !newApiHash}>
    {creating ? "Creating..." : "Add Account"}
  </button>
</div>

<style>
  .card {
    background: var(--panel);
    border: 1px solid var(--border);
    box-shadow: var(--shadow);
    border-radius: 12px;
    padding: 1.5rem;
    margin-bottom: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }
  .list { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 0.5rem; }
  .list li {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.6rem 0.75rem;
    background: var(--panel-strong);
    border-radius: 8px;
    gap: 0.5rem;
  }
  .info { display: flex; flex-direction: column; gap: 0.15rem; min-width: 0; }
  .label { font-size: 0.95rem; font-weight: 600; }
  .sub { font-size: 0.8rem; color: var(--muted); }
  .actions { display: flex; gap: 0.4rem; align-items: center; flex-shrink: 0; }
  .btn-link {
    font-size: 0.8rem;
    padding: 0.3rem 0.7rem;
    border-radius: 6px;
    background: var(--primary);
    color: white;
    text-decoration: none;
    font-weight: 600;
  }
  .btn-link:hover { background: var(--primary-hover); }
  button.small { padding: 0.3rem 0.7rem; font-size: 0.8rem; }
  label { display: flex; flex-direction: column; gap: 0.3rem; font-size: 0.85rem; color: var(--muted); }
  .hint { font-size: 0.85rem; color: var(--muted); margin: 0; }
  .hint a { color: var(--primary); }
  .empty { color: var(--muted); font-size: 0.9rem; margin: 0; }
  .status { padding: 0.6rem 1rem; border-radius: 6px; background: var(--status-bg); font-size: 0.9rem; margin-bottom: 1rem; }
  .status.error { background: var(--status-error-bg); color: var(--status-error-text); }
</style>
