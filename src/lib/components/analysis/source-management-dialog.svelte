<script lang="ts">
  import { Check, Plus, RefreshCw, Send, Video } from "@lucide/svelte";
  import DesktopDialog from "$lib/components/desktop-dialog.svelte";
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import Select from "$lib/components/ui/Select.svelte";
  import StatusMessage from "$lib/components/ui/StatusMessage.svelte";
  import YoutubeSourceAddPanel from "$lib/components/analysis/youtube-source-add-panel.svelte";
  import { addTelegramSource, listTelegramSources } from "$lib/api/sources";
  import { formatAppError } from "$lib/app-error";
  import type { AccountRecord, AccountRuntimeStatus } from "$lib/types/accounts";
  import type {
    DialogKindFilter,
    Source,
    TelegramDialogSource,
    TelegramSourceKind,
  } from "$lib/types/sources";

  let {
    open,
    accounts,
    accountStatuses,
    existingSources,
    onClose,
    onSourcesChanged,
    onStatus,
  }: {
    open: boolean;
    accounts: AccountRecord[];
    accountStatuses: Record<number, AccountRuntimeStatus>;
    existingSources: Source[];
    onClose: () => void;
    onSourcesChanged: (sourceId?: number) => void | Promise<void>;
    onStatus: (message: string) => void;
  } = $props();

  let selectedAccountId = $state("");
  let kindFilter = $state<DialogKindFilter>("all");
  let dialogQuery = $state("");
  let dialogSources = $state<TelegramDialogSource[]>([]);
  let loadingDialogs = $state(false);
  let addingSourceKey = $state<string | null>(null);
  let manualRef = $state("");
  let manualKind = $state<TelegramSourceKind | "auto">("auto");
  let localStatus = $state("");
  let activeProvider = $state<"telegram" | "youtube">("telegram");

  const readyAccounts = $derived.by(() =>
    accounts.filter((account) => accountStatuses[account.id]?.status === "ready"),
  );

  const activeAccountId = $derived.by(() => {
    if (accounts.some((account) => account.id === Number(selectedAccountId))) {
      return selectedAccountId;
    }

    return String((readyAccounts[0] ?? accounts[0])?.id ?? "");
  });

  const selectedAccount = $derived.by(() =>
    accounts.find((account) => account.id === Number(activeAccountId)) ?? null,
  );

  const selectedRuntime = $derived.by(() =>
    selectedAccount ? accountStatuses[selectedAccount.id] ?? null : null,
  );

  const selectedAccountReady = $derived(selectedRuntime?.status === "ready");

  const existingDialogSourceKeys = $derived.by(() => {
    const accountId = Number(activeAccountId);
    return new Set(
      existingSources
        .filter((source) => source.accountId === accountId)
        .map((source) => `${source.sourceSubtype}:${source.externalId}`),
    );
  });

  const filteredDialogSources = $derived.by(() => {
    const query = dialogQuery.trim().toLocaleLowerCase();
    return dialogSources
      .filter((source) => {
        const matchesKind = kindFilter === "all" || source.telegramSourceKind === kindFilter;
        if (!matchesKind) return false;
        if (!query) return true;
        return (
          source.title.toLocaleLowerCase().includes(query) ||
          (source.username ?? "").toLocaleLowerCase().includes(query) ||
          String(source.id).includes(query)
        );
      })
      .sort(compareDialogSources);
  });

  function sourceKindLabel(kind: TelegramSourceKind | string) {
    switch (kind) {
      case "channel":
        return "channel";
      case "supergroup":
        return "supergroup";
      case "group":
        return "group";
      default:
        return "telegram";
    }
  }

  function dialogSourceKey(source: TelegramDialogSource) {
    return `${source.telegramSourceKind}:${source.id}`;
  }

  function sourceAlreadyAdded(source: TelegramDialogSource) {
    return existingDialogSourceKeys.has(dialogSourceKey(source));
  }

  function normalizedSortText(value: string | null | undefined) {
    return (value ?? "").trim().toLocaleLowerCase();
  }

  function compareSortText(left: string, right: string) {
    return left.localeCompare(right, undefined, {
      numeric: true,
      sensitivity: "base",
    });
  }

  function compareDialogSources(left: TelegramDialogSource, right: TelegramDialogSource) {
    const addedDelta = Number(sourceAlreadyAdded(left)) - Number(sourceAlreadyAdded(right));
    if (addedDelta !== 0) return addedDelta;

    const titleDelta = compareSortText(
      normalizedSortText(left.title),
      normalizedSortText(right.title),
    );
    if (titleDelta !== 0) return titleDelta;

    const kindDelta = compareSortText(left.telegramSourceKind, right.telegramSourceKind);
    if (kindDelta !== 0) return kindDelta;

    const usernameDelta = compareSortText(
      normalizedSortText(left.username),
      normalizedSortText(right.username),
    );
    if (usernameDelta !== 0) return usernameDelta;

    return Number(left.id) - Number(right.id);
  }

  async function loadDialogSources() {
    if (!activeAccountId || !selectedAccountReady) {
      return;
    }

    loadingDialogs = true;
    localStatus = "";
    try {
      dialogSources = await listTelegramSources(Number(activeAccountId));
    } catch (error) {
      dialogSources = [];
      localStatus = formatAppError("loading Telegram dialogs", error);
    } finally {
      loadingDialogs = false;
    }
  }

  async function addSource(sourceRef: string, telegramSourceKind: TelegramSourceKind | null, key: string) {
    if (!activeAccountId || !sourceRef.trim()) {
      return;
    }

    addingSourceKey = key;
    localStatus = "";
    try {
      const source = await addTelegramSource({
        accountId: Number(activeAccountId),
        sourceRef: sourceRef.trim(),
        expectedKind: telegramSourceKind,
      });
      onStatus(`Source "${source.title ?? source.externalId}" added.`);
      await onSourcesChanged(source.id);
      if (key === "manual") {
        manualRef = "";
      }
    } catch (error) {
      localStatus = formatAppError("adding the source", error);
    } finally {
      addingSourceKey = null;
    }
  }

  function addDialogSource(source: TelegramDialogSource) {
    return addSource(String(source.id), source.telegramSourceKind, dialogSourceKey(source));
  }

  function addManualSource() {
    return addSource(
      manualRef,
      manualKind === "auto" ? null : manualKind,
      "manual",
    );
  }
