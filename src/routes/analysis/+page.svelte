<script lang="ts">
  import { onMount } from "svelte";
  import { open as openDialog } from "@tauri-apps/plugin-dialog";
  import StatusMessage from "$lib/components/ui/StatusMessage.svelte";
  import WorkspaceInspector from "$lib/components/analysis/workspace-inspector.svelte";
  import ReportCanvas from "$lib/components/analysis/report-canvas.svelte";
  import CompactSourceRail from "$lib/components/analysis/compact-source-rail.svelte";
  import SourceManagementDialog from "$lib/components/analysis/source-management-dialog.svelte";
  import { formatAppError } from "$lib/app-error";
  import {
    cancelAnalysisRun,
    deleteAnalysisRun,
    getAnalysisRun,
    listActiveAnalysisRuns,
    listAnalysisRunMessages,
    listAnalysisRuns,
    listenToAnalysisRunEvents,
    startAnalysisReport,
  } from "$lib/api/analysis-runs";
  import {
    askAnalysisRunQuestion,
    clearAnalysisChatMessages,
    listAnalysisChatMessages,
    listenToAnalysisChatEvents,
  } from "$lib/api/analysis-chat";
  import {
    getAnalysisRunTrace,
    resolveAnalysisTraceRefs,
  } from "$lib/api/analysis-trace";
  import {
    getWorkspaceAccountStatuses,
    listAnalysisSources,
    listWorkspaceAccounts,
  } from "$lib/api/analysis-workspace";
  import {
    createAnalysisPromptTemplate,
    createAnalysisSourceGroup,
    deleteAnalysisPromptTemplate,
    deleteAnalysisSourceGroup,
    listAnalysisPromptTemplates,
    listAnalysisSourceGroups,
    updateAnalysisPromptTemplate,
    updateAnalysisSourceGroup,
  } from "$lib/api/analysis-source-groups";
  import {
    cancelLlmRequest,
    getLlmProfiles,
    listLlmProviderModels,
  } from "$lib/api/llm";
  import {
    cancelTakeoutSourceImport,
    listTakeoutSourceImportJobs,
    listenToTakeoutImportEvents,
    startTakeoutSourceImport,
  } from "$lib/api/takeout-import";
  import {
    listSourceJobs,
    listenToSourceJobEvents,
    cancelSourceJob,
    retryFailedYoutubePlaylistVideos,
    syncYoutubePlaylistVideo,
    syncYoutubeSource,
    type SourceJobRecord,
  } from "$lib/api/source-jobs";
  import {
    getYoutubePlaylistDetail,
    getYoutubeRuntimeStatus,
    getYoutubeVideoDetail,
    listYoutubeSourceSummaries,
  } from "$lib/api/youtube-detail";
  import {
    exportSourceToNotebookLm,
    listenToNotebookLmExportEvents,
  } from "$lib/api/notebooklm-export";
  import {
    deleteSource as deleteSourceCommand,
    listSourceForumTopics,
    listSourceItems,
    listYoutubeTranscriptSegments,
    listSources,
    syncSource,
  } from "$lib/api/sources";
  import {
    createAnalysisRunWorkflow,
    type AnalysisRunRequestGuard,
    type AnalysisRunWorkflowPatch,
  } from "$lib/analysis-run-workflow";
  import {
    createAnalysisChatWorkflow,
    type AnalysisChatWorkflowPatch,
  } from "$lib/analysis-chat-workflow";
  import {
    createAnalysisTraceWorkflow,
    type AnalysisTraceWorkflowPatch,
  } from "$lib/analysis-trace-workflow";
  import {
    createAnalysisWorkspaceWorkflow,
    type AnalysisWorkspaceWorkflowPatch,
  } from "$lib/analysis-workspace-workflow";
  import {
    createAnalysisSourceGroupsWorkflow,
    type AnalysisSourceGroupsWorkflowPatch,
  } from "$lib/analysis-source-groups-workflow";
  import {
    defaultDateOffset,
    endOfDayUnix,
    formatPeriod,
    formatTimestamp,
    phaseLabel,
    reportLines,
    runTargetLabel,
    startOfDayUnix,
    statusTone,
  } from "$lib/analysis-utils";
  import { openConfirmModal } from "$lib/modals";
  import {
    applyAnalysisRunEvent,
    applyTakeoutImportJobs,
    analysisGroupSelectionState,
    analysisSourceSelectionState,
    analysisTraceRefOrigin as traceRefOriginFromState,
    activeAnalysisRunIds,
    canCancelAnalysisRun,
    createEmptyLiveRunState,
    currentTopicFilter as currentTopicFilterFromState,
    filteredAnalysisGroups,
    filteredAnalysisRuns,
    filteredAnalysisSourceCatalog,
    focusedLiveRunState,
    focusedRunChunkSummaries,
    focusedRunStreamedOutput,
    hasRealForumTopics as hasRealForumTopicsInState,
    isRunActive,
    liveRunPhase,
    liveRunProgress,
    notebookLmExportCompleteStatus,
    notebookLmExportInitialProgress,
    notebookLmExportProgressFromEvent,
    notebookLmExportRequestFromForm,
    openedRunResetState,
    normalizeSelectedTopicKey as normalizeTopicKey,
    pruneLiveRuns as pruneLiveRunMap,
    runActivePhase,
    runActiveProgress,
    selectedAnalysisGroup,
    selectedAnalysisTemplate,
    selectedAnalysisTraceRef,
    clearSourceActionPending,
    sourceActionPending,
    shouldShowTopicSelector as shouldShowTopicSelectorFromState,
    sourceDeletedStatus,
    sourceDeletionDialog,
    sourceDeletionResetState,
    sourceSyncStatus,
    syncRunSnapshot as syncLiveRunSnapshot,
    takeoutImportEventDecision,
    upsertTakeoutImportJob,
    type AnalysisRunFilter,
    type LiveRunState,
    type NotebookLmExportProgressState,
  } from "$lib/analysis-state";
  import {
    groupEditorStateFromGroup,
    isGroupSourceSelected as groupSourceIsSelected,
    templateEditorStateFromTemplate,
    toggleGroupSourceSelection,
  } from "$lib/analysis-editor-state";
  import {
    analysisHistoryScopeParams,
    currentAnalysisGroup,
    currentAnalysisScopeSummary,
    currentAnalysisScopeTitle,
    currentAnalysisSource,
    currentAnalysisSourceMetric,
  } from "$lib/analysis-scope-state";
  import {
    runSnapshotAvailabilityFromPage,
    type RunSnapshotAvailability,
  } from "$lib/analysis-report-canvas-state";
  import {
    defaultAnalysisWorkspaceUiState,
    legacyScopeFromWorkspaceSelection,
    openRunWorkspaceState,
    selectSourceGroupWorkspace,
    selectSourceWorkspace,
    type AnalysisWorkspaceUiState,
    type CanvasMode,
    type WorkspaceSelection,
  } from "$lib/analysis-workspace-state";
  import {
    fallbackWorkspaceSelection,
    loadPersistedAnalysisWorkspaceState,
    persistableAnalysisWorkspaceState,
    restoredUiStateFromPersisted,
    savePersistedAnalysisWorkspaceState,
  } from "$lib/analysis-workspace-persistence";
  import {
    accountLabel as formatAccountLabel,
    runtimeBadge,
    runtimeStatus as getRuntimeStatus,
    sourceInitial,
    sourceSyncDisabledReason as getSourceSyncDisabledReason,
  } from "$lib/analysis-source-state";
  import { sourceCapabilities } from "$lib/source-capabilities";
  import type { AccountRecord, AccountRuntimeStatus } from "$lib/types/accounts";
  import type { LlmProfile, LlmProviderModel } from "$lib/types/llm";
  import type {
    AnalysisGroupSourceType,
    AnalysisChatTurn,
    AnalysisPromptTemplate,
    AnalysisRunDetail,
    AnalysisRunEvent,
    AnalysisRunMessage,
    AnalysisRunMessageCursor,
    AnalysisRunMessagesPage,
    AnalysisRunSummary,
    AnalysisSourceGroup,
    AnalysisSourceOption,
    AnalysisTraceData,
    YoutubeCorpusMode,
  } from "$lib/types/analysis";
  import type {
    ForumTopicFilter,
    NotebookLmExportEvent,
    NotebookLmExportResult,
    Source,
    SourceForumTopic,
    SourceItem,
    TakeoutImportEvent,
    TakeoutImportJobRecord,
    YoutubeTranscriptSegment,
    YoutubeTranscriptSegmentCursor,
  } from "$lib/types/sources";
  import type {
    YoutubePlaylistDetail,
    YoutubeRuntimeStatus,
    YoutubeSourceSummary,
    YoutubeVideoDetail,
  } from "$lib/types/youtube";
  import type { NotebookLmExportForm } from "$lib/components/analysis/notebooklm-export-dialog.svelte";

  type InspectorMode = "active" | "history" | "trace" | "chunks";
  const PROFILE_DEFAULT_MODEL_OPTION = "__profile_default__";
  const CUSTOM_MODEL_OPTION = "__custom_model__";

  function createNotebookLmExportId() {
    if (typeof crypto !== "undefined" && typeof crypto.randomUUID === "function") {
      return crypto.randomUUID();
    }
    return `notebooklm-${Date.now()}-${Math.random().toString(36).slice(2)}`;
  }

  let sourceCatalog = $state<Source[]>([]);
  let sourceMetrics = $state<Record<number, AnalysisSourceOption>>({});
  let sourceItems = $state<SourceItem[]>([]);
  let sourceTopics = $state<SourceForumTopic[]>([]);
  let youtubeTranscriptSegments = $state<YoutubeTranscriptSegment[]>([]);
  let youtubeTranscriptCursor = $state<YoutubeTranscriptSegmentCursor | null>(null);
  let youtubeTranscriptHasMore = $state(false);
  let youtubeTranscriptSearch = $state("");
  let loadingYoutubeTranscriptSegments = $state(false);
  let youtubeTranscriptRequestKey = "";
  let groupLiveItemsBySource = $state<Record<number, SourceItem[]>>({});
  let groupLiveCursorsBySource = $state<Record<number, number | null>>({});
  let groupLiveHasMoreBySource = $state<Record<number, boolean>>({});
  let groupLiveLoadingBySource = $state<Record<number, boolean>>({});
  let selectedGroupSourceId = $state<number | null>(null);
  let selectedSnapshotSourceId = $state<number | null>(null);
  let accounts = $state<AccountRecord[]>([]);
  let accountStatuses = $state<Record<number, AccountRuntimeStatus>>({});
  let youtubeRuntimeStatus = $state<YoutubeRuntimeStatus | null>(null);
  let youtubeSummaries = $state<Record<number, YoutubeSourceSummary>>({});
  let youtubeVideoDetail = $state<YoutubeVideoDetail | null>(null);
  let youtubePlaylistDetail = $state<YoutubePlaylistDetail | null>(null);
  let llmProfiles = $state<LlmProfile[]>([]);
  let activeLlmProfile = $state("default");
  let selectedLlmProfileId = $state("");
  let selectedLlmModel = $state(PROFILE_DEFAULT_MODEL_OPTION);
  let customModelOverride = $state("");
  let llmProviderModels = $state<LlmProviderModel[]>([]);
  let templates = $state<AnalysisPromptTemplate[]>([]);
  let runs = $state<AnalysisRunSummary[]>([]);
  let activeRuns = $state<AnalysisRunSummary[]>([]);
  let groups = $state<AnalysisSourceGroup[]>([]);

  let loadingSourceCatalog = $state(false);
  let loadingItems = $state(false);
  let loadingSourceTopics = $state(false);
  let loadingYoutubeDetail = $state(false);
  let loadingLlmProviderModels = $state(false);
  let loadingTemplates = $state(false);
  let loadingRuns = $state(false);
  let loadingActiveRuns = $state(false);
  let loadingGroups = $state(false);
  let loadingRunDetail = $state(false);
  let loadingChat = $state(false);
  let runSnapshotAvailability = $state<RunSnapshotAvailability>("unknown");
  let runSnapshotMessages = $state<AnalysisRunMessage[]>([]);
  let runSnapshotCursor = $state<AnalysisRunMessageCursor | null>(null);
  let runSnapshotHasMore = $state(false);
  let loadingRunSnapshotMessages = $state(false);
  let runSnapshotError = $state("");
  let runSnapshotPage: AnalysisRunMessagesPage | null = null;
  let lastSnapshotLoadKey = "";

  let railQuery = $state("");
  let inspectorMode = $state<InspectorMode>("history");
  let workspaceUiState = $state<AnalysisWorkspaceUiState>(
    defaultAnalysisWorkspaceUiState(),
  );
  let workspacePersistenceReady = $state(false);
  let restoredWorkspaceSelection = $state<WorkspaceSelection | null>(null);

  let selectedSourceId = $state("");
  let selectedTopicKey = $state("__all_topics__");
  let selectedTemplateId = $state("");
  let selectedGroupId = $state("");
  let analysisScope = $state<"single_source" | "source_group">("single_source");
  let periodFrom = $state(defaultDateOffset(-30));
  let periodTo = $state(defaultDateOffset(0));
  let outputLanguage = $state("Russian");
  let youtubeCorpusMode = $state<YoutubeCorpusMode>("transcript_description");
  let llmModelStatus = $state("");
  let templateName = $state("");
  let templateBody = $state("");
  let editorBoundTemplateId = $state<number | null>(null);
  let savingTemplate = $state(false);
  let deletingTemplate = $state(false);
  let groupName = $state("");
  let groupSourceType = $state<AnalysisGroupSourceType>("telegram");
  let groupMemberSourceIds = $state<number[]>([]);
  let editorBoundGroupId = $state<number | null>(null);
  let savingGroup = $state(false);
  let deletingGroup = $state(false);

  let status = $state("");
  let startingReport = $state(false);
  let activeRunId = $state<number | null>(null);
  let liveRuns = $state<Record<number, LiveRunState>>({});
  let currentRun = $state<AnalysisRunDetail | null>(null);
  let traceData = $state<AnalysisTraceData>({ refs: [] });
  let selectedTraceRef = $state<string | null>(null);
  let savedTraceRefs = $state<string[]>([]);
  let resolvedTraceRefs = $state<string[]>([]);
  let runFilter = $state<AnalysisRunFilter>("all");
  let historyScope = $state<"all" | "current">("all");
  let chatQuestion = $state("");
  let chatMessages = $state<AnalysisChatTurn[]>([]);
  let chatting = $state(false);
  let activeChatRequestId = $state<string | null>(null);
  let activeChatRunId = $state<number | null>(null);
  let clearingChat = $state(false);
  let syncingIds = $state<Record<number, boolean>>({});
  let deletingSourceIds = $state<Record<number, boolean>>({});
  let startingTakeoutSourceIds = $state<Record<number, boolean>>({});
  let takeoutJobsBySource = $state<Record<number, TakeoutImportJobRecord>>({});
  let sourceJobsBySource = $state<Record<number, SourceJobRecord[]>>({});
  let deletingRunIds = $state<Record<number, boolean>>({});
  let sourceManagerOpen = $state(false);
  let exportDialogOpen = $state(false);
  let exportingNotebookLm = $state(false);
  let activeNotebookLmExportId = $state<string | null>(null);
  let notebookLmExportProgress = $state<NotebookLmExportProgressState | null>(null);
  let notebookLmExportResult = $state<NotebookLmExportResult | null>(null);
  let notebookLmExportForm = $state<NotebookLmExportForm>({
    outputDir: "",
    range: "entire_history",
    fromDate: "",
    toDate: "",
    includeMediaPlaceholders: true,
    minMessageLength: 3,
    maxWordsPerFile: 300000,
    maxBytesPerFile: 50000000,
    overwriteExisting: false,
  });
  let statusTimer: ReturnType<typeof setTimeout> | null = null;
  let llmModelsRequestKey = "";

  function isErrorStatus(value: string) {
    return value.startsWith("Error") || value.startsWith("Analysis failed");
  }

  function upsertTakeoutJob(job: TakeoutImportJobRecord) {
    takeoutJobsBySource = upsertTakeoutImportJob(takeoutJobsBySource, job);
  }

  function applyTakeoutJobs(jobs: TakeoutImportJobRecord[]) {
    takeoutJobsBySource = applyTakeoutImportJobs(jobs);
  }

  function applyTakeoutImportEvent(job: TakeoutImportEvent) {
    upsertTakeoutJob(job);

    const decision = takeoutImportEventDecision(job, selectedSourceId);

    if (decision.status !== null) {
      status = decision.status;
    }

    if (decision.reloadWorkspace) {
      void Promise.all([loadSourceCatalog(), loadActiveRuns(), loadRuns()]);
    }

    if (decision.reloadSelectedSourceId !== null) {
      const sourceId = decision.reloadSelectedSourceId;
      const source = sourceCatalog.find((candidate) => candidate.id === sourceId);
      void Promise.all([
        source && sourceCapabilities(source).hasTopics
          ? loadSourceTopics(sourceId, { preserveSelection: true })
          : Promise.resolve(),
        loadItems(sourceId),
      ]);
    }
  }

  function isActiveSourceJob(job: SourceJobRecord) {
    return job.status === "queued" || job.status === "running" || job.status === "cancel_requested";
  }

  function applySourceJob(job: SourceJobRecord) {
    sourceJobsBySource = {
      ...sourceJobsBySource,
      [job.source_id]: [
        job,
        ...(sourceJobsBySource[job.source_id] ?? []).filter(
          (existing) => existing.job_id !== job.job_id,
        ),
      ],
    };
  }

  function currentSource() {
    return currentAnalysisSource(selectedSourceId, sourceCatalog);
  }

  function currentSourceMetric() {
    return currentAnalysisSourceMetric(currentSource(), sourceMetrics);
  }

  function currentSourceJobs() {
    const source = currentSource();
    return source ? sourceJobsBySource[source.id] ?? [] : [];
  }

  function currentGroup() {
    return currentAnalysisGroup(selectedGroupId, groups);
  }

  function currentScopeTitle() {
    return currentAnalysisScopeTitle(analysisScope, currentSource(), currentGroup());
  }

  function currentScopeSummary() {
    return currentAnalysisScopeSummary(
      analysisScope,
      currentSource(),
      currentGroup(),
      currentSourceMetric(),
    );
  }

  function accountLabel(accountId: number | null) {
    return formatAccountLabel(accountId, accounts);
  }

  function runtimeStatus(accountId: number | null) {
    return getRuntimeStatus(accountId, accountStatuses);
  }

  function sourceSyncDisabledReason(source: Source) {
    return getSourceSyncDisabledReason(source, accountStatuses, youtubeRuntimeStatus);
  }

  function dedupeProviderModels(models: LlmProviderModel[]) {
    const unique: LlmProviderModel[] = [];
    for (const model of models) {
      if (!unique.some((entry) => entry.model === model.model)) {
        unique.push(model);
      }
    }
    return unique;
  }

  function runProfileId() {
    return selectedLlmProfileId.trim() || null;
  }

  function runModelOverride() {
    if (selectedLlmModel === CUSTOM_MODEL_OPTION) {
      return customModelOverride.trim();
    }
    if (selectedLlmModel === PROFILE_DEFAULT_MODEL_OPTION) {
      return "";
    }
    return selectedLlmModel;
  }

  function profileForModelLookup() {
    const profileId = selectedLlmProfileId || activeLlmProfile || llmProfiles[0]?.profile_id;
    return llmProfiles.find((profile) => profile.profile_id === profileId) ?? llmProfiles[0] ?? null;
  }

  async function loadRunProviderModels() {
    const profile = profileForModelLookup();
    if (!profile) {
      llmProviderModels = [];
      llmModelStatus = "";
      loadingLlmProviderModels = false;
      return;
    }

    const requestKey = `${profile.profile_id}:${profile.provider}:${profile.default_model}`;
    llmModelsRequestKey = requestKey;
    loadingLlmProviderModels = true;
    llmModelStatus = "";

    try {
      const models = await listLlmProviderModels({
        provider: profile.provider,
        profileId: profile.profile_id,
      });
      if (llmModelsRequestKey !== requestKey) {
        return;
      }
      llmProviderModels = dedupeProviderModels(models);
      if (
        selectedLlmModel !== PROFILE_DEFAULT_MODEL_OPTION &&
        selectedLlmModel !== CUSTOM_MODEL_OPTION &&
        !llmProviderModels.some((model) => model.model === selectedLlmModel)
      ) {
        selectedLlmModel = PROFILE_DEFAULT_MODEL_OPTION;
      }
    } catch (error) {
      if (llmModelsRequestKey !== requestKey) {
        return;
      }
      llmProviderModels = [];
      llmModelStatus = formatAppError(`loading ${profile.provider} models`, error);
    } finally {
      if (llmModelsRequestKey === requestKey) {
        loadingLlmProviderModels = false;
      }
    }
  }

  async function loadLlmProfiles() {
    try {
      const state = await getLlmProfiles();
      llmProfiles = state.profiles;
      activeLlmProfile = state.active_profile || "default";
      if (
        selectedLlmProfileId &&
        !state.profiles.some((profile) => profile.profile_id === selectedLlmProfileId)
      ) {
        selectedLlmProfileId = "";
      }
      await loadRunProviderModels();
    } catch (error) {
      status = formatAppError("loading LLM profiles", error);
    }
  }

  function changeLlmProfile(value: string) {
    selectedLlmProfileId = value;
    selectedLlmModel = PROFILE_DEFAULT_MODEL_OPTION;
    customModelOverride = "";
    void loadRunProviderModels();
  }

  function changeLlmModel(value: string) {
    selectedLlmModel = value;
    if (value !== CUSTOM_MODEL_OPTION) {
      customModelOverride = "";
    }
  }

  function applyTraceWorkflowPatch(patch: AnalysisTraceWorkflowPatch) {
    if ("traceData" in patch) traceData = patch.traceData ?? { refs: [] };
    if ("savedTraceRefs" in patch) savedTraceRefs = patch.savedTraceRefs ?? [];
    if ("resolvedTraceRefs" in patch) resolvedTraceRefs = patch.resolvedTraceRefs ?? [];
    if ("selectedTraceRef" in patch) selectedTraceRef = patch.selectedTraceRef ?? null;
    if ("inspectorMode" in patch && patch.inspectorMode) inspectorMode = patch.inspectorMode;
    if ("status" in patch && patch.status !== undefined) status = patch.status;
  }

  function clearTraceState() {
    traceWorkflow.clearState();
  }

  function clearChatState() {
    chatWorkflow.clearState();
  }

  function applyChatWorkflowPatch(patch: AnalysisChatWorkflowPatch) {
    if ("chatMessages" in patch) chatMessages = patch.chatMessages ?? [];
    if ("chatQuestion" in patch) chatQuestion = patch.chatQuestion ?? "";
    if ("chatting" in patch) chatting = patch.chatting ?? false;
    if ("activeChatRequestId" in patch) activeChatRequestId = patch.activeChatRequestId ?? null;
    if ("activeChatRunId" in patch) activeChatRunId = patch.activeChatRunId ?? null;
    if ("loadingChat" in patch) loadingChat = patch.loadingChat ?? false;
    if ("clearingChat" in patch) clearingChat = patch.clearingChat ?? false;
    if ("status" in patch && patch.status !== undefined) status = patch.status;
  }

  function clearOpenedRunState(runId: number) {
    const next = openedRunResetState(runId, activeRunId, currentRun, liveRuns);
    if (next === null) {
      return;
    }

    runWorkflow.invalidateOpenRunRequests();
    activeRunId = next.activeRunId;
    currentRun = next.currentRun;
    traceData = next.traceData;
    savedTraceRefs = next.savedTraceRefs;
    resolvedTraceRefs = next.resolvedTraceRefs;
    selectedTraceRef = next.selectedTraceRef;
    chatMessages = next.chatMessages;
    chatQuestion = next.chatQuestion;
    chatting = next.chatting;
    activeChatRequestId = next.activeChatRequestId;
    activeChatRunId = next.activeChatRunId;
    liveRuns = next.liveRuns;
  }

  function getLiveRunState(runId: number) {
    return liveRuns[runId] ?? createEmptyLiveRunState();
  }

  function updateLiveRunState(
    runId: number,
    updater: (current: LiveRunState) => LiveRunState,
  ) {
    liveRuns = {
      ...liveRuns,
      [runId]: updater(getLiveRunState(runId)),
    };
  }

  function syncRunSnapshot(runId: number, runStatus: string) {
    liveRuns = syncLiveRunSnapshot(liveRuns, runId, runStatus);
  }

  function pruneLiveRuns(activeRunIds: number[], preserveRunId: number | null = null) {
    liveRuns = pruneLiveRunMap(liveRuns, activeRunIds, preserveRunId);
  }

  function livePhase(runId: number) {
    return liveRunPhase(liveRuns, runId);
  }

  function liveProgress(runId: number) {
    return liveRunProgress(liveRuns, runId);
  }

  function currentTopicFilter(): ForumTopicFilter | null {
    return currentTopicFilterFromState(selectedTopicKey, sourceTopics);
  }

  function hasRealForumTopics(topics: SourceForumTopic[] = sourceTopics) {
    return hasRealForumTopicsInState(topics);
  }

  function shouldShowTopicSelector() {
    return shouldShowTopicSelectorFromState(
      currentSource(),
      analysisScope,
      loadingSourceTopics,
      sourceTopics,
    );
  }

  function normalizeSelectedTopicKey(
    topics: SourceForumTopic[],
    preferredKey: string,
  ) {
    return normalizeTopicKey(topics, preferredKey);
  }

  function applyNotebookLmExportEvent(payload: NotebookLmExportEvent) {
    const next = notebookLmExportProgressFromEvent(activeNotebookLmExportId, payload);
    if (next === null) {
      return;
    }

    notebookLmExportProgress = next.progress;
    if (next.status) {
      status = next.status;
    }
  }

  function applyRunEvent(payload: AnalysisRunEvent) {
    updateLiveRunState(payload.run_id, (current) => applyAnalysisRunEvent(current, payload));
  }

  const activeRunIds = $derived.by(() => activeAnalysisRunIds(activeRuns));

  const focusedLiveRun = $derived.by(() => focusedLiveRunState(liveRuns, activeRunId));

  const activePhase = $derived.by(() => runActivePhase(focusedLiveRun, currentRun));
  const activeProgress = $derived.by(() => runActiveProgress(focusedLiveRun));
  const focusedChunkSummaries = $derived.by(() => focusedRunChunkSummaries(focusedLiveRun));
  const focusedStreamedOutput = $derived.by(() => focusedRunStreamedOutput(
    focusedLiveRun,
    currentRun,
  ));

  const selectedRunIsActive = $derived.by(
    () => isRunActive(activeRunId, activeRunIds),
  );

  const canCancelCurrentRun = $derived.by(
    () => canCancelAnalysisRun(activeRunId, activeRunIds),
  );

  const selectedTemplate = $derived.by(() => selectedAnalysisTemplate(
    selectedTemplateId,
    templates,
  ));

  const selectedGroup = $derived.by(() => selectedAnalysisGroup(selectedGroupId, groups));

  const selectedTrace = $derived.by(() => selectedAnalysisTraceRef(
    selectedTraceRef,
    traceData.refs,
  ));

  const historyScopeParams = $derived.by(() => {
    return analysisHistoryScopeParams(
      historyScope,
      analysisScope,
      selectedSourceId,
      selectedGroupId,
    );
  });

  const filteredRuns = $derived.by(() => filteredAnalysisRuns(runs, runFilter));

  const filteredSourceCatalog = $derived.by(() => {
    return filteredAnalysisSourceCatalog(sourceCatalog, railQuery, accountLabel);
  });

  const filteredGroups = $derived.by(() => filteredAnalysisGroups(groups, railQuery));

  function restorePersistedWorkspaceState() {
    if (typeof window === "undefined") {
      return;
    }

    const persisted = loadPersistedAnalysisWorkspaceState(window.localStorage);
    if (!persisted) {
      workspacePersistenceReady = true;
      return;
    }

    const restored = restoredUiStateFromPersisted(persisted);
    workspaceUiState = restored;
    restoredWorkspaceSelection = restored.workspaceSelection;
    historyScope = persisted.runs.historyScope;
    runFilter = persisted.runs.runFilter;
    workspacePersistenceReady = true;
  }

  function persistWorkspaceState() {
    if (typeof window === "undefined" || !workspacePersistenceReady) {
      return;
    }

    savePersistedAnalysisWorkspaceState(window.localStorage,
      persistableAnalysisWorkspaceState(workspaceUiState, {
        historyScope,
        runFilter,
      }),
    );
  }

  function applyWorkspaceUiState(next: AnalysisWorkspaceUiState) {
    workspaceUiState = next;
  }

  function changeCanvasMode(mode: CanvasMode) {
    applyWorkspaceUiState({
      ...workspaceUiState,
      canvasMode: mode,
    });
  }

  function viewLiveSourceForOpenedRun() {
    applyWorkspaceUiState({
      ...workspaceUiState,
      canvasMode: "source",
      sourceViewBasis: "live_source",
    });
  }

  function backToRunSnapshot() {
    applyWorkspaceUiState({
      ...workspaceUiState,
      canvasMode: "source",
      sourceViewBasis: "run_snapshot",
    });
  }

  function resetRunSnapshotState() {
    runSnapshotAvailability = "unknown";
    runSnapshotMessages = [];
    runSnapshotCursor = null;
    runSnapshotHasMore = false;
    loadingRunSnapshotMessages = false;
    runSnapshotError = "";
    runSnapshotPage = null;
    lastSnapshotLoadKey = "";
  }

  function resetYoutubeTranscriptReader() {
    youtubeTranscriptSegments = [];
    youtubeTranscriptCursor = null;
    youtubeTranscriptHasMore = false;
    loadingYoutubeTranscriptSegments = false;
    youtubeTranscriptRequestKey = "";
  }

  function resetGroupLiveReader() {
    groupLiveItemsBySource = {};
    groupLiveCursorsBySource = {};
    groupLiveHasMoreBySource = {};
    groupLiveLoadingBySource = {};
    selectedGroupSourceId = null;
  }

  function applySnapshotPage(run: AnalysisRunDetail, page: AnalysisRunMessagesPage, append: boolean) {
    runSnapshotPage = page;
    runSnapshotMessages = append ? [...runSnapshotMessages, ...page.messages] : page.messages;
    runSnapshotCursor = page.next_cursor;
    runSnapshotHasMore = page.has_more;
    runSnapshotAvailability = runSnapshotAvailabilityFromPage({
      currentRun: run,
      page,
      loading: false,
      errorMessage: "",
    });
  }

  function clearCurrentRunForWorkspaceSwitch() {
    if (activeRunId !== null || currentRun !== null) {
      runWorkflow.invalidateOpenRunRequests();
    }

    activeRunId = null;
    currentRun = null;
    traceData = { refs: [] };
    savedTraceRefs = [];
    resolvedTraceRefs = [];
    selectedTraceRef = null;
    chatMessages = [];
    chatQuestion = "";
    chatting = false;
    activeChatRequestId = null;
    activeChatRunId = null;
  }

  function liveScopeExistsForRun(run: AnalysisRunDetail) {
    if (run.source_id !== null) {
      return sourceCatalog.some((source) => source.id === run.source_id);
    }

    if (run.source_group_id !== null) {
      return groups.some((group) => group.id === run.source_group_id);
    }

    return false;
  }

  function alignWorkspaceToOpenedRun(run: AnalysisRunDetail) {
    const next = openRunWorkspaceState(workspaceUiState, {
      runId: run.id,
      status: run.status,
      sourceId: run.source_id,
      sourceGroupId: run.source_group_id,
      liveScopeExists: liveScopeExistsForRun(run),
    });

    applyWorkspaceUiState(next);

    const legacy = legacyScopeFromWorkspaceSelection(next.workspaceSelection);
    analysisScope = legacy.analysisScope;
    selectedSourceId = legacy.selectedSourceId;
    selectedGroupId = legacy.selectedGroupId;
    resetGroupLiveReader();
    resetYoutubeTranscriptReader();
    selectedSnapshotSourceId = null;
  }

  async function applyRestoredWorkspaceSelection() {
    if (!restoredWorkspaceSelection) {
      return false;
    }

    const selection = fallbackWorkspaceSelection(
      restoredWorkspaceSelection,
      sourceCatalog,
      groups,
    );
    restoredWorkspaceSelection = null;

    if (selection.kind === "source") {
      await selectSource(selection.sourceId, { preserveRestoredCanvasState: true });
      return true;
    }

    if (selection.kind === "source_group") {
      selectGroup(selection.sourceGroupId, { preserveRestoredCanvasState: true });
      return true;
    }

    applyWorkspaceUiState({
      ...workspaceUiState,
      workspaceSelection: { kind: "none" },
    });
    return false;
  }

  function applyRunWorkflowPatch(patch: AnalysisRunWorkflowPatch) {
    if ("runs" in patch) runs = patch.runs ?? [];
    if ("activeRuns" in patch) activeRuns = patch.activeRuns ?? [];
    if ("activeRunId" in patch) activeRunId = patch.activeRunId ?? null;
    if ("currentRun" in patch) currentRun = patch.currentRun ?? null;
    if ("inspectorMode" in patch && patch.inspectorMode) inspectorMode = patch.inspectorMode;
    if ("loadingRuns" in patch) loadingRuns = patch.loadingRuns ?? false;
    if ("loadingActiveRuns" in patch) loadingActiveRuns = patch.loadingActiveRuns ?? false;
    if ("loadingRunDetail" in patch) loadingRunDetail = patch.loadingRunDetail ?? false;
    if ("startingReport" in patch) startingReport = patch.startingReport ?? false;
    if ("deletingRunIds" in patch) deletingRunIds = patch.deletingRunIds ?? {};
    if ("status" in patch && patch.status !== undefined) status = patch.status;
  }

  function applyWorkspaceWorkflowPatch(patch: AnalysisWorkspaceWorkflowPatch) {
    if ("accounts" in patch) accounts = patch.accounts ?? [];
    if ("accountStatuses" in patch) accountStatuses = patch.accountStatuses ?? {};
    if ("sourceCatalog" in patch) sourceCatalog = patch.sourceCatalog ?? [];
    if ("sourceMetrics" in patch) sourceMetrics = patch.sourceMetrics ?? {};
    if ("selectedSourceId" in patch) selectedSourceId = patch.selectedSourceId ?? "";
    if ("loadingSourceCatalog" in patch) {
      loadingSourceCatalog = patch.loadingSourceCatalog ?? false;
    }
    if ("status" in patch && patch.status !== undefined) status = patch.status;
  }

  function applySourceGroupsWorkflowPatch(patch: AnalysisSourceGroupsWorkflowPatch) {
    if ("templates" in patch) templates = patch.templates ?? [];
    if ("groups" in patch) groups = patch.groups ?? [];
    if ("selectedTemplateId" in patch) selectedTemplateId = patch.selectedTemplateId ?? "";
    if ("selectedGroupId" in patch) selectedGroupId = patch.selectedGroupId ?? "";
    if ("loadingTemplates" in patch) loadingTemplates = patch.loadingTemplates ?? false;
    if ("loadingGroups" in patch) loadingGroups = patch.loadingGroups ?? false;
    if ("savingTemplate" in patch) savingTemplate = patch.savingTemplate ?? false;
    if ("savingGroup" in patch) savingGroup = patch.savingGroup ?? false;
    if ("deletingTemplate" in patch) deletingTemplate = patch.deletingTemplate ?? false;
    if ("deletingGroup" in patch) deletingGroup = patch.deletingGroup ?? false;
    if ("status" in patch && patch.status !== undefined) status = patch.status;
  }

  const chatWorkflow = createAnalysisChatWorkflow({
    getState: () => ({
      currentRun,
      chatQuestion,
      chatMessages,
      chatting,
      activeChatRequestId,
      activeChatRunId,
      profileId: runProfileId(),
      modelOverride: runModelOverride(),
    }),
    patch: applyChatWorkflowPatch,
    listMessages: listAnalysisChatMessages,
    askQuestion: askAnalysisRunQuestion,
    clearMessages: clearAnalysisChatMessages,
    cancelRequest: cancelLlmRequest,
    confirmClearChat: () => openConfirmModal({
      title: "Clear chat history?",
      message: "This will remove all saved follow-up messages for the currently opened run.",
      confirmLabel: "Clear history",
      cancelLabel: "Cancel",
      tone: "danger",
    }),
    formatError: formatAppError,
  });

  const traceWorkflow = createAnalysisTraceWorkflow({
    getState: () => ({
      currentRun,
      traceData,
      savedTraceRefs,
      resolvedTraceRefs,
      selectedTraceRef,
    }),
    patch: applyTraceWorkflowPatch,
    getTrace: getAnalysisRunTrace,
    resolveRefs: resolveAnalysisTraceRefs,
    formatError: formatAppError,
  });

  const runWorkflow = createAnalysisRunWorkflow({
    getState: () => ({
      historyScopeParams,
      activeRunId,
      currentRun,
      activeChatRequestId,
      activeChatRunId,
      runs,
      activeRuns,
      deletingRunIds,
    }),
    patch: applyRunWorkflowPatch,
    listRuns: listAnalysisRuns,
    listActiveRuns: listActiveAnalysisRuns,
    getRun: getAnalysisRun,
    syncRunSnapshot,
    pruneLiveRuns,
    applyRunEvent,
    startReport: startAnalysisReport,
    cancelRun: cancelAnalysisRun,
    deleteRun: deleteAnalysisRun,
    confirm: openConfirmModal,
    cancelChatSilently: () => cancelChat({ silent: true }),
    clearChatState,
    clearOpenedRunState,
    setInitialLiveRun: (runId) => {
      liveRuns = {
        ...liveRuns,
        [runId]: {
          phase: "queued",
          progress: "",
          queuePosition: null,
          chunkSummaries: [],
          streamedOutput: "",
        },
      };
    },
    loadChatMessages,
    loadTrace,
    clearTraceState,
    onRunOpened: alignWorkspaceToOpenedRun,
    formatError: formatAppError,
  });

  const workspaceWorkflow = createAnalysisWorkspaceWorkflow({
    getState: () => ({ selectedSourceId }),
    patch: applyWorkspaceWorkflowPatch,
    listAccounts: listWorkspaceAccounts,
    getAccountStatuses: getWorkspaceAccountStatuses,
    listSources,
    listAnalysisSources,
    formatError: formatAppError,
  });

  function bindEditorToTemplate(template: AnalysisPromptTemplate | null) {
    const next = templateEditorStateFromTemplate(template);
    editorBoundTemplateId = next.editorBoundTemplateId;
    templateName = next.templateName;
    templateBody = next.templateBody;
  }

  function bindEditorToGroup(group: AnalysisSourceGroup | null) {
    const next = groupEditorStateFromGroup(group);
    editorBoundGroupId = next.editorBoundGroupId;
    groupName = next.groupName;
    groupSourceType = next.groupSourceType;
    groupMemberSourceIds = next.groupMemberSourceIds;
  }

  function isGroupSourceSelected(sourceId: number) {
    return groupSourceIsSelected(groupMemberSourceIds, sourceId);
  }

  function toggleGroupSource(sourceId: number) {
    groupMemberSourceIds = toggleGroupSourceSelection(groupMemberSourceIds, sourceId);
  }

  function changeGroupSourceType(value: AnalysisGroupSourceType) {
    groupSourceType = value;
    groupMemberSourceIds = groupMemberSourceIds.filter(
      (sourceId) => sourceMetrics[sourceId]?.source_type === value,
    );
  }

  function traceRefOrigin(ref: string) {
    return traceRefOriginFromState(ref, savedTraceRefs, resolvedTraceRefs);
  }

  async function focusTraceRef(ref: string) {
    await traceWorkflow.focusTraceRef(ref);
  }

  async function loadTrace(runId: number, guard?: AnalysisRunRequestGuard) {
    await traceWorkflow.loadTrace(runId, guard);
  }

  async function loadAccounts() {
    await workspaceWorkflow.loadAccounts();
  }

  async function loadSourceCatalog() {
    await workspaceWorkflow.loadSourceCatalog();
    await loadYoutubeSummaries();
  }

  async function loadYoutubeRuntimeStatus() {
    try {
      youtubeRuntimeStatus = await getYoutubeRuntimeStatus();
    } catch (error) {
      status = formatAppError("checking YouTube runtime", error);
    }
  }

  async function loadYoutubeSummaries() {
    const sourceIds = sourceCatalog
      .filter((source) => source.sourceType === "youtube")
      .map((source) => source.id);
    if (sourceIds.length === 0) {
      youtubeSummaries = {};
      return;
    }

    try {
      const summaries = await listYoutubeSourceSummaries(sourceIds);
      youtubeSummaries = Object.fromEntries(
        summaries.map((summary) => [summary.sourceId, summary]),
      );
    } catch (error) {
      youtubeSummaries = {};
      status = formatAppError("loading YouTube summaries", error);
    }
  }

  async function loadTakeoutImportJobs() {
    try {
      const jobs = await listTakeoutSourceImportJobs();
      applyTakeoutJobs(jobs);
    } catch (error) {
      status = formatAppError("loading Takeout import jobs", error);
    }
  }

  async function loadSourceTopics(
    sourceId: number,
    { preserveSelection = false }: { preserveSelection?: boolean } = {},
  ) {
    const source = sourceCatalog.find((candidate) => candidate.id === sourceId);
    if (source && !sourceCapabilities(source).hasTopics) {
      sourceTopics = [];
      selectedTopicKey = "__all_topics__";
      loadingSourceTopics = false;
      return;
    }

    const preferredKey = preserveSelection ? selectedTopicKey : "__all_topics__";
    loadingSourceTopics = true;
    try {
      const topics = await listSourceForumTopics(sourceId);
      sourceTopics = topics;
      selectedTopicKey = normalizeSelectedTopicKey(topics, preferredKey);
    } catch (error) {
      sourceTopics = [];
      selectedTopicKey = "__all_topics__";
      status = formatAppError("loading source topics", error);
    } finally {
      loadingSourceTopics = false;
    }
  }

  async function loadItems(sourceId: number) {
    loadingItems = true;
    const source = sourceCatalog.find((candidate) => candidate.id === sourceId);
    try {
      sourceItems = await listSourceItems({
        sourceId,
        limit: 120,
        beforePublishedAt: null,
        topicFilter: source && sourceCapabilities(source).hasTopics ? currentTopicFilter() : null,
      });
    } catch (error) {
      sourceItems = [];
      status = formatAppError("loading source messages", error);
    } finally {
      loadingItems = false;
    }
  }

  async function loadYoutubeTranscriptFirstPage(sourceId: number) {
    const requestKey = `${sourceId}:${youtubeTranscriptSearch.trim()}`;
    if (
      youtubeTranscriptRequestKey === requestKey &&
      (loadingYoutubeTranscriptSegments || youtubeTranscriptSegments.length > 0 || youtubeTranscriptCursor !== null)
    ) {
      return;
    }

    youtubeTranscriptRequestKey = requestKey;
    loadingYoutubeTranscriptSegments = true;
    try {
      const page = await listYoutubeTranscriptSegments({
        sourceId,
        after: null,
        limit: 80,
        searchQuery: youtubeTranscriptSearch.trim() || null,
      });
      if (youtubeTranscriptRequestKey !== requestKey) {
        return;
      }
      youtubeTranscriptSegments = page.segments;
      youtubeTranscriptCursor = page.nextCursor;
      youtubeTranscriptHasMore = page.hasMore;
    } catch (error) {
      if (youtubeTranscriptRequestKey === requestKey) {
        youtubeTranscriptSegments = [];
        youtubeTranscriptCursor = null;
        youtubeTranscriptHasMore = false;
        status = formatAppError("loading YouTube transcript", error);
      }
    } finally {
      if (youtubeTranscriptRequestKey === requestKey) {
        loadingYoutubeTranscriptSegments = false;
      }
    }
  }

  async function loadMoreYoutubeTranscriptSegments() {
    const source = currentSource();
    if (!source || source.sourceType !== "youtube" || source.sourceSubtype !== "video") return;
    if (!youtubeTranscriptCursor || loadingYoutubeTranscriptSegments) return;

    const requestKey = `${source.id}:${youtubeTranscriptSearch.trim()}`;
    loadingYoutubeTranscriptSegments = true;
    try {
      const page = await listYoutubeTranscriptSegments({
        sourceId: source.id,
        after: youtubeTranscriptCursor,
        limit: 80,
        searchQuery: youtubeTranscriptSearch.trim() || null,
      });
      if (youtubeTranscriptRequestKey !== requestKey) {
        return;
      }
      youtubeTranscriptSegments = [...youtubeTranscriptSegments, ...page.segments];
      youtubeTranscriptCursor = page.nextCursor;
      youtubeTranscriptHasMore = page.hasMore;
    } catch (error) {
      status = formatAppError("loading more YouTube transcript", error);
    } finally {
      if (youtubeTranscriptRequestKey === requestKey) {
        loadingYoutubeTranscriptSegments = false;
      }
    }
  }

  function changeYoutubeTranscriptSearch(value: string) {
    youtubeTranscriptSearch = value;
    resetYoutubeTranscriptReader();
    youtubeTranscriptSearch = value;
    const source = currentSource();
    if (source?.sourceType === "youtube" && source.sourceSubtype === "video") {
      void loadYoutubeTranscriptFirstPage(source.id);
    }
  }

  async function loadLiveGroupSourcePage(sourceId: number) {
    if (groupLiveLoadingBySource[sourceId]) return;
    groupLiveLoadingBySource = { ...groupLiveLoadingBySource, [sourceId]: true };
    try {
      const beforePublishedAt = groupLiveCursorsBySource[sourceId] ?? null;
      const items = await listSourceItems({
        sourceId,
        limit: 40,
        beforePublishedAt,
        topicFilter: null,
      });
      groupLiveItemsBySource = {
        ...groupLiveItemsBySource,
        [sourceId]: [...(groupLiveItemsBySource[sourceId] ?? []), ...items],
      };
      groupLiveCursorsBySource = {
        ...groupLiveCursorsBySource,
        [sourceId]: items.at(-1)?.publishedAt ?? beforePublishedAt,
      };
      groupLiveHasMoreBySource = {
        ...groupLiveHasMoreBySource,
        [sourceId]: items.length === 40,
      };
    } catch (error) {
      status = formatAppError("loading group source material", error);
    } finally {
      const next = { ...groupLiveLoadingBySource };
      delete next[sourceId];
      groupLiveLoadingBySource = next;
    }
  }

  async function selectSource(
    sourceId: number,
    { preserveRestoredCanvasState = false }: { preserveRestoredCanvasState?: boolean } = {},
  ) {
    const previousWorkspaceState = workspaceUiState;
    applyWorkspaceUiState(selectSourceWorkspace(workspaceUiState, sourceId));
    historyScope = "current";
    if (activeChatRequestId !== null) {
      void cancelChat({ silent: true });
    }
    clearCurrentRunForWorkspaceSwitch();

    const next = analysisSourceSelectionState(sourceId);
    const source = sourceCatalog.find((candidate) => candidate.id === sourceId) ?? null;
    analysisScope = next.analysisScope;
    selectedSourceId = next.selectedSourceId;
    selectedTopicKey = next.selectedTopicKey;
    inspectorMode = next.inspectorMode;
    youtubeVideoDetail = null;
    youtubePlaylistDetail = null;
    resetGroupLiveReader();
    selectedSnapshotSourceId = null;
    resetYoutubeTranscriptReader();
    if (!source || !sourceCapabilities(source).hasTopics) {
      sourceTopics = [];
      selectedTopicKey = "__all_topics__";
    }
    await Promise.all([
      source && sourceCapabilities(source).hasTopics ? loadSourceTopics(sourceId) : Promise.resolve(),
      loadItems(sourceId),
      source?.sourceType === "youtube" ? loadYoutubeDetail(source) : Promise.resolve(),
    ]);

    if (preserveRestoredCanvasState) {
      applyWorkspaceUiState({
        ...workspaceUiState,
        canvasMode: previousWorkspaceState.canvasMode,
        sourceViewBasis: previousWorkspaceState.sourceViewBasis,
        companionTab: previousWorkspaceState.companionTab,
      });
    }
  }

  function selectGroup(
    groupId: number,
    { preserveRestoredCanvasState = false }: { preserveRestoredCanvasState?: boolean } = {},
  ) {
    const previousWorkspaceState = workspaceUiState;
    applyWorkspaceUiState(selectSourceGroupWorkspace(workspaceUiState, groupId));
    historyScope = "current";
    if (activeChatRequestId !== null) {
      void cancelChat({ silent: true });
    }
    clearCurrentRunForWorkspaceSwitch();

    const next = analysisGroupSelectionState(groupId);
    analysisScope = next.analysisScope;
    selectedGroupId = next.selectedGroupId;
    sourceTopics = next.sourceTopics;
    selectedTopicKey = next.selectedTopicKey;
    youtubeVideoDetail = null;
    youtubePlaylistDetail = null;
    resetGroupLiveReader();
    selectedSnapshotSourceId = null;
    resetYoutubeTranscriptReader();
    inspectorMode = next.inspectorMode;

    if (preserveRestoredCanvasState) {
      applyWorkspaceUiState({
        ...workspaceUiState,
        canvasMode: previousWorkspaceState.canvasMode,
        sourceViewBasis: previousWorkspaceState.sourceViewBasis,
        companionTab: previousWorkspaceState.companionTab,
      });
    }
  }

  async function changeSelectedTopicKey(nextKey: string) {
    if (selectedTopicKey === nextKey) {
      return;
    }

    selectedTopicKey = nextKey;
    if (selectedSourceId) {
      await loadItems(Number(selectedSourceId));
    }
  }

  async function loadYoutubeDetail(source: Source) {
    loadingYoutubeDetail = true;
    try {
      if (source.sourceSubtype === "playlist") {
        youtubePlaylistDetail = await getYoutubePlaylistDetail(source.id);
        youtubeVideoDetail = null;
      } else {
        youtubeVideoDetail = await getYoutubeVideoDetail(source.id);
        youtubePlaylistDetail = null;
      }
    } catch (error) {
      youtubeVideoDetail = null;
      youtubePlaylistDetail = null;
      status = formatAppError("loading YouTube detail", error);
    } finally {
      loadingYoutubeDetail = false;
    }
  }

  async function loadTemplates() {
    await sourceGroupsWorkflow.loadTemplates();
  }

  async function loadGroups() {
    await sourceGroupsWorkflow.loadGroups();
  }

  const sourceGroupsWorkflow = createAnalysisSourceGroupsWorkflow({
    getState: () => ({
      groups,
      templates,
      selectedTemplate,
      selectedGroup,
      selectedTemplateId,
      selectedGroupId,
      editorBoundTemplateId,
      editorBoundGroupId,
    }),
    patch: applySourceGroupsWorkflowPatch,
    listTemplates: listAnalysisPromptTemplates,
    listGroups: listAnalysisSourceGroups,
    createTemplate: createAnalysisPromptTemplate,
    updateTemplate: updateAnalysisPromptTemplate,
    createGroup: createAnalysisSourceGroup,
    updateGroup: updateAnalysisSourceGroup,
    deleteTemplate: deleteAnalysisPromptTemplate,
    deleteGroup: deleteAnalysisSourceGroup,
    loadTemplates,
    confirm: openConfirmModal,
    bindTemplateEditor: bindEditorToTemplate,
    bindGroupEditor: bindEditorToGroup,
    formatError: formatAppError,
  });

  async function loadRuns() {
    await runWorkflow.loadRuns();
  }

  async function loadSourceJobs() {
    try {
      for (const job of await listSourceJobs({ limit: 100 })) {
        applySourceJob(job);
        if (isActiveSourceJob(job)) {
          syncingIds = sourceActionPending(syncingIds, job.source_id);
        }
      }
    } catch (error) {
      status = formatAppError("loading source jobs", error);
    }
  }

  async function loadActiveRuns() {
    await runWorkflow.loadActiveRuns();
  }

  async function cancelChat({ silent = false }: { silent?: boolean } = {}) {
    await chatWorkflow.cancelChat({ silent });
  }

  async function openRun(runId: number) {
    await runWorkflow.openRun(runId);
  }

  async function loadRunSnapshotFirstPage(runId: number) {
    const run = currentRun;
    if (!run || run.id !== runId) {
      return;
    }

    const loadKey = `${runId}:first:${selectedSnapshotSourceId ?? "all"}`;
    if (lastSnapshotLoadKey === loadKey && (loadingRunSnapshotMessages || runSnapshotPage !== null)) {
      return;
    }

    lastSnapshotLoadKey = loadKey;
    loadingRunSnapshotMessages = true;
    runSnapshotError = "";
    try {
      const page = await listAnalysisRunMessages({
        runId,
        after: null,
        limit: 50,
        sourceId: selectedSnapshotSourceId,
      });
      if (!currentRun || currentRun.id !== runId) {
        return;
      }
      applySnapshotPage(currentRun, page, false);
    } catch (error) {
      if (!currentRun || currentRun.id !== runId) {
        return;
      }
      runSnapshotMessages = [];
      runSnapshotCursor = null;
      runSnapshotHasMore = false;
      runSnapshotPage = null;
      runSnapshotError = formatAppError("loading run snapshot", error);
      runSnapshotAvailability = runSnapshotAvailabilityFromPage({
        currentRun,
        page: null,
        loading: false,
        errorMessage: runSnapshotError,
      });
    } finally {
      if (currentRun?.id === runId) {
        loadingRunSnapshotMessages = false;
      }
    }
  }

  async function loadMoreRunSnapshotMessages() {
    const run = currentRun;
    if (!run || !runSnapshotCursor || loadingRunSnapshotMessages) {
      return;
    }

    const runId = run.id;
    loadingRunSnapshotMessages = true;
    runSnapshotError = "";
    try {
      const page = await listAnalysisRunMessages({
        runId,
        after: runSnapshotCursor,
        limit: 50,
        sourceId: selectedSnapshotSourceId,
      });
      if (!currentRun || currentRun.id !== runId) {
        return;
      }
      applySnapshotPage(currentRun, page, true);
    } catch (error) {
      if (!currentRun || currentRun.id !== runId) {
        return;
      }
      runSnapshotError = formatAppError("loading more run snapshot messages", error);
    } finally {
      if (currentRun?.id === runId) {
        loadingRunSnapshotMessages = false;
      }
    }
  }

  function changeSelectedSnapshotSourceId(sourceId: number | null) {
    selectedSnapshotSourceId = sourceId;
    resetRunSnapshotState();
    if (
      currentRun &&
      workspaceUiState.canvasMode === "source" &&
      workspaceUiState.sourceViewBasis === "run_snapshot"
    ) {
      void loadRunSnapshotFirstPage(currentRun.id);
    }
  }

  async function loadChatMessages(runId: number, guard?: AnalysisRunRequestGuard) {
    await chatWorkflow.loadMessages(runId, guard);
  }

  async function runReport() {
    const isYoutubeAnalysisScope =
      (analysisScope === "single_source" && currentSource()?.sourceType === "youtube") ||
      (analysisScope === "source_group" && currentGroup()?.source_type === "youtube");
    await runWorkflow.startReport({
      analysisScope,
      selectedSourceId,
      selectedGroupId,
      selectedTemplateId,
      periodFrom,
      periodTo,
      outputLanguage,
      profileId: runProfileId(),
      modelOverride: runModelOverride(),
      youtubeCorpusMode: isYoutubeAnalysisScope ? youtubeCorpusMode : "transcript_description",
    });
  }

  async function cancelActiveRun(runId: number) {
    await runWorkflow.cancelRun(runId);
  }

  async function deleteSavedRun(run: AnalysisRunSummary) {
    await runWorkflow.deleteSavedRun(run);
  }

  async function askRunQuestion() {
    await chatWorkflow.askRunQuestion();
  }

  async function clearChatMessages() {
    await chatWorkflow.clearMessages();
  }

  async function syncSelectedSource(sourceId: number) {
    const source = sourceCatalog.find((candidate) => candidate.id === sourceId);
    syncingIds = sourceActionPending(syncingIds, sourceId);
    try {
      if (!source) {
        throw new Error("Source is not loaded.");
      }

      if (source.sourceType === "youtube") {
        await syncYoutubeSource(sourceId, {
          metadata: true,
          transcripts: source.sourceSubtype === "video",
          comments: source.sourceSubtype === "video",
        });
        status = "YouTube sync started.";
      } else {
        const result = await syncSource(sourceId);
        status = sourceSyncStatus(result);

        await Promise.all([loadSourceCatalog(), loadActiveRuns(), loadRuns()]);

        if (selectedSourceId === String(sourceId)) {
          await Promise.all([
            sourceCapabilities(source).hasTopics
              ? loadSourceTopics(sourceId, { preserveSelection: true })
              : Promise.resolve(),
            loadItems(sourceId),
          ]);
        }
      }
    } catch (error) {
      status = formatAppError("syncing the source", error);
    } finally {
      syncingIds = clearSourceActionPending(syncingIds, sourceId);
    }
  }

  async function startYoutubeJob(
    sourceId: number,
    action: () => Promise<SourceJobRecord>,
    successMessage: string,
  ) {
    syncingIds = sourceActionPending(syncingIds, sourceId);
    try {
      const job = await action();
      applySourceJob(job);
      status = successMessage;
    } catch (error) {
      status = formatAppError("starting YouTube source job", error);
      syncingIds = clearSourceActionPending(syncingIds, sourceId);
    }
  }

  async function syncYoutubeMetadata(sourceId: number) {
    await startYoutubeJob(
      sourceId,
      () => syncYoutubeSource(sourceId, { metadata: true, transcripts: false, comments: false }),
      "YouTube metadata sync started.",
    );
  }

  async function syncYoutubeTranscript(sourceId: number) {
    await startYoutubeJob(
      sourceId,
      () => syncYoutubeSource(sourceId, { metadata: false, transcripts: true, comments: false }),
      "YouTube transcript sync started.",
    );
  }

  async function syncYoutubeComments(sourceId: number) {
    await startYoutubeJob(
      sourceId,
      () => syncYoutubeSource(sourceId, { metadata: false, transcripts: false, comments: true }),
      "YouTube comments sync started.",
    );
  }

  async function syncYoutubePlaylist(sourceId: number) {
    await startYoutubeJob(
      sourceId,
      () => syncYoutubeSource(sourceId, { metadata: true, transcripts: true, comments: false }),
      "YouTube playlist sync started.",
    );
  }

  async function retryYoutubePlaylist(sourceId: number) {
    await startYoutubeJob(
      sourceId,
      () => retryFailedYoutubePlaylistVideos(sourceId, {
        metadata: false,
        transcripts: true,
        comments: false,
      }),
      "YouTube playlist retry started.",
    );
  }

  async function syncYoutubePlaylistVideoRow(playlistSourceId: number, videoSourceId: number) {
    await startYoutubeJob(
      playlistSourceId,
      () => syncYoutubePlaylistVideo(playlistSourceId, videoSourceId, {
        metadata: true,
        transcripts: true,
        comments: false,
      }),
      "YouTube playlist video sync started.",
    );
  }

  async function retryYoutubePlaylistVideoRow(playlistSourceId: number, videoSourceId: number) {
    await startYoutubeJob(
      playlistSourceId,
      () => syncYoutubePlaylistVideo(playlistSourceId, videoSourceId, {
        metadata: false,
        transcripts: true,
        comments: false,
      }),
      "YouTube playlist video retry started.",
    );
  }

  async function cancelYoutubeSourceJob(jobId: string) {
    try {
      await cancelSourceJob(jobId);
      status = "YouTube source job cancel requested.";
    } catch (error) {
      status = formatAppError("cancelling YouTube source job", error);
    }
  }

  async function startTakeoutImport(sourceId: number) {
    startingTakeoutSourceIds = sourceActionPending(startingTakeoutSourceIds, sourceId);
    try {
      await startTakeoutSourceImport(sourceId);
      status = "Takeout import started.";
    } catch (error) {
      status = formatAppError("starting Takeout import", error);
    } finally {
      startingTakeoutSourceIds = clearSourceActionPending(startingTakeoutSourceIds, sourceId);
    }
  }

  async function cancelTakeoutImport(jobId: string) {
    try {
      const result = await cancelTakeoutSourceImport(jobId);
      status = result.cancelled ? "Takeout import cancel requested." : "No active Takeout import to cancel.";
    } catch (error) {
      status = formatAppError("cancelling Takeout import", error);
    }
  }

  async function refreshSourcesAfterManagement(sourceId?: number) {
    await Promise.all([loadSourceCatalog(), loadGroups(), loadActiveRuns()]);
    await loadRuns();

    if (sourceId !== undefined) {
      await selectSource(sourceId);
      return;
    }

    if (selectedSourceId) {
      const sourceId = Number(selectedSourceId);
      const source = sourceCatalog.find((candidate) => candidate.id === sourceId);
      await Promise.all([
        source && sourceCapabilities(source).hasTopics
          ? loadSourceTopics(sourceId, { preserveSelection: true })
          : Promise.resolve(),
        loadItems(sourceId),
        source?.sourceType === "youtube" ? loadYoutubeDetail(source) : Promise.resolve(),
      ]);
      return;
    }

    sourceItems = [];
  }

  async function deleteSource(source: Source) {
    const confirmed = await openConfirmModal(sourceDeletionDialog(source));
    if (!confirmed) {
      return;
    }

    deletingSourceIds = { ...deletingSourceIds, [source.id]: true };
    try {
      await deleteSourceCommand(source.id);
      status = sourceDeletedStatus(source);

      const reset = sourceDeletionResetState(source.id, selectedSourceId);
      if (reset !== null) {
        sourceItems = reset.sourceItems;
        currentRun = reset.currentRun;
        activeRunId = reset.activeRunId;
        traceData = reset.traceData;
        savedTraceRefs = reset.savedTraceRefs;
        resolvedTraceRefs = reset.resolvedTraceRefs;
        selectedTraceRef = reset.selectedTraceRef;
        chatMessages = reset.chatMessages;
        chatQuestion = reset.chatQuestion;
        chatting = reset.chatting;
        activeChatRequestId = reset.activeChatRequestId;
        activeChatRunId = reset.activeChatRunId;
      }

      await refreshSourcesAfterManagement();
    } catch (error) {
      status = formatAppError("deleting the source", error);
    } finally {
      const next = { ...deletingSourceIds };
      delete next[source.id];
      deletingSourceIds = next;
    }
  }

  function openNotebookLmExportDialog() {
    if (analysisScope !== "single_source" || !currentSource()) {
      status = "Select a single synced source before exporting.";
      return;
    }

    notebookLmExportResult = null;
    notebookLmExportForm = {
      ...notebookLmExportForm,
      fromDate: periodFrom,
      toDate: periodTo,
    };
    exportDialogOpen = true;
  }

  async function chooseNotebookLmOutputDir() {
    const selected = await openDialog({
      directory: true,
      multiple: false,
    });
    if (typeof selected !== "string") {
      return;
    }

    notebookLmExportForm = {
      ...notebookLmExportForm,
      outputDir: selected,
    };
  }

  async function exportNotebookLm() {
    const source = currentSource();
    if (!source) {
      status = "Select a source before exporting.";
      return;
    }
    if (!notebookLmExportForm.outputDir.trim()) {
      status = "Choose an output folder before exporting.";
      return;
    }

    exportingNotebookLm = true;
    notebookLmExportResult = null;
    const exportId = createNotebookLmExportId();
    activeNotebookLmExportId = exportId;
    notebookLmExportProgress = notebookLmExportInitialProgress();
    try {
      const request = notebookLmExportRequestFromForm(exportId, source.id, notebookLmExportForm);

      const result = await exportSourceToNotebookLm(request);
      notebookLmExportResult = result;
      status = notebookLmExportCompleteStatus(result);
    } catch (error) {
      status = formatAppError("exporting for NotebookLM", error);
    } finally {
      exportingNotebookLm = false;
      if (activeNotebookLmExportId === exportId) {
        activeNotebookLmExportId = null;
        notebookLmExportProgress = null;
      }
    }
  }

  async function saveTemplateChanges(nextName = templateName, nextBody = templateBody) {
    await sourceGroupsWorkflow.saveTemplateChanges(nextName, nextBody);
  }

  async function saveTemplateCopy(nextName = templateName, nextBody = templateBody) {
    await sourceGroupsWorkflow.saveTemplateCopy(nextName, nextBody);
  }

  async function deleteTemplate() {
    await sourceGroupsWorkflow.deleteTemplate();
  }

  async function saveGroupChanges() {
    await sourceGroupsWorkflow.saveGroupChanges(groupName, groupMemberSourceIds, groupSourceType);
  }

  async function saveGroupCopy() {
    await sourceGroupsWorkflow.saveGroupCopy(groupName, groupMemberSourceIds, groupSourceType);
  }

  async function deleteGroup() {
    await sourceGroupsWorkflow.deleteGroup();
  }

  function startNewGroup() {
    selectedGroupId = "";
    bindEditorToGroup(null);
  }

  $effect(() => {
    const runId = currentRun?.id ?? null;
    if (runId === null) {
      resetRunSnapshotState();
      return;
    }

    if (!lastSnapshotLoadKey.startsWith(`${runId}:`) && lastSnapshotLoadKey !== "") {
      resetRunSnapshotState();
    }
  });

  $effect(() => {
    const source = currentSource();
    if (
      workspaceUiState.canvasMode === "source" &&
      workspaceUiState.sourceViewBasis === "live_source" &&
      source?.sourceType === "youtube" &&
      source.sourceSubtype === "video"
    ) {
      void loadYoutubeTranscriptFirstPage(source.id);
    }
  });

  $effect(() => {
    const group = currentGroup();
    if (
      workspaceUiState.canvasMode === "source" &&
      workspaceUiState.sourceViewBasis === "live_source" &&
      analysisScope === "source_group" &&
      group
    ) {
      for (const member of group.members.slice(0, 6)) {
        if (!groupLiveItemsBySource[member.source_id]) {
          void loadLiveGroupSourcePage(member.source_id);
        }
      }
    }
  });

  $effect(() => {
    if (
      currentRun &&
      workspaceUiState.canvasMode === "source" &&
      workspaceUiState.sourceViewBasis === "run_snapshot"
    ) {
      void loadRunSnapshotFirstPage(currentRun.id);
    }
  });

  $effect(() => {
    workspaceUiState;
    historyScope;
    runFilter;

    if (!workspacePersistenceReady) {
      return;
    }

    persistWorkspaceState();
  });

  $effect(() => {
    const params = historyScopeParams;
    if (params === null) {
      runs = [];
      return;
    }

    void runWorkflow.loadRunsForScope(params);
  });

  $effect(() => {
    const current = selectedTemplate;
    if (current && editorBoundTemplateId !== current.id) {
      bindEditorToTemplate(current);
    }
  });

  $effect(() => {
    const current = selectedGroup;
    if (current && editorBoundGroupId !== current.id) {
      bindEditorToGroup(current);
    }
  });

  $effect(() => {
    if (typeof window === "undefined") return;
    if (statusTimer) {
      clearTimeout(statusTimer);
      statusTimer = null;
    }
    if (!status || isErrorStatus(status)) {
      return;
    }
    statusTimer = window.setTimeout(() => {
      status = "";
      statusTimer = null;
    }, 5000);
  });

  onMount(() => {
    let disposed = false;
    let detachAnalysisListener: (() => void) | null = null;
    let detachChatListener: (() => void) | null = null;
    let detachNotebookLmExportListener: (() => void) | null = null;
    let detachTakeoutImportListener: (() => void) | null = null;
    let detachSourceJobListener: (() => void) | null = null;

    restorePersistedWorkspaceState();
    void loadAccounts();
    void (async () => {
      await Promise.all([loadSourceCatalog(), loadGroups()]);
      const restoredSelectionApplied = await applyRestoredWorkspaceSelection();
      if (!restoredSelectionApplied && selectedSourceId) {
        const sourceId = Number(selectedSourceId);
        const selected = sourceCatalog.find((source) => source.id === sourceId);
        void Promise.all([
          selected && sourceCapabilities(selected).hasTopics
            ? loadSourceTopics(sourceId)
            : Promise.resolve(),
          loadItems(sourceId),
          selected?.sourceType === "youtube" ? loadYoutubeDetail(selected) : Promise.resolve(),
        ]);
      }
      void loadActiveRuns();
    })();
    void loadTemplates();
    void loadLlmProfiles();
    void loadYoutubeRuntimeStatus();
    void loadTakeoutImportJobs();
    void loadSourceJobs();

    void listenToAnalysisRunEvents(({ payload }) => {
      if (disposed) {
        return;
      }

      runWorkflow.handleRunEvent(payload);
    }).then((unlisten) => {
      if (disposed) {
        unlisten();
        return;
      }
      detachAnalysisListener = unlisten;
    });

    void listenToAnalysisChatEvents(({ payload }) => {
      if (disposed) {
        return;
      }

      chatWorkflow.handleEvent(payload);
    }).then((unlisten) => {
      if (disposed) {
        unlisten();
        return;
      }
      detachChatListener = unlisten;
    });

    void listenToNotebookLmExportEvents(({ payload }) => {
      if (disposed) {
        return;
      }

      applyNotebookLmExportEvent(payload);
    }).then((unlisten) => {
      if (disposed) {
        unlisten();
        return;
      }
      detachNotebookLmExportListener = unlisten;
    });

    void listenToTakeoutImportEvents(({ payload }) => {
      if (disposed) {
        return;
      }

      applyTakeoutImportEvent(payload);
    }).then((unlisten) => {
      if (disposed) {
        unlisten();
        return;
      }
      detachTakeoutImportListener = unlisten;
    });

    void listenToSourceJobEvents((job) => {
      if (disposed) {
        return;
      }

      applySourceJob(job);
      syncingIds = isActiveSourceJob(job)
        ? sourceActionPending(syncingIds, job.source_id)
        : clearSourceActionPending(syncingIds, job.source_id);
      if (!isActiveSourceJob(job)) {
        void loadYoutubeSummaries();
        const selected = currentSource();
        if (selected?.sourceType === "youtube") {
          void loadYoutubeDetail(selected);
        }
      }
    }).then((unlisten) => {
      if (disposed) {
        unlisten();
        return;
      }
      detachSourceJobListener = unlisten;
    });

    return () => {
      disposed = true;
      if (statusTimer) {
        clearTimeout(statusTimer);
        statusTimer = null;
      }
      if (detachAnalysisListener !== null) {
        detachAnalysisListener();
      }
      if (detachChatListener !== null) {
        detachChatListener();
      }
      if (detachNotebookLmExportListener !== null) {
        detachNotebookLmExportListener();
      }
      if (detachTakeoutImportListener !== null) {
        detachTakeoutImportListener();
      }
      if (detachSourceJobListener !== null) {
        detachSourceJobListener();
      }
    };
  });
