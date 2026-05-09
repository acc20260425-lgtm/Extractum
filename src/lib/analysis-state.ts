import { endOfDayUnix, runTargetLabel, startOfDayUnix } from "$lib/analysis-utils";
import { sourceCapabilities } from "$lib/source-capabilities";
import type {
  AnalysisChunkSummaryEvent,
  AnalysisPromptTemplate,
  AnalysisChatTurn,
  AnalysisReportStartCommand,
  AnalysisRunDetail,
  AnalysisRunEvent,
  AnalysisRunSummary,
  AnalysisSourceGroup,
  AnalysisTraceData,
  AnalysisTraceRef,
  YoutubeCorpusMode,
} from "$lib/types/analysis";
import type {
  ForumTopicFilter,
  SourceItem,
  NotebookLmExportEvent,
  NotebookLmExportRequest,
  NotebookLmExportResult,
  SourceForumTopic,
  Source,
  SyncSourceResult,
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
export type AnalysisSourceSelectionState = {
  analysisScope: "single_source";
  selectedSourceId: string;
  selectedTopicKey: typeof ALL_TOPICS_KEY;
  inspectorMode: "history";
};
export type AnalysisGroupSelectionState = {
  analysisScope: "source_group";
  selectedGroupId: string;
  sourceTopics: SourceForumTopic[];
  selectedTopicKey: typeof ALL_TOPICS_KEY;
  inspectorMode: "history";
};
export type OpenedRunResetState = {
  activeRunId: number | null;
  currentRun: AnalysisRunDetail | null;
  traceData: AnalysisTraceData;
  savedTraceRefs: string[];
  resolvedTraceRefs: string[];
  selectedTraceRef: string | null;
  chatMessages: AnalysisChatTurn[];
  chatQuestion: string;
  chatting: boolean;
  activeChatRequestId: string | null;
  activeChatRunId: number | null;
  liveRuns: Record<number, LiveRunState>;
};
export type SourceDeletionDialog = {
  title: "Delete source?";
  message: string;
  confirmLabel: "Delete";
  cancelLabel: "Cancel";
  tone: "danger";
};
export type RunDeletionDialog = {
  title: "Delete saved run?";
  message: string;
  confirmLabel: "Delete";
  cancelLabel: "Cancel";
  tone: "danger";
};
export type RunDeletionDecision =
  | { ok: true; dialog: RunDeletionDialog }
  | { ok: false; status: string };
export type SourceDeletionResetState = {
  sourceItems: SourceItem[];
  currentRun: AnalysisRunDetail | null;
  activeRunId: number | null;
  traceData: AnalysisTraceData;
  savedTraceRefs: string[];
  resolvedTraceRefs: string[];
  selectedTraceRef: string | null;
  chatMessages: AnalysisChatTurn[];
  chatQuestion: string;
  chatting: boolean;
  activeChatRequestId: string | null;
  activeChatRunId: number | null;
};
export type ActiveRunSyncDecision = {
  activeRunIds: number[];
  preserveRunId: number | null;
  runSnapshots: { runId: number; status: string }[];
  nextActiveRunId: number | null;
  runToOpen: number | null;
};
export type TakeoutImportEventDecision = {
  status: string | null;
  reloadWorkspace: boolean;
  reloadSelectedSourceId: number | null;
};
export type NotebookLmExportFormState = {
  outputDir: string;
  range: "entire_history" | "analysis_period";
  fromDate: string;
  toDate: string;
  includeMediaPlaceholders: boolean;
  minMessageLength: number;
  maxWordsPerFile: number;
  maxBytesPerFile: number;
  overwriteExisting: boolean;
};
export type AnalysisReportStartState = {
  analysisScope: "single_source" | "source_group";
  selectedSourceId: string;
  selectedGroupId: string;
  selectedTemplateId: string;
  periodFrom: string;
  periodTo: string;
  outputLanguage: string;
  profileId: string | null;
  modelOverride: string;
  youtubeCorpusMode: YoutubeCorpusMode;
};
export type AnalysisReportStartDecision =
  | { ok: true; command: AnalysisReportStartCommand }
  | { ok: false; status: string };

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

export function activeRunSyncDecision(
  summaries: Pick<AnalysisRunSummary, "id" | "status">[],
  activeRunId: number | null,
  currentRunId: number | null,
): ActiveRunSyncDecision {
  const activeRunIds = summaries.map((run) => run.id);
  const runSnapshots = summaries.map((run) => ({
    runId: run.id,
    status: run.status,
  }));

  if (currentRunId !== null) {
    return {
      activeRunIds,
      preserveRunId: currentRunId,
      runSnapshots,
      nextActiveRunId: activeRunId,
      runToOpen: null,
    };
  }

  const selectedRunIsStillActive =
    activeRunId !== null && summaries.some((run) => run.id === activeRunId);

  if (!selectedRunIsStillActive && summaries.length > 0) {
    return {
      activeRunIds,
      preserveRunId: null,
      runSnapshots,
      nextActiveRunId: null,
      runToOpen: summaries[0].id,
    };
  }

  return {
    activeRunIds,
    preserveRunId: null,
    runSnapshots,
    nextActiveRunId: summaries.length === 0 ? null : activeRunId,
    runToOpen: null,
  };
}

export function filteredAnalysisRuns(
  runs: AnalysisRunSummary[],
  runFilter: AnalysisRunFilter,
) {
  if (runFilter === "all") return runs;
  return runs.filter((run) => run.status === runFilter);
}

export function filteredAnalysisSourceCatalog(
  sources: Source[],
  railQuery: string,
  accountLabel: (accountId: number | null) => string,
) {
  const query = railQuery.trim().toLocaleLowerCase();
  if (!query) return sources;

  return sources.filter((source) => {
    return (
      (source.title ?? source.externalId).toLocaleLowerCase().includes(query) ||
      accountLabel(source.accountId).toLocaleLowerCase().includes(query)
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

export function selectedAnalysisTemplate(
  selectedTemplateId: string,
  templates: AnalysisPromptTemplate[],
) {
  const templateId = selectedTemplateId ? Number(selectedTemplateId) : null;
  if (templateId === null) return null;
  return templates.find((template) => template.id === templateId) ?? null;
}

export function selectedAnalysisGroup(
  selectedGroupId: string,
  groups: AnalysisSourceGroup[],
) {
  const groupId = selectedGroupId ? Number(selectedGroupId) : null;
  if (groupId === null) return null;
  return groups.find((group) => group.id === groupId) ?? null;
}

export function selectedAnalysisTraceRef(
  selectedTraceRef: string | null,
  refs: AnalysisTraceRef[],
) {
  if (!selectedTraceRef) return null;
  return refs.find((ref) => ref.ref === selectedTraceRef) ?? null;
}

export function analysisSourceSelectionState(
  sourceId: number,
): AnalysisSourceSelectionState {
  return {
    analysisScope: "single_source",
    selectedSourceId: String(sourceId),
    selectedTopicKey: ALL_TOPICS_KEY,
    inspectorMode: "history",
  };
}

export function analysisGroupSelectionState(
  groupId: number,
): AnalysisGroupSelectionState {
  return {
    analysisScope: "source_group",
    selectedGroupId: String(groupId),
    sourceTopics: [],
    selectedTopicKey: ALL_TOPICS_KEY,
    inspectorMode: "history",
  };
}

export function analysisReportStartCommand(
  state: AnalysisReportStartState,
): AnalysisReportStartDecision {
  if (state.analysisScope === "single_source" && !state.selectedSourceId) {
    return { ok: false, status: "Select a source first." };
  }
  if (state.analysisScope === "source_group" && !state.selectedGroupId) {
    return { ok: false, status: "Select a source group first." };
  }
  if (!state.selectedTemplateId) {
    return { ok: false, status: "Select a report template first." };
  }
  if (!state.periodFrom || !state.periodTo) {
    return { ok: false, status: "Select both dates first." };
  }

  const periodFrom = startOfDayUnix(state.periodFrom);
  const periodTo = endOfDayUnix(state.periodTo);
  if (periodFrom > periodTo) {
    return { ok: false, status: "The start date must not be after the end date." };
  }

  const outputLanguage = state.outputLanguage.trim();
  if (!outputLanguage) {
    return { ok: false, status: "Output language cannot be empty." };
  }

  const modelOverride = state.modelOverride.trim();
  const profileId = state.profileId?.trim() || null;

  return {
    ok: true,
    command: {
      sourceId: state.analysisScope === "single_source" ? Number(state.selectedSourceId) : null,
      sourceGroupId: state.analysisScope === "source_group" ? Number(state.selectedGroupId) : null,
      periodFrom,
      periodTo,
      outputLanguage,
      promptTemplateId: Number(state.selectedTemplateId),
      modelOverride: modelOverride ? modelOverride : null,
      profileId,
      youtubeCorpusMode: state.youtubeCorpusMode,
    },
  };
}

export function openedRunResetState(
  runId: number,
  activeRunId: number | null,
  currentRun: AnalysisRunDetail | null,
  liveRuns: Record<number, LiveRunState>,
): OpenedRunResetState | null {
  if (activeRunId !== runId && currentRun?.id !== runId) {
    return null;
  }

  const nextLiveRuns = { ...liveRuns };
  delete nextLiveRuns[runId];

  return {
    activeRunId: null,
    currentRun: null,
    traceData: { refs: [] },
    savedTraceRefs: [],
    resolvedTraceRefs: [],
    selectedTraceRef: null,
    chatMessages: [],
    chatQuestion: "",
    chatting: false,
    activeChatRequestId: null,
    activeChatRunId: null,
    liveRuns: nextLiveRuns,
  };
}

export function runDeletionDecision(run: AnalysisRunSummary): RunDeletionDecision {
  if (isActiveRunStatus(run.status)) {
    return {
      ok: false,
      status: "Cancel or wait for this run before deleting it.",
    };
  }

  return {
    ok: true,
    dialog: {
      title: "Delete saved run?",
      message:
        `The saved report for "${runTargetLabel(run)}" and its follow-up chat history will be removed from this device.`,
      confirmLabel: "Delete",
      cancelLabel: "Cancel",
      tone: "danger",
    },
  };
}

export function runDeletedStatus(run: Pick<AnalysisRunSummary, "id">) {
  return `Saved run ${run.id} deleted.`;
}

export function sourceDisplayName(source: Pick<Source, "title" | "externalId">) {
  return source.title ?? source.externalId;
}

export function sourceDeletionDialog(
  source: Pick<Source, "title" | "externalId">,
): SourceDeletionDialog {
  const sourceName = sourceDisplayName(source);
  return {
    title: "Delete source?",
    message:
      `The source "${sourceName}" and its synced local items will be removed from Extractum.\n\n` +
      "Saved analysis snapshots remain available as frozen run artifacts.",
    confirmLabel: "Delete",
    cancelLabel: "Cancel",
    tone: "danger",
  };
}

export function sourceDeletedStatus(
  source: Pick<Source, "title" | "externalId">,
) {
  return `Source "${sourceDisplayName(source)}" deleted.`;
}

export function sourceActionPending(
  current: Record<number, boolean>,
  sourceId: number,
) {
  return {
    ...current,
    [sourceId]: true,
  };
}

export function clearSourceActionPending(
  current: Record<number, boolean>,
  sourceId: number,
) {
  const next = { ...current };
  delete next[sourceId];
  return next;
}

export function sourceSyncStatus(result: SyncSourceResult) {
  return (
    `Sync complete: inserted ${result.inserted}, skipped ${result.skipped}.` +
    (result.initialSyncPolicyApplied
      ? ` First sync policy applied: ${result.initialSyncPolicyApplied}.`
      : "") +
    (result.warnings.length > 0
      ? ` Warnings: ${result.warnings.join(" ")}`
      : "")
  );
}

export function sourceDeletionResetState(
  deletedSourceId: number,
  selectedSourceId: string,
): SourceDeletionResetState | null {
  if (selectedSourceId !== String(deletedSourceId)) {
    return null;
  }

  return {
    sourceItems: [],
    currentRun: null,
    activeRunId: null,
    traceData: { refs: [] },
    savedTraceRefs: [],
    resolvedTraceRefs: [],
    selectedTraceRef: null,
    chatMessages: [],
    chatQuestion: "",
    chatting: false,
    activeChatRequestId: null,
    activeChatRunId: null,
  };
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

export function takeoutImportEventDecision(
  job: TakeoutImportJobRecord,
  selectedSourceId: string,
): TakeoutImportEventDecision {
  if (job.status === "completed") {
    return {
      status: `Takeout import complete: inserted ${job.inserted}, skipped ${job.skipped}.`,
      reloadWorkspace: true,
      reloadSelectedSourceId: selectedSourceId === String(job.source_id) ? job.source_id : null,
    };
  }

  if (job.status === "failed") {
    return {
      status: job.error ? `Takeout import failed: ${job.error}` : "Takeout import failed.",
      reloadWorkspace: false,
      reloadSelectedSourceId: null,
    };
  }

  if (job.status === "cancelled") {
    return {
      status: job.message ?? "Takeout import cancelled.",
      reloadWorkspace: false,
      reloadSelectedSourceId: null,
    };
  }

  return {
    status: job.message && selectedSourceId === String(job.source_id) ? job.message : null,
    reloadWorkspace: false,
    reloadSelectedSourceId: null,
  };
}

export function hasRealForumTopics(topics: SourceForumTopic[]) {
  return topics.some((topic) => topic.kind === "topic");
}

export function normalizeSelectedTopicKey(
  topics: SourceForumTopic[],
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
  topics: SourceForumTopic[],
): ForumTopicFilter | null {
  if (selectedTopicKey === ALL_TOPICS_KEY) {
    return null;
  }

  const topic = topics.find((entry) => entry.key === selectedTopicKey);
  if (!topic) {
    return null;
  }

  if (topic.kind === "topic" && topic.topicId !== null) {
    return {
      kind: "topic",
      topicId: topic.topicId,
    };
  }

  return {
    kind: "uncategorized",
  };
}

export function shouldShowTopicSelector(
  source: Source | null,
  analysisScope: "single_source" | "source_group",
  loadingSourceTopics: boolean,
  topics: SourceForumTopic[],
) {
  if (!source || analysisScope !== "single_source") {
    return false;
  }

  if (loadingSourceTopics) {
    return sourceCapabilities(source).hasTopics;
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

export function notebookLmExportInitialProgress(): NotebookLmExportProgressState {
  return {
    phase: "loading",
    message: "Preparing NotebookLM export.",
    current: null,
    total: null,
  };
}

export function notebookLmExportRequestFromForm(
  exportId: string,
  sourceId: number,
  form: NotebookLmExportFormState,
): NotebookLmExportRequest {
  return {
    export_id: exportId,
    source_id: sourceId,
    output_dir: form.outputDir.trim(),
    period_from: form.range === "analysis_period" && form.fromDate
      ? startOfDayUnix(form.fromDate)
      : null,
    period_to: form.range === "analysis_period" && form.toDate
      ? endOfDayUnix(form.toDate)
      : null,
    include_media_placeholders: form.includeMediaPlaceholders,
    min_message_length: form.minMessageLength,
    max_words_per_file: form.maxWordsPerFile,
    max_bytes_per_file: form.maxBytesPerFile,
    overwrite_existing: form.overwriteExisting,
  };
}

export function notebookLmExportCompleteStatus(result: NotebookLmExportResult) {
  return `NotebookLM export complete: ${result.files.length} files, ${result.exported_message_count} messages.`;
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
