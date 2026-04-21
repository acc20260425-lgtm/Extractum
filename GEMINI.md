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
- **Session Management:** Implemented. Session is persisted to `telegram.session.json` in the app data directory using a custom `SavedSession` struct serialized via `serde_json`. Loaded on `tg_init`, saved on `tg_sign_in`, deleted on `tg_logout`.
- **`FileSession` does NOT exist** in this version of grammers. `storages` only exports `MemorySession` and `SqliteSession` (behind `sqlite-storage` feature). Do not attempt to use `FileSession`.
- **`SessionData` does NOT implement `serde::Serialize/Deserialize`** directly. Use the `SavedSession` wrapper struct that mirrors its fields (`home_dc`, `dc_options`, `updates_state`).
- **Cargo.toml:** `grammers-client` MUST have `default-features = false`. `grammers-session` MUST have `default-features = false, features = ["serde"]`. This prevents `libsql` from being pulled in and causing duplicate SQLite symbol conflicts with `tauri-plugin-sql`.

## 3. Storage & Data Model

- **Database:** SQLite is the single source of truth for local data.
- **Migrations:** Managed via `tauri-plugin-sql` in `src-tauri/migrations/`. Migrations are applied only when `Database.load()` is called from the frontend. The `+layout.svelte` calls it on app start to guarantee migrations run before any page renders. New migrations must be registered in `lib.rs` with an incrementing `version` number.
- **Direct DB access from Rust:** `sources.rs` uses `sqlx` directly (same file path as `tauri-plugin-sql`). Always ensure `Database.load()` has been called first (guaranteed by layout) before Rust commands that use `sqlx` are invoked.
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
- Full authentication flow (init, send code, sign in, logout) is implemented and compiles cleanly.
- Session persistence implemented: saved to `telegram.session.json` in app data dir, loaded on restart.
- Settings persistence (`api_id`, `api_hash`) in SQLite is implemented.
- Source management implemented: `list_telegram_channels`, `add_telegram_source`, `list_sources` commands.
- Sources UI: dialog-based channel picker + manual username/link input, sources list with `is_member` badge.
- DB migration mechanism: `tauri-plugin-sql` applies migrations on `Database.load()` call; layout triggers this on app start before any page renders.
- Next step: Implementing channel synchronization (`sync_channel`) and ZSTD-compressed message storage.
