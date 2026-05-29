# Universal Source Item Browser Design

> Date: 2026-05-29
> Status: approved design, pending implementation plan
> Scope: live single-source browsing in the `/analysis` Source canvas.

## Summary

Replace the live single-source reader split with a shared Source Browser shell
that can present provider-aware views and a universal item browser over source
items. The first slice applies only to live single-source browsing. Source
groups, saved run snapshots, and YouTube playlist live sources keep their
current readers.

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

The live single-source Source canvas renders the shell only for Telegram live
sources and YouTube video live sources in this slice. YouTube playlist live
sources keep `YoutubePlaylistReader`; richer playlist/video nested browsing is
a follow-up. The shell derives its tabs from `sourceType` and `sourceSubtype`,
opens a smart default when no compatible active tab exists, and keeps active
tab state local to the shell.

Canonical tab ids are stable implementation contracts and tests should assert
ids, not display labels:

```text
timeline
transcript
comments
items
metadata
activity
```

Smart defaults:

- YouTube video: `Transcript`
- Telegram: `Timeline`
- unknown or future source types: `Items`

Tab selection contract:

- `SourceBrowserShell` owns active tab state with local Svelte `$state`;
- the route does not store or pass `activeSourceBrowserTab`;
- the shell receives the selected source identity and type fields, including
  `source.id`, `sourceType`, and `sourceSubtype`;
- the shell reconciles the active tab whenever the selected source changes or
  the available tab ids change;
- first entry into a live source opens the smart default tab for that source;
- when `source.id` changes, the shell preserves the current active tab if the
  new source exposes the same tab id;
- when the new source does not expose the current tab, the shell resets to the
  smart default for the new source;
- switching between YouTube videos in the same playlist can therefore preserve
  `Comments`, `Items`, `Metadata`, or `Activity` instead of always returning to
  `Transcript`;
- refreshes of data for the same `source.id`, such as new jobs, metadata, or
  appended items, preserve the current active tab;
- persistent remembered tabs remain a follow-up and are not part of this slice.

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
- local sort modes over loaded rows: `newest`, `oldest`, and provider-specific
  loaded-row modes such as `most_liked` only when the loaded DTO exposes the
  needed field;
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

The comments tab renders from a small derived coverage state:

```ts
type CommentsCoverageState =
  | "unknown"
  | "not_synced"
  | "syncing"
  | "failed"
  | "synced_empty"
  | "synced_with_rows";
```

The state is computed from existing source detail, synced item rows, source
jobs, metadata/comment availability fields, and route errors. It must not
require a new database column in this slice.
The frontend helper should take an object input, such as
`commentsCoverageState({ items, detail, jobs, routeError, loadingItems })`, so
future route-owned coverage signals can be added without rewriting positional
call sites.

Comments coverage behavior:

- `unknown`: metadata/detail coverage is missing or insufficient to decide, or
  comments support is disabled/unavailable but not represented as a separate
  state in this slice; show a muted or disabled explanation and show `Sync
  metadata` or `Sync comments` only when the corresponding route callback is
  available and meaningful;
- `not_synced`: comments have never been synced for this source; show `Sync
  comments`;
- `syncing`: a comments sync job is running or queued; show a compact pending
  label and route users to Activity for job details;
- `failed`: the latest relevant comments sync failed; keep any previously
  loaded rows readable and show a compact retry CTA plus Activity details;
- `synced_empty`: comments sync succeeded and produced zero comment rows; show
  an empty state that says no comments are available locally;
- `synced_with_rows`: render the loaded comment rows.

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

Default page sizes:

- `Items` and `Comments` share the existing live source item page size:
  `SOURCE_ITEMS_PAGE_LIMIT = 120`;
- each `Load more` request appends another loaded window of up to 120 item rows;
- YouTube transcript segments keep their existing dedicated page size of 80;
- source groups and saved run snapshots keep their current page sizes in this
  slice.

Threaded comments also group only comments available in the loaded window.
Replies whose parent comment is not loaded are rendered as standalone reply
cards with a `parent not loaded` indicator instead of being hidden. Backend
whole-thread paging is a follow-up.

Threaded grouping must stay linear over the currently loaded rows. The frontend
builds an in-memory map by `commentId` plus parent buckets, then renders parent
cards with loaded replies beneath them. It must not issue extra requests only to
resolve missing parents. If a later `Load more` page brings in the parent, the
loaded-window grouping is recomputed and the reply moves under its parent.

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

Raw JSON rendering must be bounded:

- pretty-print JSON only after the user expands the section;
- render the payload in a fixed max-height scroll area;
- provide a copy button for the full raw payload when available;
- if the payload is large, show a compact large-payload notice and render a
  truncated preview instead of inserting the entire pretty-printed string into
  the visible DOM;
