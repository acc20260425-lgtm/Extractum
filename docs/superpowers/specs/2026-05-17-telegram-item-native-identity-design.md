# Telegram Item Native Identity Design

> Date: 2026-05-17
> Status: approved direction, pending implementation plan
> Scope: first item/document identity cleanup slice for Telegram messages

## Summary

This slice introduces a minimal typed identity layer for Telegram items without
rewriting the analysis/export document model. `items` remains the stable local
item container and compatibility surface. Telegram message duplicate detection
and topic/message identity move to a provider-specific child table.

The goal is to decouple Telegram native message identity from
`items.external_id`, so message ids are interpreted inside a Telegram history
domain rather than as a single per-source string. This prepares Takeout and
normal sync for migrated-history cases where multiple Telegram histories may
contain the same message id under one Extractum source.

## Current State

`items` is currently a polymorphic table. It stores Telegram messages, YouTube
transcripts, YouTube comments, media flags, compressed raw payloads, Telegram
reply/topic/reaction fields, and analysis/export input rows.

Telegram insertion currently uses:

```sql
ON CONFLICT(source_id, external_id) DO NOTHING
```

The current schema also has a unique index:

```sql
CREATE UNIQUE INDEX idx_items_ext ON items(source_id, external_id);
```

This makes `items.external_id` the durable duplicate identity for every
provider. For Telegram, `external_id` is the message id string. That is not
enough for migrated histories because Telegram message ids are scoped to a
history domain. The same source may need to represent message id `42` from the
current supergroup history and message id `42` from an old migrated small-group
history.

Forum topic resolution also casts `items.external_id` to an integer to compare
message ids with topic top-message ids. That keeps Telegram-specific identity
logic in generic item queries.

## Goals

1. Add a typed Telegram message identity child table.
2. Make Telegram duplicate detection use native Telegram identity instead of
   `(source_id, external_id)`.
3. Keep `items.id` as the stable local item id for browsing, analysis, saved
   refs, and NotebookLM export.
4. Keep `items.external_id` populated for compatibility and display/debug
   purposes.
5. Allow multiple Telegram items under the same `source_id` to share the same
   `items.external_id` when their native Telegram history domains differ.
6. Keep YouTube item upserts deterministic after the old global
   `idx_items_ext` uniqueness is removed.
7. Let forum-topic matching use typed Telegram message ids when available,
   with legacy fallback for rows that were not backfilled.
8. Preserve existing analysis and NotebookLM export behavior.

## Non-Goals

- Do not introduce a full `documents` or `source_documents` layer.
- Do not move YouTube transcript or comment identity into typed child tables.
- Do not rewrite media storage or media-aware analysis.
- Do not harden analysis snapshot schema in this slice.
- Do not materialize full `item_topic_memberships`.
- Do not remove `items.external_id`.
- Do not remove Telegram reply/topic/reaction columns from `items` in this
  slice.
- Do not enable migrated-history Takeout import automatically. The storage and
  insert path must be able to represent overlapping history domains, but live
  product enablement remains a later validation slice.

## Data Model

Add a new provider-specific table:

```sql
CREATE TABLE telegram_messages (
    item_id INTEGER PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
    source_id INTEGER NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
    history_peer_kind TEXT NOT NULL,
    history_peer_id INTEGER NOT NULL,
    telegram_message_id INTEGER NOT NULL,
    migration_domain TEXT,
    is_migrated_history INTEGER NOT NULL DEFAULT 0,
    reply_to_msg_id INTEGER,
    reply_to_peer_kind TEXT,
    reply_to_peer_id INTEGER,
    reply_to_top_id INTEGER,
    reaction_count INTEGER,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    CHECK (history_peer_kind IN ('channel', 'chat', 'user')),
    CHECK (telegram_message_id > 0),
    CHECK (is_migrated_history IN (0, 1)),
    CHECK (reply_to_msg_id IS NULL OR reply_to_msg_id > 0),
    CHECK (
        reply_to_peer_kind IS NULL
        OR reply_to_peer_kind IN ('channel', 'chat', 'user')
    ),
    CHECK (reply_to_peer_id IS NULL OR reply_to_peer_id > 0),
    CHECK (reply_to_top_id IS NULL OR reply_to_top_id > 0),
    CHECK (reaction_count IS NULL OR reaction_count >= 0)
);

CREATE UNIQUE INDEX ux_telegram_messages_native_identity
    ON telegram_messages (
        source_id,
        history_peer_kind,
        history_peer_id,
        telegram_message_id
    );

CREATE INDEX idx_telegram_messages_source_message
    ON telegram_messages(source_id, telegram_message_id);

CREATE INDEX idx_telegram_messages_source_reply_top
    ON telegram_messages(source_id, reply_to_top_id);
```

