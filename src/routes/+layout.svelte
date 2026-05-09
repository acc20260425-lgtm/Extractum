<script lang="ts">
  import { LayoutDashboard, Moon, Settings, Sun, UserRound } from "@lucide/svelte";
  import { browser } from "$app/environment";
  import { page } from "$app/state";
  import Button from "$lib/components/ui/Button.svelte";
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
      href: "/analysis",
      label: "Workspace",
      caption: "Sources, reports, chat",
      icon: LayoutDashboard,
      active: (pathname: string) => pathname.startsWith("/analysis") || pathname === "/",
    },
    {
      href: "/accounts",
      label: "Accounts",
      caption: "Source access",
      icon: UserRound,
      active: (pathname: string) =>
        pathname.startsWith("/accounts") || pathname.startsWith("/auth"),
    },
    {
      href: "/settings",
      label: "Settings",
      caption: "Models and app",
      icon: Settings,
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
        <a class="brand" href="/analysis">
          <span class="brand-mark" aria-hidden="true">E</span>
          <span class="brand-copy">
            <strong>Extractum</strong>
            <small>Research workspace</small>
          </span>
        </a>
      </div>

      <nav class="sidebar-nav" aria-label="Primary">
        {#each navItems as item (item.href)}
          {@const NavIcon = item.icon}
          <a
            href={item.href}
            class:active={item.active(page.url.pathname)}
          >
            <span class="nav-row">
              <NavIcon size={16} aria-hidden="true" />
              <span class="nav-label">{item.label}</span>
            </span>
            <span class="nav-caption">{item.caption}</span>
          </a>
        {/each}
      </nav>

      <div class="sidebar-footer">
        <div class="footer-copy">
          <span class="footer-label">Workspace mode</span>
          <strong>NotebookLM x Telegram</strong>
        </div>
        <Button className="theme-toggle" variant="secondary" type="button" onclick={toggleTheme}>
          {#if theme === "light"}
            <Moon size={15} aria-hidden="true" />
          {:else}
            <Sun size={15} aria-hidden="true" />
          {/if}
          {theme === "light" ? "Dark theme" : "Light theme"}
        </Button>
      </div>
    </aside>

    <main class="workspace">
      <div class="workspace-topbar">
        <div class="workspace-route">
          <span class="workspace-kicker">Current space</span>
          <strong>
            {#if page.url.pathname.startsWith("/analysis")}
              Analysis workspace
            {:else if page.url.pathname.startsWith("/accounts") || page.url.pathname.startsWith("/auth")}
              Source access
            {:else if page.url.pathname.startsWith("/settings")}
              Settings
            {:else}
              Extractum
            {/if}
          </strong>
        </div>
        <div class="workspace-topbar-meta">
          <span class="workspace-badge">Local-first desktop</span>
          <span class="workspace-badge">Tauri + Svelte</span>
        </div>
      </div>
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
    --bg: #eef1f5;
    --bg-alt: #e8edf3;
    --panel: #fbfcfd;
    --panel-strong: #f2f4f7;
    --panel-hover: #e7ebf0;
    --border: #d7dde5;
    --border-strong: #c6d0dc;
    --text: #17212b;
    --muted: #6e7c8a;
    --primary: #2f6dea;
    --primary-hover: #2459c3;
    --danger: #d94d4d;
    --danger-hover: #b93f3f;
    --status-bg: #e7f0ff;
    --status-error-bg: #fde9ea;
    --status-error-text: #a23535;
    --shadow: 0 18px 40px rgba(23, 33, 43, 0.06);
    --shadow-soft: 0 8px 20px rgba(23, 33, 43, 0.05);
  }
  :global([data-theme="dark"]) {
    color-scheme: dark;
    --bg: #0f1419;
    --bg-alt: #111820;
    --panel: #182028;
    --panel-strong: #111820;
    --panel-hover: #22303b;
    --border: #2d3a46;
    --border-strong: #42515f;
    --text: #edf2f7;
    --muted: #90a1b2;
    --primary: #61a3ff;
    --primary-hover: #3e88f2;
    --danger: #ff6b6b;
    --danger-hover: #db5656;
    --status-bg: #1a2d47;
    --status-error-bg: #472225;
    --status-error-text: #ffb4b8;
    --shadow: 0 18px 44px rgba(0, 0, 0, 0.28);
    --shadow-soft: 0 10px 24px rgba(0, 0, 0, 0.18);
  }
  :global(body) {
    margin: 0;
    font-family: "Segoe UI", "Inter Tight", "IBM Plex Sans", sans-serif;
    background:
      radial-gradient(circle at top left, rgba(47, 109, 234, 0.14), transparent 28%),
      radial-gradient(circle at 85% 20%, rgba(56, 189, 248, 0.08), transparent 22%),
      linear-gradient(180deg, var(--bg), color-mix(in srgb, var(--bg-alt) 84%, white 16%));
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
  :global(.page-shell) {
    display: flex;
    flex-direction: column;
    gap: 0.95rem;
    min-width: 0;
  }
  :global(.page-hero) {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 1rem;
    padding: 1rem 1.05rem;
    border: 1px solid var(--border);
    border-radius: 16px;
    background:
      linear-gradient(180deg, color-mix(in srgb, var(--panel) 98%, white 2%), var(--panel));
    box-shadow: var(--shadow);
  }
  :global(.page-hero-copy) {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    min-width: 0;
  }
  :global(.page-hero-copy h1) {
    margin: 0;
    font-size: 1.42rem;
    line-height: 1.15;
  }
  :global(.page-hero-copy p) {
    margin: 0;
    max-width: 72ch;
    color: var(--muted);
    line-height: 1.55;
  }
  :global(.page-eyebrow) {
    font-size: 0.68rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--muted);
  }
  :global(.page-hero-meta) {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
    justify-content: flex-end;
  }
  :global(.page-grid) {
    display: grid;
    grid-template-columns: minmax(0, 1.45fr) minmax(300px, 0.9fr);
    gap: 0.95rem;
    align-items: start;
  }
  :global(.page-stack) {
    display: flex;
    flex-direction: column;
    gap: 0.95rem;
    min-width: 0;
  }
  :global(.desk-panel) {
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
    padding: 1rem 1.05rem;
    border: 1px solid var(--border);
    border-radius: 16px;
    background: var(--panel);
    box-shadow: var(--shadow);
    min-width: 0;
  }
  :global(.desk-panel-subtle) {
    background:
      linear-gradient(180deg, color-mix(in srgb, var(--panel-strong) 88%, transparent), var(--panel));
  }
  :global(.panel-header) {
    display: flex;
    justify-content: space-between;
    gap: 0.8rem;
    align-items: flex-start;
  }
  :global(.panel-header h2),
  :global(.panel-header h3) {
    margin: 0;
  }
  :global(.panel-header-copy) {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }
  :global(.panel-header-copy p) {
    margin: 0;
    color: var(--muted);
    line-height: 1.5;
    font-size: 0.88rem;
  }
  :global(.desk-divider) {
    height: 1px;
    background: color-mix(in srgb, var(--border) 78%, transparent);
  }
  :global(.muted-copy) {
    color: var(--muted);
  }

  .app {
    min-height: 100vh;
    color: var(--text);
  }

  .shell {
    display: flex;
    min-height: 100vh;
    gap: 0;
  }

  .sidebar {
    width: 214px;
    flex: 0 0 214px;
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
    padding: 0.85rem 0.7rem 0.85rem 0.85rem;
    background:
      linear-gradient(180deg, color-mix(in srgb, var(--panel) 96%, white 4%), var(--panel));
    border-right: 1px solid var(--border);
    box-shadow: inset -1px 0 0 rgba(255, 255, 255, 0.18);
  }

  .sidebar-header {
    padding: 0.2rem 0.2rem 0;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 0.65rem;
    color: inherit;
    text-decoration: none;
    padding: 0.5rem 0.55rem;
    border-radius: 12px;
  }

  .brand:hover {
    background: color-mix(in srgb, var(--panel-hover) 68%, transparent);
  }

  .brand-mark {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 2.1rem;
    height: 2.1rem;
    border-radius: 0.8rem;
    background: linear-gradient(180deg, var(--primary), color-mix(in srgb, var(--primary) 74%, black));
    color: white;
    font-size: 0.8rem;
    font-weight: 700;
    letter-spacing: 0.04em;
    box-shadow: 0 10px 24px rgba(47, 109, 234, 0.22);
  }

  .brand-copy {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    min-width: 0;
  }

  .brand-copy strong {
    font-size: 0.94rem;
    line-height: 1.1;
  }

  .brand-copy small {
    color: var(--muted);
    font-size: 0.72rem;
    line-height: 1.1;
  }

  .sidebar-nav {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
  }

  .sidebar-nav a {
    display: flex;
    flex-direction: column;
    gap: 0.18rem;
    padding: 0.62rem 0.78rem;
    border-radius: 12px;
    color: var(--muted);
    text-decoration: none;
    transition: background 0.2s, color 0.2s, border-color 0.2s;
    border: 1px solid transparent;
  }

  .nav-row {
    display: flex;
    align-items: center;
    gap: 0.45rem;
    min-width: 0;
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
    font-size: 0.9rem;
    font-weight: 600;
    line-height: 1.15;
  }

  .nav-caption {
    font-size: 0.72rem;
    line-height: 1.2;
    color: var(--muted);
  }

  .sidebar-nav a.active .nav-caption,
  .sidebar-nav a:hover .nav-caption {
    color: color-mix(in srgb, var(--muted) 72%, var(--text));
  }

  .sidebar-footer {
    margin-top: auto;
    padding: 0.25rem 0.2rem 0;
    display: flex;
    flex-direction: column;
    gap: 0.65rem;
  }

  .footer-copy {
    display: flex;
    flex-direction: column;
    gap: 0.1rem;
    color: var(--muted);
  }

  .footer-copy strong {
    font-size: 0.82rem;
    color: var(--text);
  }

  .footer-label,
  .workspace-kicker {
    font-size: 0.68rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--muted);
  }

  :global(.theme-toggle) {
    width: 100%;
  }

  .workspace {
    flex: 1;
    min-width: 0;
    padding: 0.9rem 1rem 1rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .workspace-topbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    min-height: 2.25rem;
    padding: 0.15rem 0.15rem 0.35rem;
    border-bottom: 1px solid color-mix(in srgb, var(--border) 76%, transparent);
  }

  .workspace-route {
    display: flex;
    flex-direction: column;
    gap: 0.08rem;
  }

  .workspace-route strong {
    font-size: 0.96rem;
  }

  .workspace-topbar-meta {
    display: flex;
    gap: 0.45rem;
    flex-wrap: wrap;
    justify-content: flex-end;
  }

  .workspace-badge {
    display: inline-flex;
    align-items: center;
    min-height: 1.8rem;
    padding: 0 0.7rem;
    border-radius: 999px;
    border: 1px solid color-mix(in srgb, var(--border-strong) 72%, transparent);
    background: color-mix(in srgb, var(--panel) 88%, transparent);
    color: var(--muted);
    font-size: 0.74rem;
    letter-spacing: 0.02em;
  }

  .workspace-inner {
    width: min(1640px, 100%);
    margin: 0 auto;
    min-width: 0;
  }

  @media (max-width: 820px) {
    .shell {
      flex-direction: column;
    }

    .sidebar {
      width: auto;
      flex-basis: auto;
      padding: 0.8rem;
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

    .workspace-topbar,
    :global(.page-hero) {
      flex-direction: column;
      align-items: stretch;
    }

    :global(.page-grid) {
      grid-template-columns: 1fr;
    }
  }
</style>
