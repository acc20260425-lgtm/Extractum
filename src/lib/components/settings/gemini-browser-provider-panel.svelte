<script lang="ts">
  import { ExternalLink, Play, RefreshCw, Send, Square } from "@lucide/svelte";
  import { onMount } from "svelte";
  import {
    geminiBridgeListRuns,
    geminiBridgeOpenBrowser,
    geminiBridgeResume,
    geminiBridgeSendSingle,
    geminiBridgeStartCdpChrome,
    geminiBridgeStatus,
    geminiBridgeStop,
    listenToGeminiBrowserRuns,
  } from "$lib/api/gemini-browser";
  import { formatAppError } from "$lib/app-error";
  import { statusLabel } from "$lib/gemini-browser-provider-panel-contract";
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
  let browserProviderMode = $state<GeminiBrowserProviderMode>("managed");
  let cdpEndpoint = $state(DEFAULT_CDP_ENDPOINT);

  function newRunId() {
    return `gemini-browser-${Date.now()}-${Math.random().toString(16).slice(2)}`;
  }

  function currentStatusLabel() {
    return statusLabel(status?.status ?? "not_started", status?.manual_action ?? null);
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

  async function refresh() {
    try {
      const [nextStatus, log] = await Promise.all([
        geminiBridgeStatus(browserConfig()),
        geminiBridgeListRuns(8),
      ]);
      status = nextStatus;
      runs = log.runs;
      message = nextStatus.latest_message ?? "";
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
    try {
      result = await geminiBridgeSendSingle({
        runId: newRunId(),
        prompt: prompt.trim(),
        source: "settings_test",
        artifactMode: "reduced",
        browserConfig: browserConfig(),
      });
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

  <div class="runs-list">
    <h3>Recent browser runs</h3>
    {#each runs as run (run.run_id)}
      <div class="run-row">
        <span>{run.status}</span>
        <code>{run.run_id}</code>
        <p>{run.prompt_preview}</p>
      </div>
    {:else}
      <p class="empty">No browser runs yet.</p>
    {/each}
  </div>
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

  .provider-card button {
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

  .mono,
  .run-row code {
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
  }

  .run-row {
    align-items: flex-start;
    border-bottom: 1px solid var(--border);
    padding: 8px 0;
  }

  .run-row span {
    min-width: 110px;
    font-weight: 700;
  }

  .run-row p {
    margin: 0;
    flex: 1;
  }

  @media (max-width: 820px) {
    .provider-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