Native Telegram duplicate identity is:

```text
source_id + history_peer_kind + history_peer_id + telegram_message_id
```

`migration_domain` is intentionally not part of the first unique key. In this
slice it is diagnostic and future-proofing metadata only. Runtime code must not
use `migration_domain` for duplicate detection, topic matching, or ref
resolution unless a future migration explicitly promotes it into the identity
contract. The history domain for this slice is the Telegram history peer
kind/id pair. For non-migrated current history, `history_peer_kind` and
`history_peer_id` usually equal the resolved source peer. For migrated history,
they must identify the original Telegram history domain, not necessarily the
current source peer. If live validation later proves that a second
discriminator is required, a future migration can promote `migration_domain`
into the unique identity.

`history_peer_id` must use the same normalized integer representation already
used by `telegram_sources.peer_id`. Migration 21 must not add a stronger
database check for `history_peer_id` than `telegram_sources.peer_id` has unless
implementation tests prove that all supported stored peer ids satisfy that
check. If the normalized representation can be signed, the schema must allow
signed values before migration 21 ships.

`items.external_id` remains the message id string for compatibility. It is not
the authoritative Telegram duplicate key after this slice.

`telegram_messages.item_id` must always reference an `items` row whose
`item_kind = 'telegram_message'`, and `telegram_messages.source_id` must equal
the referenced `items.source_id`. These are application invariants for the first
slice rather than database triggers. Migration and runtime tests must enforce
the invariants. If later code creates a need for database-level enforcement,
add triggers in a separate hardening slice.

`created_at` and `updated_at` use integer Unix epoch seconds to match the
existing migration/runtime convention for provider child tables such as
`telegram_sources`, `youtube_playlist_items`, and typed YouTube source tables.
In this slice, `updated_at` is set when the child row is created. Duplicate
native-identity skips do not update `telegram_messages.updated_at`.

## Migration Contract

Add migration 21.

The migration may be plugin-managed SQL unless implementation discovers a need
for Rust-side parsing that cannot be expressed safely in SQLite. It does not
need the foreign-key-sensitive table rebuild machinery used by migration 19.

Migration 21 must:

1. Create `telegram_messages`.
2. Backfill typed rows for existing Telegram message items where safe.
3. Drop the global unique index `idx_items_ext`.
4. Add replacement uniqueness for non-Telegram item upserts.
5. Keep a non-unique lookup index on `(source_id, external_id)` for legacy refs
   and existing browsing/export lookups.

Backfill is best-effort:

- Include only rows where `items.item_kind = 'telegram_message'`.
- Require a matching Telegram source and matching `telegram_sources` row.
- Require `items.external_id` to be a positive ASCII decimal integer with no
  sign, whitespace, decimal point, or non-digit characters.
- Use `telegram_sources.peer_kind` and `telegram_sources.peer_id` as the
  default history domain for existing rows.
- Copy reply/topic/reaction values from `items` where they are valid.
- Copy `reply_to_peer_kind` only when it is one of `channel`, `chat`, or `user`.
- Copy `reply_to_peer_id` only when it parses as a positive decimal integer.
- Set `is_migrated_history = 0`.
- Leave malformed or incomplete legacy rows in `items` without typed identity.

Malformed legacy item rows must not fail the whole migration. They remain
readable through compatibility paths, but they are not valid typed Telegram
message rows until a future repair or re-ingest writes the child row.
Migration tests should verify the number of backfilled and skipped rows in
representative fixtures. Runtime diagnostics may expose backfilled and skipped
counts in logs, but must not include raw payload contents.

