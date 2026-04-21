# Extractum

Extractum is a desktop application built with `Tauri`, `SvelteKit`, `TypeScript`, and `Rust`.
The current MVP focuses on collecting Telegram channel data into local `SQLite`, browsing synced messages, and keeping the architecture ready for a later LLM analysis layer.

## Current status

Implemented today:
- multi-account Telegram setup;
- Telegram sign-in flow per account;
- session persistence per account in `app_data_dir`;
- automatic Telegram client restore on app startup when a saved session exists;
- local SQLite schema and migrations via `tauri-plugin-sql`;
- account management UI;
- source registration UI for Telegram channels;
- source discovery from Telegram dialogs or manual `@handle` / `t.me` input;
- manual channel sync into `items`;
- inline message browsing on the Sources page;
- persistent light/dark theme toggle, with light theme as default.

Not implemented yet:
- background or scheduled sync;
- message edit/delete reconciliation;
- media ingestion;
- advanced filtering/search across synced items;
- LLM provider integration and analysis flow.

## Stack

- frontend: `SvelteKit` + `TypeScript`
- backend: `Tauri` + `Rust`
- database: `SQLite` through `tauri-plugin-sql`
- Telegram client: `grammers`
- compression: `zstd`

## Architecture

Extractum follows a "fat frontend, thin backend" approach:
- frontend owns user flows, UI state, filtering, and orchestration;
- backend owns Telegram access, SQLite access, compression, session persistence, migrations, and secret boundaries.

The frontend should call small Tauri commands instead of accessing low-level integration details directly.

## Project structure

- `src/routes/+layout.svelte`: app shell, navigation, theme toggle
- `src/routes/accounts/+page.svelte`: account management UI
- `src/routes/auth/[id]/+page.svelte`: Telegram auth flow for one account
- `src/routes/sources/+page.svelte`: source listing, sync, and inline message browsing UI
- `src-tauri/src/lib.rs`: Tauri app bootstrap and command registration
- `src-tauri/src/telegram.rs`: Telegram client lifecycle and session persistence
- `src-tauri/src/sources.rs`: SQLite-backed account/source/item commands
- `src-tauri/migrations/*.sql`: schema migrations
- `GEMINI.md`: project rules and implementation constraints for AI agents

## Local development

From the repo root:

```powershell
npm install
npm.cmd run tauri dev
```

Useful checks:

```powershell
cd src-tauri
cargo check
```

```powershell
npm.cmd run check
```

Note: in some locked-down Windows environments, `npm` may need to be invoked as `npm.cmd`, and Vite/esbuild may hit local `EPERM` policy issues unrelated to project code.

## Database notes

- the app database is `extractum.db`;
- `tauri-plugin-sql` preloads the database at Rust startup via `tauri.conf.json`;
- Rust commands access the same pooled connection through `DbInstances`;
- migration `2.sql` is intentionally a no-op because `is_member` already exists in migration 1;
- migration metadata is patched before plugin initialization to keep older local databases compatible.

## Current sync behavior

The first sync slice is intentionally minimal:
- sync is manual and per source;
- only already-registered Telegram sources are syncable;
- only text/caption content is stored in `content_zstd`;
- empty-text messages are skipped;
- duplicates are ignored by `(source_id, external_id)`;
- `sources.last_sync_state` stores the highest synced Telegram message id;
- raw debug payload is stored in `raw_data_zstd`;
- messages are currently viewed inline on `/sources`.

## Current runtime status behavior

Telegram account readiness is now tracked at runtime with explicit statuses:
- `not_initialized`
- `restoring`
- `ready`
- `reauth_required`
- `restore_failed`

On startup, the backend restores saved Telegram sessions in the background.
The `/accounts` and `/sources` pages poll these runtime statuses so the UI can reflect restore progress without blocking window startup.
