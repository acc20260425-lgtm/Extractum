# Current Schema Baseline Reset Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the active migration history with a current-schema baseline v1 and automatically cut over the one controlled pre-reset database without touching product data.

**Architecture:** Build the new baseline SQL while the legacy migration path still compiles, prove baseline parity against the legacy-created schema, then switch the active migration list to baseline v1. A focused `baseline_reset` module classifies existing databases, creates a mandatory backup, and rewrites only `_sqlx_migrations` in one transaction. Pre-reset SQL and Rust migration code are archived outside the active build.

**Tech Stack:** Rust, SQLx SQLite, Tauri SQL plugin migrations, SHA-384 via `sha2`, existing migration tests, `cargo test`.

---

## File Structure

- Create `src-tauri/migrations/0001_current_schema_baseline.sql`: active current-schema baseline SQL.
- Create `src-tauri/src/migrations/baseline_reset.rs`: database-history classifier, backup abstraction, filesystem backup implementation, and transaction-only `_sqlx_migrations` rewrite.
- Modify `src-tauri/src/migrations.rs`: expose baseline constants, temporarily keep legacy parity helpers, then switch `build_migrations()` to baseline v1 and call the cutover path from `prepare_database()`.
- Move `src-tauri/migrations/1.sql` through `src-tauri/migrations/26.sql` to `docs/archive/migrations-pre-baseline-reset/sql/`.
- Move legacy runner-managed modules from `src-tauri/src/migrations/` to `docs/archive/migrations-pre-baseline-reset/rust/`.
- Modify `docs/database-schema.md`, `docs/backend-architecture-simplification-analysis.md`, and `docs/backlog.md`: document baseline v1, archived history, automatic one controlled cutover, and future `0002` migration numbering.

---

### Task 1: Baseline SQL Candidate And Parity Harness

**Files:**
- Create: `src-tauri/migrations/0001_current_schema_baseline.sql`
- Modify: `src-tauri/src/migrations.rs`
- Test: `src-tauri/src/migrations.rs`

- [x] **Step 1: Write the failing baseline migration candidate test**

In `src-tauri/src/migrations.rs`, add constants near the top of the file:

```rust
const BASELINE_VERSION: i64 = 1;
const BASELINE_DESCRIPTION: &str = "current schema baseline";
const BASELINE_SQL: &str = include_str!("../migrations/0001_current_schema_baseline.sql");
```

Add this helper near `build_migrations()` but do not switch `build_migrations()` yet:

```rust
fn current_schema_baseline_migration() -> Migration {
    Migration {
        version: BASELINE_VERSION,
        description: BASELINE_DESCRIPTION,
        sql: BASELINE_SQL,
        kind: MigrationKind::Up,
    }
}
```

Add this test in the migration test module:

```rust
#[test]
fn current_schema_baseline_migration_is_version_one() {
    let migration = current_schema_baseline_migration();

    assert_eq!(migration.version, 1);
    assert_eq!(migration.description, "current schema baseline");
    assert!(migration.sql.contains("CREATE TABLE accounts"));
    assert!(migration.sql.contains("CREATE TABLE archive_read_items"));
}
```

- [x] **Step 2: Run the test to verify it fails**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::tests::current_schema_baseline_migration_is_version_one
```

Expected: compile failure because `src-tauri/migrations/0001_current_schema_baseline.sql` does not exist.

- [x] **Step 3: Add a temporary minimal baseline file**

Create `src-tauri/migrations/0001_current_schema_baseline.sql` with:

```sql
-- Current schema baseline. This file is populated in Task 1 after the
-- legacy-vs-baseline parity harness is in place.
CREATE TABLE accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT
);
```

- [x] **Step 4: Run the baseline candidate test again**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::tests::current_schema_baseline_migration_is_version_one
```

Expected: FAIL because the baseline SQL does not contain `CREATE TABLE archive_read_items`.

- [x] **Step 5: Add the schema parity test harness**

Add these helpers inside the migration test module:

```rust
async fn apply_baseline_for_test_pool(pool: &sqlx::SqlitePool) {
    sqlx::raw_sql(current_schema_baseline_migration().sql)
        .execute(pool)
        .await
        .expect("apply current schema baseline");
}

async fn schema_signature(pool: &sqlx::SqlitePool) -> Vec<(String, String, String)> {
    let mut rows: Vec<(String, String, String)> = sqlx::query_as(
        r#"
        SELECT type, name, COALESCE(sql, '')
        FROM sqlite_master
        WHERE name NOT LIKE 'sqlite_%'
          AND name != '_sqlx_migrations'
        ORDER BY type ASC, name ASC
        "#,
    )
    .fetch_all(pool)
    .await
    .expect("read schema signature");

    for row in &mut rows {
        row.2 = normalize_schema_sql(&row.2);
    }

    rows
}

fn normalize_schema_sql(sql: &str) -> String {
    sql.split_whitespace().collect::<Vec<_>>().join(" ")
}
```