The replacement non-Telegram uniqueness should protect current YouTube item
upserts without blocking duplicate Telegram message ids. This design uses
Option A for legacy `item_kind` handling: migration 16 already adds
`items.item_kind TEXT NOT NULL DEFAULT 'telegram_message'`, so migration 21
must assert that no current `items.item_kind` values are `NULL` before replacing
`idx_items_ext`. If unexpected `NULL` values exist, migration 21 must fail with
a data-integrity error instead of creating a partial index that silently omits
rows that still need uniqueness.

The replacement index shape is:

```sql
CREATE UNIQUE INDEX ux_items_non_telegram_external
    ON items(source_id, external_id)
    WHERE item_kind <> 'telegram_message';

CREATE INDEX idx_items_source_external
    ON items(source_id, external_id);
```

Do not use a predicate that includes `item_kind IS NULL` unless the design is
explicitly changed to Option B and every matching UPSERT uses that exact
predicate. Under this spec, current rows must have non-`NULL` `item_kind`.

The implementation must update YouTube item UPSERT statements to target the
partial unique index explicitly, for example:

```sql
ON CONFLICT(source_id, external_id)
WHERE item_kind <> 'telegram_message'
DO UPDATE SET
    item_kind = excluded.item_kind,
    author = excluded.author,
    published_at = excluded.published_at,
    ingested_at = excluded.ingested_at,
    content_zstd = excluded.content_zstd,
    raw_data_zstd = excluded.raw_data_zstd,
    content_kind = excluded.content_kind,
    has_media = excluded.has_media,
    media_kind = excluded.media_kind,
    media_metadata_zstd = excluded.media_metadata_zstd
```

If SQLite or SQLx rejects that conflict-target shape in local tests, the plan
must choose an equivalent deterministic YouTube upsert strategy before dropping
`idx_items_ext`.

Do not drop `idx_items_ext` until local SQLx tests prove the replacement
YouTube transcript and comment upserts work deterministically against the
replacement uniqueness strategy.

Post-migration integrity checks:

- `PRAGMA foreign_key_check` returns no rows.
- No `telegram_messages` row points to a non-Telegram source item.
- Every `telegram_messages.item_id` has a matching `items` row with
  `items.item_kind = 'telegram_message'`.
- Every `telegram_messages.source_id` equals the referenced `items.source_id`.
- No duplicate native Telegram identities exist.
- Non-Telegram duplicate rows by `(source_id, external_id)` are detected before
  creating `ux_items_non_telegram_external`.

The source-id invariant check should use this shape:

```sql
SELECT tm.item_id
FROM telegram_messages tm
JOIN items i ON i.id = tm.item_id
WHERE tm.source_id <> i.source_id;
```

## Runtime Contract

Introduce a Telegram-specific insert path, for example
`insert_telegram_source_item`.

The Rust identity type should use the same vocabulary as the schema. If the
type is named `TelegramMessageIdentity`, its `history_peer_*` fields should
carry an inline comment equivalent to:

```rust
/// Telegram history/origin peer for this message, not necessarily the current source peer.
```

The insert path must:

1. Receive or derive a `TelegramMessageIdentity` containing
   `history_peer_kind`, `history_peer_id`, and `telegram_message_id`.
2. Own a write transaction before checking native identity. Use
   `BEGIN IMMEDIATE` or an equivalent SQLx/SQLite pattern that acquires the
   writer before the identity check.
3. Run:
   ```sql
   SELECT item_id
   FROM telegram_messages
   WHERE source_id = ?
     AND history_peer_kind = ?
     AND history_peer_id = ?
     AND telegram_message_id = ?
   ```
4. If a row exists, roll back or end the local transaction without changes and
   return `inserted = false`.
5. Insert the `items` row only after the write transaction is held and the
   native identity is absent.
6. Insert the matching `telegram_messages` child row in the same transaction.
7. If the child insert still hits the native unique constraint, load the
   existing `item_id`, roll back the just-inserted `items` row by rolling back
   the transaction, and return `inserted = false`.
8. Commit only after both `items` and `telegram_messages` writes succeed.
9. Return `inserted = false` for duplicates without updating the existing
   item payload.
