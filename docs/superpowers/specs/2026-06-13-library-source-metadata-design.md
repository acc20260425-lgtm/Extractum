# Library Source Metadata Design

Date: 2026-06-13

## Goal

Design the next Library slice after the `/projects/library` prototype: enrich the
Library source table, subtype filters, and Inspector with source metadata that is
already stored in the existing database.

This slice makes Library feel like an actual source catalog instead of a thin
view over `AnalysisSourceOption`.

## Confirmed Brief

- Keep `/projects/library` as the Library route inside the current projects
  shell.
- Add a dedicated backend read API named `list_library_sources`.
- Do not create a durable `library_sources` table.
- Do not implement Add/Edit/Delete mutations in this slice.
- Include YouTube subtype/details in the first version:
  - `video`;
  - `playlist`.
- Include Telegram subtype/details in the first version:
  - `channel`;
  - `supergroup`;
  - `group`.
- Keep Telegram details shallow for this slice. `account_id` and subtype are
  enough.
- Keep project relationship data minimal: expose `project_count`, but not the
  full list of linked projects yet.
- Keep status simple in the UI:
  - `active`;
  - `syncing`;
  - `error`;
  - `unavailable`.
- Do not add a `stale` status yet. Freshness can be a later slice.

## Chosen Approach

Use a dedicated Library read model over the existing source tables.

`list_analysis_sources` stays as the compact analysis/connect API. It should not
grow Library-specific fields such as `source_subtype`, provider canonical URLs,
typed YouTube metadata, or Library Inspector details.

`list_library_sources` becomes the first-class read API for the Library screen.
It composes data from existing tables and returns a stable UI-facing record for
catalog display.

This is a small backend change, but it avoids overloading the analysis API and
keeps the Library model explicit.

## Non-Goals

- Do not rename `/projects` to `/sources`.
- Do not replace the current `/analysis` UI.
- Do not add a new durable Library schema.
- Do not implement source creation, editing, deletion, or refresh jobs.
- Do not expose per-project link details in the first response.
- Do not implement YouTube channel ingestion.
- Do not decode provider raw metadata blobs for Library listing.
- Do not add a freshness or stale-source policy.

## Backend API

### Command

Add a Tauri command:

```rust
list_library_sources() -> AppResult<Vec<LibrarySourceRecord>>
```

The command should require source identity repair readiness, matching the safety
boundary used by `list_analysis_sources`.

Recommended module shape:

- `src-tauri/src/library_sources/mod.rs`;
- `src-tauri/src/library_sources/models.rs`;
- command registration in `src-tauri/src/lib.rs`.

The exact module name can change during implementation if an existing local
pattern is clearer, but the command name should stay `list_library_sources`.

### Record Contract

Use snake_case fields across the Tauri boundary, consistent with current Rust
models.

```ts
export type LibrarySourceProvider =
  | "telegram"
  | "youtube"
  | "rss"
  | "forum"
  | "web"
  | "other";

export type LibrarySourceSubtype =
  | "video"
  | "playlist"
  | "channel"
  | "supergroup"
  | "group"
  | "feed"
  | "thread"
  | "board"
  | "site"
  | null;

export interface LibrarySourceRecord {
  source_id: number;
  provider: LibrarySourceProvider;
  source_subtype: LibrarySourceSubtype;
  account_id: number | null;
  external_id: string | null;
  title: string | null;
  subtitle: string | null;
  canonical_url: string | null;
  created_at: number;
  last_synced_at: number | null;
  item_count: number;
  project_count: number;
  youtube: LibraryYoutubeSourceDetails | null;
  telegram: LibraryTelegramSourceDetails | null;
}

export interface LibraryYoutubeSourceDetails {
  video_form: string | null;
  duration_seconds: number | null;
  playlist_video_count: number | null;
  channel_title: string | null;
  availability_status: string | null;
}

export interface LibraryTelegramSourceDetails {
  account_id: number | null;
}
```

Notes:

- Dates are Unix seconds as numbers, matching existing project conventions such
  as `sources.created_at`, `sources.last_synced_at`, analysis runs, prompts, and
  groups.
- `provider` maps from `sources.source_type`.
- `source_subtype` maps from `sources.source_subtype`.
- `external_id` maps from `sources.external_id`.
- `title` starts with `sources.title`, with provider metadata as a fallback
  where useful.
- `subtitle` is a display helper. Examples: YouTube channel title, playlist
  owner, or Telegram account label. It can be `null`.
- `canonical_url` should come from provider tables when available. For YouTube,
  use `youtube_video_sources.canonical_url` and
  `youtube_playlist_sources.canonical_url`.
- `project_count` counts distinct analysis source groups that include the
  source through `analysis_source_group_members`.
