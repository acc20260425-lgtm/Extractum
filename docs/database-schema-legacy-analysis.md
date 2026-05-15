# Database Schema Legacy Analysis

> Updated: 2026-05-15
> Scope: SQLite schema, migrations, Rust backend usage, and product docs.
> Method: local code/doc review plus three read-only subagent passes focused on schema, backend cost, and product direction.

## Executive Summary

The clearest schema debt is not one bad table. It is that the storage model is carrying several historical layers at once:

- `sources` has both old Telegram-specific identity and newer provider-neutral identity.
- `items` started as Telegram messages and became a polymorphic provider document table.
- many durable product decisions are stored in compressed JSON blobs, so backend code has to decode Rust structs for data that is now queried by ordinary features.
- analysis snapshots are the right product direction, but their nullable compatibility shape keeps old fallback paths alive.

The highest-return simplification is to make provider identity explicit and typed. Start with `sources`, then item/document identity, then snapshots/topic membership/playlist linking.

## Inputs Reviewed

Primary files:

- `docs/database-schema.md`
- `docs/design-document.md`
- `docs/architecture-deep-dive.md`
- `docs/backlog.md`
- `src-tauri/migrations/*.sql`
- `src-tauri/src/sources/`
- `src-tauri/src/analysis/`
- `src-tauri/src/youtube/`
- `src-tauri/src/notebooklm_export/`
- `src-tauri/src/llm/profiles.rs`
- `src-tauri/src/telegram.rs`

Subagent checks:

- schema and migrations review;
- Rust backend complexity review;
- product/docs current-state review.

## Findings

### 1. `telegram_source_kind` Is Compatibility Debt

`source_subtype` is now the provider-local subtype column, and migration `15.sql` backfills Telegram rows from `telegram_source_kind`. However:

- Telegram source uniqueness still uses `(account_id, source_type, telegram_source_kind, external_id)`;
- source DTOs and sync targets still carry both fields;
- YouTube upserts insert `telegram_source_kind = ''` to satisfy the old `NOT NULL` shape;
- tests explicitly cover the legacy `NOT NULL` Telegram kind path.

Relevant files:

- `src-tauri/migrations/11.sql`
- `src-tauri/migrations/12.sql`
- `src-tauri/migrations/15.sql`
- `src-tauri/src/sources/types.rs`
- `src-tauri/src/sources/store.rs`
- `src-tauri/src/sources/peer_resolution.rs`

Recommendation:

- make `source_subtype` canonical;
- use a single account-scoped identity key for Telegram: `(account_id, source_type, source_subtype, external_id)`;
- remove `telegram_source_kind` from new code paths, or move Telegram-specific fields to a `telegram_sources` table;
- keep a temporary compatibility adapter only for old databases.

### 2. Telegram Peer Identity Is Buried In `metadata_zstd`

The current backend normalizes legacy source metadata fields such as `username`, `added_from`, and `access_hash` into `peer_identity`, then still carries fallback dialog scanning. This makes runtime source resolution more complex than it needs to be.

Relevant files:

- `src-tauri/src/sources/peer_resolution.rs`
- `src-tauri/src/sources/store.rs`

Recommendation:

- add typed Telegram identity storage, for example:
  - `source_id`
  - `peer_kind`
  - `peer_id`
  - `resolution_strategy`
  - `username`
  - `access_hash`
  - `avatar_cache_key`
- keep compressed source metadata only as an archival/debug payload;
- make fallback dialog scanning a legacy repair path, not the normal path for well-formed rows.

### 3. `items` Is Too Polymorphic

`items` began as Telegram messages and now also stores:

- Telegram messages;
- YouTube transcript items;
- YouTube comment items;
- text/media classification;
- raw provider payloads;
- media metadata;
- Telegram reply/thread/topic/reaction fields.

This pushes provider branching into analysis, source browsing, and NotebookLM export.

Relevant files:

- `src-tauri/migrations/1.sql`
- `src-tauri/migrations/9.sql`
- `src-tauri/migrations/13.sql`
- `src-tauri/migrations/16.sql`
- `src-tauri/src/sources/items.rs`
- `src-tauri/src/sources/items/query.rs`
- `src-tauri/src/analysis/corpus.rs`
- `src-tauri/src/notebooklm_export/query.rs`

