# Takeout Migrated-History Opt-In Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enable explicit opt-in import of Telegram migrated small-group history as a separate historical scope without changing normal Takeout, default browsing, analysis, reports, or NotebookLM export behavior.

**Architecture:** Keep normal Takeout as the current-history importer and add a separate backend command for migrated small-group history. Store source-level migrated-history availability in a companion table, import migrated rows under native old `chat` identity, and use a typed insert context to skip derived current-history writes for migrated rows. Default read models stay current-history-only until a later explicit domain-aware reader is built.

**Tech Stack:** Rust/Tauri backend, SQLx SQLite schema and tests, existing Telegram Takeout TL plumbing, Svelte 5 and TypeScript frontend state/API wrappers, Cargo tests, Vitest.

---

## Spec

Read first:

- `docs/superpowers/specs/2026-05-27-takeout-migrated-history-opt-in-design.md`
- `docs/takeout-source-import.md`
- `docs/database-schema.md`

This plan implements these accepted decisions:

- Normal Takeout never imports old small-group history.
- The explicit command is `start_takeout_migrated_history_import(source_id) -> { job_id }`.
- Source-level availability lives in a companion table, not in `telegram_sources`.
- The first implementation omits `migrated_from_max_id`; add it later only if Telegram exposes a stable validated boundary hint.
- Durable duplicate identity remains native: `(source_id, history_peer_kind, history_peer_id, telegram_message_id)`.
- Row-level migrated history uses `history_peer_kind = 'chat'`, old `migrated_from_chat_id`, `is_migrated_history = 1`, and `migration_domain = 'migrated_from_chat'`.
- Batch-level migrated run scope is `history_scope = 'migrated_small_group_history'`.
- Historical import uses the same source ingest lock as sync, delete, and current Takeout.
- Historical import does not update `sources.last_sync_state` or `sources.last_synced_at`.
- First implementation does not create `analysis_documents`, `archive_read_items`, or `item_topic_memberships` for migrated historical rows.
- Default source browsing, analysis corpus, reports, and NotebookLM export stay current-history-only.
- User-facing UI copy must describe the action as a separate historical import, not as retry or sync.

## File Structure

- Modify: `src-tauri/migrations/0001_current_schema_baseline.sql`
  - Add `telegram_migrated_history_capabilities`.
  - Add `migrated_small_group_history` to `telegram_takeout_batches.history_scope` check.
  - Tighten `telegram_messages.migration_domain` check to `NULL` or `migrated_from_chat`.
- Modify: `src-tauri/src/migrations.rs`
  - Assert the fresh baseline includes the companion table and new `history_scope`.
- Modify: `src-tauri/src/sources/test_support.rs`
  - Mirror baseline schema changes for in-memory tests.
- Modify: `src-tauri/src/sources/types.rs`
  - Add typed constants for `migrated_from_chat` and source-visible migrated-history status values.
  - Add optional source record fields exposed to frontend without raw old chat ids.
- Modify: `src-tauri/src/sources/store.rs`
  - Join companion capability state into `list_sources` and `load_source_record`.
- Create: `src-tauri/src/takeout_import/migrated_history.rs`
  - Own capability storage helpers, status/reason constants, validation result types, and sanitized availability updates.
- Modify: `src-tauri/src/takeout_import/mod.rs`
  - Record normal Takeout detection into the capability table.
  - Add `start_takeout_migrated_history_import`.
  - Add migrated-history validation and import flow.
  - Register historical batch scope and avoid current watermark advancement.
- Modify: `src-tauri/src/takeout_import/state.rs`
  - Add `history_scope` to job records so frontend can label current versus migrated jobs.
- Modify: `src-tauri/src/ingest_provenance.rs`
  - Add typed `history_scope` constants and helpers for historical batches.
- Modify: `src-tauri/src/sources/items.rs`
  - Add `TelegramInsertContext::CurrentHistory` and `TelegramInsertContext::MigratedSmallGroupHistory`.
  - Use the enum to control topic membership, analysis document, and archive-read side effects.
- Modify: `src-tauri/src/sources/items/query.rs`
  - Exclude migrated historical rows from default direct item reads.
- Modify: `src-tauri/src/archive_read_model.rs`
  - Exclude migrated historical rows from archive rebuild and single-row archive materialization.
- Modify: `src-tauri/src/analysis_documents.rs`
  - Exclude migrated historical rows from default analysis document rebuild and single-row document upsert.
- Modify: `src-tauri/src/lib.rs`
  - Register the new Tauri command.
- Modify: `src/lib/types/sources.ts`
  - Add source-visible `migratedHistoryStatus` and Takeout job `historyScope`.
- Modify: `src/lib/api/sources.ts`
  - Map sanitized migrated-history capability status from Rust.
- Modify: `src/lib/api/takeout-import.ts`
  - Add `startTakeoutMigratedHistoryImport`.
- Modify: `src/lib/api/takeout-import.test.ts`
  - Pin the new command name and job `history_scope` mapping.
- Modify: `src/lib/api/sources.test.ts`
  - Pin sanitized capability status mapping.
- Modify: `src/lib/analysis-state.ts`
  - Add UI policy helpers for the historical action label, disabled reason, and recovery warning copy.
- Modify: `src/lib/analysis-state.test.ts`
  - Prove the action is not named retry or sync.
- Modify: `src/lib/components/analysis/source-switcher-panel.svelte`
  - Render the explicit migrated-history action only when capability status is `available`.
- Modify: `src/lib/components/analysis/compact-source-rail.svelte`
  - Pass through the historical action handler and start state to `SourceSwitcherPanel`.
- Modify: `src/routes/analysis/+page.svelte`
  - Wire the frontend action to the new API wrapper.
- Modify: `docs/takeout-source-import.md`
  - Document command, capability table, locking, watermark, and read defaults.
- Modify: `docs/database-schema.md`
  - Document companion table and allowed row marker values.
- Modify: `docs/backlog.md`
  - Move the opt-in implementation from design-ready to implementation-tracked.

---

### Task 1: Lock Current Deferment And Schema Constants

**Files:**
- Modify: `src-tauri/src/takeout_import/mod.rs`
- Modify: `src-tauri/src/ingest_provenance.rs`
- Modify: `src-tauri/src/sources/types.rs`
- Modify: `src-tauri/src/sources/items.rs`
- Modify: `src-tauri/src/sources/test_support.rs`
- Modify: `src-tauri/migrations/0001_current_schema_baseline.sql`

- [x] **Step 1: Add typed constants**

In `src-tauri/src/sources/types.rs`, add these constants near the existing Telegram kind constants:

```rust
pub(crate) const TELEGRAM_MIGRATION_DOMAIN_MIGRATED_FROM_CHAT: &str = "migrated_from_chat";

pub(crate) const MIGRATED_HISTORY_STATUS_NONE: &str = "none";
pub(crate) const MIGRATED_HISTORY_STATUS_AVAILABLE: &str = "available";
pub(crate) const MIGRATED_HISTORY_STATUS_UNAVAILABLE: &str = "unavailable";
```

In `src-tauri/src/ingest_provenance.rs`, add these constants below `PROVENANCE_TEXT_MAX_LEN`:

```rust
pub(crate) const TAKEOUT_HISTORY_SCOPE_CURRENT: &str = "current_history";
pub(crate) const TAKEOUT_HISTORY_SCOPE_CURRENT_WITH_MIGRATED_DEFERRED: &str =
    "current_history_with_migrated_deferred";
pub(crate) const TAKEOUT_HISTORY_SCOPE_PARTIAL_PRIVATE: &str = "partial_private_history";
pub(crate) const TAKEOUT_HISTORY_SCOPE_MIXED_PARTIAL: &str = "mixed_partial";
pub(crate) const TAKEOUT_HISTORY_SCOPE_MIGRATED_SMALL_GROUP: &str =
    "migrated_small_group_history";
```

- [x] **Step 2: Replace string literals in provenance helpers**

In `mark_takeout_migrated_history_deferred`, replace the SQL string with a formatted query that uses the constants:

```rust
    let query = format!(
        "UPDATE telegram_takeout_batches
         SET migrated_history_detected = 1,
             migrated_history_imported = 0,
             history_scope = CASE
               WHEN only_my_messages = 1 THEN '{}'
               ELSE '{}'
             END,
             updated_at = CURRENT_TIMESTAMP
         WHERE batch_id = ?",
        TAKEOUT_HISTORY_SCOPE_MIXED_PARTIAL,
        TAKEOUT_HISTORY_SCOPE_CURRENT_WITH_MIGRATED_DEFERRED
    );
    sqlx::query(&query)
```

In `mark_takeout_only_my_messages_fallback`, use the same pattern with `TAKEOUT_HISTORY_SCOPE_MIXED_PARTIAL` and `TAKEOUT_HISTORY_SCOPE_PARTIAL_PRIVATE`.

- [x] **Step 3: Add the migrated small-group batch helper tests first**

In `src-tauri/src/ingest_provenance.rs`, add these tests inside the existing test module:

```rust
    #[tokio::test]
    async fn migrated_small_group_scope_can_be_marked_running_and_completed() {
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

        mark_takeout_migrated_small_group_scope(&pool, batch_id)
            .await
            .expect("mark historical scope");
        mark_takeout_migrated_history_imported(&pool, batch_id)
            .await
            .expect("mark imported");
        finalize_ingest_batch(&pool, batch_id, TerminalBatchStatus::Completed, None)
            .await
            .expect("finalize");

        let row: (String, i64, i64) = sqlx::query_as(
            "SELECT history_scope, migrated_history_detected, migrated_history_imported
             FROM telegram_takeout_batches
             WHERE batch_id = ?",
        )
        .bind(batch_id)
        .fetch_one(&pool)
        .await
        .expect("load batch");

        assert_eq!(
            row,
            (
                TAKEOUT_HISTORY_SCOPE_MIGRATED_SMALL_GROUP.to_string(),
                1,
                1,
            )
        );
    }

    #[tokio::test]
    async fn migrated_small_group_imported_allows_duplicate_only_success() {
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

        mark_takeout_migrated_small_group_scope(&pool, batch_id)
            .await
            .expect("mark scope");
        record_ingest_observation(
            &pool,
            IngestObservation {
                batch_id,
                source_id: 1,
                item_id: Some(55),
                provider_item_kind: ITEM_KIND_TELEGRAM_MESSAGE,
                provider_identity_kind: "telegram_message",
                provider_identity: "telegram:history_peer:chat:777:message:42".to_string(),
                outcome: "duplicate_observed",
                reason_code: None,
            },
        )
        .await
        .expect("record duplicate observation");
        mark_takeout_migrated_history_imported(&pool, batch_id)
            .await
            .expect("mark imported");
        finalize_ingest_batch(&pool, batch_id, TerminalBatchStatus::Completed, None)
            .await
            .expect("finalize");

        let row: (String, i64, i64, i64, i64) = sqlx::query_as(
            "SELECT b.completeness, t.migrated_history_imported,
                    b.item_observed_count, b.item_inserted_count, b.item_duplicate_count
             FROM ingest_batches b
             JOIN telegram_takeout_batches t ON t.batch_id = b.id
             WHERE b.id = ?",
        )
        .bind(batch_id)
        .fetch_one(&pool)
        .await
        .expect("load final state");

        assert_eq!(row, ("complete".to_string(), 1, 1, 0, 1));
    }
```

- [x] **Step 4: Run the focused tests and verify they fail**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml migrated_small_group_scope
cargo test --manifest-path src-tauri\Cargo.toml migrated_small_group_imported
```

Expected: both fail because `migrated_small_group_history`, `mark_takeout_migrated_small_group_scope`, and `mark_takeout_migrated_history_imported` do not exist yet.

- [x] **Step 5: Extend schema checks**

In `src-tauri/migrations/0001_current_schema_baseline.sql`, add `migrated_small_group_history` to the `telegram_takeout_batches.history_scope` check:

```sql
    'mixed_partial',
    'migrated_small_group_history'
```

In the same file, add this check to `telegram_messages`:

```sql
    CHECK (migration_domain IS NULL OR migration_domain IN ('migrated_from_chat')),
