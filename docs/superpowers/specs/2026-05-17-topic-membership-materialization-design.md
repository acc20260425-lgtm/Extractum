# Topic Membership Materialization Design

> Date: 2026-05-17
> Status: approved direction, pending implementation plan
> Scope: Database Schema Simplification slice after Telegram native item identity

## Summary

This slice materializes Telegram forum topic membership so source readers and
NotebookLM export no longer repeat the multi-branch topic inference join.

`item_topic_memberships` stores membership only to real
`telegram_forum_topics` rows. `Unrecognized topic` remains a derived bucket,
not a persisted topic or membership row. A source-level
`telegram_topic_resolution_state` table records when missing membership rows
can be interpreted as processed-but-unmatched instead of not-yet-resolved.

The canonical resolver is a shared SQL-oriented contract. It is used by
migration 22, by full source rebuilds after topic refresh, and by scoped
insert-time or batch resolution for newly inserted Telegram items.

## Current State

Telegram forum membership is currently inferred in readers with a shared SQL
predicate:

- `items.reply_to_top_id = telegram_forum_topics.topic_id`;
- root-message fallback through `telegram_messages.telegram_message_id =
  telegram_forum_topics.top_message_id`;
- legacy root-message fallback by casting `items.external_id` only when the row
  has no typed `telegram_messages` child row;
- `items.reply_to_msg_id = telegram_forum_topics.topic_id`;
- fallback to the real `General` topic when the local catalog contains
  `topic_id = 1`;
- otherwise a synthetic `Unrecognized topic` bucket.

This logic is used by:

- `src-tauri/src/sources/topics.rs`;
- `src-tauri/src/sources/items/query.rs`;
- `src-tauri/src/notebooklm_export/query.rs`.

The previous Telegram native item identity slice added `telegram_messages`.
That lets the root-message fallback prefer typed Telegram message identity, but
the reader/export paths still repeat topic inference.

## Goals

1. Add a materialized `item_topic_memberships` table for real Telegram forum
   topic memberships.
2. Add source-level resolution state so missing membership rows are not
   ambiguous.
3. Keep `General` as a real topic only when `telegram_forum_topics` contains
   `topic_id = 1`.
4. Keep `Unrecognized topic` as a derived bucket, not a persisted topic.
5. Make migration 22 bring existing catalog-backed Telegram forum sources to a
   ready materialized state.
6. Use one shared resolver contract for migration, topic refresh rebuilds, and
   scoped runtime resolution.
7. Move readers/export to indexed materialized joins instead of repeated
   inference joins.
8. Preserve historical matches to retained hidden/deleted topics.

## Non-Goals

- Do not create synthetic `telegram_forum_topics` rows for `Unrecognized`.
- Do not store `item_topic_memberships.topic_id = NULL` or
  `match_kind = 'unrecognized'`.
- Do not remove Telegram reply/topic columns from `items`.
- Do not remove the legacy external-id root fallback for rows without typed
  `telegram_messages` child rows.
- Do not introduce a provider-neutral document layer in this slice.
- Do not make Takeout topic catalog refresh mandatory.
- Do not redesign the visual topic UI beyond minimal API/client updates needed
  to consume the new backend shape.

## Data Model

Migration 22 adds:

```sql
CREATE TABLE item_topic_memberships (
    item_id INTEGER PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
    source_id INTEGER NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
    topic_id INTEGER NOT NULL,
    match_kind TEXT NOT NULL,
    resolver_version INTEGER NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),

    FOREIGN KEY (source_id, topic_id)
        REFERENCES telegram_forum_topics(source_id, topic_id)
        ON DELETE CASCADE,

    CHECK (match_kind IN (
        'reply_to_top_id',
        'typed_root_top_message_id',
        'legacy_root_external_id',
        'reply_to_msg_id',
        'general_fallback'
    )),
    CHECK (resolver_version > 0)
);

CREATE INDEX idx_item_topic_memberships_source_topic
    ON item_topic_memberships(source_id, topic_id);

CREATE INDEX idx_item_topic_memberships_source_item
    ON item_topic_memberships(source_id, item_id);
```

`item_id` is the primary key because a Telegram message belongs to at most one
forum topic in this model. If future providers or features need multi-topic
membership, that should be a separate schema extension.