Add the parity test:

```rust
#[tokio::test]
async fn current_schema_baseline_matches_legacy_migrated_schema() {
    let legacy_pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect legacy memory sqlite");
    apply_all_migrations_for_test_pool(&legacy_pool)
        .await
        .expect("apply legacy migrations");

    let baseline_pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect baseline memory sqlite");
    apply_baseline_for_test_pool(&baseline_pool).await;

    assert_eq!(
        schema_signature(&baseline_pool).await,
        schema_signature(&legacy_pool).await
    );
}
```

- [x] **Step 6: Run the parity test to verify it fails**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::tests::current_schema_baseline_matches_legacy_migrated_schema
```

Expected: FAIL because the temporary baseline only creates `accounts`.

- [x] **Step 7: Populate the baseline SQL**

Create the baseline SQL from the current legacy-created schema. Use the parity
test as the source of truth: the baseline is complete only when the schema
signature of `0001_current_schema_baseline.sql` matches the schema signature
created by `apply_all_migrations_for_test_pool()`.

The baseline file must include the current definitions for:

```text
accounts
sources
telegram_sources
source_identity_repair_notes
items
telegram_messages
telegram_forum_topics
item_topic_memberships
telegram_topic_resolution_state
youtube_video_sources
youtube_playlist_sources
youtube_playlist_items
youtube_transcript_segments
analysis_source_groups
analysis_source_group_members
analysis_runs
analysis_run_messages
analysis_chat_messages
analysis_trace_refs
analysis_prompt_templates
analysis_documents
archive_read_model_state
archive_read_items
ingest_batches
telegram_takeout_batches
ingest_item_observations
ingest_batch_warnings
app_settings
```

The baseline file must also include all current non-SQLite indexes and triggers
that appear in the legacy schema signature.

- [x] **Step 8: Run the Task 1 tests to verify green**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::tests::current_schema_baseline_migration_is_version_one
cargo test --manifest-path src-tauri/Cargo.toml migrations::tests::current_schema_baseline_matches_legacy_migrated_schema
```

Expected: both tests pass.

- [x] **Step 9: Commit Task 1**

Run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml --check
git diff --check
git add src-tauri/src/migrations.rs src-tauri/migrations/0001_current_schema_baseline.sql
git commit -m "feat: add current schema baseline"
```

---

### Task 2: Baseline Cutover Module

**Files:**
- Create: `src-tauri/src/migrations/baseline_reset.rs`
- Modify: `src-tauri/src/migrations.rs`
- Test: `src-tauri/src/migrations/baseline_reset.rs`

- [x] **Step 1: Write failing classification and checksum tests**

Create `src-tauri/src/migrations/baseline_reset.rs` with the test module first:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha384};

    const BASELINE_SQL_FOR_TEST: &str = "CREATE TABLE baseline_probe (id INTEGER PRIMARY KEY);";

    fn baseline_checksum() -> Vec<u8> {
        Sha384::digest(BASELINE_SQL_FOR_TEST.as_bytes()).to_vec()
    }

    async fn pool_with_migrations_table() -> sqlx::SqlitePool {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");
        create_sqlx_migrations_table_for_test(&pool).await;
        pool
    }

    async fn create_sqlx_migrations_table_for_test(pool: &sqlx::SqlitePool) {
        sqlx::query(
            r#"
            CREATE TABLE _sqlx_migrations (
                version BIGINT PRIMARY KEY,
                description TEXT NOT NULL,
                installed_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                success BOOLEAN NOT NULL,
                checksum BLOB NOT NULL,
                execution_time BIGINT NOT NULL
            )
            "#,
        )
        .execute(pool)
        .await
        .expect("create migrations table");
    }

    async fn insert_migration(pool: &sqlx::SqlitePool, version: i64, success: bool, checksum: Vec<u8>) {
        sqlx::query(
            "INSERT INTO _sqlx_migrations (
                version, description, installed_on, success, checksum, execution_time
             ) VALUES (?, ?, CURRENT_TIMESTAMP, ?, ?, 0)",
        )
        .bind(version)
        .bind(format!("migration {version}"))
        .bind(success)
        .bind(checksum)
        .execute(pool)
        .await
        .expect("insert migration row");
    }

    async fn seed_old_history(pool: &sqlx::SqlitePool) {
        for version in 1_i64..=26_i64 {
            insert_migration(pool, version, true, vec![version as u8]).await;
        }
    }

    #[tokio::test]
    async fn classifies_baseline_history_only_when_checksum_matches() {
        let pool = pool_with_migrations_table().await;
        insert_migration(&pool, 1, true, baseline_checksum()).await;

        let state = classify_migration_history(&pool, BASELINE_SQL_FOR_TEST)
            .await
            .expect("classify history");

        assert_eq!(state, MigrationHistoryState::BaselineReady);
    }

    #[tokio::test]
    async fn rejects_baseline_history_with_wrong_checksum() {
        let pool = pool_with_migrations_table().await;
        insert_migration(&pool, 1, true, vec![1, 2, 3]).await;

        let error = classify_migration_history(&pool, BASELINE_SQL_FOR_TEST)
            .await
            .expect_err("reject checksum mismatch");

        assert!(error.message.contains("baseline checksum"));
    }

    #[tokio::test]
    async fn classifies_old_history_only_when_versions_one_through_twenty_six_are_successful() {
        let pool = pool_with_migrations_table().await;
        seed_old_history(&pool).await;

        let state = classify_migration_history(&pool, BASELINE_SQL_FOR_TEST)
            .await
            .expect("classify old history");

        assert_eq!(state, MigrationHistoryState::OldHistoryReadyForCutover);
    }

    #[tokio::test]
    async fn rejects_partial_old_history_without_version_twenty_six() {
        let pool = pool_with_migrations_table().await;
        for version in 1_i64..=25_i64 {
            insert_migration(&pool, version, true, vec![version as u8]).await;
        }

        let error = classify_migration_history(&pool, BASELINE_SQL_FOR_TEST)
            .await
            .expect_err("reject partial old history");

        assert!(error.message.contains("unsupported migration history"));
    }

    #[tokio::test]
    async fn rejects_failed_migration_history() {
        let pool = pool_with_migrations_table().await;
        seed_old_history(&pool).await;
        insert_migration(&pool, 99, false, vec![99]).await;

        let error = classify_migration_history(&pool, BASELINE_SQL_FOR_TEST)
            .await
            .expect_err("reject failed history");

        assert!(error.message.contains("failed migration"));
    }
}
```

