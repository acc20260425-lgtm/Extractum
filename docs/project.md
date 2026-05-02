# Project State

This document is the shortest current-state snapshot of the repository.

## Stack

- frontend: `SvelteKit 2`, `Svelte 5`, `TypeScript`
- desktop shell: `Tauri 2`
- backend: `Rust`
- local storage: `SQLite`
- LLM: reusable Gemini and OpenAI-compatible provider profiles

## Product slice

The app is a local Telegram ingest and analysis workspace.

Implemented:

- Telegram account management and sign-in flow
- startup session restore
- source management for Telegram channels, supergroups, and groups
- history sync into local SQLite
- media-aware sync metadata for text-bearing and media-only items
- Telegram reply/thread/reaction context metadata for newly synced items
- configurable initial sync window
- source groups for analysis
- saved reports
- follow-up chat on saved runs
- single-source NotebookLM export with local reply/thread/reaction metadata
- reusable LLM provider profiles with active-profile selection
- configurable OpenAI-compatible `base_url` support in `/settings`
- provider smoke testing from `/settings`
- immutable saved run corpus snapshots
- typed app errors across Tauri commands

Not implemented yet:

- secure storage for all secrets
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

- `accounts`: local Telegram accounts and their current SQLite-backed credentials
- `sources`: registered Telegram sources
- `items`: synced Telegram messages, media-aware metadata, and nullable Telegram context metadata for new rows
- `app_settings`: app-level key/value storage, including active LLM profile, per-profile provider metadata, sync policy, and the current temporary LLM API keys
- `analysis_runs`: saved report runs
- `analysis_run_messages`: frozen corpus snapshot for saved runs
- `analysis_chat_messages`: follow-up chat history

## Current practical constraints

- analysis corpus still requires text content;
- media-only items are stored and visible, but not yet analyzed;
- older item rows may have `NULL` Telegram context metadata because there is no background backfill;
- LLM API keys still remain in `app_settings` for now;
- Telegram `api_hash` still remains in SQLite-backed account storage for now;
- Telegram peer resolution can still fall back to dialog scanning, especially for private sources.

## Reading order for implementation work

1. `src-tauri/src/sources.rs`
2. `src-tauri/src/analysis/`
3. `src-tauri/src/llm/`
4. `src/routes/analysis/+page.svelte`
5. `src/lib/components/analysis/`
6. `src/routes/settings/+page.svelte`
7. `src/routes/sources/+page.svelte`
8. `src-tauri/src/error.rs`
9. `src-tauri/src/migrations.rs`