10. Keep `items.external_id = telegram_message_id.to_string()`.

Normal Telegram sync should derive native identity from the message's Telegram
peer and message id. If the message peer is unavailable or invalid, sync should
fall back to the resolved source peer only when that fallback is semantically
equivalent for the current source.

Takeout import should derive native identity from the raw message peer and
message id when the raw TL message carries a peer. If a current Takeout path
does not expose enough peer data, it may use the resolved source peer as a
compatibility fallback and keep migrated-history import disabled until the
later validation slice.

The implementation plan must identify the production boundary where
`history_peer_kind` and `history_peer_id` are extracted from raw Takeout TL
messages and carried into the insert request. Migrated-history tests must
exercise that same boundary; they must not rely on a synthetic-only shortcut
that bypasses production parsing or insert construction.

The generic `insert_source_item` helper must not remain the normal Telegram
duplicate-detection path. It may either become non-Telegram-only or be split
into provider-specific helpers.

YouTube transcript and comment upserts remain in `items` for this slice. Their
conflict handling must continue to update existing rows deterministically.

## Topic Resolution Contract

Forum topic resolution should use `telegram_messages.telegram_message_id` when
a typed child row exists.

The existing `items.external_id` cast may remain only as a legacy fallback for
Telegram items without a typed child row. New tests must prove that typed rows
do not need `CAST(items.external_id AS INTEGER)` to match topic top messages.
The typed join must be the preferred path. Any legacy external-id cast fallback
must be visibly isolated in code, for example behind a helper named
`legacy_external_id_message_id_expr` or an equivalently explicit SQL branch, so
new topic logic does not accidentally spread raw `items.external_id` integer
casts through query code.

This slice does not introduce `item_topic_memberships`. That remains a later
schema simplification step.

## Compatibility Contract

Existing stable local refs based on `items.id`, such as
`s{source_id}-i{item_id}`, remain authoritative.

Legacy Telegram refs based on source id plus message id remain supported when
they resolve to exactly one item. If multiple Telegram history domains now have
the same message id under one source, a legacy message-id ref is ambiguous and
must return `AppError::conflict` rather than silently choosing the wrong item.
If a legacy message-id ref resolves to zero candidates, return
`AppError::not_found`.

Analysis and NotebookLM export continue to read `items` rows. They may join
`telegram_messages` only when they need typed Telegram identity. They must not
decode raw Telegram payloads merely to identify a message.

## Error Handling

Migration errors:

- Creating the new table or indexes failing is a startup migration error.
- Backfill skips malformed legacy rows instead of failing the migration.
- Foreign-key violations after migration are a migration failure.
- Post-migration integrity check failures are migration failures.

Runtime errors:

- Invalid Telegram native identity is a validation error.
- Duplicate native identity is a normal skipped insert, not an error.
- A uniqueness conflict while inserting `telegram_messages` rolls back the new
  `items` row and returns `inserted = false`.
- Missing typed child rows in old data should degrade to compatibility behavior
  in browsing/export, not crash normal reads.
- Ambiguous legacy message-id refs return `AppError::conflict`; missing legacy
  message-id refs return `AppError::not_found`.

No error message should include compressed raw payload contents.

## Test Strategy

Migration tests:

- migration 21 is registered;
- fresh schema includes `telegram_messages`;
- `idx_items_ext` is gone after all migrations;
- migration 21 fails or blocks if any `items.item_kind` is unexpectedly `NULL`
  before replacing `idx_items_ext`;
- `ux_telegram_messages_native_identity` exists with the expected columns;
- non-Telegram partial uniqueness exists;
- `PRAGMA foreign_key_check` returns no rows after migration 21;
- a fixture with `telegram_messages.item_id` pointing to a non-telegram item
  violates the application invariant in migration/runtime validation tests;
- a fixture with `telegram_messages.source_id <> items.source_id` violates the
  application invariant in migration/runtime validation tests;
- non-Telegram duplicate `(source_id, external_id)` rows are detected before
  `ux_items_non_telegram_external` is created;
- valid legacy Telegram message rows are backfilled;
- malformed `items.external_id` rows are skipped without failing migration;
- representative migration fixtures assert exact backfilled and skipped row
  counts;