```

Put it after `CHECK (is_migrated_history IN (0, 1))`.

In `src-tauri/src/sources/test_support.rs`, mirror both changes in `TELEGRAM_MESSAGES_SCHEMA_SQL` and `INGEST_PROVENANCE_SCHEMA_SQL`.

- [x] **Step 6: Add the provenance helpers**

In `src-tauri/src/ingest_provenance.rs`, add:

```rust
pub(crate) async fn mark_takeout_migrated_small_group_scope(
    pool: &sqlx::Pool<Sqlite>,
    batch_id: i64,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE telegram_takeout_batches
         SET migrated_history_detected = 1,
             migrated_history_imported = 0,
             history_scope = ?,
             updated_at = CURRENT_TIMESTAMP
         WHERE batch_id = ?",
    )
    .bind(TAKEOUT_HISTORY_SCOPE_MIGRATED_SMALL_GROUP)
    .bind(batch_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

pub(crate) async fn mark_takeout_migrated_history_imported(
    pool: &sqlx::Pool<Sqlite>,
    batch_id: i64,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE telegram_takeout_batches
         SET migrated_history_detected = 1,
             migrated_history_imported = 1,
             history_scope = ?,
             updated_at = CURRENT_TIMESTAMP
         WHERE batch_id = ?",
    )
    .bind(TAKEOUT_HISTORY_SCOPE_MIGRATED_SMALL_GROUP)
    .bind(batch_id)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}
```

- [x] **Step 7: Replace row-level marker literals in tests**

In `src-tauri/src/sources/items.rs`, import the migration-domain constant in the test module:

```rust
    use crate::sources::types::{
        TelegramMessageIdentity, ITEM_KIND_TELEGRAM_MESSAGE,
        TELEGRAM_MIGRATION_DOMAIN_MIGRATED_FROM_CHAT,
    };
```

In `insert_telegram_source_item_allows_same_message_id_in_different_history_domains`, replace:

```rust
            migration_domain: Some("migrated_from_chat".to_string()),
```

with:

```rust
            migration_domain: Some(TELEGRAM_MIGRATION_DOMAIN_MIGRATED_FROM_CHAT.to_string()),
```

- [x] **Step 8: Run focused regressions**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml migrated_small_group_scope
cargo test --manifest-path src-tauri\Cargo.toml migrated_small_group_imported
cargo test --manifest-path src-tauri\Cargo.toml insert_telegram_source_item_allows_same_message_id_in_different_history_domains
cargo test --manifest-path src-tauri\Cargo.toml migrated_history_deferred_scope_finalizes_partial_and_records_warning_once
```

Expected: all pass.

- [x] **Step 9: Commit Task 1**

Run:

```powershell
git add src-tauri\src\ingest_provenance.rs src-tauri\src\sources\types.rs src-tauri\src\sources\items.rs src-tauri\src\sources\test_support.rs src-tauri\migrations\0001_current_schema_baseline.sql docs\superpowers\plans\2026-05-27-takeout-migrated-history-opt-in.md
git commit -m "test: define migrated history storage constants"
```

Expected: commit succeeds.

---

### Task 2: Source-Level Migrated History Capability

**Files:**
- Create: `src-tauri/src/takeout_import/migrated_history.rs`
- Modify: `src-tauri/src/takeout_import/mod.rs`
- Modify: `src-tauri/src/sources/store.rs`
- Modify: `src-tauri/src/sources/types.rs`
- Modify: `src-tauri/src/sources/test_support.rs`
- Modify: `src-tauri/migrations/0001_current_schema_baseline.sql`
- Modify: `src-tauri/src/migrations.rs`
- Modify: `src/lib/types/sources.ts`
- Modify: `src/lib/api/sources.ts`
- Modify: `src/lib/api/sources.test.ts`

- [x] **Step 1: Add backend tests for capability storage**

Create `src-tauri/src/takeout_import/migrated_history.rs` with the module shell and tests:

```rust
use crate::error::{AppError, AppResult};
use crate::sources::types::{
    MIGRATED_HISTORY_STATUS_AVAILABLE, MIGRATED_HISTORY_STATUS_NONE,
    MIGRATED_HISTORY_STATUS_UNAVAILABLE,
};

pub(crate) const MIGRATED_HISTORY_REASON_NOT_DETECTED: &str = "not_detected";
pub(crate) const MIGRATED_HISTORY_REASON_MISSING_FROM_CHAT_ID: &str =
    "missing_migrated_from_chat_id";
pub(crate) const MIGRATED_HISTORY_REASON_CURRENT_SOURCE_UNAVAILABLE: &str =
    "current_source_unavailable";
pub(crate) const MIGRATED_HISTORY_REASON_OLD_CHAT_INPUT_UNAVAILABLE: &str =
    "old_chat_input_unavailable";
pub(crate) const MIGRATED_HISTORY_REASON_REVALIDATION_FAILED: &str = "revalidation_failed";

#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub(crate) struct MigratedHistoryCapability {
    pub(crate) source_id: i64,
    pub(crate) status: String,
    pub(crate) unavailable_reason: Option<String>,
    pub(crate) migrated_from_chat_id: Option<i64>,
    pub(crate) detected_at: Option<i64>,
    pub(crate) refreshed_at: i64,
}

pub(crate) async fn create_migrated_history_capability_schema(
    pool: &sqlx::SqlitePool,
) -> AppResult<()> {
    sqlx::raw_sql(MIGRATED_HISTORY_CAPABILITY_SCHEMA_SQL)
        .execute(pool)
        .await
        .map_err(AppError::database)
}

pub(crate) async fn load_migrated_history_capability(
    pool: &sqlx::SqlitePool,
    source_id: i64,
) -> AppResult<Option<MigratedHistoryCapability>> {
    sqlx::query_as(
        "SELECT source_id, status, unavailable_reason, migrated_from_chat_id,
                detected_at, refreshed_at
         FROM telegram_migrated_history_capabilities
         WHERE source_id = ?",
    )
    .bind(source_id)
    .fetch_optional(pool)
    .await
    .map_err(AppError::database)
}

pub(crate) async fn upsert_migrated_history_available(
    pool: &sqlx::SqlitePool,
    source_id: i64,
    migrated_from_chat_id: i64,
    observed_at: i64,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO telegram_migrated_history_capabilities (
             source_id, status, unavailable_reason, migrated_from_chat_id,
             detected_at, refreshed_at
         ) VALUES (?, ?, NULL, ?, ?, ?)
         ON CONFLICT(source_id) DO UPDATE SET
             status = excluded.status,
             unavailable_reason = NULL,
             migrated_from_chat_id = excluded.migrated_from_chat_id,
             detected_at = COALESCE(telegram_migrated_history_capabilities.detected_at, excluded.detected_at),
             refreshed_at = excluded.refreshed_at",
    )
    .bind(source_id)
    .bind(MIGRATED_HISTORY_STATUS_AVAILABLE)
    .bind(migrated_from_chat_id)
    .bind(observed_at)
    .bind(observed_at)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

pub(crate) async fn mark_migrated_history_unavailable(
    pool: &sqlx::SqlitePool,
    source_id: i64,
    reason: &str,
    observed_at: i64,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO telegram_migrated_history_capabilities (
             source_id, status, unavailable_reason, migrated_from_chat_id,
             detected_at, refreshed_at
         ) VALUES (?, ?, ?, NULL, NULL, ?)
         ON CONFLICT(source_id) DO UPDATE SET
             status = excluded.status,
             unavailable_reason = excluded.unavailable_reason,
             migrated_from_chat_id = NULL,
             refreshed_at = excluded.refreshed_at",
    )
    .bind(source_id)
    .bind(MIGRATED_HISTORY_STATUS_UNAVAILABLE)
    .bind(reason)
    .bind(observed_at)
    .execute(pool)
    .await
    .map_err(AppError::database)?;
    Ok(())
}

pub(crate) const MIGRATED_HISTORY_CAPABILITY_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS telegram_migrated_history_capabilities (
    source_id INTEGER PRIMARY KEY REFERENCES telegram_sources(source_id) ON DELETE CASCADE,
    status TEXT NOT NULL,
    unavailable_reason TEXT,
    migrated_from_chat_id INTEGER,
    detected_at INTEGER,
    refreshed_at INTEGER NOT NULL,
    CHECK (status IN ('none', 'available', 'unavailable')),
    CHECK (
        unavailable_reason IS NULL
        OR unavailable_reason IN (
            'not_detected',
            'missing_migrated_from_chat_id',
            'current_source_unavailable',
            'old_chat_input_unavailable',
            'revalidation_failed'
        )
    ),
    CHECK (migrated_from_chat_id IS NULL OR migrated_from_chat_id > 0),
    CHECK (status <> 'available' OR migrated_from_chat_id IS NOT NULL),
    CHECK (status <> 'unavailable' OR unavailable_reason IS NOT NULL)
);

CREATE INDEX IF NOT EXISTS idx_telegram_migrated_history_capabilities_status
    ON telegram_migrated_history_capabilities(status);
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sources::test_support::{
        create_migrated_history_capability_tables, memory_pool_with_sources,
    };

    async fn seed_telegram_source(pool: &sqlx::SqlitePool) {
        sqlx::query(
            "INSERT INTO sources (
                id, source_type, source_subtype, account_id, external_id, title,
                is_active, is_member, created_at
             ) VALUES (1, 'telegram', 'supergroup', 10, '12345', 'Forum', 1, 1, 1)",
        )
        .execute(pool)
        .await
        .expect("seed source");
        sqlx::query(
            "INSERT INTO telegram_sources (
                source_id, account_id, source_subtype, peer_kind, peer_id, resolution_strategy
             ) VALUES (1, 10, 'supergroup', 'channel', 12345, 'dialog')",
        )
        .execute(pool)
        .await
        .expect("seed telegram source");
    }

    #[tokio::test]
    async fn capability_available_is_source_level_and_restart_safe() {
        let pool = memory_pool_with_sources().await;
        create_migrated_history_capability_tables(&pool).await;
        seed_telegram_source(&pool).await;

        upsert_migrated_history_available(&pool, 1, 777, 100)
            .await
            .expect("mark available");
        upsert_migrated_history_available(&pool, 1, 777, 200)
            .await
            .expect("refresh available");

        let capability = load_migrated_history_capability(&pool, 1)
            .await
            .expect("load capability")
            .expect("capability exists");

        assert_eq!(capability.status, MIGRATED_HISTORY_STATUS_AVAILABLE);
        assert_eq!(capability.unavailable_reason, None);
        assert_eq!(capability.migrated_from_chat_id, Some(777));
        assert_eq!(capability.detected_at, Some(100));
        assert_eq!(capability.refreshed_at, 200);
    }

    #[tokio::test]
    async fn capability_unavailable_keeps_reason_internal_and_clears_chat_hint() {
        let pool = memory_pool_with_sources().await;
        create_migrated_history_capability_tables(&pool).await;
        seed_telegram_source(&pool).await;

        upsert_migrated_history_available(&pool, 1, 777, 100)
            .await
            .expect("mark available");
        mark_migrated_history_unavailable(
            &pool,
            1,
            MIGRATED_HISTORY_REASON_OLD_CHAT_INPUT_UNAVAILABLE,
            250,
        )
        .await
        .expect("mark unavailable");

        let capability = load_migrated_history_capability(&pool, 1)
            .await
            .expect("load capability")
            .expect("capability exists");

        assert_eq!(capability.status, MIGRATED_HISTORY_STATUS_UNAVAILABLE);
        assert_eq!(
            capability.unavailable_reason.as_deref(),
            Some(MIGRATED_HISTORY_REASON_OLD_CHAT_INPUT_UNAVAILABLE)
        );
        assert_eq!(capability.migrated_from_chat_id, None);
        assert_eq!(capability.detected_at, Some(100));
        assert_eq!(capability.refreshed_at, 250);
    }
}
```

- [x] **Step 2: Run the new tests and verify they fail**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml migrated_history::tests
```

Expected: fail because the module is not declared and the test-support helper does not exist.

- [x] **Step 3: Add the companion table to schemas**

In `src-tauri/migrations/0001_current_schema_baseline.sql`, add the table immediately after `telegram_sources`:

```sql
CREATE TABLE telegram_migrated_history_capabilities (
    source_id INTEGER PRIMARY KEY REFERENCES telegram_sources(source_id) ON DELETE CASCADE,
    status TEXT NOT NULL,
    unavailable_reason TEXT,
    migrated_from_chat_id INTEGER,
    detected_at INTEGER,
    refreshed_at INTEGER NOT NULL,
    CHECK (status IN ('none', 'available', 'unavailable')),
    CHECK (
        unavailable_reason IS NULL
        OR unavailable_reason IN (
            'not_detected',
            'missing_migrated_from_chat_id',
            'current_source_unavailable',
            'old_chat_input_unavailable',
            'revalidation_failed'
        )
    ),
    CHECK (migrated_from_chat_id IS NULL OR migrated_from_chat_id > 0),
    CHECK (status <> 'available' OR migrated_from_chat_id IS NOT NULL),
    CHECK (status <> 'unavailable' OR unavailable_reason IS NOT NULL)
);

