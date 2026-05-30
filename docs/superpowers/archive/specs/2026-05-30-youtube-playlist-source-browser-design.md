# YouTube Playlist Source Browser Design

> Date: 2026-05-30
> Status: implemented and merged on 2026-05-30
> Scope: live YouTube playlist browsing in the `/analysis` Source canvas.

## Summary

Move live YouTube playlist sources from the legacy playlist reader branch into
the shared live-source browser shell. Playlists should feel like part of the
same Source Browser family as Telegram sources and YouTube videos, while
preserving playlist-specific video membership browsing.

The first slice keeps source groups and saved run snapshots on their existing
readers. It also avoids changing backend storage or playlist sync semantics.

## Pre-Implementation Context

At design time, the shipped live single-source Source Browser handled:

- Telegram live sources with `Timeline | Items | Metadata | Activity`;
- YouTube video live sources with
  `Transcript | Comments | Items | Metadata | Activity`.

YouTube playlist live sources still bypassed `SourceBrowserShell` and rendered
`YoutubePlaylistReader` directly from `ReportSourceSurface`. That reader
owned:

- playlist summary header;
- playlist video membership rows;
- playlist sync/retry actions;
- per-video open/sync/retry actions;
- detailed YouTube source activity cards.

## Goals

- Route live YouTube playlist sources into `SourceBrowserShell`.
- Add a playlist-specific `Videos` tab as the smart default for playlists.
- Extract playlist membership browsing into a new `YoutubePlaylistVideosView`
  leaf for the `Videos` tab.
- Move detailed playlist job cards into the shared `Activity` tab.
- Keep contextual playlist actions reachable from `Videos`.
- Preserve `Open video source` behavior so linked videos open the existing
  YouTube video browser.
- Keep `Items` loaded-window semantics identical to the shipped source browser.
- Add playlist metadata to the existing structured `Metadata` tab without
  exposing arbitrary item raw payloads.

## Non-Goals

- Do not migrate source groups in this slice.
- Do not migrate saved run snapshots in this slice.
- Do not introduce persistent remembered browser tabs.
- Do not add backend-backed full-playlist search/filter/sort.
- Do not change playlist sync, retry, or job persistence semantics.
- Do not download YouTube media.
- Do not add audio/video analysis.
- Do not create a separate playlist-only source browser shell.

## UX Contract

YouTube playlist live sources show:

```text
Videos | Items | Metadata | Activity
```

Smart default:

- YouTube playlist: `Videos`

Tab behavior:

- Switching from a YouTube video tab such as `Items`, `Metadata`, or `Activity`
  to a playlist preserves that tab when the playlist supports it.
- Switching from a YouTube video `Transcript` or `Comments` tab to a playlist
  falls back to `Videos`.
- Switching from a playlist `Videos` tab to a YouTube video falls back to the
  video smart default, `Transcript`.
- Switching from a playlist `Videos` tab to Telegram falls back to the Telegram
  smart default, `Timeline`.

Implementation must make this behavior explicit in model tests for:

| From | To | Active before | Expected after |
| --- | --- | --- | --- |
| YouTube video | YouTube playlist | `metadata` | `metadata` |
| YouTube video | YouTube playlist | `items` | `items` |
| YouTube video | YouTube playlist | `activity` | `activity` |
| YouTube video | YouTube playlist | `transcript` | `videos` |
| YouTube video | YouTube playlist | `comments` | `videos` |
| YouTube playlist | YouTube video | `videos` | `transcript` |
| YouTube playlist | Telegram | `videos` | `timeline` |

### Videos Tab

`Videos` is the playlist-aware primary view. It shows:

- playlist title and channel identity;
- video count, linked count, unavailable count;
- captions/comments availability summary;
- `Sync all` and `Retry failed` contextual actions;
- playlist membership rows with thumbnail, position, title, duration,
  availability, captions/comments state, and published timestamp;
- per-video actions:
  - open linked video source;
  - sync linked video;
  - retry linked video when the availability state is retryable.

The `Videos` tab must not render detailed job cards. Those belong in
`Activity`.

