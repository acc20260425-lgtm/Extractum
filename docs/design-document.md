# Design Document

## 1. Goal

Extractum is a desktop-first research tool for collecting source history into a
local archive and running structured analysis over that archive. Telegram and
YouTube are implemented ingest providers today, while the shared source and
analysis layers are ready for future RSS/forum providers.

The design goals are:

- local-first storage and analysis inputs;
- predictable saved analysis artifacts;
- thin infrastructure backend with explicit ownership;
- UI flows that stay simple for a solo or small-team workflow.

## 2. Guiding decisions

### 2.1 Local-first storage

All source data and analysis state are stored locally in SQLite. The application does not depend on a remote Extractum backend.

### 2.2 Thin backend, rich frontend

Rust owns:

- Telegram client lifecycle
- YouTube `yt-dlp` orchestration
- session restore
- migrations
- SQLite access
- OS secure storage access
- compression
- report orchestration

Svelte owns:

- routes
- local UI state
- forms and filters
- workflow composition

The current frontend workflow is result-first: `/analysis` keeps setup,
opened reports, or source material in the central canvas, while source
switching, evidence, follow-up chat, live chunk summaries, and saved runs stay
nearby. Shared canvas-level workspace tools keep NotebookLM export, template editing,
and group editing reachable in both setup and opened-run states. The legacy
`/sources` route remains only as a compatibility redirect to `/analysis`.

Live source browsing now uses `SourceBrowserShell` for Telegram sources,
YouTube videos, YouTube playlists, live source groups, and available run
snapshots. The shell owns only local tab state and receives route-owned
data/callbacks through subject-specific data objects: `sourceBrowserData`,
`groupBrowserData`, and `snapshotBrowserData`. Telegram defaults to `Timeline`;
YouTube videos default to `Transcript`; YouTube playlists default to `Videos`;
live source groups default to `Sources`; available run snapshots default to
their provider-aware snapshot tab. All live sources expose loaded-window
`Items`, structured `Metadata`, and consolidated `Activity`; available run
snapshots preserve frozen snapshot semantics and do not expose live source
actions.

The `/analysis` workspace mode state is centralized in a small typed
state-machine module. `AnalysisWorkspaceEvent` values describe route-level user
and system actions, `transitionAnalysisWorkspaceState` computes the next pure
UI state, and the Svelte route applies those events through a single dispatcher.
Route effects and workflow calls stay outside that transition layer, which keeps
invalid combinations such as run-bound companion tabs without an opened run from
spreading across component-local booleans.

Svelte 5 `$effect` blocks should keep their dependency surface narrow. An
effect tracks synchronous `$state` and `$derived` reads, including reads inside
functions it calls. Route effects that call workflow functions must be reviewed
carefully when those workflows synchronously call `deps.getState()` and later
patch route state. Prefer explicit parameter APIs for effect-triggered workflow
calls; use `untrack` for incidental reads, or prefer explicit event handlers /
lifecycle flows for one-shot data loads.

### 2.3 Text-first analysis, media-aware ingest

The sync layer already preserves lightweight media metadata, but the analysis layer still uses a text-only corpus. This intentionally separates:

- archive completeness;
- analysis complexity.

That lets the product preserve media-bearing and media-only posts today without forcing immediate multimodal analysis support.

## 3. Current ingest design

### 3.1 Source model

Each source is stored in `sources` with:

- `source_type`
- `source_subtype`
- `external_id`
- `title`
- shared sync state
- account linkage

Source identity is scoped by account, provider, kind, and external id. This
matters because the same Telegram channel or group can exist in more than one
local account, and Telegram bare ids can overlap across source kinds.
YouTube identity is scoped by provider subtype and external id, so videos and
playlists dedupe separately.

Provider-specific operational identity is typed. Telegram peer identity,
resolution hints, and display cache fields live in `telegram_sources`.
Telegram message identity and reply/topic/reaction context live in
`telegram_messages`. YouTube video and playlist runtime metadata live in
`youtube_video_sources` and `youtube_playlist_sources`; `sources.metadata_zstd`
is not the owner for normal YouTube runtime reads.

Source UI actions are capability-driven. Sync, Takeout import, membership
state, and topic controls are shown only for source families that support them.

### 3.2 Sync model

The first sync window is configurable through app settings:

- `recent_messages(N)`
- `recent_days(N)`

After the first sync, the app resumes from `last_sync_state`.

`sync_source` remains the ordinary incremental ingest path. It dispatches by
provider and routes Telegram sources into the Telegram sync flow. YouTube sync
uses provider-specific source jobs for metadata, transcripts, comments, and
playlist expansion. Unsupported provider sync attempts return typed validation
errors.

### 3.3 Takeout import model

