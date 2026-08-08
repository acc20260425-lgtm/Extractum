<script lang="ts">
  import { page } from "$app/state";
  import Activity from "@lucide/svelte/icons/activity";
  import FolderKanban from "@lucide/svelte/icons/folder-kanban";
  import Library from "@lucide/svelte/icons/library";
  import Settings from "@lucide/svelte/icons/settings";
  import ShieldCheck from "@lucide/svelte/icons/shield-check";
  import { PROJECT_ICON_RAIL_ROUTES, PROJECT_RUNS_PAGE } from "$lib/ui/research-projects-navigation";
  // Retained legacy marker; PROJECT_ICON_RAIL_ROUTES owns href: "/projects/runs".

  const icons = {
    projects: FolderKanban,
    "library-prototype": Library,
    "project-runs": Activity,
    diagnostics: ShieldCheck,
    settings: Settings,
  };
  const items = PROJECT_ICON_RAIL_ROUTES.map((item) => ({
    ...item,
    icon: icons[item.id as keyof typeof icons],
  }));

  function isActive(href: string) {
    if (href === "/projects") return page.url.pathname === "/projects";
    if (href === "/projects/library") return page.url.pathname === "/projects/library";
    if (href === PROJECT_RUNS_PAGE.href) return page.url.pathname === PROJECT_RUNS_PAGE.href;
    return page.url.pathname === href;
  }
</script>

<nav class="icon-rail-nav" aria-label="Research project sections">
  {#each items as item (item.href)}
    <a
      class:active={isActive(item.href)}
      href={item.href}
      title={item.label}
      aria-label={item.label}
      aria-current={isActive(item.href) ? "page" : undefined}
    >
      <item.icon size={18} aria-hidden="true" />
    </a>
  {/each}
</nav>

<style>
  .icon-rail-nav {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 10px 8px;
  }

  .icon-rail-nav a {
    display: inline-flex;
    width: 36px;
    height: 36px;
    align-items: center;
    justify-content: center;
    border-radius: var(--extractum-radius);
    color: var(--extractum-muted);
    text-decoration: none;
  }

  .icon-rail-nav a:hover,
  .icon-rail-nav a.active {
    background: color-mix(in srgb, var(--extractum-primary) 12%, transparent);
    color: var(--extractum-primary);
  }
</style>
