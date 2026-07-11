<script lang="ts">
  import { Eye, Plus } from "@lucide/svelte";
  import {
    ExtractumBadge,
    ExtractumButton,
    ExtractumStatusMessage,
    ExtractumTextInput,
  } from "$lib/components/extractum-ui";
  import { addYoutubeSource, previewYoutubeSource } from "$lib/api/sources";
  import { formatAppError } from "$lib/app-error";
  import YoutubeThumbnail from "$lib/components/youtube-thumbnail.svelte";
  import {
    classifyYoutubeImportInput,
    existingYoutubeSmartImportSource,
  } from "$lib/ui/library-add-source-model";
  import type { LibraryCatalogSourceView } from "$lib/ui/library-catalog-model";
  import type { ProjectAddSourceContext } from "$lib/ui/project-add-source-context";
  import type { YoutubePreview } from "$lib/types/sources";

  let {
    sources,
    onSourcesChanged,
    onStatus,
    projectContext,
  }: {
    sources: LibraryCatalogSourceView[];
    onSourcesChanged: (sourceId?: number) => void | Promise<void>;
    onStatus: (message: string) => void;
    projectContext?: ProjectAddSourceContext;
  } = $props();

  let youtubeUrl = $state("");
  let preview = $state<YoutubePreview | null>(null);
  let previewedUrl = $state("");
  let previewing = $state(false);
  let adding = $state(false);
  let status = $state("");

  const trimmedUrl = $derived(youtubeUrl.trim());
  const classification = $derived(classifyYoutubeImportInput(trimmedUrl));
  const backendUrl = $derived(classification.normalizedUrl ?? trimmedUrl);
  const existingSmartImportSource = $derived(existingYoutubeSmartImportSource(sources, preview));
  const canPreview = $derived(Boolean(trimmedUrl) && classification.supported && !previewing && !adding);
  const existingSmartImportSourceConnected = $derived(
    Boolean(
      existingSmartImportSource &&
        projectContext?.connectedSourceIds.has(existingSmartImportSource.sourceId),
    ),
  );
  const canConnectExistingSmartImportSource = $derived(
    Boolean(
      projectContext &&
        existingSmartImportSource &&
        !existingSmartImportSourceConnected &&
        !previewing &&
        !adding,
    ),
  );
  const canAdd = $derived(Boolean(preview) && !existingSmartImportSource && !previewing && !adding);

  function updateUrl(value: string) {
    youtubeUrl = value;
    status = "";
    if (value.trim() !== previewedUrl) preview = null;
  }

  async function previewSource() {
    if (!canPreview) return;
    previewing = true;
    status = "";
    try {
      preview = await previewYoutubeSource(backendUrl);
      previewedUrl = backendUrl;
    } catch (error) {
      preview = null;
      status = formatAppError("previewing the YouTube source", error);
    } finally {
      previewing = false;
    }
  }

  async function addSource() {
    if (!preview || adding) return;
    if (existingSmartImportSource) {
      if (projectContext) {
        if (projectContext.connectedSourceIds.has(existingSmartImportSource.sourceId)) {
          onStatus("Already connected to this project.");
          return;
        }
        adding = true;
        status = "";
        try {
          await projectContext.onConnectExistingSource(existingSmartImportSource.sourceId);
        } finally {
          adding = false;
        }
        return;
      }
      status = `Already in Library: ${existingSmartImportSource.title}`;
      return;
    }
    adding = true;
    status = "";
    try {
      const source = await addYoutubeSource(previewedUrl || backendUrl, {
        materializePlaylistVideos: preview.kind !== "playlist",
      });
      onStatus(`Source "${source.title ?? source.externalId}" added.`);
      await onSourcesChanged(source.id);
      youtubeUrl = "";
      preview = null;
      previewedUrl = "";
    } catch (error) {
      status = formatAppError("adding the YouTube source", error);
    } finally {
      adding = false;
    }
  }

  function formatDuration(seconds: number | null) {
    if (seconds === null) return null;
    const minutes = Math.floor(seconds / 60);
    const remainingSeconds = seconds % 60;
    return `${minutes}:${String(remainingSeconds).padStart(2, "0")}`;
  }
</script>

