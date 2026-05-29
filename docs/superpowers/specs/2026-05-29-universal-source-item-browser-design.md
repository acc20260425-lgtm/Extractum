# Universal Source Item Browser Design

> Date: 2026-05-29
> Status: approved design, pending implementation plan
> Scope: live single-source browsing in the `/analysis` Source canvas.

## Summary

Replace the live single-source reader split with a shared Source Browser shell
that can present provider-aware views and a universal item browser over source
items. The first slice applies only to live single-source browsing. Source
groups and saved run snapshots keep their current readers.

The browser should preserve the strongest existing experiences:

- YouTube videos still open on the transcript reader by default.
- Telegram sources still open on the timeline reader by default.
- Source jobs and status become a first-class Activity tab.
- Comments, metadata, and all item kinds become directly inspectable.

## Current Context

Current source browsing has separate paths:

- Telegram live sources render through `TelegramTimelineReader`.
- YouTube video live sources render through `YoutubeTranscriptReader`.
- YouTube transcript segments use a dedicated paged API because one
  `youtube_transcript` item expands into many timestamped segments.
- YouTube comments are synced as `youtube_comment` item rows and can be used in
  analysis, but there is no direct reader UI for them.
- YouTube source metadata is summarized in the source switcher and transcript
  reader badges, but there is no complete live metadata view.
- `list_source_items` already exposes generic item rows with `itemKind`,
  author, content, media, forum topic, reply, reaction, history scope, and raw
  availability fields.

Known item kinds today:

- `telegram_message`
- `youtube_transcript`
- `youtube_comment`
- `youtube_description` as a synthetic analysis document/ref, not a normal
  live `items` row today

## Goals

- Add a live single-source Source Browser shell that owns tab selection.
- Keep provider-aware default tabs for readable, domain-shaped browsing.
- Add a universal `Items` tab for all live item rows and provider item kinds.
- Add a YouTube comments view with threaded and flat modes.
- Add a metadata view with readable summary, technical fields, and optional raw
  JSON for YouTube video metadata.
- Move source activity rendering out of the transcript reader into an Activity
  tab.
- Keep the design extendable for future RSS/forum item kinds without requiring
  a new browser architecture.

## Non-Goals

- Do not migrate source groups in this slice.
- Do not migrate saved run snapshots in this slice.
- Do not add persistent remembered tabs.
- Do not create a universal raw-data viewer for every item row.
- Do not change the database schema unless implementation proves existing
  typed tables and `raw_data_zstd` cannot serve the selected UI.
- Do not expose arbitrary `items.raw_data_zstd` payloads in the universal item
  browser.
- Do not send media bytes or source content to any external provider.
- Do not hide YouTube media downloads behind source browsing.

## UX Design

### Source Browser Shell

The live single-source Source canvas renders one shell. The shell derives its
tabs from `sourceType` and `sourceSubtype`, resets to a smart default when the
selected source changes, and keeps active tab state local to the route.

Smart defaults:

- YouTube video: `Transcript`
- Telegram: `Timeline`
- unknown or future source types: `Items`

### YouTube Video Tabs

YouTube videos show:

```text
Transcript | Comments | Items | Metadata | Activity
```

- `Transcript` preserves the current timestamped transcript reader, grouping,
  selected trace ref scroll, and copy timestamp link behavior.
- `Comments` shows synced `youtube_comment` rows.
- `Items` shows all live item rows for the source.
- `Metadata` shows human-readable and technical source metadata.
- `Activity` shows YouTube source jobs and status.

### Telegram Tabs

Telegram sources show:

```text
Timeline | Items | Metadata | Activity
```

- `Timeline` preserves the current bubble/timeline view, topic filter,
  migrated-history scope controls, media cards, reply badges, reaction badges,
  and selected trace ref scroll behavior.
- `Items` shows the same live rows through the universal item browser.
- `Metadata` shows source-level Telegram metadata, sync state, membership state,
  migrated-history state, and topic resolver state when already loaded.
- `Activity` shows sync, Takeout, migrated-history, recovery, and related
  source status already available to the route.

### Universal Items Tab

The universal `Items` tab renders normalized source reader items and provides:

- item kind filter chips derived from loaded rows, such as `All`, `Messages`,
  `Transcript`, `Comments`, `Media`, and `Other`;
- search over visible content and author;
- sort modes appropriate to loaded rows, starting with newest and oldest;
- generic item cards for unknown future item kinds;
- existing badges for topic, reply, reaction, source ref, history scope, media,
  and raw-data availability when useful.

The tab is a browser and diagnostic surface, not the primary polished view for
every provider. Provider-aware tabs remain first-class where they exist.

