<script lang="ts">
  import SourceMessagesPanel from "$lib/components/source-messages-panel.svelte";
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Select from "$lib/components/ui/Select.svelte";
  import type { AnalysisSourceOption } from "$lib/types/analysis";
  import type { SourceItem, SourceForumTopic } from "$lib/types/sources";

  let {
    currentRunOpen,
    currentSourceMetric,
    sourceItems,
    loadingItems,
    sourceTopics,
    loadingSourceTopics,
    selectedTopicKey,
    showTopicSelector,
    formatTimestamp,
    onChangeSelectedTopicKey,
  }: {
    currentRunOpen: boolean;
    currentSourceMetric: AnalysisSourceOption | null;
    sourceItems: SourceItem[];
    loadingItems: boolean;
    sourceTopics: SourceForumTopic[];
    loadingSourceTopics: boolean;
    selectedTopicKey: string;
    showTopicSelector: boolean;
    formatTimestamp: (value: number | null) => string;
    onChangeSelectedTopicKey: (value: string) => void | Promise<void>;
  } = $props();

  let contextExpandedOverride = $state<boolean | null>(null);
  let fullContextPreview = $state(false);

  const compactContextPreviewLimit = 8;
  const expandedContextPreviewLimit = 24;
  const contextExpanded = $derived(contextExpandedOverride ?? !currentRunOpen);
  const sortedSourceTopics = $derived([...sourceTopics].sort(compareTopics));

  function compareTopics(left: SourceForumTopic, right: SourceForumTopic) {
    if (left.kind !== right.kind) {
      return left.kind === "topic" ? -1 : 1;
    }

    if (left.isDeleted !== right.isDeleted) {
      return left.isDeleted ? 1 : -1;
    }

    const titleOrder = left.title.localeCompare(right.title, undefined, {
      sensitivity: "base",
      numeric: true,
    });
    if (titleOrder !== 0) {
      return titleOrder;
    }

    return left.key.localeCompare(right.key, undefined, {
      sensitivity: "base",
      numeric: true,
    });
  }

  function toggleContextPanel() {
    contextExpandedOverride = !contextExpanded;
    if (!contextExpanded) {
      fullContextPreview = false;
    }
  }

  function toggleContextDepth() {
    fullContextPreview = !fullContextPreview;
  }
</script>

<div class="context-panel" class:compact={!contextExpanded}>
  <div class="context-panel-header">
    <div>
      <span class="eyebrow">Source context</span>
      <h3>Recent synced messages</h3>
      <p class="context-summary">
        {#if currentRunOpen}
          Report is in focus. Open the message preview only when you need to verify source-level context.
        {:else}
          Scan the latest synced messages before you launch a run.
        {/if}
      </p>
    </div>
    <div class="context-panel-actions">
      {#if showTopicSelector}
        <label class="topic-filter">
          <span>Topic view</span>
          <Select
            value={selectedTopicKey}
            disabled={loadingSourceTopics}
            onchange={(event) =>
              onChangeSelectedTopicKey(
                (event.currentTarget as HTMLSelectElement).value,
              )}
          >
            <option value="__all_topics__">All topics</option>
            {#if loadingSourceTopics && sourceTopics.length === 0}
              <option value="__loading_topics__" disabled>Loading topics...</option>
            {:else}
              {#each sortedSourceTopics as topic (topic.key)}
                <option value={topic.key}>
                  {topic.title} ({topic.messageCount})
                </option>
              {/each}
            {/if}
          </Select>
        </label>
      {/if}
      <Badge variant="neutral">
        {currentSourceMetric?.item_count ?? sourceItems.length} messages
      </Badge>
      <Button variant="ghost" size="sm" onclick={toggleContextPanel}>
        {contextExpanded ? "Hide preview" : "Peek at messages"}
      </Button>
    </div>
  </div>

  {#if contextExpanded}
    <SourceMessagesPanel
      {loadingItems}
      items={sourceItems}
      formatDate={formatTimestamp}
      embedded={true}
      previewLimit={fullContextPreview ? expandedContextPreviewLimit : compactContextPreviewLimit}
    />

    {#if sourceItems.length > compactContextPreviewLimit}
      <div class="context-panel-footer">
        <span class="context-footnote">
          {fullContextPreview
            ? `Showing a wider slice of the latest ${Math.min(sourceItems.length, expandedContextPreviewLimit)} messages.`
            : `Showing the latest ${Math.min(sourceItems.length, compactContextPreviewLimit)} messages to keep the report area light.`}
        </span>
        <Button variant="secondary" size="sm" onclick={toggleContextDepth}>
          {fullContextPreview ? "Show fewer" : "Show more"}
        </Button>
      </div>
    {/if}
  {/if}
</div>

<style>
  .context-panel {
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
    padding: 1rem;
    background:
      linear-gradient(180deg, color-mix(in srgb, var(--panel-strong) 44%, transparent), var(--panel));
    border: 1px solid var(--border);
    box-shadow: var(--shadow);
    border-radius: 16px;
  }

  .context-panel.compact {
    background:
      linear-gradient(180deg, color-mix(in srgb, var(--panel-strong) 24%, transparent), var(--panel));
  }

  .context-panel-header {
    display: flex;
    justify-content: space-between;
    gap: 0.75rem;
    align-items: flex-start;
  }

  .eyebrow {
    display: inline-block;
    font-size: 0.68rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--muted);
    margin-bottom: 0.2rem;
  }

  .context-panel-header h3 {
    margin: 0;
  }

  .context-summary {
    margin: 0.3rem 0 0 0;
    color: var(--muted);
    line-height: 1.45;
    max-width: 56ch;
  }

  .context-panel-actions {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
    justify-content: flex-end;
  }

  .topic-filter {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    min-width: min(17rem, 100%);
    color: var(--muted);
    font-size: 0.74rem;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .topic-filter :global(select) {
    min-width: 14rem;
    text-transform: none;
    letter-spacing: normal;
    font-size: 0.9rem;
    color: var(--text);
  }

  .context-panel-footer {
    display: flex;
    justify-content: space-between;
    gap: 0.75rem;
    align-items: center;
    flex-wrap: wrap;
    padding-top: 0.1rem;
    border-top: 1px solid color-mix(in srgb, var(--border) 70%, transparent);
  }

  .context-footnote {
    color: var(--muted);
    font-size: 0.8rem;
    line-height: 1.45;
  }

  @media (max-width: 720px) {
    .context-panel-header {
      flex-direction: column;
      align-items: stretch;
    }

    .context-panel-actions {
      justify-content: flex-start;
    }
  }
</style>
