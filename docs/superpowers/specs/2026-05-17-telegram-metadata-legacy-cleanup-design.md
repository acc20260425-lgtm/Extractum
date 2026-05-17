# Telegram Metadata Legacy Cleanup Design

Date: 2026-05-17

## Summary

New Telegram source rows should stop writing Telegram-specific metadata into
`sources.metadata_zstd`. Existing Telegram blobs are preserved until an explicit
future cleanup. Telegram runtime identity and display cache fields already have
a typed home in `telegram_sources`; this slice makes that typed table the
normal-path source for required Telegram identity, resolution hints, and display
cache keys.

This is intentionally not a data cleanup migration. Existing Telegram
`sources.metadata_zstd` blobs may remain in older databases as legacy repair
input. YouTube source metadata remains unchanged and continues to use
`sources.metadata_zstd` until a separate YouTube typed-table design.

## Problem

The source identity cleanup removed the old `telegram_source_kind` mirror and
moved operational Telegram identity into `telegram_sources`, but the Telegram
add path still builds `SourceMetadata`, compresses it, and writes it to
`sources.metadata_zstd`. The same values are then written to
`telegram_sources`.

That leaves two issues:

- new Telegram rows still carry a legacy provider-specific blob in the generic
  `sources` table;
- future code can accidentally revive the blob path after sync, avatar refresh,
  Takeout import, repair, or repeated add/upsert.

The next slice should make the invariant explicit: normal Telegram runtime code
does not write or read `sources.metadata_zstd` for well-formed Telegram rows.

## Goals

- Keep `sources.metadata_zstd` `NULL` for newly created Telegram sources.
- Keep normal runtime updates from writing a new Telegram metadata blob. Rows
  that already have `NULL` stay `NULL`; existing legacy blobs are preserved
  until an explicit cleanup slice.
- Keep Telegram-specific data in `telegram_sources`, split by class:
  required identity, optional resolution hints, and optional display cache.
- Preserve atomicity: adding or re-adding a Telegram source either writes both
  `sources` and `telegram_sources`, or writes neither.
- Make startup repair prefer valid typed identity over legacy metadata blobs.
- Preserve legacy repair for old Telegram rows that do not yet have typed
  identity.
- Leave YouTube source metadata behavior unchanged.
- Update documentation to describe Telegram metadata blobs as legacy-only input,
  not current runtime state.

## Non-Goals

- Do not add a migration for this slice.
- Do not null out existing Telegram `sources.metadata_zstd` blobs, including
  during ordinary sync, list, Takeout, repair, and repeated add/upsert flows.
- Do not remove `sources.metadata_zstd` from the `sources` schema.
- Do not change YouTube video or playlist metadata storage.
- Do not change item/media metadata storage.

## Current State

Telegram add currently:

1. resolves a Telegram source from username or dialog input;
2. caches avatar bytes when available;
3. builds `SourceMetadata`;
4. writes compressed `SourceMetadata` into `sources.metadata_zstd`;
5. writes equivalent typed identity fields into `telegram_sources`;
6. commits both rows in one transaction.

Normal source list display already joins `telegram_sources` for Telegram
username and avatar cache key. Sync avatar refresh updates
`telegram_sources.avatar_cache_key`. Source resolution loads typed identity via
`telegram_sources`.

Startup repair still reads `sources.metadata_zstd` as compatibility input when
building repair candidates.

## Design

### Storage Invariant

For `source_type = 'telegram'`, normal runtime paths must not create, replace,
or decode Telegram metadata blobs. Existing legacy blob bytes may be preserved
unchanged.

The add/upsert flow will keep writing the generic source row, but the Telegram
metadata value bound into the source insert will be `NULL`. The upsert update
clause should leave the existing `metadata_zstd` value untouched. That means a
new source starts with `NULL`, a source that already has `NULL` stays `NULL`,
and an older source with a legacy blob keeps that blob until a dedicated cleanup
decision.