Takeout import is the full-history ingest path for an existing source. It
intentionally writes into the same canonical provider/archive tables as sync.
Browsing and Telegram NotebookLM export can use the provider-neutral
`archive_read_items` model when source readiness is current; analysis reads the
separate `analysis_documents` corpus model.

Important design choices:

- Takeout import targets existing sources only; it does not create sources.
- It uses in-memory job state with `sources://takeout-import` progress events.
- It shares the same-source ingest lock with sync and delete.
- It does not download media files or Telegram Desktop export assets.
- It updates `last_sync_state` only after a successful Takeout finish.
- Failed and cancelled imports can leave partial rows, and repeat runs rely on duplicate skipping.
- It persists durable ingest batches, Telegram Takeout batch details, warning
  codes, and item observations after the same-source lock is acquired.
- Normal supergroup Takeout keeps migrated small-group history as a separate
  historical scope; explicit migrated-history import and downstream
  browsing/export/analysis opt-ins are separate user choices.

The history pagination is TDesktop-first. The app models the full state machine with `largest_id_plus_one`, page-order normalization, and cursor advancement, then falls back per split to the older descending cursor profile only when the TDesktop profile is visibly unsafe for that split.

### 3.4 YouTube source model

YouTube support is text/metadata-first. Extractum shells out to `yt-dlp` for
preview, source creation, metadata refresh, captions, comments, and playlist
entry metadata. The app does not download YouTube audio or video binaries.

YouTube videos and playlists are registered as `sources` rows:

- videos use `source_type = youtube`, `source_subtype = video`;
- playlists use `source_type = youtube`, `source_subtype = playlist`;
- typed runtime metadata is stored in provider-specific source tables;
- playlist membership lives in `youtube_playlist_items`;
- transcript timing lives in `youtube_transcript_segments`.

The analysis workspace can run YouTube reports over synced transcript text,
optional synthetic description text, and optional comments. Playlist analysis
expands linked playlist video sources and excludes unlinked/unavailable rows.
Live YouTube video browsing exposes transcript segments through the existing
transcript reader, comments through generic source item rows enriched with
optional YouTube comment fields, source-level metadata through
`get_youtube_video_detail`, and background work through the shared Activity tab.

YouTube source jobs are in-memory runtime state. They are not resumed after app
restart; completed SQLite writes remain visible and the user can start a fresh
sync. Auth-gated YouTube content can use cookies configured in Settings. Those
cookies are stored in OS secure storage and are written only to temporary
backend files for the lifetime of a `yt-dlp` process.

### 3.5 Item persistence

Each synced item can be:

- `text_only`
- `text_with_media`
- `media_only`

The backend stores:

- compressed text when text exists;
- compressed raw JSON payload;
- lightweight media metadata when media exists.
- provider item kind (`telegram_message`, `youtube_transcript`, or `youtube_comment`);
- nullable Telegram context metadata for newly synced rows:
  - reply target message id;
  - reply target Telegram peer kind/id;
  - thread/topic root message id;
  - aggregate reaction count.

This keeps the storage model useful for browsing and future expansion without committing to binary media ingestion yet.

Older rows are not backfilled. A `NULL` Telegram context value means the metadata is unavailable, predates the migration, or was not exposed by Telegram.

Telegram duplicate detection uses typed native identity in
`telegram_messages`, not the generic `(source_id, external_id)` key. The
generic `items.external_id` value remains populated for compatibility, display,
and old ref handling.

YouTube transcript text is stored as a `youtube_transcript` item. Timestamped
segments are stored separately in `youtube_transcript_segments` so trace refs
can resolve to YouTube timestamp links.
YouTube comments are stored as `youtube_comment` item rows. The generic source
item listing can enrich those rows with comment ids, parent ids, like counts,
pinned/hearted flags, and author channel URLs when the stored raw payload is
valid; malformed raw payloads leave the base item row readable.

## 4. Current analysis design

### 4.1 Scope

Reports can be generated for:

- one source;
- one saved source group.

YouTube scopes can choose one of three corpus modes:

- transcript only;
- transcript plus synthetic description;
- transcript plus synthetic description plus comments.

The report run stores:

- scope metadata
- provider/model/template metadata
- result markdown
- trace data
- completion status

While a run is streaming, chunk summaries are shown in the analysis companion as
live UI state. They are intentionally not persisted with the saved run.

### 4.2 Immutable saved runs

Saved runs now persist a frozen corpus snapshot in `analysis_run_messages`.

This is important because it avoids drift caused by:

- later syncs that backfill older items;
- edits to source group membership;
- follow-up chat re-reading the live `items` table.

The intended behavior is:

- a saved run remains meaningfully reproducible;
- follow-up chat for completed runs uses the frozen snapshot context;
- trace resolution for completed runs resolves against the frozen snapshot.

