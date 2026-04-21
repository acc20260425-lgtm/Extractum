<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";

  interface LlmProfile {
    profile_id: string;
    provider: string;
    default_model: string;
    api_key: string;
  }

  interface LlmProfilesState {
    active_profile: string;
    default_profile: LlmProfile;
  }

  interface LlmUsage {
    input_tokens: number | null;
    output_tokens: number | null;
    total_tokens: number | null;
  }

  interface LlmStreamEvent {
    request_id: string;
    kind: "started" | "delta" | "completed" | "failed";
    delta: string | null;
    text: string | null;
    provider: string;
    model: string;
    usage: LlmUsage | null;
    error: string | null;
  }

  interface LlmStreamEnvelope<T> {
    payload: T;
  }

  let activeProfile = $state("default");
  let provider = $state("gemini");
  let defaultModel = $state("gemini-2.5-flash");
  let apiKey = $state("");
  let settingsStatus = $state("");
  let saving = $state(false);

  let testPrompt = $state("Summarize why local-first analysis tools are useful for research.");
  let testStatus = $state("");
  let testOutput = $state("");
  let testUsage = $state<LlmUsage | null>(null);
  let testing = $state(false);
  let activeRequestId = $state<string | null>(null);
  let lastProvider = $state("");
  let lastModel = $state("");
  let settingsStatusTimer: ReturnType<typeof setTimeout> | null = null;

  function setSettingsStatus(message: string) {
    settingsStatus = message;
    if (settingsStatusTimer !== null) {
      clearTimeout(settingsStatusTimer);
    }
    settingsStatusTimer = setTimeout(() => {
      settingsStatus = "";
      settingsStatusTimer = null;
    }, 5000);
  }

  async function loadProfiles() {
    try {
      const state = await invoke<LlmProfilesState>("get_llm_profiles");
      activeProfile = state.active_profile;
      provider = state.default_profile.provider;
      defaultModel = state.default_profile.default_model;
      apiKey = state.default_profile.api_key;
    } catch (error) {
      setSettingsStatus(`Error loading LLM settings: ${error}`);
    }
  }

  async function saveProfile(successMessage = "LLM settings saved.") {
    saving = true;
    settingsStatus = "";

    try {
      const state = await invoke<LlmProfilesState>("save_llm_profile", {
        profileId: "default",
        provider,
        defaultModel,
        apiKey,
        setActive: true,
      });

      activeProfile = state.active_profile;
      provider = state.default_profile.provider;
      defaultModel = state.default_profile.default_model;
      apiKey = state.default_profile.api_key;
      setSettingsStatus(successMessage);
      return true;
    } catch (error) {
      setSettingsStatus(`Error saving LLM settings: ${error}`);
      return false;
    } finally {
      saving = false;
    }
  }

  function newRequestId() {
    return `llm-${Date.now()}-${Math.random().toString(16).slice(2)}`;
  }

  async function runTest() {
    if (!testPrompt.trim()) {
      testStatus = "Enter a test prompt first.";
      return;
    }

    const saved = await saveProfile("LLM settings saved before test run.");
    if (!saved) return;

    testOutput = "";
    testUsage = null;
    testStatus = "";
    testing = true;
    activeRequestId = newRequestId();
    lastProvider = provider;
    lastModel = defaultModel;

    try {
      await invoke("ask_llm_stream", {
        requestId: activeRequestId,
        profileId: activeProfile || "default",
        messages: [
          {
            role: "user",
            content: testPrompt.trim(),
          },
        ],
        modelOverride: null,
      });
    } catch (error) {
      testing = false;
      testStatus = `Error starting LLM test: ${error}`;
    }
  }

  function usageLine(usage: LlmUsage | null) {
    if (!usage) return "";

    const parts = [
      usage.input_tokens !== null ? `input ${usage.input_tokens}` : null,
      usage.output_tokens !== null ? `output ${usage.output_tokens}` : null,
      usage.total_tokens !== null ? `total ${usage.total_tokens}` : null,
    ].filter(Boolean);

    return parts.join(" | ");
  }

  onMount(() => {
    let disposed = false;
    let detachListener: (() => void) | null = null;

    void loadProfiles();
    void listen<LlmStreamEvent>("llm://response", ({ payload }: LlmStreamEnvelope<LlmStreamEvent>) => {
      if (disposed || payload.request_id !== activeRequestId) {
        return;
      }

      if (payload.kind === "started") {
        testStatus = `Streaming response from ${payload.provider}/${payload.model}...`;
        lastProvider = payload.provider;
        lastModel = payload.model;
        return;
      }

      if (payload.kind === "delta") {
        testOutput += payload.delta ?? "";
        return;
      }

      if (payload.kind === "completed") {
        testing = false;
        testOutput = payload.text ?? testOutput;
        testUsage = payload.usage;
        lastProvider = payload.provider;
        lastModel = payload.model;
        testStatus = `Response completed from ${payload.provider}/${payload.model}.`;
        return;
      }

      if (payload.kind === "failed") {
        testing = false;
        lastProvider = payload.provider;
        lastModel = payload.model;
        testStatus = `LLM request failed: ${payload.error ?? "Unknown provider error"}`;
      }
    }).then((unlisten) => {
      if (disposed) {
        unlisten();
        return;
      }
      detachListener = unlisten;
    });

    return () => {
      disposed = true;
      if (detachListener !== null) {
        detachListener();
      }
      if (settingsStatusTimer !== null) {
        clearTimeout(settingsStatusTimer);
      }
    };
  });
