<script lang="ts">
  import {
    Activity,
    FolderKanban,
    Library,
    Settings,
    ShieldCheck,
  } from "@lucide/svelte";

  const items = [
    { href: "/projects", label: "Projects", icon: FolderKanban, active: true },
    { href: "/projects#library", label: "Library", icon: Library, active: false },
    { href: "/projects#runs", label: "Runs", icon: Activity, active: false },
    { href: "/diagnostics", label: "Diagnostics", icon: ShieldCheck, active: false },
    { href: "/settings", label: "Settings", icon: Settings, active: false },
  ];
</script>

<nav class="icon-rail-nav" aria-label="Research project sections">
  {#each items as item (item.href)}
    <a class:active={item.active} href={item.href} title={item.label} aria-label={item.label}>
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
