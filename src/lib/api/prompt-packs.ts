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
  UpdatePromptPackRunInput,
  YoutubeSummaryPreflightResponse,
} from "$lib/types/prompt-packs";
import {
  YOUTUBE_SUMMARY_SMOKE_FIXTURE_ACTIVE_RUN_ID,
  YOUTUBE_SUMMARY_SMOKE_FIXTURE_LABEL,
  YOUTUBE_SUMMARY_SMOKE_FIXTURE_RUN_ID,
  YOUTUBE_SUMMARY_SMOKE_FIXTURE_SOURCE_ID,
  isYoutubeSummarySmokeFixtureEnabled,
} from "$lib/ui/youtube-summary-smoke-fixture";

export const PROMPT_PACK_RUN_EVENT = "prompt-pack-run-event";
const SMOKE_FIXTURE_ENV_FLAG = "VITE_YOUTUBE_SUMMARY_SMOKE_FIXTURE";
const SMOKE_FIXTURE_VISIBLE_LABEL = "YouTube Summary Smoke Fixture";

export function getPromptPackLibrary() {
  return invoke<PromptPackLibrary>("get_prompt_pack_library");
}

export function preflightYoutubeSummaryRun(input: PreflightYoutubeSummaryRunInput) {
  if (isYoutubeSummarySmokeFixtureEnabled(import.meta.env) && includesSmokeFixtureSource(input.sourceIds)) {
    return Promise.resolve(smokePreflightResponse());
  }
  return invoke<YoutubeSummaryPreflightResponse>("preflight_youtube_summary_run", { ...input });
}

export function startYoutubeSummaryRun(input: StartYoutubeSummaryRunInput) {
  if (isYoutubeSummarySmokeFixtureEnabled(import.meta.env) && includesSmokeFixtureSource(input.sourceIds)) {
    return Promise.resolve<StartYoutubeSummaryRunOutcome>({
      kind: "blocked",
      preflight: {
        ...smokePreflightResponse(),
        includedVideos: [],
        blockingFailures: [
          {
            sourceId: YOUTUBE_SUMMARY_SMOKE_FIXTURE_SOURCE_ID,
            reason: "smoke_fixture_blocked_start",
            message: `${YOUTUBE_SUMMARY_SMOKE_FIXTURE_LABEL} blocked-start state for UI verification.`,
          },
        ],
      },
    });
  }
  return invoke<StartYoutubeSummaryRunOutcome>("start_youtube_summary_run", { ...input });
}

export function cancelPromptPackRun(runId: number) {
  if (isYoutubeSummarySmokeFixtureEnabled(import.meta.env) && isSmokeFixtureRun(runId)) {
    return Promise.resolve();
  }
  return invoke<void>("cancel_prompt_pack_run", { runId });
}

export function updatePromptPackRun(input: UpdatePromptPackRunInput) {
  return invoke<PromptPackRunSummary>("update_prompt_pack_run", { ...input });
}

export function deletePromptPackRun(runId: number) {
  return invoke<void>("delete_prompt_pack_run", { runId });
}

export function listPromptPackRuns(input?: ListPromptPackRunsInput) {
  if (isYoutubeSummarySmokeFixtureEnabled(import.meta.env)) {
    return Promise.resolve([smokeRecentRun(input?.projectId ?? null)]);
  }
  return invoke<PromptPackRunSummary[]>("list_prompt_pack_runs", { ...input });
}

export function listActivePromptPackRuns() {
  if (isYoutubeSummarySmokeFixtureEnabled(import.meta.env)) {
    return Promise.resolve([smokeActiveRun()]);
  }
  return invoke<PromptPackRunSummary[]>("list_active_prompt_pack_runs");
}

export function listPromptPackRunStages(runId: number) {
  return invoke<PromptPackStageRun[]>("list_prompt_pack_run_stages", { runId });
}

export function getPromptPackResult(runId: number) {
  if (isYoutubeSummarySmokeFixtureEnabled(import.meta.env) && isSmokeFixtureRun(runId)) {
    return Promise.resolve<PromptPackResult>({
      runId,
      resultStatus: "complete",
      canonical: smokeCanonicalResult(runId),
      storageWarning: null,
    });
  }
  return invoke<PromptPackResult>("get_prompt_pack_result", { runId });
}

export function getPromptPackValidationFindings(runId: number) {
  if (isYoutubeSummarySmokeFixtureEnabled(import.meta.env) && isSmokeFixtureRun(runId)) {
    return Promise.resolve<PromptPackValidationFinding[]>([
      {
        runId,
        stageRunId: null,
        severity: "warning",
        code: "smoke_fixture_validation",
        message: `${SMOKE_FIXTURE_ENV_FLAG} deterministic validation finding.`,
        objectPath: "$.claims[0]",
        createdAt: "2026-06-14T00:06:00Z",
      },
    ]);
  }
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
  if (isYoutubeSummarySmokeFixtureEnabled(import.meta.env)) {
    const activeTimer = window.setTimeout(() => {
      handler({ payload: smokeRunEvent("progress", "running", "stage", "Smoke fixture running") } as Event<PromptPackRunEvent>);
    }, 300);
    const terminalTimer = window.setTimeout(() => {
      handler({ payload: smokeRunEvent("completed", "complete", "terminal", "Smoke fixture completed") } as Event<PromptPackRunEvent>);
    }, 900);
    return Promise.resolve(() => {
      window.clearTimeout(activeTimer);
      window.clearTimeout(terminalTimer);
    });
  }
  return listen<PromptPackRunEvent>(PROMPT_PACK_RUN_EVENT, handler);
}

