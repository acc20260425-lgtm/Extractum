# Takeout Provenance Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add durable generic ingest provenance tables and wire them into Telegram Takeout imports only.

**Architecture:** Use regular SQL migration `23.sql` for the durable schema, then add a small Rust storage module that owns batch creation, warnings, observations, counters, and terminal updates. Refactor Telegram item insertion so Takeout can record inserted, duplicate-observed, and skipped observations in the same SQLite writer transaction while normal `sync_source` keeps its current public behavior.

**Tech Stack:** Rust, Tauri commands, SQLx SQLite, `tauri_plugin_sql` migrations, Telegram Takeout flow in `grammers_client`.

---

## File Structure

- Create `src-tauri/migrations/23.sql`: regular SQL migration for `ingest_batches`, `telegram_takeout_batches`, `ingest_item_observations`, `ingest_batch_warnings`, and their indexes.
- Create `src-tauri/src/ingest_provenance.rs`: internal storage/query helpers for batch creation, Telegram Takeout detail updates, warnings, observations, terminal counter recalculation, and sanitization.
- Modify `src-tauri/src/lib.rs`: register `mod ingest_provenance;`.
- Modify `src-tauri/src/migrations.rs`: register migration 23 and update migration tests/test helper behavior.
- Modify `src-tauri/src/sources/test_support.rs`: add a provenance schema fixture helper for item-level tests.
- Modify `src-tauri/src/sources/items.rs`: add `TelegramItemInsertOutcome`, split the open-transaction Telegram insert body, keep the existing bool wrapper for normal sync, and add a Takeout observation wrapper.
- Modify `src-tauri/src/sources/mod.rs`: export the new item insert outcome/wrapper only inside the crate.
- Modify `src-tauri/src/takeout_import/state.rs`: add `batch_id` to `TakeoutImportJobRecord` and job creation.
- Modify `src-tauri/src/takeout_import/export_dc.rs`: expose enough export-DC attempt/fallback signal for durable provenance.
- Modify `src-tauri/src/takeout_import/mod.rs`: acquire the source lock before durable batch/job creation, pass the lock guard into the task, write Takeout detail updates/warnings/observations/terminal status, and leave normal sync untouched.
- Modify `docs/database-schema.md`: document migration 23 tables and counter semantics.
- Modify `docs/takeout-source-import.md`: document durable provenance, complete vs partial semantics, and crash-interrupted `running`.

---

### Task 1: Migration 23 Schema

**Files:**
- Create: `src-tauri/migrations/23.sql`
- Modify: `src-tauri/src/migrations.rs`
- Test: `src-tauri/src/migrations.rs`

- [ ] **Step 1: Write failing migration registration tests**

Add these tests inside `#[cfg(test)] mod tests` in `src-tauri/src/migrations.rs`:

```rust
#[test]
fn includes_regular_ingest_provenance_migration() {
    let migrations = build_migrations();
    let migration = migrations
        .iter()
        .find(|migration| migration.version == 23)
        .expect("version 23 migration is registered");

    assert_eq!(migration.description, "add ingest provenance foundation");
    assert!(migration.sql.contains("CREATE TABLE ingest_batches"));
    assert!(migration.sql.contains("CREATE TABLE telegram_takeout_batches"));
    assert!(migration.sql.contains("CREATE TABLE ingest_item_observations"));
    assert!(migration.sql.contains("CREATE TABLE ingest_batch_warnings"));
    assert!(!migration.sql.contains("runner_managed"));
}

#[test]
fn build_migrations_contains_all_versions_for_sqlx_validation() {
    let versions = build_migrations()
        .into_iter()
        .map(|migration| migration.version)
        .collect::<Vec<_>>();

    assert_eq!(versions, (1_i64..=23_i64).collect::<Vec<_>>());
}
```

Replace the existing `build_migrations_contains_all_versions_for_sqlx_validation` test instead of leaving two copies.

- [ ] **Step 2: Run registration tests and confirm failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::tests::includes_regular_ingest_provenance_migration migrations::tests::build_migrations_contains_all_versions_for_sqlx_validation
```

Expected: the first test fails because migration 23 is not registered; the second fails because the version list still ends at 22.

- [ ] **Step 3: Add SQL migration 23**

Create `src-tauri/migrations/23.sql` with:

```sql
CREATE TABLE ingest_batches (
  id INTEGER PRIMARY KEY AUTOINCREMENT,

  source_id INTEGER NOT NULL REFERENCES sources(id) ON DELETE CASCADE,

  provider TEXT NOT NULL,
  ingest_kind TEXT NOT NULL,

  status TEXT NOT NULL,
  completeness TEXT NOT NULL DEFAULT 'unknown',

  started_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  finished_at TEXT,

  item_inserted_count INTEGER NOT NULL DEFAULT 0,
  item_observed_count INTEGER NOT NULL DEFAULT 0,
  item_duplicate_count INTEGER NOT NULL DEFAULT 0,
  item_skipped_count INTEGER NOT NULL DEFAULT 0,
  warning_count INTEGER NOT NULL DEFAULT 0,

  terminal_error TEXT,

  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

  CHECK (provider IN ('telegram', 'youtube')),
  CHECK (ingest_kind IN (
    'takeout',
    'sync',
    'youtube_metadata',
    'youtube_transcript',
    'youtube_comments',
    'youtube_playlist'
  )),
  CHECK (status IN ('running', 'completed', 'failed', 'cancelled')),
  CHECK (completeness IN ('unknown', 'complete', 'partial')),
  CHECK (
    (status = 'running' AND finished_at IS NULL)
    OR
    (status IN ('completed', 'failed', 'cancelled') AND finished_at IS NOT NULL)
  ),
  CHECK (item_inserted_count >= 0),
  CHECK (item_observed_count >= 0),
  CHECK (item_duplicate_count >= 0),
  CHECK (item_skipped_count >= 0),
  CHECK (warning_count >= 0),
  CHECK (
    item_observed_count >=
    item_inserted_count + item_duplicate_count + item_skipped_count
  )
);

CREATE TABLE telegram_takeout_batches (
  batch_id INTEGER PRIMARY KEY REFERENCES ingest_batches(id) ON DELETE CASCADE,

  account_id INTEGER NOT NULL,
  source_subtype TEXT NOT NULL,

  resolved_peer_kind TEXT,
  resolved_peer_id INTEGER,
  history_peer_kind TEXT,
  history_peer_id INTEGER,

  takeout_id INTEGER,
  export_dc_id INTEGER,
  used_export_dc INTEGER NOT NULL DEFAULT 0,
  fallback_used INTEGER NOT NULL DEFAULT 0,

  history_scope TEXT NOT NULL DEFAULT 'unknown',

  migrated_history_detected INTEGER NOT NULL DEFAULT 0,
  migrated_history_imported INTEGER NOT NULL DEFAULT 0,
  only_my_messages INTEGER NOT NULL DEFAULT 0,

  split_count INTEGER,
  selected_split_count INTEGER,
  message_count_estimate INTEGER,
  max_message_id INTEGER,

  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

  CHECK (source_subtype IN ('channel', 'supergroup', 'group')),
  CHECK (resolved_peer_kind IS NULL OR resolved_peer_kind IN ('channel', 'chat')),
  CHECK (history_peer_kind IS NULL OR history_peer_kind IN ('channel', 'chat', 'user')),
  CHECK (history_scope IN (
    'unknown',
    'current_history',
    'current_history_with_migrated_deferred',
    'partial_private_history',
    'mixed_partial'
  )),
  CHECK (used_export_dc IN (0, 1)),
  CHECK (fallback_used IN (0, 1)),
  CHECK (migrated_history_detected IN (0, 1)),
  CHECK (migrated_history_imported IN (0, 1)),
  CHECK (only_my_messages IN (0, 1))
);

CREATE TABLE ingest_item_observations (
  id INTEGER PRIMARY KEY AUTOINCREMENT,

  batch_id INTEGER NOT NULL REFERENCES ingest_batches(id) ON DELETE CASCADE,
  source_id INTEGER NOT NULL REFERENCES sources(id) ON DELETE CASCADE,

  item_id INTEGER REFERENCES items(id) ON DELETE SET NULL,

  provider_item_kind TEXT NOT NULL,
  provider_identity_kind TEXT NOT NULL,
  provider_identity TEXT NOT NULL,
  provider_identity_version INTEGER NOT NULL DEFAULT 1,

  outcome TEXT NOT NULL,
  reason_code TEXT,

  observed_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

  CHECK (provider_item_kind IN ('telegram_message')),
  CHECK (provider_identity_version >= 1),
  CHECK (outcome IN ('inserted', 'duplicate_observed', 'skipped', 'failed'))
);

