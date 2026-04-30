<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { formatAppError } from "$lib/app-error";
  import Badge from "$lib/components/ui/Badge.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import StatusMessage from "$lib/components/ui/StatusMessage.svelte";
  import type { BadgeVariant } from "$lib/components/ui/types";
  import { openConfirmModal } from "$lib/modals";
  import { pushErrorToast } from "$lib/toasts";
  import type { AccountRecord, AccountRuntimeStatus } from "$lib/types/accounts";

  interface RuntimeStatusEvent<T> {
    payload: T;
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
      status = formatAppError("loading accounts", e);
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

  function runtimeBadgeVariant(runtime: AccountRuntimeStatus | null): BadgeVariant {
    if (!runtime) return "neutral";
    if (runtime.status === "ready") return "success";
    if (runtime.status === "restoring" || runtime.status === "reauth_required") return "warning";
    if (runtime.status === "restore_failed") return "danger";
    return "neutral";
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
      status = formatAppError("creating the account", e);
    } finally {
      creating = false;
    }
  }

  async function deleteAccount(account: AccountRecord) {
    const confirmed = await openConfirmModal({
      title: "Delete account?",
      message:
        `The account "${account.label}" will be removed from the local app.\n\n` +
        "Its linked sources will also be deleted from the local database.",
      confirmLabel: "Delete",
      cancelLabel: "Cancel",
      tone: "danger",
    });
    if (!confirmed) return;

    try {
      await invoke("delete_account", { accountId: account.id });
      await loadAccounts();
    } catch (e) {
      status = formatAppError("deleting the account", e);
    }
  }

  onMount(() => {
    let disposed = false;
    let detachListener: (() => void) | null = null;

    void loadAccounts();
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

    return () => {
      disposed = true;
      if (detachListener !== null) {
        detachListener();
      }
    };
  });
</script>

<h1>Accounts</h1>

{#if status}
  <StatusMessage tone={status.startsWith("Error") ? "error" : "default"} className="page-status">
    {status}
  </StatusMessage>
{/if}

<div class="card">
  <h3>Configured Accounts ({accounts.length})</h3>
  {#if accounts.length === 0}
    <EmptyState description="No accounts yet. Add one below." />
  {:else}
    <ul class="list">
      {#each accounts as acc (acc.id)}
        {@const runtime = runtimeStatus(acc.id)}
        <li>
          <div class="info">
            <span class="label">{acc.label}</span>
            <div class="meta-row">
              <span class="sub">{acc.phone ?? "not signed in"} · API ID: {acc.api_id}</span>
              <Badge
                variant={runtimeBadgeVariant(runtime)}
                title={runtime?.status === "restore_failed" && runtime.message ? runtime.message : undefined}
              >
                {runtimeBadge(runtime)}
              </Badge>
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
  :global(.page-status) { margin-bottom: 1rem; }
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
