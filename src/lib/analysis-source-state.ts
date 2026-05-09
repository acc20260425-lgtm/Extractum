import type { AccountRecord, AccountRuntimeStatus } from "$lib/types/accounts";
import type { Source } from "$lib/types/sources";
import type { YoutubeRuntimeStatus } from "$lib/types/youtube";
import {
  membershipLabel as sourceMembershipLabel,
  sourceCapabilities,
  sourceKindLabel as providerSourceKindLabel,
} from "$lib/source-capabilities";

export function accountLabel(
  accountId: number | null,
  accounts: AccountRecord[],
) {
  if (accountId === null) return "No account";
  return accounts.find((account) => account.id === accountId)?.label ?? `Account #${accountId}`;
}

export function runtimeStatus(
  accountId: number | null,
  accountStatuses: Record<number, AccountRuntimeStatus>,
) {
  if (accountId === null) return null;
  return accountStatuses[accountId] ?? null;
}

export function runtimeBadge(runtime: AccountRuntimeStatus | null) {
  if (!runtime) return "";
  if (runtime.status === "restoring") return "restoring";
  if (runtime.status === "reauth_required") return "sign-in needed";
  if (runtime.status === "restore_failed") return "restore failed";
  if (runtime.status === "not_initialized") return "offline";
  return "";
}

export function sourceKindLabel(source: Source) {
  return providerSourceKindLabel(source);
}

export function membershipLabel(source: Source) {
  return sourceMembershipLabel(source);
}

export function sourceInitial(source: Source) {
  return (source.title ?? source.externalId).trim().charAt(0).toUpperCase() || "#";
}

export function sourceSyncDisabledReason(
  source: Source,
  accountStatuses: Record<number, AccountRuntimeStatus>,
  youtubeRuntimeStatus: YoutubeRuntimeStatus | null = null,
) {
  const capabilities = sourceCapabilities(source);
  if (!capabilities.canSync) return "This source type is not syncable.";

  if (source.sourceType === "youtube") {
    if (youtubeRuntimeStatus && !youtubeRuntimeStatus.ytdlpAvailable) {
      return youtubeRuntimeStatus.message || "yt-dlp is not available on PATH.";
    }
    return null;
  }

  if (!capabilities.requiresAccount) return null;

  const runtime = runtimeStatus(source.accountId, accountStatuses);
  if (source.accountId === null) return "Source is not linked to an account.";
  if (!runtime || runtime.status === "not_initialized") {
    return "Initialize this account before syncing.";
  }
  if (runtime.status === "restoring") {
    return "This account is still restoring.";
  }
  if (runtime.status === "reauth_required") {
    return "Sign in to this account again before syncing.";
  }
  if (runtime.status === "restore_failed") {
    return runtime.message ?? "The saved Telegram session could not be restored.";
  }
  return null;
}
