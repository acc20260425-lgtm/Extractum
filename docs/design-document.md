# Design Document: Extractum MVP

## 1. Overview

Extractum is a desktop application for collecting information from Telegram channels into a local SQLite database.
The long-term product goal is a full flow from Telegram ingestion to in-app LLM analysis. The current implemented slice now covers account setup, source registration, manual channel sync, minimal message browsing, Gemini-backed provider/settings, and a working saved-analysis workspace over synced local records.

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
6. run saved LLM-backed analysis over synced records;
7. later ask follow-up questions against saved reports and synced records.

The project has now implemented both the initial local message browsing slice and the first dedicated `/analysis` flow over already-synced records.

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

Planned next sync extension:
- keep `content_zstd` as the text/caption field for compatibility;
- add media-aware item metadata so media-only Telegram posts stop being skipped;
- extend `/sources` item rendering to show media presence and lightweight metadata even before file download exists;
- keep the first analysis pass text-only by reading only rows that still carry textual content.

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

## 4. Current analysis flow

The current LLM analysis flow is built on top of already-synced local `items`.

### 4.1 Supported scenarios

The currently supported scenarios are:
- one selected source;
- one selected source group;
- one selected time period;
- one saved markdown report;
- streaming partial output during report generation;
- saved run history with model and prompt metadata;
- mandatory traceability to concrete synced messages;
- report-grounded follow-up chat over completed saved runs;
- persisted chat history per saved run.

The analysis UI lives on a dedicated `/analysis` route rather than being embedded into `/sources`.

### 4.2 Retrieval boundary

Analysis must use only already-synced local data from SQLite.
The first report flow should not call Telegram APIs directly and should not depend on live MTProto state after sync has completed.

The backend now owns:
- selecting `items` by source and period;
- chunking message corpora;
- map/reduce orchestration;
- report persistence;
- traceability data persistence.

The frontend now owns:
- choosing source, period, language, and prompt template;
- starting runs;
- displaying streaming output;
- browsing saved runs and trace results.

This remains a deliberate exception to the thinner `/settings` LLM test flow, because analysis retrieval and persistence are tightly coupled to local storage.

### 4.3 Output and traceability

The first output format should be normal markdown text.

Every meaningful conclusion in the report must be traceable back to synced messages through explicit refs, for example:
- `s12-m845`
- `s12-m846`

Reports should cite these refs inline, and the backend should persist a compact trace map that can later power expandable quotes in the UI.

Each saved run therefore persists:
- the final markdown result;
- source and period metadata;
- provider, model, and prompt version metadata;
- compressed trace data for cited refs only.

The app should not persist a full snapshot of all input messages for every run in this first slice.

### 4.4 Prompt model

Prompting should use two layers:
- a backend-owned builtin scaffold that enforces grounding, markdown output, and citation behavior;
- a user-editable prompt template body stored in SQLite and versioned over time.

This lets users customize report emphasis and structure without being able to accidentally remove grounding rules.

### 4.5 Planned execution pipeline

The implemented report pipeline is:
1. load synced `items` from SQLite for the selected source or source group and period;
2. assign stable refs to each message;
3. chunk the corpus by size;
4. run per-chunk map summaries through the LLM;
5. run a final reduce step to generate the markdown report;
6. extract cited refs from the result;
7. persist the run, result, and trace data.

The map stage may use structured intermediate output internally even though the user-facing result is markdown.

## 5. Storage additions

The analysis slice now adds:
- `analysis_prompt_templates`
- `analysis_runs`
- `analysis_source_groups`
- `analysis_source_group_members`
- `analysis_chat_messages`

Planned run storage should support immutable saved artifacts:
- completed runs stay fixed even if more source messages are synced later;
- rerunning the same period later is allowed to produce a different result if the local archive has changed.

## 6. Planned MVP functionality

Still planned after the current slice:
- richer browsing and filtering over stored items
- pagination or lazy loading for message history
- message detail view
- media-aware analysis

