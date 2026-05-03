import type {
  AnalysisChunkSummaryEvent,
  AnalysisRunDetail,
  AnalysisRunEvent,
  AnalysisRunSummary,
  AnalysisSourceGroup,
  AnalysisTraceRef,
} from "$lib/types/analysis";
import type {
  ForumTopicFilter,
  NotebookLmExportEvent,
  SourceForumTopicRecord,
  SourceRecord,
  TakeoutImportJobRecord,
} from "$lib/types/sources";

export const ALL_TOPICS_KEY = "__all_topics__";

export type LiveRunState = {
  phase: string;
  progress: string;
  queuePosition: number | null;
  chunkSummaries: AnalysisChunkSummaryEvent[];
  streamedOutput: string;
};

export type NotebookLmExportProgressState = {
  phase: NotebookLmExportEvent["phase"];
  message: string;
  current: number | null;
  total: number | null;
};

export type AnalysisTraceRefOrigin = "saved" | "resolved" | "unknown";
export type AnalysisRunFilter = "all" | "completed" | "failed";

export function createEmptyLiveRunState(): LiveRunState {
  return {
    phase: "",
    progress: "",
    queuePosition: null,
    chunkSummaries: [],
    streamedOutput: "",
  };
}

export function isActiveRunStatus(value: string) {
  return value === "queued" || value === "running";
}

export function getLiveRunState(
  liveRuns: Record<number, LiveRunState>,
  runId: number,
) {
  return liveRuns[runId] ?? createEmptyLiveRunState();
}

export function updateLiveRunState(
  liveRuns: Record<number, LiveRunState>,
  runId: number,
  updater: (current: LiveRunState) => LiveRunState,
) {
  const current = getLiveRunState(liveRuns, runId);
  return {
    ...liveRuns,
    [runId]: updater(current),
  };
}

export function syncRunSnapshot(
  liveRuns: Record<number, LiveRunState>,
  runId: number,
  runStatus: string,
) {
  return updateLiveRunState(liveRuns, runId, (current) => ({
    ...current,
    phase: runStatus,
    progress: isActiveRunStatus(runStatus) ? current.progress : "",
    queuePosition: isActiveRunStatus(runStatus) ? current.queuePosition : null,
  }));
}

export function pruneLiveRuns(
  liveRuns: Record<number, LiveRunState>,
  activeRunIds: number[],
  preserveRunId: number | null = null,
) {
  const keepIds = new Set(activeRunIds);
  if (preserveRunId !== null) {
    keepIds.add(preserveRunId);
  }

  return Object.fromEntries(
    Object.entries(liveRuns).filter(([runId]) => keepIds.has(Number(runId))),
  );
}

export function activeAnalysisRunIds(activeRuns: Pick<AnalysisRunSummary, "id">[]) {
  return activeRuns.map((run) => run.id);
}

export function focusedLiveRunState(
  liveRuns: Record<number, LiveRunState>,
  activeRunId: number | null,
) {
  if (activeRunId === null) return null;
  return liveRuns[activeRunId] ?? null;
}

export function liveRunPhase(liveRuns: Record<number, LiveRunState>, runId: number) {
  return liveRuns[runId]?.phase ?? "";
}

export function liveRunProgress(liveRuns: Record<number, LiveRunState>, runId: number) {
  return liveRuns[runId]?.progress ?? "";
}

export function isRunFocused(
  runId: number,
  activeRunId: number | null,
  currentRun: Pick<AnalysisRunDetail, "id"> | null,
) {
  return activeRunId === runId || currentRun?.id === runId;
}

export function runActivePhase(
  focusedLiveRun: LiveRunState | null,
  currentRun: Pick<AnalysisRunDetail, "status"> | null,
) {
  return focusedLiveRun?.phase || currentRun?.status || "";
}

export function runActiveProgress(focusedLiveRun: LiveRunState | null) {
  return focusedLiveRun?.progress || "";
}

export function focusedRunChunkSummaries(focusedLiveRun: LiveRunState | null) {
  return focusedLiveRun?.chunkSummaries ?? [];
}

export function focusedRunStreamedOutput(
  focusedLiveRun: LiveRunState | null,
  currentRun: Pick<AnalysisRunDetail, "result_markdown"> | null,
) {
  if (focusedLiveRun?.streamedOutput) {
    return focusedLiveRun.streamedOutput;
  }

  return currentRun?.result_markdown ?? "";
}

export function isRunActive(activeRunId: number | null, activeRunIds: number[]) {
  return activeRunId !== null && activeRunIds.includes(activeRunId);
}

export function canCancelAnalysisRun(activeRunId: number | null, activeRunIds: number[]) {
  return isRunActive(activeRunId, activeRunIds);
}

export function filteredAnalysisRuns(
  runs: AnalysisRunSummary[],
  runFilter: AnalysisRunFilter,
) {
  if (runFilter === "all") return runs;
  return runs.filter((run) => run.status === runFilter);
}

export function filteredAnalysisSourceCatalog(
  sources: SourceRecord[],
  railQuery: string,
  accountLabel: (accountId: number | null) => string,
) {
  const query = railQuery.trim().toLocaleLowerCase();
  if (!query) return sources;

  return sources.filter((source) => {
    return (
      (source.title ?? source.external_id).toLocaleLowerCase().includes(query) ||
      accountLabel(source.account_id).toLocaleLowerCase().includes(query)
    );
  });
}