`Videos` consumes only already-loaded playlist detail from route state. The
leaf does not initiate data loading on mount and does not import source APIs.
It can only request work through callback props.

`YoutubePlaylistVideosView` is a leaf view. It owns no tab reconciliation, no
route selection state, and no source job state.

Playlist-level retry and row-level retry are separate actions:

- `Retry failed` calls the playlist-level failed-video retry callback.
- row `Retry this video` calls the linked-video retry callback with that row's
  video source id.

`Open video source` changes the selected source through the route callback. It
must not open a nested video browser inside the playlist browser.

### Items Tab

`Items` uses `UniversalItemsView` and the route-owned `list_source_items`
loaded window. It keeps the shipped wording and semantics:

- search label says `Search loaded items`;
- chips and filters are derived from loaded rows only;
- `Load more` uses the existing source item pagination callback.

This tab may be sparse for playlists because playlist membership rows live in
the typed playlist detail model, not necessarily in generic source items.
Playlist context must explain this explicitly when the loaded window is empty:

```text
Playlist videos live in the Videos tab. This Items tab only shows generic archived items loaded for this playlist source.
```

This is an empty-state copy change, not a semantics change:
`UniversalItemsView` still searches, filters, sorts, and derives chips from the
loaded source-item window only.

### Metadata Tab

`Metadata` should support YouTube playlists inside the existing
`SourceMetadataView` structure. The component should receive the already loaded
playlist detail DTO and render a provider-aware playlist metadata section
without becoming a second playlist membership browser.

The implementation should use optional-safe fields from the already-loaded
playlist detail DTO and source record, including:

- `playlistId`;
- `canonicalUrl`;
- `title`;
- `channelTitle`;
- `channelId`;
- `channelHandle`;
- `availabilityStatus`;
- `videoCount`;
- `linkedCount`;
- `unavailableCount`;
- existing captions/comments summary fields.

Do not extend the backend DTO only to make Metadata richer in this slice.

- Summary: title, kind, channel identity, canonical URL when available,
  created/last synced timestamps.
- Source state: availability, captions/comments state, video count, linked
  count, unavailable count.
- Technical: source id, provider type/subtype, playlist id, channel fields, and
  any safe typed playlist metadata already exposed to the frontend.
- Raw JSON: no playlist raw JSON in this slice. Never read from
  `items.raw_data_zstd` or generic source item raw payloads.

If playlist detail is not loaded, the metadata tab shows the source-level data
that is already available and a compact loading or unavailable state for the
playlist-specific fields.

### Activity Tab

`Activity` uses `SourceActivityView` for playlist jobs and source actions.
It owns detailed source job rendering for:

- playlist sync;
- video metadata/transcript/comments work started from playlist actions;
- retry/cancel states already exposed through source jobs.

The playlist `Videos` tab can keep compact contextual CTAs, but it must not
duplicate job progress, warnings, errors, timestamps, or cancel controls.

## Architecture

Extend the existing shared source browser rather than introducing a new route
state owner.

Frontend model changes:

- add a canonical tab id `videos`;
- add `Videos` label;
- include playlists in `sourceBrowserShellAppliesToSource`;
- derive playlist tabs as `videos`, `items`, `metadata`, `activity`;
- make playlist smart default `videos`;
- update tab reconciliation tests for video-to-playlist and playlist-to-video
  transitions.

Component changes:

- keep `SourceBrowserShell` as the only live single-source browser shell;
- add a `videos` branch for YouTube playlists;
- create `YoutubePlaylistVideosView` as the job-card-free `Videos` leaf;
- remove the direct `YoutubePlaylistReader` branch from `ReportSourceSurface`;
- if `YoutubePlaylistReader` remains after the migration, it is only a
  compatibility wrapper around `YoutubePlaylistVideosView` and does not render
  detailed source activity;
- route `YoutubePlaylistVideosView` through `SourceBrowserShell` from
  `ReportSourceSurface`;
- pass playlist detail into `SourceMetadataView` so Metadata can render
  provider-aware playlist summary/state/technical fields;
