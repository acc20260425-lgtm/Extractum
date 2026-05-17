# Topic Membership Materialization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Materialize Telegram forum topic memberships so readers/export use indexed membership rows instead of recomputing topic inference joins.

**Architecture:** Add runner-managed migration 22 plus a shared `topic_memberships` runtime module. The module owns schema SQL, resolver version/state constants, set-based full rebuilds, scoped inserted-item resolution, invariant checks, and bounded error recording. Readers/export move to `item_topic_memberships`; topic refresh calls the canonical full rebuild before scoped sync/Takeout resolution is added as a freshness optimization.

**Tech Stack:** Rust, Tauri commands, SQLx SQLite, existing runner-managed migration pattern, Svelte/TypeScript API wrappers, Vitest, Cargo tests.

---

## File Structure

- Create `src-tauri/migrations/22.sql`
  - Sentinel migration so SQLx records version 22 while Rust owns data repair and rebuild.
- Create `src-tauri/src/migrations/topic_membership_materialization.rs`
  - Runner-managed migration 22 wrapper, migration-history checks, transaction handling, and migration-specific tests.
- Create `src-tauri/src/topic_memberships.rs`
  - Shared schema SQL, resolver constants, state DTO structs, full rebuild, scoped resolution, state checks, bounded error helper, and unit tests.
- Modify `src-tauri/src/migrations.rs`
  - Register migration 22 and run it after migration 21 in startup/test migration flow.
- Modify `src-tauri/src/lib.rs`
  - Register `mod topic_memberships;`.
- Modify `src-tauri/src/sources/test_support.rs`
  - Add helpers for topic membership/state tables in in-memory tests.
- Modify `src-tauri/src/sources/topics.rs`
  - Return topics response with state summary, count topics through memberships, return `Unrecognized` only when ready/current, and rebuild after successful topic refresh.
- Modify `src-tauri/src/sources/items/query.rs`
  - Use materialized membership joins and return empty rows for non-ready `Uncategorized` filter.
- Modify `src-tauri/src/sources/items.rs`
  - Run scoped topic resolution only for newly inserted Telegram item ids inside the insert transaction.
- Modify `src-tauri/src/notebooklm_export/query.rs`
  - Use materialized membership joins for export topic metadata.
- Modify `src-tauri/src/takeout_import/mod.rs`
  - Rely on the Telegram insert helper for scoped resolution; verify duplicate rows do not update counts.
- Modify `src-tauri/src/sources/types.rs`
  - Add topic resolution state row structs if shared by sources modules.
- Modify `src/lib/types/sources.ts`
  - Add topic resolution state summary and `SourceForumTopicsResult`.
- Modify `src/lib/api/sources.ts`
  - Map the new `list_source_forum_topics` response shape.
- Modify `src/routes/analysis/+page.svelte`
  - Unwrap `.topics` from `listSourceForumTopics`; no new visual state display in this slice.
- Modify `src/lib/api/sources.test.ts`
  - Verify response mapping and state summary.
- Modify `docs/database-schema.md`
  - Document `item_topic_memberships` and `telegram_topic_resolution_state`.
- Modify `docs/backlog.md`
  - Keep open-work-only schema simplification follow-ups accurate after implementation.
- Modify this plan as tasks complete.

## Task 0: Branch Guard And Baseline

**Files:**
- Read: `docs/superpowers/specs/2026-05-17-topic-membership-materialization-design.md`
- Verify: git status and focused baseline tests

- [x] **Step 1: Confirm clean starting point**

Run:

```powershell
git status --short --branch
git --no-pager log -8 --oneline --decorate
```

Expected:

```text
## main
5f9fdd2 (HEAD -> main) docs: finalize topic membership scoped test wording
```

If working tree contains user changes, inspect them and keep them intact.

- [x] **Step 2: Create an implementation branch or worktree**

Use `superpowers:using-git-worktrees` before execution. A safe branch name is:

```powershell
git switch -c feature/topic-membership-materialization
```

If using a linked worktree, use:

```powershell
git worktree add .worktrees/topic-membership-materialization -b feature/topic-membership-materialization
```

- [x] **Step 3: Run focused baseline tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::
cargo test --manifest-path src-tauri/Cargo.toml sources::topics::
cargo test --manifest-path src-tauri/Cargo.toml sources::items::query::
cargo test --manifest-path src-tauri/Cargo.toml notebooklm_export::query::
cargo test --manifest-path src-tauri/Cargo.toml source_ingest::
npm.cmd test -- --run src/lib/api/sources.test.ts src/lib/analysis-state.test.ts
```

Expected:

```text
test result: ok
```

and Vitest passes.

- [x] **Step 4: Confirm no baseline changes**

Run:

```powershell
git status --short --branch
```

Expected: only the branch header.

## Task 1: Migration 22 Sentinel And Shared Module Skeleton

**Files:**
- Create: `src-tauri/migrations/22.sql`
- Create: `src-tauri/src/migrations/topic_membership_materialization.rs`
- Create: `src-tauri/src/topic_memberships.rs`
- Modify: `src-tauri/src/migrations.rs`
- Modify: `src-tauri/src/lib.rs`

- [x] **Step 1: Add RED migration registration tests**

In `src-tauri/src/migrations.rs`, add module declaration near other migration modules:

```rust
pub(crate) mod topic_membership_materialization;
```

Inside `#[cfg(test)] mod tests`, add:

```rust
#[test]
fn includes_runner_managed_topic_membership_materialization_migration() {
    let migrations = build_migrations();
    let migration = migrations
        .iter()
        .find(|migration| migration.version == 22)
        .expect("version 22 migration is registered");

    assert_eq!(migration.description, "materialize telegram topic memberships");
    assert!(
        migration
            .sql
            .contains("extractum_runner_managed_migration_22"),
        "v22 must fail if plugin-managed SQL applies it directly"
    );
}

#[test]
fn plugin_migration_list_keeps_v22_as_sentinel_only() {
    let migration = build_migrations()
        .into_iter()
        .find(|migration| migration.version == 22)
        .expect("version 22 migration is registered");

    assert!(!migration.sql.contains("CREATE TABLE item_topic_memberships"));
    assert!(!migration.sql.contains("CREATE TABLE telegram_topic_resolution_state"));
    assert!(!migration.sql.contains("INSERT INTO item_topic_memberships"));
}
```

Update version-list assertion:

```rust
assert_eq!(versions, (1_i64..=22_i64).collect::<Vec<_>>());
```

- [x] **Step 2: Run registration tests and verify RED**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::tests::includes_runner_managed_topic_membership_materialization_migration
cargo test --manifest-path src-tauri/Cargo.toml migrations::tests::plugin_migration_list_keeps_v22_as_sentinel_only
cargo test --manifest-path src-tauri/Cargo.toml migrations::tests::build_migrations_contains_all_versions_for_sqlx_validation
```

Expected: fail because migration 22 is not registered yet.

- [x] **Step 3: Add migration 22 sentinel SQL**

Create `src-tauri/migrations/22.sql`:

```sql
-- Version 22 is applied by src-tauri/src/migrations/topic_membership_materialization.rs.
-- The Rust runner owns schema creation, source-level membership rebuilds,
-- state rows, invariant checks, and migration-history recording.
SELECT extractum_runner_managed_migration_22();
```

- [x] **Step 4: Add shared topic membership module skeleton**

Create `src-tauri/src/topic_memberships.rs`:

```rust
use sqlx::{Sqlite, SqliteConnection};

use crate::error::{AppError, AppResult};

pub(crate) const CURRENT_TOPIC_RESOLVER_VERSION: i64 = 1;
pub(crate) const TOPIC_STATE_NEVER_RUN: &str = "never_run";
pub(crate) const TOPIC_STATE_READY: &str = "ready";
pub(crate) const TOPIC_STATE_DIRTY: &str = "dirty";
pub(crate) const TOPIC_STATE_REBUILDING: &str = "rebuilding";
pub(crate) const TOPIC_STATE_FAILED: &str = "failed";
pub(crate) const TOPIC_LAST_ERROR_MAX_CHARS: usize = 1000;

