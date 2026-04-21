# Database Schema Design

## 1. Storage model

Extractum uses SQLite as the only local database.
The schema is intentionally small and supports the current product slice plus the next planned step of message synchronization.

Today the application actively uses:
- `accounts`
- `sources`

The schema also already includes:
- `items`
- `app_settings`

`items` is reserved for upcoming message sync and browsing flows.

## 2. Database location and initialization

- database file: `extractum.db`
- location: `app_config_dir` managed by `tauri-plugin-sql`
- preload: configured in `src-tauri/tauri.conf.json` under `plugins.sql.preload`

This matters because:
- migrations run before frontend commands are invoked;
- Rust commands and the plugin must use the same database file and the same pool;
- direct ad-hoc SQLite access with a different path will create inconsistent state.

Rust DB access in the project goes through `DbInstances` from `tauri-plugin-sql`.

## 3. Tables

### 3.1 `accounts`

Stores Telegram account configuration.

| Column | Type | Notes |
| :--- | :--- | :--- |
| `id` | INTEGER | Primary key |
| `label` | TEXT | Human-friendly account name |
| `api_id` | INTEGER | Telegram API ID |
| `api_hash` | TEXT | Telegram API hash |
| `phone` | TEXT | Set after successful sign-in |
| `created_at` | INTEGER | Unix timestamp, UTC |

### 3.2 `sources`

Stores configured data sources such as Telegram channels.

| Column | Type | Notes |
| :--- | :--- | :--- |
| `id` | INTEGER | Primary key |
| `source_type` | TEXT | Currently `telegram_channel` |
| `external_id` | TEXT | Telegram channel identifier |
| `title` | TEXT | Source title |
| `metadata_zstd` | BLOB | Reserved for compressed metadata |
| `last_sync_state` | INTEGER | Reserved sync cursor |
| `is_active` | BOOLEAN | Whether source participates in sync |
| `is_member` | BOOLEAN | Whether the account is subscribed |
| `created_at` | INTEGER | Unix timestamp, UTC |
| `account_id` | INTEGER | FK to `accounts.id` |

### 3.3 `items`

Stores collected Telegram messages once sync is implemented.

| Column | Type | Notes |
| :--- | :--- | :--- |
| `id` | INTEGER | Primary key |
| `source_id` | INTEGER | FK to `sources.id` |
| `external_id` | TEXT | Telegram message identifier |
| `author` | TEXT | Optional sender/author |
| `published_at` | INTEGER | Original publication time |
| `ingested_at` | INTEGER | Ingestion time |
| `content_zstd` | BLOB | Reserved compressed text |
| `raw_data_zstd` | BLOB | Reserved compressed payload |

### 3.4 `app_settings`

Stores simple key/value application settings.

| Column | Type | Notes |
| :--- | :--- | :--- |
| `key` | TEXT | Primary key |
| `value` | TEXT | Setting value |

## 4. Indexes and constraints

```sql
CREATE UNIQUE INDEX idx_sources_ext
ON sources(source_type, external_id);

CREATE UNIQUE INDEX idx_items_ext
ON items(source_id, external_id);

CREATE INDEX idx_items_source_date
ON items(source_id, published_at DESC);

CREATE INDEX idx_items_author
ON items(author);
```

Why they exist:
- `idx_sources_ext` prevents duplicate source registration;
- `idx_items_ext` will prevent duplicate message storage per source;
- `idx_items_source_date` supports future browsing by source and time;
- `idx_items_author` supports future author filtering.

## 5. Migrations

Migrations live in `src-tauri/migrations/` and are registered in `src-tauri/src/lib.rs`.

Current migration history:

| Version | File | Purpose |
| :--- | :--- | :--- |
| 1 | `1.sql` | Initialize `sources`, `items`, `app_settings` |
| 2 | `2.sql` | No-op; `is_member` was already present in migration 1 |
| 3 | `3.sql` | Add `accounts` and `sources.account_id` |

Rules:
- never delete or rename an existing migration file;
- never casually edit an already-applied migration;
- always add new schema work as a new migration file;
- if a historical migration must be repaired, update metadata before SQL plugin initialization.

## 6. Migration 2 compatibility note

`2.sql` is intentionally:

```sql
SELECT 1;
```

Reason:
- `is_member` was already included in `1.sql`;
- an earlier historical version tried to add it again;
- older local databases may therefore contain stale migration metadata.

The app repairs migration metadata before SQL plugin initialization so that existing local databases can still start cleanly.

## 7. Compression status

The schema reserves compressed fields (`metadata_zstd`, `content_zstd`, `raw_data_zstd`) and the backend already depends on `zstd`.

Current implementation status:
- schema support exists;
- message sync and compression write-path are not implemented yet.

## 8. Practical status

As of the current codebase:
- `accounts` and `sources` are live production tables for the UI;
- `items` is schema-ready but not yet used by application flows;
- the database path, preload, and migration handling are aligned with the running app.
