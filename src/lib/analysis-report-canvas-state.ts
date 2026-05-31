import type {
  AnalysisRunDetail,
  AnalysisRunMessagesPage,
  YoutubeCorpusMode,
} from "$lib/types/analysis";
import type { SourceViewBasis } from "$lib/analysis-workspace-state";
import { isActiveRunStatus } from "$lib/analysis-run-snapshot-affordance";
export { isActiveRunStatus } from "$lib/analysis-run-snapshot-affordance";

export type RunSnapshotAvailability =
  | "unknown"
  | "capturing"
  | "available"
  | "unavailable";

export type SourceCanvasSurface =
  | "live_source"
  | "run_snapshot_unknown"
  | "run_snapshot_pending"
  | "run_snapshot_available"
  | "run_snapshot_unavailable";

export interface SnapshotAvailabilityInput {
  currentRun: Pick<AnalysisRunDetail, "status"> | null;
  page: Pick<AnalysisRunMessagesPage, "messages"> | null;
  loading: boolean;
  errorMessage: string;
}

export interface SourceBasisInput {
  currentRun: Pick<AnalysisRunDetail, "status"> | null;
  sourceViewBasis: SourceViewBasis;
  snapshotAvailability: RunSnapshotAvailability;
}

export function runSnapshotAvailabilityFromPage({
  currentRun,
  page,
  loading,
  errorMessage,
}: SnapshotAvailabilityInput): RunSnapshotAvailability {
  if (!currentRun) return "unknown";
  if (errorMessage.trim()) return "unavailable";
  if (loading) return "unknown";
  if (page === null) return "unknown";
  if (page.messages.length > 0) return "available";
  return isActiveRunStatus(currentRun.status) ? "capturing" : "unavailable";
}

export function sourceCanvasSurface({
  currentRun,
  sourceViewBasis,
  snapshotAvailability,
}: SourceBasisInput): SourceCanvasSurface {
  if (!currentRun || sourceViewBasis === "live_source") {
    return "live_source";
  }

  if (snapshotAvailability === "available") return "run_snapshot_available";
  if (snapshotAvailability === "capturing") return "run_snapshot_pending";
  if (snapshotAvailability === "unavailable") return "run_snapshot_unavailable";
  return "run_snapshot_unknown";
}

export function sourceBasisLabel({
  currentRun,
  sourceViewBasis,
  snapshotAvailability,
}: SourceBasisInput) {
  if (!currentRun || sourceViewBasis === "live_source") {
    return "Live source";
  }

  if (snapshotAvailability === "available") return "Snapshot available";
  if (snapshotAvailability === "capturing") return "Snapshot pending";
  if (snapshotAvailability === "unavailable") return "Snapshot unavailable";
  return "Checking snapshot";
}

export function sourceBasisDescription({
  currentRun,
  sourceViewBasis,
  snapshotAvailability,
}: SourceBasisInput) {
  if (!currentRun) {
    return "Browsing the currently selected live source context.";
  }

  if (sourceViewBasis === "live_source") {
    return "Browsing live source data while the opened run remains bound to its saved report context.";
  }

  if (snapshotAvailability === "available") {
    return "Frozen source material captured for this run is available.";
  }

  if (snapshotAvailability === "capturing") {
    return "Snapshot capture is still in progress for this run.";
  }

  if (snapshotAvailability === "unavailable") {
    return "No frozen source snapshot is available for this run.";
  }

  return "Checking whether a frozen source snapshot is available for this run.";
}

export function canReturnToRunSnapshot(availability: RunSnapshotAvailability) {
  return availability === "available";
}

export function youtubeCorpusModeLabel(value: YoutubeCorpusMode | null | undefined) {
  if (value === "transcript_only") return "Transcript";
  if (value === "transcript_description") return "Transcript + description";
  if (value === "transcript_description_comments") return "Transcript + description + comments";
  return "Not recorded";
}
