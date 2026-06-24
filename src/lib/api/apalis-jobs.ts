import { invoke } from "@tauri-apps/api/core";
import type {
  ApalisJobsListRequest,
  ApalisJobsListResponse,
  ApalisJobsPruneTerminalRequest,
  ApalisJobsPruneTerminalResponse,
} from "$lib/types/apalis-jobs";

export const APALIS_OLD_TERMINAL_PRUNE_HOURS = 24;

export function loadApalisJobs(request: ApalisJobsListRequest = {}) {
  return invoke<ApalisJobsListResponse>("apalis_jobs_list", { request });
}

export function pruneOldTerminalApalisJobs(
  request: ApalisJobsPruneTerminalRequest = {
    olderThanHours: APALIS_OLD_TERMINAL_PRUNE_HOURS,
  },
) {
  return invoke<ApalisJobsPruneTerminalResponse>("apalis_jobs_prune_terminal", { request });
}