YouTube insert/upsert paths continue to encode and update
`sources.metadata_zstd`.

### Telegram Data Classes

This slice distinguishes three classes of Telegram-specific source data:

- Required identity: `source_type`, `source_subtype`, `account_id`, and
  `external_id` in `sources`, plus `source_id`, `account_id`,
  `source_subtype`, `peer_kind`, and `peer_id` in `telegram_sources`.
- Optional resolution hints: `resolution_strategy`, `username`, and
  `access_hash`.
- Optional display cache: `avatar_cache_key`.

Required identity determines whether a Telegram source row is valid enough for
normal runtime use. Missing or conflicting required identity can be fatal during
repair. Missing optional resolution hints or display cache values are valid
runtime states and must not make a row invalid.

### Typed Identity Invariant

The Telegram add flow should build a typed identity input directly from the
resolved Telegram source and add request context. It should not use a compressed
`SourceMetadata` blob as the intermediate runtime model.

The typed identity upsert must continue to populate required identity fields:

- `source_id`
- `account_id`
- `source_subtype`
- `peer_kind`
- `peer_id`

It must also populate optional resolution and display fields when available:

- `username`
- `access_hash`
- `avatar_cache_key`

`resolution_strategy` is stored in a non-null checked column. When the add,
repair, or refresh path has no more specific strategy, it must write
`unknown`, not `NULL`.

Bookkeeping fields must still be maintained:

- `identity_refreshed_at`
- `updated_at`

The helper boundary should make this intent visible: one helper creates the
source row with no Telegram metadata blob; another helper upserts
`telegram_sources` from typed fields.

### Runtime Read Invariant

Read paths are classified by purpose:

- normal runtime command reads;
- repair-time reads;
- diagnostics/debug/admin reads;
- tests and fixtures;
- migration or historical compatibility reads.

Normal runtime command reads for well-formed Telegram sources should not decode
`sources.metadata_zstd`. This includes:

- source list display;
- source resolution before sync;
- avatar refresh after sync;
- Takeout import source loading;
- forum topic gates;
- NotebookLM source loading where Telegram source identity is needed.

These paths should rely on `sources` for generic identity and
`telegram_sources` for Telegram-specific identity/display data.

`SourceSyncTarget.metadata_zstd` can remain for provider-shared structs during
this slice, but Telegram runtime code must not require it.

Compatibility reads are allowed only inside explicit repair, migration, or
diagnostic code. They must not become a fallback during normal command
execution for a valid typed row. Tests and fixtures may still construct or
decode legacy metadata when they are proving repair or compatibility behavior.

A well-formed Telegram runtime source is defined as:

- `sources.source_type = 'telegram'`;
- `sources.account_id` is present;
- `sources.source_subtype` is one of `channel`, `supergroup`, or `group`;
- `sources.external_id` is a canonical positive decimal ASCII Telegram peer id
  with no sign, whitespace, or leading zeroes;
- exactly one `telegram_sources` row exists for `sources.id`;
- `telegram_sources.account_id` equals `sources.account_id`;
- `telegram_sources.source_subtype` equals `sources.source_subtype`;
- `telegram_sources.peer_id` equals the canonical integer value of
  `sources.external_id`;
- `telegram_sources.peer_kind` is `channel` for `channel` and `supergroup`
  sources, and `chat` for `group` sources;
- `telegram_sources.resolution_strategy` is one of `username`, `dialog`,
  `legacy_metadata`, or `unknown`;
- nullable `username`, `access_hash`, and `avatar_cache_key` are treated as
  optional resolution/display hints, not as required identity fields.

Required typed identity completeness is limited to the cross-table identity
criteria above. Optional enrichment gaps are valid runtime states and must not
trigger legacy metadata decode by themselves:

- missing `avatar_cache_key` is valid and only means no cached avatar is
  available;
- missing `username` is valid for dialog-backed, private, renamed, or unknown
  username sources;
