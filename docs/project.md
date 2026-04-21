# Project Snapshot

## Summary

Extractum is an MVP desktop tool for working with Telegram channels as local research sources.
Right now the project supports:
- creating multiple Telegram accounts;
- authenticating each account separately;
- persisting Telegram sessions locally;
- restoring saved Telegram sessions automatically after app restart;
- listing Telegram dialogs/channels for an authenticated account;
- registering Telegram channels as local sources in SQLite;
- manually syncing one source at a time into `items`;
- viewing synced messages inline in the Sources UI;
- configuring a Gemini provider profile and testing streaming responses from `/settings`;
- generating saved markdown reports from `/analysis` over already-synced messages;
- browsing saved analysis runs and trace data;
- managing reusable source groups for multi-source report runs;
- asking grounded follow-up questions over completed saved runs;
- persisting grounded chat history per saved analysis run.

## What exists in the codebase

### Frontend

- `/accounts`: create, list, and delete Telegram accounts, and show runtime Telegram readiness for each account
- `/auth/[id]`: initialize Telegram client, send code, sign in, sign out
- `/sources`: filter by account, load Telegram channels, add sources manually or from dialogs, sync a source, view synced messages, and show restore/runtime readiness
- `/settings`: edit the default Gemini provider profile and run a streaming test request
- `/analysis`: run saved report-style analysis over synced local messages, inspect traceability, manage source groups, and ask grounded follow-up questions
- global app layout with persistent light/dark theme toggle

### Backend commands

Implemented Tauri commands:
- `ping_db`
- `tg_init`
- `tg_is_authenticated`
- `tg_get_account_statuses`
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
- `get_llm_profiles`
- `save_llm_profile`
- `ask_llm_stream`
- `list_analysis_sources`
- `list_analysis_prompt_templates`
- `create_analysis_prompt_template`
- `update_analysis_prompt_template`
- `delete_analysis_prompt_template`
- `list_analysis_source_groups`
- `create_analysis_source_group`
- `update_analysis_source_group`
- `delete_analysis_source_group`
- `list_analysis_runs`
- `get_analysis_run`
- `get_analysis_run_trace`
- `resolve_analysis_trace_refs`
- `list_analysis_chat_messages`
- `clear_analysis_chat_messages`
- `start_analysis_report`
- `ask_analysis_run_question`

### Storage

Current schema includes:
- `accounts`
- `sources`
- `items`
- `app_settings`
- `analysis_prompt_templates`
- `analysis_runs`
- `analysis_source_groups`
- `analysis_source_group_members`
- `analysis_chat_messages`

Current active product flows use:
- `accounts` for multi-account setup;
- `sources` for source registration and sync cursors;
- `items` for synced Telegram messages;
- `app_settings` for temporary LLM provider profile storage.

## Current boundaries

In scope now:
- Telegram authentication
- background restore of saved Telegram sessions on startup
- account/source management
- manual per-source sync
- local message browsing
- reliable migrations
- shared SQLite access through `tauri-plugin-sql`
- ZSTD compression for source metadata and stored message payloads
- Gemini-first provider abstraction and streaming test calls
- backend-owned report generation over synced local `items`
- saved analysis runs with traceability data
- grounded report chat over completed saved runs

Out of scope in current implementation:
- background sync
- pagination beyond the simple first-page `get_items` call
- message edit/delete reconciliation
- media ingestion
- vector DB / embeddings / semantic retrieval

Planned next:
- media-aware sync metadata before full media download
- only then extend analysis beyond text-bearing messages

## Recommended reading order

1. `GEMINI.md`
2. `src-tauri/src/lib.rs`
3. `src-tauri/migrations/1.sql`, `2.sql`, `3.sql`, `4.sql`, `5.sql`, `6.sql`, `7.sql`, `8.sql`
4. `src-tauri/src/telegram.rs`
5. `src-tauri/src/sources.rs`
6. `src-tauri/src/llm.rs`
7. `src-tauri/src/analysis.rs`
8. `src/routes/accounts/+page.svelte`
9. `src/routes/auth/[id]/+page.svelte`
10. `src/routes/sources/+page.svelte`
11. `src/routes/settings/+page.svelte`
12. `src/routes/analysis/+page.svelte`
