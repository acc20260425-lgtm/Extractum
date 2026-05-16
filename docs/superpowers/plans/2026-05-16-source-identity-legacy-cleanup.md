# Source Identity Legacy Cleanup Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove the `sources.telegram_source_kind` compatibility mirror from the current database schema, backend API, frontend source types, and normal runtime paths while preserving source ids and canonical source identity.

**Architecture:** Keep migrations 1 through 18 historical, add a runner-managed version 19 cleanup that rebuilds `sources` with foreign keys disabled before the rebuild transaction, and record version 19 in `_sqlx_migrations` so SQLx/Tauri migration history remains coherent. After the schema cleanup, source repair becomes an integrity gate over `sources.source_subtype` plus `telegram_sources`, and runtime/API/frontend code uses `source_subtype`/`sourceSubtype` without old wire aliases.

**Tech Stack:** Rust 2021, Tauri 2, SQLx SQLite, tauri-plugin-sql migrations, grammers Telegram client, Svelte 5, TypeScript, Vitest.

---

## Pre-Implementation Branch Rule

Do not implement this plan directly on `main`. The plan document may live on
`main`; before Task 1 code edits, create an isolated feature branch or
worktree.

Preferred setup:

```powershell
git status --short --branch
git worktree add .worktrees/source-identity-legacy-cleanup -b feature/source-identity-legacy-cleanup
Set-Location .worktrees/source-identity-legacy-cleanup
git status --short --branch
```

Expected:

```text
## main
Preparing worktree (new branch 'feature/source-identity-legacy-cleanup')
HEAD is now at ...
## feature/source-identity-legacy-cleanup
```

If worktree creation is blocked by the environment, create a branch in the
current checkout before code edits:

```powershell
git status --short --branch
git switch -c feature/source-identity-legacy-cleanup
git status --short --branch
```

Expected:

```text
## main
Switched to a new branch 'feature/source-identity-legacy-cleanup'
## feature/source-identity-legacy-cleanup
```

---

## File Structure

Create:

- `src-tauri/migrations/19.sql`: sentinel migration body whose checksum is
  recorded for SQLx history, but whose SQL fails if tauri-plugin-sql ever tries
  to apply v19 directly.
- `src-tauri/src/migrations/source_identity_cleanup.rs`: runner-managed v19
  migration implementation, schema rebuild SQL, FK check helpers, SQLx history
  recording, and migration tests.

Modify:

- `src-tauri/src/migrations.rs`: expose app DB path, run ordinary migrations
  1..18 before plugin startup when needed, run the v19 special migration, keep
  v19 registered for SQLx validation, and keep checksum repair behavior.
- `src-tauri/src/lib.rs`: ensure database preparation failures fail startup
  before the SQL plugin and before source commands.
- `src-tauri/src/sources/identity.rs`: make canonical Telegram external id
  reject `0` and every malformed format required by the spec.
- `src-tauri/src/sources/identity_repair.rs`: remove legacy column reads and
  writes; repair validates canonical `source_subtype`, `account_id`,
  `external_id`, and typed projection drift.
- `src-tauri/src/sources/types.rs`: remove public/persisted
  `telegram_source_kind` fields from `TelegramSourceInfo`, `SourceRecord`,
  `SourceSyncTarget`, and `SourceRecordRow`; keep internal
  `TelegramSourceKind` enum if it still reduces churn.
- `src-tauri/src/sources/store.rs`: rename add-source input to
  `expected_subtype`, stop selecting/writing `telegram_source_kind`, and return
  persisted source DTOs without the mirror.
- `src-tauri/src/sources/peer_resolution.rs`: rename live dialog output to
  `source_subtype` and expected-value helpers to subtype vocabulary; stop using
  `SourceSyncTarget.telegram_source_kind`.
- `src-tauri/src/sources/test_support.rs`: make in-memory source fixtures match
  the post-v19 `sources` schema unless a test explicitly builds an old schema.
- `src-tauri/src/sources/sync.rs`, `src-tauri/src/sources/topics.rs`,
  `src-tauri/src/takeout_import/mod.rs`, `src-tauri/src/youtube/detail.rs`,
  `src-tauri/src/youtube/jobs.rs`, `src-tauri/src/notebooklm_export/model.rs`,
  `src-tauri/src/notebooklm_export/query.rs`,
  `src-tauri/src/notebooklm_export/renderer.rs`,
  `src-tauri/src/notebooklm_export/chunker.rs`: remove normal-code references
  to the legacy column/field and use canonical subtype or typed Telegram
  identity.
- `src-tauri/src/analysis/fixtures.rs`: update fixture inserts to the
  post-v19 schema, except where a test intentionally constructs an old schema.
- `src/lib/types/sources.ts`: remove `telegramSourceKind` from persisted
  `Source`, rename live dialog `telegramSourceKind` to `sourceSubtype`, and
  rename add-source `expectedKind` to `expectedSubtype`.
- `src/lib/api/sources.ts`: map `source_subtype` only, map live dialog
  `source_subtype`, and send `expectedSubtype`.
- `src/lib/source-capabilities.ts`,
  `src/lib/components/analysis/source-management-dialog.svelte`, frontend
  tests under `src/lib/*.test.ts`: use `sourceSubtype` for persisted and live
  source behavior.
- `docs/database-schema.md`, `docs/architecture-deep-dive.md`,
  `docs/backlog.md`: update current-state docs and prune shipped backlog
  cleanup entries.

---

### Task 0: Baseline And Branch Guard

**Files:**
- No source edits.

- [x] **Step 1: Confirm clean main before branch creation**

Run:

```powershell
git status --short --branch
```

Expected:

```text
## main
```

- [x] **Step 2: Create the implementation worktree or branch**

Run the preferred setup from "Pre-Implementation Branch Rule":

```powershell
git worktree add .worktrees/source-identity-legacy-cleanup -b feature/source-identity-legacy-cleanup
Set-Location .worktrees/source-identity-legacy-cleanup
git status --short --branch
```

Expected:

```text
## feature/source-identity-legacy-cleanup
```

- [x] **Step 3: Run baseline Rust tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected:

```text
test result: ok. ... 0 failed
```

- [x] **Step 4: Run baseline frontend tests**

Run:

```powershell
npm.cmd test
```

Expected:

```text
Test Files  ... passed
Tests       ... passed
```

- [x] **Step 5: Run baseline Svelte check**

Run:

```powershell
npm.cmd run check
```

Expected:

```text
svelte-check found 0 errors and 0 warnings
```

- [x] **Step 6: Confirm no baseline changes**

Run:

```powershell
git status --short --branch
```

Expected:

```text
## feature/source-identity-legacy-cleanup
```

No commit is created for this task.

---

### Task 1: Register Runner-Managed Migration 19

**Files:**
- Create: `src-tauri/migrations/19.sql`
- Create: `src-tauri/src/migrations/source_identity_cleanup.rs`
- Modify: `src-tauri/src/migrations.rs`

- [x] **Step 1: Write failing migration registration tests**

Add these tests to `src-tauri/src/migrations.rs` inside the existing
`#[cfg(test)] mod tests`:

```rust
#[test]
fn includes_runner_managed_source_identity_cleanup_migration() {
    let migrations = build_migrations();
    let migration = migrations
        .iter()
        .find(|migration| migration.version == 19)
        .expect("version 19 migration is registered");

    assert_eq!(
        migration.description,
        "remove legacy telegram source kind"
    );
    assert!(
        migration.sql.contains("extractum_runner_managed_migration_19"),
        "v19 must fail if plugin-managed SQLx applies it directly"
    );
}

#[test]
fn plugin_migration_list_keeps_v19_as_sentinel_only() {
    let migration = build_migrations()
        .into_iter()
        .find(|migration| migration.version == 19)
        .expect("version 19 migration is registered");

    assert!(!migration.sql.contains("DROP TABLE sources"));
    assert!(!migration.sql.contains("ALTER TABLE sources"));
    assert!(!migration.sql.contains("CREATE TABLE sources_new"));
}
```

- [x] **Step 2: Run the failing tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::tests::
```

Expected:

```text
includes_runner_managed_source_identity_cleanup_migration ... FAILED
```

- [x] **Step 3: Add sentinel migration SQL**

Create `src-tauri/migrations/19.sql` with this exact content:

```sql
-- Version 19 is applied by src-tauri/src/migrations/source_identity_cleanup.rs.
-- This sentinel is registered so SQLx validates the applied checksum, but
-- direct plugin-managed execution must fail because v19 needs a pre-transaction
-- foreign-key-off rebuild and runner-side PRAGMA foreign_key_check assertions.
SELECT extractum_runner_managed_migration_19();
```

- [x] **Step 4: Register version 19**

In `src-tauri/src/migrations.rs`, add a migration entry after version 18:

```rust
Migration {
    version: 19,
    description: "remove legacy telegram source kind",
    sql: include_str!("../migrations/19.sql"),
    kind: MigrationKind::Up,
},
```

- [x] **Step 5: Create the special migration module shell**

Create `src-tauri/src/migrations/source_identity_cleanup.rs` with this initial
content:

```rust
use std::time::Instant;

use sha2::{Digest, Sha384};
use sqlx::{Connection, Executor, SqliteConnection};

use crate::error::{AppError, AppResult};

pub(super) const SOURCE_IDENTITY_CLEANUP_VERSION: i64 = 19;
pub(super) const SOURCE_IDENTITY_CLEANUP_DESCRIPTION: &str =
    "remove legacy telegram source kind";
pub(super) const SOURCE_IDENTITY_CLEANUP_SENTINEL_SQL: &str =
    include_str!("../../migrations/19.sql");

pub(super) async fn apply_source_identity_cleanup_if_needed(
    db_url: &str,
) -> AppResult<()> {
    let mut conn = SqliteConnection::connect(db_url)
        .await
        .map_err(AppError::database)?;
    apply_source_identity_cleanup_on_connection(&mut conn).await
}

async fn apply_source_identity_cleanup_on_connection(
    conn: &mut SqliteConnection,
) -> AppResult<()> {
    ensure_not_missing_previous_migrations(conn).await?;
    if migration_19_recorded(conn).await? {
        return Ok(());
    }

    let _started_at = Instant::now();
    Err(AppError::internal(
        "source identity cleanup migration 19 is not implemented yet",
    ))
}

async fn ensure_not_missing_previous_migrations(conn: &mut SqliteConnection) -> AppResult<()> {
    let max_version: Option<i64> =
        sqlx::query_scalar("SELECT MAX(version) FROM _sqlx_migrations WHERE success = 1")
            .fetch_optional(&mut *conn)
            .await
            .map_err(AppError::database)?
            .flatten();

    match max_version {
        Some(version) if version >= 18 => Ok(()),
        Some(version) => Err(AppError::validation(format!(
            "Source identity cleanup requires migration 18 before migration 19; current migration version is {version}"
        ))),
        None => Err(AppError::validation(
            "Source identity cleanup requires migrations 1 through 18 before migration 19",
        )),
    }
}

