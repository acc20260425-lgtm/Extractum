<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import ReportViewer from "$lib/components/analysis/report-viewer.svelte";
  import RunHistory from "$lib/components/analysis/run-history.svelte";
  import TracePanel from "$lib/components/analysis/trace-panel.svelte";
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

  let sources = $state<AnalysisSourceOption[]>([]);
  let templates = $state<AnalysisPromptTemplate[]>([]);
  let runs = $state<AnalysisRunSummary[]>([]);
  let groups = $state<AnalysisSourceGroup[]>([]);
  let loadingSources = $state(false);
  let loadingTemplates = $state(false);
  let loadingRuns = $state(false);
  let loadingGroups = $state(false);
  let loadingRunDetail = $state(false);
  let loadingChat = $state(false);

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
  let running = $state(false);
  let activeRunId = $state<number | null>(null);
  let activePhase = $state("");
  let activeProgress = $state("");
  let streamedOutput = $state("");
  let currentRun = $state<AnalysisRunDetail | null>(null);
  let traceData = $state<AnalysisTraceData>({ refs: [] });
  let selectedTraceRef = $state<string | null>(null);
  let savedTraceRefs = $state<string[]>([]);
  let resolvedTraceRefs = $state<string[]>([]);
  let runFilter = $state<"all" | "completed" | "failed" | "running">("all");
  let chatQuestion = $state("");
  let chatMessages = $state<AnalysisChatTurn[]>([]);
  let chatting = $state(false);
  let activeChatRequestId = $state<string | null>(null);
  let activeChatRunId = $state<number | null>(null);
  let chatThreadElement: HTMLDivElement | null = null;
  let clearingChat = $state(false);
  let statusTimer: ReturnType<typeof setTimeout> | null = null;

  function isErrorStatus(value: string) {
    return value.startsWith("Error") || value.startsWith("Analysis failed");
  }

  function filteredRuns() {
    if (runFilter === "all") return runs;
    if (runFilter === "running") {
      return runs.filter((run) => run.status === "running" || run.status === "queued");
    }
    return runs.filter((run) => run.status === runFilter);
  }

  function selectedTrace() {
    if (!selectedTraceRef) return null;
    return traceData.refs.find((ref) => ref.ref === selectedTraceRef) ?? null;
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

  function selectedTemplate() {
    const templateId = selectedTemplateId ? Number(selectedTemplateId) : null;
    if (templateId === null) return null;
    return templates.find((template) => template.id === templateId) ?? null;
  }

  function selectedGroup() {
    const groupId = selectedGroupId ? Number(selectedGroupId) : null;
    if (groupId === null) return null;
    return groups.find((group) => group.id === groupId) ?? null;
  }

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

  async function focusTraceRef(ref: string) {
    if (!currentRun) return;

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
      resolvedTraceRefs = [...resolvedTraceRefs, ...resolved.map((entry) => entry.ref).filter((entry) => !resolvedTraceRefs.includes(entry))];
      selectedTraceRef = ref;
    } catch (error) {
      status = `Error resolving trace reference: ${error}`;
    }
  }

  async function loadTrace(runId: number) {
    try {
      traceData = await invoke<AnalysisTraceData>("get_analysis_run_trace", { runId });
      savedTraceRefs = traceData.refs.map((ref) => ref.ref);
      resolvedTraceRefs = [];
      selectedTraceRef = traceData.refs[0]?.ref ?? null;
    } catch (error) {
      traceData = { refs: [] };
      savedTraceRefs = [];
      resolvedTraceRefs = [];
      selectedTraceRef = null;
      status = `Error loading analysis trace: ${error}`;
    }
  }

  async function loadSources() {
    loadingSources = true;
    try {
      const result = await invoke<AnalysisSourceOption[]>("list_analysis_sources");
      sources = result.filter((source) => source.item_count > 0);
      if (!selectedSourceId && sources.length > 0) {
        selectedSourceId = String(sources[0].id);
      }
    } catch (error) {
      status = `Error loading analysis sources: ${error}`;
    } finally {
      loadingSources = false;
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
      const selected = selectedTemplate();
      if (selected && editorBoundTemplateId !== selected.id) {
        bindEditorToTemplate(selected);
      }
    } catch (error) {
      status = `Error loading report templates: ${error}`;
    } finally {
      loadingTemplates = false;
    }
  }

  async function loadGroups() {
    loadingGroups = true;
    try {
      groups = await invoke<AnalysisSourceGroup[]>("list_analysis_source_groups");
      const selected = selectedGroup();
      if (!selectedGroupId && groups.length > 0) {
        selectedGroupId = String(groups[0].id);
      }
      if (!selected && groups.length > 0 && selectedGroupId) {
        selectedGroupId = String(groups[0].id);
      }
      const bound = selectedGroup();
      if (bound && editorBoundGroupId !== bound.id) {
        bindEditorToGroup(bound);
      }
    } catch (error) {
      status = `Error loading source groups: ${error}`;
    } finally {
      loadingGroups = false;
    }
  }

  async function loadRuns() {
    loadingRuns = true;
    try {
      runs = await invoke<AnalysisRunSummary[]>("list_analysis_runs", {
        sourceId: analysisScope === "single_source" && selectedSourceId ? Number(selectedSourceId) : null,
        sourceGroupId: analysisScope === "source_group" && selectedGroupId ? Number(selectedGroupId) : null,
        limit: 20,
      });
    } catch (error) {
      status = `Error loading analysis runs: ${error}`;
    } finally {
      loadingRuns = false;
    }
  }

  async function openRun(runId: number) {
    loadingRunDetail = true;
    try {
      if (activeRunId !== runId) {
        chatMessages = [];
        chatQuestion = "";
        chatting = false;
        activeChatRequestId = null;
        activeChatRunId = null;
      }
      const run = await invoke<AnalysisRunDetail | null>("get_analysis_run", { runId });
      if (!run) {
        status = `Analysis run ${runId} was not found.`;
        return;
      }
      currentRun = run;
      streamedOutput = run.result_markdown ?? "";
      activeRunId = run.id;
      activePhase = run.status;
      activeProgress = "";
      await loadChatMessages(run.id);
      if (run.has_trace_data) {
        await loadTrace(run.id);
      } else {
        traceData = { refs: [] };
        savedTraceRefs = [];
        resolvedTraceRefs = [];
        selectedTraceRef = null;
      }
    } catch (error) {
      status = `Error loading analysis run: ${error}`;
    } finally {
      loadingRunDetail = false;
    }
  }

  async function loadChatMessages(runId: number) {
    loadingChat = true;
    try {
      const messages = await invoke<AnalysisChatMessage[]>("list_analysis_chat_messages", { runId });
      chatMessages = messages.map((message) => ({
        role: message.role,
        content: message.content,
      }));
    } catch (error) {
      chatMessages = [];
      status = `Error loading analysis chat: ${error}`;
    } finally {
      loadingChat = false;
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
    if (periodFrom > periodTo) {
      status = "The start date must be earlier than or equal to the end date.";
      return;
    }
    if (!outputLanguage.trim()) {
      status = "Output language cannot be empty.";
      return;
    }

    status = "";
    running = true;
    streamedOutput = "";
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
    activePhase = "queued";
    activeProgress = "";

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

      activeRunId = runId;
      await loadRuns();
    } catch (error) {
      running = false;
      status = `Error starting analysis report: ${error}`;
    }
  }

  async function askRunQuestion() {
    if (!currentRun) {
      status = "Open a completed report first.";
      return;
    }
    if (currentRun.status !== "completed") {
      status = "Open a completed report first.";
      return;
    }
    if (!chatQuestion.trim()) {
      status = "Question cannot be empty.";
      return;
    }

    const question = chatQuestion.trim();
    chatMessages = [...chatMessages, { role: "user", content: question }, { role: "assistant", content: "" }];
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
      chatMessages = chatMessages.slice(0, -2);
      chatting = false;
      activeChatRunId = null;
      activeChatRequestId = null;
      status = `Error starting chat answer: ${error}`;
    }
  }

  async function clearChatMessages() {
    if (!currentRun) {
      status = "Open a run first.";
      return;
    }
    if (!window.confirm("Clear saved chat history for this run?")) {
      return;
    }

    clearingChat = true;
    try {
      await invoke("clear_analysis_chat_messages", { runId: currentRun.id });
      chatMessages = [];
      status = "Saved chat history cleared.";
    } catch (error) {
      status = `Error clearing analysis chat: ${error}`;
    } finally {
      clearingChat = false;
    }
  }

  async function saveTemplateChanges() {
    const selected = selectedTemplate();
    if (!selected) {
      status = "Select a template first.";
      return;
    }
    if (selected.is_builtin) {
      status = "Built-in templates cannot be edited directly. Save a copy instead.";
      return;
    }
    if (!templateName.trim() || !templateBody.trim()) {
      status = "Template name and body cannot be empty.";
      return;
    }

    savingTemplate = true;
    try {
      const updated = await invoke<AnalysisPromptTemplate>("update_analysis_prompt_template", {
        templateId: selected.id,
        name: templateName.trim(),
        body: templateBody.trim(),
      });
      status = `Template "${updated.name}" saved.`;
      await loadTemplates();
      selectedTemplateId = String(updated.id);
      bindEditorToTemplate(updated);
    } catch (error) {
      status = `Error saving template: ${error}`;
    } finally {
      savingTemplate = false;
    }
  }

  async function saveTemplateCopy() {
    if (!templateName.trim() || !templateBody.trim()) {
      status = "Template name and body cannot be empty.";
      return;
    }

    savingTemplate = true;
    try {
      const created = await invoke<AnalysisPromptTemplate>("create_analysis_prompt_template", {
        name: templateName.trim(),
        templateKind: "report",
        body: templateBody.trim(),
      });
      status = `Template "${created.name}" created.`;
      await loadTemplates();
      selectedTemplateId = String(created.id);
      bindEditorToTemplate(created);
    } catch (error) {
      status = `Error creating template: ${error}`;
    } finally {
      savingTemplate = false;
    }
  }

  async function deleteTemplate() {
    const selected = selectedTemplate();
    if (!selected) {
      status = "Select a template first.";
      return;
    }
    if (selected.is_builtin) {
      status = "Built-in templates cannot be deleted.";
      return;
    }
    if (!window.confirm(`Delete template "${selected.name}"?`)) {
      return;
    }

    deletingTemplate = true;
    try {
      await invoke("delete_analysis_prompt_template", { templateId: selected.id });
      status = `Template "${selected.name}" deleted.`;
      await loadTemplates();
      const fallback = templates[0] ?? null;
      selectedTemplateId = fallback ? String(fallback.id) : "";
      bindEditorToTemplate(fallback);
    } catch (error) {
      status = `Error deleting template: ${error}`;
    } finally {
      deletingTemplate = false;
    }
  }

  async function saveGroupChanges() {
    const selected = selectedGroup();
    if (!selected) {
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
        groupId: selected.id,
        name: groupName.trim(),
        sourceIds: groupMemberSourceIds,
      });
      status = `Source group "${updated.name}" saved.`;
      await loadGroups();
      selectedGroupId = String(updated.id);
      bindEditorToGroup(updated);
    } catch (error) {
      status = `Error saving source group: ${error}`;
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
      status = `Error creating source group: ${error}`;
    } finally {
      savingGroup = false;
    }
  }

  async function deleteGroup() {
    const selected = selectedGroup();
    if (!selected) {
      status = "Select a source group first.";
      return;
    }
    if (!window.confirm(`Delete source group "${selected.name}"?`)) {
      return;
    }

    deletingGroup = true;
    try {
      await invoke("delete_analysis_source_group", { groupId: selected.id });
      status = `Source group "${selected.name}" deleted.`;
      await loadGroups();
      const fallback = groups[0] ?? null;
      selectedGroupId = fallback ? String(fallback.id) : "";
      bindEditorToGroup(fallback);
    } catch (error) {
      status = `Error deleting source group: ${error}`;
    } finally {
      deletingGroup = false;
    }
  }

  function startNewGroup() {
    selectedGroupId = "";
    bindEditorToGroup(null);
  }

  $effect(() => {
    if (
      (analysisScope === "single_source" && selectedSourceId) ||
      (analysisScope === "source_group" && selectedGroupId)
    ) {
      void loadRuns();
    }
  });

  $effect(() => {
    const selected = selectedTemplate();
    if (selected && editorBoundTemplateId !== selected.id) {
      bindEditorToTemplate(selected);
    }
  });

  $effect(() => {
    const selected = selectedGroup();
    if (selected && editorBoundGroupId !== selected.id) {
      bindEditorToGroup(selected);
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

  $effect(() => {
    const scrollKey = chatMessages.map((message) => `${message.role}:${message.content.length}`).join("|");
    scrollKey;
    chatting;
    if (typeof window === "undefined" || !chatThreadElement) return;
    requestAnimationFrame(() => {
      chatThreadElement?.scrollTo({
        top: chatThreadElement.scrollHeight,
        behavior: "smooth",
      });
    });
  });

  onMount(() => {
    let disposed = false;
    let detachAnalysisListener: (() => void) | null = null;
    let detachChatListener: (() => void) | null = null;

    void loadSources();
    void loadTemplates();
    void loadGroups();
    void loadRuns();

    void listen<AnalysisRunEvent>("analysis://run", ({ payload }: EventEnvelope<AnalysisRunEvent>) => {
      if (disposed || payload.run_id !== activeRunId) {
        return;
      }

      activePhase = payload.phase;
      activeProgress =
        payload.progress_current !== null && payload.progress_total !== null
          ? `${payload.progress_current}/${payload.progress_total}`
          : "";

      if (payload.kind === "started" || payload.kind === "progress") {
        if (payload.message) {
          status = payload.message;
        }
        return;
      }

      if (payload.kind === "delta") {
        streamedOutput += payload.delta ?? "";
        return;
      }

      if (payload.kind === "completed") {
        running = false;
        status = payload.message ?? "Report completed.";
        void loadRuns();
        if (activeRunId !== null) {
          void openRun(activeRunId);
        }
        return;
      }

      if (payload.kind === "failed") {
        running = false;
        status = payload.error ? `Analysis failed: ${payload.error}` : "Analysis failed.";
        void loadRuns();
        if (activeRunId !== null) {
          void openRun(activeRunId);
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

      if (payload.kind === "started") {
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
        if (payload.message) {
          status = payload.message;
        }
        return;
      }

      if (payload.kind === "failed") {
        chatting = false;
        activeChatRequestId = null;
        if (chatMessages.length >= 2 && chatMessages[chatMessages.length - 1]?.role === "assistant") {
          chatMessages = chatMessages.slice(0, -2);
        }
        status = payload.error ? `Analysis chat failed: ${payload.error}` : "Analysis chat failed.";
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

<h1>Analysis</h1>

{#if status}
  <p class="status" class:error={isErrorStatus(status)}>
    {status}
  </p>
{/if}

<div class="workspace">
  <section class="card controls">
    <h3>Run Report</h3>

    <div class="scope-toggle">
      <button
        class:activeScope={analysisScope === "single_source"}
        class="secondary"
        type="button"
        onclick={() => (analysisScope = "single_source")}
      >
        Single source
      </button>
      <button
        class:activeScope={analysisScope === "source_group"}
        class="secondary"
        type="button"
        onclick={() => (analysisScope = "source_group")}
      >
        Source group
      </button>
    </div>

    {#if analysisScope === "single_source"}
      <label>Source
        <select bind:value={selectedSourceId} disabled={loadingSources}>
          {#if loadingSources}
            <option value="">Loading synced sources...</option>
          {:else if sources.length === 0}
            <option value="">No synced sources available</option>
          {/if}
          {#each sources as source}
            <option value={String(source.id)}>
              {(source.title ?? `Source ${source.id}`)} - {source.item_count} messages
            </option>
          {/each}
        </select>
      </label>
    {:else}
      <label>Source group
        <select bind:value={selectedGroupId} disabled={loadingGroups}>
          {#if loadingGroups}
            <option value="">Loading source groups...</option>
          {:else if groups.length === 0}
            <option value="">No saved groups available</option>
          {/if}
          {#each groups as group}
            <option value={String(group.id)}>
              {group.name} - {group.members.length} sources
            </option>
          {/each}
        </select>
      </label>

      {#if selectedGroup()}
        <p class="sub">
          {selectedGroup()?.members.length} sources selected for this group report.
        </p>
      {/if}
    {/if}

    <div class="grid">
      <label>From
        <input type="date" bind:value={periodFrom} />
      </label>

      <label>To
        <input type="date" bind:value={periodTo} />
      </label>
    </div>

    <label>Output language
      <input type="text" bind:value={outputLanguage} placeholder="Russian" />
    </label>

    <label>Prompt template
      <select bind:value={selectedTemplateId} disabled={loadingTemplates}>
        {#if loadingTemplates}
          <option value="">Loading report templates...</option>
        {:else if templates.length === 0}
          <option value="">No report templates available</option>
        {/if}
        {#each templates as template}
          <option value={String(template.id)}>
            {template.name}{template.is_builtin ? " - builtin" : ""}
          </option>
        {/each}
      </select>
    </label>

    <label>Model override
      <input type="text" bind:value={modelOverride} placeholder="Use active profile default model" />
    </label>

    <button
      onclick={runReport}
      disabled={running || !selectedTemplateId || (analysisScope === "single_source" ? !selectedSourceId : !selectedGroupId)}
    >
      {running ? "Running..." : "Run report"}
    </button>

    <div class="meta-panel">
      <div><strong>Phase:</strong> {phaseLabel(activePhase)}</div>
      {#if activeProgress}
        <div><strong>Progress:</strong> {activeProgress}</div>
      {/if}
    </div>
  </section>

  <section class="card report">
    <div class="report-layout">
      <ReportViewer
        {currentRun}
        {loadingRunDetail}
        {streamedOutput}
        traceRefCount={traceData.refs.length}
        {selectedTraceRef}
        {formatTimestamp}
        {formatPeriod}
        {runTargetLabel}
        {statusTone}
        {reportLines}
        onFocusTraceRef={focusTraceRef}
      />

      <TracePanel
        traceRefs={traceData.refs}
        {selectedTraceRef}
        selectedTrace={selectedTrace()}
        {formatTimestamp}
        {traceRefOrigin}
        onSelectTraceRef={(ref) => (selectedTraceRef = ref)}
      />
    </div>
  </section>
</div>

<section class="card chat">
  <div class="panel-header">
    <div>
      <h3>Report Chat</h3>
      <p class="sub">Ask follow-up questions grounded in the saved report and matching synced messages from the same analysis scope.</p>
    </div>
    {#if currentRun && currentRun.status === "completed"}
      <button class="secondary" onclick={clearChatMessages} disabled={chatting || clearingChat}>
        {clearingChat ? "Clearing..." : "Clear chat"}
      </button>
    {/if}
  </div>

  {#if !currentRun}
    <p class="empty">Open a saved run to start a grounded chat.</p>
  {:else if currentRun.status !== "completed"}
    <p class="empty">Chat is available only for completed runs.</p>
  {:else}
    <div class="chat-thread" bind:this={chatThreadElement}>
      {#if loadingChat}
        <p class="empty">Loading saved chat history...</p>
      {:else if chatMessages.length === 0}
        <p class="empty">No saved chat turns yet. Ask a follow-up question about this report.</p>
      {:else}
        {#each chatMessages as message, index (`${message.role}-${index}`)}
          <div class={`chat-bubble chat-${message.role}`}>
            <div class="chat-role">{message.role === "user" ? "You" : "Assistant"}</div>
            <div class="chat-content">
              {#if message.role === "assistant" && message.content}
                {#each reportLines(message.content) as line (line.key)}
                  <div class="report-line">
                    {#each line.segments as segment (segment.key)}
                      {#if segment.type === "ref"}
                        <button
                          class="ref-chip"
                          class:active={segment.value === selectedTraceRef}
                          type="button"
                          onclick={() => void focusTraceRef(segment.value)}
                        >
                          [{segment.value}]
                        </button>
                      {:else}
                        <span>{segment.value}</span>
                      {/if}
                    {/each}
                  </div>
                {/each}
              {:else}
                {message.content || (chatting && message.role === "assistant" ? "..." : "")}
              {/if}
            </div>
          </div>
        {/each}
      {/if}
    </div>

    <div class="chat-compose">
      <label>Question
        <textarea
          bind:value={chatQuestion}
          rows="4"
          placeholder="Ask a grounded follow-up question about this report."
        ></textarea>
      </label>
      <button onclick={askRunQuestion} disabled={chatting || loadingChat || !chatQuestion.trim() || !currentRun || currentRun.status !== "completed"}>
        {chatting ? "Answering..." : "Ask"}
      </button>
    </div>
  {/if}
</section>

<section class="card templates">
  <div class="panel-header">
    <div>
      <h3>Prompt Template</h3>
      {#if selectedTemplate()}
        <p class="sub">
          {selectedTemplate()?.name} - v{selectedTemplate()?.version}
          {selectedTemplate()?.is_builtin ? " - builtin (edit fields below, then save as copy)" : " - custom"}
        </p>
      {/if}
    </div>
    <div class="template-actions">
      <button class="secondary" onclick={saveTemplateCopy} disabled={savingTemplate || deletingTemplate}>
        {savingTemplate ? "Saving..." : "Save as copy"}
      </button>
      <button
        onclick={saveTemplateChanges}
        disabled={savingTemplate || deletingTemplate || !selectedTemplate() || selectedTemplate()?.is_builtin === true}
      >
        {savingTemplate ? "Saving..." : "Save changes"}
      </button>
      <button
        class="danger-soft"
        onclick={deleteTemplate}
        disabled={savingTemplate || deletingTemplate || !selectedTemplate() || selectedTemplate()?.is_builtin === true}
      >
        {deletingTemplate ? "Deleting..." : "Delete"}
      </button>
    </div>
  </div>

  <div class="template-grid">
    <label>Template name
      <input type="text" bind:value={templateName} placeholder="Custom report" />
    </label>

    <label>Template body
      <textarea
        bind:value={templateBody}
        rows="10"
        placeholder="Describe how the report should be structured and what it should emphasize."
      ></textarea>
    </label>
  </div>
</section>

<section class="card groups">
  <div class="panel-header">
    <div>
      <h3>Source Groups</h3>
      <p class="sub">Save reusable named sets of synced sources for future cross-source reports.</p>
    </div>
    <div class="template-actions">
      <button class="secondary" onclick={startNewGroup} disabled={savingGroup || deletingGroup}>
        New group
      </button>
      <button class="secondary" onclick={saveGroupCopy} disabled={savingGroup || deletingGroup}>
        {savingGroup ? "Saving..." : "Save as new"}
      </button>
      <button class="secondary" onclick={saveGroupChanges} disabled={savingGroup || deletingGroup || !selectedGroup()}>
        {savingGroup ? "Saving..." : "Save changes"}
      </button>
      <button class="danger-soft" onclick={deleteGroup} disabled={savingGroup || deletingGroup || !selectedGroup()}>
        {deletingGroup ? "Deleting..." : "Delete"}
      </button>
    </div>
  </div>

  <div class="group-grid">
    <div class="group-form">
      <label>Saved groups
        <select bind:value={selectedGroupId}>
          <option value="">Create a new group</option>
          {#each groups as group}
            <option value={String(group.id)}>
              {group.name} - {group.members.length} sources
            </option>
          {/each}
        </select>
      </label>

      <label>Group name
        <input type="text" bind:value={groupName} placeholder="Core channels" />
      </label>

      {#if selectedGroup()}
        <p class="sub">
          Updated {formatTimestamp(selectedGroup()?.updated_at ?? null)}
        </p>
      {/if}
    </div>

    <div class="group-members">
      <div class="members-header">
        <h4>Group Members</h4>
        <span class="trace-count">{groupMemberSourceIds.length} selected</span>
      </div>

      {#if sources.length === 0}
        <p class="empty">No synced sources available for grouping yet.</p>
      {:else}
        <div class="member-list">
          {#each sources as source}
            <label class="member-row">
              <input
                type="checkbox"
                checked={isGroupSourceSelected(source.id)}
                onchange={() => toggleGroupSource(source.id)}
              />
              <div class="member-copy">
                <strong>{source.title ?? `Source ${source.id}`}</strong>
                <span>{source.item_count} messages</span>
              </div>
            </label>
          {/each}
        </div>
      {/if}
    </div>
  </div>
</section>

<RunHistory
  {runs}
  {loadingRuns}
  {runFilter}
  {activeRunId}
  filteredRuns={filteredRuns()}
  {formatTimestamp}
  {formatPeriod}
  {runTargetLabel}
  {statusTone}
  onRefresh={loadRuns}
  onOpenRun={openRun}
  onChangeFilter={(next) => (runFilter = next)}
/>

<style>
  .workspace {
    display: grid;
    grid-template-columns: minmax(320px, 420px) minmax(0, 1fr);
    gap: 1.5rem;
    align-items: start;
  }

  .card {
    background: var(--panel);
    border: 1px solid var(--border);
    box-shadow: var(--shadow);
    border-radius: 12px;
    padding: 1.5rem;
  }

  .controls,
  .report {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 0.8rem;
  }

  .scope-toggle {
    display: flex;
    gap: 0.6rem;
    flex-wrap: wrap;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    font-size: 0.9rem;
    color: var(--muted);
  }

  textarea {
    width: 100%;
    resize: vertical;
    min-height: 10rem;
    background: var(--panel-strong);
    border: 1px solid var(--border);
    color: var(--text);
    padding: 0.8rem;
    border-radius: 8px;
    font: inherit;
  }

  textarea:focus {
    border-color: var(--primary);
    outline: none;
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--primary) 18%, transparent);
  }

  .status {
    padding: 0.6rem 1rem;
    border-radius: 6px;
    background: var(--status-bg);
    font-size: 0.9rem;
    margin-bottom: 1rem;
  }

  .status.error {
    background: var(--status-error-bg);
    color: var(--status-error-text);
  }

  .meta-panel {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    padding: 0.8rem 1rem;
    border-radius: 10px;
    background: var(--panel-strong);
    border: 1px solid var(--border);
    color: var(--muted);
    font-size: 0.9rem;
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    gap: 1rem;
    align-items: center;
    flex-wrap: wrap;
  }

  .sub,
  .empty {
    margin: 0;
    color: var(--muted);
    font-size: 0.9rem;
  }

  .report-layout {
    display: grid;
    grid-template-columns: minmax(0, 1.7fr) minmax(280px, 0.9fr);
    gap: 1rem;
    align-items: start;
  }

  .trace-count {
    color: var(--muted);
    font-size: 0.85rem;
  }

  .report-line {
    white-space: pre-wrap;
    word-break: break-word;
  }

  .ref-chip {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.08rem 0.45rem;
    margin: 0 0.08rem;
    border-radius: 999px;
    background: color-mix(in srgb, var(--primary) 14%, var(--panel));
    color: var(--primary);
    border: 1px solid color-mix(in srgb, var(--primary) 24%, transparent);
    font-size: 0.82rem;
    font-weight: 600;
  }

  .ref-chip:hover,
  .ref-chip.active {
    background: color-mix(in srgb, var(--primary) 22%, var(--panel));
  }

  .activeScope {
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--primary) 14%, transparent);
    border-color: var(--primary);
  }

  .templates {
    margin-top: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .chat {
    margin-top: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .chat-thread {
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
    padding: 1rem;
    background: var(--panel-strong);
    border: 1px solid var(--border);
    border-radius: 10px;
    min-height: 10rem;
  }

  .chat-bubble {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    max-width: min(52rem, 100%);
    padding: 0.9rem 1rem;
    border-radius: 12px;
    border: 1px solid var(--border);
    white-space: pre-wrap;
    word-break: break-word;
  }

  .chat-user {
    align-self: flex-end;
    background: color-mix(in srgb, var(--primary) 10%, var(--panel));
    border-color: color-mix(in srgb, var(--primary) 24%, transparent);
  }

  .chat-assistant {
    align-self: flex-start;
    background: var(--panel);
  }

  .chat-role {
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--muted);
  }

  .chat-content {
    color: var(--text);
    line-height: 1.6;
  }

  .chat-compose {
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
  }

  .groups {
    margin-top: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .template-actions {
    display: flex;
    gap: 0.6rem;
    flex-wrap: wrap;
  }

  .template-grid {
    display: grid;
    grid-template-columns: minmax(260px, 360px) minmax(0, 1fr);
    gap: 1rem;
    align-items: start;
  }

  .group-grid {
    display: grid;
    grid-template-columns: minmax(260px, 360px) minmax(0, 1fr);
    gap: 1rem;
    align-items: start;
  }

  .group-form,
  .group-members {
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
  }

  .members-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
    flex-wrap: wrap;
  }

  .members-header h4 {
    margin: 0;
  }

  .member-list {
    display: flex;
    flex-direction: column;
    gap: 0.65rem;
    max-height: 24rem;
    overflow: auto;
    padding-right: 0.25rem;
  }

  .member-row {
    flex-direction: row;
    align-items: flex-start;
    gap: 0.75rem;
    padding: 0.85rem 0.95rem;
    background: var(--panel-strong);
    border: 1px solid var(--border);
    border-radius: 10px;
    cursor: pointer;
  }

  .member-row:hover {
    background: var(--panel-hover);
  }

  .member-row input {
    margin-top: 0.2rem;
  }

  .member-copy {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    min-width: 0;
  }

  .member-copy span {
    color: var(--muted);
    font-size: 0.82rem;
  }

  :global(button.danger-soft) {
    background: color-mix(in srgb, var(--danger) 14%, var(--panel));
    color: var(--danger);
    border: 1px solid color-mix(in srgb, var(--danger) 28%, transparent);
  }

  :global(button.danger-soft:hover) {
    background: color-mix(in srgb, var(--danger) 22%, var(--panel));
  }

  @media (max-width: 1080px) {
    .workspace {
      grid-template-columns: 1fr;
    }

    .report-layout {
      grid-template-columns: 1fr;
    }

    .template-grid {
      grid-template-columns: 1fr;
    }

    .group-grid {
      grid-template-columns: 1fr;
    }

  }

  @media (max-width: 720px) {
    .grid {
      grid-template-columns: 1fr;
    }
  }
</style>
