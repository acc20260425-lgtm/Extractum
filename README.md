# Extractum

Extractum is a desktop-first source ingest and analysis workspace built with:

- `SvelteKit 2 + Svelte 5 + TypeScript`
- `Tauri 2 + Rust`
- `SQLite` via `tauri-plugin-sql`

The current product slice is a local-first MVP for:

- managing Telegram accounts and sessions;
- adding Telegram channels, supergroups, and groups as sources;
- adding YouTube videos and playlists as sources;
- syncing source history into local SQLite storage;
- syncing YouTube metadata, transcripts, and comments into local SQLite storage;
- importing Telegram Takeout history for existing sources;
- browsing synced items in the `/analysis` workspace;
- running provider-backed analysis reports in `/analysis`;
- asking follow-up questions against saved analysis runs.

## Current capabilities

### Accounts and auth

- multiple Telegram accounts can be stored locally;
- sessions are restored on startup when possible;
- `/accounts` and `/analysis` receive runtime account status updates through Tauri events;
- `/auth/[id]` supports `tg_init -> send code -> sign in -> logout`.

### Source ingest

- sources are stored in `sources`;
- Telegram sources carry a `telegram_source_kind` of `channel`, `supergroup`, or `group`;
- YouTube sources carry a provider-local `source_subtype` of `video` or `playlist`;
- Telegram source uniqueness is scoped by account, source type, kind, and external id;
- YouTube video and playlist uniqueness is scoped by source type, subtype, and external id;
- synced Telegram messages are stored in `items`;
- synced YouTube transcripts and comments are stored in `items` with provider item kinds;
- YouTube transcript timing is stored in `youtube_transcript_segments`;
- YouTube playlist membership is stored in `youtube_playlist_items`;
- new synced Telegram messages store minimal local context when Telegram exposes it:
  - reply target message id;
  - reply target peer kind/id;
  - thread/topic root message id;
  - aggregate reaction count;
- the first sync window is configurable:
  - `recent_messages(N)`
  - `recent_days(N)`
- subsequent syncs continue from `last_sync_state`.
- Takeout source import can fill older local history for an existing source without creating a second source record.

YouTube source support requires `yt-dlp` to be installed and available on `PATH`. Extractum does not download YouTube audio or video binaries in the MVP; it calls `yt-dlp` for metadata, captions, comments, and playlist entries only. Auth-gated YouTube content requires cookies configured in `/settings`. YouTube sync jobs are in memory in the MVP and are not resumed after app restart.

### Media-aware item metadata

Sync is now media-aware without downloading binary media files:

- text-only messages are stored;
- text + media messages are stored;
- media-only messages are also stored if they have useful media metadata;
- lightweight media metadata is persisted in `items.media_metadata_zstd`.

The analysis workspace can show:

- content kind (`text_only`, `text_with_media`, `media_only`);
- media badges;
- media summary / file name / mime type when available.

What is still not implemented:

- full media download;
- media preview rendering;
- media-aware analysis beyond the current text-first corpus.

### LLM provider profiles

The settings flow now manages reusable LLM provider profiles:

- multiple profiles can be stored locally, with one active profile used by default;
- each profile stores a `profile_id`, provider, default model, and provider-specific settings;
- Gemini and OpenAI-compatible providers share the same backend profile-resolution path;
- OpenAI-compatible profiles persist a configurable `base_url`, used both for model discovery and live requests;
- `/settings` can save a profile without activating it, save and activate it, and run a live provider smoke test against the currently edited form.

### Analysis

Analysis currently works on already-synced local data only.

- reports can be generated for a single source or a saved source group;
- YouTube reports can use transcript-only, transcript+description, or transcript+description+comments corpus modes;
- analysis and follow-up chat resolve the active LLM profile by default unless a workflow passes an explicit profile id;
- prompt templates are versioned and stored locally;
- source groups are stored locally;
- queued and running reports are surfaced in a dedicated Active Runs panel;
- saved runs default to global history and can also be narrowed back to the current scope;
- saved runs include result markdown, trace data, chat history, and a frozen corpus snapshot;
- follow-up chat for new runs reads the saved snapshot rather than the live `items` table.
- backend preflight rejects empty or oversized analysis scopes before creating a run.
- YouTube trace refs preserve timestamp evidence links for transcript segments.

This means saved runs are now intended to be stable artifacts rather than live views over changing data.

