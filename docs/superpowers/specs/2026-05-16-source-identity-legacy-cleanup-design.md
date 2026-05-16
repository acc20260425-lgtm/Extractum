# Source Identity Legacy Cleanup Design

> Date: 2026-05-16
> Status: approved design direction
> Scope: full cleanup of the `telegram_source_kind` compatibility window

## Summary

This design removes the legacy Telegram subtype mirror from normal source
identity. The previous source identity slice made `sources.source_subtype` and
`telegram_sources` canonical while keeping `sources.telegram_source_kind` as a
temporary compatibility bridge. This slice closes that bridge.

The implementation should add a safe SQLite migration that rebuilds `sources`
without `telegram_source_kind`, then remove the field from backend DTOs,
frontend types, live Telegram dialog DTOs, command payloads, runtime queries,
and normal tests. Historical migration files and upgrade fixtures may still
mention `telegram_source_kind`; normal product code must not.

## Reading This Spec

The normative contract sections define what must be true after the slice:

- supported upgrade matrix;
- identity invariants;
- post-v19 `sources` schema;
- database migration contract;
- backend API/runtime contract;
- repair/integrity behavior;
- frontend API/UI contract;
- documentation contract;
- completion criteria.

Implementation hints and verification commands are intentionally separated near
the end. They guide the implementation plan but are not an alternative source of
truth for the product/data contract above.

## Accepted Decisions

- Add a safe `v19` migration instead of rewriting old migration files.
- The local project database has already applied migration 18 and the source
  identity repair index exists. This slice may assume databases have passed
  the v18 compatibility window before v19 removes the legacy column. This is a
  supported upgrade precondition, not an implicit repair fallback.
- Fresh installs still run the full migration chain, but the final schema after
  migration 19 must not contain `sources.telegram_source_kind`.
- Do not provide wire aliases for old payload names. The desktop frontend and
  backend ship together, so the API can move directly from
  `telegram_source_kind`/`telegramSourceKind` to
  `source_subtype`/`sourceSubtype`.
- Rename the live Telegram dialog DTO as well. Live dialog classification should
  use `source_subtype`/`sourceSubtype`, not `telegram_source_kind`.
- Keep startup repair command and state names. They remain useful as a source
  identity integrity gate even after the legacy mirror is gone.
- Do not clean up Telegram `sources.metadata_zstd` payloads in this slice,
  except for code changes required by removing `telegram_source_kind`.
- Allow quarantined legacy references only in old migrations, upgrade tests,
  old-schema fixtures, and docs that describe migration history.

## Supported Upgrade Matrix

Migration 19 runs before startup repair in the current Tauri SQL plugin flow.
Because of that ordering, migration 19 cannot depend on repair reading the
legacy column after the migration has dropped it.

Supported scenarios:

| Scenario | Expected behavior |
| --- | --- |
| Fresh install through migrations 1 to 19 | Succeeds. The final `sources` schema has no `telegram_source_kind`. Startup repair then runs against an empty or canonical database. |
| Existing database at version 18 with startup repair already successful | Succeeds. Migration 19 rebuilds `sources`, preserves ids, and recreates current indexes without the legacy column. |
| Existing database at version 18 with no fatal source identity repair findings | Supported if canonical `sources` rows and typed `telegram_sources` rows already satisfy the invariants in this spec. Migration 19 must preserve that state. |
| Existing database at version 18 where startup repair never ran or previously failed | Unsupported for this slice. Migration 19 may fail fast through schema constraints or index creation if canonical identity is incomplete or duplicated. The user should run a build containing the v18 repair first or restore a repaired database. |
| Existing database before version 18 upgrading directly to this version | Unsupported for this slice when Telegram rows exist. Migrations 18 and 19 would run in one SQL batch before Rust startup repair can use the legacy mirror. |
| Database with Telegram rows whose `account_id`, `source_subtype`, or `external_id` is invalid | Invalid database state. Migration tests and repair tests must cover failure or blocking behavior; implementation must not silently coerce or invent identity values. |

The implementation plan should include explicit tests for the supported
fresh-install and repaired-v18 paths. Direct pre-v18 upgrade may be documented
as unsupported rather than papered over with a new SQL fallback.

