import { invoke } from "@tauri-apps/api/core";
import type { AccountRecord, AccountRuntimeStatus } from "$lib/types/accounts";
import type { AnalysisSourceOption } from "$lib/types/analysis";

export function listWorkspaceAccounts() {
  return invoke<AccountRecord[]>("list_accounts");
}

export function getWorkspaceAccountStatuses(accountIds: number[]) {
  return invoke<AccountRuntimeStatus[]>("tg_get_account_statuses", { accountIds });
}

export function listAnalysisSources() {
  return invoke<AnalysisSourceOption[]>("list_analysis_sources");
}
