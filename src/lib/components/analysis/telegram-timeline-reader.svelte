<script lang="ts">
  import { tick } from "svelte";
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import TelegramMediaCard from "$lib/components/analysis/telegram-media-card.svelte";
  import { groupReaderItemsByDay, type SourceReaderItem } from "$lib/source-reader-model";

  let {
    items,
    loading,
    hasMore,
    ariaLabel = "Telegram source timeline",
    contentLabel = "messages",
    formatTimestamp,
    onLoadMore,
  }: {
    items: SourceReaderItem[];
    loading: boolean;
    hasMore: boolean;
    ariaLabel?: string;
    contentLabel?: string;
    formatTimestamp: (value: number | null) => string;
    onLoadMore: () => void | Promise<void>;
  } = $props();

  const dayGroups = $derived(groupReaderItemsByDay(items));
  let timelineElement: HTMLElement | null = $state(null);

  $effect(() => {
    const selectedRef = items.find((item) => item.selected)?.ref ?? null;
    if (selectedRef) {
      void scrollSelectedMessageIntoView(selectedRef);
    }
  });

  async function scrollSelectedMessageIntoView(selectedRef: string) {
    await tick();
    const selected = timelineElement?.querySelector<HTMLElement>(
      `[data-trace-ref="${CSS.escape(selectedRef)}"]`,
    );
    selected?.scrollIntoView({ block: "center", behavior: "smooth" });
  }
</script>

<section class="telegram-timeline-reader" aria-label={ariaLabel} bind:this={timelineElement}>
  {#if !loading && items.length === 0}
    <EmptyState description={`No synced ${contentLabel} are available for this source view.`} />
  {:else}
    <div class="timeline-days">
      {#each dayGroups as day (day.key)}
        <section class="timeline-day" aria-label={day.label}>
          <div class="day-label">{day.label}</div>
          <ul>
            {#each day.items as item (item.id)}
              <li class:selected={item.selected} data-trace-ref={item.ref}>
                <div class="telegram-message-bubble">
                  <div class="message-meta">
                    {#if item.author}<span class="message-author">{item.author}</span>{/if}
                    <span class="telegram-message-time">{formatTimestamp(item.publishedAt)}</span>
                    {#if item.historyScopeLabel}<span class="history-scope-badge">{item.historyScopeLabel}</span>{/if}
                    {#if item.topicLabel}<Badge variant="neutral">{item.topicLabel}</Badge>{/if}
                    {#if item.replyLabel}<Badge variant="info">{item.replyLabel}</Badge>{/if}
                    {#if item.reactionLabel}<Badge variant="neutral">{item.reactionLabel}</Badge>{/if}
                    {#if item.ref}<Badge variant="neutral">{item.ref}</Badge>{/if}
                  </div>
                  <p class="telegram-message-text" lang="ru">{item.content}</p>
                  {#if item.mediaCards.length > 0}
                    <div class="media-list">
                      {#each item.mediaCards as media, index (`${item.id}:${index}`)}
                        <TelegramMediaCard {media} />
                      {/each}
                    </div>
                  {/if}
                </div>
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
    gap: 0.75rem;
    min-width: 0;
  }

  .timeline-days,
  .timeline-day,
  .timeline-day ul {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
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
    align-self: center;
    padding: 0.18rem 0.5rem;
    border-radius: 999px;
    background: color-mix(in srgb, var(--panel-strong) 82%, transparent);
    color: color-mix(in srgb, var(--muted) 86%, var(--text));
    font-size: 0.75rem;
    box-shadow: 0 1px 2px color-mix(in srgb, var(--border) 34%, transparent);
  }

  li {
    display: flex;
    justify-content: center;
    padding: 0.125rem 0;
  }

  li.selected {
    border-radius: 18px;
    background: color-mix(in srgb, var(--primary) 8%, transparent);
  }

  .telegram-message-bubble {
    width: fit-content;
    max-width: 460px;
    padding: 0.5rem 0.625rem 0.4375rem;
    border-radius: 12px;
    background: color-mix(in srgb, var(--panel) 68%, #dff0e9);
    box-shadow: 0 0 0 1px color-mix(in srgb, var(--border) 38%, transparent);
    color: var(--text);
  }

  .message-meta,
  .media-list {
    display: flex;
    gap: 0.375rem;
    flex-wrap: wrap;
    align-items: center;
  }

  .message-meta {
    margin-bottom: 0.1875rem;
    color: color-mix(in srgb, var(--muted) 86%, var(--text));
    font-size: 0.8125rem;
    line-height: 1.25;
  }

  .message-author {
    color: color-mix(in srgb, var(--primary) 68%, var(--text));
    font-weight: 600;
  }

  .telegram-message-time {
    color: var(--muted);
  }

  .history-scope-badge {
    display: inline-flex;
    align-items: center;
    min-height: 1.35rem;
    padding: 0.1rem 0.4rem;
    border-radius: 999px;
    background: color-mix(in srgb, var(--primary) 13%, var(--panel));
    color: color-mix(in srgb, var(--primary) 74%, var(--text));
    font-size: 0.75rem;
    line-height: 1.2;
  }

  .telegram-message-text {
    margin: 0;
    white-space: pre-wrap;
    overflow-wrap: break-word;
    word-break: normal;
    hyphens: auto;
    font-size: 0.9375rem;
    line-height: 1.4;
  }

  .media-list {
    margin-top: 0.4375rem;
  }

  .reader-footer {
    display: flex;
    justify-content: center;
  }
</style>