- add a playlist-specific empty-state hint to `UniversalItemsView` through an
  optional `emptyDescription` prop;
- keep source groups and run snapshots on their existing reader branches.

Data ownership:

- `/analysis/+page.svelte` and parent surfaces remain responsible for loading
  playlist detail, source items, and source jobs;
- shell and leaf views trigger work through existing callbacks;
- no leaf view imports Tauri API wrappers or calls `invoke`.

Callback naming in the implementation plan should keep playlist and linked
video operations distinct:

- playlist-level sync: `onSyncPlaylist`;
- playlist-level retry failed: `onRetryFailedPlaylistVideos`;
- row-level linked-video sync: `onSyncPlaylistVideo`;
- row-level linked-video retry: `onRetryPlaylistVideo`;
- open linked video source: `onOpenSource`.

## Data Flow

1. The user selects a live single YouTube playlist source.
2. `/analysis` loads the existing playlist detail and source job state.
3. `ReportSourceSurface` routes the source into `SourceBrowserShell`.
4. `SourceBrowserShell` derives playlist tabs and opens `Videos`.
5. The `Videos` tab renders playlist detail and contextual playlist/video
   actions through props.
6. `Activity` renders the same source jobs through `SourceActivityView`.
7. Opening a linked video source changes the selected source and enters the
   already shipped YouTube video browser.

The playlist browser never nests the YouTube video browser. Video browsing is a
source selection transition, not an in-place child view.

## Error And Empty States

- Loading playlist detail: show a compact loading state in `Videos`.
- Missing playlist detail: show a muted state with available source identity.
- Empty playlist items: keep the existing "sync playlist to load video rows"
  explanation.
- Runtime diagnostic from `sourceSyncDisabledReason`: keep the existing
  top-level source diagnostic before the shell.
- Failed or active jobs: show detailed information only in `Activity`.
- Unlinked or unavailable video rows: keep their disabled open/sync/retry
  affordances and availability badges.

## Testing

Frontend contract tests should assert:

- playlist sources enter `SourceBrowserShell`;
- source groups and saved snapshots still use existing readers;
- `sourceBrowserTabsForSource(source)` returns
  `videos`, `items`, `metadata`, `activity` for YouTube playlists;
- `smartDefaultSourceBrowserTab(source)` returns `videos` for playlists;
- `reconcileSourceBrowserTab(previousTab, nextSource)` preserves shared tabs
  and falls back from unsupported video-only or playlist-only tabs according to
  the transition matrix above;
- `SourceBrowserShell` renders `YoutubePlaylistVideosView` for `videos`;
- `YoutubePlaylistVideosView` does not render detailed source activity cards;
- `YoutubePlaylistVideosView` consumes playlist detail only through props and
  imports no `$lib/api/` modules;
- structural tests should check that `YoutubePlaylistVideosView` does not
  import or render `SourceActivityView` and does not accept activity-specific
  props such as `sourceJobs` or `onCancelSourceJob`;
- playlist-level retry and row-level retry use distinct callback props;
- `Open video source` is wired to the existing route source-selection callback
  and no nested video detail view is introduced;
- `SourceMetadataView` renders playlist metadata from the playlist detail DTO
  and does not read item raw payloads;
- `UniversalItemsView` can show playlist-specific empty guidance without
  changing loaded-window search/filter/sort semantics;
- `SourceActivityView` remains wired for playlist source jobs;
- shell still imports no `$lib/api/` modules and does not call `invoke(`.

Manual smoke should verify:

- fixture YouTube playlist opens in the Source Browser on `Videos`;
- `Items`, `Metadata`, and `Activity` tabs render;
- `Open video source` enters the existing YouTube video browser;
- returning/selecting the playlist preserves supported shared tabs;
- detailed playlist jobs appear in `Activity`, not in `Videos`;
- source groups and saved snapshots still render through old readers.

## Rollout Notes

This slice intentionally migrates only live YouTube playlists. It prepares the
browser model for richer playlist/video navigation without introducing nested
video browsing in this slice or forcing source groups and frozen snapshots into
the same contract.
