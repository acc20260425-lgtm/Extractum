<script lang="ts">
  import type { ProjectRailRow } from "$lib/ui/research-projects-rail";

  let {
    row,
    onSelect,
  }: { row: ProjectRailRow; onSelect?: (id: number) => void } = $props();
</script>

<button
  type="button"
  class="project-row"
  data-status={row.status}
  onclick={() => onSelect?.(row.id)}
>
  <span class="project-row__top">
    <span
      class="project-row__dot"
      data-testid="project-row-status-dot"
      data-status={row.status}
    ></span>
    <span class="project-row__name">{row.name}</span>
    {#if row.pinned}
      <span class="project-row__pin" title="Закреплён" aria-label="Закреплён">
        <svg width="11" height="11" viewBox="0 0 16 16" fill="currentColor">
          <path
            d="M9.5 1.5l5 5-2 .6-2.3 2.3.5 3.2-1.8-1.8-3.3 3.3-.5.5.5-3.8L1.9 8.7l3.2.5L7.4 6.9 8 4.9z"
          />
        </svg>
      </span>
    {/if}
  </span>
  <span class="project-row__meta">{row.meta}</span>
</button>

<style>
  .project-row {
    display: flex;
    flex-direction: column;
    gap: 2px;
    width: 100%;
    padding: 7px 10px;
    border: none;
    border-radius: 6px;
    background: transparent;
    text-align: left;
    cursor: pointer;
  }

  .project-row:hover {
    background: var(--extractum-surface-subtle);
  }

  .project-row__top {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .project-row__name {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font: 600 12.5px/1.2 var(--extractum-font);
    color: var(--extractum-text);
  }

  .project-row__pin {
    flex-shrink: 0;
    display: inline-flex;
    color: var(--extractum-muted-2);
  }

  .project-row__dot {
    flex-shrink: 0;
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--extractum-muted);
  }

  .project-row__dot[data-status="ready"] {
    background: var(--extractum-success);
  }

  .project-row__dot[data-status="running"] {
    background: var(--extractum-primary);
  }

  .project-row__dot[data-status="needs_attention"] {
    background: var(--extractum-danger);
  }

  .project-row__dot[data-status="empty"] {
    background: var(--extractum-muted-2);
  }

  .project-row__meta {
    font: 400 11px/1.2 var(--extractum-font);
    color: var(--extractum-muted-2);
  }
</style>
