<script lang="ts">
  import type { ProjectRailRow } from "$lib/ui/research-projects-rail";

  let { row }: { row: ProjectRailRow } = $props();
</script>

<div class="project-row" data-status={row.status}>
  <span class="project-row__name">{row.name}</span>
  <span class="project-row__meta">{row.meta}</span>
</div>

<style>
  .project-row {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 7px 10px;
    border-radius: 6px;
  }

  .project-row__name {
    font: 600 12.5px/1.2 var(--extractum-font);
    color: var(--extractum-text);
  }

  .project-row__meta {
    font: 400 11px/1.2 var(--extractum-font);
    color: var(--extractum-muted-2);
  }
</style>
