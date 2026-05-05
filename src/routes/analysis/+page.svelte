<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { open as openDialog } from "@tauri-apps/plugin-dialog";
  import StatusMessage from "$lib/components/ui/StatusMessage.svelte";
  import WorkspaceInspector from "$lib/components/analysis/workspace-inspector.svelte";
  import WorkspaceMain from "$lib/components/analysis/workspace-main.svelte";
  import WorkspaceRail from "$lib/components/analysis/workspace-rail.svelte";
  import SourceManagementDialog from "$lib/components/analysis/source-management-dialog.svelte";
  import { formatAppError } from "$lib/app-error";
  import {
    getAnalysisRun,
    listActiveAnalysisRuns,
    listAnalysisRuns,
    listenToAnalysisRunEvents,
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
  import { cancelLlmRequest } from "$lib/api/llm";
  import {
    cancelTakeoutSourceImport,
    listTakeoutSourceImportJobs,
    listenToTakeoutImportEvents,
    startTakeoutSourceImport,
  } from "$lib/api/takeout-import";
  import {
    exportSourceToNotebookLm,
    listenToNotebookLmExportEvents,
  } from "$lib/api/notebooklm-export";
  import {
    deleteSource as deleteSourceCommand,
    listSourceForumTopics,
    listSourceItems,
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
    analysisReportStartCommand,
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
    runDeletedStatus,
    runDeletionDecision,
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
    groupCopyCommand,
    groupCreatedStatus,
    groupDeleteDecision,
    groupDeletedStatus,
    groupEditorStateFromGroup,
    groupFallbackSelection,
    groupUpdateCommand,
    groupUpdatedStatus,
    isGroupSourceSelected as groupSourceIsSelected,
    templateCopyCommand,
    templateCreatedStatus,
    templateDeleteDecision,
    templateDeletedStatus,
    templateEditorStateFromTemplate,
    templateFallbackSelection,
    templateUpdateCommand,
    templateUpdatedStatus,
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
    accountLabel as formatAccountLabel,
    membershipLabel,
    runtimeBadge,
    runtimeStatus as getRuntimeStatus,
    sourceInitial,
    sourceKindLabel,
    sourceSyncDisabledReason as getSourceSyncDisabledReason,
  } from "$lib/analysis-source-state";
  import type { AccountRecord, AccountRuntimeStatus } from "$lib/types/accounts";
  import type {
    AnalysisChatTurn,
    AnalysisPromptTemplate,
    AnalysisRunDetail,
    AnalysisRunEvent,
    AnalysisRunSummary,
    AnalysisSourceGroup,
    AnalysisSourceOption,
    AnalysisTraceData,
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
  } from "$lib/types/sources";
  import type { NotebookLmExportForm } from "$lib/components/analysis/notebooklm-export-dialog.svelte";

  type InspectorMode = "active" | "history" | "trace" | "chunks";

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
  let accounts = $state<AccountRecord[]>([]);
  let accountStatuses = $state<Record<number, AccountRuntimeStatus>>({});
  let templates = $state<AnalysisPromptTemplate[]>([]);
  let runs = $state<AnalysisRunSummary[]>([]);
  let activeRuns = $state<AnalysisRunSummary[]>([]);
  let groups = $state<AnalysisSourceGroup[]>([]);

  let loadingSourceCatalog = $state(false);
  let loadingItems = $state(false);
  let loadingSourceTopics = $state(false);
  let loadingTemplates = $state(false);
  let loadingRuns = $state(false);
  let loadingActiveRuns = $state(false);
  let loadingGroups = $state(false);
  let loadingRunDetail = $state(false);
  let loadingChat = $state(false);

  let railQuery = $state("");
  let inspectorMode = $state<InspectorMode>("history");

  let selectedSourceId = $state("");
  let selectedTopicKey = $state("__all_topics__");
  let selectedTemplateId = $state("");
  let selectedGroupId = $state("");
  let analysisScope = $state<"single_source" | "source_group">("single_source");
  let periodFrom = $state(defaultDateOffset(-30));
  let periodTo = $state(defaultDateOffset(0));
  let outputLanguage = $state("Russian");
  let modelOverride = $state("");
  let templateName = $state("");
  let templateBody = $state("");
  let editorBoundTemplateId = $state<number | null>(null);
  let savingTemplate = $state(false);
  let deletingTemplate = $state(false);
  let groupName = $state("");
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
      void loadSourceTopics(sourceId, { preserveSelection: true }).then(() => loadItems(sourceId));
    }
  }

  function currentSource() {
    return currentAnalysisSource(selectedSourceId, sourceCatalog);
  }

  function currentSourceMetric() {
    return currentAnalysisSourceMetric(currentSource(), sourceMetrics);
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
    return getSourceSyncDisabledReason(source, accountStatuses);
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

  function applyRunWorkflowPatch(patch: AnalysisRunWorkflowPatch) {
    if ("runs" in patch) runs = patch.runs ?? [];
    if ("activeRuns" in patch) activeRuns = patch.activeRuns ?? [];
    if ("activeRunId" in patch) activeRunId = patch.activeRunId ?? null;
    if ("currentRun" in patch) currentRun = patch.currentRun ?? null;
    if ("inspectorMode" in patch && patch.inspectorMode) inspectorMode = patch.inspectorMode;
    if ("loadingRuns" in patch) loadingRuns = patch.loadingRuns ?? false;
    if ("loadingActiveRuns" in patch) loadingActiveRuns = patch.loadingActiveRuns ?? false;
    if ("loadingRunDetail" in patch) loadingRunDetail = patch.loadingRunDetail ?? false;
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
      modelOverride,
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
    }),
    patch: applyRunWorkflowPatch,
    listRuns: listAnalysisRuns,
    listActiveRuns: listActiveAnalysisRuns,
    getRun: getAnalysisRun,
    syncRunSnapshot,
    pruneLiveRuns,
    applyRunEvent,
    cancelChatSilently: () => cancelChat({ silent: true }),
    clearChatState,
    loadChatMessages,
    loadTrace,
    clearTraceState,
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
    groupMemberSourceIds = next.groupMemberSourceIds;
  }

  function isGroupSourceSelected(sourceId: number) {
    return groupSourceIsSelected(groupMemberSourceIds, sourceId);
  }

  function toggleGroupSource(sourceId: number) {
    groupMemberSourceIds = toggleGroupSourceSelection(groupMemberSourceIds, sourceId);
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
    try {
      accounts = await invoke<AccountRecord[]>("list_accounts");
      if (accounts.length === 0) {
        accountStatuses = {};
        return;
      }
      const statuses = await invoke<AccountRuntimeStatus[]>("tg_get_account_statuses", {
        accountIds: accounts.map((account) => account.id),
      });
      accountStatuses = Object.fromEntries(
        statuses.map((runtimeStatus) => [runtimeStatus.account_id, runtimeStatus]),
      );
    } catch (error) {
      status = formatAppError("loading workspace accounts", error);
    }
  }

  async function loadSourceCatalog() {
    loadingSourceCatalog = true;
    try {
      const [allSources, analysisSources] = await Promise.all([
        listSources(null),
        invoke<AnalysisSourceOption[]>("list_analysis_sources"),
      ]);
      sourceCatalog = allSources;
      sourceMetrics = Object.fromEntries(
        analysisSources.map((source) => [source.id, source]),
      );

      if (!selectedSourceId && allSources.length > 0) {
        const firstSynced = analysisSources[0]?.id ?? allSources[0].id;
        selectedSourceId = String(firstSynced);
      } else if (
        selectedSourceId &&
        !allSources.some((source) => source.id === Number(selectedSourceId))
      ) {
        selectedSourceId = allSources[0] ? String(allSources[0].id) : "";
      }
    } catch (error) {
      status = formatAppError("loading workspace sources", error);
    } finally {
      loadingSourceCatalog = false;
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
    try {
      sourceItems = await listSourceItems({
        sourceId,
        limit: 120,
        beforePublishedAt: null,
        topicFilter: currentTopicFilter(),
      });
    } catch (error) {
      sourceItems = [];
      status = formatAppError("loading source messages", error);
    } finally {
      loadingItems = false;
    }
  }

  async function selectSource(sourceId: number) {
    const next = analysisSourceSelectionState(sourceId);
    analysisScope = next.analysisScope;
    selectedSourceId = next.selectedSourceId;
    selectedTopicKey = next.selectedTopicKey;
    inspectorMode = next.inspectorMode;
    await loadSourceTopics(sourceId);
    await loadItems(sourceId);
  }

  function selectGroup(groupId: number) {
    const next = analysisGroupSelectionState(groupId);
    analysisScope = next.analysisScope;
    selectedGroupId = next.selectedGroupId;
    sourceTopics = next.sourceTopics;
    selectedTopicKey = next.selectedTopicKey;
    inspectorMode = next.inspectorMode;
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

  async function loadTemplates() {
    loadingTemplates = true;
    try {
      templates = await invoke<AnalysisPromptTemplate[]>("list_analysis_prompt_templates", {
        templateKind: "report",
      });
      if (!selectedTemplateId && templates.length > 0) {
        selectedTemplateId = String(templates[0].id);
      }
      const current = selectedTemplate;
      if (current && editorBoundTemplateId !== current.id) {
        bindEditorToTemplate(current);
      }
    } catch (error) {
      status = formatAppError("loading report templates", error);
    } finally {
      loadingTemplates = false;
    }
  }

  async function loadGroups() {
    loadingGroups = true;
    try {
      groups = await invoke<AnalysisSourceGroup[]>("list_analysis_source_groups");
      if (!selectedGroupId && groups.length > 0) {
        selectedGroupId = String(groups[0].id);
      }
      const current = selectedGroup;
      if (current && editorBoundGroupId !== current.id) {
        bindEditorToGroup(current);
      }
    } catch (error) {
      status = formatAppError("loading source groups", error);
    } finally {
      loadingGroups = false;
    }
  }

  async function loadRuns() {
    await runWorkflow.loadRuns();
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

  async function loadChatMessages(runId: number, guard?: AnalysisRunRequestGuard) {
    await chatWorkflow.loadMessages(runId, guard);
  }

  async function runReport() {
    const command = analysisReportStartCommand({
      analysisScope,
      selectedSourceId,
      selectedGroupId,
      selectedTemplateId,
      periodFrom,
      periodTo,
      outputLanguage,
      modelOverride,
    });
    if (!command.ok) {
      status = command.status;
      return;
    }

    startingReport = true;
    inspectorMode = "active";
    if (activeChatRequestId !== null) {
      await cancelChat({ silent: true });
    }
    clearChatState();
    clearTraceState();
    currentRun = null;

    try {
      const runId = await invoke<number>("start_analysis_report", command.command);

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
      activeRunId = runId;

      await Promise.all([loadActiveRuns(), openRun(runId)]);
    } catch (error) {
      status = formatAppError("starting the analysis report", error);
    } finally {
      startingReport = false;
    }
  }

  async function cancelActiveRun(runId: number) {
    try {
      await invoke("cancel_analysis_run", { runId });
      status = `Cancelling analysis run ${runId}...`;
    } catch (error) {
      status = formatAppError("cancelling the analysis run", error);
    }
  }

  async function deleteSavedRun(run: AnalysisRunSummary) {
    const decision = runDeletionDecision(run);
    if (!decision.ok) {
      status = decision.status;
      return;
    }

    const confirmed = await openConfirmModal(decision.dialog);
    if (!confirmed) {
      return;
    }

    deletingRunIds = { ...deletingRunIds, [run.id]: true };
    try {
      if (activeChatRequestId !== null && activeChatRunId === run.id) {
        await cancelChat({ silent: true });
      }
      await invoke("delete_analysis_run", { runId: run.id });
      runs = runs.filter((entry) => entry.id !== run.id);
      activeRuns = activeRuns.filter((entry) => entry.id !== run.id);
      clearOpenedRunState(run.id);
      inspectorMode = "history";
      status = runDeletedStatus(run);
      await loadRuns();
    } catch (error) {
      status = formatAppError("deleting the saved run", error);
    } finally {
      const next = { ...deletingRunIds };
      delete next[run.id];
      deletingRunIds = next;
    }
  }

  async function askRunQuestion() {
    await chatWorkflow.askRunQuestion();
  }

  async function clearChatMessages() {
    await chatWorkflow.clearMessages();
  }

  async function syncSelectedSource(sourceId: number) {
    syncingIds = sourceActionPending(syncingIds, sourceId);
    try {
      const result = await syncSource(sourceId);
      status = sourceSyncStatus(result);

      await Promise.all([loadSourceCatalog(), loadActiveRuns(), loadRuns()]);

      if (selectedSourceId === String(sourceId)) {
        await loadSourceTopics(sourceId, { preserveSelection: true });
        await loadItems(sourceId);
      }
    } catch (error) {
      status = formatAppError("syncing the source", error);
    } finally {
      syncingIds = clearSourceActionPending(syncingIds, sourceId);
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
      await loadSourceTopics(Number(selectedSourceId), { preserveSelection: true });
      await loadItems(Number(selectedSourceId));
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
    const command = templateUpdateCommand(selectedTemplate, nextName, nextBody);
    if (!command.ok) {
      status = command.status;
      return;
    }

    savingTemplate = true;
    try {
      const updated = await invoke<AnalysisPromptTemplate>("update_analysis_prompt_template", {
        templateId: command.templateId,
        name: command.name,
        body: command.body,
      });
      status = templateUpdatedStatus(updated);
      await loadTemplates();
      selectedTemplateId = String(updated.id);
      bindEditorToTemplate(updated);
    } catch (error) {
      status = formatAppError("saving the template", error);
    } finally {
      savingTemplate = false;
    }
  }

  async function saveTemplateCopy(nextName = templateName, nextBody = templateBody) {
    const command = templateCopyCommand(nextName, nextBody);
    if (!command.ok) {
      status = command.status;
      return;
    }

    savingTemplate = true;
    try {
      const created = await invoke<AnalysisPromptTemplate>("create_analysis_prompt_template", {
        name: command.name,
        templateKind: "report",
        body: command.body,
      });
      status = templateCreatedStatus(created);
      await loadTemplates();
      selectedTemplateId = String(created.id);
      bindEditorToTemplate(created);
    } catch (error) {
      status = formatAppError("creating the template", error);
    } finally {
      savingTemplate = false;
    }
  }

  async function deleteTemplate() {
    const decision = templateDeleteDecision(selectedTemplate);
    if (!decision.ok) {
      status = decision.status;
      return;
    }
    const confirmed = await openConfirmModal({
      title: "Delete template?",
      message: `The template "${decision.name}" will be removed from the local app.`,
      confirmLabel: "Delete",
      cancelLabel: "Cancel",
      tone: "danger",
    });
    if (!confirmed) {
      return;
    }

    deletingTemplate = true;
    try {
      await invoke("delete_analysis_prompt_template", { templateId: decision.templateId });
      status = templateDeletedStatus(decision.name);
      await loadTemplates();
      const fallback = templateFallbackSelection(templates);
      selectedTemplateId = fallback.selectedTemplateId;
      bindEditorToTemplate(fallback.template);
    } catch (error) {
      status = formatAppError("deleting the template", error);
    } finally {
      deletingTemplate = false;
    }
  }

  async function saveGroupChanges() {
    const command = groupUpdateCommand(selectedGroup, groupName, groupMemberSourceIds);
    if (!command.ok) {
      status = command.status;
      return;
    }

    savingGroup = true;
    try {
      const updated = await invoke<AnalysisSourceGroup>("update_analysis_source_group", {
        groupId: command.groupId,
        name: command.name,
        sourceIds: command.sourceIds,
      });
      status = groupUpdatedStatus(updated);
      await loadGroups();
      selectedGroupId = String(updated.id);
      bindEditorToGroup(updated);
    } catch (error) {
      status = formatAppError("saving the source group", error);
    } finally {
      savingGroup = false;
    }
  }

  async function saveGroupCopy() {
    const command = groupCopyCommand(groupName, groupMemberSourceIds);
    if (!command.ok) {
      status = command.status;
      return;
    }

    savingGroup = true;
    try {
      const created = await invoke<AnalysisSourceGroup>("create_analysis_source_group", {
        name: command.name,
        sourceIds: command.sourceIds,
      });
      status = groupCreatedStatus(created);
      await loadGroups();
      selectedGroupId = String(created.id);
      bindEditorToGroup(created);
    } catch (error) {
      status = formatAppError("creating the source group", error);
    } finally {
      savingGroup = false;
    }
  }

  async function deleteGroup() {
    const decision = groupDeleteDecision(selectedGroup);
    if (!decision.ok) {
      status = decision.status;
      return;
    }
    const confirmed = await openConfirmModal({
      title: "Delete source group?",
      message: `The group "${decision.name}" will be removed, but its synced sources will stay available for analysis.`,
      confirmLabel: "Delete",
      cancelLabel: "Cancel",
      tone: "danger",
    });
    if (!confirmed) {
      return;
    }

    deletingGroup = true;
    try {
      await invoke("delete_analysis_source_group", { groupId: decision.groupId });
      status = groupDeletedStatus(decision.name);
      await loadGroups();
      const fallback = groupFallbackSelection(groups);
      selectedGroupId = fallback.selectedGroupId;
      bindEditorToGroup(fallback.group);
    } catch (error) {
      status = formatAppError("deleting the source group", error);
    } finally {
      deletingGroup = false;
    }
  }

  function startNewGroup() {
    selectedGroupId = "";
    bindEditorToGroup(null);
  }

  $effect(() => {
    if (historyScopeParams === null) {
      runs = [];
      return;
    }

    void loadRuns();
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

    void loadAccounts();
    void loadSourceCatalog().then(() => {
      if (selectedSourceId) {
        void loadSourceTopics(Number(selectedSourceId)).then(() =>
          loadItems(Number(selectedSourceId)),
        );
      }
    });
    void loadTemplates();
    void loadGroups();
    void loadActiveRuns();
    void loadTakeoutImportJobs();

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
    };
  });
</script>

{#if status}
  <StatusMessage tone={isErrorStatus(status) ? "error" : "default"} className="workspace-status">
    {status}
  </StatusMessage>
{/if}

<section class="analysis-workspace">
  <WorkspaceRail
    {sourceCatalog}
    {groups}
    {sourceMetrics}
    {loadingSourceCatalog}
    {loadingGroups}
    {railQuery}
    {filteredSourceCatalog}
    {filteredGroups}
    {analysisScope}
    {selectedSourceId}
    {selectedGroupId}
    {syncingIds}
    {deletingSourceIds}
    {startingTakeoutSourceIds}
    {takeoutJobsBySource}
    {formatTimestamp}
    {accountLabel}
    {sourceKindLabel}
    {membershipLabel}
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
    onOpenSourceManager={() => (sourceManagerOpen = true)}
    onDeleteSource={(source) => void deleteSource(source)}
  />

  <WorkspaceMain
    {analysisScope}
    currentSource={currentSource()}
    currentGroup={currentGroup()}
    currentSourceMetric={currentSourceMetric()}
    currentScopeTitle={currentScopeTitle()}
    currentScopeSummary={currentScopeSummary()}
    {periodFrom}
    {periodTo}
    {selectedTemplateId}
    {loadingTemplates}
    {templates}
    {outputLanguage}
    {modelOverride}
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
    {groupMemberSourceIds}
    {selectedGroup}
    {savingGroup}
    {deletingGroup}
    sourceMetricsList={Object.values(sourceMetrics)}
    {syncingIds}
    {formatTimestamp}
    {formatPeriod}
    {runTargetLabel}
    {statusTone}
    {reportLines}
    {phaseLabel}
    {accountLabel}
    {sourceKindLabel}
    {sourceSyncDisabledReason}
    {startOfDayUnix}
    {endOfDayUnix}
    {isGroupSourceSelected}
    onChangeSelectedTopicKey={(value) => void changeSelectedTopicKey(value)}
    onChangePeriodFrom={(value) => (periodFrom = value)}
    onChangePeriodTo={(value) => (periodTo = value)}
    onChangeSelectedTemplateId={(value) => (selectedTemplateId = value)}
    onChangeOutputLanguage={(value) => (outputLanguage = value)}
    onChangeModelOverride={(value) => (modelOverride = value)}
    onRunReport={() => void runReport()}
    onSyncCurrentSource={(sourceId) => void syncSelectedSource(sourceId)}
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
    grid-template-columns: minmax(260px, 320px) minmax(0, 1.6fr) minmax(320px, 430px);
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
      grid-template-columns: minmax(250px, 300px) minmax(0, 1fr);
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