async fn migration_19_recorded(conn: &mut SqliteConnection) -> AppResult<bool> {
    let checksum = expected_migration_19_checksum();
    let row: Option<(Vec<u8>, bool)> = sqlx::query_as(
        "SELECT checksum, success FROM _sqlx_migrations WHERE version = ?",
    )
    .bind(SOURCE_IDENTITY_CLEANUP_VERSION)
    .fetch_optional(&mut *conn)
    .await
    .map_err(AppError::database)?;

    match row {
        None => Ok(false),
        Some((applied_checksum, true)) if applied_checksum == checksum => Ok(true),
        Some((_applied_checksum, true)) => Err(AppError::internal(
            "Migration 19 checksum does not match the runner-managed source identity cleanup sentinel",
        )),
        Some((_applied_checksum, false)) => Err(AppError::internal(
            "Migration 19 is marked as failed in _sqlx_migrations",
        )),
    }
}

fn expected_migration_19_checksum() -> Vec<u8> {
    Sha384::digest(SOURCE_IDENTITY_CLEANUP_SENTINEL_SQL.as_bytes()).to_vec()
}
```

- [x] **Step 6: Wire the module into `migrations.rs` without running it yet**

At the top of `src-tauri/src/migrations.rs`, add:

```rust
mod source_identity_cleanup;
```

Do not call `apply_source_identity_cleanup_if_needed` in this task. Task 2 adds
the runner call with tests.

- [x] **Step 7: Run registration tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::tests::
```

Expected:

```text
test result: ok. 11 passed; 0 failed
```

- [x] **Step 8: Commit**

Run:

```powershell
git add src-tauri/migrations/19.sql src-tauri/src/migrations.rs src-tauri/src/migrations/source_identity_cleanup.rs
git commit -m "feat: register source identity cleanup migration"
```

Expected:

```text
[feature/source-identity-legacy-cleanup ...] feat: register source identity cleanup migration
```

---

### Task 2: Add Custom Migration Runner Before SQL Plugin Validation

**Files:**
- Modify: `src-tauri/src/migrations.rs`
- Modify: `src-tauri/src/migrations/source_identity_cleanup.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write failing tests for runner ordering**

Add these tests to `src-tauri/src/migrations.rs`:

```rust
#[test]
fn build_migrations_contains_all_versions_for_sqlx_validation() {
    let versions = build_migrations()
        .into_iter()
        .map(|migration| migration.version)
        .collect::<Vec<_>>();

    assert_eq!(versions, (1_i64..=19_i64).collect::<Vec<_>>());
}
```

Add these tests to `src-tauri/src/migrations/source_identity_cleanup.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::migrations::build_migrations;

    async fn memory_conn_with_sqlx_history_through(version: i64) -> SqliteConnection {
        let mut conn = SqliteConnection::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");

        apply_standard_migrations_through(&mut conn, version)
            .await
            .expect("apply standard migrations");

        conn
    }

    #[tokio::test]
    async fn migration_19_sentinel_checksum_is_recorded() {
        let mut conn = memory_conn_with_sqlx_history_through(18).await;

        apply_source_identity_cleanup_on_connection(&mut conn)
            .await
            .expect("apply v19");

        let row: (String, bool, Vec<u8>) = sqlx::query_as(
            "SELECT description, success, checksum FROM _sqlx_migrations WHERE version = 19",
        )
        .fetch_one(&mut conn)
        .await
        .expect("load v19 history");

        assert_eq!(row.0, SOURCE_IDENTITY_CLEANUP_DESCRIPTION);
        assert!(row.1);
        assert_eq!(row.2, expected_migration_19_checksum());
    }

    #[tokio::test]
    async fn migration_19_is_idempotent_when_checksum_matches() {
        let mut conn = memory_conn_with_sqlx_history_through(18).await;

        apply_source_identity_cleanup_on_connection(&mut conn)
            .await
            .expect("first v19");
        apply_source_identity_cleanup_on_connection(&mut conn)
            .await
            .expect("second v19");

        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations WHERE version = 19")
                .fetch_one(&mut conn)
                .await
                .expect("count v19 records");
        assert_eq!(count, 1);
    }

    async fn apply_standard_migrations_through(
        conn: &mut SqliteConnection,
        version: i64,
    ) -> AppResult<()> {
        ensure_sqlx_migrations_table(conn).await?;
        for migration in build_migrations()
            .into_iter()
            .filter(|migration| migration.version <= version)
        {
            sqlx::raw_sql(migration.sql)
                .execute(&mut *conn)
                .await
                .map_err(AppError::database)?;
            record_migration_success(
                conn,
                migration.version,
                migration.description,
                Sha384::digest(migration.sql.as_bytes()).to_vec(),
                0,
            )
            .await?;
        }
        Ok(())
    }
}
```

- [ ] **Step 2: Run the failing tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::tests::build_migrations_contains_all_versions_for_sqlx_validation migrations::source_identity_cleanup::tests::migration_19_sentinel_checksum_is_recorded migrations::source_identity_cleanup::tests::migration_19_is_idempotent_when_checksum_matches
```

Expected:

```text
migration_19_sentinel_checksum_is_recorded ... FAILED
```

- [ ] **Step 3: Add SQLx history helpers**

In `src-tauri/src/migrations/source_identity_cleanup.rs`, add:

```rust
async fn ensure_sqlx_migrations_table(conn: &mut SqliteConnection) -> AppResult<()> {
    conn.execute(sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS _sqlx_migrations (
            version BIGINT PRIMARY KEY,
            description TEXT NOT NULL,
            installed_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            success BOOLEAN NOT NULL,
            checksum BLOB NOT NULL,
            execution_time BIGINT NOT NULL
        )
        "#,
    ))
    .await
    .map_err(AppError::database)?;
    Ok(())
}

async fn record_migration_success(
    conn: &mut SqliteConnection,
    version: i64,
    description: &str,
    checksum: Vec<u8>,
    execution_time: i64,
) -> AppResult<()> {
    sqlx::query(
        r#"
        INSERT INTO _sqlx_migrations (
            version, description, success, checksum, execution_time
        )
        VALUES (?, ?, 1, ?, ?)
        "#,
    )
    .bind(version)
    .bind(description)
    .bind(checksum)
    .bind(execution_time)
    .execute(&mut *conn)
    .await
    .map_err(AppError::database)?;
    Ok(())
}
```

- [ ] **Step 4: Add standard migration pre-runner for 1..18**

In `src-tauri/src/migrations/source_identity_cleanup.rs`, add:

```rust
pub(super) async fn apply_standard_migrations_before_plugin(
    db_url: &str,
    migrations: Vec<tauri_plugin_sql::Migration>,
) -> AppResult<()> {
    let mut conn = SqliteConnection::connect(db_url)
        .await
        .map_err(AppError::database)?;
    ensure_sqlx_migrations_table(&mut conn).await?;

    for migration in migrations
        .into_iter()
        .filter(|migration| migration.version < SOURCE_IDENTITY_CLEANUP_VERSION)
    {
        let exists = migration_record_exists(&mut conn, migration.version).await?;
        if exists {
            continue;
        }

        let started_at = Instant::now();
        sqlx::raw_sql(migration.sql)
            .execute(&mut conn)
            .await
            .map_err(AppError::database)?;
        record_migration_success(
            &mut conn,
            migration.version,
            migration.description,
            Sha384::digest(migration.sql.as_bytes()).to_vec(),
            started_at.elapsed().as_nanos().min(i64::MAX as u128) as i64,
        )
        .await?;
    }

    Ok(())
}

async fn migration_record_exists(
    conn: &mut SqliteConnection,
    version: i64,
) -> AppResult<bool> {
    let exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM _sqlx_migrations WHERE version = ? AND success = 1",
    )
    .bind(version)
    .fetch_one(&mut *conn)
    .await
    .map_err(AppError::database)?;
    Ok(exists != 0)
}
```

- [ ] **Step 5: Record v19 success after the rebuild placeholder**

Replace the placeholder error in `apply_source_identity_cleanup_on_connection`
with:

```rust
let started_at = Instant::now();
run_source_identity_cleanup_rebuild(conn).await?;
record_migration_success(
    conn,
    SOURCE_IDENTITY_CLEANUP_VERSION,
    SOURCE_IDENTITY_CLEANUP_DESCRIPTION,
    expected_migration_19_checksum(),
    started_at.elapsed().as_nanos().min(i64::MAX as u128) as i64,
)
.await?;
Ok(())
```

Add a temporary no-op rebuild that Task 3 replaces:

```rust
async fn run_source_identity_cleanup_rebuild(
    _conn: &mut SqliteConnection,
) -> AppResult<()> {
    Ok(())
}
```

- [ ] **Step 6: Change `prepare_database` to return an app result**

In `src-tauri/src/migrations.rs`, change:

```rust
pub fn prepare_database() {
    if let Some(db_path) = app_config_db_path() {
        tauri::async_runtime::block_on(patch_migrations(&db_path));
    }
}
```

to:

```rust
pub fn prepare_database() -> crate::error::AppResult<()> {
    let Some(db_path) = app_config_db_path() else {
        return Ok(());
    };
    tauri::async_runtime::block_on(patch_migrations(&db_path))
}
```

Change `patch_migrations` to return `AppResult<()>`, create the parent
directory if needed, run standard migrations 1..18, then run v19:

```rust
async fn patch_migrations(db_path: &Path) -> crate::error::AppResult<()> {
    use sqlx::SqlitePool;

    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| crate::error::AppError::internal(error.to_string()))?;
    }

    let url = format!("sqlite:{}", db_path.to_string_lossy());
    source_identity_cleanup::apply_standard_migrations_before_plugin(
        &url,
        build_migrations(),
    )
    .await?;

    let pool = SqlitePool::connect(&url)
        .await
        .map_err(crate::error::AppError::database)?;
    repair_line_ending_migration_checksums(&pool).await;
    pool.close().await;

    source_identity_cleanup::apply_source_identity_cleanup_if_needed(&url).await
}
```

- [ ] **Step 7: Fail startup if preparation fails**

In `src-tauri/src/lib.rs`, replace:

```rust
prepare_database();
```

with:

```rust
prepare_database().expect("database preparation failed");
```

