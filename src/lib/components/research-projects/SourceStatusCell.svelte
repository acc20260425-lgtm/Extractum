<script lang="ts">
  let { row }: { row: Record<string, unknown> } = $props();

  let status = $derived(String(row.syncStatus ?? ""));
  let label = $derived(String(row.statusLabel ?? ""));
</script>

<span class="source-status-cell">
  <span
    class="source-status-cell__dot"
    data-testid="source-status-dot"
    data-status={status}
  ></span>
  <span class="source-status-cell__label">{label}</span>
</span>

<style>
  .source-status-cell {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
  }

  .source-status-cell__dot {
    flex-shrink: 0;
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--extractum-muted);
  }

  .source-status-cell__dot[data-status="active"] {
    background: var(--extractum-success);
  }

  .source-status-cell__dot[data-status="syncing"] {
    background: var(--extractum-primary);
  }

  .source-status-cell__dot[data-status="error"] {
    background: var(--extractum-danger);
  }

  .source-status-cell__dot[data-status="unavailable"] {
    background: var(--extractum-warning);
  }

  .source-status-cell__label {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font: 400 12px/1.2 var(--extractum-font);
    color: var(--extractum-text);
  }
</style>
