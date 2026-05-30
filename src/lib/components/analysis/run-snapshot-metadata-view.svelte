<script lang="ts">
  import Badge from "$lib/components/ui/Badge.svelte";
  import type { RunSnapshotAvailability } from "$lib/analysis-report-canvas-state";
  import type { RunSnapshotBrowserSubject } from "$lib/source-browser-model";
  import type { SourceFilterOption, SourceReaderItem } from "$lib/source-reader-model";
  import type { AnalysisRunDetail } from "$lib/types/analysis";

  let {
    run,
    snapshot,
    readerItems,
    sourceOptions,
    snapshotAvailability,
    snapshotError = "",
    formatTimestamp,
  }: {
    run: AnalysisRunDetail;
    snapshot: RunSnapshotBrowserSubject;
    readerItems: SourceReaderItem[];
    sourceOptions: SourceFilterOption[];
    snapshotAvailability: RunSnapshotAvailability;
    snapshotError?: string;
    formatTimestamp: (value: number | null) => string;
  } = $props();
</script>

<section class="run-snapshot-metadata-view" aria-label="Run snapshot metadata">
  <div class="metadata-heading">
    <div>
      <span>Run snapshot</span>
      <h3>{run.scope_label}</h3>
    </div>
    <Badge variant="success">{snapshotAvailability}</Badge>
  </div>

  <section class="metadata-section" aria-label="Summary">
    <h4>Summary</h4>
    <dl>
      <dt>Run id</dt>
      <dd>{run.id}</dd>
      <dt>Run title</dt>
      <dd>{run.scope_label}</dd>
      <dt>Scope type</dt>
      <dd>{snapshot.scopeType}</dd>
      <dt>Reader kind</dt>
      <dd>{snapshot.readerKind}</dd>
      <dt>Loaded rows</dt>
      <dd>{readerItems.length}</dd>
      <dt>Source type</dt>
      <dd>{snapshot.sourceType ?? "n/a"}</dd>
      <dt>Source subtype</dt>
      <dd>{snapshot.sourceSubtype ?? "n/a"}</dd>
      <dt>Created</dt>
      <dd>{formatTimestamp(run.created_at)}</dd>
      <dt>Completed</dt>
      <dd>{formatTimestamp(run.completed_at)}</dd>
    </dl>
  </section>

  {#if sourceOptions.length > 0}
    <section class="metadata-section" aria-label="Snapshot sources">
      <h4>Snapshot sources</h4>
      <ul>
        {#each sourceOptions as option (option.id)}
          <li>
            <span>{option.label}</span>
            <Badge variant="neutral">{option.count} rows</Badge>
          </li>
        {/each}
      </ul>
    </section>
  {/if}

  {#if snapshotError}
    <section class="metadata-section" aria-label="Snapshot error">
      <h4>Snapshot error</h4>
      <p>{snapshotError}</p>
    </section>
  {/if}
</section>

<style>
  .run-snapshot-metadata-view,
  .metadata-section,
  .metadata-heading {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    min-width: 0;
  }

  .metadata-heading {
    flex-direction: row;
    justify-content: space-between;
    align-items: flex-start;
  }

  .metadata-heading span {
    color: var(--muted);
    font-size: 0.75rem;
    text-transform: uppercase;
  }

  h3,
  h4,
  p {
    margin: 0;
  }

  dl {
    display: grid;
    grid-template-columns: minmax(8rem, 0.35fr) 1fr;
    gap: 0.5rem 0.8rem;
    margin: 0;
  }

  dt {
    color: var(--muted);
  }

  dd {
    margin: 0;
    min-width: 0;
    overflow-wrap: anywhere;
  }

  ul {
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
    list-style: none;
    margin: 0;
    padding: 0;
  }

  li {
    display: flex;
    justify-content: space-between;
    gap: 0.75rem;
    align-items: center;
  }

  p {
    color: var(--muted);
    overflow-wrap: anywhere;
  }

  @media (max-width: 760px) {
    .metadata-heading,
    li {
      flex-direction: column;
      align-items: flex-start;
    }

    dl {
      grid-template-columns: 1fr;
    }
  }
</style>
