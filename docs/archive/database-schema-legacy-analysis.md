# Database Schema Legacy Analysis

> Updated: 2026-05-20
> Scope: SQLite schema, migrations, Rust backend usage, and product docs.
> Method: local code/doc review plus three read-only subagent passes focused on schema, backend cost, and product direction.
> Status: archived historical analysis. Current schema state lives in
> `docs/database-schema.md`; pre-reset migration files are archived under
> `docs/archive/migrations-pre-baseline-reset/`.

## Executive Summary

The clearest schema debt is not one bad table. It is that the storage model is carrying several historical layers at once:

- `sources` has both old Telegram-specific identity and newer provider-neutral identity.
- `items` started as Telegram messages and became a polymorphic provider document table.
- many durable product decisions are stored in compressed JSON blobs, so backend code has to decode Rust structs for data that is now queried by ordinary features.
- analysis snapshots are the right product direction, but their nullable compatibility shape keeps old fallback paths alive.

The highest-return simplification was to make provider identity explicit and
typed. Source identity, typed Telegram message identity, typed YouTube source
metadata, and Telegram topic membership materialization have now shipped in
separate slices. Provider-neutral analysis documents v1 has also shipped for
live corpus loading. Analysis snapshot hardening has also shipped. The
archive read-model source browsing, Telegram NotebookLM export, and migration
baseline cleanup have also shipped. The remaining high-return work is narrower:
playlist simplification, legacy Telegram metadata blob cleanup after real-data
validation, and durable migrated-history enablement.

## Inputs Reviewed

Primary files:

- `docs/database-schema.md`
- `docs/design-document.md`
- `docs/architecture-deep-dive.md`
- `docs/backlog.md`
- `src-tauri/migrations/0001_current_schema_baseline.sql`
- `docs/archive/migrations-pre-baseline-reset/sql/*.sql`
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

`source_subtype` is now the provider-local subtype column. A historical
pre-baseline migration backfilled Telegram rows from `telegram_source_kind`;
that compatibility layer is archived. This was the original debt:

- Telegram source uniqueness used `(account_id, source_type, telegram_source_kind, external_id)`;
- source DTOs and sync targets carried both fields;
- YouTube upserts inserted `telegram_source_kind = ''` to satisfy the old `NOT NULL` shape;
- tests explicitly covered the legacy `NOT NULL` Telegram kind path.

Relevant files:

- `docs/archive/migrations-pre-baseline-reset/sql/11.sql`
- `docs/archive/migrations-pre-baseline-reset/sql/12.sql`
- `docs/archive/migrations-pre-baseline-reset/sql/15.sql`
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

- `docs/archive/migrations-pre-baseline-reset/sql/1.sql`
- `docs/archive/migrations-pre-baseline-reset/sql/9.sql`
- `docs/archive/migrations-pre-baseline-reset/sql/13.sql`
- `docs/archive/migrations-pre-baseline-reset/sql/16.sql`
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
- `analysis_documents` now materializes provider-neutral live analysis corpus
  documents for Telegram messages, YouTube transcript segments, YouTube
  comments, and synthetic YouTube descriptions;
- source browsing and Telegram NotebookLM export are readiness-gated archive
  read-model consumers, with fallback to canonical provider/archive paths when
  source archive rows are not current;
- YouTube NotebookLM export enrichment remains a later slice.

Remaining follow-up:

- decide whether YouTube-specific NotebookLM export enrichment needs archive
  rows, typed YouTube detail readers, or a small export-specific adapter;
- continue moving provider-specific hot-path fields out of generic `items`
  when a workflow needs indexed or validated state;
- keep raw compressed provider payloads outside the hot query path.

### 4. Takeout Provenance Still Blocks Migrated Telegram History Enablement

The typed Telegram message identity boundary can represent overlapping message
ids from different Telegram history domains. Migrated Takeout history is still
deferred because the app cannot yet explain, persist, and recover partial
Takeout imports as durable provenance.

Relevant files:

- `docs/database-schema.md`
- `src-tauri/src/sources/items.rs`
- `src-tauri/src/takeout_import/mod.rs`
- `src-tauri/src/topic_memberships.rs`

Current state:

- durable ingest batch, Telegram Takeout batch detail, warning, and item
  observation tables have shipped;
- normal Takeout import can persist partial item rows without advancing source
  sync state on failed or cancelled jobs;
