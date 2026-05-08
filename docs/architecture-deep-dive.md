# Architecture Deep Dive

## 1. Layer split

The repository is intentionally split into two strong responsibilities.

### Backend (`src-tauri/src`)

Owns:

- Telegram integration
- account runtime state and restore
- SQLite access
- OS secure storage access for saved credentials
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

The shared source layer is provider-ready: source records expose
`source_type` and provider-local `source_subtype`, source UI actions are
capability-driven, and `sync_source` dispatches by provider. Telegram is still
the only implemented ingest provider.

### 2.1 Account lifecycle

Accounts are stored locally and may restore their Telegram session on startup. The frontend observes runtime status and uses that to gate actions like sync.

Account metadata stays in SQLite, while saved Telegram `api_hash` values live in
OS secure storage under `telegram.account.<account_id>.api_hash`. Legacy
non-empty `accounts.api_hash` values migrate lazily and are blanked only after a
successful secure-store write.

Telegram session files remain app-data files, but their contents are encrypted
with per-account session keys stored in OS secure storage under
`telegram.account.<account_id>.session_key`.

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

Unsupported manual private refs such as invite links and `t.me/c/...` links are rejected with guidance to re-add those sources from dialogs.

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
- newly inserted rows persist minimal Telegram context when available: reply target, reply target peer, thread/topic root id, and aggregate reaction count.

### 2.4 Takeout source import

Takeout import is a second source ingest path for already registered Telegram sources. It is not a replacement for `sync_source`: normal sync remains the fast incremental path, while Takeout import is the full-history path that uses Telegram Takeout wrappers.

The runtime shape is:

1. load the source/account runtime and resolve the source peer through the existing source-resolution path;
2. acquire the same-source ingest lock shared with sync and delete;
3. start `account.initTakeoutSession` without `InvokeWithTakeout`;
4. run validation, split loading, count probes, and history pages through `InvokeWithTakeout`;
5. wrap history requests in `InvokeWithMessagesRange`;
6. parse raw TL messages into the shared item insert helper;
7. finish the Takeout session before advancing `sources.last_sync_state`.

Takeout job state is in memory and is mirrored to the frontend through full-record `sources://takeout-import` events. The analysis workspace can start, cancel, and display a job's phase/progress/warnings.

The history loop is TDesktop-first, not just `add_offset = -100`. Each split starts with `largest_id_plus_one = 1`, reverses raw newest-to-oldest pages into oldest-to-newest order before persistence, and advances the cursor through the newest parsed message id plus one. A per-split `DescendingFallback` restarts only the current split if the TDesktop profile returns an empty first page despite a nonzero count or if the cursor does not advance.

Current source-kind behavior:

- `channel`: import the last split only;
- `supergroup`: import the last split only and warn if migrated small-group history is detected;
- `group`: import all selected split ranges;
- `CHANNEL_PRIVATE` on channel/supergroup history switches to `messages.search(from_id=self)` and records an only-my-messages warning.

Takeout import writes to the same `items` table and does not download media bytes, thumbnails, custom emoji documents, or Telegram Desktop export assets. Failed and cancelled jobs may leave partial rows, but they do not update `last_sync_state`.

## 3. Item model

The current `items` model is intentionally richer than the current analysis corpus.

Stored dimensions include:

- text content when present;
- raw compressed payload;
- `content_kind`;
- `has_media`;
- `media_kind`;
- compressed media metadata.
- nullable Telegram context metadata:
  - `reply_to_msg_id`;
  - `reply_to_peer_kind`;
  - `reply_to_peer_id`;
  - `reply_to_top_id`;
  - `reaction_count`.

This allows the main analysis workspace to present a more faithful archive even though analysis still stays text-first.

Context metadata is not backfilled. Older rows and rows where Telegram did not expose the relevant fields keep `NULL` values.

## 4. NotebookLM export architecture

NotebookLM export reads only local SQLite state. It does not call Telegram, LLM providers, link preview services, or media download paths.

