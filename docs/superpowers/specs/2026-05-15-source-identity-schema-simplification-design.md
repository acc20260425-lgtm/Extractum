# Source Identity Schema Simplification Design

> Date: 2026-05-15
> Status: approved design direction
> Scope: first safe slice of the database schema simplification backlog

## Summary

This design turns `sources.source_subtype` into the canonical provider subtype
and moves Telegram peer identity out of the hot-path `sources.metadata_zstd`
blob. The implementation should be a soft migration: new typed storage and
new normal code paths are added first, while `telegram_source_kind` remains as
a compatibility mirror until existing databases, tests, and API callers no
longer depend on it.

The goal is to simplify provider work without destabilizing existing Telegram,
Takeout, YouTube, analysis, and NotebookLM behavior. This design does not
rewrite `items`, `analysis_run_messages`, topic membership, or YouTube playlist
linking yet. It deliberately creates the source identity boundary those later
refactors will need.

## Background

The current schema carries two source identity eras at once:

- early Telegram-only rows used `source_type = 'telegram_channel'`, then
  migrated to generic `source_type = 'telegram'`;
- migration `11.sql` added Telegram-specific `telegram_source_kind`;
- migration `12.sql` made source uniqueness account-scoped for Telegram with
  `(account_id, source_type, telegram_source_kind, external_id)`;
- migration `15.sql` added provider-local `source_subtype` and backfilled
  Telegram rows from `telegram_source_kind`;
- migration `16.sql` added YouTube sources using `source_subtype = 'video'`
  or `source_subtype = 'playlist'`, while still inserting
  `telegram_source_kind = ''` for legacy `NOT NULL` compatibility.

The backend now has a provider-neutral column but still uses a
Telegram-specific field for core identity and conflict handling. Telegram peer
resolution also decodes durable identity from `sources.metadata_zstd`, including
compatibility normalization from old `username`, `added_from`, and
`access_hash` payloads into `peer_identity`.

That shape works, but it makes every new provider and every generic source
feature pay for Telegram compatibility decisions.

## Current State

Important current schema facts:

- `sources.source_type` supports `telegram`, `youtube`, `rss`, and `forum` in
  shared contracts.
- Implemented providers are Telegram and YouTube.
- `sources.source_subtype` is provider-local:
  - Telegram: `channel`, `supergroup`, `group`
  - YouTube: `video`, `playlist`
  - future RSS/forum values are placeholders only.
- `sources.telegram_source_kind` is still selected in many backend source
  queries.
- Telegram upsert uses:
  - `ON CONFLICT(account_id, source_type, telegram_source_kind, external_id)`
- YouTube upsert uses:
  - `ON CONFLICT(source_type, source_subtype, external_id)` with partial
    YouTube indexes
  - `telegram_source_kind = ''` to satisfy legacy table shape.
- `SourceRecord` serializes both `source_subtype` and `telegram_source_kind`.
- Frontend `mapSource` still falls back from `source_subtype` to
  `telegram_source_kind`.
- `SourceSyncTarget` carries both `source_subtype` and a non-optional
  `telegram_source_kind`.
- Telegram sync, Takeout, topics, avatar cache, and peer resolution use
  `source.telegram_source_kind`.
- Telegram identity metadata is encoded in `sources.metadata_zstd`:
  - `peer_identity.strategy`
  - `peer_identity.username`
  - `peer_identity.access_hash`
  - `avatar_cache_key`
  - legacy compatibility fields normalized at decode time.

## Problem

The backend cannot treat `sources` as provider-neutral while its primary
identity path still depends on `telegram_source_kind`.

Specific costs:

1. Generic source code has to know about a Telegram-only column.
2. YouTube write paths have to write an unrelated compatibility value.
3. Telegram peer resolution has to decode compressed JSON for data that is now
   operational identity.
4. Runtime source loaders carry fields that are meaningful only for some
   providers.
5. Future provider work risks adding more provider-specific fields directly to
   `sources`.
6. Later item/document identity work lacks a clean source identity boundary.

## Goals

1. Make `source_subtype` the canonical provider subtype for all normal source
   code.
2. Add typed Telegram source identity storage for fields needed by sync,
   Takeout, avatar refresh, source listing, and peer resolution.
3. Keep old databases readable and upgradable.
4. Remove normal-path dependence on the historical `telegram_source_kind NOT
   NULL` workaround and prepare a future current-schema baseline where fresh
   installs can omit the legacy column.
5. Make Telegram-specific logic live behind Telegram-specific loaders and
   structs.
6. Keep YouTube source behavior stable while removing its normal-path need to
   know about Telegram legacy fields.
7. Preserve current product behavior:
   - add Telegram source by username, t.me URL, numeric id, or dialog;
   - list Telegram dialogs;
   - list registered sources;
   - sync Telegram channel/supergroup/group;
   - Takeout import for channel/supergroup/group;
   - topic refresh for supergroups;
   - source deletion cascade;
   - YouTube video/playlist upsert;
   - analysis and NotebookLM source selection.

## Non-Goals

This slice must not attempt these broader refactors:

- replacing `items` with a provider-neutral document layer;
- changing item uniqueness away from `(source_id, external_id)`;
- materializing Telegram topic membership;
- hardening `analysis_run_messages`;
- redesigning YouTube playlist membership or availability state;
- introducing a fresh current-schema baseline;
- removing the `telegram_source_kind` column physically from existing
  databases;
- changing the public UX for adding or browsing sources.

Those are follow-up backlog items that become easier after this source identity
boundary exists.

## Chosen Approach

Use a soft migration.

The migration adds typed provider identity and new unique constraints, then
switches normal backend reads and writes to those new boundaries. The old
`telegram_source_kind` column remains present and populated where existing table
shape requires it. It is treated as a deprecated compatibility mirror, not the
source of truth.