#[derive(Clone, Debug, sqlx::FromRow)]
pub(crate) struct TopicResolutionStateRow {
    pub(crate) source_id: i64,
    pub(crate) resolver_version: i64,
    pub(crate) catalog_refreshed_at: Option<i64>,
    pub(crate) memberships_refreshed_at: Option<i64>,
    pub(crate) status: String,
    pub(crate) unresolved_count: i64,
    pub(crate) pending_item_count: i64,
    pub(crate) last_error: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TopicRebuildStats {
    pub(crate) eligible_items: i64,
    pub(crate) inserted_memberships: i64,
    pub(crate) unresolved_count: i64,
}

pub(crate) fn is_ready_current_state(state: Option<&TopicResolutionStateRow>) -> bool {
    matches!(
        state,
        Some(row)
            if row.status == TOPIC_STATE_READY
                && row.resolver_version == CURRENT_TOPIC_RESOLVER_VERSION
    )
}

pub(crate) fn truncate_topic_resolution_error(error: impl AsRef<str>) -> String {
    error
        .as_ref()
        .chars()
        .take(TOPIC_LAST_ERROR_MAX_CHARS)
        .collect()
}

pub(crate) async fn create_topic_membership_schema(
    conn: &mut SqliteConnection,
) -> AppResult<()> {
    sqlx::raw_sql(TOPIC_MEMBERSHIP_SCHEMA_SQL)
        .execute(&mut *conn)
        .await
        .map_err(AppError::database)?;
    Ok(())
}

pub(crate) async fn rebuild_topic_memberships_for_source_on_connection(
    conn: &mut SqliteConnection,
    source_id: i64,
    refreshed_at: i64,
    visible_rebuilding: bool,
) -> AppResult<TopicRebuildStats> {
    let _ = (conn, source_id, refreshed_at, visible_rebuilding);
    Err(AppError::internal("topic membership rebuild is not implemented"))
}

pub(crate) async fn resolve_scoped_topic_memberships_on_connection(
    conn: &mut SqliteConnection,
    source_id: i64,
    inserted_item_ids: &[i64],
    resolved_at: i64,
) -> AppResult<()> {
    let _ = (conn, source_id, inserted_item_ids, resolved_at);
    Ok(())
}

pub(crate) async fn load_topic_resolution_state(
    pool: &sqlx::Pool<Sqlite>,
    source_id: i64,
) -> AppResult<Option<TopicResolutionStateRow>> {
    sqlx::query_as(
        r#"
        SELECT
            source_id,
            resolver_version,
            catalog_refreshed_at,
            memberships_refreshed_at,
            status,
            unresolved_count,
            pending_item_count,
            last_error
        FROM telegram_topic_resolution_state
        WHERE source_id = ?
        "#,
    )
    .bind(source_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)
}

pub(crate) const TOPIC_MEMBERSHIP_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS item_topic_memberships (
    item_id INTEGER PRIMARY KEY REFERENCES items(id) ON DELETE CASCADE,
    source_id INTEGER NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
    topic_id INTEGER NOT NULL,
    match_kind TEXT NOT NULL,
    resolver_version INTEGER NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    FOREIGN KEY (source_id, topic_id)
        REFERENCES telegram_forum_topics(source_id, topic_id)
        ON DELETE CASCADE,
    CHECK (match_kind IN (
        'reply_to_top_id',
        'typed_root_top_message_id',
        'legacy_root_external_id',
        'reply_to_msg_id',
        'general_fallback'
    )),
    CHECK (resolver_version > 0)
);

CREATE INDEX IF NOT EXISTS idx_item_topic_memberships_source_topic
    ON item_topic_memberships(source_id, topic_id);

CREATE INDEX IF NOT EXISTS idx_item_topic_memberships_source_item
    ON item_topic_memberships(source_id, item_id);

CREATE TABLE IF NOT EXISTS telegram_topic_resolution_state (
    source_id INTEGER PRIMARY KEY REFERENCES sources(id) ON DELETE CASCADE,
    resolver_version INTEGER NOT NULL,
    catalog_refreshed_at INTEGER,
    memberships_refreshed_at INTEGER,
    status TEXT NOT NULL,
    unresolved_count INTEGER NOT NULL DEFAULT 0,
    pending_item_count INTEGER NOT NULL DEFAULT 0,
    last_error TEXT,
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    CHECK (resolver_version > 0),
    CHECK (status IN ('never_run', 'ready', 'dirty', 'rebuilding', 'failed')),
    CHECK (unresolved_count >= 0),
    CHECK (pending_item_count >= 0)
);
"#;
```

In `src-tauri/src/lib.rs`, add:

```rust
mod topic_memberships;
```

- [x] **Step 5: Add runner-managed migration skeleton**

Create `src-tauri/src/migrations/topic_membership_materialization.rs`:

```rust
use std::time::Instant;

use sha2::{Digest, Sha384};
use sqlx::{Connection, SqliteConnection};

use crate::error::{AppError, AppResult};
use crate::topic_memberships::create_topic_membership_schema;

pub(super) const TOPIC_MEMBERSHIP_MATERIALIZATION_VERSION: i64 = 22;
pub(super) const TOPIC_MEMBERSHIP_MATERIALIZATION_DESCRIPTION: &str =
    "materialize telegram topic memberships";
pub(super) const TOPIC_MEMBERSHIP_MATERIALIZATION_SENTINEL_SQL: &str =
    include_str!("../../migrations/22.sql");

pub(super) async fn apply_topic_membership_materialization_if_needed(
    db_url: &str,
) -> AppResult<()> {
    let mut conn = SqliteConnection::connect(db_url)
        .await
        .map_err(AppError::database)?;
    apply_topic_membership_materialization_on_connection(&mut conn).await
}

pub(super) async fn apply_topic_membership_materialization_on_connection(
    conn: &mut SqliteConnection,
) -> AppResult<()> {
    ensure_previous_migration_recorded(conn).await?;
    if migration_22_recorded(conn).await? {
        return Ok(());
    }

    let started_at = Instant::now();
    sqlx::query("BEGIN IMMEDIATE")
        .execute(&mut *conn)
        .await
        .map_err(AppError::database)?;

    let result = async {
        create_topic_membership_schema(conn).await?;
        Ok::<(), AppError>(())
    }
    .await;

    match result {
        Ok(()) => {
            sqlx::query("COMMIT")
                .execute(&mut *conn)
                .await
                .map_err(AppError::database)?;
        }
        Err(error) => {
            let _ = sqlx::query("ROLLBACK").execute(&mut *conn).await;
            return Err(error);
        }
    }

    record_migration_success(
        conn,
        expected_migration_22_checksum(),
        started_at.elapsed().as_nanos().min(i64::MAX as u128) as i64,
    )
    .await
}

async fn ensure_previous_migration_recorded(conn: &mut SqliteConnection) -> AppResult<()> {
    let exists: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM _sqlx_migrations WHERE version = 21 AND success = 1",
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(AppError::database)?;
    if exists == 0 {
        return Err(AppError::validation(
            "Topic membership materialization migration 22 requires migration 21",
        ));
    }
    Ok(())
}

async fn migration_22_recorded(conn: &mut SqliteConnection) -> AppResult<bool> {
    let checksum = expected_migration_22_checksum();
    let row: Option<(Vec<u8>, bool)> =
        sqlx::query_as("SELECT checksum, success FROM _sqlx_migrations WHERE version = ?")
            .bind(TOPIC_MEMBERSHIP_MATERIALIZATION_VERSION)
            .fetch_optional(&mut *conn)
            .await
            .map_err(AppError::database)?;

    match row {
        None => Ok(false),
        Some((applied_checksum, true)) if applied_checksum == checksum => Ok(true),
        Some((_applied_checksum, true)) => Err(AppError::internal(
            "Migration 22 checksum does not match the runner-managed topic membership sentinel",
        )),
        Some((_applied_checksum, false)) => Err(AppError::internal(
            "Migration 22 is marked as failed in _sqlx_migrations",
        )),
    }
}

async fn record_migration_success(
    conn: &mut SqliteConnection,
    checksum: Vec<u8>,
    execution_time: i64,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time) VALUES (?, ?, 1, ?, ?)",
    )
    .bind(TOPIC_MEMBERSHIP_MATERIALIZATION_VERSION)
    .bind(TOPIC_MEMBERSHIP_MATERIALIZATION_DESCRIPTION)
    .bind(checksum)
    .bind(execution_time)
    .execute(&mut *conn)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

fn expected_migration_22_checksum() -> Vec<u8> {
    Sha384::digest(TOPIC_MEMBERSHIP_MATERIALIZATION_SENTINEL_SQL.as_bytes()).to_vec()
}
```

- [x] **Step 6: Register migration 22**

In `build_migrations`, append after v21:

```rust
Migration {
    version: 22,
    description: "materialize telegram topic memberships",
    sql: include_str!("../migrations/22.sql"),
    kind: MigrationKind::Up,
},
```

In `patch_migrations`, replace the final call chain with:

```rust
telegram_item_native_identity::apply_telegram_item_native_identity_if_needed(&url).await?;
topic_membership_materialization::apply_topic_membership_materialization_if_needed(&url).await
```

In `apply_all_migrations_for_test_pool`, replace the final call with:

```rust
telegram_item_native_identity::apply_telegram_item_native_identity_on_connection(conn).await?;
topic_membership_materialization::apply_topic_membership_materialization_on_connection(conn).await
```

- [x] **Step 7: Run registration tests and verify GREEN**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::tests::includes_runner_managed_topic_membership_materialization_migration
cargo test --manifest-path src-tauri/Cargo.toml migrations::tests::plugin_migration_list_keeps_v22_as_sentinel_only
cargo test --manifest-path src-tauri/Cargo.toml migrations::tests::build_migrations_contains_all_versions_for_sqlx_validation
```

Expected:

```text
test result: ok
```

- [x] **Step 8: Commit**

```powershell
git add src-tauri/migrations/22.sql src-tauri/src/lib.rs src-tauri/src/migrations.rs src-tauri/src/migrations/topic_membership_materialization.rs src-tauri/src/topic_memberships.rs
git commit -m "feat: add topic membership migration sentinel"
```

## Task 2: Schema Fixtures, State Helpers, And Migration Schema Tests

**Files:**
- Modify: `src-tauri/src/topic_memberships.rs`
- Modify: `src-tauri/src/sources/test_support.rs`
- Modify: `src-tauri/src/migrations/topic_membership_materialization.rs`

- [x] **Step 1: Add RED schema tests**

In `src-tauri/src/migrations/topic_membership_materialization.rs`, add tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::migrations::build_migrations;
    use sha2::{Digest, Sha384};
    use sqlx::SqliteConnection;

    #[tokio::test]
    async fn migration_22_creates_membership_and_state_schema() {
        let mut conn = memory_conn_with_history_through_21().await;

        apply_topic_membership_materialization_on_connection(&mut conn)
            .await
            .expect("apply v22");

        for table in ["item_topic_memberships", "telegram_topic_resolution_state"] {
            let exists: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?",
            )
            .bind(table)
            .fetch_one(&mut conn)
            .await
            .expect("check table");
            assert_eq!(exists, 1, "missing table {table}");
        }

        for index in [
            "idx_item_topic_memberships_source_topic",
            "idx_item_topic_memberships_source_item",
        ] {
            let exists: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = ?",
            )
            .bind(index)
            .fetch_one(&mut conn)
            .await
            .expect("check index");
            assert_eq!(exists, 1, "missing index {index}");
        }
    }

    #[tokio::test]
    async fn migration_22_records_sentinel_checksum_and_is_idempotent() {
        let mut conn = memory_conn_with_history_through_21().await;

        apply_topic_membership_materialization_on_connection(&mut conn)
            .await
            .expect("first v22");
        apply_topic_membership_materialization_on_connection(&mut conn)
            .await
            .expect("second v22");

        let row: (String, bool, Vec<u8>) = sqlx::query_as(
            "SELECT description, success, checksum FROM _sqlx_migrations WHERE version = 22",
        )
        .fetch_one(&mut conn)
        .await
        .expect("load v22 history");
        assert_eq!(row.0, TOPIC_MEMBERSHIP_MATERIALIZATION_DESCRIPTION);
        assert!(row.1);
        assert_eq!(row.2, expected_migration_22_checksum());
    }

    async fn memory_conn_with_history_through_21() -> SqliteConnection {
        let mut conn = SqliteConnection::connect("sqlite::memory:")
            .await
            .expect("connect memory sqlite");

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS _sqlx_migrations (
                version BIGINT PRIMARY KEY,
                description TEXT NOT NULL,
                installed_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                success BOOLEAN NOT NULL,
                checksum BLOB NOT NULL,
                execution_time BIGINT NOT NULL
            )",
        )
        .execute(&mut conn)
        .await
        .expect("create migration history");

        for migration in build_migrations()
            .into_iter()
            .filter(|migration| migration.version < 19)
        {
            sqlx::raw_sql(migration.sql)
                .execute(&mut conn)
                .await
                .unwrap_or_else(|error| panic!("apply migration {}: {error}", migration.version));
            sqlx::query("INSERT INTO _sqlx_migrations (version, description, success, checksum, execution_time) VALUES (?, ?, 1, ?, 0)")
                .bind(migration.version)
                .bind(migration.description)
                .bind(Sha384::digest(migration.sql.as_bytes()).to_vec())
                .execute(&mut conn)
                .await
                .expect("record standard migration");
        }

        crate::migrations::source_identity_cleanup::apply_source_identity_cleanup_on_connection(
            &mut conn,
        )
        .await
        .expect("apply v19");
        crate::migrations::youtube_typed_source_metadata::apply_youtube_typed_source_metadata_on_connection(
            &mut conn,
        )
        .await
        .expect("apply v20");
        crate::migrations::telegram_item_native_identity::apply_telegram_item_native_identity_on_connection(
            &mut conn,
        )
        .await
        .expect("apply v21");

        conn
    }
}
```

- [x] **Step 2: Run schema tests and verify RED**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::topic_membership_materialization::tests::migration_22_creates_membership_and_state_schema
cargo test --manifest-path src-tauri/Cargo.toml migrations::topic_membership_materialization::tests::migration_22_records_sentinel_checksum_and_is_idempotent
cargo test --manifest-path src-tauri/Cargo.toml sources::test_support::tests::source_fixture_creates_expected_tables
```

