<script lang="ts">
  let { children } = $props();
</script>

<section data-ui-route-shell="projects" class="projects-route-shell">
  <div class="projects-route-content">
    {@render children()}
  </div>
</section>

<style>
  .projects-route-shell {
    display: flex;
    flex-direction: column;
    min-height: calc(100vh - 68px);
    background: var(--extractum-surface);
  }

  .projects-route-content {
    min-width: 0;
    min-height: 0;
    flex: 1;
    display: flex;
    flex-direction: column;
  }
</style>
