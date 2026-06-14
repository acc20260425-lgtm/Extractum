import { invoke } from "@tauri-apps/api/core";
import { listen, type Event, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  ListPromptPackRunsInput,
  PreflightYoutubeSummaryRunInput,
  PromptPackLibrary,
  PromptPackRunEvent,
  PromptPackRunSummary,
  PromptPackStageRun,
  StartYoutubeSummaryRunInput,
  StartYoutubeSummaryRunOutcome,
  YoutubeSummaryPreflightResponse,
} from "$lib/types/prompt-packs";

export const PROMPT_PACK_RUN_EVENT = "prompt-pack-run-event";

export function getPromptPackLibrary() {
  return invoke<PromptPackLibrary>("get_prompt_pack_library");
}

export function preflightYoutubeSummaryRun(input: PreflightYoutubeSummaryRunInput) {
  return invoke<YoutubeSummaryPreflightResponse>("preflight_youtube_summary_run", { ...input });
}

export function startYoutubeSummaryRun(input: StartYoutubeSummaryRunInput) {
  return invoke<StartYoutubeSummaryRunOutcome>("start_youtube_summary_run", { ...input });
}

export function cancelPromptPackRun(runId: number) {
  return invoke<void>("cancel_prompt_pack_run", { runId });
}

export function listPromptPackRuns(input?: ListPromptPackRunsInput) {
  return invoke<PromptPackRunSummary[]>("list_prompt_pack_runs", { ...input });
}

export function listActivePromptPackRuns() {
  return invoke<PromptPackRunSummary[]>("list_active_prompt_pack_runs");
}

export function listPromptPackRunStages(runId: number) {
  return invoke<PromptPackStageRun[]>("list_prompt_pack_run_stages", { runId });
}

export function listenToPromptPackRunEvents(
  handler: (event: Event<PromptPackRunEvent>) => void,
): Promise<UnlistenFn> {
  return listen<PromptPackRunEvent>(PROMPT_PACK_RUN_EVENT, handler);
}
