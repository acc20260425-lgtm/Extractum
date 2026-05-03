<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { open as openDialog } from "@tauri-apps/plugin-dialog";
  import StatusMessage from "$lib/components/ui/StatusMessage.svelte";
  import WorkspaceInspector from "$lib/components/analysis/workspace-inspector.svelte";
  import WorkspaceMain from "$lib/components/analysis/workspace-main.svelte";
  import WorkspaceRail from "$lib/components/analysis/workspace-rail.svelte";
  import SourceManagementDialog from "$lib/components/analysis/source-management-dialog.svelte";
  import { formatAppError } from "$lib/app-error";
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
    analysisTraceRefOrigin as traceRefOriginFromState,
    activeAnalysisRunIds,
    activeRunSyncDecision,
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
    isActiveRunStatus,
    isRunActive,
    isRunFocused,
    liveRunPhase,
    liveRunProgress,
    mergeAnalysisTraceRefs,
    notebookLmExportCompleteStatus,
    notebookLmExportInitialProgress,
    notebookLmExportProgressFromEvent,
    notebookLmExportRequestFromForm,
    normalizeSelectedTopicKey as normalizeTopicKey,
    pruneLiveRuns as pruneLiveRunMap,
    runActivePhase,
    runActiveProgress,
    selectedAnalysisGroup,
    selectedAnalysisTemplate,
    selectedAnalysisTraceRef,
    shouldShowTopicSelector as shouldShowTopicSelectorFromState,
    syncRunSnapshot as syncLiveRunSnapshot,
    takeoutImportEventDecision,
    upsertTakeoutImportJob,
    type AnalysisRunFilter,
    type LiveRunState,
    type NotebookLmExportProgressState,
  } from "$lib/analysis-state";
  import {
    appendPendingChatExchange,
    applyAnalysisChatEvent,
    chatTurnsFromMessages,
    dropPendingChatExchange,
    matchesActiveAnalysisChatEvent,
    type AnalysisChatState,
  } from "$lib/analysis-chat-state";
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
    AnalysisChatEvent,
    AnalysisChatMessage,
    AnalysisChatTurn,
    AnalysisPromptTemplate,
    AnalysisRunDetail,
    AnalysisRunEvent,
    AnalysisRunSummary,
    AnalysisSourceGroup,
    AnalysisSourceOption,
    AnalysisTraceData,
    AnalysisTraceRef,
    EventEnvelope,
  } from "$lib/types/analysis";
  import type {
    ForumTopicFilter,
    ItemRecord,
    NotebookLmExportEvent,
    NotebookLmExportResult,
    SourceForumTopicRecord,
    SourceRecord,
    CancelTakeoutImportResponse,
    StartTakeoutImportResponse,
    SyncResult,
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

  let sourceCatalog = $state<SourceRecord[]>([]);
  let sourceMetrics = $state<Record<number, AnalysisSourceOption>>({});
  let sourceItems = $state<ItemRecord[]>([]);
  let sourceTopics = $state<SourceForumTopicRecord[]>([]);
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
  let openRunRequestToken = 0;

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

  function sourceSyncDisabledReason(source: SourceRecord) {
    return getSourceSyncDisabledReason(source, accountStatuses);
  }

  function clearTraceState() {
    traceData = { refs: [] };
    savedTraceRefs = [];
    resolvedTraceRefs = [];
    selectedTraceRef = null;
  }

  function clearChatState() {
    chatMessages = [];
    chatQuestion = "";
    chatting = false;
    activeChatRequestId = null;
    activeChatRunId = null;
  }

  function currentChatState(): AnalysisChatState {
    return {
      messages: chatMessages,
      chatting,
      activeRequestId: activeChatRequestId,
      activeRunId: activeChatRunId,
    };
  }

  function assignChatState(next: AnalysisChatState) {
    chatMessages = next.messages;
    chatting = next.chatting;
    activeChatRequestId = next.activeRequestId;
    activeChatRunId = next.activeRunId;
  }

  function clearOpenedRunState(runId: number) {
    if (activeRunId !== runId && currentRun?.id !== runId) {
      return;
    }

    openRunRequestToken += 1;
    activeRunId = null;
    currentRun = null;
    clearTraceState();
    clearChatState();
    const nextLiveRuns = { ...liveRuns };
    delete nextLiveRuns[runId];
    liveRuns = nextLiveRuns;
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

  function isFocusedRun(runId: number) {
    return isRunFocused(runId, activeRunId, currentRun);
  }

  function currentTopicFilter(): ForumTopicFilter | null {
    return currentTopicFilterFromState(selectedTopicKey, sourceTopics);
  }

  function hasRealForumTopics(topics: SourceForumTopicRecord[] = sourceTopics) {
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
    topics: SourceForumTopicRecord[],
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

  function mergeTraceRefs(nextRefs: AnalysisTraceRef[]) {
    if (nextRefs.length === 0) return;
    traceData = { refs: mergeAnalysisTraceRefs(traceData.refs, nextRefs) };
  }

  function traceRefOrigin(ref: string) {
    return traceRefOriginFromState(ref, savedTraceRefs, resolvedTraceRefs);
  }

  async function focusTraceRef(ref: string) {
    if (!currentRun) return;

    inspectorMode = "trace";
    selectedTraceRef = ref;
    if (traceData.refs.some((entry) => entry.ref === ref)) {
      return;
    }

    try {
      const resolved = await invoke<AnalysisTraceRef[]>("resolve_analysis_trace_refs", {
        runId: currentRun.id,
        refs: [ref],
      });
      mergeTraceRefs(resolved);
      resolvedTraceRefs = [
        ...resolvedTraceRefs,
        ...resolved
          .map((entry) => entry.ref)
          .filter((entry) => !resolvedTraceRefs.includes(entry)),
      ];
      selectedTraceRef = ref;
    } catch (error) {
      status = formatAppError("resolving the trace reference", error);
    }
  }

  async function loadTrace(runId: number, requestToken?: number) {
    try {
      const nextTraceData = await invoke<AnalysisTraceData>("get_analysis_run_trace", { runId });
      if (requestToken !== undefined && requestToken !== openRunRequestToken) {
        return;
      }
      traceData = nextTraceData;
      savedTraceRefs = traceData.refs.map((ref) => ref.ref);
      resolvedTraceRefs = [];
      selectedTraceRef = traceData.refs[0]?.ref ?? null;
    } catch (error) {
      if (requestToken !== undefined && requestToken !== openRunRequestToken) {
        return;
      }
      clearTraceState();
      status = formatAppError("loading the analysis trace", error);
    }
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
        invoke<SourceRecord[]>("list_sources", { accountId: null }),
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
      const jobs = await invoke<TakeoutImportJobRecord[]>("list_takeout_source_import_jobs");
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
      const topics = await invoke<SourceForumTopicRecord[]>("list_source_forum_topics", {
        sourceId,
      });
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
      sourceItems = await invoke<ItemRecord[]>("get_items", {
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
    analysisScope = "single_source";
    selectedSourceId = String(sourceId);
    selectedTopicKey = "__all_topics__";
    inspectorMode = "history";
    await loadSourceTopics(sourceId);
    await loadItems(sourceId);
  }

  function selectGroup(groupId: number) {
    analysisScope = "source_group";
    selectedGroupId = String(groupId);
    sourceTopics = [];
    selectedTopicKey = "__all_topics__";
    inspectorMode = "history";
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
    const params = historyScopeParams;
    if (params === null) {
      runs = [];
      return;
    }

    loadingRuns = true;
    try {
      const summaries = await invoke<AnalysisRunSummary[]>("list_analysis_runs", {
        sourceId: params.sourceId,
        sourceGroupId: params.sourceGroupId,
        limit: 50,
      });
      runs = summaries.filter((run) => !isActiveRunStatus(run.status));
    } catch (error) {
      status = formatAppError("loading analysis runs", error);
    } finally {
      loadingRuns = false;
    }
  }

  function syncActiveRunState(summaries: AnalysisRunSummary[]) {
    const decision = activeRunSyncDecision(
      summaries,
      activeRunId,
      currentRun?.id ?? null,
    );

    for (const snapshot of decision.runSnapshots) {
      syncRunSnapshot(snapshot.runId, snapshot.status);
    }

    pruneLiveRuns(decision.activeRunIds, decision.preserveRunId);

    if (decision.runToOpen !== null) {
      void openRun(decision.runToOpen);
      return;
    }

    activeRunId = decision.nextActiveRunId;
  }

  async function loadActiveRuns() {
    loadingActiveRuns = true;
    try {
      const summaries = await invoke<AnalysisRunSummary[]>("list_active_analysis_runs");
      activeRuns = summaries;
      syncActiveRunState(summaries);
    } catch (error) {
      status = formatAppError("loading active analysis runs", error);
    } finally {
      loadingActiveRuns = false;
    }
  }

  async function cancelChat({ silent = false }: { silent?: boolean } = {}) {
    if (!activeChatRequestId) {
      return;
    }

    const requestId = activeChatRequestId;
    try {
      await invoke("cancel_llm_request", { requestId });
      if (!silent) {
        status = "Cancelling answer...";
      }
    } catch (error) {
      if (!silent) {
        status = formatAppError("cancelling the chat answer", error);
      }
    }
  }

  async function openRun(runId: number) {
    const requestToken = ++openRunRequestToken;
    inspectorMode = "history";

    if (activeChatRequestId !== null && activeChatRunId !== null && activeChatRunId !== runId) {
      await cancelChat({ silent: true });
      clearChatState();
    }

    activeRunId = runId;
    loadingRunDetail = true;
    try {
      const run = await invoke<AnalysisRunDetail | null>("get_analysis_run", { runId });
      if (requestToken !== openRunRequestToken) {
        return;
      }

      if (!run) {
        status = `Analysis run ${runId} was not found.`;
        if (currentRun?.id === runId) {
          currentRun = null;
        }
        return;
      }

      currentRun = run;
      syncRunSnapshot(run.id, run.status);
      await loadChatMessages(run.id, requestToken);
      if (requestToken !== openRunRequestToken) {
        return;
      }
      if (run.has_trace_data) {
        await loadTrace(run.id, requestToken);
      } else {
        clearTraceState();
      }
    } catch (error) {
      if (requestToken !== openRunRequestToken) {
        return;
      }
      status = formatAppError("loading the analysis run", error);
    } finally {
      if (requestToken === openRunRequestToken) {
        loadingRunDetail = false;
      }
    }
  }

  async function loadChatMessages(runId: number, requestToken?: number) {
    loadingChat = true;
    try {
      const messages = await invoke<AnalysisChatMessage[]>("list_analysis_chat_messages", { runId });
      if (requestToken !== undefined && requestToken !== openRunRequestToken) {
        return;
      }
      chatMessages = chatTurnsFromMessages(messages);
    } catch (error) {
      if (requestToken !== undefined && requestToken !== openRunRequestToken) {
        return;
      }
      chatMessages = [];
      status = formatAppError("loading analysis chat", error);
    } finally {
      if (requestToken === undefined || requestToken === openRunRequestToken) {
        loadingChat = false;
      }
    }
  }

  async function runReport() {
    if (analysisScope === "single_source" && !selectedSourceId) {
      status = "Select a source first.";
      return;
    }
    if (analysisScope === "source_group" && !selectedGroupId) {
      status = "Select a source group first.";
      return;
    }
    if (!selectedTemplateId) {
      status = "Select a report template first.";
      return;
    }
    if (!periodFrom || !periodTo) {
      status = "Select both dates first.";
      return;
    }
    if (startOfDayUnix(periodFrom) > endOfDayUnix(periodTo)) {
      status = "The start date must not be after the end date.";
      return;
    }
    if (!outputLanguage.trim()) {
      status = "Output language cannot be empty.";
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
      const runId = await invoke<number>("start_analysis_report", {
        sourceId: analysisScope === "single_source" ? Number(selectedSourceId) : null,
        sourceGroupId: analysisScope === "source_group" ? Number(selectedGroupId) : null,
        periodFrom: startOfDayUnix(periodFrom),
        periodTo: endOfDayUnix(periodTo),
        outputLanguage: outputLanguage.trim(),
        promptTemplateId: Number(selectedTemplateId),
        modelOverride: modelOverride.trim() ? modelOverride.trim() : null,
        profileId: null,
      });

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
    if (isActiveRunStatus(run.status)) {
      status = "Cancel or wait for this run before deleting it.";
      return;
    }

    const confirmed = await openConfirmModal({
      title: "Delete saved run?",
      message: `The saved report for "${runTargetLabel(run)}" and its follow-up chat history will be removed from this device.`,
      confirmLabel: "Delete",
      cancelLabel: "Cancel",
      tone: "danger",
    });
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
      status = `Saved run ${run.id} deleted.`;
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
    if (!currentRun || currentRun.status !== "completed") {
      status = "Open a completed report first.";
      return;
    }
    if (!chatQuestion.trim()) {
      status = "Question cannot be empty.";
      return;
    }

    const question = chatQuestion.trim();
    chatMessages = appendPendingChatExchange(chatMessages, question);
    chatQuestion = "";
    chatting = true;
    activeChatRunId = currentRun.id;

    try {
      const requestId = await invoke<string>("ask_analysis_run_question", {
        runId: currentRun.id,
        question,
        modelOverride: modelOverride.trim() ? modelOverride.trim() : null,
        profileId: null,
      });
      activeChatRequestId = requestId;
    } catch (error) {
      chatMessages = dropPendingChatExchange(chatMessages);
      chatting = false;
      activeChatRunId = null;
      activeChatRequestId = null;
      status = formatAppError("starting the chat answer", error);
    }
  }

  async function clearChatMessages() {
    if (!currentRun) {
      status = "Open a run first.";
      return;
    }
    const confirmed = await openConfirmModal({
      title: "Clear chat history?",
      message: "This will remove all saved follow-up messages for the currently opened run.",
      confirmLabel: "Clear history",
      cancelLabel: "Cancel",
      tone: "danger",
    });
    if (!confirmed) {
      return;
    }

    clearingChat = true;
    try {
      await invoke("clear_analysis_chat_messages", { runId: currentRun.id });
      chatMessages = [];
      status = "Saved chat history cleared.";
    } catch (error) {
      status = formatAppError("clearing analysis chat", error);
    } finally {
      clearingChat = false;
    }
  }

  async function syncSelectedSource(sourceId: number) {
    syncingIds = { ...syncingIds, [sourceId]: true };
    try {
      const result = await invoke<SyncResult>("sync_source", { sourceId });
      status =
        `Sync complete: inserted ${result.inserted}, skipped ${result.skipped}.` +
        (result.initial_sync_policy_applied
          ? ` First sync policy applied: ${result.initial_sync_policy_applied}.`
          : "") +
        (result.warnings.length > 0
          ? ` Warnings: ${result.warnings.join(" ")}`
          : "");

      await Promise.all([loadSourceCatalog(), loadActiveRuns(), loadRuns()]);

      if (selectedSourceId === String(sourceId)) {
        await loadSourceTopics(sourceId, { preserveSelection: true });
        await loadItems(sourceId);
      }
    } catch (error) {
      status = formatAppError("syncing the source", error);
    } finally {
      const next = { ...syncingIds };
      delete next[sourceId];
      syncingIds = next;
    }
  }

  async function startTakeoutImport(sourceId: number) {
    startingTakeoutSourceIds = { ...startingTakeoutSourceIds, [sourceId]: true };
    try {
      await invoke<StartTakeoutImportResponse>("start_takeout_source_import", { sourceId });
      status = "Takeout import started.";
    } catch (error) {
      status = formatAppError("starting Takeout import", error);
    } finally {
      const next = { ...startingTakeoutSourceIds };
      delete next[sourceId];
      startingTakeoutSourceIds = next;
    }
  }

  async function cancelTakeoutImport(jobId: string) {
    try {
      const result = await invoke<CancelTakeoutImportResponse>(
        "cancel_takeout_source_import",
        { jobId },
      );
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

  async function deleteSource(source: SourceRecord) {
    const sourceName = source.title ?? source.external_id;
    const confirmed = await openConfirmModal({
      title: "Delete source?",
      message:
        `The source "${sourceName}" and its synced local items will be removed from Extractum.\n\n` +
        "Saved analysis snapshots remain available as frozen run artifacts.",
      confirmLabel: "Delete",
      cancelLabel: "Cancel",
      tone: "danger",
    });
    if (!confirmed) {
      return;
    }

    deletingSourceIds = { ...deletingSourceIds, [source.id]: true };
    try {
      await invoke("delete_source", { sourceId: source.id });
      status = `Source "${sourceName}" deleted.`;

      if (selectedSourceId === String(source.id)) {
        sourceItems = [];
        currentRun = null;
        activeRunId = null;
        clearTraceState();
        clearChatState();
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

      const result = await invoke<NotebookLmExportResult>("export_source_to_notebooklm", {
        request,
      });
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
    const current = selectedTemplate;
    if (!current) {
      status = "Select a template first.";
      return;
    }
    if (current.is_builtin) {
      status = "Built-in templates cannot be edited directly. Save a copy instead.";
      return;
    }
    if (!nextName.trim() || !nextBody.trim()) {
      status = "Template name and body cannot be empty.";
      return;
    }

    savingTemplate = true;
    try {
      const updated = await invoke<AnalysisPromptTemplate>("update_analysis_prompt_template", {
        templateId: current.id,
        name: nextName.trim(),
        body: nextBody.trim(),
      });
      status = `Template "${updated.name}" saved.`;
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
    if (!nextName.trim() || !nextBody.trim()) {
      status = "Template name and body cannot be empty.";
      return;
    }

    savingTemplate = true;
    try {
      const created = await invoke<AnalysisPromptTemplate>("create_analysis_prompt_template", {
        name: nextName.trim(),
        templateKind: "report",
        body: nextBody.trim(),
      });
      status = `Template "${created.name}" created.`;
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
    const current = selectedTemplate;
    if (!current) {
      status = "Select a template first.";
      return;
    }
    if (current.is_builtin) {
      status = "Built-in templates cannot be deleted.";
      return;
    }
    const confirmed = await openConfirmModal({
      title: "Delete template?",
      message: `The template "${current.name}" will be removed from the local app.`,
      confirmLabel: "Delete",
      cancelLabel: "Cancel",
      tone: "danger",
    });
    if (!confirmed) {
      return;
    }

    deletingTemplate = true;
    try {
      await invoke("delete_analysis_prompt_template", { templateId: current.id });
      status = `Template "${current.name}" deleted.`;
      await loadTemplates();
      const fallback = templates[0] ?? null;
      selectedTemplateId = fallback ? String(fallback.id) : "";
      bindEditorToTemplate(fallback);
    } catch (error) {
      status = formatAppError("deleting the template", error);
    } finally {
      deletingTemplate = false;
    }
  }

  async function saveGroupChanges() {
    const current = selectedGroup;
    if (!current) {
      status = "Select a source group first.";
      return;
    }
    if (!groupName.trim()) {
      status = "Group name cannot be empty.";
      return;
    }
    if (groupMemberSourceIds.length === 0) {
      status = "Select at least one source for the group.";
      return;
    }

    savingGroup = true;
    try {
      const updated = await invoke<AnalysisSourceGroup>("update_analysis_source_group", {
        groupId: current.id,
        name: groupName.trim(),
        sourceIds: groupMemberSourceIds,
      });
      status = `Source group "${updated.name}" saved.`;
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
    if (!groupName.trim()) {
      status = "Group name cannot be empty.";
      return;
    }
    if (groupMemberSourceIds.length === 0) {
      status = "Select at least one source for the group.";
      return;
    }

    savingGroup = true;
    try {
      const created = await invoke<AnalysisSourceGroup>("create_analysis_source_group", {
        name: groupName.trim(),
        sourceIds: groupMemberSourceIds,
      });
      status = `Source group "${created.name}" created.`;
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
    const current = selectedGroup;
    if (!current) {
      status = "Select a source group first.";
      return;
    }
    const confirmed = await openConfirmModal({
      title: "Delete source group?",
      message: `The group "${current.name}" will be removed, but its synced sources will stay available for analysis.`,
      confirmLabel: "Delete",
      cancelLabel: "Cancel",
      tone: "danger",
    });
    if (!confirmed) {
      return;
    }

    deletingGroup = true;
    try {
      await invoke("delete_analysis_source_group", { groupId: current.id });
      status = `Source group "${current.name}" deleted.`;
      await loadGroups();
      const fallback = groups[0] ?? null;
      selectedGroupId = fallback ? String(fallback.id) : "";
      bindEditorToGroup(fallback);
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

    void listen<AnalysisRunEvent>("analysis://run", ({ payload }: EventEnvelope<AnalysisRunEvent>) => {
      if (disposed) {
        return;
      }

      applyRunEvent(payload);

      if (payload.chunk_summary) {
        inspectorMode = "chunks";
      }

      if (activeRunId === null) {
        activeRunId = payload.run_id;
        inspectorMode = "active";
        void openRun(payload.run_id);
      }

      if (
        payload.kind === "queued" ||
        payload.kind === "started" ||
        payload.kind === "progress"
      ) {
        if (payload.message && (activeRunId === null || isFocusedRun(payload.run_id))) {
          status = payload.message;
        }
      }

      if (
        payload.kind === "completed" ||
        payload.kind === "failed" ||
        payload.kind === "cancelled"
      ) {
        if (payload.message && (activeRunId === null || isFocusedRun(payload.run_id))) {
          status = payload.message;
        } else if (payload.error && (activeRunId === null || isFocusedRun(payload.run_id))) {
          status = `Analysis failed: ${payload.error}`;
        }

        void loadActiveRuns();
        void loadRuns();

        if (activeRunId === payload.run_id || currentRun?.id === payload.run_id) {
          void openRun(payload.run_id);
        }
      }
    }).then((unlisten) => {
      if (disposed) {
        unlisten();
        return;
      }
      detachAnalysisListener = unlisten;
    });

    void listen<AnalysisChatEvent>("analysis://chat", ({ payload }: EventEnvelope<AnalysisChatEvent>) => {
      if (
        disposed ||
        !matchesActiveAnalysisChatEvent(payload, activeChatRunId, activeChatRequestId)
      ) {
        return;
      }

      const reduction = applyAnalysisChatEvent(currentChatState(), payload);
      assignChatState(reduction.state);
      if (reduction.reloadRunId !== null) {
        void loadChatMessages(reduction.reloadRunId);
      }
      if (reduction.status !== null) {
        status = reduction.status;
      }
    }).then((unlisten) => {
      if (disposed) {
        unlisten();
        return;
      }
      detachChatListener = unlisten;
    });

    void listen<NotebookLmExportEvent>("notebooklm://export", ({ payload }: EventEnvelope<NotebookLmExportEvent>) => {
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

    void listen<TakeoutImportEvent>("sources://takeout-import", ({ payload }: EventEnvelope<TakeoutImportEvent>) => {
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