- [ ] **Step 8: Run the runner tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::tests::build_migrations_contains_all_versions_for_sqlx_validation migrations::source_identity_cleanup::tests::migration_19_sentinel_checksum_is_recorded migrations::source_identity_cleanup::tests::migration_19_is_idempotent_when_checksum_matches
```

Expected:

```text
test result: ok. 3 passed; 0 failed
```

- [ ] **Step 9: Commit**

Run:

```powershell
git add src-tauri/src/migrations.rs src-tauri/src/migrations/source_identity_cleanup.rs src-tauri/src/lib.rs
git commit -m "feat: run source cleanup migration before plugin validation"
```

Expected:

```text
[feature/source-identity-legacy-cleanup ...] feat: run source cleanup migration before plugin validation
```

---

### Task 3: Implement FK-Safe Sources Rebuild

**Files:**
- Modify: `src-tauri/src/migrations/source_identity_cleanup.rs`

- [ ] **Step 1: Add failing fresh-schema and index-shape tests**

Add these tests to `src-tauri/src/migrations/source_identity_cleanup.rs`:

```rust
#[tokio::test]
async fn v19_rebuild_removes_legacy_column_and_recreates_expected_indexes() {
    let mut conn = memory_conn_with_sqlx_history_through(18).await;

    apply_source_identity_cleanup_on_connection(&mut conn)
        .await
        .expect("apply v19");

    let legacy_column_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('sources') WHERE name = 'telegram_source_kind'",
    )
    .fetch_one(&mut conn)
    .await
    .expect("count legacy column");
    assert_eq!(legacy_column_count, 0);

    assert_sources_index(
        &mut conn,
        "idx_sources_unique_telegram_identity",
        true,
        &["account_id", "source_type", "source_subtype", "external_id"],
        "source_type = 'telegram'",
    )
    .await;
    assert_sources_index(
        &mut conn,
        "idx_sources_unique_youtube_video",
        true,
        &["source_type", "source_subtype", "external_id"],
        "source_type = 'youtube' AND source_subtype = 'video'",
    )
    .await;
    assert_sources_index(
        &mut conn,
        "idx_sources_unique_youtube_playlist",
        true,
        &["source_type", "source_subtype", "external_id"],
        "source_type = 'youtube' AND source_subtype = 'playlist'",
    )
    .await;

    let old_index_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_schema WHERE type = 'index' AND name = 'idx_sources_ext'",
    )
    .fetch_one(&mut conn)
    .await
    .expect("count old index");
    assert_eq!(old_index_count, 0);
}

#[tokio::test]
async fn v19_schema_checks_reject_invalid_implemented_provider_rows() {
    let mut conn = memory_conn_with_sqlx_history_through(18).await;

    apply_source_identity_cleanup_on_connection(&mut conn)
        .await
        .expect("apply v19");

    assert_insert_fails(
        &mut conn,
        "INSERT INTO sources (source_type, source_subtype, account_id, external_id, created_at) VALUES ('telegram', 'channel', NULL, '123', 1)",
    )
    .await;
    assert_insert_fails(
        &mut conn,
        "INSERT INTO sources (source_type, source_subtype, account_id, external_id, created_at) VALUES ('telegram', NULL, 1, '123', 1)",
    )
    .await;
    assert_insert_fails(
        &mut conn,
        "INSERT INTO sources (source_type, source_subtype, account_id, external_id, created_at) VALUES ('telegram', 'video', 1, '123', 1)",
    )
    .await;
    assert_insert_fails(
        &mut conn,
        "INSERT INTO sources (source_type, source_subtype, account_id, external_id, created_at) VALUES ('youtube', NULL, NULL, 'abc', 1)",
    )
    .await;
    assert_insert_fails(
        &mut conn,
        "INSERT INTO sources (source_type, source_subtype, account_id, external_id, created_at) VALUES ('youtube', 'channel', NULL, 'abc', 1)",
    )
    .await;
    assert_insert_fails(
        &mut conn,
        "INSERT INTO sources (source_type, source_subtype, account_id, external_id, created_at) VALUES ('youtube', 'video', 1, 'abc', 1)",
    )
    .await;

    sqlx::query(
        "INSERT INTO sources (source_type, source_subtype, account_id, external_id, created_at) VALUES ('rss', 'feed', NULL, 'feed-1', 1)",
    )
    .execute(&mut conn)
    .await
    .expect("rss placeholder subtype remains allowed");
}
```

Add test helpers:

```rust
async fn assert_insert_fails(conn: &mut SqliteConnection, sql: &str) {
    let error = sqlx::query(sql)
        .execute(&mut *conn)
        .await
        .expect_err("insert should fail");
    let message = error.to_string();
    assert!(
        message.contains("CHECK constraint failed")
            || message.contains("FOREIGN KEY constraint failed")
            || message.contains("UNIQUE constraint failed"),
        "unexpected error: {message}"
    );
}

async fn assert_sources_index(
    conn: &mut SqliteConnection,
    name: &str,
    unique: bool,
    columns: &[&str],
    where_clause: &str,
) {
    let row: (String, String) = sqlx::query_as(
        "SELECT tbl_name, sql FROM sqlite_schema WHERE type = 'index' AND name = ?",
    )
    .bind(name)
    .fetch_one(&mut *conn)
    .await
    .unwrap_or_else(|_| panic!("missing index {name}"));
    assert_eq!(row.0, "sources");
    assert!(
        row.1.contains("CREATE UNIQUE INDEX") == unique,
        "unexpected uniqueness for {name}: {}",
        row.1
    );
    for column in columns {
        assert!(
            row.1.contains(column),
            "index {name} SQL missing column {column}: {}",
            row.1
        );
    }
    assert!(
        row.1.contains(where_clause),
        "index {name} SQL missing WHERE clause {where_clause}: {}",
        row.1
    );
}
```

- [ ] **Step 2: Run the failing tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::source_identity_cleanup::tests::v19_rebuild_removes_legacy_column_and_recreates_expected_indexes migrations::source_identity_cleanup::tests::v19_schema_checks_reject_invalid_implemented_provider_rows
```

Expected:

```text
v19_rebuild_removes_legacy_column_and_recreates_expected_indexes ... FAILED
```

- [ ] **Step 3: Add FK check helper**

Add to `src-tauri/src/migrations/source_identity_cleanup.rs`:

```rust
#[derive(sqlx::FromRow, Debug)]
struct ForeignKeyCheckRow {
    table: String,
    rowid: Option<i64>,
    parent: String,
    fkid: i64,
}

async fn assert_foreign_key_check_clean(
    conn: &mut SqliteConnection,
    phase: &str,
) -> AppResult<()> {
    let rows: Vec<ForeignKeyCheckRow> = sqlx::query_as("PRAGMA foreign_key_check")
        .fetch_all(&mut *conn)
        .await
        .map_err(AppError::database)?;

    if rows.is_empty() {
        return Ok(());
    }

    let detail = rows
        .into_iter()
        .map(|row| {
            format!(
                "{} rowid {:?} references {} via fk {}",
                row.table, row.rowid, row.parent, row.fkid
            )
        })
        .collect::<Vec<_>>()
        .join("; ");
    Err(AppError::validation(format!(
        "Source identity cleanup foreign_key_check failed {phase}: {detail}"
    )))
}
```

- [ ] **Step 4: Add rebuild SQL helpers**

Add to `src-tauri/src/migrations/source_identity_cleanup.rs`:

```rust
async fn captured_sources_sequence(conn: &mut SqliteConnection) -> AppResult<Option<i64>> {
    let seq = sqlx::query_scalar("SELECT seq FROM sqlite_sequence WHERE name = 'sources'")
        .fetch_optional(&mut *conn)
        .await
        .map_err(AppError::database)?;
    Ok(seq)
}

async fn restore_sources_sequence(
    conn: &mut SqliteConnection,
    seq: Option<i64>,
) -> AppResult<()> {
    sqlx::query("DELETE FROM sqlite_sequence WHERE name = 'sources'")
        .execute(&mut *conn)
        .await
        .map_err(AppError::database)?;

    if let Some(seq) = seq {
        sqlx::query("INSERT INTO sqlite_sequence(name, seq) VALUES ('sources', ?)")
            .bind(seq)
            .execute(&mut *conn)
            .await
            .map_err(AppError::database)?;
    }

    Ok(())
}

async fn rebuild_sources_table(conn: &mut SqliteConnection) -> AppResult<()> {
    sqlx::raw_sql(
        r#"
        DROP INDEX IF EXISTS idx_sources_ext;
        DROP INDEX IF EXISTS idx_sources_unique_telegram_identity;
        DROP INDEX IF EXISTS idx_sources_unique_youtube_video;
        DROP INDEX IF EXISTS idx_sources_unique_youtube_playlist;

        CREATE TABLE sources_new (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            source_type TEXT NOT NULL,
            source_subtype TEXT,
            external_id TEXT NOT NULL,
            title TEXT,
            metadata_zstd BLOB,
            last_sync_state INTEGER,
            is_active BOOLEAN DEFAULT 1,
            is_member BOOLEAN DEFAULT 0,
            created_at INTEGER NOT NULL,
            account_id INTEGER REFERENCES accounts(id) ON DELETE CASCADE,
            last_synced_at INTEGER,
            CHECK (
                source_type <> 'telegram'
                OR (
                    account_id IS NOT NULL
                    AND source_subtype IS NOT NULL
                    AND source_subtype IN ('channel', 'supergroup', 'group')
                )
            ),
            CHECK (
                source_type <> 'youtube'
                OR (
                    account_id IS NULL
                    AND source_subtype IS NOT NULL
                    AND source_subtype IN ('video', 'playlist')
                )
            )
        );

        INSERT INTO sources_new (
            id, source_type, source_subtype, external_id, title, metadata_zstd,
            last_sync_state, is_active, is_member, created_at, account_id,
            last_synced_at
        )
        SELECT
            id, source_type, source_subtype, external_id, title, metadata_zstd,
            last_sync_state, is_active, is_member, created_at, account_id,
            last_synced_at
        FROM sources;

        DROP TABLE sources;
        ALTER TABLE sources_new RENAME TO sources;

        CREATE UNIQUE INDEX idx_sources_unique_telegram_identity
            ON sources(account_id, source_type, source_subtype, external_id)
            WHERE source_type = 'telegram';

        CREATE UNIQUE INDEX idx_sources_unique_youtube_video
            ON sources(source_type, source_subtype, external_id)
            WHERE source_type = 'youtube' AND source_subtype = 'video';

        CREATE UNIQUE INDEX idx_sources_unique_youtube_playlist
            ON sources(source_type, source_subtype, external_id)
            WHERE source_type = 'youtube' AND source_subtype = 'playlist';
        "#,
    )
    .execute(&mut *conn)
    .await
    .map_err(AppError::database)?;
    Ok(())
}
```

- [ ] **Step 5: Implement FK-safe v19 rebuild sequence**

Replace the temporary no-op `run_source_identity_cleanup_rebuild` with:

```rust
async fn run_source_identity_cleanup_rebuild(
    conn: &mut SqliteConnection,
) -> AppResult<()> {
    let sequence = captured_sources_sequence(conn).await?;

    sqlx::query("PRAGMA foreign_keys = OFF")
        .execute(&mut *conn)
        .await
        .map_err(AppError::database)?;
    let foreign_keys: i64 = sqlx::query_scalar("PRAGMA foreign_keys")
        .fetch_one(&mut *conn)
        .await
        .map_err(AppError::database)?;
    if foreign_keys != 0 {
        return Err(AppError::internal(
            "SQLite foreign_keys stayed enabled before source identity cleanup rebuild",
        ));
    }

    sqlx::query("BEGIN IMMEDIATE")
        .execute(&mut *conn)
        .await
        .map_err(AppError::database)?;

    let rebuild_result = async {
        rebuild_sources_table(conn).await?;
        restore_sources_sequence(conn, sequence).await?;
        assert_foreign_key_check_clean(conn, "inside v19 transaction").await
    }
    .await;

    match rebuild_result {
        Ok(()) => {
            sqlx::query("COMMIT")
                .execute(&mut *conn)
                .await
                .map_err(AppError::database)?;
        }
        Err(error) => {
            let _ = sqlx::query("ROLLBACK").execute(&mut *conn).await;
            let _ = sqlx::query("PRAGMA foreign_keys = ON")
                .execute(&mut *conn)
                .await;
            return Err(error);
        }
    }

    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&mut *conn)
        .await
        .map_err(AppError::database)?;
    assert_foreign_key_check_clean(conn, "after v19 commit").await
}
```

- [ ] **Step 6: Run the schema tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::source_identity_cleanup::tests::v19_rebuild_removes_legacy_column_and_recreates_expected_indexes migrations::source_identity_cleanup::tests::v19_schema_checks_reject_invalid_implemented_provider_rows
```

Expected:

```text
test result: ok. 2 passed; 0 failed
```

- [ ] **Step 7: Commit**

Run:

```powershell
git add src-tauri/src/migrations/source_identity_cleanup.rs
git commit -m "feat: rebuild sources without legacy telegram kind"
```

Expected:

```text
[feature/source-identity-legacy-cleanup ...] feat: rebuild sources without legacy telegram kind
```

---

### Task 4: Preserve Source Graphs And Harden Migration Failures

**Files:**
- Modify: `src-tauri/src/migrations/source_identity_cleanup.rs`

- [ ] **Step 1: Add failing source graph preservation test**

Add a test that constructs representative child rows before v19 and verifies
their source ids after v19:

```rust
#[tokio::test]
async fn v19_preserves_source_ids_sequence_and_reference_graph() {
    let mut conn = memory_conn_with_sqlx_history_through(18).await;
    seed_repaired_v18_graph(&mut conn).await;

    sqlx::query("UPDATE sqlite_sequence SET seq = 500 WHERE name = 'sources'")
        .execute(&mut conn)
        .await
        .expect("raise sources sequence");

    apply_source_identity_cleanup_on_connection(&mut conn)
        .await
        .expect("apply v19");

    let source_ids: Vec<i64> = sqlx::query_scalar("SELECT id FROM sources ORDER BY id")
        .fetch_all(&mut conn)
        .await
        .expect("load source ids");
    assert_eq!(source_ids, vec![101, 201, 202]);

    let sequence: i64 = sqlx::query_scalar("SELECT seq FROM sqlite_sequence WHERE name = 'sources'")
        .fetch_one(&mut conn)
        .await
        .expect("load sequence");
    assert_eq!(sequence, 500);

    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT source_id FROM items WHERE id = 301")
            .fetch_one(&mut conn)
            .await
            .expect("items source id"),
        101
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT source_id FROM telegram_sources WHERE source_id = 101")
            .fetch_one(&mut conn)
            .await
            .expect("telegram typed source id"),
        101
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT source_id FROM telegram_forum_topics WHERE id = 401")
            .fetch_one(&mut conn)
            .await
            .expect("topic source id"),
        101
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT source_id FROM source_identity_repair_notes WHERE id = 501")
            .fetch_one(&mut conn)
            .await
            .expect("repair note source id"),
        101
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT playlist_source_id FROM youtube_playlist_items WHERE id = 601")
            .fetch_one(&mut conn)
            .await
            .expect("playlist source id"),
        202
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT video_source_id FROM youtube_playlist_items WHERE id = 601")
            .fetch_one(&mut conn)
            .await
            .expect("video source id"),
        201
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT source_id FROM youtube_transcript_segments WHERE id = 701")
            .fetch_one(&mut conn)
            .await
            .expect("transcript source id"),
        201
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT source_id FROM analysis_runs WHERE id = 801")
            .fetch_one(&mut conn)
            .await
            .expect("analysis run logical source id"),
        101
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT source_id FROM analysis_run_messages WHERE run_id = 801 AND ref = '1'",
        )
            .fetch_one(&mut conn)
            .await
            .expect("analysis message logical source id"),
        101
    );

    assert_foreign_key_check_clean(&mut conn, "test after graph migration")
        .await
        .expect("foreign keys clean");
}
```

Add `seed_repaired_v18_graph` with explicit inserts for accounts, three
sources, typed Telegram row, one row for every physical FK table named in the
spec, and logical `analysis_runs`/`analysis_run_messages` rows:

```rust
async fn seed_repaired_v18_graph(conn: &mut SqliteConnection) {
    sqlx::query(
        "INSERT INTO accounts (id, label, api_id, api_hash, created_at) VALUES (1, 'acct', 1, 'hash', 1)",
    )
    .execute(&mut *conn)
    .await
    .expect("insert account");
    sqlx::query(
        r#"
        INSERT INTO sources (
            id, source_type, source_subtype, telegram_source_kind, account_id,
            external_id, title, is_active, is_member, created_at
        )
        VALUES
            (101, 'telegram', 'supergroup', 'supergroup', 1, '12345', 'Forum', 1, 1, 10),
            (201, 'youtube', 'video', '', NULL, 'video-1', 'Video', 1, 0, 11),
            (202, 'youtube', 'playlist', '', NULL, 'playlist-1', 'Playlist', 1, 0, 12)
        "#,
    )
    .execute(&mut *conn)
    .await
    .expect("insert sources");
    sqlx::query(
        r#"
        INSERT INTO telegram_sources (
            source_id, account_id, source_subtype, peer_kind, peer_id,
            resolution_strategy, username
        )
        VALUES (101, 1, 'supergroup', 'channel', 12345, 'username', 'forum')
        "#,
    )
    .execute(&mut *conn)
    .await
    .expect("insert typed telegram source");
    sqlx::query(
        "INSERT INTO items (id, source_id, external_id, author, published_at, ingested_at, content_kind, item_kind) VALUES (301, 101, 'msg-1', 'alice', 1, 1, 'text_only', 'telegram_message')",
    )
    .execute(&mut *conn)
    .await
    .expect("insert item");
    sqlx::query(
        "INSERT INTO analysis_source_groups (id, name, created_at, updated_at, source_type) VALUES (1, 'group', 1, 1, 'telegram')",
    )
    .execute(&mut *conn)
    .await
    .expect("insert analysis source group");
    sqlx::query(
        "INSERT INTO analysis_source_group_members (group_id, source_id, created_at) VALUES (1, 101, 1)",
    )
    .execute(&mut *conn)
    .await
    .expect("insert analysis group member");
    sqlx::query(
        "INSERT INTO telegram_forum_topics (id, source_id, topic_id, top_message_id, title, last_seen_at, updated_at) VALUES (401, 101, 77, 88, 'topic', 1, 1)",
    )
    .execute(&mut *conn)
    .await
    .expect("insert topic");
    sqlx::query(
        "INSERT INTO source_identity_repair_notes (id, source_id, issue_code, detail, created_at) VALUES (501, 101, 'note', 'detail', 1)",
    )
    .execute(&mut *conn)
    .await
    .expect("insert repair note");
    sqlx::query(
        "INSERT INTO youtube_playlist_items (id, playlist_source_id, video_source_id, video_id, availability_status) VALUES (601, 202, 201, 'video-1', 'available')",
    )
    .execute(&mut *conn)
    .await
    .expect("insert playlist item");
    sqlx::query(
        "INSERT INTO youtube_transcript_segments (id, item_id, source_id, segment_index, start_ms, text) VALUES (701, 301, 201, 0, 0, 'caption')",
    )
    .execute(&mut *conn)
    .await
    .expect("insert transcript segment");
    sqlx::query(
        "INSERT INTO analysis_runs (id, run_type, scope_type, source_id, period_from, period_to, output_language, prompt_template_version, provider_profile, provider, model, status, created_at) VALUES (801, 'single_source', 'source', 101, 1, 2, 'en', 1, 'default', 'openai', 'gpt', 'completed', 1)",
    )
    .execute(&mut *conn)
    .await
    .expect("insert analysis run");
    sqlx::query(
        "INSERT INTO analysis_run_messages (run_id, item_id, source_id, external_id, author, published_at, ref, content_zstd, item_kind, source_type, source_subtype) VALUES (801, 301, 101, 'msg-1', 'alice', 1, '1', x'00', 'telegram_message', 'telegram', 'supergroup')",
    )
    .execute(&mut *conn)
    .await
    .expect("insert analysis run message");
}
```

- [ ] **Step 2: Add failing negative migration tests**

Add one parameterized test for invalid rows:

```rust
#[tokio::test]
async fn v19_rejects_invalid_repaired_v18_inputs_without_partial_schema() {
    for case in [
        InvalidV18Case::TelegramNullAccount,
        InvalidV18Case::TelegramNullSubtype,
        InvalidV18Case::DuplicateCanonicalTelegramIdentity,
        InvalidV18Case::DuplicateTypedTelegramPeer,
        InvalidV18Case::InvalidYoutubeSubtype,
        InvalidV18Case::YoutubeAccountId,
    ] {
        let mut conn = memory_conn_with_sqlx_history_through(18).await;
        seed_invalid_v18_case(&mut conn, case).await;

        let error = apply_source_identity_cleanup_on_connection(&mut conn)
            .await
            .expect_err("invalid v18 input must fail");
        assert!(
            error.message.contains("Source identity cleanup")
                || error.message.contains("Database error"),
            "unexpected error for {case:?}: {}",
            error.message
        );
        assert_failed_v19_left_old_sources_table(&mut conn).await;
    }
}
```

Add the enum and helpers:

```rust
#[derive(Clone, Copy, Debug)]
enum InvalidV18Case {
    TelegramNullAccount,
    TelegramNullSubtype,
    DuplicateCanonicalTelegramIdentity,
    DuplicateTypedTelegramPeer,
    InvalidYoutubeSubtype,
    YoutubeAccountId,
}

