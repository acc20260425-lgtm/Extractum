<script lang="ts">
  import { Clipboard, ExternalLink, FolderOpen, Play, RefreshCw, Send, Square } from "@lucide/svelte";
  import { onMount } from "svelte";
  import {
    geminiBridgeListRuns,
    geminiBridgeOpenBrowser,
    geminiBridgeOpenRunFolder,
    geminiBridgeResume,
    geminiBridgeSendSingle,
    geminiBridgeStartCdpChrome,
    geminiBridgeStatus,
    geminiBridgeStop,
    listenToGeminiBrowserRuns,
  } from "$lib/api/gemini-browser";
  import { formatAppError } from "$lib/app-error";
  import { statusLabel } from "$lib/gemini-browser-provider-panel-contract";
  import { runResultForActivePrompt } from "$lib/gemini-browser-provider-panel-state";
  import {
    artifactAvailability,
    copyableRunDiagnostics,
    debugFinalTextLength,
    filterRunHistoryRows,
    isPartialRiskBrowserResult,
    resultTextLength,
    sanitizeDiagnosticMessage,
    selectRunForHistory,
    type GeminiBrowserRunHistoryFilter,
  } from "$lib/gemini-browser-run-inspector";
  import type {
    GeminiBrowserProviderConfig,
    GeminiBrowserProviderMode,
    GeminiBrowserProviderStatus,
    GeminiBrowserRun,
    GeminiBrowserRunResult,
  } from "$lib/types/gemini-browser";

  const DEFAULT_CDP_ENDPOINT = "http://127.0.0.1:9222";
  const PROVIDER_MODE_STORAGE_KEY = "extractum.geminiBrowser.providerMode";
  const CDP_ENDPOINT_STORAGE_KEY = "extractum.geminiBrowser.cdpEndpoint";

  let status = $state<GeminiBrowserProviderStatus | null>(null);
  let runs = $state<GeminiBrowserRun[]>([]);
  let prompt = $state("Reply with one short sentence confirming the browser provider is connected.");
  let busy = $state(false);
  let message = $state("");
  let result = $state<GeminiBrowserRunResult | null>(null);
  let activeTestRunId = $state<string | null>(null);
  let browserProviderMode = $state<GeminiBrowserProviderMode>("managed");
  let cdpEndpoint = $state(DEFAULT_CDP_ENDPOINT);
  let inspectorMessage = $state("");
  let runHistoryFilter = $state<GeminiBrowserRunHistoryFilter>("all");
  let selectedHistoryRunId = $state<string | null>(null);
  const activeInspectorRunId = $derived(activeTestRunId ?? status?.active_run_id ?? null);
  const runHistoryRows = $derived(filterRunHistoryRows(runs, runHistoryFilter));
  const selectedInspectorRun = $derived(
    selectRunForHistory(runs, activeInspectorRunId, selectedHistoryRunId, runHistoryFilter),
  );
  const selectedInspectorResult = $derived(selectedInspectorRun?.result ?? null);
  const selectedArtifactAvailability = $derived(artifactAvailability(selectedInspectorResult));
  const selectedPartialRisk = $derived(isPartialRiskBrowserResult(selectedInspectorResult));

  function newRunId() {
    return `gemini-browser-${Date.now()}-${Math.random().toString(16).slice(2)}`;
  }

  function currentStatusLabel() {
    return statusLabel(status?.status ?? "not_started", status?.manual_action ?? null);
  }

  function selectRunHistoryFilter(filter: GeminiBrowserRunHistoryFilter) {
    runHistoryFilter = filter;
  }

  function selectHistoryRun(runId: string) {
    selectedHistoryRunId = runId;
    inspectorMessage = "";
  }

  function historyFilterLabel(filter: GeminiBrowserRunHistoryFilter) {
    if (filter === "all") return "All";
    if (filter === "problems") return "Problems";
    if (filter === "partial_risk") return "Partial risk";
    if (filter === "manual_action") return "Manual action";
    return "Failed";
  }

  function formatRunUpdatedAt(value: string) {
    const date = new Date(value);
    if (Number.isNaN(date.getTime())) return value;
    return date.toLocaleString();
  }

  function formatRunElapsed(ms: number | null) {
    if (ms === null) return "pending";
    return `${ms} ms`;
  }

  function browserConfig(): GeminiBrowserProviderConfig {
    if (browserProviderMode === "managed") {
      return { mode: "managed" };
    }
    return {
      mode: "cdp_attach",
      cdpEndpoint: cdpEndpoint.trim() || DEFAULT_CDP_ENDPOINT,
    };
  }

  function loadBrowserProviderConfig() {
    if (typeof localStorage === "undefined") return;
    const storedMode = localStorage.getItem(PROVIDER_MODE_STORAGE_KEY);
    if (storedMode === "managed" || storedMode === "cdp_attach") {
      browserProviderMode = storedMode;
    }
    cdpEndpoint = localStorage.getItem(CDP_ENDPOINT_STORAGE_KEY) || DEFAULT_CDP_ENDPOINT;
  }

  function persistBrowserProviderConfig() {
    if (typeof localStorage === "undefined") return;
    localStorage.setItem(PROVIDER_MODE_STORAGE_KEY, browserProviderMode);
    localStorage.setItem(CDP_ENDPOINT_STORAGE_KEY, cdpEndpoint.trim() || DEFAULT_CDP_ENDPOINT);
  }

  function selectBrowserProviderMode(mode: GeminiBrowserProviderMode) {
    browserProviderMode = mode;
    persistBrowserProviderConfig();
    void refresh();
  }

  function updateCdpEndpoint(value: string) {
    cdpEndpoint = value;
    persistBrowserProviderConfig();
  }

  function syncActivePromptResult(nextRuns: GeminiBrowserRun[]) {
    const completedResult = runResultForActivePrompt(nextRuns, activeTestRunId);
    if (!completedResult) return;
    result = completedResult;
    activeTestRunId = null;
    message = completedResult.message ?? completedResult.status;
  }

  async function refresh() {
    try {
      const [nextStatus, log] = await Promise.all([
        geminiBridgeStatus(browserConfig()),
        geminiBridgeListRuns(8),
      ]);
      status = nextStatus;
      runs = log.runs;
      message = nextStatus.latest_message ?? "";
      syncActivePromptResult(log.runs);
    } catch (error) {
      message = formatAppError("loading Gemini browser provider", error);
    }
  }

  async function openBrowser() {
    busy = true;
    try {
      status = await geminiBridgeOpenBrowser(browserConfig());
      message = status.latest_message ?? "Browser opened.";
    } catch (error) {
      message = formatAppError("opening Gemini browser", error);
    } finally {
      busy = false;
    }
  }

  async function startCdpChrome() {
    busy = true;
    try {
      const launch = await geminiBridgeStartCdpChrome(browserConfig());
      message = launch.message;
      await refresh();
    } catch (error) {
      message = formatAppError("starting Chrome for Gemini browser provider", error);
    } finally {
      busy = false;
    }
  }

  async function sendTestPrompt() {
    if (!prompt.trim()) {
      message = "Enter a prompt first.";
      return;
    }
    busy = true;
    result = null;
    const runId = newRunId();
    activeTestRunId = runId;
    selectedHistoryRunId = runId;
    try {
      result = await geminiBridgeSendSingle({
        runId,
        prompt: prompt.trim(),
        source: "settings_test",
        artifactMode: "reduced",
        browserConfig: browserConfig(),
      });
      activeTestRunId = null;
      message = result.message ?? result.status;
      await refresh();
    } catch (error) {
      message = formatAppError("running Gemini browser prompt", error);
    } finally {
      busy = false;
    }
  }

  async function resumeProvider() {
    busy = true;
    try {
      status = await geminiBridgeResume(browserConfig());
      message = status.latest_message ?? "Browser resumed.";
      await refresh();
    } catch (error) {
      message = formatAppError("resuming Gemini browser provider", error);
    } finally {
      busy = false;
    }
  }

  async function stopProvider() {
    busy = true;
    try {
      await geminiBridgeStop();
      await refresh();
    } catch (error) {
      message = formatAppError("stopping Gemini browser provider", error);
    } finally {
      busy = false;
    }
  }

  async function copyDiagnostics() {
    if (!selectedInspectorRun) {
      inspectorMessage = "No browser run is selected.";
      return;
    }
    try {
      await navigator.clipboard.writeText(copyableRunDiagnostics(selectedInspectorRun));
      inspectorMessage = "Diagnostics copied.";
    } catch (error) {
      inspectorMessage = formatAppError("copying Gemini browser diagnostics", error);
    }
  }

  async function openSelectedRunFolder() {
    if (!selectedInspectorRun?.result?.artifacts.run_dir) {
      inspectorMessage = "Run folder is not available.";
      return;
    }
    try {
      await geminiBridgeOpenRunFolder(selectedInspectorRun.run_id);
      inspectorMessage = "Run folder opened.";
    } catch (error) {
      inspectorMessage = formatAppError("opening Gemini browser run folder", error);
    }
  }

  onMount(() => {
    let disposed = false;
    let unlisten: (() => void) | null = null;
    loadBrowserProviderConfig();
    void refresh();
    void listenToGeminiBrowserRuns(({ payload }) => {
      if (disposed) return;
      message = payload.message ?? payload.status;
      void refresh();
    }).then((detach) => {
      if (disposed) {
        detach();
        return;
      }
      unlisten = detach;
    });
    return () => {
      disposed = true;
      unlisten?.();
    };
  });
