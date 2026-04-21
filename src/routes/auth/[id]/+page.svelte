<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { page } from "$app/stores";

  const accountId = parseInt($page.params.id);

  interface AccountRecord {
    id: number;
    label: string;
    api_id: number;
    api_hash: string;
    phone: string | null;
    created_at: number;
  }

  let label = $state("");
  let apiId = $state(0);
  let apiHash = $state("");
  let phone = $state("");
  let code = $state("");
  let status = $state("");
  let step = $state<"connecting" | "phone" | "code" | "done">("connecting");
  let loading = $state(false);

  async function loadAccount() {
    try {
      const acc = await invoke<AccountRecord | null>("get_account", { accountId });
      if (!acc) {
        status = "Account not found";
        return;
      }
      label = acc.label;
      apiId = acc.api_id;
      apiHash = acc.api_hash;
      phone = acc.phone ?? "";

      await initClient();
    } catch (e) {
      status = `Error: ${e}`;
    }
  }

  async function initClient() {
    loading = true;
    status = "Connecting...";
    try {
      const isAuth = await invoke<boolean>("tg_init", {
        accountId,
        apiId,
        apiHash,
      });
      if (isAuth) {
        step = "done";
        status = "Already authenticated.";
      } else {
        step = "phone";
        status = "";
      }
    } catch (e) {
      status = `Error: ${e}`;
      step = "phone";
    } finally {
      loading = false;
    }
  }

  async function sendCode() {
    loading = true;
    status = "";
    try {
      await invoke("tg_send_code", { accountId, phone });
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
      await invoke("tg_sign_in", { accountId, code });
      // Save phone to DB
      await invoke("set_account_phone", { accountId, phone });
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
      await invoke("tg_logout", { accountId });
      await invoke("clear_account_phone", { accountId });
      phone = "";
      step = "phone";
      status = "";
    } catch (e) {
      status = `Error: ${e}`;
    } finally {
      loading = false;
    }
  }

  onMount(loadAccount);
</script>

<div class="back-row">
  <a href="/accounts">← Accounts</a>
</div>

<h1>{label || "Account"}</h1>

{#if status}
  <p class="status" class:error={status.startsWith("Error")}>{status}</p>
{/if}

{#if step === "connecting"}
  <div class="card">
    <p class="hint">Connecting to Telegram...</p>
  </div>
{/if}

{#if step === "phone"}
  <div class="card">
    <h3>Sign In</h3>
    <p class="hint">API ID: {apiId}</p>
    <label>Phone number
      <input type="tel" bind:value={phone} placeholder="+79991234567" />
    </label>
    <button onclick={sendCode} disabled={loading || !phone}>
      {loading ? "Sending..." : "Send Code"}
    </button>
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
    <h3>✓ Authenticated</h3>
    <p class="hint">Phone: {phone}</p>
    <div class="row">
      <a href="/sources?account={accountId}" class="btn-link">View Sources</a>
      <button class="danger" onclick={logout} disabled={loading}>Logout</button>
    </div>
  </div>
{/if}

<style>
  .back-row { margin-bottom: 1rem; }
  .back-row a { color: #888; font-size: 0.9rem; text-decoration: none; }
  .back-row a:hover { color: #eee; }
  .card {
    background: #2a2a2a;
    border-radius: 12px;
    padding: 1.5rem;
    margin-bottom: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }
  label { display: flex; flex-direction: column; gap: 0.3rem; font-size: 0.85rem; color: #aaa; }
  .hint { font-size: 0.85rem; color: #888; margin: 0; }
  .row { display: flex; gap: 0.5rem; align-items: center; }
  .btn-link {
    padding: 0.6rem 1rem;
    border-radius: 6px;
    background: #007bff;
    color: white;
    text-decoration: none;
    font-size: 0.95rem;
    font-weight: 600;
  }
  .btn-link:hover { background: #0056b3; }
  .status { padding: 0.6rem 1rem; border-radius: 6px; background: #1e3a5f; font-size: 0.9rem; margin-bottom: 1rem; }
  .status.error { background: #4a1a1a; color: #f88; }
</style>
