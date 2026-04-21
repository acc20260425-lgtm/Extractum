<script lang="ts">
  import { browser } from "$app/environment";
  import { page } from "$app/stores";
  import ToastHost from "$lib/components/toast-host.svelte";

  let { children } = $props();
  let theme = $state<"light" | "dark">("light");

  if (browser) {
    theme = localStorage.getItem("theme") === "dark" ? "dark" : "light";
  }

  function toggleTheme() {
    theme = theme === "light" ? "dark" : "light";
    if (browser) {
      localStorage.setItem("theme", theme);
    }
  }
</script>

<svelte:head>
  <meta name="color-scheme" content={theme === "dark" ? "dark" : "light"} />
</svelte:head>

<div class="app" data-theme={theme}>
  <nav>
    <div class="nav-links">
      <a href="/accounts" class:active={$page.url.pathname.startsWith("/accounts") || $page.url.pathname.startsWith("/auth")}>Accounts</a>
      <a href="/sources" class:active={$page.url.pathname.startsWith("/sources")}>Sources</a>
      <a href="/analysis" class:active={$page.url.pathname.startsWith("/analysis")}>Analysis</a>
      <a href="/settings" class:active={$page.url.pathname.startsWith("/settings")}>Settings</a>
    </div>
    <button class="theme-toggle secondary" type="button" onclick={toggleTheme}>
      {theme === "light" ? "Dark theme" : "Light theme"}
    </button>
  </nav>
  <ToastHost />
  <main>
    {@render children()}
  </main>
</div>

<style>
  :global(*, *::before, *::after) { box-sizing: border-box; }
  :global(:root) {
    color-scheme: light;
    --bg: #f5f7fb;
    --panel: #ffffff;
    --panel-strong: #eef2f8;
    --panel-hover: #e4eaf5;
    --border: #d3dbea;
    --text: #1b2430;
    --muted: #657085;
    --primary: #2563eb;
    --primary-hover: #1d4ed8;
    --danger: #dc3545;
    --danger-hover: #b42330;
    --status-bg: #e3efff;
    --status-error-bg: #fde7ea;
    --status-error-text: #b42330;
    --shadow: 0 18px 45px rgba(37, 99, 235, 0.08);
  }
  :global([data-theme="dark"]) {
    color-scheme: dark;
    --bg: #111827;
    --panel: #1f2937;
    --panel-strong: #111827;
    --panel-hover: #1a2332;
    --border: #374151;
    --text: #f3f4f6;
    --muted: #9ca3af;
    --primary: #3b82f6;
    --primary-hover: #2563eb;
    --danger: #dc3545;
    --danger-hover: #a71d2a;
    --status-bg: #1e3a5f;
    --status-error-bg: #4a1a1a;
    --status-error-text: #fda4af;
    --shadow: 0 22px 50px rgba(0, 0, 0, 0.32);
  }
  :global(body) {
    margin: 0;
    font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
    background:
      radial-gradient(circle at top, rgba(37, 99, 235, 0.16), transparent 34%),
      linear-gradient(180deg, var(--bg), color-mix(in srgb, var(--bg) 82%, white 18%));
    color: var(--text);
  }
  :global(h1, h2, h3) { margin: 0 0 1rem; }
  :global(input) {
    background: var(--panel-strong);
    border: 1px solid var(--border);
    color: var(--text);
    padding: 0.6rem 0.8rem;
    border-radius: 6px;
    font-size: 1rem;
    width: 100%;
  }
  :global(select) {
    background: var(--panel-strong);
    border: 1px solid var(--border);
    color: var(--text);
  }
  :global(input::placeholder) { color: var(--muted); }
  :global(input:focus), :global(select:focus) {
    border-color: var(--primary);
    outline: none;
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--primary) 18%, transparent);
  }
  :global(button) {
    background: var(--primary);
    color: white;
    border: none;
    padding: 0.6rem 1rem;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.95rem;
    font-weight: 600;
    transition: background 0.2s, border-color 0.2s, color 0.2s;
  }
  :global(button:hover) { background: var(--primary-hover); }
  :global(button.secondary) {
    background: transparent;
    border: 1px solid var(--border);
    color: var(--text);
  }
  :global(button.secondary:hover) { background: var(--panel-hover); }
  :global(button.danger) { background: var(--danger); }
  :global(button.danger:hover) { background: var(--danger-hover); }
  :global(button:disabled) { opacity: 0.5; cursor: not-allowed; }

  .app {
    display: flex;
    flex-direction: column;
    min-height: 100vh;
    color: var(--text);
  }

  nav {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
    padding: 0.75rem 1.5rem;
    background: color-mix(in srgb, var(--panel) 88%, transparent);
    border-bottom: 1px solid var(--border);
    backdrop-filter: blur(14px);
  }
  .nav-links { display: flex; gap: 0.25rem; flex-wrap: wrap; }
  nav a {
    color: var(--muted);
    text-decoration: none;
    padding: 0.4rem 0.8rem;
    border-radius: 6px;
    font-size: 0.9rem;
    transition: color 0.2s, background 0.2s;
  }
  nav a:hover { color: var(--text); background: var(--panel-hover); }
  nav a.active { color: white; background: var(--primary); }
  .theme-toggle { white-space: nowrap; }

  main {
    flex: 1;
    padding: 2rem;
    width: min(1480px, calc(100vw - 3rem));
    max-width: 1480px;
    margin: 0 auto;
  }

  @media (max-width: 640px) {
    nav { flex-direction: column; align-items: stretch; }
    .nav-links { justify-content: center; }
    .theme-toggle { width: 100%; }
    main {
      padding: 1.25rem;
      width: 100%;
    }
  }
</style>
