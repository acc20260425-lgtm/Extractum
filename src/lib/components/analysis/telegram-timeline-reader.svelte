<script lang="ts">
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import TelegramMediaCard from "$lib/components/analysis/telegram-media-card.svelte";
  import { groupReaderItemsByDay, type SourceReaderItem } from "$lib/source-reader-model";

  let {
    items,
    loading,
    hasMore,
    contentLabel = "messages",
    formatTimestamp,
    onLoadMore,
  }: {
    items: SourceReaderItem[];
    loading: boolean;
    hasMore: boolean;
    contentLabel?: string;
    formatTimestamp: (value: number | null) => string;
    onLoadMore: () => void | Promise<void>;
  } = $props();

  const dayGroups = $derived(groupReaderItemsByDay(items));
</script>

<section class="telegram-timeline-reader" aria-label="Telegram source timeline">
  {#if !loading && items.length === 0}
    <EmptyState description={`No synced ${contentLabel} are available for this source view.`} />
  {:else}
    <div class="timeline-days">
      {#each dayGroups as day (day.key)}
        <section class="timeline-day" aria-label={day.label}>
          <div class="day-label">{day.label}</div>
          <ul>
            {#each day.items as item (item.id)}
              <li class:selected={item.selected}>
                <div class="message-meta">
                  <span>{formatTimestamp(item.publishedAt)}</span>
                  {#if item.author}<span>{item.author}</span>{/if}
                  {#if item.topicLabel}<Badge variant="neutral">{item.topicLabel}</Badge>{/if}
                  {#if item.replyLabel}<Badge variant="info">{item.replyLabel}</Badge>{/if}
                  {#if item.reactionLabel}<Badge variant="neutral">{item.reactionLabel}</Badge>{/if}
                  {#if item.ref}<Badge variant="neutral">{item.ref}</Badge>{/if}
                </div>
                <p>{item.content}</p>
                {#if item.mediaCards.length > 0}
                  <div class="media-list">
                    {#each item.mediaCards as media, index (`${item.id}:${index}`)}
                      <TelegramMediaCard {media} />
                    {/each}
                  </div>
                {/if}
              </li>
            {/each}
          </ul>
        </section>
      {/each}
    </div>
  {/if}

  {#if hasMore}
    <div class="reader-footer">
      <Button type="button" variant="secondary" disabled={loading} onclick={onLoadMore}>
        {loading ? "Loading..." : "Load older messages"}
      </Button>
    </div>
  {/if}
</section>

<style>
  .telegram-timeline-reader {
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
    min-width: 0;
  }

  .timeline-days,
  .timeline-day,
  .timeline-day ul {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .timeline-day ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .day-label {
    position: sticky;
    top: 0;
    z-index: 0;
    width: fit-content;
    padding: 0.25rem 0.55rem;
    border-radius: 999px;
    background: var(--panel);
    border: 1px solid var(--border);
    color: var(--muted);
    font-size: 0.75rem;
  }

  li {
    padding: 0.9rem 1rem;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--panel);
  }

  li.selected {
    border-color: var(--primary);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--primary) 12%, transparent);
  }

  .message-meta,
  .media-list {
    display: flex;
    gap: 0.45rem;
    flex-wrap: wrap;
    align-items: center;
  }

  .message-meta {
    margin-bottom: 0.45rem;
    color: var(--muted);
    font-size: 0.78rem;
  }

  p {
    margin: 0;
    white-space: pre-wrap;
    line-height: 1.5;
  }

  .media-list {
    margin-top: 0.65rem;
  }

  .reader-footer {
    display: flex;
    justify-content: center;
  }
</style>
