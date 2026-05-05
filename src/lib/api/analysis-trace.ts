import { invoke } from "@tauri-apps/api/core";
import type { AnalysisTraceData, AnalysisTraceRef } from "$lib/types/analysis";

export function getAnalysisRunTrace(runId: number) {
  return invoke<AnalysisTraceData>("get_analysis_run_trace", { runId });
}

export function resolveAnalysisTraceRefs(runId: number, refs: string[]) {
  return invoke<AnalysisTraceRef[]>("resolve_analysis_trace_refs", { runId, refs });
}