Row-level `item_topic_memberships.resolver_version` is diagnostic and useful
for audits, tests, and rebuild debugging. Reader truth is determined by the
source-level `telegram_topic_resolution_state`, not by mixed per-row resolver
version semantics.

`source_id` is duplicated for fast source/topic queries and for the composite
foreign key to `telegram_forum_topics`. The application invariant is:

```text
item_topic_memberships.source_id must equal items.source_id for the referenced
item_topic_memberships.item_id.
```

Migration and runtime validation must enforce this invariant before marking
resolution state as ready.

Migration 22 also adds:

```sql
CREATE TABLE telegram_topic_resolution_state (
    source_id INTEGER PRIMARY KEY REFERENCES sources(id) ON DELETE CASCADE,
    resolver_version INTEGER NOT NULL,
    catalog_refreshed_at INTEGER,
    memberships_refreshed_at INTEGER,
    status TEXT NOT NULL,
    unresolved_count INTEGER NOT NULL DEFAULT 0,
    pending_item_count INTEGER NOT NULL DEFAULT 0,
    last_error TEXT,
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),

    CHECK (resolver_version > 0),
    CHECK (status IN (
        'never_run',
        'ready',
        'dirty',
        'rebuilding',
        'failed'
    )),
    CHECK (unresolved_count >= 0),
    CHECK (pending_item_count >= 0)
);
```

Status meanings:

- `never_run`: membership resolution has not completed for this source.
- `ready`: full rebuild completed for the current resolver version.
- `dirty`: new or changed eligible items/catalog state require a full rebuild.
- `rebuilding`: a runtime full rebuild is in progress.
- `failed`: the last full rebuild failed.

`pending_item_count` is operational diagnostics. It counts eligible items
inserted or changed since the last full successful rebuild that have not yet
been processed by scoped resolution. If scoped resolution runs synchronously and
successfully for all inserted items, it normally remains `0`.

`unresolved_count` counts eligible Telegram forum items processed by the
current resolver version that did not receive a real topic membership row.

Migration 22 creates a persistent `telegram_topic_resolution_state` row for
every known Telegram `supergroup` source. Sources with catalog rows become
`ready` after rebuild. Sources without catalog rows become `never_run`.
Runtime readers still treat missing state defensively as `never_run` for older
or partially repaired databases, but normal post-migration state should be
persisted.

`telegram_topic_resolution_state` rows are valid only for sources where
`source_type = 'telegram'` and `source_subtype = 'supergroup'`. This provider
subtype invariant cannot be expressed by the table foreign key and must be
validated by migration/runtime checks.

## Core Semantics

An eligible Telegram forum item means:

- the item belongs to a Telegram source;
- the source has `source_subtype = 'supergroup'`;
- the item has `item_kind = 'telegram_message'`;
- the source has topic catalog/resolution state for the current resolver
  version.

A missing membership row means `Unrecognized topic` only for eligible items
when `telegram_topic_resolution_state.status = 'ready'` and
`resolver_version` matches the current resolver version.

Sources without topic catalog are not interpreted as unrecognized. Missing
membership for those sources means not resolved or not applicable.

`Unrecognized topic` is not a row in `telegram_forum_topics` and not a row in
`item_topic_memberships`. It remains a synthetic UI/export bucket derived from
ready resolution state.

Full rebuilds match against retained `telegram_forum_topics` rows, including
hidden/deleted rows that remain in the local catalog. This preserves historical
message-to-topic matches even when a later Telegram catalog refresh omits or
marks a topic.

## Resolver Contract

The resolver must be a single SQL-oriented contract shared by:

- migration 22 full rebuilds;
- full source rebuilds after successful topic refresh;
- scoped resolution for newly inserted items during normal sync;
- batch/scoped resolution for Takeout import.

It must choose at most one real topic per eligible item by deterministic
priority:

1. `items.reply_to_top_id = telegram_forum_topics.topic_id`
   (`match_kind = 'reply_to_top_id'`).
2. `telegram_messages.telegram_message_id =
   telegram_forum_topics.top_message_id`
   (`match_kind = 'typed_root_top_message_id'`).
3. Legacy root fallback from numeric `items.external_id` to
   `telegram_forum_topics.top_message_id`
   (`match_kind = 'legacy_root_external_id'`), but only when the item has no
   typed `telegram_messages` child row.
