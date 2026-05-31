import type { RunSnapshotAvailability } from "$lib/analysis-report-canvas-state";
import {
  isActiveRunStatus,
  isTerminalRunStatus,
  snapshotAffordanceForRun,
  type SnapshotAffordanceState,
  type SnapshotProbeState,
} from "$lib/analysis-run-snapshot-affordance";
import type {
  CanvasMode,
  CompanionTab,
  SourceViewBasis,
  WorkspaceSelection,
} from "$lib/analysis-workspace-state";
import type { AnalysisRunDetail, AnalysisRunSummary, AnalysisTraceRef } from "$lib/types/analysis";

export type ChatAvailabilityReason =
  | "enabled"
  | "no_run"
  | "pending_completion"
  | "terminal_run"
  | "checking_snapshot"
  | "missing_snapshot"
  | "missing_report"
  | "legacy_missing"
  | "capture_failed_with_error"
  | "not_captured_before_terminal"
  | "capture_failed_without_error_unknown"
  | "inconsistent"
  | "verification_failed"
  | "unknown_snapshot";

export interface ChatAvailability {
  enabled: boolean;
  reason: ChatAvailabilityReason;
  title: string;
  description: string;
}

export type EvidenceSourceActionDecision =
  | {
      kind: "run_snapshot";
      canvasMode: CanvasMode;
      sourceViewBasis: Extract<SourceViewBasis, "run_snapshot">;
      highlightedRef: string;
    }
  | {
      kind: "live_source";
      canvasMode: CanvasMode;
      sourceViewBasis: Extract<SourceViewBasis, "live_source">;
      highlightedRef: string;
      warning: string;
    }
  | {
      kind: "unavailable";
      reason: string;
    };

export type CompanionRunStatusFilter =
  | "all"
  | "completed"
  | "failed"
  | "cancelled"
  | "queued_running";

export interface CompanionRunsFilterState {
  query: string;
  status: CompanionRunStatusFilter;
  scope: "all" | "current";
  dateFrom: string;
  dateTo: string;
  provider: string;
  model: string;
  template: string;
}

export interface CompanionRunEntry {
  kind: "active" | "saved";
  run: AnalysisRunSummary;
}

export function runsFilterDefaults(): CompanionRunsFilterState {
  return {
    query: "",
    status: "all",
    scope: "all",
    dateFrom: "",
    dateTo: "",
    provider: "",
    model: "",
    template: "",
  };
}

export function hasActiveCompanionRunsFilter(filter: CompanionRunsFilterState) {
  const defaults = runsFilterDefaults();
  return filter.query !== defaults.query ||
    filter.status !== defaults.status ||
    filter.scope !== defaults.scope ||
    filter.dateFrom !== defaults.dateFrom ||
    filter.dateTo !== defaults.dateTo ||
    filter.provider !== defaults.provider ||
    filter.model !== defaults.model ||
    filter.template !== defaults.template;
}

export function defaultCompanionTabForOpenedRun(run: AnalysisRunDetail | null): CompanionTab {
  return run?.status === "completed" ? "evidence" : "runs";
}

function chatReasonForSnapshotAffordance(state: SnapshotAffordanceState): ChatAvailabilityReason {
  if (
    state === "legacy_missing"
    || state === "capture_failed_with_error"
    || state === "not_captured_before_terminal"
    || state === "capture_failed_without_error_unknown"
    || state === "inconsistent"
    || state === "verification_failed"
  ) {
    return state;
  }
  if (state === "unknown") return "unknown_snapshot";
  return "missing_snapshot";
}

export function chatAvailabilityForRun({
  currentRun,
  snapshotAvailability,
  snapshotProbeState,
}: {
  currentRun: AnalysisRunDetail | null;
  snapshotAvailability: RunSnapshotAvailability;
  snapshotProbeState: SnapshotProbeState;
}): ChatAvailability {
  if (!currentRun) {
    return {
      enabled: false,
      reason: "no_run",
      title: "Open a completed run",
      description: "Follow-up chat is available after a saved report is open.",
    };
  }

  if (isActiveRunStatus(currentRun.status)) {
    return {
      enabled: false,
      reason: "pending_completion",
      title: "Run still in progress",
      description: "Chat becomes available after the report completes and saved context is available.",
    };
  }

  if (!currentRun.result_markdown?.trim()) {
    return {
      enabled: false,
      reason: "missing_report",
      title: "No saved report",
      description: "This completed run has no saved report output for follow-up chat.",
    };
  }

  if (
    snapshotAvailability === "unavailable"
    || (snapshotAvailability !== "available" && (snapshotProbeState === "unavailable" || snapshotProbeState === "error"))
  ) {
    const affordance = snapshotAffordanceForRun({
      snapshotState: currentRun.snapshot_state,
      snapshotCapturedAt: currentRun.snapshot_captured_at,
      snapshotError: currentRun.snapshot_error,
      probeState: snapshotProbeState,
      runStatus: currentRun.status,
      surface: "chat-tab",
    });

    return {
      enabled: false,
      reason: chatReasonForSnapshotAffordance(affordance.state),
      title: "Saved context unavailable",
      description: affordance.detailDescription ?? "Saved snapshot context is unavailable for this run.",
    };
  }

  if (currentRun.status === "failed" || currentRun.status === "cancelled") {
    return {
      enabled: false,
      reason: "terminal_run",
      title: "Chat is disabled for this run",
      description: "For this MVP, follow-up chat is available only for completed reports.",
    };
  }

  if (snapshotAvailability === "available") {
    return {
      enabled: true,
      reason: "enabled",
      title: "Chat ready",
      description: "Questions use the saved report and saved run snapshot context.",
    };
  }

  return {
    enabled: false,
    reason: "checking_snapshot",
    title: "Checking saved context",
    description: "Chat becomes available when the saved run snapshot has been checked.",
  };
}

