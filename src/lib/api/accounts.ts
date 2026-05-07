import { invoke } from "@tauri-apps/api/core";
import { listen, type Event, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  AccountCodeInput,
  AccountPhoneInput,
  AccountRecord,
  AccountRuntimeStatus,
  CreateAccountInput,
} from "$lib/types/accounts";

export const TELEGRAM_ACCOUNT_STATUS_EVENT = "telegram://account-status";

export function listAccounts() {
  return invoke<AccountRecord[]>("list_accounts");
}

export function getAccount(accountId: number) {
  return invoke<AccountRecord | null>("get_account", { accountId });
}

export function createAccount(input: CreateAccountInput) {
  return invoke<AccountRecord>("create_account", { ...input });
}

export function deleteAccount(accountId: number) {
  return invoke<void>("delete_account", { accountId });
}

export function setAccountPhone(input: AccountPhoneInput) {
  return invoke<void>("set_account_phone", { ...input });
}

export function clearAccountPhone(accountId: number) {
  return invoke<void>("clear_account_phone", { accountId });
}

export function getAccountRuntimeStatuses(accountIds: number[]) {
  return invoke<AccountRuntimeStatus[]>("tg_get_account_statuses", { accountIds });
}

export function initializeTelegramAccount(accountId: number) {
  return invoke<boolean>("tg_init", { accountId });
}

export function sendTelegramCode(input: AccountPhoneInput) {
  return invoke<void>("tg_send_code", { ...input });
}

export function signInTelegramAccount(input: AccountCodeInput) {
  return invoke<void>("tg_sign_in", { ...input });
}

export function logoutTelegramAccount(accountId: number) {
  return invoke<void>("tg_logout", { accountId });
}

export function listenToAccountRuntimeStatus(
  handler: (event: Event<AccountRuntimeStatus>) => void,
): Promise<UnlistenFn> {
  return listen<AccountRuntimeStatus>(TELEGRAM_ACCOUNT_STATUS_EVENT, handler);
}
