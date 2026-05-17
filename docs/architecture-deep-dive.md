# Architecture Deep Dive

## 1. Layer split

The repository is intentionally split into two strong responsibilities.

### Backend (`src-tauri/src`)

Owns:

- Telegram integration
- YouTube integration through `yt-dlp`
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
capability-driven, and provider-specific commands dispatch by provider.

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

Telegram operational identity lives in `telegram_sources`; generic provider
identity lives in `sources`. Runtime source flows use canonical
`source_subtype` and typed Telegram peer identity. Legacy metadata is decoded
only during startup repair.

Typed Telegram source identity stores an explicit peer contract:

- `strategy = username` for public sources added by `@username` or `t.me/name`
- `strategy = dialog` for dialog-backed sources, including private channels / groups and numeric refs resolved from dialogs
- optional `username` for public fallback behavior
- optional `access_hash` for stable `channel` / `supergroup` peer reconstruction when Telegram exposes it

`resolve_source_peer` follows an explicit rules pipeline over typed identity:

1. stored `PeerRef` when peer kind, subtype, and access hash allow it;
2. stored normalized username when available;
3. account dialog scan for dialog-backed or username-less sources.

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

Telegram source subtype is canonical in `sources.source_subtype`. Operational
Telegram peer identity lives in `telegram_sources`, including `peer_kind`,
`peer_id`, username/access-hash hints, and avatar cache keys. The former
Telegram subtype compatibility mirror in `sources` was removed from the current
schema by the source identity legacy cleanup slice.

### 2.3 Sync strategy

Sync operates per source:

- first sync uses a configurable policy window;
- later sync resumes incrementally;
- Telegram duplicate items are ignored by typed native identity in
  `telegram_messages`;
- non-Telegram item upserts keep provider-specific external-id uniqueness;
- newly inserted Telegram rows persist minimal Telegram context when available:
  reply target, reply target peer, thread/topic root id, and aggregate
  reaction count.

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

There is no durable Takeout provenance table yet. In-memory job records explain
the active import, while completed database rows remain ordinary source items.
The next storage slice should add durable ingest batches, Telegram Takeout
batch details, and item origin/observation rows before migrated-history import
is enabled.

## 3. YouTube ingest flow

The `src-tauri/src/youtube/` module owns YouTube preview, source creation, metadata sync, transcript sync, comment sync, playlist membership sync, settings, cookies, runtime status, and read-only detail DTOs.

### 3.1 yt-dlp boundary

YouTube integration shells out to `yt-dlp`; Extractum does not embed a YouTube API client and does not download audio or video binaries in the MVP.

The boundary is intentionally narrow:

- preview and metadata commands use JSON output from `yt-dlp`;
- watch URLs with both `v=` and `list=` are treated as the selected video; canonical playlist URLs are used when the user wants the playlist itself;
- captions request transcript files and parse them into one `youtube_transcript` item plus `youtube_transcript_segments`;
- comments request bounded comment JSON and store `youtube_comment` items;
- playlist metadata is flattened into `youtube_playlist_items`;
- the runtime check command runs `yt-dlp --version` with a short timeout so the UI can show a missing-`yt-dlp` reason before starting a job.

Auth-gated content uses cookies from OS secure storage. When enabled in Settings, raw cookies are validated as Netscape cookie text, stored through `SecretStoreState`, and written only to a temporary backend file for the lifetime of the `yt-dlp` process. Cookies are not returned through IPC and should not appear in command args, logs, job records, or errors.

### 3.2 Source jobs

YouTube sync jobs are represented by `SourceJobState` in memory and emitted through `sources://source-job` events. Jobs cover video metadata, transcript, comments, full video sync, playlist metadata, playlist full sync, and single playlist video sync.

MVP restart behavior is explicit: active YouTube jobs are not restored after app restart, no attempt is made to resume an interrupted `yt-dlp` process, and the user can start a fresh sync after restart. Completed database writes from before shutdown remain visible.

Cancellation is cooperative around provider calls. If a cancel request races with a successful provider finish, `finish_job` preserves `cancelled` as the terminal state so the UI does not get stuck on a stale pending job.

### 3.3 Playlist expansion

Playlist source rows store typed runtime metadata in
`youtube_playlist_sources`; membership rows live in `youtube_playlist_items`.
Direct video source runtime metadata lives in `youtube_video_sources`.

Available playlist entries can link to materialized video sources through `video_source_id`. Unlinked, removed, private, auth-gated, age-restricted, geo-blocked, deleted, or unknown-unavailable rows remain visible in playlist detail but are excluded from the analysis corpus unless they become linked video sources later.

Analysis over a YouTube playlist expands linked `video_source_id` rows, then loads transcript segments, optional synthetic descriptions, and optional comments based on the selected YouTube corpus mode.

### 3.4 Timestamp evidence and detail commands

YouTube transcript segments preserve `start_ms`, optional `end_ms`, selected caption language, track kind, and auto-caption flag. Analysis trace refs can resolve segment evidence into YouTube URLs with timestamp parameters.

Read-only detail commands provide the analysis workspace with provider-aware state without introducing new persistence:

- `get_youtube_runtime_status`
- `list_youtube_source_summaries`
- `get_youtube_video_detail`
- `get_youtube_playlist_detail`

These commands aggregate status from `sources`, `items`, `youtube_transcript_segments`, `youtube_playlist_items`, and in-memory source jobs.

## 4. Item model

The current `items` model is intentionally richer than the current analysis corpus.

Stored dimensions include:

