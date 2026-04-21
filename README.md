# Extractum

Extractum is a desktop application built with `Tauri`, `SvelteKit`, `TypeScript`, and `Rust`.
The current MVP focuses on collecting Telegram channel data into local `SQLite`, browsing synced messages, and now includes a working Gemini-backed analysis workspace over already-synced records.

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
- minimal Gemini settings and streaming test UI on `/settings`;
- dedicated `/analysis` workspace for saved markdown reports over synced messages;
- saved report history with immutable runs;
- traceability through clickable message refs and quote lookup;
- named source groups and multi-source report runs;
- report-grounded follow-up chat over saved runs and local synced messages;
- persisted chat history for analysis conversations;
- persistent light/dark theme toggle, with light theme as default.

Not implemented yet:
- background or scheduled sync;
- message edit/delete reconciliation;
- media ingestion;
- advanced filtering/search across synced items;
- full media download or preview rendering.

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
- `src/routes/settings/+page.svelte`: Gemini provider settings and streaming test UI
- `src/routes/analysis/+page.svelte`: saved report generation, traceability, source groups, and grounded chat UI
- `src-tauri/src/lib.rs`: Tauri app bootstrap and command registration
- `src-tauri/src/telegram.rs`: Telegram client lifecycle and session persistence
- `src-tauri/src/sources.rs`: SQLite-backed account/source/item commands
- `src-tauri/src/llm.rs`: provider abstraction, Gemini request mapping, and streaming events
- `src-tauri/src/analysis.rs`: analysis runs, templates, source groups, trace lookup, and grounded chat commands
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

Planned next sync extension:
- keep `content_zstd` as the text/caption field;
- add media-aware item metadata so media-only posts stop being dropped;
- keep analysis text-only for now by continuing to read only rows that have textual content;
- postpone file download, thumbnail storage, and media-aware analysis to a later slice.

## Current LLM behavior

The current LLM/analysis slice now includes:
- Gemini is the only implemented provider;
- backend exposes `get_llm_profiles`, `save_llm_profile`, and `ask_llm_stream`;
- responses stream back to the UI through the `llm://response` Tauri event;
- `/settings` is the provider configuration and transport test surface;
- `/analysis` is the first real product surface built on top of the provider layer;
- backend owns analysis retrieval from local `items`, chunking, map/reduce report generation, saved run persistence, trace data, and grounded chat context assembly;
- report runs stream through `analysis://run`;
- follow-up chat streams through `analysis://chat`.

Current analysis behavior:
- reports run only over already-synced local messages from SQLite;
- reports can target one source or a saved source group;
- reports are saved as immutable runs with provider/model/template metadata;
- report and chat citations use refs like `s12-m845`;
- refs are clickable in both report output and chat replies;
- the trace panel can resolve and display cited messages from the current run scope.

Temporary security note:
- the Gemini API key is currently stored in `app_settings` in local SQLite;
- the settings UI can read the saved key back for editing;
- this is an explicit temporary security debt and should later move to secure storage.

## Current runtime status behavior

Telegram account readiness is now tracked at runtime with explicit statuses:
- `not_initialized`
- `restoring`
- `ready`
- `reauth_required`
- `restore_failed`

On startup, the backend restores saved Telegram sessions in the background.
The `/accounts` and `/sources` pages now receive runtime status changes through Tauri events rather than simple polling.
