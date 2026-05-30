<script lang="ts">
  import Badge from "$lib/components/ui/Badge.svelte";
  import type { AnalysisSourceGroup } from "$lib/types/analysis";

  let {
    group,
    formatTimestamp,
  }: {
    group: AnalysisSourceGroup;
    formatTimestamp: (value: number | null) => string;
  } = $props();

  const sortedMembers = $derived([...group.members].sort((left, right) =>
    (left.source_title ?? `Source ${left.source_id}`).localeCompare(
      right.source_title ?? `Source ${right.source_id}`,
      undefined,
      { sensitivity: "base", numeric: true },
    ),
  ));
</script>

<section class="source-group-metadata-view" aria-label="Source group metadata">
  <div class="metadata-header">
    <div>
      <span class="eyebrow">Group metadata</span>
      <h3>{group.name}</h3>
    </div>
    <Badge variant="info">{group.members.length} sources</Badge>
  </div>

  <section class="metadata-section" aria-labelledby="source-group-summary-title">
    <h4 id="source-group-summary-title">Summary</h4>
    <dl class="metadata-grid">
      <div>
        <dt>Name</dt>
        <dd>{group.name}</dd>
      </div>
      <div>
        <dt>Provider type</dt>
        <dd>{group.source_type}</dd>
      </div>
      <div>
        <dt>Members</dt>
        <dd>{group.members.length}</dd>
      </div>
      <div>
        <dt>Total indexed items</dt>
        <dd>{group.members.reduce((total, member) => total + member.item_count, 0)}</dd>
      </div>
      <div>
        <dt>Created</dt>
        <dd>{formatTimestamp(group.created_at)}</dd>
      </div>
      <div>
        <dt>Updated</dt>
        <dd>{formatTimestamp(group.updated_at)}</dd>
      </div>
    </dl>
  </section>

  <section class="metadata-section" aria-labelledby="source-group-members-title">
    <h4 id="source-group-members-title">Members</h4>
    <ul class="member-list">
      {#each sortedMembers as member (member.source_id)}
        <li>
          <span>{member.source_title ?? `Source ${member.source_id}`}</span>
          <Badge variant="neutral">{member.item_count} items</Badge>
        </li>
      {/each}
    </ul>
  </section>
</section>

<style>
  .source-group-metadata-view {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    min-width: 0;
  }

  .metadata-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 0.75rem;
    flex-wrap: wrap;
  }

  .metadata-header h3,
  .metadata-section h4,
  .metadata-grid,
  .metadata-grid dd {
    margin: 0;
  }

  .metadata-header h3 {
    font-size: 1.05rem;
  }

  .eyebrow {
    color: var(--muted);
    font-size: 0.68rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  .metadata-section {
    display: flex;
    flex-direction: column;
    gap: 0.65rem;
    min-width: 0;
    padding-top: 0.85rem;
    border-top: 1px solid var(--border);
  }

  .metadata-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(13rem, 1fr));
    gap: 0.7rem 1rem;
  }

  .metadata-grid dt {
    color: var(--muted);
    font-size: 0.72rem;
    line-height: 1.35;
  }

  .metadata-grid dd {
    color: var(--text);
    font-size: 0.9rem;
    line-height: 1.45;
    overflow-wrap: anywhere;
  }

  .member-list {
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .member-list li {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 0.75rem;
    padding: 0.55rem 0;
    border-bottom: 1px solid color-mix(in srgb, var(--border) 72%, transparent);
  }

  .member-list span {
    overflow-wrap: anywhere;
  }
</style>