CREATE TABLE ingest_batch_warnings (
  id INTEGER PRIMARY KEY AUTOINCREMENT,

  batch_id INTEGER NOT NULL REFERENCES ingest_batches(id) ON DELETE CASCADE,

  code TEXT NOT NULL,
  message TEXT NOT NULL,

  created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_ingest_batches_source_started
ON ingest_batches(source_id, started_at DESC);

CREATE INDEX idx_ingest_batches_status
ON ingest_batches(status);

CREATE INDEX idx_telegram_takeout_batches_account
ON telegram_takeout_batches(account_id);

CREATE INDEX idx_ingest_item_observations_batch
ON ingest_item_observations(batch_id);

CREATE INDEX idx_ingest_item_observations_item
ON ingest_item_observations(item_id)
WHERE item_id IS NOT NULL;

CREATE INDEX idx_ingest_item_observations_identity
ON ingest_item_observations(source_id, provider_identity_kind, provider_identity);

CREATE INDEX idx_ingest_item_observations_batch_outcome
ON ingest_item_observations(batch_id, outcome);

CREATE INDEX idx_ingest_batch_warnings_batch
ON ingest_batch_warnings(batch_id);
```

- [ ] **Step 4: Register migration 23**

In `src-tauri/src/migrations.rs`, append this entry after version 22:

```rust
Migration {
    version: 23,
    description: "add ingest provenance foundation",
    sql: include_str!("../migrations/23.sql"),
    kind: MigrationKind::Up,
},
```

- [ ] **Step 5: Update the test migration helper for post-runner SQL migrations**

In `apply_all_migrations_for_test_pool`, after `topic_membership_materialization::apply_topic_membership_materialization_on_connection(conn).await?;`, execute regular SQL migrations above v22:

```rust
topic_membership_materialization::apply_topic_membership_materialization_on_connection(conn)
    .await?;

for migration in build_migrations()
    .into_iter()
    .filter(|migration| migration.version > 22)
{
    sqlx::raw_sql(migration.sql)
        .execute(&mut *conn)
        .await
        .map_err(crate::error::AppError::database)?;
}

Ok(())
```

The function currently returns the v22 result directly, so convert that tail expression into the block above.

- [ ] **Step 6: Add schema behavior tests**

Add this async test in `src-tauri/src/migrations.rs`:

```rust
#[tokio::test]
async fn fresh_schema_includes_ingest_provenance_tables_indexes_and_constraints() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("connect memory sqlite");
    apply_all_migrations_for_test_pool(&pool)
        .await
        .expect("apply migrations");

    for table in [
        "ingest_batches",
        "telegram_takeout_batches",
        "ingest_item_observations",
        "ingest_batch_warnings",
    ] {
        let exists: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?",
        )
        .bind(table)
        .fetch_one(&pool)
        .await
        .expect("check table");
        assert_eq!(exists, 1, "missing table {table}");
    }

    for index in [
        "idx_ingest_batches_source_started",
        "idx_ingest_batches_status",
        "idx_telegram_takeout_batches_account",
        "idx_ingest_item_observations_batch",
        "idx_ingest_item_observations_item",
        "idx_ingest_item_observations_identity",
        "idx_ingest_item_observations_batch_outcome",
        "idx_ingest_batch_warnings_batch",
    ] {
        let exists: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = ?",
        )
        .bind(index)
        .fetch_one(&pool)
        .await
        .expect("check index");
        assert_eq!(exists, 1, "missing index {index}");
    }

    sqlx::query(
        "INSERT INTO sources (
            id, source_type, source_subtype, account_id, external_id, title,
            metadata_zstd, last_sync_state, last_synced_at, is_active, is_member, created_at
         ) VALUES (1, 'telegram', 'supergroup', 10, '12345', 'Forum', NULL, NULL, NULL, 1, 1, 1)",
    )
    .execute(&pool)
    .await
    .expect("seed source");

    let batch_id: i64 = sqlx::query_scalar(
        "INSERT INTO ingest_batches (source_id, provider, ingest_kind, status)
         VALUES (1, 'telegram', 'takeout', 'running')
         RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .expect("insert running batch");

    sqlx::query(
        "INSERT INTO telegram_takeout_batches (batch_id, account_id, source_subtype)
         VALUES (?, 10, 'supergroup')",
    )
    .bind(batch_id)
    .execute(&pool)
    .await
    .expect("insert takeout detail");

    let terminal_without_finished_at = sqlx::query(
        "UPDATE ingest_batches SET status = 'completed' WHERE id = ?",
    )
    .bind(batch_id)
    .execute(&pool)
    .await;
    assert!(terminal_without_finished_at.is_err());

    sqlx::query(
        "INSERT INTO ingest_item_observations (
            batch_id, source_id, provider_item_kind, provider_identity_kind,
            provider_identity, outcome
         ) VALUES (?, 1, 'telegram_message', 'telegram_message',
            'telegram:history_peer:channel:12345:message:42', 'duplicate_observed')",
    )
    .bind(batch_id)
    .execute(&pool)
    .await
    .expect("insert first observation");

    sqlx::query(
        "INSERT INTO ingest_item_observations (
            batch_id, source_id, provider_item_kind, provider_identity_kind,
            provider_identity, outcome
         ) VALUES (?, 1, 'telegram_message', 'telegram_message',
            'telegram:history_peer:channel:12345:message:42', 'duplicate_observed')",
    )
    .bind(batch_id)
    .execute(&pool)
    .await
    .expect("duplicate observation rows are allowed");

    sqlx::query(
        "INSERT INTO ingest_batch_warnings (batch_id, code, message)
         VALUES (?, 'export_dc_fallback', 'first')",
    )
    .bind(batch_id)
    .execute(&pool)
    .await
    .expect("insert first warning");
    sqlx::query(
        "INSERT INTO ingest_batch_warnings (batch_id, code, message)
         VALUES (?, 'export_dc_fallback', 'second')",
    )
    .bind(batch_id)
    .execute(&pool)
    .await
    .expect("duplicate warning codes are allowed");
}
```

- [ ] **Step 7: Run migration tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::tests::
```

Expected: all migration tests pass.

- [ ] **Step 8: Commit migration work**

Run:

```powershell
git add src-tauri/migrations/23.sql src-tauri/src/migrations.rs
git commit -m "feat: add ingest provenance schema"
```

Expected: commit succeeds with migration schema and tests.

---

### Task 2: Ingest Provenance Storage Helpers

**Files:**
- Create: `src-tauri/src/ingest_provenance.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/sources/test_support.rs`
- Test: `src-tauri/src/ingest_provenance.rs`

- [ ] **Step 1: Register the new module**

In `src-tauri/src/lib.rs`, add this near the other backend modules:

```rust
mod ingest_provenance;
```

- [ ] **Step 2: Add a test fixture helper for provenance tables**

In `src-tauri/src/sources/test_support.rs`, add:

```rust
pub(crate) async fn create_ingest_provenance_tables(pool: &sqlx::SqlitePool) {
    sqlx::raw_sql(include_str!("../../migrations/23.sql"))
        .execute(pool)
        .await
        .expect("create ingest provenance schema");
}
```

Update `source_fixture_creates_expected_tables` to call `create_ingest_provenance_tables(&pool).await;` and include these table names in the assertion list:

```rust
"ingest_batches",
"telegram_takeout_batches",
"ingest_item_observations",
"ingest_batch_warnings",
```

- [ ] **Step 3: Write failing storage helper tests**

