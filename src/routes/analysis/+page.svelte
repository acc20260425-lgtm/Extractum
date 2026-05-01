<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import ActiveRunList from "$lib/components/analysis/active-run-list.svelte";
  import ChatPanel from "$lib/components/analysis/chat-panel.svelte";
  import ChunkSummaries from "$lib/components/analysis/chunk-summaries.svelte";
  import ReportViewer from "$lib/components/analysis/report-viewer.svelte";
  import RunHistory from "$lib/components/analysis/run-history.svelte";
  import SourceGroupEditor from "$lib/components/analysis/source-group-editor.svelte";
  import TemplateEditor from "$lib/components/analysis/template-editor.svelte";
  import TracePanel from "$lib/components/analysis/trace-panel.svelte";
  import SourceMessagesPanel from "$lib/components/source-messages-panel.svelte";
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import Select from "$lib/components/ui/Select.svelte";
  import StatusMessage from "$lib/components/ui/StatusMessage.svelte";
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
  import type { AccountRecord, AccountRuntimeStatus } from "$lib/types/accounts";
  import type {
    AnalysisChatEvent,
    AnalysisChatMessage,
    AnalysisChatTurn,
    AnalysisChunkSummaryEvent,
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
  import type { ItemRecord, SourceRecord, SyncResult } from "$lib/types/sources";

  type LiveRunState = {
    phase: string;
    progress: string;
    queuePosition: number | null;
    chunkSummaries: AnalysisChunkSummaryEvent[];
    streamedOutput: string;
  };

  type InspectorMode = "active" | "history" | "trace" | "chunks";

  function createEmptyLiveRunState(): LiveRunState {
    return {
      phase: "",
      progress: "",
      queuePosition: null,
      chunkSummaries: [],
      streamedOutput: "",
    };
  }

  let sourceCatalog = $state<SourceRecord[]>([]);
  let sourceMetrics = $state<Record<number, AnalysisSourceOption>>({});
  let sourceItems = $state<ItemRecord[]>([]);
  let accounts = $state<AccountRecord[]>([]);
  let accountStatuses = $state<Record<number, AccountRuntimeStatus>>({});
  let templates = $state<AnalysisPromptTemplate[]>([]);
  let runs = $state<AnalysisRunSummary[]>([]);
  let activeRuns = $state<AnalysisRunSummary[]>([]);
  let groups = $state<AnalysisSourceGroup[]>([]);

  let loadingSourceCatalog = $state(false);
  let loadingItems = $state(false);
  let loadingTemplates = $state(false);
  let loadingRuns = $state(false);
  let loadingActiveRuns = $state(false);
  let loadingGroups = $state(false);
  let loadingRunDetail = $state(false);
  let loadingChat = $state(false);

  let railQuery = $state("");
  let inspectorMode = $state<InspectorMode>("history");

  let selectedSourceId = $state("");
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
  let runFilter = $state<"all" | "completed" | "failed">("all");
  let historyScope = $state<"all" | "current">("all");
  let chatQuestion = $state("");
  let chatMessages = $state<AnalysisChatTurn[]>([]);
  let chatting = $state(false);
  let activeChatRequestId = $state<string | null>(null);
  let activeChatRunId = $state<number | null>(null);
  let clearingChat = $state(false);
  let syncingIds = $state<Record<number, boolean>>({});
  let statusTimer: ReturnType<typeof setTimeout> | null = null;
  let openRunRequestToken = 0;

  function isErrorStatus(value: string) {
    return value.startsWith("Error") || value.startsWith("Analysis failed");
  }

  function isActiveRunStatus(value: string) {
    return value === "queued" || value === "running";
  }

  function currentSource() {
    if (!selectedSourceId) return null;
    return sourceCatalog.find((source) => source.id === Number(selectedSourceId)) ?? null;
  }

  function currentSourceMetric() {
    const source = currentSource();
    return source ? sourceMetrics[source.id] ?? null : null;
  }

  function currentGroup() {
    if (!selectedGroupId) return null;
    return groups.find((group) => group.id === Number(selectedGroupId)) ?? null;
  }

  function currentScopeTitle() {
    if (analysisScope === "source_group") {
      return currentGroup()?.name ?? "Source group";
    }
    return currentSource()?.title ?? currentSource()?.external_id ?? "Source";
  }

  function currentScopeSummary() {
    if (analysisScope === "source_group") {
      const group = currentGroup();
      if (!group) return "Select a saved source group to run a cross-source report.";
      return `${group.members.length} sources in this group workspace.`;
    }

    const source = currentSource();
    const metrics = currentSourceMetric();
    if (!source) return "Select a synced source to inspect messages and launch a report.";
    if (metrics) {
      return `${metrics.item_count} synced messages available locally for analysis.`;
    }
    return "This source is available in the workspace but has no synced message count yet.";
  }

  function accountLabel(accountId: number | null) {
    if (accountId === null) return "No account";
    return accounts.find((account) => account.id === accountId)?.label ?? `Account #${accountId}`;
  }

  function runtimeStatus(accountId: number | null) {
    if (accountId === null) return null;
    return accountStatuses[accountId] ?? null;
  }

  function runtimeBadge(runtime: AccountRuntimeStatus | null) {
    if (!runtime) return "";
    if (runtime.status === "restoring") return "restoring";
    if (runtime.status === "reauth_required") return "sign-in needed";
    if (runtime.status === "restore_failed") return "restore failed";
    if (runtime.status === "not_initialized") return "offline";
    return "";
  }

  function sourceKindLabel(kind: string) {
    switch (kind) {
      case "channel":
        return "channel";
      case "supergroup":
        return "supergroup";
      case "group":
        return "group";
      default:
        return "telegram";
    }
  }

  function membershipLabel(kind: string, isMember: boolean) {
    if (kind === "channel") {
      return isMember ? "subscribed" : "not subscribed";
    }
    return isMember ? "member" : "not a member";
  }

  function sourceInitial(source: SourceRecord) {
    return (source.title ?? source.external_id).trim().charAt(0).toUpperCase() || "#";
  }

  function sourceSyncDisabledReason(source: SourceRecord) {
    const runtime = runtimeStatus(source.account_id);
    if (source.account_id === null) return "Source is not linked to an account.";
    if (!runtime || runtime.status === "not_initialized") {
      return "Initialize this account before syncing.";
    }
    if (runtime.status === "restoring") {
      return "This account is still restoring.";
    }
    if (runtime.status === "reauth_required") {
      return "Sign in to this account again before syncing.";
    }
    if (runtime.status === "restore_failed") {
      return runtime.message ?? "The saved Telegram session could not be restored.";
    }
    return null;
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

  function dropPendingChatExchange() {
    if (
      chatMessages.length >= 2 &&
      chatMessages[chatMessages.length - 1]?.role === "assistant" &&
      chatMessages[chatMessages.length - 2]?.role === "user"
    ) {
      chatMessages = chatMessages.slice(0, -2);
    }
  }

  function getLiveRunState(runId: number) {
    return liveRuns[runId] ?? createEmptyLiveRunState();
  }

  function updateLiveRunState(
    runId: number,
    updater: (current: LiveRunState) => LiveRunState,
  ) {
    const current = getLiveRunState(runId);
    liveRuns = {
      ...liveRuns,
      [runId]: updater(current),
    };
  }

  function syncRunSnapshot(runId: number, runStatus: string) {
    updateLiveRunState(runId, (current) => ({
      ...current,
      phase: runStatus,
      progress: isActiveRunStatus(runStatus) ? current.progress : "",
      queuePosition: isActiveRunStatus(runStatus) ? current.queuePosition : null,
    }));
  }

  function pruneLiveRuns(activeRunIds: number[], preserveRunId: number | null = null) {
    const keepIds = new Set(activeRunIds);
    if (preserveRunId !== null) {
      keepIds.add(preserveRunId);
    }

    liveRuns = Object.fromEntries(
      Object.entries(liveRuns).filter(([runId]) => keepIds.has(Number(runId))),
    );
  }

  function livePhase(runId: number) {
    return liveRuns[runId]?.phase ?? "";
  }

  function liveProgress(runId: number) {
    return liveRuns[runId]?.progress ?? "";
  }

  function isFocusedRun(runId: number) {
    return activeRunId === runId || currentRun?.id === runId;
  }

  function formatRunProgress(payload: AnalysisRunEvent, currentProgress: string) {
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

  function applyRunEvent(payload: AnalysisRunEvent) {
    updateLiveRunState(payload.run_id, (current) => {
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
        progress: formatRunProgress(payload, current.progress),
        queuePosition: payload.queue_position,
        chunkSummaries: nextSummaries,
        streamedOutput: payload.delta
          ? `${current.streamedOutput}${payload.delta}`
          : current.streamedOutput,
      };
    });
  }

  const activeRunIds = $derived.by(() => activeRuns.map((run) => run.id));

  const focusedLiveRun = $derived.by(() => {
    if (activeRunId === null) return null;
    return liveRuns[activeRunId] ?? null;
  });

  const activePhase = $derived.by(() => focusedLiveRun?.phase || currentRun?.status || "");
  const activeProgress = $derived.by(() => focusedLiveRun?.progress || "");
  const focusedChunkSummaries = $derived.by(() => focusedLiveRun?.chunkSummaries ?? []);
  const focusedStreamedOutput = $derived.by(() => {
    if (focusedLiveRun?.streamedOutput) {
      return focusedLiveRun.streamedOutput;
    }

    return currentRun?.result_markdown ?? "";
  });

  const selectedRunIsActive = $derived.by(
    () => activeRunId !== null && activeRunIds.includes(activeRunId),
  );

  const canCancelCurrentRun = $derived.by(
    () => activeRunId !== null && activeRunIds.includes(activeRunId),
  );

  const selectedTemplate = $derived.by(() => {
    const templateId = selectedTemplateId ? Number(selectedTemplateId) : null;
    if (templateId === null) return null;
    return templates.find((template) => template.id === templateId) ?? null;
  });

  const selectedGroup = $derived.by(() => {
    const groupId = selectedGroupId ? Number(selectedGroupId) : null;
    if (groupId === null) return null;
    return groups.find((group) => group.id === groupId) ?? null;
  });

  const selectedTrace = $derived.by(() => {
    if (!selectedTraceRef) return null;
    return traceData.refs.find((ref) => ref.ref === selectedTraceRef) ?? null;
  });

  const historyScopeParams = $derived.by(() => {
    if (historyScope === "all") {
      return {
        sourceId: null as number | null,
        sourceGroupId: null as number | null,
      };
    }

    if (analysisScope === "single_source" && selectedSourceId) {
      return {
        sourceId: Number(selectedSourceId),
        sourceGroupId: null as number | null,
      };
    }

    if (analysisScope === "source_group" && selectedGroupId) {
      return {
        sourceId: null as number | null,
        sourceGroupId: Number(selectedGroupId),
      };
    }

    return null;
  });

  const filteredRuns = $derived.by(() => {
    if (runFilter === "all") return runs;
    return runs.filter((run) => run.status === runFilter);
  });

  const filteredSourceCatalog = $derived.by(() => {
    const query = railQuery.trim().toLocaleLowerCase();
    return sourceCatalog.filter((source) => {
      if (!query) return true;
      return (
        (source.title ?? source.external_id).toLocaleLowerCase().includes(query) ||
        accountLabel(source.account_id).toLocaleLowerCase().includes(query)
      );
    });
  });

  const filteredGroups = $derived.by(() => {
    const query = railQuery.trim().toLocaleLowerCase();
    return groups.filter((group) => {
      if (!query) return true;
      return group.name.toLocaleLowerCase().includes(query);
    });
  });

  function bindEditorToTemplate(template: AnalysisPromptTemplate | null) {
    if (!template) {
      editorBoundTemplateId = null;
      templateName = "";
      templateBody = "";
      return;
    }

    editorBoundTemplateId = template.id;
    templateName = template.name;
    templateBody = template.body;
  }

  function bindEditorToGroup(group: AnalysisSourceGroup | null) {
    if (!group) {
      editorBoundGroupId = null;
      groupName = "";
      groupMemberSourceIds = [];
      return;
    }

    editorBoundGroupId = group.id;
    groupName = group.name;
    groupMemberSourceIds = group.members.map((member) => member.source_id);
  }

  function isGroupSourceSelected(sourceId: number) {
    return groupMemberSourceIds.includes(sourceId);
  }

  function toggleGroupSource(sourceId: number) {
    if (groupMemberSourceIds.includes(sourceId)) {
      groupMemberSourceIds = groupMemberSourceIds.filter((id) => id !== sourceId);
      return;
    }

    groupMemberSourceIds = [...groupMemberSourceIds, sourceId].sort((a, b) => a - b);
  }

  function mergeTraceRefs(nextRefs: AnalysisTraceRef[]) {
    if (nextRefs.length === 0) return;
    const merged = [...traceData.refs];
    for (const nextRef of nextRefs) {
      if (!merged.some((existing) => existing.ref === nextRef.ref)) {
        merged.push(nextRef);
      }
    }
    merged.sort((left, right) => left.published_at - right.published_at);
    traceData = { refs: merged };
  }

  function traceRefOrigin(ref: string) {
    if (savedTraceRefs.includes(ref)) return "saved";
    if (resolvedTraceRefs.includes(ref)) return "resolved";
    return "unknown";
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

  async function loadItems(sourceId: number) {
    loadingItems = true;
    try {
      sourceItems = await invoke<ItemRecord[]>("get_items", {
        sourceId,
        limit: 120,
        beforePublishedAt: null,
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
    inspectorMode = "history";
    await loadItems(sourceId);
  }

  function selectGroup(groupId: number) {
    analysisScope = "source_group";
    selectedGroupId = String(groupId);
    inspectorMode = "history";
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
    for (const run of summaries) {
      syncRunSnapshot(run.id, run.status);
    }

    pruneLiveRuns(
      summaries.map((run) => run.id),
      currentRun?.id ?? null,
    );

    if (currentRun !== null) {
      return;
    }

    const selectedRunIsStillActive =
      activeRunId !== null && summaries.some((run) => run.id === activeRunId);

    if (!selectedRunIsStillActive && summaries.length > 0) {
      void openRun(summaries[0].id);
      return;
    }

    if (summaries.length === 0) {
      activeRunId = null;
    }
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
      chatMessages = messages.map((message) => ({
        role: message.role,
        content: message.content,
      }));
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
    chatMessages = [
      ...chatMessages,
      { role: "user", content: question },
      { role: "assistant", content: "" },
    ];
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
      dropPendingChatExchange();
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
          : "");

      await Promise.all([loadSourceCatalog(), loadActiveRuns(), loadRuns()]);

      if (selectedSourceId === String(sourceId)) {
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

    void loadAccounts();
    void loadSourceCatalog().then(() => {
      if (selectedSourceId) {
        void loadItems(Number(selectedSourceId));
      }
    });
    void loadTemplates();
    void loadGroups();
    void loadActiveRuns();

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
        payload.run_id !== activeChatRunId ||
        (activeChatRequestId !== null && payload.request_id !== activeChatRequestId)
      ) {
        return;
      }

      if (payload.kind === "queued" || payload.kind === "started") {
        if (payload.message) {
          status = payload.message;
        }
        return;
      }

      if (payload.kind === "delta") {
        const lastIndex = chatMessages.length - 1;
        if (lastIndex >= 0 && chatMessages[lastIndex]?.role === "assistant") {
          const updated = [...chatMessages];
          updated[lastIndex] = {
            role: "assistant",
            content: `${updated[lastIndex].content}${payload.delta ?? ""}`,
          };
          chatMessages = updated;
        }
        return;
      }

      if (payload.kind === "completed") {
        chatting = false;
        activeChatRequestId = null;
        if (activeChatRunId !== null) {
          void loadChatMessages(activeChatRunId);
        }
        activeChatRunId = null;
        if (payload.message) {
          status = payload.message;
        }
        return;
      }

      if (payload.kind === "failed" || payload.kind === "cancelled") {
        chatting = false;
        activeChatRequestId = null;
        activeChatRunId = null;
        dropPendingChatExchange();
        status =
          payload.kind === "cancelled"
            ? payload.message ?? "Answer cancelled."
            : payload.error
              ? `Analysis chat failed: ${payload.error}`
              : "Analysis chat failed.";
      }
    }).then((unlisten) => {
      if (disposed) {
        unlisten();
        return;
      }
      detachChatListener = unlisten;
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
    };
  });
</script>

{#if status}
  <StatusMessage tone={isErrorStatus(status) ? "error" : "default"} className="workspace-status">
    {status}
  </StatusMessage>
{/if}

<section class="analysis-workspace">
  <aside class="rail">
    <div class="rail-header">
      <div>
        <span class="eyebrow">Research context</span>
        <h1>Workspace</h1>
      </div>
      <Badge variant="info">{sourceCatalog.length + groups.length} items</Badge>
    </div>

    <Input
      type="search"
      value={railQuery}
      placeholder="Search sources or groups"
      oninput={(event) => (railQuery = (event.currentTarget as HTMLInputElement).value)}
      className="rail-search"
    />

    <div class="rail-section">
      <div class="rail-section-title">
        <span>Sources</span>
        <small>{filteredSourceCatalog.length}</small>
      </div>
      <div class="rail-list">
        {#if loadingSourceCatalog}
          <div class="rail-empty">Loading sources...</div>
        {:else if filteredSourceCatalog.length === 0}
          <div class="rail-empty">No sources match the current search.</div>
        {:else}
          {#each filteredSourceCatalog as source (source.id)}
            {@const metrics = sourceMetrics[source.id]}
            {@const syncReason = sourceSyncDisabledReason(source)}
            {@const runtime = runtimeStatus(source.account_id)}
            {@const isSelected = analysisScope === "single_source" && selectedSourceId === String(source.id)}
            <article class:selected={isSelected} class="rail-row">
              <button class="rail-row-main" type="button" onclick={() => void selectSource(source.id)}>
                <div class="rail-avatar" aria-hidden="true">
                  {#if source.avatar_data_url}
                    <img src={source.avatar_data_url} alt="" loading="lazy" />
                  {:else}
                    <span>{sourceInitial(source)}</span>
                  {/if}
                </div>
                <div class="rail-copy">
                  <div class="rail-copy-top">
                    <strong>{source.title ?? source.external_id}</strong>
                    {#if metrics?.last_synced_at}
                      <span>{formatTimestamp(metrics.last_synced_at)}</span>
                    {/if}
                  </div>
                  <div class="rail-copy-meta">
                    <span>{accountLabel(source.account_id)}</span>
                    <span>{sourceKindLabel(source.telegram_source_kind)}</span>
                    {#if metrics}
                      <span>{metrics.item_count} msgs</span>
                    {/if}
                  </div>
                </div>
              </button>
              <div class="rail-row-actions">
                <Badge>{membershipLabel(source.telegram_source_kind, source.is_member)}</Badge>
                {#if runtimeBadge(runtime)}
                  <Badge variant="warning" title={runtime?.message ?? undefined}>{runtimeBadge(runtime)}</Badge>
                {/if}
                <Button
                  size="sm"
                  variant="secondary"
                  onclick={() => void syncSelectedSource(source.id)}
                  disabled={!!syncingIds[source.id] || syncReason !== null}
                  title={syncReason ?? undefined}
                >
                  {syncingIds[source.id] ? "Syncing..." : "Sync"}
                </Button>
              </div>
            </article>
          {/each}
        {/if}
      </div>
    </div>

    <div class="rail-section">
      <div class="rail-section-title">
        <span>Groups</span>
        <small>{filteredGroups.length}</small>
      </div>
      <div class="rail-list">
        {#if loadingGroups}
          <div class="rail-empty">Loading groups...</div>
        {:else if filteredGroups.length === 0}
          <div class="rail-empty">No groups match the current search.</div>
        {:else}
          {#each filteredGroups as group (group.id)}
            <button
              class:selected={analysisScope === "source_group" && selectedGroupId === String(group.id)}
              class="rail-row rail-group-row"
              type="button"
              onclick={() => selectGroup(group.id)}
            >
              <div class="rail-avatar group-avatar" aria-hidden="true">
                <span>{group.name.trim().charAt(0).toUpperCase() || "G"}</span>
              </div>
              <div class="rail-copy">
                <div class="rail-copy-top">
                  <strong>{group.name}</strong>
                  <span>{group.members.length} src</span>
                </div>
                <div class="rail-copy-meta">
                  <span>Saved source group</span>
                  <span>Updated {formatTimestamp(group.updated_at)}</span>
                </div>
              </div>
            </button>
          {/each}
        {/if}
      </div>
    </div>
  </aside>

  <section class="center-pane">
    <div class="scope-hero">
      <div class="scope-hero-copy">
        <span class="eyebrow">{analysisScope === "source_group" ? "Source group workspace" : "Source workspace"}</span>
        <h2>{currentScopeTitle()}</h2>
        <p>{currentScopeSummary()}</p>
      </div>
      <div class="scope-hero-meta">
        {#if analysisScope === "single_source" && currentSource()}
          <Badge variant="info">{sourceKindLabel(currentSource()!.telegram_source_kind)}</Badge>
          <Badge>{accountLabel(currentSource()!.account_id)}</Badge>
        {/if}
        {#if analysisScope === "source_group" && currentGroup()}
          <Badge variant="info">{currentGroup()!.members.length} sources</Badge>
        {/if}
        {#if currentRun}
          <Badge variant={statusTone(currentRun.status)}>Run #{currentRun.id}</Badge>
        {/if}
      </div>
    </div>

    <div class="scope-facts">
      <div class="scope-fact">
        <span class="scope-fact-label">Scope</span>
        <strong>{analysisScope === "source_group" ? "Group analysis" : "Single source"}</strong>
      </div>
      <div class="scope-fact">
        <span class="scope-fact-label">Window</span>
        <strong>{formatPeriod(startOfDayUnix(periodFrom), endOfDayUnix(periodTo))}</strong>
      </div>
      <div class="scope-fact">
        <span class="scope-fact-label">Context</span>
        <strong>
          {#if analysisScope === "source_group" && currentGroup()}
            {currentGroup()!.members.length} sources
          {:else if currentSourceMetric()}
            {currentSourceMetric()!.item_count} synced messages
          {:else}
            Awaiting synced context
          {/if}
        </strong>
      </div>
      <div class="scope-fact">
        <span class="scope-fact-label">Output</span>
        <strong>{outputLanguage || "Default language"}</strong>
      </div>
    </div>

    <div class="controls-panel">
      <div class="controls-grid">
        <label>Period from
          <Input
            type="date"
            value={periodFrom}
            oninput={(event) => (periodFrom = (event.currentTarget as HTMLInputElement).value)}
          />
        </label>
        <label>Period to
          <Input
            type="date"
            value={periodTo}
            oninput={(event) => (periodTo = (event.currentTarget as HTMLInputElement).value)}
          />
        </label>
        <label>Prompt template
          <Select
            value={selectedTemplateId}
            disabled={loadingTemplates}
            onchange={(event) => (selectedTemplateId = (event.currentTarget as HTMLSelectElement).value)}
          >
            {#if loadingTemplates}
              <option value="">Loading templates...</option>
            {:else if templates.length === 0}
              <option value="">No report templates available</option>
            {/if}
            {#each templates as template (template.id)}
              <option value={String(template.id)}>
                {template.name}{template.is_builtin ? " - builtin" : ""}
              </option>
            {/each}
          </Select>
        </label>
        <label>Output language
          <Input
            type="text"
            value={outputLanguage}
            placeholder="Russian"
            oninput={(event) => (outputLanguage = (event.currentTarget as HTMLInputElement).value)}
          />
        </label>
      </div>

      <div class="controls-bottom">
        <label class="model-field">Model override
          <Input
            type="text"
            value={modelOverride}
            placeholder="Use active profile default model"
            oninput={(event) => (modelOverride = (event.currentTarget as HTMLInputElement).value)}
          />
        </label>
        <div class="controls-actions">
          <Button onclick={runReport} disabled={startingReport || !selectedTemplateId || (analysisScope === "single_source" ? !selectedSourceId : !selectedGroupId)}>
            {startingReport ? "Starting..." : "Run report"}
          </Button>
          {#if analysisScope === "single_source" && currentSource()}
            <Button
              variant="secondary"
              onclick={() => void syncSelectedSource(currentSource()!.id)}
              disabled={!!syncingIds[currentSource()!.id] || sourceSyncDisabledReason(currentSource()!) !== null}
              title={sourceSyncDisabledReason(currentSource()!) ?? undefined}
            >
              {syncingIds[currentSource()!.id] ? "Syncing..." : "Sync source"}
            </Button>
          {/if}
        </div>
      </div>

      {#if selectedRunIsActive || currentRun}
        <div class="live-strip">
          <span><strong>Phase:</strong> {phaseLabel(activePhase)}</span>
          {#if activeProgress}
            <span><strong>Progress:</strong> {activeProgress}</span>
          {/if}
          {#if currentRun}
            <span><strong>Provider:</strong> {currentRun.provider}/{currentRun.model}</span>
          {/if}
        </div>
      {/if}
    </div>

    {#if analysisScope === "single_source" && currentSource()}
      <div class="context-panel">
        <div class="context-panel-header">
          <div>
            <span class="eyebrow">Source context</span>
            <h3>Recent synced messages</h3>
          </div>
          <Badge variant="neutral">
            {currentSourceMetric()?.item_count ?? sourceItems.length} messages
          </Badge>
        </div>
        <SourceMessagesPanel
          {loadingItems}
          items={sourceItems}
          formatDate={formatTimestamp}
          embedded={true}
          previewLimit={120}
        />
      </div>
    {/if}

    <ReportViewer
      {currentRun}
      {loadingRunDetail}
      streamedOutput={focusedStreamedOutput}
      traceRefCount={traceData.refs.length}
      {selectedTraceRef}
      livePhase={activePhase}
      liveProgress={activeProgress}
      {canCancelCurrentRun}
      {formatTimestamp}
      {formatPeriod}
      {runTargetLabel}
      {statusTone}
      {reportLines}
      onFocusTraceRef={focusTraceRef}
      onCancelCurrentRun={() => {
        if (activeRunId !== null) {
          return cancelActiveRun(activeRunId);
        }
      }}
    />

    <ChatPanel
      {currentRun}
      {loadingChat}
      {chatMessages}
      {chatQuestion}
      {chatting}
      canCancelChat={chatting && activeChatRequestId !== null}
      {clearingChat}
      {selectedTraceRef}
      {reportLines}
      onFocusTraceRef={focusTraceRef}
      onAskQuestion={askRunQuestion}
      onCancelChat={cancelChat}
      onClearChat={clearChatMessages}
      onChangeChatQuestion={(value) => (chatQuestion = value)}
    />

    <div class="utility-strip">
      <TemplateEditor
        compact={true}
        {selectedTemplate}
        {templateName}
        {templateBody}
        {savingTemplate}
        {deletingTemplate}
        onSaveTemplateCopy={saveTemplateCopy}
        onSaveTemplateChanges={saveTemplateChanges}
        onDeleteTemplate={deleteTemplate}
      />

      <SourceGroupEditor
        compact={true}
        {groups}
        {selectedGroupId}
        {selectedGroup}
        {groupName}
        {groupMemberSourceIds}
        sources={Object.values(sourceMetrics)}
        {savingGroup}
        {deletingGroup}
        {formatTimestamp}
        {isGroupSourceSelected}
        onChangeSelectedGroupId={(value) => (selectedGroupId = value)}
        onChangeGroupName={(value) => (groupName = value)}
        onToggleSource={toggleGroupSource}
        onStartNewGroup={startNewGroup}
        onSaveGroupCopy={saveGroupCopy}
        onSaveGroupChanges={saveGroupChanges}
        onDeleteGroup={deleteGroup}
      />
    </div>
  </section>

  <aside class="inspector">
    <div class="inspector-header">
      <div>
        <span class="eyebrow">Inspector</span>
        <h3>Runs and evidence</h3>
      </div>
      <div class="inspector-tabs">
        <Button variant="secondary" size="sm" selected={inspectorMode === "active"} onclick={() => (inspectorMode = "active")}>
          Active
        </Button>
        <Button variant="secondary" size="sm" selected={inspectorMode === "history"} onclick={() => (inspectorMode = "history")}>
          History
        </Button>
        <Button variant="secondary" size="sm" selected={inspectorMode === "trace"} onclick={() => (inspectorMode = "trace")}>
          Trace
        </Button>
        <Button variant="secondary" size="sm" selected={inspectorMode === "chunks"} onclick={() => (inspectorMode = "chunks")}>
          Chunks
        </Button>
      </div>
    </div>

    <div class="inspector-body">
      {#if inspectorMode === "active"}
        <ActiveRunList
          {activeRuns}
          {loadingActiveRuns}
          {activeRunId}
          {formatTimestamp}
          {formatPeriod}
          {phaseLabel}
          {livePhase}
          {liveProgress}
          {runTargetLabel}
          {statusTone}
          onRefresh={loadActiveRuns}
          onOpenRun={openRun}
          onCancelRun={cancelActiveRun}
        />
      {:else if inspectorMode === "history"}
        <RunHistory
          {runs}
          {loadingRuns}
          {historyScope}
          historyTargetReady={historyScopeParams !== null}
          {runFilter}
          {activeRunId}
          {filteredRuns}
          {formatTimestamp}
          {formatPeriod}
          {runTargetLabel}
          {statusTone}
          onRefresh={loadRuns}
          onOpenRun={openRun}
          onChangeFilter={(next) => (runFilter = next)}
          onChangeHistoryScope={(next) => (historyScope = next)}
        />
      {:else if inspectorMode === "trace"}
        <TracePanel
          traceRefs={traceData.refs}
          {selectedTraceRef}
          {selectedTrace}
          {formatTimestamp}
          {traceRefOrigin}
          onSelectTraceRef={(ref) => (selectedTraceRef = ref)}
        />
      {:else}
        <ChunkSummaries summaries={focusedChunkSummaries} running={selectedRunIsActive} />
      {/if}
    </div>
  </aside>
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

  .rail,
  .center-pane,
  .inspector {
    min-width: 0;
  }

  .rail,
  .inspector {
    position: sticky;
    top: 0;
  }

  .rail {
    display: flex;
    flex-direction: column;
    gap: 0.8rem;
    padding: 0.85rem;
    background:
      linear-gradient(180deg, color-mix(in srgb, var(--panel) 96%, white 4%), var(--panel));
    border: 1px solid var(--border);
    border-radius: 16px;
    box-shadow: var(--shadow);
    max-height: calc(100vh - 6rem);
    overflow: auto;
  }

  .rail-header,
  .scope-hero,
  .controls-panel,
  .context-panel,
  .inspector {
    background: var(--panel);
    border: 1px solid var(--border);
    box-shadow: var(--shadow);
  }

  .rail-header,
  .scope-hero,
  .controls-panel,
  .context-panel,
  .inspector {
    border-radius: 16px;
  }

  .rail-header,
  .context-panel-header,
  .inspector-header,
  .scope-hero {
    display: flex;
    justify-content: space-between;
    gap: 0.75rem;
    align-items: flex-start;
  }

  .eyebrow {
    display: inline-block;
    font-size: 0.68rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--muted);
    margin-bottom: 0.2rem;
  }

  .rail-header h1,
  .scope-hero h2,
  .context-panel-header h3,
  .inspector-header h3 {
    margin: 0;
  }

  .rail-section {
    display: flex;
    flex-direction: column;
    gap: 0.55rem;
    padding-top: 0.1rem;
  }

  .rail-section + .rail-section {
    padding-top: 0.85rem;
    border-top: 1px solid color-mix(in srgb, var(--border) 76%, transparent);
  }

  .rail-section-title {
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 0.78rem;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  :global(.rail-search) {
    min-height: 2.5rem;
  }

  .rail-list {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .rail-empty {
    padding: 0.85rem 0.95rem;
    border: 1px dashed var(--border);
    border-radius: 12px;
    color: var(--muted);
    font-size: 0.86rem;
    background: color-mix(in srgb, var(--panel-strong) 72%, transparent);
  }

  .rail-row {
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
    width: 100%;
    padding: 0.6rem;
    border-radius: 14px;
    border: 1px solid transparent;
    background: var(--panel-strong);
    transition: background 0.2s, border-color 0.2s, transform 0.2s, box-shadow 0.2s;
  }

  .rail-group-row {
    flex-direction: row;
    align-items: center;
    text-align: left;
    cursor: pointer;
  }

  .rail-row.selected,
  .rail-group-row.selected {
    border-color: color-mix(in srgb, var(--primary) 45%, transparent);
    background: color-mix(in srgb, var(--primary) 8%, var(--panel-strong));
    box-shadow: 0 0 0 2px color-mix(in srgb, var(--primary) 10%, transparent);
  }

  .rail-row:hover,
  .rail-group-row:hover {
    border-color: color-mix(in srgb, var(--border-strong) 68%, transparent);
    background: color-mix(in srgb, var(--panel-hover) 78%, var(--panel-strong));
    transform: translateY(-1px);
  }

  .rail-row-main {
    display: flex;
    gap: 0.6rem;
    align-items: flex-start;
    width: 100%;
    background: transparent;
    border: 0;
    color: inherit;
    padding: 0;
    text-align: left;
    cursor: pointer;
  }

  .rail-avatar {
    flex: 0 0 2.35rem;
    width: 2.35rem;
    height: 2.35rem;
    border-radius: 0.9rem;
    overflow: hidden;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: color-mix(in srgb, var(--primary) 12%, var(--panel));
    color: var(--primary);
    font-size: 0.9rem;
    font-weight: 700;
  }

  .group-avatar {
    border-radius: 0.8rem;
  }

  .rail-avatar img {
    display: block;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .rail-copy {
    min-width: 0;
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.16rem;
  }

  .rail-copy-top,
  .rail-copy-meta,
  .rail-row-actions {
    display: flex;
    gap: 0.45rem;
    flex-wrap: wrap;
    align-items: center;
  }

  .rail-copy-top {
    justify-content: space-between;
  }

  .rail-copy-top strong {
    font-size: 0.92rem;
    line-height: 1.25;
  }

  .rail-copy-top span,
  .rail-copy-meta span {
    font-size: 0.75rem;
    color: var(--muted);
  }

  .center-pane {
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
    min-width: 0;
  }

  .scope-hero,
  .controls-panel,
  .context-panel,
  .inspector {
    padding: 1rem;
  }

  .scope-hero {
    background:
      linear-gradient(180deg, color-mix(in srgb, var(--panel) 98%, white 2%), var(--panel));
  }

  .scope-hero-copy p {
    margin: 0.35rem 0 0 0;
    color: var(--muted);
    line-height: 1.55;
    max-width: 64ch;
  }

  .scope-hero-meta {
    display: flex;
    gap: 0.45rem;
    flex-wrap: wrap;
    justify-content: flex-end;
  }

  .scope-facts {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 0.7rem;
  }

  .scope-fact {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    padding: 0.85rem 0.95rem;
    border-radius: 14px;
    border: 1px solid color-mix(in srgb, var(--border) 82%, transparent);
    background:
      linear-gradient(180deg, color-mix(in srgb, var(--panel-strong) 72%, transparent), transparent);
    box-shadow: var(--shadow-soft);
    min-width: 0;
  }

  .scope-fact-label {
    font-size: 0.68rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--muted);
  }

  .scope-fact strong {
    font-size: 0.88rem;
    line-height: 1.35;
  }

  .controls-panel {
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
    background:
      linear-gradient(180deg, color-mix(in srgb, var(--panel-strong) 54%, transparent), var(--panel));
  }

  .controls-grid {
    display: grid;
    grid-template-columns: repeat(4, minmax(0, 1fr));
    gap: 0.8rem;
  }

  .controls-bottom {
    display: flex;
    gap: 0.8rem;
    align-items: end;
    justify-content: space-between;
    flex-wrap: wrap;
  }

  .controls-actions {
    display: flex;
    gap: 0.55rem;
    align-items: center;
    flex-wrap: wrap;
  }

  .model-field {
    flex: 1 1 18rem;
    min-width: 16rem;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    font-size: 0.83rem;
    color: var(--muted);
  }

  .live-strip {
    display: flex;
    gap: 0.8rem;
    flex-wrap: wrap;
    align-items: center;
    padding-top: 0.2rem;
    border-top: 1px solid color-mix(in srgb, var(--border) 76%, transparent);
    color: var(--muted);
    font-size: 0.84rem;
  }

  .context-panel {
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
    background:
      linear-gradient(180deg, color-mix(in srgb, var(--panel-strong) 44%, transparent), var(--panel));
  }

  .utility-strip {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 0.8rem;
  }

  .inspector {
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
    max-height: calc(100vh - 6rem);
    overflow: auto;
    background:
      linear-gradient(180deg, color-mix(in srgb, var(--panel) 97%, white 3%), var(--panel));
  }

  .inspector-header {
    padding-bottom: 0.2rem;
    border-bottom: 1px solid color-mix(in srgb, var(--border) 76%, transparent);
  }

  .inspector-tabs {
    display: flex;
    gap: 0.35rem;
    flex-wrap: wrap;
    justify-content: flex-end;
  }

  .inspector-body {
    min-width: 0;
  }

  @media (max-width: 1500px) {
    .analysis-workspace {
      grid-template-columns: minmax(250px, 300px) minmax(0, 1fr);
    }

    .inspector {
      grid-column: 1 / -1;
      position: static;
      max-height: none;
    }
  }

  @media (max-width: 1180px) {
    .analysis-workspace {
      grid-template-columns: 1fr;
    }

    .rail {
      position: static;
      max-height: none;
    }

    .scope-facts,
    .controls-grid,
    .utility-strip {
      grid-template-columns: 1fr;
    }
  }

  @media (max-width: 720px) {
    .scope-hero,
    .context-panel-header,
    .inspector-header,
    .controls-bottom {
      flex-direction: column;
      align-items: stretch;
    }

    .controls-grid {
      grid-template-columns: 1fr;
    }

    .model-field {
      min-width: 0;
    }

    .scope-hero-meta,
    .inspector-tabs {
      justify-content: flex-start;
    }
  }
</style>
