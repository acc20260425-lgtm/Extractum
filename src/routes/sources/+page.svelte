<script lang="ts">
  import { goto } from "$app/navigation";
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import StatusMessage from "$lib/components/ui/StatusMessage.svelte";
</script>

<section class="page-shell">
  <header class="page-hero">
    <div class="page-hero-copy">
      <span class="page-eyebrow">Compatibility route</span>
      <h1>Sources</h1>
      <p>
        Source browsing and sync now live inside the analysis workspace. This route stays as a
        lightweight fallback while older entry points are still in use.
      </p>
    </div>
    <div class="page-hero-meta">
      <Badge variant="info">Workspace-first</Badge>
      <Badge>Compatibility view</Badge>
    </div>
  </header>

  <StatusMessage className="page-status">
    Use the left rail in `Analysis` to switch sources, sync them, and inspect recent messages.
  </StatusMessage>

  <div class="page-grid">
    <div class="page-stack">
      <section class="desk-panel">
        <div class="panel-header">
          <div class="panel-header-copy">
            <span class="page-eyebrow">Recommended path</span>
            <h2>Main workspace</h2>
            <p>
              Open the three-pane workspace to browse sources, inspect recent messages, run reports,
              and continue grounded chat in one place.
            </p>
          </div>
        </div>
        <div class="actions">
          <Button onclick={() => goto("/analysis")}>Open analysis workspace</Button>
        </div>
      </section>
    </div>

    <div class="page-stack">
      <section class="desk-panel desk-panel-subtle">
        <div class="panel-header">
          <div class="panel-header-copy">
            <span class="page-eyebrow">Access management</span>
            <h2>Accounts and auth</h2>
            <p>
              Need to add or reconnect Telegram accounts before syncing? Open account management.
            </p>
          </div>
        </div>
        <div class="actions">
          <Button variant="secondary" onclick={() => goto("/accounts")}>Open accounts</Button>
        </div>
      </section>
    </div>
  </div>

  <section class="desk-panel desk-panel-subtle">
    <EmptyState
      title="Sources moved into the workspace"
      description="This route is intentionally lightweight now. The full source workflow lives in Analysis."
    />
  </section>
</section>

<style>
  :global(.page-status) {
    margin-bottom: 0;
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
</style>