- missing `access_hash` is valid for small groups and for sources that can be
  resolved by username or dialog scan;
- `resolution_strategy = 'unknown'` is a valid minimal identity state.

### Repair Invariant

Startup repair should follow this rule:

`valid typed row wins; legacy blob is fallback input only when required typed
identity is missing or invalid`.

If a Telegram source already has a `telegram_sources` row that satisfies the
well-formed criteria above, repair should not decode `sources.metadata_zstd` for
that source. This protects new rows with `metadata_zstd = NULL` and also
protects typed rows whose old blob is malformed.

If the `telegram_sources` row is absent or violates required identity criteria,
repair may decode legacy `SourceMetadata` from `sources.metadata_zstd` to
recover optional `resolution_strategy`, `username`, `access_hash`, and
`avatar_cache_key`. Missing or malformed legacy metadata should remain
non-fatal when canonical source fields are enough to construct a minimal typed
identity. A typed row with optional enrichment gaps must not be treated as an
invalid required identity. A partially broken typed row must not silently block
legacy repair; it should either be repaired from canonical/legacy inputs or
reported as projection drift when it conflicts with canonical source identity.

Existing duplicate and projection-drift checks still apply.

Repair compatibility outcomes are explicit:

| State | Required canonical source fields | Legacy blob | Outcome |
| --- | --- | --- | --- |
| Valid typed row exists | Any | Not decoded | No repair diagnostic and no legacy fallback. |
| Typed row missing or invalid | Enough to derive required identity | Valid | Repair creates or fixes typed identity and enriches optional hints from the blob. |
| Typed row missing or invalid | Enough to derive required identity | Missing or malformed | Repair creates a minimal typed identity with `resolution_strategy = 'unknown'` and nullable optional hints; it may record a non-fatal diagnostic if the current diagnostics model supports one. |
| Typed row missing or invalid | Not enough to derive required identity | Any | Repair records a fatal diagnostic and apply mode must not create a typed row for that source. |

Enough canonical source fields means the row is a Telegram source,
`sources.account_id` is present, `sources.source_subtype` is a supported
Telegram subtype, and `sources.external_id` is a canonical Telegram peer id.
The legacy blob cannot repair missing or invalid required canonical source
fields on its own. A non-canonical `sources.external_id` is insufficient
canonical identity even if the legacy blob contains a recoverable peer id.

### Transactionality

The add/upsert flow must keep the existing transaction boundary around both
tables. If the typed identity insert/update fails after the source row write,
the transaction must roll back the source write too.

This is more important after this cleanup because a new Telegram source without
`telegram_sources` no longer has a legacy metadata fallback.

### Documentation

Documentation should say:

- new Telegram rows keep `sources.metadata_zstd` `NULL`;
- old Telegram blobs may remain as legacy repair input;
- normal runtime updates preserve old Telegram blobs rather than clearing them
  opportunistically;
- normal Telegram sync, Takeout, forum topic refresh, source list display, and
  source resolution use typed identity in `telegram_sources`;
- YouTube rows still use `sources.metadata_zstd` for video/playlist metadata.

The backlog item should move from "move remaining Telegram display/avatar
metadata out of `sources.metadata_zstd`" to a more precise follow-up such as
"optionally clear old Telegram metadata blobs after successful typed repair".

## Testing Strategy

The test set should prove that the invariant holds beyond the first add path.

- `add_telegram_source_by_username_metadata_null`: adding by public username
  writes no Telegram source blob.
- `add_telegram_source_from_dialog_metadata_null`: adding a private/dialog
  source writes no Telegram source blob.
- `add_telegram_source_writes_required_identity_and_available_optional_fields`:
  add/upsert always records required typed identity in `telegram_sources`,
  writes `resolution_strategy` as a non-null value, and records optional
  username/access-hash/avatar hints only when available.