Not planned for this stage:
- background sync worker
- full media download pipeline
- message edit/delete reconciliation
 - vector retrieval / embeddings / RAG

## 7. Architecture

Extractum follows a "fat frontend, thin backend" model.

### Frontend responsibilities

- routing and page state
- form handling
- user flow orchestration
- source selection
- sync triggering
- rendering synced messages
- analysis form state and run orchestration
- rendering streaming report output and run history
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
- analysis retrieval from SQLite
- analysis chunking and map/reduce orchestration
- analysis run and trace persistence

The backend should stay small and integration-oriented rather than becoming a second application layer.
The current analysis pipeline is the one place where the backend intentionally becomes a slightly thicker orchestration boundary, because it owns the local corpus and saved run artifacts.

## 8. Planned media-aware sync extension

The next major product slice should improve Telegram ingestion fidelity without jumping directly to full media download.

### 8.1 Schema direction

The intended direction is to keep `content_zstd` as the text-bearing field and add lightweight media-aware columns to `items`, for example:
- `content_kind`
- `has_media`
- `media_kind`
- `media_metadata_zstd`

This should let the app represent:
- text-only posts
- text-plus-media posts
- media-only posts

without forcing a full normalized media table in the first step.

### 8.2 Backend ingestion direction

`sync_channel` should stop treating empty text as an automatic skip.

Instead, the backend should:
- extract text/caption content when present;
- extract lightweight media metadata when present;
- store the item if either text or supported media metadata exists;
- skip only messages that have neither usable text nor supported media metadata.

The first media-aware slice should not:
- download files;
- persist thumbnails/binary previews;
- add OCR or image understanding.

### 8.3 UI direction

The Sources message browser should become media-aware enough to:
- show that a synced post contained media;
- display the primary media kind such as photo, video, or document;
- show lightweight metadata such as file name or mime type when available;
- render a useful placeholder for media-only posts instead of pretending the item does not exist.

### 8.4 Analysis compatibility

The current report and chat pipeline should remain text-first for safety.

That means the first media-aware ingestion step should not change analysis semantics yet:
- `text_only` and `text_with_media` records may continue to participate in analysis because they still have textual content;
- `media_only` records should be excluded from the current analysis corpus;
- the analysis UI should explicitly note that media-only posts are not included yet.

This gives the project a safe sequence:
1. improve sync fidelity;
2. improve `/sources` browsing;
3. later revisit media-aware analysis once ingestion and UI metadata are stable.

## 9. Storage model

SQLite is the single local source of truth.

Current schema:
- `accounts`
- `sources`
- `items`
- `app_settings`
- `analysis_prompt_templates`
- `analysis_runs`
- `analysis_source_groups`
- `analysis_source_group_members`
- `analysis_chat_messages`

Current active data paths:
- `accounts` is fully used
- `sources` is fully used for registration/listing/sync cursor state
- `items` is now populated by manual sync and read by `get_items`
- `app_settings` is now also used for temporary LLM provider profile storage
- `analysis_prompt_templates` stores report prompt templates
- `analysis_runs` stores immutable saved report runs and compressed trace data
- `analysis_source_groups` and `analysis_source_group_members` store reusable named multi-source scopes
- `analysis_chat_messages` stores persisted grounded chat history per saved run

## 10. Security boundaries

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

## 11. MVP non-goals

Still explicitly out of scope:
- vector databases
- embeddings
- semantic search
- automatic RAG pipelines
- non-Telegram ingestion
- collaborative cloud sync

## 11. Current success criteria

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

The current implementation is now successful if a user can also:
11. open a dedicated `/analysis` route;
12. choose one synced source or one saved source group and a period;
13. generate a saved markdown report over local `items`;
14. see streaming partial output while that report is being generated;
15. reopen that saved run later and inspect its cited message refs;
16. manage prompt templates and reusable source groups from the same workspace;
17. ask follow-up questions over a completed run and inspect cited refs from chat replies.
