<script lang="ts">
  import ProjectRunsScreen from "$lib/components/research-projects/ProjectRunsScreen.svelte";
  import { PROJECT_RUNS_PAGE } from "./project-runs-page";
</script>

<section data-ui-route={PROJECT_RUNS_PAGE.id}>
  <ProjectRunsScreen />
</section>
