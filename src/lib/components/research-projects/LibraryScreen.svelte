<script lang="ts">
  import type { ResearchProjectsWorkflowState } from "$lib/ui/research-projects-workflow";

  let {
    state,
    onRefresh,
  }: {
    state: ResearchProjectsWorkflowState;
    onRefresh: () => void | Promise<void>;
  } = $props();
</script>

<div data-ui-screen="library" class="library-screen">
  <button type="button" onclick={onRefresh}>Refresh</button>
  <span>{state.librarySources.length} sources</span>
</div>