</script>

<h1>Settings</h1>

{#if settingsStatus}
  <p class="status" class:error={settingsStatus.startsWith("Error")}>{settingsStatus}</p>
{/if}

<div class="card">
  <h3>LLM Provider</h3>
  <p class="hint">
    First implementation is Gemini-only. The API key is temporarily stored in local SQLite for development
    convenience. This is a known security debt and will later move to secure storage.
  </p>

  <div class="grid">
    <label>Active profile
      <input type="text" bind:value={activeProfile} disabled />
    </label>

    <label>Provider
      <input type="text" bind:value={provider} disabled />
    </label>
  </div>

  <label>Default model
    <input type="text" bind:value={defaultModel} placeholder="gemini-2.5-flash" />
  </label>

  <label>API key
    <input type="text" bind:value={apiKey} placeholder="AIza..." />
  </label>

  <div class="actions">
    <button onclick={() => saveProfile()} disabled={saving || !defaultModel.trim()}>
      {saving ? "Saving..." : "Save"}
    </button>
  </div>
</div>

<div class="card">
  <h3>Test Provider</h3>
  <label>Prompt
    <textarea bind:value={testPrompt} rows="6" placeholder="Ask Gemini something simple..." />
  </label>

  <div class="actions">
    <button onclick={runTest} disabled={testing || !testPrompt.trim() || !defaultModel.trim()}>
      {testing ? "Streaming..." : "Run test"}
    </button>
  </div>

  {#if testStatus}
    <p class="status" class:error={testStatus.startsWith("LLM request failed") || testStatus.startsWith("Error")}>
      {testStatus}
    </p>
  {/if}

  <div class="output-card">
    <div class="output-header">
      <span class="output-label">Streaming output</span>
      {#if lastProvider || lastModel}
        <span class="output-meta">{lastProvider}{lastProvider && lastModel ? " / " : ""}{lastModel}</span>
      {/if}
    </div>
    {#if testOutput}
      <pre>{testOutput}</pre>
    {:else}
      <p class="empty">No output yet.</p>
    {/if}
    {#if testUsage}
      <p class="usage">{usageLine(testUsage)}</p>
    {/if}
  </div>
</div>

<style>
  .card {
    background: var(--panel);
    border: 1px solid var(--border);
    box-shadow: var(--shadow);
    border-radius: 12px;
    padding: 1.5rem;
    margin-bottom: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
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
    min-height: 8rem;
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

  .actions {
    display: flex;
    gap: 0.6rem;
    flex-wrap: wrap;
  }

  .hint {
    margin: 0;
    color: var(--muted);
    font-size: 0.9rem;
    line-height: 1.5;
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

  .output-card {
    border: 1px solid var(--border);
    background: var(--panel-strong);
    border-radius: 10px;
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    min-height: 14rem;
  }

  .output-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
    flex-wrap: wrap;
  }

  .output-label {
    font-size: 0.95rem;
    font-weight: 600;
  }

  .output-meta,
  .usage,
  .empty {
    margin: 0;
    color: var(--muted);
    font-size: 0.9rem;
  }

  pre {
    margin: 0;
    white-space: pre-wrap;
    word-break: break-word;
    font: inherit;
    line-height: 1.6;
  }

  @media (max-width: 720px) {
    .grid {
      grid-template-columns: 1fr;
    }
  }
</style>
