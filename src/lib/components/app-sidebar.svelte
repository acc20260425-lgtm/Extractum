<script lang="ts">
  import { ChevronLeft, ChevronRight, Plus, FolderKanban, LayoutDashboard } from "@lucide/svelte";
  import { tick, type Component, onMount } from "svelte";
  import { goto } from "$app/navigation";
  import Button from "$lib/components/ui/Button.svelte";
  import { projectsSharedState } from "$lib/projects-shared.svelte";

  type NavIcon = Component<{
    size?: number;
    "aria-hidden"?: boolean | "true";
  }>;

  export type AppSidebarNavItem = {
    href: string;
    label: string;
    caption: string;
    icon: NavIcon;
    active: (pathname: string) => boolean;
  };

  let {
    navItems,
    pathname,
    collapsed,
    mobileOpen,
    onToggleCollapsed,
    onCloseMobile,
    uiMode = "legacy",
    onToggleUiMode,
  }: {
    navItems: AppSidebarNavItem[];
    pathname: string;
    collapsed: boolean;
    mobileOpen: boolean;
    onToggleCollapsed: () => void;
    onCloseMobile: () => void;
    uiMode?: "legacy" | "projects";
    onToggleUiMode: () => void;
  } = $props();

  let sidebarElement: HTMLElement | undefined;
  let previousMobileOpen = false;

  onMount(() => {
    if (uiMode === "projects" && !projectsSharedState.initialized) {
      void projectsSharedState.load();
    }
  });

  $effect(() => {
    if (uiMode === "projects" && !projectsSharedState.initialized) {
      void projectsSharedState.load();
    }
  });

  $effect(() => {
    // Equivalent to tabindex={mobileOpen ? -1 : undefined}, without leaving a focusable aside closed.
    if (mobileOpen) {
      sidebarElement?.setAttribute("tabindex", "-1");
    } else {
      sidebarElement?.removeAttribute("tabindex");
    }

    if (mobileOpen && !previousMobileOpen) {
      tick().then(() => sidebarElement?.focus());
    }
    previousMobileOpen = mobileOpen;
  });

  function handleNavClick() {
    if (mobileOpen) {
      onCloseMobile();
    }
  }
</script>