Create `src-tauri/src/ingest_provenance.rs` with only the test module first:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::sources::test_support::{
        create_ingest_provenance_tables, memory_pool_with_source_items_and_topics,
    };

    async fn seed_source(pool: &sqlx::SqlitePool) {
        sqlx::query(
            "INSERT INTO sources (
                id, source_type, source_subtype, account_id, external_id, title,
                is_active, is_member, created_at
             ) VALUES (1, 'telegram', 'supergroup', 10, '12345', 'Forum', 1, 1, 1)",
        )
        .execute(pool)
        .await
        .expect("seed source");
    }

    #[tokio::test]
    async fn create_takeout_batch_inserts_generic_and_detail_rows_atomically() {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_ingest_provenance_tables(&pool).await;
        seed_source(&pool).await;

        let batch_id = create_telegram_takeout_batch(
            &pool,
            CreateTelegramTakeoutBatch {
                source_id: 1,
                account_id: 10,
                source_subtype: "supergroup".to_string(),
            },
        )
        .await
        .expect("create batch");

        let generic: (String, String, String, String, Option<String>) = sqlx::query_as(
            "SELECT provider, ingest_kind, status, completeness, finished_at
             FROM ingest_batches WHERE id = ?",
        )
        .bind(batch_id)
        .fetch_one(&pool)
        .await
        .expect("load generic batch");
        assert_eq!(
            generic,
            (
                "telegram".to_string(),
                "takeout".to_string(),
                "running".to_string(),
                "unknown".to_string(),
                None
            )
        );

        let detail: (i64, String) = sqlx::query_as(
            "SELECT account_id, source_subtype FROM telegram_takeout_batches WHERE batch_id = ?",
        )
        .bind(batch_id)
        .fetch_one(&pool)
        .await
        .expect("load detail");
        assert_eq!(detail, (10, "supergroup".to_string()));
    }

    #[tokio::test]
    async fn terminal_update_recalculates_counters_and_sanitizes_error() {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_ingest_provenance_tables(&pool).await;
        seed_source(&pool).await;
        let batch_id = create_telegram_takeout_batch(
            &pool,
            CreateTelegramTakeoutBatch {
                source_id: 1,
                account_id: 10,
                source_subtype: "supergroup".to_string(),
            },
        )
        .await
        .expect("create batch");

        record_ingest_observation(
            &pool,
            IngestObservation {
                batch_id,
                source_id: 1,
                item_id: Some(11),
                provider_item_kind: "telegram_message",
                provider_identity_kind: "telegram_message",
                provider_identity: "telegram:history_peer:channel:12345:message:1".to_string(),
                outcome: "inserted",
                reason_code: None,
            },
        )
        .await
        .expect("record inserted");
        record_ingest_observation(
            &pool,
            IngestObservation {
                batch_id,
                source_id: 1,
                item_id: Some(11),
                provider_item_kind: "telegram_message",
                provider_identity_kind: "telegram_message",
                provider_identity: "telegram:history_peer:channel:12345:message:1".to_string(),
                outcome: "duplicate_observed",
                reason_code: None,
            },
        )
        .await
        .expect("record duplicate");
        record_ingest_observation(
            &pool,
            IngestObservation {
                batch_id,
                source_id: 1,
                item_id: None,
                provider_item_kind: "telegram_message",
                provider_identity_kind: "telegram_message",
                provider_identity: "telegram:history_peer:channel:12345:message:2".to_string(),
                outcome: "skipped",
                reason_code: Some("empty_payload"),
            },
        )
        .await
        .expect("record skipped");
        record_ingest_batch_warning(
            &pool,
            batch_id,
            "generic_warning",
            "{\"raw\":\"payload\",\"api_hash\":\"secret\"}",
        )
        .await
        .expect("record warning");

        finalize_ingest_batch(
            &pool,
            batch_id,
            TerminalBatchStatus::Failed,
            Some("{\"raw\":\"payload\",\"session\":\"secret\"}"),
        )
        .await
        .expect("finalize batch");

        let row: (String, String, i64, i64, i64, i64, i64, String) = sqlx::query_as(
            "SELECT status, completeness, item_observed_count, item_inserted_count,
                    item_duplicate_count, item_skipped_count, warning_count, terminal_error
             FROM ingest_batches WHERE id = ?",
        )
        .bind(batch_id)
        .fetch_one(&pool)
        .await
        .expect("load finalized batch");

        assert_eq!(row.0, "failed");
        assert_eq!(row.1, "partial");
        assert_eq!(row.2, 3);
        assert_eq!(row.3, 1);
        assert_eq!(row.4, 1);
        assert_eq!(row.5, 1);
        assert_eq!(row.6, 1);
        assert!(!row.7.starts_with('{'));
        assert!(!row.7.contains("session"));

        let warning_message: String =
            sqlx::query_scalar("SELECT message FROM ingest_batch_warnings WHERE batch_id = ?")
                .bind(batch_id)
                .fetch_one(&pool)
                .await
                .expect("load warning");
        assert!(!warning_message.starts_with('{'));
        assert!(!warning_message.contains("api_hash"));
    }
}
```

- [ ] **Step 4: Run storage helper tests and confirm failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml ingest_provenance::tests::
```

Expected: compile fails because the helper types/functions do not exist.

- [ ] **Step 5: Implement storage helpers**

Above the test module in `src-tauri/src/ingest_provenance.rs`, add:

