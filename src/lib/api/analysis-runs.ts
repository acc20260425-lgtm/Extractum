import { invoke } from "@tauri-apps/api/core";
import { listen, type Event, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  AnalysisRunDetail,
  AnalysisRunEvent,
  AnalysisRunSummary,
  EventEnvelope,
} from "$lib/types/analysis";

export const ANALYSIS_RUN_EVENT = "analysis://run";

export interface ListAnalysisRunsInput {
  sourceId: number | null;
  sourceGroupId: number | null;
  limit: number;
}

export function listAnalysisRuns(input: ListAnalysisRunsInput) {
  return invoke<AnalysisRunSummary[]>("list_analysis_runs", { ...input });
}

export function listActiveAnalysisRuns() {
  return invoke<AnalysisRunSummary[]>("list_active_analysis_runs");
}

export function getAnalysisRun(runId: number) {
  return invoke<AnalysisRunDetail | null>("get_analysis_run", { runId });
}

export function listenToAnalysisRunEvents(
  handler: (event: Event<AnalysisRunEvent>) => void,
): Promise<UnlistenFn> {
  return listen<AnalysisRunEvent>(
    ANALYSIS_RUN_EVENT,
    (event: EventEnvelope<AnalysisRunEvent> & Event<AnalysisRunEvent>) => handler(event),
  );
}