- never block initial Metadata tab rendering on raw JSON formatting.

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

A contextual CTA is a compact, local control that helps the current tab become
useful, such as `Sync comments` in an empty Comments tab, `Sync transcript` in
an empty Transcript tab, `Sync metadata` in an incomplete Metadata tab, or
`Clear filters` after loaded-row filtering removes every visible row. It may
show one short disabled reason or pending label, but it does not list jobs,
progress history, timestamps, warnings, or per-job actions.

A detailed job card is any UI that lists one or more background jobs, progress,
timestamps, warnings/errors, retry/cancel controls keyed by job id, or multiple
parallel source tasks. Detailed job cards belong only in Activity for this
slice.

The first slice should reuse existing job/status data. It should not introduce
new background job APIs only for this tab.

## Architecture

### Components

Add a shared live-source shell:

- `SourceBrowserShell.svelte`
  - owns tab derivation and local active tab state;
  - receives route-level data and callbacks through explicit props;
  - renders provider-aware views and the universal `Items` tab;
  - applies the smart default when the current tab is unavailable for the
    selected source.

Keep provider-aware views small:

- `YoutubeTranscriptView` or existing `YoutubeTranscriptReader` wrapper;
- `YoutubeCommentsView`;
- `TelegramTimelineView` or existing `TelegramTimelineReader` wrapper;
- `UniversalItemsView`;
- `SourceMetadataView`;
- `SourceActivityView`.

Avoid a broad frontend service layer. Existing route state and API modules
should feed the shell until a later slice proves a clearer boundary is needed.
The shell must not introduce a new global store for source browsing state.

The route or existing route-owned parent surface remains responsible for data
fetching and passes a prop bundle equivalent to:

- selected `source`, source metric, selected trace ref, and timestamp formatter;
- live `sourceItems`, `sourceItemsHasMore`, `loadingItems`, and
  `onLoadMoreSourceItems`;
- Telegram topic/history-scope state and callbacks;
- YouTube transcript segments, transcript loading state, search state, and
  transcript callbacks;
- YouTube metadata/detail DTOs and source jobs;
- Takeout recovery state, source status, disabled reasons, and source-level
  action callbacks.

The shell may own ephemeral UI state for tabs, loaded-row filters/search/sort,
and threaded/flat comments mode. Data-fetching API calls stay route-owned for
this slice; the shell and leaf views trigger loading through props callbacks
instead of importing Tauri API wrappers directly.

### Frontend Model

Extend `source-reader-model.ts` or an adjacent source browser model module with:

- provider item kind labels;
- item kind filter derivation;
- generic card display model;
- YouTube comment enrichment helpers;
- YouTube comments coverage state derivation;
- threaded comment grouping from `commentId` and `parentCommentId`;
- flat/threaded filter/sort/search helpers.

### Backend/API

Keep `list_source_items` as the generic item-row source. Extend it only where
generic browsing needs common fields.

Add or extend YouTube-specific DTOs where the UI needs provider detail:

- extend the generic source item DTO with an optional `youtubeComment` object
  derived from the stored comment raw payload:

  ```ts
  youtubeComment?: {
    commentId: string | null;
    parentCommentId: string | null;
    isReply: boolean;
    likeCount: number | null;
    isPinned: boolean;
    isHearted: boolean;
    authorChannelUrl: string | null;
  };
  ```

- extend `get_youtube_video_detail` with safe metadata fields and optional raw
  source-level metadata JSON for the `Metadata` tab.

The Tauri command DTO for `list_source_items` keeps the existing snake_case raw
API convention: backend returns `youtube_comment.comment_id`,
`parent_comment_id`, `like_count`, `is_pinned`, `is_hearted`, and
`author_channel_url`; `src/lib/api/sources.ts` maps that payload to camelCase
`SourceItem.youtubeComment`.

`Comments` and `Items` must use the same `list_source_items` pagination model.
Do not introduce a separate YouTube-comments pagination endpoint in this slice.
Unknown or malformed YouTube comment raw payloads should leave `youtubeComment`
undefined while preserving the base item row.

Do not move transcript segments into generic item pagination. The dedicated
transcript segment API stays because it has timestamp-specific pagination and
search behavior.

## Data Flow

1. The route selects a live single source.
2. The route-owned parent passes source data, current loaded windows, status,
   and callbacks to `SourceBrowserShell` through props.
3. `SourceBrowserShell` derives available tabs from the source, opens the smart
   default on first entry, and preserves the active tab across `source.id`
   changes only when the new source exposes that canonical tab id.