function includesSmokeFixtureSource(sourceIds: number[]) {
  return sourceIds.includes(YOUTUBE_SUMMARY_SMOKE_FIXTURE_SOURCE_ID);
}

function isSmokeFixtureRun(runId: number) {
  return runId === YOUTUBE_SUMMARY_SMOKE_FIXTURE_RUN_ID || runId === YOUTUBE_SUMMARY_SMOKE_FIXTURE_ACTIVE_RUN_ID;
}

function smokePreflightResponse(): YoutubeSummaryPreflightResponse {
  return {
    packId: "youtube_summary",
    packVersion: "1.0.0",
    includedVideos: [
      {
        sourceId: YOUTUBE_SUMMARY_SMOKE_FIXTURE_SOURCE_ID,
        videoId: "extractum-fixture",
        title: SMOKE_FIXTURE_VISIBLE_LABEL,
        estimatedInputTokens: 1820,
      },
    ],
    skippedVideos: [
      {
        sourceId: YOUTUBE_SUMMARY_SMOKE_FIXTURE_SOURCE_ID,
        videoId: "extractum-fixture-comment-gap",
        title: "Fixture comments shard",
        reason: "comments_not_available",
      },
    ],
    blockingFailures: [],
    estimatedInputTokens: 1820,
    selectedModelInputLimit: 32000,
  };
}

function smokeActiveRun(): PromptPackRunSummary {
  return {
    runId: YOUTUBE_SUMMARY_SMOKE_FIXTURE_ACTIVE_RUN_ID,
    projectId: null,
    packId: "youtube_summary",
    packVersion: "1.0.0",
    runStatus: "running",
    resultStatus: "none",
    runLabel: "Smoke active run",
    createdAt: "2026-06-14T00:04:00Z",
    startedAt: "2026-06-14T00:04:05Z",
    completedAt: null,
    latestMessage: `${YOUTUBE_SUMMARY_SMOKE_FIXTURE_LABEL} active run`,
    progressCurrent: 1,
    progressTotal: 2,
    queuePosition: null,
  };
}

function smokeRecentRun(projectId: number | null): PromptPackRunSummary {
  return {
    runId: YOUTUBE_SUMMARY_SMOKE_FIXTURE_RUN_ID,
    projectId,
    packId: "youtube_summary",
    packVersion: "1.0.0",
    runStatus: "complete",
    resultStatus: "complete",
    runLabel: "Smoke terminal result",
    createdAt: "2026-06-14T00:01:00Z",
    startedAt: "2026-06-14T00:01:05Z",
    completedAt: "2026-06-14T00:03:00Z",
    latestMessage: `${YOUTUBE_SUMMARY_SMOKE_FIXTURE_LABEL} terminal result`,
    progressCurrent: 2,
    progressTotal: 2,
    queuePosition: null,
  };
}

function smokeRunEvent(
  kind: PromptPackRunEvent["kind"],
  runStatus: PromptPackRunEvent["runStatus"],
  phase: PromptPackRunEvent["phase"],
  message: string,
): PromptPackRunEvent {
  return {
    runId: YOUTUBE_SUMMARY_SMOKE_FIXTURE_ACTIVE_RUN_ID,
    requestId: "smoke-fixture-request",
    kind,
    runStatus,
    phase,
    stageRunId: null,
    stageName: "youtube_summary/transcript_analysis",
    sourceSnapshotId: YOUTUBE_SUMMARY_SMOKE_FIXTURE_SOURCE_ID,
    queuePosition: null,
    progressCurrent: runStatus === "complete" ? 2 : 1,
    progressTotal: 2,
    message,
    error: null,
  };
}

function smokeCanonicalResult(runId: number) {
  return {
    schema_version: "1.0",
    result_id: `smoke_result_${runId}`,
    run_id: runId,
    pack_id: "youtube_summary",
    pack_version: "1.0.0",
    output_language: "en",
    outputs: {
      pack_data: {
        youtube_summary: {
          videos: [
            {
              video_id: "video_1",
              source_ref_id: "source_ref_smoke",
              provider_video_id: "extractum-fixture",
              title: YOUTUBE_SUMMARY_SMOKE_FIXTURE_LABEL,
              summary_text:
                "A deterministic UI fixture summary with enough text to verify wrapping in the result viewer.",
            },
          ],
        },
      },
    },
    source_refs: [
      {
        source_ref_id: "source_ref_smoke",
        source_snapshot_id: YOUTUBE_SUMMARY_SMOKE_FIXTURE_SOURCE_ID,
        title: YOUTUBE_SUMMARY_SMOKE_FIXTURE_LABEL,
      },
    ],
    claims: [
      {
        claim_id: "claim_1",
        source_ref_id: "source_ref_smoke",
        text: "Prompt Pack runs can render canonical YouTube Summary results.",
      },
    ],
    evidence: [
      {
        evidence_id: "evidence_1",
        source_ref_id: "source_ref_smoke",
        text: "Fixture transcript segment with stable evidence text.",
      },
    ],
    warnings: [{ code: "partial_coverage", message: "One fixture comments shard was skipped." }],
    limitations: [{ code: "fixture_data", message: "This result uses deterministic dev-only fixture data." }],
    quality_flags: [{ code: "smoke_fixture", message: "Used only when the explicit smoke fixture flag is set." }],
  };
}