CREATE INDEX idx_telegram_migrated_history_capabilities_status
    ON telegram_migrated_history_capabilities(status);
```

In `src-tauri/src/sources/test_support.rs`, add:

```rust
pub(crate) async fn create_migrated_history_capability_tables(pool: &sqlx::SqlitePool) {
    crate::takeout_import::migrated_history::create_migrated_history_capability_schema(pool)
        .await
        .expect("create migrated history capability schema");
}
```

Call `create_migrated_history_capability_tables(&pool).await;` at the end of `create_source_identity_tables`.

- [x] **Step 4: Declare the module**

In `src-tauri/src/takeout_import/mod.rs`, add:

```rust
pub(crate) mod migrated_history;
```

- [x] **Step 5: Expose sanitized capability status on source records**

In `src-tauri/src/sources/types.rs`, extend `SourceRecord`:

```rust
    pub migrated_history_status: String,
    pub migrated_history_detected_at: Option<i64>,
    pub migrated_history_refreshed_at: Option<i64>,
```

Extend `SourceRecordRow`:

```rust
    pub(super) migrated_history_status: Option<String>,
    pub(super) migrated_history_detected_at: Option<i64>,
    pub(super) migrated_history_refreshed_at: Option<i64>,
```

In `src-tauri/src/sources/store.rs`, add these selected columns to both `load_source_record` and `list_sources` queries:

```sql
               mhc.status AS migrated_history_status,
               mhc.detected_at AS migrated_history_detected_at,
               mhc.refreshed_at AS migrated_history_refreshed_at
```

Add the join:

```sql
        LEFT JOIN telegram_migrated_history_capabilities mhc ON mhc.source_id = s.id
```

In `source_record_from_row_parts`, set:

```rust
        migrated_history_status: row
            .migrated_history_status
            .unwrap_or_else(|| MIGRATED_HISTORY_STATUS_NONE.to_string()),
        migrated_history_detected_at: row.migrated_history_detected_at,
        migrated_history_refreshed_at: row.migrated_history_refreshed_at,
```

Import `MIGRATED_HISTORY_STATUS_NONE` from `types.rs`.

- [x] **Step 6: Add backend source-record tests**

In `src-tauri/src/sources/store.rs`, update `source_record_parts_allow_non_telegram_source` and `source_record_parts_emit_only_source_subtype` expected records to include:

```rust
        migrated_history_status: "none".to_string(),
        migrated_history_detected_at: None,
        migrated_history_refreshed_at: None,
```

Add this async test:

```rust
    #[tokio::test]
    async fn list_sources_exposes_sanitized_migrated_history_status_without_chat_id() {
        let pool = memory_pool_with_sources().await;
        create_migrated_history_capability_tables(&pool).await;
        seed_telegram_source_identity(&pool, 1, 10, "supergroup", "channel", 12345).await;
        crate::takeout_import::migrated_history::upsert_migrated_history_available(
            &pool, 1, 777, 100,
        )
        .await
        .expect("mark available");

        let row: SourceRecordRow = sqlx::query_as(
            "SELECT s.id, s.source_type, s.source_subtype, s.account_id, s.external_id,
                    s.title, s.metadata_zstd,
                    s.last_sync_state, s.last_synced_at, s.is_active, s.is_member, s.created_at,
                    ts.username AS telegram_username,
                    ts.avatar_cache_key AS telegram_avatar_cache_key,
                    mhc.status AS migrated_history_status,
                    mhc.detected_at AS migrated_history_detected_at,
                    mhc.refreshed_at AS migrated_history_refreshed_at
             FROM sources s
             LEFT JOIN telegram_sources ts ON ts.source_id = s.id
             LEFT JOIN telegram_migrated_history_capabilities mhc ON mhc.source_id = s.id
             WHERE s.id = 1",
        )
        .fetch_one(&pool)
        .await
        .expect("load row");

        let record = source_record_from_row_parts(row, None, None);

        assert_eq!(record.migrated_history_status, "available");
        assert_eq!(record.migrated_history_detected_at, Some(100));
        assert_eq!(record.migrated_history_refreshed_at, Some(100));
        assert!(!format!("{record:?}").contains("777"));
    }
```

If `seed_telegram_source_identity` does not exist, add this private helper inside the test module:

```rust
    async fn seed_telegram_source_identity(
        pool: &sqlx::SqlitePool,
        source_id: i64,
        account_id: i64,
        source_subtype: &str,
        peer_kind: &str,
        peer_id: i64,
    ) {
        sqlx::query(
            "INSERT INTO sources (
                id, source_type, source_subtype, account_id, external_id, title,
                is_active, is_member, created_at
             ) VALUES (?, 'telegram', ?, ?, ?, 'Forum', 1, 1, 1)",
        )
        .bind(source_id)
        .bind(source_subtype)
        .bind(account_id)
        .bind(peer_id.to_string())
        .execute(pool)
        .await
        .expect("seed source");
        sqlx::query(
            "INSERT INTO telegram_sources (
                source_id, account_id, source_subtype, peer_kind, peer_id, resolution_strategy
             ) VALUES (?, ?, ?, ?, ?, 'dialog')",
        )
        .bind(source_id)
        .bind(account_id)
        .bind(source_subtype)
        .bind(peer_kind)
        .bind(peer_id)
        .execute(pool)
        .await
        .expect("seed telegram source");
    }
```

- [x] **Step 7: Update frontend source types and mapper**

In `src/lib/types/sources.ts`, add:

```ts
export type TelegramMigratedHistoryStatus = "none" | "available" | "unavailable";
```

Extend `Source`:

```ts
  migratedHistoryStatus: TelegramMigratedHistoryStatus;
  migratedHistoryDetectedAt: number | null;
  migratedHistoryRefreshedAt: number | null;
```

In `src/lib/api/sources.ts`, import `TelegramMigratedHistoryStatus`, extend `RawSource`:

```ts
  migrated_history_status?: TelegramMigratedHistoryStatus | null;
  migrated_history_detected_at?: number | null;
  migrated_history_refreshed_at?: number | null;
```

In `mapSource`, add:

```ts
    migratedHistoryStatus: source.migrated_history_status ?? "none",
    migratedHistoryDetectedAt: source.migrated_history_detected_at ?? null,
    migratedHistoryRefreshedAt: source.migrated_history_refreshed_at ?? null,
```

- [x] **Step 8: Update frontend mapper tests**

In `src/lib/api/sources.test.ts`, in `lists sources with typed arguments and maps source fields`, add these raw fields:

```ts
        migrated_history_status: "available",
        migrated_history_detected_at: 100,
        migrated_history_refreshed_at: 200,
```

Add these expected fields:

```ts
        migratedHistoryStatus: "available",
        migratedHistoryDetectedAt: 100,
        migratedHistoryRefreshedAt: 200,
```

Add a test proving omitted fields map to `none`:

```ts
  it("defaults missing migrated history capability to none", async () => {
    invokeMock.mockResolvedValueOnce([
      {
        id: 9,
        source_type: "youtube",
        source_subtype: "video",
        account_id: null,
        external_id: "video-id",
        title: "Video",
        last_sync_state: null,
        last_synced_at: null,
        is_member: false,
        is_active: true,
        created_at: 1_600_002,
        avatar_data_url: null,
      },
    ]);

    await expect(listSources(null)).resolves.toMatchObject([
      {
        migratedHistoryStatus: "none",
        migratedHistoryDetectedAt: null,
        migratedHistoryRefreshedAt: null,
      },
    ]);
  });
```

- [x] **Step 9: Add migration assertions**

In `src-tauri/src/migrations.rs`, add assertions to the fresh schema test:

```rust
        "telegram_migrated_history_capabilities",
```

Add a check that the baseline SQL contains the new scope:

```rust
    assert!(migration.sql.contains("'migrated_small_group_history'"));
```

Add a check that the baseline SQL contains the row marker check:

```rust
    assert!(migration.sql.contains("migration_domain IS NULL OR migration_domain IN ('migrated_from_chat')"));
```

- [x] **Step 10: Run focused tests**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml migrated_history::tests
cargo test --manifest-path src-tauri\Cargo.toml list_sources_exposes_sanitized_migrated_history_status_without_chat_id
cargo test --manifest-path src-tauri\Cargo.toml fresh_schema
npm.cmd test -- src/lib/api/sources.test.ts
```

Expected: all pass.

- [x] **Step 11: Commit Task 2**

Run:

```powershell
git add src-tauri\migrations\0001_current_schema_baseline.sql src-tauri\src\migrations.rs src-tauri\src\sources\test_support.rs src-tauri\src\sources\types.rs src-tauri\src\sources\store.rs src-tauri\src\takeout_import\mod.rs src-tauri\src\takeout_import\migrated_history.rs src\lib\types\sources.ts src\lib\api\sources.ts src\lib\api\sources.test.ts docs\superpowers\plans\2026-05-27-takeout-migrated-history-opt-in.md
git commit -m "feat: store migrated history capability state"
```

Expected: commit succeeds.

---

### Task 3: Typed Insert Context And Default Read Exclusion

**Files:**
- Modify: `src-tauri/src/sources/items.rs`
- Modify: `src-tauri/src/sources/items/query.rs`
- Modify: `src-tauri/src/archive_read_model.rs`
- Modify: `src-tauri/src/analysis_documents.rs`

- [ ] **Step 1: Add failing insert-context test**

In `src-tauri/src/sources/items.rs`, add this test after `insert_telegram_source_item_allows_same_message_id_in_different_history_domains`:

```rust
    #[tokio::test]
    async fn migrated_small_group_insert_skips_current_history_derived_writes() {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_analysis_documents_table(&pool).await;
        create_ingest_provenance_tables(&pool).await;
        crate::sources::test_support::create_archive_read_model_tables(&pool).await;
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
        crate::archive_read_model::rebuild_source(&pool, 1)
            .await
            .expect("ready archive");
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
            history_peer_kind: "chat".to_string(),
            history_peer_id: 777,
            telegram_message_id: 42,
            migration_domain: Some(TELEGRAM_MIGRATION_DOMAIN_MIGRATED_FROM_CHAT.to_string()),
            is_migrated_history: true,
        };
        let mut item = telegram_insert("42", "historical body");
        item.telegram_context.reply_to_top_id = Some(200);

        let outcome = insert_telegram_source_item_with_observation_in_context(
            &pool,
            batch_id,
            1,
            identity,
            item,
            TelegramInsertContext::MigratedSmallGroupHistory,
        )
        .await
        .expect("insert migrated row");
        assert!(outcome.is_inserted());

        let analysis_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM analysis_documents WHERE source_id = 1")
                .fetch_one(&pool)
                .await
                .expect("count analysis docs");
        let archive_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM archive_read_items WHERE source_id = 1")
                .fetch_one(&pool)
                .await
                .expect("count archive rows");
        let membership_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM item_topic_memberships WHERE source_id = 1",
        )
        .fetch_one(&pool)
        .await
        .expect("count memberships");
        let state: (String, i64, i64) = sqlx::query_as(
            "SELECT status, unresolved_count, pending_item_count
             FROM telegram_topic_resolution_state
             WHERE source_id = 1",
        )
        .fetch_one(&pool)
        .await
        .expect("load topic state");

        assert_eq!(analysis_count, 0);
        assert_eq!(archive_count, 0);
        assert_eq!(membership_count, 0);
        assert_eq!(state, ("ready".to_string(), 0, 0));
    }
```

- [ ] **Step 2: Run the test and verify it fails**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml migrated_small_group_insert_skips_current_history_derived_writes
```

Expected: fail because `TelegramInsertContext` and the context-specific insert function do not exist.

- [ ] **Step 3: Add `TelegramInsertContext`**

In `src-tauri/src/sources/items.rs`, add near `ArchiveReadMaintenanceMode`:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TelegramInsertContext {
    CurrentHistory,
    MigratedSmallGroupHistory,
}

impl TelegramInsertContext {
    fn writes_topic_memberships(self) -> bool {
        matches!(self, Self::CurrentHistory)
    }

    fn writes_analysis_documents(self) -> bool {
        matches!(self, Self::CurrentHistory)
    }

    fn archive_maintenance(self, requested: ArchiveReadMaintenanceMode) -> ArchiveReadMaintenanceMode {
        match self {
            Self::CurrentHistory => requested,
            Self::MigratedSmallGroupHistory => ArchiveReadMaintenanceMode::Skip,
        }
    }
}
```

Extend `ArchiveReadMaintenanceMode`:

```rust
    Skip,
```

- [ ] **Step 4: Route current helpers through the enum**