export function filteredAnalysisGroups(
  groups: AnalysisSourceGroup[],
  railQuery: string,
) {
  const query = railQuery.trim().toLocaleLowerCase();
  if (!query) return groups;

  return groups.filter((group) => group.name.toLocaleLowerCase().includes(query));
}

export function formatAnalysisRunProgress(
  payload: AnalysisRunEvent,
  currentProgress: string,
) {
  if (payload.progress_current !== null && payload.progress_total !== null) {
    return `${payload.progress_current}/${payload.progress_total}`;
  }

  if (payload.queue_position !== null) {
    return `Queue ${payload.queue_position}`;
  }

  if (
    payload.kind === "completed" ||
    payload.kind === "failed" ||
    payload.kind === "cancelled"
  ) {
    return "";
  }

  return currentProgress;
}

export function applyAnalysisRunEvent(
  current: LiveRunState,
  payload: AnalysisRunEvent,
): LiveRunState {
  const nextSummaries = payload.chunk_summary
    ? [
        ...current.chunkSummaries.filter((chunk) => chunk.index !== payload.chunk_summary?.index),
        payload.chunk_summary,
      ].sort((left, right) => left.index - right.index)
    : current.chunkSummaries;
  const nextPhase =
    payload.kind === "completed" ||
    payload.kind === "failed" ||
    payload.kind === "cancelled"
      ? payload.kind
      : payload.phase || current.phase;

  return {
    phase: nextPhase,
    progress: formatAnalysisRunProgress(payload, current.progress),
    queuePosition: payload.queue_position,
    chunkSummaries: nextSummaries,
    streamedOutput: payload.delta
      ? `${current.streamedOutput}${payload.delta}`
      : current.streamedOutput,
  };
}

export function upsertTakeoutImportJob(
  jobsBySource: Record<number, TakeoutImportJobRecord>,
  job: TakeoutImportJobRecord,
) {
  const current = jobsBySource[job.source_id];
  if (
    current &&
    current.job_id !== job.job_id &&
    current.started_at > job.started_at
  ) {
    return jobsBySource;
  }

  return {
    ...jobsBySource,
    [job.source_id]: job,
  };
}

export function applyTakeoutImportJobs(jobs: TakeoutImportJobRecord[]) {
  let next: Record<number, TakeoutImportJobRecord> = {};
  for (const job of jobs) {
    next = upsertTakeoutImportJob(next, job);
  }
  return next;
}

export function hasRealForumTopics(topics: SourceForumTopicRecord[]) {
  return topics.some((topic) => topic.kind === "topic");
}

export function normalizeSelectedTopicKey(
  topics: SourceForumTopicRecord[],
  preferredKey: string,
) {
  if (!hasRealForumTopics(topics)) {
    return ALL_TOPICS_KEY;
  }

  if (preferredKey === ALL_TOPICS_KEY) {
    return preferredKey;
  }

  return topics.some((topic) => topic.key === preferredKey)
    ? preferredKey
    : ALL_TOPICS_KEY;
}

export function currentTopicFilter(
  selectedTopicKey: string,
  topics: SourceForumTopicRecord[],
): ForumTopicFilter | null {
  if (selectedTopicKey === ALL_TOPICS_KEY) {
    return null;
  }

  const topic = topics.find((entry) => entry.key === selectedTopicKey);
  if (!topic) {
    return null;
  }

  if (topic.kind === "topic" && topic.topic_id !== null) {
    return {
      kind: "topic",
      topic_id: topic.topic_id,
    };
  }

  return {
    kind: "uncategorized",
  };
}

export function shouldShowTopicSelector(
  source: Pick<SourceRecord, "telegram_source_kind"> | null,
  analysisScope: "single_source" | "source_group",
  loadingSourceTopics: boolean,
  topics: SourceForumTopicRecord[],
) {
  if (!source || analysisScope !== "single_source") {
    return false;
  }

  if (loadingSourceTopics) {
    return source.telegram_source_kind === "supergroup";
  }

  return hasRealForumTopics(topics);
}

export function notebookLmExportProgressFromEvent(
  activeExportId: string | null,
  payload: NotebookLmExportEvent,
): { progress: NotebookLmExportProgressState; status: string | null } | null {
  if (payload.export_id !== activeExportId) {
    return null;
  }

  const progress = {
    phase: payload.phase,
    message: payload.message ?? payload.error ?? "",
    current: payload.progress_current,
    total: payload.progress_total,
  };

  if (payload.kind !== "failed") {
    return { progress, status: null };
  }

  return {
    progress,
    status: payload.error
      ? `NotebookLM export failed: ${payload.error}`
      : "NotebookLM export failed.",
  };
}

export function mergeAnalysisTraceRefs(
  currentRefs: AnalysisTraceRef[],
  nextRefs: AnalysisTraceRef[],
): AnalysisTraceRef[] {
  if (nextRefs.length === 0) {
    return currentRefs;
  }

  const merged = [...currentRefs];
  for (const nextRef of nextRefs) {
    if (!merged.some((existing) => existing.ref === nextRef.ref)) {
      merged.push(nextRef);
    }
  }

  return merged.sort((left, right) => left.published_at - right.published_at);
}

export function analysisTraceRefOrigin(
  ref: string,
  savedTraceRefs: string[],
  resolvedTraceRefs: string[],
): AnalysisTraceRefOrigin {
  if (savedTraceRefs.includes(ref)) return "saved";
  if (resolvedTraceRefs.includes(ref)) return "resolved";
  return "unknown";
}
