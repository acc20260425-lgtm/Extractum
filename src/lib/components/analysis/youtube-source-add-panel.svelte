<script lang="ts">
  import { Eye, Plus } from "@lucide/svelte";
  import Badge from "$lib/components/ui/Badge.svelte";
  import Button from "$lib/components/ui/Button.svelte";
  import Input from "$lib/components/ui/Input.svelte";
  import StatusMessage from "$lib/components/ui/StatusMessage.svelte";
  import YoutubeThumbnail from "$lib/components/youtube-thumbnail.svelte";
  import { addYoutubeSource, previewYoutubeSource } from "$lib/api/sources";
  import { formatAppError } from "$lib/app-error";
  import type { YoutubePreview } from "$lib/types/sources";

  let {
    onSourcesChanged,
    onStatus,
  }: {
    onSourcesChanged: (sourceId?: number) => void | Promise<void>;
    onStatus: (message: string) => void;
  } = $props();

  let youtubeUrl = $state("");
  let youtubePreview = $state<YoutubePreview | null>(null);
  let previewingYoutube = $state(false);
  let addingYoutube = $state(false);
  let youtubeStatus = $state("");
  let previewedUrl = $state("");

  const trimmedYoutubeUrl = $derived(youtubeUrl.trim());
  const canPreview = $derived(Boolean(trimmedYoutubeUrl) && !previewingYoutube && !addingYoutube);
  const canAdd = $derived(Boolean(youtubePreview) && !previewingYoutube && !addingYoutube);

  function updateYoutubeUrl(value: string) {
    youtubeUrl = value;
    youtubeStatus = "";
    if (value.trim() !== previewedUrl) {
      youtubePreview = null;
    }
  }

  async function previewYoutube() {
    if (!trimmedYoutubeUrl || previewingYoutube || addingYoutube) {
      return;
    }

    previewingYoutube = true;
    youtubeStatus = "";
    try {
      youtubePreview = await previewYoutubeSource(trimmedYoutubeUrl);
      previewedUrl = trimmedYoutubeUrl;
    } catch (error) {
      youtubeStatus = formatAppError("previewing the YouTube source", error);
    } finally {
      previewingYoutube = false;
    }
  }

  async function addYoutube() {
    if (!youtubePreview || addingYoutube) {
      return;
    }

    addingYoutube = true;
    youtubeStatus = "";
    try {
      const source = await addYoutubeSource(previewedUrl || trimmedYoutubeUrl);
      onStatus(`Source "${source.title ?? source.externalId}" added.`);
      await onSourcesChanged(source.id);
      youtubeUrl = "";
      youtubePreview = null;
      previewedUrl = "";
    } catch (error) {
      youtubeStatus = formatAppError("adding the YouTube source", error);
    } finally {
      addingYoutube = false;
    }
  }

  function previewKindLabel(preview: YoutubePreview) {
    return preview.kind === "playlist" ? "playlist" : "video";
  }

  function availabilityLabel(preview: YoutubePreview) {
    return preview.availabilityStatus
      .split("_")
      .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
      .join(" ");
  }

  function formatDuration(seconds: number | null) {
    if (seconds === null) {
      return null;
    }

    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const remainingSeconds = seconds % 60;

    if (hours > 0) {
      return `${hours}:${String(minutes).padStart(2, "0")}:${String(remainingSeconds).padStart(2, "0")}`;
    }
    return `${minutes}:${String(remainingSeconds).padStart(2, "0")}`;
  }

  function captionsLabel(preview: YoutubePreview) {
    if (!preview.captionsEstimate) {
      return null;
    }

    const kinds = [
      preview.captionsEstimate.hasManual ? "manual" : null,
      preview.captionsEstimate.hasAuto ? "auto" : null,
    ].filter(Boolean);
    const languages = preview.captionsEstimate.languages.slice(0, 4).join(", ");

    return [kinds.join(" + "), languages].filter(Boolean).join(" captions: ");
  }

  function handleUrlKeydown(event: KeyboardEvent) {
    if (event.key === "Enter") {
      event.preventDefault();
      void previewYoutube();
    }
  }
</script>

