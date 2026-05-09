# Project State

This document is the shortest current-state snapshot of the repository.

## Stack

- frontend: `SvelteKit 2`, `Svelte 5`, `TypeScript`
- desktop shell: `Tauri 2`
- backend: `Rust`
- local storage: `SQLite`
- LLM: reusable Gemini and OpenAI-compatible provider profiles

## Product slice

The app is a local source ingest and analysis workspace. Telegram is the only
implemented ingest provider today, while the shared source model and analysis
boundary are ready for future non-Telegram providers.

Implemented:

- Telegram account management and sign-in flow
- startup session restore
- source management for Telegram channels, supergroups, and groups
- provider-ready source records with `source_type` and `source_subtype`
- capability-driven source UI for sync, Takeout, membership, and topic actions
- history sync into local SQLite
- provider-dispatched source sync with Telegram as the implemented provider
- Takeout source import for existing Telegram sources with TDesktop-first pagination
- media-aware sync metadata for text-bearing and media-only items
- Telegram reply/thread/reaction context metadata for newly synced items
- configurable initial sync window
- source groups for analysis
- saved reports
- follow-up chat on saved runs
- analysis report preflight limits for large selected corpora
- single-source NotebookLM export with local reply/thread/reaction metadata
- reusable LLM provider profiles with active-profile selection
- configurable OpenAI-compatible `base_url` support in `/settings`
- provider smoke testing from `/settings`
- immutable saved run corpus snapshots
- provider-neutral analysis refs for new live corpus rows
- typed app errors across Tauri commands
- OS secure storage for saved LLM API keys and Telegram `api_hash` values
- encrypted Telegram session file contents with per-account OS secure storage keys

Not implemented yet:

- concrete YouTube, RSS, or forum ingestion
- full media download / previews
- media-aware analysis beyond the current text-first corpus
- full Telegram Forum Topics browsing/export model
- Telegram forward metadata enrichment

## Main routes

- `/accounts`
  - create and delete local Telegram accounts
  - observe runtime status updates
- `/auth/[id]`
  - initialize Telegram login
  - send code
  - sign in
  - log out
- `/sources`
  - lightweight compatibility route
  - points older entry paths to the main analysis workspace
- `/settings`
  - manage reusable LLM provider profiles
  - set the active profile used by default
  - edit provider-specific `base_url` settings for OpenAI-compatible providers
  - refresh available models
  - run a live provider smoke test with the currently edited form
- `/analysis`
  - browse sources and inspect synced items
  - add sources manually or from dialogs
  - sync source history
  - start/cancel Takeout source imports and monitor import progress
  - configure the first sync policy
  - manage report templates
  - manage source groups
  - run reports
  - monitor active queued/running reports separately from history
  - browse saved runs through global history or the current analysis scope
  - inspect trace refs
  - ask follow-up questions against saved runs

## Backend command areas

### Accounts / auth

- account CRUD
- `tg_init`
- `tg_send_code`
- `tg_sign_in`
- `tg_logout`
- runtime account status refresh / restore

### Sources

- `list_telegram_sources`
- `add_telegram_source`
- `list_sources`
- `delete_source`
- `sync_source`
- `start_takeout_source_import`
- `cancel_takeout_source_import`
- `list_takeout_source_import_jobs`
- `get_items`
- `get_sync_settings`
- `save_sync_settings`

### Analysis

- report generation
- active runs listing and restoration
- saved runs listing, scoped/global history browsing, and detail loading
- trace resolution
- follow-up chat
- prompt template CRUD
- source group CRUD

### Settings / LLM

- load and save LLM profiles
- switch the active LLM profile
- list provider models for Gemini and OpenAI-compatible endpoints
- stream provider test requests and analysis/chat requests through the resolved profile

## Important persistence

- `accounts`: local Telegram account metadata; saved Telegram `api_hash` secrets live in OS secure storage
- Telegram session files remain app-data files, but their contents are encrypted with per-account session keys stored in OS secure storage under `telegram.account.<account_id>.session_key`.
- `sources`: registered provider sources; Telegram rows currently carry
  Telegram compatibility fields and future providers can use provider-local
  subtypes
- `items`: ingested source items; currently Telegram messages with media-aware
  metadata and nullable Telegram context metadata for new rows
- no persistent table exists for Takeout import jobs; job records are in-memory runtime state
- `app_settings`: app-level key/value storage, including active LLM profile, per-profile non-secret provider metadata, and sync policy
- `analysis_runs`: saved report runs
- `analysis_run_messages`: frozen corpus snapshot for saved runs
- `analysis_chat_messages`: follow-up chat history

## LLM scheduling and analysis caps

LLM scheduling allows two running requests per `(provider, profile)` and prioritizes interactive requests over background work. Analysis report runs run a backend preflight before run creation and are capped at `10_000` messages, `80` estimated chunks, `1_500_000` estimated input characters, and `80` background requests.

## Current practical constraints

- analysis corpus still requires text content;
- media-only items are stored and visible, but not yet analyzed;
- concrete non-Telegram ingestion commands are not implemented yet;
- older item rows may have `NULL` Telegram context metadata because there is no background backfill;
- saved LLM API keys and Telegram `api_hash` values use OS secure storage;
- Telegram session files remain app-data files, but their contents are encrypted with per-account session keys stored in OS secure storage under `telegram.account.<account_id>.session_key`;
- Telegram peer resolution can still fall back to dialog scanning, especially for private sources.
- Takeout import does not download media bytes and currently defers migrated supergroup history to avoid `(source_id, external_id)` collisions.

## Reading order for implementation work

1. `src-tauri/src/sources.rs`
2. `src-tauri/src/source_ingest.rs`
3. `src-tauri/src/takeout_import.rs`
4. `src-tauri/src/takeout_import/raw_parse.rs`
5. `src-tauri/src/analysis/`
6. `src-tauri/src/llm/`
7. `src/routes/analysis/+page.svelte`
8. `src/lib/components/analysis/`
9. `src/routes/settings/+page.svelte`
10. `src/routes/sources/+page.svelte`
11. `src-tauri/src/error.rs`
12. `src-tauri/src/migrations.rs`

Related deep dive: `docs/takeout-source-import.md`.