Target model, data invariants, migration behavior, runtime behavior,
compatibility behavior, and acceptance criteria are normative for this slice.
Concrete module names, file lists, and phase ordering below are implementation
notes unless they are repeated as invariants or acceptance criteria.

### Why This Approach

It gives the backend most of the simplification payoff without forcing a risky
SQLite table rebuild. SQLite can add tables and indexes cheaply, but removing
or changing `NOT NULL` columns requires table-copy migrations. That hard cleanup
belongs in the later fresh-baseline work after normal code paths no longer use
the legacy field.

### Rejected Alternatives

Hard removal now:

- rebuild `sources` without `telegram_source_kind`;
- update every query and fixture in one slice;
- rewrite YouTube legacy tests immediately.

This is higher risk because the field is still woven through sync, Takeout,
topics, source DTOs, fixtures, and compatibility tests.

DTO-only cleanup:

- keep schema unchanged;
- hide `telegram_source_kind` from the frontend;
- maybe add helper methods around `source_subtype`.

This reduces API noise but does not remove backend complexity. Peer resolution
would still decode operational identity from compressed metadata, and source
upserts would still rely on the old uniqueness model.

## Target Model

### Generic Source Identity

For normal code, a source is identified by:

- `id`
- `source_type`
- `source_subtype`
- provider-native `external_id`
- optional `account_id` for account-scoped providers.

Canonical combinations for implemented providers:

| Provider | source_type | source_subtype | account_id | external_id |
| --- | --- | --- | --- | --- |
| Telegram channel | `telegram` | `channel` | required | Telegram bare id as text |
| Telegram supergroup | `telegram` | `supergroup` | required | Telegram bare id as text |
| Telegram small group | `telegram` | `group` | required | Telegram bare id as text |
| YouTube video | `youtube` | `video` | null | YouTube video id |
| YouTube playlist | `youtube` | `playlist` | null | YouTube playlist id |

For new rows, `source_subtype` must be non-null. Existing nullable rows are
backfilled or handled through a legacy repair path.

In this slice, non-null `source_subtype` is an application invariant for
implemented providers, not necessarily a physical SQLite `NOT NULL` constraint
on `sources`. The later current-schema baseline can make that invariant
physical.

For Telegram rows, `sources.external_id` must be the ASCII decimal string form
of the Telegram bare peer id returned by `PeerId::bare_id()`:

- no `-100` channel prefix;
- no leading `+` or `-`;
- no leading zero padding;
- no whitespace;
- no separators or non-decimal characters;
- no empty string.

Legacy Telegram rows whose `external_id` does not parse to a non-negative
`i64` under this format are malformed for typed identity. Validation is an
exact round trip: parse as `i64`, require `parsed >= 0`, then require
`parsed.to_string() == sources.external_id`. Rust repair must not use SQLite
`CAST` or `GLOB` to coerce invalid values.

### Deprecated Compatibility Mirror

`sources.telegram_source_kind` remains in existing databases for now.

Rules:

- normal logic must not read it as source-of-truth;
- Telegram write paths may mirror `source_subtype` into it while the old column
  remains `NOT NULL`;
- YouTube write paths may continue writing `''` only inside a compatibility
  insert helper, never as business logic;
- frontend code must prefer `source_subtype`;
- backend DTOs may emit `telegram_source_kind` for one compatibility window,
  but only as `Some(source_subtype)` for Telegram rows and as non-authoritative
  deprecated data;
- frontend capability decisions and tests for new behavior must not depend on
  `telegram_source_kind`;
- the field can be physically removed only in the later current-schema
  baseline / table-rebuild slice.

## Target Schema Additions

### `telegram_sources`

Create a typed Telegram identity table:

```sql
CREATE TABLE IF NOT EXISTS telegram_sources (
    source_id INTEGER PRIMARY KEY REFERENCES sources(id) ON DELETE CASCADE,
    account_id INTEGER NOT NULL,
    source_subtype TEXT NOT NULL,
    peer_kind TEXT NOT NULL,
    peer_id INTEGER NOT NULL,
    resolution_strategy TEXT NOT NULL,
    username TEXT,
    access_hash INTEGER,
    avatar_cache_key TEXT,
    identity_refreshed_at INTEGER,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    CHECK (source_subtype IN ('channel', 'supergroup', 'group')),
    CHECK (peer_kind IN ('channel', 'chat')),
    CHECK (
        (source_subtype IN ('channel', 'supergroup') AND peer_kind = 'channel')
        OR
        (source_subtype = 'group' AND peer_kind = 'chat')
    ),
    CHECK (resolution_strategy IN ('username', 'dialog', 'legacy_metadata', 'unknown'))
);
```

`account_id` is duplicated from `sources` on purpose. SQLite cannot enforce a
cross-table unique index through a join, and Telegram uniqueness is account
scoped. The application must keep `sources.account_id` and
`telegram_sources.account_id` aligned.

`source_subtype` is also duplicated intentionally. It lets Telegram-specific
queries and indexes avoid joining `sources` just to decide peer construction.
The application must keep it aligned with `sources.source_subtype`.

For every `telegram_sources` row, these invariants must hold:

- `sources.id = telegram_sources.source_id` exists;
- `sources.source_type = 'telegram'`;
- `sources.account_id = telegram_sources.account_id`;
- `sources.source_subtype = telegram_sources.source_subtype`;
- `sources.source_subtype IN ('channel', 'supergroup', 'group')`;
- `sources.source_subtype IN ('channel', 'supergroup')` implies
  `telegram_sources.peer_kind = 'channel'`;
- `sources.source_subtype = 'group'` implies
  `telegram_sources.peer_kind = 'chat'`;
- `sources.external_id` parses as the canonical Telegram bare id text and
  equals `telegram_sources.peer_id` as text.

`sources` is the source of truth for generic source identity:
`account_id`, `source_subtype`, and `external_id`. `telegram_sources` is the
typed operational projection. If repair finds drift, it may update
`telegram_sources` from `sources` when the typed row is otherwise
non-conflicting. If the typed row conflicts with the canonical source identity
or with another typed peer identity, repair must fail startup instead of picking
a winner.

