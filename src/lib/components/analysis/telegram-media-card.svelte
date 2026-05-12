<script lang="ts">
  import { FileText, Image, Paperclip, Video } from "@lucide/svelte";
  import type { SourceReaderMediaCard } from "$lib/source-reader-model";

  let { media }: { media: SourceReaderMediaCard } = $props();

  const Icon = $derived(iconForMedia(media.kind));

  function iconForMedia(kind: string) {
    if (kind.includes("photo") || kind.includes("image")) return Image;
    if (kind.includes("video")) return Video;
    if (kind.includes("document")) return FileText;
    return Paperclip;
  }
</script>

<div class="media-card">
  <Icon size={16} aria-hidden="true" />
  <div>
    <strong>{media.title}</strong>
    {#if media.summary}<span>{media.summary}</span>{/if}
    {#if media.fileName}<span>{media.fileName}</span>{/if}
    {#if media.mimeType}<span>{media.mimeType}</span>{/if}
  </div>
</div>

<style>
  .media-card {
    display: flex;
    gap: 0.4rem;
    align-items: center;
    padding: 0.375rem 0.5rem;
    border: 1px solid color-mix(in srgb, var(--border) 72%, transparent);
    border-radius: 7px;
    background: color-mix(in srgb, var(--panel-strong) 58%, transparent);
    color: var(--text);
    font-size: 0.8125rem;
  }

  .media-card div {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
  }

  .media-card span {
    color: var(--muted);
    font-size: 0.75rem;
    overflow-wrap: anywhere;
  }
</style>