- text content when present;
- raw compressed payload;
- `content_kind`;
- `has_media`;
- `media_kind`;
- compressed media metadata.
- provider item kind (`telegram_message`, `youtube_transcript`, or `youtube_comment`);
- nullable Telegram context metadata:
  - `reply_to_msg_id`;
  - `reply_to_peer_kind`;
  - `reply_to_peer_id`;
  - `reply_to_top_id`;
  - `reaction_count`.

This allows the main analysis workspace to present a more faithful archive even though analysis still stays text-first.

Context metadata is not backfilled. Older rows and rows where Telegram did not expose the relevant fields keep `NULL` values.

YouTube transcript segment rows are not duplicated into `items.content_zstd`; the transcript item stores the selected transcript text, while `youtube_transcript_segments` stores timestamped evidence for analysis and trace links.

## 5. NotebookLM export architecture

NotebookLM export reads only local SQLite state. It does not call Telegram, LLM providers, link preview services, or media download paths.

For rows with `reply_to_msg_id`, export resolves original messages in batches from the same `source_id` by matching `items.external_id` to the Telegram reply message id. Original messages can be outside the selected period, but they are only used as YAML snippet metadata.

Exported message metadata can include:

- local reply id;
- local reply author/snippet;
- reply peer kind/id;
- thread id;
- aggregate reaction count.

For Telegram forum sources, export and source browsing read real topic
membership from `item_topic_memberships`. The source-level
`telegram_topic_resolution_state` row decides whether missing membership rows
can be surfaced as the derived `Unrecognized topic` bucket.

## 6. Analysis architecture

### 6.1 Report generation

The report flow:

1. resolve scope
2. load prompt template
3. run backend preflight for message count, estimated chunks, estimated input characters, and request caps
4. create the run only if preflight passes
5. load corpus
6. call the provider
7. stream output and live chunk summaries to the workspace
8. persist result + trace data
9. persist frozen snapshot

### 6.2 Saved run semantics

The saved run model is snapshot-first for new runs.

Frozen snapshot storage solves three drift problems:

- corpus drift after later syncs;
- source-group membership drift;
- evidence drift during follow-up chat / trace resolution.

### 6.3 Legacy compatibility

New live corpus refs use local item identity (`s{source_id}-i{item_id}`).
Legacy Telegram-shaped refs (`s{source_id}-m{message_id}`) are still accepted.
Older completed runs without snapshot rows remain openable as report artifacts,
but snapshot-bound source resolution, evidence, and follow-up chat degrade
explicitly instead of silently reading live tables. When live browsing is
offered for terminal or active runs, the UI labels it as live source context
rather than the frozen run corpus.

YouTube corpus loading adds timestamp-aware refs for transcript segments and synthetic refs for description text. Saved run snapshots preserve YouTube item kind, source type/subtype, and metadata needed for trace resolution after the live source changes.

Chunk summaries are live companion state for the opened running run. They are
not part of the saved run snapshot and terminal runs show an explicit empty
state when no live summaries remain in memory.

## 7. LLM provider architecture

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

LLM scheduling allows two running requests per `(provider, profile)` and prioritizes interactive requests over background work. Analysis report runs run a backend preflight before run creation and are capped at `10_000` messages, `80` estimated chunks, `1_500_000` estimated input characters, and `80` background requests.

## 8. Error boundary

The backend now exposes structured `AppError` values. The frontend normalizes them through `src/lib/app-error.ts`.

This is intentionally minimal: the app gets better UX than raw strings without introducing a large error framework.

## 9. Known architectural debt

- private peer resolution may still be fragile or expensive on large accounts because of dialog scans;
- Takeout import still needs broader live validation across supergroups, groups, private/left sources, and shifted export DC behavior;
- migrated supergroup history is detected but not imported until durable
  Takeout provenance and real-data validation are designed;
- RSS and forum ingestion are not implemented yet despite the provider-ready source model;
- YouTube needs broader live validation for active livestreams, upcoming videos, auto-caption-only videos, no-caption videos, private/member/age/geo-gated content, and large playlists;
- YouTube jobs are not persistent or resumable across app restart;
- YouTube-specific NotebookLM export enrichment is not implemented yet;
- the analysis layer has not yet become media-aware;
- richer Telegram Forum Topics browsing/export and forward metadata are not
  modeled yet;
- Telegram session storage may still deserve a more robust long-term format.

## 10. Practical entry points

If you are changing ingest:

- `src-tauri/src/sources.rs`
- `src-tauri/src/source_ingest.rs`
- `src-tauri/src/youtube/`
- `src-tauri/src/takeout_import.rs`
- `src-tauri/src/takeout_import/raw_parse.rs`
- `src/routes/analysis/+page.svelte`
- `src/lib/components/analysis/`

Detailed Takeout ingest notes live in `docs/takeout-source-import.md`.

If you are only changing the legacy `/sources` redirect:

- `src/routes/sources/+page.svelte`

If you are changing analysis:

- `src-tauri/src/analysis/`
- `src/routes/analysis/+page.svelte`

If you are changing YouTube runtime, sync, auth, or detail UI:

- `src-tauri/src/youtube/`
- `src/lib/api/youtube-detail.ts`
- `src/lib/api/source-jobs.ts`
- `src/lib/components/analysis/youtube-source-detail.svelte`
- `src/lib/components/analysis/youtube-playlist-detail.svelte`
- `src/routes/settings/+page.svelte`

If you are changing LLM settings or provider behavior:

- `src-tauri/src/llm/`
- `src/routes/settings/+page.svelte`

If you are changing app-wide failure behavior:

- `src-tauri/src/error.rs`
- `src/lib/app-error.ts`

If you are changing storage:

- `src-tauri/src/migrations.rs`
- `src-tauri/migrations/`
