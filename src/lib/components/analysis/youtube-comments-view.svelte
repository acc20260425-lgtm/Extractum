<script lang="ts">
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import Select from "$lib/components/ui/Select.svelte";
  import StatusMessage from "$lib/components/ui/StatusMessage.svelte";
  import {
    commentsCoverageState,
    filterLoadedYoutubeComments,
    groupLoadedYoutubeComments,
    sortLoadedYoutubeComments,
    type LoadedYoutubeCommentSort,
  } from "$lib/source-browser-model";
  import type { SourceItem, SourceJobRecord } from "$lib/types/sources";
  import type { YoutubeVideoDetail } from "$lib/types/youtube";

  type ViewMode = "threaded" | "flat";

  let {
    items,
    detail,
    sourceJobs,
    routeError,
    loading,
    hasMore,
    formatTimestamp,
    onLoadMore,
    onSyncComments,
    onSyncMetadata,
  }: {
    items: SourceItem[];
    detail: YoutubeVideoDetail | null;
    sourceJobs: SourceJobRecord[];
    routeError: string | null;
    loading: boolean;
    hasMore: boolean;
    formatTimestamp: (value: number | null) => string;
    onLoadMore: () => void | Promise<void>;
    onSyncComments: () => void | Promise<void>;
    onSyncMetadata: () => void | Promise<void>;
  } = $props();

  let search = $state("");
  let viewMode = $state<ViewMode>("threaded");
  let sortMode = $state<LoadedYoutubeCommentSort>("newest");

  const coverage = $derived(commentsCoverageState({
    items,
    detail,
    jobs: sourceJobs,
    routeError,
    loadingItems: loading,
  }));
  const filteredComments = $derived(filterLoadedYoutubeComments(items, search));
  const visibleComments = $derived(sortLoadedYoutubeComments(filteredComments, sortMode));
  const commentThreads = $derived(groupLoadedYoutubeComments(visibleComments));

  function inputValue(event: Event) {
    const target = event.currentTarget;
    return target instanceof HTMLInputElement ? target.value : "";
  }

  function changeSort(event: Event) {
    sortMode = (event.currentTarget as HTMLSelectElement).value as LoadedYoutubeCommentSort;
  }

  function commentLabel(item: SourceItem) {
    return item.youtubeComment?.isReply ? "Reply" : "Comment";
  }
</script>

