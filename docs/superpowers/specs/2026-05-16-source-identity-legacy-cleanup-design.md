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

## Accepted Decisions

- Add a safe `v19` migration instead of rewriting old migration files.
- The local project database has already applied migration 18 and the source
  identity repair index exists. This slice may assume databases have passed
  the v18 compatibility window before v19 removes the legacy column.
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
8. Update docs and backlog so the completed compatibility window is clear.

## Non-Goals

- Do not squash or rewrite migrations 1 through 18.
- Do not remove historical migration checksum repair logic.
- Do not remove or migrate Telegram `sources.metadata_zstd` display/avatar
  payloads.
- Do not rename the internal `TelegramSourceKind` enum unless it becomes a
  small, local cleanup during implementation. The public and database
  vocabulary is the important boundary.
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

## Database Migration

Add `src-tauri/migrations/19.sql` and register it in
`src-tauri/src/migrations.rs` after version 18.

SQLite cannot drop a column in a way that preserves all constraints and indexes
across the supported environment, so migration 19 should rebuild `sources`:

1. Create `sources_new` with the current intended columns, omitting
   `telegram_source_kind`.
2. Copy every row from `sources` into `sources_new`, preserving `id`,
   `source_type`, `source_subtype`, `account_id`, `external_id`, title,
   metadata, sync state, active/member flags, timestamps, and other current
   columns.
3. Drop indexes that depend on the old table shape.
4. Drop the old `sources` table.
5. Rename `sources_new` to `sources`.
6. Recreate current indexes, including:
   - canonical Telegram uniqueness on
     `(account_id, source_type, source_subtype, external_id)` where
     `source_type = 'telegram'`;
   - YouTube video and playlist partial unique indexes;
   - generic source/account indexes that are still present in the current
     schema.

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

Upgrade-style tests should construct a v18-shaped schema with source rows and
typed Telegram rows, run migration 19, and assert that:

- source ids are unchanged;
- Telegram typed identity rows still point to the same `source_id`;
- YouTube source ids and uniqueness remain stable;
- the legacy column is gone.

## Backend API And Runtime

Persisted source DTOs should expose only the canonical subtype. In Rust, remove
`telegram_source_kind` from current source data shapes:

- `SourceRecord`
- `SourceRowParts`
- `SourceSyncTarget`
- store query row structs
- NotebookLM export source models where the field is only a compatibility
  mirror
- analysis/test fixtures that build persisted source DTOs

Queries should select `source_subtype` directly and stop selecting
`telegram_source_kind`. Inserts and upserts should stop writing
`telegram_source_kind`.

Live Telegram dialog DTOs should be renamed from `telegram_source_kind` to
`source_subtype`. The classification still has the same values:

- `channel`
- `supergroup`
- `group`

The add Telegram source command should accept `expected_subtype` instead of
`expected_kind`. The TypeScript API should expose `expectedSubtype` instead of
`expectedKind`. No old aliases should be accepted.

Peer-resolution helper names and error strings should move away from
`telegram_source_kind` when the value is actually a canonical source subtype.
The internal `TelegramSourceKind` enum can remain if renaming it would add
noise without improving the boundary.

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
- require `account_id`;
- require a canonical Telegram `external_id`;
- validate duplicate canonical identity before index creation;
- validate duplicate typed peer identity;
- validate projection drift between `sources` and `telegram_sources`;
- upsert or refresh `telegram_sources` where the canonical source row and
  legacy metadata make that safe;
- keep dry-run behavior non-writing;
- keep source commands blocked while repair is pending/running/failed.

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

Update `docs/backlog.md` by marking the `telegram_source_kind` compatibility
cleanup as complete or replacing it with narrower follow-ups:

- move remaining Telegram display/avatar metadata out of `sources.metadata_zstd`;
- move YouTube identity/display metadata to typed source tables;
- continue item/document identity cleanup.

## Verification Strategy

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

- old migration SQL files;
- migration registration and upgrade tests;
- old-schema test fixtures;
- docs describing migration history.

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
- docs/backlog describe the completed compatibility cleanup;
- full Rust tests, frontend tests, and Svelte check pass.
