import { invoke } from "@tauri-apps/api/core";
import { listen, type Event, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  ListPromptPackRunsInput,
  PreflightYoutubeSummaryRunInput,
  GetPromptPackStageArtifactInput,
  PromptPackAuditEvent,
  PromptPackLibrary,
  PromptPackResult,
  PromptPackRunEvent,
  PromptPackRunSummary,
  PromptPackStageArtifact,
  PromptPackStageArtifactSummary,
  PromptPackStageRun,
  PromptPackValidationFinding,
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

export function getPromptPackResult(runId: number) {
  return invoke<PromptPackResult>("get_prompt_pack_result", { runId });
}

export function getPromptPackValidationFindings(runId: number) {
  return invoke<PromptPackValidationFinding[]>("get_prompt_pack_validation_findings", { runId });
}

export function listPromptPackStageArtifacts(stageRunId: number) {
  return invoke<PromptPackStageArtifactSummary[]>("list_prompt_pack_stage_artifacts", {
    stageRunId,
  });
}

export function getPromptPackStageArtifact(input: GetPromptPackStageArtifactInput) {
  return invoke<PromptPackStageArtifact>("get_prompt_pack_stage_artifact", { ...input });
}

export function listPromptPackAuditEvents(runId: number) {
  return invoke<PromptPackAuditEvent[]>("list_prompt_pack_audit_events", { runId });
}

export function listenToPromptPackRunEvents(
  handler: (event: Event<PromptPackRunEvent>) => void,
): Promise<UnlistenFn> {
  return listen<PromptPackRunEvent>(PROMPT_PACK_RUN_EVENT, handler);
}