</script>

<div class="gemini-browser-panel">
  <div class="panel-head">
    <div>
      <h2>Browser Providers</h2>
      <p>Gemini through a persistent local browser profile.</p>
    </div>
    <span class="status-pill">{currentStatusLabel()}</span>
  </div>

  <div class="provider-grid">
    <div class="provider-card">
      <div class="row">
        <strong>Gemini Browser</strong>
        <button type="button" onclick={refresh} disabled={busy} title="Refresh status">
          <RefreshCw size={14} />
        </button>
      </div>
      <div class="mode-group" aria-label="Browser provider mode">
        <button
          type="button"
          class:active={browserProviderMode === "managed"}
          onclick={() => selectBrowserProviderMode("managed")}
          disabled={busy}
        >
          Managed
        </button>
        <button
          type="button"
          class:active={browserProviderMode === "cdp_attach"}
          onclick={() => selectBrowserProviderMode("cdp_attach")}
          disabled={busy}
        >
          Attach Chrome
        </button>
      </div>
      {#if browserProviderMode === "cdp_attach"}
        <div class="cdp-config">
          <label for="gemini-browser-cdp-endpoint">CDP endpoint</label>
          <input
            id="gemini-browser-cdp-endpoint"
            value={cdpEndpoint}
            oninput={(event) => updateCdpEndpoint((event.currentTarget as HTMLInputElement).value)}
            disabled={busy}
          />
          <p class="message">Start Chrome on this endpoint, open Gemini, then Resume.</p>
        </div>
      {/if}
      <p class="mono">{status?.browser_profile_dir ?? "Profile path will appear after status load."}</p>
      {#if message}
        <p class="message">{message}</p>
      {/if}
      <div class="actions">
        {#if browserProviderMode === "cdp_attach"}
          <button type="button" onclick={startCdpChrome} disabled={busy}>
            <Play size={14} />
            <span>Start Chrome</span>
          </button>
        {/if}
        <button type="button" onclick={openBrowser} disabled={busy}>
          <ExternalLink size={14} />
          <span>Open</span>
        </button>
        <button type="button" onclick={resumeProvider} disabled={busy}>
          <Play size={14} />
          <span>Resume</span>
        </button>
        <button type="button" onclick={stopProvider} disabled={busy}>
          <Square size={14} />
          <span>Stop</span>
        </button>
      </div>
    </div>

    <div class="provider-card">
      <label for="gemini-browser-prompt">Test prompt</label>
      <textarea id="gemini-browser-prompt" bind:value={prompt} rows="5"></textarea>
      <button type="button" onclick={sendTestPrompt} disabled={busy || !prompt.trim()}>
        <Send size={14} />
        <span>{busy ? "Running..." : "Send"}</span>
      </button>
      {#if result?.text}
        <pre>{result.text}</pre>
      {/if}
    </div>
  </div>

  <section class="run-inspector" aria-label="Run inspector">
    <div class="row inspector-head">
      <div>
        <h3>Run inspector</h3>
        <p>Latest Browser Provider run diagnostics.</p>
      </div>
      <div class="actions">
        <button type="button" onclick={refresh} disabled={busy} title="Refresh run diagnostics">
          <RefreshCw size={14} />
          <span>Refresh</span>
        </button>
        <button type="button" onclick={copyDiagnostics} disabled={!selectedInspectorRun}>
          <Clipboard size={14} />
          <span>Copy diagnostics</span>
        </button>
        <button
          type="button"
          onclick={openSelectedRunFolder}
          disabled={!selectedInspectorResult?.artifacts.run_dir}
        >
          <FolderOpen size={14} />
          <span>Open run folder</span>
        </button>
      </div>
    </div>

    {#if selectedInspectorRun}
      <div class="inspector-grid">
        <div>
          <span class="fact-label">Run</span>
          <code>{selectedInspectorRun.run_id}</code>
        </div>
        <div>
          <span class="fact-label">Status</span>
          <strong>{selectedInspectorRun.status}</strong>
        </div>
        <div>
          <span class="fact-label">Result</span>
          <strong>{selectedInspectorResult?.status ?? "pending"}</strong>
        </div>
        <div>
          <span class="fact-label">Elapsed</span>
          <span>{selectedInspectorResult?.elapsed_ms ?? 0} ms</span>
        </div>
        <div>
          <span class="fact-label">Result text length</span>
          <span>{resultTextLength(selectedInspectorResult)}</span>
        </div>
        <div>
          <span class="fact-label">Debug final length</span>
          <span>{debugFinalTextLength(selectedInspectorResult)}</span>
        </div>
        <div>
          <span class="fact-label">Manual action</span>
          <span>{selectedInspectorResult?.manual_action ?? "none"}</span>
        </div>
        <div class:warning={selectedPartialRisk}>
          <span class="fact-label">Partial risk</span>
          <span>{selectedPartialRisk ? "yes" : "no"}</span>
        </div>
      </div>

      {#if selectedInspectorResult?.message}
        <p class="message">{sanitizeDiagnosticMessage(selectedInspectorResult.message)}</p>
      {/if}

      <div class="inspector-grid compact">
        <div>
          <span class="fact-label">Run folder</span>
          <span>{selectedArtifactAvailability.run_dir ? "available" : "missing"}</span>
        </div>
        <div>
          <span class="fact-label">Telemetry</span>
          <span>{selectedArtifactAvailability.telemetry ? "available" : "missing"}</span>
        </div>
        <div>
          <span class="fact-label">HTML</span>
          <span>{selectedArtifactAvailability.html ? "available" : "not captured"}</span>
        </div>
        <div>
          <span class="fact-label">Screenshot</span>
          <span>{selectedArtifactAvailability.screenshot ? "available" : "not captured"}</span>
        </div>
        <div>
          <span class="fact-label">Answer extraction</span>
          <span>{selectedArtifactAvailability.answer_extraction ? "available" : "not captured"}</span>
        </div>
      </div>

      {#if selectedInspectorResult?.debug_summary}
        <div class="inspector-grid compact">
          <div>
            <span class="fact-label">Mode</span>
            <span>{selectedInspectorResult.debug_summary.mode}</span>
          </div>
          <div>
            <span class="fact-label">Composer</span>
            <span>{selectedInspectorResult.debug_summary.composer_found ? "found" : "missing"}</span>
          </div>
          <div>
            <span class="fact-label">Send</span>
            <span>{selectedInspectorResult.debug_summary.send_button_found ? "found" : "missing"}</span>
          </div>
          <div>
            <span class="fact-label">Busy observed</span>
            <span>{selectedInspectorResult.debug_summary.generation_busy_observed ? "yes" : "no"}</span>
          </div>
          <div>
            <span class="fact-label">Answer selector</span>
            <code>{selectedInspectorResult.debug_summary.answer_selector ?? "none"}</code>
          </div>
          <div>
            <span class="fact-label">Answer reason</span>
            <span>{selectedInspectorResult.debug_summary.answer_completion_reason}</span>
          </div>
          <div>
            <span class="fact-label">Send wait</span>
            <span>{selectedInspectorResult.debug_summary.waited_for_send_ms} ms</span>
          </div>
          <div>
            <span class="fact-label">Answer wait</span>
            <span>{selectedInspectorResult.debug_summary.waited_for_answer_ms} ms</span>
          </div>
          <div>
            <span class="fact-label">Error stage</span>
            <span>{selectedInspectorResult.debug_summary.error_stage ?? "none"}</span>
          </div>
        </div>

        {#if selectedInspectorResult.debug_summary.extraction}
          <div class="inspector-grid compact">
            <div>
              <span class="fact-label">Raw candidates</span>
              <span>{selectedInspectorResult.debug_summary.extraction.raw_candidate_count}</span>
            </div>
            <div>
              <span class="fact-label">Grouped candidates</span>
              <span>{selectedInspectorResult.debug_summary.extraction.grouped_candidate_count}</span>
            </div>
            <div>
              <span class="fact-label">Selected grouping</span>
              <span>{selectedInspectorResult.debug_summary.extraction.selected_grouping}</span>
            </div>
            <div>
              <span class="fact-label">Selected length</span>
              <span>{selectedInspectorResult.debug_summary.extraction.selected_candidate_length}</span>
            </div>
            <div>
              <span class="fact-label">Largest candidate</span>
              <span>{selectedInspectorResult.debug_summary.extraction.largest_candidate_length}</span>
            </div>
            <div>
              <span class="fact-label">Larger valid</span>
              <span>{selectedInspectorResult.debug_summary.extraction.larger_valid_candidate_available ? "yes" : "no"}</span>
            </div>
            <div>
              <span class="fact-label">Signature changes</span>
              <span>{selectedInspectorResult.debug_summary.extraction.candidate_signature_changed_count}</span>
            </div>
            <div>
              <span class="fact-label">Stable polls</span>
              <span>{selectedInspectorResult.debug_summary.extraction.stable_poll_count_after_last_candidate_change}</span>
            </div>
          </div>
        {/if}
      {:else}
        <p class="empty">Debug summary unavailable for this run.</p>
      {/if}
    {:else}
      <p class="empty">No browser run selected.</p>
    {/if}

    {#if inspectorMessage}
      <p class="message">{inspectorMessage}</p>
    {/if}
  </section>

  <section class="runs-list" aria-label="Run history">
    <div class="row history-head">
      <div>
        <h3>Run history</h3>
        <p>Choose a Browser Provider run to inspect.</p>
      </div>
      <div class="history-filters" aria-label="Run history filters">
        <button
          type="button"
          data-filter="all"
          class:active={runHistoryFilter === "all"}
          onclick={() => selectRunHistoryFilter("all")}
        >
          {historyFilterLabel("all")}
        </button>
        <button
          type="button"
          data-filter="problems"
          class:active={runHistoryFilter === "problems"}
          onclick={() => selectRunHistoryFilter("problems")}
        >
          {historyFilterLabel("problems")}
        </button>
        <button
          type="button"
          data-filter="partial_risk"
          class:active={runHistoryFilter === "partial_risk"}
          onclick={() => selectRunHistoryFilter("partial_risk")}
        >
          {historyFilterLabel("partial_risk")}
        </button>
        <button
          type="button"
          data-filter="manual_action"
          class:active={runHistoryFilter === "manual_action"}
          onclick={() => selectRunHistoryFilter("manual_action")}
        >
          {historyFilterLabel("manual_action")}
        </button>
        <button
          type="button"
          data-filter="failed"
          class:active={runHistoryFilter === "failed"}
          onclick={() => selectRunHistoryFilter("failed")}
        >
          {historyFilterLabel("failed")}
        </button>
      </div>
    </div>

    {#each runHistoryRows as row (row.run.run_id)}
      <button
        type="button"
        class="run-row"
        class:selected={selectedInspectorRun?.run_id === row.run.run_id}
        class:warning={row.isProblem}
        onclick={() => selectHistoryRun(row.run.run_id)}
      >
        <span class="run-status">{row.status}</span>
        <span class="run-badge">{row.badge}</span>
        <span class="run-preview">{row.run.prompt_preview || "No prompt preview"}</span>
        <span class="run-meta">{formatRunUpdatedAt(row.run.updated_at)}</span>
        <span class="run-meta">{formatRunElapsed(row.elapsedMs)}</span>
        <span class="run-meta">{row.resultTextLength} chars</span>
        <span class="run-meta">{row.answerCompletionReason ?? "no debug"}</span>
      </button>
    {:else}
      <p class="empty">No browser runs match this filter.</p>
    {/each}
  </section>
</div>

<style>
  .gemini-browser-panel {
    display: flex;
    flex-direction: column;
    gap: 18px;
  }

  .panel-head,
  .row,
  .actions,
  .run-row {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .panel-head {
    justify-content: space-between;
  }

  .panel-head h2,
  .runs-list h3 {
    margin: 0;
    font-size: 18px;
  }

  .panel-head p,
  .message,
  .empty {
    margin: 4px 0 0;
    color: var(--muted-foreground);
    font-size: 13px;
  }

  .status-pill {
    border: 1px solid var(--border);
    border-radius: 999px;
    padding: 4px 10px;
    font-size: 12px;
    font-weight: 700;
    white-space: nowrap;
  }

  .provider-grid {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
    gap: 14px;
  }

  .provider-card {
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 14px;
    background: var(--card);
  }

  .provider-card button,
  .run-inspector button,
  .history-filters button {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 7px 10px;
    background: var(--background);
    color: var(--foreground);
    font-weight: 650;
  }

  .run-inspector {
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 14px;
    background: var(--card);
  }

  .inspector-head {
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 12px;
  }

  .inspector-head h3 {
    margin: 0;
    font-size: 16px;
  }

  .inspector-grid {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 10px;
    margin-top: 10px;
  }

  .inspector-grid.compact {
    grid-template-columns: repeat(4, minmax(0, 1fr));
  }

  .inspector-grid > div {
    min-width: 0;
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 8px;
    background: var(--background);
    overflow-wrap: anywhere;
  }

  .warning {
    border-color: color-mix(in srgb, var(--destructive) 55%, var(--border));
  }

  .fact-label {
    display: block;
    color: var(--muted-foreground);
    font-size: 11px;
    font-weight: 700;
    margin-bottom: 4px;
  }

  .provider-card textarea {
    width: 100%;
    box-sizing: border-box;
    resize: vertical;
    margin: 6px 0 10px;
  }

  .mode-group {
    display: inline-flex;
    gap: 0;
    margin-top: 10px;
    border: 1px solid var(--border);
    border-radius: 6px;
    overflow: hidden;
  }

  .mode-group button {
    border: 0;
    border-radius: 0;
  }

  .mode-group button + button {
    border-left: 1px solid var(--border);
  }

  .mode-group button.active {
    background: var(--accent);
    color: var(--accent-foreground);
  }

  .cdp-config {
    display: grid;
    gap: 6px;
    margin-top: 10px;
  }

  .cdp-config input {
    width: 100%;
    box-sizing: border-box;
  }

  .mono {
    font-family: ui-monospace, SFMono-Regular, Consolas, monospace;
    font-size: 12px;
    overflow-wrap: anywhere;
  }

  pre {
    white-space: pre-wrap;
    overflow-wrap: anywhere;
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 10px;
    max-height: 180px;
    overflow: auto;
  }

  .runs-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 14px;
    background: var(--card);
  }

  .history-head {
    justify-content: space-between;
    align-items: flex-start;
  }

  .history-head h3 {
    margin: 0;
    font-size: 16px;
  }

  .history-filters {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    justify-content: flex-end;
  }

  .history-filters button.active {
    background: var(--accent);
    color: var(--accent-foreground);
  }

  .run-row {
    display: grid;
    grid-template-columns:
      minmax(90px, 0.8fr) minmax(78px, 0.7fr) minmax(220px, 2fr)
      repeat(4, minmax(88px, 1fr));
    gap: 8px;
    align-items: center;
    width: 100%;
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 8px;
    background: var(--background);
    color: var(--foreground);
    text-align: left;
  }

  .run-row.selected {
    outline: 2px solid color-mix(in srgb, var(--accent) 65%, transparent);
    outline-offset: 1px;
  }

  .run-status,
  .run-badge {
    font-weight: 700;
  }

  .run-badge {
    justify-self: start;
    border: 1px solid var(--border);
    border-radius: 999px;
    padding: 2px 7px;
    font-size: 11px;
  }

  .run-preview,
  .run-meta {
    min-width: 0;
    overflow-wrap: anywhere;
  }

  .run-meta {
    color: var(--muted-foreground);
    font-size: 12px;
  }

  @media (max-width: 820px) {
    .provider-grid,
    .inspector-grid,
    .inspector-grid.compact,
    .run-row {
      grid-template-columns: 1fr;
    }
  }
</style>