4. `items.reply_to_msg_id = telegram_forum_topics.topic_id`
   (`match_kind = 'reply_to_msg_id'`).
5. Real `General` fallback (`match_kind = 'general_fallback'`), only when the
   source catalog contains `topic_id = 1`.

If no real topic is found, the resolver writes no membership row and the item
contributes to `unresolved_count` when the source is fully processed.

If multiple priorities could match, the lowest numeric priority wins. The
legacy root cast must not run as a second chance when a typed
`telegram_messages` child row exists.

The legacy external-id cast must stay isolated inside the resolver fallback and
tests. Reader/export code must not reintroduce topic inference through
`CAST(items.external_id AS INTEGER)`.

## Migration 22 Contract

Migration 22 is runner-managed and runs after migration 21. It assumes
`telegram_messages` exists and root-message matching can prefer typed Telegram
message identity.

Migration 22 must:

1. Create `item_topic_memberships`.
2. Create `telegram_topic_resolution_state`.
3. For every Telegram `supergroup` source with at least one local
   `telegram_forum_topics` row, run a full source-level rebuild.
4. For every Telegram `supergroup` source without catalog rows, create a
   persistent `never_run` resolution state row.
5. Fail fast on source rebuild or integrity errors.
6. Record migration success only after schema, rebuilds, and integrity checks
   complete.

Migration 22 should run schema creation and rebuilds inside one explicit
migration transaction where possible. If implementation is forced to rely on
source-level transactions, each source rebuild must be atomic, and migration
must fail before recording success if any source cannot be rebuilt. The app
must not start with migration 22 marked successful while sources are in a
mixed ready/partial state.

During migration, `rebuilding` may be transient and unobservable if the rebuild
runs inside the migration transaction.

The full rebuild shape is:

```text
BEGIN transaction
mark source rebuilding if visible runtime state is needed
DELETE FROM item_topic_memberships WHERE source_id = ?
bulk INSERT real-topic memberships using the shared resolver
compute eligible_items, inserted_memberships, unresolved_count
run post-rebuild invariant checks
set state ready for the current resolver version
COMMIT
```

Rebuilds must be set-based/bulk-oriented, not Rust item-by-item loops.

`catalog_refreshed_at` should be derived from the local catalog when possible,
for example the maximum relevant `telegram_forum_topics.updated_at` or
`last_seen_at`. `memberships_refreshed_at` is the rebuild time.

For catalog-backed sources after successful migration:

```text
status = ready
resolver_version = CURRENT_TOPIC_RESOLVER_VERSION
unresolved_count = computed processed-without-real-topic count
pending_item_count = 0
last_error = NULL
```

For sources without catalog:

```text
status = never_run
resolver_version = CURRENT_TOPIC_RESOLVER_VERSION
catalog_refreshed_at = NULL
memberships_refreshed_at = NULL
unresolved_count = 0
pending_item_count = 0
```

Post-rebuild invariants:

- `inserted_memberships + unresolved_count = eligible_items`;
- every membership `source_id` equals the referenced `items.source_id`;
- every membership `topic_id` references a real `telegram_forum_topics` row;
- every membership `item_id` belongs to an eligible Telegram forum item;
- every membership `resolver_version` equals the source
  `telegram_topic_resolution_state.resolver_version` for ready sources;
- every `telegram_topic_resolution_state` row belongs to a Telegram
  `supergroup` source;
- `state.status = 'ready'` only after these checks pass;
- `PRAGMA foreign_key_check` returns no rows after migration.

Provider subtype invariant check:

```sql
SELECT s.id
FROM telegram_topic_resolution_state st
JOIN sources s ON s.id = st.source_id
WHERE s.source_type <> 'telegram'
   OR s.source_subtype <> 'supergroup';
```

## Runtime Flow

Runtime uses a hybrid model:

```text
new Telegram forum item inserted -> scoped membership resolution when possible
successful topic catalog refresh -> full source-level membership rebuild
```

The correctness boundary is the full source rebuild after every successful
topic catalog refresh. Insert-time/scoped resolution is an operational
freshness optimization, not the only source of truth.

