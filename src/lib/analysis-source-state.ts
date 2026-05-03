import type { AccountRecord, AccountRuntimeStatus } from "$lib/types/accounts";
import type { SourceRecord } from "$lib/types/sources";

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

export function sourceKindLabel(kind: string) {
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

export function membershipLabel(kind: string, isMember: boolean) {
  if (kind === "channel") {
    return isMember ? "subscribed" : "not subscribed";
  }
  return isMember ? "member" : "not a member";
}

export function sourceInitial(source: SourceRecord) {
  return (source.title ?? source.external_id).trim().charAt(0).toUpperCase() || "#";
}

export function sourceSyncDisabledReason(
  source: SourceRecord,
  accountStatuses: Record<number, AccountRuntimeStatus>,
) {
  const runtime = runtimeStatus(source.account_id, accountStatuses);
  if (source.account_id === null) return "Source is not linked to an account.";
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