Recommendation:

- at minimum, change uniqueness from `(source_id, external_id)` to `(source_id, item_kind, external_id)`;
- preferably introduce a provider-neutral document layer for analysis/export text units;
- move provider-specific fields into child tables such as `telegram_messages`, `youtube_transcripts`, `youtube_comments`, and `item_media`;
- keep raw compressed provider payloads outside the hot query path.

### 4. Item Identity Blocks Migrated Telegram History

The docs already call out that Takeout cannot safely import migrated supergroup history because old small-group ids may collide under `(source_id, external_id)`. Topic resolution also has to cast `external_id` as an integer, which makes a generic text id do provider-specific work.

Relevant files:

- `docs/database-schema.md`
- `src-tauri/src/sources/items.rs`
- `src-tauri/src/forum_topics.rs`

Recommendation:

- add provider-native identity fields for Telegram rows, such as `telegram_peer_kind`, `telegram_peer_id`, `telegram_message_id`, and possibly `telegram_migration_domain`;
- use those fields for duplicate detection and topic joins;
- keep `external_id` as a display/import compatibility value rather than the only durable identity.

### 5. Topic Membership Is Recomputed In Readers

Forum topic membership is inferred with a multi-branch join:

- `reply_to_top_id = topic_id`;
- fallback from numeric `external_id` to `top_message_id`;
- fallback from `reply_to_msg_id` to `topic_id`;
- fallback to General topic;
- otherwise synthetic Unrecognized topic.

That logic is shared by source readers and NotebookLM export.

Relevant files:

- `src-tauri/src/forum_topics.rs`
- `src-tauri/src/sources/items/query.rs`
- `src-tauri/src/notebooklm_export/query.rs`

Recommendation:

- materialize `item_topic_memberships(item_id, topic_id, match_kind, updated_at)`;
- update it during Telegram sync, Takeout import, and topic refresh;
- make readers/export use a simple indexed join.

### 6. Analysis Snapshots Are Correct, But Still Compatibility-Shaped

`analysis_run_messages` is not obsolete. It is core to stable saved runs. The issue is that it was introduced before YouTube metadata and provider fields, then extended later with nullable fields:

- `item_kind`
- `source_type`
- `source_subtype`
- `metadata_zstd`

That keeps `Option` fields in Rust models and preserves snapshotless/legacy fallback logic. One fallback path reconstructs corpus from live data and currently defaults to `TranscriptDescription` instead of using the run's stored YouTube corpus mode.

Relevant files:

- `src-tauri/migrations/10.sql`
- `src-tauri/migrations/16.sql`
- `src-tauri/migrations/17.sql`
- `src-tauri/src/analysis/models.rs`
- `src-tauri/src/analysis/store.rs`
- `src-tauri/src/analysis/corpus.rs`

Recommendation:

- make snapshot document fields non-null for new runs;
- persist the corpus snapshot before provider execution, so failed or cancelled runs can still expose the exact attempted corpus where appropriate;
- backfill old completed runs when possible;
- keep historical snapshotless runs readable as report artifacts, but avoid live-source reconstruction as a normal behavior path.

### 7. Compressed Metadata Blobs Are Overloaded

Compressed blobs are useful for archival payloads, but too many current features depend on decoding them:

- `sources.metadata_zstd` for Telegram and YouTube identity/display;
- `items.raw_data_zstd` for provider payloads;
- `items.media_metadata_zstd` for media display/export;
- playlist item metadata;
- transcript segment metadata;
- analysis snapshot metadata;
- trace blobs.

Relevant files:

- `docs/database-schema.md`
- `src-tauri/src/analysis/corpus.rs`
- `src-tauri/src/youtube/detail.rs`
- `src-tauri/src/youtube/jobs.rs`

Recommendation:

- normalize high-value fields that are queried, filtered, or needed for product workflows;
- keep compressed blobs as raw archive/debug envelopes;
- version any remaining metadata envelopes explicitly.