<section class="youtube-comments-view" aria-label="YouTube comments">
  <div class="comments-header">
    <div class="comments-status">
      <span class="eyebrow">YouTube comments</span>
      <Badge variant={coverage === "failed" ? "danger" : coverage === "syncing" ? "info" : coverage === "synced_with_rows" ? "success" : "neutral"}>
        {coverage.replaceAll("_", " ")}
      </Badge>
    </div>
    <div class="comments-actions">
      <Button type="button" size="sm" variant="secondary" onclick={onSyncMetadata}>Sync metadata</Button>
      <Button type="button" size="sm" variant="secondary" onclick={onSyncComments}>Sync comments</Button>
    </div>
  </div>

  {#if routeError}
    <StatusMessage tone="error">{routeError}</StatusMessage>
  {/if}

  <div class="comments-toolbar">
    <label class="search-field">
      <span>Search loaded comments</span>
      <Input
        type="search"
        value={search}
        placeholder="Search loaded comments"
        ariaLabel="Search loaded comments"
        oninput={(event) => (search = inputValue(event))}
      />
    </label>

    <label class="sort-field">
      <span>Sort loaded comments</span>
      <Select value={sortMode} onchange={changeSort}>
        <option value="newest">Newest first</option>
        <option value="oldest">Oldest first</option>
        <option value="most_liked">Most liked</option>
      </Select>
    </label>

    <div class="view-toggle" aria-label="Comment layout">
      <Button
        type="button"
        size="sm"
        variant={viewMode === "threaded" ? "secondary" : "ghost"}
        selected={viewMode === "threaded"}
        onclick={() => (viewMode = "threaded")}
      >
        Threaded
      </Button>
      <Button
        type="button"
        size="sm"
        variant={viewMode === "flat" ? "secondary" : "ghost"}
        selected={viewMode === "flat"}
        onclick={() => (viewMode = "flat")}
      >
        Flat
      </Button>
    </div>
  </div>

  {#if !loading && visibleComments.length === 0}
    <EmptyState description={coverage === "synced_empty" ? "This video has no synced comments." : "No loaded comments match this view."} />
  {:else if viewMode === "flat"}
    <ul class="comment-list">
      {#each visibleComments as item (item.id)}
        <li>
          {@render commentCard(item, true)}
        </li>
      {/each}
    </ul>
  {:else}
    <ul class="thread-list">
      {#each commentThreads as thread (thread.item.id)}
        <li>
          {@render commentCard(thread.item, thread.parentLoaded)}
          {#if thread.replies.length > 0}
            <ul class="reply-list">
              {#each thread.replies as reply (reply.item.id)}
                <li>{@render commentCard(reply.item, reply.parentLoaded)}</li>
              {/each}
            </ul>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}

  {#if hasMore}
    <div class="reader-footer">
      <Button type="button" variant="secondary" disabled={loading} onclick={onLoadMore}>
        {loading ? "Loading..." : "Load more comments"}
      </Button>
    </div>
  {/if}
</section>

{#snippet commentCard(item: SourceItem, parentLoaded: boolean)}
  <article class:reply={item.youtubeComment?.isReply}>
    <div class="comment-heading">
      <strong>{item.author ?? "Unknown author"}</strong>
      <span>{formatTimestamp(item.publishedAt)}</span>
    </div>
    <div class="comment-meta">
      <Badge variant="neutral">{commentLabel(item)}</Badge>
      {#if item.youtubeComment?.likeCount !== null && item.youtubeComment?.likeCount !== undefined}
        <Badge variant="neutral">{item.youtubeComment.likeCount} likes</Badge>
      {/if}
      {#if item.youtubeComment?.isPinned}<Badge variant="info">Pinned</Badge>{/if}
      {#if item.youtubeComment?.isHearted}<Badge variant="info">Hearted</Badge>{/if}
      {#if !parentLoaded}<Badge variant="warning">parent not loaded</Badge>{/if}
    </div>
    <p>{item.content ?? "No comment text loaded."}</p>
  </article>
{/snippet}

<style>
  .youtube-comments-view,
  .comment-list,
  .thread-list,
  .reply-list {
    display: flex;
    flex-direction: column;
  }

  .youtube-comments-view {
    gap: 0.85rem;
    min-width: 0;
  }

  .comments-header,
  .comments-toolbar,
  .comments-status,
  .comments-actions,
  .view-toggle,
  .comment-heading,
  .comment-meta {
    display: flex;
    gap: 0.55rem;
    align-items: center;
    flex-wrap: wrap;
  }

  .comments-header {
    justify-content: space-between;
    align-items: flex-start;
  }

  .comments-toolbar {
    align-items: flex-end;
  }

  .eyebrow {
    color: var(--muted);
    font-size: 0.68rem;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  .search-field,
  .sort-field {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    color: var(--muted);
    font-size: 0.78rem;
  }

  .search-field {
    flex: 1 1 16rem;
  }

  .sort-field {
    flex: 0 1 12rem;
  }

  .comment-list,
  .thread-list,
  .reply-list {
    gap: 0.55rem;
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .reply-list {
    margin-top: 0.55rem;
    padding-left: 1.25rem;
    border-left: 2px solid color-mix(in srgb, var(--border) 80%, transparent);
  }

  article {
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
    padding: 0.7rem 0.8rem;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--panel);
  }

  article.reply {
    background: color-mix(in srgb, var(--panel) 82%, var(--panel-strong));
  }

  .comment-heading {
    justify-content: space-between;
  }

  .comment-heading span,
  p {
    color: var(--muted);
  }

  p {
    margin: 0;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
    line-height: 1.45;
  }

  .reader-footer {
    display: flex;
    justify-content: center;
  }
</style>
