import { invoke } from "@tauri-apps/api/core";
import { listen, type Event, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  AskAnalysisRunQuestionInput,
  AnalysisChatEvent,
  AnalysisChatMessage,
  EventEnvelope,
} from "$lib/types/analysis";

export const ANALYSIS_CHAT_EVENT = "analysis://chat";

export function listAnalysisChatMessages(runId: number) {
  return invoke<AnalysisChatMessage[]>("list_analysis_chat_messages", { runId });
}

export function askAnalysisRunQuestion(input: AskAnalysisRunQuestionInput) {
  return invoke<string>("ask_analysis_run_question", { ...input });
}

export function clearAnalysisChatMessages(runId: number) {
  return invoke<void>("clear_analysis_chat_messages", { runId });
}

export function listenToAnalysisChatEvents(
  handler: (event: Event<AnalysisChatEvent>) => void,
): Promise<UnlistenFn> {
  return listen<AnalysisChatEvent>(
    ANALYSIS_CHAT_EVENT,
    (event: EventEnvelope<AnalysisChatEvent> & Event<AnalysisChatEvent>) => handler(event),
  );
}