Expected: migration schema tests may pass if schema creation was wired in Task 1; fixture test fails until `memory_pool_with_source_items_and_topics` creates the new tables.

- [x] **Step 3: Add test fixture helper for membership tables**

In `src-tauri/src/sources/test_support.rs`, add:

```rust
pub(crate) async fn create_topic_membership_tables(pool: &sqlx::SqlitePool) {
    let mut conn = pool.acquire().await.expect("acquire sqlite connection");
    crate::topic_memberships::create_topic_membership_schema(&mut conn)
        .await
        .expect("create topic membership schema");
}
```

In `memory_pool_with_source_items_and_topics`, after creating `telegram_forum_topics`, call:

```rust
create_topic_membership_tables(&pool).await;
```

In `source_fixture_creates_expected_tables`, include:

```rust
"item_topic_memberships",
"telegram_topic_resolution_state",
```

- [x] **Step 4: Run schema tests and fixture test**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::topic_membership_materialization::tests::migration_22_creates_membership_and_state_schema
cargo test --manifest-path src-tauri/Cargo.toml migrations::topic_membership_materialization::tests::migration_22_records_sentinel_checksum_and_is_idempotent
cargo test --manifest-path src-tauri/Cargo.toml sources::test_support::tests::source_fixture_creates_expected_tables
```

Expected:

```text
test result: ok
```

- [x] **Step 5: Commit**

```powershell
git add src-tauri/src/migrations/topic_membership_materialization.rs src-tauri/src/sources/test_support.rs src-tauri/src/topic_memberships.rs
git commit -m "feat: add topic membership schema"
```

## Task 3: Shared Resolver And Full Source Rebuild

**Files:**
- Modify: `src-tauri/src/topic_memberships.rs`

- [x] **Step 1: Add RED resolver tests**

Inside `#[cfg(test)] mod tests` in `src-tauri/src/topic_memberships.rs`, add tests named:

```rust
#[tokio::test]
async fn rebuild_prioritizes_specific_topic_matches_before_general_fallback() {
    let pool = resolver_pool().await;
    seed_supergroup_source(&pool, 1).await;
    seed_topic(&pool, 1, 10, 1000, "Specific", false, false).await;
    seed_topic(&pool, 1, 1, 1, "General", false, false).await;
    seed_item(&pool, 101, 1, "999", Some(10), None).await;
    seed_item(&pool, 102, 1, "1000", None, None).await;
    seed_typed_message(&pool, 102, 1, 1000).await;
    seed_item(&pool, 103, 1, "1001", None, Some(10)).await;
    seed_item(&pool, 104, 1, "1002", None, None).await;

    let mut conn = pool.acquire().await.expect("acquire");
    let stats = rebuild_topic_memberships_for_source_on_connection(&mut conn, 1, 1234, false)
        .await
        .expect("rebuild");

    assert_eq!(stats.eligible_items, 4);
    assert_eq!(stats.inserted_memberships, 4);
    assert_eq!(stats.unresolved_count, 0);

    let rows: Vec<(i64, i64, String)> = sqlx::query_as(
        "SELECT item_id, topic_id, match_kind FROM item_topic_memberships ORDER BY item_id",
    )
    .fetch_all(&pool)
    .await
    .expect("load memberships");
    assert_eq!(
        rows,
        vec![
            (101, 10, "reply_to_top_id".to_string()),
            (102, 10, "typed_root_top_message_id".to_string()),
            (103, 10, "reply_to_msg_id".to_string()),
            (104, 1, "general_fallback".to_string()),
        ]
    );
}

#[tokio::test]
async fn rebuild_uses_legacy_root_only_without_typed_child() {
    let pool = resolver_pool().await;
    seed_supergroup_source(&pool, 1).await;
    seed_topic(&pool, 1, 20, 700, "Root", false, false).await;
    seed_item(&pool, 201, 1, "700", None, None).await;
    seed_item(&pool, 202, 1, "700", None, None).await;
    seed_typed_message(&pool, 202, 1, 701).await;

    let mut conn = pool.acquire().await.expect("acquire");
    let stats = rebuild_topic_memberships_for_source_on_connection(&mut conn, 1, 1234, false)
        .await
        .expect("rebuild");

    assert_eq!(stats.eligible_items, 2);
    assert_eq!(stats.inserted_memberships, 1);
    assert_eq!(stats.unresolved_count, 1);

    let rows: Vec<(i64, i64, String)> = sqlx::query_as(
        "SELECT item_id, topic_id, match_kind FROM item_topic_memberships ORDER BY item_id",
    )
    .fetch_all(&pool)
    .await
    .expect("load memberships");
    assert_eq!(rows, vec![(201, 20, "legacy_root_external_id".to_string())]);
}

#[tokio::test]
async fn rebuild_matches_retained_hidden_and_deleted_topics() {
    let pool = resolver_pool().await;
    seed_supergroup_source(&pool, 1).await;
    seed_topic(&pool, 1, 30, 300, "Hidden", true, false).await;
    seed_topic(&pool, 1, 40, 400, "Deleted", false, true).await;
    seed_item(&pool, 301, 1, "301", Some(30), None).await;
    seed_item(&pool, 401, 1, "401", Some(40), None).await;

    let mut conn = pool.acquire().await.expect("acquire");
    rebuild_topic_memberships_for_source_on_connection(&mut conn, 1, 1234, false)
        .await
        .expect("rebuild");

    let topics: Vec<i64> = sqlx::query_scalar(
        "SELECT topic_id FROM item_topic_memberships ORDER BY item_id",
    )
    .fetch_all(&pool)
    .await
    .expect("load topics");
    assert_eq!(topics, vec![30, 40]);
}

#[tokio::test]
async fn rebuild_replaces_stale_memberships_and_versions() {
    let pool = resolver_pool().await;
    seed_supergroup_source(&pool, 1).await;
    seed_topic(&pool, 1, 50, 500, "Fresh", false, false).await;
    seed_item(&pool, 501, 1, "501", Some(50), None).await;
    sqlx::query(
        "INSERT INTO item_topic_memberships (item_id, source_id, topic_id, match_kind, resolver_version)
         VALUES (501, 1, 50, 'reply_to_top_id', 999)",
    )
    .execute(&pool)
    .await
    .expect("insert stale membership");

    let mut conn = pool.acquire().await.expect("acquire");
    rebuild_topic_memberships_for_source_on_connection(&mut conn, 1, 1234, false)
        .await
        .expect("rebuild");

    let version: i64 = sqlx::query_scalar(
        "SELECT resolver_version FROM item_topic_memberships WHERE item_id = 501",
    )
    .fetch_one(&pool)
    .await
    .expect("load resolver version");
    assert_eq!(version, CURRENT_TOPIC_RESOLVER_VERSION);
}
```

Add compact test helpers in the same module:

```rust
async fn resolver_pool() -> sqlx::SqlitePool {
    crate::sources::test_support::memory_pool_with_source_items_and_topics().await
}

async fn seed_supergroup_source(pool: &sqlx::SqlitePool, source_id: i64) {
    sqlx::query(
        "INSERT OR IGNORE INTO sources (id, source_type, source_subtype, external_id, title, is_active, is_member, created_at)
         VALUES (?, 'telegram', 'supergroup', ?, 'Forum', 1, 1, 1)",
    )
    .bind(source_id)
    .bind(source_id.to_string())
    .execute(pool)
    .await
    .expect("seed supergroup source");
}

async fn seed_topic(
    pool: &sqlx::SqlitePool,
    source_id: i64,
    topic_id: i64,
    top_message_id: i64,
    title: &str,
    hidden: bool,
    deleted: bool,
) {
    sqlx::query(
        "INSERT INTO telegram_forum_topics (
            source_id, topic_id, top_message_id, title, is_closed, is_pinned,
            is_hidden, is_deleted, sort_order, last_seen_at, updated_at
         ) VALUES (?, ?, ?, ?, 0, 0, ?, ?, NULL, 100, 100)",
    )
    .bind(source_id)
    .bind(topic_id)
    .bind(top_message_id)
    .bind(title)
    .bind(i64::from(hidden))
    .bind(i64::from(deleted))
    .execute(pool)
    .await
    .expect("seed topic");
}

async fn seed_item(
    pool: &sqlx::SqlitePool,
    item_id: i64,
    source_id: i64,
    external_id: &str,
    reply_to_top_id: Option<i64>,
    reply_to_msg_id: Option<i64>,
) {
    sqlx::query(
        "INSERT INTO items (
            id, source_id, external_id, item_kind, author, published_at,
            ingested_at, content_kind, has_media, reply_to_top_id, reply_to_msg_id
         ) VALUES (?, ?, ?, 'telegram_message', 'alice', ?, ?, 'text_only', 0, ?, ?)",
    )
    .bind(item_id)
    .bind(source_id)
    .bind(external_id)
    .bind(item_id)
    .bind(item_id)
    .bind(reply_to_top_id)
    .bind(reply_to_msg_id)
    .execute(pool)
    .await
    .expect("seed item");
}

async fn seed_typed_message(
    pool: &sqlx::SqlitePool,
    item_id: i64,
    source_id: i64,
    telegram_message_id: i64,
) {
    sqlx::query(
        "INSERT INTO telegram_messages (item_id, source_id, history_peer_kind, history_peer_id, telegram_message_id)
         VALUES (?, ?, 'channel', 12345, ?)",
    )
    .bind(item_id)
    .bind(source_id)
    .bind(telegram_message_id)
    .execute(pool)
    .await
    .expect("seed typed message");
}
```

