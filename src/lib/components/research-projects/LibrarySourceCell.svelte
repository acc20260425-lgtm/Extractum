<script lang="ts">
  let {
    row,
  }: {
    row: Record<string, unknown>;
  } = $props();

  let title = $derived(String(row.title ?? ""));
  let subtitle = $derived(
    typeof row.subtitle === "string" && row.subtitle.length > 0
      ? row.subtitle
      : typeof row.filterSummary === "string"
        ? row.filterSummary
        : "Фильтры проекта применяются при запуске анализа.",
  );
</script>

<div class="library-source-cell">
  <strong>{title}</strong>
  <span>{subtitle}</span>
</div>

<style>
  .library-source-cell {
    display: flex;
    min-width: 0;
    flex-direction: column;
    gap: 2px;
  }

  .library-source-cell strong,
  .library-source-cell span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .library-source-cell strong {
    font-size: 13px;
  }

  .library-source-cell span {
    color: var(--extractum-muted);
    font-size: 11px;
  }
</style>