Declare the module from `src-tauri/src/migrations.rs`:

```rust
mod baseline_reset;
```

- [x] **Step 2: Run the tests to verify they fail**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::baseline_reset::tests::
```

Expected: compile failure because `MigrationHistoryState` and
`classify_migration_history` do not exist.

- [x] **Step 3: Implement minimal classification**

Add this production code above the test module in `baseline_reset.rs`:

```rust
use crate::error::{AppError, AppResult};
use sha2::{Digest, Sha384};

const OLD_FIRST_VERSION: i64 = 1;
const OLD_LAST_VERSION: i64 = 26;
const BASELINE_VERSION: i64 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MigrationHistoryState {
    BaselineReady,
    OldHistoryReadyForCutover,
}

pub(super) async fn classify_migration_history(
    pool: &sqlx::SqlitePool,
    baseline_sql: &str,
) -> AppResult<MigrationHistoryState> {
    let table_exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = '_sqlx_migrations'",
    )
    .fetch_one(pool)
    .await
    .map_err(AppError::database)?;

    if table_exists == 0 {
        return Err(AppError::internal(
            "Unsupported migration history: _sqlx_migrations is missing",
        ));
    }

    let failed_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations WHERE success = 0")
            .fetch_one(pool)
            .await
            .map_err(AppError::database)?;
    if failed_count != 0 {
        return Err(AppError::internal(
            "Unsupported migration history: failed migration rows are present",
        ));
    }

    let expected_baseline_checksum = Sha384::digest(baseline_sql.as_bytes()).to_vec();
    let baseline_checksum: Option<Vec<u8>> =
        sqlx::query_scalar("SELECT checksum FROM _sqlx_migrations WHERE version = ?")
            .bind(BASELINE_VERSION)
            .fetch_optional(pool)
            .await
            .map_err(AppError::database)?;

    let total_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations")
        .fetch_one(pool)
        .await
        .map_err(AppError::database)?;

    if let Some(checksum) = baseline_checksum {
        if total_count == 1 && checksum == expected_baseline_checksum {
            return Ok(MigrationHistoryState::BaselineReady);
        }
        if total_count == 1 {
            return Err(AppError::internal(
                "Unsupported migration history: baseline checksum mismatch",
            ));
        }
    }

    let old_success_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM _sqlx_migrations
         WHERE version BETWEEN ? AND ? AND success = 1",
    )
    .bind(OLD_FIRST_VERSION)
    .bind(OLD_LAST_VERSION)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)?;

    let old_last_success: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM _sqlx_migrations WHERE version = ? AND success = 1",
    )
    .bind(OLD_LAST_VERSION)
    .fetch_one(pool)
    .await
    .map_err(AppError::database)?;

    if old_success_count == OLD_LAST_VERSION && old_last_success == 1 {
        return Ok(MigrationHistoryState::OldHistoryReadyForCutover);
    }

    Err(AppError::internal(
        "Unsupported migration history: expected baseline v1 or old successful versions 1 through 26",
    ))
}
```

- [x] **Step 4: Run the classification tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::baseline_reset::tests::classifies_baseline_history_only_when_checksum_matches
cargo test --manifest-path src-tauri/Cargo.toml migrations::baseline_reset::tests::rejects_baseline_history_with_wrong_checksum
cargo test --manifest-path src-tauri/Cargo.toml migrations::baseline_reset::tests::classifies_old_history_only_when_versions_one_through_twenty_six_are_successful
cargo test --manifest-path src-tauri/Cargo.toml migrations::baseline_reset::tests::rejects_partial_old_history_without_version_twenty_six
cargo test --manifest-path src-tauri/Cargo.toml migrations::baseline_reset::tests::rejects_failed_migration_history
```