- [x] **Step 2: Run resolver tests and verify RED**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml topic_memberships::tests::rebuild_
```

Expected: fail because rebuild returns an internal error.

- [x] **Step 3: Implement set-based resolver SQL**

In `src-tauri/src/topic_memberships.rs`, add SQL constants and helper functions:

```rust
const RESOLVED_MEMBERSHIP_SELECT_SQL: &str = r#"
WITH eligible AS (
    SELECT
        items.id AS item_id,
        items.source_id,
        items.external_id,
        items.reply_to_top_id,
        items.reply_to_msg_id,
        telegram_messages.item_id AS typed_item_id,
        telegram_messages.telegram_message_id
    FROM items
    JOIN sources ON sources.id = items.source_id
    LEFT JOIN telegram_messages ON telegram_messages.item_id = items.id
    WHERE items.source_id = ?
      AND sources.source_type = 'telegram'
      AND sources.source_subtype = 'supergroup'
      AND items.item_kind = 'telegram_message'
),
candidates AS (
    SELECT e.item_id, e.source_id, t.topic_id, 'reply_to_top_id' AS match_kind, 1 AS priority
    FROM eligible e
    JOIN telegram_forum_topics t
      ON t.source_id = e.source_id
     AND e.reply_to_top_id = t.topic_id

    UNION ALL

    SELECT e.item_id, e.source_id, t.topic_id, 'typed_root_top_message_id' AS match_kind, 2 AS priority
    FROM eligible e
    JOIN telegram_forum_topics t
      ON t.source_id = e.source_id
     AND e.reply_to_top_id IS NULL
     AND e.telegram_message_id = t.top_message_id

    UNION ALL

    SELECT e.item_id, e.source_id, t.topic_id, 'legacy_root_external_id' AS match_kind, 3 AS priority
    FROM eligible e
    JOIN telegram_forum_topics t
      ON t.source_id = e.source_id
     AND e.reply_to_top_id IS NULL
     AND e.typed_item_id IS NULL
     AND e.external_id <> ''
     AND e.external_id NOT GLOB '*[^0-9]*'
     AND CAST(e.external_id AS INTEGER) = t.top_message_id

    UNION ALL

    SELECT e.item_id, e.source_id, t.topic_id, 'reply_to_msg_id' AS match_kind, 4 AS priority
    FROM eligible e
    JOIN telegram_forum_topics t
      ON t.source_id = e.source_id
     AND e.reply_to_top_id IS NULL
     AND e.reply_to_msg_id = t.topic_id

    UNION ALL

    SELECT e.item_id, e.source_id, t.topic_id, 'general_fallback' AS match_kind, 5 AS priority
    FROM eligible e
    JOIN telegram_forum_topics t
      ON t.source_id = e.source_id
     AND e.reply_to_top_id IS NULL
     AND t.topic_id = 1
),
ranked AS (
    SELECT
        item_id,
        source_id,
        topic_id,
        match_kind,
        ROW_NUMBER() OVER (PARTITION BY item_id ORDER BY priority ASC, topic_id ASC) AS rn
    FROM candidates
)
SELECT item_id, source_id, topic_id, match_kind
FROM ranked
WHERE rn = 1
"#;

async fn eligible_item_count(conn: &mut SqliteConnection, source_id: i64) -> AppResult<i64> {
    sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM items
         JOIN sources ON sources.id = items.source_id
         WHERE items.source_id = ?
           AND sources.source_type = 'telegram'
           AND sources.source_subtype = 'supergroup'
           AND items.item_kind = 'telegram_message'",
    )
    .bind(source_id)
    .fetch_one(&mut *conn)
    .await
    .map_err(AppError::database)
}

async fn inserted_membership_count(conn: &mut SqliteConnection, source_id: i64) -> AppResult<i64> {
    sqlx::query_scalar("SELECT COUNT(*) FROM item_topic_memberships WHERE source_id = ?")
        .bind(source_id)
        .fetch_one(&mut *conn)
        .await
        .map_err(AppError::database)
}
```

Replace `rebuild_topic_memberships_for_source_on_connection` with:

```rust
pub(crate) async fn rebuild_topic_memberships_for_source_on_connection(
    conn: &mut SqliteConnection,
    source_id: i64,
    refreshed_at: i64,
    visible_rebuilding: bool,
) -> AppResult<TopicRebuildStats> {
    if visible_rebuilding {
        upsert_resolution_state(
            conn,
            source_id,
            TOPIC_STATE_REBUILDING,
            None,
            None,
            0,
            0,
            None,
            refreshed_at,
        )
        .await?;
    }

    sqlx::query("DELETE FROM item_topic_memberships WHERE source_id = ?")
        .bind(source_id)
        .execute(&mut *conn)
        .await
        .map_err(AppError::database)?;

    let insert_sql = format!(
        "INSERT INTO item_topic_memberships (
             item_id, source_id, topic_id, match_kind, resolver_version, created_at, updated_at
         )
         SELECT item_id, source_id, topic_id, match_kind, ?, ?, ?
         FROM ({RESOLVED_MEMBERSHIP_SELECT_SQL})"
    );
    sqlx::query(&insert_sql)
        .bind(CURRENT_TOPIC_RESOLVER_VERSION)
        .bind(refreshed_at)
        .bind(refreshed_at)
        .bind(source_id)
        .execute(&mut *conn)
        .await
        .map_err(AppError::database)?;

    let eligible = eligible_item_count(conn, source_id).await?;
    let inserted = inserted_membership_count(conn, source_id).await?;
    let unresolved = (eligible - inserted).max(0);

    let catalog_refreshed_at = source_catalog_refreshed_at(conn, source_id).await?;
    assert_ready_source_invariants(conn, source_id, eligible, inserted, unresolved).await?;
    upsert_resolution_state(
        conn,
        source_id,
        TOPIC_STATE_READY,
        catalog_refreshed_at,
        Some(refreshed_at),
        unresolved,
        0,
        None,
        refreshed_at,
    )
    .await?;

    Ok(TopicRebuildStats {
        eligible_items: eligible,
        inserted_memberships: inserted,
        unresolved_count: unresolved,
    })
}
```

Add `upsert_resolution_state`, `source_catalog_refreshed_at`, and a temporary `assert_ready_source_invariants` that checks source-id mismatches and resolver versions:

```rust
async fn upsert_resolution_state(
    conn: &mut SqliteConnection,
    source_id: i64,
    status: &str,
    catalog_refreshed_at: Option<i64>,
    memberships_refreshed_at: Option<i64>,
    unresolved_count: i64,
    pending_item_count: i64,
    last_error: Option<&str>,
    updated_at: i64,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO telegram_topic_resolution_state (
            source_id, resolver_version, catalog_refreshed_at, memberships_refreshed_at,
            status, unresolved_count, pending_item_count, last_error, updated_at
         )
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(source_id) DO UPDATE SET
            resolver_version = excluded.resolver_version,
            catalog_refreshed_at = excluded.catalog_refreshed_at,
            memberships_refreshed_at = excluded.memberships_refreshed_at,
            status = excluded.status,
            unresolved_count = excluded.unresolved_count,
            pending_item_count = excluded.pending_item_count,
            last_error = excluded.last_error,
            updated_at = excluded.updated_at",
    )
    .bind(source_id)
    .bind(CURRENT_TOPIC_RESOLVER_VERSION)
    .bind(catalog_refreshed_at)
    .bind(memberships_refreshed_at)
    .bind(status)
    .bind(unresolved_count)
    .bind(pending_item_count)
    .bind(last_error.map(truncate_topic_resolution_error))
    .bind(updated_at)
    .execute(&mut *conn)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

async fn source_catalog_refreshed_at(
    conn: &mut SqliteConnection,
    source_id: i64,
) -> AppResult<Option<i64>> {
    sqlx::query_scalar(
        "SELECT MAX(COALESCE(updated_at, last_seen_at))
         FROM telegram_forum_topics
         WHERE source_id = ?",
    )
    .bind(source_id)
    .fetch_one(&mut *conn)
    .await
    .map_err(AppError::database)
}

async fn assert_ready_source_invariants(
    conn: &mut SqliteConnection,
    source_id: i64,
    eligible: i64,
    inserted: i64,
    unresolved: i64,
) -> AppResult<()> {
    if inserted + unresolved != eligible {
        return Err(AppError::validation(format!(
            "Topic membership rebuild for source {source_id} has inconsistent counts: inserted {inserted}, unresolved {unresolved}, eligible {eligible}"
        )));
    }

    let source_mismatch: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM item_topic_memberships m
         JOIN items i ON i.id = m.item_id
         WHERE m.source_id = ? AND m.source_id <> i.source_id",
    )
    .bind(source_id)
    .fetch_one(&mut *conn)
    .await
    .map_err(AppError::database)?;
    if source_mismatch != 0 {
        return Err(AppError::validation(format!(
            "Topic membership rebuild for source {source_id} produced {source_mismatch} source mismatches"
        )));
    }

    let version_mismatch: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM item_topic_memberships
         WHERE source_id = ? AND resolver_version <> ?",
    )
    .bind(source_id)
    .bind(CURRENT_TOPIC_RESOLVER_VERSION)
    .fetch_one(&mut *conn)
    .await
    .map_err(AppError::database)?;
    if version_mismatch != 0 {
        return Err(AppError::validation(format!(
            "Topic membership rebuild for source {source_id} produced {version_mismatch} stale resolver versions"
        )));
    }

    Ok(())
}
```

- [x] **Step 4: Run resolver tests and verify GREEN**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml topic_memberships::tests::rebuild_
```

Expected:

```text
test result: ok
```

- [x] **Step 5: Commit**

```powershell
git add src-tauri/src/topic_memberships.rs
git commit -m "feat: rebuild topic memberships with shared resolver"
```

## Task 4: Migration 22 Data Rebuild And Integrity Checks

**Files:**
- Modify: `src-tauri/src/migrations/topic_membership_materialization.rs`
- Modify: `src-tauri/src/topic_memberships.rs`

- [ ] **Step 1: Add RED migration data tests**

In `src-tauri/src/migrations/topic_membership_materialization.rs`, add:

```rust
#[tokio::test]
async fn migration_22_rebuilds_catalog_sources_and_creates_never_run_state() {
    let mut conn = memory_conn_with_history_through_21().await;
    seed_supergroup_source(&mut conn, 10, true).await;
    seed_supergroup_source(&mut conn, 20, false).await;
    seed_channel_source(&mut conn, 30).await;
    seed_topic(&mut conn, 10, 200, 700, "Roadmap").await;
    seed_item(&mut conn, 1001, 10, "701", Some(200), None).await;
    seed_item(&mut conn, 1002, 10, "999", Some(404), None).await;

    apply_topic_membership_materialization_on_connection(&mut conn)
        .await
        .expect("apply v22");

    let memberships: Vec<(i64, i64, String)> = sqlx::query_as(
        "SELECT item_id, topic_id, match_kind FROM item_topic_memberships ORDER BY item_id",
    )
    .fetch_all(&mut conn)
    .await
    .expect("load memberships");
    assert_eq!(memberships, vec![(1001, 200, "reply_to_top_id".to_string())]);

    let states: Vec<(i64, String, i64)> = sqlx::query_as(
        "SELECT source_id, status, unresolved_count FROM telegram_topic_resolution_state ORDER BY source_id",
    )
    .fetch_all(&mut conn)
    .await
    .expect("load states");
    assert_eq!(
        states,
        vec![
            (10, "ready".to_string(), 1),
            (20, "never_run".to_string(), 0),
        ]
    );
}

#[tokio::test]
async fn migration_22_rejects_state_rows_for_non_supergroups() {
    let mut conn = memory_conn_with_history_through_21().await;
    seed_channel_source(&mut conn, 30).await;
    crate::topic_memberships::create_topic_membership_schema(&mut conn)
        .await
        .expect("schema");
    sqlx::query(
        "INSERT INTO telegram_topic_resolution_state (
            source_id, resolver_version, status, unresolved_count, pending_item_count
         ) VALUES (30, 1, 'ready', 0, 0)",
    )
    .execute(&mut conn)
    .await
    .expect("dirty state row");

    let error = apply_topic_membership_materialization_on_connection(&mut conn)
        .await
        .expect_err("state invariant fails");

    assert_eq!(error.kind, crate::error::AppErrorKind::Validation);
    assert!(error.message.contains("telegram_topic_resolution_state"));
}
```

