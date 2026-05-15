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
4. Keep fresh installs and future schema baselines free from the historical
   `telegram_source_kind NOT NULL` workaround.
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

### Deprecated Compatibility Mirror

`sources.telegram_source_kind` remains in existing databases for now.

Rules:

- normal logic must not read it as source-of-truth;
- Telegram write paths may mirror `source_subtype` into it while the old column
  remains `NOT NULL`;
- YouTube write paths may continue writing `''` only inside a compatibility
  insert helper, never as business logic;
- frontend code must prefer `source_subtype`;
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
    added_from TEXT,
    identity_refreshed_at INTEGER,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    CHECK (source_subtype IN ('channel', 'supergroup', 'group')),
    CHECK (peer_kind IN ('channel', 'chat')),
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

Indexes:

```sql
CREATE UNIQUE INDEX IF NOT EXISTS idx_telegram_sources_account_peer
    ON telegram_sources(account_id, peer_kind, peer_id);

CREATE INDEX IF NOT EXISTS idx_telegram_sources_account_subtype
    ON telegram_sources(account_id, source_subtype);

CREATE INDEX IF NOT EXISTS idx_telegram_sources_username
    ON telegram_sources(username)
    WHERE username IS NOT NULL;
```

The unique peer index is intentionally based on Telegram peer address, not
`external_id` text. It protects against duplicate typed rows while keeping
`sources.external_id` as the generic provider-native id used by existing UI and
analysis contracts.

### Source Uniqueness

Add a canonical Telegram identity index:

```sql
CREATE UNIQUE INDEX IF NOT EXISTS idx_sources_unique_telegram_identity
    ON sources(account_id, source_type, source_subtype, external_id)
    WHERE source_type = 'telegram';
```

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

### Optional Source Identity Audit Table

If migration preflight finds malformed rows that cannot be safely normalized,
record them instead of deleting data:

```sql
CREATE TABLE IF NOT EXISTS source_identity_repair_notes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_id INTEGER NOT NULL,
    issue_code TEXT NOT NULL,
    detail TEXT,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
);
```

This table is optional. The implementation can instead fail startup with a
clear validation error if unrecoverable duplicate identity rows are detected.
The important rule is: do not silently delete or merge user sources.

## Migration Strategy

Use a two-part migration:

1. SQL migration creates columns/indexes/tables and performs simple relational
   backfills.
2. Rust upgrade repair decodes compressed legacy metadata and fills typed
   Telegram fields that SQL cannot derive.

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

3. Create `telegram_sources`.

4. Insert minimal typed rows from `sources`:

```sql
INSERT OR IGNORE INTO telegram_sources (
    source_id,
    account_id,
    source_subtype,
    peer_kind,
    peer_id,
    resolution_strategy
)
SELECT
    id,
    account_id,
    source_subtype,
    CASE
        WHEN source_subtype IN ('channel', 'supergroup') THEN 'channel'
        ELSE 'chat'
    END AS peer_kind,
    CAST(external_id AS INTEGER) AS peer_id,
    'unknown'
FROM sources
WHERE source_type = 'telegram'
  AND account_id IS NOT NULL
  AND source_subtype IN ('channel', 'supergroup', 'group')
  AND external_id GLOB '-[0-9]*' = 0
  AND external_id GLOB '[0-9]*';
```

The final numeric predicate may need adjustment because SQLite glob patterns
are limited. If SQL cannot safely validate all numeric ids, do only table/index
creation in SQL and move row insertion into Rust repair.

5. Create the new Telegram source identity index.

6. Create typed-table indexes.

7. Do not alter `sources.telegram_source_kind` nullability in this slice.

### Rust Upgrade Repair

Add an explicit upgrade function, for example in a new module:

- `src-tauri/src/source_identity_migration.rs`

or a provider-specific module:

- `src-tauri/src/sources/identity_migration.rs`

The function should run during startup after SQL migrations are applied. It
must be idempotent.

Responsibilities:

1. Load Telegram `sources` rows missing `telegram_sources`.
2. Parse `sources.external_id` into `peer_id`.
3. Derive `source_subtype`:
   - prefer valid `sources.source_subtype`;
   - fallback to valid `sources.telegram_source_kind`;
   - otherwise record/fail with a clear repair error.