Change `insert_telegram_source_item_outcome` to call:

```rust
        TelegramInsertContext::CurrentHistory,
        ArchiveReadMaintenanceMode::MaintainSingleWrite,
```

Change `insert_telegram_source_item_with_observation` to call the new function:

```rust
    insert_telegram_source_item_with_observation_in_context(
        pool,
        batch_id,
        source_id,
        identity,
        item,
        TelegramInsertContext::CurrentHistory,
    )
    .await
```

Add the new function with the previous body of `insert_telegram_source_item_with_observation` plus the context parameter:

```rust
pub(crate) async fn insert_telegram_source_item_with_observation_in_context(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    batch_id: i64,
    source_id: i64,
    identity: TelegramMessageIdentity,
    item: SourceItemInsert,
    insert_context: TelegramInsertContext,
) -> AppResult<TelegramItemInsertOutcome> {
    let provider_identity = crate::ingest_provenance::telegram_provider_identity(&identity);
    let mut conn = begin_immediate(pool).await?;

    let result: AppResult<TelegramItemInsertOutcome> = async {
        let outcome = insert_telegram_source_item_on_connection(
            &mut conn,
            source_id,
            identity,
            item,
            insert_context,
            ArchiveReadMaintenanceMode::MarkSourceStaleOnly,
        )
        .await?;
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

    finish_manual_transaction(&mut conn, result).await
}
```

Change `insert_telegram_source_item_on_connection` signature:

```rust
    insert_context: TelegramInsertContext,
    archive_maintenance: ArchiveReadMaintenanceMode,
```

- [ ] **Step 5: Gate derived writes in one place**

In `insert_telegram_source_item_on_connection`, replace the unconditional derived writes with:

```rust
    if insert_context.writes_topic_memberships() {
        crate::topic_memberships::resolve_scoped_topic_memberships_on_connection(
            conn,
            source_id,
            &[item_id],
            now_secs(),
        )
        .await?;
    }

    if insert_context.writes_analysis_documents() {
        crate::analysis_documents::upsert_item_backed_document_on_connection(conn, item_id).await?;
    }

    match insert_context.archive_maintenance(archive_maintenance) {
        ArchiveReadMaintenanceMode::MaintainSingleWrite => {
            crate::archive_read_model::upsert_item_on_connection(conn, item_id).await?;
        }
        ArchiveReadMaintenanceMode::MarkSourceStaleOnly => {
            crate::archive_read_model::mark_source_stale_on_connection(conn, source_id).await?;
        }
        ArchiveReadMaintenanceMode::Skip => {}
    }
```

- [ ] **Step 6: Add failing default read tests**

In `src-tauri/src/sources/items/query.rs`, add:

```rust
    #[tokio::test]
    async fn default_items_path_excludes_migrated_history_rows() {
        let pool = memory_pool_with_source_items_and_topics().await;
        seed_direct_item(&pool, 1, 10, "10", 1000, "current").await;
        seed_direct_item(&pool, 1, 11, "11", 900, "migrated").await;
        sqlx::query(
            "INSERT INTO telegram_messages (
                item_id, source_id, history_peer_kind, history_peer_id,
                telegram_message_id, migration_domain, is_migrated_history
             ) VALUES (10, 1, 'channel', 12345, 10, NULL, 0),
                      (11, 1, 'chat', 777, 10, 'migrated_from_chat', 1)",
        )
        .execute(&pool)
        .await
        .expect("seed telegram messages");

        let rows = load_item_rows_from_items_path(&pool, 1, 20, None, None, None)
            .await
            .expect("load rows");

        assert_eq!(rows.iter().map(|row| row.id).collect::<Vec<_>>(), vec![10]);
    }
```

If `seed_direct_item` does not exist in this test module, add:

```rust
    async fn seed_direct_item(
        pool: &sqlx::SqlitePool,
        source_id: i64,
        item_id: i64,
        external_id: &str,
        published_at: i64,
        content: &str,
    ) {
        sqlx::query(
            "INSERT OR IGNORE INTO sources (
                id, source_type, source_subtype, external_id, title, is_active, is_member, created_at
             ) VALUES (?, 'telegram', 'supergroup', '12345', 'Forum', 1, 1, 1)",
        )
        .bind(source_id)
        .execute(pool)
        .await
        .expect("seed source");
        sqlx::query(
            "INSERT INTO items (
                id, source_id, external_id, item_kind, author, published_at, ingested_at,
                content_zstd, raw_data_zstd, content_kind, has_media
             ) VALUES (?, ?, ?, 'telegram_message', 'alice', ?, ?, ?, NULL, 'text_only', 0)",
        )
        .bind(item_id)
        .bind(source_id)
        .bind(external_id)
        .bind(published_at)
        .bind(published_at)
        .bind(crate::compression::compress_text(content).expect("compress"))
        .execute(pool)
        .await
        .expect("seed item");
    }
```

In `src-tauri/src/archive_read_model.rs`, add:

```rust
    #[tokio::test]
    async fn rebuild_source_excludes_migrated_history_rows() {
        let pool = memory_pool_with_source_items_and_topics().await;
        seed_archive_row_fixture(&pool).await;
        sqlx::query(
            "INSERT INTO telegram_messages (
                item_id, source_id, history_peer_kind, history_peer_id,
                telegram_message_id, migration_domain, is_migrated_history
             ) VALUES (1, 1, 'channel', 12345, 10, NULL, 0),
                      (2, 1, 'chat', 777, 10, 'migrated_from_chat', 1)",
        )
        .execute(&pool)
        .await
        .expect("seed telegram messages");

        rebuild_source(&pool, 1).await.expect("rebuild source");

        let item_ids: Vec<i64> = sqlx::query_scalar(
            "SELECT item_id FROM archive_read_items WHERE source_id = 1 ORDER BY item_id",
        )
        .fetch_all(&pool)
        .await
        .expect("load archive rows");

        assert_eq!(item_ids, vec![1]);
    }
```

In `src-tauri/src/analysis_documents.rs`, add:

```rust
    #[tokio::test]
    async fn rebuild_analysis_documents_excludes_migrated_history_rows() {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_analysis_documents_schema(&pool)
            .await
            .expect("create analysis docs");
        seed_item_backed_document_fixture(&pool).await;
        sqlx::query(
            "INSERT INTO telegram_messages (
                item_id, source_id, history_peer_kind, history_peer_id,
                telegram_message_id, migration_domain, is_migrated_history
             ) VALUES (1, 1, 'channel', 12345, 10, NULL, 0),
                      (2, 1, 'chat', 777, 10, 'migrated_from_chat', 1)",
        )
        .execute(&pool)
        .await
        .expect("seed telegram messages");

        rebuild_analysis_documents_for_source(&pool, 1)
            .await
            .expect("rebuild docs");

        let item_ids: Vec<i64> = sqlx::query_scalar(
            "SELECT item_id FROM analysis_documents WHERE source_id = 1 ORDER BY item_id",
        )
        .fetch_all(&pool)
        .await
        .expect("load docs");

        assert_eq!(item_ids, vec![1]);
    }
```

For `archive_read_model.rs`, use the existing `seed_archive_source_fixture(&pool).await` helper before inserting the two `telegram_messages` rows. For `analysis_documents.rs`, use `seed_sources(&pool).await`, then call `seed_text_item(&pool, 1, 1, "1", "telegram_message", 100, "Current").await` and `seed_text_item(&pool, 2, 1, "2", "telegram_message", 90, "Migrated").await` before inserting the two `telegram_messages` rows.

- [ ] **Step 7: Run default read tests and verify they fail**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml excludes_migrated_history_rows
cargo test --manifest-path src-tauri\Cargo.toml default_items_path_excludes_migrated_history_rows
```

Expected: fail because readers still include migrated rows.

- [ ] **Step 8: Filter migrated rows from direct item reads**

In `src-tauri/src/sources/items/query.rs`, change the `around_published_at` query to:

```rust
            "SELECT items.published_at
             FROM items
             WHERE items.source_id = ?
               AND items.id = ?
               AND NOT EXISTS (
                 SELECT 1 FROM telegram_messages tm
                 WHERE tm.item_id = items.id
                   AND tm.is_migrated_history = 1
               )
             LIMIT 1",
```

In the main item query, add to the `WHERE` clause:

```sql
          AND NOT EXISTS (
            SELECT 1 FROM telegram_messages tm
            WHERE tm.item_id = items.id
              AND tm.is_migrated_history = 1
          )
```

- [ ] **Step 9: Filter migrated rows from archive read model**

In `src-tauri/src/archive_read_model.rs`, add the same `NOT EXISTS` filter to:

- the rebuild `SELECT FROM items`;
- `load_builder_row_for_item`.

For `load_builder_row_for_item`, keep `fetch_optional` and return `AppError::not_found("Item is not eligible for archive read materialization")` if the row is filtered out.

- [ ] **Step 10: Filter migrated rows from analysis documents**

In `src-tauri/src/analysis_documents.rs`, add the same `NOT EXISTS` filter to:

- `insert_item_backed_documents_for_source`;
- `upsert_item_backed_document_on_connection`.

When `upsert_item_backed_document_on_connection` filters out a migrated item, the existing `None` path deletes any stale document for that `item_id`.

- [ ] **Step 11: Run focused tests**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml migrated_small_group_insert_skips_current_history_derived_writes
cargo test --manifest-path src-tauri\Cargo.toml default_items_path_excludes_migrated_history_rows
cargo test --manifest-path src-tauri\Cargo.toml rebuild_source_excludes_migrated_history_rows
cargo test --manifest-path src-tauri\Cargo.toml rebuild_analysis_documents_excludes_migrated_history_rows
cargo test --manifest-path src-tauri\Cargo.toml telegram_insert_writes_analysis_document_in_same_writer_transaction
cargo test --manifest-path src-tauri\Cargo.toml takeout_observation_insert_marks_ready_archive_model_stale_without_per_item_build
```

Expected: all pass.

- [ ] **Step 12: Commit Task 3**

Run:

```powershell
git add src-tauri\src\sources\items.rs src-tauri\src\sources\items\query.rs src-tauri\src\archive_read_model.rs src-tauri\src\analysis_documents.rs docs\superpowers\plans\2026-05-27-takeout-migrated-history-opt-in.md
git commit -m "feat: isolate migrated history derived reads"
```

Expected: commit succeeds.

---

### Task 4: Separate Backend Command And Same-Source Lock

**Files:**
- Modify: `src-tauri/src/takeout_import/state.rs`
- Modify: `src-tauri/src/takeout_import/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/lib/types/sources.ts`
- Modify: `src/lib/api/takeout-import.ts`
- Modify: `src/lib/api/takeout-import.test.ts`

- [ ] **Step 1: Add job history-scope tests**

In `src-tauri/src/takeout_import/state.rs`, add `history_scope` to `TakeoutImportJobRecord`:

```rust
    pub history_scope: String,
```

Change `create_job` signature:

```rust
    pub(crate) async fn create_job(
        &self,
        source_id: i64,
        account_id: i64,
        batch_id: i64,
        history_scope: &str,
    ) -> AppResult<TakeoutImportJobRecord>
```

In the test `job_state_rejects_duplicate_active_source_jobs`, update calls:

```rust
        let first = state
            .create_job(7, 1, 100, TAKEOUT_HISTORY_SCOPE_CURRENT)
            .await
            .expect("create first job");
```

and:

```rust
            .create_job(7, 1, 101, TAKEOUT_HISTORY_SCOPE_MIGRATED_SMALL_GROUP)
```

Import the scope constants:

```rust
    use crate::ingest_provenance::{
        TAKEOUT_HISTORY_SCOPE_CURRENT, TAKEOUT_HISTORY_SCOPE_MIGRATED_SMALL_GROUP,
    };
```

Add:

```rust
    #[tokio::test]
    async fn job_state_records_history_scope_for_frontend_labels() {
        let state = TakeoutImportState::new();
        let job = state
            .create_job(7, 1, 100, TAKEOUT_HISTORY_SCOPE_MIGRATED_SMALL_GROUP)
            .await
            .expect("create historical job");

        assert_eq!(job.history_scope, TAKEOUT_HISTORY_SCOPE_MIGRATED_SMALL_GROUP);
    }
```

- [ ] **Step 2: Run state tests and verify they fail**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml job_state_records_history_scope_for_frontend_labels
```

Expected: fail until `create_job` stores the new field.

- [ ] **Step 3: Store history scope in job records**

In `create_job`, set:

```rust
            history_scope: history_scope.to_string(),
