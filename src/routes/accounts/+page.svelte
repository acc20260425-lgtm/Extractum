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

  interface AccountRuntimeStatus {
    account_id: number;
    status: "not_initialized" | "restoring" | "ready" | "reauth_required" | "restore_failed";
    message: string | null;
  }

  let accounts = $state<AccountRecord[]>([]);
  let accountStatuses = $state<Record<number, AccountRuntimeStatus>>({});
  let status = $state("");
  let newLabel = $state("");
  let newApiId = $state("");
  let newApiHash = $state("");
  let creating = $state(false);

  async function loadAccounts() {
    try {
      accounts = await invoke<AccountRecord[]>("list_accounts");
      await loadAccountStatuses();
    } catch (e) {
      status = `Error: ${e}`;
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

  function runtimeStatus(accountId: number) {
    return accountStatuses[accountId] ?? null;
  }

  function runtimeBadge(runtime: AccountRuntimeStatus | null) {
    if (!runtime) return "account not connected";
    if (runtime.status === "restoring") return "restoring...";
    if (runtime.status === "ready") return "ready";
    if (runtime.status === "reauth_required") return "sign in required";
    if (runtime.status === "restore_failed") return "restore failed";
    return "account not connected";
  }

  function authActionLabel(account: AccountRecord) {
    const runtime = runtimeStatus(account.id);
    if (runtime?.status === "ready") return "Open";
    if (runtime?.status === "restoring") return "Checking";
    if (runtime?.status === "reauth_required") return "Re-auth";
    if (runtime?.status === "restore_failed") return "Fix auth";
    return account.phone ? "Re-auth" : "Sign in";
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

  onMount(() => {
    let disposed = false;
    let pollHandle: ReturnType<typeof setInterval> | null = null;

    void loadAccounts();

    pollHandle = setInterval(() => {
      if (disposed || accounts.length === 0) return;
      void loadAccountStatuses();
    }, 2000);

    return () => {
      disposed = true;
      if (pollHandle !== null) {
        clearInterval(pollHandle);
      }
    };
  });
</script>

<h1>Accounts</h1>

{#if status}
  <p class="status" class:error={status.startsWith("Error")}>{status}</p>
{/if}

<div class="card">
  <h3>Configured Accounts ({accounts.length})</h3>
  {#if accounts.length === 0}
    <p class="empty">No accounts yet. Add one below.</p>
  {:else}
    <ul class="list">
      {#each accounts as acc}
        {@const runtime = runtimeStatus(acc.id)}
        <li>
          <div class="info">
            <span class="label">{acc.label}</span>
            <div class="meta-row">
              <span class="sub">{acc.phone ?? "not signed in"} · API ID: {acc.api_id}</span>
              <span
                class="badge"
                class:ready={runtime?.status === "ready"}
                class:warning={runtime?.status === "restoring" || runtime?.status === "reauth_required"}
                class:error={runtime?.status === "restore_failed"}
                title={runtime?.status === "restore_failed" && runtime.message ? runtime.message : undefined}
              >
                {runtimeBadge(runtime)}
              </span>
            </div>
          </div>
          <div class="actions">
            <a href="/auth/{acc.id}" class="btn-link">
              {authActionLabel(acc)}
            </a>
            <button class="danger small" onclick={() => deleteAccount(acc)}>Delete</button>
          </div>
        </li>
      {/each}
    </ul>
  {/if}
</div>

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
  .meta-row { display: flex; flex-wrap: wrap; gap: 0.45rem; align-items: center; }
  .actions { display: flex; gap: 0.4rem; align-items: center; flex-shrink: 0; }
  .badge {
    font-size: 0.72rem;
    padding: 0.15rem 0.45rem;
    border-radius: 999px;
    background: var(--panel-hover);
    color: var(--muted);
    line-height: 1.2;
  }
  .badge.ready {
    background: color-mix(in srgb, #22c55e 18%, var(--panel));
    color: #15803d;
  }
  .badge.warning {
    background: color-mix(in srgb, #f59e0b 22%, var(--panel));
    color: #b45309;
  }
  .badge.error {
    background: color-mix(in srgb, #ef4444 16%, var(--panel));
    color: #b91c1c;
  }
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
  @media (max-width: 800px) {
    .list li {
      flex-direction: column;
      align-items: stretch;
    }
    .actions {
      justify-content: flex-start;
      flex-wrap: wrap;
    }
  }
</style>