Failure behavior:

- migration 19 runs in one transaction or otherwise guarantees that a failed
  rebuild leaves the previous schema/data intact;
- if migration 19 fails, application startup must fail before source commands
  become available;
- if migration 19 succeeds but the repair/integrity gate fails, source commands
  remain blocked with the typed source identity repair error;
- user-facing diagnostics should identify source identity migration or repair
  failure without exposing raw compressed metadata payloads.

## Goals

1. Remove `telegram_source_kind` from the current `sources` schema.
2. Preserve all existing source ids and source data through the `v19` table
   rebuild.
3. Remove persisted-source `telegram_source_kind` from backend DTOs and
   frontend `Source` types.
4. Rename live Telegram dialog subtype fields to `source_subtype` and
   `sourceSubtype`.
5. Rename add-source command inputs from expected kind to expected subtype.
6. Keep source commands gated by source identity repair state.
7. Keep the repair engine as an integrity check over canonical
   `sources.source_subtype` plus typed `telegram_sources`.
8. Update current-state docs and prune backlog so only open follow-ups remain.

## Non-Goals

- Do not squash or rewrite migrations 1 through 18.
- Do not remove historical migration checksum repair logic.
- Do not remove or migrate Telegram `sources.metadata_zstd` display/avatar
  payloads.
- Internal Rust names such as `TelegramSourceKind` are not part of the
  public/database compatibility contract. The required boundary is DB, API,
  DTO, and frontend vocabulary. Internal names may remain unless a small local
  rename makes the implementation clearer.
- Do not change YouTube typed metadata storage in this slice.
- Do not refactor item/document identity in this slice.

## Current State

The current codebase is on `main` at the completed source identity bridge
implementation. Migration 18 creates `telegram_sources` and
`source_identity_repair_notes`, backfills `source_subtype` from the legacy
mirror where safe, and leaves `telegram_source_kind` in `sources`.

The startup repair engine currently:

- loads Telegram rows from `sources`;
- compares `source_subtype` with `telegram_source_kind`;
- decodes legacy `metadata_zstd` to build typed Telegram identity;
- upserts `telegram_sources`;
- writes both `source_subtype` and `telegram_source_kind`;
- creates `idx_sources_unique_telegram_identity`;
- blocks source commands until repair is ready.

The frontend currently still exposes `telegramSourceKind` on persisted
`Source` objects, and live Telegram dialog rows also use `telegramSourceKind`
for peer classification. The next implementation must separate those concepts
by using only `sourceSubtype` everywhere in frontend source flows.

## Identity Invariants

The current source identity boundary after this slice is:

- generic provider identity lives in `sources`;
- Telegram operational peer identity lives in `telegram_sources`;
- `sources.telegram_source_kind` no longer exists in the current schema.

For persisted Telegram sources:

- `source_type` must be `telegram`;
- `source_subtype` must be one of `channel`, `supergroup`, or `group`;
- `account_id` must be non-`NULL`;
- `external_id` must be the canonical decimal Telegram peer id;
- `telegram_sources.source_id` must match `sources.id`;
- `telegram_sources.account_id` must match `sources.account_id`;
- `telegram_sources.source_subtype` must match `sources.source_subtype`;
- `telegram_sources.peer_id` must match `sources.external_id` parsed as the
  canonical Telegram peer id;
- usernames are resolution hints and must not be stored as
  `sources.external_id`.

The canonical Telegram `external_id` format is the exact decimal string form of
the bare Telegram peer id. It has no provider prefix, no `@`, no username, no
sign, no surrounding whitespace, no non-digit characters, and no leading zeroes
except the literal string `0`. It must round-trip through the existing
`canonical_telegram_external_id` helper and back to the same string. Channels,
supergroups, and groups all use this same durable `external_id` format; their
shape difference is represented by `source_subtype` and by
`telegram_sources.peer_kind`.

`account_id` is part of Telegram identity, not optional display metadata.
SQLite unique indexes do not protect identity uniqueness when any indexed
column is `NULL`, so this invariant must be enforced in all of these places:

- migration 19 final schema contract;
- startup repair validation;
- Telegram add-source request handling and source upsert;
- migration and repair regression tests.

