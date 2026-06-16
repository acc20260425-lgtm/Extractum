<script lang="ts">
  import "$lib/styles/base.css";
  import { goto } from "$app/navigation";
  import { Activity, FolderKanban, LayoutDashboard, Library, Menu, Moon, Settings, ShieldCheck, Sun, UserRound } from "@lucide/svelte";
  import { browser } from "$app/environment";
  import { page } from "$app/state";
  import AppSidebar from "$lib/components/app-sidebar.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import ModalHost from "$lib/components/modal-host.svelte";
  import ToastHost from "$lib/components/toast-host.svelte";

  const SIDEBAR_COLLAPSED_KEY = "extractum.sidebar.collapsed";

  let { children } = $props();
  let theme = $state<"light" | "dark">("light");
  let sidebarCollapsed = $state(false);
  let mobileSidebarOpen = $state(false);
  let uiMode = $state<"legacy" | "projects">("legacy");

  if (browser) {
    theme = localStorage.getItem("theme") === "dark" ? "dark" : "light";
    sidebarCollapsed = localStorage.getItem(SIDEBAR_COLLAPSED_KEY) === "true";
    let initialMode = (localStorage.getItem("extractum.uiMode") as "legacy" | "projects") || "legacy";

    if (page.url.pathname.startsWith("/projects") && initialMode === "legacy") {
      initialMode = "projects";
      localStorage.setItem("extractum.uiMode", "projects");
    } else if (page.url.pathname.startsWith("/analysis") && initialMode === "projects") {
      initialMode = "legacy";
      localStorage.setItem("extractum.uiMode", "legacy");
    }
    uiMode = initialMode;
  }

  // Keep uiMode in sync with current route path reactively
  $effect(() => {
    const pathname = page.url.pathname;
    if (pathname.startsWith("/projects") && uiMode === "legacy") {
      setUiMode("projects");
    } else if ((pathname.startsWith("/analysis") || pathname === "/") && uiMode === "projects") {
      setUiMode("legacy");
    }
  });

  function toggleTheme() {
    theme = theme === "light" ? "dark" : "light";
    if (browser) {
      localStorage.setItem("theme", theme);
    }
  }

  function setSidebarCollapsed(collapsed: boolean) {
    sidebarCollapsed = collapsed;
    if (browser) {
      localStorage.setItem(SIDEBAR_COLLAPSED_KEY, String(collapsed));
    }
  }

  function toggleSidebarCollapsed() {
    setSidebarCollapsed(!sidebarCollapsed);
  }

  function closeMobileSidebar() {
    mobileSidebarOpen = false;
  }

  function handleShellKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      mobileSidebarOpen = false;
    }
  }

  function setUiMode(mode: "legacy" | "projects") {
    uiMode = mode;
    if (browser) {
      localStorage.setItem("extractum.uiMode", mode);
    }
  }

  const legacyNavItems = [
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
      href: "/diagnostics",
      label: "Diagnostics",
      caption: "Local health",
      icon: ShieldCheck,
      active: (pathname: string) => pathname.startsWith("/diagnostics"),
    },
    {
      href: "/settings",
      label: "Settings",
      caption: "Models and app",
      icon: Settings,
      active: (pathname: string) => pathname.startsWith("/settings"),
    },
  ];

  const projectsNavItems = [
    {
      href: "/projects",
      label: "Workspace",
      caption: "Research workspace",
      icon: LayoutDashboard,
      active: (pathname: string) => pathname === "/projects" || pathname === "/projects/",
    },
    {
      href: "/projects/list",
      label: "Projects",
      caption: "Research projects",
      icon: FolderKanban,
      active: (pathname: string) => pathname.startsWith("/projects/list"),
    },
    {
      href: "/projects/library",
      label: "Library",
      caption: "Global sources",
      icon: Library,
      active: (pathname: string) => pathname.startsWith("/projects/library"),
    },
    {
      href: "/projects/runs",
      label: "Runs",
      caption: "Prompt pack runs",
      icon: Activity,
      active: (pathname: string) => pathname.startsWith("/projects/runs"),
    },
    {
      href: "/diagnostics",
      label: "Diagnostics",
      caption: "Local health",
      icon: ShieldCheck,
      active: (pathname: string) => pathname.startsWith("/diagnostics"),
    },
    {
      href: "/settings",
      label: "Settings",
      caption: "Models and app",
      icon: Settings,
      active: (pathname: string) => pathname.startsWith("/settings"),
    },
  ];

  let currentNavItems = $derived(uiMode === "projects" ? projectsNavItems : legacyNavItems);
