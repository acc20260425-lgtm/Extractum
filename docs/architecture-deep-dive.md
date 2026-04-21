# Architecture Deep Dive

## 1. Core principle

Extractum uses a "fat frontend, thin backend" architecture.

The practical meaning in this codebase is:
- Svelte pages own user-facing flows and UI state;
- Rust owns integration boundaries and persistence boundaries;
- the frontend should call focused Tauri commands instead of reaching into Telegram or SQLite details directly.

## 2. Current runtime structure

### Frontend

Current pages:
- `src/routes/accounts/+page.svelte`
- `src/routes/auth/[id]/+page.svelte`
- `src/routes/sources/+page.svelte`

Shared shell:
- `src/routes/+layout.svelte`

Responsibilities currently implemented in frontend:
- account creation form state
- auth step transitions
- source selection flows
- account filtering in UI
- theme selection and persistence

### Backend

Current Rust modules:
- `src-tauri/src/lib.rs`
- `src-tauri/src/telegram.rs`
- `src-tauri/src/sources.rs`

Responsibilities currently implemented in backend:
- Tauri bootstrap
- SQL plugin and migration registration
- migration metadata patching before plugin initialization
- Telegram client initialization per account
- Telegram login/logout flow
- Telegram session file persistence
- account CRUD against SQLite
- source listing and registration against SQLite
- Telegram dialog discovery

## 3. Telegram subsystem

`telegram.rs` manages active MTProto clients in memory.

Current structure:
- one `TelegramState`
- one `HashMap<account_id, AccountClient>`
- one Telegram client per account
- one session file per account: `telegram_{account_id}.session.json`

Current supported Telegram flow:
1. load account credentials from SQLite;
2. initialize a client for that account;
3. send login code;
4. sign in;
5. persist session to disk;
6. reuse session on later startup;
7. delete session on logout.

## 4. Storage subsystem

SQLite is the only local application database.

Important architectural decisions already reflected in code:
- the database is preloaded at startup with `tauri-plugin-sql`;
- Rust commands read the pool from `DbInstances`;
- the app should not create a second independent SQLite connection path to another file;
- migration metadata repair happens before SQL plugin initialization, not in Tauri `setup()`.

This prevents:
- mismatched DB paths;
- commands racing migrations on startup;
- checksum-related startup failures for older local databases.

## 5. Current command surface

The active Tauri command layer is intentionally small:
- DB health: `ping_db`
- Telegram auth: `tg_init`, `tg_is_authenticated`, `tg_send_code`, `tg_sign_in`, `tg_logout`
- Accounts: `list_accounts`, `get_account`, `create_account`, `set_account_phone`, `clear_account_phone`, `delete_account`
- Sources: `list_telegram_channels`, `add_telegram_source`, `list_sources`

This matches the current implemented product slice.

## 6. What is not implemented yet

The architecture already reserves space for later subsystems, but they are not present in the running app yet:
- message sync into `items`
- ZSTD write/read path for stored messages
- browsing and filtering stored items
- LLM provider abstraction in code
- Gemini integration
- analysis UI

Those are still planned layers, not current architecture facts.

## 7. UI architecture notes

The UI is intentionally minimal right now:
- route-based pages
- no shared component library yet
- no settings page yet
- no dashboard yet

Recent current-state detail:
- the app now supports both light and dark themes;
- light theme is the default;
- theme preference is persisted in `localStorage`.

## 8. Recommended direction

Near-term implementation should continue in this order:
1. implement `sync_channel`;
2. write messages into `items`;
3. add message browsing UI;
4. add analysis flow and provider integration.

That preserves the intended architecture: frontend orchestration, backend integrations, SQLite as the single local source of truth.
