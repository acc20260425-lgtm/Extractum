import { invoke } from "@tauri-apps/api/core";
import type { ApalisJobsListRequest, ApalisJobsListResponse } from "$lib/types/apalis-jobs";

export function loadApalisJobs(request: ApalisJobsListRequest = {}) {
  return invoke<ApalisJobsListResponse>("apalis_jobs_list", { request });
}
