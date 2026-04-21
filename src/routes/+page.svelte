<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import Database from "@tauri-apps/plugin-sql";

  let apiId = $state("");
  let apiHash = $state("");
  let phone = $state("");
  let code = $state("");
  let status = $state("Idle");
  let isAuthenticated = $state(false);
  let step = $state<"init" | "phone" | "code" | "authenticated">("init");

  async function loadSettings() {
    try {
      const db = await Database.load("sqlite:extractum.db");
      const settings = await db.select<{ key: string, value: string }[]>("SELECT * FROM app_settings");
      
      const idSetting = settings.find(s => s.key === "api_id");
      const hashSetting = settings.find(s => s.key === "api_hash");
      
      if (idSetting) apiId = idSetting.value;
      if (hashSetting) apiHash = hashSetting.value;
      
      if (apiId && apiHash) {
        await initTelegram();
      }
    } catch (error) {
      console.error("Failed to load settings:", error);
    }
  }

  async function saveSettings() {
    try {
      const db = await Database.load("sqlite:extractum.db");
      await db.execute("INSERT OR REPLACE INTO app_settings (key, value) VALUES ('api_id', ?), ('api_hash', ?)", [apiId, apiHash]);
      status = "Settings saved.";
      await initTelegram();
    } catch (error) {
      status = `Error saving settings: ${error}`;
    }
  }

  async function initTelegram() {
    try {
      status = "Initializing Telegram client...";
      const isAuth = await invoke<boolean>("tg_init", { 
        apiId: parseInt(apiId), 
        apiHash 
      });
      isAuthenticated = isAuth;
      if (isAuthenticated) {
        step = "authenticated";
        status = "Already authenticated.";
      } else {
        step = "phone";
        status = "Telegram client ready. Please sign in.";
      }
    } catch (error) {
      status = `Initialization error: ${error}`;
    }
  }

  async function sendCode() {
    try {
      status = "Sending code...";
      await invoke("tg_send_code", { phone });
      step = "code";
      status = "Code sent. Please check your Telegram.";
    } catch (error) {
      status = `Error sending code: ${error}`;
    }
  }

  async function signIn() {
    try {
      status = "Signing in...";
      const result = await invoke<boolean>("tg_sign_in", { code });
      if (result) {
        isAuthenticated = true;
        step = "authenticated";
        status = "Successfully signed in!";
      }
    } catch (error) {
      status = `Error signing in: ${error}`;
    }
  }

  async function logout() {
    try {
      await invoke("tg_logout");
      isAuthenticated = false;
      step = "phone";
      status = "Logged out.";
    } catch (error) {
      status = `Logout error: ${error}`;
    }
  }

  onMount(() => {
    loadSettings();
  });
</script>

<main class="container">
  <h1>Extractum: Telegram Setup</h1>

  <div class="card">
    <p class="status">Status: <strong>{status}</strong></p>
  </div>

  {#if step === "init"}
    <div class="card">
      <h3>1. API Credentials</h3>
      <div class="input-group">
        <label for="api-id">API ID</label>
        <input id="api-id" type="text" bind:value={apiId} placeholder="1234567" />
      </div>
      <div class="input-group">
        <label for="api-hash">API Hash</label>
        <input id="api-hash" type="text" bind:value={apiHash} placeholder="abcdef123456..." />
      </div>
      <button onclick={saveSettings}>Initialize Telegram</button>
    </div>
  {/if}

  {#if step === "phone"}
    <div class="card">
      <h3>2. Telegram Login</h3>
      <p>Enter your phone number in international format (e.g., +1234567890).</p>
      <div class="input-group">
        <input type="text" bind:value={phone} placeholder="+1..." />
      </div>
      <button onclick={sendCode}>Send Code</button>
      <button class="secondary" onclick={() => step = "init"}>Change API Credentials</button>
    </div>
  {/if}

  {#if step === "code"}
    <div class="card">
      <h3>3. Enter Code</h3>
      <p>A verification code was sent to your Telegram app.</p>
      <div class="input-group">
        <input type="text" bind:value={code} placeholder="12345" />
      </div>
      <button onclick={signIn}>Sign In</button>
      <button class="secondary" onclick={() => step = "phone"}>Back to Phone</button>
    </div>
  {/if}

  {#if step === "authenticated"}
    <div class="card">
      <h3>You are Connected!</h3>
      <p>Telegram is successfully linked with Extractum.</p>
      <button class="danger" onclick={logout}>Logout</button>
    </div>
  {/if}
</main>

<style>
  :root {
    font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
    background-color: #1a1a1a;
    color: #eee;
  }
  .container {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 2rem;
    max-width: 600px;
    margin: 0 auto;
  }
  .card {
    background: #2a2a2a;
    border-radius: 12px;
    padding: 1.5rem;
    margin-bottom: 1.5rem;
    width: 100%;
    box-shadow: 0 4px 20px rgba(0,0,0,0.4);
  }
  .status {
    font-size: 0.9rem;
    color: #bbb;
  }
  .input-group {
    display: flex;
    flex-direction: column;
    margin-bottom: 1rem;
  }
  .input-group label {
    margin-bottom: 0.4rem;
    font-size: 0.85rem;
    color: #aaa;
  }
  input {
    background: #1a1a1a;
    border: 1px solid #444;
    color: white;
    padding: 0.6rem 0.8rem;
    border-radius: 6px;
    font-size: 1rem;
  }
  input:focus {
    border-color: #007bff;
    outline: none;
  }
  button {
    background: #007bff;
    color: white;
    border: none;
    padding: 0.7rem 1.2rem;
    border-radius: 6px;
    cursor: pointer;
    font-size: 1rem;
    font-weight: 600;
    transition: background 0.2s;
    width: 100%;
  }
  button:hover {
    background: #0056b3;
  }
  button.secondary {
    background: transparent;
    border: 1px solid #444;
    margin-top: 0.5rem;
  }
  button.secondary:hover {
    background: #333;
  }
  button.danger {
    background: #dc3545;
  }
  button.danger:hover {
    background: #a71d2a;
  }
  h1 { margin-bottom: 2rem; }
  h3 { margin-top: 0; margin-bottom: 1rem; }
</style>