```

Update every `create_job` call in backend tests and production code to pass `TAKEOUT_HISTORY_SCOPE_CURRENT`.

- [ ] **Step 4: Add command wrapper tests**

In `src-tauri/src/takeout_import/mod.rs`, add a test near the existing start-record tests:

```rust
    #[tokio::test]
    async fn migrated_history_start_records_use_same_source_takeout_lock() {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_ingest_provenance_tables(&pool).await;
        create_migrated_history_capability_tables(&pool).await;
        seed_takeout_source(&pool, 1, 10, "supergroup", "channel", 12345).await;
        crate::takeout_import::migrated_history::upsert_migrated_history_available(
            &pool, 1, 777, 100,
        )
        .await
        .expect("capability");
        let locks = SourceIngestLocks::new();
        let state = TakeoutImportState::new();
        let _current = locks
            .try_acquire(1, SourceIngestKind::TakeoutImport)
            .await
            .expect("current takeout lock");

        let error = create_locked_migrated_history_start_records(
            &pool,
            &locks,
            &state,
            1,
            10,
            "supergroup".to_string(),
        )
        .await
        .expect_err("same source historical import should be locked");

        assert_eq!(error.kind, AppErrorKind::Conflict);
    }
```

Add another test:

```rust
    #[tokio::test]
    async fn migrated_history_start_requires_available_capability() {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_ingest_provenance_tables(&pool).await;
        create_migrated_history_capability_tables(&pool).await;
        seed_takeout_source(&pool, 1, 10, "supergroup", "channel", 12345).await;
        let locks = SourceIngestLocks::new();
        let state = TakeoutImportState::new();

        let error = create_locked_migrated_history_start_records(
            &pool,
            &locks,
            &state,
            1,
            10,
            "supergroup".to_string(),
        )
        .await
        .expect_err("missing capability should fail");

        assert_eq!(error.kind, AppErrorKind::Validation);
        assert!(error.message.contains("migrated_history_not_detected"));
    }
```

- [ ] **Step 5: Run command wrapper tests and verify they fail**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml migrated_history_start
```

Expected: fail because `create_locked_migrated_history_start_records` does not exist.

- [ ] **Step 6: Implement start-record creation**

In `src-tauri/src/takeout_import/mod.rs`, add imports:

```rust
use crate::ingest_provenance::{
    mark_takeout_migrated_small_group_scope, TAKEOUT_HISTORY_SCOPE_CURRENT,
    TAKEOUT_HISTORY_SCOPE_MIGRATED_SMALL_GROUP,
};
```

Update current `create_locked_takeout_start_records` to pass `TAKEOUT_HISTORY_SCOPE_CURRENT` to `create_job`.

Add:

```rust
async fn create_locked_migrated_history_start_records(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    ingest_locks: &SourceIngestLocks,
    state: &TakeoutImportState,
    source_id: i64,
    account_id: i64,
    source_subtype: String,
) -> AppResult<(TakeoutImportJobRecord, SourceIngestGuard)> {
    let capability =
        migrated_history::load_migrated_history_capability(pool, source_id).await?;
    let is_available = capability
        .as_ref()
        .is_some_and(|capability| capability.status == MIGRATED_HISTORY_STATUS_AVAILABLE);
    if !is_available {
        return Err(AppError::validation("migrated_history_not_detected"));
    }
    if source_subtype != TELEGRAM_KIND_SUPERGROUP {
        return Err(AppError::validation(
            "migrated_history_not_detected: only Telegram supergroups can have migrated small-group history",
        ));
    }

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
    mark_takeout_migrated_small_group_scope(pool, batch_id).await?;
    let record = state
        .create_job(
            source_id,
            account_id,
            batch_id,
            TAKEOUT_HISTORY_SCOPE_MIGRATED_SMALL_GROUP,
        )
        .await?;
    Ok((record, ingest_guard))
}
```

Import `MIGRATED_HISTORY_STATUS_AVAILABLE`.

- [ ] **Step 7: Add the Tauri command shell**

In `src-tauri/src/takeout_import/mod.rs`, add:

```rust
#[tauri::command]
pub async fn start_takeout_migrated_history_import(
    handle: AppHandle,
    repair_state: tauri::State<'_, SourceIdentityRepairState>,
    state: tauri::State<'_, TakeoutImportState>,
    source_id: i64,
) -> AppResult<StartTakeoutImportResponse> {
    require_source_identity_ready(repair_state.inner()).await?;
    let pool = get_pool(&handle).await?;
    let source = load_source(&pool, source_id).await?;
    let account_id = source.account_id.ok_or_else(|| {
        AppError::validation(format!("Source {source_id} is not linked to an account"))
    })?;
    let telegram_source_subtype = load_takeout_source_subtype(&pool, source.id).await?;
    let ingest_locks = handle.state::<SourceIngestLocks>();
    let (record, ingest_guard) = create_locked_migrated_history_start_records(
        &pool,
        &ingest_locks,
        state.inner(),
        source_id,
        account_id,
        telegram_source_subtype,
    )
    .await?;
    emit_takeout_import_event(&handle, &record);

    let job_id = record.job_id.clone();
    let task_handle = handle.clone();
    tauri::async_runtime::spawn(async move {
        run_takeout_migrated_history_import_job(task_handle, job_id, ingest_guard).await;
    });

    Ok(StartTakeoutImportResponse {
        job_id: record.job_id,
    })
}
```

Add a scaffold job runner that fails gracefully until Task 5:

```rust
async fn run_takeout_migrated_history_import_job(
    handle: AppHandle,
    job_id: String,
    ingest_guard: SourceIngestGuard,
) {
    let takeout_state = handle.state::<TakeoutImportState>();
    let Some(running_record) = takeout_state
        .update_job(&job_id, |job| {
            job.status = STATUS_RUNNING.to_string();
            job.phase = PHASE_VALIDATING_PEER.to_string();
            job.message = Some("Validating migrated history availability.".to_string());
        })
        .await
    else {
        drop(ingest_guard);
        return;
    };
    emit_takeout_import_event(&handle, &running_record);
    let batch_id = running_record.batch_id;
    finalize_terminal_batch_best_effort(
        &handle,
        batch_id,
        TerminalBatchStatus::Failed,
        Some("migrated_history_import_not_implemented"),
    )
    .await;
    if let Some(record) = takeout_state
        .finish_job(&job_id, |job| {
            job.status = STATUS_FAILED.to_string();
            job.phase = PHASE_FAILED.to_string();
            job.message = None;
            job.error = Some("migrated_history_import_not_implemented".to_string());
        })
        .await
    {
        emit_takeout_import_event(&handle, &record);
    }
    drop(ingest_guard);
}
```

- [ ] **Step 8: Register the command**

In `src-tauri/src/lib.rs`, add `start_takeout_migrated_history_import` to the existing `use takeout_import` list and to the existing `tauri::generate_handler!` command list.

- [ ] **Step 9: Add frontend API wrapper**

In `src/lib/types/sources.ts`, extend `TakeoutImportJobRecord`:

```ts
  history_scope: "current_history" | "migrated_small_group_history";
```

In `src/lib/api/takeout-import.ts`, add:

```ts
export function startTakeoutMigratedHistoryImport(sourceId: number) {
  return invoke<StartTakeoutImportResponse>("start_takeout_migrated_history_import", { sourceId });
}
```

In `src/lib/api/takeout-import.test.ts`, add:

```ts
  it("starts a migrated history import with a separate command", async () => {
    invokeMock.mockResolvedValueOnce({ job_id: "takeout-2" });

    await expect(startTakeoutMigratedHistoryImport(7)).resolves.toEqual({
      job_id: "takeout-2",
    });

    expect(invokeMock).toHaveBeenLastCalledWith("start_takeout_migrated_history_import", {
      sourceId: 7,
    });
  });
```

Update existing `TakeoutImportEvent` fixture objects with:

```ts
      history_scope: "current_history",
```

Import `startTakeoutMigratedHistoryImport`.

- [ ] **Step 10: Run focused tests**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml job_state_records_history_scope_for_frontend_labels
cargo test --manifest-path src-tauri\Cargo.toml migrated_history_start
npm.cmd test -- src/lib/api/takeout-import.test.ts
```

Expected: all pass.

- [ ] **Step 11: Commit Task 4**

Run:

```powershell
git add src-tauri\src\takeout_import\state.rs src-tauri\src\takeout_import\mod.rs src-tauri\src\lib.rs src\lib\types\sources.ts src\lib\api\takeout-import.ts src\lib\api\takeout-import.test.ts docs\superpowers\plans\2026-05-27-takeout-migrated-history-opt-in.md
git commit -m "feat: add migrated history import command"
```

Expected: commit succeeds.

---

### Task 5: Historical Validation Without Row Writes

**Files:**
- Modify: `src-tauri/src/takeout_import/migrated_history.rs`
- Modify: `src-tauri/src/takeout_import/mod.rs`

- [ ] **Step 1: Add validation unit tests for typed errors**

In `src-tauri/src/takeout_import/migrated_history.rs`, add:

```rust
pub(crate) fn not_detected_error() -> AppError {
    AppError::validation("migrated_history_not_detected")
}

pub(crate) fn unavailable_error() -> AppError {
    AppError::conflict("migrated_history_unavailable")
}
```

Add tests:

```rust
    #[test]
    fn migrated_history_errors_are_typed_for_frontend_behavior() {
        let not_detected = not_detected_error();
        assert_eq!(not_detected.kind, crate::error::AppErrorKind::Validation);
        assert_eq!(not_detected.message, "migrated_history_not_detected");

        let unavailable = unavailable_error();
        assert_eq!(unavailable.kind, crate::error::AppErrorKind::Conflict);
        assert_eq!(unavailable.message, "migrated_history_unavailable");
    }
```

- [ ] **Step 2: Run typed error test**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml migrated_history_errors_are_typed_for_frontend_behavior
```

Expected: pass after adding the helpers.

- [ ] **Step 3: Change normal migration detection to return the old chat id**

In `src-tauri/src/takeout_import/mod.rs`, change:

```rust
) -> AppResult<bool> {
```

to:

```rust
) -> AppResult<Option<i64>> {
```

Change the non-supergroup return:

```rust
        return Ok(None);
```

Change detected return:

```rust
            warnings.push(
                "Migrated small-group history detected; current Takeout keeps it deferred until explicit historical import."
                    .to_string(),
            );
            return Ok(Some(migrated_from_chat_id));
```

Change the final return:

```rust
    Ok(None)
```

In `run_started_takeout_source_import_inner`, replace:

```rust
    let migrated_detected = detect_supergroup_migration(
        pool,
        batch_id,
        client,
        alias,
        takeout_id,
        telegram_source_subtype,
        resolved_peer.peer,
        warnings,
        fallback_used,
        export_attempts,
    )
    .await?;
    if migrated_detected {
```

with:

```rust
    let migrated_from_chat_id = detect_supergroup_migration(
        pool,
        batch_id,
        client,
        alias,
        takeout_id,
        telegram_source_subtype,
        resolved_peer.peer,
        warnings,
        fallback_used,
        export_attempts,
    )
    .await?;
    if let Some(migrated_from_chat_id) = migrated_from_chat_id {
        migrated_history::upsert_migrated_history_available(
            pool,
            source.id,
            migrated_from_chat_id,
            crate::sources::now_secs(),
        )
        .await?;
```

Keep the existing `mark_takeout_migrated_history_deferred` call inside that block.

- [ ] **Step 4: Add a sanitized warning regression**

In `src-tauri/src/takeout_import/mod.rs`, add a small pure helper for the warning copy:

```rust
fn migrated_history_detected_warning() -> String {
    "Migrated small-group history detected; current Takeout keeps it deferred until explicit historical import."
        .to_string()
}
```

Then test:

```rust
    #[test]
    fn migrated_history_detected_warning_is_sanitized() {
        let warning = migrated_history_detected_warning();

        assert!(warning.contains("Migrated small-group history detected"));
        assert!(!warning.contains("migrated_from_chat_id"));
        assert!(!warning.contains("777"));
    }
```

Use the helper in `detect_supergroup_migration`.

- [ ] **Step 5: Add validation result type**

In `src-tauri/src/takeout_import/migrated_history.rs`, add:

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MigratedHistoryValidation {
    pub(crate) migrated_from_chat_id: i64,
}

