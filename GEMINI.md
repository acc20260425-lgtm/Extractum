# Extractum: Project Context & AI Guidelines

This file is a working contract for AI agents modifying the repository.
It should reflect the current codebase, not the aspirational end-state.

## 1. Architecture

Extractum uses a "fat frontend, thin backend" model.

- Frontend (`SvelteKit + TypeScript`): UI state, route flows, filters, orchestration, presentation.
- Backend (`Tauri + Rust`): Telegram integration, SQLite access, migrations, session persistence, security boundaries, future provider calls.

Rule:
- keep low-level integration logic in Rust;
- keep user-flow orchestration in the frontend;
- prefer small, explicit Tauri commands over broad generic commands.

## 2. Telegram integration rules

The project uses the `master` branch of `grammers`.

Important API constraints:
- `LoginToken` is imported from `grammers_client::client::LoginToken`;
- `client.request_login_code(&phone, api_hash)` takes two arguments;
- `TelegramState` stores active clients as `HashMap<account_id, AccountClient>`;
- each account has an independent session file: `telegram_{account_id}.session.json`;
- `FileSession` is not available in this setup;
- `SessionData` is wrapped through a serializable `SavedSession` struct for persistence.

Current implemented Telegram flow:
- `tg_init`
- `tg_is_authenticated`
- `tg_send_code`
- `tg_sign_in`
- `tg_logout`

## 3. Database rules

- SQLite is the only local database.
- The DB file is `extractum.db` in `app_config_dir`.
- The DB is preloaded at startup through `plugins.sql.preload` in `tauri.conf.json`.
- Rust commands must use the pool exposed by `tauri-plugin-sql` through `DbInstances`.

Do not:
- open a second "manual" SQLite path to a different file;
- rely on frontend `Database.load()` for migration timing;
- assume `app_data_dir` and `app_config_dir` are interchangeable.

## 4. Migration rules

Migrations live in `src-tauri/migrations/` and are registered in `src-tauri/src/lib.rs`.

Rules:
- never delete or rename an existing migration file;
- never casually rewrite an already-applied migration;
- always add new schema changes as a new migration;
- if historical migration metadata must be repaired, do it before SQL plugin initialization.

Important current detail:
- `2.sql` is intentionally a no-op (`SELECT 1;`);
- migration metadata for version 2 may need repair on older local databases;
- the project patches `_sqlx_migrations` before registering the SQL plugin.

## 5. Current implemented command surface

Accounts and auth:
- `list_accounts`
- `get_account`
- `create_account`
- `set_account_phone`
- `clear_account_phone`
- `delete_account`
- `tg_init`
- `tg_is_authenticated`
- `tg_send_code`
- `tg_sign_in`
- `tg_logout`

Sources:
- `list_telegram_channels`
- `add_telegram_source`
- `list_sources`

Utility:
- `ping_db`

Not implemented yet:
- `sync_channel`
- `get_items`
- `ask_llm`

## 6. Current product status

Implemented:
- multi-account Telegram setup
- per-account auth flow
- session persistence
- account CRUD
- source registration linked to account
- source discovery from Telegram dialogs
- persistent light/dark theme toggle, defaulting to light

Not implemented yet:
- message sync into `items`
- browsing stored messages
- LLM provider integration
- Gemini analysis flow

## 7. Workflow rules for agents

- Read the current Rust code before changing `grammers` integration.
- Run `cargo check` after Rust changes.
- Prefer updating documentation when code meaningfully changes.
- Do not introduce vector DB / embedding assumptions into MVP docs or code.
- Do not reintroduce direct frontend ownership of low-level SQLite or secret-handling behavior.

## 8. Security rules

- never log secrets;
- keep Telegram session persistence in the backend;
- keep provider/API secrets in the backend;
- validate backend command inputs;
- preserve the frontend/backend boundary.