Add helper functions in the same test module:

```rust
async fn seed_supergroup_source(conn: &mut SqliteConnection, source_id: i64, with_identity: bool) {
    sqlx::query(
        "INSERT OR IGNORE INTO sources (
            id, source_type, source_subtype, external_id, title, is_active, is_member, created_at
         ) VALUES (?, 'telegram', 'supergroup', ?, 'Supergroup', 1, 1, 1)",
    )
    .bind(source_id)
    .bind(source_id.to_string())
    .execute(&mut *conn)
    .await
    .expect("seed supergroup");
    if with_identity {
        sqlx::query(
            "INSERT OR IGNORE INTO telegram_sources (
                source_id, account_id, source_subtype, peer_kind, peer_id, resolution_strategy
             ) VALUES (?, 1, 'supergroup', 'channel', ?, 'dialog')",
        )
        .bind(source_id)
        .bind(source_id)
        .execute(&mut *conn)
        .await
        .expect("seed telegram source identity");
    }
}

async fn seed_channel_source(conn: &mut SqliteConnection, source_id: i64) {
    sqlx::query(
        "INSERT OR IGNORE INTO sources (
            id, source_type, source_subtype, external_id, title, is_active, is_member, created_at
         ) VALUES (?, 'telegram', 'channel', ?, 'Channel', 1, 1, 1)",
    )
    .bind(source_id)
    .bind(source_id.to_string())
    .execute(&mut *conn)
    .await
    .expect("seed channel");
}

async fn seed_topic(conn: &mut SqliteConnection, source_id: i64, topic_id: i64, top_message_id: i64, title: &str) {
    sqlx::query(
        "INSERT INTO telegram_forum_topics (
            source_id, topic_id, top_message_id, title, last_seen_at, updated_at
         ) VALUES (?, ?, ?, ?, 100, 100)",
    )
    .bind(source_id)
    .bind(topic_id)
    .bind(top_message_id)
    .bind(title)
    .execute(&mut *conn)
    .await
    .expect("seed topic");
}

async fn seed_item(
    conn: &mut SqliteConnection,
    item_id: i64,
    source_id: i64,
    external_id: &str,
    reply_to_top_id: Option<i64>,
    reply_to_msg_id: Option<i64>,
) {
    sqlx::query(
        "INSERT INTO items (
            id, source_id, external_id, item_kind, author, published_at,
            ingested_at, content_kind, has_media, reply_to_top_id, reply_to_msg_id
         ) VALUES (?, ?, ?, 'telegram_message', 'alice', ?, ?, 'text_only', 0, ?, ?)",
    )
    .bind(item_id)
    .bind(source_id)
    .bind(external_id)
    .bind(item_id)
    .bind(item_id)
    .bind(reply_to_top_id)
    .bind(reply_to_msg_id)
    .execute(&mut *conn)
    .await
    .expect("seed item");
}
```

- [ ] **Step 2: Run migration data tests and verify RED**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::topic_membership_materialization::tests::migration_22_rebuilds_catalog_sources_and_creates_never_run_state migrations::topic_membership_materialization::tests::migration_22_rejects_state_rows_for_non_supergroups
```

Expected: fail because migration 22 only creates schema.

- [ ] **Step 3: Add source selection and never-run state helpers**

In `src-tauri/src/topic_memberships.rs`, add:

```rust
pub(crate) async fn catalog_backed_supergroup_source_ids(
    conn: &mut SqliteConnection,
) -> AppResult<Vec<i64>> {
    sqlx::query_scalar(
        "SELECT DISTINCT sources.id
         FROM sources
         JOIN telegram_forum_topics topics ON topics.source_id = sources.id
         WHERE sources.source_type = 'telegram'
           AND sources.source_subtype = 'supergroup'
         ORDER BY sources.id",
    )
    .fetch_all(&mut *conn)
    .await
    .map_err(AppError::database)
}

pub(crate) async fn ensure_never_run_state_for_supergroups_without_catalog(
    conn: &mut SqliteConnection,
    updated_at: i64,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO telegram_topic_resolution_state (
            source_id, resolver_version, catalog_refreshed_at, memberships_refreshed_at,
            status, unresolved_count, pending_item_count, last_error, updated_at
         )
         SELECT
            sources.id, ?, NULL, NULL, 'never_run', 0, 0, NULL, ?
         FROM sources
         WHERE sources.source_type = 'telegram'
           AND sources.source_subtype = 'supergroup'
           AND NOT EXISTS (
               SELECT 1 FROM telegram_forum_topics topics WHERE topics.source_id = sources.id
           )
         ON CONFLICT(source_id) DO UPDATE SET
            resolver_version = excluded.resolver_version,
            catalog_refreshed_at = NULL,
            memberships_refreshed_at = NULL,
            status = 'never_run',
            unresolved_count = 0,
            pending_item_count = 0,
            last_error = NULL,
            updated_at = excluded.updated_at",
    )
    .bind(CURRENT_TOPIC_RESOLVER_VERSION)
    .bind(updated_at)
    .execute(&mut *conn)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

pub(crate) async fn assert_all_topic_membership_invariants(
    conn: &mut SqliteConnection,
) -> AppResult<()> {
    assert_foreign_key_check_clean(conn).await?;
    assert_state_rows_only_for_supergroups(conn).await?;
    Ok(())
}
```

Add invariant helpers:

```rust
#[derive(sqlx::FromRow)]
struct ForeignKeyCheckRow {
    table: String,
    rowid: Option<i64>,
    parent: String,
    fkid: i64,
}

async fn assert_foreign_key_check_clean(conn: &mut SqliteConnection) -> AppResult<()> {
    let rows: Vec<ForeignKeyCheckRow> = sqlx::query_as("PRAGMA foreign_key_check")
        .fetch_all(&mut *conn)
        .await
        .map_err(AppError::database)?;
    if rows.is_empty() {
        return Ok(());
    }
    let detail = rows
        .into_iter()
        .map(|row| format!("{} rowid {:?} references {} via fk {}", row.table, row.rowid, row.parent, row.fkid))
        .collect::<Vec<_>>()
        .join("; ");
    Err(AppError::validation(format!(
        "Topic membership materialization migration 22 foreign_key_check failed: {detail}"
    )))
}

async fn assert_state_rows_only_for_supergroups(conn: &mut SqliteConnection) -> AppResult<()> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)
         FROM telegram_topic_resolution_state st
         JOIN sources s ON s.id = st.source_id
         WHERE s.source_type <> 'telegram'
            OR s.source_subtype <> 'supergroup'",
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(AppError::database)?;
    if count != 0 {
        return Err(AppError::validation(format!(
            "Migration 22 found {count} telegram_topic_resolution_state rows for non-supergroup sources"
        )));
    }
    Ok(())
}
```

- [ ] **Step 4: Wire rebuild into migration**

In `src-tauri/src/migrations/topic_membership_materialization.rs`, import:

```rust
use crate::topic_memberships::{
    assert_all_topic_membership_invariants, catalog_backed_supergroup_source_ids,
    create_topic_membership_schema, ensure_never_run_state_for_supergroups_without_catalog,
    rebuild_topic_memberships_for_source_on_connection,
};
use crate::sources::types::now_secs;
```

Inside the migration transaction, replace the schema-only body with:

```rust
create_topic_membership_schema(conn).await?;
let now = now_secs();
let source_ids = catalog_backed_supergroup_source_ids(conn).await?;
for source_id in source_ids {
    rebuild_topic_memberships_for_source_on_connection(conn, source_id, now, false).await?;
}
ensure_never_run_state_for_supergroups_without_catalog(conn, now).await?;
assert_all_topic_membership_invariants(conn).await
```

- [ ] **Step 5: Run migration data tests and focused migration suite**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::topic_membership_materialization:: migrations::tests::build_migrations_contains_all_versions_for_sqlx_validation
```

Expected:

```text
test result: ok
```

- [ ] **Step 6: Commit**

```powershell
git add src-tauri/src/migrations/topic_membership_materialization.rs src-tauri/src/topic_memberships.rs
git commit -m "feat: materialize topic memberships during migration"
```

## Task 5: Readers And Frontend API Shape Use Materialized Memberships

**Files:**
- Modify: `src-tauri/src/sources/topics.rs`
- Modify: `src-tauri/src/sources/items/query.rs`
- Modify: `src-tauri/src/notebooklm_export/query.rs`
- Modify: `src/lib/types/sources.ts`
- Modify: `src/lib/api/sources.ts`
- Modify: `src/routes/analysis/+page.svelte`
- Modify: `src/lib/api/sources.test.ts`

- [ ] **Step 1: Add RED backend reader tests**

Update existing tests in `src-tauri/src/sources/topics.rs` and `src-tauri/src/sources/items/query.rs` so fixtures insert `item_topic_memberships` and `telegram_topic_resolution_state` instead of relying on inference.

Add this test to `src-tauri/src/sources/items/query.rs`:

```rust
#[tokio::test]
async fn uncategorized_filter_returns_empty_when_topic_resolution_is_not_ready() {
    let pool = memory_pool_with_source_items_and_topics().await;
    sqlx::query(
        "INSERT OR IGNORE INTO sources (id, source_type, source_subtype, external_id, title, is_active, is_member, created_at)
         VALUES (1, 'telegram', 'supergroup', '12345', 'Forum', 1, 1, 1)",
    )
    .execute(&pool)
    .await
    .expect("seed source");
    sqlx::query(
        "INSERT INTO telegram_topic_resolution_state (
            source_id, resolver_version, status, unresolved_count, pending_item_count
         ) VALUES (1, 1, 'dirty', 1, 1)",
    )
    .execute(&pool)
    .await
    .expect("seed dirty state");
    sqlx::query(
        "INSERT INTO items (
            id, source_id, external_id, item_kind, author, published_at, ingested_at,
            content_kind, has_media
         ) VALUES (1, 1, '100', 'telegram_message', 'alice', 100, 100, 'text_only', 0)",
    )
    .execute(&pool)
    .await
    .expect("seed item");

    let rows = load_item_rows_from_pool(
        &pool,
        1,
        20,
        None,
        Some(ForumTopicFilter::Uncategorized),
        None,
    )
    .await
    .expect("load uncategorized rows");

    assert!(rows.is_empty());
}
```

