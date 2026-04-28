# Architecture Deep Dive

## 1. Layer split

The repository is intentionally split into two strong responsibilities.

### Backend (`src-tauri/src`)

Owns:

- Telegram integration
- account runtime state and restore
- SQLite access
- migrations
- compression / decompression
- analysis orchestration
- typed Tauri command errors

### Frontend (`src/routes`, `src/lib`)

Owns:

- route-level workflow
- UI state
- optimistic interaction and feedback
- filtering and presentation
- error normalization for display

## 2. Telegram ingest flow

### 2.1 Account lifecycle

Accounts are stored locally and may restore their Telegram session on startup. The frontend observes runtime status and uses that to gate actions like sync.

### 2.2 Source resolution

Sources can be added:

- by username / `t.me` reference;
- from the current account's dialogs.

Persisted Telegram source metadata now stores an explicit `peer_identity` contract:

- `strategy = username` for public sources added by `@username` or `t.me/name`
- `strategy = dialog` for dialog-backed sources, including private channels / groups and numeric refs resolved from dialogs
- optional `username` for public fallback behavior
- optional `access_hash` for stable `channel` / `supergroup` peer reconstruction when Telegram exposes it

`resolve_source_peer` follows an explicit rules pipeline:

1. username strategy -> resolve stored username -> fallback dialog scan for compatibility
2. dialog strategy -> reconstruct from stored peer identity -> optional username fallback -> fallback dialog scan
3. empty / older metadata -> compatibility dialog scan only

Supported source refs are:

- `@username`
- `t.me/name`
- dialog-backed sources picked from the current account

Support boundaries by Telegram source kind:

- `channel`: public usernames are supported; private channels are expected to work best when added from dialogs so the app can persist `access_hash`
- `supergroup`: same contract as `channel`; stored peer identity is preferred for private sources
- `group`: legacy small groups remain dialog-dependent; the app does not treat access-hash-only identity as stable support for this kind

Supported Telegram source kinds are:

- `channel`
- `supergroup`
- `group`

### 2.3 Sync strategy

Sync operates per source:

- first sync uses a configurable policy window;
- later sync resumes incrementally;
- duplicate items are ignored by `(source_id, external_id)` uniqueness.

## 3. Item model

The current `items` model is intentionally richer than the current analysis corpus.

Stored dimensions include:

- text content when present;
- raw compressed payload;
- `content_kind`;
- `has_media`;
- `media_kind`;
- compressed media metadata.

This allows `/sources` to present a more faithful archive even though `/analysis` still stays text-first.

## 4. Analysis architecture

### 4.1 Report generation

The report flow:

1. resolve scope
2. load prompt template
3. load corpus
4. call the provider
5. persist result + trace data
6. persist frozen snapshot

### 4.2 Saved run semantics

The saved run model is snapshot-first for new runs.

Frozen snapshot storage solves three drift problems:

- corpus drift after later syncs;
- source-group membership drift;
- evidence drift during follow-up chat / trace resolution.

### 4.3 Legacy compatibility

Older runs without snapshot rows can still fall back to live tables. This keeps upgrades non-breaking while making new runs more stable.

## 5. Error boundary

The backend now exposes structured `AppError` values. The frontend normalizes them through `src/lib/app-error.ts`.

This is intentionally minimal: the app gets better UX than raw strings without introducing a large error framework.

## 6. Known architectural debt

- secrets still live in SQLite-backed settings;
- private peer resolution may still be fragile or expensive on large accounts because of dialog scans;
- the analysis layer has not yet become media-aware;
- Telegram session storage may still deserve a more robust long-term format.

## 7. Practical entry points

If you are changing ingest:

- `src-tauri/src/sources.rs`
- `src/routes/sources/+page.svelte`

If you are changing analysis:

- `src-tauri/src/analysis/`
- `src/routes/analysis/+page.svelte`

If you are changing app-wide failure behavior:

- `src-tauri/src/error.rs`
- `src/lib/app-error.ts`

If you are changing storage:

- `src-tauri/src/migrations.rs`
- `src-tauri/migrations/`
