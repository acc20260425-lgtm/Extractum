<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import Database from "@tauri-apps/plugin-sql";

  let rustPing = $state("Waiting for Rust...");
  let dbStatus = $state("Checking database...");
  let sourcesCount = $state<number | null>(null);

  async function checkFoundation() {
    try {
      // 1. Check Rust Command
      rustPing = await invoke("ping_db");

      // 2. Check SQLite Plugin
      const db = await Database.load("sqlite:extractum.db");
      const result = await db.select<{ count: number }[]>("SELECT COUNT(*) as count FROM sources");
      sourcesCount = result[0].count;
      dbStatus = "Database connection successful! Tables are ready.";
    } catch (error) {
      console.error(error);
      dbStatus = `Error: ${error}`;
    }
  }

  onMount(() => {
    checkFoundation();
  });
</script>

<main class="container">
  <h1>Extractum Foundation Test</h1>

  <div class="card">
    <h3>Backend Status</h3>
    <p class="{rustPing.startsWith('Rust') ? 'success' : 'pending'}">{rustPing}</p>
  </div>

  <div class="card">
    <h3>Database Status</h3>
    <p class="{sourcesCount !== null ? 'success' : 'error'}">{dbStatus}</p>
    {#if sourcesCount !== null}
      <p>Current sources in DB: <strong>{sourcesCount}</strong></p>
    {/if}
  </div>

  <button onclick={checkFoundation}>Re-check</button>
</main>

<style>
  :root {
    font-family: sans-serif;
    background-color: #1a1a1a;
    color: #eee;
  }
  .container {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 2rem;
  }
  .card {
    background: #2a2a2a;
    border-radius: 8px;
    padding: 1rem;
    margin: 1rem;
    width: 80%;
    max-width: 500px;
    box-shadow: 0 4px 6px rgba(0,0,0,0.3);
  }
  .success { color: #4caf50; }
  .error { color: #f44336; }
  .pending { color: #ff9800; }
  button {
    margin-top: 1rem;
    padding: 0.5rem 1rem;
    cursor: pointer;
  }
</style>
