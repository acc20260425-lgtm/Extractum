<script lang="ts">
  import { ExternalLink, Link2, PlayCircle, RefreshCw } from "@lucide/svelte";
  import { ExtractumButton, ProviderBadge, StatusBadge } from "$lib/components/extractum-ui";
  import type { LibrarySourceView } from "$lib/ui/research-projects-model";

  let { selectedSource }: { selectedSource: LibrarySourceView | null } = $props();
</script>

<aside data-ui-region="library-inspector" class="library-inspector" aria-label="Library source inspector">
  {#if selectedSource}
    <header class="inspector-header">
      <div>
        <p class="eyebrow">Selected source</p>
        <h2>{selectedSource.title}</h2>
      </div>
      <ProviderBadge provider={selectedSource.provider} />
    </header>

    <div class="status-row">
      <StatusBadge status={selectedSource.status} />
      {#if selectedSource.alreadyConnected}
        <span class="meta-pill">Connected</span>
      {/if}
    </div>

    <dl class="meta-list">
      <div><dt>Source ID</dt><dd>{selectedSource.sourceId}</dd></div>
      <div><dt>Projects</dt><dd>{selectedSource.projectCount}</dd></div>
      <div><dt>Local copy</dt><dd>{selectedSource.localCopyLabel ?? "No local copy"}</dd></div>
      <div><dt>Last collected</dt><dd>{selectedSource.lastCollectedLabel ?? "Never"}</dd></div>
    </dl>

    {#if selectedSource.disabledReason}
      <p class="notice">{selectedSource.disabledReason}</p>
    {/if}

    <div class="commands" aria-label="Inspector commands">
      <ExtractumButton variant="outline"><ExternalLink size={14} aria-hidden="true" />Open</ExtractumButton>
      <ExtractumButton variant="outline"><RefreshCw size={14} aria-hidden="true" />Sync</ExtractumButton>
      <ExtractumButton variant="outline"><Link2 size={14} aria-hidden="true" />Connect</ExtractumButton>
      <ExtractumButton variant="outline"><PlayCircle size={14} aria-hidden="true" />Run report</ExtractumButton>
    </div>
  {:else}
    <div class="empty-state">
      <p class="eyebrow">Inspector</p>
      <h2>No source selected</h2>
      <p>Select a source row to inspect metadata and available commands.</p>
    </div>
  {/if}
</aside>

<style>
  .library-inspector {
    min-width: 0;
    min-height: 0;
    padding: 14px;
    overflow: auto;
    background: var(--extractum-surface-raised);
  }

  .inspector-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
  }

  .eyebrow {
    margin: 0 0 6px;
    color: var(--extractum-muted);
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
  }

  h2 {
    margin: 0;
    font-size: 16px;
    font-weight: 700;
    line-height: 1.25;
  }

  .status-row,
  .commands {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-top: 12px;
  }

  .commands {
    display: grid;
    grid-template-columns: 1fr 1fr;
  }

  .meta-list {
    display: grid;
    gap: 8px;
    margin: 14px 0;
  }

  .meta-list div {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    border-bottom: 1px solid var(--extractum-border);
    padding-bottom: 6px;
  }

  dt {
    color: var(--extractum-muted);
    font-size: 12px;
  }

  dd {
    margin: 0;
    font-size: 12px;
    text-align: right;
  }

  .meta-pill,
  .notice {
    border: 1px solid var(--extractum-border);
    border-radius: var(--extractum-radius);
    padding: 4px 7px;
    color: var(--extractum-muted);
    font-size: 12px;
  }

  .notice {
    margin: 12px 0 0;
  }

  .empty-state {
    display: grid;
    min-height: 220px;
    align-content: center;
    gap: 8px;
    color: var(--extractum-muted);
  }
</style>