pub(crate) fn validate_revalidated_chat_id(
    expected: Option<i64>,
    revalidated: Option<i64>,
) -> AppResult<MigratedHistoryValidation> {
    let expected = expected.ok_or_else(not_detected_error)?;
    match revalidated {
        Some(actual) if actual == expected => Ok(MigratedHistoryValidation {
            migrated_from_chat_id: actual,
        }),
        Some(_) | None => Err(unavailable_error()),
    }
}
```

Add tests:

```rust
    #[test]
    fn validation_accepts_matching_revalidated_chat_id() {
        let validation = validate_revalidated_chat_id(Some(777), Some(777))
            .expect("matching id");

        assert_eq!(validation.migrated_from_chat_id, 777);
    }

    #[test]
    fn validation_rejects_missing_or_changed_revalidated_chat_id() {
        assert_eq!(
            validate_revalidated_chat_id(None, Some(777))
                .expect_err("missing expected")
                .kind,
            crate::error::AppErrorKind::Validation
        );
        assert_eq!(
            validate_revalidated_chat_id(Some(777), None)
                .expect_err("missing revalidated")
                .kind,
            crate::error::AppErrorKind::Conflict
        );
        assert_eq!(
            validate_revalidated_chat_id(Some(777), Some(888))
                .expect_err("changed revalidated")
                .kind,
            crate::error::AppErrorKind::Conflict
        );
    }
```

- [ ] **Step 6: Implement live validation flow without importing rows**

Replace the scaffold runner body with this call and terminal-state handling:

```rust
match run_takeout_migrated_history_validation_only(&handle, &job_id, batch_id).await {
    Ok(outcome) => {
        if let Some(record) = takeout_state
            .finish_job(&job_id, |job| {
                job.status = STATUS_COMPLETED.to_string();
                job.phase = PHASE_COMPLETED.to_string();
                job.message = Some("Migrated history validation completed.".to_string());
                job.inserted = outcome.inserted;
                job.skipped = outcome.skipped;
                job.progress_current = outcome.progress_total;
                job.progress_total = outcome.progress_total;
                job.warnings = outcome.warnings;
            })
            .await
        {
            emit_takeout_import_event(&handle, &record);
        }
    }
    Err(error) => {
        let terminal_error = error.to_string();
        finalize_terminal_batch_best_effort(
            &handle,
            batch_id,
            TerminalBatchStatus::Failed,
            Some(&terminal_error),
        )
        .await;
        if let Some(record) = takeout_state
            .finish_job(&job_id, |job| {
                job.status = STATUS_FAILED.to_string();
                job.phase = PHASE_FAILED.to_string();
                job.message = None;
                job.error = Some(terminal_error.clone());
            })
            .await
        {
            emit_takeout_import_event(&handle, &record);
        }
    }
}
```

Add:

```rust
async fn run_takeout_migrated_history_validation_only(
    handle: &AppHandle,
    job_id: &str,
    batch_id: i64,
) -> AppResult<TakeoutImportOutcome> {
    let takeout_state = handle.state::<TakeoutImportState>();
    let telegram_state = handle.state::<TelegramState>();
    let repair_state = handle.state::<SourceIdentityRepairState>();
    require_source_identity_ready(repair_state.inner()).await?;
    let pool = get_pool(handle).await?;
    let source_id = takeout_state
        .update_job(job_id, |_| {})
        .await
        .ok_or_else(|| AppError::internal(format!("Takeout job {job_id} not found")))?
        .source_id;
    let source = load_source(&pool, source_id).await?;
    let capability = migrated_history::load_migrated_history_capability(&pool, source_id)
        .await?
        .ok_or_else(migrated_history::not_detected_error)?;
    let expected_chat_id = capability.migrated_from_chat_id;
    let account_id = source.account_id.ok_or_else(|| {
        AppError::validation(format!("Source {} is not linked to an account", source.id))
    })?;
    let runtime = get_authorized_runtime(&telegram_state, account_id).await?;
    let client = runtime.client;
    let session = runtime.session;

    update_and_emit(handle, &takeout_state, job_id, |job| {
        job.phase = PHASE_RESOLVING_SOURCE.to_string();
        job.message = Some("Resolving Telegram source.".to_string());
    })
    .await;
    let resolved_peer =
        resolve_and_refresh_peer(handle, &pool, &client, &source, account_id).await?;
    let (resolved_peer_kind, resolved_peer_id) = peer_ref_identity(resolved_peer.peer);
    update_takeout_resolved_peer(
        &pool,
        batch_id,
        resolved_peer_kind,
        resolved_peer_id,
        "chat",
        expected_chat_id.unwrap_or_default(),
    )
    .await?;

    update_and_emit(handle, &takeout_state, job_id, |job| {
        job.phase = PHASE_STARTING_TAKEOUT.to_string();
        job.message = Some("Starting Takeout session.".to_string());
    })
    .await;
    let alias = prepare_export_dc_alias(&session).await?;
    let init_request = takeout_init_request_for_source_subtype(TELEGRAM_KIND_GROUP)?;
    let mut warnings = Vec::new();
    let mut fallback_used = false;
    let mut export_attempts = ExportDcAttemptState::new();
    let takeout = export_dc_invoke_with_provenance(
        &pool,
        batch_id,
        &client,
        &alias,
        &init_request,
        &mut warnings,
        &mut fallback_used,
        &mut export_attempts,
    )
    .await?;
    let tl::enums::account::Takeout::Takeout(takeout) = takeout;
    let takeout_id = takeout.id;
    update_takeout_session_started(&pool, batch_id, takeout_id).await?;

    update_and_emit(handle, &takeout_state, job_id, |job| {
        job.phase = PHASE_VALIDATING_PEER.to_string();
        job.message = Some("Revalidating migrated history availability.".to_string());
    })
    .await;
    let revalidated_chat_id = revalidate_migrated_from_chat_id(
        &pool,
        batch_id,
        &client,
        &alias,
        takeout_id,
        resolved_peer.peer,
        &mut warnings,
        &mut fallback_used,
        &mut export_attempts,
    )
    .await?;
    let validation =
        migrated_history::validate_revalidated_chat_id(expected_chat_id, revalidated_chat_id)?;
    migrated_history::upsert_migrated_history_available(
        &pool,
        source_id,
        validation.migrated_from_chat_id,
        crate::sources::now_secs(),
    )
    .await?;

    let fallback_before = fallback_used;
    record_export_dc_attempt_if_needed(&pool, &alias, &mut export_attempts).await?;
    finish_takeout_session(&client, &alias, takeout_id, true, &mut warnings, &mut fallback_used)
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
    mark_takeout_migrated_history_imported(&pool, batch_id).await?;
    finalize_ingest_batch(&pool, batch_id, TerminalBatchStatus::Completed, None).await?;

    Ok(TakeoutImportOutcome {
        inserted: 0,
        skipped: 0,
        progress_total: Some(0),
        warnings,
    })
}
```

Fix the exact `record_export_dc_attempt_if_needed` call signature to include `batch_id`:

```rust
record_export_dc_attempt_if_needed(&pool, batch_id, &alias, &mut export_attempts).await?;
```

Add `revalidate_migrated_from_chat_id`:

```rust
async fn revalidate_migrated_from_chat_id(
    pool: &sqlx::Pool<sqlx::Sqlite>,
    batch_id: i64,
    client: &Client,
    alias: &ExportDcAlias,
    takeout_id: i64,
    peer: grammers_session::types::PeerRef,
    warnings: &mut Vec<String>,
    fallback_used: &mut bool,
    export_attempts: &mut ExportDcAttemptState,
) -> AppResult<Option<i64>> {
    let input_channel: tl::enums::InputChannel = peer.into();
    let chat_full = export_dc_invoke_with_provenance(
        pool,
        batch_id,
        client,
        alias,
        &tl::functions::InvokeWithTakeout {
            takeout_id,
            query: tl::functions::channels::GetFullChannel {
                channel: input_channel,
            },
        },
        warnings,
        fallback_used,
        export_attempts,
    )
    .await?;

    let tl::enums::messages::ChatFull::Full(chat_full) = chat_full;
    if let tl::enums::ChatFull::ChannelFull(full) = chat_full.full_chat {
        return Ok(full.migrated_from_chat_id);
    }
    Ok(None)
}
```

On validation conflict in the runner, call `mark_migrated_history_unavailable` with `MIGRATED_HISTORY_REASON_REVALIDATION_FAILED` before finalizing failed.

- [ ] **Step 7: Run focused tests and compile check**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml migrated_history_errors_are_typed_for_frontend_behavior
cargo test --manifest-path src-tauri\Cargo.toml validation_rejects_missing_or_changed_revalidated_chat_id
cargo test --manifest-path src-tauri\Cargo.toml migrated_history_detected_warning_is_sanitized
cargo check --manifest-path src-tauri\Cargo.toml
```

Expected: tests pass and `cargo check` exits 0.

- [ ] **Step 8: Commit Task 5**

Run:

```powershell
git add src-tauri\src\takeout_import\migrated_history.rs src-tauri\src\takeout_import\mod.rs docs\superpowers\plans\2026-05-27-takeout-migrated-history-opt-in.md
git commit -m "feat: validate migrated history import scope"
```

Expected: commit succeeds.

---

### Task 6: Historical Row Writes Under Native Old Chat Identity

**Files:**
- Modify: `src-tauri/src/takeout_import/mod.rs`
- Modify: `src-tauri/src/takeout_import/migrated_history.rs`
- Modify: `src-tauri/src/sources/items.rs`

- [ ] **Step 1: Add identity override unit test**

In `src-tauri/src/takeout_import/migrated_history.rs`, add:

```rust
pub(crate) fn migrated_small_group_identity(
    telegram_message_id: i64,
    migrated_from_chat_id: i64,
) -> crate::sources::TelegramMessageIdentity {
    crate::sources::TelegramMessageIdentity {
        history_peer_kind: crate::telegram::peer::TELEGRAM_PEER_KIND_CHAT.to_string(),
        history_peer_id: migrated_from_chat_id,
        telegram_message_id,
        migration_domain: Some(
            crate::sources::types::TELEGRAM_MIGRATION_DOMAIN_MIGRATED_FROM_CHAT.to_string(),
        ),
        is_migrated_history: true,
    }
}
```

Use the literal `"chat"` if there is no exported peer-kind constant in the module where this helper lives.

Add test:

```rust
    #[test]
    fn migrated_small_group_identity_uses_native_old_chat_scope() {
        let identity = migrated_small_group_identity(42, 777);

        assert_eq!(identity.history_peer_kind, "chat");
        assert_eq!(identity.history_peer_id, 777);
        assert_eq!(identity.telegram_message_id, 42);
        assert_eq!(identity.migration_domain.as_deref(), Some("migrated_from_chat"));
        assert!(identity.is_migrated_history);
    }
```

- [ ] **Step 2: Run the identity test**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml migrated_small_group_identity_uses_native_old_chat_scope
```

Expected: pass after the helper is added.

- [ ] **Step 3: Add importer idempotency storage test**

In `src-tauri/src/sources/items.rs`, add:

```rust
    #[tokio::test]
    async fn migrated_insert_idempotency_uses_old_chat_native_identity() {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_analysis_documents_table(&pool).await;
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
        let identity = crate::takeout_import::migrated_history::migrated_small_group_identity(
            42, 777,
        );

        let inserted = insert_telegram_source_item_with_observation_in_context(
            &pool,
            batch_id,
            1,
            identity.clone(),
            telegram_insert("42", "historical"),
            TelegramInsertContext::MigratedSmallGroupHistory,
        )
        .await
        .expect("insert");
        let item_id = match inserted {
            TelegramItemInsertOutcome::Inserted { item_id } => item_id,
            other => panic!("expected insert, got {other:?}"),
        };

        let duplicate = insert_telegram_source_item_with_observation_in_context(
            &pool,
            batch_id,
            1,
            identity,
            telegram_insert("42", "historical duplicate"),
            TelegramInsertContext::MigratedSmallGroupHistory,
        )
        .await
        .expect("duplicate");

        assert_eq!(duplicate, TelegramItemInsertOutcome::DuplicateObserved { item_id });

        let row: (String, i64, i64, String, i64) = sqlx::query_as(
            "SELECT history_peer_kind, history_peer_id, telegram_message_id,
                    migration_domain, is_migrated_history
             FROM telegram_messages
             WHERE item_id = ?",
        )
        .bind(item_id)
        .fetch_one(&pool)
        .await
        .expect("load telegram row");

        assert_eq!(
            row,
            ("chat".to_string(), 777, 42, "migrated_from_chat".to_string(), 1)
        );
    }
```

- [ ] **Step 4: Run idempotency storage test**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml migrated_insert_idempotency_uses_old_chat_native_identity
```

Expected: pass after Task 3 and Step 1 are in place.

- [ ] **Step 5: Replace validation-only historical runner with import runner**

