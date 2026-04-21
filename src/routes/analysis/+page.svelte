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

  let status = $state("");
  let running = $state(false);
  let activeRunId = $state<number | null>(null);
  let activePhase = $state("");
  let activeProgress = $state("");
  let streamedOutput = $state("");
  let currentRun = $state<AnalysisRunDetail | null>(null);

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

  $effect(() => {
    if (selectedSourceId) {
      void loadRuns();
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

    {#if streamedOutput}
      <pre>{streamedOutput}</pre>
    {:else}
      <p class="empty">No report output yet.</p>
    {/if}
  </section>
</div>

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

  pre {
    margin: 0;
    padding: 1rem;
    background: var(--panel-strong);
    border: 1px solid var(--border);
    border-radius: 10px;
    min-height: 22rem;
    white-space: pre-wrap;
    word-break: break-word;
    font: inherit;
    line-height: 1.6;
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

  @media (max-width: 1080px) {
    .workspace {
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