`peer_kind` describes the Telegram peer address needed to construct peer refs:

- `channel` for channel and supergroup peer refs that require a channel id and
  usually an access hash;
- `chat` for small group peer refs.

`source_subtype` remains the product/provider subtype:

- `channel`
- `supergroup`
- `group`

`resolution_strategy` records the best known source of identity:

- `username`: source can be resolved by username first;
- `dialog`: source was added from dialogs or numeric id and may depend on the
  account dialog list;
- `legacy_metadata`: row was backfilled from compressed legacy metadata;
- `unknown`: row was backfilled enough to stay readable, but lacks a trusted
  strategy.

Legacy `metadata_zstd.added_from` may be read only to derive
`resolution_strategy`; it is not persisted in `telegram_sources`.

A `telegram_sources` row can be complete or partial:

- direct channel/supergroup identity requires `peer_kind = 'channel'`,
  `peer_id`, and `access_hash`;
- username identity requires a non-empty canonical `username`;
- small group rows use `peer_kind = 'chat'`; `peer_id` is the durable typed id,
  but dialog fallback may still be needed depending on the Telegram client
  session;
- partial identity may still sync through username or dialog fallback, but the
  normal path must not decode `sources.metadata_zstd` when a typed row exists.

Store `username` as lowercase canonical operational identity, without a leading
`@`, without `t.me/` URL syntax, and without trailing path/query fragments.
Display casing is not preserved in this table. All username writes normalize
before persistence, and lookups compare the exact normalized value. Username
changes update `telegram_sources`, not the generic source identity.

Timestamp semantics:

- `created_at` is the first time the typed projection row was created;
- `updated_at` changes whenever any `telegram_sources` field changes;
- `identity_refreshed_at` changes only after a successful live Telegram
  identity refresh, not during legacy metadata backfill.

Indexes:

```sql
CREATE UNIQUE INDEX IF NOT EXISTS idx_telegram_sources_account_peer
    ON telegram_sources(account_id, peer_kind, peer_id);

CREATE INDEX IF NOT EXISTS idx_telegram_sources_account_subtype
    ON telegram_sources(account_id, source_subtype);

CREATE INDEX IF NOT EXISTS idx_telegram_sources_account_username
    ON telegram_sources(account_id, username)
    WHERE username IS NOT NULL;
```

The unique peer index is intentionally based on Telegram peer address, not
`external_id` text. It protects against duplicate typed rows while keeping
`sources.external_id` as the generic provider-native id used by existing UI and
analysis contracts.

A Telegram peer address maps to at most one Extractum Telegram source subtype
for a given account. `source_subtype` is product classification, while
`peer_kind` and `peer_id` are the MTProto peer address. If an upgraded database
contains the same `(account_id, peer_kind, peer_id)` with different
`source_subtype` values, this is a malformed identity conflict and must not be
auto-merged.

### Source Uniqueness

Add a canonical Telegram identity index after duplicate preflight and typed
repair have succeeded:

```sql
CREATE UNIQUE INDEX IF NOT EXISTS idx_sources_unique_telegram_identity
    ON sources(account_id, source_type, source_subtype, external_id)
    WHERE source_type = 'telegram';
```

This is the target uniqueness contract for new Telegram upserts. It should not
be created before the implementation has proved that every upgraded Telegram
row has a valid canonical subtype and that no duplicate canonical identities
exist. Rust upgrade repair must perform that preflight and then create this
index idempotently. SQL migration `18.sql` must not create this index.

Keep existing YouTube partial indexes for this slice:

```sql
CREATE UNIQUE INDEX IF NOT EXISTS idx_sources_unique_youtube_video
    ON sources(source_type, source_subtype, external_id)
    WHERE source_type = 'youtube' AND source_subtype = 'video';

CREATE UNIQUE INDEX IF NOT EXISTS idx_sources_unique_youtube_playlist
    ON sources(source_type, source_subtype, external_id)
    WHERE source_type = 'youtube' AND source_subtype = 'playlist';
```

Do not drop the old `idx_sources_ext` index in this slice unless the
implementation proves all old conflict targets are gone and upgrade tests pass.
Keeping it during the transition is acceptable.

### Source Identity Repair Notes Table

If migration preflight finds non-fatal enrichment gaps that do not prevent
safe sync or listing, record them instead of deleting data:

```sql
CREATE TABLE IF NOT EXISTS source_identity_repair_notes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_id INTEGER NOT NULL,
    issue_code TEXT NOT NULL,
    detail TEXT,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    FOREIGN KEY(source_id) REFERENCES sources(id) ON DELETE CASCADE,
    UNIQUE(source_id, issue_code)
);
```

This table is part of migration `18.sql`, not an optional later enhancement.
Use it for non-fatal enrichment gaps only. A missing username or avatar key is
non-fatal only when the row has a defined supported resolution path:

- channel/supergroup rows have `access_hash`, canonical username, or dialog
  fallback explicitly available for that account/session;
- group rows have chat peer id plus dialog fallback when required by the
  Telegram client.

Otherwise the row is incomplete identity, not enrichment, and must fail startup
or wait for a future explicit quarantine/manual repair path. Duplicate
canonical Telegram identities are fatal and must fail startup with a clear
validation error that lists the conflicting `source_id` values. The important
rule is: do not silently delete or merge user sources.

Repair notes are diagnostic breadcrumbs, not a quarantine system:

- no user-facing repair-notes UI is required in this slice; tests, logs, and
  future support tooling are the readers;
- affected sources may still be listed;
- affected source sync may continue only when typed identity is sufficient for
  a supported username or dialog fallback path;
- notes do not downgrade canonical duplicate or malformed identity conflicts
  into warnings;