For YouTube sources:

- `source_type` must be `youtube`;
- `source_subtype` must be `video` or `playlist`;
- `external_id` is the provider id: video id for videos, playlist id for
  playlists;
- `account_id` is currently `NULL`.

RSS and forum remain provider-model placeholders. The schema should not add
hard database checks that prevent future placeholder source types or subtypes
unless those checks are explicitly scoped to implemented providers.

## Post-v19 Sources Schema Contract

Migration 19 must leave `sources` with this exact current-schema contract:

| Column | Type | Nullability / default | Meaning |
| --- | --- | --- | --- |
| `id` | `INTEGER` | `PRIMARY KEY AUTOINCREMENT` | Stable source id. Must be preserved across the table rebuild. |
| `source_type` | `TEXT` | `NOT NULL` | Provider family such as `telegram` or `youtube`. |
| `source_subtype` | `TEXT` | nullable | Provider-local subtype. Required by application invariants for implemented Telegram and YouTube rows. |
| `external_id` | `TEXT` | `NOT NULL` | Provider-native durable id. For Telegram, canonical decimal peer id. |
| `title` | `TEXT` | nullable | Display title. |
| `metadata_zstd` | `BLOB` | nullable | Provider metadata payload retained by this slice. |
| `last_sync_state` | `INTEGER` | nullable | Provider sync cursor. |
| `is_active` | `BOOLEAN` | default `1` | Source is active unless disabled/deleted by product flow. |
| `is_member` | `BOOLEAN` | default `0` | Membership/subscription flag where applicable. |
| `created_at` | `INTEGER` | `NOT NULL` | Unix timestamp, UTC. |
| `account_id` | `INTEGER` | nullable, `REFERENCES accounts(id) ON DELETE CASCADE` | Local account owner. Required for Telegram rows. |
| `last_synced_at` | `INTEGER` | nullable | Timestamp of last successful sync/import. |

`sources` must not contain `telegram_source_kind` after migration 19.

Post-v19 `sources` indexes:

- `idx_sources_unique_telegram_identity`
  ```sql
  CREATE UNIQUE INDEX IF NOT EXISTS idx_sources_unique_telegram_identity
      ON sources(account_id, source_type, source_subtype, external_id)
      WHERE source_type = 'telegram';
  ```
- `idx_sources_unique_youtube_video`
  ```sql
  CREATE UNIQUE INDEX IF NOT EXISTS idx_sources_unique_youtube_video
      ON sources(source_type, source_subtype, external_id)
      WHERE source_type = 'youtube' AND source_subtype = 'video';
  ```
- `idx_sources_unique_youtube_playlist`
  ```sql
  CREATE UNIQUE INDEX IF NOT EXISTS idx_sources_unique_youtube_playlist
      ON sources(source_type, source_subtype, external_id)
      WHERE source_type = 'youtube' AND source_subtype = 'playlist';
  ```

The old `idx_sources_ext` index must not exist after migration 19 because it is
built on `telegram_source_kind`.

Post-v19 `sources` constraints:

- preserve `id INTEGER PRIMARY KEY AUTOINCREMENT`;
- preserve `account_id REFERENCES accounts(id) ON DELETE CASCADE`;
- preserve existing nullability and defaults listed above;
- add a table-level Telegram check:
  ```sql
  CHECK (
      source_type <> 'telegram'
      OR (
          account_id IS NOT NULL
          AND source_subtype IN ('channel', 'supergroup', 'group')
      )
  )
  ```
- add a table-level YouTube check:
  ```sql
  CHECK (
      source_type <> 'youtube'
      OR source_subtype IN ('video', 'playlist')
  )
  ```

These checks are scoped to implemented provider rows so future RSS/forum
placeholder rows are not blocked by a Telegram/YouTube-only enum. The canonical
Telegram `external_id` string format remains enforced in Rust repair/add-source
validation and regression tests rather than with SQLite string-pattern checks.

## Database Migration

Add `src-tauri/migrations/19.sql` and register it in
`src-tauri/src/migrations.rs` after version 18.