- migrated-history import remains disabled until provenance behavior and typed
  identity boundaries are validated on real data.

Remaining follow-up:

- broaden live Takeout validation across representative source kinds and export
  edge cases;
- finish the incomplete-import policy so partial, failed, and cancelled imports
  are explainable and recoverable;
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

### 6. Analysis Snapshot Hardening Has Shipped

`analysis_run_messages` is not obsolete. It is core to stable saved runs. The issue is that it was introduced before YouTube metadata and provider fields, then extended later with nullable fields:

- `item_kind`
- `source_type`
- `source_subtype`
- `metadata_zstd`

Analysis snapshot hardening is shipped: new report runs capture
`analysis_run_messages` before provider execution and expose explicit snapshot
state on saved-run DTOs. Historical completed rows without saved messages remain
readable as missing legacy snapshots, but evidence, chat, and saved-run source
context do not reconstruct them from live provider/archive state.

Relevant files:

- `docs/archive/migrations-pre-baseline-reset/sql/10.sql`
- `docs/archive/migrations-pre-baseline-reset/sql/16.sql`
- `docs/archive/migrations-pre-baseline-reset/sql/17.sql`
- `src-tauri/src/analysis/models.rs`
- `src-tauri/src/analysis/store.rs`
- `src-tauri/src/analysis/corpus.rs`

Remaining follow-up:

- make snapshot document fields non-null for new runs;
- backfill old completed runs when possible;
- add UI affordances for missing legacy and capture-failed saved-run states.

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

- `docs/archive/migrations-pre-baseline-reset/sql/16.sql`
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

### 10. Migration History Scars Were Reset

Several migrations are historical patch layers rather than a clean current schema:

- `2.sql` is effectively redundant because `is_member` already exists in `1.sql`;
- source identity indexes are dropped and recreated across migrations;
- `migrations.rs` contains checksum repair logic for changed migration SQL and line endings.

Relevant files:

- `docs/archive/migrations-pre-baseline-reset/sql/1.sql`
- `docs/archive/migrations-pre-baseline-reset/sql/2.sql`
- `src-tauri/src/migrations.rs`

Current state:

- active migration history starts at
  `src-tauri/migrations/0001_current_schema_baseline.sql`;
- historical SQL and runner-managed Rust migrations are archived under
  `docs/archive/migrations-pre-baseline-reset/`;
- the one controlled pre-reset database uses a backup-first bookkeeping cutover
  that rewrites only `_sqlx_migrations`;
- future migrations start at `0002`.

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

2. Item/document identity cleanup. Mostly shipped.
   - Provider-native Telegram item identity shipped.
   - Telegram duplicate detection moved away from only `(source_id, external_id)`.
   - Provider-neutral analysis documents v1 shipped for live corpus loading.
   - Provider-neutral archive read model shipped for source browsing and
     Telegram NotebookLM export behind readiness gates.
   - Remaining work: decide whether YouTube-specific export enrichment needs
     more archive rows or typed YouTube detail readers.

3. Takeout provenance and migrated-history enablement.
   - Durable ingest batches, Telegram Takeout batch details, warnings, and item
     observations shipped.
   - Remaining work: finish incomplete-import policy and real-data validation.
   - Enable migrated-history import only after real-data validation.

4. Snapshot hardening. Shipped for new runs.
   - Persist corpus snapshots before provider execution.
   - Expose explicit missing/capture-failed snapshot states.
   - Remaining work: optional UI affordances for legacy snapshotless runs.

5. Topic membership materialization. Shipped.
   - Repeated inference joins were replaced with `item_topic_memberships`.
   - Memberships are updated during migration, topic refresh, and scoped
     Telegram item insert when state is usable.

6. YouTube playlist simplification.
   - Make playlist entries point to stable video entities.
   - Collapse duplicated removal state.

7. Migration baseline cleanup. Shipped.
   - Active migrations start at current-schema baseline v1.
   - Pre-reset SQL and Rust migration code is archived outside the active build.
   - One controlled pre-reset database is handled by backup-first history cutover.

## Expected Backend Payoff

- fewer provider-specific branches in generic source code;
- simpler source uniqueness and upsert logic;
- safer Takeout import for migrated Telegram history;
- less repeated metadata decoding;
- simpler live analysis corpus loading;
- simpler Telegram NotebookLM export queries through the settled archive read
  model;
- clearer future provider integration boundaries.