4. Provider-aware tabs receive existing route state where possible.
5. `Transcript` loads segments through the existing route/API path for
   `list_youtube_transcript_segments`.
6. `Comments` derives comments coverage from existing route/source/job/detail
   state and item rows.
7. `Comments` and `Items` use generic source item rows plus frontend filtering
   over the currently loaded page; YouTube comment data comes from the optional
   `youtubeComment` enrichment on those rows. They do not request full-source
   search/filter/sort semantics in this slice.
8. `Metadata` combines `currentSource`, provider detail DTOs, and loaded topic
   state when relevant.
9. `Activity` reads already available source jobs, Takeout recovery state, and
   source status, and exposes the same route callbacks used by contextual CTAs.

## Error And Empty States

- Unknown comments coverage: show a muted state and avoid claiming comments are
  absent or unsynced.
- Comments not synced: show an empty state and `Sync comments`.
- Comments syncing: show a compact pending state in Comments and detailed job
  state in Activity.
- Failed comment sync: show failed job state in Activity, keep the Comments tab
  readable for any previously synced rows, and expose a compact retry CTA.
- Synced empty comments: show an empty state that says no comments are available
  locally after sync.
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
  types by canonical tab id, not display label;
- active tab opens on smart default for first entry, preserves the current tab
  across `source.id` changes when the new source exposes that tab, falls back
  to smart default when it does not, and preserves the tab on same-source data
  refresh;
- route contract showing Telegram live sources and YouTube video live sources
  use the new shell while YouTube playlist live sources, source groups, and
  snapshots stay on existing readers;
- route contract showing `SourceBrowserShell` receives route-owned data and
  callbacks through props, owns active tab state locally, and does not require
  `activeSourceBrowserTab` in `/analysis/+page.svelte` or a new global
  source-browser store;
- universal item kind filter derivation from loaded rows, scoped loaded-item
  search labels, and generic item card fallback;
- source item pagination uses the documented 120-row loaded window for `Items`
  and `Comments`;
- YouTube comments threaded/flat grouping, loaded-comment search, local sort,
  filters, and orphan reply rendering;
- YouTube comments coverage state derivation for `unknown`, `not_synced`,
  `syncing`, `failed`, `synced_empty`, and `synced_with_rows`;
- YouTube comments and Items consume the same `SourceItem.youtubeComment`
  enrichment without a separate comments pagination model;
- YouTube threaded comments grouping stays linear over loaded rows and does not
  issue missing-parent fetches;
- metadata view renders Summary, Source state, Technical, and collapsed Raw JSON
  sections, with raw JSON limited to source-level YouTube metadata;
- raw JSON rendering stays collapsed by default, uses bounded preview rendering,
  exposes copy for the full payload when available, and does not block initial
  Metadata rendering;
- Activity tab receives existing source jobs/status and source-level actions
  without provider tabs duplicating detailed job cards.
- contextual CTAs remain compact local controls while detailed job cards render
  only in Activity.

Rust/backend tests:

- `get_youtube_video_detail` exposes the selected metadata fields and raw JSON
  without regressing existing summary/membership behavior;
- `list_source_items` exposes optional `youtubeComment` enrichment from stored
  raw payload for YouTube comment item rows;
- unknown or malformed raw comment payloads degrade without command failure when
  the base item row remains valid.
- item raw-data payloads are not exposed by the generic item browser contract.

Manual verification:

- open a Telegram source and confirm `Timeline | Items | Metadata | Activity`;
- open a YouTube video with synced transcript/comments and confirm
  `Transcript | Comments | Items | Metadata | Activity`;
- confirm Comments threaded and flat modes;
- confirm Comments distinguishes unknown, not-synced, syncing, failed,
  synced-empty, and synced-with-rows coverage states;
- confirm search/filter/sort labels and empty states communicate loaded-row
  scope;
- confirm orphan replies render visibly if a loaded page contains a reply whose
  parent is not loaded;
- confirm Metadata is grouped into Summary, Source state, Technical, and Raw
  JSON sections;
- confirm Raw JSON is limited to source-level YouTube metadata, not arbitrary
  item rows;
- confirm large Raw JSON payloads stay collapsed, bounded, copyable, and do not
  inflate the initial Metadata tab;
- confirm Activity shows consolidated status and actions while provider tabs
  keep only contextual CTAs;
- confirm contextual CTAs do not render detailed job lists, progress history,
  warnings, timestamps, or per-job retry/cancel cards outside Activity;
- confirm `Sync comments` and `Sync metadata` actions are reachable from empty
  or stale states;
- confirm YouTube playlist live sources still use the existing playlist reader
  in this slice;
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