SQLite cannot drop a column in a way that preserves all constraints and indexes
across the supported environment, so migration 19 should rebuild `sources`:

1. Create `sources_new` using the exact post-v19 schema contract above.
2. Copy every row from `sources` into `sources_new`, preserving `id`,
   `source_type`, `source_subtype`, `account_id`, `external_id`, title,
   metadata, sync state, active/member flags, and timestamps.
3. Drop indexes that depend on the old table shape.
4. Drop the old `sources` table.
5. Rename `sources_new` to `sources`.
6. Recreate the post-v19 indexes listed above.

Migration 19 must not use `telegram_source_kind` as a fallback source of truth.
If a Telegram row reaches v19 without a valid `source_subtype`, that is a data
integrity problem surfaced by the repair gate or migration tests, not a new SQL
compatibility rule.

Fresh-install migration tests should apply all migrations and assert that:

- `sources` exists;
- `telegram_sources` exists;
- `source_identity_repair_notes` exists;
- `sources.telegram_source_kind` does not exist;
- `idx_sources_unique_telegram_identity` exists.
- `idx_sources_ext` does not exist.
- inserting a Telegram source with `NULL account_id` fails;
- inserting a Telegram source with an unsupported `source_subtype` fails;
- inserting a YouTube source with an unsupported `source_subtype` fails;
- inserting an RSS/forum placeholder row with a provider-local subtype is still
  allowed by the database checks.

Upgrade-style tests should construct a v18-shaped schema with source rows and
typed Telegram rows, run migration 19, and assert that:

- source ids are unchanged;
- Telegram typed identity rows still point to the same `source_id`;
- YouTube source ids and uniqueness remain stable;
- the legacy column is gone.
- the foreign-key and logical-reference graph still resolves to the same source
  ids.

The source-id graph that must be preserved includes physical foreign keys:

- `items.source_id`;
- `analysis_source_group_members.source_id`;
- `source_identity_repair_notes.source_id`;
- `telegram_forum_topics.source_id`;
- `telegram_sources.source_id`;
- `youtube_playlist_items.playlist_source_id`;
- `youtube_playlist_items.video_source_id`;
- `youtube_transcript_segments.source_id`.

It also includes logical references without a current SQLite foreign key:

- `analysis_runs.source_id`;
- `analysis_run_messages.source_id`.

Acceptance for v19 is stronger than “the migration succeeds”: every existing
row that referenced `sources(id)` before v19 must still reference an existing
source with the same id and semantic identity after v19.

## Backend API And Runtime Contract

Persisted source DTOs must expose only canonical `source_subtype` as source
subtype identity. They must not emit `telegram_source_kind` or
`telegramSourceKind`.

Normal runtime source queries must select `source_subtype` directly and must
not select `telegram_source_kind`. Source inserts and upserts must not write
`telegram_source_kind`.

Live Telegram dialog DTOs should be renamed from `telegram_source_kind` to
`source_subtype`. The classification still has the same values:

- `channel`
- `supergroup`
- `group`

In live Telegram dialog DTOs, `source_subtype`/`sourceSubtype` means “the
subtype that would be used if this dialog is registered as a source.” It is not
a persisted source identity until `add_telegram_source` succeeds and a
`sources` row plus typed `telegram_sources` row exist.

The add Telegram source command should accept `expected_subtype` instead of
`expected_kind`. The TypeScript API should expose `expectedSubtype` instead of
`expectedKind`. No old aliases should be accepted.

Peer-resolution helper names and error strings should move away from
`telegram_source_kind` when the value is actually a canonical source subtype.
Internal Rust names may remain historical; the compatibility boundary is the
database/API/DTO/frontend vocabulary.

## Repair And Integrity Gate

The repair API and state names remain:

- `preview_source_identity_repair`
- `get_source_identity_repair_status`
- `SourceIdentityRepairState`
- `SourceIdentityRepairReport`

The implementation should remove legacy mirror logic from the repair engine.
After migration 19, repair should:

- read Telegram rows from `sources` without `telegram_source_kind`;
- require a supported `source_subtype`;
- require non-`NULL` `account_id`;
- require canonical Telegram `external_id` as defined in this spec;
- require `sources` and `telegram_sources` account/subtype/peer identity to
  agree;
