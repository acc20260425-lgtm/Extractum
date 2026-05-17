# YouTube Typed Source Metadata Design

Date: 2026-05-17

## Summary

YouTube source metadata should move out of generic `sources.metadata_zstd`
blobs and into typed YouTube source tables. After this slice, YouTube typed
source tables own both hot typed metadata and an optional versioned raw provider
payload. Typed columns are authoritative for normal runtime. `raw_metadata_zstd`
is retained only for archive, debug, reparse, and migration compatibility, and
must not be decoded in normal source listing, detail, jobs, or analysis paths.

`sources.metadata_zstd` is not the owner of YouTube runtime metadata after this
slice.

## Problem

YouTube video and playlist source upserts currently serialize
`YoutubeVideoMetadata` and `YoutubePlaylistMetadata` into
`sources.metadata_zstd`. Several normal runtime paths then decode the generic
source blob:

- `src-tauri/src/youtube/detail.rs` decodes source blobs for summaries and
  detail DTOs.
- `src-tauri/src/youtube/jobs.rs` decodes source blobs before transcript,
  comment, and playlist provider work.
- `src-tauri/src/analysis/corpus.rs` decodes video source blobs for synthetic
  description rows and source-level evidence context.

That keeps provider-specific metadata in the generic source table, makes
read-only paths depend on compressed JSON decode, and leaves future provider
work with unclear ownership boundaries.

## Goals

- Add typed YouTube source metadata tables for video and playlist sources.
- Make typed columns the normal runtime source for YouTube listing, detail,
  jobs, and analysis metadata needs.
- Keep optional versioned raw provider payloads in the typed tables for archive,
  debug, reparse, and migration compatibility only.
- Stop creating or replacing YouTube `sources.metadata_zstd` blobs in normal
  upsert paths.
- Backfill existing valid YouTube source blobs into typed rows during a managed
  migration.
- Clear `sources.metadata_zstd` after successful typed backfill or successful
  typed source upsert. Clearing is part of the same transaction as the typed
  metadata write and must not happen if the typed row write fails.
- Keep invalid or unbackfillable legacy blobs inert: they may remain for
  diagnosis, but normal runtime must not read them.
- Preserve YouTube playlist items, transcript segments, comments, analysis
  snapshots, and Telegram behavior unless explicitly mentioned here.
- Keep read-only paths local: listing, detail, and analysis must not implicitly
  call `yt-dlp`.

## Non-Goals

- Do not remove the physical `sources.metadata_zstd` column.
- Do not move `youtube_playlist_items.metadata_zstd`.
- Do not move `youtube_transcript_segments.metadata_zstd`.
- Do not redesign playlist entity/removal state or nullable playlist links.
- Do not change item/document identity.
- Do not change frontend UX beyond backend DTOs exposing an existing or
  controlled missing-metadata/degraded state.
- Do not change Telegram source metadata behavior from the completed Telegram
  metadata legacy cleanup.

## Data Ownership

`sources` owns only provider-neutral source state for YouTube:

- `source_type`
- `source_subtype`
- `account_id`
- `external_id`
- `title`
- active flags
- sync watermarks
- timestamps

`youtube_video_sources` owns YouTube video runtime metadata:

- `source_id INTEGER PRIMARY KEY REFERENCES sources(id) ON DELETE CASCADE`
- `video_id TEXT NOT NULL`
- `canonical_url TEXT NOT NULL`
- `title TEXT`
- `channel_title TEXT`
- `channel_id TEXT`
- `channel_handle TEXT`
- `channel_url TEXT`
- `author_display TEXT`
- `published_at TEXT`
- `duration_seconds INTEGER`
- `description TEXT`
- `thumbnail_url TEXT`
- `tags_json TEXT NOT NULL DEFAULT '[]'`
- `chapters_json TEXT NOT NULL DEFAULT '[]'`
- `view_count INTEGER`
- `like_count INTEGER`
- `comment_count INTEGER`
- `category TEXT`
- `video_form TEXT NOT NULL`
- `availability_status TEXT NOT NULL`
- `caption_language_override TEXT`
- `raw_metadata_version INTEGER`
- `raw_metadata_zstd BLOB`
- `created_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))`
- `updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))`