- `item_count` counts stored items for the source, using the same meaning as the
  current analysis source list.

### Source Tables

The base query reads from `sources`.

Required base fields:

- `sources.id`;
- `sources.account_id`;
- `sources.source_type`;
- `sources.source_subtype`;
- `sources.external_id`;
- `sources.title`;
- `sources.created_at`;
- `sources.last_synced_at`.

Derived counts:

- `item_count` from `items`;
- `project_count` from `analysis_source_group_members`.

Provider metadata:

- YouTube video metadata from `youtube_video_sources`;
- YouTube playlist metadata from `youtube_playlist_sources`;
- Telegram first slice from `sources` only.

Implementation may use existing YouTube metadata helpers if they fit the
listing shape. If direct SQL joins are clearer for the read model, that is also
acceptable. The important rule is to avoid decoding `raw_metadata_zstd` for the
Library table or Inspector.

## UI Status Semantics

Do not persist a Library status in the new read model.

The UI-level `LibrarySourceStatus` is derived in the frontend view model:

- `syncing` when an existing `SourceJobRecord` for the source is `queued` or
  `running`;
- `error` when the latest relevant `SourceJobRecord` is `failed`;
- `unavailable` when the provider or subtype is intentionally unsupported by
  the current UI action;
- `active` otherwise.

This keeps the first slice aligned with current architecture, because YouTube
source jobs are in memory and are already exposed through `list_source_jobs`.

`last_synced_at` remains a timestamp, not a status policy. Freshness labels or a
future `stale` state should be designed separately.

## Frontend API And Types

Add a Library-specific API wrapper:

- `src/lib/api/library-sources.ts`;
- `listLibrarySources()`;
- command name `list_library_sources`.

Add Library-specific types:

- `src/lib/types/library-sources.ts`, or another local name consistent with the
  project type organization.

The existing `AnalysisSourceOption` type should stay compact and continue to
serve analysis source selection and current connect flows.

## View Model

Add or update a Library-specific view-model path that accepts
`LibrarySourceRecord[]`.

The current `buildLibrarySourcesView` was created for the first prototype and
for connect-from-library behavior. It mixes catalog display with project
connection rules. The next slice should separate those concerns:

- Library catalog screen view:
  - source metadata;
  - subtype filters;
  - table row display;
  - Inspector details.
- Connect-from-library flow:
  - project compatibility;
  - already-connected state;
  - connectable/disabled decisions.

The implementation can keep shared formatting helpers, but the top-level
Library screen should not require a selected project to describe a source.

Recommended view types:

```ts
export interface LibraryCatalogSourceView {
  id: string;
  sourceId: number;
  provider: LibrarySourceProvider;
  sourceSubtype: LibrarySourceSubtype;
  title: string;
  subtitle: string | null;
  typeLabel: string;
  status: LibrarySourceStatus;
  projectCount: number;
  itemCount: number;
  itemCountLabel: string;
  addedAtLabel: string;
  lastSyncedLabel: string | null;
  canonicalUrl: string | null;
  externalId: string | null;
  youtube: LibraryYoutubeSourceDetails | null;
  telegram: LibraryTelegramSourceDetails | null;
}
```

The exact file names are implementation details, but the model boundary should
be explicit: catalog display is not the same thing as connecting a source to a
project.

## Filter Tree

The filter tree should move from disabled subtype placeholders to real subtype
filters where the backend now returns `source_subtype`.

Rows:

- `All sources`;
- `YouTube`;
- `YouTube / Videos`;
- `YouTube / Playlists`;
- `YouTube / Channels`;
- `Telegram`;
- `Telegram / Channels`;
- `Telegram / Supergroups`;
- `Telegram / Groups`.

Behavior:

- `All sources` returns all rows.
- Provider rows return all rows for that provider.
- Subtype rows return rows with the matching provider and subtype.
- Counts are calculated from visible source records.
- `YouTube / Channels` remains disabled until channel sources are actually
  supported by the backend.
- Telegram subtype rows are enabled when rows with those subtypes exist. If a
  subtype has zero records, it can remain enabled with count `0` or be visually
  muted, but it should not pretend to be unsupported.

Stable filter ids:

```ts
"all"
"provider:youtube"
"provider:youtube/subtype:video"
"provider:youtube/subtype:playlist"
"provider:youtube/subtype:channel"
"provider:telegram"
"provider:telegram/subtype:channel"
"provider:telegram/subtype:supergroup"
"provider:telegram/subtype:group"
```

The `ExtractumTreeDataGrid` wrapper remains the only SVAR tree/grid entry point
for the filter tree.

## Table Changes

Update the Library table columns to reflect source metadata:

- `Source`;
- `Type`;
- `Status`;
- `Projects`;
- `Items`;
- `Added`;
- `Last synced`.

