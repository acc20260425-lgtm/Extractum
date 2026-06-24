<script lang="ts">
  import { Cookie, RefreshCw, Save, Trash2, X } from "@lucide/svelte";
  import { onMount } from "svelte";
  import {
    clearYoutubeAuth,
    getYoutubeAuthStatus,
    getYoutubeSettings,
    saveYoutubeCookies,
    saveYoutubeSettings,
  } from "$lib/api/youtube-settings";
  import { formatAppError } from "$lib/app-error";
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import CheckboxRow from "$lib/components/ui/CheckboxRow.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import StatusMessage from "$lib/components/ui/StatusMessage.svelte";
  import type { YoutubeAuthStatus, YoutubeSettings } from "$lib/types/youtube";

  type NumberSettingKey =
    | "delayBetweenRequestsMs"
    | "maxParallelVideoSyncs"
    | "maxParallelCommentSyncs"
    | "dailySoftLimit"
    | "retryBackoffMs"
    | "stopAfterConsecutiveFailures";

  let { embedded = false } = $props();

  let settings = $state<YoutubeSettings | null>(null);
  let draft = $state<YoutubeSettings>(defaultSettings());
  let authStatus = $state<YoutubeAuthStatus | null>(null);
  let cookieText = $state("");
  let editingCookies = $state(false);
  let dragover = $state(false);
  let loading = $state(true);
  let savingSettings = $state(false);
  let savingCookies = $state(false);
  let clearingAuth = $state(false);
  let panelStatus = $state("");

  const statusTone = $derived(panelStatus.startsWith("Error") ? "error" : "default");
  const authBadgeVariant = $derived(
    authStatus?.enabled && authStatus.hasCookies
      ? "success"
      : authStatus?.enabled
        ? "warning"
        : "neutral",
  );
  const canSaveCookies = $derived(Boolean(cookieText.trim()) && !savingCookies);

  function defaultSettings(): YoutubeSettings {
    return {
      authEnabled: false,
      preferredCaptionsLanguage: "original",
      delayBetweenRequestsMs: 1000,
      maxParallelVideoSyncs: 1,
      maxParallelCommentSyncs: 1,
      pauseOnAuthChallenge: true,
      dailySoftLimit: 0,
      retryBackoffMs: 3000,
      stopAfterConsecutiveFailures: 3,
    };
  }

  async function loadPanel() {
    loading = true;
    panelStatus = "";
    try {
      const [loadedSettings, loadedAuthStatus] = await Promise.all([
        getYoutubeSettings(),
        getYoutubeAuthStatus(),
      ]);
      settings = loadedSettings;
      draft = { ...loadedSettings };
      authStatus = loadedAuthStatus;
    } catch (error) {
      panelStatus = formatAppError("loading YouTube settings", error);
    } finally {
      loading = false;
    }
  }

  async function reloadAuthStatus() {
    authStatus = await getYoutubeAuthStatus();
  }

  function updateBoolean(key: "authEnabled" | "pauseOnAuthChallenge", event: Event) {
    draft = {
      ...draft,
      [key]: (event.currentTarget as HTMLInputElement).checked,
    };
  }

  function updateNumber(key: NumberSettingKey, event: Event) {
    const value = Number((event.currentTarget as HTMLInputElement).value);
    draft = {
      ...draft,
      [key]: Number.isFinite(value) ? value : 0,
    };
  }

  async function saveSettings() {
    savingSettings = true;
    panelStatus = "";
    try {
      const saved = await saveYoutubeSettings(draft);
      settings = saved;
      draft = { ...saved };
      await reloadAuthStatus();
      panelStatus = "YouTube settings saved.";
    } catch (error) {
      panelStatus = formatAppError("saving YouTube settings", error);
    } finally {
      savingSettings = false;
    }
  }

  async function saveCookies() {
    if (!cookieText.trim()) return;

    savingCookies = true;
    panelStatus = "";
    try {
      await saveYoutubeCookies(cookieText);
      cookieText = "";
      editingCookies = false;
      await reloadAuthStatus();
      draft = { ...draft, authEnabled: true };
      settings = settings ? { ...settings, authEnabled: true } : settings;
      panelStatus = authStatus?.message ?? "Cookies stored";
    } catch (error) {
      panelStatus = formatAppError("saving YouTube cookies", error);
    } finally {
      savingCookies = false;
    }
  }

  async function clearAuth() {
    clearingAuth = true;
    panelStatus = "";
    try {
      const cleared = await clearYoutubeAuth();
      authStatus = cleared;
      cookieText = "";
      editingCookies = false;
      draft = { ...draft, authEnabled: false };
      settings = settings ? { ...settings, authEnabled: false } : settings;
      panelStatus = cleared.message;
    } catch (error) {
      panelStatus = formatAppError("clearing YouTube auth", error);
    } finally {
      clearingAuth = false;
    }
  }

  function startCookieEdit() {
    cookieText = "";
    editingCookies = true;
  }

  function cancelCookieEdit() {
    cookieText = "";
    editingCookies = false;
  }

  function handleDragOver(event: DragEvent) {
    event.preventDefault();
    dragover = true;
  }

  function handleDragLeave() {
    dragover = false;
  }

  function handleDrop(event: DragEvent) {
    event.preventDefault();
    dragover = false;
    const files = event.dataTransfer?.files;
    if (files && files.length > 0) {
      void processFile(files[0]);
    }
  }

  function handleFileSelect(event: Event) {
    const files = (event.currentTarget as HTMLInputElement).files;
    if (files && files.length > 0) {
      void processFile(files[0]);
    }
  }

  async function processFile(file: File) {
    try {
      const text = await file.text();
      cookieText = text;
      editingCookies = true;
      panelStatus = `Loaded cookie file: ${file.name}. Review and click Save to store.`;
    } catch (error) {
      panelStatus = formatAppError("reading cookie file", error);
    }
  }

  onMount(() => {
    void loadPanel();
  });
