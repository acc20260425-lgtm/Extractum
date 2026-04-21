<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import Database from "@tauri-apps/plugin-sql";

  let apiId = $state("");
  let apiHash = $state("");
  let phone = $state("");
  let code = $state("");
  let status = $state("");
  let step = $state<"init" | "phone" | "code" | "done">("init");
  let loading = $state(false);

  async function loadSettings() {
    try {
      const db = await Database.load("sqlite:extractum.db");
      const rows = await db.select<{ key: string; value: string }[]>(
        "SELECT key, value FROM app_settings WHERE key IN ('api_id', 'api_hash')"
      );
      for (const r of rows) {
        if (r.key === "api_id") apiId = r.value;
        if (r.key === "api_hash") apiHash = r.value;
      }
      if (apiId && apiHash) await initTelegram();
    } catch (e) {
      console.error(e);
    }
  }

  async function saveAndInit() {
    if (!apiId || !apiHash) return;
    loading = true;
    try {
      const db = await Database.load("sqlite:extractum.db");
      await db.execute(
        "INSERT OR REPLACE INTO app_settings (key, value) VALUES ('api_id', ?), ('api_hash', ?)",
        [apiId, apiHash]
      );
      await initTelegram();
    } catch (e) {
      status = `Error: ${e}`;
    } finally {
      loading = false;
    }
  }

  async function initTelegram() {
    loading = true;
    status = "Connecting...";
    try {
      const isAuth = await invoke<boolean>("tg_init", {
        apiId: parseInt(apiId),
        apiHash,
      });
      if (isAuth) {
        step = "done";
        status = "Connected.";
      } else {
        step = "phone";
        status = "";
      }
    } catch (e) {
      status = `Error: ${e}`;
      step = "init";
    } finally {
      loading = false;
    }
  }

  async function sendCode() {
    loading = true;
    status = "";
    try {
      await invoke("tg_send_code", { phone });
      step = "code";
    } catch (e) {
      status = `Error: ${e}`;
    } finally {
      loading = false;
    }
  }

  async function signIn() {
    loading = true;
    status = "";
    try {
      await invoke("tg_sign_in", { code });
      step = "done";
      status = "Signed in successfully.";
    } catch (e) {
      status = `Error: ${e}`;
    } finally {
      loading = false;
    }
  }

  async function logout() {
    loading = true;
    try {
      await invoke("tg_logout");
      step = "phone";
      status = "";
    } catch (e) {
      status = `Error: ${e}`;
    } finally {
      loading = false;
    }
  }

  onMount(loadSettings);
</script>

<h1>Telegram Auth</h1>

{#if status}
  <p class="status" class:error={status.startsWith("Error")}>{status}</p>
{/if}

{#if step === "init"}
  <div class="card">
    <h3>API Credentials</h3>
    <p class="hint">Get your credentials at <a href="https://my.telegram.org" target="_blank">my.telegram.org</a></p>
    <label>API ID
      <input type="text" bind:value={apiId} placeholder="1234567" />
    </label>
    <label>API Hash
      <input type="text" bind:value={apiHash} placeholder="abcdef..." />
    </label>
    <button onclick={saveAndInit} disabled={loading || !apiId || !apiHash}>
      {loading ? "Connecting..." : "Connect"}
    </button>
  </div>
{/if}

{#if step === "phone"}
  <div class="card">
    <h3>Phone Number</h3>
    <label>International format
      <input type="tel" bind:value={phone} placeholder="+79991234567" />
    </label>
    <button onclick={sendCode} disabled={loading || !phone}>
      {loading ? "Sending..." : "Send Code"}
    </button>
    <button class="secondary" onclick={() => (step = "init")}>Change credentials</button>
  </div>
{/if}

{#if step === "code"}
  <div class="card">
    <h3>Verification Code</h3>
    <p class="hint">Check your Telegram app for the code.</p>
    <label>Code
      <input type="text" bind:value={code} placeholder="12345" />
    </label>
    <button onclick={signIn} disabled={loading || !code}>
      {loading ? "Signing in..." : "Sign In"}
    </button>
    <button class="secondary" onclick={() => (step = "phone")}>Back</button>
  </div>
{/if}

{#if step === "done"}
  <div class="card">
    <h3>✓ Connected to Telegram</h3>
    <p class="hint">You can now manage sources.</p>
    <button class="danger" onclick={logout} disabled={loading}>Logout</button>
  </div>
{/if}

<style>
  .card {
    background: #2a2a2a;
    border-radius: 12px;
    padding: 1.5rem;
    margin-bottom: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }
  label {
    display: flex;
    flex-direction: column;
    gap: 0.3rem;
    font-size: 0.85rem;
    color: #aaa;
  }
  .hint { font-size: 0.85rem; color: #888; margin: 0; }
  .hint a { color: #007bff; }
  .status { padding: 0.6rem 1rem; border-radius: 6px; background: #1e3a5f; font-size: 0.9rem; }
  .status.error { background: #4a1a1a; color: #f88; }
</style>