<section class="youtube-panel">
  <div class="youtube-entry">
    <label>YouTube URL
      <Input
        value={youtubeUrl}
        placeholder="https://www.youtube.com/watch?v=..."
        disabled={previewingYoutube || addingYoutube}
        oninput={(event) => updateYoutubeUrl((event.currentTarget as HTMLInputElement).value)}
        onkeydown={handleUrlKeydown}
      />
    </label>
    <Button onclick={previewYoutube} disabled={!canPreview}>
      <Eye size={15} aria-hidden="true" />
      {previewingYoutube ? "Previewing..." : "Preview"}
    </Button>
  </div>

  {#if youtubeStatus}
    <StatusMessage tone={youtubeStatus.startsWith("Error") ? "error" : "default"}>
      {youtubeStatus}
    </StatusMessage>
  {/if}

  {#if youtubePreview}
    <article class="preview-card">
      <div class="preview-media" aria-hidden="true">
        {#if youtubePreview.thumbnailUrl}
          <YoutubeThumbnail url={youtubePreview.thumbnailUrl} />
        {:else}
          <span>{youtubePreview.kind === "playlist" ? "PL" : "YT"}</span>
        {/if}
      </div>

      <div class="preview-copy">
        <div class="preview-badges">
          <Badge variant="info">{previewKindLabel(youtubePreview)}</Badge>
          <Badge>{availabilityLabel(youtubePreview)}</Badge>
          {#if youtubePreview.playlistVideoCount !== null}
            <Badge>{youtubePreview.playlistVideoCount} videos</Badge>
          {/if}
          {#if formatDuration(youtubePreview.durationSeconds)}
            <Badge>{formatDuration(youtubePreview.durationSeconds)}</Badge>
          {/if}
        </div>

        <strong>{youtubePreview.title ?? youtubePreview.externalId}</strong>

        <div class="preview-meta">
          {#if youtubePreview.channelTitle}
            <span>{youtubePreview.channelTitle}</span>
          {/if}
          {#if youtubePreview.channelHandle}
            <span>{youtubePreview.channelHandle}</span>
          {/if}
          {#if youtubePreview.publishedAt}
            <span>{youtubePreview.publishedAt}</span>
          {/if}
          {#if captionsLabel(youtubePreview)}
            <span>{captionsLabel(youtubePreview)}</span>
          {/if}
        </div>

        {#if youtubePreview.warnings.length > 0}
          <div class="warning-list">
            {#each youtubePreview.warnings as warning (warning)}
              <StatusMessage tone="muted" size="sm">{warning}</StatusMessage>
            {/each}
          </div>
        {/if}

        <div class="preview-actions">
          <span class="canonical-url">{youtubePreview.canonicalUrl}</span>
          <Button onclick={addYoutube} disabled={!canAdd}>
            <Plus size={15} aria-hidden="true" />
            {addingYoutube ? "Adding..." : "Add source"}
          </Button>
        </div>
      </div>
    </article>
  {/if}
</section>

<style>
  .youtube-panel {
    display: flex;
    flex-direction: column;
    gap: 0.9rem;
  }

  label {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    color: var(--muted);
    font-size: 0.83rem;
  }

  .youtube-entry {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    gap: 0.65rem;
    align-items: end;
  }

  .preview-card {
    display: grid;
    grid-template-columns: minmax(8.5rem, 12rem) minmax(0, 1fr);
    gap: 0.85rem;
    padding: 0.75rem;
    border: 1px solid color-mix(in srgb, var(--border) 82%, transparent);
    border-radius: 8px;
    background: var(--panel-strong);
    min-width: 0;
  }

  .preview-media {
    aspect-ratio: 16 / 9;
    border-radius: 6px;
    overflow: hidden;
    display: flex;
    align-items: center;
    justify-content: center;
    background: color-mix(in srgb, var(--primary) 12%, var(--panel));
    color: var(--primary);
    font-weight: 700;
    min-width: 0;
  }

  .preview-media :global(img) {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .preview-copy {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .preview-copy strong {
    font-size: 1rem;
    overflow-wrap: anywhere;
  }

  .preview-badges,
  .preview-meta {
    display: flex;
    flex-wrap: wrap;
    gap: 0.4rem;
    min-width: 0;
  }

  .preview-meta {
    color: var(--muted);
    font-size: 0.8rem;
  }

  .warning-list {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
  }

  .preview-actions {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    gap: 0.65rem;
    align-items: center;
    margin-top: auto;
  }

  .canonical-url {
    min-width: 0;
    color: var(--muted);
    font-size: 0.78rem;
    overflow-wrap: anywhere;
  }

  @media (max-width: 720px) {
    .youtube-entry,
    .preview-card,
    .preview-actions {
      grid-template-columns: 1fr;
    }
  }
</style>
