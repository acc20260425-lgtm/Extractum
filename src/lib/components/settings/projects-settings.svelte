<script lang="ts">
  import { onMount } from "svelte";
  import {
    Plus,
    Bot,
    Edit2,
    Trash2,
    Check,
    Save,
    X,
    RefreshCw,
    AlertTriangle,
    Key,
    Shield,
    Video,
    Send
  } from "@lucide/svelte";
  import {
    getLlmProfiles,
    saveLlmProfile,
    deleteLlmProfile,
    setActiveLlmProfile,
    listLlmProviderModels
  } from "$lib/api/llm";
  import { getSyncSettings, saveSyncSettings } from "$lib/api/sources";
  import { formatAppError } from "$lib/app-error";
  import GeminiBrowserProviderPanel from "$lib/components/settings/gemini-browser-provider-panel.svelte";
  import YoutubeSettingsPanel from "$lib/components/settings/youtube-settings-panel.svelte";
  import type { LlmProfile, LlmProfilesState, LlmProviderModel } from "$lib/types/llm";
  import type { SyncSettings } from "$lib/types/sources";

  // Core Svelte 5 states
  let activeTab = $state("llm");
  let loading = $state(false);
  let profilesState = $state<LlmProfilesState>({ active_profile: "default", profiles: [] });
  let syncSettingsState = $state<SyncSettings>({
    initialSyncMode: "recent_messages",
    initialSyncValue: 100,
  });

  // LLM Modal Editor state
  let dialogOpen = $state(false);
  let isEditing = $state(false);
  let dialogError = $state("");
  let loadingModels = $state(false);
  let availableModels = $state<LlmProviderModel[]>([]);

  // LLM Form drafts
  let formProfileId = $state("");
  let formProvider = $state("gemini");
  let formDefaultModel = $state("gemini-2.5-flash");
  let formApiKey = $state("");
  let formBaseUrl = $state("");
  let formApiKeyConfigured = $state(false);

  // Status banners
  let statusMessage = $state("");
  let statusTone = $state<"default" | "error" | "success">("default");
  let saveSyncLoading = $state(false);

  // Constants
  const providers = [
    { value: "gemini", label: "Gemini" },
    { value: "openai_compatible", label: "OpenAI-compatible" }
  ];

  async function loadSettings() {
    loading = true;
    try {
      const [profiles, sync] = await Promise.all([
        getLlmProfiles(),
        getSyncSettings()
      ]);
      profilesState = profiles;
      syncSettingsState = sync;
    } catch (e) {
      showStatus(formatAppError("loading settings", e), "error");
    } finally {
      loading = false;
    }
  }

  function showStatus(msg: string, tone: "default" | "error" | "success" = "default") {
    statusMessage = msg;
    statusTone = tone;
    setTimeout(() => {
      if (statusMessage === msg) {
        statusMessage = "";
      }
    }, 5000);
  }

  // Active profile handler
  async function makeProfileActive(profileId: string) {
    try {
      const state = await setActiveLlmProfile(profileId);
      profilesState = state;
      showStatus(`Profile '${profileId}' is now active.`, "success");
    } catch (e) {
      showStatus(formatAppError("activating profile", e), "error");
    }
  }

  // Delete profile handler
  async function handleDeleteProfile(profileId: string) {
    if (profileId === "default") {
      showStatus("The default profile cannot be deleted.", "error");
      return;
    }
    if (!confirm(`Are you sure you want to delete profile '${profileId}'?`)) {
      return;
    }
    try {
      const state = await deleteLlmProfile(profileId);
      profilesState = state;
      showStatus(`Profile '${profileId}' deleted successfully.`, "success");
    } catch (e) {
      showStatus(formatAppError("deleting profile", e), "error");
    }
  }

  // Dialog actions
  function openAddDialog() {
    isEditing = false;
    dialogError = "";
    availableModels = [];
    formProfileId = "";
    formProvider = "gemini";
    formDefaultModel = "gemini-2.5-flash";
    formApiKey = "";
    formBaseUrl = "";
    formApiKeyConfigured = false;
    dialogOpen = true;
  }

  function openEditDialog(profile: LlmProfile) {
    isEditing = true;
    dialogError = "";
    availableModels = [];
    formProfileId = profile.profile_id;
    formProvider = profile.provider;
    formDefaultModel = profile.default_model;
    formApiKey = "";
    formBaseUrl = profile.base_url || "";
    formApiKeyConfigured = profile.api_key_configured;
    dialogOpen = true;

    // Prefetch models in background if key is configured
    if (profile.api_key_configured) {
      void fetchModels(false);
    }
  }

  async function fetchModels(showStatusOnSuccess = true) {
    loadingModels = true;
    dialogError = "";
    try {
      const models = await listLlmProviderModels({
        provider: formProvider,
        profileId: isEditing ? formProfileId : null,
        apiKey: formApiKey.trim() ? formApiKey.trim() : null,
        baseUrl: formProvider === "openai_compatible" && formBaseUrl.trim() ? formBaseUrl.trim() : null,
      });

      // Deduplicate models
      const seen = new Set<string>();
      availableModels = models.filter((m) => {
        const key = `${m.model}::${m.display_name}`;
        if (seen.has(key)) return false;
        seen.add(key);
        return true;
      });

      if (showStatusOnSuccess) {
        showStatus(`Loaded ${availableModels.length} models for ${formProvider}.`, "success");
      }
    } catch (e) {
      dialogError = formatAppError("fetching provider models", e);
    } finally {
      loadingModels = false;
    }
  }

  async function handleSaveProfile() {
    dialogError = "";
    const pId = formProfileId.trim().toLowerCase();
    if (!pId) {
      dialogError = "Profile ID is required.";
      return;
    }

    try {
      const state = await saveLlmProfile({
        profileId: pId,
        provider: formProvider,
        defaultModel: formDefaultModel,
        apiKey: formApiKey.trim() ? formApiKey.trim() : null,
        baseUrl: formProvider === "openai_compatible" && formBaseUrl.trim() ? formBaseUrl.trim() : null,
        setActive: !isEditing // Make active by default if creating new
      });

      profilesState = state;
      dialogOpen = false;
      showStatus(`Profile '${pId}' saved successfully.`, "success");
    } catch (e) {
      dialogError = formatAppError("saving profile", e);
    }
  }

  // Telegram sync config actions
  async function handleSaveSyncSettings() {
    saveSyncLoading = true;
    try {
      const updated = await saveSyncSettings(syncSettingsState);
      syncSettingsState = updated;
      showStatus("Telegram sync settings updated successfully.", "success");
    } catch (e) {
      showStatus(formatAppError("saving sync settings", e), "error");
    } finally {
      saveSyncLoading = false;
    }
  }

  onMount(() => {
    void loadSettings();
  });
