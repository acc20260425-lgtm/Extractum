# Design Document

## 1. Goal

Extractum is a desktop-first research tool for collecting source history into a
local archive and running structured analysis over that archive. Telegram is
the only implemented ingest provider today, while the shared source and
analysis layers are ready for future non-Telegram providers.

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

The current frontend workflow is workspace-first: source browsing, sync actions, reports, trace inspection, and follow-up chat are centered in `/analysis`, while `/sources` remains a lightweight compatibility route.

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
- optional `telegram_source_kind` compatibility data (`channel`,
  `supergroup`, or `group`)
- optional compressed metadata
- account linkage
- sync state

Source identity is scoped by account, provider, kind, and external id. This
matters because the same Telegram channel or group can exist in more than one
local account, and Telegram bare ids can overlap across source kinds.

Source UI actions are capability-driven. Sync, Takeout import, membership
state, and topic controls are shown only for source families that support them.

### 3.2 Sync model

The first sync window is configurable through app settings:

- `recent_messages(N)`
- `recent_days(N)`

After the first sync, the app resumes from `last_sync_state`.

`sync_source` remains the ordinary incremental ingest path. It dispatches by
provider and currently routes only Telegram sources into the implemented sync
flow. Unsupported or not-yet-implemented provider sync attempts return typed
validation errors.

### 3.3 Takeout import model

Takeout import is the full-history ingest path for an existing source. It intentionally writes into the same `items` table as sync, so browsing, NotebookLM export, and analysis can read one local archive model.

Important design choices:

- Takeout import targets existing sources only; it does not create sources.
- It uses in-memory job state with `sources://takeout-import` progress events.
- It shares the same-source ingest lock with sync and delete.
- It does not download media files or Telegram Desktop export assets.
- It updates `last_sync_state` only after a successful Takeout finish.
- Failed and cancelled imports can leave partial rows, and repeat runs rely on duplicate skipping.

The history pagination is TDesktop-first. The app models the full state machine with `largest_id_plus_one`, page-order normalization, and cursor advancement, then falls back per split to the older descending cursor profile only when the TDesktop profile is visibly unsafe for that split.

### 3.4 Item persistence

Each synced item can be:

- `text_only`
- `text_with_media`
- `media_only`

The backend stores:

- compressed text when text exists;
- compressed raw JSON payload;
- lightweight media metadata when media exists.
- nullable Telegram context metadata for newly synced rows:
  - reply target message id;
  - reply target Telegram peer kind/id;
  - thread/topic root message id;
  - aggregate reaction count.

This keeps the storage model useful for browsing and future expansion without committing to binary media ingestion yet.

Older rows are not backfilled. A `NULL` Telegram context value means the metadata is unavailable, predates the migration, or was not exposed by Telegram.

## 4. Current analysis design

### 4.1 Scope

Reports can be generated for:

- one source;
- one saved source group.

The report run stores:

- scope metadata
- provider/model/template metadata
- result markdown
- trace data
- completion status

### 4.2 Immutable saved runs

Saved runs now persist a frozen corpus snapshot in `analysis_run_messages`.

This is important because it avoids drift caused by:

- later syncs that backfill older items;
- edits to source group membership;
- follow-up chat re-reading the live `items` table.

The intended behavior is:

- a saved run remains meaningfully reproducible;
- follow-up chat for new runs reads the frozen snapshot first;
- trace resolution for new runs resolves against the frozen snapshot first.

New live corpus refs use local item identity (`s{source_id}-i{item_id}`), while
legacy Telegram-shaped refs (`s{source_id}-m{message_id}`) remain readable.
Legacy runs without snapshot data can still fall back to live items.

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

NotebookLM export remains local-only. It does not make live Telegram requests, LLM calls, link fetches, or media downloads.

When reply metadata is present, the export layer resolves original reply messages from local SQLite in the same source by `(source_id, external_id)`. Original messages may be outside the selected export period, but they are used only as snippet metadata and are not added to the exported corpus.

The current export uses stored nullable metadata for:

- reply target id;
- reply author and snippet when the original message is available locally;
- reply target peer kind/id;
- thread id;
- aggregate reaction count.

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

Saved LLM API keys and Telegram `api_hash` values are stored through the backend
`secret_store` module in the OS credential store. Telegram session files remain
app-data files, but their contents are encrypted with per-account session keys
stored in OS secure storage under `telegram.account.<account_id>.session_key`.
Legacy plaintext values in `app_settings`, `accounts.api_hash`, or Telegram
session files are migrated lazily: the backend writes the secure-store secret
first and only then clears or replaces the legacy value. If secure storage
fails, the operation fails closed and leaves legacy plaintext untouched.

## 7. What changed since earlier planning

Earlier planning documents treated the following as future work:

- media-aware sync metadata
- immutable saved run snapshots
- typed app errors
- configurable initial sync policy
- Telegram item context metadata
- NotebookLM reply/thread/reaction metadata rendering
- Takeout source import for existing sources with TDesktop-first pagination
- provider-ready source records, capability-driven source UI, provider sync
  dispatch, and provider-neutral analysis refs

Those items are now implemented and should no longer be treated as future milestones.

## 8. Open design work

The most meaningful remaining design questions are:

- which concrete non-Telegram provider should be implemented first;
- whether private Telegram peer resolution should gain stronger cached identity data;
- how to handle migrated supergroup history without corrupting `(source_id, external_id)` uniqueness;
- whether Takeout import should run the forum-topic auxiliary refresh after successful Takeout finish;
- how to expand analysis beyond text-bearing corpus items;
- whether a full Telegram Forum Topics model is needed beyond stored `reply_to_top_id`;
- how and when to persist forward metadata;
- whether Telegram session storage should remain JSON-based long term.
