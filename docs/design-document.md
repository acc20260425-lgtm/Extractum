# Design Document: Extractum MVP

## 1. Overview

Extractum is a desktop application for collecting information from Telegram channels into a local SQLite database.
The long-term product goal is a full flow from Telegram ingestion to in-app LLM analysis. The current implemented slice now covers account setup, source registration, manual channel sync, minimal message browsing, and the first Gemini-backed LLM provider/settings layer.

The MVP architecture remains intentionally simple:
- Telegram access through MTProto;
- local SQLite as the only application database;
- SQL-first retrieval;
- no vector database, embeddings, or semantic retrieval layer.

## 2. Product goal

The intended end-to-end MVP flow is:
1. add one or more Telegram accounts;
2. authenticate them in the desktop app;
3. discover or manually add Telegram channel sources;
4. sync channel messages into local storage;
5. browse and filter stored records;
6. send selected SQL-derived context to an LLM.

The project has now implemented the first half of step 5 in a minimal form: synced messages can be viewed inline on the Sources page, but there is not yet a richer browsing, filtering, or analysis layer.

## 3. Current implemented functionality

### 3.1 Accounts

The application supports multiple Telegram accounts at the same time.
Each account has:
- `label`
- `api_id`
- `api_hash`
- optional `phone`
- its own active MTProto client in memory
- its own persisted session file on disk

At startup, the backend now attempts to restore clients automatically for accounts that already have a saved session on disk.
This restore runs in the background and exposes runtime readiness states to the UI.

Implemented user actions:
- create account
- list accounts
- delete account
- initialize Telegram client for an account
- send login code
- sign in
- sign out

Implemented runtime account states:
- `not_initialized`
- `restoring`
- `ready`
- `reauth_required`
- `restore_failed`

### 3.2 Sources

Authenticated accounts can register Telegram channels as sources in two ways:
- by loading the account's Telegram dialogs and selecting a channel;
- by entering a public channel reference manually (`@handle`, `t.me/...`, or `https://t.me/...`).

Each registered source is linked to the account that added it.
The source record also stores:
- `last_sync_state` as the current sync cursor;
- compressed metadata with channel username when available, so the backend can resolve the source later even if it is not found in current dialogs.

### 3.3 Sync and items

The first message sync slice is implemented as a manual per-source action.

Current sync behavior:
- the user triggers sync from `/sources`;
- the backend resolves the Telegram channel from the stored source;
- messages are loaded through `grammers`;
- only text/caption content is persisted in `items`;
- empty-text messages are skipped;
- duplicates are ignored by `(source_id, external_id)`;
- `sources.last_sync_state` is updated to the highest synced Telegram message id.

Stored item fields currently used:
- `source_id`
- `external_id`
- `author`
- `published_at`
- `ingested_at`
- `content_zstd`
- `raw_data_zstd`

### 3.4 UI shell

The current UI includes:
- account management page
- per-account auth page
- source management page
- source sync controls
- inline message browsing on the source page
- settings page for Gemini provider configuration and test calls
- runtime Telegram readiness badges on `/accounts` and `/sources`
- shared navigation
- persistent light/dark theme toggle, with light theme as the default

### 3.5 LLM provider settings and streaming

The first LLM abstraction slice is now implemented in a Gemini-first form.

Current behavior:
- the backend exposes generic chat-style input through `ask_llm_stream`;
- the frontend owns prompt/context assembly and sends generic `messages`;
- the backend resolves the active provider profile from `app_settings`;
- the first provider adapter is Gemini through the AI Studio API key flow;
- responses are streamed back through the `llm://response` Tauri event;
- `/settings` is the current UI surface for editing provider settings and running a test prompt.

Current provider profile model:
- one app-global active provider profile
- one currently used profile id: `default`
- provider: `gemini`
- editable `default_model`
- editable `api_key`

## 4. Planned MVP functionality

Still planned for MVP:
- richer browsing and filtering over stored items
- pagination or lazy loading for message history
- message detail view
- prompt + context workflow
- source-driven analysis flow that consumes synced records

Not planned for this stage:
- background sync worker
- media ingestion pipeline
- message edit/delete reconciliation

## 5. Architecture

Extractum follows a "fat frontend, thin backend" model.

### Frontend responsibilities

- routing and page state
- form handling
- user flow orchestration
- source selection
- sync triggering
- rendering synced messages
- prompt/context assembly for provider requests
- theme/UI presentation

### Backend responsibilities

- Telegram MTProto integration
- session persistence
- SQLite migrations
- shared DB pool access
- account/source/item commands
- ZSTD compression and decompression
- LLM provider profile resolution
- provider calls and streaming events

The backend should stay small and integration-oriented rather than becoming a second application layer.

## 6. Storage model

SQLite is the single local source of truth.

Current schema:
- `accounts`
- `sources`
- `items`
- `app_settings`

Current active data paths:
- `accounts` is fully used
- `sources` is fully used for registration/listing/sync cursor state
- `items` is now populated by manual sync and read by `get_items`
- `app_settings` is now also used for temporary LLM provider profile storage

## 7. Security boundaries

Security-sensitive work stays in the backend:
- Telegram session files
- API credentials
- DB access and migration handling
- compression/decompression of persisted payloads

Runtime note:
- account restore is backend-owned;
- the UI only observes restore state through `tg_get_account_statuses`;
- window startup is not blocked by session restore.

Temporary LLM exception:
- the Gemini `api_key` is currently stored in `app_settings` in SQLite;
- the settings UI can read that saved key back for editing;
- this is a deliberate temporary security debt to speed up the first provider slice;
- a later migration should move `api_key` into secure storage and restore the stricter backend-only secret boundary.

Future secret-storage note:
- if `api_hash` moves from SQLite into secure storage, secret keys must be profile-scoped;
- different app variants or profiles such as `test` and `work` must not share the same secret namespace by accident.
- recommended app identity scheme:
  - `org.ai.extractum` for stable
  - `org.ai.extractum.dev` for dev
  - `org.ai.extractum.test` for test
  - `org.ai.extractum.beta` for beta if that channel appears later
- secure storage service names should follow the same identity split so app variants do not collide.

The frontend should use Tauri commands and should not directly own low-level persistence or Telegram details.

## 8. MVP non-goals

Still explicitly out of scope:
- vector databases
- embeddings
- semantic search
- automatic RAG pipelines
- non-Telegram ingestion
- collaborative cloud sync

## 9. Current success criteria

The current implementation is successful if a user can:
1. create a Telegram account entry;
2. authenticate that account in the app;
3. persist the Telegram session;
4. restart the app and have that session restore automatically;
5. load Telegram channels for that account;
6. register Telegram channels as local sources;
7. sync a source into `items`;
8. view stored text messages in the app;
9. configure a Gemini model and API key in `/settings`;
10. run a streaming Gemini test request from the app.

The full MVP is successful once richer browsing/filtering and LLM analysis are also complete.