Runtime full rebuild and scoped resolution for the same source must serialize
writes to `item_topic_memberships` and `telegram_topic_resolution_state`. Use
the existing source-level ingest/topic lock if it already covers these writes,
or introduce a dedicated topic-resolution source lock. A topic refresh rebuild
must not interleave its delete-plus-bulk-insert membership write with sync or
Takeout scoped resolution for the same source.

Normal Telegram sync:

1. Insert `items` and `telegram_messages` through the native Telegram identity
   path.
2. If the source is a Telegram `supergroup` and has usable topic catalog state,
   run scoped resolution for inserted item ids.
3. If scoped resolution succeeds for all inserted eligible items and the source
   was `ready`, keep it `ready` and keep `pending_item_count = 0`.
4. When state is `ready` and scoped resolution succeeds for inserted eligible
   items, increment `unresolved_count` by the number of inserted eligible items
   that produced no membership row. These rows are unresolved, not pending.
5. If scoped resolution is skipped or fails, do not delete existing
   memberships. Mark the source `dirty` or `failed` according to severity,
   record `last_error` without raw payload contents, and require a later full
   rebuild to return to `ready`.

Takeout import:

- Use the same scoped resolver for inserted items when a topic catalog is
  available.
- Large imports may batch-resolve inserted ids instead of resolving item by
  item.
- Takeout does not have to refresh the topic catalog in this slice, but the
  hook for a later refresh-and-rebuild flow should remain clear.

Topic refresh:

1. Upsert/mark retained topic catalog rows using existing refresh semantics.
2. Run a full source-level rebuild after successful catalog refresh.
3. Topic refresh rebuild is stronger than incremental state and may delete and
   reinsert all source memberships because memberships are derived data.
4. Runtime rebuild may expose `rebuilding` state while work is in progress.

## Reader And API Contract

Reader/export paths should use materialized memberships instead of repeating
topic inference:

- `list_source_forum_topics`;
- `list_source_items` with `topicFilter`;
- NotebookLM export query.

Real topic counts use indexed joins from `telegram_forum_topics` to
`item_topic_memberships`.

`Unrecognized` count/filter uses eligible items without membership rows only
when source resolution state is `ready` for the current resolver version.
The backend, not the frontend, decides whether `state.resolver_version` is the
current resolver version. DTOs expose `resolverVersion` for diagnostics and
display only.

If `topicFilter = Unrecognized` and topic resolution state is not ready,
backend must not silently return all missing-membership rows as unrecognized.
For the first slice, `list_source_items` returns an empty item result for a
non-ready `Unrecognized` filter. Callers must use `list_source_forum_topics` to
inspect topic resolution state. This keeps the current item-list response shape
stable while avoiding mislabeled rows.

`list_source_forum_topics` should return:

- real topic rows;
- an optional synthetic `Unrecognized topic` row only when state is ready;
- `topic_resolution_state` summary.

Minimal resolution state DTO:

```text
status
resolverVersion
unresolvedCount
pendingItemCount
membershipsRefreshedAt
```

When state is `ready` and `unresolved_count = 0`, normal topic listing may omit
the synthetic `Unrecognized` bucket. An explicit `Unrecognized` filter remains
valid and returns an empty result.

The first implementation may require a small frontend API-client shape update
to unwrap `topics` and carry the state summary. It does not need new visual UI
for state display.

## Error Handling

Migration errors:

- Schema creation failures fail migration 22.
- Rebuild SQL failures fail migration 22.
- Post-rebuild invariant failures fail migration 22.
- Migration 22 must not record success if any source rebuild is partial or
  invalid.

Runtime errors:

- Scoped resolution failure does not delete existing memberships.
- Full rebuild failure marks state `failed` and preserves diagnostics without
  raw payload contents.
- Readers must not treat missing memberships as `Unrecognized` while state is
  `never_run`, `dirty`, `rebuilding`, or `failed`.
- Missing state should be treated as `never_run` rather than as ready.

## Test Strategy

Migration tests:

- migration 22 is registered after migration 21;
- schema contains both new tables and expected indexes;
- migration 22 rebuilds existing catalog-backed Telegram supergroup sources;
- sources without catalog get persistent `never_run` state rows;
- migration uses typed root matching through `telegram_messages`;
- legacy root fallback works only for rows without typed child identity;
- retained hidden/deleted topics are eligible match targets;
- no synthetic `Unrecognized` topic or membership rows are persisted;
- post-rebuild invariants reject source-id mismatch and non-eligible items;
- `inserted_memberships + unresolved_count = eligible_items`;
- migration fails on integrity failures before recording success.

