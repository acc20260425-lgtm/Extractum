# Project Snapshot

## Summary

Extractum is an MVP desktop tool for working with Telegram channels as local research sources.
Right now the project supports:
- creating multiple Telegram accounts;
- authenticating each account separately;
- persisting Telegram sessions locally;
- listing Telegram dialogs/channels for an authenticated account;
- registering Telegram channels as local sources in SQLite;
- manually syncing one source at a time into `items`;
- viewing synced messages inline in the Sources UI.

## What exists in the codebase

### Frontend

- `/accounts`: create, list, and delete Telegram accounts
- `/auth/[id]`: initialize Telegram client, send code, sign in, sign out
- `/sources`: filter by account, load Telegram channels, add sources manually or from dialogs, sync a source, view synced messages
- global app layout with persistent light/dark theme toggle

### Backend commands

Implemented Tauri commands:
- `ping_db`
- `tg_init`
- `tg_is_authenticated`
- `tg_send_code`
- `tg_sign_in`
- `tg_logout`
- `list_accounts`
- `get_account`
- `create_account`
- `set_account_phone`
- `clear_account_phone`
- `delete_account`
- `list_telegram_channels`
- `add_telegram_source`
- `list_sources`
- `sync_channel`
- `get_items`

### Storage

Current schema includes:
- `accounts`
- `sources`
- `items`
- `app_settings`

Current active product flows use:
- `accounts` for multi-account setup;
- `sources` for source registration and sync cursors;
- `items` for synced Telegram messages.

## Current boundaries

In scope now:
- Telegram authentication
- account/source management
- manual per-source sync
- local message browsing
- reliable migrations
- shared SQLite access through `tauri-plugin-sql`
- ZSTD compression for source metadata and stored message payloads

Out of scope in current implementation:
- background sync
- pagination beyond the simple first-page `get_items` call
- message edit/delete reconciliation
- media ingestion
- LLM analysis
- vector DB / embeddings / semantic retrieval

## Recommended reading order

1. `GEMINI.md`
2. `src-tauri/src/lib.rs`
3. `src-tauri/migrations/1.sql`, `2.sql`, `3.sql`
4. `src-tauri/src/telegram.rs`
5. `src-tauri/src/sources.rs`
6. `src/routes/accounts/+page.svelte`
7. `src/routes/auth/[id]/+page.svelte`
8. `src/routes/sources/+page.svelte`
