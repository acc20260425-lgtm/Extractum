import type { AnalysisSnapshotState } from "$lib/types/analysis";

export type SnapshotAffordanceSurface =
  | "runs-row"
  | "opened-header"
  | "run-details"
  | "source-tab"
  | "evidence-tab"
  | "chat-tab";

export type SnapshotAffordanceState =
  | "available"
  | "legacy_missing"
  | "capture_failed_with_error"
  | "not_captured_before_terminal"
  | "capture_failed_without_error_unknown"
  | "inconsistent"
  | "verification_failed"
  | "checking"
  | "pending"
  | "unknown";

export type SnapshotAffordanceSeverity =
  | "none"
  | "info"
  | "warning"
  | "error";

export type SnapshotProbeState =
  | "available"
  | "unavailable"
  | "error"
  | "loading"
  | "unknown";

export type SnapshotBadgeVariant = "neutral" | "info" | "warning" | "danger";
export type SnapshotAvailabilitySignal = "unknown" | "capturing" | "available" | "unavailable";

export interface SnapshotAffordanceInput {
  snapshotState: AnalysisSnapshotState | null;
  snapshotCapturedAt: string | null;
  snapshotError: string | null;
  probeState: SnapshotProbeState;
  runStatus: "queued" | "running" | "completed" | "failed" | "cancelled" | string;
  surface: SnapshotAffordanceSurface;
}

export interface SnapshotAffordance {
  state: SnapshotAffordanceState;
  severity: SnapshotAffordanceSeverity;
  compactLabel: string | null;
  badgeVariant: SnapshotBadgeVariant | null;
  headerWarning: string | null;
  detailTitle: string | null;
  detailDescription: string | null;
  disabledReason: string | null;
  sanitizedError: string | null;
}

export interface SnapshotProbeStateInput {
  snapshotAvailability: SnapshotAvailabilitySignal;
  loadingRunSnapshotMessages: boolean;
  runSnapshotError: string;
}

const AVAILABLE: SnapshotAffordance = {
  state: "available",
  severity: "none",
  compactLabel: null,
  badgeVariant: null,
  headerWarning: null,
  detailTitle: "Snapshot available",
  detailDescription: "Frozen source material captured for this run is available.",
  disabledReason: null,
  sanitizedError: null,
};

export function isActiveRunStatus(status: string): boolean {
  return status === "queued" || status === "running";
}

export function isTerminalRunStatus(status: string): boolean {
  return status === "completed" || status === "failed" || status === "cancelled";
}

export function snapshotProbeStateFromAvailability({
  snapshotAvailability,
  loadingRunSnapshotMessages,
  runSnapshotError,
}: SnapshotProbeStateInput): SnapshotProbeState {
  if (snapshotAvailability === "available") return "available";
  if (runSnapshotError.trim()) return "error";
  if (loadingRunSnapshotMessages) return "loading";
  // Existing RunSnapshotAvailability "capturing" means an active run has no
  // snapshot rows yet; keep that pending instead of treating it as degraded.
  if (snapshotAvailability === "capturing") return "unknown";
  if (snapshotAvailability === "unavailable") return "unavailable";
  return "unknown";
}

export function snapshotAffordanceForRun(input: SnapshotAffordanceInput): SnapshotAffordance {
  const sanitizedError = sanitizeSnapshotError(input.snapshotError);

  if (input.probeState === "available") return AVAILABLE;
  if (input.snapshotState === "missing_legacy") return legacyMissingAffordance();
  if (input.snapshotState === "capture_failed" && sanitizedError) return captureFailedAffordance(sanitizedError);
  if (input.snapshotState === "capture_failed") {
    return input.runStatus === "failed" || input.runStatus === "cancelled"
      ? notCapturedBeforeTerminalAffordance()
      : captureFailedWithoutErrorAffordance();
  }
  if (input.snapshotState === "captured" && input.probeState === "unavailable") {
    return inconsistentAffordance();
  }
  if (input.probeState === "error") return verificationFailedAffordance(sanitizedError);
  if (input.probeState === "loading") return checkingAffordance();
  if (isActiveRunStatus(input.runStatus)) return pendingAffordance();
  if (
    input.snapshotState === null
    && input.probeState === "unavailable"
    && (input.runStatus === "failed" || input.runStatus === "cancelled")
  ) {
    return notCapturedBeforeTerminalAffordance();
  }
  if (input.snapshotState === null && input.probeState === "unavailable") {
    return unknownUnavailableAffordance();
  }
  return checkingAffordance();
}

function sanitizeSnapshotError(value: string | null): string | null {
  const sanitized = (value ?? "").replace(/\s+/g, " ").trim();
  return sanitized ? sanitized : null;
}

function baseAffordance({
  state,
  severity,
  compactLabel,
  badgeVariant,
  headerWarning,
  detailTitle,
  detailDescription,
  disabledReason,
  sanitizedError = null,
}: SnapshotAffordance): SnapshotAffordance {
  return {
    state,
    severity,
    compactLabel,
    badgeVariant,
    headerWarning,
    detailTitle,
    detailDescription,
    disabledReason,
    sanitizedError,
  };
}