Expected: all pass.

- [x] **Step 5: Add failing backup-first and transaction tests**

Extend `baseline_reset.rs` tests with:

```rust
#[derive(Default)]
struct RecordingBackup {
    calls: std::sync::Mutex<Vec<std::path::PathBuf>>,
}

impl BaselineResetBackup for RecordingBackup {
    fn create_backup(&self, db_path: &std::path::Path) -> AppResult<std::path::PathBuf> {
        self.calls.lock().expect("lock calls").push(db_path.to_path_buf());
        Ok(db_path.with_extension("bak"))
    }
}

struct FailingBackup;

impl BaselineResetBackup for FailingBackup {
    fn create_backup(&self, _db_path: &std::path::Path) -> AppResult<std::path::PathBuf> {
        Err(AppError::internal("backup failed"))
    }
}

#[tokio::test]
async fn backup_failure_prevents_migration_history_rewrite() {
    let temp_dir = tempfile::tempdir().expect("temp dir");
    let db_path = temp_dir.path().join("extractum.db");
    let pool = sqlx::SqlitePool::connect(&format!("sqlite:{}", db_path.display()))
        .await
        .expect("connect sqlite file");
    create_sqlx_migrations_table_for_test(&pool).await;
    seed_old_history(&pool).await;
    pool.close().await;

    let error = apply_baseline_reset_if_needed(&db_path, BASELINE_SQL_FOR_TEST, &FailingBackup)
        .await
        .expect_err("backup failure blocks cutover");

    assert!(error.message.contains("backup failed"));

    let pool = sqlx::SqlitePool::connect(&format!("sqlite:{}", db_path.display()))
        .await
        .expect("reconnect sqlite file");
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations")
        .fetch_one(&pool)
        .await
        .expect("count migrations");
    assert_eq!(count, 26);
}

#[tokio::test]
async fn old_history_cutover_backs_up_then_rewrites_only_migration_history() {
    let temp_dir = tempfile::tempdir().expect("temp dir");
    let db_path = temp_dir.path().join("extractum.db");
    let pool = sqlx::SqlitePool::connect(&format!("sqlite:{}", db_path.display()))
        .await
        .expect("connect sqlite file");
    create_sqlx_migrations_table_for_test(&pool).await;
    seed_old_history(&pool).await;
    sqlx::query("CREATE TABLE product_probe (id INTEGER PRIMARY KEY, value TEXT NOT NULL)")
        .execute(&pool)
        .await
        .expect("create product probe");
    sqlx::query("INSERT INTO product_probe (id, value) VALUES (1, 'unchanged')")
        .execute(&pool)
        .await
        .expect("seed product probe");
    pool.close().await;

    let backup = RecordingBackup::default();
    apply_baseline_reset_if_needed(&db_path, BASELINE_SQL_FOR_TEST, &backup)
        .await
        .expect("apply baseline reset");

    assert_eq!(backup.calls.lock().expect("lock calls").len(), 1);

    let pool = sqlx::SqlitePool::connect(&format!("sqlite:{}", db_path.display()))
        .await
        .expect("reconnect sqlite file");
    let rows: Vec<(i64, String, bool, Vec<u8>, i64)> = sqlx::query_as(
        "SELECT version, description, success, checksum, execution_time FROM _sqlx_migrations",
    )
    .fetch_all(&pool)
    .await
    .expect("read migrations");
    assert_eq!(rows, vec![(1, "current schema baseline".to_string(), true, baseline_checksum(), 0)]);

    let product_value: String =
        sqlx::query_scalar("SELECT value FROM product_probe WHERE id = 1")
            .fetch_one(&pool)
            .await
            .expect("read product probe");
    assert_eq!(product_value, "unchanged");
}
```