Column meaning:

- `Source`: title plus subtitle.
- `Type`: provider plus subtype, for example `YouTube / Video`.
- `Status`: derived UI status.
- `Projects`: `project_count`.
- `Items`: `item_count`.
- `Added`: formatted `created_at`.
- `Last synced`: formatted `last_synced_at`, or an empty-state label.

The table should still use `ExtractumDataGrid`, not direct SVAR imports.

## Inspector Changes

`LibraryInspector` should show source context from the selected catalog row.

Top section:

- title;
- provider/subtype badge;
- UI status;
- source id.

Metadata section:

- canonical URL;
- external id;
- added at;
- last synced;
- item count;
- project count.

YouTube section when `youtube` details exist:

- channel title;
- video form;
- duration for video sources;
- playlist video count for playlist sources;
- provider availability status.

Every provider detail field is optional from the UI point of view. If one field
inside an otherwise present detail block is `null`, the Inspector should handle
that single field gracefully instead of hiding the whole block. Acceptable
display choices are either omitting that metadata row or showing a quiet empty
value such as `N/A`; the implementation should use one consistent pattern
within the Inspector.

Telegram section when `telegram` details exist:

- subtype;
- account id.

If there is no selected row, the existing neutral Inspector empty state remains.

`Edit` and `Delete` stay disabled without a selected source. Their actual
mutation flows remain outside this slice.

## Error Handling

If `list_library_sources` fails:

- show the existing route/workspace error status pattern;
- keep the previous loaded rows if the workflow already does this elsewhere;
- keep `Refresh` available so the user can retry.

If provider metadata is missing for a source:

- still return the base source row;
- set the provider detail block to `null`;
- keep `canonical_url` and `subtitle` nullable;
- do not fail the whole list for one incomplete provider detail row.

If one optional field inside a provider detail block is missing:

- keep the provider detail block present;
- serialize the missing field as `null`;
- do not synthesize misleading placeholder data in the backend;
- let the frontend render a consistent empty-field treatment.

If `source_subtype` is unknown:

- preserve it as `null` or map it to the broad provider row;
- show a generic type label such as `YouTube source` or `Telegram source`;
- do not drop the source from the Library.

## Accessibility

The existing Library prototype accessibility expectations still apply.

Additional requirements for this slice:

- subtype filter rows expose selected, disabled, and count states;
- table cells use concise text that fits at laptop width;
- Inspector links expose meaningful accessible names;
- status badges are not the only status signal if color is used.

## Testing And Verification

Backend tests:

- `list_library_sources` returns `source_subtype` and `created_at`;
- YouTube video records include canonical URL, video form, duration, channel
  title, and availability status where stored;
- YouTube playlist records include canonical URL, playlist video count, channel
  title, and availability status where stored;
- Telegram records include subtype and account id;
- `item_count` and `project_count` are correct;
- a source with missing provider metadata still appears.

Frontend tests:

- `listLibrarySources` invokes `list_library_sources`;
- Library catalog view maps provider/subtype labels correctly;
- subtype filter rows are active for YouTube video, YouTube playlist, and
  Telegram channel/supergroup/group;
- `YouTube / Channels` is disabled until supported;
- filtering by provider and subtype returns the expected rows;
- table column contract includes Source, Type, Status, Projects, Items, Added,
  and Last synced;
- Inspector contract includes canonical URL, external id, added at, last synced,
  item count, project count, and provider detail blocks;
- existing import-boundary tests still prevent direct shadcn/SVAR imports from
  feature screens.

Manual verification:

- open `/projects/library`;
- confirm subtype counts are visible in the filter tree;
- select `YouTube / Videos` and confirm only video sources remain;
- select `YouTube / Playlists` and confirm only playlist sources remain;
- confirm `YouTube / Channels` is visibly disabled if no backend support exists;
- select Telegram subtype filters and confirm matching rows remain;
- select rows and confirm Inspector metadata changes;
- confirm no horizontal overflow at the current tested desktop and laptop
  widths.

## Acceptance Criteria

- `/projects/library` uses `list_library_sources` for catalog metadata.
- Existing analysis/connect APIs continue to use `list_analysis_sources` unless
  a later migration explicitly changes them.
- Library table shows subtype, project count, item count, added date, and last
  synced date.
- Library Inspector shows canonical URL, external id, timestamps, counts, and
  provider detail blocks for YouTube and Telegram.
- YouTube video and playlist subtype filters work.
- Telegram channel, supergroup, and group subtype filters work.
- YouTube channel remains disabled or unavailable until channel sources are
  truly supported.
- No new durable `library_sources` table is introduced.
- No Add/Edit/Delete mutations are implemented in this slice.
