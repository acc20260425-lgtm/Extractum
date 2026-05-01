<script lang="ts">
  import { goto } from "$app/navigation";
  import Card from "$lib/components/ui/Card.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import StatusMessage from "$lib/components/ui/StatusMessage.svelte";
</script>

<h1>Sources</h1>

<StatusMessage className="page-status">
  Source browsing and sync are now part of the main analysis workspace. This route stays available
  as a compatibility view while the interface migration settles.
</StatusMessage>

<div class="compat-grid">
  <Card className="compat-card">
    <h3>Main Workspace</h3>
    <p>
      Open the new three-pane workspace to browse synced sources, inspect recent messages, run
      reports, and continue report chat in one place.
    </p>
    <div class="actions">
      <Button onclick={() => goto("/analysis")}>Open analysis workspace</Button>
    </div>
  </Card>

  <Card className="compat-card">
    <h3>Accounts And Access</h3>
    <p>
      If you need to add or reconnect Telegram accounts before syncing, go to account management.
    </p>
    <div class="actions">
      <Button variant="secondary" onclick={() => goto("/accounts")}>Open accounts</Button>
    </div>
  </Card>
</div>

<Card className="compat-card">
  <EmptyState
    title="Sources moved into the workspace"
    description="Use the left rail in Analysis to switch between sources and groups, sync current sources, and inspect locally synced messages before launching a report."
  />
</Card>

<style>
  :global(.page-status) {
    margin-bottom: 1rem;
  }

  .compat-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 1rem;
    margin-bottom: 1rem;
  }

  :global(.ui-card.compat-card) {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  p {
    margin: 0;
    color: var(--muted);
    line-height: 1.55;
  }

  .actions {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  @media (max-width: 820px) {
    .compat-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