</script>

<div class="projects-settings-container">
  <!-- Redesigned Top Header -->
  <header class="settings-hero">
    <div class="settings-hero-title">
      <span class="eyebrow">Redesigned Console</span>
      <h1>System Settings</h1>
      <p>Configure model profiles, Telegram synchronization modes, and YouTube caption cookies.</p>
    </div>
    {#if statusMessage}
      <div class="status-banner tone-{statusTone}" role="status">
        {#if statusTone === "error"}
          <AlertTriangle size={16} />
        {:else}
          <Check size={16} />
        {/if}
        <span>{statusMessage}</span>
      </div>
    {/if}
  </header>

  <!-- Tab Bar -->
  <div class="tabs-navigation">
    <button
      class="tab-btn"
      class:active={activeTab === "llm"}
      onclick={() => activeTab = "llm"}
    >
      <Key size={14} />
      <span>LLM Profiles</span>
    </button>
    <button
      class="tab-btn"
      class:active={activeTab === "browser"}
      onclick={() => activeTab = "browser"}
    >
      <Bot size={14} />
      <span>Browser Providers</span>
    </button>
    <button
      class="tab-btn"
      class:active={activeTab === "telegram"}
      onclick={() => activeTab = "telegram"}
    >
      <Send size={14} />
      <span>Telegram Sync</span>
    </button>
    <button
      class="tab-btn"
      class:active={activeTab === "youtube"}
      onclick={() => activeTab = "youtube"}
    >
      <Video size={14} />
      <span>YouTube Sync</span>
    </button>
  </div>

  <!-- Tab Content -->
  <div class="tab-viewport">
    {#if activeTab === "llm"}
      <!-- LLM TAB -->
      <div class="settings-card">
        <div class="card-header">
          <div class="card-header-copy">
            <h2>LLM Provider Profiles</h2>
            <p>Manage API credentials and default models. Stored keys are securely hidden in the system keychain.</p>
          </div>
          <button class="action-btn primary" onclick={openAddDialog}>
            <Plus size={16} />
            <span>Add Profile</span>
          </button>
        </div>

        <div class="table-wrapper">
          <table class="profiles-table">
            <thead>
              <tr>
                <th>Profile ID</th>
                <th>Provider</th>
                <th>Default Model</th>
                <th>Status</th>
                <th class="actions-col">Actions</th>
              </tr>
            </thead>
            <tbody>
              {#each profilesState.profiles as profile}
                <tr class:active-row={profilesState.active_profile === profile.profile_id}>
                  <td>
                    <div class="profile-identity">
                      <strong>{profile.profile_id}</strong>
                      {#if profilesState.active_profile === profile.profile_id}
                        <span class="active-badge">Active</span>
                      {/if}
                    </div>
                  </td>
                  <td>
                    <span class="provider-pill {profile.provider}">
                      {profile.provider === "openai_compatible" ? "OpenAI-comp" : "Gemini"}
                    </span>
                  </td>
                  <td class="model-cell" title={profile.default_model}>
                    <code>{profile.default_model}</code>
                  </td>
                  <td>
                    <div class="status-indicator">
                      <div class="dot" class:configured={profile.api_key_configured}></div>
                      <span>{profile.api_key_configured ? "Key Configured" : "No Key"}</span>
                    </div>
                  </td>
                  <td>
                    <div class="table-actions">
                      {#if profilesState.active_profile !== profile.profile_id}
                        <button
                          class="table-btn success-link"
                          onclick={() => makeProfileActive(profile.profile_id)}
                          title="Set as Active Profile"
                          aria-label={`Set profile ${profile.profile_id} as active`}
                        >
                          Set Active
                        </button>
                      {/if}
                      <button
                        class="table-btn icon-only"
                        onclick={() => openEditDialog(profile)}
                        aria-label={`Edit profile ${profile.profile_id}`}
                        title="Edit Profile"
                      >
                        <Edit2 size={13} />
                      </button>
                      <button
                        class="table-btn icon-only destructive"
                        onclick={() => handleDeleteProfile(profile.profile_id)}
                        disabled={profile.profile_id === "default"}
                        aria-label={`Delete profile ${profile.profile_id}`}
                        title="Delete Profile"
                      >
                        <Trash2 size={13} />
                      </button>
                    </div>
                  </td>
                </tr>
              {/each}
              {#if profilesState.profiles.length === 0}
                <tr>
                  <td colspan="5" class="empty-row">No LLM profiles defined. Click "Add Profile" to create one.</td>
                </tr>
              {/if}
            </tbody>
          </table>
        </div>
      </div>
    {:else if activeTab === "browser"}
      <div class="settings-card">
        <GeminiBrowserProviderPanel />
      </div>
    {:else if activeTab === "telegram"}
      <!-- TELEGRAM TAB -->
      <div class="settings-card max-w-2xl">
        <div class="card-header">
          <div class="card-header-copy">
            <h2>Telegram Synchronizer</h2>
            <p>Determine the depth limit and scope for sync jobs initialized from Telegram sources.</p>
          </div>
        </div>

        <form class="sync-form" onsubmit={(e) => { e.preventDefault(); handleSaveSyncSettings(); }}>
          <fieldset class="form-group">
            <legend>Initial Sync Mode</legend>
            <p class="field-desc">Choose between pulling a set number of messages or a window of recent days.</p>
            <div class="radio-group">
              <label class="radio-label">
                <input
                  type="radio"
                  name="syncMode"
                  value="recent_messages"
                  checked={syncSettingsState.initialSyncMode === "recent_messages"}
                  onchange={() => syncSettingsState.initialSyncMode = "recent_messages"}
                />
                <span class="custom-radio"></span>
                <div class="radio-text">
                  <strong>Recent Messages</strong>
                  <span>Sync up to a specific number of latest posts</span>
                </div>
              </label>
              <label class="radio-label">
                <input
                  type="radio"
                  name="syncMode"
                  value="recent_days"
                  checked={syncSettingsState.initialSyncMode === "recent_days"}
                  onchange={() => syncSettingsState.initialSyncMode = "recent_days"}
                />
                <span class="custom-radio"></span>
                <div class="radio-text">
                  <strong>Recent Days</strong>
                  <span>Sync files sent within a specified window of days</span>
                </div>
              </label>
            </div>
          </fieldset>

          <div class="form-group">
            <label for="syncValue">
              {syncSettingsState.initialSyncMode === "recent_messages" ? "Message Limit" : "Days Limit"}
            </label>
            <input
              id="syncValue"
              type="number"
              min="1"
              max="10000"
              class="text-input"
              bind:value={syncSettingsState.initialSyncValue}
              required
            />
            <span class="field-hint">
              {syncSettingsState.initialSyncMode === "recent_messages"
                ? "Recommended: 50 to 500 messages."
                : "Recommended: 7 to 30 days."}
            </span>
          </div>

          <div class="form-actions">
            <button type="submit" class="action-btn primary" disabled={saveSyncLoading}>
              <Save size={14} />
              <span>{saveSyncLoading ? "Saving..." : "Save Settings"}</span>
            </button>
          </div>
        </form>
      </div>
    {:else if activeTab === "youtube"}
      <!-- YOUTUBE TAB -->
      <div class="settings-card">
        <YoutubeSettingsPanel embedded={true} />
      </div>
    {/if}
  </div>

  <!-- Pop-up Modal Editor -->
  {#if dialogOpen}
    <div class="modal-overlay" role="dialog" aria-modal="true" aria-labelledby="modal-title">
      <div class="modal-box">
        <header class="modal-header">
          <h3 id="modal-title">{isEditing ? "Edit LLM Profile" : "Create LLM Profile"}</h3>
          <button class="close-btn" onclick={() => dialogOpen = false} aria-label="Close dialog">
            <X size={18} />
          </button>
        </header>

        <div class="modal-body">
          {#if dialogError}
            <div class="dialog-error-banner" role="alert">
              <AlertTriangle size={15} />
              <span>{dialogError}</span>
            </div>
          {/if}

          <div class="dialog-grid">
            <div class="form-group">
              <label for="modal-profile-id">Profile ID</label>
              <input
                id="modal-profile-id"
                type="text"
                class="text-input"
                placeholder="e.g. gemini_flash"
                bind:value={formProfileId}
                disabled={isEditing}
              />
              {#if !isEditing}
                <span class="field-hint">Alphanumeric characters and underscores only. Stored as lowercase.</span>
              {/if}
            </div>

            <div class="form-group">
              <label for="modal-provider">Provider</label>
              <select
                id="modal-provider"
                class="select-input"
                bind:value={formProvider}
                onchange={() => {
                  availableModels = [];
                  if (formProvider === "gemini") {
                    formDefaultModel = "gemini-2.5-flash";
                    formBaseUrl = "";
                  } else {
                    formDefaultModel = "";
                    formBaseUrl = "http://localhost:20128/v1";
                  }
                }}
              >
                {#each providers as p}
                  <option value={p.value}>{p.label}</option>
                {/each}
              </select>
            </div>

            {#if formProvider === "openai_compatible"}
              <div class="form-group col-span-2">
                <label for="modal-base-url">Base URL</label>
                <input
                  id="modal-base-url"
                  type="url"
                  class="text-input"
                  placeholder="http://localhost:20128/v1"
                  bind:value={formBaseUrl}
                />
                <span class="field-hint">Custom endpoint location for local models or proxy routers.</span>
              </div>
            {/if}

            <div class="form-group col-span-2">
              <label for="modal-api-key">API Key</label>
              <div class="input-with-icon">
                <Shield size={14} class="input-icon" />
                <input
                  id="modal-api-key"
                  type="password"
                  class="text-input icon-padded"
                  placeholder={formApiKeyConfigured ? "••••••••••••••••" : "Enter API Key"}
                  bind:value={formApiKey}
                />
              </div>
              {#if formApiKeyConfigured}
                <span class="field-hint success">✓ Saved key is already configured. Leave blank to keep existing key.</span>
              {:else}
                <span class="field-hint">No key currently saved for this profile ID.</span>
              {/if}
            </div>

            <div class="form-group col-span-2">
              <div class="fetch-models-row">
                <label for="modal-default-model">Default Model</label>
                <button
                  type="button"
                  class="action-btn secondary sm"
                  aria-label={`Fetch models for ${formProvider} profile`}
                  onclick={() => fetchModels(true)}
                  disabled={loadingModels}
                >
                  <RefreshCw size={13} class={loadingModels ? "spin" : ""} />
                  <span>{loadingModels ? "Fetching..." : "Fetch Models"}</span>
                </button>
              </div>

              {#if availableModels.length > 0}
                <select
                  id="modal-default-model"
                  class="select-input"
                  bind:value={formDefaultModel}
                >
                  {#if !availableModels.some((m) => m.model === formDefaultModel)}
                    <option value={formDefaultModel}>{formDefaultModel}</option>
                  {/if}
                  {#each availableModels as m}
                    <option value={m.model}>{m.display_name} ({m.model})</option>
                  {/each}
                </select>
              {:else}
                <input
                  id="modal-default-model"
                  type="text"
                  class="text-input"
                  placeholder={formProvider === "gemini" ? "gemini-2.5-flash" : "Enter model identifier"}
                  bind:value={formDefaultModel}
                />
              {/if}
              <span class="field-hint">Select a model or type identifier manually if catalog is not fetched.</span>
            </div>
          </div>
        </div>

        <footer class="modal-footer">
          <button class="action-btn secondary" onclick={() => dialogOpen = false}>Cancel</button>
          <button
            class="action-btn primary"
            onclick={handleSaveProfile}
            disabled={!formProfileId.trim() || !formDefaultModel.trim()}
          >
            <Save size={14} />
            <span>Save Profile</span>
          </button>
        </footer>
      </div>
    </div>
  {/if}
</div>

<style>
  /* Premium Light Theme styling matching project defaults */
  .projects-settings-container {
    display: flex;
    flex-direction: column;
    gap: 20px;
    width: 100%;
    height: 100%;
    box-sizing: border-box;
    padding: 24px;
    background: var(--background);
    color: var(--foreground);
    overflow-y: auto;
  }

  .settings-hero {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 20px;
    border-bottom: 1px solid var(--border);
    padding-bottom: 18px;
  }

  .settings-hero-title h1 {
    font-size: 24px;
    font-weight: 700;
    margin: 4px 0 6px 0;
    letter-spacing: -0.02em;
  }

  .settings-hero-title p {
    font-size: 13.5px;
    color: var(--muted-foreground);
    margin: 0;
  }

  .eyebrow {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--primary);
  }

  .status-banner {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 14px;
    border-radius: var(--radius);
    font-size: 13px;
    font-weight: 500;
    border: 1px solid transparent;
    animation: fadeIn 0.25s ease;
  }

  .status-banner.tone-default {
    background: var(--panel-strong);
    border-color: var(--border);
    color: var(--foreground);
  }

  .status-banner.tone-success {
    background: color-mix(in srgb, var(--extractum-success) 10%, var(--background));
    border-color: color-mix(in srgb, var(--extractum-success) 30%, var(--border));
    color: var(--extractum-success);
  }

  .status-banner.tone-error {
    background: color-mix(in srgb, var(--destructive) 10%, var(--background));
    border-color: color-mix(in srgb, var(--destructive) 30%, var(--border));
    color: var(--destructive);
  }

  /* Navigation tabs */
  .tabs-navigation {
    display: flex;
    gap: 8px;
    border-bottom: 1px solid var(--border);
    padding-bottom: 2px;
  }

  .tab-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    background: none;
    border: none;
    border-bottom: 2px solid transparent;
    padding: 8px 16px;
    font-size: 13.5px;
    font-weight: 500;
    color: var(--muted-foreground);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .tab-btn:hover {
    color: var(--foreground);
  }

  .tab-btn.active {
    color: var(--primary);
    border-bottom-color: var(--primary);
  }

  .tab-viewport {
    min-height: 0;
    flex: 1;
  }

  /* Cards */
  .settings-card {
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 20px;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
  }

  .max-w-2xl {
    max-width: 42rem;
  }

  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 20px;
    margin-bottom: 20px;
  }

  .card-header-copy h2 {
    font-size: 16px;
    font-weight: 600;
    margin: 0 0 4px 0;
  }

  .card-header-copy p {
    font-size: 13px;
    color: var(--muted-foreground);
    margin: 0;
  }

  /* Table styles */
  .table-wrapper {
    overflow-x: auto;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    background: var(--background);
  }

  .profiles-table {
    width: 100%;
    border-collapse: collapse;
    text-align: left;
    font-size: 13px;
  }

  .profiles-table th {
    background: var(--panel-strong);
    font-weight: 600;
    padding: 10px 14px;
    color: var(--muted);
    border-bottom: 1px solid var(--border);
  }

  .profiles-table td {
    padding: 12px 14px;
    border-bottom: 1px solid var(--border);
    vertical-align: middle;
  }

  .profiles-table tr:last-child td {
    border-bottom: none;
  }

  .active-row {
    background: color-mix(in srgb, var(--primary) 4%, transparent);
  }

  .profile-identity {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .active-badge {
    background: color-mix(in srgb, var(--primary) 12%, transparent);
    color: var(--primary);
    font-size: 10px;
    font-weight: 600;
    padding: 2px 6px;
    border-radius: 4px;
    letter-spacing: 0.02em;
  }

  .provider-pill {
    font-size: 11px;
    font-weight: 600;
    padding: 2px 6px;
    border-radius: 4px;
  }

  .provider-pill.gemini {
    background: #e0f2fe;
    color: #0369a1;
  }

  .provider-pill.openai_compatible {
    background: #f0fdf4;
    color: #166534;
  }

  .model-cell code {
    background: var(--panel-strong);
    color: var(--muted);
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 12px;
  }

  .status-indicator {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .status-indicator .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: #94a3b8;
  }

  .status-indicator .dot.configured {
    background: var(--extractum-success);
  }

  .table-actions {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .table-btn {
    background: none;
    border: none;
    cursor: pointer;
    font-size: 12.5px;
    font-weight: 500;
    color: var(--primary);
    padding: 4px 8px;
    border-radius: 4px;
    transition: background-color 0.12s;
  }

  .table-btn:hover {
    background: var(--panel-hover);
  }

  .table-btn.success-link {
    color: var(--extractum-success);
  }

  .table-btn.success-link:hover {
    background: color-mix(in srgb, var(--extractum-success) 8%, transparent);
  }

  .table-btn.icon-only {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    color: var(--muted-foreground);
    padding: 0;
  }

  .table-btn.icon-only:hover {
    color: var(--foreground);
  }

  .table-btn.icon-only.destructive {
    color: var(--destructive);
  }

  .table-btn.icon-only.destructive:hover {
    background: color-mix(in srgb, var(--destructive) 8%, transparent);
  }

  .table-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .empty-row {
    text-align: center;
    color: var(--muted-foreground);
    padding: 30px !important;
  }

  /* Button controls */
  .action-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    font-size: 13px;
    font-weight: 600;
    padding: 8px 14px;
    border-radius: var(--radius);
    cursor: pointer;
    transition: all 0.15s ease;
    border: 1px solid var(--border);
  }

  .action-btn.primary {
    background: var(--primary);
    color: var(--primary-foreground);
    border-color: transparent;
  }

  .action-btn.primary:hover {
    opacity: 0.92;
  }

  .action-btn.secondary {
    background: var(--card);
    color: var(--foreground);
  }

  .action-btn.secondary:hover {
    background: var(--panel-hover);
  }

  .action-btn.sm {
    padding: 4px 10px;
    font-size: 12px;
  }

  .action-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* Form groups */
  .sync-form {
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  fieldset.form-group {
    border: 0;
    margin: 0;
    padding: 0;
  }

  .form-group label,
  .form-group legend {
    font-size: 13.5px;
    font-weight: 600;
  }

  .field-desc {
    font-size: 12.5px;
    color: var(--muted-foreground);
    margin: 0 0 6px 0;
  }

  .field-hint {
    font-size: 11.5px;
    color: var(--muted-foreground);
  }

  .field-hint.success {
    color: var(--extractum-success);
  }

  .radio-group {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .radio-label {
    display: flex;
    align-items: flex-start;
    gap: 12px;
    padding: 12px;
    border: 1px solid var(--border);
    border-radius: var(--radius);
    cursor: pointer;
    background: var(--background);
    transition: border-color 0.15s;
  }

  .radio-label:hover {
    border-color: var(--primary);
  }

  .radio-label input[type="radio"] {
    display: none;
  }

  .custom-radio {
    position: relative;
    width: 16px;
    height: 16px;
    border: 2px solid var(--border);
    border-radius: 50%;
    margin-top: 2px;
    flex-shrink: 0;
  }

  .radio-label input[type="radio"]:checked + .custom-radio {
    border-color: var(--primary);
  }

  .radio-label input[type="radio"]:checked + .custom-radio::after {
    content: "";
    position: absolute;
    top: 3px;
    left: 3px;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--primary);
  }

  .radio-text {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .radio-text strong {
    font-size: 13.5px;
    font-weight: 600;
  }

  .radio-text span {
    font-size: 12px;
    color: var(--muted-foreground);
  }

  .text-input,
  .select-input {
    width: 100%;
    background: var(--background);
    border: 1px solid var(--border);
    color: var(--foreground);
    padding: 8px 12px;
    border-radius: var(--radius);
    font-size: 13.5px;
    font-family: inherit;
    box-sizing: border-box;
  }

  .text-input:focus,
  .select-input:focus {
    outline: none;
    border-color: var(--primary);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--primary) 15%, transparent);
  }

  .form-actions {
    display: flex;
    justify-content: flex-end;
    border-top: 1px solid var(--border);
    padding-top: 16px;
  }

  /* Modal Dialog */
  .modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    background: rgba(15, 23, 42, 0.45);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
    animation: fadeIn 0.15s ease-out;
  }

  .modal-box {
    background: var(--card);
    border: 1px solid var(--border);
    border-radius: 12px;
    width: min(540px, calc(100vw - 32px));
    max-height: min(720px, calc(100vh - 32px));
    display: flex;
    flex-direction: column;
    box-shadow: 0 10px 25px -5px rgba(0, 0, 0, 0.1), 0 8px 10px -6px rgba(0, 0, 0, 0.1);
    animation: scaleUp 0.15s ease-out;
    overflow: hidden;
  }

  .modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 20px;
    border-bottom: 1px solid var(--border);
  }

  .modal-header h3 {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--muted-foreground);
    cursor: pointer;
    padding: 4px;
    border-radius: 4px;
  }

  .close-btn:hover {
    color: var(--foreground);
    background: var(--panel-hover);
  }

  .modal-body {
    padding: 20px;
    overflow-y: auto;
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 16px 20px;
    border-top: 1px solid var(--border);
    background: var(--panel-strong);
  }

  .dialog-error-banner {
    display: flex;
    align-items: center;
    gap: 10px;
    background: color-mix(in srgb, var(--destructive) 8%, var(--card));
    border: 1px solid color-mix(in srgb, var(--destructive) 25%, var(--border));
    border-radius: var(--radius);
    padding: 10px 14px;
    color: var(--destructive);
    font-size: 12.5px;
    font-weight: 500;
    line-height: 1.4;
  }

  .dialog-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 16px;
  }

  .col-span-2 {
    grid-column: span 2 / span 2;
  }

  .input-with-icon {
    position: relative;
    width: 100%;
  }

  .input-icon {
    position: absolute;
    top: 50%;
    left: 12px;
    transform: translateY(-50%);
    color: var(--muted-foreground);
    pointer-events: none;
  }

  .text-input.icon-padded {
    padding-left: 36px;
  }

  .fetch-models-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 4px;
  }

  /* Animations */
  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  @keyframes scaleUp {
    from { transform: scale(0.95); opacity: 0; }
    to { transform: scale(1); opacity: 1; }
  }

  .spin {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
</style>