Add this test to `src-tauri/src/notebooklm_export/query.rs`:

```rust
#[tokio::test]
async fn load_export_messages_reads_materialized_topic_memberships() {
    let pool = export_pool().await;
    seed_materialized_topic_schema(&pool).await;
    sqlx::query(
        "INSERT INTO sources (id, source_type, source_subtype, external_id, title)
         VALUES (1, 'telegram', 'supergroup', '12345', 'Forum')",
    )
    .execute(&pool)
    .await
    .expect("seed source");
    sqlx::query(
        "INSERT INTO telegram_forum_topics (
            source_id, topic_id, top_message_id, title, is_deleted
         ) VALUES (1, 200, 700, 'Roadmap', 0)",
    )
    .execute(&pool)
    .await
    .expect("seed topic");
    sqlx::query(
        "INSERT INTO items (
            id, source_id, external_id, author, published_at, content_zstd,
            content_kind, has_media
         ) VALUES (1, 1, '701', 'Ada', 100, ?, 'text_only', 0)",
    )
    .bind(compress_text("Reply in topic").expect("compress"))
    .execute(&pool)
    .await
    .expect("seed item");
    sqlx::query(
        "INSERT INTO item_topic_memberships (
            item_id, source_id, topic_id, match_kind, resolver_version
         ) VALUES (1, 1, 200, 'reply_to_top_id', 1)",
    )
    .execute(&pool)
    .await
    .expect("seed membership");

    let messages = load_export_messages(&pool, 1, None, None)
        .await
        .expect("load export messages");

    assert_eq!(messages[0].forum_topic_id, Some(200));
    assert_eq!(messages[0].forum_topic_title.as_deref(), Some("Roadmap"));
}
```

Add helper to export tests:

```rust
async fn seed_materialized_topic_schema(pool: &sqlx::SqlitePool) {
    crate::sources::test_support::create_topic_membership_tables(pool).await;
}
```

- [ ] **Step 2: Run reader tests and verify RED**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::topics::tests::list_source_forum_topics_returns_sorted_topics_and_uncategorized_bucket sources::items::query::tests::uncategorized_filter_returns_empty_when_topic_resolution_is_not_ready notebooklm_export::query::tests::load_export_messages_reads_materialized_topic_memberships
```

Expected: fail because readers still use inference joins and old topic response shape.

- [ ] **Step 3: Update backend topic response DTO**

In `src-tauri/src/sources/topics.rs`, add:

```rust
#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub struct TopicResolutionStateSummary {
    pub status: String,
    pub resolver_version: i64,
    pub unresolved_count: i64,
    pub pending_item_count: i64,
    pub memberships_refreshed_at: Option<i64>,
}

#[derive(Serialize)]
pub struct SourceForumTopicsResponse {
    pub topics: Vec<SourceForumTopicRecord>,
    pub topic_resolution_state: TopicResolutionStateSummary,
}
```

Change command signature:

```rust
) -> AppResult<SourceForumTopicsResponse> {
```

Add mapping helper:

```rust
fn state_summary_from_row(
    row: Option<&crate::topic_memberships::TopicResolutionStateRow>,
) -> TopicResolutionStateSummary {
    match row {
        Some(row) => TopicResolutionStateSummary {
            status: row.status.clone(),
            resolver_version: row.resolver_version,
            unresolved_count: row.unresolved_count,
            pending_item_count: row.pending_item_count,
            memberships_refreshed_at: row.memberships_refreshed_at,
        },
        None => TopicResolutionStateSummary {
            status: crate::topic_memberships::TOPIC_STATE_NEVER_RUN.to_string(),
            resolver_version: crate::topic_memberships::CURRENT_TOPIC_RESOLVER_VERSION,
            unresolved_count: 0,
            pending_item_count: 0,
            memberships_refreshed_at: None,
        },
    }
}
```

- [ ] **Step 4: Replace topic list query with membership counts**

In `list_source_forum_topics_from_pool`, remove `resolved_topic_predicate` and count through `item_topic_memberships`:

```rust
let state = crate::topic_memberships::load_topic_resolution_state(pool, source_id).await?;
let is_ready = crate::topic_memberships::is_ready_current_state(state.as_ref());
let rows: Vec<SourceForumTopicRow> = sqlx::query_as(
    r#"
    SELECT
        topics.topic_id,
        topics.top_message_id,
        topics.title,
        topics.icon_color,
        topics.icon_emoji_id,
        topics.is_closed,
        topics.is_pinned,
        topics.is_hidden,
        topics.is_deleted,
        topics.sort_order,
        COUNT(memberships.item_id) AS message_count
    FROM telegram_forum_topics AS topics
    LEFT JOIN item_topic_memberships AS memberships
      ON memberships.source_id = topics.source_id
     AND memberships.topic_id = topics.topic_id
    WHERE topics.source_id = ?
    GROUP BY
        topics.topic_id,
        topics.top_message_id,
        topics.title,
        topics.icon_color,
        topics.icon_emoji_id,
        topics.is_closed,
        topics.is_pinned,
        topics.is_hidden,
        topics.is_deleted,
        topics.sort_order
    ORDER BY
        topics.is_pinned DESC,
        topics.sort_order ASC NULLS LAST,
        topics.title COLLATE NOCASE ASC,
        topics.topic_id ASC
    "#,
)
.bind(source_id)
.fetch_all(pool)
.await
.map_err(|e| AppError::internal(e.to_string()))?;
```

Push `Unrecognized` only when ready and `state.unresolved_count > 0`:

```rust
if is_ready && state.as_ref().map(|row| row.unresolved_count).unwrap_or(0) > 0 {
    records.push(SourceForumTopicRecord {
        kind: "uncategorized".to_string(),
        key: FORUM_TOPIC_UNCATEGORIZED_KEY.to_string(),
        title: FORUM_TOPIC_UNCATEGORIZED_TITLE.to_string(),
        message_count: state.as_ref().map(|row| row.unresolved_count).unwrap_or(0),
        topic_id: None,
        top_message_id: None,
        icon_color: None,
        icon_emoji_id: None,
        is_closed: false,
        is_pinned: false,
        is_hidden: false,
        is_deleted: false,
        sort_order: None,
    });
}

Ok(SourceForumTopicsResponse {
    topics: records,
    topic_resolution_state: state_summary_from_row(state.as_ref()),
})
```

- [ ] **Step 5: Replace item reader joins**

In `src-tauri/src/sources/items/query.rs`, remove `resolved_topic_join` import. Build SQL with:

```rust
let state = crate::topic_memberships::load_topic_resolution_state(pool, source_id).await?;
let is_ready = crate::topic_memberships::is_ready_current_state(state.as_ref());
if matches!(topic_filter, Some(ForumTopicFilter::Uncategorized)) && !is_ready {
    return Ok(Vec::new());
}
let mut sql = String::from(
    r#"
    SELECT
        items.id,
        items.source_id,
        items.external_id,
        items.item_kind,
        items.author,
        items.published_at,
        items.content_kind,
        items.has_media,
        items.media_kind,
        items.content_zstd,
        items.media_metadata_zstd,
        items.raw_data_zstd,
        items.reply_to_msg_id,
        items.reply_to_peer_kind,
        items.reply_to_peer_id,
        items.reply_to_top_id,
        items.reaction_count,
        forum_topics.topic_id AS forum_topic_id,
        forum_topics.title AS forum_topic_title,
        forum_topics.top_message_id AS forum_topic_top_message_id
    FROM items
    LEFT JOIN item_topic_memberships AS memberships
      ON memberships.item_id = items.id
    LEFT JOIN telegram_forum_topics AS forum_topics
      ON forum_topics.source_id = memberships.source_id
     AND forum_topics.topic_id = memberships.topic_id
    WHERE items.source_id = ?
    "#,
);
```

For filters:

```rust
match topic_filter {
    Some(ForumTopicFilter::Topic { .. }) => sql.push_str(" AND memberships.topic_id = ?"),
    Some(ForumTopicFilter::Uncategorized) => sql.push_str(" AND memberships.item_id IS NULL"),
    None => {}
}
```

- [ ] **Step 6: Replace NotebookLM export join**

In `src-tauri/src/notebooklm_export/query.rs`, remove `resolved_topic_join` import. In `base_query`, replace `{topic_join}` with:

```sql
LEFT JOIN item_topic_memberships AS memberships
  ON memberships.item_id = items.id
LEFT JOIN telegram_forum_topics AS forum_topics
  ON forum_topics.source_id = memberships.source_id
 AND forum_topics.topic_id = memberships.topic_id
```

Keep the selected fields unchanged.

- [ ] **Step 7: Update frontend types and mapper**

In `src/lib/types/sources.ts`, add:

```ts
export type TopicResolutionStatus = "never_run" | "ready" | "dirty" | "rebuilding" | "failed";

export interface TopicResolutionStateSummary {
  status: TopicResolutionStatus;
  resolverVersion: number;
  unresolvedCount: number;
  pendingItemCount: number;
  membershipsRefreshedAt: number | null;
}

export interface SourceForumTopicsResult {
  topics: SourceForumTopic[];
  topicResolutionState: TopicResolutionStateSummary;
}
```

In `src/lib/api/sources.ts`, import `SourceForumTopicsResult` and add raw types:

```ts
interface RawTopicResolutionStateSummary {
  status: TopicResolutionStatus;
  resolver_version: number;
  unresolved_count: number;
  pending_item_count: number;
  memberships_refreshed_at: number | null;
}

interface RawSourceForumTopicsResponse {
  topics: RawSourceForumTopic[];
  topic_resolution_state: RawTopicResolutionStateSummary;
}
```

Change `listSourceForumTopics`:

```ts
export function listSourceForumTopics(sourceId: number) {
  return invoke<RawSourceForumTopicsResponse>(SOURCE_COMMANDS.listSourceForumTopics, {
    sourceId,
  }).then(mapSourceForumTopicsResponse);
}
```

Add mapper:

```ts
function mapSourceForumTopicsResponse(
  response: RawSourceForumTopicsResponse,
): SourceForumTopicsResult {
  return {
    topics: response.topics.map(mapSourceForumTopic),
    topicResolutionState: {
      status: response.topic_resolution_state.status,
      resolverVersion: response.topic_resolution_state.resolver_version,
      unresolvedCount: response.topic_resolution_state.unresolved_count,
      pendingItemCount: response.topic_resolution_state.pending_item_count,
      membershipsRefreshedAt: response.topic_resolution_state.memberships_refreshed_at,
    },
  };
}
```

In `src/routes/analysis/+page.svelte`, update the caller near `listSourceForumTopics`:

```ts
const result = await listSourceForumTopics(sourceId);
sourceTopics = result.topics;
```

No visual state display is required in this slice.

- [ ] **Step 8: Update frontend API test**

In `src/lib/api/sources.test.ts`, replace the forum topic mock with:

```ts
invokeMock.mockResolvedValueOnce({
  topics: [
    {
      kind: "topic",
      key: "topic:200",
      title: "Announcements",
      message_count: 3,
      topic_id: 200,
      top_message_id: 700,
      icon_color: 1,
      icon_emoji_id: 2,
      is_closed: false,
      is_pinned: true,
      is_hidden: false,
      is_deleted: false,
      sort_order: 4,
    },
  ],
  topic_resolution_state: {
    status: "ready",
    resolver_version: 1,
    unresolved_count: 0,
    pending_item_count: 0,
    memberships_refreshed_at: 1234,
  },
});
```

Expected assertion:

```ts
await expect(listSourceForumTopics(7)).resolves.toEqual({
  topics: [
    {
      kind: "topic",
      key: "topic:200",
      title: "Announcements",
      messageCount: 3,
      topicId: 200,
      topMessageId: 700,
      iconColor: 1,
      iconEmojiId: 2,
      isClosed: false,
      isPinned: true,
      isHidden: false,
      isDeleted: false,
      sortOrder: 4,
    },
  ],
  topicResolutionState: {
    status: "ready",
    resolverVersion: 1,
    unresolvedCount: 0,
    pendingItemCount: 0,
    membershipsRefreshedAt: 1234,
  },
});
```

- [ ] **Step 9: Run reader/frontend tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::topics:: sources::items::query:: notebooklm_export::query::
npm test -- --run src/lib/api/sources.test.ts src/lib/analysis-state.test.ts
```

Expected:

```text
test result: ok
```

and Vitest passes.

- [ ] **Step 10: Commit**

```powershell
git add src-tauri/src/sources/topics.rs src-tauri/src/sources/items/query.rs src-tauri/src/notebooklm_export/query.rs src/lib/types/sources.ts src/lib/api/sources.ts src/routes/analysis/+page.svelte src/lib/api/sources.test.ts
git commit -m "feat: read materialized topic memberships"
```

## Task 6: Full Rebuild After Topic Refresh

**Files:**
- Modify: `src-tauri/src/sources/topics.rs`
- Modify: `src-tauri/src/topic_memberships.rs`
- Modify: `src-tauri/src/source_ingest.rs` if lock enum labels need a topic-specific variant

- [ ] **Step 1: Add RED refresh rebuild test**

In `src-tauri/src/sources/topics.rs`, add:

```rust
#[tokio::test]
async fn topic_refresh_rebuilds_materialized_memberships() {
    let pool = memory_pool_with_source_items_and_topics().await;
    sqlx::query(
        "INSERT INTO sources (id, source_type, source_subtype, account_id, external_id, title, is_active, is_member, created_at)
         VALUES (1, 'telegram', 'supergroup', 42, '1', 'Forum', 1, 1, 1)",
    )
    .execute(&pool)
    .await
    .expect("seed source");
    sqlx::query(
        "INSERT INTO items (
            id, source_id, external_id, item_kind, author, published_at,
            ingested_at, content_kind, has_media, reply_to_top_id
         ) VALUES (10, 1, '10', 'telegram_message', 'alice', 10, 10, 'text_only', 0, 55)",
    )
    .execute(&pool)
    .await
    .expect("seed item");

    upsert_forum_topics_from_refresh(
        &pool,
        1,
        &[ForumTopicSnapshot {
            topic_id: 55,
            top_message_id: 500,
            title: "Fresh".to_string(),
            icon_color: 1,
            icon_emoji_id: None,
            is_closed: false,
            is_pinned: false,
            is_hidden: false,
            sort_order: 0,
        }],
        &[],
        2000,
    )
    .await
    .expect("refresh topics and rebuild");

    let topic_id: i64 = sqlx::query_scalar(
        "SELECT topic_id FROM item_topic_memberships WHERE item_id = 10",
    )
    .fetch_one(&pool)
    .await
    .expect("load membership");
    assert_eq!(topic_id, 55);

    let state: (String, i64) = sqlx::query_as(
        "SELECT status, unresolved_count FROM telegram_topic_resolution_state WHERE source_id = 1",
    )
    .fetch_one(&pool)
    .await
    .expect("load state");
    assert_eq!(state, ("ready".to_string(), 0));
}
```

- [ ] **Step 2: Run refresh test and verify RED**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::topics::tests::topic_refresh_rebuilds_materialized_memberships
```

Expected: fail because `upsert_forum_topics_from_refresh` does not rebuild memberships yet.

- [ ] **Step 3: Add pool wrapper for full rebuild and failure state**

In `src-tauri/src/topic_memberships.rs`, add:

```rust
pub(crate) async fn rebuild_topic_memberships_for_source(
    pool: &sqlx::Pool<Sqlite>,
    source_id: i64,
    refreshed_at: i64,
) -> AppResult<TopicRebuildStats> {
    let mut conn = pool.acquire().await.map_err(AppError::database)?;
    sqlx::query("BEGIN IMMEDIATE")
        .execute(&mut *conn)
        .await
        .map_err(AppError::database)?;

    let result = rebuild_topic_memberships_for_source_on_connection(
        &mut conn,
        source_id,
        refreshed_at,
        true,
    )
    .await;

    match result {
        Ok(stats) => {
            sqlx::query("COMMIT")
                .execute(&mut *conn)
                .await
                .map_err(AppError::database)?;
            Ok(stats)
        }
        Err(error) => {
            let _ = sqlx::query("ROLLBACK").execute(&mut *conn).await;
            mark_topic_resolution_failed(pool, source_id, &error.to_string(), refreshed_at).await?;
            Err(error)
        }
    }
}

pub(crate) async fn mark_topic_resolution_failed(
    pool: &sqlx::Pool<Sqlite>,
    source_id: i64,
    error: &str,
    updated_at: i64,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO telegram_topic_resolution_state (
            source_id, resolver_version, status, unresolved_count, pending_item_count,
            last_error, updated_at
         )
         VALUES (?, ?, 'failed', 0, 0, ?, ?)
         ON CONFLICT(source_id) DO UPDATE SET
            resolver_version = excluded.resolver_version,
            status = 'failed',
            last_error = excluded.last_error,
            updated_at = excluded.updated_at",
    )
    .bind(source_id)
    .bind(CURRENT_TOPIC_RESOLVER_VERSION)
    .bind(truncate_topic_resolution_error(error))
    .bind(updated_at)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}
