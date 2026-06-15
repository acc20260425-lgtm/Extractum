import type {
  PromptPackRunEvent,
  PromptPackRunListItem,
  PromptPackRunStatus,
  YoutubeSummaryPreflightResponse,
} from "$lib/types/prompt-packs";

export interface YoutubeSummaryPartitionSummary {
  includedCount: number;
  skippedCount: number;
  blockingCount: number;
  hasPartialCoverage: boolean;
}

export function canStartYoutubeSummary(preflight: YoutubeSummaryPreflightResponse | null): boolean {
  return Boolean(preflight && preflight.includedVideos.length > 0 && preflight.blockingFailures.length === 0);
}

export function summarizePreflightPartitions(
  preflight: Pick<YoutubeSummaryPreflightResponse, "includedVideos" | "skippedVideos" | "blockingFailures">,
): YoutubeSummaryPartitionSummary {
  return {
    includedCount: preflight.includedVideos.length,
    skippedCount: preflight.skippedVideos.length,
    blockingCount: preflight.blockingFailures.length,
    hasPartialCoverage: preflight.includedVideos.length > 0 && preflight.skippedVideos.length > 0,
  };
}

export function updateRunListFromEvent(
  runs: PromptPackRunListItem[],
  event: PromptPackRunEvent,
): PromptPackRunListItem[] {
  const nextRun: PromptPackRunListItem = {
    runId: event.runId,
    runStatus: event.runStatus,
    latestMessage: event.message ?? event.error ?? null,
    progressCurrent: event.progressCurrent,
    progressTotal: event.progressTotal,
    queuePosition: event.queuePosition,
  };
  const existing = runs.find((run) => run.runId === event.runId);
  if (!existing) return [nextRun, ...runs];
  return runs.map((run) => (run.runId === event.runId ? { ...run, ...nextRun } : run));
}

export function retainSelectedRunId(
  selectedRunId: number | null,
  runs: Pick<PromptPackRunListItem, "runId">[],
): number | null {
  if (selectedRunId === null) return null;
  return runs.some((run) => run.runId === selectedRunId) ? selectedRunId : null;
}

export function shouldApplyRunEventToRunsPanel(
  runs: Pick<PromptPackRunListItem, "runId">[],
  event: PromptPackRunEvent,
  projectId: number | null,
): boolean {
  return projectId === null || runs.some((run) => run.runId === event.runId);
}

export function statusLabel(status: PromptPackRunStatus): string {
  return {
    queued: "Queued",
    running: "Running",
    complete: "Complete",
    partial: "Partial",
    failed: "Failed",
    cancelled: "Cancelled",
    interrupted: "Interrupted",
  }[status];
}
