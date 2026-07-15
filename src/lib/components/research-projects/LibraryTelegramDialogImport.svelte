<script lang="ts">
  import { onMount } from "svelte";
  import Plus from "@lucide/svelte/icons/plus";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import {
    ExtractumBadge,
    ExtractumButton,
    ExtractumStatusMessage,
    ExtractumTextInput,
  } from "$lib/components/extractum-ui";
  import { getAccountRuntimeStatuses, listAccounts } from "$lib/api/accounts";
  import { addTelegramSource, listTelegramSources } from "$lib/api/sources";
  import { formatAppError } from "$lib/app-error";
  import { telegramDialogAddInput } from "$lib/ui/library-add-source-model";
  import type { AccountRecord, AccountRuntimeStatus } from "$lib/types/accounts";
  import type { DialogKindFilter, TelegramDialogSource } from "$lib/types/sources";

  let {
    onSourcesChanged,
    onStatus,
  }: {
    onSourcesChanged: (sourceId?: number) => void | Promise<void>;
    onStatus: (message: string) => void;
  } = $props();

  let accounts = $state<AccountRecord[]>([]);
  let accountStatuses = $state<Record<number, AccountRuntimeStatus>>({});
  let selectedAccountId = $state("");
  let kindFilter = $state<DialogKindFilter>("all");
  let query = $state("");
  let dialogs = $state<TelegramDialogSource[]>([]);
  let selectedDialogId = $state<number | null>(null);
  let loadingAccounts = $state(false);
  let loadingDialogs = $state(false);
  let adding = $state(false);
  let status = $state("");

  const activeAccountId = $derived.by(() => {
    const selected = Number(selectedAccountId);
    if (accounts.some((account) => account.id === selected)) return selected;
    return accounts[0]?.id ?? null;
  });
  const selectedAccount = $derived(accounts.find((account) => account.id === activeAccountId) ?? null);
  const selectedRuntime = $derived(selectedAccount ? accountStatuses[selectedAccount.id] ?? null : null);
  const selectedAccountReady = $derived(selectedRuntime?.status === "ready");
  const selectedDialog = $derived(dialogs.find((dialog) => dialog.id === selectedDialogId) ?? null);
  const filteredDialogs = $derived.by(() => {
    const normalizedQuery = query.trim().toLocaleLowerCase();
    return dialogs.filter((dialog) => {
      if (kindFilter !== "all" && dialog.sourceSubtype !== kindFilter) return false;
      if (!normalizedQuery) return true;
      return `${dialog.title} ${dialog.username ?? ""} ${dialog.id}`.toLocaleLowerCase().includes(normalizedQuery);
    });
  });
  const canLoadDialogs = $derived(Boolean(activeAccountId) && selectedAccountReady && !loadingDialogs);
  const canAdd = $derived(Boolean(activeAccountId && selectedDialog) && selectedAccountReady && !adding);

  async function loadAccountsAndStatuses() {
    loadingAccounts = true;
    status = "";
    try {
      accounts = await listAccounts();
      selectedAccountId = String(accounts[0]?.id ?? "");
      if (accounts.length > 0) {
        const statuses = await getAccountRuntimeStatuses(accounts.map((account) => account.id));
        accountStatuses = Object.fromEntries(statuses.map((runtime) => [runtime.account_id, runtime]));
      } else {
        accountStatuses = {};
      }
    } catch (error) {
      status = formatAppError("loading Telegram accounts", error);
    } finally {
      loadingAccounts = false;
    }
  }

  async function loadDialogs() {
    if (!activeAccountId || !selectedAccountReady) return;
    loadingDialogs = true;
    status = "";
    selectedDialogId = null;
    try {
      dialogs = await listTelegramSources(activeAccountId);
    } catch (error) {
      dialogs = [];
      status = formatAppError("loading Telegram dialogs", error);
    } finally {
      loadingDialogs = false;
    }
  }

  async function addSelectedDialog() {
    if (!activeAccountId || !selectedDialog || adding) return;
    adding = true;
    status = "";
    try {
      const source = await addTelegramSource(telegramDialogAddInput(activeAccountId, selectedDialog));
      onStatus(`Source "${source.title ?? source.externalId}" added.`);
      await onSourcesChanged(source.id);
    } catch (error) {
      status = formatAppError("adding Telegram source", error);
    } finally {
      adding = false;
    }
  }

  onMount(() => {
    void loadAccountsAndStatuses();
  });
</script>