`youtube_playlist_sources` owns YouTube playlist runtime metadata:

- `source_id INTEGER PRIMARY KEY REFERENCES sources(id) ON DELETE CASCADE`
- `playlist_id TEXT NOT NULL`
- `canonical_url TEXT NOT NULL`
- `title TEXT`
- `channel_title TEXT`
- `channel_id TEXT`
- `channel_handle TEXT`
- `channel_url TEXT`
- `thumbnail_url TEXT`
- `video_count INTEGER`
- `availability_status TEXT NOT NULL`
- `raw_metadata_version INTEGER`
- `raw_metadata_zstd BLOB`
- `created_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))`
- `updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))`

Typed rows must match their parent source row:

- video rows require `sources.source_type = 'youtube'`,
  `sources.source_subtype = 'video'`, and
  `youtube_video_sources.video_id = sources.external_id`;
- playlist rows require `sources.source_type = 'youtube'`,
  `sources.source_subtype = 'playlist'`, and
  `youtube_playlist_sources.playlist_id = sources.external_id`.

SQLite cannot enforce cross-table subtype checks directly with ordinary CHECK
constraints, so Rust upsert/backfill code must validate these invariants and
tests must cover mismatch rejection.

### Value Domains

`video_form` uses the existing `YoutubeVideoForm` snake_case wire values:

- `regular`
- `short`
- `live`

`availability_status` uses the existing `YoutubeAvailabilityStatus` snake_case
wire values:

- `available`
- `upcoming`
- `live_now`
- `live_ended_transcript_pending`
- `no_captions`
- `private_or_auth_required`
- `members_only`
- `age_restricted`
- `geo_blocked`
- `deleted`
- `removed_from_playlist`
- `unavailable_unknown`

Video and playlist source rows share this vocabulary with
`youtube_playlist_items.availability_status` for compatibility, but the meaning
is scoped to the owning table. In typed source tables, it describes the source's
current availability. In `youtube_playlist_items`, it describes the playlist
entry's availability and lifecycle. Direct video/playlist metadata refresh
should not produce source-level `removed_from_playlist`; that value remains
valid for playlist entry lifecycle and legacy compatibility.

`tags_json` stores a JSON array of strings. `chapters_json` stores a JSON array
of chapter objects with `index`, `title`, `start_ms`, and optional `end_ms`.
Empty arrays are stored as `[]`. Invalid JSON or non-array values make the typed
row invalid for normal runtime until explicit metadata refresh or managed
migration/backfill code rewrites the row.

### Valid Typed Metadata

A YouTube typed metadata row is valid when all of these are true:

- the matching parent `sources` row exists;
- the parent source has `source_type = 'youtube'`;
- a video typed row has parent `source_subtype = 'video'`;
- a playlist typed row has parent `source_subtype = 'playlist'`;
- `youtube_video_sources.video_id` or `youtube_playlist_sources.playlist_id`
  equals `sources.external_id`;
- `canonical_url` is non-empty after trimming and parses as a supported YouTube
  URL for the row subtype;
- video rows have `video_form` set to one of the supported `YoutubeVideoForm`
  wire values;
- `availability_status` is one of the supported `YoutubeAvailabilityStatus`
  wire values;
- `tags_json` and `chapters_json` parse as arrays;
- `created_at` and `updated_at` are present and non-negative when supplied by
  migration or upsert code.

Rows that fail these checks are treated the same as missing typed metadata:
job-owned provider commands may refresh them explicitly, while read-only
listing, detail, and analysis return controlled missing-metadata or degraded
state and do not fall back to source blobs.

## Raw Provider Payload Policy

The typed tables may store an optional compressed raw provider payload:

- `raw_metadata_version` identifies the payload shape.
- `raw_metadata_zstd` stores compressed JSON from the provider payload, not
  cookies, request headers, command arguments, auth diagnostics, or logs.
- Normal listing, detail, jobs, and analysis paths must not decode
  `raw_metadata_zstd`.
- Runtime provider work should use typed columns. Any provider-work field needed
  after source creation must be promoted to a typed column or treated as absent.
- Reparse/debug tooling may decode `raw_metadata_zstd`, but that must be an
  explicit compatibility/debug path, not a normal read path.

`caption_language_override` is a typed provider hint copied from the video
metadata payload when present. It is not a user setting, not the selected
transcript language, and not segment evidence. Jobs may pass it to caption
selection as a provider-supplied preference for that video. Persisted transcript
language and track evidence remains in `youtube_transcript_segments`.

## Migration And Backfill

This needs a managed Rust migration because SQLite SQL cannot decode zstd JSON.
The migration should follow the existing runner-managed migration pattern used
for source identity cleanup, or an equivalent pre-plugin Rust-managed migration
step.

The migration has two responsibilities:

1. Create `youtube_video_sources` and `youtube_playlist_sources`.
2. Backfill typed rows from existing valid YouTube `sources.metadata_zstd`
   blobs.

Backfill rules:

- only rows with `source_type = 'youtube'` and `source_subtype IN
  ('video', 'playlist')` are candidates;
- a valid video blob must decode as `YoutubeVideoMetadata` and have
  `metadata.video_id = sources.external_id`;
- a valid playlist blob must decode as `YoutubePlaylistMetadata` and have
  `metadata.playlist_id = sources.external_id`;
- valid rows are inserted into the matching typed table;
- after a successful typed row insert, the migration clears
  `sources.metadata_zstd` for that source;
- missing, corrupt, wrong-shape, or mismatched blobs do not create typed rows
  and may remain in `sources.metadata_zstd` as inert diagnostic artifacts;
- migration diagnostics should be testable and should not expose raw payload
  contents in errors.

Fresh installs should start with the typed YouTube tables present. Existing
databases should upgrade safely without requiring provider network access.

## Runtime Data Flow

### Write Path

`upsert_youtube_video_source` and `upsert_youtube_playlist_source` become atomic
upserts across `sources` and the matching typed metadata table.

`sources` owns generic identity/display snapshot and sync state:
`source_type`, `source_subtype`, `external_id`, `title`, active flags, and sync
watermarks.

`youtube_video_sources` and `youtube_playlist_sources` own YouTube runtime
metadata and optional versioned raw provider payload.

New YouTube source inserts bind `sources.metadata_zstd = NULL`. Conflict
updates must not create or replace `sources.metadata_zstd`; once typed metadata
has been written successfully, conflict updates clear `sources.metadata_zstd`
to `NULL`.

The source row and typed metadata row must be written in one transaction. A
typed metadata failure rolls back the source row update.

### Read Path

YouTube listing, detail, job, and analysis runtime reads use typed columns.

`youtube/detail.rs` joins typed tables for summaries and details. It does not
decode source blobs or raw provider payloads.

`youtube/jobs.rs` loads typed metadata for canonical URLs and provider work. If
typed metadata is missing or invalid, commands that own a metadata-refresh flow
may run metadata sync first, then reload typed metadata before dependent work.
Read-only commands return a controlled missing-metadata state.

`analysis/corpus.rs` reads video typed metadata for synthetic description rows
and source-level evidence context such as canonical URL, title, and channel.
Transcript timing and caption evidence remain owned by
`youtube_transcript_segments`. Analysis must not decode `sources.metadata_zstd`
or call `yt-dlp`.

Source list DTOs may keep the generic `SourceRecord` shape. YouTube
provider-specific display state comes from typed YouTube metadata/detail
aggregation, not from generic source blobs.

### Failure Behavior

