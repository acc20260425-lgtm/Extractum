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
- **Session Management:** Currently using `MemorySession`. Transition to persistent storage (file or DB) is planned.
- **Cargo.toml:** `grammers-client` and `grammers-session` MUST have `default-features = false` to avoid conflicts between `libsql` and `tauri-plugin-sql`.

## 3. Storage & Data Model

- **Database:** SQLite is the single source of truth for local data.
- **Migrations:** Managed via `tauri-plugin-sql` in `src-tauri/migrations/`.
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
- Basic authentication flow (init, send code, sign in) is implemented and verified.
- Settings persistence (`api_id`, `api_hash`) in SQLite is implemented.
- Next step: Implementing channel synchronization and ZSTD-compressed message storage.
