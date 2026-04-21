# Extractum: Project Context & AI Guidelines

This file is a foundational mandate for AI agents. It takes precedence over general workflows.

## 1. Architectural Model: "Fat Frontend, Thin Backend"

- **Frontend (SvelteKit + TypeScript):** The orchestration layer. Handles UI state, user flows, filtering, LLM context preparation, and SQL parameter logic.
- **Backend (Tauri + Rust):** A thin systems layer. Handles MTProto (Telegram), SQLite persistence, ZSTD compression/decompression, secure secret management, and LLM provider proxying.
- **Rule:** Keep business logic in the frontend. The backend should provide small, reliable primitives.

## 2. Telegram Integration (grammers 0.8.x)

We use the `master` branch of `grammers`. The API has specific constraints that MUST be followed:

- **Imports:** `LoginToken` is exported via `grammers_client::client::LoginToken`. Do NOT look for it in `auth` or `types`.
- **Signatures:** `client.request_login_code` takes 2 arguments: `(&phone, api_hash)`. The `api_id` is already managed by the `SenderPool`.
- **Multiple accounts:** `TelegramState` holds a `HashMap<account_id, AccountClient>`. Each account has its own `Client`, `SenderPool`, `MemorySession`, and session file (`telegram_{id}.session.json`). All commands take `account_id: i64`.
- **Session Management:** Persisted to `telegram_{account_id}.session.json` in `app_data_dir`. Loaded on `tg_init`, saved on `tg_sign_in`, deleted on `tg_logout`.
- **`FileSession` does NOT exist** in this version of grammers. `storages` only exports `MemorySession` and `SqliteSession` (behind `sqlite-storage` feature). Do not attempt to use `FileSession`.
- **`SessionData` does NOT implement `serde::Serialize/Deserialize`** directly. Use the `SavedSession` wrapper struct that mirrors its fields (`home_dc`, `dc_options`, `updates_state`).
- **Cargo.toml:** `grammers-client` MUST have `default-features = false`. `grammers-session` MUST have `default-features = false, features = ["serde"]`. This prevents `libsql` from being pulled in and causing duplicate SQLite symbol conflicts with `tauri-plugin-sql`.

## 3. Storage & Data Model

- **Database:** SQLite is the single source of truth for local data.
- **DB location:** `tauri-plugin-sql` stores the database in `app_config_dir` (e.g. `AppData\Roaming\org.ai.extractum\extractum.db` on Windows). This is different from `app_data_dir`. All Rust code accessing the DB must use the same path.
- **DB initialization:** The database is preloaded at Rust startup via `plugins.sql.preload` in `tauri.conf.json`. This guarantees migrations run before any frontend command is invoked. Do NOT rely on `Database.load()` from the frontend for migration timing.
- **Rust DB access:** `sources.rs` accesses the DB by retrieving the `Pool<Sqlite>` from `DbInstances` state managed by `tauri-plugin-sql`. This ensures both the plugin and Rust commands use the same connection pool.
- **Migrations:** Numbered SQL files in `src-tauri/migrations/`, registered in `lib.rs`. Rules:
  - Never delete or rename a migration file — sqlx verifies all previously applied migrations still exist.
  - Never change the SQL of an already-applied migration — sqlx verifies checksums. If a change is unavoidable, add a `patch_migrations()` call in `lib.rs` `setup` hook to delete the stale record from `_sqlx_migrations` before the plugin runs.
  - Migration 2 (`add is_member to sources`) is a no-op (`SELECT 1`) because `is_member` was already included in migration 1. Its `_sqlx_migrations` record is patched at startup via `patch_migrations()`.
  - Always add new schema changes as a new migration with the next version number.
- **Compression:** Heavy fields (`content_zstd`, `raw_data_zstd`) MUST be compressed using ZSTD before storage and decompressed on read.
- **No Vector DB:** MVP explicitly avoids vector databases and embeddings. Context for LLM is derived directly from SQL selections.

## 4. LLM Strategy

- **Primary Provider:** Google Gemini.
- **Flow:** Frontend selects records -> Frontend builds text context -> Backend proxies request to Gemini API.

## 5. Development Workflow Mandates

- **Research First:** Before attempting to fix API errors in external libraries (like `grammers`), the agent MUST read the library source code or search for updated documentation. Never "guess" import paths or method signatures.
- **Empirical Validation:** Always run `cargo check` after modifying Rust code to ensure compilation.
- **Security:** Never log secrets. Keep API keys in the backend (using system keyring where appropriate).

## 6. Current Status (as of 2026-04-21)

- Phase 2 (Telegram Integration) is in progress.
- Multi-account support implemented: multiple Telegram accounts work simultaneously, each with independent client and session.
- Full authentication flow per account (init, send code, sign in, logout) implemented.
- Session persistence per account: `telegram_{id}.session.json` in app data dir.
- Account management: `accounts` table, commands `list_accounts`, `create_account`, `set_account_phone`, `delete_account`.
- Source management: sources linked to accounts via `account_id`, filterable by account.
- DB preloaded at Rust startup via `tauri.conf.json` `plugins.sql.preload` — no frontend dependency for migrations.
- Migration patch mechanism in place for migration 2 (see Storage section).
- Next step: Implementing channel synchronization (`sync_channel`) and ZSTD-compressed message storage.