```

This bounds `last_error` to `TOPIC_LAST_ERROR_MAX_CHARS` and avoids using state as a log store.

- [ ] **Step 4: Rebuild after successful topic refresh**

In `upsert_forum_topics_from_refresh`, after deleted-topic updates and before `Ok(())`, call:

```rust
crate::topic_memberships::rebuild_topic_memberships_for_source(
    pool,
    source_id,
    refreshed_at,
)
.await?;
```

This call is already serialized for sync and Takeout by `SourceIngestLocks`; if a standalone topic refresh command is added, it must acquire the same source ingest lock before calling this helper.

- [ ] **Step 5: Run refresh/topic tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::topics:: topic_memberships::
```

Expected:

```text
test result: ok
```

- [ ] **Step 6: Commit**

```powershell
git add src-tauri/src/sources/topics.rs src-tauri/src/topic_memberships.rs
git commit -m "feat: rebuild topic memberships after topic refresh"
```

## Task 7: Scoped Resolution For Newly Inserted Telegram Items

**Files:**
- Modify: `src-tauri/src/topic_memberships.rs`
- Modify: `src-tauri/src/sources/items.rs`
- Modify: `src-tauri/src/takeout_import/mod.rs`

- [ ] **Step 1: Add RED scoped resolution tests**

In `src-tauri/src/sources/items.rs`, add:

```rust
#[tokio::test]
async fn insert_telegram_source_item_resolves_topic_membership_only_for_new_item() {
    let pool = memory_pool_with_source_items_and_topics().await;
    seed_item_source(&pool, 1).await;
    sqlx::query(
        "INSERT INTO telegram_forum_topics (
            source_id, topic_id, top_message_id, title, last_seen_at, updated_at
         ) VALUES (1, 200, 700, 'Roadmap', 100, 100)",
    )
    .execute(&pool)
    .await
    .expect("seed topic");
    sqlx::query(
        "INSERT INTO telegram_topic_resolution_state (
            source_id, resolver_version, status, unresolved_count, pending_item_count
         ) VALUES (1, 1, 'ready', 0, 0)",
    )
    .execute(&pool)
    .await
    .expect("seed ready state");

    let identity = TelegramMessageIdentity {
        history_peer_kind: "channel".to_string(),
        history_peer_id: 12345,
        telegram_message_id: 701,
        migration_domain: None,
        is_migrated_history: false,
    };
    let mut item = telegram_insert("701", "topic reply");
    item.telegram_context.reply_to_top_id = Some(200);

    assert!(
        insert_telegram_source_item(&pool, 1, identity.clone(), item)
            .await
            .expect("insert")
    );
    assert!(
        !insert_telegram_source_item(&pool, 1, identity, telegram_insert("701", "duplicate"))
            .await
            .expect("duplicate")
    );

    let membership_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM item_topic_memberships WHERE source_id = 1",
    )
    .fetch_one(&pool)
    .await
    .expect("count memberships");
    assert_eq!(membership_count, 1);

    let state: (String, i64, i64) = sqlx::query_as(
        "SELECT status, unresolved_count, pending_item_count FROM telegram_topic_resolution_state WHERE source_id = 1",
    )
    .fetch_one(&pool)
    .await
    .expect("load state");
    assert_eq!(state, ("ready".to_string(), 0, 0));
}

#[tokio::test]
async fn scoped_resolution_increments_unresolved_count_for_inserted_unmatched_item() {
    let pool = memory_pool_with_source_items_and_topics().await;
    seed_item_source(&pool, 1).await;
    sqlx::query(
        "INSERT INTO telegram_topic_resolution_state (
            source_id, resolver_version, status, unresolved_count, pending_item_count
         ) VALUES (1, 1, 'ready', 2, 0)",
    )
    .execute(&pool)
    .await
    .expect("seed ready state");

    let identity = TelegramMessageIdentity {
        history_peer_kind: "channel".to_string(),
        history_peer_id: 12345,
        telegram_message_id: 900,
        migration_domain: None,
        is_migrated_history: false,
    };

    assert!(
        insert_telegram_source_item(&pool, 1, identity, telegram_insert("900", "unmatched"))
            .await
            .expect("insert unmatched")
    );

    let state: (String, i64, i64) = sqlx::query_as(
        "SELECT status, unresolved_count, pending_item_count FROM telegram_topic_resolution_state WHERE source_id = 1",
    )
    .fetch_one(&pool)
    .await
    .expect("load state");
    assert_eq!(state, ("ready".to_string(), 3, 0));
}
```

