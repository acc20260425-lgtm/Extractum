<script lang="ts">
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import StatusMessage from "$lib/components/ui/StatusMessage.svelte";
  import { membershipLabel, sourceCapabilities, sourceKindLabel } from "$lib/source-capabilities";
  import type { AccountRuntimeStatus } from "$lib/types/accounts";
  import type { Source } from "$lib/types/sources";

  let {
    source,
    selected,
    syncing,
    deleting,
    accountLabel,
    runtimeStatus,
    syncDisabledReason,
    formatDate,
    onSelect,
    onSync,
    onDelete,
  }: {
    source: Source;
    selected: boolean;
    syncing: boolean;
    deleting: boolean;
    accountLabel: (id: number | null) => string;
    runtimeStatus: (accountId: number | null) => AccountRuntimeStatus | null;
    syncDisabledReason: (source: Source) => string | null;
    formatDate: (timestamp: number) => string;
    onSelect: (sourceId: number) => void | Promise<void>;
    onSync: (sourceId: number) => void | Promise<void>;
    onDelete: (sourceId: number) => void | Promise<void>;
  } = $props();

  const capabilities = $derived(sourceCapabilities(source));
  const kindLabel = $derived(sourceKindLabel(source));
  const sourceMembershipLabel = $derived(membershipLabel(source));
  const syncReason = $derived(syncDisabledReason(source));
  const sourceRuntimeStatus = $derived(runtimeStatus(source.accountId));
  const runtimeBadgeLabel = $derived(
    !sourceRuntimeStatus
      ? null
      : sourceRuntimeStatus.status === "restoring"
        ? "restoring..."
        : sourceRuntimeStatus.status === "reauth_required"
          ? "sign in required"
          : sourceRuntimeStatus.status === "restore_failed"
            ? "restore failed"
            : sourceRuntimeStatus.status === "not_initialized"
              ? "account not connected"
              : null
  );

  function sourceInitial() {
    return (source.title ?? source.externalId).trim().charAt(0).toUpperCase() || "#";
  }
</script>

<li class:selected={selected}>
  <div class="source-avatar" aria-hidden="true">
    {#if source.avatarDataUrl}
      <img src={source.avatarDataUrl} alt="" loading="lazy" />
    {:else}
      <span>{sourceInitial()}</span>
    {/if}
  </div>
  <div class="channel-info">
    <button class="source-main" onclick={() => onSelect(source.id)}>
      <span class="title">{source.title ?? source.externalId}</span>
      <span class="sub">{accountLabel(source.accountId)}</span>
    </button>
    <div class="channel-actions">
      <Badge>{kindLabel}</Badge>
      {#if source.lastSyncedAt !== null}
        <Badge>synced {formatDate(source.lastSyncedAt)}</Badge>
      {/if}
      {#if capabilities.requiresAccount && source.accountId !== null}
        {#if runtimeBadgeLabel}
          <Badge
            variant="warning"
            title={sourceRuntimeStatus?.status === "restore_failed" && sourceRuntimeStatus.message
              ? sourceRuntimeStatus.message
              : undefined}
          >
            {runtimeBadgeLabel}
          </Badge>
        {/if}
      {/if}
      {#if capabilities.hasMembershipState && sourceMembershipLabel}
        <Badge variant={source.isMember ? "member" : undefined}>{sourceMembershipLabel}</Badge>
      {/if}
      {#if capabilities.canSync}
        <Button
          size="sm"
          onclick={() => onSync(source.id)}
          disabled={syncing || deleting || syncReason !== null}
          title={syncReason ?? undefined}
        >
          {syncing ? "Syncing..." : "Sync"}
        </Button>
      {/if}
      <Button size="sm" variant="danger-soft" onclick={() => onDelete(source.id)} disabled={deleting || syncing}>
        {deleting ? "Deleting..." : "Delete"}
      </Button>
    </div>
    {#if syncReason}
      <StatusMessage tone="muted" size="sm" surface={false}>
        {syncReason}
      </StatusMessage>
    {/if}
  </div>
</li>

<style>
  li {
    display: flex;
    align-items: flex-start;
    padding: 0.6rem 0.75rem;
    background: var(--panel-strong);
    border-radius: 8px;
    gap: 0.5rem;
    min-width: 0;
  }
  li.selected {
    outline: 1px solid color-mix(in srgb, var(--primary) 45%, transparent);
    background: color-mix(in srgb, var(--primary) 10%, var(--panel-strong));
  }
  .source-avatar {
    flex: 0 0 2.25rem;
    width: 2.25rem;
    height: 2.25rem;
    border-radius: 50%;
    overflow: hidden;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: color-mix(in srgb, var(--primary) 14%, var(--panel-hover));
    color: var(--primary);
    font-size: 0.9rem;
    font-weight: 700;
  }
  .source-avatar img {
    display: block;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .channel-info {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    min-width: 0;
    width: 100%;
  }
  .channel-actions {
    display: flex;
    align-items: flex-start;
    gap: 0.4rem;
    flex-wrap: wrap;
    min-width: 0;
  }
  .source-main {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 0.1rem;
    width: 100%;
    min-width: 0;
    padding: 0;
    border: 0;
    background: transparent;
    color: inherit;
    text-align: left;
    cursor: pointer;
  }
  .title {
    font-size: 0.95rem;
    max-width: 100%;
    overflow-wrap: anywhere;
    word-break: break-word;
  }
  .sub { font-size: 0.75rem; color: var(--muted); }
  @media (max-width: 1200px) {
    li {
      align-items: flex-start;
    }
  }
  @media (max-width: 1024px) {
    .channel-actions {
      max-width: 100%;
    }
  }
</style>
