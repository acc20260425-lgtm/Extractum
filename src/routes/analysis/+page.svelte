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

  interface AnalysisRunSummary {
    id: number;
    run_type: string;
    scope_type: string;
    source_id: number | null;
    source_title: string | null;
    period_from: number;
    period_to: number;
    output_language: string;
    prompt_template_id: number | null;
    prompt_template_version: number;
    provider_profile: string;
    provider: string;
    model: string;
    status: string;
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

  interface EventEnvelope<T> {
    payload: T;
  }

  let sources = $state<AnalysisSourceOption[]>([]);
  let templates = $state<AnalysisPromptTemplate[]>([]);
  let runs = $state<AnalysisRunSummary[]>([]);

  let selectedSourceId = $state("");
  let selectedTemplateId = $state("");
  let periodFrom = $state(defaultDateOffset(-30));
  let periodTo = $state(defaultDateOffset(0));
  let outputLanguage = $state("Russian");
  let modelOverride = $state("");
  let templateName = $state("");
  let templateBody = $state("");
  let editorBoundTemplateId = $state<number | null>(null);
  let savingTemplate = $state(false);
  let deletingTemplate = $state(false);

  let status = $state("");
  let running = $state(false);
  let activeRunId = $state<number | null>(null);
  let activePhase = $state("");
  let activeProgress = $state("");
  let streamedOutput = $state("");
  let currentRun = $state<AnalysisRunDetail | null>(null);
  let traceData = $state<AnalysisTraceData>({ refs: [] });
  let selectedTraceRef = $state<string | null>(null);

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

  async function loadRuns() {
    try {
      runs = await invoke<AnalysisRunSummary[]>("list_analysis_runs", {
        sourceId: selectedSourceId ? Number(selectedSourceId) : null,
        limit: 20,
      });
    } catch (error) {
      status = `Error loading analysis runs: ${error}`;
    }
  }

  async function openRun(runId: number) {
    try {
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
    if (!selectedSourceId) {
      status = "Select a source first.";
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
    activePhase = "queued";
    activeProgress = "";

    try {
      const runId = await invoke<number>("start_analysis_report", {
        sourceId: Number(selectedSourceId),
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

  $effect(() => {
    if (selectedSourceId) {
      void loadRuns();
    }
  });

  $effect(() => {
    const selected = selectedTemplate();
    if (selected && editorBoundTemplateId !== selected.id) {
      bindEditorToTemplate(selected);
    }
  });

  onMount(() => {
    let disposed = false;
    let detachAnalysisListener: (() => void) | null = null;

    void loadSources();
    void loadTemplates();
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

    return () => {
      disposed = true;
      if (detachAnalysisListener !== null) {
        detachAnalysisListener();
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

    <button onclick={runReport} disabled={running || !selectedSourceId || !selectedTemplateId}>
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
            {currentRun.source_title ?? `Source ${currentRun.source_id ?? "?"}`} - {currentRun.provider}/{currentRun.model}
          </p>
        {/if}
      </div>
    </div>

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

<section class="card history">
  <div class="panel-header">
    <h3>Saved Runs</h3>
    <button class="secondary" onclick={loadRuns}>Refresh</button>
  </div>

  {#if runs.length === 0}
    <p class="empty">No analysis runs yet.</p>
  {:else}
    <ul class="run-list">
      {#each runs as run}
        <li class:selected={run.id === activeRunId}>
          <div class="run-copy">
            <div class="run-title">
              <strong>{run.source_title ?? `Source ${run.source_id ?? "?"}`}</strong>
              <span class="badge">{run.status}</span>
            </div>
            <p class="sub">
              {formatTimestamp(run.created_at)} - {run.provider}/{run.model} - template v{run.prompt_template_version}
            </p>
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

  .templates {
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
  }

  @media (max-width: 720px) {
    .grid {
      grid-template-columns: 1fr;
    }

    .run-list li {
      flex-direction: column;
      align-items: stretch;
    }
  }
</style>
