<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";

  interface AnalysisSourceOption {
    id: number;
    account_id: number | null;
    title: string | null;
    item_count: number;
    last_synced_at: number | null;
  }

  interface AnalysisPromptTemplate {
    id: number;
    name: string;
    template_kind: string;
    body: string;
    version: number;
    is_builtin: boolean;
    created_at: number;
    updated_at: number;
  }

  interface AnalysisSourceGroupMember {
    source_id: number;
    source_title: string | null;
    item_count: number;
  }

  interface AnalysisSourceGroup {
    id: number;
    name: string;
    members: AnalysisSourceGroupMember[];
    created_at: number;
    updated_at: number;
  }

  interface AnalysisRunSummary {
    id: number;
    run_type: string;
    scope_type: string;
    source_id: number | null;
    source_title: string | null;
    source_group_id: number | null;
    source_group_name: string | null;
    period_from: number;
    period_to: number;
    output_language: string;
    prompt_template_id: number | null;
    prompt_template_name: string | null;
    prompt_template_version: number;
    provider_profile: string;
    provider: string;
    model: string;
    status: string;
    error: string | null;
    has_trace_data: boolean;
    created_at: number;
    completed_at: number | null;
  }

  interface AnalysisRunDetail extends AnalysisRunSummary {
    result_markdown: string | null;
    error: string | null;
  }

  interface AnalysisTraceRef {
    ref: string;
    item_id: number;
    source_id: number;
    external_id: string;
    published_at: number;
    excerpt: string;
  }

  interface AnalysisTraceData {
    refs: AnalysisTraceRef[];
  }

  interface AnalysisRunEvent {
    run_id: number;
    kind: "started" | "progress" | "delta" | "completed" | "failed";
    phase: string;
    message: string | null;
    progress_current: number | null;
    progress_total: number | null;
    delta: string | null;
    error: string | null;
  }

  interface AnalysisChatTurn {
    role: "user" | "assistant";
    content: string;
  }

  interface AnalysisChatEvent {
    request_id: string;
    run_id: number;
    kind: "started" | "delta" | "completed" | "failed";
    delta: string | null;
    message: string | null;
    error: string | null;
  }

  interface EventEnvelope<T> {
    payload: T;
  }

  let sources = $state<AnalysisSourceOption[]>([]);
  let templates = $state<AnalysisPromptTemplate[]>([]);
  let runs = $state<AnalysisRunSummary[]>([]);
  let groups = $state<AnalysisSourceGroup[]>([]);

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
  let runFilter = $state<"all" | "completed" | "failed" | "running">("all");
  let chatQuestion = $state("");
  let chatMessages = $state<AnalysisChatTurn[]>([]);
  let chatting = $state(false);
  let activeChatRequestId = $state<string | null>(null);
  let activeChatRunId = $state<number | null>(null);

  type ReportSegment =
    | { type: "text"; value: string; key: string }
    | { type: "ref"; value: string; key: string };

  function defaultDateOffset(offsetDays: number) {
    const date = new Date();
    date.setDate(date.getDate() + offsetDays);
    const year = date.getFullYear();
    const month = `${date.getMonth() + 1}`.padStart(2, "0");
    const day = `${date.getDate()}`.padStart(2, "0");
    return `${year}-${month}-${day}`;
  }

  function startOfDayUnix(dateString: string) {
    return Math.floor(new Date(`${dateString}T00:00:00`).getTime() / 1000);
  }

  function endOfDayUnix(dateString: string) {
    return Math.floor(new Date(`${dateString}T23:59:59`).getTime() / 1000);
  }

  function formatTimestamp(timestamp: number | null) {
    if (!timestamp) return "n/a";
    return new Date(timestamp * 1000).toLocaleString();
  }

  function formatDay(timestamp: number | null) {
    if (!timestamp) return "n/a";
    return new Date(timestamp * 1000).toLocaleDateString();
  }

  function formatPeriod(periodFromUnix: number, periodToUnix: number) {
    return `${formatDay(periodFromUnix)} - ${formatDay(periodToUnix)}`;
  }

  function runTargetLabel(run: Pick<AnalysisRunSummary, "scope_type" | "source_id" | "source_title" | "source_group_id" | "source_group_name">) {
    if (run.scope_type === "source_group") {
      return run.source_group_name ?? `Group ${run.source_group_id ?? "?"}`;
    }
    return run.source_title ?? `Source ${run.source_id ?? "?"}`;
  }

  function phaseLabel(phase: string) {
    switch (phase) {
      case "load_items":
        return "Loading items";
      case "chunking":
        return "Chunking corpus";
      case "map":
        return "Analyzing chunks";
      case "reduce":
        return "Writing report";
      case "persist":
        return "Saving run";
      default:
        return phase || "Running";
    }
  }

  function statusTone(status: string) {
    switch (status) {
      case "completed":
        return "success";
      case "failed":
        return "danger";
      case "running":
      case "queued":
        return "info";
      default:
        return "neutral";
    }
  }

  function filteredRuns() {
    if (runFilter === "all") return runs;
    if (runFilter === "running") {
      return runs.filter((run) => run.status === "running" || run.status === "queued");
    }
    return runs.filter((run) => run.status === runFilter);
  }

  function normalizeRef(candidate: string) {
    const trimmed = candidate.trim().replace(/^\[/, "").replace(/\]$/, "");
    return /^s\d+-m\d+$/.test(trimmed) ? trimmed : null;
  }

  function selectedTrace() {
    if (!selectedTraceRef) return null;
    return traceData.refs.find((ref) => ref.ref === selectedTraceRef) ?? null;
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

  function parseReportSegments(line: string): ReportSegment[] {
    const segments: ReportSegment[] = [];
    const regex = /\[([^\]]+)\]/g;
    let lastIndex = 0;
    let match: RegExpExecArray | null = null;

    while ((match = regex.exec(line)) !== null) {
      if (match.index > lastIndex) {
        segments.push({
          type: "text",
          value: line.slice(lastIndex, match.index),
          key: `text-${lastIndex}`,
        });
      }

      const refs = match[1]
        .split(",")
        .map((part) => normalizeRef(part))
        .filter((value): value is string => value !== null);

      if (refs.length === 0) {
        segments.push({
          type: "text",
          value: match[0],
          key: `text-${match.index}`,
        });
      } else {
        refs.forEach((ref, refIndex) => {
          segments.push({
            type: "ref",
            value: ref,
            key: `ref-${match?.index ?? 0}-${ref}-${refIndex}`,
          });
          if (refIndex < refs.length - 1) {
            segments.push({
              type: "text",
              value: ", ",
              key: `comma-${match?.index ?? 0}-${refIndex}`,
            });
          }
        });
      }

      lastIndex = regex.lastIndex;
    }

    if (lastIndex < line.length) {
      segments.push({
        type: "text",
        value: line.slice(lastIndex),
        key: `text-tail-${lastIndex}`,
      });
    }

    if (segments.length === 0) {
      segments.push({ type: "text", value: "", key: "empty-line" });
    }

    return segments;
  }

  function reportLines(text: string) {
    return text.split("\n").map((line, index) => ({
      key: `line-${index}`,
      segments: parseReportSegments(line),
    }));
  }

  async function loadTrace(runId: number) {
    try {
      traceData = await invoke<AnalysisTraceData>("get_analysis_run_trace", { runId });
      selectedTraceRef = traceData.refs[0]?.ref ?? null;
    } catch (error) {
      traceData = { refs: [] };
      selectedTraceRef = null;
      status = `Error loading analysis trace: ${error}`;
    }
  }

  async function loadSources() {
    try {
      const result = await invoke<AnalysisSourceOption[]>("list_analysis_sources");
      sources = result.filter((source) => source.item_count > 0);
      if (!selectedSourceId && sources.length > 0) {
        selectedSourceId = String(sources[0].id);
      }
    } catch (error) {
      status = `Error loading analysis sources: ${error}`;
    }
  }

  async function loadTemplates() {
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
    }
  }

  async function loadGroups() {
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
    }
  }

  async function loadRuns() {
    try {
      runs = await invoke<AnalysisRunSummary[]>("list_analysis_runs", {
        sourceId: analysisScope === "single_source" && selectedSourceId ? Number(selectedSourceId) : null,
        sourceGroupId: analysisScope === "source_group" && selectedGroupId ? Number(selectedGroupId) : null,
        limit: 20,
      });
    } catch (error) {
      status = `Error loading analysis runs: ${error}`;
    }
  }

  async function openRun(runId: number) {
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
      if (run.has_trace_data) {
        await loadTrace(run.id);
      } else {
        traceData = { refs: [] };
        selectedTraceRef = null;
      }
    } catch (error) {
      status = `Error loading analysis run: ${error}`;
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
    const history = chatMessages.filter((message) => message.role === "user" || message.role === "assistant");
    chatMessages = [...chatMessages, { role: "user", content: question }, { role: "assistant", content: "" }];
    chatQuestion = "";
    chatting = true;
    activeChatRunId = currentRun.id;

    try {
      const requestId = await invoke<string>("ask_analysis_run_question", {
        runId: currentRun.id,
        question,
        history,
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
        if (payload.message) {
          status = payload.message;
        }
        return;
      }

      if (payload.kind === "failed") {
        chatting = false;
        activeChatRequestId = null;
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
  <p class="status" class:error={status.startsWith("Error") || status.startsWith("Analysis failed")}>
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
        <select bind:value={selectedSourceId}>
          {#if sources.length === 0}
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
        <select bind:value={selectedGroupId}>
          {#if groups.length === 0}
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
      <select bind:value={selectedTemplateId}>
        {#if templates.length === 0}
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
    <div class="panel-header">
      <div>
        <h3>Report Output</h3>
        {#if currentRun}
          <p class="sub">
            {runTargetLabel(currentRun)} - {currentRun.provider}/{currentRun.model}
          </p>
        {/if}
      </div>
    </div>

    {#if currentRun}
      <div class="run-summary-panel">
        <div class="run-summary-header">
          <div class="run-summary-title">
            <strong>Run #{currentRun.id}</strong>
            <span class={`badge badge-${statusTone(currentRun.status)}`}>{currentRun.status}</span>
          </div>
          <span class="sub">
            {currentRun.prompt_template_name ?? "Unknown template"} - v{currentRun.prompt_template_version}
          </span>
        </div>

        <div class="run-meta-grid">
          <div>
            <span class="meta-label">Period</span>
            <strong>{formatPeriod(currentRun.period_from, currentRun.period_to)}</strong>
          </div>
          <div>
            <span class="meta-label">Scope</span>
            <strong>{currentRun.scope_type === "source_group" ? "Source group" : "Single source"}</strong>
          </div>
          <div>
            <span class="meta-label">Output language</span>
            <strong>{currentRun.output_language}</strong>
          </div>
          <div>
            <span class="meta-label">Created</span>
            <strong>{formatTimestamp(currentRun.created_at)}</strong>
          </div>
          <div>
            <span class="meta-label">Completed</span>
            <strong>{formatTimestamp(currentRun.completed_at)}</strong>
          </div>
          <div>
            <span class="meta-label">Provider profile</span>
            <strong>{currentRun.provider_profile}</strong>
          </div>
          <div>
            <span class="meta-label">Trace refs</span>
            <strong>{traceData.refs.length}</strong>
          </div>
        </div>

        {#if currentRun.error}
          <p class="run-error">{currentRun.error}</p>
        {/if}
      </div>
    {/if}

    <div class="report-layout">
      <div class="report-body">
        {#if streamedOutput}
          <div class="report-output">
            {#each reportLines(streamedOutput) as line (line.key)}
              <div class="report-line">
                {#each line.segments as segment (segment.key)}
                  {#if segment.type === "ref"}
                    <button
                      class="ref-chip"
                      class:active={segment.value === selectedTraceRef}
                      type="button"
                      onclick={() => (selectedTraceRef = segment.value)}
                    >
                      [{segment.value}]
                    </button>
                  {:else}
                    <span>{segment.value}</span>
                  {/if}
                {/each}
              </div>
            {/each}
          </div>
        {:else}
          <p class="empty">No report output yet.</p>
        {/if}
      </div>

      <aside class="trace-panel">
        <div class="trace-header">
          <h4>Traceability</h4>
          {#if traceData.refs.length > 0}
            <span class="trace-count">{traceData.refs.length} refs</span>
          {/if}
        </div>

        {#if traceData.refs.length === 0}
          <p class="empty">No saved trace data yet.</p>
        {:else}
          <div class="trace-list">
            {#each traceData.refs as ref}
              <button
                class="trace-link"
                class:selected={ref.ref === selectedTraceRef}
                type="button"
                onclick={() => (selectedTraceRef = ref.ref)}
              >
                <strong>{ref.ref}</strong>
                <span>{formatTimestamp(ref.published_at)}</span>
              </button>
            {/each}
          </div>

          {#if selectedTrace()}
            <div class="trace-detail">
              <div class="trace-meta">
                <strong>{selectedTrace()?.ref}</strong>
                <span>
                  Source {selectedTrace()?.source_id} / message {selectedTrace()?.external_id}
                </span>
                <span>{formatTimestamp(selectedTrace()?.published_at ?? null)}</span>
              </div>
              <blockquote>{selectedTrace()?.excerpt}</blockquote>
            </div>
          {/if}
        {/if}
      </aside>
    </div>
  </section>
</div>

<section class="card chat">
  <div class="panel-header">
    <div>
      <h3>Report Chat</h3>
      <p class="sub">Ask follow-up questions grounded in the saved report and matching synced messages from the same analysis scope.</p>
    </div>
  </div>

  {#if !currentRun}
    <p class="empty">Open a saved run to start a grounded chat.</p>
  {:else if currentRun.status !== "completed"}
    <p class="empty">Chat is available only for completed runs.</p>
  {:else}
    <div class="chat-thread">
      {#if chatMessages.length === 0}
        <p class="empty">No chat turns yet. Ask a follow-up question about this report.</p>
      {:else}
        {#each chatMessages as message, index (`${message.role}-${index}`)}
          <div class={`chat-bubble chat-${message.role}`}>
            <div class="chat-role">{message.role === "user" ? "You" : "Assistant"}</div>
            <div class="chat-content">{message.content || (chatting && message.role === "assistant" ? "..." : "")}</div>
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
      <button onclick={askRunQuestion} disabled={chatting || !currentRun || currentRun.status !== "completed"}>
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

<section class="card history">
  <div class="panel-header">
    <div>
      <h3>Saved Runs</h3>
      <p class="sub">Immutable report runs with saved model, prompt version, and traceability data.</p>
    </div>
    <div class="history-actions">
      <div class="filter-group">
        <button class:activeFilter={runFilter === "all"} class="secondary" onclick={() => (runFilter = "all")}>All</button>
        <button class:activeFilter={runFilter === "completed"} class="secondary" onclick={() => (runFilter = "completed")}>Completed</button>
        <button class:activeFilter={runFilter === "running"} class="secondary" onclick={() => (runFilter = "running")}>Running</button>
        <button class:activeFilter={runFilter === "failed"} class="secondary" onclick={() => (runFilter = "failed")}>Failed</button>
      </div>
      <button class="secondary" onclick={loadRuns}>Refresh</button>
    </div>
  </div>

  {#if runs.length === 0}
    <p class="empty">No analysis runs yet.</p>
  {:else if filteredRuns().length === 0}
    <p class="empty">No runs match the current filter.</p>
  {:else}
    <ul class="run-list">
      {#each filteredRuns() as run}
        <li class:selected={run.id === activeRunId}>
          <div class="run-copy">
            <div class="run-title">
              <strong>{runTargetLabel(run)}</strong>
              <span class={`badge badge-${statusTone(run.status)}`}>{run.status}</span>
            </div>
            <p class="sub">
              {formatTimestamp(run.created_at)} - {run.provider}/{run.model} - {run.prompt_template_name ?? "Unknown template"} v{run.prompt_template_version}
            </p>
            <p class="sub">Period: {formatPeriod(run.period_from, run.period_to)}</p>
            {#if run.completed_at}
              <p class="sub">Completed: {formatTimestamp(run.completed_at)}</p>
            {/if}
            {#if run.error}
              <p class="run-list-error">{run.error}</p>
            {/if}
          </div>
          <button class="secondary" onclick={() => openRun(run.id)}>Open</button>
        </li>
      {/each}
    </ul>
  {/if}
</section>

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
  .report,
  .history {
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

  .run-summary-panel {
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
    padding: 1rem;
    background: var(--panel-strong);
    border: 1px solid var(--border);
    border-radius: 10px;
  }

  .run-summary-header {
    display: flex;
    justify-content: space-between;
    gap: 1rem;
    align-items: flex-start;
    flex-wrap: wrap;
  }

  .run-summary-title {
    display: flex;
    gap: 0.6rem;
    align-items: center;
    flex-wrap: wrap;
  }

  .run-meta-grid {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 0.8rem;
  }

  .run-meta-grid > div {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
    padding: 0.75rem 0.85rem;
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 10px;
  }

  .meta-label {
    color: var(--muted);
    font-size: 0.78rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .run-error,
  .run-list-error {
    margin: 0;
    padding: 0.7rem 0.85rem;
    border-radius: 8px;
    background: var(--status-error-bg);
    color: var(--status-error-text);
    font-size: 0.88rem;
  }

  .report-body,
  .trace-panel {
    min-width: 0;
  }

  .report-output {
    margin: 0;
    padding: 1rem;
    background: var(--panel-strong);
    border: 1px solid var(--border);
    border-radius: 10px;
    min-height: 22rem;
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    font: inherit;
    line-height: 1.6;
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

  .trace-panel {
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
    padding: 1rem;
    background: var(--panel-strong);
    border: 1px solid var(--border);
    border-radius: 10px;
    min-height: 22rem;
  }

  .trace-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
    flex-wrap: wrap;
  }

  .trace-header h4 {
    margin: 0;
  }

  .trace-count {
    color: var(--muted);
    font-size: 0.85rem;
  }

  .trace-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .trace-link {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 0.2rem;
    width: 100%;
    padding: 0.75rem 0.85rem;
    background: var(--panel);
    border: 1px solid var(--border);
    border-radius: 10px;
    color: var(--text);
    text-align: left;
  }

  .trace-link:hover,
  .trace-link.selected {
    background: var(--panel-hover);
    border-color: var(--primary);
  }

  .trace-link span,
  .trace-meta span {
    color: var(--muted);
    font-size: 0.82rem;
  }

  .trace-detail {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
    padding-top: 0.25rem;
    border-top: 1px solid var(--border);
  }

  .trace-meta {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
  }

  blockquote {
    margin: 0;
    padding: 0.9rem 1rem;
    border-left: 4px solid color-mix(in srgb, var(--primary) 45%, transparent);
    background: color-mix(in srgb, var(--panel) 70%, transparent);
    border-radius: 0 10px 10px 0;
    color: var(--text);
    white-space: pre-wrap;
    word-break: break-word;
  }

  .run-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .run-list li {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
    padding: 1rem;
    border: 1px solid var(--border);
    background: var(--panel-strong);
    border-radius: 10px;
  }

  .run-list li.selected {
    border-color: var(--primary);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--primary) 12%, transparent);
  }

  .run-copy {
    min-width: 0;
  }

  .run-title {
    display: flex;
    gap: 0.6rem;
    align-items: center;
    flex-wrap: wrap;
    margin-bottom: 0.35rem;
  }

  .badge {
    padding: 0.2rem 0.55rem;
    border-radius: 999px;
    background: var(--panel-hover);
    color: var(--muted);
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .badge-success {
    background: color-mix(in srgb, #1f8f5f 16%, var(--panel));
    color: #1f8f5f;
  }

  .badge-danger {
    background: color-mix(in srgb, var(--danger) 16%, var(--panel));
    color: var(--danger);
  }

  .badge-info {
    background: color-mix(in srgb, var(--primary) 16%, var(--panel));
    color: var(--primary);
  }

  .history-actions,
  .filter-group {
    display: flex;
    gap: 0.6rem;
    flex-wrap: wrap;
    align-items: center;
  }

  .activeFilter {
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--primary) 14%, transparent);
    border-color: var(--primary);
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

    .run-meta-grid {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }

  @media (max-width: 720px) {
    .grid {
      grid-template-columns: 1fr;
    }

    .run-list li {
      flex-direction: column;
      align-items: stretch;
    }

    .run-meta-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