</script>

<svelte:head>
  <meta name="color-scheme" content={theme === "dark" ? "dark" : "light"} />
</svelte:head>

<svelte:window onkeydown={handleShellKeydown} />

<div class="app" data-theme={theme}>
  <ToastHost />
  <ModalHost />
  <div class="shell">
    <AppSidebar
      navItems={currentNavItems}
      pathname={page.url.pathname}
      collapsed={sidebarCollapsed}
      mobileOpen={mobileSidebarOpen}
      onToggleCollapsed={toggleSidebarCollapsed}
      onCloseMobile={closeMobileSidebar}
      {uiMode}
      onToggleUiMode={() => {
        const nextMode = uiMode === "legacy" ? "projects" : "legacy";
        setUiMode(nextMode);
        if (nextMode === "projects") {
          goto("/projects");
        } else {
          goto("/analysis");
        }
      }}
    />

    <main class="workspace">
      <div class="workspace-topbar">
        <div class="workspace-topbar-main">
          <Button
            className="mobile-menu-button"
            variant="ghost"
            iconOnly
            ariaLabel="Open navigation"
            ariaControls="app-sidebar"
            ariaExpanded={mobileSidebarOpen}
            onclick={() => (mobileSidebarOpen = true)}
          >
            <Menu size={17} aria-hidden="true" />
          </Button>
          <div class="workspace-route">
            <span class="workspace-kicker">Current space</span>
            <strong>
              {#if page.url.pathname.startsWith("/projects")}
                Research projects
              {:else if page.url.pathname.startsWith("/analysis")}
                Analysis workspace
              {:else if page.url.pathname.startsWith("/accounts") || page.url.pathname.startsWith("/auth")}
                Source access
              {:else if page.url.pathname.startsWith("/diagnostics")}
                Diagnostics
              {:else if page.url.pathname.startsWith("/settings")}
                Settings
              {:else}
                Extractum
              {/if}
            </strong>
          </div>
        </div>
        <div class="workspace-topbar-actions">
          <div class="workspace-topbar-meta">
            <span class="workspace-badge">Local-first desktop</span>
            <span class="workspace-badge">Tauri + Svelte</span>
          </div>
          <Button
            className="theme-toggle"
            variant="secondary"
            iconOnly
            ariaLabel={theme === "light" ? "Switch to dark theme" : "Switch to light theme"}
            title={theme === "light" ? "Dark theme" : "Light theme"}
            type="button"
            onclick={toggleTheme}
          >
            {#if theme === "light"}
              <Moon size={15} aria-hidden="true" />
            {:else}
              <Sun size={15} aria-hidden="true" />
            {/if}
          </Button>
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
    gap: 0.8rem;
    min-height: 2.25rem;
    padding: 0.15rem 0.15rem 0.35rem;
    border-bottom: 1px solid color-mix(in srgb, var(--border) 76%, transparent);
  }

  .workspace-topbar-main,
  .workspace-topbar-actions {
    display: flex;
    align-items: center;
    gap: 0.7rem;
    min-width: 0;
  }

  .workspace-topbar-actions {
    justify-content: flex-end;
  }

  :global(.mobile-menu-button.ui-button) {
    display: none;
    flex: 0 0 auto;
  }

  :global(.theme-toggle) {
    flex: 0 0 auto;
  }

  .workspace-route {
    display: flex;
    flex-direction: column;
    gap: 0.08rem;
    min-width: 0;
  }

  .workspace-route strong {
    font-size: 0.96rem;
  }

  .workspace-kicker {
    font-size: 0.68rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--muted);
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
      display: block;
      min-height: 100vh;
    }

    .workspace {
      padding: 1rem;
    }

    .workspace-topbar {
      align-items: center;
    }

    .workspace-topbar-main {
      flex: 1;
    }

    :global(.mobile-menu-button.ui-button) {
      display: inline-flex;
    }

    .workspace-topbar-actions {
      gap: 0.45rem;
    }

    .workspace-topbar-meta {
      display: none;
    }

    :global(.page-hero) {
      flex-direction: column;
      align-items: stretch;
    }

    :global(.page-grid) {
      grid-template-columns: 1fr;
    }
  }
</style>
