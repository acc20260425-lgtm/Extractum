# Design Document: Extractum (MVP v3)

## 1. Project Overview

**Extractum** is a desktop application (Windows/macOS/Linux) designed to collect, store, and analyze information from digital sources, starting with Telegram channels.

The MVP architecture is intentionally simple:
- Telegram data is collected through MTProto.
- All local data is stored in SQLite.
- Relevant records are selected from SQL storage and sent to a local or cloud LLM for analysis.
- No vector database, embedding pipeline, or semantic index is used in MVP.

## 2. Product Goals

The initial product goal is to provide a reliable desktop workflow for:
- adding Telegram sources;
- synchronizing messages into a local database;
- browsing and filtering collected records;
- sending selected SQL-derived context into an LLM;
- receiving analytical answers inside the app.

The MVP is focused on correctness, privacy, and a short end-to-end path from source ingestion to analysis.

## 3. Core Functional Requirements

### 3.1 Data Collection
- **Primary source:** Telegram channels.
- **Access method:** MTProto user client for maximum access to public and subscribed channels.
- **Sync model:** Manual sync first, with a path to background sync later.
- **Persistence:** Messages and source metadata are stored locally in SQLite.

### 3.2 Data Browsing
The user must be able to:
- view configured sources;
- open collected items for a source;
- filter items by source, date range, and other available metadata;
- inspect original message text and, if needed, related metadata.

### 3.3 LLM Analysis
The user must be able to:
- select a source, date range, or subset of messages;
- provide a free-form analysis prompt;
- send the resulting context to a chosen LLM provider;
- receive a response inside the application UI.

The context is formed from SQL-selected records, not from a vector retrieval layer.

### 3.4 Configuration
The application must support:
- provider selection for LLM analysis;
- local or cloud model configuration;
- secure handling of API credentials;
- basic application settings stored locally.

## 4. Non-Goals for MVP

The following are explicitly out of scope for MVP:
- vector databases;
- embeddings;
- semantic search;
- automatic RAG pipelines;
- multi-source ingestion beyond Telegram;
- advanced collaborative or cloud-sync features.

This scope control is important to keep the first version small and shippable.

## 5. Tech Stack

### 5.1 Backend
- **Tauri / Rust**
- `tauri-plugin-sql` for SQLite integration
- `grammers` for Telegram MTProto integration
- ZSTD compression library for compact local storage
- provider adapters for local and cloud LLMs.

### 5.2 Frontend
- **SvelteKit / TypeScript**
- UI component library such as `shadcn-svelte` or `Skeleton UI`
- `Lucide Icons`
- Tauri IPC commands for backend interaction.

## 6. Architectural Model

Extractum follows a **Fat Frontend, Thin Backend** approach:
- **Frontend:** orchestrates user flows, filtering, context selection, and LLM request preparation;
- **Backend:** handles MTProto, SQLite, ZSTD compression, secrets, and LLM provider calls.

The backend should remain a compact systems layer rather than a full business-logic engine.

## 7. Main Components

### 7.1 Backend Services
1. **Telegram Manager**  
   Handles Telegram authentication, session management, and channel synchronization.

2. **Database Manager**  
   Manages SQLite schema, migrations, inserts, and filtered selects.

3. **LLM Coordinator**  
   Routes requests to the selected provider and normalizes responses.

4. **Security Layer**  
   Handles secret storage, Telegram session persistence, and safe command boundaries.

### 7.2 Frontend Layers
1. **Dashboard**  
   Shows sources, sync state, and recent activity.

2. **Source Manager**  
   Lets the user add, remove, and synchronize Telegram channels.

3. **Message Browser**  
   Displays collected items with filtering and inspection tools.

4. **Analysis Lab**  
   Allows the user to select records, write prompts, and review LLM output.

5. **Settings**  
   Stores LLM provider configuration and app preferences.

## 8. Data Model

The storage model is based on SQLite as the only local database.

### 8.1 `sources`
Stores data sources such as Telegram channels:
- `source_type`
- `external_id`
- `title`
- `metadata`
- `last_sync_state` (`message_id` INTEGER)
- `is_active`
- `created_at`.

### 8.2 `items`
Stores collected content:
- `source_id`
- `external_id`
- `author`
- `published_at`
- `content_zstd`
- `raw_data_zstd`.

### 8.3 `app_settings`
Stores local application settings as key-value pairs.

### 8.4 Compression
Heavy text fields and raw API payloads are compressed with ZSTD before being written into SQLite BLOB fields, then decompressed on read.

## 9. Data Flow

### 9.1 Sync Flow
1. User initiates source synchronization.
2. Frontend calls a Tauri command such as `sync_channel`.
3. Backend fetches data through MTProto, managed by `TelegramGuard`.
4. Backend sends `sync_progress` events to frontend.
5. Backend compresses and stores normalized content in SQLite.

### 9.2 Retrieval Flow
1. User opens a source or applies filters.
2. Frontend requests records through a Tauri command such as `get_items`.
3. Backend performs SQL selection and decompresses stored content.
4. Frontend receives ready-to-display records.

### 9.3 Analysis Flow
1. User selects records or defines filters.
2. Frontend assembles an analysis context from SQL-derived items, applying context size limits.
3. Frontend calls `ask_llm`.
4. Backend forwards the request through `LLM Coordinator`.
5. The chosen provider (Gemini) returns the answer to the UI.

## 10. Security Considerations

Security-sensitive operations must stay in the backend:
- API key storage (`keyring` for Gemini);
- Telegram session storage;
- LLM provider credentials;
- input validation for IPC commands;
- avoidance of secret leakage through logs.

The frontend must not directly access secrets or low-level Telegram session data.

<h2>11. MVP Milestones</h2>

<h3>Phase 1: Foundations</h3>
<ul>
<li>Initialize Tauri + SvelteKit project</li>
<li>Connect SQLite with `tauri-plugin-sql` file-based migrations</li>
<li>Apply initial schema</li>
<li>Build minimal source and message UI.</li>
</ul>

<h3>Phase 2: Telegram Integration</h3>
<ul>
<li>Implement Telegram authentication using `grammers`</li>
<li>Add source registration</li>
<li>Implement first channel sync with progress events</li>
<li>Save messages into SQLite with ZSTD compression.</li>
</ul>

<h3>Phase 3: Browsing and Filtering</h3>
<ul>
<li>List sources</li>
<li>Show stored items</li>
<li>Add filtering by source and date</li>
<li>Add message detail view.</li>
</ul>

<h3>Phase 4: LLM Analysis</h3>
<ul>
<li>Implement `LLMProvider` abstraction</li>
<li>Add **Google Gemini** as the first working provider</li>
<li>Build prompt + context flow with frontend size limits</li>
<li>Render answer in Analysis Lab.</li>
</ul>

<h3>Phase 5: Polish</h3>
<ul>
<li>Improve UX</li>
<li>Improve error handling</li>
<li>Refine settings and secret management</li>
<li>Clean documentation.</li>
</ul>

<h2>12. Success Criteria for MVP</h2>

The MVP is successful if a user can:
<ol>
<li>add a Telegram source;</li>
<li>sync its messages into local SQLite storage;</li>
<li>browse and filter collected items;</li>
<li>select a subset of records;</li>
<li>send that context to a **Google Gemini LLM**;</li>
<li>receive a useful answer inside the app.</li>
</ol>

## 13. Known Limitations of MVP
- The application in the MVP version does not track editing or deleting messages that have already been loaded into the local database.