async fn assert_failed_v19_left_old_sources_table(conn: &mut SqliteConnection) {
    let legacy_column_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM pragma_table_info('sources') WHERE name = 'telegram_source_kind'",
    )
    .fetch_one(&mut *conn)
    .await
    .expect("count legacy column after failed v19");
    assert_eq!(legacy_column_count, 1);

    let partial_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_schema WHERE type = 'table' AND name = 'sources_new'",
    )
    .fetch_one(&mut *conn)
    .await
    .expect("count sources_new");
    assert_eq!(partial_count, 0);
}
```

Add `seed_invalid_v18_case`:

```rust
async fn seed_invalid_v18_case(conn: &mut SqliteConnection, case: InvalidV18Case) {
    sqlx::query(
        "INSERT INTO accounts (id, label, api_id, api_hash, created_at) VALUES (1, 'acct', 1, 'hash', 1)",
    )
    .execute(&mut *conn)
    .await
    .expect("insert account");

    match case {
        InvalidV18Case::TelegramNullAccount => {
            sqlx::query(
                r#"
                INSERT INTO sources (
                    id, source_type, source_subtype, telegram_source_kind,
                    account_id, external_id, title, created_at
                )
                VALUES (101, 'telegram', 'channel', 'channel', NULL, '12345', 'source', 1)
                "#,
            )
            .execute(&mut *conn)
            .await
            .expect("insert null-account telegram source");
        }
        InvalidV18Case::TelegramNullSubtype => {
            sqlx::query(
                r#"
                INSERT INTO sources (
                    id, source_type, source_subtype, telegram_source_kind,
                    account_id, external_id, title, created_at
                )
                VALUES (101, 'telegram', NULL, 'channel', 1, '12345', 'source', 1)
                "#,
            )
            .execute(&mut *conn)
            .await
            .expect("insert null-subtype telegram source");
        }
        InvalidV18Case::DuplicateCanonicalTelegramIdentity => {
            sqlx::query(
                r#"
                INSERT INTO sources (
                    id, source_type, source_subtype, telegram_source_kind,
                    account_id, external_id, title, created_at
                )
                VALUES
                    (101, 'telegram', 'channel', 'channel', 1, '12345', 'one', 1),
                    (102, 'telegram', 'channel', 'channel', 1, '12345', 'two', 2)
                "#,
            )
            .execute(&mut *conn)
            .await
            .expect("insert duplicate canonical sources");
            insert_typed_telegram_projection(conn, 101, "channel", "channel", 12345).await;
            insert_typed_telegram_projection(conn, 102, "channel", "channel", 67890).await;
        }
        InvalidV18Case::DuplicateTypedTelegramPeer => {
            sqlx::query(
                "DROP INDEX IF EXISTS idx_telegram_sources_account_peer",
            )
            .execute(&mut *conn)
            .await
            .expect("drop typed peer unique index for fixture");
            sqlx::query(
                r#"
                INSERT INTO sources (
                    id, source_type, source_subtype, telegram_source_kind,
                    account_id, external_id, title, created_at
                )
                VALUES
                    (101, 'telegram', 'channel', 'channel', 1, '12345', 'one', 1),
                    (102, 'telegram', 'supergroup', 'supergroup', 1, '67890', 'two', 2)
                "#,
            )
            .execute(&mut *conn)
            .await
            .expect("insert typed duplicate sources");
            insert_typed_telegram_projection(conn, 101, "channel", "channel", 12345).await;
            insert_typed_telegram_projection(conn, 102, "supergroup", "channel", 12345).await;
        }
        InvalidV18Case::InvalidYoutubeSubtype => {
            sqlx::query(
                r#"
                INSERT INTO sources (
                    id, source_type, source_subtype, telegram_source_kind,
                    account_id, external_id, title, created_at
                )
                VALUES (201, 'youtube', 'channel', '', NULL, 'video-1', 'video', 1)
                "#,
            )
            .execute(&mut *conn)
            .await
            .expect("insert invalid youtube subtype");
        }
        InvalidV18Case::YoutubeAccountId => {
            sqlx::query(
                r#"
                INSERT INTO sources (
                    id, source_type, source_subtype, telegram_source_kind,
                    account_id, external_id, title, created_at
                )
                VALUES (201, 'youtube', 'video', '', 1, 'video-1', 'video', 1)
                "#,
            )
            .execute(&mut *conn)
            .await
            .expect("insert youtube account id");
        }
    }
}

async fn insert_typed_telegram_projection(
    conn: &mut SqliteConnection,
    source_id: i64,
    source_subtype: &str,
    peer_kind: &str,
    peer_id: i64,
) {
    sqlx::query(
        r#"
        INSERT INTO telegram_sources (
            source_id, account_id, source_subtype, peer_kind, peer_id,
            resolution_strategy
        )
        VALUES (?, 1, ?, ?, ?, 'unknown')
        "#,
    )
    .bind(source_id)
    .bind(source_subtype)
    .bind(peer_kind)
    .bind(peer_id)
    .execute(&mut *conn)
    .await
    .expect("insert typed telegram projection");
}
```

- [ ] **Step 3: Run failing graph and negative tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::source_identity_cleanup::tests::v19_preserves_source_ids_sequence_and_reference_graph migrations::source_identity_cleanup::tests::v19_rejects_invalid_repaired_v18_inputs_without_partial_schema
```

Expected:

```text
v19_preserves_source_ids_sequence_and_reference_graph ... FAILED
```

- [ ] **Step 4: Add preflight diagnostics before table rebuild**

Before `rebuild_sources_table(conn).await?;` in
`run_source_identity_cleanup_rebuild`, call:

```rust
preflight_sources_for_v19(conn).await?;
```

Add:

```rust
async fn preflight_sources_for_v19(conn: &mut SqliteConnection) -> AppResult<()> {
    let invalid_telegram: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM sources
        WHERE source_type = 'telegram'
          AND (
              account_id IS NULL
              OR source_subtype IS NULL
              OR source_subtype NOT IN ('channel', 'supergroup', 'group')
          )
        "#,
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(AppError::database)?;
    if invalid_telegram != 0 {
        return Err(AppError::validation(
            "Source identity cleanup migration 19 requires repaired Telegram sources with account_id and supported source_subtype",
        ));
    }

    let invalid_youtube: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM sources
        WHERE source_type = 'youtube'
          AND (
              account_id IS NOT NULL
              OR source_subtype IS NULL
              OR source_subtype NOT IN ('video', 'playlist')
          )
        "#,
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(AppError::database)?;
    if invalid_youtube != 0 {
        return Err(AppError::validation(
            "Source identity cleanup migration 19 requires YouTube sources with account_id NULL and subtype video or playlist",
        ));
    }

    let duplicate_typed: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM (
            SELECT account_id, peer_kind, peer_id
            FROM telegram_sources
            GROUP BY account_id, peer_kind, peer_id
            HAVING COUNT(*) > 1
        )
        "#,
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(AppError::database)?;
    if duplicate_typed != 0 {
        return Err(AppError::validation(
            "Source identity cleanup migration 19 found duplicate typed Telegram peer identity",
        ));
    }

    Ok(())
}
```

The unique index creation in `rebuild_sources_table` handles duplicate
canonical Telegram identity and duplicate YouTube identity.

- [ ] **Step 5: Add unsupported pre-v18 direct-upgrade guard test**

Add:

```rust
#[tokio::test]
async fn pre_v18_database_with_telegram_rows_gets_repair_window_error() {
    let mut conn = memory_conn_with_sqlx_history_through(17).await;
    sqlx::query(
        "INSERT INTO accounts (id, label, api_id, api_hash, created_at) VALUES (1, 'acct', 1, 'hash', 1)",
    )
    .execute(&mut conn)
    .await
    .expect("insert account");
    sqlx::query(
        r#"
        INSERT INTO sources (
            id, source_type, source_subtype, telegram_source_kind, account_id,
            external_id, title, created_at
        )
        VALUES (101, 'telegram', 'channel', 'channel', 1, '12345', 'source', 1)
        "#,
    )
    .execute(&mut conn)
    .await
    .expect("insert pre-v18 telegram source");

    let error = reject_unsupported_pre_v18_telegram_upgrade(&mut conn)
        .await
        .expect_err("pre-v18 telegram upgrade should fail");
    assert!(error.message.contains("v18 source identity repair build"));
    assert!(error.message.contains("repaired backup"));
}
```

Add the production helper and call it from
`apply_standard_migrations_before_plugin` before applying pending migrations:

```rust
async fn reject_unsupported_pre_v18_telegram_upgrade(
    conn: &mut SqliteConnection,
) -> AppResult<()> {
    let max_version: Option<i64> =
        sqlx::query_scalar("SELECT MAX(version) FROM _sqlx_migrations WHERE success = 1")
            .fetch_optional(&mut *conn)
            .await
            .map_err(AppError::database)?
            .flatten();
    if max_version.unwrap_or(0) >= 18 {
        return Ok(());
    }

    let sources_exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sqlite_schema WHERE type = 'table' AND name = 'sources'",
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(AppError::database)?;
    if sources_exists == 0 {
        return Ok(());
    }

    let telegram_rows: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sources WHERE source_type IN ('telegram', 'telegram_channel')",
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(AppError::database)?;
    if telegram_rows == 0 {
        return Ok(());
    }

    Err(AppError::validation(
        "Source identity cleanup cannot upgrade pre-v18 databases with Telegram rows directly. Open the database with a v18 source identity repair build first, or restore a repaired backup before applying migration 19.",
    ))
}
```

- [ ] **Step 6: Run the migration hardening tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::source_identity_cleanup::tests::v19_preserves_source_ids_sequence_and_reference_graph migrations::source_identity_cleanup::tests::v19_rejects_invalid_repaired_v18_inputs_without_partial_schema migrations::source_identity_cleanup::tests::pre_v18_database_with_telegram_rows_gets_repair_window_error
```

Expected:

```text
test result: ok. 3 passed; 0 failed
```

- [ ] **Step 7: Commit**

Run:

```powershell
git add src-tauri/src/migrations/source_identity_cleanup.rs
git commit -m "test: harden source cleanup migration safety"
```

Expected:

```text
[feature/source-identity-legacy-cleanup ...] test: harden source cleanup migration safety
```

---

### Task 5: Tighten Canonical Telegram Identity And Repair Gate

**Files:**
- Modify: `src-tauri/src/sources/identity.rs`
- Modify: `src-tauri/src/sources/identity_repair.rs`
- Modify: `src-tauri/src/sources/test_support.rs`

- [ ] **Step 1: Add failing canonical external id tests**

In `src-tauri/src/sources/identity.rs`, extend
`canonical_external_id_rejects_malformed_values`:

```rust
#[test]
fn canonical_external_id_rejects_malformed_values() {
    for value in [
        "",
        "0",
        "00123",
        "-123",
        "+123",
        " 123",
        "123 ",
        "@name",
        "name",
        "telegram:123",
        "12a3",
        "１２３",
    ] {
        assert!(
            canonical_telegram_external_id(value).is_err(),
            "{value} should be rejected"
        );
    }
    assert_eq!(canonical_telegram_external_id("123").unwrap(), 123);
}
```

- [ ] **Step 2: Add failing repair tests without legacy column**

In `src-tauri/src/sources/identity_repair.rs`, add tests:

```rust
#[tokio::test]
async fn repair_reads_post_v19_sources_without_legacy_column() {
    let pool = post_v19_repair_pool().await;
    insert_test_account(&pool, 1).await;
    insert_post_v19_telegram_source(&pool, 101, Some("channel"), Some(1), "12345", None).await;

    let report = repair_source_identity(&pool, SourceIdentityRepairMode::Apply)
        .await
        .expect("repair succeeds without legacy column");

    assert_eq!(report.repaired_sources, vec![101]);
    let typed: (i64, String, i64) =
        sqlx::query_as("SELECT account_id, source_subtype, peer_id FROM telegram_sources WHERE source_id = 101")
            .fetch_one(&pool)
            .await
            .expect("typed row");
    assert_eq!(typed, (1, "channel".to_string(), 12345));
}

