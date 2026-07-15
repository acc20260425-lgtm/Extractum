<script lang="ts">
  import { page } from "$app/state";
  import Activity from "@lucide/svelte/icons/activity";
  import FolderKanban from "@lucide/svelte/icons/folder-kanban";
  import Library from "@lucide/svelte/icons/library";
  import Settings from "@lucide/svelte/icons/settings";
  import ShieldCheck from "@lucide/svelte/icons/shield-check";

  const items = [
    { href: "/projects", label: "Projects", icon: FolderKanban },
    { href: "/projects/library", label: "Library", icon: Library },
    { href: "/projects/runs", label: "Runs", icon: Activity },
    { href: "/diagnostics", label: "Diagnostics", icon: ShieldCheck },
    { href: "/settings", label: "Settings", icon: Settings },
  ];

  function isActive(href: string) {
    if (href === "/projects") return page.url.pathname === "/projects";
    if (href === "/projects/library") return page.url.pathname === "/projects/library";
    if (href === "/projects/runs") return page.url.pathname === "/projects/runs";
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
