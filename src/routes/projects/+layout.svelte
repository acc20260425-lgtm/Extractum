<script lang="ts">
  import IconRail from "$lib/components/research-projects/IconRail.svelte";

  let { children } = $props();
</script>

<section data-ui-route-shell="projects" class="projects-route-shell">
  <aside data-ui-region="icon-rail" class="icon-rail">
    <IconRail />
  </aside>

  <div class="projects-route-content">
    {@render children()}
  </div>
</section>

<style>
  .projects-route-shell {
    display: grid;
    grid-template-columns: 56px minmax(0, 1fr);
    min-height: calc(100vh - 68px);
    border: 1px solid var(--extractum-border);
    background: var(--extractum-surface);
  }

  .icon-rail {
    min-width: 0;
    min-height: 0;
    border-right: 1px solid var(--extractum-border);
    background: var(--extractum-surface-raised);
  }

  .projects-route-content {
    min-width: 0;
    min-height: 0;
  }
</style>
