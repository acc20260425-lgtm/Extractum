# Test Migration Atomicity Design

## Goal

Eliminate the intermittent Apalis `Jobs` schema race in the parallel Rust test
suite by making the test-only application migration helper atomic on one
SQLite connection, without changing production migrations or serializing
independent tests globally.

## Observed Failure

Two full-suite runs have now failed at an Apalis enqueue boundary with:

```text
table Jobs has 13 columns but 14 values were supplied
```

The latest failure occurred in
`gemini_browser::jobs::tests::worker_timeout_marks_run_failed_and_processes_next_job`.
Afterward, that exact test passed five times in isolation, the complete
`gemini_browser::jobs::tests` module passed ten consecutive times, and a fresh
full Rust suite passed all 1117 tests. The failure is therefore
concurrency-dependent rather than a stable defect in the named behavior test.

## Investigation Evidence

Pinned `apalis-sqlite 1.0.0-rc.8` inserts into `Jobs` without a column list and
supplies 14 values, including `idempotency_key`.

The observed 13-column shape is exact and meaningful:

- `20251018164941_move_to_bytes.sql` recreates `Jobs` with 13 columns;
- `20260506101935_idempotency_key.sql` adds the fourteenth column;
- enqueue therefore observed the schema after the first migration but before
  the second was visible.

The test-only `apply_all_migrations_for_test_pool` helper currently executes
each migration SQL through a five-connection pool, then records its history in
a separate statement. The complete batch is not pinned to one connection and
is not enclosed in one transaction.

SQLx 0.8.6 documents migration locking as enabled by default, but its SQLite
`Migrate::lock()` and `unlock()` implementations are no-ops. Context7's Apalis
documentation describes the SQL backend and priority/idempotency context but
does not document an additional cross-database migration lock. Local source
inspection found no process-global migration state in `apalis-sqlite`'s
`SqliteStorage::migrations()` implementation.

Every failing test creates a distinct `tempfile::tempdir()` database, so path
reuse has not been found. The precise SQLite/SQLx visibility mechanism across
pool connections under full-suite load remains a hypothesis, not a proven
third-party root cause. What is proven is that a consumer reached a partial
test schema and that the helper currently permits non-atomic schema and
bookkeeping publication.

## Selected Design

Change only the `#[cfg(test)]` migration infrastructure in
`src-tauri/src/migrations.rs`.

Introduce a private test-only batch function that accepts the migration list,
begins one SQLx transaction from the supplied pool, creates
`_sqlx_migrations`, applies every migration, writes each corresponding history
row through that same transaction, and commits only after the complete batch
succeeds.

`apply_all_migrations_for_test_pool` remains the public test helper used by the
repository. It delegates to the batch function with `build_migrations()`.
Existing callers and their return type do not change.

The transaction provides one publication boundary:

```text
BEGIN
  create migration history
  apply all 20 migrations
  write all 20 history rows
COMMIT
```

If any migration or history insert fails, dropping or rolling back the
transaction prevents both partial schema and partial history from becoming the
helper's successful output.

This does not introduce a global mutex. Tests using different temporary
database files remain parallel. Within one helper invocation, migrations were
already sequential; pinning them to one connection removes connection
switching and reduces SQLite commit overhead. Expected fresh-database time is
normally tens of milliseconds, with slow Windows storage potentially taking
low hundreds of milliseconds.

## Test Strategy

Add tests in the existing `migrations.rs` test module:

1. **Atomic rollback test.** Invoke the private batch function with a valid
   initial migration followed by a deliberately invalid migration. Require an
   error, then query `sqlite_master` and verify that neither the schema created
   by the valid migration nor the `_sqlx_migrations` table exists afterward.
   Do not query a history row directly: correct rollback removes the history
   table itself, so such a query would correctly fail with `no such table`.
   This deterministically proves the all-or-nothing boundary.
2. **Concurrent independent-database stress test.** Start multiple tasks at a
   barrier. Each task creates its own file-backed SQLite database with a pool
   of five connections, calls `apply_all_migrations_for_test_pool`, verifies
   that `Jobs` has 14 columns and that the idempotency migration is recorded,
   then performs a real `apalis-sqlite` enqueue. This exercises the production
   shape of the previously failing boundary while preserving independent DB
   paths.
3. Keep the existing migration checksum, history compatibility, and schema
   tests unchanged.

The source-level RED contract must first show that the helper does not yet use
`pool.begin()`, does not execute migration SQL through a transaction, and does
not commit a complete batch. The stress test is regression evidence; because
the original race is probabilistic, it is not required to fail on every
pre-fix run.

Final verification runs the focused migration tests, the complete Gemini jobs
tests, and three full Rust-suite runs. Any database/schema-shaped failure at a
test migration, Apalis setup, or enqueue boundary that refers to `Jobs`
reopens investigation rather than being retried away. This includes the known
column/value mismatch and possible post-fix forms such as a missing `Jobs`
table (`no such table: Jobs`), missing-column errors, or
`database schema has changed`.

## Failure Handling

- Transaction acquisition, migration SQL, history insertion, and commit errors
  continue mapping through `AppError::database`.
- The helper returns the first error and publishes no successful partial
  batch.
- Test assertion context identifies the failed stage without changing the
  application's database-error shape.
- No automatic retry, sleep, global lock, warning suppression, or test-thread
  reduction is added.

## Scope

Implementation modifies only `src-tauri/src/migrations.rs` and its in-file
tests. It does not modify:

- production `build_migrations()` contents or ordering;
- Tauri plugin migration registration in `lib.rs`;
- Apalis production setup or enqueue logic;
- vendored migration SQL or Cargo dependencies;
- runtime values, persisted wire values, UI code, `docs/project.md`, or
  `docs/value-registry.md`.

## Rejected Alternatives

- A process-global test mutex would likely reduce the symptom but would hide
  the helper's missing atomic boundary and unnecessarily serialize independent
  database tests.
- `--test-threads=1` would slow all 1117 Rust tests and mask future concurrency
  defects.
- Retrying failed enqueue or migration operations would conceal a partially
  published schema rather than prevent it.
- Upgrading Apalis or SQLx is a larger dependency and compatibility slice, and
  no evidence currently shows that a specific newer version fixes this test
  helper boundary.
- Changing production migrations is unjustified because the failing helper is
  compiled only for tests and production uses the Tauri plugin migration path.

## Acceptance Criteria

- `apply_all_migrations_for_test_pool` applies the entire migration batch and
  history on one transaction-owned SQLite connection.
- A forced mid-batch SQL failure leaves neither earlier schema nor the
  `_sqlx_migrations` table visible.
- Parallel independent file-backed databases finish with 14-column `Jobs`
  tables and accept real Apalis enqueue operations.
- Existing migration and Gemini jobs tests pass.
- Three consecutive full Rust-suite runs pass without any `Jobs` table
  existence, column-shape, or schema-visibility failure at migration, setup, or
  enqueue boundaries.
- `cargo check --all-targets` and `npm.cmd run check:rustfmt` pass with zero
  warnings or formatting differences.
- Production code paths, migration contents, dependencies, and runtime
  behavior remain unchanged.
