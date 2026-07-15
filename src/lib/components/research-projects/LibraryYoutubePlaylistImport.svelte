<script lang="ts">
  import Plus from "@lucide/svelte/icons/plus";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import { SvelteSet } from "svelte/reactivity";
  import {
    ExtractumBadge,
    ExtractumButton,
    ExtractumStatusMessage,
    ExtractumTextInput,
  } from "$lib/components/extractum-ui";
  import { addYoutubeSource } from "$lib/api/sources";
  import { getYoutubePlaylistDetail } from "$lib/api/youtube-detail";
  import { formatAppError } from "$lib/app-error";
  import {
    YOUTUBE_PLAYLIST_IMPORT_LIMIT,
    buildPlaylistImportRows,
    libraryYoutubePlaylistSources,
    playlistSelectionLimitMessage,
    selectedAddablePlaylistRows,
    type PlaylistImportSummary,
  } from "$lib/ui/library-add-source-model";
  import { addSelectedYoutubePlaylistVideos } from "$lib/ui/library-add-source-workflow";
  import type { LibraryCatalogSourceView } from "$lib/ui/library-catalog-model";
  import type { ProjectAddSourceContext } from "$lib/ui/project-add-source-context";
  import type { YoutubePlaylistDetail } from "$lib/types/youtube";

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

  let playlistQuery = $state("");
  let selectedPlaylistId = $state<number | null>(null);
  let detail = $state<YoutubePlaylistDetail | null>(null);
  let loadingDetail = $state(false);
  let adding = $state(false);
  let selectedVideoIds = new SvelteSet<string>();
  let status = $state("");
  let summary = $state<PlaylistImportSummary | null>(null);

  const playlists = $derived(libraryYoutubePlaylistSources(sources));
  const filteredPlaylists = $derived.by(() => {
    const query = playlistQuery.trim().toLocaleLowerCase();
    if (!query) return playlists;
    return playlists.filter((source) =>
      `${source.title} ${source.subtitle ?? ""} ${source.externalId ?? ""}`.toLocaleLowerCase().includes(query),
    );
  });
  const rows = $derived(buildPlaylistImportRows(detail));
  const selectedRows = $derived(selectedAddablePlaylistRows(rows, selectedVideoIds));
  const limitMessage = $derived(playlistSelectionLimitMessage(selectedRows.length));
  const canAddSelected = $derived(selectedRows.length > 0 && !limitMessage && !adding);

  async function loadPlaylist(sourceId: number) {
    selectedPlaylistId = sourceId;
    detail = null;
    summary = null;
    status = "";
    selectedVideoIds.clear();
    loadingDetail = true;
    try {
      detail = await getYoutubePlaylistDetail(sourceId);
    } catch (error) {
      status = formatAppError("loading YouTube playlist", error);
    } finally {
      loadingDetail = false;
    }
  }

  function toggleVideo(id: string) {
    if (selectedVideoIds.has(id)) {
      selectedVideoIds.delete(id);
    } else {
      selectedVideoIds.add(id);
    }
  }

  async function addSelected() {
    if (!canAddSelected) return;
    adding = true;
    status = "";
    try {
      summary = await addSelectedYoutubePlaylistVideos({
        rows: selectedRows,
        addYoutubeSource,
        formatError: formatAppError,
      });
      if (summary.added > 0) {
        onStatus(`Added ${summary.added} YouTube video source${summary.added === 1 ? "" : "s"}.`);
        if (projectContext) {
          const addedSourceIds = summary.results
            .filter((result) => result.status === "added" && result.sourceId !== null)
            .map((result) => result.sourceId as number);
          if (addedSourceIds.length > 0) {
            await projectContext.onConnectAddedSources(addedSourceIds);
          }
        } else {
          await onSourcesChanged(summary.results.find((result) => result.sourceId !== null)?.sourceId ?? undefined);
        }
      }
    } finally {
      adding = false;
    }
  }
</script>