### YouTube Comments View

The `Comments` tab is specific to YouTube video live sources but uses the
generic item model plus YouTube comment enrichment.

Controls:

- mode toggle: `Threaded` and `Flat`;
- default mode: `Threaded`;
- search by comment text or author;
- sort: `Newest`, `Oldest`, `Most liked`;
- filters: `Top-level only`, `Replies`, `Pinned`, `Hearted`;
- `Load more` pagination using live item pagination.

Comment cards show:

- author;
- published timestamp;
- comment text;
- like count;
- reply/top-level state;
- pinned and hearted badges when available;
- comment id or source ref only in compact technical badges/tooltips.

When comments are not synced, the tab shows an empty state with `Sync comments`.

### Pagination And Filtering Contract

The first slice is a loaded-page browser. `Items` and `Comments` search,
filters, sort order, and derived chips operate only on rows already loaded into
the current source view unless a future API explicitly backs those parameters.

Required UI wording:

- search labels must use scoped wording such as `Search loaded items` or
  `Search loaded comments`;
- empty states must say whether no rows match the loaded window rather than the
  full source;
- item kind chips are derived from loaded rows only and must not imply that
  missing kinds are absent from the full source;
- `Most liked` sorts loaded comments only.

Backend-backed full-source search, sort, item-kind filtering, and
comment-specific filtering are a follow-up. That follow-up would require query
parameters such as item kind, search text, sort mode, and provider-specific
filters.

Threaded comments also group only comments available in the loaded window.
Replies whose parent comment is not loaded are rendered as standalone reply
cards with a `parent not loaded` indicator instead of being hidden. Backend
whole-thread paging is a follow-up.

### Metadata View

The `Metadata` tab must be structured, not a dump. It is divided into:

1. Summary: user-readable identity and content facts.
2. Source state: sync, membership, captions/comments, migrated-history, topic,
   and job-related state that describes current local coverage.
3. Technical: ids and provider/runtime fields useful for debugging.
4. Raw JSON: explicit collapsed payload viewer when available.

For YouTube video:

- title;
- channel title/handle;
- canonical URL;
- thumbnail URL when present;
- published date;
- duration;
- availability;
- captions and comments sync status;
- playlist memberships;
- source id and external video id;
- last synced timestamps;
- technical fields from the typed YouTube metadata table;
- collapsed `Show raw JSON` for the stored metadata payload when available.

YouTube fields should be grouped into Summary, Source state, Technical, and Raw
JSON sections rather than one long list.

For Telegram:

- title;
- source type/subtype;
- source id and external id;
- account id;
- username when present;
- membership state;
- sync state and last synced timestamp;
- migrated-history state and row count;
- topic resolver state when loaded.

Telegram fields should use the same section structure: Summary for title/type,
Source state for membership/sync/migrated-history/topic readiness, Technical
for ids, and no Raw JSON unless a future explicit raw metadata contract exists.

Raw JSON must be explicit and collapsed by default. It is exposed only for the
source-level YouTube metadata payload selected by `get_youtube_video_detail`,
not for arbitrary item `raw_data_zstd` rows. If raw metadata is unavailable,
the raw control is hidden or disabled with a short explanation.

### Activity View

The `Activity` tab consolidates source work status:

- YouTube metadata/transcript/comments/playlist jobs for YouTube sources;
- Telegram sync state;
- Takeout import jobs and recovery notices;
- migrated-history import state;
- terminal warnings and errors already exposed by existing route state.

Activity is a status-and-actions surface. It may contain source-level actions
such as sync metadata, sync transcript, sync comments, retry/cancel source jobs,
start Takeout, or start migrated-history import when those callbacks already
exist in route state. Provider tabs may still show contextual CTAs in empty or
stale states, but those buttons must call the same callbacks and use the same
disabled reasons as Activity. Activity owns the consolidated job/status view;
provider tabs should not duplicate detailed job cards.

The first slice should reuse existing job/status data. It should not introduce
new background job APIs only for this tab.

## Architecture

### Components

Add a shared live-source shell:

- `SourceBrowserShell.svelte`
  - owns tab derivation and local active tab state;
  - receives route-level data and callbacks;
  - renders provider-aware views and the universal `Items` tab;
  - applies smart default on source changes.

Keep provider-aware views small:

- `YoutubeTranscriptView` or existing `YoutubeTranscriptReader` wrapper;
- `YoutubeCommentsView`;
- `TelegramTimelineView` or existing `TelegramTimelineReader` wrapper;
- `UniversalItemsView`;
- `SourceMetadataView`;
- `SourceActivityView`.