New live corpus refs use local item identity (`s{source_id}-i{item_id}`), while
legacy Telegram-shaped refs (`s{source_id}-m{message_id}`) remain readable.
Completed runs without snapshot data stay readable as reports, but source
resolution, evidence, and follow-up chat degrade explicitly instead of silently
falling back to live `items`.
YouTube transcript refs preserve timestamp evidence and can resolve to
canonical YouTube URLs with `t=` parameters.

### 4.3 LLM provider profiles

The LLM layer now resolves requests through saved provider profiles rather than a single hard-coded provider configuration.

Each profile currently stores:

- `profile_id`
- provider kind
- default model
- provider-specific `base_url` when relevant
- an `api_key_configured` flag for frontend display

The current runtime contract is:

- one profile is marked active and used by default when a workflow does not pass an explicit profile id;
- saved API keys are resolved only in the Rust backend from OS secure storage;
- Gemini and OpenAI-compatible providers share the same backend request-resolution path;
- OpenAI-compatible model listing and live requests both use the saved or currently edited `base_url`;
- analysis runs persist `provider_profile`, `provider`, and `model` metadata so later review can see which profile produced the result.

LLM scheduling allows two running requests per `(provider, profile)` and prioritizes interactive requests over background work. Analysis report runs run a backend preflight before run creation and are capped at `10_000` messages, `80` estimated chunks, `1_500_000` estimated input characters, and `80` background requests.

`/settings` is intentionally profile-oriented:

- existing profiles can be selected and edited;
- new profiles can be saved without activation;
- the same save flow can also activate a profile immediately;
- the provider smoke test saves the visible form first and then runs through that saved profile state.

### 4.4 Why not full multimodal analysis yet

The current design stops at media-aware metadata because the next complexity jump would require:

- new prompt contracts;
- new retrieval / citation semantics for non-text items;
- more UI affordances for evidence display.

That work is deliberately postponed.

### 4.5 NotebookLM export context

NotebookLM export remains local-only. It does not make live Telegram requests,
LLM calls, link fetches, or media downloads. For Telegram sources with a
current ready archive model, export reads message rows from
`archive_read_items`; non-ready states preserve the local provider/archive
items fallback.

When reply metadata is present, the export layer resolves original reply messages from local SQLite in the same source by `(source_id, external_id)`. Original messages may be outside the selected export period, but they are used only as snippet metadata and are not added to the exported corpus.

The current export uses stored nullable metadata for:

- reply target id;
- reply author and snippet when the original message is available locally;
- reply target peer kind/id;
- thread id;
- aggregate reaction count.

Telegram forum topic names and filters use materialized
`item_topic_memberships` plus source-level `telegram_topic_resolution_state`.
`Unrecognized topic` remains a derived bucket for ready/current resolution
state and is not stored as a topic row.

## 5. Error design

The backend now uses a minimal typed error model:

- `validation`
- `not_found`
- `auth`
- `network`
- `conflict`
- `internal`

This preserves a simple Tauri boundary while giving the frontend a usable semantic contract.

## 6. Storage decisions

### 6.1 Compression

Compressed blobs are used for:

- item text
- item raw payloads
- media metadata
- analysis trace data
- snapshot message content

This keeps SQLite reasonably small without introducing extra infrastructure.

### 6.2 App settings

`app_settings` currently stores:

- active LLM profile selection
- LLM provider profile metadata
- initial sync policy keys
- YouTube settings metadata

Saved LLM API keys and Telegram `api_hash` values are stored through the backend
`secret_store` module in the OS credential store. Telegram session files remain
app-data files, but their contents are encrypted with per-account session keys
stored in OS secure storage under `telegram.account.<account_id>.session_key`.
Legacy plaintext values in `app_settings`, `accounts.api_hash`, or Telegram
session files are migrated lazily: the backend writes the secure-store secret
first and only then clears or replaces the legacy value. If secure storage
fails, the operation fails closed and leaves legacy plaintext untouched.
YouTube cookies use the same secure-store boundary and are never returned to the
frontend through IPC.

## 7. Open design work

The most meaningful remaining design questions are:

- whether RSS or forum ingestion should be implemented next;
- whether private Telegram peer resolution should gain stronger cached identity data;
- whether natural shifted export DC fallback appears in future live Takeout
  evidence beyond the current code-backed validation;
- whether migrated-history merged export or purge/unimport behavior is needed
  beyond the current explicit import and opt-in browsing/export/analysis flows;
- whether YouTube jobs should become persistent/resumable across app restart;
- whether YouTube-specific NotebookLM export enrichment should be shipped;
- how to expand analysis beyond text-bearing corpus items;
- whether Telegram Forum Topics needs richer browsing/export controls beyond
  materialized topic memberships and the current source item filters;
- how and when to persist forward metadata;
- whether Telegram session storage should remain JSON-based long term.
