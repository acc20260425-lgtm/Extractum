<script lang="ts">
  import { PlayCircle } from "@lucide/svelte";
  import {
    ExtractumCheckbox,
    ExtractumButton,
    ExtractumDialog,
    ExtractumStatusMessage,
    ExtractumTextInput,
  } from "$lib/components/extractum-ui";
  import {
    preflightYoutubeSummaryRun,
    startYoutubeSummaryRun,
  } from "$lib/api/prompt-packs";
  import {
    geminiBridgeListRuns,
    geminiBridgeStatus,
  } from "$lib/api/gemini-browser";
  import {
    deriveGeminiBrowserSetupChecks,
    setupCheckStateLabel,
  } from "$lib/gemini-browser-setup-status";
  import { canStartYoutubeSummary, summarizePreflightPartitions } from "$lib/ui/youtube-summary-workflow";
  import type {
    PromptPackRuntimeProvider,
    YoutubeSummaryPreflightResponse,
  } from "$lib/types/prompt-packs";
  import { getLlmProfiles } from "$lib/api/llm";
  import type { LlmProfile } from "$lib/types/llm";
  import type {
    GeminiBrowserProviderConfig,
    GeminiBrowserProviderMode,
    GeminiBrowserProviderStatus,
    GeminiBrowserRun,
  } from "$lib/types/gemini-browser";

  type YoutubeSummaryLaunchSource = {
    sourceId: number;
    title: string;
  };

  let {
    open = $bindable(false),
    projectId = null,
    source,
    onStarted,
  }: {
    open?: boolean;
    projectId?: number | null;
    source: YoutubeSummaryLaunchSource | null;
    onStarted?: (runId: number) => void;
  } = $props();

  let outputLanguage = $state("ru");
  let controlPreset = $state("detailed_report");
  let profileId = $state("");
  let modelOverride = $state("");
  let runtimeProvider = $state<PromptPackRuntimeProvider>("api");
  let browserProviderMode = $state<GeminiBrowserProviderMode>("managed");
  let cdpEndpoint = $state("http://127.0.0.1:9222");
  let browserStatus = $state<GeminiBrowserProviderStatus | null>(null);
  let browserRuns = $state<GeminiBrowserRun[]>([]);
  let browserStatusLoadError = $state<string | null>(null);
  let includeComments = $state(false);
  let loading = $state(false);
  let preflight = $state<YoutubeSummaryPreflightResponse | null>(null);
  let error = $state<string | null>(null);
  let llmProfiles = $state<LlmProfile[]>([]);

  let partitionSummary = $derived(
    preflight ? summarizePreflightPartitions(preflight) : null,
  );
  const browserProviderConfig = $derived<GeminiBrowserProviderConfig | null>(
    runtimeProvider === "gemini_browser"
      ? browserConfig()
      : null,
  );
  const browserSetupChecks = $derived(
    deriveGeminiBrowserSetupChecks({
      status: browserStatus,
      providerMode: browserProviderMode,
      cdpEndpoint,
      runs: browserRuns,
      selectedRun: browserRuns[0] ?? null,
      busy: loading,
      statusLoadError: browserStatusLoadError,
    }),
  );
  const browserRuntimeBlockingCheck = $derived(
    runtimeProvider === "gemini_browser"
      ? browserSetupChecks.find((check) => check.state === "failed" || check.state === "action_needed") ?? null
      : null,
  );

  $effect(() => {
    if (open) {
      outputLanguage = "ru";
      controlPreset = "detailed_report";
      runtimeProvider = "api";
      browserProviderMode = "managed";
      cdpEndpoint = "http://127.0.0.1:9222";
      browserStatus = null;
      browserRuns = [];
      browserStatusLoadError = null;
      void loadProfiles();
      if (source) queueMicrotask(() => void runPreflight());
    }
  });

  async function loadProfiles() {
    try {
      const state = await getLlmProfiles();
      llmProfiles = state.profiles;
      if (!profileId) {
        profileId = state.active_profile;
      }
    } catch (e) {
      console.error("Failed to load LLM profiles:", e);
    }
  }

  function handleLanguageChange(event: Event) {
    outputLanguage = (event.currentTarget as HTMLSelectElement).value;
    void runPreflight();
  }

  function browserConfig(): GeminiBrowserProviderConfig {
    if (browserProviderMode === "managed") return { mode: "managed" };
    return {
      mode: "cdp_attach",
      cdpEndpoint: cdpEndpoint.trim() || null,
    };
  }

  async function refreshBrowserStatus() {
    if (runtimeProvider !== "gemini_browser") return;
    try {
      browserStatusLoadError = null;
      const [status, runs] = await Promise.all([
        geminiBridgeStatus(browserConfig()),
        geminiBridgeListRuns(5),
      ]);
      browserStatus = status;
      browserRuns = runs.runs;
    } catch (cause) {
      browserStatusLoadError = cause instanceof Error ? cause.message : String(cause);
    }
  }

  function handleRuntimeChange(event: Event) {
    runtimeProvider = (event.currentTarget as HTMLSelectElement).value as PromptPackRuntimeProvider;
    if (runtimeProvider === "gemini_browser") void refreshBrowserStatus();
    void runPreflight();
  }

  async function runPreflight() {
    if (!source || !outputLanguage) return;
    loading = true;
    error = null;
    try {
      preflight = await preflightYoutubeSummaryRun({
        projectId,
        sourceIds: [source.sourceId],
        profileId: runtimeProvider === "api" ? profileId || null : null,
        modelOverride: runtimeProvider === "api" ? modelOverride || null : null,
        runtimeProvider,
        browserProviderConfig: browserProviderConfig,
        outputLanguage,
        controlPreset,
        evidenceMode: "standard",
        includeComments,
      });
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      loading = false;
    }
  }

  async function startRun() {
    if (!source || !outputLanguage || !canStartYoutubeSummary(preflight) || browserRuntimeBlockingCheck) return;
    loading = true;
    error = null;
    const clientRequestId = `youtube-summary-${projectId ?? "global"}-${source.sourceId}-${Date.now()}`;
    try {
      const outcome = await startYoutubeSummaryRun({
        clientRequestId,
        projectId,
        sourceIds: [source.sourceId],
        profileId: runtimeProvider === "api" ? profileId || null : null,
        modelOverride: runtimeProvider === "api" ? modelOverride || null : null,
        runtimeProvider,
        browserProviderConfig: browserProviderConfig,
        outputLanguage,
        controlPreset,
        evidenceMode: "standard",
        includeComments,
      });
      if (outcome.kind === "blocked") {
        preflight = outcome.preflight;
        return;
      }
      onStarted?.(outcome.run.runId);
      open = false;
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      loading = false;
    }
  }