<section class="library-telegram-dialog-import" aria-label="Telegram dialog import">
  <div class="toolbar">
    <label>
      <span>Account</span>
      <select
        bind:value={selectedAccountId}
        disabled={loadingAccounts || adding}
        onchange={() => {
          dialogs = [];
          selectedDialogId = null;
        }}
      >
        {#if accounts.length === 0}
          <option value="">No accounts configured</option>
        {/if}
        {#each accounts as account (account.id)}
          <option value={String(account.id)}>{account.label}</option>
        {/each}
      </select>
    </label>

    <label>
      <span>Kind</span>
      <select bind:value={kindFilter} disabled={loadingDialogs || adding}>
        <option value="all">All</option>
        <option value="channel">Channels</option>
        <option value="supergroup">Supergroups</option>
        <option value="group">Groups</option>
      </select>
    </label>

    <ExtractumButton onclick={loadDialogs} disabled={!canLoadDialogs}>
      <RefreshCw size={14} aria-hidden="true" />
      {loadingDialogs ? "Loading..." : "Load dialogs"}
    </ExtractumButton>
  </div>

  {#if !selectedAccount}
    <ExtractumStatusMessage tone="muted">Add and sign in to a Telegram account before adding Telegram sources.</ExtractumStatusMessage>
  {:else if !selectedAccountReady}
    <ExtractumStatusMessage tone="muted">
      Sign in to "{selectedAccount.label}" before loading Telegram dialogs.
    </ExtractumStatusMessage>
  {/if}

  {#if status}
    <ExtractumStatusMessage tone={status.startsWith("Error") ? "error" : "default"}>{status}</ExtractumStatusMessage>
  {/if}

  <ExtractumTextInput
    value={query}
    placeholder="Search dialogs"
    aria-label="Search Telegram dialogs"
    disabled={loadingDialogs || dialogs.length === 0}
    oninput={(event) => (query = (event.currentTarget as HTMLInputElement).value)}
  />

  <div class="dialog-list" aria-label="Telegram dialogs">
    {#if loadingDialogs}
      <ExtractumStatusMessage tone="muted">Loading Telegram dialogs...</ExtractumStatusMessage>
    {:else if dialogs.length === 0}
      <ExtractumStatusMessage tone="muted">Load dialogs from a ready account.</ExtractumStatusMessage>
    {:else}
      {#each filteredDialogs as dialog (dialog.id)}
        <button
          type="button"
          class:selected={dialog.id === selectedDialogId}
          onclick={() => (selectedDialogId = dialog.id)}
        >
          <span>
            <strong>{dialog.title}</strong>
            <small>{dialog.username ? `@${dialog.username}` : dialog.id}</small>
          </span>
          <ExtractumBadge>{dialog.sourceSubtype}</ExtractumBadge>
        </button>
      {/each}
    {/if}
  </div>

  <div class="footer">
    <ExtractumBadge>{filteredDialogs.length} visible</ExtractumBadge>
    <ExtractumButton onclick={addSelectedDialog} disabled={!canAdd}>
      <Plus size={14} aria-hidden="true" />
      {adding ? "Adding..." : "Add selected"}
    </ExtractumButton>
  </div>
</section>

<style>
  .library-telegram-dialog-import {
    display: grid;
    gap: 12px;
  }

  .toolbar,
  .footer {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    align-items: end;
    justify-content: space-between;
  }

  label {
    display: grid;
    gap: 4px;
    color: var(--extractum-muted);
    font-size: 13px;
  }

  select {
    height: 32px;
    min-width: 150px;
    border: 1px solid var(--extractum-border);
    border-radius: var(--extractum-radius);
    background: var(--extractum-surface);
    color: var(--extractum-text);
    font-size: 13px;
  }

  .dialog-list {
    display: grid;
    gap: 6px;
    max-height: 340px;
    overflow: auto;
    border: 1px solid var(--extractum-border);
    border-radius: var(--extractum-radius);
    padding: 6px;
  }

  .dialog-list button {
    display: flex;
    justify-content: space-between;
    gap: 8px;
    align-items: center;
    border: 1px solid transparent;
    border-radius: var(--extractum-radius);
    padding: 8px;
    background: transparent;
    color: var(--extractum-text);
    text-align: left;
  }

  .dialog-list button.selected,
  .dialog-list button:hover {
    border-color: var(--extractum-border);
    background: var(--extractum-surface-subtle);
  }

  .dialog-list span {
    min-width: 0;
    display: grid;
    gap: 2px;
  }

  small {
    color: var(--extractum-muted);
    font-size: 12px;
  }
</style>