- [ ] **Step 2: Run scoped tests and verify RED**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::items::tests::insert_telegram_source_item_resolves_topic_membership_only_for_new_item sources::items::tests::scoped_resolution_increments_unresolved_count_for_inserted_unmatched_item
```

Expected: fail because `insert_telegram_source_item` does not call scoped resolution.

- [ ] **Step 3: Implement scoped resolver with transactional delta**

In `src-tauri/src/topic_memberships.rs`, replace `resolve_scoped_topic_memberships_on_connection` with logic that:

- returns `Ok(())` when `inserted_item_ids.is_empty()`;
- loads state and exits if not ready/current;
- inserts memberships for only inserted ids;
- computes `inserted_eligible_count - inserted_membership_count`;
- updates `unresolved_count` and `pending_item_count` in the same transaction.

Implementation shape:

```rust
pub(crate) async fn resolve_scoped_topic_memberships_on_connection(
    conn: &mut SqliteConnection,
    source_id: i64,
    inserted_item_ids: &[i64],
    resolved_at: i64,
) -> AppResult<()> {
    if inserted_item_ids.is_empty() {
        return Ok(());
    }

    let state: Option<TopicResolutionStateRow> = sqlx::query_as(
        "SELECT source_id, resolver_version, catalog_refreshed_at, memberships_refreshed_at,
                status, unresolved_count, pending_item_count, last_error
         FROM telegram_topic_resolution_state
         WHERE source_id = ?",
    )
    .bind(source_id)
    .fetch_optional(&mut *conn)
    .await
    .map_err(AppError::database)?;
    if !is_ready_current_state(state.as_ref()) {
        return Ok(());
    }

    let placeholders = std::iter::repeat_n("?", inserted_item_ids.len())
        .collect::<Vec<_>>()
        .join(", ");
    let scoped_select = RESOLVED_MEMBERSHIP_SELECT_SQL.replace(
        "WHERE items.source_id = ?",
        &format!("WHERE items.source_id = ? AND items.id IN ({placeholders})"),
    );
    let insert_sql = format!(
        "INSERT OR REPLACE INTO item_topic_memberships (
            item_id, source_id, topic_id, match_kind, resolver_version, created_at, updated_at
         )
         SELECT item_id, source_id, topic_id, match_kind, ?, ?, ?
         FROM ({scoped_select})"
    );
    let mut insert = sqlx::query(&insert_sql)
        .bind(CURRENT_TOPIC_RESOLVER_VERSION)
        .bind(resolved_at)
        .bind(resolved_at)
        .bind(source_id);
    for item_id in inserted_item_ids {
        insert = insert.bind(item_id);
    }
    let inserted_memberships = insert
        .execute(&mut *conn)
        .await
        .map_err(AppError::database)?
        .rows_affected() as i64;

    let eligible_sql = format!(
        "SELECT COUNT(*)
         FROM items
         JOIN sources ON sources.id = items.source_id
         WHERE items.source_id = ?
           AND items.id IN ({placeholders})
           AND sources.source_type = 'telegram'
           AND sources.source_subtype = 'supergroup'
           AND items.item_kind = 'telegram_message'"
    );
    let mut eligible_query = sqlx::query_scalar::<_, i64>(&eligible_sql).bind(source_id);
    for item_id in inserted_item_ids {
        eligible_query = eligible_query.bind(item_id);
    }
    let inserted_eligible_count = eligible_query
        .fetch_one(&mut *conn)
        .await
        .map_err(AppError::database)?;
    let unresolved_delta = (inserted_eligible_count - inserted_memberships).max(0);

    sqlx::query(
        "UPDATE telegram_topic_resolution_state
         SET unresolved_count = unresolved_count + ?,
             pending_item_count = 0,
             last_error = NULL,
             updated_at = ?
         WHERE source_id = ?",
    )
    .bind(unresolved_delta)
    .bind(resolved_at)
    .bind(source_id)
    .execute(&mut *conn)
    .await
    .map_err(AppError::database)?;

    Ok(())
}
```

This is the required scoped delta:

```text
unresolved_delta = inserted_eligible_count - inserted_membership_count
```

It runs in the same SQLite transaction as the membership insert/state update. Duplicate inserts with `inserted = false` must not call this helper and must not increment `unresolved_count`.

- [ ] **Step 4: Call scoped resolver from Telegram insert transaction**

In `src-tauri/src/sources/items.rs`, after inserting `telegram_messages` and before `Ok(true)`, call:

```rust
crate::topic_memberships::resolve_scoped_topic_memberships_on_connection(
    &mut conn,
    source_id,
    &[item_id],
    now_secs(),
)
.await?;
```

This keeps membership insert and state delta in the same transaction as the new item. Existing duplicate path returns `Ok(false)` before `item_id` exists and therefore does not update scoped counts.

- [ ] **Step 5: Run scoped tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::items:: topic_memberships::
```

Expected:

```text
test result: ok
```

- [ ] **Step 6: Verify Takeout duplicate behavior**

Add to `src-tauri/src/takeout_import/mod.rs` test `takeout_parsed_items_with_same_message_id_insert_under_different_history_peers` or a new nearby test:

```rust
let state: (String, i64) = sqlx::query_as(
    "SELECT status, unresolved_count FROM telegram_topic_resolution_state WHERE source_id = 1",
)
.fetch_one(&pool)
.await
.expect("load topic state");
assert_eq!(state.0, "ready");
```

If this fixture does not seed topic state, create a separate test that seeds ready state, inserts the same parsed item twice, and asserts `unresolved_count` changes only once.

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml takeout_import::tests::takeout_
```

Expected:

```text
test result: ok
```

- [ ] **Step 7: Commit**

```powershell
git add src-tauri/src/topic_memberships.rs src-tauri/src/sources/items.rs src-tauri/src/takeout_import/mod.rs
git commit -m "feat: resolve topic membership for inserted telegram items"
```

## Task 8: Documentation, Containment Scans, And Focused Verification

**Files:**
- Modify: `docs/database-schema.md`
- Modify: `docs/backlog.md`
- Verify: containment scans and focused tests

- [ ] **Step 1: Update database schema docs**

In `docs/database-schema.md`, add sections for `item_topic_memberships` and `telegram_topic_resolution_state` near `telegram_forum_topics`.

Include these notes:

```markdown
`item_topic_memberships` stores only real Telegram forum topic memberships.
`Unrecognized topic` is a derived bucket for ready/current resolution state and
is not persisted as a topic or membership row.

Reader truth is source-level `telegram_topic_resolution_state`. Row-level
`item_topic_memberships.resolver_version` is diagnostic and must match state
version for ready sources.

`telegram_topic_resolution_state` rows are valid only for Telegram supergroup
sources. Missing state is treated defensively as `never_run`.
```

- [ ] **Step 2: Update backlog open work**

In `docs/backlog.md`, keep the Database Schema Simplification item open for the provider-neutral document layer and Takeout provenance. Do not add completed-work notes.

If the current line says topic membership materialization is still open, replace it with:

```markdown
- [ ] continue item/document identity cleanup after topic membership
  materialization, including Takeout provenance and a later provider-neutral
  document layer
```

- [ ] **Step 3: Run containment scans**

Run:

```powershell
rg -n "CAST\\(.*external_id AS INTEGER\\)|external_id NOT GLOB" src-tauri\src
rg -n "reply_to_top_id.*telegram_forum_topics|top_message_id|reply_to_msg_id.*telegram_forum_topics" src-tauri\src\sources src-tauri\src\notebooklm_export
rg -n "item_topic_memberships|telegram_topic_resolution_state" src-tauri\src src-tauri\migrations docs
```

Expected:

- `items.external_id` integer casts appear only in topic resolver legacy fallback, migration tests, or existing Telegram item identity migration backfill.
- `top_message_id` and topic resolver fields appear in resolver, migration, tests, and docs, but not as embedded inference logic in `sources/items/query.rs`, `sources/topics.rs`, or `notebooklm_export/query.rs`.
- new membership/state tables appear in migration, resolver/runtime code, docs, and tests.

- [ ] **Step 4: Run focused backend and frontend verification**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations:: topic_memberships:: sources::topics:: sources::items:: sources::items::query:: takeout_import:: notebooklm_export::query::
npm test -- --run src/lib/api/sources.test.ts src/lib/analysis-state.test.ts
```

Expected:

```text
test result: ok
```

and Vitest passes.

- [ ] **Step 5: Commit**

```powershell
git add docs/database-schema.md docs/backlog.md
git commit -m "docs: document topic membership materialization"
```

## Task 9: Final Verification

**Files:**
- Modify: this plan if task checkboxes were updated during execution
- Verify: full project status

- [ ] **Step 1: Run full Rust verification**

Use `superpowers:verification-before-completion`, then run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
cargo fmt --manifest-path src-tauri/Cargo.toml --check
git diff --check
```

Expected:

```text
test result: ok
```

`cargo fmt --check` and `git diff --check` exit 0.

- [ ] **Step 2: Run frontend verification**

Because this plan changes TypeScript API types and `+page.svelte`, run:

```powershell
npm run check
npm test -- --run
```

Expected:

```text
0 errors
```

and Vitest passes.

- [ ] **Step 3: Final git status**

Run:

```powershell
git status --short --branch
git --no-pager log -12 --oneline --decorate
```

Expected: only expected committed branch state.

- [ ] **Step 4: Commit plan checkbox updates if changed**

If this plan was updated during execution:

```powershell
git add docs/superpowers/plans/2026-05-17-topic-membership-materialization.md
git commit -m "docs: update topic membership plan progress"
```

- [ ] **Step 5: Finish the branch**

Use `superpowers:finishing-a-development-branch`. Present merge/push/keep/discard options after verification passes.

## Self-Review

- Spec coverage: Tasks 1-4 cover migration 22, schema, persistent state rows, full rebuild, resolver order, retained topics, and migration invariants. Task 5 covers readers/export, state-aware `Unrecognized`, backend-owned resolver-version freshness, and frontend API unwrapping. Task 6 covers the topic refresh correctness boundary and bounded `last_error`. Task 7 covers scoped sync/Takeout resolution, same-transaction unresolved deltas, duplicate insert behavior, and stale resolver-version replacement through rebuilds. Task 8 covers docs and containment scans. Task 9 covers final verification.
- Placeholder scan: No task uses deferred-work markers. Each code-changing task names files, tests, commands, and expected results.
- Type consistency: The plan consistently uses `item_topic_memberships`, `telegram_topic_resolution_state`, `CURRENT_TOPIC_RESOLVER_VERSION`, `TopicResolutionStateRow`, `SourceForumTopicsResponse`, `TopicResolutionStateSummary`, `resolverVersion`, `unresolvedCount`, and `pendingItemCount`.