</script>

<DesktopDialog
  {open}
  title="Manage sources"
  description="Add Telegram and YouTube sources."
  width="58rem"
  onClose={onClose}
>
  <div class="source-manager">
    <div class="provider-tabs" role="tablist" aria-label="Source providers">
      <Button
        variant="secondary"
        selected={activeProvider === "telegram"}
        role="tab"
        ariaSelected={activeProvider === "telegram"}
        ariaControls="telegram-source-panel"
        onclick={() => (activeProvider = "telegram")}
      >
        <Send size={15} aria-hidden="true" />
        Telegram
      </Button>
      <Button
        variant="secondary"
        selected={activeProvider === "youtube"}
        role="tab"
        ariaSelected={activeProvider === "youtube"}
        ariaControls="youtube-source-panel"
        onclick={() => (activeProvider = "youtube")}
      >
        <Video size={15} aria-hidden="true" />
        YouTube
      </Button>
    </div>

    {#if activeProvider === "telegram"}
      <div id="telegram-source-panel" class="provider-panel" role="tabpanel">
    <div class="manager-toolbar">
      <label>Account
        <Select
          value={activeAccountId}
          onchange={(event) => {
            selectedAccountId = (event.currentTarget as HTMLSelectElement).value;
            dialogSources = [];
            localStatus = "";
          }}
          disabled={accounts.length === 0 || loadingDialogs || addingSourceKey !== null}
        >
          {#if accounts.length === 0}
            <option value="">No accounts configured</option>
          {/if}
          {#each accounts as account (account.id)}
            {@const runtime = accountStatuses[account.id]}
            <option value={String(account.id)}>
              {account.label} - {runtime?.status === "ready" ? "ready" : "sign in required"}
            </option>
          {/each}
        </Select>
      </label>

      <label>Dialog kind
        <Select
          value={kindFilter}
          onchange={(event) => (kindFilter = (event.currentTarget as HTMLSelectElement).value as DialogKindFilter)}
          disabled={loadingDialogs}
        >
          <option value="all">All</option>
          <option value="channel">Channels</option>
          <option value="supergroup">Supergroups</option>
          <option value="group">Groups</option>
        </Select>
      </label>

      <Button
        variant="secondary"
        onclick={loadDialogSources}
        disabled={!selectedAccountReady || loadingDialogs || addingSourceKey !== null}
      >
        <RefreshCw size={15} aria-hidden="true" />
        {loadingDialogs ? "Refreshing..." : "Refresh"}
      </Button>
    </div>

    {#if !selectedAccount}
      <StatusMessage tone="muted">Add and sign in to a Telegram account before adding sources.</StatusMessage>
    {:else if !selectedAccountReady}
      <StatusMessage tone="muted">
        Sign in to "{selectedAccount.label}" before loading Telegram dialogs.
      </StatusMessage>
    {/if}

    {#if localStatus}
      <StatusMessage tone={localStatus.startsWith("Error") ? "error" : "default"}>
        {localStatus}
      </StatusMessage>
    {/if}

    <section class="manual-section">
      <div class="section-copy">
        <strong>Manual add</strong>
        <span>Use public @username, t.me/name, or a numeric id visible in this account's dialogs.</span>
      </div>
      <div class="manual-grid">
        <label>Source reference
          <Input
            value={manualRef}
            placeholder="@channel or t.me/name"
            disabled={!selectedAccountReady || addingSourceKey !== null}
            oninput={(event) => (manualRef = (event.currentTarget as HTMLInputElement).value)}
          />
        </label>
        <label>Expected kind
          <Select
            value={manualKind}
            disabled={!selectedAccountReady || addingSourceKey !== null}
            onchange={(event) => (manualKind = (event.currentTarget as HTMLSelectElement).value as TelegramSourceKind | "auto")}
          >
            <option value="auto">Auto</option>
            <option value="channel">Channel</option>
            <option value="supergroup">Supergroup</option>
            <option value="group">Group</option>
          </Select>
        </label>
        <Button
          onclick={addManualSource}
          disabled={!selectedAccountReady || !manualRef.trim() || addingSourceKey !== null}
        >
          <Plus size={15} aria-hidden="true" />
          {addingSourceKey === "manual" ? "Adding..." : "Add"}
        </Button>
      </div>
    </section>

    <section class="dialog-section">
      <div class="section-head">
        <div class="section-copy">
          <strong>Account dialogs</strong>
          <span>Add private or public Telegram sources already visible to this account.</span>
        </div>
        <Badge>{filteredDialogSources.length} visible</Badge>
      </div>

      <Input
        type="search"
        value={dialogQuery}
        placeholder="Search dialogs"
        ariaLabel="Search Telegram dialogs"
        disabled={loadingDialogs}
        oninput={(event) => (dialogQuery = (event.currentTarget as HTMLInputElement).value)}
      />

      <div class="dialog-list">
        {#if loadingDialogs}
          <div class="dialog-empty">Loading Telegram dialogs...</div>
        {:else if selectedAccountReady && dialogSources.length === 0}
          <div class="dialog-empty">Refresh to load dialogs for this account.</div>
        {:else if filteredDialogSources.length === 0}
          <div class="dialog-empty">No dialogs match the current filters.</div>
        {:else}
          {#each filteredDialogSources as source (dialogSourceKey(source))}
            {@const alreadyAdded = sourceAlreadyAdded(source)}
            {@const key = dialogSourceKey(source)}
            <article class="dialog-row">
              <div class="dialog-avatar" aria-hidden="true">
                {#if source.photoDataUrl}
                  <img src={source.photoDataUrl} alt="" loading="lazy" />
                {:else}
                  <span>{source.title.trim().charAt(0).toUpperCase() || "#"}</span>
                {/if}
              </div>
              <div class="dialog-copy">
                <strong>{source.title}</strong>
                <div class="dialog-meta">
                  <span>{sourceKindLabel(source.telegramSourceKind)}</span>
                  {#if source.username}
                    <span>@{source.username}</span>
                  {:else}
                    <span>{source.id}</span>
                  {/if}
                  <span>{source.isMember ? "member" : "not member"}</span>
                </div>
              </div>
              <Button
                size="sm"
                variant={alreadyAdded ? "secondary" : "primary"}
                onclick={() => addDialogSource(source)}
                disabled={alreadyAdded || addingSourceKey !== null}
              >
                {#if alreadyAdded}
                  <Check size={13} aria-hidden="true" />
                {:else}
                  <Plus size={13} aria-hidden="true" />
                {/if}
                {alreadyAdded ? "Added" : addingSourceKey === key ? "Adding..." : "Add"}
              </Button>
            </article>
          {/each}
        {/if}
      </div>
    </section>
      </div>
    {:else}
      <div id="youtube-source-panel" class="provider-panel" role="tabpanel">
        <YoutubeSourceAddPanel {onSourcesChanged} {onStatus} />
      </div>
    {/if}
  </div>
</DesktopDialog>

<style>
  .source-manager {
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    color: var(--muted);
    font-size: 0.83rem;
  }

  .manager-toolbar {
    display: grid;
    grid-template-columns: minmax(0, 1.2fr) minmax(11rem, 0.55fr) auto;
    gap: 0.7rem;
    align-items: end;
  }

  .provider-tabs {
    display: flex;
    flex-wrap: wrap;
    gap: 0.45rem;
    padding-bottom: 0.75rem;
    border-bottom: 1px solid color-mix(in srgb, var(--border) 76%, transparent);
  }

  .provider-tabs :global(.ui-button) {
    min-width: 7rem;
  }

  .provider-panel {
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
  }

  .manual-section,
  .dialog-section {
    display: flex;
    flex-direction: column;
    gap: 0.7rem;
    padding-top: 0.85rem;
    border-top: 1px solid color-mix(in srgb, var(--border) 76%, transparent);
  }

  .section-head {
    display: flex;
    justify-content: space-between;
    gap: 0.7rem;
    align-items: flex-start;
  }

  .section-copy {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    min-width: 0;
  }

  .section-copy strong {
    font-size: 0.92rem;
  }

  .section-copy span {
    color: var(--muted);
    font-size: 0.82rem;
    line-height: 1.4;
  }

  .manual-grid {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(10rem, 0.35fr) auto;
    gap: 0.65rem;
    align-items: end;
  }

  .dialog-list {
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
    max-height: 22rem;
    overflow: auto;
    padding-right: 0.1rem;
  }

  .dialog-row {
    display: flex;
    align-items: center;
    gap: 0.65rem;
    padding: 0.65rem;
    border: 1px solid color-mix(in srgb, var(--border) 82%, transparent);
    border-radius: 10px;
    background: var(--panel-strong);
    min-width: 0;
  }

  .dialog-avatar {
    width: 2.25rem;
    height: 2.25rem;
    flex: 0 0 2.25rem;
    border-radius: 0.75rem;
    overflow: hidden;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: color-mix(in srgb, var(--primary) 12%, var(--panel));
    color: var(--primary);
    font-weight: 700;
    font-size: 0.86rem;
  }

  .dialog-avatar img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .dialog-copy {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
  }

  .dialog-copy strong {
    font-size: 0.9rem;
    overflow-wrap: anywhere;
  }

  .dialog-meta {
    display: flex;
    flex-wrap: wrap;
    gap: 0.45rem;
    color: var(--muted);
    font-size: 0.76rem;
  }

  .dialog-empty {
    padding: 0.85rem 0.95rem;
    border: 1px dashed var(--border);
    border-radius: 10px;
    color: var(--muted);
    background: color-mix(in srgb, var(--panel-strong) 72%, transparent);
    font-size: 0.86rem;
  }

  @media (max-width: 780px) {
    .manager-toolbar,
    .manual-grid {
      grid-template-columns: 1fr;
    }

    .section-head,
    .dialog-row {
      align-items: stretch;
    }

    .dialog-row {
      flex-wrap: wrap;
    }
  }
</style>
