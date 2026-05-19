# YouTube Playlist-Entry Read-Model Decision

> Scope: Database Schema Simplification decision-only slice.

## Decision

YouTube playlist-entry browsing stays on typed YouTube detail/readers. Do not
materialize playlist membership rows into `archive_read_items`.

`youtube_playlist_items` remains the canonical typed model for playlist
membership and list state. `archive_read_items` remains the item-level
archive/read UI model for actual archive rows, such as source browsing items
and Telegram NotebookLM export rows.

## Rationale

A YouTube playlist entry is membership/list state, not an archived document or
ingested content row. It records a video's relationship to a playlist:
position, `video_id`, linked `video_source_id`, availability/removal state, and
entry snapshots. That relationship should remain in typed YouTube tables and
detail readers.

Transcript, comment, and description data for materialized videos may still
participate in archive and analysis models. The playlist membership row itself
does not become an archive document.

## Boundaries

- `youtube_playlist_sources` owns typed metadata for playlist sources.
- `youtube_playlist_items` owns playlist membership rows and per-entry
  availability/removal state.
- `youtube_video_sources` owns typed metadata for materialized video sources.
- YouTube playlist detail/browsing continues to read typed YouTube tables and
  typed detail readers.
- Later UI for linked, unavailable, removed, upcoming, live, auth-gated,
  deleted, or unknown-unavailable entries should improve the typed YouTube
  playlist detail surface rather than introduce archive item paging.

## Non-Goals

- No Rust code changes.
- No SQL migration.
- No `archive_read_items` schema change.
- No archive builder change.
- No `ARCHIVE_READ_MODEL_VERSION` bump.
- No cleanup of duplicated canonical removal state between
  `availability_status` and `is_removed_from_playlist`.
- No YouTube NotebookLM export enrichment.

## Documentation Updates

- Update the archive read-model decision matrix so playlist membership,
  linked/unlinked entries, availability, and removal state are typed YouTube
  detail/list state, not `archive_read_items` rows.
- Update `youtube_playlist_items` schema docs to state that playlist entries
  intentionally remain typed membership/detail state.
- Remove the open Database Schema Simplification backlog item asking whether
  playlist-entry browsing needs archive rows or typed detail only.
- Keep future UI improvements for playlist entry display in the YouTube
  follow-up area.

## Resulting Backlog Shape

After this decision, strict Database Schema Simplification work is mainly the
current-schema baseline after the read-model boundary stabilizes. Optional
Telegram metadata blob cleanup remains blocked on typed repair and real
private/dialog-backed source validation.
