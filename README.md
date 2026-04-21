# Extractum

Extractum is a desktop application built with Tauri, SvelteKit, TypeScript, and Rust.
The current MVP focuses on collecting Telegram channel sources, storing local metadata in SQLite, and preparing the app architecture for message sync and later LLM analysis.

## Current status

Implemented today:
- multi-account Telegram setup;
- Telegram sign-in flow per account;
- session persistence per account in `app_data_dir`;
- local SQLite schema and migrations via `tauri-plugin-sql`;
- account management UI;
- source registration UI for Telegram channels;
- source discovery from Telegram dialogs or manual `@handle` / `t.me` input;
- persistent light/dark theme toggle, with light theme as default.

Not implemented yet:
- channel message synchronization into `items`;
- message browsing UI;
- LLM provider integration and analysis flow.

## Stack

- frontend: `SvelteKit` + `TypeScript`
- backend: `Tauri` + `Rust`
- database: `SQLite` through `tauri-plugin-sql`
- Telegram client: `grammers`
- planned content compression: `zstd`

## Architecture

Extractum follows a "fat frontend, thin backend" approach:
- frontend owns user flows, UI state, filtering, and orchestration;
- backend owns Telegram access, SQLite access, session persistence, migrations, and secret boundaries.

The frontend should call small Tauri commands instead of accessing low-level integration details directly.

## Project structure

- `src/routes/+layout.svelte`: app shell, navigation, theme toggle
- `src/routes/accounts/+page.svelte`: account management UI
- `src/routes/auth/[id]/+page.svelte`: Telegram auth flow for one account
- `src/routes/sources/+page.svelte`: source listing and Telegram channel import UI
- `src-tauri/src/lib.rs`: Tauri app bootstrap and command registration
- `src-tauri/src/telegram.rs`: Telegram client lifecycle and session persistence
- `src-tauri/src/sources.rs`: SQLite-backed account/source commands
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
