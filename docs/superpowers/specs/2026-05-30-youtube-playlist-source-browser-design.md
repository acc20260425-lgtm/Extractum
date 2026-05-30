# YouTube Playlist Source Browser Design

> Date: 2026-05-30
> Status: approved design, pending implementation plan
> Scope: live YouTube playlist browsing in the `/analysis` Source canvas.

## Summary

Move live YouTube playlist sources from the legacy playlist reader branch into
the shared live-source browser shell. Playlists should feel like part of the
same Source Browser family as Telegram sources and YouTube videos, while
preserving playlist-specific video membership browsing.

The first slice keeps source groups and saved run snapshots on their existing
readers. It also avoids changing backend storage or playlist sync semantics.

## Current Context

The shipped live single-source Source Browser currently handles:

- Telegram live sources with `Timeline | Items | Metadata | Activity`;
- YouTube video live sources with
  `Transcript | Comments | Items | Metadata | Activity`.

YouTube playlist live sources still bypass `SourceBrowserShell` and render
`YoutubePlaylistReader` directly from `ReportSourceSurface`. That reader
currently owns:

- playlist summary header;
- playlist video membership rows;
- playlist sync/retry actions;
- per-video open/sync/retry actions;
- detailed YouTube source activity cards.

## Goals

- Route live YouTube playlist sources into `SourceBrowserShell`.
- Add a playlist-specific `Videos` tab as the smart default for playlists.
- Reuse the existing playlist membership UI as the `Videos` tab body.
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
- Do not create a separate playlist-only source browser shell unless the shared
  shell cannot remain readable during implementation.

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

### Items Tab

`Items` uses `UniversalItemsView` and the route-owned `list_source_items`
loaded window. It keeps the shipped wording and semantics:

- search label says `Search loaded items`;
- chips and filters are derived from loaded rows only;
- `Load more` uses the existing source item pagination callback.

This tab may be sparse for playlists because playlist membership rows live in
the typed playlist detail model, not necessarily in generic source items.

### Metadata Tab

`Metadata` should support YouTube playlists using the existing
`SourceMetadataView` structure:

- Summary: title, kind, channel identity, canonical URL when available,
  created/last synced timestamps.
- Source state: availability, captions/comments state, video count, linked
  count, unavailable count.
- Technical: source id, provider type/subtype, playlist id, channel fields, and
  any safe typed playlist metadata already exposed to the frontend.
- Raw JSON: no playlist raw JSON in this slice unless the existing playlist
  detail DTO already exposes explicit source-level sanitized raw metadata. Do
  not read from item raw payloads.

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
- convert `YoutubePlaylistReader` into a job-card-free `Videos` leaf, or split
  its playlist rows into a new `YoutubePlaylistVideosView` while preserving the
  current UI contract;
- route `YoutubePlaylistReader`/playlist view through the shell from
  `ReportSourceSurface`;
- keep source groups and run snapshots on their existing reader branches.

Data ownership:

- `/analysis/+page.svelte` and parent surfaces remain responsible for loading
  playlist detail, source items, and source jobs;
- shell and leaf views trigger work through existing callbacks;
- no leaf view imports Tauri API wrappers or calls `invoke`.

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
- `sourceBrowserTabsForSource` returns
  `videos`, `items`, `metadata`, `activity` for YouTube playlists;
- playlist smart default is `videos`;
- playlist tab reconciliation preserves shared tabs and falls back from
  unsupported video-only tabs;
- `SourceBrowserShell` renders the playlist videos leaf for `videos`;
- `YoutubePlaylistReader` or the new playlist videos leaf no longer renders
  detailed source activity cards;
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
browser model for richer nested playlist/video browsing without forcing source
groups or frozen snapshots into the same contract.