### 8. YouTube Playlist Linking Has Nullable Branches Everywhere

`youtube_playlist_items.video_source_id` can be null. That is correct for unavailable/unlinked entries, but it leaks into analysis, detail views, retry jobs, and skipped-row accounting. There is also duplicated removal state: `availability_status` includes `removed_from_playlist`, while `is_removed_from_playlist` stores the same lifecycle separately.

Relevant files:

- `src-tauri/migrations/16.sql`
- `src-tauri/src/youtube/playlist.rs`
- `src-tauri/src/youtube/detail.rs`
- `src-tauri/src/analysis/corpus.rs`

Recommendation:

- introduce stable YouTube video entities separate from syncable source rows, or always create a video source/entity with availability state;
- choose one removal representation, either status or boolean, not both;
- make playlist analysis/detail logic operate over one uniform entry model.

### 9. Legacy Secret Migration Still Lives In Runtime

SQLite no longer owns saved LLM API keys or Telegram API hashes, but runtime code still migrates:

- `accounts.api_hash`;
- `llm.profile.<profile_id>.api_key`;
- legacy plaintext Telegram session files.

This is safety-sensitive compatibility code, so it should not be casually removed, but it should be isolated from normal product logic.

Relevant files:

- `src-tauri/src/accounts.rs`
- `src-tauri/src/telegram.rs`
- `src-tauri/src/llm/profiles.rs`
- `src-tauri/src/telegram_session_store.rs`

Recommendation:

- move legacy secure-store migration into an explicit upgrade module;
- keep normal account/profile code assuming secure-store ownership;
- once a minimum supported upgrade window is chosen, remove SQLite secret columns from the fresh schema baseline.

### 10. Migration History Has Fresh-Install Scars

Several migrations are historical patch layers rather than a clean current schema:

- `2.sql` is effectively redundant because `is_member` already exists in `1.sql`;
- source identity indexes are dropped and recreated across migrations;
- `migrations.rs` contains checksum repair logic for changed migration SQL and line endings.

Relevant files:

- `src-tauri/migrations/1.sql`
- `src-tauri/migrations/2.sql`
- `src-tauri/src/migrations.rs`

Recommendation:

- introduce a current-schema baseline for fresh installs;
- keep old migrations only for upgrade from existing databases;
- quarantine checksum repair and old migration compatibility behind a legacy upgrade path.

## Not Legacy

These pieces should not be treated as removable debt right now:

- `analysis_run_messages`: core saved-run snapshot model.
- media metadata fields: aligned with current media-aware ingest, text-first analysis.
- same-provider source groups: current code and product direction intentionally reject mixed-provider groups.
- NotebookLM export follow-up ideas: future work, not schema cleanup by themselves.

## Recommended Refactor Sequence

1. Source identity cleanup.
   - Canonicalize `source_subtype`.
   - Move Telegram/YouTube identity into typed provider tables.
   - Remove normal-path dependence on `telegram_source_kind`.

2. Item/document identity cleanup.
   - Add provider-native item identity.
   - Change duplicate detection away from only `(source_id, external_id)`.
   - Introduce a provider-neutral document layer for analysis/export.

3. Snapshot hardening.
   - Make new snapshot fields non-null.
   - Persist corpus snapshots before provider execution.
   - Backfill or explicitly mark old snapshotless runs.

4. Topic membership materialization.
   - Replace repeated inference joins with `item_topic_memberships`.
   - Update memberships during sync, Takeout import, and topic refresh.

5. YouTube playlist simplification.
   - Make playlist entries point to stable video entities.
   - Collapse duplicated removal state.

6. Migration baseline cleanup.
   - Add a fresh current-schema baseline.
   - Move legacy upgrades and checksum repair into an isolated module.

## Expected Backend Payoff

- fewer provider-specific branches in generic source code;
- simpler source uniqueness and upsert logic;
- safer Takeout import for migrated Telegram history;
- less repeated metadata decoding;
- simpler analysis corpus loading;
- simpler NotebookLM export queries;
- clearer future provider integration boundaries.