### Exporting sources for Google NotebookLM

The `/analysis` source workspace can export one synced Telegram source to NotebookLM-friendly Markdown.

- export reads from the local `sources` and `items` tables only;
- no live Telegram requests, LLM calls, link fetching, or binary media downloads happen during export;
- output is written under the selected folder as a generated `notebooklm_export_*` directory;
- `glossary.md` summarizes participants by stored author string;
- conversation files include source summary, chronology, per-message YAML metadata, plain text, detected `http://` / `https://` links, and stored media placeholders from `items.media_metadata_zstd`;
- when new Telegram context fields are present, export metadata includes local reply snippets, reply target ids, reply peer ids, thread ids, and reaction counts;
- reply snippets are resolved from local SQLite only and can point to messages outside the selected export period without adding those original messages to the export corpus;
- files are grouped by year when they fit the configured limits, fall back to month when needed, and split into numbered parts by word and byte limits.

Current limitations:

- export works only for data already synced into Extractum;
- existing rows synced before migration `13.sql` may not contain reply, reaction, or thread metadata;
- forward metadata and rich Telegram formatting metadata are not exported yet;
- media binaries are not downloaded, so media-only rows are represented only through lightweight stored metadata;
- URL titles and descriptions are not enriched in the MVP.

Privacy warning: Only export chats and channels you are authorized to access. Be careful with private data, personal information, and confidential conversations before uploading exports to third-party tools such as Google NotebookLM.

## Current constraints

- analysis remains text-first: media-only items are visible in the analysis workspace but are not yet part of the analysis corpus;
- YouTube support is metadata/text only and does not download audio or video binaries;
- YouTube source jobs are process-local and in memory; after app restart, active jobs are not resumed and the user can start a fresh sync;
- saved LLM API keys and Telegram `api_hash` values are stored in the OS secure credential store; Telegram session files remain app-data files, but their contents are encrypted with per-account session keys stored in OS secure storage;
- YouTube cookies, when enabled in Settings, are stored in OS secure storage and are written only to temporary backend cookie files for `yt-dlp`;
- peer resolution still falls back to dialog scanning when cached username metadata is insufficient.

## Architecture

The project follows a practical split:

- Rust backend owns Telegram access, YouTube `yt-dlp` orchestration, session restore, migrations, SQLite I/O, compression, secure storage, and analysis orchestration.
- Svelte frontend owns route flow, UI state, forms, filtering, and user-facing workflows.

The backend is intentionally thin in UI concerns, while the frontend is intentionally thin in infrastructure concerns.

## Important routes

- `/accounts`: create/delete accounts, inspect runtime status
- `/auth/[id]`: Telegram sign-in and logout
- `/sources`: lightweight compatibility route that points people to the main workspace
- `/settings`: manage reusable LLM provider profiles, active profile selection, model refresh, live provider smoke tests, and YouTube cookie/settings controls
- `/analysis`: source browsing and sync, reports, source groups, active runs, saved run history, follow-up chat, trace inspection

## Storage overview

Main tables:

- `accounts`
- `sources`
- `items`
- `app_settings`
- `analysis_prompt_templates`
- `analysis_runs`
- `analysis_source_groups`
- `analysis_source_group_members`
- `analysis_chat_messages`
- `analysis_run_messages`
- `youtube_playlist_items`
- `youtube_transcript_segments`

Recent schema additions:

- migration `9.sql`: media-aware item metadata
- migration `10.sql`: immutable saved run snapshots
- migration `11.sql`: Telegram source kind
- migration `12.sql`: account-scoped source uniqueness
- migration `13.sql`: Telegram reply/thread/reaction context metadata on `items`
- migration `14.sql`: local Telegram Forum Topics catalog
- migration `15.sql`: provider-local source subtype
- migration `16.sql`: YouTube source foundation, provider item kinds, playlist rows, transcript segments, YouTube analysis snapshots, source-group provider type, and YouTube settings defaults

## Error model

The Tauri backend now exposes typed application errors:

- `validation`
- `not_found`
- `auth`
- `network`
- `conflict`
- `internal`

The frontend normalizes these errors through `src/lib/app-error.ts` instead of relying on plain strings.

## Recommended reading

1. `docs/project.md`
2. `docs/architecture-deep-dive.md`
3. `docs/database-schema.md`
4. `docs/design-document.md`
5. `docs/backlog.md`
