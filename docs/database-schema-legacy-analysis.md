# Database Schema Legacy Analysis

> Updated: 2026-05-17
> Scope: SQLite schema, migrations, Rust backend usage, and product docs.
> Method: local code/doc review plus three read-only subagent passes focused on schema, backend cost, and product direction.

## Executive Summary

The clearest schema debt is not one bad table. It is that the storage model is carrying several historical layers at once:

- `sources` has both old Telegram-specific identity and newer provider-neutral identity.
- `items` started as Telegram messages and became a polymorphic provider document table.
- many durable product decisions are stored in compressed JSON blobs, so backend code has to decode Rust structs for data that is now queried by ordinary features.
- analysis snapshots are the right product direction, but their nullable compatibility shape keeps old fallback paths alive.

The highest-return simplification was to make provider identity explicit and
typed. Source identity, typed Telegram message identity, typed YouTube source
metadata, and Telegram topic membership materialization have now shipped in
separate slices. The remaining high-return work is Takeout provenance, the
provider-neutral document layer, snapshot hardening, playlist simplification,
and migration baseline cleanup.

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

### 1. `telegram_source_kind` Compatibility Debt Has Been Contained

`source_subtype` is now the provider-local subtype column, and migration
`15.sql` backfills Telegram rows from `telegram_source_kind`. This was the
original debt:

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

Current state:

- `source_subtype` is canonical;
- current Telegram source identity uses
  `(account_id, source_type, source_subtype, external_id)`;
- operational Telegram peer identity lives in `telegram_sources`;
- old Telegram metadata blobs may still exist as repair input until an
  explicit cleanup slice clears them.

### 2. Telegram Peer Identity Was Buried In `metadata_zstd`

The backend used to normalize legacy source metadata fields such as `username`,
`added_from`, and `access_hash` from compressed source blobs into runtime peer
identity. That made source resolution more complex than it needed to be.

Current state:

- normal Telegram runtime paths use typed `telegram_sources`;
- legacy metadata decode remains a repair/compatibility path;
- dialog scanning can still be used when typed identity lacks enough stable
  peer data, especially for private sources.

Relevant files:

- `src-tauri/src/sources/peer_resolution.rs`
- `src-tauri/src/sources/store.rs`

Remaining follow-up:

- optionally clear old Telegram `sources.metadata_zstd` blobs after successful
  typed repair;
- validate private/dialog-backed source behavior on real accounts before
  tightening fallback behavior further.

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

Current state:

- Telegram message identity moved to `telegram_messages`;
- Telegram duplicate detection no longer uses only `(source_id, external_id)`;
- non-Telegram duplicate detection uses provider item kind with external id;
- high-value source metadata moved into typed Telegram and YouTube source
  tables;
- provider-neutral analysis/export documents remain open work.

Remaining follow-up:

- introduce a provider-neutral document layer for analysis/export text units;
- continue moving provider-specific hot-path fields out of generic `items`
  when a workflow needs indexed or validated state;
- keep raw compressed provider payloads outside the hot query path.

### 4. Takeout Provenance Still Blocks Migrated Telegram History

The typed Telegram message identity boundary can represent overlapping message
ids from different Telegram history domains. Migrated Takeout history is still
deferred because the app cannot yet explain, persist, and recover partial
Takeout imports as durable provenance.

Relevant files:

- `docs/database-schema.md`
- `src-tauri/src/sources/items.rs`
- `src-tauri/src/takeout_import/mod.rs`
- `src-tauri/src/topic_memberships.rs`

Recommendation:

- add durable ingest batches for Takeout runs;
- add Telegram Takeout batch details for split/fallback/migration detection;
- add item origin and observation rows so repeat runs can distinguish inserted
  rows from duplicates seen by a later import;
- enable migrated-history import only after real-data validation proves the
  provenance and typed identity model is safe.

### 5. Topic Membership Materialization Has Shipped

Forum topic membership used to be inferred with a multi-branch join:

- `reply_to_top_id = topic_id`;
- fallback from numeric `external_id` to `top_message_id`;
- fallback from `reply_to_msg_id` to `topic_id`;
- fallback to General topic;
- otherwise synthetic Unrecognized topic.

That logic has been moved into the shared topic membership resolver.
Source readers and NotebookLM export now read materialized
`item_topic_memberships` plus source-level `telegram_topic_resolution_state`.
`Unrecognized topic` remains a derived bucket rather than a stored topic row.

Relevant files:

- `src-tauri/src/forum_topics.rs`
- `src-tauri/src/sources/items/query.rs`
- `src-tauri/src/notebooklm_export/query.rs`

Remaining follow-up:

- keep the resolver as the only place that uses the legacy
  `items.external_id` integer fallback;
- decide whether Takeout should refresh the forum-topic catalog after a
  successful import;
- add richer topic/export UI only if product workflows need it.

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

Compressed blobs are useful for archival payloads, but too many historical
features depended on decoding them. Source runtime metadata has been moved out
of the normal hot path for Telegram and YouTube, but several compressed payloads
remain intentionally archival or diagnostic:

- legacy `sources.metadata_zstd` rows that may remain as repair input;
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

1. Source identity cleanup. Shipped.
   - Canonicalize `source_subtype`.
   - Move Telegram/YouTube identity into typed provider tables.
   - Remove normal-path dependence on `telegram_source_kind`.

2. Item/document identity cleanup. Partially shipped.
   - Provider-native Telegram item identity shipped.
   - Telegram duplicate detection moved away from only `(source_id, external_id)`.
   - Remaining work: durable Takeout provenance and a provider-neutral document
     layer for analysis/export.

3. Takeout provenance and migrated-history enablement.
   - Add durable ingest batches, Telegram Takeout batch details, and item
     origin/observation rows.
   - Distinguish complete, partial, failed, and cancelled imports in durable
     state.
   - Enable migrated-history import only after real-data validation.

4. Snapshot hardening.
   - Make new snapshot fields non-null.
   - Persist corpus snapshots before provider execution.
   - Backfill or explicitly mark old snapshotless runs.

5. Topic membership materialization. Shipped.
   - Repeated inference joins were replaced with `item_topic_memberships`.
   - Memberships are updated during migration, topic refresh, and scoped
     Telegram item insert when state is usable.

6. YouTube playlist simplification.
   - Make playlist entries point to stable video entities.
   - Collapse duplicated removal state.

7. Migration baseline cleanup.
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