- [x] **Step 6: Run the backup-first tests to verify they fail**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::baseline_reset::tests::backup_failure_prevents_migration_history_rewrite
cargo test --manifest-path src-tauri/Cargo.toml migrations::baseline_reset::tests::old_history_cutover_backs_up_then_rewrites_only_migration_history
```

Expected: compile failure because `BaselineResetBackup` and
`apply_baseline_reset_if_needed` do not exist.

- [x] **Step 7: Implement backup abstraction and transaction rewrite**

Add this production code to `baseline_reset.rs`:

```rust
pub(super) trait BaselineResetBackup {
    fn create_backup(&self, db_path: &std::path::Path) -> AppResult<std::path::PathBuf>;
}

pub(super) async fn apply_baseline_reset_if_needed<B: BaselineResetBackup>(
    db_path: &std::path::Path,
    baseline_sql: &str,
    backup: &B,
) -> AppResult<()> {
    let url = format!("sqlite:{}", db_path.to_string_lossy());
    let pool = sqlx::SqlitePool::connect(&url)
        .await
        .map_err(AppError::database)?;

    let state = classify_migration_history(&pool, baseline_sql).await?;
    pool.close().await;

    if state == MigrationHistoryState::BaselineReady {
        return Ok(());
    }

    backup.create_backup(db_path)?;

    let pool = sqlx::SqlitePool::connect(&url)
        .await
        .map_err(AppError::database)?;
    let state = classify_migration_history(&pool, baseline_sql).await?;
    if state != MigrationHistoryState::OldHistoryReadyForCutover {
        pool.close().await;
        return Err(AppError::internal(
            "Unsupported migration history changed before baseline reset could be applied",
        ));
    }

    rewrite_migration_history_to_baseline(&pool, baseline_sql).await?;
    pool.close().await;
    Ok(())
}

async fn rewrite_migration_history_to_baseline(
    pool: &sqlx::SqlitePool,
    baseline_sql: &str,
) -> AppResult<()> {
    let checksum = Sha384::digest(baseline_sql.as_bytes()).to_vec();
    let mut tx = pool.begin().await.map_err(AppError::database)?;

    sqlx::query("DELETE FROM _sqlx_migrations")
        .execute(&mut *tx)
        .await
        .map_err(AppError::database)?;

    sqlx::query(
        "INSERT INTO _sqlx_migrations (
            version, description, installed_on, success, checksum, execution_time
         ) VALUES (?, ?, CURRENT_TIMESTAMP, 1, ?, 0)",
    )
    .bind(BASELINE_VERSION)
    .bind("current schema baseline")
    .bind(checksum)
    .execute(&mut *tx)
    .await
    .map_err(AppError::database)?;

    tx.commit().await.map_err(AppError::database)
}
```

- [x] **Step 8: Run all cutover module tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::baseline_reset::tests::
```

Expected: all cutover module tests pass.

- [x] **Step 9: Commit Task 2**

Run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml --check
git diff --check
git add src-tauri/src/migrations.rs src-tauri/src/migrations/baseline_reset.rs
git commit -m "feat: add baseline migration cutover"
```

---

### Task 3: Wire Baseline As The Active Migration Path

**Files:**
- Modify: `src-tauri/src/migrations.rs`
- Modify: `src-tauri/src/migrations/baseline_reset.rs`
- Test: `src-tauri/src/migrations.rs`

- [x] **Step 1: Write failing active migration list and no-file tests**

Replace `build_migrations_contains_all_versions_for_sqlx_validation` with:

```rust
#[test]
fn build_migrations_starts_at_current_schema_baseline() {
    let migrations = build_migrations();
    let versions = migrations
        .iter()
        .map(|migration| migration.version)
        .collect::<Vec<_>>();

    assert_eq!(versions, vec![1]);
    assert_eq!(migrations[0].description, "current schema baseline");
    assert!(migrations[0].sql.contains("CREATE TABLE archive_read_items"));
}
```

Add a test for the no-existing-database branch:

```rust
#[test]
fn prepare_database_skips_cutover_when_database_file_is_missing() {
    let temp_dir = tempfile::tempdir().expect("temp dir");
    let db_path = temp_dir.path().join("extractum.db");

    prepare_database_at_path(&db_path).expect("prepare missing database path");

    assert!(!db_path.exists(), "prepare_database must not create a DB before the SQL plugin");
}
```

- [x] **Step 2: Run the tests to verify they fail**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::tests::build_migrations_starts_at_current_schema_baseline
cargo test --manifest-path src-tauri/Cargo.toml migrations::tests::prepare_database_skips_cutover_when_database_file_is_missing
```