{#if mobileOpen}
  <button
    class="sidebar-overlay"
    type="button"
    aria-label="Close navigation"
    onclick={onCloseMobile}
  ></button>
{/if}

<aside
  bind:this={sidebarElement}
  id="app-sidebar"
  class="sidebar"
  class:collapsed={!mobileOpen && collapsed}
  class:mobile-open={mobileOpen}
  role="navigation"
  aria-label="Primary navigation"
>
  <div class="sidebar-header">
    <a class="brand" href="/analysis" onclick={handleNavClick} title="Extractum">
      <span class="brand-mark" aria-hidden="true">E</span>
      {#if !collapsed || mobileOpen}
        <span class="brand-copy">
          <strong>Extractum</strong>
          <small>Research workspace</small>
        </span>
      {/if}
    </a>

    <Button
      className="sidebar-collapse"
      variant="ghost"
      iconOnly
      ariaLabel={collapsed && !mobileOpen ? "Expand navigation" : "Collapse navigation"}
      title={collapsed && !mobileOpen ? "Expand navigation" : "Collapse navigation"}
      onclick={onToggleCollapsed}
    >
      {#if collapsed && !mobileOpen}
        <ChevronRight size={16} aria-hidden="true" />
      {:else}
        <ChevronLeft size={16} aria-hidden="true" />
      {/if}
    </Button>
  </div>

  <nav class="sidebar-nav" aria-label="Primary navigation">
    {#each navItems as item (item.href)}
      {@const NavIcon = item.icon}
      <div class="nav-item-container">
        <a
          href={item.href}
          class:active={item.active(pathname)}
          aria-current={item.active(pathname) ? "page" : undefined}
          aria-label={item.label}
          title={!mobileOpen && collapsed ? item.label : undefined}
          onclick={handleNavClick}
        >
          <span class="nav-row">
            <NavIcon size={16} aria-hidden="true" />
            {#if !collapsed || mobileOpen}
              <span class="nav-label">{item.label}</span>
            {/if}
          </span>
          {#if !collapsed || mobileOpen}
            <span class="nav-caption">{item.caption}</span>
          {/if}
        </a>

        {#if item.href === "/projects" && uiMode === "projects" && (!collapsed || mobileOpen)}
          <div class="projects-subnav">
            {#each projectsSharedState.projects as project (project.id)}
              <button
                type="button"
                class="subnav-project-item"
                class:active={projectsSharedState.selectedProjectId === project.id}
                onclick={(e) => {
                  e.preventDefault();
                  e.stopPropagation();
                  projectsSharedState.selectedProjectId = project.id;
                  if (pathname !== "/projects") {
                    goto("/projects");
                  }
                }}
              >
                <span class="subnav-project-dot" class:running={project.status === "running"} class:ready={project.status === "ready"}></span>
                <span class="subnav-project-title" title={project.title}>{project.title}</span>
              </button>
            {/each}
            <button
              type="button"
              class="subnav-add-btn"
              onclick={(e) => {
                e.preventDefault();
                e.stopPropagation();
                projectsSharedState.showCreateDialog = true;
                if (pathname !== "/projects") {
                  goto("/projects");
                }
              }}
            >
              <Plus size={12} aria-hidden="true" />
              <span>Create project</span>
            </button>
          </div>
        {/if}
      </div>
    {/each}
  </nav>

  <div class="sidebar-footer">
    {#if !collapsed || mobileOpen}
      <div class="footer-copy">
        <span class="footer-label">Workspace mode</span>
        <strong>{uiMode === "projects" ? "Research Projects" : "NotebookLM x Telegram"}</strong>
      </div>
      <button
        class="mode-switch-button"
        type="button"
        onclick={onToggleUiMode}
      >
        {uiMode === "projects" ? "Switch to Legacy UI" : "Try New Projects UI"}
      </button>
    {:else}
      <button
        class="mode-switch-button collapsed"
        type="button"
        onclick={onToggleUiMode}
        title={uiMode === "projects" ? "Switch to Legacy UI" : "Try New Projects UI"}
        aria-label={uiMode === "projects" ? "Switch to Legacy UI" : "Try New Projects UI"}
      >
        {#if uiMode === "projects"}
          <LayoutDashboard size={16} aria-hidden="true" />
        {:else}
          <FolderKanban size={16} aria-hidden="true" />
        {/if}
      </button>
    {/if}
  </div>
</aside>

<style>
  .sidebar-overlay {
    display: none;
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
    transition: width 0.18s ease, flex-basis 0.18s ease, transform 0.18s ease;
  }

  .sidebar.collapsed {
    width: 64px;
    flex-basis: 64px;
    padding: 0.85rem 0.5rem;
    align-items: center;
  }

  .sidebar-header {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.2rem 0.2rem 0;
    min-width: 0;
  }

  .sidebar.collapsed .sidebar-header {
    flex-direction: column-reverse;
    padding-inline: 0;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: 0.65rem;
    color: inherit;
    text-decoration: none;
    padding: 0.5rem 0.55rem;
    border-radius: 12px;
    min-width: 0;
  }

  .brand:hover {
    background: color-mix(in srgb, var(--panel-hover) 68%, transparent);
  }

  .sidebar.collapsed .brand {
    justify-content: center;
    padding: 0.45rem;
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
    flex: 0 0 auto;
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

  :global(.sidebar-collapse) {
    flex: 0 0 auto;
  }

  .sidebar-nav {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    min-width: 0;
  }

  .sidebar.collapsed .sidebar-nav {
    width: 100%;
    align-items: center;
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
    min-width: 0;
  }

  .sidebar.collapsed .sidebar-nav a {
    width: 2.45rem;
    min-height: 2.45rem;
    align-items: center;
    justify-content: center;
    padding: 0;
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

  .sidebar.collapsed .sidebar-footer {
    align-items: center;
    padding-inline: 0;
    width: 100%;
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

  .footer-label {
    font-size: 0.68rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--muted);
  }

  .projects-subnav {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    padding-left: 0.5rem;
    margin-top: 0.2rem;
    border-left: 1px solid var(--border);
    margin-left: 1.25rem;
  }

  .subnav-project-item {
    display: flex;
    align-items: center;
    gap: 0.45rem;
    padding: 0.35rem 0.5rem;
    border-radius: 8px;
    color: var(--muted);
    font-size: 0.8rem;
    font-weight: 500;
    text-align: left;
    background: transparent;
    border: none;
    cursor: pointer;
    width: 100%;
    min-width: 0;
    transition: color 0.15s, background 0.15s;
  }

  .subnav-project-item:hover {
    color: var(--text);
    background: color-mix(in srgb, var(--panel-hover) 50%, transparent);
  }

  .subnav-project-item.active {
    color: var(--primary);
    background: color-mix(in srgb, var(--primary) 10%, transparent);
    font-weight: 600;
  }

  .subnav-project-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--border);
    flex-shrink: 0;
  }

  .subnav-project-dot.running {
    background: var(--primary);
    box-shadow: 0 0 8px var(--primary);
    animation: pulse 2s infinite;
  }

  .subnav-project-dot.ready {
    background: #10b981;
  }

  .subnav-project-title {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex: 1;
  }

  .subnav-add-btn {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0.35rem 0.5rem;
    border-radius: 8px;
    color: var(--muted);
    font-size: 0.78rem;
    background: transparent;
    border: 1px dashed var(--border);
    cursor: pointer;
    margin-top: 0.25rem;
    width: fit-content;
    transition: border-color 0.15s, color 0.15s;
  }

  .subnav-add-btn:hover {
    border-color: var(--primary);
    color: var(--primary);
  }

  .mode-switch-button {
    width: 100%;
    padding: 0.45rem 0.6rem;
    border-radius: 8px;
    background: linear-gradient(135deg, var(--primary), color-mix(in srgb, var(--primary) 80%, black));
    color: white;
    font-size: 0.8rem;
    font-weight: 600;
    border: none;
    cursor: pointer;
    box-shadow: 0 4px 12px rgba(47, 109, 234, 0.15);
    transition: transform 0.1s, opacity 0.15s;
    text-align: center;
  }

  .mode-switch-button:hover {
    opacity: 0.95;
    transform: translateY(-1px);
  }

  .mode-switch-button:active {
    transform: translateY(0);
  }

  .mode-switch-button.collapsed {
    width: 2.45rem;
    height: 2.45rem;
    padding: 0;
    border-radius: 12px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }

  @keyframes pulse {
    0% { opacity: 0.5; }
    50% { opacity: 1; }
    100% { opacity: 0.5; }
  }

  @media (max-width: 820px) {
    .sidebar-overlay {
      display: block;
      position: fixed;
      inset: 0;
      z-index: 40;
      background: rgba(15, 23, 42, 0.34);
      border: 0;
      border-radius: 0;
      padding: 0;
      cursor: default;
    }

    .sidebar {
      position: fixed;
      inset: 0 auto 0 0;
      z-index: 41;
      width: min(214px, calc(100vw - 3rem));
      max-width: calc(100vw - 3rem);
      height: 100vh;
      transform: translateX(-100%);
      border-bottom: none;
      box-shadow: 18px 0 44px rgba(15, 23, 42, 0.16);
      overflow: auto;
    }

    .sidebar.mobile-open {
      transform: translateX(0);
    }

    .sidebar.collapsed {
      width: min(214px, calc(100vw - 3rem));
      flex-basis: auto;
      align-items: stretch;
      padding: 0.85rem 0.7rem 0.85rem 0.85rem;
    }

    .sidebar.collapsed .sidebar-header {
      flex-direction: row;
      padding: 0.2rem 0.2rem 0;
    }

    :global(.sidebar-collapse) {
      display: none;
    }
  }
</style>
