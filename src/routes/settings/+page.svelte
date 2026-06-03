<script lang="ts">
  import { Eraser, Play, RefreshCw, Save, Square, Terminal } from "@lucide/svelte";
  import { onMount } from "svelte";
  import {
    askLlmStream,
    cancelLlmRequest,
    clearLlmProfileApiKey,
    getLlmProfiles,
    listLlmProviderModels,
    listenToLlmResponses,
    saveLlmProfile,
  } from "$lib/api/llm";
  import { formatAppError } from "$lib/app-error";
  import DesktopDialog from "$lib/components/desktop-dialog.svelte";
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import MetaPill from "$lib/components/ui/MetaPill.svelte";
  import Select from "$lib/components/ui/Select.svelte";
  import StatusMessage from "$lib/components/ui/StatusMessage.svelte";
  import SurfaceCard from "$lib/components/ui/SurfaceCard.svelte";
  import Textarea from "$lib/components/ui/Textarea.svelte";
  import type { LlmProfile, LlmProfilesState, LlmProviderModel, LlmUsage } from "$lib/types/llm";

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
  let apiKeyConfigured = $state(false);
  let baseUrl = $state("");

  let settingsStatus = $state("");
  let saving = $state(false);
  let availableModels = $state<LlmProviderModel[]>([]);
  let modelQuery = $state("");
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

  function suggestNewProfileId() {
    const existingIds = new Set(profiles.map((profile) => profile.profile_id));
    const base = provider === "omniroute" ? "omniroute_profile" : "gemini_profile";

    if (!existingIds.has(base)) {
      return base;
    }

    let index = 2;
    while (existingIds.has(`${base}_${index}`)) {
      index += 1;
    }

    return `${base}_${index}`;
  }

  function effectiveDraftProfileId() {
    return creatingProfile ? draftProfileId : selectedProfileId;
  }

  function canSaveProfile() {
    return Boolean(defaultModel.trim() && normalizeProfileId(effectiveDraftProfileId()));
  }

  function canRefreshModels() {
    return Boolean(apiKey.trim() || (!creatingProfile && apiKeyConfigured));
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

  const filteredAvailableModels = $derived.by(() => {
    const query = modelQuery.trim().toLowerCase();
    if (!query) return availableModels;
    return availableModels.filter((model) =>
      `${model.display_name} ${model.model}`.toLowerCase().includes(query),
    );
  });

  function dedupeProviderModels(models: LlmProviderModel[]) {
    const unique: LlmProviderModel[] = [];

    for (const model of models) {
      const key = `${model.model}::${model.display_name}`;
      if (!unique.some((entry) => `${entry.model}::${entry.display_name}` === key)) {
        unique.push(model);
      }
    }

    return unique;
  }

  function applyProfile(profile: LlmProfile) {
    selectedProfileId = profile.profile_id;
    creatingProfile = false;
    draftProfileId = profile.profile_id;
    provider = profile.provider;
    defaultModel = profile.default_model;
    apiKey = "";
    apiKeyConfigured = profile.api_key_configured;
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
    apiKeyConfigured = false;
    baseUrl = "";
    clearModelCatalog();
    clearTestResult();
  }

  async function loadProfiles() {
    try {
      const state = await getLlmProfiles();
      syncProfilesState(state);

      const currentProfile = state.profiles.find((profile) => profile.profile_id === state.active_profile);
      if (currentProfile?.api_key_configured) {
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
      const models = await listLlmProviderModels({
        provider,
        profileId: creatingProfile ? null : selectedProfileId || activeProfile || "default",
        apiKey: apiKey.trim() ? apiKey : null,
        baseUrl: providerSupportsBaseUrl() && baseUrl.trim() ? baseUrl : null,
      });

      availableModels = dedupeProviderModels(models);
      if (showSuccess) {
        const duplicateCount = models.length - availableModels.length;
        modelsStatus =
          duplicateCount > 0
            ? `Loaded ${availableModels.length} ${providerLabel()} models (${duplicateCount} duplicates hidden).`
            : `Loaded ${availableModels.length} ${providerLabel()} models.`;
      }
    } catch (error) {
      modelsStatus = formatAppError(`loading ${providerLabel()} models`, error);
    } finally {
      loadingModels = false;
    }
  }

  function beginNewProfile() {
    creatingProfile = true;
    draftProfileId = suggestNewProfileId();
    clearModelCatalog();
    clearTestResult();
    settingsStatus = "Review the generated profile ID, then save the new profile.";
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

  function onSelectedProfileChange(event: Event) {
    selectedProfileId = (event.currentTarget as HTMLSelectElement).value;
    handleProfileSelectionChange();
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

  function onProviderChange(event: Event) {
    provider = (event.currentTarget as HTMLSelectElement).value;
    handleProviderChange();
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
      const state = await saveLlmProfile({
        profileId: targetProfileId,
        provider,
        defaultModel,
        apiKey: apiKey.trim() ? apiKey : null,
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

  async function clearSavedApiKey() {
    if (creatingProfile || !selectedProfileId || !apiKeyConfigured) return;

    saving = true;
    settingsStatus = "";
    try {
      const state = await clearLlmProfileApiKey(selectedProfileId);
      syncProfilesState(state, selectedProfileId);
      apiKey = "";
      setSettingsStatus(`Cleared API key for '${selectedProfileId}'.`);
    } catch (error) {
      setSettingsStatus(formatAppError("clearing the API key", error));
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
      await askLlmStream({
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
      activeRequestId = null;
      testStatus = formatAppError("starting the provider test", error);
    }
  }

  async function cancelTest() {
    if (!activeRequestId) return;

    testStatus = "Cancelling provider test...";
    try {
      await cancelLlmRequest(activeRequestId);
    } catch (error) {
      testing = false;
      activeRequestId = null;
      testStatus = formatAppError("cancelling the provider test", error);
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
    void listenToLlmResponses(({ payload }) => {
      if (disposed || payload.request_id !== activeRequestId) {
        return;
      }

      if (payload.kind === "started") {
        testing = true;
        testUsage = null;
        testStatus = `Streaming response from ${payload.provider}/${payload.model}...`;
        return;
      }

      if (payload.kind === "queued") {
        testing = true;
        testUsage = null;
        testStatus =
          payload.queue_position !== null
            ? `Waiting for an LLM slot from ${payload.provider}/${payload.model} (queue #${payload.queue_position})...`
            : `Waiting for an LLM slot from ${payload.provider}/${payload.model}...`;
        return;
      }

      if (payload.kind === "delta") {
        testOutput += payload.delta ?? "";
        return;
      }

      if (payload.kind === "completed") {
        testing = false;
        activeRequestId = null;
        testOutput = payload.text ?? testOutput;
        testUsage = payload.usage;
        testStatus = `Response completed from ${payload.provider}/${payload.model}.`;
        return;
      }

      if (payload.kind === "failed") {
        testing = false;
        activeRequestId = null;
        testUsage = null;
        testStatus = `Provider test failed: ${payload.error ?? "Unknown provider error"}`;
        return;
      }

      if (payload.kind === "cancelled") {
        testing = false;
        activeRequestId = null;
        testUsage = null;
        testStatus = payload.error ?? "Provider test cancelled.";
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

<section class="page-shell">
  <header class="page-hero">
    <div class="page-hero-copy">
      <span class="page-eyebrow">LLM configuration</span>
      <h1>Settings</h1>
      <p>
        Keep provider profiles, model defaults, and smoke testing in one compact operator surface.
        Dense tools, clear status, no dashboard clutter.
      </p>
    </div>
    <div class="page-hero-meta">
      <Badge variant="info">{profiles.length} profiles</Badge>
      <Badge>{providerLabel()}</Badge>
      {#if !creatingProfile && selectedProfileId === activeProfile}
        <Badge variant="success">Active by default</Badge>
      {/if}
    </div>
  </header>

  {#if settingsStatus}
    <StatusMessage tone={settingsStatus.startsWith("Error") ? "error" : "default"} className="page-status">
      {settingsStatus}
    </StatusMessage>
  {/if}

  <div class="page-grid settings-grid">
    <div class="page-stack">
      <section class="desk-panel">
        <div class="panel-header">
          <div class="panel-header-copy">
            <span class="page-eyebrow">Profiles</span>
            <h2>LLM profiles</h2>
            <p>
              Manage reusable provider profiles for analysis and chat. Saved API keys stay in OS
              secure storage and never load back into this form.
            </p>
          </div>
        </div>

        <div class="grid">
          <label>Saved profile
            <Select value={selectedProfileId} onchange={onSelectedProfileChange}>
              {#each profiles as profile (profile.profile_id)}
                <option value={profile.profile_id}>
                  {profile.profile_id} - {providerLabel(profile.provider)}
                </option>
              {/each}
              <option value={NEW_PROFILE_OPTION}>Create new profile...</option>
            </Select>
          </label>

          <label>Active profile
            <Input type="text" value={activeProfile} disabled />
          </label>
        </div>

        <div class="profile-status-strip">
          <MetaPill>Editing profile: {creatingProfile ? "new profile" : selectedProfileId}</MetaPill>
          <MetaPill tone={selectedProfileId === activeProfile && !creatingProfile ? "active" : "default"}>
            Active profile: {activeProfile || "none"}
          </MetaPill>
          {#if !creatingProfile && selectedProfileId !== activeProfile}
            <MetaPill>Set active after save</MetaPill>
          {/if}
        </div>

        <div class="desk-divider"></div>

        <div class="grid">
          <label>Profile ID
            <Input
              type="text"
              value={draftProfileId}
              placeholder="default"
              disabled={!creatingProfile}
              spellcheck={false}
              oninput={(event) => (draftProfileId = (event.currentTarget as HTMLInputElement).value)}
            />
            <span class="field-hint">Stored in lowercase and written into analysis run metadata.</span>
          </label>

          <label>Provider
            <Select value={provider} onchange={onProviderChange}>
              {#each providerOptions as option (option.value)}
                <option value={option.value}>{option.label}</option>
              {/each}
            </Select>
          </label>
        </div>

        <label>Default model
          {#if availableModels.length > 0}
            <Input
              type="search"
              value={modelQuery}
              placeholder="Search models"
              ariaLabel="Search models"
              oninput={(event) => (modelQuery = (event.currentTarget as HTMLInputElement).value)}
            />
            <Select
              value={defaultModel}
              onchange={(event) => (defaultModel = (event.currentTarget as HTMLSelectElement).value)}
            >
              {#if !availableModels.some((model) => model.model === defaultModel)}
                <option value={defaultModel}>{defaultModel}</option>
              {/if}
              {#each filteredAvailableModels as model (model.model)}
                <option value={model.model}>{model.display_name} - {model.model}</option>
              {/each}
            </Select>
          {:else}
            <Input
              type="text"
              value={defaultModel}
              placeholder={providerPlaceholder()}
              oninput={(event) => (defaultModel = (event.currentTarget as HTMLInputElement).value)}
            />
          {/if}
        </label>

        {#if providerSupportsBaseUrl()}
          <label>Base URL
            <Input
              type="url"
              value={baseUrl}
              placeholder={providerBaseUrlPlaceholder()}
              spellcheck={false}
              oninput={(event) => (baseUrl = (event.currentTarget as HTMLInputElement).value)}
            />
            <span class="field-hint">
              Use this for OmniRoute or any other OpenAI-compatible endpoint exposed through the same
              backend path.
            </span>
          </label>
        {/if}

        <label>API key
          <Input
            type="password"
            value={apiKey}
            placeholder={apiKeyPlaceholder()}
            autocomplete="off"
            oninput={(event) => (apiKey = (event.currentTarget as HTMLInputElement).value)}
          />
          <span class="field-hint">
            {apiKeyConfigured ? "Saved key configured. Leave blank to keep it." : "No saved key configured."}
          </span>
        </label>

        <div class="actions">
          <Button onclick={() => saveProfile(true)} disabled={saving || !canSaveProfile()}>
            <Save size={15} aria-hidden="true" />
            {saving ? "Saving..." : "Save and set active"}
          </Button>
          <Button variant="secondary" onclick={() => saveProfile(false)} disabled={saving || !canSaveProfile()}>
            <Save size={15} aria-hidden="true" />
            Save only
          </Button>
          <Button
            variant="secondary"
            onclick={() => loadProviderModels()}
            disabled={loadingModels || !canRefreshModels()}
          >
            <RefreshCw size={15} aria-hidden="true" />
            {loadingModels ? "Loading models..." : "Refresh models"}
          </Button>
          <Button
            variant="danger-soft"
            type="button"
            onclick={clearSavedApiKey}
            disabled={saving || creatingProfile || !apiKeyConfigured}
          >
            <Eraser size={15} aria-hidden="true" />
            Clear API key
          </Button>
          <Button variant="secondary" type="button" onclick={openTestDialog}>
            <Terminal size={15} aria-hidden="true" />
            Open test
          </Button>
        </div>

        {#if modelsStatus}
          <StatusMessage
            tone={modelsStatus.startsWith("Error") ? "error" : "default"}
            className="compact-status"
          >
            {modelsStatus}
          </StatusMessage>
        {/if}

        {#if availableModels.length > 0}
          <div class="model-list">
            {#each filteredAvailableModels as model (model.model)}
              <Button
                variant="secondary"
                selected={model.model === defaultModel}
                className="catalog-option"
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
              </Button>
            {/each}
          </div>
        {/if}
      </section>
    </div>

    <div class="page-stack">
      <section class="desk-panel desk-panel-subtle provider-notes">
        <div class="panel-header-copy">
          <span class="page-eyebrow">Operator note</span>
          <h3>Settings now follow the workspace pattern</h3>
          <p>
            Settings stay focused on LLM provider profiles and test runs.
          </p>
        </div>
      </section>
    </div>
  </div>
</section>

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
      <Textarea
        value={testPrompt}
        rows={8}
        placeholder={`Ask ${providerLabel()} something simple...`}
        oninput={(event) => (testPrompt = (event.currentTarget as HTMLTextAreaElement).value)}
      />
    </label>

    <div class="actions modal-actions">
      <Button onclick={runTest} disabled={testing || !testPrompt.trim() || !canSaveProfile()}>
        <Play size={15} aria-hidden="true" />
        {testing ? "Streaming..." : "Run test"}
      </Button>
      {#if testing}
        <Button variant="danger-soft" type="button" onclick={cancelTest}>
          <Square size={15} aria-hidden="true" /> Cancel
        </Button>
      {/if}
      {#if provider || defaultModel}
        <MetaPill>{providerModelLine()}</MetaPill>
      {/if}
    </div>

    {#if testStatus}
      <StatusMessage
        tone={testStatus.startsWith("Provider test failed") || testStatus.startsWith("Error") ? "error" : "default"}
      >
        {testStatus}
      </StatusMessage>
    {/if}

    <SurfaceCard
      title="Streaming output"
      meta={testUsage ? usageLine(testUsage) : ""}
      className="output-surface"
    >
      {#if testOutput}
        <pre>{testOutput}</pre>
      {:else}
        <EmptyState description="No output yet." />
      {/if}
    </SurfaceCard>
  </div>
</DesktopDialog>

<style>
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

  .field-hint {
    font-size: 0.78rem;
    color: var(--muted);
    line-height: 1.45;
  }

  .profile-status-strip {
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

  :global(.ui-button.catalog-option) {
    align-items: flex-start;
    display: flex;
    flex-direction: column;
    gap: 0.28rem;
    height: 100%;
    justify-content: flex-start;
    min-height: 6.5rem;
    padding: 0.85rem;
    text-align: left;
    white-space: normal;
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

  :global(.page-status) {
    margin-bottom: 0;
  }

  :global(.compact-status) {
    margin-bottom: 0;
  }

  :global(.ui-surface-card.output-surface) {
    min-height: 14rem;
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

  .provider-notes h3 {
    margin: 0;
  }

  @media (max-width: 720px) {
    .grid {
      grid-template-columns: 1fr;
    }

    .modal-actions {
      flex-direction: column;
      align-items: stretch;
    }
  }
</style>
