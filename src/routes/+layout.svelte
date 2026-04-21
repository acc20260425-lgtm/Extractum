<script lang="ts">
  import { page } from "$app/stores";
  let { children } = $props();
</script>

<div class="app">
  <nav>
    <a href="/accounts" class:active={$page.url.pathname.startsWith("/accounts") || $page.url.pathname.startsWith("/auth")}>Accounts</a>
    <a href="/sources" class:active={$page.url.pathname.startsWith("/sources")}>Sources</a>
  </nav>
  <main>
    {@render children()}
  </main>
</div>

<style>
  :global(*, *::before, *::after) { box-sizing: border-box; }
  :global(body) {
    margin: 0;
    font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
    background-color: #1a1a1a;
    color: #eee;
  }
  :global(h1, h2, h3) { margin: 0 0 1rem; }
  :global(input) {
    background: #1a1a1a;
    border: 1px solid #444;
    color: white;
    padding: 0.6rem 0.8rem;
    border-radius: 6px;
    font-size: 1rem;
    width: 100%;
  }
  :global(input:focus) { border-color: #007bff; outline: none; }
  :global(button) {
    background: #007bff;
    color: white;
    border: none;
    padding: 0.6rem 1rem;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.95rem;
    font-weight: 600;
    transition: background 0.2s;
  }
  :global(button:hover) { background: #0056b3; }
  :global(button.secondary) {
    background: transparent;
    border: 1px solid #444;
  }
  :global(button.secondary:hover) { background: #333; }
  :global(button.danger) { background: #dc3545; }
  :global(button.danger:hover) { background: #a71d2a; }
  :global(button:disabled) { opacity: 0.5; cursor: not-allowed; }

  .app { display: flex; flex-direction: column; min-height: 100vh; }

  nav {
    display: flex;
    gap: 0.25rem;
    padding: 0.75rem 1.5rem;
    background: #111;
    border-bottom: 1px solid #333;
  }
  nav a {
    color: #aaa;
    text-decoration: none;
    padding: 0.4rem 0.8rem;
    border-radius: 6px;
    font-size: 0.9rem;
    transition: color 0.2s, background 0.2s;
  }
  nav a:hover { color: #eee; background: #222; }
  nav a.active { color: #fff; background: #007bff; }

  main { flex: 1; padding: 2rem; max-width: 700px; margin: 0 auto; width: 100%; }
</style>
