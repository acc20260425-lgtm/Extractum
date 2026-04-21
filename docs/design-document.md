# Design Document: Extractum MVP

## 1. Overview

Extractum is a desktop application for collecting information from Telegram channels into a local SQLite database.
The long-term product goal is a full flow from Telegram ingestion to in-app LLM analysis, but the current implemented slice is focused on account setup and source registration.

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

The project is currently between steps 3 and 4: account and source setup are implemented, while message sync and LLM analysis are still pending.

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

Implemented user actions:
- create account
- list accounts
- delete account
- initialize Telegram client for an account
- send login code
- sign in
- sign out

### 3.2 Sources

Authenticated accounts can register Telegram channels as sources in two ways:
- by loading the account's Telegram dialogs and selecting a channel;
- by entering a public channel reference manually (`@handle`, `t.me/...`, or `https://t.me/...`).

Each registered source is linked to the account that added it.

### 3.3 UI shell

The current UI includes:
- account management page
- per-account auth page
- source management page
- shared navigation
- persistent light/dark theme toggle, with light theme as the default

## 4. Planned MVP functionality

Still planned for MVP:
- `sync_channel` command
- message ingestion into `items`
- message browsing and filtering UI
- message detail view
- LLM provider abstraction
- first LLM provider integration
- prompt + context workflow

## 5. Architecture

Extractum follows a "fat frontend, thin backend" model.

### Frontend responsibilities

- routing and page state
- form handling
- user flow orchestration
- filtering and context assembly logic
- theme/UI presentation

### Backend responsibilities

- Telegram MTProto integration
- session persistence
- SQLite migrations
- shared DB pool access
- account/source commands
- future compression and LLM provider calls

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
- `sources` is fully used for registration/listing
- `items` exists for future sync, but is not yet populated by the app

## 7. Security boundaries

Security-sensitive work stays in the backend:
- Telegram session files
- API credentials
- future provider secrets
- DB access and migration handling

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
4. load Telegram channels for that account;
5. register Telegram channels as local sources.

The full MVP is successful once sync, browsing, and LLM analysis are also complete.
