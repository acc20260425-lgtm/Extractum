<script lang="ts">
  import type { ResearchProjectView } from "$lib/ui/research-projects-model";

  let {
    project,
    connectedCount,
    materialCount,
    libraryCount,
  }: {
    project: ResearchProjectView | null;
    connectedCount: number;
    materialCount: number;
    libraryCount: number;
  } = $props();

  const stats = $derived([
    { label: "Подключено", value: connectedCount },
    { label: "Материалы", value: materialCount },
    { label: "В библиотеке", value: libraryCount },
    { label: "В проекте", value: project?.sourceCount ?? 0 },
  ]);
</script>

<section class="source-summary" aria-label="Project source summary">
  {#each stats as stat (stat.label)}
    <div class="extractum-stat-card">
      <span>{stat.label}</span>
      <strong>{stat.value}</strong>
    </div>
  {/each}
</section>

<style>
  .source-summary {
    display: grid;
    grid-template-columns: repeat(4, minmax(120px, 1fr));
    gap: 10px;
  }

  .source-summary span {
    display: block;
    color: var(--extractum-muted);
    font-size: 11px;
    text-transform: uppercase;
  }

  .source-summary strong {
    display: block;
    margin-top: 4px;
    font-size: 18px;
  }
</style>