- inactive or apparently unreferenced malformed identity rows are still fatal
  unless a deliberate manual quarantine path is implemented in a later slice;
- `detail` must be redacted and must not contain raw metadata, `access_hash`,
  session data, auth material, or full compressed payloads.

## Migration Strategy

Use a two-part migration:

1. SQL migration creates columns, tables, and non-conflicting indexes, then
   performs simple relational backfills.
2. Rust upgrade repair decodes compressed legacy metadata and fills typed
   Telegram fields that SQL cannot derive. It also performs duplicate
   preflight before enabling the canonical Telegram source unique index.
   Startup must gate source commands on this repair: list, sync, add, and
   Takeout source commands cannot run until repair succeeds or fails with a
   typed startup error.

### SQL Migration

Add a migration after `17.sql`, likely `18.sql`.

Required SQL steps:

1. Backfill missing `source_subtype` for Telegram rows:

```sql
UPDATE sources
SET source_subtype = telegram_source_kind
WHERE source_type = 'telegram'
  AND source_subtype IS NULL
  AND telegram_source_kind IN ('channel', 'supergroup', 'group');
```

2. Backfill mismatched Telegram rows conservatively:

```sql
UPDATE sources
SET source_subtype = telegram_source_kind
WHERE source_type = 'telegram'
  AND source_subtype NOT IN ('channel', 'supergroup', 'group')
  AND telegram_source_kind IN ('channel', 'supergroup', 'group');
```

This makes the legacy value a repair input only. After migration, canonical
code reads `source_subtype`.

SQL must not overwrite a valid but conflicting pair such as
`source_subtype = 'channel'` and `telegram_source_kind = 'supergroup'`.
Rust repair treats that as a fatal malformed identity conflict.

3. Create `source_identity_repair_notes`.

4. Create `telegram_sources`.

5. Create typed-table indexes that are safe on an empty typed table.

6. Do not insert typed `telegram_sources` rows in SQL. SQLite `GLOB` cannot
   safely express the required "entire string is a signed integer" predicate,
   and `CAST(external_id AS INTEGER)` can silently coerce malformed values.
   Rust repair must parse `external_id` with `parse::<i64>()` and decide
   whether the row is repairable.

7. Do not create `idx_sources_unique_telegram_identity` in SQL migration
   `18.sql`. Rust repair creates the canonical unique index after successful
   validation/backfill.

8. Do not alter `sources.telegram_source_kind` nullability in this slice.

### Rust Upgrade Repair

Add an explicit upgrade function, for example in a new module:

- `src-tauri/src/source_identity_migration.rs`

or a provider-specific module:

- `src-tauri/src/sources/identity_migration.rs`

The function should run during startup after SQL migrations are applied and
before normal source sync/list operations can use the database. It must be
idempotent.

Run the repair inside one database transaction per startup repair attempt:

- duplicate preflight;
- legacy metadata decode and typed identity derivation;
- `source_subtype` backfill;
- `telegram_sources` upsert;
- non-fatal `source_identity_repair_notes` upsert;
- canonical Telegram source unique index creation.

If a fatal row is found, roll back the transaction and fail startup with a
specific error. If the process is interrupted, the next startup reruns the
repair from the last committed database state.

Migration must never recreate `sources` rows with new ids. Existing `source_id`
values are stable contracts for items, source groups, analysis scopes, saved
runs, NotebookLM export, and source browsing.

Responsibilities:

1. Load all Telegram `sources` rows and all existing `telegram_sources` rows.
2. Validate `sources.external_id` by exact round trip before deriving
   `peer_id`: parse as `i64`, require `parsed >= 0`, and require
   `parsed.to_string() == sources.external_id`.
3. Derive `source_subtype`:
   - prefer valid `sources.source_subtype`;
   - fallback to valid `sources.telegram_source_kind`;
   - if both fields are valid and different, fail startup with a typed conflict
     error instead of choosing one silently;
   - otherwise fail with a clear repair error unless the row is explicitly
     handled as a non-fatal enrichment gap.
4. Treat any Telegram `sources` row with `account_id IS NULL` as a fatal
   malformed identity row. Do not create `telegram_sources` or the canonical
   unique index until it is fixed.
5. Decode `sources.metadata_zstd` with the existing compatibility decoder.
6. Backfill missing typed rows.
7. Validate existing typed rows against canonical `sources` identity:
   - repair non-conflicting projection drift from `sources`;
   - fail startup on orphan typed rows, conflicting projection drift, or
     duplicate typed peer identity.
8. Fill typed fields:
   - `resolution_strategy`
   - `username`
   - `access_hash`
   - `avatar_cache_key`
9. Derive candidate typed peer identities and preflight duplicates by
   `(account_id, peer_kind, peer_id)` before inserting/updating
   `telegram_sources`, so diagnostics can list conflicting `source_id` values
   instead of surfacing generic SQLite constraint errors.
10. Insert or update `telegram_sources`.
11. Upsert `source_identity_repair_notes` for non-fatal gaps.
12. Backfill `sources.source_subtype` where still null.
13. Optionally mirror `sources.telegram_source_kind = source_subtype` for
   Telegram rows whose legacy field is null or empty.
14. Detect duplicates under the new canonical key and fail with a specific
    migration error before unique index creation.
15. Create `idx_sources_unique_telegram_identity` idempotently after duplicate
    detection succeeds.

### Duplicate Handling

Duplicates should be rare because existing unique indexes already constrain
Telegram rows by `account_id`, `source_type`, `telegram_source_kind`, and
`external_id`.

Still, migration must check for duplicates under the new canonical key:

```sql
SELECT account_id, source_type, source_subtype, external_id, COUNT(*) AS count
FROM sources
WHERE source_type = 'telegram'
GROUP BY account_id, source_type, source_subtype, external_id
HAVING COUNT(*) > 1;
```

If duplicates exist:

- do not delete rows automatically;
- do not pick a winner silently;
- fail startup with a message that lists conflicting `source_id` values;
- do not create `idx_sources_unique_telegram_identity` until the user or a
  manual repair path resolves the conflict.

For this app, failing fast is safer than silent merge because source deletion,
analysis scopes, saved runs, and group membership all reference `source_id`.

## Backend Design

### New Rust Types

Add a canonical source identity type:

```rust
pub(crate) struct SourceIdentity {
    pub(crate) id: i64,
    pub(crate) source_type: String,
    pub(crate) source_subtype: String,
    pub(crate) account_id: Option<i64>,
    pub(crate) external_id: String,
}
```

Add Telegram typed identity:

```rust
pub(crate) struct TelegramSourceIdentity {
    pub(crate) source_id: i64,
    pub(crate) account_id: i64,
    pub(crate) source_subtype: TelegramSourceKind,
    pub(crate) peer_kind: TelegramPeerKind,
    pub(crate) peer_id: i64,
    pub(crate) resolution_strategy: TelegramPeerResolutionStrategy,
    pub(crate) username: Option<String>,
    pub(crate) access_hash: Option<i64>,
    pub(crate) avatar_cache_key: Option<String>,
}
```

`TelegramSourceKind` can stay as the enum for Telegram subtype values, but its
error text and call sites should stop calling the value
`telegram_source_kind` in generic paths. It becomes the typed representation of
Telegram `source_subtype`.

Add helper conversion:

```rust
impl TelegramSourceKind {
    pub(crate) fn from_source_subtype(value: &str) -> AppResult<Self> { ... }
}
```

The old `parse` function can remain temporarily but should delegate to the new
name or be used only in legacy adapter code.

### Source Loaders

Split source loading by responsibility.

Generic loader:

```rust
load_source_identity(pool, source_id) -> AppResult<SourceIdentity>
```

This loader must not select `telegram_source_kind`.

Provider sync loader:

```rust
load_source_for_sync(pool, source_id) -> AppResult<ProviderSyncTarget>
```

Possible enum:

```rust
pub(crate) enum ProviderSyncTarget {
    Telegram(TelegramSyncTarget),
    Youtube(YoutubeSyncTarget),
}
```

For a smaller first implementation, keep `SourceSyncTarget` but remove its
normal-path dependency on `telegram_source_kind`:

- make `source_subtype: String`, not `Option<String>`;
- either remove `telegram_source_kind`, or keep it as a temporary
  compatibility field populated from canonical `source_subtype` until Phase 4
  moves sync, Takeout, topics, and avatar refresh to typed identity;
- add a separate `load_telegram_source_identity` call inside Telegram sync and
  Takeout.

The enum is cleaner. The smaller step has lower blast radius. Either is
acceptable if tests prove generic code no longer reads the legacy column.

### Add Telegram Source

Current behavior:

- resolve Telegram peer;
- encode peer identity into `sources.metadata_zstd`;
- insert into `sources`;
- conflict on `telegram_source_kind`.

Target behavior:

1. Resolve Telegram peer.
2. Create/update `sources` using canonical conflict:

```sql
ON CONFLICT(account_id, source_type, source_subtype, external_id)
WHERE source_type = 'telegram'
DO UPDATE SET ...
```

3. Mirror `telegram_source_kind = source_subtype` only for compatibility.
4. Upsert `telegram_sources` with typed identity.
5. Store `sources.metadata_zstd` only as optional archival/debug metadata, not
   as the required peer-resolution source.

Conflict update should keep current behavior:

- title refreshes;
- `is_member` refreshes;
- account remains scoped;
- source remains active;
- avatar cache key refreshes when a new avatar is available.

### List Sources

`list_sources` should select `sources.source_subtype` as the canonical subtype.

For Telegram display fields:

- username comes from `telegram_sources.username`;
- avatar cache key comes from `telegram_sources.avatar_cache_key`;
- `telegram_source_kind` may still be serialized temporarily as
  `Some(source_subtype)` for Telegram rows, but the Rust mapping should make it
  obvious this is compatibility output.

Frontend should map:

```ts
sourceSubtype: source.source_subtype ?? null
```

The fallback:

```ts
source.source_subtype ?? source.telegram_source_kind ?? null
```

should move to a dedicated legacy adapter test or be removed once backend
guarantees `source_subtype` for all returned sources.

### Telegram Peer Resolution

Current normal path:

- decode `sources.metadata_zstd`;
- derive peer resolution plan;
- try username, stored identity, dialog scan;
- update avatar by rewriting metadata blob.

Target normal path:

- load `TelegramSourceIdentity`;
- build peer resolution plan from typed fields;
- use `peer_kind`, `peer_id`, and `access_hash` to build `PeerRef` directly
  where possible;
- use username fallback when a canonical username is present;
- use dialog scan as a normal typed fallback for partial identities;
- refresh avatar cache key in `telegram_sources`, not in
  `sources.metadata_zstd`.

Typed identity presence does not guarantee direct peer construction. It means
normal peer resolution has all durable identity data outside the compressed
metadata blob. For partial identities, normal resolution may still use username
or dialog scan. It must not decode `sources.metadata_zstd` except in the legacy
startup repair path that repairs missing typed identity or non-conflicting
projection drift.

Runtime Telegram peer resolution must not perform legacy metadata repair. If a
`telegram_sources` row is missing after startup repair succeeded, or if typed
identity violates the invariants above, the command must fail with a typed
internal/validation error.

Startup repair fallback:

- if `telegram_sources` row is missing, decode metadata with the old decoder;
- insert typed identity if enough information exists;
- continue with typed path;
- otherwise fail startup unless the row qualifies as a non-fatal enrichment gap.

This preserves old databases during startup repair while letting well-formed
rows avoid compressed metadata decoding during normal sync.

### Takeout Import

Takeout currently accepts and passes around `telegram_source_kind`. After this
slice:

- Takeout source loading should use `TelegramSourceIdentity.source_subtype`.
- Functions may still use the variable name `telegram_source_kind` internally
  during the transition, but signatures should move toward
  `telegram_source_subtype` or `TelegramSourceKind`.
- Validation should parse from `source_subtype`.
- Export DC selection and pagination split selection should not require the
  legacy column.

Target behavior must remain:

- supported source kinds: channel, supergroup, group;
- supergroup-specific migrated-history policy stays unchanged;
- export DC and only-my-messages fallback warnings stay unchanged.

### Telegram Topics

Topic refresh currently checks `source.telegram_source_kind == supergroup`.

Target behavior:

- check typed Telegram subtype from `TelegramSourceIdentity`;
- do not read `sources.telegram_source_kind`;
- topic membership materialization remains out of scope.

### Avatar Cache

Current cache key format:

```text
{account_id}_{telegram_source_kind}_{external_id}.jpg
```

This can remain for compatibility, but creation should pass canonical
`source_subtype`.

`telegram_sources.avatar_cache_key` becomes the active field. Existing
`sources.metadata_zstd.avatar_cache_key` is read only as a migration fallback.
YouTube and other non-Telegram legacy rows with `telegram_source_kind = ''`
never created Telegram avatar cache keys, so this slice should not create or
repair avatar rows for them.

New avatar refreshes must write only `telegram_sources.avatar_cache_key`.
`sources.metadata_zstd` is not the source of truth after repair, and the cache
key format remains a compatibility detail rather than identity.

### YouTube Source Upsert

Current YouTube upsert explicitly writes `telegram_source_kind = ''`.

Target behavior for this slice:

- isolate that legacy write inside a helper or compatibility insert expression;
- keep YouTube business logic expressed only in terms of:
  - `source_type = 'youtube'`
  - `source_subtype = 'video' | 'playlist'`
  - `external_id`
  - typed YouTube metadata as currently stored.

No YouTube typed source table is required in this first slice. A later slice can
move YouTube metadata out of `sources.metadata_zstd`.

## API And Frontend Design

### Backend DTO

Short-term DTO:

```rust
pub struct SourceRecord {
    pub id: i64,
    pub source_type: String,
    pub source_subtype: String,
    pub telegram_source_kind: Option<String>,
    ...
}
```

`source_subtype` should become non-optional for backend DTOs returned by
`list_sources` and `load_source_record`.

`telegram_source_kind` remains optional and deprecated:

- `Some(source_subtype)` for Telegram only;
- `None` for YouTube and future providers.

During this compatibility window:

- backend `source_subtype` is authoritative;
- `telegram_source_kind` is emitted only as a mirror for Telegram rows;
- frontend code must not use `telegramSourceKind` for capability decisions;
- tests for new behavior should assert `sourceSubtype`, not the deprecated
  mirror;
- removing the mirror is a separate compatibility-breaking cleanup.

Long-term DTO:

```rust
pub struct SourceRecord {
    pub id: i64,
    pub source_type: String,
    pub source_subtype: String,
    ...
}
```

Removal of `telegram_source_kind` from the wire contract is a follow-up after
frontend and tests no longer depend on it.

### Frontend Types

Move frontend `Source` usage toward:

- `sourceType`
- `sourceSubtype`

Keep `telegramSourceKind` only where UI copy or Telegram-specific controls have
not yet been updated. Prefer a local helper:

```ts
function telegramSubtype(source: Source): TelegramSourceKind | null
```

The helper should use `source.sourceSubtype` and validate that it is one of the
Telegram subtype values. This prevents the old field from leaking into generic
source UI.

### Compatibility Window

During transition:

- backend may still emit `telegram_source_kind`;
- frontend may keep a fallback adapter for old command mocks;
- new tests should assert `sourceSubtype` is enough for current behavior;
- old tests that explicitly require `telegramSourceKind` should be rewritten or
  scoped as legacy compatibility tests.

## Testing Strategy

### Migration Tests

Add tests that build representative old schemas/data and run the new migration
or upgrade repair.

Cases:

1. Fresh install through all migrations:
   - `sources` exists;
   - `telegram_sources` exists;
   - new indexes exist;
   - no migration checksum repair is needed.
2. Existing v17 Telegram channel:
   - `source_subtype = channel`;
   - `telegram_sources` row exists;
   - username/access hash/avatar key are decoded from metadata.
3. Existing v17 Telegram supergroup:
   - peer kind is `channel`;
   - subtype is `supergroup`;
   - access hash is preserved when present.
4. Existing v17 Telegram small group:
   - peer kind is `chat`;
   - access hash absence is accepted;
   - dialog strategy is preserved.
5. Existing old metadata payload:
   - `username` only backfills `resolution_strategy = username`;
   - legacy `added_from = dialog` only derives `resolution_strategy = dialog`;
   - `access_hash` backfills stored peer identity where applicable.
6. Telegram row with null `source_subtype`:
   - backfilled from valid `telegram_source_kind`.
7. Telegram row with invalid subtype and valid legacy kind:
   - repaired to legacy kind.
8. Malformed Telegram row:
   - migration does not delete it;
   - startup repair fails with typed, redacted diagnostics;
   - no repair note downgrade is allowed.
9. YouTube video/playlist rows:
   - unaffected;
   - existing unique indexes still work;
   - upsert still returns existing id.
10. Duplicate canonical Telegram identity:
    - startup fails with typed, redacted diagnostics listing conflicting
      `source_id` values;
    - no repair note downgrade is allowed;
    - no source row is silently merged.
11. Telegram row with `account_id IS NULL`:
    - startup repair fails with typed, redacted diagnostics;
    - no `telegram_sources` row is created;
    - no canonical unique index is created.
12. Existing `telegram_sources` row drift:
    - non-conflicting projection drift is repaired from `sources`;
    - orphan typed rows, conflicting projection drift, and duplicate typed peer
      identities fail startup.
