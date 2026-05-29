# Legacy Telegram Source Metadata Cleanup Policy Design

> Date: 2026-05-23
> Status: implemented and archived
> Scope: historical policy and implementation shape for guarded legacy Telegram
> source metadata cleanup.
> Current source of truth: `docs/database-schema.md` and
> `src-tauri/src/sources/legacy_metadata_cleanup.rs`.

## Goal

Close the architecture decision for old Telegram `sources.metadata_zstd` blobs:
whether and when they may be cleared now that Telegram runtime identity lives in
typed tables.

This historical slice defined the cleanup policy and the shape of the explicit
maintenance operation. The guarded audit/clear helper has since been
implemented; cleanup remains explicit and is not an automatic migration or
normal runtime side effect.

## Decision

Legacy Telegram `sources.metadata_zstd` is no longer the runtime source of
truth.

Normal Telegram source list, sync, Takeout, forum topic refresh, source
resolution, source browsing, and export paths must use typed identity and
display fields in `telegram_sources`, not Telegram source metadata blobs.

Old Telegram blobs may be cleared only by an explicit guarded
audit/dry-run/clear operation. They must not be cleared by automatic destructive
startup migration, opportunistic sync/update/list/Takeout behavior, or ordinary
schema migration.

## Evidence At The Time

Live DB aggregate evidence for the original policy decision, collected without
decoding blob contents:

| Field | Count |
| --- | ---: |
| Telegram sources | 21 |
| Telegram sources with typed identity | 21 |
| Telegram sources with legacy `metadata_zstd` | 11 |
| Legacy blobs with typed identity | 11 |
| Legacy blobs without typed identity | 0 |
| Legacy blobs on `channel` sources | 5 |
| Legacy blobs on `supergroup` sources | 6 |
| Legacy blobs on `group` sources | 0 |

Recent real-data validation has exercised representative typed identity paths,
including dialog-backed/no-username supergroup Takeout and small-group Takeout.
At the time, `CHANNEL_PRIVATE` fallback was not yet run. Later validation and
current cleanup behavior are tracked in current-state docs and reusable
verification notes; this archived spec remains the policy rationale.

These aggregate counts are decision input only. They are not an eligibility
result for the implemented cleanup helper; the helper recomputes guards at run
time.

## Cleanup Preconditions

The implemented cleanup operation may clear a Telegram `sources.metadata_zstd`
blob only when all guards pass:

1. `sources.source_type = 'telegram'`.
2. `sources.metadata_zstd IS NOT NULL`.
3. A matching `telegram_sources` row exists.
4. `sources.source_subtype = telegram_sources.source_subtype`.
5. `sources.account_id = telegram_sources.account_id`.
6. `telegram_sources.source_subtype`, `peer_kind`, and `resolution_strategy`
   pass the existing enum and subtype/peer-kind invariants.
7. Peer identity fields are valid for their subtype and peer kind.
8. Startup source identity repair has completed successfully for the database,
   and there is no fatal repair error for the source. Non-fatal
   `source_identity_repair_notes` may be reported but must not automatically
   block cleanup unless the helper explicitly treats a note code as blocking.

The helper may choose stricter product gates, such as requiring a source to
have survived list, sync, or Takeout validation. Those gates should be policy
parameters or documented implementation choices, not implicit side effects of
normal runtime paths.

## Explicit Operation Shape

The shipped implementation is a guarded maintenance helper, not a schema
migration:

- `audit_legacy_telegram_source_metadata()`
- `clear_legacy_telegram_source_metadata(dry_run: bool)`

The audit mode reports eligible and skipped rows without mutation. The clear
mode should share the exact same eligibility logic, then set
`sources.metadata_zstd = NULL` only for eligible rows.

Output boundary:

- aggregate counts;
- source ids;
- subtype counts;
- candidate counts;
- skipped reason codes and counts;
- cleared count;
- dry-run flag.

The operation must never return, log, or decode blob contents.

## Why Not SQL Migration

Do not implement this as:

```sql
UPDATE sources SET metadata_zstd = NULL WHERE source_type = 'telegram';
```

This cleanup is data-retention and repair-input policy, not schema evolution.
The active migration history now starts from the current baseline v1, and future
schema migrations start at `0002`. A blanket SQL migration would be
irreversible, would provide weak auditability, and would mix schema evolution
with product-table mutation.

The project already treats baseline schema shape separately from controlled
data cleanup and migration-history cutover. This decision follows that boundary:
schema migrations may add or reshape schema, but clearing repair input requires
an explicit guarded operation with a dry-run/audit path.

## Historical Non-Goals For This Policy Slice

- It did not add SQL migration `0002` for blob cleanup.
- It did not decode or log legacy blob contents.
- It did not change startup source identity repair semantics.
- It did not change normal sync, list, update, Takeout, or source resolution paths.
- It did not mark `CHANNEL_PRIVATE` fallback validation complete.

## Implementation Status

Implemented as:

- `audit_legacy_telegram_source_metadata`
- `clear_legacy_telegram_source_metadata(dry_run: bool)`

The helper has focused tests for:

- audit output with eligible rows;
- skipped rows for missing typed identity;
- skipped rows for subtype/account mismatch;
- skipped rows for invalid peer-kind/subtype combinations;
- dry-run no-op behavior;
- clear mode mutating only eligible rows;
- sanitized output that never includes blob contents.
