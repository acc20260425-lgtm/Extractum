# Test Migration Atomicity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the test-only application migration batch atomic on one SQLite connection so parallel tests cannot enqueue against a partially published Apalis `Jobs` schema.

**Architecture:** Refactor the existing `#[cfg(test)]` helper into a public-in-crate wrapper over one private batch function. The batch function owns one SQLx transaction for migration-table creation, all 20 migration scripts, all history inserts, and the final commit; in-file tests prove rollback atomicity and concurrent file-backed Apalis enqueue behavior.

**Tech Stack:** Rust 2021, Tokio, SQLx 0.8.6, SQLite WAL, Apalis/apalis-sqlite 1.0.0-rc.8, Cargo.

## Global Constraints

- Modify only `src-tauri/src/migrations.rs` during implementation.
- Keep `build_migrations()` contents and ordering unchanged.
- Keep the signature `pub(crate) async fn apply_all_migrations_for_test_pool(pool: &sqlx::SqlitePool) -> crate::error::AppResult<()>` unchanged.
- Add private test-only `async fn apply_migration_batch_for_test_pool(pool: &sqlx::SqlitePool, migrations: Vec<tauri_plugin_sql::Migration>) -> crate::error::AppResult<()>`.
- Use one transaction-owned SQLite connection for `_sqlx_migrations`, every migration, every history row, and commit.
- Do not add a global mutex, retry, sleep, warning suppression, `--test-threads=1`, dependency change, migration SQL change, or production-path change.
- A forced rollback must remove both the early fixture table and `_sqlx_migrations`; do not query a history row from a table that correctly no longer exists.
- The concurrent regression uses independent file-backed SQLite databases with five-connection pools and real Apalis enqueue operations.
- Any `Jobs` table-existence, column-shape, or schema-visibility error at migration, setup, or enqueue boundaries reopens investigation.
- Do not modify `docs/project.md` or `docs/value-registry.md`; no runtime or persisted value changes.

---

### Task 1: Make the Test Migration Batch Atomic

**Files:**
- Modify: `src-tauri/src/migrations.rs:290-335`
- Test: `src-tauri/src/migrations.rs:337-end` (existing in-file test module)

**Interfaces:**
- Preserves `apply_all_migrations_for_test_pool(&sqlx::SqlitePool) -> AppResult<()>` for all existing test callers.
- Adds private `apply_migration_batch_for_test_pool(&sqlx::SqlitePool, Vec<Migration>) -> AppResult<()>` for wrapper delegation and forced-failure testing.
- Uses `apalis::prelude::TaskSink::push` in the regression test with `SqliteStorage<String>` inferred from a pushed `String`.
- Produces no production interface because both helper functions remain under `#[cfg(test)]`.

- [ ] **Step 1: Verify the clean-tree and approved-spec preconditions**

Run:

```powershell
$status = @(git status --short --untracked-files=all)
git merge-base --is-ancestor 7b761445 HEAD
$specPresent = $LASTEXITCODE -eq 0
"STATUS_COUNT=$($status.Count)"
"APPROVED_SPEC_PRESENT=$specPresent"
if ($status.Count -ne 0 -or -not $specPresent) { exit 1 }
```

Expected: `STATUS_COUNT=0`, `APPROVED_SPEC_PRESENT=True`, exit 0.

- [ ] **Step 2: Record the non-atomic source RED baseline**

Run:

```powershell
$source = Get-Content -Raw src-tauri/src/migrations.rs
$hasBatchFunction = $source.Contains('async fn apply_migration_batch_for_test_pool(')
$hasTransaction = $source -match 'let mut transaction = pool\s*\.begin\(\)\s*\.await'
$transactionExecutors = ([regex]::Matches(
    $source,
    '\.execute\(&mut \*transaction\)'
)).Count
$commitsBatch = $source -match 'transaction\s*\.commit\(\)\s*\.await'
"HAS_BATCH_FUNCTION=$hasBatchFunction"
"HAS_TRANSACTION=$hasTransaction"
"TRANSACTION_EXECUTOR_COUNT=$transactionExecutors"
"COMMITS_BATCH=$commitsBatch"
if (-not $hasBatchFunction -or -not $hasTransaction -or $transactionExecutors -lt 3 -or -not $commitsBatch) { exit 1 }
```

Expected: exit 1 with both booleans `False`, executor count 0, and commit
boolean `False`.

- [ ] **Step 3: Add the failing rollback and concurrent enqueue tests**

Replace the test module's initial imports with:

```rust
use super::{
    apalis_sqlite_migrations, apply_all_migrations_for_test_pool,
    apply_migration_batch_for_test_pool, build_migrations,
    current_schema_baseline_migration, prepare_database_at_path,
};
use apalis::prelude::TaskSink;
use sha2::{Digest, Sha384};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use std::{sync::Arc, time::Duration};
use tauri_plugin_sql::{Migration, MigrationKind};
use tokio::{sync::Barrier, task::JoinSet};
```

Add these tests after `sha384_hex` and before the existing schema tests:

```rust
#[tokio::test]
async fn test_migration_batch_rolls_back_schema_and_history_together() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("connect rollback test database");
    let migrations = vec![
        Migration {
            version: 9_000_000_000_001,
            description: "atomic rollback valid fixture",
            sql: "CREATE TABLE migration_atomicity_probe (id INTEGER NOT NULL);",
            kind: MigrationKind::Up,
        },
        Migration {
            version: 9_000_000_000_002,
            description: "atomic rollback invalid fixture",
            sql: "THIS IS NOT VALID SQLITE;",
            kind: MigrationKind::Up,
        },
    ];

    apply_migration_batch_for_test_pool(&pool, migrations)
        .await
        .expect_err("invalid migration rolls back the complete batch");

    let visible_tables: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM sqlite_master
         WHERE type = 'table'
           AND name IN ('migration_atomicity_probe', '_sqlx_migrations')",
    )
    .fetch_one(&pool)
    .await
    .expect("inspect schema after rollback");
    assert_eq!(visible_tables, 0);
}

#[tokio::test]
async fn concurrent_test_migrations_publish_complete_apalis_schemas() {
    const DATABASE_COUNT: usize = 16;

    let barrier = Arc::new(Barrier::new(DATABASE_COUNT));
    let mut tasks = JoinSet::new();

    for index in 0..DATABASE_COUNT {
        let barrier = barrier.clone();
        tasks.spawn(async move {
            let temp_dir = tempfile::tempdir().expect("create concurrent migration temp dir");
            let db_path = temp_dir.path().join("extractum.db");
            let options = SqliteConnectOptions::new()
                .filename(&db_path)
                .create_if_missing(true)
                .journal_mode(SqliteJournalMode::Wal)
                .busy_timeout(Duration::from_secs(5));
            let pool = SqlitePoolOptions::new()
                .max_connections(5)
                .connect_with(options)
                .await
                .expect("connect concurrent migration database");

            barrier.wait().await;
            apply_all_migrations_for_test_pool(&pool)
                .await
                .expect("apply atomic test migrations");

            let jobs_columns: i64 =
                sqlx::query_scalar("SELECT COUNT(*) FROM pragma_table_info('Jobs')")
                    .fetch_one(&pool)
                    .await
                    .expect("read Jobs columns");
            assert_eq!(jobs_columns, 14, "database {index} has incomplete Jobs schema");

            let idempotency_history: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM _sqlx_migrations WHERE version = ?",
            )
            .bind(20260506101935_i64)
            .fetch_one(&pool)
            .await
            .expect("read idempotency migration history");
            assert_eq!(idempotency_history, 1);

            let mut storage =
                apalis_sqlite::SqliteStorage::new_in_queue(&pool, "migration-atomicity");
            storage
                .push(format!("migration-atomicity-{index}"))
                .await
                .expect("enqueue against complete Jobs schema");

            pool.close().await;
            drop(temp_dir);
        });
    }

    tokio::time::timeout(Duration::from_secs(30), async move {
        while let Some(result) = tasks.join_next().await {
            result.expect("concurrent migration task joins");
        }
    })
    .await
    .expect("concurrent migration stress completes");
}
```

Expected: the rollback test uses a one-connection memory pool so its schema
inspection is deterministic. The stress test holds each temp directory until
its pool is closed and performs a real 14-value Apalis insert.

- [ ] **Step 4: Run the new tests to verify compile RED**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib migrations::tests::test_migration_batch_rolls_back_schema_and_history_together -- --exact
```

Expected: compilation fails because
`apply_migration_batch_for_test_pool` does not exist yet. This is the required
RED; do not weaken the tests.

- [ ] **Step 5: Implement one-transaction migration batching**

Replace the current `apply_all_migrations_for_test_pool` implementation with
exactly:

```rust
#[cfg(test)]
pub(crate) async fn apply_all_migrations_for_test_pool(
    pool: &sqlx::SqlitePool,
) -> crate::error::AppResult<()> {
    apply_migration_batch_for_test_pool(pool, build_migrations()).await
}

#[cfg(test)]
async fn apply_migration_batch_for_test_pool(
    pool: &sqlx::SqlitePool,
    migrations: Vec<Migration>,
) -> crate::error::AppResult<()> {
    use sha2::{Digest, Sha384};

    let mut transaction = pool
        .begin()
        .await
        .map_err(crate::error::AppError::database)?;
    sqlx::raw_sql(
        "CREATE TABLE IF NOT EXISTS _sqlx_migrations (
            version BIGINT PRIMARY KEY,
            description TEXT NOT NULL,
            installed_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            success BOOLEAN NOT NULL,
            checksum BLOB NOT NULL,
            execution_time BIGINT NOT NULL
        )",
    )
    .execute(&mut *transaction)
    .await
    .map_err(crate::error::AppError::database)?;

    for migration in migrations {
        sqlx::raw_sql(migration.sql)
            .execute(&mut *transaction)
            .await
            .map_err(crate::error::AppError::database)?;
        let checksum = Sha384::digest(migration.sql.as_bytes()).to_vec();
        sqlx::query(
            "INSERT INTO _sqlx_migrations (
                version,
                description,
                success,
                checksum,
                execution_time
            ) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(migration.version)
        .bind(migration.description)
        .bind(true)
        .bind(checksum)
        .bind(0_i64)
        .execute(&mut *transaction)
        .await
        .map_err(crate::error::AppError::database)?;
    }

    transaction
        .commit()
        .await
        .map_err(crate::error::AppError::database)?;
    Ok(())
}
```

Expected: the wrapper signature is unchanged. Every database operation in the
batch uses `&mut *transaction`; no operation uses the pool after `begin()` and
before `commit()`.

- [ ] **Step 6: Verify source GREEN and focused atomicity tests**

Run the Step 2 source contract again.

Expected: all three booleans are `True`, transaction executor count is 3, and
the command exits 0.

Then run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib migrations::tests::test_migration_batch_rolls_back_schema_and_history_together -- --exact
cargo test --manifest-path src-tauri/Cargo.toml --lib migrations::tests::concurrent_test_migrations_publish_complete_apalis_schemas -- --exact
```