function legacyMissingAffordance(): SnapshotAffordance {
  return baseAffordance({
    state: "legacy_missing",
    severity: "warning",
    compactLabel: "Legacy snapshot missing",
    badgeVariant: "warning",
    headerWarning: "Saved report is readable, but this legacy run has no saved source snapshot.",
    detailTitle: "Legacy run has no saved snapshot",
    detailDescription: "Older saved runs may not include frozen source rows, so exact source browsing, evidence source resolution, and follow-up chat stay unavailable.",
    disabledReason: "Exact source resolution is unavailable because this legacy run has no saved source snapshot.",
    sanitizedError: null,
  });
}

function captureFailedAffordance(sanitizedError: string): SnapshotAffordance {
  return baseAffordance({
    state: "capture_failed_with_error",
    severity: "error",
    compactLabel: "Snapshot capture failed",
    badgeVariant: "danger",
    headerWarning: "Saved report is readable, but Extractum could not save the frozen source context for this run.",
    detailTitle: "Snapshot capture failed",
    detailDescription: "Extractum could not save the frozen source context for this run. Exact source browsing, evidence source resolution, and follow-up chat stay unavailable.",
    disabledReason: "Exact source resolution is unavailable because snapshot capture failed for this run.",
    sanitizedError,
  });
}

function notCapturedBeforeTerminalAffordance(): SnapshotAffordance {
  return baseAffordance({
    state: "not_captured_before_terminal",
    severity: "warning",
    compactLabel: "Snapshot not captured",
    badgeVariant: "warning",
    headerWarning: "Saved report is readable, but the run ended before a frozen source snapshot was saved.",
    detailTitle: "Snapshot was not captured before the run ended",
    detailDescription: "The saved report remains readable, but there is no frozen source corpus for exact source browsing, evidence source resolution, or follow-up chat.",
    disabledReason: "Exact source resolution is unavailable because the run ended before a frozen source snapshot was saved.",
    sanitizedError: null,
  });
}

function captureFailedWithoutErrorAffordance(): SnapshotAffordance {
  return baseAffordance({
    state: "capture_failed_without_error_unknown",
    severity: "warning",
    compactLabel: "Snapshot not captured",
    badgeVariant: "warning",
    headerWarning: "Saved report is readable, but saved source context is unavailable for this run.",
    detailTitle: "Saved snapshot is unavailable",
    detailDescription: "Extractum did not record a snapshot capture error, but saved snapshot rows are unavailable for this run.",
    disabledReason: "Exact source resolution is unavailable because saved snapshot rows are unavailable for this run.",
    sanitizedError: null,
  });
}

function inconsistentAffordance(): SnapshotAffordance {
  return baseAffordance({
    state: "inconsistent",
    severity: "error",
    compactLabel: "Snapshot rows unavailable",
    badgeVariant: "danger",
    headerWarning: "Saved report is readable, but the stored snapshot marker is inconsistent with saved rows.",
    detailTitle: "Snapshot rows are unavailable",
    detailDescription: "This run is marked as captured, but Extractum could not load saved snapshot rows for it.",
    disabledReason: "Exact source resolution is unavailable because the run is marked captured but saved snapshot rows are unavailable.",
    sanitizedError: null,
  });
}

function verificationFailedAffordance(sanitizedError: string | null): SnapshotAffordance {
  return baseAffordance({
    state: "verification_failed",
    severity: "error",
    compactLabel: "Snapshot check failed",
    badgeVariant: "danger",
    headerWarning: "Saved report is readable, but Extractum could not verify the saved source snapshot.",
    detailTitle: "Saved snapshot could not be verified",
    detailDescription: "Extractum could not verify saved snapshot rows for this run. Exact source browsing, evidence source resolution, and follow-up chat stay unavailable until verification succeeds.",
    disabledReason: "Exact source resolution is unavailable because Extractum could not verify the saved snapshot rows.",
    sanitizedError,
  });
}

function unknownUnavailableAffordance(): SnapshotAffordance {
  return baseAffordance({
    state: "unknown",
    severity: "warning",
    compactLabel: "Snapshot unavailable",
    badgeVariant: "warning",
    headerWarning: "Saved report is readable, but saved source context is unavailable for this run.",
    detailTitle: "Saved snapshot is unavailable",
    detailDescription: "Saved snapshot rows are unavailable for this run, and the run does not identify the missing context as a legacy snapshot.",
    disabledReason: "Exact source resolution is unavailable because saved snapshot rows are unavailable for this run.",
    sanitizedError: null,
  });
}

function checkingAffordance(): SnapshotAffordance {
  return baseAffordance({
    state: "checking",
    severity: "info",
    compactLabel: null,
    badgeVariant: null,
    headerWarning: null,
    detailTitle: "Checking saved snapshot",
    detailDescription: "Extractum is checking whether frozen source material is available for this run.",
    disabledReason: "Exact source resolution is unavailable until the saved snapshot check finishes.",
    sanitizedError: null,
  });
}

function pendingAffordance(): SnapshotAffordance {
  return baseAffordance({
    state: "pending",
    severity: "info",
    compactLabel: null,
    badgeVariant: null,
    headerWarning: null,
    detailTitle: "Snapshot pending",
    detailDescription: "Snapshot capture is still pending for this active run.",
    disabledReason: "Exact source resolution is unavailable until the run snapshot is captured.",
    sanitizedError: null,
  });
}
