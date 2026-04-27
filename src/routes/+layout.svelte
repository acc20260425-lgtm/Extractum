<script lang="ts">
  import { browser } from "$app/environment";
  import { page } from "$app/state";
  import ModalHost from "$lib/components/modal-host.svelte";
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

  const navItems = [
    {
      href: "/accounts",
      label: "Accounts",
      caption: "Telegram access",
      active: (pathname: string) =>
        pathname.startsWith("/accounts") || pathname.startsWith("/auth"),
    },
    {
      href: "/sources",
      label: "Sources",
      caption: "Channels and sync",
      active: (pathname: string) => pathname.startsWith("/sources"),
    },
    {
      href: "/analysis",
      label: "Analysis",
      caption: "Reports and chat",
      active: (pathname: string) => pathname.startsWith("/analysis"),
    },
    {
      href: "/settings",
      label: "Settings",
      caption: "Models and app",
      active: (pathname: string) => pathname.startsWith("/settings"),
    },
  ];
</script>

<svelte:head>
  <meta name="color-scheme" content={theme === "dark" ? "dark" : "light"} />
</svelte:head>

<div class="app" data-theme={theme}>
  <ToastHost />
  <ModalHost />
  <div class="shell">
    <aside class="sidebar">
      <div class="sidebar-header">
        <a class="brand" href="/accounts">
          <span class="brand-mark" aria-hidden="true">E</span>
          <span class="brand-copy">
            <strong>Extractum</strong>
            <small>Research workspace</small>
          </span>
        </a>
      </div>

      <nav class="sidebar-nav" aria-label="Primary">
        {#each navItems as item (item.href)}
          <a
            href={item.href}
            class:active={item.active(page.url.pathname)}
          >
            <span class="nav-label">{item.label}</span>
            <span class="nav-caption">{item.caption}</span>
          </a>
        {/each}
      </nav>

      <div class="sidebar-footer">
        <button class="theme-toggle secondary" type="button" onclick={toggleTheme}>
          {theme === "light" ? "Dark theme" : "Light theme"}
        </button>
      </div>
    </aside>

    <main class="workspace">
      <div class="workspace-inner">
        {@render children()}
      </div>
    </main>
  </div>
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
    min-height: 100vh;
    color: var(--text);
  }

  .shell {
    display: flex;
    min-height: 100vh;
  }

  .sidebar {
    width: 248px;
    flex: 0 0 248px;
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
    padding: 1rem 0.85rem 1rem 1rem;
    background:
      linear-gradient(180deg, color-mix(in srgb, var(--panel) 94%, white 6%), var(--panel));
    border-right: 1px solid var(--border);
    box-shadow: inset -1px 0 0 rgba(255, 255, 255, 0.3);
  }

  .sidebar-header {
    padding: 0.25rem 0.25rem 0;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    color: inherit;
    text-decoration: none;
    padding: 0.45rem 0.55rem;
    border-radius: 14px;
  }

  .brand:hover {
    background: color-mix(in srgb, var(--panel-hover) 68%, transparent);
  }

  .brand-mark {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 2.25rem;
    height: 2.25rem;
    border-radius: 999px;
    background: linear-gradient(180deg, var(--primary), color-mix(in srgb, var(--primary) 75%, black));
    color: white;
    font-size: 1rem;
    font-weight: 700;
    box-shadow: 0 10px 24px rgba(37, 99, 235, 0.2);
  }

  .brand-copy {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    min-width: 0;
  }

  .brand-copy strong {
    font-size: 0.98rem;
    line-height: 1.1;
  }

  .brand-copy small {
    color: var(--muted);
    font-size: 0.77rem;
    line-height: 1.1;
  }

  .sidebar-nav {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
  }

  .sidebar-nav a {
    display: flex;
    flex-direction: column;
    gap: 0.18rem;
    padding: 0.72rem 0.85rem;
    border-radius: 14px;
    color: var(--muted);
    text-decoration: none;
    transition: background 0.2s, color 0.2s, border-color 0.2s;
    border: 1px solid transparent;
  }

  .sidebar-nav a:hover {
    color: var(--text);
    background: color-mix(in srgb, var(--panel-hover) 72%, transparent);
    border-color: color-mix(in srgb, var(--border) 72%, transparent);
  }

  .sidebar-nav a.active {
    color: var(--text);
    background:
      linear-gradient(180deg, color-mix(in srgb, var(--primary) 12%, var(--panel)), color-mix(in srgb, var(--primary) 7%, var(--panel)));
    border-color: color-mix(in srgb, var(--primary) 20%, var(--border));
    box-shadow: 0 10px 22px rgba(37, 99, 235, 0.08);
  }

  .nav-label {
    font-size: 0.95rem;
    font-weight: 600;
    line-height: 1.15;
  }

  .nav-caption {
    font-size: 0.76rem;
    line-height: 1.2;
    color: var(--muted);
  }

  .sidebar-nav a.active .nav-caption,
  .sidebar-nav a:hover .nav-caption {
    color: color-mix(in srgb, var(--muted) 72%, var(--text));
  }

  .sidebar-footer {
    margin-top: auto;
    padding: 0.25rem;
  }

  .theme-toggle {
    width: 100%;
    white-space: nowrap;
  }

  .workspace {
    flex: 1;
    min-width: 0;
    padding: 1.25rem;
  }

  .workspace-inner {
    width: min(1480px, 100%);
    margin: 0 auto;
  }

  @media (max-width: 820px) {
    .shell {
      flex-direction: column;
    }

    .sidebar {
      width: auto;
      flex-basis: auto;
      padding: 0.9rem;
      border-right: none;
      border-bottom: 1px solid var(--border);
      box-shadow: none;
    }

    .sidebar-nav {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    }

    .workspace {
      padding: 1rem;
    }
  }
</style>