4. Decode `sources.metadata_zstd` with the existing compatibility decoder.
5. Fill typed fields:
   - `resolution_strategy`
   - `username`
   - `access_hash`
   - `avatar_cache_key`
   - `added_from` if still useful for provenance.
6. Insert or update `telegram_sources`.
7. Backfill `sources.source_subtype` where still null.
8. Optionally mirror `sources.telegram_source_kind = source_subtype` for
   Telegram rows whose legacy field is null or empty.
9. Detect duplicates under the new canonical key and fail with a specific
   migration error before unique index creation if the SQL migration cannot
   guarantee safety.

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
- fail startup with a message that lists conflicting `source_id` values, or
  record repair notes and leave the old index active until a manual repair path
  is implemented.

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
- remove `telegram_source_kind`;
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
- use `peer_kind`, `peer_id`, and `access_hash` to build `PeerRef` where
  possible;
- use username fallback when configured;
- use dialog scan as a repair/fallback path;
- refresh avatar cache key in `telegram_sources`, not in
  `sources.metadata_zstd`.

Legacy fallback:

- if `telegram_sources` row is missing, decode metadata with the old decoder;
- insert typed identity if enough information exists;
- continue with typed path;
- otherwise use old dialog scan and record the row for repair.

This preserves old databases while letting well-formed rows avoid compressed
metadata decoding during normal sync.

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
   - `added_from = dialog` and `access_hash` backfill dialog identity.
6. Telegram row with null `source_subtype`:
   - backfilled from valid `telegram_source_kind`.
7. Telegram row with invalid subtype and valid legacy kind:
   - repaired to legacy kind.
8. Malformed Telegram row:
   - migration does not delete it;
   - repair reports a clear error or records a repair note.
9. YouTube video/playlist rows:
   - unaffected;
   - existing unique indexes still work;
   - upsert still returns existing id.
10. Duplicate canonical Telegram identity:
    - startup fails or records repair notes;
    - no source row is silently merged.

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

Exit criteria:

- add/list source tests pass;
- YouTube upsert tests pass;
- generic source mapping does not require legacy Telegram field.

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
2. Telegram add/upsert conflict handling uses
   `(account_id, source_type, source_subtype, external_id)`.
3. Telegram sync and Takeout read subtype/peer identity from typed storage in
   the normal path.
4. `sources.metadata_zstd` is no longer required for normal Telegram peer
   resolution when `telegram_sources` is present.
5. YouTube video and playlist upserts still work and do not expose
   Telegram-specific fields outside a compatibility insert boundary.
6. `list_sources` returns canonical non-null `source_subtype` for implemented
   providers.
7. Frontend source mapping no longer needs `telegram_source_kind` for normal
   current backend responses.
8. Existing databases upgrade without losing source ids.
9. Fresh installs create the new bridge schema.
10. Migration/repair code is idempotent.

## Risks And Mitigations

### Risk: SQLite Cannot Decode Metadata In SQL

Mitigation:

- use SQL only for table/index creation and simple subtype backfill;
- use Rust repair for zstd JSON decoding.

### Risk: Duplicate Canonical Telegram Rows

Mitigation:

- detect before relying on the new unique index;
- fail clearly or record repair notes;
- never delete or merge source rows automatically.

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
2. Whether malformed source identity rows should fail startup or be recorded in
   `source_identity_repair_notes`.
3. Whether SQL migration should insert minimal `telegram_sources` rows or leave
   all typed backfill to Rust repair.
4. Whether frontend `telegramSourceKind` can be removed in the same slice or
   should stay for one release as deprecated compatibility.

Recommended choices:

1. Narrow `SourceSyncTarget` first, then introduce a provider enum if the diff
   stays manageable.
2. Fail startup on duplicate canonical identities; record notes only for
   non-fatal metadata enrichment gaps.
3. Let Rust repair perform typed backfill because it can parse ids and decode
   zstd metadata safely.
4. Keep `telegramSourceKind` in the wire DTO for this slice, but make normal
   frontend behavior depend on `sourceSubtype`.