Expected: each command runs exactly one test and passes. The rollback test sees
zero matching tables after the forced failure; the stress test completes all
16 independent databases within 30 seconds.

- [ ] **Step 7: Repeat the concurrent regression ten times**

Run:

```powershell
$failedRuns = @()
1..10 | ForEach-Object {
    & cargo test --manifest-path src-tauri/Cargo.toml --lib `
        migrations::tests::concurrent_test_migrations_publish_complete_apalis_schemas `
        -- --exact *> $null
    if ($LASTEXITCODE -ne 0) { $failedRuns += $_ }
}
"STRESS_FAILURE_COUNT=$($failedRuns.Count)"
"STRESS_FAILED_RUNS=$($failedRuns -join ',')"
if ($failedRuns.Count -ne 0) { exit 1 }
```

Expected: `STRESS_FAILURE_COUNT=0`.

- [ ] **Step 8: Run all migration and Gemini jobs tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib migrations::tests
cargo test --manifest-path src-tauri/Cargo.toml --lib gemini_browser::jobs::tests
```

Expected: both groups pass with zero failures. Existing migration checksums,
history compatibility tests, and all 36 Gemini jobs tests remain green.

- [ ] **Step 9: Run the full Rust suite three times with broad recurrence detection**

Run:

```powershell
$schemaPatterns = @(
    'table Jobs has \d+ columns but \d+ values were supplied',
    'no such table:\s*Jobs',
    'Jobs.{0,120}(has no column|no column named|schema)',
    '(has no column|no column named).{0,120}Jobs',
    'database schema has changed'
)
1..3 | ForEach-Object {
    $run = $_
    $output = & cargo test --manifest-path src-tauri/Cargo.toml 2>&1
    $cargoExit = $LASTEXITCODE
    $text = $output | Out-String
    $schemaMatches = @($schemaPatterns | Where-Object { $text -match $_ })
    "FULL_RUN_${run}_EXIT=$cargoExit"
    "FULL_RUN_${run}_JOBS_SCHEMA_MATCHES=$($schemaMatches.Count)"
    if ($schemaMatches.Count -ne 0) {
        $output
        Write-Error "Jobs schema visibility failure recurred in full run $run"
        exit 1
    }
    if ($cargoExit -ne 0) {
        $output
        exit $cargoExit
    }
}
```

Expected: all three exit lines are 0 and all three schema-match counts are 0.
Any other test failure also stops the slice; failures are not retried away.

- [ ] **Step 10: Verify formatting and all Rust targets with zero warnings**

Run:

```powershell
npm.cmd run check:rustfmt
$output = & cargo check --manifest-path src-tauri/Cargo.toml --all-targets --message-format=short 2>&1
$cargoExit = $LASTEXITCODE
$text = $output | Out-String
$warnings = ($text -split "`r?`n") | Where-Object {
    $_ -match 'warning:' -and $_ -notmatch '^warning: `extractum`'
}
"CARGO_EXIT=$cargoExit"
"WARNING_COUNT=$($warnings.Count)"
$warnings
if ($warnings.Count -ne 0) { exit 1 }
exit $cargoExit
```

Expected: rustfmt exits 0 with no diff; `CARGO_EXIT=0` and
`WARNING_COUNT=0`.

- [ ] **Step 11: Review exact scope and commit**

Run:

```powershell
git diff --check
$changed = @(git status --porcelain=v1 --untracked-files=all | ForEach-Object {
    $_.Substring(3).Replace('\', '/')
})
$allowed = @('src-tauri/src/migrations.rs')
$unexpected = @($changed | Where-Object { $_ -notin $allowed })
"CHANGED=$($changed -join ',')"
"UNEXPECTED=$($unexpected -join ',')"
if ($changed.Count -ne 1 -or $unexpected.Count -ne 0) { exit 1 }
git diff -- src-tauri/src/migrations.rs
git add -- src-tauri/src/migrations.rs
git commit -m "test: make migration batches atomic"
git status --short --branch
```

Expected: the implementation commit contains only the test-only helper and its
two in-file tests. The worktree is clean after commit.
