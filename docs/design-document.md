# Design Document

## 1. Goal

Extractum is a desktop-first research tool for collecting Telegram source history into a local archive and running structured analysis over that archive.

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
- compression
- report orchestration

Svelte owns:

- routes
- local UI state
- forms and filters
- workflow composition

### 2.3 Text-first analysis, media-aware ingest

The sync layer already preserves lightweight media metadata, but the analysis layer still uses a text-only corpus. This intentionally separates:

- archive completeness;
- analysis complexity.

That lets the product preserve media-bearing and media-only posts today without forcing immediate multimodal analysis support.

## 3. Current ingest design

### 3.1 Source model

Each Telegram source is stored in `sources` with:

- `external_id`
- `title`
- `telegram_source_kind` (`channel`, `supergroup`, or `group`)
- optional compressed metadata
- account linkage
- sync state

Source identity is scoped by account and kind. This matters because the same Telegram channel or group can exist in more than one local account, and Telegram bare ids can overlap across source kinds.

### 3.2 Sync model

The first sync window is configurable through app settings:

- `recent_messages(N)`
- `recent_days(N)`

After the first sync, the app resumes from `last_sync_state`.

### 3.3 Item persistence

Each synced item can be:

- `text_only`
- `text_with_media`
- `media_only`

The backend stores:

- compressed text when text exists;
- compressed raw JSON payload;
- lightweight media metadata when media exists.

This keeps the storage model useful for browsing and future expansion without committing to binary media ingestion yet.

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

Legacy runs without snapshot data can still fall back to live items.

### 4.3 Why not full multimodal analysis yet

The current design stops at media-aware metadata because the next complexity jump would require:

- new prompt contracts;
- new retrieval / citation semantics for non-text items;
- more UI affordances for evidence display.

That work is deliberately postponed.

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

- LLM provider profile settings
- temporary Gemini `api_key`
- initial sync policy keys

Secret storage is still a known follow-up.

## 7. What changed since earlier planning

Earlier planning documents treated the following as future work:

- media-aware sync metadata
- immutable saved run snapshots
- typed app errors
- configurable initial sync policy

Those items are now implemented and should no longer be treated as future milestones.

## 8. Open design work

The most meaningful remaining design questions are:

- how to move secrets out of SQLite cleanly;
- whether private Telegram peer resolution should gain stronger cached identity data;
- how to expand analysis beyond text-bearing corpus items;
- whether Telegram session storage should remain JSON-based long term.
