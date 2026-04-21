# Project Snapshot

## Summary

Extractum is an MVP desktop tool for working with Telegram channels as local research sources.
Right now the project supports:
- creating multiple Telegram accounts;
- authenticating each account separately;
- persisting Telegram sessions locally;
- listing Telegram dialogs/channels for an authenticated account;
- registering Telegram channels as local sources in SQLite.

The next major implementation milestone is message synchronization into the `items` table.

## What exists in the codebase

### Frontend

- `/accounts`: create, list, and delete Telegram accounts
- `/auth/[id]`: initialize Telegram client, send code, sign in, sign out
- `/sources`: filter by account, load Telegram channels, add sources manually or from dialogs
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

### Storage

Current schema includes:
- `accounts`
- `sources`
- `items`
- `app_settings`

At the moment, the active product flows use `accounts` and `sources`.
`items` exists in schema but is not populated yet because sync is not implemented.

## Current boundaries

In scope now:
- Telegram authentication
- account/source management
- reliable migrations
- shared SQLite access through `tauri-plugin-sql`

Out of scope in current implementation:
- channel sync
- message browsing
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