#[tokio::test]
async fn repair_rejects_zero_external_id() {
    let pool = post_v19_repair_pool().await;
    insert_test_account(&pool, 1).await;
    insert_post_v19_telegram_source(&pool, 101, Some("channel"), Some(1), "0", None).await;

    let error = repair_source_identity(&pool, SourceIdentityRepairMode::Apply)
        .await
        .expect_err("zero external id fails");

    assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
    assert!(error.message.contains("malformed_telegram_external_id"));
}

#[tokio::test]
async fn repair_treats_typed_projection_mismatch_as_fatal() {
    let pool = post_v19_repair_pool().await;
    insert_test_account(&pool, 1).await;
    insert_post_v19_telegram_source(&pool, 101, Some("channel"), Some(1), "12345", None).await;
    sqlx::query(
        r#"
        INSERT INTO telegram_sources (
            source_id, account_id, source_subtype, peer_kind, peer_id,
            resolution_strategy
        )
        VALUES (101, 1, 'channel', 'channel', 67890, 'unknown')
        "#,
    )
    .execute(&pool)
    .await
    .expect("insert conflicting typed row");

    let error = repair_source_identity(&pool, SourceIdentityRepairMode::Apply)
        .await
        .expect_err("projection drift fails");

    assert!(error.message.contains("telegram_projection_drift_conflict"));
}
```

Add helpers:

```rust
async fn post_v19_repair_pool() -> sqlx::SqlitePool {
    let pool = crate::sources::test_support::memory_pool().await;
    sqlx::query(
        r#"
        CREATE TABLE accounts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            label TEXT NOT NULL,
            api_id INTEGER NOT NULL,
            api_hash TEXT NOT NULL,
            phone TEXT,
            created_at INTEGER NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("create accounts");
    sqlx::query(
        r#"
        CREATE TABLE sources (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            source_type TEXT NOT NULL,
            source_subtype TEXT,
            external_id TEXT NOT NULL,
            title TEXT,
            metadata_zstd BLOB,
            last_sync_state INTEGER,
            is_active BOOLEAN DEFAULT 1,
            is_member BOOLEAN DEFAULT 0,
            created_at INTEGER NOT NULL,
            account_id INTEGER REFERENCES accounts(id) ON DELETE CASCADE,
            last_synced_at INTEGER
        )
        "#,
    )
    .execute(&pool)
    .await
    .expect("create post-v19 sources");
    crate::sources::test_support::create_source_identity_tables(&pool).await;
    pool
}

async fn insert_post_v19_telegram_source(
    pool: &sqlx::SqlitePool,
    id: i64,
    subtype: Option<&str>,
    account_id: Option<i64>,
    external_id: &str,
    metadata_json: Option<&[u8]>,
) {
    let metadata_zstd = metadata_json
        .map(compress_json_bytes)
        .transpose()
        .expect("compress metadata");
    sqlx::query(
        r#"
        INSERT INTO sources (
            id, source_type, source_subtype, account_id, external_id,
            title, metadata_zstd, is_active, is_member, created_at
        )
        VALUES (?, 'telegram', ?, ?, ?, 'source', ?, 1, 1, 100)
        "#,
    )
    .bind(id)
    .bind(subtype)
    .bind(account_id)
    .bind(external_id)
    .bind(metadata_zstd)
    .execute(pool)
    .await
    .expect("insert post-v19 source");
}
```

- [ ] **Step 3: Run failing repair tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::identity::tests::canonical_external_id_rejects_malformed_values sources::identity_repair::tests::repair_reads_post_v19_sources_without_legacy_column sources::identity_repair::tests::repair_rejects_zero_external_id sources::identity_repair::tests::repair_treats_typed_projection_mismatch_as_fatal
```

Expected:

```text
canonical_external_id_rejects_malformed_values ... FAILED
```

- [ ] **Step 4: Tighten canonical external id parser**

Replace `canonical_telegram_external_id` in `src-tauri/src/sources/identity.rs`
with:

```rust
pub(crate) fn canonical_telegram_external_id(value: &str) -> AppResult<i64> {
    if value.is_empty()
        || value.starts_with('0')
        || !value.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(AppError::validation(
            "Malformed Telegram external_id for source identity",
        ));
    }

    let parsed = value
        .parse::<i64>()
        .map_err(|_| AppError::validation("Malformed Telegram external_id for source identity"))?;
    if parsed <= 0 || parsed.to_string() != value {
        return Err(AppError::validation(
            "Malformed Telegram external_id for source identity",
        ));
    }
    Ok(parsed)
}
```

- [ ] **Step 5: Remove legacy repair column reads/writes**

In `src-tauri/src/sources/identity_repair.rs`:

1. Remove `telegram_source_kind` from `TelegramSourceRepairRow`.
2. Change the query to:

```rust
SELECT id, source_subtype, account_id, external_id, metadata_zstd
FROM sources
WHERE source_type = 'telegram'
ORDER BY id
```

3. Replace `derive_source_subtype` with:

```rust
fn derive_source_subtype(
    row: &TelegramSourceRepairRow,
) -> Result<TelegramSourceKind, SourceIdentityRepairDiagnostic> {
    row.source_subtype
        .as_deref()
        .map(TelegramSourceKind::from_source_subtype)
        .transpose()
        .map_err(|_| SourceIdentityRepairDiagnostic {
            code: "unsupported_telegram_source_subtype".to_string(),
            source_ids: vec![row.id],
            detail: format!("Telegram source {} has unsupported source_subtype", row.id),
        })?
        .ok_or_else(|| SourceIdentityRepairDiagnostic {
            code: "unsupported_telegram_source_subtype".to_string(),
            source_ids: vec![row.id],
            detail: format!("Telegram source {} has no supported source_subtype", row.id),
        })
}
```

4. Delete this write from apply mode:

```rust
sqlx::query(
    "UPDATE sources SET source_subtype = ?, telegram_source_kind = ? WHERE id = ?",
)
```

Repair should only upsert `telegram_sources` after v19.

- [ ] **Step 6: Make malformed metadata non-fatal when identity is enough**

In `candidate_from_row`, change metadata decoding so a bad
`metadata_zstd` yields empty optional hints instead of a fatal error:

```rust
let metadata = decode_source_metadata(row.metadata_zstd.as_deref()).ok();
let identity = metadata
    .as_ref()
    .and_then(|metadata| metadata.peer_identity.as_ref());
let strategy = match identity.map(|identity| identity.strategy) {
    Some(SourcePeerResolutionStrategy::Username) => TelegramResolutionStrategy::Username,
    Some(SourcePeerResolutionStrategy::Dialog) => TelegramResolutionStrategy::Dialog,
    None => TelegramResolutionStrategy::Unknown,
};
```

Set `avatar_cache_key` from `metadata.and_then(|metadata| metadata.avatar_cache_key)`.

- [ ] **Step 7: Run repair tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::identity::tests::canonical_external_id_rejects_malformed_values sources::identity_repair
```

Expected:

```text
test result: ok. ... 0 failed
```

- [ ] **Step 8: Commit**

Run:

```powershell
git add src-tauri/src/sources/identity.rs src-tauri/src/sources/identity_repair.rs src-tauri/src/sources/test_support.rs
git commit -m "feat: make source repair use canonical identity only"
```

Expected:

```text
[feature/source-identity-legacy-cleanup ...] feat: make source repair use canonical identity only
```

---

### Task 6: Remove Backend DTO And Store Legacy Fields

**Files:**
- Modify: `src-tauri/src/sources/types.rs`
- Modify: `src-tauri/src/sources/store.rs`
- Modify: `src-tauri/src/sources/peer_resolution.rs`
- Modify: `src-tauri/src/sources/sync.rs`
- Modify: `src-tauri/src/sources/topics.rs`
- Modify: `src-tauri/src/sources/test_support.rs`

- [ ] **Step 1: Add failing backend API/store tests**

In `src-tauri/src/sources/store.rs`, replace existing mirror tests with:

```rust
#[test]
fn source_record_parts_emit_only_source_subtype() {
    let record = source_record_from_row_parts(
        SourceRecordRow {
            id: 1,
            source_type: TELEGRAM_SOURCE_TYPE.to_string(),
            source_subtype: Some("supergroup".to_string()),
            account_id: Some(1),
            external_id: "12345".to_string(),
            title: Some("source".to_string()),
            metadata_zstd: None,
            last_sync_state: None,
            last_synced_at: None,
            is_active: true,
            is_member: true,
            created_at: 100,
            telegram_username: Some("example".to_string()),
            telegram_avatar_cache_key: None,
        },
        Some("example".to_string()),
        None,
    );

    let json = serde_json::to_value(&record).expect("serialize source record");
    assert_eq!(json["source_subtype"], "supergroup");
    assert!(json.get("telegram_source_kind").is_none());
}
```

In `src-tauri/src/sources/peer_resolution.rs`, rename the expected-kind test to
subtype vocabulary and assert the error text no longer names the old field:

```rust
#[test]
fn validate_expected_telegram_source_subtype_reports_requested_and_actual_subtype() {
    let source = ResolvedTelegramSource {
        external_id: "123".to_string(),
        title: "Example".to_string(),
        telegram_source_kind: TELEGRAM_KIND_SUPERGROUP.to_string(),
        is_member: true,
        username: Some("example".to_string()),
        access_hash: Some(42),
        avatar_bytes: None,
    };

    let error = validate_expected_telegram_source_subtype(
        &source,
        Some(TELEGRAM_KIND_CHANNEL),
    )
    .expect_err("mismatch fails");

    assert!(error.message.contains("requested source subtype"));
    assert!(!error.message.contains("telegram_source_kind"));
}
```

- [ ] **Step 2: Run failing backend tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::store::tests::source_record_parts_emit_only_source_subtype sources::peer_resolution::tests::validate_expected_telegram_source_subtype_reports_requested_and_actual_subtype
```

Expected:

```text
source_record_parts_emit_only_source_subtype ... FAILED
```

- [ ] **Step 3: Remove legacy fields from backend types**

In `src-tauri/src/sources/types.rs`:

1. Change `TelegramSourceInfo` field:

```rust
pub source_subtype: String,
```

2. Remove `telegram_source_kind` from `SourceRecord`.
3. Remove `telegram_source_kind` from `SourceSyncTarget`.
4. Remove `telegram_source_kind` from `SourceRecordRow`.

- [ ] **Step 4: Rename add-source request field**

In `src-tauri/src/sources/store.rs`, change:

```rust
pub expected_kind: Option<TelegramSourceKind>,
```

to:

```rust
pub expected_subtype: Option<TelegramSourceKind>,
```

Because the struct uses `#[serde(rename_all = "camelCase")]`, the frontend
wire key becomes `expectedSubtype` and `expectedKind` is no longer accepted.

- [ ] **Step 5: Stop selecting and writing the removed column**

In `src-tauri/src/sources/store.rs`:

1. Change `load_source` query to:

```rust
"SELECT id, source_type, source_subtype, account_id, external_id, title, metadata_zstd, last_sync_state FROM sources WHERE id = ?"
```

2. Change `load_source_record` and `list_sources` SELECT lists to remove
   `s.telegram_source_kind`.
3. Change YouTube inserts to remove `telegram_source_kind` from column lists
   and values.
4. Change Telegram insert to remove `telegram_source_kind` from column lists,
   values, and update assignments.
5. Use:

```rust
let expected_subtype = request.expected_subtype.map(TelegramSourceKind::as_str);
let resolved = resolve_telegram_source(&client, &request.source_ref, expected_subtype).await?;
```

6. Bind `&resolved.telegram_source_kind` only once as `source_subtype`.

- [ ] **Step 6: Stop emitting the persisted mirror**

In `source_record_from_row_parts`, remove mirror creation and return:

```rust
SourceRecord {
    id: row.id,
    source_type: row.source_type,
    source_subtype,
    account_id: row.account_id,
    external_id: row.external_id,
    title: row.title,
    last_sync_state: row.last_sync_state,
    last_synced_at: row.last_synced_at,
    is_member: row.is_member,
    is_active: row.is_active,
    created_at: row.created_at,
    telegram_username,
    avatar_data_url,
}
```

- [ ] **Step 7: Rename live dialog DTO in peer resolution**

In `src-tauri/src/sources/peer_resolution.rs`:

1. Keep `ResolvedTelegramSource.telegram_source_kind` internal if changing it
   creates churn.
2. Change `telegram_source_info_from_peer` to fill
   `TelegramSourceInfo.source_subtype`.
3. Rename helper arguments and error messages from expected kind to expected
   subtype:

```rust
fn validate_expected_telegram_source_subtype(
    source: &ResolvedTelegramSource,
    expected_subtype: Option<&str>,
) -> AppResult<()> {
    if telegram_source_subtype_matches(source, expected_subtype)? {
        return Ok(());
    }

    Err(AppError::validation(format!(
        "Resolved Telegram source has a different source subtype than the requested source subtype: requested {}, actual {}",
        expected_subtype.unwrap_or("unknown"),
        source.telegram_source_kind
    )))
}
```

- [ ] **Step 8: Update runtime tests and fixtures**

In Rust tests that construct `SourceSyncTarget` or `SourceRecordRow`, delete
`telegram_source_kind` initializers. In test SQL that creates a current
`sources` table, remove `telegram_source_kind`; keep it only in old-schema
tests whose name says `legacy`, `v18`, `pre_v19`, or `upgrade`.

- [ ] **Step 9: Run backend source tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::store sources::peer_resolution sources::sync sources::topics
```

Expected:

```text
test result: ok. ... 0 failed
```

- [ ] **Step 10: Commit**

Run:

```powershell
git add src-tauri/src/sources/types.rs src-tauri/src/sources/store.rs src-tauri/src/sources/peer_resolution.rs src-tauri/src/sources/sync.rs src-tauri/src/sources/topics.rs src-tauri/src/sources/test_support.rs
git commit -m "feat: remove legacy source kind from backend DTOs"
```

Expected:

```text
[feature/source-identity-legacy-cleanup ...] feat: remove legacy source kind from backend DTOs
```

---

### Task 7: Remove Runtime Legacy References Outside Source Store

**Files:**
- Modify: `src-tauri/src/takeout_import/mod.rs`
- Modify: `src-tauri/src/takeout_import/pagination.rs`
- Modify: `src-tauri/src/takeout_import/export_dc.rs`
- Modify: `src-tauri/src/notebooklm_export/model.rs`
- Modify: `src-tauri/src/notebooklm_export/query.rs`
- Modify: `src-tauri/src/notebooklm_export/renderer.rs`
- Modify: `src-tauri/src/notebooklm_export/chunker.rs`
- Modify: `src-tauri/src/youtube/detail.rs`
- Modify: `src-tauri/src/youtube/jobs.rs`
- Modify: `src-tauri/src/analysis/fixtures.rs`

- [ ] **Step 1: Run containment scan and record current runtime matches**

Run:

```powershell
rg -n "telegram_source_kind|expected_kind" src-tauri\src -g '!target'
```

Expected before this task: matches remain in runtime modules and tests.

- [ ] **Step 2: Update Takeout naming without changing behavior**

In `src-tauri/src/takeout_import/mod.rs`, rename function parameters and fields
that represent canonical source subtype:

```rust
pub(crate) struct TakeoutSourceImportJob {
    pub(crate) telegram_source_subtype: String,
}
```

Rename helpers:

```rust
fn ensure_supported_takeout_source_subtype(source_subtype: &str) -> AppResult<()> {
    TelegramSourceKind::from_source_subtype(source_subtype).map(|_| ())
}

fn takeout_init_request_for_source_subtype(source_subtype: &str) -> AppResult<TakeoutInitRequest> {
    match source_subtype {
        TELEGRAM_KIND_CHANNEL => Ok(TakeoutInitRequest::Channel),
        TELEGRAM_KIND_SUPERGROUP => Ok(TakeoutInitRequest::Megagroup),
        TELEGRAM_KIND_GROUP => Ok(TakeoutInitRequest::Chat),
        other => Err(AppError::validation(format!(
            "Unsupported Telegram source_subtype '{other}'"
        ))),
    }
}
```

Update all call sites in `mod.rs`, `pagination.rs`, and `export_dc.rs` to use
`telegram_source_subtype` or `source_subtype` names. Keep string values
unchanged.

- [ ] **Step 3: Update NotebookLM source model**

In `src-tauri/src/notebooklm_export/model.rs`, change:

```rust
pub(crate) source_subtype: String,
```

and remove `telegram_source_kind`. In `query.rs`, return `source_subtype`
directly:

```rust
Ok(NotebookLmSource {
    id: source.id,
    source_type: source.source_type,
    source_subtype: source_subtype.as_str().to_string(),
    external_id: source.external_id,
    title: source.title,
})
```

In `renderer.rs` and `chunker.rs`, replace
`context.source.telegram_source_kind` with `context.source.source_subtype`.

- [ ] **Step 4: Update YouTube and analysis fixtures**

In `src-tauri/src/youtube/detail.rs`, remove `telegram_source_kind` from
`SourceSyncTarget` fixture rows and test table definitions. In
`src-tauri/src/analysis/fixtures.rs`, remove `telegram_source_kind` from
current fixture inserts. If a fixture is intentionally old-schema, rename the
helper or test to include `legacy_schema` and keep it quarantined.

- [ ] **Step 5: Run targeted runtime tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml takeout_import notebooklm_export youtube::detail youtube::jobs analysis
```

Expected:

```text
test result: ok. ... 0 failed
```

- [ ] **Step 6: Run containment scan for Rust normal code**

Run:

```powershell
rg -n "telegram_source_kind|expected_kind" src-tauri\src -g '!target'
```

Allowed matches after this task:

```text
src-tauri\src\migrations.rs: historical migration 11/15/18 tests
src-tauri\src\migrations\source_identity_cleanup.rs: old-schema migration tests only
```

If matches remain in `sources/store.rs`, `sources/peer_resolution.rs`,
`takeout_import`, `notebooklm_export`, `youtube`, or normal analysis fixtures,
finish removing them before committing.

- [ ] **Step 7: Commit**

Run:

```powershell
git add src-tauri/src/takeout_import src-tauri/src/notebooklm_export src-tauri/src/youtube src-tauri/src/analysis/fixtures.rs
git commit -m "refactor: use source subtype in runtime modules"
```

Expected:

```text
[feature/source-identity-legacy-cleanup ...] refactor: use source subtype in runtime modules
```

---

### Task 8: Update Frontend API, Types, And Source Dialog

**Files:**
- Modify: `src/lib/types/sources.ts`
- Modify: `src/lib/api/sources.ts`
- Modify: `src/lib/api/sources.test.ts`
- Modify: `src/lib/source-capabilities.test.ts`
- Modify: `src/lib/analysis-scope-state.test.ts`
- Modify: `src/lib/analysis-source-state.test.ts`
- Modify: `src/lib/analysis-state.test.ts`
- Modify: `src/lib/analysis-workspace-persistence.test.ts`
- Modify: `src/lib/analysis-workspace-workflow.test.ts`
- Modify: `src/lib/components/analysis/source-management-dialog.svelte`

- [ ] **Step 1: Add failing API tests for new wire contract**

In `src/lib/api/sources.test.ts`, update the first source mapping test so raw
source rows omit `telegram_source_kind` and expected mapped sources omit
`telegramSourceKind`.

Add:

```ts
it("adds telegram sources with expectedSubtype", async () => {
  invokeMock.mockResolvedValueOnce({
    id: 8,
    source_type: "telegram",
    source_subtype: "supergroup",
    account_id: 3,
    external_id: "456",
    title: "Forum",
    last_sync_state: null,
    last_synced_at: null,
    is_member: true,
    is_active: true,
    created_at: 1_600_001,
    avatar_data_url: null,
  });

  await expect(
    addTelegramSource({
      accountId: 3,
      sourceRef: "456",
      expectedSubtype: "supergroup",
    }),
  ).resolves.toMatchObject({
    id: 8,
    sourceSubtype: "supergroup",
    accountId: 3,
  });
  expect(invokeMock).toHaveBeenLastCalledWith("add_telegram_source", {
    request: { accountId: 3, sourceRef: "456", expectedSubtype: "supergroup" },
  });
});

it("maps live telegram dialogs with sourceSubtype", async () => {
  invokeMock.mockResolvedValueOnce([
    {
      id: 123,
      title: "Forum",
      username: "forum",
      source_subtype: "supergroup",
      is_member: true,
      photo_data_url: null,
    },
  ]);

  await expect(listTelegramSources(3)).resolves.toEqual([
    {
      id: 123,
      title: "Forum",
      username: "forum",
      sourceSubtype: "supergroup",
      isMember: true,
      photoDataUrl: null,
    },
  ]);
});
```

Delete the old test named
`does not derive persisted sourceSubtype from deprecated telegram_source_kind`.

- [ ] **Step 2: Run failing frontend API tests**

Run:

```powershell
npm.cmd test -- src/lib/api/sources.test.ts
```

Expected:

```text
FAIL src/lib/api/sources.test.ts
```

- [ ] **Step 3: Update frontend types**

In `src/lib/types/sources.ts`:

1. Change `TelegramDialogSource` to:

```ts
export interface TelegramDialogSource {
  id: number;
  title: string;
  username: string | null;
  sourceSubtype: TelegramSourceKind;
  isMember: boolean;
  photoDataUrl: string | null;
}
```

2. Remove `telegramSourceKind` from `Source`.
3. Change `AddTelegramSourceInput` to:

```ts
export interface AddTelegramSourceInput {
  accountId: number;
  sourceRef: string;
  expectedSubtype: TelegramSourceKind | null;
}
```

- [ ] **Step 4: Update frontend API mapping**

In `src/lib/api/sources.ts`:

1. Change `RawTelegramDialogSource` field to:

```ts
source_subtype: TelegramSourceKind;
```

2. Remove `telegram_source_kind` from `RawSource`.
3. Change add-source payload to:

```ts
request: {
  accountId: input.accountId,
  sourceRef: input.sourceRef,
  expectedSubtype: input.expectedSubtype,
},
```

4. Change dialog mapping to:

```ts
sourceSubtype: source.source_subtype,
```

5. Remove `telegramSourceKind` from `mapSource`.

- [ ] **Step 5: Update source management dialog**

In `src/lib/components/analysis/source-management-dialog.svelte`, replace live
dialog references:

```ts
source.telegramSourceKind
```

with:

```ts
source.sourceSubtype
```

Change the add-source function signature:

```ts
async function addSource(
  sourceRef: string,
  sourceSubtype: TelegramSourceKind | null,
  key: string,
) {
```

Send:

```ts
expectedSubtype: sourceSubtype,
```

Keep existing UI labels, filters, sort order, and keys using the same subtype
values.

- [ ] **Step 6: Update frontend fixtures and tests**

In every frontend test fixture under `src/lib/*.test.ts`, delete
`telegramSourceKind` from persisted `Source` objects. For Telegram sources,
keep `sourceSubtype` set to `"channel"`, `"supergroup"`, or `"group"`. For
YouTube sources, keep `sourceSubtype` as `"video"` or `"playlist"`. For tests
that asserted null legacy fields, delete those assertions.

- [ ] **Step 7: Run frontend tests**

Run:

```powershell
npm.cmd test
```

Expected:

```text
Test Files  ... passed
Tests       ... passed
```

- [ ] **Step 8: Run Svelte check**

Run:

```powershell
npm.cmd run check
```

Expected:

```text
svelte-check found 0 errors and 0 warnings
```

- [ ] **Step 9: Commit**

Run:

```powershell
git add src/lib/types/sources.ts src/lib/api/sources.ts src/lib/api/sources.test.ts src/lib/source-capabilities.test.ts src/lib/analysis-scope-state.test.ts src/lib/analysis-source-state.test.ts src/lib/analysis-state.test.ts src/lib/analysis-workspace-persistence.test.ts src/lib/analysis-workspace-workflow.test.ts src/lib/components/analysis/source-management-dialog.svelte
git commit -m "feat: remove legacy source kind from frontend API"
```

Expected:

```text
[feature/source-identity-legacy-cleanup ...] feat: remove legacy source kind from frontend API
```

---

### Task 9: Update Current-State Documentation

**Files:**
- Modify: `docs/database-schema.md`
- Modify: `docs/architecture-deep-dive.md`
- Modify: `docs/backlog.md`

- [ ] **Step 1: Update database schema docs**

In `docs/database-schema.md`:

1. Remove `telegram_source_kind` from the current `sources` field list.
2. Delete the current `telegram_source_kind values` section.
3. Replace compatibility-window notes with:

```markdown
- migration `19.sql` is runner-managed by Rust and removes the old
  `telegram_source_kind` compatibility mirror from the current `sources`
  schema;
- Telegram source subtype is canonical in `sources.source_subtype`;
- Telegram operational peer identity lives in `telegram_sources`.
```

4. Add migration history entry:

```markdown
| 19 | `19.sql` | Runner-managed rebuild of `sources` without `telegram_source_kind`; records the sentinel checksum for SQLx history |
```

- [ ] **Step 2: Update architecture docs**

In `docs/architecture-deep-dive.md`, replace the paragraph that says
`sources.telegram_source_kind` remains as a deprecated mirror with:

```markdown
Telegram source subtype is canonical in `sources.source_subtype`. Operational
Telegram peer identity lives in `telegram_sources`, including `peer_kind`,
`peer_id`, username/access-hash hints, and avatar cache keys. The former
`sources.telegram_source_kind` compatibility mirror was removed from the
current schema by the source identity legacy cleanup slice.
```

- [ ] **Step 3: Prune backlog shipped entries**

In `docs/backlog.md`, remove shipped cleanup bullets:

```markdown
- [ ] make `source_subtype` the canonical provider subtype and retire normal-path `telegram_source_kind` usage
- [ ] remove `telegram_source_kind` from DTOs and database writes after the compatibility window
- [ ] rebuild the fresh current schema without the legacy `telegram_source_kind` column
```

Keep open follow-ups only:

```markdown
- [ ] move remaining Telegram display/avatar metadata out of `sources.metadata_zstd`
- [ ] move YouTube identity/display metadata to typed source tables
- [ ] continue item/document identity cleanup
```

- [ ] **Step 4: Run docs scan**

Run:

```powershell
rg -n "telegram_source_kind|telegramSourceKind|expectedKind|expected_kind" docs
```

Allowed matches:

```text
docs\database-schema.md: migration history entries only
docs\superpowers\specs\2026-05-16-source-identity-legacy-cleanup-design.md: design history
docs\superpowers\plans\2026-05-16-source-identity-legacy-cleanup.md: implementation plan history
```

- [ ] **Step 5: Commit**

Run:

```powershell
git add docs/database-schema.md docs/architecture-deep-dive.md docs/backlog.md
git commit -m "docs: document source identity legacy cleanup"
```

Expected:

```text
[feature/source-identity-legacy-cleanup ...] docs: document source identity legacy cleanup
```

---

### Task 10: Final Verification And Containment

**Files:**
- No planned source edits. Fix only failures found by the commands in this task.

- [ ] **Step 1: Run full Rust tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected:

```text
test result: ok. ... 0 failed
```

- [ ] **Step 2: Run frontend tests**

Run:

```powershell
npm.cmd test
```

Expected:

```text
Test Files  ... passed
Tests       ... passed
```

- [ ] **Step 3: Run Svelte check**

Run:

```powershell
npm.cmd run check
```

Expected:

```text
svelte-check found 0 errors and 0 warnings
```

- [ ] **Step 4: Run whitespace check**

Run:

```powershell
git diff --check
```

Expected: no output.

- [ ] **Step 5: Run normal-code containment scan**

Run:

```powershell
rg -n "telegram_source_kind|telegramSourceKind|expectedKind|expected_kind" src-tauri\src src\lib src\routes
```

Allowed matches:

```text
src-tauri\src\migrations.rs: tests that mention historical migrations 11, 15, or 18
src-tauri\src\migrations\source_identity_cleanup.rs: old-schema upgrade fixtures/tests
```

Disallowed matches:

```text
src-tauri\src\sources\store.rs
src-tauri\src\sources\peer_resolution.rs
src-tauri\src\sources\identity_repair.rs
src-tauri\src\sources\sync.rs
src-tauri\src\sources\topics.rs
src-tauri\src\takeout_import
src-tauri\src\notebooklm_export
src-tauri\src\youtube
src\lib\api\sources.ts
src\lib\types\sources.ts
src\lib\components\analysis\source-management-dialog.svelte
src\routes
```

- [ ] **Step 6: Verify migration 19 does not run through plugin SQL**

Run:

```powershell
rg -n "extractum_runner_managed_migration_19|SOURCE_IDENTITY_CLEANUP_VERSION|apply_source_identity_cleanup_if_needed|PRAGMA foreign_keys = OFF|PRAGMA foreign_key_check" src-tauri\migrations\19.sql src-tauri\src\migrations.rs src-tauri\src\migrations\source_identity_cleanup.rs
```

Expected:

```text
src-tauri\migrations\19.sql: SELECT extractum_runner_managed_migration_19();
src-tauri\src\migrations.rs: apply_source_identity_cleanup_if_needed
src-tauri\src\migrations\source_identity_cleanup.rs: SOURCE_IDENTITY_CLEANUP_VERSION
src-tauri\src\migrations\source_identity_cleanup.rs: PRAGMA foreign_keys = OFF
src-tauri\src\migrations\source_identity_cleanup.rs: PRAGMA foreign_key_check
```

- [ ] **Step 7: Commit final fixes if any**

If the verification commands required fixes, run:

```powershell
git add src-tauri src docs
git commit -m "test: complete source identity cleanup verification"
```

Expected:

```text
[feature/source-identity-legacy-cleanup ...] test: complete source identity cleanup verification
```

If no files changed after verification, do not create a commit.

- [ ] **Step 8: Show final status**

Run:

```powershell
git status --short --branch
git --no-pager log -8 --oneline --decorate
```

Expected:

```text
## feature/source-identity-legacy-cleanup
recent commits from this plan are listed above
```

---

## Self-Review Checklist

- Migration 19 is runner-managed and not a plain tauri-plugin-sql SQL rebuild.
- The SQL plugin still receives versions 1..19, so SQLx does not report a
  missing applied migration after v19 is recorded.
- The v19 path disables foreign keys before `BEGIN`, performs the rebuild in a
  transaction, consumes `PRAGMA foreign_key_check` rows before commit, restores
  foreign keys, and checks again after commit.
- `sources.sqlite_sequence` is preserved.
- Post-v19 `sources` has no `telegram_source_kind`.
- Post-v19 Telegram and YouTube checks explicitly include
  `source_subtype IS NOT NULL`.
- The three final `sources` unique indexes are recreated without
  `IF NOT EXISTS`.
- Repair reads only `source_subtype`, `account_id`, `external_id`, and
  `metadata_zstd`; it never reads or writes `telegram_source_kind`.
- Persisted backend source DTOs and frontend `Source` have no
  `telegram_source_kind` / `telegramSourceKind`.
- Live Telegram dialog DTOs use `source_subtype` / `sourceSubtype`.
- Add Telegram source uses `expected_subtype` / `expectedSubtype`.
- Remaining old-name matches are confined to old migrations, old-schema tests,
  and docs that describe migration history.