13. Non-fatal enrichment gap:
    - repair records `source_identity_repair_notes`;
    - source remains listable;
    - sync may continue only if typed identity supports a defined fallback path.
14. Telegram `external_id` malformed forms:
    - `+123`, `-123`, `00123`, `123 `, and `12a3` fail exact round-trip
      validation with typed, redacted diagnostics;
    - no typed row or canonical index is created.
15. Existing `telegram_sources` row with subtype/peer-kind mismatch:
    - startup repair fails with typed, redacted diagnostics;
    - no correction is guessed from the typed row.
16. Duplicate typed peer identity:
    - repair detects conflicting `(account_id, peer_kind, peer_id)` candidates
      before upserting `telegram_sources`;
    - diagnostics list conflicting `source_id` values.

### Rust Unit Tests

Add focused unit tests for:

- `TelegramSourceKind::from_source_subtype`;
- `TelegramPeerKind` conversion from subtype;
- typed peer resolution plan;
- metadata fallback to typed identity;
- avatar cache key update in `telegram_sources`;
- generic `SourceRecord` mapping hiding compatibility fields for non-Telegram
  providers.

### Backend Integration Tests

Target commands and flows:

- `add_telegram_source` writes `source_subtype` and `telegram_sources`;
- adding the same Telegram source twice updates the same `source_id` through
  canonical conflict;
- `list_sources` returns non-null `source_subtype`;
- `sync_source` for Telegram resolves peer without requiring
  `sources.metadata_zstd` when typed identity exists;
- `sync_source` for YouTube still routes by `source_subtype`;
- Takeout import loads source subtype from typed identity;
- topic refresh uses subtype from typed identity;
- delete source cascades `telegram_sources`.

### Frontend Tests

Target files:

- `src/lib/api/sources.test.ts`
- source capability tests that currently use `telegramSourceKind`
- analysis source selection tests if they depend on the legacy field.

Assertions:

- `mapSource` uses `source_subtype` as canonical;
- Telegram controls still appear for channel/supergroup/group;
- YouTube controls do not require or receive `telegram_source_kind`;
- compatibility fallback is either removed or covered in one explicit legacy
  adapter test.

### Manual Smoke Tests

After implementation, smoke the app with an existing database if possible:

1. Start app.
2. Confirm existing Telegram sources list with subtype, username, avatar.
3. Add a Telegram channel by username.
4. Add a Telegram supergroup/group from dialog.
5. Sync each Telegram subtype.
6. Run Takeout preflight/import for each supported subtype where credentials
   allow.
7. Add a YouTube video.
8. Add a YouTube playlist.
9. Open `/analysis` and confirm source selection still works.
10. Delete a source and confirm provider identity row is removed.

## Implementation Sequence

### Phase 1: Schema Bridge

Files likely touched:

- `src-tauri/migrations/18.sql`
- `src-tauri/src/migrations.rs`
- `src-tauri/src/sources/test_support.rs`
- migration tests in `src-tauri/src/migrations.rs` or a new test module.

Tasks:

1. Add `telegram_sources` table.
2. Add indexes.
3. Backfill simple subtype fields.
4. Register migration.
5. Add tests that the migration is included.

Exit criteria:

- fresh migration list includes v18;
- schema can be created from empty DB;
- no normal code paths changed yet.

### Phase 2: Typed Identity Backfill

Files likely touched:

- `src-tauri/src/sources/identity_migration.rs`
- `src-tauri/src/sources/mod.rs`
- `src-tauri/src/migrations.rs` or startup database prepare path.

Tasks:

1. Add idempotent Rust repair.
2. Decode old metadata with existing decoder.
3. Insert/update `telegram_sources`.
4. Add duplicate detection.
5. Add tests for old metadata shapes.

Exit criteria:

- existing v17-style rows get typed identity;
- malformed rows fail clearly;
- repair can run twice without changing valid rows.

### Phase 3: Source Store Switch

Files likely touched:

- `src-tauri/src/sources/types.rs`
- `src-tauri/src/sources/store.rs`
- `src-tauri/src/sources/peer_resolution.rs`
- `src-tauri/src/sources/avatar.rs`
- `src/lib/api/sources.ts`
- `src/lib/api/sources.test.ts`

Tasks:

1. Add typed identity loaders.
2. Change Telegram add/upsert conflict target to canonical subtype.
3. Write `telegram_sources` during add/update.
4. Read username/avatar from typed table.
5. Make `source_subtype` non-optional for returned source records where
   possible.
6. Keep compatibility serialization of `telegram_source_kind` only at DTO
   edge.
7. Keep any temporary `SourceSyncTarget.telegram_source_kind` value derived
   from canonical `source_subtype`, not from `sources.telegram_source_kind`.

Exit criteria:

- add/list source tests pass;
- YouTube upsert tests pass;
- generic source mapping does not require legacy Telegram field.
- `load_source_for_sync` either no longer exposes `telegram_source_kind`, or
  exposes it only as a temporary compatibility copy of `source_subtype` for
  Phase 4 consumers.

### Phase 4: Sync, Takeout, Topics

Files likely touched:

- `src-tauri/src/sources/sync.rs`
- `src-tauri/src/sources/peer_resolution.rs`
- `src-tauri/src/sources/topics.rs`
- `src-tauri/src/takeout_import/mod.rs`
- `src-tauri/src/takeout_import/pagination.rs`
- `src-tauri/src/takeout_import/export_dc.rs`

Tasks:

1. Resolve Telegram peers from `telegram_sources`.
2. Refresh avatar cache key in typed table.
3. Pass `TelegramSourceKind` / source subtype through Takeout.
4. Route topic refresh by typed subtype.
5. Keep metadata fallback only for missing typed rows.
6. Remove the temporary normal-path dependency on
   `SourceSyncTarget.telegram_source_kind`, if Phase 3 kept it for sequencing.

Exit criteria:

- Telegram sync tests pass;
- Takeout source-kind tests pass;
- topic tests pass;
- normal path no longer decodes `sources.metadata_zstd` for peer identity when
  typed identity exists.

### Phase 5: Compatibility Containment

Files likely touched:

- `src-tauri/src/sources/store.rs`
- `src-tauri/src/sources/types.rs`
- `src-tauri/src/youtube/*`
- `src/lib/api/sources.ts`
- docs.

Tasks:

1. Move all direct `telegram_source_kind` reads into a legacy adapter or tests.
2. Ensure YouTube code does not branch on or expose Telegram legacy field.
3. Update docs to say `telegram_source_kind` is deprecated compatibility state.
4. Add code comments only at compatibility boundaries.

Exit criteria:

- `rg "telegram_source_kind" src-tauri/src src/lib` shows only:
  - migration/backfill code;
  - legacy DTO compatibility edge;
  - tests explicitly marked legacy;
  - old migration SQL;
  - unavoidable temporary insert mirror.

## Acceptance Criteria

The slice is complete when all of these are true:

1. New and upgraded Telegram rows have canonical `source_subtype` and a
   `telegram_sources` row.
2. Migration never recreates `sources` rows with new ids.
3. Telegram add/upsert conflict handling uses
   `(account_id, source_type, source_subtype, external_id)`.
4. Creating or updating a Telegram source updates `sources`,
   `telegram_sources`, and the legacy mirror in one transaction.
5. Telegram sync and Takeout read subtype/peer identity from typed storage in
   the normal path.
6. `sources.metadata_zstd` is no longer required for normal Telegram peer
   resolution when `telegram_sources` is present, except as a startup/legacy
   repair fallback for missing typed identity or non-conflicting projection
   drift.
7. YouTube video and playlist upserts still work and do not expose
   Telegram-specific fields outside a compatibility insert boundary.
8. `list_sources` returns canonical non-null `source_subtype` for implemented
   providers.
9. Frontend source mapping no longer needs `telegram_source_kind` for normal
   current backend responses.
10. Existing databases upgrade without losing source ids.
11. Fresh installs create the new bridge schema while still replaying legacy
    migrations until the later current-schema baseline work.
12. Migration/repair code is idempotent and transactional.
13. Duplicate canonical identities, duplicate typed peer identities, and valid
    but conflicting `source_subtype` / `telegram_source_kind` rows fail startup
    with typed, redacted diagnostics.
14. No list, sync, add, or Takeout source command can run against the database
    until source identity repair has succeeded or failed with a typed startup
    error.
15. Runtime Telegram peer resolution never decodes `sources.metadata_zstd` to
    repair missing or invalid typed identity after the startup repair gate.
16. `telegram_sources.source_subtype` and `telegram_sources.peer_kind` always
    satisfy the subtype-to-peer-kind invariant.

## Risks And Mitigations

### Risk: SQLite Cannot Decode Metadata In SQL

Mitigation:

- use SQL only for table/index creation and simple subtype backfill;
- use Rust repair for zstd JSON decoding.

### Risk: Duplicate Canonical Telegram Rows

Mitigation:

- detect before relying on the new unique index;
- fail startup with typed, redacted diagnostics listing conflicting `source_id`
  values;
- never delete, merge, or downgrade to repair notes in this slice.

### Risk: Existing Tests Encode Legacy Shape

Mitigation:

- split tests into current behavior and legacy compatibility;
- keep legacy tests only around explicit compatibility boundaries;
- require new tests to use `source_subtype`.

### Risk: Takeout Behavior Regresses

Mitigation:

- keep `TelegramSourceKind` enum values unchanged;
- change data source, not Takeout source-kind semantics;
- run targeted Takeout tests before commit.

### Risk: Avatar Cache Paths Change

Mitigation:

- preserve existing cache key format;
- move active cache key storage to `telegram_sources`;
- read old metadata avatar key only during backfill.

### Risk: Frontend Mocks Lag Behind Backend DTO

Mitigation:

- update `src/lib/api/sources.test.ts`;
- keep one legacy fallback test if needed;
- document `telegramSourceKind` as deprecated in TS type comments if it remains.

## Documentation Updates

Update after implementation:

- `docs/database-schema.md`
  - add `telegram_sources`;
  - mark `telegram_source_kind` as deprecated compatibility state;
  - document new Telegram uniqueness.
- `docs/architecture-deep-dive.md`
  - explain source identity boundary.
- `docs/backlog.md`
  - mark this first source identity slice as completed or split follow-ups.
- `docs/database-schema-legacy-analysis.md`
  - add a note that the source identity cleanup slice has a design/spec.

## Follow-Up Refactors Enabled

After this slice, the next schema simplification tasks become more tractable:

1. Item/document identity cleanup:
   - provider-native item identity;
   - `(source_id, item_kind, external_id)` or typed child identities;
   - document/corpus table for analysis and NotebookLM export.
2. Analysis snapshot hardening:
   - non-null provider/document fields for new runs;
   - snapshot persistence before provider execution.
3. Telegram topic membership materialization:
   - `item_topic_memberships`;
   - update during sync, Takeout import, topic refresh.
4. YouTube source metadata normalization:
   - typed `youtube_videos` / `youtube_playlists`;
   - fewer `sources.metadata_zstd` decodes.
5. Current-schema baseline:
   - fresh installs without historical table scars;
   - legacy migrations quarantined for upgrade only.

## Open Decisions For Implementation Planning

These are intentionally left for the implementation plan, not for design
approval:

1. Whether `load_source_for_sync` should become a provider enum immediately or
   whether `SourceSyncTarget` should be narrowed first.

Recommended choices:

1. Narrow `SourceSyncTarget` first, then introduce a provider enum if the diff
   stays manageable.
2. Let Rust repair perform typed backfill because it can parse ids, decode zstd
   metadata safely, and create the canonical Telegram unique index after
   duplicate preflight.
