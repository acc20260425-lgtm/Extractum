<script lang="ts">
  import { PanelRightOpen } from "@lucide/svelte";
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import StatusMessage from "$lib/components/ui/StatusMessage.svelte";

  onMount(() => {
    void goto("/analysis", { replaceState: true });
  });
</script>

<section class="page-shell">
  <StatusMessage className="page-status">
    Redirecting to the analysis workspace...
  </StatusMessage>

  <div class="actions">
    <Button onclick={() => goto("/analysis", { replaceState: true })}>
      <PanelRightOpen size={15} aria-hidden="true" /> Open analysis workspace
    </Button>
  </div>
</section>

<style>
  :global(.page-status) {
    margin-bottom: 0;
  }

  .actions {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
  }
</style>
