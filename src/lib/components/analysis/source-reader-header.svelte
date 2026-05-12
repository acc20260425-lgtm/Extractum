<script lang="ts">
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Select from "$lib/components/ui/Select.svelte";
  import type { SourceViewBasis } from "$lib/analysis-workspace-state";

  const allSourcesValue = "__all_sources__";

  let {
    title,
    subtitle,
    surfaceLabel = "Source material",
    sourceViewBasis,
    canViewLiveSource,
    canBackToRunSnapshot,
    selectedSourceId,
    sourceOptions,
    onViewLiveSource,
    onBackToRunSnapshot,
    onChangeSelectedSourceId,
  }: {
    title: string;
    subtitle: string;
    surfaceLabel?: string;
    sourceViewBasis: SourceViewBasis;
    canViewLiveSource: boolean;
    canBackToRunSnapshot: boolean;
    selectedSourceId: number | null;
    sourceOptions: Array<{ id: number; label: string; count: number }>;
    onViewLiveSource: () => void;
    onBackToRunSnapshot: () => void;
    onChangeSelectedSourceId: (sourceId: number | null) => void;
  } = $props();

  function changeSelectedSource(event: Event) {
    const value = (event.currentTarget as HTMLSelectElement).value;
    onChangeSelectedSourceId(value === allSourcesValue ? null : Number(value));
  }
</script>

<header class="source-reader-header" aria-label={title}>
  <div class="reader-title">
    <span class="eyebrow">{surfaceLabel}</span>
    <p>{subtitle}</p>
  </div>

  <div class="reader-actions">
    <Badge variant={sourceViewBasis === "live_source" ? "warning" : "success"}>
      {sourceViewBasis === "live_source" ? "Live source" : "Run snapshot"}
    </Badge>

    {#if sourceOptions.length > 1}
      <label>
        <span>Source focus</span>
        <Select
          value={selectedSourceId === null ? allSourcesValue : String(selectedSourceId)}
          onchange={changeSelectedSource}
        >
          <option value={allSourcesValue}>All sources</option>
          {#each sourceOptions as option (option.id)}
            <option value={String(option.id)}>{option.label} ({option.count})</option>
          {/each}
        </Select>
      </label>
    {/if}

    {#if canViewLiveSource}
      <Button type="button" variant="secondary" onclick={onViewLiveSource}>View live source</Button>
    {/if}

    {#if canBackToRunSnapshot}
      <Button type="button" variant="secondary" onclick={onBackToRunSnapshot}>Back to run snapshot</Button>
    {/if}
  </div>
</header>

<style>
  .source-reader-header {
    display: flex;
    justify-content: space-between;
    gap: 1rem;
    align-items: flex-start;
    padding: 1rem;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--panel);
  }

  .reader-title {
    min-width: 0;
  }

  .eyebrow {
    display: inline-block;
    font-size: 0.68rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--muted);
  }

  p {
    margin: 0.2rem 0 0;
    color: var(--muted);
    line-height: 1.45;
  }

  .reader-actions {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    justify-content: flex-end;
    flex-wrap: wrap;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    color: var(--muted);
    font-size: 0.75rem;
    min-width: 12rem;
  }

  @media (max-width: 760px) {
    .source-reader-header {
      flex-direction: column;
    }

    .reader-actions {
      justify-content: flex-start;
      width: 100%;
    }

    label {
      width: 100%;
    }
  }
</style>