Expected: first test fails because `build_migrations()` still returns old
versions 1 through 26; second test fails because `prepare_database_at_path`
does not exist.

- [x] **Step 3: Add filesystem backup implementation**

Add this to `baseline_reset.rs`:

```rust
pub(super) struct FileSystemBaselineResetBackup;

impl BaselineResetBackup for FileSystemBaselineResetBackup {
    fn create_backup(&self, db_path: &std::path::Path) -> AppResult<std::path::PathBuf> {
        let timestamp = backup_timestamp();
        let file_name = db_path
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| AppError::internal("Database path has no valid file name"))?;
        let backup_path = db_path.with_file_name(format!(
            "{file_name}.pre-baseline-reset-{timestamp}.bak"
        ));
        std::fs::copy(db_path, &backup_path)
            .map_err(|error| AppError::internal(format!("Could not create baseline reset backup: {error}")))?;
        Ok(backup_path)
    }
}

fn backup_timestamp() -> String {
    let now = time::OffsetDateTime::now_utc();
    format!(
        "{:04}{:02}{:02}-{:02}{:02}{:02}",
        now.year(),
        u8::from(now.month()),
        now.day(),
        now.hour(),
        now.minute(),
        now.second()
    )
}
```

- [x] **Step 4: Replace active migration list and prepare path**

In `migrations.rs`, change `build_migrations()` to:

```rust
pub fn build_migrations() -> Vec<Migration> {
    vec![current_schema_baseline_migration()]
}
```

Replace `prepare_database()` with path-injectable helpers:

```rust
pub fn prepare_database() -> crate::error::AppResult<()> {
    let Some(db_path) = app_config_db_path() else {
        return Ok(());
    };
    prepare_database_at_path(&db_path)
}

fn prepare_database_at_path(db_path: &Path) -> crate::error::AppResult<()> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| crate::error::AppError::internal(error.to_string()))?;
    }

    if !db_path.exists() {
        return Ok(());
    }

    tauri::async_runtime::block_on(baseline_reset::apply_baseline_reset_if_needed(
        db_path,
        BASELINE_SQL,
        &baseline_reset::FileSystemBaselineResetBackup,
    ))
}
```

Keep the old legacy migration helper functions in this task so the Task 1
parity commit remains reviewable. They are removed after the archive move in
Task 4.

- [x] **Step 5: Update test-only migration helper and remove parity-only test**

Delete the Task 1-only test `current_schema_baseline_matches_legacy_migrated_schema`.
The baseline parity was proven before the active path switched, and after this
step `apply_all_migrations_for_test_pool()` becomes the baseline-backed helper.

Replace `apply_all_migrations_for_test_pool()` with:

```rust
#[cfg(test)]
pub(crate) async fn apply_all_migrations_for_test_pool(
    pool: &sqlx::SqlitePool,
) -> crate::error::AppResult<()> {
    sqlx::raw_sql(BASELINE_SQL)
        .execute(pool)
        .await
        .map_err(crate::error::AppError::database)?;
    Ok(())
}
```

- [x] **Step 6: Run targeted active-path tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::tests::build_migrations_starts_at_current_schema_baseline
cargo test --manifest-path src-tauri/Cargo.toml migrations::tests::prepare_database_skips_cutover_when_database_file_is_missing
cargo test --manifest-path src-tauri/Cargo.toml migrations::tests::current_schema_baseline_migration_is_version_one
```

Expected: all pass.

- [x] **Step 7: Run source fixture smoke tests that rely on migrated schema**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::test_support::tests::source_fixture_creates_expected_tables
cargo test --manifest-path src-tauri/Cargo.toml migrations::tests::fresh_schema_includes_archive_read_model_tables_indexes_and_constraints
cargo test --manifest-path src-tauri/Cargo.toml migrations::tests::fresh_schema_includes_analysis_documents_table_indexes_and_constraints
cargo test --manifest-path src-tauri/Cargo.toml migrations::tests::fresh_schema_includes_ingest_provenance_tables_indexes_and_constraints
```

Expected: all pass with the baseline-backed test helper.

- [x] **Step 8: Commit Task 3**

