<script lang="ts">
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
</script>

<section class="source-group-activity-view" aria-label="Source group activity">
  <EmptyState description="Group activity is not available yet. Source jobs are still tracked per source." />
</section>

<style>
  .source-group-activity-view {
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
    min-width: 0;
  }
</style>
