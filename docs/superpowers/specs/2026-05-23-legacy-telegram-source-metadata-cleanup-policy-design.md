# Legacy Telegram Source Metadata Cleanup Policy Design

> Date: 2026-05-23
> Status: approved policy slice
> Scope: documentation and decision only; no Rust, SQL, or live data mutation.

## Goal

Close the architecture decision for old Telegram `sources.metadata_zstd` blobs:
whether and when they may be cleared now that Telegram runtime identity lives in
typed tables.

This slice does not clear any blob. It defines the cleanup policy and the shape
of a future explicit maintenance operation.

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

## Current Evidence

Current live DB aggregate evidence, collected without decoding blob contents:

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
`CHANNEL_PRIVATE` fallback is still not run. Therefore this policy can permit
cleanup after typed identity audit, but it must not claim that every possible
private or lost-access Telegram scenario has been exhausted.

These aggregate counts are decision input only. They are not an eligibility
result for the future cleanup helper; the helper must recompute guards at run
time.

## Cleanup Preconditions

A future cleanup operation may clear a Telegram `sources.metadata_zstd` blob
only when all guards pass:

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

The future helper may choose stricter product gates, such as requiring a source
to have survived list, sync, or Takeout validation. Those gates should be policy
parameters or documented implementation choices, not implicit side effects of
normal runtime paths.

## Explicit Operation Shape

The preferred future implementation is a guarded maintenance helper, not a
schema migration:

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

## Non-Goals

- Do not clear blobs in this slice.
- Do not add a Rust helper in this slice.
- Do not add SQL migration `0002` for blob cleanup.
- Do not decode or log legacy blob contents.
- Do not change startup source identity repair semantics.
- Do not change normal sync, list, update, Takeout, or source resolution paths.
- Do not mark `CHANNEL_PRIVATE` fallback validation complete.

## Follow-Up

Implement the explicit guarded helper in a separate slice if and when the
project is ready to mutate existing local data.

That slice should include focused tests for:

- audit output with eligible rows;
- skipped rows for missing typed identity;
- skipped rows for subtype/account mismatch;
- skipped rows for invalid peer-kind/subtype combinations;
- dry-run no-op behavior;
- clear mode mutating only eligible rows;
- sanitized output that never includes blob contents.
