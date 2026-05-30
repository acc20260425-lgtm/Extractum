<script lang="ts">
  import { Copy } from "@lucide/svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import StatusMessage from "$lib/components/ui/StatusMessage.svelte";
  import { formatRawJsonPreview } from "$lib/source-browser-model";

  let {
    value,
    maxChars = 4000,
  }: {
    value: unknown | null;
    maxChars?: number;
  } = $props();

  let expanded = $state(false);
  let copied = $state(false);
  const preview = $derived(expanded ? formatRawJsonPreview(value, maxChars) : null);

  async function copyRawJson() {
    if (!preview?.full || typeof navigator === "undefined" || !navigator.clipboard) return;
    try {
      await navigator.clipboard.writeText(preview.full);
      copied = true;
    } catch (error) {
      console.error("Failed to copy raw JSON", error);
    }
  }
</script>

<section class="raw-json-panel" aria-label="Raw JSON">
  <div class="raw-json-actions">
    <Button
      type="button"
      variant="secondary"
      ariaExpanded={expanded}
      onclick={() => (expanded = !expanded)}
    >
      {expanded ? "Hide raw JSON" : "Show raw JSON"}
    </Button>
    {#if expanded && preview}
      <Button type="button" variant="ghost" onclick={copyRawJson}>
        <Copy size={14} aria-hidden="true" />
        {copied ? "Copied" : "Copy"}
      </Button>
    {/if}
  </div>

  {#if expanded}
    {#if preview}
      {#if preview.truncated}
        <StatusMessage tone="info">
          Large payload preview is truncated.
        </StatusMessage>
      {/if}
      <pre>{preview.preview}</pre>
    {:else}
      <EmptyState description="No source-level raw JSON is available." />
    {/if}
  {/if}
</section>

<style>
  .raw-json-panel {
    display: flex;
    flex-direction: column;
    gap: 0.65rem;
    min-width: 0;
  }

  .raw-json-actions {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    flex-wrap: wrap;
  }

  pre {
    max-height: 20rem;
    overflow: auto;
    margin: 0;
    padding: 0.75rem;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--panel);
    color: var(--text);
    font-size: 0.8125rem;
    line-height: 1.45;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }
</style>
