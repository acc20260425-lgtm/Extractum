# YouTube Playlist Source Browser - Historical Note

> Status: shipped and archived. Current source browsing behavior is documented
> in root docs and implemented by Source Browser components.

## Decision

YouTube playlists use the shared Source Browser surface with playlist-specific
tabs for videos/items, metadata, and activity. Runtime browsing should use typed
playlist rows rather than decoding raw provider payloads.

## Rationale

- Playlists need the same provider-neutral navigation shell as Telegram and
  saved-run sources.
- Playlist entries are ordered source material, so the browser should expose
  item structure instead of treating the playlist as one blob.
- Typed `youtube_playlist_items` data keeps provider metadata queryable and
  testable.

## Preserved Contract

- Use Source Browser shell patterns.
- Keep playlist videos/items separate from source metadata and activity.
- Treat YouTube-specific NotebookLM export enrichment as separate future work.
