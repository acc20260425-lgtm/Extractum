# Extractum: Project Context & AI Guidelines

This file is a working contract for AI agents modifying the repository.
It should reflect the current codebase, not the aspirational end-state.

## 1. Architecture

Extractum uses a "fat frontend, thin backend" model.

- Frontend (`SvelteKit + TypeScript`): UI state, route flows, filters, orchestration, presentation.
- Backend (`Tauri + Rust`): Telegram integration, SQLite access, migrations, compression, session persistence, security boundaries, future provider calls.

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
- `TelegramState` also stores per-account runtime readiness status used by the UI;
- each account has an independent session file: `telegram_{account_id}.session.json`;
- `FileSession` is not available in this setup;
- `SessionData` is wrapped through a serializable `SavedSession` struct for persistence.

Current implemented Telegram flow:
- `tg_init`
- `tg_is_authenticated`
- `tg_get_account_statuses`
- `tg_send_code`
- `tg_sign_in`
- `tg_logout`
- `list_telegram_sources`
- `add_telegram_source`
- `sync_source`

Current runtime restore behavior:
- on app startup, the backend tries to restore saved account sessions in the background;
- restore must not block window startup;
- runtime statuses currently used in UI are:
  - `not_initialized`
  - `restoring`
  - `ready`
  - `reauth_required`
  - `restore_failed`

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
- `tg_get_account_statuses`
- `tg_send_code`
- `tg_sign_in`
- `tg_logout`

Sources and items:
- `list_telegram_sources`
- `add_telegram_source`
- `list_sources`
- `sync_source`
- `get_items`

LLM:
- `get_llm_profiles`
- `save_llm_profile`
- `ask_llm_stream`

Utility:
- `ping_db`

## 6. Current product status

Implemented:
- multi-account Telegram setup
- per-account auth flow
- session persistence
- startup restore of saved sessions
- account CRUD
- source registration linked to account and Telegram source kind
- source discovery from Telegram dialogs
- manual per-source sync into `items`
- inline browsing of synced messages on `/sources`
- runtime status display on `/accounts` and `/sources`
- minimal Gemini settings/test UI on `/settings`
- persistent light/dark theme toggle, defaulting to light

Current sync constraints:
- analysis remains text-first even though ingest stores media metadata
- duplicates ignored
- no binary media download
- no edit/delete reconciliation
- no background sync

Not implemented yet:
- richer item filtering/pagination
- dedicated message detail views
- media-aware analysis beyond the current text-first corpus

## 7. Workflow rules for agents

- Read the current Rust code before changing `grammers` integration.
- Run `cargo check` after Rust changes.
- Prefer updating documentation when code meaningfully changes.
- Do not introduce vector DB / embedding assumptions into MVP docs or code.
- Do not reintroduce direct frontend ownership of low-level SQLite behavior unless the current design explicitly allows it.
- Keep compression/decompression for persisted data in Rust unless the architecture explicitly changes.

## 8. Security rules

- never log secrets;
- keep Telegram session persistence in the backend;
- current temporary exception: the Gemini API key is stored in SQLite and read back into `/settings` for editing;
- treat that exception as temporary security debt and document it when touching LLM code;
- if secrets later move to secure storage, keep them profile-scoped and aligned with app identity;
- recommended app identity scheme for parallel installs:
  - `org.ai.extractum`
  - `org.ai.extractum.dev`
  - `org.ai.extractum.test`
  - `org.ai.extractum.beta`
- validate backend command inputs;
- preserve the frontend/backend boundary.