- duplicate Telegram message ids with different peer domains are allowed;
- duplicate native Telegram identity is rejected by the child-table unique
  index;
- YouTube transcript/comment upserts still update existing rows after
  `idx_items_ext` is replaced.

Runtime tests:

- Telegram insert skips duplicate native identity even if payload differs;
- Telegram insert allows the same message id for different peer domains under
  one source;
- normal sync helper builds typed identity from message peer/id;
- Takeout raw parse propagates history peer kind/id and message id into the insert
  request;
- Takeout import can insert two synthetic messages with the same message id and
  different raw peer domains;
- legacy source item rows without typed child rows remain readable;
- topic resolution uses `telegram_messages.telegram_message_id` when present;
- legacy topic fallback still works for old rows without child identity.

Compatibility tests:

- existing analysis corpus tests still pass;
- existing NotebookLM export tests still pass;
- `s{source_id}-i{item_id}` refs still resolve;
- legacy `s{source_id}-m{message_id}` refs resolve when unique and report a
  conflict error when not unique.

Containment scans:

```powershell
rg -n "ON CONFLICT\\(source_id, external_id\\) DO NOTHING" src-tauri\src\sources src-tauri\src\takeout_import
rg -n "CAST\\(.*items\\.external_id AS INTEGER\\)|items\\.external_id NOT GLOB" src-tauri\src
rg -n "telegram_messages" src-tauri\src src-tauri\migrations docs
```

Expected:

- Telegram insert paths no longer use `(source_id, external_id)` as duplicate
  identity.
- `items.external_id` integer casts remain only in explicit legacy fallback
  logic or tests.
- `telegram_messages` appears in migration, test support, Telegram insert
  paths, and topic/query code that needs typed Telegram identity.

## Implementation Phases

1. Draft migration 21, backfill typed Telegram message identities, add
   migration tests, and keep the final `idx_items_ext` replacement step gated
   until the replacement upsert strategy is proven.
2. Adjust YouTube transcript/comment upserts to target the replacement
   uniqueness strategy and prove them with SQLx tests.
3. Finalize migration 21 so it drops `idx_items_ext`, adds the replacement
   indexes, and verifies migration integrity checks.
4. Add the Telegram item insert helper and wire normal Telegram sync through
   native identity.
5. Propagate Takeout raw history identity through the production raw parser and
   insert request boundary.
6. Prefer typed Telegram message identity in topic resolution and isolate the
   legacy `items.external_id` cast fallback.
7. Handle legacy message-id ref ambiguity with `AppError::conflict` and
   missing refs with `AppError::not_found`.
8. Update docs and run containment scans.

## Documentation

Update `docs/database-schema.md`:

- document `telegram_messages`;
- explain that `items.external_id` is compatibility/display identity for
  Telegram messages, not authoritative duplicate identity;
- explain that `items` remains the local item/archive container in this slice;
- document the migration-21 replacement for `idx_items_ext`.

Update `docs/backlog.md` according to its open-work-only rule:

- keep future document-layer cleanup open;
- keep topic membership materialization open;
- keep Takeout provenance open;
- do not add a "completed" shipped-work note to backlog after implementation.

## Follow-Up Sequence

Recommended order after this slice:

1. Takeout provenance and ingest batches.
2. Enable migrated-history import behind explicit real-data validation.
3. Materialize topic memberships.
4. Introduce a provider-neutral document layer for analysis/export.
5. Move YouTube item identities if a concrete YouTube collision or workflow
   problem appears.

## Acceptance Criteria

The slice is complete when:

- Telegram messages have typed native identity rows where they can be derived.
- Telegram duplicate detection uses native identity, not `items.external_id`.
- The same source can store overlapping Telegram message ids from different
  peer domains.
- Existing `items.id` based refs, analysis, browsing, and NotebookLM export
  remain compatible.
- Legacy message-id refs still work when unique and fail safely when
  ambiguous.
- Forum topic matching uses typed message identity for typed rows.
- YouTube item upserts still update existing transcript/comment rows.
- Full Rust tests pass.
- Frontend checks are only required if frontend files change.
- Documentation describes the new boundary and remaining follow-ups.
