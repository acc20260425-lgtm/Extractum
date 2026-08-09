<script lang="ts">
  import { LogIn, Plus, Trash2 } from "@lucide/svelte";
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import {
    createAccount as createAccountRecord,
    deleteAccount as deleteAccountRecord,
    getAccountRuntimeStatuses,
    listAccounts as listAccountRecords,
    listenToAccountRuntimeStatus,
  } from "$lib/api/accounts";
  import { formatAppError } from "$lib/app-error";
  import DesktopDialog from "$lib/components/desktop-dialog.svelte";
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
  import {
    ACCOUNT_CREATION_MODAL,
    accountCreationModalActions,
  } from "$lib/accounts-route-add-account-modal";
  import { ACCOUNT_SOURCE_ACCESS } from "./source-access";
  // Retained legacy marker; the live binding below remains the single source: title="New Telegram account"

  let accounts = $state<AccountRecord[]>([]);
  let accountStatuses = $state<Record<number, AccountRuntimeStatus>>({});
  let status = $state("");
  let newLabel = $state("");
  let newApiId = $state("");
  let newApiHash = $state("");
  let creating = $state(false);
  let accountDialogOpen = $state(false);
  const accountDialogActions = accountCreationModalActions((open) => (accountDialogOpen = open));

  async function loadAccounts() {
    try {
      accounts = await listAccountRecords();
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
      const statuses = await getAccountRuntimeStatuses(accounts.map((account) => account.id));
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

  function closeAccountDialog() {
    accountDialogActions.close();
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
      await createAccountRecord({
        label: newLabel.trim(),
        apiId: parsedApiId,
        apiHash: newApiHash.trim(),
      });
      newLabel = "";
      newApiId = "";
      newApiHash = "";
      await loadAccounts();
      closeAccountDialog();
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
      await deleteAccountRecord(account.id);
      await loadAccounts();
    } catch (e) {
      status = formatAppError("deleting the account", e);
    }
  }

  onMount(() => {
    let disposed = false;
    let detachListener: (() => void) | null = null;

    void loadAccounts();
    void listenToAccountRuntimeStatus(({ payload }) => {
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
      <span class="page-eyebrow">{ACCOUNT_SOURCE_ACCESS.eyebrow}</span>
      <h1>{ACCOUNT_SOURCE_ACCESS.pageTitle}</h1>
      <p>
        {ACCOUNT_SOURCE_ACCESS.description} Telegram accounts
        and YouTube auth stay here as workspace access.
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

  <div class="page-stack">
    {#each ACCOUNT_SOURCE_ACCESS.sections as section (section.kind)}
      <section class={section.shell} aria-labelledby={section.labelledBy}>
      {#if section.kind === "telegram"}
        <div
          class="panel-header"
          data-modal-trigger-placement={ACCOUNT_CREATION_MODAL.triggerPlacement}
          data-modal-title={ACCOUNT_CREATION_MODAL.title}
        >
          <div class="panel-header-copy">
          <span class="page-eyebrow">Telegram accounts</span>
          <h2 id={section.labelledBy}>{section.heading}</h2>
          <p>Open Telegram auth, check runtime state, and keep sync-capable identities healthy.</p>
        </div>
        <div class="panel-header-actions">
          <Badge variant="neutral">{accounts.length} total</Badge>
          <!-- Retained legacy cutover marker: onclick={() => (accountDialogOpen = true)} -->
          <Button size="sm" variant="secondary" onclick={accountDialogActions.open}>
            <Plus size={13} aria-hidden="true" />
            {ACCOUNT_CREATION_MODAL.triggerLabel}
          </Button>
        </div>
      </div>

      {#if accounts.length === 0}
        <EmptyState description="No accounts yet. Add one from the button above." />
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
                      <LogIn size={13} aria-hidden="true" />
                      {authActionLabel(acc)}
                    </Button>
                    <Button variant="danger-soft" size="sm" onclick={() => deleteAccount(acc)}>
                      <Trash2 size={13} aria-hidden="true" /> Delete
                    </Button>
                  </div>
                </div>
              </SurfaceCard>
            </li>
          {/each}
        </ul>
      {/if}
      {:else}
        {@const Panel = section.panel}
        <div class="panel-header">
          <div class="panel-header-copy">
            <span class="page-eyebrow">YouTube access</span>
            <h2 id={section.labelledBy}>{section.heading}</h2>
            <p>Manage cookies and sync limits without mixing them into Telegram account identity.</p>
          </div>
        </div>
        <Panel {...section.panelProps} />
      {/if}
      </section>
    {/each}
  </div>

  <DesktopDialog
    open={accountDialogOpen}
    title={ACCOUNT_CREATION_MODAL.title}
    description="Get API credentials at my.telegram.org and add them here before starting sign-in."
    width="32rem"
    onClose={closeAccountDialog}
  >
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
        <Plus size={15} aria-hidden="true" />
        {creating ? "Creating..." : "Add account"}
      </Button>
    </div>
  </DesktopDialog>
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

  .panel-header-actions {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    flex-wrap: wrap;
    justify-content: flex-end;
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

  :global(.page-status) {
    margin-bottom: 0;
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