Resolver tests:

- priority order is `reply_to_top_id`, typed root, legacy root, `reply_to_msg_id`,
  real `General`, unresolved;
- typed child rows do not use legacy external-id fallback as a second chance;
- real `General` fallback applies only when catalog contains `topic_id = 1`;
- unresolved items produce no membership row;
- scoped and full rebuild paths use the same resolver contract.

Runtime tests:

- normal sync resolves newly inserted Telegram forum items when catalog is
  ready;
- successful scoped resolution keeps ready state ready;
- scoped unresolved items update unresolved accounting, not pending accounting,
  when incremental accounting is maintained;
- scoped failure preserves existing memberships and marks state dirty/failed;
- topic refresh performs full delete-plus-bulk-insert rebuild for the source;
- Takeout can batch-resolve inserted items when catalog is available.

Reader/export tests:

- `list_source_forum_topics` counts real topics through memberships;
- synthetic `Unrecognized` appears only for ready state;
- explicit `Unrecognized` filter is valid and empty when ready/unresolved zero;
- non-ready `Unrecognized` filter returns no mislabeled missing rows;
- `list_source_items` topic filtering uses memberships;
- NotebookLM export uses memberships;
- reader/export code no longer embeds resolver match order.
- ready-source memberships have the same `resolver_version` as the source
  resolution state.

Containment scans:

```powershell
rg -n "CAST\\(.*external_id AS INTEGER\\)|external_id NOT GLOB" src-tauri\src
rg -n "reply_to_top_id.*telegram_forum_topics|top_message_id|reply_to_msg_id.*telegram_forum_topics" src-tauri\src\sources src-tauri\src\notebooklm_export
rg -n "item_topic_memberships|telegram_topic_resolution_state" src-tauri\src src-tauri\migrations docs
```

Expected:

- `items.external_id` integer casts appear only in resolver legacy fallback or
  tests;
- topic resolver fields such as `top_message_id` are allowed in resolver,
  migration, tests, and docs, but disallowed as embedded inference logic in
  readers/export;
- new membership/state tables appear in migration, resolver/runtime code,
  docs, and tests.

## Documentation

Update `docs/database-schema.md`:

- document `item_topic_memberships`;
- document `telegram_topic_resolution_state`;
- explain real-only memberships;
- explain that `Unrecognized topic` is derived from ready state and not
  persisted;
- explain current resolver version and state semantics.

Backlog remains open-work-only. Keep follow-ups such as provider-neutral
document layer, Takeout provenance, and richer topic/export enhancements where
still open. Do not add completed implementation notes to `docs/backlog.md`.

`docs/database-schema-legacy-analysis.md` may remain historical. If updated,
only adjust it to point at remaining open simplification work; do not turn it
into a shipped-work changelog.

## Implementation Phases

1. Add migration 22 schema and registration tests.
2. Implement the shared resolver SQL contract and migration rebuild tests.
3. Run migration 22 rebuild for catalog-backed sources and write ready/never-run
   state.
4. Move `list_source_forum_topics`, `list_source_items`, and NotebookLM export
   to materialized joins.
5. Run full source rebuild after successful topic refresh.
6. Add scoped runtime resolution for normal Telegram sync and Takeout insert
   paths.
7. Update docs and containment scans.

## Acceptance Criteria

This slice is complete when:

- existing Telegram supergroup sources with topic catalog are ready immediately
  after migration 22;
- `item_topic_memberships` contains only real Telegram forum topic memberships;
- `Unrecognized topic` is derived only for ready state and never persisted;
- source readers and NotebookLM export no longer repeat topic inference joins;
- runtime sync/Takeout can resolve new items without waiting for the next topic
  refresh when catalog state is usable;
- every successful topic refresh triggers a full source membership rebuild;
- hidden/deleted retained topics still preserve historical matches;
- containment scans keep legacy external-id casts inside resolver fallback;
- full Rust verification passes after implementation;
- frontend checks pass if frontend API client files change.