</script>

<section class={`youtube-settings-panel ${embedded ? "embedded" : "desk-panel desk-panel-subtle"}`.trim()}>
  {#if !embedded}
    <div class="panel-header">
      <div class="panel-header-copy">
        <span class="page-eyebrow">YouTube</span>
        <h2>Auth and sync</h2>
      </div>
      <div class="status-strip">
        <Badge variant={authBadgeVariant}>{authStatus?.message ?? "Loading"}</Badge>
      </div>
    </div>
  {:else}
    <div class="status-strip embedded-status">
      <Badge variant={authBadgeVariant}>{authStatus?.message ?? "Loading"}</Badge>
    </div>
  {/if}

  {#if panelStatus}
    <StatusMessage tone={statusTone}>{panelStatus}</StatusMessage>
  {/if}

  <section class="youtube-auth-section" aria-label="YouTube authentication">
    <h3>Authentication</h3>
    <CheckboxRow
      title="Enable YouTube auth"
      description={authStatus?.hasCookies ? "Cookies stored" : "No cookies configured"}
      checked={draft.authEnabled}
      disabled={loading || savingSettings}
      onchange={(event) => updateBoolean("authEnabled", event)}
    />

    <div class="cookie-box">
      <div class="cookie-box-header">
        <div>
          <strong>{authStatus?.message ?? "Auth status"}</strong>
          <span>{authStatus?.hasCookies ? "Stored cookie text is hidden." : "No cookies saved."}</span>
        </div>
        <div class="cookie-actions">
          <Button
            variant="secondary"
            size="sm"
            title={authStatus?.hasCookies ? "Update YouTube cookies" : "Paste or update YouTube cookies"}
            ariaLabel={authStatus?.hasCookies ? "Update YouTube cookies" : "Paste or update YouTube cookies"}
            onclick={startCookieEdit}
            disabled={loading || savingCookies}
          >
            <Cookie size={13} aria-hidden="true" />
            {authStatus?.hasCookies ? "Update cookies" : "Paste/update cookies"}
          </Button>
          <Button
            variant="danger-soft"
            size="sm"
            title="Clear YouTube authentication and remove stored cookies"
            ariaLabel="Clear YouTube authentication and remove stored cookies"
            onclick={clearAuth}
            disabled={loading || clearingAuth || !authStatus?.hasCookies}
          >
            <Trash2 size={13} aria-hidden="true" />
            {clearingAuth ? "Clearing..." : "Clear YouTube auth"}
          </Button>
        </div>
      </div>

      {#if editingCookies}
        <div
          class="cookie-dropzone"
          class:dragover={dragover}
          ondragover={handleDragOver}
          ondragleave={handleDragLeave}
          ondrop={handleDrop}
          role="region"
          aria-label="Cookies file drop zone"
        >
          <div class="dropzone-content">
            <span class="dropzone-text">Drag & drop <code>cookies.txt</code> file here, or</span>
            <input
              type="file"
              accept=".txt"
              id="cookie-file-upload"
              class="hidden-file-input"
              onchange={handleFileSelect}
            />
            <label for="cookie-file-upload" class="file-select-label">browse file</label>
          </div>
        </div>

        <label>Or paste cookies text directly
          <textarea
            bind:value={cookieText}
            rows="8"
            spellcheck="false"
            autocomplete="off"
            autocapitalize="off"
            class="cookie-textarea"
          ></textarea>
        </label>
        <div class="actions">
          <Button
            title="Save YouTube cookies"
            ariaLabel="Save YouTube cookies"
            onclick={saveCookies}
            disabled={!canSaveCookies}
          >
            <Save size={15} aria-hidden="true" />
            {savingCookies ? "Saving..." : "Save cookies"}
          </Button>
          <Button
            variant="secondary"
            title="Cancel cookie editor"
            ariaLabel="Cancel cookie editor"
            onclick={cancelCookieEdit}
            disabled={savingCookies}
          >
            <X size={15} aria-hidden="true" />
            Cancel cookie edit
          </Button>
        </div>
      {/if}
    </div>
  </section>

  <div class="desk-divider"></div>

  <section class="youtube-sync-policy-section" aria-label="YouTube sync policy">
    <h3>Sync policy</h3>
    <div class="settings-fields">
      <label>Preferred captions language
        <Input
          type="text"
          value={draft.preferredCaptionsLanguage}
          placeholder="original"
          disabled={loading || savingSettings}
          spellcheck={false}
          max="32"
          oninput={(event) =>
            (draft = {
              ...draft,
              preferredCaptionsLanguage: (event.currentTarget as HTMLInputElement).value,
            })}
        />
      </label>

      <label>Delay between requests
        <Input
          type="number"
          value={draft.delayBetweenRequestsMs}
          min="0"
          max="60000"
          step="100"
          disabled={loading || savingSettings}
          oninput={(event) => updateNumber("delayBetweenRequestsMs", event)}
        />
        <span class="field-hint">0 means no deliberate delay.</span>
      </label>

      <label>Max parallel video syncs
        <Input
          type="number"
          value={draft.maxParallelVideoSyncs}
          min="1"
          max="4"
          step="1"
          disabled={loading || savingSettings}
          oninput={(event) => updateNumber("maxParallelVideoSyncs", event)}
        />
      </label>

      <label>Max parallel comment syncs
        <Input
          type="number"
          value={draft.maxParallelCommentSyncs}
          min="1"
          max="2"
          step="1"
          disabled={loading || savingSettings}
          oninput={(event) => updateNumber("maxParallelCommentSyncs", event)}
        />
      </label>

      <label>Daily soft limit
        <Input
          type="number"
          value={draft.dailySoftLimit}
          min="0"
          max="10000"
          step="1"
          disabled={loading || savingSettings}
          oninput={(event) => updateNumber("dailySoftLimit", event)}
        />
        <span class="field-hint">0 means no soft limit.</span>
      </label>

      <label>Retry backoff
        <Input
          type="number"
          value={draft.retryBackoffMs}
          min="0"
          max="300000"
          step="100"
          disabled={loading || savingSettings}
          oninput={(event) => updateNumber("retryBackoffMs", event)}
        />
        <span class="field-hint">0 means no wait before retry.</span>
      </label>

      <label>Stop after consecutive failures
        <Input
          type="number"
          value={draft.stopAfterConsecutiveFailures}
          min="1"
          max="50"
          step="1"
          disabled={loading || savingSettings}
          oninput={(event) => updateNumber("stopAfterConsecutiveFailures", event)}
        />
      </label>
    </div>

    <CheckboxRow
      title="Pause on auth challenge"
      checked={draft.pauseOnAuthChallenge}
      disabled={loading || savingSettings}
      onchange={(event) => updateBoolean("pauseOnAuthChallenge", event)}
    />

    <div class="actions">
      <Button
        title="Save YouTube settings"
        ariaLabel="Save YouTube settings"
        onclick={saveSettings}
        disabled={loading || savingSettings}
      >
        <Save size={15} aria-hidden="true" />
        {savingSettings ? "Saving..." : "Save settings"}
      </Button>
      <Button
        variant="secondary"
        title="Reload YouTube settings"
        ariaLabel="Reload YouTube settings"
        onclick={loadPanel}
        disabled={loading || savingSettings}
      >
        <RefreshCw size={15} aria-hidden="true" />
        Reload
      </Button>
    </div>
  </section>
</section>

<style>
  .youtube-settings-panel {
    gap: 1rem;
  }

  .youtube-settings-panel.embedded {
    padding: 0;
    border: 0;
    background: transparent;
    box-shadow: none;
  }

  .youtube-settings-panel.embedded .cookie-box {
    background: color-mix(in srgb, var(--panel-strong) 72%, transparent);
  }

  .status-strip,
  .actions,
  .cookie-actions {
    display: flex;
    gap: 0.55rem;
    align-items: center;
    flex-wrap: wrap;
  }

  .youtube-auth-section,
  .youtube-sync-policy-section,
  .cookie-box {
    display: flex;
    flex-direction: column;
    gap: 0.8rem;
  }

  .youtube-auth-section,
  .youtube-sync-policy-section {
    min-width: 0;
  }

  h3 {
    margin: 0;
    color: var(--text);
    font-size: 0.95rem;
  }

  .embedded-status {
    justify-content: flex-start;
  }

  .cookie-box {
    padding: 0.9rem;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--panel-strong);
  }

  .cookie-box-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 0.9rem;
  }

  .cookie-box-header > div:first-child {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    min-width: 0;
  }

  .cookie-box-header span,
  .field-hint {
    color: var(--muted);
    font-size: 0.78rem;
    line-height: 1.45;
  }

  .settings-fields {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 0.8rem;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    color: var(--muted);
    font-size: 0.9rem;
  }

  .cookie-dropzone {
    border: 2px dashed var(--border);
    border-radius: 8px;
    padding: 1.5rem;
    text-align: center;
    background: var(--panel);
    transition: border-color 0.2s, background-color 0.2s;
  }

  .cookie-dropzone.dragover {
    border-color: var(--primary);
    background: color-mix(in srgb, var(--primary) 8%, var(--panel));
  }

  .dropzone-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.5rem;
  }

  .dropzone-text {
    font-size: 0.85rem;
    color: var(--text);
  }

  .hidden-file-input {
    display: none;
  }

  .file-select-label {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 0.45rem 0.9rem;
    font-size: 0.82rem;
    font-weight: 600;
    color: white;
    background: var(--primary);
    border-radius: 6px;
    cursor: pointer;
    box-shadow: 0 2px 6px rgba(47, 109, 234, 0.12);
    transition: opacity 0.15s, transform 0.1s;
  }

  .file-select-label:hover {
    opacity: 0.9;
  }

  .cookie-textarea {
    width: 100%;
    min-width: 0;
    max-width: 100%;
    box-sizing: border-box;
    resize: vertical;
    min-height: 10rem;
    background: var(--panel);
    border: 1px solid var(--border);
    color: var(--text);
    padding: 0.8rem;
    border-radius: 8px;
    font: 0.9rem/1.45 ui-monospace, SFMono-Regular, Consolas, "Liberation Mono", monospace;
  }

  .cookie-textarea:focus {
    border-color: var(--primary);
    outline: none;
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--primary) 18%, transparent);
  }

  @media (max-width: 720px) {
    .settings-fields {
      grid-template-columns: 1fr;
    }

    .cookie-box-header {
      flex-direction: column;
      align-items: stretch;
    }
  }
</style>