Avoid a broad frontend service layer. Existing route state and API modules
should feed the shell until a later slice proves a clearer boundary is needed.

### Frontend Model

Extend `source-reader-model.ts` or an adjacent source browser model module with:

- provider item kind labels;
- item kind filter derivation;
- generic card display model;
- YouTube comment enrichment helpers;
- threaded comment grouping from `commentId` and `parentCommentId`;
- flat/threaded filter/sort/search helpers.

### Backend/API

Keep `list_source_items` as the generic item-row source. Extend it only where
generic browsing needs common fields.

Add or extend YouTube-specific DTOs where the UI needs provider detail:

- extend YouTube comment rows with fields derived from the stored comment raw
  payload: `commentId`, `parentCommentId`, `isReply`, `likeCount`, `isPinned`,
  `isHearted`, `authorChannelUrl`;
- extend `get_youtube_video_detail` with safe metadata fields and optional raw
  source-level metadata JSON for the `Metadata` tab.

Do not move transcript segments into generic item pagination. The dedicated
transcript segment API stays because it has timestamp-specific pagination and
search behavior.

## Data Flow

1. The route selects a live single source.
2. `SourceBrowserShell` derives available tabs from the source.
3. Provider-aware tabs receive existing route state where possible.
4. `Transcript` loads segments through `list_youtube_transcript_segments`.
5. `Comments` and `Items` use generic source item rows plus frontend filtering
   over the currently loaded page; they do not request full-source
   search/filter/sort semantics in this slice.
6. `Metadata` combines `currentSource`, provider detail DTOs, and loaded topic
   state when relevant.
7. `Activity` reads already available source jobs, Takeout recovery state, and
   source status, and exposes the same route callbacks used by contextual CTAs.

## Error And Empty States

- Missing comments: show an empty state and `Sync comments`.
- Failed comment sync: show failed job state in Activity and keep the Comments
  tab readable for any previously synced rows.
- Missing YouTube typed metadata: show a metadata error and `Sync metadata`.
- Missing raw JSON: hide or disable `Show raw JSON`.
- Unknown item kind: render a generic item card.
- No rows after filters/search: show an empty state that says no loaded rows
  match and offers `Clear filters`.
- Orphan YouTube replies in threaded mode: render as standalone reply cards with
  a parent-not-loaded badge.
- Route-level API failures continue to use the existing formatted app error
  behavior.

## Testing

Frontend tests:

- tab mapping and smart defaults for YouTube, Telegram, and unknown source
  types;
- route contract showing live single-source uses the new shell while source
  groups and snapshots stay on existing readers;
- universal item kind filter derivation from loaded rows, scoped loaded-item
  search labels, and generic item card fallback;
- YouTube comments threaded/flat grouping, loaded-comment search, local sort,
  filters, and orphan reply rendering;
- metadata view renders Summary, Source state, Technical, and collapsed Raw JSON
  sections, with raw JSON limited to source-level YouTube metadata;
- Activity tab receives existing source jobs/status and source-level actions
  without provider tabs duplicating detailed job cards.

Rust/backend tests:

- `get_youtube_video_detail` exposes the selected metadata fields and raw JSON
  without regressing existing summary/membership behavior;
- `list_source_items` or the chosen comment-detail path exposes YouTube comment
  enrichment from stored raw payload;
- unknown or malformed raw comment payloads degrade without command failure when
  the base item row remains valid.
- item raw-data payloads are not exposed by the generic item browser contract.

Manual verification:

- open a Telegram source and confirm `Timeline | Items | Metadata | Activity`;
- open a YouTube video with synced transcript/comments and confirm
  `Transcript | Comments | Items | Metadata | Activity`;
- confirm Comments threaded and flat modes;
- confirm search/filter/sort labels and empty states communicate loaded-row
  scope;
- confirm orphan replies render visibly if a loaded page contains a reply whose
  parent is not loaded;
- confirm Metadata is grouped into Summary, Source state, Technical, and Raw
  JSON sections;
- confirm Raw JSON is limited to source-level YouTube metadata, not arbitrary
  item rows;
- confirm Activity shows consolidated status and actions while provider tabs
  keep only contextual CTAs;
- confirm `Sync comments` and `Sync metadata` actions are reachable from empty
  or stale states;
- confirm source groups and saved run snapshots keep their previous readers.

## Rollout Notes

This design intentionally creates the live-source browser first. After it is
stable, follow-up slices can migrate:

- source group readers;
- saved run snapshots;
- richer playlist/video nested browsing;
- RSS/forum item kinds;
- persistent remembered tab state;
- richer raw-data inspection policies.
