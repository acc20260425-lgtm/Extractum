import { invoke } from "@tauri-apps/api/core";
import type { AnalysisSourceOption } from "$lib/types/analysis";

export {
  getAccountRuntimeStatuses as getWorkspaceAccountStatuses,
  listAccounts as listWorkspaceAccounts,
} from "$lib/api/accounts";

export function listAnalysisSources() {
  return invoke<AnalysisSourceOption[]>("list_analysis_sources");
}
