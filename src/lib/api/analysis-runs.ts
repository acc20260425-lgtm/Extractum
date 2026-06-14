import { invoke } from "@tauri-apps/api/core";
import { listen, type Event, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  AnalysisReportStartCommand,
  AnalysisRunDetail,
  AnalysisRunEvent,
  AnalysisRunMessagesPage,
  AnalysisRunSummary,
  EventEnvelope,
  ListAnalysisRunMessagesInput,
  ListAnalysisRunsInput,
} from "$lib/types/analysis";

export const ANALYSIS_RUN_EVENT = "analysis://run";

export function listAnalysisRuns(input: ListAnalysisRunsInput) {
  return invoke<AnalysisRunSummary[]>("list_analysis_runs", { ...input });
}

export function listActiveAnalysisRuns() {
  return invoke<AnalysisRunSummary[]>("list_active_analysis_runs");
}

export function getAnalysisRun(runId: number) {
  return invoke<AnalysisRunDetail | null>("get_analysis_run", { runId });
}

export function listAnalysisRunMessages(input: ListAnalysisRunMessagesInput) {
  return invoke<AnalysisRunMessagesPage>("list_analysis_run_messages", { ...input });
}

export function startAnalysisReport(command: AnalysisReportStartCommand) {
  return invoke<number>("start_analysis_report", { ...command });
}

export { startProjectAnalysis } from "$lib/api/projects";

export function cancelAnalysisRun(runId: number) {
  return invoke<void>("cancel_analysis_run", { runId });
}

export function deleteAnalysisRun(runId: number) {
  return invoke<void>("delete_analysis_run", { runId });
}

export function listenToAnalysisRunEvents(
  handler: (event: Event<AnalysisRunEvent>) => void,
): Promise<UnlistenFn> {
  return listen<AnalysisRunEvent>(
    ANALYSIS_RUN_EVENT,
    (event: EventEnvelope<AnalysisRunEvent> & Event<AnalysisRunEvent>) => handler(event),
  );
}