<section class="library-youtube-playlist-import" aria-label="YouTube playlist import">
  <div class="playlist-picker">
    <label>
      <span>Playlist search</span>
      <ExtractumTextInput
        value={playlistQuery}
        placeholder="Search playlists"
        oninput={(event) => (playlistQuery = (event.currentTarget as HTMLInputElement).value)}
      />
    </label>
    <ExtractumBadge>{filteredPlaylists.length} playlists</ExtractumBadge>
  </div>

  {#if playlists.length === 0}
    <ExtractumStatusMessage tone="muted">No YouTube playlists are in Library yet.</ExtractumStatusMessage>
  {:else}
    <div class="playlist-list" aria-label="Existing YouTube playlists">
      {#each filteredPlaylists as playlist (playlist.id)}
        <button
          type="button"
          class:selected={playlist.sourceId === selectedPlaylistId}
          onclick={() => void loadPlaylist(playlist.sourceId)}
        >
          <strong>{playlist.title}</strong>
          <span>{playlist.subtitle ?? playlist.externalId ?? "YouTube playlist"}</span>
        </button>
      {/each}
    </div>
  {/if}

  {#if status}
    <ExtractumStatusMessage tone={status.startsWith("Error") ? "error" : "default"}>{status}</ExtractumStatusMessage>
  {/if}

  {#if loadingDetail}
    <ExtractumStatusMessage tone="muted">Loading playlist videos...</ExtractumStatusMessage>
  {:else if detail}
    <div class="video-toolbar">
      <div>
        <strong>{detail.summary.title ?? "Playlist videos"}</strong>
        <span>{selectedRows.length} selected, limit {YOUTUBE_PLAYLIST_IMPORT_LIMIT}</span>
      </div>
      <ExtractumButton onclick={addSelected} disabled={!canAddSelected}>
        {#if adding}
          <RefreshCw size={14} aria-hidden="true" />
          Adding...
        {:else}
          <Plus size={14} aria-hidden="true" />
          Add selected
        {/if}
      </ExtractumButton>
    </div>

    {#if limitMessage}
      <ExtractumStatusMessage tone="error">{limitMessage}</ExtractumStatusMessage>
    {/if}

    <div class="video-list" aria-label="Playlist videos">
      {#each rows as row (row.id)}
        <label class:disabled={!row.addable}>
          <input
            type="checkbox"
            checked={selectedVideoIds.has(row.id)}
            disabled={!row.addable || adding}
            onchange={() => toggleVideo(row.id)}
          />
          <span>
            <strong>{row.item.title ?? row.item.videoId}</strong>
            <small>{row.disabledReason ?? row.item.canonicalUrl}</small>
          </span>
          {#if row.disabledReason}
            <ExtractumBadge>
              {row.disabledReason === "Already in Library" ? "Already in Library" : row.disabledReason}
            </ExtractumBadge>
          {/if}
        </label>
      {/each}
    </div>
  {/if}

  {#if summary}
    <ExtractumStatusMessage tone={summary.failed > 0 ? "error" : "default"}>
      Added {summary.added}, skipped {summary.skipped}, failed {summary.failed}.
    </ExtractumStatusMessage>
    <ul class="import-result-list" aria-label="Playlist import results">
      {#each summary.results as result (result.id)}
        <li class={result.status}>
          <strong>{result.title}</strong>
          <span>{result.message ?? result.status}</span>
        </li>
      {/each}
    </ul>
  {/if}
</section>

<style>
  .library-youtube-playlist-import {
    display: grid;
    gap: 12px;
  }

  .playlist-picker,
  .video-toolbar {
    display: flex;
    gap: 8px;
    align-items: end;
    justify-content: space-between;
  }

  label {
    display: grid;
    gap: 4px;
    color: var(--extractum-muted);
    font-size: 13px;
  }

  .playlist-list,
  .video-list {
    display: grid;
    gap: 6px;
    max-height: 280px;
    overflow: auto;
    border: 1px solid var(--extractum-border);
    border-radius: var(--extractum-radius);
    padding: 6px;
  }

  .playlist-list button,
  .video-list label {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    gap: 6px;
    align-items: center;
    border: 1px solid transparent;
    border-radius: var(--extractum-radius);
    padding: 8px;
    background: transparent;
    color: var(--extractum-text);
    text-align: left;
  }

  .import-result-list {
    display: grid;
    gap: 4px;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .import-result-list li {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(120px, auto);
    gap: 8px;
    align-items: center;
    border: 1px solid var(--extractum-border);
    border-radius: var(--extractum-radius);
    padding: 6px 8px;
    font-size: 12px;
  }

  .import-result-list li.failed {
    border-color: var(--status-error-text);
    color: var(--status-error-text);
  }

  .playlist-list button.selected,
  .playlist-list button:hover,
  .video-list label:hover {
    border-color: var(--extractum-border);
    background: var(--extractum-surface-subtle);
  }

  .video-list label {
    grid-template-columns: auto minmax(0, 1fr) auto;
  }

  .video-list label.disabled {
    opacity: 0.66;
  }

  strong,
  small,
  .video-toolbar span {
    min-width: 0;
    overflow-wrap: anywhere;
  }

  small,
  .playlist-list span,
  .video-toolbar span {
    color: var(--extractum-muted);
    font-size: 12px;
  }
</style>