- `add_telegram_source_rolls_back_source_when_typed_identity_fails`: a forced
  typed identity failure leaves no orphan `sources` row.
- `readd_existing_telegram_source_keeps_metadata_null`: repeated add/upsert of
  a source whose blob is already `NULL` does not recreate the blob.
- `readd_existing_legacy_source_preserves_old_metadata_blob`: repeated
  add/upsert does not opportunistically clear an older blob in this slice.
- `list_sources_uses_telegram_sources_avatar_cache_key`: source listing reads
  avatar display data from typed identity, not decoded metadata.
- `sync_refresh_does_not_recreate_metadata_blob`: sync/avatar refresh updates
  `telegram_sources.avatar_cache_key` and leaves `sources.metadata_zstd` `NULL`.
- `sync_refresh_preserves_existing_legacy_metadata_blob`: sync/avatar refresh
  does not opportunistically clear an older blob in this slice.
- `takeout_source_import_does_not_create_or_require_metadata_blob`: Takeout
  source loading works from typed identity with no source metadata blob and does
  not write one during import finalization.
- `legacy_repair_decodes_old_metadata_when_typed_missing`: old rows without
  typed identity still repair from legacy blobs.
- `legacy_repair_skips_metadata_when_typed_present_even_if_blob_null`: new rows
  with typed identity and no blob do not produce repair diagnostics.
- `legacy_repair_skips_malformed_metadata_when_typed_present`: typed identity
  protects rows even if an old blob is corrupt.
- `legacy_repair_ignores_optional_enrichment_gaps_when_typed_identity_valid`:
  `NULL` username/access-hash/avatar hints and `unknown` strategy do not cause
  repair to decode legacy metadata.
- `legacy_repair_fails_when_blob_unusable_and_canonical_identity_insufficient`:
  missing required canonical fields plus missing or malformed legacy metadata
  produce a fatal repair diagnostic.
- `youtube_add_source_still_writes_metadata`: this Telegram slice does not
  regress YouTube source metadata.

Containment scans should also check that `encode_source_metadata` is not used by
normal Telegram add/sync/list paths. `decode_source_metadata` should remain
reachable only from explicit repair, migration or diagnostic code, plus tests
that prove legacy behavior.

## Risks And Mitigations

- Hidden write path recreates Telegram blobs: cover sync, add/upsert, Takeout,
  and source listing with tests and targeted scans.
- Repair treats new rows as damaged because the blob is missing: make valid
  typed identity the first-class repair outcome.
- Existing databases still contain old blobs: document that this slice stops
  new writes only; cleanup is a later decision.
- Atomicity regression creates source rows without typed identity: keep the
  transaction boundary and add rollback coverage.
- YouTube behavior regresses because `metadata_zstd` is shared: keep YouTube
  tests and do not generalize Telegram behavior to all source types.

## Acceptance Criteria

- Newly inserted Telegram source rows keep `sources.metadata_zstd IS NULL`.
- Normal runtime updates never create, replace, or decode Telegram
  `sources.metadata_zstd` blobs and do not opportunistically clear existing
  legacy blobs.
- Telegram add/upsert writes required typed identity plus available optional
  hints/display cache into `telegram_sources`.
- Telegram add/upsert is atomic across `sources` and `telegram_sources`; if
  typed identity insert/update fails, no orphan `sources` row remains.
- Required identity, optional resolution hints, and optional display cache are
  treated as separate data classes.
- Valid typed identity, as defined in this spec, prevents repair from decoding
  or depending on legacy metadata blobs.
- Legacy repair still handles old rows missing typed identity.
- Legacy repair emits a fatal diagnostic when required canonical source fields
  are insufficient to construct typed identity; legacy metadata cannot override
  that.
- Source list display, sync refresh, Takeout source loading, and forum/source
  identity paths work without Telegram source metadata blobs.
- YouTube source metadata behavior remains unchanged.
- Current docs distinguish new Telegram runtime behavior from old legacy blobs.
