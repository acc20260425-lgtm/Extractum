import type { CompanionRunsFilterState } from "$lib/analysis-run-companion-state";
import type { EvidenceSourceScope as SourceScope } from "$lib/analysis-evidence-source-navigation";
import { formatAppError } from "$lib/app-error";

export function createSavedRunsLoadScheduler<TParams>(
  load: (params: TParams, filter: CompanionRunsFilterState) => void | Promise<void>,
  delayMs = 250,
) {
  let timer: ReturnType<typeof setTimeout> | null = null;
  return {
    schedule(params: TParams, filter: CompanionRunsFilterState) {
      if (timer !== null) clearTimeout(timer);
      timer = setTimeout(() => {
        timer = null;
        void load(params, filter);
      }, delayMs);
    },
    dispose() {
      if (timer !== null) clearTimeout(timer);
      timer = null;
    },
  };
}

export function youtubeSyncOptionsForSource(subtype: string | null) {
  const video = subtype === "video";
  return { metadata: true, transcripts: video, comments: video };
}

export function shouldProbeRunSnapshot({ runId }: { runId: number | null; canvasMode: string }) {
  return runId !== null;
}

export function createLatestRequestGate() {
  let sequence = 0;
  let current: { key: string; sequence: number } | null = null;
  return {
    begin(key: string) {
      current = { key, sequence: ++sequence };
      return current;
    },
    isCurrent(request: { key: string; sequence: number }) {
      return current?.key === request.key && current.sequence === request.sequence;
    },
    invalidate() {
      current = null;
    },
    complete(request: { key: string; sequence: number }) {
      return current?.key === request.key && current.sequence === request.sequence;
    },
  };
}

export interface FocusedSourceRequest {
  requestId: string;
  runId: number | null;
  sourceScope: SourceScope;
  sourceViewBasis: "run_snapshot" | "live_source";
  traceRef: string;
}

export interface FocusedSourceCurrent {
  pending: FocusedSourceRequest | null;
  runId: number | null;
  sourceScope: SourceScope | null;
  sourceViewBasis: "run_snapshot" | "live_source";
  selectedTraceRef: string | null;
}

function sourceScopesEqual(left: SourceScope | null, right: SourceScope | null) {
  if (!left || !right || left.kind !== right.kind || left.sourceId !== right.sourceId) return false;
  return left.kind === "source" || right.kind === "source"
    ? left.kind === right.kind
    : left.groupId === right.groupId;
}

export function focusedSourceRequestMatches(request: FocusedSourceRequest, current: FocusedSourceCurrent) {
  return current.pending?.requestId === request.requestId
    && current.pending.traceRef === request.traceRef
    && current.runId === request.runId
    && sourceScopesEqual(current.sourceScope, request.sourceScope)
    && current.sourceViewBasis === request.sourceViewBasis
    && current.selectedTraceRef === request.traceRef;
}

export function focusedSourceLoadExit(
  request: FocusedSourceRequest,
  current: FocusedSourceCurrent,
  outcome: { kind: "success" } | { kind: "missing_target"; reason?: string } | { kind: "failed"; error: unknown },
) {
  if (!focusedSourceRequestMatches(request, current)) {
    return { accepted: false, clearPendingFocus: false, clearHighlight: false, clearLoading: false, status: "" };
  }
  return {
    accepted: true,
    clearPendingFocus: true,
    clearHighlight: outcome.kind !== "success",
    clearLoading: true,
    status: outcome.kind === "success"
      ? ""
      : outcome.kind === "missing_target"
      ? "Selected evidence was not found in the loaded source window."
      : formatAppError("loading selected source evidence", outcome.error),
  };
}

export function createEvidenceRequestSequence() {
  let sequence = 0;
  return {
    next() {
      return `evidence-source-${++sequence}`;
    },
  };
}

export function createRouteTimer() {
  let timer: ReturnType<typeof setTimeout> | null = null;
  return {
    schedule(callback: () => void, delayMs: number) {
      if (timer !== null) clearTimeout(timer);
      timer = setTimeout(() => {
        timer = null;
        callback();
      }, delayMs);
    },
    dispose() {
      if (timer !== null) clearTimeout(timer);
      timer = null;
    },
  };
}

export function clearEvidenceNavigation({
  clearReturnContext,
  clearPendingFocus,
  clearHighlight,
}: {
  clearReturnContext: () => void;
  clearPendingFocus: () => void;
  clearHighlight: () => void;
}) {
  clearReturnContext();
  clearPendingFocus();
  clearHighlight();
}