Run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml --check
git diff --check
git add src-tauri/src/migrations.rs src-tauri/src/migrations/baseline_reset.rs
git commit -m "feat: activate current schema baseline"
```

---

### Task 4: Archive Pre-Reset History And Prune Legacy Tests

**Files:**
- Move: `src-tauri/migrations/1.sql` through `src-tauri/migrations/26.sql`
- Move: `src-tauri/src/migrations/analysis_documents.rs`
- Move: `src-tauri/src/migrations/source_identity_cleanup.rs`
- Move: `src-tauri/src/migrations/telegram_item_native_identity.rs`
- Move: `src-tauri/src/migrations/topic_membership_materialization.rs`
- Move: `src-tauri/src/migrations/youtube_typed_source_metadata.rs`
- Modify: `src-tauri/src/migrations.rs`
- Test: `src-tauri/src/migrations.rs`

- [ ] **Step 1: Move archived SQL files**

Run:

```powershell
New-Item -ItemType Directory -Force docs\archive\migrations-pre-baseline-reset\sql
git mv src-tauri\migrations\1.sql docs\archive\migrations-pre-baseline-reset\sql\1.sql
git mv src-tauri\migrations\2.sql docs\archive\migrations-pre-baseline-reset\sql\2.sql
git mv src-tauri\migrations\3.sql docs\archive\migrations-pre-baseline-reset\sql\3.sql
git mv src-tauri\migrations\4.sql docs\archive\migrations-pre-baseline-reset\sql\4.sql
git mv src-tauri\migrations\5.sql docs\archive\migrations-pre-baseline-reset\sql\5.sql
git mv src-tauri\migrations\6.sql docs\archive\migrations-pre-baseline-reset\sql\6.sql
git mv src-tauri\migrations\7.sql docs\archive\migrations-pre-baseline-reset\sql\7.sql
git mv src-tauri\migrations\8.sql docs\archive\migrations-pre-baseline-reset\sql\8.sql
git mv src-tauri\migrations\9.sql docs\archive\migrations-pre-baseline-reset\sql\9.sql
git mv src-tauri\migrations\10.sql docs\archive\migrations-pre-baseline-reset\sql\10.sql
git mv src-tauri\migrations\11.sql docs\archive\migrations-pre-baseline-reset\sql\11.sql
git mv src-tauri\migrations\12.sql docs\archive\migrations-pre-baseline-reset\sql\12.sql
git mv src-tauri\migrations\13.sql docs\archive\migrations-pre-baseline-reset\sql\13.sql
git mv src-tauri\migrations\14.sql docs\archive\migrations-pre-baseline-reset\sql\14.sql
git mv src-tauri\migrations\15.sql docs\archive\migrations-pre-baseline-reset\sql\15.sql
git mv src-tauri\migrations\16.sql docs\archive\migrations-pre-baseline-reset\sql\16.sql
git mv src-tauri\migrations\17.sql docs\archive\migrations-pre-baseline-reset\sql\17.sql
git mv src-tauri\migrations\18.sql docs\archive\migrations-pre-baseline-reset\sql\18.sql
git mv src-tauri\migrations\19.sql docs\archive\migrations-pre-baseline-reset\sql\19.sql
git mv src-tauri\migrations\20.sql docs\archive\migrations-pre-baseline-reset\sql\20.sql
git mv src-tauri\migrations\21.sql docs\archive\migrations-pre-baseline-reset\sql\21.sql
git mv src-tauri\migrations\22.sql docs\archive\migrations-pre-baseline-reset\sql\22.sql
git mv src-tauri\migrations\23.sql docs\archive\migrations-pre-baseline-reset\sql\23.sql
git mv src-tauri\migrations\24.sql docs\archive\migrations-pre-baseline-reset\sql\24.sql
git mv src-tauri\migrations\25.sql docs\archive\migrations-pre-baseline-reset\sql\25.sql
git mv src-tauri\migrations\26.sql docs\archive\migrations-pre-baseline-reset\sql\26.sql
```

- [ ] **Step 2: Move archived Rust migration modules**

Run:

```powershell
New-Item -ItemType Directory -Force docs\archive\migrations-pre-baseline-reset\rust
git mv src-tauri\src\migrations\analysis_documents.rs docs\archive\migrations-pre-baseline-reset\rust\analysis_documents.rs
git mv src-tauri\src\migrations\source_identity_cleanup.rs docs\archive\migrations-pre-baseline-reset\rust\source_identity_cleanup.rs
git mv src-tauri\src\migrations\telegram_item_native_identity.rs docs\archive\migrations-pre-baseline-reset\rust\telegram_item_native_identity.rs
git mv src-tauri\src\migrations\topic_membership_materialization.rs docs\archive\migrations-pre-baseline-reset\rust\topic_membership_materialization.rs
git mv src-tauri\src\migrations\youtube_typed_source_metadata.rs docs\archive\migrations-pre-baseline-reset\rust\youtube_typed_source_metadata.rs
```

- [ ] **Step 3: Remove legacy module declarations and parity-only tests**

In `src-tauri/src/migrations.rs`, remove these module declarations:

```rust
pub(crate) mod analysis_documents;
pub(crate) mod source_identity_cleanup;
pub(crate) mod telegram_item_native_identity;
pub(crate) mod topic_membership_materialization;
pub(crate) mod youtube_typed_source_metadata;
```

Remove legacy migration tests that inspect old migration versions or runner
sentinels. Delete tests whose names start with these prefixes:

```text
includes_telegram_item_context_migration
includes_telegram_forum_topics_migration
includes_provider_source_subtype_migration
includes_youtube_source_foundation_migration
includes_analysis_run_youtube_corpus_mode_migration
includes_source_identity_schema_bridge_migration
includes_runner_managed_
plugin_migration_list_keeps_
source_identity_schema_bridge_does_not_sql_backfill_typed_identity
checksum_match_accepts_line_ending_only_differences
```

Also remove now-unused helpers:

```text
patch_migrations
apply_regular_sql_migrations_before_runner
apply_regular_sql_migrations_before_runner_on_connection
checksum_matches_line_ending_variant
repair_line_ending_migration_checksums
repair_legacy_v2_migration_checksum
normalize_sql_lf
normalize_sql_crlf
sha384_bytes
```

- [ ] **Step 4: Run tests to find remaining stale references**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::tests::
```

