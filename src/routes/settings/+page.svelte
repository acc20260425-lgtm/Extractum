<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { formatAppError } from "$lib/app-error";
  import DesktopDialog from "$lib/components/desktop-dialog.svelte";

  interface LlmProfile {
    profile_id: string;
    provider: string;
    default_model: string;
    api_key: string;
    base_url: string;
  }

  interface LlmProfilesState {
    active_profile: string;
    profiles: LlmProfile[];
  }

  interface LlmProviderModel {
    model: string;
    name: string;
    display_name: string;
    description: string;
    input_token_limit: number | null;
    output_token_limit: number | null;
    supported_generation_methods: string[];
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

  interface ProviderOption {
    value: string;
    label: string;
    placeholder: string;
    keyPlaceholder: string;
    baseUrlPlaceholder: string;
    supportsBaseUrl: boolean;
  }

  const NEW_PROFILE_OPTION = "__new_profile__";
  const providerOptions: ProviderOption[] = [
    {
      value: "gemini",
      label: "Gemini",
      placeholder: "gemini-2.5-flash",
      keyPlaceholder: "AIza...",
      baseUrlPlaceholder: "",
      supportsBaseUrl: false,
    },
    {
      value: "omniroute",
      label: "OpenAI-compatible",
      placeholder: "if/kimi-k2-thinking",
      keyPlaceholder: "sk_omniroute",
      baseUrlPlaceholder: "http://localhost:20128/v1",
      supportsBaseUrl: true,
    },
  ];

  let profiles = $state<LlmProfile[]>([]);
  let activeProfile = $state("default");
  let selectedProfileId = $state("default");
  let creatingProfile = $state(false);
  let draftProfileId = $state("default");

  let provider = $state("gemini");
  let defaultModel = $state("gemini-2.5-flash");
  let apiKey = $state("");
  let baseUrl = $state("");

  let settingsStatus = $state("");
  let saving = $state(false);
  let availableModels = $state<LlmProviderModel[]>([]);
  let loadingModels = $state(false);
  let modelsStatus = $state("");

  let testPrompt = $state("Summarize why local-first analysis tools are useful for research.");
  let testStatus = $state("");
  let testOutput = $state("");
  let testUsage = $state<LlmUsage | null>(null);
  let testing = $state(false);
  let testDialogOpen = $state(false);
  let activeRequestId = $state<string | null>(null);
  let settingsStatusTimer: ReturnType<typeof setTimeout> | null = null;

  function normalizeProfileId(value: string) {
    return value.trim().toLowerCase();
  }

  function currentProviderOption(value = provider) {
    return providerOptions.find((option) => option.value === value) ?? providerOptions[0];
  }

  function providerLabel(value = provider) {
    return currentProviderOption(value)?.label ?? value;
  }

  function providerPlaceholder() {
    return currentProviderOption().placeholder;
  }

  function apiKeyPlaceholder() {
    return currentProviderOption().keyPlaceholder;
  }

  function providerSupportsBaseUrl(value = provider) {
    return currentProviderOption(value).supportsBaseUrl;
  }

  function providerBaseUrlPlaceholder(value = provider) {
    return currentProviderOption(value).baseUrlPlaceholder;
  }

  function effectiveDraftProfileId() {
    return creatingProfile ? draftProfileId : selectedProfileId;
  }

  function canSaveProfile() {
    return Boolean(defaultModel.trim() && normalizeProfileId(effectiveDraftProfileId()));
  }

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

  function clearModelCatalog() {
    availableModels = [];
    modelsStatus = "";
  }

  function clearTestResult() {
    testStatus = "";
    testOutput = "";
    testUsage = null;
  }

  function applyProfile(profile: LlmProfile) {
    selectedProfileId = profile.profile_id;
    creatingProfile = false;
    draftProfileId = profile.profile_id;
    provider = profile.provider;
    defaultModel = profile.default_model;
    apiKey = profile.api_key;
    baseUrl = profile.base_url;
    clearModelCatalog();
    clearTestResult();
  }

  function syncProfilesState(state: LlmProfilesState, preferredProfileId?: string) {
    profiles = state.profiles;
    activeProfile = state.active_profile;

    const normalizedPreferred = preferredProfileId ? normalizeProfileId(preferredProfileId) : "";
    const nextProfile =
      state.profiles.find((profile) => profile.profile_id === normalizedPreferred) ??
      state.profiles.find((profile) => profile.profile_id === selectedProfileId) ??
      state.profiles.find((profile) => profile.profile_id === state.active_profile) ??
      state.profiles[0];

    if (nextProfile) {
      applyProfile(nextProfile);
      return;
    }

    selectedProfileId = state.active_profile || "default";
    creatingProfile = false;
    draftProfileId = state.active_profile || "default";
    provider = "gemini";
    defaultModel = "gemini-2.5-flash";
    apiKey = "";
    baseUrl = "";
    clearModelCatalog();
    clearTestResult();
  }

  async function loadProfiles() {
    try {
      const state = await invoke<LlmProfilesState>("get_llm_profiles");
      syncProfilesState(state);

      const currentProfile = state.profiles.find((profile) => profile.profile_id === state.active_profile);
      if (currentProfile?.api_key.trim()) {
        void loadProviderModels(false);
      }
    } catch (error) {
      setSettingsStatus(formatAppError("loading LLM settings", error));
    }
  }

  async function loadProviderModels(showSuccess = true) {
    loadingModels = true;
    modelsStatus = "";

    try {
      const models = await invoke<LlmProviderModel[]>("list_llm_provider_models", {
        provider,
        profileId: creatingProfile ? null : selectedProfileId || activeProfile || "default",
        apiKey: apiKey.trim() ? apiKey : null,
        baseUrl: providerSupportsBaseUrl() && baseUrl.trim() ? baseUrl : null,
      });

      availableModels = models;
      if (showSuccess) {
        modelsStatus = `Loaded ${models.length} ${providerLabel()} models.`;
      }
    } catch (error) {
      modelsStatus = formatAppError(`loading ${providerLabel()} models`, error);
    } finally {
      loadingModels = false;
    }
  }

  function beginNewProfile() {
    creatingProfile = true;
    draftProfileId = "";
    clearModelCatalog();
    clearTestResult();
    if (providerSupportsBaseUrl() && !baseUrl.trim()) {
      baseUrl = providerBaseUrlPlaceholder();
    }
  }

  function handleProfileSelectionChange() {
    if (selectedProfileId === NEW_PROFILE_OPTION) {
      beginNewProfile();
      return;
    }

    const profile = profiles.find((candidate) => candidate.profile_id === selectedProfileId);
    if (profile) {
      applyProfile(profile);
    }
  }

  function handleProviderChange() {
    clearModelCatalog();

    if (provider === "gemini") {
      baseUrl = "";
      if (defaultModel.startsWith("if/")) {
        defaultModel = "gemini-2.5-flash";
      }
      return;
    }

    if (!baseUrl.trim()) {
      baseUrl = providerBaseUrlPlaceholder(provider);
    }
    if (defaultModel.startsWith("gemini-")) {
      defaultModel = "";
    }
  }

  function formatTokenLimit(value: number | null) {
    if (value === null) return "";
    return value.toLocaleString();
  }

  function providerModelLine() {
    const profileId = normalizeProfileId(effectiveDraftProfileId());
    return [profileId, provider, defaultModel].filter(Boolean).join(" / ");
  }

  async function saveProfile(setActive: boolean, successMessage?: string) {
    saving = true;
    settingsStatus = "";

    const targetProfileId = normalizeProfileId(effectiveDraftProfileId());
    const wasCreatingProfile = creatingProfile;

    try {
      const state = await invoke<LlmProfilesState>("save_llm_profile", {
        profileId: targetProfileId,
        provider,
        defaultModel,
        apiKey,
        baseUrl: providerSupportsBaseUrl() ? baseUrl : null,
        setActive,
      });

      syncProfilesState(state, targetProfileId);
      setSettingsStatus(
        successMessage ??
          (setActive
            ? `Saved and activated profile '${targetProfileId}'.`
            : wasCreatingProfile
              ? `Created profile '${targetProfileId}'.`
              : `Saved profile '${targetProfileId}'.`)
      );
      return targetProfileId;
    } catch (error) {
      setSettingsStatus(formatAppError("saving LLM settings", error));
      return null;
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

    const savedProfileId = await saveProfile(
      false,
      "Profile settings saved before the provider test run."
    );
    if (!savedProfileId) return;

    testOutput = "";
    testUsage = null;
    testStatus = "";
    testing = true;
    activeRequestId = newRequestId();

    try {
      await invoke("ask_llm_stream", {
        requestId: activeRequestId,
        profileId: savedProfileId,
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
      testStatus = formatAppError("starting the provider test", error);
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

  function openTestDialog() {
    testDialogOpen = true;
  }

  function closeTestDialog() {
    testDialogOpen = false;
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
        testStatus = `Response completed from ${payload.provider}/${payload.model}.`;
        return;
      }

      if (payload.kind === "failed") {
        testing = false;
        testStatus = `Provider test failed: ${payload.error ?? "Unknown provider error"}`;
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
  <h3>LLM Profiles</h3>
  <p class="hint">
    Manage reusable provider profiles for analysis and chat flows. The active profile is used by
    default when a workflow does not pick a profile explicitly. API keys are still stored in local
    SQLite until phase 4.2 replaces this with secure storage.
  </p>

  <div class="grid">
    <label>Saved profile
      <select bind:value={selectedProfileId} onchange={handleProfileSelectionChange}>
        {#each profiles as profile (profile.profile_id)}
          <option value={profile.profile_id}>
            {profile.profile_id} - {providerLabel(profile.provider)}
          </option>
        {/each}
        <option value={NEW_PROFILE_OPTION}>Create new profile...</option>
      </select>
    </label>

    <label>Active profile
      <input type="text" bind:value={activeProfile} disabled />
    </label>
  </div>

  <div class="profile-strip">
    <span class="summary-chip">
      {creatingProfile ? "Creating new profile" : `Editing ${selectedProfileId}`}
    </span>
    {#if !creatingProfile && selectedProfileId === activeProfile}
      <span class="summary-chip active-chip">Used by default in analysis</span>
    {/if}
  </div>

  <div class="grid">
    <label>Profile ID
      <input
        type="text"
        bind:value={draftProfileId}
        placeholder="default"
        disabled={!creatingProfile}
        spellcheck={false}
      />
      <span class="field-hint">Stored in lowercase and used in analysis run metadata.</span>
    </label>

    <label>Provider
      <select bind:value={provider} onchange={handleProviderChange}>
        {#each providerOptions as option (option.value)}
          <option value={option.value}>{option.label}</option>
        {/each}
      </select>
    </label>
  </div>

  <label>Default model
    {#if availableModels.length > 0}
      <select bind:value={defaultModel}>
        {#if !availableModels.some((model) => model.model === defaultModel)}
          <option value={defaultModel}>{defaultModel}</option>
        {/if}
        {#each availableModels as model (model.model)}
          <option value={model.model}>{model.display_name} - {model.model}</option>
        {/each}
      </select>
    {:else}
      <input type="text" bind:value={defaultModel} placeholder={providerPlaceholder()} />
    {/if}
  </label>

  {#if providerSupportsBaseUrl()}
    <label>Base URL
      <input type="url" bind:value={baseUrl} placeholder={providerBaseUrlPlaceholder()} spellcheck={false} />
      <span class="field-hint">
        Use this for OmniRoute or any other OpenAI-compatible endpoint exposed through the same
        backend path.
      </span>
    </label>
  {/if}

  <label>API key
    <input type="password" bind:value={apiKey} placeholder={apiKeyPlaceholder()} autocomplete="off" />
  </label>

  <div class="actions">
    <button onclick={() => saveProfile(true)} disabled={saving || !canSaveProfile()}>
      {saving ? "Saving..." : "Save and set active"}
    </button>
    <button class="secondary" onclick={() => saveProfile(false)} disabled={saving || !canSaveProfile()}>
      Save only
    </button>
    <button class="secondary" onclick={() => loadProviderModels()} disabled={loadingModels || !apiKey.trim()}>
      {loadingModels ? "Loading models..." : "Refresh models"}
    </button>
  </div>

  {#if modelsStatus}
    <p class="status compact" class:error={modelsStatus.startsWith("Error")}>{modelsStatus}</p>
  {/if}

  {#if availableModels.length > 0}
    <div class="model-list">
      {#each availableModels as model (model.model)}
        <button
          class:active={model.model === defaultModel}
          type="button"
          onclick={() => (defaultModel = model.model)}
          title={model.description}
        >
          <span class="model-name">{model.display_name}</span>
          <span class="model-id">{model.model}</span>
          {#if model.input_token_limit !== null || model.output_token_limit !== null}
            <span class="model-limits">
              {#if model.input_token_limit !== null}
                in {formatTokenLimit(model.input_token_limit)}
              {/if}
              {#if model.output_token_limit !== null}
                out {formatTokenLimit(model.output_token_limit)}
              {/if}
            </span>
          {/if}
        </button>
      {/each}
    </div>
  {/if}
</div>

<div class="card">
  <h3>Test Provider</h3>
  <p class="hint">
    Run a smoke test with the profile currently open in this form. The test always saves the form
    first, then uses that saved provider, model, key, and base URL.
  </p>
  <div class="test-summary">
    <div class="summary-copy">
      <span class="summary-label">Prompt draft</span>
      <p>{testPrompt}</p>
    </div>
    <div class="summary-meta">
      {#if provider || defaultModel}
        <span class="summary-chip">{providerModelLine()}</span>
      {/if}
      {#if testUsage}
        <span class="summary-chip">{usageLine(testUsage)}</span>
      {/if}
      <button class="secondary" onclick={openTestDialog}>
        {testOutput || testing ? "Open test console" : "Open test"}
      </button>
    </div>
  </div>

  {#if testStatus}
    <p class="status" class:error={testStatus.startsWith("Provider test failed") || testStatus.startsWith("Error")}>
      {testStatus}
    </p>
  {/if}

  <div class="output-card compact">
    <div class="output-header">
      <span class="output-label">Latest response</span>
      {#if testing}
        <span class="output-meta">streaming...</span>
      {/if}
    </div>
    {#if testOutput}
      <pre>{testOutput}</pre>
    {:else}
      <p class="empty">No output yet. Open the test console to run a prompt.</p>
    {/if}
  </div>
</div>

<DesktopDialog
  open={testDialogOpen}
  title="Provider Test Console"
  description="Run a live request with the profile currently open in settings before using it in reports."
  labelledBy="provider-test-title"
  width="52rem"
  onClose={closeTestDialog}
>
  <div class="test-dialog">
    <label>Prompt
      <textarea bind:value={testPrompt} rows="8" placeholder={`Ask ${providerLabel()} something simple...`}></textarea>
    </label>

    <div class="actions modal-actions">
      <button onclick={runTest} disabled={testing || !testPrompt.trim() || !canSaveProfile()}>
        {testing ? "Streaming..." : "Run test"}
      </button>
      {#if provider || defaultModel}
        <span class="dialog-meta">{providerModelLine()}</span>
      {/if}
    </div>

    <div class="output-card">
      <div class="output-header">
        <span class="output-label">Streaming output</span>
        {#if testUsage}
          <span class="output-meta">{usageLine(testUsage)}</span>
        {/if}
      </div>
      {#if testOutput}
        <pre>{testOutput}</pre>
      {:else}
        <p class="empty">No output yet.</p>
      {/if}
    </div>
  </div>
</DesktopDialog>

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

  input,
  textarea,
  select {
    width: 100%;
    background: var(--panel-strong);
    border: 1px solid var(--border);
    color: var(--text);
    padding: 0.8rem;
    border-radius: 8px;
    font: inherit;
  }

  textarea {
    resize: vertical;
    min-height: 8rem;
  }

  input:focus,
  textarea:focus,
  select:focus {
    border-color: var(--primary);
    outline: none;
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--primary) 18%, transparent);
  }

  .field-hint {
    font-size: 0.78rem;
    color: var(--muted);
    line-height: 1.45;
  }

  .profile-strip {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
    align-items: center;
  }

  .actions {
    display: flex;
    gap: 0.6rem;
    flex-wrap: wrap;
  }

  .model-list {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(16rem, 1fr));
    gap: 0.65rem;
    max-height: 22rem;
    overflow: auto;
    padding: 0.15rem;
  }

  .model-list button {
    align-items: flex-start;
    background: var(--panel-strong);
    border: 1px solid var(--border);
    border-radius: 8px;
    color: var(--text);
    display: flex;
    flex-direction: column;
    gap: 0.28rem;
    min-height: 6.5rem;
    padding: 0.85rem;
    text-align: left;
  }

  .model-list button:hover,
  .model-list button.active {
    border-color: var(--primary);
    background: var(--panel-hover);
  }

  .model-name {
    font-size: 0.95rem;
    font-weight: 600;
    line-height: 1.25;
  }

  .model-id,
  .model-limits {
    color: var(--muted);
    font-size: 0.8rem;
    line-height: 1.35;
    overflow-wrap: anywhere;
  }

  .test-summary {
    display: flex;
    justify-content: space-between;
    gap: 1rem;
    align-items: flex-start;
    padding: 0.95rem 1rem;
    border: 1px solid var(--border);
    border-radius: 10px;
    background: var(--panel-strong);
  }

  .summary-copy {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    min-width: 0;
  }

  .summary-label {
    font-size: 0.78rem;
    font-weight: 600;
    letter-spacing: 0.02em;
    text-transform: uppercase;
    color: var(--muted);
  }

  .summary-copy p {
    margin: 0;
    color: var(--text);
    line-height: 1.5;
    white-space: pre-wrap;
  }

  .summary-meta {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    justify-content: flex-end;
    flex-wrap: wrap;
  }

  .summary-chip,
  .dialog-meta {
    color: var(--muted);
    font-size: 0.82rem;
    padding: 0.25rem 0.55rem;
    border-radius: 999px;
    background: color-mix(in srgb, var(--panel-hover) 80%, transparent);
  }

  .active-chip {
    color: var(--text);
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

  .status.compact {
    margin-bottom: 0;
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

  .output-card.compact {
    min-height: 10rem;
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

  .test-dialog {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .modal-actions {
    align-items: center;
    justify-content: space-between;
  }

  @media (max-width: 720px) {
    .grid {
      grid-template-columns: 1fr;
    }

    .test-summary,
    .modal-actions {
      flex-direction: column;
      align-items: stretch;
    }
  }
</style>