```rust
use sqlx::{Sqlite, SqliteConnection};

use crate::error::{AppError, AppResult};
use crate::sources::TelegramMessageIdentity;

pub(crate) const PROVENANCE_TEXT_MAX_LEN: usize = 512;

pub(crate) struct CreateTelegramTakeoutBatch {
    pub(crate) source_id: i64,
    pub(crate) account_id: i64,
    pub(crate) source_subtype: String,
}

pub(crate) struct IngestObservation {
    pub(crate) batch_id: i64,
    pub(crate) source_id: i64,
    pub(crate) item_id: Option<i64>,
    pub(crate) provider_item_kind: &'static str,
    pub(crate) provider_identity_kind: &'static str,
    pub(crate) provider_identity: String,
    pub(crate) outcome: &'static str,
    pub(crate) reason_code: Option<&'static str>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TerminalBatchStatus {
    Completed,
    Failed,
    Cancelled,
}

impl TerminalBatchStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
}

pub(crate) fn telegram_provider_identity(identity: &TelegramMessageIdentity) -> String {
    format!(
        "telegram:history_peer:{}:{}:message:{}",
        identity.history_peer_kind, identity.history_peer_id, identity.telegram_message_id
    )
}

pub(crate) fn sanitize_provenance_text(value: &str) -> String {
    let mut sanitized = value.replace('\0', " ");
    for marker in [
        "api_hash",
        "auth_key",
        "authorization",
        "cookie",
        "session",
        "secret",
    ] {
        sanitized = sanitized.replace(marker, "[redacted]");
        sanitized = sanitized.replace(&marker.to_ascii_uppercase(), "[redacted]");
    }
    let trimmed = sanitized.trim();
    let without_raw_shape = if trimmed.starts_with('{') || trimmed.starts_with('[') {
        "sanitized structured Telegram error"
    } else {
        trimmed
    };
    without_raw_shape.chars().take(PROVENANCE_TEXT_MAX_LEN).collect()
}

pub(crate) async fn create_telegram_takeout_batch(
    pool: &sqlx::Pool<Sqlite>,
    input: CreateTelegramTakeoutBatch,
) -> AppResult<i64> {
    let mut conn = pool.acquire().await.map_err(AppError::database)?;
    sqlx::query("BEGIN IMMEDIATE")
        .execute(&mut *conn)
        .await
        .map_err(AppError::database)?;

    let result: AppResult<i64> = async {
        let batch_id: i64 = sqlx::query_scalar(
            "INSERT INTO ingest_batches (source_id, provider, ingest_kind, status, completeness)
             VALUES (?, 'telegram', 'takeout', 'running', 'unknown')
             RETURNING id",
        )
        .bind(input.source_id)
        .fetch_one(&mut *conn)
        .await
        .map_err(AppError::database)?;

        sqlx::query(
            "INSERT INTO telegram_takeout_batches (batch_id, account_id, source_subtype)
             VALUES (?, ?, ?)",
        )
        .bind(batch_id)
        .bind(input.account_id)
        .bind(&input.source_subtype)
        .execute(&mut *conn)
        .await
        .map_err(AppError::database)?;

        Ok(batch_id)
    }
    .await;

    finish_manual_transaction(&mut conn, result).await
}

pub(crate) async fn update_takeout_resolved_peer(
    pool: &sqlx::Pool<Sqlite>,
    batch_id: i64,
    resolved_peer_kind: &str,
    resolved_peer_id: i64,
    history_peer_kind: &str,
    history_peer_id: i64,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE telegram_takeout_batches
         SET resolved_peer_kind = ?, resolved_peer_id = ?,
             history_peer_kind = ?, history_peer_id = ?, updated_at = CURRENT_TIMESTAMP
         WHERE batch_id = ?",
    )
    .bind(resolved_peer_kind)
    .bind(resolved_peer_id)
    .bind(history_peer_kind)
    .bind(history_peer_id)
    .bind(batch_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

pub(crate) async fn update_takeout_session_started(
    pool: &sqlx::Pool<Sqlite>,
    batch_id: i64,
    takeout_id: i64,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE telegram_takeout_batches
         SET takeout_id = ?, updated_at = CURRENT_TIMESTAMP
         WHERE batch_id = ?",
    )
    .bind(takeout_id)
    .bind(batch_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

pub(crate) async fn mark_takeout_export_dc_attempted(
    pool: &sqlx::Pool<Sqlite>,
    batch_id: i64,
    export_dc_id: i32,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE telegram_takeout_batches
         SET export_dc_id = ?, used_export_dc = 1, updated_at = CURRENT_TIMESTAMP
         WHERE batch_id = ?",
    )
    .bind(i64::from(export_dc_id))
    .bind(batch_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

pub(crate) async fn mark_takeout_export_dc_fallback(
    pool: &sqlx::Pool<Sqlite>,
    batch_id: i64,
    message: &str,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE telegram_takeout_batches
         SET fallback_used = 1, updated_at = CURRENT_TIMESTAMP
         WHERE batch_id = ?",
    )
    .bind(batch_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    record_ingest_batch_warning(pool, batch_id, "export_dc_fallback", message).await
}

pub(crate) async fn update_takeout_split_metadata(
    pool: &sqlx::Pool<Sqlite>,
    batch_id: i64,
    split_count: i64,
    selected_split_count: i64,
    message_count_estimate: Option<i64>,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE telegram_takeout_batches
         SET split_count = ?, selected_split_count = ?, message_count_estimate = ?,
             updated_at = CURRENT_TIMESTAMP
         WHERE batch_id = ?",
    )
    .bind(split_count)
    .bind(selected_split_count)
    .bind(message_count_estimate)
    .bind(batch_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

pub(crate) async fn mark_takeout_migrated_history_deferred(
    pool: &sqlx::Pool<Sqlite>,
    batch_id: i64,
    message: &str,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE telegram_takeout_batches
         SET migrated_history_detected = 1,
             migrated_history_imported = 0,
             history_scope = CASE
               WHEN only_my_messages = 1 THEN 'mixed_partial'
               ELSE 'current_history_with_migrated_deferred'
             END,
             updated_at = CURRENT_TIMESTAMP
         WHERE batch_id = ?",
    )
    .bind(batch_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    record_ingest_batch_warning(pool, batch_id, "migrated_history_deferred", message).await
}

pub(crate) async fn mark_takeout_only_my_messages_fallback(
    pool: &sqlx::Pool<Sqlite>,
    batch_id: i64,
    message: &str,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE telegram_takeout_batches
         SET only_my_messages = 1,
             history_scope = CASE
               WHEN migrated_history_detected = 1 THEN 'mixed_partial'
               ELSE 'partial_private_history'
             END,
             updated_at = CURRENT_TIMESTAMP
         WHERE batch_id = ?",
    )
    .bind(batch_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    record_ingest_batch_warning(pool, batch_id, "only_my_messages_fallback", message).await
}

pub(crate) async fn update_takeout_max_message_id(
    pool: &sqlx::Pool<Sqlite>,
    batch_id: i64,
    max_message_id: i64,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE telegram_takeout_batches
         SET max_message_id = MAX(COALESCE(max_message_id, 0), ?),
             updated_at = CURRENT_TIMESTAMP
         WHERE batch_id = ?",
    )
    .bind(max_message_id)
    .bind(batch_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

pub(crate) async fn record_ingest_batch_warning(
    pool: &sqlx::Pool<Sqlite>,
    batch_id: i64,
    code: &str,
    message: &str,
) -> AppResult<()> {
    let message = sanitize_provenance_text(message);
    sqlx::query(
        "INSERT INTO ingest_batch_warnings (batch_id, code, message)
         VALUES (?, ?, ?)",
    )
    .bind(batch_id)
    .bind(code)
    .bind(message)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

pub(crate) async fn record_ingest_observation(
    pool: &sqlx::Pool<Sqlite>,
    observation: IngestObservation,
) -> AppResult<()> {
    let mut conn = pool.acquire().await.map_err(AppError::database)?;
    record_ingest_observation_on_connection(&mut conn, observation).await
}

pub(crate) async fn record_ingest_observation_on_connection(
    conn: &mut SqliteConnection,
    observation: IngestObservation,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO ingest_item_observations (
            batch_id, source_id, item_id, provider_item_kind, provider_identity_kind,
            provider_identity, provider_identity_version, outcome, reason_code
         ) VALUES (?, ?, ?, ?, ?, ?, 1, ?, ?)",
    )
    .bind(observation.batch_id)
    .bind(observation.source_id)
    .bind(observation.item_id)
    .bind(observation.provider_item_kind)
    .bind(observation.provider_identity_kind)
    .bind(observation.provider_identity)
    .bind(observation.outcome)
    .bind(observation.reason_code)
    .execute(conn)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

pub(crate) async fn finalize_ingest_batch(
    pool: &sqlx::Pool<Sqlite>,
    batch_id: i64,
    status: TerminalBatchStatus,
    terminal_error: Option<&str>,
) -> AppResult<()> {
    let mut conn = pool.acquire().await.map_err(AppError::database)?;
    sqlx::query("BEGIN IMMEDIATE")
        .execute(&mut *conn)
        .await
        .map_err(AppError::database)?;

    let result: AppResult<()> = async {
        let counts: (i64, i64, i64, i64) = sqlx::query_as(
            "SELECT
                COUNT(*),
                COALESCE(SUM(CASE WHEN outcome = 'inserted' THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN outcome = 'duplicate_observed' THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN outcome = 'skipped' THEN 1 ELSE 0 END), 0)
             FROM ingest_item_observations
             WHERE batch_id = ?",
        )
        .bind(batch_id)
        .fetch_one(&mut *conn)
        .await
        .map_err(AppError::database)?;

        let warning_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM ingest_batch_warnings WHERE batch_id = ?",
        )
        .bind(batch_id)
        .fetch_one(&mut *conn)
        .await
        .map_err(AppError::database)?;

        let detail: (i64, i64, String) = sqlx::query_as(
            "SELECT only_my_messages, migrated_history_detected, history_scope
             FROM telegram_takeout_batches WHERE batch_id = ?",
        )
        .bind(batch_id)
        .fetch_one(&mut *conn)
        .await
        .map_err(AppError::database)?;

        let completeness = classify_completeness(status, counts.0, detail);
        let terminal_error = terminal_error.map(sanitize_provenance_text);
        sqlx::query(
            "UPDATE ingest_batches
             SET status = ?, completeness = ?, finished_at = CURRENT_TIMESTAMP,
                 item_observed_count = ?, item_inserted_count = ?,
                 item_duplicate_count = ?, item_skipped_count = ?,
                 warning_count = ?, terminal_error = ?, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?",
        )
        .bind(status.as_str())
        .bind(completeness)
        .bind(counts.0)
        .bind(counts.1)
        .bind(counts.2)
        .bind(counts.3)
        .bind(warning_count)
        .bind(terminal_error)
        .bind(batch_id)
        .execute(&mut *conn)
        .await
        .map_err(AppError::database)?;

        Ok(())
    }
    .await;

    finish_manual_transaction(&mut conn, result).await
}

fn classify_completeness(
    status: TerminalBatchStatus,
    observation_count: i64,
    detail: (i64, i64, String),
) -> &'static str {
    let (only_my_messages, migrated_history_detected, history_scope) = detail;
    match status {
        TerminalBatchStatus::Completed
            if only_my_messages == 0
                && migrated_history_detected == 0
                && history_scope != "mixed_partial" =>
        {
            "complete"
        }
        TerminalBatchStatus::Completed => "partial",
        TerminalBatchStatus::Failed | TerminalBatchStatus::Cancelled if observation_count > 0 => {
            "partial"
        }
        TerminalBatchStatus::Failed | TerminalBatchStatus::Cancelled => "unknown",
    }
}

async fn finish_manual_transaction<T>(
    conn: &mut sqlx::pool::PoolConnection<Sqlite>,
    result: AppResult<T>,
) -> AppResult<T> {
    match result {
        Ok(value) => {
            sqlx::query("COMMIT")
                .execute(&mut **conn)
                .await
                .map_err(AppError::database)?;
            Ok(value)
        }
        Err(error) => {
            let _ = sqlx::query("ROLLBACK").execute(&mut **conn).await;
            Err(error)
        }
    }
}
```