In `src-tauri/src/takeout_import/mod.rs`, replace `run_takeout_migrated_history_validation_only` with `run_takeout_migrated_history_import`. Keep the validation code, then build:

```rust
let input_peer = tl::enums::InputPeer::Chat(tl::types::InputPeerChat {
    chat_id: validation.migrated_from_chat_id,
});
```

After validation, call the same split loading, counting, and page import helpers used by current Takeout, but pass historical mode:

```rust
let historical_import = import_takeout_history_ranges(
    handle,
    job_id,
    batch_id,
    &client,
    &alias,
    takeout_id,
    input_peer,
    counted_ranges,
    &source,
    total,
    TELEGRAM_KIND_GROUP,
    warnings,
    fallback_used,
    export_attempts,
    &mut only_my_messages_recorded,
    Some(validation.migrated_from_chat_id),
)
.await?;
```

Change `import_takeout_history_ranges` and `import_takeout_history_pages` signatures to accept:

```rust
    migrated_from_chat_id: Option<i64>,
```

For current Takeout calls, pass `None`.

- [ ] **Step 6: Override identity and insert context in page import**

In `import_takeout_history_pages`, after parsing a message, replace:

```rust
                    let identity = item.telegram_identity.clone().ok_or_else(|| {
```

with:

```rust
                    let parsed_identity = item.telegram_identity.clone().ok_or_else(|| {
```

Then add:

```rust
                    let (identity, insert_context) = if let Some(migrated_from_chat_id) =
                        migrated_from_chat_id
                    {
                        (
                            migrated_history::migrated_small_group_identity(
                                parsed_identity.telegram_message_id,
                                migrated_from_chat_id,
                            ),
                            crate::sources::TelegramInsertContext::MigratedSmallGroupHistory,
                        )
                    } else {
                        (parsed_identity, crate::sources::TelegramInsertContext::CurrentHistory)
                    };
```

Replace `insert_telegram_source_item_with_observation` with:

```rust
                    match crate::sources::insert_telegram_source_item_with_observation_in_context(
                        &pool,
                        batch_id,
                        source.id,
                        identity,
                        item,
                        insert_context,
                    )
                    .await?
```

- [ ] **Step 7: Prevent current watermark and topic refresh for historical import**

In the historical import runner, after successful `finish_takeout_session(success=true)`, do:

```rust
mark_takeout_migrated_history_imported(&pool, batch_id).await?;
finalize_ingest_batch(&pool, batch_id, TerminalBatchStatus::Completed, None).await?;
```

Do not call:

- `finalize_sync`
- `refresh_forum_topics_after_completed_takeout`

Set the completion message:

```rust
"Migrated history import completed. Inserted {}, skipped {}."
```

- [ ] **Step 8: Run focused tests and check**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml migrated_small_group_identity_uses_native_old_chat_scope
cargo test --manifest-path src-tauri\Cargo.toml migrated_insert_idempotency_uses_old_chat_native_identity
cargo test --manifest-path src-tauri\Cargo.toml migrated_small_group_insert_skips_current_history_derived_writes
cargo check --manifest-path src-tauri\Cargo.toml
```

Expected: all pass.

- [ ] **Step 9: Commit Task 6**

Run:

```powershell
git add src-tauri\src\takeout_import\mod.rs src-tauri\src\takeout_import\migrated_history.rs src-tauri\src\sources\items.rs docs\superpowers\plans\2026-05-27-takeout-migrated-history-opt-in.md
git commit -m "feat: import migrated history rows explicitly"
```

Expected: commit succeeds.

---

### Task 7: Failure, Cancellation, Watermarks, And Read-Model Safety

**Files:**
- Modify: `src-tauri/src/takeout_import/mod.rs`
- Modify: `src-tauri/src/takeout_import/recovery.rs`
- Modify: `src-tauri/src/notebooklm_export/query.rs`
- Modify: `src-tauri/src/analysis/corpus.rs`
- Modify: `src-tauri/src/sources/items/query.rs`
- Modify: `src-tauri/src/archive_read_model.rs`
- Modify: `src-tauri/src/analysis_documents.rs`

- [ ] **Step 1: Add provenance/watermark test**

In `src-tauri/src/takeout_import/mod.rs`, add a pure storage regression that does not need live Telegram:

```rust
    #[tokio::test]
    async fn historical_batch_completion_does_not_advance_source_watermark() {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_ingest_provenance_tables(&pool).await;
        seed_takeout_source(&pool, 1, 10, "supergroup", "channel", 12345).await;
        sqlx::query("UPDATE sources SET last_sync_state = 99, last_synced_at = 1000 WHERE id = 1")
            .execute(&pool)
            .await
            .expect("seed watermark");
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

        crate::ingest_provenance::mark_takeout_migrated_small_group_scope(&pool, batch_id)
            .await
            .expect("mark scope");
        crate::ingest_provenance::mark_takeout_migrated_history_imported(&pool, batch_id)
            .await
            .expect("mark imported");
        crate::ingest_provenance::finalize_ingest_batch(
            &pool,
            batch_id,
            crate::ingest_provenance::TerminalBatchStatus::Completed,
            None,
        )
        .await
        .expect("finalize");

        let watermark: (Option<i64>, Option<i64>) =
            sqlx::query_as("SELECT last_sync_state, last_synced_at FROM sources WHERE id = 1")
                .fetch_one(&pool)
                .await
                .expect("load watermark");
        let batch: (String, String, i64) = sqlx::query_as(
            "SELECT b.status, t.history_scope, t.migrated_history_imported
             FROM ingest_batches b
             JOIN telegram_takeout_batches t ON t.batch_id = b.id
             WHERE b.id = ?",
        )
        .bind(batch_id)
        .fetch_one(&pool)
        .await
        .expect("load batch");

        assert_eq!(watermark, (Some(99), Some(1000)));
        assert_eq!(
            batch,
            (
                "completed".to_string(),
                "migrated_small_group_history".to_string(),
                1,
            )
        );
    }
```

- [ ] **Step 2: Add recovery visibility test**

In `src-tauri/src/takeout_import/recovery.rs`, add a test proving failed historical batches are listed with `history_scope = migrated_small_group_history` in diagnostics:

```rust
    #[tokio::test]
    async fn recovery_state_includes_migrated_history_scope_for_historical_batches() {
        let pool = memory_pool_with_source_items_and_topics().await;
        create_ingest_provenance_tables(&pool).await;
        seed_recovery_source(&pool, 1).await;
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
        mark_takeout_migrated_small_group_scope(&pool, batch_id)
            .await
            .expect("mark scope");
        finalize_ingest_batch(
            &pool,
            batch_id,
            TerminalBatchStatus::Failed,
            Some("migrated_history_unavailable"),
        )
        .await
        .expect("finalize failed");

        let states = list_takeout_import_recovery_states_for_sources(&pool, &TakeoutImportState::new(), None)
            .await
            .expect("list recovery");

        assert_eq!(states[0].history_scope, "migrated_small_group_history");
    }
```

Add `pub history_scope: String` to Rust and TypeScript recovery DTOs and map `t.history_scope` from the recovery query.

- [ ] **Step 3: Add default consumer safety tests**

Add one focused test per default consumer:

In `src-tauri/src/sources/items/query.rs`:

```rust
    #[tokio::test]
    async fn default_source_browsing_does_not_surface_migrated_rows_after_archive_ready() {
        let pool = memory_pool_with_source_items_and_topics().await;
        seed_direct_item(&pool, 1, 10, "10", 1000, "current").await;
        seed_direct_item(&pool, 1, 11, "11", 900, "migrated").await;
        sqlx::query(
            "INSERT INTO telegram_messages (
                item_id, source_id, history_peer_kind, history_peer_id,
                telegram_message_id, migration_domain, is_migrated_history
             ) VALUES (10, 1, 'channel', 12345, 10, NULL, 0),
                      (11, 1, 'chat', 777, 10, 'migrated_from_chat', 1)",
        )
        .execute(&pool)
        .await
        .expect("seed telegram rows");
        crate::archive_read_model::rebuild_source(&pool, 1)
            .await
            .expect("rebuild archive");

        let rows = load_item_rows_from_pool(&pool, 1, 20, None, None, None)
            .await
            .expect("load rows");

        assert_eq!(rows.iter().map(|row| row.id).collect::<Vec<_>>(), vec![10]);
    }
```

In `src-tauri/src/notebooklm_export/query.rs`, add:

```rust
    #[tokio::test]
    async fn notebooklm_default_export_excludes_migrated_history_rows() {
        let pool = memory_pool_with_source_items_and_topics().await;
        seed_notebooklm_export_parity_fixture(&pool).await;
        sqlx::query(
            "INSERT INTO telegram_messages (
                item_id, source_id, history_peer_kind, history_peer_id,
                telegram_message_id, migration_domain, is_migrated_history
             ) VALUES (1, 1, 'channel', 12345, 1, NULL, 0),
                      (2, 1, 'chat', 777, 1, 'migrated_from_chat', 1)",
        )
        .execute(&pool)
        .await
        .expect("seed telegram rows");

        let messages = load_export_messages_from_items_path(&pool, 1, None, None)
            .await
            .expect("load export messages");

        assert!(messages.iter().all(|message| message.item_id != 2));
    }
```

In `src-tauri/src/analysis/corpus.rs`, add:

```rust
    #[tokio::test]
    async fn default_analysis_corpus_excludes_migrated_history_documents() {
        let pool = test_pool().await;
        seed_analysis_source(&pool, 1, "telegram", "supergroup").await;
        crate::sources::test_support::create_analysis_documents_table(&pool).await;
        crate::analysis_documents::rebuild_analysis_documents_for_source(&pool, 1)
            .await
            .expect("rebuild docs");

        let document_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*)
             FROM analysis_documents d
             JOIN telegram_messages tm ON tm.item_id = d.item_id
             WHERE d.source_id = 1 AND tm.is_migrated_history = 1",
        )
        .fetch_one(&pool)
        .await
        .expect("count migrated docs");

        assert_eq!(document_count, 0);
    }
```

Use `snapshot_pool().await` in this module, create `telegram_messages` with the same schema used by `sources::test_support::create_telegram_messages_table`, seed one current Telegram item plus one migrated Telegram item, then rebuild `analysis_documents`.

- [ ] **Step 4: Run safety tests and verify failures**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml historical_batch_completion_does_not_advance_source_watermark
cargo test --manifest-path src-tauri\Cargo.toml recovery_state_includes_migrated_history_scope_for_historical_batches
cargo test --manifest-path src-tauri\Cargo.toml default_source_browsing_does_not_surface_migrated_rows_after_archive_ready
cargo test --manifest-path src-tauri\Cargo.toml notebooklm_default_export_excludes_migrated_history_rows
cargo test --manifest-path src-tauri\Cargo.toml default_analysis_corpus_excludes_migrated_history_documents
```

Expected: watermark test passes, and the recovery, export, corpus, and browsing tests fail until DTO and filter changes in the next steps are applied.

- [ ] **Step 5: Add recovery `history_scope` DTO**

In `src-tauri/src/takeout_import/recovery.rs`, extend the recovery row and DTO:

```rust
    pub history_scope: String,
```

Select `t.history_scope` in recovery queries and map it into `TakeoutImportRecoveryState`.

In `src/lib/types/sources.ts`, extend `TakeoutImportRecoveryState`:

```ts
  history_scope: "current_history" | "current_history_with_migrated_deferred" | "partial_private_history" | "mixed_partial" | "migrated_small_group_history" | "unknown";
```

Update fixtures in frontend tests with `history_scope: "current_history"`.

- [ ] **Step 6: Ensure NotebookLM items-path excludes migrated rows**

In `src-tauri/src/notebooklm_export/query.rs`, add the `NOT EXISTS` migrated-row filter to the items-path export query:

```sql
AND NOT EXISTS (
  SELECT 1 FROM telegram_messages tm
  WHERE tm.item_id = items.id
    AND tm.is_migrated_history = 1
)
```

Also add the same `NOT EXISTS` filter to the archive-path export query through `archive_read_items.item_id`, so manually inserted archive rows cannot bypass the default-domain contract.

- [ ] **Step 7: Run focused tests**

Run:

```powershell
cargo test --manifest-path src-tauri\Cargo.toml historical_batch_completion_does_not_advance_source_watermark
cargo test --manifest-path src-tauri\Cargo.toml recovery_state_includes_migrated_history_scope_for_historical_batches
cargo test --manifest-path src-tauri\Cargo.toml default_source_browsing_does_not_surface_migrated_rows_after_archive_ready
cargo test --manifest-path src-tauri\Cargo.toml notebooklm_default_export_excludes_migrated_history_rows
cargo test --manifest-path src-tauri\Cargo.toml default_analysis_corpus_excludes_migrated_history_documents
```