For rows with `reply_to_msg_id`, export resolves original messages in batches from the same `source_id` by matching `items.external_id` to the Telegram reply message id. Original messages can be outside the selected period, but they are only used as YAML snippet metadata.

Exported message metadata can include:

- local reply id;
- local reply author/snippet;
- reply peer kind/id;
- thread id;
- aggregate reaction count.

## 5. Analysis architecture

### 5.1 Report generation

The report flow:

1. resolve scope
2. load prompt template
3. load corpus
4. call the provider
5. persist result + trace data
6. persist frozen snapshot

### 5.2 Saved run semantics

The saved run model is snapshot-first for new runs.

Frozen snapshot storage solves three drift problems:

- corpus drift after later syncs;
- source-group membership drift;
- evidence drift during follow-up chat / trace resolution.

### 5.3 Legacy compatibility

New live corpus refs use local item identity (`s{source_id}-i{item_id}`).
Legacy Telegram-shaped refs (`s{source_id}-m{message_id}`) are still accepted.
Older runs without snapshot rows can still fall back to live tables. This keeps
upgrades non-breaking while making new runs more stable.

## 6. LLM provider architecture

The `src-tauri/src/llm/` module is now profile-oriented.

Runtime resolution works like this:

1. load the requested profile id, or fall back to the active profile;
2. normalize provider-specific settings such as OpenAI-compatible `base_url`;
3. resolve the saved API key from OS secure storage when no temporary key was supplied;
4. resolve the effective model from the profile default plus any per-request override;
5. dispatch to the provider-specific runner.

Current provider behavior:

- Gemini uses the shared profile path with no `base_url`;
- OpenAI-compatible providers use the same profile path but require a configured `base_url` for both `/models` and `/chat/completions`.

The frontend `/settings` route mirrors that contract:

- it can select existing profiles or create new ones;
- it can save without activation or save and set active;
- it receives only `api_key_configured`, never saved secret values;
- it runs provider smoke tests only after saving the currently visible form, so the test uses the same profile state the user sees.

This keeps analysis runs, provider tests, and follow-up chat aligned on one backend profile-resolution model.

## 7. Error boundary

The backend now exposes structured `AppError` values. The frontend normalizes them through `src/lib/app-error.ts`.

This is intentionally minimal: the app gets better UX than raw strings without introducing a large error framework.

## 8. Known architectural debt

- private peer resolution may still be fragile or expensive on large accounts because of dialog scans;
- Takeout import still needs broader live validation across supergroups, groups, private/left sources, and shifted export DC behavior;
- migrated supergroup history is detected but not imported until the `(source_id, external_id)` collision policy is decided;
- concrete YouTube, RSS, and forum ingestion are not implemented yet despite
  the provider-ready source model;
- the analysis layer has not yet become media-aware;
- full Telegram Forum Topics and forward metadata are not modeled yet;
- Telegram session storage may still deserve a more robust long-term format.

## 9. Practical entry points

If you are changing ingest:

- `src-tauri/src/sources.rs`
- `src-tauri/src/source_ingest.rs`
- `src-tauri/src/takeout_import.rs`
- `src-tauri/src/takeout_import/raw_parse.rs`
- `src/routes/analysis/+page.svelte`
- `src/lib/components/analysis/`

Detailed Takeout ingest notes live in `docs/takeout-source-import.md`.

If you are only changing the legacy compatibility route:

- `src/routes/sources/+page.svelte`

If you are changing analysis:

- `src-tauri/src/analysis/`
- `src/routes/analysis/+page.svelte`

If you are changing LLM settings or provider behavior:

- `src-tauri/src/llm/`
- `src/routes/settings/+page.svelte`

If you are changing app-wide failure behavior:

- `src-tauri/src/error.rs`
- `src/lib/app-error.ts`

If you are changing storage:

- `src-tauri/src/migrations.rs`
- `src-tauri/migrations/`