- [ ] **Step 6: Run storage helper tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml ingest_provenance::tests:: sources::test_support::tests::source_fixture_creates_expected_tables
```

Expected: tests pass after small compiler fixes for lifetimes or imports.

- [ ] **Step 7: Commit storage helpers**

Run:

```powershell
git add src-tauri/src/lib.rs src-tauri/src/ingest_provenance.rs src-tauri/src/sources/test_support.rs
git commit -m "feat: add ingest provenance storage helpers"
```

Expected: commit succeeds with storage helpers and tests.

---

### Task 3: Telegram Item Insert Outcomes And Observations

**Files:**
- Modify: `src-tauri/src/sources/items.rs`
- Modify: `src-tauri/src/sources/mod.rs`
- Test: `src-tauri/src/sources/items.rs`

- [x] **Step 1: Write failing item outcome/observation tests**

In `src-tauri/src/sources/items.rs`, update the test imports:

```rust
use crate::sources::test_support::{
    create_ingest_provenance_tables, create_item_identity_indexes,
    memory_pool_with_source_items_and_topics,
};
```

Add this test:

```rust
#[tokio::test]
async fn telegram_insert_outcome_returns_item_ids_for_insert_and_duplicate() {
    let pool = memory_pool_with_source_items_and_topics().await;
    seed_item_source(&pool, 1).await;
    let identity = TelegramMessageIdentity {
        history_peer_kind: "channel".to_string(),
        history_peer_id: 12345,
        telegram_message_id: 42,
        migration_domain: None,
        is_migrated_history: false,
    };

    let inserted = insert_telegram_source_item_outcome(
        &pool,
        1,
        identity.clone(),
        telegram_insert("42", "first payload"),
    )
    .await
    .expect("insert first");
    let first_id = match inserted {
        TelegramItemInsertOutcome::Inserted { item_id } => item_id,
        other => panic!("expected inserted outcome, got {other:?}"),
    };

    let duplicate = insert_telegram_source_item_outcome(
        &pool,
        1,
        identity,
        telegram_insert("42", "second payload"),
    )
    .await
    .expect("observe duplicate");
    assert_eq!(
        duplicate,
        TelegramItemInsertOutcome::DuplicateObserved { item_id: first_id }
    );
}
```

Add this test:

```rust
#[tokio::test]
async fn telegram_insert_with_observation_records_insert_duplicate_and_skipped_rows() {
    let pool = memory_pool_with_source_items_and_topics().await;
    create_ingest_provenance_tables(&pool).await;
    seed_item_source(&pool, 1).await;
    let batch_id = crate::ingest_provenance::create_telegram_takeout_batch(
        &pool,
        crate::ingest_provenance::CreateTelegramTakeoutBatch {
            source_id: 1,
            account_id: 10,
            source_subtype: "supergroup".to_string(),
        },
    )
    .await
    .expect("create batch");
    let identity = TelegramMessageIdentity {
        history_peer_kind: "channel".to_string(),
        history_peer_id: 12345,
        telegram_message_id: 42,
        migration_domain: None,
        is_migrated_history: false,
    };

    let inserted = insert_telegram_source_item_with_observation(
        &pool,
        batch_id,
        1,
        identity.clone(),
        telegram_insert("42", "first payload"),
    )
    .await
    .expect("insert with observation");
    let item_id = match inserted {
        TelegramItemInsertOutcome::Inserted { item_id } => item_id,
        other => panic!("expected insert, got {other:?}"),
    };

    let duplicate = insert_telegram_source_item_with_observation(
        &pool,
        batch_id,
        1,
        identity.clone(),
        telegram_insert("42", "duplicate payload"),
    )
    .await
    .expect("duplicate with observation");
    assert_eq!(
        duplicate,
        TelegramItemInsertOutcome::DuplicateObserved { item_id }
    );

    let empty_item = SourceItemInsert {
        payload: ExtractedItemPayload {
            content: None,
            content_kind: CONTENT_KIND_TEXT_ONLY,
            media: None,
        },
        ..telegram_insert("43", "")
    };
    let skipped_identity = TelegramMessageIdentity {
        telegram_message_id: 43,
        ..identity
    };
    let skipped = insert_telegram_source_item_with_observation(
        &pool,
        batch_id,
        1,
        skipped_identity,
        empty_item,
    )
    .await
    .expect("skipped with observation");
    assert_eq!(
        skipped,
        TelegramItemInsertOutcome::Skipped {
            reason_code: "empty_payload"
        }
    );

    let rows: Vec<(String, Option<i64>, String, Option<String>)> = sqlx::query_as(
        "SELECT outcome, item_id, provider_identity, reason_code
         FROM ingest_item_observations
         WHERE batch_id = ?
         ORDER BY id",
    )
    .bind(batch_id)
    .fetch_all(&pool)
    .await
    .expect("load observations");

    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].0, "inserted");
    assert_eq!(rows[0].1, Some(item_id));
    assert_eq!(
        rows[0].2,
        "telegram:history_peer:channel:12345:message:42"
    );
    assert_eq!(rows[1].0, "duplicate_observed");
    assert_eq!(rows[1].1, Some(item_id));
    assert_eq!(rows[2].0, "skipped");
    assert_eq!(rows[2].1, None);
    assert_eq!(rows[2].3.as_deref(), Some("empty_payload"));
}
```

- [x] **Step 2: Run item tests and confirm failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::items::tests::telegram_insert_outcome_returns_item_ids_for_insert_and_duplicate sources::items::tests::telegram_insert_with_observation_records_insert_duplicate_and_skipped_rows
```

Expected: compile fails because the outcome enum and observation wrapper do not exist.

- [x] **Step 3: Add outcome enum and bool compatibility wrapper**

In `src-tauri/src/sources/items.rs`, add near `PreparedSourceItem`:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TelegramItemInsertOutcome {
    Inserted { item_id: i64 },
    DuplicateObserved { item_id: i64 },
    Skipped { reason_code: &'static str },
}

impl TelegramItemInsertOutcome {
    pub(crate) fn is_inserted(self) -> bool {
        matches!(self, Self::Inserted { .. })
    }

    pub(crate) fn observation_parts(self) -> (&'static str, Option<i64>, Option<&'static str>) {
        match self {
            Self::Inserted { item_id } => ("inserted", Some(item_id), None),
            Self::DuplicateObserved { item_id } => ("duplicate_observed", Some(item_id), None),
            Self::Skipped { reason_code } => ("skipped", None, Some(reason_code)),
        }
    }
}
```

- [x] **Step 4: Split Telegram insert into an open-transaction helper**

Keep the public `insert_telegram_source_item(...) -> AppResult<bool>` signature and rewrite it as:

```rust
pub(crate) async fn insert_telegram_source_item(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source_id: i64,
    identity: TelegramMessageIdentity,
    item: SourceItemInsert,
) -> AppResult<bool> {
    Ok(insert_telegram_source_item_outcome(pool, source_id, identity, item)
        .await?
        .is_inserted())
}
```

Add:

```rust
pub(crate) async fn insert_telegram_source_item_outcome(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    source_id: i64,
    identity: TelegramMessageIdentity,
    item: SourceItemInsert,
) -> AppResult<TelegramItemInsertOutcome> {
    let mut conn = pool.acquire().await.map_err(AppError::database)?;
    sqlx::query("BEGIN IMMEDIATE")
        .execute(&mut *conn)
        .await
        .map_err(AppError::database)?;

    let result =
        insert_telegram_source_item_on_connection(&mut conn, source_id, identity, item).await;

    match result {
        Ok(outcome) => {
            sqlx::query("COMMIT")
                .execute(&mut *conn)
                .await
                .map_err(AppError::database)?;
            Ok(outcome)
        }
        Err(error) => {
            let _ = sqlx::query("ROLLBACK").execute(&mut *conn).await;
            if error.kind == crate::error::AppErrorKind::Conflict
                || error.message.contains("telegram_messages")
            {
                return Ok(TelegramItemInsertOutcome::Skipped {
                    reason_code: "conflict_without_item_id",
                });
            }
            Err(error)
        }
    }
}
```

Move the existing body of `insert_telegram_source_item` into:

```rust
async fn insert_telegram_source_item_on_connection(
    conn: &mut sqlx::SqliteConnection,
    source_id: i64,
    identity: TelegramMessageIdentity,
    item: SourceItemInsert,
) -> AppResult<TelegramItemInsertOutcome> {
    identity.validate()?;
    if item.item_kind != ITEM_KIND_TELEGRAM_MESSAGE {
        return Err(AppError::validation(format!(
            "insert_telegram_source_item requires item_kind '{ITEM_KIND_TELEGRAM_MESSAGE}'"
        )));
    }

    let Some(prepared) = prepare_source_item(&item)? else {
        return Ok(TelegramItemInsertOutcome::Skipped {
            reason_code: "empty_payload",
        });
    };

    let existing: Option<i64> = sqlx::query_scalar(
        "SELECT item_id
         FROM telegram_messages
         WHERE source_id = ?
           AND history_peer_kind = ?
           AND history_peer_id = ?
           AND telegram_message_id = ?",
    )
    .bind(source_id)
    .bind(&identity.history_peer_kind)
    .bind(identity.history_peer_id)
    .bind(identity.telegram_message_id)
    .fetch_optional(&mut *conn)
    .await
    .map_err(AppError::database)?;
    if let Some(item_id) = existing {
        return Ok(TelegramItemInsertOutcome::DuplicateObserved { item_id });
    }

    let item_id: i64 = sqlx::query_scalar(
        "INSERT INTO items (
            source_id, external_id, item_kind, author, published_at, ingested_at,
            content_zstd, raw_data_zstd, content_kind, has_media, media_kind,
            media_metadata_zstd, reply_to_msg_id, reply_to_peer_kind,
            reply_to_peer_id, reply_to_top_id, reaction_count
         )
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         RETURNING id",
    )
    .bind(source_id)
    .bind(identity.telegram_message_id.to_string())
    .bind(&item.item_kind)
    .bind(&item.author)
    .bind(item.published_at)
    .bind(now_secs())
    .bind(prepared.content_zstd)
    .bind(prepared.raw_data_zstd)
    .bind(prepared.content_kind)
    .bind(prepared.has_media)
    .bind(prepared.media_kind)
    .bind(prepared.media_metadata_zstd)
    .bind(item.telegram_context.reply_to_msg_id)
    .bind(&item.telegram_context.reply_to_peer_kind)
    .bind(&item.telegram_context.reply_to_peer_id)
    .bind(item.telegram_context.reply_to_top_id)
    .bind(item.telegram_context.reaction_count)
    .fetch_one(&mut *conn)
    .await
    .map_err(AppError::database)?;

    sqlx::query(
        "INSERT INTO telegram_messages (
            item_id, source_id, history_peer_kind, history_peer_id,
            telegram_message_id, migration_domain, is_migrated_history,
            reply_to_msg_id, reply_to_peer_kind, reply_to_peer_id,
            reply_to_top_id, reaction_count
         )
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(item_id)
    .bind(source_id)
    .bind(&identity.history_peer_kind)
    .bind(identity.history_peer_id)
    .bind(identity.telegram_message_id)
    .bind(&identity.migration_domain)
    .bind(i64::from(identity.is_migrated_history))
    .bind(item.telegram_context.reply_to_msg_id)
    .bind(&item.telegram_context.reply_to_peer_kind)
    .bind(
        item.telegram_context
            .reply_to_peer_id
            .as_deref()
            .and_then(|value| value.parse::<i64>().ok()),
    )
    .bind(item.telegram_context.reply_to_top_id)
    .bind(item.telegram_context.reaction_count)
    .execute(&mut *conn)
    .await
    .map_err(AppError::database)?;

    crate::topic_memberships::resolve_scoped_topic_memberships_on_connection(
        conn,
        source_id,
        &[item_id],
        now_secs(),
    )
    .await?;

    Ok(TelegramItemInsertOutcome::Inserted { item_id })
}
```

- [x] **Step 5: Add the provenance-aware wrapper**

In `src-tauri/src/sources/items.rs`, add:

```rust
pub(crate) async fn insert_telegram_source_item_with_observation(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    batch_id: i64,
    source_id: i64,
    identity: TelegramMessageIdentity,
    item: SourceItemInsert,
) -> AppResult<TelegramItemInsertOutcome> {
    let provider_identity = crate::ingest_provenance::telegram_provider_identity(&identity);
    let mut conn = pool.acquire().await.map_err(AppError::database)?;
    sqlx::query("BEGIN IMMEDIATE")
        .execute(&mut *conn)
        .await
        .map_err(AppError::database)?;

    let result: AppResult<TelegramItemInsertOutcome> = async {
        let outcome =
            insert_telegram_source_item_on_connection(&mut conn, source_id, identity, item).await?;
        let (outcome_name, item_id, reason_code) = outcome.observation_parts();
        crate::ingest_provenance::record_ingest_observation_on_connection(
            &mut conn,
            crate::ingest_provenance::IngestObservation {
                batch_id,
                source_id,
                item_id,
                provider_item_kind: ITEM_KIND_TELEGRAM_MESSAGE,
                provider_identity_kind: "telegram_message",
                provider_identity,
                outcome: outcome_name,
                reason_code,
            },
        )
        .await?;
        Ok(outcome)
    }
    .await;

    match result {
        Ok(outcome) => {
            sqlx::query("COMMIT")
                .execute(&mut *conn)
                .await
                .map_err(AppError::database)?;
            Ok(outcome)
        }
        Err(error) => {
            let _ = sqlx::query("ROLLBACK").execute(&mut *conn).await;
            Err(error)
        }
    }
}
```

- [x] **Step 6: Export the new crate-private helpers**

In `src-tauri/src/sources/mod.rs`, extend the crate-private export:

```rust
pub(crate) use items::{
    insert_source_item, insert_telegram_source_item, insert_telegram_source_item_outcome,
    insert_telegram_source_item_with_observation, upsert_youtube_comment_item,
    upsert_youtube_transcript_item, SourceItemInsert, TelegramItemContext,
    TelegramItemInsertOutcome,
};
```

- [x] **Step 7: Update existing bool tests for outcome internals**

Existing tests that call `insert_telegram_source_item` should still pass because the bool wrapper remains. Do not change normal sync call sites in this task.

- [x] **Step 8: Run item tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml sources::items::tests::
```

