<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { formatAppError } from "$lib/app-error";
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import StatusMessage from "$lib/components/ui/StatusMessage.svelte";
  import SurfaceCard from "$lib/components/ui/SurfaceCard.svelte";
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
    const parsedApiId = Number.parseInt(newApiId.trim(), 10);
    if (!Number.isInteger(parsedApiId) || parsedApiId <= 0) {
      status = "Telegram API ID must be a positive number.";
      return;
    }

    creating = true;
    status = "";
    try {
      await invoke("create_account", {
        label: newLabel.trim(),
        apiId: parsedApiId,
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

<section class="page-shell">
  <header class="page-hero">
    <div class="page-hero-copy">
      <span class="page-eyebrow">Telegram accounts</span>
      <h1>Accounts</h1>
      <p>
        Manage Telegram identities used for sync and analysis. Each account keeps its own API
        credentials, auth state, and linked sources.
      </p>
    </div>
    <div class="page-hero-meta">
      <Badge variant="info">{accounts.length} configured</Badge>
      <Badge>{Object.values(accountStatuses).filter((runtime) => runtime.status === "ready").length} ready</Badge>
    </div>
  </header>

  {#if status}
    <StatusMessage tone={status.startsWith("Error") ? "error" : "default"} className="page-status">
      {status}
    </StatusMessage>
  {/if}

  <div class="page-grid">
    <div class="page-stack">
      <section class="desk-panel account-catalog">
        <div class="panel-header">
          <div class="panel-header-copy">
            <span class="page-eyebrow">Workspace access</span>
            <h2>Configured accounts</h2>
            <p>Open auth, check runtime state, and keep sync-capable accounts healthy.</p>
          </div>
          <Badge variant="neutral">{accounts.length} total</Badge>
        </div>

        {#if accounts.length === 0}
          <EmptyState description="No accounts yet. Add one from the panel on the right." />
        {:else}
          <ul class="list">
            {#each accounts as acc (acc.id)}
              {@const runtime = runtimeStatus(acc.id)}
              <li>
                <SurfaceCard className="account-row">
                  <div class="row-main">
                    <div class="identity">
                      <div class="identity-mark" aria-hidden="true">
                        {acc.label.trim().charAt(0).toUpperCase() || "A"}
                      </div>
                      <div class="info">
                        <div class="title-row">
                          <span class="label">{acc.label}</span>
                          <Badge
                            variant={runtimeBadgeVariant(runtime)}
                            title={runtime?.status === "restore_failed" && runtime.message ? runtime.message : undefined}
                          >
                            {runtimeBadge(runtime)}
                          </Badge>
                        </div>
                        <div class="meta-row">
                          <span class="sub">{acc.phone ?? "not signed in"}</span>
                          <span class="sub">API ID {acc.api_id}</span>
                        </div>
                        {#if runtime?.message && runtime.status !== "ready"}
                          <p class="runtime-note">{runtime.message}</p>
                        {/if}
                      </div>
                    </div>
                    <div class="actions">
                      <Button variant="secondary" size="sm" onclick={() => goto(`/auth/${acc.id}`)}>
                        {authActionLabel(acc)}
                      </Button>
                      <Button variant="danger-soft" size="sm" onclick={() => deleteAccount(acc)}>Delete</Button>
                    </div>
                  </div>
                </SurfaceCard>
              </li>
            {/each}
          </ul>
        {/if}
      </section>
    </div>

    <div class="page-stack">
      <section class="desk-panel desk-panel-subtle">
        <div class="panel-header">
          <div class="panel-header-copy">
            <span class="page-eyebrow">Add account</span>
            <h2>New Telegram account</h2>
            <p>
              Get API credentials at
              <a href="https://my.telegram.org" target="_blank" rel="noreferrer">my.telegram.org</a>
              and add them here before starting sign-in.
            </p>
          </div>
        </div>

        <div class="form-stack">
          <label>Label
            <Input
              type="text"
              value={newLabel}
              placeholder="Personal"
              oninput={(event) => (newLabel = (event.currentTarget as HTMLInputElement).value)}
            />
          </label>
          <label>API ID
            <Input
              type="text"
              value={newApiId}
              placeholder="1234567"
              oninput={(event) => (newApiId = (event.currentTarget as HTMLInputElement).value)}
            />
          </label>
          <label>API Hash
            <Input
              type="text"
              value={newApiHash}
              placeholder="abcdef..."
              oninput={(event) => (newApiHash = (event.currentTarget as HTMLInputElement).value)}
            />
          </label>
        </div>

        <div class="desk-divider"></div>

        <div class="action-row">
          <Button onclick={createAccount} disabled={creating || !newLabel || !newApiId || !newApiHash}>
            {creating ? "Creating..." : "Add account"}
          </Button>
        </div>
      </section>

      <section class="desk-panel desk-panel-subtle notes-panel">
        <div class="panel-header-copy">
          <span class="page-eyebrow">Flow notes</span>
          <h3>How this fits the workspace</h3>
          <p>
            Accounts stay outside the main analysis canvas, but keep the same compact split:
            catalog on the left, utility panel on the right.
          </p>
        </div>
      </section>
    </div>
  </div>
</section>

<style>
  .list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }

  :global(.ui-surface-card.account-row) {
    padding: 0.9rem 1rem;
    background:
      linear-gradient(180deg, color-mix(in srgb, var(--panel-strong) 72%, transparent), transparent);
    border-color: color-mix(in srgb, var(--border) 88%, transparent);
  }

  .row-main {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 0.75rem;
  }

  .identity {
    display: flex;
    gap: 0.8rem;
    min-width: 0;
    align-items: flex-start;
  }

  .identity-mark {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 2.35rem;
    height: 2.35rem;
    border-radius: 0.9rem;
    background: color-mix(in srgb, var(--primary) 12%, var(--panel));
    color: var(--primary);
    font-size: 0.95rem;
    font-weight: 700;
    flex: 0 0 2.35rem;
  }

  .info {
    display: flex;
    flex-direction: column;
    gap: 0.22rem;
    min-width: 0;
  }

  .title-row {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    flex-wrap: wrap;
  }

  .label {
    font-size: 0.95rem;
    font-weight: 600;
  }

  .sub {
    font-size: 0.8rem;
    color: var(--muted);
  }

  .meta-row {
    display: flex;
    flex-wrap: wrap;
    gap: 0.45rem;
    align-items: center;
  }

  .runtime-note {
    margin: 0;
    font-size: 0.78rem;
    line-height: 1.45;
    color: var(--muted);
  }

  .actions,
  .action-row {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    flex-wrap: wrap;
  }

  .actions {
    flex-shrink: 0;
  }

  .form-stack {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    font-size: 0.85rem;
    color: var(--muted);
  }

  .panel-header-copy a {
    color: var(--primary);
  }

  :global(.page-status) {
    margin-bottom: 0;
  }

  .notes-panel h3 {
    margin: 0;
  }

  @media (max-width: 800px) {
    .row-main {
      flex-direction: column;
      align-items: stretch;
    }

    .identity {
      width: 100%;
    }

    .actions {
      justify-content: flex-start;
    }
  }
</style>