<section class="library-youtube-smart-import" aria-label="YouTube smart import">
  <div class="entry-row">
    <label>
      <span>YouTube URL</span>
      <ExtractumTextInput
        value={youtubeUrl}
        placeholder="https://www.youtube.com/watch?v=..."
        disabled={previewing || adding}
        oninput={(event) => updateUrl((event.currentTarget as HTMLInputElement).value)}
        onkeydown={(event) => {
          if (event.key === "Enter") {
            event.preventDefault();
            void previewSource();
          }
        }}
      />
    </label>

    <ExtractumButton onclick={previewSource} disabled={!canPreview}>
      <Eye size={14} aria-hidden="true" />
      {previewing ? "Previewing..." : "Preview"}
    </ExtractumButton>
  </div>

  {#if classification.reason && trimmedUrl}
    <ExtractumStatusMessage tone={classification.kind === "channel" ? "info" : "muted"}>
      {classification.kind === "channel" ? "Not supported yet. " : ""}{classification.reason}
    </ExtractumStatusMessage>
  {/if}

  {#if status}
    <ExtractumStatusMessage tone={status.startsWith("Error") ? "error" : "default"}>
      {status}
    </ExtractumStatusMessage>
  {/if}

  {#if existingSmartImportSource}
    <ExtractumStatusMessage tone="info">
      {#if existingSmartImportSourceConnected}
        Already connected to this project.
      {:else}
        Already in Library: {existingSmartImportSource.title}
      {/if}
    </ExtractumStatusMessage>
  {/if}

  {#if preview}
    <article class="preview-card">
      <div class="preview-media" aria-hidden="true">
        {#if preview.thumbnailUrl}
          <YoutubeThumbnail url={preview.thumbnailUrl} />
        {:else}
          <span>{preview.kind === "playlist" ? "PL" : "YT"}</span>
        {/if}
      </div>
      <div class="preview-copy">
        <div class="badges">
          <ExtractumBadge>{preview.kind}</ExtractumBadge>
          <ExtractumBadge>{preview.availabilityStatus.replaceAll("_", " ")}</ExtractumBadge>
          {#if preview.playlistVideoCount !== null}
            <ExtractumBadge>{preview.playlistVideoCount} videos</ExtractumBadge>
          {/if}
          {#if formatDuration(preview.durationSeconds)}
            <ExtractumBadge>{formatDuration(preview.durationSeconds)}</ExtractumBadge>
          {/if}
        </div>
        <strong>{preview.title ?? preview.externalId}</strong>
        <p>{preview.channelTitle ?? preview.channelHandle ?? preview.canonicalUrl}</p>
        <div class="actions">
          <span>{preview.canonicalUrl}</span>
          <ExtractumButton onclick={addSource} disabled={!canAdd && !canConnectExistingSmartImportSource}>
            <Plus size={14} aria-hidden="true" />
            {#if existingSmartImportSourceConnected}
              Already connected to this project
            {:else if existingSmartImportSource && projectContext}
              {adding ? "Connecting..." : "Connect to project"}
            {:else if existingSmartImportSource}
              Already in Library
            {:else}
              {adding ? "Adding..." : "Add source"}
            {/if}
          </ExtractumButton>
        </div>
      </div>
    </article>
  {/if}
</section>

<style>
  .library-youtube-smart-import {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .entry-row {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    gap: 8px;
    align-items: end;
  }

  label {
    display: grid;
    gap: 4px;
    color: var(--extractum-muted);
    font-size: 13px;
  }

  .preview-card {
    display: grid;
    grid-template-columns: minmax(150px, 220px) minmax(0, 1fr);
    gap: 12px;
    border: 1px solid var(--extractum-border);
    border-radius: var(--extractum-radius);
    padding: 10px;
  }

  .preview-media {
    display: grid;
    place-items: center;
    aspect-ratio: 16 / 9;
    overflow: hidden;
    border-radius: var(--extractum-radius);
    background: var(--extractum-surface-subtle);
    color: var(--extractum-muted);
    font-weight: 700;
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
    gap: 8px;
  }

  .badges,
  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    align-items: center;
  }

  .actions {
    justify-content: space-between;
  }

  .actions span,
  p {
    margin: 0;
    min-width: 0;
    overflow-wrap: anywhere;
    color: var(--extractum-muted);
    font-size: 12px;
  }

  @media (max-width: 760px) {
    .entry-row,
    .preview-card {
      grid-template-columns: 1fr;
    }
  }
</style>
