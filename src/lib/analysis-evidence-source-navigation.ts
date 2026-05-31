import { liveSourceItemRef, youtubeSegmentRef, type SourceReaderItem } from "$lib/source-reader-model";
import type { AnalysisTraceRef } from "$lib/types/analysis";
import type { SourceItem, YoutubeTranscriptSegment } from "$lib/types/sources";

export type EvidenceSourceViewBasis = "run_snapshot" | "live_source";

export type EvidenceSourceScope =
  | { kind: "source"; sourceId: number }
  | { kind: "group_member"; groupId: number; sourceId: number };

export type SourceReturnContext =
  | {
      kind: "evidence";
      runId: number;
      sourceScope: EvidenceSourceScope;
      sourceViewBasis: EvidenceSourceViewBasis;
      traceRef: string;
    }
  | null;

export type PendingEvidenceSourceFocus = {
  requestId: string;
  runId: number;
  sourceScope: EvidenceSourceScope;
  sourceViewBasis: EvidenceSourceViewBasis;
  traceRef: string;
};

export type EvidenceHighlightToken = {
  tokenId: string;
  runId: number;
  sourceScope: EvidenceSourceScope;
  sourceViewBasis: EvidenceSourceViewBasis;
  traceRef: string;
  createdAt: number;
};

export type FocusedLiveSourceTarget =
  | { kind: "source_item"; aroundItemId: number }
  | { kind: "youtube_transcript"; aroundStartMs: number }
  | { kind: "unsupported"; reason: string };

export type LoadedEvidenceSourceData =
  | { kind: "snapshot"; items: SourceReaderItem[] }
  | { kind: "source_items"; items: SourceItem[] }
  | { kind: "youtube_transcript"; segments: YoutubeTranscriptSegment[] };

export function canonicalEvidenceTraceRef(
  highlightedRef: string | null | undefined,
  traceRef: string,
): string {
  return highlightedRef ?? traceRef;
}

export function sourceScopeForEvidence(input: {
  runSourceGroupId: number | null;
  workspaceSourceGroupId: number | null;
  traceSourceId: number;
}): EvidenceSourceScope | null {
  if (input.runSourceGroupId !== null) {
    if (
      input.workspaceSourceGroupId !== null &&
      input.workspaceSourceGroupId !== input.runSourceGroupId
    ) {
      return null;
    }
    return {
      kind: "group_member",
      groupId: input.runSourceGroupId,
      sourceId: input.traceSourceId,
    };
  }

  return { kind: "source", sourceId: input.traceSourceId };
}

export function sourceScopesEqual(
  left: EvidenceSourceScope | null,
  right: EvidenceSourceScope | null,
): boolean {
  if (left === null || right === null) return left === right;
  if (left.kind !== right.kind || left.sourceId !== right.sourceId) return false;
  if (left.kind === "source") return true;
  return right.kind === "group_member" && left.groupId === right.groupId;
}

export function sourceReturnContextIsActive(
  context: SourceReturnContext,
  current: {
    runId: number | null;
    sourceScope: EvidenceSourceScope | null;
    sourceViewBasis: EvidenceSourceViewBasis;
    selectedTraceRef: string | null;
  },
): boolean {
  return (
    context !== null &&
    context.runId === current.runId &&
    context.sourceViewBasis === current.sourceViewBasis &&
    context.traceRef === current.selectedTraceRef &&
    sourceScopesEqual(context.sourceScope, current.sourceScope)
  );
}

export function pendingFocusMatchesCurrent(
  pending: PendingEvidenceSourceFocus | null,
  current: {
    requestId: string;
    runId: number | null;
    sourceScope: EvidenceSourceScope | null;
    sourceViewBasis: EvidenceSourceViewBasis;
    selectedTraceRef: string | null;
  },
): boolean {
  return (
    pending !== null &&
    pending.requestId === current.requestId &&
    pending.runId === current.runId &&
    pending.sourceViewBasis === current.sourceViewBasis &&
    pending.traceRef === current.selectedTraceRef &&
    sourceScopesEqual(pending.sourceScope, current.sourceScope)
  );
}

export function evidenceHighlightMatchesCurrent(
  token: EvidenceHighlightToken | null,
  current: {
    runId: number | null;
    sourceScope: EvidenceSourceScope | null;
    sourceViewBasis: EvidenceSourceViewBasis;
    selectedTraceRef: string | null;
  },
): boolean {
  return (
    token !== null &&
    token.runId === current.runId &&
    token.sourceViewBasis === current.sourceViewBasis &&
    token.traceRef === current.selectedTraceRef &&
    sourceScopesEqual(token.sourceScope, current.sourceScope)
  );
}

export function focusedLiveSourceTargetForTrace(
  trace: Pick<AnalysisTraceRef, "item_id" | "youtube_timestamp_seconds" | "is_synthetic">,
): FocusedLiveSourceTarget {
  if (trace.youtube_timestamp_seconds !== null) {
    if (!Number.isFinite(trace.youtube_timestamp_seconds)) {
      return { kind: "unsupported", reason: "Trace has an invalid YouTube timestamp." };
    }
    return {
      kind: "youtube_transcript",
      aroundStartMs: Math.round(trace.youtube_timestamp_seconds * 1000),
    };
  }

  if (!trace.is_synthetic && trace.item_id > 0) {
    return { kind: "source_item", aroundItemId: trace.item_id };
  }

  return { kind: "unsupported", reason: "Trace has no focusable source item or timestamp." };
}

export function loadedSourceDataContainsTraceRef(
  data: LoadedEvidenceSourceData,
  canonicalTraceRef: string,
  sourceScope: EvidenceSourceScope,
): boolean {
  if (data.kind === "snapshot") {
    return data.items.some((item) => item.ref === canonicalTraceRef && item.sourceId === sourceScope.sourceId);
  }

  if (data.kind === "source_items") {
    return data.items.some((item) => {
      return item.sourceId === sourceScope.sourceId && liveSourceItemRef(item) === canonicalTraceRef;
    });
  }

  return data.segments.some((segment) => {
    return segment.sourceId === sourceScope.sourceId && youtubeSegmentRef(segment) === canonicalTraceRef;
  });
}