Expected: all pass.

- [ ] **Step 8: Commit Task 7**

Run:

```powershell
git add src-tauri\src\takeout_import\mod.rs src-tauri\src\takeout_import\recovery.rs src-tauri\src\notebooklm_export\query.rs src-tauri\src\analysis\corpus.rs src-tauri\src\sources\items\query.rs src-tauri\src\archive_read_model.rs src-tauri\src\analysis_documents.rs src\lib\types\sources.ts docs\superpowers\plans\2026-05-27-takeout-migrated-history-opt-in.md
git commit -m "test: protect default reads from migrated history"
```

Expected: commit succeeds.

---

### Task 8: Frontend Explicit Historical Action

**Files:**
- Modify: `src/lib/analysis-state.ts`
- Modify: `src/lib/analysis-state.test.ts`
- Modify: `src/lib/components/analysis/source-switcher-panel.svelte`
- Modify: `src/lib/components/analysis/compact-source-rail.svelte`
- Modify: `src/routes/analysis/+page.svelte`

- [ ] **Step 1: Add UX policy tests**

In `src/lib/analysis-state.test.ts`, add:

```ts
  it("labels migrated history action as historical import, not retry or sync", () => {
    expect(migratedHistoryActionLabel()).toBe("Import migrated history");
    expect(migratedHistoryActionLabel().toLowerCase()).not.toContain("retry");
    expect(migratedHistoryActionLabel().toLowerCase()).not.toContain("sync");
  });

  it("enables migrated history action only for available capability without active source work", () => {
    const source = sourceRecord({
      sourceType: "telegram",
      sourceSubtype: "supergroup",
      migratedHistoryStatus: "available",
    });

    expect(migratedHistoryActionDisabledReason(source, false, false, false)).toBeNull();
    expect(migratedHistoryActionDisabledReason(source, true, false, false)).toBe(
      "Source job is active.",
    );
    expect(migratedHistoryActionDisabledReason(source, false, true, false)).toBe(
      "Takeout import is active.",
    );
    expect(migratedHistoryActionDisabledReason(source, false, false, true)).toBe(
      "Starting migrated history import.",
    );
    expect(
      migratedHistoryActionDisabledReason(
        sourceRecord({ migratedHistoryStatus: "none" }),
        false,
        false,
        false,
      ),
    ).toBe("Migrated history is not available.");
  });
```

Use the existing `sourceRecord(...)` helper in this file.

- [ ] **Step 2: Run tests and verify failure**

Run:

```powershell
npm.cmd test -- src/lib/analysis-state.test.ts
```

Expected: fail because the helper functions do not exist.

- [ ] **Step 3: Add UX helper functions**

In `src/lib/analysis-state.ts`, add:

```ts
import type { Source } from "$lib/types/sources";

export function migratedHistoryActionLabel() {
  return "Import migrated history";
}

export function migratedHistoryActionDisabledReason(
  source: Source,
  sourceJobActive: boolean,
  takeoutActive: boolean,
  startingHistoricalImport: boolean,
) {
  if (source.migratedHistoryStatus !== "available") {
    return "Migrated history is not available.";
  }
  if (sourceJobActive) return "Source job is active.";
  if (takeoutActive) return "Takeout import is active.";
  if (startingHistoricalImport) return "Starting migrated history import.";
  return null;
}
```

`Source` is already imported in this file; keep that existing import.

- [ ] **Step 4: Wire route state**

In `src/routes/analysis/+page.svelte`, import:

```ts
import { startTakeoutMigratedHistoryImport } from "$lib/api/takeout-import";
```

Add state:

```ts
let startingMigratedHistorySourceIds = $state<Record<number, boolean>>({});
```

Add handler:

```ts
async function handleStartMigratedHistoryImport(sourceId: number) {
  startingMigratedHistorySourceIds = { ...startingMigratedHistorySourceIds, [sourceId]: true };
  try {
    await startTakeoutMigratedHistoryImport(sourceId);
  } catch (error) {
    status = formatAppError("starting migrated history import", error);
  } finally {
    const next = { ...startingMigratedHistorySourceIds };
    delete next[sourceId];
    startingMigratedHistorySourceIds = next;
  }
}
```

Pass to source panels:

```svelte
{startingMigratedHistorySourceIds}
onStartMigratedHistoryImport={handleStartMigratedHistoryImport}
```

- [ ] **Step 5: Render the action in `source-switcher-panel.svelte`**

Add props:

```ts
    startingMigratedHistorySourceIds: Record<number, boolean>;
    onStartMigratedHistoryImport: (sourceId: number) => void;
```

Import helpers:

```ts
import {
  migratedHistoryActionDisabledReason,
  migratedHistoryActionLabel,
} from "$lib/analysis-state";
```

Near the existing Takeout button, add:

```svelte
                {#if source.migratedHistoryStatus === "available"}
                  {@const migratedHistoryReason = migratedHistoryActionDisabledReason(
                    source,
                    sourceJobActive,
                    takeoutActive,
                    !!startingMigratedHistorySourceIds[source.id],
                  )}
                  <button
                    type="button"
                    class="secondary-action"
                    onclick={() => onStartMigratedHistoryImport(source.id)}
                    disabled={migratedHistoryReason !== null}
                    title={migratedHistoryReason ?? undefined}
                  >
                    {startingMigratedHistorySourceIds[source.id]
                      ? "Starting historical import..."
                      : migratedHistoryActionLabel()}
                  </button>
                {/if}
```

Use the same button class as the existing Takeout action in this component.

- [ ] **Step 6: Propagate through compact rail**

`src/lib/components/analysis/compact-source-rail.svelte` renders `SourceSwitcherPanel`; add the same props and pass them through unchanged:

```svelte
      {startingMigratedHistorySourceIds}
      {onStartMigratedHistoryImport}
```

- [ ] **Step 7: Add route/component tests**

In the existing route/component tests that cover source action placement, add one expectation:

```ts
expect(rendered.text()).toContain("Import migrated history");
expect(rendered.text()).not.toContain("Retry migrated history");
expect(rendered.text()).not.toContain("Sync migrated history");
```

Use the established test renderer in `src/lib/analysis-source-access-placement.test.ts` or `src/lib/analysis-compact-source-rail.test.ts`.

- [ ] **Step 8: Run frontend tests**

Run:

```powershell
npm.cmd test -- src/lib/analysis-state.test.ts src/lib/api/takeout-import.test.ts src/lib/api/sources.test.ts src/lib/analysis-source-access-placement.test.ts src/lib/analysis-compact-source-rail.test.ts
npm.cmd run check
```

Expected: all listed tests pass and `svelte-check` exits 0.

- [ ] **Step 9: Commit Task 8**

Run:

```powershell
git add src/lib/analysis-state.ts src/lib/analysis-state.test.ts src/lib/components/analysis/source-switcher-panel.svelte src/lib/components/analysis/compact-source-rail.svelte src/routes/analysis/+page.svelte docs\superpowers\plans\2026-05-27-takeout-migrated-history-opt-in.md
git commit -m "feat: add explicit migrated history action"
```

Expected: commit succeeds.

---

### Task 9: Documentation And Final Verification

**Files:**
- Modify: `docs/takeout-source-import.md`
- Modify: `docs/database-schema.md`
- Modify: `docs/backlog.md`
- Modify: `docs/superpowers/plans/2026-05-27-takeout-migrated-history-opt-in.md`

- [ ] **Step 1: Update Takeout docs**

In `docs/takeout-source-import.md`, add a section named `Migrated Small-Group History Opt-In` with these points:

```markdown
## Migrated Small-Group History Opt-In

Normal Takeout imports current source history only. When a supergroup exposes a
`migrated_from_chat_id`, Extractum records source-level availability and keeps
the old small-group history deferred.

The explicit command is:

- `start_takeout_migrated_history_import(source_id) -> { job_id }`

The command uses the same source ingest lock as sync, delete, and current
Takeout. It revalidates the current supergroup before opening the old
`InputPeerChat`. If revalidation fails, the backend returns a typed
`migrated_history_unavailable` conflict and records an internal availability
reason.

Historical import writes rows with native old chat identity:

- `history_peer_kind = chat`
- `history_peer_id = migrated_from_chat_id`
- `is_migrated_history = 1`
- `migration_domain = migrated_from_chat`

Historical import does not update `sources.last_sync_state` or
`sources.last_synced_at`. The first implementation does not materialize
historical rows into `analysis_documents`, `archive_read_items`, or
`item_topic_memberships`, and default browsing, analysis, reports, and
NotebookLM export stay current-history-only.
```

- [ ] **Step 2: Update database schema docs**

In `docs/database-schema.md`, document:

```markdown
### `telegram_migrated_history_capabilities`

Source-level Telegram capability state for explicit migrated small-group
history import. The table is keyed by `source_id` and stores private old-chat
access hints separately from `telegram_sources`.

Allowed `status` values:

- `none`
- `available`
- `unavailable`

Allowed `unavailable_reason` values are internal diagnostics:

- `not_detected`
- `missing_migrated_from_chat_id`
- `current_source_unavailable`
- `old_chat_input_unavailable`
- `revalidation_failed`

Frontend source records expose only sanitized availability status and
timestamps. They do not expose `migrated_from_chat_id`.
```

Also update the `telegram_messages` section:

```markdown
`migration_domain` is a row-level marker. The first functional value is
`migrated_from_chat`; it marks rows imported from a supergroup's old small-group
history. It is not part of the primary duplicate identity.
```

And update the `telegram_takeout_batches` section:

```markdown
`history_scope = migrated_small_group_history` identifies an explicit
historical import batch. It is a run-level scope, not a row-level migration
domain.
```

- [ ] **Step 3: Update backlog**

In `docs/backlog.md`, under `3.1 Takeout Source Import Follow-Ups`, add a completed entry:

```markdown
- [x] implement explicit migrated small-group history opt-in import as a
  separate Takeout action with source-level capability state, native old-chat
  identity, same-source locking, and default current-history read behavior
```

Keep any future merged timeline or domain-aware analysis/export work open.

- [ ] **Step 4: Run full backend verification**

Run:

```powershell
cargo check --manifest-path src-tauri\Cargo.toml
cargo test --manifest-path src-tauri\Cargo.toml
```

Expected: both commands exit 0.

- [ ] **Step 5: Run full frontend verification**

Run:

```powershell
npm.cmd test
npm.cmd run check
```

Expected: both commands exit 0.

- [ ] **Step 6: Run formatting and whitespace checks**

Run:

```powershell
git diff --check
```

Expected: no output and exit 0.

- [ ] **Step 7: Commit Task 9**

Run:

```powershell
git add docs\takeout-source-import.md docs\database-schema.md docs\backlog.md docs\superpowers\plans\2026-05-27-takeout-migrated-history-opt-in.md
git commit -m "docs: document migrated history opt-in import"
```

Expected: commit succeeds.

---

## Final Acceptance Checklist

- [ ] Normal `start_takeout_source_import` still records migrated history as deferred and does not import old `chat` rows.
- [ ] `start_takeout_migrated_history_import` exists as a separate command.
- [ ] Historical import uses the same same-source ingest lock as sync, delete, and current Takeout.
- [ ] Source-level capability survives restart in `telegram_migrated_history_capabilities`.
- [ ] Frontend sees sanitized status only and never receives `migrated_from_chat_id`.
- [ ] `unavailable` has an internal reason code and a typed frontend-safe error.
- [ ] Migrated rows use `chat` native identity and `migration_domain = migrated_from_chat`.
- [ ] Native duplicate identity remains `(source_id, history_peer_kind, history_peer_id, telegram_message_id)`.
- [ ] Re-running historical import observes duplicates without inserting extra rows.
- [ ] Duplicate-only successful reruns set `migrated_history_imported = 1`.
- [ ] Historical import does not update `last_sync_state` or `last_synced_at`.
- [ ] Historical rows do not create `analysis_documents`.
- [ ] Historical rows do not create `archive_read_items`.
- [ ] Historical rows do not create `item_topic_memberships`.
- [ ] Default browsing excludes historical rows from items path and archive path.
- [ ] Default analysis corpus, reports, and NotebookLM export exclude historical rows.
- [ ] UI action text says `Import migrated history` and does not say retry or sync.
- [ ] `cargo check --manifest-path src-tauri\Cargo.toml` passes.
- [ ] `cargo test --manifest-path src-tauri\Cargo.toml` passes.
- [ ] `npm.cmd test` passes.
- [ ] `npm.cmd run check` passes.
- [ ] `git diff --check` passes.
