# Current Schema Baseline Reset

> Scope: Database Schema Simplification design slice.

## Decision

Reset the active migration history around the current schema.

The only supported runtime starting point after this slice is baseline v1.
Fresh installs create the current schema directly from the baseline migration.
The one existing controlled database is converted automatically at startup from
the old migration history to the new baseline history.

Pre-reset migration history is archived for project history, but is not part of
the active build and is not a supported automatic upgrade path.

## Context

The current migration layer carries the whole project history: ordinary SQL
migrations, runner-managed Rust migrations, sentinel SQL files, checksum
repair, line-ending repair, and historical schema cleanup. That work was
valid while old databases needed to upgrade through many shapes.

The project now has one known database and one user. The risk of unsupported
old external databases is accepted. This allows the project to replace the
runtime upgrade contract with a simpler current-schema baseline.

## Goals

- Make the current database schema the new migration baseline.
- Keep existing product data unchanged during cutover.
- Create a mandatory backup before changing migration bookkeeping.
- Remove old migration and repair code from the active runtime path.
- Keep pre-reset SQL and Rust migration history archived in git.
- Start future migrations at version 2.

## Non-Goals

- No product table data migration during cutover.
- No support for arbitrary pre-reset databases.
- No strict `sqlite_master` schema diff in this slice.
- No export/import rewrite of existing data.
- No deletion of project history from git.

## Active Migration Layout

After the reset, active migrations use zero-padded filenames:

```text
src-tauri/migrations/
  0001_current_schema_baseline.sql
  0002_next_real_change.sql
  0003_...
```

`0001_current_schema_baseline.sql` creates the current schema directly. It does
not patch older schema shapes, run repair logic, backfill product data, or
decode legacy blobs. It is the current schema from scratch.

`build_migrations()` returns baseline v1 and future v2+ migrations only. Old
versions 1 through 26 no longer appear in the active migration list.

## Archived Migration History

Pre-reset history is kept in git outside the active build, for example:

```text
docs/archive/migrations-pre-baseline-reset/
  sql/
    1.sql
    ...
    26.sql
  rust/
    source_identity_cleanup.rs
    youtube_typed_source_metadata.rs
    telegram_item_native_identity.rs
    topic_membership_materialization.rs
    analysis_documents.rs
```

These files are historical reference material only. They are not compiled, not
included by `build_migrations()`, and not a supported upgrade path.

## Startup Flow

Startup classifies the database into one of four states:

1. No database file exists.
   Create the database through baseline v1.

2. Database already has baseline v1 with the expected checksum.
   Start normally. Cutover does not run again.

3. Database has old successful migration history through version 26.
   Run the automatic baseline cutover.

4. Any other state.
   Fail startup with a clear unsupported-database error.

A database that contains baseline v1 with the wrong checksum is unsupported.
The application should fail before the migration runner reaches its own
checksum mismatch error.

## Weak Validation

The old-history cutover uses weak validation because there is one controlled
database and external schema mutation is not an expected case.

Cutover is allowed only when:

- `_sqlx_migrations` exists;
- no migration is marked failed;
- the old history contains a successful migration 26 record;
- old versions 1 through 26 are all present and successful.

This slice does not maintain a separate expected `sqlite_master` table/index
list. The baseline SQL is the source of truth for fresh databases.

## Cutover Mechanics

The automatic cutover is bookkeeping-only:

```text
existing DB with old history
  -> validate old history through migration 26
  -> create mandatory backup beside the database
  -> replace _sqlx_migrations contents with baseline v1 record
  -> continue startup as a baseline database
```

The backup must be created before `_sqlx_migrations` is changed. If backup
creation fails, startup fails and the database remains untouched.

Backup naming:

```text
extractum.db.pre-baseline-reset-YYYYMMDD-HHMMSS.bak
```

Backup files remain beside the database after a successful cutover. The
application does not delete or rotate them automatically; retention is a user
responsibility.

The cutover does not insert, update, delete, alter, or drop product tables. It
only reads migration state and rewrites `_sqlx_migrations`.

The `_sqlx_migrations` rewrite is one transaction. Clearing the old migration
rows and inserting the baseline v1 row must commit or roll back together so an
interrupted rewrite cannot leave the database without migration history.

## Sqlx Bookkeeping

The synthetic baseline row must match what the migration runner expects for
the baseline migration file.

Required fields:

- `version = 1`, matching `0001_current_schema_baseline.sql`;
- `description = "current schema baseline"`, matching `build_migrations()`;
- `success = 1`;
- `checksum = SHA-384` over the exact baseline SQL file bytes;
- `installed_on` populated with the cutover timestamp;
- `execution_time = 0` is acceptable for the synthetic bookkeeping write.

The checksum must not be arbitrary. It must use the same algorithm the active
migration runner uses for SQL file checksum validation.

## Structural Safety

The backup operation must be isolated behind a small explicit interface so
tests can inject a backup failure without relying on readonly filesystems or
platform-specific permission behavior.

The cutover code must make the "no product table mutation" invariant
structural. The preferred shape is a small cutover module whose write API is
limited to `_sqlx_migrations`. It may read migration state and write migration
bookkeeping, but it must not expose generic product-table mutation helpers.

## Testing

Minimum tests:

- fresh baseline creates the expected core tables, indexes, and constraints;
- `build_migrations()` contains baseline version 1 and reserves future
  migrations for version 2 and above;
- old successful history through version 26 triggers backup-first cutover;
- injected backup failure prevents any `_sqlx_migrations` rewrite;
- representative product data counts/hashes are unchanged before and after
  cutover;
- already reset baseline history is idempotent;
- baseline v1 with wrong checksum is rejected as unsupported;
- failed migration history is rejected;
- partial old history without migration 26 is rejected.

The product-data preservation test should seed representative rows and compare
counts or hashes before and after cutover. It does not need to diff every table
row-by-row because the cutover module is structurally limited to migration
bookkeeping writes.

## Documentation

Update schema and architecture docs to say:

- baseline v1 is the only supported starting point after the reset;
- old migrations 1 through 26 and associated Rust modules are archived, not
  active runtime code;
- future migrations start at `0002`;
- the automatic startup cutover is for the one controlled pre-reset database;
- unsupported pre-reset databases must be restored from backup or handled
  manually outside the active app migration path.