- validate duplicate canonical identity before index creation;
- validate duplicate typed peer identity;
- validate projection drift between `sources` and `telegram_sources`;
- keep dry-run behavior non-writing;
- keep source commands blocked while repair is pending/running/failed.

Repair safety cases:

| Case | Repair action |
| --- | --- |
| `telegram_sources` row exists and matches `sources.account_id`, `sources.source_subtype`, and `sources.external_id` parsed as peer id | OK. Include the source in the report as checked/repaired according to the existing report convention, but do not rewrite `sources`. |
| `telegram_sources` row is missing, and `sources` has valid Telegram `account_id`, supported `source_subtype`, and canonical `external_id` | Upsert `telegram_sources` from canonical `sources`. Derive `peer_kind` from `source_subtype`, derive `peer_id` from `external_id`, and copy username/access hash/resolution/avatar hints from `metadata_zstd` only if the metadata decodes cleanly. If metadata is absent, use `resolution_strategy = 'unknown'` and nullable hint fields. |
| `telegram_sources` row is missing and `sources` lacks `account_id`, supported `source_subtype`, or canonical `external_id` | Fatal diagnostic. Do not partially write typed identity. |
| `sources.source_subtype` conflicts with `telegram_sources.source_subtype` | Fatal projection drift diagnostic. |
| `sources.account_id` conflicts with `telegram_sources.account_id` | Fatal projection drift diagnostic. |
| `sources.external_id` parsed as peer id conflicts with `telegram_sources.peer_id` | Fatal projection drift diagnostic unless no other source can own the existing typed peer and the implementation deliberately treats this as non-conflicting repair drift. That non-conflicting drift path must be covered by a targeted test. |
| Duplicate `(account_id, source_type, source_subtype, external_id)` among Telegram sources | Fatal duplicate canonical identity diagnostic naming the source ids. |
| Duplicate `(account_id, peer_kind, peer_id)` among typed Telegram identities | Fatal duplicate typed peer identity diagnostic naming the source ids. |
| Username, access hash, resolution strategy, or avatar hint is missing or differs but account/subtype/peer identity agrees | Non-fatal. Keep the existing typed row or refresh nullable hint fields from decodable metadata; do not block startup solely because optional hints are absent. |
| `metadata_zstd` is malformed while canonical `sources` identity is valid and a matching typed row already exists | Non-fatal. Keep the typed row and record no fatal identity error. |
| `metadata_zstd` is malformed while typed row is missing | Fatal only if the implementation cannot build the required typed row from canonical `sources`; otherwise upsert required identity fields and leave optional hints empty. |

The diagnostic
`telegram_subtype_legacy_kind_conflict` should be removed from normal tests and
code. The old conflict scenario belongs to the completed compatibility-window
slice.

Using `metadata_zstd` inside repair remains allowed for transitional recovery
of username, access hash, resolution strategy, and avatar cache key. Normal
runtime peer resolution must continue reading typed Telegram identity from
`telegram_sources`, not from `metadata_zstd`.

## Frontend

Remove `telegramSourceKind` from persisted `Source`. Persisted source behavior
should use `sourceSubtype` for:

- source capabilities;
- source labels;
- membership/topic behavior;
- analysis source state;
- source-management existing-source keys;
- source fixtures in frontend tests.

Rename live Telegram dialog source shape:

- `telegramSourceKind` becomes `sourceSubtype`;
- filters and sorting use `sourceSubtype`;
- add-source calls pass `expectedSubtype`;
- rendered labels call source-kind/subtype label helpers with `sourceSubtype`.

Frontend API mapping should no longer accept or emit
`telegram_source_kind` for persisted source rows. Raw source fixtures should be
updated so a missing legacy field is normal.

## Documentation

Update `docs/database-schema.md` so `sources` no longer lists
`telegram_source_kind` as a current column. The migration history can still say
that migrations 11, 12, 15, and 18 handled the old compatibility mirror and
that migration 19 removed it.

Update `docs/architecture-deep-dive.md` so it says Telegram source subtype is
canonical in `sources.source_subtype`, operational Telegram peer identity lives
in `telegram_sources`, and the legacy mirror has been removed from the current
schema.