Expected: all source item tests pass.

- [x] **Step 9: Commit item outcome work**

Run:

```powershell
git add src-tauri/src/sources/items.rs src-tauri/src/sources/mod.rs
git commit -m "feat: record telegram item insert outcomes"
```

Expected: commit succeeds with normal sync behavior preserved.

---

### Task 4: Start Locking, Batch Creation, And Job Correlation

**Files:**
- Modify: `src-tauri/src/takeout_import/state.rs`
- Modify: `src-tauri/src/takeout_import/mod.rs`
- Test: `src-tauri/src/takeout_import/state.rs`
- Test: `src-tauri/src/takeout_import/mod.rs`

- [x] **Step 1: Add `batch_id` to job state tests first**

In `src-tauri/src/takeout_import/state.rs`, update existing tests to call `create_job(7, 1, 100)` and assert:

```rust
assert_eq!(first.batch_id, 100);
```

Also update the second created job in `job_state_can_cancel_and_finish_job`:

```rust
let next = state.create_job(7, 1, 101).await.expect("source released");
assert_eq!(next.job_id, "takeout-2");
assert_eq!(next.batch_id, 101);
```

- [x] **Step 2: Run state tests and confirm failure**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml takeout_import::state::tests::
```

Expected: compile fails because `batch_id` and the new `create_job` argument do not exist.

- [x] **Step 3: Implement job correlation**

In `TakeoutImportJobRecord`, add:

```rust
pub batch_id: i64,
```

Change `create_job` signature:

```rust
pub(crate) async fn create_job(
    &self,
    source_id: i64,
    account_id: i64,
    batch_id: i64,
) -> AppResult<TakeoutImportJobRecord>
```

Set the record field:

```rust
batch_id,
```

Update all call sites to pass a batch id.

- [x] **Step 4: Extract start-record creation after lock acquisition**

In `src-tauri/src/takeout_import/mod.rs`, import:

```rust
use crate::ingest_provenance::{create_telegram_takeout_batch, CreateTelegramTakeoutBatch};
use crate::source_ingest::SourceIngestGuard;
```

Add this helper near `start_takeout_source_import`:

```rust
async fn create_locked_takeout_start_records(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    ingest_locks: &SourceIngestLocks,
    state: &TakeoutImportState,
    source_id: i64,
    account_id: i64,
    source_subtype: String,
) -> AppResult<(TakeoutImportJobRecord, SourceIngestGuard)> {
    let ingest_guard = ingest_locks
        .try_acquire(source_id, SourceIngestKind::TakeoutImport)
        .await?;
    let batch_id = create_telegram_takeout_batch(
        pool,
        CreateTelegramTakeoutBatch {
            source_id,
            account_id,
            source_subtype,
        },
    )
    .await?;
    let record = state.create_job(source_id, account_id, batch_id).await?;
    Ok((record, ingest_guard))
}
```

- [x] **Step 5: Move source lock acquisition out of the background task**

In `start_takeout_source_import`, after loading the source/account/subtype, add:

```rust
let telegram_source_subtype = load_takeout_source_subtype(&pool, source.id).await?;
let ingest_locks = handle.state::<SourceIngestLocks>();
let (record, ingest_guard) = create_locked_takeout_start_records(
    &pool,
    &ingest_locks,
    &state,
    source_id,
    account_id,
    telegram_source_subtype,
)
.await?;
```

Spawn with the guard:

```rust
tauri::async_runtime::spawn(async move {
    run_takeout_import_job(task_handle, job_id, ingest_guard).await;
});
```

Change the task signature:

```rust
async fn run_takeout_import_job(
    handle: AppHandle,
    job_id: String,
    ingest_guard: SourceIngestGuard,
)
```

Remove the lock acquisition block from inside `run_takeout_import_job`. Keep `drop(ingest_guard);` at the end and on the early cancellation path so the lifetime is explicit.

- [x] **Step 6: Add lock-conflict side-effect test**

In `src-tauri/src/takeout_import/mod.rs` tests, import:

```rust
use crate::takeout_import::state::TakeoutImportState;
use crate::source_ingest::{SourceIngestKind, SourceIngestLocks};
use crate::sources::test_support::{
    create_ingest_provenance_tables, memory_pool_with_source_items_and_topics,
};
```

Add:

```rust
#[tokio::test]
async fn locked_start_conflict_creates_no_provenance_rows() {
    let pool = memory_pool_with_source_items_and_topics().await;
    create_ingest_provenance_tables(&pool).await;
    seed_item_source(&pool, 1).await;
    let locks = SourceIngestLocks::new();
    let _existing = locks
        .try_acquire(1, SourceIngestKind::Sync)
        .await
        .expect("hold existing lock");
    let state = TakeoutImportState::new();

    let error = create_locked_takeout_start_records(
        &pool,
        &locks,
        &state,
        1,
        10,
        "supergroup".to_string(),
    )
    .await
    .expect_err("conflicting lock should reject start");

    assert_eq!(error.kind, crate::error::AppErrorKind::Conflict);
    for table in [
        "ingest_batches",
        "telegram_takeout_batches",
        "ingest_item_observations",
        "ingest_batch_warnings",
    ] {
        let query = format!("SELECT COUNT(*) FROM {table}");
        let count: i64 = sqlx::query_scalar(&query)
            .fetch_one(&pool)
            .await
            .unwrap_or_else(|err| panic!("count {table}: {err}"));
        assert_eq!(count, 0, "unexpected rows in {table}");
    }
}
```

- [x] **Step 7: Add concurrent start helper test**

Add:

```rust
#[tokio::test]
async fn locked_start_allows_only_one_batch_for_same_source() {
    let pool = memory_pool_with_source_items_and_topics().await;
    create_ingest_provenance_tables(&pool).await;
    seed_item_source(&pool, 1).await;
    let locks = SourceIngestLocks::new();
    let state = TakeoutImportState::new();

    let first = create_locked_takeout_start_records(
        &pool,
        &locks,
        &state,
        1,
        10,
        "supergroup".to_string(),
    )
    .await
    .expect("first start");

    let second = create_locked_takeout_start_records(
        &pool,
        &locks,
        &state,
        1,
        10,
        "supergroup".to_string(),
    )
    .await;

    assert!(second.is_err());
    let batch_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ingest_batches")
        .fetch_one(&pool)
        .await
        .expect("count batches");
    assert_eq!(batch_count, 1);

    drop(first);
}
```

- [x] **Step 8: Run start locking tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml takeout_import::state::tests:: takeout_import::tests::locked_start_
```