</script>

{#if status}
  <StatusMessage tone={isErrorStatus(status) ? "error" : "default"} className="workspace-status">
    {status}
  </StatusMessage>
{/if}

<section class="analysis-workspace">
  <CompactSourceRail
    {sourceCatalog}
    {groups}
    {sourceMetrics}
    {loadingSourceCatalog}
    {loadingGroups}
    {railQuery}
    {filteredSourceCatalog}
    {filteredGroups}
    workspaceSelection={workspaceUiState.workspaceSelection}
    {syncingIds}
    {deletingSourceIds}
    {startingTakeoutSourceIds}
    {takeoutJobsBySource}
    {sourceJobsBySource}
    {youtubeSummaries}
    {youtubeRuntimeStatus}
    {formatTimestamp}
    {accountLabel}
    {sourceInitial}
    {runtimeStatus}
    {runtimeBadge}
    {sourceSyncDisabledReason}
    onChangeRailQuery={(value) => (railQuery = value)}
    onSelectSource={(sourceId) => void selectSource(sourceId)}
    onSelectGroup={selectGroup}
    onSyncSource={(sourceId) => void syncSelectedSource(sourceId)}
    onStartTakeoutImport={(sourceId) => void startTakeoutImport(sourceId)}
    onCancelTakeoutImport={(jobId) => void cancelTakeoutImport(jobId)}
    onCancelSourceJob={(jobId) => void cancelYoutubeSourceJob(jobId)}
    onOpenSourceManager={() => (sourceManagerOpen = true)}
    onDeleteSource={(source) => void deleteSource(source)}
  />

  <ReportCanvas
    {analysisScope}
    currentSource={currentSource()}
    currentGroup={currentGroup()}
    currentSourceMetric={currentSourceMetric()}
    currentScopeTitle={currentScopeTitle()}
    currentScopeSummary={currentScopeSummary()}
    canvasMode={workspaceUiState.canvasMode}
    sourceViewBasis={workspaceUiState.sourceViewBasis}
    {runSnapshotAvailability}
    {runSnapshotMessages}
    {loadingRunSnapshotMessages}
    {runSnapshotError}
    hasMoreRunSnapshotMessages={runSnapshotHasMore}
    {youtubeTranscriptSegments}
    {loadingYoutubeTranscriptSegments}
    {youtubeTranscriptHasMore}
    {youtubeTranscriptSearch}
    {groupLiveItemsBySource}
    {groupLiveHasMoreBySource}
    {selectedGroupSourceId}
    {selectedSnapshotSourceId}
    {periodFrom}
    {periodTo}
    {selectedTemplateId}
    {loadingTemplates}
    {templates}
    {outputLanguage}
    {youtubeCorpusMode}
    {llmProfiles}
    {activeLlmProfile}
    {selectedLlmProfileId}
    {selectedLlmModel}
    {customModelOverride}
    {llmProviderModels}
    {loadingLlmProviderModels}
    {llmModelStatus}
    {startingReport}
    {selectedSourceId}
    {selectedGroupId}
    {currentRun}
    {loadingRunDetail}
    {selectedRunIsActive}
    {activeProgress}
    {activePhase}
    {focusedStreamedOutput}
    {canCancelCurrentRun}
    {sourceItems}
    {loadingItems}
    {sourceTopics}
    {loadingSourceTopics}
    {selectedTopicKey}
    showTopicSelector={shouldShowTopicSelector()}
    {selectedTraceRef}
    traceRefCount={traceData.refs.length}
    {chatMessages}
    {chatQuestion}
    {chatting}
    canCancelChat={chatting && activeChatRequestId !== null}
    {clearingChat}
    {loadingChat}
    {selectedTemplate}
    {templateName}
    {templateBody}
    {savingTemplate}
    {deletingTemplate}
    {groups}
    {groupName}
    {groupSourceType}
    {groupMemberSourceIds}
    {selectedGroup}
    {savingGroup}
    {deletingGroup}
    sourceMetricsList={Object.values(sourceMetrics)}
    {syncingIds}
    sourceJobs={currentSourceJobs()}
    {youtubeVideoDetail}
    {youtubePlaylistDetail}
    {loadingYoutubeDetail}
    {formatTimestamp}
    {formatPeriod}
    {runTargetLabel}
    {statusTone}
    {reportLines}
    {phaseLabel}
    {accountLabel}
    {sourceSyncDisabledReason}
    {startOfDayUnix}
    {endOfDayUnix}
    {isGroupSourceSelected}
    onChangeCanvasMode={(mode) => changeCanvasMode(mode)}
    onViewLiveSource={() => viewLiveSourceForOpenedRun()}
    onBackToRunSnapshot={() => backToRunSnapshot()}
    onLoadMoreRunSnapshotMessages={() => void loadMoreRunSnapshotMessages()}
    onChangeTranscriptSearch={changeYoutubeTranscriptSearch}
    onLoadMoreYoutubeTranscriptSegments={() => void loadMoreYoutubeTranscriptSegments()}
    onLoadLiveGroupSourcePage={(sourceId) => void loadLiveGroupSourcePage(sourceId)}
    onChangeSelectedGroupSourceId={(sourceId) => (selectedGroupSourceId = sourceId)}
    onChangeSelectedSnapshotSourceId={changeSelectedSnapshotSourceId}
    onChangeSelectedTopicKey={(value) => void changeSelectedTopicKey(value)}
    onChangePeriodFrom={(value) => (periodFrom = value)}
    onChangePeriodTo={(value) => (periodTo = value)}
    onChangeSelectedTemplateId={(value) => (selectedTemplateId = value)}
    onChangeOutputLanguage={(value) => (outputLanguage = value)}
    onChangeYoutubeCorpusMode={(value) => (youtubeCorpusMode = value)}
    onChangeLlmProfile={changeLlmProfile}
    onChangeLlmModel={changeLlmModel}
    onChangeCustomModelOverride={(value) => (customModelOverride = value)}
    onRunReport={() => void runReport()}
    onSyncCurrentSource={(sourceId) => void syncSelectedSource(sourceId)}
    onSyncYoutubeMetadata={(sourceId) => void syncYoutubeMetadata(sourceId)}
    onSyncYoutubeTranscript={(sourceId) => void syncYoutubeTranscript(sourceId)}
    onSyncYoutubeComments={(sourceId) => void syncYoutubeComments(sourceId)}
    onSyncYoutubePlaylist={(sourceId) => void syncYoutubePlaylist(sourceId)}
    onRetryFailedYoutubePlaylistVideos={(sourceId) => void retryYoutubePlaylist(sourceId)}
    onSyncYoutubePlaylistVideo={(playlistSourceId, videoSourceId) => void syncYoutubePlaylistVideoRow(playlistSourceId, videoSourceId)}
    onRetryYoutubePlaylistVideo={(playlistSourceId, videoSourceId) => void retryYoutubePlaylistVideoRow(playlistSourceId, videoSourceId)}
    onCancelSourceJob={(jobId) => void cancelYoutubeSourceJob(jobId)}
    onOpenSource={(sourceId) => void selectSource(sourceId)}
    {exportDialogOpen}
    {notebookLmExportForm}
    notebookLmExportResult={notebookLmExportResult}
    notebookLmExportProgress={notebookLmExportProgress}
    {exportingNotebookLm}
    onOpenNotebookLmExport={openNotebookLmExportDialog}
    onCloseNotebookLmExport={() => (exportDialogOpen = false)}
    onChooseNotebookLmOutputDir={() => void chooseNotebookLmOutputDir()}
    onChangeNotebookLmExportForm={(form) => (notebookLmExportForm = form)}
    onExportNotebookLm={() => void exportNotebookLm()}
    onFocusTraceRef={focusTraceRef}
    onCancelCurrentRun={() => {
      if (activeRunId !== null) {
        void cancelActiveRun(activeRunId);
      }
    }}
    onAskRunQuestion={() => void askRunQuestion()}
    onCancelChat={() => void cancelChat()}
    onClearChat={() => void clearChatMessages()}
    onChangeChatQuestion={(value) => (chatQuestion = value)}
    onSaveTemplateCopy={() => void saveTemplateCopy()}
    onSaveTemplateChanges={() => void saveTemplateChanges()}
    onDeleteTemplate={() => void deleteTemplate()}
    onChangeSelectedGroupId={(value) => (selectedGroupId = value)}
    onChangeGroupName={(value) => (groupName = value)}
    onChangeGroupSourceType={changeGroupSourceType}
    onToggleGroupSource={toggleGroupSource}
    onStartNewGroup={startNewGroup}
    onSaveGroupCopy={() => void saveGroupCopy()}
    onSaveGroupChanges={() => void saveGroupChanges()}
    onDeleteGroup={() => void deleteGroup()}
  />

  <div class="inspector-slot">
    <WorkspaceInspector
      {inspectorMode}
      {activeRuns}
      {loadingActiveRuns}
      {activeRunId}
      {runs}
      {loadingRuns}
      {historyScope}
      historyTargetReady={historyScopeParams !== null}
      {runFilter}
      {deletingRunIds}
      {filteredRuns}
      {traceData}
      {selectedTraceRef}
      {selectedTrace}
      {focusedChunkSummaries}
      {selectedRunIsActive}
      {formatTimestamp}
      {formatPeriod}
      {phaseLabel}
      {livePhase}
      {liveProgress}
      {runTargetLabel}
      {statusTone}
      {traceRefOrigin}
      onChangeInspectorMode={(mode) => (inspectorMode = mode)}
      onRefreshActiveRuns={() => void loadActiveRuns()}
      onOpenRun={(runId) => void openRun(runId)}
      onCancelRun={(runId) => void cancelActiveRun(runId)}
      onRefreshRuns={() => void loadRuns()}
      onDeleteRun={(run) => void deleteSavedRun(run)}
      onChangeFilter={(next) => (runFilter = next)}
      onChangeHistoryScope={(next) => (historyScope = next)}
      onSelectTraceRef={(ref) => (selectedTraceRef = ref)}
    />
  </div>

  <SourceManagementDialog
    open={sourceManagerOpen}
    {accounts}
    {accountStatuses}
    existingSources={sourceCatalog}
    onClose={() => (sourceManagerOpen = false)}
    onSourcesChanged={(sourceId) => void refreshSourcesAfterManagement(sourceId)}
    onStatus={(message) => (status = message)}
  />
</section>

<style>
  .analysis-workspace {
    display: grid;
    grid-template-columns: minmax(4.25rem, 4.75rem) minmax(0, 1.6fr) minmax(320px, 430px);
    gap: 0.9rem;
    align-items: start;
    min-width: 0;
  }

  :global(.workspace-status) {
    margin-bottom: 0.85rem;
  }

  .inspector-slot {
    min-width: 0;
  }

  @media (max-width: 1500px) {
    .analysis-workspace {
      grid-template-columns: minmax(4.25rem, 4.75rem) minmax(0, 1fr);
    }

    .inspector-slot {
      grid-column: 2;
    }
  }

  @media (max-width: 1180px) {
    .analysis-workspace {
      grid-template-columns: 1fr;
    }

    .inspector-slot {
      grid-column: 1;
    }
  }
</style>