Expected: either pass or fail with concrete unresolved references to deleted
legacy helpers. Remove only stale legacy references; keep baseline and fresh
schema tests.

- [ ] **Step 5: Run broader backend tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: all Rust tests pass.

- [ ] **Step 6: Commit Task 4**

Run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml --check
git diff --check
git add src-tauri/src/migrations.rs src-tauri/src/migrations docs/archive/migrations-pre-baseline-reset src-tauri/migrations
git commit -m "refactor: archive pre-baseline migrations"
```

---

### Task 5: Documentation And Full Verification

**Files:**
- Modify: `docs/database-schema.md`
- Modify: `docs/backend-architecture-simplification-analysis.md`
- Modify: `docs/backlog.md`
- Modify: `docs/superpowers/plans/2026-05-20-current-schema-baseline-reset.md`

- [ ] **Step 1: Update schema documentation**

In `docs/database-schema.md`, update the migration history section to describe:

```text
Baseline v1 (`0001_current_schema_baseline.sql`) is the active starting point
for supported databases. Pre-reset migrations 1 through 26 are archived under
`docs/archive/migrations-pre-baseline-reset/` and are not an automatic upgrade
path. Future migrations start at `0002`.
```

Also document the automatic startup cutover:

```text
The application performs a one-time baseline-history cutover for the one
controlled pre-reset database. The cutover validates old successful migration
history through version 26, creates a mandatory backup beside the database,
then rewrites only `_sqlx_migrations` to baseline v1 in one transaction.
Product tables are not modified.
```

- [ ] **Step 2: Update architecture simplification analysis**

In `docs/backend-architecture-simplification-analysis.md`, mark the current
schema baseline item as implemented after this slice:

```text
Current status:

- the active migration history starts at baseline v1;
- pre-reset SQL and Rust migration history is archived outside the active
  build;
- one controlled pre-reset database is converted through a backup-first
  bookkeeping cutover;
- future migrations start at `0002`.
```

- [ ] **Step 3: Update backlog**

In `docs/backlog.md`, remove or close the open Database Schema Simplification
item for current-schema baseline. Keep unrelated cleanup items that are outside
this reset slice.

- [ ] **Step 4: Run full verification**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
cargo fmt --manifest-path src-tauri/Cargo.toml --check
git diff --check
npm.cmd test
npm.cmd run check
```

Expected:

```text
cargo test: all Rust tests pass
cargo fmt --check: exits 0
git diff --check: exits 0
npm.cmd test: all frontend tests pass
npm.cmd run check: svelte-check reports 0 errors and 0 warnings
```

- [ ] **Step 5: Commit Task 5**

Run:

```powershell
git add docs/database-schema.md docs/backend-architecture-simplification-analysis.md docs/backlog.md docs/superpowers/plans/2026-05-20-current-schema-baseline-reset.md
git commit -m "docs: document current schema baseline reset"
```

---

## Final Verification

After all task commits, run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
cargo fmt --manifest-path src-tauri/Cargo.toml --check
git diff --check
npm.cmd test
npm.cmd run check
git status --short --branch
```

Expected:

```text
cargo test: all Rust tests pass
cargo fmt --check: exits 0
git diff --check: exits 0
npm.cmd test: all frontend tests pass
npm.cmd run check: 0 errors and 0 warnings
git status --short --branch: clean main branch
```