Expected: tests pass after imports and call sites are updated.

- [x] **Step 9: Commit start locking and batch correlation**

Run:

```powershell
git add src-tauri/src/takeout_import/state.rs src-tauri/src/takeout_import/mod.rs
git commit -m "feat: create takeout batches after source lock"
```

Expected: commit succeeds.

---

### Task 5: Takeout Detail, Warnings, Observations, And Terminal Status

**Files:**
- Modify: `src-tauri/src/takeout_import/export_dc.rs`
- Modify: `src-tauri/src/takeout_import/mod.rs`
- Test: `src-tauri/src/takeout_import/export_dc.rs`
- Test: `src-tauri/src/takeout_import/mod.rs`

- [x] **Step 1: Add export-DC fallback signal tests**

In `src-tauri/src/takeout_import/export_dc.rs`, extend tests with a small unit around a new helper:

```rust
#[test]
fn export_dc_attempt_state_detects_first_fallback_transition() {
    let mut state = ExportDcAttemptState::new();
    assert!(state.mark_attempted(40002));
    assert!(!state.mark_attempted(40002));
    assert!(state.mark_fallback("fallback message".to_string()).is_some());
    assert!(state.mark_fallback("second fallback".to_string()).is_none());
}
```

- [x] **Step 2: Implement export-DC attempt state**

In `export_dc.rs`, add:

```rust
#[derive(Default)]
pub(crate) struct ExportDcAttemptState {
    attempted_export_dc_id: Option<i32>,
    fallback_recorded: bool,
}

impl ExportDcAttemptState {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn mark_attempted(&mut self, export_dc_id: i32) -> bool {
        if self.attempted_export_dc_id == Some(export_dc_id) {
            return false;
        }
        self.attempted_export_dc_id = Some(export_dc_id);
        true
    }

    pub(crate) fn mark_fallback(&mut self, message: String) -> Option<String> {
        if self.fallback_recorded {
            return None;
        }
        self.fallback_recorded = true;
        Some(message)
    }
}
```

Keep `export_dc_invoke` signature unchanged; the Takeout flow will compare `fallback_used` before/after each call and use this state to avoid duplicate durable fallback warnings for one batch.

- [x] **Step 3: Add Takeout helper functions for durable warning/detail updates**

In `takeout_import/mod.rs`, import:

```rust
use crate::ingest_provenance::{
    finalize_ingest_batch, mark_takeout_export_dc_attempted,
    mark_takeout_export_dc_fallback, mark_takeout_migrated_history_deferred,
    mark_takeout_only_my_messages_fallback, record_ingest_batch_warning,
    update_takeout_max_message_id, update_takeout_resolved_peer,
    update_takeout_session_started, update_takeout_split_metadata,
    TerminalBatchStatus,
};
use export_dc::ExportDcAttemptState;
use grammers_session::types::{PeerKind, PeerRef};
```

Add:

```rust
async fn record_export_dc_attempt_if_needed(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    batch_id: i64,
    alias: &ExportDcAlias,
    attempts: &mut ExportDcAttemptState,
) -> AppResult<()> {
    if attempts.mark_attempted(alias.export_dc_id) {
        mark_takeout_export_dc_attempted(pool, batch_id, alias.export_dc_id).await?;
    }
    Ok(())
}

async fn record_export_dc_fallback_if_needed(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    batch_id: i64,
    warnings: &[String],
    fallback_before: bool,
    fallback_after: bool,
    attempts: &mut ExportDcAttemptState,
) -> AppResult<()> {
    if !fallback_before && fallback_after {
        let message = warnings
            .last()
            .cloned()
            .unwrap_or_else(|| "Export DC fallback was used.".to_string());
        if let Some(message) = attempts.mark_fallback(message) {
            mark_takeout_export_dc_fallback(pool, batch_id, &message).await?;
        }
    }
    Ok(())
}

fn peer_ref_identity(peer: PeerRef) -> (&'static str, i64) {
    let kind = match peer.id.kind() {
        PeerKind::User | PeerKind::UserSelf => "user",
        PeerKind::Chat => "chat",
        PeerKind::Channel => "channel",
    };
    (kind, peer.id.bare_id())
}
```

- [x] **Step 4: Thread `batch_id` and export attempt state through Takeout flow**

Add `batch_id: i64` to:

```rust
run_takeout_source_import(handle, job_id, batch_id)
run_started_takeout_source_import(..., batch_id, ...)
run_started_takeout_source_import_inner(..., batch_id, ...)
import_takeout_history_ranges(..., batch_id, ...)
import_takeout_history_pages(..., batch_id, ...)
```

In `run_takeout_import_job`, load `batch_id` from the job record:

```rust
let batch_id = running_record.batch_id;
match run_takeout_source_import(&handle, &job_id, batch_id).await {
```

In `run_takeout_source_import`, create:

```rust
let mut export_attempts = ExportDcAttemptState::new();
```

Before each direct `export_dc_invoke` or `finish_takeout_session` in the Takeout flow, call `record_export_dc_attempt_if_needed(...)`. After each call, compare fallback state:

```rust
let fallback_before = fallback_used;
record_export_dc_attempt_if_needed(&pool, batch_id, &alias, &mut export_attempts).await?;
let takeout = export_dc_invoke(
    &client,
    &alias,
    &init_request,
    &mut warnings,
    &mut fallback_used,
)
.await?;
record_export_dc_fallback_if_needed(
    &pool,
    batch_id,
    &warnings,
    fallback_before,
    fallback_used,
    &mut export_attempts,
)
.await?;
```

Use the same pattern around `finish_takeout_session(...)`.

- [x] **Step 5: Write resolved peer and session details**

After `resolve_and_refresh_peer(...)`:

```rust
let (resolved_peer_kind, resolved_peer_id) = peer_ref_identity(resolved_peer.peer);
update_takeout_resolved_peer(
    &pool,
    batch_id,
    resolved_peer_kind,
    resolved_peer_id,
    resolved_peer_kind,
    resolved_peer_id,
)
.await?;
```

After `takeout_id` is known:

```rust
update_takeout_session_started(&pool, batch_id, takeout_id).await?;
```

- [x] **Step 6: Persist split/count metadata and scope warnings**

Immediately after `let split_ranges = ...`:

```rust
let split_count = split_ranges.len() as i64;
let selected_ranges = select_history_splits(telegram_source_subtype, split_ranges)?;
let selected_split_count = selected_ranges.len() as i64;
```

After count probes:

```rust
update_takeout_split_metadata(
    pool,
    batch_id,
    split_count,
    selected_split_count,
    Some(total),
)
.await?;
```

Change `detect_supergroup_migration(...)` to return `AppResult<bool>` and return `Ok(true)` when `migrated_from_chat_id` is present. At the call site:

```rust
let migrated_detected = detect_supergroup_migration(...).await?;
if migrated_detected {
    mark_takeout_migrated_history_deferred(
        pool,
        batch_id,
        "Supergroup migrated history detected; current foundation import defers migrated history.",
    )
    .await?;
}
```

When `takeout_history_count_probe` or `takeout_history_page_response` flips `only_my_messages` from `false` to `true`, call:

```rust
mark_takeout_only_my_messages_fallback(
    &pool,
    batch_id,
    "Channel history is private; importing only messages visible through from_id=self fallback.",
)
.await?;
```

- [x] **Step 7: Replace Takeout item insertion with observation wrapper**

In `import_takeout_history_pages`, replace:

```rust
if insert_telegram_source_item(&pool, source.id, identity, item).await? {
    imported.inserted += 1;
} else {
    imported.skipped += 1;
}
```

with:

```rust
match crate::sources::insert_telegram_source_item_with_observation(
    &pool,
    batch_id,
    source.id,
    identity,
    item,
)
.await?
{
    crate::sources::TelegramItemInsertOutcome::Inserted { .. } => {
        imported.inserted += 1;
    }
    crate::sources::TelegramItemInsertOutcome::DuplicateObserved { .. }
    | crate::sources::TelegramItemInsertOutcome::Skipped { .. } => {
        imported.skipped += 1;
    }
}
```

After updating `imported.max_message_id`, persist diagnostic max id cheaply:

```rust
update_takeout_max_message_id(&pool, batch_id, imported.max_message_id).await?;
```

- [x] **Step 8: Add terminal batch finalization**

On success, before emitting the final job record and after `finalize_sync(...)`, call:

```rust
finalize_ingest_batch(&pool, batch_id, TerminalBatchStatus::Completed, None).await?;
```

On failure:

```rust
let _ = finalize_ingest_batch(
    &pool,
    batch_id,
    TerminalBatchStatus::Failed,
    Some(&error.to_string()),
)
.await;
```

On cancellation:

```rust
let _ = finalize_ingest_batch(
    &pool,
    batch_id,
    TerminalBatchStatus::Cancelled,
    None,
)
.await;
```

If `finish_takeout_session(..., false, ...)` fails in the failure/cancel path, add:

```rust
let _ = record_ingest_batch_warning(
    &pool,
    batch_id,
    "finish_takeout_failed",
    &format!("Failed to finish Takeout session after terminal error: {finish_error}"),
)
.await;
```

Ignore the result of terminal provenance updates only when the job is already failing; keep a best-effort in-memory job error if the provenance update itself fails during the success path.

- [x] **Step 9: Add pure completeness tests for terminal helper behavior**

In `src-tauri/src/ingest_provenance.rs`, add tests for zero-message success and mixed partial:

```rust
#[tokio::test]
async fn completed_zero_observation_batch_is_complete_without_partial_flags() {
    let pool = memory_pool_with_source_items_and_topics().await;
    create_ingest_provenance_tables(&pool).await;
    seed_source(&pool).await;
    let batch_id = create_telegram_takeout_batch(
        &pool,
        CreateTelegramTakeoutBatch {
            source_id: 1,
            account_id: 10,
            source_subtype: "supergroup".to_string(),
        },
    )
    .await
    .expect("create batch");

    finalize_ingest_batch(&pool, batch_id, TerminalBatchStatus::Completed, None)
        .await
        .expect("finalize complete empty batch");

    let completeness: String =
        sqlx::query_scalar("SELECT completeness FROM ingest_batches WHERE id = ?")
            .bind(batch_id)
            .fetch_one(&pool)
            .await
            .expect("load completeness");
    assert_eq!(completeness, "complete");
}

#[tokio::test]
async fn mixed_partial_scope_finalizes_as_partial() {
    let pool = memory_pool_with_source_items_and_topics().await;
    create_ingest_provenance_tables(&pool).await;
    seed_source(&pool).await;
    let batch_id = create_telegram_takeout_batch(
        &pool,
        CreateTelegramTakeoutBatch {
            source_id: 1,
            account_id: 10,
            source_subtype: "supergroup".to_string(),
        },
    )
    .await
    .expect("create batch");
    mark_takeout_only_my_messages_fallback(&pool, batch_id, "private history")
        .await
        .expect("mark private fallback");
    mark_takeout_migrated_history_deferred(&pool, batch_id, "migrated deferred")
        .await
        .expect("mark migrated deferred");

    finalize_ingest_batch(&pool, batch_id, TerminalBatchStatus::Completed, None)
        .await
        .expect("finalize partial batch");

    let row: (String, String) = sqlx::query_as(
        "SELECT b.completeness, t.history_scope
         FROM ingest_batches b
         JOIN telegram_takeout_batches t ON t.batch_id = b.id
         WHERE b.id = ?",
    )
    .bind(batch_id)
    .fetch_one(&pool)
    .await
    .expect("load final state");
    assert_eq!(row, ("partial".to_string(), "mixed_partial".to_string()));
}
```

- [x] **Step 10: Run targeted Takeout/provenance tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml ingest_provenance::tests:: takeout_import::tests:: takeout_import::export_dc::tests:: sources::items::tests::
```

Expected: targeted tests pass.

- [x] **Step 11: Commit Takeout runtime wiring**

Run:

```powershell
git add src-tauri/src/takeout_import/export_dc.rs src-tauri/src/takeout_import/mod.rs src-tauri/src/ingest_provenance.rs
git commit -m "feat: persist takeout provenance at runtime"
```

Expected: commit succeeds.

---

### Task 6: Documentation And Verification

**Files:**
- Modify: `docs/database-schema.md`
- Modify: `docs/takeout-source-import.md`
- Optionally modify: `docs/database-schema-legacy-analysis.md`

- [ ] **Step 1: Update database schema docs**

In `docs/database-schema.md`, add a section for ingest provenance after the source/item identity sections. Include this exact semantic content:

```markdown
## Ingest provenance

Migration `23.sql` adds generic ingest provenance tables. Runtime wiring in
this slice is Telegram Takeout-only; normal `sync_source` does not write these
tables yet.

`ingest_batches` stores one durable row per actually-started locked ingest
attempt. `status` is persisted as `running`, `completed`, `failed`, or
`cancelled`; crash-interrupted imports remain `running` and can be interpreted
by query/UI code as interrupted when no in-memory job exists after restart.

`completeness` is separate from status. A completed zero-message traversal can
be `complete` when selected history traversal finished normally and no partial
flags were set. Only-my-messages fallback and migrated-history deferment are
`partial`.

`item_observed_count` counts all item-level observation rows. It can be greater
than `item_inserted_count + item_duplicate_count + item_skipped_count` when
`outcome = 'failed'` rows exist, because there is no dedicated failed counter in
the foundation schema.

`telegram_takeout_batches.account_id` is a historical identity snapshot for the
Takeout run. The source/batch relationship owns provenance retention; deleting
an account must not delete the detail row while leaving the generic batch row.

`ingest_item_observations.provider_identity` is a generic text identity. For
Telegram it uses `telegram:history_peer:<kind>:<id>:message:<message_id>`,
where the peer is the message history peer from `telegram_messages`, not the
current resolved source peer.

Warning messages and terminal errors are bounded and sanitized. They must not
store raw Telegram TL payloads, session data, auth material, cookies, headers,
or compressed payload dumps.
```

- [ ] **Step 2: Update Takeout docs**

In `docs/takeout-source-import.md`, replace the old provenance gap wording with:

```markdown
Telegram Takeout imports now create durable ingest provenance after the
same-source ingest lock is acquired. The in-memory job remains the current UI
state mechanism, and `batch_id` is a correlation id for tests and future UI.

Successful Takeout marks the batch `completed`. Failed and cancelled runs mark
the batch `failed` or `cancelled` and leave already inserted rows linked to the
batch through item observations. Source watermarks still advance only after
`finishTakeoutSession(success=true)` and `finalize_sync(...)` succeed.

`running` batches survive restart. The schema does not persist an
`interrupted` status; query/UI code may derive that display state from a
durable running batch with no active in-memory job.

Migrated supergroup history remains disabled in this foundation slice. When it
is detected, Takeout records `migrated_history_detected = 1`,
`migrated_history_imported = 0`, a `migrated_history_deferred` warning, and
partial completeness.
```

- [ ] **Step 3: Run documentation scans**

Run:

```powershell
rg -n "completed_at|runner-managed.*23|migrated history import is enabled|T[O]DO|T[B]D|FIX[M]E" docs src-tauri
```

Expected: no misleading new matches. Existing historical references outside this slice may remain only if they describe old migrations or old plans.

- [ ] **Step 4: Run targeted tests**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml migrations::tests:: ingest_provenance::tests:: sources::items::tests:: takeout_import::tests:: takeout_import::export_dc::tests::
```

Expected: all targeted tests pass.

- [ ] **Step 5: Run full Rust test suite**

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: full Rust test suite passes.

- [ ] **Step 6: Check formatting and diff**

Run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml --check
git diff --check
git status --short
```

Expected: formatting check passes, diff check has no whitespace errors, and status shows only intended files.

- [ ] **Step 7: Commit documentation and final verification updates**

Run:

```powershell
git add docs/database-schema.md docs/takeout-source-import.md docs/database-schema-legacy-analysis.md
git commit -m "docs: document takeout provenance foundation"
```

If `docs/database-schema-legacy-analysis.md` was not modified, omit it from `git add`.

Expected: commit succeeds.

---

## Self-Review Checklist

- Spec coverage: Task 1 covers regular migration and schema constraints; Task 2 covers storage helpers, sanitization, counters, completeness; Task 3 covers inserted/duplicate/skipped observations and history-peer identity; Task 4 covers same-source lock ordering and `batch_id`; Task 5 covers Takeout runtime detail, warnings, terminal states, finish cleanup, source watermark ordering, and migrated-history deferment; Task 6 covers documentation and verification.
- Public API: `start_takeout_source_import`, `cancel_takeout_source_import`, and `list_takeout_source_import_jobs` remain minimal; only `TakeoutImportJobRecord` gains `batch_id`.
- Containment: normal `sync_source` still calls the bool wrapper and does not write provenance.
- Crash behavior: no persisted `interrupted` enum is added; `running` survives restart.
- Scalability note: generic text identity and observation retention are documented as foundation tradeoffs and follow-up work.