export function evidenceSourceActionDecision({
  currentRun,
  selectedTrace,
  snapshotAvailability,
  snapshotProbeState,
}: {
  currentRun: AnalysisRunDetail | null;
  selectedTrace: AnalysisTraceRef | null;
  snapshotAvailability: RunSnapshotAvailability;
  snapshotProbeState: SnapshotProbeState;
}): EvidenceSourceActionDecision {
  if (!currentRun || !selectedTrace) {
    return {
      kind: "unavailable",
      reason: "Select evidence from an opened run before showing it in source.",
    };
  }

  const hasUsableSnapshot = snapshotAvailability === "available" && snapshotProbeState === "available";

  if (snapshotAvailability === "available") {
    return {
      kind: "run_snapshot",
      canvasMode: "source",
      sourceViewBasis: "run_snapshot",
      highlightedRef: selectedTrace.ref,
    };
  }

  if (!hasUsableSnapshot && (isTerminalRunStatus(currentRun.status) || snapshotProbeState === "error")) {
    const affordance = snapshotAffordanceForRun({
      snapshotState: currentRun.snapshot_state,
      snapshotCapturedAt: currentRun.snapshot_captured_at,
      snapshotError: currentRun.snapshot_error,
      probeState: snapshotProbeState,
      runStatus: currentRun.status,
      surface: "evidence-tab",
    });

    return {
      kind: "unavailable",
      reason: affordance.disabledReason ?? "Exact source resolution is unavailable because saved snapshot rows are unavailable for this run.",
    };
  }

  return {
    kind: "live_source",
    canvasMode: "source",
    sourceViewBasis: "live_source",
    highlightedRef: selectedTrace.ref,
    warning: "Showing current source for this in-progress run. This is not the frozen run snapshot.",
  };
}

function normalizedText(value: string | null | undefined) {
  return (value ?? "").trim().toLowerCase();
}

function runSearchText(run: AnalysisRunSummary) {
  return [
    run.scope_label,
    run.source_title,
    run.source_group_name,
    run.prompt_template_name,
    run.provider_profile,
    run.provider,
    run.model,
    run.error,
  ].map(normalizedText).join(" ");
}

function runMatchesStatus(run: AnalysisRunSummary, status: CompanionRunStatusFilter) {
  if (status === "all") return true;
  if (status === "queued_running") return run.status === "queued" || run.status === "running";
  return run.status === status;
}

function runMatchesWorkspace(run: AnalysisRunSummary, selection: WorkspaceSelection) {
  if (selection.kind === "none") return false;
  if (selection.kind === "source") return run.source_id === selection.sourceId;
  return run.source_group_id === selection.sourceGroupId;
}

export function hasSavedRunsForWorkspace({
  savedRuns,
  workspaceSelection,
}: {
  savedRuns: AnalysisRunSummary[];
  workspaceSelection: WorkspaceSelection;
}) {
  return savedRuns.some((run) => runMatchesWorkspace(run, workspaceSelection));
}

function parseDateStart(value: string) {
  const trimmed = value.trim();
  if (!trimmed) return null;
  const time = Date.parse(`${trimmed}T00:00:00Z`);
  return Number.isFinite(time) ? Math.floor(time / 1000) : null;
}

function parseDateEnd(value: string) {
  const trimmed = value.trim();
  if (!trimmed) return null;
  const time = Date.parse(`${trimmed}T23:59:59Z`);
  return Number.isFinite(time) ? Math.floor(time / 1000) : null;
}

export function filterCompanionRuns({
  activeRuns,
  savedRuns,
  filter,
  workspaceSelection,
}: {
  activeRuns: AnalysisRunSummary[];
  savedRuns: AnalysisRunSummary[];
  filter: CompanionRunsFilterState;
  workspaceSelection: WorkspaceSelection;
}): CompanionRunEntry[] {
  const queryTerms = normalizedText(filter.query).split(/\s+/).filter(Boolean);
  const provider = normalizedText(filter.provider);
  const model = normalizedText(filter.model);
  const template = normalizedText(filter.template);
  const from = parseDateStart(filter.dateFrom);
  const to = parseDateEnd(filter.dateTo);

  return [
    ...activeRuns.map((run): CompanionRunEntry => ({ kind: "active", run })),
    ...savedRuns.map((run): CompanionRunEntry => ({ kind: "saved", run })),
  ].filter(({ run }) => {
    if (filter.scope === "current" && !runMatchesWorkspace(run, workspaceSelection)) {
      return false;
    }
    if (!runMatchesStatus(run, filter.status)) {
      return false;
    }
    if (from !== null && run.created_at < from) {
      return false;
    }
    if (to !== null && run.created_at > to) {
      return false;
    }
    if (provider && !normalizedText(run.provider).includes(provider)) {
      return false;
    }
    if (model && !normalizedText(run.model).includes(model)) {
      return false;
    }
    if (template && !normalizedText(run.prompt_template_name).includes(template)) {
      return false;
    }

    const haystack = runSearchText(run);
    return queryTerms.every((term) => haystack.includes(term));
  }).sort((left, right) => right.run.created_at - left.run.created_at);
}
