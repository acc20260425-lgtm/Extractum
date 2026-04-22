# Project State

This document is the shortest current-state snapshot of the repository.

## Stack

- frontend: `SvelteKit 2`, `Svelte 5`, `TypeScript`
- desktop shell: `Tauri 2`
- backend: `Rust`
- local storage: `SQLite`
- LLM: Gemini provider flow

## Product slice

The app is a local Telegram ingest and analysis workspace.

Implemented:

- Telegram account management and sign-in flow
- startup session restore
- source management for Telegram broadcast channels
- history sync into local SQLite
- media-aware sync metadata for text-bearing and media-only items
- configurable initial sync window
- source groups for analysis
- saved reports
- follow-up chat on saved runs
- immutable saved run corpus snapshots
- typed app errors across Tauri commands

Not implemented yet:

- secure storage for all secrets
- full media download / previews
- media-aware analysis beyond the current text-first corpus

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
  - list Telegram dialogs
  - add sources manually or from dialogs
  - sync source history
  - configure the first sync policy
  - browse synced items
- `/settings`
  - store active LLM provider profile
  - run a Gemini connectivity test
- `/analysis`
  - manage report templates
  - manage source groups
  - run reports
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

- `list_telegram_channels`
- `add_telegram_source`
- `list_sources`
- `delete_source`
- `sync_channel`
- `get_items`
- `get_sync_settings`
- `save_sync_settings`

### Analysis

- report generation
- saved runs listing and detail loading
- trace resolution
- follow-up chat
- prompt template CRUD
- source group CRUD

### Settings / LLM

- load and save LLM settings
- test Gemini provider connectivity

## Important persistence

- `sources`: registered Telegram channels
- `items`: synced Telegram messages and media-aware metadata
- `app_settings`: app-level key/value storage
- `analysis_runs`: saved report runs
- `analysis_run_messages`: frozen corpus snapshot for saved runs
- `analysis_chat_messages`: follow-up chat history

## Current practical constraints

- analysis corpus still requires text content;
- media-only items are stored and visible, but not yet analyzed;
- LLM `api_key` remains in `app_settings` for now;
- Telegram peer resolution can still fall back to dialog scanning.

## Reading order for implementation work

1. `src-tauri/src/sources.rs`
2. `src-tauri/src/analysis/`
3. `src/routes/sources/+page.svelte`
4. `src/routes/analysis/+page.svelte`
5. `src-tauri/src/error.rs`
6. `src-tauri/src/migrations.rs`
7. `src-tauri/migrations/9.sql`
8. `src-tauri/migrations/10.sql`
