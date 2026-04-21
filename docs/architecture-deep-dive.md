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
- polling runtime Telegram readiness for rendered accounts/sources
- source selection flows
- account filtering in UI
- manual sync triggers
- inline message browsing state
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
- background restore of saved Telegram sessions on startup
- Telegram login/logout flow
- Telegram session file persistence
- account CRUD against SQLite
- source listing and registration against SQLite
- Telegram dialog discovery
- source resolution for sync
- item persistence and retrieval
- ZSTD compression/decompression for persisted metadata and message content

## 3. Telegram subsystem

`telegram.rs` manages active MTProto clients in memory.

Current structure:
- one `TelegramState`
- one `HashMap<account_id, AccountClient>`
- one runtime status map keyed by `account_id`
- one Telegram client per account
- one session file per account: `telegram_{account_id}.session.json`

Current supported Telegram flow:
1. load account credentials from SQLite;
2. initialize a client for that account;
3. send login code;
4. sign in;
5. persist session to disk;
6. restore saved sessions in the background on later startup;
7. expose runtime status as one of `not_initialized`, `restoring`, `ready`, `reauth_required`, `restore_failed`;
8. delete session on logout.

Current sync flow:
1. frontend calls `sync_channel(source_id)`;
2. backend loads the source and its `account_id`;
3. backend gets the active Telegram client for that account;
4. backend resolves the source channel from dialogs or stored username metadata;
5. backend iterates Telegram messages;
6. backend writes normalized rows into `items`;
7. backend updates `sources.last_sync_state`.

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
- Telegram runtime state: `tg_get_account_statuses`
- Accounts: `list_accounts`, `get_account`, `create_account`, `set_account_phone`, `clear_account_phone`, `delete_account`
- Sources: `list_telegram_channels`, `add_telegram_source`, `list_sources`, `sync_channel`
- Items: `get_items`

This matches the current implemented product slice.

## 6. Current sync constraints

The first sync slice is intentionally narrow:
- sync is manual and per source;
- only already-registered sources are syncable;
- only text/caption content is stored;
- empty-text messages are skipped;
- duplicates are ignored, not updated;
- there is no background worker;
- there is no reconciliation for edits or deletions;
- there is no media ingestion.

This is a deliberate MVP constraint, not an accidental omission.

## 7. UI architecture notes

The UI is still intentionally small:
- route-based pages
- no shared component library yet
- no settings page yet
- no dedicated message browser route yet

Current-state details:
- the app supports both light and dark themes;
- light theme is the default;
- theme preference is persisted in `localStorage`;
- the Sources page now combines source management, sync actions, and a first-pass inline message viewer;
- both `/accounts` and `/sources` surface Telegram runtime readiness from backend state.

## 8. Recommended direction

Near-term implementation should continue in this order:
1. improve message browsing and filtering over `items`;
2. add pagination or incremental loading to `get_items`;
3. replace simple UI polling of runtime account status with a more event-driven approach when it becomes worth the complexity;
4. add analysis flow and provider integration.

That preserves the intended architecture: frontend orchestration, backend integrations, SQLite as the single local source of truth.