</script>

<ExtractumDialog bind:open title="YouTube Summary">
  <form class="youtube-summary-dialog" onsubmit={(event) => { event.preventDefault(); void startRun(); }}>
    <header class="dialog-banner">
      <div class="header-content">
        <p class="eyebrow">Prompt Pack</p>
        <h2>{source?.title ?? "No source selected"}</h2>
      </div>
      <div class="header-icon">
        <PlayCircle size={22} aria-hidden="true" />
      </div>
    </header>

    {#if error}
      <ExtractumStatusMessage tone="error">{error}</ExtractumStatusMessage>
    {/if}

    <div class="inputs-grid">
      <label>
        <span>Output language</span>
        <select bind:value={outputLanguage} aria-label="Output language" onchange={handleLanguageChange}>
          <option value="ru">ru</option>
          <option value="en">en</option>
        </select>
      </label>
      <label>
        <span>Summary mode</span>
        <select bind:value={controlPreset} aria-label="Summary mode" onchange={() => void runPreflight()}>
          <option value="standard">Standard</option>
          <option value="detailed_report">Detailed report</option>
          <option value="gem_analysis">Gem analysis</option>
        </select>
      </label>
      <label>
        <span>Runtime</span>
        <select bind:value={runtimeProvider} aria-label="Runtime provider" onchange={handleRuntimeChange}>
          <option value="api">API profile</option>
          <option value="gemini_browser">Gemini Browser</option>
        </select>
      </label>
      {#if runtimeProvider === "api"}
        <label>
          <span>LLM profile</span>
          <select bind:value={profileId} aria-label="LLM Profile" onchange={() => void runPreflight()}>
            {#each llmProfiles as profile (profile.profile_id)}
              <option value={profile.profile_id}>
                {profile.profile_id} ({profile.default_model})
              </option>
            {/each}
          </select>
        </label>
        <label class="full-width">
          <span>Model override</span>
          <ExtractumTextInput bind:value={modelOverride} placeholder="Optional" onchange={() => void runPreflight()} />
        </label>
      {:else}
        <label>
          <span>Browser mode</span>
          <select bind:value={browserProviderMode} aria-label="Browser Provider mode" onchange={() => { void refreshBrowserStatus(); void runPreflight(); }}>
            <option value="managed">Managed</option>
            <option value="cdp_attach">Attach Chrome</option>
          </select>
        </label>
        {#if browserProviderMode === "cdp_attach"}
          <label class="full-width">
            <span>CDP endpoint</span>
            <ExtractumTextInput bind:value={cdpEndpoint} onchange={() => { void refreshBrowserStatus(); void runPreflight(); }} />
          </label>
        {/if}
      {/if}
    </div>

    <label class="checkbox-row">
      <ExtractumCheckbox bind:checked={includeComments} onchange={() => void runPreflight()} />
      <span>Include comments</span>
    </label>

    {#if runtimeProvider === "gemini_browser"}
      <section class="preflight-section" aria-label="Browser Provider setup">
        <h3>Browser Provider</h3>
        <div class="browser-checks-list">
          {#each browserSetupChecks.slice(0, 3) as check (check.id)}
            <div class="browser-check-item">
              <span class="status-dot {check.state}" aria-hidden="true"></span>
              <div class="check-details">
                <span class="check-label">{check.label}:</span>
                <span class="check-state-badge {check.state}">{setupCheckStateLabel(check.state)}</span>
                <span class="check-message"> - {check.message}</span>
              </div>
            </div>
          {/each}
        </div>
        <ExtractumButton type="button" variant="outline" onclick={() => void refreshBrowserStatus()} disabled={loading}>
          Refresh Browser Provider
        </ExtractumButton>
      </section>
    {/if}

    <section class="preflight-section" aria-label="Preflight">
      <h3>Preflight check</h3>
      {#if loading && !preflight}
        <div class="loading-state">
          <div class="spinner"></div>
          <span>Checking source readiness...</span>
        </div>
      {:else if partitionSummary}
        <dl class="preflight-stats">
          <div class="stat-card">
            <dt>Ready videos</dt>
            <dd class="stat-value included">{partitionSummary.includedCount}</dd>
          </div>
          <div class="stat-card">
            <dt>Skipped</dt>
            <dd class="stat-value skipped">{partitionSummary.skippedCount}</dd>
          </div>
          <div class="stat-card">
            <dt>Blocking</dt>
            <dd class="stat-value blocking">{partitionSummary.blockingCount}</dd>
          </div>
        </dl>
        {#if partitionSummary.hasPartialCoverage}
          <ExtractumStatusMessage tone="info">Some playlist videos will be skipped.</ExtractumStatusMessage>
        {/if}
        {#if preflight?.blockingFailures.length}
          <ExtractumStatusMessage tone="error">
            {preflight.blockingFailures.map((failure) => failure.reason).join(", ")}
          </ExtractumStatusMessage>
        {/if}
      {:else}
        <p class="preflight-placeholder">Open the dialog to check source readiness.</p>
      {/if}
    </section>

    <footer>
      <ExtractumButton type="button" variant="outline" onclick={() => void runPreflight()} disabled={!source || loading}>Refresh</ExtractumButton>
      <ExtractumButton type="submit" disabled={!source || loading || !canStartYoutubeSummary(preflight) || Boolean(browserRuntimeBlockingCheck)}>Start</ExtractumButton>
    </footer>
  </form>
</ExtractumDialog>

<style>
  .youtube-summary-dialog {
    display: grid;
    min-width: min(560px, calc(100vw - 96px));
    gap: 16px;
  }

  .dialog-banner {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding-bottom: 14px;
    border-bottom: 1px solid var(--extractum-border);
  }

  .header-content {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .header-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    border-radius: 50%;
    background: color-mix(in srgb, var(--extractum-primary) 8%, transparent);
    color: var(--extractum-primary);
  }

  .eyebrow {
    margin: 0;
    color: var(--extractum-muted);
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  h2,
  h3,
  p {
    margin: 0;
  }

  h2 {
    font-size: 16px;
    line-height: 1.25;
    font-weight: 600;
    color: var(--extractum-text);
  }

  h3 {
    font-size: 11px;
    font-weight: 700;
    color: var(--extractum-text);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .inputs-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 12px;
  }

  .inputs-grid .full-width {
    grid-column: 1 / -1;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  label span {
    font-size: 11px;
    font-weight: 600;
    color: var(--extractum-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  select {
    min-height: 32px;
    border: 1px solid var(--extractum-border);
    border-radius: var(--extractum-radius);
    background: var(--extractum-surface);
    color: var(--extractum-text);
    padding: 4px 8px;
    font-size: 13px;
    width: 100%;
  }

  .checkbox-row {
    display: flex;
    flex-direction: row;
    align-items: center;
    gap: 10px;
    padding: 10px 14px;
    border: 1px solid var(--extractum-border);
    border-radius: 8px;
    background: var(--extractum-surface-raised);
    cursor: pointer;
    transition: border-color 0.15s, background 0.15s;
    user-select: none;
  }

  .checkbox-row:hover {
    border-color: var(--extractum-primary);
    background: color-mix(in srgb, var(--extractum-primary) 4%, var(--extractum-surface-raised));
  }

  .checkbox-row span {
    font-size: 12px;
    font-weight: 500;
    color: var(--extractum-text);
    text-transform: none;
    letter-spacing: normal;
  }

  .preflight-section {
    display: flex;
    flex-direction: column;
    gap: 12px;
    border: 1px solid var(--extractum-border);
    border-radius: 8px;
    background: var(--extractum-surface-subtle);
    padding: 14px;
  }

  .loading-state {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 4px 0;
    color: var(--extractum-muted);
    font-size: 12px;
  }

  .spinner {
    width: 16px;
    height: 16px;
    border: 2px solid var(--extractum-border);
    border-top-color: var(--extractum-primary);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .preflight-stats {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 10px;
    margin: 0;
  }

  .stat-card {
    background: var(--extractum-surface-raised);
    border: 1px solid var(--extractum-border);
    border-radius: 6px;
    padding: 8px 12px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .stat-card dt {
    color: var(--extractum-muted);
    font-size: 10px;
    text-transform: uppercase;
    font-weight: 600;
    letter-spacing: 0.02em;
  }

  .stat-card dd {
    margin: 0;
    font-size: 18px;
    font-weight: 700;
  }

  .stat-card dd.included {
    color: var(--extractum-success);
  }

  .stat-card dd.skipped {
    color: var(--extractum-warning);
  }

  .stat-card dd.blocking {
    color: var(--extractum-danger);
  }

  .preflight-placeholder {
    color: var(--extractum-muted);
    font-size: 12px;
    margin: 0;
  }

  footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 4px;
  }

  .browser-checks-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
    background: var(--extractum-surface-raised);
    border: 1px solid var(--extractum-border);
    border-radius: 6px;
    padding: 8px 10px;
  }

  .browser-check-item {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .status-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    flex-shrink: 0;
    background: var(--extractum-muted);
  }

  .status-dot.ready {
    background: var(--extractum-success);
    box-shadow: 0 0 4px var(--extractum-success);
  }

  .status-dot.failed,
  .status-dot.action_needed {
    background: var(--extractum-danger);
    box-shadow: 0 0 4px var(--extractum-danger);
  }

  .status-dot.not_applicable {
    background: var(--extractum-muted);
  }

  .status-dot.unknown {
    background: var(--extractum-muted);
  }

  .status-dot.running {
    background: var(--extractum-primary);
    box-shadow: 0 0 4px var(--extractum-primary);
  }

  .status-dot.warning {
    background: var(--extractum-warning);
    box-shadow: 0 0 4px var(--extractum-warning);
  }

  .check-details {
    font-size: 11px;
    line-height: 1.4;
    color: var(--extractum-text);
  }

  .check-label {
    font-weight: 600;
  }

  .check-state-badge {
    font-weight: 700;
    font-size: 9px;
    text-transform: uppercase;
    padding: 1px 4px;
    border-radius: 3px;
    margin-left: 4px;
    display: inline-block;
  }

  .check-state-badge.ready {
    background: color-mix(in srgb, var(--extractum-success) 12%, transparent);
    color: var(--extractum-success);
  }

  .check-state-badge.failed,
  .check-state-badge.action_needed {
    background: color-mix(in srgb, var(--extractum-danger) 12%, transparent);
    color: var(--extractum-danger);
  }

  .check-state-badge.not_applicable {
    background: color-mix(in srgb, var(--extractum-muted) 12%, transparent);
    color: var(--extractum-muted);
  }

  .check-state-badge.unknown {
    background: color-mix(in srgb, var(--extractum-muted) 12%, transparent);
    color: var(--extractum-muted);
  }

  .check-state-badge.running {
    background: color-mix(in srgb, var(--extractum-primary) 12%, transparent);
    color: var(--extractum-primary);
  }

  .check-state-badge.warning {
    background: color-mix(in srgb, var(--extractum-warning) 12%, transparent);
    color: var(--extractum-warning);
  }

  .check-message {
    color: var(--extractum-muted);
    margin-left: 2px;
  }
</style>