Update `docs/backlog.md` according to its open-work-only rule. Do not mark the
`telegram_source_kind` cleanup as complete in backlog. Remove shipped
compatibility-window cleanup entries and leave only still-open follow-ups such
as:

- move remaining Telegram display/avatar metadata out of `sources.metadata_zstd`;
- move YouTube identity/display metadata to typed source tables;
- continue item/document identity cleanup.

## Implementation Hints

Likely Rust data shapes affected by the backend API/runtime contract:

- `SourceRecord`;
- `SourceRowParts`;
- `SourceSyncTarget`;
- store query row structs;
- live Telegram dialog DTOs;
- add Telegram source request DTO;
- NotebookLM export source models where the field is only a compatibility
  mirror;
- analysis/test fixtures that build persisted source DTOs.

Likely frontend shapes affected:

- persisted `Source`;
- live `TelegramDialogSource`;
- add-source input types;
- source capability fixtures;
- analysis source state fixtures;
- source management dialog filters, sort keys, labels, and add-source payload.

These are hints for plan writing. The implementation must still be driven by
the normative contracts above and by compile/test feedback.

## Verification Appendix

Use TDD for each implementation task. Important red/green checks:

- migration registration fails before version 19 is added;
- migration schema test fails while `sources.telegram_source_kind` still exists
  after all migrations;
- store tests fail while queries select or writes bind the removed column;
- frontend API tests fail while `telegramSourceKind` remains on persisted
  `Source`;
- add-source API tests fail while the invoke payload uses `expectedKind`;
- containment scan fails until normal code removes old names.

Final verification should run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
npm.cmd test
npm.cmd run check
git diff --check
rg -n "telegram_source_kind|telegramSourceKind|expectedKind|expected_kind" src-tauri\src src\lib src\routes
```

The final scan may only match quarantined legacy zones if the command is run
against the whole repository:

Allowed matches:

- old migration SQL files: `src-tauri/migrations/11.sql`,
  `src-tauri/migrations/12.sql`, `src-tauri/migrations/15.sql`, and
  `src-tauri/migrations/18.sql`;
- migration registration and upgrade tests that assert old migrations existed
  or old schemas upgrade safely;
- old-schema test fixtures used only for upgrade/regression setup;
- docs describing migration history.

Disallowed matches:

- runtime source store, sync, peer-resolution, Takeout, topics, NotebookLM, and
  YouTube source code;
- command request/response payload structs for current APIs;
- frontend API mapping;
- persisted frontend `Source` and live dialog source types;
- UI source filtering, sorting, labeling, keying, and add-source logic;
- normal runtime/frontend tests that are not explicitly old-schema upgrade
  fixtures.

Normal runtime modules, frontend source APIs, and UI components must not match.

## Risks And Mitigations

- **SQLite table rebuild can drop constraints or indexes.** Mitigate with
  migration tests that inspect columns and indexes after applying all
  migrations.
- **Foreign-key relationships can break if ids change.** Mitigate with upgrade
  tests that assert source ids survive and typed `telegram_sources.source_id`
  still points at the same rows.
- **Frontend and backend payload names can drift.** Mitigate with API tests that
  assert `expectedSubtype` is sent and `sourceSubtype` is mapped without the old
  field.
- **Repair can accidentally keep writing the removed column.** Mitigate with
  targeted repair tests and a containment scan.
- **The scope can expand into metadata cleanup.** Mitigate by explicitly
  leaving `metadata_zstd` cleanup for a later slice.

## Completion Criteria

The slice is complete when:

- migration 19 is registered and tested;
- a fresh database after all migrations has no `sources.telegram_source_kind`;
- a v18-style database upgrades through v19 without changing source ids;
- persisted source DTOs and frontend `Source` types no longer include
  Telegram legacy kind fields;
- live dialog source DTOs use `sourceSubtype`;
- add-source commands use `expectedSubtype`;
- repair remains an integrity gate and no longer reads or writes the legacy
  mirror;
- current-state docs describe the completed compatibility cleanup and backlog
  contains only open follow-ups;
- full Rust tests, frontend tests, and Svelte check pass.