| State | Runtime outcome |
| --- | --- |
| Typed row valid | Normal runtime works; source blobs and raw payloads are ignored. |
| Typed row missing or invalid, command owns metadata refresh | Command runs explicit metadata refresh, reloads typed row, then continues dependent work. |
| Typed row missing or invalid, no refresh ownership or refresh fails | Command returns a typed missing-metadata or degraded-state error. |

Valid typed metadata makes legacy `sources.metadata_zstd` irrelevant. Corrupt
source blobs do not break listing, detail, jobs, or analysis.

Corrupt `raw_metadata_zstd` does not break normal runtime because normal
runtime uses typed columns only.

Listing, detail, and analysis are local database reads and must not implicitly
call `yt-dlp`.

## Testing Strategy

Migration and backfill tests:

- valid legacy video blobs backfill into `youtube_video_sources`;
- valid legacy playlist blobs backfill into `youtube_playlist_sources`;
- successful backfill clears `sources.metadata_zstd`;
- corrupt or mismatched legacy blobs do not create typed rows and do not fail
  the whole migration;
- fresh-schema migration tests include the new typed tables;
- managed migration tests confirm the SQL plugin does not directly execute a
  Rust-only backfill step if a sentinel migration is used.

Write-path tests:

- video upsert writes a source row, a typed video row, optional raw payload, and
  leaves `sources.metadata_zstd NULL`;
- playlist upsert writes a source row, a typed playlist row, optional raw
  payload, playlist item rows, and leaves `sources.metadata_zstd NULL`;
- conflict upserts update typed rows and clear any existing legacy
  `sources.metadata_zstd`;
- typed row validation failure rolls back the source row update.

Read-path tests:

- detail summaries/details work with `sources.metadata_zstd` missing or corrupt
  when typed rows are valid;
- analysis description and transcript evidence context work from typed columns
  and do not decode source blobs;
- `detail_missing_typed_metadata_does_not_run_provider`: detail/read-only paths
  return controlled missing metadata and do not invoke provider work;
- `analysis_missing_typed_metadata_fails_or_skips_with_typed_error`: analysis
  does not decode blobs and does not trigger provider refresh;
- `jobs_refresh_missing_typed_metadata_then_reloads_before_dependent_work`:
  job-owned refresh reloads typed metadata before transcript/comment work;
- `raw_metadata_zstd_corrupt_detail_analysis_still_use_typed_columns`: corrupt
  raw payload does not break detail or analysis when typed columns are valid.

Containment scans:

- normal YouTube listing/detail/analysis paths do not call
  `decode_youtube_metadata` on `sources.metadata_zstd`;
- normal listing/detail/jobs/analysis paths do not decode `raw_metadata_zstd`;
- YouTube source upserts no longer bind compressed metadata into
  `sources.metadata_zstd`;
- Telegram legacy metadata compatibility remains scoped to the Telegram repair
  and compatibility paths from the previous cleanup.

## Acceptance Criteria

- Fresh YouTube source inserts and refresh upserts create or update typed
  metadata rows.
- Successful YouTube typed writes keep `sources.metadata_zstd NULL` or clear it
  to `NULL`.
- Existing valid YouTube source blobs are backfilled into typed rows during
  managed migration.
- Missing, corrupt, wrong-shape, or mismatched legacy YouTube source blobs do
  not fail the whole migration, do not create typed rows, remain inert
  diagnostic artifacts, and lead to controlled missing-metadata behavior in
  normal runtime.
- Normal YouTube listing, detail, jobs, and analysis runtime reads typed
  columns and do not decode `sources.metadata_zstd`.
- `raw_metadata_zstd` is optional, versioned, secret-safe, and not decoded by
  normal source listing, detail, jobs, or analysis.
- Corrupt source blobs and corrupt raw payloads do not break normal runtime
  when typed columns are valid.
- Missing or invalid typed metadata produces controlled missing-metadata
  behavior or explicit job-owned refresh, never an implicit provider call from
  read-only paths.
- Telegram behavior and the completed Telegram metadata boundary remain
  unchanged.
